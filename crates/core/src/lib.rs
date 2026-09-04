//! Shared types and domain-separated hash formulas for the private M-of-N multisig.
//!
//! Compiled **both** for the host and for the risc0 guest, so it avoids I/O, panicking constructs
//! in library paths, and anything that cannot cross-compile to `riscv32im-risc0-zkvm-elf`.
//!
//! Hashing goes through [`risc0_zkvm::sha`], the same implementation LEZ uses, so digests produced
//! here are byte-identical to LEZ's on both targets.
//!
//! Every formula below is specified in `docs/adr/ADR-001-architecture.md` §3. That document is
//! normative; if the two ever disagree, the ADR is right and this file is a bug.

use risc0_zkvm::sha::{Impl, Sha256 as _};

pub mod tree;

/// Threshold `M` of an M-of-N multisig.
pub type Threshold = u8;

/// Number of members `N` of an M-of-N multisig.
pub type MemberCount = u8;

/// A 32-byte domain-separated digest.
pub type Digest32 = [u8; 32];

/// Version of the on-chain state layout this build speaks.
pub const STATE_VERSION: u16 = 1;

/// Builds a 32-byte domain separator: ASCII label, zero-padded on the right.
///
/// Mirrors LEZ's own convention (`b"/LEE/v0.3/Commitment/\x00…"`, `lee/state_machine/core`).
///
/// # Panics
/// At compile time, via the `assert!`, if the label exceeds 32 bytes. Every call site is a `const`,
/// so an over-long label is a build error rather than a runtime fault.
#[allow(
    clippy::indexing_slicing,
    clippy::panic,
    reason = "const fn: the assert bounds the loop, and any violation fails the build"
)]
const fn domain(label: &[u8]) -> Digest32 {
    assert!(label.len() <= 32, "domain separator must fit in 32 bytes");
    let mut out = [0_u8; 32];
    let mut i = 0;
    while i < label.len() {
        out[i] = label[i];
        i += 1;
    }
    out
}

/// Domain separator for [`config_hash`]. 22 ASCII bytes + 10 zero bytes.
pub const DS_CONFIG: Digest32 = domain(b"/LP0002/v1/ConfigHash/");
/// Domain separator for [`approval_nullifier`]. 29 ASCII bytes + 3 zero bytes.
pub const DS_NF: Digest32 = domain(b"/LP0002/v1/Nullifier/Approve/");
/// Domain separator for [`member_leaf`]. 22 ASCII bytes + 10 zero bytes.
pub const DS_LEAF: Digest32 = domain(b"/LP0002/v1/MemberLeaf/");
/// Domain separator for [`proposal_seed`]. 24 ASCII bytes + 8 zero bytes.
pub const DS_PROP: Digest32 = domain(b"/LP0002/v1/ProposalSeed/");

fn sha256(bytes: &[u8]) -> Digest32 {
    // `Impl::hash_bytes` returns a 32-byte digest; the conversion cannot fail.
    let digest = Impl::hash_bytes(bytes);
    let mut out = [0_u8; 32];
    out.copy_from_slice(digest.as_bytes());
    out
}

/// `config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] ‖ membership_program_id[32] )`
///
/// Used directly as the 32-byte PDA seed of the multisig's config account. Because `M`,
/// `member_root` and the verifier's program id are all inside the digest, a prover who lowers the
/// threshold, substitutes a member set, or names a hostile membership program derives a *different*
/// address rather than a weaker multisig (ADR-001 INV-1/INV-2, extended by ADR-002).
///
/// `membership_program_id` is a LEZ `ProgramId` (`[u32; 8]`) serialised little-endian.
#[must_use]
pub fn config_hash(
    member_root: &Digest32,
    m: Threshold,
    n: MemberCount,
    multisig_id: &Digest32,
    membership_program_id: &[u32; 8],
) -> Digest32 {
    let mut buf = [0_u8; 32 + 32 + 1 + 1 + 32 + 32];
    let mut w = Writer::new(&mut buf);
    w.put(&DS_CONFIG);
    w.put(member_root);
    w.put(&[m]);
    w.put(&[n]);
    w.put(multisig_id);
    for word in membership_program_id {
        w.put(&word.to_le_bytes());
    }
    sha256(w.finish())
}

/// `nf_approve = SHA256( DS_NF ‖ nsk[32] ‖ multisig_id[32] ‖ proposal_id[32] )`
///
/// Deterministic in `(member, multisig, proposal)`, so a second approval of the same proposal
/// reproduces the same nullifier and is rejected. Keyed to `nsk` rather than to an account id, so a
/// member cannot vote twice from another address in their own 2^128 family (ADR-001 D5).
#[must_use]
pub fn approval_nullifier(
    nsk: &Digest32,
    multisig_id: &Digest32,
    proposal_id: &Digest32,
) -> Digest32 {
    let mut buf = [0_u8; 32 * 4];
    let mut w = Writer::new(&mut buf);
    w.put(&DS_NF);
    w.put(nsk);
    w.put(multisig_id);
    w.put(proposal_id);
    sha256(w.finish())
}

/// `member_leaf = SHA256( DS_LEAF ‖ npk[32] )`
///
/// The member set is a tree over nullifier **public** keys, not account ids, so one member may
/// approve from any address in their family while still yielding one nullifier.
#[must_use]
pub fn member_leaf(npk: &Digest32) -> Digest32 {
    let mut buf = [0_u8; 64];
    let mut w = Writer::new(&mut buf);
    w.put(&DS_LEAF);
    w.put(npk);
    sha256(w.finish())
}

/// `SHA256( left[32] ‖ right[32] )` — the internal-node hash of the member tree.
///
/// Same shape as LEZ's commitment-set tree, so one mental model covers both.
#[must_use]
pub fn sha256_pair(left: &Digest32, right: &Digest32) -> Digest32 {
    let mut buf = [0_u8; 64];
    let mut w = Writer::new(&mut buf);
    w.put(left);
    w.put(right);
    sha256(w.finish())
}

/// `proposal_pda_seed = SHA256( DS_PROP ‖ config_hash[32] ‖ proposal_id[32] )`
#[must_use]
pub fn proposal_seed(config_hash: &Digest32, proposal_id: &Digest32) -> Digest32 {
    let mut buf = [0_u8; 96];
    let mut w = Writer::new(&mut buf);
    w.put(&DS_PROP);
    w.put(config_hash);
    w.put(proposal_id);
    sha256(w.finish())
}

/// Fixed-capacity buffer writer.
///
/// Preimages here are built from fixed-size fields, so a `Vec` would add an allocator to the guest
/// for no benefit. Writing past the end is impossible: each caller sizes its buffer from the same
/// field widths it then writes.
struct Writer<'a> {
    buf: &'a mut [u8],
    at: usize,
}

impl<'a> Writer<'a> {
    const fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, at: 0 }
    }

    fn put(&mut self, bytes: &[u8]) {
        let end = self.at + bytes.len();
        if let Some(slot) = self.buf.get_mut(self.at..end) {
            slot.copy_from_slice(bytes);
            self.at = end;
        }
    }

    fn finish(&self) -> &[u8] {
        self.buf.get(..self.at).unwrap_or(&[])
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

    const ROOT: Digest32 = [0x11; 32];
    const VERIFIER: [u32; 8] = [9; 8];
    const MSIG: Digest32 = [0x22; 32];
    const NSK: Digest32 = [0x33; 32];
    const PROP: Digest32 = [0x44; 32];

    #[test]
    fn state_version_is_pinned() {
        assert_eq!(STATE_VERSION, 1);
    }

    #[test]
    fn domain_separators_are_ascii_then_zero_padding() {
        // The exact byte arithmetic quoted in ADR-001 §3.
        assert_eq!(&DS_CONFIG[..22], b"/LP0002/v1/ConfigHash/");
        assert_eq!(&DS_CONFIG[22..], &[0_u8; 10]);
        assert_eq!(&DS_NF[..29], b"/LP0002/v1/Nullifier/Approve/");
        assert_eq!(&DS_NF[29..], &[0_u8; 3]);
        assert_eq!(&DS_LEAF[..22], b"/LP0002/v1/MemberLeaf/");
        assert_eq!(&DS_LEAF[22..], &[0_u8; 10]);
        assert_eq!(&DS_PROP[..24], b"/LP0002/v1/ProposalSeed/");
        assert_eq!(&DS_PROP[24..], &[0_u8; 8]);
    }

    #[test]
    fn domains_are_distinct() {
        let all = [DS_CONFIG, DS_NF, DS_LEAF, DS_PROP];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "domain separators must be distinct");
            }
        }
    }

    /// ADR-001 INV-1: lowering the threshold changes the address rather than weakening the multisig.
    #[test]
    fn lowering_m_changes_config_hash() {
        let honest = config_hash(&ROOT, 2, 3, &MSIG, &VERIFIER);
        let lowered = config_hash(&ROOT, 1, 3, &MSIG, &VERIFIER);
        assert_ne!(honest, lowered);
    }

    /// ADR-001 INV-2: substituting a member set changes the address.
    #[test]
    fn substituting_member_root_changes_config_hash() {
        let honest = config_hash(&ROOT, 2, 3, &MSIG, &VERIFIER);
        let forged = config_hash(&[0xAA; 32], 2, 3, &MSIG, &VERIFIER);
        assert_ne!(honest, forged);
    }

    #[test]
    fn config_hash_separates_every_field() {
        let base = config_hash(&ROOT, 2, 3, &MSIG, &VERIFIER);
        assert_ne!(
            base,
            config_hash(&ROOT, 2, 4, &MSIG, &VERIFIER),
            "n must matter"
        );
        assert_ne!(
            base,
            config_hash(&ROOT, 2, 3, &[0x99; 32], &VERIFIER),
            "multisig_id must matter"
        );
        assert_ne!(
            base,
            config_hash(&ROOT, 2, 3, &MSIG, &[1; 8]),
            "the membership verifier must matter (ADR-002)"
        );
    }

    /// ADR-002: naming a hostile verifier changes the address, exactly as lowering M does.
    #[test]
    fn substituting_the_verifier_changes_config_hash() {
        let honest = config_hash(&ROOT, 2, 3, &MSIG, &VERIFIER);
        let hostile = config_hash(&ROOT, 2, 3, &MSIG, &[0xDEAD_BEEF; 8]);
        assert_ne!(honest, hostile);
    }

    /// ADR-001 INV-4: the same member approving the same proposal twice yields the same nullifier.
    #[test]
    fn approval_nullifier_is_deterministic() {
        assert_eq!(
            approval_nullifier(&NSK, &MSIG, &PROP),
            approval_nullifier(&NSK, &MSIG, &PROP)
        );
    }

    /// The same member on a different proposal must be unlinkable — a different nullifier.
    #[test]
    fn approval_nullifier_differs_per_proposal_multisig_and_member() {
        let base = approval_nullifier(&NSK, &MSIG, &PROP);
        assert_ne!(base, approval_nullifier(&NSK, &MSIG, &[0x55; 32]));
        assert_ne!(base, approval_nullifier(&NSK, &[0x66; 32], &PROP));
        assert_ne!(base, approval_nullifier(&[0x77; 32], &MSIG, &PROP));
    }

    /// Distinct domains must keep distinct constructions apart even on identical input bytes.
    #[test]
    fn domain_separation_holds_across_formulas() {
        let x: Digest32 = [0x01; 32];
        let leaf = member_leaf(&x);
        let prop = proposal_seed(&x, &x);
        let nf = approval_nullifier(&x, &x, &x);
        assert_ne!(leaf, prop);
        assert_ne!(leaf, nf);
        assert_ne!(prop, nf);
    }

    #[test]
    fn sha256_matches_a_known_vector() {
        // SHA-256 of the empty string — guards against the hash impl changing underneath us.
        assert_eq!(
            sha256(b""),
            [
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55
            ]
        );
    }
}
