# Known limitations

Written to be useful to someone deciding whether to rely on this, which means it is specific rather
than reassuring. Everything here is a real constraint of the current build, not a hypothetical.

Plan gate **H12/W16** requires this file to exist and resolve at the pinned commit — one prior
submission to this prize linked a `limitations.md` that 404'd. It is also the file preflight check
PF-06 refuses to submit without.

---

## 1. Not audited, not production

No third-party security review. The cryptographic construction is standard in shape — Merkle
membership plus a domain-separated nullifier, composed under LEZ's privacy-preserving circuit — but
"standard in shape" is not "audited". Do not hold real value with this.

## 2. The member set is fixed at creation

`member_root` is inside `config_hash`, which seeds the multisig's PDA. Changing the member set
therefore changes the address: it does not modify a multisig, it names a different one. There is no
add-member or remove-member instruction, and adding one would require a design that lets the address
survive a membership change.

**Consequence:** a compromised member cannot be evicted. The remedy is to create a new multisig and
move the funds — which is a governance action the old multisig can itself approve.

## 3. Changing the verifier rotates every address

[ADR-002](adr/ADR-002-bind-verifier-to-config-hash.md) binds the membership program's id into
`config_hash`, so that an attacker cannot name a permissive verifier. The cost is that a new build of
the membership program has a new ImageID, and therefore changes the `config_hash` — and the
address — of every multisig.

Existing multisigs keep working against the verifier they were created with. New ones must be created
to adopt a new verifier. This was a deliberate trade: the alternative, storing the verifier id in the
account, is trust-on-first-use.

## 4. Anonymity is bounded by the member set

The scheme hides *which* member approved, within the member set. It cannot make that set larger. At
the default 2-of-3, an observer who knows the membership knows that two of three approved. That is
inherent to threshold visibility, not a defect — but it means the privacy claim is "unlinkable within
N", not "anonymous".

If the operator publishes the member list, the anonymity set is whatever remains.

## 5. No defence against timing or network correlation

Each approval is a transaction at a point in time. An adversary watching the chain learns how many
approvals landed and when; one watching the network learns which IP submitted them. With a small `N`
and members in known time zones, timing is a real correlation channel.

Nothing in this design addresses either. Use an anonymising transport if it matters.

## 6. Proposal content is public

By design and by scope: the prize hides member identity and vote, not the proposed action. Anyone can
read what a proposal would do.

## 7. Inner receipts are secret material

An approval is proved in two layers. Only the outer `PrivacyPreservingCircuitOutput` reaches the
chain, and it carries just nullifiers, commitments and ciphertext. The **inner** `ProgramOutput`
contains the approver's `account_id`, because every LEZ program commits its pre-states — that is how
the runtime validates execution, and it is not removable.

The SDK therefore never persists or transmits inner receipts, and the member's key material is kept
out of them entirely (a separate private input, not `instruction_data`). We shipped that bug once and
caught it by decoding a journal — see [tried-failed.md](tried-failed.md). **Treat an inner receipt
like a private key at rest**: anyone holding one learns which account approved.

## 8. An approval spends the member's shielded account

LEZ private accounts advance their nonce on every use, so approving consumes and recreates the
member's account state. A member therefore needs a live shielded account, and cannot produce two
approvals from the same account state. The SDK sequences this; it is a real constraint on a member
who wants to approve several proposals at once.

## 9. Dependency pins that are not releases

- **SPEL** is pinned to `main` at commit `5126b7ed8a9b`, **not** the v0.6.0 release. The release pins
  LEZ v0.2.0, whose `AccountId::for_regular_private_account` omits the viewing key and therefore
  derives addresses the live testnet does not recognise. `main` pins v0.2.4. Depending on an
  unreleased commit is a real cost, taken deliberately; see [VERSIONS.md](VERSIONS.md).
- **LEZ** is pinned to `v0.2.4`, established by fingerprinting the testnet's deployed ImageIDs rather
  than by assumption.

## 10a. The composed approval proof does not complete on this laptop

The **standalone** membership proof completes in 53.26 s. The **composed** approval — LEZ's
privacy-preserving circuit running `env::verify` over two chained programs — did not complete on an
8-core laptop. It was stopped with r0vm at **~4.4 GB resident** and the system **swapping 7.8 GB of
9.2 GB**; wall-clock time was dominated by paging, not proving.

Composition needs *succinct* receipts, which means a lift+join for every segment of every inner
program. That is a different order of cost from the composite receipt the standalone proof produces.

**This matters for criterion P-F5** ("proof generation runs client-side on a standard laptop"). The
honest position: it holds for the standalone membership proof, and is **unverified for the composed
approval** on hardware of this size. Finishing it needs materially more RAM or a GPU prover. This is
a hardware constraint rather than a design fault, but the claim is not made until it is measured.

Evidence: `artifacts/phase-E-ppe-approve-attempt.txt`.

## 10. Measurement caveats

- **Proving time (53.26 s)** is a single sample, on one 8-core laptop with no GPU. It is not a
  distribution and not a benchmark suite.
- **A second proof in the same process** did not complete within 25 minutes on two occasions, while
  the first took 53 s. Undiagnosed. It does not affect the recorded figure, which is the first proof
  of a fresh process — what a member actually experiences — but it is unexplained and is recorded
  rather than smoothed over.

## 11. Build reproducibility

`artifacts/IMAGE_IDS.md` currently records a **local** build and says so in the file. A deployed or
quoted binary must come from `./scripts/build-guests.sh --docker`, which builds inside the pinned
container LEZ itself uses. Until that has been run, the recorded ImageID is a development value.

## 12. The CLI does not yet reach a sequencer

`pmsig` runs against a local state file and prints `[local]` on every such command. It applies the
real transition rules, but it is not a chain and no CLI output is testnet evidence.

Its `create` also takes every member's secret key, so that one machine can act as several members in
a demo. A real deployment never does this: each member derives their own npk, shares only that, and
keeps their own authentication path. The on-chain state has no member list at all.

## 13. What is not yet demonstrated

Stated plainly, because the difference between "designed" and "demonstrated" is the whole point of
this prize's evidence gates:

- **A completed privacy-preserving approval.** The composition is wired end to end and a real
  transaction reaches the prover — the CLI selects the private path, resolves the membership program
  as a dependency, and builds a well-formed circuit input. Nothing rejects it. But the proof did not
  finish on this hardware (§10a), so **no approval has been recorded on chain**. The composition is
  wired, not demonstrated.
- **Anything on the public testnet.** Programs are deployed to a *local* standalone sequencer only,
  and `create_multisig` / `create_proposal` have executed there. There are no public testnet
  transactions and no explorer links. `docs/DEPLOYMENT.md` and the on-chain CU figures in
  [cu-costs.md](cu-costs.md) do not exist yet.
