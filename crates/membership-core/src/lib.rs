//! Types exchanged with the membership program.
//!
//! Split into a public [`ApprovalClaim`] and a secret [`ApprovalWitness`], carried together in the
//! instruction.
//!
//! # Why both travel in `instruction_data`, and what that costs
//!
//! LEZ hands every program exactly four inputs — `program_id`, `caller_program_id`, `pre_states`
//! and `instruction_data` — and there is no fifth
//! (`lee/state_machine/src/program/mod.rs::write_inputs`). A program therefore has **no private
//! channel**: any secret it needs must arrive in `instruction_data`, which LEZ echoes into the
//! `ProgramOutput` it commits to the guest's journal.
//!
//! Phase E established this the hard way. An earlier design read the witness as a separate
//! `env::read()` after the standard inputs, to keep it out of the journal; that works in a bespoke
//! harness and **fails on LEZ**, because nothing in the runtime ever writes a fifth input. The guest
//! aborted with `DeserializeUnexpectedEnd` the first time a real transaction reached it.
//! `docs/tried-failed.md` records the whole arc.
//!
//! So the honest position is: **the inner journal contains the member's `nsk`**, and that journal is
//! prover-local material which never reaches the chain — only `PrivacyPreservingCircuitOutput` does,
//! and it carries just nullifiers, commitments and ciphertext
//! (`lee/state_machine/core/src/circuit_io.rs:156-180`). An inner receipt must be treated like a
//! private key at rest; see `docs/security.md` §3b.
//!
//! The split is kept because it still documents which half is *conceptually* public — the claim is
//! exactly what the multisig program records on chain — even though both now share one channel.

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

/// The **secret** half.
///
/// Travels in `instruction_data` because LEZ offers no other channel (see the module docs), and is
/// therefore present in the guest's inner journal. Never persisted or transmitted outside the
/// proving process.
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

/// What the membership program is asked to verify: the public claim and the secret witness.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct VerifyApprovalArgs {
    pub claim: ApprovalClaim,
    pub witness: ApprovalWitness,
}

/// The instruction enum of the membership program.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Instruction {
    /// Verify a membership claim against its witness.
    VerifyApproval(Box<VerifyApprovalArgs>),
}
