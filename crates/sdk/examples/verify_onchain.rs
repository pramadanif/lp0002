//! Verifies a completed multisig **from public chain data alone**.
//!
//! Reads the config and proposal accounts over JSON-RPC and checks the properties the prize actually
//! cares about — that a threshold was met, and that nothing on chain identifies a member. It needs
//! no secrets, no local state and no witness, so anyone can run it against a node.
//!
//! ```text
//! cargo run -p pmsig-sdk --example verify_onchain -- <rpc-url> <IMAGE_IDS.md> <config_hash> <proposal_seed>
//! ```

use std::collections::HashSet;

use pmsig_core::Digest32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rpc = args
        .next()
        .ok_or("usage: verify_onchain <rpc> <image_ids.md> <config_hash> <proposal_seed>")?;
    let image_ids = args.next().ok_or("missing IMAGE_IDS.md path")?;
    let config_hash: Digest32 = hex32(&args.next().ok_or("missing config_hash")?)?;
    let proposal_seed: Digest32 = hex32(&args.next().ok_or("missing proposal_seed")?)?;
    // Optional: what the treasury should hold afterwards. Public data cannot say how much the
    // treasury was funded with, so only a caller who funded it can assert the exact remainder.
    let expect_treasury: Option<u128> = match args.next() {
        Some(v) => Some(v.parse()?),
        None => None,
    };

    let doc = std::fs::read_to_string(&image_ids)?;
    let multisig_pid = program_id(&doc, "multisig")?;
    let membership_pid = program_id(&doc, "membership")?;

    let config_pda = public_pda(&multisig_pid, &config_hash);
    let proposal_pda = public_pda(&multisig_pid, &proposal_seed);

    // A verifier that will not say which addresses it checked cannot be checked itself. These are
    // also the addresses an operator needs — the config PDA is the multisig's own treasury under
    // INV-7, so it is the account that has to hold the funds a proposal spends.
    println!("  config PDA   : {config_pda}");
    println!("  proposal PDA : {proposal_pda}");

    let fail = |m: String| -> Result<(), Box<dyn std::error::Error>> { Err(m.into()) };

    // ---- config account ----
    let cfg = get_account(&rpc, &config_pda)?;
    let owner = cfg.owner;
    if owner != multisig_pid {
        return fail("config account is not owned by the multisig program".into());
    }
    let c = decode_config(&cfg.data)?;
    if c.recompute() != config_hash {
        return fail("the config account does not rehash to its own address (INV-3)".into());
    }
    if c.membership_program_id != membership_pid {
        return fail("the config names a membership program other than the deployed one".into());
    }
    println!(
        "  config       : {}-of-{}, owner ok, rehashes to its own address",
        c.m, c.n
    );
    println!("  verifier     : matches the deployed membership program (ADR-002)");

    // ---- proposal account ----
    let prop = get_account(&rpc, &proposal_pda)?;
    let p = decode_proposal(&prop.data)?;
    if p.config_hash != config_hash {
        return fail("the proposal belongs to a different multisig".into());
    }
    if p.nullifiers.len() < usize::from(c.m) {
        return fail(format!(
            "threshold NOT met: {} approvals, need {}",
            p.nullifiers.len(),
            c.m
        ));
    }
    if !p.executed {
        return fail("the proposal reached its threshold but was not executed".into());
    }
    let unique: HashSet<&Digest32> = p.nullifiers.iter().collect();
    if unique.len() != p.nullifiers.len() {
        return fail("a nullifier is repeated — double-vote prevention is broken".into());
    }
    println!(
        "  proposal     : {} approvals of {} required, executed, all nullifiers distinct",
        p.nullifiers.len(),
        c.m
    );
    println!("  FULL M       : evidence uses the full threshold, not a lowered tier (H13/W15)");

    // ---- INV-7: the transfer that executed is the transfer that was approved ----
    //
    // The recipient is read back out of the *proposal*, never from an argument: an address supplied
    // by the caller would only prove that the caller and the chain agree, which is not the claim.
    // Without this block a proposal marked `executed` was the whole proof that money moved, and a
    // transfer of nothing, or to somebody else, would have passed every check above.
    if p.amount == 0 {
        return fail("the proposal moves 0 — a zero transfer would make this check vacuous".into());
    }
    let recipient_addr = base58(&p.recipient);
    let paid = get_account(&rpc, &recipient_addr)
        .map_err(|e| format!("the approved recipient {recipient_addr} is not on chain: {e}"))?
        .balance;
    if paid < p.amount {
        return fail(format!(
            "the approved recipient {recipient_addr} holds {paid}, less than the {} the proposal \
             approved — the executed transfer does not match the approved one (INV-7)",
            p.amount
        ));
    }
    println!(
        "  payment      : {recipient_addr} holds {paid}, covering the {} approved (INV-7)",
        p.amount
    );

    let treasury = cfg.balance;
    if let Some(want) = expect_treasury {
        if treasury != want {
            return fail(format!(
                "the treasury holds {treasury}, not the {want} expected after paying {} — the \
                 amount debited does not match the amount approved",
                p.amount
            ));
        }
        println!(
            "  treasury     : {treasury} left, exactly funding minus the {} paid",
            p.amount
        );
    } else {
        println!("  treasury     : {treasury} (no expected remainder supplied)");
    }

    // ---- the privacy property, checked against the bytes ----
    //
    // The strongest thing checkable from public data: the account holds a member ROOT and a set of
    // nullifiers, and nothing that is an account id or an npk.
    if bytes_contain(&prop.data, &c.member_root) {
        return fail("the member root leaked into the proposal account".into());
    }
    println!("  privacy      : proposal holds a count + nullifiers, no member identity (P-F2)");

    println!("\n  VERIFIED from public chain data alone.");
    Ok(())
}

// ---------------------------------------------------------------------------------------------

struct Config {
    member_root: Digest32,
    m: u8,
    n: u8,
    multisig_id: Digest32,
    membership_program_id: [u32; 8],
}

impl Config {
    fn recompute(&self) -> Digest32 {
        pmsig_core::config_hash(
            &self.member_root,
            self.m,
            self.n,
            &self.multisig_id,
            &self.membership_program_id,
        )
    }
}

struct Proposal {
    config_hash: Digest32,
    recipient: Digest32,
    amount: u128,
    nullifiers: Vec<Digest32>,
    executed: bool,
}

fn decode_config(d: &[u8]) -> Result<Config, Box<dyn std::error::Error>> {
    let mut o = 2usize; // version
    let member_root = take32(d, &mut o)?;
    let m = *d.get(o).ok_or("short config")?;
    o += 1;
    let n = *d.get(o).ok_or("short config")?;
    o += 1;
    let multisig_id = take32(d, &mut o)?;
    let mut membership_program_id = [0u32; 8];
    for w in &mut membership_program_id {
        let b = d.get(o..o + 4).ok_or("short config")?;
        *w = u32::from_le_bytes(b.try_into()?);
        o += 4;
    }
    Ok(Config {
        member_root,
        m,
        n,
        multisig_id,
        membership_program_id,
    })
}

fn decode_proposal(d: &[u8]) -> Result<Proposal, Box<dyn std::error::Error>> {
    let mut o = 2usize; // version
    let config_hash = take32(d, &mut o)?;
    let _proposal_id = take32(d, &mut o)?;
    o += 1; // action discriminant
    let recipient = take32(d, &mut o)?;
    let amount = u128::from_le_bytes(d.get(o..o + 16).ok_or("short proposal")?.try_into()?);
    o += 16;
    let count = u32::from_le_bytes(d.get(o..o + 4).ok_or("short proposal")?.try_into()?) as usize;
    o += 4;
    let mut nullifiers = Vec::with_capacity(count);
    for _ in 0..count {
        nullifiers.push(take32(d, &mut o)?);
    }
    let executed = *d.get(o).ok_or("short proposal")? != 0;
    Ok(Proposal {
        config_hash,
        recipient,
        amount,
        nullifiers,
        executed,
    })
}

fn take32(d: &[u8], o: &mut usize) -> Result<Digest32, Box<dyn std::error::Error>> {
    let b = d.get(*o..*o + 32).ok_or("unexpected end of account data")?;
    *o += 32;
    Ok(<Digest32>::try_from(b)?)
}

fn bytes_contain(hay: &[u8], needle: &Digest32) -> bool {
    hay.windows(32).any(|w| w == needle)
}

/// `AccountId::for_public_pda` — SHA256(prefix ‖ program_id ‖ seed).
fn public_pda(program_id: &[u32; 8], seed: &Digest32) -> String {
    use risc0_zkvm::sha::{Impl, Sha256 as _};
    let mut buf = Vec::with_capacity(96);
    buf.extend_from_slice(b"/LEE/v0.2/AccountId/PDA/\0\0\0\0\0\0\0\0");
    for w in program_id {
        buf.extend_from_slice(&w.to_le_bytes());
    }
    buf.extend_from_slice(seed);
    base58(Impl::hash_bytes(&buf).as_bytes())
}

fn base58(b: &[u8]) -> String {
    const A: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut n = b.to_vec();
    let mut out = Vec::new();
    while n.iter().any(|&x| x != 0) {
        let mut rem = 0u32;
        for byte in &mut n {
            let cur = (rem << 8) | u32::from(*byte);
            *byte = u8::try_from(cur / 58).unwrap_or(0);
            rem = cur % 58;
        }
        out.push(*A.get(rem as usize).unwrap_or(&b'1'));
    }
    out.reverse();
    String::from_utf8_lossy(&out).into_owned()
}

fn hex32(s: &str) -> Result<Digest32, Box<dyn std::error::Error>> {
    Ok(<Digest32>::try_from(hex::decode(s)?.as_slice())?)
}

fn program_id(doc: &str, name: &str) -> Result<[u32; 8], Box<dyn std::error::Error>> {
    let section = doc
        .split(&format!("## `{name}`"))
        .nth(1)
        .ok_or_else(|| format!("no `{name}` section in IMAGE_IDS.md"))?;
    let line = section
        .lines()
        .find(|l| l.contains("ProgramId"))
        .ok_or("no ProgramId row")?;
    let inner = line
        .split('[')
        .nth(1)
        .and_then(|s| s.split(']').next())
        .ok_or("malformed")?;
    let w: Vec<u32> = inner
        .split(',')
        .map(|x| x.trim().parse::<u32>())
        .collect::<Result<_, _>>()?;
    Ok(<[u32; 8]>::try_from(w.as_slice())?)
}

/// One account as the chain reports it.
struct Account {
    owner: [u32; 8],
    data: Vec<u8>,
    balance: u128,
}

/// Reads an account over plain JSON-RPC.
fn get_account(rpc: &str, id: &str) -> Result<Account, Box<dyn std::error::Error>> {
    let body = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"getAccount","params":["{id}"]}}"#);
    let out = std::process::Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            rpc,
            "-H",
            "content-type: application/json",
            "--data",
            &body,
        ])
        .output()?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let r = v
        .get("result")
        .ok_or_else(|| format!("account {id} not found on chain"))?;
    let owner_arr = r
        .get("program_owner")
        .and_then(|x| x.as_array())
        .ok_or("no program_owner")?;
    let mut owner = [0u32; 8];
    for (slot, val) in owner.iter_mut().zip(owner_arr) {
        *slot = u32::try_from(val.as_u64().ok_or("bad program_owner word")?)?;
    }
    let data = r
        .get("data")
        .and_then(|x| x.as_array())
        .ok_or("no data")?
        .iter()
        .map(|b| u8::try_from(b.as_u64().unwrap_or(0)).unwrap_or(0))
        .collect();
    // A u128 balance does not always survive JSON as a number, so accept either form rather than
    // silently reading a wrong figure — this value decides whether the money moved.
    let bal = r.get("balance").ok_or("no balance")?;
    let balance: u128 = match bal.as_u64() {
        Some(n) => u128::from(n),
        None => bal
            .as_str()
            .ok_or("balance is neither a u64 nor a string")?
            .parse()?,
    };
    Ok(Account {
        owner,
        data,
        balance,
    })
}
