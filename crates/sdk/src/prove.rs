//! Generating and verifying membership proofs.
//!
//! Runs entirely client-side on the member's own machine — criterion **P-F5**. The witness
//! (`nsk`, `vpk`, `identifier`, Merkle path) never leaves this process except inside the proof,
//! where it is an input rather than a committed output.
//!
//! # Dev mode is refused, not tolerated
//!
//! With `RISC0_DEV_MODE=1` risc0 emits a *fake* receipt that proves nothing. The tempting failure
//! mode — fall back to dev mode when the prover is unavailable, and appear to succeed — is exactly
//! how a submission ends up claiming proofs it never generated. [`prove_approval`] refuses
//! (`SdkError::DevModeRefused`, error code 2003) unless the caller explicitly opts in for tests.

use std::time::{Duration, Instant};

use lee_core::{
    account::AccountWithMetadata,
    program::{InstructionData, ProgramId},
};
use pmsig_membership_core::{ApprovalClaim, ApprovalWitness, Instruction};
use risc0_zkvm::{compute_image_id, default_prover, ExecutorEnv, ProverOpts, Receipt};

use crate::SdkError;

/// A generated membership proof, with the measurements a benchmark needs.
#[derive(Debug)]
pub struct ApprovalProof {
    /// The receipt. Verifying it establishes that the guest ran and accepted the witness.
    pub receipt: Receipt,
    /// Wall-clock time spent proving. Recorded in `docs/cu-costs.md` for **P-F5**.
    pub prove_time: Duration,
    /// ImageID of the guest that produced it — the LEZ `ProgramId`.
    pub image_id: [u32; 8],
}

/// Whether `RISC0_DEV_MODE` is switched on in this process's environment.
#[must_use]
pub fn dev_mode_enabled() -> bool {
    dev_mode_from(std::env::var("RISC0_DEV_MODE").ok().as_deref())
}

/// The parsing half of [`dev_mode_enabled`], separated so it can be tested.
///
/// Mutating the environment in a test would need `unsafe`, which this workspace forbids — and
/// rightly, since it races every other test in the process. The interesting logic is which values
/// count as "on", and that is pure.
#[must_use]
pub fn dev_mode_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| {
        let v = v.trim().to_ascii_lowercase();
        v == "1" || v == "true" || v == "yes"
    })
}

/// Builds the executor environment for the membership guest.
///
/// The four writes and their order are dictated by LEZ's `read_lee_inputs`
/// (`lee/state_machine/core/src/program/mod.rs:647-662`): `self_program_id`, `caller_program_id`,
/// `pre_states`, then the instruction as risc0 words. Getting the order wrong would surface as an
/// opaque guest panic, so it is kept adjacent to that citation.
fn build_env<'a>(
    self_program_id: ProgramId,
    caller_program_id: Option<ProgramId>,
    pre_states: &'a [AccountWithMetadata],
    claim: &'a ApprovalClaim,
    witness: &'a ApprovalWitness,
) -> Result<ExecutorEnv<'a>, SdkError> {
    let instruction = Instruction::VerifyApproval(claim.clone());
    let instruction_words: InstructionData = risc0_zkvm::serde::to_vec(&instruction)
        .map_err(|e| SdkError::ProofGenerationFailed(format!("encoding instruction: {e}")))?;

    ExecutorEnv::builder()
        .write(&self_program_id)
        .and_then(|b| b.write(&caller_program_id))
        .and_then(|b| b.write(&pre_states.to_vec()))
        .and_then(|b| b.write(&instruction_words))
        // The witness is written AFTER the standard LEZ inputs and is never echoed into
        // ProgramOutput, so it stays out of the journal. See pmsig_membership_core's module docs.
        .and_then(|b| b.write(witness))
        .map_err(|e| SdkError::ProofGenerationFailed(format!("building executor env: {e}")))?
        .build()
        .map_err(|e| SdkError::ProofGenerationFailed(format!("building executor env: {e}")))
}

/// Proves that `witness` is a valid approval for the account in `pre_states[0]`.
///
/// `program_binary` is the guest's risc0 `ProgramBinary` — `artifacts/membership.bin`, produced by
/// `scripts/build-guests.sh`.
///
/// # Errors
///
/// - [`SdkError::DevModeRefused`] if `RISC0_DEV_MODE` is on and `allow_dev_mode` is false;
/// - [`SdkError::ProofGenerationFailed`] if the guest rejects the witness or the prover fails. The
///   guest's panic message is passed through, so an invalid approval says which check failed.
pub fn prove_approval(
    program_binary: &[u8],
    self_program_id: ProgramId,
    caller_program_id: Option<ProgramId>,
    pre_states: &[AccountWithMetadata],
    claim: &ApprovalClaim,
    witness: &ApprovalWitness,
    allow_dev_mode: bool,
) -> Result<ApprovalProof, SdkError> {
    if dev_mode_enabled() && !allow_dev_mode {
        return Err(SdkError::DevModeRefused);
    }

    let image_id = compute_image_id(program_binary)
        .map_err(|e| SdkError::ProofGenerationFailed(format!("computing image id: {e}")))?;
    let image_id_words: [u32; 8] = image_id
        .as_words()
        .try_into()
        .map_err(|_| SdkError::ProofGenerationFailed("image id must be 8 words".into()))?;

    let env = build_env(
        self_program_id,
        caller_program_id,
        pre_states,
        claim,
        witness,
    )?;

    let started = Instant::now();
    let receipt = default_prover()
        .prove_with_opts(env, program_binary, &ProverOpts::default())
        .map_err(|e| SdkError::ProofGenerationFailed(e.to_string()))?
        .receipt;
    let prove_time = started.elapsed();

    receipt
        .verify(image_id)
        .map_err(|e| SdkError::ProofGenerationFailed(format!("receipt failed to verify: {e}")))?;

    Ok(ApprovalProof {
        receipt,
        prove_time,
        image_id: image_id_words,
    })
}

/// Runs the guest without proving.
///
/// Far faster than proving, so negative tests use it to check that an invalid witness is *rejected*
/// — there is no point spending minutes proving something that must fail. It is **not** a substitute
/// for proving on any submission path.
pub fn execute_approval(
    program_binary: &[u8],
    self_program_id: ProgramId,
    caller_program_id: Option<ProgramId>,
    pre_states: &[AccountWithMetadata],
    claim: &ApprovalClaim,
    witness: &ApprovalWitness,
) -> Result<u64, SdkError> {
    let env = build_env(
        self_program_id,
        caller_program_id,
        pre_states,
        claim,
        witness,
    )?;
    let session = risc0_zkvm::default_executor()
        .execute(env, program_binary)
        .map_err(|e| SdkError::ProofGenerationFailed(e.to_string()))?;
    Ok(session.cycles())
}

/// Runs the guest and returns `(cycles, journal_bytes)`.
///
/// Used to inspect what the guest commits without paying for a proof. The journal is identical
/// whether the session is proved or merely executed.
pub fn execute_approval_journal(
    program_binary: &[u8],
    self_program_id: ProgramId,
    caller_program_id: Option<ProgramId>,
    pre_states: &[AccountWithMetadata],
    claim: &ApprovalClaim,
    witness: &ApprovalWitness,
) -> Result<(u64, Vec<u8>), SdkError> {
    let env = build_env(
        self_program_id,
        caller_program_id,
        pre_states,
        claim,
        witness,
    )?;
    let session = risc0_zkvm::default_executor()
        .execute(env, program_binary)
        .map_err(|e| SdkError::ProofGenerationFailed(e.to_string()))?;
    let journal = session.journal.bytes.clone();
    Ok((session.cycles(), journal))
}
