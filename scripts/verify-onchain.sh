#!/usr/bin/env bash
# verify-onchain.sh — verify a completed multisig from PUBLIC chain data alone (plan gate W1).
#
# Takes no secrets, no local state and no witness. Anyone with the RPC URL can run it and satisfy
# themselves that a threshold was genuinely met and that the chain records no member identity.
#
# What it checks:
#   1. the config account is owned by the deployed multisig program
#   2. the config rehashes to its own address                       (ADR-001 INV-3)
#   3. the config names the deployed membership program              (ADR-002)
#   4. the proposal belongs to that multisig
#   5. the threshold was met at FULL M, and the proposal executed    (H13/W15)
#   6. every nullifier is distinct                                   (no double vote)
#   7. no member root or identity appears in the proposal account    (P-F2)
#
# Usage:
#   ./scripts/verify-onchain.sh                      # reads docs/DEPLOYMENT.md
#   ./scripts/verify-onchain.sh <rpc> <config_hash> <proposal_seed>
#
# Missing tools fail this script (gate H2).

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || { echo "FATAL: '$1' is required. $2" >&2; exit 1; }; }
require cargo "Install Rust: https://rustup.rs"
require curl  "Install curl."

RPC="${1:-}"; CONFIG_HASH="${2:-}"; PROPOSAL_SEED="${3:-}"

# With no arguments, take the deployment record as the source of truth. That keeps the published
# evidence and this check from drifting apart: if DEPLOYMENT.md is wrong, this fails.
if [[ -z "$RPC" ]]; then
  [[ -f docs/DEPLOYMENT.md ]] || {
    echo "FATAL: docs/DEPLOYMENT.md not found, and no arguments given." >&2
    echo "       usage: $0 <rpc-url> <config_hash> <proposal_seed>" >&2
    exit 1
  }
  RPC=$(awk -F'`' '/^\| *RPC/ {print $2; exit}' docs/DEPLOYMENT.md)
  CONFIG_HASH=$(awk -F'`' '/^\| *config_hash/ {print $2; exit}' docs/DEPLOYMENT.md)
  PROPOSAL_SEED=$(awk -F'`' '/^\| *proposal_seed/ {print $2; exit}' docs/DEPLOYMENT.md)
fi

for v in RPC CONFIG_HASH PROPOSAL_SEED; do
  [[ -n "${!v}" ]] || { echo "FATAL: $v is empty — check docs/DEPLOYMENT.md or pass it as an argument." >&2; exit 1; }
done

[[ -s artifacts/IMAGE_IDS.md ]] || { echo "FATAL: artifacts/IMAGE_IDS.md missing. Run ./scripts/build-guests.sh" >&2; exit 1; }

echo "verify-onchain.sh"
echo "  rpc           : $RPC"
echo "  config_hash   : $CONFIG_HASH"
echo "  proposal_seed : $PROPOSAL_SEED"
echo "  commit        : $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
echo

# Reachability is a hard failure: a verification that silently passes against a dead node is worse
# than no verification.
curl -s -X POST "$RPC" -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"checkHealth","params":[]}' --max-time 15 \
  | grep -q '"result"' || { echo "FATAL: $RPC did not answer checkHealth." >&2; exit 1; }
echo "  node reachable: yes"
echo

cargo run --quiet -p pmsig-sdk --example verify_onchain -- \
  "$RPC" artifacts/IMAGE_IDS.md "$CONFIG_HASH" "$PROPOSAL_SEED"
