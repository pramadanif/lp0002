# ADR-002 — Bind the membership verifier into `config_hash`

**Status:** Accepted (Phase C, 2026-09-04)
**Amends:** ADR-001 §3 (the `config_hash` formula) and INV-1/INV-2
**Written before the code change, as ADR-001 §"If architecture changes later" requires.**

## Context

ADR-001 D2 has `approve` chain to a separate membership program, and LEZ's privacy-preserving circuit
proves it ran via `env::verify(chained_call.program_id, …)`. That proves *some* program with that id
ran and accepted the witness.

Implementing `approve` in Phase C exposed the gap: **nothing said which program id that had to be.**
`config_hash` bound the member set, `M` and `N`, but not the verifier. So an attacker could stand up
their own "membership" program — one that accepts every witness — create a multisig naming it, and
approve freely. The member set would be honest and the threshold real; the thing checking membership
would not be.

Storing the verifier id in the config account instead is weaker: whoever creates the account at that
address chooses it, so an honest member set could be paired with a hostile verifier on a
trust-on-first-use basis.

## Decision

Fold the membership program id into the digest that seeds the PDA:

```text
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] ‖ membership_program_id[32] )
```

`membership_program_id` is a LEZ `ProgramId` — `[u32; 8]`, serialised little-endian, 32 bytes. ADR-001
§3 anticipated this: the plan's formula was `H(member_root ‖ M ‖ extra)`, and this is that `extra`.

`approve` then requires the chained call's `program_id` to equal the one the config account rehashes
to. Naming a different verifier changes `config_hash`, which changes the PDA address, which means the
multisig is not there.

## Consequences

- **INV-1 and INV-2 extend to the verifier.** Substituting the membership program is now the same
  class of failure as lowering `M`: it does not weaken the multisig, it names one that does not exist.
- **The verifier is pinned at creation and cannot be swapped**, not even by whoever created the
  account.
- **Upgrading the membership program changes every `config_hash`.** A new verifier build has a new
  ImageID, hence a new address for every multisig. Existing multisigs keep working against the
  verifier they were created with; new ones must be created to adopt a new verifier. Recorded in
  `docs/limitations.md`.
- One identical formula string still appears in README, ADR-001 §3 and the solution write-up, so
  H14/W17 and preflight PF-13 are unaffected in kind — the string simply gained a field.

## Alternatives rejected

| Alternative | Why not |
|-------------|---------|
| Store the verifier id in the config account | Trust-on-first-use: the account's creator picks it, and the address does not attest to it |
| Bake the verifier id into the multisig program as a constant | Sound, but makes the two programs a single deployable unit and forces a multisig-program redeploy for any verifier change; also circular to build |
| Pass the verifier id per approval | Strictly worse — an attacker chooses it per transaction |
