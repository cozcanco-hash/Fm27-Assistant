use fm27_core::{
    analysis::{player_detail, prompt_context, PlayerDetail},
    codex::{ChatEvent, ChatMessage, CodexBridge, Status},
    scouting::{PlayerHistory, Shortlist},
    store::{Dashboard, PlayerPage, PlayerQuery},
    Formation, Store,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use tauri::{ipc::Channel, Manager};

struct AppState {
    store: Arc<Mutex<Store>>,
    codex: Arc<CodexBridge>,
    started: Instant,
    smoke_report: Option<PathBuf>,
}

async fn with_store<T: Send + 'static>(
    store: Arc<Mutex<Store>>,
    op: impl FnOnce(&mut Store) -> fm27_core::Result<T> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut store = store
            .lock()
            .map_err(|_| "Veri çalışma alanı kullanılamıyor.".to_string())?;
        op(&mut store).map_err(|e| e.to_string())
    })
    .await
    .map_err(|_| "Veri işlemi tamamlanamadı.".to_string())?
}

#[tauri::command]
async fn dashboard(state: tauri::State<'_, AppState>) -> Result<Dashboard, String> {
    with_store(state.store.clone(), |store| Ok(store.dashboard())).await
}

#[tauri::command]
async fn list_players(
    query: PlayerQuery,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerPage, String> {
    with_store(state.store.clone(), move |store| store.players(query)).await
}

#[tauri::command]
async fn list_shortlists(state: tauri::State<'_, AppState>) -> Result<Vec<Shortlist>, String> {
    with_store(state.store.clone(), |store| store.shortlists()).await
}

#[tauri::command]
async fn create_shortlist(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Shortlist>, String> {
    with_store(state.store.clone(), move |store| {
        store.create_shortlist(&name)
    })
    .await
}

#[tauri::command]
async fn delete_empty_shortlist(
    id: i64,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Shortlist>, String> {
    with_store(state.store.clone(), move |store| {
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
    with_store(state.store.clone(), move |store| {
        store.set_shortlisted(list_id, player_id, selected)
    })
    .await
}

#[tauri::command]
async fn player_history(
    id: u32,
    state: tauri::State<'_, AppState>,
) -> Result<PlayerHistory, String> {
    with_store(state.store.clone(), move |store| store.player_history(id)).await
}

#[tauri::command]
async fn get_player(id: u32, state: tauri::State<'_, AppState>) -> Result<PlayerDetail, String> {
    with_store(state.store.clone(), move |store| {
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
    with_store(state.store.clone(), move |store| {
        store.set_formation(formation)
    })
    .await
}

#[tauri::command]
async fn import_roster(
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<Dashboard, String> {
    with_store(state.store.clone(), move |store| {
        store.import(content.as_bytes())
    })
    .await
}

#[tauri::command]
async fn load_demo(state: tauri::State<'_, AppState>) -> Result<Dashboard, String> {
    with_store(state.store.clone(), |store| store.reset_demo()).await
}

#[tauri::command]
async fn codex_status(state: tauri::State<'_, AppState>) -> Result<Status, String> {
    Ok(state.codex.status().await)
}

#[tauri::command]
async fn codex_login(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let url = state.codex.login_url().await?;
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
async fn send_chat(
    prompt: String,
    history: Vec<ChatMessage>,
    model: Option<String>,
    on_event: Channel<ChatEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let context = with_store(state.store.clone(), |store| {
        Ok(prompt_context(store.snapshot(), store.formation()))
    })
    .await?;
    state
        .codex
        .chat(prompt, history, context, model, move |event| {
            let _ = on_event.send(event);
        })
        .await
}

#[tauri::command]
fn stop_chat(state: tauri::State<'_, AppState>) {
    state.codex.cancel();
}

#[tauri::command]
fn frontend_ready(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(path) = &state.smoke_report {
        let store = state
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
            let store = Store::open(&data_dir.join("assistant.sqlite"))?;
            app.manage(AppState {
                store: Arc::new(Mutex::new(store)),
                codex: Arc::new(CodexBridge::new(data_dir.join("codex-workspace"))),
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
            send_chat,
            stop_chat,
            frontend_ready
        ])
        .build(tauri::generate_context!())
        .expect("FM27 Assistant could not start");
    app.run(|handle, event| {
        if let tauri::RunEvent::Exit = event {
            let state = handle.state::<AppState>();
            tauri::async_runtime::block_on(state.codex.shutdown());
        }
    });
}
