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
# Exit 0 = every binary is at least as new as its sources. Non-zero = stale, or the comparison
# cannot be trusted.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

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
