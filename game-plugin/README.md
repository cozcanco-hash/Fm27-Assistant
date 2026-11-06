# FM27 Assistant — experimental in-game plugin

**Development preview. The managed plugin builds, but it has not been loaded in
an actual FM27 game yet. Do not treat this as a game-verified release.**

The C# BepInEx 6 IL2CPP plugin is a second interface to the same local Rust
assistant used by the desktop app. It contains no scoring engine, AI credentials
or independent conversation history. The desktop application must remain running.

## What this first implementation does

- Adds an optional FM Assistant button and a collapsible UI Toolkit chat panel.
- Tries F8 (fn+F8 on a Mac); the button is the fallback if the game's input backend
  does not expose legacy hotkeys. The key is intentionally different from FM's F7.
- Reads the same career conversation, model selection, in-progress answer and
  completed messages as the desktop interface. Either interface can stop a turn.
- Keeps network/JSON work off the game's render thread. Network requests are
  bounded and time out. Failed sends are never automatically retried.
- Clearly labels the desktop roster as demo or manual data.

**No live player, save-file or tactic extraction is included.** The panel uses
whatever validated roster is loaded in the desktop app. Automatic recognition of
the open player and real FM27 data adapters are subsequent milestones.

## Architecture

```text
Tauri / React desktop ──┐
                       ├── one Rust Assistant ── SQLite + local Codex
C# in-game panel ───────┘
```

The desktop app hosts the Rust service. Enable **Veri & bağlantı → Oyun içi
bağlantıyı aç** once. On macOS, closing the window while this connection is
active hides the window and keeps the assistant running. Click the Dock icon to
reopen it. Choose Quit from the app menu to fully stop it.

Only one host can open the same workspace. The transport listens on an ephemeral
`127.0.0.1` port; its rotating token is in the user's private discovery file. The
plugin gets no filesystem/command execution API and cannot import arbitrary
rosters. The desktop UI never receives this token.

Discovery on macOS:

```text
~/Library/Application Support/studio.kutra.fm27assistant/in-game/bridge.json
```

A custom `FM27_DATA_DIR` requires setting the plugin's `Connection.DescriptorPath`
to that directory's `in-game/bridge.json`. Never copy the token into a bug report.

Conversation history is stored by career in the existing SQLite database, up to
40 messages / 256 KiB of text per conversation. Interrupted turns are marked on
restart. Imports without a career identity stay isolated. A request from a stale
career is rejected; a second turn cannot run concurrently. A chat captures the
current roster/formation at send time. Model history comes from Rust, not clients.

## Build

Requires the pinned Rust toolchain, .NET SDK 10, and .NET 6 targeting references
(restored by the SDK). Use BepInEx 6 **IL2CPP**, not BepInEx 5 / Mono.

```bash
python3 game-plugin/scripts/prepare-references.py /tmp/fm27-loader-reference
FM27_BEPINEX_CORE=/tmp/fm27-loader-reference dotnet build game-plugin/src/FM27.Assistant.Plugin/FM27.Assistant.Plugin.csproj -c Release
python3 game-plugin/scripts/package.py
```

The preparation script downloads **compile-time references only**, pinned to an
official BepInEx v6.0.0-pre.2 archive and checksum. It extracts two managed
assemblies, never runs native loader code, and does not install anything into a
game. You can instead point `FM27_BEPINEX_CORE` at your actual compatible loader's
`BepInEx/core` directory. No BepInEx or FM game binaries are shipped in our ZIP.

The adapter late-binds generated Unity interop types and uses public UI Toolkit
APIs. This avoids shipping proprietary game assemblies but moves exact type and
UI-lifecycle compatibility to runtime acceptance checks. A successful build alone
cannot establish compatibility with an FM27/macOS/loader combination.

## First game acceptance check

Use an already working **FM27-compatible BepInEx 6 IL2CPP loader** for the exact
Mac/Steam build. ARM64 loaders have community-specific setup; this plugin does
not install, patch or re-sign the game or replace an existing loader.

1. Build/run desktop version 0.3.0 from the same branch and enable the in-game
   connection. Sign into ChatGPT through the desktop app if testing real chat.
2. Close FM27. Extract the `FM27Assistant` folder from our experimental ZIP into
   that loader's `BepInEx/plugins/` directory. Both managed DLLs must stay together.
3. Start FM27. Check `BepInEx/LogOutput.log` for **FM27 Assistant plugin loaded**.
4. Open the FM Assistant button. Confirm the panel labels the roster as demo or
   manual. It must not claim to have read your current save.
5. Send a question from the desktop app. Confirm the same question and answer
   appear in the game panel. Reply from the panel and check the desktop view.
6. Start a reply in one interface and stop it from the other. Then fully quit and
   relaunch the desktop app; check that messages and model choice are retained.
7. Test opening/closing the panel, text entry, focus, navigation, fullscreen and
   different display scaling. Confirm there is no duplicate panel after loading
   a new screen/save. Verify game shortcuts do not trigger while typing.
8. Disable the connection or quit the desktop app. The game panel should show a
   connection message; FM itself should continue running.

Remove only our `FM27Assistant` plugin folder with the game closed to uninstall
our plugin. Leave other mods, the loader and all saves in place.

Record Mac chip, macOS version, store, FM build, Unity version and loader version
with the results. Real-account chat and game rendering have **not** been
validated by the automated checks.

## Protocol and automated checks

`docs/bridge-protocol.md` documents the restricted transport. The C# acceptance
program starts a real isolated Rust host with a deterministic fake Codex CLI; it
checks cross-client history, model selection, persistence, cancellation, stale
career rejection and input/response limits. It never uses a real account.

```bash
cd desktop
cargo build --locked -p fm27-core --example bridge_host
cd ..
dotnet run --project game-plugin/tests/FM27.Assistant.Checks -c Release -- desktop/target/debug/examples/bridge_host desktop/crates/fm27-core/tests/fixtures/fake-shared-codex.py
```

These checks are additional to the Rust, browser and native desktop checks.
