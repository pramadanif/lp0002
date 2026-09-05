# Prize checklist — LP-0002 Private M-of-N Multisig

Mirror of the official Success Criteria in [`prizes/LP-0002.md`](docs/plan/LP-0002.md), with the plan's
`P-*` ids attached so every criterion has one name across this repo.

## Status lives in one place, and it is not this file

**→ [`docs/criteria-checklist.md`](docs/criteria-checklist.md)** carries the status and evidence for
every criterion below, including what is **not** met.

This file used to carry a status column too, and it went stale: it showed all 21 criteria as "not
started" long after several were done and evidenced, so the two documents contradicted each other.
Two places holding the same fact is how documentation drifts — the same failure this repository
guards against for the `config_hash` formula, which is asserted identical across three files by
preflight PF-13. Rather than keep two tables in step by hand, the status column is gone and the
criteria list stays, which is what the plan asked this file to be (§5 Phase 0: "mirror of §1").

**Rule, unchanged:** a criterion counts as met only when a command was run and its evidence path
recorded. "Looks done" is not done.

## Functionality

| ID | Criterion | Phase |
|----|-----------|-------|
| P-F1 | Shielded member approves without revealing identity to on-chain observers **or other members** | A/C/D |
| P-F2 | On-chain verifier confirms M approvals reached **without recording which** members approved | C |
| P-F3 | A member cannot approve the same proposal twice (nullifiers) | B/C |
| P-F4 | Completed execution unlinkable to any individual member's shielded account | A/C/G |
| P-F5 | Proof generation runs client-side on a standard laptop | B/E |
| P-F6 | Reference integration: threshold-gated action (treasury transfer) on LEZ testnet with shielded members | G |
| P-F7 | ≥1 multisig on testnet: create + propose + approve-to-threshold + execute; reproducible + evidence | G |
| P-F8 | Full documentation and a clean public repository | H |

## Usability

| ID | Criterion | Phase |
|----|-----------|-------|
| P-U1 | Module/SDK for building Logos modules against the program | D |
| P-U2 | Basecamp GUI: local build instructions, downloadable assets, loadable in Logos app | F |
| P-U3 | IDL for the LEZ program, using the SPEL framework | C |

## Reliability

| ID | Criterion | Phase |
|----|-----------|-------|
| P-R1 | Proof-generation failures handled gracefully, clear error to the member | D |
| P-R2 | Partial approvals (fewer than M) preserved and resumable across client restarts | D |
| P-R3 | Deterministic, documented error codes for all invalid-proof and double-vote scenarios | A/C |

## Performance

| ID | Criterion | Phase |
|----|-----------|-------|
| P-P1 | CU cost of each on-chain operation documented (numeric, never "unavailable") | G |

## Supportability

| ID | Criterion | Phase |
|----|-----------|-------|
| P-S1 | Program deployed and tested on LEZ devnet/testnet | G |
| P-S2 | E2E integration tests against a LEZ sequencer (**standalone mode**) included in CI | E |
| P-S3 | CI green on the default branch | E |
| P-S4 | README documents E2E usage: deploy steps, program addresses, CLI **and** Basecamp steps | H |
| P-S5 | Reproducible `demo.sh` against a **real local sequencer** with `RISC0_DEV_MODE=0` | E |
| P-S6 | Narrated video showing terminal output incl. proof generation, confirming `RISC0_DEV_MODE=0` | H |

## Submission requirements (beyond the criteria table)

| Item | Phase | Status | Evidence |
|------|-------|--------|
| Public repo, MIT **or** Apache-2.0 (this repo is dual) | 0 | ☑ | `LICENSE-MIT` + `LICENSE-APACHE`, present since the first commit `f6a1a15` |
| Verifier program deployed with a verified program ID | G | ☐ | |
| Narrated architecture + demo video (not a silent screencast) | H | ☐ | **human gate** |
| Write-up: threshold proof scheme, nullifier design, LEZ account model (`nonce` + `program_owner`), security assumptions, known limitations, integration guide | A/H | ☐ | |
| Proof generation time + on-chain verification cost benchmarks | B/G | ☐ | |
| Reproducible deployment steps + evidence for ≥1 testnet multisig instance | G | ☐ | |

## Notes

- Phase I (opening the solution PR) proceeds per the operator's decision of 2026-09-04; see
  `docs/phase-N1-status.md` §3 for the public record on upstream issue #105.
