#!/usr/bin/env bash
# check-idl-flags.sh — every flag the scripts pass to the SPEL CLI must exist in the IDL.
#
# WHY
#
# `--submitter` was added to `execute` and, by a careless pattern replace, to `create_proposal` too,
# which has no such account. The CLI rejected it with a message about global flow flags — a full
# demo run, sequencer build included, spent to learn that a flag name was wrong. This check answers
# the same question in milliseconds, before anything is built.
#
# It compares each invocation's `--flags` against the accounts and args the IDL declares for that
# instruction. PDA accounts are excluded: the CLI derives those and takes no flag for them.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }
[[ -s artifacts/multisig-idl.json ]] || { echo "FATAL: artifacts/multisig-idl.json missing" >&2; exit 1; }

python3 - <<'PY'
import json, re, sys

idl = json.load(open("artifacts/multisig-idl.json"))
allowed = {}
for ix in idl["instructions"]:
    names = [a["name"] for a in ix["accounts"] if not a.get("pda")]
    names += [a["name"] for a in ix["args"]]
    allowed[ix["name"].replace("_", "-")] = {n.replace("_", "-") for n in names}

bad = []
checked = 0
for path in ("scripts/e2e-local-sequencer.sh", "scripts/deploy-testnet.sh"):
    lines = open(path).read().split("\n")
    current, flags, start = None, set(), 0
    for n, line in enumerate(lines, 1):
        if current is None:
            for ix in allowed:
                if re.search(rf'(^|\s|--\s){re.escape(ix)}(\s|\\|$)', line):
                    current, flags, start = ix, set(), n
                    break
            if current is None:
                continue
        flags |= set(re.findall(r'--([a-z][a-z-]*)', line))
        if not line.rstrip().endswith("\\"):
            flags.discard(current)
            unknown = flags - allowed[current] - {"idl", "bin-membership", "export", "co-signer"}
            checked += 1
            if unknown:
                bad.append(f"{path}:{start} {current} passes unknown flag(s): {sorted(unknown)}")
            current = None

print(f"check-idl-flags.sh: {checked} invocation(s) checked against the IDL")
if bad:
    print("  FAIL", file=sys.stderr)
    for b in bad:
        print(f"        {b}", file=sys.stderr)
    sys.exit(1)
print("  OK    every flag exists in the IDL")
PY
