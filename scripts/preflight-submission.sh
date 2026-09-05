#!/usr/bin/env bash
# preflight-submission.sh — plan gate H15, spec in planlp0002.md §6 (PF-01 … PF-15).
#
# Exits 0 only when every check passes. Any FAIL or PENDING check exits 1.
#
# A PENDING check is one whose evidence does not exist yet because the phase that produces it
# has not run. PENDING is NOT a pass: it exits 1 exactly like a FAIL, so this script can never
# report a submission as ready while work is outstanding.
#
# Phase 0 status: the harness is complete and every check is wired, but the artefacts most checks
# look for are produced in Phases B–H, so this script exits 1 today. That is intended.

set -uo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }
ROOT="$PWD"

pass=0; fail=0; pending=0

ok()      { printf 'PASS     %-6s %s\n' "$1" "$2"; pass=$((pass+1)); }
bad()     { printf 'FAIL     %-6s %s\n' "$1" "$2"; fail=$((fail+1)); }
pend()    { printf 'PENDING  %-6s %s\n' "$1" "$2"; pending=$((pending+1)); }

# A file that a later phase produces: PENDING when absent, so it never reads as a pass.
need_file() { # id, path, description
  if [[ -s "$2" ]]; then ok "$1" "$3"; else pend "$1" "$3 (missing/empty: $2)"; fi
}

echo "preflight-submission.sh — LP-0002"
echo "repo: $ROOT"
echo "commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'no git')"
echo "date: $(date -u +%FT%TZ)"
echo "------------------------------------------------------------------"

# PF-01 — dual licence
if [[ -s LICENSE-MIT && -s LICENSE-APACHE ]]; then
  ok "PF-01" "LICENSE-MIT and LICENSE-APACHE both present"
else
  bad "PF-01" "dual licence missing (need LICENSE-MIT and LICENSE-APACHE)"
fi

# PF-02 — demo.sh must reach a standalone sequencer, not just an in-process executor
if [[ ! -f demo.sh ]]; then
  pend "PF-02" "demo.sh not written yet (Phase E)"
elif grep -qE 'e2e-local-sequencer\.sh|sequencer_service|--features[[:space:]]+standalone' demo.sh; then
  ok "PF-02" "demo.sh references a standalone sequencer path"
else
  bad "PF-02" "demo.sh does not reference a standalone sequencer or e2e-local-sequencer.sh"
fi

# PF-03 — no missing-tool skip that still exits 0, on demo/e2e paths
pf03_targets=(demo.sh scripts/e2e-local-sequencer.sh)
pf03_present=0; pf03_bad=""
for f in "${pf03_targets[@]}"; do
  [[ -f "$f" ]] || continue
  pf03_present=1
  # A missing-tool branch must not exit 0. Flag `exit 0` sitting near a tool-presence test.
  if grep -nE '(command -v|which |\-x )' "$f" | grep -q .; then
    if awk '/command -v|which |not found|missing/{ctx=NR} /exit 0/{if (NR-ctx<=4 && ctx>0) print NR}' "$f" | grep -q .; then
      pf03_bad="$pf03_bad $f"
    fi
  fi
done
if [[ $pf03_present -eq 0 ]]; then
  pend "PF-03" "demo/e2e scripts not written yet (Phase E)"
elif [[ -n "$pf03_bad" ]]; then
  bad "PF-03" "missing-tool branch appears to exit 0 in:$pf03_bad"
else
  ok "PF-03" "no missing-tool skip-to-exit-0 on demo/e2e paths"
fi

# PF-04 — RISC0_DEV_MODE=1 must not appear on the demo/e2e submission path
# Comments and echoed help text mention the literal, so strip comments before matching and skip the
# two scripts whose whole job is to talk about it. Matching a comment would make this gate cry wolf,
# which is how a real violation later gets waved through.
pf04_files=$( { [[ -f demo.sh ]] && echo demo.sh
                find scripts -type f -name '*.sh' 2>/dev/null; } \
              | grep -v 'demo-fast\.sh' \
              | grep -v 'check-dev-mode-clobber\.sh' \
              | grep -v 'preflight-submission\.sh' | sort -u )
pf04_hits=""
for f in $pf04_files; do
  # Drop everything from the first '#' on each line, then look for a real assignment.
  hit=$(sed 's/#.*//' "$f" | grep -nE 'RISC0_DEV_MODE[[:space:]]*=[[:space:]]*1' || true)
  [[ -n "$hit" ]] && pf04_hits="$pf04_hits$f:$hit"$'\n'
done

if [[ -z "$pf04_hits" ]]; then
  ok "PF-04" "no RISC0_DEV_MODE=1 on the demo/e2e submission path"
else
  bad "PF-04" "RISC0_DEV_MODE=1 is set on the submission path:"$'\n'"$pf04_hits"
fi

# PF-05 — CI must run an e2e job on push to main, not cron-only
CI=.github/workflows/ci.yml
if [[ ! -f $CI ]]; then
  pend "PF-05" "ci.yml not present"
elif ! grep -qE '^[[:space:]]+[a-z0-9_-]*e2e[a-z0-9_-]*:' "$CI"; then
  pend "PF-05" "ci.yml has no job whose name contains 'e2e' (Phase E)"
elif python3 - "$CI" <<'PY'
import sys, re
src = open(sys.argv[1]).read()
# crude but sufficient: does the `on:` block include push with main?
m = re.search(r'^on:\s*$(.*?)^\S', src, re.M | re.S)
block = m.group(1) if m else ''
push = re.search(r'^\s+push:\s*$(.*?)^\s{2}\S', block + '\n  x', re.M | re.S)
sys.exit(0 if (push and 'main' in push.group(1)) else 1)
PY
then
  ok "PF-05" "ci.yml runs on push to main and defines an e2e job"
else
  bad "PF-05" "ci.yml e2e job is not gated on push to main (cron-only or path-filtered away)"
fi

# PF-06 — limitations.md exists and is non-empty (H12/W16 — the gap #125 left at its pin)
need_file "PF-06" "docs/limitations.md" "docs/limitations.md present and non-empty"

# PF-07 — criteria checklist mentions every prize criterion id
CHK=docs/criteria-checklist.md
if [[ ! -s $CHK ]]; then
  pend "PF-07" "docs/criteria-checklist.md not written yet (Phase H)"
else
  missing=""
  # Each id must appear in a TABLE ROW, not merely somewhere in the file: a criterion mentioned only
  # in prose is not a covered criterion, and a bare file-wide grep would count it as one. The
  # trailing character class keeps `P-F1` from being satisfied by a hypothetical `P-F10`.
  for id in P-F1 P-F2 P-F3 P-F4 P-F5 P-F6 P-F7 P-F8 P-U1 P-U2 P-U3 P-R1 P-R2 P-R3 P-P1 P-S1 P-S2 P-S3 P-S4 P-S5 P-S6; do
    grep -qE "^\|.*${id}([^0-9A-Za-z]|\$)" "$CHK" || missing="$missing $id"
  done
  if [[ -z "$missing" ]]; then ok "PF-07" "criteria-checklist covers all 21 prize criteria"
  else bad "PF-07" "criteria-checklist missing:$missing"; fi
fi

# PF-08 — on-chain CU costs must be numeric, never "unavailable"
#
# Looks for the per-instruction CU table specifically, not just any digits in the file: the client
# proving benchmarks land in the same document in Phase B and would otherwise satisfy a naive check
# while the actual criterion (P-P1, on-chain CU) was still unmeasured.
CU=docs/cu-costs.md
cu_row_re='^\|[[:space:]]*`?(create_multisig|create_proposal|approve|execute)`?[[:space:]]*\|'
if [[ ! -s $CU ]]; then
  pend "PF-08" "docs/cu-costs.md not written yet (Phase G)"
elif ! grep -qE "$cu_row_re" "$CU"; then
  pend "PF-08" "cu-costs.md has no per-instruction on-chain CU table yet (Phase G)"
elif grep -E "$cu_row_re" "$CU" | grep -qiE 'unavailable|n/?a|tbd'; then
  bad "PF-08" "the on-chain CU table still says unavailable/TBD"
elif ! grep -E "$cu_row_re" "$CU" | grep -qE '[0-9]'; then
  bad "PF-08" "the on-chain CU table has no numeric values"
else
  ok "PF-08" "cu-costs.md has numeric on-chain CU per instruction"
fi

# PF-09 — deployment evidence + live explorer links
if [[ ! -s docs/DEPLOYMENT.md ]]; then
  pend "PF-09" "docs/DEPLOYMENT.md not written yet (Phase G)"
elif ! grep -qE 'https?://explorer\.' docs/DEPLOYMENT.md; then
  bad "PF-09" "DEPLOYMENT.md has no explorer URL"
elif [[ -x scripts/check-explorer-links.sh ]]; then
  if scripts/check-explorer-links.sh >/dev/null 2>&1; then
    ok "PF-09" "DEPLOYMENT.md explorer links present and check-explorer-links.sh exits 0"
  else
    bad "PF-09" "check-explorer-links.sh exited non-zero (dead evidence link)"
  fi
else
  pend "PF-09" "scripts/check-explorer-links.sh not executable yet (Phase G)"
fi

# PF-10 — public on-chain verification must actually run
#
# Distinguishes "Phase G has not run" from "verification failed". A script that cannot verify
# because nothing is deployed yet is PENDING; one that runs and disagrees with the chain is a FAIL.
# Conflating the two would either block early phases or, worse, let a real verification failure hide
# behind "not deployed yet".
if [[ ! -x scripts/verify-onchain.sh ]]; then
  pend "PF-10" "scripts/verify-onchain.sh not executable yet (Phase G)"
elif [[ ! -s docs/DEPLOYMENT.md ]]; then
  pend "PF-10" "verify-onchain.sh exists but nothing is deployed yet (Phase G)"
elif scripts/verify-onchain.sh >/dev/null 2>&1; then
  ok "PF-10" "verify-onchain.sh exits 0 against the published deployment"
else
  bad "PF-10" "verify-onchain.sh exited non-zero against the published deployment"
fi

# PF-11 — pinned guest ImageIDs, and the binaries they fingerprint are not stale
need_file "PF-11" "artifacts/IMAGE_IDS.md" "artifacts/IMAGE_IDS.md present and non-empty"

# An ImageID pins whichever binary was committed; it says nothing about whether that binary was
# built from the source now in the repo. A stale binary means the deployed program, the ImageIDs
# quoted in the submission and the executor tests all describe code the reviewer cannot see.
if ./scripts/check-guests-fresh.sh >/dev/null 2>&1; then
  ok "PF-11" "committed guest binaries are at least as new as their sources"
else
  bad "PF-11" "a committed guest binary is stale — run ./scripts/check-guests-fresh.sh"
fi

# PF-06b — every relative Markdown link resolves at the pin. #125 was pulled up for a
# `limitations.md` link that 404'd at its own pinned commit; a reviewer reads this repo at a commit,
# not on a live branch.
if ./scripts/check-links.sh >/dev/null 2>&1; then
  ok "PF-06" "every relative Markdown link resolves"
else
  bad "PF-06" "a relative Markdown link is broken — run ./scripts/check-links.sh"
fi

# PF-12 — narrated video link + transcript
vid_src=""
[[ -f docs/SOLUTION_DRAFT.md ]] && vid_src=docs/SOLUTION_DRAFT.md
# The URL and the word "video" must be on the SAME line, and that line must not be one denying a
# video exists. Three independent file-wide greps used to stand here, and they passed on a document
# whose only occurrence of "video" was the sentence "**No narrated video.** P-S6 unmet." — the gate
# guarding the video requirement was satisfied by the statement that the video did not exist.
vid_line=""
if [[ -n "$vid_src" ]]; then
  vid_line=$(grep -iE 'video' "$vid_src" | grep -E 'https?://' \
             | grep -viE '\bno\b|not |unmet|missing|todo|placeholder|tbd|pending' | head -1)
fi
if [[ -n "$vid_line" ]] && [[ -s docs/video-transcript.md ]]; then
  ok "PF-12" "narrated video URL present in SOLUTION_DRAFT and docs/video-transcript.md exists"
else
  pend "PF-12" "narrated video URL + docs/video-transcript.md not in place yet (Phase H, human gate)"
fi

# PF-13 — one config_hash formula everywhere (H14/W17 — the drift #125 was pulled up on)
formula_files=(README.md docs/adr/ADR-001-architecture.md docs/SOLUTION_DRAFT.md)
present=(); for f in "${formula_files[@]}"; do [[ -f $f ]] && present+=("$f"); done
if [[ ${#present[@]} -lt 3 ]]; then
  pend "PF-13" "config_hash formula sources not all present yet (need README + ADR-001 + SOLUTION_DRAFT)"
else
  # Normalise whitespace, then compare the single line declaring config_hash =
  norm() { grep -ho 'config_hash[[:space:]]*=[^|]*' "$1" | head -1 | tr -d ' `' ; }
  a=$(norm "${present[0]}"); b=$(norm "${present[1]}"); c=$(norm "${present[2]}")
  if [[ -n "$a" && "$a" == "$b" && "$b" == "$c" ]]; then
    ok "PF-13" "config_hash formula identical across README, ADR-001 and SOLUTION_DRAFT"
  else
    bad "PF-13" "config_hash formula differs: README='$a' ADR='$b' SOLUTION='$c'"
  fi
fi

# PF-14 — the prize demo cited must be demo.sh, never demo-fast.sh
if [[ ! -s docs/SOLUTION_DRAFT.md ]]; then
  pend "PF-14" "docs/SOLUTION_DRAFT.md not written yet (Phase H)"
elif grep -qE 'demo-fast\.sh' docs/SOLUTION_DRAFT.md && \
     grep -qE 'demo-fast\.sh[^\n]*(prize|canonical|evaluat)' docs/SOLUTION_DRAFT.md; then
  bad "PF-14" "SOLUTION_DRAFT cites demo-fast.sh as the prize demo"
elif grep -qE '\./demo\.sh' docs/SOLUTION_DRAFT.md; then
  ok "PF-14" "SOLUTION_DRAFT cites ./demo.sh as the prize demo"
else
  bad "PF-14" "SOLUTION_DRAFT does not cite ./demo.sh as the prize demo"
fi

# PF-15 — print the pin and remind about day-of re-verification
PIN=$(git rev-parse HEAD 2>/dev/null || echo 'unknown')
ok "PF-15" "pin commit $PIN — RE-RUN verify-onchain.sh AND check-explorer-links.sh on the day the PR is opened (testnet wipes)"

echo "------------------------------------------------------------------"
printf 'pass=%d fail=%d pending=%d\n' "$pass" "$fail" "$pending"

if (( fail > 0 || pending > 0 )); then
  echo "RESULT: NOT READY — a submission must have every check PASS."
  exit 1
fi
echo "RESULT: READY"
exit 0
