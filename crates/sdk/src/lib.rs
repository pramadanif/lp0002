//! Client-side SDK for the private M-of-N multisig (prize criterion P-U1).
//!
//! Phase 0 scope: error surface skeleton only. Proving lands in Phase B, transaction building in
//! Phase C, and the member-facing API in Phase D.

use thiserror::Error;

/// Errors surfaced to a member by the SDK.
///
/// Reliability criterion **P-R1** requires proof-generation failures to reach the member as a
/// clear error rather than a panic or an opaque code, and **P-R3** requires the set to be
/// deterministic and documented. The full catalogue is written in Phase A
/// (`docs/error-codes.md`); this enum grows to match it.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SdkError {
    /// Placeholder so the surface compiles before Phase A fixes the catalogue.
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_renders_its_context() {
        let e = SdkError::NotImplemented("prove");
        assert_eq!(e.to_string(), "not implemented yet: prove");
    }
}
