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
Perl never reads a real `.bbl` through this path. Rust's binding does, and a
biber `.bbl` reaches it routinely: biblatex's apa style requests two sorting
schemes, so the `.bbl` carries **two `\datalist` blocks** with the same
references, and each `\enddatalist` expands to a whole bare
`\thebibliography…\endthebibliography` (`biblatex_sty.rs::bib_as_thebibliography`,
mirroring Perl's `biblatex_as_thebibliography` L105-119).

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
(`latexml_oxide/tests/108_preclass_kernel_autoload.rs`).

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
numeric-only, `CrossRef::fill_in_bibrefs` collapses to the bracketed number `[N]`/`[N, M]`, matching
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
Guard: `06_cluster_regressions::cluster_svg_subfloat_survives_subcaption_2563`.

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
Rust honors brace depth (separators at depth 0 only).

Minimal trigger: `\newcommand{\myInd}[1]{\index{#1@\mbox{#1}\csuse{r@e@m}}}` + `\csdef{r@e@m}{}` + itemize item `\myInd{x}`.

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
