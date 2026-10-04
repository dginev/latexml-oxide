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
command inside a `\fancypagestyle` body both engines discard),
`frontespizio_sty.rs` (the package typesets its title page in a SECOND pdflatex
run on a generated file and re-includes it as a graphic — external compilation,
out of scope like shell-escape; the binding forces the package's own inline
route, batch 56cj), `wrapstuff_sty.rs` (the package places its box from the
LaTeX2e paragraph hooks `para/begin`/`para/end` with `\prevgraf`/`\parshape`
arithmetic — the paragraph builder neither engine models; bound as wrapfig's
inline float, which Perl binds the same way, batch 56cr), `abntex2cite_sty.rs`
(contrib: the package redefines `\bibliography` to input a bibtex-produced
`.bbl` LaTeXML never has, and the `.bib` list is live only in that command's
argument — the interception Perl itself makes for bibunits, batch 56cw),
`directory_sty.rs`, `figbib_sty.rs` and the `nmbib_sty.rs` extension (contrib:
the same shape — `\directory`, `\fbList` and `\multibibliography`/`\printbibliography`
write the `.bib` list into a `\bibdata` `\write` and input a bibtex `.bbl`;
satisfying the raw `\@input@` is structurally impossible, so the wrapping
command runs the kernel `\lx@bibliography`, batch 56dc). A raw class's
title-page STORES (`\inst`, `\abst`, `\recdate`, `\kword`, … — text kept for an
`\@maketitle` LaTeXML never runs) need no binding at all: K11
(`latexml_engine/src/frontmatter_stores.rs`, batch 56dj) detects the store-shaped
setters after the raw load and reroutes them to the frontmatter API by kind. Phase 58 added two raw-load-then-overlay
bindings of the same justified kind: `floatrow_sty.rs` (58i: LaTeXML's locked `\@caption` never fills floatrow's
`\@floatcapt`, so the kernel's caption material is pointed at it) and `enotez_sty.rs` (58l: the package fills its list
only from the previous run's `.aux`). Phase 59 added `g_brief_cls.rs`/`g_brief2_cls.rs` (59o: the letter's sender and
addressee live only in a first-page style LaTeXML never typesets; they become frontmatter at `\begin{g-brief}`, user
ruling 2026-10-01). A raw-first attempt is still the default for every new cluster; a new binding
requires a justification of this kind in the file header.

> **Stages.** The error-free stage (S0∧S1) closed 2026-09-17; the program now measures content preservation and markup
> quality over the in-scope manuals (PERFECT_KERNEL.md → Scope): S2 schema validity, S3 recall in the post-processed HTML,
> and a semantic-markup audit. Verdicts and clusters are logged in `LEDGER.md`.

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
Sweep: `tools/perfect_kernel/sweep.sh <corpus.tsv> [outroot]` (JOBS=8, TIMEOUT_S=120 defaults; resumable — a doc with a
`verdict.tsv` is skipped; release binary, user 2026-09-03). On cortex (the host since 2026-09-17) a sweep runs as a
`systemd-run --user` unit on its own cores with the vendor TeX Live first on PATH; the current recipe (JOBS=16, 180 s,
sweep → validate → post → HTML recall) is `~/data/pk_agents/w70/sweep<N>_launch.sh`; builds, gates and probes take other
cores. Topic repro corpus: `tools/perfect_kernel/repros/<topic>/*.tex` (27 mechanism topics) + runner
`tools/perfect_kernel/repros.sh <topic> [--perl] [--pdflatex] [--recall]`, each repro with a
witness/oracle/engines/expect/status header (conventions in `tools/perfect_kernel/repros/README.md`). Read-only analysis
goes to at most two narrow subagents (one until 2026-10-07; root-causer, reviewer, log-scanner); the main session lands every fix.

The runner converts to **core XML** (`--xml`) with
`--preload=[rawstyles,rawclasses]latexml.sty`, an 8 GB memory cap (`--max-memory=8192`, `ulimit -v 8912896`) and a
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
4. **S3 — content-complete**: the golden PDF's words are present in the post-processed HTML (`s3_sweep.sh`; goldens typeset
   from another source are curated in `tools/perfect_kernel/golden_reference.tsv`).

The sweep measures S0-S2 mechanically and S3 over the HTML root; page furniture is not content (user, 2026-10-01: running
heads, letterheads and page numbers are dropped; semantic notes are kept wherever a class prints them).

**The headline is the in-scope scoreboard** (`tools/perfect_kernel/scoreboard.py`; scope = PERFECT_KERNEL.md → Scope):
clean, errors, schema-valid, HTML recall, cpu_h. A 0-error timeout is not a win (it truncated the document). Rules:
(1) sweeps run on quiet cores (or their timeouts are re-run solo before tallying); (2) compare sweeps per document by
error-count delta and status, not by zero-error flips (the nicematrix exemplar sat at 108 → 1001 for four sweeps under a
flip-only diff; 0 errors since 2026-09-04, history in `archive/LEDGER_PHASE56_2026-09-27.md`); (3) S2 (`validate.sh`)
and S3 (`post_sweep.sh` + `s3_sweep.sh`, HTML root) run with every sweep — zero errors is not correctness.

## Working method

1. **Sweep** the corpus → `~/data/perfect_kernel_s<N>/`; `tools/perfect_kernel/scoreboard.py` prints the in-scope table
   (the 1,602 manuals some engine compiles cleanly) and then the crash canaries.
2. **Pick from the scoreboard** (user, 2026-10-01): S3 missing words first, then schema-invalid in-scope manuals, then
   timeouts; filter by scope first (PERFECT_KERNEL.md → Scope).
3. **Root-cause and fix in the kernel/engine** faithfully to real TeX (`tex.web`, `latex.ltx`, the package's own source —
   for raw interpretation the ground truth is the *real* kernel, not LaTeXML's `.pool` simplifications).
4. **Guard** each fix (a repro in `tools/perfect_kernel/repros/<topic>/` + a red/green test), log it in
   [LEDGER.md](LEDGER.md); a finding not fixed in the batch becomes a RED repro at once. Review rounds are capped.
5. Re-sweep; repeat.

Difficult / open-ended cases (unsupported graphics backends, placement
semantics, side-notes …) are cataloged in
[DIFFICULT_CASES.md](DIFFICULT_CASES.md) instead of being hacked around.

## Documents here

| Doc | Role |
|---|---|
| [LEDGER.md](LEDGER.md) | Living progress ledger: the phase-59 fix log; phases 56-58 summarized, their rows archived |
| [PLANS.md](PLANS.md) | Execution-ready improvement plans still open (P16 residue, P30, P35, P72) and the standing items 5-13; closed rows archived |
| [DIFFICULT_CASES.md](DIFFICULT_CASES.md) | Catalog of hard/open-ended cases and their plans |
| [LUA_REBINDING.md](LUA_REBINDING.md) | LuaTeX-escape strategy: why rebinding IS the emulation; shim tiers, mirror protocol, witnesses |
| [ARCHITECTURE_THEMES.md](ARCHITECTURE_THEMES.md) | Design brief: the twelve kernel mechanisms behind the recurring root causes (group/mode stacks, seam binding, `\halign`, token stream, engine persona, loader/VFS, typed parameters, the horizontal list, bibliographies, the regression net, box sizes, math ranking) with tex.web/latex.ltx models, witnesses, fix shapes and ordering |
| [KERNEL_CAPABILITIES.md](KERNEL_CAPABILITIES.md) | **The approved generalized kernel-capability program** (2026-09-05): K1–K19 with source of truth, abstraction, landing plan, guards, order |
| [gemini.md](gemini.md) | Open-task brief for the second collaborating agent (no open tasks since 2026-10-04) |
| [archive/](archive/) | Frozen: the phase 59-62 close (`PERFECT_KERNEL_PHASE59_62_CLOSE_2026-10-04.md`: sweeps #135-148, the ranked path, the stream-G log), the old root-causer preamble (`AGENT_PREAMBLE_W3_2026-09-23.md`, superseded by `.claude/agents/root-causer.md`), Gemini round 13 (`gemini_ROUND13_2026-09-29.md`); the phase-56 and phase-57/58 ledgers (`LEDGER_PHASE56_2026-09-27.md`, `LEDGER_PHASE57_58_2026-10-02.md`), landed/stopped/closed plans (`PLANS_DONE_PHASE56_2026-09-27.md`, `PLANS_CLOSED_2026-10-02.md`), the KERNEL_CAPABILITIES status log through 09-24 and its landed designs (`KERNEL_CAPABILITIES_LANDED_2026-10-02.md`), superseded PERFECT_KERNEL notes (phase 56; the phase-57/58 plan and corpus-wide scoreboard s113-s130, `PERFECT_KERNEL_PHASE57_58_PLAN_2026-10-02.md`), DIFFICULT_CASES before its 2026-10-02 compaction, CLUSTERS (sweeps 2–25), and the 09-17/09-19 snapshots (recall triage, semantic-markup audit, red-test triage, Windows validation) |

Branch discipline: all of this lives on the `perfect_kernel` branch, pushed at checkpoints.
