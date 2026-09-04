# LEZ account model: how `nonce` and `program_owner` are handled

The prize's write-up requirement names these two constraints specifically, because they are what make
the existing public multisig PoC inapplicable to shielded accounts. This document states what LEZ
actually does, with citations, and what our design does about it.

All citations are to `logos-execution-zone` at tag **v0.2.4** (the revision the live testnet runs —
see `docs/VERSIONS.md`).

---

## 1. The account record

`lee/state_machine/core/src/account.rs:98`

```rust
pub struct Account {
    pub program_owner: ProgramId,   // [u32; 8]
    pub balance: Balance,           // u128
    pub data: Data,
    pub nonce: Nonce,               // u128
}
```

An account id is a 32-byte value (`account.rs:155`), rendered as base58.

## 2. `program_owner`

`program_owner` records which program is allowed to mutate the account. The public multisig PoC
relies on this: members hand their accounts to the multisig program, which claims them.

**A shielded account cannot be claimed that way.** It is owned by the privacy protocol, and the
privacy-preserving circuit is what authorises changes to it. If the multisig program claimed a
member's account, it would stop being a shielded account.

**What we do instead.** We never touch `program_owner` on member accounts. Members keep their
shielded accounts exactly as they are. The program owns only its own PDAs:

| Account | `program_owner` |
|---------|-----------------|
| Multisig config PDA | the multisig program |
| Proposal PDA | the multisig program |
| Member's shielded account | unchanged — the privacy protocol |

Membership is therefore not an ownership relation at all. It is a statement proven in zero knowledge:
*the `npk` behind this account is a leaf under `member_root`* (ADR-001 D4). That is the substitution
that makes a private multisig possible where the public design cannot go.

Authority over the program's own PDAs during a private execution comes from LEZ's `ChainedCall.pda_seeds`:

```rust
pub struct ChainedCall {
    pub program_id: ProgramId,
    pub pre_states: Vec<AccountWithMetadata>,
    pub instruction_data: InstructionData,
    /// PDA seeds authorized for the callee. For each seed, the callee is authorized to
    /// mutate the `AccountId` derived from `(caller_program_id, seed)`, regardless of
    /// whether the account is public or private.
    pub pda_seeds: Vec<PdaSeed>,
}
```
(`lee/state_machine/core/src/program/mod.rs:202-212`)

## 3. `nonce` — the constraint that rules out fresh zero-nonce keypairs

Public and private accounts advance the nonce by completely different rules.

**Public account** — an ordinary counter (`account.rs:21-26`):

```rust
pub const fn public_account_nonce_increment(&mut self) {
    self.0 = self.0.checked_add(1).expect("Overflow when incrementing nonce");
}
```

**Private account** — derived, never counted (`account.rs:29-47`):

```rust
pub fn private_account_nonce_init(account_id: &AccountId) -> Self {
    // SHA256(account_id ‖ [0u8; 32]), first 16 bytes, little-endian u128
}

pub fn private_account_nonce_increment(self, nsk: &NullifierSecretKey) -> Self {
    // SHA256(nsk ‖ nonce.to_le_bytes() ‖ [0u8; 16]), first 16 bytes, little-endian u128
}
```

Two consequences matter here:

1. **A private account's nonce is never zero by construction.** Its initial value is a hash of the
   account id. The public multisig's requirement of "fresh zero-nonce keypairs" cannot be met.
2. **It advances by a value only the holder can compute**, because it is keyed to `nsk`. Nobody can
   predict or replay a private account's next nonce without the secret.

That second property is doing real work for privacy: because `nonce` is one of the fields inside the
account commitment —

```
Commitment::new = SHA256( "/LEE/v0.3/Commitment/"‖[0;11] ‖ account_id ‖ program_owner ‖ balance ‖ nonce ‖ SHA256(data) )
```
(`lee/state_machine/core/src/commitment.rs`)

— every use of the account produces an unrelated-looking commitment. Successive uses of the *same*
account are unlinkable to an observer.

**What we do.** Nothing special, and that is the point. An approval spends the member's account as a
normal `PrivateAuthorizedUpdate`; LEZ recomputes the nonce with `private_account_nonce_increment` and
emits the new commitment. Our program never reads, writes or asserts anything about a member
account's nonce. Trying to manage it would be both unnecessary and a privacy leak.

The one operational consequence, recorded honestly: **an approval consumes the member's account
state**, so a member needs a live account and cannot produce two approvals from the same account
state. Sequencing is handled client-side by the SDK.

## 4. Identity derivation

```
npk        = SHA256( "LEE/keys" ‖ nsk[32] ‖ [7u8] ‖ [0u8; 23] )
account_id = SHA256( "/LEE/v0.3/AccountId/Private/"‖[0;4] ‖ npk[32] ‖ vpk[1184] ‖ identifier[16 LE] )
```
(`lee/state_machine/core/src/nullifier.rs`; `ViewingPublicKey::LEN = 1184`,
`lee/state_machine/core/src/encryption/shared_key_derivation.rs:28`)

`identifier` is a `u128`, so one `(npk, vpk)` pair controls a family of 2^128 addresses. This is why
our approval nullifier is keyed to `nsk` and not to `account_id`: otherwise a member could vote again
from a different address in their own family (ADR-001 D5).

**Version sensitivity.** This derivation is *not* stable across LEZ versions. At v0.2.0 it was
`SHA256(prefix ‖ npk ‖ identifier)` with no `vpk` — an 80-byte preimage. Building against v0.2.0
would derive addresses this testnet does not recognise. See `docs/VERSIONS.md` for how the correct
pin was established by fingerprinting deployed ImageIDs.

## 5. Live-account membership

LEZ maintains a commitment set over live private accounts, with a Merkle digest:

```rust
pub fn compute_digest_for_path(commitment: &Commitment, proof: &MembershipProof) -> CommitmentSetDigest
```
(`commitment.rs`; `MembershipProof = (usize, Vec<[u8; 32]>)`)

Spending a private account requires producing a path from its commitment to the live root
(`lee/privacy_preserving_circuit/src/output.rs:347-357`). This is the mechanism our design leans on
for **live**-account binding rather than mere key derivation — see ADR-001 D4 and gate H8.

## 6. Known-answer vectors

LEZ pins expected outputs for these derivations in its own test modules — `nullifier.rs`
(`from_secret_key`, `account_id_from_nullifier_public_key`, `constructor_for_account_update`),
`commitment.rs` (`nothing_up_my_sleeve_dummy_commitment`) and `program/tests.rs`
(`for_private_pda_matches_pinned_value`).

Phase B reuses these as regression vectors so that byte-compatibility with LEZ is demonstrated
against upstream's own expected values rather than asserted (**SC-B.7**).
