# Known Errors in Upstream Perl LaTeXML

This file documents issues in the original Perl LaTeXML codebase.
These are upstream behaviors or design quirks — NOT bugs introduced by the Rust port.
For Rust-specific error bookkeeping, see `latexml_package/src/engine/SYNC_STATUS.md`.

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

## 7. `NewScript` XMDual content arm uses meaningless `Apply(∅, XMRef)` for subscripted identifiers

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
