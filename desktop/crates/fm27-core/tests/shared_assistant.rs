use fm27_core::{
    assistant::{Assistant, SendRequest},
    bridge::{BridgeServer, Descriptor, MAX_REQUEST},
    codex::CodexBridge,
    conversation::Client,
    Store,
};
use serde_json::{json, Value};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

fn host(directory: &Path) -> Arc<Assistant> {
    Assistant::new(
        Store::open(&directory.join("assistant.sqlite")).unwrap(),
        CodexBridge::with_executable(
            directory.join("codex"),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-shared-codex.py"),
        ),
    )
}
async fn done(host: &Assistant) -> fm27_core::assistant::Conversation {
    tokio::time::timeout(Duration::from_secs(6), async {
        loop {
            let state = host.conversation().await.unwrap();
            if !state.busy {
                return state;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap()
}
async fn request(descriptor: &Descriptor, method: &str, params: Value, token: &str) -> Value {
    let mut socket = TcpStream::connect(("127.0.0.1", descriptor.port))
        .await
        .unwrap();
    let mut data =
        serde_json::to_vec(&json!({"protocol":1,"token":token,"method":method,"params":params}))
            .unwrap();
    data.push(b'\n');
    socket.write_all(&data).await.unwrap();
    let mut line = String::new();
    BufReader::new(socket).read_line(&mut line).await.unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn conversation_persists_isolates_careers_and_recovers_interrupted_turns() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("state.sqlite");
    let mut store = Store::open(&path).unwrap();
    let scope = store.scope();
    store
        .set_conversation_model(Some("chosen-model".into()))
        .unwrap();
    let id = store
        .begin_conversation_turn("Merhaba", Client::Game)
        .unwrap();
    store
        .finish_conversation_turn(&scope, id, "Birinci yanıt", "complete")
        .unwrap();
    let pending = store
        .begin_conversation_turn("Bekleyen", Client::Desktop)
        .unwrap();
    drop(store);
    let mut store = Store::open(&path).unwrap();
    assert_eq!(
        store.conversation_model().unwrap().as_deref(),
        Some("chosen-model")
    );
    assert_eq!(
        store.conversation().unwrap().last().unwrap().state,
        "interrupted"
    );
    assert!(store
        .finish_conversation_turn(&scope, pending, "Geç yanıt", "complete")
        .is_err());
    let mut manual = fm27_core::demo_snapshot();
    manual.source = fm27_core::Source::Manual;
    manual.career_id = Some("other-career".into());
    store.replace(manual).unwrap();
    assert!(store.conversation().unwrap().is_empty());
    assert!(store.conversation_model().unwrap().is_none());
    store.reset_demo().unwrap();
    assert_eq!(store.conversation().unwrap().len(), 4);
    assert_eq!(store.conversation_history().unwrap().len(), 2);
}

#[test]
fn retention_is_bounded_and_utf8_history_never_cuts_a_character() {
    let mut store = Store::memory().unwrap();
    let scope = store.scope();
    for _ in 0..24 {
        let id = store
            .begin_conversation_turn("İstanbul", Client::Desktop)
            .unwrap();
        store
            .finish_conversation_turn(&scope, id, &"ş".repeat(60_000), "complete")
            .unwrap();
    }
    let messages = store.conversation().unwrap();
    assert!(messages.len() <= 40);
    assert!(messages.iter().map(|m| m.text.len()).sum::<usize>() <= 256 * 1024);
    assert!(store
        .conversation_history()
        .unwrap()
        .iter()
        .all(|m| m.text.len() <= 3500));
    assert!(store
        .begin_conversation_turn(&"x".repeat(8001), Client::Game)
        .is_err());
}

#[tokio::test]
async fn one_rust_host_owns_the_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let first = Assistant::open(temp.path()).unwrap();
    assert!(Assistant::open(temp.path()).is_err());
    drop(first);
    assert!(Assistant::open(temp.path()).is_ok());
}

#[tokio::test]
async fn game_and_desktop_share_history_model_busy_state_and_cancellation() {
    let temp = tempfile::tempdir().unwrap();
    let host = host(temp.path());
    let server = BridgeServer::start(host.clone(), temp.path())
        .await
        .unwrap();
    let descriptor: Descriptor =
        serde_json::from_slice(&std::fs::read(temp.path().join("in-game/bridge.json")).unwrap())
            .unwrap();
    let scope = host.conversation().await.unwrap().scope;
    host.set_model(scope.clone(), Some("protocol-test".into()))
        .await
        .unwrap();
    let response = request(
        &descriptor,
        "send",
        json!({"scope":scope,"prompt":"Oyundan ilk mesaj"}),
        &descriptor.token,
    )
    .await;
    assert_eq!(response["ok"], true);
    assert_eq!(response["result"]["model"], "protocol-test");
    let state = host.conversation().await.unwrap();
    assert!(state.busy);
    assert_eq!(state.messages[0].client, Client::Game);
    assert!(host
        .send(
            SendRequest {
                scope: scope.clone(),
                prompt: "Aynı anda ikinci".into()
            },
            Client::Desktop
        )
        .await
        .is_err());
    assert!(host.mutate(|s| s.reset_demo()).await.is_err());
    let state = done(&host).await;
    assert_eq!(state.messages.last().unwrap().text, "TEST: first answer");
    host.send(
        SendRequest {
            scope: scope.clone(),
            prompt: "Masaüstünden devam".into(),
        },
        Client::Desktop,
    )
    .await
    .unwrap();
    let state = done(&host).await;
    assert_eq!(
        state.messages.last().unwrap().text,
        "TEST: shared conversation received"
    );
    let capture = std::fs::read_to_string(temp.path().join("codex/captured-context.txt")).unwrap();
    assert!(capture.contains("Oyundan ilk mesaj"));
    let state = request(&descriptor, "state", Value::Null, &descriptor.token).await;
    assert_eq!(state["result"]["messages"].as_array().unwrap().len(), 4);
    let state = host
        .send(
            SendRequest {
                scope: scope.clone(),
                prompt: "Durdur".into(),
            },
            Client::Desktop,
        )
        .await
        .unwrap();
    assert!(request(
        &descriptor,
        "cancel",
        json!({"turnId":state.active_turn_id.unwrap()}),
        &descriptor.token
    )
    .await["ok"]
        .as_bool()
        .unwrap());
    assert_eq!(
        done(&host).await.messages.last().unwrap().state,
        "interrupted"
    );
    // A late cancel from the other interface must not stop a newer turn.
    assert!(host.cancel(state.active_turn_id.unwrap()).is_err());
    assert!(host
        .send(
            SendRequest {
                scope: "old-career".into(),
                prompt: "Eski görünüm".into()
            },
            Client::Game
        )
        .await
        .is_err());
    server.stop().await;
    host.shutdown().await;
}

#[tokio::test]
async fn bridge_authentication_limits_and_discovery_lifecycle() {
    let temp = tempfile::tempdir().unwrap();
    let host = host(temp.path());
    let server = BridgeServer::start(host.clone(), temp.path())
        .await
        .unwrap();
    let path = temp.path().join("in-game/bridge.json");
    let descriptor: Descriptor = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(descriptor.address, "127.0.0.1");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let refused = request(&descriptor, "state", Value::Null, &"0".repeat(64)).await;
    assert_eq!(refused["ok"], false);
    assert!(refused.get("result").is_none());
    assert_eq!(
        request(
            &descriptor,
            "runCommand",
            json!({"command":"no"}),
            &descriptor.token
        )
        .await["ok"],
        false
    );
    let mut socket = TcpStream::connect(("127.0.0.1", descriptor.port))
        .await
        .unwrap();
    socket
        .write_all(&vec![b'x'; MAX_REQUEST + 1])
        .await
        .unwrap();
    socket.write_all(b"\n").await.unwrap();
    let mut line = String::new();
    let closed = tokio::time::timeout(
        Duration::from_secs(2),
        BufReader::new(socket).read_line(&mut line),
    )
    .await
    .unwrap();
    assert!(closed.is_err() || closed.unwrap() == 0);
    assert!(BridgeServer::start(host.clone(), temp.path())
        .await
        .is_err());
    server.stop().await;
    assert!(!path.exists());
    let second = BridgeServer::start(host.clone(), temp.path())
        .await
        .unwrap();
    let fresh: Descriptor = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_ne!(descriptor.token, fresh.token);
    assert_eq!(
        request(&fresh, "state", Value::Null, &descriptor.token).await["ok"],
        false
    );
    second.stop().await;
}
