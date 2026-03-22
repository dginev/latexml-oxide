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

---

## Architecture

### System-Level View

LaTeXML (Perl) has two main programs: `latexml` (TeX→XML) and `latexmlpost` (XML→HTML/MathML).
The Rust port currently covers the `latexml` pipeline. The `latexmlpost` pipeline is planned
for Phase 3 (see `mini_3_plan.md`).

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
full inter-conversion. `to_attribute()` produces identical hex strings. Model-aware
comparison matches Perl's `isDiff` semantics (cmyk black ≠ rgb black).

### 6. Font Defaults: None vs Named Strings

**Decision:** `DEFBACKGROUND = None` and `DEFLANGUAGE = None` (Perl uses `undef`).

**Rationale:** Perl's `undef` for these defaults is semantically "no value set", not
"white" or "en". The Rust port uses `Option<Color>` and `Option<Cow<str>>` to represent
this correctly, rather than sentinel strings.

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

### 18. Source-Level Bindings via `*.src` Files

**Decision:** Perl's per-document `.latexml` files are replaced by `*_src.rs` files in the `latexml_contrib` crate, loaded via `\input{name.src}` in the `.tex` source.

**Perl mechanism:** When processing `foo.tex`, Perl automatically checks for `foo.latexml` in the same directory. If found, it loads and executes the Perl code, which typically contains `DefMathRewrite`, `DefMacro`, `DefConstructor` calls that customize the conversion for that specific document.

**Rust mechanism:**
1. The `.tex` file includes `\input{name.src}` to explicitly request the binding
2. The `latexml_contrib` dispatcher maps `"name.src"` to `name_src::load_definitions()`
3. The `*_src.rs` file in `latexml_contrib/src/` contains the Rust equivalent of the `.latexml` definitions

**Rationale:**
- Rust cannot interpret Perl at runtime, so `.latexml` files cannot be loaded directly
- Compile-time binding registration is required for Rust's type system
- Explicit `\input{name.src}` makes the dependency visible in the TeX source
- The `*_src.rs` naming convention distinguishes source-level bindings from package bindings (`*_sty.rs`)

**Critical insight:** Math rewrite rules (`DefMathRewrite`) in `.latexml` files execute BEFORE the Marpa grammar parses the expression. This means setting `role="ID"` or `role="FUNCTION"` via rewrites changes how the grammar interprets the tokens — it is NOT equivalent to a post-processing role change. The `*_src.rs` mechanism preserves this pre-parse semantics.

**Example:** `simplemath_src.rs` mirrors `simplemath.latexml`:
```rust
// Sets MATHPARSER_SPECULATE + rewrite rules for a,b,x,D → ID, f → FUNCTION
add_math_rewrite("a", "ID")?;
add_math_rewrite("f", "FUNCTION")?;
AssignValue!("MATHPARSER_SPECULATE" => true, Scope::Global);
```

**Impact:** Tests with `.latexml` files need corresponding `*_src.rs` files and `\input{name.src}` in their `.tex` source to get the same parsing behavior as Perl.
