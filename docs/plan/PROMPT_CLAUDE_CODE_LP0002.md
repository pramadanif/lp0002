# Claude Code — LP-0002 Autonomous Build Prompt

**How to use:** Paste everything below the line into Claude Code (or `@` this file).  
**Layout:** plan hidup di `lambda-prize/`; solusi dibangun di folder **sibling** `../lp-0002-private-multisig/` (berdampingan, gampang track + evaluasi).

```text
myproject/
├── lambda-prize/                 ← plan, prize text, prompt (READ-ONLY for build)
│   ├── planlp0002.md
│   ├── planlpoo0023.md
│   ├── prizes/LP-0002.md
│   └── PROMPT_CLAUDE_CODE_LP0002.md
└── lp-0002-private-multisig/     ← SOLUTION REPO (all code/commits happen here)
```

---

```text
You are Claude Code building LP-0002 (Private M-of-N Multisig for LEZ) for the λPrize program.

═══════════════════════════════════════════════════════════════════
0. REPO LAYOUT (do this before coding — mandatory)
═══════════════════════════════════════════════════════════════════

Assume this prompt is run from (or with access to) the `lambda-prize` checkout.

Create the solution as a SIBLING directory — same parent as `lambda-prize`,
NOT inside `lambda-prize/`, NOT a nested clone.

Exact layout:

  <parent>/
    lambda-prize/                      # existing — plans + prize specs (do not pollute with solution code)
    lp-0002-private-multisig/          # NEW — all implementation lives here

Steps (Phase 0 bootstrap, but create folder as soon as read-receipt done):
  1. Detect parent of `lambda-prize` (e.g. `…/myproject`).
  2. If `../lp-0002-private-multisig` does not exist:
       mkdir ../lp-0002-private-multisig
       cd ../lp-0002-private-multisig
       git init -b main
  3. Copy (do not move) plan sources into solution for offline eval:
       mkdir -p docs/plan
       cp ../lambda-prize/planlp0002.md docs/plan/
       cp ../lambda-prize/planlpoo0023.md docs/plan/
       cp ../lambda-prize/prizes/LP-0002.md docs/plan/
       cp ../lambda-prize/PROMPT_CLAUDE_CODE_LP0002.md docs/plan/
  4. Write `docs/TRACKING.md` in the solution repo:

       # Tracking
       - Plan source: ../lambda-prize/planlp0002.md
       - Prize source: ../lambda-prize/prizes/LP-0002.md
       - Sibling layout: intentional — easy human eval side-by-side
       - Current phase:
       - Last green SC:
       - Blockers:

  5. ALL subsequent code, commits, CI, demo.sh, docs/phase-*-status.md
     happen ONLY inside `lp-0002-private-multisig/`.
  6. Do NOT commit solution artifacts into `lambda-prize/` except when
     opening the upstream solution PR (Phase I: solutions/LP-0002.md fork).

Why sibling: human can open both folders, diff plan vs implementation,
and evaluate phase status without hunting nested paths.

═══════════════════════════════════════════════════════════════════
A. MANDATORY READ (do this FIRST — no feature coding until DONE)
═══════════════════════════════════════════════════════════════════

Read these files END-TO-END from `lambda-prize/` (or `docs/plan/` copies).
Do not skim. After reading, write INSIDE the solution repo:
`lp-0002-private-multisig/docs/agent-read-receipt.md`

Sources:
1. `planlp0002.md` (v5.1) — FULL file, every section including Appendices A–G
2. `prizes/LP-0002.md` — prize source of truth
3. `planlpoo0023.md` — competitor/reject context (gaps only; do NOT copy code)

In `docs/agent-read-receipt.md` you MUST list:
- [ ] Sibling path created: `../lp-0002-private-multisig` (absolute path written)
- [ ] Plan version string (must be 5.1 or newer)
- [ ] Count Hard gates H1–H15 (=15)
- [ ] Count Win bars W1–W17 (=17)
- [ ] Phase list: −1, 0, A, B, C, D, E, F, G, H, I
- [ ] Preflight PF-01…PF-15 acknowledged
- [ ] Abort conditions (§0.4.7) one-line quote
- [ ] Human gates listed
- [ ] Beat-#125 deltas (5 rows) summarized
- [ ] Conflict rule acknowledged
- [ ] docs/plan/ copies present

If ANY item missing → STOP and tell human. Do not invent.

═══════════════════════════════════════════════════════════════════
B. ANTI-HALLUCINATION RULES (non-negotiable)
═══════════════════════════════════════════════════════════════════

1. NEVER invent: LEZ APIs, SPEL macros, Risc0 APIs, RPC URLs, program IDs,
   explorer links, CU numbers, ImageIDs, tx hashes, or “tests passed”.
2. ONLY claim green after you RAN the command and captured exit code + log path
   in `docs/phase-<id>-status.md` (inside solution repo).
3. If docs/API unclear → look up official public docs OR mark
   `UNKNOWN — blocked` and ask human. Do not guess.
4. Do NOT clone/rebrand competitor repos (#123/#125/#133).
   `jimmy-claw/lez-multisig` = “why private is hard” only.
5. Do NOT fake `RISC0_DEV_MODE=0`. No nested hardcode `=1`.
6. Do NOT `exit 0` / `continue-on-error` when tools missing on demo/CI e2e.
7. Do NOT open solution PR until Phase H green + video URL + preflight exit 0.
8. Do NOT invent funded wallets or private keys. Ask human.
9. Prefer small verified commits over large unverified dumps.
10. Unsure SC? → RED. Stay in phase.
11. Do NOT put solution code inside `lambda-prize/` working tree.

═══════════════════════════════════════════════════════════════════
C. MISSION
═══════════════════════════════════════════════════════════════════

Build ORIGINAL public repo at sibling path `lp-0002-private-multisig`:
private M-of-N multisig for LEZ — shielded members, no public who-voted,
threshold-only on-chain, nullifiers, Risc0, SPEL, SDK/CLI, Basecamp,
testnet evidence, narrated video → ONE clean PR to logos-co/lambda-prize.

Locked architecture (plan §2 — do not simplify away):
- Privacy-preserving approve (PPE / chained env::verify) — not public re-exec
- LEZ-native membership guest emitting ProgramOutput
- PDA config_hash = H(member_root ‖ M ‖ extra)
- In-circuit live account binding (not derivation-only)
- Nullifier anti double-vote
- Peer privacy: co-members do not learn approver identity
- Treasury transfer reference; default 2-of-3; FULL M as primary evidence

═══════════════════════════════════════════════════════════════════
D. AUTONOMOUS LOOP (strict phase order)
═══════════════════════════════════════════════════════════════════

Work directory for all phases: `../lp-0002-private-multisig` (sibling).
Re-read plan from `docs/plan/planlp0002.md` or `../lambda-prize/planlp0002.md`.

Phases ONLY: −1 → 0 → A → B → C → D → E → F → G → H → I

For EACH phase:
  1. Re-read phase contract in plan §5
  2. Implement only that phase (in sibling repo)
  3. Run EVERY SC-* for the phase
  4. Write `docs/phase-<id>-status.md` + update `docs/TRACKING.md`
  5. ANY SC red → FIX; do NOT advance
  6. All green → commit "phase <id> complete" → next phase

Abort (stop features; document only) if:
  - `gh pr view 125 --repo logos-co/lambda-prize --json reviewDecision -q .reviewDecision` == APPROVED
  - OR any LP-0002 solution PR merged upstream

Check abort at start of every phase.

═══════════════════════════════════════════════════════════════════
E. PHASE CHEAT-SHEET (full SC lists in plan — obey them)
═══════════════════════════════════════════════════════════════════

−1 Competitor+env snapshot; prize Open; VERSIONS draft; toolchain
 0  Bootstrap sibling repo public-ready; dual MIT+Apache; CI; preflight stub exit 1
 A  ADR-001, account model, security, ≥8 errors, formulas, tried-failed stub
 B  Guests + nullifier; DEV_MODE=0 prove once; IMAGE_IDS; SC-B.5 kills derivation-only
 C  SPEL create/propose/approve/execute; IDL; no voter list; public path rejected
 D  SDK/CLI; resume <M; clear errors; peer privacy W8
 E  demo.sh = STANDALONE sequencer + RISC0_DEV_MODE=0; missing tools fail; CI e2e on push
 F  Basecamp .lgx + SHA256 + module.json
 G  Testnet + CU numeric + verify-onchain + explorer CI + full M
    (PAUSE — human funded keys)
 H  Docs + checklist + limitations + video (PAUSE — human narrated video)
    preflight PF-01…PF-15 exit 0
 I  Day-of verify; open solution PR from fork of logos-co/lambda-prize (packet W9)

H1–H15 + W1–W17 green before Phase I. Preflight = plan §6.

═══════════════════════════════════════════════════════════════════
F. HUMAN ESCALATION
═══════════════════════════════════════════════════════════════════

- Funded LEZ testnet keys (before G deploy)
- Narrated video URL + commit on screen (H / W5)
- Eligibility if ban notice #105 unclear (ask human; no alt-account evasion)
- Basecamp click-QA if automation fails

═══════════════════════════════════════════════════════════════════
G. OUTPUT DISCIPLINE
═══════════════════════════════════════════════════════════════════

- Solution path: `<parent>/lp-0002-private-multisig` (sibling of `lambda-prize`)
- Dual license commit 1: LICENSE-MIT + LICENSE-APACHE
- Layout = plan §4
- Every phase: status file + TRACKING.md + commit
- Session end: current phase, SC reds, next action, blockers, absolute sibling path

Start NOW:
  1) Section 0 — create sibling folder + docs/plan copies + TRACKING.md
  2) Section A — read-receipt
  3) Phase −1
Do not skip. Do not hallucinate evidence. Do not build inside lambda-prize/.
```

---

## Operator notes (untuk kamu)

1. Buka Claude Code di `lambda-prize` **atau** parent `myproject` (supaya kedua folder kelihatan).
2. Paste prompt (blok `text`) sebagai first message.
3. Pastikan muncul:
   - `…/myproject/lp-0002-private-multisig/`
   - `docs/agent-read-receipt.md` + `docs/TRACKING.md` + `docs/plan/*`
4. Evaluasi = buka dua folder berdampingan: plan kiri, solusi kanan.
5. Klarifikasi ban #105 sebelum Phase I.
6. Human gates: funded key + video narrasi = tugas kamu.
