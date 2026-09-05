# VERSIONS — pinned toolchain and network (draft, Phase −1)

**Status:** DRAFT (Phase −1). Re-pin and re-verify at Phase G (deploy) and Phase I (day-of PR).
**Captured:** 2026-09-04 (UTC). All values below were **observed**, not assumed — evidence paths given.

---

## Upstream dependencies

| Component | Value | How verified |
|-----------|-------|--------------|
| LEZ repo | `https://github.com/logos-blockchain/logos-execution-zone` | Prize `Resources` section |
| LEZ latest tag | **v0.2.4** | `gh api repos/logos-blockchain/logos-execution-zone/tags` → `v0.2.4, v0.2.3, v0.2.2, v0.2.2-rc1, v0.2.1, v0.2.0` |
| LEZ Rust channel | **1.94.0** (`profile = default`) | `rust-toolchain.toml` @ tag `v0.2.4` |
| `risc0-zkvm` (what **LEZ v0.2.4** pins) | **3.0.5** | `Cargo.toml` @ `v0.2.4` line 131 |
| `risc0-zkvm` (what **this repo** uses) | **3.0.6** (`default-features = false`, `features = ["std"]`) | our `Cargo.toml` line 36; `r0vm` 3.0.6 locally and in CI |
| `risc0-build` | **3.0.5** | `Cargo.toml` @ `v0.2.4` line 132 |

**On the 3.0.5 / 3.0.6 difference.** LEZ v0.2.4 pins risc0 3.0.5; this repository builds and proves
with 3.0.6, which is what `rzup` installs and what CI verifies (`r0vm --version` is asserted before
the e2e runs). Both numbers appear in this repository — `risc0-binfmt = "3.0.5"` in
`crates/guest-tools` and in the SDK's dev-dependencies — so the difference is stated here rather than
left for a reviewer to notice and wonder about.

It is not silently assumed to be safe. What makes it checkable is that ImageIDs are compared against
LEZ's own artefacts: `crates/membership-core/tests/lez_compat.rs` asserts byte-compatibility with
LEZ's published vectors, and `crates/sdk/tests/image_ids_match_binaries.rs` asserts that the ImageIDs
recorded in `artifacts/IMAGE_IDS.md` are the ones the committed binaries actually have. A divergence
in ImageID computation between the two versions would fail those, not pass quietly.
| SPEL repo | `https://github.com/logos-co/spel` | Prize `Success Criteria → Usability` (IDL requirement) |
| SPEL latest tag | **v0.6.0** — *not used, pins LEZ v0.2.0* | `gh api repos/logos-co/spel/tags` → `v0.6.0, v0.6.0-rc.2, v0.6.0-rc.1, v0.5.0, v0.5.0-rc.1` |
| **SPEL pin used** | **`main` @ `5126b7ed8a9b`** (2026-09-04T06:48:20Z) — pins LEZ `v0.2.4` | `gh api repos/logos-co/spel/contents/spel-framework/Cargo.toml?ref=main` |
| SPEL license | MIT **and** Apache-2.0 files present (`LICENSE-MIT`, `LICENSE-APACHE-v2`) | local clone `.refs/spel` @ `v0.6.0` |

**LEZ pin decision: `v0.2.4` — settled empirically, not by preference.**

A LEZ `ProgramId` is the risc0 **ImageID** of the program ELF, and LEZ commits prebuilt program binaries under
`artifacts/` at every tag. Computing those ImageIDs host-side (`risc0-zkvm` 3.0.5 `compute_image_id`) and comparing
them against the live testnet's `getProgramIds` fingerprints the deployed version exactly:

| Program | Live testnet `getProgramIds` | v0.2.0 artifact ImageID | v0.2.4 artifact ImageID |
|---------|------------------------------|-------------------------|-------------------------|
| `token` | `ccc4713e2b5ecdff37b0c67c295369effc04b7e8994eb11c3f410bb226b82e9b` | `c5d50f88…53c69a7c` ✗ | `ccc4713e…26b82e9b` **✓ exact** |
| `privacy_preserving_circuit` | `383e884f67e016e9e046294a6f8ed2dab5b516bbcf452c18f32145f2d400206f` | `ab86d257…6ad6a249` ✗ | `383e884f…d400206f` **✓ exact** |

`artifacts/*.bin` blobs are byte-identical across **v0.2.2 / v0.2.3 / v0.2.4** and differ at v0.2.0, so the testnet
runs that family and **v0.2.4** is its latest tag. Evidence: `artifacts/phase-N1-testnet-version-fingerprint.txt`.

**This matters — it is not a cosmetic pin.** `AccountId::for_regular_private_account` is **not compatible** between the
two: v0.2.0 hashes `prefix ‖ npk ‖ identifier` (80 bytes), while v0.2.4 hashes `prefix ‖ npk ‖ vpk ‖ identifier` with
`ViewingPublicKey::LEN = 1184`. Building against v0.2.0 would derive **wrong shielded account addresses** on this
testnet. (`Account` layout and the `Commitment` preimage are unchanged between the two.)

**Consequence for SPEL — a trap worth naming:** the released SPEL tag **v0.6.0 pins LEZ `v0.2.0`**
(`spel-framework/Cargo.toml`: `tag = "v0.2.0", package = "lee_core"`), so the published SPEL release derives private
account ids the live testnet will not agree with. SPEL **`main`** already pins `tag = "v0.2.4"`
(head `5126b7ed8a9b`, 2026-09-04T06:48:20Z). Therefore this build pins **SPEL `main` at a specific commit**, not the
v0.6.0 tag — and that is an unreleased dependency, to be disclosed in `docs/limitations.md` (W7).

## LEZ testnet (live, probed 2026-09-04T11:07:40Z)

| Item | Value | Evidence |
|------|-------|----------|
| RPC endpoint | `https://testnet.lez.logos.co` | JSON-RPC 2.0, HTTP 200 |
| Explorer | `https://explorer.testnet.lez.logos.co` | HTTP 200 |
| Endpoint provenance | Both URLs are cited in the **merged upstream** solution `solutions/LP-0005.md` in `logos-co/lambda-prize` — not invented by this agent | `rg 'testnet.lez.logos.co' ../lambda-prize/solutions/LP-0005.md` |
| `checkHealth` | `{"jsonrpc":"2.0","id":1,"result":null}` (healthy — no error object) | `artifacts/phase-N1-testnet-probe.txt` |
| `getLastBlockId` | **37251** at 11:07:40Z (37250 at 11:05Z → chain is advancing) | same |
| `getProgramIds` | `amm`, `authenticated_transfer`, `pinata`, **`privacy_preserving_circuit`**, `token` | same |

**Note on `privacy_preserving_circuit`:** the live testnet exposes a `privacy_preserving_circuit` program id. This is the on-chain surface the plan's **PPE / chained `env::verify`** approve path (plan §2, H9) must integrate with. Exact interface to be read from LEZ source in Phase A — **not** assumed here.

**Verified RPC method names** (from LEZ `sequencer/service/rpc/src/lib.rs`, local checkout `v0.2.0-rc4-147-gcf9177a0`; re-confirm against v0.2.4 in Phase A):
`sendTransaction`, `checkHealth`, `getBlock`, `getBlockRange`, `getLastBlockId`, `getAccountBalance`, `getTransaction`, `getAccountsNonces`, `getProofForCommitment`, `getAccount`, `getProgramIds`

## Builder machine (this machine)

| Tool | Version | Note |
|------|---------|------|
| OS | macOS (Darwin 25.0.0), aarch64 | |
| `rustc` (default) | 1.95.0 (59807616e 2026-04-14) | |
| `cargo` | 1.95.0 (f2d3ce0bd 2026-03-21) | |
| rustup toolchains | `stable`, `nightly`, `1.78.0`, `1.81.0`, **`1.94.0`**, `solana` | 1.94.0 (LEZ channel) already present |
| `rustup` | 1.28.2 | |
| `gh` | 2.83.0, authed as `pramadanif` (scopes: gist, read:org, repo, workflow) | |
| `docker` | 29.1.3 | available for standalone-sequencer e2e if needed |
| `jq` | 1.7.1-apple | |
| `rg` | 14.1.1 | |
| `rzup` / `r0vm` / `cargo-risczero` | see `docs/phase-N1-status.md` (installed in Phase −1) | install cmd: `curl -L https://risczero.com/install \| bash` then `rzup install` (per LEZ v0.2.4 README lines 112–120) |

## Local reference checkouts (gitignored, `.refs/`)

| Path | Rev | Use |
|------|-----|-----|
| `.refs/spel` | tag `v0.6.0` | SPEL macros/IDL/CLI API — read, do not copy wholesale |
| `../lez-event-system/logos-execution-zone` | `cf9177a0` = `v0.2.0-rc4-147-gcf9177a0` (2026-05-21) | pre-existing local LEZ checkout (own prior work, LP-0012). **Older than v0.2.4** — use only for orientation; pin v0.2.4 for the build |

## Open unknowns (must resolve before claiming, never guess)

| ID | Unknown | Resolve in |
|----|---------|-----------|
| ~~U-1~~ | ~~Exact LEZ version running on testnet~~ | **RESOLVED in Phase −1** by ImageID fingerprint → v0.2.2/v0.2.3/**v0.2.4** family. See pin decision above |
| U-2 | `privacy_preserving_circuit` **input encoding** end-to-end (`PrivacyPreservingCircuitInput` construction, witness/dummy inputs) — the `env::verify` chaining mechanism itself is understood (`docs/phase-N1-status.md` §5) | Phase A/B |
| ~~U-6~~ | ~~Whether SPEL's CLI submits the private path or public only~~ | **RESOLVED in Phase E — it does.** `spel-cli/src/tx.rs:434-502`: the CLI detects `Private/`-prefixed account ids, builds a `ProgramWithDependencies` (the program plus dependency programs supplied via `--bin-<NAME>`), assembles an `AccountIdentity::{PrivateOwned,Public}` list, and calls `send_privacy_preserving_tx`. Public transactions take a separate branch (`LeeTransaction::Public`). The dependency mechanism is what resolves our multisig's `ChainedCall` to the membership program |
| ~~U-3~~ | ~~Shielded-account commitment byte format + `nonce` / `program_owner` rules~~ | **RESOLVED in Phase −1** — formulas read from LEZ v0.2.4 source with pinned known-answer vectors; see `docs/phase-N1-status.md` §5 |
| U-4 | Funded testnet key(s) for M members + treasury | **HUMAN GATE** before Phase G |
| U-5 | Basecamp `.lgx` packaging toolchain + `module.json` schema | Phase F |

---

*Draft written in Phase −1. Every row above is either an observed command output or a citation to a file at a named revision.*
