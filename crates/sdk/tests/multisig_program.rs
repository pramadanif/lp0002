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

/// Asserts a program failure carries exactly the documented error, checked on **both** halves of
/// the wire format that SPEL produces for `SpelError::Custom`:
///
/// ```text
/// Program error [7002]: Program error 1002: DuplicateNullifier
///                ^^^^                 ^^^^  ^^^^^^^^^^^^^^^^^^
///                6000+code            code  name
/// ```
///
/// SPEL maps `Custom { code }` to the numeric code `6000 + code`
/// (`spel-framework-core/src/error.rs`), so `6000 + code` — not the bare code — is what a client
/// sees. Both are pinned here so that a change to either the offset or our own numbering is caught
/// by a failing test rather than discovered by a reviewer. See `docs/error-codes.md`.
///
/// Deliberately strict: an earlier version of these assertions allowed
/// `|| err.contains("panic")`, which made them pass for *any* guest panic and so verified nothing.
#[track_caller]
fn assert_program_error(err: &str, code: u32, name: &str) {
    let wire = 6000 + code;
    assert!(
        err.contains(&format!("[{wire}]")),
        "expected on-wire code [{wire}] ({name}), got: {err}"
    );
    assert!(
        err.contains(&format!("{code}: {name}")),
        "expected `{code}: {name}` in the message, got: {err}"
    );
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
    assert_program_error(&err, 1003, "ConfigHashMismatch");
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
    assert_program_error(&err, 1009, "InvalidThresholdConfig");
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
    assert_program_error(&err, 1002, "DuplicateNullifier");
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
    assert_program_error(&err, 1001, "InvalidProof");
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
    .expect("approve runs");
    validate_execution(&out.pre_states, &out.post_states, out.self_program_id)
        .expect("LEZ must accept the output of approve");
}

/// Sanity: the receipt type is in scope, so this file compiles against the same risc0 the SDK uses.
#[allow(dead_code)]
fn _type_anchor(_: Receipt) {}

/// Shared fixture for the `execute` tests: a 1-of-3 whose single proposal has met its threshold and
/// is payable to `RECIPIENT`.
fn executable_proposal() -> (ProgramId, Digest32, Digest32, MultisigConfig, Proposal) {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 1, 3, &MULTISIG_ID, &pid);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);
    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: tree.root(),
        m: 1,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: pid,
        proposal_count: 1,
    };
    let prop = Proposal {
        version: pmsig_core::STATE_VERSION,
        config_hash,
        proposal_id: PROPOSAL_ID,
        action: pmsig_multisig_core::ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        nullifiers: vec![approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID)],
        executed: false,
    };
    (pid, config_hash, proposal_seed, cfg, prop)
}

fn execute_accounts(
    pid: ProgramId,
    config_hash: Digest32,
    proposal_seed: Digest32,
    cfg: &MultisigConfig,
    prop: &Proposal,
    recipient_id: AccountId,
) -> Vec<AccountWithMetadata> {
    // The multisig's funds sit in its own config PDA — there is no caller-supplied treasury slot
    // (INV-7), so the config account is the one carrying a balance.
    let mut config = account(
        pid,
        borsh::to_vec(cfg).unwrap(),
        public_pda(&pid, &config_hash),
        false,
    );
    config.account.balance = 5_000;
    vec![
        config,
        account(
            pid,
            borsh::to_vec(prop).unwrap(),
            public_pda(&pid, &proposal_seed),
            false,
        ),
        // The payee is an account that already exists and is owned by some program — on chain,
        // one registered with auth-transfer. Two LEZ rules force this, and between them they leave
        // no room for paying a fresh address:
        //
        //   * a never-used account cannot be credited at all (`DefaultAccountModifiedWithoutClaim`,
        //     enforced by the sequencer's admission check, **not** by `validate_execution` — which
        //     is why the executor tests passed while the chain rejected the transaction);
        //   * an account with a *default owner* that is no longer in default state is refused by
        //     `validate_execution` rule 7.
        //
        // So the demo pays the creator's account rather than an invented address.
        {
            let mut a = Account {
                balance: 1,
                ..Account::default()
            };
            a.program_owner = ProgramId::from([0x11_u32; 8]);
            AccountWithMetadata::new(a, false, recipient_id)
        },
        // The submitter. Signed, but not an authority: the program never reads it, and every
        // account execute touches is pinned by the config, the seed or the approved action.
        account(
            ProgramId::default(),
            vec![],
            AccountId::new([0x77; 32]),
            true,
        ),
    ]
}

/// The happy path: a proposal at threshold pays the account it named.
#[test]
fn execute_pays_the_proposals_recipient() {
    let (pid, config_hash, proposal_seed, cfg, prop) = executable_proposal();
    let out = run(
        execute_accounts(
            pid,
            config_hash,
            proposal_seed,
            &cfg,
            &prop,
            AccountId::new(RECIPIENT),
        ),
        &Instruction::Execute {
            config_hash,
            proposal_seed,
        },
    )
    .expect("a proposal at threshold must execute");

    // `execute` writes back `[config, proposal, recipient]`.
    assert_eq!(
        out.post_states[2].account().balance,
        1001,
        "the named recipient must receive the approved amount on top of what it held"
    );
    assert_eq!(
        out.post_states[0].account().balance,
        4_000,
        "the multisig's own account must be debited by exactly that amount"
    );
    let after: Proposal =
        borsh::from_slice(out.post_states[1].account().data.as_ref()).expect("proposal decodes");
    assert!(after.executed, "the proposal must be marked executed");
}

/// **Security regression (INV-7).** The members approved paying `RECIPIENT`. Whoever submits the
/// `execute` transaction chooses which account sits in the recipient slot, so the program must
/// check that account against the action the proposal actually carries — otherwise a submitter
/// redirects an approved payment to themselves and the approval covers a transfer that never
/// happened. Nothing else can catch this: the members' signatures are over the proposal, and the
/// proposal is unchanged.
#[test]
fn execute_refuses_a_recipient_the_proposal_did_not_name() {
    let (pid, config_hash, proposal_seed, cfg, prop) = executable_proposal();
    let mallory = AccountId::new([0x66; 32]);
    assert_ne!(mallory, AccountId::new(RECIPIENT));

    let err = run(
        execute_accounts(pid, config_hash, proposal_seed, &cfg, &prop, mallory),
        &Instruction::Execute {
            config_hash,
            proposal_seed,
        },
    )
    .expect_err(
        "PAYMENT REDIRECTION: execute paid an account the proposal never named. The approved \
         action was a transfer to RECIPIENT.",
    );
    assert_program_error(&err, 1012, "InvalidProposalAction");
}

/// `create_proposal` was the last instruction with no coverage through the executor. It writes a
/// proposal that decodes to what was asked for, at the address its seed derives to.
#[test]
fn create_proposal_runs_in_the_program_and_writes_correct_state() {
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

    let out = run(
        vec![
            account(
                pid,
                borsh::to_vec(&cfg).unwrap(),
                public_pda(&pid, &config_hash),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                public_pda(&pid, &proposal_seed),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x77; 32]),
                true,
            ),
        ],
        &Instruction::CreateProposal {
            config_hash,
            proposal_seed,
            proposal_id: PROPOSAL_ID,
            recipient: RECIPIENT,
            amount: 1000,
        },
    )
    .expect("create_proposal must succeed in the program");

    let prop: Proposal =
        borsh::from_slice(out.post_states[1].account().data.as_ref()).expect("proposal decodes");
    assert_eq!(prop.proposal_id, PROPOSAL_ID);
    assert_eq!(prop.config_hash, config_hash);
    assert_eq!(
        prop.action,
        pmsig_multisig_core::ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        "the stored action must be the one proposed — `execute` pays whoever this names (INV-7)"
    );
    assert_eq!(prop.approvals(), 0, "a fresh proposal carries no approvals");
    assert!(!prop.executed);
}

/// A proposal must live at the address its own `(config_hash, proposal_id)` derives to, or a
/// caller could park a proposal at an address of their choosing and `execute` would find it there.
#[test]
fn create_proposal_refuses_a_seed_that_is_not_derived_from_its_contents() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &pid);
    let honest_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);
    // The seed for a *different* proposal id, so it is well-formed but not this proposal's.
    let wrong_seed = pmsig_core::proposal_seed(&config_hash, &[0xEE; 32]);
    assert_ne!(honest_seed, wrong_seed);

    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: MULTISIG_ID,
        membership_program_id: pid,
        proposal_count: 0,
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
                ProgramId::default(),
                vec![],
                public_pda(&pid, &wrong_seed),
                false,
            ),
            account(
                ProgramId::default(),
                vec![],
                AccountId::new([0x77; 32]),
                true,
            ),
        ],
        &Instruction::CreateProposal {
            config_hash,
            proposal_seed: wrong_seed,
            proposal_id: PROPOSAL_ID,
            recipient: RECIPIENT,
            amount: 1000,
        },
    )
    .expect_err("a proposal seed that does not derive from the proposal must be refused");
    assert_program_error(&err, 1006, "UnknownProposal");
}

/// **H9 — the approval markers `execute` counts must be ones only this program could have written.**
///
/// `execute` spends money on the strength of a nullifier set it reads out of the proposal account.
/// If that account could be supplied from anywhere, the threshold would be forgeable and the proofs
/// pointless: an attacker would hand `execute` an account of their own containing M invented
/// nullifiers. Note this is *not* covered by the `config_hash` argument — forging approvals does not
/// change the config account's address.
///
/// Three things stop it, and this test pins the first, which is the one in our reach:
///
/// 1. **SPEL** rejects a proposal account whose id is not `compute_pda(self_program_id,
///    proposal_seed)` — asserted below.
/// 2. **LEZ** lets a program claim ownership only of accounts derived from *its own* id:
///    `Claim::Pda(PdaSeed)` is documented as "the program emits the seed; the `AccountId` is derived
///    from `(program_id, seed)`" (`lee/state_machine/core/src/program/mod.rs`). So no other program
///    can come to own an account at our PDA.
/// 3. **LEZ** allows a data change only when `account_program_owner == executing_program_id`, or the
///    pre-state is default (`validate_execution` rule 6), and forbids a silent owner change
///    (rule 4).
///
/// Together: the account at that address can only ever have been written by this program.
#[test]
fn execute_refuses_a_proposal_account_at_the_wrong_address() {
    let (pid, config_hash, proposal_seed, cfg, prop) = executable_proposal();

    let mut accounts = execute_accounts(
        pid,
        config_hash,
        proposal_seed,
        &cfg,
        &prop,
        AccountId::new(RECIPIENT),
    );
    // Same well-formed, threshold-met proposal — parked at an address it does not derive to, as an
    // attacker supplying their own account would have to do.
    accounts[1] = account(
        pid,
        borsh::to_vec(&prop).unwrap(),
        AccountId::new([0x5A; 32]),
        false,
    );

    let err = run(
        accounts,
        &Instruction::Execute {
            config_hash,
            proposal_seed,
        },
    )
    .expect_err("a proposal account that is not the derived PDA must be refused");
    assert!(
        err.contains("[1009]") || err.contains("PdaMismatch"),
        "expected SPEL's PdaMismatch (1009), got: {err}"
    );
}

/// **`execute` is rejected by LEZ, and no test caught it.**
///
/// `execute_pays_the_proposals_recipient` passes: the program computes the right post-states. But
/// the chain has the final say, and on the public testnet the transaction was submitted and never
/// confirmed — twice, with a funded treasury and a proposal at full threshold. The guest was right
/// and the *output* was inadmissible.
///
/// `program_output_passes_lez_own_execution_validation` covers `create_multisig` and `approve` and
/// stops there, which is why this went unseen. Running the same validator over `execute` reproduces
/// the chain's verdict in milliseconds instead of a twenty-minute submission.
#[test]
fn execute_output_passes_lez_own_execution_validation() {
    use lee_core::program::validate_execution;

    let (pid, config_hash, proposal_seed, cfg, prop) = executable_proposal();
    let out = run(
        execute_accounts(
            pid,
            config_hash,
            proposal_seed,
            &cfg,
            &prop,
            AccountId::new(RECIPIENT),
        ),
        &Instruction::Execute {
            config_hash,
            proposal_seed,
        },
    )
    .expect("execute must produce output");

    validate_execution(&out.pre_states, &out.post_states, out.self_program_id)
        .expect("LEZ must accept the output of execute");
}

/// **The admission rules, checked in milliseconds instead of a fifty-minute demo run.**
///
/// `validate_execution` is not the only thing standing between a program's output and a block. LEZ
/// admits a transaction through `ValidatedStateDiff::from_public_transaction`
/// (`lee/state_machine/src/validated_state_diff/mod.rs`), which enforces a further eighteen rules —
/// and that is the layer that rejected `execute` four times while
/// `execute_output_passes_lez_own_execution_validation` passed throughout. Each rejection cost a
/// full run to discover: two proofs, a sequencer build, roughly fifty minutes, to learn one rule.
///
/// That module cannot be called from here — it lives in the `lee` crate under `.refs/`, a local
/// checkout that is not committed, so depending on it would break CI and any fresh clone. The rules
/// this test can check without it are transcribed instead, from the source, with the reasoning kept
/// next to each one.
#[test]
fn every_instruction_satisfies_lez_admission_rules() {
    let pid = program_id();
    let tree = member_tree();
    let config_hash = pmsig_core::config_hash(&tree.root(), 1, 3, &MULTISIG_ID, &pid);
    let (epid, ech, eps, cfg, prop) = executable_proposal();

    let outputs = [
        (
            "create_multisig",
            run(
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
                    m: 1,
                    n: 3,
                    multisig_id: MULTISIG_ID,
                    membership_program_id: pid,
                },
            ),
        ),
        (
            "execute",
            run(
                execute_accounts(epid, ech, eps, &cfg, &prop, AccountId::new(RECIPIENT)),
                &Instruction::Execute {
                    config_hash: ech,
                    proposal_seed: eps,
                },
            ),
        ),
    ];

    for (name, out) in outputs {
        let out = out.unwrap_or_else(|e| panic!("{name} must produce output: {e}"));

        // "Public transaction must have at least one account."
        assert!(
            !out.pre_states.is_empty(),
            "{name}: a public transaction needs at least one account"
        );

        // "Duplicate account_ids found in message." Reusing one account in two roles is the
        // mistake that rejected `execute` once the payee was set to the submitter.
        let mut seen = std::collections::HashSet::new();
        for a in &out.pre_states {
            assert!(
                seen.insert(a.account_id),
                "{name}: account {} appears twice; LEZ requires unique account ids",
                a.account_id
            );
        }

        // `DefaultAccountModifiedWithoutClaim`: an account whose *pre* state has the default owner
        // may only be modified if the *post* state claims it — i.e. carries a non-default owner.
        // This is what refused a transfer to an address nobody had ever used, and claiming a payee
        // is not an option for a multisig, so the payee must already be owned by some program.
        for (pre, post) in out.pre_states.iter().zip(out.post_states.iter()) {
            if pre.account.program_owner != ProgramId::default() {
                continue;
            }
            if pre.account == *post.account() {
                continue;
            }
            // The rule is checked against the *applied* diff, after claims have been processed —
            // so an output satisfies it either by already carrying a non-default owner or by
            // requesting one. Checking only the owner failed `create_multisig`, which has worked on
            // chain in every run: its config account is `init`, and SPEL expresses that as a claim.
            assert!(
                post.required_claim().is_some()
                    || post.account().program_owner != ProgramId::default(),
                "{name}: account {} starts with the default owner and is modified, so LEZ requires \
                 the output to claim it — see DefaultAccountModifiedWithoutClaim",
                pre.account_id
            );
        }

        // Authorization is checked in *both* directions: an account the program marks authorized
        // must really be authorized (`InvalidAccountAuthorization`), and one that really is
        // authorized must be marked (`AuthorizedAccountMarkedAsNotAuthorized`).
        //
        // For a top-level public call the authorized set is exactly the signers: LEZ's own test
        // `compute_public_authorized_pdas_no_caller_returns_empty` pins that a call with no caller
        // authorizes no PDAs at all. So the config and proposal PDAs must NOT be marked, and the
        // signer must be.
        let marked: Vec<_> = out
            .pre_states
            .iter()
            .filter(|a| a.is_authorized)
            .map(|a| a.account_id)
            .collect();
        assert!(
            marked.len() <= 1,
            "{name}: {} accounts marked authorized; a top-level public call authorizes only its \
             signer — PDAs are never in that set",
            marked.len()
        );

        // "Every account the caller declared must appear in the final diff."
        assert_eq!(
            out.pre_states.len(),
            out.post_states.len(),
            "{name}: every declared account must appear in the output"
        );
    }
}
