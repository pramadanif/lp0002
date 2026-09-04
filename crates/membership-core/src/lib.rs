//! The instruction the membership program receives.
//!
//! Shared between the guest (which verifies it) and host-side callers (which build it), so the two
//! can never drift out of encoding agreement.
//!
//! **Everything in [`ApproveWitness`] is secret.** It is safe to put it in instruction data only
//! because approvals travel LEZ's privacy-preserving path: the PPE circuit commits
//! `PrivacyPreservingCircuitOutput`, whose `PrivateAction` carries just
//! `{ nullifier, root, commitment, encrypted_post_state }`. The inner program outputs and their
//! instruction data are circuit *inputs* — verified by `env::verify`, never committed
//! (`lee/state_machine/core/src/circuit_io.rs:156-180`). See ADR-001 D7.

use borsh::{BorshDeserialize, BorshSerialize};
use lee_core::encryption::ViewingPublicKey;
use pmsig_core::Digest32;
use serde::{Deserialize, Serialize};

pub mod verify;
pub use verify::verify_approval;

/// Length of a LEZ `ViewingPublicKey`, from `ViewingPublicKey::LEN`
/// (`lee/state_machine/core/src/encryption/shared_key_derivation.rs:28`). ML-KEM-768 sized.
pub const VIEWING_PUBLIC_KEY_LEN: usize = 1184;

/// Witness proving "I am a member of this multisig and this is my approval nullifier".
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ApproveWitness {
    /// The member's nullifier secret key. Never leaves the proof.
    pub nsk: Digest32,
    /// The member's viewing public key, needed to re-derive their account id.
    ///
    /// Carried as LEZ's own type rather than loose bytes: it derives Borsh and Serde without the
    /// `host` feature, so the guest gets it back by deserialisation. `ViewingPublicKey::from_bytes`
    /// is host-only and cannot be used inside the guest.
    pub vpk: ViewingPublicKey,
    /// Which address in the member's `2^128` family is being used.
    pub identifier: u128,
    /// The member's leaf index in the member tree.
    pub member_index: u64,
    /// Sibling hashes from leaf to root.
    pub siblings: Vec<Digest32>,
    /// The multisig this approval belongs to.
    pub multisig_id: Digest32,
    /// The proposal being approved.
    pub proposal_id: Digest32,
    /// The member root the approval is claimed against. The caller checks this equals the root
    /// stored in the multisig's config account (error 1007).
    pub member_root: Digest32,
    /// The nullifier the caller intends to record. The guest recomputes it and rejects a mismatch,
    /// so the caller cannot write an arbitrary value into the proposal's nullifier set.
    pub claimed_nullifier: Digest32,
}

/// The instruction enum of the membership program.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Instruction {
    /// Verify a membership claim and its approval nullifier.
    VerifyApproval(Box<ApproveWitness>),
}
