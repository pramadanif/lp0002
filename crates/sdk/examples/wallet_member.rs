//! Builds a member set that includes a **real** shielded account from a LEZ wallet.
//!
//! Two jobs:
//!
//! 1. Emit the parameters and the Borsh-encoded witness needed to drive a genuine `approve`.
//! 2. **Cross-check our LEZ-compatible derivation against a wallet-generated account.** The wallet
//!    created the account; we re-derive its id from `(npk, vpk, identifier)` using our own code and
//!    assert they match. That is a far stronger check of SC-B.7 than a pinned vector, because the
//!    account was produced by an independent implementation.
//!
//! Secrets are written to a file, never printed: the witness contains the member's `nsk`.
//!
//! ```text
//! cargo run -p pmsig-sdk --example wallet_member -- <wallet-storage.json> <out-dir>
//! ```

use std::path::PathBuf;

use lee_core::{account::AccountId, encryption::ViewingPublicKey};
use pmsig_core::{tree::MemberTree, Digest32};
use pmsig_membership_core::{verify::npk_of, ApprovalWitness};
use serde_json::Value;

const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];
/// Two stand-in co-members. Only their npks matter; nobody needs their secrets to approve.
const CO_MEMBERS: [Digest32; 2] = [[0x22; 32], [0x33; 32]];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let storage: PathBuf = args.next().ok_or("usage: wallet_member <storage.json> <out-dir>")?.into();
    let out_dir: PathBuf = args.next().ok_or("usage: wallet_member <storage.json> <out-dir>")?.into();

    let doc: Value = serde_json::from_slice(&std::fs::read(&storage)?)?;
    let account = doc["key_chain"]["accounts"]
        .as_array()
        .ok_or("no accounts")?
        .iter()
        .find_map(|a| a.get("Private"))
        .ok_or("the wallet has no private (shielded) account")?;

    let account_id: AccountId = account["account_id"].as_str().ok_or("no account_id")?.parse()?;
    let key = &account["data"]["value"][0];
    let nsk = bytes32(&key["private_key_holder"]["nullifier_secret_key"])?;
    let vpk_bytes = byte_vec(&key["viewing_public_key"])?;
    let wallet_npk = bytes32(&key["nullifier_public_key"])?;

    // --- cross-check 1: our npk derivation reproduces the wallet's own npk ---
    let derived_npk = npk_of(&nsk).to_byte_array();
    if derived_npk != wallet_npk {
        return Err("our npk derivation disagrees with the wallet's".into());
    }

    // --- cross-check 2: our account-id derivation reproduces the wallet's account ---
    let vpk = decode_vpk(&vpk_bytes)?;
    let identifier = (0_u128..4)
        .find(|i| AccountId::for_regular_private_account(&npk_of(&nsk), &vpk, *i) == account_id)
        .ok_or("could not reproduce the wallet's account id for any identifier in 0..4")?;

    println!("CROSSCHECK_NPK=ok");
    println!("CROSSCHECK_ACCOUNT_ID=ok");
    println!("IDENTIFIER={identifier}");
    println!("APPROVER_ACCOUNT_ID={account_id}");

    // --- the member set: the wallet's real account plus two stand-ins ---
    let mut npks = vec![derived_npk];
    npks.extend(CO_MEMBERS.iter().map(|n| npk_of(n).to_byte_array()));
    let tree = MemberTree::new(&npks).ok_or("member tree")?;
    let path = tree.path(0).ok_or("wallet member has no path")?;

    let verifier = read_membership_image_id()?;
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &verifier);

    println!("MEMBER_ROOT={}", hex::encode(tree.root()));
    println!("CONFIG_HASH={}", hex::encode(config_hash));
    println!("MULTISIG_ID={}", hex::encode(MULTISIG_ID));
    println!("PROPOSAL_ID={}", hex::encode(PROPOSAL_ID));
    println!(
        "PROPOSAL_SEED={}",
        hex::encode(pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID))
    );
    println!(
        "VERIFIER={}",
        verifier.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    );
    println!(
        "NULLIFIER={}",
        hex::encode(pmsig_core::approval_nullifier(&nsk, &MULTISIG_ID, &PROPOSAL_ID))
    );

    // --- the witness: secret, so it goes to a file the caller reads, not to stdout ---
    let witness = ApprovalWitness {
        nsk,
        vpk,
        identifier,
        member_index: 0,
        siblings: path.siblings,
    };
    let encoded = borsh::to_vec(&witness)?;
    std::fs::create_dir_all(&out_dir)?;
    let witness_path = out_dir.join("witness.csv");
    let csv = encoded.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
    std::fs::write(&witness_path, &csv)?;
    println!("WITNESS_FILE={}", witness_path.display());
    println!("WITNESS_BYTES={}", encoded.len());
    Ok(())
}

fn bytes32(v: &Value) -> Result<Digest32, Box<dyn std::error::Error>> {
    Ok(<Digest32>::try_from(byte_vec(v)?.as_slice())?)
}

fn byte_vec(v: &Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    v.as_array()
        .ok_or("expected a byte array")?
        .iter()
        .map(|b: &Value| -> Result<u8, Box<dyn std::error::Error>> {
            let n = b.as_u64().ok_or("non-numeric byte")?;
            Ok(u8::try_from(n)?)
        })
        .collect()
}

/// `ViewingPublicKey::from_bytes` is host-only but available here; keep the length check explicit.
fn decode_vpk(bytes: &[u8]) -> Result<ViewingPublicKey, Box<dyn std::error::Error>> {
    if bytes.len() != pmsig_membership_core::VIEWING_PUBLIC_KEY_LEN {
        return Err(format!("viewing key is {} bytes", bytes.len()).into());
    }
    ViewingPublicKey::from_bytes(bytes.to_vec()).map_err(|e| format!("{e:?}").into())
}

fn read_membership_image_id() -> Result<[u32; 8], Box<dyn std::error::Error>> {
    let doc = std::fs::read_to_string("artifacts/IMAGE_IDS.md")?;
    let section = doc.split("## `membership`").nth(1).ok_or("no membership section")?;
    let line = section.lines().find(|l| l.contains("ProgramId")).ok_or("no ProgramId row")?;
    let inner = line.split('[').nth(1).and_then(|s| s.split(']').next()).ok_or("malformed")?;
    let words: Vec<u32> = inner.split(',').map(|w| w.trim().parse::<u32>()).collect::<Result<_, _>>()?;
    Ok(<[u32; 8]>::try_from(words.as_slice())?)
}
