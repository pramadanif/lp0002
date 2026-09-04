//! The four state transitions, as pure functions.
//!
//! Kept separate from the program binary so the lifecycle can be exercised on the host in
//! milliseconds. The SPEL program (`programs/multisig-spel`) is a thin wrapper that supplies
//! accounts and writes results; the rules live here.
//!
//! Every function returns `Result<_, MultisigError>` rather than panicking, because these are
//! *documented* outcomes (**P-R3**) that a caller must be able to distinguish — unlike the
//! membership guest, where a failure means "this proof is invalid" and aborting is the point.

use pmsig_core::{config_hash, Digest32, MemberCount, Threshold, STATE_VERSION};
use pmsig_membership_core::ApprovalClaim;

use crate::{MultisigConfig, MultisigError, ProgramIdWords, Proposal, ProposedAction};

/// Everything `create_multisig` needs.
#[derive(Debug, Clone)]
pub struct CreateMultisig {
    pub member_root: Digest32,
    pub m: Threshold,
    pub n: MemberCount,
    pub multisig_id: Digest32,
    pub membership_program_id: ProgramIdWords,
}

/// Builds the config account for a new multisig, and the `config_hash` that must seed its PDA.
///
/// # Errors
/// [`MultisigError::InvalidThresholdConfig`] if `M == 0`, `N == 0` or `M > N`.
pub fn create_multisig(
    params: &CreateMultisig,
) -> Result<(MultisigConfig, Digest32), MultisigError> {
    if params.m == 0 || params.n == 0 || params.m > params.n {
        return Err(MultisigError::InvalidThresholdConfig);
    }
    let config = MultisigConfig {
        version: STATE_VERSION,
        member_root: params.member_root,
        m: params.m,
        n: params.n,
        multisig_id: params.multisig_id,
        membership_program_id: params.membership_program_id,
        proposal_count: 0,
    };
    let hash = config.recompute_config_hash();
    Ok((config, hash))
}

/// Checks a config account against the PDA seed it was found under.
///
/// **ADR-001 INV-3.** Everything else in this module calls it first; a config account that does not
/// attest to its own address is not a multisig.
///
/// # Errors
/// [`MultisigError::ConfigHashMismatch`] if the stored fields do not rehash to `seed`, or
/// [`MultisigError::InvalidThresholdConfig`] if the stored configuration is malformed.
pub fn validate_config(config: &MultisigConfig, seed: &Digest32) -> Result<(), MultisigError> {
    if !config.is_well_formed() {
        return Err(MultisigError::InvalidThresholdConfig);
    }
    if config.recompute_config_hash() != *seed {
        return Err(MultisigError::ConfigHashMismatch);
    }
    Ok(())
}

/// Builds a new proposal, plus the seed its PDA must use.
///
/// # Errors
/// Propagates [`validate_config`]; [`MultisigError::InvalidProposalAction`] for a zero-value
/// transfer, which is never a meaningful governance action and is almost always a client bug.
pub fn create_proposal(
    config: &MultisigConfig,
    config_seed: &Digest32,
    proposal_id: Digest32,
    action: ProposedAction,
) -> Result<(Proposal, Digest32), MultisigError> {
    validate_config(config, config_seed)?;
    match &action {
        ProposedAction::TreasuryTransfer { amount, .. } if *amount == 0 => {
            return Err(MultisigError::InvalidProposalAction);
        }
        ProposedAction::TreasuryTransfer { .. } => {}
    }
    let proposal = Proposal {
        version: STATE_VERSION,
        config_hash: *config_seed,
        proposal_id,
        action,
        nullifiers: Vec::new(),
        executed: false,
    };
    let seed = pmsig_core::proposal_seed(config_seed, &proposal_id);
    Ok((proposal, seed))
}

/// Records an approval.
///
/// The membership proof itself is **not** checked here: it is proved by the chained call to the
/// membership program, which LEZ's privacy-preserving circuit verifies with `env::verify`. What this
/// function enforces is everything the chain must decide for itself — that the claim belongs to this
/// multisig and proposal, that the verifier is the bound one, and that the nullifier is new.
///
/// `verified_by` is the program id of the chained call that produced the proof. The caller must pass
/// the id it actually invoked, not one taken from user input.
///
/// # Errors
/// - [`MultisigError::ConfigHashMismatch`] / [`MultisigError::InvalidThresholdConfig`] via [`validate_config`]
/// - [`MultisigError::WrongMembershipProgram`] if `verified_by` is not the bound verifier (ADR-002)
/// - [`MultisigError::UnknownProposal`] if the proposal belongs to another multisig or another id
/// - [`MultisigError::ProposalClosed`] if the proposal has already executed
/// - [`MultisigError::MemberRootMismatch`] if the claim names a different member root
/// - [`MultisigError::DuplicateNullifier`] if this member has already approved (**P-F3**, INV-4)
pub fn approve(
    config: &MultisigConfig,
    config_seed: &Digest32,
    proposal: &mut Proposal,
    claim: &ApprovalClaim,
    verified_by: &ProgramIdWords,
) -> Result<(), MultisigError> {
    validate_config(config, config_seed)?;

    // ADR-002: only the verifier this multisig's address attests to may vouch for an approval.
    if *verified_by != config.membership_program_id {
        return Err(MultisigError::WrongMembershipProgram);
    }
    if proposal.config_hash != *config_seed || claim.multisig_id != config.multisig_id {
        return Err(MultisigError::UnknownProposal);
    }
    if claim.proposal_id != proposal.proposal_id {
        return Err(MultisigError::UnknownProposal);
    }
    if proposal.executed {
        return Err(MultisigError::ProposalClosed);
    }
    // Catches an approval proved against a stale member set after a configuration change (INV-5).
    if claim.member_root != config.member_root {
        return Err(MultisigError::MemberRootMismatch);
    }
    if proposal.has_nullifier(&claim.claimed_nullifier) {
        return Err(MultisigError::DuplicateNullifier);
    }

    proposal.nullifiers.push(claim.claimed_nullifier);
    Ok(())
}

/// Executes a proposal that has reached its threshold.
///
/// Returns the action to carry out; the program applies it to the treasury accounts.
///
/// # Errors
/// - [`validate_config`] errors
/// - [`MultisigError::UnknownProposal`] if the proposal is not this multisig's
/// - [`MultisigError::AlreadyExecuted`] if it has already run
/// - [`MultisigError::ThresholdNotMet`] if fewer than `M` approvals are recorded
pub fn execute(
    config: &MultisigConfig,
    config_seed: &Digest32,
    proposal: &mut Proposal,
) -> Result<ProposedAction, MultisigError> {
    validate_config(config, config_seed)?;
    if proposal.config_hash != *config_seed {
        return Err(MultisigError::UnknownProposal);
    }
    if proposal.executed {
        return Err(MultisigError::AlreadyExecuted);
    }
    if !proposal.threshold_met(config.m) {
        return Err(MultisigError::ThresholdNotMet);
    }
    proposal.executed = true;
    Ok(proposal.action.clone())
}

/// The `config_hash` a set of parameters implies. Convenience for host-side callers.
#[must_use]
pub fn derive_config_hash(params: &CreateMultisig) -> Digest32 {
    config_hash(
        &params.member_root,
        params.m,
        params.n,
        &params.multisig_id,
        &params.membership_program_id,
    )
}
