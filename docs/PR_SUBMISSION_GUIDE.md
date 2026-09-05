# PR submission guide — LP-0002

Everything needed to open the prize PR, in one file, so nobody has to re-read the repository to
write it. Written to be handed to a person or an agent cold.

**Read this first, then `docs/SOLUTION_DRAFT.md`** — that file is the PR body. This one is the
procedure around it.

---

## 0. The one thing that decides whether to open the PR at all

Run this, and believe it over any summary including this document:

```bash
./scripts/preflight-submission.sh
```

**PENDING counts as failure.** The rule is not "mostly green": a submission opens when every check
says PASS. The script prints the pin commit and exits non-zero otherwise.

If it is not all-PASS, §3 says what each remaining check needs.

---

## 1. What this submission is

A private M-of-N multisig for LEZ. Members hold shielded accounts; each approval is a zero-knowledge
proof of membership plus a nullifier, so the chain can count that M distinct members approved
**without learning which**. The approve path is privacy-preserving execution (PPE): the multisig
program emits a `ChainedCall` to a membership program, and LEZ's own circuit proves that call ran
via `env::verify`.

- Prize: [`docs/plan/LP-0002.md`](plan/LP-0002.md) — $1,200, deadline **2026-09-11**
- Target repo: `logos-co/lambda-prize`, PR against `main`
- Solution repo: <https://github.com/pramadanif/lp0002> (public, dual MIT/Apache-2.0)
- The competing open PR is **#125** (edenbd1) — open, not approved. See §6.

---

## 2. Opening the PR

1. **Re-run the two things testnet expiry invalidates.** The testnet is wiped periodically, so
   evidence goes stale on its own:
   ```bash
   ./scripts/verify-onchain.sh          # reads the chain, not our notes
   ./scripts/check-explorer-links.sh    # every explorer link in DEPLOYMENT.md still resolves
   ```
   This is preflight **PF-15**, and it is the check most likely to be skipped and most likely to
   embarrass. Do it on the day the PR is opened, not the day before.

2. **Pin the commit.** Preflight prints it. Every link in the PR body must point at that commit,
   not at `main` — `main` moves, and a reviewer clicking a moved link finds a 404. #125 was pulled
   up for exactly that (`limitations.md` 404 at its pinned commit).

3. **Body** = `docs/SOLUTION_DRAFT.md`, with the video URL filled in.

4. **Title**: `LP-0002: Private M-of-N Multisig for LEZ`

---

## 3. What each remaining preflight check needs

| Check | What it wants | How to get it |
|-------|---------------|---------------|
| PF-08 | Numeric per-instruction on-chain CU costs | `./scripts/measure-cu.sh` after deployment. Never write "unavailable" — the gate rejects placeholders, and it is right to |
| PF-09 | `docs/DEPLOYMENT.md` | Written by `./scripts/deploy-testnet.sh` |
| PF-10 | Something actually deployed and verifiable | `./scripts/deploy-testnet.sh`, then `verify-onchain.sh` |
| PF-12 | Narrated video URL in `SOLUTION_DRAFT.md` | §4 |

All four collapse into **one working session**: deploy to testnet while recording. Doing the deploy
without recording means proving it all again for the video.

---

## 4. The video — read before recording

The prize text is specific, and this is where a good submission is thrown away.

> **L58:** the recording must show terminal output **including proof generation** to confirm
> `RISC0_DEV_MODE=0` was active.
> **L90:** the builder narrates what they built and why, walks through the architecture and key
> decisions, and demonstrates M-of-N approval **and execution** with shielded accounts. *A silent
> screencast is not sufficient.*

The script is [`docs/video-transcript.md`](video-transcript.md). Two things it exists to stop you
getting wrong:

- **Free ~9 GB of RAM first.** The composed approval proof peaks at 8.74 GB. The first attempt at
  this failed for *hours* — not for lack of a machine, but because a browser and two editors held
  ~9 GB and the prover went to swap. With them closed it took 19 minutes.
- **Nothing in the prize text or the evaluation policy says a recording may be edited.** Do not
  assume it. Compress the wait as a *labelled time-lapse* with the wall clock visible through it,
  keep the start and the finish at real speed, and commit the raw log so the elapsed time can be
  checked rather than trusted. Full M needs **two** approvals — record both; do not reuse a clip.

Budget ~45–60 minutes of proving for the session (2 × ~19 min plus the rest).

---

## 5. Claims that must not be overstated

The repository is deliberately explicit about what is *not* done. Keep it that way in the PR body —
every claim below is one a reviewer can check in minutes, and the value of the honest ones is lost
if one dishonest one is found next to them.

- **Proving time is two numbers, not one.** ~53 s is the standalone membership proof. **~19 min** is
  the composed approval, which is what a member actually pays. Quoting only 53 s is misleading.
- **The SDK cannot submit anything.** It prepares and proves; submission is the SPEL CLI's job. The
  receipt from `prove_approval` is not what the chain accepts.
- **`demo-fast.sh` is not the demo.** It runs no prover and proves nothing, and says so in its own
  header. The prize demo is `./demo.sh`.
- **The CLI is a local state file**, not a chain. Every command prints `[local]`.
- **One machine, one sample.** Timing and memory figures are not a benchmark suite.

`docs/limitations.md` is the canonical list. If a reviewer finds something missing from it, that is
worse than the thing itself.

---

## 6. Where #125 lost points, and what we did instead

Useful for the PR body's framing — but state these as *our* properties, never as criticism of
another submission.

| #125 gap | Ours |
|----------|------|
| `demo.sh` was an in-process executor tour; missing tools skipped to `exit 0` | `demo.sh` drives a **real standalone sequencer**; a missing prerequisite exits non-zero (H1/H2) |
| e2e ran on cron and behind path filters | `e2e-sequencer` runs on **every push to `main`**, not path-filtered (H4) |
| Treasury evidence used a *lowered* threshold | Primary evidence is **full M** |
| `limitations.md` 404 at the pinned commit | Present and non-empty at the pin, and every link resolves (PF-12/PF-13 territory) |
| `config_hash` formula drifted between README and code | **One identical formula string** in README, ADR-001 and SOLUTION_DRAFT, asserted by PF-13 |

#125 is light on the machine precisely because it skips the expensive part. Our ~19 min per approval
is the cost of actually doing it — that is the difference, and it is worth saying plainly.

---

## 7. Repository facts, so nothing has to be re-derived

**Canonical formulas** — byte-identical in README, ADR-001 and SOLUTION_DRAFT, and a gate enforces
that:

```
config_hash = SHA256( DS_CONFIG ‖ member_root[32] ‖ M[1] ‖ N[1] ‖ multisig_id[32] ‖ membership_program_id[32] )
nf_approve  = SHA256( DS_NF ‖ nsk ‖ multisig_id ‖ proposal_id )
```

**Error codes.** On-chain errors are raised as `1001`–`1013` but SPEL reports a custom program error
as `6000 + code`, so a client sees **`7001`–`7013`**. Match the bracketed number:
`Program error [7002]: Program error 1002: DuplicateNullifier`. Client-side codes are seven, not a contiguous range: `2001`–`2004`, `2006`, `2007`, `2010`. The gaps
are retired codes that were documented but never implemented — do not quote a range. Full catalogue:
[`docs/error-codes.md`](error-codes.md).

**Invariants** are INV-1 … INV-7 in ADR-001. INV-7 is the newest and the one worth knowing: `execute`
pays the account the proposal named, out of the multisig's own PDA — there is no caller-supplied
treasury or recipient to redirect.

**Version pins**, each established by measurement rather than assumption
([`docs/VERSIONS.md`](VERSIONS.md)):

| | |
|---|---|
| LEZ | `v0.2.4` |
| SPEL | `main` @ `5126b7ed8a9b` — **not** the v0.6.0 release, which pins LEZ v0.2.0 and derives different private account ids |
| risc0 / r0vm | `3.0.6` |
| Rust | `1.94.0` (guest toolchain `1.97.0`) |

**ImageIDs live in `artifacts/IMAGE_IDS.md`** and are checked against the committed binaries by
`crates/sdk/tests/image_ids_match_binaries.rs`. On LEZ a `ProgramId` **is** the ImageID, so changing
guest code changes every address derived from it — after any guest change, redeploy.

**Gates worth knowing about before touching anything:**

| Script | Refuses |
|--------|---------|
| `check-guests-fresh.sh` | a committed guest binary older than its sources |
| `check-dev-mode-clobber.sh` | any submission-path script forcing `RISC0_DEV_MODE=1` |
| `check-basecamp-privacy.sh` | the Basecamp UI persisting the approval witness |
| `preflight-submission.sh` | the submission itself, PF-01 … PF-15 |

---

## 8. Standing rules

These are constraints on how the work is done, not suggestions.

- **Never claim a command passed without running it.** Record the exit code and the log path. Most
  of the real bugs in this repository were found because a claim was checked instead of trusted.
- **Never fake `RISC0_DEV_MODE=0`.** A dev-mode receipt verifies and proves nothing.
- **Never invent a funded key or wallet.** If one is needed, ask.
- **No alt accounts, no rebranding another submission.**
- **A gate that cannot do its work must fail, not pass.** Several checks in this repo once reported
  green on no evidence — the video gate passed because the document said *"No narrated video"* — so
  when adding a check, mutate it in both directions and confirm it actually fails.
