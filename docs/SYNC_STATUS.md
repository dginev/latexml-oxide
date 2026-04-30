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
--path=~/git/ar5iv-bindings/bindings`). Status as of 2026-04-30
late-evening sweep: **12/35 clean** (was 10 at session start).
Cleared this session: cond-mat0110319, quant-ph0203083,
supr-con9608003 (`\font\gnuplot` picture cluster via
`gullet::read_keyword` Perl-faithful fix), cond-mat9911130,
math0007178 (latex209 `\sym<font>` stubs), astro-ph0611848
(`\longtab` env phantom def), hep-th9912229, cond-mat0103038
(`\documentstyle` re-bind after latex_dump). 1504.01713,
math9805021 clean since before.
Cleared this round-17 session: **math0004127** (`\math<class>
Digested` arg type, commit `9fa56e4c5`), **astro-ph0004127**
(NUL byte catcode IGNORE not ESCAPE in `state.rs:415`).

**Remaining 23 papers with errors — sweep 2026-04-30 latest:**

| Paper | Errs | Notes |
|---|---:|---|
| 1710.03688 | 1 | `unexpected:}` — `\begin{abstract}` mode-switch, French elsart |
| 1802.05444 | 1 | `undefined:\textrhookrevepsilon` — tipa T3 enc |
| 1804.04412 | 1 | `expected:}` — keyvals `readBalanced` unbalanced |
| astro-ph0512041 | 1 | `malformed:ltx:equation in <ltx:date>` — `\date{$$Id…$$}` schema |
| astro-ph9608077 | 1 | `malformed:ltx:tags` |
| hep-ph0702114 | 1 | `unexpected:}` (same as 1710.03688) |
| hep-th9601176 | 1 | `unexpected:double-superscript` — `\hspace`/`\hs` math-mode Tbox not reaching `pop_box_list` |
| math0111087 | 1 | `malformed:ltx:theorem` — amsppt `\proclaim` inside `\abstract` |
| math0606553 | 1 | `undefined:\lx` — xy.sty `\CompileMatrices` `.xyc` re-tokenization, in-progress |
| cmp-lg9407011 | 2 | `malformed:ltx:tags` |
| alg-geom9604001 | 2 | `malformed:ltx:equation` |
| 1806.06448 | 3 | `expected:}`/`expected:\fi` |
| 1812.01892 | 4 | `unexpected:\@personname` IEEEtran |
| astro-ph9808081 | 5 | `Unexpected:_` math-parser cumulative |
| astro-ph9903386 | 5 | `Unexpected:^` math-parser cumulative |
| math-ph0406029 | 8 | `unexpected:double-subscript` math-parser |
| math0411005 | 8 | filecontents extracted file processing |
| 1710.11409 | 9 | `malformed:ltx:section` schema; `\roman`/`\@ifundefined`/`\tag`/`\normalfont` |
| math-ph0303066 | 10 | `unexpected:double-subscript` |
| alg-geom9703018 | 10 | `\roman`/`\@ifundefined`/`\tag`/`\normalfont`/`\document`/`\lx@equation@settag` |
| 1806.08417 | 13 | `\seq@after`/`\delimsize`; brace tracking |
| alg-geom9604020 | 510 | amsppt cluster (deep `\proclaim` cascade) |
| math0601451 | timeout | amsppt `\proclaim` id3 cascade (`Iter-17` notes below) |

**Remaining 25 papers — error breakdown (older table, kept for context):**

| Paper | Errs | Top error pattern | Likely root cause |
|---|---|---|---|
| 1710.03688 | 1 | `unexpected:}` | `\begin{abstract}` mode-switch, French elsart |
| 1802.05444 | 1 | `undefined:\textrhookrevepsilon` | tipa raw .sty load (contrib binding fires but raw input fails) |
| 1804.04412 | 1 | `expected:}` | keyvals `readBalanced` unbalanced (Stage 5b parsing) |
| astro-ph0004127 | 1 | `undefined:\uninger` | Bisected 2026-04-30: trigger is `\bibliography` + paper-local `.bbl` processing (truncations not reaching `\bibliography` are clean). Tiny doc + full `.bbl` reproduces. The string "uninger"/"inger" is NOT in the bbl, so the CS is being synthesized during Rust's `\thebibliography`/`\bibitem` digestion, not literally read. Next: instrument the bibliography-building path (`before_digest_bibliography`, `\thebibliography` constructor) to find which macro emits `\uninger`. Note: bibstyle is `spiebib` (unknown — Info, not error) |
| astro-ph0512041 | 1 | `malformed:ltx:equation` | equation in `<ltx:date>` (schema/mode) |
| astro-ph9608077 | 1 | `malformed:ltx:tags` | `<ltx:tags>` schema malformed |
| hep-ph0702114 | 1 | `unexpected:}` | `\begin{abstract}` mode-switch (same as 1710.03688) |
| hep-th9601176 | 1 | `unexpected:double-superscript` | Bisected 2026-04-30: trigger is `\hspace`-between-superscript-and-prime in math mode. Min repro `$x_a^{\mu\nu}\hspace{2em}'(p)$` errors in Rust, clean in Perl. Even visible-width hspace fails — so it's not the `if !s.is_empty()` short-circuit in `\hspace OptionalMatch:* {Dimension}` (latex_constructs.rs:7700). The `\hspace` Tbox either isn't reaching `pop_box_list` in `script_handler` (tex_math.rs:99), or its `isSpace` property isn't being read at the math-mode boundary. Next: instrument `script_handler` to log popped-box properties on this input |
| ~~math0004127~~ | 0 | ~~`undefined:\oo`~~ | RESOLVED 2026-04-30: `\math<class>` constructors switched from `{}` (Plain) to `Digested` to mirror Perl `TeX_Math.pool.ltxml:689-697`. Plain expanded `\ifcase`'s body when there was no `{` after the CS, dragging `\oo` from the case-0 branch into evaluation. `\mathop` retained `{}` for now (`Digested` regresses `tests/math/testscripts` `scriptpos` depth-counting) |
| math0111087 | 1 | `malformed:ltx:theorem` | amsppt `\proclaim` inside `\abstract` (schema) |
| math0606553 | 1 | `undefined:\lx` | math-parser path during `\multline*`; `name`/`vattach` keyvals declared in `9d5cfb8ce` reduced Info noise but `\lx` source unidentified |
| alg-geom9604001 | 2 | `malformed:ltx:equation` | equation in text (schema) |
| cmp-lg9407011 | 2 | `malformed:ltx:tags` | schema mode-switch |
| 1806.06448 | 3 | `expected:}` `expected:\fi` | `\iffalse` not closed (post-`\end{document}` content) |
| 1812.01892 | 4 | `unexpected:\@personname` | IEEEtran `@IEEEauthorhalign` mode-switch |
| astro-ph9808081 | 5 | `Unexpected:_` | math-parser cumulative state |
| astro-ph9903386 | 5 | `Unexpected:^` | math-parser cumulative state |
| math0411005 | 8 | `undefined:\Trace,\DeclareMathOperator,…` | `\begin{filecontents}` extracted file processing |
| math-ph0406029 | 8 | `unexpected:double-subscript` | math-parser |
| 1710.11409 | 10 | `malformed:ltx:section` | schema |
| alg-geom9703018 | 10 | `malformed:ltx:tags` | schema |
| math-ph0303066 | 10 | `unexpected:double-subscript` | math-parser |
| 1806.08417 | 13 | `unexpected:}` | brace tracking |
| alg-geom9604020 | 510 | `expected:$` | amsppt cluster (deep `\proclaim` cascade) |
| math0601451 | 2414 | `malformed:ltx:XMTok` | amsppt `\proclaim` id3 cascade (see Iter-17 detail below) |

Min repros, deferred-fix notes, and per-paper diagnostics in
`~/data/10k_failures_April30/in_scope_papers.txt` and memory file
`memory/project_failures_april30_deferred.md`.



`results.tsv` has ~93 papers in the conversion_error bucket. Iter
39 sample of 12 random papers showed 2 fully clean on HEAD and 10
with 1-26 errors each. **Iter-41 deeper triage** ran Perl baseline
on 9 of those 10 (`alg-geom9604001`, `1710.03688`, `astro-ph9901170`,
`supr-con9608003`, `1504.01713`, `astro-ph0309636`, `astro-ph0310631`,
`1312.7418`, `astro-ph9608045`) — **all 9 are Perl-clean** = in scope.

**Key insight (iter-41):** for at least `1312.7418` (`\bullets` undefined)
and `supr-con9608003` (`\gnuplot` undefined), both Rust and Perl
use the same `Error('undefined', …)` site, but Perl reports
"Conversion complete: No obvious problems" while Rust reports
"1 error". Hypothesis: Rust *invokes* the undefined CS during
digest where Perl never reaches it. The trigger CS for `\bullets`
is `\renewcommand{\labelitemi}{$\bullets$}` — body is just stored,
not expanded, until `\labelitemi` is invoked. Either Rust eagerly
expands the body, or Rust's `\labelitemi` is invoked in a context
Perl's isn't. **Adding `\bullets`/`\gnuplot` stubs would mask
the trigger; the real fix is finding why Rust digests them.**

**Iter-55 — amsppt \\proclaim trigger identified:** Source pattern in
both papers is `\proclaim{Theorem N.M } Let $X$ be ...` where the
**body** has math. Rust's `\proclaim` impl (amsppt_sty.rs:147) and
Perl's are nearly identical:
```rust
DefConstructor!("\\proclaim Undigested DigestUntil:\\endproclaim",
    "<ltx:theorem class='ltx_theorem_proclaim'><ltx:title>#1</ltx:title>#2",
    after_construct => sub[doc,_a] { doc.maybe_close_element("ltx:theorem")?; });
```
Yet Perl handles it cleanly; Rust emits the body's `<ltx:XMTok>` *into*
`<ltx:title>` rather than after `</ltx:title>`. Suggests the Rust
constructor-template parser re-enters/re-emits inside the still-open
`<ltx:title>` element instead of properly closing it before #2. Needs
a focused trace of the constructor body insertion order. Would erase
~2924 sandbox errors (math0601451 + alg-geom9604020).

**Iter-17 (round-17 cont.) — refined trigger localized to id3 in
math0601451:** First two `\proclaim`s in the paper (Theorem 3.1
xml:id=id1, Theorem 5.1 xml:id=id2) emit clean `<theorem
class="ltx_theorem_proclaim">` with proper `<title>` containing only
"Theorem N.M" and a `<para><p>` body. The third `\proclaim` —
`\proclaim{Definition 2.1 } Let $X$ be a non-compact complete
orbifold. Then we say that...` at source line 398 — emits **broken
id3**: NO `class` attribute, `<title font="bold">` (font="bold" is
*not* in the constructor template), and the title text overruns into
"Definition 2.1  Let X be a non-compact complete orbifold" (the
entire first sentence of the body). Subsequent body text is correctly
in `<para><p>`. From id3 onward, deeper body math expressions begin
absorbing `<theorem>` elements as descendants of `<XMath>`, producing
the 1495 `XMTok-in-text`/`equation-in-text` cascade. The fact that
id3 has no class means it was NOT opened by amsppt's `\proclaim`
constructor — some other `<theorem>`-emitting CS ran instead. Fonts
"bold" suggests a `\noindent {\bf ...}` section heading is involved.
Hypothesis: between Theorem 5.1's `\endproclaim` (line 104) and
Definition 2.1 (line 398), state-leak from the dump pivot causes
a font-frame to leak; investigation deferred to a focused trace.

**Iter-17 root cascade origin:** The very first errors in the log
are at "Anonymous String" (1969 occurrences total) — emitted during
macro expansion before any user-document line is reached. First two
errors are `Attempt to close a group that switched to mode
display_math` and `Attempt to end mode display_math in display_math`.
This precedes ALL XMTok-in-text errors and is the original sin.
Some macro is emitting a `\lx@begin@display@math` followed by an
unmatched `}` close, leaving display-math state unbalanced from
process-start. amsppt's `\proclaim` body Token-stream may include
unbalanced `\par`/group tokens emitted by amstex format file. Need
to trace which macro definition fires display-math at expansion
time. Min repro elusive — naive 10-line `\proclaim{Theorem N}$$x=y$$
\endproclaim` produces 0 errors.

**Iter-54 amsppt cluster sized:** post `660103563` (the
\@startsection fix erased 1561 errors of 1608.04650), the next big
residual outliers are both amsppt+amstex papers:

| paper | Rust errs | Perl errs | dominant pattern |
|---|---:|---:|---|
| math0601451  | 2414 | 0 | 1481× XMTok-in-title |
| alg-geom9604020 | 510 | 4-warn | 172× XMTok-in-title |

Combined ~**2924 sandbox errors** stem from amsppt's title machinery
not wrapping math properly. Same root cause as the documented
siunitx XMTok-in-text issue but in an amstex context. amsppt-fix
candidates: amsppt_sty.rs title constructor, or AmSTeX.pool title
emission. Investigation queued — high-value cluster if tractable.

**Iter-53 — 1608.04650 root cause traced:** mst-stylefile.sty defines:
```
\newcommand\Proof{\@startsection{Proof}{5}{...}{-1em}{\normalsize\sc}}
\newenvironment{proof}{\Proof{Proof:}}{ \indent}
```
The user defines `\Proof` (capital) as a sectioning command via
`\@startsection{Proof}{5}{...}` — first arg is "Proof" (the counter
name). Rust's `\@startsection` impl uses arg #1 verbatim as the
ltx element tag → `<ltx:Proof>` (capital P). Schema only has
`<ltx:proof>` lowercase. So Rust opens an unknown element, and
all subsequent inserts (1561 of them) hit
`Error:malformed:* isn't allowed in <ltx:Proof>`.

Faithful fix: Rust's `\@startsection` should normalize the first
arg or map standard counter-names to ltx element tags via a lookup
table — mirroring whatever Perl's `\@startsection` does (likely
treats the first arg as a counter, not a tag name). 1561 sandbox
errors collapse with one fix.

**Iter-52 — 1608.04650 (1561-error outlier):** All 1561 errors share
the same root: `Error:malformed:* isn't allowed in <ltx:Proof>` —
754× `#PCDATA`, 593× `ltx:Math`, 80× `ltx:ref`, etc. Single content-
schema mismatch on the `<ltx:Proof>` element (capital P). Perl on
the same input emits 0 errors / 82 warnings, so Rust's `<ltx:Proof>`
content model is over-restrictive vs Perl's. Paper uses
mst-stylefile.sty (paper-local, no proof env redef) + amsthm-style
`\begin{proof}…\end{proof}`. Likely fix: locate the constructor that
emits `<ltx:Proof>` (capital) — should emit lowercase `<ltx:proof>`
to match the schema's content-model definition. Investigation queued.

**Iter-51 residual triage:**
- `1802.05444` (`\textrhookrevepsilon`): tipa.sty IS raw-loaded by
  `latexml_contrib/src/tipa_sty.rs`, but T3 phonetic encoding chars
  resolve via `\DeclareTextSymbolDefault{T3}` and need a t3enc.def
  shim that maps T3 slots → Unicode codepoints. Significant work
  (cluster of ~100 IPA glyphs). Deferred.
- `cmp-lg9407011`: `Error:malformed:ltx:tags isn't allowed in
  <ltx:tag>` — schema mismatch, deeper.
- `1806.06448`: `\fi` mismatch / `Gullet->readBalanced ran out of
  input` — conditional balancing, deeper.

**Iter-50 quantified impact of `db8a4815a`:** 30-paper random sample
from `results.tsv` conversion_error bucket on current HEAD:
**17 fully clean (`No obvious problems`)**, 1 warnings-only, 12
still error. **~60% recovered** from a previously-error bucket
just from the paper-local sty discovery fix. Full canvas rebuild
will likely show results.tsv conv_error count dropping from 93 to
~37 (38 papers worth ~+0.5% on the 7898-paper canvas). Residual
12-paper error pattern: 7 with 1-13 errors (per-paper long tail),
2 outliers (alg-geom9604020 510 errs, 1608.04650 1561 errs likely
amstex/amsppt cascade), 3 in the per-paper-stub bucket (`\gnuplot`,
`\textrhookrevepsilon`, `\seq@after`, `\delimsize`).

**Iter-49:** instrumented tex_fonts.rs with eprintln after the
DefPrimitive! call. Confirmed at install time `\gnuplot` IS in the
meaning table (`lookup_meaning("\\gnuplot") = true`). So the install
succeeds; the meaning is **lost between line 4 and line 5** of the
min repro. Strong evidence this is a frame-pop issue: either picture
env, or the `\font` body's parameter parsing (`SkipSpaces Token …
SkipMatch:= … TeXFileName`), opens a frame that gets popped before
the `\gnuplot` invocation. `\def` doesn't have parameter parsing
(it stores the body raw), so it's unaffected. Investigation has the
right anchor for next iteration: trace `push_frame` calls during
`\font`-body's parameter resolution.

**Iter-48:** confirmed `\global\font\gnuplot=...` ALSO fails (err=1)
but `\def\gnuplot{HELLO}` in the same picture-env body works (err=0).
So the bug is **NOT** generic local-scope-frame issue — it's specific
to `\font` primitive's CS-install path. `tex_fonts.rs:137` calls
`DefPrimitive!(cs, ...)` which dispatches to `def_primitive(cs, ...)`
inside picture's frame. Either (a) `def_primitive` for runtime-CS
(non-literal) installs into a different frame than `\def`, or (b)
something during `\font`-body parsing pops the current frame before
the install. Investigation continues — needs `def_primitive` source
read + frame-state trace.

**Iter-47 (after `db8a4815a`):** quant-ph0203083 still 1 error, but
the trigger is *different* from the iter-43 hypothesis. Source has
`\font\gnuplot=cmr10 at 10pt` followed by `\gnuplot` USE, **inside
a `\begin{picture}` block**. Min repro:
```tex
\documentclass{article}
\begin{document}
\begin{picture}(100,100)(0,0)
\font\gnuplot=cmr10 at 10pt
\gnuplot
\end{picture}
\end{document}
```
- Rust: 1 error (\gnuplot undefined at line 5)
- Perl: 0 errors

`\font` at top level works in BOTH engines. The bug is specific to
`\font` *inside `picture` env*. Both Perl FontDef.pm:42-43 and Rust
tex_fonts.rs:137 install the FontDef CS as **local** scope; yet
Perl handles the picture-frame correctly. Rust loses the def
between the same-frame `\font\gnuplot=...` line and the `\gnuplot`
invocation at the very next line. Suggests Rust's `\begin{picture}`
opens an additional frame around the body, or `\font` body's
DefPrimitive! installs into a frame that's already popped.
Investigation queued — needs picture-env framing trace.

**Iter-43 cluster (astro-ph + quant-ph):** sampled 4 more conv_error
papers with main-file detection: ALL are Perl-clean (0 errors / a
few warnings) and Rust-error on a small set of undefined CSes:

| paper | doc class/style | undefined in Rust |
|---|---|---|
| `astro-ph0607182` | `\documentstyle[…,ysc,…,epsf]{article}` | `\plotone` |
| `astro-ph0512041` | `\documentclass[…]{revtex4}` | (1 unspecified) |
| `astro-ph0611848` | `\documentclass{aa}` | (1 warn + 1 err) |
| `quant-ph0203083` | (TBD) | `\gnuplot` |

Perl detailed log on `astro-ph0607182` does NOT mention `\plotone`
at all — Perl never invokes it, even though `\plotone{fig1k.eps}`
sits inside a `\begin{figure}` body. Rust's
`latexml_package/src/package/aas_support_sty.rs` exists, but isn't
auto-loaded for plain `\documentstyle{article}` papers. Perl
similarly doesn't auto-load `aas_support.sty.ltxml` for plain
article — so the Perl tolerance is from a different mechanism
(maybe `\documentstyle` 2.09-compat treats undefined CSes inside
floats as text, or `\begin{figure}` body in plain article
swallows unknown CSes silently).

**Action plan:** add `Sub-task 5a` — investigate `\documentstyle`
2.09-compat path's handling of undefined CSes inside float
environments (`figure`, `table`). The 9-paper iter-41 sample +
4-paper iter-43 sample suggests a sizeable cluster of Rust-only
errors stem from this. Until the root divergence is found,
do NOT add per-paper stubs (would mask the real bug).

**Iter-42 update:** **min repro confirmed** the divergence:
```tex
\documentclass{article}
\renewcommand{\labelitemi}{$\bullets$}
\begin{document}
\begin{itemize}\item One\end{itemize}
\end{document}
```
On THIS minimal input, BOTH Perl AND Rust report 1 error / 1
undefined macro for `\bullets`. So at the min-repro level, both
engines agree. Yet on the full Centralisateur.tex paper Perl
reports "0 errors / 86 warnings" while Rust reports the error.
The full paper differs from the min repro by 86 warnings worth
of math-parser issues, hundreds of `\newcommand`s, multi-file
`\input{}` chain. Something in that environment makes Perl skip
the `\labelitemi` expansion. Investigation parked — the
divergence is real but not min-reducible from a top-down bisect
in 5 minutes. Next iteration: bisect by progressively stripping
preamble macros from Centralisateur.tex until Perl-vs-Rust
classification flips.

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
