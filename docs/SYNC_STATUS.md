# Engine Sync Status — Task List

**Mission.** Improve the Rust translation until the 10k-paper sandbox
is error-free on every paper that Perl LaTeXML also converts cleanly.
Perl is the ground truth; Perl-error-only papers are out of scope.

Earlier per-iteration narrative is archived at
`docs/archive/SYNC_STATUS_2026-04-30_pre-tasklist.md`. Tactical
insights are in `docs/WISDOM.md`; upstream Perl bugs in
`docs/KNOWN_PERL_ERRORS.md`; intentional divergences in
`docs/OXIDIZED_DESIGN.md`.

---

## Open tasks (highest leverage first)

### 1. math0005251 — math-parser cumulative-state OOM

Only filesystem-level hard failure left in the April29 sandbox. Rust
allocates ~28 GB digesting the paper's math while Perl finishes in
~10.5 s / 234 MB. Min repros run cleanly; the trigger requires
enough prior math-state accumulation. See
`memory/project_math_parser_state_cumulative_hangs.md`.

* Goal: process `math0005251.zip` under 6 GB cap.
* Expected fix path: grammar-level work in `latexml_math_parser`
  (per-formula state reset is bounded but doesn't restore parity).
* Acceptance: `( ulimit -v 6291456; latexml_oxide --preload=ar5iv.sty
  math0005251.zip … )` exits 0 with non-empty HTML.

### 2. math0601451 — `XMTok` / `XMApp` leaking into `<ltx:title>`

1481× `Error:malformed:ltx:XMTok in <ltx:title>` (plus 54×
`XMApp in <ltx:text>`) on a single amsppt + amstex paper.
Distinct from the documented siunitx XMTok-in-text trigger.

* Goal: math constructs inside amsppt's `\title` / `\heading`
  expand to `XMText`-wrapped content, not raw XMath tokens.
* Scope: `latexml_engine/src/amsppt*` (or wherever amsppt's title
  capture lives) + the digest path that promotes XMath into
  text-context elements.
* Acceptance: `latexml_oxide … math0601451.zip` produces 0
  `Error:malformed:ltx:XMTok` lines.

### 3. siunitx XMTok-in-text (deferred from earlier session)

`\num{2.6e7}` in text context emits pre-built XMath tokens that
escape the inline-math wrap. Min repro is 4 lines; documented in
`memory/project_xmtok_in_text_repro.md`.

* Goal: `siunitx_sty.rs::six_format_scinumber` returns a properly
  wrapped inline-math whatsit, not raw XMath.
* Acceptance: 4-line min repro produces 0 errors.

### 4. `\lx@dual` recovery-recursion follow-up — regression test

The `f6a6175ea` fix (display char `"$"` not `"\\$"`) resolved the
8-paper cluster. Add a TDD regression test so the fix can't drift.

* Add `latexml_oxide/tests/structure/math_dollar.tex` containing
  `$\$$` and matching `.xml` (run `cargo clean` to force test
  rediscovery).
* Acceptance: test passes; `cargo test --tests` stays at
  current `1109+1 / 0 / 0`.

### 5. Sandbox conv_error long-tail — per-paper triage

`results.tsv` has ~93 papers in the conversion_error bucket. Iter
39 sample of 12 random papers showed 2 fully clean on HEAD and 10
with 1-26 errors each — no shared cluster, mostly per-paper
stub-undefined CSes (`\gnuplot`, `\bullets` typo) and mode-switch
edge cases. Continue triaging in batches; add stubs only where
Perl emits no error on the same input.

* Tooling: `tools/triage_failure.sh <arxiv_id>` is the entry point.
* Reference: `easy_rerun_failures_list.txt` (181 failure-list from
  earlier canvas, mostly already recovered).
* Acceptance per paper: Rust error count ≤ Perl error count on
  same input under `--preload=ar5iv.sty
  --path=~/git/ar5iv-bindings/bindings`.

### 6. Sandbox results.tsv — fresh rebuild

Last full canvas snapshot is `~/data/10k_sandbox_html_April29/results.tsv`
(7796/7898 = 98.71% ok). Per-paper retest of the 12 hard failures
shows 11/12 now resolved on HEAD. Re-run the canvas to capture the
post-`f6a6175ea` headline number.

* Tooling: `tools/benchmark_10k.sh --worker-bin <path>` (default
  test profile).
* Acceptance: rebuild and update the dashboard row in this doc.

### 7. AmSTeX.pool.ltxml — 70% gap

112 defs, ~30% ported. Plain-TeX papers using `\input amstex`
(e.g. math0601451) hit the gap. Low priority while sandbox impact
stays small, but converting more amsppt/amstex papers depends on it.

### 8. expl3 / pgfmath / pgfplots residual clusters

Long-standing deep clusters parked in
`docs/archive/sandbox_failures_SYNC_STATUS.md`. Re-survey whether
recent fixes have reduced the surface enough to make individual
items tractable.

* `1803.03288` / `1902.08705` — expl3 cascade + pgfmath `\ifdim`.
* `1305.3934` / `1404.1023` / `1405.3906` — pgfplots `\pgfplots@curlegend`
  state-machine. Deferred fix-plan in
  `latexml_package/src/package/pgfplots_sty.rs:18-28`.

### 9. Schema generation — `--dump-model` CLI flag

Stage 2 of `tools/compileschema.sh` (rng → model) still requires
Perl. Add `latexml_oxide --dump-model` that writes the loaded
schema in `.model` format, then extend `compileschema.sh` to call
it. Diff Rust-emitted vs Perl-emitted `.model` from the same `.rnc`.

### 10. Distribution — bundle multi-TL dumps

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022 … TL2026 and select at runtime
by `kpsewhich --version`. Currently dumps are loaded from
`resources/dumps/` on disk.

---

## Engine file open gaps

| File | Status | Open Gap |
|------|--------|----------|
| `base_parameter_types.rs` | MINOR | Parameterized `CommaList:Type` form unported (no Perl users). |
| `tex_box.rs` | MINOR | Box dimension edge cases. |
| `tex_fonts.rs` | MINOR | `\fontdimen` array semantics; `FontDef` simplified to `FontToken` blocks per-font `\hyphenchar`. |
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
* **Perl-error-only papers (Rust SUPERSEDES Perl):** `1207.6068`,
  `0909.3444` — Rust converts cleanly, Perl emits errors; tracked
  here so they stay out of the parity target.
* **Unported pools:** `AmSTeX.pool.ltxml` (~70% remaining), `BibTeX.pool.ltxml`
  (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1109/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| Filesystem-level hard failures in latest canvas | 1 (math0005251) | 0 |
| `results.tsv` `ok` rate | 7796/7898 = 98.71% (Apr29) | match Perl on the same set |

A sandbox paper is **in scope** iff Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` produces
0 errors on it. The mission completes when every in-scope paper
also produces 0 errors on Rust.
