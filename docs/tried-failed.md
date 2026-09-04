# Tried and failed

A running, honest log of approaches attempted and abandoned, and of things that surprised us. Kept
because the reasoning behind a rejected path is worth as much to a reviewer as the path taken — and
because a repo where nothing ever went wrong is not a believable repo.

Entries are appended as work happens. Design-time rejections that were never coded live in
ADR-001 §7; this file is for things actually attempted or discovered by measurement.

---

## Phase −1

### Pinning LEZ by "latest tag" would have been wrong — and quietly so

**Tried.** The obvious pin is the newest tag, or else the one a competitor used (#123 pinned v0.2.2).
The SPEL framework's released version, v0.6.0, pins LEZ **v0.2.0**, so "use released SPEL" looked like
the safe, conservative choice.

**Failed, because.** Address derivation is not stable across those versions. At v0.2.0,
`AccountId::for_regular_private_account` hashes `prefix ‖ npk ‖ identifier` (80 bytes). At v0.2.4 it
hashes `prefix ‖ npk ‖ vpk ‖ identifier`, where `ViewingPublicKey::LEN = 1184`. Building against
v0.2.0 derives shielded account addresses **the live testnet does not recognise** — and the failure
would surface late, as unexplained rejections during testnet deployment in Phase G.

**What we did instead.** Fingerprinted the deployed version rather than guessing. A LEZ `ProgramId`
is the risc0 ImageID of the program ELF, and LEZ commits prebuilt binaries under `artifacts/` at every
tag. Computing those ImageIDs and comparing with the live `getProgramIds`:

| Program | Live testnet | v0.2.0 | v0.2.4 |
|---------|--------------|--------|--------|
| `token` | `ccc4713e…` | `c5d50f88…` ✗ | `ccc4713e…` ✓ exact |
| `privacy_preserving_circuit` | `383e884f…` | `ab86d257…` ✗ | `383e884f…` ✓ exact |

So: pin LEZ **v0.2.4**, and pin SPEL to **`main`** (which tracks v0.2.4) rather than its released
v0.6.0. Depending on an unreleased SPEL commit is a real cost, and it is disclosed in
`docs/limitations.md` rather than hidden.

Evidence: `artifacts/phase-N1-testnet-version-fingerprint.txt`.

### `getBlockHeight` is not a LEZ RPC method

**Tried.** Probing the testnet with `getBlockHeight`, by analogy with other chains.

**Failed.** `{"error":{"code":-32601,"message":"Method not found"}}`. Reading
`sequencer/service/rpc/src/lib.rs` gives the actual set: `sendTransaction`, `checkHealth`, `getBlock`,
`getBlockRange`, `getLastBlockId`, `getAccountBalance`, `getTransaction`, `getAccountsNonces`,
`getProofForCommitment`, `getAccount`, `getProgramIds`. The height method is `getLastBlockId`.

Minor, but it is the whole reason this repo reads LEZ's source for every interface instead of
assuming shapes from other ecosystems.

## Phase 0

### `mapfile` is not available to macOS evaluators

**Tried.** `mapfile -t targets < <(…)` in `scripts/check-dev-mode-clobber.sh`.

**Failed.** macOS ships bash **3.2.57** as `/bin/bash`, and `mapfile` arrived in bash 4. The script
died with `mapfile: command not found` — and, worse, `set -u` then made it exit non-zero for the
*wrong reason*, which would have looked like a real H3 violation.

**What we did instead.** A `while IFS= read -r` loop. Every script in this repo is written for bash
3.2, because the prize says evaluators clone the repo and run the demo from a clean environment, and
some of those environments are Macs.

## Phase A

### The membership guest cannot take `nsk` on trust

**Nearly shipped.** The first sketch had the membership guest verify `npk ∈ member_root` and compute
the approval nullifier from the witness `nsk` — and stop there. It looks complete: membership proven,
nullifier bound to a secret.

**Why that is broken.** Nothing tied the `nsk` given to *our guest* to the `nsk` given to LEZ's
privacy-preserving circuit. A member could pass their real `nsk` to the PPE circuit — legitimately
spending their own account — while passing a different `nsk` to the membership guest on each attempt.
Every attempt yields a different `nf_approve`, and one member votes as many times as they like. The
double-vote defence would have been decorative.

**Fix.** The guest re-derives `AccountId::for_regular_private_account(npk, vpk, identifier)` from its
own witnesses and asserts it equals the approver's `pre_state.account_id` — the account the PPE
circuit independently bound to a live commitment. ADR-001 D4; the assertion is exactly what **SC-B.5**
requires a test to catch the removal of.
