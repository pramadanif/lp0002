#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! Runs the **actual multisig program binary** in the risc0 executor.
//!
//! Until this file existed, every test exercised the *rules* (`pmsig_multisig_core::logic`) and none
//! exercised the *program*. Account ordering, PDA derivation, state encoding and the `ChainedCall`
//! were covered by nothing but a manual run — a bug in the SPEL wrapper would have reached testnet
//! before it reached a test.
//!
//! These drive `artifacts/multisig.bin`, the same binary that gets deployed (plan gate **W3**).

use lee_core::encryption::ViewingPublicKey;
use lee_core::{
    account::{Account, AccountId, AccountWithMetadata},
    program::{InstructionData, ProgramId},
};
use pmsig_core::{approval_nullifier, tree::MemberTree, Digest32};
use pmsig_membership_core::{verify::npk_of, ApprovalWitness};
use pmsig_multisig_core::{Instruction, MultisigConfig, Proposal};
use risc0_zkvm::{ExecutorEnv, Receipt};

const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];
const RECIPIENT: Digest32 = [0xC3; 32];
const ALICE: Digest32 = [0x11; 32];
const BOB: Digest32 = [0x22; 32];
const CAROL: Digest32 = [0x33; 32];

fn program() -> Vec<u8> {
    let path = std::env::var("PMSIG_MULTISIG_BIN")
        .unwrap_or_else(|_| "../../artifacts/multisig.bin".to_string());
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}\nRun ./scripts/build-guests.sh first."))
}

fn program_id() -> ProgramId {
    let doc = std::fs::read_to_string("../../artifacts/IMAGE_IDS.md").expect("IMAGE_IDS.md");
    let section = doc.split("## `multisig`").nth(1).expect("multisig section");
    let line = section
        .lines()
        .find(|l| l.contains("ProgramId"))
        .expect("ProgramId row");
    let inner = line.split('[').nth(1).unwrap().split(']').next().unwrap();
    let w: Vec<u32> = inner
        .split(',')
        .map(|x| x.trim().parse().unwrap())
        .collect();
    <[u32; 8]>::try_from(w.as_slice()).unwrap()
}

/// A real Borsh-encoded witness. The program decodes it before building the chained call, so a
/// dummy byte string is rejected with 1001 — which is correct, and worth keeping a test for.
fn witness_bytes(nsk: &Digest32, index: u64) -> Vec<u8> {
    let tree = member_tree();
    let path = tree.path(index as usize).expect("member has a path");
    borsh::to_vec(&ApprovalWitness {
        nsk: *nsk,
        vpk: ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32]),
        identifier: 0,
        member_index: index,
        siblings: path.siblings,
    })
    .expect("witness encodes")
}

fn member_tree() -> MemberTree {
    let npks: Vec<Digest32> = [ALICE, BOB, CAROL]
        .iter()
        .map(|n| npk_of(n).to_byte_array())
        .collect();
    MemberTree::new(&npks).expect("three members")
}

/// `AccountId::for_public_pda` — the address the program will insist on.
fn public_pda(pid: &ProgramId, seed: &Digest32) -> AccountId {
    use risc0_zkvm::sha::{Impl, Sha256 as _};
    let mut buf = Vec::with_capacity(96);
    buf.extend_from_slice(b"/LEE/v0.2/AccountId/PDA/\0\0\0\0\0\0\0\0");
    for w in pid {
        buf.extend_from_slice(&w.to_le_bytes());
    }
    buf.extend_from_slice(seed);
    let d: [u8; 32] = Impl::hash_bytes(&buf).as_bytes().try_into().unwrap();
    AccountId::new(d)
}

fn account(
    owner: ProgramId,
    data: Vec<u8>,
    id: AccountId,
    authorized: bool,
) -> AccountWithMetadata {
    let a = Account {
        program_owner: owner,
        data: data.try_into().expect("account data fits"),
        ..Account::default()
    };
    AccountWithMetadata::new(a, authorized, id)
}

/// Runs the program and returns its committed `ProgramOutput`, or the guest's failure message.
fn run(
    pre_states: Vec<AccountWithMetadata>,
    ix: &Instruction,
) -> Result<lee_core::program::ProgramOutput, String> {
    let pid = program_id();
    let words: InstructionData = risc0_zkvm::serde::to_vec(ix).map_err(|e| e.to_string())?;
    let env = ExecutorEnv::builder()
        .write(&pid)
        .and_then(|b| b.write(&None::<ProgramId>))
        .and_then(|b| b.write(&pre_states))
        .and_then(|b| b.write(&words))
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let session = risc0_zkvm::default_executor()
        .execute(env, &program())
        .map_err(|e| e.to_string())?;
    risc0_zkvm::serde::from_slice(&session.journal.bytes).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------------------

/// The program creates a multisig and writes state that decodes to what we asked for.
#[test]
fn create_multisig_runs_in_the_program_and_writes_correct_state() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);

    let config_id = public_pda(&pid, &config_hash);
    let creator = AccountId::new([0x77; 32]);

    let out = run(
        vec![
            account(ProgramId::default(), vec![], config_id, false),
            account(ProgramId::default(), vec![], creator, true),
        ],
        &Instruction::CreateMultisig {
            config_hash,
            member_root: tree.root(),
            m: 2,
            n: 3,
            multisig_id: MULTISIG_ID,
            membership_program_id: pid,
        },
    )
    .expect("create_multisig must succeed in the program");

    let data = out.post_states[0].account().data.as_ref().to_vec();
    let cfg: MultisigConfig = borsh::from_slice(&data).expect("config decodes");
    assert_eq!(cfg.m, 2);
    assert_eq!(cfg.n, 3);
    assert_eq!(cfg.member_root, tree.root());
    assert_eq!(cfg.membership_program_id, pid);
    assert_eq!(
        cfg.recompute_config_hash(),
        config_hash,
        "the state the PROGRAM wrote must rehash to the address it was created at (INV-3)"
    );
}

/// **The point of ADR-002, executed rather than argued.** A config whose stored verifier does not
/// match the address is rejected by the program itself.
#[test]
fn the_program_rejects_a_config_that_does_not_match_its_address() {
    let pid = program_id();
    let tree = member_tree();
    // Address derived for a 2-of-3, but the instruction claims 1-of-3.
    let honest_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);
    let config_id = public_pda(&pid, &honest_hash);

    let err = run(
        vec![
            account(ProgramId::default(), vec![], config_id, false),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x77; 32]),
                true,
            ),
        ],
        &Instruction::CreateMultisig {
            config_hash: honest_hash,
            member_root: tree.root(),
            m: 1, // lowered
            n: 3,
            multisig_id: MULTISIG_ID,
            membership_program_id: pid,
        },
    )
    .expect_err("a lowered threshold must not produce a valid output");
    assert!(
        err.contains("1003") || err.to_lowercase().contains("panic"),
        "expected ConfigHashMismatch (1003), got: {err}"
    );
}

/// A nonsensical threshold is refused by the program, not just by the host-side rules.
#[test]
fn the_program_refuses_m_greater_than_n() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 4, 3, &MULTISIG_ID, &pid);
    let err = run(
        vec![
            account(
                ProgramId::default(),
                vec![],
                public_pda(&pid, &config_hash),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x77; 32]),
                true,
            ),
        ],
        &Instruction::CreateMultisig {
            config_hash,
            member_root: tree.root(),
            m: 4,
            n: 3,
            multisig_id: MULTISIG_ID,
            membership_program_id: pid,
        },
    )
    .expect_err("4-of-3 must be refused");
    assert!(
        err.contains("1009") || err.to_lowercase().contains("panic"),
        "got: {err}"
    );
}

/// `approve` must emit a `ChainedCall` to the membership program — that call is what LEZ's
/// privacy-preserving circuit verifies. If the program stopped emitting it, approvals would carry
/// no proof at all, which is the failure prize PR #131 was closed for.
#[test]
fn approve_emits_a_chained_call_to_the_bound_membership_program() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);

    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: pid,
        proposal_count: 0,
    };
    let prop = Proposal {
        version: pmsig_core::STATE_VERSION,
        config_hash,
        proposal_id: PROPOSAL_ID,
        action: pmsig_multisig_core::ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        nullifiers: Vec::new(),
        executed: false,
    };

    let out = run(
        vec![
            account(
                pid,
                borsh::to_vec(&cfg).unwrap(),
                public_pda(&pid, &config_hash),
                false,
            ),
            account(
                pid,
                borsh::to_vec(&prop).unwrap(),
                public_pda(&pid, &proposal_seed),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x55; 32]),
                true,
            ),
        ],
        &Instruction::Approve {
            config_hash,
            proposal_seed,
            member_root: tree.root(),
            claimed_nullifier: approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID),
            witness: witness_bytes(&ALICE, 0),
        },
    )
    .expect("approve must produce an output");

    assert_eq!(
        out.chained_calls.len(),
        1,
        "approve must emit exactly one chained call — without it the approval carries no proof"
    );
    assert_eq!(
        out.chained_calls[0].program_id, pid,
        "the chained call must target the verifier the config is bound to (ADR-002)"
    );

    // And the approval must actually be recorded.
    let data = out.post_states[1].account().data.as_ref().to_vec();
    let after: Proposal = borsh::from_slice(&data).expect("proposal decodes");
    assert_eq!(after.approvals(), 1);
    assert_eq!(
        after.nullifiers[0],
        approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID)
    );
}

/// The program refuses a second approval carrying the same nullifier (**P-F3**, error 1002).
#[test]
fn the_program_refuses_a_duplicate_nullifier() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);
    let nf = approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID);

    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: pid,
        proposal_count: 0,
    };
    let prop = Proposal {
        version: pmsig_core::STATE_VERSION,
        config_hash,
        proposal_id: PROPOSAL_ID,
        action: pmsig_multisig_core::ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        nullifiers: vec![nf], // already recorded
        executed: false,
    };

    let err = run(
        vec![
            account(
                pid,
                borsh::to_vec(&cfg).unwrap(),
                public_pda(&pid, &config_hash),
                false,
            ),
            account(
                pid,
                borsh::to_vec(&prop).unwrap(),
                public_pda(&pid, &proposal_seed),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x55; 32]),
                true,
            ),
        ],
        &Instruction::Approve {
            config_hash,
            proposal_seed,
            member_root: tree.root(),
            claimed_nullifier: nf,
            witness: witness_bytes(&ALICE, 0),
        },
    )
    .expect_err("a duplicate nullifier must be refused by the program");
    assert!(
        err.contains("1002") || err.to_lowercase().contains("panic"),
        "got: {err}"
    );
}

/// A malformed witness is refused before any chained call is built (error 1001). Found by this
/// suite when a placeholder byte string was passed — the program was right and the test was wrong,
/// so the behaviour is now pinned deliberately.
#[test]
fn the_program_refuses_a_malformed_witness() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);

    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: pid,
        proposal_count: 0,
    };
    let prop = Proposal {
        version: pmsig_core::STATE_VERSION,
        config_hash,
        proposal_id: PROPOSAL_ID,
        action: pmsig_multisig_core::ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        nullifiers: Vec::new(),
        executed: false,
    };

    let err = run(
        vec![
            account(
                pid,
                borsh::to_vec(&cfg).unwrap(),
                public_pda(&pid, &config_hash),
                false,
            ),
            account(
                pid,
                borsh::to_vec(&prop).unwrap(),
                public_pda(&pid, &proposal_seed),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x55; 32]),
                true,
            ),
        ],
        &Instruction::Approve {
            config_hash,
            proposal_seed,
            member_root: tree.root(),
            claimed_nullifier: approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID),
            witness: vec![0u8; 8],
        },
    )
    .expect_err("a malformed witness must be refused");
    assert!(
        err.contains("1001"),
        "expected InvalidProof (1001), got: {err}"
    );
}

/// **The program's output must be one LEZ itself will accept.**
///
/// Producing a `ProgramOutput` is not enough — the runtime independently re-checks it with
/// `validate_execution`: unique pre-state account ids, matching pre/post lengths, and the ownership
/// rules for who may mutate what. A program can look correct in its own tests and still emit
/// something the chain rejects.
///
/// So each instruction's real output is run through LEZ's own validator. This is the check that
/// distinguishes "our rules are right" from "our program is right *as LEZ will execute it*".
#[test]
fn program_output_passes_lez_own_execution_validation() {
    use lee_core::program::validate_execution;

    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);

    // --- create_multisig ---
    let out = run(
        vec![
            account(ProgramId::default(), vec![], public_pda(&pid, &config_hash), false),
            account(ProgramId::default(), vec![], AccountId::new([0x77; 32]), true),
        ],
        &Instruction::CreateMultisig {
            config_hash,
            member_root: tree.root(),
            m: 2,
            n: 3,
            multisig_id: MULTISIG_ID,
            membership_program_id: pid,
        },
    )
    .expect("create_multisig runs");
    validate_execution(&out.pre_states, &out.post_states, out.self_program_id)
        .expect("LEZ must accept the output of create_multisig");

    // --- approve, which also emits a chained call ---
    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: pid,
        proposal_count: 0,
    };
    let prop = Proposal {
        version: pmsig_core::STATE_VERSION,
        config_hash,
        proposal_id: PROPOSAL_ID,
        action: pmsig_multisig_core::ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        nullifiers: Vec::new(),
        executed: false,
    };
    let out = run(
        vec![
            account(pid, borsh::to_vec(&cfg).unwrap(), public_pda(&pid, &config_hash), false),
            account(pid, borsh::to_vec(&prop).unwrap(), public_pda(&pid, &proposal_seed), false),
            account(ProgramId::default(), vec![], AccountId::new([0x55; 32]), true),
        ],
        &Instruction::Approve {
            config_hash,
            proposal_seed,
            member_root: tree.root(),
            claimed_nullifier: approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID),
            witness: witness_bytes(&ALICE, 0),
        },
    )
    .expect("approve runs");
    validate_execution(&out.pre_states, &out.post_states, out.self_program_id)
        .expect("LEZ must accept the output of approve");
}

/// Sanity: the receipt type is in scope, so this file compiles against the same risc0 the SDK uses.
#[allow(dead_code)]
fn _type_anchor(_: Receipt) {}
