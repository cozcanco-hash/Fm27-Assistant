//! The Rust host owns the conversation; frontends never supply their own history.
use crate::{
    analysis::prompt_context,
    codex::{ChatEvent, CodexBridge, ModelChoice, Status},
    conversation::{bounded_text, validate_prompt, Client, Message},
    Error, Result, Source, Store,
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

struct ActiveTurn {
    id: i64,
    scope: String,
    live: String,
    cancelled: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub scope: String,
    pub club: String,
    pub source: Source,
    pub captured_at: String,
    pub model: Option<String>,
    pub messages: Vec<Message>,
    pub busy: bool,
    pub live: String,
    pub active_turn_id: Option<i64>,
    pub models: Vec<ModelChoice>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendRequest {
    pub scope: String,
    pub prompt: String,
}

pub struct Assistant {
    pub store: Arc<Mutex<Store>>,
    pub codex: Arc<CodexBridge>,
    active: Mutex<Option<ActiveTurn>>,
    gate: Arc<tokio::sync::Mutex<()>>,
    models: Mutex<Vec<ModelChoice>>,
    account: Mutex<Option<Status>>,
    turn_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    error: Mutex<Option<String>>,
    _lease: Option<std::fs::File>,
}

impl Assistant {
    pub fn open(data_dir: &Path) -> Result<Arc<Self>> {
        std::fs::create_dir_all(data_dir)?;
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lease = options.open(data_dir.join("assistant.lock"))?;
        lease.try_lock().map_err(|_| {
            Error::Invalid("Bu çalışma alanı başka bir FM Assistant uygulamasında açık.".into())
        })?;
        let mut host = Self::new(
            Store::open(&data_dir.join("assistant.sqlite"))?,
            CodexBridge::new(data_dir.join("codex-workspace")),
        );
        Arc::get_mut(&mut host).unwrap()._lease = Some(lease);
        Ok(host)
    }

    pub fn new(store: Store, codex: CodexBridge) -> Arc<Self> {
        Arc::new(Self {
            store: Arc::new(Mutex::new(store)),
            codex: Arc::new(codex),
            active: Mutex::new(None),
            gate: Arc::new(tokio::sync::Mutex::new(())),
            models: Mutex::new(Vec::new()),
            account: Mutex::new(None),
            turn_task: Mutex::new(None),
            error: Mutex::new(None),
            _lease: None,
        })
    }

    pub async fn read<T: Send + 'static>(
        &self,
        op: impl FnOnce(&mut Store) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            let mut store = store
                .lock()
                .map_err(|_| Error::Invalid("Çalışma alanı kullanılamıyor.".into()))?;
            op(&mut store)
        })
        .await
        .map_err(|_| Error::Invalid("İşlem tamamlanamadı.".into()))?
    }

    /// A roster, formation or model cannot change underneath an active AI turn.
    pub async fn mutate<T: Send + 'static>(
        &self,
        op: impl FnOnce(&mut Store) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let _permit = self
            .gate
            .clone()
            .try_lock_owned()
            .map_err(|_| Error::Invalid("Önce devam eden asistan yanıtını durdur.".into()))?;
        self.read(op).await
    }

    pub async fn conversation(&self) -> Result<Conversation> {
        let (scope, club, source, captured_at, model, messages) = self
            .read(|store| {
                Ok((
                    store.scope(),
                    store.snapshot().club.clone(),
                    store.snapshot().source,
                    store.snapshot().captured_at.clone(),
                    store.conversation_model()?,
                    store.conversation()?,
                ))
            })
            .await?;
        let active = self
            .active
            .lock()
            .map_err(|_| Error::Invalid("Asistan durumu kullanılamıyor.".into()))?;
        let current = active.as_ref().filter(|turn| turn.scope == scope);
        Ok(Conversation {
            scope,
            club,
            source,
            captured_at,
            model,
            messages,
            busy: current.is_some(),
            live: current.map(|t| t.live.clone()).unwrap_or_default(),
            active_turn_id: current.map(|t| t.id),
            models: self.models.lock().unwrap().clone(),
            error: self.error.lock().unwrap().clone(),
        })
    }

    pub async fn status(&self) -> Status {
        // Status discovery uses the same Codex transport; do not queue it behind a long turn.
        if self.active.lock().unwrap().is_some() {
            return self.account.lock().unwrap().clone().unwrap_or(Status {
                state: "busy".into(),
                message: "Yanıt işlenirken hesap durumu kontrol edilemiyor. Yanıt sonrasında yeniden kontrol et.".into(),
                plan: None,
                models: self.models.lock().unwrap().clone(),
            });
        }
        let status = self.codex.status().await;
        *self.models.lock().unwrap() = status.models.clone();
        *self.account.lock().unwrap() = Some(status.clone());
        status
    }

    pub async fn set_model(&self, scope: String, model: Option<String>) -> Result<Conversation> {
        self.mutate(move |store| {
            check_scope(store, &scope)?;
            store.set_conversation_model(model)
        })
        .await?;
        self.conversation().await
    }

    pub async fn send(
        self: &Arc<Self>,
        request: SendRequest,
        client: Client,
    ) -> Result<Conversation> {
        validate_prompt(&request.prompt)?;
        let permit = self.gate.clone().try_lock_owned().map_err(|_| {
            Error::Invalid(
                "Diğer arayüzde bir yanıt hazırlanıyor. Tamamlanmasını bekle veya durdur.".into(),
            )
        })?;
        let scope = request.scope.clone();
        let prompt = request.prompt.clone();
        let (id, context, history, model) = self
            .read(move |store| {
                check_scope(store, &scope)?;
                let history = store.conversation_history()?;
                let model = store.conversation_model()?;
                let context = prompt_context(store.snapshot(), store.formation());
                let id = store.begin_conversation_turn(&prompt, client)?;
                Ok((id, context, history, model))
            })
            .await?;
        let cancelled = Arc::new(AtomicBool::new(false));
        *self.active.lock().unwrap() = Some(ActiveTurn {
            id,
            scope: request.scope.clone(),
            live: String::new(),
            cancelled: cancelled.clone(),
        });
        *self.error.lock().unwrap() = None;
        let host = self.clone();
        let task = tokio::spawn(async move {
            let result = host
                .codex
                .chat_with_cancel(
                    request.prompt,
                    history,
                    context,
                    model,
                    cancelled.clone(),
                    |event| {
                        let mut active = host.active.lock().unwrap();
                        if let Some(turn) = active.as_mut().filter(|t| t.id == id) {
                            match event {
                                ChatEvent::Delta { text } => {
                                    if turn.live.len() + text.len()
                                        <= crate::conversation::MAX_ANSWER
                                    {
                                        turn.live.push_str(&text);
                                    }
                                }
                                ChatEvent::Complete { text } => {
                                    turn.live = bounded_text(&text, crate::conversation::MAX_ANSWER)
                                }
                                ChatEvent::Interrupted => {}
                            }
                        }
                    },
                )
                .await;
            let (text, state) = match result {
                Ok(answer) if !cancelled.load(Ordering::Acquire) => (answer, "complete"),
                Ok(_) => ("Yanıt durduruldu.".into(), "interrupted"),
                Err(message) => (
                    bounded_text(&message, 8000),
                    if cancelled.load(Ordering::Acquire) {
                        "interrupted"
                    } else {
                        "failed"
                    },
                ),
            };
            let scope = request.scope;
            let persisted = host
                .read(move |store| store.finish_conversation_turn(&scope, id, &text, state))
                .await;
            if persisted.is_err() {
                *host.error.lock().unwrap() = Some(
                    "Yanıt kaydedilemedi. Konuşmanın son durumu yeniden açılışta kontrol edilecek."
                        .into(),
                );
            }
            *host.active.lock().unwrap() = None;
            drop(permit);
        });
        *self.turn_task.lock().unwrap() = Some(task);
        self.conversation().await
    }

    pub fn cancel(&self, id: i64) -> Result<()> {
        let active = self.active.lock().unwrap();
        if let Some(turn) = active.as_ref().filter(|t| t.id == id) {
            turn.cancelled.store(true, Ordering::Release);
            Ok(())
        } else {
            Err(Error::Invalid("Bu yanıt artık etkin değil.".into()))
        }
    }

    pub async fn shutdown(&self) {
        if let Some(turn) = self.active.lock().unwrap().as_ref() {
            turn.cancelled.store(true, Ordering::Release);
        }
        let timed_out = tokio::time::timeout(std::time::Duration::from_secs(5), self.gate.lock())
            .await
            .is_err();
        let task = self.turn_task.lock().unwrap().take();
        if let Some(task) = task {
            if timed_out {
                task.abort();
            }
            let _ = task.await;
        }
        self.codex.shutdown().await;
    }
}

fn check_scope(store: &Store, expected: &str) -> Result<()> {
    if store.scope() != expected {
        return Err(Error::Invalid(
            "Kadro veya kariyer değişti. Güncel konuşmayı açıp yeniden gönder.".into(),
        ));
    }
    Ok(())
}
