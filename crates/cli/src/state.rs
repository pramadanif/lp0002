//! A local, file-backed stand-in for chain state.
//!
//! **This is not a chain.** It exists so the CLI's full lifecycle can be exercised — and reviewed —
//! before the sequencer transport lands in Phase E, and so the demo has something deterministic to
//! run against. It applies exactly the transitions `pmsig_multisig_core::logic` defines, so the rules
//! under test are the real ones; what it does not do is prove anything or reach a network.
//!
//! Every command that uses it prints a `[local]` marker, so no output from this path can be mistaken
//! for testnet evidence.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use pmsig_core::Digest32;
use pmsig_multisig_core::{MultisigConfig, Proposal};
use serde::{Deserialize, Serialize};

/// The whole local world: one multisig and its proposals.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalState {
    pub config: Option<MultisigConfig>,
    pub config_hash: Option<Digest32>,
    pub proposals: Vec<Proposal>,
    /// Balances, so a treasury transfer has something to move.
    pub balances: Vec<(Digest32, u128)>,
    /// Member npks, in the order they were committed to the tree.
    ///
    /// **Local mode only.** A real deployment never publishes this — the chain holds a root, and each
    /// member keeps their own authentication path from the moment the multisig was created. It lives
    /// here so a single-machine demo can act as several members; `docs/integration.md` says so
    /// explicitly, and the on-chain state has no equivalent field.
    pub member_npks: Vec<Digest32>,
}

impl LocalState {
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("{} is not a valid local state file", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let json = serde_json::to_vec_pretty(self)?;
        let tmp: PathBuf = path.with_extension("json.tmp");
        std::fs::write(&tmp, &json).with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("renaming into {}", path.display()))?;
        Ok(())
    }

    pub fn proposal_mut(&mut self, proposal_id: &Digest32) -> Option<&mut Proposal> {
        self.proposals
            .iter_mut()
            .find(|p| p.proposal_id == *proposal_id)
    }

    pub fn balance(&self, account: &Digest32) -> u128 {
        self.balances
            .iter()
            .find(|(a, _)| a == account)
            .map_or(0, |(_, b)| *b)
    }

    pub fn set_balance(&mut self, account: Digest32, amount: u128) {
        match self.balances.iter_mut().find(|(a, _)| *a == account) {
            Some(entry) => entry.1 = amount,
            None => self.balances.push((account, amount)),
        }
    }
}
