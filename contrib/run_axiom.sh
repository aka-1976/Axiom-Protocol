#!/usr/bin/env bash
set -euo pipefail
# Simple supervisor: restarts the node binary if it exits

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN1="$ROOT_DIR/target/release/axiom-core"
BIN2="$ROOT_DIR/target/release/axiom"
LOG="$ROOT_DIR/node.log"

if [ -x "$BIN1" ]; then
    BIN="$BIN1"
elif [ -x "$BIN2" ]; then
    BIN="$BIN2"
else
    echo "No built binary found. Run 'cargo build --release' first." >&2
    exit 1
fi

echo "Supervisor starting: $BIN" >> "$LOG"
while true; do
    echo "$(date --iso-8601=seconds) - starting $BIN" >> "$LOG"
    "$BIN" >> "$LOG" 2>&1 || true
    echo "$(date --iso-8601=seconds) - $BIN exited; restarting in 5s" >> "$LOG"
    sleep 5
done
