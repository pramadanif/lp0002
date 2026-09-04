#!/usr/bin/env bash
# demo.sh — THE prize demo (criterion P-S5, plan gate H1).
#
# Runs the full private multisig lifecycle against a REAL standalone LEZ sequencer with
# RISC0_DEV_MODE=0. An evaluator should be able to clone this repository and run:
#
#     ./demo.sh
#
# ─── What this script will not do ────────────────────────────────────────────────────────────────
#
# It will not pass on a machine that cannot actually run it. If a prerequisite is missing, or the
# sequencer will not start, or a proof cannot be generated, this script exits NON-ZERO. There is no
# skip, no `continue-on-error`, and no fallback to dev mode anywhere on this path (gates H1/H2/H3).
#
# The reason is specific: a demo that reports success without proving anything is the failure mode
# that has sunk several prior submissions to this prize. See docs/limitations.md.
#
# `demo-fast.sh` exists for a quick executor-only tour. It is NOT this script, it is not the prize
# demo, and nothing in the submission cites it as evidence.

set -euo pipefail
cd "$(dirname "$0")" || { echo "cannot cd to repo root" >&2; exit 1; }

# RISC0_DEV_MODE=0 is set HERE, at the entrypoint, and inherited by every child. No nested script
# sets it to 1 — scripts/check-dev-mode-clobber.sh fails the build if one ever does (gate H3).
export RISC0_DEV_MODE=0

cat <<'BANNER'
┌──────────────────────────────────────────────────────────────────────────────┐
│  Private M-of-N Multisig for LEZ — end-to-end demo                            │
│                                                                              │
│  Shielded members approve a treasury transfer. The chain records that a       │
│  threshold was met, and nothing about which members met it.                   │
│                                                                              │
│  RISC0_DEV_MODE=0  — proofs are real                                          │
│  Sequencer         — a real standalone LEZ node, not an in-process executor    │
└──────────────────────────────────────────────────────────────────────────────┘
BANNER

echo "RISC0_DEV_MODE=${RISC0_DEV_MODE}"
echo "commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'not a git checkout')"
echo

exec ./scripts/e2e-local-sequencer.sh "$@"
