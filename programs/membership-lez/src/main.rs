//! Membership program — the LEZ-native callee that makes an approval anonymous but accountable.
//!
//! Invoked as a `ChainedCall` from the multisig program. LEZ's privacy-preserving circuit proves it
//! ran by calling `env::verify` on this program's `ProgramOutput`
//! (`lee/privacy_preserving_circuit/src/execution_state.rs:151-155`).
//!
//! It answers one question — *may this approval nullifier be recorded against this member root?* —
//! and answers by panicking if not. A panic means no valid `ProgramOutput`, so `env::verify` fails
//! and the transaction is invalid.
//!
//! # What ends up in the journal
//!
//! `ProgramOutput` is written to the journal and echoes `instruction_data`, so the witness — the
//! member's `nsk` included — is in this guest's journal. That is unavoidable: LEZ writes exactly
//! four inputs to a program and offers no private channel
//! (`lee/state_machine/src/program/mod.rs::write_inputs`).
//!
//! It is safe only because this journal never reaches the chain. LEZ's privacy-preserving circuit
//! consumes it via `env::verify` and commits `PrivacyPreservingCircuitOutput`, which carries just
//! nullifiers, commitments and ciphertext. **An inner receipt is prover-local secret material** —
//! see `docs/security.md` §3b.

use lee_core::program::{read_lee_inputs, AccountPostState, ProgramInput, ProgramOutput};
use pmsig_membership_core::{verify_approval, Instruction};

fn main() {
    let (
        ProgramInput {
            self_program_id,
            caller_program_id,
            pre_states,
            instruction,
        },
        instruction_words,
    ) = read_lee_inputs::<Instruction>();

    let Instruction::VerifyApproval(args) = instruction;
    let (claim, witness) = (&args.claim, &args.witness);

    // The approver's shielded account. LEZ's PPE circuit has already bound this account id to a
    // live, unspent commitment; `verify_approval` ties the witness to it.
    let Some(approver) = pre_states.first() else {
        panic!("membership: expected the approver account as pre_states[0]");
    };

    verify_approval(claim, witness, &approver.account_id);

    // This program owns no state and mutates nothing: every pre-state is echoed unchanged.
    let post_states = pre_states
        .iter()
        .map(|pre| AccountPostState::new(pre.account.clone()))
        .collect();

    ProgramOutput::new(
        self_program_id,
        caller_program_id,
        instruction_words,
        pre_states,
        post_states,
    )
    .write();
}
