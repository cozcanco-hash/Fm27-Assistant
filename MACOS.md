# macOS developer preview

This private adaptation targets Can's Apple Silicon Mac. It packages the existing
Java/Vaadin application as a macOS `.app`; the interface opens in the default
browser on the local computer. It is not a SwiftUI rewrite or a plugin injected
into Football Manager.

## Current scope

- Build a native Apple Silicon `.app` with Java 25 bundled inside.
- Discover macOS Steam libraries and the native FM27 tactics/shortlists folders.
- Find a locally installed Codex CLI when launched from Finder's minimal PATH.
- Keep the packaged app's AI workspace in a writable user directory.
- Show a clear preview notice and prevent unsupported live memory reads on Mac.

**Live Mac squad loading is not implemented.** Player form, attributes, recruitment
and squad-based tactical advice require a verified data source. They must not be
presented as working simply because the interface starts. Existing Windows/Linux
memory offsets are not assumed to match the native Mac game. Tactic `.fmf` parsing
and Codex conversations are inherited features; real FM27 Mac files and a real
ChatGPT sign-in still need an end-to-end test on the owner's Mac.

## Get a build

The [2026-11-06 ARM64 preview](https://github.com/cozcanco-hash/Fm27-Assistant/actions/runs/42161326949/artifacts/13477226606)
is available until 2026-11-20. Code commit
[`61e2b01`](https://github.com/cozcanco-hash/Fm27-Assistant/commit/61e2b01b670be6752837e9d883395df1cb62dafa)
passed full verification on the `macos-15` ARM64 runner: 403 tests passed,
9 opt-in local/live tests were skipped, and coverage thresholds passed. The
packaged launcher started with its bundled runtime, health returned `UP`, and
the root URL returned HTTP 200. See the
[complete workflow](https://github.com/cozcanco-hash/Fm27-Assistant/actions/runs/42161326949).

Open the repository's **Actions → macOS Apple Silicon preview** workflow. A
successful run uploads `FM27-Assistant-macos-arm64-preview` as a private artifact
for 14 days. Extract the artifact, then its application ZIP. The ZIP includes a
`.app` and has a companion SHA-256 checksum. The app contains its Java runtime;
an end user does not need a separate JDK.

These development builds are not Developer ID signed or notarized. Distribution
and Gatekeeper installation testing are a later milestone. Prefer a local source
build for development; do not disable macOS security features to test live reads.

## Build on your Mac

Install a full **JDK 25** and **Node.js 24**, then:

```bash
git clone https://github.com/cozcanco-hash/Fm27-Assistant.git
cd Fm27-Assistant
export JAVA_HOME="$(/usr/libexec/java_home -v 25)"
./mvnw --batch-mode --no-transfer-progress clean verify -Pdesktop
open "target/dist/FM AI Assistent.app"
```

The JDK and build machine determine the output architecture. CI targets ARM64;
Intel packaging has not been validated. Stop the application using **Exit** in
its desktop control window. Closing a browser tab alone does not stop the server.

The macOS Maven profile selects application runtime modules explicitly. It
excludes the JDK packaging tools so Temurin's runtime linking can build the image
without JMOD archives ([Temurin explanation](https://adoptium.net/en-GB/news/2025/03/eclipse-temurin-jdk24-JEP493-enabled)).

For a headless application health check after building:

```bash
bash scripts/macos-smoke.sh
```

CI runs the full Maven verification gate, including unit/integration/architecture
tests, coverage, production frontend and SBOM, then starts the actual `.app`
launcher with a minimal PATH and checks `/actuator/health` and `/`. This verifies
startup, not graphical Finder interaction, a Codex account or a running FM game.

## Codex and ChatGPT

Install the official Codex CLI on the Mac and sign in using your ChatGPT account
through its normal login flow. The existing application uses the local Codex
app-server. The ChatGPT subscription's applicable Codex limits apply; this does
not turn the subscription into an OpenAI API key or promise unlimited usage.

Executable search keeps an explicit setting first, then PATH, then user
installations, Homebrew (`/opt/homebrew/bin` and `/usr/local/bin`) and NVM.
For a custom install, launch the app with:

```bash
"target/dist/FM AI Assistent.app/Contents/MacOS/FM AI Assistent" \
  --app.codex.executable=/absolute/path/to/codex
```

No account credentials or model access are bundled in the app. Do not commit
Codex authentication files, game saves or local database snapshots.

## Local folders and overrides

| Purpose | Default on macOS |
| --- | --- |
| Steam | `~/Library/Application Support/Steam` |
| FM27 tactics | `~/Library/Application Support/Sports Interactive/Football Manager 27/tactics` |
| FM27 shortlists | `~/Library/Application Support/Sports Interactive/Football Manager 27/shortlists` |
| Packaged AI workspace | `~/Library/Application Support/FM27 Assistant/workspace` |
| Existing database and desktop log | `~/.fm-ai-assistent/` |

Custom FM user-data locations are supported through Java system properties:
`fmaiassistent.tactics.directory`, `fmaiassistent.shortlists.directory`, and
`fmaiassistent.steam.directory`. Example for a custom tactics folder:

```bash
JAVA_TOOL_OPTIONS='-Dfmaiassistent.tactics.directory="/Users/you/FM Data/tactics"' \
  "target/dist/FM AI Assistent.app/Contents/MacOS/FM AI Assistent"
```

The desktop profile binds HTTP/MCP to `127.0.0.1`. AI workspaces remain outside
the application bundle, which may be read-only after installation.

## Next acceptance milestone: actual squad advice

1. Validate app launch, Codex sign-in and a real exported tactic on the M5 Mac.
2. Establish a supported Mac game-data route: structured user exports first, or
   a native reader only if the game's process permissions and layout permit it.
3. Record FM build, architecture and field provenance; validate player identity,
   attribute ranges and data freshness before replacing a stored snapshot.
4. Add bounded parsing and malformed/stale-data fixtures. Reject incompatible
   layouts without discarding the previous snapshot.
5. Compare tactical recommendations with the same squad in FM, then enable
   squad features for that verified route.

There is no automated change to the running game's tactics in this preview.
Signing/notarization and Intel testing are separate distribution milestones.

## Source provenance

Imported from [JelmerBouma1985/fm-ai-assistent](https://github.com/JelmerBouma1985/fm-ai-assistent)
at commit `a06ac845acfd6c0eccbd61e3a633b3518584e457`. The imported README is retained
as `README_UPSTREAM.md`. No new license grant is asserted for upstream code; this
repository remains private. The original snapshot contains no root LICENSE file.
