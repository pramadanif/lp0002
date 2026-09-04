#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]

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
    ApprovalClaim, ApprovalWitness,
};
use pmsig_sdk::prove::{
    dev_mode_enabled, execute_approval, execute_approval_journal, prove_approval,
};

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
fn alice_approves() -> (ApprovalClaim, ApprovalWitness, Vec<AccountWithMetadata>) {
    let npks: Vec<Digest32> = [ALICE_NSK, BOB_NSK, CAROL_NSK]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();
    let tree = MemberTree::new(&npks).expect("three members");
    let path = tree.path(0).expect("alice is a member");
    let alice_account_id = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 0);

    let witness = ApprovalWitness {
        nsk: ALICE_NSK,
        vpk: vpk(),
        identifier: 0,
        member_index: 0,
        siblings: path.siblings,
    };
    let claim = ApprovalClaim {
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
    (claim, witness, pre_states)
}

/// SC-B.1 and SC-B.3 — a real proof, generated and verified with `RISC0_DEV_MODE=0`.
#[test]
#[ignore = "generates a real proof; run via scripts/prove-bench.sh"]
fn an_honest_approval_proves_and_verifies() {
    assert!(
        !dev_mode_enabled(),
        "SC-B.3 requires RISC0_DEV_MODE=0: a dev-mode receipt proves nothing"
    );

    let (claim, witness, pre_states) = alice_approves();
    let binary = program_binary();

    let proof = prove_approval(
        &binary,
        SELF_PROGRAM_ID,
        None,
        &pre_states,
        &claim,
        &witness,
        false,
    )
    .expect("honest approval must prove");

    // prove_approval verifies the receipt against the image id before returning, so reaching here
    // means the proof is valid for this exact guest.
    println!("PROVE_SECONDS={:.3}", proof.prove_time.as_secs_f64());
    println!("IMAGE_ID_WORDS={:?}", proof.image_id);
    println!("JOURNAL_BYTES={}", proof.receipt.journal.bytes.len());
}

/// The proved receipt's journal must satisfy the same rule as the executed one.
#[test]
#[ignore = "generates a real proof; run via scripts/prove-bench.sh"]
fn the_proved_journal_carries_no_member_secret() {
    let (claim, witness, pre_states) = alice_approves();
    let binary = program_binary();
    let proof = prove_approval(
        &binary,
        SELF_PROGRAM_ID,
        None,
        &pre_states,
        &claim,
        &witness,
        false,
    )
    .expect("honest approval must prove");

    let journal = &proof.receipt.journal.bytes;
    let npk = npk_of(&ALICE_NSK).to_byte_array();

    assert!(
        !contains(journal, &word_encode(&witness.nsk)),
        "SC-B.4: the member's nsk appears in the proved journal"
    );
    assert!(
        !contains(journal, &word_encode(&npk)),
        "SC-B.4: the member's npk appears in the proved journal"
    );
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// SC-B.2 negatives. Executed rather than proved — rejection is the same guest code either way.
mod negatives {
    use super::*;

    /// Runs the guest and returns the cycle count, or the guest's **rejection** message.
    ///
    /// Distinguishing a rejection from an infrastructure failure matters: without this, every
    /// negative test passes whenever `r0vm` is missing, because "prover not found" is also an
    /// `Err`. CI hit exactly that — the binary was committed but `r0vm` was not installed, and the
    /// negatives reported green against a prover that never ran. A test that passes when the thing
    /// under test is absent is worse than no test (gate H2 in spirit).
    fn run(
        claim: &ApprovalClaim,
        witness: &ApprovalWitness,
        pre_states: &[AccountWithMetadata],
    ) -> Result<u64, String> {
        let result = execute_approval(
            &program_binary(),
            SELF_PROGRAM_ID,
            None,
            pre_states,
            claim,
            witness,
        );
        if let Err(e) = &result {
            let msg = e.to_string();
            assert!(
                !looks_like_missing_prover(&msg),
                "the prover did not run at all ({msg}). This is an environment failure, not a \
                 guest rejection — install r0vm (`rzup install`) rather than reading this as a pass."
            );
        }
        result.map_err(|e| e.to_string())
    }

    /// Heuristic for "the executor never started", as opposed to "the guest panicked".
    fn looks_like_missing_prover(msg: &str) -> bool {
        let m = msg.to_ascii_lowercase();
        m.contains("no such file or directory")
            || m.contains("not found")
            || m.contains("permission denied")
            || m.contains("failed to spawn")
    }

    /// Also reports the cycle count, which is what sets proving time (P-F5) and, later, CU cost.
    #[test]
    fn an_honest_approval_executes() {
        let (c, w, pre) = alice_approves();
        let cycles = run(&c, &w, &pre).expect("honest approval must execute");
        println!("GUEST_CYCLES={cycles}");
        assert!(cycles > 0);
    }

    #[test]
    fn a_wrong_member_root_is_rejected() {
        let (mut c, w, pre) = alice_approves();
        c.member_root = [0xFF; 32];
        assert!(run(&c, &w, &pre).is_err());
    }

    #[test]
    fn a_wrong_proposal_is_rejected() {
        let (mut c, w, pre) = alice_approves();
        // The claimed nullifier no longer matches the proposal it names.
        c.proposal_id = [0xC3; 32];
        assert!(run(&c, &w, &pre).is_err());
    }

    #[test]
    fn a_forged_nullifier_is_rejected() {
        let (mut c, w, pre) = alice_approves();
        c.claimed_nullifier = [0xAB; 32];
        assert!(run(&c, &w, &pre).is_err());
    }

    #[test]
    fn a_non_member_is_rejected() {
        let (mut c, mut w, _) = alice_approves();
        let outsider: Digest32 = [0x99; 32];
        w.nsk = outsider;
        c.claimed_nullifier = approval_nullifier(&outsider, &MULTISIG_ID, &PROPOSAL_ID);
        let account_id = derive_account_id(&npk_of(&outsider), &vpk(), 0);
        let pre = vec![AccountWithMetadata::new(
            Account::default(),
            true,
            account_id,
        )];
        assert!(run(&c, &w, &pre).is_err());
    }

    /// H8 in the guest: a witness that does not control the presented account is rejected.
    #[test]
    fn an_unbound_account_is_rejected() {
        let (c, w, _) = alice_approves();
        let unrelated = derive_account_id(&npk_of(&[0xEE; 32]), &vpk(), 0);
        let pre = vec![AccountWithMetadata::new(
            Account::default(),
            true,
            unrelated,
        )];
        assert!(
            run(&c, &w, &pre).is_err(),
            "H8 REGRESSION: the guest accepted a witness that does not control the account"
        );
    }

    #[test]
    fn a_member_using_a_different_address_is_rejected() {
        let (c, w, _) = alice_approves();
        // Alice's identifier-1 address, while the witness claims identifier 0.
        let other = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 1);
        let pre = vec![AccountWithMetadata::new(Account::default(), true, other)];
        assert!(run(&c, &w, &pre).is_err());
    }

    #[test]
    fn a_missing_approver_account_is_rejected() {
        let (c, w, _) = alice_approves();
        assert!(
            run(&c, &w, &[]).is_err(),
            "guest must require pre_states[0]"
        );
    }

    /// Sanity: the fixture used by the unrelated-account test is genuinely a different account.
    #[test]
    fn the_fixture_accounts_are_distinct() {
        let alice = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 0);
        let unrelated: AccountId = derive_account_id(&npk_of(&[0xEE; 32]), &vpk(), 0);
        assert_ne!(alice, unrelated);
    }
}

/// **SC-B.4** — the member's secrets must not reach the guest's journal.
///
/// Checked by decoding the journal, not by scanning it. A raw byte scan is a **false negative**
/// here: risc0's serde writes each `u8` as its own 32-bit word, so a 32-byte secret occupies 128
/// journal bytes and never appears as a contiguous run. An earlier version of this test scanned raw
/// bytes, reported "clean", and was wrong — the witness was fully recoverable. See
/// `docs/tried-failed.md`.
///
/// Executed rather than proved: the journal is identical either way, and this needs to run in CI.
#[test]
fn the_journal_carries_no_member_secret() {
    let (claim, witness, pre_states) = alice_approves();
    let (_cycles, journal) = execute_approval_journal(
        &program_binary(),
        SELF_PROGRAM_ID,
        None,
        &pre_states,
        &claim,
        &witness,
    )
    .expect("honest approval executes");

    let npk = npk_of(&ALICE_NSK).to_byte_array();

    // Word-encoded scan: the form the secrets would actually take if committed.
    assert!(
        !contains(&journal, &word_encode(&witness.nsk)),
        "SC-B.4: the member's nsk is in the guest journal"
    );
    assert!(
        !contains(&journal, &word_encode(&npk)),
        "SC-B.4: the member's npk is in the guest journal"
    );
    assert!(
        !contains(&journal, &word_encode(witness.vpk.to_bytes())),
        "SC-B.4: the member's viewing key is in the guest journal"
    );
    for sibling in &witness.siblings {
        assert!(
            !contains(&journal, &word_encode(sibling)),
            "SC-B.4: a Merkle sibling is in the guest journal"
        );
    }

    // Decode it properly: instruction_data must be the public claim and nothing more.
    let output: lee_core::program::ProgramOutput =
        risc0_zkvm::serde::from_slice(&journal).expect("journal decodes as ProgramOutput");
    let recovered: pmsig_membership_core::Instruction =
        risc0_zkvm::serde::from_slice(&output.instruction_data)
            .expect("instruction_data decodes as our Instruction");
    let pmsig_membership_core::Instruction::VerifyApproval(recovered_claim) = recovered;
    assert_eq!(
        recovered_claim, claim,
        "the committed instruction must be exactly the public claim"
    );
}

/// What the journal *does* contain, asserted so the docs cannot drift from reality.
///
/// The approver's `account_id` is in `pre_states`, and that is unavoidable: every LEZ program
/// commits its pre/post states, which is how the runtime validates execution. It is not a leak of
/// this design — the inner `ProgramOutput` never reaches the chain. Only
/// `PrivacyPreservingCircuitOutput` does, and that carries just nullifiers, commitments and
/// ciphertext (`lee/state_machine/core/src/circuit_io.rs:156-180`).
///
/// The consequence, recorded in `docs/security.md`: **the inner receipt is prover-local material**.
/// The SDK must never persist or transmit it.
#[test]
fn the_journal_does_contain_the_approver_account_id() {
    let (claim, witness, pre_states) = alice_approves();
    let (_cycles, journal) = execute_approval_journal(
        &program_binary(),
        SELF_PROGRAM_ID,
        None,
        &pre_states,
        &claim,
        &witness,
    )
    .expect("honest approval executes");

    let output: lee_core::program::ProgramOutput =
        risc0_zkvm::serde::from_slice(&journal).expect("journal decodes as ProgramOutput");
    let account_id = derive_account_id(&npk_of(&ALICE_NSK), &vpk(), 0);
    assert_eq!(
        output.pre_states[0].account_id, account_id,
        "pre_states carry the approver account id — inherent to LEZ, and why inner receipts are secret"
    );
}

/// How risc0's serde lays a byte slice into the journal: one 32-bit little-endian word per byte.
fn word_encode(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().flat_map(|b| [*b, 0, 0, 0]).collect()
}
