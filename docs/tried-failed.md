# Tried and failed

A running, honest log of approaches attempted and abandoned, and of things that surprised us. Kept
because the reasoning behind a rejected path is worth as much to a reviewer as the path taken — and
because a repo where nothing ever went wrong is not a believable repo.

Entries are appended as work happens. Design-time rejections that were never coded live in
ADR-001 §7; this file is for things actually attempted or discovered by measurement.

---

## Phase −1

### Pinning LEZ by "latest tag" would have been wrong — and quietly so

**Tried.** The obvious pin is the newest tag, or else the one a competitor used (#123 pinned v0.2.2).
The SPEL framework's released version, v0.6.0, pins LEZ **v0.2.0**, so "use released SPEL" looked like
the safe, conservative choice.

**Failed, because.** Address derivation is not stable across those versions. At v0.2.0,
`AccountId::for_regular_private_account` hashes `prefix ‖ npk ‖ identifier` (80 bytes). At v0.2.4 it
hashes `prefix ‖ npk ‖ vpk ‖ identifier`, where `ViewingPublicKey::LEN = 1184`. Building against
v0.2.0 derives shielded account addresses **the live testnet does not recognise** — and the failure
would surface late, as unexplained rejections during testnet deployment in Phase G.

**What we did instead.** Fingerprinted the deployed version rather than guessing. A LEZ `ProgramId`
is the risc0 ImageID of the program ELF, and LEZ commits prebuilt binaries under `artifacts/` at every
tag. Computing those ImageIDs and comparing with the live `getProgramIds`:

| Program | Live testnet | v0.2.0 | v0.2.4 |
|---------|--------------|--------|--------|
| `token` | `ccc4713e…` | `c5d50f88…` ✗ | `ccc4713e…` ✓ exact |
| `privacy_preserving_circuit` | `383e884f…` | `ab86d257…` ✗ | `383e884f…` ✓ exact |

So: pin LEZ **v0.2.4**, and pin SPEL to **`main`** (which tracks v0.2.4) rather than its released
v0.6.0. Depending on an unreleased SPEL commit is a real cost, and it is disclosed in
`docs/limitations.md` rather than hidden.

Evidence: `artifacts/phase-N1-testnet-version-fingerprint.txt`.

### `getBlockHeight` is not a LEZ RPC method

**Tried.** Probing the testnet with `getBlockHeight`, by analogy with other chains.

**Failed.** `{"error":{"code":-32601,"message":"Method not found"}}`. Reading
`sequencer/service/rpc/src/lib.rs` gives the actual set: `sendTransaction`, `checkHealth`, `getBlock`,
`getBlockRange`, `getLastBlockId`, `getAccountBalance`, `getTransaction`, `getAccountsNonces`,
`getProofForCommitment`, `getAccount`, `getProgramIds`. The height method is `getLastBlockId`.

Minor, but it is the whole reason this repo reads LEZ's source for every interface instead of
assuming shapes from other ecosystems.

## Phase 0

### `mapfile` is not available to macOS evaluators

**Tried.** `mapfile -t targets < <(…)` in `scripts/check-dev-mode-clobber.sh`.

**Failed.** macOS ships bash **3.2.57** as `/bin/bash`, and `mapfile` arrived in bash 4. The script
died with `mapfile: command not found` — and, worse, `set -u` then made it exit non-zero for the
*wrong reason*, which would have looked like a real H3 violation.

**What we did instead.** A `while IFS= read -r` loop. Every script in this repo is written for bash
3.2, because the prize says evaluators clone the repo and run the demo from a clean environment, and
some of those environments are Macs.

## Phase A

### The membership guest cannot take `nsk` on trust

**Nearly shipped.** The first sketch had the membership guest verify `npk ∈ member_root` and compute
the approval nullifier from the witness `nsk` — and stop there. Membership proven, nullifier bound to
a secret; it reads as complete.

**Why that is not enough.** Nothing tied the witness to the transaction. The guest would prove only
*"someone knows an `nsk` whose `npk` is in the member set"* — a statement about key material, not
about chain state. It is true of a member who never created a shielded account, true of one whose
account has been fully spent, and it remains true forever once the key exists. That is exactly the
derivation-only property reviewers rejected in prize PR #91, and gate H8 exists to catch it.

**Fix.** The guest re-derives `AccountId::for_regular_private_account(npk, vpk, identifier)` from its
own witnesses and asserts it equals the approver's `pre_state.account_id` — the account LEZ's PPE
circuit independently proved is live, unspent and being spent in *this* transaction (ADR-001 D4).

### A wrong reason for the right fix — corrected

**Claimed, in the first draft of ADR-001 D4 and of this file:** that without the account-binding
assertion a member could double-vote, by handing their real `nsk` to LEZ's PPE circuit and a
*different* `nsk` to the membership guest, minting a fresh nullifier on each attempt.

**That reasoning is wrong**, and it is recorded here rather than quietly deleted. The substituted
`nsk` would have to derive an `npk` that is itself a leaf under `member_root` — i.e. the attacker
would need a *second* key that is already a member. A member holds one. So the substitution fails at
the membership check, before the nullifier is ever computed.

**What is actually true.** Double-voting is prevented by INV-4 alone: `nf_approve` is a deterministic
function of `(nsk, multisig_id, proposal_id)`, and a member has only one `nsk`. The account-binding
assertion buys something different and still necessary — **liveness**. Without it, an approval's
on-chain footprint could be an account with no relationship to the member: the transaction spends some
account, the witness names a member key, and nothing connects them.

The fix did not change; the justification did. `SC-B.5` now tests the property that is actually at
stake — a witness that does not control the presented account is accepted by a derivation-only variant
and rejected by the real one.

## Phase B

### The guest journal was leaking the member's spending key

**Shipped, briefly.** The first working membership guest took the whole witness — `nsk`, `vpk`,
`identifier`, Merkle path — as its instruction argument, the obvious shape for a program that has to
verify all of it.

**Why that is bad.** A LEZ program echoes its `instruction_data` into the `ProgramOutput` it writes,
and `ProgramOutput::write()` **commits to the guest's journal**. So the journal contained the
member's `nsk` verbatim.

On-chain privacy would have survived: the inner `ProgramOutput` never reaches the chain, because
LEZ's privacy-preserving circuit consumes it via `env::verify` and commits only
`PrivacyPreservingCircuitOutput`. But the inner receipt would have been a **spending key sitting in a
file** — a worse failure than the identity leak the prize is about, and one that any SDK caching or
debug dump would have turned into key loss.

**How it was nearly missed.** The first version of the SC-B.4 test scanned the journal for the raw
32 bytes of `nsk` and reported it clean. That scan is a **false negative**: risc0's serde writes each
`u8` as its own 32-bit word, so a 32-byte secret occupies 128 journal bytes and never appears as a
contiguous run. Decoding the journal properly showed the witness could be read straight back out:

```
raw_nsk=false            <- what the naive scan saw
word_nsk=true            <- the same secret, word-encoded
witness_recoverable_from_journal=true
```

**Fix.** The instruction was split. `ApprovalClaim` — `multisig_id`, `proposal_id`, `member_root`,
`claimed_nullifier`, all already public on chain — stays in `instruction_data`. `ApprovalWitness`
moved to a **separate private input**, read with `env::read()` after the standard LEZ inputs and
never echoed into `ProgramOutput`. The chained-call check that the caller's `instruction_data`
matches the callee's still holds, because both sides carry the claim.

`the_journal_carries_no_member_secret` now decodes the journal instead of scanning it, and asserts
the recovered instruction is exactly the public claim.

**What remains in the journal, by necessity:** the approver's `account_id`, in `pre_states`. Every
LEZ program commits its pre/post states — that is how the runtime validates execution — so this is
not removable and is not specific to this design. It is why `docs/security.md` records that **inner
receipts are prover-local secret material** that the SDK never persists or transmits.

### A second proof in the same process does not finish in reasonable time

**Observed, not explained.** `scripts/prove-bench.sh` runs two proving tests sequentially with
`--test-threads=1`. The first completes reliably — 123.9 s before the witness split, 53.3 s after.
The **second** proof in the same process ran for over 25 minutes on two separate occasions without
completing, at ~740% CPU in `r0vm` throughout, for a guest of only 598 k cycles.

**What is known:** the guest is small; the first proof of an identical workload takes under a minute;
`ProverOpts::default()` is `ReceiptKind::Composite`, so this is not recursion. Version skew was ruled
out — r0vm was aligned to 3.0.6 and the behaviour persisted.

**Not diagnosed.** Plausible candidates are r0vm session/process reuse across successive
`prove_with_opts` calls, or host memory pressure on an 8-core laptop. Recorded here rather than
guessed at, and it will be filed upstream if it reproduces on a clean machine
(`docs/BUGS_FILED.md`).

**Why it does not block Phase B.** The property the second test asserts — that the journal carries no
member secret — does not depend on proving. A journal is determined by what the guest commits, and is
byte-identical whether the session is executed or proved. `the_journal_carries_no_member_secret`
asserts it by execution, decoding the journal, and runs in CI in under a second. The proved variant is
kept, `#[ignore]`d, as a belt-and-braces check for when the slowdown is understood.
