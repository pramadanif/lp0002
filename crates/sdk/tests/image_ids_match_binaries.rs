#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! `artifacts/IMAGE_IDS.md` must describe the binaries actually committed next to it.
//!
//! On LEZ a program's `ProgramId` **is** the risc0 ImageID of its binary, so this file is not a
//! convenience: it is the identifier that must appear on chain, the one quoted in the submission,
//! and the one `deploy-testnet.sh` and `verify-onchain.sh` compare against. Nothing checked it.
//!
//! It would fail quietly rather than loudly. The executor tests read the `ProgramId` out of this
//! document instead of computing it, and the guest is *told* its own id through `write(&pid)` — so
//! a wrong value is used consistently everywhere inside the test, every PDA still derives against
//! it, and the suite stays green while the addresses on a real chain would be different ones.
//!
//! This is the same family as the stale-binary gap `scripts/check-guests-fresh.sh` covers:
//! committed artefacts drifting away from what they claim to describe. That script compares a
//! binary against its *sources*; this compares the recorded id against the *binary*.

use risc0_binfmt::ProgramBinary;
use risc0_zkvm::compute_image_id;

/// The ImageID recorded for `name` in IMAGE_IDS.md, as lowercase hex.
fn recorded(doc: &str, name: &str) -> String {
    let section = doc
        .split(&format!("## `{name}`"))
        .nth(1)
        .unwrap_or_else(|| panic!("no `{name}` section in artifacts/IMAGE_IDS.md"));
    let line = section
        .lines()
        .find(|l| l.contains("ImageID"))
        .unwrap_or_else(|| panic!("no ImageID row for `{name}`"));
    line.split('`')
        .nth(1)
        .unwrap_or_else(|| panic!("ImageID row for `{name}` has no backticked value"))
        .trim()
        .to_lowercase()
}

/// The ImageID of a committed `ProgramBinary`, computed the way `pmsig-image-id` does.
fn computed(path: &str) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    // The committed artefact is already an encoded ProgramBinary, not a bare ELF.
    let id = ProgramBinary::decode(&bytes)
        .map(|b| compute_image_id(&b.encode()).expect("image id of decoded binary"))
        .unwrap_or_else(|_| compute_image_id(&bytes).expect("image id"));
    format!("{id}").to_lowercase()
}

#[test]
fn image_ids_md_matches_the_committed_binaries() {
    let doc = std::fs::read_to_string("../../artifacts/IMAGE_IDS.md").expect("IMAGE_IDS.md");
    for (name, path) in [
        ("multisig", "../../artifacts/multisig.bin"),
        ("membership", "../../artifacts/membership.bin"),
    ] {
        let want = recorded(&doc, name);
        let got = computed(path);
        assert_eq!(
            want, got,
            "artifacts/IMAGE_IDS.md records {want} for `{name}`, but {path} has ImageID {got}. \
             The recorded id is what must appear on chain — re-run ./scripts/build-guests.sh"
        );
    }
}

/// The `ProgramId` word array must be the same value as the hex ImageID, since the executor tests
/// read the words and the chain reports the hex.
#[test]
fn the_program_id_words_agree_with_the_hex_image_id() {
    let doc = std::fs::read_to_string("../../artifacts/IMAGE_IDS.md").expect("IMAGE_IDS.md");
    for name in ["multisig", "membership"] {
        let section = doc.split(&format!("## `{name}`")).nth(1).unwrap();
        let line = section.lines().find(|l| l.contains("ProgramId")).unwrap();
        let inner = line.split('[').nth(1).unwrap().split(']').next().unwrap();
        let words: Vec<u32> = inner
            .split(',')
            .map(|x| x.trim().parse().unwrap())
            .collect();
        let hex: String = words
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            hex,
            recorded(&doc, name),
            "the ProgramId words and the hex ImageID disagree for `{name}`"
        );
    }
}
