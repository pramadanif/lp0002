//! Deterministic, documented error codes.
//!
//! Criterion **P-R3** requires the verifier to return deterministic, documented codes for every
//! invalid-proof and double-vote scenario. The catalogue is `docs/error-codes.md`; this enum is its
//! executable half, and `code_matches_documentation` keeps the two from drifting.
//!
//! **These codes say as little as possible.** `DuplicateNullifier` reports *that* a nullifier
//! repeated, never whose — a code like "member 2 already approved" would undo the entire scheme.

use core::fmt;

/// An on-chain failure of the multisig program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MultisigError {
    /// 1001 — the membership proof did not verify.
    InvalidProof,
    /// 1002 — this nullifier is already recorded: a double-vote attempt (**P-F3**).
    DuplicateNullifier,
    /// 1003 — the config account's stored fields do not rehash to the address it sits at (INV-3).
    ConfigHashMismatch,
    /// 1004 — `execute` attempted below the threshold.
    ThresholdNotMet,
    /// 1005 — `execute` on a proposal that has already run.
    AlreadyExecuted,
    /// 1006 — the proposal does not belong to this multisig, or is not at its derived address.
    UnknownProposal,
    /// 1007 — the approval was proved against a different member root.
    MemberRootMismatch,
    /// 1008 — an approval arrived after the proposal executed.
    ProposalClosed,
    /// 1009 — nonsensical configuration: `M == 0`, `N == 0`, or `M > N`.
    InvalidThresholdConfig,
    /// 1010 — the target account is already initialised.
    AccountAlreadyInitialized,
    /// 1011 — an approval was attempted outside the privacy-preserving path (**H9**).
    PublicApprovePathRejected,
    /// 1012 — the proposed action is malformed or unsupported.
    InvalidProposalAction,
    /// 1013 — the chained call named a membership program this multisig is not bound to (ADR-002).
    WrongMembershipProgram,
}

impl MultisigError {
    /// The stable numeric code. Never reused once retired.
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::InvalidProof => 1001,
            Self::DuplicateNullifier => 1002,
            Self::ConfigHashMismatch => 1003,
            Self::ThresholdNotMet => 1004,
            Self::AlreadyExecuted => 1005,
            Self::UnknownProposal => 1006,
            Self::MemberRootMismatch => 1007,
            Self::ProposalClosed => 1008,
            Self::InvalidThresholdConfig => 1009,
            Self::AccountAlreadyInitialized => 1010,
            Self::PublicApprovePathRejected => 1011,
            Self::InvalidProposalAction => 1012,
            Self::WrongMembershipProgram => 1013,
        }
    }

    /// Short stable name, as it appears in `docs/error-codes.md`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::InvalidProof => "InvalidProof",
            Self::DuplicateNullifier => "DuplicateNullifier",
            Self::ConfigHashMismatch => "ConfigHashMismatch",
            Self::ThresholdNotMet => "ThresholdNotMet",
            Self::AlreadyExecuted => "AlreadyExecuted",
            Self::UnknownProposal => "UnknownProposal",
            Self::MemberRootMismatch => "MemberRootMismatch",
            Self::ProposalClosed => "ProposalClosed",
            Self::InvalidThresholdConfig => "InvalidThresholdConfig",
            Self::AccountAlreadyInitialized => "AccountAlreadyInitialized",
            Self::PublicApprovePathRejected => "PublicApprovePathRejected",
            Self::InvalidProposalAction => "InvalidProposalAction",
            Self::WrongMembershipProgram => "WrongMembershipProgram",
        }
    }

    /// Every code, so tests and the IDL can enumerate them.
    #[must_use]
    pub const fn all() -> [Self; 13] {
        [
            Self::InvalidProof,
            Self::DuplicateNullifier,
            Self::ConfigHashMismatch,
            Self::ThresholdNotMet,
            Self::AlreadyExecuted,
            Self::UnknownProposal,
            Self::MemberRootMismatch,
            Self::ProposalClosed,
            Self::InvalidThresholdConfig,
            Self::AccountAlreadyInitialized,
            Self::PublicApprovePathRejected,
            Self::InvalidProposalAction,
            Self::WrongMembershipProgram,
        ]
    }
}

impl fmt::Display for MultisigError {
    /// Renders as `"<code> <Name>"`, so a failed transaction is greppable against the catalogue.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.name())
    }
}

/// A documented error an integrator handles should compose with `?` and `Box<dyn Error>` like any
/// other. risc0 guests link `std`, so this costs the guest build nothing.
impl std::error::Error for MultisigError {}

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

    #[test]
    fn codes_are_unique_and_in_the_documented_range() {
        let mut seen = Vec::new();
        for e in MultisigError::all() {
            assert!((1001..=1999).contains(&e.code()), "{e} out of range");
            assert!(!seen.contains(&e.code()), "duplicate code {e}");
            seen.push(e.code());
        }
        assert_eq!(seen.len(), 13);
    }

    #[test]
    fn it_is_a_real_error_type() {
        fn takes_error(_: Box<dyn std::error::Error>) {}
        takes_error(Box::new(MultisigError::ThresholdNotMet));
    }

    #[test]
    fn display_is_greppable() {
        assert_eq!(
            MultisigError::DuplicateNullifier.to_string(),
            "1002 DuplicateNullifier"
        );
    }

    /// **P-R3**: every code in this enum must appear in `docs/error-codes.md`, and with the same
    /// number. A code the operator cannot look up is not a documented code.
    #[test]
    fn every_code_appears_in_the_documentation() {
        let doc = include_str!("../../../docs/error-codes.md");
        for e in MultisigError::all() {
            assert!(
                doc.contains(e.name()),
                "{e} is missing from docs/error-codes.md"
            );
            assert!(
                doc.contains(&e.code().to_string()),
                "code {} is missing from docs/error-codes.md",
                e.code()
            );
        }
    }
}
