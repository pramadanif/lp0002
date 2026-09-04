# LP-0002 — Autonomous Build Plan (v5.1 — agent-executable + research lock)

**Prize source of truth:** [`prizes/LP-0002.md`](prizes/LP-0002.md)  
**Status:** Open · **$1,200** · Large · **Deadline 2026-09-11 23:59 CEST**  
**Plan version:** 5.1 (v5 agent clarity + restored riset: historis H, rival detail, peluang %, reject matrix)  
**Work mode:** Agent executes phases **−1 → I in order**. Never skip Success Criteria (SC).  
**Hard stop before PR:** Phase H green + video URL live + `preflight-submission.sh` exit **0**.

---

## 0. AGENT START HERE (read once, obey always)

### 0.1 Mission (one sentence)

Build **original** private M-of-N multisig for LEZ: shielded members, no public who-voted, threshold-only on-chain, nullifiers, Risc0, SPEL program, SDK, Basecamp, testnet evidence, narrated video — then open **one** clean solution PR that beats reject patterns and closes #125 gaps.

### 0.2 Reading order (do not shuffle)

1. This §0 (rules + glossary)  
2. §1 Prize map (P-* ↔ exact prize bullets)  
3. §2 Locked architecture  
4. §3 Hard gates H1–H15 + Win bars W1–W17  
5. Current phase contract (§5) only — finish SC before next phase  
6. §6 Preflight spec before Phase I  
7. Appendix only if stuck (history / rivals / sprint)

### 0.3 ID glossary (one namespace = one meaning)

| Prefix | Meaning | Fail = |
|--------|---------|--------|
| **P-F / P-U / P-R / P-P / P-S** | Official prize Success Criteria (must all be true for win) | Criteria fail |
| **H1–H15** | Hard gates from historical reject + #125 gaps — treat as compile errors | Submission unsafe |
| **W1–W17** | Win bars (reviewer ergonomics + beat #125) — required before Phase I | Weak vs rivals |
| **X1–X9** | Anti-reject shorthand (subset of H/W) | Same as related H |
| **P1–P15** | Winner patterns (canon from merged prizes) — design guidance | Soft; covered by H/W/SC |
| **SC-*** | Phase Success Criteria — measurable exit for that phase | Stay in phase |

**Conflict rule:** If prize text (`prizes/LP-0002.md`) conflicts with this plan → **prize wins**. If H conflicts with convenience → **H wins**. If W conflicts with H → satisfy **both**.

### 0.4 Inviolable rules (agent)

1. **Original work only.** Do not clone/rebrand repos of PR #123 / #125 / #133. Allowed: prize text, public LEZ/SPEL/Risc0 docs, reject-comment *patterns* (not their code). `jimmy-claw/lez-multisig` = **why private is hard** only — never ship public-multisig PoC as this prize solution.
2. **Dual license** MIT **and** Apache-2.0 from first commit (`LICENSE-MIT` + `LICENSE-APACHE`).
3. **Phase order.** Red SC → stay. Green SC → write `docs/phase-<id>-status.md` → commit → next phase.
4. **Canonical demo:** `./demo.sh` = real **standalone** LEZ sequencer E2E + `RISC0_DEV_MODE=0`. Optional `./demo-fast.sh` = executor tour only — **never** cite as prize demo.
5. **No fake green:** missing `r0vm`/sequencer/tools on demo or CI e2e → **exit ≠ 0**. No `continue-on-error` / skip→0 on those paths.
6. **Escalate to human only:** (a) funded testnet keys, (b) narrated video recording, (c) Basecamp click-QA if automation fails.
7. **Abort build features** if either true:
   - `gh pr view 125 --repo logos-co/lambda-prize --json reviewDecision -q .reviewDecision` == `APPROVED`
   - Any LP-0002 solution PR **merged** upstream  
   Then: document only; no new feature work.
8. **Max 3 submissions / 1 review per week.** First PR must be preflight-0. Do not burn slot on half-ready.
9. **Pin one commit** for SOLUTION. Re-verify explorer **day of PR open** (testnet wipe).
10. **Honesty:** No plan guarantees win. FCFS = first submission eng accepts as meeting **all** criteria. This plan maximizes pass-criteria + beat-#125 gaps + speed.

### 0.5 Definition of done (whole prize)

All true:

- Every **P-*** checkbox green (§1) with evidence path  
- Every **H1–H15** green (§3)  
- Every **W1–W17** green (§3) — W5 needs human video URL  
- `scripts/preflight-submission.sh` exit 0  
- Solution PR open on `logos-co/lambda-prize` with reviewer packet (§5 Phase I)

### 0.6 Human gates (agent must pause and ask)

| Gate | When | Agent action |
|------|------|--------------|
| Testnet funded key(s) | Before Phase G deploy | Ask human; do not invent keys |
| Narrated video | Phase H W5 | Deliver shot list; wait for URL + transcript |
| Abort decision | #125 APPROVED | Stop features; report |

---

## 1. Prize map (exact text → ID → where proven)

Source: `prizes/LP-0002.md` Success Criteria. Agent marks evidence in `docs/criteria-checklist.md`.

### 1.1 Functionality

| ID | Prize requirement (verbatim intent) | Prove in |
|----|-------------------------------------|----------|
| **P-F1** | Shielded member can approve without revealing identity to **on-chain observers or other members** | A security.md, D peer flow, C state has no voter list, W8 |
| **P-F2** | On-chain confirms threshold M met **without** recording which members | C state layout, G explorer |
| **P-F3** | Cannot approve same proposal twice (nullifiers or equivalent) | B/C double-nf tests |
| **P-F4** | Completed execution unlinkable to any individual shielded account | A/C design + G evidence |
| **P-F5** | Proof generation client-side on standard laptop | B/E demo prove |
| **P-F6** | Reference integration: threshold-gated action (e.g. treasury transfer) on LEZ testnet with shielded members | G |
| **P-F7** | ≥1 multisig on testnet: create + propose + approve-to-threshold + execute; reproducible + evidence | G DEPLOYMENT.md |
| **P-F8** | Full docs + clean public repo | H |

### 1.2 Usability

| ID | Requirement | Prove in |
|----|-------------|----------|
| **P-U1** | Module/SDK for Logos modules | D |
| **P-U2** | Basecamp GUI: local build, downloadable assets, loadable | F |
| **P-U3** | SPEL IDL for LEZ program | C |

### 1.3 Reliability

| ID | Requirement | Prove in |
|----|-------------|----------|
| **P-R1** | Proof failures → clear error to member | D |
| **P-R2** | Partial approvals (&lt;M) preserved + resumable across restarts | D |
| **P-R3** | Deterministic documented error codes (invalid proof, double-vote, …) | A error-codes + C tests |

### 1.4 Performance

| ID | Requirement | Prove in |
|----|-------------|----------|
| **P-P1** | CU cost of each on-chain op documented (numeric) | G `docs/cu-costs.md` |

### 1.5 Supportability

| ID | Requirement | Prove in |
|----|-------------|----------|
| **P-S1** | Deployed + tested on LEZ devnet/testnet | G |
| **P-S2** | E2E vs LEZ sequencer **standalone** in CI | E |
| **P-S3** | CI green on default branch | E |
| **P-S4** | README: deploy, addresses, CLI + Basecamp steps | H |
| **P-S5** | Reproducible `demo.sh` vs **real local sequencer**, `RISC0_DEV_MODE=0` | E (**H1**) |
| **P-S6** | Narrated video; terminal shows proof gen + DEV_MODE=0 | H (**W5**) |

### 1.6 Scope lock

**In scope:** ZK membership + nullifier (Risc0); LEZ verifier; SDK/CLI; testnet reference; docs (scheme, nullifier, nonce/`program_owner`, assumptions, limits, integration).

**Out of scope (do not build):** public-only multisig; hiding proposal *content*; polished consumer UI; long-term maint/audit.

**Submission extras (also required):** public repo dual-license OK; verified program ID; narrated architecture video; write-up; prove-time + CU benchmarks.

---

## 2. Locked architecture (change only with ADR)

| Decision | Choice | Why (agent must not “simplify” away) |
|----------|--------|--------------------------------------|
| Tx path | **Privacy-preserving only** for approve (+ execute if private composition needed) | Public path = host re-exec, no real proof composition (reject class #131) |
| Membership guest | LEZ-native program emitting `ProgramOutput` + chained `env::verify` | Standalone Risc0 journal ≠ LEZ ProgramOutput → reject |
| Anchor | PDA seeds include `config_hash = H(member_root ‖ M ‖ extra)` | Attacker invents root or lowers M → wrong PDA |
| Binding | In-circuit proof vs **live** shielded account commitment format (byte-compatible LEZ) | Derivation-only = reject (#91) |
| Nullifier | `nf = H(domain ‖ member_secret ‖ proposal_id ‖ multisig_id)` stored on-chain set | Double-vote deterministic |
| Peer privacy | Client does **not** publish approver identity to other members; shared = on-chain count/nullifiers | P-F1 “other members” |
| Reference action | Treasury transfer, default **2-of-3** (M=2, N=3) | Prize reference |
| Persist | Local store for partial approvals | P-R2 |
| Evidence | Public verify scripts + binary hash + explorer CI | Beat #125 review friction |

**Files Phase A must create:**

- `docs/adr/ADR-001-architecture.md`  
- `docs/lez-account-model.md` (nonce + `program_owner`)  
- `docs/security.md` (observers + other members + unlinkable definition)  
- `docs/error-codes.md` (≥8 codes)  
- Formulas for nullifier + `config_hash` copy-pasteable into code

**If architecture changes later:** write `ADR-002` before coding the change.

---

## 3. Hard gates + Win bars

### 3.1 Hard gates H1–H15 (zero exception)

| ID | Gate | Historis (riset) | Absolute rule in our repo |
|----|------|------------------|---------------------------|
| **H1** | Demo = standalone sequencer | #68 mock; #89 demo gagal; #125 demo=executor | `demo.sh` boots/calls real standalone LEZ sequencer + full lifecycle + `RISC0_DEV_MODE=0`. May thin-wrap `scripts/e2e-local-sequencer.sh`. **Not** in-process executor-only (`demo-fast.sh` only). **Stricter than LP-0005** (0005 allowed demo=testnet + e2e-local split; we require `demo.sh` itself standalone). |
| **H2** | No skip→exit 0 | #98 S2 skip=fail; #125 skip→0 risk | Missing tools on demo/CI e2e → non-zero exit. Skip only in non-submission helpers. |
| **H3** | No DEV_MODE clobber | #97 nested `=1` | Nested scripts must not hardcode `RISC0_DEV_MODE=1` on demo path. CI runs `check-dev-mode-clobber.sh`. |
| **H4** | E2E on push | #91 no CI e2e; #98 skip; #125 cron/path | Job `e2e-sequencer` on `push` to `main` for program/script/crate paths. **Not** cron-only. No path-filter that skips program changes. |
| **H5** | Live multisig-domain txs | #131/#97 explorer/localnet | create / propose / approve×M / execute on testnet + verify + explorer CI |
| **H6** | CU numeric | #91/#120/#131 “unavailable” | `docs/cu-costs.md` — no “unavailable” |
| **H7** | Dual license | #133 MIT-only risk | MIT **and** Apache-2.0 files present |
| **H8** | In-circuit live binding | #91 derivation-only | Not derivation-only (SC-B.5) |
| **H9** | Auth + PPE | #131 no proof on execute; #133 auth gap | Approve via PPE/chained verify; execute auth sound (markers owned by verifier) |
| **H10** | Basecamp `.lgx` | #91/#98 U2; #123 `module.json` warn | Downloadable release + SHA256 + `module.json` |
| **H11** | Narrated video | #98 S6/#89/#102/#87 | Commit on screen + mode 0 + prove visible |
| **H12** | limitations.md | celah #125 404 | File exists at pin; SOLUTION links resolve (no 404) |
| **H13** | Full M evidence | celah #125 tier 2-of-3 shortcut | Primary demo/testnet transfer uses **M full approvals**. Tiers OK as *extra*, not sole evidence path. |
| **H14** | config_hash one formula | celah #125 README drift | Identical string/formula in README + ADR + SOLUTION |
| **H15** | Preflight | slot burn / half PR | `preflight-submission.sh` exit 0 before Phase I; asserts H1–H14 |

### 3.2 Win bars W1–W17

| ID | Measurable done-when |
|----|----------------------|
| **W1** | `scripts/verify-onchain.sh` exit 0 on clean machine using **public RPC only** (bytecode hash, PPE path, no fake receipt, PDA/marker OK) |
| **W2** | CI job `explorer-links` fails on Evidence URL 404 / null result |
| **W3** | CI runs **deployed** guest/program bytes through sequencer executor (happy + reject) |
| **W4** | Negatives in CI: wrong root, lowered M (PDA miss), double nullifier, stale proposal, public-path approve → deterministic codes |
| **W5** | Video: commit hash + `RISC0_DEV_MODE=0` + prove start/end + full lifecycle; `docs/video-transcript.md` committed |
| **W6** | SOLUTION pins **one** commit; if “same as CI”, `git diff <ci-sha> <pin> --stat` only docs/video |
| **W7** | `docs/limitations.md` honest (no overclaim) |
| **W8** | Written test or demo note: approve flow does not require revealing identity to co-members |
| **W9** | PR body = tx table + one verify command + CI URLs (review &lt;15 min) |
| **W10** | `docs/criteria-checklist.md` — every P-* → evidence path |
| **W11** | SOLUTION Summary = precise on-chain claim (PPE? receipt? PDA binds?) |
| **W12** | `.lgx` SHA-256 + byte size in SOLUTION/Release |
| **W13** | Beat #125: demo standalone; missing tools fail (= H1/H2) |
| **W14** | Beat #125: e2e on every relevant push to main (= H4) |
| **W15** | Beat #125: full-M approvals on primary treasury evidence (= H13) |
| **W16** | Beat #125: limitations.md exists at pin (= H12) |
| **W17** | Beat #125: single config_hash formula everywhere (= H14) |

**Phase H incomplete until W1–W17 checked** (W5 blocked only until human sets video URL).

### 3.3 Anti-reject overlay X* (quick map)

| X | Means | Covered by |
|---|--------|------------|
| X1 Explorer live | H5, W2 | G |
| X2 Multisig txs + proof path | H5, H9 | C/E/G |
| X3 CI sequencer no skip | H2, H4 | E |
| X4 No DEV_MODE clobber | H3 | E |
| X5 CU numeric | H6 | G |
| X6 Partial resume | P-R2 | D |
| X7 In-circuit binding | H8 | A/B |
| X8 Basecamp downloadable | H10 | F |
| X9 Narrated video | H11, W5 | H |

### 3.4 Beat #125 (mandatory deltas)

| Their gap | Our rule |
|-----------|----------|
| demo = executor; skip→0 | demo = standalone; missing tool → fail |
| e2e cron / path-only | e2e on push to `main` |
| transfer via lowered tier vs full M | primary evidence = full M |
| `limitations.md` 404 | file at pin |
| `config_hash` doc drift | one formula everywhere |

---

## 4. Target repo layout

Create **new public repo** (name suggestion: `lp-0002-private-multisig`). Not inside `lambda-prize` tree except final `solutions/LP-0002.md` PR.

```text
lp-0002-private-multisig/
  LICENSE-MIT
  LICENSE-APACHE
  README.md
  demo.sh                          # H1 canonical
  demo-fast.sh                     # optional; NOT prize demo
  Cargo.toml                       # workspace
  programs/multisig-spel/
  programs/membership-lez/         # LEZ-native callee for chained verify
  methods/                         # if separate risc0 methods
  crates/sdk/
  crates/cli/
  crates/client-store/
  app/                             # Basecamp
  scripts/
    e2e-local-sequencer.sh
    deploy-testnet.sh
    measure-cu.sh
    verify-onchain.sh              # W1
    check-explorer-links.sh        # W2
    check-dev-mode-clobber.sh      # H3
    preflight-submission.sh        # H15 — see §6
  docs/
    VERSIONS.md
    phase-*-status.md
    adr/ADR-001-architecture.md
    lez-account-model.md
    security.md
    error-codes.md
    limitations.md                 # H12
    criteria-checklist.md          # W10
    cu-costs.md
    DEPLOYMENT.md
    tried-failed.md
    BUGS_FILED.md
    video-transcript.md
    SOLUTION_DRAFT.md
  artifacts/                       # IDL, ImageIDs, binary hashes
  .github/workflows/ci.yml         # unit + e2e-sequencer + explorer-links + clobber
```

---

## 5. Phase contracts

**Format every phase:** Goal → Inputs → Agent does → Do not → Artifacts → Success Criteria → Exit.

After each green phase: write `docs/phase-<id>-status.md` with: date, commands run, log paths, SC checklist (all `[x]`), next phase id.

---

### Phase −1 — Competitor + env preflight

**Goal:** Know abort state; pin toolchain versions; prove machine can build.

**Inputs:** `gh` auth; network; this plan.

**Agent does:**
1. `gh pr view 123 125 133 --repo logos-co/lambda-prize --json number,author,reviewDecision,updatedAt,state`
2. Confirm prize still Open in `logos-co/lambda-prize` README / `prizes/LP-0002.md`
3. Probe LEZ testnet RPC + explorer alive; note versions into draft `docs/VERSIONS.md`
4. Confirm Rust + Risc0 toolchain install commands work on builder machine
5. Document abort rule in `docs/phase-N1-status.md`

**Do not:** Start feature code. Open solution PR.

**Artifacts:** `docs/VERSIONS.md` (draft), `docs/phase-N1-status.md`

**Success Criteria:**
- [ ] **SC-N1.1** Abort condition written; if #125 already `APPROVED` → **stop** (do not continue)
- [ ] **SC-N1.2** `docs/VERSIONS.md` draft has LEZ/SPEL/Risc0/testnet RPC/explorer URLs
- [ ] **SC-N1.3** Toolchain bootstrap command succeeds (recorded in status)

**Exit:** all SC-N1 green → Phase 0

---

### Phase 0 — Bootstrap

**Goal:** Empty-but-strict public skeleton + CI skeleton.

**Agent does:** Create public repo; dual licenses; Cargo workspace stub; CI fmt/clippy/test; `PRIZE_CHECKLIST.md` mirror of §1; stub `scripts/preflight-submission.sh` that exits 1 with “not ready”.

**Do not:** Implement circuits yet.

**Artifacts:** repo `main` public; CI workflow; licenses; VERSIONS.md present.

**Success Criteria:**
- [ ] **SC0.1** Public `main` exists
- [ ] **SC0.2** `LICENSE-MIT` + `LICENSE-APACHE` present (**H7**)
- [ ] **SC0.3** `cargo fmt` + `clippy -D warnings` green
- [ ] **SC0.4** CI green on push to `main`
- [ ] **SC0.5** `docs/VERSIONS.md` committed

**Exit:** → Phase A

---

### Phase A — Design (no skip)

**Goal:** Spec so clear that Phase B implements without guessing formulas.

**Agent does:** Write ADR-001 + account model + security + ≥8 error codes + nullifier/`config_hash` formulas + privacy surface table + `docs/tried-failed.md` stub + “Why Logos” outline for SOLUTION.

**Do not:** Skip ADR. Do not leave “TBD” in formulas.

**Success Criteria:**
- [ ] **SC-A.1** ADR locks: PPE path + PDA `config_hash` + in-circuit binding + LEZ-native callee
- [ ] **SC-A.2** Nonce + `program_owner` explained with LEZ citation
- [ ] **SC-A.3** ≥8 error codes in `docs/error-codes.md`
- [ ] **SC-A.4** Nullifier + `config_hash` formulas copy-pasteable (no ambiguity)
- [ ] **SC-A.5** Explicit table: other members learn X / do not learn Y (**P-F1**)
- [ ] **SC-A.6** Attack “prover lowers M” → PDA fail — written as invariant
- [ ] **SC-A.7** Alternatives rejected listed; `docs/tried-failed.md` stub exists
- [ ] **SC-A.8** “Why Logos / why not centralized multisig” outline ready for SOLUTION

**Exit:** → Phase B

---

### Phase B — Guests / membership proof

**Goal:** Real membership+nullifier proof with live binding; ImageID pinned.

**Agent does:** LEZ-compatible commitment; guest; SDK prove API; negatives; one `RISC0_DEV_MODE=0` prove; record ImageID + prove seconds into `docs/cu-costs.md` (client section).

**Do not:** Ship derivation-only stub as “done”. Do not leave DEV_MODE=1 as default prove path.

**Success Criteria:**
- [ ] **SC-B.1** Honest verify pass
- [ ] **SC-B.2** Negatives fail: double nf / wrong root / wrong proposal / wrong commitment format
- [ ] **SC-B.3** DEV_MODE=0 prove recorded (seconds) in cu-costs client section (**P-F5**)
- [ ] **SC-B.4** Journal has no npk / member id plaintext
- [ ] **SC-B.5** Test **fails** if binding replaced by derivation-only stub (proves H8 real)
- [ ] **SC-B.6** `artifacts/IMAGE_IDS.md` pins ImageID(s)
- [ ] **SC-B.7** Commitment/account-id derivation byte-compatible with LEZ; regression vs known vector if available

**Exit:** → Phase C

---

### Phase C — SPEL multisig + membership program

**Goal:** On-chain lifecycle with PPE approve and sound execute.

**Semantics (fixed — implement exactly):**

| Ix | Behavior |
|----|----------|
| `create_multisig` | PDA(`multisig_id`, `config_hash`) |
| `create_proposal` | Public action OK (content not hidden) |
| `approve` | Chained verify; nullifier set; count++ |
| `execute` | count≥M; treasury transfer; executed flag |

**Success Criteria:**
- [ ] **SC-C.1** IDL published (**P-U3**)
- [ ] **SC-C.2** Lifecycle test pass (create→propose→approve×M→execute)
- [ ] **SC-C.3** Double approve → documented error code (**P-F3**, **P-R3**)
- [ ] **SC-C.4** Early execute → documented error code
- [ ] **SC-C.5** Invalid proof → documented error code
- [ ] **SC-C.6** State layout: **no** voter identity list (**P-F2**)
- [ ] **SC-C.7** Valid proof but wrong `config_hash`/M → PDA/ownership fail
- [ ] **SC-C.8** Doc+test: approve on public tx path unsupported/rejected (**H9**)

**Exit:** → Phase D

---

### Phase D — SDK / CLI / resume / peer privacy

**Goal:** Member UX without leaking identity to co-members; resume works.

**Success Criteria:**
- [ ] **SC-D.1** CLI full lifecycle local (**P-U1**)
- [ ] **SC-D.2** Kill client mid-threshold → resume still reaches M (**P-R2**)
- [ ] **SC-D.3** Prove failure → clear error (**P-R1**)
- [ ] **SC-D.4** Integration guide builds/compiles
- [ ] **SC-D.5** Co-member can approve knowing only multisig/proposal ids + on-chain count — **not** first member account id (**W8**, **P-F1**)

**Exit:** → Phase E

---

### Phase E — demo.sh + CI (beat #125)

**Goal:** Literal prize demo + CI that cannot soft-skip.

**Agent does:**
1. Implement `demo.sh` per **H1/H2/H3/H13**
2. Optional `demo-fast.sh` (not cited as prize demo)
3. CI jobs: `unit`, verifier-executor, **`e2e-sequencer` on push to `main`** (H4/W14), clobber check, later explorer-links (may stub until G)
4. SOLUTION_DRAFT cites `demo.sh` only as prize demo

**Do not:** Make `demo.sh` executor-only. Do not `exit 0` when sequencer missing.

**Success Criteria:**
- [ ] **SC-E.1** Fresh clone `./demo.sh` → 0 with standalone sequencer (**H1/W13**, **P-S5**)
- [ ] **SC-E.2** Log shows `RISC0_DEV_MODE=0` + real prove + sequencer RPC
- [ ] **SC-E.3** `check-dev-mode-clobber.sh` → 0 (**H3**)
- [ ] **SC-E.4** CI `main` green including **push-gated** e2e-sequencer (**H4/W14**, **P-S2/S3**)
- [ ] **SC-E.5** No skip / continue-on-error on demo/e2e (**H2**)
- [ ] **SC-E.6** W3 deployed-bytes tests present
- [ ] **SC-E.7** Missing `r0vm` → `demo.sh` **fails** (prove H2)
- [ ] **SC-E.8** Docs cite `demo.sh` as prize demo; `demo-fast.sh` = non-criteria

**Exit:** → Phase F

---

### Phase F — Basecamp + Release

**Goal:** Loadable GUI + downloadable `.lgx` with hashes.

**Success Criteria:**
- [ ] **SC-F.1** README build/load instructions work (**P-U2**)
- [ ] **SC-F.2** Release assets downloadable (**H10**)
- [ ] **SC-F.3** SHA256SUMS match downloaded bytes
- [ ] **SC-F.4** `lgx verify` or ui-host READY proof recorded in docs
- [ ] **SC-F.5** `module.json` / metadata present (no validator warn)
- [ ] **SC-F.6** GUI shows threshold progress **without** other members’ account ids
- [ ] **SC-F.7** Release notes: `.lgx` SHA-256 + byte size (**W12**)

**Exit:** → Phase G

---

### Phase G — Testnet + CU + public verify

**Goal:** Live evidence + automatable public verify.

**Preflight (block deploy until true):**
- [ ] Human provided funded accounts for M members + treasury path
- [ ] Same LEZ rev as `docs/VERSIONS.md`

**Agent does:** deploy; full lifecycle; `DEPLOYMENT.md` table; measure CU; implement `verify-onchain.sh` (W1); wire explorer-links CI (W2); supersede bad txs with strike-through + reason.

**Do not:** Submit with dead explorer links. Do not use tier-lowered M as sole evidence (**H13**).

**Success Criteria:**
- [ ] **SC-G.1** Program IDs published
- [ ] **SC-G.2** Explorer live: deploy + create + propose + ≥M approve + execute (**P-F6/F7**, **H5**)
- [ ] **SC-G.3** `verify-onchain.sh` → 0 (**W1**)
- [ ] **SC-G.4** CU numeric all ops (**H6**, **P-P1**)
- [ ] **SC-G.5** Txs are multisig-domain (not unrelated noise)
- [ ] **SC-G.6** Binary hash ↔ commit artifact
- [ ] **SC-G.7** `check-explorer-links.sh` in CI green (**W2**)
- [ ] **SC-G.8** Re-run verify ≥ once after ≥1h or schedule day-of-PR
- [ ] **SC-G.9** `DEPLOYMENT.md` numbered table; superseded txs struck-through + why
- [ ] **SC-G.10** Signer/anchorer account link published
- [ ] **SC-G.11** Primary treasury evidence uses **full M** approvals (**H13/W15**)
- [ ] **SC-G.12** Checklist item reserved: day-of-PR verify (execute in Phase I)

**Exit:** → Phase H

---

### Phase H — Docs + preflight + video

**Goal:** Submission packet ready; preflight 0; video done.

**Agent does:** stranger-proof README; `SOLUTION_DRAFT.md` = full prize mirror; FURPS; limitations; criteria-checklist; tried-failed; BUGS_FILED; video shot list → human records → transcript; implement full `preflight-submission.sh` per §6.

**Do not:** Open solution PR from this phase. Do not leave TBD in checklist.

**Success Criteria:**
- [ ] **SC-H.1** README alone enough for E2E (**P-S4**)
- [ ] **SC-H.2** Every P-* → evidence in criteria-checklist (**W10**, **P-F8**)
- [ ] **SC-H.3** FURPS / write-up complete (scheme, nullifier, account model, assumptions, limits, integration)
- [ ] **SC-H.4** Known limitations honest (**W7**)
- [ ] **SC-H.5** Video URL narrated + `docs/video-transcript.md` (**W5**, **H11**, **P-S6**)
- [ ] **SC-H.6** `preflight-submission.sh` exit 0 (**H15**)
- [ ] **SC-H.7** W1–W17 marked done (W5 after URL)
- [ ] **SC-H.8** Pin commit chosen; CI green or docs-only diff vs CI (**W6**)
- [ ] **SC-H.9** `docs/tried-failed.md` + Why Logos filled
- [ ] **SC-H.10** `docs/BUGS_FILED.md` — issues or explicit “none”
- [ ] **SC-H.11** `docs/criteria-checklist.md` complete
- [ ] **SC-H.12** SOLUTION Summary = precise claim paragraph (**W11**)
- [ ] **SC-H.13** `docs/limitations.md` exists; all links resolve (**H12/W16**)
- [ ] **SC-H.14** README + ADR + SOLUTION same `config_hash` formula (**H14/W17**)
- [ ] **SC-H.15** Preflight asserts H1–H14 mechanically (§6)

**Exit:** → Phase I **only if** SC-H.5 URL set and SC-H.6 = 0

---

### Phase I — Solution PR (one clean shot)

**Goal:** Open `Solution: LP-0002 — …` on `logos-co/lambda-prize`.

**Agent does:**
1. Day-of: re-run `verify-onchain.sh` + explorer-links (wipe catch) — **SC-G.12**
2. Abort check on #125 again
3. Fork / branch; write `solutions/LP-0002.md` from SOLUTION_DRAFT; pin commit
4. PR title: `Solution: LP-0002 — <short>`
5. PR body = reviewer packet (**W9**):

```markdown
## Pin
- Repo:
- Commit:
- CI runs:
- Video:

## One-command verify
./scripts/verify-onchain.sh

## Testnet table
| step | tx | explorer |

## Gates
W1–W17 + H1–H15 preflight green (attach preflight log)

## Competitors note
Snapshot date; prize still open; #125 reviewDecision=
```

6. Fix validate bot warnings immediately
7. Continue daily abort watch until merge or loss

**Do not:** Open if preflight ≠ 0. Do not open second PR same week.

**Success Criteria:**
- [ ] **SC-I.1** Validate ✅ (fix non-blocking warns if possible)
- [ ] **SC-I.2** All links at pin resolve
- [ ] **SC-I.3** T&C acknowledged per template
- [ ] **SC-I.4** Preflight was 0 at open time
- [ ] **SC-I.5** Daily abort watch documented until terminal state

**Exit:** Wait for eng review. On CHANGES_REQUESTED → fix items only (playbook Appendix C). Do not start unrelated features.

---

## 6. Preflight spec (`scripts/preflight-submission.sh`)

Agent **must** implement these checks. Exit **1** on any fail. Print `PASS/FAIL` per check.

| Check ID | Assert |
|----------|--------|
| PF-01 | `LICENSE-MIT` and `LICENSE-APACHE` exist |
| PF-02 | `demo.sh` exists and does **not** only call in-process executor; must reference standalone sequencer binary/service or `e2e-local-sequencer.sh` |
| PF-03 | `demo.sh` / e2e scripts: no `exit 0` after missing-tool skip; missing `r0vm` path fails |
| PF-04 | `rg -n 'RISC0_DEV_MODE=1' scripts demo.sh` → no matches on demo/e2e submission path (or allowlist empty) |
| PF-05 | `.github/workflows/ci.yml` has job name containing `e2e` with `on.push.branches` including `main` (not cron-only) |
| PF-06 | `docs/limitations.md` exists and non-empty |
| PF-07 | `docs/criteria-checklist.md` exists; every `P-F`/`P-U`/`P-R`/`P-P`/`P-S` id appears |
| PF-08 | `docs/cu-costs.md` has numeric CU (regex `[0-9]+`) per op; no `unavailable` |
| PF-09 | `docs/DEPLOYMENT.md` has ≥1 explorer URL; `check-explorer-links.sh` → 0 |
| PF-10 | `scripts/verify-onchain.sh` → 0 |
| PF-11 | `artifacts/IMAGE_IDS.md` non-empty |
| PF-12 | Video URL file or SOLUTION_DRAFT contains `http` video link; `docs/video-transcript.md` exists |
| PF-13 | `config_hash` formula string identical across README, ADR-001, SOLUTION_DRAFT (normalized compare) |
| PF-14 | Dual: SOLUTION cites `demo.sh` not `demo-fast.sh` as prize demo |
| PF-15 | Print git pin SHA; remind day-of verify |

Stub in Phase 0 may exit 1. Full impl required by Phase H.

---

## 7. Autonomous loop (copy this)

```text
LOOP phase in [-1, 0, A, B, C, D, E, F, G, H, I]:
  IF abort condition (§0.4.7): STOP features
  READ phase contract §5
  DO agent tasks
  RUN all SC for phase
  WRITE docs/phase-<id>-status.md
  IF any SC red: FIX; stay
  IF all SC green: COMMIT "phase <id> complete"; NEXT
BEFORE Phase I:
  RUN scripts/preflight-submission.sh
  MUST exit 0
```

---

## Appendix A — Winner patterns P1–P15 (guidance)

| Pola | Arti | Covered by |
|------|------|------------|
| P1 Evidence on-chain | Live explorer + verify scripts | W1, G |
| P2 PPE path | Real privacy-preserving composition | Arch, C |
| P3 Demo reproducible | Clean clone demo.sh mode 0 | E |
| P4 CI = proof | Standalone e2e green | E |
| P5 Video narrated | Arch + E2E + mode 0 | H |
| P6 Pin commit | One hash | I |
| P7 Checklist ↔ bukti | criteria-checklist | W10 |
| P8 FURPS + limitations | Honest limits | W7 |
| P9 Approach = decisions | ADR + tried-failed | A/H |
| P10 Reusable surface | SDK/IDL/lgx | D/F |
| P11 Dual license | MIT and Apache | H7 |
| P12 Supersede transparency | Struck-through old txs | G |
| P13 Reviewer packet | W9 PR body | I |
| P14 No stub critical path | Preflight | H |
| P15 Upstream honesty | BUGS_FILED | H |

---

## Appendix B — Rivals (operational + riset detail)

| Rival | Author | Residual risk (riset) | Our counter |
|-------|--------|----------------------|-------------|
| **#125** | edenbd1 | Strongest on paper (PPE+PDA+chained verify+verify-onchain+testnet); **not eng APPROVED**; only “CI green” ping; gaps: demo=executor + skip→0, e2e cron/path, tier may lower M, `limitations.md` 404 at pin, config_hash doc drift | Match PPE+PDA; **surpass** H1/H2/H4/H12/H13/H14 + W1/W2 packet |
| **#123** | FidelCoder | Validate ✅; warn **`module.json`**; pin LEZ **v0.2.2** may lag testnet; video CDN vs pin drift; waiting ~1 month no eng approve | Fresh testnet pin; video=pin; packaging+module.json; W2 |
| **#133** | jeefxM | 3rd try after #91/#97; discloses **Execute not auth-gated** risk; **MIT-only** risk; unreviewed fixes | Dual license; H8/H9; day-of explorer; no derivation-only |

**Win evaluation (all must be true):** P-* 100% · no H holes · W* not weaker than #125 · our PR first eng-accepted (FCFS).

**Kalibrasi peluang (chat riset — bukan jaminan):**  
- P(#125 merge **if** deep-reviewed) ~**70–80%** (demo≠standalone alone ≠ auto-reject; LP-0005 had demo/e2e split). Hard reject #125 ~**5–10%**.  
- If we execute plan exactly: P(pass criteria) ~**88–93%**.  
- P(win prize) ~**25–40%** if ship ≤~1 week before #125 awarded; →**0%** if #125 APPROVED first.  
Detail chase multi-prize: `planlpoo0023.md`.

---

## Appendix C — Pra-merge playbook (sumber PR)

| # | Action | Sumber historis |
|---|--------|-----------------|
| 1 | Day-of verify-onchain + explorer (wipe sering) | #64 re-submit txs |
| 2 | Self-audit binding/PDA **sebelum** juri; redeploy if under-bound | #64 self-fix mid-review |
| 3 | Validate + CI hijau; sync fork cepat kalau upstream CI pecah | #64 CI/checkout |
| 4 | Checklist **nol TBD**; video = pin commit | #56 CU/video TBD |
| 5 | CHANGES_REQUESTED → nutup item only; jangan PR setengah | #14 iterate |
| 6 | Minor UX nits → fix jam yang sama | #21 |
| 7 | Daily `gh pr view 125 … reviewDecision`; APPROVED → abort fitur | FCFS |
| 8 | PR packet = 1 verify cmd + tx table + CI (W9) | congrats-cepat #80/#100 |

---

## Appendix D — Sprint (aggressive wall-clock)

| Block | Phases | Wall (fast) |
|-------|--------|-------------|
| 0 | −1 + 0 + A | ≤1 h |
| 1 | B + C | 3–6 h |
| 2 | D + E | 2–4 h |
| 3 | F | 1–2 h |
| 4 | G | 1–3 h (+ funding wait) |
| 5 | H + video | 1 h + human |
| 6 | I | 0.5 h |

**Critical path:** B/C correctness → E CI standalone → G verify-onchain → H video → I.

**vs LP-0012:** 0002 is heavier (ZK+PPE+SPEL+Basecamp+testnet+video). Warm env: local E2E in ~1 aggressive day possible; testnet+video often &gt;1 day. Do not claim easier than 0012.

---

## Appendix E — Non-goals / “pemenang bukan”

- Clone/rebrand competitor repos (`jimmy-claw` public PoC ≠ solution)  
- Absolute win guarantee / “1000% masuk”  
- Silent screencast / video beda versi dari pin  
- Localnet-only saat criteria minta testnet  
- CU “unavailable” / derivation-only / mock proof  
- Half-ready PR that burns 1-of-3 weekly slot  

---

## Appendix F — Changelog

| Ver | Change |
|-----|--------|
| v2 | Win Bars, peer privacy, config_hash, public verify, explorer CI, preflight, FCFS abort, PR gate = H |
| v3 | Canon P1–P15; W10–W12; tried-failed / Why Logos / BUGS_FILED / criteria-checklist / supersede / lgx SHA |
| v4 | H1–H15; W13–W17 beat #125; demo standalone; CI push-gated; full-M; limitations wajib |
| v4+ | Pra-merge playbook; 1-day vs 0012 note |
| v5 | Agent-executable rewrite: §0/glossary; prize map; phase contracts; preflight PF-01…15 |
| **v5.1** | Restore riset: H historis PR refs; rival detail; peluang %; reject matrix Appendix G; pra-merge sumber PR; jimmy-claw rule |

---

## Appendix G — Research lock (temuan chat — jangan hilang)

### G.1 Sembilan reject-gap (wajib ketutup — sudah di H/SC)

| Gap | Historis | Plan cover |
|-----|----------|------------|
| In-circuit live-account, bukan derivation-only | #91 | H8, SC-B.5, Arch Binding |
| Execute/approve PPE, bukan public re-exec | #131 / bar #125 | H9, SC-C.8, Arch Tx path |
| Tx multisig lengkap di explorer | #131 | H5, SC-G.2 |
| CI real LEZ sequencer, no skip | hampir semua | H2/H4, SC-E.* |
| `demo.sh` DEV_MODE=0, no child hardcode | #97 | H3, SC-E.3 |
| CU angka nyata | #91/#120/#131 | H6, SC-G.4 |
| Partial-approval resume | #91 | SC-D.2, Arch Persist |
| Basecamp `.lgx` downloadable | #91 / warn #123 | H10, SC-F.* |
| Video narrated + commit on screen | #98/#89/#102 | H11, W5 |

### G.2 Reject matrix LP-0002 (reviewer publik)

| PR | Siapa | Alasan / pola |
|----|-------|---------------|
| #91 | jeefxM | CU hilang; no resume; no CI e2e; derivation-only; Basecamp assets |
| #97 | jeefxM | Nested DEV_MODE=1; CI merah; explorer txs |
| #133 | jeefxM | Resubmit; unreviewed; residual auth/MIT |
| #68/#92/#131 | Tranquil-Flow | Localnet; wipe; execute no proof; no multisig-domain txs; CU unavailable |
| #120 → #123 | FidelCoder | CI/sequencer/testnet/CU; module.json warn |
| #102/#87/#115 | lain | No video / no LEZ CI / CU “no CU” |

Full chase notes (0003/0008 too): **`planlpoo0023.md`**.

### G.3 Yang plan **jamin** vs **tidak**

**Jamin operasional** (kalau semua SC+H+W hijau): tidak mati di kelas reject di atas; bar ≥ #125 di evidence/privacy/verify; PR pertama = peluru terbaik.

**Tidak jamin:** FCFS kalau #125 di-APPROVED dulu; Logos sole discretion; testnet wipe mid-review; review latency.

### G.4 Sengaja dipindah / dipendekkan di v5 (bukan hilang substansi)

| Item v4 | Status v5.1 |
|---------|-------------|
| Section “Gaps di plan v1” | Diganti changelog + contracts (fix sudah di SC) |
| Master FURPS checkbox list | = §1 P-* map + criteria-checklist |
| Honesty ×2 duplikat | Digabung §0.4.10 + G.3 |
| Map P1–P15 panjang | Appendix A (pendek) + SC cover |
| Peluang % race | **Appendix B** (restored) |
| Historis kolom H | **§3.1** (restored) |
| Detail rival #123/#133 | **Appendix B** (restored) |
| Pra-merge sumber #64/#56… | **Appendix C** (restored) |

---

**START COMMAND FOR AGENT:** Begin **Phase −1**. Do not skip to coding. After SC-N1 green, continue Phase 0 → … → I per §5–§7.
