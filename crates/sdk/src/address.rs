//! Deriving the public addresses a multisig lives at.
//!
//! An integrator needs these to look anything up: the config account's address is where the
//! multisig's state — and, under **INV-7**, its funds — actually sit, and the proposal address is
//! where approvals accumulate. They were previously computed inside one example and nowhere else,
//! so a second caller had to copy the formula. That is how two spellings of the same rule start
//! drifting apart, which is the failure `config_hash` is asserted against across three documents.

use pmsig_core::Digest32;
use pmsig_multisig_core::ProgramIdWords;
use risc0_zkvm::sha::{Impl, Sha256 as _};

/// LEZ's domain prefix for a public PDA, padded to 32 bytes
/// (`lee/state_machine/core/src/program/mod.rs`).
const PDA_PREFIX: &[u8; 32] = b"/LEE/v0.2/AccountId/PDA/\0\0\0\0\0\0\0\0";

/// `AccountId::for_public_pda` — `SHA256(prefix ‖ program_id ‖ seed)`.
///
/// The program id is hashed as little-endian words, which is how LEZ serialises a `ProgramId`.
#[must_use]
pub fn public_pda(program_id: &ProgramIdWords, seed: &Digest32) -> Digest32 {
    let mut buf = Vec::with_capacity(96);
    buf.extend_from_slice(PDA_PREFIX);
    for w in program_id {
        buf.extend_from_slice(&w.to_le_bytes());
    }
    buf.extend_from_slice(seed);
    let d = Impl::hash_bytes(&buf);
    let mut out = [0_u8; 32];
    out.copy_from_slice(d.as_bytes());
    out
}

/// Base58, the encoding LEZ prints account ids in and the only form its RPC accepts.
#[must_use]
pub fn base58(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut n: Vec<u8> = bytes.to_vec();
    let mut out = Vec::new();
    while n.iter().any(|&x| x != 0) {
        let mut rem = 0_u32;
        for byte in &mut n {
            let cur = (rem << 8) | u32::from(*byte);
            *byte = u8::try_from(cur / 58).unwrap_or(0);
            rem = cur % 58;
        }
        // `rem` is a remainder mod 58, so it always indexes the alphabet — but say so rather than
        // index and hope.
        match ALPHABET.get(rem as usize) {
            Some(&c) => out.push(c),
            None => return String::new(),
        }
    }
    // Base58 encodes each leading zero byte as '1'.
    let leading_zeros = bytes.iter().take_while(|&&b| b == 0).count();
    out.resize(out.len() + leading_zeros, b'1');
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

/// The address a multisig's config account lives at — and, under INV-7, holds its funds.
#[must_use]
pub fn config_address(multisig_program_id: &ProgramIdWords, config_hash: &Digest32) -> String {
    base58(&public_pda(multisig_program_id, config_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned against the address the live testnet actually used for the deployed 2-of-3, so this
    /// is checked against a real chain rather than against itself.
    #[test]
    fn matches_the_address_the_testnet_used() {
        let pid: ProgramIdWords = [
            1590442955, 1212499613, 1170339484, 938167288, 4105589115, 2918885946, 3976464305,
            2797495876,
        ];
        let mut ch = [0_u8; 32];
        assert!(
            hex::decode_to_slice(
                "f8c4c3bd0145c054ba8448aa062085a21792edc04806cec151a36ea6ac6c1ce6",
                &mut ch,
            )
            .is_ok(),
            "the pinned config_hash must be valid hex"
        );
        assert_eq!(
            config_address(&pid, &ch),
            "FV4UKbXGimwoHjvRZHgP6hnBMcPyE98E5ZAdkrHvQkP1"
        );
    }

    #[test]
    fn base58_keeps_leading_zeros_as_ones() {
        assert_eq!(base58(&[0, 0, 1]), "112");
    }
}
