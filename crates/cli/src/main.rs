//! `pmsig` — CLI for shielded members of a private M-of-N multisig.
//!
//! Phase 0 scope: the binary exists and reports that the lifecycle is not wired yet. It
//! deliberately exits **non-zero** for unimplemented subcommands so that no script can mistake a
//! stub for a working step (plan gate **H2**: nothing on a submission path may exit 0 without
//! having done the work). The real lifecycle lands in Phase D.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("-V") => {
            println!("pmsig {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help") | Some("-h") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!(
                "pmsig: subcommand `{other}` is not implemented yet (Phase D wires the lifecycle)."
            );
            eprintln!("Run `pmsig --help` for what exists today.");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "pmsig {} — private M-of-N multisig for LEZ

USAGE:
    pmsig <COMMAND>

COMMANDS (planned — Phase D):
    create      Create a multisig with a shielded member set
    propose     Submit a proposal against an existing multisig
    approve     Approve a proposal without revealing which member you are
    execute     Execute a proposal once M approvals are on-chain
    status      Show threshold progress for a proposal

Nothing but --help/--version is wired yet; every other subcommand exits non-zero on purpose.",
        env!("CARGO_PKG_VERSION")
    );
}
