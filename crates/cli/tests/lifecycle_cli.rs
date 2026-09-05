#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! **SC-D.1 / P-U1** — the CLI drives the whole lifecycle, and refuses what it must.
//!
//! Runs the real binary as a subprocess, so what is tested is what a member types. Each invocation
//! is a fresh process, which also makes the restart-resume case (**SC-D.2 / P-R2**) genuine rather
//! than simulated.

use std::{path::Path, process::Command};

const A: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const C: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const P1: &str = "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2";
const RECIPIENT: &str = "c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3";

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "pmsig-cli-{tag}-{}-{:?}",
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

struct Output {
    ok: bool,
    text: String,
}

fn pmsig(dir: &Path, args: &[&str]) -> Output {
    let out = Command::new(env!("CARGO_BIN_EXE_pmsig"))
        .arg("--state")
        .arg(dir.join("state.json"))
        .arg("--store-dir")
        .arg(dir)
        .args(args)
        .output()
        .expect("pmsig runs");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Output {
        ok: out.status.success(),
        text,
    }
}

fn create_2_of_3(dir: &Path) {
    let out = pmsig(
        dir,
        &["create", "--members", &format!("{A},{B},{C}"), "--m", "2"],
    );
    assert!(out.ok, "create failed: {}", out.text);
}

fn propose(dir: &Path, id: &str) {
    let out = pmsig(
        dir,
        &[
            "propose",
            "--proposal-id",
            id,
            "--recipient",
            RECIPIENT,
            "--amount",
            "1000",
        ],
    );
    assert!(out.ok, "propose failed: {}", out.text);
}

#[test]
fn the_cli_drives_the_whole_lifecycle() {
    let dir = TempDir::new("lifecycle");
    create_2_of_3(&dir.0);
    propose(&dir.0, P1);

    let a1 = pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", A]);
    assert!(a1.ok, "{}", a1.text);
    assert!(a1.text.contains("approvals : 1 of 2"), "{}", a1.text);

    // Below the threshold, execution must refuse — with the documented code.
    let early = pmsig(&dir.0, &["execute", "--proposal-id", P1]);
    assert!(!early.ok);
    assert!(
        early.text.contains("1004"),
        "expected 1004, got: {}",
        early.text
    );

    let a2 = pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", B]);
    assert!(a2.ok, "{}", a2.text);
    assert!(a2.text.contains("approvals : 2 of 2"), "{}", a2.text);

    let exec = pmsig(&dir.0, &["execute", "--proposal-id", P1]);
    assert!(exec.ok, "{}", exec.text);
    assert!(exec.text.contains("transferred : 1000"), "{}", exec.text);

    // Executing twice is refused.
    let again = pmsig(&dir.0, &["execute", "--proposal-id", P1]);
    assert!(!again.ok);
    assert!(
        again.text.contains("1005"),
        "expected 1005, got: {}",
        again.text
    );
}

/// **SC-C.3 through the CLI**: the same member approving twice, while still below the threshold, is
/// refused as a double vote (2006) rather than as a stale proposal.
#[test]
fn a_double_approval_below_the_threshold_is_refused_as_a_double_vote() {
    let dir = TempDir::new("double");
    // 3-of-3, so one member approving twice cannot reach the threshold first.
    let out = pmsig(
        &dir.0,
        &["create", "--members", &format!("{A},{B},{C}"), "--m", "3"],
    );
    assert!(out.ok, "{}", out.text);
    propose(&dir.0, P1);

    assert!(pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", A]).ok);
    let dup = pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", A]);
    assert!(!dup.ok);
    assert!(
        dup.text.contains("2006"),
        "expected 2006 AlreadyApproved, got: {}",
        dup.text
    );
}

/// **SC-D.2 / P-R2** — the client is "killed" between approvals (each command is its own process),
/// and the partial set still reaches the threshold.
#[test]
fn a_partial_approval_set_survives_between_processes() {
    let dir = TempDir::new("resume");
    create_2_of_3(&dir.0);
    propose(&dir.0, P1);

    assert!(pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", A]).ok);

    // A completely fresh process reads the partial set back.
    let status = pmsig(&dir.0, &["status", "--proposal-id", P1]);
    assert!(status.ok, "{}", status.text);
    assert!(
        status.text.contains("approvals : 1 of 2"),
        "{}",
        status.text
    );

    // And the client's own approval store persisted it.
    let store = std::fs::read_to_string(dir.0.join("approvals.json")).expect("store written");
    assert!(store.contains("\"version\": 1"), "{store}");

    assert!(pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", B]).ok);
    let exec = pmsig(&dir.0, &["execute", "--proposal-id", P1]);
    assert!(
        exec.ok,
        "resumed client reaches the threshold: {}",
        exec.text
    );
}

/// **SC-C.6 / P-F2 at the UI layer** — `status` prints a count and nullifiers, never who approved.
#[test]
fn status_shows_a_count_and_no_identities() {
    let dir = TempDir::new("status");
    create_2_of_3(&dir.0);
    propose(&dir.0, P1);
    assert!(pmsig(&dir.0, &["approve", "--proposal-id", P1, "--member", A]).ok);

    let out = pmsig(&dir.0, &["status", "--proposal-id", P1]);
    assert!(out.ok, "{}", out.text);
    assert!(out.text.contains("approvals : 1 of 2"));
    // Neither member key may appear in what a co-member sees.
    for key in [A, B, C] {
        assert!(
            !out.text.contains(key),
            "a member key leaked into status output"
        );
    }
    // Nor the npk of any member.
    for nsk_hex in [A, B, C] {
        let nsk: [u8; 32] = hex::decode(nsk_hex).unwrap().try_into().unwrap();
        let npk = hex::encode(pmsig_membership_core::verify::npk_of(&nsk).to_byte_array());
        assert!(
            !out.text.contains(&npk),
            "a member npk leaked into status output"
        );
    }
}

/// A non-member is told so, with the documented code, rather than being allowed to try.
#[test]
fn a_non_member_is_refused_with_a_documented_code() {
    let dir = TempDir::new("nonmember");
    create_2_of_3(&dir.0);
    propose(&dir.0, P1);
    let outsider = "9999999999999999999999999999999999999999999999999999999999999999";
    let out = pmsig(
        &dir.0,
        &["approve", "--proposal-id", P1, "--member", outsider],
    );
    assert!(!out.ok);
    assert!(
        out.text.contains("2004"),
        "expected 2004, got: {}",
        out.text
    );
}

/// Bad input is rejected with a message that says what was wrong, not a panic (P-R1).
#[test]
fn malformed_input_produces_a_clear_message_not_a_panic() {
    let dir = TempDir::new("badinput");
    create_2_of_3(&dir.0);
    let out = pmsig(
        &dir.0,
        &[
            "propose",
            "--proposal-id",
            "not-hex",
            "--recipient",
            RECIPIENT,
            "--amount",
            "1",
        ],
    );
    assert!(!out.ok);
    assert!(
        out.text.contains("hex"),
        "message must name the problem: {}",
        out.text
    );
    assert!(
        !out.text.contains("panicked"),
        "must not panic: {}",
        out.text
    );
}

#[test]
fn help_and_version_work() {
    let dir = TempDir::new("help");
    assert!(pmsig(&dir.0, &["--help"]).ok);
    assert!(pmsig(&dir.0, &["--version"]).ok);
}

/// A member's spending key must have a way in that does not put it in the process list.
///
/// `--member` is convenient and the demo uses it, but `ps` shows the full argument list to every
/// other process on the machine, and shells record it in history. For a tool whose whole subject is
/// not revealing which member acted, that deserved an alternative rather than a footnote.
#[test]
fn the_member_key_can_be_given_without_putting_it_in_the_process_list() {
    let dir = TempDir::new("member-file");
    create_2_of_3(&dir.0);
    propose(&dir.0, P1);

    let key_path = dir.0.join("alice.key");
    std::fs::write(&key_path, format!("{A}\n")).expect("write key file");

    let out = pmsig(
        &dir.0,
        &[
            "approve",
            "--proposal-id",
            P1,
            "--member-file",
            key_path.to_str().unwrap(),
        ],
    );
    assert!(out.ok, "--member-file must work: {}", out.text);
    assert!(
        !out.text.contains(A),
        "the key must never be echoed back: {}",
        out.text
    );

    // Passing the key inline still works — the demo relies on it — but says what it costs.
    let dir2 = TempDir::new("member-inline");
    create_2_of_3(&dir2.0);
    propose(&dir2.0, P1);
    let inline = pmsig(&dir2.0, &["approve", "--proposal-id", P1, "--member", A]);
    assert!(inline.ok, "--member must still work: {}", inline.text);
    assert!(
        inline.text.contains("process list"),
        "--member must warn that it exposes the key: {}",
        inline.text
    );

    // The two are mutually exclusive, so no script can pass both and be left guessing which won.
    let both = pmsig(
        &dir.0,
        &[
            "approve",
            "--proposal-id",
            P1,
            "--member",
            A,
            "--member-file",
            key_path.to_str().unwrap(),
        ],
    );
    assert!(!both.ok, "--member and --member-file must conflict");
}
