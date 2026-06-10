# Oxidized Design: Rust Port Design Decisions

This document records design decisions, architectural choices, and intentional divergences
from the Perl [LaTeXML](https://github.com/brucemiller/latexml) in the Rust port
**latexml-oxide**. It is meant for both public readers evaluating the project and
internal contributors resuming work.

---

## Table of Contents

1. [Guiding Principles](#guiding-principles)
2. [Architecture](#architecture)
3. [Intentional Divergences from Perl](#intentional-divergences-from-perl)
4. [Type System Improvements](#type-system-improvements)
5. [Tactical Insights (Known Pitfalls)](#tactical-insights)
6. [Known Upstream Perl Issues](#known-upstream-perl-issues)

---

## Guiding Principles

- **Faithfulness first.** The Rust port aims for behavioral parity with the Perl original.
  Organization, abstractions, and naming follow the Perl where possible. Divergences are
  made only when Rust's type system enables a meaningfully better representation, or when
  the Perl behavior is a known bug.

- **Meaningful types for untyped Perl.** Where Perl uses strings, arrayrefs, or blessed
  hashrefs, Rust introduces enums, structs, and newtypes that make invalid states
  unrepresentable.

- **Test parity as the north star.** The Perl test suite (`.t` files with `.tex`/`.xml`
  pairs) is the ground truth. Every passing test in the Perl suite is a target for the
  Rust port.

- **Curated Rust types and binding layer.** The current Rust types and the binding layer
  (the `DefMacro!`/`DefPrimitive!`/`DefConstructor!`/`DefEnvironment!` macro system) have
  been thoughtfully curated. Follow their patterns and levels of abstraction when adding
  new code. Traits will often need to be extended with new implementations, and sometimes
  new traits may be useful to introduce — consider that when the existing abstractions
  don't quite fit.

- **Self-contained, portable binary.** A conversion must not *read* any of
  latexml_oxide's *own* auxiliary resources from disk during its main operation.
  Everything the binary owns — engine format dumps, the compiled RelaxNG schema/model,
  XSLT stylesheets and their `xsl:import` chains, the post-processor's CSS/JS — is
  embedded at compile time (`include_bytes!` / `include_str!`) and served from memory.
  *Writing* files is expected and fine: auxiliary outputs (CSS/JS resources, split
  documents, extracted images) placed into the conversion's **destination** directory,
  and staging the binary's own embedded data through a temp file.

  **Out of scope — the TeX ecosystem.** The host TeX Live installation is *not* part of
  latexml_oxide. Reading `.sty` / `.cls` / `.tfm` and other texmf assets from the user's
  TeX tree (or from the conversion's source directory) via `kpathsea` is allowed and
  expected — those are ecosystem files the user supplies, exactly as Perl LaTeXML and
  `pdflatex` consume them. The portability guarantee is about *our* assets, not theirs.

  The litmus test: copy a release binary into an empty directory on a machine that has
  a TeX Live install but has never seen the LaTeXML source tree, run a conversion, and
  it must succeed using only the user's input file(s) plus the TeX ecosystem. This is
  what makes the distribution goal viable — official releases ship the `maxperf` profile
  binaries as GitHub Release Assets, runnable with no install step, no accompanying
  `resources/` tree, and no environment setup.

  **Status (2026-05-23): met for all owned assets, verified end-to-end.**
  - *XSLT/CSS/JS:* served from byte embeds through `libxml::io::register_input_callback`
    over the `embed:///` URL scheme (`libxml` ≥ 0.3.12); the whole `xsl:import` chain
    resolves in memory with zero `.xsl` disk reads (confirmed via `strace`).
  - *Format dumps:* embedded via `include_str!`. Confirmed by renaming the dev-tree
    `resources/dumps/` away and converting in an isolated dir — the binary logged
    `using embedded TL2025 dump — no on-disk dump found`, loaded 922 + 23903 entries
    `from <embedded TL2025>`, and produced byte-identical output. The resolver still
    *prefers* an on-disk copy when one is present (a dev/override convenience, see
    [`DUMP_DESIGN.md`](DUMP_DESIGN.md)), but the embedded copy guarantees a relocated
    binary needs no source tree.

---

## Architecture

### System-Level View

LaTeXML (Perl) has two main programs: `latexml` (TeX→XML) and `latexmlpost` (XML→HTML/MathML).
The Rust port currently covers the `latexml` pipeline. The `latexmlpost` pipeline is planned
for Phase 3 (post-processing pipeline).

The `latexml` pipeline processes input through five stages:
1. **Digestion** — Mouth (chars→tokens), Gullet (expansion), Stomach (digestion into boxes/whatsits)
2. **Construction** — boxes/whatsits→XML DOM via Constructors, with auto-open/close from Model
3. **Rewriting** — DOM mutation rules (ligatures, math token declarations)
4. **Math Parsing** — grammar-based parse of flat XMath token sequences into expression trees
5. **Serialization** — DOM→XML string output

### Workspace Structure

Six crates mirror the Perl module hierarchy:

| Crate | Perl equivalent | Role |
|-------|----------------|------|
| `latexml_core` | `LaTeXML::Core::*` | Tokenizer (Mouth), expander (Gullet), digester (Stomach), document builder, state |
| `latexml_package` | `LaTeXML::Package` + `LaTeXML::Engine::*` | Package/engine definitions, compile-time macro system |
| `latexml_oxide` | top-level `latexml` CLI | Binary targets + integration tests |
| `latexml_math_parser` | `LaTeXML::MathParser` | Marpa-style math expression parser |
| `latexml_codegen` | *(no Perl equivalent)* | Proc macros for compile-time code generation |
| `latexml_contrib` | *(no Perl equivalent)* | User-contributed / test-specific package bindings |

### State Model

Perl LaTeXML uses a global `$STATE` object. Rust uses a **thread-local, global, mutable
singleton** (decided in CHANGELOG 0.3.2). This preserves the Perl semantics — TeX's
execution model is inherently stateful and sequential — while avoiding the overhead of
threading an explicit state parameter through every function.

### String Interning

All frequently-used strings (CS names, attribute keys, font names) go through a
**string interner** (`arena` module). This gives O(1) equality comparison and reduces
memory allocation pressure compared to Perl's copy-on-read string semantics.

### Compile-Time Macro Definitions

TeX macro definitions (`DefMacro!`, `DefConstructor!`, `DefPrimitive!`) are compiled
at build time via proc macros in `latexml_codegen`. The expansion tokens, parameter
specs, and constructor templates are parsed and packed into the binary. This eliminates
the runtime parsing overhead that Perl pays on every `\usepackage`.

### Engine File Organization

Perl's single large `LaTeX.pool.ltxml` (~5400 lines) is split by Lamport chapter into
individual Rust files (e.g. `latex_ch4_sectioning_and_toc.rs`). The four plain-TeX format
files (`plain_bootstrap`, `plain_base`, `plain_constructs`, `math_common`) are merged
into a single `plain.rs`. See [`ORGANIZATION.md`](ORGANIZATION.md) for the full mapping.

### latexml_contrib Crate

The `latexml_contrib` crate handles test-specific and user-contributed package bindings.
It dispatches package names to Rust binding loaders via
`Rc<dyn Fn(&str) -> Option<Result<()>>>`. Packages that need only raw TeX loading
(no `.ltxml` bindings) use `InputDefinitions!(name, noltxml => true)` for passthrough.

---

## Intentional Divergences from Perl

These are deliberate design decisions where latexml-oxide breaks with Perl behavior.

### 1. No DTD Support — RelaxNG Only

**Decision:** DTD functionality is removed entirely. Only RelaxNG schemas are supported.

**Rationale:** DTD-based containment requires a completely different model path that
conflicts with the RelaxNG-based indirect model computation. The auto-open chain for
custom DTD elements doesn't work because `model.tagprop` only stores schema-loaded
rules, and `compute_indirect_model` cannot discover DTD elements. Fixing this properly
would require significant rearchitecting of the containment model for a rarely-used feature.

**Impact:** Namespace tests (ns1–ns5) are permanently ignored. The `DocType!` macro and
`set_doc_type()` function have been removed.

### 2. No `%\n` in TeX Attributes

**Decision:** Rust does not emit `%\n` (TeX comment-newline line-break separator) in
`tex` attributes.

**Rationale:** `%\n` is a TeX formatting artifact with no semantic content — it exists
only to break long source lines without introducing whitespace. Perl preserves it in
reversion/tex attributes, but it carries no information for downstream consumers.

**Impact:** 146 occurrences of `%&#10;` removed from 26 test XML files. When copying
test XMLs from Perl, strip `%&#10;`.

### 3. `\cdots` Role: ELIDEOP Instead of ID

**Decision:** `\cdots` uses `role="ELIDEOP"` (Perl uses `role="ID"`).

**Rationale:** This enables dedicated grammar rules in the Marpa math parser
(e.g. `term mulop tight_term elideop => infix_apply_and_elide`) for better-structured
parse trees. The ID role is too generic for ellipsis operators.

**Impact:** Test XMLs must use `role="ELIDEOP"` for `\cdots`.

### 4. Marpa-Style Math Parser

**Decision:** The math parser uses a highly ambiguous Marpa grammar instead of Perl's
hand-coded recursive descent parser.

**Rationale:** This is the primary research contribution of the Rust rewrite. The
approach is to be highly ambiguous in parsing but aggressively prune in semantics rules,
minimizing final parse count. This produces better-structured parse trees for complex
mathematical expressions.

**Impact:** Math parse trees differ structurally from Perl. This is active research;
math tests are deferred until the core engine is solid.

### 5. Color as a First-Class Type

**Decision:** Colors are represented as `enum Color { Rgb(f64,f64,f64), Cmy(f64,f64,f64), Cmyk(f64,f64,f64,f64), Hsb(f64,f64,f64), Gray(f64) }` instead of Perl's blessed arrayrefs.

**Rationale:** Rust's enum makes the color model explicit and prevents model mismatches
at compile time. The Font struct stores `Option<Color>` instead of `Option<Cow<str>>`,
eliminating string-parsing at comparison time.

**Parity:** All five Perl color models (rgb, cmy, cmyk, hsb, gray) are supported with
full inter-conversion. `to_attribute()` produces identical hex strings.

#### Font Color Comparison: Discriminant-Based Reference Equality

Perl's `Font::isDiff` uses `$x ne $y` — string comparison of *object references*. Two
Color objects at different memory addresses are "different" even if visually identical.
This means `Cmyk(0,0,0,1)` (CMYK black) is "different" from `Rgb(0,0,0)` (DEFCOLOR)
even though both render as `#000000`.

In Rust, we use two comparison functions:

| Function | Mode | Used by |
|---|---|---|
| `is_diff_font_color` | Visual: `unwrap_or(DEFCOLOR)` then `to_rgb()` fallback | `PartialEq`, `Hash`, `font_match` |
| `is_diff_font_color_ref` | Exact: `unwrap_or(DEFCOLOR)` then `cx != cy` (derived PartialEq — checks variant + values) | `distance()`, `relative_to()` |

The key insight: **different Color enum variants = different Perl object references**.

- `\color{black}` → `LookupColor("black")` → stored `Rgb(0,0,0)` = DEFCOLOR → not diff
- `\color[cmyk]{0,0,0,1}` → new `Cmyk(0,0,0,1)` ≠ `Rgb(0,0,0)` → diff (variant differs)
- `\color[gray]{0.0}` → new `Gray(0.0)` ≠ `Rgb(0,0,0)` → diff (variant differs)
- `\color{red}` → stored `Rgb(1,0,0)` ≠ `Rgb(0,0,0)` → diff (values differ)

The `color` field uses `Option<Color>` where `None` means "inherited default" (treated
as `DEFCOLOR = Rgb(0,0,0)` via `unwrap_or`). The `bg` field also uses `Option<Color>`
but `None` means "transparent" (no background), so it uses the original `is_diff_color`
which treats `None` as distinct from `Some(Black)`.

**Edge case:** `\color[rgb]{0,0,0}` creates `Rgb(0,0,0)` which equals DEFCOLOR by both
variant and value — treated as "not different", matching Perl where the stored pre-defined
`black` object is the same type. If someone defined a *new* Rgb(0,0,0) via `\definecolor`
then looked it up, Perl would see it as a new reference (diff), but our code would not.
This theoretical edge case does not appear in any test.

### 6. Font Defaults: None vs Named Strings

**Decision:** `DEFBACKGROUND = None` and `DEFLANGUAGE = None` (Perl uses `undef`).
Font `color` also defaults to `None` (not `Some(DEFCOLOR)`), meaning "inherited/unset".

**Rationale:** Perl's `undef` for these defaults is semantically "no value set", not
"white" or "en". The Rust port uses `Option<Color>` and `Option<Cow<str>>` to represent
this correctly, rather than sentinel strings. For color specifically, `None` enables the
discriminant-based comparison in section 5 — if the default were `Some(Rgb(0,0,0))`,
looking up pre-defined `black` would always match and the CMYK/Gray distinction would
be lost.

**Previous bug:** Early Rust code used `DEFBACKGROUND = "white"` and `DEFLANGUAGE = "en"`,
which caused spurious font diffs when compared against elements that had no explicit
background/language.

### 7. SVG Support Deferred

**Decision:** SVG-related code paths removed from glue, kern, and box modules.

**Rationale:** latexml-oxide targets XML/HTML output. SVG generation is not critical
for the core TeX→XML pipeline and adds significant complexity.

**Planning condition:** When we advance to translating `pgf.sty` and `tikz.sty` support,
we will add the full breadth of SVG infrastructure from Perl, including all other
SVG-producing bindings (e.g. `collapseSVGGroup`, `svg:foreignObject`, `svg:g` tags).
This is deferred, not permanently removed.

### 8. OML Font Map Position 127

**Decision:** Rust stores `'\u{0361}'` (COMBINING DOUBLE INVERTED BREVE) for OML
position 127. Perl stores a two-character string.

**Rationale:** The single combining character is the correct Unicode representation.
Perl's two-char string is a legacy artifact of its string handling.

### 9. Constructor Compiler `font` Attribute Interception

**Decision:** The constructor compiler (`constructable.rs`) special-cases `font` as an
attribute key, replacing it with a no-op `();`. Font information on elements is instead
handled through `_force_font` which triggers `finalize_rec` font computation.

**Rationale:** Font attributes in constructors need special treatment because they
represent inherited typographic state, not simple XML attributes. The `_force_font`
mechanism ensures font properties are computed correctly for empty elements (like
`XMTok`) where no text content triggers normal font specialization.

### 10. `*` in Math Uses U+2217 (ASTERISK OPERATOR)

**Decision:** The `*` character in math mode produces U+2217 (ASTERISK OPERATOR)
instead of ASCII `*` (U+002A).

**Rationale:** Matches Perl behavior. U+2217 is the semantically correct mathematical
operator character; ASCII `*` is the text asterisk.

### 11. `\lgroup`/`\rgroup` Use U+27EE/U+27EF

**Decision:** `\lgroup` and `\rgroup` produce U+27EE (MATHEMATICAL LEFT FLATTENED
PARENTHESIS) and U+27EF (MATHEMATICAL RIGHT FLATTENED PARENTHESIS) without bold font.

**Rationale:** Matches Perl commit "Lrgroup (#2762)". Previous Rust code used different
codepoints with bold font, which was incorrect.

### 12. DefEnvironmentI Default Mode

**Decision:** `DefEnvironmentI` always sets mode to `restricted_horizontal` when no
explicit mode is specified.

**Rationale:** Matches Perl `Package.pm` line 1885. Previously Rust left the mode
unset, causing environments to inherit the parent mode incorrectly.

### 13. `\accent` Full Primitive Implementation

**Decision:** `\accent Number` is fully implemented with the assignment loop from
Perl's `TeX_Character.pool.ltxml`, including dotless i/j replacement (only for
above-accents U+0300–U+0315, U+0361) and combining dot removal.

**Rationale:** The previous stub implementation didn't handle the complex TeX semantics
of accent application, especially the interactions with dotted characters and
above/below accent positioning.

### 14. Typewriter/ASCII Font Accent Hack

**Decision:** `\^` and `\~` use standalonechar U+02C6 (MODIFIER LETTER CIRCUMFLEX
ACCENT) and U+02DC (SMALL TILDE) respectively. When the font is typewriter or ASCII,
`apply_accent` uses the raw ASCII characters instead of combining characters.

**Rationale:** Matches Perl behavior. The typewriter font hack ensures that accents
in monospace contexts produce the expected ASCII-compatible output.

### 15. Improved Math Parses Over Perl

**Decision:** When the Rust Marpa grammar successfully parses an expression that Perl's
Parse::RecDescent left unparsed, the Rust output is preferred if the parse is mathematically
correct. The expected test XML is updated to match Rust's improved output.

**Rationale:** The Marpa grammar is more powerful than Parse::RecDescent and can handle
expressions that Perl gives up on. Matching Perl's *failure* modes is not a goal — matching
Perl's *success* modes is. When Rust produces a better parse, that's an improvement.

**Process:** When a test fails because Rust produces a parsed structure where Perl had flat
unparsed tokens, the developer asks the user to confirm whether the Rust XML should be updated.

---

## Type System Improvements

These are places where Rust's type system provides a better representation than Perl's
stringly-typed approach, without changing observable behavior.

### Color Type

Perl: `bless ['rgb', 0, 0, 0], 'LaTeXML::Common::Color::rgb'` — a blessed arrayref
with the model name as element 0 and components as elements 1..n.

Rust: `Color::Rgb(0.0, 0.0, 0.0)` — an enum with typed variants. Implements `Copy`,
`Eq`, `Hash`. Serialized to/from state as `"rgb 0 0 0"` strings.

### Font Color/Background Fields

Perl: `color => "rgb(0,0,0)"` — a string that must be parsed on every comparison.

Rust: `color: Option<Color>` — direct comparison without parsing. The hex string is
produced only at XML emission time via `to_attribute()`.

### RegisterType Discrimination

Perl: uses string comparison for register types.

Rust: `RegisterType` enum with a custom `PartialEq` that treats `CharDef == Number`
(since char defs are numerically-valued). **Pitfall:** use `matches!()` pattern matching,
not `==`/`!=`, when you need to distinguish CharDef from Number.

---

## Tactical Insights

Hard-won debugging insights about system internals. These prevent re-introducing
known bugs. See [`WISDOM.md`](WISDOM.md) for full details.

### DefMacro Double-Packing

`DefMacro!` with compile-time expansion (`compile_expansion!`) must set
`nopack_parameters: true`. Otherwise `pack_parameters()` runs twice — once at build time,
once at runtime — producing spurious `Error:misdefined:expansion` warnings for alignment
templates containing `#`.

### Font::merge() Must Not Call specialize()

`specialize()` is a text-classification function (examines Unicode properties of rendered
text). Calling it with font filenames like "cmb10" triggers the "Other Symbol" branch,
which resets `series="bold"` to `series="medium"`. In Perl, `merge()` has an optional
`specialize` parameter that is NOT called by default.

### Catcode::CS vs Catcode::ESCAPE

`ESCAPE` (catcode 0) is the backslash input character. `CS` is the catcode of a
fully-formed control sequence token. Use `cc.is_active_or_cs()` to test for CS/ACTIVE
tokens — never compare `cc == Catcode::ESCAPE`.

### align_group_count ($ALIGN_STATE)

This counter must be adjusted at the scan level (Gullet) only, not at the digestion
level (Stomach). `unread_one()` must retract the count for `{`/`}` tokens.
`bgroup()`/`egroup()` in the Stomach must NOT adjust it.

### Sizer String Property Lookup

Constructor sizer strings like `"#alignment"` use property lookup, not argument lookup.
The parser must distinguish `#digit` (arg) from `#word` (property), matching Perl's
`$sizer =~ /^(#\w+)*$/`.

---

## Known Upstream Perl Issues

These are behaviors in the original Perl LaTeXML that are bugs or limitations, not
intentional design. See [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) for full details.

1. **`packParameters` fires on alignment templates** — `#` followed by CS (not digit)
   in `\halign` bodies triggers a spurious "malformed arg" warning. Non-fatal.

2. **`\fontname` format** — Perl synthesizes the font descriptor from the Font object;
   it may not match what TeX engines produce.

3. **`\hyphenchar` is not truly per-font** — LaTeXML's font model is higher-level
   (family/series/shape/size) rather than per-font-instance.

4. **`specialize()` can reset explicit properties** — For "Other Symbol" characters,
   it unconditionally resets series/shape. Perl avoids the worst case because `merge()`
   doesn't call `specialize` by default.

5. **`readBalanced` can't distinguish `#` uses** — Both parameter markers and alignment
   cell placeholders use catcode 6 (PARAM). Perl processes at a higher abstraction level
   and cannot distinguish the two.

6. **`guessTableHeaders` heuristic** — Post-processing heuristic for table header
   detection can produce unexpected results on tables without intended headers.

### 16. Math Parser Design Rules

**Rule 1: Prefer grammar rules over post-parse rewrites.** Do not create rewrite rules in `semantics.rs` if the behavior can be expressed as a token rule or grammar rule in Marpa. If Perl's `MathGrammar` hints a grammar-level rule, implement it as a grammar rule.

**Rule 2: Aggressive intermediate pruning.** Ambiguous parses should be pruned early via pragmatic semantic actions. The same atoms and sub-expressions must coordinate their meanings — a given subexpression should always produce the same parse and use the same meaning within a single expression.

**Rule 3: Value-specific tokens via Marpa terminals.** When matching specific token values (like `d` for DIFFOP), prefer value-specific terminal definitions (e.g., `token!(diffd = "UNKNOWN:d")`) over runtime string checks in semantic actions. Note: the current Marpa tree builder has a limitation where one lexeme cannot match two terminals simultaneously, so value-specific terminals that overlap with role-based terminals (e.g., `diffd` overlapping `unknown`) require workarounds until the tree builder is fixed.

### 17. No Daemon Functionality

**Decision:** The Rust port does not include daemonized (latexmls) functionality.

**Rationale:** The daemon is a Perl-specific server architecture. The Rust port focuses on
the core conversion pipeline (tokenizer → expander → digester → document builder → output).
Daemon test XMLs in `LaTeXML/t/daemon/` are not tracked or synced.

**Impact:** 7 daemon format test XMLs have known differences (lang attributes, MathML
namespace declarations, Content-Type casing, logo styling) that are not being addressed.

### 18. Source-Level Bindings via `\input{name.latexml}`

**Decision:** Perl's per-document `.latexml` files are emulated by `*_src.rs` files in the test helpers, loaded via `\input{name.latexml}` in the `.tex` source.

**Perl mechanism:** When processing `foo.tex`, Perl automatically checks for `foo.latexml` in the same directory. If found, it loads and executes the Perl code, which typically contains `DefMathRewrite`, `DefMacro`, `DefConstructor` calls that customize the conversion for that specific document.

**Rust mechanism:**
1. The `.tex` file includes `\input{name.latexml}` to explicitly request the binding
2. The `input()` function recognizes `.latexml` extension and routes to `input_definitions()`
3. The test's dispatcher (in `tests/helpers/`) maps `"name.latexml"` to `name_src::load_definitions()`
4. The `*_src.rs` file in `tests/helpers/` contains the Rust equivalent of the `.latexml` definitions

**Test organization:** The `*_src.rs` files live in `latexml_oxide/tests/helpers/` and are dispatched by per-suite functions passed to `tex_tests!`. This compartmentalizes test concerns and keeps `latexml_contrib` clean for user-contributed bindings.

**Rationale:**
- Rust cannot interpret Perl at runtime, so `.latexml` files cannot be executed directly
- Using `\input{name.latexml}` preserves Perl's naming convention
- The `.latexml` extension is recognized by the `input()` function and always routes through `input_definitions()` (the binding dispatch path)
- Test-specific bindings in `tests/helpers/` keep the dispatch logic close to where it's used

**Critical insight:** Math rewrite rules (`DefMathRewrite`) in `.latexml` files execute BEFORE the Marpa grammar parses the expression. This means setting `role="ID"` or `role="FUNCTION"` via rewrites changes how the grammar interprets the tokens — it is NOT equivalent to a post-processing role change. The `*_src.rs` mechanism preserves this pre-parse semantics.

**Example:** `simplemath_src.rs` mirrors `simplemath.latexml`:
```rust
// Sets MATHPARSER_SPECULATE + rewrite rules for a,b,x,D → ID, f → FUNCTION
add_math_rewrite("a", "ID")?;
add_math_rewrite("f", "FUNCTION")?;
AssignValue!("MATHPARSER_SPECULATE" => true, Scope::Global);
```

**Impact:** Tests with `.latexml` files need corresponding `*_src.rs` files in `tests/helpers/` and `\input{name.latexml}` in their `.tex` source to get the same parsing behavior as Perl.

### 7. Angle Bracket Inner Product Parsing

**Decision:** `<x,y>` with RELOP `<` and `>` is recognized as an inner product
(fenced expression with angle bracket delimiters), producing
`delimited-<>@(list@(x, y))`.

**Rationale:** Old typesetting conventions used `<` `>` instead of `\langle` `\rangle`
for operator delimiters such as inner products. Perl's parser leaves these expressions
unparsed (`ltx_math_unparsed`). We do better by recognizing the `<term, term>` pattern
as fenced content. The `<<` and `>>` two-part relops (much-less-than, much-greater-than)
still take priority via the `two_part_relop` grammar rule.

**Grammar:** `fenced_factor += langle_rel term_list rangle_rel => fenced`, where
`term_list = term punct term | term_list punct term` handles arbitrary-length
comma-separated term chains.

**Impact:** `ambiguous_relations_test` equations `0=<x,y>` and `0=<x,y>A` now parse
correctly instead of being marked `ltx_math_unparsed`. Test XMLs updated to match.

### 8. Broad Bigop Argument Absorption

**Decision:** Bigops (`\sum`, `\int`, etc.) absorb the full `term` (mulop/invisible-times
chain), not just the next `tight_term`.

**Rationale:** `\sum_{i=0}^{\infty} f_i x^i` should produce `∑(f_i * x^i)`, not
`∑(f_i) * x^i`. The summation variable `i` appears in both `f_i` and `x^i`, so the
entire product is the summand. Perl's `addOpArgs` (Parse::RecDescent) non-deterministically
selects narrow absorption for some expressions (documented in KNOWN_PERL_ERRORS #9).

**Grammar:** `bigop_application = bigop/scripted_bigop/composed_bigop term`, lifted to
`expression` level so bigops can't be followed by invisible-times on the right.

**Impact:** `declare_test` sum equations updated. `calculus_test` improved (331→273 diffs).

### 9. Document-Order xml:id Renumbering

**Decision:** After math parsing completes, xml:ids inside each XMath subtree are
renumbered to be sequential in document order (pre-order DFS). Perl's
Parse::RecDescent generates IDs in bottom-up parse order (tokens first, then
higher-level constructs).

**Rationale:** The Marpa grammar parser explores multiple parse alternatives
simultaneously, consuming ID counter slots for pruned nodes. This produced
non-sequential IDs like `m1.1, m1.7, m1.12` instead of `m1.1, m1.2, m1.3`.
Document-order assignment is predictable and deterministic regardless of
parser internals. It uses a pure post-processing pass in `core_interface.rs`
after all parsing and kludge processing, before `document.finalize()`.

**Implementation:** `renumber_math_ids()` performs a single DFS walk per XMath
subtree, collecting both xml:id and idref nodes. Parent prefixes are derived
via O(1) string parsing (rfind('.')) instead of DOM ancestor walks. IDs are
stripped in a batch pass before reassignment to avoid idstore collisions.

**Impact:** Test XMLs for mathaccents, esint, mathbbol, not, choose, declare,
sampler, amsarticle, latextheorem, amstheorem, genfracs, amsdisplay, sets,
multirelations, standalone_modifiers, sequences_and_lists, and compose were
updated to reflect document-order IDs. All structural content is identical
to Perl; only ID values differ.

### 10. Grammar: Two-level sequence semantics (formulae vs list)

**Decision:** The Marpa grammar distinguishes two levels of comma/punct-separated
sequences, matching Perl's `Formulae`/`extendFormula` distinction:

- **`formulae`** (formula level): Punct-separated COMPLETE relational formulas.
  `a=b, c=d` → `formulae@(a=b, c=d)`. Produced by `formula_list` rule via
  `formulae_apply` semantic action.

- **`list`** (expression level): Punct-separated expressions within a formula.
  `a, b, c` → `list@(a, b, c)`. Also used for RHS extension: `a=b, c` →
  `a = list(b, c)`. Produced by `statements` rule via `list_apply`.

**Disambiguation rules** (semantic pruning, since Marpa explores both paths):

1. `formulae_apply` rejects when NO items are relational → forces `list_apply`.
2. `list_apply` rejects when BOTH items are relational → forces `formulae_apply`.
3. `list_apply` rejects when either item is relational and left is not already a
   list/formulae Dual → forces `formulae_apply`.
4. `infix_relation` (multirelation extension) rejects when the left formula's
   last operand is a `list` Dual → prevents `a = list(b,c) = d`, forcing the
   comma to be a formula boundary instead.
5. Both `list_apply` and `formulae_apply` reject items with `absent` relop
   operands (equation fragments) — see rule 11.

**Rationale:** Perl's Parse::RecDescent resolves this structurally through rule
ordering (extendFormula consumes commas before moreFormulae can see them). Marpa
explores all alternatives simultaneously, so semantic pruning is needed. The
rules above create a clean partition: relational items go through formulae,
non-relational through list, with multirelation rejection preventing the
"comma inside formula RHS" misparse.

### 11. Grammar: Absent operands are formula-level only

**Decision:** The `absent` token (meaning="absent") represents a missing/implied
operand, typically from alignment cell boundaries in multi-line equations:

```latex
a(x) &= f(x) + g(x) + h(x) \\
     &= f(x) + \phantom{g(x)} + h(x)
```

The second row `= f(x) + \phantom{g(x)} + h(x)` has an absent LHS (the `a(x)`
from the row above). This is a single formula fragment: `absent = f(x) + ... + h(x)`.

**Rules:**
- `absent` as a relop operand is valid in a single **formula** (equation fragment).
- `absent` is NOT valid inside a **list** — `list_apply` rejects.
- `absent` is NOT valid inside a **formulae** collection — `formulae_apply` rejects.
- At the top level, a formula with `absent` is a standalone fragment, not part of
  a multi-formula collection.

**Open question:** `\phantom` creates intentional gap space that may need a
dedicated grammar rule. Currently, `\phantom{g(x)}` produces a box with
invisible content. When alignment cell boundaries split an expression containing
`\phantom`, the fragments become unparseable. The proper fix requires alignment
infrastructure to join cells before math parsing, or a dedicated phantom rule
that preserves expression continuity across cell boundaries.

### 12. Grammar: bigop_application at term level

**Decision:** `bigop_application` (e.g. `\neg b`, `\sum x dx`) is placed at the
`term` level in the grammar (`term += bigop_application`), not at the `expression`
level. This prevents exponential Marpa ambiguity when ADDOP precedes BIGOP
(e.g. `a + \neg b`).

**Rationale:** At expression level, `expression += bigop_application` combined
with `expression = term addop expression` created multiple derivation paths for
the same semantic result (e.g. `π + ¬a`). The Marpa Earley recognizer explored
all paths, causing exponential tree enumeration. At term level, the addop rule
handles the combination with a single derivation.

### 13. Grammar: Period and comma precedence in formulae

**Decision:** Period (`.`) and comma (`,`) are both formula/list separators at
the same grammar level (`statements`/`formula_list`). Comma after a relational
formula's RHS groups as a list (`a=b,c` → `a=list(b,c)`), while period always
creates a hard formula boundary (`a=b.c` → `formulae(a=b, c)`).

For `a=b.c,d=e`, the Rust parse is `formulae(a=b, c, d=e)` — three separate items.
Perl produces `formulae(a=b, list(c,d)=e)` — grouping `c,d` across the period as
a list LHS. The Rust parse is accepted as a valid alternative.

**Rationale — the long tail of rare mathematical notation:**

Mathematical notation is a natural language with centuries of accumulated conventions.
While common patterns (like `a=b,c=d` for parallel equations or `a=b,c` for a set-like
RHS list) appear frequently and have clear semantic intent, the interaction between
MULTIPLE separators in a single expression creates a combinatorial explosion of
edge cases that are vanishingly rare in practice.

Expressions like `a=b.c,d=e` (mixing period and comma with multiple relations)
essentially never appear in real mathematical writing. When they do, the intended
semantics are ambiguous even to human readers without surrounding context. Attempting
to match Perl's interpretation for every long-tail combination:
- Adds grammar complexity that risks regressions on common patterns
- Encodes arbitrary choices that may not reflect any real author's intent
- Cannot be validated against actual mathematical usage

The Rust port prioritizes:
1. **Correct handling of common patterns** (>99% of real math)
2. **Defensible alternatives** for rare patterns (valid parse, just different grouping)
3. **Grammar simplicity** to avoid Marpa ambiguity explosion

When the Rust parse differs from Perl on a rare notation, both parses are typically
valid mathematical interpretations. We accept the Rust parse as a documented
intentional divergence rather than adding complexity to match Perl exactly.

### 14. Grammar: Generic open/close fenced delimiters

**Decision:** Added `open expression close => fenced` rule for generic OPEN/CLOSE
delimiter pairs (e.g. `\lfloor...\rfloor`, `\lceil...\rceil`, `\Lbag...\Rbag`).
Previously, only specific delimiter pairs (parens, brackets, braces, vertbar)
had fenced rules. Added floor/ceiling/norm semantic meanings for known delimiter
pairs.

### 15. Grammar: Evaluated-at and norm patterns

**Decision:** Added `evaluated-at` pattern (`a|_∞` → `evaluated-at@(a, ∞)`)
and `norm` pattern (`||a||` → `norm@(a)` with ‖). These match Perl's
MathGrammar `evalAtOp`/`maybeEvalAt` and `SINGLEVERTBAR SINGLEVERTBAR`
rules respectively.

### 16. Grammar: Bigop argument scope after invisible times

**Decision:** Removed `any_bigop` from `scripted_factor_r11`/`scripted_factor_r12`
rules. Bigops now ONLY get scripts via `scripted_bigop`, ensuring
`bigop_application` always fires and absorbs the following term.

Before this change, `1/2∫_0^1 f dx` parsed as `(1/2)*(∫_0)^1*f*dx` because
the integral was treated as a scripted factor, preventing argument absorption.
After: `(1/2)*((∫_0)^1)@(f*dx)`.

**Note:** Explicit mulop (`\times`) between bigop and its argument still breaks
absorption: `∫ F×G dx` → `integral(F)*G*dx`. Both `∫(F)` and `∫(F×G×dx)` are
valid Marpa parses; tree selection currently prefers the shorter absorption.
This is a known limitation affecting rare explicit-mulop-in-integrand patterns.

### 17. Script content preservation (C5)

**Decision:** `faux_wrap` now returns `XM::Wrap([start_script_lexeme, parsed_content])`
instead of just the lexeme. `new_script_inner` detects this and uses the parsed
content directly, avoiding re-reading from DOM via `obtain_arg`.

This fixes empty XMRef for any parsed expression inside scripts:
- `f^{(n)}` → `f ^ n` (was `f ^ []` — fenced XMDual discarded)
- `q_{a,b}` → `q _ list(a,b)` (was `q _ list([], [])`)

The root cause was that `obtain_arg` re-read the original DOM, which still had
the raw tokens `(`, `n`, `)` — not the parsed `fenced@(n)` XMDual.

### 18. Speculative function application produces Apply, not invisible times

**Decision:** For any UNKNOWN token `f` followed by a fenced expression `(x)`,
Rust produces `f@(x)` (function application) rather than Perl's default
`f * x` (invisible-times multiplication). This is the *always-on* default,
not gated on any flag.

**Rationale.** Parse::RecDescent (Perl) can only commit to one parse. Its
`MaybeFunctions` mechanism was a workaround: mark the UNKNOWN token with
`possibleFunction="yes"` and then fail the production, yielding invisible-times
with an advisory attribute. Marpa (Rust) is an ambiguous CFG engine — the
grammar produces *both* interpretations in the forest, and the pragmatic layer
picks one. `FencedLettersAreFunctionArguments` is the authoritative selector:
when mathematical practice reads `f(x)` as function application (which it
always does for a letter `f` and any non-NUMBER content in the parens), that
is the tree we keep.

**Role of `MATHPARSER_SPECULATE`.** The flag no longer influences parse
structure. Its only remaining effect is to enable the `possibleFunction="yes"`
diagnostic attribute on UNKNOWN tokens that participate in such speculation.
`\usepackage[mathparserspeculate]{latexml}` is kept for backwards compatibility
but does not change which tree wins.

**Author override.** Authors who want `f(x) = f * x` can declare `f` as ID:
`\lxDeclare[role=ID]{f}`. With the ID role, the speculative grammar rule
`unknown fenced_factor` does not apply (it's gated on role UNKNOWN), so only
the invisible-times parse is produced.

**Affected tests:** 13 test XMLs updated session 107 (previously recorded
Perl's SPECULATE-off behavior; now record mathematically-consistent parses).

---

### 19. Perl `local` Mechanism — `latexml_core::common::local_assignments`

Perl's `local` keyword provides dynamic scoping: a variable is temporarily overridden
within a block and automatically restored when the block exits. LaTeXML uses `local`
extensively for context-dependent state (e.g., `local $LaTeXML::SPACE`, `local @LaTeXML::LIST`,
`local $LaTeXML::CURRENT_TOKEN`).

**Rust implementation:** `latexml_core::common::local_assignments` provides a thread-local
stack-based mechanism for global state that needs dynamic scoping. Each "localized" field
uses a `Vec<T>` as a stack: `push` to shadow, `pop` to restore.

**When to use `local_assignments`:**
- For GLOBAL state that Perl declares with `local $LaTeXML::VARIABLE`
- When the variable is accessed across multiple function calls (not just one recursion chain)
- Examples: `$LaTeXML::CURRENT_TOKEN`, `@LaTeXML::LIST`, `$LaTeXML::ALIGN_STATE`

**When to use struct field save/restore instead:**
- For state passed through a single recursion chain (e.g., `LstContext.space_token`)
- When the value is part of a mutable struct passed by reference
- The save-on-entry / restore-on-exit pattern is equivalent to Perl's `local` in this case:
  ```rust
  let saved = ctx.field;
  ctx.field = new_value;
  recursive_call(ctx);
  ctx.field = saved;
  ```

**Adding a new localized field:**
1. Add the field to `Localized` struct in `local_assignments.rs`
2. Add `set_*` / `get_*` / `expire_*` functions following existing patterns
3. Call `set_*` at scope entry, `expire_*` at scope exit
4. Ideally, use RAII guards (Drop trait) for automatic cleanup — TODO improvement

### 20. Color Comparison: Visual Equivalence

**Decision:** In latexml-oxide, two `Color` values are compared by variant and values
(structural equality), not by object identity. `Color::Rgb(0.0, 0.0, 0.0)` equals
`Color::Rgb(0.0, 0.0, 0.0)` regardless of how or when they were created. Colors from
different models (e.g., `Gray(0)` vs `Rgb(0,0,0)`) ARE considered different even when
visually equivalent — the comparison is by variant + values, not by conversion to a
common model.

**Perl behavior:** `Font.pm`'s `isDiff` uses Perl's `ne` operator on unoverloaded
`Color` objects, which compares memory addresses (reference equality). Two Color objects
with identical values (e.g., both `Color::rgb(0,0,0)`) are considered "different" if
they are different Perl objects. This produces incidental `color="#000000"` attributes on
elements when the author explicitly sets `\color{black}` in a scope that already has
black as the default color.

**Observable differences:**

- `\color{black}` in a black context produces NO `color="#000000"` attribute (Perl may
  produce one due to reference inequality)
- `\color[gray]{0}` vs default `Rgb(0,0,0)` DOES produce a `color` attribute because
  `Gray(0) != Rgb(0,0,0)` (different Color variants)
- SVG elements like `svg:g` do not get redundant `color="#000000" fill="#000000"
  stroke="#000000"` attributes when the parent already establishes black

**Implementation:** Two comparison functions in `font.rs`:

| Function | Mode | Used by |
|---|---|---|
| `is_diff_font_color` | Visual: `unwrap_or(DEFCOLOR)` then `to_rgb()` fallback | `PartialEq`, `Hash`, `font_match` |
| `is_diff_font_color_ref` | Variant+values (no `to_rgb` fallback) | `distance()`, `relative_to()` |

Both treat `None` (inherited default) as equivalent to `DEFCOLOR = Rgb(0,0,0)` via
`unwrap_or(DEFCOLOR)`.

**Rationale:** Perl's reference-inequality semantics are an accident of its object
model, not an intentional design. When a user writes `\color{black}` in a context that
is already black, the redundant `color="#000000"` attribute carries no information. The
Rust port's structural equality produces cleaner output without changing any visible
rendering. Cross-model comparison (`Gray(0)` vs `Rgb(0,0,0)`) still detects the
difference because the Color enum variant differs, preserving the ability to distinguish
colors specified via different models — see also section 5 ("Font Color Comparison:
Discriminant-Based Reference Equality").

**Impact:** Tikz SVG tests show fewer `color`/`fill`/`stroke` attributes than Perl
output. This is the primary source of remaining diffs in `tikz_3d_cone` and
`ac_drive_components` tests.

### 21. No `tex=` Attribute on `<picture>` Elements

**Decision:** The `tex=` attribute on `<ltx:picture>` elements is suppressed by default.
It is only emitted when the environment variable `LATEXML_SVG_TEX_ATTRIBUTE=true` is set.

**Perl behavior:** Perl emits a `tex=` attribute on `<picture>` containing the full TeX
source of the tikz/pgf picture environment. This can be extremely long (thousands of
characters of raw pgf commands) and is not used by downstream consumers.

**Rationale:** The `tex=` attribute on pictures is a debugging artifact. It inflates the
XML output size significantly (often 10x the rest of the element) with raw pgf
instructions that are illegible and serve no rendering or accessibility purpose. Making
it opt-in via an environment variable keeps it available for debugging while producing
cleaner default output.

**Impact:** All tikz/pgf test reference XMLs omit the `tex=` attribute on `<picture>`
elements. When copying test XMLs from Perl, strip `tex="..."` from `<picture>` tags.

### 22. No Empty Nested Language-Return Wrappers on Group Exit

**Decision:** When exiting a group that changed `xml:lang` (e.g.,
`\foreignlanguage{english}{…}` nested inside `\begin{otherlanguage}{french}`),
Rust emits at most one empty `<text xml:lang="…">` wrapper per closing group,
not a nested chain mirroring each enclosing language scope.

**Perl behavior:** Perl's document builder unwinds each enclosing font/language
frame as a separate empty `<text>` element. For a document with class option
`[german]{article}` + `\usepackage[french,english]{babel}` + the page545 test's
nested `\foreignlanguage{english}{…}\end{otherlanguage}`, Perl emits
`<text xml:lang="fr"><text xml:lang="de"></text></text></p>` at end of the
English foreignlanguage paragraph.

**Rust behavior:** Rust's document builder emits only
`<text xml:lang="fr"></text></p>` — the outer wrap from returning to French,
but not a further nested wrap for the default-document German. This reflects
a single-level language-change tracking model vs. Perl's per-frame stack
unwind.

**Rationale:** Both empty wrappers contain zero content and are invisible in
rendering. The nested wrap is a Perl-specific structural artifact with no
semantic or visual impact. Matching it would require deeper font-stack
unwinding logic at group close that has no downstream benefit.

**Impact:** The `tests/babel/page545.xml` expected XML has been updated to
the Rust form (single empty wrap). Any future test XMLs copied from Perl
with this pattern should be similarly normalized.

### 23. `_loaded` Flag Naming — Drop `ltxml_loaded`, Add `_raw_loaded`

**Decision:** Rust uses a unified `<name>_loaded` flag for *bindings* (Rust
modules under `latexml_package/src/package/`) and a separate `<name>_raw_loaded`
flag for raw .sty/.cls/.def TeX files. The Perl `<name>.ltxml_loaded` form
is dropped.

**Perl behavior** (Package.pm L2311-2316, L2346-2347):
- `loadLTXML` (binding load): sets BOTH `$request_loaded` AND
  `$ltxname_loaded` where `$ltxname = $name . '.ltxml'`
  (e.g. `babel.sty.ltxml_loaded`).
- `loadTeXDefinitions` (raw .sty/.cls load): sets only `$request_loaded`
  (e.g. `babel.sty_loaded`).
- The `.ltxml`-suffixed key was a Perl-specific marker indicating "binding
  loaded", checked by `\@ifpackageloaded` and `\RequirePackage` guards.

**Rust translation:**
- Binding load (Rust module dispatch, e.g. `babel_sty.rs`) → sets
  `<filename>_loaded` (e.g. `babel.sty_loaded`). This is the ONLY flag
  set on binding load.
- Raw `.sty`/`.cls`/`.def` load (the underlying TeX file, possibly
  triggered from inside a binding via `\input`) → sets
  `<filename>_raw_loaded` (e.g. `babel.sty_raw_loaded`). This is the
  ONLY flag set on raw load.
- A binding `.rs` can load a raw `.sty` of the same name without the
  flags clobbering each other:
  - `babel_sty.rs` runs → `babel.sty_loaded = 1`
  - inside, `InputDefinitions("babel", noltxml=true)` → `babel.sty_raw_loaded = 1`
- Reads check the appropriate flag(s):
  - "Was the binding loaded?" → `<filename>_loaded`
  - "Was the raw file loaded?" → `<filename>_raw_loaded`
  - "Either?" → check both

**Rationale:** Perl's two-key scheme leaks the `.ltxml` filesystem suffix
into the API. In Rust, bindings are compile-time modules with no `.ltxml`
filename, so the Perl convention is meaningless and confusing. The
`_loaded` rename simplifies the Rust API. The `_raw_loaded` key preserves
the binding-vs-raw distinction needed for correctness (e.g., when a binding
replaces a raw file, we should not double-load the raw file when something
later `\input <name>.sty`s).

**Migration:** Sites that check `<name>.ltxml_loaded` migrate to
`<name>_loaded`. Sites that check whether the *raw* file was loaded use
`<name>_raw_loaded`.

### 24. Graphics Content-Hash Deduplication

**Decision:** The graphics post-processor (`latexml_post::Graphics`)
deduplicates conversion and copy work by the SipHash of the source
file's bytes (paired with the graphicx `options=` string), not by
source path. Byte-identical sources with the same options produce a
single conversion job and a single output file in the bundle; every
`<ltx:graphics>` node that resolved to that content references the
shared dest.

**Perl behavior:** `LaTeXML::Post::Graphics::process` walks
`<ltx:graphics>` nodes serially and calls
`processGraphic`/`generate_resource` per node. Two nodes that resolve
to byte-identical files at different paths (or the same path multiple
times) trigger two `Image::Magick` reads and two `Write` calls,
producing two output files in the bundle (`foo-1.png`, `foo-2.png` or
similar).

**Rust behavior:** Source bytes are hashed once
(`std::hash::DefaultHasher` / SipHash, 64-bit). The key
`(content_hash, options)` indexes a `HashMap<JobKey, usize>` for the
parallel-conversion path and a `HashMap<CopyKey, String>` for the
raster-copy path. On hit, the existing dest is reused and the node's
`imagesrc` points at the first-seen filename. The `options` part of
the key is essential: graphicx `angle=` is applied via an in-place
post-conversion `convert -rotate`, so different rotations of the same
content need separate output files.

**Rationale:** Author-list and badge papers re-include the same icon
hundreds of times. Witness arXiv:2402.01336 (LHCb 1067-author paper)
includes `figs/orcidIcon.pdf` 1067 times via `\lhcborcid`. Without
dedup that's 1067 PDF→PNG conversions and 1067 entries in the bundle;
with dedup it's 1 conversion and 17 total output files for the 1083
`<ltx:graphics>` nodes. The per-node walk is preserved, only the
expensive subprocess + file-write side-effects are coalesced.

**Impact:** Output bundles for graphics-heavy papers shrink
proportionally to their duplicate rate. The graphics phase wall time
drops by the same ratio because subprocess fork-exec is the dominant
cost (see `docs/PERFORMANCE.md` §5). HTML output still has the
correct number of `<img>` tags — only the underlying file count is
deduplicated.

### 25. Direct Ghostscript EPS Path

**Decision:** EPS and PS sources are rasterized by calling `gs`
directly with the same flags ImageMagick's delegate uses, bypassing
the `convert` wrapper. `convert` remains the fallback.

**Perl behavior:** `LaTeXML::Util::Image::image_graphicx_complex`
calls `Image::Magick::Read` / `Write` for every conversion, which
shells out to `gs` for PostScript inputs.

**Rust behavior:** `convert_eps_via_gs` runs `gs -q -dNOPAUSE -dBATCH
-dSAFER -dTextAlphaBits=4 -dGraphicsAlphaBits=4 -dMaxBitmap=500000000
-dAlignToPixels=0 -dGridFitTT=2 -dEPSCrop -sDEVICE=pngalpha
-r{density} -sOutputFile={tmp} {source}` and atomically renames the
result into place. The antialiasing and bitmap flags mirror IM's
`delegate.xml` `ps:alpha` entry, so output quality matches `convert`.
On failure, falls through to `convert`/`gs` via the existing path.

**Rationale:** `convert` shells out to `gs` anyway — invoking `gs`
ourselves saves the IM read-pipeline overhead (50–200 ms per
image). gs uses CCW Rotate, the same convention as graphicx and IM,
so this does not reintroduce the rotation regression we saw with the
disabled `ps2pdf -dEPSCrop` path (which produced a PDF with a `/Rotate`
metadata entry that's CW in PDF spec).

**Impact:** EPS-heavy papers see ~1.7-1.8× faster graphics phase
on the EPS bands. Measured on `lhcb-logo.eps`: 72 ms (gs-direct)
vs 127 ms (`convert`).

**Status:** Decision made 2026-04-26 during babel.sty timeout investigation.
Implementation completed 2026-04-26 (commits `1eb66c75c`, `de21ae928`,
`01df250c6`). See `docs/archive/BABEL_TIMEOUT_BISECT.md` for the triggering
investigation.

#### Path-aware gating (commit `de21ae928`)

CRITICAL invariant: a binding `<file>.rs` MUST be allowed to call
`InputDefinitions(noltxml=>1)` for its same-named raw `.sty/.cls/.def`
AFTER its own `_loaded` flag was already set. Examples:
- `babel_sty.rs` → raw `babel.sty`
- `cite_sty.rs` → raw `cite.sty`

`input_definitions` therefore gates by the load path actually being
taken (helper `already_handled` in `binding/content.rs:226`):
- `noltxml=true` (raw-only path) → check ONLY `_raw_loaded`
- `notex=true` (binding-only path) → check ONLY `_loaded`
- otherwise (default: binding-then-raw) → check EITHER

The step-4 raw-search gate (L437) drops the `_loaded` check entirely:
when the search reaches step 4, the calling context has already
decided to load raw (binding either failed or was suppressed via
`noltxml`). Only the raw flag should block.

`_load_binding` keeps a binding-only `_loaded` gate (mirrors Perl
`loadLTXML` Package.pm L2311 which checks only the binding flag).

#### Reader semantics (commit `01df250c6`)

User-level "is X loaded?" queries consult EITHER flag — they don't
care which path produced the load. This applies to:
- `\@ifpackageloaded` / `\@ifclassloaded`
  (`latex_constructs.rs:3598`)
- `soul_sty.rs` color-presence checks (3 sites)
- `cleveref_sty.rs` amsmath-fake-loaded probe

#### Rationalization: drop `_found_loaded`

The Rust port also accumulated a Rust-only `<filename>_found_loaded` flag
that has no Perl equivalent. It's set at:
- `binding/content.rs:334` — alongside `_loaded` on binding load
- `binding/content.rs:441` — on raw-file load
- Read at `binding/content.rs:565`, `:1247`, `:1368`, `:1510`

The original intent was "binding actually fired AND loaded successfully"
(distinct from "_loaded" which could be set even on early-skip paths).
This distinction is not present in Perl and produces a third flag that
shadows the same lifecycle.

**Action**: Audit every `_found_loaded` site and either:
- Replace with `_loaded` (in cases where it represents post-load state).
- Replace with `_raw_loaded` (cases tracking raw .sty/.cls load).
- Delete entirely (cases that duplicate `_loaded`).

After the rename, the Rust set of `_loaded`-family flags should be
EXACTLY: `<name>_loaded`, `<name>_raw_loaded`, `<name>_loaded_with_options`
(matches Perl's `_loaded_with_options` at L2569/L2612).

#### Important: Perl error semantics

Perl's `loadLTXML` (L2296) and `loadTeXDefinitions` (L2332) BOTH set
`_loaded` BEFORE attempting to read the file (L2315 & L2347). On read
error, `_loaded` is already set, so subsequent calls early-skip.

Rust's `binding/content.rs:317` mimics this for binding load. But Rust's
`_found_loaded` was added because the existing `_loaded` flag is set
even on error paths in some routes — so callers needed a way to ask
"did the load *actually succeed*?".

Perl does NOT have this distinction. Perl's caller of `loadLTXML` /
`loadTeXDefinitions` checks the return value (truthy = success).

**Migration plan (to be implemented carefully)**:
1. Keep `_loaded` semantics exactly Perl-faithful: set BEFORE read
   attempt, persist on error.
2. The "did it succeed" question is answered by the `Result` return,
   not a flag.
3. The 6 sites that read `<name>_found_loaded` are checking "did it
   actually load (not just attempt)". Audit each:
   - If they truly need success-not-error semantics, add an explicit
     return/result check at the call site rather than a flag.
   - If they only need "loaded at all" semantics, switch to `_loaded`.
4. Drop the `_found_loaded` flag in `dump_writer.rs` (it shouldn't
   be in dumps) and `dump_reader.rs` (its skip-list).

This must be done WITH CARE — the error behavior at `binding/content.rs:317`
could be load-bearing for Rust-specific recursion guards. Implementer
must run the full test suite and sandbox after each change.

#### Perl's dump-format equivalent of SKIP_VALUE_CONTAINS

Question: does Perl have an equivalent to Rust's
`dump_reader::SKIP_VALUE_CONTAINS = ["_loaded"]`?

**Answer**: NO. Perl's `latex_dump.pool.ltxml` dump emits all
`_loaded` flags verbatim, e.g.:
```
V('antomega.cfg_loaded',1);
V('dumyhyph.tex_loaded',1);
V('expl3-code.tex_loaded',1);
V('expl3.ltx.ltxml_loaded',1);
V('expl3.ltx_loaded',1);
```
Perl carries BOTH `expl3.ltx_loaded` (raw) AND
`expl3.ltx.ltxml_loaded` (binding) into the post-dump state.

Why Rust needed the skip-list: the runtime engine treats the
dump-loaded `<file>_loaded` flag as "raw was loaded", which makes
subsequent `\input <file>` short-circuit and skips re-execution
that the engine actually depends on (e.g., babel's hyphenation
language registers).

**Rationalization opportunity** with #23's binding/raw split:
- Perl `<name>_loaded` (raw) → Rust `<name>_raw_loaded`
- Perl `<name>.ltxml_loaded` (binding) → Rust `<name>_loaded`

If `dump_writer` faithfully maps Perl's two-key scheme into Rust's
two-key scheme, the dump's `_raw_loaded` entries correctly mark
"already raw-loaded" state. The skip-list is then no longer a
workaround but reflects intentional state. The underlying issue
(raw-load short-circuiting) is solved by `LoadFormat`-style
mutual exclusivity (dump-cache vs raw-load): one path is active
at a time, never both. See SYNC_STATUS D0 "dump/_base
mutual-exclusivity".

After mutual-exclusivity lands, `SKIP_VALUE_CONTAINS` should
become empty/removable.

### 26. `mdframed` Uses `inline-logical-block`, Not `inline-block`

**Decision:** `\begin{mdframed}…\end{mdframed}` wraps body in
`<ltx:inline-logical-block>` (Misc.class container that accepts
Para.model body — theorem / proof / para), not `<ltx:inline-block>`
(Misc.class but accepts Block.model only — rejects theorem).

**Perl behavior:** `ar5iv-bindings/mdframed.sty.ltxml` uses
`<ltx:inline-block framed="rectangle" …>`. A paper that wraps a
theorem environment in mdframed (a common pattern for highlighting
key results) hits a schema-rejection cascade:
`"ltx:theorem" isn't allowed in <ltx:inline-block>`.

**Rust behavior:** `latexml_contrib/src/mdframed_sty.rs` emits
`<ltx:inline-logical-block framed='rectangle' …>`. Choosing
`inline-logical-block` over the also-valid `logical-block` is
deliberate:

* `inline-logical-block` ∈ Misc.class (same membership as Perl's
  `inline-block`) — accepted in every parent context where Perl's
  choice fits, including inline contexts.
* `logical-block` ∈ Para.class — REJECTED in inline contexts; would
  break papers using `\fbox{\begin{mdframed}…}` or similar inline
  wrappers.
* Both candidates expose the same `Backgroundable.attributes`
  surface (`framed`, `framecolor`, `backgroundcolor`).
* `LaTeXML.css` sets `.ltx_inline-logical-block { display:
  inline-block }` — identical CSS to `.ltx_inline-block`, so the
  visual output is unchanged.

**Witness:** arXiv:2506.03074v1 (ICML 2025 — multiple
`\begin{mdframed}\begin{theorem}…\end{theorem}\end{mdframed}`
blocks). 3 errors → 0. Tests 1328/0/0.

### 27. `\DeclareMathSymbol` U-encoding Fallback: U+FFFD, not Empty

**Decision:** When `\DeclareMathSymbol{cs}{type}{fontkind}{slot}` resolves
the symbol-font's encoding to a value whose `LoadFontMap()` returns
`None` (the most common case is `U` — "Unknown" encoding declared via
`\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}`), we substitute U+FFFD
(REPLACEMENT CHARACTER) for any slot in the C0 control range (0x00-0x1F
minus tab/LF/CR) and the raw codepoint otherwise. Perl's
`Package.pm::FontDecode` returns `undef` glyph for the same case;
Perl's `DefMathI($cs, undef, undef, role => …)` defines the CS as an
**empty** XMTok with just the role attribute set.

**Why diverge:** Perl emits the literal byte (e.g. `\x10` for hex slot
`"10`) into the XML, which is **not valid XML 1.0** (§2.2: C0 chars
except 0x09/0x0A/0x0D are forbidden). When libxml2 later parses the
serialized document for post-processing (`find_node_by_id` / XPath),
it aborts mid-tree on the first invalid byte. Every `xml:id` past that
point becomes unresolvable, surfacing as the
`Error:expected:id Cannot find a node with xml:id=…` cluster (which
dominated CONVERR on second-500K canvas stage_51, ~63% of papers
with errors). U+FFFD is the canonical "unrepresentable character"
placeholder and is XML-1.0-valid, so the downstream parse stays
clean.

**Shared upstream gap:** Neither Perl nor we ship a `u.fontmap.ltxml`
nor a `("U", family="msa")`-keyed registration of the AMSa table.
Resolving the slot to its correct Unicode codepoint (e.g. U+21A0 for
`\onto` at AMSa slot 0x10) would require registering the existing
`AMSa_fontmap` data under the `"U_msa_fontmap"` key, which neither
engine currently does. The fix is parity-neutral if landed on both
sides; we defer it as a beyond-Perl improvement.

**Witness:** arXiv:1501.05180 (`\DeclareMathSymbol\onto\mathrel
{latex-font msa}{"10}`). With the U+FFFD substitution, the paper
converts cleanly through post-processing; without it, the dominant
CONVERR_N cluster fires. See `latexml_engine/src/latex_constructs.rs`
the `xml_safe_char` helper around line 6243.

---

### 28. Bib-section title = leading balanced group, not all trailing tokens

**Decision:** In `begin_bibliography_clean`
(`latexml_engine/src/latex_constructs.rs`), when deciphering
`\bibsection`'s body for the bibliography title, after stripping the
sectional-unit CS and an optional `*` we take **only the leading
balanced `{...}` group** as the title, rather than all remaining
expansion tokens. When there is no leading group (an un-braced title)
we fall back to all tokens — Perl's behavior.

**Perl ground truth:** `beginBibliography_clean`
(`LaTeX.pool.ltxml` L4035-4053) sets `$bibtitle = Tokens(@t)` — *all*
remaining tokens after the unit + `*`. Right at that line the Perl
author left the TODO: `# Check for balanced? or just take balanced
begining?` — i.e. they knew the title should be the unit's argument
(the brace group), not whatever trails it. We realize that intent.

**Why diverge:** Papers that prevent the bibliography from breaking to
a new page do
`\renewcommand\bibsection[1]{\section*{\refname}\small #1}`
(a *parameterized* `\bibsection`). After the unit+`*` strip Perl's
"all tokens" leaves `{\refname}\small #1`, and digesting that pushes
the page/font directive `\small` **and** the bare parameter token
`#1` — an ARG-catcode token that errors `The token "#1" (catcode ARG)
should never reach Stomach!`. Perl only escapes this in the witness by
a fragile, comment-line-dependent mouth artifact (the *same*
`\bibsection` macro leaks in a minimal Perl repro, perl-rc=1); the
leading-group rule fixes it deterministically and is strictly more
robust. Output is identical to Perl on the witness:
`<bibliography xml:id="bib"><title>References</title>…`. Trailing
page/font directives (`\small`, `\markboth`, `\thispagestyle`) that
LaTeXML never renders in a title are correctly dropped.

**Witness:** arXiv:1702.01165 (llncs + IEEEtranN `.bbl`,
`\renewcommand\bibsection[1]{\section*{\refname}\small #1}`).

### 29. `wrapfigure`/`wraptable` emit the declared wrap width

**Decision:** `wrapfig.sty`'s `{wrapfigure}`/`{wraptable}` set the figure/table
element's `@width` to the mandatory `{Dimension}` wrap-width argument (→ CSS
`width:`), capping the float — image *and* caption — to that width.

**Perl behavior:** Perl `wrapfig.sty.ltxml` captures the wrap width as the last
`{Dimension}` argument of the environment but then **discards it** — the emitted
`ltx:figure` carries only `float='right'|'left'`, no width.

**Why diverge:** A wrapfig float with no width constraint shrinks/expands to its
content. Under ar5iv.css (`.ltx_align_floatright { float:right }`, no width cap)
a small figure whose caption fits on one long line balloons into an enormous
box — the caption sets the float width, not the image. Real LaTeX confines the
float to the declared wrap width (`\begin{wrapfigure}{r}{0.4\textwidth}`); we
honor that intent. The width renders via the existing `@width` → `base-styling`
`width:` path (the same mechanism `{minipage}` uses), so the image (CSS
`width:auto; max-width:100%`) and the caption both wrap within the declared
width. This keeps `width:auto` working as CSS intends (the SVG/image keeps its
natural intrinsic size; the *figure* is what's bounded) rather than pinning the
image's own dimensions.

**Impact:** `<ltx:figure>`/`<ltx:table>` from wrap environments gain a
`width="<dim>"` attribute (e.g. `width="138.0pt"` for `0.35\textwidth`). Witness
arXiv:2012.00499 Figure 3 (`\begin{wrapfigure}{r}{0.4\textwidth}` around a
`width=0.4\textwidth` histogram): previously the float filled the column width to
fit the single-line caption; now both image and caption are capped to the wrap
width.

---

### 30. `\href` is `protected` (robust), unlike Perl's

Rust's hyperref binding marks `\href` `protected => true`; Perl LaTeXML does
not. Real hyperref's `\href` IS robust (`\DeclareRobustCommand`), so this is
*more* faithful to real TeX: an `\edef`/`\xdef` over `\href{u}{t}` leaves the
literal call in the body. LaTeXML's `\href` expansion re-emits `\href` itself
(the `\lx@hyper@url@` reversion argument), so WITHOUT the flag any
partial-expansion context re-expands it forever — Perl *hangs* on
`\xdef\x{\href{u}{t}}` (rc=124), and ems-journal.sty's `\Emsaffil` does
exactly that (witness 2110.10227). At top-level digestion (`fully_expand`)
protected macros still expand, so normal `\href` behavior is unchanged.
Pinned by `tests/58_href_edef_loop.rs`.

### 31. natbib bibitem labels with text-encoding symbols are not force-expanded

Perl's `\lx@NAT@parselabel` (natbib.sty.ltxml L564) unconditionally
`Expand`s a "bare" bibitem label to locate the `(year)` paren. Rust skips the
full expansion when the label carries text-encoding symbol commands
(`\i`, `\j`, `\ss`, `\oe`, …) — under `[T1]{fontenc}` the kernel's
`\@changed@cmd` dispatcher (`\T1-cmd \i \T1\i`) re-injects the CS through
`\csname\cf@encoding\string#1\endcsname`, which loops under Rust's full
expansion where Perl's happens to terminate (witness 2111.00584,
`M{\'\i}guez`). The `(year)` is always a literal paren in natbib/BibTeX
output, so the raw label suffices. This is a STOPGAP at the consumer level —
the tracked root cause is the encoding-dispatcher expansion loop itself
(SYNC_STATUS "natbib dispatcher" open item); the guard list should be deleted
when that lands. Pinned by `tests/59_natbib_label_dotless_i.rs`.

### 32. NUL's default catcode is 12 (OTHER) — Perl parity over TeXbook

The TeXbook gives NUL (`^^@`) catcode 9 (IGNORED); Perl LaTeXML uses 12
(OTHER), and Rust now matches Perl. With IGNORE, the `^^@`-notation char was
dropped at tokenization, so the alphabetic constant `` `^^@ `` skipped to the
NEXT token and returned its code (114 for `\relax`) instead of 0 — breaking
xint's `\romannumeral`&&@`` expansion idiom. An explicit `\catcode`^^Q=9`
is still honored (only the *default* changed). Stray raw NUL bytes (BibTeX
`\"u`-mangling) become OTHER chars and are stripped at the XML serialization
sinks (`xml_sanitize` in document.rs — NUL + C0 controls + U+FFFE/FFFF), so
no invalid XML and no libxml `CString` panic. Pinned by
`tests/60_caret_charcode.rs` + `tests/62_nul_byte_input.rs`.

---

### 33. Frontmatter Queue Pre-Cleared Before Deferred Digestion

**Decision:** `digest_front_matter` snapshots **and clears**
`frontmatter_raw` before digesting the queued commands. Perl
(post-PR-2767 `digestFrontMatter`) digests from the live queue and
wipes it only after the loop.

**Perl behavior:** when a queued entry's own content re-triggers
`digestFrontMatter` — which genuinely happens when a class binding's
greedy argument capture swallows the document's `\maketitle` into
queued frontmatter content — the nested invocation re-reads the
still-live queue and re-digests it, unboundedly. PR-head Perl dies
with `Fatal:perl:deep_recursion … Stomach::invokeToken` and produces
**no output** (verified against `LaTeXML@23f3acfa`, 2026-06-04). See
`KNOWN_PERL_ERRORS.md` #30 for the Perl-origin record.

**Rust behavior:** the nested invocation sees an empty queue and
terminates; the digest still happens at exactly the PR's deferred
moments (`\maketitle` / document-begin / end-of-document fallback),
in the PR's order, with late `\let`/`\def` redefinitions honored —
the divergence is *only* the termination guard. Entries queued
*during* a digest survive for the next invocation or the fallback
(Perl's post-loop wipe silently deletes them).

**Witness:** arXiv:0907.0384 (A&A, aa.cls): `\abstract{…}{}` makes
the binding dispatch the 5-arg `\abstract@new`, whose greedy `{}`
parameters swallow `\keywords` (#3, #4) and `\maketitle` (#5); the
queued abstract therefore contains `\maketitle` →
`\lx@frontmatterhere` → afterDigest re-entry. Perl: fatal, 0 bytes.
Rust: 0 errors, correct title/creator/affiliation/email joins.
(pdflatex also compiles this paper — robust behavior is the
LaTeX-like one.)

---

### 34. Contentless Frontmatter Annotation Labels Are Dropped

**Decision:** `clean_frontmatter_labels` skips fields with no real
content. Perl `cleanFrontmatterLabels` prefixes empty fields too, so
a doubled comma, a trailing-comma-plus-interior-empty, or an empty
keyval (`label={a,,b}`) yields a contentless `"prefix:"` label.

**Perl behavior:** `split(',')` + unconditional `$prefix . ':' . $label`
emits `affiliation:`-style labels with no payload; these enter the
`_annotations`/`_label` matching tables and can spuriously match
*another* contentless label during `relocateAnnotations`, attaching
an annotation to an unrelated parent. Recorded as a Perl-origin
buglet in `KNOWN_PERL_ERRORS.md` #31.

**Rust behavior:** empty fields (after trim; including `\ref{}` with
empty referent) are dropped before prefixing. Perl's `split`-drops-
trailing-empties semantics is otherwise preserved exactly.

**Witness:** none in the corpus (defensive); decided at plan time —
`docs/frontmatter_api_refactor.md` decisions log #5.

---

## Future Work (Beyond Perl Parity)

The Rust port aims first for behavioral parity with Perl LaTeXML
(see "Faithfulness first" above). But the project also positions us
to **go beyond parity** in places where Perl LaTeXML's grammar or
output choices are themselves limited. This section records
deliberate "future work" directions where we know what better looks
like; their resolution is not a parity regression to fix but an
extension of the project's value.

### Rich math-grammar parsing for kerned-stack norm idioms

**Status:** Future work — extends beyond Perl LaTeXML.

**Background.** Papers routinely fake double-bar and triple-bar
norms by stacking `\left|\right|` pairs with small negative kerns:

```latex
\newcommand{\vertii}[1]{{\left\vert\kern-0.25ex\left\vert
                          #1 \right\vert\kern-0.25ex\right\vert}}     % ‖x‖
\newcommand{\vertiii}[1]{{\left\vert\kern-0.25ex\left\vert\kern-0.25ex
                          \left\vert #1
                          \right\vert\kern-0.25ex\right\vert\kern-0.25ex
                          \right\vert}}                                % |||x|||
```

Visually the bars touch and render as `‖x‖` / `|||x|||`. Semantically
both Perl LaTeXML and the Rust port currently parse each
`\left|`/`\right|` pair as an *independent fence delimiter*,
producing nested `|·|` inside `|·|` rather than a single
norm-delimiter pair. For a juxtaposed expression like
`|||M||| · |||Σ||| · ‖M−M'‖_F + ‖M−M'‖_F · |||Σ||| · |||M'|||`
this yields ~25-level nesting in MathML (witness
`tests/math/norm_kerned_delims.tex`, originally from arXiv:2211.13044
§S4.Ex17).

**Why this is "beyond parity" not a regression.** Perl LaTeXML
focuses on fence-pairing rules that mirror TeX's `\left`/`\right`
matching and does not attempt to detect kerned-stack idioms. The
Rust port's math layer is built on a more expressive Marpa-based
grammar (see [`MATH_GRAMMAR_FIRST_PRINCIPLES.md`](MATH_GRAMMAR_FIRST_PRINCIPLES.md)
and [`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md)), giving us
the option to produce **well-structured MathML Core** that follows
the XMath taxonomy: a single `<mrow intent=":Frobenius-norm">` or a
proper U+2016 `‖` / U+2AF4 `⫴` delimiter, instead of token-level
fence soup.

**Approach (sketch, three layers — pick any):**

1. **Gullet-level rewrite.** Detect the kerned-stack pattern in the
   gullet (the kern argument has a known small negative value
   between two adjacent `\left|` or `\right|` tokens) and merge into
   a synthesized macro like `\lx@doublebar` / `\lx@triplebar`. The
   math parser then sees clean delimiters and the existing fence
   rules produce well-typed MathML directly. Smallest blast radius.

2. **Math-grammar level.** Add explicit NORM / OPERATORNORM
   nonterminals to the Marpa grammar that accept balanced `|`/`‖`/
   `|||` openings, with their own action closures that emit a
   semantic `intent=":operator-norm"` mrow. This is the
   "richer-grammar" path the Rust port was designed to enable.

3. **Both, with role tagging.** Pre-process at the gullet AND keep
   the grammar prepared for U+2016 / U+2AF4 delimiters arriving on
   the token stream. Belt-and-suspenders for varied paper inputs.

**Related future-work item (same paper, same equation).** Equation
rows whose first non-whitespace token is a binary relation (e.g.
`\leq`, `=`, `\subseteq`) currently get a phantom `<mi></mi>`
left operand inserted by the math parser. The continuation-row
semantics — "the LHS is the prior row" — should be made explicit
either by suppressing the empty operand or by tagging the row with
`intent=":continuation"`. Tracked as task #264.

**Pinned-baseline test.** The current (over-nested) output is
captured as `tests/math/norm_kerned_delims.{tex,xml}` so we can
detect when a future grammar/preprocess change *improves* it
without it silently regressing. The test file's leading
`% comments` annotate each section with the expected shape.

### TOML profiles instead of Perl `.opt` (issue #191, `--profile`)

**Status:** Planned — not yet implemented. Deliberate divergence from
Perl's profile file format.

**Perl behavior.** `--profile=NAME` (and its `--mode` alias) loads
`<NAME>.opt` — a flat `key = value` file (`Config.pm::_obey_profile`).
We already ship the set under `resources/Profiles/*.opt` (`fragment`,
`math`, `standard`, `modern`, `stex*`, …). The format has three warts: an
empty value means "boolean true" (`pmml =`), lists are repeated keys
(`preload = …` ×N), and everything is stringly-typed.

**Planned Rust shape.** Express profiles as **TOML**, deserialized via
serde into the same option struct `clap` already populates — so a profile
is just a *defaults layer*: `built-in/embedded profile < user CLI flags`
(CLI wins, matching Perl's precedence). TOML fixes all three warts
natively (`pmml = true`, `preload = ["a","b"]`, `timeout = 120`) and adds
`extends = "fragment"` profile inheritance that `.opt` can't express
cleanly.

```toml
# fragment.toml
extends   = "math"          # optional inheritance
format    = "xhtml"
whatsin   = "fragment"
whatsout  = "fragment"
pmml = true; cmml = true; mathtex = true
nodefaultresources = true
preload = ["LaTeX.pool", "article.cls", "amsmath.sty", "[ids]latexml.sty"]
path    = ["$LATEXMLINPUTS"]
```

**Decision (2026-05-24): TOML-native, convert-and-drop.** Convert the
shipped `resources/Profiles/*.opt` to `*.toml` and remove the `.opt`
files; **no legacy `.opt` reader** — `--profile` consumes only TOML. (A
Perl `.opt` is trivially hand-portable, and we control the shipped set, so
the compat reader isn't worth the surface area.)

**Constraints to preserve:** built-in profiles stay **embedded**
(`include_str!`/`include_dir!`) per the self-contained-binary principle,
with a disk override (`<NAME>.toml`); keep `$LATEXMLINPUTS` expansion in
`path`; keep `--mode` as an alias for `--profile`.
Tracked under issue #191 in [`ISSUE_AUDIT.md`](ISSUE_AUDIT.md).
