# Phase C status — SPEL multisig + membership program

**Date:** 2026-09-04
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase C
**Result:** **all 8 SC green — Phase C complete.**

Abort check at phase start: #125 `reviewDecision` empty; merged LP-0002 PRs → 0. Not aborting.

## Verification run

| Command | Exit |
|---------|------|
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace --all-targets` | 0 — **74 tests** (was 46) |
| `./scripts/generate-idl.sh` | 0 — 4 instructions, all four lifecycle steps asserted present |
| `shellcheck -S warning scripts/*.sh` | 0 |
| `./scripts/check-dev-mode-clobber.sh` | 0 |

## Success criteria

| SC | Requirement | State | Evidence |
|----|-------------|-------|----------|
| **SC-C.1** | IDL published (**P-U3**) | ✅ green | `artifacts/multisig-idl.json`, generated from the `#[lez_program]` annotations at compile time by `scripts/generate-idl.sh`, so it cannot drift from the instruction set. Program `private_multisig`, 4 instructions |
| **SC-C.2** | Lifecycle test passes (create→propose→approve×M→execute) | ✅ green | `the_full_lifecycle_completes_at_full_m` — a 2-of-3 taken through both approvals to execution, asserting refusal at 1-of-2 on the way (**H13/W15**: the primary path is full M, never a lowered tier) |
| **SC-C.3** | Double approve → documented error code (**P-F3**, **P-R3**) | ✅ green | `a_member_cannot_approve_the_same_proposal_twice` → **1002 `DuplicateNullifier`** (`7002` on chain), and the rejected approval leaves no trace. Plus `a_member_cannot_double_vote_from_another_of_their_addresses` |
| **SC-C.4** | Early execute → documented error code | ✅ green | `executing_before_the_threshold_is_rejected` → **1004 `ThresholdNotMet`**; also `executing_twice_is_rejected` → 1005, `approving_after_execution_is_rejected` → 1008 |
| **SC-C.5** | Invalid proof → documented error code | ✅ green | `an_approval_from_the_wrong_verifier_is_rejected` → **1013 `WrongMembershipProgram`**; stale root → 1007; wrong multisig/proposal → 1006. An invalid *witness* is rejected one layer down, in the guest (Phase B) |
| **SC-C.6** | State layout has **no** voter identity list (**P-F2**) | ✅ green | `Proposal` has six fields — version, config_hash, proposal_id, action, nullifiers, executed — and no roster, bitmap or approver list. `the_executed_state_records_a_threshold_and_no_identities` encodes a completed 2-of-3 and asserts no member `npk` or `nsk` appears in the bytes |
| **SC-C.7** | Valid proof but wrong `config_hash`/M → PDA/ownership fail | ✅ green | `a_lowered_threshold_does_not_match_the_address` → **1003 `ConfigHashMismatch`**; same for a substituted member set and a substituted verifier |
| **SC-C.8** | Doc + test: approve on a public tx path unsupported/rejected (**H9**) | ✅ green | `there_is_no_public_approve_path`. See the note below on how this is guaranteed |

## ADR-002: the verifier is now part of `config_hash`

Implementing `approve` exposed a hole ADR-001 had left. The chained call proves *some* program with a
given id ran and accepted the witness — but nothing said **which** program that had to be. An attacker
could stand up a permissive "membership" program, create a multisig naming it, and approve at will:
honest member set, real threshold, hostile verifier.

Per ADR-001's own rule, [ADR-002](adr/ADR-002-bind-verifier-to-config-hash.md) was written *before*
the code change. The formula gained a field:

```
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] ‖ membership_program_id[32] )
```

Naming a different verifier now changes the address, so the multisig is simply not there — the same
failure mode as lowering `M`. Storing the verifier in the account instead was rejected as
trust-on-first-use. The string is identical in README and ADR-001 §3 (PF-13 normalisation), and
`a_substituted_verifier_does_not_match_the_address` tests it.

The cost, recorded in `docs/limitations.md`: a new verifier build has a new ImageID, so it changes
every `config_hash`. Existing multisigs keep working against the verifier they were created with.

## How SC-C.8 is guaranteed

There is no public approve path to *reject at runtime*, because there is nothing to reject: `approve`
takes the verifying program's id from the **validated config account**, never from instruction data,
and the proof itself is carried by a `ChainedCall` that only exists on the privacy-preserving path. A
transaction that carried no chained call has no verified output for `env::verify` to cover, so it
never becomes valid.

The structural guarantee is the signature: there is no `approve` overload that omits `verified_by`.

An earlier revision also carried a `PublicApprovePathRejected` (1011) error code "for the dispatcher
to return in that case". It was retired: nothing could ever construct it — the case is
unrepresentable, not rejected at runtime — and its only test asserted the constant against itself.
Documenting an error the program claims to raise and never does is an overclaim, so the code is gone
and `there_is_no_public_approve_path` now asserts the real behaviour: a verifier id naming no bound
program is refused with **1013 `WrongMembershipProgram`** (`7013` on chain). See
[`error-codes.md` §1.3](error-codes.md).

## Shape of the implementation

Same split that worked in Phase B: the rules are pure functions in `pmsig_multisig_core::logic`,
tested on the host in milliseconds (19 lifecycle tests), and `programs/multisig-spel` is a thin SPEL
wrapper that supplies accounts, encodes state and emits the chained call. The wrapper compiles
against SPEL `main` @ `5126b7ed8a9b`.

## Not yet proven — and not claimed

**The end-to-end PPE composition has not been demonstrated.** This phase establishes that the program
compiles, emits the chained call, and enforces the right rules once a proof exists. It does **not**
yet show a real transaction going through LEZ's privacy-preserving circuit with `env::verify`
covering the membership output, because that needs a running sequencer — which is Phase E's job
(`demo.sh` against a standalone sequencer, `RISC0_DEV_MODE=0`).

Until that runs, the composition is designed and unit-tested, not demonstrated. It is listed as an
open risk rather than folded into the green above.

## Exit

All SC-C green → **proceed to Phase D** (SDK, CLI, restart-resume, peer privacy).
