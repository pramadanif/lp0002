#!/usr/bin/env bash
# build-basecamp.sh — regenerate and build the Logos Basecamp module (criterion P-U2).
#
# Stages:
#   1. regenerate the Qt/QML scaffold from the IDL   (needs spel-client-gen)
#   2. re-apply our hardening to the generated files (see below)
#   3. build the Qt plugin                            (needs Qt6 + CMake)
#   4. package a distributable .lgx                   (needs the `lgx` tool)
#
# Stage 1 and 2 work anywhere. Stages 3 and 4 need a toolchain that is not on every machine, so this
# script FAILS with instructions rather than skipping — a "successful" run that silently produced no
# package would be worse than no run (gate H2).
#
# Usage:  ./scripts/build-basecamp.sh [--regen]
#           --regen  overwrite the generated scaffold from the current IDL
#
# ─── Why stage 2 exists ─────────────────────────────────────────────────────────────────────────
#
# The generator adds a "recent values" history to input fields, stored via QSettings. That is fine
# for a config hash and catastrophic for the approval witness, which carries a member's nullifier
# secret key. It currently escapes only because the generator skips `Vec<u8>` fields — luck, not
# design. `scripts/check-basecamp-privacy.sh` turns that into an enforced property, and this script
# runs it before declaring success.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

REGEN=0
[[ "${1:-}" == "--regen" ]] && REGEN=1

die() { echo "FATAL: $*" >&2; exit 1; }
info() { printf '    %s\n' "$*"; }
log() { printf '\n==> %s\n' "$*"; }

command -v cargo >/dev/null 2>&1 || die "cargo is required. Install Rust: https://rustup.rs"
[[ -s artifacts/multisig-idl.json ]] || die "artifacts/multisig-idl.json missing. Run ./scripts/generate-idl.sh"

# ─── 1. Regenerate from the IDL ─────────────────────────────────────────────────────────────────
if (( REGEN )); then
  CG="${PMSIG_CLIENT_GEN:-.refs/spel-main/target/release/spel-client-gen}"
  [[ -x "$CG" ]] || die "spel-client-gen not found at $CG.
       Build it with:  cd .refs/spel-main && cargo build --release -p spel-client-gen"

  log "regenerating the Basecamp scaffold from the IDL"
  # --skip-ui preserves our hardened Main.qml; drop it only for a clean regeneration.
  "$CG" --idl artifacts/multisig-idl.json --out-dir app --target logos-module \
        --module-name PrivateMultisig --ffi-lib-path lib/libpmsig_ffi.dylib \
    || die "scaffold generation failed"
  info "regenerated — re-apply the manifest and QML hardening before committing"
fi

[[ -d app ]] || die "app/ does not exist. Run with --regen first."

# ─── 2. The hardening must hold ─────────────────────────────────────────────────────────────────
log "checking the UI does not leak member secrets"
./scripts/check-basecamp-privacy.sh || die "the Basecamp UI failed its privacy checks"

log "checking the manifest is complete"
python3 - <<'PY' || exit 1
import json, sys
m = json.load(open("app/manifest.json"))
required = ["name", "version", "type", "author", "description", "main", "manifestVersion"]
missing = [k for k in required if not m.get(k)]
if missing:
    print(f"FATAL: manifest.json has empty required fields: {missing}", file=sys.stderr)
    sys.exit(1)
print(f"    manifest ok — {m['name']} {m['version']}, {len(m['main'])} platform targets")
PY

# ─── 3. Build the Qt plugin ─────────────────────────────────────────────────────────────────────
log "building the Qt plugin"
missing=()
command -v cmake >/dev/null 2>&1 || missing+=("cmake")
command -v qmake6 >/dev/null 2>&1 || command -v qmake >/dev/null 2>&1 || missing+=("Qt6")
if (( ${#missing[@]:-0} > 0 )); then
  cat >&2 <<EOF
FATAL: cannot build the Basecamp plugin — missing: ${missing[*]}

  macOS:  brew install qt cmake ninja
  Linux:  apt install qt6-base-dev qt6-declarative-dev cmake ninja-build

This script does not skip the build and report success. Criterion P-U2 requires a loadable module
with downloadable assets, and a build that did not happen is not one.
EOF
  exit 1
fi

cmake -S app -B app/build -DCMAKE_BUILD_TYPE=Release || die "cmake configure failed"
cmake --build app/build --parallel || die "plugin build failed"
info "plugin built"

# ─── 4. Package the .lgx ────────────────────────────────────────────────────────────────────────
log "packaging the .lgx"
command -v lgx >/dev/null 2>&1 || die "the 'lgx' tool is not installed.
       Get it from https://github.com/logos-co/logos-package
       Criterion P-U2 requires a downloadable, loadable package."

rm -rf app/.lgx-staging && mkdir -p app/.lgx-staging
cp app/build/lib*_plugin.* app/.lgx-staging/ 2>/dev/null || die "no built plugin to package"
cp app/qml/Main.qml app/.lgx-staging/
( cd app && lgx create private_multisig ) || die "lgx create failed"
lgx add app/private_multisig.lgx -f app/.lgx-staging -y || die "lgx add failed"
python3 scripts/patch_lgx_manifest.py app/private_multisig.lgx app/manifest.json 2>/dev/null || true
lgx verify app/private_multisig.lgx || die "lgx verify failed"
rm -rf app/.lgx-staging

SHA=$(shasum -a 256 app/private_multisig.lgx | awk '{print $1}')
SIZE=$(wc -c < app/private_multisig.lgx | tr -d ' ')
log "packaged"
info "app/private_multisig.lgx"
info "sha256 $SHA"
info "bytes  $SIZE"
echo
echo "Record these in the release notes and SOLUTION (plan gate W12)."
