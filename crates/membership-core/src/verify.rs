//! The membership check itself.
//!
//! Lives here rather than in the guest binary so it can be exercised directly on the host. The guest
//! (`programs/membership-lez`) is a thin wrapper: read inputs, call [`verify_approval`], echo states.
//!
//! Every failure is a panic. Inside a guest a panic aborts the proof, so an invalid approval yields
//! no `ProgramOutput`, `env::verify` fails in LEZ's privacy-preserving circuit, and the transaction
//! is invalid. There is deliberately no "returns false" path that a caller could forget to check.

use lee_core::{
    account::AccountId, encryption::ViewingPublicKey, Identifier, NullifierPublicKey,
    NullifierSecretKey,
};
use pmsig_core::{
    approval_nullifier, member_leaf,
    tree::{root_from_path, MemberPath},
};

use crate::{ApproveWitness, VIEWING_PUBLIC_KEY_LEN};

/// Verifies a membership claim and its approval nullifier against the account the caller presented.
///
/// `bound_account_id` must be the `account_id` of the approver's pre-state — the account LEZ's
/// privacy-preserving circuit has independently bound to a live, unspent commitment
/// (`lee/privacy_preserving_circuit/src/output.rs:91-94` and `:347-357`).
///
/// # Panics
///
/// - the viewing key is not [`VIEWING_PUBLIC_KEY_LEN`] bytes, or does not decode;
/// - the witness does not control `bound_account_id` (**the live-account binding**, ADR-001 D4);
/// - the approver's `npk` is not a leaf under `member_root`;
/// - `claimed_nullifier` is not the nullifier this witness must produce.
pub fn verify_approval(witness: &ApproveWitness, bound_account_id: &AccountId) {
    let npk = npk_of(&witness.nsk);

    // 1. LIVE-ACCOUNT BINDING (ADR-001 D4, gate H8).
    //
    //    Re-derive the account this `nsk` controls and require it to be the account presented.
    //
    //    Without this, the program would prove only "someone knows a member key" — true of a member
    //    with no live account at all, and true forever once the key exists. That is the
    //    derivation-only property rejected in prize PR #91. With it, the approval is pinned to a
    //    specific live account being spent in this transaction.
    //
    //    Note what this check is *not*: it is not what stops double-voting. That is the nullifier
    //    (INV-4) — substituting a different `nsk` fails the membership check below. What it stops is
    //    an approval whose on-chain footprint is an account unrelated to the member.
    //    `tests/derivation_only.rs` demonstrates the difference (SC-B.5).
    let derived = derive_account_id(&npk, &witness.vpk, witness.identifier);
    assert!(
        derived == *bound_account_id,
        "membership: witness does not control the approving account"
    );

    // 2. Membership in this multisig's member set.
    assert!(
        proves_membership(&npk, witness),
        "membership: approver is not a member of this multisig"
    );

    // 3. The nullifier the caller intends to record is the one this member must produce.
    assert!(
        approval_nullifier(&witness.nsk, &witness.multisig_id, &witness.proposal_id)
            == witness.claimed_nullifier,
        "membership: claimed nullifier does not match the witness"
    );
}

/// `npk = SHA256("LEE/keys" ‖ nsk ‖ [7] ‖ [0; 23])` — LEZ's own derivation.
#[must_use]
pub fn npk_of(nsk: &[u8; 32]) -> NullifierPublicKey {
    let nsk: NullifierSecretKey = *nsk;
    NullifierPublicKey::from(&nsk)
}

/// Whether the witness's Merkle path carries this `npk` to the claimed member root.
#[must_use]
pub fn proves_membership(npk: &NullifierPublicKey, witness: &ApproveWitness) -> bool {
    let leaf = member_leaf(&npk.to_byte_array());
    let path = MemberPath {
        index: usize::try_from(witness.member_index).unwrap_or(usize::MAX),
        siblings: witness.siblings.clone(),
    };
    root_from_path(&leaf, &path) == witness.member_root
}

/// `AccountId::for_regular_private_account`.
///
/// The length check is explicit rather than delegated: `ViewingPublicKey::from_bytes`, which would
/// perform it, is gated behind LEZ's `host` feature and is unavailable in the guest. Without the
/// check a wrong-length key would surface only as an unexplained account-id mismatch.
///
/// # Panics
/// If the viewing key is not [`VIEWING_PUBLIC_KEY_LEN`] bytes.
#[must_use]
pub fn derive_account_id(
    npk: &NullifierPublicKey,
    vpk: &ViewingPublicKey,
    identifier: Identifier,
) -> AccountId {
    assert!(
        vpk.to_bytes().len() == VIEWING_PUBLIC_KEY_LEN,
        "membership: viewing public key must be {VIEWING_PUBLIC_KEY_LEN} bytes"
    );
    AccountId::for_regular_private_account(npk, vpk, identifier)
}
