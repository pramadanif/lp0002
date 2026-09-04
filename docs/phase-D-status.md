# Phase D status — SDK / CLI / resume / peer privacy

**Date:** 2026-09-04
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase D
**Result:** **all 5 SC green — Phase D complete.**

Abort check at phase start: #125 `reviewDecision` empty; merged LP-0002 PRs → 0. Not aborting.

## Verification run

| Command | Exit |
|---------|------|
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace --all-targets` | 0 — **102 tests** (was 74) |
| `cargo run -p pmsig-sdk --example integrate` | 0 |
| `shellcheck -S warning scripts/*.sh` | 0 |

## Success criteria

| SC | Requirement | State | Evidence |
|----|-------------|-------|----------|
| **SC-D.1** | CLI full lifecycle local (**P-U1**) | ✅ green | `the_cli_drives_the_whole_lifecycle` runs the real binary as a subprocess through create → propose → approve×2 → execute, asserting `1004` below threshold and `1005` on re-execution. 7 CLI tests total |
| **SC-D.2** | Kill client mid-threshold → resume still reaches M (**P-R2**) | ✅ green | `a_partial_approval_set_survives_between_processes` — each CLI command is its **own process**, so the restart is real, not simulated. Plus 9 store tests incl. atomic writes and corruption handling |
| **SC-D.3** | Prove failure → clear error (**P-R1**) | ✅ green | `prove_failures` — a truncated binary and an empty binary both yield `2001`; a rejected witness passes the guest's own reason through ("not a member"); dev mode yields `2003` saying *why* |
| **SC-D.4** | Integration guide builds/compiles | ✅ green | `docs/integration.md` does not contain snippets — it points at `crates/sdk/examples/integrate.rs`, which `cargo test --workspace --all-targets` compiles and which runs the full lifecycle |
| **SC-D.5** | Co-member approves knowing only ids + on-chain count, **not** the first member's account id (**W8**, **P-F1**) | ✅ green | `a_co_member_approves_without_learning_who_approved_first`, plus 4 more in `peer_privacy.rs`. See below |

## Peer privacy is enforced by the type system, not by discipline

The half of **P-F1** that is easy to lose is privacy from *other members*: a coordinator that tracks
"who has approved" passes every chain-level test and still fails the criterion.

So the API splits its inputs, and the split **is** the guarantee:

- `MultisigView` — ids, member **root**, `M`, `N`, and the on-chain approval **count**. All public.
  There is no field for a co-member's account id, npk or approval status, because there is nowhere to
  put one.
- `MemberSecrets` — the member's own key material and Merkle path. `Debug` renders it as
  `<redacted>`, so a stray `dbg!` cannot put a spending key in a log (asserted by test).

`prepare_approval` takes one of each. It is therefore not possible to *call* it with another member's
identity. `a_co_member_approves_without_learning_who_approved_first` has Bob approve after Alice using
a view assembled purely from chain data; `the_on_chain_record_does_not_distinguish_which_members_approved`
runs two different approver pairs and shows the records are structurally identical.

`status` in the CLI is held to the same rule: a test asserts no member key and no member npk appears
in its output.

## Failing fast, before a 53-second proof

Two client-side checks exist purely so a member does not wait a minute for a rejection the client
could see immediately: `2006 AlreadyApproved` (the nullifier is already in the on-chain set) and
`2007 StaleProposal` (the threshold is already met). Both are asserted by test.

`2002 ProverNotFound` names the install command in its message. An error that tells a member what
failed but not what to do is only half of P-R1.

## Two things this phase did not paper over

1. **`unsafe` stayed forbidden.** The dev-mode refusal test wanted to set `RISC0_DEV_MODE` in-process,
   which needs `unsafe` — forbidden workspace-wide, and it would race every other test. The parsing
   logic was extracted into a pure `dev_mode_from(Option<&str>)` and tested directly; the real
   environment is covered by `scripts/prove-bench.sh`, which sets `RISC0_DEV_MODE=0` and has the
   proving test assert it is off.
2. **`MultisigError` became a real `std::error::Error`.** The integration example could not use `?`
   with it, which is exactly the friction an integrator would hit. Fixed in the type rather than
   worked around in the example.

## Local mode is labelled, not disguised

The CLI runs against a local state file, not a sequencer. Every such command prints `[local]`, the
integration guide says so, and `docs/limitations.md` will carry it. Two specific caveats:

- No CLI output is testnet evidence. The network transport lands in Phase E.
- `create` takes every member's secret key so one machine can act as several members in a demo. A
  real deployment never does this — each member derives their own npk, shares only that, and keeps
  their own authentication path. The on-chain state has no member list at all.

## Exit

All SC-D green → **proceed to Phase E** (`demo.sh` against a standalone sequencer with
`RISC0_DEV_MODE=0`, and CI e2e on push — where the PPE composition finally gets demonstrated rather
than designed).
