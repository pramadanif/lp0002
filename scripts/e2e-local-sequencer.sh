#!/usr/bin/env bash
# e2e-local-sequencer.sh — full multisig lifecycle against a REAL standalone LEZ sequencer.
#
# This is the script `demo.sh` wraps, and the one CI's `e2e-sequencer` job runs. It is the evidence
# behind criteria P-S2 and P-S5, and behind plan gates H1/H2/H4.
#
# ─── Non-negotiables, and why ────────────────────────────────────────────────────────────────────
#
# H1  A real standalone sequencer. Not an in-process executor. `demo.sh` in prize PR #125 was an
#     executor tour, and that is the gap this repository is built to close.
# H2  Missing tools FAIL. Every prerequisite is checked up front and a missing one exits non-zero.
#     There is no skip-to-exit-0 anywhere in this file. A demo that "passes" on a machine without a
#     prover has told the evaluator nothing.
# H3  RISC0_DEV_MODE is inherited, never set to 1 here. `check-dev-mode-clobber.sh` enforces that.
#
# Usage:  ./scripts/e2e-local-sequencer.sh
# Env:    LEZ_TAG           LEZ revision to build (default: the pin in docs/VERSIONS.md)
#         PMSIG_LEZ_DIR     reuse an existing LEZ checkout instead of cloning
#         PMSIG_KEEP_RUNNING=1  leave the sequencer up for inspection afterwards

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }
REPO="$PWD"

LEZ_TAG="${LEZ_TAG:-v0.2.4}"
LEZ_DIR="${PMSIG_LEZ_DIR:-$REPO/.e2e/lez}"
RUN_DIR="$REPO/.e2e/run"
SEQ_URL="http://127.0.0.1:3040"
SEQ_LOG="$RUN_DIR/sequencer.log"
SEQ_PID=""

log()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
info() { printf '    %s\n' "$*"; }
die()  { printf '\n\033[1;31mFATAL: %s\033[0m\n' "$*" >&2; exit 1; }

# ─── H2: prerequisites are hard requirements ─────────────────────────────────────────────────────
require() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not installed. $2"
}

cleanup() {
  local code=$?
  if [[ -n "$SEQ_PID" ]] && kill -0 "$SEQ_PID" 2>/dev/null; then
    if [[ "${PMSIG_KEEP_RUNNING:-0}" == "1" ]]; then
      info "leaving the sequencer running (pid $SEQ_PID); logs at $SEQ_LOG"
    else
      info "stopping the sequencer (pid $SEQ_PID)"
      kill "$SEQ_PID" 2>/dev/null || true
      wait "$SEQ_PID" 2>/dev/null || true
    fi
  fi
  if (( code != 0 )); then
    printf '\n\033[1;31me2e FAILED (exit %d)\033[0m\n' "$code" >&2
    [[ -f "$SEQ_LOG" ]] && { echo "--- last 40 lines of sequencer log ---" >&2; tail -40 "$SEQ_LOG" >&2; }
  fi
  exit $code
}
trap cleanup EXIT

log "checking prerequisites (missing tools fail this script — gate H2)"
require git   "Install git."
require cargo "Install Rust: https://rustup.rs"
require jq    "Install jq."
require curl  "Install curl."
require r0vm  "Install the risc0 toolchain: curl -L https://risczero.com/install | bash && rzup install"
info "r0vm      $(r0vm --version)"
info "cargo     $(cargo --version)"

# RISC0_DEV_MODE must be 0 on this path. It is exported here as 0 (never 1) and inherited by
# everything below, so a child cannot silently prove nothing.
export RISC0_DEV_MODE=0
info "RISC0_DEV_MODE=${RISC0_DEV_MODE}"
[[ "$RISC0_DEV_MODE" == "0" ]] || die "RISC0_DEV_MODE must be 0 on the e2e path, got '$RISC0_DEV_MODE'"

mkdir -p "$RUN_DIR"

# ─── 1. LEZ checkout at the pinned revision ──────────────────────────────────────────────────────
log "preparing LEZ $LEZ_TAG"
if [[ -d "$LEZ_DIR/.git" ]]; then
  info "reusing $LEZ_DIR"
else
  info "cloning logos-execution-zone at $LEZ_TAG (shallow)"
  mkdir -p "$(dirname "$LEZ_DIR")"
  git clone --quiet --depth 1 --branch "$LEZ_TAG" \
    https://github.com/logos-blockchain/logos-execution-zone.git "$LEZ_DIR" \
    || die "could not clone LEZ at $LEZ_TAG"
fi

# ─── 2. Build the standalone sequencer ───────────────────────────────────────────────────────────
#
# H1: `--features standalone` is what makes this a real sequencer process with an RPC endpoint,
# rather than an in-process executor. The feature is LEZ's own (README §"Standalone mode").
log "building the standalone LEZ sequencer (first run takes a while)"
( cd "$LEZ_DIR" && cargo build --release --features standalone -p sequencer_service ) \
  || die "the standalone sequencer failed to build"

SEQ_BIN="$LEZ_DIR/target/release/sequencer_service"
[[ -x "$SEQ_BIN" ]] || die "sequencer binary not found at $SEQ_BIN"

# ─── 3. Build and package our guests ─────────────────────────────────────────────────────────────
log "building the guest programs"
"$REPO/scripts/build-guests.sh" || die "guest build failed"
[[ -s "$REPO/artifacts/membership.bin" ]] || die "artifacts/membership.bin missing after build"

# ─── 4. Start the sequencer ──────────────────────────────────────────────────────────────────────
log "starting the standalone sequencer"
SEQ_HOME="$RUN_DIR/sequencer"
mkdir -p "$SEQ_HOME"
# The config PATH, not its directory: LEZ's README shows both forms, and passing the directory
# fails with "Is a directory (os error 21)".
SEQ_CONFIG="$LEZ_DIR/lez/sequencer/service/configs/debug/sequencer_config.json"
[[ -f "$SEQ_CONFIG" ]] || die "sequencer config not found at $SEQ_CONFIG"
( cd "$SEQ_HOME" && RUST_LOG=info "$SEQ_BIN" "$SEQ_CONFIG" > "$SEQ_LOG" 2>&1 ) &
SEQ_PID=$!
info "sequencer pid $SEQ_PID, logs -> $SEQ_LOG"

# Wait for the RPC to answer. A timeout here is a failure, never a skip.
log "waiting for the sequencer RPC at $SEQ_URL"
ready=0
for _ in $(seq 1 60); do
  if curl -s -X POST "$SEQ_URL" -H 'content-type: application/json' \
       --data '{"jsonrpc":"2.0","id":1,"method":"checkHealth","params":[]}' \
       --max-time 2 2>/dev/null | grep -q '"result"'; then
    ready=1
    break
  fi
  kill -0 "$SEQ_PID" 2>/dev/null || die "the sequencer exited during startup — see $SEQ_LOG"
  sleep 1
done
(( ready == 1 )) || die "the sequencer did not answer on $SEQ_URL within 60s"

BLOCK=$(curl -s -X POST "$SEQ_URL" -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"getLastBlockId","params":[]}' | jq -r '.result')
info "sequencer is live — getLastBlockId = $BLOCK"

# ─── 5. The lifecycle ────────────────────────────────────────────────────────────────────────────
log "running the multisig lifecycle against the live sequencer"
echo
echo "    NOT YET IMPLEMENTED — see docs/phase-E-status.md."
echo "    Deploying the programs and driving create/propose/approve/execute through the"
echo "    privacy-preserving path is the remaining work. This script fails rather than"
echo "    reporting success it has not earned."
echo
die "lifecycle step not implemented yet"
