# Phase −1 status — Competitor + env preflight

**Date:** 2026-09-04
**Repo:** `/Users/muhammadbaguspramadani/Documents/myproject/lp-0002-private-multisig`
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase −1
**Result:** *(SC table at bottom — updated as commands complete)*

---

## 1. Abort check (plan §0.4.7) — **NOT triggered**

Command run (2026-09-04, ~11:05Z):

```
gh pr view 125 --repo logos-co/lambda-prize --json reviewDecision -q .reviewDecision
```

Output: **empty string** (no review decision recorded) → **not `APPROVED`**.

Merged-PR check:

```
gh pr list --repo logos-co/lambda-prize --state merged --search "LP-0002 in:title" --json number,title,mergedAt --limit 20
```

Output: **empty** → no LP-0002 solution PR merged upstream.

**Abort rule, restated for every later phase:** stop all feature work and switch to documentation-only if either
(a) `gh pr view 125 … reviewDecision` == `APPROVED`, or (b) any LP-0002 solution PR is merged upstream.
This check runs at the start of every phase and is logged in `docs/TRACKING.md` → "Abort watch".

## 2. Competitor snapshot (2026-09-04)

| PR | Author | State | reviewDecision | Last update |
|----|--------|-------|----------------|-------------|
| [#123](https://github.com/logos-co/lambda-prize/pull/123) | FidelCoder | OPEN | *(none)* | 2026-09-02T06:14:23Z |
| [#125](https://github.com/logos-co/lambda-prize/pull/125) | edenbd1 | OPEN | *(none)* | 2026-08-29T19:06:44Z |
| [#133](https://github.com/logos-co/lambda-prize/pull/133) | jeefxM | OPEN | *(none)* | 2026-08-28T12:40:04Z |

All three still open, none engineer-approved. Prize still **[OPEN]** upstream — verified by fetching
`prizes/LP-0002.md` from `logos-co/lambda-prize` via the GitHub API (heading reads `# LP-0002: Private M-of-N Multisig [OPEN]`),
not from the local checkout alone. Local `lambda-prize` checkout is at `a0fc371` and **0 commits behind** `origin/master`.

## 3. Eligibility note (#105) — operator decision: proceed

This is not part of the plan's SC-N1 list but is an abort-class fact found during the competitor snapshot,
and the prompt (§F) lists it as a human-escalation item.

**Public facts from `logos-co/lambda-prize`:**

- Issue **#105** "Lambda Prize — payment", opened **2026-07-13T12:10:01Z by `pramadanif`** (the account this machine's
  `gh` is authenticated as), body carries Typeform response ID `gH1-LP-0012-L0G05` and an ETH payout address.
- `solutions/LP-0012.md` in that repo records **"Submitted by: bristinWild"** (repo `github.com/bristinWild/logos-execution-zone`).
- Collaborator **`mart1n-xyz`** replied: *"LP12 has already paid out and you didn't solve it. This is a spoofing attempt
  and therefore, we'll ban you from any further contributions in this repo."* Issue closed 2026-07-17T10:03:35Z.
- Issue **#140** `[ACCESS-TEST-DELETE-ME]` opened by the same account 2026-09-04T08:35:35Z, now closed.

**Operator's account of it (recorded verbatim as given, 2026-09-04):** the LP-0012 payment claim was **filed in error** on
their part; they hold no claim to `bristinWild`'s work.

**Consequence for this build — operator decision, 2026-09-04 (revised):**

The operator reviewed the above and directed the build to proceed through **all** phases, Phase I included.
Whether to submit, and the standing of #105 with the Logos program team, is the operator's call, not the agent's.
This section is retained as an honest record of what the public repository says — it is **not** a gate.

Unchanged and not negotiable: the solution PR is opened from the operator's own account (`pramadanif`), under their
own authorship. No secondary or substitute account, and no attributing the submission to anyone other than its actual
author (prompt §B/§F, "no alt-account evasion"). Nothing about this decision requires either, so nothing about the
build changes.

## 4. Environment snapshot

Full detail in `docs/VERSIONS.md`. Headlines, all observed:

- **LEZ** `logos-blockchain/logos-execution-zone` latest tag **v0.2.4** → Rust **1.94.0**, `risc0-zkvm`/`risc0-build` **3.0.5**
- **SPEL** `logos-co/spel` latest tag **v0.6.0** (dual MIT + Apache-2.0 licence files)
- **Testnet RPC** `https://testnet.lez.logos.co` — `checkHealth` → `result: null` (healthy), `getLastBlockId` → **37251**,
  chain advancing (37250 → 37251 across two probes ~2 min apart)
- **Explorer** `https://explorer.testnet.lez.logos.co` — HTTP **200**
- Both endpoint URLs are taken from the **merged upstream** solution `solutions/LP-0005.md`, not invented
- Live `getProgramIds` → `amm`, `authenticated_transfer`, `pinata`, **`privacy_preserving_circuit`**, `token`
- Evidence file: `artifacts/phase-N1-testnet-probe.txt`

Local reference checkouts (gitignored under `.refs/`): `spel` @ `v0.6.0`, `lez` @ `v0.2.4` (22 MB, shallow).

## 5. Architecture facts confirmed against LEZ v0.2.4 source (de-risks Phase A/B)

Read from `.refs/lez` — cited, not assumed. These resolve `VERSIONS.md` unknowns **U-2** and **U-3**.

| Fact | Source |
|------|--------|
| `Account { program_owner: ProgramId, balance: u128, data: Data, nonce: Nonce(u128) }` | `lee/state_machine/core/src/account.rs:98` |
| Public nonce: `+1` per use. **Private (shielded) nonce**: init `= SHA256(account_id ‖ [0;32])[0..16]` as u128 LE; increment `= SHA256(nsk ‖ nonce_le ‖ [0;16])[0..16]` as u128 LE — i.e. it advances on every use and is **nsk-derived**, which is precisely the constraint the prize overview says public multisig cannot satisfy | `account.rs:20–47` |
| `Commitment::new = SHA256("/LEE/v0.3/Commitment/"‖[0;11] ‖ account_id(32) ‖ program_owner(8×u32 LE) ‖ balance(u128 LE) ‖ nonce(u128 LE) ‖ SHA256(data))` | `lee/state_machine/core/src/commitment.rs` |
| `AccountId::for_regular_private_account = SHA256("/LEE/v0.3/AccountId/Private/"‖[0;4] ‖ npk(32) ‖ vpk ‖ identifier(u128 LE))` | `lee/state_machine/core/src/nullifier.rs` |
| `npk = SHA256("LEE/keys" ‖ nsk(32) ‖ [7] ‖ [0;23])` | `nullifier.rs` |
| `AccountId::for_private_pda = SHA256("/LEE/v0.3/AccountId/PrivatePDA/"‖[0;1] ‖ program_id(32) ‖ seed(32) ‖ npk(32) ‖ vpk ‖ identifier(u128 LE))` — **seed is exactly 32 bytes**, so `config_hash` drops straight in as a `PdaSeed` | `lee/state_machine/core/src/program/mod.rs:154–179` |
| `AccountId::for_public_pda = SHA256("/LEE/v0.2/AccountId/PDA/"‖[0;8] ‖ program_id(32) ‖ seed(32))` | `program/mod.rs:127–144` |
| Commitment-set membership: leaf `= SHA256(commitment_bytes)`, then pairwise `SHA256(left‖right)` selected by index bit — `compute_digest_for_path(commitment, (index, path))` | `commitment.rs` |
| **PPE = chained `env::verify`**: `lee/privacy_preserving_circuit` reads `PrivacyPreservingCircuitInput { program_outputs, account_identities, program_id, dummy_inputs }`, walks the chained-call queue and calls `env::verify(chained_call.program_id, program_output_words)` per output, asserting `self_program_id` and `caller_program_id` match | `lee/privacy_preserving_circuit/src/{main,execution_state}.rs` |
| `ChainedCall { program_id, pre_states, instruction_data, pda_seeds }`; `ProgramOutput { self_program_id, caller_program_id, instruction_data, pre_states, post_states, chained_calls, block_validity_window, timestamp_validity_window }` | `program/mod.rs:202, 430` |
| Standalone sequencer: `RUST_LOG=info cargo run --features standalone -p sequencer_service lez/sequencer/service/configs/debug` | LEZ v0.2.4 `README.md` §"Standalone mode" (lines 218–221) |
| Reproducible guest build: `cargo risczero build` with `RISC0_DOCKER_CONTAINER_TAG=r0.1.91.1` | LEZ v0.2.4 `Justfile` |

## 5b. Version pin settled empirically — LEZ **v0.2.4**, SPEL **`main`** (not the v0.6.0 tag)

The plan (Appendix B) warns that #123 pinned LEZ v0.2.2 and "may lag testnet". Rather than guess, the deployed
version was **fingerprinted**: a LEZ `ProgramId` is the risc0 **ImageID** of the program ELF, and LEZ commits
prebuilt binaries under `artifacts/` at every tag. Computing those ImageIDs host-side and diffing against the live
`getProgramIds` identifies the deployment exactly.

| Program | Live testnet | v0.2.0 artifact | v0.2.4 artifact |
|---------|--------------|-----------------|-----------------|
| `token` | `ccc4713e…26b82e9b` | `c5d50f88…53c69a7c` ✗ | `ccc4713e…26b82e9b` **✓ exact** |
| `privacy_preserving_circuit` | `383e884f…d400206f` | `ab86d257…6ad6a249` ✗ | `383e884f…d400206f` **✓ exact** |

Artifact blobs are byte-identical across v0.2.2/v0.2.3/v0.2.4 and differ at v0.2.0 → the testnet runs that family;
**v0.2.4** is its latest tag. Evidence: `artifacts/phase-N1-testnet-version-fingerprint.txt`.
Tool: `risc0-zkvm` 3.0.5 `compute_image_id`, run over binaries fetched from `raw.githubusercontent.com` at each tag.

**Why this is load-bearing, not cosmetic:** `AccountId::for_regular_private_account` is **incompatible** across the
two tags — v0.2.0 hashes `prefix ‖ npk ‖ identifier` (80 bytes); v0.2.4 hashes `prefix ‖ npk ‖ vpk ‖ identifier`
with `ViewingPublicKey::LEN = 1184` (a post-quantum-sized viewing key). Building against v0.2.0 would produce
**wrong shielded account addresses** on this testnet. `Account` layout and the `Commitment` preimage are unchanged.

**SPEL trap found:** released SPEL **v0.6.0 pins LEZ `v0.2.0`** (`spel-framework/Cargo.toml`:
`tag = "v0.2.0", package = "lee_core"`), i.e. the published SPEL release derives private account ids this testnet
will not agree with. SPEL **`main`** already pins `tag = "v0.2.4"` (head `5126b7ed8a9b`, 2026-09-04T06:48:20Z).
→ This build pins **SPEL `main` at a fixed commit**. That is an **unreleased** dependency and must be disclosed
honestly in `docs/limitations.md` (W7) rather than presented as "SPEL v0.6.0".

**Known-answer vectors available** for `Commitment`, `Nullifier`, `npk`, `for_regular_private_account` and
`for_private_pda` in LEZ's own `#[test]` blocks (`nullifier.rs`, `commitment.rs`, `program/tests.rs`). These will be used
as regression vectors for **SC-B.7** ("byte-compatible with LEZ; regression vs known vector if available") — the vectors
exist, so SC-B.7 must cite them rather than claim none were available.

## 6. Success criteria

| SC | Requirement | State | Evidence |
|----|-------------|-------|----------|
| **SC-N1.1** | Abort condition written; stop if #125 APPROVED | ✅ green | §1 above; #125 reviewDecision empty → continue. Rule mirrored into `docs/TRACKING.md` abort-watch table |
| **SC-N1.2** | `docs/VERSIONS.md` draft has LEZ / SPEL / Risc0 / testnet RPC / explorer URLs | ✅ green | `docs/VERSIONS.md` — all five present with per-row verification method; probe output at `artifacts/phase-N1-testnet-probe.txt` |
| **SC-N1.3** | Toolchain bootstrap command succeeds (recorded in status) | ✅ green | §7 below — `rzup` install exit 0; `r0vm` 3.0.5 / `cargo-risczero` 3.0.5 / risc0 rust 1.97.0 all report versions |

## 7. Toolchain bootstrap (SC-N1.3) — **green**

Install commands are the ones LEZ v0.2.4's own README prescribes (lines 112–120), not improvised.

| # | Command | Exit | Log |
|---|---------|------|-----|
| 1 | `curl -L https://risczero.com/install \| bash` | **0** (`INSTALLER_EXIT=0`) | `logs/phase-N1-rzup-install.log` |
| 2 | `rzup install rust` | **0** (`EXIT_rust=0`) | `logs/phase-N1-rzup-components.log` |
| 3 | `rzup install r0vm 3.0.5` | **0** (`EXIT_r0vm=0`) | same |
| 4 | `rzup install cargo-risczero 3.0.5` | **0** (`EXIT_cargorisczero=0`) | same |

`rzup show` after install:

```
cargo-risczero   * 3.0.5
r0vm             * 3.0.5
rust             * 1.97.0
rzup home: /Users/muhammadbaguspramadani/.risc0
```

Version probes actually executed (not assumed):

```
$ rzup --version        -> rzup 0.5.0
$ r0vm --version        -> risc0-r0vm 3.0.5
$ cargo risczero --version -> cargo-risczero 3.0.5
$ rustup toolchain list | grep -i risc -> risc0
```

`r0vm`/`cargo-risczero` are symlinked into `~/.cargo/bin`; `rzup` lives in `~/.risc0/bin` (needs `PATH`).
**r0vm 3.0.5 matches the `risc0-zkvm` 3.0.5 that LEZ v0.2.4 pins** — no version skew on the proving path.

**Not yet claimed:** no real `RISC0_DEV_MODE=0` proof has been generated on this machine yet. That is
**SC-B.3**'s job in Phase B, and prove-time seconds will be recorded there — not estimated here.

## 8. Dual license (plan §0.4.2 / H7)

`LICENSE-MIT` and `LICENSE-APACHE` are added **in this same first commit**, because §0.4.2 requires dual licensing
"from first commit" — which outranks the fact that the plan otherwise lists licences under Phase 0 (SC0.2).
Copyright holder: `pramadanif` (matches the repo's git identity). SC0.2 re-verifies them in Phase 0.

## 9. Exit

All three SC-N1 green → **proceed to Phase 0**.

Carried forward as live constraints:
- Pin **LEZ v0.2.4** + **SPEL `main` @ `5126b7ed8a9b`** (§5b) — not SPEL v0.6.0.
- Phases run **−1 → I** with no eligibility hard-stop (§3, operator decision 2026-09-04).
- Abort check (#125 APPROVED / merged LP-0002 PR) re-runs at the start of every phase and is logged in
  `docs/TRACKING.md`.

