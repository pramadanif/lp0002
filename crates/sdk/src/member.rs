//! The member-facing API: what a shielded member needs in order to approve.
//!
//! # Peer privacy is enforced by the shape of these types
//!
//! Criterion **P-F1** requires a member to approve "without revealing their identity to on-chain
//! observers **or other members**". The second half is easy to lose accidentally: a coordinator that
//! collects "who has approved so far" would satisfy every on-chain test and still fail the criterion.
//!
//! So the inputs are split, and the split is the guarantee:
//!
//! - [`MultisigView`] — everything a member may know about the multisig. All of it is public: ids,
//!   the member **root**, the threshold, the on-chain approval count. There is **no field** for a
//!   co-member's account id, npk, or approval status, because there is nowhere to put one.
//! - [`MemberSecrets`] — the member's own key material and Merkle path. Never leaves the process
//!   except inside a proof.
//!
//! [`approve`] takes one of each. It is therefore impossible to *call* it with another member's
//! identity: the type system has no slot for it. `SC-D.5` asserts this by having a second member
//! approve knowing only a [`MultisigView`] built from chain data.

use pmsig_core::{approval_nullifier, tree::MemberPath, Digest32, MemberCount, Threshold};
use pmsig_membership_core::{ApprovalClaim, ApprovalWitness};

use crate::SdkError;

/// Everything a member may legitimately know about a multisig. All public.
///
/// Constructible entirely from the config account, the proposal account and the member set the
/// member was given when the multisig was created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultisigView {
    /// Public identifier of the multisig.
    pub multisig_id: Digest32,
    /// Merkle **root** over member npks. The leaves are not here, and are not needed to approve.
    pub member_root: Digest32,
    /// Threshold `M`, from the config account.
    pub m: Threshold,
    /// Member count `N`, from the config account.
    pub n: MemberCount,
    /// The approval count currently on chain. Public — this is what P-F2 requires be visible.
    pub approvals_on_chain: usize,
}

impl MultisigView {
    /// Whether the threshold has already been reached.
    #[must_use]
    pub fn threshold_met(&self) -> bool {
        self.approvals_on_chain >= usize::from(self.m)
    }

    /// How many further approvals are needed.
    #[must_use]
    pub fn remaining(&self) -> usize {
        usize::from(self.m).saturating_sub(self.approvals_on_chain)
    }
}

/// A member's own secrets. Never serialised, never logged, never sent anywhere.
#[derive(Clone)]
pub struct MemberSecrets {
    /// The member's nullifier secret key.
    pub nsk: Digest32,
    /// The member's viewing public key.
    pub vpk: lee_core::encryption::ViewingPublicKey,
    /// Which address of the member's family is being spent.
    pub identifier: u128,
    /// The member's own authentication path to `member_root`.
    pub path: MemberPath,
}

impl core::fmt::Debug for MemberSecrets {
    /// Redacted on purpose: a stray `dbg!` must not put a spending key in a log.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemberSecrets")
            .field("nsk", &"<redacted>")
            .field("vpk", &"<redacted>")
            .field("identifier", &self.identifier)
            .field("path_len", &self.path.siblings.len())
            .finish()
    }
}

/// The pair of values an approval transaction needs.
#[derive(Debug, Clone)]
pub struct PreparedApproval {
    /// The public half, committed by the guest and recorded on chain.
    pub claim: ApprovalClaim,
    /// The secret half, passed to the guest as a private input.
    pub witness: ApprovalWitness,
}

/// Prepares an approval for `proposal_id`.
///
/// Note what is *not* a parameter: any other member. A caller cannot pass a co-member's account id,
/// npk or approval status, because no parameter accepts one (**W8**, **P-F1**, SC-D.5).
///
/// # Errors
/// - [`SdkError::AlreadyApproved`] if `already_recorded` contains this member's nullifier for this
///   proposal — detected locally, before spending ~53 s on a proof (error 2006);
/// - [`SdkError::StaleProposal`] if the threshold has already been reached (error 2007).
pub fn prepare_approval(
    view: &MultisigView,
    secrets: &MemberSecrets,
    proposal_id: Digest32,
    already_recorded: &[Digest32],
) -> Result<PreparedApproval, SdkError> {
    if view.threshold_met() {
        return Err(SdkError::StaleProposal);
    }

    let nullifier = approval_nullifier(&secrets.nsk, &view.multisig_id, &proposal_id);
    if already_recorded.contains(&nullifier) {
        return Err(SdkError::AlreadyApproved);
    }

    Ok(PreparedApproval {
        claim: ApprovalClaim {
            multisig_id: view.multisig_id,
            proposal_id,
            member_root: view.member_root,
            claimed_nullifier: nullifier,
        },
        witness: ApprovalWitness {
            nsk: secrets.nsk,
            vpk: secrets.vpk.clone(),
            identifier: secrets.identifier,
            member_index: secrets.path.index as u64,
            siblings: secrets.path.siblings.clone(),
        },
    })
}

/// The nullifier this member would produce for a proposal.
///
/// Lets a client check "have I already approved?" against the on-chain set without proving.
#[must_use]
pub fn nullifier_for(
    secrets: &MemberSecrets,
    multisig_id: &Digest32,
    proposal_id: &Digest32,
) -> Digest32 {
    approval_nullifier(&secrets.nsk, multisig_id, proposal_id)
}
