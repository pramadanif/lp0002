# Why Logos, and why not a centralised multisig

Outline for the solution write-up (SC-A.8). Argument first; prose comes in Phase H.

---

## 1. The problem a public multisig cannot solve

A multisig that publishes its member list and approval history is a surveillance surface. It reveals
voting patterns over time, exposes members to targeted pressure once their votes are attributable, and
permanently links identities to governance decisions.

For treasuries and protocol governance this is not a cosmetic concern. Knowing *which* two of three
signers move a treasury tells an adversary exactly whom to compromise, subpoena or coerce — and the
history tells them who is likely to sign next.

## 2. Why not a centralised or off-chain multisig

- **Trusted coordinator.** Anything that aggregates approvals off-chain learns who approved. That is
  the exact fact we are protecting; moving it off-chain relocates the leak, it does not close it.
- **Unverifiable.** An off-chain tally cannot be audited by anyone who was not shown the books.
- **Threshold signatures don't fix it either.** FROST-style signing needs an interactive round among
  participants, so the co-signers learn who took part. The prize requires privacy from *other members*,
  not merely from observers — which rules the whole family out (ADR-001 §7).

## 3. Why Logos specifically

Logos supplies the two primitives this needs, already in the base layer:

1. **Shielded accounts with unlinkable use.** LEZ private accounts carry an nsk-derived nonce inside
   their commitment, so successive uses of the same account are unlinkable. The anonymity set is the
   whole live commitment set, not something we had to bootstrap.
2. **Privacy-preserving execution.** LEZ verifies proofs of local execution instead of re-executing.
   Contract logic can therefore depend on secrets that never reach the chain — this scheme is not
   possible on a chain where validators must see the inputs.

The result composes: LEZ's PPE circuit proves *you control a live shielded account*, our membership
guest proves *that account belongs to this multisig*, and the chain records only that a threshold was
reached. No component has to be trusted with the member's identity, because no component ever sees it.

## 4. What this unlocks

Private DAO treasuries; governance where a vote cannot be traced to a voter; any shared-custody
arrangement whose *membership* is as sensitive as its funds. Each is a primitive that a public
multisig cannot provide at all — not one it provides less conveniently.

## 5. Honest framing

This is a working primitive with real testnet evidence, not a finished product. It is unaudited, the
member set is fixed at creation, and it does not defend against timing or network-level correlation.
`docs/limitations.md` is the complete list, and the write-up links to it rather than talking around it.
