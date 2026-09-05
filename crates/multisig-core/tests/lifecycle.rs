#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! Phase C lifecycle: create → propose → approve×M → execute, and every documented rejection.
//!
//! These drive the pure transition functions, so the whole suite runs in milliseconds. The zkVM
//! layer is tested separately (`crates/sdk/tests/prove_membership.rs`): here the question is what the
//! *chain* decides once a proof has been verified, which is a different set of rules.

use pmsig_core::{approval_nullifier, tree::MemberTree, Digest32};
use pmsig_membership_core::{verify::npk_of, ApprovalClaim};
use pmsig_multisig_core::{
    logic::{approve, create_multisig, create_proposal, execute, CreateMultisig},
    MultisigError, ProgramIdWords, Proposal, ProposedAction,
};

const VERIFIER: ProgramIdWords = [7; 8];
const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];
const RECIPIENT: Digest32 = [0xC3; 32];

const ALICE: Digest32 = [0x11; 32];
const BOB: Digest32 = [0x22; 32];
const CAROL: Digest32 = [0x33; 32];

/// The default configuration the prize asks for: a 2-of-3 treasury multisig.
struct Fixture {
    config: pmsig_multisig_core::MultisigConfig,
    config_seed: Digest32,
    proposal: Proposal,
    tree: MemberTree,
}

fn params() -> CreateMultisig {
    let npks: Vec<Digest32> = [ALICE, BOB, CAROL]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();
    let tree = MemberTree::new(&npks).expect("three members");
    CreateMultisig {
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: VERIFIER,
    }
}

fn fixture() -> Fixture {
    let p = params();
    let npks: Vec<Digest32> = [ALICE, BOB, CAROL]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();
    let tree = MemberTree::new(&npks).expect("three members");
    let (config, config_seed) = create_multisig(&p).expect("valid 2-of-3");
    let (proposal, _seed) = create_proposal(
        &config,
        &config_seed,
        PROPOSAL_ID,
        ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1_000,
        },
    )
    .expect("valid proposal");
    Fixture {
        config,
        config_seed,
        proposal,
        tree,
    }
}

/// The claim a member's approval carries. In production the membership guest recomputes the
/// nullifier and refuses a mismatch; here we build the honest value.
fn claim_of(nsk: &Digest32, member_root: Digest32) -> ApprovalClaim {
    ApprovalClaim {
        multisig_id: MULTISIG_ID,
        proposal_id: PROPOSAL_ID,
        member_root,
        claimed_nullifier: approval_nullifier(nsk, &MULTISIG_ID, &PROPOSAL_ID),
    }
}

// ---------------------------------------------------------------------------------------------
// SC-C.2 — the happy path, at FULL M (plan gate H13/W15: the primary evidence path is never a
// lowered threshold).
// ---------------------------------------------------------------------------------------------

#[test]
fn the_full_lifecycle_completes_at_full_m() {
    let mut f = fixture();
    let root = f.tree.root();
    assert_eq!(f.config.m, 2, "the reference configuration is 2-of-3");
    assert_eq!(f.proposal.approvals(), 0);

    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&ALICE, root),
        &VERIFIER,
    )
    .expect("alice approves");
    assert_eq!(f.proposal.approvals(), 1);

    // One short of the threshold: execution must still refuse.
    assert_eq!(
        execute(&f.config, &f.config_seed, &mut f.proposal).unwrap_err(),
        MultisigError::ThresholdNotMet
    );

    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&BOB, root),
        &VERIFIER,
    )
    .expect("bob approves");
    assert_eq!(f.proposal.approvals(), 2);

    let action = execute(&f.config, &f.config_seed, &mut f.proposal).expect("threshold reached");
    assert_eq!(
        action,
        ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1_000
        }
    );
    assert!(f.proposal.executed);
}

// ---------------------------------------------------------------------------------------------
// SC-C.3 — double approve
// ---------------------------------------------------------------------------------------------

/// **P-F3 / INV-4.** The same member approving twice yields the same nullifier and is rejected.
#[test]
fn a_member_cannot_approve_the_same_proposal_twice() {
    let mut f = fixture();
    let root = f.tree.root();
    let claim = claim_of(&ALICE, root);
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim,
        &VERIFIER,
    )
    .expect("first approval");
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim,
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::DuplicateNullifier
    );
    assert_eq!(
        f.proposal.approvals(),
        1,
        "the rejected approval left no trace"
    );
}

/// A member cannot dodge the check by approving from another of their 2^128 addresses: the
/// nullifier is keyed to `nsk`, not to an account id (ADR-001 D5).
#[test]
fn a_member_cannot_double_vote_from_another_of_their_addresses() {
    let mut f = fixture();
    let root = f.tree.root();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&ALICE, root),
        &VERIFIER,
    )
    .expect("first approval");
    // Same nsk, so the same nullifier, regardless of which address was spent.
    let again = claim_of(&ALICE, root);
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &again,
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::DuplicateNullifier
    );
}

// ---------------------------------------------------------------------------------------------
// SC-C.4 — early execute
// ---------------------------------------------------------------------------------------------

#[test]
fn executing_before_the_threshold_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    assert_eq!(
        execute(&f.config, &f.config_seed, &mut f.proposal).unwrap_err(),
        MultisigError::ThresholdNotMet
    );
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&ALICE, root),
        &VERIFIER,
    )
    .expect("one approval");
    assert_eq!(
        execute(&f.config, &f.config_seed, &mut f.proposal).unwrap_err(),
        MultisigError::ThresholdNotMet
    );
    assert!(!f.proposal.executed);
}

#[test]
fn executing_twice_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&ALICE, root),
        &VERIFIER,
    )
    .unwrap();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&BOB, root),
        &VERIFIER,
    )
    .unwrap();
    execute(&f.config, &f.config_seed, &mut f.proposal).expect("first execution");
    assert_eq!(
        execute(&f.config, &f.config_seed, &mut f.proposal).unwrap_err(),
        MultisigError::AlreadyExecuted
    );
}

#[test]
fn approving_after_execution_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&ALICE, root),
        &VERIFIER,
    )
    .unwrap();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&BOB, root),
        &VERIFIER,
    )
    .unwrap();
    execute(&f.config, &f.config_seed, &mut f.proposal).unwrap();
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim_of(&CAROL, root),
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::ProposalClosed
    );
}

// ---------------------------------------------------------------------------------------------
// SC-C.5 / SC-C.8 — invalid proof, and the absence of a public approve path
// ---------------------------------------------------------------------------------------------

/// **ADR-002 / error 1013.** An approval vouched for by a program this multisig is not bound to is
/// rejected — this is what stops an attacker substituting a permissive "membership" program.
#[test]
fn an_approval_from_the_wrong_verifier_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    let hostile: ProgramIdWords = [0xDEAD_BEEF; 8];
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim_of(&ALICE, root),
            &hostile
        )
        .unwrap_err(),
        MultisigError::WrongMembershipProgram
    );
    assert_eq!(f.proposal.approvals(), 0);
}

/// **SC-C.8 / H9.** There is no public approve path to reject at runtime, because `approve` cannot
/// be reached without a verified chained call — so the guarantee is asserted structurally instead.
///
/// `approve` requires a `verified_by` program id, and the only caller that can supply one truthfully
/// is the program dispatcher, which sets it from the chained call it actually made. A transaction
/// that carried no chained call has no id to pass.
///
/// There is deliberately **no** error code for this case. An earlier revision carried a
/// `PublicApprovePathRejected` (1011) variant, but nothing could ever return it — the condition it
/// described is unrepresentable, not merely rejected — and the only test of it asserted the
/// constant against itself. It was retired rather than left in the catalogue as an error the
/// program claims to raise and never does. What is asserted below is the real guarantee: a
/// verifier id that names no bound program is refused.
#[test]
fn there_is_no_public_approve_path() {
    // The signature itself is the guarantee: there is no `approve` overload without `verified_by`.
    let mut f = fixture();
    let root = f.tree.root();
    let hostile: ProgramIdWords = [0; 8];
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim_of(&ALICE, root),
            &hostile
        )
        .unwrap_err(),
        MultisigError::WrongMembershipProgram,
        "a zero/absent verifier id must be refused as an unbound verifier, not merely fail"
    );
}

#[test]
fn an_approval_naming_another_multisig_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    let mut claim = claim_of(&ALICE, root);
    claim.multisig_id = [0xFF; 32];
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim,
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::UnknownProposal
    );
}

#[test]
fn an_approval_naming_another_proposal_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    let mut claim = claim_of(&ALICE, root);
    claim.proposal_id = [0xFF; 32];
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim,
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::UnknownProposal
    );
}

/// **INV-5.** An approval proved against a stale member root is rejected.
#[test]
fn an_approval_against_a_stale_member_root_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    let mut claim = claim_of(&ALICE, root);
    claim.member_root = [0xFF; 32];
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim,
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::MemberRootMismatch
    );
}

// ---------------------------------------------------------------------------------------------
// SC-C.7 — a valid proof against a wrong config_hash or a lowered M must fail
// ---------------------------------------------------------------------------------------------

/// **INV-1.** A prover who lowers `M` does not weaken the multisig — they name one that does not
/// exist. Here the config account is presented at the honest address with a lowered threshold, and
/// the rehash catches it.
#[test]
fn a_lowered_threshold_does_not_match_the_address() {
    let mut f = fixture();
    let root = f.tree.root();
    let honest_seed = f.config_seed;
    f.config.m = 1; // attacker claims 1-of-3
    assert_eq!(
        execute(&f.config, &honest_seed, &mut f.proposal).unwrap_err(),
        MultisigError::ConfigHashMismatch,
        "INV-3 must catch a config account that no longer attests to its own address"
    );
    assert_eq!(
        approve(
            &f.config,
            &honest_seed,
            &mut f.proposal,
            &claim_of(&ALICE, root),
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::ConfigHashMismatch
    );
}

/// **INV-2.** Same for substituting the member set.
#[test]
fn a_substituted_member_set_does_not_match_the_address() {
    let mut f = fixture();
    let root = f.tree.root();
    let honest_seed = f.config_seed;
    f.config.member_root = [0xAA; 32];
    assert_eq!(
        approve(
            &f.config,
            &honest_seed,
            &mut f.proposal,
            &claim_of(&ALICE, root),
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::ConfigHashMismatch
    );
}

/// **ADR-002.** And for substituting the verifier: the address changes, so the multisig is not there.
#[test]
fn a_substituted_verifier_does_not_match_the_address() {
    let mut f = fixture();
    let root = f.tree.root();
    let honest_seed = f.config_seed;
    f.config.membership_program_id = [0xDEAD_BEEF; 8];
    assert_eq!(
        approve(
            &f.config,
            &honest_seed,
            &mut f.proposal,
            &claim_of(&ALICE, root),
            &[0xDEAD_BEEF; 8]
        )
        .unwrap_err(),
        MultisigError::ConfigHashMismatch
    );
}

#[test]
fn a_proposal_from_another_multisig_is_rejected() {
    let mut f = fixture();
    let root = f.tree.root();
    f.proposal.config_hash = [0xEE; 32];
    assert_eq!(
        approve(
            &f.config,
            &f.config_seed,
            &mut f.proposal,
            &claim_of(&ALICE, root),
            &VERIFIER
        )
        .unwrap_err(),
        MultisigError::UnknownProposal
    );
}

// ---------------------------------------------------------------------------------------------
// Creation-time validation
// ---------------------------------------------------------------------------------------------

#[test]
fn nonsensical_thresholds_are_rejected_at_creation() {
    for (m, n) in [(0_u8, 3_u8), (3, 0), (4, 3)] {
        let mut p = params();
        p.m = m;
        p.n = n;
        assert_eq!(
            create_multisig(&p).unwrap_err(),
            MultisigError::InvalidThresholdConfig,
            "{m}-of-{n} must be refused"
        );
    }
}

#[test]
fn a_zero_value_transfer_is_rejected() {
    let (config, seed) = create_multisig(&params()).unwrap();
    assert_eq!(
        create_proposal(
            &config,
            &seed,
            PROPOSAL_ID,
            ProposedAction::TreasuryTransfer {
                recipient: RECIPIENT,
                amount: 0
            }
        )
        .unwrap_err(),
        MultisigError::InvalidProposalAction
    );
}

/// A 1-of-1 is legitimate and must work — the threshold logic should not special-case the default.
#[test]
fn a_one_of_one_multisig_works() {
    let npks = vec![npk_of(&ALICE).to_byte_array()];
    let tree = MemberTree::new(&npks).expect("one member");
    let p = CreateMultisig {
        member_root: tree.root(),
        m: 1,
        n: 1,
        multisig_id: MULTISIG_ID,
        membership_program_id: VERIFIER,
    };
    let (config, seed) = create_multisig(&p).expect("1-of-1 is valid");
    let (mut proposal, _) = create_proposal(
        &config,
        &seed,
        PROPOSAL_ID,
        ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 5,
        },
    )
    .unwrap();
    let claim = ApprovalClaim {
        multisig_id: MULTISIG_ID,
        proposal_id: PROPOSAL_ID,
        member_root: tree.root(),
        claimed_nullifier: approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID),
    };
    approve(&config, &seed, &mut proposal, &claim, &VERIFIER).unwrap();
    execute(&config, &seed, &mut proposal).expect("1-of-1 executes on one approval");
}

/// **SC-C.6 / P-F2.** After a completed 2-of-3, the on-chain record shows the threshold was met and
/// says nothing about who met it.
#[test]
fn the_executed_state_records_a_threshold_and_no_identities() {
    let mut f = fixture();
    let root = f.tree.root();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&ALICE, root),
        &VERIFIER,
    )
    .unwrap();
    approve(
        &f.config,
        &f.config_seed,
        &mut f.proposal,
        &claim_of(&BOB, root),
        &VERIFIER,
    )
    .unwrap();
    execute(&f.config, &f.config_seed, &mut f.proposal).unwrap();

    // What an observer can read off the proposal account:
    assert_eq!(f.proposal.approvals(), 2);
    assert!(f.proposal.executed);

    // What they cannot: any member's npk, or an account id, anywhere in the encoded state.
    let encoded = borsh::to_vec(&f.proposal).expect("encodes");
    for nsk in [ALICE, BOB, CAROL] {
        let npk = npk_of(&nsk).to_byte_array();
        assert!(
            !encoded.windows(32).any(|w| w == npk),
            "a member npk leaked into the proposal account"
        );
        assert!(
            !encoded.windows(32).any(|w| w == nsk),
            "a member secret leaked into the proposal account"
        );
    }
    // Carol never approved, and her nullifier must not be present either.
    let carol_nf = approval_nullifier(&CAROL, &MULTISIG_ID, &PROPOSAL_ID);
    assert!(!f.proposal.has_nullifier(&carol_nf));
}
