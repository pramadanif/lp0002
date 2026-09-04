//! The integration example from `docs/integration.md`, kept honest by being compiled.
//!
//! **SC-D.4.** A guide whose code does not compile is worse than no guide, so this file *is* the
//! guide's code: `cargo test --workspace --all-targets` builds it, and `docs/integration.md` points
//! here rather than repeating a snippet that could drift.
//!
//! Run it with:
//! ```text
//! cargo run -p pmsig-sdk --example integrate
//! ```

use lee_core::encryption::ViewingPublicKey;
use pmsig_core::{tree::MemberTree, Digest32};
use pmsig_membership_core::verify::npk_of;
use pmsig_multisig_core::{
    logic::{self, CreateMultisig},
    ProgramIdWords, ProposedAction,
};
use pmsig_sdk::member::{prepare_approval, MemberSecrets, MultisigView};
use pmsig_store::{ApprovalRecord, ApprovalStatus, ApprovalStore};

/// The membership verifier this multisig is bound to. In a real integration, read the deployed
/// program id from `artifacts/IMAGE_IDS.md`.
const VERIFIER: ProgramIdWords = [7; 8];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---------------------------------------------------------------------------------------
    // 1. Build the member set.
    //
    // Members are identified by their nullifier PUBLIC keys. Each member derives their own npk
    // from their nsk and shares only the npk. The set is committed as a Merkle root; the leaves
    // are never published on chain.
    // ---------------------------------------------------------------------------------------
    let alice_nsk: Digest32 = [0x11; 32];
    let bob_nsk: Digest32 = [0x22; 32];
    let carol_nsk: Digest32 = [0x33; 32];

    let npks: Vec<Digest32> = [alice_nsk, bob_nsk, carol_nsk]
        .iter()
        .map(|nsk| npk_of(nsk).to_byte_array())
        .collect();
    let tree = MemberTree::new(&npks).ok_or("a multisig needs at least one member")?;

    // ---------------------------------------------------------------------------------------
    // 2. Create the multisig.
    //
    // `config_hash` seeds the config account's PDA. Because the member root, the threshold and the
    // verifier are all inside it, an attacker who changes any of them derives a different address
    // rather than a weaker multisig.
    // ---------------------------------------------------------------------------------------
    let params = CreateMultisig {
        member_root: tree.root(),
        m: 2,
        n: 3,
        multisig_id: [0xA1; 32],
        membership_program_id: VERIFIER,
    };
    let (config, config_hash) = logic::create_multisig(&params)?;
    println!("config_hash = {}", hex::encode(config_hash));

    // ---------------------------------------------------------------------------------------
    // 3. Propose an action. Proposal content is public; only the identity of approvers is hidden.
    // ---------------------------------------------------------------------------------------
    let proposal_id: Digest32 = [0xB2; 32];
    let (mut proposal, proposal_seed) = logic::create_proposal(
        &config,
        &config_hash,
        proposal_id,
        ProposedAction::TreasuryTransfer {
            recipient: [0xC3; 32],
            amount: 1_000,
        },
    )?;
    println!("proposal_seed = {}", hex::encode(proposal_seed));

    // ---------------------------------------------------------------------------------------
    // 4. A member approves.
    //
    // Note what the approving member needs: the public view, and their OWN secrets. Nothing about
    // any other member. That is what keeps approvals private from co-members, not only from
    // on-chain observers.
    // ---------------------------------------------------------------------------------------
    let store = ApprovalStore::new(std::env::temp_dir().join("pmsig-integration-example"));

    for (index, nsk) in [(0_usize, alice_nsk), (1, bob_nsk)] {
        let view = MultisigView {
            multisig_id: config.multisig_id,
            member_root: config.member_root,
            m: config.m,
            n: config.n,
            approvals_on_chain: proposal.approvals(),
        };

        let secrets = MemberSecrets {
            nsk,
            vpk: ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32]),
            identifier: 0,
            path: tree.path(index).ok_or("member has no path")?,
        };

        let prepared = prepare_approval(&view, &secrets, proposal_id, &proposal.nullifiers)?;

        // In production this is where the approval is proved and submitted:
        //
        //     let proof = pmsig_sdk::prove::prove_approval(
        //         &program_binary, self_program_id, caller, &pre_states,
        //         &prepared.claim, &prepared.witness, false)?;
        //
        // `prove_approval` refuses to run under RISC0_DEV_MODE=1, because a dev-mode receipt
        // proves nothing.

        logic::approve(
            &config,
            &config_hash,
            &mut proposal,
            &prepared.claim,
            &VERIFIER,
        )?;

        // Persist locally so a crash cannot lose the approval (P-R2).
        store.record(&ApprovalRecord {
            multisig_id: config.multisig_id,
            proposal_id,
            nullifier: prepared.claim.claimed_nullifier,
            status: ApprovalStatus::Confirmed,
        })?;

        println!("approvals = {} of {}", proposal.approvals(), config.m);
    }

    // ---------------------------------------------------------------------------------------
    // 5. Execute, once the threshold is met. Anyone may submit this — including a non-member.
    // ---------------------------------------------------------------------------------------
    let action = logic::execute(&config, &config_hash, &mut proposal)?;
    println!("executed: {action:?}");

    // The on-chain record now shows that two of three approved, and nothing about which two.
    assert_eq!(proposal.approvals(), 2);
    assert!(proposal.executed);

    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("pmsig-integration-example"));
    Ok(())
}
