#!/usr/bin/env bash
# check-dev-mode-clobber.sh — plan gate H3.
#
# Historical reject pattern (#97): the top-level demo ran with RISC0_DEV_MODE=0, but a nested
# script it called hardcoded RISC0_DEV_MODE=1, so the "real proof" demo silently proved nothing.
#
# This check fails if any script on the submission path sets RISC0_DEV_MODE=1. Child scripts must
# inherit the value from the entrypoint rather than setting their own.
#
# demo-fast.sh is exempt: it is explicitly not a submission path and is never cited as the prize
# demo (plan §0.4.4). CI runs this on every push.

set -uo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

echo "check-dev-mode-clobber.sh — scanning submission-path scripts for RISC0_DEV_MODE=1"

# Files on the submission path: the canonical demo and everything under scripts/, minus the two
# files whose own text necessarily mentions the literal (this checker and the preflight harness).
# Written for bash 3.2 (macOS default) as well as bash 5 — no mapfile, no associative arrays.
targets=()
while IFS= read -r f; do
  [[ -n "$f" ]] && targets+=("$f")
done < <(
  { [[ -f demo.sh ]] && echo demo.sh
    find scripts -type f -name '*.sh' 2>/dev/null
  } | grep -v 'check-dev-mode-clobber\.sh' \
    | grep -v 'preflight-submission\.sh' \
    | sort -u
)

if (( ${#targets[@]:-0} == 0 )); then
  echo "no submission-path scripts to scan yet"
  echo "OK"
  exit 0
fi

printf 'scanning %d file(s):\n' "${#targets[@]}"
printf '  %s\n' "${targets[@]}"

hits=$(grep -nE 'RISC0_DEV_MODE[[:space:]]*=[[:space:]]*1' "${targets[@]}" 2>/dev/null || true)

if [[ -n "$hits" ]]; then
  echo
  echo "FAIL: RISC0_DEV_MODE=1 is set on the submission path:"
  echo "$hits"
  echo
  echo "Child scripts must inherit RISC0_DEV_MODE from the entrypoint (H3)."
  exit 1
fi

echo "OK: no script on the submission path sets RISC0_DEV_MODE=1"
exit 0
