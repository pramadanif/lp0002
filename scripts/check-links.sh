#!/usr/bin/env bash
# check-links.sh — every relative Markdown link in this repository must resolve.
#
# WHY
#
# A reviewer reads the submission at a pinned commit. A link that 404s there is not a typo, it is a
# missing piece of the argument — and prize PR #125 was pulled up for exactly that, its
# `limitations.md` link dead at its own pin. This repository links its documentation heavily, so the
# failure is cheap to introduce and invisible without a check.
#
# Only relative links are checked. External URLs are not fetched: this must work offline, in CI, and
# at any commit, and a flaky network must never fail a submission gate.
#
# docs/plan/ is skipped. It holds verbatim copies of upstream files whose relative links point into
# the upstream repository; see docs/plan/README.md for why they are not edited to fit.

set -euo pipefail
cd "$(dirname "$0")/.." || { echo "cannot cd to repo root" >&2; exit 1; }
[[ -f README.md && -d .git ]] || {
  echo "FATAL: not at the repo root (got $PWD)." >&2
  exit 2
}

python3 - <<'PY'
import os, re, sys, urllib.parse

SKIP_DIRS = {".git", "target", ".e2e", "node_modules", ".refs"}
SKIP_PREFIX = os.path.join(".", "docs", "plan")

bad, checked, files = [], 0, 0
for root, dirs, names in os.walk("."):
    dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
    if root.startswith(SKIP_PREFIX):
        continue
    for name in sorted(names):
        if not name.endswith(".md"):
            continue
        path = os.path.join(root, name)
        files += 1
        text = open(path, encoding="utf-8", errors="replace").read()
        for m in re.finditer(r'\[[^\]]*\]\(([^)]+)\)', text):
            link = m.group(1).strip()
            if link.startswith(("http://", "https://", "#", "mailto:")):
                continue
            target = urllib.parse.unquote(link.split("#", 1)[0])
            if not target:
                continue
            checked += 1
            if not os.path.exists(os.path.normpath(os.path.join(root, target))):
                line = text[: m.start()].count("\n") + 1
                bad.append(f"{path}:{line} -> {link}")

print(f"check-links.sh: {checked} relative link(s) across {files} file(s)")
if bad:
    print(f"  FAIL  {len(bad)} broken:", file=sys.stderr)
    for b in bad:
        print(f"        {b}", file=sys.stderr)
    sys.exit(1)
print("  OK    every relative link resolves")
PY
