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

## A text-symbol CS (`\i`/`\j`) in a Semiverbatim argument hangs (SHARED — FIXED in Rust 2026-07-26)

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

**✅ FIXED IN RUST 2026-07-26 (surpass-Perl; Perl still hangs).** The cure is
the one this entry predicted — resolve the inner `\i` to `\OT1\i` — but
reached without needing `\fontencoding` to take effect inside the collect
loop. `\UseTextSymbol{#1}{#2}` (`latex_constructs.rs`, Perl
`latex_constructs.pool.ltxml:2642`) now expands to the encoding-specific
glyph CS `\csname #1\string#2\endcsname` **when that glyph is defined**,
keeping Perl's literal `{\fontencoding{#1}#2}` only as the fallback for when
it is not. That is not an invention: it is exactly what Perl's own
`\DeclareTextSymbolDefault` (`latex_constructs.pool.ltxml:2684-2688`) makes
`\?<cs>` expand to — the direct glyph, with no `\fontencoding` wrapper — so
the observable result matches Perl in every case Perl can reach.

**The dump is NOT the differentiator** — worth recording, because the obvious
guess is wrong. Measured 2026-07-26 against a format-equipped Perl 0.8.8
(`cd LaTeXML && cpanm --build-arg formats .`, which installs
`{plain,latex}_dump.pool.ltxml` beside the modules): Perl's own dump carries
`\?\i` → `\UseTextSymbol{OT1}\i`, 72 `UseTextSymbol` records. Perl has the
identical looping shape available. On that one install:

| trigger | Perl (with dumps) | Rust (before) | verdict |
|---|---|---|---|
| `\usepackage[pdfauthor={Mar{\'\i}n}]{hyperref}` | **hangs**, exit 124 | hangs | SHARED |
| `\cite{garcía2024key}` under `[OT1]{fontenc}` | converts, 0.89 s, `bibrefs="garcía2024key"` | `Fatal:Timeout` | **GENUINE-RUST-ONLY** |

So the second witness, **2606.11784**, is the same loop reached from a
*literal* non-ASCII character rather than an author-typed CS: fontenc's `.dfu`
maps `í` (U+00ED) onto the text-symbol chain and a `\cite` key is Semiverbatim.
It went from `Fatal:Timeout:PushbackLimit` with no output to 0 errors / 519 KB.
**Residual, unpinned:** *why* our `\cite`-key read reaches the encoding
dispatch when Perl's does not — the fix removes the loop shape for both, but
that read-path delta is still unexplained and deserves its own look.

Guard: `tests/encoding/textsymbol_semiverbatim` (pins that the literal and
`\'{i}` spellings converge). Causality checked by restoring Perl's literal body
at runtime on the fixed binary — the Fatal returns. Of the 25 `PushbackLimit`
papers in the 2605+2606 sandboxes only 2606.11784 carries a non-ASCII cite key,
so this repairs a failure *shape*, not that whole cluster.

Tracked in memory `robust-cs-semiverbatim-loop`. (Separately, a
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

**Amendment (user ruling 2026-09-29; 57bv): an enumeration attaches.** A comma run holding an ellipsis
(`\ldots`, `\dots`, `\cdots`, `\dotsc`/`b`/`m`/`i`/`o`, `...`) is one operand and joins the adjacent relation, as
Perl reads it (`maybeColRHS`/`maybeRHS`, MathGrammar:146-170): `i=1,\ldots,n` is `i = list@(1, ldots, n)`,
`\{h_i : i=1,\ldots,m\}` `conditional-set@(h _ i, i = list@(1, ldots, m))`, `x_1,\ldots,x_n\in X`
`list@(x _ 1, ldots, x _ n) element-of X`, `x_i\ge0,\ i=1,\ldots,n` `formulae@(x _ i >= 0, i = list@(1, ldots, n))`.
The right operand of the relation before the run when the run reaches its segment's end or ends in an item; the left
operand of the relation after it when it opens its segment; segments end at `\quad`-spaced punctuation, `;` and a
period. A run ending in its ellipsis between two relations stays (`a_1=0,\ldots,a_n=0`, `G=G_0,G_1,\dots,G_L=G'`),
and plain lists keep the rule above (`a=b,c,d`, `i=1,2`). A pass on the chosen parse (`attach_enumerations`,
semantics.rs), which adds no parse trees; "and so on" after a chain of relations stays an item (`x=0, y=1,
\ldots`). Two holes closed with it: a comma list opening with two relations then a
plain item had no derivation (`x=0, y=1, z` unparsed; `formulae_then_item_apply` reads `list@(x = 0, y = 1, z)` — a grammar rule whose left side
only relation pairs build, so no other comma list gains a derivation),
and a missing comma beside an ellipsis is supplied (DIVERGENCES #373). Golden `tests/parse/enumerations.tex`.
**57bv.1 (the 57bv A/B's ~39 worse formulas): where the run is plainly not the relation's operand, it stays an
item** — beyond Perl, whose `maybeRHS` attaches unconditionally: a tuple component's equation between delimiters
(its left operand does not continue the run's progression, or its right operand is one of the run:
`(j_1,\ldots,j_L,j_{L+1}=j_1)`, `(0,\ldots,0,k_\ell=k,0,\ldots,0)`, `x=(\nu^{(1)},\ldots,\nu^{(N)}=\nu)` stay
formulae@(…), named as `(a,b,c=d)` reads — Perl's `maybeRHS` attaches the left run here too; 2605.18633, 2605.09683,
2605.00514), a left operand past a text
(2605.23087), a right run after a relation whose left operand is a scripted member of it (`P_0=I,P_1,\dots,P_n`,
2605.23874), a run bridging to a relation whose left operand is a scripted member (`a=a_0,\dots,a_{n-1},a_n=b`,
2605.24348), a run after a relation that closes an elided run of relations (`A_1\lhd B_1,\ldots,A_n\lhd B_n,C_1,\ldots`,
2605.14476), and a lone ellipsis before a break whose next relation repeats the statement (`f(v_1)=f(v_2),\dots,\quad
f(v_{k-1})=f(v_k)`, 2605.00553). A bare `|` condition holds no statements after it;
since 57cc a relation that is no event after it relates the whole conditional (`y_i|\theta_i\sim P,\quad i=1,\ldots,n`
formulae@(conditional@(y_i, θ_i) ∼ P, i = list@(1, …, n)); DIVERGENCES #377). **57cc (user ruling 2026-09-29): a plain
run beside an attached enumeration attaches too**, in either direction, when its items continue the relation's value
(`i=1,\ldots,4,\,j=0,1,2` j = list@(0, 1, 2), `d=0,1,\,k=1,\ldots,K`; DIVERGENCES #378) — Perl's `maybeColRHS` reading,
kept from runs with no enumeration beside them.

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

Reported in the wild as [arXiv/html_feedback#6776](https://github.com/arXiv/html_feedback/issues/6776)
("the references are not loading") against **arXiv:2508.17585**
(`PMTCornersSpinor.tex`, amsrefs + a shipped `.bbl` of 34 `\bib` entries). The
deployed arXiv HTML — Perl-produced — carries `<ul id="bib.L1"
class="ltx_biblist"></ul>`, empty; same-host Perl reproduces it exactly
(`Warning:expected:bibkeys Missing bibkeys …`, 34 `<bibentry>` in the core XML,
0 `ltx_bibitem` after `latexmlpost`). pdflatex and Rust both render all 34.

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
>   [`kbordermatrix_halign_math/`](../archive/known_crashes/kbordermatrix_halign_math/README.md).
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
analysis: [`docs/archive/known_crashes/blkarray_halign_math/`](../archive/known_crashes/blkarray_halign_math/README.md)
+ sibling [`kbordermatrix_halign_math/`](../archive/known_crashes/kbordermatrix_halign_math/README.md).

## 54. `standalone.sty` requires a subimported child's class OPTIONS as packages

`standalone.sty.ltxml` L24-33 intercepts a sub-document's `\documentclass` and
`RequirePackage`s the comma-split **optional** argument:

```perl
DefPrimitive('\@standalone@documentclass[]{}', sub {
    my ($stomach, $packages) = @_;          # $packages = the OPTIONAL [] arg
    $stomach->bgroup;
    AssignValue(inPreamble => 1);
    for my $package (split(",", ToString($packages))) { RequirePackage($package); }
```

That argument holds **class options**, not package names, and the loop is
ungated, so any option that does not happen to name a package misses. Minimal
trigger (`index.tex` + `child.tex` in one directory):

```latex
% index.tex
\documentclass[12pt]{book}
\usepackage{import}\usepackage{standalone}
\begin{document}\subimport*{./}{child.tex}\end{document}

% child.tex
\documentclass[12pt]{article}
\begin{document}child\end{document}
```

Perl: `Warning:missing_file:12pt Can't find binding for package 12pt at child.tex;
line 1 col 1` → `Conversion complete: 1 warning; 1 missing file[12pt.sty]`.
`\documentclass[border=2pt]{standalone}` misses the same way. Content is never
lost — the damage is a false `missing_file` in the log and the missing-file tally.

The package being emulated disagrees: `standalone.sty` L604-614 consults a
subfile's class options only when the subfile's class is literally `standalone`
(and only under `obeyclassoptions`, which `\newif` defaults to **false**), then
feeds them to a keyval family rather than `\RequirePackage`.

**Fixed in Rust** (`latexml_package/src/package/standalone_sty.rs`): the loop is
gated on the class being `standalone` and on the option being one that
`standalone.cls` itself turns into a same-named package load (`tikz`, `pstricks`,
`preview`, `varwidth`, `multido`) — preserving upstream LaTeXML#1432's
`\documentclass[tikz]{standalone}`, which is why the loop exists. See
OXIDIZED_DESIGN #63; regression test
`06_cluster_regressions::standalone_subimport_documentclass_no_spurious_require`.
The mandatory half of the same defect (`\documentclass{article}` →
`missing_file:article`) was issue #293. Candidate to upstream. A second defect of
the same subfile group is entry #55 (a package loaded in the child's preamble
loses its definitions).

## 55. A package loaded inside a group loses its definitions while its document hooks survive

`standalone.sty.ltxml` L24-33 opens the subfile group at the child's
`\documentclass`, closes it at `\@standalone@end@input`, and — unlike real
`standalone.sty`, which *gobbles* the child preamble via `\sa@gobble` — executes
that preamble, so packages genuinely load inside the group. `import.sty.ltxml` L44-47 adds a second such group.

```latex
% index.tex
\documentclass[12pt]{book}
\usepackage{import}\usepackage{standalone}
\begin{document}\subimport*{./}{child.tex}\end{document}

% child.tex
\documentclass[tikz,border=2pt]{standalone}
\begin{document}
\begin{tikzpicture}\draw (0,0) -- (1,1);\end{tikzpicture}
\end{document}
```

Perl 0.8.8: `1 error; 1 undefined macro[\ifpgf@external@grabshipout]` (the
accompanying `missing file[border=2pt.sty]` is entry 54). The picture renders; the error
fires at the parent's `\end{document}`.

`tikz` raw-loads `pgfcoreexternal.code.tex`, whose L152
`\newif\ifpgf@external@grabshipout` is TeX-local (so it belongs to the child's
group) while its L171-179 `\AtEndDocument{\ifpgf@external@grabshipout…\fi}` goes
onto the **global** queue, flushed at the *parent's* `\end{document}`. Real LaTeX never gets here: `\@fileswithoptions` tests `\currentgrouplevel > \z@`
(latex.ltx L18700) and errors *"Loading a class or package in a group"* (L18702).

Reproducing needs a **raw-loaded** `.sty` (`--includestyles`): a package with a
Rust binding installs its definitions globally already, so a bound package cannot
exhibit this — the trigger above works because `tikz` raw-loads
`pgfcoreexternal.code.tex`.

**Fixed in Rust** at the package-load seam: `content.rs::require_package` hoists
the load past brackets LaTeXML itself opened. An author's own group is deliberately not
rescued (parity). Boundary, mechanism, refuted alternatives and guards:
OXIDIZED_DESIGN #65.
Issue #311. Candidate to upstream (not filed as of 2026-07-23).

## 56. `\includefrom` / `\subincludefrom` silently drop the included file

`import.sty.ltxml` L45/L47 declare one argument after the star but use `#3`:

```perl
DefMacro('\includefrom OptionalMatch:* {}',    '{\lx@set@path #1{#2} \include{#3}}');
DefMacro('\subincludefrom OptionalMatch:* {}', '{\lx@append@path #1{#2} \include{#3}}');
```

The undeclared `#3` expands to nothing, so `\includefrom{dir/}{file}` becomes
`\include{}` — content dropped with no error and no warning. Real `import.sty` L57/L58 route both
arguments through the same `\@doimport` as `\import`/`\subimport`, and Perl's own
`\import`/`\subimport` declare `{}{}` — a typo in the two `\include` variants.

**Fixed in Rust**: both prototypes take `{}{}`. Guard
`06_cluster_regressions::includefrom_takes_directory_and_file`. Candidate to
upstream (not filed as of 2026-07-23) — a two-character fix at
`import.sty.ltxml` L45/L47.

## 57. `setupPseudoBibitem` re-arming makes `\save@bibitem` a self-referential `\let` → infinite expansion

`latex_constructs.pool.ltxml:setupPseudoBibitem` (L4028-4032) saves the real
meanings before installing its "missing `\bibitem`" redirection:

```perl
Let('\save@bibitem', '\bibitem');
Let('\save@par',     '\par');
Let('\save@backbackslash', '\\\\');
Let('\bibitem', '\restoring@bibitem');
Let('\par',     '\par@in@bibliography');
Let('\\\\',     '\par@in@bibliography');
```

The captures are unconditional. If it runs a second time while the redirection
is still armed, all three save the *redirectors*: `\save@bibitem` becomes
`\restoring@bibitem`, whose body is
`\let\bibitem\save@bibitem\let\par\save@par\let\\\save@backbackslash\bibitem`
(L4067) — it points `\bibitem` back at `\restoring@bibitem` and then calls it.
That is an unconditional infinite expansion, not a slow document.

`\thebibliography` / `\endthebibliography` are `DefConstructor`s, not an
environment ("Should be an environment, but people seem to want to misuse it"),
so a *bare-CS* pair opens no group and the arming survives it. Minimal trigger —
hangs Perl 0.8.8 (>400 s on 8 lines; the same file converts in <2 s once the
double-arm is broken):

```latex
\documentclass{article}
\begin{document}
\thebibliography{9}
\endthebibliography
\thebibliography{9}
\bibitem{b} Author B.
\endthebibliography
\end{document}
```

The first bibliography must contain no `\bibitem` — one there fires
`\restoring@bibitem`, which disarms and makes the second capture legitimate.
`\ifx\save@bibitem\restoring@bibitem` after the second `\thebibliography` is
true in **both** engines, so the latent defect is shared.

It stays latent upstream because Perl's biblatex binding
(`ar5iv-bindings/biblatex.sty.ltxml`) never defines `\printbibliography`, so
Perl never reads a real `.bbl` through this path. Rust's binding did until batch
56jc, and a biber `.bbl` reached it routinely: biblatex's apa style requests two
sorting schemes, so the `.bbl` carries **two `\datalist` blocks** with the same
references, and each `\enddatalist` expanded to a whole bare
`\thebibliography…\endthebibliography` (`bib_as_thebibliography`, mirroring
Perl's `biblatex_as_thebibliography` L105-119). Since 56jc the `.bbl` is read into
bibentries (OXIDIZED_DESIGN_DIVERGENCES #306); a hand-written bare pair still
reaches it.

**Fixed in Rust**, in two symmetric halves:
* `setup_pseudo_bibitem` guards the three captures on
  `\ifx\bibitem\restoring@bibitem` — capture the originals once per arming.
* `\endthebibliography` now *disarms* (the same three `\let`s
  `\restoring@bibitem` performs, minus its trailing `\bibitem`). Upstream has no
  teardown, relying on `\begin`/`\end{thebibliography}` popping the group; that
  never covers the bare-CS pair, so the redirection outlived the bibliography
  and the next `\par` — a blank line after `\printbibliography` — expanded to
  `\par@in@bibliography` and deposited a stray empty `\save@bibitem{}` outside
  the biblist (`Error:malformed:ltx:bibitem <ltx:bibitem> isn't allowed in
  <ltx:p>`). Restoring is a no-op for the grouped shape and for a bare
  `\thebibliography` with no closer.

Witness arXiv 2605.17646 (biblatex apa, 2 × 29 entries, blank line after
`\printbibliography`): `Fatal:Timeout:TokenLimit` at 1e9 tokens with no output →
now converts with 1 error (`\missing{Cowen2021}`, undefined in both engines) and
58 bibitems / 2 bibliographies / 2 biblists, matching same-host Perl exactly
(Perl: 59 errors, 33.7 s). Guard
`06_cluster_regressions::cluster_biblatex_two_datalists`. Candidate to upstream
(not filed as of 2026-07-25).
## 58. A listing's trailing-empty-line trim discards the braces that close it

`listings.sty.ltxml` L1330 trims trailing blank lines by slicing the generated token
vector:

```perl
@LaTeXML::lsttokens = @LaTeXML::lsttokens[0 .. $LaTeXML::emptyfrom - 1] if $LaTeXML::emptyfrom;
```

`$emptyfrom` is a token index, not a structural boundary. Any delimited class still
open there (string, comment, styled span) has its closing `}` in the discarded tail,
so the listing body is emitted with unclosed groups and `\@@listings@block` reads its
arguments past the end of the document.

Trigger: `lastline=N` on a file with **more** than N lines — the line-skipping loop
consumes the rest without closing what the last rendered line left open.

```latex
\documentclass{article}\usepackage{listings}
\begin{document}
\lstinputlisting[lastline=3]{four_line_file.py}
Text after the listing.
\end{document}
```

Perl: `Error:expected:{} Missing argument {} for Core::Definition::Constructor
[\@@listings@block {}{}{}]`, ×7 on the witness. **Both engines lose the snippet.**

**Fixed in Rust** (OXIDIZED_DESIGN #68): truncate as Perl does, then re-close whatever
the cut left open — the discarded region is by construction only empty-line markup.
Witness arXiv 2412.04705 (arXiv/html_feedback#6735): 22 errors → **0**, while
same-host Perl still reports 15. Guard
`104_lstinputlisting_range_crlf::lastline_shorter_than_file_does_not_swallow_the_document`.
Candidate to upstream (not filed as of 2026-07-25).

## 59. A CRLF listing source makes line-comment styling bleed down the file

`listingsReadRawFile` slurps the file verbatim, but every end-of-line test in the
listings processor is written against `\n` (the `__NEWLINE__` comment-close test, the
blank-line test in `lstProcessStartLine`, the line-skipping loops). A `\r` before the
`\n` defeats them all, so a line comment never terminates and its style bleeds over
every following line. The `ltx_lst_comment` class wrapper *does* close — only the
font/colour group leaks — so the bug is invisible if you inspect classes rather than
`font`/`color`.

```latex
\lstdefinestyle{s}{morecomment=[l]{\#},commentstyle=\itshape}
\lstinputlisting[style=s]{crlf_file.py}   % every line comes out italic
```

TeX never sees the CR (its file reader strips the terminator and appends
`\endlinechar`), so this is a slurp that skips what the engine does.

Ground truth on arXiv 2412.04705 (CRLF Python sources): pdflatex renders only the `#`
line in comment green — 9 green vs 69 black glyph groups on the page — while both
LaTeXML engines paint the whole snippet green and slanted. Pre-fix A/B confirms the
cause: an LF copy of the same file renders correctly, the CRLF original does not.

**Fixed in Rust** (OXIDIZED_DESIGN #69): normalize `\r\n` and lone `\r` to `\n` on
read. Guard
`104_lstinputlisting_range_crlf::crlf_line_comment_style_does_not_bleed_past_its_line`.
Candidate to upstream (not filed as of 2026-07-25).

## 60. A BibTeX `type` field is APPENDED to the entry-type label instead of replacing it

Real BibTeX treats `type` as an **override**: in `@techreport`/`@phdthesis`/
`@mastersthesis`/`@inbook`, `type = {...}` replaces the default label
("Technical report", "PhD thesis", "chapter"). `plain.bst` implements this as
`format.tr.number`: `type empty$ { "Technical report" } { type } if$`.

LaTeXML renders both. The value lands in `ltx:bib-type` (`BibTeX.pool.ltxml`
L1544 `\bib@field@default@type`), while the format spec independently carries an
unconditional `"Technical Report "` prestring in front of
`ltx:bib-part[@role='number']` — so the two concatenate.

```bibtex
@techreport{pr, author={Page, L.}, title={PageRank}, year={1998},
  institution={Stanford}, type={Technical Report}, number={SIDL-WP-1999-0120} }
```

Both engines render, byte-identically:

```
Technical Report Technical Report SIDL-WP-1999-0120 , Stanford
```

where real BibTeX prints the label once. Witness arXiv 2607.00052 (the PageRank
entry in `main.bib`); it is the ONLY `type` field across the nine 2607 witness
`.bib` files, so the practical blast radius is small.

**PARITY — not fixed.** Rust only started showing it on 2026-07-25, when the
`.bib` emitter stopped dropping the `type` field altogether (previously the
duplication was hidden by losing the field, which also lost genuinely distinct
types like `type = {Technical Memo}` — a strictly worse trade). Suppressing the
prestring when `ltx:bib-type` is present would make Rust emit less than Perl on
the same input, i.e. a surpass-Perl divergence needing explicit authorization.
Candidate to upstream (not filed as of 2026-07-25).

## 61. `do_names_short` is dead code — the author-year label carries every author

`LaTeXML/lib/LaTeXML/Post/MakeBibliography.pm` defines exactly the helper the
author-year citation label needs:

```perl
sub do_names_short {
  my (@names) = @_;
  if (@names > 2) {
    return ($names[0]->childNodes, ' ', ['ltx:text', { class => 'ltx_bib_etal' }, 'et al.']); }
  elsif (@names > 1) {
    return ($names[0]->childNodes, ' and ', $names[1]->childNodes); }
  elsif (@names) {
    return ($names[0]->childNodes); } }
```

It is **never called** — `grep do_names_short` finds only the definition (L586).
The `role="refnum"` `ltx_bib_author-year` label instead goes through
`do_authors`→`do_names` (L505-517, L568-584), which emits **every** author and
says "et al." only when the BibTeX field literally ends `and others`. The
author-year branch then drops the entry's first block
(`shift(@blockspecs); # Skip redundant 1st block!!`) on the grounds that the
authors are already in the label.

For a collaboration paper the result is a citation label thousands of characters
long which IS the entry. Witness arXiv 2607.21432 (A&A, Simons Observatory):
a **5104-character** label; 9 of its 19 entries exceed 120 characters. Reader
report: [arXiv/html_feedback#6797](https://github.com/arXiv/html_feedback/issues/6797).
Reproduce with any `.bib` entry of >2 authors that does not end `and others`,
under an author-year citestyle (the bibliography must be built from the `.bib` —
LaTeXML does not interpret `.bst`).

That this is an oversight rather than a considered style is corroborated inside
the same file: the `role="authors"` tag already truncates at `>2` (L433-437), so
the full-list label contradicts its neighbour; and BibTeX itself disagrees —
running the witness's `aa.bst` puts the SHORT form in `\bibitem[…]` (natbib's
`Abitbol {et~al.}(2025)`) and prints the authors in the entry body, the long
surname list being only natbib's optional `\citet*` form.

**Fixed in Rust** by calling the short form for the label and keeping the first
block, so the full author list survives in the body (max label on the witness
5104 → 48 chars). Intentional divergence OXIDIZED_DESIGN #71, guard
`cluster_bib_long_author_list_refnum`. Candidate to upstream — the fix upstream is
to route the refnum through the already-present `do_names_short` and stop
skipping the first block.

## 62. `\href` in a **Semiverbatim** argument expands forever (`doi = {\href{…}{…}}` hangs `latexmlc`)

`hyperref.sty.ltxml` expands `\href` into a stream that re-emits `\href`:

```perl
DefMacro('\href HyperVerbatim {}', '\lx@hyper@url@\href{}{}{#1}{#2}');
```

The re-emitted `\href` exists only to fill `\lx@hyper@url@`'s reversion slot
`#1` (`Undigested`), and is normally consumed as an argument without expanding.
But `Core/Parameter.pm` L123-132 pre-expands a **semiverbatim** argument before
digesting it —

```perl
# If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
while (defined(my $token = $gullet->getPendingComment || $gullet->readXToken(1))) {
```

— and `readXToken(1)` sets `$fully_expand = $toplevel = 1`, which by
`Core/Gullet.pm` L408-409 expands even a `isProtected` definition. That pass
linearizes tokens one at a time and never reaches `\lx@hyper@url@`'s parameter
list, so `\lx@hyper@url@` is kept (a Constructor, not expandable) and the
re-emitted `\href` is expanded again: `\href` → `\lx@hyper@url@\href{}{}…` →
`\href` → … unbounded.

Minimal trigger — a 7-line `.bib`, since `\bib@field@default@doi` reads
`Semiverbatim` and INSPIRE exports DOIs wrapped in a link:

```bibtex
@article{K,
  author = {Doe, Jane}, title = {{T}}, journal = {J}, year = {2021},
  doi = {\href{https://doi.org/10.5281/zenodo.19852912}{10.5281/zenodo.19852912}},
}
```

```latex
\documentclass{article}\usepackage{hyperref}
\begin{document}\cite{K}.\bibliographystyle{plain}\bibliography{thatfile}\end{document}
```

Measured same-host on an IDLE box (1-min load 4.5), as an A/B against the
identical document with the `\href` removed from the `doi` field:

| `latexmlc` on | wall | status |
|---|---|---|
| `doi = {10.5281/zenodo.19852912}` | **3.7 s** | `Status:conversion:0` |
| `doi = {\href{https://doi.org/…}{…}}` | **439 s, killed (rc=124)** | `Status:conversion:3` |

so it is a hang, not slowness. Perl `latexml` alone is unaffected only because
it never reads the `.bib`; the loop is in the post-processing bibliography
session. Witnesses arXiv 2605.00181, 2605.19650, 2606.06645 (each has exactly
this `doi = {\href{…}{…}}`), which Perl `latexml` converts cleanly in 8-28 s.

Related to entry 57 (`\save@bibitem`): both are a definition whose expansion
names itself, surviving only where nothing re-expands it.

**Fixed in Rust** in `hyperref_sty.rs` by putting the command NAME in the
reversion slot as an OTHER-catcode token rather than the live control sequence —
which is what the sibling `\url` path (`\lx@hyper@url`) already does
(`Tokens!(cmd.as_other())`). Inert under every expansion regime, and it
stringifies and reverts identically. Guard
`href_in_semiverbatim_bib_field_does_not_loop`
(`latexml_oxide/tests/59_href_semiverbatim_loop.rs`); the `\edef`/`\xdef` half of
the same defect is guarded by `58_href_edef_loop.rs`. Candidate to upstream —
the same one-token change applies to `hyperref.sty.ltxml`.

---

## 63. An unclosed math region swallows the rest of the document — in BOTH engines

**Symptom.** One macro-level breakage inside `$…$` / `\[…\]` / an
`align`-family body leaves the math group open. Digestion never returns to text
mode, so every following `_`, `^`, `&`, `\end{…}` and section break is an error
and the whole remainder of the document lands inside a single `<ltx:XMath>` — the
leak surfaces in `<ltx:title>`, `<ltx:tag>`, `<ltx:proof>`. The engine then hits
its `too_many_errors` circuit breaker and produces no document at all.

**This is shared, not a Rust regression.** Classified 2026-07-27 against
same-host Perl 0.8.8 (`/usr/local/bin/latexml`, verbose — never `--quiet` —
`--preload=ar5iv.sty --path=ar5iv-bindings/bindings`, the fleet's ar5iv
profile, each paper's cortex-chosen main file). **At shipped defaults Perl goes
`Fatal:too_many_errors` on all eleven witnesses**, at 101 errors + the fatal;
we go fatal at 1001 + the fatal, because `tikz.sty` raises our `MAX_ERRORS` to
1000 while Perl's `Core/State.pm` L96 default of 100 has no override anywhere in
its tree. Same severity, same outcome, different amount of diagnostics on the way
down.

Lifting both caps (Perl: a preloaded binding doing
`AssignValue('MAX_ERRORS'=>100000)`; Rust: a throwaway patch to the two circuit
breakers in `common/error.rs`) shows the flood itself is the same flood — same
first error, same classes, frequently the same counts:

| witness | Perl uncapped | Rust uncapped | first error (both engines, same site) |
|---|---|---|---|
| 2605.03113 | 1956 | 1953 | `undefined:\overarrow@` — amsmath internal behind a hand-rolled `\overrightharpoon` via `\mathpalette` |
| 2605.05934 | 4211¹ | 3604 | `\lx@begin@alignment` in math — mhchem `\ce{}` inside a `\bea`/`\eea` alignment (see also #53) |
| 2605.07772 | 100002² | 2040 | `undefined:\usephysicsmodule` (physics2) |
| 2605.09261 | 926³ | 1079 | custom tikz `diagram` env inside `align*` |
| 2605.11190 | 19984 | 1159 | `\input` of a tikz `\matrix` whose cells carry `$…$`, inside `align*` |
| 2605.12930 | 2842 | 1097 | tikz-cd "Diagrams cannot be nested" |
| 2605.15522 | 614 | OOM⁴ | mathtools `\DeclarePairedDelimiterX` starred form spanning lines |
| 2605.15678 | 1976¹ | 2024 | `undefined:\nin`; author `\def\({\left(}` `\def\){\right)}` |
| 2605.23308 | 1055 | OOM⁴ | `\g{c}{summ}` custom macro inside `align*` |
| 2605.30732 | 1137 | 1173 | `\brackets{…}` group leak inside `equation*` |
| 2606.01903 | 258 | 1810 | `undefined:\ext@arrow` — **the one Rust-only case, see below** |

¹ killed at the harness timeout, count is a floor. ² hit the raised cap.
³ Perl ends in `Fatal:perl:deep_recursion` rather than finishing.
⁴ with the circuit breaker removed the runaway exhausts RAM and is SIGKILLed —
which is what the breaker exists to prevent.

Two rows are worth reading as exact matches rather than "same order of
magnitude". 2605.05934, uncapped on both sides: `XMHint` 260,
`\lx@end@inline@math` 189, `unexpected:_` 71, `bibitem` 64,
`\lx@begin@alignment` 45 — five classes identical, and the same first error at
the same line:col. 2605.23308, our *capped* run against Perl uncapped:
`unexpected:_` 160 = 160, `unexpected:^` 104 = 104, `\lx@begin@alignment`
54 = 54 — i.e. we saturate before we can diverge.

**The one exception, fixed:** `2606.01903` was GENUINE-RUST-ONLY. Perl has no
`\ext@arrow` binding at all, so it errors once and recovers; we bind it, and the
binding read four of its seven undelimited arguments as `Token`, splitting
`extpfeil`'s braced `\mkern` amount `{40}` and spilling a `}` that closed the
display math. Fixed (OXIDIZED_DESIGN #81, guard
`06_cluster_math::cluster_ext_arrow_braced_mkern`): 1002 errors + fatal → **0**.
Grepping the 536 currently-fatal 2605+2606 papers for
`ext@arrow|extpfeil|newextarrow` found one more of the same shape, 2606.14212
(`\xtwoheadrightarrow` in an `align*`): 194 errors + fatal → **0**, against 3
in Perl.

**Not to be "fixed" into a divergence.** The remaining ten need their individual
undefined internals (`\overarrow@`, `\underarrow@`, `\usephysicsmodule`, …)
implemented before the math can close — each of those is a capability Perl also
lacks, so adding them is beyond-Perl work, not parity work. Do not mistake the
1000-vs-100 error-cap difference for a divergence: it is a diagnostics budget,
and both engines fail these papers.

---

## 64. A LaTeX **kernel** command before `\documentclass` is undefined (the class is never selected)

In real LaTeX there is no "before the kernel": `latex.ltx` *is* the format, so
every kernel command is live from token one. LaTeXML instead loads `LaTeX.pool`
lazily, on first sight of a *trigger* control sequence — the hand-maintained
list in `TeX.pool.ltxml` L33-56:

```perl
foreach my $ltxtrigger (qw(documentclass
  newcommand renewcommand newenvironment renewenvironment
  NeedsTeXFormat ProvidesFile
  ProvidesPackage RequirePackage PassOptionsToPackage
  makeatletter makeatother
  typeout begin listfiles nofiles)) {
  DefAutoload($ltxtrigger, 'LaTeX.pool.ltxml'); }
```

Any kernel command **not** on that list is simply undefined at that point, gets
`generateErrorStub`'s `<ltx:ERROR/>`, and its arguments leak into the stream.
The list has grown one witness at a time and its gaps are arbitrary:
`\PassOptionsToPackage` is there but `\PassOptionsToClass` is not;
`\newcommand`/`\renewcommand` are there but `\providecommand` is not;
`\IfFileExists`/`\InputIfFileExists` are absent entirely.

The damaging case is the completely standard "use this class if installed"
idiom, because the collapsed conditional means **no class is ever selected** —
and worse, both branches leak, so the *first* (wrong) `\documentclass` wins:

```latex
\IfFileExists{ltxo-no-such-class.cls}{\documentclass{ltxo-no-such-class}}{\documentclass{article}}
\begin{document}
Selected the fallback class.
\end{document}
```

Same-host Perl `latexml` (v0.8.8) on that four-line file:

```
Error:undefined:\IfFileExists ... at perl_probe.tex; line 1 col 14
Warning:missing_file:ltxo-no-such-class Can't find binding for class ltxo-no-such-class (using OmniBus)
Error:undefined:\warn@unusedclassoptions ... at perl_probe.tex; line 2 col 1
```

— i.e. it picks `class="ltxo-no-such-class"`, the branch that was supposed to be
*rejected*. The second trigger, `\providecommand`/`\PassOptionsToClass` before
`\documentclass`, is the same defect without the class damage:
`Error:undefined:\PassOptionsToClass`, `Error:undefined:\providecommand`, and
then `Error:undefined:` for every macro the lost `\providecommand` should have
defined.

On a real paper the class loss cascades: witnesses arXiv 2605.25877
(`\IfFileExists{proc-l.cls}{…}{\documentclass{amsproc}}`) and 2606.06905
(`siamart251216.cls`, same idiom) both hit **101 errors + `Fatal:TooManyErrors`,
no class at all**. Also 2606.09693, 2606.16723. Seven papers across sandbox
corpora 2605+2606 have `undefined:\IfFileExists` as their FIRST error.

**Fixed in Rust** generally rather than by extending the list, which would only
move the gap. `latexml_engine/src/latex_kernel.rs` registers a hook consulted at
the two undefined-CS paths (`gullet::read_x_token`, `stomach::
invoke_token_undefined`) *before* the error is raised: if the ambient kernel
dump defines the control sequence, load `LaTeX.pool` and retry the token; else
take the ordinary bounded `Error:undefined` path. Fires at most once per
session, never during `--init` dump-build, and not at all on the degraded
no-dump branch of `LoadFormat('latex')`. It also retired the two Rust-only
trigger accretions (`\UseRawInputEncoding`, `\DocumentMetadata`). Guards
`preclass_iffileexists_test` / `preclass_kernel_cs_test`
(`latexml_oxide/tests/structure/`) and
`nodump_leaves_pre_documentclass_kernel_cs_undefined`
(`latexml_oxide/tests/cluster_package_guards/preclass_kernel_autoload.rs`).

Candidate to upstream, though not as a straight port: Perl has no dump to use as
the membership oracle, so the upstream-shaped fix is to extend
`TeX.pool.ltxml`'s list with at least `IfFileExists InputIfFileExists
PassOptionsToClass providecommand`.

---

## 65. `\meaning` of a `\chardef` token prints the value in DECIMAL, and says `\char` for `\mathchardef`

**Perl source:** `LaTeXML/Engine/TeX_Debugging.pool.ltxml` lines 166-168

```perl
elsif ($type =~ /chardef$/i) {    # from \chardef or \mathchardef
  my $prefix = ($$definition{mathglyph} ? '\mathchar' : '\char');
  $meaning = $prefix . '"' . $definition->valueOf->valueOf; }
```

**Symptom:** two deviations from real TeX, both benign in isolation but wrong
for packages that parse `\meaning` to recover a character code.

1. **Decimal, not hex.** `tex.web` L22897-22899 prints the value with
   `print_hex`, i.e. `"` followed by *uppercase hexadecimal*. Perl interpolates
   `valueOf->valueOf`, a Perl integer, so it renders decimal.
2. **`\char` for a `\mathchardef`.** `tex.web` L22899 prints `\mathchar` for the
   `math_given` command code. Perl's ternary keys off `$$definition{mathglyph}`,
   but `Core/Definition/CharDef.pm` L32-35 blesses only
   `cs/parameters/mode/value/encoding/registerType/readonly/locator` — no
   `mathglyph` key is ever set on a CharDef — so the `\mathchar` arm is
   unreachable and every chardef reports `\char`.

**Minimal example:**
```tex
\newcount\mycnt  \mycnt="41
\chardef\chA\mycnt
\mathchardef\mcA="0141
\meaning\chA   % TeX: \char"41      Perl/Rust: \char"65
\meaning\mcA   % TeX: \mathchar"141 Perl/Rust: \char"321
```

**Real-world consequence:** `bxcoloremoji.sty` L1366-1386 builds emoji tag
codepoints as `E00` concatenated with the `\meaning` tail, expecting hex — so
`@A` resolves to `E0065` instead of `E0041` (a different tag character). Only
the rarely used `@!`..`@~` tag range is affected; nothing errors.

**Kept as-is in Rust** — deliberately. `latexml_engine/src/tex_debugging.rs`
ports Perl exactly (`\char` + decimal, unconditionally), because `\meaning`
output feeds goldens copied from Perl and every corpus baseline; switching to
hex is a behaviour change with corpus-wide blast radius, not a local fix. The
Rust `Register` *does* carry a decoded `mathglyph`, so the `\mathchar` arm could
be revived at any time — that is the divergence the comment at the fix site
warns against taking accidentally.

Note this entry is about the *format* only. The Rust port separately had no
chardef arm at all and returned the internal class name `Register`, dropping the
`"` that packages split on; that was a Rust-only defect, fixed with guard
`meaning_chardef` (`latexml_oxide/tests/expansion/meaning_chardef.{tex,xml}`).
Candidate to upstream: both deviations are one-line fixes (`sprintf('%X')` and
threading the math flag), but they change observable `\meaning` output.

## 66. acmart `\Description` emits the OPTIONAL short argument and discards the mandatory long one

`LaTeXML/lib/LaTeXML/Package/acmart.cls.ltxml` L78-86:

```perl
DefConstructor('\Description[]{}', '^^<ltx:note xml:id="#id" class="ltx_nodisplay">#1</ltx:note>',
  properties => sub { ('width' => Dimension(0), 'height' => Dimension(0), RefStepCounter('acmlabel')) },
  beforeConstruct => sub {
    my ($document, $whatsit) = @_;
    # TODO: Is there something useful to do with the short description in our schema?
    ... $document->setAttribute($figure, 'aria:labelledby', $whatsit->getProperty('id'));
```

For `\Description[]{}` the parameters are `#1` = **optional** short description,
`#2` = **mandatory** long description. The template emits `#1`. The long
description — the extended alternative ACM actually mandates, and the reason the
command exists — is digested and then dropped. Upstream's own `TODO` asks what
to do with "the short description", suggesting `#1` was believed to be the main
one. `\Description[S]{L}` emits `S` and loses `L`; `\Description{L}` emits
nothing at all.

Confirmed on `LaTeXML/t/complex/acm_aria.tex` (whose golden `acm_aria.xml`
records the defect) and on arXiv **2607.21760** — an ACM accessibility paper
with four figures and zero descriptions in its HTML.

**`aria:labelledby` on the float is a second defect.** acmart's documentation
says "Unlike `\caption`, which is used alongside the image, `\Description` is
intended to be used **instead of** the image", i.e. it is a *text alternative*,
which in ARIA is name-like — so pointing a name relation at it reads as
defensible, and this entry originally said so. It is not: `aria-labelledby`
sets the accessible **name**, and a float's name is its caption, so the
relation displaces "Figure 1. caption text" and hides the caption from a screen
reader. The alternative also belongs to the *image*, not to the float that
contains it. Reported in review on brucemiller/LaTeXML#430 (`r3674103638`);
Rust now puts the text on the lone `ltx:graphics` as `@alt` and never emits a
name relation (`OXIDIZED_DESIGN_DIVERGENCES.md` #83).

Two further problems do stand:

1. **`ltx:note` carries footnote decoration.** `LaTeXML-meta-xhtml.xsl` wraps a
   note in a `†` mark plus a `<role>: ` type prefix, and because the note is the
   *name* target, all of that lands in the computed accessible name —
   "†† : Fly 1 and Fly 2 look identical".
2. **The argument is digested although the class gobbles it.** `acmart.cls`
   L895 is
   `\newcommand\Description[2][]{\global\@Description@presenttrue\ignorespaces}`,
   so pdflatex never expands the description and an author cannot see a defect
   inside it. Digesting therefore manufactures errors invisible in the normal
   workflow: 2607.21760 writes `\D1 … \D5` inside `\Description` (a copy-paste
   slip from the adjacent `alt=` text, which has plain `D1 … D5`) and both
   engines report `Error:undefined:\D` — for content they then discard.

**Fixed in Rust, deliberately diverging** (`latexml_package/src/package/acmart_cls.rs`;
see `OXIDIZED_DESIGN_DIVERGENCES.md` #83): the description is read `Undigested`
so nothing inside it expands; where the float holds a lone image the short form
becomes that image's `@alt` and the long form an `aria:describedby` block, and
where it holds none or several (an empty float, a table, a multi-panel figure)
both stay referenced from the float itself — never as a name relation, so the
caption is always what names the float. A dedicated XSLT template strips the
footnote scaffolding. `acm_aria.xml` was re-blessed — it previously matched Perl
byte-for-byte and so certified the defect.

Candidate to upstream: swapping `#1`→`#2` is a one-token fix; the
note-decoration and undigested-reading parts need the XSLT and parameter-type
changes too.

## 67. `do_year`'s bibliography disambiguation suffix is dead code — a sigil mismatch

`LaTeXML/lib/LaTeXML/Post/MakeBibliography.pm` binds the entry's disambiguation
letter as a **scalar** and reads it back as an **array**:

```perl
# L417, in formatBibEntry:
local $LaTeXML::Post::MakeBibliography::SUFFIX = $$entry{suffix};
# L613-615, in do_year:
return (' (', @stuff, @LaTeXML::Post::MakeBibliography::SUFFIX, ')');
```

`$Pkg::SUFFIX` and `@Pkg::SUFFIX` are different Perl variables. The array is
never assigned anywhere in the distribution (`grep -rn '@SUFFIX' LaTeXML/lib/`
is empty), so it always interpolates to nothing and the letter never reaches the
entry body — only the refnum label, which reads `$$entry{suffix}` directly.

**Minimal trigger** — two entries sharing an author+year (`latexml_oxide/tests/
cluster_regressions/bib_alpha_style.{tex,bib}`, entries `wide1`/`wide2`), under
`\bibliographystyle{alpha}`. Same-host Perl LaTeXML 0.8.8 renders

```html
<span class="ltx_tag ltx_bib_abbrv …">[SBC99a]</span> … <span class="ltx_text ltx_bib_year"> (1999)</span>
```

— suffix on the label, bare year in the body.

**Not fixed, in either engine, and that is deliberate.** The dead code's evident
intent (a disambiguated body year) is what author-year styles like
`apalike.bst` print, but the styles that actually reach this branch do not want
it: `alpha.bst` prints the bare `1999` in the entry body, exactly as Perl
already does. And Rust's author-year branch drops the first block's year
outright (OXIDIZED_DESIGN #71), so "fixing" the sigil would change output *only*
for the alpha and numeric styles — precisely where it would be wrong. Rust
therefore matches Perl's behaviour, with the reason recorded at the seam
(`make_bibliography.rs`, `Formatter::Year`) and pinned by
`06_cluster_bibliography::cluster_bib_alpha_style_labels`.

Worth knowing because the *source* reads as though the suffix is emitted: an
audit item in `BIBLIOGRAPHY_WORKLIST.md` ("`Formatter::Year` drops the
disambiguation `@SUFFIX`") was opened off that reading and closed only when the
Perl output was measured.

Candidate to upstream: deleting the dead `@…::SUFFIX` interpolation, or a
style-conditional emission. Not a one-token `$`/`@` swap — that would change
alpha-styled output for the worse.
## 68. An arg-taking `\fnum@<type>` swallows the caption's closing brace and absorbs the rest of the document

`LaTeXML/lib/LaTeXML/Engine/Base_Utility.pool.ltxml` L1041-1043 expands the
author's caption-number hook bare:

```perl
DefMacro('\lx@fnum@@{}',
  '{\normalfont\@ifundefined{fnum@font@#1}{}{\csname fnum@font@#1\endcsname}'
    . '\@ifundefined{fnum@#1}{\lx@@fnum@@{#1}}{\csname fnum@#1\endcsname}}');
```

Real `\fnum@<type>` takes no argument. But LaTeX's `\@makecaption` is
`\sbox\@tempboxa{#1: #2}`, so a **one-argument** `\fnum@<type>` eats the `:`
that follows it — which is exactly the point of the widely-copied "change
`Fig. 1:` to `Fig. 1.`" hack:

```tex
\makeatletter
\renewcommand*{\fnum@figure}[1]{\figurename~\thefigure.}
\makeatother
```

LaTeXML has no `:` **token** to eat: its separator is a tag **attribute**
(`\lx@tag[][: ]`, `latex_constructs.pool.ltxml` L3158-3159). So the argument
scan runs past the hook and takes the caption group's closing brace instead. The
`<figure>` never closes, and **every following section — the bibliography
included — is absorbed into it**, which is why the symptom presents as a
truncated document with no References section rather than as a bad caption.

**Minimal trigger** (`latexml_oxide/tests/cluster_regressions/fnum_arg_hook.tex`
covers all three hooks; plain `article` suffices — `cas-sc` is not implicated):

```tex
\documentclass{article}
\makeatletter
\renewcommand*{\fnum@figure}[1]{\figurename~\thefigure.}
\makeatother
\begin{document}
\section{First}
\begin{figure}\caption{A caption.}\end{figure}
\section{Second}
Text after the figure must survive.
\end{document}
```

Measured on that input: **pdflatex 0 errors** (renders `Figure 1. A caption.`),
**Perl LaTeXML 0.8.8 nine errors**, pre-fix Rust seven — same
`\lx@tag@intags` / `\lx@tag` / `\end{figure}` "Attempt to end mode
restricted_horizontal" signature in both engines. Witnesses `2605.01731`
(18 figures × 3 errors) and `2605.12842` (10 × 3), both confirmed live on the
current fleet run. **Breadth is smaller than once recorded:** a 2026-07-14 note
claimed 18 papers from a `grep 'lx@tag@intags'` proxy; re-measured 2026-07-29
that proxy gives 23 papers across sandbox-arxiv-2605+2606 (60,505 docs) of which
only **2** carry this cause's actual signature — the symptom has several causes,
so the proxy over-attributes.

**Fixed in Rust** as a deliberate surpass — `OXIDIZED_DESIGN #85`: the hook is
expanded as `\csname fnum@#1\endcsname{}`, giving an arg-taking definition a
harmless empty group and reproducing pdflatex's result, while the 0-arg hooks
that are the normal case are unaffected. Guard
`06_cluster_regressions::cluster_fnum_arg_hook`.

Reported upstream as **brucemiller/LaTeXML#2856**: the one-token change applies
verbatim to the Perl definition, and to `\lx@fnum@toc@@` L1065-1066 and the
theorem-header formatter alongside it. Note the fix does NOT reach the `close=": "` separator, so the
caption still reads `Figure 1.: A caption.` in both engines — closing that gap
needs the tag attribute to become conditional, which is a larger change.

## 69. `fill_in_relations` walks EVERY ancestor's siblings — quadratic navigation on a split mega-document

Upstream `LaTeXML/lib/LaTeXML/Post/CrossRef.pm` L106-122 emits a `<link rel=…>`
for the siblings of the page, then of its parent, then of its grandparent, with
no bound — and carries its own acknowledgement of the cost:

```perl
# Firstly, look at siblings of this page, then at siblings of parent,
# then those of grandparent, etc.
# In a large/complex site, this gets way too much. But how to prune?
while ($xentry = $self->getParentPage($xentry)) {
  foreach my $sib ($self->getChildPages($xentry)) { … addNavigation … } }
```

Measured on the 131 MB witness split at `subsubsection` (40,201 pages): **406
`<link rel=…>` per page**, i.e. **16.3 M relation links**, and at ~55 KB/page
they are the bulk of the 2.25 GB of HTML. The CrossRef phase is **77.9 % of the
whole post run** (1227.9 s of 1576 s attributed; XSLT 17.0 %, MathML-pres 2.3 %)
— ~30.5 ms per page. Engine telemetry (`--telemetry-out`) is how the phase split
was obtained; this box's PMU has no branch-stack sampling, so `perf --call-graph
lbr` cannot profile here.

**This is faithful, and pruning it would be a divergence.** The Rust port
(`latexml_post/src/crossref.rs::fill_in_relations`, the `while let Some(parent) =
self.get_parent_page_id(&xentry)` loop) reproduces the walk exactly, including
the `primary` → element-name relation and the `sidebar` fallback. Only the
*cost per link* is ours to optimize (`child_pages` is already memoized); the
*number* of links is Perl's answer and must stay. A future "optimization" that
bounds the ancestor walk needs a surpass-Perl decision, not a perf argument.

## 70. Default CSS renders adjacent display equations touching (no display skips)

Upstream's default `LaTeXML.css` (v0.8.8 and master, checked 2026-08-01) gives
the display-math containers (`.ltx_eqn_table` L244, `.ltx_eqn_div` L241) no
vertical margin. Text paragraphs are spaced by the UA's `p { margin:1em 0 }`
collapsing through `div.ltx_para`; equation tables have no such margin, so two
displays with no text between them render with a 0px gap — where pdflatex
inserts `\abovedisplayskip`/`\belowdisplayskip` (~1em of the body font).

Minimal trigger (issue #473):

```tex
\documentclass[12pt]{article}
\begin{document}
\[ A = B \]

\[ s(s^2+10s+24) \]
\end{document}
```

Same-host Perl 0.8.8 and latexml-oxide emit the same body markup for this MWE
(identical elements and classes; only whitespace serialization and the
sanctioned OXIDIZED_DESIGN #18 invisible-operator differ) with the same
vanilla CSS — and Perl's own HTML+CSS artifacts, rendered as-is, measure a
**0.0 px** gap between the two displays (headless Chrome,
`getBoundingClientRect`, 2026-08-01). The touching rendering is Perl-origin,
unreported upstream (tracker searched 2026-08-01; nearest are #2438
intra-alignment spacing and #572 display-math paragraph breaking), and the
ar5iv fork hit and fixed this exact rule downstream in its site CSS instead
(`ar5iv-css/css/ar5iv.css` `.ltx_eqn_table { margin: 0.65rem auto }`, a value
calibrated to ar5iv's own paragraph rhythm rather than the UA's 1em). Rust resolves
it with a bundled-CSS local delta (OXIDIZED_DESIGN divergence #92):
`.ltx_eqn_table, .ltx_eqn_div { margin-top:1em; margin-bottom:1em; }`.

## 71. Default CSS destroys verbatim rendering (`white-space:nowrap` on `.ltx_verbatim`)

Upstream's default `LaTeXML.css` (v0.8.8 and master) sets `.ltx_verbatim
{ text-align:left; white-space:nowrap; }`. Author CSS beats the UA
stylesheet, so on a plain `{verbatim}` `<pre class="ltx_verbatim">` the
`nowrap` overrides `pre { white-space:pre }` and the whole block renders as
ONE line (measured 2026-08-02, headless Chrome: a 4-line block renders 15 px
tall). On fancyvrb's per-line spans, `nowrap` collapses leading indentation
and runs of spaces, and the fixed-width inline-blocks flow side-by-side in a
wide window instead of one line per row.

Minimal trigger (issue #431):

```tex
\documentclass{book}
\usepackage{fancyvrb}
\begin{document}
\begin{Verbatim}
TEST 1  ABC

    print(i)
\end{Verbatim}
\begin{verbatim}
PLAIN 1
PLAIN 2
\end{verbatim}
\end{document}
```

Same-host Perl 0.8.8 renders identically (measured on its own HTML+CSS
artifacts); the flagship deployments never see it because they override the
CSS — ar5iv drops the `nowrap` (`ar5iv.css:2949`). Rust resolves it with a
bundled-CSS local delta (OXIDIZED_DESIGN divergence #93). Note Perl's
fancyvrb binding itself is fine (`fancyvrb.sty.ltxml` adds the per-line
`ltx_verbatim` class); the Rust port of that binding had dropped the hack
and now carries it.

## 72. `seealsoPartition_aux` keys its attribute hash on attribute NODES — the styling attributes it means to copy are lost

`Post/MakeIndex.pm` L445, re-wrapping a `\see`/`\seealso` phrase's styling
element around each partitioned sub-chunk:

```perl
my $attr = { map { ($_ => $ch->getAttribute($_)) } $ch->attributes };
push(@result, map { [$$_[0], [$tag, $attr, cdr($_)]] } seealsoPartition_aux($doc, $ch));
```

`$ch->attributes` yields attribute **nodes**, not names. Used as hash keys they
stringify to their serialized form, and `getAttribute($node)` then looks up an
attribute by that same junk string and finds nothing. So `%$attr` is a hash of
unusable keys mapped to `undef` — the `ltx:text`/`ltx:emph` wrapper is rebuilt
with none of the attributes the line is written to preserve (`font`, `class`,
…). The intended read is `$_->nodeName` (or `getQName`), as the sibling
`mergeAttributes` (`Post.pm` L1303-1305) does correctly.

Measured (same-host `XML::LibXML`):

```perl
my $doc = XML::LibXML->load_xml(string => q{<r><t role="x" font="bold" xml:id="i1">hi</t></r>});
my ($ch) = $doc->documentElement->childNodes;
my $attr = { map { ($_ => $ch->getAttribute($_)) } $ch->attributes };
#   [ font="bold"]  => (undef)
#   [ role="x"]     => (undef)
#   [ xml:id="i1"]  => (undef)
```

Low impact — it only degrades styling inside a see/seealso phrase, and only
when that phrase is itself styled. **Rust implements the intent instead**
(`make_index.rs::seealso_partition_aux` copies the real attributes), minus the
id attributes: the wrapper is cloned once per sub-chunk, so copying `xml:id`
would mint duplicate ids — which Perl's bug incidentally also avoids.

## 73. `xkeyval.sty.ltxml`'s "pretend keyval loaded" also suppresses raw `keyval.sty`, so `\KV@do` and friends never exist

`xkeyval.sty.ltxml` L23:

```perl
AssignValue('keyval.sty_loaded' => 1, 'global');    # pretend keyval loaded too.
```

The intent is sound — keyval's plain `\setkeys`/`\define@key` must not clobber
xkeyval's extended ones. The problem is that `keyval.sty_loaded` is the flag
BOTH load paths gate on: `Package.pm:loadLTXML` L2328-2330 (the binding) and
`loadTeXDefinitions` L2363 (the raw file). `keyval.sty.ltxml` gets keyval's
internals from `InputDefinitions('keyval', noltxml => 1)`, i.e. from the raw
`keyval.sty`; after the pretense that read never happens, and nothing else
defines `\KV@do` (keyval.sty L31), `\KV@split`, `\KV@errx` or `\KV@@sp@def`.

Raw packages LaTeXML reads call those internals directly. `fancyvrb.sty`
L112-117:

```tex
\def\FV@UseKeyValues{%
  \ifx\FV@KeyValues\@empty\else
    \def\KV@prefix{KV@FV@}%
    \expandafter\KV@do\FV@KeyValues,\relax,%
```

Trigger (same-host Perl 0.8.8 ⇒ `Error:undefined:\KV@do`, 1 error):

```tex
\documentclass{article}
\usepackage{xkeyval}
\usepackage{fancyvrb}
\DefineVerbatimEnvironment{myBox}{Verbatim}{
}
\begin{document}
\begin{myBox}
text
\end{myBox}
\end{document}
```

The options argument must be non-empty — `{\n  }` tokenizes to one space —
or `\ifx\FV@KeyValues\@empty` short-circuits before `\KV@do` is reached.

Real LaTeX has no such gap: `xkeyval.sty` L39 `\input xkeyval` pulls in the
xkeyval bundle's own `keyval.tex`, whose L52 defines `\KV@do`. Loading xkeyval
genuinely provides keyval there.

**Rust fixes this** by loading keyval for real before xkeyval's own
definitions, exactly as `xkeyval.sty` does — see OXIDIZED_DESIGN #95. Reported
as latexml-oxide issue #500, where Rust hit it on a plain
`standalone`+`fancyvrb` preamble: `standalone_sty.rs` carries real
`standalone.sty` L107's `\RequirePackage{xkeyval}`, which
`standalone.sty.ltxml` omits. Filed upstream as
<https://github.com/brucemiller/LaTeXML/issues/2864>.
## 74. `DimensionToSpaces` sizes a faked space by the font the document ENDS in

`TeX_Glue.pool.ltxml` L43-45:

```perl
sub DimensionToSpaces {
  my ($dimen) = @_;
  my $fs      = LookupValue('font')->getSize;         # 1 em
  my $ems     = $dimen->ptValue / $fs;
```

The width is converted to **ems**, so the font supplying the em decides which
Unicode space glyphs come out. But `DimensionToSpaces` is called from
`DefConstructor` bodies (`\hskip` L66-79, `\kern`, `\lx@intercol`), which run
in the CONSTRUCTION phase — after the entire document is digested. At that
point `LookupValue('font')` is no longer the font the skip occurred in; it is
whatever font the document happens to end in.

Minimal trigger — the same file twice, differing only in a trailing `\small`
that is nowhere near the skip:

```tex
\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
\begin{Verbatim}[fontsize=\small,numbers=left]
alpha
\end{Verbatim}
\end{document}
```

gives `1\x{2003}\x{2009}` for the line-number skip; adding `\small` on the line
before `\end{document}` gives `1\x{2003}\x{2004}` instead. The skip did not
move and its width did not change — only the document's final font did.

The consequence is not merely instability: because the glyph run is an
*approximation of a fixed pt width*, measuring it in a font that will not
render it makes the rendered spacing wrong by the font-size ratio.

**Rust fixes this** by passing the whatsit's own digest-time font — see
OXIDIZED_DESIGN #96, where the defect surfaced as an eager/streaming
byte-identity break.

## 75. A lazy single-`\author`-block with `\\[1em]` breaks scrambles authors, affiliations and emails (Perl; Rust surpasses)

`Base_Utility.pool.ltxml` (the `\lx@add@authors` splitter, `base_utilities.rs`
L870-878) splits an author block on `\\`, `\quad`, `\and`, `,` and guesses
author-vs-affiliation from superscript position. Two of its own comments flag the
limits: *"This is a mess!"* and *"matching `\\` this way fails to catch
`\\[1em]`, so really should Let it"*. The `\\` **control sequence** is the split
token, so a `\\[1em]` leaves its optional-length `[1em]` at the head of the next
segment, where it leaks as literal text; and a comma-list affiliation line
(`Dept. of Foo, University of Pisa, Italy`) is split into phantom `<personname>`
creators.

Minimal trigger (IEEEtran — the class is already bound; this is **not** a missing
binding):

```tex
\documentclass[12pt,onecolumn]{IEEEtran}
\begin{document}
\author{
Alice Smith\textsuperscript{1,2}, Bob Jones\textsuperscript{3}, \\
Carol White\textsuperscript{1} \\[1em]
\textsuperscript{1}Dept. of Foo, University of Pisa, Italy \\
\textsuperscript{2}Naval Centre, La Spezia, Italy \\[1em]
\texttt{alice@unipi.it, bob@unipd.it,}\\ \texttt{carol@unipi.it}
}
\title{T}\maketitle
\end{document}
```

Perl LaTeXML 0.8.8 emits garbled frontmatter: `[1em]` leaks (`Italy[1em]`,
`<personname>[1em]`), the affiliation lines become phantom `Dept. of Foo` /
`University of Pisa` / `Italy` authors, and the shared `\texttt{}` email line
lands inside one affiliation contact. latexml-oxide originally reproduced this
byte-for-byte (upstream parity, not a Rust divergence).

**Rust now surpasses Perl here.** `\lx@add@authors` / `\lx@add@affiliations`
(`base_utilities.rs`) gained `strip_linebreak_options` (consume the `*`/`[len]`
after a `\\` row-break token before splitting — so no `[1em]` leak, and the
following `\textsuperscript` stays at the line front, keeping the comma-bearing
address one affiliation instead of phantom authors) and `line_is_email_list` + a
3-way line kind (a marker-less pure-address line becomes its own `role=email`
contact, shown once, instead of being welded into an affiliation). The witness
now yields clean structured frontmatter (3 authors, affiliations matched by
superscript, emails as `role=email`). Guard:
`06_cluster_frontmatter::frontmatter_ieee_linebreak_optarg`. Witness
arXiv:2605.23553 (`arxiv.org/html/2605.23553v1`).

## 76. An empty `\hypertarget{id}{}` at the head of `\footnotetext` breaks the note (Perl; Rust surpasses)

`hyperref.sty.ltxml`'s `localized_anchor` (L238, the `afterConstruct` of
`\hypertarget`/`\hyperdef`) DFS-walks from the current node and wraps the first
node `ltx:anchor` may contain, with no empty-content short-circuit and no
open-node guard. An **empty** `\hypertarget{id}{}` therefore localizes onto
unrelated surrounding content, and at the head of a floating `ltx:note` (the
"linked footnote" idiom) it wraps and prematurely **closes** the open note —
emptying it and orphaning the footnote text — with `Error:malformed:ltx:anchor` +
`Error:malformed:ltx:note`.

Minimal trigger:

```tex
\documentclass{article}
\usepackage{hyperref}
\begin{document}
\footnotetext[1]{\hypertarget{x}{}Footnote text after a hypertarget anchor.}
\end{document}
```

Perl LaTeXML 0.8.8 emits `<anchor xml:id="x"><note …/></anchor>Footnote text…`
(2 errors); mid-paragraph an empty `\hypertarget` likewise wraps the preceding
run (`<anchor>Before </anchor>after`). Rust **surpasses** (OXIDIZED_DESIGN #104):
two general guards in `localized_anchor` — empty content ⇒ a bare destination
anchor, and never wrap a still-open node — yield
`<note …><anchor xml:id="x"/>Footnote text…</note>` with 0 errors, non-empty
targets unchanged. Issue #526; upstream-fileable against `brucemiller/LaTeXML`.
Guard: `50_structure::hypertarget_empty_anchor_test`. Witness
arXiv:2607.16395v1 (revtex4-2, `\linkedfootnotetext`).

## 77. Default CSS lets display math escape a width-constrained cell (Rust surpasses)

Upstream's default `LaTeXML.css` (v0.8.8 and master) renders display math as
`.ltx_eqn_table { display:table; width:100% }` with 50%-wide center-pad cells,
and never constrains it to a containing box. Inside a `p{}` cell / `\parbox` /
`minipage` (`.ltx_inline-block`) or a table cell (`.ltx_td`), a wide equation's
intrinsic width exceeds the box; since `overflow` is ignored on `display:table`,
nothing clips it, so the equation escapes the cell and scatters across the page.

Minimal trigger (issue #533):

```tex
\documentclass{article}
\usepackage{amsmath}\usepackage{longtable}\usepackage{enumitem}
\begin{document}
\begin{longtable}{|p{1in}|p{2in}|}
A & \begin{enumerate}[leftmargin=.28cm]
\item text \[\begin{aligned} a &= b\\ &= c \end{aligned}\]
\item more \end{enumerate} \\\hline
\end{longtable}
\end{document}
```

Same-host Perl 0.8.8 renders the identical breakage (headless-Chrome
screenshots match pixel-for-pixel modulo the OXIDIZED_DESIGN #103 caption row) —
Perl-origin, unreported upstream. The lualatex PDF keeps the math in its cell.
Rust **surpasses** with a bundled-CSS local delta (OXIDIZED_DESIGN #108):
`.ltx_inline-block .ltx_eqn_table, .ltx_td .ltx_eqn_table { display:block;
overflow-x:auto; max-width:100% }` — the equation becomes a block scroll
container that stays within its cell (scrolling horizontally when too wide),
mirrored in `ar5iv-css`. Normal full-width display math is untouched. Guard:
`latexml_post::xslt::witnessed_css_delta::constrained_equation_overflow_delta_stays_present`.

## 78. A `\text{…}`-only display equation `\[…\]` stacks one word per line (Perl; Rust surpasses)

**Trigger:** `\[\text{The solution is not valid}\]`.

Perl's `LaTeXML.css` applies `white-space:nowrap` to aligned *table* cells
(`.ltx_td`/`.ltx_th` with `.ltx_align_{left,right,center}`) but not to the
equation content cell (`.ltx_eqn_cell`). A single display equation lays its
content in an `ltx_eqn_cell` (no `ltx_td`) between two 50%-width centering pad
cells of a `width:100%` `ltx_eqn_table`. Real math is unwrappable so this is
invisible; but a `\text{}`-only display digests to *wrappable* `ltx_markedasmath`
text, which then collapses to min-content — one word per line. `\begin{align*}`
is unaffected (its content is a real `ltx_td`, already nowrapped). Same-host Perl
0.8.8 reproduces the stacking byte-for-byte. Rust surpasses by extending the
nowrap rule to `.ltx_eqn_cell` ([OXIDIZED_DESIGN #109]). Issue #527;
upstream-fileable against `brucemiller/LaTeXML`. Guard:
`cluster_cli::display_math_text_nowrap::display_math_text_cell_gets_nowrap_css`.
## 79. fancyvrb `frame=single` draws disconnected rules, not a box (Rust surpasses)

LaTeXML (Perl and Rust) raw-loads `fancyvrb.sty` and lets `frame=single` draw the frame with
raw `\vrule`/`\hrule` (`fancyvrb.sty` `@Single` hooks, L869-968). Those become literal
`<ltx:rule>` elements that never reconstruct into an HTML box, so the frame renders as
disconnected horizontal/vertical fragments; the bottom `\FV@SingleFrameSep` box (side vrules,
no text) also surfaces as a stray empty line, and fvextra's `backgroundcolor` (a per-line
`\colorbox`) is not captured.

Minimal trigger (issue #525):

```tex
\documentclass{article}\usepackage{xcolor}\usepackage{fancyvrb}
\begin{document}
\begin{Verbatim}[frame=single, framerule=0.5pt, framesep=6pt]
line 1
line 2
\end{Verbatim}
\end{document}
```

Same-host Perl 0.8.8 renders the identical broken frame (headless-Chrome screenshots match); the
lualatex PDF draws a proper rectangle. Perl-origin, unreported upstream. Rust **surpasses**
(OXIDIZED_DESIGN #111): redefine the `@Single` frame hooks so the frame becomes an
`ltx_framed_rectangle` box (framesep→padding, framerule→border, fvextra background→background),
dropping the raw rules and the stray line. Guard:
`00_tokenize` `tests/tokenize/fancyvrb_frame.{tex,xml}`.

## 80. `pdfcol.sty` undefined → `\pdfcolInitStack` error (Rust surpasses)

Neither Perl nor Rust LaTeXML ships a `pdfcol.sty.ltxml`. `pdfcol` is a pdfTeX colour-stack
manager pulled in transitively by tcolorbox's `breakable` library (`\tcbuselibrary{breakable}`).
With no binding, `\pdfcolInitStack` / `\pdfcolIfStackExists` / `\pdfcolSwitchStack` /
`\pdfcolSetCurrentColor` / `\pdfcolSetCurrent` are undefined, so a breakable coloured tcolorbox
reports `Error:undefined:\pdfcolInitStack` and leaks the args as body text. Minimal trigger:

```latex
\documentclass{article}
\usepackage{pdfcol}
\begin{document}\pdfcolInitStack{main}\end{document}
```

Same-host Perl (TL2025) errors identically (`1 error; 1 undefined macro[\pdfcolInitStack]`).
Issue #531 (reporter nasser1). Perl-origin, unreported upstream. Rust **surpasses**
(OXIDIZED_DESIGN #112): `pdfcol_sty.rs` ports pdfcol.sty's own "disabled" fallback (all commands
no-op, `\pdfcolIfStackExists` takes the false branch) — a PDF colour stack has no HTML output.
Guard: `06_cluster_regressions::cluster_pdfcol_stub_no_undefined`.

## 81. `\sys_if_shell:TF` undefined on a newer texmf expl3.sty (Perl never fires `\everyjob`)

Neither Perl nor Rust LaTeXML fired TeX's `\everyjob` at job start. l3sys defers its *system*
constants — `\c_sys_shell_escape_int`, the `\sys_if_shell:*` conditional families, the
`\c_sys_{minute,…,year}_int` date/time ints — into `\__sys_everyjob:n { … }`
(`expl3-code.tex` L8131-8217), i.e. into `\g__sys_everyjob_tl`, run by `\__kernel_sys_everyjob:`
at job start. With `\everyjob` never fired, those constants stay undefined on the
dump/short-circuit path (where a texmf `expl3.sty` newer than the embedded dump skips
`\input expl3-code.tex`). A `expl3.sty` dated ≥ 2026-03-20 then USES `\sys_if_shell:TF` in its
support-file/shell-escape check → `Error:undefined:\sys_if_shell:TF` on a breakable coloured
`tcolorbox` (issue #531 secondary; reporter's TL2026 dump 2026-01-19 vs texmf 2026-03-20).
Minimal trigger (needs the version skew, reproduced in `texlive-docker:2026` with l3kernel
2026-07-20 over a 2026-01-19 dump):

```latex
\documentclass{article}\usepackage{expl3}
\ExplSyntaxOn \sys_if_shell:TF{}{} \ExplSyntaxOff
\begin{document}x\end{document}
```

Perl-origin (Perl never fires `\everyjob`). Rust **surpasses** (OXIDIZED_DESIGN #113): fire
`\__kernel_sys_everyjob:` at `LoadFormat('latex')` completion, faithfully emulating TeX's
job-start `\everyjob` (tex.web §1030), so the family is defined with live values before the
preamble. Guard: `06_cluster_regressions::cluster_everyjob_defines_l3sys_shell`.

## 82. Empty-symbol siunitx unit renders as the word "nothing" (Rust surpasses)

A siunitx unit declared with an empty symbol — `\DeclareSIUnit{\nothing}{\relax}`
(arXiv/html_feedback#970, paper 2312.06275) — produces a math token with EMPTY content but
`meaning="nothing"`. Perl `MathML.pm` `stylizeContent` falls back to the `meaning` attribute for
empty content, so the presentation MathML is a VISIBLE `<m:mi>nothing</m:mi>` (painted red as a
suspected error) — the literal word "nothing" appears next to every `\SI{…}{\nothing}` number,
where the author meant no unit at all. Same-host Perl is byte-identical (SHARED-FAILURE),
Perl-origin, unreported upstream. Rust **surpasses** (OXIDIZED_DESIGN #114): an empty
`class="ltx_unit"` token renders as an invisible `<m:mphantom>`, never its `meaning`. Guard:
`06_cluster_regressions::cluster_siunitx_empty_unit_renders_invisible`.

## 83. `\verb` inside `\index{…}` yields an empty `<verbatim/>` + `Verbatim argument lost` (Rust surpasses)

`\index` is bound `SanitizedVerbatim` (`latex_constructs.pool.ltxml` L4397
`DefMacro('\index SanitizedVerbatim', \&process_index_phrases)`), which reads the argument as
literal text and then re-tokenizes it — collapsing a `\verb`'s raw catcode-12 body back into
control sequences and leaving `\verb` with no mouth to scan a delimiter from. `\verb` emits an
empty `<verbatim/>` and its body leaks out mis-tokenized (`\delta` → math-italic δ); a `|`
delimiter additionally collides with the makeindex encap separator `process_index_phrases` splits
on, losing everything after the first `|` into a bogus `style=` attribute and raising
`Error:expected:delimiter Verbatim argument lost`. Minimal trigger:

```latex
\documentclass{article}\usepackage{makeidx}\makeindex
\begin{document}
A\index{\verb+\delta+}. B\index{\verb|\delta|}.
\end{document}
```

Measured: pdflatex TL2025 passes the chars through (`.idx` = `\indexentry{\verb|\delta|}{1}`, index
typesets `\delta` in typewriter). **Same-host Perl LaTeXML 0.8.8 is byte-identical** to Rust
(SHARED-FAILURE; Perl differs only by a `key=""` on the empty phrase) — Perl-origin, split out of
issue #347 into #354. Rust **surpasses** (OXIDIZED_DESIGN #119): `process_index_phrases` consumes
a `\verb<D>body<D>` run atomically before the `!`/`@`/`|` split can see the delimiter, and emits
`\@internal@text@verb`, so the body renders as `<verbatim font="typewriter">`. Guard:
`06_cluster_regressions::cluster_verb_in_index_renders_typewriter`.
Sibling (manyind/mindsample, 2026-09-09): the same re-tokenize uses style catcodes (`@` a
letter), so `\index{\AB@\relax…}` is one undefined `\AB@` and an undefined sort-key word
`\A` is digested — Perl 15 errors on the manual. Rust re-reads what survives the `\protected@write` expansion with the document's table
and keeps undefined words inert / literal in the key (OXIDIZED_DESIGN_DIVERGENCES #222).

## 84. `\ref` to a `\label` on a `\nonumber` eqnarray row renders the document title (Rust surpasses)

A `\label` placed right after `\begin{eqnarray}` whose first row is `\nonumber`:

```latex
\begin{eqnarray}\label{eqx}
&& a = b \nonumber\\
&& c = d
\end{eqnarray}
\ref{eqx}   % pdflatex: "1"
```

pdflatex steps the `equation` counter once at `\begin{eqnarray}`, so `\@currentlabel` is `1`
before the `\nonumber` row suppresses its display; the `.aux` records `\newlabel{eqx}{{1}{1}…}`
and `\ref` yields **1**. LaTeXML instead binds the label to the unnumbered first row
(`<ltx:equation xml:id="S0.Ex1">`, no refnum) while the number lands on a later row
(`S0.E1`); CrossRef's `generateRef`, finding no refnum, walks parents and falls back to
`show="title"`, which reaches the document element and returns the **paper title** as the visible
link text. Same-host Perl 0.8.8 is byte-identical (`title="…paper title…"`, SHARED-FAILURE),
Perl-origin. Reported as arXiv/html_feedback#94 (witness 2308.06222, an equation ref that renders
the whole title "High-temperature superconductivity induced by the Su-Schrieffer-Heeger…"). Rust
**surpasses** (OXIDIZED_DESIGN #120): a labelled equation row with no refnum inherits its group's
number from a numbered sibling during Scan, so `\ref` renders "1" identically to a normal numbered
equation. Guard: `06_cluster_regressions::cluster_eqnarray_nonumber_label_ref_is_the_number`.

## 85. `\usepackage{jcappub}` truncates the author list to the last author (Rust surpasses)

jcappub is JCAP's SISSA/IOP publication class — the JCAP sibling of jheppub (JHEP) — with the
same accumulating `\author[⟨affil⟩]{⟨name⟩}` + `\affiliation[N]{…}` + `\emailAdd{…}` +
`\keywords`/`\acknowledgments` frontmatter. Perl LaTeXML ships `jheppub.sty.ltxml` but **no**
jcappub binding, so `\usepackage{jcappub}` reports `missing file[jcappub.sty]`, `\author` falls
through to article's kernel `\author` (which *overwrites* on each call), and only the LAST
`\author` survives; `\affiliation`/`\emailAdd`/`\keywords` are undefined. Same-host Perl 0.8.8 is
byte-identical (SHARED-FAILURE: 1 author, 4 undefined macros), Perl-origin. Reported as
arXiv/html_feedback#6884 (witness 2404.03569, a 63-author DESI paper collapsing to 1). Rust
**surpasses**: `latexml_package::BINDINGS` routes `jcappub` to `jheppub_sty::load_definitions` (the
sibling class's identical author API), so all authors accumulate — the same "route the sibling
package to its bound binding" move as the biblatex variants (OXIDIZED_DESIGN #117). Guard:
`06_cluster_regressions::cluster_jcappub_accumulates_authors`.

## 86. `\@ifundefined{r@LABEL}` forward-references need LaTeX's multi-pass `.aux` (single-pass LaTeXML cannot)

LaTeXML — Perl and Rust alike — is **single-pass**: `\label{L}` records the label for
post-processing (`labelref` → CrossRef) but never defines the LaTeX `\r@L` macro that documents
read back. In pdflatex `\r@L` exists only after a *previous* run wrote `\newlabel{L}{…}` to the
`.aux`, so a macro gating on `\@ifundefined{r@L}` takes the "undefined" branch on run 1 and
resolves only after 2+ runs; LaTeXML has no `.aux`/`\r@` mechanism at all, so the gate is
**always** undefined. Verified same-host: after `\label{foo}`, `\@ifundefined{r@foo}{U}{D}` prints
`U` on both Perl 0.8.8 and Rust (SHARED-FAILURE, Perl-origin — architectural to LaTeXML's single
pass). Reported as arXiv/html_feedback#6895 (witness 2608.12272): the paper's `datalabmacros.tex`
`\HA`/`\HL` cross-linking scheme —

```tex
\newcommand{\HA@place}[2]{... \phantomsection\label{HA:#1} ...}          % anchor
\newcommand{\HL@to}[2]{\@ifundefined{r@HA:#1}
  {\textcolor{red}{[Error: link ``#1'' has no anchor]}}{\hyperref[HA:#1]{#2}}}  % link
```

renders every `\HL{…}` as the red inline `[Error: link "…" has no anchor]` (the user's "internal
links show as missing") in **both** engines, because `\r@HA:…` is never defined. Minimal trigger:

```tex
\documentclass{article}\begin{document}\label{foo}%
\makeatletter\@ifundefined{r@foo}{UNDEF}{DEF}\makeatother\end{document}
```

→ `UNDEF` in Perl and Rust; pdflatex prints `DEF` only from its 2nd run. Not fixed: LaTeXML
resolves references in post-processing by design, not through `.aux`/`\r@`; emulating the two-pass
`\r@` table would not even rescue this witness — its first `\HL` precedes its `\HA` (a forward
reference, undefined on pdflatex's run 1 too). The rendering half of #6895 (an oversized inline
ORCID icon) is unrelated: correct LaTeXML markup, a downstream ar5iv-css over-reach fixed in the
`ar5iv-css` repo.

## 87. `\centering` in a redefined `\abstractname` leaks as literal text into the abstract heading (Rust surpasses)

`\renewcommand{\abstractname}{\centering {\large Abstract}}` (arXiv/html_feedback#6870, paper
2312.14226, aistats2024) makes the abstract heading render the literal text `\centeringAbstract`.
LaTeXML extracts the heading via `getFrontmatterName` → `DigestText(\lx@abstract@name)`, and
`\lx@abstract@name` is `\format@title@abstract{\abstractname}` with `\format@title@abstract` the
identity hook `#1` (`latex_constructs.pool.ltxml` L1146-1148). `\centering` is a `DefConstructor`
(L1237); digesting it into the text-only `name=` attribute serializes its **reversion** back as
`\centering`. Both engines emit `<ltx:abstract name="\centeringAbstract">` and the XSLT renders
`<h6 class="ltx_title ltx_title_abstract">\centeringAbstract</h6>`. **Same-host Perl LaTeXML 0.8.8
is byte-identical** (core XML and post-processed HTML) — SHARED-FAILURE, Perl-origin (upstream
filing pending, owned by maintainer). Minimal trigger:

```latex
\documentclass{article}
\renewcommand{\abstractname}{\centering {\large Abstract}}
\begin{document}\begin{abstract}Text.\end{abstract}\end{document}
```

Rust **surpasses** (OXIDIZED_DESIGN #121): the `\format@title@abstract` hook neutralizes alignment
declarations during name extraction (`{\let\centering\relax\let\raggedright\relax\let\raggedleft\relax#1}`),
mirroring LaTeXML's own `titlepage` `Let('\centering','\relax')` precedent (L1168), so the name is
the clean label "Abstract". Font-size/series primitives (`\large`, `\bfseries`) never leaked. Guard:
`06_cluster_frontmatter::frontmatter_abstract_centering_name`.

## 89. natbib citations of a numeric `.bbl` render the raw key, not the number (Rust surpasses)

natbib loaded in its default author-year mode, cited against a numeric `.bbl` — plain
`\bibitem{key}` with no `[author(year)]` label, as `\bibliographystyle{unsrt}`/`plain` emit —
renders every `\cite` as the citation *key*, not the number. Real pdflatex/bibtex handle this
via natbib's `\NAT@force@numbers`: a numeric `.bbl` writes `\providecommand\NAT@force@numbers{}`
into the `.aux`, forcing numbers mode *globally* on the next pass, so every citation prints the
bracketed number `[N]` even when `\bibliographystyle{unsrt}` sits AFTER the `\cite`s. Golden
pdflatex `.aux`: `\bibcite{foo}{{1}{}{{}}{{}}}` (number=1, author/year empty) →
`Text citing [1] and also [2] and both [1, 2].`

Single-pass LaTeXML freezes each `\cite`'s author-year `<ltx:bibref show="Authors…">` at digest
time (natbib is not yet in numbers mode), and post-processing `CrossRef.pm::make_bibcite` L542 —
`$show = 'refnum' unless … || $keytag;` — keeps the author-year format because its `|| $keytag`
guard is always satisfied (every `\bibitem` has a key). The numeric `<ltx:bibitem>` has a
`number`/`refnum` but no author/year, so the citation prints `key ()` (Rust) / `key ` (Perl).
Verified same-host on 0.8.8 (SHARED-FAILURE, Perl-origin). Minimal trigger:

```tex
\documentclass{article}\usepackage{natbib}\begin{document}
See \cite{alpha}, \cite{beta}, \cite{alpha,beta}.
\bibliographystyle{unsrt}
\begin{thebibliography}{10}
\bibitem{alpha} A. Author. A paper. Journal, 2020.
\bibitem{beta}  C. Coder.  A paper. Proc, 2021.
\end{thebibliography}\end{document}
```

→ Perl `alpha `, pre-fix Rust `alpha ()`; pdflatex `[1]`. Reported as arXiv/html_feedback#62
(witness 2308.06262, a NeurIPS-2023 paper: 263 `\cite`s all rendered `key ()`). Rust **surpasses**
(OXIDIZED_DESIGN #123): when a frozen author-year bibref resolves to entries that are all
numeric-only, `CrossRef::make_bibcite` (called by `fill_in_bibrefs`) collapses to the bracketed number `[N]`/`[N, M]`, matching
`\NAT@force@numbers`. Guard:
`06_cluster_bibliography::cluster_bib_natbib_late_numeric_style_forces_numbers`.

## 90. Content injected into `\@maketitle` is discarded with the title machinery (Rust surpasses)

LaTeXML replaces the LaTeX kernel's `\maketitle`→`\@maketitle` typesetting pipeline with its own
frontmatter model: `\maketitle` deposits the separately-captured title/author/date and then
`\global\let\@maketitle\relax` (`latex_constructs.pool.ltxml` L1105), the source comment (L1094)
admitting "In case `\@maketitle` defines these — we can't yet emulate that." So content a document
appends to `\@maketitle` via `\g@addto@macro` — a teaser figure, an epigraph — is silently dropped,
and any `\ref` to a `\label` inside it renders the raw internal key `LABEL:fig:teaser`. Real
pdflatex runs `\@maketitle`, so the figure appears below the title and its `\ref` resolves.
Same-host Perl 0.8.8 drops it identically (SHARED-FAILURE, Perl-origin). Minimal trigger:

```latex
\documentclass{article}\usepackage{graphicx}\title{T}\author{A}
\makeatletter
\g@addto@macro\@maketitle{\begin{figure}\includegraphics{x}\caption{C}\label{fig:t}\end{figure}}
\makeatother
\begin{document}\maketitle See \ref{fig:t}.\end{document}
```

→ both engines drop the figure and render `\ref` as "LABEL:fig:t"; pdflatex shows the figure and
"1". Reported as arXiv/html_feedback#4281 (witness 2506.23854, an ICCV paper whose teaser
`\figref{fig:teaser}` rendered "Fig. LABEL:fig:teaser"). Rust **surpasses** (OXIDIZED_DESIGN #124):
`\@maketitle` is predefined empty (clean `\g@addto@macro` append) and `\maketitle` deposits its
accumulated content in a title-neutralized group before relaxing it, so the figure renders and the
reference resolves to "Fig. 1". Guard:
`06_cluster_frontmatter::frontmatter_maketitle_injected_figure_survives`.

Second witness via `titlepic.sty`, which *redefines* `\@maketitle` (rather than
`\g@addto@macro`-appending) to inject a `\@titlepic`-held `\captionof{figure}`+`\label`:
arXiv/html_feedback#6675 (witness 2606.25280, the boids/EvoFlock paper — teaser
`\ref{fig:boid_flock}` rendered "Figure LABEL:fig:boid_flock", the figure dropped, and
the real Figure 2 became Figure 1). Same #124 mechanism recovers it (the redefinition
leaves `\@maketitle` non-empty, so it is deposited); production ar5iv (Perl) still drops
it. Guard: `06_cluster_frontmatter::frontmatter_titlepic_redefined_maketitle_figure_survives`.
## 88. Partially-bold author block renders incoherently (Rust surpasses)

`neurips_2023` (and similar classes) bold the *whole* author block with a block-level `\bf` in
their `\@maketitle` tabular — pure PDF layout LaTeXML does not emulate, since it captures semantic
creators from `\author`. A paper (arXiv 2308.06262, html_feedback#61) that `\textbf`s only its
second author line and relies on that class `\bf` for the first then renders incoherently: the
first line plain, the second bold. **Same-host Perl LaTeXML 0.8.8 is byte-identical** — both emit
`<ltx:personname><ltx:text font="bold">Name</ltx:text></ltx:personname>` on the `\textbf` lines and
a bare `<ltx:personname>Name</ltx:personname>` on the rest (SHARED-FAILURE, Perl-origin, upstream
filing pending, owned by maintainer). Minimal trigger (plain `article`, no neurips needed):

```latex
\documentclass{article}\title{T}
\author{Alpha One \\ \textbf{Beta Two}}
\begin{document}\maketitle\end{document}
```

Rust **surpasses** (OXIDIZED_DESIGN #122): an `ltx:personname` `afterClose` handler unwraps a
personname whose sole meaningful child is a *pure* bold `<text>` (series=bold, otherwise default
upright serif), so all author names render in the same weight; mixed styles (bold-italic, bold-sans)
are left untouched. Guard: `06_cluster_frontmatter::frontmatter_neurips_author_bold_coherent`.

## 91. Multi-line author block with a trailing `\quad\\` loses the second line's first author (Rust surpasses)

A `\author{}` whose first line ends with `\quad \\` — a common NeurIPS/ACL idiom for wrapping a
long author list (`… Zhiyuan Zhu\quad \\ \textbf{Ruiqi Li}\quad …`, arXiv 2507.06670) — has the
`\\` leak to the head of the next `\quad`-split group in the no-marker author arm, so that group's
first `\\`-piece is empty and its real first author is demoted to a bogus affiliation under an empty
`<personname/>`. **Same-host Perl LaTeXML 0.8.8 mangles it identically** (SHARED-FAILURE,
Perl-origin, upstream filing pending). Minimal trigger:

```latex
\documentclass{article}\title{T}
\author{Alice One\quad Bob Two\quad \\ Carol Three\quad Dan Four \\ Some University \\}
\begin{document}\maketitle\end{document}
```

→ Perl/pre-fix Rust: "Carol Three" is an empty `<personname/>` + a "Carol Three" affiliation. Rust
**surpasses** (OXIDIZED_DESIGN #52(d)): empty `\\`-pieces are dropped before choosing the name line,
so the first NON-empty piece is the names. Guard:
`06_cluster_frontmatter::frontmatter_multiline_author_leading_break`.

## 92. `insertBlock` overwrites a single block child's `class` with the wrapper's, not merges (Rust surpasses)

When a box (minipage/parbox) is absorbed onto its content because the content is a single block the
context can hold directly, `insertBlock` (`TeX_Box.pool.ltxml` L489-493) copies the box's attributes
onto the child via `setAttribute` — and for `class` that **overwrites**. LaTeXML has a separate
`addClass` (merges the space-separated set; used elsewhere in the same file at L887/892/896) but
`insertBlock` doesn't use it. So a `lstlisting`/`minted` block that is the sole content of a
`minipage`-in-a-`figure` becomes `<listing class="ltx_minipage">`, losing `ltx_lstlisting` and thus
the whitespace-preserving CSS keyed on it. Verified same-host: Perl 0.8.8 emits the identical
`class="ltx_minipage"` (SHARED-FAILURE, Perl-origin). Minimal trigger:

```tex
\documentclass{article}\usepackage{listings}\begin{document}
\begin{figure}\begin{minipage}{0.3\textwidth}\begin{lstlisting}
a
\end{lstlisting}\end{minipage}\end{figure}\end{document}
```

→ `<listing class="ltx_minipage" …>` in both engines. Rust **surpasses**: `insert_block` `add_class`es
the wrapper's class instead of overwriting, so the child keeps `ltx_lstlisting` and gains
`ltx_minipage`. Full rationale + guard in OXIDIZED_DESIGN #125.

## 93. Numeric-mode natbib `.bbl` mislabels authored+dated References with author-year, not `[N]` (Rust surpasses)

natbib's `\NAT@wrout` (`natbib.sty.ltxml` L609-620) chooses each `\bibitem`'s reference-list label
from `CITE_STYLE`, but its numeric branch is guarded on `$style eq 'number'` (**singular**) — a
value `CITE_STYLE` never holds (`'numbers'`/`'super'`/`'authoryear'`). Number style is therefore
reachable only via the empty-author/year fallback (L612). So in numeric/superscript mode, a
pre-formatted `.bbl` entry (`thebibliography`/`\bibitem`, not the `.bib`/MakeBibliography path)
whose `\bibitem[{Name(Year)}]{key}` label has an author AND a year keeps an author-year label
(`Shor [1994]`) while the inline `\cite` correctly shows `[N]` — a list that disagrees with its own
cites and with pdflatex+bibtex. Verified same-host: Perl 0.8.8 emits the identical `Shor [1994]`
(numbers) / `Shor 1994` (super) (SHARED-FAILURE, Perl-origin). Minimal trigger:

```tex
\documentclass{article}\usepackage[numbers]{natbib}\begin{document}\cite{s}
\begin{thebibliography}{9}\bibitem[{Shor(1994)}]{s} P. Shor.\end{thebibliography}\end{document}
```

→ reference label `Shor [1994]` in Perl and Rust; pdflatex+bibtex (apsrev4-2 / `[numbers]natbib`)
give `[1]`. Reported as arXiv/html_feedback#4295 (witness 2410.05202, 57 entries). Rust
**surpasses**: `\NAT@wrout` forces number style for `'numbers'`/`'super'` too, so the whole list is
`[N]`, matching the PDF — the `authoryear` path is untouched. Full rationale + guard in
OXIDIZED_DESIGN #126.

## 94. IEEEtran multi-row author grid is linearized column-major, scrambling author order (Rust surpasses)

An IEEEtran conference `\author{}` grid — `\and` starts a new COLUMN, a top-level `\\` a
new ROW within a column (arXiv:2403.16405, 6 authors in 3×2) — has each
`\IEEEauthorblockN` emit its creator in token order (down each column), so the sequence
is **column-major** (Zhao, Ding, Chen, Kong, Huang, Zhang) instead of the **row-major
reading order** the PDF / arXiv `citation_author` metadata show (Zhao, Chen, Huang, Ding,
Kong, Zhang). **Same-host Perl LaTeXML 0.8.8 mis-handles the same grid** (SHARED-FAILURE,
Perl-origin, upstream filing pending). Minimal trigger:

```latex
\documentclass[conference]{IEEEtran}
\author{\IEEEauthorblockN{A}\\ \IEEEauthorblockN{B}
\and \IEEEauthorblockN{C}\\ \IEEEauthorblockN{D}}
\begin{document}\maketitle\end{document}
```

→ column-major A, B, C, D; reading order is A, C, B, D. Rust **surpasses**
(OXIDIZED_DESIGN #127): the IEEEtran `\author` dispatch transposes a REGULAR `\and`×`\\`
grid to row-major before emitting creators, guarded so single-row `\and` lists and
`\\` inside `\IEEEauthorblockA` are never reordered. Guard:
`06_cluster_frontmatter::frontmatter_ieee_author_grid_transpose`.
---

## 95. Nested inline-math superscript author markers desync math mode (Rust surpasses)

**Perl source:** `LaTeXML/Engine/Base_Utility.pool.ltxml` L687-740 (`\lx@add@authors`,
`\lx@author@withsup`) + L552 (`\lx@request@frontmatter@annotation`).

**Symptom:**
```
Error:unexpected:\lx@end@inline@math Attempt to end mode math
	current frame is boxing group due to T_BEGIN[{]
```
repeated once per marker, with every author collapsed into one garbled creator.

**Root cause:** the author-marker branch `\let`s `^`/`\textsuperscript` onto
`\lx@request@frontmatter@annotation`, whose `{}` argument reads a single token. A marker
operand that is a control sequence carrying its own group — `^\text{...}`, which real
LaTeX math reads as `^{\text{...}}` — has `\text` severed from its `{...}`; inside the
marker's inline math the orphaned `{...}` leaves a brace-group frame on top, so the
closing `$` ends math against the brace group. Nested `$^\text{$...$}$` markers cascade.

**Minimal trigger:**
```latex
\author{Alice$^\text{$\star$}$ \and Bob$^\text{$\star$}$ \\ $^\text{$\star$}$ Uni}
\begin{document}\maketitle\end{document}
```

**Perl status:** present in 0.8.8 (same-host), errs identically. Upstream filing pending.

**Rust status (FIXED, beneficial divergence — OXIDIZED_DESIGN #129):** the `^`-hijack
wrappers read a FULL superscript operand (`read_frontmatter_sup_operand`,
`base_utilities.rs`), keeping `\text{...}` with its group and any nested `$...$` whole
and undigested; the surrounding math stays balanced. Witness arXiv:2403.11905
(html_feedback#1021): 6 errors → 0. Guard
`06_cluster_frontmatter::frontmatter_nested_math_author_marker`.

## 96. `\hspace`-separated authors bunch, and footnote-symbol marks vanish (Rust surpasses)

**Perl source:** `LaTeXML/Engine/Base_Utility.pool.ltxml` L679-740 (`\lx@add@authors` split
sets `@authorsplits`/`@authoraffilsplits` know only `\and`/`\quad`/`\qquad`/`\\`;
`\lx@author@withsup` `\let`s `^`/`\textsuperscript` onto the affiliation-linker).

**Symptom:** no error is raised — the frontmatter is silently mis-structured. Authors laid
out with `\hspace`/`\hfill` between them collapse into a single `<personname>`, and a
literal equal-contribution superscript (`$^{*}$`) is consumed into an affiliation label that
matches nothing and is discarded (the visible mark disappears).

**Minimal trigger:**
```latex
\author{Alice \hspace{1cm} Bob$^{*}$ \hspace{1cm} Carol}
\begin{document}\maketitle\end{document}
```
Perl 0.8.8 yields one creator `<personname>Alice     Bob     Carol</personname>` — Bob's
`$^{*}$` gone. Witness arXiv:2506.06941 (the arXiv production HTML is byte-identical Perl
0.8.8): six authors welded, "Apple" glued to the last name, Iman Mirzadeh's `$^{*}$` dropped.

**Perl status:** present in 0.8.8 (same-host), same output. Author blocks that avoid `\and`
(using `\hspace`/`\\` layout) and mark equal contribution with a literal `$^{*}$` are a
regular arXiv idiom Perl does not structure. Upstream filing pending.

**Rust status (FIXED, beneficial divergence — OXIDIZED_DESIGN #52(i)):** `\hspace`/`\hfill`
normalize to the `\quad` separator, and footnote-SYMBOL superscripts rewrite to a visible
`\lx@frontmatter@keepsup` sup before branch selection (`normalize_hspace_separators`,
`rewrite_symbol_superscripts`, `base_utilities.rs`). Numeric affiliation marks are untouched.
Guards `06_cluster_frontmatter::{frontmatter_hspace_author_split,
frontmatter_symbol_superscript_mark, frontmatter_thanks_literal_mark_mix}`.

---

## 97. Main-file guess: pdf-`\includegraphics` heuristic runs before the `.bbl` tie-break

**Perl source:** `LaTeXML/Util/Pack.pm` `detect_source` L188-213 and
`heuristic_check_for_pdftex` L222-241.

**Symptom:** For a multi-file arXiv submission whose real top-level file
**delegates its figures** to `\input`-ed section files (so contains no direct
`\includegraphics`), Perl selects a bundled class **template / how-to / supplement**
as the main source whenever that decoy carries an example
`\includegraphics{fig.png}`. The HTML then renders the template ("How to Use the
IEEEtran LaTeX Templates", "Formatting Instructions for ICLR 2025", …) instead of
the paper.

**Root cause:** the multi-candidate tie-break applies the pdf-`\includegraphics`
heuristic (heuristic 2) **before** the matching-`.bbl` heuristic (heuristic 3).
The pdf heuristic narrows the set to the decoy, so the `.bbl` signal — which
uniquely fingerprints the real main (arXiv bundles `<main>.bbl`) — never runs.

**Minimal trigger:** a directory with `main.tex` (`\documentclass … \input{sec1}
… \begin{document}`, no `\includegraphics`) + `main.bbl`, alongside
`template.tex` (`\documentclass … \includegraphics[width=1in]{fig.png}`, no
`.bbl`). Perl → `template.tex`; correct is `main.tex`.

**Impact:** Perl-origin, SHARED with the old Rust port. **Rust status (FIXED,
surpasses — OXIDIZED_DESIGN #132):** the `.bbl` heuristic runs before the pdf
heuristic in `main_tex.rs`; 0 regressions across a 133-paper blast-radius sweep.
Witnesses: html_feedback #1721, #6100, #5867, #5476, #4156, #4067, #2369, #2224.

**Secondary quirk (`heuristic_check_for_pdftex`):** the `$pdfoutput_checks`
counter (init 5, `$pdfoutput_checks-- if $pdfoutput_checks`) clamps at 0, so the
`$pdfoutput_checks >= 0` guard on the `\pdfoutput=1` probe is *always* true — the
intended "first few lines only" cap is a no-op and `\pdfoutput=1` matches on any
line. The Rust port (`has_pdftex_marker`) mirrors this effective behavior.

---

## 98. neurips `{hide}` environment defined unconditionally swallows the body in preprint/final mode (Rust surpasses)

**Perl source:** `LaTeXML/lib/LaTeXML/Package/neurips.sty.ltxml` line 59:
`DefEnvironment('{hide}', '');`

**Symptom:** A `neurips_2019`–`neurips_2025` paper in `[preprint]` (or `[final]`)
mode that defines its own brace-gobbling `\newcommand{\hide}[1]{}` and uses it as
`\hide{ … }` loses **everything after the abstract**:
```
Info:ignore:\hide Ignoring redefinition (\newcommand) of '\hide'
Warning:unexpected:\end{document} Attempt to end document with open groups …
Warning:expected:\endhide body should have ended with '\endhide'
```

**Root cause:** The real `neurips_20XX.cls` only runs `\NewEnviron{hide}{}` in the
**submission** branch (`\if@preprint … \else \if@neuripsfinal … \else <here>`,
neurips_2023.cls L336-390), so in preprint/final mode `\hide` is left undefined
and the author's own `\newcommand{\hide}[1]{}` wins. Perl's `.ltxml` defines
`{hide}` **unconditionally**, so `\hide` is already a CS, the `\newcommand` is
ignored as a redefinition, and `\hide{` opens a runaway environment that consumes
tokens to `\end{document}` looking for `\endhide` — swallowing the whole body.
Same failure family as entry #48 (unconditional `DefEnvironment` shadows a
definition → unclosed group eats the document).

**Minimal trigger:**
```tex
\documentclass{article}
\usepackage[preprint]{neurips_2023}
\newcommand{\hide}[1]{}
\begin{document}\maketitle
\begin{abstract}Abstract.\end{abstract}
\hide{\section{Hidden}Gone.}
\section{Visible}Body must survive.
\end{document}
```

**Impact:** Perl-origin, SHARED with the Rust binding (a faithful port of L59).
**Rust status (FIXED — `neurips_sty.rs`):** the `{hide}` `DefEnvironment` is gated
on submission mode (neither `neurips_preprint` nor `neurips_final` set), matching
the real class; submission-mode `\begin{hide}…\end{hide}` still hides. Guard:
`06_cluster_regressions::neurips_hide_preprint_preserves_body`. Witness:
html_feedback #861 (arXiv:2403.15796v1).

---

## 99. IEEE journal `\textsuperscript`-keyed author/affiliation block scrambles into phantom authors (Rust surpasses)

**Perl source:** the default `\author` name-splitter (`Base_Utility.pool.ltxml`),
which has no notion of a trailing `\textsuperscript{N}`-keyed affiliation list.

**Symptom:** The IEEEtran *journal* front-matter idiom — all authors first, each
tagged `\textsuperscript{N}` (a comma list `\textsuperscript{1,2}` links one author
to several), then the affiliations one per `\\` line each led by `\textsuperscript{N}`,
then a `\texttt{…}` email block, `\\[1em]` spacing between groups — comes out
scrambled: `\\[1em]` leaks as literal `[1em]`, and the affiliation lines
("University of Pisa", "Italy", …) are promoted to **phantom authors** (9 creators
for a 6-author paper). Distinct from entry #94 (the `\IEEEauthorblockN` *conference*
grid).

**Minimal trigger:**
```tex
\documentclass[12pt,onecolumn]{IEEEtran}
\author{
Alice\textsuperscript{1,2}, Bob\textsuperscript{3} \\[1em]

\textsuperscript{1}Univ A \\
\textsuperscript{2}Lab B \\
\textsuperscript{3}Univ C
}
\title{T}\begin{document}\maketitle\end{document}
```
Perl → a single phantom creator named `[1em]`, the real authors Alice/Bob dropped
entirely; on the full witness the affiliation lines themselves surface as extra
creators ("University of Pisa", "Italy") — 9 for a 6-author paper. Correct (Rust):
`Alice` (→ Univ A, Lab B) and `Bob` (→ Univ C).

**Impact:** Perl-origin. **Rust status (FIXED — surpasses, OXIDIZED_DESIGN #52):**
the beyond-Perl author-splitter keys each author to its affiliation(s) by the
superscript number (comma lists attach to several), drops the `\\[1em]` spacing, and
never promotes an affiliation line to a creator. Guard:
`06_cluster_frontmatter::frontmatter_ieeetran_journal_superscript_affil`. Witness:
html_feedback #6880 (arXiv:2605.23553v1). Sibling of #6242 (single-line variant).

---

## 100. IJCAI-derivative `\author{}` with `\affiliations`/`\emails` shreds emails into phantom authors (Rust surpasses)

**Perl source:** the default `\author` splitter (`Base_Utility.pool.ltxml`); Perl's
`ijcai.sty.ltxml` handles the idiom, but only for a document that actually loads
the `ijcai` package.

**Symptom:** The IJCAI author idiom (ijcai97.sty) packs names, `\affiliations` and a
comma-separated `\emails` list into ONE `\author{}`. A paper using a *renamed copy*
of ijcai97.sty — e.g. the `ttm.sty` bundled with arXiv:2401.03955 — never loads the
`ijcai` binding, so the raw package is used and neither engine recognises the
section markers: the comma-joined email list is shredded into phantom author
creators (13 for 7 authors), the `\affiliations` payload is dropped, and
`\affiliations`/`\emails` raise `Error:undefined:`. Same on Perl 0.8.8.

**Minimal trigger:**
```tex
\documentclass{article}
\author{Alice \and Bob \affiliations Some Lab \emails a@x.org, b@x.org}
\title{T}\begin{document}\maketitle\end{document}
```
Perl → `Alice`, `Bob`, and a phantom `b@x.org` creator (`Some Lab` mishandled); on
the full witness it shreds all six emails into creators (13 for 7). Correct (Rust):
`Alice`/`Bob`, `Some Lab` an affiliation, the addresses as emails, no errors.

**Impact:** Perl-origin, SHARED with the Rust default splitter. **Rust status (FIXED —
surpasses, OXIDIZED_DESIGN #52):** `\lx@add@authors` detects an `\affiliations`/
`\emails` marker in the body and delegates to the shared sectioned-author machinery
(`\lx@ijcai@authorsplit`, hoisted from `ijcai_sty` into `base_utilities.rs`) — names /
affiliations / emails split, n-th email to n-th author, markers consumed as
delimiters (no undefined-CS error). Guard:
`06_cluster_frontmatter::frontmatter_ijcai_affiliations_emails`. Witness:
html_feedback #1361 + #1362 (arXiv:2401.03955v5, ttm.sty).

## 101. `arrange_panels_and_breaks` wraps figure/table panels in a schema-invalid `ltx:block` (Rust surpasses)

**Perl source:** `Engine/latex_constructs.pool.ltxml:3322`
(`arrange_panels_and_breaks`): `my $block = $document->wrapNodes('ltx:block', $prev_node, $child)`.

**Symptom:** The per-row panel arranger groups two sibling panels into a single
`ltx:block` when a merge heuristic fires — a zero-width sibling, a >8× size
disparity, or a joint width below `0.03125·float_width` (L3305). When the panels
are `ltx:figure`/`ltx:table` (subfigures) the result is
`<ltx:block><ltx:figure/>…</ltx:block>` — schema-INVALID, since a block cannot
contain a float. Reported upstream by the LaTeXML author
(brucemiller/LaTeXML#2709, `acmart` + `subfigure`). Present in Perl 0.8.8
(identical L3322 code): the `child_width==0` branch fires for **every**
`\subcaptionbox`/`\subfloat` multi-panel figure (their panels report width 0), so
Perl-generated HTML carries the invalid block widely — not only Bruce's `acmart`
case. The Rust port reaches it via the disparity / tiny-sum branches (explicit
small/disparate `{width}` subfigures).

**Minimal trigger:**
```tex
\documentclass{article}\usepackage{graphicx}\usepackage{subcaption}
\begin{document}
\begin{figure}\centering
  \begin{subfigure}{0.9\linewidth}\includegraphics[width=\linewidth]{a}\caption{}\end{subfigure}
  \begin{subfigure}{0.05\linewidth}\includegraphics[width=\linewidth]{b}\caption{}\end{subfigure}
  \caption{}\end{figure}
\end{document}
```
Both engines emit `<block>` wrapping the two subfigure `<figure>` panels.

**Impact:** Perl-origin, SHARED with the Rust `arrange_panels`. **Rust status
(FIXED — surpasses):** the block-merge now asks the MODEL whether `ltx:block` can
validly contain the incoming panel (`model::can_contain_sym`, per merge branch) —
a float cannot, so the panels stay siblings: valid markup and the correct
side-by-side layout. Guard:
`06_cluster_regressions::cluster_panel_merge_never_wraps_a_figure_in_a_block_2709`.
An upstream Perl patch (same model-guard at L3305/3322) is to be filed at
brucemiller/LaTeXML#2709.

## 102. Loading `svg` after `subcaption` breaks subfig's `\subfloat` (Rust surpasses)

**Perl source:** `subfig.sty.ltxml:114` — the RawTeX trailer
`\@ifundefined{c@subfigure}{\newsubfloat{figure}}{}`. `svg.sty.ltxml:19` does
`RequirePackage('subfig')`, so loading `svg` pulls subfig in.

**Symptom:** `\newsubfloat{figure}` defines *both* the `subfigure` counter and the
actual `\lx@subfloat@figure` implementation macro, but subfig guards the whole call
on the **counter** existing. When `subcaption` is loaded first it already defines
`c@subfigure` (`subcaption.sty.ltxml:25`), so subfig skips `\newsubfloat` entirely
and never defines `\lx@subfloat@figure`. `\subfloat` then expands through
`\sf@subfloat` → `\csname lx@subfloat@figure\endcsname` = `\relax`, and its
`[caption]{body}` arguments leak as literal text. Same on Perl 0.8.8.

**Minimal trigger:**
```tex
\documentclass{article}
\usepackage{subcaption}
\usepackage{svg}
\begin{document}
\begin{figure}\subfloat[This is a caption.]{This is a figure.}\end{figure}
\end{document}
```
Perl → `<figure><p>[This is a caption.]This is a figure.</p></figure>` (no panel,
no caption). Correct (Rust): a `<figure>` panel whose `<caption>` carries
`This is a caption.` and whose body is `This is a figure.`.

**Impact:** Perl-origin, in subfig's load-time guard. **Rust status (FIXED —
surpasses):** `subfig_sty.rs` defines `\lx@subfloat@figure`/`\lx@subfloat@table`
**unconditionally** and calls `NewCounter!` directly (idempotent), dropping the
counter guard — so the subfloat macros exist regardless of a pre-existing counter.
Since 57l the svg trigger no longer reaches subfig (svg loads no subfig, #322); the
subcaption-then-subfig path stays guarded by
`06_cluster_regressions::cluster_subfigure_panels_share_a_row_6903` (its fixture loads
both, 6 `ltx_figure_panel`s).

## 103. `\scalerel` is undefined, so a scaled inline icon renders unscaled (Rust surpasses)

**Perl source:** none — the `scalerel` package has **no** `.ltxml` binding, and
`\RequirePackage{scalerel}` loads only the raw `.sty`'s dependencies (calc, graphicx,
etoolbox), not its body, so `\scalerel` is never defined.

**Symptom:** an inline icon built with `\scalerel*{obj}{ref}` (which should scale
`obj` to the height of `ref`) — e.g. the `\orcidicon` macro that packs a tikz
`orcidlogo` picture into `\scalerel*` — raises `Error:undefined:\scalerel` and drops
the object in **unscaled**, so the ORCID logo covers multiple text lines. Same on
Perl 0.8.8 (verified same-host: `Error:undefined:\scalerel`).

**Minimal trigger:**
```tex
\documentclass{article}\usepackage{scalerel}
\begin{document}X\scalerel*{\rule{2cm}{2cm}}{Xg}Y\end{document}
```
Perl → `Error:undefined:\scalerel`, the `2cm` rule unscaled. Correct (Rust): the
object wrapped in an inline-block scaled to text height (a 16×16 px inline glyph for
the ORCID witness), zero errors.

**Impact:** Perl-origin (missing binding), shared with the Rust raw-load. **Rust status
(FIXED — surpasses):** `scalerel_sty.rs` binds `\scalerel`/`\stretchrel`; `\scalerel*`
wraps the object in `.ltx_scalerel`, which `LaTeXML.css` sizes to `1em` with its
`svg`/`img` child at `height:100%; width:auto`, so the object scales to the text
height (aspect preserved). Box-measurement scaling being unavailable, the CSS sizes to
the *text* height rather than an arbitrary `ref` — correct for the dominant
inline-icon use. Guard: `06_cluster_regressions::cluster_scalerel_defined_6895`.
Witness: arXiv/html_feedback#6895 (arXiv:2608.12272). The ar5iv stylesheet carries the
matching `.ltx_scalerel` rule.

## 104. amsart authors declared up front bunch every address/email under the last author (Rust surpasses)

**Perl source:** `ams_support.sty.ltxml` (`\address`/`\email`/`\curraddr` → `\lx@add@address`
etc.) + `Base_Utility.pool.ltxml` `\lx@annotate@frontmatter@now` (L510-530). With no
`label`/`labelseq`/`annotate` option, a contact attaches to the **single preceding**
(most-recent) creator.

**Symptom:** the amsart idiom that declares all authors first, then one `\address`/`\email`
pair each —
```tex
\documentclass{amsart}\begin{document}\title{T}
\author{A}\author{B}\author{C}
\address{A-addr}\email{a@x}\address{B-addr}\email{b@y}\address{C-addr}\email{c@z}
\maketitle\end{document}
```
— makes every `\address`/`\email` attach to the *last* author C, so all three addresses and
emails render in C's column while A and B are bare. Same on Perl 0.8.8 (verified same-host,
byte-identical `<ltx:creator>` output). amsart's own PDF also lists them as one flat block
(no per-author association), so there is no ground-truth pairing — only reading-order intent.

**Impact:** Perl-origin (default single-preceding attachment), shared with Rust. **Rust status
(FIXED — surpasses):** `base_utilities.rs::distribute_upfront_contacts` (a DOM pass beside
`coalesce_empty_creators`) redistributes ONLY a clean `N × m` pile — the other N−1 authors
carry no contact and the last author's `K` contacts split evenly (`K = N·m`) into a
role-periodic sequence — handing group *i* to author *i*. Any irregular pile (heterogeneous
roles, differing per-author counts) or already-interleaved contacts fail the gate and are left
exactly as Perl attached them, so the common interleaved idiom (guard
`tests/structure/amsarticle.tex`) is untouched. Guard:
`06_cluster_frontmatter::frontmatter_amsart_upfront_contact_distribution`. Witness:
arXiv/html_feedback#46 (arXiv:2308.06214v1). Divergence: OXIDIZED_DESIGN #140.

## 105. algorithm2e `\Comment*[r]` statement loses its line number (Rust surpasses)

**Perl source:** `algorithm2e.sty.ltxml` L171 —
`DefMacro('\lx@algo@endline', '\lx@prepend@indentation\the\everypar\lx@algo@@endline')`.
Perl fires `\the\everypar` (which under `linesnumbered` is `\nl`) at **end-of-line**, not
at paragraph start. `enterHorizontal` (Stomach.pm) is a plain mode switch and never fires
`\everypar` the way real TeX's `new_graf` does.

**Symptom:** with `[linesnumbered]`, a statement that carries a trailing right side comment
—
```tex
\usepackage[linesnumbered]{algorithm2e}
...
$a \leftarrow 1$ \Comment*[r]{scaling}
```
— renders the statement **unnumbered** and pushes the comment to the next line. The raw
side-comment path (algorithm2e.sty `\SetKwComment`, the non-`altsidecomment` branch)
resets `everyparnl` to `\relax` before `\lx@algo@endline` runs, so the end-of-line
`\the\everypar` sees `\relax` and emits no number. A KwInOut header, whose `\relax` is set
*before* its content, is correctly unnumbered — the two are indistinguishable at end-of-line
and only separable at content-start. Verified same-host on Perl 0.8.8 (witness arXiv
2602.20153): the JUCAL algorithm's `\Comment*[r]` statement lines are unnumbered.

**Rust:** fixed by firing `\everypar` at content-start (tex.web `new_graf`) — see
`OXIDIZED_DESIGN_DIVERGENCES.md` #148. Statement keeps its number; comment stays on the
statement's line intent (numbering matches the pdflatex golden). To be filed upstream.

## 106. Float body frame (`ruled`/`boxed`) is dropped onto `<ltx:tags>` and never drawn (Rust surpasses)

**Perl source:** `float.sty.ltxml` L82 — `addFloatFrames` picks the body as the first
non-caption child: `grep { getNodeQName !~ /^ltx:(?:toc)?caption$/ } $float->childNodes`.
But a `\refstepcounter`'d float emits `<ltx:tags>` as its **first** child, and `<tags>`
(`LaTeXML-block.rnc:325`, `element tags { tag+ }`) has **no attributes**, so
`setAttribute($tags, framed => …)` is silently schema-rejected. The inner frame is lost.

**Symptom:** a `ruled` float draws only its top rule (the outer `framed="top"`, set on the
float itself, survives); a `boxed` float draws **no frame at all**.
```tex
\usepackage{newfloat}\floatstyle{ruled}\newfloat{algorithm}{thp}{lop}
% or: \usepackage[boxed]{algorithm2e}
```
Verified same-host on Perl 0.8.8: `floatnames.tex` and a `[boxed]` algorithm2e MWE emit only
the outer `framed`, never the inner `framed="topbottom"`/`"rectangle"` that pdflatex draws.
Separately, algorithm2e's binding (`algorithm2e.sty.ltxml` L88-91) wires only the `box` family
to a frame, so the default `[ruled]` family draws no rules in either engine.

**Rust:** fixed by also skipping `<ltx:tags>` when selecting the body, so the inner frame lands
on the real body element — and by extending algorithm2e's `\algocf@style` dispatch to map the
`ruled` family → `ruled`. See `OXIDIZED_DESIGN_DIVERGENCES.md` #149. All framed floats
(algorithm/algorithmicx, newfloat, float.sty, algorithm2e boxed/ruled) now frame their body,
matching the pdflatex golden. To be filed upstream.

## 107. `\fname@<type>` is undefined — float.sty's real caption-name internal missing (Rust surpasses)

**Perl source:** `float.sty.ltxml` L36 reimplements `\floatname` as
`\@namedef{lx@name@#1}{#2}` — LaTeXML's own internal — and never defines real float.sty's
`\fname@<type>` (`float.sty` L34, `\@namedef{fname@#1}`). `\newfloat` likewise defaults only
`\lx@name@<type>` (L46-47).

**Symptom:** a document that references the real float.sty internal directly — e.g. the
widely-copied `breakablealgorithm` recipe —
```tex
\usepackage{algorithm}
\newenvironment{breakablealgorithm}{...
  \renewcommand{\caption}[2][\relax]{\textbf{\fname@algorithm~\thealgorithm} ##2\par ...}}...
```
leaks a raw, undefined `\fname@algorithm`: `<ltx:ERROR ...>\fname@algorithm</ltx:ERROR>`
("Still LaTeX / has not been compiled"). Verified same-host on Perl 0.8.8 (witness arXiv
2408.07803, html_feedback #1998): the algorithm caption errors identically.

**Rust:** fixed by defining `\fname@<type>` alongside `\lx@name@<type>` in `\floatname` and
`\newfloat` (real float.sty's internal name). See `OXIDIZED_DESIGN_DIVERGENCES.md` #150. The
caption compiles to "Algorithm 1 …". Additive; to be filed upstream.

## 108. `algpseudocodex` emits spurious empty `<equation/>` blocks (Rust-only; pruned)

**Symptom:** an algorithm using `algpseudocodex` (raw-loaded — there is no `.ltxml`
binding — under `--includestyles` / ar5iv preload) emits TWO childless, RefStepCounter'd
`<ltx:equation/>` nodes per `\State $math$ \Comment{…}` line. Each renders as a tall
EMPTY display-math block, so a whole algorithm is blown apart by huge vertical gaps
between its lines. Witness arXiv 2511.21969 ("trueTriad", html_feedback).

**Cause:** `algpseudocodex` builds every line with TikZ code-boxes plus
`\savebox{\algpx@boxedStringBox}{$\m@th#2$}` (sty L519) and right-justifies `\Comment`
via `\tabto` (sty L895). Our engine's handling of that box/math machinery opens and
closes an equation with no Math content. **GENUINE-RUST-ONLY:** same-host Perl
(`--includestyles`) emits ZERO empty equations for the same input — Perl's box handling
never creates them.

**Rust:** rather than chase the exact box-digestion divergence, we prune at the schema
layer: `Tag!("ltx:equation", after_close_late => …)` drops any equation left with no
`<ltx:Math>` child (`latex_constructs.rs`). A well-formed equation always carries a
`<Math>` element from construction (only its XMath parse is deferred), so the
presence-test is parse-order-safe; `after_close_late` runs after every other
equation-close handler (e.g. amsmath's `rearrangeLoneAMSAligned`, `amsmath.sty.ltxml:638`)
so it never races one that legitimately fills the Math. Reaches Perl parity (0 empty
equations). Guard: `06_cluster_regressions::cluster_algpseudocodex_no_spurious_empty_equation`.

## 109. algorithm2e `\\`-separated body lines lose indentation under the Vline `|` (Rust surpasses)

**Trigger** (`\For`/`\While`/`\If` body using `\\` instead of `\;` for line breaks;
witness arXiv 2002.09766 Algorithm 1):

```latex
\usepackage[algo2e]{algorithm2e}
\begin{algorithm*}
 \For{i=2,\ldots,L-1}{
  ~~Compute line A\\
  Line B\;\\
  Line C\;\\
 }
\end{algorithm*}
```

The `\For` body lines render **flushed flat after the `|` vertical rule** instead of
indented beneath it: they merge into ONE `<ltx:listingline>` joined by inline
`<ltx:break/>`, with a single leading indentation `<ltx:rule>`, rather than three
separate indented listinglines.

**Cause (shared by both engines).** algorithm2e's `beforeDigest` does
`Let('\\','\lx@algo@par')` (the algorithm line-break) then calls `beforeFloat('algorithm')`
**last**; `beforeFloat` re-lets `\\`→`\lx@newline` (a tabular-in-float guard, Perl #2775,
`latex_constructs.pool.ltxml` L3376 / Rust `latex_constructs.rs` `before_float_ex`). So the
reset **clobbers** the intended `\lx@algo@par` binding, and `\\` inside an algorithm2e
listing degrades to `<break/>`. `\par` (also Let to `\lx@algo@par`) and `\;`
(→`\@endalgocfline`→`\lx@algo@par`) are untouched by `beforeFloat`, so they still break
correctly — only `\\` is broken. Verified byte-identical in Perl LaTeXML (the reimpl
author's own `Let('\\','\lx@algo@par')` shows the break was intended).

**Rust:** re-assert `Let('\\','\lx@algo@par')` **after** `before_float` in the algorithm2e
`before_digest` (`algorithm2e_sty.rs`), so each `\\`-separated body line becomes its own
indented listingline, matching the pdflatex golden. A **surpass** (Perl shares the bug).
Safe: a nested `tabular`/`array` rebinds `\\` locally (`\@tabularcr`), shadowing this.
Guard: `06_cluster_regressions::cluster_algorithm2e_for_body_indentation`.

## 110. `.bbl` preamble opens a phantom empty `(N)` bibliography entry (Rust surpasses)

**Trigger** (an ACM-Reference-Format-style `.bbl`: a macro-definition preamble and a blank
line before the first `\bibitem`; witness arXiv 2605.03143):

```latex
\begin{thebibliography}{2}

\providecommand\bibinfo[2]{#2}

\bibitem{A}\bibinfo{title}{First}.
\bibitem{B}\bibinfo{title}{Second}.
\end{thebibliography}
```

emits a spurious empty first entry `<ltx:bibitem xml:id="bib.bib1">` (a `(1)` refnum, a
whitespace-only `<ltx:bibblock>`, no `key`), shifting the real references to `bib.bib2…`.

**Cause (shared by both engines).** The blank line after `\begin{thebibliography}` is a
`\par`; inside a bibliography that is `\par@in@bibliography`, which — seeing the next token is
`\providecommand`, not `\par`/`\bibitem` — opens a keyless `\lx@bibitem` for the preamble
(`latex_constructs.pool.ltxml` L4049 / Rust `latex_constructs.rs` `\par@in@bibliography`). The
digest-time prune both engines carry (Perl #2409) only inspects the immediately-previous box,
which the preamble whitespace displaces, so the phantom survives. Verified byte-identical in
Perl LaTeXML.

**Rust:** an after-close DOM scrub (`Tag!("ltx:bibitem", after_close_late)`) removes any
bibitem with no non-empty `key` and only whitespace `<ltx:bibblock>`s — the phantom. A real
`\bibitem` always has a key, so citeable references are untouched. A surpass (OXIDIZED_DESIGN
#155). Guard `06_cluster_regressions::cluster_bib_preamble_no_phantom_entry`.

## 80. `\define@cmdkey` code never sees its value; stray `#` reaches the stomach (Rust fixes)

Perl `Core/KeyVal.pm:defineCommand` L124-133 emits the key code's invocation as
`\ltxml@orig@<qname>{#<value>}` — a literal `T_PARAM` before the value, flagged by the
author's own `# $value !?!??! Is it a number 1--9 ???` comment. Every `\define@cmdkey`
use then raises `Error:misdefined:# The token T_PARAM[#] should never reach Stomach!`
and the code body's `#1` expands to `#<value>` junk instead of the value. Real xkeyval
(`xkeyval.tex` command-key definer) runs the code with `#1` = the bare value, and also
`\def`s `\cmd<header><key>` to it.

Minimal trigger:

```tex
\documentclass{article}\usepackage{xkeyval}
\makeatletter\define@cmdkey{fam}{ka}{(A:#1)}\makeatother
\begin{document}\setkeys{fam}{ka=x}\end{document}
```

Same-host Perl 0.8.8 errors identically (`misdefined:#`, code sees `#x`). Perl-origin,
unreported upstream. Rust fixes: `latexml_core/src/keyval.rs:define_command` emits the
bare value (guard `cluster_package_guards::xkeyval_internals`; witness
`doc/latex/xkeymask/xkeymask.tex`, perfect-kernel sweep 12).

## 81. ALIGN_STATE drifts on expl3 brace-tricks; l3doc manuals emit stray-`&` (Rust fixes)

Perl's `$LaTeXML::ALIGN_STATE` retracts braces on every `unread`
(Gullet.pm L343-358) including expansion output that was never scanned, and
`readBalanced` localizes the state to 1000000 with an entry decrement. l3tl's
delimited replace machinery pushes net-unbalanced fragments (`\if_true: {
\else: } \fi:` halves), whose kept `{` gets retracted at pushback but
compensated (not counted) when later consumed as an argument opener — the
ledger lands at -1 and the next cell-top `&` errors `Stray alignment "&"`.
One error per l3doc `{function}` block; every l3doc manual affected.

Minimal trigger (Perl errors, pdflatex clean):

```tex
\documentclass{article}
\ExplSyntaxOn
\tl_new:N \g_my_tl
\cs_new_protected:Npn \my_amp: { & }
\cs_new_protected:Npn \my_row:
  {
    \tl_gset:Nn \g_my_tl { a~b }
    \tl_greplace_all:Nnn \g_my_tl { ~ } { x }
    name \my_amp: e \\
  }
\ExplSyntaxOff
\begin{document}
\begin{tabular}{lr}
\ExplSyntaxOn \my_row: \ExplSyntaxOff
\end{tabular}
\end{document}
```

Rust fix: tex.web align_state protocol (scan-count §342/§357, back_input
retract §325, begin_token_list no-adjust; scan_toks doesn't localize) — see
OXIDIZED_DESIGN #172.

## 82. Locked `\newtheorem` mis-parses class-provided leading optional; `\[` clobbered (Rust fixes)

aomart.cls L676-679 wraps `\newtheorem` to accept-and-discard a leading style
optional (`\newtheorem[{}\it]{thm}{Theorem}[section]`). Both engines lock
`\newtheorem` (pool L2835), so the wrapper is a no-op and the pool signature
grabs `[` as the theorem NAME — defining an environment named `[` whose
csname form clobbers `\[`; every later display math opens a spurious
theorem (aomsample: 89 of 101 errors). pdflatex clean. Rust extends the
signature with a discarded leading `[]` (the class's own semantics).

Minimal trigger: `\documentclass{aomart}` + `\newtheorem[{}\it]{thm}{Theorem}[section]` + `\[ x \]`.

## 83. `\index` phrase splitter ignores brace depth; separators inside groups shred the token stream (Rust fixes)

Perl `process_index_phrases` (latex_constructs.pool.ltxml L4326-4350) splits
on `@`/`!`/`|`/`"` with a flat scan. packdoc.sty L328/L331 writes
`\index{#2@\PDElement{#1}{#2}\csuse{packdoc@#1@IndexRemark}}` — the
in-group `@`s cut through the braces, emitting UNBALANCED braces into the
live stream: one mode error + one orphaned `ltx:indexphrase` per use
(algxpar-doc 162+149 errors; numerica). Real makeindex splits the
out-of-band .idx string where imbalance cannot corrupt the document.
Rust honors brace depth (separators at depth 0 only), and since batch 56cl
math too: `\index{arroba@$@$}` (latex-via-exemplos.tex:1042
`\arrobasymbforindex`) is the key `arroba` with the display `$@$`; Perl's flat
scan splits at the inner `@` and digests the lone `$` silently
(`<indexphrase key="$"/>`), while here the lone `$` opened inline math the
bounded `\@index` box never closed (three errors per entry, a leaked
`<XMath>`). A `$` at brace depth 0 toggles the phrase-material state.

Minimal trigger: `\newcommand{\myInd}[1]{\index{#1@\mbox{#1}\csuse{r@e@m}}}` + `\csdef{r@e@m}{}` + itemize item `\myInd{x}`; `\index{arroba@$@$}`.

## 84. glossaries: `\gls` inside math emits bare XM* under `ltx:glossaryref` (schema-invalid)

Perl's glossaries.sty.ltxml (L26-37) rewires `\@gls@link` through a text-level
`ltx:glossaryref` wrapper that is not math-mode aware: fired inside math
(glosmathtools wraps every symbol entry in `\ensuremath`, sty L59-100), the
content digests in math mode and bare `ltx:XMTok`/`ltx:XMApp` land as
glossaryref children — `glossaryref_model = Inline.model` rejects them. Both
engines insert anyway and the math parser still produces a correct
POSTSUBSCRIPT parse, so the errors are schema-validity noise. Byte-identical
Rust=Perl on the min-repro; on the full glosmathtools manuals Perl dies at
MAX_ERRORS=100 while Rust completes (status 2).

Minimal trigger:
```tex
\documentclass{article}
\usepackage{glossaries}
\newglossaryentry{k}{name={\ensuremath{k}},description={t}}
\newglossaryentry{sub.v}{name={\ensuremath{\mathrm{v}}},description={v}}
\begin{document}
\ensuremath{\glsdisp{k}{\ensuremath{k}_{\gls{sub.v}}}}
\end{document}
```
Fix would be a math-aware `\lx@glossaries@gls@link` (drop the wrapper in math
mode) — a surpass needing its own approval (PLANS P16).

## 85. lstlisting inside tabular cells rejected by `td_model = Inline.model`

Legal LaTeX (lexref.tex L305-320, engtlc, expex-glossonly) puts
`\lstnewenvironment` environments inside tabular cells; the schema's
`td_model` (LaTeXML-tabular.rnc L142) excludes block-level `ltx:listing`, so
both engines report `malformed:ltx:listing` and insert anyway (content
survives in the output). Upstream schema question — admit a small Block
subset into td — tracked, not patched locally (two-load-path rule:
LaTeXML.model would need the same edit).

## 86. `\maketitle` inside box captures scatters frontmatter into `_CaptureBlock_`

`insertFrontMatter` (Base_Utility.pool.ltxml L824/L918) opens
`ltx:title`/`ltx:creator` at the CURRENT insertion point; inside a
`\vbox`/minipage/td capture the schema rejects them (byte-identical Rust=Perl
on `\vbox{\maketitle}`: 4 errors + 1 warning). Witnesses ltx-talk ×2,
milsymb, unifront. Surpass shape (fall back to the document-level
`\lx@frontmatter@fallback` insertion point) needs approval — PLANS P16.


## 111. xcolor `\definecolor[ps]{…}` dropped entirely; later `\color{name}` undefined (Rust fixes)

`xcolor.sty.ltxml` L403-409 `checkNoPostscript` returns before `DefColor`, so a
PostScript-typed color is never registered. Real xcolor.sty L531-533 registers
it with the raw PostScript as its driver spec and the MODEL'S WHITE
(`\XC@clr@<model>@white`, L510-516) as its ordinary value, so every
non-PostScript driver renders it white. Witness: TL doc xcolor/xcolor2
(xcolor2.tex:143 defines `lambda`, :134 uses it under `\multiput` ×2280) —
Perl fatals earlier on figure 3, Rust reached this and produced 101
`Can't find color named 'lambda'` + `Fatal:TooManyErrors`. Rust
(`xcolor_sty.rs` `\XC@definecolor`/`\providecolor`) now keeps the
`Info:ignored` line and registers white; `\colorlet`/`\definecolorset` still
skip like Perl (no model to fall back on). Guard
`perfect_kernel_batch49::xcolor_ps_color_registers_as_model_white`.

```latex
\documentclass{article}
\usepackage{xcolor}
\begin{document}
\definecolor[ps]{lambda}{rgb}{Red Corr Green Corr Blue Corr}
\textcolor{lambda}{hello}
\end{document}
```

## 112. CJK binding omits `\CJK@uniPunct`/`\CJK@punctchar`; raw CJKpunct errors on every curly quote (Rust fixes)

ctex's pdfTeX layer requires CJKpunct (ctex-engine-pdftex.def:122). Raw
CJKpunct.sty:442-450 routes U+2018/2019/201C/201D/2014/2026 through
`\CJKpunct@utfasymbol` → `\CJK@punctchar{\CJK@uniPunct}{0}{"80}{byte}` once
`\punctstyle{quanjiao}` fires at `\begin{document}` (:389, :372). Real CJK
supplies them from CJK.enc:291 and a lazily-input `*.chr`; the CJK.sty.ltxml
binding (ar5iv-bindings) never loads either, so both engines emit 2
`undefined` per document (18 TL ctex manuals; jnuexam/jnuexam has nothing
else). Rust `cjk_sty.rs` defines both, mapping the low byte to the Unicode
punctuation (the reduction of CJKpunct.sty:451-474's `plain` branch). Guard
`perfect_kernel_batch49::ctex_cjkpunct_unicode_punctuation`.

```latex
\documentclass{article}
\usepackage[scheme=plain]{ctex}
\begin{document}
A“B”C—D…E‘F’G
\end{document}
```

## 113. amsgen binding Lets `\new@ifnextchar` to the space-skipping `\@ifnextchar` (Rust fixes)

amsgen.sty:54-62 `\new@ifnextchar` is `\@ifnextchar` WITHOUT the space skip
— that is its whole reason to exist. Perl amsgen.sty.ltxml:42 ("Do we need
to worry about the skip space issues...?") Lets it to `\@ifnextchar`.
bibleref.sty:969 `\bibleverse{book}` uses it to look for an
immediately-following `(chapter:verse)`; with the space skipped, a book
name followed by a space and a parenthesised remark opens
`\@bibleverse(#1:` and the `Until::` scan runs to the end of the document
(en/de-bibleref-german, bibleref-german-preamble.tex:120, 12 misses each).
Rust `amsgen_sty.rs` defines the real macro from amsgen.sty. Guard
`perfect_kernel_batch51::new_ifnextchar_keeps_space`.

```latex
\documentclass{article}
\usepackage{bibleref}
\begin{document}
\bibleverse{Psalms} (singular) and \bibleverse{Psalms}(23:1).
\end{document}
```

## 114. biblatex binding leaves `\verb` rebound after `\printbibliography` (Rust fixes)

ar5iv-bindings biblatex.sty.ltxml:410 rebinds `\verb` to the `.bbl`
reader `\biblatex@verb{} Until:\endverb` around `\InputIfFileExists
{\jobname.bbl}` and never restores it; every `\verb+x+` after the first
`\printbibliography` then scans for `\endverb` to the end of the document
(docsurvey.tex:2876-2898: 7 `\verb+.dtx+` after the bibliographies, ~500
lines of body lost; rub-kunstgeschichte-example). Rust `biblatex_sty.rs`
saves and restores `\verb`/`\endverb` around the `.bbl` read. Guard
`perfect_kernel_batch51::verb_survives_printbibliography`.

```latex
\documentclass{article}
\usepackage{filecontents}
\begin{filecontents}{t.bib}
@book{knuth84, author={Donald Knuth}, title={The TeXbook}, year={1984}, publisher={Addison-Wesley}}
\end{filecontents}
\usepackage[backend=biber]{biblatex}
\addbibresource{t.bib}
\begin{document}
Cite \cite{knuth84}.
\printbibliography
Files: \verb+foo.dtx+ and \verb|bar.ins| here.
\end{document}
```

## 115. `\@currbox` is an empty macro, not a box register; dpfloat's per-box `\csname` store scans to end of document (Rust fixes)

latex.ltx:17443 takes `\@currbox` from `\@freelist` (`\@next\@currbox
\@freelist`), a list of `\newbox` registers (:424/442), so
`\string\@currbox` is `\bx@A`…`\bx@M`. Perl latex_constructs.pool.ltxml:1025
defines `\@currbox` as an EMPTY macro; dpfloat.sty:82-88 keys its
per-float store on `\@namedef{LP:\expandafter\string\@currbox}`, which
then `\string`s the empty expansion, `\@namedef` finds nothing between
`\csname` and `\endcsname`, and the float body plus everything after it is
absorbed by the `\csname` scan (memoir/memman via `\newfloat`, oxref ×4:
1001 errors each). Rust `latex_constructs.rs` declares `\newbox\@currbox`.
Guard `perfect_kernel_batch52::currbox_is_a_box_register`.

```latex
\documentclass{memoir}
\usepackage{dpfloat}
\newfloat[chapter]{tegresult}{loe}{Typeset Example}
\begin{document}
Before float.
\begin{tegresult}
Inside custom float.
\end{tegresult}
SWALLOWED text one. SWALLOWED text two. SWALLOWED text three.
\end{document}
```

## 116. xspace omits the pending-space exception; `\foo[x] and` gets two spaces (Rust fixes)

xspace.sty:49 lists `\@sptoken` — LaTeX's `\let` alias of a catcode-10
space, i.e. a pending SPACE token — among the exceptions, so `\xspace`
followed by a surviving space (after a `]`-delimited argument, or after
amsgen's non-space-skipping `\new@ifnextchar`) inserts nothing. Perl
xspace.sty.ltxml's `@XSPACES` compares the literal CS `\@sptoken`, never a
space token, so it inserts a second space (pdflatex: one). Rust
`xspace_sty.rs` treats a `Catcode::SPACE` next token as an exception.
Guard `perfect_kernel_batch52::xspace_pending_space_token_is_an_exception`;
witness glossaries `\gls{potato} and` (structure/glossary golden).

```latex
\documentclass{article}
\usepackage{xspace}
\def\bazA[#1]{baz#1\xspace}
\begin{document}
D \bazA[x] and E.
\end{document}
```

## 117. `\extractcolorspecs` braces the spec; `\definecolor{x}{\m}{\s}` round-trip fails (Rust fixes)

xcolor.sty:1033-1036 defines the plural `\extractcolorspecs{c}{\m}{\s}`
to store the BARE spec (`0.5,0.25,0`), unlike the singular
`\extractcolorspec{c}{\cmd}` which stores `{rgb}{0.5,0.25,0}`. Perl
xcolor.sty.ltxml:808 braces the plural spec too, so a re-defined color
`\definecolor{dst}{\m}{\s}` (pgf-PeriodicTable's `\pgfPT@set@rgb@fill`,
witness pgfPT.colorSchemes.info) parses `{0.5,0.25,0}` as a component and
fails. Rust `xcolor_sty.rs` `\extractcolorspecs` stores the unbraced spec.
Guard `perfect_kernel_batch52::extractcolorspecs_plural_is_unbraced`.

```latex
\documentclass{article}
\usepackage{xcolor}
\begin{document}
\definecolor{src}{rgb}{0.5,0.25,0}
\extractcolorspecs{src}{\m}{\s}
[\m;\s]
\definecolor{dst}{\m}{\s}
\textcolor{dst}{X}
\end{document}
```

## 118. `\@startsection` string-coerces its level; `\numexpr` levels (every KOMA heading) read as 0 (Rust fixes)

latex.ltx `\@sect` compares the level as a TeX <number>: `\ifnum #2>\c@secnumdepth`.
Perl `latex_constructs.pool.ltxml:555-575` does `$level > CounterValue('secnumdepth')`
on the ToString of the argument, which coerces anything non-literal to 0. The
KOMA classes wrap EVERY level as `{\numexpr #2\relax}` (scrartcl.cls:3421/3425,
`#2` = `\csname <name>numdepth\endcsname`), so under a raw KOMA class Perl numbers
every heading down to `\subparagraph` (level 4/5 never exceeds `secnumdepth`), and
a hand-rolled `\@startsection{x}{\numexpr…}` misbehaves the same way. Rust reads a
non-literal level through a sub-mouth `read_number` (latex_constructs.rs
`\@startsection`). Guard `perfect_kernel_batch53::startsection_level_is_a_tex_number`;
witnesses: every raw-KOMA manual (tudaexercise, tikzlings-doc, contract-example-*).

```latex
\documentclass{article}
\makeatletter
\newcounter{deep}\def\deepnumdepth{4}
\newcommand\deep{\@startsection{deep}{\numexpr\deepnumdepth\relax}{\z@}{1ex}{1ex}{\bfseries}}
\makeatother
\begin{document}
\section{S}
\deep{D} % must be UNNUMBERED (4 > secnumdepth 3); Perl numbers it
\end{document}
```

## 119. `\def` parameter text collapses adjacent space tokens in a delimiter (Rust keeps them)

`TeX_Macro.pool.ltxml` L127 builds a macro's delimited-parameter (`Until:`)
delimiter with `push(@delim, $d) unless $pc == CC_SPACE && $inner_cc == CC_SPACE;
# BUT collapse whitespace!`. tex.web §473-476 (`scan_toks` for a parameter
text) reads with `get_token` and keeps every token; the only space folding is
the tokenizer's `skip_blanks` state, which has already run for file-sourced
text. So the collapse is dead for a `\def` read from a file and wrong for a
parameter text built by expansion. expkv.tex L709-712 `\ekv@set@was@blank`
defines a `#1` delimiter `…\ekv@mark␣␣\ekv@nil…` with TWO real spaces (`{ }`
pushed through `\ekv@strip@key` twice); Perl's one-space delimiter never
matches, the marker dance derails (`\ekv@stop`/`\ekv@nil`/`\ekv@mark`
"undefined", then 100 errors → `too_many_errors`), and in Rust the re-scanned
tail ran to `Fatal:Timeout:TokenLimit`. Triggered by any empty/blank expkv
entry — clrstrip.sty L49-77 `\colorstripSet` → `\ekvset{clrstrip}{}` (witness
tutodoc-en/fr, `examples-showcase-input-stripe` line 15).

Rust keeps every delimiter token (`base_utilities.rs` `parse_def_parameters`);
guards `perfect_kernel_batch53::def_delimiter_keeps_adjacent_spaces`,
`expkv_blank_entry_does_not_leak_markers`. Package-free trigger (pdflatex 0
errors; Perl 1 error):

```latex
\documentclass{article}
\makeatletter
\def\A{}\def\B{}\def\SP{ }
\protected@edef\deltoks{\noexpand\A\SP\SP\noexpand\B}
\expandafter\def\expandafter\x\expandafter#\expandafter1\deltoks{[GOT:#1]OK}
\makeatother
\begin{document}
\expandafter\x\expandafter Q\deltoks  % Perl: Missing argument Until:\A \B
\end{document}
```

## 120. `\addcontentsline` digests its title (hangs on LaTeX's write-only `\protect` idiom)

`latex_constructs.pool.ltxml` L749 `DefConstructor('\addcontentsline{}{}{}', …)`
digests all three arguments and then discards the title (`$title` unused —
only `$inlist` is read). latex.ltx L17351-17363 hands `#3` to
`\protected@write`, where `\protect` is `\@unexpandable@protect`, and the text
is written to the `.toc`, never typeset. That is what makes the self-`\protect`
idiom `\def\appfmt#1{\protect\appfmt{#1}}` safe in real LaTeX
(nlctuserguide.sty L1553 `\@loe@disable@cmds`, used by every Talbot manual's
"list of examples"). Under digestion `\protect` is `\relax`, so the macro
re-expands to itself forever: Perl 0.8.8 hangs (timeout, no output); Rust's
cycle guard turned it into `Fatal:Timeout:Recursion` (`\protect\appfmt{xindy}`,
9-token window) or `Fatal:Timeout:TokenLimit` (`…{makeindex}`, 13 tokens, past
the guard's 10-token window). Witness glossaries-user examples `ex:xdy` /
`ex:mkidx`; masked before batch 53 because the kernel `\numberline{}{}` 2-arg
no-op swallowed `\example@title` — raw tocbasic's 1-arg `\numberline` exposed it.

Rust: the title parameter is `Undigested` (`latex_constructs.rs`); guard
`perfect_kernel_batch53::addcontentsline_title_is_not_digested`. Trigger
(pdflatex 0 errors; Perl hangs):

```latex
\documentclass{article}
\newcommand*{\appfmt}[1]{\texttt{#1}}
\begin{document}
\def\thetitle{uses \appfmt{xindy}}%
\def\appfmt#1{\protect\appfmt{#1}}% \@loe@disable@cmds idiom
\addcontentsline{toc}{section}{\thetitle}%
done\end{document}
```

## 121. `\pagestyle` / `\thispagestyle` are non-expandable primitives (scrlayer's `\expandafter` freeze recurses)

`latex_constructs.pool.ltxml` L997-998 (the "# Ignored" block) uses
`DefPrimitive('\pagestyle{}', undef)`; latex.ltx L18297-18300 defines it as a
plain `\def`. scrlayer.sty L2183-2196 redefines `\pagestyle` with the
triple-`\expandafter` freeze
`\expandafter\expandafter\expandafter\renewcommand … {\expandafter\reserved@a
\pagestyle{#1}…}`, which inlines the OLD body at definition time. A primitive
cannot be inlined, so the literal `\pagestyle{#1}` survives in the new body
and `\AtBeginDocument{\pagestyle{test}}` (scrlayer.sty L2198-2213) recurses:
Perl 0.8.8 hangs; Rust reported `Fatal:Timeout:PushbackLimit` (raw scrlayer)
or `Fatal:Timeout:Recursion` (the 13-line freeze below). Reached by every
document loading raw `scrlayer` / `scrlayer-scrpage` (KOMA header/footer;
witnesses DEMO-TUDaPhD, DEMO-TUDaThesis, neoschool, bfh-ci, arXiv 2110.09330 —
the original "runaway" that motivated the old stub). The same block makes
`\markright`, `\markboth`, `\pagenumbering`, `\leftmark`, `\rightmark`
primitives; no witness freezes those yet.

Rust: `def_macro_noop` (expandable empty macro, page style still ignored) for
`\pagestyle`/`\thispagestyle` in `latex_constructs.rs`; guard
`perfect_kernel_batch53::pagestyle_expandafter_freeze_terminates`. Trigger
(pdflatex 0 errors; Perl hangs):

```latex
\documentclass{article}
\makeatletter
\expandafter\expandafter\expandafter\renewcommand
\expandafter\expandafter\expandafter*%
\expandafter\expandafter\expandafter\pagestyle
\expandafter\expandafter\expandafter[%
\expandafter\expandafter\expandafter1%
\expandafter\expandafter\expandafter]%
\expandafter\expandafter\expandafter{\pagestyle{#1}}%
\makeatother
\begin{document}
\pagestyle{plain}
x\end{document}
```
## 122. xkeyval's `\DeclareOptionX*` handler ignored by non-star `\ProcessOptionsX`

xkeyval.tex L496-502: inside `\ProcessOptionsX` (`\ifXKV@inpox`), an option
that matches no key runs `\XKV@doxs` — the `\DeclareOptionX*` handler — when
one is defined, else `\@unknownoptionerror` (packages only). The star form
only adds the class options to the scan. `xkeyval.sty.ltxml` L355-356 arms
the hook with `if ((defined $star) && …)`, so a package whose `\ProcessOptionsX`
has no star silently drops every undeclared option (Rust additionally warned
"unknown KeyVals key"); the handler never sees `\CurrentOption`. pdflatex:
`E=[[english][foo=bar]] W=[3cm]`.

Rust: `xkeyval_sty.rs` `\ProcessOptionsX@int` arms `hook_missing` whenever
`\XKV@doxs` has a meaning; guard
`perfect_kernel_batch53::processoptionsx_unknown_option_reaches_star_handler`.
Trigger (`mypk.sty` + document):

```latex
\ProvidesPackage{mypk}
\RequirePackage{xkeyval}
\def\my@extra{}
\define@key{mypk.sty}{width}{\def\my@width{#1}}
\DeclareOptionX*{\edef\my@extra{\my@extra[\CurrentOption]}}
\ProcessOptionsX
```

```latex
\documentclass{article}
\usepackage[english,width=3cm,foo=bar]{mypk}
\makeatletter
\begin{document}
E=[\my@extra] W=[\my@width]
\end{document}
```

## 123. Backquote charcode of a detokenized backslash reads as 0 (Rust fixes)

`Gullet.pm` L923-928 (`readNumber`, the `` ` `` arm) does `$s =~ s/^\\//`
on the *string* of the next token, then `ord($s)`. For a control sequence
that yields TeX's single-character charcode (`` `\a `` = 97, `` `\\ `` = 92),
but for a **catcode-12 backslash character** — what `\detokenize{\foo}` or
`\string\foo` puts in the stream — the strip empties the string and
`ord("")` is 0. TeX (tex.web §442) takes the character code of any character
token directly: 92. Every "is this a control sequence?" test written as
`\expandafter\test\detokenize{#1}…` + `\ifnum`#1=92` misfires; witness
bibleref-parse.sty L481-486 `\brp@ifcs`, so `\bibleverse{\name}` with a
`\foreach` variable never expands the variable and every such book name is
"unknown" (bibleref-parse.tex, 70+ errors, 100-cap fatal). The same root
aborts every `\fpeval{\dimen0 > \dimen1}` (right operand a bare register
under `>`/`<`/`=`): l3fp's comparison chain-detect (expl3-code.tex
L17662-17673) routes `\if_case:w` on `` ` \token_to_str:N <register> `` →
arm 0 instead of the default → the `@` sentinel of `\__fp_parse_after:ww`
is never emitted → `Missing argument Until:@` + Fatal EoF (Perl: 102 errors,
`too_many_errors`). Witness swfigure `\fptest`/`\DFscalefactor`.

Rust: `gullet.rs` `read_normal_integer` strips the `\` only when the token's
catcode is CS; guards `perfect_kernel_batch54::backquote_charcode_of_other_backslash`
and `::fpeval_register_right_operand_of_comparison`.
Trigger:

```latex
\documentclass{article}
\begin{document}
\def\name{x}
\def\first#1#2\end{[\number`#1]}
\expandafter\first\detokenize{\name}aa\end
\end{document}
```

Expected `[92]`; Perl `[0]` (with "Missing number" warning).

## 124. `\numexpr` division truncates toward zero for negative quotients (Rust fixes)

`Number.pm` `divideround` computes `int(0.5 + $n/$d)`: correct for positive
quotients, but `int` truncates toward zero, so `\numexpr -1/2` gives 0 and
`-7/2` gives -3. eTeX's `quotient` (etex.ch, the `scan_expr` subprocedures)
works on magnitudes and rounds half **away** from zero: -1, -4
(pdflatex-probed). l3fp's multiplication dispatcher
`\__fp_mul_cases_o:NnNnww` (expl3-code.tex:18724-18760) selects its case by
`(#5 #2 #8) / 2 * 2 + 7` and needs `-1/2 = -1` for the `0 × normal` case; with
the truncating quotient a zero LEFT operand of `*` routes to the
`invalid_operation` arm and every enclosing `+`/`-` expression collapses to
0: `\fp_eval:n { 800 - 0 * 3 }` = 0 (pdflatex: 800). Witness wheelchart
(wheelchart.sty:2423 `\pgf@yy * \pgf@xx - \pgf@yx * \pgf@xy` — the shear
registers are 0pt, the transform determinant becomes 0, its inversion
desyncs l3fp: 1001 errors, cap fatal). Perl fails identically on the
kernel repro.

Rust: `numeric_ops.rs` `divideround` is the etex.ch `quotient`; guard
`perfect_kernel_batch54::numexpr_division_rounds_half_away_from_zero`.
Trigger:

```latex
\documentclass{article}
\begin{document}
[\the\numexpr -1/2\relax][\the\numexpr -7/2\relax]
\ExplSyntaxOn [\fp_eval:n { 800 - 0 * 3 }] \ExplSyntaxOff
\end{document}
```

Expected `[-1][-4] [800]`; Perl `[0][-3] [0]`.

## 125. `\read` past end-of-file emits an IGNORE-catcode `\endlinechar` token (Rust fixes)

`Mouth.pm` L303-307 builds the end-of-file token for `\read` as
`$eolcc == CC_EOL ? T_CS('\par') : Token($eolch, $eolcc)`, so when
`\catcode`\^^M=9` is in force the synthetic final line yields a catcode-9
`^^M` token, which later reaches the Stomach as `misdefined: The token
T_IGNORE[U+000d/CR] should never reach Stomach!`. TeX reads that synthetic
empty line in state N like any other line (tex.web §345-349): an IGNORE
(or SPACE) character is skipped and can never become a token. Witness
liftarm: pgfmanual-en-macros.tex:1745-1748 (`codeexample`) sets
`\catcode`\^^M=9` around `\scantokens{\code@temp}`, inside which
`\liftarmanimate` (liftarm.sty:680-728) drives animate.sty's
`\@anim@buildtmln` (animate.sty:2560-2650), a `\whiledo` `\read` loop that
runs to EOF — 501 errors, cap fatal. Perl: 1 error per animation on the
repro.

Rust: `mouth.rs` `read_token` EOF branch drops SPACE/IGNORE endline chars;
guard `perfect_kernel_batch54::read_at_eof_drops_ignored_endlinechar`.
Trigger (`rdtest.dat` = one line `lineone`):

```latex
\documentclass{article}
\begin{document}
\newread\myr \openin\myr=rdtest.dat
\catcode`\^^M=9\relax
\read\myr to \la \read\myr to \lb
\catcode`\^^M=5\relax \closein\myr
X\lb X
\end{document}
```

Expected `XX` with no error; Perl errors `misdefined` on the `^^M` token.

## 126. A brace read as a backquote charcode is not un-counted for ALIGN_STATE (Rust fixes)

tex.web §442: after `get_token` fetches the character following a backquote
(`` `} ``), TeX undoes the `align_state` step that `get_next` applied to the
brace ("if cur_cmd=right_brace then incr(align_state) else decr"). That is
what makes `\iffalse{\fi\ifnum0=`}\fi` — expl3's `\group_align_safe_begin:`
(expl3-code.tex, used by `\tl_replace_all`, `\tl_if_in`, `\seq` splitting…)
and amsmath — leave `align_state` +1 with no group open. `Gullet.pm` L926
`readNumber`'s backquote arm reads the token and returns its code without
the undo, so the idiom nets 0 and a tab-catcode token inside a delimited
macro definition in a tabular cell (l3tl's search pattern, a rescanned
`_` of catcode 4 from l3doc's `\__codedoc_meta:n`) triggers the outer
alignment's column-end program, which is spliced into the parameter text
(`Until:\lx@column@trimright\hfil\lx@alignment@column@after_` runaway).
Masked in Perl only by #127 (its rescan yields an empty pattern).

Rust: `gullet.rs` `read_normal_integer` backquote arm mirrors §442; guard
`perfect_kernel_batch54::backquote_brace_charcode_keeps_align_state`
(pdflatex-probed). Trigger:

```latex
\documentclass{article}\usepackage{expl3}
\begin{document}
\begin{tabular}{l}\begin{minipage}{3cm}
\ExplSyntaxOn
\tl_set_rescan:Nnn \l_tmpa_tl { \char_set_catcode:nn { `_ } {4} } { _ }
\tl_set:Nn \l_tmpb_tl { a_b }
\tl_replace_all:NVn \l_tmpb_tl \l_tmpa_tl { X }
[\tl_use:N \l_tmpb_tl]
\ExplSyntaxOff
\end{minipage}\end{tabular}
\end{document}
```

Expected `[a_b]` in one cell (pdflatex agrees); Perl (given a working
rescan) splices the column template into `\__tl_replace_wrap:w`'s delimiter.

## 127. `\tl_set_rescan` leaks the rescanned tokens — `\everyeof` is never inserted (Rust fixes)

`eTeX.pool.ltxml` L251-258 defines `\everyeof` as a register whose tokens
"are NOT used anywhere (yet?)", and `\scantokens` (`openMouth(writableTokens)`)
never inserts them at the pseudo-file's end. expl3's rescan protocol
(expl3-code.tex:3758-3790) relies on exactly that: `\everyeof{::}` then
`\__tl_rescan:NNw #1#2#3 ::` captures the whole `\scantokens` output as a
delimited argument, PARAM tokens included. Without the marker the delimited
read runs to the pseudo-file end, `Gullet.pm` L683-685 `readUntil` unreads
the collected tokens on the miss, and a rescanned macro MEANING
(`\cs_meaning:N` → `\long macro:#1#2#3->…`) reaches the Stomach as
`misdefined:#` (substances.sty:452 `\substances_contains_see:NT` — 720
errors on the substances manual; Perl 6 per call).

Rust: `latex_constructs_rust_only.rs` overrides `\__tl_set_rescan:nNN` to
tokenize the string itself under the caller's catcodes and feed the
unchanged `\__tl_rescan:NNw` protocol with the marker appended (the
`\scantokens` side stays unmarked — PLANS P15); guard
`perfect_kernel_batch54::tl_set_rescan_captures_param_tokens`. Trigger:

```latex
\documentclass{article}
\ExplSyntaxOn
\cs_new:Npn \FooEntry #1#2#3 { #1@#3|see{#2} }
\cs_new_protected:Npn \contains_see:N #1
  { \tl_set_rescan:Nnx \l_tmpa_tl {} {\cs_meaning:N #1}
    \tl_if_in:VnT \l_tmpa_tl { |see } { YESSEE } }
\ExplSyntaxOff
\begin{document}
\ExplSyntaxOn \contains_see:N \FooEntry \ExplSyntaxOff
\end{document}
```

Expected `YESSEE`; Perl emits 6× `misdefined:#` and prints the meaning.

## 128. `\@setfontsize` is unguarded inside `\protected@edef` (Rust fixes)

latex.ltx:14103-14107 `\@setfontsize#1#2#3{\@nomath#1 \ifx\protect\@typeset@protect
\let\@currsize#1\fi \fontsize{#2}{#3}\selectfont}` — the `\ifx` makes it
inert while `\protect` is `\@unexpandable@protect`. `latex_constructs.pool.ltxml`
L5622 `DefMacro('\@setfontsize{}{}{}', '\let\@currsize#1')` drops the guard,
so a raw class that routes its size commands through `\@setfontsize`
(tufte-common.def:368-405) re-expands `\@currsize`→`\normalsize`→
`\@setfontsize\normalsize…` without bound once pgf `\protected@edef`s a
`font=\normalsize` label (tikz-network manual, `\Vertex[fontsize=…]`). Perl
runs out of memory on the repro; Rust hit its PushbackLimit.

Rust: `latex_constructs.rs` mirrors the guard (the `\@nomath`/`\fontsize…
\selectfont` halves stay dropped); guard
`perfect_kernel_batch54::setfontsize_is_inert_inside_protected_edef`. Trigger:

```latex
\documentclass{article}
\makeatletter
\renewcommand\normalsize{\@setfontsize\normalsize\@xpt{14}}
\protected@edef\lx@probe{\normalsize}
\makeatother
\begin{document}probe ok\end{document}
```

## 129. `\AtEndPreamble` code runs before the `begindocument/before` hook (Rust fixes)

etoolbox.sty:1743 (2020-10+ formats) makes `\AtEndPreamble` literally
`\AddToHook{begindocument/before}`, so its code is queued IN ORDER with the
other chunks of that hook — in particular after doc.sty:907-910's chunk that
loads hypdoc (→ hyperref) at `\begin{document}`. LaTeXML keeps a private
end-of-preamble list that fires before the L3 hook, so under `ltxdoc`
`\AtEndPreamble{\hypersetup{…}}` (liftarm.tex:39, wheelchart.tex:128) sees
`\hypersetup` undefined in both engines (Perl 1 error, same repro).

Rust: etoolbox_sty.rs routes `\AtEndPreamble` through `\AddToHook`; guard
`perfect_kernel_batch54::etoolbox_atendpreamble_runs_after_earlier_begindocument_before_chunks`.
Trigger: `\documentclass{ltxdoc}\usepackage{etoolbox}\AtEndPreamble{\hypersetup{colorlinks=true}}\begin{document}Hello.\end{document}`.

## 130. A bare-style pgf path drawn inside an `ltx:` box in a picture escapes it (Rust fixes)

`pgfsys-latexml.def.ltxml` L392-398 opens a self-contained `svg:svg`
(`_autoopened`, `_autoclose`) when `\lxSVG@begingroup@` fires while the
current node is an `ltx:` element inside a picture (a `\phantom`/node-label
`svg:foreignObject`). Only the group opener is guarded: a `\draw`/`\fill`
with no dash or color option never passes through `\lxSVG@begingroup` and
reaches `\lxSVG@drawpath@unclipped` (L337-339), which inserts the `svg:path`
directly; the document then relocates it up to the picture's main group,
the phantom's `ltx:text` is left open, and every later close desyncs
(`Closing tag "ltx:text" whose open descendents do not auto-close` …
`svg:g isn't allowed in ltx:block`). pmdraw.sty:56-66 wraps whole drawing
loops in `\phantom{\draw …}` (pmdraw manual: 64 errors; Perl 7 on the repro).

Rust: `pgfsys_latexml_def.rs` `ensure_svg_context` fronts the group opener
and the three path/clip emitters; guard
`perfect_kernel_batch54::pgf_bare_path_inside_phantom_stays_in_its_box`.
Trigger:

```latex
\documentclass{article}\usepackage{tikz}
\begin{document}
\begin{center}\begin{minipage}{0.85\textwidth}\begin{minipage}[c]{0.4\linewidth}
\raisebox{0.5cm}{\begin{tikzpicture}\phantom{\draw (0,0)--(1,1);}\draw (0,0)--(2,0);\end{tikzpicture}}
\end{minipage}\end{minipage}\end{center}
\end{document}
```

## 131. A zero-width `\vrule` strut inside an alignment becomes a cell border (Rust fixes)

TeX_Box.pool.ltxml L811-814 tests the alignment branch first: any
`\vrule` with `h > 3 * w` is `isVerticalRule`, so the standard strut
`\vrule height 12pt width 0pt` (and a real `\strutbox`, if a class ever
sets one) is a column rule — `border="ll"` on the cell below in Perl and,
before OXIDIZED_DESIGN #176, in Rust. Guard:
`perfect_kernel_batch54::zero_width_vrule_is_a_strut_not_a_border`.

```latex
\documentclass{article}
\begin{document}
\halign{\vrule height 12pt width 0pt#&\vrule#&#\cr &&a\cr}
\end{document}
```

## 132. `\AtEndOfPackage` code runs after `@`'s catcode is restored (Rust fixes)

latex.ltx:18856 fires `\<name>.<ext>-h@@k` inside `\@onefilewithoptions`,
before `\@popfilename` (L18801) restores `\catcode`\@`, so hook code sees `@`
as a letter. Perl's `InputDefinitions` (Package.pm L2651) digests the hook
after `loadTeXDefinitions` returns — the raw Mouth's `finish` (Mouth.pm L117)
has already put `@` back to other — so a `.def` inputted from the hook is read
with `@` other: europecv.cls:27 `\AtEndOfPackage{\InputIfFileExists{ecven.def}…}`
splits `\ecv@utf` into `\ecv`+`@utf` and the title row loops to the pushback
limit (europecv manual: Fatal; Rust now 0 errors in 3.9 s). Rust sets `@`
to letter around the hook digest (`binding/content.rs`). Guard:
`perfect_kernel_batch54::at_end_of_package_hook_runs_with_at_letter`.

```latex
% hookcls.cls
\ProvidesClass{hookcls}
\AtEndOfPackage{\InputIfFileExists{hookcls.def}{}{}}
\newcommand\hook@one{ONE}
\LoadClass{article}
% hookcls.def:  \providecommand\hooktwo{[\hook@one]}
% t.tex:  \documentclass{hookcls}\begin{document}\hooktwo\end{document}
```

## 133. biblatex binding defines the `.bbl` commands document-wide, shadowing LaTeX's `\list` (Rust fixes)

ar5iv-bindings biblatex.sty.ltxml:342 `DefMacro('\list{}{}{}', …)` (and
`\name`, `\field`, `\strng`, `\entry`, …) are global, while
biblatex.sty:8995-9024 `\blx@bblstart` `\let`s them to their `\blx@bbl@…`
bodies only while the `.bbl` is read. LaTeX's `\list{label}{setup}` then
takes three arguments in every biblatex document: `\begin{cnltxlist}`
(cnltx-doc.cls:492 `\list{}{\cmltx@list@setup}`) ends with `\endlx@list`
"Attempt to end mode internal_vertical" (cnltx_en manual under `add-bib`).
Rust keeps the bodies under `\biblatex@bbl@<name>` and brackets the `.bbl`
input with `\biblatex@bblstart`/`\biblatex@bblend` (saved meanings
restored). Guard: `perfect_kernel_batch54::biblatex_bbl_commands_do_not_shadow_list`.

```latex
\documentclass{article}\usepackage{biblatex}
\newenvironment{mylist}{\list{}{\leftmargin=0pt}}{\endlist}
\begin{document}\begin{mylist}\item one\end{mylist}\end{document}
```

## 134. `\newcommand` optional defaults keep their `#` characters undoubled (Rust fixes)

latex.ltx stores an optional default through two `\def` bodies —
`\@xargdef` (latex.ltx:1245-1258, TL 2025; `\def\foo{\@protected@testopt\foo\\foo{<default>}}`)
and `\kernel@ifnextchar` (latex.ltx:1759 `\def\reserved@b{#3}`) — each reading
`##` as one parameter character, so `\newcommand{\x}[4][########1]`
hands `\\x` a default of `##1` (pdflatex-probed `\detokenize{#1}` =
`####1`). Perl's `convertLaTeXArgs` (Package.pm) stores the default raw;
etoolbox's `\patchcmd` idiom needs the halving — biditools.sty:769
`\newcommand{\bidi@@patchcmd}[4][########1]` sends raw `#`s to the
stomach (`misdefined:#`) in every biditools-loading manual (crbox, lineno
ulineno, multiple-choice, jwjournal ×2, ghab, ucalgmthesis). Rust halves
twice in `convert_latex_args`. Guard:
`perfect_kernel_batch54::newcommand_default_halves_param_tokens_twice`.

```latex
\documentclass{article}
\newcommand{\foo}[2][########1]{[\detokenize{#1}|#2]}
\begin{document}\foo{A}\end{document}   % pdflatex: [####1|A]
```

## 135. `\secdef` drops the `\@dblarg` (Rust fixes)

latex.ltx:16187 `\def\secdef#1#2{\@ifstar{#2}{\@dblarg{#1}}}`; Perl's
shortcut `DefMacro('\secdef {}{} OptionalMatch:*', sub { $_[3] ? $_[2] :
$_[1] })` (latex_constructs.pool.ltxml:567) hands the unstarred form to `#1`
without doubling the title into `[#1]`, so a raw `\long\def\@book[#1]#2`
reached from memoir.cls:2787 `\secdef\@book\@sbook` scans to EOF for its `[`
(srbook-mem Test/TestLight/SerbianBookMem: `Until:]`). Rust defines the
real macro. Guard: `perfect_kernel_batch54::secdef_doubles_the_title_for_the_unstarred_form`.

## 136. `\numprint` binding is not robust (Rust fixes)

numprint.sty:779 `\DeclareRobustCommand*\numprint[2][\@empty]`; the binding
(numprint.sty.ltxml:37, Rust mirror) is a plain macro, so the one-token
lookahead of `\the\toks255` (tex.web §440-448) pre-expands its
`\ifmmode…\else…\fi` dispatch into the stored token list — calctab.sty:334-335
stashes `\numprint{…}` in `\toks255`: 94 "Extra \or already saw \else for
\ifmmode" in the calctab manual. Rust: `robust => true`. Guard:
`perfect_kernel_batch54::numprint_is_robust_under_a_the_toks_lookahead`.

## 137. Bare `\endflushleft`/`\endflushright` end a list nobody opened (Rust fixes)

Both engines alias `\flushleft`/`\flushright` to the frame-less
`\raggedright`/`\raggedleft` (pool:1257-1258) but leave `\endflushleft` as
latex.ltx's `\endtrivlist`: comment.tex:12-18 `noverb`, bidicode.sty:195
`BDef` (tram-doc) → "Attempt to end mode". Rust: the `\end…` partners are
`\relax`. Guard: `perfect_kernel_batch54::bare_endflushleft_is_a_noop`.

## 138. `\lstinline{…}` stops at the first `}` (Rust fixes)

listings.sty:1968 `\lst@InlineG` reads a balanced group for a `{` delimiter;
listings.sty.ltxml:281 ("does NOT balance groups") stops at the first `}`,
leaking the real closer (coolfn `\mintinline{latex}{\renewcommand{\fnindent}{1.25em}}`,
tikz-shields). Rust tracks brace depth for the `{` delimiter only. Guard:
`perfect_kernel_batch54::lstinline_brace_delimiter_is_balanced`.

## 139. `\centering`/`\raggedright`/`\raggedleft` are constructors, so expl3 V-expansion `\the`s them (Rust fixes)

latex.ltx:16419-16433 defines the three as macros; the bindings (pool:1237-1240,
Rust likewise) are non-expandable constructors, and expl3's
`\__exp_eval_register:N` (expl3-code.tex:2507-2517 `\exp_after:wN\if_meaning:w
\exp_not:N #1 #1`) then takes a `\let\raggedsignature=\centering` (DIN.lco:130)
for a register and applies `\tex_the:D` — scrlttr2.cls:5095 `\closing`'s
`\tl_if_in:nVTF {…} \raggedsignature`: "You can't use \raggedsignature after
\the" (bfh-ci letter, SFSesim, makelabels ×2, scrlttr2copy). Rust: the three
are macros over `\lx@do@…` constructors. Guard:
`perfect_kernel_batch54::centering_is_expandable_for_expl3_v_expansion`.

## 140. `\index` re-tokenizes an UnTeX string that glues a control word to a following non-letter (Rust fixes)

`SanitizedVerbatim` (pool:4376-4394) rebuilds the `\index` argument as
`TokenizeInternal(UnTeX($arg))`; UnTeX inserts a space after a control word
only before a LETTER/digit, so a macro-assembled entry `\index{packages!#1@\texttt{#1}}`
with `#1` = `\TIKZ` (pgfornament usefulcommands.tex:93 `\docpkg`, :102 `\docStyle`)
re-reads as the undefined `\TIKZ@` ("Error:undefined:\TIKZ@", pgfornament
ornaments, tikzrput). Minimal trigger:

```latex
\newcommand*{\TIKZ}{Ti\emph{k}Z}
\newcommand{\docpkg}[1]{\texttt{#1}\index{packages!#1@\texttt{#1}}}
\docpkg{\TIKZ}
```

Rust: the argument is stringified the way `\@wrindex`'s `\write` puts it in
the `.idx` file (`writable_tokens`, tex.web §262 print_cs: a space after every
control word). Guard: `perfect_kernel_batch54::index_control_word_before_at_is_not_glued`.

## 141. `\secdef\@part\@spart` / `\@chapter` workers and the locked `\chapter` (Rust fixes)

latex.ltx defines none of `\@part`/`\@spart`/`\@chapter`/`\@schapter`
(article.cls:281-311, book.cls:439-475 do), the class bindings that replace
those files never did, and `\@sect` is a no-op stub (latex_base). A document
that rebuilds its sectioning the way the classes write it —
source3body.tex:96-123 (`\renewcommand\part{…\secdef\@part\@spart}`,
`\newcounter{chapter}`, `\newcommand\chapter{…\secdef\@chapter\@schapter}`:
l3kernel interface3 + source3; frankenstein lips) errors `undefined:\@part`
and, with the kernel `\chapter` locked, "Ignoring redefinition of \chapter" →
`undefined:\chapter` per chapter (2 → 101 errors, fatal). Minimal trigger:

```latex
\documentclass{article}\makeatletter
\renewcommand\part{\secdef\@part\@spart}
\newcounter{chapter}\newcommand\chapter{\secdef\@chapter\@schapter}
\makeatother\begin{document}\part{P}\chapter{C}\section{S}\end{document}
```

Rust: the four workers and a real `\@sect` route to the `\@startsection`
dispatcher (latex_constructs_rust_only.rs §9), and OXIDIZED_DESIGN #179's
undefine of `\chapter` also unlocks it. Guard:
`perfect_kernel_batch54::secdef_part_and_chapter_workers_exist`.

## 142. `\valign` is an empty macro, leaving its alignment template in the stream (Rust fixes)

TeX_Tables.pool.ltxml:555 `DefMacro('\valign','')` drops the primitive but not
its `{<alignment>}` material, whose template `#` then reaches the stomach:
fancyvrb.sty:566-575 `showtabs` renders each tab through `\FancyVerbTab` =
`\valign{\vfil##\vfil\cr…}` — one "The token # should never reach Stomach"
per displayed `Verbatim` line containing a tab (pygmentex_demo ×3). Trigger:
`\begin{Verbatim}[showtabs,tabsize=1]` with a literal tab in a line. Rust:
`\valign BoxSpecification {}` reads and discards the alignment (tex.web
§768). Guard: `perfect_kernel_batch54::valign_swallows_its_alignment`.

## 143. A second `\begin{document}` re-fires the begin-document hooks (Rust fixes)

`\document` is `\@onlypreamble` and its hooks are `\UseOneTimeHook`s
(latex.ltx:9512/9537), so an inner `\begin{document}` fires nothing;
latex_constructs.pool.ltxml:304-335 re-runs `@at@begin@document` on every
`\begin{document}`. ltnews.tex:236/296 and l3news.tex:109/177
`\renewenvironment{document}{}{}` and `\input` every issue file (each with
its own `\begin{document}`), so csquotes' end-preamble block ran twice and
its hooks — `\undef`ed after the first use (csquotes.sty:2434-2446) — erred
`undefined:\csq@hook@nomultilang`, `\csq@hook@hyperref`. Trigger:

```latex
\usepackage{csquotes}\usepackage{hyperref}
\begin{document}A\begin{document}B\end{document}\end{document}
```

Rust: the hook sequence runs only while `inPreamble` is still set (the first
`\begin{document}`). Guard: `perfect_kernel_batch54::second_begin_document_fires_no_hooks`.

## 144. newfloat's `\DeclareFloatingEnvironment` names the float after its LIST (Rust fixes)

newfloat.sty.ltxml:47-80 defines `\<type>name` from the `listname` option
(default "List of <type>s"), so `\DeclareFloatingEnvironment{floppy}` captions
read "List of floppys 1" (Perl's own t/structure/floatnames.xml golden shows
it). newfloat.sty:87-111: `name=` → `\<type>name`, default the capitalized
type ("Floppy"); `listname=` → `\list<type>name`, default "List of <Type>s";
pdflatex captions "Floppy 1". Rust: the two macros are set separately (and the
trailing `[singular][listname]` optionals, newfloat.sty:117-125, are read);
`50_structure/floatnames.xml` re-blessed to "Floppy 1". Guard:
`perfect_kernel_batch54::declare_caption_type_makes_a_float`.

## 145. `\title`/`\author`/`\date` copy the RAW argument into the frontmatter (Rust fixes)

latex_constructs.pool.ltxml:1066-1069 `\date` = `\def\@date{#1}\lx@add@date[…]{#1}`
(and `\title`/`\author` alike): the stored macro halves `##` once (latex.ltx:17214
`\gdef\@date{#1}`), but an ARGUMENT position never halves, so the frontmatter copy
digests `\def\$##1: ##2 ##3${##2}` with doubled hashes and a literal `#` reaches
the stomach — the RCS-keyword idiom, ulineno.tex:16 (2 errors). Trigger:

```latex
\date{\def\$##1: ##2 ##3${##2}\$Revision: 3.1 $}
```

Rust: the frontmatter copy is `\expandafter\lx@add@date@halved\expandafter{\@date}`
(likewise `\@title`, `\@author`). Guard:
`perfect_kernel_batch54::frontmatter_copies_the_halved_macro`.

## 146. amsopn binding omits `\operatorfont` (Rust fixes)

amsopn.sty:90 `\def\operatorfont{\operator@font}` is the user-level name;
`amsopn.sty.ltxml` defines `\operator@font` only, so glosmathtools.sty:54
`\newcommand*{\sbu}[1]{_{\operatorfont{#1}}}` errors on every use (~54 per
manual, glosmathtools en/fr). Trigger:

```latex
\usepackage{amsmath}
$x_{\operatorfont{i}}$
```

Rust: `DefMacro!("\\operatorfont", "\\operator@font")` in amsopn_sty.rs. Guard:
`perfect_kernel_batch54::amsopn_operatorfont_is_defined`.

## 147. `\ifx` reconstructs a native macro from its `\meaning` text (Rust fixes, OXIDIZED_DESIGN #183)

`Common/Object.pm:80-95` `Equals` matches a CODE-ref expansion against a token
list reading `CODE(0x…)`, so biditools.sty:792 `\bidi@ifscanable` (`\meaning` →
`\scantokens` → `\ifx`) reports the native `\begin` as reconstructable and
biditools' patchcmd clone replaces it with a body ending in the literal text
`CODE(…)`; every later environment loses its `\begingroup` (Perl: "Attempt to end
mode internal_vertical"; crbox-doc, ghab-doc: "Attempt to close a group that
switched to mode horizontal"). Trigger:

```latex
\usepackage{biditools}
\begin{tabular}{ll}a & b\\\end{tabular}
```

Rust: cross-variant equality is `false` (the same hack was removed from
`ExpansionBody::PartialEq`). Guard:
`perfect_kernel_batch54::biditools_env_patch_leaves_begin_end_intact`.

## 148. LGR fontmap slot 0x73 (`s`) is final sigma ς instead of σ

`lgr.fontmap.ltxml:40` maps slot 115 to U+03C2 (ς); lgrenc.def:190-192 makes `s`
the ordinary sigma σ (U+03C3, `\textsigma` = `s\noboundary`) and `c` (slot 99)
the final ς — the LGR font's word-end ligature turns `s` into ς, which no map can
express. Every mid-word sigma in LGR text (`\textsigma`, `s` under
`\fontencoding{LGR}`) came out as ς. Trigger:

```latex
\usepackage[LGR,T1]{fontenc}\usepackage{textalpha}
\textsigma
```

Slot 0x22 (`"`) likewise holds the psili U+1FBD where lgrenc.def:434 declares the
dialytika (¨, U+00A8).

Rust: `lgr_fontmap.rs` slots 0x73 = U+03C3, 0x22 = U+00A8. Guard:
`perfect_kernel_batch54::provide_text_command_dispatches_on_encoding`.

**Extension (batch 56ji).** lgr.fontmap.ltxml maps 12 LGR slots to symbol variants or other letters
where the CB fonts (CB.enc) and lgrenc.def have the plain letter: 85 `U` ϒ → Υ; 106 and 107 `j k`
ϑ ϰ → θ κ; 223 ϔ → Ϋ; 184 ὼ → ώ; and 6, 13, 15, 18, 21, 26, 38. It leaves 7 glyphs unmapped (2–5,
16, 17, 22), decodes the breathings, tonos and ypogegrammeni (60, 62, 39, 124) as quotation marks
rather than the Greek spacing marks ῾ ᾿ ΄ ͺ, and its accent table lists the omicron and upsilon
rows with a stray `|`, so those ligatures never form. Trigger:
`{\fontencoding{LGR}\selectfont U jk l'ogos >En}` gives Perl `ϒ ϑϰ λ´ογος ’Εν`, pdflatex
`Υ θκ λόγος ᾿Εν`. Fixed in Rust (lgr_fontmap.rs). Guard `greek_text::lgr_slots_decode_to_the_font_glyphs`.

## 149. The refnum formatter cannot take an argument-taking `\p@<ctr>` (ctex) (Rust fixes)

`Base_Utility.pool.ltxml:1027-1028` `\lx@@therefnum@@` = `{\normalfont\csname
p@#1\endcsname\csname the#1\endcsname}`. ctex's `\labelformat{section}
{\CTEX@thesection}` (ctex-heading-article.def:747) redefines `\p@section` as
`macro:#1->\CTEX@thesection` and patches every kernel `\p@#1\the#1` site to
`\csname p@#1\expandafter\endcsname\csname the#1\endcsname` (:770-771); LaTeXML's
own site is unpatched, so `\p@section` swallows the next `\csname` and the
orphaned `\endcsname` errors on every heading (caspervector 23, sduthesis 36,
tabular2 28, inkpaper-en 3). Trigger:

```latex
\documentclass[UTF8]{ctexart}
\begin{document}\section{X}\end{document}
```

Rust: the `\expandafter` idiom in `\lx@@therefnum@@` (`\lx@@typerefnum@@` keeps the
bare shape: bracing its refnum leaves an empty group that `\lx@refnum@compose` no
longer sees as empty — a trailing space on unnumbered theorem tags). Guard: `perfect_kernel_batch54::ctex_argument_taking_p_macro_keeps_the_refnum`.

## 150. multicol's spanning text closes an `ltx:p` a block `#2` already closed (Rust fixes)

`multicol.sty.ltxml:22,27` `?#2(<ltx:para><ltx:p>#2</ltx:p><ltx:para>)…`: with
`\begin{multicols}{2}[\section*{Contents}]` the section auto-closes the `ltx:p`
and `ltx:para`, and the template's explicit `</ltx:p>` errors "Attempt to close
</ltx:p>, which isn't open" (thuaslogos-doc-english/-dutch). Trigger:

```latex
\usepackage{multicol}
\begin{multicols}{2}[\section*{Contents}]x\end{multicols}
```

Rust: the explicit close is dropped (the next `<ltx:para>` open closes an inline
`#2`). Guard: `perfect_kernel_batch54::multicols_spanning_section_is_not_double_closed`.

## 151. `\pdfannot` never reads its `rule spec` (Rust fixes)

`pdfTeX.pool.ltxml:156-171` `OpenAnnotSpecification` reads `reserveobjnum` /
`useobjnum n` / `stream [attr …]` and then the general text; the pdfTeX
grammar's `annot type spec → [useobjnum n] [rule spec] general text` puts an
optional `(width|height|depth) dimen …` before the text, and pdfmarginpar.sty:142
`\expandafter\pdfannot\pdfmarginpar@rulespec{…}` emits it whenever a
`width=`/`height=` key is set (pdfmarginpar doc): "Expected opening '{'". Trigger:

```latex
Hi\pdfannot width 4cm height 0.5cm {/Subtype /Text /Contents (x)}
```

Rust: the same `while read_keyword(["width","height","depth"]) read_dimension`
loop as `RuleSpecification`. Guard: `perfect_kernel_batch54::pdfannot_reads_its_rule_spec`.

## 152. A class that owns `\section` as a non-sectioning environment hits the kernel lock (Rust fixes)

`latex_constructs.pool.ltxml:559` defines `\section` `locked => 1`, so
examdesign.cls:323-344 (`\def\section{\stepcounter{section}\setcounter
{question}{1}}`, `\def\endsection{\make@qlist}`, and `\begin{section}…\end
{section}` around every question block, :802-812) is refused; `\@startsection`
then runs on the environment body — "Expected opening '{'", `\lx@tag Attempt
to end mode restricted_horizontal`, an `\endgroup` error per `\end` (examplea:
Perl 67 errors, Rust Fatal at 100). Trigger:

```latex
\documentclass{examdesign}
\begin{document}
\begin{matching}[title={T}]\pair{Elvis}{Spike}\end{matching}
\end{document}
```

Rust: an `examdesign.cls` binding clears `\section:locked` before the raw load
(the `\chapter` precedent, #141). Guard:
`perfect_kernel_batch54::examdesign_owns_section_as_an_environment`.

## 153. robustindex's page-reference hooks scan for makeindex's `.ind` line (Rust fixes)

robustindex.sty:201-216 `\gobblepageref` = `\protect\gobbleindpageref` =
`\wrappageref\@gobble`, and `\def\wrapindpageref#1, \indpageref#2` consumes the
`, \indpageref{N}` makeindex writes into each `.ind` line. LaTeXML's index has
no such line, so `\index{alpha!see also gamma\gobblepageref}`
(robustsample.tex:82; multisample, robustmanual) runs the delimited scan off the
entry — Perl "Missing argument Until:, \indpageref for \wrapindpageref", Rust
`readBalanced ran out of input`. Trigger: that line with `\usepackage{makeidx}
\usepackage{robustindex}\makeindex`. Rust: a `robustindex.sty` binding (raw
load, then `\gobblepageref` → empty and `\wrappageref{}` → empty). Guard:
`perfect_kernel_batch54::robustindex_page_reference_hooks_are_inert`.

## 154. A `{}` argument opened by `\bgroup` is read as the one token `\bgroup` (Rust fixes)

`Gullet.pm:732 readArg` starts a balanced read only on catcode-BEGIN; for
`\mbox\bgroup A … B\egroup` (syntax.sty:158 `\syn@assist`, whose `\egroup` is
even inserted by `\readupto`'s `\aftergroup`; the newcommand manual) the
argument is the lone `\bgroup`, which then opens a boxing frame inside the
constructor's mode frame — "`\mbox` Attempt to end mode restricted_horizontal".
TeX hands `\bgroup` to `\mbox#1` the same way, but the `\hbox{` that `\mbox`
opens is then closed by the `\egroup` (its own `}` closed the `\bgroup`), so the
box runs to the `\egroup`. Trigger:

```latex
\def\OPEN{\mbox\bgroup A}\def\CLOSE{ B\egroup}
X\OPEN\CLOSE Y
```

Rust: a `{}` argument that is exactly one implicit-begin-group token reads its
group by digestion through the `{` primitive (`parameter.rs`). Guard:
`perfect_kernel_batch54::implicit_bgroup_argument_reads_its_group`.

## 155. The counter reset list is not the latex.ltx `\cl@<ctr>` macro (Rust fixes)

`Package.pm:674 NewCounter` keeps the reset list as the State value
`\cl@<ctr>`; latex.ltx:10140-10156 keeps it as a macro `\@elt{child}…` that raw
code expands and rewrites — contract.sty:336 `\edef\cl@Clause{\cl@Clause
\cl@contractClause}` (`\cl@contractClause` undefined, contract-example ×2),
afthesis.cls:44-49 `\@removefromreset` (`\cl@chapter expands into itself`).
Trigger: `\newcounter{a}\newcounter{b}[a]\edef\cl@a{\cl@a\cl@b}`. Rust: the
macro mirrors the value after every mutation. Guard:
`perfect_kernel_batch54::reset_list_is_an_expandable_cl_macro`.

## 156. `\abstract{…}` is taken as a pre-tokenized argument (Rust fixes)

`latex_constructs.pool.ltxml` `\abstract` → `\lx@add@abstract{}`: the group is
read as one argument, so a `\makeatletter` inside it cannot precede the
`\patch@level` that follows (char-list-alphabeta.tex:88-103: `\patch`
undefined). In LaTeX `\abstract` is the environment's begin code and `{…}` a
plain group read incrementally. Rust: the brace stays a group and
`\aftergroup\lx@end@abstract` closes the abstract. Guard:
`perfect_kernel_batch54::braced_abstract_reads_its_body_incrementally`.

## 157. `\index` re-tokenization welds a `\@sanitize`d control symbol (Rust fixes)

`latex_constructs.pool.ltxml:4433-4451 SanitizedVerbatim` re-reads the UnTeX'd
entry with normal catcodes, so amsldoc.cls:84-89's sort key `\*` for
`\cn{\\*}` becomes the live `\*` (amsldoc.cls:213 `\def\*#1`) and eats the
entry (itamsldoc, amsldoc-vi; Perl "Expected a relational token" ×2). After
`\@sanitize` (latex.ltx:1778) no control sequence can form, and makeindex never
typesets the sort key. Rust: the round-trip runs per segment between
catcode-12 backslashes, each emitted OTHER. Guard:
`perfect_kernel_batch54::index_sanitized_backslash_symbol_stays_literal`.

## 158. `\@ifundefined` defines the probed name as `\relax` (Rust fixes)

`Base_Utility.pool.ltxml:23-31` implements `\@ifundefined` with the
`\csname…\endcsname\relax` idiom, which DEFINES the name; modern latex.ltx
(:1729-1737) probes with `\ifcsname` and leaves it undefined. A reentrancy-
guarded file loaded as `\@ifundefined{sentinel}{\input file}{}` then finds its
sentinel `\relax`: polyglossia's gloss-latin.ldf:591 `\@ifundefined
{initiate@active@char}{\input{babelsh.def}}{}` meets babelsh.def:1
`\ifx\initiate@active@char\@undefined\else\bbl@afterfi\endinput\fi` and the
shorthand surface never loads (`\bbl@afterfi`, `\shorthandoff`,
`\bbl@deactivate` undefined; hang, sample; 19 gloss-*.ldf files use the idiom).
Trigger: `\@ifundefined{zz}{}{}\ifx\zz\@undefined U\else POLLUTED\fi`. Rust: no
pollution. Guard: `perfect_kernel_batch54::ifundefined_does_not_define_the_name`.

## 159. `\underline`/`\overline` are not robust (Rust fixes)

`TeX_Math.pool.ltxml:989-991` define them as plain macros whose body is
`\protect\ifmmode…\else…\fi`; under `\protected@write`/`\edef` (`\protect` =
`\@unexpandable@protect`) only the `\ifmmode` head is frozen and the
`\else…\fi` tail expands into a stream with no open conditional ("Didn't
expect `\else`/`\fi`"; bibarts.sty:2231 `\edef\@tempa{\write\@auxout
{…\underline{Publ.}…}}` — ba-short, bibarts). latex.ltx:16369 declares them
robust, so the whole body rides in the frozen `\underline␣` token. Trigger:
`\let\protect\@unexpandable@protect \edef\x{\underline{P}}`. Rust: `protected`
macros. Guard: `perfect_kernel_batch54::underline_is_robust_in_an_edef_write`.

## 160. `\labelformat` is undefined; the `\p@<ctr>` handoff to a typerefnum (Rust fixes)

Two halves. (a) latex.ltx:14978 `\def\labelformat#1{\expandafter\def\csname
p@#1\endcsname##1}` has been a KERNEL macro since 2019-10-01 (varioref only
re-exports it); Perl's kernel lacks it and `varioref.sty.ltxml:32` noops it.
contract.sty:978 probes `\scr@ifundefinedorrelax{labelformat}` and, finding it
missing, installs its pre-2019 fallback `\p@sentence`=`\expandafter\p@@sentence`,
whose one-token grab of `\thesentence`'s expansion (`\arabic`) leaves
`{sentence}` behind and ends the label with `\arabic}`. (b)
`Base_Utility.pool.ltxml:1080-1084` (`\lx@@typerefnum@@`) hands an
argument-taking `\p@<ctr>` the token `\lx@the@@` (`\csname p@#1\endcsname
\lx@the@@{#1}`) instead of the single `\the<ctr>` latex.ltx:14976 gives it
(`\csname p@#1\expandafter\endcsname\csname the#1\endcsname`), so the
`{<ctr>}` argument is orphaned ("You can't use } after \the" ×3 per
`\refstepcounter{sentence}`; contract-example-en/-de 44 errors). Trigger:
`\labelformat{equation}{[E:#1]} \begin{equation}\label{e}x\end{equation}`
(Perl: `\labelformat` undefined) and `\newtheorem{thm}{Theorem}
\labelformat{thm}{[T:#1]} \begin{thm}\label{t}x\end{thm}` (Perl: `\the}`).
Rust: the kernel macro is defined in `latex_constructs.rs` next to
`\refstepcounter`, the varioref binding leaves it to the kernel, and both
formatters share `\lx@p@the@@{type}` = `\p@<ctr>\the<ctr>` in latex.ltx's shape.
Guard: `perfect_kernel_batch54::labelformat_is_a_kernel_macro`.

## 161. `\ProcessOptions` reads the loader's option list, not `\opt@<file>` (Rust fixes)

latex.ltx:18557 `\ProcessOptions` reads `\@ptionlist{\@currname.\@currext}`
= the MACRO `\opt@<pkg>.<ext>` (:18393), which a package may REWRITE before
processing: babel.sty:316-347 strips its `language.modifier` syntax
(`greek.polutoniko` → `greek`, `\bbl@mod@greek`=polutoniko). `Package.pm`
`ProcessOptions` reads the State list the loader stored, so the rewrite is
invisible and the raw option dispatch errors "Unknown option
'greek.polutoniko'" (alphabeta-doc, hyperref-with-greek). Trigger:
`\usepackage[greek.polutoniko,english]{babel}` under raw loading. Rust:
`process_options` expands `\opt@<name>.<ext>` and splits it at depth-0
commas, falling back to the State list only when the macro is undefined; the
loader's `\opt@` builder joins BOTH stored shapes (`String` and the `Strings`
a `--preload='[a,b]pkg'` stores — skipping the latter left the macro EMPTY and
silently turned `rawstyles` off). Guard:
`perfect_kernel_batch54::processoptions_reads_the_rewritten_opt_macro`.

## 162. `\pdfoutline`/`\pdfdest` are undefined (Rust fixes)

`pdfTeX.pool.ltxml:179-180` only comments them. tools-overview.tex:93
`\pdfoutline attr {/C[0 0 1]} user {<<…>>} {[#1]}` errors "undefined" and,
worse, the spec words `attr`/`user` land in the text. Trigger: the line
above in any document. Rust: `OutlineSpecification` (`[attr <text>] <action
spec> [count N] <text>`, pdfTeX manual §8.13) and `DestSpecification`
(`num N | name <text>` + `xyz [zoom N] | fitr <rule spec> | fit…`, §8.14)
readers consume the specs; `read_action_spec` (`user`/`goto`/`thread` with
`file`/`num`/`name`/`page`/`newwindow`) is shared. Guard:
`perfect_kernel_batch54::pdfoutline_and_pdfdest_consume_their_specs`.

## 163. `\parse@UTFviii@a`/`@b` are undefined (Rust fixes)

utf8.def:253-265's octet arithmetic is a KERNEL internal (latex.ltx:22224
inputs utf8.def at format time); `utf8.def.ltxml` reimplements
`\DeclareUnicodeCharacter` natively and omits it. paresse-utf8.sty:203-204
`\global\let\GA@parse@UTFviii@a=\parse@UTFviii@a` then calls it while
building its own UTF-8 sequences (paresse-eng 3, paresse-fra 6 errors).
Trigger: `\makeatletter\count@=233 \parse@UTFviii@a;`. Rust: the two macros
are `RawTeX` verbatim in `latex_constructs.rs` beside
`\DeclareUnicodeCharacter`. Guard:
`perfect_kernel_batch54::utf8_octet_parsers_are_defined`.

## 164. `\autopageref` is undefined (Rust fixes)

`hyperref.sty.ltxml` defines `\autoref` (:367) but not hyperref.sty:8183's
`\autopageref{label}` = `\hyperref[{label}]{\HyRef@autopagerefname
\pageref*{label}}` (abntex2cite.tex:1367). Trigger: `\autopageref{s}`.
Rust: `\HyRef@autopagerefname` (`\pageautorefname`/`\pagename` +
`\nobreakspace`, :8196-8203) and the starred/unstarred macro over the
kernel's `\pageref`. Guard: `perfect_kernel_batch54::autopageref_is_a_page_reference`.

## 165. `\verb` on the `\end{verbatim}` line leaks verbatim catcodes; tabs in verbatim (Rust fixes)

`latex_constructs.pool.ltxml:1777` pre-tokenizes the remainder of the
`\end{verbatim}` line (`unread(Tokenize($remaining))`). latex.ltx:15438
`\@xverbatim` is delimited by the catcode-12 string and `\end` runs
`\endgroup` before the rest of the line is tokenized (tex.web §332), so a
`\verb` there scans raw characters with restored catcodes; on frozen tokens
`\verb`'s activated delimiter never matches and the scan re-tokenizes the
rest of the DOCUMENT under `\dospecials` (ddphonism.tex:87 `\end{verbatim}
produces the same as \verb|\dmatrix{…}|.` — every later `{`/`}`/`\` literal,
three lists never closed, 9 errors). Also `:1773` decodes a TAB through
`FontDecodeString(…'OT1_typewriter')` → slot 9 `Ψ`; `\dospecials` never
touches `^^I`, so a tab is a space in verbatim. Trigger:
`\begin{verbatim}\nx\n\end{verbatim} same \verb|z| y.`. Rust: the remainder
is re-read from a lazy raw mouth; tabs become spaces. Guard:
`perfect_kernel_batch54::verb_on_the_endverbatim_line_scans_raw`.

## 166. `\marginnote` is one macro-argument layer short (Rust fixes)

`marginnote.sty.ltxml:37-40` expands `\marginnote` straight to `\marginpar`;
marginnote.sty:319-343 routes the body through `\@dblarg\@mn@marginnote` →
`\@mn@@marginnote` → `\@mn@@@marginnote`. A body calibrated for that depth —
skdoc.cls:631 `\marginnote{\clist_map_inline:Nn…{\index@option*{####1}}}` —
then leaves a literal `#1` (`misdefined:#` ×48) and defines every glossary
entry under the leaked key (`Glossary entry 'index-1-opt' has not been
defined` ×96; iodhbwm). Trigger: the guard's `\DeclareDocumentCommand` +
`\marginnote{\clist_map_inline:Nn…{[####1]}}`. Rust: the real chain, with
`\@mn@@@marginnote` setting the note as `\marginpar`. Guard:
`perfect_kernel_batch54::marginnote_body_rides_three_argument_layers`.

## 167. soul's scanner surface (`\SOUL@setup`, `\SOUL@`) is undefined (Rust fixes)

`soul.sty.ltxml` reimplements the public API only; highlightx.sty:193 and
proofread.sty:74 run `\SOUL@setup` (soul-ori.sty:557-567), redefine the
per-token hooks and hand text to the scanner `\SOUL@` (:131) — "undefined
`\SOUL@setup`" (highlightx-doc, proofread/example). Trigger:
`\makeatletter\SOUL@setup\SOUL@{text}`. Rust: the hooks as the redefinable
macros they are, `\SOUL@setup` resetting them as :557-566, `\SOUL@`
setting its argument as plain text. Guard:
`perfect_kernel_batch54::soul_scanner_surface_is_defined`.

## 168. `\aftergroup` in a tabular cell fires after the column ends (Rust fixes)

LaTeX's entry template is a brace group (latex.ltx `\@classz`
`{\hfil\hskip1sp\ignorespaces\@sharp\unskip\hfil}`), so `\aftergroup` in a
cell fires at the entry's `}` — inside the cell, before `&`/`\cr` acts.
Perl's cell frame (`Alignment.pm` `bgroup`/`egroup` around the column)
unreads the tokens after the column has ended: babel.def:738-742
`\selectlanguage` = `\aftergroup\bbl@pop@language…` in a non-first cell ran
as the NEXT cell and, after the last cell, opened a spurious one
("`\@end@tabular` Attempt to close boxing group"; uantwerpenexam.cls:426
`\engdut`, uantwerpenexam-example2 41; derivative 101). Trigger:
`\begin{tabular}{cc}\selectlanguage{english}A&\selectlanguage{dutch}B
\end{tabular}`. Rust: `end_column` digests the cell frame's `\aftergroup`
list inside the cell. Guard:
`perfect_kernel_batch54::aftergroup_in_a_tabular_cell_fires_inside_the_cell`.

## 169. `\expandafter` retracts its saved brace before the expansion (Rust fixes)

tex.web §368 expands the second token FIRST and only then `back_input`s the
saved one (§325 retracts a scanned brace from `align_state`). Both engines
retracted first, so in `\exp_after:wN { \use_none:nn & …}` (numerica.sty:
1748 `\__nmc_delim_arg:` on the slash path of `\eval{1/8}`; mhchem's `\ce`;
tablists) the `&` was read at ledger 0 and fired the column template
mid-cell — the 6-error "close a group that switched to mode math" cascade
(numerica 83, tablists-rus 101, mhchem 14). Trigger (package-free):
`\newcommand\doit{\exp_after:wN { \use_none:nn & Z } }` in an `align*`
cell. Rust: retract after the one-step expansion. Guard:
`perfect_kernel_batch54::argument_scan_is_align_state_neutral`.

## 170. `\g@addto@macro` is an expandable side-effecting macro (Rust fixes)

latex.ltx:1832 `\long\def\g@addto@macro#1#2{\begingroup\toks@\expandafter{#1#2}
\xdef#1{\the\toks@}\endgroup}` appends at DIGESTION. `latex_constructs.pool.
ltxml:968` makes it an expandable `DefMacro` with the side effect in the
closure, so the number scan's one-token look-ahead (tex.web §444) after an
`\ifnum` operand EXECUTES it even in a false branch: numspell-english.sty:
79-105 `\ifnum\numspell@group@digit@i>0\numspell@{ hundred}\fi` with
`\numspell@#1` = `\g@addto@macro\thenumspell{#1}` spelled every group
("hundred and -twotwelve thousand, nought", then `\StrChar` on the leading
space → `\GenericError`; numspell 12 errors). Trigger: `\def\out{}\ifnum0>0
\g@addto@macro\out{WRONG}\fi[\out]`. Rust: latex.ltx's macro verbatim (the
hyperref and CJKutf8 bindings now provide the hook targets raw packages
append to: `\Hy@UseMaketitleInfos`, `\pdfstringdefPreHook`). The same
shape remains in `\AtBeginDocument`/`\AtEndDocument`/`\AtEndOfPackage`/
`\@addtofilelist`/`\nocite` (no corpus witness; latent). Guard:
`perfect_kernel_batch54::g_addto_macro_appends_at_digestion`.

## 171. `\nocite` defers its key unexpanded (Rust fixes)

latex.ltx's `\nocite` writes `\citation{#1}` through `\protected@write` at
the call site, expanding the key there. `latex_constructs.pool.ltxml:4214`
pushes the raw tokens to the end-of-document list, so a key held in a
transient macro — tufte-common.def:934 `\@for\@temp@bibkeyx:=\@tufte@
citations\do{…\bibentry{\@temp@bibkeyx}}` inside a `\marginpar`
(bibentry.sty:64 `\bibentry` = `\nocite`) — expands at `\end{document}`
when the loop variable is gone ("`\@temp@bibkeyx` is not defined"; tufte
sample-book, sample-handout). Trigger: `\def\keys{k1}\marginpar{\@for
\@temp@bibkeyx:=\keys\do{\nocite{\@temp@bibkeyx}}}`. Rust: the key is
expanded at the call site. Guard:
`perfect_kernel_batch54::nocite_expands_its_key_at_the_call_site`.

## 172. `{titlepage}` is locked (Rust fixes)

report/book define `{titlepage}` with `\newenvironment`, so a class may
`\def\titlepage{…}` as a plain vertical macro (uwthesis.cls:610, used as
`{… \titlepage }` at uwthesis.tex:95-102). `latex_constructs.pool.ltxml:
1183` locks the environment, the class `\def` is refused, and the bare
`\titlepage` opens the internal_vertical environment frame that the `}`
then meets ("Attempt to close a group that switched to mode
internal_vertical"). Trigger: `\def\titlepage{\par T\par}` then
`{\titlepage}`. Rust: not locked; the environment binding still applies
when not redefined. Guard:
`perfect_kernel_batch54::titlepage_environment_is_overridable`.

## 173. soul's color setters store a macro name unexpanded (Rust fixes)

`soul.sty.ltxml:75` `\setulcolor` (and `\setstcolor`/`\sethlcolor`, :93/:104)
store the argument unexpanded; real soul resolves the name through
`\color` at use time, expanding a macro-valued name — europasscv.cls:560
`\setulcolor{\ecv@textcolor}` ("Can't find color named '\ecv@textcolor'").
Trigger: `\def\n{red}\setulcolor{\n}\ul{x}`. Rust: the name is expanded.
Guard: `perfect_kernel_batch54::soul_color_setters_expand_a_macro_name`.

## 174. The array/tabular row continuation macros carry an unpaired `$` (Rust fixes)

Both engines retract `\@arraycr`/`\@tabularcr` to the alignment newline but
keep latex.ltx:16585-16594's `\@xarraycr`/`\@argarraycr` (`…}${}\cr`,
the closing half of `\@arraycr`'s `${` trick) and `\@xtabularcr`/
`\@argtabularcr`. tablists.sty's `\TeXr@arraycr` opens with `\iffalse{\fi`
(no `$`) inside its own raw `\halign` and dispatches to `\@xarraycr`, so
the `$` opens inline math the row's `\cr` cannot balance ("`\org@halign`
Attempt to close a group that switched to mode math"; tablists-rus 101,
Perl 12). Trigger: `\begin{tabenum}\tabenumitem a;\\ \tabenumitem b;
\end{tabenum}`. Rust: the continuation macros drop the `$`/brace halves and
keep the `\ifdim` spacing dispatch. Guard:
`perfect_kernel_batch54::array_continuation_macros_carry_no_math_shift`.

## 175. A content `.tex` re-input while reading a `.sty` is skipped (Rust-only, fixed)

Rust routed every `\input`/`\InputIfFileExists` issued while
`INTERPRETING_DEFINITIONS` through the once-only package guard, so a plain
`.tex` read twice from a `.sty` ran once (Perl `Input` re-reads). babel.sty:
4210-4227 inputs `babel-<lang>.tex` once per language occurrence; with
french as BOTH the class option and `main=french` the second read was
skipped, its `\BabelBeforeIni` descriptor never recorded, and french.ldf
never loaded (`\og`/`\fg`/`\ieme` undefined: paresse-fra). Rust: a plain
`.tex` request is reloadable in that path. Guard:
`perfect_kernel_batch54::content_tex_reinput_during_definitions_rereads`.

## 176. nmbib's `\citeall` runs natbib's low-level engine (Rust fixes)

nmbib.sty:343 `\citeall` → `\@@@citeall` (:347) opens with natbib.sty:780's
`\NAT@reset@parser` and uses 57 natbib internals that the high-level
`natbib.sty.ltxml` emulation (8 `\NAT@*`) never defines; there is no nmbib
binding (nmbib-sample 22). Trigger: `\usepackage{nmbib}…\citeall{key}`.
Rust: an nmbib binding raw-loads the style and emulates `\citeall` as
`\citet*`. Guard: `perfect_kernel_batch54::nmbib_citeall_is_a_cite`.

## 177. `\@declaredcolor`/`\@undeclaredcolor` are undefined (Rust fixes)

xcolor.sty:762-763 `\color` = `\@ifnextchar[\@undeclaredcolor\@declaredcolor`;
fancyqr.sty:20-22 calls the named-color branch `\@declaredcolor{tl!50!br}`
directly. Both engines bind `\color` as one primitive (`color.sty.ltxml`,
`color_sty.rs`) and lack the branches ("`\@declaredcolor` undefined";
fancyqr). Trigger: `\makeatletter{\@declaredcolor{red}x}`. Rust: both
branches defined over `\color`. Guard:
`perfect_kernel_batch54::color_switch_branches_are_defined`.


## 178. siunitx `S`/`s` cells are scanned with full expansion (Rust fixes)

siunitx.sty.ltxml L1414 reads the `S` cell with `XUntil:\lx@si@column@end`
(an `\edef`-strength scan). Real siunitx collects the cell unexpanded
(`\__siunitx_table_collect_begin:`), and a raw class's size command cannot
survive an unprotected full expansion: `\@setfontsize` (latex.ltx:14103)
tests `\ifx\protect\@typeset@protect` and its true branch reaches
`\@currsize` → `\normalsize` → `\@setfontsize` again — pdflatex's
`\edef\x{\small}` overflows the same way. Perl's primitive `\small` hides
it; under `rawclasses` (KOMA `\DeclareRobustCommand\small{\@setfontsize
\small…}`) a cell opening with an unbraced `\small` looped
(zugferd-invoice.sty:113, scrartcl: `PushbackLimit`). Trigger:
`\documentclass{scrartcl}\usepackage{siunitx}\begin{tabular}{S}\small a\\
\end{tabular}` with raw classes. Rust: the scan runs under LaTeX's
`\protected@edef` context (`\protect` = `\@unexpandable@protect`,
latex.ltx:1384) so robust commands stay `\protect\cs ` (the `ProtectedXUntil`
parameter type, OD #191), and the cell is emitted as one group as LaTeX's
column template does. Guard: `perfect_kernel_batch54::s_column_unbraced_size_command_is_scoped`.

## 179. Display listings open their group at expansion time (Rust fixes)

listings.sty.ltxml:117 `\begin{lstlisting}` pushes the listing's group with
`$STATE->bgroup` INSIDE the macro's expansion and closes it with a `}` in the
emitted tokens. tex.web §785 `align_peek` expands a cell's first token before
the row and cell frames exist, so in a `p{}` cell the group sits under the
cell's box frame and the box's own `}` closes the wrong group (`\endgroup`
"Attempt to close non-boxing group", `\@@tabular` "Attempt to end mode";
pfdicons-doc:996, tikzcodeblocks-documentation:679, shipunov). Trigger:
`\begin{tabular}{p{4cm}l}\begin{lstlisting}…\end{lstlisting} & b \\
\end{tabular}`. Rust: the expansion leaves no frame behind; it emits
`\begingroup\lx@lst@activate{env}[keys]` (keys re-activated at digestion)
and the trailer closes with `\endgroup`. Guard:
`perfect_kernel_batch54::block_listing_in_a_paragraph_cell`.

## 180. `\@tabarray` skips the array setup (Rust fixes)

latex_constructs.pool.ltxml:3765 `\@tabarray` = `\m@th\@@array[c]` opens
`\@@array`'s boxing group but never runs `\@array@bindings` +
`\lx@begin@alignment` (real latex.ltx: `\m@th\@ifnextchar[\@array
{\@array[c]}`). A package building its own array on it (t-angles.sty:491)
nested in an outer array cell under `\begingroup` broke the outer cell's
group (t-angles/t-manual, 101 errors; pdflatex clean). Rust: `\@array` is
defined as the internal behind `\array` and `\@tabarray` routes through it.
Guard: `perfect_kernel_batch54::tabarray_is_the_full_array_setup`.

## 181. colortbl.sty.ltxml lacks the `\CT@*` internal surface (Rust fixes)

Raw colortbl derivatives reach colortbl.sty internals — tabu.sty:720
`\CT@everycr\expandafter{…\the\CT@everycr…}` (colortbl.sty:116 `\let
\CT@everycr\everycr`, a toks register), tabulary/tabularht/keyvaltable
`\CT@arc@`, `\CT@column@color`… — and the Perl binding defines none
(srdp-mathematik). Rust: `\CT@everycr` is `\let` to `\everycr`; the colour
painters are `\relax`/`\@empty` no-ops as in colortbl.sty:75-166. Guard:
`perfect_kernel_batch54::colortbl_internal_surface_is_defined`.

## 182. `\@tabbing@accent` saves only `'` and `` ` `` (Rust fixes)

latex_constructs.pool.ltxml:3547 `\a<accent>` → `\@tabbing@<accent>`, with
:3572-3573 saving only `\@tabbing@'`/`` \@tabbing@` `` before tabbing rebinds
the control symbols, so `\a=` (encguide macron), `\a<` (greek-fontenc
breathings) and any never-rebound accent `\a"` are undefined. latex.ltx:10005
`\@tabacckludge` recovers the encoding-level accent by name. Trigger:
`\begin{tabbing}\a=o\end{tabbing}`. Rust: `=`, `<`, `>` are saved before the
rebinding and an unsaved accent falls back to the accent command itself.
Guard: `perfect_kernel_batch54::tabbing_accent_kludge_recovers_rebound_accents`.

## 183. `\noalign` bodies are pre-scanned as a token argument (Rust fixes)

TeX_Tables.pool.ltxml's alignment column reader takes the `\noalign` body as
a balanced argument; tex.web §15513 executes it to the `}` that closes the
no_align_group, and latex.ltx's `\hline` brace hack (`\noalign{\ifnum0=`}
\fi\hrule…`) has a char-constant `}` that the pre-scan miscounts, cutting the
body at `\ifnum0=` and leaking the rule into the alignment (boldline
`\hlineB`, shipunov/boldline-ex-en; LaTeXML's own `\hline` sidesteps the hack).
Trigger: the raw hack in any `\noalign`. Rust: the body is digested at
execution time inside its group. Guard:
`perfect_kernel_batch54::noalign_body_is_executed_to_its_group_end`.

The brace before the body (and before an `\halign`'s or a box's) is found as
§403 `scan_left_brace` finds it: expanded, past spaces and `\relax` (§774,
§785, §645). Perl's `\halign` reads the next token unexpanded ("Missing \halign
box" at `\halign\relax{`) and its `readBoxContents` skips any token to the next
`{` (`\hbox\relax\mac` drops what follows); Rust's `\noalign` (since the
batch-54 execution above) took `\relax` for the brace. Rust (60j): the gullet's
`scan_left_brace` in `\noalign`, `\halign`, `\valign` and the box reader.
Repros kernel-alignment/alignment_brace_is_scanned,
boxes-groups/box_brace_is_scanned; guards
`perfect_kernel_batch60::{alignment_brace_is_scanned, box_brace_is_scanned}`.

## 184. Box captures report their non-auto-closeable descendants (Rust fixes)

Core/Document.pm `closeToNode`/`closeNode` error "Closing … whose open
descendents do not auto-close" when `insertBlock`'s `ltx:_CaptureBlock_`
closes over a `verbatim`/listing line, then close anyway — the tree is right
and the diagnostic spurious (testnumberedblock `numVblock`; algpseudocodex,
coloredtheorem). Rust: a capture block is a completed box (tex.web box
completeness) and closes its descendants silently. Guard:
`perfect_kernel_batch54::capture_box_closes_its_descendants`.

## 185. Kernel `\author` is locked against class redefinition (design; Rust binds the class)

latex_constructs.pool.ltxml:1079 locks `\author` to keep the frontmatter
capture, so a class whose author machinery lives in `\renewcommand{\author}`
(quantumview.cls:661 → :673 `\internal@elseauthor` initialising the
`\@authorgroup` list) never runs it and the raw `\maketitle` loop
(quantumarticle.cls:1169 `\forlistloop`) meets an undefined list (quantumview
8, Perl 33; pdflatex clean). Rust: a quantumview class binding initialises the
lists; the lock stays. Guard:
`perfect_kernel_batch54::quantumview_author_group_lists_are_initialised`.

## 186. Raw `\halign` templates do not recognise a `\let`-to-`#` slot (Rust fixes)

TeX_Tables.pool.ltxml's `\halign` template reader tests the slot token's own
catcode; array.sty:97 `\let\@sharp##` makes the cell placeholder a control
sequence whose MEANING is `#`, and array's real `\@mkpream` (which runs raw
once a package redefines `\@array` over `\@tabarray`: sgame.sty:58,
tabularcalc, tabvar, epslatex) builds `…\d@llarbegin\@sharp\d@llarend…`, so
the template had no slot and `#` reached the stomach; `\omit`/`\noalign`
inside then "cannot be used here" (~6 docs). tex.web §783 checks the meaning
(`mac_param`). Trigger: `\let\@sharp=#` + `\ialign{\hfil\@sharp\hfil\cr a\cr}`.
Guard: `perfect_kernel_batch54::ialign_template_accepts_the_sharp_placeholder`.

## 187. `\DeclareMathVersion` registers nothing (Rust fixes)

latex_constructs.pool.ltxml:2658 no-ops `\DeclareMathVersion` while :5290
`\mathversion` accepts only `bold`/`normal`, so a class's own version (oz.sty:34
→ :70, iwonamath, askmaps `sans`, zed) errors "Unknown math version" (5 docs,
ozguide 28). Rust: declared names are registered and selectable (no font
change); an undeclared name still errors. Guard:
`perfect_kernel_batch54::declared_math_versions_are_selectable`.

## 188. `\ifinner` is true at the document body's galley (Rust fixes)

TeX_Logic.pool.ltxml:127 tests the MODE string (`internal_vertical`,
`restricted_horizontal`, `math` → inner), and latex_constructs.pool:314 opens the
document body as a frameless `internal_vertical`, so after `\par` at the main
galley `\ifinner` is true — paracol.sty:1996 `\ifinner\@parmoderr` errors "Not
in outer par mode" on every `\begin{paracol}` (tidyres); conversely a
`\parbox` interior in horizontal mode reads outer. tex.web §211: inner is the
sign of a box or non-display-math interior. Trigger: `\usepackage{paracol}
\begin{paracol}{2}…\end{paracol}`. Rust: a frame-bound `INNER_BOX` flag set by
framed mode switches (boxes, inline math; not display math) is the predicate.
Guard: `perfect_kernel_batch54::ifinner_is_the_box_frame_sign`.

## 189. beamer's `\usetheme` options never reach the theme (Rust fixes)

beamer.cls.ltxml no-ops `\usetheme` (and `\ProcessOptionsBeamer`), so a theme
option (`\usetheme[sidebar]{Verona}`, beamerthemeVerona.sty:43
`\DeclareOptionBeamer{sidebar}`) is dropped and the theme installs its
missing-option stub ("`\sidegraphics` defined only with the 'sidebar'
option"; beamer-verona-sidebar). Real: beamerbasethemes.sty:18
`\beamer@calltheme` = `\usepackage[{opts}]{beamertheme<name>}`,
beamerbaseoptions.sty:15 `\ProcessOptionsBeamer` = `\setkeys{\@currname}` over
the passed options. Rust: both real bodies. Guard:
`perfect_kernel_batch54::usetheme_options_reach_the_theme`.


## 190. A float inside a Block container is malformed instead of floating up (Rust fixes)

latex_constructs.pool.ltxml:3394 builds `figure`/`table` with a plain
`<ltx:figure>` opener, no `^` float-up marker, so a float opened inside a Block
container (`quote`, a list item, a box) is inserted there and rejected:
`malformed:ltx:figure <ltx:figure> isn't allowed in <ltx:quote>` (isorot/rotman,
bashful). LaTeX floats escape their environment to the page (pdflatex clean).
Trigger: `\begin{quote}\begin{figure}\caption{X}\end{figure}\end{quote}`. Rust:
the four float environments carry `^` (`floatToElement`), as do float.sty's
custom floats (bashful's `program`), the listings `lol` wrappers
(`\lstinputlisting` in minipage > quote, listings.sty.ltxml:239 identical),
rotating's sideways floats and subfloat's, so the float lands beside the quote
in the enclosing `ltx:para` (or inside the enclosing box). Guard:
`perfect_kernel_batch54::floats_escape_block_containers`.

## 191. A misplaced `\omit`/`\span` opens a group it never closes (Rust fixes)

TeX_Tables.pool.ltxml:128-135 (`\omit`) and the `\span` twin report the misuse
and then call `$stomach->bgroup` with `\let`s to `\relax` inside — the group is
never closed, so the next `}` closes it instead of its own frame and every
enclosing frame drifts by one. nicematrix.sty:5135 invokes `\multicolumn`
(= `\omit…`) in a measurement pass off any alignment; each use leaked two
frames and the manual ended in a `\Body` runaway to EoF (Perl recovers by
luck of its group model). tex.web §1128 `align_error`: "Misplaced \omit" is
reported and nothing else happens. Trigger: `A{\multicolumn{1}{c}{B}}C`.
Rust: error only. Guard: `perfect_kernel_batch54::misplaced_omit_does_not_open_a_group`.

## 192. `array` never lets `\tabularnewline` to the row break (Rust fixes)

latex_constructs.pool.ltxml:3792-3809 (`\@array@bindings`) lets `\\` to the
alignment newline but not `\tabularnewline`, which latex.ltx:16576 `\@array`
sets for `array` and `tabular` alike; the text `tabular` binding does it, the
math one did not, so `\tabularnewline` inside an `array` stayed latex.ltx's
top-level `\relax`. tabvar.sty:117-122's `C` column opens a varwidth box in its
`>{}` part and re-lets `\\` to `\TVtabularnewline` (→ `\tabularnewline`): the
row break became `\relax`, the next row fused into the last cell's box and every
following `&` was an "Extra alignment tab" (tabvar demo ×80; Perl identical).
Trigger: `\newcolumntype{C}{>{\begin{varwidth}{3cm}\let\\=\tabularnewline$}c<{$\end{varwidth}}}`
`\[\begin{array}{cC}a&b\\ c&d\end{array}\]`. Rust: `\@array@bindings` lets both.
Guard: `perfect_kernel_batch54::math_array_lets_tabularnewline_to_the_row_break`.

## 193. `\DeclareMathDelimiter` is ignored with four arguments (Rust fixes)

latex_constructs.pool.ltxml:2654 `DefPrimitive('\DeclareMathDelimiter{}{}{}{}', ignoredDefinition)`,
but latex.ltx:13531-13548 takes SIX arguments in both forms
(`{sym}{class}{font}{slot}{font}{slot}`), so the trailing `{font}{slot}` pair
leaks into the stream and a control-sequence symbol is never defined: oz.sty:261-264
`\DeclareMathDelimiter\ulcorner{4}{AMSa}{"70}{AMSa}{"70}` (…`\lrcorner`) left
`\ulcorner` undefined (ozguide ×4). Trigger: `\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}
\DeclareMathDelimiter\ulcorner{4}{AMSa}{"70}{AMSa}{"70} $\ulcorner a$`. Rust: six
arguments; a cs symbol is `\DeclareMathSymbol`'d from the small variant
(`\@xxDeclareMathDelimiter`), numeric classes 0-7 map as `\mathchar@type`. Guard:
`perfect_kernel_batch54::declare_math_delimiter_defines_the_symbol`.

## 194. Any defined `\<type>name` is taken as the counter's name noun (Rust fixes)

Base_Utility.pool.ltxml:1048-1055 `\lx@@fnum@@` composes the reference tag from
`\<type>name` whenever it is defined, assuming a parameterless noun
(`\figurename`). argumentation.sty:403 pairs counter `af` with the drawing command
`\NewDocumentCommand{\afname}{…}{… \node …}`, so `\refstepcounter{af}` executes
`\node` inside the tag and reports `undefined:\node` (Perl and Rust identical;
pdflatex never expands `\afname` there). Trigger: `\newcounter{af}
\NewDocumentCommand{\afname}{m}{\node[caption](x){#1};} \refstepcounter{af}`. Rust:
`\iflx@namenoun` (base_utilities.rs `is_name_noun`) admits `\<type>name` only when it is an
expandable macro whose arguments are all optional (an ltcmd `\__cmd_start_optimized:` dispatcher is
followed to its ` code` macro, a general one's signature read as written, a `\DeclareRobustCommand` wrapper followed
to its inner macro; 60d); `\lx@@fnum@@` and `\lx@typerefnum@@` fall back to `\the<type>` otherwise, and hyperref's
`\lx@autorefnum@@` (`is_name_noun_cs`, 60d) sets the `~` alone before the number when its first defined name is not a
noun — what hyperref's `\<type>name~` prints for amsthm's `\let\thmname\@iden`, without running a drawing command. Witness: argumentation-doc. Guards:
`perfect_kernel_batch56::{counter_name_command_is_not_a_name_noun, autoref_name_is_a_name_noun}`. The gemini
HANDOFF.md hypothesis (a `stomach::digest` mouth leaking into the parent stream) was
refuted: `stomach::digest` opens a non-autoclose mouth (`gullet.rs:3445`, `:1221`).

## 195. `\errmessage` is only a Note, so an `\errmessage` loop never trips the error cap (Rust fixes)

TeX_Debugging.pool.ltxml:71-74 `DefPrimitive('\errmessage{}', sub { Note(...) })`, but
tex.web §1283/§23571 runs `error` for `\errmessage` and §1887-1890 stops fatally at
100 errors. expl3's `\msg_error` is `\tex_errmessage:D` (expl3-code.tex:349), so a
package that repeats an error in a loop — csvsimple-l3.sty:227-235
`\__csvsim_read_head:` looping on a blank `\csvline` while `\ior_get:NNTF` on the
missing `<jobname>_sorted._csv` (the CSV-Sorter never runs without shell escape)
raises `file-error` each turn — repeats forever (Perl: 1,496+ in 120 s to the wall
timeout; Rust before the fix: 90,508 to TokenLimit, 39-byte XML) where pdflatex aborts
at 100. Trigger: `\csvreader[sort by=namesort.xml]{grade.csv}{}{X}` with no sorter.
Rust: `\errmessage` is a counted `Error!`, and the consecutive-error breaker ends the
loop with a partial document. Guard:
`perfect_kernel_batch56::errmessage_counts_toward_the_error_breaker`.

## 196. `\parbox`'s dispatch leaves the box body inside an open `\ifx` (Rust fixes)

latex_constructs.pool.ltxml:4748 `\parbox` = `\ifx.#2.\lx@parbox[#1]{#4}{#5}\else
\lx@parbox[#1][#2][#3]{#4}{#5}\fi`: the body `#5` is digested while the wrapper's
own conditional is still open, so a body that leaves an `\if` dangling — jourcl.cls:145
`\RecommendedPerson` (three `\if`, two `\fi`; pdflatex only warns "incomplete
\ifx") — meets the wrapper's `\else`: "Extra \else already saw \else for \ifx"
(jourcl, 3 errors; Perl 1 identical). Real `\@iiiparbox` has no such wrapper.
Trigger: `\def\ifempty#1{\def\temp{#1} \ifx\temp\empty }` … `\parbox{3cm}{\RP{x}}`.
Rust: the dispatch is `\ifx.#2.\expandafter\@firstoftwo\else\expandafter
\@secondoftwo\fi{…}{…}` — the conditional closes before the body digests. Guard:
`perfect_kernel_batch56::parbox_body_dangling_conditional_is_not_the_wrappers`.

## 197. `\iffontchar` is undefined (Rust fixes)

eTeX.pool.ltxml:335 leaves `\iffontchar` a comment; the dump's `\tex_iffontchar:D`
LETs to it and l3text / unicodefonttable code desyncs mid-conditional (Perl on a
0–00FF `\displayfonttable`: 101 errors + too-many-errors Fatal). Rust: defined from
the font file's coverage — a `\font`-declared TFM's populated `char_info` slots, a
fontspec-selected OpenType font's `cmap` (`latexml_core::common::font::coverage`;
`\fontspec`/`\setmainfont`/`\setfontface` record the resolved file as
`FONTSPEC_FONTFILE`); unresolvable fonts answer TRUE. Guards:
`perfect_kernel_batch56::iffontchar_reads_tfm_coverage`,
`iffontchar_bounds_unicodefonttable_to_font_coverage`.

## 198. listings.sty.ltxml never inputs `lstlocal.cfg` (Rust fixes)

listings.sty:2315-2316 inputs both `listings.cfg` and the user's `lstlocal.cfg`;
listings.sty.ltxml:1609 reads only the first, so a document shipping its own
`lstlocal.cfg` (labyrinth: `\pkgname`, `\meta`, the `{code}` environment) loses every
definition in it (13 errors + a token-limit Fatal). Guard:
`perfect_kernel_batch56::listings_reads_lstlocal_cfg`.

## 199. `\begin`/`\end` never fire the kernel's `env/<name>/*` hooks (Rust fixes)

latex.ltx:15347/15362/15388/15391 fire `\UseHook{env/#1/before|begin|end|after}`
around every environment; Perl's `\begin`/`\end` (latex_constructs.pool.ltxml:190-231)
consult only etoolbox's private `@environment@*` store, and `\AddToHook` is a no-op
(latex_base.pool.ltxml:833). Under a raw-loaded lthooks the hooks are registered but
never used: functional.sty `\AddToHook{env/demohigh/before}{\MyDeleteShortVerb}` left
shortvrb's `|` active for codehigh's rescan ("\verb ended by end of line", 10 errors).
Rust: `\begin{}`/`\end{}` (sect01.rs) and DefEnvironment's constructors (dialect.rs)
fire the four hooks in the kernel's own token shapes (latex.ltx:15386's
`\romannumeral\IfHookEmptyTF…` for `end`, load-bearing: an unexpandable token between an
alignment's last cell and `\endtabular`'s implicit `\crcr` leaks the cell group).
Guard: `perfect_kernel_batch56::kernel_env_hooks_fire_around_environments`.

## 200. The `\psset` constructor drops pst family key bodies (Rust fixes)

pstricks_support.sty.ltxml:622 `DefConstructor('\psset [] RequiredKeyVals:pstricks')`
consumes the keys without running them; raw pst-node.tex:1248-1257 defines
`\psk@mnodesize`/`\psk@mnode`/`\psk@mcol` only as the side effect of
`\psset[pst-node]{mnodesize=-1pt,…}`, and `\psm@endnode` (:1224) reads them:
`{psmatrix}` under pstricks-add (dsptricks 101 errors, pst-eucl; Perl 8 identical on
the repro). Rust: `\psset` stays pst-xkey.tex:60-63's family-aware `\setkeys+[psset]`
from the raw load. Guard: `perfect_kernel_batch56::psset_dispatches_family_key_bodies`.

## 201. No kernel `\inst`: classes defining it inside the title box (Rust fixes)

Base_Utility.pool.ltxml:549 leaves `\inst` to the class ("typically would
`\let\inst`" to `\lx@request@frontmatter@annotation`); Perl has no fallback.
bfhsciposter.cls:445,476 (`\cs_set_eq:NN \inst \__ptxcd_inst:n` inside
`\ptxcd_poster_setup_title_box:`) and beamerbasetitle.sty:148/233 define `\inst`
only in the scope where `\@author` expands; LaTeXML's `\author` digests its
argument at once, outside that scope → `undefined:\inst` (bfh-ci/DEMO-BFHSciPoster,
Perl 1 = Rust 1, lualatex 0). Rust: the kernel `\providecommand\inst[1]
{\textsuperscript{#1}}` beside `\author` (sect05.rs) — `\providecommand` as
beamerbasetitle.sty:262's own fallback (an empty `\inst`; the superscript is what
beamer's `\insertauthor` renders); class bindings' affiliation-linking `\inst`
(llncs, sv_support, inst_support) still win. Guard:
`perfect_kernel_batch56::kernel_inst_fallback_is_a_superscript`. **Superseded by
the addendum below (batch 56di): no kernel-global `\inst` exists any more.**




**Addendum (batch 56di, 2026-09-18).** The fallback is no longer a kernel-global
`\providecommand` and no longer a superscript: `\lx@author@withinst` provides `\inst`
inside a group around the author content only (the shape of Perl's
`\lx@author@withsup`) with its frontmatter meaning, an affiliation-link request
(`\lx@request@frontmatter@annotation[affiliation]`, llncs.cls.ltxml:50). The global
form blocked a class that STORES with `\newcommand\inst` (ptptex.cls:616
`\gdef\@inst{#1}`); that refusal typeset the affiliation into the body ahead of
`\maketitle` and put the frontmatter after it (manptp, 3 jing lines); K11
(`frontmatter_stores.rs`) reroutes ptptex's — and any raw class's — store-shaped
`\inst`/`\subtitle`/`\recdate`/`\abst`/`\pubinfo` setters through the frontmatter API; a footnote-SYMBOL `\inst{*}` keeps its glyph
(`\lx@frontmatter@keepsup`, the `$^{*}$` rule). Note the two witnesses differ: on
the poster Perl errors `undefined:\inst`; on ptptex Perl's raw `\inst` works and
silently drops the affiliation. Guards
`perfect_kernel_batch56::author_inst_is_an_affiliation_link_request` and
`kernel_fallbacks_never_block_newcommand::ptptex_inst_store_keeps_the_frontmatter_first`.

## 202. `\@tabarray` cells are forced into math (Rust fixes)

Perl `\@tabarray` = `\m@th\@@array[c]` (latex_constructs.pool.ltxml:3765) never
starts the alignment, and LaTeXML's array setup always binds math cells:
latex.ltx keys the cell mode on `\@classz` (`\array` :16550 → `\@arrayclassz`
= `$\@sharp$`; `\@tabular` :16561 → `\@tabclassz` = text). A deluxetable-style
raw scaffold (`\hbox\bgroup$\let\@classz\@tabclassz…\@tabarray`, aguplus.cls:305
`\pt@tabular`) therefore got math cells whose `$45^\circ$` opened a TEXT box:
"Script ^ can only appear in math mode" ×6 + mode-stack errors (aguplus.tex:633;
Perl 8-10 on the repros, pdflatex 0). Its `\endtabular` (latex.ltx:16554
`\crcr\egroup\egroup $\egroup`) also needs the trailing `$\egroup` LaTeXML's
constructor path omits, and the class's `\edef\pt@format{\string lcc}` (:314) needs
the template `\edef`-expanded (`\string`). Rust: `\@arrayclassz`/`\@tabclassz`
defined, `\@array@bindings`/`\@array` keyed on `\ifx\@classz\@tabclassz`,
`\endtabular` closes a raw-opened scaffold (`lx@raw@array@open`), and
`ReadAlignmentTemplate` expands every expandable except `\csname`/`\expandafter`/
`\noexpand` (OXIDIZED_DESIGN #206). Guards:
`perfect_kernel_batch56::{tabarray_cell_mode_follows_classz,
raw_tabular_scaffold_closes_and_template_expands}`.

## 203. An unbalanced `\abstract{` errors at `\end{document}` (Rust fixes)

screenplay-pkg.tex:67 `\abstract{\noindent\begin{quote}…\section…` never closes
its brace; pdflatex ends with tex.web §1335's "(\end occurred inside a group at
level 1)" warning. Perl's `\lx@add@abstract[]{}` reads the group as an argument
("readBalanced ran out of input") plus mode-close errors (10). Rust's incremental
path left the `{` frame for the bounded primitive's `egroup()` to meet ("Attempt to
close a group that switched to mode internal_vertical") and the document closed
over the open abstract (3 errors). Rust now: `\end{document}` abandons open plain
groups §1335-style (stack frame + `boxing` popped, `\aftergroup` discarded, one
warning), a bounded primitive closes only the frame it opened, the document's
close is lenient about elements those groups left open, and a `\section` inside
the open group ends the abstract from inside the nested brace body
(`until_terminal_inside_group`; OXIDIZED_DESIGN #207). Guard:
`perfect_kernel_batch56::unbalanced_abstract_brace_unwinds_at_end`.

## 204. thumbs.sty's shipout state machine never resets (Rust fixes)

thumbs.sty draws page-edge tabs at shipout (`\AtBeginShipout`, :1076) and resets
`\th@mbtoprint` there (:1151); LaTeXML never ships out, so `\thumbnewcolumn` after
`\addthumb` raises the package's own "\thumbnewcolumn after \addthumb" (:573) on
the second column (thumbs/thumbs-example; Perl identical; pdflatex 0). Rust:
`thumbs_sty.rs` loads the real package and applies its own `hidethumbs` branch
(:1531-1538). Guard: `perfect_kernel_batch56::thumbs_loads_in_its_own_hide_mode`.
\n

## 205. A raw `\def\multicolumn` runs `\@mkpream` against LaTeXML's array model (Rust fixes)

agupp.sty:599 re-`\def`s `\multicolumn` as latex.ltx:16603's `\multispan…\@mkpream{#2}
…`; `\@mkpream` executes `\@classz`/`\@acol` (latex.ltx:16641/16643), which only
`\array`/`\@tabular`'s raw scaffolds `\let` (:16550/:16560) — LaTeXML's constructor
`\tabular` never does, so the cell errors `undefined:\@classz` + `\@acol` and is typeset
outside the alignment model (aguplus.tex:731; Perl identical: its `\multicolumn` is
unlocked, latex_constructs.pool.ltxml:3702). Rust: `\multicolumn` is LOCKED like
`\tabular`/`\endtabular` (OXIDIZED_DESIGN #209); binding-level redefinitions
(colortbl…) load unlocked and still win. Guard:
`perfect_kernel_batch56::raw_multicolumn_redefinition_is_dropped`.

## 206. The alignment cell-head peek runs in the cell's mode (Rust fixes)

tex.web §15350 (`init_align`: a display/box `\halign` enters -vmode) and §15510
(`align_peek`): the per-column peek that expands the cell-head token runs in the
alignment's inter-row mode, internal vertical; `init_row` §15532 enters the cell's
restricted horizontal mode only afterwards. LaTeXML digests the whole `\halign`
body in restricted_horizontal (TeX_Tables.pool.ltxml:181; `readXToken` peek :376), so
a cell head `\ifhmode\else\expandafter\hbox\fi\bgroup…$…$…\egroup` (abntexto.tex:79-81,
in the class's `$$\halign{$#$\cr…}$$`, abntexto.cls:727-736) dropped its `\hbox`,
its inner `$` became a math END inside the template's math and the display, the
list and the following section never closed (abntexto 8 errors; Perl 4 — SHARED;
pdflatex 0). Rust: the peek (and `\noalign` material, §15513) runs under
`internal_vertical`, restored on every exit (`AlignPeekMode`, tex_tables.rs).
Guard: `perfect_kernel_batch56::alignment_cell_head_peeks_in_internal_vertical_mode`.

## 207. The SVG pgf driver makes a graphics-state scope a TeX group (Rust fixes)

pgfsys-common-pdf.def:37-38: `\pgfsys@beginscope` = `q`, `\pgfsys@endscope` = `Q`,
output literals pgf interleaves with its own boxes (`\pgfsys@begin@idscope`/
`\pgfsys@hbox`, pgfsys.code.tex:572-611/1524: a scope opened before a `\setbox…
\hbox` and closed inside it). pgfsys-latexml.def.ltxml:576-586 maps them to a real
`\begingroup`/`\endgroup`; the straddling scope interleaves with the box frame and
every later `\endgroup` reports "close non-boxing group" / "end mode" (the authors'
note at :887-890): msc/msc 24 errors and an empty chart, modernposter/demo 15
(Perl 25/31 — SHARED; pdflatex 0). Rust: the scope is the `<svg:g _scopebegin>`
element only, no stomach frame: the straddle repro and an empty `{msc}` chart are
clean. msc (21) and modernposter (14) still fail on a distinct root — pgf's own
`\pgfinterruptpicture`/idscope `\begingroup` inside `\pgf@maketext`'s deferred
`\hbox\bgroup` meets our fused mode/save frames (DIFFICULT_CASES D12, parked). Guard:
`perfect_kernel_batch56::pgf_scope_straddling_a_box_is_not_a_tex_group`.

## 208. `\eqno`/`\leqno` collect their tag instead of digesting it (Rust fixes)

tex.web §21745-21748 `start_eq_no` pushes a math-list level and returns to the main
loop: the tag is DIGESTED (assignments execute in place) up to the display's `$$`
(§22405-22432 `after_math`). TeX_Math.pool.ltxml:1239 gullet-collects the tokens up
to a terminator net, so mhequ.sty:184's `\@restoreMHComms` (`\let\\=\MHsavecr`)
after `\eqno{…}` (:307-311) never executed before `\\` was read, and the `\if…\fi`s
were swallowed ("Fell of the end reading tag"; mhequ-example 3, Perl 8 — SHARED;
pdflatex 0). Rust: `\lx@eqno EqnoTag` digests the tag as a bounded math sub-body
that stops BEFORE the display-end token and retracts it (same stop net for
`\eqno` abuse outside display math). Guard:
`perfect_kernel_batch56::eqno_digests_its_tag_material`.

## 209. `\globaldefs` globalizes the save-frame bookkeeping (Rust fixes)

State.pm:144-151 rescopes EVERY assignment to global while `\globaldefs>0` —
including the stack-frame records Stomach.pm:284-294 `pushStackFrame` writes
(`groupNonBoxing`, `groupInitiator`, `beforeAfterGroup`…) and `beginMode`'s
`BOUND_MODE`. tex.web §1214 adds the global flag only to the ASSIGNMENTS of
`prefixed_command`; the save stack (§274 `new_save_level`, §282 `unsave`) never
consults `\globaldefs`. So while msc.sty:2616 `\msc@global@set`
(`\globaldefs=1\relax\mscset{#1}\globaldefs=0`) runs pgfkeys code that opens
groups, a closed `{` group keeps reporting itself as the current frame, and every
later `\endgroup` reports "Attempt to close non-boxing group" (msc/msc: Perl 25,
Rust 21 → 0; latex.ltx:12612/12911/12976 `\globaldefs\@ne \math@fonts` and
tikzexternalshared carry the same shape). Rust: the frame bookkeeping binds
through `assign_local_unconditional` (state.rs), exempt from `\globaldefs` and
the `\global` prefix. Guard:
`perfect_kernel_batch56::globaldefs_does_not_globalize_the_save_stack`.

## 210. pgfmath trig is float, so pgf's exact-equality bisections never exit (Rust fixes)

pgfmath.code.tex.ltxml:182-218 evaluates `sin`/`cos`/`atan2` in float; pgf's
`\pgfmathpointintersectionoflineandarc` (pgfmathcalc.code.tex:366-468) bisects an
arc's parametric angle until the angle from the concave point to the arc point
EQUALS (`\ifdim\x pt=\q pt`, :447) the target angle in pgf's fixed-point trig —
its only exit. Float trig leaves the two ~0.0005° apart forever, so a
rounded-rectangle corner border query (a self-loop wire: tikz-cd `\ar[loop]`,
zx-calculus `\zxLoopAboveDots`; callout nodes, arXiv 2201.09268) spins — Perl to
its wall clock, Rust to the 50,000-box cycle fatal (zx-calculus/zx-calculus;
pdflatex 0). Rust: the intersection is bound in closed form
(`pgfmathcalc_code_tex.rs`, quadratic of the ray against the ellipse, the root on
the arc). Guard: `perfect_kernel_batch56::line_and_arc_intersection_is_closed_form`.

## 211. The document hooks are digested in an isolated mouth (Rust fixes)

latex_constructs.pool.ltxml digests `\AtEndDocument`'s list and the `enddocument`
hook (and etoolbox's `\AfterEndPreamble` store) with `Digest(...)` in a string
mouth, where latex.ltx:15255-15259 `\enddocument` runs `\UseOneTimeHook
{enddocument}` INLINE before `\@checkend{document}` and etoolbox.sty:1774-1776
runs `\@afterendpreamblehook` inline from `\document`. A box opened from the
one hook and closed from the other — modernposter.cls's document-spanning
`tikzpicture[overlay]` (`\AfterEndPreamble{\begin{tikzpicture}…}` +
`\AtEndDocument{\end{tikzpicture}}`): the pgfpicture `\setbox\hbox\bgroup`
and every node box — has its reader run dry at the hook's end and `end_mode`
meet pgf's `\begingroup` ("`\hbox` Attempt to end mode restricted_horizontal";
modernposter/demo Rust 14 → 0, Perl 15; pdflatex 0). Rust: both stores and both
lthooks slots are unread onto the galley, the end-document finalizer follows
(`\lx@finalize@document`). Guard:
`perfect_kernel_batch56::atenddocument_closer_reaches_the_galley_box`.


## 212. `KeyVals` drops xkeyval's empty family and misnames its keys (Rust fixes)

xkeyval's empty family is a family: `\XKV@makehd` (xkeyval.tex:83-88) builds the
key-macro header as `<prefix>@` plus `<family>@` only when the family is
non-empty, so `\define@key[psset]{}{precode}` (pstricks.tex:808) defines
`\psset@precode`, and pst-xkey.tex:53-57 accumulates `\pst@famlist` as
`,pstricks` precisely so every `\psset` searches "" before "pstricks"
(pdflatex probe: `FAMLIST=[,pstricks]`, an empty-family key is found).
Perl `KeyVal.pm:60-62` always emits `<prefix>@<keyset>@<key>` — `psset@@precode`
— and `KeyVals.pm:52` `grep { $_ ne '' }`s the empty entries away (LaTeXML
#2777), so pstricks' own `\psset{precode={},postcode={}}` (pstricks.tex:810)
and pst-node's `Xnodesep`/`Ynodesep` family report "unknown KeyVals key" and
are dropped. Minimal trigger: `\usepackage{pstricks}\makeatletter
\define@key[psset]{}{foo}{\gdef\x{#1}}\psset{foo=42}` — Perl never runs the
body. Rust: `keyval_qname` applies the header rule (`psset@precode`, leaving
the `@@` names to pstricks' delimited helpers `\psset@@dash`/`\psset@@ArrowInside`,
the collision that motivated #2777) and `KeyVals::new` keeps the empty family;
only an absent keyset list falls back to `_anonymous_`. Guard:
`perfect_kernel_batch56::xkeyval_empty_family_is_searched_by_psset`.

## 213. `\batchmode` and a terminal `\read` are no-ops, so wrong-engine halts never halt (Rust fixes)

tex.web §484: a `\read` whose stream is not open reads from the terminal, and
below scroll mode that is `fatal_error("*** (cannot \read from terminal in
nonstop modes)")`. iftex.sty:42-52 (`\IFTEX@Require`, behind every
`\Require<engine>`) and expl3-code.tex:10951-10954 (`\__msg_fatal_exit:`,
behind `\msg_fatal:nn`) rely on exactly that: `\batchmode\read -1 to \tmp`
stops the job the moment a package finds itself under the wrong engine. Perl
LaTeXML defines the four interaction modes as no-ops (TeX.pool `\batchmode`)
and a `\read` on an unopened stream does nothing, so bidi.sty:60
(`\sys_if_engine_xetex:F{\msg_fatal:nn{bidi}{cannot-use-engine}}`) and
ucharclasses.sty:987 (`\RequireXeTeX`) report their fatal message and then
RUN their bodies on the undefined `\XeTeXcharclass`; bidi.sty:354's `\loop`
never terminates ("Missing number, treated as zero" every iteration). Minimal
trigger: `\usepackage{bidi}` under pdflatex/lualatex, or bare
`\batchmode\read-1 to\x`. Rust (batch 56ak): the interaction modes set a
global `INTERACTION_MODE`, a closed-stream `\read` below scroll mode is a
Fatal, iftex's `\Require*` bodies are the real ones (the binding had 13
no-ops). 23 sweep-57 documents traded a 3,288,334,336-byte `alloc_failed`
after ~45 s for one Fatal in seconds, matching their lualatex oracle's exit.
Guard: `perfect_kernel_batch56::batchmode_terminal_read_halts_the_job`.

## 214. `\pdfximage` never sets `\pdflastximagepages` (Rust fixes)

pdfTeX manual §8.9: `\pdfximage` registers the image and sets `\pdflastximage`
(the object number) and `\pdflastximagepages` (a PDF's page count; 1 for a
bitmap). Perl LaTeXML's pdfTeX pool (pdfTeX.pool.ltxml:131-145) leaves `\pdfximage`
UNDEFINED and defines both registers as constant 0, so pdfpages' `\AM@getpagecount`
(pppdftex.def:79-82), the pdfTeX l3 backend's `\__graphics_backend_get_pagecount:n`
and any document that tests the count (`\ifnum\pdflastximagepages=1`, the
notebeamer demo) see 0. Minimal trigger: `\pdfximage{example-image-a4.pdf}
\the\pdflastximagepages` → Perl an undefined-macro error and 0, pdflatex 1. Rust (batch 56ap): the
primitive resolves the file (pdfTeX's extension search), reads the root
`/Type /Pages` node's `/Count` from the raw bytes or the inflated object
streams (`latexml_core/src/util/image.rs::read_pdf_page_count`), and sets both
registers globally; the DVI-persona l3 hook in latexml.sty now reports the
real count instead of l3's constant-1 fallback. Guard:
`perfect_kernel_batch56::pdfximage_reports_the_pdf_page_count`.

## 215. Choice-key bin macros get catcode-12 letters (Rust fixes)

xkeyval.tex `\XKV@checkchoice` stores the value chosen through
`\define@choicekey{fam}{key}[\bin\nr]{choices}{code}` in `\bin` with the
value's ORIGINAL catcodes (the `*` form lowercases with `\lowercase`, which
keeps catcodes), so packages compare it with `\ifx` or `\in@` against
catcode-11 lists. Perl KeyVal.pm:143 binds `\bin` as `Explode($nvalue)` —
every letter catcode 12 — and the Rust port copied it (`keyval.rs
define_choice`). powerdot.cls:54-61 builds `\pd@cursetup` through such a key
and :353-387 `\pd@pdifs@tup` tests it with `\in@` against `\pd@@ifsetup`;
the mismatch leaves `\ifpd@ifsetup` false, so :363 never defines any
`\pd@@<key>` (titlepos, titlewidth, titlefont, tocpos, tocwidth…). Minimal
trigger: `\define@choicekey{f}{k}[\v]{a,b}{}\setkeys{f}{k=b}\def\b{b}
\ifx\v\b same\else diff\fi` → Perl "diff", pdflatex "same". Rust (batch
56ar): the bin macro holds the original tokens (a catcode-preserving lowercase
for `*`). Witness powerdot-fuberlin exampleClass/exampleStyle (20 undefined
`\pd@@…` each, surfaced when batch 56ao started digesting `\rput` bodies).
Guard: `perfect_kernel_batch56::choicekey_bin_macro_keeps_letter_catcodes`.

## 216. `\index` scans forward to the next `{` (Rust fixes)

Perl's `SanitizedVerbatim` parameter (latex_constructs.pool.ltxml:4376-4383,
`readUntil(T_BEGIN)`) discards everything up to the next `{` wherever it is, so
a bare `\index` in prose swallows the following text and, in varindex.dtx:1497
("the \index command … multiple \index entries"), the brace of `\end{abstract}`:
the abstract runs to the end of the document (`<ltx:TOC>` and every section
inside `<ltx:abstract>`, `ltx:section isn't allowed in ltx:abstract` ×8).
latex.ltx `\index` = `\@bsphack\begingroup\@sanitize\@wrindex`, and `\@wrindex#1`
reads ONE undelimited argument: a brace group, or the single next token.
Rust (batch 56ba) skips spaces, reads a balanced group when a `{` follows and
one token otherwise. Trigger: `\begin{abstract}the \index command\end{abstract}
\section{S}` — Perl nests the section in the abstract; pdflatex indexes `c`.
Guard `bare_index_takes_one_token`.

## 217. etoolbox binding omits `\gundef` (Rust fixes)

etoolbox.sty.ltxml:867-872 transcribes `\undef`, `\csundef` and `\csgundef` but
not etoolbox.sty:931 `\newrobustcmd{\gundef}[1]{\global\let#1\etb@undefined}`.
yquantlanguage-groups.sty:185 calls `\gundef\yquantgroup@registers@text` in every
register-group cleanup, so each `yquantgroup` cascades (`\etb@tempa`,
`\yquant@lang@attr@value` … undefined, then unbalanced `\hbox` ends). Trigger:
`\usepackage{etoolbox}\def\x{1}\gundef\x\ifdefined\x still\else gone\fi`.
Rust (batch 56bd) adds the line. Guard `etoolbox_cs_definers_are_protected_and_gundef_exists`.

## 218. `annote` fields are typeset as notes, so `.bib` typos in them surface (kept)

BibTeX.pool.ltxml:679-681 maps `annote` to `\bib@@field{ltx:bib-note}[role=annotation]`,
so LaTeXML typesets a field that no bibtex/biber style prints. A macro typo in
an annotation — biblatex-chicago's dates-test.bib:169 `\textt{…}` for `\texttt` —
therefore becomes `Error:undefined:\textt` in both engines (cms-dates-sample,
cms-trad-sample) while the real PDF is clean. Trigger: `@book{k, author={A},
title={T}, year={2000}, annote={\textt{x}}}` + `\nocite{k}\bibliographystyle{plain}`.
Perl-origin policy, not a binding gap: the annotation is content LaTeXML chooses
to keep; a source typo is not defined away. Kept as-is (batch 56ca).

## 219. A Unicode-engine-only manual under pdfTeX emulation: fontsetup's `fspdefault.tex` runs before amsmath (kept)

fontsetup.sty:5 `\iftutex` picks unicode-math (which loads amsmath) only on XeTeX/LuaTeX;
under pdfTeX it raises "Use Unicode-compliant TeX-engines" (:15) and still `\input`s
`fspdefault.tex` (:206) when loaded without options, whose line 307
`\DeclareMathOperator*{\convolution}{…}` therefore precedes any amsmath load:
`undefined:\DeclareMathOperator` then `undefined:\convolution`. LaTeXML emulates pdfTeX
(`\iftutex` false: Perl iftex.sty.ltxml:29/44, Rust iftex_sty.rs:31/44) and both engines
report the pair, as pdflatex does ("Undefined control sequence" at fspdefault.tex:307).
Trigger: `\documentclass{book}\usepackage{fontsetup}\begin{document}Hello.\end{document}`.
Witness latex-via-exemplos (which itself demands XeLaTeX at :24). Kept: the only cure is
reporting a Unicode engine, a parked family with a document-wide blast radius. Corpus
footprint: this one manual (the other fontsetup users load it with an option and skip
fspdefault). Root-causer `~/data/pk_agents/w23/regr83/declaremathoperator/NOTES.md`.

## 220. xytree's `\xyconnect` inside `\xymatrix` under the SVG `\xy` overlay: `\xy@@ix@` never bound, digest runaway (kept)

xytree.sty:41-66 drives each tree through `\xymatrix` (xymatrix.tex:54/60, text mode:
`\xy \nter@\endxy`), and LaTeXML's `\xy` = `\if\inxy@\lx@xy@svgnested\else\lx@xy@svg\fi
\lx@xy@original` (Perl xy.tex.ltxml:148, Rust xy_sty.rs:382, byte-identical) digests the
picture through a Stomach constructor. Inside the matrix `\halign` the overlay
desynchronises xy's conditional/group nesting ("not in a conditional" `\else`/`\fi`,
"Attempt to close a group that switched to mode restricted_horizontal"), so
`\xy@@ix@` — only ever `\let`-bound by `\plainxy@` (xy.tex:269) or `\xyqall@`
(xymatrix.tex:309) — is undefined when `\entry@@norm` (:296) calls it; the `<ERROR>`
leaves its entry argument's braces unconsumed and the box nesting runs away. Both
engines: 10 Errors + 1 Fatal (Perl `deep_recursion` in `digestUntil`, Rust
`Fatal:Stomach:Recursion`). Trigger:
`\documentclass{article}\usepackage{xytree}\begin{document}\xytree[2]{\xyconnect[->](R,L){0,1}"^<{x}" &}\end{document}`.
Witness xytree/xytree-doc-en (pdflatex clean). Kept: the fix is the xy emulation itself
(bind `\xy@@ix@` before the entry loop, keep the `\halign` group), HIGH risk, one doc.
Root-causer `~/data/pk_agents/w59/xytree/`.

## 221. A successful etoolbox `\patchcmd` turns a nested macro's parameter into the outer one (FIXED in Rust)

Perl's native `\patchcmd` (etoolbox.sty.ltxml:1290-1310) patches the `UnTeX` string of the
stored body and re-installs it through `TokenizeInternal(...)->packParameters`
(Expandable.pm:31). `Token.pm:306-308` renders a stored `CC_ARG` as `#1` and a `CC_PARAM` as a
bare `#`, so a nested macro's `##1` (stored as one PARAM + digit) comes out as `#1` — and
`packParameters` (Tokens.pm:122-140) re-packs it as the OUTER parameter. Trigger:
`\usepackage{etoolbox}\def\outer#1{\def\inner##1{[#1|##1]}\inner{b}}\patchcmd{\outer}{[}{(}{}{}`
then `\outer{a}` — pdflatex `(a—b]`, Perl `(a—a]b`. Witness pgfornament-han/pgfornament-han-doc:
pgfornament.sty:47-51's `\def\m ##1 ##2 {…}` path operators inside the patched
`\pgf@@ornamenthan` read the ornament number — every SVG point collapsed. **Rust (FIXED,
batch 56gt):** the body string doubles each PARAM token (`##`, as `\meaning` and etoolbox's own
patterns write it), so the re-pack is exact; `etoolbox_sty.rs` `\patchcmd`. Guard
`perfect_kernel_batch56::patchcmd_keeps_nested_macro_parameters`.

## 222. A space in a macro's prefix delimiter swallows every following space (FIXED in Rust)

Perl's `readMatch` (Gullet.pm:614-617), which matches a `\def`'s literal tokens before its
first parameter (the `Match` parameter), skips every space token that follows a matched
space: "If this was space, SKIP any following!!!". TeX matches delimiter tokens one for one
(tex.web §392), and consecutive space tokens only arise from expansion, the mouth having
already collapsed source spaces. xint relies on the second space surviving
(`\xintZapSpaces`' `\XINT_zapsp_b`-style loop). Trigger:
`\long\def\myfirstofone#1{#1}\long\def\showit#1\stop{(\detokenize{#1})}`
`\myfirstofone{\def\again\Bdelim} {\showit}\def\Bdelim{}`
`\def\mk#1{\def\mk{\again\Bdelim#1#1X\stop}}\mk{ }`, then `\mk`. pdflatex prints `( X)`, Perl
`(X)`. Witnesses: xint/xinttools users (ipsum/ipsum-doc: 70 `Match` errors under the strict
delimiter check of batch 56gn). **Rust (FIXED, batch 56gy):** `gullet.rs` `read_match`
consumes exactly the delimiter's tokens. Guard `perfect_kernel_batch56::prefix_space_delimiter_matches_one_space`.

## 223. A `\relax` or space between a prefix and its command drops the prefix (FIXED in Rust)

Perl's Stomach clears the pending prefixes after any non-prefix primitive, `\relax`
included (Stomach.pm:212 `clearPrefixes unless $meaning->isPrefix`), and at every
space character (:234). TeX reads on to the next non-blank non-relax token after a
prefix (tex.web §1211, §404), so `\protected\relax\def\foo` defines a protected
`\foo`, and a space there is skipped, not typeset. catoptions defines every
`\robust@def*` macro as `\protected\relax\def` (catoptions.sty:375-381, :771-773):
unprotected, `\cpt@newv@riables` expanded inside `\cpt@newvariables`' `\edef`
(:1277), ran off its `[#4]` delimiter, and every later internal came out undefined.
That is 70 errors per `\usepackage{catoptions}` in Rust and 103 in Perl (keyval2e,
concepts, other ltxkeys users). The failure can also be silent. Trigger:
`\protected\relax\def\foo#1[#2]{X#1Y#2Z}\edef\bad{\noexpand\@testopt{\foo{Q}}{}}`
then `\bad[W]` gives pdflatex `XQYWZ`, and Rust and Perl `XQ[]YWZ` with no error.
**Rust (FIXED, batch 56hb):** `stomach.rs` keeps the prefixes across a `\relax`
(its definition, so `\let` copies too) and skips a space while one of TeX's
prefixes is pending (`state::has_tex_prefixes`; LaTeXML's own `didpar` keeps Perl's clearing). Guard
`perfect_kernel_batch56::relax_after_a_prefix_keeps_the_prefix`.

## 224. A bare `\usepackage{babel}` ignores the language given as a class option (FIXED in Rust)

`\documentclass[ngerman]{article}\usepackage{babel}`: babel takes the main language
from the class options (babel.sty:4197-4245), loads it last and prepends it to
`\bbl@loaded` (:4130-4132). Its `.ldf` then runs `\main@language` (ngermanb.ldf:255
`\ldf@finish`). Perl's babel binding runs no `\extras<lang>` at all, so blindtext
prints its Latin default. Rust's `.ldf` bindings (lib.rs `ngerman`/`french`/… interception)
never run `\main@language`, and `\lx@babel@activate@mainlang` read the language only
from `\opt@babel.sty`, so it fell back to english. The result was `xml:lang="en"`,
English captions, and English blindtext in a German letter (scrlttr2copy/letter-copy-test,
recall 20.7). Trigger: the line above plus `\usepackage{blindtext}` and `\blindtext`;
pdflatex prints "Dies hier ist ein Blindtext". **Rust (FIXED, batch 56hd):** with no
babel package options, `\bbl@loaded`'s first language is the main one. Guard
`perfect_kernel_batch56::bare_babel_takes_the_class_language`.

## 225. microtype's `babel`+`kerning` never switch off French babel's active characters (FIXED in Rust)

microtype.sty:3162-3195: with `babel=true,kerning=true` and a French-family (or
Turkish) option among babel's or the class's options, microtype runs
`\shorthandoff{:;!?}` (`:!=`) at `\begin{document}`. Perl's microtype binding (and
Rust's until 56hf) is a stub. Code frozen in the preamble relies on the switch-off:
cahierprof.sty:366-388's `\tikzmath{…; …}`, run from its `\AtBeginDocument` hook,
needs an other-catcode `;`, or tikzmath (tikzlibrarymath.code.tex:131-136) picks the
active-`;` delimiter and the statements leak as undefined control sequences. Perl
gives 15 errors; Rust gave 11-12 plus a timeout once 56hd made the class-option French real.
Trigger: `\documentclass[french]{article}\usepackage{babel}\usepackage{tikz}\usetikzlibrary{math}`
`\usepackage[babel=true,kerning=true]{microtype}\newcommand\calc{\tikzmath{\cc=int(5); \s=int(6);}}`
`\AtBeginDocument{\calc}`. **Rust (FIXED, batch 56hf):** `microtype_sty.rs` sets
`\ifMT@babel`/`\ifMT@kerning` from its options and registers that begin-document
switch-off. Guard `perfect_kernel_batch56::microtype_babel_kerning_switches_off_french_shorthands`.

## 226. `\import`/`\subimport` pop everything the imported file defines (FIXED in Rust)

import.sty.ltxml L44-47 wraps the imported `\input` in `{…}`; real import.sty:65-92
runs it at the caller's level. The file's `\newcommand`s vanish at the `}`, and so do
the definitions of any package it loads, although the package's loaded-flag is global.
Trigger: a file `m.tex` holding `\newcommand{\mymac}{M}`, then
`\usepackage{import}\subimport{}{m.tex}` and `\mymac` in the body gives
`Error:undefined:\mymac`; pdflatex prints M. **Rust (FIXED, batch 56hu):** the input
runs ungrouped, with the search paths saved and restored around it (OXIDIZED_DESIGN
#280). Guard `regress_2605_clusters::subimport_keeps_definitions`.

## 227. listings binding lacks `\lst@ifdisplaystyle` (FIXED in Rust)

listings.sty:1742 `\let\lst@ifdisplaystyle\iffalse`; the Perl binding never defines it.
A style that tests it, `\lstdefinelanguage{x}{basicstyle=\ttfamily\lst@ifdisplaystyle\scriptsize\else\fi}`
with `\lstset{language=x}`, gives three errors per listing (undefined, `\else`, `\fi`):
arXiv 2605.12091, Fatal. **Rust (FIXED, batch 56hu):** `listings_sty.rs` lets it to
`\iffalse`. Guard `regress_2605_clusters::lst_ifdisplaystyle_is_false`.

## 228. xcolor: a repeat load drops its name sets; active separators corrupt expressions (FIXED in Rust)

(a) `\usepackage{xcolor}\usepackage[dvipsnames]{xcolor}` defines no `Maroon`: the option's
handler is `\relax` after the first `ProcessOptions` (xcolor.sty.ltxml:52-53), where real
xcolor processes the key again (xcolor.sty:171-193). (b) `\catcode`!=13\def!{\itshape}`
then `\textcolor{red!50!black}{x}`: xcolor.sty.ltxml:561 expands the expression with the
active `!`, where real xcolor's `\XC@edef` (xcolor.sty:104-116) makes it stand for itself.
arXiv 2605.28926 and 2605.30133 fatal in both engines. **Rust (FIXED, batch 56hu):**
OXIDIZED_DESIGN #281. Guards `regress_2605_clusters::xcolor_reload_loads_dvipsnames`,
`xcolor_expression_ignores_an_active_bang`.

## 229. algorithm2e `{procedure}`/`{function}` lose their caption (FIXED in Rust)

Perl raw-loads these environments; their `\algocf@setcaption` lets `\@caption` be
`\algocf@caption@proc#1[#2]#3` (algorithm2e.sty:2402, :2436), and LaTeXML's `\caption`
supplies no `[short]`. Trigger: `\usepackage[ruled]{algorithm2e}`
`\begin{procedure}\caption{P(x)}\KwData{x}\end{procedure}` gives 14 "Missing argument"
errors and a float with no caption. **Rust (FIXED, batch 56hu):** OXIDIZED_DESIGN #279.
Guard `regress_2605_clusters::algorithm2e_procedure_caption_is_bound`.

## 230. An autoload fired inside a group loses the package at the `}` (FIXED in Rust)

`DefAutoload` (Package.pm:1086-1105) requires the package in the current frame, while
the package's loaded-flag is global. OmniBus.cls.ltxml:49-51 autoloads natbib on
`\citep`: `\documentclass{nosuchclass}` then `{\itshape a \citep{x}} b \citep{y}` gives
`Error:undefined:\citep` for the second. arXiv 2605.08349 and 2605.14513: 101 errors +
Fatal. **Rust (FIXED, batch 56hu):** OXIDIZED_DESIGN #282. Guard
`regress_2605_clusters::autoloaded_package_outlives_the_group`.

## 231. `\fontdimen`'s font identifier is read unexpanded (FIXED in Rust)

tex.web §577 `scan_font_ident` gets the next non-blank non-call token, expanding; Perl's
`FontToken` (TeX_Fonts.pool.ltxml) reads it raw. `\fontdimen8 \ifx\x\y\textfont\else
\scriptfont\fi 3` then takes `\ifx` as the font and leaves `\else`/`\fi` outside any
conditional. Trigger: the `\mathpalette` underline macro of arXiv 2605.21425 (1000
"not in a conditional", Fatal in both engines). **Rust (FIXED, batch 56hv):**
`tex_fonts.rs` `FontToken` reads with `read_x_non_space`. Guard
`regress_2605_clusters::fontdimen_font_identifier_is_expanded`.

## 232. `\textbf`/`\texttt`/… end with a `}` character (FIXED in Rust)

latex.ltx's `\DeclareTextFontCommand` closes the text branch with `\expandafter\egroup
\fi`; latex_constructs.pool.ltxml:5272-5281 uses `{\ttfamily #1}`. A `\futurelet`
character scanner run over the argument peeks the catcode-2 `}`: `\usepackage{seqsplit}`
`\seqsplit{\texttt{a\_b}}` loops to the conditional limit (arXiv 2605.04530; Perl lacks
a seqsplit binding, so only an inline copy shows it there). **Rust (FIXED, batch 56hv):**
`sect13.rs` `\text..` close with `\expandafter\egroup\fi`. Guard
`regress_2605_clusters::text_font_command_ends_with_egroup`.

## 233. glossaries labels are digested to build `key=` (FIXED in Rust)

glossaries.sty.ltxml:30/:89 read the entry label as a `{}` argument, digested, where
only its string is used. `\newglossaryentry{beat_frequency}{name={x},description={y}}`
gives "_ can only appear in math mode" (arXiv 2605.01773: 122 such labels, Fatal).
**Rust (FIXED, batch 56hv):** `glossaries_sty.rs` reads it ExpandedSemiverbatim, as
`\label` reads its label. Guard `regress_2605_clusters::glossaries_underscore_label_is_a_key`.

## 234. orcidlink's emptiness test ends a p-column cell (FIXED in Rust)

orcidlink.sty.ltxml:28 tests `\orcidlinkX`'s optional texts with `\ifx&#1&`, where the
real package uses etoolbox's `\ifstrempty`. A bare `&` read at alignment brace level 0 is a
column end (tex.web §342, Gullet.pm:223-226/399-402), and `\bgroup` does not raise that
level: `\begin{tabular}{p{3cm}}\bgroup A\orcidlinkX{}{0}{}\egroup\\\end{tabular}` gives 7
errors in both engines. Perl's `\textbf` is `{\bfseries #1}`, which shields
`\textbf{A~\orcidlink{…}}`; Rust's is latex.ltx's `\bgroup…\egroup` since batch 56hv (KPE
#232), which exposed it (arXiv 2605.21922). **Rust (FIXED, batch 56hw):** the test is
`\if\relax\detokenize{#1}\relax`. Guard `regress_2605_clusters::orcidlink_in_a_p_cell_keeps_the_cell`.

## 235. Bindings drop internals that raw classes call (FIXED in Rust)

A hand-written binding replaces the raw package or class, so the internal
macros the real file defines are missing when a raw class (loaded under
`[rawclasses]`) uses them. The Perl bindings lack them too:
- `report.cls:277`/`book.cls:283` `\@chapapp`;
- `amsmath.sty:1073` `\env@matrix`;
- `hyperref.sty:3106` `\HyLang@addto`;
- `lineno.sty:1254` `\linenumberdisplaymath`;
- `amsfonts.sty:59-60` `\DeclareSymbolFont{AMSa|AMSb}` (so `\symAMSb`);
- `natbib.sty:344` `\NAT@find@eq`;
- `varioref.sty:58` `\vref@addto`.

book/report also defaulted `\@titlepagefalse` where the classes set `\@titlepagetrue`
(book.cls:51), so jurabook's "does not support notitlepage" fired. mdframed's
`\mdtheorem` was missing from the Rust binding (Perl loads mdframed raw).

Trigger: `\documentclass{utexasthesis}` (TeX Live class census 2026-09-24:
utexasthesis, willowtreebook, unbtex, ascelike, acmart-tagged, univie-ling-*,
rbt-mathnotes*, jurabook, hausarbeit-jura, newlfm).

**Rust (FIXED, batch 56hy):** each binding carries the real definition, and
newlfm's fancyhdr marker `\ps@@empty` (fancyhdr.sty:849-851) is set too. Guard
`class_census::binding_internals_classes_call`.

## 236. `\chardef` & co. match their optional `=` unexpanded (FIXED in Rust)

tex.web §1224 `shorthand_def` reads the optional `=` with `scan_optional_equals` (§405),
whose `get_x_token` expands. Perl's `\chardef`/`\mathchardef`/`\countdef`… prototypes
(`SkipMatch:=`, TeX_Character.pool.ltxml:150; Gullet.pm:604 `readMatch` uses the
non-expanding `readToken`) do not. Trigger: `\def\eqfour{=4\relax}\chardef\x\eqfour`
gives "Missing number, treated as zero", `\x` = 0 and a leaked `=4`. catoptions.sty:215/1756
sets its option-stack limit that way, so the second option-state push errs (cv4tw,
arabic-book; TeX Live class census 2026-09-24). `\let`'s optional `=` stays unexpanded
(§1221). **Rust (FIXED, batch 56hz):** the shorthand definitions read it with
`read_keyword(&["="])`. Guard `class_census::chardef_optional_equals_expands`.

## 237. `\pagestyle` runs no `\ps@<style>` (FIXED in Rust)

latex.ltx:18297-18300 `\pagestyle{#1}` runs `\ps@#1`. Perl makes it a no-op
(latex_constructs.pool.ltxml:997 `DefPrimitive('\pagestyle{}', undef)`), so a class
that defines macros inside its page-style body never gets them. dccpaper-base.sty:387-470
defines `\TitleHead`/`\TitleFoot`/`\NormalHead`/`\NormalFoot` inside
`\ps@title`/`\ps@dccpaper`, and its `\AtEndPreamble` code then uses them: 4 undefined
control sequences (idcc, ijdc-v14, ijdc-v9; TeX Live class census 2026-09-24). Trigger:
`\makeatletter\def\ps@mine{\def\Head{HEAD}}\makeatother\pagestyle{mine}` then `\Head`.
**Rust (FIXED, batch 56hz):** `\pagestyle` is `\@ifundefined{ps@#1}{}{\@nameuse{ps@#1}}`
(an unknown style stays silent, as before); `\thispagestyle` stays a no-op. Guard
`class_census::pagestyle_runs_its_ps_macro`.

## 238. `\@ifpackagelater` is always true, and `\ver@<file>` is stored unexpanded (FIXED in Rust)

latex.ltx:18405-18411 `\@ifpackagelater{pkg}{date}` compares the date in `\ver@pkg.sty`,
which `\ProvidesPackage`/`\ProvidesClass` store by `\protected@xdef` (:18481-18483) and
`\ProvidesFile` by `\xdef` (:22454-22457). Perl answers "later" unconditionally
(latex_constructs.pool.ltxml:972-973 `DefMacro('\@ifpackagelater{}{}{}{}', '#3')`) and
stores the version unexpanded. A package not loaded yet then counts as recent:
yathesis.cls:459 passes `main=\YAD@mainlanguage` to babel inside
`\@ifpackagelater{babel}{2013/04/15}`, passes it again at :519, and babel refuses the
second ("Bad option 'main=french' … previous setting of 'main'"). Trigger:
`\makeatletter\@ifpackagelater{babel}{2013/04/15}{T}{F}` before babel loads (pdflatex F,
Perl T). The unexpanded storage matters once the test is real: expl3.sty's
`\ProvidesExplPackage{expl3}{\ExplFileDate}…` stored `\ExplFileDate \space …`, whose
date parse reads the year alone, so ctex, xparse and l3keys2e called expl3 "too old".
**Rust (FIXED, batch 56ia):** the kernel's `\@ifl@ter` definitions, and the versions stored
expanded. The loader defines `\ver@<file>` for every file it loads (content.rs:
`provides_version_of`, else `\fmtversion`). Guard
`class_census::ifpackagelater_reads_the_loaded_version`.

## 239. titlesec keeps no `\ttls@<section>` record (FIXED in Rust)

titlesec stores each title's spacing as `\ttls@<section>` =
`{left}{right}{before}{after}{afterindent}` (titlesec.sty:640-658) and fills it at load for
`\section`…`\subparagraph` (`\ttl@extract`, :1579-1628). The Perl binding
(titlesec.sty.ltxml) maps only `\titleformat`/`\titlespacing`/`\titleclass`, so the record
is undefined. ctex reads it when titlesec finishes loading
(ctex-heading-article.def:490-528 `\__ctex_titlesec_spacing:nnnnnn` takes its five groups
plus the name); with nothing there the six-argument grab ran away across the headings map
into `\csname CTEX@…` (mynsfc, qyxf-book, bjfuthesis; TeX Live class census 2026-09-24).
ctex-heading-book.def:669 also resets `\ttl@chapterout` (titlesec.sty:402). Trigger:
`\usepackage{titlesec}` then `\@ifundefined{ttls@section}{MISSING}{}` (Perl 0.8.8 and Rust
MISSING, pdflatex defined). **Rust (FIXED, batch 56ia):** the binding defines the five
records with titlesec's fallback for a command not built on `\@startsection` (:1587-1590
`\titlespacing*#1{\z@}{*3}{*2}`), and `\ttl@chapterout`. Guard
`class_census::titlesec_spacing_record`.

## 240. `\tableofcontents` holds no `\@starttoc{toc}` for etoolbox to patch (FIXED in Rust)

book.cls/article.cls `\tableofcontents` typeset the list with `\@starttoc{toc}`
(`\listoffigures`/`\listoftables` with `{lof}`/`{lot}`), and classes patch that call:
exam-zh.cls:272-278 `\patchcmd{\tableofcontents}{\@starttoc{toc}}{…}{}{\fail}`, whose
failure branch runs the undefined `\fail`. Perl's `\tableofcontents` is a constructor
(latex_constructs.pool.ltxml:724), so `\patchcmd` fails outright; Rust's delegating macro
held no `\@starttoc{toc}` either. **Rust (FIXED, batch 56ia):** the body carries the call as
the argument of `\lx@kernel@listbody`, which discards it (the ToC is built in
post-processing, and our `\@starttoc` would read a stale `\jobname.toc`). Guard
`class_census::patchcmd_tableofcontents_starttoc`.

## 241. listings resolves `rulecolor`/`backgroundcolor` at `\lstset` time (FIXED in Rust)

listings stores the colour command of `rulecolor`/`backgroundcolor` and runs it when a
listing is drawn. Perl's `lstExtractColor` (listings.sty.ltxml:945-953) digests it when the
key is set, so a colour defined later in the preamble errs: easybase.sty:2419
`rulecolor = \color{ctex@frame}` under the `\lstset{style=…}` at :2431, `ctex@frame`
defined at :2439 (easybook). Trigger: `\lstset{frame=single, rulecolor=\color{later}}`
then `\definecolor{later}{HTML}{C00000}` (Perl and Rust: "color 'later' is undefined",
pdflatex clean). **Rust (FIXED, batch 56ia):** the keys keep the tokens and
`lst_extract_color` resolves them at the listing. Guard
`class_census::listings_color_resolved_at_listing`.

## 242. The beamer binding does not load scrlfile (FIXED in Rust)

Real beamer loads scrlfile through its font setup: beamerbasefont.sty:307-308 requires
sansmathaccent, which in a beamer document requires scrlfile (sansmathaccent.sty:49-56). Perl's
beamer binding comments the sansmathaccent load out (beamer.cls.ltxml:1315), so KOMA's
`\AfterPackage`/`\BeforePackage` are undefined in a beamer document, and their arguments spill
into the text. univie-ling-poster.cls:822/835 `\AfterPackage*{csquotes}{\SetCiteCommand{\parencite}}`
(TeX Live class census 2026-09-24). Trigger: `\documentclass{beamer}\AfterPackage*{csquotes}{…}`
(Perl and Rust: `\AfterPackage` undefined; pdflatex clean). **Rust (FIXED, batch 56ib):** the
binding requires scrlfile. Guard `class_census::beamer_loads_scrlfile`.

## 243. expl3's l3text case changers loop on accented input (FIXED in Rust)

The LaTeX format's l3text is built for pdfTeX: `\__text_codepoint_process:nN`
(expl3-code.tex:35892-35925) reads a character above `"80` as a UTF-8 lead byte and takes the
next one to three tokens as continuation bytes, and the output side re-encodes results as bytes.
LaTeXML's tokens are whole code points (the Unicode engines' model), so `\text_lowercase:n {É}`
consumes the recursion quarks as "bytes": `\q__text_recursion_tail` expands into itself and
`Until:\q__text_recursion_stop` runs out. Perl reports the errors and recovers; Rust ran away,
and letgut (letgut.cls:1711 lowercases acronym keys; `INSPÉ` in letgut-acronyms.tex) never
finished (TeX Live class census 2026-09-24). Trigger:
`\ExplSyntaxOn\tl_set:Nx\l_tmpa_tl{\text_lowercase:n{É}}` (pdflatex `é`); a medial accent leaves
the letters after it uncased (`\text_uppercase:n{élan}` → `élaN`). **Rust (FIXED, batch 56if):**
expl3's own Unicode-engine codepoint layer (the `\sys_if_engine_opentype:TF` true branches) is
installed over the format's 8-bit one after the dump (latex_constructs_rust_only.rs):
`\c_max_char_int` = `"10FFFF` (:7376; `\char_generate:nn` then builds any code point through
the dump's `\tex_Ucharcat:D` branch), `\codepoint_generate:nn` (:34994-35002),
`\__text_codepoint_process:nN`, `\__text_codepoint_compare:nNn`,
`\__text_codepoint_from_chars:Nw` (:35894, :35931-35938), `\__text_change_case_catcode:nn`
(:36929) and `\__codepoint_to_nfd:n` (:35171). `\codepoint_str_generate:n` stays 8-bit: it only
keys the SpecialCasing tables, built with it when the format was made (`ß` → `SS`). One
adaptation: the final-sigma test (:36829-36840) counts a catcode-12 character above `"80` as a
letter, as the pdfTeX branch counts an active one (our letters are neither catcode 11 nor
active), so `\text_lowercase:n{ΣΑΣ}` is `σας` as in lualatex. 56ic had routed the changers to
the native `\MakeUppercase` mapping instead; that broke mfirstuc's `\capitalisewords` over an
arrayjob array (ftc-notebook's example-notebook.tex timed out, sweep #121) and missed text held
in token lists. Titlecasing (`\MakeTitlecase` is `\text_titlecase_first:n` in the kernel) leaves
the rest of the text as it is, as l3text does: Perl lowercases it
(latex_constructs.pool.ltxml:5507-5523; `\MakeTitlecase{hELLO wORLD}` is `Hello world` there,
`HELLO wORLD` in pdflatex). Guards `class_census::{l3text_case_change_accented,
l3text_codepoint_layer_token_lists, mfirstuc_capitalisewords_arrayjob}`.

## 244. `\pdfcreationdate` is empty (FIXED in Rust)

pdfTeX's `\pdfcreationdate` (LuaTeX: `\pdffeedback creationdate`, which luatex85.sty:62
`\xdef`s into `\pdfcreationdate`) is the PDF date string `D:YYYYMMDDhhmmss…` in catcode-12
characters. Perl defines it empty (pdfTeX.pool.ltxml:94), so pdfx-style parsers delimited by
`D:` fail: novel-pdfx.sty:293-323 `\pdfx@getYear D:#1#2#3#4` (Perl: "Missing argument
Match:D:", 28 errors and a Fatal; Rust's luatex profile answered `0` and ran away to a
PushbackLimit Fatal; TeX Live class census 2026-09-24). **Rust (FIXED, batches 56hz/56ic):**
both return the job's date as catcode-12 characters (`pdftex::pdf_creation_date`). Guards
`class_census::{pdfcreationdate_is_other_catcode, luatex_pdffeedback_creationdate}`.

## 245. Box code in an algorithm2e listing line closes its capture block twice (MITIGATED in Rust)

A document that redefines algorithm2e's block macros with box drawing — arXiv 2605.20533
`\renewcommand{\algocf@Vsline}[1]{\algocf@bblockcode\hbox{\vrule\vtop{#1}}\algocf@eblockcode}`
for algorithm2e releases before 2017/07/19 — has that `\hbox`/`\vtop` digested inside the
`ltx:listingline` the binding is building; the box's capture block is closed by the nested box
and then closed again ("Attempt to close ltx:_CaptureBlock_, which isn't open", 8× in the
witness). Perl errs the same way when the renewal runs, but its always-true `\@ifpackagelater`
(KPE #238) skips the renewal; Rust takes it since 56ia. Trigger: the repro
`captions-floats/algorithm2e_block_macros_locked.tex`. **Rust (MITIGATED, batch 56ie):** the
binding's `\algocf@Vline`/`\algocf@Vsline`/`\algocf@Noline`, the listing's block structure, are
locked, so the restyling is ignored; the capture bug itself is open. Guard
`class_census::algorithm2e_block_macros_locked`.

## 246. `\pdfliteral` matches its spec keyword unexpanded (FIXED in Rust)

pdfTeX's `\pdfliteral [direct|page] <general text>` scans the keyword with `scan_keyword`,
which expands. Perl's binding (pdfTeX.pool.ltxml:199) matches it with `OptionalMatch`, which does
not, so a keyword behind a macro is missed and the general text's `{` is not found. accsupp's
pdftex driver writes `\pdfliteral\ACCSUPP@pdfliteral{\ACCSUPP@span BDC}` with `page` stored in
`\ACCSUPP@pdfliteral` (accsupp.sty:193). Perl never reaches it (its `\ifpdf` is false, so accsupp
takes its dvips driver); Rust does since PDF output became the default (56id): arXiv 2605.21262
gave 480 "Expected opening '{'" errors, 2605.04642 and 2605.10085 a few. Trigger:
`\def\mm{page}\pdfliteral\mm{x}`. **Rust (FIXED, batch 56if):** the keyword is read with
`read_keyword` (expanding). Guard `class_census::pdfliteral_keyword_expands`.


## 247. `\pdfobj` object spec: `useobjnum` and `stream` taken as alternatives, `file` unknown (FIXED in Rust)

pdfTeX's object type spec is `reserveobjnum | [useobjnum <n>] [stream [attr <text>]] <object contents>`,
with `<object contents>` = `file <text>` or `<text>` (pdftex manual, pdftex.tex:4224-4246). Perl's
`OpenAnnotSpecification` reader (pdfTeX.pool.ltxml:156-171) reads `useobjnum` and `stream` in one
`if/elsif` chain and never reads `file`, so `\pdfobj useobjnum 1 stream attr {/N 3} file {x.icc}`
meets `stream` where it wants `{`. l3backend-pdftex.def:274-302 writes exactly that for
pdfmanagement's PDF/A colour profile; the tuda-ci DEMOs (pdfstandard=a-2b) hit it once PDF output
became the default (56id, sweep #121). Trigger:
`\RequirePackage{pdfmanagement}\SetKeys[document/metadata]{pdfstandard=a-2b}\documentclass{article}\begin{document}Hello\end{document}`
(pdflatex: 0 errors). **Rust (FIXED, batch 56ig):** the keywords are read in sequence and `file` is
consumed (pdftex.rs `OpenAnnotSpecification`). Guard `class_census::pdfobj_stream_file_spec`.

## 248. epstopdf does not load grfext (FIXED in Rust)

epstopdf.sty:150 `\RequirePackage{grfext}`, whose `\AppendGraphicsExtensions`,
`\PrependGraphicsExtensions` and `\RemoveGraphicsExtensions` (grfext.sty:153-237) edit
`\Gin@extensions`. Perl's binding (epstopdf.sty.ltxml:19) is a stub that loads nothing, so a document
calling them after `\usepackage{epstopdf}` meets undefined commands. arXiv 2606.05709 does it inside
`\ifpdf`, which runs since PDF output became the default (56id). Trigger:
`\usepackage{graphicx}\usepackage{epstopdf}\PrependGraphicsExtensions{.svg}` (pdflatex: 0 errors).
**Rust (FIXED, batch 56ij):** the binding loads the real grfext raw; the graphics lookup does not read
the list. Guard `class_census::epstopdf_loads_grfext`.

## 249. underscore's active `_` breaks file names (FIXED in Rust)

The LaTeX kernel replaces underscore.sty's active `_` with its first aid
(latex2e-first-aid-for-external-files.ltx:176-177 loads underscore-ltx.sty): `\protected`, and a
literal `_` inside a `\csname` (underscore-ltx.sty:38-51). `\input` expands its file name within a
`\csname` (`\set@curr@file`), so `\input{sections/logic_T}` reads that file. Perl's binding
(underscore.sty.ltxml:20) defines `_` as `\ifmmode\sb\else\textunderscore\fi`, and the name becomes
`sections/logic\textunderscoreT`: missing file (arXiv 2606.31852, whose `\usepackage{underscore}` sits
in `\ifpdf`). **Rust (FIXED, batch 56ij):** an `underscore-ltx` binding (latexml_contrib) carries the
first-aid `_`. The kernel's own `file/underscore.sty/after` hook loads it: the format dump holds that
hook, since it is made from raw latex.ltx, first-aid file included; the degraded no-dump branch has no
hook and keeps Perl's `_`. `\input`, `\include`, `\IfFileExists` and `\InputIfFileExists` expand
the name with `\ifincsname` true (`gullet::expand_as_csname_text`). Guard
`class_census::underscore_first_aid_file_name`.

## 250. fontenc leaves the last mapped encoding in force (FIXED in Rust)

fontenc.sty:116 ends with `\usefont\encodingdefault\familydefault\seriesdefault\shapedefault`: the
encoding in force is `\encodingdefault`, i.e. the last option, whatever it is. Perl's binding
(fontenc.sty.ltxml:90-99) merges an encoding into the font only when it has a font map and never
does the final `\usefont`, so `\usepackage[LGR,TU]{fontenc}` leaves LGR active: every Latin letter of
the body comes out as its LGR Greek letter (greek-fontenc's test-tuenc-greek, recall 25 %). Trigger:
`\usepackage[LGR,TU]{fontenc}` + `Hello` (pdflatex: "Hello"; under pdfTeX tuenc.def turns the default
to T1). With `[T1,LGR]` pdflatex itself prints Greek. **Rust (FIXED, batch 56ij):** after the options,
the font takes `\encodingdefault` (fontenc_sty.rs). Guard `class_census::fontenc_last_encoding_in_force`.

## 251. `\verb` never runs `\verb@egroup` (FIXED in Rust)

LaTeX's closing `\verb` delimiter runs `\verb@egroup` (latex.ltx:15492, 15501), the hook packages use
to close what their `\verb` wrapper opened: accessibility.sty:1566-1577 begins `PDFInlineObjInText` in
`\verb` and ends it in `\verb@egroup`; newverbs appends its `\egroup` there (newverbs.sty:58,64). Perl's
`\verb` (latex_constructs.pool.ltxml:1797-1829) reads the body itself and closes its hidden group
directly, so such a group leaks. arXiv 2606.00334 (accessibility, activated by PDF output) gave 35
"Attempt to close boxing group" errors for `\verb` in tabular cells. **Rust (FIXED, batch 56ij):** the
verb group closes through `\verb@egroup`, by default `\lx@hidden@egroup` (sect06.rs). Guard
`class_census::verb_egroup_closes_package_wrappers`.

## 252. natbib expands a bare `\bibitem` label before splitting it (FIXED in Rust)

natbib splits a bare label at its literal `(year)` without expanding it: `\@lbibitem` →
`\NAT@ifcmd` → `\NAT@bare#1(#2)#3(@)#4\@nil#5` (natbib.sty:809-818, 827), a delimited-parameter
match. Perl's binding runs `Expand($label)` first (natbib.sty.ltxml:564). That expansion descends into
whatever the label's commands are made of. A text command becomes its typesetting code:
`\textcommabelow` (latex.ltx:10097-10100) turns into `\ooalign{…\hbox{…\selectfont,}}`, and the author
split then cuts through that `,`. Trigger: `\usepackage{natbib}` +
`\bibitem[{Mari\textcommabelow{s} et~al.(2020)Mari\textcommabelow{s}, Li, and Wu}]{k1} Body.`
Perl: 5 errors (`readBalanced ran out of input`, missing `\NAT@wrout` arguments) and an author of
`MariΩ`; pdflatex: clean. Rust hit a PushbackLimit Fatal on the same input (arXiv 2605.08338 via
`Mari{\textcommabelow s}`, 2605.21804 via `M\u{a}lina\textcommabelow{s}`).
**Rust (FIXED, batch 56il):** `\lx@NAT@parselabel` splits the unexpanded label, as `\NAT@bare`
does. This replaces two lists of commands that the binding had already been exempting from the
expansion: `\cite`/`\href`/`\bibinfo` (2404.06289) and the T1 text symbols `\i`, `\ss`, …
(2111.00584). Guard `perfect_kernel_batch56::fontenc_keeps_preloaded_encoding`.

## 253. `\afterassignment` is lost when `\setbox` takes `\box`/`\copy` (FIXED in Rust)

TeX inserts the `\afterassignment` token right after the next assignment (tex.web §1211 `done:`,
§1269). For `\setbox<n>=\hbox{…}` that is inside the box, after its `{`. For
`\setbox<n>=\box<m>`, `\copy`, `\lastbox` or `\vsplit` there is no body, so the token runs
immediately after the assignment. Perl's `\setbox` (TeX_Box.pool.ltxml:599-617) parks the token for
the next box body (`BeforeNextBox`, consumed at :171-172). With a `\box` operand nothing consumes
it, so it fires inside some later, unrelated box. luatexja's `\raise`/`\lower`/`\moveleft`/
`\moveright` (luatexja-core.sty:684-702, ltj-base.sty:232-237 `\ltj@afterbox`) take exactly this
path. So every pgf/TikZ/tcolorbox picture under luatexja came out as an empty `<svg:g/>`, at 0
errors.
```latex
\newbox\afb
\def\grab{\ifnum\lvl<\currentgrouplevel\expandafter\aftergroup\fi\place}
\def\place{\raise2pt\box\afb}
\def\lraise{\edef\lvl{\number\currentgrouplevel}\afterassignment\grab\setbox\afb}
\setbox2\hbox{Echo words}\setbox2\hbox{\lraise\box2}\box2
```
pdflatex prints "Echo words" raised by 2pt; Perl and Rust before the fix print nothing.
**Rust (FIXED, batch 56ir):** `\setbox` unreads a still-pending token after the assignment.
Witnesses: suanpan-l3 (2,342 empty pictures), qworld, kksymbols, codebox-doc-en, qyxf-book.
Guard `perfect_kernel_batch56::afterassignment_setbox_box_operand`.

## 254. `\setbox<n> = <box>` with a space after `=` loses the box (FIXED in Rust)

`scan_box` reads the next non-blank non-relax non-call token (tex.web §1084, §404), so
`\setbox0 = \hbox{xx}` stores the box. Perl's `\setbox` (TeX_Box.pool.ltxml:599-617) reads the very
next token as the operand. The space is taken instead, and the `\hbox` is typeset in place:
`\setbox0 = \hbox{xx}[\box0]` prints `xx[]` in Perl and Rust before the fix, and `[xx]` in pdflatex.
19 files under TL `tex/latex` and `tex/generic` spell `\setbox… = \hbox`.
**Rust (FIXED, batch 56ir):** `\setbox` skips spaces and `\relax` before its box operand, as the
`Variable` reader already does. Guard `perfect_kernel_batch56::setbox_skips_blanks_before_the_box`.

## 255. A `\noexpand`'d token keeps its no-expand marker after a scanner puts it back (FIXED in Rust)

`\noexpand` suppresses expansion only for the read that meets it (tex.web:7509-7514). get_x_token
leaves the plain control sequence in `cur_tok` (:7837-7838), and a scanner that reads one token too
many, such as the optional space after a number (:8755-8757), puts that plain token back. When it is
read again, it expands. Perl and Rust put back the marked token instead (Rust `\special_relax`
family), so a later `\csname` met it:
```latex
\def\foo{FOO}\expandafter\def\csname a\romannumeral-`\q\noexpand\foo b\endcsname{Y}
```
Rust errored `unexpected:\special_relax\foo`; pdflatex defines `\aFOOb`. trimspaces' `\trim@spaces`
on an argument that starts with a macro takes exactly this path. yquant-doc's 501-error runaway went
through it.
**Rust (FIXED, batch 56is):** gullet.rs `unread_scanned` puts back the shadowed plain token from
`skip_one_space` (expanded) and the digit scanner; `read_keyword` backs up its read tokens plain, as
scan_keyword does (tex.web §407); and `read_factor` tests the token that ended its digits for the
point and puts it back once, as scan_dimen tests `cur_tok` (§448, §452). `read_float`, which is not
a TeX scanner, re-reads the terminator with expansion, so a `\noexpand`'d terminator now expands
there. The sign scanner keeps the marker: TeX's scan_int examines `cur_tok` right after the signs
without re-reading it (§440, §444), so there it still acts as `\relax`. Guard
`perfect_kernel_batch56::noexpand_marker_does_not_survive_a_scan`.

## 256. A braced file name keeps scanning past its `}` (FIXED in Rust)

TeX Live reads `\input{name}` / `\tex_input:D {name}` as a braced file name: the expanded group up to
its matching `}`, spaces included. Perl's `TeXFileName` (Base_ParameterTypes.pool.ltxml:296-307)
reads on past the `}` until a space or control sequence, expanding the next macro before the file is
read:
```latex
\begin{filecontents*}{tfnini.tex}
\def\tfnloaded{LOADED}
\end{filecontents*}
\makeatletter\@@input{tfnini.tex}\tfnloaded
```
Perl and Rust before the fix: `undefined:\tfnloaded`. pdflatex prints LOADED. The witness is
texnegar-luatex.sty:17 `\tex_input:D { texnegar-ini.tex }` followed by `\bool_if:NT …`.
**Rust (FIXED, batch 56is):** base_parameter_types.rs `TeXFileName` reads a braced name to its
matching brace and stops. The braces stay on the name, so `\input` still strips them and
auto-loads LaTeX.pool (TeX_FileIO.pool.ltxml:164-169). The group expands as `\edef` does, and
spaces inside it are kept. Guards `perfect_kernel_batch56::braced_file_name_stops_at_its_brace` and
`braced_input_of_a_fragment_loads_latex`.

## 257. `\advance`, `\multiply` and `\divide` do not fire `\afterassignment` (FIXED in Rust)

An arithmetic assignment is a prefixed command, so TeX inserts the `\afterassignment` token right
after it (tex.web §1211 `done:`, §1269; §1236 do_register_command). Perl's primitives in
TeX_Registers.pool.ltxml set the value and never fire it; Rust kept it pending until a later
assignment:
```latex
\def\f{F}\newdimen\x [\afterassignment\f\advance\x 1pt \the\x]
```
pdflatex prints `[F1.0pt]`; Perl and Rust printed `[1.0pt]`. pstricks' raw `\psaddtolength` depends on
it (lsc).
**Rust (FIXED, batch 56is):** tex_registers.rs `\advance`/`\multiply`/`\divide` call
`after_assignment()`. Guard `perfect_kernel_batch56::afterassignment_fires_after_arithmetic`.

## 258. Three bindings miss a definition their raw package makes (FIXED in Rust)

Each of these errors in Perl and in Rust before batch 56iv. pdflatex is clean on all three.
- **pdfpages.** Perl's `\includepdf` reads its file name as `{}` (pdfpages.sty.ltxml:30), so
  `\includepdf{a_b.pdf}` tokenizes `_` as a subscript: "Script _" outside math. Rust reads it as
  `Semiverbatim`, as Perl's own `\includegraphics` does (graphicx.sty.ltxml:52). Witness: latex4wp.
- **listings.** Perl defines `\thelstnumber` only (listings.sty.ltxml:879-880), with no
  `\theHlstnumber`, so hyperref's anchor for a listing line is undefined (SASnRdisplay). Rust defines
  the uncaptioned branch of lstmisc.sty:1236-1239.
- **varioref.** Perl defines no `\if@vrefhandlespace` (varioref.sty.ltxml:24-44), which varioref.sty
  :799-802 tests (tikz-cookingsymbols-doc).

**Rust (FIXED, batch 56iv):** guards
`binding_singletons_56::{includepdf_file_name_is_semiverbatim, listings_the_h_lstnumber_is_defined,
varioref_handlespace_switch_exists}`.

## 259. `\everyeof` is never inserted, so `\everyeof{\noexpand}` cannot carry a scan past a file's end (FIXED in Rust)

tex.web §367 reads `\noexpand`'s token under normal scanner status, so the end of a file or
`\scantokens` pseudo-file it meets closes without a runaway (§362, §336) and the token comes from the
enclosing input. `\everyeof{\noexpand}` relies on it to let an `\edef`, `\message` or x-expansion
take a whole file:
```latex
\everyeof{\noexpand}\edef\x{\@@input f }   % f.tex: A\foo B
```
pdflatex defines `\x` as `AFOOB ` (0 errors). Perl never inserts the `\everyeof` tokens at all
(eTeX.pool.ltxml:256-258, "These tokens are NOT used anywhere (yet?)"), and its `readBalanced` stops
at the mouth's end (Gullet.pm:470-472): 2 errors, "readBalanced ran out of input" and a stray `}`.
Witnesses: catchfile.sty:251-261 `\CatchFileEdef` (makron, arXiv 1611.01359), morewrites.sty:465,
l3build regression-test.tex:101.
**Rust (FIXED, batch 56ix):** Rust did insert `\everyeof`, but its `\noexpand` returned nothing at the
spent input. gullet.rs `read_token_across_input_ends` (closes autoclose mouths with a parent) is now
what `\noexpand` reads with; `read_balanced`, `read_until` and `read_token` are unchanged, so a
definition that runs off a file's end is still the §338 runaway. Two Rust-only defects of the same
mechanism went with it: over `\scantokens` the next token was expanded
(`\xdef\z{\scantokens{abc}\bar}` gave `abcBAR`, pdflatex `abc\bar`), and `\endinput` inserted
`\everyeof` (eTeX's `force_eof` does not, §362). Guards `noexpand_input_ends::*`.

## 260. `\textnormal` in math sets `\f@family` to `cmtt` (FIXED in Rust)

latex_constructs.pool.ltxml:5267 defines `\textnormal@math` with `\f@family` = `cmtt`, contradicting
the serif font it selects:
```latex
$\textnormal{\selectfont x}$
```
Perl gives `<text class="ltx_markedasmath" font="typewriter">x</text>`; pdflatex prints x in cmr10.
Rust (batch 56je) sets `cmr` (sect13.rs `\textnormal@math`). Guard `nfss_font_state::size_switch_reselects_named_families`.

## 261. `\input` of a raw definitions file already read is skipped (FIXED in Rust)

While reading definitions, Perl routes `\input` to InputDefinitions (Package.pm:2287-2288), which
returns for a file already `_loaded` (:2363); TeX re-reads. Trigger: two packages each
`\input{x.def}` — Perl reads it once. greek-fontenc's tuenc-greek.def:126 re-inputs
greek-fontenc.def for TU after lgrenc.def read it for LGR, so the TU Greek declarations never exist
and babel-greek falls back to LGR (a plain `U` prints ϒ). Rust (batch 56ji, OXIDIZED_DESIGN_DIVERGENCES #312).
Guard `greek_text::input_rereads_a_raw_definitions_file`.

## 262. A primitive's token read at the end of an `\input` file finds nothing (FIXED in Rust)

`Gullet.pm` `readToken` returns undef at the mouth's end instead of crossing into the enclosing
input (tex.web §362: at `scanner_status=normal` every `get_token` does). A file whose last token is
`\expandafter`, `\string` or `\meaning` is a Fatal ("Can't call method 'defined_as' / 'getCatcode'
on an undefined value"); `\afterassignment`, `\aftergroup`, `\let`, `\futurelet` there are "Missing
argument Token". Trigger: `x.tex` = `\expandafter\x`, main `\def\x#1{[#1]}\def\foo{FOO}` and
`\@@input x \foo` — pdflatex prints "[F]OO" (LaTeX's `\input{x}` would hand `\expandafter` its own
file hooks instead). Repro `expansion-primitives/input_end_expandafter.tex`. Rust (batch 56jj, OXIDIZED_DESIGN_DIVERGENCES #313).
Guards `token_kernel_gaps::*_crosses_a_file_end`.

## 263. `\let` skips only an explicit space after `=` (FIXED in Rust)

Perl's `\let` reads `SkipSpaces Token SkipSpaces SkipMatch:= Skip1Space Token`
(TeX_Macro.pool.ltxml:184); tex.web §1221 skips every token whose command is a spacer, which
includes an implicit space. Trigger: `\let\c\@sptoken=\b` — Perl and pre-56jj Rust give `\c` = `=`,
TeX gives `\c` = `\b`. Rust (batch 56jj, #313). Guard `token_kernel_gaps::let_crosses_a_file_end`.

## 264. A token-list assignment takes no implicit left brace (FIXED in Rust)

`readTokensValue` (Gullet.pm:824-842) accepts only an explicit catcode-1 token or a token register;
tex.web §1226-1227 scans for a left brace by expanding, so `\bgroup` opens the list. Trigger:
`\toks0=\bgroup abc}` — Perl stores `\bgroup` as the value and typesets `abc}`. Witnesses 2605.02221,
2605.25087 (`diagrams.sty:15`, `\toks0=\bgroup}`). Rust (batch 56jj, #313). Repro
`expansion-primitives/toks_implicit_left_brace.tex`; guard
`token_kernel_gaps::a_token_list_assignment_takes_an_implicit_brace`.

## 265. `\message`, `\errmessage` and `\mark` read a `{}` argument, not `scan_toks` (FIXED in Rust)

Perl declares them with `{}` (TeX_Debugging.pool.ltxml:65,71, TeX_Marks.pool.ltxml:30), so the
brace is not found by expansion, `\message` expands its text a second time, and `\mark` does not
expand at all (an undefined macro in a mark is silent). Trigger:
`\def\foo{hi}\message\expandafter{\foo}` — TeX logs `hi`; Perl reads `\expandafter` as the argument
and typesets `hi`. Rust reads `XGeneralText` (batch 56jj, #313); an unbraced text is an error
with the token put back (Perl takes it silently as the text). Repros `expansion-primitives/message_*.tex`, `mark_unbraced.tex`.

## 266. `\shipout` is not a primitive: LaTeX's `\shipout` errors, or loses a register's box silently (FIXED in Rust)

TeX_FileIO.pool.ltxml:257 lists `\shipout` as a placeholder and defines nothing. LaTeX's
`\shipout` (latex.ltx:19911-19917) sets `\l_shipout_box` with an `\afterassignment` whose chain
ends in `\tex_shipout:D \box_use:N \l_shipout_box` (latex.ltx:19986). With a braced operand the
`\afterassignment` fires (Perl parks it for the box body, TeX_Box.pool.ltxml:170-172), and
`\shipout\vbox{Cover page}` is `Error:undefined:\tex_shipout:D` followed by the box in place. With a
register operand #253 keeps it from firing, so the box disappears without a diagnostic. Trigger:
`\setbox255\vbox{Cover text}\shipout\box255 Body text.` — pdflatex ships two pages, Perl's XML has only
"Body text.". Rust emits the box in place (batch 56jk, OXIDIZED_DESIGN_DIVERGENCES #314). Repro
`boxes-groups/shipout_box_register_simplesample.tex`; guard
`shipout_parskip::shipout_emits_the_box_in_place`.

## 267. The kernel's peeking macros peek while a number scan expands them (FIXED in Rust)

latex.ltx's `\@ifnextchar` (1756-1760; `\kernel@ifnextchar`), `\@ifstar` (:1775), `\@testopt`
(:1259-1260) and every `\newcommand` optional argument (`\@protected@testopt`, :1249, :1261) open
with an unexpandable `\let` and peek by `\futurelet`, so a number scan ends at them (tex.web §445)
and the peek happens when they are executed. Perl makes them closures that read the next token as
they expand (latex_constructs.pool.ltxml:5636-5642 `\@ifnextchar`, 5646-5655 `\@ifnext@n`,
5657-5664 `\@ifstar`; Package.pm:240-251 `convertLaTeXArgs` Optional), so a scan's look-ahead runs
them and reads the chosen branch into the number, or its conditionals into a false branch's skip.
Triggers: `\count@=1\@ifstar{7}{5}*\the\count@.` — pdflatex "71.", Perl "17."-shaped; `\newcommand
\foo[1][7]{#1}` + `\cc=1\foo \the\cc.` — pdflatex "71.", Perl "."; `\ifodd2\x x\else y\fi z` with
`\def\x{\@ifnextchar*{\footrue}{\foofalse}}` — pdflatex "yz", Perl 2 errors. Witness egpeirce-doc
(egpeirce.sty:184-189 `\ifodd\the\value{cutdepth}%` + `\psset` → `\XKV@ifstar` exposed
`\let\ifXKV@st\iffalse`; the skip counted it as a nested `\if` and ran off the document: 51 errors,
recall 3.6% → 2 errors, 96.2%). Rust (batch 56jm): such definitions are flagged
`peeks_by_futurelet` (`Parameter::testopt` for the `\newcommand` family, via
`convert_latex_args`/`convert_twoopt_args`) and stay unexpanded in a number scan
(`gullet::scan_stops_at_futurelet_peek`, also on `\expandafter`'s one-level expansion); digestion a
binding runs in mid-scan is outside the scan (`NumberScan::suspend`: `\widthof{…}` inside
`\makebox[…]`). Guards `ifnextchar_scans::*` (11).

Also Perl's `\@testopt` (:5667-5670) passes the default unbraced: `\def\oo[#1]{<#1>}\@testopt\oo{a]b}`
gives Perl "<a>b]", pdflatex "<a]b>" (latex.ltx:1260 `#1[{#2}]`); Rust braces it (56jm).

Not TeX-faithful yet (Perl alike): outside number scans the closures still peek as they expand
(`\edef`, `\csname`, `\if` operands, the alignment row-head peek that LaTeXML's `\rowcolor` and
`\cmidrule` rely on); binding `DefMacro`s with a leading `[]` and xparse `o` arguments are not
flagged; `\@testopt` does not skip a space before `[` (`\@testopt\oo {z} [w]`: Rust "<z> [w]",
pdflatex "<w>").

## 268. A definition's name may be any token: `\def{` defines the Begin catcode (FIXED in Rust)

TeX_Macro.pool.ltxml:174-177 (`\def SkipSpaces Token UntilBrace …`), :184 (`\let`) and :189
(`\futurelet`) read the name as a plain `Token`. tex.web §1215 `get_r_token` accepts only a control
sequence or an active character; anything else is "Missing control sequence inserted", the token is
read again and the frozen `\inaccessible` is defined instead. Trigger: `\gdef{}X{Y} Before {a} and
\textbf{b} after.` — pdflatex 1 error, "XY Before a and b after."; Perl 4 errors, "Before Ya and Y"
with the bold running to the paragraph's end, because every later `{` is the new macro. A
`\noexpand`-marked name (`\expandafter\def\noexpand\foo{X}`, §358 `cur_cs`) defines Perl's
`\special_relax` instead of `\foo` (pdflatex "X"; Perl 2 errors). Witness yquant-doc.tex:3832.
Rust (batch 56jp, worker W17; OXIDIZED_DESIGN_DIVERGENCES #313): `gullet.rs` `read_redefinable_token`. Repros
`expansion-primitives/def_noncs_target_yquant.tex`, `let_family_noncs_target.tex`,
`noexpand_definition_name.tex`; guards `rtoken_patchcmd::*`.

## 269. The NFSS font switches are plain macros, so `\protected@edef` expands them (FIXED in Rust)

latex.ltx declares `\fontencoding` (:10490), `\fontfamily` (:14139), `\fontseries` (:12259),
`\fontshape` (:12436), `\usefont` (:10518), `\rmfamily`/`\sffamily`/`\ttfamily` (:13983-13993),
`\mdseries` (:13948), `\bfseries` (:13923), `\upshape`/`\slshape`/`\scshape`/`\itshape`
(:13794-13803) and `\normalfont` (:14113) with `\DeclareRobustCommand`; at `\begin{document}`
`\reinstall@nfss@defs` (:12489-12514) makes the shape switches `\protected` macros, so in the body
even a plain `\edef` keeps them (`\edef\x{\itshape}` → `macro:->\itshape `). latex_constructs.pool.ltxml:
2637 and 5170-5194 define plain macros, so inside `\protected@edef` (or an `\index` entry)
`\ttfamily` expands to `\edef\f@family{\ttdefault}` and `\f@family` expands in turn: the stored
text is `\edef cmr{cmtt}\selectfont`, a definition of the letter c, and the font is lost. Trigger:
`\makeatletter\protected@edef\x{\ttfamily B}\makeatother … A {\x} C \texttt{D}` — pdflatex B in
typewriter; Perl 0 errors, B upright. Also titlecaps' `\titlecap{\ttfamily …}` ("mtt\" junk). Rust
(batch 56jp, worker W17; OXIDIZED_DESIGN_DIVERGENCES #315): `robust => true` in sect13.rs/sect08.rs, and
`\reinstall@nfss@defs` run at `\begin{document}`. Repros
`fonts-nfss/protected_edef_ttfamily_pythonimmediate.tex`, `index/index_ttfamily_guitar.tex`,
`fonts-nfss/body_shape_switches_protected.tex`.

## 270. `\patchcmd` drops the macro's `\protected\long\outer` prefix (FIXED in Rust)

etoolbox.sty:1358-1371 rebuilds the patched macro from its `\meaning`, keeping its prefix; the
optional `[<prefix>]` replaces it (`[]` strips it). etoolbox.sty.ltxml:1310 installs
`Expandable->new($cs, $params, $string)`, a plain macro, and ignores `[<prefix>]`. Trigger:
`\protected\def\clip#1{\undefinedinside{#1}}\let\clipvert=\clip
\patchcmd\clipvert{\undefinedinside}{\alsoundefined}{}{}` then `\edef\cmd{\clipvert{a}}` — pdflatex 0
errors (`\clipvert` stays unexpanded); Perl 1 error (`\alsoundefined` undefined), `\ifdefprotected`
false. Witness yquant-doc (yquant-config.tex:594-596 patches a copy of yquant-shapes.tex:26's
`\protected\def\pgfshapeclippath`, used in yquant-draw.tex:201-211's `\edef`): 1001 errors and a
Fatal. Rust (batch 56jp, worker W17): `etoolbox_sty.rs` `\patchcmd`. Repros
`macro-state/patchcmd_keeps_protected_yquant.tex`, `patchcmd_prefix_option.tex`.

## 271. `\bezier` draws no stroke, so its curve is invisible in a `{picture}` (FIXED in Rust)

Perl's `\lx@pic@bezier` template (latex_constructs.pool.ltxml:5034-5038) writes `points` and
`stroke-width` but no `stroke`, where `\qbezier`'s (:5027-5031) writes `stroke='#color'`; the
`{picture}` element carries `stroke='none'`, which the curve inherits. Trigger:
`\begin{picture}(100,50)\bezier{20}(0,0)(50,50)(100,0)\end{picture}` — pdflatex draws 20 dots
along the curve; Perl's XML is `<bezier displayedpoints="20" points="0,0 50,50 100,0"
stroke-width="0.4"/>` inside `<picture … stroke="none">`, and its SVG path has no stroke. Rust
(W12) makes `\lx@pic@bezier` Perl's constructor with `\qbezier`'s stroked template
(OXIDIZED_DESIGN_DIVERGENCES #316); guard `node_box_append::bezier_is_stroked_and_counts_its_points`.

## 272. A removal whose content lives on drops its box from the enclosing boxes: an aligned cell's `tex` loses digits, `\mbox`, `\raisebox`, `\hbox` (FIXED in Rust)

`applyMathLigature` (Core/Document.pm:1186-1204) merges `2`, `.`, `414` into one `XMTok` and
`removeNode`s the others; `removeNode` runs `removeNodeBox` on each token's parent (:1788-1789),
which drops the token's box from the parent's box list and from each auto-opened ancestor's.
Perl's own comment (:1198-1200) notes the parent lists go out of sync. An `aligned` cell's Math
takes its `tex` from such a list, so digits vanish. Trigger (amsmath):
`\[\begin{aligned}a &= 2.414 \times 10^{-3}\end{aligned}\]` — pdflatex typesets 2.414 and 10;
Perl writes `tex="\displaystyle=414\times 0^{-3}"`. With hyperref, `\hyperref[x]{\mathsf{CUA}}` in
math gives Rust's inner Math `tex="UA"`. Rust ported `removeNodeBox` in batch 56jo, which reproduced
this in 153 of 3,003 arXiv papers (2605.05619: 1,486 `tex` values). The W12 fixup removes a
ligature-merged token without touching any box (`Document::apply_math_ligature`): its box lives on
in the merged token's. The tex is then complete, and the `\mathsf{CUA}` Math reads "CUA" (it was
"C" before 56jo). Guard `node_box_append::math_ligatures_keep_their_boxes`; repro
`math-parse/node_box_math_ligature_tex.tex`; witnesses 2605.02288, 2605.00812.

The same removal follows a rename: `renameNode` (:2029-2071) copies every attribute, `_box` included,
to the new node and then `removeNode`s the old one (:2064), taking the box the new node still holds
out of the parent's. `cleanup_XMText` (TeX_Math.pool.ltxml:246-263) renames an XMText holding a
single Math to `XMWrap`, so an aligned cell drops a boxed piece from its `tex`: `\[\begin{aligned}h &=
\mbox{$x$} + \raisebox{1pt}{$z$} + \hbox{$y$}\end{aligned}\]` — Perl writes `\displaystyle=++`. (A
`\resizebox`/`\scalebox` survives in Perl: its XMText first takes the inner `ltx:inline-block`'s `_box`
in the raw attribute copy, :259-260, so the rename removes that one, which the list never held.) Rust
removed all five since 56jo, and `\sideset`'s copy-then-remove (Perl moves the node,
amsmath.sty.ltxml:1242) its `\sideset` (witnesses 2605.07372, 2605.17816, 2605.23113). The W12 fixup
leaves the box in place when a renamed or copied node lives on (`rename_node`,
`Document::remove_node_keeping_box`). Guard `node_box_append::aligned_cells_keep_their_boxed_pieces`; repro
`math-parse/node_box_aligned_boxed_pieces.tex`.

## 273. The case changer expands a `\newcommand` optional-argument command and cases its default (FIXED in Rust)

l3text keeps a command whose one-step expansion opens with `\@protected@testopt` unexpanded
(`\__text_expand_testopt:N`, expl3-code.tex:36319-36328): every `\newcommand`/`\newenvironment`
with an optional argument (latex.ltx:1249 `\@xargdef`) reads its optional argument when it is
typeset, after the case change. Perl's `latexChangeCase` (latex_constructs.pool.ltxml:5520-5563)
expands it with `readXToken`, so the default is read and cased. Trigger:
`\newcommand\foo[1][x]{#1}` + `\MakeUppercase{a\foo b}` — pdflatex "AxB", Perl "AXB"; with
`\newcommand\foob[2][y]{#1-#2}`, `\MakeUppercase{a\foob{c}d}` — pdflatex "Ay-CD", Perl "AY-CD".
Rust (batch 56jn): the changer stores a `testopt`-flagged command, or the first argument of a
spelled-out `\@protected@testopt`, unexpanded (`case_testopt_store`,
`latex_constructs/mod.rs`). Guard `case_change_equivalents::optional_argument_commands_stay_unexpanded`;
repro `tools/perfect_kernel/repros/unicode-catcodes/case_change_keeps_testopt_commands.tex`.

## 274. A standalone figure panel's width carries into the next panel row (FIXED in Rust)

`arrange_panels_and_breaks` (latex_constructs.pool.ltxml:3331-3343) ends the row of a standalone
panel (an `ltx:p`) with a break, sets `$current_width = 0`, and then adds the panel's width to it
(`$current_width += $child_width` after the `if`). So the next row starts as full as the panel
was wide. Trigger (subcaption): four `\begin{subfigure}[b]{0.24\textwidth}…\end{subfigure}`
joined by `\hfill`, a caption line `{\small\makebox[0.49\textwidth]{L}\hfill
\makebox[0.49\textwidth]{R}}`, then four more panels — pdflatex sets four per row; Perl splits the
second row 1 | 3. Rust mirrored it, and with batch 56jo's node boxes (the caption line measures its
whole width, as in Perl) matched Perl's 1 | 3 (2605.02317, 2605.02364, 2605.03152, 2605.03497; 56jk
split 2 | 2). Perl's own `t/complex/figure_mixed_content.xml` has it too: the two 3cm `\subfloat`s of
`$\begin{array}{cc}…\end{array}$` are split by a break after the Math's panel, where pdflatex sets them
side by side. The W12 fixup starts the next row empty (tests/complex/figure_mixed_content.xml loses
that break). Guard
`node_box_append::a_standalone_panel_row_starts_the_next_row_empty`; repro
`graphics-tikz/node_box_panel_row_after_caption_line.tex`.

The heuristic's merge of a small panel into the block after it (:3316-3319) also reorders content:
`$child->appendChild($prev_node)` puts the panel after the block's content, so
`\includegraphics[width=5pt]{x}\begin{minipage}{0.5\textwidth}ONE\par TWO\end{minipage}` opening a
figure gives `<block><p>ONE</p><p>TWO</p><graphics/></block>` (pdflatex: the image first). With the
row carry-over gone the merge also runs after a paragraph. Rust puts the panel first in the block.
Guard `node_box_append::a_panel_merged_into_a_block_keeps_its_place`; repro
`graphics-tikz/node_box_panel_merge_keeps_order.tex`.

## 275. A braced `{Dimension}`/`{Glue}`/`{Number}` argument drops what follows its value (FIXED in Rust)

A typed braced argument is a `Plain` parameter re-read by its inner type (Package.pm:212-218,
Parameters.pm:78-85 `reparseArgument`), inside `readingFromMouth` (Gullet.pm:108-146), which closes
the argument's mouth with whatever the scan left unread (:134-135, a forced `closeMouth`) — no
diagnostic. TeX never sees those braces: latex.ltx hands every such length to `\setlength#1#2{#1
#2\relax}` (:10253) and every count to `\global\csname c@#1\endcsname#2\relax` (:10115-10122), so the
rest stays in the input. Triggers, with `\def\foo{x}`: `A\hspace{1em\foo}B` — pdflatex "Ax B", Perl
"A B"; `\setlength{\parindent}{2pt\foo}G` — pdflatex "xG", Perl "G"; `\setcounter{c}{5\foo}` drops
the `x`. The same loss hides a type narrower than TeX's scan: `\setlength`/`\addtolength`
(latex_constructs.pool.ltxml:4603-4615) and `\hspace` (:4629) read a `{Dimension}`, so
`\setlength{\parskip}{3pt plus 1pt}` keeps 3pt and `\hspace{\stretch{1}}` is 0pt; `\rotatebox{Number}`
(revtex4_support.sty.ltxml:118) drops the `.5` of `22.5`; floatflt's `{floatingtable}{Dimension}`
(floatflt.sty.ltxml:44) drops the table itself, which is that argument. calc's expression reader
(calc.sty.ltxml:114-121) drops an unparsable rest where calc.sty reports it (calc.sty:281-284), and
reads a parenthesized group up to its FIRST `)` (:183-184), so `1.5\x*((\value{c})-1)` (hexgame.sty:77)
loses its `-1)`; and a `[Dimension]` read of `\makebox[\widthof{ab}+\widthof{cd}]` stops after the first
term (pdflatex with calc 38.6pt; Perl 0.0pt). Related binding drift: pgfsys-latexml.def.ltxml:492
reads `\pgfsys@declarepattern` with nine arguments, pgf 3.1 passes fifteen (a matrix before the code,
pgfcorepatterns.code.tex:159-164), so a pattern's code is lost and the call's rest runs loose. Rust
(batch 56jr): OXIDIZED_DESIGN_DIVERGENCES #317.
Repros `expansion-primitives/{braced_value_tail_after_assignment,braced_length_skip_keeps_stretch,
braced_length_tail_box_commands,calc_braced_length_expression,calc_braced_length_invalid_tail,
picture_length_default_units,floatingtable_table_argument}.tex`; guards `braced_quantity_tail::*`.

The same Perl bindings declare package dimens as counts: lineno.sty.ltxml:46 and :64 (`\linenumbersep`, `\quotelinenumbersep`, `\newdimen` at lineno.sty:1549 and :2852) and floatflt/floatfig.sty.ltxml (`\htdone`, floatflt.sty:35), so `\the\linenumbersep` prints `0` (pdflatex `10.0pt`) and `\setlength{\linenumbersep}{2.5pt}` assigns 2, whose `.5pt` Perl drops and Rust (56jr) would print. Rust declares them `Dimension` with the package initial values (guard `braced_quantity_tail::package_registers_are_dimens`; witness 2605.07149).

Some Perl readers take less of an argument than TeX. The rest is dropped silently, so the loss
does not show:
- multido.sty.ltxml:71 starts a Number variable at `readFloat->revert`, so `\multido{\n=1+1}{3}{\n}`
  gives `1.0, 2, 3` (pdflatex `1, 2, 3`: multido.tex:193-197 `\edef#3{#1}`).
- Its `\fpAdd` sums two Floats (:112-119), so `2.00+-3.05` steps to `-4.1` (pdflatex `-4.10`, in
  the step's decimals, :215-283).
- pstricks_support.sty.ltxml:198-215 `ReadPSAngle` reads no coordinate angle and no PostScript
  angle, so `\psarc(0,0){1}{(1,1)}{(0,1)}` has angles 0 (pdflatex 45 and 90, pstricks.tex:990-999).
- graphics.sty.ltxml:26-36 `GraphixDimension` is a plain `readDimension`, so with calc
  `\resizebox{\width}{\ht\x+\dp\x}{…}` scales to the height alone (graphics.sty:555-568 reads
  it with `\setlength`).

Rust (batch 56ju): OXIDIZED_DESIGN_DIVERGENCES #317, "Readers that took less than TeX". Repros
`expansion-primitives/{multido_number_variable_as_written,pstricks_angle_argument_read_whole,
calc_resizebox_length_expression}.tex`.

## 276. glossaries: an entry added by `\glsadd`/`\glsaddall`, or any entry of a glossaries-extra document, is never listed (FIXED in Rust)

makeindex lists every entry written to the glossary file. `\glsadd` (glossaries.sty:5280-5292)
writes through `\@@do@wrglossary` (:6299) as `\gls` does (:3813 → :6246); `\glsaddall` (:5295) and
`\glsaddallunused` (:5302) `\glsadd` every entry. Perl lists an entry only when an
`ltx:glossaryref` refers to it (MakeIndex.pm:468) and wraps only `\@gls@link`
(glossaries.sty.ltxml:26-37), so an entry added by `\glsadd` alone is dropped. glossaries-extra
replaces `\@gls@link` (glossaries-extra.sty:3465) and `\glsadd` (:3571, through `\@glsadd` :3581,
also reached by `\glsaddeach` :3605 and `\glsstartrange`), so under it no `\gls` makes a reference
and the glossary is empty. Trigger: `\newglossaryentry{str}{…}` + `\glsaddall` + `\printglossary`
— pdflatex lists Strength, Perl an empty glossary; `\usepackage{glossaries-extra}` +
`\gls{dex}` + `\printglossary` — pdflatex lists the entry, Perl nothing. Rust (batch 56jt):
`\glsadd` (and glossaries-extra's `\@glsadd`) emits a location-only
`<ltx:glossaryref show="none"/>`, carried from the preamble into the body's first paragraph; CrossRef
leaves it unfilled and the XSLT prints nothing for it; both wraps are re-applied after
glossaries-extra loads (`glossaries_sty.rs`, OXIDIZED_DESIGN_DIVERGENCES #320). Witness
ualberta/ualberta 83.9 → 95.7 % recall.

The `\@gls@link` wrap is an `ltx:glossaryref`, so a `\gls` in math is an `XMText` atom of the
formula. Trigger: `\newglossaryentry{v}{name={\ensuremath{\mathbf{v}}},description={velocity}}` +
`$\gls{v} = \frac{d\gls{v}}{dt}$` — Perl's `tex=` is
`\lx@glossaries@gls@link{main}{v}{{{}}\mathbf{v}}=\frac{d\lx@glossaries@gls@link{main}{v}{…}}{dt}`,
with 2 "`<ltx:XMTok>` isn't allowed in `<ltx:glossaryref>`" errors (same host, 2026-09-26); the
HTML term is an `<mtext>` holding a nested `<math>`. Rust (batch 56jt): in math the original
typesets the term alone and the location-only reference follows the formula (none in display
math), so `alttext` is `{{}}\mathbf{v}=\frac{d{{}}\mathbf{v}}{dt}`; witness
glosmathtools/sample_glosmathtools_en (20 formulae). Guards
`stream_a_recall::{glsaddall_lists_every_entry, glossaries_extra_lists_used_and_added_entries,
gls_in_math_typesets_the_term_alone, preamble_glsadd_opens_no_paragraph}`; repros
`tools/perfect_kernel/repros/index/glsadd_entries_listed.tex`, `glsadd_glossaries_extra.tex`,
`glsadd_preamble_first_paragraph.tex`, `gls_in_math_glossaries.tex`, `gls_in_math_glossaries_extra.tex`.

## 277. MakeBibliography reads the first bibliography's files for every bibliography (FIXED in Rust)

`getBibliographies` takes `@files` from the first `//ltx:bibliography` (MakeBibliography.pm:101)
whichever bibliography it is processing, so an inline `{thebibliography}` (no `@files`) before
`\bibliography{refs}` leaves the real list empty ("Missing bibkeys" for every citation), and in a
multi-bibliography document every list reads the first one's files. Trigger: `\begin{thebibliography}
{9}\bibitem{ex} X.\end{thebibliography}` + `\cite{knuth}` + `\bibliography{refs}` — pdflatex+bibtex
print Knuth, Perl an empty second list. Rust (batch 56jt, `make_bibliography.rs`
`get_bibliographies`): the processed node's `@files`, then its parent's, then the first
bibliography that names any (a biblatex `\printbibliography` after the first took the resource
list names none). A cited key without an entry in those files is missing, as in Perl (:340-343);
the pseudo-item the port built from another list's ObjectDB record ("[n] Cited by: …", taking over
the citation's link) is gone. biblatex's `\addglobalbib` resources head the list of a refsection
that names its own (biblatex.sty:10797-10801, `biblatex_sty.rs` `\biblatex@section@resources`).
Witnesses lshort-german/l2kurz 97.1 → 99.3 %, latex-via-exemplos 98.2 → 98.3 %; biblatex-apa-test
keeps its recall with 81 fewer duplicated items. Guards
`stream_a_recall::{each_bibliography_reads_its_own_files, refsection_reads_the_global_resources,
a_citation_of_another_lists_item_is_missing_here}`; repros
`tools/perfect_kernel/repros/index-bib/bib_files_of_each_bibliography.tex`,
`biblatex_refsection_global_resources.tex`, `bib_other_list_item_is_missing.tex`.

## 278. A `.bib` title's re-case lowercases a bare control word into an undefined command (FIXED in Rust)

`\bib@@title` re-cases the raw title (BibTeX.pool.ltxml:293-333, default `capitalize1`), and its
word pattern `(?:\w|\\(?:\w+|.))+` takes a control word with its word, so `lc` turns `\LaTeXe`
into `\latexe` and `\TeX` into `\tex`: undefined-macro errors. BibTeX's `change.case$` does the
same to an unbraced control word, so a document that pdflatex compiles clean either braces it or
uses a style that keeps the title's case (unsrtdin). Trigger: `title = {Using Imported Graphics in
\LaTeXe}` under `\bibliographystyle{unsrtdin}` — pdflatex "…in LaTeX2e", Perl
`Error:undefined:\latexe`. Rust (batch 56jt, `bibtex.rs` `recase_title_with`): a control word is
re-cased only when the re-cased command is defined (`\O` → `\o` still). 278 bibliography manuals:
817 → 802 errors (aomart, asmeconf, asmejour, erdc, memman, ndsu-thesis, nostarch, resphilosophica,
seuthesix, uowthesis, lshort-german/l2kurz). Guards
`stream_a_recall::bib_title_recase_keeps_undefined_control_words`,
`bibtex::tests::recase_keeps_a_control_word_whose_recased_name_is_undefined`; repro
`tools/perfect_kernel/repros/index-bib/bib_title_recase_keeps_control_words.tex`.

## 279. `\openout` and `{filecontents}` store an extension-less file under its bare name (FIXED in Rust)

TeX completes an output name without an extension to `<name>.tex` (tex.web §1374, tex.web:24928
"if cur_ext="" then cur_ext:=".tex""; the extension follows the last `.` of the last path
component in web2c), and LaTeX's `{filecontents}` writes through `\immediate\openout`
(latex.ltx:18998, :19023). Perl's `\openout` stores the bare name (TeX_FileIO.pool.ltxml:120-126),
so a later `\IfFileExists{name.tex}` is false and `\input{name.tex}` a missing file. Trigger:
`\begin{filecontents*}{democode}Alpha code.\end{filecontents*}` +
`\IfFileExists{democode.tex}{Yes}{No}` + `\input{democode.tex}` — pdflatex "Yes Alpha code.",
Perl "No" + `Error:missing_file`. Rust (batch 56jt): `virtual_files::output_file_name` names the
file for `\openout`, `{filecontents}`, tcolorbox's `\tcbverbatimwrite` (expl3 `\iow_open:Nn`) and
listings' `\lst@WFBegin` (lstmisc.sty:61); TeX Live's braced name is its group's content. Witness
latex4wp (latexdemo.sty:97-101 writes `democode`, :159/:167 test `democode.tex`: every
`\PrintDemo` example was dropped at 0 errors; 93.9 → 99.6 % recall). Of 687 corpus manuals that
load a file-writing package, only latex4wp and sesamanuel/sesamath-doc-fr (fancyvrb `VerbatimOut`
names read back by `\input`, unchanged) write an extension-less name. Guards
`stream_a_recall::{openout_names_an_extensionless_file_tex,
wrapped_filecontents_prints_the_code_and_its_result}`,
`virtual_files::tests::output_file_name_completes_as_openout_does`; repros
`tools/perfect_kernel/repros/string-mouth/openout_completes_tex_extension.tex`,
`openout_wrapped_filecontents.tex`, `openout_latexdemo_printdemo.tex`. Residual: a `{filecontents}`
without `[overwrite]` now shadows a `name.tex` already on disk, which LaTeX keeps (SYNC_STATUS).

## 280. A bibliography entry's fields run together with no separator (FIXED in Rust)

MakeBibliography's `%FMT_SPEC` rows of one block carry `""` punctuation where neither field supplies
a space, and LaTeXML.css adds no separator between the `ltx_bib_*` spans: the editor after the
author in the name block of `book`/`report`/`thesis`/`website` (MakeBibliography.pm:708, :752, :769,
:783 — "George Frideric HandelAtlanta Symphony (Ed.)"), a website's title and type after its name
(:781-787 — "V. JacobsonModified TCP…(Website)"), `software`'s type after its key (:794), and every
row whose path matches several elements, which `do_any` (:550) concatenates: an entry's notes (:688
— `note` and `howpublished` are both `ltx:bib-note`: "NovemberhowLimanote"), `organization` and
`institution` (both `ltx:bib-organization`: "OscarorgCobaltinstitution"). Trigger: `@misc{m,
author={A B}, title={T}, howpublished={H}, note={N}}` + `@online{o, author={V. Jacobson},
title={Modified TCP}}` — Perl "Note: HN", "V. JacobsonModified TCP(Website)". Rust (batch 56jt,
`make_bibliography.rs` `get_fmt_spec`): the editor follows the author after ", " (as the
`do_editorsB` rows follow other fields), the website title and the parenthesized types after a space
(as a title's block follows the name block in every other type, and as Perl's `status`/`language`
rows are spaced), the notes are units (`Formatter::Units`, biblatex's `\newunit` / BibTeX's
`new.block`: "Note: H. N"), and the elements of any other row are a list with ", " between them
(`Formatter::Any`); OXIDIZED_DESIGN_DIVERGENCES #321. Guard `stream_a_recall::bibliography_fields_are_separated`; repro
`tools/perfect_kernel/repros/index-bib/bib_fields_separated.tex`.

## 281. `\glossary` in text is dropped, a missing makeindex output prints nothing, and MakeIndex builds one list (FIXED in Rust)

Perl's `\glossary{}` (latex_constructs.pool.ltxml:4424-4436) warns `unexpected:glossary` and
discards the entry inside `ltx:p`/`ltx:text`; elsewhere it inserts an `ltx:glossaryphrase`, which the
schema rejects at document level (`Error:malformed:ltx:glossaryphrase isn't allowed in
<ltx:document>`). doc.sty's `\changes` is a `\glossary` entry (doc.sty:626-650), so every change
record is lost. `\PrintChanges` and `\PrintIndex` input `\jobname.gls` and `\jobname.ind`
(doc.sty:680, :624), which only makeindex writes: Perl's `\@input@` (pool:4230) prints "No file",
so the Change History and the index prologue (doc.sty:583-597, `\IndexPrologue`) never appear. Even
when the file exists, Perl's `\begin` runs its own `{theindex}` constructor first (pool:193-213), not
doc.sty's `\theindex`, so the prologue is lost anyway. Perl's MakeIndex fixes the list to `idx`
(MakeIndex.pm:77-78). Its :89 test compares the scanned `inlist` hash with `'idx'`, so any mark that
has a list is skipped. Trigger:

```latex
\documentclass{ltxdoc}
\RecordChanges
\begin{document}
Text.\changes{v1.0}{2011/01/10}{Alpha stable release}
\PrintChanges
\end{document}
```

Perl 0.8.8 (same host, `[rawstyles,rawclasses]`): one `Warning:unexpected:glossary` and "Text."
only. pdflatex + `makeindex -s gglo.ist`: "Change History v1.0 General: Alpha stable release 1".
Rust (batch 56js): after `\makeglossary`, `\glossary` is an `ltx:indexmark` in list `glo`; before
it, the entry is read and dropped, as latex.ltx:17742 does. A missing `\jobname.ind`/`.gls`
is stood in for, and MakeIndex builds each list (OXIDIZED_DESIGN_DIVERGENCES #318). Witnesses: stream
A cluster 1, 38 TL manuals with 832 missing words (source2e 244, biblatex-ieee 110, biblatex-chem 81;
joinbox, circledtext and sunpath prologues). Repro
`index/doc_changes_index_prologue.tex`; guards `doc_changes_index::*`.

## 282. `\index` entries are split in makeindex's default characters, and a `\verb` inside one ignores makeindex's quoting (FIXED in Rust)

Perl's `process_index_phrases` (latex_constructs.pool.ltxml:4326-4371) splits every entry at `"`,
`@`, `!` and `|`. It never expands the entry and handles the quote only in its split loop, after
`SanitizedVerbatim` has re-tokenized the entry (pool:4376-4395). Three consequences follow.
First, an entry written for another makeindex style stays one phrase. doc.sty writes
`\@gtempa\actualchar\verb\quotechar*\verbatimchar\bslash\@gtempa\verbatimchar…` for gind.ist
(`actual '='`, `quote '!'`, `level '>'`; doc.sty:521-524, 1054-1093). Perl's key is then
`foo=“verb!*+“foo+`, where pdflatex + `makeindex -s gind.ist` prints `\verb*+\foo+`. Second, a
quoted `\verb` delimiter is taken literally. lshort's `\index{^@\verb"|^"|}` (math.tex in the
Slovenian, Mongolian, Vietnamese, Finnish, Persian and Spanish translations) and amsldoc.tex's
`\index{"|@\verb"*+"\"|+}` are `\verb|^|` and `\verb*+\|+` for makeindex. Trigger:

```latex
\documentclass{article}
\usepackage{makeidx}\makeindex
\begin{document}
Superscripts\index{^@\verb"|^"|} and a bar\index{"|@\verb"*+"\"|+}.
\printindex
\end{document}
```

Perl 0.8.8: `Error:unexpected:^ Script ^ can only appear in math mode` ×2,
`Error:expected:{} Missing argument {}`, `Error:misdefined`, and index entries "^—" with no `\|`.
pdflatex + makeindex: "^, 1" and "\|, 1".

Third, `\string\verb` (amsldoc.cls:87-92 `\@indexcs`) writes the characters `\verb`, which the
`.ind` re-read makes the command. Neither engine treated it that way. Rust (batch 56js) reads the
entry as makeindex reads the written line (OXIDIZED_DESIGN_DIVERGENCES #318). Repros
`index/index_quote_verb_lshort.tex`, `index/index_string_verb_amsldoc.tex` and
`index/doc_index_entry_writers.tex`; guards `doc_changes_index::{index_verb_reads_makeindex_quotes,
index_string_verb_is_the_command, doc_index_entries_read_their_macros,
index_entries_follow_their_writers}`.

## 284. `\hyperref{url}{category}{name}{text}` reads its link text as Semiverbatim (FIXED in Rust)

Perl hyperref.sty.ltxml:217-222 declares `\hyperref@@iv` with four Semiverbatim parameters, so the
link text is fully expanded and neutralized before it is digested (Parameter.pm:122-131): a `$`
or `^` in it is printed, and a font switch loses its effect. The anchor is `CleanID("cat.name")`
even for an empty category (`#X.` for two empty arguments). Real hyperref reads three arguments
and typesets the text as ordinary material through `\hyper@@link` (hyperref.sty:4825-4832); the
anchor is `\ifx\\#2\\\else#2.\fi#3` (:4827). Trigger:

```latex
\documentclass{article}
\usepackage{hyperref}
\begin{document}
A \hyperref{univie-ling-expose.pdf}{}{}{\textbf{Manual}} B

C \hyperref{doc.pdf}{section}{intro}{$x^2$ text} D
\end{document}
```

Perl: `<ref href="univie-ling-expose.pdf#X.">Manual</ref>` (not bold) and `<ref
href="doc.pdf#section.intro">$x^2$ text</ref>`. pdflatex: a bold "Manual" linked to the file, and
math. Under raw styles the expansion also reached `\edef\f@series` with `\f@series` expanded, a
definition named `m` (tex.web §1215, batch 56jp): univie-ling 7 errors, fixdif-zh-cn 2. The Rust
port had also dropped the URL (`#X.`). Rust (batch 56jw): `\hyperref@@iv Semiverbatim Semiverbatim
Semiverbatim {}`, bounded, href `compose_url(BASE_URL, url, anchor)`; `\htmlref {} Semiverbatim`
likewise. univie-ling 7 → 0, fixdif-zh-cn 7 → 5. Repro
`tools/perfect_kernel/repros/backend-persona/hyperref_four_argument_text.tex`; guard
`sweep125_roots::hyperref_four_argument_text_is_material`.

## 285. listings labels a listing through `Invocation(\label, …)`, which a redefined `\label` misreads (FIXED in Rust)

listings.sty.ltxml:196-198 prepends `Invocation(T_CS('\label'), $label)` to a labelled listing's
body. An Invocation reverts its arguments against the CURRENT definition's parameters, so under a
package that gives `\label` an optional argument — cleveref.sty.ltxml:22
`\lx@cleverref@label[]`, zref-clever — the label goes into the optional argument and is lost:
`labels="LABEL:"`. Real listings writes a literal `\label{\lst@label}` (listings.sty:1641).
Trigger: `\usepackage{listings}\usepackage{cleveref}` + `\begin{lstlisting}[caption=Example of
How,label=lst:T] x = 1 \end{lstlisting}` — Perl `labels="LABEL:"`, pdflatex "Listing 1: Example of
How" referable as `lst:T`. In Rust the real `\label` then read the caption's expansion as its
argument: 4 "cannot be a definition's name" errors and no `<caption>`. Rust (batch 56jw): literal
`\label{<label>}` tokens (`listings_sty.rs` `lst_process_display_with`). ualberta 16 → 0 errors
(recall 83.9 → 95.7 %), unbtex-example 22 → 2; empty listing labels restored in regulatory-en/-nl
(5 each, zref-clever), elteiktdk_en/_hu and elteikthesis_en/_hu (2 each). Repro
`tools/perfect_kernel/repros/captions-floats/listings_label_under_cleveref.tex`; guard
`sweep125_roots::listings_label_under_cleveref`.

## 286. Binding closures that define as they expand run inside a number scan's look-ahead (FIXED in Rust)

etoolbox.sty.ltxml:39-50 binds `\newrobustcmd`/`\renewrobustcmd`/`\providerobustcmd` as `DefMacro`
closures that make the definition while being expanded. The real commands are `\protected\def
\newrobustcmd{\@star@or@long\etb@new@command}` (etoolbox.sty:57-58, 85, 96): they reach a
`\futurelet` peek, where a number scan ends (tex.web §445), and an `\edef` keeps them. So
`\ifnum0=4%` + a newline + `\newrobustcmd*{\foo}{A}` defines `\foo` in the scan's look-ahead,
before the `\ifnum` has chosen its branch (synthslant.sty:277). Trigger:

```latex
\documentclass{article}
\usepackage{etoolbox}
\ifnum0=4%
  \newrobustcmd*{\foo}{A}%
\else
  \newrobustcmd*{\foo}{B}%
\fi
\begin{document}
x\foo y
\end{document}
```

Perl and Rust (56js) "xAy", pdflatex "xBy". The same holds, silently, for etoolbox's `\patchcmd`,
`\AfterPreamble`, `\AfterEndPreamble`, `\AfterEndDocument` and `\At…Environment` hooks
(etoolbox.sty.ltxml:1290, 1710, 1718, 1720, 1729/1731), amsthm's `\pushQED`/`\popQED`/`\qedhere`
(amsthm.sty.ltxml:120/135), `\nocite` (latex_constructs.pool.ltxml:4214) and enumitem's `\restartlist`
(enumitem.sty.ltxml:128, a Fatal in Perl). Rust (batch 56jw): these carry `peeks_by_futurelet`, so a
number scan ends at them, and etoolbox's are `protected`, as the real ones are (the
`tests/expansion/etoolbox` golden's `\ifcsprefix{newrobustcmd}` is now "true", its PDF's value).
synthslant-gauge 1 → 0 errors. Repros
`tools/perfect_kernel/repros/expansion-primitives/{newrobustcmd_number_scan,binding_assignments_number_scan}.tex`;
guards `sweep125_roots::{newrobustcmd_waits_for_a_number_scan,
binding_assignments_wait_for_a_number_scan}`. e-TeX's `\readline` (eTeX.pool.ltxml:233, a
`DefMacro`) is an unexpandable assignment of `\read`'s class: `{\endlinechar=-1%` + a newline +
`\readline\f to \x}` (latexgit.sty:53-54, shdoc.sty:210-211) read the line before `\endlinechar`
was set, with a trailing `^^M` (pdflatex "RLSAME", Perl and Rust "RLDIFF"); it is a primitive now
(repro `expansion-primitives/readline_in_number_scan.tex`, guard
`sweep125_roots::readline_is_an_assignment`).

## 290. Under babel, a `\fontencoding` switch leaves text commands in the language's encoding (FIXED in Rust)

babel_support.sty.ltxml:151 (upstream PR #2233) defines `\cf@encoding` as the expansion of
`\f@encoding` at every language switch, so `\ifx\cf@encoding\bbl@t@one` (babel.sty:3925) sees
concrete tokens. Nothing moves it afterwards: LaTeX's `\selectfont` does, through `\@@enc@update`
(latex.ltx:10502-10516, `\let\cf@encoding\f@encoding`), but Perl's `\fontencoding`
(`\lx@fontencoding`, TeX_Fonts.pool.ltxml:169-176) only merges the font. Once babel has selected
english at `\begin{document}`, `\cf@encoding` stays T1 for the whole body, and every text command
dispatches there, whatever `\fontencoding` selected. UTF-8 Greek under inputenc is
`\ensuregreek{\accpsili\textepsilon}` (lgrenc.dfu:220): `\greekscript` switches to LGR, then
`\T1\accpsili` and `\T1\textepsilon` are undefined, and so are the `\?\…` defaults, and the letter
prints nothing. `\textgreek{…}` and `{\fontencoding{LGR}\selectfont\acctonos\textalpha}` lose
theirs the same way. Trigger:

```latex
\documentclass{article}
\usepackage[LGR,T1]{fontenc}
\usepackage[greek,english]{babel}
\begin{document}
A ἐἑ άέ B
\end{document}
```

Perl 0.8.8: "A   Β" and `Warning:unexpected:\end{document}` (open groups). pdflatex: "A ἐἑ άέ B".
Without babel the kernel's `\cf@encoding` reads the font and both engines are right. Rust (batch
56jv): `\lx@fontencoding` ends with `\@@enc@update`'s `\let\cf@encoding\f@encoding` when
`\cf@encoding` does not name the encoding just merged (latex.ltx:10495). babel's concrete value
stays until the next switch and comes back at the group's end; after the `\let` `\cf@encoding` is
the live macro, so `\ifx\cf@encoding\bbl@t@one` (babel.sty:1694 `\allowhyphens`) is false after
an ungrouped switch until the next language switch. Perl's lgr.fontmap.ltxml ligatures also
lack the CB fonts' capital with the iota (grmn1000.tfm `(LABEL C A) (LIG O 174 O 11)`, H → O 12,
W → O 13): UTF-8 ᾼ ᾯ print Αͺ ῟Ωͺ where pdflatex prints ᾼ ῟ῼ. Rust takes all of the font's iota
ligatures (lgr_fontmap.rs), which also compose the precomposed vowel a declared composite prints
(`\accpsili\textalpha` + `\ypogegrammeni` → ᾀ, not ἀͺ). Witness: greek-fontenc
hyperref-with-greek. Repro `unicode-catcodes/babel_cf_encoding_follows_fontencoding.tex`; guard
`greek_text::babel_keeps_text_commands_in_the_selected_encoding`.

## 291. arydshln's `;{dash/gap}` column and the dashed rules' `[dash/gap]` option are unknown (FIXED in Rust)

The ar5iv arydshln binding (arydshln.sty.ltxml:18-24) defines only the `:` column type and
`\let`s `\hdashline`/`\cdashline` to `\hline`/`\cline`. Real arydshln maps `;` to
`\adl@argarraydashrule`, reading a `{dash/gap}` spec (arydshln.sty:198,
:236, :270-273, :287-288; `\adl@classvfordash` takes the next template item as the spec), and its dashed rules take an optional `[dash/gap]` (:432-438, :459-462,
:482-483). Trigger: `\begin{tabular}{c;{2pt/2pt}c} a & b\\\hdashline[2pt/2pt] c & d\end{tabular}`
with calc — pdflatex a clean 2-column table; Perl warns on `;`, strips the spec's braces into
columns (`p{t}` twice, "Missing number"), and leaves `[2pt/2pt]` in the next row's first cell
(arXiv 2605.19920: 6 errors in Perl, 9 in Rust 56ju, where 56jr's calc also reports the `t`).
Rust (batch 56jx, `latexml_contrib/src/arydshln_sty.rs`): `;{}` adds the same dashed border as
`:`; `\hdashline[]`, `\cdashline{}[]`, `\firsthdashline[]`, `\lasthdashline[]` read and drop the spec.
Guard `regress_2605_clusters::arydshln_dash_spec_column_and_hdashline_option`; repro
`tools/perfect_kernel/repros/alignment-bindings/arydshln_dash_spec_column.tex`.

## 292. A column type that takes an argument, at the template's end without one, reads on into the document (FIXED in Rust)

`ReadAlignmentTemplate` (Alignment.pm:895-921) reads the template token by token from the
document's input, counting braces, and invokes each column type's `\NC@rewrite@<op>` there.
A column type that takes an argument, at the end of the template with none, reads the
template's `}` as the start of its argument and scans on through the document. Trigger:
`before \begin{tabular}{cp} a \end{tabular} after` (also `{c@}`, array's `{c>}`): the document
from `a` on is consumed, with 30-34 "Unrecognized tabular template" warnings and 1 error ("Input
ended while environment tabular was open"), and "after" is lost. pdflatex reports "Missing p-arg
in array arg" (latex.ltx:16643-16644, `\@preamerr` 8997-9003) or, with array, "Missing arg: token
ignored" (array.sty:331, :414), and typesets "before a after". Rust read the template the same
way and lost the same text with 0 errors; arydshln's `{c;}` joined the triggers once batch 56jx
defined `;{}` (Perl has no `;` column: it warns and keeps "after").
Rust (batch 56jz, `latexml_core/src/alignment.rs` `read_alignment_template`): the template is
read as its argument and parsed in a mouth of its own (OXIDIZED_DESIGN_DIVERGENCES #322).
Guard `regress_2605_clusters::tabular_template_missing_argument_stays_in_the_template`; repro
`tools/perfect_kernel/repros/kernel-alignment/tabular_template_missing_argument.tex`.

## 293. A box that opens a paragraph does not start it: the space after `\mbox`, `\fbox`, `\colorbox` is dropped (FIXED in Rust)

LaTeX's box commands begin with `\leavevmode` (latex.ltx:16082 `\mbox`, 16077-16078
`\makebox`, 16182-16183 `\fbox`, 16196-16197 `\@iframebox`, 16373-16374 `\raisebox`;
color.sty:104 `\textcolor`, 163-164 `\color@b@x` for `\colorbox`/`\fcolorbox`), so a box at a
paragraph's or an item's start starts the paragraph, fires `\everypar` before the box, and the
space after it is read in horizontal mode. Perl's `\mbox` (latex_constructs.pool.ltxml:4649-4654),
`\@framebox` (:4689-4699) and color.sty.ltxml's `\colorbox`/`\fcolorbox`/`\textcolor` (:103-108)
have no enterHorizontal, so the paragraph starts later and the space is skipped in vertical mode.
Trigger: `\mbox{A} b` at a paragraph start gives "Ab" (pdflatex "A b"); fancyvrb's `\Verb` ends in
`\mbox` (fancyvrb.sty:1245), so `\item \Verb+x+ et` gives "xet" (matapli-doc). Perl's `\makebox`
and `\raisebox` have enterHorizontal => 1 (:4658-4667, :4800-4802); Rust had lost it in batch 54n.
Rust (batch 56kb): all of them start the paragraph (OXIDIZED_DESIGN_DIVERGENCES #323); `\fcolorbox`
split the paragraph for another reason, its `internal_vertical` body, until 57r (KPE #336). Guard
`mbox_argument_is_bounded::box_commands_start_the_paragraph`; repro
`tools/perfect_kernel/repros/boxes-groups/mbox_leavevmode_space.tex`.

## 294. biblatex's own entry types and an article's editors are unknown to the formatter (FIXED in Rust)

biblatex's standard styles alias its entry types to the standard drivers (standard.bbx:740-752:
`review`/`suppperiodical` → article, `bookinbook`/`suppbook` → inbook, `mv*`, `reference`,
`inreference`, `software` → misc) and print an article's editors after the journal (standard.bbx:46-48
`byeditor+others`). Perl's formatter knows neither (`MakeBibliography.pm:410`
`@{ $FMT_SPEC{$type} || [] }`): through `\bibliography{}` a `@review` or `@bookinbook` is an empty
"[n] Cited by" and a `@software` its title alone, and an article's editor is not printed (Perl `latexmlc`
on biblatex's `\addbibresource` builds no bibliography: `\addbibresource`, `\printbibliography`
undefined). An in-book or in-collection entry's edition, filed under its host, is printed by no row
either. Trigger: `@review{rev, author={Ann Author}, title={Review of Things}, journaltitle={Atlantic
Monthly}, pages={12--14}}` — biber "Review of Things. In: Atlantic Monthly, pp. 12–14."
Rust (batch 56kc): OXIDIZED_DESIGN_DIVERGENCES #324. Guards
`bibliography_names_fields::{biblatex_entry_type_aliases_take_the_standard_drivers,
biblatex_alias_declarations_retype_only_to_entry_types, biblatex_articles_print_their_editors,
biblatex_inreference_keeps_its_editor_label_and_edition, bst_incollection_prints_its_host_edition}`;
repros `tools/perfect_kernel/repros/index-bib/biblatex_{type_aliases,alias_declarations,article_editor,
inreference_editor_label}.tex`, `bib_host_edition.tex`.

## 295. `\@makeschapterhead` is undefined under book and report (FIXED in Rust)

book.cls:396 and report.cls:369 define `\@makeschapterhead{title}`, the unnumbered chapter head that
classes and packages call for their own starred heads (thesis classes' renewed `theindex`, toc/lof
renewals, apacite, sectsty; 110 files in TeX Live). Perl's book/report bindings do not define it, so a
direct call is "undefined". Trigger: `\documentclass{book}` … `\makeatletter\@makeschapterhead{Index}`.
Rust (batch 56kd): book_cls.rs and report_cls.rs define it as `\@schapter`'s dispatch
(`\@startsection{chapter}{0}{}{}{}{}*{#1}`); article keeps none (texilikechaps.sty:89 tests
`\@ifundefined{@makeschapterhead}`). Guard `doc_changes_index::renewed_theindex_opens_an_unnumbered_chapter`;
repro `tools/perfect_kernel/repros/sectioning-frontmatter/makeschapterhead_renewed_theindex.tex`
(ryethesis/ryesample).

## 296. A binding's braced length ignores a redefined `\setlength` (FIXED in Rust)

latex.ltx passes the user lengths of `\hspace`, `\\[..]`, `\makebox`, `\framebox`, `\parbox`, `minipage`,
`\rule` and `\raisebox` to `\setlength` (`\@hspace#1{\setlength\sp@ce@skip{#1}\hskip\sp@ce@skip}`,
latex.ltx:9425; `\@imakebox` 16101, `\@rule` 16363), so a package that redefines `\setlength` extends what
such a length may be: bxcalcux.sty's `\bxcx@decl@patch\setlength` adds units
(`\newcalcunit{tm}{0.05em}`), adjustbox and combinedgraphics swap in their own. Perl's
`\hspace{Dimension}` (latex_constructs.pool.ltxml:4629), `\lx@newline [Glue]` (:254),
`\makebox[Dimension]`, `\rule{Dimension}` read the length themselves, so the redefinition is bypassed.
Trigger: `\let\old\setlength \renewcommand*\setlength[2]{\old{#1}{2\dimexpr#2\relax}}` then
`\setbox0\hbox{\hspace{4pt}}\the\wd0` — pdflatex 8.0pt, Perl 4.0pt. With bxcalc raw-loaded
(`--includestyles`; there is no binding), Perl's `\hspace{6tm}` warns "Illegal unit of measure" and drops
the letters (#275); without it `\newcalcunit` is undefined. Rust (batch 56kf): OXIDIZED_DESIGN_DIVERGENCES
#325. Guards `braced_quantity_tail::redefined_setlength_reads_a_braced_length`,
`::redefined_setlength_scans_a_dimension_as_a_dimen`, `::nested_pgfpicture_lengths_stay_off_setlength`; repro
`tools/perfect_kernel/repros/expansion-primitives/braced_length_redefined_setlength.tex`.

## 297. Under ar5iv, a finite long pgf plot stops at the pushback limit (FIXED in Rust)

pgf expands a whole soft path at once (pgfsyssoftpath.code.tex:66-75, 94-98, 122-131). Perl's pushback holds a copy of every expanded body, where TeX pushes a pointer (tex.web §323). So a 9,000-sample `plot[smooth]` exceeds ar5iv.sty.ltxml:16's `pushbacklimit=599999`: Perl stops with `Fatal:timeout:pushback_limit 599999` after 107 s. pdflatex converts it, and so does Perl without ar5iv, with the same path as Rust byte for byte.

Trigger: `\draw plot[smooth,samples=9000,domain=0:10] (\x,{0.1*\x});` under `--preload=ar5iv.sty`.

Rust (batch 56kh): the ar5iv limit is the binary's 5,000,000 (OXIDIZED_DESIGN_DIVERGENCES #326). Guard: `perfect_kernel_batch56::smooth_plot_past_the_box_cycle_floor`.


## 298. A box's size arguments do not measure the box: `\width`, `\height`, `\depth`, `\totalheight` are `0pt` text (FIXED in Rust, but `\raisebox`'s raise)

latex.ltx sets a box before it evaluates the box command's size arguments (`\@begin@tempboxa`, latex.ltx:16085-16094). While they are evaluated, `\width` is `\wd\@tempboxa`, and likewise `\height`, `\depth` and `\totalheight`. This covers `\@imakebox` 16099, `\@iframebox` 16196, `\@iiiparbox`'s height (16254) and `\@irsbox`/`\@iirsbox` (16378-16393).

Perl does two things differently:
- It defines the four as the text `0pt` (latex_constructs.pool.ltxml:4644-4647).
- It reads the arguments as typed parameters before the box (`\@makebox`, `\@framebox`, `\lx@parbox`, `\raisebox`, :4658-4800).

graphics.sty.ltxml:61 notes the gap. The text binding also breaks the arithmetic: `2\width` reads as `20pt` and `-.5\height` as `-.50pt`.

Triggers, with pdflatex's values:

| Input | pdflatex | Perl |
|---|---|---|
| `\raisebox{-.5\height}{x}` | raised -2.15277pt | `yoffset="-0.5pt"` |
| `\makebox[2\width]{x}` | 10.5556pt | 20pt |
| siamart's `\raisebox{0pt}[\height][0pt]{g}` | depth 0pt | Perl's sizer ignores both optionals |

Rust (batch 56kl): the width, height and `[height][depth]` arguments are fixed (OXIDIZED_DESIGN_DIVERGENCES #328). `\raisebox`'s raise waits for a `yoffset` render change (RED repro `raisebox_raise_measures_the_box.tex`). Guard: `perfect_kernel_batch56::box_size_arguments_measure_the_box`.

## 299. `\rotatebox[…]` turns about the reference point, not graphicx's centre (FIXED in Rust)

graphicx's `\Grot@box@kv` (graphicx.sty:226-242) starts the point of rotation at the box's centre whenever `[…]` is given. It then applies the `Grot` keys in order: `origin` letters `l`/`r`/`t`/`b`/`B` set one axis each, `c` changes nothing, and `x`/`y` set lengths.

Perl's `rotatedProperties` (graphics.sty.ltxml:159-169) differs in two ways:
- It starts at the reference point.
- It reads `c` as "the centre".

As a result, a single-axis origin, `[]` or `[x=…]` turns about another point. Trigger: `\setbox0\hbox{x\rotatebox[origin=r]{90}{Qg}}\the\ht0/\the\dp0`: pdflatex gives 4.30554pt/10.33339pt, Perl 4.30554pt/12.77782pt. The height is the `x`'s in both; the rotated box itself is 0/12.78pt in Perl. arXiv: 2605.23694, 2605.30813, 2605.25220.

Rust (batch 56km): OXIDIZED_DESIGN_DIVERGENCES #329. Guard: `perfect_kernel_batch56::rotatebox_turns_about_its_origin`.

## 300. A skip assigned to a dimen register keeps its stretch (FIXED in Rust)

calc's `\setlength` is `\calc@assign@skip` (calc.sty:86), so it assigns every length as a skip (:51-53). TeX then keeps only the width: "When a glue_val changes to a dimen_val, we use the width component" (tex.web §429).

Perl's calc.sty.ltxml:42-46 stores the glue in the dimen register as is.

Trigger: `\usepackage{calc}\setlength{\unitlength}{2pt plus 1pt minus 1fil}[\the\unitlength]`. pdflatex prints `[2.0pt]`; Perl prints `[2.0pt plus 1.0pt minus 1.0fil]`.

Rust stored the glue the same way. Its picture code then read a non-dimen `\unitlength` as 1pt, which collapsed pictures toward the origin: 201 pictures in 6 of 14 probed arXiv papers, including 2605.11190, 2605.05506 and 2605.15276.

Rust (batch 56kn): OXIDIZED_DESIGN_DIVERGENCES #330. Guard: `perfect_kernel_batch56::unitlength_set_through_calc_is_the_length`.

## 301. A one-item hbox reports its item's negative depth (FIXED in Rust)

TeX's `hpack` starts an hbox's height and depth at 0 and raises them to the items' maxima (tex.web §649). An hbox never has a negative height or depth.

Perl's `List()` returns a lone box of the list's mode as that box (List.pm:41-44). `\hbox`'s sizer then measures it through `getSize` (TeX_Box.pool.ltxml:366, Whatsit.pm:264-265, Font.pm:648-649), which does not floor.

Trigger: `\setbox2\hbox{x}\dp2=-5pt \setbox0\hbox{\box2}\the\dp0` gives Perl −5.0pt, pdflatex 0.0pt. Likewise `\hbox{\rotatebox[origin=l,y=1.2em]{90}{Qg}}` is −11.99998pt deep, and so are the `\fbox`, `\resizebox` and `\scalebox` of such a box.

Rust (batch 56ko): OXIDIZED_DESIGN_DIVERGENCES #331. Guard: `perfect_kernel_batch56::a_one_item_hbox_has_no_negative_depth`.

## 302. Assigning `\ht`/`\dp` of a one-item vbox changes its item (Rust correct)

In TeX a `\vbox` is a box of its own: `\ht0=0pt` changes the vbox, and `\unvbox0` gives back its item unchanged (tex.web §1247, §1110).

Perl's `List()` returns a lone box of the list's mode as that box (List.pm:41-44, as in #301), so `\setbox0\vbox{\hbox to 3cm{Text}}` holds the hbox itself, and the assignment reaches the item.

Trigger: `\setbox0\vbox{\hbox to 3cm{Text}}\ht0=0pt \dp0=0pt \setbox4\vbox{\unvbox0}\the\ht4` gives pdflatex 6.83331pt, Perl 0.0pt.

Rust gives 6.83331pt (found probing batch 56kq's review; no change needed).

## 303. A font loaded `at` a size keeps its design size's parameters, truncated to a whole multiple (FIXED in Rust)

TeX scales a font's parameters by its `at` size over its design size (tex.web §575): `\font\y=cmr10 at 12pt` has `\fontdimen2` 4.0pt.

Perl's `\font` computes the scale as `$at->divide($size)` (TeX_Fonts.pool.ltxml:92). `Number::divide` truncates to an integer (Number.pm:112-114), so the scale is `int(at/design)`. Then it multiplies the metric's parameters by `design × scale` (:108-113).

Trigger: `\font\y=cmr10 at 12pt [\the\fontdimen2\y]` gives pdflatex 4.0pt and Perl 3.33334pt. `at 9pt` gives Perl 0pt for every parameter.

Rust (batch 56kr) scales by the size the font is loaded at. Guard: `perfect_kernel_batch56::a_font_without_its_own_metric_has_cmr_parameters`.

## 304. A class's `\LoadClass` options become global options (FIXED in Rust)

In LaTeX, `\LoadClass[opts]{cls}` passes `opts` to that class alone. `\@classoptionslist`, the global options every later package sees, is set only at the first class load, `\documentclass` (latex.ltx:18716-18718).

Perl's `InputDefinitions` pushes the options of every `cls` load onto `class_options` and redefines `\@classoptionslist` to them (Package.pm:2578-2581). That includes the `LoadClass` the dependency scan runs for an unbound class (Package.pm:2806). amsmath's `ProcessOptions` then takes them as global options (Package.pm:2455, 2472-2475).

Trigger: a class `lcfleqn.cls` containing `\LoadClass[fleqn]{article}`, then `\documentclass{lcfleqn}\usepackage{amsmath}` and an `equation`. pdflatex centres the equation: article's `fleqn` stays the class's own, and amsmath's `\newif\if@fleqn` (amsmath.sty:64) and display environments discard it, since amsmath got no `fleqn`. Perl writes `<document class="ltx_fleqn">`.

Rust matched Perl until batch 56kv (DIVERGENCES #334). Witnesses: webofc (2605.12407, 2605.15288) and USG (2605.00042, 2605.20200). Each class loads amsmath itself without `fleqn`. Repro: `tools/perfect_kernel/repros/loader/loadclass_options_stay_local.tex` (no preload).

With raw classes, the class's `\LoadClass[fleqn]{article}` runs the article binding's `fleqn` handler. Rust and Perl then give `ltx_fleqn` whether or not amsmath loads. The Rust fix has both halves: a class loaded while another class loads keeps its options to itself, and the amsmath binding's `\if@fleqn` reset discards a class's `fleqn`. Perl also replaces `\@classoptionslist` with the nested class's options, so `\documentclass[french]{lcfleqn}` reads `fleqn` where pdflatex reads `french`. Guard: `perfect_kernel_batch56::loadclass_options_are_the_class_own`.

## 305. elsarticle always sets its equations flush left (FIXED in Rust)

elsarticle.cls inputs `fleqn.clo` only for the `5p` layout, or `3p` with `twocolumn` (elsarticle.cls:1280, 1295). The default `preprint` layout, `1p`, and one-column `3p` centre their equations.

Perl's binding runs `RequirePackage('fleqn')` for every elsarticle document (elsarticle.cls.ltxml:45), and fleqn.sty.ltxml:20 sets `ltx_fleqn`. Perl treats the `1p`/`3p`/`5p` options as ignorable.

Trigger: `\documentclass{elsarticle}` and an `equation`. pdflatex centres it; `[3p,twocolumn]` sets it flush left (x=84pt). Perl writes `<document class="ltx_fleqn">` for both.

Rust (batch 56kw, DIVERGENCES #335) loads fleqn only for `5p`, or `3p` with `twocolumn`. A document that loads amsmath is centred since batch 56kv (DIVERGENCES #334), as in pdflatex. Guard: `perfect_kernel_batch56::elsarticle_fleqn_follows_the_journal_type`. Repro: `tools/perfect_kernel/repros/loader/elsarticle_fleqn_follows_journal_type.tex`.

## 306. amsmath's tags ignore the order of `leqno`/`reqno`, and ACM papers number on the left (FIXED in Rust)

amsmath declares `leqno` before `reqno` (amsmath.sty:54-55), and `\ProcessOptions` runs them in that order. acmart loads amsart with `reqno` (acmart.cls:282).

Perl's amsmath binding declares `reqno` first (amsmath.sty.ltxml:55-56), so a global `leqno` beats a local `reqno`. acmart.cls.ltxml:19 loads amsart without `reqno`, so ams_core's default `ltx_leqno` stays.

Triggers:
- `\documentclass{acmart}` with an `equation`: pdflatex numbers it on the right, Perl writes `<document class="ltx_leqno">`.
- `\documentclass[leqno]{article}\usepackage[reqno]{amsmath}`: pdflatex numbers on the right (x=464.7pt), Perl writes `ltx_leqno`.

Rust (batch 56kw, DIVERGENCES #336) numbers both on the right. Witnesses: 2605.02222, 2605.24417 (acmart). Guard: `perfect_kernel_batch56::amsmath_tags_follow_its_own_options`.

## 307. A trimmed raster is cropped by its resolution without the unit, and per pt instead of per bp (FIXED in Rust)

graphicx's `trim=`/`viewport=` lengths are bp, and pdfTeX sizes a raster by its own resolution: a PNG's `pHYs` (per metre), a JPEG's JFIF density (per inch or centimetre), 72 dpi by default.

Perl's `image_graphicx_complex` crops with `$idppt = (x-resolution // $dpi)/72.27` (Util/Image.pm:402-403). The divisor is a pt's 72.27, so the crop is 0.4 % short. ImageMagick's `x-resolution` for a PNG is in pixels per centimetre, so a 300-dpi PNG (118.11 px/cm) crops 2.54 times too little.

Trigger: `\includegraphics[trim=90 30 50 50,clip]` of a 3000×1500 PNG at 300 dpi. pdflatex keeps 2417×1167 pixels; Perl keeps 2772×1370 (118.11/72.27 = 1.634 pixels per bp) (read from the code; this host's Perl has no Image::Magick and skips the crop with `Error:imageprocessing:imageclass`).

Rust (batch 56kx, DIVERGENCES #337) crops by the resolution in dpi over 72. Witness 2510.17772.

## 308. A generated image name can overwrite the author's own image (FIXED in Rust)

Perl's `generateResourcePathname` numbers a processed image's file `x1`, `x2`, … (Post.pm:192-200) without checking what else the output holds. A document whose output lands beside its sources and which has its own `x1.png` gets that file overwritten by the first processed image; the figure that showed `x1.png` then shows the processed image.

Trigger: `\includegraphics[trim=100 0 0 50,clip]{a.png}` and `\includegraphics{x1.png}`, converted with the destination in the source directory.

Rust (batch 56kx) resolves every graphic's source first and reserves each copied source's relative path before naming any output (`used_dests`), so the crop takes `x2.png`; a copy onto the source itself is refused (`copy_beside`). Guard: `latexml_post graphics::tests::a_crop_gets_its_own_file_and_a_zero_trim_none`.

## 309. A `\parbox` in vertical mode does not start a paragraph (FIXED in Rust)

latex.ltx's `\parbox` ends in `\@iiiparbox`, whose body begins with `\leavevmode` (latex.ltx:16236-16262): a parbox met between paragraphs starts one, and the text after it continues that paragraph. Perl's `\lx@parbox` constructor begins `inline_internal_vertical` mode without `enterHorizontal` (latex_constructs.pool.ltxml:4750-4763), so the box becomes a block of its own and the following text a new paragraph.

Trigger: `Before.\n\n\parbox{3cm}{A} text after.` — Perl (and Rust as of 57c): three blocks, the parbox as `<para class="ltx_parbox">` between two paragraphs; pdflatex: one paragraph holding the box and "text after.".

Found by the K13 binding-conformance audit (KERNEL_CAPABILITIES; `binding_conformance::kernel_box_family_conforms_to_latex_ltx` reports it as PROLOGUE_UNKNOWN). Repro `tools/perfect_kernel/repros/boxes-groups/parbox_starts_the_paragraph.tex` (GREEN since 57d). The same shape as `\makebox`/`\raisebox`, fixed in Rust by batch 56kb. Rust (batch 57d, DIVERGENCES #338) starts the paragraph for `\parbox` where its `\leavevmode` leaves TeX in a paragraph (not inside a restricted box such as `\rotatebox`'s, nor among a float's panels); `{minipage}` (`\@iiiminipage`, latex.ltx:16305-16306; Perl :4771) has the same shape, fixed in batch 57f (repro `boxes-groups/minipage_starts_the_paragraph`, GREEN). Rust keeps Perl's shape for a box whose whole content is one float (`\captionof`): it becomes that float (user ruling 2026-09-27).

## 310. A minipage folded into its one paragraph takes a figure row of its own

`insertBlock` folds a minipage (or a `\parbox`) that holds a single `<p>` into that `<p>`, carrying the box's class and width (TeX_Box.pool.ltxml:489-492), and `arrange_panels_and_breaks` gives every `ltx:p` panel a row of its own (`%standalone_panel_names`, latex_constructs.pool.ltxml:3225-3227), though this `<p>` is a sized minipage.

Trigger: in a `figure`, `\begin{minipage}[t]{0.45\textwidth}Left text.\end{minipage}\hfill\begin{minipage}[t]{0.45\textwidth}Right text.\end{minipage}` — Perl and Rust: `<p class="ltx_figure_panel ltx_minipage">Left text.</p><break class="ltx_break"/><p class="ltx_figure_panel ltx_minipage">Right text.</p>`; pdflatex: the two side by side. Three `\centering\parbox{.3\textwidth}{P1 text}` panels stack the same way (`<p class="ltx_figure_panel ltx_parbox">` separated by breaks).

Found in the 57d A/B triage (arXiv 2605.27134 S5.F8 took this shape while a first cut of 57d folded its parbox-topped minipages into `<p>`s). Repro `tools/perfect_kernel/repros/captions-floats/minipage_text_panels_share_a_row.tex`. The same fold loses a minipage's width to panel layout: `captions-floats/minipage_panel_single_child_width.tex`. Rust fix (57o): a node carrying `ltx_minipage`/`ltx_parbox` and a `width` is a panel at that width, not a standalone row (DIVERGENCES #343).

**Guard**: `perfect_kernel_batch56::{minipage_text_panels_share_a_row, minipage_panel_single_child_width}`.

## 311. A box's `vattach` depends on a source newline after its only content

`trimNodeRightWhitespace` (TeX_Paragraph.pool.ltxml:183-193) empties a paragraph's trailing whitespace text node with `setData` and leaves it in place, and `isVAttached` (TeX_Box.pool.ltxml:433-440) counts that empty node as a second child. So whether `insertBlock`'s one-node hack ("TeX doesn't shift a single node", :470-479) drops a box's `vattach` depends on whether the paragraph's last box was followed by a space or glue in the source, which TeX discards at `\par` (tex.web §816).

Trigger: `\begin{minipage}[t]{5cm}` / `\raisebox{0pt}{\parbox[t]{2cm}{Inner}}` / `\end{minipage} after.` on three lines — Perl: `<para class="ltx_minipage" vattach="top" …>`; with `%` after the `\raisebox`: `<para class="ltx_minipage" …>`, no `vattach`. The same holds after `\hfill`, `\hspace`, `\kern`, `\hskip`, `~`, `\ ` and `\/`. Rust frees the emptied node (`trim_node_right_whitespace`, document.rs `discard_subtree`) and gives the no-whitespace answer both ways (Rust correct; found in 57e's review).

## 312. `\captionof` makes a caption-only float beside the image it captions

caption.sty's `\captionof{type}` only sets the caption type (`\caption@of` = `\setcaptiontype*`, caption.sty:391); LaTeXML's binding (caption.sty.ltxml) opens the float environment around the caption alone. In a minipage holding an image and `\captionof{figure}`, the `<figure>` holds only `<caption>`, and the image sits beside it in a `<para>` of the box; the figure does not contain what it captions.

Trigger: `\begin{minipage}{.45\textwidth}\includegraphics{a}\captionof{figure}{X}\end{minipage}` — Perl: `<logical-block class="ltx_minipage"><para><graphics/></para><figure><caption>…</caption></figure></logical-block>`. Rust (batch 57g, DIVERGENCES #339): `<figure class="ltx_minipage"><graphics/><caption>…</caption></figure>`. Repro `tools/perfect_kernel/repros/captions-floats/figure_boxes_become_figures_and_rows.tex` (GREEN).

## 313. `\orcidlink` does not start the paragraph it begins

orcidlink.sty's `\orcidlinkX` is `\href{https://orcid.org/#2}{…}` (orcidlink.sty:69), and hyperref's `\hyper@linkurl` begins with `\leavevmode` (hpdftex.def:411), so a link at a paragraph's start opens it. LaTeXML's binding makes `\orcidlinkX` a macro around the constructor `\lx@orcidlink`, which declares no mode (orcidlink.sty.ltxml:23-29): built in vertical mode, the link drops the space that follows it.

Trigger: `Before.\par \orcidlink{0000-0002-1825-0097} text after.` — Perl and Rust 57h: `<ref …>…</ref>text after.`; pdflatex: the link, a space, "text after.". Found by the K13 binding audit (57h). Rust fix: DIVERGENCES #340.

## 314. physics's matrix family: a 1×1 `\xmatrix*` gets a subscript, `\zeromatrix` needs two sizes

physics.sty's starred `\xmatrix` subscripts an entry with its row only when there is more than one row, and with its column only when there is more than one column (physics.sty:647 `_{\ifnum #3 > 1 \the\rowcount \fi \ifnum #4 > 1 \the\colcount \fi}`), so a 1×1 matrix shows its item without an index. physics.sty.ltxml:600-615 takes the column for one row, so it adds a `1`. physics.sty's `\zeromatrix` is `m g` (physics.sty:662): the column count is an optional braced argument that defaults to the row count. The binding reads `{}{}` (physics.sty.ltxml:619), so the documented `\zmat{3}` takes the next token as its second size.

Trigger: `\[\xmatrix*{x}{1}{1}\]` — Perl: `x _ 1`; pdflatex: x. `\[\begin{pmatrix}\zmat{3}\end{pmatrix}\]` — Perl: `matrix@(Array[[], []])` with 4 errors; Rust 57h: "Missing } inserted" and the matrix lost; pdflatex: a 3×3 zero matrix. Rust fix (57i): the index follows physics.sty (no subscript for 1×1, the invisible comma Perl puts between row and column kept), and `\zeromatrix{}` reads its second size only before a `{`.

**Guard**: `perfect_kernel_batch56::xmatrix_star_subscripts_entries`; repro `alignment-bindings/xmatrix_star_subscripts_entries.tex`.

## 315. threeparttable: the placement optional is typeset, a `\tnote` outside the environment prints

threeparttable.sty's environments take an optional vertical placement (`\newenvironment{threeparttable}[1][t]`, :107; `{measuredfigure}`, :122) that chooses the box and is never typeset; `\TPToverlap` gobbles its argument (`\def\TPToverlap#1{}`, :281) and is `\let` to `\relax` inside the environment (:118), so a `\tnote` prints only inside a `{threeparttable}`. threeparttable.sty.ltxml gives both environments no optional (:31, :36) and `\TPToverlap` no argument (:24).

Trigger: `\begin{table}\begin{threeparttable}[b]\caption{Cap}\begin{tabular}{c} x \\ \end{tabular}\end{threeparttable}\end{table}` — Perl: `<p>[b] <tabular…></p>`; pdflatex: no `[b]`. `\begin{tabular}{c} 4000\tnote{2} \\ \end{tabular}` — Perl: `4000<sup>2</sup>`; pdflatex: 4000. Witnesses 2605.04144 (four `[b]` panels), 2605.26854 (4000 read as squared), 2605.23257. Rust fix (57j): both environments read `[]`, `\TPToverlap{}` gobbles, the environment lets it to `\relax`.

**Guard**: `perfect_kernel_batch56::{threeparttable_reads_its_placement, tnote_outside_threeparttable_prints_nothing}`.

## 316. caption/subfig: `\clearcaptionsetup` and `\listsubcaptions` leave their star and options as text

caption3.sty's `\clearcaptionsetup` reads `*[option]{type}` (:275-280); subfig.sty's `\listsubcaptions` tests for a star (:476-479). caption.sty.ltxml:130 reads no argument, subfig.sty.ltxml:109 only `{}`, and subfig.sty.ltxml:105 no star.

Trigger: `\usepackage{subfig}\clearcaptionsetup[position]{subfloat}` — Perl: `position]subfloat` in the first paragraph; `\usepackage{caption}\clearcaptionsetup{figure}` — Perl: `figure` in the text; `\listsubcaptions*` in a figure — Perl: a `*` panel. pdflatex prints none of them. Rust fix (57j): `\clearcaptionsetup OptionalMatch:* []{}` in both bindings, `\listsubcaptions OptionalMatch:*`.

**Guard**: `perfect_kernel_batch56::{clearcaptionsetup_reads_its_options, caption_clearcaptionsetup_reads_its_type}`.

## 317. titlesec: `\titleline`, `\iftitlemeasuring` and `\wordsep` read differently from titlesec.sty

titlesec.sty's `\titleline` reads a star, `[align]` (default `s`) and the material (:1088-1095); `\iftitlemeasuring` is `\@secondoftwo`, a two-branch choice (:1047); `\wordsep` is the font's interword glue (:1164-1165). titlesec.sty.ltxml reads `\titleline[]{}` (:70), makes `\iftitlemeasuring` a TeX conditional (:75) and `\wordsep` a 0pt register (:68).

Trigger: the titlesec manual's `\titleformat{\section}[block]{\large\titleline*[c]{\titlerule*[.6pc]{\tiny\textbullet}}\normalfont}{\thesection}{1em}{}` (titlesec.tex:1779-1793) — Perl: `<title>[c]1   [c]Intro</title>`; `\titleformat{\section}{\normalfont\iftitlemeasuring{M}{\bfseries}}…` — Perl: "conditional fell off end" and `<title/>`; `\titleformat{\section}[runin]{\bfseries}{\thesection}{\wordsep}{}` — Perl: "1Intro". pdflatex: "1 Intro" each. Rust fix (57j): `\titleline OptionalMatch:* []{}`, `\let\iftitlemeasuring\@secondoftwo`, `\wordsep` as titlesec.sty's glue. Since 57o the title format runs once (#327) and `\titleline` prints its material as a line of the title (a rule-only material sets nothing).

**Guard**: `perfect_kernel_batch56::{titleline_reads_its_star_and_alignment, titleline_prints_its_material, iftitlemeasuring_takes_the_second_branch, wordsep_is_an_interword_space}`.

## 318. changepage (ar5iv binding): `{adjustwidth}` leaves its paragraph open; the page checks are stubs

changepage.sty's `{adjustwidth}` is a `\list` (changepage.sty:110-139), whose end closes the paragraph (`\endtrivlist`, latex.ltx:15915-15926); its margins are list parameters; `\checkoddpage`, `\cp@tempcnt`, `cp@cntr`, `\cplabel` and `[strict]` are the package's TeX (:22, 29-32, 59-67); under memoir the package stops (:8-11, memoir.cls:12216 `\EmulatedPackage`). The ar5iv-bindings changepage.sty.ltxml (:23, :28), which the Rust binding copied, made `{adjustwidth}` a transparent body without the closing `\par`, digested the margins (`\linewidth` in `{-0.005\linewidth}` became an assignment that set it to 0pt), stubbed every macro, overrode memoir's, and served chngpage (a different package, chngpage.sty) as an alias. Upstream Perl has no binding.

Trigger: `Before text \begin{adjustwidth}{1cm}{1cm} Inner text. \end{adjustwidth} After text.` — ar5iv Perl and Rust 57k: `<p>Inner text. After text.</p>`; pdflatex: "After text." a new paragraph. Witness 2605.02723 (a `width=\linewidth` figure at 0pt). Rust fix (57l): changepage.sty and chngpage.sty loaded raw under their bindings, which keep the environments transparent (DIVERGENCES #341) and close the paragraph before the end; nothing is loaded under memoir, whose `\checkoddpage` stays.

**Guard**: `perfect_kernel_batch56::{adjustwidth_ends_its_paragraph, adjustwidth_margins_are_read_not_typeset, changepage_page_checks_are_the_packages, changepage_under_memoir_keeps_memoirs, chngpage_is_its_own_package}`.

## 319. subfig: its `\captionsetup` stub replaces caption's

subfig.sty loads caption (subfig.sty:124-142: `\RequirePackage{caption}` by default, `{caption3}` with `caption=false`), so `\captionsetup*[type][subtype]{options}` (caption3.sty:244-265) and `\caption*` are caption's. subfig.sty.ltxml:107 defines `\captionsetup[]{}` as a no-op and loads nothing (its :18 "Needs RequirePackage('caption'); but not yet implemented"): loaded after caption, it replaces caption's working command; alone, `\caption*` is undefined.

Trigger: `\usepackage{caption}\usepackage{subfig}` … `\begin{minipage}{0.4\textwidth}\captionsetup{type=figure}\caption{Typed}\end{minipage}` — Perl: `Error: \caption outside any known float`; `\captionsetup*{labelfont=bf}` prints "labelfont=bf". Rust fix (57l): subfig requires caption first and declares its own caption keys (subfig.sty:163-167, 271-282).

**Guard**: `perfect_kernel_batch56::{subfig_keeps_captions_captionsetup, subfig_loads_caption}`.

## 320. setspace: the spacing environments do not end their paragraph

setspace.sty's `{spacing}`, `{singlespace}`, `{onehalfspace}` and `{doublespace}` all end with `\par` (setspace.sty:489-548, `\restore@spacing` :516-523); `{spacing}` and `{singlespace}` also begin one. setspace.sty.ltxml makes them transparent `#body` environments, so the text after one continues its last paragraph.

Trigger: `After. \begin{doublespace} Inner2. \end{doublespace} After2.` — Perl and Rust 57k: one paragraph; pdflatex: "After. Inner2." | "After2.". Rust fix (57l): each closes its paragraph at its end — `{spacing}`/`{singlespace}` in their own vertical mode, `{onehalfspace}`/`{doublespace}`, which go on in the paragraph before them, by closing the `<p>` after their body (a paragraph opened outside a group, ARCHITECTURE_THEMES 1).

**Guard**: `perfect_kernel_batch56::{setspace_environments_end_their_paragraph, setspace_environments_begin_as_setspace_does}`.

## 321. caption/rotating: `\rotcaption` is dropped or misread

caption.sty redefines `\rotcaption` as a caption only when rotating is loaded (caption.sty:1284); rotating.sty's reads `[short]{long}` (`\@dblarg`, rotating.sty:260-270). caption.sty.ltxml:131 defines it as a no-op whatever is loaded, replacing rotating's; rotating.sty.ltxml:164 reads `{}` only.

Trigger: `\usepackage{rotating}\usepackage{caption}` … `\begin{sidewaystable}…\rotcaption{Rotated caption}\end{sidewaystable}` — Perl: no caption, no number; `\rotcaption[Short]{Side caption}` with rotating alone — Perl: the caption "[" and a panel "Short]Side caption". Rust fix (57l): caption leaves `\rotcaption` to rotating, which reads `[short]{long}`.

**Guard**: `perfect_kernel_batch56::{rotcaption_is_a_caption, rotcaption_reads_its_short_form}`.

## 322. svg: the binding loads subfig

svg.sty loads iftex, scrbase, pdftexcmds, trimspaces, graphicx and shellesc (svg.sty:66-73), and xcolor/transparent only when a drawing needs them (:337-352) — no subfig. svg.sty.ltxml:19 does `RequirePackage('subfig')`, so every document with svg gets subfig's `\subfloat`, `\ContinuedFloat` and `\captionsetup` beside the ones it asked for; after subcaption, subfig's `\lx@subfloat@figure` is left undefined behind its `\@ifundefined{c@subfigure}` guard (subfig.sty.ltxml:114, #102) and `\subfloat`'s arguments print as text (brucemiller/LaTeXML#2563).

Trigger: `\usepackage{subcaption}\usepackage{svg}` … `\begin{figure}\subfloat[This is a caption.]{This is a figure.}\end{figure}` — Perl: `<p>[This is a caption.]This is a figure.</p>`. Witness 2605.17685 (which `\ContinuedFloat` was in force depended on load order). Rust fix (57l): svg loads no subfig; `\subfloat` is the loaded package's own (KPE #323).

**Guard**: `06_cluster_regressions::cluster_svg_subfloat_survives_subcaption_2563`.

## 323. subcaption: `\subfloat` reads a lone optional as the list entry

subcaption.sty's `\subfloat[list][caption]{body}` (subcaption.sty:278-291) goes through `\subcaptionbox`: a lone optional is the caption (`\subcaptionbox{#1}`), two are `\subcaptionbox[{#1}]{#2}`, and the sub-float follows `\@captype`. subcaption.sty.ltxml:104 defines `\subfloat[][]{}` with the caption from the second optional and always a `{subfigure}`, so `\subfloat[Caption]{body}` has an empty caption, and inside a `table` it is a subfigure.

Trigger: `\usepackage{subcaption}` … `\begin{figure}\subfloat[One]{A}\subfloat[List][Two]{B}\caption{Main}\end{figure}` — Perl: panel (a) uncaptioned; pdflatex: "(a) One", "(b) Two". Rust fix (57l): `\subfloat` dispatches on its optionals as subcaption.sty does and opens `sub\@captype` — a leading `sub` dropped (inside a `{subfigure}`), `figure` outside a float or for a type with no `sub<type>` environment (Perl's `\subcaption` guard, :50-53); witness 2111.00007 (Perl's own comment, :102) keeps its two sub-captions. Residuals (task list): without an optional subcaption sets a `\phantomcaption` (:293-300), here an empty caption; `\lx@subcaption@addinlist` (Perl :116-117) stores the list entry as the sub-float's `inlist`, a list name.

**Guard**: `perfect_kernel_batch56::{subfloat_reads_a_lone_optional_as_its_caption, subfloat_in_a_subfigure_is_a_subfigure}`, `06_cluster_regressions::cluster_svg_subfloat_survives_subcaption_2563`.

## 324. subcaption: a nested sub-float steps the float counter again

`beforeFloat` pre-increments the main counter for the first sub-float of a float (latex_constructs.pool.ltxml:3378-3381: `$type ne LAST_FLOATTYPE` and no main caption yet); `LAST_FLOATTYPE` is set only at `afterFloat` (:3391), so a `{subfigure}` opened inside another, before either ends, steps `figure` a second time.

Trigger: `\usepackage{subcaption}` … `\begin{figure}\begin{subfigure}{0.4\textwidth}\begin{subfigure}{\linewidth}Inner\caption{Inner}\end{subfigure}\caption{Middle}\end{subfigure}\caption{Outer}\end{figure}` — Perl and Rust: "Figure 2: Outer" (ids `S0.F2…`); pdflatex: "Figure 1: Outer". Not fixed (task list); pinned as is by `perfect_kernel_batch56::subfloat_in_a_subfigure_is_a_subfigure`.

## 325. natbib: `\bibpreamble` is never typeset

natbib's `thebibliography` runs `\bibpreamble` before its list (natbib.sty:1066), and packages hook it: apacite prints `\bibliographyprenote` and `\nocitemeta`'s "References marked with an asterisk indicate studies included in the meta-analysis." through it (apacite.sty:1835-1850). natbib.sty.ltxml:454 defines `\bibpreamble` empty and nothing typesets it, so a document's `\renewcommand{\bibpreamble}{…}` and apacite's notes are lost.

Trigger: `\usepackage[natbibapa]{apacite}\renewcommand{\bibliographyprenote}{Preamble note.}` … `\nocitemeta{smith2001}` before a `thebibliography` — pdflatex (two runs): "Preamble note.References marked with an asterisk …"; Perl: 5 errors (no apacite binding), both texts missing; Rust 57m: 0 errors, the asterisk printed, both texts missing.

Rust (Gemini round 13): the kernel `\thebibliography` digests `\lx@bibliography@preamble` (natbib's `\bibpreamble`) after its heading and before the list's `\bibitem` redirection (sect11.rs); apacite's `\bibpreamble` wrapper is apacite.sty:1835-1845 verbatim. A font switch in it reaches the entries, not the heading. Open: natbib's `\bibpostamble`, KOMA's `\setbibpreamble` without natbib. **Guard**: `perfect_kernel_gemini::bibpreamble_is_printed`.

## 326. OmniBus: `\doi{}` digests the DOI as TeX

OmniBus.cls.ltxml:154 reads `\doi{}` as a plain argument, so `_` is a subscript outside math, `~` a space, `%` a comment and `<…>` OT1's `¡…¿` — in the frontmatter, in the text and in the `href`. The classes that fall to OmniBus include apa6 and apa7, which define no `\doi`: their apacite `.bbl`s reach OmniBus's `\doi` (apacite's own is only `\providecommand`ed, apacite.sty:1812-1816).

Trigger: `\documentclass{apa7}\usepackage[natbibapa]{apacite}` … `\begin{APACrefDOI} \doi{10.1000/j_x~y.2002} \end{APACrefDOI}` in a `thebibliography` — Perl: `Error:unexpected:_`; pdflatex: "doi: 10.1000/j_x~y.2002". Rust fix (57m): a braced `\doi` reads `HyperVerbatim` as revtex4_1_cls.rs's does — `\lx@doi` a `Semiverbatim` (ASCII) argument, the frontmatter note set in the ASCII encoding — and a bare `\doi` its next token, as before (`HyperVerbatim` scans ahead to the next `{`). A command inside the braces prints as its name (`\allowbreak`), as url.sty's reading does in pdflatex.

A class binding whose real class defines its own `\doi` must define it too: pnas-new's is a setter for the footer (pnas-new.cls:422), so `\doi{\url{…}}` under OmniBus's printed "\urlwww.pnas…" (57p, pnas_new_cls.rs: the front matter's DOI a pubnote, a bibliography's `\doi` nothing, as the setter prints; witnesses 2605.03599, 2605.07504). Residual: jfm.cls:864 (`\gdef\@doi{10.1017/#1}`, 2605.01015) and interact.cls:245 (2605.01467) have setters their bindings (jfm_cls.rs, interact_cls.rs) do not define; no 2605 paper uses `\doi` with them.

**Guard**: `perfect_kernel_batch56::{omnibus_doi_reads_as_a_url, omnibus_frontmatter_doi_keeps_its_characters, bare_doi_reads_one_token, pnas_doi_is_the_classs_setter}`.

## 327. titlesec: a title format runs twice

titlesec runs `<format>` once, before the label, and sets label and title inside it (titlesec.sty:740-795 `\ttlh@display`/`\ttlh@hang`/`\ttlh@runin`, :797-818 `\ttlhx@block`). titlesec.sty.ltxml:50-57 defines `\format@title@font@<sec>` as the format and a composer that runs it before the label; the kernel's `\lx@format@title@@` (Base_Utility.pool.ltxml:1099-1101) applies `\format@title@font@<sec>` to the title again. Fonts applied twice look the same; visible material prints twice, and an alignment in the format is lost from numbered titles.

Trigger: `\titleformat{\section}[block]{\normalfont\bfseries XX}{\thesection}{1em}{}` — Perl and Rust 57n: `<title font="bold">XX1  XXIntro</title>`; `\titleformat{\subsection}{\raggedright\bfseries}…` — no alignment on the numbered title. pdflatex: "XX1 Intro", ragged right. Rust fix (57o): the composer runs the format from a copy and empties `\format@title@font@<sec>` for the rest of the title's group (a copy, so a format ending in `\MakeUppercase` does not take the `\let`); a shape's class goes before the format; a `hang`/`runin` format takes label, separator and title as one group (titlesec.sty:767, 788: `\MakeUppercase` uppercases them); an alignment in the format aligns the title itself (DIVERGENCES #344). The star form's default label is the class's `\@seccntformat` (amsart's `\@secnumpunct` is not in our binding: "1 Alpha" for pdflatex's "1. Alpha").

**Guard**: `perfect_kernel_batch56::{titlesec_format_runs_once, titleline_prints_its_material, titlesec_format_takes_label_and_title}`.

## 328. titlesec: `\titleformat*` drops the section number

`\titleformat*{cmd}{format}` replaces only the format (titlesec.sty:672-683). titlesec.sty.ltxml:30-34 redefines the whole composer as `<format> #1`, so the label is gone.

Trigger: `\titleformat*{\subsubsection}{\bfseries}` — Perl and Rust 57n: `<title font="bold">Deep</title>`; pdflatex: "1.1.1 Deep". Witness 2605.21802 (`\titleformat*{\section}{\large\bfseries}`, every section unnumbered). Rust fix (57o): after a `\titleformat`, the star form replaces its format only (keeping its shape class); on `\section`…`\subparagraph` never given one it builds titlesec's default (titlesec.sty:1541-1563: `\titleformat\cmd[runin or hang]{format}{\@seccntformat{cmd}}{0pt}`), so the format covers label and title as titlesec sets them.

**Guard**: `perfect_kernel_batch56::{titlesec_format_runs_once, titlesec_format_takes_label_and_title}`.

## 329. framed: the environments digest their body in restricted horizontal mode

Every framed.sty environment is `\MakeFramed` (framed.sty:113-167, 228-239), which begins with `\par` (:289) and sets its body in `\setbox\@tempboxa\vbox\bgroup` (:326) — internal vertical mode. framed.sty.ltxml:21-104 declares no mode, so `DefEnvironmentI` digests the body restricted horizontal (Package.pm:1902).

Trigger: `\begin{framed}First paragraph $$x^2$$ and more.\par\begin{minipage}{0.4\textwidth}Boxed text.\end{minipage}\end{framed}` — Perl and Rust 57n: "Script ^ can only appear in math mode", the minipage a stacked `<p>`; pdflatex: x² displayed, the minipage in a paragraph. Witness 2605.16567 (4 errors). Rust fix (57o): all eight environments `internal_vertical`, as the contrib `{mdframed}` (mdframed_sty.rs, witness 2402.07712).

**Guard**: `perfect_kernel_batch56::framed_body_is_vertical`.

## 330. subcaption: the sub-figure number carries the caption's parentheses

caption3's `\DeclareCaptionSubType` makes `\the<sub>` the bare letter and `\p@<sub>` the parent number (caption3.sty:1803-1806); the parentheses come only from the caption label format (subcaption.sty:218-222 `labelformat=parens`). subcaption.sty.ltxml:27-28 defines `\thesubfigure` as `(\alph{subfigure})`, so every `\ref` to a sub-figure reads "1(a)"; Perl's own golden `t/structure/subcaption.xml` encodes it.

Trigger: `\usepackage{subcaption}` … a `{subfigure}` with `\caption{Alpha}\label{a}`, then `See \ref{a}.` — Perl and Rust 57o: "See 1(a)."; pdflatex: "See 1a.", caption "(a) Alpha". 16 papers of the 57l+57m A/B showed it once svg stopped loading subfig (whose binding already had the bare letter). Rust fix (57p): `\thesubfigure`/`\thesubtable` are `\alph`; `\fnum@sub<type>` applies the sub-caption's label format to it (caption3.sty:730-739 — `\DeclareCaptionLabelFormat` now stores its formats; the `[sub<type>]` setting, else the `[sub]` one with subcaption's package options, else subcaption's `parens`, subcaption.sty:214-222), so `labelformat=simple` with an author's `(\alph{subfigure})` stays "(a)" (about 75 papers of 2605; 2605.01394); a counter subfigure.sty already declared keeps its numbers (subcaption.sty:226-230; 2605.01846). Residual: `\subref` prints the refnum "1a" where subcaption prints the bare "a" (RED `captions-floats/subref_prints_the_letter`).

**Guard**: `perfect_kernel_batch56::{subfigure_refnum_is_the_letter, subcaption_labelformat_option_keeps_the_authors_number, subcaption_labelformat_setup_keeps_the_authors_number, subfigure_before_subcaption_keeps_its_numbers}`; goldens `structure/subcaption`, `structure/figure_grids`, `complex/figure_mixed_content` re-blessed (refnums "1(a)" → "1a", list tags "(a)" → "a").

## 331. Figure panels: a merge puts the neighbouring panel inside a minipage's box

`arrange_panels_and_breaks` merges a panel eight times narrower (or wider) than the one before it into one block, and reuses an `ltx:block` as the container when either panel is one (latex_constructs.pool.ltxml:3303-3325: `$prev_node->appendChild($child)`, `$child->appendChild($prev_node)`). A minipage's or `\parbox`'s own block is such a block: the other panel goes inside a box TeX set at its declared width with only its own material.

Trigger: `\begin{figure}\begin{minipage}{0.7\textwidth}\includegraphics[width=\linewidth]{a}\end{minipage} \begin{minipage}{0.04\textwidth}(a)\end{minipage}\caption{Rows}\end{figure}` — Rust 57o/57p (text minipages panels at their width, DIVERGENCES #343): "(a)" inside the picture's block, under the picture; pdflatex: "(a)" beside the picture. Perl sets the folded label `<p>` as a standalone row and does not reach the merge here, but merges any `\includegraphics` into a following minipage the same way (KPE #274). Witness 2605.00042 S5.F4. Rust fix (57q): a node with the `ltx_minipage`/`ltx_parbox` class and a width (`box_panel_width`) is never the merge's container; the merge wraps the two panels in a new block, which is the panel (a wrapped row start loses its own panel class). Perl's wrap pops the last recorded panel (L3324), which is an earlier row's when the row started with a zero-width node; ours pops only the node it wraps. The witness is fixed in part: (a), (d), (f) sit beside their pictures; "(b)" and "(e)" still pair with the next row's picture, because the source `\par` between the rows is not seen (RED `captions-floats/par_breaks_a_panel_row`).

**Guard**: `perfect_kernel_batch56::panel_merge_keeps_boxes_closed`, `node_box_append::a_panel_merged_into_a_block_keeps_its_place`.

## 332. titlesec: the label's font runs into the title; before-code and run-in after-code are ignored

titlesec sets the label in a group of its own in every shape (titlesec.sty:752 `{#2\ttl@strut\@@par}`, :773 `\sbox\z@{#2…}`, :791, :810, :837, :879, :953, :1003) — on a line of its own for `display` and `frame` (:752, :837-846) — hands the title to the before-code, `#4{#8}` (`explicit`: the title is the before-code's `#1`, :296-299, :714-719), and runs the after-code in the heading's group after the title (:756, :780, :816, :862; `runin` in the same box, `#4{#8}#5\unskip`, :792; `rightmargin`, `wrap` and `drop` never use it, :901-1029). titlesec.sty.ltxml:42-57 composes `<format> <label> \hspace{<sep>} #1`, ignoring before- and after-code.

Trigger: `\titleformat{\chapter}[display]{\Huge}{\filleft\Large\chaptertitlename\ \thechapter}{0mm}{\filleft}` — Rust 57o/57p: `<title class="ltx_align_right" fontsize="144%">Chapter 1Introduction</title>` (the label's `\Large` sizing the title, no line between); Perl "Chapter 1Introduction" with the title at `\Huge` (the kernel re-applying the format, KPE #327). `\usepackage[explicit]{titlesec}\titleformat{\section}{\bfseries}{\thesection}{1em}{#1.}` — Perl and Rust 57p: "1 Intro"; pdflatex: "1 Intro.". Witness unamth-template tesis (TL doc; PhDthesisPSnPDF.cls:55-63); a 57q sample scan found 118 text-setting before-codes in 46 of 3,003 papers (93 `#1`, 13 `#1.`). Witnesses 2605.03501, 2605.04348, 2605.07586 (an explicit label holding `#1`, an empty before-code): Perl and Rust 57p printed the title twice, 57q once. Rust fix (57q): the composer groups the label, sets a display/frame label as a `\titleline` of its own, applies the before-code to the title (`{#1}`, or `{}` under `explicit`) and the after-code after it, except in the shapes that drop it (a run-in title ends `\unskip`): AVT.sty's `\list…`/`[\endlist]` `\part` closes, a `[\setcounter{equation}{0}]` resets (2605.06498, 2605.11886). A parameter reference (`#`, not an escaped `##`) in the label, before- or after-code without `explicit` raises TeX's "Illegal parameter number in definition of \ttlf@<sec>" and is the title (pdflatex's recovery prints "1 1Intro"). A `\titleformat*` under `explicit` stops pdflatex ("Missing \begin{document}"); the default composer then passes the title, as `\ttl@passexplicit` would. Residuals (SYNC_STATUS K13 (5)): an unnumbered title does not use the format; an alignment switch in the before-code aligns an inner `<text>` (DIVERGENCES #344 lifts only the format's); a `leftmargin` after-code, horizontal material at the start of the next paragraph in TeX (`\@svsechd`), ends the title; `explicit` given as a global class option is not seen (`\opt@titlesec.sty` holds the package's own options).

**Guard**: `perfect_kernel_batch56::{titlesec_display_label_is_a_line_of_its_own, titlesec_before_code_takes_the_title, titlesec_explicit_title_is_the_before_codes_argument, titlesec_after_code_follows_the_title, titlesec_before_code_parameter_needs_explicit, titlesec_before_code_inner_definition_keeps_its_parameters, titleline_prints_its_material, centred_title_format_keeps_its_size}`.

## 333. colortbl: `\rowcolor` does not read its overhangs

colortbl.sty:208-231: `\rowcolor[model]{color}` then `\CT@rowc` reads two optional overhangs `[left][right]` (the right defaulting to the left). colortbl.sty.ltxml:55 `\rowcolor[]{}` stops after the colour.

Trigger: `\rowcolor{gray!20}[0pt][0pt]\multicolumn{2}{l}{Group}\\ \rowcolor{gray!20}[2pt][2pt] c & d` — Perl: 4 errors (a misplaced `\omit`), "[0pt][0pt] Group", "[2pt][2pt] c"; Rust 57q: 1 error, the same text; pdflatex: "Group" / "c d". Witnesses 2605.22864, 2605.08915, 2605.04906. Rust fix (57r): `\rowcolor[]{}[][]`, the overhangs read by LaTeXML's optional reader as `\columncolor`'s (no `\lbrack` trap) and dropped as layout. They are not checked as dimensions: a row starting `[12]` after `\rowcolor{c}` loses "[12]" silently where pdflatex reports "Illegal unit of measure".

**Guard**: `perfect_kernel_batch56::rowcolor_reads_its_overhangs`.

## 334. hyperref: `\pdfstringdefDisableCommands` takes no argument

hyperref.sty:790-798: `\pdfstringdefDisableCommands` = `\begingroup\makeatletter\HyPsd@DisableCommands`, which appends its argument to `\pdfstringdefPreHook` for bookmark strings — never typeset. hyperref.sty.ltxml:403 defines it with no argument, so the group runs in place under the document's catcodes.

Trigger: `\pdfstringdefDisableCommands{\let\cite\@gobble}` in the preamble — Perl and Rust 57q: "gobbleBody."; pdflatex: "Body.". 36 of the 3,003 A/B papers call it. Rust fix (57r): the raw two-macro shape, `\pdfstringdefPreHook` provided empty.

**Guard**: `perfect_kernel_batch56::pdfstringdef_disable_commands_reads_its_argument`.

## 335. hyperref: `\hyperdef` has no `[label]`

hyperref.sty:4833 `\hyperdef{\@ifnextchar[{\label@hyperdef}{\@hyperdef}}`, :4864-4876 `\label@hyperdef[#1]#2#3#4`. hyperref.sty.ltxml:261 reads three `Semiverbatim`s, splitting `[lab]` into the category and name.

Trigger: `\hyperdef[lab]{cat}{nm}{Target text}` — Perl and Rust 57q: `<anchor xml:id="X.l">a</anchor>b]catnmTarget text`; pdflatex: "Target text". Rust fix (57r): the optional label is read first, verbatim (a label name: `sec_a` is no subscript), and an empty category names the anchor by its name alone (:4835-4839; Perl: "X.bare"). The label is not recorded: `ltx:anchor` carries no `labels` (LaTeXML-inline.rnc), so a `\ref` to it stays unresolved. Open, shared with Perl: a paragraph-initial `\hyperdef`/`\hypertarget` builds its anchor in vertical mode and the space after it is lost (RED `block-model/hyperdef_starts_the_paragraph`); mid-paragraph, `localized_anchor` wraps the running text before it (RED `block-model/hyperdef_anchor_holds_only_its_text`).

**Guard**: `perfect_kernel_batch56::hyperdef_reads_its_label`.

## 336. xcolor/color: `\fcolorbox` has no background model and splits the paragraph

xcolor.sty:822-826 `\fcolorbox#1#{\color@fbox{#1}}`, `\color@fbox#1#2#3#{…}`: `\fcolorbox[model]{frame}[bg-model]{bg}{text}` (xcolor.dtx `\fcolorbox[gray]{0.5}[wave]{580}{test}`), the background in its own model else the frame's; the box is `\color@b@x` (color.sty:163-164, xcolor.sty:827-829), an `\hbox`. xcolor.sty.ltxml:878 reads `[]{}{} Undigested` — no background-model slot — and, as color.sty.ltxml, sets the box `internal_vertical`: mid-paragraph the paragraph splits around it and both spaces are lost.

Trigger: `B \fcolorbox{red}[rgb]{1,1,0}{boxed} end.` — Perl: 1 error; Rust 57q: 1 error "Can't find color named '['"; both `<p>B</p><p>…boxed…end.</p>`; pdflatex: one paragraph "B boxed end.". Rust fix (57r): the `[bg-model]` slot, and both `\fcolorbox` bindings `restricted_horizontal` (enter_horizontal kept): the box stays in the paragraph with its spaces; a `\parbox` inside is unchanged.

**Guard**: `perfect_kernel_batch56::{fcolorbox_reads_the_background_model, fcolorbox_color_names_are_expanded_not_digested, fcolorbox_stays_in_the_paragraph}` (color.sty's and xcolor's).

## 337. xcolor: `\definecolor`/`\providecolor` have no `[prefix]`

xcolor.sty:519-522 `\XC@definec@lor[#1]#2[#3]#4#5`: an optional name prefix after `{name}`, default `\colornameprefix`; the same for `\providecolor` (:594-597), `\preparecolor` (:638-641) and `\xdefinecolor` (:593). xcolor.sty.ltxml:412/418 read the `[` as the model.

Trigger: `\definecolor{myred}[XC@]{rgb}{1,0,0}` — Perl: 2 errors; Rust 57q: 0 errors, "C@]rgb1,0,0" typeset; pdflatex: nothing. Rust fix (57r): `\definecolor[]{}[Default:\colornameprefix]{}{}`, `\providecolor[]{}[]{}{}`.

**Guard**: `perfect_kernel_batch56::definecolor_reads_a_name_prefix`.

## 338. etoolbox: `\AfterEndPreamble` in the body drops its code

etoolbox.sty:1740-1747 (a 2020+ format): `\AfterEndPreamble` is `\AddToHook{begindocument/end}`, a one-time hook, so code added after `\begin{document}` finished runs at once. etoolbox.sty.ltxml:1718 pushes it on a list nothing reads again.

Trigger: `A \AfterEndPreamble{Late}Z` — Perl and Rust 57q: "A Z"; pdflatex: "A LateZ". Rust fix (57r): once `\document` has taken the list, `\AfterEndPreamble` returns its code; Perl's `\AtEndDocument{\let\AfterEndPreamble\@gobble}` (etoolbox.sty.ltxml:1724, etoolbox.sty:1781 — after the `\endinput` a 2020+ format takes) is dropped, so code added at the document's end runs too. Open: `\document` unreads the hook code without latex.ltx's closing `\ignorespaces`, so a newline after `\begin{document}` stays a space after the hook's text (RED `macro-state/afterendpreamble_code_is_followed_by_ignorespaces`, RUST-ONLY); and `\document` fires `begindocument` and `begindocument/end` with `\hook_use:n`, not latex.ltx's `\UseOneTimeHook` (:9512, :9525), so `\AtBeginDocument` or `\AddToHook{begindocument…}` in the body is dropped (RED `macro-state/atbegindocument_in_the_body_runs_now`, shared) — the general fix, after which the 57r done flag goes.

**Guard**: `perfect_kernel_batch56::afterendpreamble_in_the_body_runs_now`.

## 339. enumitem: `\AddEnumerateCounter*` has no star

enumitem.sty:575-591 `\AddEnumerateCounter` is `\@ifstar\enit@addcounter@s\enit@addcounter`: the starred form registers a counter command whose argument is a counter (`\fnsymbol*`). enumitem.sty.ltxml:255 reads `{}{}{}`, so the star is the first argument and the third, the width sample, is typeset.

Trigger: `\AddEnumerateCounter*{\fnsymbol}{\@fnsymbol}{9}` — Perl and Rust 57r: a stray `<p>9</p>` before the list; pdflatex: nothing. Rust fix (57s): `\AddEnumerateCounter OptionalMatch:* {}{}{}` (a no-op: a starred counter label already becomes the item counter).

**Guard**: `perfect_kernel_batch56::addenumeratecounter_reads_the_star`.

## 340. lineno: `\modulolinenumbers*` has no star

lineno.sty:2151-2158 `\modulolinenumbers` is `\@ifstar`, then `[1][\z@]` (:2182). lineno.sty.ltxml:43 reads `[Number]`: the star and the option are typeset.

Trigger: `\modulolinenumbers*[5]` — Perl and Rust 57r: "*[5]Text."; pdflatex: "Text.". Rust fix (57s): `OptionalMatch:* [Number]`.

**Guard**: `perfect_kernel_batch56::modulolinenumbers_reads_the_star`.

## 341. lineno: `{numquote*}` is undefined; `{numquote}` prints its option

lineno.sty:2849-2867: `{numquote}`/`{numquotation}` are `\quote`/`\quotation` followed by `\numquotelist`, whose `\quotelinenumbers` reads a star or `[n]` as `\linenumbers` does; the starred environments exist too. lineno.sty.ltxml:58-61 makes both `\quote` and defines no starred forms.

Trigger: `\begin{numquote*}quoted\end{numquote*}`, `\begin{numquote}[5]numbered\end{numquote}` — Perl and Rust 57r: "The environment {numquote*} is not defined", "[5]" typeset; pdflatex: three numbered quotes. Rust fix (57s): the real definitions, over the binding's `\linenumbers`.

**Guard**: `perfect_kernel_batch56::numquote_star_is_a_quote`.

## 342. pifont: `\dingline`, `\dingfill`, `\Piline`, `\Pifill` are no-ops

pifont.sty:30-34: `\Pifill` fills the line with the symbol (`\leaders`), `\Piline` sets such a fill as a paragraph of its own; :57-58 `\dingfill`/`\dingline` are their `pzd` forms. pifont.sty.ltxml:49-50, 60-61 define no-ops.

Trigger: `Text \dingline{43} more \dingfill{51} end.` — Perl and Rust 57r: one paragraph "Text more end."; pdflatex: "Text" / a line of ☞ / "more ✓✓✓ end.". Rust fix (57s): the real macros; the kernel's `\leaders` gives an `ltx_leader` text (one symbol: the repetition is not rendered, LaTeXML.css has no `.ltx_leader` fill).

**Guard**: `perfect_kernel_batch56::dingline_is_a_line_of_its_own`.

## 343. rotating: `\turnbox` splits the paragraph

rotating.sty:100-107 `\turnbox` is `\leavevmode` then `\setbox\z@\hbox{{#2}}`: a box in the running paragraph. rotating.sty.ltxml:69-79 digests it `internal_vertical`.

Trigger: `Before \turnbox{30}{box} after.` — Perl and Rust 57r: `<p>Before</p><inline-block/><p>after.</p>`; pdflatex: one paragraph. Rust fix (57s): `restricted_horizontal` with `enter_horizontal`, as graphicx's `\rotatebox`. Open, shared: at a paragraph's start the box still stands before the paragraph it opens, as `\rotatebox`/`\scalebox`/`\resizebox` do (RED `boxes-groups/rotatebox_starts_the_paragraph`, the KPE #309 family).

**Guard**: `perfect_kernel_batch56::turnbox_stays_in_the_paragraph`.

## 344. amsmath: a bracket group at an `aligned`/`gathered` formula's start is lost

amsmath.sty:1441-1456 `\ams@start@box`: the optional of `aligned`, `alignedat` and `gathered` is a position only when its head-expanded text is `t`, `b`, `c` or empty (`\ams@pos@<…>`); any other bracket group is put back as the first cell's material (`\ams@return@opt@arg`, :1490, :1536) with the warning "Bracket group [..] at formula start!". amsmath.sty.ltxml:573/622/625 read the optional and drop it; `gathered` passed it as `vattach`.

Trigger: `\begin{aligned}[\alpha,\beta] &= \gamma\end{aligned}`, `\begin{gathered}[a,b] = c\end{gathered}` — Perl and Rust 57s: the first cells lose `[\alpha,\beta]` and `[a,b]` (gathered: `vattach="a"` and "unknown KeyVals key 'b'"); pdflatex: the brackets kept, two amsmath warnings. Witnesses 2605.04504, 2605.10596, 2605.11552, 2605.12210, 2605.18213 (×6), 2605.22557 (6 of 3,003 A/B papers, 11 formulas). Rust fix (57t): `\ams@start@box` as TeX (`\lx@ams@start@box`, amsmath_sty.rs), its `\romannumeral-`\0` head expansion included; the returned group opens the first cell, and `gathered` takes `vattach` from a position only.

**Guard**: `perfect_kernel_batch56::aligned_returns_bracket_group`.

## 345. physics: optional arguments are not found past spaces

physics.sty's optional arguments are ltcmd `s`, `t\ket`, `g` and `d()` (`\bra{ s m t\ket s g }`, `\outerproduct{ s m g }`, `\expectationvalue{ s s m g }`, `\derivative{ s o m g d() }`, physics.sty:418-570; `\@quantity{ t\big t\Big t\bigg t\Bigg g o d() d|| }`, :36): all look past spaces and put them back when the argument is absent (xparse.sty:161-165 `\__cmd_peek_nonspace:NTF`). physics.sty.ltxml:85-89 `phys_readArg` and :495 `readMatch(\ket)` read the next token.

Trigger: `\bra{a} \ket{b}`, `\ev{A} {\psi}`, `\ketbra{a} {b}`, `\dv{f} {x}` — Perl and Rust 57s: a bra and a ket, ⟨A⟩ψ, |a⟩⟨a| b, d/df x; pdflatex: ⟨a|b⟩, ⟨ψ|A|ψ⟩, |a⟩⟨b|, df/dx. Witness 2605.08402 (`\ketbra{\psi_i} {\psi_i}`). Rust fix (57t): one `phys_after_spaces` peek for the group/delimiter reads, the `\ket` test and every star.

Golden `complex/physics` re-blessed: its `\bra{\phi}\ket{\psi} \qq{as opposed to} \bra{\phi} \ket{\psi}` (from Perl's test suite) expected the space to stop the contraction, as old xparse did; TeX Live 2025 contracts both (`\bra{a} \ket{b}` and `\bra{a}\ket{b}` 20.13pt wide, `\bra{a}{}\ket{b}` 24.58pt).

**Guard**: `perfect_kernel_batch56::physics_optional_args_skip_spaces`.

## 346. amsmath: `\alignedat` reads `{n}` before `[pos]`

amsmath.sty:1518-1524 `\alignedat` is `\alignedat@a[#1][c]` then `\start@aligned{#1}{#2}`: the position comes before the column count. amsmath.sty.ltxml:625 `\alignedat{} alignsafeOptional` reads the count first.

Trigger: `\begin{alignedat}[t]{2} a &= b\end{alignedat}` — Perl and Rust 57s: the first cell `t]{2}a`; pdflatex: "a = b", top-aligned. Rust fix (57t): the optional first, then `{n}`, through `\ams@start@box` (KPE #344).

**Guard**: `perfect_kernel_batch56::alignedat_reads_position_first`.

## 347. mathtools: `\ArrowBetweenLines` and `\shortvdotswithin*`

mathtools.sty:1299-1322: `\ArrowBetweenLines*[\Updownarrow]` is a row of its own holding the arrow — `&&\quad<arrow>` starred, `<arrow>\quad` otherwise — after an empty row whose height a negative `\noalign` skip cancels; both rows are `\notag`ged unless `\in@{\@currenvir}{alignedat,aligned,gathered}`, a substring test that also holds in `align` and `gather`; :1338-1344 `\shortvdotswithin` is `\@ifstar` (`\vdotswithin{#2}&` starred, `&\vdotswithin{#2}` otherwise). mathtools.sty.ltxml:644-654 define `\ArrowBetweenLines` as empty and the starred forms as `\csname …*\endcsname`, unreachable from the source.

Trigger: `\\ \ArrowBetweenLines*[\Downarrow]`, `\shortvdotswithin*{=}` — Perl and Rust 57s: a cell `*[\Downarrow]` (Warning:unparsed_math), `\vdotswithin{*}` then `{=}c=d`; pdflatex: an arrow row, a vdots row. Rust fix (57t): the real shapes, with the same `\in@` test; the empty row is left out, so in `align`/`gather`, where pdflatex numbers it, later equation numbers run one lower per arrow (RED `alignment-bindings/arrow_rows_in_align_are_numbered`).

Golden `ams/mathtools` re-blessed: its `\ArrowBetweenLines` row now holds ⇕ (the later unnumbered ids shift by one).

**Guard**: `perfect_kernel_batch56::mathtools_starred_row_macros`.

## 348. mathtools: `{multlined}` takes its first optional as the position

mathtools.sty:677-688, 795-812 (`\MT_test_for_tcb_other:nnnnn`): each of `{multlined}`'s two optionals is the position when it is `t`, `c` or `b` and the width otherwise, in either order. mathtools.sty.ltxml:581-585 `\multlined[][]` takes the first as the position.

Trigger: `\begin{multlined}[4cm][b]…` — Perl and Rust 57s: `vattach="4cm"` and "Missing number (Dimension)"; pdflatex: bottom-attached, 4cm wide. Rust fix (57t): each optional sorted by `\ifcsname lx@mt@multlined@pos@<text>`, the text head-expanded (`\romannumeral-`\0`) as `\MH_if:w t#1` expands it, so `[\mypos]` holding `t` is the position.

**Guard**: `perfect_kernel_batch56::multlined_classifies_its_optionals`.

## 349. amsmath: `\cfrac[l]` reads `[` as the numerator

amsmath.sty:912 `\DeclareRobustCommand{\cfrac}[3][c]`: `[l]`/`[r]` place the numerator. amsmath.sty.ltxml:1113-1116 read two arguments.

Trigger: `x = \cfrac[l]{1}{2+\cfrac[r]{3}{4}}` — Perl and Rust 57s: numerator `[`, denominator `l`; pdflatex: a continued fraction. Rust fix (57t): `\lx@inner@cfrac [] InFractionStyle InFractionStyle`; the alignment kept in the reversion (MathML Core has no `numalign`).

**Guard**: `perfect_kernel_batch56::cfrac_reads_its_alignment`.

## 350. bm: `\bmdefine` is local and refuses an existing name

bm.sty:229 `\def\bmdefine{\DeclareBoldMathCommand[bold]}`, whose `\bm@define` (:307-316) `\xdef`s the command: global, overwriting. bm.sty.ltxml:22 uses `\newcommand`.

Trigger: `\newcommand\bx{x}\bmdefine\bx{x}` and `{\bmdefine\balpha{\alpha}}` — Perl and Rust 57s: `\bx` stays italic, "\balpha is not defined"; pdflatex: bold x, bold α. Rust fix (57t): `\bmdefine` and `\DeclareBoldMathCommand` define globally. `\bm@define` expands the body at definition, so `\DeclareBoldMathCommand{\nabla}{\nabla}` is bold over the old `\nabla`; our body stays unexpanded, so the meaning at that moment is first saved under a name of its own, `\lx@bm@saved@<n>@<name>`, new per definition (a plain `\gdef` recursed to Fatal:Stomach:Recursion; one shared name made a second self-referential definition save the first's wrapper and recurse the same way).

**Guard**: `perfect_kernel_batch56::bmdefine_is_global`.

## 351. physics: `\pmqty{…}` and its kin drop their body

physics.sty:70-73, 105-108: `\pmqty{m}`, `\Pmqty`, `\bmqty`, `\vmqty` and the small `\spmqty`, `\sPmqty`, `\sbmqty`, `\svmqty` put their argument in the matrix. physics.sty.ltxml:701-710 declare them `{}` without passing `#1` to `\lx@physics@mat`, which then reads the next group, or none, as the body.

Trigger: `\pmqty{a & b \\ c & d} = \bmqty{1 & 0}` — Perl: two empty matrices and 4 errors ("Expected an open delimiter", "Expected a Token, got undef" per matrix); Rust 57s: two empty matrices, no error; pdflatex: (a b / c d) = [1 0]. Rust fix (57t): the body is handed on as a group (`…{(}{)}{#1}`), so `\lx@physics@mat` reads it.

**Guard**: `perfect_kernel_batch56::physics_matrix_keeps_its_body`.

## 352. physics: a body-less `\mqty`/`\qty` is an error, not `()`

physics.sty's quantity and matrix commands take only ltcmd optionals (`\@quantity{ t\big t\Big t\bigg t\Bigg g o d() d|| }`, :36; `\@matrixquantity{ s g o d() d|| }`, :75): with none of them, `\mqty` prints `()` and pdflatex reports nothing. physics.sty.ltxml:116-118 `phys_readArg($gullet, 1, …)` (from `\quantity` :135, `\evaluated` :174, `\lx@physics@mat` :683) reports "Expected an open delimiter", and `\lx@physics@mat` then "Expected a Token, got undef".

Trigger: `\[ \mqty = x \]` — Perl: 2 errors; pdflatex: "() = x", 0 errors; Rust 57u: an empty matrix (`\qty`: empty braces), 0 errors. Rust fix (57v): a bare `\matrixquantity`/`\smallmatrixquantity`/`\quantity` prints `()` (its `tex=` reads `()`, the text printed). Guard `perfect_kernel_batch56::physics_bare_mqty_prints_parentheses`.


## 353. The list depth registers stay 0

latex.ltx's `\itemize`/`\enumerate` advance `\@itemdepth`/`\@enumdepth` inside their group, and the `\list` they open advances `\@listdepth` globally until `\endlist` (latex.ltx:15852, :15913); enumitem's inline lists advance `\@listdepth` too (enumitem.sty:1188-1191, :1254). Packages read them (enumitem's `list<depth>` keys, custom list macros). LaTeX.pool beginItemize (pool:1314) keeps its own levels and never touches the registers.

Trigger: `\begin{itemize}\item a[\the\@listdepth] \begin{enumerate}\item b[\the\@listdepth][\the\@enumdepth]…` — Perl and Rust 57t: every value 0; pdflatex: a[1], b[2][1], c[3][2], after the lists 0. Rust fix (57u): `begin_itemize` advances `\@itemdepth`/`\@enumdepth` for itemize/enumerate (inline forms included) and `\@listdepth` globally, popped by an `afterGroup` `\lx@listdepth@pop`, for every list but the kernel `\list` (which keeps its own), `\trivlist` and paralist's in-paragraph lists (`BeginItemizeOptions::inline`).

The restore at the list's end sets the saved value rather than decrementing, since a binding list closed by the kernel `\endlist` (nih/denselists `{Enumerate}{\Onumerate}{\endlist}`) has already decremented it.

Residuals: the itemize binding still labels by its own `@item` level, which counts a kernel `\list`, so an itemize inside `\begin{list}` gets `\labelitemii` (pdflatex `\labelitemi`; RED `list-structure/itemize_in_a_list_takes_the_first_label`); latex.ltx's `quote`, `quotation`, `verse` and `thebibliography` open a `\list`, but their bindings do not advance `\@listdepth`.

**Guards**: `perfect_kernel_batch56::list_depth_registers_follow_the_lists`, `list_depth_registers_across_list_kinds`, `list_depth_survives_an_endlist_close`.

## 354. enumitem: `\setlist` names and levels, replace and append, `list<depth>`

enumitem.sty:1674-1696 `\enit@setlist@i`: each entry of `\setlist[…]` is a list when `\enitdp@<entry>` is defined (the standard lists, `trivlist`, every `\newlist`; the inline lists run under their base list's name, :1796-1805, so `enumerate*` reads `\setlist[enumerate]`) and a level otherwise; lists default to `list`, levels to 0, and the keys are stored for every list at every level (`\enit@saveset`, :1597-1612, a local `\def` that `\setlist` replaces and `\setlist*` appends to). At a list's start enumitem applies `list`, `list<\@listdepth>`, `<name>` and `<name><level>` in turn (:977-980). `\setenumerate[1][0]` and kin default the level to 0 (:1700-1705), and only a counter command in `\enit@labellist` takes a star (`\alph*`; :573-598). enumitem.sty.ltxml:210-221 takes the first entry as the list and the rest as its levels, appends always, skips `list<depth>`, stores `\setenumerate{…}` under level "" and `[0]` under "0", and replaces every `*` in a label.

Trigger: `\setenumerate[0]{label=(\alph*)}`, `\setlist[itemize,description]{label=--}` (witnesses 2605.01646, 2605.00593) — Perl and Rust 57t: "1." and a bullet; pdflatex: "(a)", "–". `\setlist[2]{label=**}` — Perl and Rust: "enumiienumii". Rust fix (57u): `enumitem_is_list_name`, `enumitem_defaults_key`, the merge order, `\setlist` replace/`\setlist*` append, `[Default:0]` shorthands and `\setdisplayed`; (57y) an enumerate-type list's label is read by `\lx@enit@normlabel`, enumitem's own `\enit@normlabel` expansion (enumitem.sty:910-936), so only a counter command of `\enit@labellist` takes a star.

**Guards**: `perfect_kernel_batch56::enumitem_setlist_levels_and_names`, `enumitem_setlist_replaces_appends_and_depth`, `enumitem_setlist_in_a_group_is_local`, `enumitem_inline_list_reads_its_base_keys`.

## 355. `\varmathbb` is undefined under txfonts and newtxmath; fourier's `\mathbb` is not its own

txfonts.sty:920 and newtxmath.sty:2466-2467 define `\varmathbb` (and newtxmath `\vmathbb`) as blackboard alphabets of their own; fourier.sty:300-303 makes `\mathbb` its fourier-bb alphabet `\math@bb` at `\begin{document}`, undoing a preamble `\renewcommand{\mathbb}{\varmathbb}`. txfonts.sty.ltxml:18 and fourier.sty.ltxml define none of these.

Trigger: `\usepackage{amsmath}\usepackage{txfonts}\renewcommand{\mathbb}{\varmathbb}` then `$\mathbb{E}$` — Perl: "Error:undefined:\varmathbb"; Rust 57t: Fatal:Timeout:Recursion (its amsmath binding `\let` `\varmathbb` to the `\mathbb` autoload trigger, which amsmath.sty never defines, so the user's `\renewcommand` made the two expand into each other; RUST-ONLY); pdflatex: a blackboard E. Witnesses 1205.4484 (Fatal → 0 errors, 2,743 formulas), 2406.06884 (amsart + fourier; Fatal → its 2 unrelated errors). Rust fix (57u): the amsmath aliases removed; txfonts (and newpxmath) define `\varmathbb` as a blackboard constructor of its own (a copy of the current `\mathbb` would loop after an earlier `\renewcommand{\mathbb}{\varmathbb}`), newtxmath `\vmathbb`/`\vvmathbb` (:2577), pxfonts (which the binding builds on txfonts) keeps none, as pxfonts.sty; fourier defines `\math@bb` (reverting as `\mathbb`), and `\AtBeginDocument{\let\mathbb\math@bb}`. fourier-orns stays unloaded (a missing-file warning): a raw load prints its `futs` slot characters for want of a glyph map (RED `fonts-nfss/fourier_orns_ornaments_are_their_glyphs`).

**Guards**: `perfect_kernel_batch56::varmathbb_is_its_own_alphabet`, `varmathbb_survives_an_earlier_renewcommand`, `newtxmath_blackboard_variants`, `fourier_mathbb_is_its_blackboard`.

## 356. physics: the trig family takes a `{…}` argument

physics.sty:300-305 `\trigbraces{ m o d() }` (the trig functions, `\log`, `\ln`) takes only a `(…)` argument; `\opbraces{ m g o d() }` (`\exp`, `\det`, `\Pr`, `\tr`, `\Tr`, `\Res`) also takes `{…}`. physics.sty.ltxml:85-118 phys_readArg accepts `{` for every command, so `\sin{y}` gains parentheses the PDF lacks.

Trigger: `\sin{y}` — Perl and Rust 57t: sin(y); pdflatex: "sin y". `\sin[\ell] {e}^x` (witness 2605.20398) — Rust 57t: `(sin^ℓ)(e)^x`, a regression of 57t's space skip (KPE #345); pdflatex "sin[ℓ] e^x". Rust fix (57u): `phys_read_arg(required, braced, …)`; `\lx@physics@operatorP` passes `braced = false` for the `PHYS_TRIGBRACES` commands. Golden `complex/physics` re-blessed: `\sin[x]{\frac{X}{Y}}` is `sin[x]` then the fraction (Perl's golden: `(power@(sine, x))@(X / Y)`).

`\opbraces`' `g` argument prints between braces in pdflatex (`\det{M}` is "det{M}"), Perl prints parentheses; Rust fix (57v): `PHYS_OPBRACES` (exp, det, Pr, tr, Tr, Res) take `\{…\}` around a braced argument (guard `physics_opbraces_keeps_its_braces`); the long names sharing their bindings (`\trace`, `\Trace`, `\exponential`, `\determinant`) print braces too, where pdflatex takes no argument at all. Residuals: `\opbraces`' `[…]` is a bracketed argument, not a power, in physics.sty; `\PV{g}` (:276); `\rank`, `\erf`, `\trace`, `\Trace` (plain `\DeclareMathOperator`s), `\principalvalue` (`{g}` only) and the long kernel names (`\sine`, `\exponential`) still take `[…]`/`{…}`/`(…)`.

**Guard**: `perfect_kernel_batch56::physics_trig_takes_no_braced_argument`.

## 357. Generic messages drop the space after a control word

tex.web §262 print_cs: `\write` and `\message` print a multi-letter control word followed by a space, so `\PackageWarning{test}{B \noexpand\foo c}` logs "B \foo c". latex_constructs.pool.ltxml:5571-5587 make_message joins `ToString(Expand(...))`, which puts no space after a control word ("B \fooc").

Rust fix (57v): `make_generic_message` writes each expanded body with `writable_tokens` (the `\write` stringifier: current `\escapechar`, a macro parameter `#` doubled to `##` as pdflatex logs it), decoding byte-mouth runs as `Tokens`' Display does (CJKutf8 input).

**Guards**: `perfect_kernel_batch56::package_warning_keeps_the_space_after_a_control_word`, `package_warning_decodes_byte_mouth_text`.

## 358. An itemize inside a kernel `\list` takes the second-level bullet

latex.ltx:16068-16075 `\itemize` labels by `\labelitem\romannumeral\the\@itemdepth`, and only itemizes advance `\@itemdepth`; a kernel `\list` or `\trivlist` does not. latex_constructs.pool.ltxml:1639 `\list` calls `beginItemize('list')`, which counts in the `@item` family, so `\begin{list}{}{}\item x \begin{itemize}\item y\end{itemize}\end{list}` labels `y` with `\labelitemii` ("–"); pdflatex prints "•".

The same level is read by paralist's `[<label>]` (`setItemizationStyle` defines `\labelitem<@itemlevel>`; paralist.sty:281-282 names `labelitem\romannumeral\the\@itemdepth`) and by enumitem's level keys (`\setlist[itemize,1]`, which enumitem applies by `\@itemdepth`): inside a `\list`, `\begin{compactitem}[--]` prints "•" and `\setlist[itemize,1]{label=$\circ$}` misses; pdflatex "–" and "◦".

Rust fix (57y): `begin_itemize` labels an itemize by the `\@itemdepth` it advances (defining `\label@item<level>` as `\labelitem<depth>` when the two differ, for the `@item` counter only); the counter, and so every id, stays Perl's. A list's own label is defined after it (enumitem's `label=` block runs after `begin_itemize`), `set_itemization_style` names `\labelitem<\@itemdepth>`, and enumitem keys an itemize's level by `\@itemdepth`.

**Guards**: `perfect_kernel_batch56::{itemize_in_a_list_takes_the_first_label, itemize_label_follows_the_itemize_depth, paralist_label_follows_the_itemize_depth}` (repros in `list-structure/`).

## 359. A numbered `\list` and an enumerate inside it on the same counter loop forever

latex_constructs.pool.ltxml:1323 defines a list's id through the enclosing list's `\the<counter>@ID`, and the list's items' through its own. `\begin{list}{}{\usecounter{enumi}}\item x \begin{enumerate}\item y\end{enumerate}\end{list}` numbers both lists by `enumi` (the `\list` does not advance the enumerate level), so the inner definitions reference each other: Perl runs until its timeout, Rust stops at `Fatal:Timeout:PushbackLimit`; pdflatex prints "x", "1. y".

Rust: RED `list-structure/numbered_list_with_a_nested_enumerate_converts`. Direction: expand the outer id prefix once when a list begins (the outer item is fixed by then), keeping `list_id_chain_reaches`' inherited-counter test working on recorded values rather than macro bodies.

## 360. elsarticle's itemize counts as an enumerate

elsarticle.cls.ltxml:64 redefines `{itemize}[]` with `beginItemize('itemize', 'enum')` ("not even sure what the intended effect is"), so every itemize item is labelled by `\labelenum<level>`: "1." at the first level, "(a)" nested, and an itemize nested in an enumerate takes the enumerate's second level. elsarticle.cls:1143-1150 labels an itemize by `\labelitem\romannumeral\the\@itemdepth`, like the kernel; pdflatex prints "•" and "–".

Rust (57y, DIVERGENCES #346): the itemize counts in `@item`. Its ids are the kernel itemize's (Perl's for `article`).

**Guard**: `perfect_kernel_batch56::elsarticle_itemize_has_bullets` (repro `list-structure/elsarticle_itemize_has_bullets`).

## 361. `\textcircled` circles the source of its argument

latex_constructs.pool.ltxml:5399 builds the circle from `ToString($arg)`, the argument's raw tokens: `\textcircled{\small{2}}` is "\small{2}⃝", and an enumitem label `\textcircled{\arabic*}` prints "\arabicenumi⃝" (enumitem.sty.ltxml:114-119 puts one `enumi` token for the star). `\textcircled` is a text accent (latex.ltx:10057, :14449-14450 `\UseTextAccent`) that typesets its argument; pdflatex prints a circled 2 and ①.

Rust fix (57y): after leaving vertical mode (`\hmode@bgroup`, omsenc.def:62), the argument is digested in a group and the circle holds its text (`\small{2}` → ②, `\arabic{enumi}` → ①); the enclosed-alphanumeric table and the U+20DD fallback are unchanged. Residual: a box or math argument's text is its TeX reversion, so `\textcircled{\raisebox{-0.9pt}{1}}` still circles its source (RED `fonts-nfss/textcircled_of_a_box_circles_its_text`). Witness latex-via-exemplos (`label={\large\protect\textcircled{\normalsize\arabic*}}`).

**Guard**: `perfect_kernel_batch56::textcircled_circles_its_typeset_argument` (repro `fonts-nfss/textcircled_circles_its_typeset_argument`).

## 362. caption's `\ContinuedFloat` is a no-op

caption.sty.ltxml:91 defines `\ContinuedFloat` as `Tokens()`: a figure continued over several floats takes a new number for each part and restarts its sub-float letters — refnums 1, 1a, 1b, 2, 2a, 2b, 3 where pdflatex prints "(a) (b) Figure 1", "(c) (d) Figure 1", "Figure 2" (caption.sty:496-536). Trigger: two `figure`s with `\subfloat`s under caption + subcaption, the second starting `\ContinuedFloat`.

Rust fix (57ad): `\continuedfloat[*]` counts the parts in `continuedfloat` (restarted at every real step of the current float's own counter through the `\lx@float@stepped` hook, caption's `\caption@reset@continuedfloat`, :501-503, :577-579; a sub-float's counter restarts nothing), leaves one global pending continuation of its type (`lx@float@continued`, caption's `caption@flags`, :173-192) under which the float's next step keeps the number without resetting the sub-counters (`latex_constructs::step_float_counter`; caption's `\caption@setcontinued`, :192, then `\caption@@refcounter`, :557-568), and suffixes the ids once (`\@alph\c@continuedfloat`); a continuation after another float type is caption's error. Wherever caption runs `\caption@settype` (:300-303) — the begin of a float built by `\@xfloat` (`\caption@xfloat`, :271-275), `\captionof`, `\captionsetup{type=…}` and a longtable's caption (:1167) — the type becomes the current float's and a pending continuation is cleared (`latex_constructs::begin_float_continuation`); floats caption never types — subfig's, subfigure.sty's and subfloat.sty's sub-floats, listings, an uncaptioned longtable — leave it pending, and their own counters step plainly (`before_untyped_float`). subfig's `\ContinuedFloat` calls `\caption@ContinuedFloat` (subfig.sty:581-590). Residual: a minipage `\subcaption` does not pre-step the main counter as caption's `\caption@subtypehook` does (:642), so a continued part's sub-captions restart at (a) (RED `captions-floats/continuedfloat_minipage_subcaption_continues_the_letters`). Witness 2605.17685.

**Guard**: `perfect_kernel_batch56::{continuedfloat_keeps_the_number_and_the_letters, continuedfloat_suppresses_one_step_per_real_float, continuedfloat_counts_only_the_main_floats_steps, continuedfloat_scope_opens_where_caption_sets_the_type, continuedfloat_captionof_wrapper_does_not_leak}` (repro `captions-floats/continuedfloat_keeps_the_number_and_the_letters`).

## 363. A longtable's `\caption*` loses its text

longtable.sty.ltxml:127-129 defines the longtable `\caption` as `\lx@longtable@caption[]{}`, with no star: `\caption*{Text}` reads `*` as the caption and leaves `{Text}` as a stray group, so the table's caption is `Table N: *` and the text is gone. longtable.sty's `\LT@c@ption` takes the star (an unnumbered caption, the counter still stepped at the table's begin). Trigger: `\begin{longtable}{l}\caption*{LT star}\\ x\\\end{longtable}`. The Rust binding copies Perl's definition (`longtable_sty.rs`).

**Repro**: `captions-floats/longtable_starred_caption_keeps_its_text` (RED).

## 364. Leading digits keep `\mathrm` letters from joining

Base_XMath.pool.ltxml:443-458 walks back from the last letter while `getNodeFont->equals` holds and the text read is `/^[0-9a-zA-Z]+$/`, then joins only a run that starts with a letter. A digit's font (a mathchar's fontinfo merged into the current font) is `equals` to the `\mathrm` letters' font, so `10\mathrm{log}_{10}` reads "10log", is refused, and stays `l * o * g` (also `2\mathrm{skew}`, `10\mathrm{GeV}`). Trigger: `$2\mathrm{KL}$` → Perl `2`, `K`, `L`.

Rust (57ae): the letters after the leading digits join (OXIDIZED_DESIGN #348). Witness 2605.31599; golden `tests/parse/math_lexemes.tex#letters_ligature_reads_back_through_a_digit`.

## 365. `,\quad` in a gathered row is two punctuations

MathParser.pm `filter_hints` (:417-491) turns a hint of 10pt or more into a virtual PUNCT unless the node before it is already one, and tests that by the node's own `role` (:482-483); a hint after an OPEN waits for the next node, again by the node's own `role` (:445). A gathered/split/multline row's content branch is XMRefs (`rearrangeAMSSplit` → `createXMRefs`), which carry no role, so the `\quad` after a `,` becomes a second punctuation, and the `\quad` after a `(` a punctuation inside the fence; either leaves the whole formula unparsed, where inline the `,` takes the `\quad` as its padding and the `(` passes it on. Triggers: `\[\begin{gathered}a=b,\quad c=d\end{gathered}\]` → Perl `a@=@b@,@quad@c@=@d` (unparsed), inline `formulae@(a = b, c = d)`; `\[\begin{gathered}(\quad x)\end{gathered}\]` → Perl `(@quad@x@)` (unparsed), inline `x`.

Rust (57af): the tests read the realized role (OXIDIZED_DESIGN #349). Golden `tests/parse/aligned_content_branch.tex#content_branch_reads_its_delimiters`.

## 366. ams_support's `\authors`, `\shortauthors`, `\addresses` read an argument

ams_support.sty.ltxml:82-84 defines `\authors{}`, `\shortauthors{}` and `\addresses{}` as one-argument no-ops. In amsart they are parameterless storage macros that `\author[#1]{#2}` accumulates, joined by `\and` (amscls/amsart.cls:460-477; `\@dblarg`: an absent short form is the full name, an explicit `[]` adds none): a document's `\maketitle` or running head prints them. Perl's read the next token, so `Written by \authors\ (short: \shortauthors).` is "Written by (short: ." with 4 errors, and 2605.03453's `\authors` inside a tabular swallowed the `\end`. Trigger: `\documentclass{amsart}\author[A.~Author]{Ann Author}\begin{document}\authors\end{document}`.

Rust fix (57ah): the AMS classes (ams_core, amsbook) install amsart's storage (`ams_support_sty::amsart_author_storage`); amsart's `\maketitle` keeps `\and` (amsart.cls:599-621), so they empty the kernel title code's clearing step (`\lx@maketitle@clear@and`, article's `\global\let\and\relax`), and a document's own `\and` survives the title (57ai); amsart's list joiners (`\nxandlist`, `\andify`, `\author@andify`, amsart.cls:580-598, :803-807) come with the storage, so a derived class printing `\authors` (resphilosophica.cls:323) reads "A, B, and C" (57ai). Not modelled: `\maketitle`'s rewrite of `\shortauthors` for the running head (`\andify`, or the short title when empty, amsart.cls:604-606). ams_support alone (the `\curraddr`/`\subjclass` autoloads in other classes) keeps Perl's setters. `\addresses` stays empty: LaTeXML builds the addresses from `\address`. The text is pdflatex's.

**Guard**: `perfect_kernel_batch57::ams_authors_are_storage_macros` (repro `sectioning-frontmatter/ams_authors_are_storage_macros`).

## 367. varioref's macros read none of their optionals

varioref.sty.ltxml:24-27 reads `\vref`, `\vpageref`, `\vrefrange` and `\vpagerefrange` as `OptionalMatch:*` plus labels only; `\fullref` and `\reftextfaraway` take no argument (:49, :57), and `\reftextlabelrange`/`\reftextpagerange` use `#2`/`#3` in two-parameter macros. The real macros read `\vref*[text]{l}`, `\vpageref*[here][far]{l}`, `\vrefrange[here]{a}{b}`, `\vpagerefrange*[here]{a}{b}`, `\fullref{l}` and `\reftextfaraway{l}` (tools/varioref.sty:803-966, :123-125). `\vref[here]{sec:a}` is a ref to the label `[` followed by the text "here]sec:a"; `\fullref{sec:a}` prints "sec:a". pdflatex: "1 here", "1 on page 1".

Rust fix (57ah): the real signatures (`varioref_sty.rs`). The bodies stay Perl's page-less `\ref`: the page text they would add is layout-relative. `\vrefrange` prints `\reftextlabelrange`, as `\vrefrangedefaultformat` does, and the range keeps Perl's language-neutral dash (varioref's "to"/"bis"/… come from its per-language tables, which the binding lacks).

**Guard**: `perfect_kernel_batch57::varioref_reads_its_optionals` (repro `singletons/varioref_reads_its_optionals`).

## 368. attachfile does not load hyperref

attachfile.sty.ltxml:19-22 requires keyval, ifpdf, calc and color; attachfile.sty:40 also runs `\RequirePackageWithOptions{hyperref}`. A document that loads only attachfile and writes `\href` or `\autoref` gets `Error:undefined:\href` and `Error:undefined:\autoref`. Trigger: `\usepackage{attachfile}` then `\href{https://example.org}{site}`.

Rust fix (57ah): the binding requires hyperref with its options (`attachfile_sty.rs`). A later `\usepackage[…]{hyperref}` then loads nothing and its options are dropped, as in pdflatex (which raises an option clash).

**Guard**: `perfect_kernel_batch57::attachfile_loads_hyperref` (repro `singletons/attachfile_loads_hyperref`).

## 369. placeins' `\FloatBarrier` does not end the paragraph

placeins.sty.ltxml:24 defines `\FloatBarrier` as empty. It begins with `\par` (placeins.sty:30), so the text around a `\FloatBarrier` line is two paragraphs in pdflatex and one in Perl. Trigger: `First.\n\FloatBarrier\nSecond.`

Rust fix (57ah): `\FloatBarrier` is `\par` (`placeins_sty.rs`); the float flushing has nothing to do, since floats stay where found.

**Guard**: `perfect_kernel_batch57::floatbarrier_ends_the_paragraph` (repro `singletons/floatbarrier_ends_the_paragraph`).

## 370. `\MakeUppercase` reads no locale optional

latex_constructs.pool.ltxml:5914-5935 declares `\MakeUppercase`, `\MakeLowercase` and `\MakeTitlecase` with `[1]`. The kernel's read `O{} +m` (latex.ltx:22367-22378), the optional being the locale keys (`lang=`); textcase lets `\MakeTextUppercase` to them. `\MakeUppercase[lang=en]{word}` is "[lang=en]word"; pdflatex prints "WORD".

Rust fix (57ah, 57ai): e-TeX-protected inner commands (`\cs_new_protected`, latex.ltx:22380-22394) behind the kernel's expandable fronts, which re-brace what they read, so `\MakeUppercase\foo` takes `\foo` whole and any `\edef` keeps the call (`latex_constructs/sect13.rs`; the pre-2022 `\protected@edef\MakeUppercase#1{…}` wrappers read `[` as the argument). The locale's own mappings (Turkish dotted i) are not modelled.

**Guard**: `perfect_kernel_batch57::case_changers_read_their_locale` (repro `singletons/case_changers_read_their_locale`).

## 371. supertabular reads no position optional

supertabular.sty.ltxml:24, :40, :60, :70 read the column template directly after `\begin{supertabular}` (`{supertabular*}{width}`, and the `mp` variants). Every variant first reads a `[pos]` it then ignores (supertabular.sty:352-400), so Perl takes `[t]` for the template, and each `&` of the row is `Error:unexpected:& Extra alignment tab '&'`. Trigger: `\begin{supertabular}[t]{ll} a & b \\ \end{supertabular}`.

Rust fix (57ah): the `[]` is read and dropped (`supertabular_sty.rs`).

**Guard**: `perfect_kernel_batch57::supertabular_reads_its_position` (repro `alignment-bindings/supertabular_reads_its_position`).

## 372. todonotes' `\listoftodos` prints its heading

todonotes.sty.ltxml:38 defines `\listoftodos` with no argument; todonotes.sty:323 reads `[1][\@todonotes@todolistname]`, the list's heading. `\listoftodos[My Notes]` prints "[My Notes]".

Rust fix (57ah): `\listoftodos[]` (`todonotes_sty.rs`). The list itself is not built, as in Perl.

**Guard**: `perfect_kernel_batch57::listoftodos_reads_its_heading` (repro `singletons/listoftodos_reads_its_heading`).

## 373. relsize's `\mathlarger` sizes the rest of the formula

relsize.sty.ltxml:45-46 defines `\mathlarger`/`\mathsmaller` as `\relsize{±#1}` with an optional step: an open size declaration that runs on to the end of the formula's group. The real macros read their atom, gather the `\limits`/`\nolimits` and scripts after it and size only that, in a group (relsize.sty:263-310). `$\mathlarger{\sum}_{i} x_i + y$` has x, + and y at 120%; pdflatex enlarges only the ∑ and its script. Trigger: `$a \mathsmaller{b} c$` (c at 83%).

Rust fix (57aj): relsize's collector verbatim, the choice step Perl's `\relsize{±1}` where relsize picks a `\mathchoice` of styles (`relsize_sty.rs`); the parse is unchanged.

**Guard**: `perfect_kernel_batch57::mathlarger_sizes_only_its_atom` (repro `fonts-nfss/mathlarger_sizes_only_its_atom`).

## 374. caption's `\captionsetup` never runs a declared key

caption.sty.ltxml stores `\captionsetup`'s keys without running them, so a key declared with `\DeclareCaptionOption{mya}[yes]{\def\myA{#1}}` never runs its code: `\captionsetup{mya}` leaves `\myA` undefined (2 errors). caption's `\captionsetup` sets them (`\caption@setkeys{caption}`, caption3.sty:244-259). pdflatex prints "Values: yes, bee." for the repro.

Rust fix (57aj): a key `\DeclareCaptionOption` declares is recorded and an untyped `\captionsetup` runs its keyval macro with the value (`caption_sty.rs`); a typed `\captionsetup[type]`/`[type][sub]` only stores its keys for that type, as caption's `\caption@setup@options` (:252-262) — bicaption's `\captionsetup[bi-second]{bi-second}` (sjtuthesis.cls:730-735, cquthesis.cls:341-350) must not rename every figure. Not modelled: a starred key is kept after its package ends (caption3.sty:221-224 undefines it). The Rust binding also loaded no keyval (caption3.sty:209, `Error:undefined:\define@key`) and its `\DeclareCaptionOption*` branch gobbled its own helper, printing the key name and code (RUST-ONLY, fixed with it).

**Guard**: `perfect_kernel_batch57::declared_caption_option_runs_its_code` (repro `captions-floats/declared_caption_option_runs_its_code`).

## 375. siunitx's `\SI` reads no pre-unit

siunitx.sty.ltxml:1150 reads `\SI[opts]{number}{units}`; siunitx reads `O{} m o m` (siunitx.sty:9535), the optional between the number and the units a pre-unit printed before the quantity. `\SI{10}[\$]{\per\kilo\gram}` takes `[\$]` for the units and prints "10 [ $]/" with the rest as text (2 errors); pdflatex prints "$10 kg⁻¹".

Rust fix (57aj): the pre-unit is read and set before the quantity in the same formula (`siunitx_sty.rs`).

**Guard**: `perfect_kernel_batch57::siunitx_si_reads_its_pre_unit` (repro `singletons/siunitx_si_reads_its_pre_unit`).

## 376. siunitx's qualifier modes drop the prefix's name, and `\of` loses its text

siunitx.sty.ltxml:986-993 folds a qualifier into the unit in the `phrase`/`space` qualifier modes, then clears `$pre` before the unit's name is built, so `\kilo\gram\polymer` under `qualifier-mode=phrase` has `meaning="gram"` (the prefix gone from the name, kept in the presentation). And `\of{sample}` gives an empty subscript: its argument is not the qualifier's presentation. Trigger: `\sisetup{qualifier-mode=phrase}\si{\kilo\gram\polymer}`.

Rust (57aj): the qualifier modes are Perl's, the naming quirk kept (the meaning follows Perl's); `\of{sample}` shows g with the subscript "sample" (the argument is the qualifier's presentation, `six_convert_units_from_tokens`).

**Guard**: `perfect_kernel_batch57::siunitx_qualifier_and_highlight_apply` (default mode); golden `complex/si` (every mode).

## 377. A sum of outer products with a coefficient reads a braket across the sum

Perl's `LANGLE ketExpression MIDBAR maybeBra` (MathGrammar:305-306, :373-393) takes the bra's `\langle 0|` and the next
ket's `|1\rangle` as one quantum-operator product whose middle is the signed term between them:
`\rho=p|0\rangle\langle 0|+(1-p)|1\rangle\langle 1|` reads `rho = p * ket@(0) * quantum-operator-product@(0, + (1 - p), 1)
* bra@(1)` — a sum of two projectors read as one matrix element (0 errors). The bra·…·ket product the author wrote, a sum
of ket·bra outer products, is lost.

Rust (57ap): the same reading, with plain bars and — since the sided bars divide as `|` does — with sided ones
(`\left|0\right\rangle\left\langle 0\right|+(1-p)\left|1\right\rangle\left\langle 1\right|`: `p * ket@(0) * …`; until
57bp the unknown was applied by divergence #18, which now skips Dirac brackets); 32 formulas in 18 papers of 2605 (2605.00091, 2605.02774, 2605.03468) that were
unparsed now read so. A beyond-Perl reading would refuse a braket middle that starts with a sign.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the outer-product row).

## 378. A declaration scope that is not a counter warns twice in Perl's own code

`getDeclarationScope` (latexml.sty.ltxml:549-556) tests every scope with `LookupRegister("\c@<scope>")`, which warns
for a control sequence that is no register (Package.pm:1364-1368): `\lxDeclare[scope=label:nope,role=FUNCTION]{$q$}`
logs `Warning:expected:register The control sequence \c@label:nope is not a register` — for every `label:`/`id:` scope,
the documented forms, and `scope=bogus` alike. And `getLabelID` (Rewrite.pm:49-55) reports a scope label no element
carries and returns undef, which the `scope` clause concatenates into its XPath (Rewrite.pm:302): besides
`Error:misdefined:<rewrite> No id for label nope in Rewrite` Perl logs `Warning:uninitialized:value Use of uninitialized
value value in concatenation (.) or string at …/Rewrite.pm line 302`. Both warnings are Perl-internal artifacts.

Rust (57aq): the errors (`No id for label …`, `Unrecognized scope pattern …`) once per conversion (OXIDIZED_DESIGN #357);
neither warning.

**Guard**: `perfect_kernel_batch57::declaration_scopes_resolve_as_perl`.

## 379. A declaration after a bare `\refstepcounter` applies nowhere

`getDeclarationScope` takes the counter `\refstepcounter` last stepped (`current_counter`, Package.pm:776) and names its
unit `id:` + `\the<counter>@ID`; a counter no element carries that id for leaves the declaration nowhere, not even where
it stands: `\newcounter{foo}\refstepcounter{foo}\label{foo1}\lxDeclare[role=ID]{$k$}$k$` leaves `k` UNKNOWN (0 errors,
0 warnings: a silent loss). (An amsthm `proof` steps no counter, `\@proof`, amsthm.sty.ltxml:147-172: its declaration is
the section's.)

Rust (57aq): the same (the port is faithful); a surpass would fall back to the nearest unit that carries an id.

**Guard**: golden `tests/parse/declaration_scope.tex` (the bare-refstepcounter section; the proof section beside it).

## 380. amsart: `\uppercasenonmath` is undefined

amsart.cls:405-426 defines `\uppercasenonmath\x`, which upper-cases a macro's text outside math; the class's title and
running-head code and derived classes call it. Neither ams_core.cls.ltxml nor ams_support.sty.ltxml defines it.

Trigger: `\documentclass{amsart}` … `\makeatletter\def\x{Title $x$ here}\uppercasenonmath\x\makeatother T: \x.` —
pdflatex "T: TITLE x HERE."; Perl: `Error:undefined:\uppercasenonmath`.

Rust (Gemini round 13): amsart.cls:405-426 ported (`amsart_uppercase_nonmath`, ams_support_sty.rs; amsart, amsbook,
amsproc). Open (SHARED): the class switches to `\altucnm` when textcase is loaded (amsart.cls:427-430), and there the
title empties (RED `sectioning-frontmatter/amsart_uppercasenonmath_textcase_keeps_the_title`). **Guard**:
`perfect_kernel_gemini::amsart_uppercasenonmath_is_defined`.

## 381. `\PackageWarning` re-expands `\unexpanded` text

latex.ltx's `\GenericWarning` writes its text with `\immediate\write` (latex.ltx:8773-8799), whose expansion keeps an
`\unexpanded{…}` or `\the\toks` result and a protected macro as they are. latex_constructs.pool.ltxml:5586-5587
(`make_message`) runs `ToString(Expand(…))`, a full expansion, so the text is expanded again.

Trigger: `\PackageWarning{test}{\unexpanded{\foo x ## y}}` in the preamble — pdflatex "Package test Warning: \foo x
#### y"; Perl: `Error:undefined:\foo`.

Rust (Gemini round 13): the message text is expanded as `\edef` expands (`do_expand_partially`, base_utilities.rs
`make_generic_message`), for every `\Generic*`/`\Package*`/`\Class*`/`\@latex@*` message (DIVERGENCES #369). **Guard**:
`perfect_kernel_gemini::package_warning_keeps_unexpanded_text`.

## 382. `\hyperdef`/`\hypertarget` anchor the words before them

hyperref's `\hypertarget{name}{text}` anchors its own text (hyperref.sty:4834-4845, `\hyper@@anchor{…}{#3}`). The
binding's `localized_anchor` (hyperref.sty.ltxml:238-258) walks from the insertion point for the first node an anchor
may hold and wraps it; mid-paragraph that is the paragraph's running text, so the anchor takes the words before it.

Trigger: `A \hyperdef{cat}{nm}{Target} b. \hypertarget{tt}{T3} d.` — Perl `<anchor xml:id="cat.nm">A
Target</anchor><anchor xml:id="tt"> b. T3</anchor> d.`.

Rust (Gemini round 13): where the insertion point admits an anchor and the text is horizontal material, the anchor holds
exactly the text (`anchor_own_text`, hyperref_sty.rs); display material and vertical contexts keep Perl's walk
(DIVERGENCES #370). Open (SHARED): a target heading a paragraph drops the space after it (RED
`block-model/hypertarget_heading_a_paragraph_keeps_the_space`). **Guard**:
`perfect_kernel_gemini::hyperdef_anchor_holds_only_its_text`.

## 383. babel-french keeps the typed space before high punctuation

french.ldf's active `;:!?` remove the space typed before them in horizontal mode and put their own thin space
(french3.ldf:277-318, `\ifdim\lastskip>1sp\unskip\penalty\@M\FBthinspace`). LaTeXML's french binding adds the thin space
and keeps the space.

Trigger: `\usepackage[french]{babel}` … `Mid bold ; suite.` — pdflatex one thin space before `;`; Perl U+0020 U+2006.

Rust (Gemini round 13): `unskip_before_high_punct` (french_ldf.rs) drops a space box or skip before the punctuation.
Residual: a space inside a closed group (`{\bfseries gras }?`) stays, and a negative skip is removed where TeX removes
only one above 1sp. **Guard**: `perfect_kernel_gemini::french_high_punctuation_unskips_the_space` (witness
matapli-doc).

## 384. A bare `\subfloat{…}` prints a caption; `\phantomcaption` is a no-op

subcaption.sty:293-300 and subfig.sty:348-349 give a `\subfloat` with no optional argument a phantom caption: the
counter steps, nothing is printed. caption.sty:392-395's `\phantomcaption` is `\caption@refstepcounter\@captype`. The
bindings print an "(a)" caption for the bare form and stub `\phantomcaption`.

Trigger: `\usepackage{subcaption}` … `\begin{figure}\subfloat{\rule{1cm}{1cm}}\subfloat[]{…}\caption{Main}\end{figure}` —
pdflatex no caption on the first panel, "(b)" on the second; Perl "(a)" on the first.

Rust (Gemini round 13): both bindings route the bare form through `\phantomcaption`, which steps the counter (in a float
as the kernel `\caption` does, `\lx@donecaptiontrue`; witness 2503.21681) and lists nothing. A phantom-numbered
float counts as captioned when floats collapse (57bu, DIVERGENCES #372). **Guard**: `perfect_kernel_gemini::bare_subfloat_has_a_phantom_caption`.

## 385. subfig: sub-labels ignore the caption label format

subfig passes its package options to `\captionsetup[subfloat]` (subfig.sty:188-195, :208-225) with labelformat `parens`
as default (:285-288); the binding hard-codes `\fnum@subfigure` as `(\thesubfigure)`.

Trigger: `\usepackage[caption=false,labelformat=simple]{subfig}\renewcommand\thesubfigure{(\alph{subfigure})}` —
pdflatex "(a) Cap A"; Perl "((a)) Cap A".

Rust (Gemini round 13): subfig and subcaption labels go through the caption label format (`sub_label_tokens`,
caption_sty.rs); subfig's options reach `\captionsetup[subfloat]`. Open: subfloat.sty still hard-codes the parentheses.
**Guard**: `perfect_kernel_gemini::subfig_label_follows_the_caption_label_format`.

## 386. subcaption: `{subcaptiongroup}` is undefined

subcaption.sty:60-69 `{subcaptiongroup}`/`{subcaptiongroup*}` run `\setcaptionsubtype` for the group (caption sets
`\@subcaptype`, `\caption@@settype{sub}`, caption.sty:328), so a `\phantomcaption` inside numbers a panel of the float
for the `\label` after it. LaTeXML's subcaption binding has no such environment.

Trigger: `\usepackage{subcaption}` … `\begin{figure}\begin{subcaptiongroup}\phantomcaption\label{a}\end{subcaptiongroup}
x\caption{Grouped}\end{figure}` — pdflatex Figure 1, `\ref{a}` 1a; Perl: `Error:undefined:{subcaptiongroup}`, 2 errors.

Rust (57bu, 57bu.2): defined (subcaption_sty.rs `\lx@subcaption@group`) without an element. The group makes the sub-type
`\@captype` (the kernel numbers by `\@captype`), steps its counter plainly (`lx@float@untyped`), pre-increments the float's
counter once per float as caption's sub-type hook does (caption.sty:639-641; `preincrement_float_counter`) and records the
sub-type as the last float closed, so a later group or a sub-float inside or after it does not step it again; at its
end it drops the panel values a caption stores for a sub-float element. Outside a float it is subcaption's "outside
float" error. Residual: the panel's `\label` sits on the figure (no panel element holds it), so `\ref{a}` reads the
figure's number. The same holds for two `\phantomsubcaption\label`s in one `subfigure` (2605.28276
`fig:corridor-a`/`-b`): both labels sit on the one panel, tagged with the last number, so `\cref{fig:corridor-a}` reads
1b where pdflatex prints 1a. **Guards**: `perfect_kernel_gemini::{subcaptiongroup_numbers_the_panels,
subcaptiongroup_steps_the_figure_counter_once}` (witness 2605.01925).

## 387. `\partial` takes a greedy operand: a Leibniz quotient reads ∂(F/∂T)

Perl's grammar lists DIFFOP among the big operators (`bigop : … | DIFFOP`, MathGrammar:717), so `\partial` takes every
factor after it (`moreOpArgFactors`, :612-617), across a MulOp too.

Trigger: `$\partial V/\partial\theta$` `$T\,\partial F/\partial T$` `$\partial\Omega\times(0,T]$` — pdflatex prints a
derivative and a boundary times an interval; Perl reads `partial-differential@(V / partial-differential@(theta))`,
`T * partial-differential@(F / partial-differential@(T))`, `partial-differential@(Omega * open-closed-interval@(0, T))`.

Rust: fixed by divergence #374 (57cj; user ruling 2026-09-29): a DIFFOP takes one factor and a Leibniz quotient is
one derivative — `partial-differential@(V) / partial-differential@(theta)`, `T * (partial-differential@(F) /
partial-differential@(T))`, `partial-differential@(Omega) * open-closed-interval@(0, T)`. Witnesses 2605.03741,
2605.24774 (`\partial z^{(k)}/\partial x_i`), 2605.21149. Goldens `tests/parse/integrals_and_differentials.tex`,
`tests/parse/bigop_operands.tex`.

## 388. collapseFloat copies a side panel's box geometry onto its float

`collapseFloat` (latex_constructs.pool.ltxml:3437-3464) copies every attribute of a float's one inner float onto the
outer (:3447-3449), including the box geometry of an inner float that was a caption minipage beside another panel.

Trigger: `\begin{figure}\begin{minipage}{0.65\linewidth}Wide.\end{minipage}\hfill
\begin{minipage}{0.3\linewidth}\caption{Side.}\end{minipage}\end{figure}` — pdflatex sets the figure at `\textwidth`;
Perl writes `<figure class="ltx_figure_panel ltx_minipage" vattach="middle" width="103.5pt">` around the 224.3pt panel,
and a subfigure built the same way claims its caption minipage's width, so the parent sets `\linewidth` subfigures on
one row. Rust: fixed by divergence #375 (57by; witnesses 2605.03502, 2605.15932). Repro
`captions-floats/collapsed_panel_keeps_the_float_geometry`.

## 389. A box folded into its one rule gives the rule the box's width

`insertBlock`'s single-node fold (TeX_Box.pool.ltxml:489-493) writes the box's `width` over its only child's, so a
`\rule` alone in a minipage is drawn at the minipage's width.

Trigger: `\begin{minipage}{0.65\linewidth}\rule{6cm}{3cm}\end{minipage}` — pdflatex draws a 170.7pt rule; Perl and Rust
write `<rule class="ltx_minipage" height="85.4pt" vattach="middle" width="224.3pt"/>`. A graphic keeps its own size
(`\includegraphics[width=3cm]`, whose width lives in its options). Open (found in the 57by review).

## 390. A register command on a non-register defines it a register

`\advance`, `\multiply` and `\divide` read their `Variable` (Base_ParameterTypes.pool.ltxml:271-290); a token that is
no register is an error, and the reader then defines it a Dimension register (`DefRegisterI($token, undef,
Dimension(0))`, :278-284) and the command reads on through `by` and its operand (TeX_Registers.pool.ltxml:102-125).
TeX reports the error and returns (tex.web §1236-1237 `do_register_command`: one `get_x_token`, "You can't use … after
\divide", no `by`, no operand).

Trigger: `A \divide\relax by 2 B \multiply\relax by 2 C \advance\relax by 2 D` — pdflatex 3 errors, "A by 2 B by 2 C
by 2 D"; Perl 1 error, 1 warning ("Illegal unit of measure (pt inserted)") and "A B C D" (the first command makes
`\relax` a register, so the next two act on it), and a
later `\relax` reads a dimension ("Missing number (Dimension), treated as zero"). Rust follows TeX since 57ce: the
reader's error is the only one, the command reads `by` and its operand only after a valid variable (tex_registers.rs),
and the reader skips `\relax` only after a prefix (§1211) or in a braced `{Variable}`, which TeX executes as code
(`\setlength{\relax\mylen}{5pt}` sets it; base_parameter_types.rs `Variable`). Repro
`expansion-primitives/register_command_on_a_non_register.tex`, guard
`perfect_kernel_batch57::register_command_on_a_non_register_is_one_error`.

## 391. A copied picture repeats its svg ids

TeX's `\copy` duplicates a node list with its whatsits verbatim (tex.web:20952, :24757); PDF has no ids. The pgf
driver fixes an svg object's id when the drawing is digested (`properties => { obj => SVGNextObject() }`,
pgfsys-latexml.def.ltxml:346-352, :370-375; `$objcount` baked in at :678-690, :703-714), so every copy of a saved
picture writes the same `<svg:clipPath id="pgfcp1">`/`<svg:radialGradient id="pgfsh2">`, which the svg schema types as
an ID (svg-core-attrib.rng:26-28); `recordID`/`modifyID` (Document.pm:1447-1494) cover `xml:id` only.

Trigger: `\usepackage{tikz}\newsavebox\bead\sbox\bead{\begin{tikzpicture}\shade[ball color=red] (0,0) circle
(1);\end{tikzpicture}}` then `\usebox\bead\usebox\bead` — jing `ID "pgfcp1" has already been defined`, `ID "pgfsh2" …`;
the same inside one picture (`\node{\usebox\bead};\node{\usebox\bead};`). Perl and Rust (before 57cl) identical.

Rust: fixed by divergence #383 (57cl, `Document::record_svg_ids`): suanpan-l3 9,481 → 1 jing lines (expl3 coffins drawn
once, placed with `\box_use:N`), thuaslogos-doc-english/-dutch 4 → 0. Repro
`graphics-tikz/copied_picture_keeps_unique_svg_ids`.

## 392. glossaries turns its link targets off with its links

glossaries.sty.ltxml:42 runs `\glsdisablehyper` so `\gls` makes no link (the binding's `ltx:glossaryref` carries
it). That command also sets `\@glstarget` to `\@secondoftwo` (glossaries.sty:4302-4306), so `\glstarget` makes no
anchor. glossaries-extra.sty:5204-5210 turns the links back on when `\hyperlink` exists, restoring the target only
if `\@glstarget` still is `\glsdohypertarget` (:6558): every `\glshyperlink` (:5185) points at nothing.

Trigger: `\usepackage{hyperref}\usepackage{glossaries-extra}\newglossaryentry{foo}{name={foo},description={a foo}}`,
then `\glstarget{foo}{Foo}` and `\glshyperlink{foo}` — Perl `<ref idref="glo:foo">` with no anchor (2 jing lines);
pdflatex's `/Names` holds `glo:foo`.

Rust: fixed in 57cn — the binding keeps the target choice glossaries.sty made (:4295-4301) across
`\glsdisablehyper`. glossariesbegin 44 → 0 and mfirstuc-manual 32 → 0 jing lines (glossaries-user 969,
datatool-user 1,235, glossaries-extra-manual 1,838 dangling idrefs, nlctuserguide's `\targetorhyperlink`). Repro
`index/glshyperlink_target_resolves`.

## 393. hyperref's anchor internals are missing or mis-sized

hyperref.sty.ltxml defines neither `\Hy@raisedlink` (hyperref.sty:2100-2118 typesets its argument; the non-PDF
drivers `\let` it to `\@empty`, hdvips.def:40, which leaves the argument to run) nor `\hyper@@anchor`
(hyperref.sty:5121, what `\hypertarget` calls, :4805-4811). And `\hypertarget` (L266) has no `sizer`, so the
anchor's name is measured as text — TeX's anchor is a zero-size whatsit.

Trigger: `\makeatletter Text\Hy@raisedlink{\hyper@@anchor{Hendnotepage.1}{\empty}}` then
`\hyperlink{Hendnotepage.1}{back}` — Perl 2 undefined errors and a dangling idref; and
`\setbox0\hbox{\hypertarget{averylongtargetname}{}}\the\wd0` — Perl 91.16689pt, pdflatex 0.0pt.

Rust: before 57cn the binding had `\Hy@raisedlink` as a no-op swallowing its argument — silently the same loss. Fixed
in 57cn: `\let\Hy@raisedlink\@empty`, `\hyper@@anchor` = `\hypertarget`, sizers `#2`/`#4` on `\hypertarget`/
`\hyperdef`. biblatex-chicago cms-noteref-demo 18 → 0 jing lines, arXiv 2308.06254 43 → 0. Repros
`singletons/hy_raisedlink_keeps_its_anchor`, `boxes-groups/hypertarget_has_no_size`.

## 394. The blank-footnote idiom gets the footnote counter's mark

latex.ltx's `\@footnotetext` typesets `\@thefnmark` as the note's mark, so `\gdef\@thefnmark{}\@footnotetext{…}`
is a note with no mark (the common `\blfootnote`). Perl's `\@footnotetext` is `\lx@notetext{footnote}`
(latex_constructs.pool.ltxml:490), which ignores `\@thefnmark`: the note carries the footnote counter's mark ("0"
before any footnote, shown by `.ltx_note_mark`), and after a pending `\footnotemark` it fills that mark's note — whose
own `\footnotetext` then becomes a second note.

Trigger: `\def\blfootnote{\gdef\@thefnmark{}\@footnotetext}` then
`Cell\footnotemark{} then\blfootnote{Blank note.} and more.\footnotetext{The text.}` — Perl and Rust: footnote 1 reads
"Blank note.", "The text." a stray note; pdflatex: footnote 1 "The text.", "Blank note." unmarked.

Rust: the same (sect03.rs:303 `\@footnotetext` = `\lx@current@footnotetext`); since 57cn.1 footnotehyper's saved notes
reach it through `\H@@footnotetext` (the footnotehyper manual's table note attaches to the table's `\footnotemark`).
Open. RED repro `singletons/blank_footnote_mark_is_blank`; the guard
`perfect_kernel_batch57::footnote_anchor_name_is_defined` pins the `mark="0"` as this residual.

## 395. A float nested in a captioned float takes the outer caption's number

`\@@add@caption@counters` (latex_constructs.pool.ltxml:3193-3201) stores a caption's tags, id and list entry in one
global slot per float type (`table_tags`, `table_id`, `table_inlist`), which the float's end takes
(`afterFloat`/`RescueCaptionCounters`, :3384-3392, :3203-3214). A float of the same type opened inside the captioned
float overwrites the slot with its own caption and takes it at its own end: the outer float is left with no
number and no List of Tables line, and its `\label` lands on `<document>`.

Trigger: `\begin{table}\caption{Outer}\label{t:o}\begin{minipage}{.45\linewidth}\begin{table}[H]\caption{Inner}`
`\label{t:i}x\end{table}\end{minipage}\end{table}` — Perl: `<document labels="LABEL:t:o">`, the outer `table`
untagged; pdflatex: "Table 1: Outer", "Table 2: Inner", `\ref`s 1 and 2.

Rust (57cp): each float sets the enclosing float's pending caption state aside when it begins (for the enclosing
`\@captype` and its own type, in its own group) and restores it when it ends (`set_aside_pending_caption`,
`restore_pending_caption`, latex_constructs/mod.rs). Witnesses: a tabularray tall table in a captioned table (the 57cp
review's nest2.tex, sub.tex). Guard `perfect_kernel_batch57::nested_float_keeps_the_outer_caption`; repro
`captions-floats/nested_float_keeps_the_outer_caption`. Open: the set-aside covers the tags, id and list line, not the
number a sub-float row reserved for its parent (`PREINCREMENTED_<type>`, `LAST_FLOATTYPE`): a captioned `table` in a
minipage between two `subtable`s takes that number — "K=1 O=2 N=3" where pdflatex prints "A=1a K=2 B=2a O=3 N=4"
(Perl alike; restoring the pre-increment alone steps the counter again where caption.sty's flags do not). RED repro
`captions-floats/float_in_a_subfloat_row_keeps_the_parent_number`.

## 396. Two `\caption`s in one float share one number

A float holds one caption state (`\@@add@caption@counters`, latex_constructs.pool.ltxml:3193-3201: one global
`<type>_tags`/`_id`/`_inlist` per type, taken at the float's end by `RescueCaptionCounters`, :3203-3214). A second
`\caption` in the same float steps the counter again and overwrites the first's state: the float carries the last
number, every `\label` in it reads that number, and the List of Tables has one line for it.

Trigger: `\begin{table}\caption{First}\label{t:a}\begin{tabular}{l}x\end{tabular}\caption{Second}\label{t:b}\end{table}`
`Refs \ref{t:a} \ref{t:b}` — Perl and Rust: one table numbered 2, "Refs 2 2", one LoT line; pdflatex: "Table 1:
First", "Table 2: Second", "Refs 1 2", two lines.

Rust: the same (Open). tabularray tall tables no longer reach it (57cp gives each its own float, collapsed into a
caption-less document float). RED repro `captions-floats/two_captions_in_one_float_are_numbered_apart`.


## 397. A `sidewaysfigure` is not in the List of Figures

rotating.sty's `sidewaysfigure`/`sidewaysfigure*` are figure floats (`\@float{figure}` set rotated), and their
`\caption` writes a List of Figures line as any figure's. Perl's binding (rotating.sty.ltxml:94-124) gives their
`ltx:figure` no `inlist='#inlist'`, which `{figure}` (latex_constructs.pool.ltxml:3395) and its own
`sidewaystable` have: sideways figures are numbered and labelled but missing from the list.

Trigger: `\usepackage{rotating}` … `\listoffigures\begin{sidewaysfigure}x\caption{Side}\end{sidewaysfigure}` —
Perl: the figure without `inlist`, an empty List of Figures; pdflatex: "1 Side".

Rust (57cp): `inlist='#inlist'` on both (rotating_sty.rs). Guard `perfect_kernel_batch57::sidewaysfigure_is_listed`;
repro `captions-floats/sidewaysfigure_is_listed` (57cp review 6).

## 400. A function before a limit-type operator multiplies it: `\log\det A` reads log·det(A)

Perl's OPFUNCTION takes no big operator as its bare argument (`aBarearg`, MathGrammar:323-331), and a LIMITOP is one
(`bigop : BIGOP | SUMOP | INTOP | LIMITOP | DIFFOP`, :717), so the function before it is a factor of its own.

Trigger: `$\log\det A$` `$\sin\det A$` `$\log\det(\Sigma)$` — pdflatex prints the log of a determinant; Perl reads
`logarithm * determinant@(A)`, `sine * determinant@(A)`, `logarithm * determinant@(Sigma)` (the review's Perl oracle,
`~/data/pk_agents/math/reviews/rev57cj8/perl/p1`).

Rust: fixed by divergence #390 (57cj.9; 57cj.8 review): the function takes the limit-type operator's application —
`logarithm@(determinant@(A))`, `sine@(determinant@(A))`, `((nabla _ x)@(logarithm))@(determinant@(A))` for
`\nabla_x\log\det(A)`. 243 formulas in 55 of the 3,003 A/B papers, most `\log\det\Sigma` (2605.00130, 2605.26554,
2605.02883, 2605.03984, 2605.24401, 2605.25592, 2605.14289). The take nests through a function's bare application to a
function (57cj.10: `\log\log\det A` log@(log@(det A)), `\min_\theta\log\det\Sigma_\theta`, `\log\exp\sup_x f`), and inside a
trig argument an OPFUNCTION or (57cj.11) a composed trig function takes it (`\sin\log\det A` sin@(log@(det A)), `\sin\cos\det A`
sin@(cos@(det A)); Perl sine@(logarithm)·det(A), sine·cosine·det(A)). A word that names the operator's variant keeps
Perl's product: `\arg` before an infimum or a supremum (`\arg\inf f(\theta)`, 2605.30648, 2605.16560), and
`\operatorname{ess}`, never a function of a value, before every limit-type operator (57cj.12: `\operatorname{ess}\lim_n f_n`,
`\operatorname{ess}\det A` ess·det(A)); before any other limit-type operator `\arg` takes it (57cj.11:
`\arg\det M_q` argument@(det M_q), `\arg\min_x\log\det\Sigma_x` argument@(min_x@(log@(det Σ_x)))). Goldens
`tests/parse/bigop_operands.tex`, `tests/parse/rust_parse_additions.tex`, `tests/parse/operator_application.tex`.

## 398. The optional `=` of `\setbox`, `\font`, `\openin`, `\openout` is matched unexpanded

TeX scans an assignment's optional `=` with expansion (tex.web §405 `scan_optional_equals`, used by `\setbox`
§1241, `\font` §1257, `\openin` §1275, `\openout` §1351), so a conditional or a macro may supply it. Perl spells it
`SkipMatch:=` (TeX_Box.pool.ltxml:599, TeX_Fonts.pool.ltxml:82, TeX_FileIO.pool.ltxml:50/:120), which reads the
next token unexpanded: the conditional is left in the input and "=" is typeset.

Trigger: `\newbox\b \newif\ifup\uptrue \setbox\b\ifup=\hbox{Up}\fi A\box\b` — Perl: "UpA=" (the box on the page);
pdflatex: "AUp". reledmac/eledmac write `\setbox\l@dlp@rbox\ifleftnoteup=\vbox…`.

Rust (58a): `SkipKeyword:=` (the expanding keyword read) for `\setbox`, `\font`, `\openin`, `\openout`, the
`\Umath…`/`\Udelcode` assignments, `\luadef` and the XeTeX interchar stand-ins; `\let` keeps its unexpanded read (§1221). Guard
`perfect_kernel_batch58::optional_equals_expands`; repro `expansion-primitives/optional_equals_expands`.

## 399. `\setcounter`, `\addtocounter`, `\stepcounter`, `\refstepcounter` and `\pagenumbering` cannot be patched

latex.ltx defines them as macros (:10115-10137, :14893-14895), and packages patch them with etoolbox
(`\apptocmd`/`\pretocmd` rescan the `\meaning`): reledmac appends its page-counter hooks
(reledmac.sty:10022-10031, :6632-6651, :6607-6611). Perl defines the first three as primitives
(latex_constructs.pool.ltxml:3000-3002) and `\pagenumbering` as a no-op (:1003): every patch takes the failure
branch, and `\pagenumbering{roman}` leaves arabic page numbers.

Trigger: `\usepackage{etoolbox}\newcounter{foo}\apptocmd{\setcounter}{\typeout{hooked}}{}{\typeout{FAIL}}` — Perl:
FAIL; pdflatex: the hook runs on every `\setcounter`.

Rust (58a): macros that call one another as latex.ltx's do, so a patch sees every step through them:
`\setcounter`/`\addtocounter` around `\lx@setcounter`/`\lx@addtocounter` (calc's around `\lx@calc@…`),
`\stepcounter` = `\addtocounter{#1}\@ne` plus the resets within, each through `\@stpelt` in a group (latex.ltx:10132-10138:
set to -1, then `\stepcounter`; redefinitions such as footmisc's `perpage` and zref-perpage's apply; calc's own
`\stepcounter` skips `\addtocounter`, calc.sty:64-69), `\refstepcounter` = a macro around `\lx@refstepcounter`, which
expands to `\stepcounter` plus the labelling half of Perl's `RefStepCounter` (`label_stepped_counter`). The xml:id
scheme is Perl's: `AddToCounter` defines `\@<ctr>@ID` as `StepCounter` does (Package.pm:736/:745), a reset leaves
value and ID 0 as `ResetCounter`, and the `UN` companions are zeroed directly (Package.pm:883-893). `\pagenumbering` is latex.ltx's body, and the book/amsbook/llncs matter commands
switch it as their classes do (book.cls:284-291, amsbook.cls:944-945, llncs.cls:250-253). Residual, by design: the
bindings' constructors (`\chapter`, `equation`, floats, theorems) step their counters in Rust (`ref_step_counter`,
Perl's model), so a patch on `\stepcounter` does not see them. Open: LaTeX's generic command hooks
(`\AddToHook{cmd/refstepcounter/after}`) do not run (before 58a too). Guards `perfect_kernel_batch58::{
counter_commands_are_patchable, counter_steps_go_through_the_patches, matter_commands_set_the_page_numbering}`;
repros `macro-state/{counter_commands_are_patchable, counter_steps_go_through_the_patches,
matter_commands_set_the_page_numbering}`.

## 401. `\lastbox`, `\unskip`, `\unkern`, `\unpenalty` at the start of a paragraph reach the material before it

tex.web §1091 `new_graf` starts a paragraph on a list of its own (holding the indent box), so `\lastbox` (§1080) in
`\everypar` finds at most that indent box. Perl builds the paragraph on the enclosing list and `\lastbox` is
`pop(@LaTeXML::LIST)` (TeX_Box.pool.ltxml:596-597), so it removes whatever came before — the previous paragraph's line.

Trigger: `\hbox{Kept line.}\everypar{\setbox0=\lastbox}Next paragraph.` — Perl/Rust before 58b: "Next paragraph.";
pdflatex: "Kept line. Next paragraph." reledmac's `\autopar` (`\everypar{\setbox0=\lastbox …}`, reledmac.sty:2179)
dropped every paragraph but the last of each `\autopar` block (2-line_numbers_in_header: 1,448 of ~1,600 words, 0 errors).

The same holds for `\unskip`/`\unkern`/`\unpenalty` (§1105 `delete_last`): `\vbox{\hbox{A}\vskip 1cm\everypar{\unskip}B}`
lost its `\vskip` (pdflatex 47.29pt, Rust before 58b 6.83pt).

Rust (58b): each box list records where its paragraph began (`Stomach::paragraph_start`, set when horizontal mode is
entered from vertical, saved and restored with the list); in a paragraph, the four commands find nothing at or below it
(`pop_own_box`). `\lastbox` taking any last item, not only a box, is #432. Guard `box_primitives::lastbox`; repro
`boxes-groups/box_primitives_lastbox`.

## 402. `\vsplit` returns the whole box and never empties the register

tex.web §977 `vsplit` removes the top of the register's list up to the best break (§970-974 `vert_break`: glue after
a non-discardable item, a kern before glue, a penalty), returns it, prunes the remainder's top glue and kerns (§968) and
leaves the register void once empty. Perl's `\vsplit` (TeX_Inserts.pool.ltxml:36-40) behaves like `\box`: it returns the
whole box and leaves the register untouched, so a drain loop never terminates.

Trigger: `\setbox0\vbox{\hbox{A}\vskip2pt\hbox{B}}\loop\ifvbox0\setbox2\vsplit0 to 0pt\repeat` — Perl: `Fatal:timeout`;
pdflatex: two passes. reledmac's `\do@line` (reledmac.sty:2099-2101, :2199-2200) and short-math-guide's column
splitter drain this way.

Rust (batch 54, 58b): the register is split at top-level items and the remainder stored back in place (the drain
survives the caller's group); since 58b a new piece starts only at a breakpoint TeX has — glue after a non-discardable
item, a kern before glue or a box, or a box or sized item after a non-discardable item other than a rule (standing
for the interline glue LaTeXML's lists lack; a rule is no break, nor is the box after one, §1056) — so zero-size items (the `\par` of an `\endgraf`, anchors) stay with the piece before them, the
remainder's top glue and kerns are pruned and an emptied register is void; the piece ends at the last breakpoint whose
piece fits (§974). Since 59v a `\penalty` is a list item (forced breaks); since 59x the break search measures as
`vert_break` does (§970-976: interline glue as `append_to_vlist` made it, §679; `\splitmaxdepth`), the piece is
exactly the split height and as deep as its last box (§977), and the remainder gets `\splittopskip` glue less its
first box's height (§968-969) at its natural size (a `\vbox to` remainder no longer keeps the split box's height):
`\vbox{\hbox{One}\hbox{Two}\hbox{Three}}` split to `2.5\baselineskip` keeps two lines, not three. Residuals: a
paragraph is one item (never split into its lines: reledmac numbers a wrapped `\pstart` once —
2-titles_in_line_numbering_with_notes 16 lines for the golden's 28); the piece is a list, not a vbox (`\ifvbox` false,
its lines run together when typeset directly); no stretch, shrink or penalty costs (§974-975) and no `\vfil` item;
`\nointerlineskip` leaves no item, so the search assumes interline glue there; a whatsit that is no box (`\label`)
reports a width and is counted as a line; null paragraphs (`\noindent{}\par`) are counted as lines; a `\vtop`'s
remainder is typeset top-attached (box_primitives_vsplit RED cases 1-2 and 8 for R2, 18-20, 24, 27-28). Guards
`box_primitives::vsplit`, `perfect_kernel_batch58::vsplit_breaks_only_where_tex_can`;
repro `boxes-groups/vsplit_breaks_only_where_tex_can`.

## 403. slides' `\addtime`/`\settime` read a bare `Number`, so the documented braced form misparses

slides.cls:85-87 defines `\addtime{<seconds>}`/`\settime{<seconds>}` (they only set counters; nothing is typeset).
Perl's binding reads `\addtime Number` (slides.cls.ltxml:73-74), so the brace is not a number.

Trigger: `\documentclass{slides}\begin{document}\begin{slide}\addtime{120}\end{slide}\end{document}` — Perl:
`Warning:expected:<number> Missing number, treated as zero`, `<note>add time 0</note>120` (the 120 typeset); pdflatex:
nothing printed.

Rust: same binding spec (slides_cls.rs:60-61), and a register value absorbs as nothing (`digested.rs:378`, where Perl's
`beAbsorbed` writes its text, Object.pm:160-170), so even `\addtime 120` gives `<note>add time </note>`. Open: read
`{Number}` and absorb register values as text; RED repro `singletons/register_value_absorbs_as_its_text`.

## 404. amsart's `\smaller` is an absolute 0.83pt size

amsart.cls's `\smaller` is relative (`\larger[-1]`: the next smaller size), as its `\larger`. Perl's binding defines
`\larger` as `scale => 1.2` but `\smaller` as `size => 1/1.2` (ams_support.sty.ltxml:46): an absolute 0.83pt font.

Trigger: `\documentclass{amsart}\begin{document}A {\smaller B} C\end{document}` — Perl: B at `fontsize="8%"`; pdflatex: B at
the next smaller size.

Rust (58c): `scale => 1/1.2` (ams_support_sty.rs); with size switches setting the leading (58c) the absolute size had
also given a 1pt `\baselineskip`.

## 405. The rule after `\leaders` ends or starts the paragraph

After `\leaders`, TeX reads the next non-blank, non-`\relax` token (tex.web §1084 `scan_box`); an `\hrule` or `\vrule`
there is a rule specification (§1078), never executed, so a horizontal-mode `\hrule` does not end the paragraph (§1094
`head_for_vmode`) and a vertical-mode `\vrule` does not start one (§1090). Perl digests the rule as `\leaders`' first
argument, and its afterDigest `leaveHorizontal`/`enterHorizontal` (TeX_Box.pool.ltxml:838, :803) does both. Its argument
reader skips only the literal `\relax` token (`readDigested`, Base_ParameterTypes.pool.ltxml:369-370) where `scan_box`
skips anything meaning `\relax` (§404), so a robust filler — `\DeclareRobustCommand\rrule{\hrule}`,
`\protect\relax`-headed — digests as an empty leader and its rule is lost (`A\leaders\rrule\hfill B`: Perl
`<p>A</p><p><text class="ltx_leader"/> B</p>`), as is a box after a `\let` alias of `\relax`.

Trigger: `A\leaders\hrule\hfill B` — Perl: `<p>A</p><p><rule class="ltx_filled_leader" height="1px"
width="345.0pt"/>B</p>`; pdflatex: one line. Every `\hrulefill` (latex.ltx:643
`\leavevmode\leaders\hrule\hfill\kern\z@`) split its paragraph (2605.00332: a figure panel's `$z$ (…) \hrulefill`).

Rust (58d): `\leaders` reads past spacers and `\relax`-meaning tokens with full expansion as `scan_box` does
(`read_box_operand`) and, for an `\hrule`/`\vrule`, marks its group (`lx@leaders@rule`); that rule skips
`leave_horizontal`/`enter_horizontal` (tex_box.rs). A box filler's own rules execute. Guard
`perfect_kernel_batch58::leaders_rule_keeps_the_paragraph`; repro `boxes-groups/leaders_rule_keeps_the_paragraph`.

## 406. A vertical `\kern` is stored as a width

In vertical mode a kern is vertical space (tex.web §1061, §1057 `append_kern`); `\lastkern` reads its size (§424). Perl's
`\kern` always stores `width` (TeX_Kern.pool.ltxml:51-52) and `\lastkern` returns it (:74), so a vertical list sizes the
kern as a line of `\baselineskip`.

Trigger: `\setbox0\vbox{\hbox{A}\kern3pt\hbox{B}}[\the\ht0]` — Perl: "[30.83331pt]"; pdflatex: "[21.83331pt]"
(`\vbox{\kern3pt\hbox{A}}`: Perl 12.0pt, pdflatex 9.83331pt; `\vtop` depth: Perl 24.0pt, pdflatex 15.0pt).

Rust (58d): in vertical modes `\kern` stores `height` with `isVerticalSpace`/`isBreak`, as `\vskip` does (tex_kern.rs),
and `\lastkern` reads the height. Guard `perfect_kernel_batch58::vertical_kern_is_vertical_space`; repro
`boxes-groups/vertical_kern_is_vertical_space`.

## 407. `\tenln`, `\tenlnw`, `\tencirc`, `\tencircw` are not fonts

preload.ltx:42-43 loads LaTeX's picture fonts (`\font\tenln=line10 \font\tenlnw=linew10 \font\tencirc=lcircle10
\font\tencircw=lcirclew10`), which `\thinlines`/`\thicklines` (latex.ltx:16806-16811) and diagrams.sty use as fonts.
Perl defines them as primitives selecting a family of that name (latex_constructs.pool.ltxml:2751-2754), with no
encoding, so the text stays OT1: `\tenln\char45` (latex.ltx:16914's right arrowhead) is a hyphen in the text font.

Trigger: `\setbox0\hbox{\tenln\char45}[\the\wd0]` — Perl: "[3.33333pt]"; pdflatex: "[10.0pt]". In 2605.02221 diagrams.sty's
`\rTo{\pi}` then leaves a 10pt `\hskip` (pdflatex 6.67pt), a `\quad` hint, and its exact sequences parse as fragments.

Rust (58f): the constructs load the four fonts as preload.ltx does (sect08.rs), so they are the graphic family in the
`line`/`lcircle` encodings. Guard `perfect_kernel_batch58::picture_fonts_are_fonts`; repro
`fonts-nfss/picture_fonts_are_fonts`.

## 408. A character of a raw `\font` is measured by the Unicode character it maps to

TeX measures a character from its font's TFM (tex.web §554, §571-572). Perl measures a box's string through the
standard metrics (Font.pm:535-548 `getMetric`, :587-614 `computeStringSize`): a font whose family has none falls back
to cmr, cmmi, cmsy, … for the decoded Unicode character, so the width belongs to whatever font has that character;
`\font` does not load metrics (TeX_Fonts.pool.ltxml:80-81).

Trigger: `\setbox0\hbox{\tencirc\char0}[\the\wd0]` — Perl: "[6.25002pt]"; pdflatex: "[3.99998pt]".

Rust (58f): a font loaded by name whose family has no standard metric measures a character from its TFM, its slot
recorded on the box (`common/font/tfm.rs`, `Font::measuring_tfm`, `Tbox::with_tfm_slot`). Fonts with a standard metric
keep Perl's measure, including its fallback for a character the metric lacks: `\font\x=cmsy10 \x\char'041` is 5.00002pt
(cmmi's `\vec`) for pdflatex's 10.00002pt (open). Guard `perfect_kernel_batch58::line_font_char_has_its_tfm_width`; repro
`fonts-nfss/line_font_char_has_its_tfm_width`.

## 409. `\font` without `at` sizes the font from the digits of its name

TeX loads a font at its TFM design size, times `scaled` when given (tex.web §568). Perl's `decodeFontname`
(Font.pm:220-223) takes the size from the digits ending the name, or 1 when there are none.

Trigger: `\font\manual=manfnt` is a 1pt font in Perl, manfnt at its 10pt design size in pdflatex; bbm17's design size
is 17.28pt, Perl's 17pt. Measured with the TFM (KPE #408), `\setbox0\hbox{\manual\char127}` is 1.38889pt where pdflatex
gives 13.88893pt.

Rust (58g): `\font` without `at` takes the TFM design size (times `scaled`) when the TFM is found (tex_fonts.rs,
`Tfm::design_size`); otherwise Perl's rule. The plain dump's manfnt (plain.tex:467 `\font\preloaded=manfnt`) records its
parameters at 655360sp where Perl's dump has 65536sp; `\font\x=cmr11` is 10.95pt. Guard
`perfect_kernel_batch58::raw_font_scales_by_design_size`; repro `fonts-nfss/raw_font_scales_by_design_size`.

## 410. A font selected by `\font` outlives the next font selection, and math characters take it

`\font\y=msbm10` then `\y` changes only `cur_font` (tex.web §1217); NFSS's `\f@encoding`, `\f@family`, … stay, so the
next family, series, shape or encoding declaration (`\textrm`, `\bfseries`, `\emph`, `\normalfont`) re-selects the
NFSS font through `\selectfont` (latex.ltx:12576-12579). In math, a class-7 character takes its mathcode family
(tex.web §1151-1155) whatever the text font is. Perl keeps one font object whose encoding the raw font set: its
`\selectfont` merges only family, series and shape (latex_constructs.pool.ltxml:5202-5221), and `\f@encoding` reads the
font; in math, a class-7 character with `\fam` < 0 decodes through any current font that differs from the initial math
font (Package.pm:2950-2955).

Trigger: `\font\y=msbm10 {\y\textrm{abc}} $\y a+b$` — Perl: msbm glyphs for both; pdflatex: "abc" and "a + b".

Rust (58g): a `\font` identifier keeps the NFSS font it replaces (`Font::nfss_font`, set in `content::merge_font_ref`,
which no longer writes `\f@family`/`\f@size` from it), and `Font::merge_ref` returns to that font's family, series,
shape, size and encoding at the next font selection; colour and a bare
size change keep the raw font. `decode_math_char` skips the current-font path for a raw font. Guard `perfect_kernel_batch58::raw_font_ends_at_a_font_selection`; repro
`fonts-nfss/raw_font_encoding_ends_with_its_font`.

## 411. `\font\y=cmr10 \y` reads the undefined `\y` while looking for "at"

tex.web §1257 `new_font` defines the identifier as `\nullfont` before it scans the file name and the `at`/`scaled`
keywords, so a `\y` right after `\font\y=cmr10 ` is a font when the keyword scan expands it. Perl's `\font` reads the
keywords first (TeX_Fonts.pool.ltxml:89-95), when `\y` has no meaning.

Trigger: `\font\y=cmr10 \y abc` — Perl: `Error:undefined:\y`; pdflatex: "abc", 0 errors.

Rust (58g): `\font` gives the identifier `\nullfont`'s meaning (in the assignment's scope) before reading the
keywords (tex_fonts.rs): a plain meaning assignment, so a pending `\afterassignment` token is not fired before the size
is read (it follows the whole `\font`, DIVERGENCES #394), and not for a locked name, whose binding the lock keeps
(`state::is_definition_locked`; `install_font_def` still records the font's `fontinfo_` values for it). Guards
`perfect_kernel_batch58::{font_name_is_defined_before_its_size, font_keeps_a_locked_name}`; repros
`fonts-nfss/{font_name_is_defined_before_its_size, font_keeps_a_locked_name}`.

## 412. A caption, equation or item steps its counter without `\refstepcounter`

LaTeX steps them through `\refstepcounter` — `\refstepcounter\@captype` (latex.ltx:17391-17399; caption.sty:200-205,
:551-569), `\refstepcounter{equation}` (:15737-15738), `\refstepcounter\@listctr` — so a local rebinding applies:
floatrow typesets every floatbox in throwaway boxes under `\FR@loc@`, whose `\refstepcounter` is a local `\advance`
(floatrow.sty:543-556), undone when the box's group ends. Perl calls `RefStepCounter` directly
(latex_constructs.pool.ltxml:3193-3196 and the equation and list bindings), so each measuring pass steps the counter for
good.

Trigger: `\begin{table}\setbox0\vbox{\def\refstepcounter#1{\advance\csname c@#1\endcsname\@ne}\caption{M}}\caption{One}
\end{table}` — Perl: "Table 2"; pdflatex: "Table 1". kaytannollista-latexia numbers its tables 2.3, 2.6, 2.9 for 2.1,
2.2, 2.3.

Rust (58h): float, equation, item, theorem and section steps go through `\refstepcounter`'s current meaning
(`ref_step_counter_planned`, `RefStepCounter!`): with the kernel meaning directly; otherwise `\refstepcounter{<counter>}`
is digested — the counter, as LaTeX passes it — with the type whose tags to make left for the kernel's label (a
shared-counter theorem's own type), and the props it records are taken, else the counter's value as it stands is
labelled. As caption does (caption.sty:551-570), its prepare hook runs first and a continued float's first step
suppresses the counter's own `\stepcounter` for the length of the `\refstepcounter` — the suppression uses the
continuation up, as caption's next step clears its flags (:590-599) — so a copy of the kernel's `\refstepcounter`
(crossreference.sty:85) continues it too. Notes (`\stepcounter\@mpfn`, latex.ltx:17649) and longtable
(`\@kernel@refstepcounter`, longtable.sty:115) step directly; so does the kernel's `\@thm` in LaTeX
(`\@kernel@refstepcounter`, latex.ltx:17197), where Rust's theorems, as amsthm's and ntheorem's (amsthm.sty:145,
ntheorem.sty:870), go through `\refstepcounter`. Guards `perfect_kernel_batch58::{caption_steps_through_refstepcounter,
floatrow_floatbox_steps_its_counter_once, floatbox_steps_its_equation_once,
steps_keep_their_type_and_continuation_through_a_wrapper, continued_float_through_a_copied_refstepcounter,
continued_float_continues_one_step}`. What a rebinding typesets is digested and dropped (a `\refstepcounter` that prints
`[STEP-#1]` prints nothing; pdflatex prints it); none of the rebindings above typesets anything.

## 413. A caption never reaches a package's `\@makecaption`, so floatrow's `\floatfoot` is lost

LaTeX's `\@caption` typesets the caption with `\@makecaption` (latex.ltx:17401ff); floatrow redefines it to fill its
caption box `\@floatcapt` (`\flrow@makecaption`, floatrow.sty:85-102), and its layout places a floatbox's
`\floatfoot` text only where that box is filled (`\flrow@FB@`, `\flrow@FC@`, :363-418). Perl locks `\@caption` and emits
the caption directly (latex_constructs.pool.ltxml:3169-3187), so no package `\@makecaption` runs (a raw class's
typographic one would replace `<ltx:caption>`).

Trigger: `\usepackage{floatrow}` … `\floatbox{table}{\caption{Cap}}{Body\floatfoot{A foot note.}}` — Perl: the foot
printed twice among 8 errors (caption3 internals undefined); Rust before 58i: the foot lost at 0 errors; pdflatex:
"Body", "Table 1: Cap", "A foot note.".

Rust (58i): the caption material goes through `\lx@setfloatcapt` (caption.sty's `\caption@setfloatcapt`, :622,
`\@firstofone` by default; sect09.rs), reset at every float and sub-float begin (caption.sty:648, `\caption@subtypehook`); the floatrow
binding (`floatrow_sty.rs`) points it at `\@floatcapt` inside each floatbox. Guards
`perfect_kernel_batch58::{floatrow_floatfoot_keeps_its_text, floatrow_subcaption_stays_in_its_panel,
floatrow_keeps_an_object_it_measures_empty, floatrow_drops_an_empty_object}`. floatrow's own empty-object test
(:366-369: a 0pt object box and its frame are gobbled) stays; a `pspicture`, which measured 0pt, has its size
(DIVERGENCES #397); a truly missing graphic measures 0pt in pdflatex too, but a graphic `filecontents` writes
(floatrow's samples' `pslearn.eps`, pictures.tex:1) is kept only in memory and not found when it is measured, so it is
dropped (SYNC_STATUS 58i residual). With a binding, floatrow is now loaded for an
unknown class that requires it (psta.cls), as pdflatex loads it.

## 414. A list numbered by its enclosing list's counter loops in its id formatters

`beginItemize` makes a list's id relative to the enclosing item, `\the<list>@ID` = `\the<outer>@ID.I…`, then the item
ids relative to the list, `\the<counter>@ID` = `\the<list>@ID.i…` (latex_constructs.pool.ltxml:1323-1335). When the list
uses the enclosing list's counter — `\begin{list}{}{\usecounter{enumi}}` around an `enumerate`, both on `enumi` — the
second definition redefines the first's reference, and each formatter expands the other.

Trigger: `\begin{list}{}{\usecounter{enumi}}\item x \begin{enumerate}\item y\end{enumerate}\end{list}` — Perl loops until
terminated (200 s); pdflatex: "x", then "1. y".

Rust (58j): when the enclosing id chain reaches the list's own counter (`list_id_chain_reaches`: directly, or through
lists in between), the list hangs on the enclosing item's id, expanded once — the item's `\@<counter>@ID`, which the
list's own steps move, is put back when the list closes — and the open lists' counters are taken out of the shared
counter's reset list, so its items do not reset them (`begin_itemize`, counter dialect);
only inputs that looped meet the condition. The list id is built from explicit tokens, so a counter name with a digit
(enumitem's `steps2i`) stays one control sequence where Perl's string body splits it (`\thesteps` undefined). Guards
`perfect_kernel_batch58::{numbered_list_with_a_nested_enumerate_converts, list_numbered_by_an_enclosing_counter}`.

## 415. Clearing `\everypar` by name at `\begin{document}` warns once a package redefines it

Perl clears `\everypar` at `\begin{document}` by name (`AssignRegister('\everypar', …)`,
latex_constructs.pool.ltxml:319). latex.ltx:9498's `\everypar{}` goes through `\everypar`'s current
meaning, and the paragraph hook runs the register latex.ltx:9070 allocated (`\newtoks\everypar`,
named by number in `\g__para_standard_everypar_tl`, :9072-9077), so a package that redefines
`\everypar` as a macro keeps working in pdflatex while Perl warns "The control sequence
'\everypar' is not a register".

Trigger (rlbicig.sty:62-64, loaded by montex/mls): `\let\oldeverypar\everypar
\def\everypar#1{\oldeverypar{[EP]#1}}` in the preamble, `\everypar{X}` in the body — Perl: 1
warning; pdflatex: every paragraph starts `[EP]X`.

Rust (58k): the register is captured as `\lx@para@everypar` (`latex_constructs_rust_only.rs`) and
the paragraph start fires it (DIVERGENCES #267); `\begin{document}` clears `\everypar` through its
meaning — the register it names (a preamble chain such as arabicore.sty:123-128 or babel's
rlbabel.def:121-125 is kept) or, for a macro, `\everypar{}` digested through it. Read by name, the
Rust firing had warned twice per paragraph (montex manual 2,031 warnings, mlsquick 276, zanabazr 9)
and fired nothing; after `\let\everypar\mytoks` it fired the wrong list. Guards
`perfect_kernel_batch58::{everypar_redefined_by_a_package_still_fires,
everypar_let_to_another_register_keeps_the_list,
everypar_chained_in_the_preamble_survives_the_document}`.

## 416. enotez's endnotes are lost: `\printendnotes` lists only notes read back from the `.aux`

enotez writes each note to the `.aux` (`\enotez@note{id}{mark}{split}{…}{text}`, enotez.sty:323-327)
and fills the list `\printendnotes` loops over only when the next run reads it back (:330-340,
:435-471). Perl has no enotez binding and keeps no `.aux`, so the list prints its heading only, the
notes' text is lost and every mark's `\hyperlink{enz.N}` (:197-212) dangles (with
`--includestyles`; by default `\endnote` is undefined).

Trigger: `\usepackage{enotez}\usepackage{hyperref}` … `Alpha\endnote{First note text.}` …
`\printendnotes` — pdflatex (2 runs): "Notes / 1. First note text."; Perl: "Notes", nothing listed.

Rust (58l): an enotez binding (`enotez_sty.rs`) loads the package raw and records each note as it is
made; the notes are `ltx:note role="endnote"` (as endnotes.sty's binding), each in the list `ent` and
in its partition's `ent<k>`, and `\printendnotes` prints the package's own heading and split titles
with an `ltx:TOC` per partition. `split=section|chapter` advances by comparing what the headings
changed at each note — counters, printed numbers and the starred-heading counters — since
`\section`/`\chapter` are locked (enotez prepends to them, :884-905); under `split=section` a
`\chapter` heading starts a split where pdflatex waits for a `\section`. skeldoc manual: recall
15.1 % → 98.6 %, schema errors 15 → 0; mla-example 1 → 0. Guards
`perfect_kernel_batch58::{enotez_notes_are_listed, enotez_split_lists_each_section, enotez_split_by_chapter, enotez_marks_texts_and_repeated_lists, enotez_split_by_section_in_a_book}`.

## 417. A note in a heading or caption steps its counter twice: the toc copy is digested as typeset

Perl builds a heading's `toctitle` and a caption's `toccaption` by digesting the argument again
(latex_constructs.pool.ltxml:627-628, :3187-3190). LaTeX writes that copy to the .toc/.lof untypeset
(latex.ltx:17351-17362, :17401-17412), so a note in it never steps a counter at the heading.

Trigger: `\section{One\protect\footnote{In title.}} A\footnote{After.}` — pdflatex: marks 1, 2; Perl:
1, 3 (a hidden copy took 2). A caption's `\protect\footnote` appears twice (`toccaption` and
`caption`); a heading's `\footnotemark` leaves its `\footnotetext` unpaired.

Rust (58n): the toc copy is digested with notes, `\label`, `\index`, `\glossary` neutralized
(DIVERGENCES #399). Witnesses 2605.15775 (marks 1 then 3), 2605.09284, 2605.19033. Guards
`perfect_kernel_batch58::{note_in_a_title_or_caption_steps_once,
footnotemark_in_a_heading_pairs_with_its_text}`.

## 418. A copyright year without a holder reads ", 2020"

`\lx@add@copyrightyear` (Base_Utility.pool.ltxml:606-608) tests `\ifx.\lx@copyright@holder.`, which compares
`.` with the macro and never holds, so a year given without a holder is printed after a comma.

Trigger: `\documentclass{aipproc}\copyrightyear{2020}` — Perl `<date role="copyright">, 2020</date>` (and
any acmart document before 58q).

Rust (58q): `\ifx\lx@copyright@holder\@empty`, the year alone; a holder then a year still reads "AIP,
2020". Guard `perfect_kernel_batch58::copyright_year_without_holder_is_the_year`.

## 419. acmart's `\setcopyright` keyword becomes the copyright, its statement is lost

Perl's acmart binding maps `\setcopyright{#1}` to `\lx@add@copyright{#1}` (acmart.cls.ltxml:57): the
copyright date reads the mode's keyword ("rightsretained"), and the owner and permission texts acmart
prints in the first-page footnote (acmart.cls:2014-2198, printed at :2264-2296) are lost; `nonacm`,
`acmcp` and `authorversion`, which suppress them, are not read.

Trigger: `\documentclass[sigconf]{acmart}\setcopyright{rightsretained}\copyrightyear{2020}` — pdflatex:
"© 2020 Copyright held by the owner/author(s)." and the permission paragraph; Perl:
`<date role="copyright">rightsretained</date>`, then ", 2020".

Rust (58q): the binding ports acmart's `\setcopyright` choice key, `\setcctype`, `\@copyrightowner`,
`\@copyrightpermission` (the cc text names the licence without its logo) and `\copyrightyear`; at
`\maketitle` the permission text is a `license` pubnote and `\copyright\ <year>\ <owner>` the
copyright date, under acmart's conditions and through `\footnotetextcopyrightpermission` (a document
that renews it to nothing prints none, 2605.24417). Witnesses 2605.02222, 2605.03623, 2605.11901 gain
the statement; 2605.30212 (`nonacm`) none. A public-domain or `none` mode's bare year carries no
copyright sign (`role="copyrightyear"`). Without `\copyrightyear` or `\acmYear` the year is acmart's
`\the\year`, the conversion's (as pdflatex's at compile time). The `nonacm`, `authorversion` and
`acmcp` options are read from `\opt@acmart.cls` as xkeyval reads them (`\PassOptionsToClass` counts,
case-sensitive keys, last setting wins). Open: a document's own non-empty `\footnotetextcopyrightpermission` lands in the
first paragraph; the cc licence's logo and link and the `authorversion` statement (acmart.cls:
2300-2312) are not ported; an unknown `\setcopyright` value is accepted silently. Guards
`perfect_kernel_batch58::{acmart_copyright_statement_is_frontmatter, acmart_nonacm_prints_no_statement,
acmart_renewed_sink_prints_no_statement, acmart_public_domain_year_has_no_copyright_sign,
acmart_options_follow_xkeyval, acmart_under_a_wrapper_class_keeps_its_setters,
setter_let_undefined_loads_cleanly}`. A class that loads acmart keeps acmart's setters: the binding's
`\copyrightyear` has the store shape of the raw-class reroute's table (K11, `frontmatter_stores.rs`),
which skips a setter whose current definition is the one a binding's load left
(`latexml_core::binding::store_setters`, snapshotted around each binding load); a raw class's later
redefinition is still rerouted. Open (K11, not acmart-specific): a raw class that defines a setter
and never calls it has the class default harvested at `\maketitle` even where its own `\@maketitle`
would not print it (a wrapper over `nonacm` acmart with its own `\copyrightyear` gives "© <year>").


## 420. `\pgfmathsetlength\reg{+…}` reads past its argument

The `+` fast path of Perl's `\pgfmathsetlength` (pgfmath.code.tex.ltxml:412-416) unreads the
argument and calls `readGlue` on the live input, so the optional space and `plus` keyword scan after
the unit expand the next token. pgf assigns `#1#2\unskip` (pgfmathcalc.code.tex:30-38): the scan
ends with the argument. In pgf's decoration automaton the next token is
`\ifdim\pgfdecoratedremainingdistance<\pgf@x` (pgfmoduledecorations.code.tex:1020-1035), evaluated
against the stale `\pgf@x`; text along a left-to-right path stopped short (wheelchart `arc data`:
"The arc data in slice N did (possibly) not fit").

Trigger: `\makeatletter\pgf@x=5pt \pgfmathsetlength\pgf@x{+0pt}\ifdim 1pt<\pgf@x Y\else N\fi` —
Perl "Y", pdflatex "N".

Rust (59b): the value is read from the argument alone (`reading_from_mouth`), a dimension for a
dimen register and glue for a skip register. Guard
`perfect_kernel_batch59::pgfmathsetlength_reads_only_its_argument`. The Perl-copied golden
`tests/tikz/unit_tests_by_silviu.xml` re-blessed: its `decoration=zigzag` line now zigzags the
whole 3cm, as pdflatex draws it (Perl stopped after ten segments and drew the rest straight).

## 421. A pgfmath function body runs outside a group

Perl's `pgfmath_apply` (pgfmath.code.tex.ltxml:452-455) digests `\pgfmath<name>@{…}` with no
group; `\pgfmathparse` evaluates inside `\begingroup … \pgfmath@smuggleone\pgfmathresult
\endgroup` (pgfmathparser.code.tex:21, :145-148), so what a function body defines locally does not
outlive the parse. tikzmath declares a function's parameters as `\cx=#1` in the body
(tikzlibrarymath.code.tex:690-703): ungrouped, `\cx` stays a parameterless macro after the call,
and an indexed variable of the same name, `\cx1` (:328-333, :394-399), reads the leaked value
followed by `1`.

Trigger: `\tikzmath{function F(\cx) {\dx = \cx;}; \cx1 = 0; F(\cx1); F(\cx1);}[\cx1]` — Perl
`[11]` (Rust before 59b `[0.011]`), pdflatex `[0]`; 2605.28612 figure A2.F7 drew its nodes at
111 cm.

Rust (59b): a parse that calls a user function opens one group for the rest of the parse
(`pgfmath_grammar::evaluate`), as `\pgfmathparse` does — a later call of the same parse sees an
earlier one's local definitions (`loc(5)+rd` is 5.0); the units flag, global in pgf, is set again after
the group. Guard
`perfect_kernel_batch59::pgfmath_function_body_is_grouped`. Open (shared): a `\draw` inside the
body draws nothing — the call's boxes are discarded (RED `graphics-tikz/tikzmath_function_draws`).

## 422. fancyhdr's `\f@nch@setoffs` is undefined

Perl's fancyhdr binding (fancyhdr.sty.ltxml:27-75) defines the user commands only. A document that
re-derives the running heads' offsets after `\newgeometry` calls the internal
`\f@nch@setoffs` (fancyhdr.sty:668) and gets `Error:undefined:\f@nch@setoffs`.

Trigger: `\usepackage{fancyhdr}\pagestyle{fancy}` … `\newgeometry{left=2pt}\makeatletter\f@nch@setoffs\makeatother`
— Perl 1 error, pdflatex 0. Witness: the wheelchart manual (wheelchart.tex:2743), which completes
since 59b and then raised this error.

Rust (59e): defined empty — it sizes running heads, which are not converted. Guard
`perfect_kernel_batch59::fancyhdr_setoffs_after_newgeometry`.

## 423. An `&` in an `\index`/`\glossary` entry is a stray alignment

LaTeX's `\index`/`\glossary` only write the entry, read under `\@sanitize` (latex.ltx:17720-17740; :1778
makes `&` other), so its `&` is an ordinary character — robustglossary's documented entries are
`formula&explanation`. Perl's `\index` (latex_constructs.pool.ltxml:4397, `SanitizedVerbatim` then
re-tokenized) and `\glossary` (:4424-4437) digest the entry with `&` an alignment character: `Error:unexpected:&
Stray alignment "&"`, and the `&` is lost from the phrase and its key. Perl's `\glossary` in the text flow also
warns and drops the entry.

Trigger: `\makeindex` … `Text.\index{$B$&Borel}` — Perl 1 error, pdflatex 0. Witness: the robustindex manual
robustsample (robustsample.tex:45/:59, 2 errors).

In an alignment it is worse: the entry's `&` ends the enclosing cell — `\begin{tabular}{>{\bfseries}c<{X}c}
e\index{y&z} & f` puts the cell's `<{X}` template inside the entry, and `\index{x&y}` in an `align` loses the
rest of the document. And Perl's `neutralizeFont` sets the entry in OT1, so under `[T1]{fontenc}` an entry's `<`
reads `¡`.

Rust (59k): `\@index` lets `&` to `\&` while the entry is digested (natbib's idiom, natbib.sty.ltxml:632) and
raises the alignment level by one for it, as the argument's braces do in TeX (tex.web §358), so the entry's `&`
is never a column end; it reverts to nothing and neutralizes the font as Perl's `\@index` does (:4409-4412),
keeping the text encoding (OXIDIZED_DESIGN_DIVERGENCES #411). Guard
`perfect_kernel_batch59::index_entry_ampersand_is_literal`.

## 424. `\hypertarget` around a display puts `ltx:anchor` inside `ltx:equation`

hyperref's default `\Hy@nestingfalse` (hyperref.sty:323) makes `\hypertarget{name}{text}`
`\hyper@@anchor{name}{\relax}text` (:4805-4810): a point destination, then the text. Perl's `\hypertarget`
(hyperref.sty.ltxml:240-258) absorbs the text and `localized_anchor` wraps the first node an anchor may hold
(`wrapNodes`, Document.pm:1972-1995, checks no content model), which for a display is its `ltx:Math`: the anchor
lands inside `ltx:equation`, where the schema allows none (jing "element anchor not allowed here"); in a `\parbox`
Perl wraps the whole `ltx:para` (`Error:malformed`).

Trigger: `\usepackage{hyperref}` … `A \hypertarget{d}{\[ y=2 \]} b.` — Rust 0 errors and 1 jing error before 59m,
pdflatex 0; with the `\parbox` case beside it (repro block-model/hypertarget_display_text_anchors_before_it) Perl
gives 2 `Error:malformed` (anchor, `_CaptureBlock_`) and 3 jing errors. Witness: philexmanual (philex.sty:136, `\lb[c]{compo}{\[…\]}` →
`\parbox{\centro}{\centering \hypertarget{#2}{#3}\philpunct}`).

Rust (59m): in running text, non-horizontal text (a display, list, tabular, footnote) follows a bare destination
(hyperref's order); the walk wraps a node only where its parent may hold `ltx:anchor`, treats refused anchor content
(a display's `ltx:Math`, a figure's `ltx:graphics`/`ltx:rule`, a `ltx:tabular` in a block) as one unit, and never
enters `ltx:tags` or `ltx:MathBranch` — so the destination is never hidden in a numbering tag, a cell or math
`XMText` (where the math pass renames its id). With nothing wrappable, the bare-anchor fallback places it at the
insertion point: in vertical mode, a `\parbox`, after `\item`, in `quote`/`minipage`/`p{}` cells, the paragraph after
the block — one block late. Guards `perfect_kernel_batch59::{hypertarget_display_text_anchors_before_it,
hypertarget_block_text_keeps_a_visible_destination}`, `perfect_kernel_gemini::hyperdef_anchor_holds_only_its_text`.

## 425. A braced file name is read with its braces by `\openin`, `\openout` and `\font`

TeX Live's `scan_file_name` takes a braced name's group as the name (`\openin\r{article.cls}`; tex.ch, TeX Live 2020+),
which is how pgfmanual's example machinery opens its sources (pgfmanual-en-macros.tex:1663
`\openin\examplesource\expandafter{\codeexamplesource}`). Perl's `TeXFileName` reads the braces as part of the name
(Base_ParameterTypes.pool.ltxml:296-307) and only `\input` strips them (TeX_FileIO.pool.ltxml:161-170), so `\openin`
looks for `{article.cls}`, finds nothing, and the stream is at end of file.

Trigger: `\newread\r \openin\r{article.cls}\ifeof\r NOFILE\else OPEN\fi` — Perl and Rust before 59s `NOFILE`, pdflatex
`OPEN`. Witness: pgf-pie-manual (its code examples print empty).

Rust (59s): `tex_file_name` (base_parameter_types.rs) takes a braced name's group as the name and drops `"` (tex.ch
`more_name`: a quote is not part of a name), beside the `TeXFileName` reader, for `\input` (which still loads
LaTeX.pool for a braced name, as Perl does), `\openin`, `\openout` and `\font`.
Repro singletons/file_names_versions_fonts_textblocks (p1); guard
`perfect_kernel_batch59::file_names_versions_fonts_and_textblocks_are_kept`.

## 426. A document's own `\fileversion` and `\filedate` are refused

latex.ltx defines neither (doc.sty's `\GetFileInfo` does); Perl predefines both empty (latex_constructs.pool.ltxml:5726-5727),
so a package's, a class's or a document's `\newcommand\fileversion{…}` is `\newcommand`'s "already defined" and the
text prints the empty definition.

Trigger: `\newcommand\fileversion{0.3d}` … `Version \fileversion.` — Perl `Version .` with an error, pdflatex
`Version 0.3d.` Witnesses: sepfootnotes (`\thanks{Version \fileversion, dated \filedate.}`), amshelp, classics,
clipboard, othelloboard; umthesis.cls:73-74 and tkz-doc.cls:23 define them in a class.

Rust (59s): neither is predefined, as in latex.ltx (OXIDIZED_DESIGN_DIVERGENCES #417). A binding does not run its
package's prologue, so after a bound package that `\def`s them (listings.sty:19-20, ae.sty:18-19, lineno, newtxmath,
…) both stay undefined where pdflatex has the package's values: a document that prints them then gets an
undefined-control-sequence error, which no paper of the 3,003-paper arXiv A/B did. Reading the prologue statically is
a dead end: the definitions sit in branches TeX never runs (setspace.sty:275-280, the LaTeX 2.09 branch), in macro
bodies (amsdtx.cls:481), or twice with the last winning (g-brief.cls:36-39). The empty definitions had also hidden a
binding's `\ver@<pkg>` holding the file's `\Provides*` text unexpanded (`provides_version_of`, content.rs): listings'
`[\filedate\space\fileversion\space(Carsten Heinz)]`, which LaTeX's First Aid for listings expands (2 errors in every
document loading listings). The text is now taken as `\xdef` leaves it when its only macros are the kernel's `\space`
and `\ `, and falls back to `\fmtversion` otherwise. Repro p2 (with setspace, whose prologue pdflatex never runs);
guards as #425, `perfect_kernel_batch56::listings_reads_lstlocal_cfg`, `content::declared_info_tests`.

## 427. A font family LaTeX does not know leaves the previous font, `\nullfont` included, in force

latex.ltx `\selectfont` never fails: when no font shape is declared for the selected encoding/family/series/shape,
even after trying `<enc><family>.fd` (`\try@load@fontshape`, latex.ltx:10605-10620), the shape, then the series, then
the family become the encoding's defaults (`\wrong@fontshape`, :10689-10705, from `\D@<enc>`,
`\DeclareFontSubstitution` :10367-10391; cmr/m/n for OT1 and T1), with a font warning. It runs inside
`\define@newfont`'s group (:10593-10603), so `\f@family` and the rest keep the document's codes. Perl's `\selectfont`
(latex_constructs.pool.ltxml:5202-5221) gives an Info and keeps the previous family: inside a pgf picture that is
pgf's `\nullfont`, and a node's text in an unknown family is dropped. `\DeclareFontFamily` and
`\DeclareFontSubstitution` are no-ops (:2691, :2731), so neither a declared family nor a later encoding's defaults are
known.

Trigger: `\usepackage{tikz}` … `{\fontfamily{verdana}\selectfont \tikz\node{Inside node};}` — Perl and Rust before 59s
an empty picture, pdflatex the node's text; `{\bfseries A {\fontfamily{verdana}\selectfont B}}` sets B in cmr/m/n.
Witness: pgf-periodictable (pgfPT.colorSchemes.info, `\usefont{T1}{verdana}{m}{n}`).

Rust (59s): a family that is neither in LaTeXML's font tables nor a fontmap's encoding, neither declared
(`\<enc>+<family>`) nor loadable (`<enc><family>.fd`, lowercased or as written, as latex.ltx:10617-10619 tries —
autoinst families are mixed-case, `OT1LinuxLibertineT-TLF.fd`; remembered per name), takes the encoding's default family,
series and shape; `\D@<enc>` runs in a group, and the Info names the substitute. `\DeclareFontFamily` defines
`\<enc>+<family>` and `\DeclareFontSubstitution` defines `\D@<enc>`, as the kernel does (without its unknown-encoding
error). Residual (Perl the same): an undeclared series or shape of a known family (`\fontshape{zz}` in cmr; pdflatex
cmr/m/n) keeps the previous one — LaTeXML loads no `.fd`, so it cannot tell a declared shape from an unknown one.
Repro p3; guard as #425.

## 428. textpos's `[absolute]` blocks are never placed

With `[absolute]`, textpos collects every `textblock` in `\TP@holdbox` (textpos.sty:372-376) and places it at shipout
(`shipout/background`/`foreground` hooks, `\EveryShipout`, :389-414). LaTeXML has no shipout, so the box is never
used and the blocks' text is lost, without a diagnostic. Perl has no textpos binding and loads the package raw.

Trigger: `\usepackage[absolute]{textpos}` … `\begin{textblock*}{1cm}(7cm,14.5cm)Charlie\end{textblock*}` — Perl and
Rust before 59s nothing, pdflatex `Charlie` at (7cm,14.5cm). Witnesses: pdfcomment's examples ×3, stubs_ex,
niepraschk-eso-pic, ftc-notebook.

Rust (59s): the contrib binding `textpos_sty.rs` loads the package raw and places each absolute block's text box
(`\TP@textbox`, at its natural size) where the block ends, emptying the hold box (OXIDIZED_DESIGN_DIVERGENCES #416);
a block that shows nothing is not placed (`typesets_content`): autonum.sty:159-173 captures each display
environment's `\\` and `\label` in an absolute block at (0,0) holding an empty `equation` (2605.31413). A block of
rules alone is placed decoration and is dropped as well, as eso-pic's rule-only overlay is.
Repro p4; guard as #425. In the default relative mode a block written inside a paragraph is lost as well: textpos
sets it with `\vadjust` (:383-385), whose material is dropped (RED boxes-groups/
textpos_relative_block_in_a_paragraph_is_kept; the `\vadjust` kernel item).

## 429. `\autoref` names: forced English, no babel language, no fallback to `\<type>name` or the counter

hyperref only `\providecommand*`s its English names (hyperref.sty:8293-8312), hooks each babel language present at
load into `\extras<lang>` (`\HyLang@DeclareLang`, :3120-3180), and names a reference by its anchor's type, the
counter, trying `\<ctr>autorefname`, `\<ctr>name`, then both without a trailing `*` (`\HyRef@testreftype`,
:8236-8278). Perl's binding runs `\HyLang@english` at load ("For now...", hyperref.sty.ltxml:685-686) and tries only
`\<type>autorefname` (:373-382): a class's or document's earlier `\figureautorefname` is overwritten, a German
document says "Figure", `\newtheorem{lemma}{\lemmaname}` and a `[theorem]`-numbered proposition get a bare number.

Trigger: `\newcommand\figureautorefname{Fig.}\usepackage{hyperref}\newtheorem{theorem}{Theorem}`
`\newtheorem{prop}[theorem]{Proposition}` … `\autoref{f}; \autoref{p}` — Perl and Rust before 59t "Figure 1; 1",
pdflatex "Fig. 1; Theorem 1"; `\usepackage[ngerman]{babel}` + hyperref: "Figure 1" for pdflatex's "Abbildung 1".
Witnesses: 2605.01034, 2605.28533, 2605.30447 (shared-counter theorems, 8 references in the 3,003-paper A/B).

Rust (59t, hyperref_sty.rs): hyperref.sty:3016-3105 (the five language blocks the binding lacked), :3120-3180 and
:8293-8312 verbatim; babel's own `\selectlanguage` at `\begin{document}` runs the extras. `\lx@autorefnum@@` tries the
target's `\<type>autorefname`, then hyperref's chain on the counter (OXIDIZED_DESIGN_DIVERGENCES #418). subcaption's
sub-types get caption3's `\autoref` name (caption3.sty:1792: `\subfigureautorefname` = `\figureautorefname`; its
empty `\subfigurename` is left out, which here would make a " 1a" `typerefnum` tag). Repros
singletons/autoref_names_follow_hyperref, babel-lang/autoref_names_follow_the_babel_language; guards
`perfect_kernel_batch59::{autoref_names_follow_hyperref, autoref_names_follow_the_babel_language}`. RED
babel-lang/ngerman_tilde_is_a_nobreak_space (the German separator is U+0020), babel-lang/greek_autoref_name_keeps_its_sigma
(`\textsigma` loses LGR in the tag), singletons/hyperref_language_option_names (`\usepackage[ngerman]{hyperref}`: the
`Hyp` language keys are not set). The binding's French block was Perl's old copy ("Figure", "\'Equation"); all
sixteen language blocks are now hyperref.sty's.

## 430. A listings `literate` key written with a control symbol never matches

listings reads each token of a `literate` key as the character it stands for (`\lst@CArgX`, listings.sty:1134;
`\lst@MakeActive@` :139-166), so `literate={\\Real}{$\mathbb{R}$}1` replaces the source's `\Real`. Perl takes the
tokens' text (`ToString($pattern)`, listings.sty.ltxml:1115): the key is the string `\\Real`, which no source holds,
silently.

Trigger: `\lstset{literate={\\Real}{{$\mathbb{R}$}}1}` … `x : \Real` in a `lstlisting` — Perl and Rust before 59t
`x : \Real`, pdflatex `x : R`. Witness: 2305.00594 §4 (arXiv html_feedback #6486).

Rust (59t, listings_sty.rs `\lst@@literate`): the key is built token by token, a control symbol giving its character
(`lst_deslash`). Repro singletons/listings_linerange_and_literate_keys; guard
`perfect_kernel_batch59::listings_linerange_and_literate_keys_select_and_replace`.

## 431. `\printglossary[title=…]` keeps the type's title

glossaries' `title=` sets `\glossarytitle`, which the heading prints instead of the type's title
(glossaries.sty:7546-7548, :6529-6552). Perl's `\lx@printglossary` reads only `type=` (glossaries.sty.ltxml:111-121).

Trigger: `\printglossary[title=Symbols]` — Perl and Rust before 59t "Glossary", pdflatex "Symbols". Witness:
glosmathtools en/fr.

Rust (59t, glossaries_sty.rs): the `title` key's tokens are the title. Repro index/glossary_title_and_nomentbl_columns;
guard `perfect_kernel_batch59::glossary_title_and_nomentbl_columns_are_kept`.

## 432. `\lastbox` takes any item, a paragraph's end included

tex.web §1080 takes the last node only when it is an hlist or vlist (a box); at a character, a formula, a rule, glue, a
kern, a penalty or a space the box is void and the item stays, and a paragraph's end is no node at all. Perl pops the
last item of the list whatever it is (TeX_Box.pool.ltxml:596-597): `\hbox{xy\lastbox}` loses its "y", and the classic
line count `\loop\unskip\unpenalty\setbox0\lastbox\ifvoid0…\repeat` over a one-line paragraph counts the paragraph's
`\par` too and never ends on a space (Rust before 59w 2; Perl loops, the closing box never void:
caesar_book.cls:106-115, sidenotes caesar_example). Perl's `\lastskip` reads a skip's `width`
(TeX_Glue.pool.ltxml:157-164), so after a `\vskip` it is 0pt.

Trigger: `\setbox0\vbox{A title line here\par \count255=0 \loop\unskip\unpenalty\unskip\setbox0\lastbox\ifvoid0
\xdef\nl{\the\count255}\else\advance\count255 1 \repeat}[\nl]` — pdflatex [1], Rust before 59w [2].

Rust (59w, stomach.rs `peek_own_box`/`is_list_tail_transparent`/`end_graf`): `\lastbox`, `\unskip`, `\unkern` and
`\unpenalty` look at the last item past the paragraph's closing `\par` marker, a `\noindent` and comments, and
remove it only when it is theirs; `\lastbox` takes a box or a paragraph's line, never a character, formula, rule,
penalty, glue, kern or space (`\indent` and the phantoms are hboxes, though flagged as space). A paragraph holding
only its indent box (`\leavevmode\par`) leaves an empty line at an explicit `\par` (§1096 `end_graf`); one without an
indent box and with nothing listed is null (`\noindent\par`; `\indent\setbox0\lastbox\par`, the indent box taken —
`ParagraphStart::implicit_indent`, `take_implicit_indent`). `\lastpenalty`, `\lastkern` and `\lastskip` read past the
marker too, and `\lastskip` of a `\vskip` is its height. Repro boxes-groups/box_primitives_lastbox (cases 8, 12-16,
18-24, 27-29, 31-32); guard `box_primitives::lastbox`. Residuals:
- a group in horizontal mode is a list of its own, taken whole (case 26);
- no `\parskip` glue goes on the vertical list before a paragraph (§1091, case 25);
- LaTeX's `\par` (`\para_end:`, latex.ltx:9088) `\unskip`s first, so `\noindent\hfill\par` is null in LaTeX (case 30);
- the implicit indent box `\lastbox` takes is void here, not TeX's empty `\hbox to\parindent`
  (`\leavevmode\setbox1\lastbox`: TeX a 15pt box);
- every `\par` that `leave_horizontal` inserts only repacks, so no empty line follows TeX's §1094 `head_for_vmode`
  `\par` (`\vbox{\leavevmode\vskip3pt\hbox{B}}` 9.83pt high, TeX 15pt; box_primitives_unpack case 22), nor the end
  of a box's contents, which `leave_horizontal_internal` only repacks (§1085; `\parbox[b]{3cm}{A\par\leavevmode}`
  6.83pt high, TeX 18.83pt; case 21) — `leave_horizontal` also ends a paragraph before a block LaTeXML sets apart
  (a minipage), where TeX's paragraph goes on, and an empty line there grew 2605.02442's table cells;
- a non-empty `\everypar` is digested as a list of its own, which keeps the paragraph (`\everypar{\noindent}
  \leavevmode\par\hbox{B}` 6.83pt high, TeX 12pt);
- `~` does not start a paragraph (Perl `\lx@NBSP` alike, Base_Utility.pool.ltxml:51-53), so `~\par` after vertical
  material is a 3.33pt box, not a line.
The `\un*` unpacking (R1) and an hbox in vertical mode as its own line (R6) are the box family's next stages
(agent_reports/2026-10-02_box_family_design.md).

## 433. `\vphantom` takes its argument's width

latex.ltx:15613-15617 `\finph@nt` (plain.tex:1031 alike) builds a `\vphantom` as `\null` with the argument's height
and depth; only `\phantom` and `\hphantom` copy the width. Perl's `\vphantom` sets all three from the argument
(math_common.pool.ltxml:684-687), so text after it is pushed right in the sizing.

Trigger: `\setbox1\hbox{x\vphantom{y}}[\the\wd1]` — pdflatex [5.2778pt], Rust before 59w [10.5556pt].

Rust (59w, math_common.rs `\vphantom`): the width is 0pt. Repro boxes-groups/box_primitives_unpack (case 20); guard
`box_primitives::unpack`. The tikz fixture ac-drive-components' `$\vphantom{+}-$` node narrows from 15.37pt to 4.61pt
(golden updated; TeX's is the minus alone, 7.78pt — the rest is our minus glyph's metric).

## 434. `\unhbox` and `\unhcopy` put the box back, not its list

tex.web §1110 `unpackage` appends the box's list to the current list and discards the box: its dimensions (a `to`
width) and its being one item. Perl puts the whatsit of `\setbox0\hbox{…}` back whole (`Whatsit::unlist` returns the
whatsit; TeX_Box.pool.ltxml:705-722), so `\lastbox` after `\unhbox` takes the whole box and `\hbox{\unhcopy0}` keeps
the `to` width.

Trigger: `\setbox0\hbox to 3cm{ab}\setbox1\hbox{\unhcopy0}[\the\wd1]` — pdflatex [10.55559pt], Rust before 59y
[85.35826pt]; `\setbox0\hbox{a\hbox{b}}\setbox1\hbox{\unhbox0\global\setbox2\lastbox}` — pdflatex box 2 = "b".

Rust (59y, tex_box.rs `hlist_of`): `\unhbox`/`\unhcopy` of an `\hbox` whatsit (LaTeX's `\sbox`/`\savebox` too) put
its contents' items back — `\unhcopy` as copies (`copy_node_list`: a `\wd` set on one leaves the register's box
alone) — marked as paragraph material when they land in a paragraph (a `\parbox` spliced at a paragraph's start was
set apart as its own paragraph) and as items of its line, whatever their own mode (`in_hlist`, §1076: tcolorbox's
`sidebyside` halves, minipages `\unhbox`ed side by side, stacked — 2605.11712, 2605.31228); what a box inside them
holds stays restricted, an environment's body too. An alignment cell keeps a phantom opening it as its content, not as
padding (#435: a TikZ `align=` line opening with `\phantom` shifted the whole column, 2605.03603). In math mode the box goes back whole,
as in Perl: LaTeXML is in math mode inside `$\emph{…}$`, where TeX is not, so TeX's "Incompatible list can't be
unboxed" for true math mode is a residual (unpack case 34). Repros boxes-groups/box_primitives_unpack (cases 2,
27-29, 33, 37-38), box_primitives_lastbox (case 4), tikz_align_line_padding (cases 1-3); guards `box_primitives::{unpack, lastbox,
unhcopy_in_math_text_keeps_the_box}`. Residuals: `\unskip` removes no word space (unpack case 1); a spliced `\vbox`
is set apart as a block (case 30); a horizontal penalty leaves no item (case 31); the unpacked material keeps its own
color under `\color` (case 32); `\copy` shares the box, so a `\wd` set on an item of the `\unhbox`ed original
reaches the copy (case 35); a `\rotatebox` spliced at a paragraph's start puts the rest on a line of its own, as typed
(case 36, boxes-groups/rotatebox_starts_the_paragraph — the `\hbox` wrapper hid it before 59y); `\unvbox` of a vbox
still puts the box back (R1 vertical, with R6); minipages typed into a paragraph stack (case 39; Perl's repack too,
Stomach.pm:458-461); a cell's leading control spaces become column padding (tikz_align_line_padding case 4; Perl's
per-column padding, Alignment.pm:567-568, 659).

## 435. An alignment cell's leading phantom is taken for padding

Perl peels a cell's leading and trailing space-flagged items as its padding (TeX_Tables.pool.ltxml:498-500) and drops
padding under 1.5em; a phantom is flagged as space (math_common.pool.ltxml:645-650), so `\phantom{0}5` loses the
phantom and the 5 no longer lines up under `12`. In TeX a phantom is a box, the cell's content. Wider phantoms became
column padding, which a column takes at its widest cell's, so every row moved (a TikZ `align=` line, whose lines are
`\halign` cells).

Trigger: `\begin{tabular}{l}\phantom{0}5\\12\end{tabular}` — pdflatex sets the 5 under the 2; Perl and Rust before
59y set it under the 1.

Rust (59y, tex_tables.rs `is_space_flagged`, tex_box.rs `is_space_flagged_box`): a phantom with a width ends the
leading peel as content (`<text class="ltx_phantom">0</text>5`); zero-width ones (`\vphantom`, `\mathstrut`) are peeled
as before — kept, they moved a cell's fill alignment and the header guess. Repros boxes-groups/alignment_cell_phantom
(cases 1, 3-4), tikz_align_line_padding (cases 1-3); guards `box_primitives::{alignment_cell_phantom,
tikz_align_line_padding}`. Residuals: a trailing phantom is still dropped, and a row holding only a phantom with it
(`\lx@column@trimright`; alignment_cell_phantom cases 2, 5); leading control spaces become column padding
(tikz_align_line_padding case 4, per-column padding); an `\hfill` after a leading phantom is no longer read as the
cell's fill (`\phantom{0}\hfill 5`: left, where TeX sets the 5 right), as a mid-cell `\hfill` already is not.

## 436. `\unskip` keeps an interword space

tex.web §1105-1106 `delete_last`: `\unskip` removes the last node of the current list when it is glue, and in
horizontal mode an interword space is glue (§1041 `app_space`), as are `\ `, `\hskip`/`\hspace`, `\quad`, `\hfil` and
the glue after `~`'s penalty. Perl's `\unskip` removes only `isSkip` items (TeX_Glue.pool.ltxml:105-115), which only
`\hskip`/`\vskip` are: a space is a plain box (Stomach.pm:244-250), so `a \unskip,` keeps the space before the comma.
`\lastskip` reads the same items, so after a space it is 0pt.

Trigger: `[a \unskip,]` — pdflatex `[a,]`, Perl and Rust before 59z `[a ,]`. In documents: elsarticle `\sep`
(`\unskip,\space`: keywords "alpha , beta"), `\def\and{\unskip{}, \ignorespaces}` author lists ("Alice , Bob",
2605.02925), amsmath `\tag{A }` "(A )", ams `\and`, the apacite `.bbl`'s `\unskip\ \newblock`.

Rust (59z): glue is flagged `isSkip` where it is made — the catcode-10 space (stomach.rs), `\ `, `\hfil`/`\hfill` (width
0pt), `~`/`\nobreakspace`, plain `\enskip`/`\quad`/`\qquad`/`\hglue`/`\<TAB>`, unstarred `\hspace` — and kerns `isKern`
(`\enspace`, `\/`, and in text `\,`/`\!`/`\:`/`\>`/`\;` and LaTeX's `\thinspace`, `\negthinspace`, `\medspace`,
`\negmedspace`, `\thickspace`, `\negthickspace` — the `\tmspace` kerns of latex.ltx:15669-15681, at their em widths
for `\lastkern`), so `\unskip`, `\unkern`, `\lastskip` and `\lastkern` act as TeX's; `\lastskip` of a space is its
width, and `\ ` and `\<TAB>` are as wide as the font's interword space (Perl: a fixed 0.5em and 1em; in math still
0.5em, where TeX takes the text font's space). french_ldf's
own space removal before high punctuation is now the kernel's, with french.ldf's `\lastskip>1sp` test. Repro
boxes-groups/unskip_horizontal_glue; guard `box_primitives::unskip_horizontal_glue`. Residuals: `\lastskip` is a
dimension (no stretch or shrink); `\/` has no width; a horizontal penalty leaves no item (#419); `\hspace*`'s
`\z@skip`; a group's list (`{a }\unskip`, `\textbf{a }\unskip`) is not reached; `\leaders`/`\cleaders`, a literal
U+00A0 and math-mode muglue (`$a\,\unskip$`, `\mskip`) are not flagged; code that saves `\lastskip`, unskips and
re-adds it (cite.sty `\cite@adjust`'s idiom) writes the space back as `\hskip` of its width, a U+2004 where the space
was U+0020 (none in the 3,003-paper A/B).

## 437. pgfmath's `em` is the picture's `\nullfont`'s

pgf evaluates every length after `\pgfmath@selectfont` (= `\selectfont`, pgfmathutil.code.tex:167-174):
`\pgfmathsetlength`'s quick path is `\begingroup\pgfmath@selectfont #1#2\unskip\endgroup` (pgfmathcalc.code.tex:30-38)
and `\pgfmathparse` selects it too (pgfmathparser.code.tex:132), so `em` and `ex` are the document font's even inside a
picture, where `\nullfont` is in force. Perl's native `pgfmath_convert` (pgfmath.code.tex.ltxml:476-485) takes
`$STATE->convertUnit($unit)` in the font in force, and LaTeXML's `\nullfont` keeps the size's metrics.

Trigger: `{\footnotesize\tikz{\pgfmathsetlength{\pgf@xa}{8em}\global\dimen4=\pgf@xa}\the\dimen4}` — pdflatex 68.00098pt
(cmr8's quad), Perl and Rust before 60a 64.00012pt. A node's `text width=8em` frame and centring came out narrower than
its minipage (tests/tikz/consort-flowchart: the picture 333.16pt wide, pdflatex 351.49686pt).

Rust (60a): `pgfmath_font` is the font `\pgfmath@selectfont` selects, by `\selectfont`'s own resolver
(`nfss_selected_font`: family, family-as-encoding, an undeclared family's encoding defaults, series, shape); `em` and
`ex` are converted in it, `\pgfmathsetlength`'s quick path reads in a group under it (the parse path is ungrouped, as
pgf's, so its units flag outlives it), and `width("…")`/`height`/`depth` typeset in it (`\nullfont` measured them 0: a `minimum width={width(…)}` node was 5.73282pt, pdflatex 36.2894pt).
consort-flowchart is pdflatex's 351.5pt (OXIDIZED_DESIGN_DIVERGENCES #423). Repro graphics-tikz/pgfmath_em_selectfont; guard
`picture_sizing::pgfmath_em_is_the_document_fonts`. Residuals: a raw `8em` under `\nullfont` is the size's quad, TeX 0pt
(row 2); a `\fontsize` with
no `\selectfont` after it is not applied (`{\fontsize{14}{16}\tikz…4em}`: pdflatex 56.39935pt, Rust 40.00006pt); the
argument is expanded before the group, so a font-dependent expansion (`+\the\fontdimen6\font`) sees the font in force,
and text after the value (`\pgfmathsetlength\d{+3pt XY}`) is set after the group, in the font in force (pgf sets it inside).

## 438. montex's LMC encoding reads as Latin transliteration

montex's Mongolian Cyrillic encoding (lmcenc.def; mls.sty:196-197 `\mnr` = `\fontencoding{LMC}\selectfont`) puts the
Cyrillic letters at the Latin slots they transliterate and the rest of the alphabet above 127, and the kmr fonts'
ligature program composes `"o` → ө, `ya` → я, `sh` → ш, `<<` → « (kmr10.tfm LIGTABLE, from mcyrill.mf). Perl has no
`lmc.fontmap`: `\lx@fontencoding` falls back to OT1 (TeX_Fonts.pool.ltxml:169-175), so the text reads as its Latin
source and `\No`/`\MyTogrog` (slots 249/250) print nothing.

Trigger: `\usepackage[latin1]{mls}` … `{\mnr Xalx "ond"or <<A>> \MyTogrog\ \No}` — pdflatex "Халх өндөр «А» ₮ №", Perl
and Rust before 60b "Xalx ”ond”or ¡¡A¿¿". S3 recall cannot see it: montex's golden PDFs set the kmr fonts as Type 3
bitmaps with no text layer.

Rust (60b): `lmc_fontmap.rs` decodes each slot to the glyph mccoding.mf/mcyrsymb.mf draw there (cmr's punctuation and
accents otherwise, no glyph at `v`/`V`; kmtt's ASCII glyphs in a typewriter map) and applies kmr10's ligatures to LMC
text only — the montex ones in every LMC font; `` `` ``/`''` → “” (carried on into a vowel: `''o` → ө) and `<<`/`>>` in
the roman ones, as the shared quote ligatures (#451) are other encodings'. A text node's ligatures reach only the run
set in their font (OXIDIZED_DESIGN_DIVERGENCES #424): `a''{\mnr o}` is `a”о`, as TeX never ligates across a font change.
Repro fonts-nfss/lmc_encoding_prints_cyrillic; guard `perfect_kernel_batch59::lmc_encoding_prints_cyrillic`. Witnesses
montex/montex, montex/mlsquick. Residual: montex's other encodings — LMS (Bicig), LMO, LMU, LMA (lmsenc.def …
lmaenc.def) — have no map, so mlsquick's Mongolian-script passages read as their OT1 transliteration ("¡¡cag -i
tukinagulugci¿¿"; RED fonts-nfss/lms_encoding_bicig).

## 439. A TikZ node's text in another size is scaled twice

Perl anchors a measured foreignObject's CSS `font-size` at its whatsit's font size (TeX_Box.pool.ltxml:427-430), while
the text inside carries its `fontsize` relative to the nearest ancestor that declared a size (Document.pm's finalize
pass; `ltx:para`, `ltx:picture`, `svg:*` declare none). Where the picture's surroundings changed the size — a
`\footnotesize` figure, a `{\Large …}` group — or the node has its own `font=`, the browser applies the change twice.

Trigger: `\begin{figure}\footnotesize\tikz\node{Foot};\end{figure}` — pdflatex sets "Foot" in cmr8; Perl writes
`font-size:8pt` around `<text fontsize="80%">Foot</text>`, 6.4pt on screen; `\tikz\node[font=\scriptsize]{Small};` in a
10pt document, 4.9pt (Rust before 60c, anchored at the quad: 5.58pt); `{\Large\tikz\node{Big};}`, `font-size:14.4pt` around
`fontsize="144%"`, 20.7pt (enlarged twice). Witness tests/tikz/consort-flowchart
(`\sffamily\footnotesize` nodes at 6.4pt).

Rust (60c): the anchor is the foreignObject's node font size (the font around the picture), the em widths divide by it
(OXIDIZED_DESIGN_DIVERGENCES #46), and `finalize` declares that size for the foreignObject's content (`_font_anchor`), so
its `fontsize` is relative to the anchor: "Foot" bare at an 8pt anchor, `font=\tiny` at 63 % (62.5 % rounded),
`font=\scriptsize` 70 % of 10pt, "Big" bare at 14.4pt. Repro graphics-tikz/foreignobject_anchor_font_size; guard `picture_sizing::foreignobject_anchor_is_the_content_font`.

## 440. A picture inside a TikZ node's text closes the line's matrix column

pgfsys-latexml.def.ltxml's `\lxSVG@insertpicture` closes `svg:g`s that are not its own. In SVG (a nested picture,
:80-85) it opens a `_scopebegin` group, absorbs the content — which ends in `\lxSVG@closescope`, closing that group —
and then `closeElement('svg:g')`, the group around it. Outside SVG (a picture in a foreignObject's text, :86-107) it
runs `while ($document->maybeCloseElement('svg:g')) { }`, which closes every `svg:g` it finds above the picture. In a
TikZ `align=` node each line is an `\halign` row and column of `svg:g`s (tikz.code.tex:4252-4256), so the column closed
early: the text after the picture left its column, and the matrix closes cascaded up to the line's `\vbox` capture
block ("Closing tag svg:g whose open descendents do not auto-close"). Perl puts the line back as one `\hbox` (#434),
whose own group took the extra close; Rust since 59y splices it, which exposed the over-close.

Trigger: `\tikz\node[align=left]{A\\ \tikz\draw (0,0) -- (1,1);};` — pdflatex 0 errors; Rust 59y-60c 2 errors.

Rust (60d): each branch closes only the scope group it opened, if still open (`maybe_close_node`). Repro
graphics-tikz/nested_picture_in_aligned_node; guard `picture_sizing::nested_picture_in_aligned_node`. Witnesses
codeanatomy/codeanatomy.usage (32 errors), causets/causets_example2 (6), mercatormap (2 jing lines); sweep #136.

## 441. `\vadjust` material runs into the next paragraph, and is read as a macro argument

`\vadjust{…}` builds its material as a vertical list that TeX appends after the line it sits in: after the `{`
(`scan_left_brace`) the tokens are read and processed one by one in the group's own internal vertical list, and a
paragraph begun in it ends at the group's end (tex.web §1097-1100: an `insert_group` save level, closed by
`end_graf`). Perl (TeX_Paragraph.pool.ltxml:32, :139-140) reads a macro argument, queues its tokens
(`PushValue('vAdjust')`) and digests them at the paragraph's `\par`, without ending a paragraph they begin:
`Paragraph\vadjust{X}, More.\par Another paragraph.` gives "XAnother paragraph." (Perl's own t/expansion/aftergroup
golden), where pdflatex sets "X" as a line of its own; a short verbatim in the material (`|\ifx|`, fancyvrb) runs its
`\ifx`, being tokenized before it acts; and the material sees the state of the paragraph's end (a box register a group
restored, a macro redefined, a counter stepped later).

Rust: 60e queued each `\vadjust` as `{<material>\lx@normal@par}`; 60g finds the `{` as §403 `scan_left_brace` does
(gullet.rs `scan_left_brace`: expanded, a `\protected` macro too, past spaces, `\relax` and `\noexpand`ed tokens, else
"Missing { inserted"), reads the material live after it and builds it there
(base_utilities.rs `predigest_insert_group_contents`: the box loop in `inline_internal_vertical`, closed with the
primitive `\lx@normal@par` — TeX's `end_graf`, which reads no `\par` token; pdfTeX does under LaTeX's
`\partokencontext=2`, latex.ltx:22441, so a material ending with `\par` = `\relax` loops in pdflatex), its own
`\vadjust`s going to its paragraph and its indentation its own `\parindent`'s; the paragraph's `\par` sets the built
lists (at the `\par` before a display, their paragraph ends inside the one that goes on). `\pagebreak` in a
paragraph is `\vadjust{\clearpage}`, its braces explicit. A restricted horizontal box keeps what it queued (§655:
`\settowidth`, `\sbox`, `\mbox`, calc's `\widthof`, pgfmath's `width()` material is never set; stomach.rs
`with_own_adjust_queue`), and a `\par` in one, or in math, ends no paragraph and leaves the queue alone (§1096
`end_graf` acts in `hmode` only).
Repros boxes-groups/vadjust_material_is_a_vertical_list, vadjust_material_is_read_live,
vadjust_material_is_built_where_it_is_read; guards `box_primitives::{vadjust, vadjust_material_is_read_live,
vadjust_material_is_built_where_it_is_read}`; fixture t/expansion/aftergroup re-blessed; witness etextools-examples
(6 → 1 error). Residuals, Perl-shared:
- the queue is one list replayed at the next digested `\par`: a parbox's, minipage's, footnote's or p{} cell's
  material leaves its box, a quote's or an item's follows the next paragraph, and an outer paragraph's material is
  taken by a later box's first `\par` — RED boxes-groups/vadjust_material_stays_with_its_line; a vertical-mode
  `\hbox` (§1076 `adjusted_hbox_group`) drops its material, where TeX sets it after the box;
- hboxes LaTeX builds outside the box reader (`\phantom`, `\hphantom`, amsmath's `\text`, `\resizebox`, `\scalebox`,
  `\underline`) print their material, and a tabular inside a box loses its cells' material, which TeX sets after the
  row (§796, §799) — RED boxes-groups/vadjust_material_stays_in_its_hbox;
- pdfTeX's `\vadjust pre {…}` sets the material before the line; it is set after it here (Perl took the `p` as
  its argument and printed "re");
- before a display the material's paragraph is inside the host one, whose `ltx:para` its `\noindent` marks
  `ltx_noindent` — RED boxes-groups/vadjust_material_class_stays_its_own;
- latex_constructs' `\pagebreak[3-4]` (Perl latex_constructs.pool.ltxml:4568, sect12.rs:49-58) queues
  `\vadjust{\clearpage}` in vertical mode too, where TeX forbids `\vadjust` (§1098) and latex.ltx:9227-9229 sets a
  `\penalty`, so the clear waits for the next paragraph's end.

## 442. A glossary loses its parent headings and the keys a package adds

glossaries' `\glsaddkey{key}{default}{…}` (glossaries.sty:2748) adds a display key, with `\glsentry<key>`-style
accessors, that a glossary style may print: glosmathtools.sty:185 adds `descseclang`, a second-language description
that its `nomencl-L1L2` style prints after the first, "d diameter (diametre)". An entry's `parent=` makes it a
sub-entry: makeindex writes the child's index key under the parent's (glossaries.sty:3226-3236), so makeglossaries
lists the parent heading, referenced or not, with its children after it (`\subglossentry`). Perl's binding records
a fixed key list (glossaries.sty.ltxml:56-82), so `descseclang` is dropped, and MakeIndex.pm:467-468 lists the
referenced entries only, flat: glosmathtools' sample loses its "Latin symbols (Symboles latins)" headings and every
French description (recall 89 %). Minimal trigger:

```latex
\usepackage{glosmathtools}\makeglossaries\setglossarystyle{nomencl-L1L2}
\newglossaryentry{latin}{name={latin},description={Latin symbols},descseclang={Symboles latins},sort=1}
\newglosentrymath{d}{d}{description={diameter},descseclang={diametre},sort=d,parent=latin}
\begin{document} \gls{d} \printglossary \end{document}
```

Rust (60i): the binding wraps `\glsaddkey` to record each display key with its default
(`\lx@glossaries@userkeys`), and an entry whose value differs from the default (`\ifx`) gets a phrase classed
`ltx_glossary_userkey`; `\glsaddstoragekey` keys (glossaries.sty:2720-2747: glossaries-extra's `category`, `alias`,
`seealso`; glosmathtools' `dot`, whose default `\glsadd`s) hold data and stay unrecorded. MakeIndex lists the parent
of every listed entry (to a fixpoint), each level sorted, each child after its parent classed
`ltx_glossary_level_<depth>` (the schema nests no list in a `glossaryentry`; LaTeXML.css indents levels 1-2), and the
classed phrases after the description as `ltx:text class="ltx_glossary_<key>"`. The `sort` and `parent` phrases
are the strings makeindex compares, not set in the font: the sort key from `\glo@<label>@sort` (`\@gls@defsort`), as
`\@glo@storeentry` has rewritten `\@glo@sort` with makeindex's quote character (glossaries.sty:3204-3209, :3923), and
both as their source text (Perl digests them: under OT1 the sanitized `\` and `"` print as `“` `”`, so `$\alpha $`
sorted after every letter). DIVERGENCES #426. Residuals:
- a display key prints after the description whatever the glossary style does with it: glosmathtools'
  `nomencl-L2L1` (glosmathtools.sty:176-178) prints it first, its `nomencl`/`nomencl-L1` (:165-167) not at all, and
  its `\glossentry` (:254-258) prints a heading's description only, where the list keeps the entry's name;
- the style's punctuation ("(Symboles latins)") and its `symbol` column (glosmathtools' units) are not reproduced;
- makeindex compares all-digit sort keys as numbers (`9` before `10`), the list as text;
- a parent defined in another glossary is not listed (makeindex lists it in the child's glossary): the child goes to
  the top level;
- the list sorts by the source string as makeindex does, not as the other sorters: `sort=use` (the key is empty when
  the entry is defined, so the list falls back to the label), `\makenoidxglossaries` (it compares the expanded UTF-8
  key: `\'Etude` after `fig`, RED index-bib/glossary_noidx_sort_expands_accents), bib2gls (`sort={custom}`,
  nlctuserguide.sty:3082-3085: a command's entry by its name without the backslash).
Repros index-bib/glossary_user_keys_and_parents, glossary_sort_key_is_its_string; guards
`glossary_refs_post::{glossary_user_keys_and_parents, glossary_sort_key_is_its_string}`; witness glosmathtools
sample_glosmathtools_en/fr (`perfect_kernel_batch56::glosmathtools_sample_is_not_emptied_by_the_math_rebuild`).

## 443. A class's end matter queued on amsart's `\enddoc@text` is lost

amsart.cls:518-520 (amsproc.cls alike) defines `\enddoc@text`, the translators and addresses it sets after the body,
and runs it `\AtEndDocument`; a derived class queues its own end matter there: resphilosophica.cls:94
`\AddtoEndMatter` (`\g@addto@macro\enddoc@text`) holds its `{notes}` collection (:432) and its `\bibliography` (:99).
Perl's ams_core.cls.ltxml defines no `\enddoc@text` and registers no hook, so the queued material never runs, with no
diagnostic: rpsample loses its "Bibliography notes" (G3 slice 4 R2). Minimal trigger:

```latex
\documentclass{resphilosophica}\title{T}\author{A}
\begin{document}\maketitle Body text.
\begin{notes}{Bibliography notes}Collected sentence alpha.\end{notes}
\end{document}
```

Rust (60m, ams_core_cls.rs): amsart's `\enddoc@text`, its `\AtEndDocument` hook and its setters
`\@settranslators`/`\@setaddresses` (:524-577, raw); the accumulators the hook reads (`\thankses`, `\@translators`,
`\addresses`, :505/:571) stay empty, the frontmatter taking the binding's `\address`/`\translator`/`\thanks`, so it
sets only what a class queued or filled itself (smfart.cls:420-422 appends to `\addresses`; pdflatex prints that block
after the body too). Residual: resphilosophica's `\bibliography` (:99 `\AddtoEndMatter{\RESP@bibliography…}`) is
refused by the kernel's locked `\bibliography` (sect11.rs), so the bibliography stays where it stands, before the
queued notes, where pdflatex sets the notes first; `\@settranslators`' `\hbox to\columnwidth{\hss…}` line is a
full-width inline block, its text not flushed right. Repros sectioning-frontmatter/amsart_end_matter_is_set,
amsart_raw_addresses_are_set; guards `perfect_kernel_batch60::{amsart_end_matter_is_set,
amsart_raw_addresses_are_set}`.

## 444. mciteplus's starred keys are cited as keys named `*key`

mciteplus sends every citation through `\mciteCiteA` (mciteplus.sty:1020; natbib's commands :1079-1084): a key with a
leading `*` joins the previous entry's group (`\@mciteCheckKey`, :757-762), is written to the aux (:764) so bibtex
lists it, and is left out of the citation the text prints (:567-568). Perl has no mciteplus binding; the Rust stub
passed `bibrefs="a,*b"` on, so the bibliography looked for a key `*b` ("Missing bibkeys: *b") and dropped the entry:
achemso-demo 89.0 % recall (G3 slice 4 R1). Minimal trigger: `\usepackage{mciteplus}` … `\cite{a,*b}` with a `.bib`
holding `a` and `b`, `unsrt`.

Rust (60m, latexml_contrib mciteplus_sty.rs): `\@@bibref` cites every key, the `*` stripped, in order for the
bibliography (`\lx@mark@nocite` where it stands, so an unsorted style numbers them as bibtex does) and prints the
unstarred ones: `\cite{a,*b,c,*d}` … `\cite{e}` is "[1, 3]" … "[5]" over five entries, as pdflatex; an empty key adds
nothing. DIVERGENCES #427 (the groups are separate entries). Repro index-bib/mciteplus_starred_keys; guard
`06_cluster_bibliography::mciteplus_starred_keys_join_the_bibliography`.

## 445. A table's cells keep the document's `\baselineskip`

latex.ltx:16580 `\@array` sets `\lineskip\z@skip \baselineskip\z@skip` for a tabular's or array's body, after it
builds the row strut from `\strutbox` (:16567-16570; array.sty:231-232, longtable.sty:191 alike), and
`\@arrayparboxrestore` (:16272-16287) gives a p cell (`\@startpbox`, :16755), a `\parbox` and a minipage the
document's values back. colortbl's `\ifdim\baselineskip=\z@\noalign\fi{…}` (colortbl.sty:158, :163; tabu.sty:2159,
srdp-tables.sty, willowtreebook.cls) uses that to tell the space between rows or an l/c/r cell from a p cell or the
text outside. Perl's `tabularBindings`/`alignmentBindings` (latex_constructs.pool.ltxml:3622-3642,
TeX_Tables.pool.ltxml:254-259) never assign them, so the idiom's group became a cell: "\noalign cannot be used here"
and two "Extra alignment tab" (Perl 5 errors, Rust 3). Minimal trigger:

```latex
\usepackage{colortbl,xcolor}
\makeatletter\def\myarc#1{\ifdim\baselineskip=\z@\noalign\fi{\gdef\CT@arc@{\color{#1}}}}\makeatother
\begin{tabular}{|l|l|}\hline a&b\\\myarc{blue}\hline c&d\\\hline\end{tabular}
```

Rust (60l): `tabular_bindings` and the math `\@array@bindings` mark the alignment `zero_interline` after the strut is
read, and `\lx@begin@alignment` zeroes both in the alignment's own group, as `\@array` does inside its `\bgroup`
(plain `\halign` keeps them); the row strut of a LaTeX table, of amsmath's matrices and `cases` (`\array`s,
amsmath.sty:1073-1076, :1121-1126) and of the ams alignments (`\strut@`, :714-718) is `\strutbox`'s height plus depth
(tex_tables.rs `array_strut`: a table or matrix nested in a cell, where `\baselineskip` is zero, keeps its rows'
height); the vertical boxes restore the three values (`\lx@restore@interline`, the interline part of
`\@arrayparboxrestore`, which gains its `\lineskiplimit\normallineskiplimit`): a p cell (`\lx@tabular@p`, `\@startpbox`
:16755), tabularx's X, tabulary's L/C/R/J, tabu's X (tabu.sty:874-876), a `\multirow` with a width (multirow.sty:174-177),
`\parbox` and minipage. pdflatex's values in an l, p, m, b and w cell, a parbox, minipage, varwidth, tcolorbox, makecell,
nested table, `\shortstack`, array cell and between rows, the p cell's first `\the\baselineskip` included (expanded by
TeX's look for `\omit`, §788-789, before the u-template restores it); a raw `\vbox` in an l cell measures as pdflatex's.
`\\[\baselineskip]` in a table adds no space, as pdflatex (a common idiom's gap is gone in the HTML too). A vertical environment's captured body (a minipage) records the three values while its group is open
(constructor.rs, whatsit.rs: measured after the group, it took the cell's zero `\lineskip`, arXiv 2605.19065's
tcolorbox cells, and under `\lineskip=0pt` before 60l too). Residuals:
the longtable `\caption` and a `\footnote` in a cell still read zero (LaTeX's `\parbox` and `\footnotesize` reset
it); amsmath's matrices and `cases` are not zeroed (they are `\array`s in TeX; the binding builds them apart) and
IEEEtrantools' `IEEEeqnarraybox` is (IEEEtrantools.sty:2180-2182 keeps `\normalbaselineskip`). tabularray's `tblr`,
set as a `\tabular`, keeps them: its mapping marks the table `\lx@array@keeps@interline` (`array_zeroes_interline`). Repros kernel-alignment/colortbl_noalign_idiom, array_zeroes_the_interline_values,
nested_table_keeps_its_strut, array_cells_restore_their_interline_values, array_strut_in_a_cell,
tblr_keeps_its_interline_values, minipage_in_a_cell_keeps_its_lineskip; guards
`perfect_kernel_batch60::{colortbl_noalign_idiom, array_zeroes_the_interline_values, nested_table_keeps_its_strut,
array_cells_restore_their_interline_values, array_strut_in_a_cell, tblr_keeps_its_interline_values,
minipage_in_a_cell_keeps_its_lineskip}`.

## 446. Author-block continuation lines are welded to the line before

In an author block with superscript markers, a `\\`-separated line with no marker continues the previous entry (Perl
Base_Utility.pool.ltxml:701-703 `Tokens($entries[-1][1], $line)`): the split drops the `\\` between the lines, so their
words run together, "Department of PhysicsUniversity of Somewhere"; a collaboration's author list (jacow-collaboration)
became one 1,500-character `personname`. Minimal trigger:

```latex
\author{Alice Smith\textsuperscript{1}\\ \textsuperscript{1}Department of Physics\\ University of Somewhere}
```

Rust (60n, base_utilities.rs `add_authors_calls`): the split keeps each piece's delimiter (`split_tokens_delimited`) and
a continuation is appended with it where it stood, so the `\\` is an `ltx:break` (`\quad` a space). Repro
sectioning-frontmatter/author_continuation_line_keeps_its_break; guard
`06_cluster_frontmatter::frontmatter_author_continuation_line_keeps_its_break`.

## 447. A `\nocite` an `\AtEndDocument` hook runs is never cited

`\nocite` queues its mark on the end-document list (Perl latex_constructs.pool.ltxml:4214-4216), which `\end{document}`
reads once (:346); a `\nocite` that one of those hooks runs (wsemclassic.cls:302-306, :324 `\AtEndDocument{\makebib}` →
`\nocite{*}`) is pushed after the read, so its keys are never cited and the bibliography is empty ("0 cited"). LaTeX
writes the `\citation` at once. Minimal trigger: `\AtEndDocument{\nocite{*}\bibliographystyle{plain}\bibliography{b}}`.

Rust (60n, sect11.rs `\nocite`): once the list is read (`lx@enddocument@hooks@fired`), `\nocite` returns its mark where
it stands. Repro index-bib/nocite_in_atenddocument_cites; guard `06_cluster_bibliography::nocite_in_atenddocument_cites`.

## 448. biblatex's `\fullcite` prints a label, not the entry

The ar5iv binding sets `\fullcite` (and `\footfullcite`) as a bare citation (ar5iv-bindings biblatex.sty.ltxml:174
`_cite_bare`): a numeric style prints `[1]`, an entry no `\printbibliography` lists prints "Missing Entry". biblatex
typesets the entry's driver where it is cited (biblatex.def:2481-2495), whether or not the document prints a
bibliography; `\footcite`/`\footfullcite`/`\footcites` set their citation in a footnote (`\mkbibfootnote`,
authoryear.cbx:106, numeric.cbx:77), `\footcitetext` in a `\footnotetext` — Perl sets all of them inline. Minimal
trigger:

```latex
\usepackage{biblatex}\addbibresource{refs.bib}
\begin{document}See \fullcite{just22}.\end{document}
```

Rust (60p): `\fullcite` is `\@@cite{fullcite}{\@@bibref{FullEntry}{keys}{}{}}` with the notes around it (no
brackets, the postnote after `\addcomma\space`; several keys joined by ", " as pdflatex prints them). Post's show word
`FullEntry` (crossref.rs `full_entry`) copies the entry's bibblocks but "Cited by" from the copy MakeBibliography keeps
in the ObjectDB (so a citation on another split page finds it), puts back the year an author-year entry's first block
left to its label, and drops the closing `.` node (`\usedriver`'s `\finentry`, biblatex.sty:4245); the copied entry's
own citations (a crossref'd parent) are filled after it. A label style's `\footcite`/`\footcitetext`/`\footcites` set the
bare label (numeric.cbx:77, `\cite`'s brackets are its own `\mkbibbrackets`, :63) — a label style is found through the
style's `\RequireCitationStyle` chain (ieee → numeric-verb, phys/nature → numeric-comp); an author-title or notes style
(verbose, authortitle, chicago-notes, sbl, oscola) keeps the bracketed citation in its note — the full citation it prints
there is the style's rendering (ruling 7b). `\mkbibfootnote` in a note's text sets
`(…)` with biblatex's "Nested notes" warning (biblatex.sty:13227-13238; the toggle set where a note's text begins,
`\lx@note@reset`, which setspace now appends to as well). Residuals: the toggle is set in every LaTeXML note (todonotes'
`\todo`, aipproc `\source`), not only footnotes, and not in `\thanks`; an inline numeric `\cite` still drops a prenote
(`blx_cite_fallback`); MakeBibliography lists a crossref'd parent that biber's `mincrossrefs=2` leaves out, which can
shift numbers; a plain citation into the hidden bibliography links to a hidden entry; the year put back into an
author-year full citation has no disambiguation suffix ("2022", biblatex "2022a"; as the numeric body, #67);
an author-year style whose name lacks "authoryear"/"apa" (chicago-authordate, bath, ascelike, oxyear, unified, nwejm,
philosophy-classic, gb7714-*ay, …) is not recognised as author-year (`blx_set_style` reads the name, as Perl's
`_set_biblatex_style`), so its `\footcite` note holds the bracketed citation. A document that full-cites and prints no bibliography — checked after
every end-document hook, so an `\AtEndDocument{\printbibliography}` counts — gets the binding's own one at the end,
`ltx_nodisplay` (`lx@bibliography@class`; no TOC entry, no split page; S3 and `pdf_recall.py` drop hidden text;
OXIDIZED_DESIGN_DIVERGENCES #428). Witnesses quantumcubemodel (recall 93.3 → 99.0%), sidenotesplus `\sidecite*`
(89.0 → 97.2%), hidden text excluded. Repro index-bib/biblatex_fullcite_prints_the_entry; guards
`06_cluster_bibliography::{biblatex_fullcite_prints_the_entry, biblatex_fullcite_with_a_printed_bibliography,
biblatex_fullcite_keeps_the_author_year_year, biblatex_footcite_in_a_footnote_is_parenthesized,
biblatex_footcite_in_a_notes_style_keeps_its_brackets, biblatex_footcite_in_a_numeric_based_style_is_the_bare_label,
biblatex_footcite_style_chain_decides_the_label}`.

## 449. biblatex's `\printbibliography` options are dropped; `\defbibnote` is undefined

The ar5iv binding's `\printbibliography[]` (ar5iv-bindings biblatex.sty.ltxml:857) discards its options, and nothing
defines `\defbibnote` (`Error:undefined:\defbibnote`): a document's `title=` heading becomes "References" and its
`prenote=` note is lost. biblatex reads the options in a group (biblatex.sty:9688-9716, 9811-9832): `title=` replaces
the heading's default title (`\defbibheading{bibliography}[\refname]`, biblatex.def:2212; `[\bibname]` in a book,
:2239) and is kept in `\blx@thetitle` (:9694); `prenote=<name>` typesets the `\defbibnote{name}{text}` (:9372-9376)
after the heading as an unindented paragraph (`\blx@bibnote`, :10002, :10050-10058) — nothing for an empty note or the
empty name (:9702); a name with no note is biblatex's "Note '<name>' not found" error (:9703-9709). Minimal trigger:

```latex
\usepackage{biblatex}\addbibresource{refs.bib}\defbibnote{pn}{Prenote prose.}
\begin{document}Text \cite{k1}.\printbibliography[title=Primary Sources, prenote=pn]\end{document}
```

Rust (60q): `\printbibliography OptionalKeyVals:blx@bib2` (biblatex's keyset, where a style's own `\blx@kv@defkey`
keys land) opens a group; `title=` defines the kernel hook `\lx@bibliography@title` (sect11.rs), which
`begin_bibliography_clean` reads before `\bibsection`/`\refname`/`\bibname` — never `\refname` itself, so
`title={\refname}` (MIT-Thesis.tex:379) names "References" (biblatex's strings define `\refname`/`\bibname` in any
class, :5781-5789; the binding supplies the english ones in the group when the class has none). `prenote=` sets
`\lx@bibliography@preamble` to `\begingroup\noindent<note>\par\endgroup`, typeset as the bibliography's `#preamble` on
both routes (`\lx@bibliography` for a `.bib`, `\biblatex@bbl@thebibliography` for a biber `.bbl`). Residuals:
`postnote=` (typeset after the list, :10026) is read and not printed — MakeBibliography appends the list after
everything the bibliography holds; `\DeclarePrintbibliographyDefaults{title=,prenote=}` (:9780; njuthesis,
omgtudoc-asoiu) and `\printbibheading[title=,prenote=]` still ignore the keys; the strings are english only.
Witnesses biblatex-apa-test (`annotated bibliographies` …), xurl, MIT-Thesis; `title=` in 22 manuals. Guards
`06_cluster_bibliography::{biblatex_printbibliography_title_and_prenote, biblatex_bbl_printbibliography_title_and_prenote,
biblatex_printbibliography_title_names_the_default, biblatex_printbibliography_undefined_prenote_is_an_error,
biblatex_printbibliography_empty_prenote_prints_nothing}`; repro index-bib/biblatex_printbibliography_title_prenote.

## 450. Text set through OT1 in a TU or T1 document: fontspec never selects TU; a font reset returns to OT1

OT1 has no `<`, `>` or `|`: its slots print ¡, ¿ and —, and `"` is ”. Two Perl paths leave a document's text there.
(1) fontspec.sty.ltxml:24 only loads xunicode; fontspec selects the format's TU encoding (fontspec-xetex.sty:431-441
`\RequirePackage[TU]{fontenc}`), so every fontspec document's body prints `<` as ¡. (2) The font a construct resets
to — a footnote's (`\reset@font`, latex.ltx:17659), LaTeXML's tags (an `\item[…]` label, which in LaTeX inherits the
body's font) — is the TeX default with encoding OT1 (TeX_Box.pool.ltxml:218-221, Font.pm:41,275-278), where LaTeX's
`\normalfont` selects `\encodingdefault` (latex.ltx:14113-14122), so a T1 document's notes print `<b> x|y` as
`¡b¿ x—y`. Minimal triggers:

```latex
\usepackage{fontspec}\begin{document}Arrow <--- a>b, x|y.\end{document}
\usepackage[T1]{fontenc}\begin{document}Body.\footnote{Note <b> x|y.}\end{document}
```

Rust (60r): fontspec selects the format's TU, installing it first under the pdfTeX persona — tuenc.def input through
its XeTeX branch (`\XeTeXrevision` defined for that input only, restored after; its gate :49-57 would fall back to
T1) — and not through the fontenc binding, whose Perl-inherited `setupCyrillic` defines `\cyrr` & co. for any
encoding where TU leaves them undefined; TU's fontmap applies the TeX ligatures except in typewriter (tu_fontmap.rs),
as xelatex prints.
`neutralize_font` (base_utilities.rs) takes its encoding from `\encodingdefault`, expanded as `\fontencoding` expands
it (babel's `\latinencoding`, greek.ldf's `\greekfontencoding`; OT1 under plain TeX); LaTeXML's tags (`\lx@tag@intags`
— generated names, `\item[…]` labels) are Latin text, set in babel's `\latinencoding` where babel defines it
(`neutralize_font_latin`). Identifiers digested after it are read from their source tokens, as Perl's OT1 left them
ASCII: a tag's role (`OptionalUndigested`), a note's and a section's type (`type_name`), an index list, `\lxDeclare`'s
role/name/meaning/scope — inside `\selectlanguage{greek}` (LGR) they read back `ρεφνυμ`, `φοοτνοτε` (an undefined
counter), `ΑΔΔΟΠ`. fontspec selects TU after any
encoding the document chose (fontspec-xetex.sty:441). Witnesses
changelog (36 ¡/¿ → 0, its author-list `<…>` tags), latex-via-exemplos (140 → 8, the rest genuine), latex-mr (16 → 0),
hvfloat (6 → 0), cms-noteref-demo (`<---` in notes); errors unchanged. Guards `perfect_kernel_batch60::{
fontspec_text_is_tu_encoded, fontspec_selects_tu_after_a_t1_fontenc, normalfont_note_keeps_the_text_encoding,
normalfont_note_expands_the_encoding_default, identifiers_are_not_font_decoded}`; repros fonts-nfss/{fontspec_text_is_tu_encoded,
normalfont_note_keeps_the_text_encoding, identifiers_are_not_font_decoded}. Open: fontspec's `\rmdefault`/`\sfdefault`/`\ttdefault` (lmr/lmss/lmtt,
:437-439) and an explicit `Mapping=tex-text` on a mono font are not modelled.

## 451. TeX ligatures are cmr's for every encoding

Perl's text ligatures (TeX_Fonts.pool.ltxml L335-365) are cmr's — `` '' !` ?` — and only for OT1 and T1; but a
ligature is the font's own (its TFM lig/kern program). pdflatex, per encoding measured (OT1 T1 T2A T2B T2C X2 LY1 T5)
and TU's tex-text.map: `` '' → “ ” in all of them; ,, << >> → „ « » in all but OT1 (whose `<` `>` slots are ¡ ¿); !` → ¡
in OT1, T1, LY1, TU, not the Cyrillic or Vietnamese fonts (?` inferred alike). Perl leaves a T2A document's quotes ‘‘q’’
and never makes « » „. It also ligates a listing's code (set in the roman `basicstyle`), which pdflatex boxes column by
column (listings' fixed columns): LaTeXML prints `s -- t` as s – t. Minimal trigger:

```latex
\usepackage[T2A,T1]{fontenc}\begin{document}``q'' <<g>> ,,l.{\fontencoding{T2A}\selectfont ``q''}\end{document}
```

Rust (60s): the table by encoding (tex_fonts.rs `quote_ligatures`, `guillemet_ligatures`, `inverted_ligatures`), and no
ligature under `_noligatures` (a listing's lines and inline listings, listings_sty.rs). Residuals: X2 and T5 have no
fontmap (their text decodes as OT1); OT4, LGR and other encodings keep no quote ligature (unmeasured); T1's ec typewriter
fonts ligate in pdflatex while typewriter text never does here (Perl's `nonTypewriter`); listings' `columns=flexible`
would ligate; listings' `-` is a math minus − in pdflatex, a hyphen here and in Perl; a group boundary does not break a
ligature (RED fonts-nfss/group_breaks_a_ligature: `-{}-`, `<{}<`).
Witness arXiv 2605.01573 (T2A). Guards `perfect_kernel_batch60::{text_ligatures_follow_the_encoding,
listing_code_is_not_ligatured}`; repros fonts-nfss/{text_ligatures_follow_the_encoding, listing_code_is_not_ligatured,
t1_guillemet_ligatures}.

## 452. jurabib's citations print `?, .` and select no bibliography entry

Perl has no jurabib binding; the raw package's `\@citex[annotator][postnote]{keys}` (jurabib.sty:5699) typesets each
citation from `\b@<key>`, the record jurabib.bst writes to the `.aux` (`\bibcite{key}{{Author}{Short title}{…}}`), which
LaTeXML never has: every citation prints pdflatex's first pass — `\mbox{{\bfseries ?}, …}` (≈:5797), the postnote
dropped — and registers no bibref, so the bibliography selects no entry. jurabib's `\bibstyle` (:1195-1212) also
replaces the kernel's constructor, so the bibliography records no style. Minimal trigger (with a `.bib` holding
`broxbgb`):

```latex
\usepackage{jurabib}\begin{document}Text\footcite[Rn.~78]{broxbgb}.
\bibliographystyle{jurabib}\bibliography{refs}\end{document}
```

Rust (61b, jurabib_sty.rs): the raw package keeps its options, formatting macros and switches; its citation commands
(`\cite`, and the `\jb…` commands `\let` to `\footcite`/`\fullcite`/`\footfullcite`/`\citetitle`/`\footcitetitle` at
`\begin{document}`, :5949-5955; `\footcite*` = `\jbfootcitenotitle`, :3881) become `\@@cite`/`\@@bibref` with jurabib's
argument order (`\@citex`, :5719-5752: one optional = postnote; two = annotator and postnote, swapped under
`jurabiborder` when the first is not empty), the annotator after `\jbhowsepannotatorlast` (or before,
`annotatorfirst`), the postnote after `\jbprformat`, a footnote citation's `\unskip.` (:5704 — after a postnote's own
period too, "Rn. 5.."); both `\bibstyle`s run, and the jurabib styles are author-year
(`lookup_bibstyle_params`). pdflatex: "Brox Rn. 78." / "Soergel/Leptien § 167, Rn. 38." Residuals: the short title
jurabib.bst adds for an author's several works (decided by BibTeX); "A and B" where jurabib writes "A/B"; the natbib
compatibility commands; under `super`, a `\cite` inside a document's `\footnote` nests a note (jurabib's `\ifjb@fn`
comes from its `\@footnotetext` wrap); a full citation's layout (ruling 7b). Witness jbtest (recall 10.3 → 81.0 %).
Guards `06_cluster_bibliography::{jurabib_citations_are_bibrefs, jurabib_order_swaps_two_optionals_only}`; repro
index-bib/jurabib_citations_are_bibrefs.

## 453. T1 and T2A/T2B/T2C decode `^` and `~` as spacing accents

t1enc.def:141-142 and t2aenc.def:83,88 (t2b, t2c, x2 alike) put `\textasciicircum`/`\textasciitilde` in slots 94/126
— the accents `\^`/`\~` are slots 2/3; Perl t1.fontmap.ltxml:30,34 (and the T2 maps) decode 94/126 as U+02C6 ˆ / U+02DC
˜, so ASCII `^`/`~` typeset in such a document — `\string^`, Verbatim, listings, l3doc's `\cs`/meta — print modifier
letters. pdflatex prints `^ ~`. LY1 is right as it is (ly1enc.def:99,106: the accents are slots 94/126). Minimal trigger:

```latex
\usepackage[T1]{fontenc}\begin{document}\string^ \string~ \texttt{\string^}\end{document}
```

Rust (61c, DIV #433): slots 94/126 are `^`/`~` in the T1, T2A, T2B, T2C fontmaps. Witnesses precattl ("edef"), 116 s139 HTMLs
carried ˆ/˜ from ASCII sources (enverb, easylist, zref-vario, mnras sampled). Guard
`perfect_kernel_batch61::t1_ascii_slots_print_ascii`; repro fonts-nfss/t1_ascii_slots_print_ascii.

## 454. `\pdfsetmatrix` prints its matrix

pdfTeX's `\pdfsetmatrix {<matrix>}` reads its matrix as a general text (pdfTeX manual, pdftex.tex:3253-3264); Perl
pdfTeX.pool.ltxml:223 defines it as an argument-less macro, so the matrix is typeset. Minimal trigger:

```latex
\begin{document}A\pdfsave\pdfsetmatrix{1 0 .2 1}B\pdfrestore C\end{document}
```

Perl prints "A1 0 .2 1BC", pdflatex "ABC" (B slanted). Rust (61e): an unexpandable primitive reading a general text.
Witness synthslant-gauge ("1 0 .05 1" in ~105 table cells); raw users pm-isomath, bxtexlogo, unravel. Guard
`perfect_kernel_batch61::pdfsetmatrix_reads_its_matrix`; repro fonts-nfss/pdfsetmatrix_reads_its_matrix.
