#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]

//! SC-B.7 — byte-compatibility with LEZ's own account derivation.
//!
//! The values asserted here are **LEZ's**, lifted from the `#[test]` blocks upstream pins in
//! `lee/state_machine/core/src/nullifier.rs` at tag v0.2.4. Reusing upstream's expected outputs means
//! compatibility is demonstrated against their numbers, not asserted against our own.
//!
//! If LEZ ever changes a derivation, these fail loudly instead of the mismatch surfacing later as
//! unexplained rejections on testnet — which is precisely how the v0.2.0-vs-v0.2.4 `vpk` change would
//! have bitten us (see `docs/tried-failed.md`).

use lee_core::{account::AccountId, encryption::ViewingPublicKey};
use pmsig_membership_core::verify::{derive_account_id, npk_of};

/// The `nsk` LEZ pins in `nullifier.rs::from_secret_key`.
const LEZ_NSK: [u8; 32] = [
    57, 5, 64, 115, 153, 56, 184, 51, 207, 238, 99, 165, 147, 214, 213, 151, 30, 251, 30, 196, 134,
    22, 224, 211, 237, 120, 136, 225, 188, 220, 249, 28,
];

/// The `npk` LEZ expects from `LEZ_NSK`.
const LEZ_NPK: [u8; 32] = [
    78, 20, 20, 5, 177, 198, 233, 100, 175, 134, 174, 200, 24, 205, 68, 215, 130, 74, 35, 54, 154,
    184, 219, 42, 168, 106, 126, 147, 133, 244, 18, 218,
];

/// Upstream's viewing key for these vectors: `ViewingPublicKey::from_seed(&[1; 32], &[2; 32])`.
fn lez_vpk() -> ViewingPublicKey {
    ViewingPublicKey::from_seed(&[1_u8; 32], &[2_u8; 32])
}

#[test]
fn npk_derivation_matches_lez_pinned_vector() {
    assert_eq!(
        npk_of(&LEZ_NSK).to_byte_array(),
        LEZ_NPK,
        "npk derivation diverged from LEZ's pinned vector"
    );
}

#[test]
fn account_id_derivation_matches_lez_pinned_vector_identifier_0() {
    // nullifier.rs::account_id_from_nullifier_public_key
    let expected = AccountId::new([
        242, 239, 57, 244, 89, 109, 65, 201, 223, 100, 43, 87, 205, 83, 148, 161, 176, 22, 208,
        220, 68, 135, 10, 171, 182, 80, 54, 74, 228, 244, 236, 7,
    ]);
    let got = derive_account_id(&npk_of(&LEZ_NSK), &lez_vpk(), 0);
    assert_eq!(got, expected);
}

#[test]
fn account_id_derivation_matches_lez_pinned_vector_identifier_1() {
    // nullifier.rs::account_id_from_nullifier_public_key_identifier_1
    let expected = AccountId::new([
        149, 125, 157, 109, 119, 81, 9, 163, 231, 181, 214, 43, 57, 113, 221, 72, 180, 149, 189,
        170, 32, 181, 255, 231, 19, 92, 235, 59, 153, 185, 172, 206,
    ]);
    let got = derive_account_id(&npk_of(&LEZ_NSK), &lez_vpk(), 1);
    assert_eq!(got, expected);
}

#[test]
fn account_id_derivation_matches_lez_byte_asymmetric_identifier() {
    // nullifier.rs::account_id_from_nullifier_public_key_byte_asymmetric_identifier — catches any
    // little/big-endian slip in the identifier, which a symmetric value would hide.
    let identifier: u128 = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210;
    let expected = AccountId::new([
        30, 232, 222, 201, 233, 125, 124, 194, 58, 39, 121, 96, 185, 84, 168, 109, 80, 111, 159,
        112, 84, 100, 133, 244, 16, 34, 221, 35, 128, 131, 98, 159,
    ]);
    let got = derive_account_id(&npk_of(&LEZ_NSK), &lez_vpk(), identifier);
    assert_eq!(got, expected);
}

#[test]
fn viewing_public_key_is_the_length_the_witness_declares() {
    assert_eq!(
        lez_vpk().to_bytes().len(),
        pmsig_membership_core::VIEWING_PUBLIC_KEY_LEN
    );
}

/// The v0.2.0 derivation omitted `vpk` entirely. If we were accidentally building against it, the
/// vectors above would fail — but this makes the reason explicit rather than leaving a puzzling diff.
#[test]
fn identifier_and_vpk_both_affect_the_address() {
    let npk = npk_of(&LEZ_NSK);
    let a = derive_account_id(&npk, &lez_vpk(), 0);
    let b = derive_account_id(&npk, &lez_vpk(), 1);
    let other_vpk = ViewingPublicKey::from_seed(&[9_u8; 32], &[9_u8; 32]);
    let c = derive_account_id(&npk, &other_vpk, 0);
    assert_ne!(a, b, "identifier must affect the address");
    assert_ne!(
        a, c,
        "vpk must affect the address (this is the v0.2.4 change)"
    );
}
