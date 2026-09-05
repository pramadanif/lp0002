# Phase F status — Basecamp + Release

**Date:** 2026-09-05
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase F
**Result:** **IN PROGRESS — 2 of 7 SC green. Blocked on a toolchain that is not installed.**

Abort check at phase start: #125 `reviewDecision` empty; merged LP-0002 PRs → 0. Not aborting.

## Success criteria

| SC | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| **SC-F.1** | README build/load instructions work (**P-U2**) | ⛔ | Instructions exist in `scripts/build-basecamp.sh`, but nobody has run them to completion, so they are unverified |
| **SC-F.2** | Release assets downloadable (**H10**) | ⛔ | No `.lgx` built — the `lgx` tool is not installed |
| **SC-F.3** | SHA256SUMS match downloaded bytes | ⛔ | Nothing to hash yet |
| **SC-F.4** | `lgx verify` / ui-host READY recorded | ⛔ | `lgx` not installed |
| **SC-F.5** | `module.json` / metadata present, no validator warning | ✅ | `app/manifest.json` — every required field populated, **6** platform targets. See "the manifest was a trap" below |
| **SC-F.6** | GUI shows threshold progress **without** other members' account ids | ✅ | `scripts/check-basecamp-privacy.sh` — enforced and mutation-tested, not merely observed |
| **SC-F.7** | Release notes carry `.lgx` SHA-256 + byte size (**W12**) | ⛔ | No package yet |

## What exists

The Basecamp module scaffold is **generated from our own IDL**, not hand-written:

```
app/manifest.json          module metadata (module.json equivalent)
app/module.yaml            Logos module descriptor
app/CMakeLists.txt         Qt plugin build
app/qml/Main.qml           the UI, one panel per instruction
app/src/PrivateMultisig*   C++ backend + Qt plugin, FFI into our Rust client
```

Generated with `spel-client-gen --target logos-module` from `artifacts/multisig-idl.json`, so the UI
and the on-chain program cannot drift apart: regenerate and the UI follows the IDL.

## The finding: the generated UI nearly wrote a spending key to disk

`spel-client-gen` adds a "recent values" history to input fields, persisted through `QSettings`.
Sensible for a config hash. **Catastrophic for the approval witness**, which carries the member's
`nullifier_secret_key`.

Checked rather than assumed. It turns out the witness escapes — but **only by luck**: the generator
adds history to fixed-size `[u8; N]` fields and skips `Vec<u8>`, and the witness happens to be a
`Vec<u8>`. Change the witness encoding, or change the generator, and a spending key lands in a
settings file.

So the luck is now an enforced property. `scripts/check-basecamp-privacy.sh` asserts:

1. the witness is never history-saved;
2. the backend never writes it to `QSettings` under any name;
3. no member-identity field exists in the UI (**SC-F.6**);
4. the witness field is labelled as secret to the user.

**Mutation-tested**: injecting `saveHistory("approve_witnessf", …)` into the QML makes the check fail;
removing it makes the check pass. A gate that has never failed is not a gate.

Check (4) was a real finding, not a formality — the generated field carried no warning at all. The UI
now shows: *"⚠ SECRET — this witness contains your nullifier secret key. It never leaves this machine
except inside a proof, is never saved to disk, and must never be shared."*

## The manifest was a trap too

As generated, `manifest.json` had **empty `author`, empty `icon`**, and declared only
`linux-amd64` targets. A competing submission to this prize was marked down for exactly a
`module.json` validator warning, so this matters.

Fixed: every required field populated, licence and homepage added, and **6** platform targets
including `darwin-arm64` / `darwin-amd64`. `scripts/build-basecamp.sh` refuses to proceed if any
required field is empty, so a future regeneration cannot silently reintroduce the blanks.

## What is blocked, and on what

Stages 3 and 4 of `scripts/build-basecamp.sh` need a toolchain this machine does not have:

| Tool | Needed for | Status |
|------|-----------|--------|
| Qt6 + CMake + Ninja | building the Qt plugin | **not installed** (~2–3 GB) |
| `lgx` | packaging the distributable archive | **not installed** — from `logos-co/logos-package` |

The script **fails with instructions** rather than skipping. A run that quietly produced no package
while reporting success is exactly the failure mode gate H2 exists to prevent, and P-U2 asks for a
loadable module — a build that did not happen is not one.

`.lgx` itself is a gzipped tarball with a `manifest.json` at its root, produced by
`lgx create` / `lgx add` and checked with `lgx verify` (learned from SPEL's own Makefile, not guessed).

## Exit

**Not taken.** Phase F needs Qt6, CMake and `lgx` installed, then stages 3–4 of
`scripts/build-basecamp.sh`, then release notes carrying the `.lgx` SHA-256 and byte size (W12).
