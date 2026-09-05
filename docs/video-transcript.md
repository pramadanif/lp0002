# Demo video — shot list and transcript

**Status: NOT RECORDED.** This is the plan for the recording, written so the session is a matter of
reading and running rather than improvising. The transcript proper replaces the "say" lines once the
video exists.

The prize is explicit that **a silent screencast is not sufficient** — the builder must narrate what
they built and why, walk through the architecture, and demonstrate M-of-N approval and execution
using shielded accounts. It also requires terminal output showing proof generation, confirming
`RISC0_DEV_MODE=0`.

---

## Before recording

1. `./scripts/build-guests.sh --docker` — the binary on screen must be the reproducible one.
2. Run `./demo.sh` once to warm every build. **Record the second run**: same evidence, far shorter,
   and nothing about it is less true.
3. **Close Chrome and one editor.** The composed proof needs ~9 GB free; on a loaded machine it
   swaps and appears to hang. This is the single most likely way for a recording to go wrong.
4. Terminal at a legible font size. A reviewer has to read `RISC0_DEV_MODE=0`.

## Shot list

### 1. Identity and pin (~30 s)

Show `git log -1` and `git status` on screen.

> Say: "This is the private M-of-N multisig for LEZ, at commit `<sha>`, clean tree. Everything you
> are about to see runs from this commit."

**Must be on camera:** the commit hash. It has to match the commit the submission pins.

### 2. The problem (~60 s) — slide or README

> Say: "A public multisig on LEZ can't work with shielded accounts. Its members must be fresh
> zero-nonce keypairs claimed by the program. A shielded account is owned by the privacy protocol,
> and its nonce isn't a counter you can hold at zero — LEZ derives it from the account id and then
> re-derives it from the member's secret on every use. So membership can't be an ownership relation.
> It has to be proven in zero knowledge."

### 3. Architecture (~2 min) — ADR-001

Show the `config_hash` line.

> Say: "Everything hangs off this. The member root, the threshold M, and the membership verifier's
> ImageID are all hashed into the seed of the multisig's address. So an attacker who lowers M, or
> swaps in a permissive verifier, doesn't get a weaker multisig — they get a different address, where
> nothing exists."

> Say: "An approval proves two things at once. LEZ's privacy-preserving circuit proves you control a
> live shielded account. Our membership guest proves that same account is in the member set, and
> emits a nullifier. The nullifier is keyed to your secret, so you can't vote twice from another of
> your own addresses."

### 4. The demo (~4 min edited)

Run `./demo.sh`. Narrate as it goes.

**Must be on camera:**
- the `RISC0_DEV_MODE=0` banner
- `sequencer is live — getLastBlockId = N` — a real node, not an in-process executor
- both program deployments
- **proof generation starting**, then the confirmation

> Say at the proof: "This is a real proof — dev mode is off, you can see it in the banner. It takes
> about twenty minutes and roughly nine gigabytes. I'm speeding up the wait, not cutting it; the
> clock stays on screen, and the full log is in the repo."

**Compress the wait as a labelled time-lapse. Do not cut the start or the finish.**

Nothing in the prize text or the evaluation policy says whether a recording may be edited — so do
not rely on it being allowed. What LP-0002 does require is that the recording *shows* proof
generation and confirms `RISC0_DEV_MODE=0` was active. A time-lapse satisfies that only if nothing
is hidden, so:

- keep a running wall clock (or the shell's own timestamps) visible **through** the sped-up section,
  so the ~19 minutes is verifiable on camera rather than asserted;
- put an on-screen label — e.g. `time-lapse ×60, no cuts` — over that section;
- show the `RISC0_DEV_MODE=0` banner and the start at real speed, and the receipt and confirmation
  at real speed;
- commit the unedited run log alongside it, so a reviewer who doubts the edit can check the
  timestamps against `artifacts/phase-E-ppe-approve-SUCCESS.txt` and the demo log.

The point is that a reviewer never has to *trust* the edit: the elapsed time is on screen and the
raw log is in the repo. Two approvals are needed for full M, so this happens twice — record both,
and do not reuse one clip for the other.

### 5. What the chain records (~90 s)

Run `./scripts/verify-onchain.sh`.

> Say: "This reads only public chain data — no secrets, no local state. It confirms the config
> account rehashes to its own address, the verifier is the deployed one, the threshold was met at
> full M, every nullifier is distinct, and the proposal executed."

Then show the decoded proposal account.

> Say: "Here's the whole record of who approved: a count, and two nullifiers. No account ids, no
> member list, nothing that identifies anyone. That's the point of the prize, and it's visible on
> chain rather than only in a test."

### 6. Honesty (~60 s) — limitations.md

> Say: "What this doesn't do. It's unaudited. The member set is fixed at creation. It doesn't defend
> against timing or network correlation. Proving needs about nine gigabytes free — the first time I
> ran it with a browser open it swapped for hours and looked broken. And here's what I got wrong
> along the way."

Show `tried-failed.md`.

> Say: "I shipped a version where the member's spending key was recoverable from the guest journal.
> My first test scanned for the raw bytes and said it was clean — that was a false negative, because
> risc0 word-encodes each byte. Decoding the journal showed the key sitting there. That's fixed, and
> the test now decodes instead of scanning."

**Do not skip this section.** A submission that only shows what works invites the reviewer to go
looking for what does not.

---

## Checklist before publishing

- [ ] Commit hash legible and matching the pinned commit
- [ ] `RISC0_DEV_MODE=0` legible
- [ ] Proof generation visibly starts and finishes
- [ ] Full lifecycle at **full M**, not a lowered threshold
- [ ] Narrated throughout — not a silent screencast
- [ ] Video URL added to `SOLUTION_DRAFT.md` (preflight PF-12 checks for it)
- [ ] This file replaced with the actual transcript
