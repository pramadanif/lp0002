//! Local persistence for partial approval sets.
//!
//! Criterion **P-R2**: "a partial set of approvals (fewer than M) is preserved and resumable across
//! client restarts". A member who has approved should never have to prove again because the client
//! was closed, and a coordinator collecting approvals should not lose the ones already gathered.
//!
//! # Durability
//!
//! Writes go to a temporary file in the same directory and are then `rename`d over the target, which
//! is atomic on POSIX. A process killed mid-write leaves either the previous complete file or an
//! orphaned temp file — never a half-written store. `save_all_unsafe_for_tests` exists solely to
//! *prove* that: it writes in place so a test can produce the corruption that
//! [`StoreError::Corrupt`] is meant to report.
//!
//! # What is deliberately not stored
//!
//! No `nsk`, no viewing key, no Merkle path, no receipt. The store holds nullifiers — which identify
//! nobody — and public ids. If this file leaks, it reveals which *proposals* a client was working on,
//! not who its owner is. Receipts are prover-local secret material (`docs/security.md` §3b) and are
//! never written to disk.

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
};

use pmsig_core::Digest32;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Filename used under the client's data directory.
pub const STORE_FILENAME: &str = "approvals.json";

/// Version of the on-disk format.
pub const STORE_VERSION: u16 = 1;

/// Where an approval has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Proved locally, not yet accepted by the sequencer.
    Pending,
    /// Observed on chain: the nullifier is in the proposal's set.
    Confirmed,
}

/// One approval this client knows about.
///
/// Holds no member-identifying data — see the module docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub multisig_id: Digest32,
    pub proposal_id: Digest32,
    /// The approval nullifier. Preimage-hiding; identifies nobody.
    pub nullifier: Digest32,
    pub status: ApprovalStatus,
}

/// The on-disk document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StoreDocument {
    version: u16,
    approvals: Vec<ApprovalRecord>,
}

/// Failures the store surfaces. Codes match `docs/error-codes.md`.
#[derive(Debug, Error)]
pub enum StoreError {
    /// 2010 — the store exists but could not be parsed. Never silently discarded.
    #[error("2010 StoreCorrupt: {path} could not be read as an approval store: {source}")]
    Corrupt {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    /// 2010 — the store could not be read or written.
    #[error("2010 StoreCorrupt: I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// 2010 — the file was written by an incompatible version.
    #[error("2010 StoreCorrupt: {path} has format version {found}, expected {expected}")]
    UnsupportedVersion {
        path: PathBuf,
        found: u16,
        expected: u16,
    },
}

/// A client's approval store, backed by a single JSON file.
#[derive(Debug, Clone)]
pub struct ApprovalStore {
    path: PathBuf,
}

impl ApprovalStore {
    /// Opens the store at `dir/approvals.json`. The file need not exist yet.
    #[must_use]
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join(STORE_FILENAME),
        }
    }

    /// The file this store reads and writes.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every approval recorded, in insertion order.
    ///
    /// A missing file is an empty store, not an error — a first run is not a failure. A file that
    /// exists but cannot be parsed **is** an error: silently starting over would discard exactly the
    /// work P-R2 requires be preserved.
    ///
    /// # Errors
    /// [`StoreError::Corrupt`], [`StoreError::Io`] or [`StoreError::UnsupportedVersion`].
    pub fn load(&self) -> Result<Vec<ApprovalRecord>, StoreError> {
        let bytes = match fs::read(&self.path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(StoreError::Io {
                    path: self.path.clone(),
                    source,
                })
            }
        };
        let doc: StoreDocument =
            serde_json::from_slice(&bytes).map_err(|source| StoreError::Corrupt {
                path: self.path.clone(),
                source,
            })?;
        if doc.version != STORE_VERSION {
            return Err(StoreError::UnsupportedVersion {
                path: self.path.clone(),
                found: doc.version,
                expected: STORE_VERSION,
            });
        }
        Ok(doc.approvals)
    }

    /// Records an approval, or updates the status of one already present.
    ///
    /// Idempotent on `(proposal_id, nullifier)`: re-recording the same approval does not create a
    /// duplicate, so a retry after an ambiguous submission is safe.
    ///
    /// # Errors
    /// As [`ApprovalStore::load`], plus write failures.
    pub fn record(&self, record: &ApprovalRecord) -> Result<(), StoreError> {
        let mut approvals = self.load()?;
        match approvals
            .iter_mut()
            .find(|a| a.proposal_id == record.proposal_id && a.nullifier == record.nullifier)
        {
            Some(existing) => existing.status = record.status,
            None => approvals.push(record.clone()),
        }
        self.save_all(&approvals)
    }

    /// Approvals recorded for one proposal.
    ///
    /// # Errors
    /// As [`ApprovalStore::load`].
    pub fn for_proposal(&self, proposal_id: &Digest32) -> Result<Vec<ApprovalRecord>, StoreError> {
        Ok(self
            .load()?
            .into_iter()
            .filter(|a| a.proposal_id == *proposal_id)
            .collect())
    }

    /// How many approvals this client knows of for a proposal — the resume count.
    ///
    /// # Errors
    /// As [`ApprovalStore::load`].
    pub fn approval_count(&self, proposal_id: &Digest32) -> Result<usize, StoreError> {
        Ok(self.for_proposal(proposal_id)?.len())
    }

    /// Whether this nullifier is already recorded, so the client can refuse a double-vote locally
    /// (error 2006) rather than spend a minute proving something the chain will reject.
    ///
    /// # Errors
    /// As [`ApprovalStore::load`].
    pub fn contains_nullifier(&self, nullifier: &Digest32) -> Result<bool, StoreError> {
        Ok(self.load()?.iter().any(|a| a.nullifier == *nullifier))
    }

    /// Writes the whole document atomically: temp file in the same directory, then `rename`.
    ///
    /// # Errors
    /// [`StoreError::Io`] if the directory cannot be created or the file cannot be written.
    fn save_all(&self, approvals: &[ApprovalRecord]) -> Result<(), StoreError> {
        let doc = StoreDocument {
            version: STORE_VERSION,
            approvals: approvals.to_vec(),
        };
        let json = serde_json::to_vec_pretty(&doc).map_err(|source| StoreError::Corrupt {
            path: self.path.clone(),
            source,
        })?;

        let io = |source| StoreError::Io {
            path: self.path.clone(),
            source,
        };
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        {
            let mut f = fs::File::create(&tmp).map_err(io)?;
            f.write_all(&json).map_err(io)?;
            // Flush to disk before the rename, so a power loss cannot leave a renamed-but-empty file.
            f.sync_all().map_err(io)?;
        }
        fs::rename(&tmp, &self.path).map_err(io)?;
        Ok(())
    }

    /// Writes raw bytes in place, without the temp-and-rename dance.
    ///
    /// Exists only so tests can produce a corrupt store and check it is *reported* rather than
    /// silently discarded. Never used on any real path.
    ///
    /// # Errors
    /// [`StoreError::Io`].
    #[doc(hidden)]
    pub fn write_raw_for_tests(&self, bytes: &[u8]) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                path: self.path.clone(),
                source,
            })?;
        }
        fs::write(&self.path, bytes).map_err(|source| StoreError::Io {
            path: self.path.clone(),
            source,
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
mod tests {
    use super::*;

    #[test]
    fn store_filename_is_stable() {
        assert_eq!(STORE_FILENAME, "approvals.json");
    }
}
