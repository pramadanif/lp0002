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
| Guest cycles | **602,662** (measured against the deployed binary — `artifacts/cu-measured.md`) |
| Proving time | **115.97 s** |
| `RISC0_DEV_MODE` | **0** (asserted inside the test, not merely set) |
| Journal size | **5,928 bytes** (inner journal — prover-local, never published; see below) |
| Receipt kind | Composite (`ProverOpts::default()`) |
| Prover | external `r0vm` 3.0.6 |
| Host | Darwin arm64, 8 cores, no GPU acceleration |
| Guest binary | `artifacts/membership.bin`, 393,868 bytes, reproducible (container r0.1.91.1) |
| Reproduce | `./scripts/build-guests.sh --docker && ./scripts/prove-bench.sh` |
| Log | `logs/phase-B-prove-bench.log` |

Re-measured 2026-09-07 against the reproducible binary. The earlier figures in this row — 53.26 s
and a 776-byte journal — were real, but they were measured on a design that **was abandoned**, and
are corrected below rather than quietly dropped.

**Does this satisfy "runs client-side on a standard laptop"?** Yes: 116 s on an 8-core laptop with
no GPU. That is a wait, not an inconvenience, for an action a member takes deliberately. The
measurement is a single sample on one machine, not a distribution — stated plainly rather than
dressed up as a benchmark suite.

### The witness split, and why the cheaper design is not the one we ship

An earlier design moved the member's secrets out of `instruction_data` into a private input. It was
faster and produced a much smaller journal:

| | Witness in `instruction_data` (**shipped**) | Witness as a private input (**abandoned**) |
|---|---|---|
| Proving time | **115.97 s** | 53.26 s |
| Journal size | **5,928 bytes** | 776 bytes |
| Member's `nsk` recoverable from that journal | **yes** | no |

**It does not work on LEZ.** LEZ writes a program exactly four inputs and offers no private channel
(`lee/state_machine/src/program/mod.rs::write_inputs`), so a guest reading a fifth input fails with
`DeserializeUnexpectedEnd` the moment a real transaction reaches it. It passed every host-side test
first. See `docs/tried-failed.md`.

So the shipped design costs roughly twice the proving time and an eight-times larger journal, and
the member's `nsk` **is** in that journal. That is not a leak: the journal in question is the
**inner** membership receipt, which is consumed by `env::verify` inside LEZ's privacy-preserving
circuit and never reaches the chain. It is prover-local secret material, and
`the_inner_journal_contains_the_witness_and_must_be_treated_as_secret` asserts exactly that, so the
property cannot drift silently. The chain-facing privacy claim is a separate assertion against
`PrivacyPreservingCircuitOutput` — see `docs/security.md` §3b.

Both columns were measured on the same machine with the same settings.

### The composed proof — measured

The 53.26 s above is the **standalone** membership proof: one program, composite receipt.

An approval as actually submitted is a **composition** — LEZ's privacy-preserving circuit calling
`env::verify` over the multisig program and the chained membership program — which needs succinct
receipts and therefore recursion. Measured on the same machine:

| Measurement | Value |
|-------------|-------|
| Proving time | **≈19 min 26 s** (21:25:01 → 21:44:27) |
| Peak r0vm resident memory | **8.74 GB** |
| Free RAM required (practical) | **≈7 GB** (a run starting from 7.4 GB free completed both approvals) |
| Swap movement | none — stable throughout |
| Stages observed | 7 (each releases memory on completion) |
| Host | 8-core laptop, 16 GB, **no GPU prover** |
| Result | tx `f2458791…198fbcb5`, confirmed in a block |

That is **≈22× the standalone proof**, which is the price of recursion over two inner programs.

**A caveat that matters more than the number:** an earlier attempt at the same proof did not finish
at all, because ~9 GB of the machine's 16 GB was already held by a browser and two editors. It was
contention, not capacity. See `docs/limitations.md` §10a.

### Known measurement anomaly

A *second* standalone proof in the same process did not complete within 25 minutes on two occasions,
while the first completed in 53 s. This was observed **before** we understood the memory picture, and
is now most likely the same cause: the machine was already loaded, so the second proof swapped. Not
re-tested. Recorded rather than smoothed over.

## 2. On-chain compute units (P-P1)

**Measured, all four instructions**, by running the **reproducible binaries the chain is given** in
the risc0 executor:

| Instruction | Program | Cycles |
|-------------|---------|--------|
| `create_multisig` | `multisig` | **155,809** |
| `create_proposal` | `multisig` | **257,625** |
| `execute` | `multisig` | **315,293** |
| `verify_approval` (chained from `approve`) | `membership` | **602,662** |

Regenerate with `./scripts/measure-cu.sh`; the raw output is `artifacts/cu-measured.md`.

Each figure is taken from a run that **succeeded**: the measurement decodes the guest's journal into
a `ProgramOutput`, which only a successful execution commits. Without that, a run ending in a
program error would burn cycles and be published here as the cost of the happy path — checked by
pointing `execute` at a payee the proposal never named, which fails with `7012
InvalidProposalAction` rather than reporting a number.

### Why cycles

LEZ runs programs in the risc0 zkVM and exposes no separate per-instruction compute counter — the
`GasCost` in its source is the Logos-layer publish fee, not per-instruction compute. On a zkVM the
quantity that *is* compute is the cycle count: it sets proving time, segment count, and any budget
the chain imposes. The prize's own note that "LEZ's per-transaction compute budget may change during
testnet" is consistent with that.

### What these figures are, and are not

`create_multisig`, `create_proposal` and `execute` are **public** transactions: no proof is
generated for them, but the sequencer still runs them in the zkVM, so they still cost cycles. That
is what the table reports.

These are cycles of the deployed binary measured **host-side in the executor**, not a reading from a
testnet meter — LEZ exposes no per-instruction compute counter to read one from. The executor runs
the same ELF the sequencer runs, so the cycle counts are the same quantity; what a host measurement
cannot capture is sequencer-side overhead outside the guest.

Criterion P-P1 requires actual figures. Preflight check PF-08 looks for this per-instruction table
specifically — not merely for digits somewhere in the file, which the proving benchmarks above would
otherwise satisfy — and refuses a submission whose table is missing, empty or hedged. Several
historical submissions were rejected for reporting no compute-unit figures at all; that is the
failure this gate exists to prevent.
