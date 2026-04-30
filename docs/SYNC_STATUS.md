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

### 0. math0606553 — `\CompileMatrices` + `\xy@@ix@` re-tokenization

In-progress investigation, paused mid-debug 2026-04-30. Single-error
paper (`undefined:\lx`); affects every paper using `\usepackage{xy}`
+ `\CompileMatrices` whose matrix cells contain `\lx@*` CSes (e.g.
`\DeclareMathOperator` operators expand into `\lx@dual …`).

**4-line min repro** (now committed at
`latexml_oxide/tests/graphics/xycompile.tex`, **no `.xml` pair yet** —
test would fail until fix lands):
```tex
\documentclass{article}
\usepackage[arrow,curve,matrix]{xy}
\CompileMatrices
\begin{document}\xymatrix{A \ar[r] & B}\end{document}
```
Triggered with `\DeclareMathOperator{\shom}{...}` and `\shom` in a
matrix cell. Without `\CompileMatrices`, clean. With it, `\lx`
undefined.

**Root cause traced**: xy.tex compile mode writes a `.xyc` file via
`\write` (UnTeX/`untex`), then `\input`s it back. Each cell entry is
re-input via `\xy@@ix@{body}` which expands to `\xyxy@@ix@`'s body
(xy.tex L266-267):
```tex
\xydef@\xyxy@@ix@{\begingroup
 \xyuncatcodes\afterassignment\endgroup\global\toks9=}
```
`\xyuncatcodes` sets `@` to OTHER **before** `\toks9 = {body}` reads
the body. So `\lx@dual` inside the cell body re-tokenizes as
`\lx`+`@dual` (wrong). Stored in `\toks9`. Later `\the\toks9` expands
the bad tokens; `\lx` undefined error fires.

**Catcode trace confirms**: at `\input min_repro-01.xyc`, `@`=OTHER.
`\xycompiled` body fires `\xycatcodes` → `@`=LETTER (depth=8). Then
each `\xy@@ix@{...}` opens `\begingroup` (depth=9), runs
`\xyuncatcodes` → `@`=OTHER. `\afterassignment\endgroup` IS firing
correctly (verified: 17× saved + 17× consumed via Register::digest →
state::after_assignment), and the group does pop. But the body inside
`\toks9={...}` was already tokenized at depth=9 with `@`=OTHER, so
the popped-to-LETTER catcode comes too late.

**Perl works on the same input.** Open question: how? Hypotheses to
audit:
1. Perl's `\toks N = {balanced}` reading uses a different catcode
   snapshot than ours.
2. Perl's `UnTeX($tokens, 1)` writes the `.xyc` content with a CS
   form that re-tokenizes correctly even with `@`=OTHER (e.g.
   inserts a guard or escapes differently).
3. Perl's `\xy@@ix@` resolves to a different macro than ours
   (`\meaning\xy@@ix@` in Perl returned the body of `\xy` itself,
   not `\xyxy@@ix@`'s — strongly suggests Perl's `\plainxy@`
   `\let\xy@@ix@=\xyxy@@ix@` did not fire in our test, OR Perl
   reroutes `\xy@@ix@` via `xylatexml.tex.ltxml`).
4. Perl's `remove_value`-equivalent for `afterAssignment` is a
   two-step `lookupValue` + `assignValue(=>undef, 'global')`; ours
   is one-step `remove_value`. If `remove_value` collapses local
   frames where the two-step preserves them, that could let the
   group-pop revert the wrong catcode binding. Worth a focused diff.

**Perl-faithful changes already applied to `xy_sty.rs`** (compile but
do not fix):
* `\xystycatcode` is now `sub[_args]` returning `Explode(catcode('@'))`
  dynamically (mirrors Perl xy.sty.ltxml L19), replacing the
  `"12"` hard-coding.
* Pre-`InputDefinitions` `assign_catcode('@', OTHER, Global)` and
  post-load restore (mirrors xy.sty.ltxml L21
  `AssignCatcode('@' => CC_OTHER)` + `\xyuncatcodes`'s implicit
  reset back).

**Empirical band-aid that DOES fix it** (NOT applied — non-Perl-
faithful, recorded for reference): in `load_tex_content`
(`latexml_core/src/binding/content.rs`), set `at_letter: true` when
the input path ends with `.xyc`. Both min repro and full math0606553
go to 0 errors. Side-stepping `\xyuncatcodes`'s effect by forcing
`@`=LETTER throughout the .xyc input.

**Next steps for the fix**:
1. Verify hypothesis (3): patch Perl's xy.tex.ltxml to log
   `\meaning\xy@@ix@` at .xyc-input time and compare to ours.
2. Verify hypothesis (4): replace `remove_value("afterAssignment")`
   in `state::after_assignment()` with `lookup_value` +
   `assign_value(... Stored::None, Global)` and re-run min repro.
3. Verify hypothesis (1)/(2): patch Perl's TeX_FileIO write to log
   the bytes + Perl's `\toks` reader to log catcode of `@` at
   read-time. Compare with our trace.

* Acceptance: min repro → 0 errors AND full math0606553.zip → 0
  errors AND `cargo test --tests` 1109+/0/0.
* TDD test pair queued at `latexml_oxide/tests/graphics/xycompile.tex`;
  needs an `.xml` golden once fix lands.

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

### 4. `\lx@dual` recovery-recursion follow-up — regression test ✅ DONE

Done in commit `61bb505dc` — `tests/structure/math_dollar.{tex,xml}`.
Tests now 1110/0/0.

### 5. Sandbox conv_error long-tail — per-paper triage

**Round-17 deferred sandbox `~/data/10k_failures_April30/`** (35
in-scope papers, all Perl-clean under `--preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings`). Status 2026-04-30 evening:
**all 35 clean** in spot-check.

Final-mile fixes:
* `ce5c247e9` `_load_binding` UNLOCK + Perl-faithful
  `@lx@bibliography` parent counter rename.
* `b15870b34` `AddToMacro!` UNLOCK guard.
* `ef42bf0d0` `@add@frontmatter@now` text-mode digest
  (Perl `DigestText`).
* `8541d808d` `make_generic_message` `\@spaces` padding +
  `read_arg` isolated-mouth (Perl `make_message` /
  `readArg`-via-`readingFromMouth`).

Open: full 7898-paper sandbox rerun to verify no regressions
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

### 10. UNLOCKED scopes — DONE 2026-04-30

All 5 Perl `local $UNLOCKED = 1` sites translated:
`execute_before_digest` / `execute_after_digest` /
`execute_after_digest_body` (definition.rs), `_load_binding`
body (binding/content.rs), `AddToMacro!` (setup_binding_language.rs).
Plus explicit `=0` re-lock in raw TeX read (binding/content.rs).
Surgical `:locked` clear in revtex3_support removed in `4e800c537`.

### 11. Distribution — bundle multi-TL dumps

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
