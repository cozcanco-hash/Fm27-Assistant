#!/usr/bin/env bash
set -euo pipefail
if [[ "$(uname -s)" != Darwin ]]; then
  echo 'This test needs macOS.' >&2
  exit 1
fi
app_path="${1:-target/release/bundle/macos/FM27 Assistant.app}"
app_path="$(cd "$(dirname "$app_path")" && pwd)/$(basename "$app_path")"
binary="$app_path/Contents/MacOS/fm27-assistant"
test -x "$binary"
mkdir -p target/smoke-data
report="$PWD/target/native-smoke.json"
rm -f "$report"
FM27_DATA_DIR="$PWD/target/smoke-data" FM27_SMOKE_REPORT="$report" FM27_SMOKE_EXIT=1 \
  PATH=/usr/bin:/bin:/usr/sbin:/sbin "$binary" > target/native-smoke.log 2>&1 &
app_pid=$!
cleanup() {
  kill "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
}
trap cleanup EXIT
for attempt in {1..90}; do
  if [[ -s "$report" ]]; then
    python3 - "$report" <<'PY'
import json, sys
report = json.load(open(sys.argv[1]))
assert report['ready'] is True
assert report['players'] == 24 and report['filled'] == 11
assert report['architecture'] == 'aarch64'
print(json.dumps(report))
PY
    for attempt in {1..10}; do
      kill -0 "$app_pid" 2>/dev/null || break
      sleep 1
    done
    echo 'PASS: packaged Rust app opened its WebView, loaded React and queried the Rust/SQLite core.'
    exit 0
  fi
  if ! kill -0 "$app_pid" 2>/dev/null; then
    cat target/native-smoke.log >&2
    echo 'Application exited before frontend readiness.' >&2
    exit 1
  fi
  sleep 1
done
cat target/native-smoke.log >&2
echo 'Timed out waiting for frontend readiness.' >&2
exit 1
