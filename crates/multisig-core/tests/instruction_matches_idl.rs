#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: panicking is how a test reports failure"
)]
//! The instruction enum and the generated IDL must agree.
//!
//! `#[lez_program(instruction = "…")]` dispatches on variant and field **names**. If a name here
//! drifts from the handler signature, dispatch breaks — and it breaks at runtime, on chain, not at
//! compile time. The IDL is generated from the handlers, so comparing against it catches the drift
//! here instead.

use pmsig_multisig_core::Instruction;

fn idl() -> serde_json::Value {
    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../artifacts/multisig-idl.json"
    ))
    .expect("artifacts/multisig-idl.json — run ./scripts/generate-idl.sh");
    serde_json::from_str(&raw).expect("the IDL must be valid JSON")
}

/// Every instruction in the IDL must exist as a variant, and vice versa.
#[test]
fn every_idl_instruction_has_a_variant() {
    let idl = idl();
    let idl_names: Vec<String> = idl["instructions"]
        .as_array()
        .expect("instructions array")
        .iter()
        .map(|i| i["name"].as_str().expect("name").to_string())
        .collect();

    let variants = [
        Instruction::CreateMultisig {
            config_hash: [0; 32],
            member_root: [0; 32],
            m: 2,
            n: 3,
            multisig_id: [0; 32],
            membership_program_id: [0; 8],
        },
        Instruction::CreateProposal {
            config_hash: [0; 32],
            proposal_seed: [0; 32],
            proposal_id: [0; 32],
            recipient: [0; 32],
            amount: 1,
        },
        Instruction::Approve {
            config_hash: [0; 32],
            proposal_seed: [0; 32],
            member_root: [0; 32],
            claimed_nullifier: [0; 32],
            witness: vec![1],
        },
        Instruction::Execute {
            config_hash: [0; 32],
            proposal_seed: [0; 32],
        },
    ];
    let variant_names: Vec<&str> = variants.iter().map(Instruction::name).collect();

    for name in &idl_names {
        assert!(
            variant_names.contains(&name.as_str()),
            "IDL has instruction `{name}` with no matching enum variant — dispatch would fail"
        );
    }
    for name in &variant_names {
        assert!(
            idl_names.contains(&(*name).to_string()),
            "enum has variant `{name}` that the program does not expose"
        );
    }
    assert_eq!(idl_names.len(), variant_names.len());
}

/// Field names must match too — the macro binds handler arguments by name.
#[test]
fn every_idl_argument_has_a_matching_field() {
    let idl = idl();
    // Serialising a variant gives us its field names, which is what the macro matches on.
    let cases: Vec<(&str, serde_json::Value)> = vec![
        (
            "create_multisig",
            serde_json::to_value(Instruction::CreateMultisig {
                config_hash: [0; 32],
                member_root: [0; 32],
                m: 2,
                n: 3,
                multisig_id: [0; 32],
                membership_program_id: [0; 8],
            })
            .unwrap(),
        ),
        (
            "approve",
            serde_json::to_value(Instruction::Approve {
                config_hash: [0; 32],
                proposal_seed: [0; 32],
                member_root: [0; 32],
                claimed_nullifier: [0; 32],
                witness: vec![1],
            })
            .unwrap(),
        ),
        (
            "execute",
            serde_json::to_value(Instruction::Execute {
                config_hash: [0; 32],
                proposal_seed: [0; 32],
            })
            .unwrap(),
        ),
    ];

    for (ix_name, value) in cases {
        let ix = idl["instructions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == ix_name)
            .unwrap_or_else(|| panic!("IDL has no instruction `{ix_name}`"));
        let idl_args: Vec<&str> = ix["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();

        // serde serialises a struct variant as {"Variant": {field: ...}}
        let fields = value
            .as_object()
            .and_then(|o| o.values().next())
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| panic!("`{ix_name}` did not serialise as a struct variant"));

        for arg in &idl_args {
            assert!(
                fields.contains_key(*arg),
                "`{ix_name}`: IDL argument `{arg}` has no field on the enum variant — the macro \
                 would fail to bind it"
            );
        }
        assert_eq!(
            fields.len(),
            idl_args.len(),
            "`{ix_name}`: enum has {} fields, IDL has {} arguments",
            fields.len(),
            idl_args.len()
        );
    }
}

/// Only `approve` is privacy-preserving. If that ever changes silently, the security story changes
/// with it (ADR-001 D7), so it is asserted rather than assumed.
#[test]
fn only_approve_is_privacy_preserving() {
    assert!(Instruction::Approve {
        config_hash: [0; 32],
        proposal_seed: [0; 32],
        member_root: [0; 32],
        claimed_nullifier: [0; 32],
        witness: vec![1],
    }
    .is_privacy_preserving());

    assert!(!Instruction::Execute {
        config_hash: [0; 32],
        proposal_seed: [0; 32]
    }
    .is_privacy_preserving());
}
