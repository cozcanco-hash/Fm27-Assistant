# MacBook testing guide

Thank you for testing FM27 Assistant. The current preview is for **Apple Silicon Macs (M1 or later)** and has a Turkish interface. You do not need Football Manager or an AI account for the local squad and tactics features.

## Install

1. Open [Rust desktop builds](https://github.com/cozcanco-hash/Fm27-Assistant/actions/workflows/rust-desktop.yml).
2. Select the latest successful run on `main` and download the **FM27-Assistant-rust-macos-arm64** artifact. A GitHub account may be required.
3. Extract it, open the `.dmg` and copy the app to Applications. The download also contains an app ZIP and `SHA256SUMS.txt`.
4. Open **FM27 Assistant**. This developer preview is ad-hoc signed, without notarization. If macOS blocks it, review the app approval under **System Settings → Privacy & Security** after attempting to open it.

Test only a build from this repository. The version appears in the sidebar. No developer tools are needed for installation.

## Ten-minute local check

| Step | Action | Expected result |
| --- | --- | --- |
| 1 | Launch the app. | The overview opens with KUTRA FC, 24 players and an explicit demo-data label. |
| 2 | Open **Kadro** and search for `Deniz`; filter to `GK`. | Deniz Arslan is the matching goalkeeper. |
| 3 | Open the player's profile, then press Escape. | Attributes and player context are visible; Escape closes the profile. |
| 4 | Open **Taktik laboratuvarı**; try all three formations. | The board updates with 11 unique eligible players and role-fit information. |
| 5 | Leave a different formation selected, quit fully and relaunch. | The selected formation is preserved. |
| 6 | Open **Veri & bağlantı** and import [demo-squad.json](desktop/fixtures/demo-squad.json), downloaded as a raw JSON file. | Valid sample data loads with its fictional/demo provenance. |
| 7 | Try importing a `.json` file containing `invalid json`. | A validation error appears; the last valid squad remains available. |
| 8 | Resize the window and navigate between all sections. | Text, controls and the tactic board remain usable. |

Imported data follows [the documented schema](desktop/README.md#data-contract). This is not a live FM connection. The browser preview cannot test native JSON import or persistence.

## Optional ChatGPT check

This part requires the official Codex CLI and a ChatGPT account with Codex access. Use your own account; usage limits apply.

1. Install Codex from its official distribution if it is not already available.
2. In **Veri & bağlantı**, check the connection and follow the ChatGPT sign-in flow when needed. Keep credentials in the official sign-in flow.
3. Open **AI asistan**, choose an available model and ask which of the three formations fits the current squad. The response should stream into the chat.
4. Send a follow-up about a named player. Check whether the answer respects the fictional data and actual attributes. Try stopping a response.

This is a real-account acceptance check; it has not been claimed as passed by the automated tests. The app supplies the selected XI, up to 20 alternatives and bounded recent conversation when a message is sent. Advice should acknowledge missing data. It cannot control the game.

## Report findings

Use [GitHub Issues](https://github.com/cozcanco-hash/Fm27-Assistant/issues) with:

- Mac chip, macOS version and app version.
- The action you tried, what you expected and what happened.
- Whether the issue repeats after quitting and reopening the app.
- A screenshot or exact error text, with account details removed.

Do not include passwords, tokens or other sign-in credentials. Useful feedback includes layout problems, unclear explanations, bad lineup choices and AI connection errors, as well as crashes.
