# Error codes

Prize criterion **P-R3** requires deterministic, documented error codes for every invalid-proof and
double-vote scenario. This is that catalogue.

**Deterministic** means: the same invalid input always yields the same code, the code is a stable
`u32` that never changes meaning, and the mapping is testable. Phase C asserts each on-chain code
with a test; Phase D asserts the client-side ones.

Codes are grouped by origin. Numbering is stable — a retired code is never reused.

---

## 1. On-chain program errors (`7xxx` on chain, `1xxx` internally)

Returned by the multisig program. Visible to anyone inspecting the failed transaction. They must not
leak identity, so none of them names a member.

### Wire format

The program reports these through `SpelError::custom(code, name)`. SPEL maps `Custom { code }` to
the numeric code **`6000 + code`** (`spel-framework-core/src/error.rs`), so a client sees *two*
numbers:

```text
Program error [7002]: Program error 1002: DuplicateNullifier
               ~~~~                  ~~~~  ~~~~~~~~~~~~~~~~~~
               on-chain code         our code, and its name
               (6000 + 1002)
```

**`6000 + code` is the authoritative on-chain code** — it is what a transaction inspector,
explorer, or client library reports. The bare `1xxx` appears only inside the message. Both columns
below are therefore listed, and both halves are asserted by
`crates/sdk/tests/multisig_program.rs::assert_program_error`, which also pins the `6000` offset: if
SPEL changes it, those tests fail rather than the docs silently going stale.

The offset also means our numbering cannot collide with SPEL's own framework codes, which occupy
1000-1010 — a bare `1002` from the framework (`AccountAlreadyInitialized`) and ours
(`DuplicateNullifier`) are distinguishable on chain as `1002` versus `7002`.

| On chain | Code | Name | Meaning | Raised when |
|----------|------|------|---------|-------------|
| **7001** | **1001** | `InvalidProof` | The membership proof did not verify | The chained membership call's output fails `env::verify`, or its `self_program_id`/`caller_program_id` do not match the expected call |
| **7002** | **1002** | `DuplicateNullifier` | Double-vote attempt | `nf_approve` is already in the proposal's nullifier set (INV-4). **This is the double-vote code.** |
| **7003** | **1003** | `ConfigHashMismatch` | Config account does not match its own address | the canonical `config_hash` formula (ADR-001 §3) recomputed over the stored fields ≠ the PDA seed the account was found under (INV-3) |
| **7004** | **1004** | `ThresholdNotMet` | Execute attempted below threshold | `execute` with `approvals < M` (INV-6) |
| **7005** | **1005** | `AlreadyExecuted` | Proposal executed twice | `execute` on a proposal whose `executed` flag is set |
| **7006** | **1006** | `UnknownProposal` | Proposal does not belong to this multisig | The proposal account's `config_hash` ≠ the config account's, or its address ≠ `for_public_pda(program_id, proposal_pda_seed)` |
| **7007** | **1007** | `MemberRootMismatch` | Approval proved against a different member set | The membership output's `member_root` ≠ the root in the config account (catches a stale root after a config change, INV-5) |
| **7008** | **1008** | `ProposalClosed` | Approval arrived after execution | `approve` on a proposal already executed |
| **7009** | **1009** | `InvalidThresholdConfig` | Nonsensical configuration at creation | `create_multisig` with `M == 0`, `N == 0`, or `M > N` |
| — † | **1010** | `AccountAlreadyInitialized` | Multisig or proposal created twice | Raised by `pmsig-cli` as a pre-check, so a duplicate `create` fails before a transaction is built. **† Never appears on chain**: there, SPEL's `#[account(init)]` constraint catches the same condition first and raises its own framework error `1002` (see §1.2) |
| **7012** | **1012** | `InvalidProposalAction` | Proposed action is malformed or unsupported | Action fails to decode, or a transfer names an account the multisig does not control |
| **7013** | **1013** | `WrongMembershipProgram` | The approval was verified by a program this multisig is not bound to | The chained call's `program_id` ≠ the `membership_program_id` the config account rehashes to ([ADR-002](adr/ADR-002-bind-verifier-to-config-hash.md)) |

### 1.2 Framework errors from SPEL (`1000`–`1010`)

Not every on-chain failure is ours. SPEL raises its own errors *before* our handler body runs —
account-constraint violations, deserialisation, overflow — and those keep their bare numbers,
`1000`–`1010` (`spel-framework-core/src/error.rs`). Because ours are offset by `6000`, the two sets
never collide: a bare `1002` on chain is SPEL's `AccountAlreadyInitialized`, while ours is `7002`
`DuplicateNullifier`.

The one an integrator meets in normal use is **`1002`**, from the `#[account(init)]` constraint on
`create_multisig` / `create_proposal` when the target PDA already holds an account.

### 1.3 Retired codes

Never reused, listed so an older log or client remains readable.

| Code | Name | Why retired |
|------|------|-------------|
| `1011` | `PublicApprovePathRejected` | Never raisable. It was meant for an `approve` arriving outside the privacy-preserving path, but that case is *unrepresentable* rather than rejected: `approve` requires a `verified_by` program id that only the dispatcher can supply, from a chained call that exists only on the privacy-preserving path. Nothing could construct the error, and its only test asserted the constant against itself. The guarantee is structural and is asserted by `there_is_no_public_approve_path` (SC-C.8); carrying a code the program claimed to raise and never did was an overclaim, so it was removed |

## 2. Client-side SDK errors (2xxx)

Surfaced to the member by `pmsig-sdk`. Criterion **P-R1** requires proof-generation failure to reach
the member as a clear error — so each carries what the member can actually do about it.

| Code | Name | Meaning | Member-facing guidance |
|------|------|---------|------------------------|
| **2001** | `ProofGenerationFailed` | The prover did not produce a receipt | Underlying error and log path are included. Most common causes: `r0vm` missing, or the machine ran out of memory |
| **2002** | `ProverNotFound` | `r0vm` is not installed or not on `PATH` | Reports the install command. **Never** silently degrades to dev mode |
| **2003** | `DevModeRefused` | `RISC0_DEV_MODE=1` on a real submission path | Dev-mode receipts prove nothing; the SDK refuses rather than producing a worthless proof (H3) |
| **2004** | `NotAMember` | The supplied `nsk` derives an `npk` that is not under `member_root` | Wrong key, or wrong multisig |
| **2005** | `AccountNotLive` | The member's account commitment is not in the live commitment set | Usually the account was already spent by a concurrent transaction — refresh state and retry |
| **2006** | `AlreadyApproved` | This member already approved this proposal | Detected locally from the on-chain nullifier set before wasting a proof |
| **2007** | `StaleProposal` | Local state is behind the chain | The proposal was executed or closed; resynchronise |
| **2008** | `SequencerUnreachable` | The RPC endpoint did not answer | Endpoint and underlying transport error are included |
| **2009** | `SequencerRejected` | The sequencer refused the transaction | The sequencer's reason is passed through verbatim, not reinterpreted |
| **2010** | `StoreCorrupt` | The local approval store failed to load | Path and parse error included; the store is never silently discarded (**P-R2**) |
| **2011** | `ConfigMismatch` | Local multisig config does not hash to the on-chain address | Protects against a tampered or wrong local config file |

## 3. Design notes

**Why on-chain codes say so little.** Codes 1001–1012 deliberately carry no member-identifying detail.
A code such as "member 2 already approved" would defeat the entire scheme. `DuplicateNullifier`
reports *that* a nullifier repeated, never whose.

**Failing before proving.** Several client errors (2004, 2006, 2007) are detectable from public state
before proof generation, which takes far longer than the check. The SDK checks them first — cheaper,
and a much clearer message than a proof that fails for an unstated reason.

**Dev mode is an error, not a fallback.** 2003 exists because the tempting failure mode, when `r0vm`
is missing, is to fall back to `RISC0_DEV_MODE=1` and appear to succeed. That produces a fake receipt.
The SDK treats it as a hard error, matching gates H2 and H3.

## 4. Test mapping

Each code is asserted by a test; this table is filled in with test names as the phases land.

| Codes | Asserted in | Phase |
|-------|-------------|-------|
| 1001, 1002, 1007, 1013 | membership guest + program negative tests | B, C |
| 1003, 1006, 1009, 1010 | PDA / config validation tests | C |
| 1004, 1005, 1008, 1012 | lifecycle tests | C |
| 2001–2003 | prover failure injection | D |
| 2004–2007, 2011 | pre-flight validation tests | D |
| 2008, 2009 | RPC failure tests | D |
| 2010 | store round-trip + corruption test | D |
