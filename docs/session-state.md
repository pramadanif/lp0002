# Where this stands — 2026-09-06

A reminder to pick up from, written at the end of a long debugging session. Every number here was
read from a command, not recalled.

```
128 tests / 0 failed · fmt 0 · clippy 0 · shellcheck 0 · links 0
preflight: pass=13 fail=0 pending=4  → NOT READY (correct)
1 commit not yet pushed (the network dropped; push it first)
```

## The one thing to do next

**`demo.sh` has never completed.** It now gets as far as the RAM gate with everything before it
working — faucet, deploy, create 2-of-3, fund treasury, create payee, propose. What remains unproven
is the last stretch: two proofs (~40 min) and then `execute`.

Before starting a run, free memory: a composed approval peaks near 8.7 GB and the gate refuses below
8 GB. Chrome alone held 3.6 GB across ten processes on the last attempt; closing it took free memory
from 6.8 GB to 9.8 GB. Docker holds a VM too if it has been started.

## What is fixed but **not yet proven on chain**

Both landed after the last chain run, so neither has faced a sequencer:

- **the payee** — a second public account, created and initialised under auth-transfer
- **the `submitter`** — a signer account on `execute`

`docs/lez-admission-rules.md` argues on paper that the current shape passes all eighteen admission
rules. That is an argument, not evidence. Treat `execute` as unproven until a run confirms it.

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

## Still open, in priority order

1. One clean `demo.sh` run → **P-S5**, and the CI e2e green → **P-S2**
2. Redeploy to testnet (guest ImageIDs changed) → **P-F6, P-F7, P-S1, P-P1**
3. `demo.sh` overwrites the committed reproducible artifacts with local non-reproducible builds —
   a reviewer running it silently downgrades them. Not yet fixed.
4. Basecamp `.lgx` → **P-U2**. The toolchain is installed now (cmake, ninja, Qt6, `lgx` 0.1.0), but
   the module still cannot link: **there is no FFI crate**, and the generated UI calls thirteen
   `extern "C"` functions nothing provides.
5. Narrated video → **P-S6**. Human gate.

Not to be done without the operator: opening the solution PR to `logos-co/lambda-prize`.
