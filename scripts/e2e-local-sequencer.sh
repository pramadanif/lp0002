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

# Runs a command with a wall-clock limit. macOS has no `timeout(1)`, and every wallet call on this
# path was unbounded — `wallet check-health` hung for four hours with no output, no CPU and no open
# socket, and the demo waited on it the whole time. A demo that can hang forever is worse than one
# that fails: the failure at least tells you something.
#
# Proving is deliberately NOT wrapped: it legitimately takes ~20 minutes and has its own heartbeat.
with_timeout() { # seconds, then command...
  local secs="$1"; shift
  "$@" &
  local cmd_pid=$!
  ( sleep "$secs"; kill -0 "$cmd_pid" 2>/dev/null && kill "$cmd_pid" 2>/dev/null ) &
  local killer=$!
  local rc=0
  wait "$cmd_pid" 2>/dev/null || rc=$?
  kill "$killer" 2>/dev/null || true
  return "$rc"
}

# Free memory in GB, or empty when it cannot be determined.
#
# Not decoration. A composed approval peaks near 9 GB (docs/cu-costs.md); below that the prover does
# not fail, it swaps — and the first attempt at this ran for hours before anyone worked out that
# Chrome and two editors were holding the memory. Hours of thrashing look exactly like slow proving,
# which is the worst failure mode there is: it wastes the time and teaches you nothing.
free_gb() {
  if [[ "$(uname)" == "Darwin" ]]; then
    vm_stat 2>/dev/null | awk '
      /page size of/ {ps=$8}
      /Pages free/        {gsub(/\./,"",$3); f=$3}
      /Pages inactive/    {gsub(/\./,"",$3); i=$3}
      /Pages speculative/ {gsub(/\./,"",$3); s=$3}
      END {if (ps>0) printf "%.1f", (f+i+s)*ps/1073741824}'
  else
    awk '/MemAvailable/ {printf "%.1f", $2/1048576}' /proc/meminfo 2>/dev/null
  fi
}

# Refuses to start a ~20-minute prove that the machine cannot hold. Override with
# PMSIG_MIN_FREE_GB=0 if you know better than this check — it is a resource precondition, not a
# correctness one, and nothing about the proof changes if you clear it.
require_free_ram() {
  local need="${PMSIG_MIN_FREE_GB:-9}" have
  have=$(free_gb)
  if [[ -z "$have" ]]; then
    info "could not measure free memory on this platform; skipping the check"
    return 0
  fi
  info "free memory: ${have} GB (need ~${need} GB)"
  if [[ "$need" != "0" ]] && awk "BEGIN{exit !($have < $need)}"; then
    die "only ${have} GB of memory is free; a composed approval needs about ${need} GB.
       Below that the prover swaps and a twenty-minute proof can take hours, which is
       indistinguishable from a hang. Close what is holding memory (browsers and editors are
       the usual culprits) and re-run. To proceed anyway: PMSIG_MIN_FREE_GB=0"
  fi
}

# ─── H2: prerequisites are hard requirements ─────────────────────────────────────────────────────
require() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not installed. $2"
}

cleanup() {
  local code=$?
  # An abort must never look like a pass. `code` alone did not hold: a `set -u` failure on an
  # unbound variable produced exit 0 here, and demo.sh is the prize's evidence — a false green in it
  # is worse than any bug it could hide. So success additionally requires the script to have reached
  # its own end and said so.
  if (( code == 0 )) && [[ "${E2E_COMPLETED:-0}" != "1" ]]; then
    printf '\n\033[1;31me2e ABORTED before completing, but exited 0 — treating as failure\033[0m\n' >&2
    code=70
  fi
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

# A run starts from a clean chain. The sequencer's data and the demo wallet both live under
# $RUN_DIR, and keeping them meant a second run reused the same shielded accounts, hence the same
# member_root, hence the same config_hash, hence the same PDA — and `#[account(init)]` correctly
# refused to create a multisig that already existed. So this demo worked exactly once per machine
# and failed every time after, which is the opposite of what a reviewer running it needs.
#
# Set PMSIG_KEEP_RUN=1 to keep the previous run's state for inspection.
if [[ -d "$RUN_DIR" && "${PMSIG_KEEP_RUN:-0}" != "1" ]]; then
  info "clearing the previous run ($RUN_DIR) — set PMSIG_KEEP_RUN=1 to keep it"
  rm -rf "$RUN_DIR"
fi
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

# Free the port before starting, and verify it. A sequencer left behind by an earlier run answers
# the readiness check below on the *old* chain, so ours can fail to start — "Address already in use"
# — while the run carries on talking to a stale node. That is how a create_multisig came back as
# "Transaction NOT confirmed": the account already existed, on a chain this run thought it had wiped.
SEQ_PORT=${SEQ_URL##*:}
if command -v lsof >/dev/null 2>&1; then
  holders=$(lsof -ti :"$SEQ_PORT" 2>/dev/null || true)
  if [[ -n "$holders" ]]; then
    info "port $SEQ_PORT is held by pid(s) $holders — stopping them first"
    # shellcheck disable=SC2086
    kill $holders 2>/dev/null || true
    for _ in $(seq 1 20); do
      lsof -ti :"$SEQ_PORT" >/dev/null 2>&1 || break
      sleep 1
    done
    lsof -ti :"$SEQ_PORT" >/dev/null 2>&1 \
      && die "port $SEQ_PORT is still held after 20s by pid(s) $(lsof -ti :"$SEQ_PORT" | tr '\n' ' ')"
  fi
fi

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

# Something answered — make sure it was ours. A startup failure that leaves an older node serving
# would otherwise pass this check silently, and every later step would run against the wrong chain.
kill -0 "$SEQ_PID" 2>/dev/null \
  || die "something is answering on $SEQ_URL but our sequencer is not running — see $SEQ_LOG"
if grep -qiE 'address already in use|failed to create HTTP listener' "$SEQ_LOG" 2>/dev/null; then
  tail -10 "$SEQ_LOG" >&2
  die "our sequencer could not bind its ports, so the node answering $SEQ_URL is someone else's.
       Stop it and re-run; see $SEQ_LOG"
fi

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
with_timeout 60 "$WALLET" check-health >/dev/null 2>&1 \
  || die "the wallet could not reach the sequencer within 60s (it hangs rather than erroring)"

# A 2-of-3 needs two shielded members. Top up to two; `account new private` is idempotent enough
# because we only ever count the first two.
privates=$(python3 -c "
import json,sys
try: d=json.load(open('$LEE_WALLET_HOME_DIR/storage.json'))
except Exception: print(0); sys.exit()
print(sum(1 for a in d['key_chain']['accounts'] if 'Private' in a))")
while (( privates < 2 )); do
  info "creating a shielded account ($((privates+1)) of 2)"
  with_timeout 120 "$WALLET" account new private >/dev/null 2>&1 \
    || die "could not create a shielded account within 120s"
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
# The demo moves a modest amount: a faucet claim is 150, and the proposal must be payable out of
# the multisig's own treasury (INV-7). Defined here, before the faucet loop reads TREASURY_AMOUNT.
TRANSFER_AMOUNT=100
TREASURY_AMOUNT=100

# ─── Fund the creator from the local Piñata faucet ───────────────────────────────────────────────
#
# The wallet on a fresh local sequencer has nothing, and the treasury transfer below fails with
# "Can not pay for operation". Nothing needed funds before: every earlier step submits a
# transaction, none of them *moves* value.
#
# Piñata is one of LEZ's built-in programs, so it exists on a standalone node exactly as it does on
# the public testnet. (scripts/fund-testnet.sh refuses localhost on the assumption that a faucet is
# a testnet thing — that assumption is wrong, and this is why.)
log "funding the demo wallet from the local Piñata faucet"
creator_id=${CREATOR#Public/}
lez_balance() {
  curl -s -X POST "$SEQ_URL" -H 'content-type: application/json' \
    --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountBalance\",\"params\":[\"$creator_id\"]}" \
    --max-time 20 | jq -r '.result // 0'
}

# The faucet will not pay an uninitialised account, and `auth-transfer init` hangs on an already
# initialised one rather than erroring — so the nonce decides. A nonce above zero means used, hence
# initialised. (Learned the hard way; see docs/tried-failed.md.)
#
# That reading only holds while nothing else has touched the account, which is why this runs first.
# It used to sit after the deploy, the multisig and the proposal — each of which bumps the nonce —
# so init was skipped on an account that had never been registered with auth-transfer, and the
# transfer below died with UnauthorizedBalanceDecrease: the account was not owned by the program
# trying to debit it. The nonce is a proxy for "has been used", not for "is initialised"; the two
# coincide only when funding comes first.
nonce=$(curl -s -X POST "$SEQ_URL" -H 'content-type: application/json' \
  --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccount\",\"params\":[\"$creator_id\"]}" \
  --max-time 20 | jq -r '.result.nonce // 0')
if [[ "$nonce" == "0" ]]; then
  "$WALLET" auth-transfer init --account-id "$CREATOR" > "$RUN_DIR/init.log" 2>&1 || true
  grep -q 'included in block' "$RUN_DIR/init.log" \
    || { tail -5 "$RUN_DIR/init.log" >&2; die "could not initialise the demo account"; }
fi

# Each claim is a fixed amount, so claim until the treasury transfer is affordable.
claims=0
while [[ "$(lez_balance)" -lt "$TREASURY_AMOUNT" ]]; do
  claims=$((claims + 1))
  [[ "$claims" -le 8 ]] || die "claimed from the faucet $((claims - 1)) times and the balance is still \
       $(lez_balance), below the $TREASURY_AMOUNT this demo moves. See $RUN_DIR/claim.log"
  "$WALLET" pinata claim --to "$CREATOR" > "$RUN_DIR/claim.log" 2>&1 || true
  grep -q 'included in block' "$RUN_DIR/claim.log" \
    || { tail -5 "$RUN_DIR/claim.log" >&2; die "the faucet claim was not included in a block"; }
done
info "wallet balance: $(lez_balance) after $claims claim(s)"

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

# ─── Fund the multisig's own treasury ────────────────────────────────────────────────────────────
#
# INV-7 made `execute` pay out of the multisig's own config PDA rather than a caller-supplied
# treasury account. Nothing funds that PDA, so `execute` failed with "Transaction NOT confirmed" —
# on the public testnet and in CI, both times after two real ~20-minute proofs had already
# succeeded. The old shape could not have worked either: the caller-supplied treasury was $CREATOR,
# and the proposal asked for more than it held.
#
# This is the step that was missing. It runs before the proposal so the funds are in place by the
# time the threshold is reached.
log "funding the multisig's treasury ($CONFIG_PDA)"
[[ -n "${CONFIG_PDA:-}" ]] || die "CONFIG_PDA was not derived — is examples/wallet_member.rs emitting it?"
fund_out=$("$WALLET" auth-transfer send \
  --from "$CREATOR" --to "Public/$CONFIG_PDA" --amount "$TREASURY_AMOUNT" 2>&1) \
  || { printf '%s\n' "$fund_out" >&2; die "could not fund the multisig treasury"; }
info "funded with $TREASURY_AMOUNT — $(printf '%s\n' "$fund_out" | awk '/included in block/{print; exit}')"

# One variable for both steps. `execute` refuses a recipient the proposal did not name (INV-7), so
# these must agree; they used to be written out separately and disagreed.
# The payee is a second public account, created and initialised for the purpose. Three LEZ rules
# between them leave no other option, and each was learned by having a transaction rejected:
#
#   * a never-used account cannot be credited at all — `DefaultAccountModifiedWithoutClaim` — and
#     claiming it is wrong, since a multisig must not take ownership of the account it pays;
#   * an account whose owner is default but whose state is not is refused by `validate_execution`
#     rule 7, so the payee has to be registered with a program;
#   * the payee cannot simply be the submitter: LEZ requires unique account ids in a message, and
#     reusing $CREATOR produced "Duplicate account_ids found in message".
#
# So: a distinct account, already existing, owned by auth-transfer.
log "creating the payee account"
"$WALLET" account new public > "$RUN_DIR/payee.log" 2>&1 || die "could not create the payee account"
PAYEE=$("$WALLET" account list 2>/dev/null | awk '/Public\//{print $2}' | grep -v "^${CREATOR}$" | head -1)
[[ -n "$PAYEE" ]] || die "no second public account after creating one — see $RUN_DIR/payee.log"
with_timeout 120 "$WALLET" auth-transfer init --account-id "$PAYEE" > "$RUN_DIR/payee-init.log" 2>&1 || true
grep -q 'included in block' "$RUN_DIR/payee-init.log" \
  || { tail -5 "$RUN_DIR/payee-init.log" >&2; die "could not initialise the payee account"; }

RECIPIENT=$(python3 -c "
import sys
A='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
s = sys.argv[1].split('/')[-1]
n = 0
for ch in s:
    n = n * 58 + A.index(ch)
print(f'{n:064x}')
" "$PAYEE")
[[ ${#RECIPIENT} -eq 64 ]] || die "could not derive a 32-byte recipient id from $PAYEE (got '$RECIPIENT')"
info "payee: $PAYEE ($RECIPIENT)"

log "submitting a treasury-transfer proposal"
spel_run -- create-proposal \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" --proposal-id "$PROPOSAL_ID" \
  --recipient "$RECIPIENT" \
  --amount "$TRANSFER_AMOUNT" --proposer "$CREATOR" \
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

require_free_ram
  log "approval $((i+1)) of 2 — shielded member, anonymous, RISC0_DEV_MODE=$RISC0_DEV_MODE"
  info "this proves a real proof and takes ~20 minutes; it is not hung"
  started=$(date +%s)
  "$SPEL" --idl "$REPO/artifacts/multisig-idl.json" -p "$REPO/artifacts/multisig.bin" \
    --bin-membership "$REPO/artifacts/membership.bin" -- \
    approve --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
    --member-root "$MEMBER_ROOT" --claimed-nullifier "$nf" \
    --witness "$(cat "$wit_file")" --approver "Private/$acct" \
    > "$RUN_DIR/approve$i.log" 2>&1 &
  spel_pid=$!

  # Heartbeat. Everything above prints as it goes; this step then produces nothing for twenty
  # minutes, which is indistinguishable from a hang — to a reviewer running ./demo.sh, to anyone
  # watching the video, and to us reading a CI job that had been "in progress" for an hour with no
  # way to tell proving from stuck. The r0vm RSS is the honest signal that real work is happening:
  # a composed approval peaks around 9 GB (docs/cu-costs.md), and dev mode would show none of it.
  while kill -0 "$spel_pid" 2>/dev/null; do
    sleep 60
    kill -0 "$spel_pid" 2>/dev/null || break
    hb_min=$(( ($(date +%s) - started) / 60 ))
    hb_rss=$(ps -A -o rss=,comm= 2>/dev/null \
             | awk '/r0vm/ {s += $1} END {if (s > 0) printf "%.1f GB", s/1048576}')
    hb_last=$(tail -n 1 "$RUN_DIR/approve$i.log" 2>/dev/null | tr -d '\r' | cut -c1-80)
    info "    … ${hb_min} min${hb_rss:+, r0vm ${hb_rss}}${hb_last:+ — ${hb_last}}"
  done

  wait "$spel_pid" \
    || { tail -25 "$RUN_DIR/approve$i.log" >&2; die "approval $((i+1)) failed"; }
  grep -q 'confirmed' "$RUN_DIR/approve$i.log" || die "approval $((i+1)) was not confirmed"
  info "approval $((i+1)) confirmed in $(( ($(date +%s)-started)/60 )) min"
  info "  tx $(awk '/tx_hash/{print $2}' "$RUN_DIR/approve$i.log" | head -1)"
done

# ─── 11. execute at full M ───────────────────────────────────────────────────────────────────────
log "executing the proposal (threshold reached with FULL M)"
spel_run -- execute \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
  --recipient "$RECIPIENT" --submitter "$CREATOR" \
  > "$RUN_DIR/execute.log" 2>&1 || { tail -20 "$RUN_DIR/execute.log" >&2; die "execute failed"; }
grep -q 'confirmed' "$RUN_DIR/execute.log" || die "execute was not confirmed"
info "executed and confirmed"

# ─── 12. Verify the on-chain state says what it should ───────────────────────────────────────────
log "verifying the on-chain state"
cargo run --quiet -p pmsig-sdk --example verify_onchain -- \
  "$SEQ_URL" "$REPO/artifacts/IMAGE_IDS.md" "$CONFIG_HASH" "$PROPOSAL_SEED" \
  || die "on-chain verification failed"

E2E_COMPLETED=1
log "DEMO COMPLETE"
cat <<EOF

  A 2-of-3 private multisig executed a treasury transfer on a real LEZ sequencer.

  Both approvals were proved with RISC0_DEV_MODE=$RISC0_DEV_MODE, from shielded accounts,
  through LEZ's privacy-preserving circuit. The chain recorded a count and two
  nullifiers — and no member identity anywhere.

  Logs: $RUN_DIR

EOF
