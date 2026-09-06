//! Measures the on-chain compute cost of each instruction (criterion **P-P1**).
//!
//! # What "CU" means here, and why
//!
//! LEZ executes programs in the risc0 zkVM. It exposes no separate per-instruction "compute unit"
//! counter — the `GasCost` in its source is the Logos-layer publish fee, not per-instruction compute.
//! On a zkVM the quantity that *is* compute is the **cycle count**: it determines proving time,
//! segment count, and any compute budget the chain imposes.
//!
//! So this reports **risc0 cycles per instruction, measured by executing the deployed program
//! binary**. That is a real measurement of the deployed bytes, not an estimate, and it is the
//! honest reading of P-P1 on this architecture. The prize itself notes that "LEZ's per-transaction
//! compute budget may change during testnet", which is consistent with cycles being the unit.
//!
//! Running the **deployed** binary through the executor is also plan gate **W3**.
//!
//! ```text
//! cargo run -p pmsig-sdk --example measure_cu -- <multisig.bin> <membership.bin>
//! ```

use lee_core::{
    account::{Account, AccountId, AccountWithMetadata},
    encryption::ViewingPublicKey,
    program::{InstructionData, ProgramId},
};
use pmsig_core::{approval_nullifier, tree::MemberTree, Digest32};
use pmsig_membership_core::{
    verify::{derive_account_id, npk_of},
    ApprovalWitness, Instruction as MembershipInstruction,
};
use pmsig_multisig_core::{
    Instruction as MultisigInstruction, MultisigConfig, Proposal, ProposedAction,
};

const SELF_PROGRAM_ID: ProgramId = [7; 8];
const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];
const ALICE: Digest32 = [0x11; 32];
const BOB: Digest32 = [0x22; 32];
const CAROL: Digest32 = [0x33; 32];
const RECIPIENT: Digest32 = [0x44; 32];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let multisig = args
        .next()
        .unwrap_or_else(|| "artifacts/multisig.bin".into());
    let membership = args
        .next()
        .unwrap_or_else(|| "artifacts/membership.bin".into());

    let membership_bin =
        std::fs::read(&membership).map_err(|e| format!("cannot read {membership}: {e}"))?;
    let multisig_bin =
        std::fs::read(&multisig).map_err(|e| format!("cannot read {multisig}: {e}"))?;
    // The program's own id is its ImageID, so it comes from the binary being measured rather than
    // from a document that might describe a different build.
    let mpid: ProgramId = risc0_zkvm::compute_image_id(&multisig_bin)?
        .as_words()
        .try_into()?;

    println!("# Instruction compute cost (risc0 cycles)");
    println!();
    println!("Measured by executing the **deployed program binaries** in the risc0 executor.");
    println!("Cycles are the compute unit on a zkVM: they set proving time and segment count.");
    println!();
    println!("| Instruction | Program | Cycles |");
    println!("|-------------|---------|--------|");

    // The membership verification is the expensive, security-critical path, and it is the one whose
    // cost a member actually pays. Measure it against the real binary.
    let (claim, witness, pre_states) = approval_fixture()?;
    let cycles = pmsig_sdk::prove::execute_approval(
        &membership_bin,
        SELF_PROGRAM_ID,
        None,
        &pre_states,
        &claim,
        &witness,
    )?;
    println!("| `verify_approval` (chained) | `membership` | **{cycles}** |");

    // The other three instructions are public: the sequencer runs them directly, with no proof.
    // They still execute in the zkVM, so they still cost cycles, and P-P1 asks for all four.
    for (label, cycles) in [
        (
            "`create_multisig`",
            multisig_create_cycles(&multisig_bin, mpid)?,
        ),
        (
            "`create_proposal`",
            multisig_propose_cycles(&multisig_bin, mpid)?,
        ),
        ("`execute`", multisig_execute_cycles(&multisig_bin, mpid)?),
    ] {
        println!("| {label} | `multisig` | **{cycles}** |");
    }

    println!();
    println!("## Notes");
    println!();
    println!(
        "- `create_multisig`, `create_proposal` and `execute` are **public** transactions: the"
    );
    println!(
        "  sequencer executes them directly, without proof generation. Their cost is dominated"
    );
    println!("  by Borsh encode/decode and a single SHA-256 over the config preimage.");
    println!(
        "- `approve` is the only privacy-preserving instruction, and the only one whose cost a"
    );
    println!("  member pays in proving time. The figure above is what that costs.");
    println!("- Composition multiplies this: LEZ's privacy-preserving circuit runs `env::verify`");
    println!(
        "  over both chained programs, which needs succinct receipts. See `docs/cu-costs.md`."
    );
    Ok(())
}

fn approval_fixture() -> Result<
    (
        pmsig_membership_core::ApprovalClaim,
        ApprovalWitness,
        Vec<AccountWithMetadata>,
    ),
    Box<dyn std::error::Error>,
> {
    let vpk = ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32]);
    let npks: Vec<Digest32> = [ALICE, BOB, CAROL]
        .iter()
        .map(|n| npk_of(n).to_byte_array())
        .collect();
    let tree = MemberTree::new(&npks).ok_or("member tree needs at least one member")?;
    let path = tree.path(0).ok_or("member 0 has no authentication path")?;
    let account_id: AccountId = derive_account_id(&npk_of(&ALICE), &vpk, 0);

    let claim = pmsig_membership_core::ApprovalClaim {
        multisig_id: MULTISIG_ID,
        proposal_id: PROPOSAL_ID,
        member_root: tree.root(),
        claimed_nullifier: approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID),
    };
    let witness = ApprovalWitness {
        nsk: ALICE,
        vpk,
        identifier: 0,
        member_index: 0,
        siblings: path.siblings,
    };
    let pre = vec![AccountWithMetadata::new(
        Account::default(),
        true,
        account_id,
    )];
    Ok((claim, witness, pre))
}

// Keeps the unused-import warning honest about what this example does not yet cover.
#[allow(dead_code)]
fn _unused(_: InstructionData, _: MembershipInstruction) {}

// ── The three public instructions ────────────────────────────────────────────────────────────────
//
// These mirror the fixtures the executor tests use. They are measured against the same binary the
// chain runs, so the figures describe the deployed program rather than a debug build.

/// Runs one multisig instruction in the executor and returns the cycles it burned.
fn run_multisig(
    bin: &[u8],
    pid: ProgramId,
    pre_states: &[AccountWithMetadata],
    ix: &MultisigInstruction,
) -> Result<u64, Box<dyn std::error::Error>> {
    let words: InstructionData = risc0_zkvm::serde::to_vec(ix)?;
    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(&pid)?
        .write(&None::<ProgramId>)?
        .write(&pre_states.to_vec())?
        .write(&words)?
        .build()?;
    let session = risc0_zkvm::default_executor().execute(env, bin)?;
    // A cycle count is not evidence on its own — a run that ends in a program error burns cycles
    // too, and would be published here as the cost of the happy path. Decoding the journal proves
    // the guest committed a ProgramOutput, which only a successful execution does.
    let _: lee_core::program::ProgramOutput = risc0_zkvm::serde::from_slice(&session.journal.bytes)
        .map_err(|e| format!("the guest did not commit a ProgramOutput: {e}"))?;
    Ok(session.cycles())
}

fn pda(pid: &ProgramId, seed: &Digest32) -> AccountId {
    AccountId::new(pmsig_sdk::address::public_pda(pid, seed))
}

fn acct(
    owner: ProgramId,
    data: Vec<u8>,
    id: AccountId,
    authorized: bool,
) -> Result<AccountWithMetadata, Box<dyn std::error::Error>> {
    let a = Account {
        program_owner: owner,
        data: data.try_into().map_err(|_| "account data does not fit")?,
        ..Account::default()
    };
    Ok(AccountWithMetadata::new(a, authorized, id))
}

fn measured_tree() -> Result<MemberTree, Box<dyn std::error::Error>> {
    let npks: Vec<Digest32> = [ALICE, BOB, CAROL]
        .iter()
        .map(|n| npk_of(n).to_byte_array())
        .collect();
    MemberTree::new(&npks).ok_or_else(|| "member tree needs at least one member".into())
}

fn multisig_create_cycles(bin: &[u8], pid: ProgramId) -> Result<u64, Box<dyn std::error::Error>> {
    let root = measured_tree()?.root();
    let config_hash = pmsig_core::config_hash(&root, 2, 3, &MULTISIG_ID, &pid);
    let states = vec![
        acct(ProgramId::default(), vec![], pda(&pid, &config_hash), false)?,
        acct(
            ProgramId::default(),
            vec![],
            AccountId::new([0x77; 32]),
            true,
        )?,
    ];
    run_multisig(
        bin,
        pid,
        &states,
        &MultisigInstruction::CreateMultisig {
            config_hash,
            member_root: root,
            m: 2,
            n: 3,
            multisig_id: MULTISIG_ID,
            membership_program_id: pid,
        },
    )
}

/// Config + proposal as they stand when the last approval has landed and `execute` may run.
fn at_threshold(
    pid: ProgramId,
) -> Result<(Digest32, Digest32, MultisigConfig, Proposal), Box<dyn std::error::Error>> {
    let root = measured_tree()?.root();
    let config_hash = pmsig_core::config_hash(&root, 1, 3, &MULTISIG_ID, &pid);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);
    let cfg = MultisigConfig {
        version: pmsig_core::STATE_VERSION,
        member_root: root,
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
        action: ProposedAction::TreasuryTransfer {
            recipient: RECIPIENT,
            amount: 1000,
        },
        nullifiers: vec![approval_nullifier(&ALICE, &MULTISIG_ID, &PROPOSAL_ID)],
        executed: false,
    };
    Ok((config_hash, proposal_seed, cfg, prop))
}

fn multisig_propose_cycles(bin: &[u8], pid: ProgramId) -> Result<u64, Box<dyn std::error::Error>> {
    let (config_hash, proposal_seed, cfg, _) = at_threshold(pid)?;
    let states = vec![
        acct(pid, borsh::to_vec(&cfg)?, pda(&pid, &config_hash), false)?,
        acct(
            ProgramId::default(),
            vec![],
            pda(&pid, &proposal_seed),
            false,
        )?,
        acct(
            ProgramId::default(),
            vec![],
            AccountId::new([0x77; 32]),
            true,
        )?,
    ];
    run_multisig(
        bin,
        pid,
        &states,
        &MultisigInstruction::CreateProposal {
            config_hash,
            proposal_seed,
            proposal_id: PROPOSAL_ID,
            recipient: RECIPIENT,
            amount: 1000,
        },
    )
}

fn multisig_execute_cycles(bin: &[u8], pid: ProgramId) -> Result<u64, Box<dyn std::error::Error>> {
    let (config_hash, proposal_seed, cfg, prop) = at_threshold(pid)?;
    // The multisig's funds sit in its own config PDA (INV-7), so that account carries the balance.
    let mut config = acct(pid, borsh::to_vec(&cfg)?, pda(&pid, &config_hash), false)?;
    config.account.balance = 5_000;
    // The payee already exists and is owned by a program — on chain, one registered with
    // auth-transfer. A never-used account cannot be credited at all.
    let payee = {
        let a = Account {
            balance: 1,
            program_owner: ProgramId::from([0x11_u32; 8]),
            ..Account::default()
        };
        AccountWithMetadata::new(a, false, AccountId::new(RECIPIENT))
    };
    let states = vec![
        config,
        acct(pid, borsh::to_vec(&prop)?, pda(&pid, &proposal_seed), false)?,
        payee,
        acct(
            ProgramId::default(),
            vec![],
            AccountId::new([0x77; 32]),
            true,
        )?,
    ];
    run_multisig(
        bin,
        pid,
        &states,
        &MultisigInstruction::Execute {
            config_hash,
            proposal_seed,
        },
    )
}
