#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! **SC-D.5 / W8 / P-F1** — a co-member approves without learning who approved before them.
//!
//! The prize requires privacy from "on-chain observers **or other members**". The on-chain half is
//! covered elsewhere; this file is about the second, which is the one a design can lose by accident —
//! a coordinator that tracks "who has signed" passes every chain-level test and still fails.
//!
//! The argument here is structural rather than statistical. Bob's approval is built from a
//! `MultisigView` assembled **only** from data any observer can read, plus Bob's own secrets. If
//! anything about Alice were required, this file would not compile.

use lee_core::encryption::ViewingPublicKey;
use pmsig_core::{approval_nullifier, tree::MemberTree, Digest32};
use pmsig_membership_core::verify::npk_of;
use pmsig_multisig_core::{
    logic::{approve, create_multisig, create_proposal, CreateMultisig},
    ProgramIdWords, ProposedAction,
};
use pmsig_sdk::member::{prepare_approval, MemberSecrets, MultisigView};

const VERIFIER: ProgramIdWords = [7; 8];
const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];
const ALICE: Digest32 = [0x11; 32];
const BOB: Digest32 = [0x22; 32];
const CAROL: Digest32 = [0x33; 32];

fn vpk() -> ViewingPublicKey {
    ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32])
}

fn member_tree() -> MemberTree {
    let npks: Vec<Digest32> = [ALICE, BOB, CAROL]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();
    MemberTree::new(&npks).expect("three members")
}

/// Bob's own secrets. Note the absence of anything belonging to Alice or Carol.
fn bob_secrets(tree: &MemberTree) -> MemberSecrets {
    MemberSecrets {
        nsk: BOB,
        vpk: vpk(),
        identifier: 0,
        path: tree.path(1).expect("bob is member 1"),
    }
}

/// **The SC-D.5 assertion.**
///
/// Alice approves first. Bob then approves using a view assembled purely from chain data — the
/// config account's `member_root`/`M`/`N`, the multisig id, and the proposal's approval **count**.
/// He never sees Alice's account id, her npk, or the fact that it was Alice rather than Carol.
#[test]
fn a_co_member_approves_without_learning_who_approved_first() {
    let tree = member_tree();
    let params = CreateMultisig {
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: VERIFIER,
    };
    let (config, seed) = create_multisig(&params).unwrap();
    let (mut proposal, _) = create_proposal(
        &config,
        &seed,
        PROPOSAL_ID,
        ProposedAction::TreasuryTransfer {
            recipient: [0xC3; 32],
            amount: 1_000,
        },
    )
    .unwrap();

    // --- Alice approves. Bob is not told, and is not involved. ---
    let alice_claim = pmsig_membership_core::ApprovalClaim {
        multisig_id: MULTISIG_ID,
        proposal_id: PROPOSAL_ID,
        member_root: tree.root(),
        claimed_nullifier: approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID),
    };
    approve(&config, &seed, &mut proposal, &alice_claim, &VERIFIER).unwrap();

    // --- Everything Bob is allowed to know, read off the chain ---
    let view = MultisigView {
        multisig_id: config.multisig_id,
        member_root: config.member_root,
        m: config.m,
        n: config.n,
        approvals_on_chain: proposal.approvals(), // a count, not a list
    };
    assert_eq!(view.approvals_on_chain, 1);
    assert_eq!(view.remaining(), 1);

    // Bob prepares his approval from that view plus his own secrets, and nothing else.
    let prepared = prepare_approval(
        &view,
        &bob_secrets(&tree),
        PROPOSAL_ID,
        &proposal.nullifiers,
    )
    .expect("bob can approve");

    // What Bob holds must contain nothing of Alice's.
    let alice_npk = npk_of(&ALICE).to_byte_array();
    let alice_nf = approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID);
    assert_ne!(prepared.witness.nsk, ALICE);
    assert_ne!(prepared.claim.claimed_nullifier, alice_nf);
    for sibling in &prepared.witness.siblings {
        assert_ne!(
            *sibling, alice_npk,
            "a raw member npk must never appear in a sibling path"
        );
    }

    approve(&config, &seed, &mut proposal, &prepared.claim, &VERIFIER).expect("bob approves");
    assert_eq!(proposal.approvals(), 2);
    assert!(proposal.threshold_met(config.m));
}

/// The count Bob reads tells him *how many*, never *which*. Two different pairs of approvers produce
/// an on-chain record that is identical in everything except the nullifier values themselves — and
/// those are preimage-hiding.
#[test]
fn the_on_chain_record_does_not_distinguish_which_members_approved() {
    let tree = member_tree();
    let params = CreateMultisig {
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: VERIFIER,
    };
    let (config, seed) = create_multisig(&params).unwrap();

    let run = |a: Digest32, b: Digest32| {
        let (mut p, _) = create_proposal(
            &config,
            &seed,
            PROPOSAL_ID,
            ProposedAction::TreasuryTransfer {
                recipient: [0xC3; 32],
                amount: 1_000,
            },
        )
        .unwrap();
        for nsk in [a, b] {
            let claim = pmsig_membership_core::ApprovalClaim {
                multisig_id: MULTISIG_ID,
                proposal_id: PROPOSAL_ID,
                member_root: tree.root(),
                claimed_nullifier: approval_nullifier(&nsk, &MULTISIG_ID, &PROPOSAL_ID),
            };
            approve(&config, &seed, &mut p, &claim, &VERIFIER).unwrap();
        }
        p
    };

    let alice_bob = run(ALICE, BOB);
    let bob_carol = run(BOB, CAROL);

    // Same shape, same count, same executed flag: the record's *structure* reveals nothing.
    assert_eq!(alice_bob.approvals(), bob_carol.approvals());
    assert_eq!(alice_bob.executed, bob_carol.executed);
    assert_eq!(alice_bob.proposal_id, bob_carol.proposal_id);
    // The nullifiers differ, but no member id can be recovered from them.
    assert_ne!(alice_bob.nullifiers, bob_carol.nullifiers);
    for nsk in [ALICE, BOB, CAROL] {
        let npk = npk_of(&nsk).to_byte_array();
        for nf in alice_bob
            .nullifiers
            .iter()
            .chain(bob_carol.nullifiers.iter())
        {
            assert_ne!(*nf, npk, "a nullifier must not be a member npk");
            assert_ne!(*nf, nsk, "a nullifier must not be a member secret");
        }
    }
}

/// A member is told locally that they have already approved, before paying for a proof (error 2006).
#[test]
fn a_repeat_approval_is_refused_locally_before_proving() {
    let tree = member_tree();
    let view = MultisigView {
        multisig_id: MULTISIG_ID,
        member_root: tree.root(),
        m: 2,
        n: 3,
        approvals_on_chain: 1,
    };
    let bob_nf = approval_nullifier(&BOB, &MULTISIG_ID, &PROPOSAL_ID);
    let err = prepare_approval(&view, &bob_secrets(&tree), PROPOSAL_ID, &[bob_nf]).unwrap_err();
    assert!(err.to_string().starts_with("2006 "), "got {err}");
}

/// Once the threshold is met there is nothing to approve; the client says so rather than proving.
#[test]
fn approving_a_satisfied_proposal_is_refused_locally() {
    let tree = member_tree();
    let view = MultisigView {
        multisig_id: MULTISIG_ID,
        member_root: tree.root(),
        m: 2,
        n: 3,
        approvals_on_chain: 2,
    };
    let err = prepare_approval(&view, &bob_secrets(&tree), PROPOSAL_ID, &[]).unwrap_err();
    assert!(err.to_string().starts_with("2007 "), "got {err}");
}

/// A stray debug print must not leak a spending key.
#[test]
fn member_secrets_are_redacted_in_debug_output() {
    let tree = member_tree();
    let rendered = format!("{:?}", bob_secrets(&tree));
    assert!(rendered.contains("<redacted>"));
    assert!(
        !rendered.contains("22, 22, 22"),
        "the nsk must not appear in Debug output: {rendered}"
    );
}
