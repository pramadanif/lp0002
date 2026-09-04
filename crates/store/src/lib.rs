//! Local client store for partial approval sets.
//!
//! Reliability criterion **P-R2** requires a partial set of approvals (fewer than `M`) to be
//! preserved and resumable across client restarts. Phase 0 scope: skeleton only; the on-disk
//! format and the restart test land in Phase D (SC-D.2).

/// Filename used under the client's data directory.
pub const STORE_FILENAME: &str = "approvals.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_filename_is_stable() {
        assert_eq!(STORE_FILENAME, "approvals.json");
    }
}
