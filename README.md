<p align="center">
  <img src="desktop/src/assets/fm27-icon.png" width="112" height="112" alt="FM27 Assistant app icon">
</p>

<h1 align="center">FM27 Assistant</h1>
<p align="center"><strong>Your club. Your decisions. Inside your game.</strong></p>
<p align="center">A macOS-first Football Manager companion built around an in-game Assistant page.<br>Understand your squad, plan with AI and review changes without leaving your career.</p>
<p align="center"><strong>In-game experience under development · Apple Silicon · Turkish interface</strong></p>
<p align="center"><a href="#the-new-direction">The new direction</a> · <a href="#roadmap">Roadmap</a> · <a href="#development-status">Development status</a> · <a href="#try-the-published-desktop-preview">Try the desktop preview</a></p>

## The new direction

FM27 Assistant began as a separate desktop tactics workspace: import a squad, compare formations and ask an AI for advice. The project now focuses on an **Assistant page inside Football Manager**, backed by a local Rust service.

The aim is one connected workflow:

1. **Load your current club.** Read the active career and managed squad through a version-checked game connection.
2. **Understand the squad.** Browse players, compare roles, keep shortlists and inspect the information available in that career.
3. **Plan with AI.** Discuss the squad and prepare structured tactical proposals grounded in observed game data.
4. **Review before applying.** Check the proposed changes, apply supported actions explicitly and retain a clear result and undo history.
5. **Keep playing.** Move between the Assistant and ordinary game screens while preserving the right workspace for the current session.

Normal football decisions, such as tactics and team selection, have their own workflow. Optional **Editor** capabilities cover artificial player, staff and club changes and are off by default in the new design. Hidden information has a separate disclosure setting.

**The immediate priority is AI tactics: complete the path from a proposal to real game application and undo.** Full formation, player-role and starting-XI application is still being built. The newer development package keeps tactic application disabled while the shared game-state checks are completed.

## Development status

**Status snapshot: 8 November 2026.** The public repository and the latest local development are at different stages. The roadmap below includes work that has not yet been published here.

| Track | What it represents |
| --- | --- |
| **Published `main`: v0.2.0 desktop preview** | The Rust/Tauri desktop app with fictional demo data, custom JSON imports, squad analysis, scouting and Codex chat integration. This is what the current GitHub Actions downloads build. |
| **Earlier local in-game verification** | An Assistant entry in the game, live squad capture and player views, a managed-club page and a verified return-to-Assistant flow. A limited editor baseline has also been exercised on a separate test career. These results are not a public download of the full new product. |
| **Current local implementation** | Native in-game pages and a windowless Rust helper, expanded player/club/staff editing, batch workflows, AI drafts and additional finance views. Much of this is source-level work with compilation/static checks only; new in-game acceptance remains pending. |

The earlier live baseline captured a 30-player squad with all 49 normal attributes per player on the supported build. Seven bounded edit types were exercised across the earlier local versions: current ability, potential ability, condition, retirement-history retention, club balance, transfer budget and weekly wage budget. Their specific apply/undo and save/reload checks do not validate every later editor field or every game situation.

A separate **0.23.0 local audit package** has been Developer ID signed and Apple-notarized. It is not a public release or proof that the new gameplay paths work. Signing, compilation, in-game operation and career save/reload are separate verification steps.

The currently verified adapter targets **Football Manager 27 Steam build 27.0.0.2493426 on macOS ARM64**, with SIP enabled. Other builds, Intel Macs, Windows and Linux are not covered by that result. A game update requires adapter revalidation.

## Roadmap

The product goal combines in-game assistance, scouting and optional editing across players, staff, clubs and the wider football world. Work is ordered around a usable, verifiable game workflow rather than the number of fields exposed.

| Priority | Milestone | Remaining work |
| --- | --- | --- |
| **1 · AI tactics** | Discuss, draft, review, apply and undo a tactic | Finish normal-game application and readback, then connect formations, roles and the starting XI. Current instruction catalogues and drafts are not a completed tactic editor. |
| **2 · In-game acceptance** | Verify the new flows on a separate test career | Exercise editing, undo, save/reload, reconnects, career changes and disclosure settings. New gameplay tests are reserved for a supervised session. |
| **3 · Player and staff tools** | Complete the player/staff workspace | Extend the supported attribute, position, health, personal-detail and staff subsets; add missing suspensions, permits, relationships, languages and specialist staff views. |
| **4 · Contracts, transfers and club management** | Connect the club's sporting and financial decisions | Expand contracts, loans, clauses, transfers, sponsorships, debts, financial history, ownership, vision and affiliated clubs. Current sponsorship/debt table work is read-only. |
| **5 · Search and bulk work** | Find and manage more than the loaded squad | Add wider world search, staff/club filters, richer presets, complete bulk operations and value-freezing workflows with explicit partial-result handling. |
| **6 · Football world and reports** | Cover the wider career | Add career statistics, stadiums, nations, competitions, fixtures, rules and recurring reports tied to a reliable career identity. |
| **7 · Product and distribution** | Deliver a consistent macOS experience | Finish navigation, accessibility, performance, Steam launch/reconnect handling, reproducible packaging and release-specific validation. |

The broader inventory contains **110 scope items in 18 categories**. At the 8 November checkpoint, 48 had partial source implementations and 62 were not implemented. “Partial” does not mean complete, released or verified in game. The project does not yet claim full FMRTE feature parity.

### Rules for the new experience

- **Ordinary mode stays ordinary.** Artificial editing is optional. Hidden CA/PA and personality information must stay out of ordinary views, exports and AI context.
- **The AI proposes; the user decides.** Structured drafts do not grant the model permission to write to the game.
- **Check the current game, not an old snapshot.** Supported writes require fresh session, object and value checks, followed by independent readback.
- **Keep an operation history.** Record intent and results, and offer bounded undo where the original state can be restored safely. Undo is not a rollback of the entire simulation or an already saved career file.
- **Leave unknowns unknown.** Missing data is not zero. An uncertain write must not be silently retried or reported as successful.
- **Validate the exact build.** Keep SIP enabled and reject unsupported game versions rather than reuse guessed layouts.

These are implementation and acceptance requirements. The latest source additions still need their own real-game verification.

## Architecture

| Layer | Direction of the in-game implementation |
| --- | --- |
| Product interface | Native Assistant pages inside the game |
| Launcher and local service | Windowless Rust process |
| Game connection | Version-checked native macOS ARM64 adapter |
| Analysis and operation rules | Shared Rust core |
| Local workspace and operation journal | SQLite |
| Optional AI | Local Codex app-server using the user's own ChatGPT account |
| Development tools | The earlier Tauri/React interface, retained as an optional diagnostic surface in current local development |

The public `main` branch still contains the earlier **Rust + Tauri 2 + React + SQLite** desktop application. The published [`codex/in-game-assistant` branch](https://github.com/cozcanco-hash/Fm27-Assistant/tree/codex/in-game-assistant) is an early shared-core experiment, not the complete current native implementation. The original Java application remains a separate reference.

## Try the published desktop preview

**Apple Silicon developer preview · Turkish interface · Earlier desktop experience**

The preview is useful for exploring the squad-analysis foundation without owning the game. It does **not** include the newer native in-game roadmap described above.

<p align="center"><a href="https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml"><img src="https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml/badge.svg" alt="Published desktop preview checks"></a></p>

![Earlier desktop dashboard with a fictional squad, browser preview](screenshots/rust-desktop-overview.png)

*Earlier desktop/browser preview, not a screenshot of the new in-game Assistant.*

1. Open [Rust desktop builds](https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml) and select the latest successful run for `main`.
2. Under **Artifacts**, download **FM27-Assistant-rust-macos-arm64**. GitHub may ask you to sign in.
3. Extract the ZIP, open the `.dmg` and copy **FM27 Assistant** to Applications. The ZIP also includes an application archive and SHA-256 checksums.
4. Explore **KUTRA FC**, a squad of 24 clearly fictional players.

These Actions builds are ad-hoc signed and not notarized. macOS may require approval in **System Settings → Privacy & Security** after the first launch attempt. Intel Macs have not been validated. Running the built app requires no Rust, Node.js or Java. Artifacts are retained for 14 days; an expired build is not a permanent release download.

[Mac testing guide →](TESTING.md)

### What the published preview can do

| Area | Available on `main` |
| --- | --- |
| **Squad and scouting** | Search players; combine age, ability, potential, condition, form, foot, role and attribute filters; inspect optional CA/PA and hidden fields when present in imported data. |
| **Comparison and shortlists** | Compare up to three players for a selected role and keep named local watchlists. |
| **Development history** | Compare known attributes and ability across up to 24 game dates in an identified imported career. |
| **Tactical analysis** | Compare 4–3–3 DM, 4–2–3–1 and 4–4–2; build a position-eligible XI without duplicate players, accounting for injury and condition. |
| **Transparent role scores** | Inspect weighted attributes and missing-data coverage. Scores require at least 70% of the role's attribute weight to be known. |
| **AI conversation** | Use the local Codex integration for model selection, streamed advice and stop controls. |
| **Local data** | Import the app's validated JSON schema, up to 4 MiB / 5,000 players, and retain the squad and formation in SQLite. Invalid imports leave the current squad intact. |

Role scores are Assistant heuristics, not Football Manager's match-engine formula or a guarantee of results. The custom JSON format is not an automatic adapter for every FM export. The published preview does not read live games, parse saves, decode `.fmf` files or control the game.

See [the published scouting/data contract](desktop/SCOUTING.md). The browser preview uses Rust-generated analysis; native imports, persistence and real AI require the desktop app.

### ChatGPT connection

AI features use the **official Codex CLI**, installed separately, and a ChatGPT account with Codex access. Subscription limits apply; the app does not request an API key. Authentication is handled by Codex.

Squad analysis runs locally. Sending an AI message transmits its bounded game/squad context and conversation to the connected service; opening the app does not itself send a model request. The published desktop chat includes the selected XI, up to 20 alternatives and bounded recent conversation. Its automated protocol tests use a fake CLI and do not prove real-account sign-in or gameplay integration. The newer AI tactic and editor flows still require separate end-to-end acceptance.

## Develop the published version

```bash
cd desktop
npm ci
npm run desktop:dev
```

Development requires Node.js 24, the pinned Rust toolchain and Xcode command-line tools. See [developer documentation](desktop/README.md), [sample data](desktop/fixtures/demo-squad.json) and the [desktop workflow](.github/workflows/rust-desktop.yml).

That workflow checks Rust formatting, Clippy, core tests, browser interactions and the native Apple Silicon build. Its smoke test requires React to read 24 demo players and an 11-player lineup from Rust/SQLite. These checks apply to the published desktop app, not to the unpublished in-game implementation.

<details>
<summary>Historical desktop performance baseline · v0.1.0 · 6 November 2026</summary>

[Verified source](https://github.com/cozcanco-hash/Fm27-Assistant/commit/fa66465e02e145e6f7ca0ca4827ff5e85248ee39) · [Successful run: 21 Rust tests and 4 browser tests](https://github.com/cozcanco-hash/Fm27-Assistant/actions/runs/42180058741)

| Apple Silicon CI measurement | Result |
| --- | --- |
| Application bundle | 7.72 MiB |
| DMG | 3.35 MiB |
| Launch to interface and data ready | 4.152 seconds |
| Synthetic 5,000-player lineup calculation | Median 7.01 ms; p95 11.96 ms across 25 runs |

These measurements predate the new icon and the in-game implementation. The lineup benchmark excludes interface, network and AI latency. They are not a Java-versus-Rust comparison or a performance guarantee for the current roadmap.
</details>

## Project history

FM27 Assistant is a KUTRA project. The icon combines a football tactics board, a passing route and a conversation bubble in charcoal and lime.

The Java reference was imported from [JelmerBouma1985/fm-ai-assistent](https://github.com/JelmerBouma1985/fm-ai-assistent) at commit `a06ac845acfd6c0eccbd61e3a633b3518584e457`. Its documentation is preserved in [README_UPSTREAM.md](README_UPSTREAM.md), with previous Mac validation in [JAVA_PREVIEW.md](JAVA_PREVIEW.md). No new license grant is asserted over the upstream code.

This is an independent project, unaffiliated with Sports Interactive, SEGA or FMRTE.
