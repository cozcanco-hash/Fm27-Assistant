# Local bridge protocol 1

UTF-8, one JSON object per line, one request and reply per TCP connection.
The server binds only to IPv4 loopback. Discovery is atomically replaced on
startup; protocol, address, port and 32 random bytes represented as hex form the
private `bridge.json`. The directory is mode 0700 and file is mode 0600 on Unix.
The host retains an exclusive filesystem lease for its workspace.

Every request includes `protocol: 1`, `token`, `method`, `params`. Unsupported
versions, incorrect tokens and unknown fields are rejected before dispatch.
Request limit: 16 KiB including newline. Reply limit: 3 MiB. Eight concurrent
connections maximum; 10-second host and 5-second client deadlines. No unbounded
read-line or client-provided host name is used.

| Method | Params | Result |
| --- | --- | --- |
| `state` | `null` | Canonical conversation, roster provenance, shared model, pending/live answer |
| `send` | `{scope,prompt}` | Canonical state after Rust has persisted and reserved the turn |
| `cancel` | `{turnId}` | `null`; rejects a stale turn ID |
| `setModel` | `{scope,model}` | Canonical state; `model:null` restores the CLI default |

Success: `{protocol:1,ok:true,result:...}`.
Failure: `{protocol:1,ok:false,error:"..."}`. A refused authentication never
returns conversation state. Input errors are not echoed verbatim.

The server owns history, scoring, account access and prompt construction.
Clients poll `state` while a turn runs; `live` is transient. Only successfully
persisted final answers have `state:complete`. Failed/interrupted messages are
visible but excluded from subsequent model history. A host restart marks any
persisted pending response as interrupted. It does not resume or resend a turn.

The desktop can import rosters and change formation through its native commands;
the plugin transport cannot. Roster/formation/model changes are rejected during
an active turn. `scope` identifies the loaded career, or one isolated unidentified
snapshot. A model request from a previous scope is never silently redirected.

A send may have been accepted even if its acknowledgement is lost. Clients must
refresh `state` and let the user decide; they must not automatically retry sends.
This prototype intentionally exposes no HTTP, shell, arbitrary file access,
login URL, credential read, or game-save modification endpoint.
