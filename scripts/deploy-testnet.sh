#!/usr/bin/env bash
# deploy-testnet.sh — deploy both programs to the public LEZ testnet and run the full lifecycle.
#
# Produces the evidence criteria P-F6, P-F7 and P-S1 require, and writes docs/DEPLOYMENT.md.
#
# ─── This script will not invent keys ───────────────────────────────────────────────────────────
#
# It needs a wallet that already holds funds on the public testnet. It will NOT generate one, fund
# one, or fall back to a local chain. If the wallet is missing or unfunded it stops and says so,
# because a deployment "succeeding" against the wrong network is how a submission ends up citing
# evidence that does not exist.
#
# Prerequisites (human gate):
#   1. A wallet home with a funded public account:
#        export LEE_WALLET_HOME_DIR=/path/to/wallet
#        wallet check-health
#   2. Two shielded accounts in that wallet:
#        wallet account new private     # twice
#
# Usage:  ./scripts/deploy-testnet.sh
# Env:    PMSIG_RPC        testnet RPC        (default https://testnet.lez.logos.co)
#         PMSIG_EXPLORER   explorer base URL  (default https://explorer.testnet.lez.logos.co)
#         PMSIG_LEZ_DIR / PMSIG_SPEL_DIR   reuse existing checkouts

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }
REPO="$PWD"

RPC="${PMSIG_RPC:-https://testnet.lez.logos.co}"
EXPLORER="${PMSIG_EXPLORER:-https://explorer.testnet.lez.logos.co}"
LEZ_DIR="${PMSIG_LEZ_DIR:-$REPO/.e2e/lez}"
SPEL_DIR="${PMSIG_SPEL_DIR:-$REPO/.e2e/spel}"
OUT="$REPO/.e2e/testnet"

die() { echo "FATAL: $*" >&2; exit 1; }

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
log() { printf '\n==> %s\n' "$*"; }
info() { printf '    %s\n' "$*"; }

export RISC0_DEV_MODE=0
mkdir -p "$OUT"

log "preconditions"
for t in cargo curl jq r0vm; do
  command -v "$t" >/dev/null 2>&1 || die "'$t' is required but not installed."
done
[[ -n "${LEE_WALLET_HOME_DIR:-}" ]] || die "LEE_WALLET_HOME_DIR is not set.
       This script needs a wallet that already holds testnet funds. It will not create one.
       See the header of this script."
[[ -f "$LEE_WALLET_HOME_DIR/wallet_config.json" ]] || die "no wallet_config.json in $LEE_WALLET_HOME_DIR"

WALLET="$LEZ_DIR/target/release/wallet"
SPEL="$SPEL_DIR/target/release/spel"
[[ -x "$WALLET" ]] || die "wallet binary not found at $WALLET — build it first (see scripts/e2e-local-sequencer.sh)"
[[ -x "$SPEL" ]] || die "spel binary not found at $SPEL"
[[ -s artifacts/membership.bin && -s artifacts/multisig.bin ]] || die "guest binaries missing. Run ./scripts/build-guests.sh --docker"

# The submission must quote a reproducible build, not a laptop build.
if grep -q 'NOT reproducible' artifacts/IMAGE_IDS.md; then
  die "artifacts/IMAGE_IDS.md records a LOCAL build.
       A deployed program must come from a reproducible build:
         ./scripts/build-guests.sh --docker"
fi

log "checking the testnet is reachable and is not a local node"
case "$RPC" in
  *127.0.0.1*|*localhost*) die "PMSIG_RPC points at a local node ($RPC). This script publishes evidence; it must target the public testnet." ;;
esac
curl -s -X POST "$RPC" -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"checkHealth","params":[]}' --max-time 20 \
  | grep -q '"result"' || die "$RPC did not answer checkHealth"
HEIGHT=$(curl -s -X POST "$RPC" -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":2,"method":"getLastBlockId","params":[]}' --max-time 20 | jq -r '.result')
info "testnet live at height $HEIGHT"

log "checking the wallet can reach it and holds funds"
"$WALLET" check-health >/dev/null 2>&1 || die "the wallet could not reach $RPC. Is its config pointing at the testnet?"
# Captured whole, then parsed — never piped straight into `awk ... exit`. Rust ignores SIGPIPE, so
# a reader that closes the pipe early makes the writer panic on its next print and exit 101, and
# `pipefail` then reports a successful command as failed. That is exactly how this line failed in
# CI. Keeping stderr means the next failure says what it was instead of vanishing into /dev/null.
wallet_accounts=$("$WALLET" account list 2>&1) \
  || die "the wallet could not list accounts: $wallet_accounts"
CREATOR=$(printf '%s\n' "$wallet_accounts" | awk '/Public\//{print $2; exit}')
[[ -n "$CREATOR" ]] || die "the wallet has no public account. Fund one on the testnet first."
# `account list` prints the id with a `Public/` prefix; the RPC wants the bare base58 and rejects
# the prefixed form with `InvalidBase58Character('l', 3)` — the l of "Public". fund-testnet.sh has
# always stripped it; this script did not.
PAYER_ID=${CREATOR#Public/}
BAL_RESP=$(curl -s -X POST "$RPC" -H 'content-type: application/json' \
  --data "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"getAccountBalance\",\"params\":[\"$PAYER_ID\"]}" --max-time 20)

# An RPC error is not a zero balance. `jq -r '.result // 0'` mapped both to 0, so a malformed
# request reported itself as an unfunded account and sent the operator off to fund a wallet that
# already held 450 — with the message "human gate", which is the worst possible thing to be wrong
# about, because it stops the run and blames the human.
if BAL_ERR=$(printf '%s' "$BAL_RESP" | jq -e -r '.error.message // empty' 2>/dev/null) && [[ -n "$BAL_ERR" ]]; then
  die "the RPC refused the balance query for $PAYER_ID: $BAL_ERR
       This is a bad request, not an empty account. Full response: $BAL_RESP"
fi
BAL=$(printf '%s' "$BAL_RESP" | jq -r '.result // empty')
[[ -n "$BAL" ]] || die "the RPC returned no balance for $PAYER_ID. Response: $BAL_RESP"
info "payer $CREATOR, balance $BAL"
[[ "$BAL" != "0" ]] || die "payer account $PAYER_ID really is empty. Run ./scripts/fund-testnet.sh"

privates=$(python3 -c "
import json,sys
try: d=json.load(open('$LEE_WALLET_HOME_DIR/storage.json'))
except Exception: print(0); sys.exit()
print(sum(1 for a in d['key_chain']['accounts'] if 'Private' in a))")
(( privates >= 2 )) || die "the wallet has $privates shielded accounts; a 2-of-3 needs 2.
       Create them with:  wallet account new private"
info "$privates shielded accounts available"

# ─── deploy ─────────────────────────────────────────────────────────────────────────────────────
log "deploying both programs to the testnet"
declare -a DEPLOY_TX
for prog in membership multisig; do
  out=$("$WALLET" deploy-program "$REPO/artifacts/$prog.bin" 2>&1) || { echo "$out" >&2; die "deploying $prog failed"; }
  tx=$(echo "$out" | awk '/Transaction hash is/{print $NF}')
  blk=$(echo "$out" | awk '/included in block/{print $NF}')
  [[ -n "$tx" ]] || die "could not read a transaction hash for $prog"
  DEPLOY_TX+=("$prog|$tx|$blk")
  info "$prog  tx $tx  block $blk"
done

# ─── lifecycle ──────────────────────────────────────────────────────────────────────────────────
log "deriving parameters from the wallet's own shielded accounts"
PARAMS=$(cargo run --quiet -p pmsig-sdk --example wallet_member -- \
          "$LEE_WALLET_HOME_DIR/storage.json" "$OUT/witness") || die "parameter derivation failed"
eval "$(echo "$PARAMS" | grep '=')"
[[ "$CROSSCHECK_ACCOUNT_ID" == "ok" ]] || die "our derivation disagrees with the wallet's accounts"

run_ix() { # name, then args
  local name="$1"; shift
  "$SPEL" --idl "$REPO/artifacts/multisig-idl.json" -p "$REPO/artifacts/multisig.bin" "$@" \
    > "$OUT/$name.log" 2>&1 || { tail -25 "$OUT/$name.log" >&2; die "$name failed"; }
  grep -q 'confirmed' "$OUT/$name.log" || { tail -25 "$OUT/$name.log" >&2; die "$name was not confirmed"; }
  awk '/tx_hash/{print $2; exit}' "$OUT/$name.log"
}

log "create_multisig (2-of-3)"
TX_CREATE=$(run_ix create -- create-multisig \
  --config-hash "$CONFIG_HASH" --member-root "$MEMBER_ROOT" --m 2 --n 3 \
  --multisig-id "$MULTISIG_ID" --membership-program-id "$VERIFIER" --creator "$CREATOR")
info "tx $TX_CREATE"

# The demo moves a modest amount: a faucet claim is 150, and the proposal must be payable out of
# the multisig's own treasury (INV-7). Both are defined here, before the funding step uses them.
TRANSFER_AMOUNT=100
TREASURY_AMOUNT=100

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

log "create_proposal (treasury transfer)"
# One variable for both steps: `execute` refuses a recipient the proposal did not name (INV-7).
# The payee is a second public account, created and initialised for the purpose. See the same block
# in scripts/e2e-local-sequencer.sh for the three LEZ rules that leave no other option: a never-used
# account cannot be credited, an account with a default owner and non-default state is refused by
# validate_execution rule 7, and the payee cannot be the submitter because account ids in a message
# must be unique.
log "creating the payee account"
"$WALLET" account new public > "$OUT/payee.log" 2>&1 || die "could not create the payee account"
PAYEE=$("$WALLET" account list 2>/dev/null | awk '/Public\//{print $2}' | grep -v "^${CREATOR}$" | head -1)
[[ -n "$PAYEE" ]] || die "no second public account after creating one — see $OUT/payee.log"
"$WALLET" auth-transfer init --account-id "$PAYEE" > "$OUT/payee-init.log" 2>&1 || true
grep -q 'included in block' "$OUT/payee-init.log" \
  || { tail -5 "$OUT/payee-init.log" >&2; die "could not initialise the payee account"; }

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
TX_PROPOSE=$(run_ix propose -- create-proposal \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" --proposal-id "$PROPOSAL_ID" \
  --recipient "$RECIPIENT" \
  --amount "$TRANSFER_AMOUNT" --proposer "$CREATOR")
info "tx $TX_PROPOSE"

# H13/W15: the published evidence uses the FULL threshold, never a lowered tier.
declare -a TX_APPROVE
for i in 0 1; do
  a="MEMBER${i}_ACCOUNT"; n="MEMBER${i}_NULLIFIER"; w="MEMBER${i}_WITNESS"
require_free_ram
  log "approve $((i+1)) of 2 — anonymous, privacy-preserving, RISC0_DEV_MODE=$RISC0_DEV_MODE"
  info "expect ~20 minutes and ~9 GB of free RAM; it is not hung"

  # Written to a file and parsed afterwards, never piped into `awk ... exit`. awk stops at the
  # first match and closes the pipe; Rust ignores SIGPIPE, so the prover panics on its next write
  # and exits 101 — a successful approval reported as a failure, on the path that produces the
  # submission's on-chain evidence. See docs/tried-failed.md; this was the third instance.
  started=$(date +%s)
  "$SPEL" --idl "$REPO/artifacts/multisig-idl.json" -p "$REPO/artifacts/multisig.bin" \
    --bin-membership "$REPO/artifacts/membership.bin" -- \
    approve --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
    --member-root "$MEMBER_ROOT" --claimed-nullifier "${!n}" \
    --witness "$(cat "${!w}")" --approver "Private/${!a}" \
    > "$OUT/approve$i.log" 2>&1 &
  spel_pid=$!

  # A silent twenty-minute step is indistinguishable from a hang. The r0vm RSS is the honest signal
  # that real proving is happening — dev mode would show none of it.
  while kill -0 "$spel_pid" 2>/dev/null; do
    sleep 60
    kill -0 "$spel_pid" 2>/dev/null || break
    hb_min=$(( ($(date +%s) - started) / 60 ))
    hb_rss=$(ps -A -o rss=,comm= 2>/dev/null \
             | awk '/r0vm/ {s += $1} END {if (s > 0) printf "%.1f GB", s/1048576}')
    info "    … ${hb_min} min${hb_rss:+, r0vm ${hb_rss}}"
  done

  wait "$spel_pid" \
    || { tail -25 "$OUT/approve$i.log" >&2; die "approval $((i+1)) failed"; }
  grep -q 'confirmed' "$OUT/approve$i.log" || die "approval $((i+1)) was not confirmed"
  tx=$(awk '/tx_hash/{print $2; exit}' "$OUT/approve$i.log")
  [[ -n "$tx" ]] || die "approval $((i+1)) confirmed but no tx_hash in $OUT/approve$i.log"
  TX_APPROVE+=("$tx")
  info "tx $tx"
done

log "execute (threshold reached at FULL M)"
TX_EXECUTE=$(run_ix execute -- execute \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
  --recipient "$RECIPIENT" --submitter "$CREATOR")
info "tx $TX_EXECUTE"

# ─── evidence ───────────────────────────────────────────────────────────────────────────────────
log "writing docs/DEPLOYMENT.md"
{
  echo "# Deployment — LEZ public testnet"
  echo
  echo "Generated by \`scripts/deploy-testnet.sh\`. Re-verify on the day a PR is opened:"
  echo "testnets are wiped, and a dead evidence link fails plan gate W2."
  echo
  echo "| Field | Value |"
  echo "|-------|-------|"
  echo "| RPC | \`$RPC\` |"
  echo "| Explorer | \`$EXPLORER\` |"
  echo "| Deployed at | $(date -u +%FT%TZ) |"
  echo "| Commit | \`$(git rev-parse HEAD)\` |"
  echo "| Payer | \`$CREATOR\` |"
  echo "| config_hash | \`$CONFIG_HASH\` |"
  echo "| proposal_seed | \`$PROPOSAL_SEED\` |"
  echo "| Threshold | 2-of-3 (**full M** — no lowered tier) |"
  echo
  echo "## Programs"
  echo
  echo "| Program | ImageID (= ProgramId) | Deployment tx |"
  echo "|---------|----------------------|---------------|"
  for e in "${DEPLOY_TX[@]}"; do
    IFS='|' read -r p tx blk <<< "$e"
    id=$(grep -A8 "## \`$p\`" artifacts/IMAGE_IDS.md | awk -F'`' '/ImageID/{print $2; exit}')
    echo "| \`$p\` | \`$id\` | [\`${tx:0:16}…\`]($EXPLORER/transaction/$tx) (block $blk) |"
  done
  echo
  echo "## Lifecycle"
  echo
  echo "| # | Step | Transaction |"
  echo "|---|------|-------------|"
  echo "| 1 | create_multisig | [\`${TX_CREATE:0:16}…\`]($EXPLORER/transaction/$TX_CREATE) |"
  echo "| 2 | create_proposal | [\`${TX_PROPOSE:0:16}…\`]($EXPLORER/transaction/$TX_PROPOSE) |"
  i=3
  for tx in "${TX_APPROVE[@]}"; do
    echo "| $i | approve (anonymous, privacy-preserving) | [\`${tx:0:16}…\`]($EXPLORER/transaction/$tx) |"
    i=$((i+1))
  done
  echo "| $i | execute | [\`${TX_EXECUTE:0:16}…\`]($EXPLORER/transaction/$TX_EXECUTE) |"
  echo
  echo "## Reproducing"
  echo
  echo '```bash'
  echo './scripts/build-guests.sh --docker     # reproducible guest binaries'
  echo 'export LEE_WALLET_HOME_DIR=/path/to/funded/wallet'
  echo './scripts/deploy-testnet.sh'
  echo './scripts/verify-onchain.sh            # verifies from public data alone'
  echo '```'
  echo
  echo "## Superseded"
  echo
  echo "_None yet. If a testnet wipe invalidates the transactions above, the replacements are"
  echo "recorded here and the old ones struck through rather than deleted._"
} > docs/DEPLOYMENT.md

log "verifying from public chain data"
./scripts/verify-onchain.sh "$RPC" "$CONFIG_HASH" "$PROPOSAL_SEED" || die "on-chain verification failed"
./scripts/check-explorer-links.sh || die "an evidence link does not resolve"

log "DONE — docs/DEPLOYMENT.md written and verified"
echo
echo "  Next: ./scripts/measure-cu.sh  (criterion P-P1)"
