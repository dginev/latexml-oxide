# Engine Sync Status — Task List

**Mission.** Improve the Rust translation until the 10k-paper sandbox
is error-free on every paper that Perl LaTeXML also converts cleanly.
Perl is the ground truth; Perl-error-only papers are out of scope.

A sandbox paper is **in scope** iff Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` produces
0 errors on it. Mission completes when every in-scope paper also
produces 0 errors on Rust.

Earlier per-iteration narrative: `docs/archive/`. Tactical insights:
`docs/WISDOM.md`. Upstream Perl bugs: `docs/KNOWN_PERL_ERRORS.md`.
Intentional divergences: `docs/OXIDIZED_DESIGN.md`.

---

## Open tasks (highest leverage first)

### 0. math0606553 — `\CompileMatrices` + `\xy@@ix@` re-tokenization

Single-error paper (`undefined:\lx`); affects every paper using
`\usepackage{xy}` + `\CompileMatrices` whose matrix cells contain
`\lx@*` CSes (e.g. `\DeclareMathOperator` operators that expand into
`\lx@dual …`). Min repro at
`latexml_oxide/tests/graphics/xycompile.tex` (no `.xml` pair yet).

Root cause: xy.tex compile mode writes a `.xyc` file, then `\input`s
it back. `\xyxy@@ix@` body sets `@`=OTHER **before** `\toks9 =
{body}` reads, so `\lx@dual` re-tokenizes as `\lx`+`@dual`. Stored
in `\toks9`, later `\the\toks9` fires `\lx` undefined.

Open hypotheses for the Perl-faithful fix: (1) Perl's `\toks =
{balanced}` uses a different catcode snapshot; (2) Perl's
`UnTeX($tokens, 1)` writes `.xyc` differently; (3) Perl reroutes
`\xy@@ix@` via `xylatexml.tex.ltxml`; (4) Rust's one-step
`remove_value("afterAssignment")` collapses local frames where
Perl's two-step `lookup`+`assign(undef)` preserves them. Empirical
band-aid (NOT applied): force `at_letter:true` on `.xyc` paths.
Acceptance: min repro + math0606553.zip → 0 errs, tests 1110+/0/0.

### 1. math0005251 — math-parser cumulative-state OOM

Only filesystem-level hard failure left in the April29 sandbox. Rust
allocates ~28 GB digesting the paper's math while Perl finishes in
~10.5 s / 234 MB. Min repros run cleanly; the trigger needs enough
prior math-state accumulation. See
`memory/project_math_parser_state_cumulative_hangs.md`. Expected
fix is grammar-level work in `latexml_math_parser`.
Acceptance: `( ulimit -v 6291456; latexml_oxide … math0005251.zip )`
exits 0 with non-empty HTML.

### 2. math0601451 — `XMTok` / `XMApp` leaking into `<ltx:title>`

1481× `Error:malformed:ltx:XMTok in <ltx:title>` (plus 54×
`XMApp in <ltx:text>`) on a single amsppt + amstex paper. Distinct
from the siunitx XMTok-in-text trigger. Math constructs inside
amsppt's `\title`/`\heading` need `XMText`-wrapped output, not raw
XMath tokens. Scope: `latexml_engine/src/amsppt*` + the digest path
that promotes XMath into text-context elements.

### 3. siunitx XMTok-in-text

`\num{2.6e7}` in text context emits pre-built XMath tokens that
escape the inline-math wrap. 4-line min repro in
`memory/project_xmtok_in_text_repro.md`. Fix:
`siunitx_sty.rs::six_format_scinumber` should return a wrapped
inline-math whatsit, not raw XMath.

### 4. Sandbox conv_error long-tail — full-canvas verification

35 in-scope April-30 papers all clean in spot-check after
`5e65deaec`. Pending: full 7898-paper rerun via
`tools/benchmark_10k.sh` to verify no regressions and update the
headline number.

### 5. AmSTeX.pool.ltxml — 70% gap

112 defs, ~30% ported. Plain-TeX papers using `\input amstex` (e.g.
math0601451) hit the gap. Low priority while sandbox impact stays
small, but converting more amsppt/amstex papers depends on it.

### 6. expl3 / pgfmath / pgfplots residual clusters

Long-standing deep clusters parked in
`docs/archive/sandbox_failures_SYNC_STATUS.md`. Re-survey whether
recent fixes have shrunk the surface enough to make individual
items tractable. Notables: `1803.03288`/`1902.08705` (expl3 cascade
+ pgfmath `\ifdim`); `1305.3934`/`1404.1023`/`1405.3906` (pgfplots
`\pgfplots@curlegend` state-machine, plan in
`pgfplots_sty.rs:18-28`).

### 7. Schema generation — `--dump-model` CLI flag

Stage 2 of `tools/compileschema.sh` (rng → model) still requires
Perl. Add `latexml_oxide --dump-model` and diff Rust-emitted vs
Perl-emitted `.model` from the same `.rnc`.

### 8. Distribution — bundle multi-TL dumps

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022 … TL2026 and select at runtime
by `kpsewhich --version`. Currently dumps load from
`resources/dumps/` on disk.

---

## Engine file open gaps

| File | Status | Open Gap |
|------|--------|----------|
| `base_parameter_types.rs` | MINOR | Parameterized `CommaList:Type` form unported (no Perl users). |
| `tex_box.rs` | MINOR | Box dimension edge cases. |
| `tex_fonts.rs` | MINOR | `\fontdimen` array semantics; per-font `\hyphenchar`. |
| `tex_tables.rs` | MINOR | Padding CSS classes (XSLT concern). |
| `plain_base.rs` | OPEN | Some closure-backed defs need conversion to Token bodies for dump round-trip. |
| `latex_base.rs` | OPEN | Closure-backed defs need conversion or relocation to `latex_constructs.rs`. |

---

## Tikz known diffs vs Perl

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox / total width differs slightly
4. tikz matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs
   inline-blocks (Perl)

---

## Permanent ignores

* **Sandbox out-of-scope:** ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
* **Rust supersedes Perl** (both still in scope, but Rust passes
  where Perl errors): `1207.6068`, `0909.3444`.
* **Unported pools:** `AmSTeX.pool.ltxml` (~70% remaining),
  `BibTeX.pool.ltxml` (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1110/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| April-30 in-scope (35 papers) | 35/35 clean (spot-check) | 35/35 |
| Filesystem-level hard failures in latest canvas | 1 (math0005251) | 0 |
| `results.tsv` `ok` rate | 7796/7898 = 98.71% (Apr29 baseline) | match Perl on the same set |
