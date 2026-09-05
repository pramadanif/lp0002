# `docs/plan/` — verbatim upstream copies

These files are **unmodified copies** of documents that live in the `logos-co/lambda-prize`
repository, kept here so this repository states exactly which text it was built against and so the
plan's clause numbers can be cited without a network round trip.

| File | Upstream |
|------|----------|
| [`LP-0002.md`](LP-0002.md) | `prizes/LP-0002.md` — the prize's own text and success criteria |
| [`planlp0002.md`](planlp0002.md) | the build plan (v5.1) this repository follows |
| [`planlpoo0023.md`](planlpoo0023.md) | the companion plan |
| [`PROMPT_CLAUDE_CODE_LP0002.md`](PROMPT_CLAUDE_CODE_LP0002.md) | the build prompt |

**Their relative links do not resolve here, and that is expected.** They point at paths in the
upstream repository — `../TERMS.md`, `../README.md#evaluation-policies`, `prizes/LP-0002.md` — which
have no counterpart in this tree. The files are left byte-for-byte as published rather than edited
to suit our layout: a copy that has been "fixed" is no longer evidence of what was actually
required.

`scripts/check-links.sh` knows about this directory and skips it. Every other Markdown file in the
repository must have working relative links, because a link that 404s at the pinned commit is a real
failure — it is one of the things the competing submission #125 was pulled up for.
