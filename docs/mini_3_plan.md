# Strategic Plan: Roadmap to Complete Faithful Rewrite

## Current State

**227 pass, 0 fail, 92 ignored (72% of non-permanent tests)**

The Rust port covers the full `latexml` (TeX→XML) pipeline: tokenization, expansion,
digestion, construction, rewriting, math parsing, and serialization. What remains is
bringing each subsystem to full Perl parity, then porting the `latexmlpost` pipeline.

---

## LaTeXML System Architecture (from the manual)

The Perl system has **two main programs**:

1. **`latexml`** — TeX→XML conversion (our current focus)
   - Digestion: Mouth→Gullet→Stomach (tokens→boxes/whatsits)
   - Construction: boxes→XML DOM (via Constructors)
   - Rewriting: DOM mutation rules (ligatures, math declarations)
   - Math Parsing: grammar-based parse of XMath token sequences
   - Serialization: DOM→XML string

2. **`latexmlpost`** — XML→target format conversion (future work)
   - Split, Scan, MakeIndex, MakeBibliography, CrossRef
   - MathML/OpenMath/MathImages conversion
   - Graphics/SVG conversion
   - XSLT stylesheets → HTML5/XHTML/HTML4
   - Writer

## What "complete and faithful" means

Per the manual's design goals:
- Faithful emulation of TeX's behavior
- Easily extensible (binding system)
- Lossless: preserve semantic AND presentation cues
- Infer mathematical semantics

For the Rust port, "complete" means:
1. Every Perl `.pool.ltxml` engine file has a corresponding Rust module at OK status
2. Every Perl `.sty.ltxml` / `.cls.ltxml` binding has a Rust equivalent
3. The `latexmlpost` pipeline is ported
4. The full Perl test suite passes (minus permanent ignores)

---

## Phase Plan

### Phase 1: Engine Parity (current — 72% tests passing)

**Goal:** All engine files at OK/MINOR. All Perl test suite tests pass that don't
require `latexmlpost` or unported packages.

**Remaining engine work (from SYNC_STATUS):**
- `tex_math.rs`: `\nonscript`, `TeXDelimiter`, `adjustMathRole()`, math ligatures
- `tex_box.rs`: SVG functions, `\hbox/vbox/vtop` TODOs, `\vrule/\hrule`
- `tex_tables.rs`: `\halign BoxSpecification`, alignment helpers
- `tex_fonts.rs`: per-font `\hyphenchar`, `getFontDimen()`, ligature defs
- `plain.rs`: `\alloc@`, `\multispan`, `\hglue`
- LaTeX ch4 (sectioning): `\format@title@*`, `\@tag`
- LaTeX ch7 (math envs): `\intertext`, afterConstruct rearrangement gaps
- LaTeX ch8 (defining): `\DeclareMathAccent`, `\DeclareFontShape/Family`
- LaTeX ch14 (pictures): picture environment (30% ported)
- `latex_constructs.pool.ltxml`: ~92% ported, missing afterConstruct, picture env

**Remaining infrastructure:**
- `rewrite.rs`: ~20% ported → need full Select/Replace/Match
- `FontDef` parameter type: simplified to FontToken, blocks per-font tracking
- `adjustMathstyle`: stub exists, needs full implementation for fraction font propagation
- Alignment cell whitespace: our glue renders as U+00A0, Perl uses U+2003

**Current test blockers by category:**
- Math parser grammar (~35 tests): function application with fences, integral patterns,
  formulae vs list, adjustMathstyle
- Missing package bindings (~12 tests): stmaryrd, mathtools, cleveref, picture, etc.
- Equation numbering (~13 tests): tag font, counter stepping, MathFork tex= attributes
- Font/sizing (~5 tests): adjustMathstyle, dimension rounding
- Crashes/timeouts (~6 tests): cells (\@nil expansion), diagbox/ncases (infinite loop),
  vmode (segfault), babel (memory leak)

### Phase 2: Package Binding Parity

**Goal:** Every Perl `.sty.ltxml` / `.cls.ltxml` has a Rust equivalent.

Per the manual: "LaTeXML benefits from having its own implementation of macros,
primitives, environments... because these define the mapping into XML." The binding
system is what makes LaTeXML useful beyond plain TeX.

**Approach:** Port bindings on-demand as tests require them. Priority order:
1. Bindings blocking multiple tests (mathtools, cleveref, stmaryrd)
2. Bindings for popular packages (physics, siunitx, tikz/pgf ecosystem)
3. Long-tail bindings (moderncv, beamer, slides, etc.)

**Key constraint:** Some packages need `expl3` (`\ExplSyntaxOn/Off`), which is a
major infrastructure piece. This blocks: beamer, fontspec, unicode-math, and many
modern packages. Porting expl3 is a Phase 2 milestone.

### Phase 3: Post-Processing Pipeline

**Goal:** Port `latexmlpost` — the XML→HTML/MathML/ePub pipeline.

Per the manual, `latexmlpost` applies filters in order:
1. **Split** — paginate into multiple documents
2. **Scan** — collect IDs, labels, cross-references
3. **MakeIndex** — fill in `\printindex`
4. **MakeBibliography** — fill in `\bibliography` from `.bib.xml`
5. **CrossRef** — resolve cross-references, generate link text
6. **Math conversion** — XMath → Presentation MathML / Content MathML / OpenMath
7. **Graphics** — convert picture environments, SVG, images
8. **XSLT** — apply stylesheets for HTML5/XHTML output
9. **Writer** — serialize to files

**Resources (copy as-is):** XSLT stylesheets, CSS, JavaScript, RelaxNG schemas.
These are declarative and used unchanged.

**Approach:** Port each Post module as a Rust module. Use `libxslt` (already a dep)
for XSLT transforms. The MathML conversion is the most complex piece — it walks the
XMath tree and generates parallel markup.

### Phase 4: Production Readiness

**Goal:** Feature-complete CLI tools, daemon mode, performance optimization.

- `latexml_oxide` CLI with full option parity
- `latexmlpost_oxide` CLI
- `latexmlc_oxide` combined CLI
- Daemon mode (`pushDaemonFrame`/`popDaemonFrame` state isolation)
- Performance: the Rust port should be significantly faster than Perl
- Documentation: user-facing manual

---

## Immediate Next Steps (ordered)

### 1. adjustMathstyle implementation
**Impact:** Unblocks fracs_test (87 diffs), choose_test (113 diffs), and font propagation
in all fraction-containing expressions.
**What:** Port Perl's `adjustMathstyle()` from `TeX_Math.pool.ltxml` — it retroactively
adjusts font sizes in `\over` fraction numerators/denominators.

### 2. Fix alignment glue character (U+00A0 → proper spacing)
**Impact:** Unblocks supertabular_test (49 diffs, all spacing), improves all tables with
custom `\tabcolsep`.
**What:** Investigate why alignment template glue produces U+00A0 (nbsp) instead of
U+2003 (em-space) or being absorbed silently.

### 3. Port stmaryrd.sty (font symbol map)
**Impact:** Unblocks stmaryrd_test (1449 diffs → likely much fewer with binding).
**What:** Port the symbol definitions from `stmaryrd.sty.ltxml` — mostly `DefMath!` calls
for mathematical symbols.

### 4. Equation numbering refinement
**Impact:** Reduces diffs in eqnums_test (316), badeqnarray (151), and all equation tests.
**What:** Fix counter stepping for `\tag`, tag font propagation, tex= attribute synthesis
on MathFork Math elements.

### 5. Math parser: function application with fences
**Impact:** Improves simplemath (139 diffs), functions (494 diffs), and many other tests.
**What:** Implement `possibleFunction` heuristic — when UNKNOWN token is followed by
parenthesized content, treat as function application. Per manual: "FUNCTION: a function
which (may) apply to following arguments... to parenthesized arguments."
