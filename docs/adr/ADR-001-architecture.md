# ADR-001 — Architecture of a private M-of-N multisig on LEZ

**Status:** Accepted (Phase A, 2026-09-04)
**Supersedes:** nothing · **Superseded by:** nothing
**Changing any decision here requires ADR-002 written *before* the code changes.**

Every LEZ behaviour cited below was read from `logos-execution-zone` at tag **v0.2.4** — the revision
the live testnet runs, established by ImageID fingerprint in `docs/VERSIONS.md`. File and line
references are to that tag.

---

## 1. Context

A public multisig already exists for LEZ ([lez-multisig](https://github.com/jimmy-claw/lez-multisig)).
It cannot be adapted to shielded members, for two structural reasons:

1. **Ownership.** Its member accounts must be claimed by the multisig program. A shielded account's
   `program_owner` belongs to the privacy protocol; the multisig program cannot claim it.
2. **Nonce.** It needs fresh zero-nonce keypairs. A private account's nonce is not a counter that can
   be held at zero — LEZ *derives* it from the account id, then re-derives it from the member's
   nullifier secret key on every use (`lee/state_machine/core/src/account.rs:29-47`).

So membership cannot be established by ownership. It has to be proven in zero knowledge: *I control
an account that belongs to the member set* — without revealing which account.

## 2. Decisions

### D1 — Approvals travel the privacy-preserving execution path, never public re-execution

An approval is submitted as a **private** LEZ transaction: the member executes the program locally
and proves it, and the sequencer verifies the proof instead of re-executing.

LEZ implements this as a wrapper circuit, `lee/privacy_preserving_circuit`. It reads
`PrivacyPreservingCircuitInput { program_outputs, account_identities, program_id, dummy_inputs }`,
walks the chained-call queue and, for each program output, calls

```rust
env::verify(chained_call.program_id, program_output_words)
```

(`lee/privacy_preserving_circuit/src/execution_state.rs:151-155`), then asserts that the output's
`self_program_id` and `caller_program_id` match the call it was expected to answer.

**Rejected:** a "public path" approve where the sequencer re-executes the instruction. Re-execution
requires the inputs in the clear, which is precisely the identity we must not reveal. A public path
that merely *looks* zero-knowledge — proof material attached to a transaction that is still executed
publicly — is the pattern reviewers rejected in prize PR #131 ("execute tx contains no proof").
There is therefore **no** public approve path in this design, and one is explicitly tested for
absence (SC-C.8).

### D2 — Membership is a separate LEZ-native program, invoked as a chained call

Membership and nullifier derivation live in their own program (`programs/membership-lez`), which
emits a normal LEZ `ProgramOutput` and is invoked by the multisig program through a `ChainedCall`.
The PPE circuit's `env::verify` loop proves it ran.

**Rejected:** a standalone Risc0 guest whose receipt is passed as opaque bytes. Its journal would not
be a LEZ `ProgramOutput`, so the PPE circuit could not verify it as part of the execution chain, and
the "proof" would be an unverified attachment rather than part of the transaction's validity.

### D3 — The member set and the threshold are anchored in the PDA seed

```
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] )
```

and the multisig's config account lives at `AccountId::for_public_pda(program_id, PdaSeed(config_hash))`
(`lee/state_machine/core/src/program/mod.rs:127`). A `PdaSeed` is exactly 32 bytes
(`program/mod.rs:44`), so a SHA-256 digest drops straight in with no truncation.

This is the invariant that defeats the two obvious forgeries — see §4.

### D4 — The proof is bound in-circuit to a **live** shielded account, not merely to a derived key

This is the decision that separates a real submission from the derivation-only stub rejected in
prize PR #91, so it is spelled out in full.

For a private account being spent, the PPE circuit is given
`InputAccountIdentity::PrivateAuthorizedUpdate { vpk, nsk, membership_proof, identifier, … }` and it:

1. derives `npk = NullifierPublicKey::from(nsk)` — so the prover must know `nsk`;
2. derives `account_id = AccountId::for_regular_private_account(&npk, vpk, identifier)`;
3. **asserts `account_id == pre_state.account_id`** (`output.rs:94`) — the claimed identity is the
   account the program actually ran against;
4. computes `commitment_pre = Commitment::new(account_id, pre_account)` from that pre-state and
   `set_digest = compute_digest_for_path(&commitment_pre, membership_proof)` (`output.rs:347-357`).

The sequencer accepts the transaction only if `set_digest` equals the live on-chain commitment-set
root, and rejects a re-used LEZ nullifier. **A prover cannot fabricate that root**: the account must
genuinely exist, unspent, in the live commitment set.

Our membership guest then closes the loop. It receives, as private witnesses, `(nsk, vpk, identifier,
merkle_path)` and:

- recomputes `npk` from `nsk`;
- verifies `npk` is a leaf under the multisig's `member_root`;
- **re-derives `AccountId::for_regular_private_account(npk, vpk, identifier)` and asserts it equals
  the approver's `pre_state.account_id`** — the same account the PPE circuit bound to a live
  commitment;
- emits the approval nullifier.

Composed, these say: *the approver knows the `nsk` of a member of this multisig, and that member's
account is live and unspent on chain right now.*

**Why the assertion is not redundant.** Without it, the guest proves only *"someone knows an `nsk`
whose `npk` is in the member set"*. That is a statement about key material, not about the chain — it
is true of a member who never created a shielded account, or whose account has been fully spent, and
it stays true forever once the key exists. That is precisely the derivation-only property rejected in
prize PR #91.

With the assertion, the approval is pinned to a **specific live account in this transaction**: the one
LEZ's circuit proved is in the current commitment set and is being spent now. The approver must have
standing on chain at the moment of approval, not merely once have held a key.

To be precise about what the assertion does *not* do: it is not what prevents double-voting. That is
INV-4, and it holds through `nf_approve` being a deterministic function of `nsk` — a member cannot mint
a second nullifier by substituting a different `nsk`, because a different `nsk` would fail the
membership check in the first place. What removing the assertion would allow is an approval whose
on-chain footprint is an account **unrelated to the member** — the transaction spends some account,
the witness names a member key, and nothing ties the two together. **SC-B.5** demonstrates exactly
that: a witness that does not control the presented account is accepted by a derivation-only variant
and rejected by this one.

### D5 — Approval nullifiers are keyed to the member secret, the multisig and the proposal

```
nf_approve = SHA256( DS_NF ‖ nsk ‖ multisig_id ‖ proposal_id )
```

Deterministic per `(member, multisig, proposal)`, so a second approval of the same proposal produces
the same nullifier and is rejected. Preimage-hiding, so the stored value says nothing about which
member approved. Keyed to `nsk` rather than to an account id, so a member cannot vote twice by moving
to another of their addresses — one `npk` (hence one `nsk`) controls a family of 2^128 addresses
(`program/mod.rs:151-153`), and all of them yield the same approval nullifier.

This is **our** nullifier, in **our** domain (`/LP0002/…`), and is distinct from LEZ's own
account-update nullifier (`/LEE/v0.3/Nullifier/Update/`). Both exist per approval and serve different
purposes: LEZ's prevents double-spending the account; ours prevents double-voting the proposal.

### D6 — Co-members learn the count, never the voter

Nothing in the approve flow requires a member to tell other members who they are. The shared state is
the on-chain proposal account: an approval count and a set of nullifiers. Full table in
`docs/security.md`; asserted as a test in Phase D (**SC-D.5**).

### D7 — Config and proposal accounts are public; witnesses are private

The multisig config and proposal accounts are **public** PDAs, so anyone can verify that a threshold
was reached. This is deliberate and it is safe, because of what the PPE circuit actually publishes.
Its journal is only:

```rust
PrivacyPreservingCircuitOutput { public_actions, private_actions,
                                 block_validity_window, timestamp_validity_window }
```

where `PrivateAction` is `{ nullifier, root, commitment, encrypted_post_state }`
(`lee/state_machine/core/src/circuit_io.rs:156-180`). The inner `program_outputs`, their
`instruction_data`, and `account_identities` are circuit **inputs** — verified, never committed. So
the membership witnesses (`nsk`, `vpk`, `identifier`, `merkle_path`) do not appear on chain.

What *is* public: the proposal's approval count, the approval nullifier set, and the proposed action.
Hiding the proposal's content is explicitly out of scope for this prize.

### D8 — Reference action: treasury transfer, default 2-of-3, evidenced at full M

The reference integration is a treasury transfer gated on M approvals. Default configuration is
M=2, N=3. Primary testnet evidence uses the **full** M approvals — not a lowered threshold tier.

## 3. Canonical formulas

Domain separators are ASCII, zero-padded to 32 bytes, mirroring LEZ's own convention
(`b"/LEE/v0.3/Commitment/\x00…"`). Byte lengths below are exact and were checked, not estimated.

```text
DS_CONFIG = "/LP0002/v1/ConfigHash/"        ++ [0u8; 10]   // 22 + 10 = 32
DS_NF     = "/LP0002/v1/Nullifier/Approve/" ++ [0u8;  3]   // 29 +  3 = 32
DS_LEAF   = "/LP0002/v1/MemberLeaf/"        ++ [0u8; 10]   // 22 + 10 = 32
DS_PROP   = "/LP0002/v1/ProposalSeed/"      ++ [0u8;  8]   // 24 +  8 = 32
```

**This is the one and only definition of `config_hash` in this repository.** README and the solution
write-up quote this line verbatim; preflight check PF-13 fails the build if they ever drift (H14/W17).

```text
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] )
```

```text
nf_approve  = SHA256( DS_NF ‖ nsk[32] ‖ multisig_id[32] ‖ proposal_id[32] )

member_leaf = SHA256( DS_LEAF ‖ npk[32] )

member_node = SHA256( left[32] ‖ right[32] )      // left/right chosen by index bit, LSB first

proposal_pda_seed = SHA256( DS_PROP ‖ config_hash[32] ‖ proposal_id[32] )
```

Sizes: `M` and `N` are single bytes (`u8`); `member_root`, `multisig_id`, `proposal_id`, `npk`, `nsk`
are 32 bytes each. All multi-byte integers elsewhere are little-endian, matching LEZ.

`member_node` deliberately mirrors LEZ's `compute_digest_for_path` (`commitment.rs`): the leaf is
hashed, then combined pairwise with `SHA256(left ‖ right)`, selecting sides by the index bit and
shifting right at each level.

## 4. Invariants

**INV-1 — a prover cannot lower the threshold.** `M` is inside `config_hash`, and `config_hash` is the
PDA seed. A prover who claims `M' = 1` computes `config_hash' ≠ config_hash`, which derives a
*different* account address. No multisig exists there, so there is nothing to approve. Lowering the
threshold does not weaken a multisig; it names one that does not exist.

**INV-2 — a prover cannot substitute a member set.** Identical argument for `member_root`: inventing a
set containing yourself changes `config_hash` and therefore the address.

**INV-3 — the config account must match its own address.** The program recomputes `config_hash` from
the account's stored `(member_root, M, N, multisig_id)` and asserts it equals the seed the account was
found under. This closes the gap where a config account is created at a correct address but stores
different values.

**INV-4 — one approval per member per proposal.** `nf_approve` is deterministic in `(nsk, multisig_id,
proposal_id)`; the proposal account rejects a nullifier already in its set.

**INV-5 — approvals bind to live accounts.** Per D4, an approval is only valid if the approver's
account commitment is in the live commitment set. A member removed by a config change cannot approve
under the old root, because the old root yields a different PDA.

**INV-6 — execution requires the threshold.** `execute` asserts `approvals >= M` where `M` is read
from the config account validated by INV-3, and sets an `executed` flag checked on entry.

## 5. Account model

| Account | Kind | Address | Holds |
|---------|------|---------|-------|
| Multisig config | public PDA | `for_public_pda(program_id, config_hash)` | `member_root`, `M`, `N`, `multisig_id`, treasury id, proposal counter |
| Proposal | public PDA | `for_public_pda(program_id, proposal_pda_seed)` | `config_hash`, `proposal_id`, action, approval count, **nullifier set**, `executed` |
| Member account | shielded, regular private | `for_regular_private_account(npk, vpk, identifier)` | the member's own funds/state; touched as `PrivateAuthorizedUpdate` when approving |
| Treasury | public or shielded | per configuration | the assets the reference transfer moves |

The proposal account stores **nullifiers, never member identities** (SC-C.6, P-F2).

## 6. Consequences

**Good.** Approvals reveal no identity to observers or co-members. Double-voting is prevented without
a voter list. The threshold cannot be forged, because forging it changes the address. Verification
needs only public data, so anyone can audit a completed multisig.

**Costs, accepted.** Every approval spends and recreates the member's shielded account, so a member
must hold a live account and cannot approve twice in the same block from the same account. Proving
happens client-side and is the dominant latency. The member set is fixed at creation — changing it
means a new `config_hash`, hence a new multisig address; membership rotation is out of scope and is
recorded in `docs/limitations.md`.

**Unresolved, tracked.** Exact `PrivacyPreservingCircuitInput` construction from the client
(`docs/VERSIONS.md` U-2) and whether SPEL's CLI can submit the private path or only the public one
(U-6). Both are settled in Phases B–C by building against LEZ v0.2.4, not by assumption.

## 7. Alternatives rejected

| Alternative | Why rejected |
|-------------|--------------|
| Public multisig with encrypted member labels | Observers still link approvals to accounts; fails P-F1 |
| Threshold signature (FROST) over a shared key | Needs an interactive DKG among members and a coordinator; members would learn who signed, failing the "or other members" half of P-F1 |
| Semaphore-style external nullifier with an off-chain member list | Membership would not bind to a live LEZ account — the derivation-only pattern rejected in #91 |
| Standalone Risc0 receipt attached to a public tx | Journal is not a LEZ `ProgramOutput`; unverifiable in the execution chain (D2) |
| Member set stored on-chain as a list | Publishes the membership; also grows CU cost linearly |
| Nullifier keyed to `account_id` instead of `nsk` | A member could vote again from another address in their 2^128 family |
| Storing an approver bitmap for convenience | Directly records who approved; fails P-F2 |

Running log of things attempted and abandoned during implementation: `docs/tried-failed.md`.
