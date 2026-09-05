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
| Guest cycles | **598,666** (measured against the deployed binary — `artifacts/cu-measured.md`) |
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

### The composed proof — measured

The 53.26 s above is the **standalone** membership proof: one program, composite receipt.

An approval as actually submitted is a **composition** — LEZ's privacy-preserving circuit calling
`env::verify` over the multisig program and the chained membership program — which needs succinct
receipts and therefore recursion. Measured on the same machine:

| Measurement | Value |
|-------------|-------|
| Proving time | **≈19 min 26 s** (21:25:01 → 21:44:27) |
| Peak r0vm resident memory | **8.74 GB** |
| Free RAM required (practical) | **≈9 GB** |
| Swap movement | none — stable throughout |
| Stages observed | 7 (each releases memory on completion) |
| Host | 8-core laptop, 16 GB, **no GPU prover** |
| Result | tx `f2458791…198fbcb5`, confirmed in a block |

That is **≈22× the standalone proof**, which is the price of recursion over two inner programs.

**A caveat that matters more than the number:** an earlier attempt at the same proof did not finish
at all, because ~9 GB of the machine's 16 GB was already held by a browser and two editors. It was
contention, not capacity. See `docs/limitations.md` §10a.

### Known measurement anomaly

### Known measurement anomaly

A *second* standalone proof in the same process did not complete within 25 minutes on two occasions,
while the first completed in 53 s. This was observed **before** we understood the memory picture, and
is now most likely the same cause: the machine was already loaded, so the second proof swapped. Not
re-tested. Recorded rather than smoothed over.

## 2. On-chain compute units (P-P1)

**Partially measured.** The instruction that actually costs anything — `approve`, the only
privacy-preserving one — is measured against the **deployed binary**:

| Instruction | Program | Cycles |
|-------------|---------|--------|
| `verify_approval` (chained from `approve`) | `membership` | **598,666** |

Regenerate with `./scripts/measure-cu.sh`; the raw output is `artifacts/cu-measured.md`.

### Why cycles

LEZ runs programs in the risc0 zkVM and exposes no separate per-instruction compute counter — the
`GasCost` in its source is the Logos-layer publish fee, not per-instruction compute. On a zkVM the
quantity that *is* compute is the cycle count: it sets proving time, segment count, and any budget
the chain imposes. The prize's own note that "LEZ's per-transaction compute budget may change during
testnet" is consistent with that.

### Still outstanding

`create_multisig`, `create_proposal` and `execute` are **public** transactions executed directly by
the sequencer, and their per-instruction figures are **not yet measured on a public testnet**.

**Still to do — Phase G.** Will carry one numeric CU figure per instruction
(`create_multisig`, `create_proposal`, `approve`, `execute`), measured against the live LEZ testnet
with the deployed program.

Criterion P-P1 requires actual figures. Preflight check PF-08 looks for this per-instruction table
specifically — not merely for digits somewhere in the file, which the proving benchmarks above would
otherwise satisfy — and refuses a submission whose table is missing, empty or hedged. Several
historical submissions were rejected for reporting no compute-unit figures at all; that is the
failure this gate exists to prevent.
