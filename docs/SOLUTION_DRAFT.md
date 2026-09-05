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
| Anonymous approval proved and confirmed on chain | `artifacts/phase-E-ppe-approve-SUCCESS.txt` |
| Standalone sequencer builds, runs, serves RPC | `artifacts/phase-E-sequencer-live.txt` |
| Both programs deployed, ELF on chain | `artifacts/phase-E-deployment-local.txt` |
| Testnet funding obtained unattended | `artifacts/phase-G-faucet.txt` |
| Byte-compatibility with LEZ's own vectors | `crates/membership-core/tests/lez_compat.rs` |
| Derivation cross-checked against a real wallet account | `crates/sdk/examples/wallet_member.rs` |
| CU / proving cost | [cu-costs.md](cu-costs.md) |
| Criteria → evidence map | [criteria-checklist.md](criteria-checklist.md) |

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

- **Nothing is on the public testnet.** The wallet is connected and funded (balance 300, txs in
  blocks 38138/38139/38148), but the programs are not deployed there. P-F6, P-F7, P-S1 and the
  on-chain CU figures for P-P1 are therefore unmet.
- **`demo.sh` has not completed an unattended end-to-end run.** Every step has been demonstrated —
  sequencer, deployment, create, propose, an anonymous approval — but by hand. P-S5 is not claimed.
- **No CI e2e job.** It is deliberately unwired: a present-but-failing job is not evidence, and a red
  default branch is itself a P-S3 failure.
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
