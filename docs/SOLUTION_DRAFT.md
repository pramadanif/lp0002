# Solution: LP-0002 — Private M-of-N Multisig for LEZ

**Status: DRAFT — not ready to submit.** Sections marked ⛔ are unmet. See "What is not done" before
reading anything here as a claim.

**Submitted by:** pramadanif
**Repository:** https://github.com/pramadanif/lp0002
**Commit:** _pinned at submission time; not yet fixed_

---

## Summary

A private M-of-N multisig for the Logos Execution Zone. Members hold shielded LEZ accounts and
approve proposals **anonymously**: the chain records that a threshold was reached and never which
members reached it.

The precise on-chain claim, stated so a reviewer can check it rather than take it on trust:

- An approval is a **privacy-preserving transaction**. LEZ's privacy-preserving circuit runs
  `env::verify` over the multisig program **and** a chained membership program. There is no public
  approve path — not one that is discouraged, one that does not exist.
- The proof is bound to a **live** shielded account, not merely to a derived key. LEZ's circuit
  proves the approver controls an account whose commitment is in the current commitment set; our
  membership guest re-derives that same account id from its own witnesses and refuses if it differs.
  Dropping that assertion is the derivation-only pattern rejected in prize PR #91, and a test fails
  if it is removed.
- The proposal account holds an **approval count and a set of nullifiers**. It has no roster, no
  bitmap, and no field that could hold one.
- The member set, the threshold **and the membership verifier's ImageID** are all committed in the
  PDA seed. Lowering M, substituting the member set, or naming a permissive verifier does not weaken
  the multisig — it names one that does not exist.

## Why `execute` carries no proof

Answering this up front, because a previous submission to this prize was closed partly for *"the
execute transaction contains no proof"*.

The proof is at **approve** time. Each approval is a privacy-preserving transaction whose validity
depends on LEZ's circuit verifying the multisig program **and** the chained membership program. By
the time `execute` runs, the threshold is already a fact on chain: a set of distinct, proof-backed
nullifiers.

`execute` reads that verified state and moves funds if `count >= M`. It takes no secret input and
asserts nothing that was not already proven, so there is nothing left for a proof to establish.

**Keeping it public is deliberate, and better for privacy.** Anyone may execute a proposal that has
reached its threshold — including a non-member. If execution required a member, the executor *would
be* a member, and that would link a member to the proposal. A permissionless execute is what makes
criterion **P-F4** — execution unlinkable to any individual member — achievable at all.

**Which raises the obvious question: if anyone may execute, what stops them redirecting the money?**
The approvals cover the *proposal*; every other part of the transaction is chosen by whoever submits
it. So `execute` pins both ends of the transfer — the funds leave the multisig's own config PDA, and
the recipient account must be the one the proposal named, or the call is refused (**INV-7**, error
`7012` on chain).

This was a real hole, not a hypothetical one. `execute` destructured the approved action as
`{ amount, .. }`, discarding the recipient, and took a caller-supplied treasury account that nothing
tied to the multisig — so a submitter could have redirected an approved payment to themselves while
every approval still verified. It was found by running the program through the risc0 executor rather
than by reading it, and neither script would have caught it: both proposed a transfer to one address
and then executed with `--treasury $CREATOR --recipient $CREATOR`, moving money from an account to
itself. `execute_refuses_a_recipient_the_proposal_did_not_name` is the regression test; with the
binding removed it fails, showing the multisig paying an account the proposal never named.

## Architecture

```
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] ‖ membership_program_id[32] )
```

That single line is the anchor. The config account lives at
`for_public_pda(program_id, PdaSeed(config_hash))`, so its address attests to its own configuration,
and the program re-hashes the stored fields and rejects a mismatch.

**Nullifier:** `nf = SHA256(DS_NF ‖ nsk ‖ multisig_id ‖ proposal_id)` — deterministic per
(member, multisig, proposal), preimage-hiding, and keyed to `nsk` rather than an account id so a
member cannot vote twice from another of their 2^128 addresses.

Full reasoning: [ADR-001](adr/ADR-001-architecture.md) and
[ADR-002](adr/ADR-002-bind-verifier-to-config-hash.md).

## Evidence

| Claim | Where |
|-------|-------|
| Anonymous approval proved and confirmed on chain † | `artifacts/phase-E-ppe-approve-SUCCESS.txt` |
| Standalone sequencer builds, runs, serves RPC | `artifacts/phase-E-sequencer-live.txt` |
| Both programs deployed, ELF on chain | `artifacts/phase-E-deployment-local.txt` |
| Testnet funding obtained unattended | `artifacts/phase-G-faucet.txt` |
| Byte-compatibility with LEZ's own vectors | `crates/membership-core/tests/lez_compat.rs` |
| Derivation cross-checked against a real wallet account | `crates/sdk/examples/wallet_member.rs` |
| CU / proving cost | [cu-costs.md](cu-costs.md) |
| Criteria → evidence map | [criteria-checklist.md](criteria-checklist.md) |

**† That run predates the INV-7 fix**, so the multisig ImageID it records is not the one in
`artifacts/IMAGE_IDS.md` today — changing guest code changes its ImageID, and on LEZ the ImageID
*is* the ProgramId. The approve path itself is unchanged (INV-7 touched `execute` only), so what the
run demonstrates still holds, but it will be regenerated by the testnet deployment rather than left
to be cross-checked against a superseded program.

**The prize demo is `./demo.sh`.** It drives a real standalone LEZ sequencer with
`RISC0_DEV_MODE=0`. `demo-fast.sh` is a development tour, generates no proof, and is **not** cited
as evidence anywhere.

## Measurements

| | |
|---|---|
| Membership proof (standalone) | 53.26 s, 598,666 cycles |
| Composed approval (what a member actually pays) | ≈19 min 26 s, peak 8.74 GB |
| Free RAM required | ≈9 GB |
| Host | 8-core laptop, 16 GB, no GPU prover |

## What is not done ⛔

Listed first-class, because the difference between built and demonstrated is what this prize's gates
test:

- **Nothing is on the public testnet.** Funding was obtained once from the proof-of-work faucet
  (balance 300, txs in blocks 38138/38139/38148), but that wallet lived under a gitignored directory
  and no longer exists on the build machine; `fund-testnet.sh` creates and funds a fresh one
  unattended, so this is a step to redo rather than a blocker. The programs are not deployed. P-F6, P-F7, P-S1 and the
  on-chain CU figures for P-P1 are therefore unmet.
- **`demo.sh` has not completed an unattended end-to-end run.** Every step has been demonstrated —
  sequencer, deployment, create, propose, an anonymous approval — but by hand. P-S5 is not claimed.
- **The CI e2e job is wired but has not yet gone green.** `e2e-sequencer` runs on every push to
  `main`, not on cron and not path-filtered, and builds LEZ v0.2.4 and the pinned SPEL from source
  before proving anything. Each run has failed further along than the last — guest toolchain, then
  a missing `libpcsclite`, then a `SIGPIPE` panic in our own script — and each failure was a real
  defect worth fixing. Until it completes, **P-S2 is not claimed.**
- **No Basecamp `.lgx`.** The module is generated and hardened, but building it needs Qt6 and the
  `lgx` tool, neither installed. P-U2 unmet.
- **No narrated video.** P-S6 unmet.
- **Execution at full M not yet demonstrated** — one approval is on chain, not two.

Full list: [limitations.md](limitations.md).

## Why Logos

[why-logos.md](why-logos.md). Briefly: this scheme needs shielded accounts whose repeated use is
unlinkable, and execution that can depend on secrets the chain never sees. LEZ provides both in the
base layer; on a chain where validators must see the inputs, it is not possible at all.

## Honest notes

- SPEL is pinned to `main`, not the v0.6.0 release, because the release pins LEZ v0.2.0 and derives
  private account ids the live testnet does not recognise. An unreleased dependency is a real cost;
  it is taken deliberately and recorded.
- Things we got wrong and fixed are in [tried-failed.md](tried-failed.md), including a leak of the
  member's spending key into the guest journal that we shipped, then caught by decoding the journal
  rather than trusting a byte scan.
- Upstream papercuts found along the way: [BUGS_FILED.md](BUGS_FILED.md).
