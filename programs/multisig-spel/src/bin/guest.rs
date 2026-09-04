//! The deployable guest: the multisig program as a risc0 binary.
//!
//! `#[lez_program]` emits the dispatcher as a top-level `pub fn main()` in the library, so the guest
//! binary is just an entry point pointing at it. Kept as a separate binary from `idl`, because the
//! IDL generator is a host tool and the guest must cross-compile to `riscv32im-risc0-zkvm-elf`.

#![no_main]

risc0_zkvm::guest::entry!(main);

use pmsig_multisig_program::main;
