#!/usr/bin/env bash
# check-explorer-links.sh — every evidence URL in the submission must actually resolve (plan gate W2).
#
# A prior submission to this prize was marked down for a documentation link that 404'd at the pinned
# commit, and testnet wipes routinely kill transaction links. So this runs in CI and, per plan gate
# SC-G.12, must be re-run on the day the PR is opened.
#
# Fails on: a dead link, a 404, or an explorer page that resolves but reports no such transaction.
# A missing DEPLOYMENT.md is also a failure once Phase G has run — silence is not success.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || { echo "FATAL: curl is required." >&2; exit 1; }

echo "check-explorer-links.sh"
echo "  commit: $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
echo

if [[ ! -f docs/DEPLOYMENT.md ]]; then
  echo "docs/DEPLOYMENT.md does not exist yet (Phase G has not run)."
  echo "Nothing to check — but note this is NOT evidence of anything."
  exit 0
fi

# Collect every http(s) URL in the deployment record and the solution draft.
mapfile_compat() {                       # bash 3.2 has no mapfile
  local __arr="$1"; shift
  local __line; eval "$__arr=()"
  while IFS= read -r __line; do [[ -n "$__line" ]] && eval "$__arr+=(\"\$__line\")"; done < <("$@")
}
collect() {
  grep -ohE 'https?://[A-Za-z0-9./_%#?=&:+-]+' docs/DEPLOYMENT.md docs/SOLUTION_DRAFT.md 2>/dev/null \
    | sed 's/[.,)]*$//' | sort -u
}
mapfile_compat urls collect

if (( ${#urls[@]:-0} == 0 )); then
  echo "FATAL: docs/DEPLOYMENT.md exists but contains no URLs." >&2
  echo "       Phase G must publish explorer links as evidence." >&2
  exit 1
fi

fail=0
for url in "${urls[@]}"; do
  code=$(curl -s -o /tmp/explorer-body.$$ -w '%{http_code}' -L --max-time 25 "$url" || echo 000)
  body_says_missing=0
  if grep -qiE 'not found|no such transaction|does not exist|null' /tmp/explorer-body.$$ 2>/dev/null; then
    # Only treat this as fatal for transaction/account pages, where an empty result is the failure
    # mode that matters (a wiped testnet still serves a 200 page).
    [[ "$url" == *"/transaction/"* || "$url" == *"/account/"* ]] && body_says_missing=1
  fi
  rm -f /tmp/explorer-body.$$

  if [[ "$code" == "200" && $body_says_missing -eq 0 ]]; then
    printf '  OK   %s\n' "$url"
  elif [[ $body_says_missing -eq 1 ]]; then
    printf '  DEAD %s  (HTTP %s but the page reports no such record — testnet wiped?)\n' "$url" "$code"
    fail=1
  else
    printf '  FAIL %s  (HTTP %s)\n' "$url" "$code"
    fail=1
  fi
done

echo
if (( fail )); then
  echo "FAILED: at least one evidence URL does not resolve." >&2
  echo "Re-deploy and update docs/DEPLOYMENT.md before submitting (SC-G.12)." >&2
  exit 1
fi
echo "All ${#urls[@]} evidence URLs resolve."
