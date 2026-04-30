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

### 4. `\lx@dual` recovery-recursion follow-up — regression test ✅ DONE

Done in commit `61bb505dc` — `tests/structure/math_dollar.{tex,xml}`.
Tests now 1110/0/0.

### 5. Sandbox conv_error long-tail — per-paper triage

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

### 10. (Long-term, low-priority) `_load_binding` UNLOCKED audit

Rust's `_load_binding` (`latexml_core/src/binding/content.rs:625`)
lacks Perl's `local $UNLOCKED = 1` wrapper around the binding-load
body (Package.pm:2318). Adding it is the Perl-faithful long-term
fix that would let sibling `.ltxml`/`.rs` bindings cleanly redefine
slots an earlier binding installed with `locked => true`. Trying
the wrapper alone regressed 5 unit tests (natbib_test, crazybib_test,
percent_test, textcase_test, amstheorem_test) — natbib's
`<bibliography>` element stops opening, `<bibitem>` ends up nested
inside `<para><p>0`. The "0" suggests a counter or value redef now
takes effect that previously was blocked, but which natbib downstream
state depends on staying blocked.

Workaround that ships today: surgical `:locked` flag clears in
`revtex3_support_sty.rs` immediately before its `\equation` redef
(commit `663895c56`). Other bindings with the same need can use the
same workaround until a wider audit completes.

Audit plan when revisited:
* Identify which bindings legitimately need to override locked
  slots (the broad fix DOES enable this).
* Identify which downstream binding state breaks under the broader
  fix and trace the actual failing redef-ordering.
* Fix the ordering issue forward, then enable the wrapper.
* Acceptance: `cargo test --tests` 1110+/0/0 with wrapper applied
  AND existing surgical workarounds removed.

Deprioritized — current sandbox-recovery work has higher leverage.

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
