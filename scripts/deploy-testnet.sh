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
BAL=$(curl -s -X POST "$RPC" -H 'content-type: application/json' \
  --data "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"getAccountBalance\",\"params\":[\"$CREATOR\"]}" --max-time 20 | jq -r '.result // 0')
info "payer $CREATOR, balance $BAL"
[[ "$BAL" != "0" && "$BAL" != "null" ]] || die "payer account has zero balance — fund it before deploying (human gate)."

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

log "create_proposal (treasury transfer)"
# One variable for both steps: `execute` refuses a recipient the proposal did not name (INV-7).
RECIPIENT=c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3
TX_PROPOSE=$(run_ix propose -- create-proposal \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" --proposal-id "$PROPOSAL_ID" \
  --recipient "$RECIPIENT" \
  --amount 1000 --proposer "$CREATOR")
info "tx $TX_PROPOSE"

# H13/W15: the published evidence uses the FULL threshold, never a lowered tier.
declare -a TX_APPROVE
for i in 0 1; do
  a="MEMBER${i}_ACCOUNT"; n="MEMBER${i}_NULLIFIER"; w="MEMBER${i}_WITNESS"
  log "approve $((i+1)) of 2 — anonymous, privacy-preserving, RISC0_DEV_MODE=$RISC0_DEV_MODE"
  info "expect ~20 minutes and ~9 GB of free RAM; it is not hung"
  tx=$("$SPEL" --idl "$REPO/artifacts/multisig-idl.json" -p "$REPO/artifacts/multisig.bin" \
        --bin-membership "$REPO/artifacts/membership.bin" -- \
        approve --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
        --member-root "$MEMBER_ROOT" --claimed-nullifier "${!n}" \
        --witness "$(cat "${!w}")" --approver "Private/${!a}" \
        2>&1 | tee "$OUT/approve$i.log" | awk '/tx_hash/{print $2; exit}') \
    || { tail -25 "$OUT/approve$i.log" >&2; die "approval $((i+1)) failed"; }
  grep -q 'confirmed' "$OUT/approve$i.log" || die "approval $((i+1)) was not confirmed"
  TX_APPROVE+=("$tx")
  info "tx $tx"
done

log "execute (threshold reached at FULL M)"
TX_EXECUTE=$(run_ix execute -- execute \
  --config-hash "$CONFIG_HASH" --proposal-seed "$PROPOSAL_SEED" \
  --recipient "$RECIPIENT")
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
