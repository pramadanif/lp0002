# Integration guide

How to build a Logos module against this multisig (criterion **P-U1**).

**The code in this guide is compiled.** Rather than paste snippets that rot, the working example
lives at [`crates/sdk/examples/integrate.rs`](../crates/sdk/examples/integrate.rs) and is built by
`cargo test --workspace --all-targets`. Run it:

```bash
cargo run -p pmsig-sdk --example integrate
```

---

## The crates

| Crate | Use it for |
|-------|-----------|
| `pmsig-core` | The formulas: `config_hash`, `approval_nullifier`, `member_leaf`, and the `MemberTree` |
| `pmsig-multisig-core` | On-chain state types, the 13 error codes, and the four transition rules |
| `pmsig-membership-core` | `ApprovalClaim` (public) and `ApprovalWitness` (secret) |
| `pmsig-sdk` | The member-facing API and proof generation |
| `pmsig-store` | Durable partial-approval sets |

## The five steps

1. **Build the member set.** Members share their nullifier *public* keys; you commit a `MemberTree`
   root. The leaves are never published on chain.
2. **Create the multisig.** `logic::create_multisig` returns the config plus the `config_hash` that
   seeds its PDA.
3. **Propose.** Content is public — the prize hides *who approved*, not *what was proposed*.
4. **Approve.** `member::prepare_approval` takes a `MultisigView` (public) and `MemberSecrets`
   (private) and returns a claim/witness pair. Prove it with `prove::prove_approval`, then submit.
5. **Execute.** Anyone may submit this once the threshold is met — including a non-member.

## Two things that are easy to get wrong

**Do not put secrets in the instruction.** A LEZ program echoes `instruction_data` into the
`ProgramOutput` it commits, so anything there lands in the guest's journal. That is why the witness is
a separate private input. We shipped this bug and caught it by decoding a journal — see
[`tried-failed.md`](tried-failed.md).

**Do not let a coordinator collect "who approved".** It would pass every on-chain test and still fail
P-F1, which requires privacy from *other members*. `prepare_approval` has no parameter for another
member's identity, which is the point: the type system refuses the mistake.

## Storing partial approvals

`ApprovalStore` writes atomically (temp file + rename) so a killed client leaves either the previous
complete file or nothing. A corrupt store is **reported**, never silently discarded — discarding it
would throw away exactly the partial set **P-R2** requires be kept.

```rust,ignore
let store = ApprovalStore::new(dir);
store.record(&ApprovalRecord { multisig_id, proposal_id, nullifier, status: ApprovalStatus::Confirmed })?;
let resumed = store.approval_count(&proposal_id)?;   // survives a restart
```

## Error handling

Codes are stable and documented in [`error-codes.md`](error-codes.md). On-chain failures are
`1001`–`1013`; client failures `2001`–`2011`. Both render as `"<code> <Name>"`, so a failed
transaction can be grepped against the catalogue. `MultisigError` implements `std::error::Error`, so
it composes with `?`.

Three worth handling explicitly:

- **2002 `ProverNotFound`** — `r0vm` is missing. The message names the install command.
- **2003 `DevModeRefused`** — `RISC0_DEV_MODE=1` produces a receipt that proves nothing. The SDK
  refuses rather than appearing to succeed.
- **2006 `AlreadyApproved`** — detected locally from the on-chain nullifier set, before spending
  ~53 s on a proof.

## The CLI

`pmsig` drives the same crates:

```bash
pmsig create   --members <nsk,nsk,nsk> --m 2
pmsig propose  --proposal-id <hex> --recipient <hex> --amount 1000
pmsig approve  --proposal-id <hex> --member <nsk>
pmsig execute  --proposal-id <hex>
pmsig status   --proposal-id <hex>
```

Two limits of the current CLI, stated plainly:

- It runs against a **local state file**, not a sequencer. Every such command prints `[local]`. The
  network transport lands in Phase E; until then no CLI output is testnet evidence.
- `create` takes every member's secret key so one machine can act as several members in a demo. A
  real deployment never does this: each member derives their own npk, shares only that, and keeps
  their own authentication path. The on-chain state has no member list at all.
