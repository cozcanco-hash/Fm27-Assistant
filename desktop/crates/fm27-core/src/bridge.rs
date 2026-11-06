//! Authenticated, loopback-only, bounded JSON-line transport for the game plugin.
use crate::{
    assistant::{Assistant, SendRequest},
    conversation::Client,
    Error, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{oneshot, Semaphore},
    task::{JoinHandle, JoinSet},
};

pub const PROTOCOL: u32 = 1;
pub const MAX_REQUEST: usize = 16 * 1024;
pub const MAX_RESPONSE: usize = 3 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub protocol: u32,
    pub address: String,
    pub port: u16,
    pub token: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    protocol: u32,
    token: String,
    method: String,
    #[serde(default)]
    params: Value,
}

pub struct BridgeServer {
    stop: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
    _lease: File,
    descriptor_path: PathBuf,
    token: String,
    enabled: Arc<AtomicBool>,
}

impl BridgeServer {
    pub async fn start(host: Arc<Assistant>, data_dir: &Path) -> Result<Self> {
        let directory = data_dir.join("in-game");
        std::fs::create_dir_all(&directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if std::fs::symlink_metadata(&directory)?
                .file_type()
                .is_symlink()
            {
                return Err(Error::Invalid(
                    "Eklenti bağlantı klasörü bir bağlantı olamaz.".into(),
                ));
            }
            std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lease = options.open(directory.join("host.lock"))?;
        lease.try_lock().map_err(|_| {
            Error::Invalid("Oyun içi bağlantı başka bir FM Assistant penceresinde açık.".into())
        })?;
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
        let mut random = [0u8; 32];
        getrandom::fill(&mut random)
            .map_err(|_| Error::Invalid("Güvenli yerel bağlantı oluşturulamadı.".into()))?;
        let token: String = random.iter().map(|b| format!("{b:02x}")).collect();
        let descriptor = Descriptor {
            protocol: PROTOCOL,
            address: "127.0.0.1".into(),
            port: listener.local_addr()?.port(),
            token: token.clone(),
        };
        let temporary = directory.join(format!("bridge-{}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let write = (|| -> Result<()> {
            let mut file = options.open(&temporary)?;
            file.write_all(&serde_json::to_vec(&descriptor)?)?;
            file.sync_all()?;
            std::fs::rename(&temporary, directory.join("bridge.json"))?;
            Ok(())
        })();
        if write.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        write?;
        let enabled = Arc::new(AtomicBool::new(true));
        let (stop, mut stopped) = oneshot::channel();
        let task_token = token.clone();
        let task_enabled = enabled.clone();
        let task = tokio::spawn(async move {
            let slots = Arc::new(Semaphore::new(8));
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stopped => break,
                    Some(_) = connections.join_next(), if !connections.is_empty() => {},
                    accepted = listener.accept() => {
                        let Ok((stream,address)) = accepted else { break; };
                        if !address.ip().is_loopback() { continue; }
                        let Ok(slot) = slots.clone().try_acquire_owned() else { continue; };
                        let host = host.clone(); let token = task_token.clone(); let enabled = task_enabled.clone();
                        connections.spawn(async move {
                            let _slot = slot;
                            let _ = tokio::time::timeout(Duration::from_secs(10),serve(stream,host,&token,enabled)).await;
                        });
                    }
                }
            }
            connections.abort_all();
            while connections.join_next().await.is_some() {}
        });
        Ok(Self {
            stop: Some(stop),
            task,
            _lease: lease,
            descriptor_path: directory.join("bridge.json"),
            token,
            enabled,
        })
    }

    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }

    pub async fn stop(mut self) {
        self.enabled.store(false, Ordering::Release);
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        let _ = (&mut self.task).await;
    }
}

impl Drop for BridgeServer {
    fn drop(&mut self) {
        self.enabled.store(false, Ordering::Release);
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        self.task.abort();
        // Never remove another instance's discovery file.
        if let Ok(file) = File::open(&self.descriptor_path) {
            let mut bytes = Vec::new();
            if file.take(4097).read_to_end(&mut bytes).is_err() {
                return;
            }
            if bytes.len() <= 4096
                && serde_json::from_slice::<Descriptor>(&bytes).is_ok_and(|d| d.token == self.token)
            {
                let _ = std::fs::remove_file(&self.descriptor_path);
            }
        }
    }
}

async fn serve(
    stream: TcpStream,
    host: Arc<Assistant>,
    token: &str,
    enabled: Arc<AtomicBool>,
) -> Result<()> {
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let mut bytes = Vec::new();
    loop {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() {
            return Ok(());
        }
        let end = buffer.iter().position(|b| *b == b'\n');
        let take = end.map(|i| i + 1).unwrap_or(buffer.len());
        if bytes.len() + take > MAX_REQUEST {
            return Ok(());
        }
        bytes.extend_from_slice(&buffer[..take]);
        reader.consume(take);
        if end.is_some() {
            break;
        }
    }
    let response = match serde_json::from_slice::<Request>(&bytes) {
        Ok(request)
            if request.protocol == PROTOCOL
                && authorized(&request.token, token)
                && enabled.load(Ordering::Acquire) =>
        {
            match dispatch(host, request).await {
                Ok(value) => json!({"protocol":PROTOCOL,"ok":true,"result":value}),
                Err(message) => json!({"protocol":PROTOCOL,"ok":false,"error":message}),
            }
        }
        _ => {
            json!({"protocol":PROTOCOL,"ok":false,"error":"Bağlantı doğrulanamadı. Masaüstü uygulamasındaki bağlantıyı kontrol et."})
        }
    };
    let mut bytes = serde_json::to_vec(&response)?;
    if bytes.len() > MAX_RESPONSE {
        return Err(Error::Invalid("Yanıt boyutu aşıldı.".into()));
    }
    bytes.push(b'\n');
    write.write_all(&bytes).await?;
    Ok(())
}

fn authorized(given: &str, expected: &str) -> bool {
    if given.len() != expected.len() {
        return false;
    }
    given
        .bytes()
        .zip(expected.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

async fn dispatch(host: Arc<Assistant>, request: Request) -> std::result::Result<Value, String> {
    match request.method.as_str() {
        "state" => serde_json::to_value(host.conversation().await.map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string()),
        "send" => {
            let send: SendRequest = serde_json::from_value(request.params)
                .map_err(|_| "Geçersiz mesaj.".to_string())?;
            serde_json::to_value(
                host.send(send, Client::Game)
                    .await
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())
        }
        "cancel" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Cancel {
                turn_id: i64,
            }
            let cancel: Cancel = serde_json::from_value(request.params)
                .map_err(|_| "Geçersiz yanıt kimliği.".to_string())?;
            host.cancel(cancel.turn_id).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "setModel" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Model {
                scope: String,
                model: Option<String>,
            }
            let model: Model = serde_json::from_value(request.params)
                .map_err(|_| "Geçersiz model.".to_string())?;
            serde_json::to_value(
                host.set_model(model.scope, model.model)
                    .await
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())
        }
        _ => Err("Bu işlem oyun içi bağlantıda desteklenmiyor.".into()),
    }
}
