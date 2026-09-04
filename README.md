# lp-0002-private-multisig

A **private M-of-N multisig** for the Logos Execution Zone (LEZ): members hold shielded accounts,
approvals leave no public trace of who voted, and on-chain state records only that a threshold was
met — never which members approved.

Built for [λPrize LP-0002](docs/plan/LP-0002.md). Licensed **MIT OR Apache-2.0**.

Repository: <https://github.com/pramadanif/lp0002>

> **Status: Phase D of 10 complete.** The membership guest proves and verifies for real
> (`RISC0_DEV_MODE=0`, 53.26 s on a laptop) and the on-chain SPEL program enforces the full
> lifecycle with a published IDL. The end-to-end privacy-preserving composition has **not** been
> demonstrated against a running sequencer yet, the demo is not written, and nothing here has touched
> testnet. See [Build status](#build-status) for exactly what exists.

---

## Why a private multisig needs a different architecture

The existing [lez-multisig](https://github.com/jimmy-claw/lez-multisig) PoC is a *public* multisig,
and it cannot be adapted to shielded accounts. Its member accounts must be fresh zero-nonce
keypairs claimed by the multisig program. Shielded LEZ accounts cannot satisfy either half of that:

- they are owned by the privacy protocol, not by the multisig program, and
- their nonce is **not** a counter you can hold at zero. For a private account LEZ derives it as
  `nonce_init = SHA256(account_id ‖ [0;32])[0..16]` and then advances it as
  `nonce' = SHA256(nsk ‖ nonce ‖ [0;16])[0..16]` — a fresh value from the member's nullifier
  secret key on **every** use.
  (`lee/state_machine/core/src/account.rs` @ `logos-execution-zone` v0.2.4.)

So membership cannot be proven by "the program owns your account". It has to be proven in zero
knowledge: *I control an account whose commitment is in the member set*, without saying which one.

## Approach

Locked in [`docs/adr/ADR-001-architecture.md`](docs/adr/ADR-001-architecture.md) — summarised here:

| Decision | Choice |
|----------|--------|
| Approve path | **Privacy-preserving execution** (chained `env::verify`), never public re-execution |
| Membership proof | LEZ-native guest emitting a `ProgramOutput`, verified as a chained call |
| Anchoring | PDA seeded by `config_hash`, binding the member set and the threshold `M` together |
| Binding | Proof is bound in-circuit to a **live** shielded account commitment, not merely derived off-chain |
| Double-vote prevention | Nullifier set on-chain, one nullifier per (member, proposal) |
| Peer privacy | Co-members learn the approval **count**, never who approved |
| Reference action | Treasury transfer, default 2-of-3 |

Anchoring the member root **and** `M` in the PDA seed is what stops the obvious attack: a prover who
invents their own member set, or quietly lowers the threshold, derives a different PDA and simply
does not find the multisig there.

The multisig's config account lives at `for_public_pda(program_id, PdaSeed(config_hash))`, where

```text
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] ‖ membership_program_id[32] )
DS_CONFIG   = "/LP0002/v1/ConfigHash/" ++ [0u8; 10]      // 22 + 10 = 32 bytes
```

This line is the single definition of `config_hash` in the project; [ADR-001
§3](docs/adr/ADR-001-architecture.md) and the solution write-up quote it verbatim, and preflight check
PF-13 fails the build if the three ever drift apart.

## Build status

Each phase has a status document recording the exact commands run, their exit codes and log paths.

| Phase | What it delivers | Status |
|-------|------------------|--------|
| −1 | Competitor + environment preflight, version pins | ✅ [`docs/phase-N1-status.md`](docs/phase-N1-status.md) |
| 0 | Repo skeleton, dual licence, CI, preflight harness | ✅ [`docs/phase-0-status.md`](docs/phase-0-status.md) |
| A | ADR, account model, security model, error codes | ✅ [`docs/phase-A-status.md`](docs/phase-A-status.md) |
| B | Membership + nullifier guest, one real `RISC0_DEV_MODE=0` proof | ✅ [`docs/phase-B-status.md`](docs/phase-B-status.md) |
| C | SPEL program: create / propose / approve / execute, IDL | ✅ [`docs/phase-C-status.md`](docs/phase-C-status.md) |
| D | SDK, CLI, restart-resume, peer privacy | ✅ [`docs/phase-D-status.md`](docs/phase-D-status.md) |
| E | `demo.sh` against a standalone sequencer, CI e2e | ☐ |
| F | Basecamp app, downloadable `.lgx` | ☐ |
| G | Testnet deployment, CU costs, public verification | ☐ |
| H | Documentation, preflight green, narrated video | ☐ |

Progress and blockers: [`docs/TRACKING.md`](docs/TRACKING.md).
Criteria mirror: [`PRIZE_CHECKLIST.md`](PRIZE_CHECKLIST.md).

## Pinned versions

Established by measurement, not assumption — see [`docs/VERSIONS.md`](docs/VERSIONS.md).

| Component | Pin |
|-----------|-----|
| LEZ | **v0.2.4** |
| SPEL | **`main` @ `5126b7ed8a9b`** (the v0.6.0 release pins LEZ v0.2.0 and derives different private account ids) |
| Rust (host) | 1.94.0 |
| risc0 | 3.0.5 (`r0vm`, `cargo-risczero`); guest toolchain 1.97.0 |

The LEZ pin was settled by fingerprinting the live testnet: a LEZ `ProgramId` is the risc0 ImageID of
the program ELF, and the ImageIDs of LEZ's committed `artifacts/` binaries match the testnet's
`getProgramIds` output **exactly at v0.2.4** and not at v0.2.0.

## Building

```bash
cargo test --workspace          # unit tests
cargo clippy --workspace --all-targets -- -D warnings
./scripts/preflight-submission.sh   # submission gate — exits 1 until every check passes
```

`preflight-submission.sh` exiting 1 is the correct result today: checks whose evidence a later phase
produces report `PENDING`, and pending is never treated as a pass.

There is no `demo.sh` yet. When it lands (Phase E) it will drive a **real standalone LEZ sequencer**
with `RISC0_DEV_MODE=0`, and it will fail — not skip — if a required tool is missing.

## Repository layout

```
crates/core      shared types and hash formulas (host + guest)
crates/multisig-core  on-chain state, error codes, lifecycle rules
crates/membership-core witness types and the membership check
crates/sdk       client-side proving, and the member-facing API
crates/store     local persistence for partial approval sets
crates/cli       `pmsig` command-line client
scripts/         preflight, dev-mode clobber check; e2e + verification land later
docs/            phase status docs, ADRs, security model, error codes, version pins
artifacts/       evidence: ImageIDs, probe output, binary hashes
```

## Licence

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
