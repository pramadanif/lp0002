//! Private M-of-N multisig — the LEZ program.
//!
//! Written with the [SPEL framework](https://github.com/logos-co/spel), which supplies the IDL that
//! criterion **P-U3** asks for. The rules themselves live in `pmsig_multisig_core::logic` so they can
//! be tested on the host in milliseconds; this layer supplies accounts, encodes state and emits the
//! chained call.
//!
//! # The approve path
//!
//! `approve` never checks a membership proof itself. It emits a `ChainedCall` to the membership
//! program, and LEZ's privacy-preserving circuit proves that call ran by verifying its
//! `ProgramOutput` with `env::verify`. If the witness were invalid the membership guest would panic,
//! no output would exist, and the transaction would be rejected before it reached a block.
//!
//! What this program decides for itself is everything the chain must not delegate: that the config
//! account attests to its own address, that the verifier is the one this multisig is bound to
//! (ADR-002), that the claim names this proposal, and that the nullifier is new.
//!
//! **There is no public approve path.** `approve` is only meaningful under the privacy-preserving
//! path, because that is what makes `env::verify` cover the chained call. See `docs/security.md`.

#![allow(clippy::needless_pass_by_value, reason = "SPEL handlers take accounts by value")]

use spel_framework::prelude::*;

/// Encodes a value into a LEZ account's data field.
fn encode<T: borsh::BorshSerialize>(value: &T) -> Vec<u8> {
    borsh::to_vec(value).unwrap_or_default()
}

/// Maps a documented multisig error onto SPEL's error type.
///
/// The message carries the stable name from `docs/error-codes.md`, so a failed transaction can be
/// looked up rather than decoded (**P-R3**).
fn err(e: pmsig_multisig_core::MultisigError) -> SpelError {
    SpelError::custom(e.code(), e.name())
}

/// Writes Borsh-encoded state into an account, returning a documented error if it does not fit.
fn with_data<T: borsh::BorshSerialize>(
    account: &AccountWithMetadata,
    value: &T,
) -> Result<AccountWithMetadata, SpelError> {
    let mut updated = account.clone();
    updated.account.data = encode(value)
        .try_into()
        .map_err(|_| err(pmsig_multisig_core::MultisigError::InvalidProposalAction))?;
    Ok(updated)
}

#[lez_program]
mod private_multisig {
    #[allow(unused_imports)]
    use super::*;

    use pmsig_multisig_core::{
        logic::{self, CreateMultisig},
        MultisigConfig, Proposal, ProposedAction,
    };

    /// Creates a multisig at the PDA its own configuration hashes to.
    ///
    /// `config_hash` is supplied by the caller and used as the PDA seed, then **recomputed** from
    /// the parameters and compared: a caller cannot point a well-formed configuration at someone
    /// else's address, nor store parameters that disagree with the address (ADR-001 INV-1..3).
    #[instruction]
    pub fn create_multisig(
        #[account(init, pda = arg("config_hash"))] config: AccountWithMetadata,
        #[account(signer)] creator: AccountWithMetadata,
        config_hash: [u8; 32],
        member_root: [u8; 32],
        m: u8,
        n: u8,
        multisig_id: [u8; 32],
        membership_program_id: [u32; 8],
    ) -> SpelResult {
        let params = CreateMultisig {
            member_root,
            m,
            n,
            multisig_id,
            membership_program_id,
        };
        let (state, derived) =
            logic::create_multisig(&params).map_err(err)?;
        if derived != config_hash {
            return Err(err(pmsig_multisig_core::MultisigError::ConfigHashMismatch));
        }

        let config = with_data(&config, &state)?;
        Ok(SpelOutput::execute(vec![config, creator], vec![]))
    }

    /// Submits a proposal. Its content is public — the prize hides who approved, not what.
    #[instruction]
    pub fn create_proposal(
        #[account(pda = arg("config_hash"))] config: AccountWithMetadata,
        #[account(init, pda = arg("proposal_seed"))] proposal: AccountWithMetadata,
        #[account(signer)] proposer: AccountWithMetadata,
        config_hash: [u8; 32],
        proposal_seed: [u8; 32],
        proposal_id: [u8; 32],
        recipient: [u8; 32],
        amount: u128,
    ) -> SpelResult {
        let state = decode_config(&config)?;
        let (new_proposal, derived_seed) = logic::create_proposal(
            &state,
            &config_hash,
            proposal_id,
            ProposedAction::TreasuryTransfer { recipient, amount },
        )
        .map_err(err)?;
        if derived_seed != proposal_seed {
            return Err(err(pmsig_multisig_core::MultisigError::UnknownProposal));
        }

        let proposal = with_data(&proposal, &new_proposal)?;
        Ok(SpelOutput::execute(vec![config, proposal, proposer], vec![]))
    }

    /// Records one anonymous approval, gated on a chained membership proof.
    ///
    /// `approver` is the member's **shielded** account. It is passed to the membership program as
    /// its `pre_states[0]`, which is what binds the proof to a live account: LEZ's
    /// privacy-preserving circuit independently proves the prover controls it and that its
    /// commitment is in the live commitment set (ADR-001 D4).
    #[instruction]
    pub fn approve(
        #[account(pda = arg("config_hash"))] config: AccountWithMetadata,
        #[account(mut, pda = arg("proposal_seed"))] proposal: AccountWithMetadata,
        approver: AccountWithMetadata,
        config_hash: [u8; 32],
        proposal_seed: [u8; 32],
        member_root: [u8; 32],
        claimed_nullifier: [u8; 32],
    ) -> SpelResult {
        let state = decode_config(&config)?;
        let mut proposal_state = decode_proposal(&proposal)?;

        let claim = pmsig_membership_core::ApprovalClaim {
            multisig_id: state.multisig_id,
            proposal_id: proposal_state.proposal_id,
            member_root,
            claimed_nullifier,
        };

        // The verifier is taken from the *validated config*, never from instruction data, so a
        // caller cannot nominate the program that vouches for them (ADR-002, error 1013).
        let verifier = state.membership_program_id;
        logic::approve(&state, &config_hash, &mut proposal_state, &claim, &verifier)
            .map_err(err)?;

        // The chained call whose ProgramOutput `env::verify` must cover. The membership guest reads
        // its private witness separately; only the public claim travels in instruction data.
        let call = ChainedCall::new(
            verifier,
            vec![approver.clone()],
            &pmsig_membership_core::Instruction::VerifyApproval(claim),
        );

        let proposal = with_data(&proposal, &proposal_state)?;
        Ok(SpelOutput::execute(vec![config, proposal, approver], vec![call]))
    }

    /// Executes a proposal that has reached its threshold, moving treasury funds.
    #[instruction]
    pub fn execute(
        #[account(pda = arg("config_hash"))] config: AccountWithMetadata,
        #[account(mut, pda = arg("proposal_seed"))] proposal: AccountWithMetadata,
        #[account(mut)] treasury: AccountWithMetadata,
        #[account(mut)] recipient: AccountWithMetadata,
        config_hash: [u8; 32],
        proposal_seed: [u8; 32],
    ) -> SpelResult {
        let state = decode_config(&config)?;
        let mut proposal_state = decode_proposal(&proposal)?;

        let action = logic::execute(&state, &config_hash, &mut proposal_state)
            .map_err(err)?;

        let ProposedAction::TreasuryTransfer { amount, .. } = action;

        let mut treasury = treasury.clone();
        let mut recipient = recipient.clone();
        treasury.account.balance = treasury
            .account
            .balance
            .checked_sub(amount)
            .ok_or_else(|| err(pmsig_multisig_core::MultisigError::InvalidProposalAction))?;
        recipient.account.balance = recipient
            .account
            .balance
            .checked_add(amount)
            .ok_or_else(|| err(pmsig_multisig_core::MultisigError::InvalidProposalAction))?;

        let proposal = with_data(&proposal, &proposal_state)?;
        Ok(SpelOutput::execute(
            vec![config, proposal, treasury, recipient],
            vec![],
        ))
    }

    fn decode_config(account: &AccountWithMetadata) -> Result<MultisigConfig, SpelError> {
        borsh::from_slice(account.account.data.as_ref()).map_err(|_| {
            err(pmsig_multisig_core::MultisigError::ConfigHashMismatch)
        })
    }

    fn decode_proposal(account: &AccountWithMetadata) -> Result<Proposal, SpelError> {
        borsh::from_slice(account.account.data.as_ref()).map_err(|_| {
            err(pmsig_multisig_core::MultisigError::UnknownProposal)
        })
    }
}
