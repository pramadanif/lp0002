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
# SPEL main pins LEZ v0.2.4; the released v0.6.0 pins v0.2.0 and derives different account ids.
SPEL_REV="${SPEL_REV:-5126b7ed8a9b}"
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

# ─── 5. Supporting tools (wallet + SPEL CLI) ─────────────────────────────────────────────────────
log "building the LEZ wallet and the SPEL CLI"
( cd "$LEZ_DIR" && cargo build --release -p wallet ) || die "the LEZ wallet failed to build"
WALLET="$LEZ_DIR/target/release/wallet"

SPEL_DIR="${PMSIG_SPEL_DIR:-$REPO/.e2e/spel}"
if [[ ! -d "$SPEL_DIR/.git" ]]; then
  info "cloning SPEL at $SPEL_REV (shallow)"
  git clone --quiet https://github.com/logos-co/spel.git "$SPEL_DIR" || die "could not clone SPEL"
  ( cd "$SPEL_DIR" && git checkout --quiet "$SPEL_REV" ) || die "could not check out SPEL $SPEL_REV"
fi
( cd "$SPEL_DIR" && cargo build --release -p spel ) || die "the SPEL CLI failed to build"
SPEL="$SPEL_DIR/target/release/spel"

# ─── 6. Wallet with two shielded members ─────────────────────────────────────────────────────────
log "preparing a wallet with two shielded accounts"
export LEE_WALLET_HOME_DIR="$RUN_DIR/wallet"
mkdir -p "$LEE_WALLET_HOME_DIR"
cp -n "$LEZ_DIR/lez/wallet/configs/debug/wallet_config.json" "$LEE_WALLET_HOME_DIR/" 2>/dev/null || true
"$WALLET" check-health >/dev/null 2>&1 || die "the wallet could not reach the sequencer"

# A 2-of-3 needs two shielded members. Top up to two; `account new private` is idempotent enough
# because we only ever count the first two.
privates=$(python3 -c "
import json,sys
try: d=json.load(open('$LEE_WALLET_HOME_DIR/storage.json'))
except Exception: print(0); sys.exit()
print(sum(1 for a in d['key_chain']['accounts'] if 'Private' in a))")
while (( privates < 2 )); do
  info "creating a shielded account ($((privates+1)) of 2)"
  "$WALLET" account new private >/dev/null 2>&1 || die "could not create a shielded account"
  privates=$((privates+1))
done
info "wallet has $privates shielded accounts"

# Captured whole, then parsed — never piped straight into `awk ... exit`. Rust ignores SIGPIPE, so
# a reader that closes the pipe early makes the writer panic on its next print and exit 101, and
# `pipefail` then reports a successful command as failed. That is exactly how this line failed in
# CI. Keeping stderr means the next failure says what it was instead of vanishing into /dev/null.
wallet_accounts=$("$WALLET" account list 2>&1) \
  || die "the wallet could not list accounts: $wallet_accounts"
CREATOR=$(printf '%s\n' "$wallet_accounts" | awk '/Public\//{print $2; exit}')
[[ -n "$CREATOR" ]] || die "the wallet has no public account to pay with"
info "creator: $CREATOR"

# ─── 7. Deploy both programs ─────────────────────────────────────────────────────────────────────
log "deploying the programs"
for prog in membership multisig; do
  out=$("$WALLET" deploy-program "$REPO/artifacts/$prog.bin" 2>&1) || die "deploying $prog failed"
  blk=$(echo "$out" | awk '/included in block/{print $NF}')
  info "$prog deployed, block $blk"
done

# ─── 8. Derive the parameters ────────────────────────────────────────────────────────────────────
#
# config_hash commits to the member root, the threshold AND the membership verifier's ImageID
# (ADR-002), so it is derived from one place rather than typed by hand.
log "deriving multisig parameters from the wallet's own shielded accounts"
PARAMS=$(cargo run --quiet -p pmsig-sdk --example wallet_member -- \
          "$LEE_WALLET_HOME_DIR/storage.json" "$RUN_DIR/witness") \
  || die "could not derive parameters from the wallet"
eval "$(echo "$PARAMS" | grep '=')"
[[ "$CROSSCHECK_NPK" == "ok" && "$CROSSCHECK_ACCOUNT_ID" == "ok" ]] \
  || die "our LEZ derivation disagrees with the wallet's own accounts"
info "derivation cross-checked against the wallet's accounts: ok"
info "config_hash $CONFIG_HASH"

spel_run() { "$SPEL" --idl "$REPO/artifacts/multisig-idl.json" -p "$REPO/artifacts/multisig.bin" "$@"; }

# ─── 9. create → propose ─────────────────────────────────────────────────────────────────────────
log "creating the 2-of-3 multisig"
spel_run -- create-multisig \
  --config-hash "$CONFIG_HASH" --member-root "$MEMBER_ROOT" --m 2 --n 3 \
  --multisig-id "$MULTISIG_ID" --membership-program-id "$VERIFIER" --creator "$CREATOR" \
  > "$RUN_DIR/create.log" 2>&1 || { tail -20 "$RUN_DIR/create.log" >&2; die "create_multisig failed"; }
grep -q 'confirmed' "$RUN_DIR/create.log" || die "create_multisig was not confirmed"
info "multisig created and confirmed"

log "submitting a treasury-transfer proposal"
# One variable for both steps. `execute` refuses a recipient the proposal did not name (INV-7), so
# these must agree; they used to be written out separately and disagreed.
RECIPIENT=c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3
spel_run -- create-proposal \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" --proposal-id "$PROPOSAL_ID" \
  --recipient "$RECIPIENT" \
  --amount 1000 --proposer "$CREATOR" \
  > "$RUN_DIR/propose.log" 2>&1 || { tail -20 "$RUN_DIR/propose.log" >&2; die "create_proposal failed"; }
grep -q 'confirmed' "$RUN_DIR/propose.log" || die "create_proposal was not confirmed"
info "proposal created and confirmed"

# ─── 10. approve × M, anonymously, on the privacy-preserving path ───────────────────────────────
#
# H13/W15: the primary evidence path uses the FULL threshold, never a lowered tier.
#
# Each approval is a privacy-preserving transaction: LEZ's PPE circuit runs env::verify over the
# multisig program AND the chained membership program. That composition needs succinct receipts,
# i.e. recursion, so expect roughly 20 minutes and ~9 GB of FREE memory per approval on a laptop
# without a GPU prover. See docs/cu-costs.md.
for i in 0 1; do
  acct_var="MEMBER${i}_ACCOUNT"; nf_var="MEMBER${i}_NULLIFIER"; wit_var="MEMBER${i}_WITNESS"
  acct="${!acct_var}"; nf="${!nf_var}"; wit_file="${!wit_var}"
  [[ -s "$wit_file" ]] || die "witness file for member $i is missing ($wit_file)"

  log "approval $((i+1)) of 2 — shielded member, anonymous, RISC0_DEV_MODE=$RISC0_DEV_MODE"
  info "this proves a real proof and takes ~20 minutes; it is not hung"
  started=$(date +%s)
  "$SPEL" --idl "$REPO/artifacts/multisig-idl.json" -p "$REPO/artifacts/multisig.bin" \
    --bin-membership "$REPO/artifacts/membership.bin" -- \
    approve --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
    --member-root "$MEMBER_ROOT" --claimed-nullifier "$nf" \
    --witness "$(cat "$wit_file")" --approver "Private/$acct" \
    > "$RUN_DIR/approve$i.log" 2>&1 \
    || { tail -25 "$RUN_DIR/approve$i.log" >&2; die "approval $((i+1)) failed"; }
  grep -q 'confirmed' "$RUN_DIR/approve$i.log" || die "approval $((i+1)) was not confirmed"
  info "approval $((i+1)) confirmed in $(( ($(date +%s)-started)/60 )) min"
  info "  tx $(awk '/tx_hash/{print $2}' "$RUN_DIR/approve$i.log" | head -1)"
done

# ─── 11. execute at full M ───────────────────────────────────────────────────────────────────────
log "executing the proposal (threshold reached with FULL M)"
spel_run -- execute \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
  --recipient "$RECIPIENT" \
  > "$RUN_DIR/execute.log" 2>&1 || { tail -20 "$RUN_DIR/execute.log" >&2; die "execute failed"; }
grep -q 'confirmed' "$RUN_DIR/execute.log" || die "execute was not confirmed"
info "executed and confirmed"

# ─── 12. Verify the on-chain state says what it should ───────────────────────────────────────────
log "verifying the on-chain state"
cargo run --quiet -p pmsig-sdk --example verify_onchain -- \
  "$SEQ_URL" "$REPO/artifacts/IMAGE_IDS.md" "$CONFIG_HASH" "$PROPOSAL_SEED" \
  || die "on-chain verification failed"

log "DEMO COMPLETE"
cat <<EOF

  A 2-of-3 private multisig executed a treasury transfer on a real LEZ sequencer.

  Both approvals were proved with RISC0_DEV_MODE=$RISC0_DEV_MODE, from shielded accounts,
  through LEZ's privacy-preserving circuit. The chain recorded a count and two
  nullifiers — and no member identity anywhere.

  Logs: $RUN_DIR

EOF
