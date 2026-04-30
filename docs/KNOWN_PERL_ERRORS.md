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

**Impact:** Non-fatal. Warning is noisy but harmless — tokens are preserved.

**Perl status:** Still present (Tokens.pm line 139). Unfixed.

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
