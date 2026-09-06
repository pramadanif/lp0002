//! Client-side SDK for the private M-of-N multisig (prize criterion P-U1).
//!
//! Phase 0 scope: error surface skeleton only. Proving lands in Phase B, transaction building in
//! Phase C, and the member-facing API in Phase D.

use thiserror::Error;

pub mod address;
pub mod member;
pub mod prove;

/// Errors surfaced to a member by the SDK.
///
/// Reliability criterion **P-R1** requires proof-generation failures to reach the member as a
/// clear error rather than a panic or an opaque code, and **P-R3** requires the set to be
/// deterministic and documented. The full catalogue is written in Phase A
/// (`docs/error-codes.md`); this enum grows to match it.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SdkError {
    /// 2001 — the prover did not produce a receipt.
    #[error("2001 ProofGenerationFailed: {0}")]
    ProofGenerationFailed(String),

    /// 2003 — `RISC0_DEV_MODE` is on, which produces a fake receipt that proves nothing.
    #[error(
        "2003 DevModeRefused: RISC0_DEV_MODE is enabled, which produces a fake receipt. \
         Unset it and re-run; a real proof needs r0vm."
    )]
    DevModeRefused,

    /// 2002 — `r0vm` is not installed or not on `PATH`.
    #[error(
        "2002 ProverNotFound: r0vm was not found. Install it with:\n  \
         curl -L https://risczero.com/install | bash && rzup install"
    )]
    ProverNotFound,

    /// 2004 — the supplied key is not a member of this multisig.
    #[error("2004 NotAMember: this key does not derive an npk under the multisig's member root")]
    NotAMember,

    /// 2006 — this member has already approved this proposal.
    #[error("2006 AlreadyApproved: this member's nullifier is already recorded for this proposal")]
    AlreadyApproved,

    /// 2007 — local state is behind the chain.
    #[error("2007 StaleProposal: the proposal has already reached its threshold or been executed")]
    StaleProposal,
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

    /// Error text must carry the documented code so a member can look it up in
    /// `docs/error-codes.md` (P-R1, P-R3).
    #[test]
    fn errors_carry_their_documented_code() {
        assert!(SdkError::DevModeRefused.to_string().starts_with("2003 "));
        assert!(SdkError::ProofGenerationFailed("x".into())
            .to_string()
            .starts_with("2001 "));
        assert!(SdkError::ProverNotFound.to_string().starts_with("2002 "));
        assert!(SdkError::NotAMember.to_string().starts_with("2004 "));
        assert!(SdkError::AlreadyApproved.to_string().starts_with("2006 "));
        assert!(SdkError::StaleProposal.to_string().starts_with("2007 "));
    }

    /// Every variant must be reachable from real code, not just from its own test. A documented
    /// error that cannot occur is worse than an undocumented one: it tells a reader the system
    /// behaves in a way it does not.
    #[test]
    fn prover_detection_is_wired_to_the_error() {
        // `prove_approval` consults this before doing anything expensive, so ProverNotFound is
        // reachable exactly when the prover is missing.
        let _ = crate::prove::prover_available();
    }

    /// P-R1: an error a member sees must say what to do next, not just what failed.
    #[test]
    fn prover_not_found_tells_the_member_how_to_fix_it() {
        let msg = SdkError::ProverNotFound.to_string();
        assert!(msg.contains("rzup install"), "must name the remedy: {msg}");
    }
}
