# Phase 0 status — Bootstrap

**Date:** 2026-09-04
**Plan contract:** `docs/plan/planlp0002.md` §5 Phase 0
**Result:** **3 of 5 SC green; SC0.1 and SC0.4 blocked on a permission gate — Phase 0 is NOT complete.**

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
| **SC0.1** | Public `main` exists | ⛔ **blocked** | `gh repo create … --public --push` was refused by the environment's permission classifier. Local `main` exists with 2 commits (`f6a1a15`, `5ef6b93`); nothing is published. Needs operator approval — see below. |
| **SC0.2** | `LICENSE-MIT` + `LICENSE-APACHE` present (**H7**) | ✅ green | Both non-empty, committed in the **first** commit `f6a1a15` per plan §0.4.2 |
| **SC0.3** | `cargo fmt` + `clippy -D warnings` green | ✅ green | Both exit 0, table above |
| **SC0.4** | CI green on push to `main` | ⛔ **blocked** | Depends on SC0.1 — no remote to push to, so GitHub Actions has never run. The workflow is written and its jobs pass locally (fmt/clippy/test/shellcheck/clobber all exit 0), but "CI green" cannot be claimed without an actual run. |
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

## Blocker: publishing the repository

`gh repo create lp-0002-private-multisig --public --source=. --push` was **denied by the sandbox's
permission classifier**. This was not retried or worked around.

Both remaining SC depend on it: SC0.1 *is* publication, and SC0.4 needs a real GitHub Actions run.

The operator must either approve the repository-creation command, or create the repo manually and
add the remote:

```bash
gh repo create lp-0002-private-multisig --public --source=. --remote=origin --push
# or, if the repo already exists:
git remote add origin git@github.com:<user>/lp-0002-private-multisig.git
git push -u origin main
```

**Note on ordering:** publishing is a prize *submission requirement* ("public repository with all
circuit code…"), but it is independent of the #105 eligibility question — a public open-source repo is
not a submission. Phase I remains blocked regardless.

## Exit

**Not taken.** Phase 0 stays open until SC0.1 and SC0.4 are green. Phase A design work does not depend
on publication and can proceed in parallel once the operator decides — but Phase 0 is not marked
complete until CI has actually run green on `main`.
