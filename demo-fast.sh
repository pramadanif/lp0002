#!/usr/bin/env bash
# demo-fast.sh — a quick tour for development. NOT THE PRIZE DEMO.
#
# Runs the guest in the risc0 executor and the lifecycle rules on the host. It does NOT start a
# sequencer and does NOT generate a proof, so it finishes in seconds instead of minutes.
#
# ─── Read this before citing it ──────────────────────────────────────────────────────────────────
#
# This script proves nothing, in the literal sense: no receipt is produced. It exists so a developer
# can check the lifecycle quickly. The prize demo is `./demo.sh`, which drives a real standalone
# sequencer with RISC0_DEV_MODE=0, and it is the only script the submission cites as evidence.
#
# Preflight check PF-14 fails the submission if any document cites this file as the prize demo.

set -euo pipefail
cd "$(dirname "$0")" || { echo "cannot cd to repo root" >&2; exit 1; }

cat <<'BANNER'
┌──────────────────────────────────────────────────────────────────────────────┐
│  demo-fast.sh — DEVELOPMENT TOUR ONLY                                         │
│                                                                              │
│  No sequencer. No proof generated. Not the prize demo.                        │
│  The prize demo is ./demo.sh                                                  │
└──────────────────────────────────────────────────────────────────────────────┘
BANNER

command -v cargo >/dev/null 2>&1 || { echo "FATAL: cargo not found." >&2; exit 1; }

echo
echo "==> lifecycle rules (host)"
cargo test -q -p pmsig-multisig-core --test lifecycle 2>&1 | tail -5

echo
echo "==> membership guest in the risc0 executor (no proof)"
cargo test -q -p pmsig-sdk --test prove_membership negatives 2>&1 | tail -5

echo
echo "==> CLI lifecycle against local state"
cargo test -q -p pmsig-cli 2>&1 | tail -5

echo
echo "Done. Remember: this generated no proof and touched no sequencer."
echo "Run ./demo.sh for the real thing."
