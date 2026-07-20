# Known Errors in Upstream Perl LaTeXML

This file documents issues in the original Perl LaTeXML codebase.
These are upstream behaviors or design quirks — NOT bugs introduced by the Rust port.
For Rust-specific error bookkeeping, see `docs/SYNC_STATUS.md`.

---

## 1. `packParameters` spurious warning for alignment templates

**Perl source:** `LaTeXML/Core/Tokens.pm` lines 122–142

**Symptom:** Documents trigger:
```
Error:misdefined:expansion  Parameter has a malformed arg, should be #1-#9 or ##.
```

**Root cause:** `packParameters()` is called on all `\def`/`\edef` bodies.
When a body contains an alignment template like `\halign{#\hfil&...}`, the
`#` is the alignment cell marker — valid TeX. But `packParameters` expects
`#` followed by a digit (`#1`–`#9`) or `##`. A `#` followed by CS (e.g.
`\hfil`) hits the error branch.

**Minimal example:**
```tex
\def\foo{\halign{#\hfil\cr test\cr}}
```

**Impact:** Non-fatal in principle — but Perl's branch emits a *counted* `Error`
**and drops both tokens**, corrupting the template. Perl rarely reaches it
because it often can't find the offending package and skips the raw load; we
*do* raw-load such packages, so it broke the error-free target for the common
halign-in-macro idiom (e.g. easyeqn.sty's `{MATRIX}` env → `$\mathstrut##$`).

**Perl status:** Still present (Tokens.pm line 139). Unfixed upstream.

**Rust status (FIXED 2026-05-28, beneficial divergence):** `pack_parameters`
(`latexml_core/src/tokens.rs`) now **preserves** the `#` and the following
token losslessly (so the alignment template / `#{` delimiter survives) and logs
at `Info` (non-counted) instead of `Error`. Real TeX resolves the
PARAM-vs-alignment-cell ambiguity during alignment processing, below the level
LaTeXML operates at, so a genuine typo can't be reliably told apart — preserving
+ Info is strictly more faithful to TeX than erroring + dropping. Witness
2006.02269 (easyeqn `{MATRIX}`): 2 errors → 0. cargo test 1344/0/0.

---

## 2. `\fontname` returns synthesized font descriptor, not TeX-native format

**Perl source:** `LaTeXML/Engine/TeX_Fonts.pool.ltxml`

Perl's `\fontname` returns a string constructed from the Font object. It may
not match what TeX engines produce (e.g. `"select font cmr10 at 5.0pt"`).
The format depends on how the font was loaded and what the Font struct retains.

---

## 3. `\hyphenchar` is not truly per-font

**Perl source:** `LaTeXML/Engine/TeX_Fonts.pool.ltxml`

In real TeX, `\hyphenchar\myfont=99` sets the hyphenchar only for `\myfont`.
LaTeXML's font model is higher-level (family/series/shape/size) rather than
per-font-instance. The `\hyphenchar` implementation stores values in state
keyed by font command name, but grouping interactions may not perfectly match.

---

## 4. Font `specialize()` can reset explicit font properties

**Perl source:** `LaTeXML/Common/Font.pm`, `specialize()` method

`specialize($text)` examines Unicode properties to infer font characteristics.
For "Other Symbol" characters, it resets `series` to "medium" and `shape` to
"upright". If called with unexpected input (e.g. font filenames classified as
"Other Symbol"), it overwrites explicitly-set properties like `series="bold"`.

Perl avoids the worst case because `merge()` doesn't call `specialize` by
default. But the underlying logic can still produce surprising results.

---

## 5. `readBalanced` cannot distinguish parameter `#` from alignment `#`

**Perl source:** `LaTeXML/Core/Gullet.pm`, `readBalanced()` with `$macrodef=1`

When reading a macro body, `$macrodef=1` triggers `packParameters()` on the
result. This is correct for normal bodies but fires spurious warnings (see
item 1) when the body contains alignment templates.

The issue is architectural: both parameter markers and alignment cell
placeholders use catcode 6 (PARAM). Real TeX resolves this during `\halign`
processing at a lower level. LaTeXML processes TeX at a higher abstraction
level and cannot distinguish the two uses.

---

## 6. `guessTableHeaders` heuristic can fire unexpectedly

**Perl source:** Post-processing pipeline

LaTeXML applies a heuristic to guess header rows in tabulars, adding
`<thead>`, `thead="column"` attributes, and `class="ltx_guessed_headers"`.
This is an accessibility enhancement, not LaTeX semantics. The heuristic
can produce different results than manual markup and may fire on tables
where no header was intended.

---

## 7. `alignment_skip_data` continuation-line logic is dead code

**Perl source:** `LaTeXML/Core/Alignment.pm` line 1339

**Symptom:** The heuristic that allows "continuation lines" (mostly-empty
data rows) to be accepted despite exceeding the threshold never actually
fires.

**Root cause:** The continuation check compares:
```perl
scalar(grep { $$_{content_class} eq '_' } @{ $::TABLINES[$i + $n] })
  <= 0.4 * scalar($::TABLINES[0])
```
`$::TABLINES[0]` is an array reference. `scalar($::TABLINES[0])` returns
the reference itself, which in numeric context evaluates to its memory
address (a huge number like ~140 trillion on 64-bit). So `0.4 * scalar(...)`
is always enormous, and the `<=` comparison is always TRUE.

The intended code was almost certainly:
```perl
0.4 * scalar(@{$::TABLINES[0]})  # count of cells in first line
```

**Effect:** `alignment_skip_data` effectively breaks on ANY comparison that
exceeds the threshold — no continuation lines are ever accepted. This makes
the data-block scan more conservative (shorter blocks), which in turn makes
the header heuristic less likely to succeed on borderline cases.

**Rust fix:** Match the Perl behavior — break immediately when diff >=
threshold. The continuation-line logic is commented out with a reference
to this entry.

---

## 8. `NewScript` XMDual content arm uses meaningless `Apply(∅, XMRef)` for subscripted identifiers

**Perl source:** `LaTeXML/MathParser.pm` line 1637, `NewScript()` function

**Symptom:** When a subscripted expression like `f_1` is assigned `role="ID"` via
`DefMathRewrite`, the math parser wraps it in `XMDual`. The presentation branch
correctly shows the subscript structure (`SUBSCRIPTOP + f + 1`). But the content
branch contains:

```xml
<XMApp>
  <XMTok/>                              <!-- empty/absent operator -->
  <XMRef idref="S0.Ex4.m1.1"/>          <!-- reference to subscript value "1" -->
</XMApp>
```

This is `Apply(∅, 1)` — applying a nonexistent operator to just the subscript
value. It is **not mathematically meaningful**. An identifier `f₁` should be
represented as a single atomic token (a skolem constant), e.g.:

```xml
<XMTok name="f_1" role="ID"/>
```

or simply left as the flat subscript structure with `role="ID"`:

```xml
<XMApp role="ID">
  <XMTok role="SUBSCRIPTOP" scriptpos="post1"/>
  <XMTok>f</XMTok>
  <XMTok meaning="1" role="NUMBER">1</XMTok>
</XMApp>
```

**Root cause:** `NewScript()` always creates `Apply(SCRIPTOP, base, script)` for
the presentation branch. The XMDual content branch is constructed mechanically
by extracting `Arg($script, 0)` and wrapping in `Apply(empty_tok, XMRef)`. This
pattern works for operators where the subscript carries semantic meaning (e.g.,
`∑_i` → `Apply(sum, i)`), but for plain identifiers (`f_1`) the subscript is
just a name component, not an argument.

**Minimal example:**
```tex
% In .latexml file:
DefMathRewrite(match => 'f_\WildCard', attributes => { role => 'ID' });
% In .tex file:
$f_1(a+b)$
```

**Impact:** Content MathML generation would produce `<apply><csymbol/><cn>1</cn></apply>`
instead of `<ci>f₁</ci>`. No known downstream breakage because content MathML
is rarely consumed for such tokens, but semantically incorrect.

**Rust fix:** Rust produces the flat `XMApp[role="ID"]` form without XMDual.
The test XML is updated to match the Rust output. This is an intentional
divergence — the Rust form is semantically cleaner (no meaningless `Apply(∅, ref)`).
If XMDual is needed later, the content branch should use a skolem `XMTok[name="f_1"]`.

---

## 9. `addOpArgs` narrow bigop absorption in declare test

**Perl source:** `LaTeXML/MathGrammar` lines 668-672, `addOpArgs` / `moreOpArgFactors`

**Symptom:** In `f(x) = \sum_{i=0}^{\infty} f_i x^i`, Perl's Parse::RecDescent
parser produces `∑(f_i) * x^i` — the sum absorbs only `f_i`, not `f_i * x^i`.
This is mathematically wrong: `i` is the summation variable, so `x^i` must be
inside the summand. The correct parse is `∑(f_i * x^i)`.

**Root cause:** `moreOpArgFactors` in Parse::RecDescent tries alternatives in
order. After absorbing `f_i`, the next token `x^i` could extend the chain via
invisible times (`Factor moreOpArgFactors`). But Parse::RecDescent's
backtracking and top-down evaluation means the "stop absorbing" alternative
(`{ $arg[0]; }`) can win depending on the context. The result is
non-deterministic — the narrow parse happens to be selected for this specific
expression.

**Perl expected XML:** `text="... ((sum _ (i = 0)) ^ infinity)@(f _ i) * x ^ i"`

**Correct parse:** `text="... ((sum _ (i = 0)) ^ infinity)@(f _ i * x ^ i)"`

**Rust fix:** Rust's `bigop_application` nonterminal at expression level absorbs
the full `term` (factor chain with mulop/invisible-times). The declare test XML
is updated to match the mathematically correct broad absorption.

---

## 10. Quantifier period-binding parsed as formulae split

**Symptom:** `\exists x. P(x)` is parsed as `formulae@(exists@(x), P*x)` — two
separate formulas separated by a period. The correct mathematical reading is
`exists@(x, P(x))` — a bound quantifier where the period separates the bound
variable from the body (the predicate `P(x)`).

**Root cause:** Perl's MathGrammar treats `.` as a ColRHS (column-right-hand-side)
separator, which creates a `formulae` structure splitting `exists@(x)` from `P*x`.
The grammar has no special handling for quantifier-period-body patterns like
`\exists x. P(x)` or `\forall \epsilon > 0. \exists \delta > 0. |x - a| < \delta`.

**Perl expected XML:** `text="formulae@(exists@(x), P * x)"`

**Correct parse:** `text="exists@(x, P(x))"` — the period should bind the quantifier's
variable to its body, similar to how `\int f(x)\,dx` binds the integral to its
integrand and differential.

**Rust status:** Currently unparsed (`ltx_math_unparsed`). Future fix should add
quantifier-period-body grammar rules rather than mimicking Perl's incorrect
formulae split.

## 11. `io.tex` produces `Error:unexpected:}` from unmatched braces in `\read` content

**Perl source:** `Stomach.pm` L336–340 (`egroup()`)

**Symptom:** The io digestion test reads `exists.data` which contains:
```
line { with extra } } silently discards }
```
When `\read` stores this line in `\aline` and it's expanded, the `{` opens a group
(switching to horizontal mode), the first `}` closes it, but the second `}` finds
a mode-switch frame and triggers:
```
Error:unexpected:} Attempt to close a group that switched to mode horizontal
```

**Root cause:** Both Perl and Rust LaTeXML's `\read` implementation do not fully
match standard TeX behavior. In standard TeX/pdflatex, `\read` auto-balances
braces: it continues reading lines until braces are balanced, and silently discards
any tokens after a balanced top-level group. So line 21 of `exists.data`
(`line { with extra } } silently discards }`) would have the trailing ` } silently
discards }` discarded by `\read`, and `\aline` would contain only balanced content.

In LaTeXML (both Perl and Rust), `\read` does not implement this auto-balancing.
It reads the line literally, producing unbalanced content. When `\showline`
expands `\aline`, the extra `}` triggers `egroup()` which checks
`isValueBound('BOUND_MODE', 0)` and reports an Error for the mode-switch frame.
This is correct error-reporting for the actual (unbalanced) content, but the real
bug is the incomplete `\read` implementation.

**Perl also errors:** Yes — running Perl's LaTeXML on `io.tex` with `verbosity=>5`
produces the exact same 2 `Error:unexpected:}` messages. The Perl test suite
passes because these errors are logged to an internal report, not printed to stderr.
The test passes in both because the expected XML was generated with this same bug.

**Rust status:** Identical behavior — 2 `Error:unexpected:}` messages. These are
expected and match Perl. A future `\read` brace-balancing fix would eliminate these
errors, but it would also change the test output (requiring XML updates).

---

## 12. `SVGNextObject()` timing inconsistency between clipPaths and shadings

**Perl source:** `pgfsys-latexml.def.ltxml` lines 348, 371, 674, 699

**Symptom:** In Perl, `SVGNextObject()` is called from `properties` closures for both
clipPaths (lines 348, 371) and shadings (lines 674, 699). Properties closures run
during the **digestion** phase, so the counter increments in document order (clip1,
shade2, clip3, shade4...). This is correct but **fragile** — it relies on properties
closures having the same execution timing as `DefPrimitiveI` bodies.

If clipPaths used a constructor body instead of a properties closure (natural for
imperative DOM manipulation), the counter would increment during construction phase
instead of digestion, breaking the interleaving. Perl's design accidentally works
because Perl's DefConstructor template-based approach naturally uses properties for
computed values.

**Impact:** None in Perl (the timing happens to be correct). In the Rust port,
initially placing `svg_next_object()` in the constructor body (construction phase)
caused all shading IDs to be assigned before clipPath IDs, breaking the interleaving.
Fixed by matching Perl's properties-based approach.

**Rust fix:** Moved `svg_next_object()` to `properties` closures for clipPath
constructors (`\lxSVG@drawpath@clipped`, `\lxSVG@discardpath@clipped`), matching
Perl's digestion-phase counter increment timing.

---

## 13. Duplicate xml:id generation for `\subequations` after `\addtocounter{equation}{-1}` inside theorem with shared `equation` counter

**Perl source:** `LaTeXML/Package/amsmath.sty.ltxml` (subequations environment)
plus shared-counter interaction with `\newtheorem{thm}[equation]{...}`.

**Symptom:** Documents with the pattern:
```tex
\newtheorem{thm}[equation]{Theorem}
...
\begin{thm} \label{...}
...
\end{thm}
\addtocounter{equation}{-1}
\begin{subequations}
\begin{equation}\label{eq:foo}
...
\end{equation}
\end{subequations}
```
trigger `Info:malformed:id Duplicated attribute xml:id` warnings in Perl LaTeXML.
The preceding theorem got xml:id e.g. `S5.E2` (via the shared equation counter);
the following subequations' equationgroup, after the `\addtocounter{-1}`,
tries to use the same number and claims `S5.E2` as well.

**Minimal trigger:** arxiv 1106.1389 (5 duplicate-id Info warnings in both
Perl and Rust post-fix; Perl reports 14 sites but dedups them correctly too).

**Impact in Perl:** non-fatal (Info-level warnings only) — `modifyID` appends
`a`, `b`, … suffixes so the DOM ends up with unique xml:ids.

**Impact in Rust (post-session-128 fix):** matches Perl — same 5 Info warnings,
same deduped DOM. Prior to session 128, `record_id_with_node` had a shadow-
variable bug (`let id = self.modify_id(…)` scoped to the `if let Some(prev)`
block only) that caused the deduped id to be silently dropped; the caller
wrote the original id to DOM and libxml2 validation subsequently spun
O(n²) on the actual duplicates (100s timeout / 16 GB RSS on 1106.1389).
Fixed in commit `bab8beb53`: extract `final_id` outside the `if let`.

## 14. `eurosym.sty.ltxml` declares `gennorrow` option (typo for `gennarrow`)

**File:** `lib/LaTeXML/Package/eurosym.sty.ltxml` L28.

Perl:
```perl
DeclareOption('gennorrow', undef);
```

Upstream eurosym.sty uses `gennarrow` (narrow variant of the generic
euro symbol). The Perl declaration is a typo — any user writing
`\usepackage[gennarrow]{eurosym}` falls through to the default option
handler instead of the registered no-op.

**Rust behavior:** the Rust port (eurosym_sty.rs) declares both
`gennarrow` (for correct user input) and `gennorrow` (Perl-parity).
Both are no-ops in either form, so the practical impact is only
log-order: Perl's log says "gennarrow is unknown, using default",
Rust's says "gennarrow matched option, processed".

---

## 15. `revtex4_support.sty.ltxml` `\eqnum` body references `#2` with only one parameter

**Perl source:** `LaTeXML/Package/revtex4_support.sty.ltxml` L172

```perl
DefMacro('\eqnum {}', '\lx@equation@settag{\edef\theequation{#2}\lx@make@tags{equation}}',
  locked => 1);
```

**Root cause:** The signature `\eqnum {}` declares one required argument
(`#1`), but the expansion references `#2`. `#2` is out of range and
substitutes undefined/empty — so the `\edef` assigns an empty string to
`\theequation`, and `\lx@make@tags{equation}` then emits whatever
`\theequation` was before the body fired (likely the counter default).

**Impact:** `\eqnum{foo}` in revtex4 docs always tags the equation with
the counter value, never with the user-supplied label. Intended was
probably `#1`.

**Perl status:** Still present. Unfixed upstream.

**Rust behavior:** `revtex4_support_sty.rs` defines `\eqnum{}` → `""`
(silently drops the label). Semantically equivalent to Perl's buggy
`#2`-is-empty behavior — both lose the user label. A faithful "fix
Perl's typo" port using `#1` would be a deliberate divergence from
upstream.

---

## 16. `aipproc.cls.ltxml` `\tablenote` body references `#1` (star flag) instead of `#2` (content)

**Perl source:** `LaTeXML/Package/aipproc.cls.ltxml` L101

```perl
DefMacro('\tablenote OptionalMatch:* {}', '\footnote{#1}');
```

**Root cause:** The signature `OptionalMatch:* {}` occupies two
positional slots — `#1` is the star flag (literal `*` or undef),
`#2` is the required `{}` content. The body expands `\footnote{#1}`
which passes the *star marker* (or empty) to `\footnote`, silently
dropping the user's note content. The same file on L100 uses
`\tablehead{}{}{}{}` → `\multicolumn{#1}{#2}{\parbox{#3}{#4}}` where
the #N indexing is correct — so this is a localized typo.

**Confirming convention:** other ltxml files using the same signature
index content at `#2`. For example, `physics.sty.ltxml` L356:

```perl
DefMacro('\qqtext OptionalMatch:* {}', '\mbox{\ifx.#1.\quad\fi#2\quad}');
```

Here `#1` is explicitly tested as the star flag (`\ifx.#1.`) and `#2`
is the content, proving the star occupies slot 1.

**Impact:** `\tablenote{note}` in aipproc conference papers expands to
`\footnote{}` (empty footnote) instead of `\footnote{note}`. The note
body is lost; only the footnote marker remains.

**Perl status:** Still present. Unfixed upstream.

**Rust behavior:** `aipproc_cls.rs` L115 uses `\footnote{#2}` —
semantically correct. A faithful port of Perl's buggy `#1` would
silently lose note content; the Rust port deliberately diverges by
indexing the content correctly. The sibling `elsart_support_core.sty`
`\collab OptionalMatch:* {}` → `\author{#1}` exhibits the same
pattern; `elsart_support_core_sty.rs` L135 likewise deliberately uses
`#2` so the author name reaches `\author` (fix cycle 172).

## 17. `titling.sty.ltxml` `\symbolthanksmark` redefined two lines later

**Perl source:** `LaTeXML/Package/titling.sty.ltxml` L39 + L41

```perl
DefMacroI('\symbolthanksmark', undef, '\fnsymbol');        # L39
DefMacro('\thanksmarkseries{}',  '');                       # L40
DefMacro('\symbolthanksmark',    '');                       # L41 — overrides L39
```

**Root cause:** `\symbolthanksmark` is defined twice in consecutive
statements. The second definition (empty body) always wins, so the
first (`\fnsymbol` alias) is unreachable dead code.

**Confirming convention:** the Perl `DefMacro`/`DefMacroI` pairing
writes to the global state directly with no guard against prior
definitions — the second call replaces the first unconditionally.

**Impact:** Users of `\symbolthanksmark` get an empty expansion rather
than the `\fnsymbol` numbering the first (abandoned) definition
suggested. Likely a stale edit: either L39 was meant to be removed or
L41 was meant to apply to a different CS.

**Perl status:** Still present. Unfixed upstream as of the 2026-03 sync.

**Rust behavior:** `titling_sty.rs` ports only the second (empty)
definition — matches Perl's effective observable behavior. Preserving
both would be bit-identical but would also preserve the dead code; the
Rust port intentionally elides the shadowed L39.

---

## 18. `numprint` `\lenprint` — test reference is stale relative to current Perl

**Perl source:** `LaTeXML/lib/LaTeXML/Package/numprint.sty.ltxml`

**Symptom (revised 2026-04-28):** `tests/babel/numprints.xml` is
heavily out-of-date relative to current Perl output. Verified via
side-by-side run:
* Test reference: 91 lines (truncated, presumably from a much older
  Perl that errored at `\lenprint{\textwidth}`)
* Current Perl output: **1689 lines** (`\lenprint` renders fully with
  `<Math mode="inline" tex="\numprint[pt]{433.62}">…</Math>`)
* Rust output: 622 lines (also renders `\lenprint` fully, structurally
  similar to current Perl with some flat-vs-nested XMTok differences
  inherited from the math-parser divergence)

**Status:** The earlier rationale ("Perl baseline errors out, don't
refresh test XML") no longer applies — Perl no longer errors. Both
Rust and current Perl render the full content. The remaining gap is
math-parser structural differences (XMApp-nested vs flat XMTok), which
is the documented `KNOWN_PERL_ERRORS #8` (f_1 flat XMApp[role=ID])
class of divergence — not specific to numprint.

**How to apply:** When the math-parser nested-XMTok divergence is
addressed, regenerate the test reference from current Perl. Until
then, `numprints_test` remains documented as failing for
math-parser-deep reasons.

---

## 19. TL2025 babel-french `frenchb` deprecation shim breaks Perl

**TL source:** `texmf-dist/tex/generic/babel-french/frenchb.ldf`
(babel-french 3.7e, 2025-08-15).

**Symptom:** `\usepackage[frenchb]{babel}` (or any paper passing the
deprecated `frenchb` option) on Perl LaTeXML with TL2025 emits:
```
Error:undefined:\bbl@main@language … is not defined.
Error:latex:(babel) Package babel Error: You haven't defined the
language '\bbl@main@language' yet.
```

**Root cause:** TL2025's `frenchb.ldf` is a 30-line deprecation
shim that does `\chardef\l@frenchb=\l@french` and
`\def\CurrentOption{french}` but does NOT chain `\input french.ldf`.
Perl LaTeXML's `frenchb.ldf.ltxml` loads the shim raw and then
relies on the never-firing chain.

**Minimal example:**
```tex
\documentclass{article}
\usepackage[frenchb]{babel}
\begin{document}
Bonjour.
\end{document}
```

**Verification (2026-04-29):** Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings`
emits 2 errors on this 4-line min repro. Same paper produces
2 errors on `0909.3444` (taln09 conference paper).

**Impact:** Affects any paper using the deprecated `frenchb` option.
Mostly older arXiv submissions written before babel-french 3.x
mainstreamed `\usepackage[french]{babel}`.

**Rust port status:** Rust now SUPERSEDES Perl on this — round-17
commit `989c5a8ed` adds babel-level `\l@frenchb` + caption/extras/
date hook aliases in `french_ldf.rs::load_definitions`, so
`\selectlanguage{frenchb}` resolves silently. Rust converts
0909.3444 with 0 errors; Perl baseline still emits 2.

---

## 20. `AmSTeX.pool.ltxml` `\italic`/`\slanted`/`\boldkey` font hash duplicate keys

**Perl source:** `LaTeXML/Engine/AmSTeX.pool.ltxml:278-286`

**Symptom:** Three AmSTeX font commands have duplicate hash keys in
their `font => { ... }` argument, so the second value silently
overwrites the first:

```perl
DefConstructor('\italic{}', '#1', ...,
  font => { shape => 'italic', series => 'medium', shape => 'upright' });
DefConstructor('\slanted{}', '#1', ...,
  font => { shape => 'slanted', series => 'medium', shape => 'upright' });
DefConstructor('\boldkey{}', '#1', ...,
  font => { series => 'bold', family => 'typewriter',
            series => 'medium', shape => 'upright' });
```

In Perl `{}` is a hash literal; later keys overwrite earlier ones.
So `\italic`, `\slanted`, and `\boldkey` end up applying:

| CS | Effective shape | Effective series | Effective family |
|---|---|---|---|
| `\italic` | upright (NOT italic) | medium | inherited |
| `\slanted` | upright (NOT slanted) | medium | inherited |
| `\boldkey` | upright | medium (NOT bold) | typewriter |

**Root cause:** Looks like a copy-paste error — the `'upright'` was
likely meant to override the prior `\bold`-derived font that wraps
the macro. But because the keys are the same name (not e.g. a hash
merge), the original `italic`/`slanted`/`bold` settings are lost.

**Impact:** The three CSes don't render in the intended style under
Perl. AmSTeX papers using `\italic{...}` get upright, not italic.
Real-world impact is minor since these CSes are rarely used directly
in modern papers (most authors just write `\textit{...}` or use
`amsmath` macros).

**Rust port status:** Rust DIVERGES from Perl here intentionally.
`amstex.rs:258-269` keeps only the *first* shape/series value
(the obviously-correct one):
* `\italic` → shape: italic, series: medium
* `\slanted` → shape: slanted, series: medium
* `\boldkey` → series: bold, family: typewriter, shape: upright

This produces visually correct output. If strict Perl-bug parity is
ever needed, swap the values to match the Perl typo's effective
behavior (use `upright`/`medium` everywhere); it would be a regression
in rendering quality, so the divergence stays.

---

## 21. `AmSTeX.pool.ltxml` missing `\edef\@{\string @}` from amstex.tex L165

**Perl source:** `LaTeXML/blib/lib/LaTeXML/Engine/AmSTeX.pool.ltxml` (no `\@` definition)

**Symptom:** AmSTeX documents (`\input amstex` + `\documentstyle{...}`)
that embed email addresses as `user\@host.tld` report:
```
Error:undefined:\@ The token T_CS[\@] is not defined.
```
The conversion bails before producing usable XML.

**Root cause:** `amstex.tex` line 165 redefines `\@` (which TeX/plain
binds as a sentence-end no-op) to expand to the literal character `@`
via `\edef\@{\string @}`. This is the canonical AmSTeX way to write
an at-sign — used pervasively for emails in author-address blocks.
Perl LaTeXML's `AmSTeX.pool.ltxml` does not mirror this redefinition,
so `plain_base.pool.ltxml`'s `DefConstructor('\@', '')` (which absorbs
`\@` to empty) stays in effect. Then `amsppt.sty`'s subsequent
`\let\@sf\empty@\relaxnext@` chain (lines 788/807) — or the user's
inline `\@` — looks up the bare `\@` later and reports it as
undefined / produces malformed output.

**Minimal example:**
```tex
\input amstex
\documentstyle{amsppt}
e-mail: ramm\@math.ksu.edu
\bye
```

**Impact:** 36 papers across staged_canvas runs (math-ph0001012/15,
math0209244, math0311498, …, 2012.06011, 1809.08150) fail because of
this single missing redefinition. All match the AmSTeX-email
signature.

**Rust resolution:** Mirror `amstex.tex` directly in `amstex.rs`:
```rust
DefMacro!("\\@", "@");
```
(Perl-equivalent literal translation of the canonical AmSTeX source —
faithful to the upstream `.tex` file, divergent only from Perl
LaTeXML's incomplete pool.) Fixed at commit time; all 36 sampled
witnesses now convert with 0 errors.

## 22. `\altaffiliation` missing optional `[note]` arg in `revtex4_support.sty.ltxml`

**Perl pattern (revtex4_support.sty.ltxml):**
```perl
DefMacro('\affiliation{}',  '\@add@to@frontmatter{ltx:creator}{\@@@affiliation{#1}}');
DefMacro('\altaddress',     '\altaffiliation');
DefMacro('\altaffiliation', '\affiliation');
```

**Real REVTeX4 semantics:** `\altaffiliation[note]{address}` accepts an
optional leading note (typical `[Also at ]`) that is prepended to the
address text. Perl's binding drops the `[]` from the signature, so the
TeX parser reads the `[` token as `#1` of `\affiliation{}`, emitting a
bare literal `[` into `<ltx:contact role='affiliation'>` and dumping
the rest of the note (`Also at ]`) into the author-name slot.

**Witness:** physics0210041 (stage 3 sweep). Source:
```tex
\author{Lars Egil Helseth}
\address{Max Planck Institute of Colloids and Interfaces, D-14424 Potsdam, Germany}%
\altaffiliation[Also at ]{Department of Physics, University of Oslo, ...}%
```
Output before fix:
```html
<span class="ltx_contact ltx_role_affiliation">Max Planck Institute …</span>
<span class="ltx_contact ltx_role_affiliation">[</span>
```

**Rust resolution:** `latexml_package::revtex4_support_sty` now uses
`\altaffiliation[]{}` with body `\@add@to@frontmatter{ltx:creator}
{\@@@affiliation{#1#2}}`; same shape on `\altaddress`. When no
optional `[]` is present, `#1` is empty and the original single-arg
behaviour is recovered. SURPASS-PERL.

## 23. `article.cls.ltxml` `\Huge` defined as 29.8 pt — diverges from LaTeX's 24.88 pt

**Perl pattern (`Package/article.cls.ltxml`, also `book.cls.ltxml`,
`slides.cls.ltxml`):**
```perl
DefPrimitiveI('\Huge', undef, undef, font => { size => 29.8 });
```

**Real LaTeX (`article.cls` 10pt option):**
```tex
\renewcommand\Huge{\@setfontsize\Huge{24.88}{30}}
```

At a 10pt body, real LaTeX `\Huge` is 248.8% of the base; Perl emits
298%, an extra ~20% in size. Visible whenever an author uses `\Huge`
to scale subfigure panel labels — they come out noticeably larger
than the kerned typography of a typesetter would produce.

Cross-check: Perl's own `Common/Font.pm` declares `Huge => 2.488`
(semantic-name table, matching LaTeX). The `.cls.ltxml` size override
of 29.8 is the inconsistency.

**Witness:** cond-mat0301062 §S4.F2 / F3 — `\centerline{\Huge (a)}` /
`\centerline{\Huge\bf (b)}` subfigure markers render at
`font-size:298%`. Both Perl and Rust output 298%.

**Rust resolution:** *not yet patched.* Tracking as Perl-faithful
divergence from real LaTeX. Switching `\Huge` to 24.88 in
`article_cls.rs`/`book_cls.rs`/`slides_cls.rs` would be a SURPASS-PERL
change correcting the font scaling to match LaTeX defaults; safe
because the `Common/Font.pm` semantic value already encodes 24.88.
Open for future round if visual quality matters more than Perl-test
parity.

## 24. `latex_constructs.pool.ltxml` `\@evenfoot` defined twice (typo for `\@evenhed`)

**Perl source (`Engine/latex_constructs.pool.ltxml` L1254-1257):**
```perl
DefMacroI('\@oddfoot',  undef, Tokens());
DefMacroI('\@oddhed',   undef, Tokens());
DefMacroI('\@evenfoot', undef, Tokens());
DefMacroI('\@evenfoot', undef, Tokens());
```

L1255 is `\@oddhed` (abbreviated from kernel `\@oddhead`). By the
oddfoot/oddhed pattern, L1257 was clearly intended to be `\@evenhed`
— defining the matching abbreviated stub. Instead it's a verbatim
duplicate of L1256, leaving `\@evenhed` undefined while `\@evenfoot`
is redundantly defined twice.

**Impact:** Functionally zero — `\@oddhed` / `\@evenhed` are
LaTeXML-internal stubs that nothing references (the kernel uses
`\@oddhead` / `\@evenhead`). The duplicate `\@evenfoot` Def just
overwrites itself identically.

**Rust resolution:** kept the duplicate to match Perl exactly,
including in dump output (Perl emits `\@evenfoot` 3× in
`latex_dump.pool.ltxml`). No fix because no observable behavior
diverges. Documented here in case future Perl-side audit fixes it.

---

## 25. `latex_constructs.pool.ltxml` `\@checkend` body has a stray trailing `}`

**Perl source (`Engine/latex_constructs.pool.ltxml` L190):**
```perl
DefMacro('\@checkend{}', '\def\reserved@a{#1}\ifx\reserved@a\@currenvir \else\@badend{#1}\fi}');
```

The replacement-text string ends with a stray `}`. It is a
transcription artifact from the LaTeX kernel's
`\def\@checkend#1{\def\reserved@a{#1}\ifx\reserved@a\@currenvir
\else\@badend{#1}\fi}` — that final `}` closes the `\def`, it is **not**
part of the macro body. Standard-LaTeX `\@checkend` therefore expands
to `\def\reserved@a{#1}\ifx…\fi` (no trailing brace), but LaTeXML's
`DefMacro` body includes the `}`, so every `\@checkend{env}` expansion
emits one unmatched `}`.

**Impact:** LaTeXML's own `\begin{}`/`\end{}` never call `\@checkend`
(the magic-CS path skips it), so the stray brace is normally invisible.
It only surfaces when a package **redefines `\end` to call
`\@checkend`** the standard-LaTeX way — e.g. `extract.sty`'s
`AfterEndEnv` machinery:
```latex
\def\begin#1{...\begingroup ...\csname #1\endcsname}
\def\end#1{\csname end#1\endcsname\@checkend{#1}\expandafter\endgroup ...}
```
Here `\@checkend{#1}`'s stray `}` runs while extract's wrapping
`\begingroup` is the open frame. Perl's gullet silently tolerates the
extra `}`; the Rust port raises `Error:unexpected:} Attempt to close
boxing group; current frame is non-boxing group due to \begingroup`
— **one error per environment** in the affected document.

**Rust resolution (`latex_constructs.rs` `\@checkend`):** dropped the
stray trailing `}` so the body matches standard-LaTeX semantics.
`\@checkend` is only reachable via packages that mimic the kernel
`\end`, all of which assume the kernel (brace-free) body, so this is
strictly more faithful. Witness 2007.09971 (IEEEtran + `extract.sty`
under ar5iv: 41 boxing-group errors → clean, matching Perl's 0 errors /
9 warnings).

## 26. `\raise`/`\lower` of a void box register (`\copy`/`\box`/`\lastbox`) spuriously errors

**Trigger (real-LaTeX-valid, errors in Perl):**
```latex
\setbox0=\hbox{X\raise1pt\copy\strutbox\lower1pt\copy\strutbox Y}
```
Perl emits `Error:expected:<box> A <box> was supposed to be here` twice; Rust
(pre-fix) did the same.

**Why it is wrong:** In TeX, fetching an UNSET box register via `\box`/`\copy`/
`\lastbox` yields a **void box**, which is a perfectly valid `<box>` operand for
`\raise`/`\lower`/`\moveleft`/`\moveright` (TeXbook p.388). The LaTeX kernel
relies on this — `\raise1pt\copy\strutbox` is a standard strut idiom — and
LaTeXML never `\setbox`es the visual `\strutbox`, so `\copy\strutbox` is always
void. Both engines' `MoveableBox` parameter reader treated the empty result as
"no box at all" and raised `expected:<box>`, where real TeX raises nothing.

**Impact:** Mostly invisible, EXCEPT when such an op sits in a `\halign` column
template (`\halign{...\raise1pt\copy\strutbox\lower1pt\copy\strutbox\vrule#...}`),
where it fires **once per cell/row**. On a many-row manual table this floods the
log: witness **1907.04219** — a `\halign`+`\Hline`/`\vrule` table → **102 errors
→ FATAL_3 abort (no output)** in Rust, while Perl (erroring fewer times) completed
with 7. Real TeX emits none.

**Rust resolution (`base_parameter_types.rs`, `MoveableBox::predigest`):** on an
empty box-fetch result, ERROR only when the box-starter was NOT a box-register op;
for `\box`/`\copy`/`\lastbox` substitute a void box silently (the substitution was
already there — only the spurious `Error!` was removed). Faithful to real TeX,
eliminates the per-cell cascade. Witness 1907.04219: 102 errors / FATAL_3 → **0
errors, 4.9 MB doc** (6 tables, 787 tabulars). Surpasses Perl on this shared
Perl/LaTeXML bug.

## 27. `\expandafter{\alignat}` orphans `\else`/`\fi` (amsmath env-begin macros modeled with a `{}` arg)

**Perl source:** `LaTeXML/Package/amsmath.sty.ltxml` L515-518 (`\alignat`),
plus siblings `alignat*`, `xalignat`, `xxalignat`.

**Symptom:** two errors per occurrence:
```
Error:unexpected:\else Didn't expect a T_CS[\else] since we seem not to be in a conditional
Error:unexpected:\fi   Didn't expect a T_CS[\fi] since we seem not to be in a conditional
```

**Minimal example** (verified identical on Perl 0.8.8 and Rust, 2026-06-03):
```tex
\usepackage{amsmath}
\edef\foo{\unexpanded\expandafter{\alignat}}
```

**Real-world trigger:** etoolbox `\cspreto{alignat}{...}` — used by the
ECCV class (`eccv.sty` "linenomathpatchAMS" block, arXiv:2409.02543) to
patch AMS environments for line numbering. `\preto`'s false branch runs
`\edef#1{\unexpanded{#2}\unexpanded\expandafter{#1}}`.

**Root cause:** real amsmath defines the `alignat` *begin-code* as a
parameterless macro (the pair-count is read downstream by
`\start@align`), so `\expandafter{\alignat}` is harmless in real TeX.
LaTeXML models it as `DefMacro('\alignat{}', '\ifmmode...\else...\fi')`
— a macro with one parameter. Forcing one expansion step via
`\expandafter` makes it read its argument from a stream whose next token
is `}`; the argument read derails the brace balance, and the
`\ifmmode...\else...\fi` body tokens subsequently surface with no active
conditional frame, yielding the orphaned `\else`/`\fi` pair.

**Impact:** 2 non-fatal errors per `\cspreto`/`\csappto`-style single-step
expansion of an affected env-begin CS; the patch the author intended is
also silently lost (same as Perl).

**Rust resolution:** none needed — behavior is verified bit-identical to
Perl (warn + 2 errors). Reproducers under
`~/data/reproducers/` (`alignat-cspreto-eccv.tex`,
`alignat-expandafter-orphaned-elsefi.tex` — see its README.md).
A genuine fix belongs upstream (model `alignat`-family begin-code as
parameterless, reading the pair count in the alignment setup), and would
be a documented divergence if taken before Perl does.

## 28. tikz-cd / quantikz matrix coordinates unparseable by the LaTeXML tikz interpretation — error cascade to fatal

**Perl source:** the raw-TikZ interpretation pathway (`tikz.sty.ltxml` +
pgfsys driver). Both engines interpret the *real* tikz/pgf from texmf;
`tikz-cd`'s arrow/matrix machinery produces coordinates the
LaTeXML-driven pgf parsing cannot handle.

**Symptom:** with a TeX Live that provides `quantikz`/`tikz-cd`
(library `quantikz2`, TL2024+), every cell of every `tikzcd` diagram
yields
```
Error:latex:(tikz) Package tikz Error: Cannot parse this coordinate
```
cascading until the error cap kills the conversion:
- Perl 0.8.8 (TL2025): 90×, then `Fatal:too_many_errors:100 Too many errors (> 100)!`
- Rust HEAD f5637c92ba: same cascade, `Fatal:TooManyErrors:MaxLimit(500)`
  ("same error fired 501 times in a row"; 514 errors total).

Also identical in both: `Error:undefined:\tikzcdmatrixname`, "Giving up
on this path. Did you forget a semicolon?".

**Witness:** arXiv:2403.19758 (`\usepackage{tikz}` +
`\usetikzlibrary{quantikz2}`, inline `\begin{tikzcd} \qw & \gate{X} ...`).
On *older* TL (production cortex container) quantikz2 is absent, so
`{tikzcd}` is simply undefined → 95 recoverable errors and a surviving
(degraded) document — the failure mode is TL-vintage-dependent.

**Impact:** papers using quantikz/tikz-cd convert to nothing (fatal) on
modern TL, in both engines.

**Rust resolution:** parity confirmed (2026-06-03) — no Rust-side defect.
Two follow-ups worth separate consideration:
1. cap-semantics alignment: Perl fatals at >100 *total* errors; Rust's
   consecutive-same-error cap (500) let this run reach 514 total before
   dying. Same outcome here, but counts/log shape diverge.
2. an actual tikz-cd/quantikz coordinate fix would be upstream-grade work
   benefiting both engines (or a Rust-first divergence to be documented).

## 29. OmniBus `\ead{}[]` emits the optional arg as the email (PR #2767 typo)

Upstream PR #2767 rewrote OmniBus.cls.ltxml's email macros:

```perl
DefMacro('\email{}',     '\lx@add@email{#1}');
DefMacro('\emailaddr{}', '\lx@add@email{#1}');
DefMacro('\ead{}[]',     '\lx@add@email{#2}');   # <-- #2 is the OPTIONAL
```

With prototype `{}[]`, `#1` is the address and `#2` the trailing
optional (the elsart-style type, e.g. `[url]`). The body passes `#2`,
so the common call `\ead{user@example.org}` produces an **empty**
`<ltx:contact role="email"/>` and drops the address. The pre-PR body
correctly used `#1` (`\@@@email{#1}{#2}`).

**Minimal trigger** (with an OmniBus-fallback class):

```latex
\documentclass{unknownclass}
\author{A. Author}\ead{user@example.org}
\begin{document}\maketitle x\end{document}
```

Perl: `<contact role="email"></contact>` (empty). Expected: the address.

**Rust:** `omnibus_cls.rs` deliberately uses `{#1}` (documented
divergence; this entry). Revisit if upstream fixes the typo.

## 30. PR-2767 `digestFrontMatter` unguarded re-entry → `deep_recursion` fatal

**Perl source:** `LaTeXML/Engine/Base_Utility.pool.ltxml` (post-#2767),
`digestFrontMatter` — digests from the **live** `frontmatter_raw` queue
and wipes it only after the loop.

**Symptom:** conversion dies with
```
Fatal:perl:deep_recursion Deep recursion on subroutine "LaTeXML::Core::Stomach::invokeToken"
```
(stack alternates `\lx@frontmatterhere` ↔ `\lx@add@frontmatter@now`),
**zero output**. Verified on `LaTeXML@23f3acfa` 2026-06-04.

**Root cause:** when a queued entry's *content* contains `\maketitle`
(→ `\lx@frontmatterhere`, whose `afterDigest` calls
`digestFrontMatter`), the nested invocation re-reads the still-live
queue and re-digests it — including the entry being digested —
unboundedly. `\maketitle`'s own `\global\let\maketitle\relax` cannot
stop it: it sits *after* `\lx@frontmatterhere` in the expansion, so
the recursion dives first.

**Real-world trigger:** arXiv:0907.0384 (A&A). aa.cls's `\abstract`
is 1-arg *or* 5-arg; the paper writes `\abstract{…} {}` so the
binding (faithfully, in both engines) dispatches the 5-arg
`\abstract@new`, whose greedy `{}` parameters swallow `\keywords`
(#3, #4) and **`\maketitle` (#5)** into the queued abstract content.
pdflatex compiles this paper.

**Minimal trigger** (with aa.cls):
```latex
\documentclass{aa}
\begin{document}
\title{T}\author{A}
\abstract{body} {}
\keywords{k}
\maketitle
\end{document}
```

**Rust:** not affected — `digest_front_matter` snapshots and
pre-clears the queue, so the nested invocation terminates and the
paper converts with zero errors (intentional divergence,
`OXIDIZED_DESIGN.md` #33). Worth reporting upstream.

## 31. `cleanFrontmatterLabels` prefixes empty fields → contentless `"prefix:"` labels

**Perl source:** `LaTeXML/Engine/Base_Utility.pool.ltxml`
(post-#2767), `cleanFrontmatterLabels` — `split(',')` then
unconditional `$prefix . ':' . $label`.

**Symptom:** a doubled comma or empty keyval field (`label={a,,b}`,
`\inst{1,,2}`) yields a contentless label like `affiliation:`. It
enters the `_annotations`/`_label` matching tables, where two
unrelated contentless labels can spuriously match each other during
`relocateAnnotations`, attaching an annotation to the wrong parent.

**Minimal trigger:**
```latex
\author{A. Author\inst{1,}}
\institute{Univ A}
```
→ creator `_annotations` gains `affiliation:1,affiliation:` (the
second field is empty but still prefixed).

**Rust:** drops fields with no real content before prefixing
(intentional divergence, `OXIDIZED_DESIGN.md` #34; plan decisions
log #5). Perl's trailing-empty `split` semantics is otherwise
preserved byte-exactly.

## `catoptions.sty` raw-load fails in Perl too (SHARED, not Rust-only)

`catoptions.sty` (a dependency of `keyval2e.sty`) cannot be raw-loaded
by Perl LaTeXML either. With `--includestyles` (or the ar5iv
`rawstyles` profile) Perl FATALs:

```
Error:unexpected:\let ... should not appear between \csname and \endcsname
  at catoptions.sty; line 6362
Fatal:too_many_errors:100 Too many errors (> 100)!
```

catoptions does heavy `\csname`-driven catcode machinery that neither
engine interprets. Perl's *default* (no `--includestyles`) treats
`keyval2e.sty`/`catoptions.sty` as **missing files** and skips them,
producing output; the ar5iv pipeline (rawstyles on) fails identically
in Perl and Rust. Minimal trigger:

```latex
\documentclass{article}
\usepackage{keyval2e}   % → \RequirePackage{catoptions}
\begin{document}x\end{document}
```

Witnesses (round-37 second-500K, all SHARED): 1501.07012, 1502.01082,
1507.04637, 1512.01732 (a Cretan/Hadamard-matrix paper family). Our
engine FATALs earlier with `ParamSpec:Expected` (the `\@namedef{#1@#2@…}`
body executes at load time because catoptions' `\robust@def`/`\cpt@def@`
expansion misfires), but the net outcome — no HTML — matches Perl. Not
actionable as a Rust-only fix; revisit only if catoptions raw-load
becomes a deliberate engine goal.

## `mdwmath.sty` `\sq@readrad` `#`-leak — `\meaning\sqrtsign` lacks the `"` delimiter (SHARED)

`mdwmath.sty` (mdwtools) redefines `\sqrt`/`\root` by reading the
*meaning* of the kernel `\sqrtsign` mathchar to recover its radical
delimiter code. With `|` temporarily made the escape character it
defines (L50–51):

```tex
|def|sq@readrad#1"#2\#3|relax{|global|sq@sqrt"#2|relax}
|expandafter|sq@readrad|meaning|sqrtsign|relax
```

i.e. `\def\sq@readrad #1"#2\#3\relax{…}` then
`\expandafter\sq@readrad \meaning\sqrtsign \relax`. The macro is
delimited by a literal `"` (the `#2` runs *up to* a double-quote) and
expects `\meaning\sqrtsign` to expand to something like
`\mathchar"1270` so that `#2` captures the hex code after the `"`.

This only works when `\sqrtsign` is a genuine **`\mathchar` primitive**
whose `\meaning` string contains `"`. Under LaTeXML — **both** engines —
`\sqrtsign` is not a raw `\mathchar`, so `\meaning\sqrtsign` carries no
`"`; the `#1"#2\#3` delimited scan never finds its `"` terminator,
over-runs the intended argument, and the literal `#` parameter tokens
from the *body* leak out to be digested. The result is a burst of:

```
Error:misdefined:# The token "#" (catcode PARAM) should never reach Stomach!
```

emitted **while processing `mdwmath.sty` itself** (load time, not use
time). Confirmed SHARED 2026-05-29 against Perl `~/perl5/bin/latexml
--path=~/git/ar5iv-bindings/bindings --preload=ar5iv.sty`: witness
**1811.09652** gives RUST 43 / PERL 44 errors, and Perl's own log shows
the identical `Error:misdefined:# The token T_PARAM[#] should never reach
Stomach! at mdwmath.s…`. Re-confirmed 2026-05-31 by a fresh untested-corpus
sweep: **1405.7843** (RUST 43 / PERL 51) and **1711.06771** (RUST 43 / PERL 44)
— in both, Perl emits the identical 43 `misdefined:#` *plus* extra
alignment/`\omit`/`\tab@*` errors, so Perl is strictly worse. The `misdefined:#`
cluster is one of the largest in the corpus (~1300 papers via the mdwtools
largest in the corpus (~1300 papers via the mdwtools family), but it is
an **upstream LaTeXML limitation** — `\meaning` of LaTeXML's `\sqrtsign`
does not reproduce TeX's `\mathchar"…` form — not a Rust-only defect.
Not actionable as a Rust-only fix; would require teaching LaTeXML's
`\sqrtsign`/`\meaning` to round-trip mathchar codes the way TeX does,
which is out of scope and equally absent in Perl.

## A text-symbol CS (`\i`/`\j`) in a `\usepackage` Semiverbatim option hangs (SHARED)

`\usepackage[pdfauthor={…Mar{\'\i}n…}]{hyperref}` — i.e. a font-encoding
text symbol (`\i`, `\j`, …) inside a `\usepackage`/`\RequirePackage`
**Semiverbatim** option value — infinite-loops in **both** Perl and Rust
(`Fatal:Timeout:PushbackLimit`, Perl exit 143 under `timeout`). Confirmed
2026-05-28 against Perl `~/perl5/bin/latexml --path=~/git/ar5iv-bindings
--preload=ar5iv.sty` on the real paper **2004.08143** *and* minimal
reproducers.

Minimal trigger (both engines hang):

```latex
\documentclass{article}
\usepackage[pdfauthor={Daniel Mar{\'\i}n}]{hyperref}
\begin{document}\href{u}{t}\end{document}
```

Mechanism (identical in both engines):
1. `\usepackage`'s `Semiverbatim` option is digested by *expanding* it
   under `beginSemiverbatim`, which merges the current font with
   `encoding => 'ASCII'` (Perl `State.pm:597`, Rust `state.rs:2296` —
   faithful) — a "stay-ASCII" neutralization. The expansion is a pure
   `readXToken` collect-loop (Perl `Parameter.pm::digest` "BLECH!!!!",
   Rust `parameter.rs:388`).
2. `\i` is `\DeclareTextSymbol`-defined `\i → \T1-cmd \i \T1\i`, with
   `\T1-cmd`≡`\@changed@cmd`. In the preamble `\protect`≡`\relax`≡
   `\@typeset@protect`, so the *typeset* branch resolves the glyph via
   `\csname\cf@encoding\string\i\endcsname` → `\csname ASCII\string\i…` =
   `\ASCII\i`, which is **undefined** (ASCII is a char-decode font *map*,
   not a LaTeX text *encoding* with `\i` glyphs).
3. `\@changed@cmd` `\global\let`s `\ASCII\i` to the `?`-fallback `\?\i` =
   `\UseTextSymbol{OT1}\i` = `{\fontencoding{OT1}\i}`. But `{` and
   `\fontencoding{OT1}` are non-expandable, so the `readXToken` loop
   *collects* them without executing — the font encoding stays "ASCII" —
   and the inner `\i` re-expands → step 2. Infinite.

Breaking the loop requires making `\fontencoding{OT1}` take effect inside
the semiverbatim `readXToken` (so the inner `\i` resolves to `\OT1\i`,
CD 16, which IS defined). Perl does not, so Perl hangs too. **This is a
RELEASE_CRITERIA surpass-Perl reliability item, not a translation parity
bug.** Tracked in memory `robust-cs-semiverbatim-loop`. (Separately, a
genuine adjacent divergence was fixed: Rust's `\cf@encoding`/`\f@encoding`
fell back to *empty* when the live font's encoding slot is `None`; Perl's
Font always carries OT1 — `Common/Font.pm:331`/`$DEFENCODING`. Now falls
back to OT1 when a font exists. That does not fix this shared loop.)

## `aas_support.sty.ltxml` omits `\floattable` (aastex62/631 macro)

The AASTeX class macro `\floattable` — `aastex62.cls` L4574
`\def\floattable{\global\deluxestartrue\global\floattrue}`, a no-arg
declaration that makes the FOLLOWING deluxetable a full-width (spanning)
float in two-column PDF layout — is **not** provided by Perl's
`aas_support.sty.ltxml` (which has `\deluxetable`/`\planotable`/
`\splitdeluxetable` but not `\floattable`). So a paper that bundles
`aastex62.cls` and writes `\floattable` before a table raises
`Error:undefined:\floattable` in Perl too:

```
Conversion complete: … 1 error; 1 undefined macro[\floattable]
```

Witness: 1909.08916 (`\documentclass{aastex62}`, `\floattable` before
deluxetables). Both LaTeXML bindings route `aastex62` through the
`aastex.cls.ltxml`/`aas_support` path rather than raw-loading the bundled
`.cls`, so the gap is shared. Since `\floattable` is pure page-layout
(full-width float placement), it is moot in our HTML paradigm; the Rust
port adds it as a no-op in `aas_support_sty.rs` (alongside `\placetable`/
`\platewidth`), which makes Rust convert the witness cleanly where Perl
still errors. Minimal trigger:

```latex
\documentclass{aastex62}    % bundled aastex62.cls
\begin{document}
\floattable
\begin{deluxetable}{cc}\tablehead{\colhead{a} & \colhead{b}}
\startdata 1 & 2 \enddata\end{deluxetable}
\end{document}
```

## `mdwmath.sty` raw-load — `#` (catcode PARAM) reaches Stomach

`mdwmath.sty` (TeX Live `mdwtools`) cannot be raw-loaded cleanly by LaTeXML —
**Perl and Rust both** emit ~43 `Error:misdefined:# The token "#" (catcode
PARAM) should never reach Stomach!` at `mdwmath.sty line 133` (the `\bbigg@#1#2#3`
body redefining `\big`/`\Big`/`\bigg`/`\Bigg`), plus a Perl
`Error:expected:Until:"` on `\sq@readrad` (the `\root`/`\sqrt` delimited-arg
macro). The `#1/#2/#3` parameters in the `\bbigg@` body leak to digestion when
the macro is used. There is **no** `mdwmath` binding in upstream LaTeXML or
ar5iv-bindings, so it is always raw-loaded and always errors.

This is an **upstream LaTeXML limitation, shared by Perl** — Rust is faithful and
must NOT "fix" it (doing so would diverge from the ground truth). Conversions
still complete (rc=0) with these errors in both engines. Frequent in the wild
(~25–30 affected papers per 10k in the large-scale canvas). Minimal trigger:

```latex
\documentclass{article}
\usepackage{mdwmath}
\begin{document}
$\big( x \big)$ and $\Big[ y \Big]$
\end{document}
```

Reproduce both: `latexml --includestyles test.tex` (Perl) vs `cortex_worker
--standalone --input test.zip` (Rust) — identical `#`-leak error count.

## `\alignat` family arg-taking breaks etoolbox `\preto`/`\cspreto` — `\else`/`\fi` leak (SHARED; FIXED in Rust)

`amsmath.sty.ltxml` (Perl L514–545) and the Rust port both define the
`alignat`-family environment-start macros **arg-taking**, to capture (and
ignore) the column-pair count:

```perl
DefMacro('\alignat{}',
  '\ifmmode\let\endalignat\endalignedat\alignedat{#1}\else'
    . '\lx@hidden@bgroup\@ams@align@bindings\@@amsalign'
    . '\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}'
    . '\lx@begin@alignment\fi');
```

(likewise `\csname alignat*\endcsname{}`, `\xalignat{}`,
`\csname xalignat*\endcsname{}`, `\xxalignat{}`).

**Real amsmath's `\alignat` is parameterless** — `\alignat ->
\start@align \z@ \st@rredfalse` — and `\start@align` reads the count
*later* from the stream. LaTeXML's arg-taking form is the divergence.

etoolbox's `\preto`/`\appto`/`\cspreto`/`\csappto` prepend/append to a
macro by re-`\edef`-ing it with `\unexpanded\expandafter{<cs>}` (=
`\expandonce<cs>`), which **forces exactly one expansion** of the target.
For a *parameterless* macro that just stores the body tokens (wrapped by
`\unexpanded`) — safe. For an **arg-taking** macro, the forced expansion
makes `<cs>` read its `#1` from the only token available — the group's
closing `}` — which collapses the `\unexpanded{...}` braces and lets the
body's `\ifmmode … \else … \fi` escape as a **bare `\else` then `\fi`**:

```
Error:unexpected:\else Didn't expect a "T_CS[\else]" since we seem not to be in a conditional
Error:unexpected:fi    Didn't expect a "T_CS[\fi]"    since we seem not to be in a conditional
```

This is exactly what `lineno`'s amsmath patch does (and what conference
classes like **eccv** invoke):

```tex
\newcommand*\linenomathpatchAMS[1]{\cspreto{#1}{\linenomathAMS}\cspreto{#1*}{\linenomathAMS}…}
\linenomathpatchAMS{alignat}   % -> \cspreto{alignat}{…} + \cspreto{alignat*}{…}, each leaks one \else/\fi
```

so `\linenomathpatchAMS{alignat}` alone produces **4** errors (2 per
`\cspreto`); `align`/`gather`/`multline`/`flalign` are parameterless and
stay clean. Confirmed SHARED: Perl `latexml --includestyles` on an eccv
witness emits the identical 4 conditional errors.

**FIXED in Rust (surpasses Perl), 2026-06-07.** `amsmath_sty.rs` now
mirrors real amsmath's *parameterless* structure via indirection: the
public macro is parameterless and forwards to an internal arg-reader, so
`\expandonce\alignat` yields a single token (no brace-grab, no premature
conditional):

```rust
DefMacro!("\\alignat", "\\lx@alignat@col");      // parameterless wrapper
DefMacro!("\\lx@alignat@col{}", "\\ifmmode…\\alignedat{#1}\\else…\\fi");
```

applied to `\alignat`, `\alignat*`, `\xalignat`, `\xalignat*`,
`\xxalignat`. Witness papers (canvas `large_scale_canvas_3_third`):
**2310.18293** (4→0), **2309.17074**, **2310.00161** — all now convert
error-free; normal `\begin{alignat}{2}` rendering (rows/cells/eqno)
unchanged; full Rust suite 1359/0. The Perl reference is left as-is per
the no-modify-`LaTeXML/` rule.

## Missing `line`/`lcircle` fontmaps → zero-width picture chars → `\@whiledim` infinite loop / OOM (2026-06-09)

Perl LaTeXML ships **no fontmap for the LaTeX picture-mode line fonts**
(`line10`, `linew10`, `lcircle10`, `lcirclew10`); `FontDecode` reports
`Info:fontmap:line Couldn't find fontmap for 'line'` and drops every
`\char` from those fonts, so an `\hbox{\@linefnt\@getlinechar(x,y)}`
measures **0 pt wide**. LaTeX-2.09-era plain-TeX documents (arXiv
math0102053, math0102089, math0212126, math0504436, math0506088,
math0604321, …) inline picture mode's `\@sline`, whose drawing loop
advances by exactly that width:

```tex
\@clnwd=\wd\@linechar
\@whiledim \@clnwd <\@linelen \do {…\advance\@clnwd \wd\@linechar}
```

Real TeX gets nonzero widths (2.5–10 pt) from `line10.tfm` and terminates;
Perl loops forever, accumulating boxes until OOM (observed: rc=124 after
3 m 19 s at a 6 GB cap on math0102053). Modern `latex.ltx` even guards this
exact hazard (`\ifdim\wd\@linechar=\z@\setbox\@linechar\hbox{.}%
\@badlinearg\fi`), but pre-guard 2.09 macro copies bypass it, so the font
width is the only lever that reaches them.

**Minimal trigger** (Perl hangs, real TeX prints 2.5 pt):

```tex
\font\tenln=line10
\setbox0=\hbox{\tenln \char'27}
\message{WD=\the\wd0}
\bye
```

**FIXED in Rust (surpasses Perl), 2026-06-09:** shipped `line.fontmap` +
`lcircle.fontmap` bindings (`latexml_package/src/package/line_fontmap.rs`,
`lcircle_fontmap.rs`) mapping the TFM slots to diagonal/arrow/arc/disk
glyphs — every populated slot gets a nonzero-width glyph, so the loops
terminate. All six witness papers now convert error-free with full-size
documents (math0102053: 4.5 GB OOM → 3.2 s, 0 errors). No control-flow
divergence: Perl given the same fontmap would behave identically.

---

## 32. `\item[\refstepcounter{<itemcounter>}…]` infinite recursion (shared Perl/Rust)

**Perl source:** `LaTeXML/Engine/latex_constructs.pool.ltxml` `sub RefStepItemCounter`
(L1362-1393); Rust port `latexml_core/src/binding/counter/dialect.rs::ref_step_item_counter`.

**Symptom:** A list item whose *optional argument* (custom label) contains
`\refstepcounter{<C>}` where `<C>` is the **same counter the list itself uses**
(`enumi` at enumerate level 1) recurses without bound. Rust trips the
`Fatal:Stomach:Recursion` fuse; Perl trips its own runtime
`Fatal:perl:deep_recursion` (`Deep recursion on subroutine
"LaTeXML::Core::Gullet::readingFromMouth"`). **Both implementations fail with a
conversion-fatal** (`Status:conversion:3`).

**Minimal trigger:**
```tex
\documentclass{article}
\begin{document}
\begin{enumerate}
\item[\refstepcounter{enumi}Stage] Hello
\end{enumerate}
\end{document}
```
(Independent of `enumitem`/`hyperref` — reproduced with each removed.)

**Root cause:** `RefStepItemCounter`/`ref_step_item_counter` embeds the optarg
into `\def\fnum@<itemcounter>{\makelabel{<optarg>}}` and then digests
`\lx@make@tags{<itemcounter>}`. The default ("") tag formatter `\lx@fnum@@`
expands `\fnum@<itemcounter>` → digests the optarg → runs
`\refstepcounter{<itemcounter>}` → `ref_step_counter` → `\lx@make@tags{<itemcounter>}`
→ reads `\fnum@<itemcounter>` (still the optarg) → `\refstepcounter` → … The
optarg's counter and the item counter being identical (`enumi == enumi`) closes
the loop. The stack is the repeating unit
`\lx@tags → \lx@tag@intags → { → \refstepcounter → \lx@tags → …`.

**Witnesses:** tikz-cd 2009.08640 (`stab_map.tex:28`,
`\item[\refstepcounter{enumi}\scshape Stage $0$]`). Perl reference
(`tex_to_html.zip`) on the same paper: `Status:conversion:3`,
`deep_recursion`.

**Status:** Shared upstream/Rust limitation — **parity preserved** (both fatal).
The real-LaTeX semantics (step the counter once as a side effect of typesetting
the label) differ from LaTeXML's tag-machinery model, which re-executes the
label each time the tag is formatted. **Kept as-is**: a fix would have to break
the re-entrancy inside the core item/tag path that every list relies on — high
regression risk for a pathological input that Perl also rejects. Rust's outcome
(`Fatal:Stomach:Recursion`, caught by the engine fuse) is arguably cleaner than
Perl's (a Perl-runtime deep-recursion warning).

---

## 33. `\numexpr` division (`divideround`) rounds half toward +∞, not away from zero

**Perl source:** `LaTeXML/Common/Number.pm:117-119`
```perl
sub divideround {
  my ($self, $other) = @_;
  return (ref $self)->new(int(0.5 + $self->valueOf / (... || $EPSILON))); }
```
used by `eTeX.pool.ltxml:189` for the `/` operator of `\numexpr`/`\dimexpr`.

**Symptom:** `\numexpr a/b\relax` disagrees with real (e)TeX whenever the exact
quotient is negative or a negative half-tie. TeX's `\numexpr` rounds the
quotient to the nearest integer with **ties away from zero**; Perl computes
`int(0.5 + a/b)`, which is round-half-toward-**positive infinity** (`int()`
truncates toward zero, so the `+0.5` only rounds up — never down for negatives).

**Minimal example & divergence (real TeX → Perl/Rust):**
```tex
\the\numexpr -7/2\relax   % real TeX: -4   Perl/Rust: -3
\the\numexpr -7/3\relax   % real TeX: -2   Perl/Rust: -1
\the\numexpr -1/2\relax   % real TeX: -1   Perl/Rust:  0
```
Positive operands are correct in all three (`7/2 → 4`, `7/3 → 2`, `1/2 → 1`).

**Impact:** Subtle off-by-one in `\numexpr`-based arithmetic (calc, etoolbox,
pgfmath, expl3's `\int_div_round:nn`/`\int_mod:nn`, …) when a sub-expression
divides to a negative or negative-half value. Rare in practice — most package
arithmetic divides positive lengths/counts.

**Perl status:** present and unchanged upstream.

**Rust status: KEPT FAITHFUL (verified parity).** `divideround`
(`latexml_core/src/common/numeric_ops.rs:149`) is `(0.5 + a/b).trunc()`, where
Rust's `f64::trunc` truncates toward zero exactly like Perl's `int()` — so Rust
reproduces Perl bit-for-bit (confirmed: `\numexpr` probe gives identical
`a..j` on `/usr/local/bin/latexml` v0.8.8 and the Rust binary). Under the
strict-Perl-parity priority this is **deliberately NOT changed** — a true-TeX
round-half-away-from-zero would diverge from every Perl-derived reference XML.
Contrast `\ifodd` (TeX_Logic), where Perl's `valueOf % 2` *does* match TeX for
negatives but the Rust `% 2 == 1` did not — that was a genuine Rust bug, fixed
to `% 2 != 0` (see git `5787070020`). The discriminator: faithful-to-Perl is the
target; only fix Rust where it diverges *from Perl*, not where Perl diverges
from TeX.

---

## 34. `revtex4_support.sty.ltxml` `\endpage` missing `{}` parameter text → `#1` leaks

**Perl source:** `LaTeXML/Package/revtex4_support.sty.ltxml:317-318`
```perl
DefMacro('\startpage{}',    '\pageref{FirstPage}{#1}');   # correct: declares {}
DefMacro('\endpage',        '\pageref{LastPage}{#1}');    # BUG: no {} but body uses #1
```

**Symptom:** A revtex4 paper that calls `\endpage{<n>}` (standard front matter,
typeset by `\maketitle`) emits:
```
Error:misdefined:#1 The token #1 (catcode ARG) should never reach Stomach!
```
The `\endpage` definition declares **no** parameter text, so the literal `#1` in
its body is never bound to an argument; the unmatched `T_ARG[#1]` survives
expansion and reaches the digester. The adjacent `\startpage{}` is correct.

**Minimal example:**
```tex
\documentclass[prl,byrevtex,twocolumn]{revtex4}
\begin{document}\title{T}\author{A}
\endpage{ }
\maketitle
\end{document}
```

**Impact:** one spurious error per affected revtex4 paper (witness arXiv
`0804.1404`: 1 error → 0 after the fix). Sibling of #15 (the same file's
`\eqnum` references `#2` with one parameter).

**Perl status:** present and unchanged — Perl errors identically (verified on
`/usr/local/bin/latexml` v0.8.8: `Error:misdefined:#1 … should never reach
Stomach!`).

**Rust status (FIXED 2026-06-20, beneficial divergence):** declare the missing
parameter — `DefMacro!("\\endpage{}", "\\pageref{LastPage}{#1}")`
(`revtex4_support_sty.rs`), mirroring `\startpage{}` and real revtex4 (where
`\endpage` takes the page number). Unambiguously correct; the same
fix-and-document pattern as #1.

---

## 35. `\fbox`/`\framebox` always emit `cssstyle='padding:3.0pt'` (Dimension-vs-string compare)

**Perl source:** `LaTeXML/Engine/latex_constructs.pool.ltxml:4702`
```perl
properties => sub {
  my $sep     = LookupRegister('\fboxsep');     # a Dimension OBJECT
  my $sep_pts = $sep->toAttribute;              # e.g. "3.0pt"
  ...
  ($sep ne '3.0pt' ? (cssstyle => 'padding:' . $sep_pts) : ()), ... }
```

**Symptom:** Every `\fbox{…}` / `\framebox{…}` carries
`cssstyle='padding:3.0pt'` even at the DEFAULT `\fboxsep` (3pt) — including
inside `\fcolorbox`, enumerate custom labels, etc.

**Root cause:** the guard compares `$sep` — the `\fboxsep` **Dimension object** —
to the string `'3.0pt'` with `ne`, forcing a string compare of the object's
stringification (its internal sp form, never the literal `"3.0pt"`). So the
guard is **always true** and the padding cssstyle is **always** added. The
author plainly intended `$sep->toAttribute ne '3.0pt'` (skip the default).

**Minimal example:** `\fbox{x}` → `<ltx:text cssstyle='padding:3.0pt'
framecolor='#000000' framed='rectangle'>x</ltx:text>` (the padding appears even
though `\fboxsep` is the default 3pt).

**Perl status:** RESOLVED upstream by PR #2829 (merged 2026-07-02): the
hand-rolled properties block was replaced by `framedProperties(margin =>
'\fboxsep', rule => '\fboxrule')`, which compares attribute strings properly
(`$th_pt ne '0.4pt'` for the border) and emits `padding:` whenever a margin is
given — the buggy `$sep ne '3.0pt'` guard is gone.

**Rust status:** tracked Perl throughout — first the faithful mirror of the
buggy always-true guard (2026-06-20), now the #2829 `framed_properties` port
(2026-07-02, `tex_box.rs`), byte-identical fixtures both times.

## 36. OmniBus `\lx@doi` emits a malformed `https:/doi.org/` URL (single slash)

**Perl source:** `LaTeXML/Package/OmniBus.cls.ltxml:157`
```perl
DefConstructor('\lx@doi{}', '<ltx:ref href="https:/doi.org/#1">#1</ltx:ref>');
```

**Symptom:** every `\doi{…}` in the body of an OmniBus-fallback document (any
unknown `\documentclass`) produces a **broken** DOI link
`href="https:/doi.org/<doi>"` — the scheme separator is `https:/` (one slash),
not `https://`, so the URL does not resolve.

**Root cause:** a plain typo in the constructor template (`https:/` should be
`https://`). Confirmed via `/usr/local/bin/latexml` on `\documentclass{zzz}` +
`\doi{10.1234/example.5678}` → `href="https:/doi.org/10.1234/example.5678"`.

**Perl status:** present and unchanged.

**Rust status — DELIBERATELY CORRECT (Rust supersedes):** `omnibus_cls.rs`'s
`\lx@doi` emits `href='https://doi.org/#1'` (valid double slash). Unlike #35
(an output-*attribute* parity case where the faithful choice was to replicate
Perl's bug), a DOI href is a **functional link**, so per the policy "fix simple
Perl bugs in Rust" we keep the working URL rather than reproduce the typo. The
constructor carries a code comment marking this as an intentional divergence so
a future faithfulness pass does not revert it. (Maintainer may overrule toward
strict parity if exact href bytes ever matter for a comparison.)

---

## 37. Comma-list as a bare relation operand; right-nested formulae

**Perl source:** `LaTeXML/MathGrammar` (the `Parse::RecDescent` grammar) — the
relation productions admit a comma-list as a single RHS operand, and
`moreRHS`/`maybeColRHS` build right-recursive formulae.

**Symptom / Perl behavior** (verified via
`latexmlmath --cmml` and `latexmlc --preload=stmaryrd.sty --whatsin=math`):
* `a=b,c,d` → `eq(a, list(b,c,d))` — the comma-list becomes the **bare operand**
  of `=`.
* `0<x,y` → `lt(0, list(x,y))` — likewise for an inequality.
* `\quad`-separated formulas → **right-nested** `formulae@(f1, formulae@(f2, …))`.

**Why it's wrong:** a bare (unparenthesized) comma-list is **not a single
expression**, so it can never be the operand of a relation — in no STEM reading
does `a=b,c,d` mean "a equals the tuple (b,c,d)". It means the comma-separated
list `[a=b, c, d]`. (A *parenthesized* list `(x,y)` IS a single expression —
that stays a vector/tuple operand, unchanged.) The right-nesting of `formulae`
is likewise an artifact, not a semantic structure.

**Rust status — DELIBERATE DIVERGENCE (Rust supersedes; user-directed
2026-06-21).** The math grammar drops the `formula relop formula_list` rule
(`latexml_math_parser/src/grammar/builder.rs`), so a relation never takes a bare
list operand. Bare separated sequences are classified by
`latexml_math_parser/src/semantics.rs::list_apply`:
* **comma, all items relational** → `formulae@(x=0, y=1)`
* **comma, mixed/non-relational** → `list@(0<x, y)`, `list@(a=b, c, d)`
* **`\quad` (WIDE_PUNCT), any items** → a distinct flat `fragments@(…)` class
  (top-level heterogeneous fragments)

All multi-item containers are kept **flat** (the `moreRHS`-analog
`restructure_flat_to_right` nesting pass was removed). Besides being the correct
reading, this **eliminates a large grammar-ambiguity over-parse**: on
`1510.03361` the worst equation fell from the 5000-tree cap (578 ms) to 256
trees (31 ms, ~19×) and the `math_parse` phase dropped ~12%. Suite 1466/0/0.

## 38. `\marginpar` does not scope font/catcode changes (leaks into body)

**Trigger:**
```latex
\marginpar{\Large !} BODYWORD
```
**Perl behavior:** `BODYWORD` (and everything after) renders at `\Large` (144%) —
the `\Large` inside the margin note leaks into the main galley. Verified on Perl
LaTeXML 0.8.8 (`<text fontsize="144%">BODYWORD`). Real pdflatex typesets the note
in a separate margin box, so the switch is scoped; the LaTeXML `\marginpar`
`DefConstructor` (`latex_constructs.pool.ltxml` L3487) is not `bounded`, so its
argument digests in the enclosing group and the font assignment persists.

**Severity:** can be catastrophic for documents that put a size/style switch in a
margin note — e.g. the mhchem package manual's `\marginpar{\Large !}` rendered the
*entire* manual at 144%.

**Rust status — DELIBERATE DIVERGENCE (Rust supersedes).** `\marginpar` now carries
`bounded => true` (mirrors `\mbox`), scoping the note's font/catcode changes. Output-
neutral across the suite (1487/0). See `OXIDIZED_DESIGN.md` #39. Candidate to upstream.

## 39. booktabs `\cmidrule` defined via `\cline` → infinite loop under `\let\cline\cmidrule`

`booktabs.sty.ltxml` defines `\cmidrule` to draw its partial rule by expanding to
`\cline{<cols>}` (`\ltx@cmidrule` / `\ltx@@cmidrule` → `\cline{#2}`/`\cline{#3}`).
This is a simplification — real booktabs `\cmidrule` draws the rule directly and
does **not** touch `\cline`.

**Trigger:** a document that does `\let\cline\cmidrule` (a common idiom to make
`\cline` render as a nicer booktabs-style partial rule). In real LaTeX this is
harmless because `\cmidrule` is self-contained. In LaTeXML it creates a cycle:
`\cline` → `\cmidrule` → `\ltx@cmidrule` → `\cline` → `\cmidrule` → … — an infinite
macro expansion.

**Perl behavior:** Perl LaTeXML **hangs** (confirmed: `latexml --quiet` on
arXiv 2506.23179 runs to a 90 s+ timeout with no output) — the identical
`\cmidrule`→`\cline` binding loops with no conditional/expansion guard.

**Rust status — DELIBERATE DIVERGENCE (Rust supersedes).** Rust's gullet has an
8M-conditional `IfLimit` guard, so it fatals at ~12 s rather than hanging; and the
booktabs binding now routes `\cmidrule` through a **private saved copy** of `\cline`
(`\ltx@saved@cline`, captured at package-load before any document `\let`), so the
cycle never forms — the witnesses convert cleanly (2506.23179 172.9 s→fatal ⇒ **3 s,
0 errors**; 2511.17056 171.4 s→fatal ⇒ **1 s, 0 errors**). Output-neutral for ordinary
`\cmidrule` (the saved CS equals `\cline` at load). Guard:
`06_cluster_regressions.rs::cluster_cmidrule_cline_let`. Candidate to upstream.
File: `latexml_package/src/package/booktabs_sty.rs`.

## 40. amsfonts binding omits `\dabar@` → author `\xdashrightarrow` copies loop forever

**Trigger:** real `amsfonts.sty` defines
`\DeclareMathSymbol{\dabar@}{\mathord}{AMSa}{"39}` — the dash piece it
composes into `\dashrightarrow`/`\dashleftarrow`. Both LaTeXML bindings map the
arrows directly to `⇢`/`⇠` and omit `\dabar@`. Papers that paste the classic
extensible dashed-arrow snippet (`\xdashrightarrow`, mathtools-era folklore)
measure `\sbox4{$\dabar@\m@th$}` and grow a bar chain with
`\@whiledim\count@\wd4<\dimen@` — with `\dabar@` undefined, box 4 is 0 wide
and the loop can never terminate. Minimal trigger:
`docs/reproducers/xdasharrow_dabar_whiledim_loop.tex` (pdflatex compiles it
fine — the real package defines the glyph).

**Perl behavior:** emits `undefined \dabar@` but *completes* — only because
Perl computes **all** box widths as 0, so the loop target `\dimen@` is also 0
and `0 < 0` exits immediately (witness arXiv `1705.09248`: 2 errors, 58 s).
The escape is accidental, not a guard.

**Rust status — FIXED (2026-07-02), faithful to the real package.** Rust's
tfm-based label widths make `\dimen@ > 0`, so the same papers ran to
`Fatal:Timeout:TokenLimit` (31 papers in the 2026-07 full-arXiv run). The
binding now defines `\dabar@` (`╌`, U+254C) in `amsfonts_sty.rs`, terminating
the loop exactly as real TeX does. `\symAMSa` remains undefined in both
engines (same 2-error surface as Perl on the witness). Candidate to upstream.

## 41. PR #2829 `LookupDimension` rewrite loses the macro-body-read path

**Perl source:** `LaTeXML/Package.pm` `LookupDimension` (as of #2829, merged
2026-07-02)
```perl
elsif ((ref $cs eq 'LaTeXML::Core::Token') && ($defn = $STATE->lookupDefinition($cs))
  && $defn->isRegister) { return $defn->valueOf; }
elsif (ref $cs eq 'LaTeXML::Core::Tokens') { ... readDimension ... }
elsif (!$noerror) { Warn('expected', 'register', ...); }
```

**Symptom:** a document that `\def`s a length into a plain macro (e.g.
`\def\arraycolsep{5pt}` — real arXiv usage, our eqnarray/numcases cluster
regressions) now triggers `Warn('expected','register')` and the dimension
silently degrades to 0. Pre-#2829 Perl read the macro's body as a dimension
(`readingFromMouth($cs, sub { readDimension })`).

**Root cause:** the #2829 coercion rewrite ("LookupDimension coerces more
strings, CS, Dimensions") tokenizes a string argument and unwraps a
single-token result to a `Token` — but the new elsif chain only accepts a
single Token when its definition **isRegister**; the old defined-but-not-
register fallback (read the body) was dropped, presumably unintentionally
(the PR is about framing consistency).

**Minimal example:** `\def\arraycolsep{5pt}\begin{eqnarray}a&=&b\end{eqnarray}`
→ `expected:register` warning + zero column separation (was: silent, 5pt).

**Perl status:** present as of #2829 (d666adf8). Candidate to upstream.

**Rust status (kept pre-#2829 behavior, deliberate divergence):**
`state.rs::lookup_dimension_cs` ports the #2829 coercions (obvious-dimension
strings, register tokens, multi-token read) but RETAINS the macro-body-read
branch for a single defined-but-not-register token. Covered by the
`cluster_{eqnarray,numcases}_arraycolsep_macro_no_register_warning` tests.

## 42. `\cfrac[l]`/`\cfrac[r]` optional alignment argument is not consumed

**Perl source:** `LaTeXML/Engine/../Package/amsmath.sty.ltxml` L1110-1125 —
`\lx@inner@cfrac InFractionStyle InFractionStyle` takes no optional argument.

**Symptom:** real amsmath supports `\cfrac[l]{1}{2}` (numerator alignment);
LaTeXML reads `[` as the numerator and `l` as the denominator, mangling the
fraction and leaking `]{1}{2}` into the math.

**Minimal example:** `$\cfrac[l]{1}{2}$`.

**Perl status:** present (the trampoline + inner constructor never declare
an optional).

**Rust status:** faithful parity as of the #F15 trampoline port
(2026-07-02, `3b20c4f399`) — NOTE this is a behavior REGRESSION vs the
pre-audit Rust binding, whose fused `\cfrac[]` constructor tolerated (and
discarded) the optional. Candidate to fix in BOTH engines by adding `[]`
to `\lx@inner@cfrac` and passing the alignment through.

## 43. PR #2846 leaves the preamble too early → `\RequirePackage`/`\usepackage` in `\AtBeginDocument` wrongly errors

**Perl source:** `LaTeXML/Engine/latex_constructs.pool.ltxml`, `\begin{document}`
`afterDigest` (as of PR #2846 "Leave preamble at right place", fixes #2754).

**Symptom:** a package deferred to the begin-document hook —
`\AtBeginDocument{\RequirePackage{xcolor}}` (real-world: `inconsolata.sty` does
`\AtBeginDocument{...\usepackage{upquote}}`) — triggers
`Error:unexpected:\RequirePackage The current command '\RequirePackage' can only
appear in the preamble`. Ground truth (same host): **pdflatex → 0 errors**;
**pre-#2846 Perl 0.8.8 → 0 errors**. Corpus witnesses: arXiv:2605.00022,
arXiv:2605.00119.

**Minimal example** (`docs/reproducers/atbegindocument_requirepackage.tex`):
```tex
\documentclass{article}
\AtBeginDocument{\RequirePackage{xcolor}}
\begin{document} Hello \end{document}
```

**Root cause:** PR #2846 **moved** `AssignValue(inPreamble => 0)` from AFTER
`@at@begin@document` (pre-#2846: comment `# atbegin is still (sorta) preamble`)
to just BEFORE it (post-#2846: comment `# We're now leaving the preamble (!?)`).
So `@at@begin@document` (which digests `\AtBeginDocument` code) now runs with
`inPreamble=0`, and `\RequirePackage`/`\usepackage`'s `onlyPreamble` guard fires.
Real `latex.ltx` `\document` disables the `\@onlypreamble` commands
(`\@preamblecmds`, L54) only AFTER firing the begindocument hook (L44), so the
deferred load is legal — #2846 contradicts the kernel. The `(!?)` in the moved
comment is the author's own doubt.

**Perl status:** REGRESSION introduced by #2846 (verified: vendored post-#2846
`latexml` rev 51fea96a errors on the reproducer; installed pre-#2846 0.8.8 does
not). **Fixed in both Rust and Perl here** (revert #2846 + make `\par` context-aware
— see below); candidate to upstream as the #2846 follow-up.

**#2846 tried to overload `inPreamble` for two transitions.** `latex.ltx`
`\document` performs two things at different points: (A) body typesetting begins —
governs `\par` — BEFORE the begindocument hook (`\UseOneTimeHook`, L9512); and (B)
`\@preamblecmds` disables the `\@onlypreamble` commands — governs this guard —
AFTER it (L9522). #2846 cleared `inPreamble` before the hook to get (A), but
`inPreamble` also gates (B), so it disabled the guard too early. The resolution is
NOT a second flag, but to stop routing `\par` through `inPreamble` at all.

**The fix (both engines — `\par` made context-aware; #2846 reverted).**
`\begin{document}` restores the pre-#2846 placement (`inPreamble=0` AFTER the hooks
— so a deferred `\RequirePackage`/`\usepackage` stays legal; the onlyPreamble guard
is a plain `inPreamble` check again, no `inBeginDocumentHook`). `\lx@normal@par` is a
no-op **only in the RAW preamble** — `inPreamble` set AND `document` NOT on the env
stack. Everywhere else it closes the paragraph being built. Signals used (both are
existing state in Perl and Rust): `inPreamble`; and `current_environment`, which
`\begin{document}` sets to `document` at its START (Perl L316 / Rust
`latex_constructs.rs`), so it is on the stack throughout the hooks and the body.
Hence a blank line inside `\AtBeginDocument` (which runs in the document env) splits
paragraphs (#2754), while `\RequirePackage` there stays legal (inPreamble still 1).

Why *context*, not the note's literal "no-op in vertical mode"? LaTeXML's mode
tracking isn't faithful enough: it stays `vertical` after a display equation (a mode
test would drop the blank line between `$$…$$` groups — `spacing.xml`, `verb.xml`,
AND `\AtBeginDocument{\[x\]\n\ntext}`), and raw-preamble text is `horizontal` yet
must stay merged (expl3 case fixtures) — mode can't tell it from a hook `\par`. The
env-**stack** check (Perl `grep {…} lookupStackedValues('current_environment')` /
Rust `with_stacked_values`) also keeps a hook that opens a nested environment
(`\AtBeginDocument{\begin{center}…}`) counting as "in document"; the walk only runs
while `inPreamble` is set (`&&` short-circuits in the hot body path). Covered by both
reproducers (`docs/reproducers/atbegindocument_paragraph_break.tex` +
`atbegindocument_requirepackage.tex`, wired as `tests/structure/atbegindocument_*`),
with a body-level `\RequirePackage` still erroring (parity).

## 44. apxproof + kvoptions: `\ProcessLocalKeyvalOptions*` aborts the bibliography

**Perl source:** none — LaTeXML ships no `apxproof.sty.ltxml` (neither upstream
nor ar5iv-bindings), so Perl relies on raw-loading `apxproof.sty` under
`--includestyles`.

**Symptom (Perl, verbose, same host):** apxproof.sty L58 `\ProcessLocalKeyvalOptions*`
trips Perl's kvoptions handling —
`Package kvoptions Error: \ProcessLocalKeyvalOptions is intended for packages only`
— which then cascades to `Error: unsupported option bibliography=common for package
apxproof`. Net result: the `biblatex` citation wiring never runs and the document
renders **0 bibliography entries**. Ground truth (same host): **pdflatex → full
bibliography**. Witness: `/home/deyan/Downloads/bib_bug/gdsm.tex` (biblatex +
`\usepackage[bibliography=common]{apxproof}`, 24 cited entries).

**Rust status:** SURPASSES Perl. A `latexml_contrib/src/apxproof_sty.rs` binding
force-raw-loads `apxproof.sty` in every config (bare / `--includestyles` / ar5iv),
and Rust's kvoptions raw-load handles `\ProcessLocalKeyvalOptions*` — so apxproof's
setup runs, biblatex reads the `.bib`, all 24 citations link, and the 6 `proof`
environments keep LaTeXML's usual amsthm `ltx_proof` markup (apxproof defers only
its own `apxproof`/`proofatend` environments, unused here). Fixing this also
required a core catcode fix (option values stored with LETTER catcode — see
WISDOM #61) so apxproof's `\ifthenelse{\equal{\axp@bibliography}{common}}`
validation succeeds. Regression fixture: `tests/keyval_options/optcatcode*`.

## 45. IEEEeqnarray raw `\halign`: a row starting with an empty cell breaks the alignment

**Perl source:** none — LaTeXML ships no `IEEEtrantools.sty.ltxml`; it binds the
IEEEeqnarray family only inside `IEEEtran.cls.ltxml` (L242-332,
`DefMacroI('\IEEEeqnarray', '{}', '\eqnarray')`). So `article` +
`\usepackage{IEEEtrantools}` raw-loads IEEEtrantools.sty and uses its raw
`\halign`.

**Symptom:** an IEEEeqnarray row that BEGINS with an empty cell (a leading `&`,
e.g. `\nonumber\\ & & +\beta\ldots`) raises
`Error:unexpected:\halign Attempt to end mode restricted_horizontal`, then a
cascade of `_`/`^ can only appear in math mode` as the body leaks out of math
mode; the equation is mangled (the rest of the document still converts).
Reduction: a single row or two FULL rows are fine; only a leading-empty-cell row
triggers it; `{}` before the `&` is the author-side workaround. Ground truth
(same host): **pdflatex typesets it fine**; **Perl LaTeXML fails the same way**
(shared raw-`\halign` limitation — LaTeXML's alignment model, both engines,
mishandles the empty first cell; the code even flags it "mostly Wrong … not
there yet", `tex_tables.rs::digest_alignment_column` region).

**Minimal example** (`docs/reproducers/ieeeeqnarray_leading_empty_cell.tex`, run
with `--includestyles`):
```tex
\documentclass{article}\usepackage{IEEEtrantools}
\begin{document}
\begin{IEEEeqnarray}{rCl}
a & = & b \\
& = & d
\end{IEEEeqnarray}
\end{document}
```

**Rust status:** SURPASSES Perl via a native `IEEEtrantools.sty` binding
(`latexml_package/src/package/ieeetrantools_sty.rs`) that maps the IEEEeqnarray
family onto native `\eqnarray` (which handles leading-empty cells), instead of
the raw `\halign`. The underlying raw-`\halign` empty-first-cell limitation
remains for other raw alignments (the broader `\lx@begin@alignment` family).

---

## 46. `rearrangeEqnarray`: `label` vs `labels` typo drops numbers on distinctly-labelled continuation rows

**Perl source:** `LaTeXML/lib/LaTeXML/Engine/latex_constructs.pool.ltxml`
`rearrangeEqnarray` (L2299-2389), specifically the row scan L2310
(`labelled => $rownode->hasAttribute('label')`) and the R-column classifier
L2360-2362.

**Symptom:** an `eqnarray` (or anything mapped onto it, e.g. IEEEeqnarray) whose
continuation rows — empty first *and* second column, only the RHS filled — each
carry BOTH an automatic number and their own `\label` collapse onto a SINGLE
number instead of numbering separately. Concretely, four constraint rows that
should be `(a),(b),(c),(d)` render as only `(a)` and `(d)`; the middle labels
`(b),(c)` pile onto the last row's `labels` attribute and never render a number.
Witness: arXiv Problem-𝒫1 `IEEEeqnarray` (`ieee_eqn_bug/main_arXiv.tex` L554-591).

**Root cause:** `rearrangeEqnarray` merges continuation rows into the previous
equation, but the author added a safeguard — *"Separately numbered AND labeled?
… must keep separate, but weird!"* — gated on `$$row{labelled}`. That field is
set from `$rownode->hasAttribute('label')` (**singular**), yet LaTeXML only ever
emits the **plural** `labels` attribute (`LaTeXML-common.rnc` L134; there is no
singular `label` attribute in the schema). So `labelled` is **always false**,
the safeguard is dead code, and every such row is merged.

**Minimal example** (`latexml_oxide/tests/structure/eqnarray_labelled_rows.tex`):
```tex
\begin{subequations}\begin{eqnarray}
\operatorname{minimize}\; & & f(x) + g(x) \nonumber\\
& & {} +\, h(x) \label{eq:obj}\\
\text{s.t.}\; & & a(x) \leq 0 \label{eq:ca}\\
& & b(x) \leq 0 \label{eq:cb}\\
& & c(x) = 0 \label{eq:cc}
\end{eqnarray}\end{subequations}
```
Ground truth (same host): **pdfTeX numbers all four** `(a),(b),(c),(d)`; **Perl
LaTeXML collapses to `(a),(d)`** (dead-code safeguard).

**Rust status:** SURPASSES Perl (standing PDF-fidelity authorization; honors the
Perl author's documented intent). `rearrange_eqnarray`
(`latexml_engine/src/latex_constructs.rs` L1085) reads the real `labels`
attribute, so distinctly-numbered-and-labelled continuation rows stay separate
and match pdfTeX. Candidate to upstream (one-char fix). Strictly monotone: the
change can only *split* a merged equation whose row was numbered AND `\label`-ed;
it never merges. Marked `OXIDIZED_DESIGN divergence` at the call site.

## 47. Author-local `\def\name`/`\email`/`\addr` inside a redefined `\@maketitle` never take effect

A JMLR-style `article` paper redefines `\@maketitle` to *locally* `\def\name`,
`\def\email`, `\def\addr` (as font switches) and then expand `\@author` in that
group:

```tex
\def\@maketitle{\vbox{ … {\def\addr{\small\it}\def\email{\hfill\small\tt}%
  \def\name{\normalsize\bf}\@startauthor \@author \@endauthor}}}
\author{\name Knut Vanderbush \email{knutv@stanford.edu}\\ \addr{Stanford University} …}
```

LaTeXML (both Perl and Rust) uses its own structural `\maketitle`/frontmatter
machinery and never runs the paper's redefined `\@maketitle`, so `\name`,
`\email`, `\addr` are undefined when the `\author` argument is digested and leak
as literal text (`\name Knut Vanderbush \email …`).

**Ground truth (same host):** Perl LaTeXML emits `Error:undefined:\name`
/`\email`/`\addr` and renders `<ERROR class="undefined">\name</ERROR>Knut
Vanderbush …` — **identical** to Rust. This is **PARITY**, not a Rust
regression. Reproduces on `/usr/local/bin/latexml main.tex` (witness
arXiv:2601.05137). Faithfully emulating an arbitrary user `\@maketitle`
redefinition is out of scope; left at parity.

## 48. subcaption clobbers subfigure's `\subfigure`/`\subtable` (unconditional `DefEnvironment`) → unclosed group swallows the document

A document loads the (unsupported) `subfigure` package and then `subcaption`
(arXiv:2507.21938 loads `subfigure`, `caption`, `subcaption`, `subfigure` in
that order):

```tex
\usepackage{subfigure}\usepackage{caption}\usepackage{subcaption}
...
\subfigure[]{\includegraphics[width=0.35\textwidth]{plot1.pdf}}
```

The two packages have INCOMPATIBLE contracts for `\subfigure`: subfigure.sty
binds a self-contained MACRO `\subfigure[][]{}` (mandatory arg = the figure
body); subcaption binds an ENVIRONMENT `{subfigure}[]{Dimension}` (mandatory arg
= a length; opens a group closed only by `\end{subfigure}`). Perl's
`subcaption.sty.ltxml` declares the environment with an **unconditional**
`DefEnvironment('{subfigure}[]{Dimension}')`, which CLOBBERS the already-defined
`\subfigure` macro. The macro-form call above then reparses as
`\begin{subfigure}` with `{\includegraphics{…}}` misread as the `{Dimension}`
(→ *Missing number, treated as zero*) and the environment opened with no
matching `\end{subfigure}` — leaking an internal-vertical group that absorbs the
rest of the document (figures, sections, bibliography).

**Ground truth (same host):** reference Perl LaTeXML (0.8.8) **times out**
(>300 s, exit 124, zero output) on arXiv:2507.21938. Rust previously truncated
mid-body (2 sections, 0 bibitems). Real LaTeX avoids this because subcaption
declares the environment via `\newenvironment{subfigure}`, which REFUSES to
redefine an already-defined `\subfigure` (raising "Command \subfigure already
defined" and keeping subfigure.sty's macro), and because the two packages are
officially declared incompatible.

**Fixed in Rust** (`latexml_package/src/package/subcaption_sty.rs`): the
`{subfigure}` / `{subtable}` `DefEnvironment`s are now guarded by
`has_meaning(\subfigure)` / `has_meaning(\subtable)` — mirroring
`\newenvironment`'s "already defined" guard — and emit a `Warn!` naming the
package incompatibility when the guard fires. subfigure.sty's macro is kept, so
2507.21938 now converts fully (7 sections, 36 bibitems, 0 errors). Beyond-Perl
reliability win + upstream candidate (Perl should apply the same guard). Witness
arXiv:2507.21938; regression fixture
`subcaption_subfigure_conflict.tex`.

## 49. amsrefs inline bibliographies are dropped whole by `MakeBibliography` (empty References, every `\cite` dangling)

`amsrefs` writes the bibliography **into the document** rather than into an
external `.bib` (arXiv:2605.01646 `AIPFa.tex`, and 40 papers across sandboxes
2605+2606):

```tex
\usepackage[lite,abbrev,msc-links,alphabetic]{amsrefs}
...
\begin{bibdiv}\begin{biblist}
\bib{Bei87}{article}{ author={Be\u{\i}linson, A.}, title={Height pairing between algebraic cycles}, }
\end{biblist}\end{bibdiv}
```

The engine digests this correctly — `Package/amsrefs.sty.ltxml` turns each `\bib`
into an `ltx:bibentry` inside `ltx:biblist`. The loss happens in
**post-processing**:

* `MakeBibliography::getBibEntries` collects entries only from
  `foreach my $bibdoc ($self->getBibliographies($doc))`.
* `getBibliographies` resolves names from the command line or from
  `//ltx:bibliography/@files`. An amsrefs bibliography has **no `@files`** (its
  entries are already inline), so it returns an **empty list** and
  `getBibEntries` collects nothing.
* `process` then runs its unconditional
  `$doc->removeNodes($doc->findnodes('//ltx:bibentry'))` — *"Remove any
  bibentry's (these should have been converted to bibitems)"* — deleting every
  entry that nothing ever converted.

Result: an **empty `<ul class="ltx_biblist"></ul>`**, every `\cite` rendered as
`ltx_missing_citation`, and **no error is reported** — only
`Warning:expected:bibkeys Missing bibkeys ...`. Silent, total data loss for a
supported package.

Reproducer (both engines produce `ltx_bibitem: 0`, one `ltx_missing_citation`):

```tex
\documentclass{article}
\usepackage{amsrefs}
\begin{document}
Cite: \cite{Smith2020}.
\begin{bibdiv}\begin{biblist}
\bib{Smith2020}{article}{ author={John Smith}, title={On Examples}, journal={JMP}, year={2020} }
\end{biblist}\end{bibdiv}
\end{document}
```

Confirmed on the installed Perl 0.8.8 **and** the vendored tree
(`perl -I LaTeXML/blib/lib`, rev `51fea96a`) — not a version skew. On
arXiv:2605.01646 Perl yields 0 bibitems and 81 dangling citations.

**Fixed in Rust** (OXIDIZED_DESIGN #57): `get_bib_entries` also scans the main
document for inline `ltx:bibentry`. Papers with an external `.bib`/`.bbl` carry
no inline entries, so the scan is a no-op for them. All 40 corpus papers went
from 0 rendered references to 1,482 with zero dangling citations. **Upstream
candidate** — the upstream fix is one extra source document in the
`getBibEntries` loop.

## 50. Loading `bibunits`/`chapterbib` dangles EVERY citation (`Scan` and `CrossRef` disagree on the list chain)

Merely loading `bibunits` — without ever opening a `bibunit` environment — makes
every `\cite` in an otherwise ordinary document render as `ltx_missing_citation`,
while the References list itself renders perfectly. Witness arXiv:2303.06077
(revtex4-2 + `bibunits`): **93 bibitems, 93 dangling keys, 0 links.**

Six-line reproducer — deleting the one `\usepackage` line resolves the cite:

```tex
\documentclass{article}
\usepackage{bibunits}
\begin{document}
See \cite{Smith2020} for details.
\bibliography{refs}
\end{document}
```

The chain:

* `bibunits.sty.ltxml` L32-41 redefines `\cite` so **every** citation runs
  `\lx@bibunits@resetglobal`, which sets `CITE_UNIT` to `\bu@unitname` = `bu0`.
  The bibref is therefore emitted as `inlist='bu0'` just because the package is
  loaded.
* The document's single `\bibliography` has no unit, so `\lx@bibliography`'s
  `lists='#1'` is empty and its bibitems register under the default list
  (`Scan.pm` L465: `... || 'bibliography'`).
* `CrossRef.pm` L515 then looks **only** in the bibref's own list:
  `my @lists = split(/\s+/, $bibref->getAttribute('inlist') || 'bibliography');`
  → searches `BIBLABEL:bu0:<key>` alone, which has no `id`, and reports
  `Warning:expected:ids Missing Entry for citation: <key>`.

Upstream disagrees with itself: **`Scan.pm` L379-380 registers the reference
under the unit lists PLUS `'bibliography'`** — commented *"Citation specifies
main 'bibliography', as well as any specific others (eg. per chapter)"* — but
`CrossRef.pm` never consults that main list. Scan records two lists; CrossRef
reads one.

Confirmed on same-host installed Perl 0.8.8 with the reproducer above: 1
bibitem, 1 `ltx_missing_citation`, 0 links, plus the `expected:ids` warning.
(2303.06077 itself gives no Perl verdict — Perl `Fatal:timeout` /
`Status:conversion:3` on it, where Rust converts in ~2 min.)

**Fixed in Rust** (OXIDIZED_DESIGN #59): `CrossRef` appends `bibliography` to the
searched lists, following `Scan.pm`'s own convention; unit lists are still
searched first, so a real per-chapter bibliography keeps priority. 2303.06077 →
93 bibitems / 0 dangling / 179 resolved links. **Upstream candidate** — the fix
is one line in `CrossRef.pm` L515 to mirror `Scan.pm` L379-380.

## 51. `\end{lstlisting}` with content before it on the same line silently swallows the rest of the document

`listings.sty.ltxml` L316 (`listingsReadRawLines`) anchors the terminator at the
start of the line:

```perl
if ($line =~ /^\s*\\end\{\Q$environment\E\}(.*?)$/) {
```

A line that carries content *before* the terminator therefore never matches, and
the reader consumes every remaining line — `\end{document}` included. The
document ends wherever the input does. **Nothing is reported**: from the reader's
point of view the environment is not unterminated, it merely ran out of file. The
whole tail of the paper (sections, `\bibliography`, appendices) is lost with zero
`Error:`.

Real `listings` terminates there — this is not an author error. Minimal trigger:

```latex
\documentclass{article}
\usepackage{listings}
\begin{document}
Before the listing.
\begin{lstlisting}
hello world \end{lstlisting}
AFTER-THE-LISTING-MARKER
\end{document}
```

Ground truth `pdflatex`: compiles cleanly (rc=0, no errors), renders `hello world`
as the listing's last line, then typesets `AFTER-THE-LISTING-MARKER` normally.

Same-host Perl 0.8.8 on that file: `Conversion complete: No obvious problems`,
but the marker is **absent** from the XML and the base64 `data` attribute of the
`<listing>` literally contains `hello world \end{lstlisting}\nAFTER-THE-LISTING-MARKER\n\end{document}`
— i.e. the environment ate the document. Rust behaved identically before the fix.

Witness `2605.11619`: a complete 54 KB paper whose listing body ends
`</body></html> \end{lstlisting}` silently lost its Conclusion, `\bibliography`
and appendix — 1.3 MB of HTML, 0 errors, 0 references.

**Fixed in Rust** (OXIDIZED_DESIGN #61): match `\end{<env>}` anywhere in the line;
text before it becomes the listing's final line, text after it is unread (as Perl
already does for the trailing part). **Upstream candidate** — the change is the
one regex on L316.

## 52. `Text::Balanced` reads `.bib` braces as escaped → one `\{` abandons every later entry

`Pre/BibTeX.pm` parses a brace-delimited value with `Text::Balanced`
(L19, L282):

```perl
while ((!defined($string = extract_bracketed($$self{line}, '{}'))) && $self->extendLine) { }
```

`extract_bracketed` honours `\` as an escape, so a value containing `\{Q\}`
never balances. The loop then keeps calling `extendLine` — swallowing line after
line to EOF — and the resulting parse error propagates out of `parseTopLevel`,
so **every remaining entry in the file is lost**, not just the offending one.

Real `bibtex` 0.99d knows nothing about `\` when scanning brace depth
(`bibtex.web`): it parses the same entry with at most a benign *"empty journal"*
warning, so the references exist in the author's PDF.

The same routine also excludes `\` from name characters, deliberately (L216):

> *"Especially `\`, which BibTeX allows, but it throws us off (semiverbatim vs
> verbatim) when we store the bibentries before digesting the key!"*

That does not dodge the hazard, it just loses the entry a different way: the key
in `@misc{apple\_rl,` ends at the backslash, and the bogus `\author={...}` field
name that follows kills its entry outright. BibTeX takes `apple\_rl` verbatim
and treats `\author` as an unknown field, keeping the entry.

Minimal trigger:

```bibtex
@article{chen2017,
  title = {Bounds on $\boldsymbol{\{Q\}}$},
  author = {Chen, A.},
}
@article{later2018, title = {This entry is lost too}, author = {Roe, B.} }
```

Perl LaTeXML on the escaped-brace reproducer: **0 bibitems, 2 dangling
citations** — it abandons the whole file. `bibtex` emits both entries.

Witness `2605.00264` (`\{Q\}` in `chen2017ucb`): 1144 of the file's 1170 entries
parsed, 18 dangling citations. Further witnesses: `2605.28695` (`ñ` in the key),
`2605.00121` (stray U+FE0F in the key), `2605.06974` (26 bare `@Comment`
banners), `2605.14212` (`\` in the key).

**Fixed in Rust** (OXIDIZED_DESIGN #60, and #58 for the resync): scan brace depth
the way `bibtex.web` does, ignoring `\`; admit `\` as a name character; resync at
the next `@` rather than abandoning the file. On 2605.00264 that is all 1170
entries and 0 dangling citations. **Upstream candidate** — but it is a rewrite of the
scanner, not a one-line change, since `Text::Balanced` cannot express
BibTeX's rule.

## 53. Raw `blkarray.sty` `\halign`-in-math degraded BOTH engines — ✅ both halves resolved

> **RE-MEASURED 2026-07-20 — the entry below is superseded on every engine claim.**
> * `blkarray_min.tex` on the current binary: **rc=0, "No obvious problems"** (the
>   `blkarray_sty.rs` binding shadows the raw `.sty`). Same-host **Perl: 0.6 s,
>   rc=0** — a bounded `too_many_errors` cap, *not* the "~90 s → rc=124 hang"
>   recorded below.
> * The `kbordermatrix` half is **FIXED** (2026-07-20) and was **never a
>   `stomach.rs::egroup` bug**: Rust inherits the real kernel `\@arraycr` from its
>   `latex.ltx` dump, which Perl does not have at all. Retracting it
>   (`Let!("\\@arraycr", "\\lx@alignment@newline")`) fixed the witness — 2605.23849
>   now 1.9 s / 0 errors. See WISDOM #64 and
>   [`kbordermatrix_halign_math/`](../known_crashes/kbordermatrix_halign_math/README.md).
> * So there is **no known residual `kbordermatrix` exposure**, and the shared
>   "LaTeXML's alignment × math-mode frame accounting cannot pop the per-cell
>   inline-math frame" diagnosis was never verified for either witness — treat it
>   as a hypothesis that did not survive.
>
> Retained below as the original record (it is still the best description of the
> *input* that triggers this, and of the pdflatex golden behaviour).

`blkarray`'s `block`/`blockarray` and `kbordermatrix` build a matrix with raw
`\halign`/`\ialign` whose column template wraps **each cell in inline math**
(`…$##$…`), digested inside surrounding display math. LaTeXML's alignment ×
math-mode frame accounting cannot pop the per-cell inline-math frame at the
alignment close, and the recovery re-enters and spins.

- **Perl**: on `blkarray` (a `block` with a paren-delimited spec `(cc)` nested in
  a `blockarray`) Perl **hangs ~90 s → rc=124 (terminated)** — same-host, with
  `--includestyles`. (On the `kbordermatrix` sibling Perl instead *completes* in
  ~0.4 s, so that one is Rust-only; blkarray degrades both engines.)
- **Rust**: cascades into a runaway that hits the 4500 MB memory cap →
  `Fatal:Timeout:MemoryBudget` at ~12 s (faster failure, same root).
- **pdflatex**: renders the matrix cleanly — the golden behaviour is well-defined;
  both LaTeXML engines are wrong.

Minimal trigger (`blkarray.sty` is in TeX Live):

```latex
\documentclass{article}\usepackage{blkarray}\begin{document}
\[\begin{blockarray}{cc}
\begin{block}{(cc)} 1 & 2 \\ \end{block}
\end{blockarray}\]
\end{document}
```

Dropping the `(`/`)` delimiter (`{cc}`) OR the `blockarray` wrapper converts in
0.2 s. **Fixed for blkarray** via a Rust binding
(`latexml_package/src/package/blkarray_sty.rs`) that shadows the raw `.sty` and
routes `blockarray`/`block` through the `array` machinery (surpass-Perl; Perl has
no binding): 1811.10792 (#594) OOM→0, 2310.17416 (#473) OOM→9. The `block`
sub-region delimiters are dropped (documented simplification — `array` can't wrap
a sub-region). ~~The **underlying** `stomach.rs::egroup` math-frame bug is unchanged
and still reachable via `kbordermatrix` (HIGH-DIFFICULTY, post-release).~~
*(Retracted — see the banner at the top of this entry.)* Full
analysis: [`docs/known_crashes/blkarray_halign_math/`](../known_crashes/blkarray_halign_math/README.md)
+ sibling [`kbordermatrix_halign_math/`](../known_crashes/kbordermatrix_halign_math/README.md).
