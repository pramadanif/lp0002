#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! **SC-D.2 / P-R2** — a partial approval set survives a client restart.
//!
//! "Restart" is modelled the way it actually happens: the process goes away and a *new*
//! `ApprovalStore` is opened over the same directory. Nothing is carried across in memory — if the
//! state did not reach disk, these tests fail.

use pmsig_core::{approval_nullifier, Digest32};
use pmsig_store::{ApprovalRecord, ApprovalStatus, ApprovalStore, StoreError};

const MULTISIG: Digest32 = [0xA1; 32];
const PROPOSAL: Digest32 = [0xB2; 32];
const ALICE: Digest32 = [0x11; 32];
const BOB: Digest32 = [0x22; 32];

/// A scratch directory that cleans itself up.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "pmsig-store-{tag}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn approval(nsk: &Digest32, status: ApprovalStatus) -> ApprovalRecord {
    ApprovalRecord {
        multisig_id: MULTISIG,
        proposal_id: PROPOSAL,
        nullifier: approval_nullifier(nsk, &MULTISIG, &PROPOSAL),
        status,
    }
}

#[test]
fn a_missing_store_is_an_empty_store_not_an_error() {
    let dir = TempDir::new("missing");
    let store = ApprovalStore::new(&dir.0);
    assert_eq!(store.load().unwrap(), Vec::new());
    assert_eq!(store.approval_count(&PROPOSAL).unwrap(), 0);
}

/// The headline case: one of two approvals is collected, the client dies, a fresh client resumes and
/// reaches the threshold.
#[test]
fn a_partial_approval_set_survives_a_restart_and_reaches_the_threshold() {
    let dir = TempDir::new("resume");
    let m = 2_usize;

    // --- client instance #1: collects one approval, then "crashes" ---
    {
        let store = ApprovalStore::new(&dir.0);
        store
            .record(&approval(&ALICE, ApprovalStatus::Confirmed))
            .unwrap();
        assert_eq!(store.approval_count(&PROPOSAL).unwrap(), 1);
        assert!(
            store.approval_count(&PROPOSAL).unwrap() < m,
            "below threshold"
        );
    } // the store value is dropped: nothing survives in memory

    // --- client instance #2: a brand-new store over the same directory ---
    let store = ApprovalStore::new(&dir.0);
    let resumed = store.approval_count(&PROPOSAL).unwrap();
    assert_eq!(resumed, 1, "the first approval must survive the restart");

    store
        .record(&approval(&BOB, ApprovalStatus::Confirmed))
        .unwrap();
    assert_eq!(
        store.approval_count(&PROPOSAL).unwrap(),
        m,
        "the resumed client reaches M without redoing the first approval"
    );
}

/// Re-recording the same approval must not inflate the count — a retry after an ambiguous
/// submission is safe.
#[test]
fn recording_the_same_approval_twice_is_idempotent() {
    let dir = TempDir::new("idempotent");
    let store = ApprovalStore::new(&dir.0);
    store
        .record(&approval(&ALICE, ApprovalStatus::Pending))
        .unwrap();
    store
        .record(&approval(&ALICE, ApprovalStatus::Pending))
        .unwrap();
    assert_eq!(store.approval_count(&PROPOSAL).unwrap(), 1);
}

/// A pending approval that later confirms updates in place rather than duplicating.
#[test]
fn status_transitions_update_in_place() {
    let dir = TempDir::new("status");
    let store = ApprovalStore::new(&dir.0);
    store
        .record(&approval(&ALICE, ApprovalStatus::Pending))
        .unwrap();
    store
        .record(&approval(&ALICE, ApprovalStatus::Confirmed))
        .unwrap();

    let records = ApprovalStore::new(&dir.0).for_proposal(&PROPOSAL).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, ApprovalStatus::Confirmed);
}

/// The client can refuse a double-vote locally instead of spending ~53 s proving something the chain
/// will reject (error 2006).
#[test]
fn a_known_nullifier_is_detected_before_proving() {
    let dir = TempDir::new("dupe");
    let store = ApprovalStore::new(&dir.0);
    let alice_nf = approval_nullifier(&ALICE, &MULTISIG, &PROPOSAL);
    assert!(!store.contains_nullifier(&alice_nf).unwrap());
    store
        .record(&approval(&ALICE, ApprovalStatus::Confirmed))
        .unwrap();
    assert!(ApprovalStore::new(&dir.0)
        .contains_nullifier(&alice_nf)
        .unwrap());
}

/// Approvals for other proposals must not be counted toward this one's threshold.
#[test]
fn approvals_are_scoped_per_proposal() {
    let dir = TempDir::new("scope");
    let store = ApprovalStore::new(&dir.0);
    store
        .record(&approval(&ALICE, ApprovalStatus::Confirmed))
        .unwrap();
    let other = ApprovalRecord {
        multisig_id: MULTISIG,
        proposal_id: [0xCC; 32],
        nullifier: approval_nullifier(&ALICE, &MULTISIG, &[0xCC; 32]),
        status: ApprovalStatus::Confirmed,
    };
    store.record(&other).unwrap();
    assert_eq!(store.approval_count(&PROPOSAL).unwrap(), 1);
    assert_eq!(store.approval_count(&[0xCC; 32]).unwrap(), 1);
}

/// A corrupt store is **reported**, never silently discarded — discarding it would throw away
/// exactly the partial set P-R2 requires be preserved.
#[test]
fn a_corrupt_store_is_reported_rather_than_discarded() {
    let dir = TempDir::new("corrupt");
    let store = ApprovalStore::new(&dir.0);
    store
        .record(&approval(&ALICE, ApprovalStatus::Confirmed))
        .unwrap();
    store.write_raw_for_tests(b"{ this is not json").unwrap();

    match ApprovalStore::new(&dir.0).load() {
        Err(StoreError::Corrupt { path, .. }) => {
            assert_eq!(path, *store.path());
            let msg = StoreError::Corrupt {
                path,
                source: serde_json::from_str::<u8>("x").unwrap_err(),
            }
            .to_string();
            assert!(
                msg.starts_with("2010 StoreCorrupt"),
                "code must be greppable"
            );
        }
        other => panic!("expected a corruption error, got {other:?}"),
    }
}

/// A store written by a future version is refused, not misread.
#[test]
fn an_unsupported_version_is_refused() {
    let dir = TempDir::new("version");
    let store = ApprovalStore::new(&dir.0);
    store
        .write_raw_for_tests(br#"{"version":999,"approvals":[]}"#)
        .unwrap();
    assert!(matches!(
        store.load(),
        Err(StoreError::UnsupportedVersion { found: 999, .. })
    ));
}

/// Writes are atomic: a temp file is renamed over the target, so an interrupted write cannot leave a
/// half-written store. Asserted by checking no temp file survives a successful write.
#[test]
fn writes_leave_no_partial_file_behind() {
    let dir = TempDir::new("atomic");
    let store = ApprovalStore::new(&dir.0);
    store
        .record(&approval(&ALICE, ApprovalStatus::Confirmed))
        .unwrap();

    let leftovers: Vec<_> = std::fs::read_dir(&dir.0)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temp files left behind: {leftovers:?}"
    );
    // And the store is readable by a fresh instance.
    assert_eq!(
        ApprovalStore::new(&dir.0)
            .approval_count(&PROPOSAL)
            .unwrap(),
        1
    );
}
