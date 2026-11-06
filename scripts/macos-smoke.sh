#!/usr/bin/env bash
# Verify the real .app launcher and its bundled runtime without a GUI or game save.
set -euo pipefail

if [[ "$(uname -s)" != Darwin ]]; then
  echo "This smoke test requires macOS." >&2
  exit 1
fi

app_path="${1:-target/dist/FM AI Assistent.app}"
launcher="$app_path/Contents/MacOS/FM AI Assistent"
if [[ ! -x "$launcher" ]]; then
  echo "Missing packaged launcher: $launcher" >&2
  exit 1
fi

mkdir -p target
log_file="$PWD/target/macos-smoke.log"
port=18080
PATH=/usr/bin:/bin:/usr/sbin:/sbin \
  JAVA_TOOL_OPTIONS="${JAVA_TOOL_OPTIONS:-} -Djava.awt.headless=true" \
  "$launcher" --server.port="$port" \
    '--spring.datasource.url=jdbc:h2:mem:macos_smoke;DB_CLOSE_ON_EXIT=FALSE' \
    >"$log_file" 2>&1 &
app_pid=$!

cleanup() {
  kill "$app_pid" 2>/dev/null || true
  for attempt in {1..25}; do
    kill -0 "$app_pid" 2>/dev/null || break
    sleep 1
  done
  kill -9 "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
}
trap cleanup EXIT

for attempt in {1..120}; do
  if ! kill -0 "$app_pid" 2>/dev/null; then
    tail -n 100 "$log_file" >&2
    echo "Packaged application exited before becoming healthy." >&2
    exit 1
  fi
  health="$(curl --silent --fail --max-time 2 "http://127.0.0.1:$port/actuator/health" || true)"
  if [[ "$health" == *'"status":"UP"'* ]]; then
    code="$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 15 "http://127.0.0.1:$port/")"
    if [[ "$code" != 200 ]]; then
      echo "Application is healthy but root returned HTTP $code." >&2
      exit 1
    fi
    echo "PASS: packaged runtime started, health is UP and root returned HTTP 200."
    exit 0
  fi
  sleep 1
done

tail -n 100 "$log_file" >&2
echo "Timed out waiting for the packaged application." >&2
exit 1
