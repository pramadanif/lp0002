# Tracking

- Plan source: ../lambda-prize/planlp0002.md (v5.1)
- Prize source: ../lambda-prize/prizes/LP-0002.md
- Sibling layout: intentional — easy human eval side-by-side
- Solution repo absolute path: /Users/muhammadbaguspramadani/Documents/myproject/lp-0002-private-multisig
- Current phase: **E (demo.sh + CI e2e)** — IN PROGRESS. The lifecycle *is* implemented end to end (create → propose → approve at full M → execute); what is missing is a completed unattended run.
- Last green SC: SC-D.1–SC-D.5 (Phase D). Phase E: SC-E.3/E.5/E.7 green; E.1/E.4/E.6 not met
- Blockers: **Phase E** — the `e2e-sequencer` CI job is wired (every push to `main`, not cron, not path-filtered) but has not yet completed. Each run has failed further along than the last: missing guest toolchain → missing `libpcsclite` → a SIGPIPE panic in our own script (`docs/tried-failed.md`). Every failure so far has been a real defect, and the job fails rather than faking a pass. The build itself is large: a full blockchain node plus a C++ groth16 stack, ~12 min before anything else starts. (#105 eligibility: operator decided 2026-09-04 to proceed through Phase I — `docs/phase-N1-status.md` §3.)

- Remote: https://github.com/pramadanif/lp0002 (public)

## Pins (settled in Phase −1, evidence-backed)

| What | Pin | Why |
|------|-----|-----|
| LEZ | **v0.2.4** | Live testnet ImageIDs match v0.2.4 artifacts exactly; v0.2.0 does not (`artifacts/phase-N1-testnet-version-fingerprint.txt`) |
| SPEL | **`main` @ `5126b7ed8a9b`** | Released v0.6.0 pins LEZ v0.2.0 → wrong private account ids on this testnet. `main` pins v0.2.4. Unreleased → disclose in `docs/limitations.md` |
| Rust (host) | 1.94.0 | LEZ v0.2.4 `rust-toolchain.toml` |
| risc0 | 3.0.5 (`r0vm`, `cargo-risczero`); guest rust 1.97.0 | LEZ v0.2.4 `Cargo.toml` |
| Testnet RPC / explorer | `https://testnet.lez.logos.co` / `https://explorer.testnet.lez.logos.co` | Cited in merged upstream `solutions/LP-0005.md`; both probed live |

## Abort watch

| Date (UTC) | `gh pr view 125 … reviewDecision` | Merged LP-0002 PR? | Action |
|------------|-----------------------------------|--------------------|--------|
| 2026-09-04 | *(empty — not APPROVED)* | none | continue |

## Human gates outstanding

| Gate | Needed by | State |
|------|-----------|-------|
| ~~Funded LEZ testnet keys~~ | — | **NOT A HUMAN GATE.** LEZ ships a proof-of-work faucet (Piñata). `./scripts/fund-testnet.sh` obtains funds unattended — verified: balance 150 → 300 on the public testnet |
| Narrated video URL + transcript | Phase H (W5/H11) | not yet requested |
| Basecamp click-QA (if automation fails) | Phase F | not yet needed |

## Carried forward

| Item | Needed by | Note |
|------|-----------|------|
| ~~Reproducible guest build~~ | — | **DONE 2026-09-05.** `build-guests.sh --docker` now works and `artifacts/IMAGE_IDS.md` records `reproducible (cargo risczero build, container r0.1.91.1)`. It had never run: the docker branch passed `--bin`, which `cargo risczero build` does not accept. Two further defects behind it — the pinned container's guest rustc is 1.91 against a workspace MSRV of 1.94, and the ELF was picked with `head -1` from a directory holding the raw ELF, an already-wrapped `.bin` and a copy under `deps/` |
| Second-proof slowdown undiagnosed | Phase H (BUGS_FILED) | `docs/tried-failed.md`; does not affect the recorded 53.26 s |
| ~~PPE composition not demonstrated~~ | — | **DONE.** tx `f2458791…198fbcb5` confirmed; ≈19 min, peak 8.74 GB (`artifacts/phase-E-ppe-approve-SUCCESS.txt`) |
| ~~U-6~~ | — | **RESOLVED.** It can; `--bin-<NAME>` resolves the ChainedCall dependency |
| ~~Recursive composition cost unmeasured~~ | — | **MEASURED.** ≈19 min 26 s, peak 8.74 GB, needs ~9 GB free RAM |
| Verifier change rotates every `config_hash` | Phase H (limitations) | Consequence of ADR-002 |
| Funded testnet wallet no longer on this machine | Phase G | It lived under `.e2e/`, gitignored and cleaned when a leaked key was purged. `fund-testnet.sh` creates a wallet when absent and the faucet is proof-of-work, so this is recoverable unattended — but the recorded balance of 300 is not an asset in hand |
| **Both** programs must be redeployed before any evidence is pinned | Phase G | `artifacts/phase-E-*.txt` records ImageIDs `821c23d9…` (membership) and `cee07cd3…` (multisig). Neither matches `artifacts/IMAGE_IDS.md` today (`f5cc9f37…`, `94bc1426…`). On LEZ the ImageID *is* the ProgramId, so **every** recorded on-chain result is for a superseded binary, and `config_hash` — which commits to the membership program id (ADR-002) — changes with it, moving every multisig address. An earlier version of this row said the membership ImageID was unchanged; that was true only of the INV-7 rebuild on 2026-09-05, not of the Phase E evidence, and reading it as "the membership evidence still stands" would have been wrong |

## Phase ledger

Commit column: the commit that **added** each status document. Two entries here named commits that
do not exist (`8e2f0b1`, `4c1a8f2`) — history was rewritten by a `git filter-branch` that purged a
leaked key, and the recorded hashes were never updated. A reviewer checking them would have found
nothing. Corrected 2026-09-05; every hash below now resolves.

| Phase | Status | Status doc | Commit |
|-------|--------|-----------|--------|
| −1 | ✅ complete | `docs/phase-N1-status.md` | `f6a1a15` |
| 0  | ✅ complete | `docs/phase-0-status.md` | `5ef6b93` / `b14f49e` |
| A  | ✅ complete | `docs/phase-A-status.md` | `ec9534a` |
| B  | ✅ complete | `docs/phase-B-status.md` | `f8b6e5f` |
| C  | ✅ complete | `docs/phase-C-status.md` | `aebb278` |
| D  | ✅ complete | `docs/phase-D-status.md` | `10276c1` |
| E  | ◐ in progress (5/8 green, 2 in progress) | `docs/phase-E-status.md` | `7026ecf` |
| F  | ◐ in progress (2/7 SC) | `docs/phase-F-status.md` | `de77b2e` |
| G  | not started | — | — |
| H  | not started | — | — |
| I  | not started | — | — |
