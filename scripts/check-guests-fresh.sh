#!/usr/bin/env bash
# check-guests-fresh.sh — fail if a committed guest binary is older than the source it is built from.
#
# WHY THIS EXISTS
#
# artifacts/*.bin are committed build products. Everything downstream trusts them: the executor
# tests in crates/sdk/tests/ run them, deploy-testnet.sh deploys them, and IMAGE_IDS.md fingerprints
# them. Nothing rebuilds them automatically, so a source change that is never followed by
# ./scripts/build-guests.sh leaves the repo in a state where the tests pass, the deployment
# succeeds, and neither is testing or running the code in the repo.
#
# That is not hypothetical: artifacts/multisig.bin went stale behind programs/multisig-spel, and the
# discrepancy only surfaced when a new test happened to exercise an instruction whose account list
# had changed. This gate makes it surface immediately instead.
#
# HOW
#
# Comparing hashes is not an option: a local build is not reproducible across hosts (see
# build-guests.sh), so a rebuilt binary legitimately differs byte-for-byte from a committed one
# built elsewhere. What *is* comparable everywhere is commit order — if the newest commit touching a
# guest's sources is newer than the newest commit touching its binary, the binary cannot contain
# those sources.
#
# WHY IMAGE_IDS.md COUNTS AS PART OF THE ARTEFACT
#
# A source change that touches only test code — an inline `#[cfg(test)]` module, say — does not
# change the guest, so a rebuild is byte-identical, so git has nothing to commit, so the binary's
# last commit never moves and this gate stays red forever. That is a deadlock, and it happened.
#
# Rebuilding always rewrites artifacts/IMAGE_IDS.md, whose `Built` timestamp and `Commit` field move
# even when the ImageID does not. So the freshness of a guest is judged on the newer of its binary
# and that record: the pair says "someone rebuilt after the last source change", which is the
# property actually wanted. Editing the record by hand to silence this does not help — the ImageID
# it names must equal the committed binary's, which
# crates/sdk/tests/image_ids_match_binaries.rs checks.
#
# Exit 0 = every binary is at least as new as its sources. Non-zero = stale, or the comparison
# cannot be trusted.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

# Guard against being invoked through a copy outside the tree: `dirname $0/..` would then land
# somewhere unrelated, every binary would look "missing", and the gate would fail for the wrong
# reason. (This happened while testing this very script from /tmp.)
[[ -f scripts/build-guests.sh && -d .git ]] || {
  echo "FATAL: not at the repo root (got $PWD). Run this as ./scripts/check-guests-fresh.sh" >&2
  exit 2
}

# "<binary>|<space-separated source paths>"
GUESTS=(
  "artifacts/multisig.bin|programs/multisig-spel crates/core crates/membership-core crates/multisig-core"
  "artifacts/membership.bin|programs/membership-lez crates/core crates/membership-core"
)

fail=0

for entry in "${GUESTS[@]}"; do
  bin="${entry%%|*}"
  srcs="${entry#*|}"

  if [[ ! -s "$bin" ]]; then
    echo "FAIL: $bin is missing. Run ./scripts/build-guests.sh" >&2
    fail=1
    continue
  fi

  # An uncommitted edit to either side makes commit timestamps meaningless.
  # shellcheck disable=SC2086
  if ! git diff --quiet -- "$bin" $srcs || ! git diff --cached --quiet -- "$bin" $srcs; then
    echo "FAIL: uncommitted changes under $bin or its sources ($srcs)." >&2
    echo "      Commit them, or run ./scripts/build-guests.sh, before this gate can judge freshness." >&2
    fail=1
    continue
  fi

  bin_ts=$(git log -1 --format=%ct -- "$bin" 2>/dev/null || true)
  ids_ts=$(git log -1 --format=%ct -- artifacts/IMAGE_IDS.md 2>/dev/null || true)
  if [[ -n "$ids_ts" && ( -z "$bin_ts" || "$ids_ts" -gt "$bin_ts" ) ]]; then
    bin_ts="$ids_ts"
  fi
  # shellcheck disable=SC2086
  src_ts=$(git log -1 --format=%ct -- $srcs 2>/dev/null || true)

  if [[ -z "$bin_ts" ]]; then
    echo "FAIL: $bin is not committed, so nothing pins what it was built from." >&2
    fail=1
    continue
  fi
  if [[ -z "$src_ts" ]]; then
    echo "FAIL: no commits found for the sources of $bin ($srcs)." >&2
    fail=1
    continue
  fi

  if (( src_ts > bin_ts )); then
    echo "FAIL: $bin is STALE." >&2
    echo "      binary last committed : $(git log -1 --format='%h %ad %s' --date=short -- "$bin")" >&2
    echo "      IMAGE_IDS.md          : $(git log -1 --format='%h %ad %s' --date=short -- artifacts/IMAGE_IDS.md)" >&2
    # shellcheck disable=SC2086
    echo "      sources last committed: $(git log -1 --format='%h %ad %s' --date=short -- $srcs)" >&2
    echo "      The tests and any deployment would run a program that is not the one in this repo." >&2
    echo "      Fix: ./scripts/build-guests.sh (or --docker for anything deployed), then commit." >&2
    fail=1
  else
    echo "ok: $bin is at least as new as its sources"
  fi
done

exit "$fail"
