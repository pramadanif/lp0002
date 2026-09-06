# Criteria checklist

Every official Success Criterion from [`prizes/LP-0002.md`](plan/LP-0002.md), mapped to the evidence
that supports it — or marked plainly as unmet.

**Rule:** a row is ✅ only when a command was run and its output recorded. "The code does it" is not
evidence. Rows that are not there yet say so, and say what is missing.

Status: ✅ evidence exists · ◐ partial · ⛔ not met

---

## Functionality

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **P-F1** | Shielded member approves without revealing identity to on-chain observers **or other members** | ✅ | On chain: tx `f2458791…198fbcb5` from `Private/Dm4TU2ht…`; the proposal account holds a count and nullifiers only (`artifacts/phase-E-ppe-approve-SUCCESS.txt`). Against co-members: `crates/sdk/tests/peer_privacy.rs` — `prepare_approval` takes a public `MultisigView` plus the member's own secrets, and has **no parameter** for another member's identity |
| **P-F2** | On-chain verifier confirms M approvals **without recording which** members | ✅ | Decoded from chain: `approvals=1, nullifier[0]=c67cce32…, executed=false`, no identity field. `Proposal` has six fields and none can hold a roster (`crates/multisig-core/src/lib.rs`) |
| **P-F3** | A member cannot approve twice (nullifiers) | ✅ | `a_member_cannot_approve_the_same_proposal_twice` → error **1002** (`7002` on chain); `a_member_cannot_double_vote_from_another_of_their_addresses` (nullifier keyed to `nsk`, not account id) |
| **P-F4** | Completed execution unlinkable to any individual member | ◐ | Argued and tested at the record level (`the_on_chain_record_does_not_distinguish_which_members_approved`), and the on-chain approval carries no identity. **Not yet shown for a completed `execute`** — that needs the full 2-of-3 run |
| **P-F5** | Proof generation runs client-side on a standard laptop | ✅ | Standalone membership proof **115.97 s**; composed approval **≈19 min 26 s**, peak **8.74 GB**, on an 8-core 16 GB laptop with no GPU. Both in `docs/cu-costs.md`. Caveat stated: needs ~7 GB *free* (`docs/limitations.md` §10a) |
| **P-F6** | Reference integration: threshold-gated action on LEZ **testnet** with shielded members | ⛔ | Works on a **local** standalone sequencer. Nothing deployed to public testnet — no program id, no explorer links |
| **P-F7** | ≥1 multisig on testnet: create + propose + approve-to-threshold + execute, reproducible with evidence | ⛔ | Locally: create ✅, propose ✅, approve ✅ (1 of 2). Not yet: second approval, `execute`, and anything on public testnet. All four instructions are now exercised through the risc0 executor, which is how INV-7 was found: `execute` paid whichever account the submitter named rather than the one the proposal approved ([ADR-001 INV-7](adr/ADR-001-architecture.md)) |
| **P-F8** | Full documentation and a clean public repository | ◐ | Public repo, CI green, 123 tests, ADRs, security model, error codes, limitations, `SOLUTION_DRAFT.md`, `BUGS_FILED.md`, and a documentation index in the README. Missing: `DEPLOYMENT.md`, which needs the testnet run |

## Usability

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **P-U1** | Module/SDK for building Logos modules | ✅ | `pmsig-sdk` (prove + member API), `pmsig-core`, `pmsig-store`, `pmsig-cli`. Guide: `docs/integration.md`, whose code is the **compiled** example `crates/sdk/examples/integrate.rs` |
| **P-U2** | Basecamp GUI: local build, downloadable assets, loadable | ⛔ | `app/` exists and is generated from the IDL — QML, C++ backend, `CMakeLists.txt`, `module.yaml`, `manifest.json` — and its privacy properties are asserted by `scripts/check-basecamp-privacy.sh` in CI. **Unmet because nothing has been built or downloaded:** no `.lgx`, no SHA-256, no release asset. Blocked on Qt6, `cmake` and `lgx`, none installed ([`docs/phase-F-status.md`](phase-F-status.md)) |
| **P-U3** | IDL for the LEZ program, using SPEL | ✅ | `artifacts/multisig-idl.json`, generated from `#[lez_program]` at compile time by `scripts/generate-idl.sh`. Independently confirmed usable: the SPEL CLI built working commands from it and submitted real transactions |

## Reliability

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **P-R1** | Proof failures handled gracefully, clear error to the member | ✅ | `prove_failures` tests: truncated/empty binary → **2001**; the guest's own reason is passed through; dev mode → **2003** explaining *why*. `2002 ProverNotFound` names the install command |
| **P-R2** | Partial approvals (< M) preserved and resumable across client restarts | ✅ | `crates/store/tests/resume.rs` (9 tests) and `a_partial_approval_set_survives_between_processes` — each CLI command is its **own process**, so the restart is real. Atomic writes; a corrupt store is reported, never discarded |
| **P-R3** | Deterministic, documented error codes for all invalid-proof and double-vote cases | ✅ | 13 on-chain codes + 11 client codes in `docs/error-codes.md`. A test asserts **every** enum code appears in that document with the same number |

## Performance

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **P-P1** | CU cost of each on-chain operation documented (numeric) | ⛔ | Client-side proving costs are measured (`docs/cu-costs.md` §1). **On-chain CU per instruction is not measured** — needs testnet. The document says so rather than guessing, and preflight PF-08 blocks submission while it is missing |

## Supportability

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **P-S1** | Deployed and tested on LEZ devnet/testnet | ⛔ | Local standalone sequencer only |
| **P-S2** | E2E tests against a LEZ sequencer (**standalone**) in CI | ◐ | The `e2e-sequencer` job runs on **every push to `main`** — not on cron, not path-filtered, no `continue-on-error` — and builds LEZ v0.2.4 and the pinned SPEL from source before proving anything. It has **not yet completed a run**, so this is not green: each attempt has failed further along than the last, every failure a real defect ([`docs/phase-E-status.md`](phase-E-status.md)) |
| **P-S3** | CI green on the default branch | ✅ | GitHub Actions green on `main`: `fmt + clippy + tests`, `shellcheck`, `RISC0_DEV_MODE clobber check` |
| **P-S4** | README documents E2E usage: deploy steps, addresses, CLI **and** Basecamp | ◐ | README now has an end-to-end section: prerequisites (incl. the ~9 GB memory requirement), guest build, demo, testnet deploy, public verification, CLI and Basecamp walkthroughs. **Program addresses are still absent** because nothing is deployed to public testnet — stated rather than faked |
| **P-S5** | Reproducible `demo.sh` against a **real local sequencer** with `RISC0_DEV_MODE=0` | ◐ | `demo.sh` exists and drives a genuine standalone sequencer with `RISC0_DEV_MODE=0`; the sequencer, deployment, create, propose and one anonymous approval have all been demonstrated. **The script has not yet completed a full unattended run**, so this is not claimed green |
| **P-S6** | Narrated video showing terminal output incl. proof generation, confirming `RISC0_DEV_MODE=0` | ⛔ | Not recorded. **Human gate** — see `docs/limitations.md` |

---

## Cross-check against actual rejections

[`reviewer-gaps.md`](reviewer-gaps.md) reads all nine closed LP-0002 submissions and the three
accepted ones. Three causes account for nearly every rejection: **CI not running a real LEZ sequencer
(6×)**, **missing or dead testnet evidence (5×)**, and **missing CU cost (5×)**. Our two ⛔ rows on
CI and testnet are exactly those, and are the right place to spend remaining effort.

Causes that killed others and are already closed here: derivation-only binding (#91), dev-mode
clobber in a child script (#97), and no partial-approval resume (#91) — all three mutation-tested
rather than merely implemented.

## Summary

| | Count |
|---|---|
| ✅ evidence exists | **10** |
| ◐ partial | **5** |
| ⛔ not met | **6** |

**The single biggest gap is testnet.** Everything demonstrated so far is on a local standalone
sequencer. P-F6, P-F7, P-P1 and P-S1 all depend on a public testnet deployment, which needs funded
keys (a human gate). P-U2 (Basecamp) and P-S6 (video) are independent of that and also outstanding.

This file is regenerated by hand as phases land. Preflight check **PF-07** fails the submission if
any criterion id is missing from it.
