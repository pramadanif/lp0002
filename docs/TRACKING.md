# Tracking

- Plan source: ../lambda-prize/planlp0002.md (v5.1)
- Prize source: ../lambda-prize/prizes/LP-0002.md
- Sibling layout: intentional — easy human eval side-by-side
- Solution repo absolute path: /Users/muhammadbaguspramadani/Documents/myproject/lp-0002-private-multisig
- Current phase: **0 (Bootstrap)** — publishing to https://github.com/pramadanif/lp0002
- Last green SC: SC-N1.1, SC-N1.2, SC-N1.3 (Phase −1)
- Blockers: none. (#105 eligibility: operator decided 2026-09-04 to proceed through Phase I — `docs/phase-N1-status.md` §3.)

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
| Funded LEZ testnet keys (M members + treasury) | Phase G | not yet requested |
| Narrated video URL + transcript | Phase H (W5/H11) | not yet requested |
| Basecamp click-QA (if automation fails) | Phase F | not yet needed |

## Phase ledger

| Phase | Status | Status doc | Commit |
|-------|--------|-----------|--------|
| −1 | ✅ complete | `docs/phase-N1-status.md` | (this commit) |
| 0  | ◐ 3/5 SC green — blocked on publish | `docs/phase-0-status.md` | `5ef6b93` |
| A  | not started | — | — |
| B  | not started | — | — |
| C  | not started | — | — |
| D  | not started | — | — |
| E  | not started | — | — |
| F  | not started | — | — |
| G  | not started | — | — |
| H  | not started | — | — |
| I  | not started | — | — |
