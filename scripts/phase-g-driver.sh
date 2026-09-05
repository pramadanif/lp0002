#!/usr/bin/env bash
# phase-g-driver.sh — run the rest of Phase G unattended, in order, without supervision.
#
# WHY THIS EXISTS
#
# Each Phase G step gates the next, and driving them by hand means something has to notice when a
# step ends. Periodic wake-ups were tried for that and did not fire, so progress sat invisible while
# the work itself was fine. This removes the need to notice: the chain runs itself and reports once,
# at the end, whatever the outcome.
#
# Every step writes its own log and its real exit code. Nothing here reports success it did not get.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export LEE_WALLET_HOME_DIR="${LEE_WALLET_HOME_DIR:-$PWD/.e2e/wallet-testnet}"
SUM=logs/phase-g-summary.txt
: > "$SUM"

say() { printf '%s\n' "$*" | tee -a "$SUM"; }
step() { # name, logfile, command...
  local name="$1" log="$2"; shift 2
  say "── $name"
  "$@" > "$log" 2>&1
  local rc=$?
  say "   exit=$rc  log=$log"
  return $rc
}

say "phase-g-driver start $(date -u +%FT%TZ)"

# 1. Wait for the deployment already in flight. Polling a log rather than a PID: the script may be a
#    child of a shell that has gone away, and EXIT= is written by the launcher as its last act.
say "── waiting for deploy-testnet to finish"
waited=0
while ! grep -q 'EXIT=' logs/deploy-testnet.log 2>/dev/null; do
  sleep 30
  waited=$((waited + 30))
  if [ "$waited" -ge 7200 ]; then say "   TIMEOUT after 2h waiting for deploy"; exit 3; fi
done
DEPLOY_RC=$(sed -n 's/^EXIT=//p' logs/deploy-testnet.log | tail -1)
say "   deploy exit=$DEPLOY_RC after ${waited}s of waiting"

if [ "$DEPLOY_RC" != "0" ]; then
  say ""
  say "deploy failed; not running the steps that depend on it. Last lines:"
  grep -E '==>|FATAL|error' logs/deploy-testnet.log | tail -15 | tee -a "$SUM"
  exit 4
fi

# 2. Everything below reads the chain, so it only makes sense once something is on it.
rc_total=0
step "measure on-chain CU"     logs/measure-cu.log       ./scripts/measure-cu.sh        || rc_total=1
step "verify from public RPC"  logs/verify-onchain.log   ./scripts/verify-onchain.sh    || rc_total=1
step "explorer links resolve"  logs/explorer-links.log   ./scripts/check-explorer-links.sh || rc_total=1
step "preflight"               logs/preflight.log        ./scripts/preflight-submission.sh || true

say ""
say "── artefacts"
for f in docs/DEPLOYMENT.md docs/cu-costs.md artifacts/IMAGE_IDS.md; do
  if [ -s "$f" ]; then say "   $f  ($(wc -l < "$f" | tr -d ' ') lines)"; else say "   $f  MISSING"; fi
done
say ""
say "phase-g-driver done $(date -u +%FT%TZ), rc=$rc_total"
exit "$rc_total"
