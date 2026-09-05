# Perfect Kernel — raw-interpretation conversion of the TeX Live doc corpus

**Mission** (from `docs/PERFECT_KERNEL.md`): iteratively test, audit and develop
latexml-oxide **kernel** support until package-documentation manuals shipped with
TeX Live convert to high-quality core XML — schema-healthy markup, all content
preserved (auditable against the golden PDF sitting next to each manual).

The defining constraint: **raw interpretation for the uncovered long tail**.
Conversions run with `--preload=[rawstyles,rawclasses]latexml.sty`, so any
`.sty`/`.cls` **without** a compiled `.rs` binding is read as real TeX source
through the engine. Compiled bindings — contrib included, OmniBus-delegating
ones too — **always keep precedence** (user directive 2026-08-31); raw mode
never demotes them. What raw mode changes is the bindingless case: a class
with no binding raw-loads instead of falling to the OmniBus unknown-class
fallback (guard: `cluster_package_guards::rawclasses_binding_precedence_and_no_omnibus`).
We do **not** write new binding files for the packages under test — the point
is to make the TeX kernel emulation strong enough that they *just work* raw.
Improving the pre-compiled kernel-dump coverage is in scope; per-package shims
are not. Corpus work therefore focuses on manuals whose packages/classes have
**no `.rs` binding yet**.

**Recorded exceptions to "no new bindings" (all user-precedented):** the
mission text predates three user amendments — locked-CS conflicts resolve via a
new class binding (ltxdockit precedent), bindings always outrank raw, and
complete support beats stubs. Under those, this mission has added bindings
ONLY where raw interpretation is structurally impossible or out of scope, each
with the justification in the file header: `xkeymask_sty.rs` (raw package
depends on a genuinely self-referential macro that only real TeX's
single-level expansion tolerates — our/Perl recursion guard is load-bearing),
`assoccnt_sty.rs` (raw package wraps kernel counter commands, which our engine
also invokes at CONSTRUCTION time inside elements — the wrapper leaks tokens
into the DOM), `titleps_sty.rs`/`schooldocs_sty.rs` (purely presentational
page-style surfaces with no XML counterpart; schooldocs hides its one semantic
command inside a `\fancypagestyle` body both engines discard). A raw-first
attempt is still the default for every new cluster; a new binding requires a
justification of this kind in the file header.

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
TIMEOUT_S=120 default; resumable — a doc with a `verdict.tsv` is skipped;
build the sweep binary `--release`, user directive 2026-09-03).
Topic repro corpus: `tools/perfect_kernel/repros/<topic>/*.tex` + runner
`tools/perfect_kernel/repros.sh <topic> [--perl] [--pdflatex]` — minimal
self-contained repros grouped by MECHANISM (alignment, boxes-groups, index,
string-mouth, sectioning-frontmatter, luatex-profile, expl3), each with a
witness/oracle/engines/expect/status header; the residue is worked one topic
at a time, five read-only `root-causer` agents feeding repros + fix plans per
topic, the main session landing fixes (user workflow 2026-09-03; conventions in
`tools/perfect_kernel/repros/README.md`).

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

**The headline number is S0∧S1 on the oracle-clean slice** — COMPLETED with
zero `Error:` lines — computed by `tools/perfect_kernel/tally.sh`. A 0-error
timeout is not a win (it truncated the document), and the legacy "zero-error"
count included them: sweep 26 reported 1,276 zero-error / 1,049 slice while
its honest S0∧S1 was 1,163 / 981 — it ran under 8-agent + build contention
and timed out 266 docs (111 with 0 errors) against sweep 25's 19. Rules that
follow: (1) sweeps run on a **quiet machine** (no builds, ≤ 2 subagents) or
their timeouts are re-run solo before tallying (`delete verdict.tsv` for
status-124 docs, re-invoke `sweep.sh` at `JOBS=6`); (2) `tally.sh` diffs the
previous sweep by **per-document error-count delta** (Δ ≥ 5 or status
worsening), not by zero-error flips — the nicematrix exemplar sat at
108 → 1001 for four sweeps unnoticed by a flip-only diff; (3) the exemplar
rows are printed in every tally and the LEDGER exemplar table gets a row per
sweep; (4) S2 (`validate.sh`) and S3 word-recall (`s3_audit.sh`) are re-measured
over the S0∧S1 slice every few sweeps — zero errors is not correctness.

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
| [PLANS.md](PLANS.md) | Detailed, execution-ready improvement-plans ledger (P1…P77+) |
| [DIFFICULT_CASES.md](DIFFICULT_CASES.md) | Catalog of hard/open-ended cases and their plans |
| [LUA_REBINDING.md](LUA_REBINDING.md) | LuaTeX-escape strategy: why rebinding IS the emulation; shim tiers, mirror protocol, witnesses |
| [ARCHITECTURE_THEMES.md](ARCHITECTURE_THEMES.md) | Design brief: the six kernel mechanisms behind the recurring root causes (group/mode stacks, seam binding, `\halign`, token stream, engine persona, loader/VFS) with tex.web/latex.ltx models, witnesses, fix shapes and ordering |
| [KERNEL_CAPABILITIES.md](KERNEL_CAPABILITIES.md) | **The approved generalized kernel-capability program** (2026-09-05): K1–K8 with source of truth, abstraction, landing plan, guards, order |
| [AGENT_PREAMBLE_W3.md](AGENT_PREAMBLE_W3.md) | Standard instructions & constraints for read-only root-causer subagents |
| [HANDOFF_2026-09-01.md](HANDOFF_2026-09-01.md) | Session handoff notes for Wave 3 investigation restart |
| [HANDOFF_2026-09-03.md](HANDOFF_2026-09-03.md) | Advisory notes & Batch-54q handoff: digestion timing, VFS, alignment, SymHashMap and WASM synergy |

Branch discipline: all of this lives on the `perfect_kernel` branch; not pushed
until the work is complete.
