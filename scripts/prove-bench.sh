#!/usr/bin/env bash
# prove-bench.sh — generate a REAL membership proof and record how long it took.
#
# Evidence for SC-B.3 / criterion P-F5 ("proof generation runs client-side on a standard laptop").
#
# RISC0_DEV_MODE=0 is set explicitly and asserted inside the test. A dev-mode receipt proves nothing,
# so a benchmark run in dev mode would be worse than no benchmark: it would report a fast, meaningless
# number. Missing tools fail this script (gate H2).

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

export RISC0_DEV_MODE=0

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "FATAL: '$1' not found. $2" >&2
    exit 1
  }
}
require cargo "Install Rust: https://rustup.rs"
require r0vm  "Install the risc0 toolchain: curl -L https://risczero.com/install | bash && rzup install"

if [[ ! -s artifacts/membership.bin ]]; then
  echo "FATAL: artifacts/membership.bin missing. Run ./scripts/build-guests.sh first." >&2
  exit 1
fi

echo "=== prove-bench ==="
echo "date:            $(date -u +%FT%TZ)"
echo "commit:          $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
echo "RISC0_DEV_MODE:  ${RISC0_DEV_MODE}"
echo "r0vm:            $(r0vm --version)"
echo "host:            $(uname -sm)"
echo "cpus:            $(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo '?')"
echo "guest binary:    artifacts/membership.bin ($(wc -c < artifacts/membership.bin | tr -d ' ') bytes)"
echo

# --ignored runs precisely the two tests that generate real proofs.
cargo test -p pmsig-sdk --release --test prove_membership -- --ignored --nocapture --test-threads=1
