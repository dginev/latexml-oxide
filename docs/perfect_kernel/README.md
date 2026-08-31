# Perfect Kernel — raw-interpretation conversion of the TeX Live doc corpus

**Mission** (from `docs/PERFECT_KERNEL.md`): iteratively test, audit and develop
latexml-oxide **kernel** support until package-documentation manuals shipped with
TeX Live convert to high-quality core XML — schema-healthy markup, all content
preserved (auditable against the golden PDF sitting next to each manual).

The defining constraint: **raw interpretation**. Every `.sty` and `.cls` the
manual uses is read as real TeX source through the engine, via
`--preload=[rawstyles,rawclasses]latexml.sty`. We do **not** write new binding
files (`*_sty.rs` / `*_cls.rs`) for the packages under test, and we do **not**
lean on OmniBus for unknown classes — the point is to make the TeX kernel
emulation strong enough that the packages *just work*. Improving the
pre-compiled kernel-dump coverage is in scope; per-package shims are not.

## Why this corpus

Every TeX Live package ships its manual as `doc/latex/<bundle>/<name>.tex`
with the author-compiled `<name>.pdf` beside it. That is a free, huge
(≈2,400-document) test suite with golden renderings, written by the package
authors themselves — the people who stress their own package hardest. A manual
that converts cleanly is strong evidence the kernel handles that package's real
implementation, not a binding's approximation of it.

## Corpus definition

`tools/perfect_kernel/enumerate_corpus.sh` emits the corpus as TSV
(`bundle \t tex \t pdf \t lines`): every `doc/latex/<bundle>/<name>.tex` (depth
≤ 2) that contains an uncommented `\documentclass` and has a sibling
`<name>.pdf`. On this host's TL2025 that is **2,374 documents across ~1,600
bundles**. Some candidates are imperfect (LuaLaTeX-only manuals, fragments that
happen to match) — they stay in; a principled skip goes to the ledger with a
reason, never silently.

## Protocol

One document: `tools/perfect_kernel/run_doc.sh <manual.tex> [outroot]`
Sweep: `tools/perfect_kernel/sweep.sh <corpus.tsv> [outroot]` (JOBS=12,
TIMEOUT_S=120 default; resumable — a doc with a `verdict.tsv` is skipped).

The runner converts to **core XML** (`--xml`) with
`--preload=[rawstyles,rawclasses]latexml.sty`, a 6 GiB RAM guard and a
timeout, into `~/data/perfect_kernel/<bundle>/<name>/` (bulk output stays out
of the repo and out of tmpfs). It writes an ANSI-stripped log and a
`verdict.tsv` line:

```
bundle  name  status  exit  errors  fatals  warnings  seconds
```

`status` follows cortex: **3 fatal, 2 error, 1 warning, 0 clean**, plus
**124 timeout** and **137 killed** (RAM guard / crash). Error counting is the
strict `^Error:[a-z]` grep on the stripped log — never a lax stderr grep.
Sweeps use the **release** profile binary (CLAUDE.md: release = sandbox
sweeps); single-doc triage uses the default test profile.

## Quality bar ("perfect")

A document is *converted perfectly* when, in order of increasing strictness:

1. **S0 — completes**: no fatal, no timeout, no kill.
2. **S1 — silent**: zero `Error:` lines (Perl-zero-error parity bar).
3. **S2 — schema-valid**: the core XML validates against the LaTeXML RelaxNG
   schema.
4. **S3 — content-complete**: all PDF content is present in the XML (spot
   audit against the golden PDF: section census, math census, no swallowed
   pages).

The sweep measures S0–S1 mechanically; S2 is a validation pass over surviving
XML; S3 is per-document audit work, sampled.

## Working method

1. **Sweep** the corpus (or the current tier) → `sweep_verdicts.tsv`.
2. **Cluster** failures by first-error signature (`cluster-classify` skill
   discipline: sample representatives, don't count papers as fixes).
3. **Pick the top cluster**, min-repro it, fix it **in the kernel/engine**
   faithfully to real TeX (`tex.web`, `latex.ltx`, the package's own source —
   note: for raw interpretation the ground truth is the *real* kernel, not
   LaTeXML's `.pool` simplifications).
4. **Guard** each fix with a fixture test under the repo's normal test suite,
   and log it in [LEDGER.md](LEDGER.md).
5. Re-sweep; repeat.

Difficult / open-ended cases (unsupported graphics backends, placement
semantics, side-notes …) are cataloged in
[DIFFICULT_CASES.md](DIFFICULT_CASES.md) instead of being hacked around.

## Documents here

| Doc | Role |
|---|---|
| [LEDGER.md](LEDGER.md) | Living progress ledger: sweep tallies, tier status, fix log |
| [CLUSTERS.md](CLUSTERS.md) | Living failure-cluster worklist from the latest sweep |
| [DIFFICULT_CASES.md](DIFFICULT_CASES.md) | Catalog of hard/open-ended cases and their plans |

Branch discipline: all of this lives on the `perfect_kernel` branch; not pushed
until the work is complete.
