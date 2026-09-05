#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! **P-R3** — the error catalogue and the code must agree, *in both directions*.
//!
//! `MultisigError` already asserts that every variant it defines appears in
//! `docs/error-codes.md`. That direction alone is not enough: it says nothing about a documented
//! code with no error behind it, and the catalogue had accumulated four of them — `2005
//! AccountNotLive`, `2008 SequencerUnreachable`, `2009 SequencerRejected` and `2011 ConfigMismatch`.
//! Three described a sequencer transport `pmsig-sdk` does not have. A member could have waited
//! forever for a code nothing can raise, and an evaluator grepping the catalogue would have found
//! the gap before we did.
//!
//! So this test reads the catalogue and requires every documented client code to be a code some
//! error type actually renders.

use std::collections::BTreeMap;

const DOC: &str = include_str!("../../../docs/error-codes.md");

/// `(code, name)` for every `2xxx` row in the catalogue's tables, excluding the retired list.
fn documented_client_codes() -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    let mut in_retired = false;
    for line in DOC.lines() {
        if line.starts_with("### ") {
            in_retired = line.contains("Retired");
        } else if line.starts_with("## ") {
            in_retired = false;
        }
        if in_retired || !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 3 {
            continue;
        }
        let code = cells[1].trim_matches('*').trim();
        // Only catalogue rows: their name cell is in backticks. The coverage table further down
        // also has a code in column 1, but column 2 there is prose ("store round-trip + corruption
        // test"), and reading it as a name made this test fail on its own parser.
        if !(cells[2].starts_with('`') && cells[2].ends_with('`')) {
            continue;
        }
        let name = cells[2].trim_matches('`').trim();
        if let Ok(n) = code.parse::<u32>() {
            if (2000..3000).contains(&n) {
                out.insert(n, name.to_string());
            }
        }
    }
    out
}

/// Every `"<code> <Name>"` an error type in this workspace can actually render.
///
/// Built from the errors themselves rather than a hand-kept list, so a variant that stops being
/// constructible is still counted here — what this test is for is the *catalogue* claiming errors
/// that do not exist, not reachability.
fn implemented_client_codes() -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    let mut add = |rendered: String| {
        let mut parts = rendered.splitn(2, ' ');
        if let (Some(code), Some(rest)) = (parts.next(), parts.next()) {
            if let Ok(n) = code.parse::<u32>() {
                let name = rest.split(':').next().unwrap_or(rest).trim().to_string();
                out.insert(n, name);
            }
        }
    };

    for e in [
        pmsig_sdk::SdkError::ProofGenerationFailed(String::new()),
        pmsig_sdk::SdkError::ProverNotFound,
        pmsig_sdk::SdkError::DevModeRefused,
        pmsig_sdk::SdkError::NotAMember,
        pmsig_sdk::SdkError::AlreadyApproved,
        pmsig_sdk::SdkError::StaleProposal,
    ] {
        add(e.to_string());
    }

    // The store renders its own 2xxx code (2010 StoreCorrupt) and is part of the client surface.
    add(pmsig_store::StoreError::Corrupt {
        path: std::path::PathBuf::from("/x"),
        source: serde_json::from_slice::<u8>(b"{").unwrap_err(),
    }
    .to_string());

    out
}

#[test]
fn every_documented_client_code_is_implemented() {
    let documented = documented_client_codes();
    let implemented = implemented_client_codes();
    assert!(
        !documented.is_empty(),
        "parsed no 2xxx rows out of docs/error-codes.md — the parser, not the catalogue, is wrong"
    );

    let mut phantom = Vec::new();
    for (code, name) in &documented {
        match implemented.get(code) {
            None => phantom.push(format!("{code} {name} — documented, nothing raises it")),
            Some(actual) if actual != name => {
                phantom.push(format!(
                    "{code} is `{name}` in the docs but `{actual}` in code"
                ));
            }
            Some(_) => {}
        }
    }
    assert!(
        phantom.is_empty(),
        "docs/error-codes.md documents client codes that do not exist:\n  {}",
        phantom.join("\n  ")
    );
}

#[test]
fn every_implemented_client_code_is_documented() {
    let documented = documented_client_codes();
    let mut undocumented = Vec::new();
    for (code, name) in implemented_client_codes() {
        if !documented.contains_key(&code) {
            undocumented.push(format!("{code} {name}"));
        }
    }
    assert!(
        undocumented.is_empty(),
        "these client errors exist but are not in docs/error-codes.md:\n  {}",
        undocumented.join("\n  ")
    );
}
