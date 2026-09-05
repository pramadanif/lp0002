#!/usr/bin/env bash
# fund-testnet.sh — obtain LEZ public-testnet funds via the Piñata faucet.
#
# The plan lists "funded testnet keys" as a human gate. It is not one: LEZ ships a proof-of-work
# faucet (the Piñata program) and the wallet carries the command. This script does the whole thing.
#
# Usage:  ./scripts/fund-testnet.sh [wallet-home]
#         default wallet-home: .e2e/wallet-testnet
#
# Missing tools fail (gate H2). The wallet is created if absent — its keys stay under the wallet
# home, which is gitignored.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }
REPO="$PWD"

HOME_DIR="${1:-$REPO/.e2e/wallet-testnet}"
RPC="${PMSIG_RPC:-https://testnet.lez.logos.co}"
LEZ_DIR="${PMSIG_LEZ_DIR:-$REPO/.e2e/lez}"
WALLET="$LEZ_DIR/target/release/wallet"

die() { echo "FATAL: $*" >&2; exit 1; }
info() { printf '    %s\n' "$*"; }
log() { printf '\n==> %s\n' "$*"; }

command -v jq >/dev/null 2>&1 || die "jq is required."
command -v curl >/dev/null 2>&1 || die "curl is required."
[[ -x "$WALLET" ]] || die "wallet binary not found at $WALLET.
       Build it:  ( cd $LEZ_DIR && cargo build --release -p wallet )"

case "$RPC" in
  *127.0.0.1*|*localhost*) die "PMSIG_RPC points at a local node. The faucet is a public-testnet thing." ;;
esac

mkdir -p "$HOME_DIR"
if [[ ! -f "$HOME_DIR/wallet_config.json" ]]; then
  log "writing a wallet config pointed at $RPC"
  cat > "$HOME_DIR/wallet_config.json" <<EOF
{
    "sequencers": [{ "sequencer_addr": "$RPC" }],
    "seq_poll_timeout": "60s",
    "seq_tx_poll_max_blocks": 30,
    "seq_poll_max_retries": 20,
    "seq_block_poll_max_amount": 100,
    "calibration_limit": 100
}
EOF
fi
export LEE_WALLET_HOME_DIR="$HOME_DIR"

# The wallet prompts for a password on stdin. Without one it hangs forever, which looks like a
# network problem and is not.
#
# `|| true` guards against SIGPIPE: an `awk '{...; exit}'` downstream closes the pipe early, the
# wallet dies with 141, and `pipefail` would kill the script for a command that actually worked.
# Run the wallet and capture everything. pipefail is disabled around the call because a downstream
# `grep -q` or `awk … exit` closes the pipe early, the wallet dies with SIGPIPE, and the script would
# abort over a command that actually succeeded.
w() {
  set +o pipefail
  echo "" | "$WALLET" "$@" 2>&1
  local rc=$?
  set -o pipefail
  return $rc
}

log "checking the wallet can reach $RPC"
w check-health > "$HOME_DIR/.health.log" || true
grep -q 'All looks good' "$HOME_DIR/.health.log" || { tail -5 "$HOME_DIR/.health.log" >&2; die "the wallet could not reach $RPC"; }
info "connected"

w account list > "$HOME_DIR/.accounts.log" || true
ACC=$(awk '/Public\//{print $2}' "$HOME_DIR/.accounts.log" | head -1)
[[ -n "$ACC" ]] || die "the wallet has no public account"
ID=${ACC#Public/}          # `account list` already includes the prefix; re-adding it fails
info "payer $ID"

balance() {
  curl -s -X POST "$RPC" -H 'content-type: application/json' \
    --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccount\",\"params\":[\"$ID\"]}" \
    --max-time 20 | jq -r '.result.balance // 0'
}

BEFORE=$(balance)
info "balance before: $BEFORE"

# The faucet refuses an uninitialised recipient, and the error only appears after the first attempt.
# The faucet refuses an uninitialised recipient, and only says so after the first attempt fails.
# But running `auth-transfer init` on an ALREADY-initialised account hangs rather than erroring, so
# check the nonce first: a nonce above zero means the account has been used, hence initialised.
NONCE=$(curl -s -X POST "$RPC" -H 'content-type: application/json' \
  --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccount\",\"params\":[\"$ID\"]}" \
  --max-time 20 | jq -r '.result.nonce // 0')
if [[ "$NONCE" == "0" ]]; then
  log "initialising the account (the faucet will not pay an uninitialised one)"
  w auth-transfer init --account-id "$ACC" > "$HOME_DIR/.init.log" || true
  grep -q 'included in block' "$HOME_DIR/.init.log" \
    || { tail -5 "$HOME_DIR/.init.log" >&2; die "account initialisation failed"; }
  info "initialised"
else
  info "account already initialised (nonce $NONCE) — skipping init"
fi

log "claiming from the Piñata faucet (proof-of-work — takes a few seconds)"
w pinata claim --to "$ACC" > "$HOME_DIR/.claim.log" || true
out=$(cat "$HOME_DIR/.claim.log")
echo "$out" | grep -q 'included in block' || { echo "$out" | tail -5 >&2; die "the claim was not included in a block"; }
info "$(echo "$out" | grep -oE 'Found solution [0-9]+ in [0-9.]+s' | head -1)"
info "tx    $(echo "$out" | awk '/Transaction hash is/{print $NF}')"
info "block $(echo "$out" | awk '/included in block/{print $NF}')"

AFTER=$(balance)
log "done"
info "balance: $BEFORE -> $AFTER"
[[ "$AFTER" != "0" ]] || die "balance is still zero after a claim that reported success"
echo
echo "  Wallet home: $HOME_DIR"
echo "  Run again to top up. Then:  LEE_WALLET_HOME_DIR=$HOME_DIR ./scripts/deploy-testnet.sh"
