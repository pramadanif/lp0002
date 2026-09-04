//! SC-B.5 — proof that the live-account binding is load-bearing.
//!
//! Plan gate **H8** demands the membership proof bind to a *live* shielded account rather than
//! merely to a derived key. The reject in prize PR #91 was exactly that: derivation-only membership.
//!
//! A gate you cannot fail is not a gate. So this file keeps a deliberately weakened copy of the
//! check — [`verify_approval_derivation_only`], identical to the real one minus the account-binding
//! assertion — and demonstrates an input the two disagree on. If someone deletes the assertion from
//! `verify_approval`, [`the_binding_assertion_is_load_bearing`] fails.
//!
//! **What the assertion does and does not buy.** It does not prevent double-voting: that is INV-4,
//! via the nullifier being a deterministic function of `nsk`, and a substituted `nsk` would fail the
//! membership check anyway. What it buys is *liveness* — without it the guest proves only "someone
//! knows a member key", which is true of a member who never created a shielded account and stays true
//! forever. See `docs/tried-failed.md` for the incorrect version of this rationale we shipped first.

use lee_core::{account::AccountId, encryption::ViewingPublicKey};
use pmsig_core::{approval_nullifier, tree::MemberTree, Digest32};
use pmsig_membership_core::{
    verify::{derive_account_id, npk_of, proves_membership},
    verify_approval, ApproveWitness,
};

const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];

fn vpk() -> ViewingPublicKey {
    ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32])
}

/// A 2-of-3 multisig whose members are Alice, Bob and Carol.
struct Fixture {
    witness: ApproveWitness,
    alice_account: AccountId,
}

fn fixture() -> Fixture {
    let alice_nsk: Digest32 = [0x11; 32];
    let bob_nsk: Digest32 = [0x22; 32];
    let carol_nsk: Digest32 = [0x33; 32];

    let npks: Vec<Digest32> = [alice_nsk, bob_nsk, carol_nsk]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();

    let tree = MemberTree::new(&npks).expect("three members");
    let path = tree.path(0).expect("alice is a member");

    let alice_npk = npk_of(&alice_nsk);
    let alice_account = derive_account_id(&alice_npk, &vpk(), 0);

    let witness = ApproveWitness {
        nsk: alice_nsk,
        vpk: vpk(),
        identifier: 0,
        member_index: 0,
        siblings: path.siblings,
        multisig_id: MULTISIG_ID,
        proposal_id: PROPOSAL_ID,
        member_root: tree.root(),
        claimed_nullifier: approval_nullifier(&alice_nsk, &MULTISIG_ID, &PROPOSAL_ID),
    };

    Fixture {
        witness,
        alice_account,
    }
}

/// The weakened variant: membership and nullifier are checked, the account binding is not.
///
/// This is what the guest would be if D4's assertion were dropped — and it is what prize PR #91
/// shipped.
fn verify_approval_derivation_only(witness: &ApproveWitness, _bound_account_id: &AccountId) {
    let npk = npk_of(&witness.nsk);
    assert!(
        proves_membership(&npk, witness),
        "membership: approver is not a member of this multisig"
    );
    assert!(
        approval_nullifier(&witness.nsk, &witness.multisig_id, &witness.proposal_id)
            == witness.claimed_nullifier,
        "membership: claimed nullifier does not match the witness"
    );
}

fn rejects(f: impl FnOnce() + std::panic::UnwindSafe) -> bool {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // keep expected panics out of test output
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(hook);
    result.is_err()
}

#[test]
fn an_honest_approval_verifies() {
    let f = fixture();
    verify_approval(&f.witness, &f.alice_account);
}

/// **The SC-B.5 assertion.**
///
/// A witness naming Alice's key, presented alongside an account Alice does not control, is accepted
/// by the derivation-only variant and rejected by the real one. That difference *is* gate H8.
#[test]
fn the_binding_assertion_is_load_bearing() {
    let f = fixture();

    // An account unrelated to Alice — e.g. a throwaway the submitter happens to control.
    let unrelated = derive_account_id(&npk_of(&[0xEE; 32]), &vpk(), 0);
    assert_ne!(unrelated, f.alice_account);

    // Derivation-only: happily accepts. This is the #91 failure mode.
    verify_approval_derivation_only(&f.witness, &unrelated);

    // The real check: refuses, because the witness does not control the presented account.
    assert!(
        rejects(|| verify_approval(&f.witness, &unrelated)),
        "H8 REGRESSION: verify_approval accepted a witness that does not control the presented \
         account. The live-account binding in ADR-001 D4 has been removed or weakened."
    );
}

/// The same member's *other* addresses are still not the account that was presented.
#[test]
fn a_different_address_of_the_same_member_is_still_rejected() {
    let f = fixture();
    let alice_other = derive_account_id(&npk_of(&f.witness.nsk), &vpk(), 1);
    assert_ne!(alice_other, f.alice_account);
    assert!(rejects(move || verify_approval(&f.witness, &alice_other)));
}

#[test]
fn a_non_member_cannot_approve() {
    let mut w = fixture().witness;
    let outsider: Digest32 = [0x99; 32];
    w.nsk = outsider;
    w.claimed_nullifier = approval_nullifier(&outsider, &MULTISIG_ID, &PROPOSAL_ID);
    let account = derive_account_id(&npk_of(&outsider), &vpk(), 0);
    // Rejected even though the witness genuinely controls the account it presents.
    assert!(rejects(move || verify_approval(&w, &account)));
}

#[test]
fn a_forged_nullifier_is_rejected() {
    let f = fixture();
    let mut w = f.witness;
    w.claimed_nullifier = [0xFF; 32];
    let account = f.alice_account;
    assert!(rejects(move || verify_approval(&w, &account)));
}

#[test]
fn an_approval_against_the_wrong_member_root_is_rejected() {
    let f = fixture();
    let mut w = f.witness;
    w.member_root = [0x00; 32];
    let account = f.alice_account;
    assert!(rejects(move || verify_approval(&w, &account)));
}

#[test]
fn an_approval_for_a_different_proposal_is_rejected() {
    let f = fixture();
    let mut w = f.witness;
    // The nullifier no longer matches the proposal named in the witness.
    w.proposal_id = [0xC3; 32];
    let account = f.alice_account;
    assert!(rejects(move || verify_approval(&w, &account)));
}

/// A wrong-length viewing key must be named as such, not surface as an opaque id mismatch.
///
/// `ViewingPublicKey` is a newtype over `Vec<u8>` whose length-checking constructor is host-only, so
/// a malformed one is built the way a malicious client would: by deserialising bytes directly.
#[test]
fn a_wrong_length_viewing_key_is_rejected_by_name() {
    use borsh::BorshDeserialize as _;
    let f = fixture();
    let mut w = f.witness;
    let mut encoded = borsh::to_vec(&vec![0_u8; 32]).expect("encode short key");
    w.vpk = ViewingPublicKey::deserialize(&mut encoded.as_slice()).expect("decode short key");
    let account = f.alice_account;
    assert!(rejects(move || verify_approval(&w, &account)));
}

/// The nullifier is what stops double-voting, and it does so without the binding assertion.
/// Recorded as a test so the corrected reasoning in `docs/tried-failed.md` is checkable.
#[test]
fn the_same_member_reproduces_one_nullifier_per_proposal() {
    let f = fixture();
    let again = approval_nullifier(&f.witness.nsk, &MULTISIG_ID, &PROPOSAL_ID);
    assert_eq!(again, f.witness.claimed_nullifier, "double-vote detectable");

    let other_proposal = approval_nullifier(&f.witness.nsk, &MULTISIG_ID, &[0xD4; 32]);
    assert_ne!(
        other_proposal, f.witness.claimed_nullifier,
        "unlinkable across proposals"
    );
}
