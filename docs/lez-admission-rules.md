# What LEZ checks before a transaction reaches a block

`execute` was submitted and never confirmed four times — twice on the public testnet, once in CI,
once locally — while the program's own tests passed. This document is the result of reading the
layer that was actually rejecting it, and it maps every rule to how this program satisfies it.

## The layer nobody was testing

There are two distinct layers, and only one of them was covered:

| | |
|---|---|
| `validate_execution` (`lee/state_machine/core/src/program/mod.rs`) | eight rules about a program's own input/output. `crates/sdk/tests/multisig_program.rs` exercised this and passed throughout |
| `ValidatedStateDiff::from_public_transaction` (`lee/state_machine/src/validated_state_diff/mod.rs`) | admission. Eighteen rules, and it *calls* `validate_execution` as one step. **Every rejection came from here** |

That is why the executor tests were green while the chain refused the transaction: they tested the
inner layer. It is also why the public testnet was useless for diagnosis — it reports only
`Transaction not found in preconfigured amount of blocks`. The actual reason exists only in a
sequencer's log, which is available when you run one locally.

## Every rule, and how `execute` satisfies it

`execute` submits four accounts: `config` (PDA), `proposal` (PDA), `recipient`, `submitter` (signer).

| Rule | How this transaction satisfies it |
|------|-----------------------------------|
| Public transaction must have at least one account | four |
| No duplicate `account_ids` | the payee is a separate account from the submitter. **Violated once**: setting the payee to `$CREATOR`, which is also the submitter |
| Nonce count matches signature count | one signer, one nonce; the CLI fetches nonces and exits non-zero if it cannot |
| Valid signature | the submitter signs. **Violated once**: before `execute` took a signer at all, the witness set was empty. `approve` gets away without one only because it is privacy-preserving — the proof stands in for the signature |
| Unknown program | both programs are deployed before this runs |
| `InconsistentAccountPreState` | the state machine builds the pre-states from chain state itself; the program echoes them unchanged |
| `InvalidAccountAuthorization` / `AuthorizedAccountMarkedAsNotAuthorized` | checked **in both directions**, and satisfied by construction: the state machine sets `is_authorized = signer_account_ids.contains(id)` when it builds the pre-states, and the program passes them through. For a top-level call `authorized_pdas` is empty — LEZ's own test `compute_public_authorized_pdas_no_caller_returns_empty` pins that — so the two PDAs must not carry the flag, and do not |
| `MismatchedProgramId` / `MismatchedCallerProgramId` | emitted by the SPEL dispatcher from the call it was actually given |
| `ExecutionValidationFailed` | the inner eight rules. **Violated once**: the treasury held nothing, so `checked_sub` failed |
| `DefaultAccountModifiedWithoutClaim` | the payee is an existing account owned by auth-transfer. **Violated once**: paying `0xc3c3…c3`, an address nobody had ever used. Claiming it is not an option — a multisig must not take ownership of the account it pays |
| `ClaimedNonDefaultAccount`, `ClaimedUnauthorizedAccount`, `MismatchedPdaClaim` | `execute` requests no claims |
| `DeclaredAccountMissingFromOutput` | all four accounts are returned |
| `MaxChainedCallsDepthExceeded` | `execute` makes no chained call; only `approve` does |

## Why the payee cannot be anything simpler

Three rules together leave exactly one shape. The payee must **already exist** (a never-used account
cannot be credited), must be **owned by a program** (`validate_execution` rule 7 refuses a default
owner with non-default state), and must be **distinct from the submitter** (ids in a message are
unique). Hence: a second public account, created and initialised under auth-transfer.

## What this cost, and what it should have cost

Four rejections, each found by running the full demo — two proofs and a sequencer build, roughly
fifty minutes — to learn one rule. Reading this module takes minutes.

`every_instruction_satisfies_lez_admission_rules` in `crates/sdk/tests/multisig_program.rs` now
transcribes the rules that can be checked without the `lee` crate, which lives in an uncommitted
local checkout and so cannot be a dependency. It catches two of the four in 0.1 seconds; it is
mutation-tested against both.
