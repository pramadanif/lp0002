//! Phase B proving tests: an honest approval proves and verifies; invalid ones are rejected.
//!
//! The honest-path test (`an_honest_approval_proves_and_verifies`) generates a **real** proof with
//! `RISC0_DEV_MODE=0`. It is `#[ignore]`d by default because proving takes far longer than a unit
//! test should; `scripts/prove-bench.sh` runs it and records the timing for **SC-B.3 / P-F5**.
//!
//! Negative cases use the executor rather than the prover: an invalid witness must be *rejected*,
//! and spending minutes proving something that has to fail buys nothing. Rejection happens in the
//! guest either way — the executor runs the same code.

use lee_core::{
    account::{Account, AccountId, AccountWithMetadata},
    encryption::ViewingPublicKey,
    program::ProgramId,
};
use pmsig_core::{approval_nullifier, tree::MemberTree, Digest32};
use pmsig_membership_core::{
    verify::{derive_account_id, npk_of},
    ApproveWitness,
};
use pmsig_sdk::prove::{dev_mode_enabled, execute_approval, prove_approval};

const SELF_PROGRAM_ID: ProgramId = [7; 8];
const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];

const ALICE_NSK: Digest32 = [0x11; 32];
const BOB_NSK: Digest32 = [0x22; 32];
const CAROL_NSK: Digest32 = [0x33; 32];

fn program_binary() -> Vec<u8> {
    let path = std::env::var("PMSIG_MEMBERSHIP_BIN")
        .unwrap_or_else(|_| "../../artifacts/membership.bin".to_string());
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!("cannot read guest binary at {path}: {e}\nRun ./scripts/build-guests.sh first.")
    })
}

fn vpk() -> ViewingPublicKey {
    ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32])
}

/// A 2-of-3 multisig with Alice, Bob and Carol; Alice approves.
fn alice_approves() -> (ApproveWitness, Vec<AccountWithMetadata>) {
    let npks: Vec<Digest32> = [ALICE_NSK, BOB_NSK, CAROL_NSK]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();
    let tree = MemberTree::new(&npks).expect("three members");
    let path = tree.path(0).expect("alice is a member");
    let alice_account_id = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 0);

    let witness = ApproveWitness {
        nsk: ALICE_NSK,
        vpk: vpk(),
        identifier: 0,
        member_index: 0,
        siblings: path.siblings,
        multisig_id: MULTISIG_ID,
        proposal_id: PROPOSAL_ID,
        member_root: tree.root(),
        claimed_nullifier: approval_nullifier(&ALICE_NSK, &MULTISIG_ID, &PROPOSAL_ID),
    };

    let pre_states = vec![AccountWithMetadata::new(
        Account::default(),
        true,
        alice_account_id,
    )];
    (witness, pre_states)
}

/// SC-B.1 and SC-B.3 — a real proof, generated and verified with `RISC0_DEV_MODE=0`.
#[test]
#[ignore = "generates a real proof; run via scripts/prove-bench.sh"]
fn an_honest_approval_proves_and_verifies() {
    assert!(
        !dev_mode_enabled(),
        "SC-B.3 requires RISC0_DEV_MODE=0: a dev-mode receipt proves nothing"
    );

    let (witness, pre_states) = alice_approves();
    let binary = program_binary();

    let proof = prove_approval(&binary, SELF_PROGRAM_ID, None, &pre_states, &witness, false)
        .expect("honest approval must prove");

    // prove_approval verifies the receipt against the image id before returning, so reaching here
    // means the proof is valid for this exact guest.
    println!("PROVE_SECONDS={:.3}", proof.prove_time.as_secs_f64());
    println!("IMAGE_ID_WORDS={:?}", proof.image_id);
    println!("JOURNAL_BYTES={}", proof.receipt.journal.bytes.len());
}

/// SC-B.4 — the journal must not carry the member's identity in the clear.
#[test]
#[ignore = "generates a real proof; run via scripts/prove-bench.sh"]
fn the_journal_reveals_no_member_identity() {
    let (witness, pre_states) = alice_approves();
    let binary = program_binary();
    let proof = prove_approval(&binary, SELF_PROGRAM_ID, None, &pre_states, &witness, false)
        .expect("honest approval must prove");

    let journal = &proof.receipt.journal.bytes;
    let npk = npk_of(&ALICE_NSK).to_byte_array();

    assert!(
        !contains(journal, &witness.nsk),
        "SC-B.4: the member's nsk appears in the journal"
    );
    assert!(
        !contains(journal, &npk),
        "SC-B.4: the member's npk appears in the journal"
    );
    assert!(
        !contains(journal, witness.vpk.to_bytes()),
        "SC-B.4: the member's viewing key appears in the journal"
    );
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// SC-B.2 negatives. Executed rather than proved — rejection is the same guest code either way.
mod negatives {
    use super::*;

    fn run(witness: &ApproveWitness, pre_states: &[AccountWithMetadata]) -> Result<u64, String> {
        execute_approval(
            &program_binary(),
            SELF_PROGRAM_ID,
            None,
            pre_states,
            witness,
        )
        .map_err(|e| e.to_string())
    }

    /// Also reports the cycle count, which is what sets proving time (P-F5) and, later, CU cost.
    #[test]
    fn an_honest_approval_executes() {
        let (w, pre) = alice_approves();
        let cycles = run(&w, &pre).expect("honest approval must execute");
        println!("GUEST_CYCLES={cycles}");
        assert!(cycles > 0);
    }

    #[test]
    fn a_wrong_member_root_is_rejected() {
        let (mut w, pre) = alice_approves();
        w.member_root = [0xFF; 32];
        assert!(run(&w, &pre).is_err());
    }

    #[test]
    fn a_wrong_proposal_is_rejected() {
        let (mut w, pre) = alice_approves();
        // The claimed nullifier no longer matches the proposal it names.
        w.proposal_id = [0xC3; 32];
        assert!(run(&w, &pre).is_err());
    }

    #[test]
    fn a_forged_nullifier_is_rejected() {
        let (mut w, pre) = alice_approves();
        w.claimed_nullifier = [0xAB; 32];
        assert!(run(&w, &pre).is_err());
    }

    #[test]
    fn a_non_member_is_rejected() {
        let (mut w, _) = alice_approves();
        let outsider: Digest32 = [0x99; 32];
        w.nsk = outsider;
        w.claimed_nullifier = approval_nullifier(&outsider, &MULTISIG_ID, &PROPOSAL_ID);
        let account_id = derive_account_id(&npk_of(&outsider), &vpk(), 0);
        let pre = vec![AccountWithMetadata::new(
            Account::default(),
            true,
            account_id,
        )];
        assert!(run(&w, &pre).is_err());
    }

    /// H8 in the guest: a witness that does not control the presented account is rejected.
    #[test]
    fn an_unbound_account_is_rejected() {
        let (w, _) = alice_approves();
        let unrelated = derive_account_id(&npk_of(&[0xEE; 32]), &vpk(), 0);
        let pre = vec![AccountWithMetadata::new(
            Account::default(),
            true,
            unrelated,
        )];
        assert!(
            run(&w, &pre).is_err(),
            "H8 REGRESSION: the guest accepted a witness that does not control the account"
        );
    }

    #[test]
    fn a_member_using_a_different_address_is_rejected() {
        let (w, _) = alice_approves();
        // Alice's identifier-1 address, while the witness claims identifier 0.
        let other = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 1);
        let pre = vec![AccountWithMetadata::new(Account::default(), true, other)];
        assert!(run(&w, &pre).is_err());
    }

    #[test]
    fn a_missing_approver_account_is_rejected() {
        let (w, _) = alice_approves();
        assert!(run(&w, &[]).is_err(), "guest must require pre_states[0]");
    }

    /// Sanity: the fixture used by the unrelated-account test is genuinely a different account.
    #[test]
    fn the_fixture_accounts_are_distinct() {
        let alice = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 0);
        let unrelated: AccountId = derive_account_id(&npk_of(&[0xEE; 32]), &vpk(), 0);
        assert_ne!(alice, unrelated);
    }
}
