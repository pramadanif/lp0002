# Phase 0 status — Bootstrap

**Date:** 2026-09-04
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase 0
**Result:** **all 5 SC green — Phase 0 complete.**

Abort check at phase start: `gh pr view 125 … reviewDecision` → empty (not APPROVED); merged LP-0002
PRs → 0. Not aborting.

## Commands run (all local, exit codes captured)

| Command | Exit |
|---------|------|
| `cargo fmt --all -- --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `cargo test --workspace --all-targets` | **0** (3 tests pass) |
| `shellcheck -S warning scripts/*.sh` | **0** |
| `./scripts/check-dev-mode-clobber.sh` | **0** |
| `./scripts/preflight-submission.sh` | **1** — correct for this phase (see below) |

## Success criteria

| SC | Requirement | State | Evidence |
|----|-------------|-------|----------|
| **SC0.1** | Public `main` exists | ✅ green | Pushed to <https://github.com/pramadanif/lp0002> (public). `git push -u origin main` → `* [new branch] main -> main` |
| **SC0.2** | `LICENSE-MIT` + `LICENSE-APACHE` present (**H7**) | ✅ green | Both non-empty, committed in the **first** commit `f6a1a15` per plan §0.4.2 |
| **SC0.3** | `cargo fmt` + `clippy -D warnings` green | ✅ green | Both exit 0, table above |
| **SC0.4** | CI green on push to `main` | ✅ green | GitHub Actions run [33868559625](https://github.com/pramadanif/lp0002/actions/runs/33868559625) on push to `main`, `completed/success` in 37s. Jobs: `fmt + clippy + tests` ✅, `shellcheck` ✅, `RISC0_DEV_MODE clobber check (H3)` ✅ |
| **SC0.5** | `docs/VERSIONS.md` committed | ✅ green | `git ls-files docs/VERSIONS.md` → tracked since `f6a1a15` |

## What was built

- **Cargo workspace** — `pmsig-core` (shared host+guest types), `pmsig-sdk` (client proving/tx building),
  `pmsig-store` (partial-approval persistence, P-R2), `pmsig-cli` (`pmsig` binary).
  Deliberately skeletal: plan says "do not implement circuits yet".
  Lint policy denies `unwrap`/`expect`/`panic`/`indexing_slicing` in library code, since a panic in a
  guest aborts the proof.
- **`rust-toolchain.toml`** pinned to 1.94.0, matching LEZ v0.2.4.
- **`scripts/preflight-submission.sh`** — all fifteen PF-01…PF-15 checks from plan §6 are wired now,
  not stubbed. Checks whose evidence a later phase produces report `PENDING`, and **PENDING exits 1
  exactly like FAIL**, so the script can never call a submission ready while work is outstanding.
  Current tally: `pass=3 fail=0 pending=12`.
- **`scripts/check-dev-mode-clobber.sh`** — H3 guard against the #97 reject pattern (a nested script
  hardcoding `RISC0_DEV_MODE=1` under a demo that advertises real proving). Written for bash 3.2 so it
  runs on a macOS evaluator's default shell, not just CI's bash 5.
- **`.github/workflows/ci.yml`** — `quality`, `dev-mode-clobber`, `shellcheck`. No `continue-on-error`
  and no conditional skips anywhere (H2). The jobs a later phase adds (`e2e-sequencer`, `explorer-links`,
  `preflight`) are named in a trailing comment so their absence stays visible.
- **`PRIZE_CHECKLIST.md`** — all 21 criteria with empty evidence columns.
- **`README.md`** — states the phase plainly: no `demo.sh`, nothing deployed, nothing claimed.

## Publication

Published to <https://github.com/pramadanif/lp0002> (public, created by the operator).

The earlier `gh repo create --public --push` attempt was refused by the sandbox permission classifier
and was **not** worked around; the operator created the repository and the remote was wired to the
existing history rather than re-initialising it. All three commits made before publication are intact:
`f6a1a15` (phase −1), `5ef6b93` (phase 0), `b14f49e` (phase 0 status).

## Exit

All five SC green → **proceed to Phase A**.

CI note: the workflow deliberately does **not** run `preflight-submission.sh` yet. Preflight exits 1
until the submission packet is real, so wiring it now would make `main` permanently red and destroy
the signal SC0.4 is meant to carry. It is added in Phase H, when it can pass.
