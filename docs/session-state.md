# Where this stands — 2026-09-07

A reminder to pick up from, written at the end of a long debugging session. Every number here was
read from a command, not recalled.

```
128 tests / 0 failed · fmt 0 · clippy 0 · shellcheck 0 · links 0
demo.sh: COMPLETE, exit 0, execute confirmed, treasury 0 / payee 100
```

## `demo.sh` completed, and `execute` moved the money

A full run finished on 2026-09-07 00:46 — every stage, `DEMO COMPLETE`, exit 0. Both approvals
proved with `RISC0_DEV_MODE=0` (19 and 20 minutes), then `execute` confirmed in a block. It is the
first time `execute` has succeeded anywhere, so the payee and `submitter` fixes are now evidence
rather than argument.

The money genuinely moved. Read back over JSON-RPC from the completed chain:

| account | balance |
|---|---|
| config PDA `9KywRP…eH8T` (the treasury) | 0 |
| payee `GfipZY…HcRF` | 100 |

Kept in `evidence/execute-inv7-local.md`, because the next run wipes the sequencer's RocksDB.

Before starting a run, free memory: a composed approval peaks near 8.7 GB, and the gate refuses
below 7 GB (`PMSIG_MIN_FREE_GB`). The successful run started from 7.4 GB. Note that `free_gb()`
counts only free + inactive + speculative pages, so it reads low while the file cache is still warm
from a previous run — right after one it reported 6.7 GB on an otherwise idle machine that
`memory_pressure` called 82% free. Waiting a few minutes, or closing Chrome, clears it.

## Six bugs fixed today, all on the `execute` path

Each hid the next, which is why four full demo runs were spent finding them one at a time:

| | bug | how it surfaced |
|---|---|---|
| 1 | `build-guests.sh --docker` never worked (`--bin` unsupported) | running it |
| 2 | treasury never funded — INV-7 moved the source of funds and no step filled it | execute rejected |
| 3 | `execute` had no signer, so an empty witness set | execute rejected |
| 4 | `auth-transfer init` skipped: the nonce means "used", not "initialised" | transfer rejected |
| 5 | payee was an address nobody had ever used | `DefaultAccountModifiedWithoutClaim` |
| 6 | payee was also the submitter | `Duplicate account_ids` |

Plus: `demo.sh` aborted midway and exited 0 (a completion sentinel now prevents that), `wallet
check-health` hung for four hours unbounded (now 60 s), and the demo was not idempotent — it worked
once per machine and failed after.

## The lesson worth keeping

**The tests covered the wrong layer.** `validate_execution` has eight rules about a program's own
output; `ValidatedStateDiff::from_public_transaction` has eighteen and calls it as one step. Every
rejection came from the outer layer, so the executor tests stayed green throughout.

The public testnet cannot diagnose this — it reports only `Transaction not found in preconfigured
amount of blocks`. **The reason exists only in a local sequencer's log.** Run locally first, always.

`every_instruction_satisfies_lez_admission_rules` now transcribes the checkable rules and catches
two of the six in 0.1 seconds. It is mutation-tested against both.

## The artifacts the repo ships are not reproducible

Found while checking what the completed run had dirtied. `git log` on `artifacts/IMAGE_IDS.md` shows
**one** commit, `566286f`, built reproducibly. Every commit since — HEAD included — records
`local toolchain — NOT reproducible, do not deploy or quote in a submission`, because each demo run
built straight into `artifacts/` and replaced them.

This was never cosmetic: a LEZ `ProgramId` **is** the guest's ImageID. Reproducible membership is
`960db4f2…`, local is `f5cc9f37…` — different program ids, and through `config_hash` (ADR-002) a
different address for every multisig the repo documents. `deploy-testnet.sh` refuses artifacts
marked NOT reproducible, so the deployment path was protected; the repo's own record was not.

Fixed at the root: `build-guests.sh` writes to `$PMSIG_ARTIFACTS_DIR` (default `artifacts/`), and
the demo points it at `.e2e/run/artifacts`. When the committed binaries are reproducible **and**
current, the demo skips the build and runs the very binaries the submission ships.

`566286f` is far behind the current sources, so a fresh `./scripts/build-guests.sh --docker` is
required before anything is deployed or quoted.

## Still open, in priority order

1. `./scripts/build-guests.sh --docker`, commit the artifacts — blocks 2, and shortens the video
2. Redeploy to testnet on those ImageIDs → **P-F6, P-F7, P-S1, P-P1**. `execute` is proven locally
   but has never run on the public testnet
3. CI `e2e-sequencer` green → **P-S2**
4. Basecamp `.lgx` → **P-U2**. The toolchain is installed now (cmake, ninja, Qt6, `lgx` 0.1.0), but
   the module still cannot link: **there is no FFI crate**, and the generated UI calls thirteen
   `extern "C"` functions nothing provides.
5. Narrated video → **P-S6**. Human gate.

Not to be done without the operator: opening the solution PR to `logos-co/lambda-prize`.
