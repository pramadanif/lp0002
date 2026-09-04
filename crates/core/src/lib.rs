//! Shared types for the private M-of-N multisig.
//!
//! This crate is compiled **both** for the host and for the risc0 guest, so it must stay
//! `no_std`-compatible in spirit: no I/O, no panicking constructs in library paths, and no
//! dependency that cannot cross-compile to `riscv32im-risc0-zkvm-elf`.
//!
//! Phase 0 scope: type skeleton only. The nullifier and `config_hash` formulas land in Phase A
//! (`docs/adr/ADR-001-architecture.md`) and are implemented here in Phase B.

/// Threshold `M` of an M-of-N multisig.
pub type Threshold = u8;

/// Number of members `N` of an M-of-N multisig.
pub type MemberCount = u8;

/// A 32-byte domain-separated digest (SHA-256, matching LEZ's `risc0_zkvm::sha` usage).
pub type Digest32 = [u8; 32];

/// Version of the on-chain state layout this build speaks.
pub const STATE_VERSION: u16 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_version_is_pinned() {
        assert_eq!(STATE_VERSION, 1);
    }
}
