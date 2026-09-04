//! Types exchanged with the membership program.
//!
//! Split deliberately into two halves, because they have **different privacy fates**:
//!
//! - [`ApprovalClaim`] is the program's `instruction_data`. LEZ programs echo `instruction_data`
//!   into their `ProgramOutput`, which is **committed to the guest's journal**, so nothing secret
//!   may live here. Everything in the claim is already public on chain.
//! - [`ApprovalWitness`] is read as an extra private input and is **never committed**. This is where
//!   the member's secrets go.
//!
//! Phase B measured this rather than assuming it. An earlier version carried the whole witness in
//! `instruction_data`; decoding the journal showed the member's `nsk` could be read straight back
//! out (`docs/tried-failed.md`). On-chain privacy would still have held — only
//! `PrivacyPreservingCircuitOutput` reaches the chain — but the inner receipt would have been a
//! spending key in a file, which is a far worse failure than an identity leak.

use borsh::{BorshDeserialize, BorshSerialize};
use lee_core::encryption::ViewingPublicKey;
use pmsig_core::Digest32;
use serde::{Deserialize, Serialize};

pub mod verify;
pub use verify::verify_approval;

/// Length of a LEZ `ViewingPublicKey`, from `ViewingPublicKey::LEN`
/// (`lee/state_machine/core/src/encryption/shared_key_derivation.rs:28`). ML-KEM-768 sized.
pub const VIEWING_PUBLIC_KEY_LEN: usize = 1184;

/// The **public** half of an approval: what the multisig program records on chain anyway.
///
/// Safe to commit. The caller passes exactly this as the chained call's `instruction_data`, so
/// LEZ's check that the call and the output agree still holds.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ApprovalClaim {
    /// The multisig this approval belongs to.
    pub multisig_id: Digest32,
    /// The proposal being approved.
    pub proposal_id: Digest32,
    /// The member root the approval is claimed against. The multisig program checks this equals the
    /// root in its config account (error 1007).
    pub member_root: Digest32,
    /// The nullifier to be recorded in the proposal's set. The guest recomputes it from the witness
    /// and rejects a mismatch, so a caller cannot write an arbitrary value.
    pub claimed_nullifier: Digest32,
}

/// The **secret** half: never committed, never persisted, never transmitted.
///
/// Read by the guest as a private input after the standard LEZ inputs.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ApprovalWitness {
    /// The member's nullifier secret key.
    pub nsk: Digest32,
    /// The member's viewing public key, needed to re-derive their account id.
    ///
    /// LEZ's own type: it derives Borsh and Serde without the `host` feature, so the guest can
    /// deserialise it. `ViewingPublicKey::from_bytes` is host-only.
    pub vpk: ViewingPublicKey,
    /// Which address in the member's 2^128 family is being used.
    pub identifier: u128,
    /// The member's leaf index in the member tree.
    pub member_index: u64,
    /// Sibling hashes from leaf to root. Secret: they would otherwise narrow the member's position.
    pub siblings: Vec<Digest32>,
}

/// The instruction enum of the membership program.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Instruction {
    /// Verify a membership claim against the private witness supplied alongside it.
    VerifyApproval(ApprovalClaim),
}
