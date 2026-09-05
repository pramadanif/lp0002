# Security model

What this system hides, what it does not, and what it assumes. Written to be checkable rather than
reassuring: each claim names the mechanism that enforces it, and §3 lists what an attacker still gets.

Citations are to `logos-execution-zone` v0.2.4.

---

## 1. Privacy surface — who learns what

The prize requires approval "without revealing their identity to on-chain observers **or other
members**". Those are two different adversaries, so they get two columns.

| Fact | On-chain observer | Co-member | Sequencer | Enforced by |
|------|------------------|-----------|-----------|-------------|
| That a multisig exists, and its `config_hash` | **yes** | yes | yes | Config account is a public PDA (by design — enables public verification) |
| `member_root`, `M`, `N` | **yes** | yes | yes | Stored in the public config account |
| The **member list** (which accounts/npks) | **no** | only members they already knew | **no** | Only the Merkle root is published; leaves are never on chain |
| That a proposal exists, and its action | **yes** | yes | yes | Proposal account is public; hiding proposal content is out of scope |
| How many approvals so far | **yes** | yes | yes | Approval counter — this is exactly what P-F2 requires to be visible |
| **Which member approved** | **no** | **no** | **no** | Witnesses (`nsk`, `vpk`, `identifier`, `merkle_path`) are circuit *inputs*, never journal output — `circuit_io.rs:156-180` |
| The approval nullifier `nf_approve` | yes | yes | yes | Stored to prevent double-voting; preimage-hiding under SHA-256 |
| Link between `nf_approve` and a member | **no** | **no** | **no** | `nf_approve = SHA256(DS_NF ‖ nsk ‖ multisig_id ‖ proposal_id)`; inverting it needs `nsk` |
| Link between two approvals by the same member on **different** proposals | **no** | **no** | **no** | `proposal_id` is in the preimage, so nullifiers across proposals are unrelated |
| That *some* shielded account was spent in the tx | yes | yes | yes | LEZ emits `PrivateAction { nullifier, root, commitment, encrypted_post_state }` for any private account update |
| **Which** shielded account was spent | **no** | **no** | **no** | LEZ private-account unlinkability: the commitment changes on every use because the nsk-derived `nonce` is inside it |
| Who executed the proposal | pseudonymous | pseudonymous | pseudonymous | Execute needs only threshold and can be sent by anyone, including a non-member |

### Why co-members learn nothing extra

There is no member-to-member protocol. A member approving needs: the `multisig_id`, the
`proposal_id`, the `member_root` and their own Merkle path. The first three are public; the path is
derived from the member set they were given at creation. No approval is announced to peers, and no
coordinator sees who signed.

The only thing co-members observe is what everyone observes — the counter going up. This is
**W8/SC-D.5** and is asserted by test in Phase D.

Contrast with a threshold signature scheme: FROST-style signing requires an interactive round among
participants, so the other signers learn who took part. That is why it was rejected (ADR-001 §7).

## 2. Unlinkability — the precise claim

> **P-F4:** a completed execution is unlinkable to any individual member's shielded account.

Precisely: given the full on-chain transcript of a completed multisig — config account, proposal
account with its nullifier set, every transaction — and given the *entire* member list, an adversary
cannot do better than guessing which members approved.

Holds because the transcript's only member-derived values are `nf_approve` values, and
`nf_approve = SHA256(DS_NF ‖ nsk ‖ multisig_id ‖ proposal_id)`. Distinguishing which member produced
one means finding a preimage under SHA-256 or guessing `nsk`.

**Stated honestly, this is where it stops:** unlinkability is *cryptographic* within the transcript.
It is not anonymity against an adversary with network-level or timing observation — see §3.

## 3. What an attacker still learns

Listing these is the point of the section; `docs/limitations.md` carries the full set.

1. **Approval count and timing.** Each approval is a transaction at a point in time. An adversary who
   watches the chain learns how many approvals landed and when. With a small `N` and members in known
   time zones, timing is a real correlation channel. Nothing here defends against it.
2. **Network-level identity.** Submitting a transaction reveals an IP to whoever accepts it. Out of
   scope for this prize; use a transport that anonymises if it matters.
3. **Membership if the set is published elsewhere.** We publish only `member_root`. If the operator
   announces the member list, the anonymity set is whatever remains — at 2-of-3, small by definition.
   The scheme gives anonymity *within* the member set; it cannot make that set larger.
4. **Small anonymity sets.** With N=3 and 2 approvals, an adversary knows 2 of 3 members approved.
   That is inherent to threshold visibility, not a defect.
5. **Proposal content.** Explicitly out of scope — the prize hides identity and vote, not the action.

## 3b. The inner receipt is secret material

Approvals are proved in two layers: our membership guest produces a `ProgramOutput`, and LEZ's
privacy-preserving circuit verifies it with `env::verify` and commits only
`PrivacyPreservingCircuitOutput`. **Only the outer journal reaches the chain**, and it carries just
nullifiers, commitments, ciphertext and public-account states (`circuit_io.rs:156-180`).

The inner `ProgramOutput` is different. Every LEZ program commits its `pre_states`, which is how the
runtime validates execution — so the inner journal contains **the approver's `account_id` in the
clear**. That is inherent to LEZ, not to this design.

Two consequences, both enforced rather than assumed:

1. **The member's secrets are kept out of the inner journal.** The guest's instruction carries only
   the public `ApprovalClaim`; `nsk`, `vpk`, `identifier` and the Merkle path arrive as a separate
   private input that is never committed. This was a bug once — the witness was in `instruction_data`
   and the `nsk` was fully recoverable from the journal (`docs/tried-failed.md`). The test
   `the_journal_carries_no_member_secret` decodes the journal and asserts it.
2. **Inner receipts are never persisted or transmitted.** They identify the approver's account. The
   SDK holds them only in memory, for exactly as long as it takes to build the outer proof.

An adversary who obtains an inner receipt learns which account approved — not the member's key, but
enough to break anonymity for that approval. Treat it like a private key at rest.

## 4. Trust and cryptographic assumptions

| Assumption | Rests on |
|------------|----------|
| SHA-256 is collision- and preimage-resistant | Nullifier unlinkability, Merkle soundness, PDA unforgeability |
| Risc0 zkVM proofs are sound | Every claim proven in-circuit; a soundness break forges approvals |
| Risc0 proofs are zero-knowledge | Witnesses stay secret |
| LEZ's PPE circuit is correct | Live-account binding (ADR-001 D4) — we verify our composition, not LEZ's internals |
| The sequencer checks commitment roots and rejects re-used nullifiers | Liveness binding and no double-spend |
| Members keep `nsk` secret | Anything else is impersonation |
| **No trusted setup** | Risc0 is transparent — there is none, and we claim none |

`RISC0_DEV_MODE=1` produces **fake receipts that prove nothing**. Every submission-path script runs
with `RISC0_DEV_MODE=0`, and `scripts/check-dev-mode-clobber.sh` fails the build if any script on that
path sets it to 1 (gate H3, the pattern that sank prize PR #97).

## 5. Attacks considered

| Attack | Outcome | Mechanism |
|--------|---------|-----------|
| Approve twice on one proposal | rejected | Same `nf_approve`; already in the set (INV-4) |
| Approve twice from two addresses of the same member | rejected | Nullifier keyed to `nsk`, not `account_id` (ADR-001 D5) |
| Lower `M` to 1 | fails — no such multisig | `M` is inside `config_hash`, which is the PDA seed (INV-1) |
| Substitute a member set containing yourself | fails — different address | `member_root` inside `config_hash` (INV-2) |
| Create a config account at a valid address but store other values | rejected | Program rehashes stored fields against the seed (INV-3) |
| Prove membership for an account you do not control | rejected | Guest re-derives `account_id` from witness `nsk` and matches the PPE-bound pre-state (D4) |
| Prove membership for a member's account you do not control | rejected | PPE requires `nsk` to derive the account id it binds (`output.rs:91-94`) |
| Replay an approval from a spent account | rejected | Commitment no longer in the live set; LEZ nullifier already used |
| Execute below threshold | rejected | `approvals >= M` against INV-3-validated config (INV-6) |
| Execute twice | rejected | `executed` flag |
| **Redirect an approved transfer to another account** | rejected | `execute` compares the recipient account against the one the proposal named and refuses otherwise (**INV-7**, error 1012 / `7012` on chain). The submitter chooses the accounts in the transaction; the approvals cover only the proposal, so the two must be tied together explicitly. This was a real hole — `execute` destructured the approved action as `{ amount, .. }` and discarded the recipient — found by running the program through the executor, and it is now a regression test |
| **Pay out of an account the multisig does not control** | unsupported | There is no caller-supplied treasury account. Funds leave the multisig's own config PDA, so there is nothing to point elsewhere (**INV-7**) |
| Approve on a public (re-executed) path to dodge the proof | unsupported | No public approve path exists; tested absent (SC-C.8) |
| Non-member submits an approval | rejected | Merkle path to `member_root` cannot be produced |
| Removed member approves under the old root | fails — different address | Config change means a new `config_hash` (INV-5) |

## 6. Out of scope

Membership rotation, proposal-content privacy, defence against timing/network correlation, formal
verification, and a security audit. All are recorded in `docs/limitations.md` rather than implied to
be handled.
