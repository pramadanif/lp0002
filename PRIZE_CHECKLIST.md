# Prize checklist — LP-0002 Private M-of-N Multisig

Mirror of the official Success Criteria in [`prizes/LP-0002.md`](docs/plan/LP-0002.md), with the plan's
`P-*` ids attached so every criterion has one name across this repo.

**Rule:** a box is ticked only when a command was run and its evidence path is recorded. "Looks done"
is not done. The evidence column stays empty until then. `docs/criteria-checklist.md` (Phase H) is the
final, evidence-complete version of this table — this file is the working mirror.

Status legend: ☐ not started · ◐ in progress · ☑ done with evidence

## Functionality

| ID | Criterion | Phase | Status | Evidence |
|----|-----------|-------|--------|----------|
| P-F1 | Shielded member approves without revealing identity to on-chain observers **or other members** | A/C/D | ☐ | |
| P-F2 | On-chain verifier confirms M approvals reached **without recording which** members approved | C | ☐ | |
| P-F3 | A member cannot approve the same proposal twice (nullifiers) | B/C | ☐ | |
| P-F4 | Completed execution unlinkable to any individual member's shielded account | A/C/G | ☐ | |
| P-F5 | Proof generation runs client-side on a standard laptop | B/E | ☐ | |
| P-F6 | Reference integration: threshold-gated action (treasury transfer) on LEZ testnet with shielded members | G | ☐ | |
| P-F7 | ≥1 multisig on testnet: create + propose + approve-to-threshold + execute; reproducible + evidence | G | ☐ | |
| P-F8 | Full documentation and a clean public repository | H | ☐ | |

## Usability

| ID | Criterion | Phase | Status | Evidence |
|----|-----------|-------|--------|----------|
| P-U1 | Module/SDK for building Logos modules against the program | D | ☐ | |
| P-U2 | Basecamp GUI: local build instructions, downloadable assets, loadable in Logos app | F | ☐ | |
| P-U3 | IDL for the LEZ program, using the SPEL framework | C | ☐ | |

## Reliability

| ID | Criterion | Phase | Status | Evidence |
|----|-----------|-------|--------|----------|
| P-R1 | Proof-generation failures handled gracefully, clear error to the member | D | ☐ | |
| P-R2 | Partial approvals (fewer than M) preserved and resumable across client restarts | D | ☐ | |
| P-R3 | Deterministic, documented error codes for all invalid-proof and double-vote scenarios | A/C | ☐ | |

## Performance

| ID | Criterion | Phase | Status | Evidence |
|----|-----------|-------|--------|----------|
| P-P1 | CU cost of each on-chain operation documented (numeric, never "unavailable") | G | ☐ | |

## Supportability

| ID | Criterion | Phase | Status | Evidence |
|----|-----------|-------|--------|----------|
| P-S1 | Program deployed and tested on LEZ devnet/testnet | G | ☐ | |
| P-S2 | E2E integration tests against a LEZ sequencer (**standalone mode**) included in CI | E | ☐ | |
| P-S3 | CI green on the default branch | E | ◐ | `quality` + `dev-mode-clobber` + `shellcheck` green from Phase 0; `e2e-sequencer` lands in Phase E |
| P-S4 | README documents E2E usage: deploy steps, program addresses, CLI **and** Basecamp steps | H | ☐ | |
| P-S5 | Reproducible `demo.sh` against a **real local sequencer** with `RISC0_DEV_MODE=0` | E | ☐ | |
| P-S6 | Narrated video showing terminal output incl. proof generation, confirming `RISC0_DEV_MODE=0` | H | ☐ | **human gate** |

## Submission requirements (beyond the criteria table)

| Item | Phase | Status | Evidence |
|------|-------|--------|----------|
| Public repo, MIT **or** Apache-2.0 (this repo is dual) | 0 | ☑ | `LICENSE-MIT` + `LICENSE-APACHE`, present since the first commit `f6a1a15` |
| Verifier program deployed with a verified program ID | G | ☐ | |
| Narrated architecture + demo video (not a silent screencast) | H | ☐ | **human gate** |
| Write-up: threshold proof scheme, nullifier design, LEZ account model (`nonce` + `program_owner`), security assumptions, known limitations, integration guide | A/H | ☐ | |
| Proof generation time + on-chain verification cost benchmarks | B/G | ☐ | |
| Reproducible deployment steps + evidence for ≥1 testnet multisig instance | G | ☐ | |

## Blocked

| Item | Why |
|------|-----|
| Opening the solution PR (Phase I) | Eligibility unresolved — see `docs/phase-N1-status.md` §3 (upstream issue #105). The build hard-stops after Phase H. |
