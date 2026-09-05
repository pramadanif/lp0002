//! On-chain state, instructions and error codes for the private M-of-N multisig.
//!
//! Shared by the program (which enforces this) and host-side callers (which build against it).
//!
//! # What the state deliberately does not contain
//!
//! There is **no voter list, no approver bitmap and no member roster** anywhere in these types.
//! A proposal records a count and a set of nullifiers, and nothing that can be traced back to a
//! member (criterion **P-F2**, plan SC-C.6). The temptation to store "who approved" for a nicer UI
//! is exactly what this design exists to refuse.

use borsh::{BorshDeserialize, BorshSerialize};
use pmsig_core::{Digest32, MemberCount, Threshold, STATE_VERSION};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod instruction;
pub mod logic;
pub use error::MultisigError;
pub use instruction::Instruction;

/// A LEZ program id: `[u32; 8]`.
pub type ProgramIdWords = [u32; 8];

/// The multisig's configuration account.
///
/// Lives at `for_public_pda(program_id, PdaSeed(config_hash))`. Public on purpose: anyone must be
/// able to verify that a threshold was reached. It holds the *parameters*, never the membership.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MultisigConfig {
    /// State layout version.
    pub version: u16,
    /// Merkle root over member nullifier public keys. The leaves are never published.
    pub member_root: Digest32,
    /// Threshold `M`.
    pub m: Threshold,
    /// Member count `N`.
    pub n: MemberCount,
    /// Caller-chosen identifier, distinguishing multisigs with identical membership.
    pub multisig_id: Digest32,
    /// The membership verifier this multisig is bound to (ADR-002).
    pub membership_program_id: ProgramIdWords,
    /// Number of proposals created so far. Advisory only; proposal addresses come from ids.
    pub proposal_count: u64,
}

impl MultisigConfig {
    /// Recomputes the `config_hash` this account's stored fields imply.
    ///
    /// **ADR-001 INV-3.** The program compares this against the PDA seed the account was found
    /// under, so a config account created at a valid address but holding different values is
    /// rejected (`ConfigHashMismatch`, error 1003).
    #[must_use]
    pub fn recompute_config_hash(&self) -> Digest32 {
        pmsig_core::config_hash(
            &self.member_root,
            self.m,
            self.n,
            &self.multisig_id,
            &self.membership_program_id,
        )
    }

    /// Whether the configuration is self-consistent and usable.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.version == STATE_VERSION && self.m > 0 && self.n > 0 && self.m <= self.n
    }
}

/// The action a proposal will take if it reaches the threshold.
///
/// Proposal *content* is public by design — the prize hides who approved, not what was proposed.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum ProposedAction {
    /// Move `amount` from the multisig's treasury to `recipient`. The reference integration.
    TreasuryTransfer { recipient: Digest32, amount: u128 },
}

/// A proposal account.
///
/// Lives at `for_public_pda(program_id, PdaSeed(proposal_seed))`.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Proposal {
    /// State layout version.
    pub version: u16,
    /// The multisig this belongs to. Checked against the config account (`UnknownProposal`, 1006).
    pub config_hash: Digest32,
    /// Caller-chosen proposal identifier.
    pub proposal_id: Digest32,
    /// What executing this proposal does.
    pub action: ProposedAction,
    /// Approval nullifiers recorded so far.
    ///
    /// **This is the whole record of who approved, and it identifies nobody.** Each entry is
    /// `SHA256(DS_NF ‖ nsk ‖ multisig_id ‖ proposal_id)` — preimage-hiding, and unlinkable to the
    /// same member's nullifier on any other proposal. The count is `nullifiers.len()`; there is no
    /// separate counter to drift out of step with the set.
    pub nullifiers: Vec<Digest32>,
    /// Set once the proposal has executed.
    pub executed: bool,
}

impl Proposal {
    /// How many approvals have been recorded.
    #[must_use]
    pub fn approvals(&self) -> usize {
        self.nullifiers.len()
    }

    /// Whether this nullifier has already been recorded (**INV-4**, double-vote prevention).
    #[must_use]
    pub fn has_nullifier(&self, nf: &Digest32) -> bool {
        self.nullifiers.iter().any(|n| n == nf)
    }

    /// Whether the threshold has been reached.
    #[must_use]
    pub fn threshold_met(&self, m: Threshold) -> bool {
        self.approvals() >= usize::from(m)
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
mod tests {
    use super::*;

    fn config() -> MultisigConfig {
        MultisigConfig {
            version: STATE_VERSION,
            member_root: [0x11; 32],
            m: 2,
            n: 3,
            multisig_id: [0x22; 32],
            membership_program_id: [7; 8],
            proposal_count: 0,
        }
    }

    fn proposal() -> Proposal {
        Proposal {
            version: STATE_VERSION,
            config_hash: config().recompute_config_hash(),
            proposal_id: [0x33; 32],
            action: ProposedAction::TreasuryTransfer {
                recipient: [0x44; 32],
                amount: 1_000,
            },
            nullifiers: Vec::new(),
            executed: false,
        }
    }

    /// SC-C.6 / P-F2: the encoded state must contain nothing member-identifying.
    ///
    /// Asserted structurally: the only member-derived values a `Proposal` can hold are nullifiers,
    /// because it has no other field that could carry one.
    #[test]
    fn a_proposal_records_nullifiers_and_nothing_else_member_derived() {
        let mut p = proposal();
        p.nullifiers.push([0xAB; 32]);
        let encoded = borsh::to_vec(&p).expect("proposal encodes");
        let decoded = Proposal::try_from_slice(&encoded).expect("proposal decodes");
        assert_eq!(decoded, p);
        // Field-by-field: version, config_hash, proposal_id, action, nullifiers, executed.
        // None of these is a member id, an npk, an account id, or a bitmap of who approved.
        assert_eq!(decoded.nullifiers.len(), 1);
        assert_eq!(decoded.approvals(), 1);
    }

    #[test]
    fn config_rehashes_to_its_own_seed() {
        let c = config();
        assert_eq!(
            c.recompute_config_hash(),
            pmsig_core::config_hash(
                &c.member_root,
                c.m,
                c.n,
                &c.multisig_id,
                &c.membership_program_id
            )
        );
    }

    /// INV-3: tampering with any stored field breaks the rehash, so the account no longer matches
    /// the address it sits at.
    #[test]
    fn tampering_with_stored_config_breaks_the_rehash() {
        let honest = config().recompute_config_hash();
        for mutate in [
            (|c: &mut MultisigConfig| c.m = 1) as fn(&mut MultisigConfig),
            |c: &mut MultisigConfig| c.n = 9,
            |c: &mut MultisigConfig| c.member_root = [0xFF; 32],
            |c: &mut MultisigConfig| c.multisig_id = [0xFF; 32],
            |c: &mut MultisigConfig| c.membership_program_id = [0xFF; 8],
        ] {
            let mut c = config();
            mutate(&mut c);
            assert_ne!(c.recompute_config_hash(), honest);
        }
    }

    #[test]
    fn threshold_and_double_vote_helpers() {
        let mut p = proposal();
        assert!(!p.threshold_met(2));
        p.nullifiers.push([0xAB; 32]);
        assert!(!p.threshold_met(2));
        assert!(p.has_nullifier(&[0xAB; 32]));
        assert!(!p.has_nullifier(&[0xCD; 32]));
        p.nullifiers.push([0xCD; 32]);
        assert!(p.threshold_met(2));
    }

    #[test]
    fn malformed_configurations_are_detected() {
        let mut c = config();
        assert!(c.is_well_formed());
        c.m = 0;
        assert!(!c.is_well_formed());
        let mut c = config();
        c.m = 4; // m > n
        assert!(!c.is_well_formed());
    }
}
