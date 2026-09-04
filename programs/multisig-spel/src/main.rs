//! Prints the program's IDL as JSON.
//!
//! Criterion **P-U3** asks for an IDL for the LEZ program, generated with the SPEL framework.
//! `scripts/generate-idl.sh` runs this and writes `artifacts/multisig-idl.json`.
fn main() {
    println!("{}", pmsig_multisig_program::PROGRAM_IDL_JSON);
}
