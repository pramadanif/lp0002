# Costs and benchmarks

Two different things live here, and they are kept apart on purpose:

- **Client-side proving** — how long a member waits to produce an approval proof (criterion **P-F5**).
  Measured in Phase B; numbers below are real.
- **On-chain compute units** — the CU cost of each instruction (criterion **P-P1**). Measured against
  the live testnet in Phase G. **This section is deliberately empty until then rather than filled
  with estimates**; preflight check PF-08 keeps the submission gate closed while it is missing.

Every number below was produced by a command that was run, with the log path given. Nothing here is
extrapolated.

---

## 1. Client-side proof generation (P-F5)

Guest: `programs/membership-lez` — membership + nullifier verification with live-account binding.

| Measurement | Value |
|-------------|-------|
| Guest cycles | **598,184** |
| Proving time | **53.26 s** |
| `RISC0_DEV_MODE` | **0** (asserted inside the test, not merely set) |
| Journal size | **776 bytes** |
| Receipt kind | Composite (`ProverOpts::default()`) |
| Prover | external `r0vm` 3.0.6 |
| Host | Darwin arm64, 8 cores, no GPU acceleration |
| Guest binary | `artifacts/membership.bin`, 377,084 bytes |
| Reproduce | `./scripts/build-guests.sh && ./scripts/prove-bench.sh` |
| Log | `logs/phase-B-prove-bench.log` |

**Does this satisfy "runs client-side on a standard laptop"?** Yes: 53 s on an 8-core laptop with no
GPU, well inside what a member waits for a governance action. The measurement is a single sample on
one machine, not a distribution — stated plainly rather than dressed up as a benchmark suite.

### What the witness split bought

Moving the member's secrets out of `instruction_data` and into a private input (see
`docs/tried-failed.md`) was done for privacy, but it also made proving substantially cheaper, because
the witness is no longer serialised into the committed journal:

| | Witness in `instruction_data` | Witness as a private input |
|---|---|---|
| Proving time | 123.86 s | **53.26 s** |
| Journal size | 5,928 bytes | **776 bytes** |
| Member's `nsk` recoverable from journal | **yes** | **no** |

Both rows were measured on the same machine with the same settings.

### Known measurement anomaly

A *second* proof in the same process did not complete within 25 minutes on two occasions, while the
first completed in 53 s. Not diagnosed; recorded in `docs/tried-failed.md` rather than smoothed over.
It does not affect the number above, which is the first proof of a fresh process — the case a member
actually experiences.

## 2. On-chain compute units (P-P1)

**Not yet measured — Phase G.** Will carry one numeric CU figure per instruction
(`create_multisig`, `create_proposal`, `approve`, `execute`), measured against the live LEZ testnet
with the deployed program.

Criterion P-P1 requires actual figures. Preflight check PF-08 looks for this per-instruction table
specifically — not merely for digits somewhere in the file, which the proving benchmarks above would
otherwise satisfy — and refuses a submission whose table is missing, empty or hedged. Several
historical submissions were rejected for reporting no compute-unit figures at all; that is the
failure this gate exists to prevent.
