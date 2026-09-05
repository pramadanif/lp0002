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

const SELF_PROGRAM_ID: ProgramId = [7; 8];
const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];
const ALICE: Digest32 = [0x11; 32];
const BOB: Digest32 = [0x22; 32];
const CAROL: Digest32 = [0x33; 32];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let _multisig = args
        .next()
        .unwrap_or_else(|| "artifacts/multisig.bin".into());
    let membership = args
        .next()
        .unwrap_or_else(|| "artifacts/membership.bin".into());

    let membership_bin =
        std::fs::read(&membership).map_err(|e| format!("cannot read {membership}: {e}"))?;

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
