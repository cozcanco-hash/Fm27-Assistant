<p align="center">
  <img src="desktop/src/assets/fm27-icon.png" width="112" height="112" alt="FM27 Assistant app icon">
</p>

<h1 align="center">FM27 Assistant</h1>
<p align="center"><strong>Your squad. Your system. Your next move.</strong></p>
<p align="center">A personal football tactics workspace for macOS, built with Rust and Tauri.<br>Explore your squad, compare formations and discuss your decisions with an AI assistant.</p>
<p align="center"><a href="https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml"><img src="https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml/badge.svg" alt="Rust desktop checks"></a></p>
<p align="center"><a href="#try-it-on-mac">Try it on Mac</a> · <a href="#features">Features</a> · <a href="TESTING.md">Testing guide</a> · <a href="desktop/README.md">Developer docs</a></p>

![FM27 Assistant dashboard with a fictional squad, browser preview](screenshots/rust-desktop-overview.png)

## Try it on Mac

**Apple Silicon developer preview · Turkish interface**

1. Open [Rust desktop builds](https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml) and select the latest successful run for `main`.
2. Under **Artifacts**, download **FM27-Assistant-rust-macos-arm64**. GitHub may ask you to sign in to download an Actions artifact.
3. Extract the ZIP, open the `.dmg` and copy **FM27 Assistant** to Applications. The ZIP also includes an application archive and SHA-256 checksums.
4. Launch the app and explore **KUTRA FC**, a squad of 24 fictional players. You do not need to own the game.

This is an ad-hoc signed developer preview, without notarization. macOS may require approval in **System Settings → Privacy & Security** after the first launch attempt. Intel Macs have not been validated. You do not need Rust, Node.js or Java to run the built app. Actions downloads are retained for 14 days; use the latest successful build.

[Follow the Mac testing guide →](TESTING.md)

## Features

| Feature | What you can do today |
| --- | --- |
| **Start before owning the game** | Explore a complete fictional squad with clearly marked demo data. |
| **Understand your players** | Search the squad, filter by position and inspect attributes, recent form, condition, morale and minutes played. |
| **Compare three formations** | Switch between 4–3–3 DM, 4–2–3–1 and 4–4–2 on an interactive tactic board. |
| **Build a balanced starting XI** | Assign eligible players across all eleven positions without duplicates; injured players are excluded and condition affects selection. |
| **See why a role fits** | Inspect weighted attribute scores and missing-data coverage. A score is shown only when at least 70% of the role's attribute weight is known. |
| **Ask your AI assistant** | Use the local Codex integration to sign in with ChatGPT, choose an available model and receive streamed tactical advice with stop controls. |
| **Bring your own squad data** | Import the app's validated JSON format, up to 4 MiB and 5,000 players. Invalid imports leave the current squad intact. |
| **Pick up where you left off** | Keep the squad and selected formation in a local SQLite database between launches. |

Role scores are transparent heuristics for this app. Form, morale and minutes provide context; they are not all part of the role-score formula. The system does not reproduce Football Manager's match engine or guarantee results.

### ChatGPT connection

AI features require the **official Codex CLI**, installed separately, and your own ChatGPT account with Codex access. Subscription limits apply. The app does not take an API key. It sends the selected XI, up to 20 alternatives and bounded recent conversation to Codex when you send a message; no model request is made just by opening the app. Authentication is handled by Codex.

Squad analysis runs locally; AI chat uses the connected service. Real ChatGPT sign-in and replies still need testing on a user's Mac. Automated tests verify the integration protocol with a fake CLI without consuming AI credits.

## Current scope

This is an early companion app. The current Rust version supports demo data and the app's **custom JSON schema**, not automatic imports from every FM export. Live game reading, save-file parsing, `.fmf` decoding, automatic in-game control and the Java reference's recruitment/staff features have not been implemented in the Rust app. FM27 compatibility remains to be verified with actual game data.

The screenshot above is the browser preview. It uses Rust-generated analysis; imports, persistence and real AI require the native desktop app.

## Built for macOS

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri 2 and the system WebView |
| Validation, role analysis and lineup assignment | Rust |
| Interface | React and TypeScript |
| Local squad storage | SQLite |
| Optional AI connection | Local Codex app-server |

Active development lives in [`desktop/`](desktop/). The original Java application is retained separately as a reference; this is a focused Rust implementation.

```bash
cd desktop
npm ci
npm run desktop:dev
```

Development requires Node.js 24, the pinned Rust toolchain and Xcode command-line tools. See the [developer documentation](desktop/README.md) for native builds, checks and the [sample data](desktop/fixtures/demo-squad.json).

## Verification

The pipeline checks Rust formatting, Clippy, core tests, browser interactions and the native Apple Silicon build. A native smoke test opens the packaged app and requires React to read all 24 demo players and the 11-player lineup from the Rust/SQLite backend. See [the workflow and downloadable reports](https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml).

<details>
<summary>Measured v0.1.0 baseline — 6 November 2026</summary>

[Verified source](https://github.com/cozcanco-hash/Fm27-Assistant/commit/fa66465e02e145e6f7ca0ca4827ff5e85248ee39) · [Successful run: 21 Rust tests and 4 browser tests](https://github.com/cozcanco-hash/Fm27-Assistant/actions/runs/42180058741)

| Apple Silicon CI measurement | Result |
| --- | --- |
| Application bundle | 7.72 MiB |
| DMG | 3.35 MiB |
| Launch to interface and data ready | 4.152 seconds |
| Synthetic 5,000-player lineup calculation | Median 7.01 ms; p95 11.96 ms across 25 runs |

These are measurements from the earlier v0.1.0 CI build, before the new icon. The lineup benchmark excludes interface, network and AI latency. These results are not a Java-versus-Rust comparison or a performance guarantee for every Mac.
</details>

## Next milestones

- Complete real-account and MacBook testing.
- Add manual tactic instructions and richer role definitions.
- Add squad editing and snapshot comparison/restore controls.
- Build import adapters against real game exports and verify game compatibility.
- Prepare signing and notarization for wider distribution.

## Project history

FM27 Assistant is a KUTRA project. The icon combines a football tactics board, a passing route and a conversation bubble in charcoal and lime.

The Java reference was imported from [JelmerBouma1985/fm-ai-assistent](https://github.com/JelmerBouma1985/fm-ai-assistent) at commit `a06ac845acfd6c0eccbd61e3a633b3518584e457`. Its documentation is preserved in [README_UPSTREAM.md](README_UPSTREAM.md), with previous Mac validation in [JAVA_PREVIEW.md](JAVA_PREVIEW.md). No new license grant is asserted over the upstream code.

This is an independent project, unaffiliated with Sports Interactive or SEGA.
