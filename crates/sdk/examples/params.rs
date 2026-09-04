//! Derives the concrete parameters an operator needs to drive the SPEL CLI.
//!
//! The multisig's address depends on `config_hash`, which commits to the member root, the
//! threshold **and** the membership verifier's program id (ADR-002). Getting any of those wrong
//! yields a different address, so they are computed here from one place rather than by hand.
//!
//! ```text
//! cargo run -p pmsig-sdk --example params -- <membership-program-id-hex-or-words>
//! ```
//!
//! With no argument it reads the membership ImageID from `artifacts/IMAGE_IDS.md`.

use pmsig_core::{tree::MemberTree, Digest32};
use pmsig_membership_core::verify::npk_of;

/// The demo member set. Real members hold their own keys and share only their npk.
const MEMBERS: [Digest32; 3] = [[0x11; 32], [0x22; 32], [0x33; 32]];
const MULTISIG_ID: Digest32 = [0xA1; 32];
const PROPOSAL_ID: Digest32 = [0xB2; 32];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let verifier = match std::env::args().nth(1) {
        Some(arg) => parse_program_id(&arg)?,
        None => read_membership_image_id()?,
    };

    let npks: Vec<Digest32> = MEMBERS.iter().map(|n| npk_of(n).to_byte_array()).collect();
    let tree = MemberTree::new(&npks).ok_or("a multisig needs at least one member")?;
    let config_hash = pmsig_core::config_hash(&tree.root(), 2, 3, &MULTISIG_ID, &verifier);
    let proposal_seed = pmsig_core::proposal_seed(&config_hash, &PROPOSAL_ID);

    println!("MEMBER_ROOT={}", hex::encode(tree.root()));
    println!("CONFIG_HASH={}", hex::encode(config_hash));
    println!("MULTISIG_ID={}", hex::encode(MULTISIG_ID));
    println!("PROPOSAL_ID={}", hex::encode(PROPOSAL_ID));
    println!("PROPOSAL_SEED={}", hex::encode(proposal_seed));
    println!(
        "VERIFIER={}",
        verifier
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    for (i, nsk) in MEMBERS.iter().enumerate() {
        println!(
            "MEMBER{}_NULLIFIER={}",
            i,
            hex::encode(pmsig_core::approval_nullifier(
                nsk,
                &MULTISIG_ID,
                &PROPOSAL_ID
            ))
        );
    }
    Ok(())
}

/// Accepts either comma-separated u32 words or a 64-char hex ImageID.
fn parse_program_id(arg: &str) -> Result<[u32; 8], Box<dyn std::error::Error>> {
    if arg.contains(',') {
        let words: Vec<u32> = arg
            .split(',')
            .map(|w| w.trim().parse::<u32>())
            .collect::<Result<_, _>>()?;
        return Ok(<[u32; 8]>::try_from(words.as_slice())?);
    }
    let bytes = hex::decode(arg)?;
    let mut out = [0_u32; 8];
    for (word, chunk) in out.iter_mut().zip(bytes.chunks_exact(4)) {
        *word = u32::from_le_bytes(<[u8; 4]>::try_from(chunk)?);
    }
    Ok(out)
}

/// Reads the membership `ProgramId` recorded by `scripts/build-guests.sh`.
fn read_membership_image_id() -> Result<[u32; 8], Box<dyn std::error::Error>> {
    let doc = std::fs::read_to_string("artifacts/IMAGE_IDS.md").map_err(|e| {
        format!("artifacts/IMAGE_IDS.md: {e} — run ./scripts/build-guests.sh first")
    })?;
    let section = doc
        .split("## `membership`")
        .nth(1)
        .ok_or("no `membership` section in artifacts/IMAGE_IDS.md")?;
    let line = section
        .lines()
        .find(|l| l.contains("ProgramId"))
        .ok_or("no ProgramId row for `membership`")?;
    let inner = line
        .split('[')
        .nth(1)
        .and_then(|s| s.split(']').next())
        .ok_or("malformed ProgramId row")?;
    parse_program_id(inner)
}
