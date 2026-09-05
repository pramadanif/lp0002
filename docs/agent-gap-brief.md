# Gap brief — 2026-09-05

Written against commit `534b571`. Every row below was checked with a command, not recalled. Where the
brief that prompted this disagreed with the repository, the repository was re-checked and the
disagreement is noted rather than silently resolved either way.

## Where this actually stands

Phases −1 through D are complete; **E and F are in progress; G, H and I have not started.** The
code is done and audited: 124 tests, `fmt`/`clippy`/`shellcheck` clean, every gate script
mutation-tested in both directions, and the four instructions all exercised through the risc0
executor. What is missing is **evidence, not implementation** — no completed unattended demo run,
nothing deployed to a public testnet, no `.lgx`, no video. Preflight is `pass=13 fail=0 pending=4`
and therefore still exits non-zero, which is correct.

**Abort check:** #125 is `OPEN` with an empty `reviewDecision`; no LP-0002 solution PR is merged.
Not aborting.

## Critical gaps

| ID | Gap | State | Evidence |
|----|-----|-------|----------|
| C1 | Unattended `./demo.sh` → 0 | **FAIL** | Never completed. Lifecycle is implemented end to end; `docs/phase-E-status.md` SC-E.1 |
| C2 | `e2e-sequencer` green on `main` | **FAIL** | Job wired on every push, not path-filtered. Four runs, none complete; run 4 in flight at 72 min. SC-E.4 |
| C3 | Public testnet evidence | **FAIL** | Nothing deployed. `docs/DEPLOYMENT.md` absent; preflight PF-09/PF-10 PENDING |
| C4 | Full M + execute demonstrated | **PARTIAL** | Scripts use full M and never a lowered tier (H13). One approval is on chain; the second and `execute` are not |
| C5 | On-chain CU table | **FAIL** | `docs/cu-costs.md` has client proving only. Needs deployment; PF-08 PENDING |
| C6 | Narrated video | **FAIL** | Human gate. Shot list ready in `docs/video-transcript.md` |
| C7 | Basecamp `.lgx` | **FAIL** | `app/` generated (9 files); `qmake6`, `cmake`, `lgx` all absent — checked. Phase F 2/7 SC |

## High

| ID | Gap | State | Note |
|----|-----|-------|------|
| H-A | Doc drift | **FIXED this session** | The brief was right and I had missed it. README's banner said the demo was not written; criteria-checklist said P-U2 had no `app/` and P-S2 was not wired; reviewer-gaps said the job was deliberately absent. All corrected |
| H-B | ImageID drift | **CONFIRMED — worse than stated** | Phase E evidence records `821c23d9…`/`cee07cd3…`; IMAGE_IDS.md holds `f5cc9f37…`/`94bc1426…`. **Both** differ, so `config_hash` moves too and every multisig address with it. TRACKING's claim that membership was unchanged was true of the INV-7 rebuild and misleading about the Phase E evidence; corrected |
| H-C | `explorer-links` hollow | **PARTIAL** | Job now wired. With no `DEPLOYMENT.md` it reports "nothing to check — NOT evidence of anything" and exits 0; it has never reported a pass. Must fail on dead txs once the file exists |
| H-D | risc0 pin drift | **FIXED this session** | VERSIONS.md gave 3.0.5 — LEZ v0.2.4's pin, not ours. We use 3.0.6. Both now stated, with why the difference is checkable rather than assumed |
| H-E | Guests non-reproducible | **FIXED** | `build-guests.sh --docker` works and IMAGE_IDS.md records a reproducible build, so `deploy-testnet.sh` no longer refuses. It had never run at all — three defects, see the commit. ImageIDs moved again: membership `f5cc9f37…`→`960db4f2…`, multisig `94bc1426…`→`cb3bcc5e…` |

## Where I disagree with the brief

- **"C2: many runs cancelled — stop cancel storms."** Fixed before this brief arrived, and the
  diagnosis differs: the cancels came from a workflow-level `concurrency` with
  `cancel-in-progress`, which I had added myself. A job-level group does not protect against it,
  because cancelling a run cancels its jobs. Workflow-level concurrency is gone.
- **"M4: preflight must stay exit 1 until done."** It does, and not by choice — PENDING counts as
  failure by construction, so it cannot be talked into passing.

## Order for this session

1. ~~Doc sync (H-A, H-D)~~ — done.
2. **C2 then C1**: let run 4 finish; fix whatever it finds; repeat until green. This is the gate
   everything else waits behind, and each failure so far has been a real defect.
3. **H-E then H-B**: reproducible `--docker` guest build, refresh IMAGE_IDS, then redeploy both
   programs. Nothing may be pinned as evidence before this.
4. **C3/C4/C5** (Phase G) and **C6** (video) in one recorded session — doing the deploy without
   recording means proving it all again.
5. **C7** needs an operator decision: install Qt6 and `lgx`, or accept P-U2 unmet and say so.
