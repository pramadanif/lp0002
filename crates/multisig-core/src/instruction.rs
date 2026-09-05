//! The program's instruction enum.
//!
//! Defined here rather than generated inside the guest, and wired in with
//! `#[lez_program(instruction = "…")]`. Two reasons, both practical:
//!
//! 1. **The generated enum is private**, so nothing outside the guest crate can construct an
//!    instruction — which means the SPEL program itself could never be exercised by a test. Every
//!    test we had ran the *rules* (`logic::*`) and none ran the *program*: account ordering, PDA
//!    derivation, encoding and the `ChainedCall` were untested.
//! 2. Host-side callers (SDK, CLI) need to build instructions anyway, and should not re-encode by
//!    hand against a format they cannot see.
//!
//! The variant names and field names must match the handler signatures in the program exactly — the
//! macro dispatches on them. `tests/instruction_matches_idl.rs` asserts that against the generated
//! IDL, so a rename cannot silently break dispatch.

use pmsig_core::Digest32;
use serde::{Deserialize, Serialize};

use crate::ProgramIdWords;

/// One call into the multisig program.
///
/// Field order follows the handler signatures; the macro matches on names, but keeping the order
/// aligned makes the correspondence readable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    /// Create a multisig at the PDA its own configuration hashes to.
    CreateMultisig {
        config_hash: Digest32,
        member_root: Digest32,
        m: u8,
        n: u8,
        multisig_id: Digest32,
        membership_program_id: ProgramIdWords,
    },
    /// Submit a proposal. Its content is public by design.
    CreateProposal {
        config_hash: Digest32,
        proposal_seed: Digest32,
        proposal_id: Digest32,
        recipient: Digest32,
        amount: u128,
    },
    /// Record one anonymous approval, gated on the chained membership proof.
    Approve {
        config_hash: Digest32,
        proposal_seed: Digest32,
        member_root: Digest32,
        claimed_nullifier: Digest32,
        witness: Vec<u8>,
    },
    /// Execute a proposal that has reached its threshold.
    Execute {
        config_hash: Digest32,
        proposal_seed: Digest32,
    },
}

impl Instruction {
    /// The instruction's name as it appears in the IDL.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CreateMultisig { .. } => "create_multisig",
            Self::CreateProposal { .. } => "create_proposal",
            Self::Approve { .. } => "approve",
            Self::Execute { .. } => "execute",
        }
    }

    /// Whether this instruction may only travel the privacy-preserving path.
    ///
    /// `approve` carries a member's witness and is meaningless without the chained membership proof
    /// that LEZ's circuit verifies. The others are public by design (ADR-001 D7).
    #[must_use]
    pub const fn is_privacy_preserving(&self) -> bool {
        matches!(self, Self::Approve { .. })
    }
}
