use fm27_core::{
    analysis::{player_detail, PlayerDetail},
    assistant::{Assistant, Conversation, SendRequest},
    bridge::BridgeServer,
    codex::Status,
    conversation::Client,
    scouting::{PlayerHistory, Shortlist},
    store::{Dashboard, PlayerPage, PlayerQuery},
    Formation, Store,
};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};
use tauri::Manager;

struct AppState {
    assistant: Arc<Assistant>,
    bridge: tokio::sync::Mutex<Option<BridgeServer>>,
    bridge_available: AtomicBool,
    bridge_error: Mutex<Option<String>>,
    data_dir: PathBuf,
    started: Instant,
    smoke_report: Option<PathBuf>,
}

async fn with_store<T: Send + 'static>(
    assistant: Arc<Assistant>,
    op: impl FnOnce(&mut Store) -> fm27_core::Result<T> + Send + 'static,
) -> Result<T, String> {
    assistant.read(op).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn dashboard(state: tauri::State<'_, AppState>) -> Result<Dashboard, String> {
    with_store(state.assistant.clone(), |store| Ok(store.dashboard())).await
}

#[tauri::command]
async fn list_players(
    query: PlayerQuery,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerPage, String> {
    with_store(state.assistant.clone(), move |store| store.players(query)).await
}

#[tauri::command]
async fn list_shortlists(state: tauri::State<'_, AppState>) -> Result<Vec<Shortlist>, String> {
    with_store(state.assistant.clone(), |store| store.shortlists()).await
}

#[tauri::command]
async fn create_shortlist(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Shortlist>, String> {
    with_store(state.assistant.clone(), move |store| {
        store.create_shortlist(&name)
    })
    .await
}

#[tauri::command]
async fn delete_empty_shortlist(
    id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Shortlist>, String> {
    with_store(state.assistant.clone(), move |store| {
        store.delete_empty_shortlist(id)
    })
    .await
}

#[tauri::command]
async fn set_shortlisted(
    list_id: i64,
    player_id: u32,
    selected: bool,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Shortlist>, String> {
    with_store(state.assistant.clone(), move |store| {
        store.set_shortlisted(list_id, player_id, selected)
    })
    .await
}

#[tauri::command]
async fn player_history(
    id: u32,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerHistory, String> {
    with_store(state.assistant.clone(), move |store| {
        store.player_history(id)
    })
    .await
}

#[tauri::command]
async fn get_player(id: u32, state: tauri::State<'_, AppState>) -> Result<PlayerDetail, String> {
    with_store(state.assistant.clone(), move |store| {
        store
            .snapshot()
            .players
            .iter()
            .find(|p| p.id == id)
            .map(player_detail)
            .ok_or(fm27_core::Error::Invalid("Oyuncu bulunamadı.".into()))
    })
    .await
}

#[tauri::command]
async fn set_formation(
    formation: Formation,
    state: tauri::State<'_, AppState>,
) -> Result<Dashboard, String> {
    state
        .assistant
        .mutate(move |store| store.set_formation(formation))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_roster(
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<Dashboard, String> {
    state
        .assistant
        .mutate(move |store| store.import(content.as_bytes()))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_demo(state: tauri::State<'_, AppState>) -> Result<Dashboard, String> {
    state
        .assistant
        .mutate(|store| store.reset_demo())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn codex_status(state: tauri::State<'_, AppState>) -> Result<Status, String> {
    Ok(state.assistant.status().await)
}

#[tauri::command]
async fn codex_login(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let url = state.assistant.codex.login_url().await?;
    // URL comes from the CLI and is validated against exact official HTTPS hosts.
    #[cfg(target_os = "macos")]
    {
        let status = tokio::process::Command::new("/usr/bin/open")
            .arg(url)
            .kill_on_drop(true)
            .status()
            .await
            .map_err(|_| "Giriş tarayıcısı açılamadı.".to_string())?;
        if !status.success() {
            return Err("Giriş tarayıcısı açılamadı.".into());
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = url;
        Err("Uygulama içi tarayıcı girişi şu aşamada macOS'ta destekleniyor. Codex CLI üzerinden giriş yapabilirsin.".into())
    }
}

#[tauri::command]
async fn conversation(state: tauri::State<'_, AppState>) -> Result<Conversation, String> {
    state
        .assistant
        .conversation()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_chat(
    request: SendRequest,
    state: tauri::State<'_, AppState>,
) -> Result<Conversation, String> {
    state
        .assistant
        .send(request, Client::Desktop)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_chat_model(
    scope: String,
    model: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Conversation, String> {
    state
        .assistant
        .set_model(scope, model)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_chat(turn_id: i64, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.assistant.cancel(turn_id).map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameLinkStatus {
    enabled: bool,
    available: bool,
    message: String,
}

#[tauri::command]
async fn game_link_status(state: tauri::State<'_, AppState>) -> Result<GameLinkStatus, String> {
    let enabled = state
        .assistant
        .read(|s| s.game_bridge_enabled())
        .await
        .map_err(|e| e.to_string())?;
    let available = state
        .bridge
        .lock()
        .await
        .as_ref()
        .is_some_and(|b| b.is_running());
    state.bridge_available.store(available, Ordering::Release);
    let error = state.bridge_error.lock().unwrap().clone();
    Ok(GameLinkStatus { enabled,available,message:error.unwrap_or_else(|| if available {
        "Oyun içi eklenti aynı kadroya ve konuşmaya bağlanabilir. Canlı oyun verisi aktarımı henüz eklenmedi.".into()
    } else { "Oyun içinden kullanmak için bağlantıyı aç ve C# eklentisini kur.".into() }) })
}

#[tauri::command]
async fn set_game_link(
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<GameLinkStatus, String> {
    let mut slot = state.bridge.lock().await;
    if enabled {
        if slot.as_ref().is_some_and(|b| !b.is_running()) {
            if let Some(old) = slot.take() {
                old.stop().await;
            }
        }
        if slot.is_none() {
            let bridge = BridgeServer::start(state.assistant.clone(), &state.data_dir)
                .await
                .map_err(|e| e.to_string())?;
            if let Err(e) = state
                .assistant
                .read(|s| s.set_game_bridge_enabled(true))
                .await
            {
                bridge.stop().await;
                return Err(e.to_string());
            }
            *slot = Some(bridge);
        }
    } else {
        state
            .assistant
            .read(|s| s.set_game_bridge_enabled(false))
            .await
            .map_err(|e| e.to_string())?;
        if let Some(bridge) = slot.take() {
            bridge.stop().await;
        }
    }
    state
        .bridge_available
        .store(slot.is_some(), Ordering::Release);
    *state.bridge_error.lock().unwrap() = None;
    drop(slot);
    game_link_status(state).await
}

#[tauri::command]
fn frontend_ready(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(path) = &state.smoke_report {
        let store = state
            .assistant
            .store
            .lock()
            .map_err(|_| "Smoke store unavailable".to_string())?;
        let report = serde_json::json!({"ready":true,"version":env!("CARGO_PKG_VERSION"),"startupMs":state.started.elapsed().as_millis(),"players":store.snapshot().players.len(),"filled":store.dashboard().analysis.filled,"architecture":std::env::consts::ARCH});
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap())
            .map_err(|e| e.to_string())?;
        drop(store);
        if std::env::var_os("FM27_SMOKE_EXIT").is_some() {
            app.exit(0);
        }
    }
    Ok(())
}

pub fn run() {
    let started = Instant::now();
    let app = tauri::Builder::default()
        .setup(move |app| {
            let data_dir = std::env::var_os("FM27_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or(app.path().app_data_dir()?);
            std::fs::create_dir_all(&data_dir)?;
            let assistant = Assistant::open(&data_dir)?;
            let (bridge, bridge_error) = if assistant.store.lock().unwrap().game_bridge_enabled()? {
                match tauri::async_runtime::block_on(BridgeServer::start(
                    assistant.clone(),
                    &data_dir,
                )) {
                    Ok(bridge) => (Some(bridge), None),
                    Err(error) => (None, Some(error.to_string())),
                }
            } else {
                (None, None)
            };
            app.manage(AppState {
                assistant,
                bridge_available: AtomicBool::new(bridge.is_some()),
                bridge: tokio::sync::Mutex::new(bridge),
                bridge_error: Mutex::new(bridge_error),
                data_dir,
                started,
                smoke_report: std::env::var_os("FM27_SMOKE_REPORT").map(PathBuf::from),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dashboard,
            list_players,
            list_shortlists,
            create_shortlist,
            delete_empty_shortlist,
            set_shortlisted,
            player_history,
            get_player,
            set_formation,
            import_roster,
            load_demo,
            codex_status,
            codex_login,
            conversation,
            send_chat,
            set_chat_model,
            stop_chat,
            game_link_status,
            set_game_link,
            frontend_ready
        ])
        .build(tauri::generate_context!())
        .expect("FM27 Assistant could not start");
    app.run(|handle, event| match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::WindowEvent {
            label,
            event: tauri::WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            if handle
                .state::<AppState>()
                .bridge_available
                .load(Ordering::Acquire)
            {
                api.prevent_close();
                if let Some(window) = handle.get_webview_window(&label) {
                    let _ = window.hide();
                }
            }
        }
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        tauri::RunEvent::Exit => {
            let state = handle.state::<AppState>();
            tauri::async_runtime::block_on(async {
                if let Some(bridge) = state.bridge.lock().await.take() {
                    bridge.stop().await;
                }
                state.assistant.shutdown().await;
            });
        }
        _ => {}
    });
}
