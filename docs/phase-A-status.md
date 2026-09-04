# Phase A status — Design

**Date:** 2026-09-04
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase A
**Result:** **all 8 SC green — Phase A complete.**

Abort check at phase start: `gh pr view 125 … reviewDecision` → empty (not APPROVED); merged LP-0002
PRs → 0. Not aborting.

No feature code was written this phase, by contract ("do not skip ADR", design only). What was
produced is the specification Phase B implements without guessing.

## Artifacts

| File | Purpose |
|------|---------|
| `docs/adr/ADR-001-architecture.md` | The eight locked decisions, canonical formulas, six invariants, rejected alternatives |
| `docs/lez-account-model.md` | How `nonce` and `program_owner` are handled, with LEZ v0.2.4 citations |
| `docs/security.md` | Privacy surface per adversary, unlinkability claim, assumptions, attack table |
| `docs/error-codes.md` | 23 deterministic codes (12 on-chain, 11 client) with test mapping |
| `docs/tried-failed.md` | Running log of approaches attempted and abandoned |
| `docs/why-logos.md` | Argument outline for the write-up |

## Success criteria

| SC | Requirement | State | Evidence |
|----|-------------|-------|----------|
| **SC-A.1** | ADR locks PPE path + PDA `config_hash` + in-circuit binding + LEZ-native callee | ✅ green | ADR-001 **D1** (PPE only, no public approve path), **D3** (`config_hash` as PDA seed), **D4** (live-account binding), **D2** (LEZ-native membership program via `ChainedCall`) |
| **SC-A.2** | Nonce + `program_owner` explained with LEZ citation | ✅ green | `docs/lez-account-model.md` §2–§3: `account.rs:98` (record), `account.rs:21-26` (public counter), `account.rs:29-47` (private nsk-derived nonce), `program/mod.rs:202-212` (`pda_seeds` authority) |
| **SC-A.3** | ≥8 error codes | ✅ green | 23 distinct codes — 1001–1012 on-chain, 2001–2011 client. Verified: `grep -oE '\*\*[12][0-9]{3}\*\*' docs/error-codes.md \| sort -u \| wc -l` → 23 |
| **SC-A.4** | Nullifier + `config_hash` formulas copy-pasteable, no ambiguity | ✅ green | ADR-001 §3 — every field carries its byte width, endianness stated, domain separators given with exact ASCII+padding arithmetic (checked in Python, not estimated) |
| **SC-A.5** | Explicit table: other members learn X / do not learn Y | ✅ green | `docs/security.md` §1 — one row per fact, separate columns for observer / co-member / sequencer, each with the enforcing mechanism |
| **SC-A.6** | "Prover lowers M" → PDA fail, written as an invariant | ✅ green | ADR-001 **INV-1**; reinforced by **INV-3** (config account must rehash to its own address) and error code **1003** |
| **SC-A.7** | Alternatives rejected listed; `docs/tried-failed.md` stub exists | ✅ green | ADR-001 §7 (seven rejected alternatives with reasons); `docs/tried-failed.md` has four real entries, not a placeholder |
| **SC-A.8** | "Why Logos / why not centralised multisig" outline ready | ✅ green | `docs/why-logos.md` |

## The decision that matters most

**D4 — binding to a live account rather than a derived key.** This is what separates this design from
the derivation-only submission rejected in prize PR #91, so it was worked out against LEZ's source
rather than assumed:

- LEZ's PPE circuit, given `PrivateAuthorizedUpdate`, derives `npk` from the witness `nsk`, derives
  the account id, and **asserts it equals `pre_state.account_id`** (`output.rs:91-94`). It then proves
  the account's commitment sits under the live commitment-set root (`output.rs:347-357`). A prover
  cannot fabricate that root.
- Our membership guest re-derives the same account id from **its own** witnesses and asserts it
  matches that same pre-state, then proves `npk ∈ member_root` and emits the approval nullifier.

Composed: *the approver knows the `nsk` of a member of this multisig, and that member's account is
live and unspent right now.*

The re-derivation in the guest is not redundant, and `docs/tried-failed.md` records why: without it,
a member could give their real `nsk` to the PPE circuit and a different one to the membership guest,
producing a fresh nullifier per attempt and voting without limit. **SC-B.5** exists to prove that
assertion is load-bearing — a derivation-only stub must make a test fail.

## Privacy claim verified against source, not assumed

The design puts membership witnesses in the guest's `instruction_data`, which is only safe if that
never reaches the chain. Checked rather than assumed: the PPE circuit commits
`PrivacyPreservingCircuitOutput { public_actions, private_actions, block_validity_window,
timestamp_validity_window }`, where `PrivateAction = { nullifier, root, commitment,
encrypted_post_state }` (`circuit_io.rs:156-180`). The inner `program_outputs`, their
`instruction_data` and `account_identities` are circuit **inputs** — verified by `env::verify`, never
committed. Witnesses stay private.

Consequence recorded openly: the config and proposal accounts *are* public, so the approval count and
the nullifier set are public. That is what criterion P-F2 asks for.

## H14 / PF-13 — one `config_hash` formula

The formula string is now byte-identical (under preflight's normalisation) in `README.md` and
`docs/adr/ADR-001-architecture.md`:

```
config_hash=SHA256(DS_CONFIG‖member_root[32]‖M[1]‖N[1]‖multisig_id[32])
```

`docs/SOLUTION_DRAFT.md` is the third source PF-13 compares; it is written in Phase H and will quote
the same line. PF-13 stays `PENDING` until then — it is not claimed as passing.

## Verification run

| Command | Exit |
|---------|------|
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace --all-targets` | 0 |
| `./scripts/check-dev-mode-clobber.sh` | 0 |
| `./scripts/preflight-submission.sh` | 1 (expected — 12 checks still PENDING) |

## Exit

All SC-A green → **proceed to Phase B** (membership + nullifier guest, one real `RISC0_DEV_MODE=0`
proof, pinned ImageIDs).
