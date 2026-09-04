# Plan: Gap chaseable — LP-0002 / LP-0003 / LP-0008

**Tanggal:** 2026-09-04  
**Deadline legacy:** 11 Sep 2026, 23:59 CEST (~7 hari)  
**Aturan:** karya **original**. Baca kelemahan kompetitor = belajar evidence gate. **Bukan** clone/rebrand repo orang. TERMS + FAQ melarang klaim prize di punggung kerja orang lain.

---

## 0. Ringkas putusan

| Prize | Masih bisa dikejar? | Kenapa |
|-------|---------------------|--------|
| **LP-0003** | **Hampir tidak** | #124 Timidan **eng APPROVED** → nunggu merge program. Gap mereka sudah ditutup. |
| **LP-0002** | **Ya, tipis** | 3 PR open, **belum ada eng approve**. #125 paling kuat di kertas; #123/#133 masih punya lubang review-history. |
| **LP-0008** | **Ya, tapi berat** | 2 PR open, belum eng approve. Scope XL; #129/#130 sudah sangat lengkap. Hanya masuk akal kalau kamu sudah punya agent stack hampir jadi. |

**Rekomendasi prioritas chase (kalau tetap legacy):**

1. **LP-0002** — satu-satunya open Large dengan review belum selesai + history reject jelas → kamu tahu exact bar yang harus dilampaui.  
2. **LP-0008** — hanya jika capacity penuh + sudah partial build.  
3. **LP-0003** — **skip** kecuali #124 dibatalkan/ditolak program team (pantau harian).

---

## 1. LP-0003 — Private Allowlist / Airdrop ($600)

### Kompetitor open

| PR | Author | Status |
|----|--------|--------|
| [#124](https://github.com/logos-co/lambda-prize/pull/124) | Timidan | **APPROVED** eng (`danisharora099`); rekomendasi award + merge |
| [#126](https://github.com/logos-co/lambda-prize/pull/126) | edenbd1 | Validate ✅; belum human review; datang belakangan |

### Gap historis (PR closed) — sudah ditutup pemenang-calon

Dari #84 / #98 Timidan + #86 retraca + #121 dhozil:

| Gap dulu | Status sekarang (#124) |
|----------|-------------------------|
| No / lemah testnet (≥2 dist, ≥20 claims) | **Fixed** — live evidence + manifest |
| CI standalone **skip path** (S2) | **Fixed** — full run real proof mode |
| Video tak legible / tak tunjukkan `RISC0_DEV_MODE=0` + claim + dup reject (S6) | **Fixed** — video baru disetujui |
| No downloadable Basecamp `.lgx` (U2) | **Fixed** — packages load |
| CU ragu | Ada di report |

Follow-up #124 **non-blocking** (clarifikasi wording native claim, dua revisi release/source, bash requirement) — **tidak menahan merge**.

### Gap yang masih “terbuka” buat chase?

**Praktis: tidak.** FCFS = first yang **lolos semua criteria**. Eng sudah bilang #124 lolos. #126 hanya menang jika #124 ditolak di final award (kecil kemungkinan).

**Aksi kamu:** pantau #124. Kalau merged → LP-0003 mati. Jangan invest build 0003 minggu ini.

---

## 2. LP-0002 — Private M-of-N Multisig ($1,200) — TARGET UTAMA JIKA CHASE LEGACY

### Kompetitor open

| PR | Author | Dibuka | Signal reviewer |
|----|--------|--------|-----------------|
| [#123](https://github.com/logos-co/lambda-prize/pull/123) | FidelCoder | Aug 11 | Validate ✅; warn `module.json`; **no eng approve**; nunggu lama |
| [#125](https://github.com/logos-co/lambda-prize/pull/125) | edenbd1 | Aug 13 | Validate ✅; weboko: CI green (claimed fixed); **evidence style paling tebal** |
| [#133](https://github.com/logos-co/lambda-prize/pull/133) | jeefxM | Aug 28 | Validate ✅; submit ke-3; belum review ulang setelah #97 |

### Gap dari PR **ditolak** (ini checklist yang harus kamu lampaui)

#### A. jeefxM — #91 lalu #97 (sekarang #133 mencoba nutup)

| Gap reviewer (weboko) | Cara nutup (karya sendiri) |
|-----------------------|----------------------------|
| CU cost not reported | Ukur tiap ix di sequencer/testnet → `docs/cu-costs.md` angka nyata |
| Partial-approval resume across restarts belum | Persist approval set ke disk/local store; restart client → lanjut tanpa hilang; test + demo |
| E2E standalone sequencer **tidak di CI** | Job CI: clone pin LEZ, start standalone, full lifecycle, **tanpa skip** |
| README CLI/Basecamp walkthrough kurang | Step-by-step clone → build → create → propose → approve ×M → execute |
| Basecamp assets bukan downloadable terpisah | GitHub Release: `.lgx` + SHA256; link di solution MD |
| Anonymous-approval **derivation-only**, bukan in-circuit live-account proof | Circuit harus prove control terhadap **shielded account hidup** (model LEZ nyata), bukan cuma derive key off-chain |
| Nested `demo.sh` hardcode `RISC0_DEV_MODE=1` (#97) | Satu entrypoint `./demo.sh`; child **inherit** env; default `0`; log boot line `(RISC0_DEV_MODE=0)` |
| CI default branch merah (#97) | CI hijau di `main` sebelum buka solution PR |
| Tx tak verifiable di explorer (#97) | Deploy + propose + approve + execute links **hidup** di `explorer.testnet.lez.logos.co` |

#### B. Tranquil-Flow — #68 / #92 / #131

| Gap reviewer | Cara nutup |
|--------------|------------|
| Cuma localnet, bukan testnet | Deploy + lifecycle di public testnet |
| Tx explorer kosong / perlu re-submit | Setelah testnet wipe/redeploy: **ulang** semua tx; update links |
| CI tidak run E2E LEZ sequencer | Job dedicated lifecycle + assert inclusion |
| Execute tx **contains no proof** (#131) | Path **privacy-preserving** dengan receipt nyata; bukan public re-exec “ZK costume” |
| **No multisig-related transactions** (#131) | On-chain harus kelihatan: create/propose/approve/execute multisig — bukan cuma deploy gate generic |
| CU “unavailable” | Jangan tulis unavailable; ukur atau fail criteria |

#### C. FidelCoder — #120 (lalu #123)

| Gap reviewer | Cara nutup |
|--------------|------------|
| CI harus hijau + **LEZ sequencer tests** | Sama: real sequencer job |
| Testnet evidence missing | Links explorer + state akhir |
| CU cost hilang | `docs/cu-costs.md` |
| Video / `module.json` warning (validator) | Narrated video + packaging Basecamp yang validator/heuristik terima |

#### D. Lain (cepat close)

| Siapa | Gap |
|-------|-----|
| nizarsyahmi #102 | No narrated video |
| duongja #115 | CI tidak test LEZ logic; CU file bilang “no CU” |
| retraca #87 | No video |

### Gap **tersisa** di PR open (peluang mengungguli)

Bukan “kode mereka jelek” — ini **lubang yang reviewer masih bisa pakai untuk fail**, atau yang kamu harus **samakan/lampaui** dengan build sendiri:

| Kompetitor | Lubang / risiko residual | Cara kamu menutup lebih baik |
|------------|--------------------------|------------------------------|
| **#123 FidelCoder** | Warn `module.json`; review 1 bln tanpa approve; pin LEZ **v0.2.2** (testnet mungkin sudah maju); video di CDN commit lama vs solution commit | Pin **versi testnet terkini**; evidence + video **satu commit**; Basecamp load proof; packaging lengkap |
| **#125 edenbd1** | Sedikit lubang publik; weboko belum deep-review crypto; bergantung reviewer sempat audit PPE/PDA claims | Samakan bar: PPE path + PDA-anchored member set/threshold + chained `env::verify` + verify script dari public data; jujur `limitations.md` |
| **#133 jeefxM** | History #91/#97 berat; #133 klaim fix tapi **belum di-review**; risiko residual: binding live-account + tx freshness setelah wipe | Jangan ulangi derivation-only; pastikan setiap explorer link resolve **hari-H submit** |

### Bar menang LP-0002 (minimal lolos — mirror criteria + reject list)

**Functionality**
- [ ] Approval dari shielded member **tanpa** bocorkan identity on-chain / ke member lain
- [ ] Verifier on-chain: threshold M terpenuhi **tanpa** catat siapa approve
- [ ] Nullifier (atau setara) anti double-vote
- [ ] Execution unlinkable ke shielded account individu
- [ ] Proof client-side laptop
- [ ] ≥1 multisig testnet: create → propose → approve×M → execute; evidence reproducible
- [ ] **In-circuit live-account binding** (bukan derivation-only) — gap #91
- [ ] Prefer **privacy-preserving** path dengan proof di execute (bukan public re-exec) — gap #131 / pelajaran #125

**Usability**
- [ ] SDK/module
- [ ] Basecamp GUI + **downloadable** `.lgx` / assets (Release)
- [ ] SPEL IDL

**Reliability**
- [ ] Proof failure → error jelas
- [ ] Partial approvals **resume after restart**
- [ ] Deterministic error codes (invalid proof, double-vote, …)

**Performance**
- [ ] CU per operation **terukur** (bukan “unavailable”)

**Supportability**
- [ ] Deployed testnet
- [ ] CI: E2E vs **real** standalone LEZ sequencer, hijau di default branch, **no skip**
- [ ] README E2E CLI + Basecamp
- [ ] `demo.sh` vs real local sequencer, **`RISC0_DEV_MODE=0`** benar (cek nested scripts)
- [ ] Video narrated: terminal tunjukkan mode 0 + proof gen (wait boleh dipotong)

### Plan eksekusi LP-0002 (original, 7 hari — agresif)

> Hanya realistis jika kamu full-time + sudah kenal Risc0/LEZ. Kalau mulai dari nol minggu ini → peluang rendah vs #125.

| Hari | Fokus | Output |
|------|--------|--------|
| D0 | Baca LP-0002.md + spel + LEZ account model; desain nullifier + membership; **bukan** copy repo kompetitor | ADR 1 halaman |
| D1–D2 | Circuit + SPEL verifier; local prove/verify | Guest + program compile |
| D3 | Local sequencer E2E: create/propose/approve/execute | `demo.sh` DEV_MODE=0 lokal |
| D4 | CI job sequencer + unit tests | CI hijau |
| D5 | Testnet deploy + full lifecycle tx; CU measure | `DEPLOYMENT.md` + explorer links |
| D6 | Basecamp module + Release `.lgx` + SDK/CLI polish | Downloadable assets |
| D6 malam | Video narrated (commit on screen) | YouTube/Release MP4 |
| D7 | `solutions/LP-0002.md` pin commit; buka PR `Solution: LP-0002 — …` | PR sebelum deadline |

**Urutan PR matters:** FCFS = first **yang lolos criteria**, bukan first open. Tapi kalau #125 di-approve sebelum kamu siap → game over. Pantau #123/#125/#133 tiap hari.

---

## 3. LP-0008 — Autonomous AI Module ($1,200)

### Kompetitor open

| PR | Author | Dibuka | Catatan |
|----|--------|--------|---------|
| [#129](https://github.com/logos-co/lambda-prize/pull/129) | edenbd1 | Aug 19 | On-chain spend policy PDA; 3 agent testnet; batasan jujur di `limitations.md` |
| [#130](https://github.com/logos-co/lambda-prize/pull/130) | aegonmyy | Aug 23 | Full skills + A2A + `.lgx` + demo; style winner LP-0017 |

### Gap dari PR **ditolak**

| PR | Gap reviewer | Cara nutup |
|----|--------------|------------|
| #99 duongja | No visible agents on **testnet**; video **localnet only**; E2E tak jalan; no evidence; **`demo.sh` doesn't run** | 3 agent kategori Storage/Messaging/Blockchain di testnet + tx; video testnet; CI E2E; demo.sh dari clean clone |
| #81/#88 retraca | Validation fail / no video | Video narrated + criteria penuh |
| #128 | Diganti #129 | — |

### Gap residual di open (tipis)

| Kompetitor | Lubang yang mereka **akui** / risiko | Cara nutup lebih bersih |
|------------|-------------------------------------|-------------------------|
| **#129** | 3 agent published: above-threshold approve path **beku** (owner anchor saat unclaimed); storage skill kurang transcript/CI; inbound A2A tidak auto-dispatch ke skill; no real model inference (out of scope OK) | Provision agent dengan owner claimed **sebelum** anchor; demo `approve_spend` pada agent yang sama; CI transcript storage; dokumentasikan A2A inbound jelas |
| **#130** | Belum human review publik panjang; harus buktikan semua default skills + 3 use case + A2A payment | Checklist skills satu per satu + 3 use case E2E evidence |

### Bar menang LP-0008 (singkat)

- [ ] Logos Core module load bersama wallet/storage/messaging
- [ ] Agent punya shielded LEZ account sendiri
- [ ] Deploy 1 CLI command + owner chat via Messaging
- [ ] Spending threshold + escalate ke owner
- [ ] **Semua** default skills (Storage/Messaging/Blockchain/A2A/Meta)
- [ ] A2A-compatible cards + lifecycle + LEZ payment
- [ ] ≥2 agent discover + task + pay tanpa owner
- [ ] ≥3 illustrative use cases E2E testnet
- [ ] **3 agents** testnet (satu per kategori skill) + evidence
- [ ] CI sequencer E2E; demo.sh `DEV_MODE=0`; video narrated; CU docs; Basecamp assets

### Plan eksekusi LP-0008

**Jangan mulai dari nol dalam 7 hari.** Scope = banyak sistem (Core module + Waku + Storage + LEZ + A2A).

Kalau sudah punya skeleton:
1. Nutup evidence testnet (3 agents + settlements) dulu — gap #99 paling mematikan.  
2. Baru skills completeness + video + CI.  
3. Baca `limitations.md` #129 — jangan ulangi freeze policy.

---

## 4. Matriks: gap mana yang “masih bisa dikejar”

| Gap type | 0003 | 0002 | 0008 |
|----------|------|------|------|
| Kompetitor belum eng-approve | ❌ (#124 approved) | ✅ | ✅ |
| History reject kasih blueprint jelas | — | ✅ sangat jelas | ✅ (#99) |
| Residual weakness di open PR | ❌ hampir nol | ⚠️ #123 packaging/version; #133 unreviewed; #125 tipis | ⚠️ #129 limitations; #130 unreviewed |
| Waktu 7 hari cukup dari nol | ❌ | ⚠️ hanya full-time + ZK siap | ❌ |
| Skill-fit ex-LP-0012 (LEZ runtime) | medium | medium–hard (ZK+) | rendah (AI/agent/stack) |

**Satu kalimat:** gap yang masih bisa dikejar = **LP-0002 evidence/crypto bar yang bikin #91/#97/#131 gagal**, dikerjakan sebagai **implementasi original** yang lebih bersih dari #123/#133 dan setidaknya setara #125 — sebelum salah satu dari mereka di-approve.

---

## 5. Cara kerja anti-reject (berlaku 002/003/008)

Salin mental model reviewer:

```
1. Explorer tx hidup hari-H?
2. Ada domain tx (multisig / claims / agent settle) + proof di path yang benar?
3. CI: real LEZ sequencer, no skip, hijau di main?
4. demo.sh: RISC0_DEV_MODE=0 nyata (cek child script)?
5. Video: narrated + mode + commit + happy + reject path?
6. CU documented dengan angka?
7. Basecamp downloadable?
8. Criteria checklist → bukti path/tx/test per baris?
```

Gagal satu → close / resubmit (max 3 / prize, 1 review / minggu).

---

## 6. Larangan (agar tidak diskualifikasi)

- Jangan clone repo kompetitor lalu strip author / rebrand.  
- Jangan submit “fix” dari kode orang sebagai karya sendiri.  
- Boleh baca PR comment + prize spec + **prior art berlisensi dengan attribution** (bukan PR solusi aktif orang).  
- Dual license MIT **dan** Apache-2.0.  
- Pin satu commit di solution MD.

---

## 7. Keputusan operasional minggu ini

1. **Harian:** cek status #124 (0003), #123/#125/#133 (0002), #129/#130 (0008).  
2. **Default chase:** LP-0002 original dengan bar di §2 — **atau** bail ke LP-0023 (adoption-first) kalau D2 belum ada circuit+local E2E.  
3. **Skip 0003** sampai ada sinyal #124 gagal award.  
4. **0008** hanya jika sudah ≥50% skills + module jalan hari ini.

---

## 8. Definisi menang (0002)

Evaluator:
- clone repo kamu  
- `./demo.sh` sukses tanpa edit, `RISC0_DEV_MODE=0`  
- verifikasi explorer links  
- cek CI hijau + sequencer E2E  
- cek Basecamp load + IDL  
- crypto/binding tidak “derivation-only”  

→ solution PR merge sebelum / mengungguli approve kompetitor.

---

*Dokumen ini = plan chase gap legacy open. Bukan endorsement menyalin kompetitor.*
