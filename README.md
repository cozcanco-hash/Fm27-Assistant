# FM27 Assistant for macOS

Private development workspace for Can Özcan's Football Manager 27 AI assistant.

## Status

macOS adaptation in progress. The original project currently supports Windows and Linux; this repository does not yet claim a tested macOS live-game connection.

## Upstream

- Project: https://github.com/JelmerBouma1985/fm-ai-assistent
- Baseline commit: `a06ac845acfd6c0eccbd61e3a633b3518584e457`
- Upstream author: JelmerBouma1985 and contributors

The upstream source will be imported with attribution. No new license or redistribution grant is asserted for upstream code.

## Goals

1. Package an Apple Silicon macOS application with its Java runtime.
2. Discover native FM27 tactics, shortlists and Steam installations.
3. Connect the user's locally authenticated Codex CLI.
4. Validate a macOS-specific game-data integration before enabling live squad analysis.

No game saves, account credentials or API keys belong in this repository.
