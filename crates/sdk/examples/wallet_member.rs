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
/// One stand-in co-member so N=3 with two real wallet accounts. Only its npk matters; nobody needs
/// its secret, and it never approves.
const CO_MEMBER: Digest32 = [0x33; 32];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let storage: PathBuf = args
        .next()
        .ok_or("usage: wallet_member <storage.json> <out-dir>")?
        .into();
    let out_dir: PathBuf = args
        .next()
        .ok_or("usage: wallet_member <storage.json> <out-dir>")?
        .into();

    let doc: Value = serde_json::from_slice(&std::fs::read(&storage)?)?;
    let privates: Vec<&Value> = dig(&doc, &["key_chain", "accounts"])?
        .as_array()
        .ok_or("`key_chain.accounts` is not an array")?
        .iter()
        .filter_map(|a| a.get("Private"))
        .collect();
    if privates.len() < 2 {
        return Err(format!(
            "need at least 2 shielded accounts for a 2-of-3 demo, wallet has {}. \
             Create one with: wallet account new private",
            privates.len()
        )
        .into());
    }

    // Read each real member's key material and cross-check our derivation against the wallet's.
    struct Member {
        nsk: Digest32,
        vpk: ViewingPublicKey,
        identifier: u128,
        account_id: AccountId,
        npk: Digest32,
    }
    let mut members = Vec::new();
    for account in privates.iter().take(2) {
        let account_id: AccountId = dig(account, &["account_id"])?
            .as_str()
            .ok_or("account_id is not a string")?
            .parse()?;
        let key = dig(account, &["data", "value"])?
            .as_array()
            .and_then(|v| v.first())
            .ok_or("the account has no key material")?;
        let nsk = bytes32(dig(key, &["private_key_holder", "nullifier_secret_key"])?)?;
        let vpk = decode_vpk(&byte_vec(dig(key, &["viewing_public_key"])?)?)?;
        let wallet_npk = bytes32(dig(key, &["nullifier_public_key"])?)?;

        let npk = npk_of(&nsk).to_byte_array();
        if npk != wallet_npk {
            return Err("our npk derivation disagrees with the wallet's".into());
        }
        let identifier = (0_u128..8)
            .find(|i| AccountId::for_regular_private_account(&npk_of(&nsk), &vpk, *i) == account_id)
            .ok_or("could not reproduce the wallet's account id for any identifier in 0..8")?;

        members.push(Member {
            nsk,
            vpk,
            identifier,
            account_id,
            npk,
        });
    }
    println!("CROSSCHECK_NPK=ok");
    println!("CROSSCHECK_ACCOUNT_ID=ok");

    // Member set: both real wallet accounts plus one stand-in, giving 2-of-3.
    let mut npks: Vec<Digest32> = members.iter().map(|m| m.npk).collect();
    npks.push(npk_of(&CO_MEMBER).to_byte_array());
    let tree = MemberTree::new(&npks).ok_or("member tree")?;

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
        verifier
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    std::fs::create_dir_all(&out_dir)?;
    for (i, m) in members.iter().enumerate() {
        let path = tree.path(i).ok_or("member has no path")?;
        let witness = ApprovalWitness {
            nsk: m.nsk,
            vpk: m.vpk.clone(),
            identifier: m.identifier,
            member_index: i as u64,
            siblings: path.siblings,
        };
        let encoded = borsh::to_vec(&witness)?;
        let wp = out_dir.join(format!("witness{i}.csv"));
        std::fs::write(
            &wp,
            encoded
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )?;
        println!("MEMBER{i}_ACCOUNT={}", m.account_id);
        println!(
            "MEMBER{i}_NULLIFIER={}",
            hex::encode(pmsig_core::approval_nullifier(
                &m.nsk,
                &MULTISIG_ID,
                &PROPOSAL_ID
            ))
        );
        println!("MEMBER{i}_WITNESS={}", wp.display());
    }
    Ok(())
}

/// Walks a JSON path, naming the field that is missing rather than yielding a silent `Null`.
fn dig<'a>(v: &'a Value, path: &[&str]) -> Result<&'a Value, Box<dyn std::error::Error>> {
    let mut cur = v;
    for key in path {
        cur = cur
            .get(key)
            .ok_or_else(|| format!("wallet storage has no `{}`", path.join(".")))?;
    }
    Ok(cur)
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
    let section = doc
        .split("## `membership`")
        .nth(1)
        .ok_or("no membership section")?;
    let line = section
        .lines()
        .find(|l| l.contains("ProgramId"))
        .ok_or("no ProgramId row")?;
    let inner = line
        .split('[')
        .nth(1)
        .and_then(|s| s.split(']').next())
        .ok_or("malformed")?;
    let words: Vec<u32> = inner
        .split(',')
        .map(|w| w.trim().parse::<u32>())
        .collect::<Result<_, _>>()?;
    Ok(<[u32; 8]>::try_from(words.as_slice())?)
}
