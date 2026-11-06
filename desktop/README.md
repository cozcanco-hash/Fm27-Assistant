# FM27 Assistant — Rust desktop

A personal, local macOS tactical workspace. **Rust + Tauri 2 + React +
SQLite.** Designed to be useful before owning the game. Starts with 24 fictional
players and calculates its analysis in Rust. No JVM or Spring server is needed
to render the desktop interface.

## Branding

The shared app icon is `src/assets/fm27-icon.png`. It is used in the interface,
the repository README and the generated macOS icon set. Regenerate platform icons
with `npm run icons`. The mark combines a tactics board, passing route and
conversation bubble in the app’s charcoal/lime palette.

## Implemented in the Rust app

- Turkish interface: overview, searchable/paginated squad, player profile,
  tactic board, AI chat and data/connection settings.
- 4–3–3 DM, 4–2–3–1 and 4–4–2 formations, with unique-player assignment across
  the entire eleven using the Hungarian algorithm.
- Explicit role attribute weights, missing-data coverage, condition adjustment
  for selection and exclusion of injured players.
- Strict custom JSON import, maximum 4 MiB / 5,000 players. SQLite transaction
  commits before new state becomes visible. Last three valid snapshots retained.
- Persistent roster and formation between launches. Demo/manual provenance and
  the source timestamp remain visible.
- Optional CA/PA and hidden attributes, advanced combined filters, three-player
  role comparison, named local shortlists and development across game dates.
  Existing role weights are unchanged. See [the scouting contract](SCOUTING.md).
- Local Codex app-server integration: account check, official ChatGPT browser
  login, account model discovery, streamed replies, stop and timeout handling.
  The integration requires the official Codex CLI; credentials stay there.
- ARM64 `.app`/`.dmg` pipeline, native WebView readiness test, core tests,
  browser interaction tests and a repeatable synthetic performance measurement.

This is a focused new implementation, not feature parity with the Java source.
Native FM27 live reading, FM save parsing, the Java `.fmf` tactic decoder,
staff/recruitment databases and the full MCP tool set have **not** been ported.
Custom JSON is our application schema, not an automatically supported FM export.

## Develop on macOS

Prerequisites: Xcode command-line tools, Rust via rustup and Node.js 24. The
checked-in toolchain file pins Rust 1.99.1. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
cd desktop
npm ci
cargo test --locked -p fm27-core
npm run desktop:dev
```

Build:

```bash
npm run desktop:build
```

Output: `target/release/bundle/macos/FM27 Assistant.app` and the DMG in
`target/release/bundle/dmg/`. An installed application needs no Rust, Node.js,
Java or development tools. Codex CLI is needed only for ChatGPT features.

Development packages are ad-hoc signed, without Developer ID notarization.
Signing/notarization for public distribution remains a later milestone.

## Checks

```bash
cargo fmt --all --check
cargo clippy --locked -p fm27-core --all-targets -- -D warnings
cargo test --locked -p fm27-core
cargo run --locked --quiet -p fm27-core --example preview > /tmp/preview-state.json
cmp fixtures/preview-state.json /tmp/preview-state.json
npm run build
npx playwright install chromium --only-shell
npm test
```

On a Mac, also run `npm run icons`, then `cargo clippy --locked --workspace
--all-targets -- -D warnings` and `cargo test --locked --workspace`. After a
release build, `bash scripts/macos-smoke.sh` opens the actual application with
a minimal PATH. A successful report requires React to load and query the
Rust/SQLite backend; it does not sign into ChatGPT or use a real FM game.

`cargo run --locked --release -p fm27-core --example benchmark` measures 25
lineup calculations for a synthetic 5,000-player dataset and reports median
and p95. This excludes UI, network and AI latency. No Java-versus-Rust RAM or
speed multiplier is claimed without equivalent measured workloads.

The browser development preview uses checked-in **Rust-generated results**;
there is no second role-scoring engine in TypeScript. File import and real AI
are disabled in the browser preview. Regenerate its fixtures after changing
Rust calculations:

```bash
cargo run --quiet -p fm27-core --example preview > fixtures/preview-state.json
```

## Data contract

See [the complete fictional sample](fixtures/demo-squad.json). Root fields:

| Field | Contract |
| --- | --- |
| `schemaVersion` | `1` |
| `club`, `season` | Nonempty text, at most 100 / 30 characters |
| `capturedAt` | ISO 8601 timestamp including time zone |
| `source` | `demo` or `manual`; no live source is supported |
| `players` | 1–5,000 entries with unique positive IDs |

Each player has `id`, `name`, `age` (14–60), `positions`, `foot`, `condition`
(0–100), `morale` (1–20), `form` (0–10 or `null`), `minutes`, `injured` and
`attributes`. Positions: `GK/DC/DL/DR/DM/MC/AML/AMR/AMC/ST`; foot:
`left/right/both`. Attribute values are 1–20; omit unknown ones instead of
using zero. Unknown fields and attribute keys are rejected.

Role scores use the weighted average of known attributes, divided by 20 and
scaled to 100. Coverage is the known share of total role weight; less than
70% yields **no score**. Selection respects positions and injuries and adds
a condition penalty to the assignment cost. Displayed role skill remains
separate from that penalty. Overall XI fit appears only when all 11 slots
are covered. These are transparent product heuristics, not FM match-engine
formulas or guarantees of winning tactics.

Optional additions keep schema version 1 backward compatible: player fields
`currentAbility`, `potentialAbility` (1–200) and `hiddenAttributes` (known keys,
1–20). Root `careerId` identifies a career and `gameDate` is `YYYY-MM-DD` in
game time. Manual development requires both. Unknown values are never inferred.
Lists and 24 distinct game-date points persist by career; unidentified imports
are isolated instead of merging unrelated saves.

## Storage and ChatGPT

The default app data directory on macOS is
`~/Library/Application Support/studio.kutra.fm27assistant/`. SQLite holds
rosters and formation. `FM27_DATA_DIR` can override it for tests. Java's existing
H2 data is left in place and is not silently migrated.

AI connects only after a user action. No model request is sent just by opening
the application. Sending a message passes the selected XI, up to 20 alternatives
and bounded recent conversation to the local Codex process, which uses the
signed-in ChatGPT account. Demo provenance and missing data are included. The
application does not accept API keys or start model requests in API-key mode.
Subscription limits still apply. See [the official app-server documentation](https://learn.chatgpt.com/docs/app-server).

Codex discovery checks PATH, common user installations, NVM and Homebrew.
`FM27_CODEX_PATH` can specify an absolute executable path. The frontend has
no general filesystem, shell or HTTP plugins. Rust exposes specific commands;
the chat thread requests a read-only sandbox and automatically declines
command/file-change permission requests. Authentication tokens are neither
read by our code nor returned to the frontend.

Conversations and model choice are persisted per career in SQLite, up to 40
messages / 256 KiB of text. Rust supplies bounded completed history to each
ephemeral Codex turn. The optional C# game panel shares the same conversation
and pending reply through an authenticated local connection. See
[the experimental in-game plugin](../game-plugin/README.md). Closing the Mac
window keeps the assistant in the background while the connection is enabled;
Quit fully stops it. Game rendering and live data extraction are separate
acceptance milestones; the current panel uses the desktop roster. Real ChatGPT sign-in and actual FM27 data still require
testing on the owner's Mac. Protocol tests use a deterministic fake CLI and
never spend model credits.

## Next game-independent milestones

1. Manual tactic instructions and richer, documented role definitions.
2. Squad editing and version comparison, with an explicit restore UI.
3. Controlled `.fmf` decoder migration using the Java fixtures.
4. Import adapters for real FM exports once examples and a game version exist.
5. Actual-game verification and distribution signing.
