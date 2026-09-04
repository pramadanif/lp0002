#!/usr/bin/env bash
# generate-idl.sh — emit the SPEL IDL for the multisig program (criterion P-U3).
#
# The IDL is generated from the #[lez_program] annotations at compile time, so it cannot drift from
# the program's actual instruction set. Committed to artifacts/ so reviewers and the CLI can use it
# without a build.
#
# Missing tools fail this script (gate H2).

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || { echo "FATAL: cargo not found. Install Rust: https://rustup.rs" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FATAL: jq not found (needed to format the IDL)." >&2; exit 1; }

mkdir -p artifacts
OUT=artifacts/multisig-idl.json

echo "==> generating IDL from #[lez_program] annotations"
( cd programs/multisig-spel && cargo run --quiet --bin idl ) | jq . > "$OUT"

INSTRUCTIONS=$(jq -r '.instructions | length' "$OUT")
NAME=$(jq -r '.name' "$OUT")
echo "==> wrote $OUT — program '$NAME', $INSTRUCTIONS instructions:"
jq -r '.instructions[] | "      - \(.name)"' "$OUT"

# The lifecycle the prize asks for must all be present; a truncated IDL is a silent failure.
for ix in create_multisig create_proposal approve execute; do
  jq -e --arg ix "$ix" '.instructions[] | select(.name == $ix)' "$OUT" >/dev/null || {
    echo "FATAL: instruction '$ix' missing from the generated IDL" >&2
    exit 1
  }
done
echo "==> all four lifecycle instructions present"
