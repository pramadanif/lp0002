//! Client-side SDK for the private M-of-N multisig (prize criterion P-U1).
//!
//! Phase 0 scope: error surface skeleton only. Proving lands in Phase B, transaction building in
//! Phase C, and the member-facing API in Phase D.

use thiserror::Error;

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

    /// Placeholder for codes not yet wired. Removed once Phase D completes the catalogue.
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
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

    #[test]
    fn error_renders_its_context() {
        let e = SdkError::NotImplemented("prove");
        assert_eq!(e.to_string(), "not implemented yet: prove");
    }

    /// Error text must carry the documented code so a member can look it up in
    /// `docs/error-codes.md` (P-R1, P-R3).
    #[test]
    fn errors_carry_their_documented_code() {
        assert!(SdkError::DevModeRefused.to_string().starts_with("2003 "));
        assert!(SdkError::ProofGenerationFailed("x".into())
            .to_string()
            .starts_with("2001 "));
    }
}
