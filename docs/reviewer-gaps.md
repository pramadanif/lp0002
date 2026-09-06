# What the reviewer actually rejects, and where we stand

Every LP-0002 submission so far has been closed. This is a reading of **all nine** rejections and
the three accepted submissions elsewhere in the programme, taken from the reviewers' own words —
not from guesswork about what they might want.

Purpose: make sure we close the things that actually kill submissions, and **not** spend effort on
things nobody was ever rejected for.

---

## 1. Every rejection, by reviewer comment

| PR | Author | Reviewer's stated reason |
|----|--------|--------------------------|
| #68 | Tranquil-Flow | localnet not testnet · no CU · `demo.sh` not the real DEV_MODE=0 path · no e2e-vs-sequencer in CI |
| #87 | retraca | no video walkthrough |
| #91 | jeefxM | no CU · no partial-approval resume · no e2e-vs-sequencer in CI · README incomplete · Basecamp assets not separately downloadable · **binding is derivation-only, not in-circuit live-account** |
| #92 | Tranquil-Flow | transactions **not visible on the explorer** · CI does not actually run a LEZ sequencer |
| #97 | jeefxM | `demo.sh` not really DEV_MODE=0 — **hardcoded in an internal script** · CI not green on default branch · **no verifiable transactions** |
| #102 | nizarsyahmi37 | no narrated video |
| #115 | duongja | CI does not test anything using LEZ · **CU file literally says there is no CU** |
| #120 | FidelCoder | CI must be green **and run LEZ sequencer tests** · testnet evidence missing · no CU |
| #131 | Tranquil-Flow | **the execute transaction contains no proof** · **no multisig-related transactions** · CU recorded as "unavailable" |

## 2. Frequency — what actually kills a submission

| Rejection cause | Times cited |
|-----------------|-------------|
| **CI not running a real LEZ sequencer e2e, or not green** | **6** |
| **Testnet evidence missing, or explorer links dead** | **5** |
| **CU cost missing or "unavailable"** | **5** |
| `demo.sh` not genuinely `RISC0_DEV_MODE=0` | 2 |
| No narrated video | 2 |
| Derivation-only binding | 1 |
| No partial-approval resume | 1 |
| Basecamp assets not downloadable | 1 |
| Execute transaction carries no proof | 1 |
| No multisig-domain transactions | 1 |
| README incomplete | 1 |

**Three causes account for nearly every rejection.** Everything else is a long tail.

## 3. The winning pattern

Three submissions were accepted: #64 (LP-0005), #80 (LP-0016), #100 (LP-0017).

- **#80 and #100 were awarded with no criticism at all.** No back-and-forth, no follow-up list.
- **#64 was awarded after exactly two fixes**, both mechanical:
  1. *"can you please re-submit transactions as they are not available anymore"* — the explorer
     links had **expired between submission and review**;
  2. *"make sure CI passes"*, then *"CI should be fixed, can you update your fork"*.

The lesson from #64 is sharper than it looks: the submission was already good enough on substance.
What nearly held it up was **evidence decaying** and **a red CI**. Neither is about cryptography.

So the winning shape is: **live transactions at the moment a human looks, green CI, CU numbers
present, and a video.** Substance beyond that was never the deciding factor for anyone.

## 4. Where we stand, honestly

| Killer | Our status | Note |
|--------|-----------|------|
| CI runs real LEZ sequencer e2e, green | ◐ **wired, not yet green** | `e2e-sequencer` runs on every push to `main`, not on cron and not path-filtered (the shape #125 was pulled up for). It has not completed a run, so nothing is claimed. The earlier reasoning here — that the job should stay absent because a failing job is not evidence — was backwards: the job is what tells you whether the script passes, and keeping it out hid three real defects. This is still our **largest** gap and matches the **#1** rejection cause |
| Testnet evidence, live explorer links | ⛔ **nothing deployed** | Funding is solved (`fund-testnet.sh`); deployment is not |
| CU cost | ◐ **partial** | Client proving measured (602,662 cycles; 116 s; composed ≈19 min). On-chain per-instruction figures need testnet |
| `demo.sh` genuinely DEV_MODE=0 | ◐ | Set at the entrypoint, never in a child. `check-dev-mode-clobber.sh` enforces exactly the #97 failure, and is mutation-tested. But the script has not completed an unattended run |
| Narrated video | ⛔ | Human gate. Shot list ready |
| Derivation-only binding (#91) | ✅ | In-circuit live-account binding, **mutation-tested**: removing the assertion fails 3 tests |
| Partial-approval resume (#91) | ✅ | 9 store tests; each CLI command is its own process, so the restart is real |
| Basecamp downloadable (#91) | ⛔ | Module generated and hardened; needs Qt6 + `lgx` to package |
| Execute carries no proof (#131) | ✅ | Different situation — see §5 |
| Multisig-domain transactions (#131) | ◐ | create/propose/approve exist **locally**; not yet on testnet |
| README complete | ◐ | Honest about status; missing deploy steps and Basecamp walkthrough |

## 5. The one rejection that needs an explicit answer

#131 was closed partly for *"the execute transaction contains no proof"*. Our `execute` is **also** a
public transaction with no proof, so this must be answered head-on rather than hoped past.

**Why ours is not the same failure.** In #131 there was no proof anywhere on the multisig path — the
same review also says *"there are no multisig related transactions"*. In this design the proof is at
**approve** time, and it is real: each approval is a privacy-preserving transaction whose validity
depends on LEZ's circuit verifying both chained programs. By the time `execute` runs, the threshold
is a fact already recorded on chain as a set of distinct, proof-backed nullifiers.

`execute` then does one thing: read that already-verified state and move funds if `count >= M`. It
takes no secret input and asserts nothing that was not already proven, so there is nothing for a
proof to say.

**Making it public is deliberate, and better for privacy.** Anyone may execute a proposal that has
reached its threshold — including a non-member. If execution required a member, the executor would
be a member, and *that* would link a member to the proposal. A permissionless public execute is what
makes criterion **P-F4** (execution unlinkable to any individual member) achievable at all.

This is stated in the solution write-up so a reviewer does not have to infer it.

## 6. What we should NOT do

Guarding against over-fixing, since effort here is scarce:

- **Do not add a proof to `execute`.** It would prove nothing new, cost ~20 minutes per execution,
  and make P-F4 *worse* by tying execution to a member.
- **Do not build more tests for things nobody was rejected for.** We have 102. The gaps are
  deployment and CI, not coverage.
- **Do not chase Basecamp polish.** One submission was dinged for assets not being downloadable —
  the bar is a package that exists and loads, not a beautiful UI.
- **Do not redesign anything.** Derivation-only (#91) and dev-mode clobber (#97) are already closed
  and mutation-tested. Re-opening them would be motion, not progress.

## 7. The order that follows from the evidence

1. **Deploy to testnet** — closes the #2 and #3 killers, and most of #10.
2. **Get `demo.sh` through one clean run, then wire the CI e2e job** — closes the #1 killer.
3. **Record the video** — closes a cause that killed two submissions outright.
4. **Package the `.lgx`** — one citation, lowest priority of the four.

Re-verify explorer links on the day the PR is opened. #64 shows a reviewer hitting dead links on an
otherwise-winning submission, and #92 was closed for it. `scripts/check-explorer-links.sh` exists for
exactly that, and treats a page that loads but reports no such transaction as dead.
