#!/usr/bin/env bash
# check-basecamp-privacy.sh — the Basecamp UI must not persist or display member secrets.
#
# `spel-client-gen` builds the UI from the IDL, and it adds a "recent values" history to some input
# fields, stored via QSettings. That is a sensible convenience for a config hash. It would be a
# disaster for the approval **witness**, which carries the member's nullifier secret key: a spending
# key written to a settings file on disk.
#
# Today the generator only adds history to fixed-size `[u8; N]` fields, so the witness (a `Vec<u8>`)
# escapes by luck rather than by design. This check turns that luck into a guarantee, so a future
# regeneration cannot quietly start persisting it.
#
# Checks:
#   1. no history is saved for the witness field
#   2. the witness is not written to QSettings by any other name
#   3. the UI does not display another member's account id (SC-F.6)
#
# Run by CI. Missing files fail (gate H2) once app/ exists.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }

QML=app/qml/Main.qml
BACKEND=app/src/PrivateMultisigBackend.cpp

# Phase F has run and app/ is committed, so its absence is now a broken checkout or a deleted
# directory — not a phase that has not happened yet. This used to `exit 0` here, which is the
# skip-to-green pattern gate H2 forbids and the one #125 was pulled up for: a privacy check that
# passes because it could not run tells an evaluator nothing.
if [[ ! -d app ]]; then
  echo "FATAL: app/ is missing. It is committed, so this is a broken checkout." >&2
  echo "       Regenerate with ./scripts/build-basecamp.sh — this check does not pass by default." >&2
  exit 1
fi

for f in "$QML" "$BACKEND"; do
  [[ -f "$f" ]] || { echo "FATAL: $f missing — regenerate with scripts/build-basecamp.sh" >&2; exit 1; }
done

echo "check-basecamp-privacy.sh"
fail=0

# 1. The witness must never be history-saved.
if grep -q 'saveHistory("approve_witness' "$QML"; then
  echo "  FAIL  the approval witness is saved to field history — that is a spending key on disk" >&2
  fail=1
else
  echo "  OK    the approval witness is not saved to field history"
fi

# 2. Nothing else may persist it either. The witness reaches the backend as a parameter; it must not
#    appear anywhere near a settings write.
if grep -nE 'setValue|QSettings' "$BACKEND" | grep -qi 'witness'; then
  echo "  FAIL  the witness appears near a QSettings write in the backend" >&2
  fail=1
else
  echo "  OK    the backend never writes the witness to settings"
fi

# 3. SC-F.6: the UI must not surface another member's identity. The program has no instruction that
#    takes one, so the check is that no such field was generated.
if grep -qiE 'id: (approve|execute)_[a-z_]*(member_account|approver_account|voter|member_id)' "$QML"; then
  echo "  FAIL  the UI exposes a member account field" >&2
  fail=1
else
  echo "  OK    no member-identity field in the UI (SC-F.6)"
fi

# 4. The witness field should at least be marked as sensitive to the user.
if grep -q 'approve_witnessf' "$QML" && ! grep -qiE 'secret|sensitive|never shared|do not share' "$QML"; then
  echo "  WARN  the witness field is not labelled as secret in the UI"
fi

echo
if (( fail )); then
  echo "FAILED: the Basecamp UI would leak member secrets." >&2
  exit 1
fi
echo "Basecamp UI privacy checks passed."
