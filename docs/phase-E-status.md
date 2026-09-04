# Phase E status — demo.sh + CI e2e

**Date:** 2026-09-04
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase E
**Result:** **IN PROGRESS — 5 of 8 SC green. Phase E is NOT complete, but the composition is now DEMONSTRATED.**

Abort check at phase start: #125 `reviewDecision` empty; merged LP-0002 PRs → 0. Not aborting.

This is the phase the whole plan is pointed at: it is where the privacy-preserving composition stops
being designed and starts being demonstrated. It is also the phase whose gates (H1/H2/H4) are the
ones prior submissions failed. So the status below is deliberately blunt about what has not happened.

## Success criteria

| SC | Requirement | State | Evidence |
|----|-------------|-------|----------|
| **SC-E.1** | Fresh clone `./demo.sh` → 0 with a standalone sequencer (**H1/W13**, **P-S5**) | ⛔ **not met** | `demo.sh` and `scripts/e2e-local-sequencer.sh` are written and get as far as building the sequencer. The lifecycle step is unimplemented and the script **fails there** rather than reporting a success it has not earned |
| **SC-E.2** | Log shows `RISC0_DEV_MODE=0` + real prove + sequencer RPC | ✅ green | All three demonstrated. Sequencer: builds, starts, `RPC server on 0.0.0.0:3040`, `getLastBlockId` read by the script. Real prove: an anonymous approval proved with `RISC0_DEV_MODE=0` in ≈19 min and **confirmed on chain** (tx `f2458791…198fbcb5`). Evidence: `artifacts/phase-E-sequencer-live.txt`, `artifacts/phase-E-ppe-approve-SUCCESS.txt` |
| **SC-E.3** | `check-dev-mode-clobber.sh` → 0 (**H3**) | ✅ green | Exit 0 over 5 submission-path scripts incl. `demo.sh`. Verified in both directions: injecting `export RISC0_DEV_MODE=1` makes it exit 1 |
| **SC-E.4** | CI green on `main` including a **push-gated** `e2e-sequencer` job (**H4/W14**) | ⛔ **not met** | Deliberately not wired — see below |
| **SC-E.5** | No skip / `continue-on-error` on demo/e2e (**H2**) | ✅ green | Neither string appears in `ci.yml`, `demo.sh` or `scripts/`. Every prerequisite in the e2e script is a hard `die`. Preflight PF-03 passes |
| **SC-E.6** | W3 deployed-bytes tests present | ⛔ **not met** | Requires deployed programs |
| **SC-E.7** | Missing `r0vm` → `demo.sh` **fails** (**H2**) | ✅ green | `require r0vm` in the e2e script dies with the install command. PF-03 passes |
| **SC-E.8** | Docs cite `demo.sh` as the prize demo; `demo-fast.sh` = non-criteria | ✅ green | `demo-fast.sh` states in its banner and header that it generates no proof, touches no sequencer, and is not the prize demo. PF-14 will assert it once `SOLUTION_DRAFT.md` exists |

## Why the CI e2e job is not wired yet

Wiring `e2e-sequencer` now would make `main` red, and a red default branch is itself a criteria
failure (**P-S3**). The job goes in when the e2e script passes — not before. That ordering is the
point: a CI job that is present but failing is not evidence of anything.

The `ci.yml` comment block already names the job and what it must do, so its absence is visible
rather than forgotten.

## Where the work actually stands

**Done:**
- `demo.sh` — the prize entrypoint. Sets `RISC0_DEV_MODE=0` at the top and `exec`s the e2e script.
- `scripts/e2e-local-sequencer.sh` — clones LEZ at the pinned `v0.2.4`, builds
  `sequencer_service --features standalone` (a real node with an RPC endpoint, per LEZ's own README),
  starts it, polls `checkHealth` until it answers, and reports `getLastBlockId`. Every prerequisite
  is a hard failure; there is no skip path anywhere in it.
- `programs/multisig-spel` split into an `idl` host binary and a `multisig` **guest** binary, so the
  program can be cross-compiled to `riscv32im-risc0-zkvm-elf` and deployed.
- `demo-fast.sh` — the development tour, labelled as not-the-prize-demo in three places.

**The sequencer now builds and runs.** `cargo build --release --features standalone -p sequencer_service`
completed with **0 errors** (31 MB binary) after pulling a full Logos blockchain node plus a C++
groth16 stack (circom_runtime, ffiasm, rapidsnark, pistache, googletest, rapidjson, jellyfish) —
~600 MB of git dependencies and roughly an hour of compilation on an 8-core laptop, of which the
final Rust link was 15m51s. A one-off cost per machine, but a substantial one, and CI will pay it
too unless the build is cached.

`scripts/e2e-local-sequencer.sh` now drives it end to end: builds both guests, starts the node, polls
`checkHealth` until it answers, reports `getLastBlockId`, then **fails at the unimplemented lifecycle
step** and stops the sequencer on the way out. One real fix came out of this: the sequencer wants the
config *file*, not its directory — LEZ's README shows both forms, and passing the directory fails
with `Is a directory (os error 21)`.

**Both programs are now deployed to a live local sequencer.** LEZ's own `wallet deploy-program`
put them on chain:

| Program | Bytes | ImageID | Deployment tx | Block |
|---------|-------|---------|---------------|-------|
| `membership` | 377,084 | `821c23d9…61f460ff` | `2decd739…00924f5a` | 2 |
| `multisig` | 469,516 | `cee07cd3…f465ed36` | `1a822979…6224bd92` | 4 |

Both transactions are retrievable via `getTransaction` and their payloads begin with the ELF magic,
so the bytecode is genuinely on chain. Evidence: `artifacts/phase-E-deployment-local.txt`.

**A correction that came out of this:** `getProgramIds` returns only LEZ's **built-in** programs
(`amm`, `authenticated_transfer`, `pinata`, `privacy_preserving_circuit`, `token`). It is a name
registry, not a list of deployed user programs, so a freshly deployed program does **not** appear
there — an easy thing to misread as a failed deployment. This does not affect the version fingerprint
in `docs/VERSIONS.md`, which compared exactly those built-in programs.

**Still not started, and each is substantial:**
1. Submitting a multisig instruction. `docs/VERSIONS.md` **U-6** is still open: whether SPEL's CLI can
   submit the **privacy-preserving** path at all, or only public transactions. The CLI is building.
3. The composition cost. `env::verify` inside the PPE circuit needs *succinct* receipts, i.e.
   recursion. The standalone membership proof is a 53 s **composite** receipt; a recursive
   composition is materially more expensive, and no measurement of it exists yet. Nothing in this
   repository should be read as claiming that number is known.

## The composition is now demonstrated

The privacy-preserving approval works end to end on a live sequencer:

- **tx `f245879141688563e6fad0b54d6e620cb71394bf22a6ed9d6afb2889198fbcb5`**, confirmed in a block
- approver was a **real shielded account** created by LEZ's own wallet (`Private/Dm4TU2ht…`)
- `--bin-membership` resolved the `ChainedCall`, so LEZ's PPE circuit ran `env::verify` over **two**
  chained programs — the composition ADR-001 D2/D4 describes
- `RISC0_DEV_MODE=0` throughout; ≈19 min, peak 8.74 GB, no swap movement
- the proposal account on chain now reads: **1 approval, 1 nullifier matching the client's own
  computation, `executed=false`, and no member identity anywhere**

That last line is P-F1 and P-F2 observable on chain rather than only in a unit test.

## What is still missing for Phase E

`demo.sh` does not yet drive this sequence itself — the lifecycle step in
`scripts/e2e-local-sequencer.sh` is still unimplemented, so **SC-E.1 and SC-E.4 remain unmet**. The
steps were run by hand. Turning them into the scripted demo, and only then wiring the CI job, is the
remaining work.

Also still missing: a second approval to reach full M, and `execute`.

## Exit

**Not taken.** Phase E stays open. Next actions, in order: finish the sequencer build, deploy both
programs, resolve U-6, then implement the lifecycle step in `scripts/e2e-local-sequencer.sh` and only
then wire the CI job.
