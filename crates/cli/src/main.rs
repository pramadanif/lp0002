//! `pmsig` — CLI for shielded members of a private M-of-N multisig.
//!
//! Criterion **P-U1**. Every subcommand drives the same crates the on-chain program uses, so what
//! the CLI enforces and what the chain enforces cannot drift.
//!
//! # Local mode
//!
//! Commands run against a local state file (`--state`), which applies the real transition rules but
//! is **not a chain** — every such command prints a `[local]` marker. The sequencer transport lands
//! in Phase E; until then no CLI output is testnet evidence, and none of it pretends to be.

mod state;

use anyhow::{bail, Context as _, Result};
use clap::{Parser, Subcommand};
use lee_core::encryption::ViewingPublicKey;
use pmsig_core::{tree::MemberTree, Digest32};
use pmsig_membership_core::verify::npk_of;
use pmsig_multisig_core::{
    logic::{self, CreateMultisig},
    MultisigError, ProgramIdWords, ProposedAction,
};
use pmsig_sdk::member::{prepare_approval, MemberSecrets, MultisigView};
use pmsig_store::{ApprovalRecord, ApprovalStatus, ApprovalStore};
use state::LocalState;

#[derive(Parser)]
#[command(
    name = "pmsig",
    version,
    about = "Private M-of-N multisig for the Logos Execution Zone",
    long_about = "Create and operate a multisig whose members hold shielded LEZ accounts.\n\
                  Approvals reveal no identity to on-chain observers or to other members."
)]
struct Cli {
    /// Local state file used in place of a sequencer (Phase E adds the network transport).
    #[arg(long, global = true, default_value = ".pmsig/state.json")]
    state: std::path::PathBuf,

    /// Directory holding the client's approval store, so partial sets survive restarts.
    #[arg(long, global = true, default_value = ".pmsig")]
    store_dir: std::path::PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a multisig from a member set.
    Create {
        /// Member secret keys, hex, comma-separated. Demo convenience: real members hold their own.
        #[arg(long, value_delimiter = ',')]
        members: Vec<String>,
        /// Threshold M.
        #[arg(long)]
        m: u8,
        /// Multisig identifier, hex (32 bytes).
        #[arg(
            long,
            default_value = "a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1"
        )]
        multisig_id: String,
        /// Starting treasury balance, so a transfer has something to move.
        #[arg(long, default_value_t = 1_000_000)]
        treasury_balance: u128,
    },
    /// Submit a proposal to move treasury funds.
    Propose {
        #[arg(long)]
        proposal_id: String,
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        amount: u128,
    },
    /// Approve a proposal as one member, revealing nothing about which member.
    Approve {
        #[arg(long)]
        proposal_id: String,
        /// The approving member's secret key, hex.
        #[arg(long)]
        member: String,
    },
    /// Execute a proposal once the threshold is met.
    Execute {
        #[arg(long)]
        proposal_id: String,
    },
    /// Show threshold progress. Prints a count, never a list of who approved.
    Status {
        #[arg(long)]
        proposal_id: Option<String>,
    },
}

/// The membership verifier this build is bound to (ADR-002).
///
/// Read from `artifacts/IMAGE_IDS.md` at deployment time; the placeholder here keeps local mode
/// self-contained. Phase E wires the real deployed id.
const LOCAL_VERIFIER: ProgramIdWords = [7; 8];

fn main() {
    if let Err(e) = run() {
        // A member should see what happened and what to do, not a backtrace.
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Command::Create {
            members,
            m,
            multisig_id,
            treasury_balance,
        } => create(&cli, members, *m, multisig_id, *treasury_balance),
        Command::Propose {
            proposal_id,
            recipient,
            amount,
        } => propose(&cli, proposal_id, recipient, *amount),
        Command::Approve {
            proposal_id,
            member,
        } => approve(&cli, proposal_id, member),
        Command::Execute { proposal_id } => execute(&cli, proposal_id),
        Command::Status { proposal_id } => status(&cli, proposal_id.as_deref()),
    }
}

fn parse32(label: &str, s: &str) -> Result<Digest32> {
    let bytes = hex::decode(s).with_context(|| format!("{label} must be hex"))?;
    let arr: Digest32 = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("{label} must be exactly 32 bytes, got {}", bytes.len()))?;
    Ok(arr)
}

fn demo_vpk() -> ViewingPublicKey {
    ViewingPublicKey::from_seed(&[7_u8; 32], &[8_u8; 32])
}

fn create(cli: &Cli, members: &[String], m: u8, multisig_id: &str, treasury: u128) -> Result<()> {
    if members.is_empty() {
        bail!("at least one member is required");
    }
    let nsks: Vec<Digest32> = members
        .iter()
        .map(|s| parse32("member key", s))
        .collect::<Result<_>>()?;
    let npks: Vec<Digest32> = nsks.iter().map(|n| npk_of(n).to_byte_array()).collect();
    let tree = MemberTree::new(&npks).context("building the member set")?;

    let n = u8::try_from(nsks.len()).context("too many members")?;
    let params = CreateMultisig {
        member_root: tree.root(),
        m,
        n,
        multisig_id: parse32("multisig-id", multisig_id)?,
        membership_program_id: LOCAL_VERIFIER,
    };
    let (config, config_hash) =
        logic::create_multisig(&params).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut st = LocalState::load(&cli.state)?;
    st.set_balance(config.multisig_id, treasury);
    st.config = Some(config.clone());
    st.config_hash = Some(config_hash);
    st.member_npks = npks.clone();
    st.save(&cli.state)?;

    println!("[local] created {m}-of-{n} multisig");
    println!("  multisig_id : {}", hex::encode(config.multisig_id));
    println!("  member_root : {}", hex::encode(config.member_root));
    println!("  config_hash : {}", hex::encode(config_hash));
    println!("  treasury    : {treasury}");
    println!(
        "\nThe member set is committed as a root only; the members themselves are not published."
    );
    Ok(())
}

fn propose(cli: &Cli, proposal_id: &str, recipient: &str, amount: u128) -> Result<()> {
    let mut st = LocalState::load(&cli.state)?;
    let (config, config_hash) = loaded_config(&st)?;
    let pid = parse32("proposal-id", proposal_id)?;
    let (proposal, seed) = logic::create_proposal(
        &config,
        &config_hash,
        pid,
        ProposedAction::TreasuryTransfer {
            recipient: parse32("recipient", recipient)?,
            amount,
        },
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    if st.proposal_mut(&pid).is_some() {
        // Rendered from the catalogue entry itself (`"1010 AccountAlreadyInitialized"`) rather
        // than hardcoded, so the message cannot drift from `docs/error-codes.md`.
        bail!(
            "{}: a proposal with that id already exists",
            MultisigError::AccountAlreadyInitialized
        );
    }
    st.proposals.push(proposal);
    st.save(&cli.state)?;

    println!("[local] proposal created");
    println!("  proposal_id   : {}", hex::encode(pid));
    println!("  proposal_seed : {}", hex::encode(seed));
    println!("  action        : transfer {amount} to {recipient}");
    println!("  approvals     : 0 of {}", config.m);
    Ok(())
}

fn approve(cli: &Cli, proposal_id: &str, member: &str) -> Result<()> {
    let mut st = LocalState::load(&cli.state)?;
    let (config, config_hash) = loaded_config(&st)?;
    let pid = parse32("proposal-id", proposal_id)?;
    let nsk = parse32("member key", member)?;

    let proposal = st
        .proposal_mut(&pid)
        .context("1006 UnknownProposal: no proposal with that id")?;
    let recorded = proposal.nullifiers.clone();
    let approvals_on_chain = proposal.approvals();

    // Everything the approving member may know: public config plus a count.
    let view = MultisigView {
        multisig_id: config.multisig_id,
        member_root: config.member_root,
        m: config.m,
        n: config.n,
        approvals_on_chain,
    };

    // The member's own path. A real client holds this from multisig creation; here it is recovered
    // from the member set, which this demo happens to know.
    let secrets = member_secrets(&st, &nsk)?;

    let prepared =
        prepare_approval(&view, &secrets, pid, &recorded).map_err(|e| anyhow::anyhow!("{e}"))?;

    let proposal = st
        .proposal_mut(&pid)
        .context("1006 UnknownProposal: no proposal with that id")?;
    logic::approve(
        &config,
        &config_hash,
        proposal,
        &prepared.claim,
        &LOCAL_VERIFIER,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    let now = proposal.approvals();

    // Persist locally first, so a crash after this point cannot lose the approval (P-R2).
    ApprovalStore::new(&cli.store_dir)
        .record(&ApprovalRecord {
            multisig_id: config.multisig_id,
            proposal_id: pid,
            nullifier: prepared.claim.claimed_nullifier,
            status: ApprovalStatus::Confirmed,
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    st.save(&cli.state)?;

    println!("[local] approval recorded");
    println!(
        "  nullifier : {}",
        hex::encode(prepared.claim.claimed_nullifier)
    );
    println!("  approvals : {now} of {}", config.m);
    println!("\nThe nullifier identifies nobody: it is a hash of the member's secret with this");
    println!("multisig and proposal. No member id is written anywhere on chain.");
    Ok(())
}

fn execute(cli: &Cli, proposal_id: &str) -> Result<()> {
    let mut st = LocalState::load(&cli.state)?;
    let (config, config_hash) = loaded_config(&st)?;
    let pid = parse32("proposal-id", proposal_id)?;

    let proposal = st
        .proposal_mut(&pid)
        .context("1006 UnknownProposal: no proposal with that id")?;
    let action =
        logic::execute(&config, &config_hash, proposal).map_err(|e| anyhow::anyhow!("{e}"))?;
    let ProposedAction::TreasuryTransfer { recipient, amount } = action;

    let treasury_before = st.balance(&config.multisig_id);
    if treasury_before < amount {
        bail!(
            "1012 InvalidProposalAction: treasury holds {treasury_before}, proposal needs {amount}"
        );
    }
    st.set_balance(config.multisig_id, treasury_before - amount);
    let to = st.balance(&recipient);
    st.set_balance(recipient, to + amount);
    st.save(&cli.state)?;

    println!("[local] proposal executed");
    println!("  transferred : {amount}");
    println!("  recipient   : {}", hex::encode(recipient));
    println!(
        "  treasury    : {} -> {}",
        treasury_before,
        treasury_before - amount
    );
    Ok(())
}

fn status(cli: &Cli, proposal_id: Option<&str>) -> Result<()> {
    let st = LocalState::load(&cli.state)?;
    let (config, _) = loaded_config(&st)?;
    println!("[local] multisig {}-of-{}", config.m, config.n);
    println!("  multisig_id : {}", hex::encode(config.multisig_id));

    let wanted = proposal_id.map(|s| parse32("proposal-id", s)).transpose()?;
    for p in &st.proposals {
        if let Some(w) = wanted {
            if p.proposal_id != w {
                continue;
            }
        }
        println!("\n  proposal {}", hex::encode(p.proposal_id));
        println!("    approvals : {} of {}", p.approvals(), config.m);
        println!("    executed  : {}", p.executed);
        // Deliberately: a count and a set of nullifiers. Never a list of who approved.
        println!("    nullifiers:");
        for nf in &p.nullifiers {
            println!("      {}", hex::encode(nf));
        }
    }
    Ok(())
}

fn loaded_config(st: &LocalState) -> Result<(pmsig_multisig_core::MultisigConfig, Digest32)> {
    let config = st
        .config
        .clone()
        .context("no multisig here yet — run `pmsig create` first")?;
    let hash = st
        .config_hash
        .context("local state is missing its config hash")?;
    Ok((config, hash))
}

/// Recovers a member's authentication path from local state.
///
/// A real client holds its own path from the moment the multisig was created and never needs the
/// other members' keys. Local mode keeps the member npks beside the state so one machine can act as
/// several members in a demo — see `docs/integration.md`. Nothing on chain carries this list.
fn member_secrets(st: &LocalState, nsk: &Digest32) -> Result<MemberSecrets> {
    let npk = npk_of(nsk).to_byte_array();
    let index = st
        .member_npks
        .iter()
        .position(|m| *m == npk)
        .context("2004 NotAMember: that key is not a member of this multisig")?;
    let tree = MemberTree::new(&st.member_npks).context("rebuilding the member set")?;
    let path = tree
        .path(index)
        .context("member index has no authentication path")?;
    Ok(MemberSecrets {
        nsk: *nsk,
        vpk: demo_vpk(),
        identifier: 0,
        path,
    })
}
