<p align="center">
  <img src="desktop/src/assets/fm27-icon.png" width="112" height="112" alt="FM27 Assistant icon">
</p>

<h1 align="center">FM27 Assistant</h1>
<p align="center"><strong>Your club. Your decisions. Inside your game.</strong></p>
<p align="center">An in-game Football Manager assistant for macOS.<br>Squad analysis, scouting, AI tactics and optional editing, through the game's own interface.</p>
<p align="center"><strong>Native in-game interface · Rust · Apple Silicon · Turkish UI</strong></p>
<p align="center"><a href="#inside-football-manager">Experience</a> · <a href="#the-in-game-workspace">Workspace</a> · <a href="#roadmap">Roadmap</a> · <a href="#development-status">Status</a> · <a href="#architecture">Architecture</a></p>

## Inside Football Manager

FM27 Assistant lives in the game's **Asistan** page. Open your club, inspect the squad, discuss your next move and return to the match preparation flow from the same game interface.

**All product screens are in-game.** A windowless Rust launcher and local service handle the connection, analysis and storage in the background. There is no separate user-facing Assistant window in the current production design.

The experience is built around your active career:

1. **Load your club.** Resolve the current manager and managed club, then capture the squad with its game date and source.
2. **Explore and compare.** Move between player profiles, scouting filters, shortlists, role comparisons and team analysis.
3. **Plan with AI.** Ask questions using the observed squad context and prepare structured tactical proposals.
4. **Review supported changes.** Inspect a proposal before applying it, follow its result and use verified undo where supported.
5. **Return to the game.** Keep the workspace when the game context still matches; prevent old career data from authorizing new actions.

The current focus is **AI tactics → review → apply to the game → undo**. This full path is still under development. Formation, role and starting-XI application are not yet complete, and normal tactic application remains disabled in the current development source while the shared game-state checks are finished.

## The in-game workspace

These areas are present in the current local source. Their verification levels differ; the [status section](#development-status) separates earlier live results from newer, untested work.

| Area | What it brings together |
| --- | --- |
| **Club and squad** | Current-club context, squad browsing, player profiles, attributes and available contract information. |
| **Scouting and comparison** | Combined player filters, favourites, named shortlists, role comparisons and observation/development history. |
| **Team analysis** | Shared Rust role calculations, selection preferences and configurable analysis weights. |
| **AI assistant** | In-game conversation, model selection, streamed replies, Stop controls and structured proposals for review. |
| **Tactics and drafts** | Observed instruction options and manual/AI drafts, with a separate normal-game action path under development. |
| **Staff** | Current-club staff list, profiles and a restricted set of editable ability, reputation and coaching/scouting fields. |
| **Club and finance** | Club information, supported budget fields and source-integrated read-only sponsorship and debt/loan tables. |
| **Optional Editor** | Supported player, staff and club changes, explicit previews, operation history and bounded undo. |
| **Bulk work and templates** | Draft queues, selected-field copying, templates and CSV workflows, with individual operation results. |
| **Settings and help** | Editor and hidden-information controls, analysis preferences and in-game guidance. |

A page or input being present does not mean every field, object subtype or game situation is supported. Unknown observations remain unknown, and role scores are Assistant heuristics rather than Football Manager's match-engine formula.

## AI assistance and player control

The AI is intended to help explain the squad and turn an idea into a reviewable plan. It does not receive authority to make arbitrary game changes.

- **Normal football decisions** use the tactics and team-selection workflow.
- **Artificial edits** use the optional Editor, which starts off.
- **Hidden information** has a separate setting. Private CA/PA and personality values must stay out of ordinary views, exports and AI context.
- **Every supported write** needs fresh session, target and value checks, followed by independent readback.
- **Undo** is tied to the original operation and its verified state. It does not rewind the simulation or rewrite an already saved career file.
- **Uncertain results** stay visible and are not silently retried or reported as success.

AI features use the official **Codex CLI** and the user's own ChatGPT account with Codex access. Subscription limits apply; authentication stays with Codex. Local analysis does not need a model request. Sending an AI message transmits its bounded permitted game context and conversation to the connected service. The new tactic and editor proposal flows still need end-to-end gameplay acceptance.

## Roadmap

### 1. Complete AI tactics first

Connect the full proposal, review, application and undo flow to the real game tactic system. Finish formations, player roles, the starting XI and instruction handling. Undo must preserve the relevant prior tactical state, including custom role and player instructions, rather than merely reselect an old formation.

### 2. Expand player and staff tools

Complete the supported attribute, position, health, personal-detail and staff subsets. Add missing suspensions, work permits, relationships, languages and specialist staff views. Keep field-level support and actual-game verification explicit.

### 3. Connect contracts, transfers and club management

Extend contracts, loans, clauses, player moves and future transfers. Add sponsorship/debt editing, financial history, ownership, club vision and affiliated clubs. Existing sponsorship and debt table work is read-only.

### 4. Broaden search and bulk workflows

Move beyond the loaded squad with wider world search, staff/club filters, richer presets, complete bulk operations and value-freezing workflows. Preserve clear partial-success, conflict and undo behaviour.

### 5. Cover the wider football world

Add career statistics, stadiums, nations, competitions, fixtures, rules and reports. Recurring reports need a reliable career identity so one save's data cannot be confused with another's.

### 6. Verify the complete in-game experience

Run supervised tests on a separate test career: application, independent readback, undo, save/reload, reconnects, career changes and disclosure settings. Finish navigation, accessibility, performance, Steam launch handling and release packaging. Gameplay tests for the new work remain deferred to that joint testing session.

The scope inventory contains **110 items across 18 categories**. At the 8 November 2026 checkpoint, 48 had partial source implementations and 62 were not implemented. Those figures describe coverage, not completed or released features. Full FMRTE feature parity has not been achieved.

## Development status

**Reviewed against the active local project on 8 November 2026. Development is ongoing.**

| Evidence level | Current position |
| --- | --- |
| **Observed in game** | Earlier local checks verified the Assistant entry, live squad/player views, managed-club loading and returning to the Assistant without reloading the same workspace. A baseline capture contained 30 players with all 49 normal attributes per player. |
| **Limited edits verified** | Earlier local versions exercised seven bounded edit types: current ability, potential ability, condition, retirement-history retention, club balance, transfer budget and weekly wage budget, with specific apply/undo and save/reload checks. |
| **Implemented in source** | The in-game workspace, windowless service and broader player/club/staff, bulk, finance and AI-draft work. Newer additions have compilation/static evidence but still require their own runtime acceptance. |
| **Actively being completed** | Full AI tactic integration, real tactic application and undo, then the remaining feature scope. Normal tactic writes are currently blocked; a draft is not an applied tactic. |

The current native adapter is anchored to **Football Manager 27 Steam build 27.0.0.2493426 on macOS ARM64**, with SIP enabled. Other game builds and platforms are not covered by that verification. Game updates require adapter revalidation.

A signed and Apple-notarized local audit package exists. Notarization verifies distribution requirements, not the gameplay correctness of every source feature. The active source has continued to change since that package was produced.

### Source and release availability

The latest native implementation is still being developed locally and has not been fully synchronized to the public repository. This README describes the current product and roadmap; the checked-in source and existing workflow artifacts do not yet represent a complete current in-game release. No current in-game public release download is advertised here.

## Architecture

| Layer | Current production direction |
| --- | --- |
| **User interface** | Native Assistant pages inside Football Manager |
| **Startup and background service** | Windowless Rust launcher and local host |
| **Game connection** | Version-checked native macOS ARM64 adapter |
| **Analysis and operation rules** | Shared Rust core |
| **Local data and operation history** | SQLite |
| **Optional AI** | Local Codex app-server |

The default production source does not include Tauri or WebKit. All user interaction belongs to the in-game pages; the background helper has no separate product screen.

## Project and attribution

FM27 Assistant is a **KUTRA** project. Its charcoal-and-lime icon combines a tactics board, a passing route and a conversation bubble.

The repository retains a Java reference imported from [JelmerBouma1985/fm-ai-assistent](https://github.com/JelmerBouma1985/fm-ai-assistent) at commit `a06ac845acfd6c0eccbd61e3a633b3518584e457`. Original attribution is preserved in [README_UPSTREAM.md](README_UPSTREAM.md). No new license grant is asserted over the upstream code.

This is an independent project, unaffiliated with Sports Interactive, SEGA or FMRTE.
