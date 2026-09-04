# Tracking

- Plan source: ../lambda-prize/planlp0002.md (v5.1)
- Prize source: ../lambda-prize/prizes/LP-0002.md
- Sibling layout: intentional — easy human eval side-by-side
- Solution repo absolute path: /Users/muhammadbaguspramadani/Documents/myproject/lp-0002-private-multisig
- Current phase: **E (demo.sh + CI e2e)** — IN PROGRESS, 3/8 SC
- Last green SC: SC-D.1–SC-D.5 (Phase D). Phase E: SC-E.3/E.5/E.7 green; E.1/E.4/E.6 not met
- Blockers: **Phase E** — the standalone LEZ sequencer build is very large (full blockchain node + C++ groth16 stack); the e2e lifecycle step is unimplemented and fails rather than faking a pass. (#105 eligibility: operator decided 2026-09-04 to proceed through Phase I — `docs/phase-N1-status.md` §3.)

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

## Carried forward

| Item | Needed by | Note |
|------|-----------|------|
| Reproducible guest build (`build-guests.sh --docker`) | Phase G | `artifacts/IMAGE_IDS.md` currently records a local, non-reproducible build and says so |
| Second-proof slowdown undiagnosed | Phase H (BUGS_FILED) | `docs/tried-failed.md`; does not affect the recorded 53.26 s |
| **PPE composition not yet demonstrated end-to-end** | Phase E | Still open. Needs a running sequencer, both programs deployed, and U-6 resolved |
| U-6: can SPEL's CLI submit the private path? | Phase E | Unresolved. If not, the client must build the PPE transaction itself |
| Recursive composition cost unmeasured | Phase E | `env::verify` needs succinct receipts; the 53 s figure is a composite proof and does not cover it |
| Verifier change rotates every `config_hash` | Phase H (limitations) | Consequence of ADR-002 |

## Phase ledger

| Phase | Status | Status doc | Commit |
|-------|--------|-----------|--------|
| −1 | ✅ complete | `docs/phase-N1-status.md` | (this commit) |
| 0  | ✅ complete | `docs/phase-0-status.md` | `5ef6b93` / `b14f49e` |
| A  | ✅ complete | `docs/phase-A-status.md` | `ec9534a` |
| B  | ✅ complete | `docs/phase-B-status.md` | `f8b6e5f` |
| C  | ✅ complete | `docs/phase-C-status.md` | `8e2f0b1` |
| D  | ✅ complete | `docs/phase-D-status.md` | `4c1a8f2` |
| E  | ◐ in progress (3/8 SC) | `docs/phase-E-status.md` | — |
| F  | not started | — | — |
| G  | not started | — | — |
| H  | not started | — | — |
| I  | not started | — | — |
