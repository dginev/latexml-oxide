# Oxidized Design — Intentional Divergences from Perl

[← OXIDIZED_DESIGN.md](OXIDIZED_DESIGN.md) · Deliberate breaks with Perl behavior, numbered. Code comments reference these as `OXIDIZED_DESIGN #N`.

> **Numbering note:** the `### N` numbers are load-bearing (referenced from `.rs` comments) and are kept verbatim. `#16` and the math-grammar entries `#7–#18` live in [OXIDIZED_DESIGN_MATH.md](../math/OXIDIZED_DESIGN_MATH.md); in particular the code-referenced **`#18` is the f(x) "Speculative function application"** entry there, *not* the "Source-Level Bindings" `#18` below.
>
> **`#76` is a RETIRED number, not an omission** — its entry was consolidated into `#74` and the number was deliberately not reused (see the placeholder in sequence below). Next free number: **#379**.

---

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

**Related — source comments off by default (`INCLUDE_COMMENTS`):** Perl LaTeXML
defaults `INCLUDE_COMMENTS` to *true* (Core.pm L143), so it preserves source `%`
comments in the output as XML comments AND sneaks a `%**** <file> Line N ****`
progress marker into the stream every 25 lines (Mouth.pm:334). The Rust binary
defaults it to *false* (`converter.rs`: `include_comments.or(Some(false))`; the
test harness/presets pass `Some(false)`), so neither real `%` comments nor the
`****` line markers appear by default. This is deliberate: those comments are
source-debugging noise with no semantic content for downstream consumers, and
suppressing them keeps the XML clean. The machinery is fully ported (mouth.rs
emits both when `INCLUDE_COMMENTS` is on), so `--comments` restores Perl's
behavior; a handful of fixtures generated with comments enabled (e.g.
`hello/hello_new.xml`) exercise that path. When diffing against Perl, run Perl
with `--nocomments` (or ignore `<!-- … -->` / `%**** … ****` lines).

### 3. `\cdots` Role: ELIDEOP Instead of ID

**Decision:** `\cdots` uses `role="ELIDEOP"` (Perl uses `role="ID"`).

**Rationale:** This enables dedicated grammar rules in the Marpa math parser
(e.g. `term mulop tight_term elideop => infix_apply_and_elide`) for better-structured
parse trees. The ID role is too generic for ellipsis operators.

**Impact:** Test XMLs must use `role="ELIDEOP"` for `\cdots`.

An ELIDEOP is also a factor (57bs, user ruling 2026-09-29), as Perl's ID is: it stands among juxtaposed factors and
as a MulOp/BinOp operand — `a_1a_2\cdots a_n` a₁·a₂·⋯·aₙ, `a\times\cdots\times b`, `x_{i_1\cdots i_k}`,
`(1-x)\cdots(1-x^n)` (1,035 formulas in 236 A/B papers were unparsed; 2605.10811, 2605.31453). Not a `factor_base`:
a trig bare argument, a limit-from and a differential take none; an OPFUNCTION's or operator's bare argument does, as
Perl's `aBarearg` takes an ID (`\nabla u\cdots` ∇@(u·⋯)) (57bx: `\max_{m}|a||b|\cdots|c|` max_m@(|a|·|b|·⋯·|c|), 2605.23673; `\log\cdots` log@(⋯)
was unparsed). A product ending in a bare ellipsis is no
right operand of a visible operation (`infix_apply_nary`), so an operation elided at the end keeps its elision reading
(`a\cdot b\cdots` a cdot b cdot cdots). Golden `tests/parse/ellipsis_products.tex`.

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

**Decision:** The `tex=` attribute on `<ltx:picture>` elements is suppressed
**unconditionally**. A `LATEXML_SVG_TEX_ATTRIBUTE=true` opt-in was designed but never
implemented — the name appears in no source file (verified 2026-07-29), so there is no
way to turn the attribute back on.

**Perl behavior:** Perl emits a `tex=` attribute on `<picture>` containing the full TeX
source of the tikz/pgf picture environment. This can be extremely long (thousands of
characters of raw pgf commands) and is not used by downstream consumers.

**Rationale:** The `tex=` attribute on pictures is a debugging artifact. It inflates the
XML output size significantly (often 10x the rest of the element) with raw pgf
instructions that are illegible and serve no rendering or accessibility purpose.
Suppressing it produces cleaner output at no cost to any downstream consumer.

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
cost (see `docs/performance/PERFORMANCE.md` §5). HTML output still has the
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

#### Package-load flag machinery (`_loaded` / `_raw_loaded` / `_found_loaded`)

*(Cross-refs #23. This block sits under #25 for historical reasons but is about
the load-flag scheme.)*

- **Path-aware gating** (`binding/content.rs` `already_handled`, commit
  `de21ae928`): a binding `<file>.rs` may `InputDefinitions(noltxml=>1)` its
  same-named raw `.sty/.cls` *after* its own `_loaded` is set (e.g. `babel_sty.rs`
  → raw `babel.sty`). The gate therefore keys on the path taken — `noltxml`
  (raw-only) → check only `_raw_loaded`; `notex` (binding-only) → only `_loaded`;
  default → either. The step-4 raw-search gate checks only the raw flag;
  `_load_binding` keeps a binding-only `_loaded` gate (mirrors Perl `loadLTXML`,
  Package.pm L2311).
- **Reader semantics** (commit `01df250c6`): user-level "is X loaded?" queries
  (`\@ifpackageloaded`, soul/cleveref probes) consult EITHER flag.
- **Perl error semantics:** Perl sets `_loaded` *before* reading (persists on read
  error → later calls early-skip); "did it succeed" is answered by the return
  value, not a flag. Rust mirrors this. The Rust-only `_found_loaded` flag (a
  redundant "actually succeeded" marker with no Perl equivalent) is slated for
  removal — audit its ~6 read sites to `_loaded` / `_raw_loaded` / an explicit
  `Result` check, leaving exactly `_loaded`, `_raw_loaded`, `_loaded_with_options`.
- **Dump skip-list:** Perl dumps emit all `_loaded` flags verbatim (both raw
  `expl3.ltx_loaded` and binding `expl3.ltx.ltxml_loaded`). Rust's
  `dump_reader::SKIP_VALUE_CONTAINS = ["_loaded"]` is a workaround — a dump-loaded
  `_loaded` wrongly short-circuits later `\input`s (e.g. babel hyphenation
  registers). It becomes removable once `LoadFormat`-style dump-vs-raw mutual
  exclusivity lands (SYNC_STATUS D0): only one path is ever active.

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

**Extension — inner `\linewidth`:** the `@width` cap on the figure is not enough
when the body carries its OWN intrinsic size — a `\includegraphics[width=\linewidth]`
or `\begin{minipage}{\linewidth}` that read `\linewidth` at the full page width
render a fixed-size box (e.g. a tikz picture serializes to `<svg width="479">`)
that overflows the narrow float and collides with the wrapped text (`max-width:100%`
does not rein a fixed-`width` SVG in). So `set_wrap_width` also reduces the INNER
`\hsize`/`\columnwidth`/`\linewidth` to the wrap width (in `after_digest_begin`,
before the body digests), exactly as real LaTeX wrapfig does (`\hsize<width>` +
`\@parboxrestore`'s `\linewidth\hsize`) and as `{minipage}` already does here.
`\textwidth` is left alone (the `@width` percentage is relative to the OUTER
textwidth). Perl discards the width arg entirely, so Perl's wrapfig image renders
full-width too (SHARED-FAILURE). Witness arXiv 2603.23669 Figure 3
(`0.296\textwidth` wrapfigure around a 512px tikz image): the picture dropped from
479px → 141px and the text now wraps cleanly. Guard
`cluster_sizing::wrapfig_inner_linewidth`.

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
`docs/archive/frontmatter_api_refactor.md` decisions log #5.

---

### 35. etoolbox `\robustify` is a no-op on native (closure) bindings

**Decision:** `\robustify` (and the etoolbox patching family it shares
machinery with) leaves a **native, Rust-closure-bodied** binding
unchanged, instead of reconstructing it from its `\meaning`.

**Rationale:** etoolbox's `\robustify` makes a *fragile* macro robust by
reading its `\meaning` (`macro:<params>->...body...`), then
re-`\def`-ing it (via `\scantokens`) wrapped in `\protected`. That round
trip only works when the body is real tokens. Many LaTeXML commands are
realized as native closures whose `\meaning` renders as
`...->CODE(0x<ptr>)`; reconstructing from that produces a broken macro
whose param text (`#1#2#3#4`) is taken literally and whose body is the
literal text `CODE(0x…)` — so e.g. a robustified natbib `\cite` grabs the
wrong number of arguments and can swallow a following `\begin{equation}`.
Native bindings are *already* robust (no `\protect` fragility), so the
faithful-to-intent behavior is to leave them alone.

**Perl behavior:** Perl LaTeXML ports the identical etoolbox
`\etb@robustify` and has the **same** bug — its robustified native `\cite`
emits the literal pointer text (`Start CODE(0x…)…`) — it simply does not
raise an `Error:`. So this is a **surpass-Perl** correction, not a Perl
parity match: Rust both avoids the error *and* keeps `\cite` working.

**Implementation:** `\lx@ifnativecmd` in
`latexml_package/src/package/etoolbox_sty.rs` mirrors etoolbox's own
`\ifdefmacro` `\meaning`-split idiom (sentinel `CODE(`); `\robustify` is
wrapped to no-op on natives and delegate to the original for token macros.

**Witness:** 2110.11931 (mnras — its template ships `\robustify{\cite}`):
10 errors → 0, with correct citation output. User-macro robustify
(`\robustify{\foo}`) is unaffected. (The `\patchcmd`/`\apptocmd`/`\pretocmd`
siblings were checked and do NOT need wrapping: on a native binding they hit
etoolbox's `\etb@ifscanable`-FALSE branch and **fail gracefully** via the
caller's `{fail}` callback, leaving the binding intact — verified
`\patchcmd{\cite}…`/`\apptocmd{\cite}…` → graceful fail, 0 errors, no garbage.
Only `\robustify`'s `\ifdefparam`-false → `\protected\edef` path was broken.)

### 36. Author-list splitting protects balanced parentheses

**Decision:** `SplitTokens` (`base_utilities.rs`, the author/frontmatter
list splitter) does NOT match a delimiter (`,`, ` and `, `\and`, `\quad`, …)
that sits inside balanced `(…)` parentheses — extending the brace `{…}` and
math `$…$` protection it already has.

**Rationale — what the heuristics assume, and why this is the safe level.**
`\author{}` is free-form; LaTeX's only *designed* author separator is
`\and`. To recover author lists from documents that didn't use it, LaTeXML
heuristically also splits on `,`, the literal word ` and `, and `\quad`.
Those tokens are **ambiguous**: the same `,` is an author separator in
`Alice, Bob` and ordinary punctuation in an affiliation `MIT, Cambridge`.
The *unambiguous* signal is syntactic **grouping**: content inside a balanced
grouper is one unit and must never be split. Braces and math were already
protected; parentheses are the remaining natural text grouper, so a
parenthesized affiliation `(Scuola Normale Superiore, Pisa)` is now kept
whole. The guard `paren_closes_ahead` means an *unbalanced* `(` is treated as
an ordinary token (it must not greedily swallow a later `\\` name/affiliation
separator).

**Perl behavior & scope.** Perl's `SplitTokens` (Base_Utility.pool.ltxml)
protects braces/math but NOT parens, so it makes the same mistake — witness
**arXiv 0804.0870**, where `\author{Alessio Martini\\(Scuola Normale
Superiore, Pisa)\\…}` produced a spurious second `<personname>Pisa)`. So this
is a **surpass-Perl** correction. It deliberately stops at the *unambiguous*
case: bare (unparenthesized) commas/` and ` in an affiliation (`MIT,
Cambridge`; `School of Arts and Sciences`) and `Lastname, Firstname` name
order remain genuinely undecidable from the token stream alone — the same
tokens read as either one comma-affiliation or two authors — so we keep
Perl's recall-oriented over-split there rather than substitute a different
wrong guess. Authors who want such an affiliation kept whole can group it in
`{…}` or `(…)` (both now honoured).

**Witnesses (real arXiv, both Perl-wrong):** 0804.0870 —
`(Scuola Normale Superiore, Pisa)` (comma in parens) stays one affiliation;
hep-ex0007011 — `(On behalf of the H1 and ZEUS collaborations)` (the literal
` and ` separator in parens) stays one affiliation instead of splitting off a
spurious `ZEUS collaborations)` author. So the protection covers *every*
delimiter inside the group, not just commas. Suite 1465/0; verified
balanced/nested parens protect, unbalanced parens do not regress the `\\`
split.

A delimiter also matches an alias of its own definition (Perl's `Equals`,
Token.pm:321-327, compares catcode and name only): `\AND` `\let` to `\and`
splits as `\and`. Only its own — a delimiter `\let` to another command matches
itself alone, or an `\and` disabled to `\relax` by a `\maketitle` split a later
`\author` at every `\relax` alias, `\protect` and a disabled `\thanks` (59e;
guard `perfect_kernel_batch59::relax_aliases_do_not_split_a_late_author`).

### 37. XSLT `f:seclev-aux` memoized to global variables (O(n²)→O(n), output-neutral)

**Decision:** In the embedded `resources/XSLT/LaTeXML-structure-xhtml.xsl`,
the recursive `f:seclev-aux` (which computes a section heading's `<hN>` level)
is replaced by a lookup into precomputed global `<xsl:variable>`s
(`seclev_document` … `seclev_backmatter`). The function body now just selects
the variable matching the element-type name.

**Perl behavior:** upstream LaTeXML's `f:seclev-aux` recomputes whole-tree
`boolean(//ltx:chapter/ltx:title)`-style **descendant scans** on *every* call,
and `f:section-head-level` calls it once per `ltx:title`. That is
O(headings × tree-size) ≈ **O(n²)** — the dominant XSLT cost on large
section/math-heavy documents.

**Rationale & neutrality:** the level for a given element-type *name* is a
**document-global constant** — it depends only on which structural element
types are present (the `boolean(//…)` probes), never on the calling node. So
computing it once per name yields *identical* values; only the redundant
recomputation is removed. Verified byte-identical (a 99k-element truncation of
arXiv 2404.12418 `diff`s IDENTICAL pre/post; full suite 1480/0 unchanged).

**Impact:** 2404.12418 went 179 s fatal-timeout → 34.7 s; all 14 "XSLT-dominated"
arXiv perf-testbed papers (formerly 176–179 s timeouts) now complete. This is a
**surpass-Perl** perf win (Perl keeps the O(n²); Rust @99k is now 5.3 s vs Perl
8.7 s on the same stylesheet) and a candidate to upstream. Local divergence from
upstream XSLT only. Full analysis: `docs/performance/ARXIV_PERFORMANCE.md` (Hotspot #2).

### 38. `theorem`/`proof` allowed inside `figure`/`table`/`float` (schema expansion)

**Decision:** The schema content models for `ltx:figure`, `ltx:table`, and
`ltx:float` now permit `ltx:theorem` and `ltx:proof` children. Edited the
precompiled `resources/RelaxNG/LaTeXML.model` (the flattened `canContain` table the
document builder actually consults) plus the `figure_model`/`table_model`/`float_model`
source in `resources/RelaxNG/LaTeXML-para.{rng,rnc}`.

**Perl behavior:** upstream LaTeXML's float models do NOT include theorem/proof, so
Perl emits `Error:malformed:ltx:theorem <ltx:theorem> isn't allowed in <ltx:figure>`
for the same input (verified: parity — both engines error).

**Rationale & neutrality:** a boxed/framed theorem or proof inside a figure/table
float is valid LaTeX (e.g. `\begin{figure}…\begin{theorem}…\end{theorem}…`). The
document builder already PLACED the theorem inside the figure (it logged the schema
error but inserted the node anyway), so accepting it in the model is **output-neutral**
— the golden `figure_mixed_content.xml` is byte-identical pre/post; only the spurious
malformed-error disappears. The change is **monotonic** (strictly more permissive): it
cannot invalidate any document that validated before, so no existing test can break
(full suite 1481/0 unchanged).

**Impact:** drains the last `ERROR_DEBT` entry (`figure_mixed_content`); `ERROR_DEBT`
is now empty. Surpass-Perl; candidate to upstream. (mdframed-style framed blocks
typically lower to `float`/`theorem` too, so they benefit as well.)

### 39. `\marginpar` font/catcode changes are scoped (`bounded`)

**Decision:** `\marginpar[]{}` (`latex_constructs.rs`) now carries `bounded => true`,
so font/catcode switches inside the margin note are local to the note. Mirrors
`\mbox`'s `bounded => true`.

**Perl behavior:** upstream Perl LaTeXML's `\marginpar` is NOT bounded, so a
`\marginpar{\Large …}` **leaks** the `\Large` (or any switch) into the body text that
follows. Verified parity bug — Perl LaTeXML 0.8.8 reproduces it identically
(`\marginpar{\Large !} X` renders `X` at 144%); real pdflatex scopes the note to its
margin box, so the leak is a LaTeXML-engine bug shared by both ports, NOT a Rust
regression.

**Rationale:** the margin note's content is conceptually a separate box; its size/font
changes must not affect the main galley. **Witness:** the mhchem manual's
`\marginpar{\Large !}` (line 120) leaked `\Large` document-wide, rendering the ENTIRE
manual at 144% (1388 `fontsize="144%"` nodes → 4 after the fix). Output-neutral across
the suite (1487/0): no golden test relies on the leak. Surpass-Perl; candidate to
upstream. See `KNOWN_PERL_ERRORS.md`.

### 40. XSLT `head-keywords` index dedup via Muenchian key (O(n²)→O(n), output-neutral)

**Decision:** In the embedded `resources/XSLT/LaTeXML-webpage-xhtml.xsl`, the
`head-keywords` template (which builds `<meta name="keywords">` from the distinct
index phrases) selects its distinct set with a hashed `xsl:key`
(`f:indexphrase-by-value`, the **Muenchian method**:
`//ltx:indexphrase[generate-id() = generate-id(key('f:indexphrase-by-value',.)[1])]`)
instead of upstream's `//ltx:indexphrase[not(.=preceding::ltx:indexphrase)]`.

**Perl behavior:** upstream LaTeXML deduplicates by testing each indexphrase
against the entire `preceding::ltx:indexphrase` axis — O(P²) string comparisons in
the indexphrase count P, and each `preceding::` traversal is itself O(tree-size).
On index-bearing math documents (large trees) this is the dominant XSLT cost. Perl
keeps the O(n²).

**Rationale & neutrality:** the Muenchian key returns, for each distinct
string-value, the first indexphrase in document order — exactly the set
`not(.=preceding::)` keeps. The `<xsl:sort>` is unchanged, so the keywords string is
**identical**. Verified byte-identical via `xsltproc` (full HTML `diff` IDENTICAL on
arXiv 2208.07515) and a full-pipeline regression guard
(`08_xslt_head_keywords.rs`); suite unchanged.

**Impact:** the `head-keywords` template went 145 s → 0.04 s on 2208.07515 (560
indexphrases); cluster-wide the index-bearing arXiv perf survivors dropped 2–5×
(2208.07515 95 s→33 s, 1802.06435 78 s→17 s, 0807.4838 78 s→13 s). This **supersedes**
the prior campaign's deferral of the "third XSLT O(n²)" (`docs/performance/ARXIV_PERFORMANCE.md`)
— head-keywords, not the index-render templates, was the real root. Surpass-Perl;
candidate to upstream. Local divergence from upstream XSLT only. Full analysis:
`docs/performance/ARXIV_PERFORMANCE.md` (Hotspot #3).

### 41. XSLT `maketitle` navigation scan memoized to a global variable (O(n²)→O(n), output-neutral)

**Decision:** In the embedded `resources/XSLT/LaTeXML-structure-xhtml.xsl`, the
`maketitle` template decides whether to emit the title's `\date` block with
`not($maketitle_has_up_nav)`, where `maketitle_has_up_nav` is a single global
`<xsl:variable select="boolean(//ltx:navigation/ltx:ref[@rel='up'])"/>` evaluated
once. Upstream re-evaluates `not(//ltx:navigation/ltx:ref[@rel='up'])` **inline, once
per title**.

**Perl behavior:** upstream LaTeXML scans `//ltx:navigation` (a full descendant
traversal from the document root) inside `maketitle`, which runs for every titled
unit. On a large book with hundreds of titles this is O(titles × tree-size) — Perl
keeps the O(n²).

**Rationale & neutrality:** `//ltx:navigation` always resolves from the root
regardless of the current title (the `//` axis resets to the document node), so the
boolean is document-global and identical for every title. Hoisting it to a global
variable changes nothing in the output — verified `xsltproc` **byte-identical** HTML
on the 25 MB Core XML of arXiv 2605.01585, plus a full-pipeline regression guard
(`09_xslt_maketitle_navscan.rs`, asserting the `\date` still renders for a non-split
document where the memoized value is `false`).

**Impact:** `maketitle` self-time 22.739 s → 0.004 s; the whole html5 transform
24.94 s → 2.15 s (11.6×) on 2605.01585 (a 2000+-formula physics book, 512 titles).
This was the dominant residual XSLT cost on large math books after #2/#3 landed.
Surpass-Perl; candidate to upstream. Local divergence from upstream XSLT only. Full
analysis: `docs/performance/ARXIV_PERFORMANCE.md` (Hotspot #4).

---

### 42. `\linewidth` tracks the reduced text width in boxed contexts (kernel-faithful; Perl leaves it stale)

**Decision:** Three coordinated completions make `\linewidth` inside a
box reflect the box's text width, as in real LaTeX:

1. The `{minipage}` binding's width assignment (Perl latex_constructs.pool
   L4787-4789 assigns `\hsize`/`\textwidth`/`\columnwidth`) additionally
   assigns `\linewidth`.
2. The `\parbox` raw macro (Perl L4746, same trio) appends
   `\linewidth\hsize`.
3. `\@parboxrestore`/`\@arrayparboxrestore` are real macros ported from
   `latex.ltx` (minus the `\if`-lets and accent `\let`s LaTeXML manages
   itself) instead of Perl's empty/`\relax` stubs — relevant on the
   no-dump path; with a format dump the raw `latex.ltx` kernel versions
   are captured anyway.

**Why:** Real LaTeX's `\@iiiminipage`/`\@iiiparbox` run `\@parboxrestore`,
whose `\linewidth\hsize` is what raw-loaded packages read back. tcolorbox
wraps every box's content in `\minipage` (`tcb@lrbox`) and sizes a nested
`tcolorbox` as `width=\linewidth` — with `\linewidth` stale at the page
width, an inner box drew itself full-outer-width and overflowed its parent
frame (arXiv 2605.02240, `innercode` inside `responsebox`). Probe
(`nested.tex`, outer+inner tcolorbox): pdflatex gives OUTER
`hsize=linewidth=313.70206pt`, INNER `282.40411pt`; after the fix Rust
matches **both to the sp**; Perl (and pre-fix Rust) leave `linewidth=345pt`
at both levels.

**Perl behavior:** shared limitation — Perl's minipage binding assigns only
the trio, and its `\@parboxrestore` is `Tokens()`. Perl does not draw
boxes from measured sizes at this fidelity, so the staleness is invisible
there; in our sizing-driven pgf pipeline it is a visible frame overflow.
Candidate to upstream.

**Golden churn:** `figure_dual_caption.xml` — `\includegraphics[width=0.95\linewidth]`
inside `\begin{minipage}{.5\textwidth}` now yields 163.87pt (= 0.95 x 172.5,
the pdflatex value); the prior 327.75pt golden had the stale full-page
`\linewidth` baked in (image at double its true width).

### 43. Repeat package loads apply surviving handlers for NEW options (modern-kernel fidelity)

**Decision:** When an already-loaded package is `\usepackage`d/`\RequirePackage`d
again with options the first load did not have, `input_definitions` digests
any surviving `\ds@<option>` handler for each new option before skipping the
load (plus the pre-existing Info diagnostic). Bindings opt IN to durable
repeat-options by re-asserting the handler after `ProcessOptions!` (classic
handlers are cleared to `\relax`); the first adopter is xcolor's `table`
(`\ds@table` -> `\RequirePackage{colortbl}`).

**Why:** Real xcolor v3.02+ (TL2024) processes options as PERSISTENT l3
key-values: `\usepackage{xcolor}` ... `\usepackage[table]{xcolor}` raises NO
option clash — the repeat load processes the `table` key and loads colortbl,
so `\cellcolor` works and such papers build cleanly on arXiv. Both Perl
LaTeXML and the old Rust behavior drop repeat-load options (classic-options
semantics), leaving `\cellcolor` undefined — a ~483-paper error cluster in
sandbox-arxiv-2605 (witness 2605.00310: 0 errors and 133 colored cells after
the fix; previously mis-classified as "parity option-clash" against the
obsolete semantics).

**Scope/safety:** only options with a live (non-cleared) handler fire —
packages that never re-assert handlers behave exactly as before (digesting
`\relax` is a no-op). `\ds@<opt>` is a global namespace, so a later package
redeclaring the same option name could in principle leave a stale handler;
accepted as rare next to the recovered class. Perl divergence: Perl skips
silently; candidate to upstream alongside a survey of other l3-keyval
packages whose options should be durable.

### 44. Vertical stacking: `\prevdepth` is transparent to glue (TeX vpack discipline; Perl #2798 resets it)

**Decision:** In `compute_boxes_size_stack` (the height estimator for every
vertical list: `\vbox`/`\vtop`, minipage, `p{}` cells, tcolorbox content),
vertical glue entries are TRANSPARENT to `\prevdepth` — only a box updates
it (to its depth), and only a rule disables it (TeX's `\prevdepth=-1000pt`
sentinel). Encoded as per-line flags: box = its baseline, `-1` = glue
(transparent), `-2` = rule (reset).

**Why:** the ported Perl #2798 algorithm folds vskips and rules into one
`-1` flag and resets prevdepth for both, so ANY glue item between lines
silently disables `\baselineskip` accounting for the following line.
Content shaped "box, glue, box, glue, ..." (fancyvrb interlines, list
`\itemsep`, author `\vspace`) is systematically under-measured — up to
exactly 2x for strict alternation. Witness 2605.00468: 49-line verbatim
Prompt boxes budgeted 292.6pt vs the TeX-true ~588pt; content spilled
through every following box. After the fix the budget lands at 58.3em vs
TeX's ~58.8em. tex.web vpack is the ground truth; upstream candidate
against Perl's Common/Font.pm.

**Perl parity note:** vskip-interleaved stacks now measure TALLER than
Perl (which keeps the flawed reset) — e.g. the itemize-in-vbox probe that
previously matched Perl to the sp. Deliberate: truer to TeX, and the safe
direction for frame/content agreement.

### 45. NFSS family-code vocabulary extended to modern font packages

**Decision:** `FONT_FAMILY` (Common/Font.pm `%font_family` port) gains the
family codes of the dominant modern font packages: inconsolata (`zi4`,
`fi4`), TeX Gyre (`qcr`/`qpl`/`qtm`/`qbk`/`qcs`/`qhv`/`qag`/`qzc`), Latin
Modern (`lmr`/`lmss`/`lmtt`/`lmvtt`), Bera (`fvm`/`fve`/`fvs`), Source
Code Pro / Fira Mono codes.

**Why:** raw `\fontfamily{<code>}\selectfont` (fancyvrb's font setup, and
any package that repoints `\ttdefault` et al.) decodes the code through
this table to recover the ABSTRACT family; unknown codes silently lose it.
colm2026_conference loads inconsolata (`\ttdefault`=zi4), so boxed
Verbatim dropped `ltx_font_typewriter` — the browser painted full-size
serif prose inside frames TeX measured as compact monospace (witness
2605.00468). Perl's table has the same gap (frozen at ~2005-era fonts);
upstream candidate. Future refinement: derive family knowledge from `.fd`
files instead of an enumerated table.

### 46. foreignObject font-size anchor = the font's QUAD, not its point size

**Decision:** the `font-size:<N>pt` appended to a measured box's
`--ltx-fo-*` style (`tex_box.rs`, Perl TeX_Box.pool L427-430) is emitted
as `em_width/65536` — the SAME quad the `--ltx-fo-width/height/depth` em
values were divided by — instead of Perl's `$f->getSize`.

**Why:** the em values only reproduce the TeX dimension if the browser
multiplies them by the em basis used to divide. Perl divides by
`emValue` (the quad) but anchors at the point size, so any font whose
quad ≠ size renders systematically off: cmr7's quad is 7.97pt at size
7pt, shrinking every 70%-scaled tikz label 12% under TeX truth; cmtt10
(quad 10.5pt) shrinks typewriter-content boxes 5%. With the quad anchor,
`em × anchor = TeX pt` holds exactly for every font. Upstream candidate.
Golden churn: `font-size:7pt` → `font-size:7.97pt` in the tikz suite
(5 fixtures re-blessed 2026-07-04 after per-diff review).

### 47. Typewriter whitespace is never ignorable (verbatim indentation)

**Decision:** whitespace-only TYPEWRITER-font text is inserted rather
than dropped by the document builder's two ignorable-whitespace gates
(`open_text`'s initial guard + `open_text_internal`'s Perl-L1146 gate,
bridged by a `verbatim_space_pending` handoff), and the `ltx:p`
afterClose edge-trim (i) skips paragraphs whose PARENT font context is
typewriter and (ii) stops its recursion at `font="typewriter"` text
wrappers. `ltx:verbatim` itself stays trimmable (Perl trims an inline
`\verb`'s leading space at a paragraph edge — tokenize/verb.t parity).

**Why:** fancyvrb/fvextra line-map verbatim into ltx:p's, where leading
spaces ARE code indentation and a space-only line is content; both
engines' whitespace machinery predates that shape and deleted the
indentation (2605.00468 JSON schemas flush-left, 15–33px measured-frame
spills). Line-leading cat-10 SOURCE spaces never reach these gates (the
mouth's state-N skip eats them at tokenization), so ordinary
source-formatting whitespace is unaffected. Perl comparison: Perl's own
`{verbatim}` lands in `ltx:verbatim` (PCDATA-capable, no trim hook) so
it never faces this; the raw-fancyvrb constructs that do are
UNCONVERTIBLE by same-host Perl (raw fvextra+breaklines exceeded 7 min
on a 6-line file) — surpass-Perl scope, user-directed 2026-07-04.

The same `open_text` gate also keeps an explicit control space `\ `
(a box named `space`), which TeX always typesets. **Except inside SVG**
(batch 56ks): where the current element is an `svg:*` one without
#PCDATA, Perl's rule stands and the whitespace is dropped. pgf pads
every protocol literal with a `\space` (pgfsysprotocol.code.tex:33) that
`\pgfsys@invoke` hands on to digestion, and under `\ttfamily` each one
opened a whitespace-only `svg:foreignObject` (185 in 2605.01517,
2605.07323, 2605.27124; Perl none). A TikZ node text's leading `\ ` or
typewriter space is dropped there too, as in Perl; the rendered node is
unchanged (its foreignObject content is a left-aligned block, where a
leading space collapses). Guard:
`perfect_kernel_batch56::a_typewriter_space_opens_no_foreign_object`.

### 48. Author heuristic splits font-wrapped name lists; affiliation "and" preserved

**Decision:** the superscript-marker author/affiliation heuristic
(`\lx@add@authors`, Base_Utility.pool) gains two beyond-Perl corrections
in the "author" arm (`split_author_line`):

1. **Font-wrapped name lists are split per-author.** When a line
   classified as authors is a single whole-line font wrapper
   `\textbf{A$^1$, B$^1$, C$^1$}`, the separating commas are
   brace-hidden, so `SplitTokens` (which skips delimiters inside `{…}`)
   collapses the wrapper into ONE creator that then hoards every `$^n$`
   marker as a duplicate affiliation. We detect the whole-line wrapper
   (`whole_line_cs_wrapper`), split the inner list, and re-apply the
   wrapper to each name so every author is its own creator with the
   correct single affiliation.
2. **Affiliation names keep their "and".** The literal word " and " is
   removed from the line-level `author_affil_splits` (Perl includes it)
   and applied only in the author arm. That split runs BEFORE
   author/affiliation classification, so on the mixed block it shredded
   institution names — "Princeton Language **and** Intelligence" →
   "Princeton Language" + "Intelligence, …" rejoined without a space.
   Authors written "Alice and Bob" still split, because " and " is a
   name separator inside `split_author_line`. (Mirrors the existing
   `affil_splits` decision to exclude literal "and".)

**Why:** arXiv 2605.00347 (colm2026 class) lists 13 authors across three
`\textbf{…}` lines with `$^{1,2,3,*}$` affiliation markers. Perl and the
pre-fix Rust both lumped the two bold lines into 2 mega-creators, each
carrying 3–5 copies of the "Princeton…" affiliation, and dropped the
"and". Post-fix the assignment exactly matches the PDF: ¹→11 authors,
²→Lu, ³→Yang, \*→the three equal-contributors, one affiliation each.
Perl is broken the same way (confirmed same-host); surpass-Perl scope,
user-directed 2026-07-05. Unit tests: `author_split_tests` in
base_utilities.rs.

### 49. Begin-document hooks digest with the state RE-LOCKED (locked binding macros survive raw redefinition)

**Decision:** In `\begin{document}`'s after-digest (`latex_constructs.rs`), the
begin-document hook lists — `@document@preamble@atend` and `@at@begin@document`
(where `\AtBeginDocument{…}` bodies land) — are digested with the state
**re-locked** (`local_state_unlocked(false)` around each `digest`). So a raw
`\def`/`\let`/`\renewcommand` of a binding-**locked** macro inside
`\AtBeginDocument` is refused, exactly as a preamble-level one already is.

**Why:** A constructor's before/after-digest runs state-**unlocked**
(`definition.rs::execute_after_digest`, a faithful port of Perl
`Primitive.pm::executeAfterDigest`'s `local $UNLOCKED=1`) so bindings can
rebind/load *within their own* before/after methods. That unlock unintentionally
**leaks into the nested raw-TeX digest** of the begin-document hooks: a raw
`\AtBeginDocument{\def\maketitle{…}}` then slips past `\maketitle:locked` and
overrides LaTeXML's semantic `\maketitle`. Because `\title`/`\author` also emit
SEMANTIC frontmatter (`\lx@add@title`/`\lx@add@authors`), the class's *visual*
`\maketitle`/`\@maketitle` reconstruction then renders the title/authors a
**second** time (a duplicate title + author block after the abstract).

**Ground truth** (reproducer `docs/reproducers/frontmatter_maketitle_double.tex`,
an inline pure-`.tex` `\AtBeginDocument{\def\maketitle{\@maketitle}}`):
pdflatex emits the title **once**; Perl AND pre-fix Rust emit it **twice** — a
SHARED LaTeXML bug vs pdflatex. Perl only escapes on acl.sty (arXiv:2606.00012)
because its `\maketitle` lock incidentally holds for a **raw-loaded `.sty`**;
with an inline hook Perl doubles too. (I could not locate the exact Perl
mechanism that discriminates raw-`.sty` from inline under the same structural
unlock, so this is achieved by a Rust-specific relock, not a literal Perl port.)

**Impact / scope:** Post-fix Rust emits the title **once** everywhere. On acl.sty
this MATCHES Perl (LaTeXML's own `\maketitle` runs, so `\ltx@authors@oneline`
fires → `class="ltx_authors_1line"`, identical to Perl); on the inline case it
SURPASSES Perl (Rust 1, Perl 2). The relock is narrow — only these two nested
hook digests, never the general before/after-digest unlock — so binding-internal
rebinding is unaffected. Full suite 1532/0 (no binding pushing a *locked*-macro
rebind through these hooks is disturbed). Root-cause fix chosen over a
frontmatter-only neutralization (user-directed 2026-07-07) precisely because it is
general (protects every locked macro) and more faithful (recovers the Perl class).

### 50. Class bindings establish T1 font encoding where the real class does (`<`/`>` literal, not OT1 `¡`/`¿`)

**Decision:** Class bindings whose real `.cls` establishes T1 font encoding load
`\RequirePackage[T1]{fontenc}` themselves, so those documents digest under the
**T1** font map. Under T1 the ASCII special-char slots — `<` `>` `|` `\` `{` `}`
`_` `"` — map to their **literal** glyphs, as in the PDF. Covered so far
(2026-07-07 audit of the TeX Live `.cls` tree for `\RequirePackage[T1]{fontenc}`
+ true T1-forcing font packages):

| binding | real-class trigger |
|---|---|
| `acmart` | libertine + `\RequirePackage[T1]{fontenc}` (acmart.cls L867-881) |
| `elsarticle` | unconditional `\RequirePackage[T1]{fontenc}` (elsarticle.cls L47) |
| `moderncv` | `\ifpdftex … \RequirePackage[T1]{fontenc}` (moderncv.cls L124-125) |

The audit found these are the only *substantive* bound classes among the 106 TL
classes that set T1 directly (or via libertine); revisit when new class bindings
land. **`memoir` is deferred:** its real class also defaults to T1
(`\memfontenc`=T1 + `\RequirePackage[\memfontenc]{fontenc}` under `\iftutex\else`,
memoir.cls L658/675), but our current `memoir_cls.rs` is only a minimal stub over
`OmniBus`. Rather than bolt T1 onto the stub, memoir wants a proper binding first
(so the encoding lands with the rest of the class semantics, not ahead of them).
Note we deliberately did **not** add T1 to `OmniBus` itself — it is the generic
fallback for *unsupported* classes, many of which are genuinely OT1, so forcing
T1 there would corrupt their `<`/`>`/etc. This divergence is opt-in per class
whose real `.cls` is known to establish T1.

**Why:** These classes really run under T1 in pdflatex (directly via
`\RequirePackage[T1]{fontenc}`, or via a T1-forcing font package like libertine).
LaTeXML's default text font map is **OT1**, where the non-typewriter `<` slot is
`¡` (U+00A1) and `>` is `¿` (U+00BF) — genuinely correct OT1 TeX behavior, but
*wrong* for a T1 class. Neither LaTeXML binding modeled the class's encoding, so
both rendered `num < 0 && num > 0` as `num ¡ 0 && num ¿ 0` (witness
arXiv:2405.17739 under acmart, html_feedback issue).

**Ground truth:** pdflatex (class → T1) renders `<`/`>` **literal**. Perl LaTeXML
AND pre-fix Rust both render `¡`/`¿` — a SHARED LaTeXML limitation vs pdflatex
(verified same-host on acmart: identical `num ¡ 0 && num ¿ 0` from both engines;
Perl's bindings carry zero `fontenc`/`T1` refs for acmart/elsarticle/moderncv).

**Impact / scope:** Post-fix Rust renders literal `<`/`>` (and the other T1 slots)
for documents in these classes, matching the PDF and SURPASSING Perl (Perl stays
at OT1 `¡`/`¿`). Divergence from Perl, per the user's standing rule for the
Rust==Perl-but-wrong-vs-pdflatex pattern (2026-07-07). Blast radius is narrow —
OT1 and T1 agree on all letters/digits/common punctuation; they differ only in the
eight special-char slots above, which T1 makes literal (the faithful class
behavior). Rust already honored an explicit `\usepackage[T1]{fontenc}`; this only
makes the class establish it by default, as the real class does. Verified: full
2405.17739 (0 errors, paper `¡`/`¿` count 1/1→0/0), `acm_aria` + `elsart` fixtures
unchanged, full suite green.

### 51. `\lx@add@frontmatter` is a no-op on empty arguments (no empty frontmatter elements)

**Decision:** `\lx@add@frontmatter [keys]{tag}[attrs]{content}`
(`base_utilities.rs`) early-quits — emitting nothing — when its **tag** or
**content** argument is empty (empty or whitespace-only). A general
defensive principle for the frontmatter API: any add with an empty string is
void.

**Why:** Perl's `\lx@add@frontmatter` (Base_Utility.pool.ltxml L354-358) queues
the entry **unconditionally**, so a binding that funnels an empty argument
through it yields a stray empty element. Concretely, ICML's
`\printAffiliationsAndNotice{}` — empty braces are the *sanctioned* "no notice"
form (icml2026.sty L511-512) — maps to
`\lx@add@frontmatter{ltx:note}[role=affiliationnotice]{#1}` with an empty `#1`,
producing an empty `<ltx:note role="affiliationnotice">` that renders as a bare
"affiliationnotice:" footnote marker (witness arXiv:2606.00309). The affiliation
*list* is unaffected — it is fed separately via `\icmlaffiliation` →
`\lx@add@contact`.

**Scope:** guards the shared primitive once, so **every** frontmatter binding
(icml notice, `\keywords`, `\firstpage`, contacts, …) is covered rather than
patched one `\ifx.#1.` at a time. Divergence from Perl (which would emit the
empty element — a shared latent bug); a beyond-Perl robustness improvement, per
the user's frontmatter-hardening directive (2026-07-07). No legitimate
frontmatter element carries empty content, so nothing real is dropped (full
suite green; witness 2606.00309: empty note count 1→0, affiliations preserved).

**Escape hatch — `\lx@add@frontmatter@container[keys]{tag}[attrs]`:** the one
legitimate empty-content case is a *deliberate* container element that exists
only to anchor later annotations. moderncv opens an empty cv `<ltx:creator>`
so its lazily-added contacts (`\firstname` / `\familyname` / `\email` /
`\mobile` / `\address` / `\homepage`, each annotating the most-recent creator)
have a parent. Perl smuggles this through the same primitive with empty
content — `\lx@add@frontmatter{ltx:creator}[role=cv]{}` (moderncv.cls.ltxml
L27). Rather than exempt one tag from the guard (a per-tag carve-out is the
same code smell moved into the engine), we add an **intention-revealing
container primitive** that queues the entry unconditionally. moderncv (and any
future binding that genuinely needs an empty anchor) calls it explicitly; the
general `\lx@add@frontmatter` guard stays carve-out-free. Both primitives share
the `queue_add_frontmatter_now` lowering helper, so their queueing is
byte-identical.

### 52. Structured author↔affiliation recovery from abused frontmatter idioms

**Decision:** Two beyond-Perl hardenings of `\lx@add@authors` / `\lx@add@thanks`
(`base_utilities.rs`) that recover *structured* author/affiliation metadata from
two idioms arXiv authors routinely abuse, where both Perl LaTeXML and a literal
port emit garbage. Both are **surpass-Perl divergences** (same-host Perl
reproduces the bad output — witnesses below), authorized under the
PDF-fidelity/beyond-Perl policy.

**(a) `\thanks`-abuse → affiliations.** `\thanks` is semantically an
acknowledgement footnote, but authors smuggle affiliations into it, linked to
authors by a leading superscript mark:
`\thanks{$^{1}$Univ. Bordeaux… $^{2}$School… $^{3}$Instituto…}`. Because
`\lx@add@thanks` reads its content **Semiverbatim** (faithful to Perl
`Base_Utility.pool.ltxml:661` — it protects `~ # % &` in URL/email-bearing
notes), `$^{1}$` freezes to catcode-*other* and surfaces as a literal `$^1$` in
one opaque `role="thanks"` blob (witness arXiv:2606.00313). The fix keys off a
**leading NUMERIC superscript mark as the abuse signature**
(`starts_with_affiliation_mark` — content begins with `$^1…`/`$^{1}…`/
`\textsuperscript{1}…`): affiliation linking is by *number*, so a digit mark is
the reliable signal. Crucially this **excludes footnote-symbol marks**
(`$^*$`, `$^\dagger$`, `$^\ddagger$`, `$^\S$`) and lettered marks — those head
*legitimate* acknowledgements (corresponding-author, equal-contribution,
present-address notes) that must stay `role=thanks`; re-routing them would
create an affiliation that fails to link and could be discarded, silently losing
the note (an early "any superscript" heuristic did exactly this — caught in
review). When the numeric signature fires we re-tokenize with normal catcodes,
split the blob at each embedded mark (`split_before_affiliation_marks` — the
marks, not `\\`, delimit the entries; a mark is only a boundary when preceded by
whitespace, so a superscript *inside* an institution name like "Center for
R$^2$ Studies" does not split it), and feed each segment through the existing
`\lx@affiliation@withsup` machinery, which sets the `affiliation:N` label that
the authors' own marks *already* request (`relocate_annotations` then links
author↔affiliation). Every non-numeric-mark `\thanks` stays the parity-faithful
Semiverbatim contact, byte-identical. This detects the abuse without a class
allow-list — the numeric mark generalizes across `ieeeconf`, generic `article`,
springer, etc.

**(b) NeurIPS comma-address → fake authors/emails.** The no-marker author
heuristic split author *groups* on `author_splits()`, which (like Perl's
`@authorsplits`, `Base_Utility.pool.ltxml:679`) **includes the comma**. A
multi-part address then shreds at its commas into fake authors —
`\author{… Nam Q. Le \\ Johns Hopkins…, Laurel, MD 20723 \\ \texttt{…@jhuapl.edu} \and …}`
produced personnames "Laurel", "MD 20723" and mislabeled the email as an
affiliation (witness arXiv:2606.00315). The fix splits groups on the `\and`
family / `\quad` only (`author_group_splits()` — comma excluded); within a group
the first `\\`-line is the name list (comma/" and "-split by `split_author_line`,
so "Alice, Bob" still separates) and each remaining `\\`-line is an affiliation
attached to the group's last author. A bare `user@host` line (visible text has
`@` and no whitespace — institution names always have spaces; `line_is_email`)
is relabeled `role="email"`. Simple shapes (`A \and B`, `A, B, C`, `A, B \\ MIT`)
are unchanged; an empty `\\`-leading name list keeps a single empty author so its
affiliations are not dropped. `author_affil_splits()` already carried the comment
"NO comma in affiliations!!!"; this extends that discipline to the no-marker arm.

**(c) Short author names in a marker-labeled line.** The marker-labeled arm
classifies each `\quad`/`\\`-split line as author vs affiliation by where its
superscript sits. The original proxy — a marker within the first 8 tokens ⇒
affiliation ("¹CMU") — misread a **short author name** whose trailing mark landed
under the threshold: "Min Xu" is 7 tokens, so `Min Xu\textsuperscript{1}` was
demoted to an affiliation and the author lost (witness arXiv:2606.08234,
html_feedback#6614). Replaced by the length-independent signal
`name_precedes_marker`: a line reads "Name\textsuperscript{n}" (letter text
before the marker ⇒ author, split into creators) or "\textsuperscript{n}Affil"
(the marker leads the line ⇒ affiliation). Here Rust surpasses Perl, which keeps
the four authors but ALSO emits the affiliation line itself as phantom author
creators.

**(d) Multi-line author block with a trailing `\quad\\`.** In the no-marker arm,
author *groups* split on `\quad`/`\and` and each group then splits on `\\` into a
name line + trailing affiliations. When a line ends with `\quad \\` (a common
NeurIPS/ACL idiom for wrapping a long author list), the `\\` leaks to the HEAD of
the next `\quad`-group, so that group's first `\\`-piece is empty and its real
first author was demoted to an affiliation under an empty `<personname/>`. Fixed by
dropping empty `\\`-pieces before choosing the name line: the first NON-empty piece
is the names, the rest affiliations. Witness arXiv:2507.06670 (acl): line 2's first
author "Ruiqi Li" was an empty personname + a bogus "Ruiqi Li" affiliation; now an
author. Same-host Perl 0.8.8 mangles it identically (SHARED-FAILURE) — recorded as
KNOWN_PERL_ERRORS #91.
**(e) Phantom empty creators from comma-split `\IEEEmembership`/`\thanks`.** A flat
comma author list with interspersed non-name commands — `\author{Alice,
\IEEEmembership{…}, and Bob, …\thanks{…}}` (html_feedback#4539, witness 2508.00603) —
comma/" and "-splits into pieces where the `\IEEEmembership{…}` pieces digest to
nothing, surfacing as empty `<ltx:personname/>` creators; a trailing `\thanks` then
strands its affiliation/email on a nameless creator, and the reader sees a stray "`,
,`" between authors. `insert_frontmatter`'s `coalesce_empty_creators` drops
name-empty author creators, moving any contacts they carry to the preceding real
author (a `\footnotemark`-note keeps a personname non-empty, so real authors with a
marker — 2507.06670 "Yu Zhang" — are untouched). Same-host Perl 0.8.8 emits the same
empty creators (SHARED-FAILURE); Rust surpasses. Guard:
`06_cluster_frontmatter::frontmatter_ieee_membership_no_phantom`.

**(f) Multiple `\textsuperscript{n}Affil` on one space-separated affiliation line.** A
marker-led affiliation line may carry several numbered institutions with only spaces
between them — `\textsuperscript{1}University A \textsuperscript{2}University B`
(html_feedback#6242, arXiv:2510.02340). The marker-branch classified the whole line as
ONE affiliation, merging both (and attaching them to a single author). Now the line is
split at each whitespace-preceded mark (reusing `split_before_affiliation_marks`, the
`\thanks`-abuse splitter — which by construction never breaks a superscript glued
INSIDE an institution name, e.g. "Center for R$^2$ Studies"), so each numbered
institution becomes its own affiliation and `relocate_annotations` attaches it to the
authors bearing that number. Perl produces no structured creators for this superscript
idiom at all, so Rust already surpasses; this refines the surpass. Guard:
`06_cluster_frontmatter::frontmatter_multi_affil_superscript`. (Residual, separate: when
a `\texttt{…}` email is glued to the last affil with no `\\`, it still bleeds into that
affiliation — a distinct email-boundary defect.)

**(g) `\and` is a HARD author boundary in the superscript-marker branch.** The
marker-branch flat-split `\and`/`\quad`/`\\` into one list, then appended any
marker-less line to the PREVIOUS entry. When a 2nd/3rd author's superscript is
macro-delivered — `Alice\mk$^*$ \and Bob\mk \and Carol\mk`, where `\mk` expands to a
`$^…$` marker so `Bob\mk`/`Carol\mk` carry no *literal* `^` — those segments merged
back across the `\and` into one `<personname>AliceBobCarol</personname>`
(html_feedback#1021 F2 residual, arXiv:2403.11905). Now the branch groups on the `\and`
family FIRST and only appends a marker-less line to an entry created WITHIN the same
`\and` group, so `\and`-separated authors never merge. Groups carry no `\and`, so the
intra-group split (reusing `author_affil_splits`) is equivalent to splitting on
`\quad`/`\\`; the *only* behavior change is that a marker-less first line of a non-first
`\and` group becomes a new author instead of merging — verified by an OLD-vs-NEW full-XML
diff over all 30 frontmatter fixtures, exactly ONE (this case) changed. Guard:
`06_cluster_frontmatter::frontmatter_and_hard_author_boundary`. (Residual, separate —
deferred as Phase 2: a marker delivered *purely* by macro with no literal `^` anywhere on
its line, e.g. `\handPointerZ Johns Hopkins University`, is still appended to the author
name rather than recognized as an affiliation, because classification keys on a literal
marker and does not expand macros. So arXiv:2403.11905's Kate Sanders now separates from
Kevin Xu but still carries her institution in the name; full attachment needs
expansion-aware marker detection.)
**(h) authblk `\author{A, B, C}` comma list splits into separate creators.** authblk's
`\author` routes a no-`\and` argument to `\lx@add@creator` (a single creator), so a
comma list `\author{Alice One, Bob Two, Carol Three}` was kept as ONE
`<personname>Alice One, Bob Two, Carol Three</personname>` (html_feedback#6255,
googledeepmind). The DEFAULT `\author` already routes to `\lx@add@authors`, which splits
a comma / " and " list into separate creators via `split_author_line`. authblk's
`\author` now routes to `\lx@add@authors` too when the argument has a top-level comma (or
`\and`), so the comma list resolves to individual creators — matching the default and the
PDF. A single name that legitimately contains a comma ("Smith, Jr.") is mis-split, but
that is the pre-existing #52 comma tradeoff the default `\author` already makes, not a new
divergence; the label and single-name (no-comma) authblk paths stay Perl-faithful. Guard:
`06_cluster_frontmatter::frontmatter_authblk_comma_list`.

**(i) `\hspace` separates co-authors; footnote-symbol marks render instead of vanishing.**
Two composable normalizations applied to the `\author` argument BEFORE branch selection
(`normalize_hspace_separators` + `rewrite_symbol_superscripts`), addressing
html_feedback#6637 (arXiv:2506.06941, "The Illusion of Thinking", plain `article`), where
the six authors were welded into one `<personname>` with "Apple" glued on and Iman
Mirzadeh's literal `$^{*}$` silently dropped — Perl 0.8.8 (the arXiv production engine)
produced byte-identical output, so this is a surpass, not a Rust-only fix.
- **`\hspace{len}` / `\hspace*{len}` / `\hfill` → `\quad`.** Authors laid out with
  `A \hspace{1cm} B \hspace{1cm} C` (a regular poor-man's separator) collapsed into one
  creator, because the splitter only knew `\and`/`\quad`. `\quad` is already a hard
  separator in every author/affiliation split set, so the rewrite needs no new plumbing;
  `\hspace`'s optional `*` and mandatory length are consumed so the length cannot leak.
- **Footnote-SYMBOL superscripts (`$^{*}$`, `${}^{\dagger}$`, `\textsuperscript{\ddagger}`)
  → a visible `\lx@frontmatter@keepsup` sup.** These are equal-contribution / corresponding
  notes, NEVER affiliation numbers (the same note-vs-affiliation split already encoded in
  `starts_with_affiliation_mark`). Rewriting them onto a sentinel the `\lx@author@withsup`
  hijack does not touch means they (1) render as a real superscript instead of being consumed
  into an `affiliation:*` label that matches nothing and is discarded, and (2) no longer count
  as an affiliation-marker trigger, so a block whose ONLY superscript is such a note-mark takes
  the clean no-marker branch. NUMERIC/lettered affiliation marks (`$^{1}$`) are untouched, and
  a superscript with a non-empty base (`$x^2$`) is real math, left alone.
- **Combined effect on the witness:** removing Mirzadeh's `$^{*}$` (its only literal `^`)
  drops the whole block into the no-marker branch, where the `\quad`s split all six authors,
  "Apple" becomes the last author's affiliation, and Shojaee keeps his two `\thanks`. Verified
  by an OLD-vs-NEW full-XML diff over all 57 author-bearing fixtures: exactly the THREE new
  fixtures changed, zero existing regressions. Guards:
  `06_cluster_frontmatter::{frontmatter_hspace_author_split, frontmatter_symbol_superscript_mark,
  frontmatter_thanks_literal_mark_mix}`. (This is also why (g)'s guard fixture now uses a
  numeric `$^{1}$` marker: a symbol `$^{*}$` there is now a note-mark and would reroute out of
  the marker branch it is meant to exercise.)
- **Residual (out of scope):** the *exact* `\fnsymbol` marks the author intended next to
  Shojaee's name (∗ for footnote 1, † for footnote 2) are not reconstructed — `\thanks`
  becomes a `role=thanks` contact note, not a numbered superscript. And an author block that
  mixes marked and UNMARKED authors while still carrying a NUMERIC affiliation mark stays in
  the marker branch, where marker-less horizontally-separated co-authors can still append; the
  clean recovery above depends on the note-marks being the *only* superscripts.

**(j) A shared author-email line distributes per author instead of bunching on the last.**
`\email{a@x, b@y, c@z}` (or a single `\email` covering several authors) attached every address to
whichever creator was open when the line digested — usually the LAST — leaving the rest
email-less. The `Email` author-line branch now resolves the individual addresses and, when there
are no more than one per author, hands each to `\lx@add@email` with `labelseq=author`, so address
*i* relocates to creator *i*'s own `author:N` label:
- **Distributed (`a@x, b@y, c@z`)** → email *i* to author *i*, in declaration order.
- **Grouped brace-expansion (`{a,b,c}@dom`)** → expand to `a@dom, b@dom, c@dom` first, then
  distribute; the expansion is never glued into an affiliation label.
- **A single shared address** → `author:1`, the LEAD author, never a trailing one.
A whole-line `\texttt`/`\url` wrapper is re-applied per address. MORE addresses than authors
cannot map cleanly, so the original line is kept as one contact (the prior behavior). Witness
arXiv:2605.23553 (`frontmatter_ieee_linebreak_optarg`). Guard
`06_cluster_frontmatter::frontmatter_shared_email_distribution` (fixtures
`frontmatter_email_{distributed,grouped,single_shared}.tex`).

**Scope/limits:**
- The `*` equal-contribution suffix on a combined author mark (`$^{1*}$`) still
  labels `affiliation:1*`, so it does not yet match a plain `affiliation:1`
  (2606.00313's first two authors stay unlinked — strictly better than the
  former literal blob, no regression).
- Dropping the comma from group-level splitting makes one previously-handled
  shape worse: `\author{Alice\\MIT, Bob\\CMU}` (comma separating two
  author+affiliation groups, where the standard idiom is `\and`) now yields a
  single author *Alice* with affiliations "MIT, Bob" and "CMU" — *Bob* is folded
  into affiliation text and lost. This is the unavoidable dual of fixing the
  common address case: "MIT, Bob" (affil, author) and "…Laboratory, Laurel"
  (address parts) are structurally identical, so no heuristic separates them.
  Multi-author docs overwhelmingly use `\and`, so this is rare; the frequent
  address-shredding it trades against is the right call.
- A numeric abuse-mark note that fails to link to any author is shown as a
  `role=affiliation` contact attached to the last creator (mislabeled, but not
  lost). Email relabeling is applied in the no-marker author arm only.

Full suite green (1532/0); clippy clean.

### 53. `inst`-style `\author[marks]{name}` accepts the optional marks and accumulates

**Decision:** `inst_support.sty`'s `\author` (used by classes following the
`\inst` institution convention — the fallthrough for a raw-loaded `ifacconf.cls`
via OmniBus, and historically aa/llncs/sv) is redefined from Perl's
`DefMacro('\author{}', …)` to `DefMacro('\author[]{}', …)`. This is a
**surpass-Perl divergence** authorized under the PDF-fidelity policy: same-host
Perl reproduces the bad output.

**The shared bug.** Perl `inst_support.sty.ltxml:33` documents `\author[marks]{author}`
in its own comment but defines a **single-argument** `\author{}` whose body is
`\lx@clear@creators[role=author]\lx@splitting{\lx@add@author}{\and\And,}{#1}`.
A class that calls `\author` **once per author with a label** —
ifacconf's `\author[First]{Eryn Vaid}` (four such calls) — then (a) reads the
literal `[` of `[First]` as the single mandatory argument, so the personname
becomes `[`, and (b) `\lx@clear@creators` wipes the prior author on every call,
so only the last survives. Result: one `<ltx:personname>[</ltx:personname>`.
Perl and Rust emit the identical garbage. Witness arXiv:2605.00004, whose
pdflatex PDF lists all four authors (Vaid, Chiri, Guglielmi, Notomista).

**The fix** (`inst_support_sty.rs`): accept the optional `[marks]` (so `[` is
never mistaken for the name), take the name from `#2`, and **drop the per-call
`\lx@clear@creators`** so successive `\author` calls **accumulate**. Dropping the
clear is safe: it is a no-op on the first `\author` call, so single-`\author`
classes are unaffected — and aa/llncs/sv define their own `\author` regardless.
The `[marks]` (the author↔affiliation label) are dropped exactly as Perl's
single-arg form dropped them; wiring them to the affiliation annotation (à la
`\inst`) is a separate follow-up. Verified: 2605.00004 now yields all four
personnames; full suite 1532/0.

---

### 54. `eqnarray` keeps distinctly-`\label`-ed continuation rows separately numbered

**Decision:** `rearrange_eqnarray` (`latexml_engine/src/latex_constructs.rs`
L1085) reads the real **plural `labels`** attribute when deciding whether a
continuation row is "labelled". Perl's `rearrangeEqnarray` checks
`hasAttribute('label')` (**singular**) — an attribute LaTeXML never sets
(`LaTeXML-common.rnc` L134 defines only `labels`) — so its own documented
safeguard *"Separately numbered AND labeled? … must keep separate"* is dead code.
This is a **surpass-Perl divergence** under the PDF-fidelity policy that honors
the Perl author's stated intent.

**The shared bug.** An `eqnarray` (or any environment mapped onto it, e.g.
IEEEeqnarray) merges continuation rows — empty first *and* second column — into
the previous equation. When several such rows each carry their **own** automatic
number **and** their own `\label`, they should stay separate; the safeguard that
would keep them separate never fires because of the `label`/`labels` typo, so they
collapse onto one number and the middle labels pile onto the last row's `labels`
attribute (rendering no number). Witness arXiv Problem-𝒫1 (`ieee_eqn_bug`): four
constraint rows render `(28a),(28d)` instead of `(28a),(28b),(28c),(28d)`;
pdfTeX numbers all four. Perl and Rust-before emit the identical collapse.

**The fix** reads `labels` (not `label`), so the R-column classifier's
`numbered && row.numbered && row.labelled` → keep-separate branch fires as
intended. Strictly monotone — it can only *split* a merged equation whose row
was both numbered and `\label`-ed, never merge — so `\nonumber` continuations and
unlabelled multi-line RHSs are untouched (`subnumcases`/`ncases` builds its own
alignment and is unaffected). Regression fixture
`latexml_oxide/tests/structure/eqnarray_labelled_rows.tex`; full record in
[`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) #46. Verified: 𝒫1 now numbers all
four; full suite 1541/0.

---

### 55. Quoted `\graphicspath` directories are unquoted before lookup

**Decision:** the `\graphicspath` constructor
(`latexml_package/src/package/graphics_sty.rs` L459) strips a surrounding pair
of double-quotes from each directory entry before it is made absolute and pushed
onto `GRAPHICSPATHS`, and `image_candidates`
(`latexml_core/src/util/image.rs` L76) strips them defensively at the
consumption site too. This is a **surpass-Perl divergence** under the
PDF-fidelity policy: pdflatex/kpathsea both tolerate quoted paths, so a document
that renders under pdflatex keeps its figures under LaTeXML.

**The shared bug.** A `\graphicspath{{"./figures"}}` — the MiKTeX/Windows idiom
where the quotes guard embedded spaces — is accepted by pdflatex, which strips
the quotes before any filesystem lookup. Perl LaTeXML's `DirectoryList`
parameter type keeps the literal quotes (it strips them only later, and only for
`\special{psfile="…"}` in `\lx@special@graphics`, never for `\graphicspath`), so
the stored search directory becomes `<sourcedir>/"./figures"` — a path that can
never match the real `figures/` directory. Every `\includegraphics` then fails
to resolve and emits `Warning:expected:source`, and the HTML carries an empty
`<img src="" class="ltx_graphics ltx_missing_image">`. Perl and Rust-before emit
the identical loss. Witness: arXiv **2606.22880** ("DJM: Compact Base Meshes for
Displacement Mapping", acmart) declares `\graphicspath{{"./figures"}}` and loses
**all 8** of its `\includegraphics` figures under both engines.

**The fix** removes the surrounding quotes (`trim_matches('"')`), mirroring the
quote strip already applied to the `\includegraphics` FILENAME side
(`image.rs:53`). Strictly monotone — it can only *resolve* an image that a quoted
path had hidden, never hide one: an unquoted directory contains no leading/
trailing `"` for `trim_matches` to remove, and a real directory path never
legitimately begins or ends with a double-quote. Covers quoted directories from
`\graphicspath`, `\svgpath`, and the `--graphicspaths` CLI option alike. All 8
figures in the witness now resolve.

---

### 56. acmart `teaserfigure` is relocated to the top-matter position

**Decision:** the acmart `teaserfigure` environment
(`latexml_package/src/package/acmart_cls.rs`) is digested and constructed **in
place** as a normal `<ltx:figure class="ltx_teaserfigure">`, then a
`DOCUMENT_REWRITE` rule moves the finished node to immediately **before the
abstract** — matching acmart's PDF top-matter order (title, authors, teaser,
abstract). Perl LaTeXML has **no** `teaserfigure` binding at all, so this is a
beyond-Perl behavior throughout.

**The shared bug.** Real `acmart.cls` defers the teaser:
`\newenvironment{teaserfigure}{\Collect@Body\@saveteaser}{}` (cls L2202) stashes
the body into `\@teaserfigures`, and `\maketitle` renders it via `\@mkteasers`
(cls L2240, L2899) as the last part of the top-matter box — so the teaser always
appears after the title+authors regardless of where the environment is written.
Papers write `\begin{teaserfigure}…\end{teaserfigure}` **before** `\maketitle`
(it is declared next to `\title`/`\author`). LaTeXML digests the environment at
its source position, so the emitted `<ltx:figure>` became the **first**
`<document>` child — ahead of the title. Witness arXiv **2606.22880** ("DJM:
Compact Base Meshes…", acmart): the teaser rendered at the very top of the HTML,
before the paper title, and is `\ref`-ed 6+ times ("Fig~\ref{fig:teaser}d").

**Why construct-then-relocate (not defer-digestion).** The figure must own its
`\label` so the 6+ `\ref{fig:teaser}` resolve to "Figure 1"; a label only
attaches to the enclosing float **while that float is the open element during
digestion**. Deferring the *digestion* to `\maketitle` (e.g. via the frontmatter
`…@until` hook) digests the body as detached content, stranding the label on
`<document>` and breaking every reference. So the float is built normally (label,
caption number, `xml:id`, `inlist` all correct) and only its **position** is
changed, post-construction.

**The relocation** is a `DefRewrite` anchored on the **abstract**, not the
teaser: the rewrite `replace` engine unbinds the matched node *and every
following sibling*, so matching the teaser (the first child) would detach the
whole frontmatter. Anchoring on the abstract keeps the teaser (a *preceding*
sibling) bound, and the still-bound teaser is moved to just before the
re-attached abstract. The xpath predicate
`//ltx:abstract[//ltx:figure[contains(@class,'ltx_teaserfigure')]]` gates the
rule to teaser-bearing documents, so a plain acmart abstract is untouched.
Verified: 2606.22880's teaser now renders between the authors and the abstract,
its `\ref`s read "Figure 1", and a teaser-free acmart document is unchanged.

---

### 57. amsrefs inline bibliographies are collected (upstream drops them whole)

**Decision:** `MakeBibliography::get_bib_entries`
(`latexml_post/src/make_bibliography.rs`) scans the **main document** for inline
`ltx:bibentry` elements in addition to the external bibliography documents
returned by `get_bibliographies`. Perl's `getBibEntries`
(`LaTeXML/lib/LaTeXML/Post/MakeBibliography.pm`) only ever iterates
`getBibliographies($doc)`. This is a **surpass-Perl divergence** under the
PDF-fidelity policy: the references are unambiguously present in the source and
in the author's PDF.

**The shared bug.** `amsrefs` writes the bibliography *into the document* —

```latex
\begin{bibdiv}\begin{biblist}
\bib{Bei87}{article}{ author={Be\u{\i}linson, A.}, title={Height pairing...}, }
\end{biblist}\end{bibdiv}
```

— rather than into an external `.bib`. The engine digests this correctly into
`ltx:biblist`/`ltx:bibentry` (our `amsrefs_basic` structure test covers exactly
that, and passes). But there is no `@files` attribute for `getBibliographies` to
resolve, so it returns an empty list, `getBibEntries` collects **nothing**, and
`process` then executes its unconditional
`$doc->removeNodes($doc->findnodes('//ltx:bibentry'))` — deleting every entry it
never collected. The result is a **silently empty References section with every
`\cite` left dangling, and zero errors reported**.

Confirmed identical on the installed **and** the vendored Perl 0.8.8
(rev `51fea96a`): witness 2605.01646 (`AIPFa.tex`) gives Perl `ltx_bibitem: 0` /
`ltx_missing_citation: 81`. Recorded upstream as KNOWN_PERL_ERRORS #49.

**Why this is safe.** A paper with an external `.bib` carries no inline
`ltx:bibentry` in the main document at this point in the pipeline, so the extra
scan contributes nothing and the entry map is byte-identical. A biblatex `.bbl`
is read into inline bibentries since batch 56jc (#306), and those are exactly the
entries the bibliography formats. The scan runs
*after* the external documents, so a key defined both externally and inline
resolves to the inline one — matching upstream's own last-source-wins loop.

**Measured.** All 40 amsrefs papers in sandboxes 2605+2606 went from 0 rendered
references (100% loss, every citation dangling) to **1,482 references rendered
with zero dangling citations**. Witness 2605.01646 (23 entries), 2605.00783,
2605.03852.


### 58. A malformed `.bib` entry resyncs at the next `@` (upstream abandons the file)

**Decision:** `PreBibTeX::parse_top_level` (`latexml_engine/src/pre_bibtex.rs`)
reports a malformed entry and **continues at the next `@`**. Perl's
`parseTopLevel` lets the first parse error propagate out, abandoning every
LATER entry.

**Why.** Real BibTeX does not abandon the file: on *"I was expecting a `,' or a
`}'"* it reports the error and skips to the next entry (`bibtex.web`), which is
the behaviour authors' `.bib`/`.bbl` files are written against — a single
unbalanced `{` costs its own entry, not the rest of the bibliography. Under
Perl's rule one stray brace silently deletes the whole tail of the References.
`skip_junk` already *is* the resync, so the loop simply keeps going; the
malformed entry itself is dropped, exactly as BibTeX drops it.

**Loud, never silent.** Each resync emits
`Warning:bibtex:unbalanced <label> line N: <error>; resyncing at the next '@'`,
so the lost entry is always visible in the log (CLAUDE.md's
fail-safe-toward-flagging-failure rule). The corpus carries 19 papers /
298 messages in this category.

**History.** This robustness previously lived in a bespoke second BibTeX parser
inside `latexml_post::make_bibliography`, which has been deleted in favour of
the faithful `pre_bibtex` port; the resync moved here so the single shared
parser keeps both the faithful grammar and the BibTeX-grade error recovery.

### 59. A citation also searches the main `bibliography` list, not just its bibunit

**Decision:** `CrossRef::make_bibcite` (`latexml_post/src/crossref.rs`, called by `fill_in_bibrefs`) searches
the bibref's `inlist` units **and then the main `bibliography` list**. Perl
`CrossRef.pm` L515 reads `inlist || 'bibliography'` — an *exclusive* choice that
searches the unit list alone whenever `inlist` is set.

**Why.** `bibunits`/`chapterbib` stamp `CITE_UNIT` onto every `\cite` (bibunits'
`\lx@bibunits@resetglobal`, `bibunits.sty.ltxml` L39-41), so a bibref carries
`inlist='bu0'` **merely because the package is loaded** — even when the document
never opens a `bibunit` environment and has exactly one ordinary
`\bibliography`. That bibliography registers its bibitems under the default
`bibliography` list (`MakeBibliography`, mirroring `Scan.pm` L465), so the
unit-only lookup can never match and **every** citation dangles.

Perl's own `Scan.pm` L379-380 spells the intended chain — the unit lists **plus**
the main one, commented *"Citation specifies main 'bibliography', as well as any
specific others (eg. per chapter)"* — and registers the reference under both. So
upstream already disagrees with itself: Scan records two lists, CrossRef reads
one. We follow Scan's convention in both places.

**Specificity is preserved.** The unit lists are searched first and the scan
breaks on the first list yielding an `id`, so a genuine per-chapter bibliography
still wins over the global one; the main list is only ever a fallback.

**PARITY with same-host Perl** — fixed here rather than reproduced
(KNOWN_PERL_ERRORS #50). Witness 2303.06077 (revtex4-2 + `bibunits`): 93
bibitems rendered, 93 keys dangling, 0 links → now 93 / 0 / 179 links. The
minimal reproducer is 6 lines (`tests/cluster_regressions/bibunits_cite.tex`):
deleting the single `\usepackage{bibunits}` line resolves the cite. Perl on that
same reproducer: 1 bibitem, 1 dangling, 0 links.

### 60. `.bib` scanning follows BibTeX, not `Text::Balanced` (escaped braces, non-ASCII keys, bare `@Comment`)

**Decision:** `pre_bibtex` parses `.bib` the way **BibTeX itself** does on three
points where Perl's `Pre/BibTeX.pm` diverges from the real tool:

1. **Braces count literally.** `\{` / `\}` do NOT escape the brace depth
   (`find_balanced_brace_end`). Perl uses
   `Text::Balanced::extract_bracketed($line, '{}')`, which treats them as
   escaped, returns `undef` for a title like
   `"…{\textbackslash}boldsymbol\{Q\}…"`, then extends line-by-line to EOF and
   abandons the file.
2. **Non-ASCII is a name character** (`is_bib_name_or_noise`). Perl's class is
   the literal `a-zA-Z0-9` (L221), so a Zotero-style key
   `alvarado-leañosLasing2022` truncates at the accent → `Expected ","`.
   Non-ASCII is never a BibTeX delimiter, so admitting it cannot swallow
   structure.
3. **A bare `@Comment` banner is not an error** (`parse_comment`). BibTeX
   ignores everything after `@comment`; Perl demands a delimited string.

**Why — ground truth is the real tool.** All three inputs are accepted by
`bibtex` 0.99d (TeX Live 2025) with at most a benign *"empty journal"* warning,
so the references exist in the author's PDF. Perl LaTeXML loses them: on the
escaped-brace reproducer it emits **0 bibitems and 2 dangling citations**
(it abandons the whole file), where `bibtex` emits both entries. This is the
authorized surpass-Perl case — Rust == Perl but both wrong vs the PDF.

**Measured.** These were exposed by routing post-side `.bib` parsing through
this port (#58): `bibtex/unbalanced` went 19 → 593 papers in sandbox 2605
because the deleted bespoke parser had been permissive on exactly these points.
On a 24-paper sample of the affected set: **0/24 clean → 22/24 clean**
(brace fix alone: 7/24). Witness 2605.00264 (`\{Q\}` in `chen2017ucb`):
1144 of the file's 1170 entries parsed → **all 1170**, 18 dangling
citations → **0**.
Witnesses 2605.28695 (`ñ` key), 2605.00121 (stray U+FE0F in the key),
2605.06974 (26 `@Comment` banners).

4. **`\` is a name character.** Perl excludes it *on purpose* (L215-217:
   *"Especially `\`, which BibTeX allows, but it throws us off (semiverbatim vs
   verbatim) when we store the bibentries before digesting the key!"*) — but
   excluding it does not dodge the hazard, it just loses the entry: the key in
   `@misc{apple\_rl,` ends at the backslash, and a bogus `\author={...}` field
   name kills its entry outright. BibTeX takes `apple\_rl` as the key verbatim
   and treats `\author` as an *unknown field* (hence its "empty author"
   warning), keeping the entry. Witnesses 2605.14212, 2605.06974.

**Known limit of (4) — Perl's warning is accurate downstream.** Admitting `\`
makes the entry *parse*, and the `\author=` case then resolves end-to-end. But a
`\cite{apple\_rl}` is **digested** to `bibrefs="apple_rl"` while the entry keeps
the verbatim key `apple\_rl`, so that citation still dangles
(`Missing bibkeys: apple_rl`). This is strictly better than dropping the entry,
but making such a cite *link* needs key normalisation at the
`\ProcessBibTeXEntry` seam — not done here.

**Residual (not fixed):** a small tail of malformed-entry shapes still warns
`bibtex:unbalanced`; they lose no cited keys — the #58 resync recovers the next
entry — so it is log noise, not data loss.

### 61. `\end{lstlisting}` terminates the listing anywhere on the line, not only at its start

Perl `listings.sty.ltxml` L316 (`listingsReadRawLines`) anchors the terminator:

```perl
if ($line =~ /^\s*\\end\{\Q$environment\E\}(.*?)$/) {
```

so a line that carries content *before* the terminator —
`</body></html> \end{lstlisting}` — never matches. The reader then consumes every
remaining line, `\end{document}` included, and the document simply ends where the
input does. Nothing is reported: the environment is not "unterminated" from the
reader's point of view, it just ran out of file. **The entire tail of the paper is
lost with zero `Error:`.**

Real `listings` terminates there. Ground truth (pdflatex on the minimal repro
`hello world \end{lstlisting}`): compiles cleanly, renders `hello world` as the
listing's last line, and typesets the following text normally.

We therefore search for `\end{<env>}` **anywhere** in the line: text before it
becomes the listing's final line (whitespace-only before → no line, preserving
the ordinary terminator-on-its-own-line case), text after it is unread — which is
what Perl already does for the trailing part.

This is a **shared upstream bug**, not a Rust regression: same-host Perl loses the
tail identically and reports `Conversion complete: No obvious problems`. See
KNOWN_PERL_ERRORS #51 — candidate to upstream.

Witness **2605.11619**: a complete 54 KB paper silently lost its Conclusion,
`\bibliography` and appendix (1.3 MB of HTML, 0 errors, 0 references). After:
Conclusion + appendix restored and **32 references, 0 dangling**. Breadth: 7 of
the 169 truncated papers in the 2026-07-14 empty-References sweep, 3 of them in
the silent (no-`Error:`) subset. Regression test:
`06_cluster_regressions::inline_end_lstlisting_does_not_swallow_the_document`.

### 62. The biblatex binding announces itself as `Info`, not an unconditional `Warn`

**Decision:** `latexml_contrib/src/biblatex_sty.rs` opens with

```rust
Info!("bibliography", "biblatex",
  "biblatex.sty is provided by a native binding, not interpreted raw.");
```

where the ar5iv-bindings Perl original emits a **`Warn`** under `missing_file`:
*"biblatex.sty is only minimally stubbed and will not be interpreted raw"*.

**Why.** Both halves of the Perl message had become false. Nothing is *missing* —
`biblatex.sty` is deliberately not raw-loaded precisely because this binding
stands in for it; and "minimally stubbed" stopped being true once the binding
grew author-year cite families, biber `.bbl` handling, `\printbibliography`,
`maxbibnames` and structured `\name` parts. It also fired **unconditionally at
load**, so it carried no information about the paper in hand. A biblatex feature
we actually get wrong reports itself through its own error, where it happens.

**Severity impact — read this before quoting corpus deltas.** This warning was the
**#1 `missing_file` "what" in the corpus: 1,167 papers** across sandboxes
2605+2606 (second only to arydshln). Because cortex ranks a task by its worst
message, the unconditional `Warn` **downgraded every biblatex paper from
`no_problem` to `warning`** regardless of how well it converted. Retiring it
therefore moves a population of papers into `no_problem` **for a logging reason,
not a conversion improvement**.

Any `no_problem` delta measured across this change is **confounded**: it mixes
recovered bibliographies with papers that were always clean but mis-ranked. A
run-over-run comparison is only attributable to engine fixes if **both** runs sit
on the same side of this commit (`14dd3345be`, 2026-07-14). The 2026-07-14
`sandbox-13` rerun is the first on the `Info` side; comparisons against it must
either be from another `Info`-side run or state the confound explicitly.

**Not a severity-downgrade cheat.** CLAUDE.md forbids emitting fewer `Error:`s
than Perl to flatter a signal. This is a `Warning` → `Info` on a *load-time
banner* that describes the binding rather than the document; no per-paper
diagnostic is suppressed, and the genuinely useful fact (this is an
approximation, not the real package) is retained under an accurate category.

### 63. A subimported child's `\documentclass` options are class options, not a package list

Perl `standalone.sty.ltxml` L24-33 intercepts a sub-document's `\documentclass`
and `RequirePackage`s its **optional** argument, comma-split:

```perl
DefPrimitive('\@standalone@documentclass[]{}', sub {
    my ($stomach, $packages) = @_;          # $packages = the OPTIONAL [] arg
    for my $package (split(",", ToString($packages))) { RequirePackage($package); }
```

But that argument holds **class options**. `\documentclass[12pt]{article}` in a
`\subimport`ed child therefore requires `12pt.sty` and warns
`missing_file:12pt` (issue #309); `\documentclass[border=2pt]{standalone}` — the
most common standalone idiom — requires `border=2pt.sty`. The mandatory half had
the same defect until #293 (`\documentclass{article}` → `missing_file:article`).
Content is never lost; the damage is a false `missing_file` in the log and tally.

Ground truth is the package being emulated. `standalone.sty` L604-614:

```tex
\newcommand{\sa@documentclass}[2][]{%
  \let\sa@subfile@options\@empty
  \ifsa@obeyclassoptions                      % \newif ⇒ default FALSE
    \edef\@tempa{#2}\edef\@tempb{standalone}%
    \ifx\@tempa\@tempb \def\sa@subfile@options{#1}%  % ONLY if the class IS standalone
    \else \fi
  \fi
```

It ignores a subfile's class options by default, and even under
`obeyclassoptions` consults them only when the subfile's class is literally
`standalone` — then handing them to a keyval family, never to `\RequirePackage`.

**We keep the loop but gate it twice.** The class must be `standalone`
(`standalone.sty`'s own `\ifx` guard), and the option must be one that
`standalone.cls` itself turns into a same-named package load — exactly
`tikz`, `pstricks`, `preview`, `varwidth`, `multido` (standalone.cls
L171/193/237/249/255, resolved at L562 and L611-620). Every other option
(`crop`, `multi`, `math`, `beamer`, `float`, `png`, `border=`, `class=`,
`10pt`/`11pt`/`12pt`, …) the class handles internally.

Note the asymmetry this creates, deliberately: the allowlist is read off
`standalone.cls`, but the code path being emulated is `standalone.sty`'s, and
**that path never `\RequirePackage`s any of them** — it hands the options to
`\setkeys{standalone.sty/class}`, where `tikz` means
`multi=tikzpicture,varwidth=false` (L815-820) and `varwidth` *warns* "Please
load this package in the preamble" and disables itself if the package is not
already loaded (L821-831). Verified empirically: with a plain
`\usepackage{standalone}`, `\ifsa@obeyclassoptions` is **false** (so real
LaTeX ignores subfile class options entirely) yet `varwidth.sty` is loaded
anyway, because `standalone.sty` preloads it unconditionally at L744-746 —
deleting `varwidth` from a subfile's options leaves pdflatex clean. So the
loop is a LaTeXML convenience justified by LaTeXML#1432, not a port; the
divergence is intentional and worth keeping, but it should not be described
as ported.

The option list is read as **`OptionalKeyVals`**, not comma-split by hand:
every one of these options has a valued form (`\sa@boolorvalue` accepts
`varwidth=5cm` / `tikz=true` exactly as the bare word, L815-824) and values
may be brace groups containing commas (`border={1pt 2pt}`). Matching whole
comma-split items missed all of it — `[varwidth=5cm]{standalone}` dropped the
package and raised `Error:undefined:{varwidth}` where pdflatex is clean, i.e.
a harder failure than the spurious warning this entry exists to fix. Matching
on the KEY of the parsed keyval list keeps the bare and valued forms
equivalent and puts these options through the same reader
`\documentclass`/`\usepackage` options already use.

Dropping the loop entirely would be *more* faithful to `standalone.sty`'s default,
but it would discard the reason the binding exists: upstream LaTeXML#1432's
motivating MWE is a `\documentclass[tikz]{standalone}` child, where the option
really is the package to load.

This is a **shared upstream bug**, not a Rust regression: same-host Perl warns
identically (`Warning:missing_file:12pt Can't find binding for package 12pt`).
See KNOWN_PERL_ERRORS #54 — candidate to upstream. Executing that preamble is
also what makes a package load inside our own bracket possible; the consequence
and its fix are #65.

Witness = issue #309's `index.tex` + `child.tex` (`No obvious problems` after,
1 warning before). Regression test:
`06_cluster_regressions::standalone_subimport_documentclass_no_spurious_require`,
which also guards the `{standalone}` half — its child *uses* `varwidth`, so
dropping a name from `CLASS_OPTION_PACKAGES` fails the test with
`Error:undefined:{varwidth}` — and the valued/brace-group form
(`[varwidth=5cm,border={1pt 2pt}]`) alongside it. **#293 originally landed a guard asserting the
un-gated behavior** (`[zzznope]{article}` must warn); this entry is why that
assertion was inverted rather than restored.

### 64. The generator identifier names the full product, "LaTeXML oxide"

Perl's generator stamp says "Generated … by LaTeXML" — the head comment in
`LaTeXML-common.xsl`'s `LaTeXML_identifier` template (`<xsl:text> by LaTeXML</xsl:text>`)
and the footer `ltx_page_logo` (the styled `LaTeXML` logo + Sammy mascot). This is
**our** product, not the Perl original, so both surfaces spell out the full name
**"LaTeXML oxide"**:

* `LaTeXML-common.xsl` — the head comment reads `<!--Generated by LaTeXML oxide
  (version X) http://dlmf.nist.gov/LaTeXML/.-->` (the `(version X)` is our own Cargo
  `X.Y.Z`, see divergence note in `core_interface::LATEXML_VERSION` / #320).
* `LaTeXML-webpage-xhtml.xsl` — the footer logo appends a plain-text ` oxide` after the
  `…XML` logo anchor (previously a parenthesized ` (oxide)`), so it renders
  "LaTeXML oxide".

Intentional, user-directed branding. Guard:
`latexml_oxide/tests/10_xslt_generator_version.rs` asserts both the head comment
(`by LaTeXML oxide (version …)`) and the footer (`…</a> oxide</div>`, and the absence
of the old `(oxide)`).

### 65. A package load is hoisted past LaTeXML's own subfile brackets

A package must end up defined at the outermost level; real LaTeX guarantees it by
refusing to load in a group at all. LaTeXML manufactures in-group loads real
LaTeX does not have — a `standalone`/`\import` subfile's preamble runs inside a
bracket of ours, and LaTeXML *executes* that preamble (which the real
`standalone.sty` gobbles) precisely so `\documentclass[tikz]{standalone}` loads
tikz (divergence #63). The package is then split in half: frame-local
definitions, global hooks. Anatomy, minimal trigger and the upstream verdict:
KNOWN_PERL_ERRORS #55.

**`require_package` hoists the load's conditionals past the bracket**
(`snapshot_top_frame_meaning_keys` + `hoist_top_frame_conditional_delta`), as
`tex.rs::def_autoload` does for the mirror-image autoload failure (witness
1711.11576) — note that one is UNGATED, since an autoload fires from arbitrary
body depth and has no bracket to be inside, and hoists the whole load (#282). We keep LaTeX's *invariant*, not its *enforcement*: refusing the
load would discard divergence #63.

**Only our own brackets.** An author's group keeps real LaTeX's verdict —
`{\usepackage{amsthm}}` leaves `\theoremstyle` undefined in pdflatex and in Perl,
so hoisting there would emit *fewer* errors than Perl on an authoring mistake.
What separates them is *where* the bracket was opened, so the region is named
`subfile:<frame depth>` — Perl's own `section:4` / `label:foo` convention
(State.pm L965-975) — and activated by `standalone_sty.rs` right after its
`bgroup()` (`import_sty.rs` activated it inside its `{…}` until batch 56hu, which
removed that group: #280). Activity
alone is NOT enough: `StashActive` is `Scope::Local` at the bracket's frame, so a
plain "am I in a subfile?" test is also true at every *deeper* frame, and an
author's `{\usepackage{…}}` written **inside** a subfile preamble was hoisted too
— Rust 0 errors where Perl reports 1. Matching the depth confines the region to
the bracket's own level. Mechanics and traps: WISDOM #66.

Refuted alternatives: **dropping the brackets** (a child's preamble then leaks to
its siblings and the parent — silent wrong content — and nesting stays broken);
**`\globaldefs=1`** (globalizes pgf's active-character handling →
`Error:undefined:"`, zero pictures); **hoisting every in-group load** (the
downgrade above).

Shared upstream defect (KNOWN_PERL_ERRORS #55) fixed at the mechanism;
surpass-Perl per RELEASE_CRITERIA §8. Neither binding's Perl-derived semantics
change — they gain only the `activate_scope` marker.

Guards in `06_cluster_regressions`:
`standalone_child_preamble_package_survives_the_subfile_group` (ungated;
`lx311demo.sty` = `\newif` + `\AtEndDocument`, over all four bracket routes —
`\input`, `\subimport*`, inside a parent-body group, nested in another child),
`standalone_child_tikz_survives_the_subfile_group` (the witness, TeX Live-gated),
`standalone_child_preamble_definitions_stay_scoped` (the half that must not
leak), `author_written_group_around_usepackage_still_loses_the_package` (the
boundary), `subimport_sibling_calls_do_not_accumulate_search_paths` (import.sty's
own path scoping, witnesses arXiv:2604.09744 / 2603.04457). Separately,
`100_stale_autoload_no_runaway::stale_autoload_trigger_does_not_run_away` asserts
the same boundary from a fresh process. A load in a subfile's *body* additionally errors `can only appear in the
preamble`, but the load still happens, so the region is observable there too —
which is why the gate is depth-matched, not merely active/inactive.

Scope of the hoist: **conditionals only**, and within the Meaning table only.
The failure this exists for is a definition destroyed while a *global* document
hook still reads it, and every witness is a `\newif` (`\newif` installs `\ifX` as
a Conditional; `\Xtrue`/`\Xfalse` are plain macros the hooks do not read).
Hoisting a package's ordinary macros as well is what made a second sibling
subfile render the FIRST one's content: promoting pkgA's `\newcommand` to global
turns pkgB's same-named `\newcommand` into a silent no-op, so sibling B shows A's
body — silent wrong content, and worse than Perl, which scopes both. Guard:
`standalone_child_preamble_definitions_stay_scoped`. The residue: a package
pairing a non-conditional definition with a document-level hook is still broken,
exactly as on `main` — no witness has needed it, and the Value/Catcode/register
tables are likewise untouched (`state.rs`: "callers that need to promote
Value/Catcode/etc. should add parallel helpers"), so a package that also sets a
length keeps the macro and loses the value.

### 66. `insertXML` accepts a document fragment, not only a single root

**Perl.** `LaTeXML::Common::XML::Parser::parseChunk` (`Common/XML/Parser.pm:36-39`)
carries its own comment — *"This expects only a single node, not a document
fragment"* — and LaTeXML ships no fragment parser at all, so a Perl binding that
wants to insert `<b>a</b><i>b</i>` has to wrap it in a container itself.

**Rust.** `common::xml::parse_fragment` (behind `Document::insert_xml` and the
Rhai `ParseXML`/`insertXML`) accepts a fragment: several sibling roots, or bare
text. It parses as-is first — so every single-root chunk, including one led by an
XML declaration, behaves exactly as `parse_chunk` always did — and only on failure
retries inside a throwaway wrapper whose children are returned.

**Why this is safe rather than a parity break.** The INSERTION half already
understood fragments: `Document::appendTree` has an `XML_DOCUMENT_FRAG_NODE`
branch in Perl, faithfully ported in `document.rs::append_tree`. Perl simply never
chains its parser to it. Accepting fragments therefore inserts more of the author's
content correctly and can never emit *fewer* errors than Perl.

### 67. A fence split by TeX's null delimiter is re-balanced before it is given up on

**Perl.** `\left( a+b \right.` is a fence whose right delimiter is *empty*, and
digestion emits no token for the `.` at all. When such a fence is split across an
alignment break — the standard way to break a long parenthesised expression:

```latex
\begin{align}
  H & = \frac{\hbar}{2} \left( \Omega_{IX} IX + \ldots \right. \nonumber \\
    & \quad \left. + \Omega_{ZX} ZX + \ldots \right).
\end{align}
```

each cell is left holding one unmatched delimiter. Perl's MathGrammar has no rule
for that, so it reports `not_parsed` (four warnings on the minimal case) and the
formula degrades to unstructured markup.

**Rust.** The XMath we build is byte-identical to Perl's — same `XMWrap` with an
unmatched `OPEN`, same bare `CLOSE` — so this is purely a parser difference. When
the grammar returns **zero** derivations, `parser.rs::balance_null_delimiters`
re-supplies the delimiter TeX says is there and retries once. The synthesized
`XMTok` carries the fence `role` but empty text, so it renders as nothing, which
is exactly `\right.`'s meaning; `tex=` reversions stay verbatim
(`\left(a+b\right.`). The cells then parse through the *existing* open/close
fence semantics as `delimited-(@(…)` / `delimited-)@(…)`.

**Why this is safe rather than a parity break.** It is gated on total parse
failure, so no formula that already parses can be re-read — the repair is
unreachable for them. That gate is load-bearing, not incidental: an earlier
unconditional version read the `⟩` of a ket `|f⟩` as a dangling close (its partner
is a `VERTBAR`, not an `OPEN`), prepended a bogus `(`, and broke formulae that had
been fine. Guarded by `tests/math/split_fence.tex`, which pins the ket case
alongside the split fence. Emitting *more* structure than Perl on input Perl
cannot parse never yields fewer errors than Perl. Witness: arXiv 2606.13010.

**What did NOT loosen.** Parser recovery stays OFF in both attempts. libxml's
recovery mode silently destroys author content — measured: `<b>a</b> <i>b</i>`
salvaged to just `<b>a</b>`, `a&nbsp;b` to `ab`, `a & b` to `a  b` — so malformed
markup is still rejected outright rather than quietly mangled. Perl agrees here:
`XML::LibXML->parse_string` defaults to `recover => 0`. Empty markup is not a parse
failure (it parses to zero nodes); `insert_xml` is the layer that reports it.

**The wrapper is invisible.** It is never returned, and never reachable: walking
UP from a top-level parsed node reports NO parent (`common::xml::is_parse_artifact`
covers both the wrapper and the parsed document node), so a script can neither see
`_lxfragment` nor splice it into the page with `insertXML(n.parent())`.

**Guards:** `common::xml::parse_fragment_tests::*` (siblings kept whole, bare text,
malformed still rejected, wrapper never returned, empty → zero nodes) and the
end-to-end `30_script_bindings::script_binding_macro_and_constructor_convert`
(`\rhfragment` parses two siblings, edits them while detached, inserts both, and
asserts `data-top="detached"` plus a blanket `!contains("_lxfragment")`).

### 68. Trimming a listing's trailing empty lines re-closes what the trim cut open

Perl `listings.sty.ltxml` L1330 drops trailing blank lines by slicing the generated
token vector:

```perl
@LaTeXML::lsttokens = @LaTeXML::lsttokens[0 .. $LaTeXML::emptyfrom - 1] if $LaTeXML::emptyfrom;
```

`$emptyfrom` is where the run of trailing empty lines began — but it is an index into
a token stream, not a structural boundary. When a delimited class (a string, a
comment, a styled span) is still **open** at that point, its closing `}` tokens live
in the discarded tail. The slice throws them away and the listing body is emitted with
unclosed groups; `\@@listings@block` then reads its arguments past the end of the
listing and off the end of the **document**.

`lastline=N` on a file with more than N lines is the everyday way to reach it: the
skip loop consumes the remaining lines without closing what the last rendered line
left open. Measured discarded tail on the witness below —

```
["\@lst@startline", "{", "}", "}", "}", "}", "\@lst@endline"]
```

— three of those `}` close groups opened *before* the cut, leaving the body at brace
depth 3.

**Both engines lose the snippet**, with different diagnostics: Perl reports
`Missing argument {} for \@@listings@block {}{}{}`, we reported
`Gullet->readBalanced ran out of input in an unbalanced state` plus a cascade of
`Attempt to end mode internal_vertical` and malformed-sectioning errors as the
mode-switch frame was never unwound.

Rust truncates as Perl does, then re-closes whatever the cut left open. The discarded
region is by construction only empty-line markup (it starts at a line with
`colnum == 0`), so nothing visible is lost by closing there.

Witness: arXiv 2412.04705 (arXiv/html_feedback#6735) — 22 errors → **0**, where
same-host Perl still reports 15. Guard:
`104_lstinputlisting_range_crlf::lastline_shorter_than_file_does_not_swallow_the_document`.

### 69. A listing source file's CRLF line endings are normalized on read

`listingsReadRawFile` slurps the file verbatim in Perl. Every end-of-line test in the
listings processor is then written against `\n` — the `__NEWLINE__` close test for
line comments, the blank-line test in `lstProcessStartLine`, the line-skipping loops —
and a `\r` sitting before the `\n` defeats all of them. A line comment therefore never
terminates, and its **style** bleeds over every following line. (The
`ltx_lst_comment` class wrapper does close; it is the font/colour group that leaks, so
the defect is invisible if you inspect classes rather than `font`/`color`.)

TeX never sees the CR — its file reader strips the line terminator and appends
`\endlinechar` — so normalizing `\r\n` and lone `\r` to `\n` at ingestion is what
matches the engine we emulate, not a special case.

Ground truth, arXiv 2412.04705 (arXiv/html_feedback#6735), whose Python sources are
CRLF: pdflatex renders only the `#` line in comment green (measured on the rendered
page: 9 green vs 69 black glyph groups), while both LaTeXML engines painted the whole
snippet green and slanted. Verified by A/B — pre-fix, an LF copy of the same file
renders correctly and the CRLF original does not.

Guard: `104_lstinputlisting_range_crlf::crlf_line_comment_style_does_not_bleed_past_its_line`.

### 70. A comma between digit groups is a thousands separator, not a list separator

**Perl behaviour.** `$50,000$` parses as a two-item LIST — `list(50, 000)` (or,
after a relation, `>(absent, list(50,000))`). The number ligature that would join
them (`Base_XMath.pool.ltxml` L506-508) demands `$r ne 'PUNCT'` on the separator
— "Be paranoid about lists" — and a math-mode comma is ALWAYS `role="PUNCT"`, so
for `en`, where the thousands separator *is* the comma, that arm is unreachable
dead code. The content arm is then nonsense and even the presentation grouping is
wrong: `mrow(mn 50, mo ",", mn 000)` instead of a single `mn`.

**Rust behaviour (owner-directed 2026-07-25).** `$50,000$` is ONE number, 50000.
The default reading is **US** (comma groups digits); the **European** reading
(comma as DECIMAL separator, `$3,14$` = 3.14) stays available and is selected the
way it already was — by the document language, through the `DECIMAL_SEP` /
`THOUSANDS_SEP` maps, whose decimal arm never had the role guard and so always
worked. Real corpora contain both conventions, hence a default rather than a
single answer.

Merged: `50,000`→`50000`, `1,234,567`→`1234567`, `12,345,678,901`, `1,234.56`.
Left alone: `3,14` (two digits — the European decimal shape), `50,0001`,
`f(x,000)` (no NUMBER left of the comma), `(1, 2024)`, `(12, 3456)`.
`$(12, 345)$` DOES merge — genuinely ambiguous, and this is what "default US"
decides. Presentation becomes a single `<mn>50,000</mn>`.

**Where, and why not elsewhere.** A `DefRewrite` in the post-build `Rewriting`
phase (`base_xmath.rs`, `THOUSANDS_SELECT_XPATH`), not the ligature. Ligatures run
per-token from `Document::open_math_text_internal` while the document is being
built, so `get_next_sibling()` is None for every node — there is no right context
at all, and a merge-on-three-digits rule fires before a fourth digit can arrive.
Measured: it turned `$(1, 2024)$` into `12024` and `$(12, 3456)$` into `123456`.
By the Rewriting phase the ligature has already collapsed each digit run into ONE
token, so the group length is directly testable and those cases are rejected by
construction. Package-built number markup is excluded (`not(parent::ltx:XMWrap)`)
— siunitx/numprint construct their own `XMDual(semantic, XMWrap[...])` and are
already correct.

Golden updated: `tests/complex/si.xml` (three author-typed `$3,762$` cells).
Guards `cluster_thousands_separator_us_default`, `cluster_thousands_separator_eu`.
Witness: arXiv 2605.17646 (`population $ > 50,000$`).

### 71. The author-year citation label is the SHORT author form, not every author

**Perl behaviour.** `MakeBibliography.pm` builds the `role="refnum"`
`ltx_bib_author-year` label from `do_authors`→`do_names` (L505-517, L568-584):
**every** author, with "et al." only when the BibTeX field literally ends
`and others`. It then drops the entry's first block — `shift(@blockspecs); # Skip
redundant 1st block!!` — because the authors are already in the label.

On a collaboration paper that label is the whole entry. Witness arXiv 2607.21432
(A&A, Simons Observatory): a **5104-character** citation label, and 9 of its 19
entries over 120 characters. Reported by a reader as
[arXiv/html_feedback#6797](https://github.com/arXiv/html_feedback/issues/6797).
Applies whenever the bibliography is built from a `.bib` (we do not interpret
`.bst`), which is the default `BIB_CONFIG` order `['bib','bbl']`.

**Rust behaviour.** The label is the short form — `Abitbol et al. (2025)`,
`Jones and Brown (2019)`, `Berg (2018)` — and the first block is **kept**, so the
full author list still appears in the entry body. Nothing is lost; the label
becomes usable. Max label on the witness: 5104 → **48** characters.

**Why — pdflatex is the ground truth for the shape.** Running the witness's own
`aa.bst` through BibTeX emits

```
\bibitem[{Abitbol {et~al.}(2025)Abitbol, Abril-Cabezas, Adachi, …}]{Abitbol_2025}
Abitbol, M., Abril-Cabezas, I., Adachi, S., {et~al.} 2025, JCAP, 2025, 034
```

natbib's **short** form (`Abitbol et al. (2025)`) is the citation label; the long
surname list is only natbib's *optional* full-author form, used by `\citet*` and
never printed in the bibliography; and the authors are shown in the entry BODY.
That is exactly the shape adopted here.

Two further signs the Perl behaviour is an oversight rather than a decision:
`do_names_short` — the correct `>2 → "First et al."` helper — is **defined at
`MakeBibliography.pm` L586 and never called**; and Perl's own `role="authors"`
tag already truncates at `>2` (L433-437), so the full-list label contradicted its
own neighbouring policy. See `KNOWN_PERL_ERRORS.md` #61.

Beyond Perl's unused helper, a trailing BibTeX `others` is dropped and forces the
"et al." form, so `Smith and others` reads "Smith et al." rather than
"Smith and others" — keeping it consistent with `do_names`, which does handle
`others`.

**Witnesses**: arXiv 2607.21432 (arXiv/html_feedback#6797).
**Guard**: `cluster_bib_long_author_list_refnum`.
**Upstream**: to file against `brucemiller/LaTeXML` (dead `do_names_short`).

### 72. A `.bib` field's `\url` gets url.sty's real definition, not a bare `\providecommand`

**Perl behaviour.** `convertBibliography` (`MakeBibliography.pm` L180-242) spins
the recursive BibTeX session with the article's class and packages preloaded and
nothing else. If the document loads neither `url` nor `hyperref`, `\url` is
simply undefined in that session, so a `.bib` field carrying `\url{...}` raises
`Error:undefined:\url` and its argument is digested as ordinary text — which
means a **percent-encoded** URL is truncated at the first `%` (catcode 14
comments out the rest of the line, closing brace included). Measured with
same-host `latexmlc` on `tests/cluster_regressions/bib_field_no_url_package.tex`:
**7 errors**, the note rendered as `https://example.org/B130936`, and the raw
`@misc{...}` entry spilled into it. Real `bibtex` + pdflatex break the same way.

**Rust behaviour.** Before digesting entries, the session provides the block a
`.bst`-generated `.bbl` provides (`\providecommand{\url}`, `\doi`, `\bibinfo`,
`\eprint`, `\newblock`, ...), and — the part that is a divergence —
when `\url` is *undefined* it loads LaTeXML's own `url.sty` binding rather than
settling for the `\providecommand` shape. That matters because the binding
declares `\url`'s argument **Semiverbatim** (`url_sty.rs`), which is what keeps
a `%` literal; a `\providecommand{\url}[1]{...}` renders but protects nothing.
Same fixture: **0 errors**, three entries, and
`B130936%20Law%20of%20War.pdf` intact.

**Why.** A `.bst` that emits `\url{...}` into the `.bbl` assumes the document
provides `\url`, and nearly every document that cites a URL does. Supplying the
real definition reconstructs what the author's document meant, rather than
punishing the bibliography for a package the *article body* happened not to need.
It cannot mask a diagnostic for a correct document: a document that loads
`url`/`hyperref` keeps its own definition — `input_definitions` early-returns on
its `_loaded` flag and `\providecommand` defers.

Guards: `06_cluster_bibliography::bib_field_bbl_fallbacks_render_without_a_url_package`
(no url package) and `::bib_field_markup_survives_into_the_bibliography`
(hyperref loaded — hyperref's `\url` must still win). *(Both moved out of
`06_cluster_regressions` when PR #400 split the bibliography cluster into its own
test file.)*

### 73. A `.bib` `abstract`/`keywords`/`contents` field is read verbatim, not digested

**Perl behaviour.** `BibTeX.pool.ltxml` L708-709/716-717/732-733 route these three
fields to `\bib@@field{ltx:bib-extract}[role=...]`, whose value is `Digested` —
so the field's content is tokenized and digested as TeX. These fields are bulk
prose, and prose contains `%`. A `%` is catcode 14: it comments out the rest of
the LINE, taking the field's closing brace with it. The entry's group never
closes, the next `@article` is absorbed as more abstract, and `\end{bib@entry}`
fails against the wrong group. The damage is not local — entries stack open and
the **entire** bibliography is lost.

Measured on witness **2605.00184** (`warm-ref.bib`, a Mendeley export with an
abstract on every entry, 52 entries): same-host `latexmlc` emits **101 errors
plus a `too_many_errors` Fatal and produces no output at all**. Perl's own
`\bib@field@default@abstract` cannot survive its own input.

**Rust behaviour.** The same three fields become `DefConstructor`s taking a
`Verbatim` argument (`bibtex.rs`). `Verbatim` calls
`begin_semiverbatim(Some(&['%', '\\']))` (`base_parameter_types.rs` L457-469),
so `%` and `\` are neutralized for the field's duration and the content cannot
break out of it. The value is then recovered from the entry's stored field via
`bib_extract_text` + a `#rawdata` property — **Perl's own idiom for a
`Verbatim`-read field**, whose comment at L346 reads "IGNORE the tokenized data"
(`\bib@field@default@default Verbatim Verbatim` -> `\bib@field@unknownasdata{#1}`,
which builds its content the same way). The verbatim token form is a dead end:
measured, `#1` substitutes to nothing in a macro expansion body, which leaves
`\bib@@field`'s `Digested` slot empty — that opens `ltx:bib-extract` and never
closes it, so every later field nests inside and the entry is malformed.

An **empty** value is a second, independent failure mode: `abstract = {},` is a
routine reference-manager export, and an element opened with no content is never
closed, so every later field nests inside `ltx:bib-extract` and the entry is
malformed. The constructor is therefore conditional —
`?#rawdata(<ltx:bib-extract .../>)()` with the property left undefined when the
text is empty — so nothing is emitted rather than something unclosed. Witness
2605.00555 `refs.bib` L956: that one line cost **86 errors**, while the same
file's 17 percent-bearing abstracts were already handled.

The **key is still supported exactly as Perl supports it** — the same
`ltx:bib-extract[@role]` element is emitted with its value whenever there is
one, so metadata consumers keep the data. Only the *reading* changed.

Measured, current binary: **2605.00184 102 -> 0 errors, 0 -> 41 entries**;
**2605.00555 86 -> 0 errors, 15 entries**. Seven more papers sampled from the
cluster (2605.00120/00208/00254/00314/00426/00440/00462) go to 0-or-1 errors
with **zero** `bibtex@bibliography` / `bib@entry` / `bib-extract` errors, from
300/77/57 on the worst three. In sandbox-arxiv-2605 run 272 (which PREDATES this
fix) the cluster is 695 `\end{bibtex@bibliography}` + 656 `\end{bib@entry}`
documents; `abstract` is the driver in 47 of 48 lethal-`%` field hits across an
8-paper sample (the 48th is a `title`, which is genuine parity — pdflatex breaks
on it too, and `title` is rendered so it cannot be read verbatim).

**Why.** Real BibTeX never puts these fields in front of a TeX tokenizer. A
`.bst` declares a closed `ENTRY` field list, and plain/unsrt/alpha/abbrv all omit
`abstract` and `keywords`, so `bibtex(1)` drops them when writing the `.bbl` and
pdflatex never sees them. Verified by running bibtex 0.99d: for an entry whose
abstract contains `64.84%`, the generated `.bbl` carries author/title/journal/year
and no trace of the abstract. Digesting them is an artifact of reading `.bib`
**directly** instead of running bibtex+bst, so pdflatex — not Perl — is the
ground truth here, and pdflatex's answer is "this text never reaches TeX".

The cost of the divergence is markup inside these three fields, and it is zero in
practice: **nothing renders `ltx:bib-extract`.** No format spec in
`make_bibliography.rs` queries it, in either engine's output path. This is
`surpass-perl` on a shared bug, not a parity gap — Rust was already better than
Perl on the witness before the fix (102 errors and no fatal, vs Perl's 101 plus
a fatal), and the fix widens that rather than papering over a Rust-only defect.

Other `.bst`-dependent fields keep Perl's exact coverage; only the three that
reach the unrendered `ltx:bib-extract` are neutralized.

Guard: `06_cluster_bibliography::bib_abstract_percent_does_not_sink_the_entry`,
which uses the new `convert_and_post_clean` helper — the ordinary
`convert_and_post` gates only the CORE stage, so a post-stage error flood (17 on
this fixture, 203 on the witness) passed every bibliography guard silently.
(percent in `abstract`, specials in `keywords`, and a third entry as the
containment canary).

### 74. A `.bib` field's content is DATA — `% & # _ ^` are literal, not catcodes

Supersedes the separate `%`-only and `&`-only entries this consolidates
(PRs #405 and #409); `_` was a third instance of the same defect.

**The two regimes, and the two treatments that restore them.** The real
toolchain is `pdflatex → bibtex → pdflatex`, and it treats these characters
differently at two distinct points. We collapse both into one pass over live
core state, so we have to perform both, in order — the frame is
[`BIBLIOGRAPHY_WORKLIST.md` → "two regimes collapsed into one pass"](BIBLIOGRAPHY_WORKLIST.md).

**Treatment 1 — reading the `.bib` (be `bibtex`).** A field's bytes are inert
data. BibTeX's lexer interprets only braces and the entry/field delimiters: `%`
is not a comment (it is significant only in the junk BETWEEN entries,
`Pre::BibTeX::skipJunk`), `&` is not an alignment tab, `#` is not a parameter.
So the read path neutralizes those three **without altering the text** — the
stored value keeps its exact bytes. `Mouth::with_bib_data_literals()` is a
per-Mouth property, applied to the per-entry mouth in `\ProcessBibTeXEntry` and,
via `mouth::tokenize_bib_literal` / `bibtex.rs::tokenize_bib_field`, to the
handlers that re-read a raw field instead. Deliberately NOT a State catcode
assignment: a raw `.sty` opened from inside a field handler — and the document
itself — must keep TeX's meanings, so the rule belongs to the *text BibTeX
lexed*, not to the session.

**Treatment 2 — synthesizing the `.bbl` and digesting it (be `pdflatex` pass 2).**
Now the content must be valid TeX. We are the ones writing the `.bbl` and we
know the author meant the literal character, so we escape at that boundary:
`%`→`\%`, `&`→`\&`, `#`→`\#`, `_`→`\_`, `^`→`\textasciicircum{}`.
`bibtex.rs::escape_bib_data_specials`. Escaping here rather than suppressing
catcodes during digestion is what keeps the TeX regime intact **by
construction**: `\emph{…}`, `{\v S}pakov` and `$x_1+x_2$` are already valid TeX
and simply pass through.

**Why `_` and `^` are in treatment 2 only.** A catcode is decided at
tokenization, before anything knows whether it is inside `$…$` — and a
sub/superscript in a title's math (`title = {Bounds on $x^2+y_1$}`) is
legitimate TeX that must keep working. Measured: adding `_` to treatment 1
silently flattened every subscript in a bibliography title. Only the escaper can
be math-aware, so the two scripting characters live there alone. The other three
have no legitimate TeX meaning inside a `.bib` field, in math or out.

**`^` is `_`'s twin, and the symmetry was verified rather than assumed.** Both
are TeX scripting characters; `bibtex(1)`'s lexer gives neither any meaning
inside an entry, so both are plain data; and outside math both raise the same
diagnostic, "Script … can only appear in math mode". Checked end-to-end: a
`note = {q _ r ^ s}` renders `q _ r ^ s`, zero errors, both characters intact.

**But the escape is NOT `\` + the character, and that is the whole reason `^`
needs its own arm** (`BIB_DATA_CARET`, placed before the generic
`BIB_DATA_SPECIALS` arm). `\_` is the underscore text command; `\^` is the
circumflex **accent**, so the generic arm would turn `^o` into "ô" — a wrong
glyph, silently, where the author wrote a caret. `\textasciicircum{}` is the
actual caret, and the braces are load-bearing so a following letter is not
absorbed into the control word. Idempotency needs nothing new: `\textasciicircum`
is a control word and is copied whole, `\^{}` is a control symbol and is copied
as a pair.

**The exclusion list is principled, not ad hoc.** A handler that consumes the
field's characters *itself*, under its own catcode regime — `url`'s Verbatim
href, `doi`'s Semiverbatim id — is still operating in **treatment 1, on data**,
so it must receive the *unescaped* value. `bib_field_source` reads that
declaration back off the `Definition` (Perl declares it per field,
`BibTeX.pool.ltxml` L740 and L684/L750-783) rather than keeping a second copy of
the list. **Trap:** only `Semiverbatim` sets the `semiverbatim` *descriptor*
field; `Verbatim` calls `begin_semiverbatim(Some(&['%','\\']))` inside its reader
closure and leaves the descriptor empty, so a descriptor-only test silently
misses `url` — measured, it planted a literal `\%` in a href. The check must
also test `Parameter::name`.

**Treatment 2 covers three seams, not one.** Two handlers re-read the RAW field
instead of using the value the entry line passed them, so escaping only the
entry line silently missed them — `title` most of all:

* `\ProcessBibTeXEntry`'s synthesized entry line (Perl L147-157);
* `\bib@@title` (Perl L293-333), which re-reads the raw field to recase it — the
  value it is handed lands in Perl's vestigial `ignoretitle` slot and is dropped;
* `\bib@@pages` (Perl L670-674), which re-reads the raw field to normalize `-`.

Escaping runs **before** `recase_title`: that pass splits on words and treats a
`\…` escape as part of its word, so raw `AT&T` would recase to `AT&t` (three
words) while raw `AT\&T` recases to `AT&T` (one). Every real `.bib` mixes both
spellings and they must render identically.

**Idempotency.** Most real `.bib` files already write `\&`, `\%`, `\_`. In the
escaper a backslash consumes the next character as a pair and neither is
re-examined, so `\&` stays `\&`. The tricky `\\&` falls out of the same rule:
`\\` is consumed as one pair, leaving a genuinely bare `&` that IS escaped —
and so does `\\^`, which becomes `\\\textasciicircum{}`.

**A nested data region.** url.sty reads `\url`/`\nolinkurl`/`\path`'s argument
verbatim, so `howpublished = {\url{http://x.org/a%20b}}` must keep its `%20`.
Those control words' single next group is copied through untouched
(`VERBATIM_ARG_COMMANDS`); `\href`'s *second* argument is prose and is still
escaped.

**A separate input corruption, fixed alongside: `\&amp;`.** A reference manager
rendered the field to HTML (`&` → `&amp;`), then a second pass TeX-escaped the
ampersand of that entity, so the file carries `\&amp;` / `{\&}amp;` / `&amp;`
where the source said `&`. TeX has no idea — `\&` produces the glyph and `amp;`
is four more ordinary characters — so the entry renders "Computer Engineering,
&amp; Applied Computing", and pdflatex prints exactly the same. Not a parity gap
in either direction: an input corruption only the `.bib` reader is positioned to
undo. `undouble_escaped_ampersand` decodes it to a plain `&` and lets the two
treatments give it its meaning. Witnesses: `\&amp;` in `booktitle` (2605.00833,
2605.01362), `journal` (2605.00922, 2605.01200, 2605.01224), `title`
(2605.01353); `{\&}amp;` in `title` (2605.01224); bare `&amp;` in `publisher`
(2605.00859) and `journal` (2605.01187).

**Why this is authorized surpass-Perl AND surpass-pdflatex.** User decision,
2026-07-27. LaTeXML reads `.bib` **directly**, with no `.bst` and no `bibtex(1)`
in the loop, so it is the component deciding what reaches the tokenizer. That
the real toolchain also breaks on these characters is a property of that
toolchain, not a semantic we are obliged to reproduce: the author's intent for
`AT&T` in a bibliography field is plainly the two letters, an ampersand and a T.
This supersedes the "genuine parity, pdflatex breaks on it too" reading that #73
applied to a `title`, and it covers `volume = {27 suppl_4}` and
`language = {en_US}`, which a narrower earlier version of this fix left erroring.

**Guards.** `06_cluster_bibliography::bib_field_specials_are_data_not_tex` (one
ten-entry fixture, because the risk is precisely that fixing one case breaks
another: the five specials bare, the same five already escaped rendering an
IDENTICAL string, `$x^2+y_1$` keeping BOTH its `SUPERSCRIPTOP` and its
`SUBSCRIPTOP`, `\emph` still markup,
`{\v S}pakov` still reverting, `%20` inside `\url{…}` intact, and `url`/`doi`
values with no backslash); `::bib_field_percent_is_an_ordinary_character`;
`::bib_bare_ampersand_is_literal_data`;
`::bib_bare_ampersand_leaves_live_markup_alone`;
`::bib_escaped_amp_entity_decodes_to_one_ampersand`;
`55_bibtex::runaway_field_costs_only_its_own_entry`; and the `escape_specials_*`
unit tests in `bibtex.rs` (seven when this entry landed, **13** today — #79 added
the five unmatched-`$` cases), which isolate the `\\&` hazard that cannot live
end-to-end (see below) and pin
`escape_specials_caret_is_textasciicircum_not_an_accent`.

**Measured**, `--release` before/after on the same host, TOTAL document errors:

| cluster | witness | before | after |
|---|---|---|---|
| `_` | 2605.06926 | 8 | **0** |
| `_` | 2605.01936 | 13 | **0** |
| `_` | 2605.04604 | 2 | **0** |
| `_` | 2605.08986 | 2 | **0** |
| `_` | 2605.05898 | 1 | **0** |
| `_` | 2605.11300 | 1 | **0** |
| `%` | 2605.01196 | 28 | **0** |
| `%` | 2605.02131 | 28 | **0** |
| `%` | 2605.00879 | 103 | **1** |
| `&` | 2605.06249 | 3 | **0** |
| `&` | 2605.03054 | 1 | **0** |
| `&` | 2605.00462 | 1 | **0** |
| `&` | 2605.08753 | 1 | **0** |
| `&` | 2605.10409 | 1 | **0** |
| `&` | 2605.00833 | 0 | **0** |

**193 → 0.** Two residuals are unrelated to this cluster and were unchanged by
it: 2605.00879's remaining error is `undefined:\mathsemicolon`, and 2605.11579
(listed for the `_` cluster) had 5 errors before AND after with zero
`unexpected:_` in either — its apparent three were an artifact of measuring in
the DEGRADED no-dump mode a fresh worktree starts in. Run
`tools/make_formats.sh` before believing any error count. (That 5 no longer
reproduces: 2605.11579 is at **0** on current main, because #80 stopped digesting
its uncited entries — a second reason to re-measure rather than quote.)
2605.00833 is a RENDERING witness, not an error-count one: its `\&amp;` printed
as "&amp;" and now prints "&".

**Known not covered.** A literal `\\` in a title makes
`\bib@field@default@title` open a nested `<ltx:bibitem>`
(`malformed:ltx:bibitem <ltx:bibitem> isn't allowed in <ltx:bib-title>`) with no
special character anywhere in the entry — a pre-existing behaviour of `\\` in a
bibliography, independent of this work, and the reason the `\\&` hazard is
pinned by a unit test rather than end-to-end. An `&` inside a `.bib` field's
`$\begin{array}…$` would lose its alignment meaning — treatment 1 is
catcode-level and cannot be math-aware; no witness exhibits one.

**A knock-on for the digest-once guard.** `105_bib_field_digest_once` needs a
probe that re-raises on EVERY digest (an undefined macro self-heals into
`<ltx:ERROR/>` on first sight and would pass with the bug present). It used `_`,
then `^` when `_` became data; both are now data, so the probe moved to `\hline`
in a `note`, which expands to `\noalign` — a context error with nothing to
memoize. Both scripting characters stay in that fixture as the standing
zero-error check.

### 75. A `.bib`-derived bibliography does not run the missing-`\bibitem` rescue

**Perl behaviour.** `BibTeX.pool.ltxml:183` gives `{bibtex@bibliography}` the
same `afterDigestBegin => beginBibliography` as a hand-written
`{thebibliography}`. `beginBibliography` = `beginBibliography_clean` +
`setupPseudoBibitem` (`latex_constructs.pool.ltxml` L4028-4047), and
`setupPseudoBibitem` `\let`s **both** `\par` and `\\` to
`\par@in@bibliography`, which emits a fresh `\save@bibitem{}` every time it
fires. Its purpose is stated in its own comment: "Since SOME people seem to
write bibliographies w/o `\bibitem`, just blank lines between apparent
entries".

That rescue cannot apply to a `.bib`-derived bibliography.
`Pre::BibTeX::toTeX` (L110-122) generates the body mechanically — one
`\ProcessBibTeXEntry{key}` per line between `\begin{bibtex@bibliography}` and
`\end{bibtex@bibliography}` — so there is never a missing `\bibitem` to
recover, and the only `\par`/`\\` that can reach the heuristic come from
**inside a field value**. There the heuristic opens `<ltx:bibitem>` in the
middle of `<ltx:surname>` / `<ltx:bib-title>` / `<ltx:bib-note>`; the model
rejects it (`Error:malformed:ltx:bibitem`), it is inserted anyway, and it is
never closed, so every later entry nests inside it.

Note the diagnostic trap: the element named in the error is **not** the one
that failed to close. `<ltx:surname>` is opened and closed exactly as its
constructor says — it is merely the insertion context at the moment the
spurious item is opened.

**Rust behaviour.** `{bibtex@bibliography}` calls `begin_bibliography_clean`
and skips `setup_pseudo_bibitem`. Nothing else in that environment needs the
skipped bindings: `\newblock`→`\lx@bibnewblock` is already `Let` globally
(`latex_constructs.pool.ltxml` L4133), `\bibitem`/`\item`/`\vskip` never occur
in the generated body, and the trailing "risky" lookahead that unreads a `\par`
is a no-op because the body starts with the executable `\ProcessBibTeXEntry`.
`{thebibliography}` and the biblatex/amsrefs/revtex/OmniBus bibliographies keep
the full `begin_bibliography` — the rescue still applies where it was meant to.

**Why.** Same-host Perl 0.8.8 emits byte-identical malformed output on the same
input (over the fixture `.bib`: the same 6 `malformed:ltx:bibitem` errors, one
of them naming `<ltx:givenname>` where Rust names `<ltx:surname>` — a name-split
nuance, same cluster), so pdflatex is the ground truth, and it disagrees with
both engines:
bibtex 0.99d compresses every white-space run in a field value to a single
space, so a blank line **never reaches TeX at all**, and it copies `\\` through
to the `.bbl`, where `thebibliography` renders it as a line break *inside* the
item. Verified by running bibtex 0.99d over the fixture's `blankline` entry:
the generated `.bbl` reads `Debarati Das and Michal Koucky.` and `A dynamic
structure for one-dimensional top-k range reporting.` on single lines. Under
neither reading does a field value start a new bibliography item.

Measured, current binary: **2605.03313 7 -> 0**, **2605.03693 7 -> 1** (the
residual is an unrelated text-mode `^`), **2605.11080 1 -> 0**. The final
rendered HTML is **byte-identical** on all three — `MakeBibliography` rebuilds
each cited entry from its fields, so the injected items never reached the page
and the whole cost was diagnostic noise plus a malformed intermediate.

Guard: `06_cluster_bibliography::bib_field_blank_line_does_not_inject_a_bibitem`
(`convert_and_post_clean`, since the damage is done in the recursive `.bib`
session that `MakeBibliography` drives during POST). Fixture
`bib_field_blank_line.{bib,tex}` carries all three witness shapes: a blank line
in `title`, in `author`, in a `note`-routed `Annote`, plus the `\\` note — 6
errors RED, 0 GREEN.

### 76. — RETIRED NUMBER, deliberately unused. Do NOT reuse it.

Not a mistake and not a gap to fill. #76 briefly held "A `.bib` field's content is
DATA — `_ & # %` are literal, not catcodes", whose first witness was the
underscores in an `eprint` PDF URL. When the separate `%`-only and `&`-only
entries were consolidated it was merged into **#74** — which adds `^`, the
two-treatment framing, and the exclusion list — and the number was **retired
rather than renumbered**, because divergence numbers are cited verbatim from
`.rs` comments and renumbering silently invalidates every citation. Nothing in
the tree cites `OXIDIZED_DESIGN #76`. For the next free number see the header at
the top of this file — it is the single authoritative counter. (This spot used to
restate it and drifted stale at **#81** while the header had already moved on;
restating the number in two places is what makes it get taken out from under
people.)

### 77. `silence.sty` and the bundled `arxiv.sty` family get bindings Perl does not have

**Perl behaviour.** Neither package has a `.ltxml` — not in
`LaTeXML/lib/LaTeXML/Package/`, not in the installed 0.8.8 tree, not in
ar5iv-bindings. `\keywords` exists in Perl only inside a *class* binding
(`OmniBus.cls.ltxml`, `llncs.cls.ltxml`, `sv_support.sty.ltxml`, …), and these
papers are `\documentclass{article}` + `\usepackage{arxiv}`, so none of them
fires. Both packages are therefore undefined in any configuration that does not
raw-load the `.sty`. Measured, same-host Perl 0.8.8, verbose, no `--includestyles`:
2605.05327 `undefined:\WarningFilter`; 2605.06624 `undefined:\WarningFilter` +
`\ActivateWarningFilters` + `\DeactivateWarningFilters`; 2605.02338 and 2605.10111
`undefined:\keywords`. So these bindings are **new work, not a port**, and Rust
ends up with fewer errors than Perl on all four.

**Why the gap is worth closing rather than matching.** LaTeXML recovers an
undefined CS as a **zero-argument** `<ltx:ERROR/>`, so the arguments do not
vanish — they leak into the body as text. 2605.05327 renders the literal
"Extended allocation already in use" (an `\WarningFilter` message) in the
document, and 2605.02338 renders its keyword list as an unlabelled paragraph
next to an `ltx_ERROR` span. Arity, not the body, is what the binding is for.

**Two different shapes, because the two packages are different kinds of file.**

*`silence.sty`* is a stable CTAN package (v1.5b, 2012) whose entire job is
filtering LaTeX's *console* messages; it contributes no document content, so
every public command is a no-op with silence's own arity
(`\WarningFilter*[family]{package}Semiverbatim`, …). It is registered
**unconditionally** — it pre-empts the raw `.sty` everywhere — because the raw
file rebinds `\PackageError`/`\GenericError` (L581-599) and `\ErrorsOff` then
drops messages that LaTeXML would have reported. Measured on a two-line probe
(`\usepackage{silence}\ErrorsOff` + a package raising `\PackageError`): Perl
`--includestyles` reports **0 errors**, Rust with this binding reports **1**.
Pre-empting the raw file *restores* a suppressed diagnostic; it does not hide one.

*`arxiv.sty`* (George Kour's arXiv-preprint style) and its `PRIMEarxiv.sty` fork
are **bundled inside the paper**, so their contents vary and they carry real
formatting — `\@maketitle`, `abstract`/`table` redefinitions, section shapes.
Their bindings are therefore gated on `lookup_bool("INCLUDE_STYLES")` — the same
predicate the raw-load path uses (`binding/content.rs` L776): when raw style
loading is available the binding does nothing but hand control back to the
paper's own file, and only in bare mode does it define arxiv.sty L10/L29-31/L33/
L44-47 (`\keywords`, `\keywordname`, `\headeright`, `\undertitle`,
`\shorttitle`, and the two `\RequirePackage`s a registered binding would
otherwise skip past the dependency scan). `\keywords` is ported verbatim,
including the local `\and` → `$\cdot$` rebinding and PRIMEarxiv's unbraced
`\emph Keywords`, so bare mode renders exactly what the raw load renders.

**Measured.** Bare mode, four witnesses: 2605.05327 **1 → 0**, 2605.06624
**4 → 1** (the residual `unexpected:&` is an unrelated pre-existing cluster,
present in the ar5iv baseline too), 2605.02338 **1 → 0**, 2605.10111 **1 → 0**.
ar5iv mode (`--preload=ar5iv.sty`), before vs after: all four HTML outputs
**byte-identical**, same error counts. A fifth witness, 2504.08779
(`ascelike-new.cls`, whose `\RequirePackage{silence}` never reaches a raw load),
carried `undefined:\WarningFilter` in the full-arXiv corpus report and now
converts at 0 errors — registering the binding is what lets the class's
dependency scan resolve `silence` at all.

Corpus scale from the full-arXiv report: `\keywords` 428 tasks;
`\WarningFilter` witnesses 2010.00969, 2101.00910, 2504.08779, 2508.11482,
2509.17283, 2509.20705, 2509.20709, 2510.02612, 2512.12232, 2512.14031,
2601.01344, 2602.11517.

Guards: `00_contrib::silence_filters_test`, `00_contrib::arxiv_keywords_test`,
`00_contrib::primearxiv_keywords_test` (bare mode, golden `.tex`+`.xml` pairs),
`106_arxiv_sty_defers_to_bundled` (the `INCLUDE_STYLES` gate — a bundled
`arxiv.sty` with a distinctive keyword label must still win),
`107_silence_keeps_diagnostics` (the raw-load suppression above).

### 78. `mathscinet.sty` gets a binding — and nothing loads it for you

`mathscinet.sty` is a real package: AMS, v1.05 (2002/04/17), LPPL, shipped in TeX
Live inside the **amsrefs** bundle
(`texmf-dist/tex/latex/amsrefs/mathscinet.sty`). It holds the vocabulary
[MathSciNet](https://mathscinet.ams.org) records transliterate Cyrillic and
South-Slavic names with: `\cprime` (ь), `\Dbar`/`\dbar` (Đ/đ), `\cdprime`,
`\bud`, `\cydot`, `\polhk`, `\soft` and the under-accents. Perl LaTeXML has
`amsrefs.sty.ltxml` but **no** `mathscinet.sty.ltxml`, so the binding is a
Rust-only addition — though a port of the real `.sty`, not an invention.
`latexml_package/src/package/mathscinet_sty.rs`.

**Mappings come from the file's own T1 branches**, which say what each glyph IS
rather than how it is drawn: `\Dbar`→`\DJ` (U+0110), `\dbar`→`\dj` (U+0111),
`\cprime`→`\tprime` (U+02B9), `\cdprime`→two of them (U+02BA), `\polhk`→`\k`,
`\soft`→`\v`, `\udot`→`\d`. The Default branches overprint with `\accent` and
`\hbox` kerning (`\Dbar` is
`\leavevmode\lower.5ex\rlap{\hskip-.07em\accent"16}D`), which would reach the XML
as a bare "D" (WISDOM #50). Two deliberate departures from the source: L36's
`\RequirePackage{textcmds}` is not reproduced (it would raw-load a
`\pcatcode`-juggling docstrip `.sty` for two commands whose results are inlined),
and `\Cprime`/`\Cdprime` — `cyracc.def` L53-55 spellings that arrive with the
same data but are absent from this `.sty` — are provided alongside.

**Nothing auto-loads it, and that is the load-bearing decision.** The recursive
`.bib` session already loads `url.sty` on a document's behalf (divergence #72),
so doing the same for mathscinet is the obvious move. It would be wrong.
Checked, not assumed, on witness 2605.11579: the paper never mentions
`mathscinet` or `amsrefs`, and it uses `\bibliographystyle{alpha}` — and
`alpha.bst` contains **zero** occurrences of `Dbar`, so no `.bst` `@preamble`
supplies it either. `\Dbar` is therefore undefined in the author's own build:
real pdflatex raises the same undefined control sequence. **The residual
`undefined:\Dbar` is PARITY, not a defect**, and supplying the macro anyway would
push our error count below what the author's toolchain produces — the one thing
the canvas signal must never do. (That witness no longer *shows* the residual —
see the measurement below — because its `\Dbar` entry is uncited, not because
anything about this reasoning changed.)

**Why a package and not an always-present kernel definition.** A format-chain
definition runs before the document's preamble, and LaTeXML's `\newcommand` over
an already-defined CS silently keeps the OLD meaning (no error, no warning), so
an always-present vendor macro SHADOWS an author's own. Scanned 4,000 papers of
arXiv 2605:

| macro | authors define it | with `\newcommand` (would be shadowed) | used-but-undefined |
|---|---|---|---|
| `\cprime` family | 10 | 0 (all `\def`, which overrides cleanly) | ~20 |
| `\Dbar` | 6 | **4** | 2 |
| `\dbar` | 12 | 8 | 0 |

An always-on `\Dbar` renders `Đ` where four authors wrote
`\newcommand{\Dbar}{\bar{D}}` — verified, with zero diagnostics — i.e. it breaks
more papers than it fixes. Inside a document that DOES load the package, the
upstream `\ProvideTextCommand`/`\ProvideTextCommandDefault` deferral (kept as
`mathscinet_sty.rs::provide`) yields to a name already taken. That is also what
makes `\dbar` safe to bind here and nowhere else.

Read the `\cprime` row correctly: **zero** authors define that family with
`\newcommand`, so a stub for it would have shadowed nobody. Shadowing is the
`\Dbar`/`\dbar` argument, not the `\cprime` one — the `\cprime` family is
package-only because the errors that motivated a stub were manufactured by
digesting uncited entries (below), and because vendor vocabulary belongs to the
vendor's binding.

**The `\cprime` family is package vocabulary too — there is NO always-on stub**
(deleted 2026-07-27; the block it left behind in
`latex_constructs_rust_only.rs` records why). All three of its witnesses load the
package by name — 2508.13753 L7, 2508.20226 L3, 2509.07628 L13 — which corrects
the `cyracc.def` / "no Cyrillic encoding otherwise loaded" justification the
family used to carry. The stub briefly lived in `latex_constructs.rs`, then moved
to `latex_constructs_rust_only.rs` §5 (the Perl-parity file mirrors
`latex_constructs.pool.ltxml` byte-for-byte and must hold no non-Perl
definitions), then went away entirely. `\polhk` stays behind in
`latex_constructs.rs` as a fallback; its comment there claimed tipa.sty as its
source, and the real one is `mathscinet.sty` L111-113, corrected in place.

**Why the stub's justification collapsed.** It rested on four papers regaining
`undefined:\cprime` without it (2605.00173/.00186/.00190/.00305). Three of the
four were artifacts of a defect since fixed: we digested EVERY entry of a `.bib`
library, so we met `\cprime` in entries `bibtex(1)` never copies into the `.bbl`.
Since **divergence #80** digests only the CITED entries, that trigger is gone
structurally rather than papered over — and a definition that is always live can
shadow an author's own, the same hazard that kept `\Dbar` package-only from the
start.

**The rule, therefore.** `mathscinet.sty`'s vocabulary belongs to its binding. A
paper gets it the way the real toolchain gives it: by loading `mathscinet` (or
`amsrefs`, which `\RequirePackage{mathscinet}[2002/01/01]`s it at `amsrefs.sty`
L217), or by carrying the definition in its own `.bib` `@preamble` — which
executes, faithfully to Perl (`Pre/BibTeX.pm::toTeX` L118-122 →
`pre_bibtex::to_tex`), guarded by
`bib_preamble_defines_macros_for_the_whole_bibliography`.

**Measured** on current main, `--includestyles`, idle box, serial — total
document errors and `undefined:\cprime` count:

| paper | errors | `undefined:\cprime` | why |
|---|---|---|---|
| 2605.00173 / .00186 / .00190 | 0 | 0 | the `\cprime`-bearing entry is **uncited** (2605.00173: `MR2562222`, `bibliography.bib` L885), so #80 never digests it |
| 2605.00305 | 1 | 1 | **the only real cost, and it is honest.** It CITES `MR710121` (`mybib.bib` L26, `MRREVIEWER = {V.\ Z.\ Enol\cprime ski\u i}`), loads neither `mathscinet` nor `amsrefs`, ships no `@preamble`, and uses `\bibliographystyle{plain}` — `plain.bst` contains zero `cprime`. Real pdflatex fails there too |
| 2605.11579 | 0 | 0 | its own `.bib` `@preamble` — 14 copies of `\def\cprime{$'$}`, `biblo.bib` L4768/L6910/… — covers all 17 uses |

So the whole cost of package-only is one paper and one PARITY diagnostic. Earlier
corpus framing, kept because it is expensive to re-derive: across the first 600
papers of arXiv 2605, **seven** use `\cprime` inside a `.bib` and **six carry no
`@preamble` at all**; per-paper table in
[`BIBLIOGRAPHY_WORKLIST.md`](BIBLIOGRAPHY_WORKLIST.md).

**Measured**, same host: 2508.13753 **0 errors**, `Kondratʹev` composing;
2508.20226 **0 errors**; 2509.07628 `--includestyles` **0 errors**, `Drinfelʹ d`
composing (its 6 bare-mode errors are an unloaded local `Latex-document.sty`,
unrelated). 2605.11579 measured **1 error / 36 bibitems** before #416 and **0
errors** after: the `\Dbar` residual vanished not because the macro became
available but because the entry carrying it (`KacNilpotentorbits`, `biblo.bib`
L2059) is **uncited**, so #80 never digests it. The PARITY reasoning above is
unchanged — a paper that CITES a `\Dbar` entry while loading no package still
gets the diagnostic — but that case is now pinned by the guard fixture alone, not
by this witness.

Guards: `06_cluster_bibliography::bib_mathscinet_package_supplies_its_transliteration_glyphs`
(a document that loads the package, exercising both body prose and a `.bib`
`MRREVIEWER`; `\Dbar` was the discriminating assertion while `\cprime` had a stub
beneath it — both are package-only now, so either discriminates) and
`::bib_mathscinet_macro_yields_to_the_authors_own_definition` (a document that
does not — RED under an always-on `\Dbar`, where the author's barred-D math
renders `Đ`, and equally RED if the `.bib` session is made to auto-load).
`bib_mr_reviewer_accent.tex`, whose `primerev` entry carries `Gel\cprime fand`,
now loads the package: that fixture is about accent welds surviving reversion,
not about macro availability.

### 79. An UNMATCHED `$` in a `.bib` field is data, not a math shift

Extends divergence #74 (a `.bib` field's content is DATA) to the one special it
left out. Treatment 2's escaper is math-aware precisely so `$x_1+x_2$` passes
through — but it treated `$` as an unconditional toggle, with no check that the
toggles pair up.

**The defect.** One stray `$` opens an inline-math group that never closes. A
`.bib` is digested as ONE unit, so the leak crosses `\end{bib@entry}` and every
subsequent element of every subsequent entry lands inside `<ltx:XMath>`:
`<ltx:bib-organization> isn't allowed in <ltx:XMath>`, ~100 times over, tripping
the 100-error cap and taking the whole document **Fatal**. Witness 2605.00166,
`annote = {… costs … are probably of the order of $10 million. …}` — a literal
currency dollar: **0 errors before the `.bib` became a real conversion, 103 and
a Fatal after**. Across the 2605+2606 sandboxes, 33 of the 69 papers whose
bibliography sub-conversion newly failed carry an odd `$` in a field, mostly in
`title`, `abstract` and Mendeley-exported `keywords`/`mendeley-tags`.

**The rule.** A `$` with no partner cannot be a math delimiter — there is no
reading under which it opens a span that closes. It is therefore data, and `\$`
is what a careful `.bst` author would have written, which is exactly #74's
stated principle. With pure toggling an even count is always balanced, so only
an odd count has an unmatched member; "immediately followed by an ASCII digit"
is the tell that picks which one, and it settles the real cases:
`of the order of $10 million` (demote the lone toggle), `$x$ costs $10` (demote
the digit one, `$x$` stays math), `costs $10 and $x$` (note the *last* toggle
would have been the wrong pick), `$1 and $2 and $3` (demote all three). With no
digit anywhere, fall back to the trailing toggle.
`bibtex.rs::demote_unmatched_dollars`.

**SURPASS-PERL, and deliberately.** Same-host Perl cascades identically —
`latexmlc` on 2605.00166 gives 102 errors and `Fatal:too_many_errors:100`, rc=1
— because `\bibentry@create` interpolates the field raw into a fresh Mouth
(`BibTeX.pool.ltxml` L155-166) with no escaping of any kind, and `Digested`
(L230) then digests it live. Upstream *already agrees* an unmatched `$` is not
math: `\bib@@title` balances it with `extract_delimited`, raises
`Error('expected','$',…)` and **deletes** the stray (L324-330). It just applies
that judgement to one field, and drops the character instead of keeping it. We
apply it to every field and keep the character.

Nothing is suppressed: this removes the CAUSE of the cascade, so the ~100
`malformed:` errors are gone because the entry is now well-formed, not because
they stopped being reported. It costs no fidelity against `pdflatex` either — a
standard `.bst` never emits `annote` at all, so the character only ever reaches
a tokenizer because we read the `.bib` directly, with no `.bst` in the loop.

Guards: `bibtex.rs::escape_specials_lone_dollar_is_currency_not_math`,
`::escape_specials_balanced_math_is_untouched`,
`::escape_specials_digit_dollar_is_preferred_over_the_last`,
`::escape_specials_all_currency_dollars_demote`,
`::escape_specials_unmatched_dollar_without_a_digit_falls_back_to_the_last`,
and `06_cluster_bibliography::bib_unmatched_dollar_does_not_leak_math`.

### 80. A `.bib` library is digested down to the CITED entries

**Perl behaviour.** `Pre/BibTeX.pm::toTeX` (L110-122) emits one
`\ProcessBibTeXEntry{key}` per entry in the file, unconditionally, and the whole
block is then digested. That was affordable while a raw `.bib` was read by a
hand-rolled string parser; since it became a real conversion (PR #396) every
entry costs a full expand/digest/construct cycle.

**What that cost.** A `.bib` is a library, not a document: `anthology.bib` ships
**80,576** ACL entries and the citing paper wants **9**. Witness **2605.07796**:
112 s and 4.8 GB RSS, tripping the memory budget and producing **zero**
bibentries — the paper loses its whole bibliography — with the fleet's 60 s
timeout killing the conversion outright. The same shape covers **59 of the 69**
papers in the 2605/2606 sandbox `never_completed_with_retries` bucket (median
80,597 entries each); 10 of them had converted cleanly before #396.

**Rust behaviour.** `pre_bibtex::to_tex` emits `\ProcessBibTeXEntry` only for the
selected entries. The cited set is not guessed: `MakeBibliography` already holds
it as the `BIBLABEL:<list>:<key>` ObjectDB records written during the *document*
conversion, so it is complete before post-processing asks. It travels as
`BibConversionRequest::wanted_keys` → `pre_bibtex::set_wanted_keys` (a
thread-local the caller must `clear_wanted_keys` after use).

**Why this is MORE faithful, not less.** `bibtex(1)` has always read the `.aux`'s
`\citation` records and emitted only those entries, plus `crossref` targets, plus
everything under `\nocite{*}`. Filtering here reproduces the real pipeline;
digesting the library whole never did.

**Selection is closed transitively** (`pre_bibtex::select_cited`) over both edges
that can reach an uncited entry — `crossref`, BibTeX's own inheritance link, and
a `\cite` made from inside an already-selected entry, which `getBibEntries`
follows — so a filtered run keeps everything an unfiltered one kept.

**Every entry is still REGISTERED** (`bibtex::register_entry`, Perl's
`assignValue 'BIBENTRY@<lc-key>'` at L114-116). That is a map insert of
already-parsed strings, and keeping it complete is what lets `crossref` and by-key
lookup resolve against an entry nobody cited.

**`None` means "digest everything"**, and is used for `\nocite{*}` and —
deliberately — when no `BIBLABEL` record exists at all: an empty filter and absent
citation data are indistinguishable, and dropping every entry on a missing record
would be a silent, unrecoverable total loss.

**Measured.** 2605.07796: 112 s / 0 bibentries / killed → **10 s / 9 bibentries /
0 errors**, matching what the pre-#396 run produced. 9 of the 10 cleanly-converting
regressions recover; the tenth (2605.16752) is an unrelated
`Fatal:Timeout:TokenLimit`.

**A knock-on that invalidates older measurements, and is expensive to
rediscover.** An error raised only by an *uncited* entry now disappears without
the macro becoming available. That is what took 2605.00173/.00186/.00190 off the
`undefined:\cprime` list and what removed 2605.11579's `undefined:\Dbar` residual
(its `KacNilpotentorbits` entry, `biblo.bib` L2059, is uncited) — see #78.
**Re-measure any bibliography error count recorded before 2026-07-27** rather than
reading a drop as a fix.

Guards (`pre_bibtex.rs`): `filter_digests_only_the_cited_entries`,
`filter_follows_crossref`, `filter_follows_a_cite_from_inside_an_entry`,
`filter_still_registers_every_entry`,
`filter_is_case_insensitive_like_the_registry`,
`filter_tolerates_a_cited_key_with_no_entry`, `no_filter_digests_every_entry`,
`cite_key_scanner_handles_the_cite_family`.

### 81. amsmath's `\ext@arrow` / `\arrowfill@` internals get bindings Perl does not have

**Perl behaviour.** `LaTeXML/lib/LaTeXML/Package/amsmath.sty.ltxml` binds the
*public* extensible arrows (`\xrightarrow`, `\xleftarrow`, …) as constructors and
never defines the internals they are built from. `\ext@arrow` (amsmath.sty L1012,
`\def\ext@arrow#1#2#3#4#5#6#7`) and `\arrowfill@` (L971, `\def\arrowfill@#1#2#3#4`)
are therefore **undefined** in Perl in every configuration that does not raw-load
`amsmath.sty` itself. Any package or preamble that builds its own arrow on top of
them — `extpfeil.sty`'s `\newextarrow`, `mathtools`' `\xhookrightarrow`, a
hand-rolled `\newcommand*{\xfoo}[2][]{\ext@arrow …}` — hits
`Error:undefined:\ext@arrow` plus `Error:undefined:\arrowfill@`, and the arrow's
own arguments then leak into the surrounding math as text.

**We define both** (`latexml_package/src/package/amsmath_sty.rs`), passing through
to `\to^{above}_{below}` and `\to` respectively — we do not model stretchy arrow
rendering, but the arity is what the binding is for. Witnesses 2411.17873 and
2412.00464 (amsmath's own `\ext@arrow 0359\rightarrowfill@…`), 1308.1071
(`extpfeil`'s `\xmapsto`), 2606.01903 (`extpfeil`'s `\xtwoheadleftarrow`). On
2606.01903 this is the whole difference between the two engines: same-host Perl
0.8.8, verbose, ar5iv profile, reports **258 errors** with `MAX_ERRORS` lifted
(and `Fatal:too_many_errors` at 102 with the shipped cap of 100); we report **0**.

**The parameters are TeX undelimited arguments, all of them.** Each `#n` of a
plain `\def` reads a single token OR a balanced `{…}` group. Spelling any of them
`Token` in the parameter spec reads only the opening `{` of a braced argument and
spills the remainder — including its closing `}` — back into the stream, where the
stray `}` closes the enclosing math group and everything after it is swallowed
into the leaked `<ltx:XMath>`. The braced form is not exotic: `\newextarrow`
expands to `\ext@arrow #2{\arrowfill@#3}{##1}{##2}`, and the `\mkern` quadruple
`#2` is only four bare digits when every amount is a single digit —
`\newextarrow{\xtwoheadleftarrow}{500{40}}{…}` braces the 40. So all seven
parameters of `\ext@arrow` and all four of `\arrowfill@` are `{}`. Guard
`06_cluster_math::cluster_ext_arrow_braced_mkern`.

### 82. spconf.sty's `keywords` block becomes `ltx:keywords` frontmatter, not inline body text

**Perl behaviour.** `spconf.sty` (the ICASSP/ICIP/Interspeech conference style,
bundled inside the paper) has no `.ltxml` — not in `LaTeXML/lib/LaTeXML/Package/`,
not in the installed 0.8.8 tree. Its "Index Terms" block is a bare plain-TeX
environment pair, not a `\newenvironment` (spconf.sty L211-214):

```tex
\def\keywords{\vspace{.5em}{\bfseries\textit{Index Terms}---\,\relax}}
\def\endkeywords{\par}
```

Measured, same-host Perl 0.8.8, verbose, witness 2605.00480 (`main.tex`):
**bare** = 4 errors (`\name`, `\address`, `\ninept`, `{keywords}`);
**`--includestyles`** = 0 errors, because the raw `.sty` is read — but the block
then lands in the body as
`<para><p><text font="bold italic">Index Terms<text>—…`, with **no `ltx:keywords`
element**. Perl produces **zero `<creator>` and zero `<keywords>` for these papers
in either configuration**: LaTeXML locks `\maketitle`, so spconf's own
`\def\maketitle` — the only thing that would ever emit the stashed `\@name` — is
ignored (`Info:ignore:\maketitle:locked`). That is why this binding is registered
unconditionally rather than gated on `INCLUDE_STYLES` the way the bundled
`arxiv.sty` is (#77): deferring to the raw file here *loses* the frontmatter.

**We bind it as structured frontmatter.** `latexml_contrib/src/spconf_sty.rs`
defines `\keywords` → `\lx@begin@keywords[name={\spconf@keywordsname:~}]` and
`\endkeywords` → `\lx@end@keywords`, mirroring the `.sty`'s `\def` pair rather
than declaring a `DefEnvironment!` (so a document calling the two macros directly
also works). The label rides in `@name`, not in the content; the XSLT renders it
as the block's `<h6 class="ltx_title ltx_title_keywords">`.

**Why this exact shape.** spconf.sty's own comment says the section was "adapted
from IEEEtrans", and IEEEtran.cls L5286-5288 typesets it identically
(`\textit{\IEEEkeywordsname}---`). Perl LaTeXML **does** bind that construct, in
`IEEEtran.cls.ltxml` L147-148 — as `\lx@begin@keywords[name={\IEEEkeywordsname:~}]`,
i.e. `ltx:keywords` with the label in `@name` and the print-only `---` separator
normalized to `:~`. So the divergence is only against *raw-loaded* spconf; against
Perl's own binding for the same markup it is a verbatim follow.

`\keywords` is argument-less in the `.sty`, so `\keywords{a, b}` is legal there
too — the group just typesets after the label. Routed straight to the environment
opener that form has no `\endkeywords` to stop at and
`\lx@add@frontmatter@until` scans to EOF, pulling the whole body inside
`<ltx:keywords>` (loudly: `malformed:ltx:section`, `malformed:ltx:document`). The
binding peeks for a `{` and dispatches to a one-argument form, exactly as Perl
does for the same legacy pair in `IEEEtran.cls.ltxml` L398-404
(`\keywords@onearg`). No corpus paper hits it today (`undefined:\keywords` has
zero reports in either sandbox corpus), but the form is valid spconf input.

The same file's `\twoauthors{names1}{affil1}{names2}{affil2}` (L183-190) is bound
alongside, routed to `\author{#1 \\ #2 \and #3 \\ #4}` so each pair becomes a
creator with its own affiliation instead of a zero-argument `<ltx:ERROR/>` whose
four braced arguments leak into the body as text.

**Corpus scale.** `{keywords}` is the single largest `undefined` *what* in the
sandbox corpora: **94 tasks in sandbox-arxiv-2605**, **49 in sandbox-arxiv-2606**;
142 of those 143 papers ship a byte-identical `spconf.sty`. `\twoauthors` adds 3.

**Measured**, before → after, identical in bare and `--preload=ar5iv.sty` mode:
2605.00480 **1 → 0**, 2605.00698 **1 → 0**, 2605.00721 **1 → 0**, 2605.01187
**2 → 1** (residual `undefined:\bstctlcite`, unrelated), 2605.05692 **2 → 0**,
2605.18923 **1 → 0**, 2605.26747 **2 → 0**.

Guards: `06_cluster_frontmatter::frontmatter_spconf_keywords`,
`frontmatter_spconf_keywords_braced`, `frontmatter_spconf_twoauthors` (all via
`convert_to_xml_contrib_clean`, so a returning error fails them).

### 83. acmart `\Description` becomes the image's text alternative

acmart documents `\Description` as "used **instead of** the image" (unlike
`\caption`, "used alongside" it), so it is a *text alternative*, not
supplementary prose. Perl (`acmart.cls.ltxml` L78-86) emits `#1` — the
**optional short** description — into `<ltx:note class="ltx_nodisplay">` and
points `aria:labelledby` at it. `#1` is the optional argument and `#2` the
mandatory one, so the long description is digested and then discarded
(`\Description{L}` produced no output at all). Recorded as
`KNOWN_PERL_ERRORS.md` #66.

The thing a `\Description` is an alternative **to** is the image, so that is
where it lands — as `@alt`, via `ltx:graphics/@description`
(`LaTeXML-misc-xhtml.xsl` L167-171):

| source | HTML |
|---|---|
| `\Description[s]{l}` | `<img alt="s" aria-describedby=`→`l>` |
| `\Description{l}`, `l` plain | `<img alt="l">` (it replaces the image) |
| `\Description{l}`, `l` with markup | `<img aria-describedby=`→`l>`, alt unchanged |
| any, when `\includegraphics[alt=…]` is also present | author's alt kept, both notes referenced |

`[short]` is the concise alternative and `{long}` the extended description, so
`@alt` / `aria-describedby` is their natural pairing. A lone description takes
the `@alt`, because it is what stands in for the image — unless it carries
markup, which an attribute cannot hold, and the generic `alt` fallback ("Refer
to caption") stands instead.

**Not `aria:label` on the `<ltx:figure>`**, which an earlier revision of this
divergence used. `aria-label` sets the accessible **name**, and a float's name
is its caption, so labelling the figure with the description displaced
"Figure 1. caption text" and hid the caption from a screen reader — reported in
review on brucemiller/LaTeXML#430 (`r3674103638`), which also asked for the
`<img>` to receive `@alt` and not `@aria-label`. Nothing in this binding emits
`aria-label` any more.

Three shapes have no image to use. The author's annotation is never dropped, so
it goes to the next best host — the enclosing element — as `aria:describedby`,
which supplements the name rather than replacing it, so the caption survives
either way. All three **`Warn!`**, naming the actual cause, since the result is
second-best and the author can act on it:

* **no `ltx:graphics` in the float** — a figure built from tabular, text or
  TikZ content (which `t/complex/acm_aria` is), a `table` float, or an empty
  one. There is no image to be an alternative to.
* **more than one** — a `\Description` is scoped to the whole float, so on a
  multi-panel figure it describes the ensemble. Making it panel 1's `@alt`
  would assert that one sentence is the alternative for one panel, a claim the
  author never made. The review says "the first image"; we narrow that to the
  case where "first" is also "only", where it is unambiguous.
* **outside any float** — a bare `\Description` in running text lands on
  whatever element encloses it (a `<p>`, typically). Nothing to describe, but
  the text is still carried.

References ACCUMULATE rather than overwrite (`add_describedby`):
`aria-describedby` is an id list, and a second `\Description` in the same float
would otherwise write straight over the first one's reference, leaving that
description sitting in the DOM announced by nothing — losing an annotation the
author wrote.

Only graphics **already built** are visible when `\Description` is constructed,
so a `\Description` written *before* its `\includegraphics` falls into the
first case. That is the safe direction to fail — the description is still
announced, just not as the image's alternative — the warning names it as a
possible cause, and acmart's own documentation puts `\Description` after the
graphic.

Choosing between the slots is why the argument is read **`Undigested`**
(`ExpansionLevel::Off`): the tokens must be inspected for control sequences
*before* anything expands. That also means nothing inside a `\Description` is
ever expanded — matching `acmart.cls` L895, which gobbles the argument, so
pdflatex never expands it either and an author cannot see a defect there.
arXiv:2607.21760 went from `1 error` (an `Error:undefined:\D` on a copy-paste
slip inside `\Description`) and **zero** figure descriptions, to **zero errors**
and four descriptions in its HTML.

Both descriptions are emitted as separate `ltx:note`s with their own `xml:id`
and class (`ltx_acm_description_short` / `ltx_acm_description`) — two distinct
authored fields, so concatenating them into one element would produce a run-on
no consumer could take apart. A block is referenced only when its text is not
already the `@alt`, so the same sentence is never announced twice; an
unreferenced hidden block is inert, since `display:none` content is announced
only when something references it. Where both are referenced,
`aria-describedby` takes a space-separated id list announced in order, short
first.

A dedicated template in `LaTeXML-meta-xhtml.xsl` strips the footnote
scaffolding — the generic `ltx:note` rendering adds a `†` mark and a
`<role>: ` prefix, which landed in the computed accessible text ("†† : …") —
and drops the `ltx_note` class, which these are not. Perl's `width`/`height`
`Dimension(0)` are carried over unchanged.

acmart's newer mechanism for the same purpose, `\includegraphics[alt=…]`
(switched on by `\DocumentMetadata`), is handled separately in `graphicx_sty.rs`
and sets the same `description` attribute; we accept it unconditionally rather
than gating it behind `\DocumentMetadata`, which is itself a no-op
(`latex_constructs_rust_only.rs`). When an author uses BOTH — e.g.
arXiv:2607.21760, which repeats the same paragraph in each — the explicit
`alt=` **wins**: it names one image, while `\Description` names the float, so
the more specific statement stands and `\Description` only adds its
`aria-describedby` references.

Guard: `latexml_oxide/tests/complex/acm_aria.{tex,xml}` (re-blessed — it
previously matched Perl byte-for-byte and so certified the defect; it has no
graphics, so it pins the float-level branch) plus
`110_acmart_description_aria.rs`, which drives six figures — one per branch
above — through to HTML and asserts that each lands where the table says, that
`aria-label` appears nowhere, that captions survive, and that no
`aria-describedby` reference dangles.

### 84. Bibliography sort keys collate at UCA's PRIMARY level, not by full UCA

Perl's `Post::unisort` (`Post.pm` L1399-1403) sorts the bibliography sort keys
with a `Unicode::Collate::Locale` built from the document's `xml:lang` and
configured `variable => 'non-ignorable'`, `upper_before_lower => 1`. The Rust
port called `Vec::sort()` — plain codepoint order — so every non-ASCII surname
was exiled past `z`: on `bib_alpha_style.tex`, `Ångström` sorted **after**
`Smith` where Perl (and every real `.bst`) puts it between `Adams` and `Baker`.

`make_bibliography.rs::unisort` now collates. It reproduces UCA's **primary**
level only — NFD-decompose, drop combining marks, case-fold — and breaks ties on
the raw key. That is **exact** for accented Latin, which is what these keys
actually contain.

`upper_before_lower` needs no counterpart at all here, and the honest reason is
that it is **moot**, not that the tie-break reproduces it: `getBibEntries`
lowercases the whole sort key before it is ever stored
(`format!(...).to_lowercase()`), so no comparison this function performs can
see a case difference. Codepoint order on the raw key does happen to sort
uppercase first, but that property is never exercised.

It **diverges** from Perl for: orders that cross scripts, letters with no
canonical decomposition (`Ø`, `Æ`, `Ł`, `Đ`), and locale tailorings (Swedish
sorts `Ö` last, German does not). Closing those means a DUCET table, i.e. a new
dependency shipping embedded collation data — declined on the standing
dependency-conservatism rule, and the approximation stays inside the range Perl
itself ships: `Post.pm` L123-128 falls back to a codepoint `DumbCollator`
whenever `Unicode::Collate` is not installed, which is strictly worse than this.

Guard: `06_cluster_bibliography::cluster_bib_alpha_style_labels`, whose expected
order was ground-truthed against same-host Perl LaTeXML 0.8.8 on the fixture.

### 85. `\fnum@<type>` is expanded with an empty group, so an arg-taking author redefinition cannot eat the caption's closing brace

**Perl behavior.** `\lx@fnum@@` (`Base_Utility.pool.ltxml` L1041-1043) expands
the author's hook bare — `\@ifundefined{fnum@#1}{\lx@@fnum@@{#1}}{\csname
fnum@#1\endcsname}`. Rust's definition was byte-identical.

**Rust behavior.** The same, plus a trailing empty group: `{\csname
fnum@#1\endcsname{}}`. Applied at all three `fnum@` hook sites —
`\lx@fnum@@` and `\lx@fnum@toc@@` (`base_utilities.rs`) and the theorem-header
formatter (`latex_constructs.rs`).

**Why.** Real `\fnum@<type>` takes no argument, but LaTeX's `\@makecaption` is
`\sbox\@tempboxa{#1: #2}`, so a *one-argument* `\fnum@<type>` eats the `:` that
follows it. That is a widely-copied author hack — "change `Fig. 1:` to
`Fig. 1.`":

```tex
\makeatletter
\renewcommand*{\fnum@figure}[1]{\figurename~\thefigure.}
\makeatother
```

pdflatex accepts it and prints `Figure 1. A caption.` LaTeXML has **no `:` token
to eat** — its separator is a tag ATTRIBUTE (`\lx@tag[][: ]`,
`latex_constructs.pool.ltxml` L3158-3159) — so the argument scan ran past the
hook and swallowed the caption group's closing brace. The `<figure>` then never
closed and **every following section, the bibliography included, was absorbed
into it**: to a reader the document is truncated. The empty group gives an
arg-taking hook something harmless to consume, reproducing pdflatex's result,
and is inert for the 0-arg hooks that are the normal case (`\fnum@subfigure`,
`\fnum@lstlisting`, `\fnum@sidebar`, `\fnum@ALC@line`, `\fnum@equation`, and the
dynamic ones `enumitem`/`newfloat` create).

Not a TeX-semantics change: it does not redefine argument scanning, and the
`\lx@@fnum@@` default branch — what fires when no `\fnum@<type>` exists at all,
i.e. for nearly every figure and table caption — is untouched.

**What this does NOT buy.** The rendered separator still comes from the tag's
`close=": "` attribute, so the caption reads `Figure 1.: A caption.` rather than
pdflatex's `Figure 1. A caption.` The divergence buys **error-freedom and an
un-truncated document**, not punctuation parity. Suppressing the attribute when
the hook is arg-taking would be a second, far more speculative change and is
deliberately not part of this one.

**`\lx@typerefnum@@` has the identical shape and is deliberately NOT changed.**
`typerefnum@<type>` is a LaTeXML-internal hook with no LaTeX kernel behind it,
so no author writes an arg-taking version to eat a separator token that LaTeXML
never emits. The pdflatex-compatibility argument does not reach it, and without
that argument the change would be speculative.

**Witnesses.** `2605.01731` (cas-sc, 18 figures x 3 errors -> body collapsed to
one section, 19 `<bibref>` but no `<bibliography>` element) and `2605.12842`
(10 x 3). `cas-sc` is NOT implicated — plain `article` reproduces; that was the
first hypothesis and it was wrong.

**Breadth — re-measured live 2026-07-29, and the recorded figure does NOT hold.**
The 2026-07-14 note claimed "18 papers corpus-wide" from a `grep 'lx@tag@intags'`
proxy. Against the current fleet run that proxy yields **23 papers** across
sandbox-arxiv-2605 (9) + 2606 (14), 60,505 documents — but only **2** of the 23
carry the signature this cause actually produces (equal counts of
`unexpected:\lx@tag@intags`, `unexpected:\lx@tag` and `unexpected:\end{figure}`,
one triple per figure): **2605.01731** (18 figures x 3) and **2605.12842**
(10 x 3). Several more match partially (2606.06276 18/18 but no `\end{figure}`;
2606.18583, 2606.23565). So `\lx@tag@intags` is a **shared symptom with multiple
causes** and over-attributes to this one; the "5 of them with no References"
sub-claim is likewise unverified and is withdrawn. Witness 2605.01731 itself is
confirmed live on the fleet binary with exactly the recorded 18x3 signature.

**Measured.** Guard fixture `cluster_regressions/fnum_arg_hook.tex`, which
exercises all three hooks: **10 errors -> 0**, and the bibliography stops being
absorbed into the unclosed `<figure>`. On the two-hook minimal form, same-host
Perl 0.8.8 raises **9** errors and pre-fix Rust raised **7**; pdflatex raises
**0**. Full suite unchanged by the divergence — 106/106 targets, no golden
re-blessed.

**Upstream.** Perl's definition is byte-identical, so the same one-token fix
applies there — filed as **brucemiller/LaTeXML#2856**. Also
`KNOWN_PERL_ERRORS.md` #68.

Guard: `06_cluster_regressions::cluster_fnum_arg_hook`.

### 86. `\bibliography{}` with an empty argument still inputs `\jobname.bbl`

`latex.ltx` ends `\bibliography` with an **unconditional**
`\@input@{\jobname.bbl}` — the argument only drives the `.aux` `\bibdata`
record, not whether the `.bbl` is read. So `\bibliography{}` beside a shipped
`<jobname>.bbl` renders the full reference list under pdflatex (measured:
"References / [1] A. Uthor. A paper. 2020.", with `\cite` resolving to `[1]`).

Perl returns before looking at anything (`latex_constructs.pool.ltxml` L3901,
`return unless $bib_files;`) and raises no diagnostic, so the references are
dropped in silence; this port mirrored that. Rust now follows `latex.ltx`
instead: on an empty argument it takes the `.bbl` branch **when
`\jobname.bbl` actually exists**, and otherwise still returns quietly.

Ground truth is the arXiv PDF (bibliography formats are config-driven —
`BIBLIOGRAPHY_WORKLIST.md`), and this is the shape where Rust and Perl agreed
with each other but not with the PDF. 7 papers in the 2605+2606 sandboxes,
including the GWTC-5 LIGO set, which share one template that writes
`\bibliography{}` and ships the `.bbl`. Witness **2605.27226**; repro
`docs/parity/bib_absence_2026-07-29/repros/f3_empty_arg_bbl/`; audit family
F3(a) in [`BIB_ABSENCE_AUDIT_2026-07-29.md`](BIB_ABSENCE_AUDIT_2026-07-29.md).

Guard: `bib_empty_argument_still_reads_the_jobname_bbl`.

### 87. bibunits' `\putbib` inputs the per-unit `bu<N>.bbl`

Same shape as #86, one level down. The real package
(`bibunits.sty` L324-330) writes the optional argument to the bibunit `.aux`
as a `\bibdata` record and then runs `\@input@{\@bibunitname.bbl}`
**unconditionally** — the argument never decides whether the `.bbl` is read.

Perl's binding (`bibunits.sty.ltxml` L78) expands `\putbib[#1]` to
`\lx@bibliography[\bu@unitname]{…}`, i.e. the `.bib` route only. arXiv
submissions ship the *generated* `bu1.bbl`/`bu2.bbl` precisely because arXiv
does not run bibtex, and the named `.bib` is usually absent — so the lookup
finds nothing and the References section renders empty. Rust prefers the
shipped `.bbl` and keeps Perl's route as the fallback.

15 papers measured across the 2605+2606 sandboxes, every one 0 entries before
and complete after: 2606.04416 (79), 2606.28854 (180), 2605.21570 (46 = bu1's
34 + bu2's 12, each matching its unit's own `\begin{thebibliography}{N}`).
Audit family F3(c) in [`BIB_ABSENCE_AUDIT_2026-07-29.md`](BIB_ABSENCE_AUDIT_2026-07-29.md).

Guard: `bibunits_putbib_reads_the_per_unit_bbl`.

### 88. The core `\cite` is locked against raw `.sty` redefinition

arXiv submissions ship their conference style, and under `--includestyles`
(the fleet's ar5iv profile) it is raw-loaded. aaai, iccc, flairs, kr,
achicago, harvard and fixbib all `\def\cite`, and the replacement records no
citation — so `MakeBibliography` selects nothing and the document renders an
empty References section under bold `?` markers: *"N bibentries, **0 cited**"*.

`\cite` now carries `locked => true`, exactly as natbib locks its own variants
(`natbib.sty.ltxml` L151, L191, L225). Raw `.sty`/`.cls` and document-source
redefinitions are ignored; native bindings still override freely, because
binding loads run UNLOCKED (Perl `Package.pm:loadLTXML` L2318 →
`local_state_unlocked_guard` in `binding/content.rs`).

**Deliberately beyond both Perl and the PDF.** Perl raw-loads the same styles
and loses the citations identically. The PDF has no reference list either —
these submissions ship no `.bbl` and arXiv does not run bibtex — so the styles'
own `\cite` never had a bibliography to point at. Keeping LaTeXML's semantic
`ltx:cite` is what makes the HTML usable: the citation resolves, the reference
list renders, and only the citation *label style* differs from what that
class would have printed.

13 of 15 measured witnesses recovered, every one 0 entries before: 2605.07102
(0 → 50), 2606.21959 (0 → 54), 2605.00671 (0 → 44, the `\affiliations`
cluster), 2606.29340 (0 → 40), 2605.09519 (0 → 24). Audit family F9(a) in
[`BIB_ABSENCE_AUDIT_2026-07-29.md`](BIB_ABSENCE_AUDIT_2026-07-29.md).

**Exception — non-destructive etoolbox hooks assign through the lock
(2026-08-05).** `\pretocmd`/`\apptocmd` (and the param-branch `\etb@hooktocmd@i`)
re-`\edef`/`\let` the target but preserve the original via `\expandonce#2`, so
they only WRAP a definition — they cannot displace it the way a raw `\def\cite`
does. Refusing them silently drops legitimate hooks: witness ar5iv 2606.01320
does `\pretocmd{\cite}{\stepcounter{cite}}` then gates its whole bibliography on
`\ifnum\value{cite}>0`, and the refused assignment left the counter at 0 and the
References vanished with no diagnostic. The etoolbox binding therefore opens a
scoped unlock window (`state_is_unlocked()`) around JUST the hook's assignment to
`#2` — `\lx@etb@unlock`/`\lx@etb@relock` in `etoolbox_sty.rs`; a plain
`\def`/`\renewcommand` from raw source is outside the window and stays refused.

Guards: `bib_raw_cite_redefinition_is_ignored` (raw redefinition refused) and
`etoolbox_pretocmd_assigns_through_cite_lock` (non-destructive hook allowed).

### 89. `\captionof` does not open a verbatim-bodied environment

Perl hosts the faked caption inside the named environment — *"it isn't
necessarily IN a figure or any float, so we'll wrap it in an otherwise empty
one!"* (`caption.sty.ltxml` L124-125, `\@captionof@` →
`\begin{#1}…\end{#1}`). That is fatal when the environment reads its body
verbatim: `\captionof{lstlisting}{…}` becomes
`\begin{lstlisting}…\end{lstlisting}`, and listings scans the **raw input**
for its terminator, never the token stream. It finds none, and swallows the
rest of the file — the document tail, `\bibliography` included, comes out as
line-numbered listing text.

Real caption.sty never opens the environment: `\caption@of` is
`\setcaptiontype*{#2}#1` (caption.sty L391) — it only sets the caption type.
So for the verbatim-bodied environments (`VERBATIM_BODY_ENVS` in
`caption_sty.rs`: lstlisting, verbatim, fancyvrb's Verbatim family, minted,
alltt) Rust wraps the caption in the kernel's generic `{@float}{<type>}` (Perl's
`\@float`) instead: a float of that type, numbered by its own counter where it
stands — its tags, its id (`LST1`), its list entry (`lol`) and the `\label`
after it — which `collapse_float` folds into a captionless enclosing figure or
minipage (57bu re-review; 2606.08339 re-converted: the two listing labels, which
had leaked onto Figure 5, now name `LST1`/`LST2`, 30 bibliography entries kept).
Every other type keeps Perl's own-environment wrapper, since that is what gives
an unfloated `\captionof{figure}` its container. Guard
`perfect_kernel_gemini::captionof_verbatim_type_numbers_its_own_counter`.

Witness **2606.08339**: one `\captionof{lstlisting}{PROMISE.yml}` cost the
paper all 30 of its bibliography entries (measured 0 → 30, and the swallowed
tail restored). pdflatex renders that paper correctly and its source has four
balanced `lstlisting` pairs, so the runaway was ours. Audit family F5 in
[`BIB_ABSENCE_AUDIT_2026-07-29.md`](BIB_ABSENCE_AUDIT_2026-07-29.md); reduction
notes in `bib_absence_2026-07-29/repros/f5_captionof_swallow/`.

Guard: `bib_captionof_verbatim_env_does_not_swallow_the_bibliography`.

## Known Upstream Perl Issues (brief)

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

### 90. Tab marks are suppressed while a macro argument is scanned

`tex.web` §394 `macro_call` sets `align_state:=1000000; {disable tab marks,
etc.}` before it scans a macro's parameters, so a `&` or `\cr` **inside an
argument** is an ordinary token and cannot end an alignment cell. Neither Perl
LaTeXML nor this port modelled that: the cell break is decided in `readToken`
purely on `ALIGN_STATE == 0` (`Gullet.pm` L320-324, ported at
`gullet.rs` L837/L1043), so an argument's `&` reached the alignment.

It only bites for a **delimiter-fenced** argument. `\mqty{a &b}` was always
safe because cell scanning skips balanced groups — but `(…)` is not a group, so
`\mqty( b_0 &0 \\ 0 &b_1 )` inside an `eqnarray` split the row mid-argument,
orphaned the `\left(`/`\right)` fences, and the alignment could then not close
its own group:

```
Error:unexpected:\lx@begin@alignment Attempt to close a group that switched to
mode restricted_horizontal
```

The document was truncated there, so the bibliography went with it. All three
conditions were needed: the fenced form, at least one `&` inside it, and an
enclosing row with no `&` of its own.

Rust arms a `SuppressedTabMarks` guard (`common/local_assignments.rs`) for the
duration of a **custom delimited read** — currently physics.sty's
`phys_read_arg`, which is where `\mqty` and friends consume their fenced body.
It is only armed inside an alignment, so ordinary macro calls keep their hot
path.

**Scope is deliberately narrow.** Arming the same guard in
`Parameters::read_arguments` — TeX's actual `macro_call` site, which would also
cure a plain `\def\myfence(#1){…}` — regresses **5 tests**: `cells_test`
(17 errors), `numprints_test`, `xytest_test`, `consort_flowchart_test`,
`unit_tests_by_silviu_test`. That path is *also* how an alignment reads its own
cell content here, so suppressing tab marks across it stops cells terminating.
Curing the general case needs a way to distinguish a macro's parameter scan from
a cell-content read; until then a fenced `&` in a plain user macro still splits
the row (reproducer: `\def\myfence(#1){\left(\begin{array}{cc}#1\end{array}\right)}`,
12 errors).

**Beyond Perl.** Perl raises the identical error — 11 of them on the 14-line
reproducer `repros/f7_alignment_fenced_amp/mqty_in_eqnarray.tex`, tail lost —
so `pdflatex`, which renders it silently, is the ground truth. Witnesses
**2605.05903** and **2007.06211** (revtex4-1 + physics). This was the largest
single cluster of the 2026-07-29 bibliography-absence residual: **28 papers**,
of which **14** recovered here (961 entries); the remaining 14 all still fail on the same `\lx@begin@alignment` via the general parameter path. Guard
`alignment_fenced_amp_does_not_split_a_row`.

### 91. Equationgroup ids minted at digest time (ancestry-consistent prefixes)

Perl mints the `@equationgroup` id (`RefStepID`) inside the alignment's
container-open hook — at **absorb** time, after all digestion — so every
group's id carries the **last** section's prefix: `LaTeXML/t/structure/
eqnums.xml` stamps `S3.EGx1` on a group whose own rows are `S2.E*`. That
violates the id design's own intent (in-order counter increase, id fragments
mirroring the node's XML ancestry), and it made the eager and streaming
(interleaved digest/build) pipelines disagree, since streaming absorbs with
the *current* section counter.

Rust mints at **digest** time in both pipelines (`latex_constructs.rs`
eqnarray, `amsmath_sty.rs` gather/align): prefixes now match the group's
actual ancestry (`S2.EGx1`, `S2.EGx2`, `S3.EGx3`, …) with the counter still
strictly in document order. User ruling 2026-07-29. Goldens updated:
`structure/eqnums`, `structure/amsarticle`, `math/sampler`, `ams/mathtools`.

### 92. Display equations carry a 1em vertical CSS margin

Vanilla `LaTeXML.css` (upstream master included, checked 2026-08-01) gives the
display-math containers no vertical margin: `.ltx_eqn_table` is
`display:table; width:100%; border-collapse:collapse;` (L244) and
`.ltx_eqn_div` (the unaligned rendering, L241) likewise has none. Text
paragraphs get their spacing from the UA's `p { margin:1em 0 }` collapsing
through `div.ltx_para`, so two adjacent displays render **touching** (issue
#473 MWE: two `\[…\]` in a row) — identically in Perl 0.8.8 and Rust
(same-host check: same body elements and classes, differing only in
whitespace serialization and the sanctioned #18 invisible-operator; same
CSS). pdflatex is the ground
truth and separates displays with `\abovedisplayskip`/`\belowdisplayskip`
≈ 1em of the body font (10pt @ 10pt, 12pt @ 12pt).

The bundled stylesheet adds a local delta:
`.ltx_eqn_table, .ltx_eqn_div { margin-top:1em; margin-bottom:1em; }`.
`1em` (not ar5iv's `0.65rem`) because it collapses with the UA `<p>` margins
through the `ltx_para` wrappers: measured headless-Chrome gaps
(text→eqn / eqn→eqn / eqn→text) go 16/0/16 px → uniform 16/16/16 px, i.e.
the text↔display gap is unchanged (no doubling) while display↔display gains
the paragraph rhythm. Guard:
`witnessed_css_delta::equation_display_margin_delta_stays_present`
(`latexml_post/src/xslt.rs`).

### 93. Verbatim renders true-to-source: `white-space:pre` + one line per row

Vanilla `LaTeXML.css:454` sets `.ltx_verbatim { text-align:left;
white-space:nowrap; }`. Because author CSS beats the UA stylesheet, that
`nowrap` overrides `pre { white-space:pre }`, so a plain `{verbatim}` block
collapses to a **single line** (measured: the 4-line `<pre>` renders 15 px
tall, one line, headless Chrome 2026-08-02), and fancyvrb's per-line spans
lose leading indentation and runs of spaces, flowing side-by-side two-up in
a wide window. Perl 0.8.8 renders identically (same markup, same CSS);
pdflatex is the ground truth and preserves all of it. ar5iv fixed the `<pre>`
half downstream by dropping the `nowrap` (`ar5iv-css/css/ar5iv.css:2949`).
Recorded as KNOWN_PERL_ERRORS #71.

The bundled stylesheet adds a local delta (issue #431):
`.ltx_verbatim { white-space:pre; }` (keeps nowrap's no-wrapping, restores
newlines/indentation/spacing) and
`.ltx_text.ltx_verbatim.ltx_inline-block { display:block; text-indent:0;
min-height:1lh; }` (one fancyvrb source line per row, blank lines keep a
line's height, no inherited paragraph indent). Only the fancyvrb binding
puts `ltx_verbatim` on an inline-block text span, so inline `\verb`
(`<code>`, no `ltx_inline-block`) is untouched. Measured after: each line on
its own row, `    print(i)` keeps its 4-space indent, `<pre>` renders 4
lines. Guard: `witnessed_css_delta::verbatim_whitespace_delta_stays_present`.

One serialization nuance, same entry: Perl nests two wrappers per line
(`<text font="typewriter" width="345.0pt"><text class="ltx_verbatim"
width="345.0pt">…`), an artifact of its box construction; Rust's
`\lx@add@cssclass` — like Perl's, "add class to the current element" —
merges onto the one line box: `<text class="ltx_verbatim" font="typewriter"
width="345.0pt">…`. Same attributes, same semantics, one element instead of
two; no upstream `t/` golden pins the nested form. Golden:
`tokenize/fancyvrb.xml`.

### 94. Post-processing errors cap at MAX_ERRORS; the cap latches instead of dying

Perl's error cap lives in `Common/Error.pm:372` behind `$STATE &&` — and Post
runs with `$STATE` undef, so Perl post-processing neither counts toward nor
triggers `too_many_errors`, and a post error storm runs unbounded. Since the
single-vehicle diagnostics rework (#484 + the cap-latch fix), Rust post
`Error!` flows through `emit_error`, which applies the same MAX_ERRORS (100)
and consecutive-error (500) caps as the core phase. Second divergence in the
same seam: on crossing a cap, Perl `Fatal` **dies**; `emit_error` has no
unwind channel (its callers return `String`/`Option`, not `Result`), so it
emits `Fatal:TooManyErrors`, latches the sticky fatal — the run **continues**
and keeps writing pages, but the verdict, exit code, and
`Status:conversion:3` all report the fatal. Rationale: a partial site plus an
honest fatal beats Perl's nothing-plus-death; the cap keeps a post storm from
producing a million-line log. The latch is guarded by the sticky fatal (fires
once, robust to skipped checks), and the runaway (consecutive) diagnosis is
tested before the total cap so its message is reachable. Guards:
`suppression_never_mutes_error_or_fatal`, `119_final_status_report`.

### 95. `xkeyval` really loads `keyval`, instead of only pretending to

Perl's `xkeyval.sty.ltxml` L23 sets `AssignValue('keyval.sty_loaded' => 1,
'global')` — "pretend keyval loaded too" — so that keyval's plain
`\setkeys`/`\define@key` can never clobber xkeyval's extended ones. But that
flag is what `Package.pm:loadLTXML` L2328-2330 and `loadTeXDefinitions` L2363
gate on, so it also suppresses the **raw** `keyval.sty` that
`keyval.sty.ltxml` reads via `InputDefinitions('keyval', noltxml => 1)`. Every
keyval internal lives only there: `\KV@do`, `\KV@split`, `\KV@errx`,
`\KV@@sp@def`, … No binding defines them, in either engine.

Raw packages call those internals directly — `fancyvrb.sty` L112-117
`\FV@UseKeyValues` expands `\KV@do` — so after xkeyval, `\KV@do` is undefined
and `\DefineVerbatimEnvironment` reports `Error:undefined:\KV@do` (issue #500).
Perl behaves identically once xkeyval is loaded first (KNOWN_PERL_ERRORS #73);
Rust reaches it on the reporter's `standalone`-then-`fancyvrb` MWE because
`standalone_sty.rs` carries the beyond-Perl `RequirePackage!("xkeyval")` of real
`standalone.sty` L107, which Perl's `standalone.sty.ltxml` omits.

**Rust behavior**: `xkeyval_sty.rs` opens with `RequirePackage!("keyval")` and
drops the pretense. Real `xkeyval.sty` L39 `\input xkeyval` pulls in the
bundle's own `keyval.tex`, which defines `\KV@do` at L52 — loading xkeyval in
real LaTeX genuinely provides keyval, so this is the faithful shape, not an
invention. Ordering is real xkeyval's: keyval first, xkeyval's extended
definitions after, so xkeyval still wins; and since `RequirePackage` sets
`keyval.sty_loaded` itself, a later `\RequirePackage{keyval}` remains the no-op
the pretense was protecting.

**Witnesses**: issue #500's MWE (`standalone` + `fancyvrb` +
`\DefineVerbatimEnvironment`); the same MWE with an explicit
`\usepackage{xkeyval}`, which Perl 0.8.8 also fails.

**Upstream**: <https://github.com/brucemiller/LaTeXML/issues/2864>.

**Guard**: `06_cluster_standalone_subfiles::keyval_internals_survive_xkeyval_preloading_it`.
### 96. A faked space is sized by the font it was DIGESTED in, not the ambient one

`tex_glue::dimension_to_spaces` renders a width as a run of Unicode space
glyphs, and it does the arithmetic in **ems** — so which font supplies the em
decides the answer completely.

**Perl behavior**: `TeX_Glue.pool.ltxml` L44 reads the live
`LookupValue('font')`. In a `DefConstructor` that is the font at CONSTRUCTION
time, and Perl builds only after the whole document is digested — so it is
whatever font the document *ends* in. Appending `\small` before
`\end{document}` changes the glyph chosen for a skip that occurred pages
earlier (verified same-host on the witness below: `U+2009` → `U+2004`).

**Rust behavior**: the constructor and whatsit call sites (`\hskip`, `\kern`,
`\lx@text@intercol`, `digested_to_text`) pass the whatsit's own
`props["font"]` / `get_font()`. Digest-time callers (`\hspace`, `\hglue`, the
tabskip Tbox) keep the ambient read, where `lookup_font()` already IS the
digest font. One site keeps the ambient read for lack of an alternative:
`cleanup_math`'s `<ltx:XMHint>` records only `width`, never a font.

**Why**: the ambient read made the output depend on WHEN the build ran, which
broke the eager/streaming byte-identity invariant outright — streaming builds
mid-document, so it read the local font and emitted `U+2004` where the eager
path emitted Perl's `U+2009`. There is no way to make streaming reproduce
"the font the document will end in"; the only deterministic choice is the
font the skip actually occurred in. That is also the *correct* one: the glyph
run is an approximation of a fixed pt width, and it is only a good
approximation when measured in the font that will render those glyphs.

**Witness**: `latexml_oxide/tests/cluster_regressions/fancyvrb_fontsize_numbers.tex`
— a `numbers=left` line-number skip, digested in the number's `56%` font,
inside a `fontsize=\small` verbatim, inside a default-size document: three
distinct sizes, so every candidate font gives a different answer. Rust emits
`1␣␣␣` (`U+2003 U+2003 U+2004`); Perl emits `1␣␣` (`U+2003 U+2009`).

**Cost**: zero golden churn — the full suite was 1885/1885 with no fixture
updates, so no existing test exercised a skip whose digest font differed from
the document's final font.

**Guards**: `06_cluster_regressions::faked_space_is_sized_by_the_font_it_was_digested_in`
pins the value; `114_streaming_cluster_regressions::streaming_matches_eager_on_cluster_regressions`
pins eager == streaming (it is what caught the defect).

### 97. `\hrulefill`/`\dotfill` restore the LaTeX kernel's `\leavevmode`

**Perl** (`plain_constructs.pool.ltxml` L86-87): `\hrulefill` → `\leaders\hrule\hfill`,
`\dotfill` → `\leaders\hbox{.}\hfill` — dropping the leading `\leavevmode` and
trailing `\kern\z@` of the real LaTeX kernel (`latex.ltx` L643-644:
`\def\hrulefill{\leavevmode\leaders\hrule\hfill\kern\z@}`).

**Rust** uses the kernel definitions verbatim (with `\leavevmode\…\kern\z@`).

**Why**: `\hrule` is a vertical-mode command and the fill leader is horizontal,
so LaTeX enters horizontal mode FIRST (`\leavevmode` starts the paragraph). Only
then does a following `\vskip`/`\vspace*` fire TeX's `head_for_vmode` (tex.web
L21160 `hmode+vskip: head_for_vmode`) — the internal `\par` that ends the
paragraph. Perl gets away without the `\leavevmode` because `\hfill`'s
`enterHorizontal` (an `inplace` MODE assignment) persists past `\leaders` (a
`bounded` constructor); Rust's `bounded` reverts it, so after `\hrulefill` the
mode was `internal_vertical`, `leaveHorizontal` never fired, the leader's
`<ltx:p>` never closed, and the next float panel merged into it — a
schema-invalid `<caption>`-in-`<block>` (arXiv **2302.11635**), and, more
generally, a *silent* paragraph-merge in any float with `\hrulefill\vspace*`
between rows. Restoring the kernel definition fixes the root faithfully; where
`\hrulefill` is used mid-line (already horizontal) the `\leavevmode` is a no-op,
so output is byte-identical to Perl.

**Cost**: no golden churn — the `\leavevmode` only bites in vertical contexts,
where it makes Rust match BOTH Perl's output and the LaTeX kernel. 2302.11635:
4 errors → 0, 10 `<figure>` / 0 `<block>` (Perl-identical structure).

**Guards**: `50_structure::vspace_closes_leader_para_test` (the `\vspace*` vs
`\par` control pair). Cross-ref WISDOM #38.

### 98. `\autoref` feeds the number as `\<type>autorefname~<number>\null` (active `~`), enabling the delimited-arg idiom

**Perl** (`hyperref.sty.ltxml` L373-382, `\lx@autorefnum@@`): builds the autoref
text as `\<type>autorefname \nobreakspace \the<counter>` — a *control-sequence*
separator and **no trailing `\null`**.

**Rust** (`hyperref_sty.rs` `\lx@autorefnum@@`) instead mirrors the real hyperref
kernel (`hyperref.sty` L8211-8268, `\HyRef@autosetref`/`\HyRef@testreftype`),
feeding `\<type>autorefname` + the **active `~`** (catcode 13, `\noexpand~`,
L8247) + the number + **`\null`** (L8226).

**Why**: the hyperref manual documents the delimited-argument idiom
`\def\equationautorefname~#1\null{(#1)\null}` to wrap an equation's autoref number
in parens → `(1.1)`. It works only if the stream after `\equationautorefname` is
`~<number>\null` with the *active* tilde as the delimiter and `\null` as the
terminator. Perl's `\nobreakspace` (a CS) never matches the `~` delimiter and the
missing `\null` is never found, so **both** engines emitted the broken
`() 1.1` (empty parens + a dangling number). This is a SHARED-FAILURE that Perl
LaTeXML also exhibits; the user sanctioned surpassing it.

**Cost**: no golden churn on the default path — active `~` is bound to `\lx@NBSP`
(the same nbsp `\u{00A0}`@0.333em as `\nobreakspace`→`\lx@nobreakspace`) and the
trailing `\null` (=`\hbox{}`) digests to nothing, so `Equation 1.1` / `section 1`
render unchanged (`50_structure::autoref_test`).

**Guards**: `50_structure::autoref_delimited_test` (the `\def\equationautorefname~#1\null`
idiom → `(1.1)`/`(1.2)` while a sibling `section` autoref stays `section 1`);
`50_structure::autoref_test` (default path unchanged). Witness: ar5iv #607
(arXiv **2607.12124**).

### 99. `geometry` sizes measured SVG graphics (not the HTML flow)

**Perl** (`geometry.sty.ltxml` L27-59) makes every geometry macro a no-op —
*"AND, in the end, they're all ignored!"* — so `\textwidth` keeps the class
default (345pt) no matter what margins the document asks for. That is correct
for the **reflowable HTML body**: page geometry is meaningless when the browser
reflows the column. Rust keeps it for the flow — a `\rule{0.5\linewidth}`, a
text minipage, `\includegraphics[width=\linewidth]` are all class-default-sized,
byte-identical to Perl.

**Rust divergence**: a *measured SVG graphic* that reads `\linewidth` — a
`tcolorbox`, `tikzpicture`, or bare `pgfpicture` — is emitted as a fixed-size
`<svg>` whose aspect ratio is **baked at conversion time**. Ignoring the real
page width there makes such a box `0.495\linewidth = 0.495 × 345pt` instead of
the `0.495 × 472pt` the PDF draws (letterpaper − the class's 2.5cm margins). The
too-narrow interior over-wraps the content, ≈doubling the box height (aspect
2:1 vs the PDF's ~4:1) and pushing text through the border. Witness
**arXiv:2605.29955** Fig 1 (two side-by-side statement/proof cards). Perl has
the same 2:1 boxes — it is a shared under-fidelity, not a Rust regression — so
this is a beyond-Perl improvement toward the PDF, opted into by the user.

**Mechanism** (`geometry_sty.rs`): parse the margin/paper keys (via `\setkeys*`,
so the ~40 unimplemented keys are silently ignored — divergence-adjacent fix to
`\setkeys*` in `keyval_sty.rs`), compute `\Gm@tw`/`\Gm@th`, and at
`\AtBeginDocument` prepend an SVG-scope injector to `\tcolorbox` (the whole
tcolorbox family funnels through it — `\newtcolorbox` envs call `\tcolorbox`),
`\tikzpicture`, and `\pgfpicture`. The injector raises `\linewidth`/`\hsize`/
`\columnwidth`/`\textwidth` to `\Gm@tw` **only when `\linewidth=\textwidth`**
(top level) so a locally-reduced `\linewidth` (nesting minipage/parbox) is
preserved, and only inside the picture's group so the surrounding HTML flow is
never touched. Whole feature is gated on `geometry` being loaded — zero effect
on documents that do not use it. A paper-class binding that omitted geometry as
"visual-only" must now load it with the class's margins to benefit
(`fairmeta_cls.rs`, which is why the boxes were 2:1 before).

Because the panels become geometry-sized but the figure's float width
(`arrange_panels`, div. #62 / WISDOM #62) is captured from `\hsize` in the HTML
flow (345pt), the two bases would disagree and two `0.495\linewidth` boxes that
sit side-by-side in the PDF would wrap to separate rows. `after_float`
(`latex_constructs.rs`) therefore captures the **arrangement** float width as
`\Gm@tw` when the float spans the full text width — the row threshold only, the
float's HTML content stays class-default.

**Known residual**: a `\ttfamily` code box (2605.29955 Fig 1 right card) can
still poke a few px past its border — the browser's monospace is wider than
cmtt10, forcing one extra wrap the engine did not measure. Shared with Perl
(whose box is likewise near-overflow), orthogonal to geometry, and much reduced
by this change (~22px → ~6px). Tracked with the foreignObject em-basis follow-up
(WISDOM #47).

**Guards**: `122_geometry_svg_sizing::geometry_sizes_svg_but_not_html_flow`
(tcolorbox SVG widens to the geometry `\linewidth`; a sibling `\rule` stays
class-default; the `--ltx-fo-*` em sizing survives).

### 100. A text-mode `{...}` group that breaks a paragraph repacks the outer paragraph at digestion

**Perl** `Core/Stomach.pm` `T_BEGIN` (`Engine/TeX_Box.pool` L30-42): a text-mode
`{...}` flows its content FLAT into the enclosing list (`push($open);
$stomach->digestUntil()`, no localization), so a `\par` inside the group
`repackHorizontal`s the ENCLOSING paragraph. `computeBoxesSize` (Font.pm
L667-682) then never sees a run of loose character boxes in a vertical list —
every box it meets is already a packed paragraph `List`, a rule, or a vskip.

**Rust** digests a text-mode `{...}` into a **localized** box-list — `tex_box.rs`
`DefPrimitive!("{")` calls `digest_next_body`, returning the group as a `List`.
The downstream pgf/tikz/box-capture bindings are built around that `List`
representation, so a faithful flat `digestUntil` port is NOT viable — it regresses
them engine-wide (the #99 tcolorbox path went 111→192pt on the experiment).
Consequence of the localization: a `\par` INSIDE a text-mode group (e.g.
`\tcbline@`'s `{\parskip\z@\par\nointerlineskip}`) repacks only the empty inner
list, leaving the outer paragraph's characters LOOSE (no `mode` property).
`computeBoxesSize` counted each loose glyph as its own vertical line — one
`\baselineskip` per extra glyph — inflating tcolorbox/tikz SVG heights
(`xxx\tcbline yyy` @ width=100pt measured **111.03pt** vs Perl **77.82pt**;
arXiv:2605.29955 Fig 1).

**Divergence** ("R2", `tex_box.rs` `{` primitive): reproduce Perl's END STATE
without its flat digestion. When the `{` primitive is entered mid-paragraph
(`MODE=horizontal`, vertical `BOUND_MODE`) and its digested body **resumed
vertical mode** (contains a `\par` whatsit — a box whose `mode` ends in
`vertical`), call `repack_horizontal()` after the group closes. That packs the
outer paragraph's now-restored loose run into a `mode="horizontal"` `List` — the
same structure Perl's flat `\par` would have produced — so `compute_boxes_size`
runs the pure Perl one-line-per-box algorithm (no measurement-layer change) and
lands on 77.82pt, an exact Perl match. The guard is narrow: ordinary inline
`{...}`, math groups, and groups entered in vertical mode never satisfy both
conditions, so only the rare `\par`-inside-a-text-group idiom is touched. (The
box-list divergence has NO other observable symptom — the DOM absorption already
coalesces the loose boxes into one `<ltx:p>`, verified; measurement was the sole
consumer that didn't.)

**Guard**: `123_tcbline_nointerlineskip::tcbline_box_not_over_measured`
(`xxx\tcbline yyy` @ width=100pt sizes to < 90pt, i.e. Perl's 77.82 not the
pre-fix 111.03). Do not confuse with #99's *known residual* (a `\ttfamily` box
poking a few px past its border): that is the TeX-realm-vs-SVG **font impedance
mismatch** (drawn shapes frozen at TeX-font metrics while only the foreignObject
text reflows in the client), a structural limitation tracked with WISDOM #47 —
orthogonal to this repack fix.

### 101. Math nested inside a text box that itself sits in math is converted, not left raw

**Perl** `Post/MathML.pm` `pmml_text_aux` (L1063-1073): when an `ltx:XMText`
(the atom a `\parbox`/`\mbox`/`\text` becomes inside math) contains an element
Perl does not special-case — e.g. the `ltx:inline-block` a `\parbox` produces —
Perl clones that subtree **verbatim** and warns `unexpected:nested-math`. The
inline `$...$` inside are `ltx:Math` nodes the top-level pass already skipped
(`//ltx:Math[not(ancestor::ltx:Math)]`), so their `<ltx:XMath>` content-MathML
survives to the browser, which renders the tokens in **document (operator-first)
order** — `A\in\mathcal L` comes out `∈AL`. A SHARED FAILURE: both engines
produce the identical garbling (arXiv:2608.05024, arXiv html_feedback #6847 —
three `\Delta_k(S):=\sup\{…:\parbox{…}{$A\in…$,\\$B\in…$}\}` displays).

**Divergence** (surpass-Perl): `rebuild_text_subtree_with_doc`
(`latexml_post/src/mathml/mod.rs`) — the shared XMText-subtree materializer —
now CONVERTS a nested `ltx:Math` to a self-contained inline `<m:math>` element
(`nested_ltx_math_to_inline_mathml`) instead of cloning its `<ltx:XMath>` raw,
so the parbox's inline math renders as real math in reading order (`A ∈ ℒ(…)`,
subscripts and all). The `<math>` wrapper is load-bearing: the parbox becomes an
HTML `<span class="ltx_inline-block">`, and `span` is an HTML5 MathML-**breakout**
tag, so a bare `<mrow>` there would be parsed as HTML and render as flat text —
the `<math>` re-enters MathML context. (The top-level pass gets the equivalent
wrapper from `MathProcessor::outer_wrapper`; this is its nested analogue. The
sibling direct-child case — a nested `ltx:Math` DIRECTLY in an `m:mtext`, as
`\text{$x$}` yields — needs no wrapper because `<mrow>` in `<mtext>` stays
MathML, so `pmml_text_aux`'s `ltx:Math` arm deliberately keeps the bare form.)
This requires `convert_to_pmml` to be **reentrancy-safe**: it overwrites the
inherited style/font/color thread-locals, so it now snapshots and restores them,
letting a nested conversion run mid-outer-conversion without corrupting the outer
math's remaining tokens. The `unexpected:nested-math` warning is retired.

**Guard**: `124_parbox_nested_math::parbox_nested_math_converts_to_presentation_mathml`
(full-pipeline binary run: no `XMTok`/`XMApp` leaks into the HTML, and the
operand `A` precedes the relation `∈` inside the parbox).

### 102. The title-page date renders bare, with no surrounding parentheses

**Perl** `resources/XSLT/LaTeXML-structure-xhtml.xsl`, the `dates` named
template: it wraps every combined dates `<div class="ltx_dates">`
in literal `(`…`)` (`<xsl:text>(</xsl:text>` … `<xsl:text>)</xsl:text>`), so a
title-page `\date{August 1, 2024}` renders as `(August 1, 2024)`. This is a
long-standing LaTeXML web-output convention, ported verbatim; same-host Perl
`latexmlc` emits the identical parens. But no LaTeX puts parentheses around
`\date` — titlepage or inline — so the parens are a pure fidelity gap vs the
PDF (arXiv html_feedback #1934, arXiv:2408.08811). Note: ar5iv only neutralizes
`\today` (the auto compile-date), not an author's explicit `\date`, so the date
itself is legitimate content — only its parenthesization is wrong.

**Divergence** (surpass-Perl): drop the two paren text nodes from the `dates`
template so the date renders bare (`August 1, 2024`). The core XML is unchanged
(`<date role="creation">August 1, 2024</date>` — the parens were never there);
this is purely a rendering change, XSLT-only, so no core-XML golden moves and the
`ltx:date` source-locator path (`52_source_map`) is untouched (it asserts on
`data-sourcepos`, not the paren text).

**Guard**: `125_date_no_parens::date_renders_without_surrounding_parens`
(full-pipeline binary run: the `ltx_dates` div keeps the date text but carries no
`(`/`)`).

### 103. A `longtable` `\caption` leaves no stray empty body row

**Perl** In real `longtable.sty` the caption is **one full-width row**:
`\LT@makecaption` emits `\LT@mcol\LT@cols c{…}` (`longtable.sty:476`) and
`\LT@mcol` is `\multicolumn` (`longtable.sty:127`) — i.e. a single
`\multicolumn{ncols}{c}{caption}` cell spanning every column, terminated by the
user's `\\`. LaTeXML instead **hoists** the caption text into a semantic
`<ltx:caption>` (correct for HTML) but models the row with the plain column
template rather than a `\multicolumn`, so the now-textless row degrades into a
line of empty per-column `<ltx:td>` cells instead of vanishing. Same-host Perl
0.8.8 keeps that stray empty `<ltx:tr>` **byte-for-byte** (so it is parity, not a
Rust-only defect); pdflatex renders the caption as spanning text with **no** blank
body row (issue #534, reporter nasser1; the reporter's screenshot shows the
spurious bordered cell above the header). The stray row only manifests when the
caption sits in the table **body** — inside `\endfirsthead`/`\endhead` the grab
machinery already discards it (`tests/alignment/longtable.{tex,xml}` shows the
clean head-grabbed case, unchanged by this fix).

**Divergence** (surpass-Perl): because the whole row was only ever the caption,
drop it entire. `\lx@longtable@caption@` sets the existing `LONGTABLE_KILL_NEXT`
row-discard flag (the very flag `\lx@longtable@kill@flag` uses for `\kill`); the
caption's terminating `\\` (vtype `cr`) consumes it in `tex_tables.rs` and pops
the just-ended empty row via `remove_row`. Uniform across caption position
(start/middle/end) and body-vs-grabbed-head; the caption text is still hoisted
(and, in a grabbed head/foot, still moves `LONGTABLE_CAPTIONS→…_HEAD_CAPTIONS`
before the grab). `longtable_bindings` resets the flag per table so a malformed
caption-without-`\\` cannot leak the drop into the next table's first row. A
mid-table caption flanked by two `\hline`s correctly coalesces to a double rule
(`border="…tt"`) once its row is gone — the two rules were really there.

**Guard**: `53_alignment::longtable_caption_test`
(`tests/alignment/longtable_caption.{tex,xml}` — four longtables exercising
caption at start+`\hline`, start without rules, middle, and end; the golden has
no stray empty caption row in any).

### 104. An empty `\hypertarget`/`\hyperdef` emits a bare anchor, never wraps an open node

**Perl** `hyperref.sty.ltxml`'s `localized_anchor` (`afterConstruct` of
`\hypertarget`/`\hyperdef`, L238) DFS-walks from the current node and wraps the
first node `ltx:anchor` may legally contain — with **no** empty-content
short-circuit and **no** open-node guard. Two failures follow, and same-host
Perl 0.8.8 exhibits **both byte-for-byte** (so this is SHARED-FAILURE, not a
Rust-only defect):
- An **empty** `\hypertarget{id}{}` has nothing of its own to localize onto, so
  the walk grabs unrelated *surrounding* content. Mid-paragraph
  (`Before \hypertarget{b}{}after`) Perl wraps the preceding run:
  `<anchor>Before </anchor>after`.
- At the **head of a floating `ltx:note`** — the common "linked / back-referenced
  footnote" idiom `\footnotetext{\hypertarget{id}{}#2}` — the only candidate is
  the still-**open** note, which `ltx:anchor` *may* contain, so the walk wraps and
  prematurely **closes** it: the note comes out empty and the footnote text is
  orphaned into the enclosing `<p>`, with `Error:malformed:ltx:anchor` +
  `Error:malformed:ltx:note`. Renders in ar5iv as a bare "number + rule" in the
  margin (issue #526, reporter dginev; witness **arXiv:2607.16395v1**, revtex4-2,
  whose `\linkedfootnotetext` macro hits this on every call).

**Divergence** (surpass-Perl): two general guards in `localized_anchor`
(`hyperref_sty.rs`), no per-constructor special-case:
1. the localizable content is the construct's **last argument** (the `{text}` of
   both `\hypertarget` and `\hyperdef`); when it `is_empty()`, the anchor is a
   pure destination — emit a bare self-closed `<ltx:anchor xml:id=id/>` at the
   insertion point;
2. never select a candidate the document reports as still **open**
   (`document.is_open`); when the walk finds no in-content target, emit a bare
   anchor rather than failing.
Net: the empty-in-note idiom yields `<note …><anchor xml:id=id/>text…</note>`
with **0 errors**, the empty-in-paragraph case yields a clean bare
`<anchor/>` between the surrounding text (Perl wraps the preceding run), and
non-empty targets are wrapped exactly as before (Perl-identical). Upstream:
fileable against `brucemiller/LaTeXML` (Perl would benefit from the same two
guards).

**Guard**: `50_structure::hypertarget_empty_anchor_test`
(`tests/structure/hypertarget_empty_anchor.{tex,xml}` — non-empty in text, empty
in text, empty at head of note, non-empty at head of note; the golden is clean
with 0 errors across all four).

### 105. enumitem `leftmargin` is surfaced for CSS theming, not silently dropped

**Perl** `enumitem.sty.ltxml:54` groups `leftmargin` (with `align`, `left`,
`labelindent`, `itemindent`, `labelsep`, penalties…) under `# IGNORED: Alignment,
Positioning, penalties`: the key is `DefKeyVal`'d only to consume its value, never
applied. This is a deliberate design choice — LaTeXML targets rich *structural*
HTML themeable by CSS on logical roles, not a transcription of print
micro-typesetting (which rarely reflows well responsively). So both engines drop
positioning keys; `\begin{enumerate}[leftmargin=*]` renders with the default
indent (issue #559, reporter nasser1).

**Divergence** (surpass-Perl, user-approved 2026-08-15): we keep the same
structural stance but expose `leftmargin` on the emitted `<ltx:enumerate>` /
`<ltx:itemize>` / `<ltx:description>` so a stylesheet *can* act on it, split by
kind (`enumitem_sty.rs::begin_enum_itemize`):
- `leftmargin=*` (the flush *mode* — a boolean-like toggle) ⇒ a semantic
  `class="ltx_leftmargin_flush"` a theme can target;
- `leftmargin=<dim>` (a *length*) ⇒ a `cssstyle="--ltx-enum-leftmargin:<dim>"`
  CSS custom property.
The default stylesheets consume both — `.ltx_itemize, .ltx_enumerate {
margin-left: var(--ltx-enum-leftmargin, 1em) }` plus `.ltx_leftmargin_flush {
--ltx-enum-leftmargin: 0 }` (ltx-article/book/report.css). Lists without a
`leftmargin` key emit **no** attribute (the constructor omits an absent-property
attribute), so existing output is byte-identical — zero golden churn. The
`--ltx-*` custom property follows the ar5iv-css public-surface convention
(`~/git/ar5iv-css` `docs/rfc_latexml_custom_properties.md`); the ar5iv stylesheet
consumes it separately.

**Guard**: `50_structure::enumitem_leftmargin_test`
(`tests/structure/enumitem_leftmargin.{tex,xml}` — `leftmargin=*` enumerate +
itemize ⇒ the flush class, `leftmargin=2em` ⇒ the custom property, and a plain
enumerate with no attribute).

### 106. `parskip.sty` is loaded raw (no-indent paragraphs) instead of an empty stub

**Perl** `parskip.sty.ltxml` is an **empty stub** (`package …Pool; …; 1;` — "Nothing
to do here, really"): `\usepackage{parskip}` has no effect, so the first-line
indent the package exists to remove is left in place (issue #558, reporter
nasser1; same-host Perl 0.8.8 identical ⇒ SHARED-FAILURE), and a document relying
on the etoolbox it requires gets `undefined:\AtEndPreamble` (2 errors on the
liftarm repro).

**Divergence** (surpass-Perl, user-approved 2026-08-15): the binding
(`parskip_sty.rs`) raw-loads the real `parskip.sty` v2.0h (since batch 56jk; before,
it hand-set `\parindent` and `\parskip`). The package requires kvoptions (L44) and
processes `indent`/`parfill`/`skip`/`tocskip` (L45-50), sets `\parskip` (L51-55,
default `.5\baselineskip plus 2pt`), `\parfillskip` (L56-57) and `\parindent` (L58,
default `indent=0pt`), and requires etoolbox (L59). `\parindent=0` is the
load-bearing part: the paragraph machinery marks every paragraph, the first one
included since issue #719, with the existing `ltx_noindent` class when the
`\parindent` register is zero (`tex_paragraph.rs`), exactly as a manual
`\setlength{\parindent}{0pt}` does, and the CSS suppresses the first-line indent
(`.ltx_noindent > .ltx_p:first-child { text-indent:0 }`). `\parskip`'s glue is not
typeset into HTML (true for a manual `\setlength` too); list padding takes
parskip's zero `\topsep`/`\partopsep` as pdflatex does. Expected on every parskip
document: its `\patchcmd\@startsection` fails against LaTeXML's `\@startsection`
(one `Info:unexpected:patchcmd` and the console line "Couldn't patch
\@startsection"); the `\@starttoc` and `\@xsect` patches apply. **Out of scope:**
parskip's vertical spacing as a themeable feature.

**Guard**: `50_structure::parskip_test` (`tests/structure/parskip.{tex,xml}` —
`\usepackage{parskip}` + three paragraphs, all `class="ltx_noindent"`);
`shipout_parskip::{parskip_loads_kvoptions_and_etoolbox,
parskip_processes_its_package_options}`.

### 107. A natural-size vector figure is sized in font-relative `em`, not fixed pixels

**Perl** sizes `\includegraphics` figures in absolute pixels: `LaTeXML::Post::Graphics`
asks ImageMagick to raster the image and reports the pixel count, which the XSLT
emits as `<img width="N" height="N">` (bare = CSS px). For a figure with no author
size that pixel count is fixed — it ignores the reader's font-size, browser zoom,
and device density, so on enlarged text the figure shrinks relative to the prose,
and at the document's intended font it lands ~25% small (a bp value read as px:
72-vs-96 dpi). Oxide's own vector-SVG path inherited the same fixed-pixel basis.

**Divergence** (surpass-Perl, user-approved 2026-08-15; issue #562, reporter
xworld21): a figure included at its **natural size** (no `width=`/`height=`/
`totalheight=`/`scale=`) from a **vector** source is sized in `em` — its true
typeset size over the local font size — so it reproduces the figure-to-text
proportion of the source at any reading size (and hits the correct physical size at
the document's font). The natural size is read directly as a physical length in TeX
pt — a PDF page box, an EPS/PS `%%BoundingBox`, or an SVG's lengths/viewBox, all
bp→pt (`latexml_core::util::image::natural_display_size_pt`, which returns `None`
for raster formats and so gates the em path to vector figures). It is read fresh at
`<ltx:graphics>` construction rather than reused from `image_graphicx_sizer`'s
`cached_width`, which runs EPS/raster sizes through a device-DPI round-trip and is
not a physical length. `em = size_pt / font_pt` against the LOCAL font
(`graphicx_sty.rs` `after_construct`), emitted as `cssstyle="width:Nem; height:Nem"`
— copied verbatim into the `<img>`/`<object>` style by `LaTeXML-common.xsl`, where
CSS overrides the pixel `imagewidth`/`imageheight` fallback (which is retained). No
XSLT change.

Scope: only **unsized** vector inclusions. Author-sized (`width=`/`scale=`) figures
keep the pixel path — their size is the author's absolute choice, not the figure's
intrinsic size — and raster (PNG/JPEG) inclusions are unchanged (a pixel count is
not a physical size). This is the font-relative direction upstream Perl LaTeXML is
independently moving toward (relative/`em` SVG sizing), so it converges rather than
diverges long-term; it commits **no** absolute px factor. The one free parameter —
the base-font reference (here the local font's pt size) — is the seam to reconcile
with the upstream scheme.

Known limitation: an SVG *source* whose root carries only a unitless `viewBox` (no
`pt`/`cm` lengths) is read by `read_svg_size_pt` as px, so its em can be off by
96/72; PDF/EPS/PS sources and SVGs with absolute lengths are exact. Validated on
witnesses — em matches the source box: arXiv:2103.00051 (PDF, three natural
figures), 1601.00046 + 0704.0052 (EPS) — all with 0 graphics errors.

**Guard**: `cluster_cli::em_figure_sizing::natural_vector_figure_is_em_sized_author_sized_stays_pixels`
(a minimal `/MediaBox`-only PDF sizes to `10.037em/5.019em`; `[width=]`/`[scale=]`
siblings stay on the pixel path).

### 108. Display math is contained within a width-constrained box, not left to escape

**Perl** LaTeXML.css renders display math as `.ltx_eqn_table { display:table;
width:100% }` with 50%-wide center-pad cells. Inside a width-constrained box —
a `p{}` cell / `\parbox` / `minipage` (`.ltx_inline-block`) or a table cell
(`.ltx_td`) — a wide equation's intrinsic width exceeds the box, and because
`overflow` is ignored on `display:table`, nothing clips it: the equation
**escapes the cell** and scatters across the page (issue #533, reporter nasser1
— a `longtable` `p{}` cell holding an `enumerate` whose items contain
`\[\begin{aligned}…\end{aligned}\]`). Same-host Perl 0.8.8 renders the identical
breakage ⇒ SHARED-FAILURE; the lualatex PDF keeps the math in its cell.

**Divergence** (surpass-Perl, user-approved 2026-08-15): a bundled-CSS local
delta re-boxes display math as a **block scroll container** *only under a
constrained ancestor*, so it stays inside the cell and scrolls horizontally when
too wide — the same containment `.ltx_listing` (code) already uses:
`.ltx_inline-block .ltx_eqn_table, .ltx_td .ltx_eqn_table { display:block;
overflow-x:auto; max-width:100% }`. `display:block` (not `overflow` on the
table, which browsers ignore) is required to establish the scroll box; the
`.ltx_eqn_row/cell` children regenerate an anonymous table, so center-pad
centering and eqno columns still lay out correctly. **Scoped deliberately**:
normal full-width display math keeps `display:table` untouched (the heavily-
exercised common path is unchanged; the broader unconditional variant that also
tackles page-level equation overflow was declined for this ticket). Mirrored in
`~/git/ar5iv-css` (both stylesheets define `.ltx_eqn_table` identically). CSS
only — zero core/XML/HTML change, no golden churn.

**Guard**: `latexml_post::xslt::witnessed_css_delta::constrained_equation_overflow_delta_stays_present`
(asserts the bundled `LaTeXML.css` keeps the `#533` selectors + declaration, so a
future re-vanilla sweep cannot silently drop it — same shape as the #473/#431
CSS-delta guards).

### 109. A `\text{…}`-only display equation nowraps its cell (renders on one line)

**Perl** `LaTeXML.css` gives every aligned *table* cell `white-space:nowrap`
(`.ltx_td.ltx_align_{left,right,center}`, `.ltx_th…`) but NOT the equation cell
(`.ltx_eqn_cell`). A single display equation `\[…\]` puts its content in an
`ltx_eqn_cell` with an alignment class and NO `ltx_td`, laid out between two
50%-width centering pad cells inside a `width:100%` `ltx_eqn_table`. For ordinary
math that is fine (a `<math>`/image is atomic and unwrappable), but `\[\text{The
solution is not valid}\]` digests to `ltx_markedasmath` **text** — which *is*
wrappable — so the center cell collapsed to its min-content and the text stacked
one word per line. `\begin{align*}` is unaffected because its content sits in a
real `ltx:td` (`ltx_td.ltx_align_right`), which already nowraps. Same-host Perl
0.8.8 reproduces the broken stacking byte-for-byte (issue #527, reporter nasser1).

**Divergence** (surpass-Perl, user-approved 2026-08-15): `LaTeXML.css` gives
`ltx_eqn_cell` the **same** nowrap-with-`ltx_wrap`-optout treatment as `ltx_td`/
`ltx_th`, for `ltx_align_left`/`right`/`center` — so a display equation renders on
one line regardless of whether its content is real math or marked-as-math text,
and a very wide display overflows/scrolls (standard) rather than reflowing into a
column. CSS-only: the emitted HTML is byte-identical (zero golden churn); a
too-wide equation was already the behavior for `align*` and real math. Upstream-
fileable (`brucemiller/LaTeXML`, not yet filed).

Covers both content shapes that reach the centering cell: a pure `\text{}` display
(a bare `ltx_markedasmath` run — **stacks** one word per line without the fix) and
mixed math+text `\[x^2+\text{…}=y^2\]` (a `<math>` with `<mtext>` — **clips** its
text without the fix). Both render on one line with it.

**Guards** (two levels):
- `cluster_cli::display_math_text_nowrap::display_math_text_cell_gets_nowrap_css` —
  platform-independent structural guard: both displays land in the
  `ltx_eqn_cell ltx_align_center` cell, and the destination `LaTeXML.css` carries
  the `.ltx_eqn_cell.ltx_align_center` nowrap rule.
- `cluster_cli::browser_render_display_math::display_math_renders_on_one_line_without_clipping` —
  the **rendered** guarantee: Playwright (system Chrome, `tests/browser/measure.js`)
  measures the cell geometry and asserts both displays are one line tall
  (`cellHeight < 40px` — catches stacking) and un-clipped (`scrollWidth −
  clientWidth ≤ 2px` — catches the mixed overflow). **Opt-in / local only** — a
  headless-browser job is expensive, so it is NOT run in CI (the structural guard
  above is the CI gate); self-skips unless `npm install`ed in `tests/browser` and
  run with `LATEXML_BROWSER_TESTS=1`. Verified red→green: reverting the CSS rule
  fails it at 74px-tall (pure) / 35px-overflow (mixed).

### 110. Publication metadata pubnotes render outside the title `<h1>`, not inside it

**Perl** `LaTeXML-structure-xhtml.xsl` `maketitle` collects **every** `ltx:pubnote`
into a "footnote-like block" *inside* the title `<h{level}>` (a `†`-collapsed hover
popup). Class bindings route a lot through `\lx@add@pubnote`: acmart
(`acmart.cls.ltxml` L59-70) maps `\acmConference`/`\acmDOI`/`\acmISBN`/
`\acmJournal`/`\acmVolume`/… all to `pubnote`s. So the `<h1 class="ltx_title">`
ends up containing the conference/DOI/ISBN text (behind a stray `†` on the
heading) — publication metadata leaking into the title element. Same-host Perl
0.8.8 produces the identical structure. Not acmart-specific: IEEEtran and other
class bindings route the same way. Issue arXiv/html_feedback#6886, witnesses
arXiv:2410.20027 + 2603.16021 + 2601.14324 (acmart) and 2509.09112 (IEEEtran) —
all four `<h1>` (and `<head><title>`) titles are clean after the fix.

**Divergence** (surpass-Perl, user-approved 2026-08-15): split the pubnotes by
role in `maketitle`. Genuine title **footnotes** — `\thanks`, `\titlenote`/
`\subtitlenote` (`role='note'`/`'thanks'`) — stay inside the `<h1>`, matching
author intent (a `\thanks` *is* a title footnote). Publication **metadata**
(everything else) moves to a **sibling** `<span class="ltx_pubnotes
ltx_pubnotes_meta">` block after the title, so the `<h1>` holds only the title
text (and its real footnotes). The `pubnotes` XSLT template is parametrized with
a `notes` node-set; no core/XML change, and non-pubnote papers are byte-identical.
For arXiv:2410.20027 the `<h1>` is now just "Agentic Feedback Loop Modeling
Improves Recommendation and User Simulation" (was …+"Conference: … DOI: …" + a
stray `†`), verified with headless Chrome under ar5iv.css. The ar5iv stylesheet
styling of the moved `ltx_pubnotes_meta` block — the inherited dagger
hover-popup on narrow viewports, promoted to always-visible right-margin
marginalia on wide (`>= 96rem`, sized from the real available margin so it never
overflows) — is `dginev/ar5iv-css#45`.

The HTML `<head><title>` is clean of notes by construction, independent of this
`<h1>` fix: the engine extracts every note out of `\title{}` into **sibling**
`<pubnote>` elements (a `\thanks` nested in the title lands as a `role='thanks'`
sibling), so the core `<title>` node — and the navigation title the head
`<title>` derives from — carry only title text.

**Guards**: `cluster_cli::title_pubnote_pollution::title_h1_excludes_metadata_pubnotes_keeps_footnotes`
(a `\lx@add@pubnote[role=conference/doi/note]` fixture: metadata not in the `<h1>`,
the `role=note` footnote kept in it, the metadata in an `ltx_pubnotes_meta`
sibling after it) and `…::head_title_excludes_all_note_content` (a `\thanks` in
`\title{}` + metadata pubnotes: the head `<title>` keeps the title text and no
note/metadata text; verified red under a note-injecting head-title mutation).

### 111. fancyvrb `frame=single` renders as a semantic box, not raw rules

**Perl** LaTeXML raw-loads `fancyvrb.sty` and lets its frame machinery run: `frame=single`
selects the `@Single` hooks (`fancyvrb.sty:776`) that draw the frame with `\vrule`/`\hrule`
(top/bottom `\FV@SingleFrameLine` L869-904, per-line side rules L948-963, the sep box
`\FV@SingleFrameSep` L929-947). LaTeXML captures those as literal `<ltx:rule>` elements
that never reconstruct into an HTML box — so the frame renders as disconnected line
fragments, not a rectangle — and the bottom `\FV@SingleFrameSep` box (only ink = two side
`\vrule`s, no text) surfaces as a **stray empty line** below the last line. fvextra's
`backgroundcolor` (a per-line `\colorbox` strip, `fvextra.sty:2547`) is likewise not
captured. Issue #525 (reporter nasser1: frame not drawn, extra trailing line, backgroundcolor
ignored). Same-host Perl 0.8.8 is byte-identical (SHARED-FAILURE); the lualatex PDF draws a
proper frame.

**Divergence** (surpass-Perl, user-approved 2026-08-15): redefine fancyvrb's `@Single` frame
hooks in the binding (`fancyvrb_sty.rs`) so `\FV@BeginListFrame` **opens** an
`ltx_framed_rectangle` box and `\FV@EndListFrame` **closes** it (both fire exactly once around
the whole line set, inside `\FV@List`/`\FV@EndList`'s single group), neutralizing the per-line
side rules — which drops the raw `<rule>` elements AND the stray sep line in one move.
`framesep`→`padding`, `framerule`→`border-width`, fvextra `\FancyVerbBackgroundColor`→
`background` (its per-line strip collapsed to one wrapper background; the key is also ported in
`fvextra_sty.rs` so it works on a host fvextra predating the feature — the TL fvextra loaded in
CI errors `keyval: backgroundcolor undefined` otherwise). `rulecolor` keeps the default black
border. Only per-instance VALUES are inline (`cssstyle` border-width/padding/background); the
box reuses the existing `ltx_framed_rectangle` semantics for the border and carries a dedicated
`ltx_framed_verbatim` class for the fixed **responsive** behaviour — `max-width:100%;
overflow-x:auto; box-sizing:border-box` in a bundled-CSS delta (`LaTeXML.css` + ar5iv-css
mirror). Rationale: the box spans the print `\linewidth` (~460px, faithful on desktop/tablet)
but is a shrink-to-fit inline-block of non-wrapping verbatim lines, so on a phone viewport it
would push its right border off-screen and scroll the whole page; the class caps it at the
viewport and scrolls over-long lines *within* the box (the containment `.ltx_listing` uses for
code, and the #533 delta for math). **Scoped to `frame=single`** (the issue's case); the
`@Lines` `topline`/`bottomline`/`lines` variants still draw raw rules — a follow-up.

**Guards**: `00_tokenize` fixture `tests/tokenize/fancyvrb_frame.{tex,xml}` — two `Verbatim`s
(`frame=single` with explicit framerule/framesep, and `frame=single`+`backgroundcolor=yellow`):
the golden has two `framed="rectangle" class="ltx_framed_verbatim"` boxes with the right
`border-width`/`padding`/`background` cssstyle and **zero** `<ltx:rule>` elements; plus
`latexml_post::xslt::witnessed_css_delta::framed_verbatim_responsive_delta_stays_present`
(the `.ltx_framed_verbatim` responsive rule stays in the bundled `LaTeXML.css`).

### 112. `pdfcol.sty` is a no-op stub (PDF colour stacks have no HTML output)

**Perl** LaTeXML ships no `pdfcol.sty.ltxml`. When tcolorbox's `breakable` library
(`\tcbuselibrary{breakable}`) or a document `\RequirePackage{pdfcol}`s the package, LaTeXML
finds no binding, and pdfcol's `\pdfcolInitStack` / `\pdfcolIfStackExists` / `\pdfcolSwitchStack`
/ `\pdfcolSetCurrentColor` / `\pdfcolSetCurrent` are undefined → `Error:undefined:\pdfcolInitStack`
and the args leak as body text. Issue #531 (reporter nasser1). Same-host Perl (TL2025) errors
identically (SHARED-FAILURE).

**Divergence** (surpass-Perl, SHARED-FAILURE → clean): `pdfcol_sty.rs` ports `pdfcol.sty`'s own
`\ifpdfcolAvailable … \else … \fi` **disabled** fallback branch (`pdfcol.sty` L275-291), which
the real package takes whenever pdfTeX's colour-stack primitives are unavailable — always the
case for LaTeXML. Every command becomes a no-op; `\pdfcolIfStackExists#1#2#3` expands to `#3`
(the stack is never registered, so the false branch runs); `\pdfcolErrorNoStacks` is silenced to
`\relax` (raising a conversion error for a PDF-only capability with no HTML output would defeat
the fix). A PDF colour stack is a page-model construct with no rendering in HTML/XML, so no
currently-passing paper changes shape — the divergence is purely the *absence* of the shared
error. **Upstream**: Perl LaTeXML would benefit from the same no-op binding.

**Guards**: `06_cluster_regressions::cluster_pdfcol_stub_no_undefined`
(`tests/cluster_regressions/pdfcol_stub.tex`) — the five `\pdfcol…` commands emit no `<ERROR>`
and the body collapses to exactly `STACK-NO` (the disabled false-branch), no leaked args.

### 113. `\everyjob` is fired at job start (l3sys system constants get defined)

**Perl** LaTeXML never fires TeX's `\everyjob` token list. Real TeX inserts `\everyjob` at
`main_control` start — right after the format is loaded, before the first token of the main
input (`tex.web` §1030: `if every_job<>null then begin_token_list(every_job,every_job_text)`).
LaTeX's `\everyjob` contains `\__kernel_sys_everyjob:`, which runs `\g__sys_everyjob_tl` to
define the l3sys *system* constants — `\c_sys_shell_escape_int`, the `\sys_if_shell:*` /
`\sys_if_shell_unrestricted:*` / `\sys_if_shell_restricted:*` conditional families, and the
`\c_sys_{minute,hour,day,month,year}_int` date/time ints. l3sys defers ALL of these into
`\__sys_everyjob:n { … }` blocks (`expl3-code.tex` L8131-8217), i.e. into `\g__sys_everyjob_tl`,
precisely so they are (re)computed at each job start. Because Perl (and, pre-fix, Rust) never
fires the hook, those constants are undefined on the dump/short-circuit path — a texmf
`expl3.sty` NEWER than the embedded dump takes the `\ifx\csname tex_let:D\endcsname\relax`-false
branch and skips `\input expl3-code.tex`, relying on the format to have run everyjob. That newer
`expl3.sty` then USES `\sys_if_shell:TF` in its support-file/shell-escape check →
`Error:undefined:\sys_if_shell:TF`. Issue #531 secondary (reporter nasser1; TL2026 dump
2026-01-19 vs texmf 2026-03-20). Reproduced in `ghcr.io/tkw1536/texlive-docker:2026` with
l3kernel 2026-07-20 overlaid on the 2026-01-19 dump.

**Divergence** (surpass-Perl, user-approved 2026-08-15): fire `\__kernel_sys_everyjob:` at the
completion of `LoadFormat('latex')` (`latex.rs`) — the faithful equivalent of TeX's job-start
`\everyjob` insertion, since our LaTeX "format" is exactly that pool block. The constants are
then defined with LIVE values on every conversion, before the document preamble runs. The
`INI_MODE` early return in the same block means the firing is SKIPPED during dump-build, so the
date/time ints are never frozen into the dump (only `\g__sys_everyjob_tl` — the deferred
*recipe* — is dumped, which is correct). Guarded on `\__kernel_sys_everyjob:` existing (present
via the latex dump or the raw-base path; absent for plain/math dumps, a clean no-op). This is a
genuine TeX-semantics improvement Perl LaTeXML would also benefit from. **Upstream**: worth
filing against `brucemiller/LaTeXML`.

**Guards**: `06_cluster_regressions::cluster_everyjob_defines_l3sys_shell`
(`tests/cluster_regressions/everyjob_sys_shell.tex`) — a preamble `\@ifundefined{sys_if_shell:TF}`
probe emits `EVERYJOB-PRESENT` (was `EVERYJOB-MISSING` without the firing), and a body
`\sys_if_shell:TF` takes the FALSE branch (`SHELL-NO`, LaTeXML has no shell). Full suite 1971/0
confirms firing `\everyjob` on every latex conversion is output-neutral.

### 114. An empty-symbol unit renders invisibly, not as its `meaning` name

**Perl** LaTeXML renders a math token whose CONTENT is empty by falling back to its `meaning`
attribute (`MathML.pm` `stylizeContent`). For a siunitx unit declared with an empty symbol —
`\DeclareSIUnit{\nothing}{\relax}` (arXiv/html_feedback#970, paper 2312.06275) — the core emits
an empty `<ltx:XMTok class="ltx_unit" meaning="nothing" role="ID"/>`, so the presentation MathML
becomes a VISIBLE `<m:mi class="ltx_unit">nothing</m:mi>` (Perl even paints it red as a
suspected error). The author intended `\SI{5}{\nothing}` to render "5" with no unit; instead the
literal word "nothing" appears next to every number. Same-host Perl is byte-identical
(SHARED-FAILURE).

**Divergence** (surpass-Perl, user-approved 2026-08-15): in `presentation.rs::pmml_token_inner`,
an empty-content token carrying `class="ltx_unit"` renders as an empty `<m:mphantom>` — the same
invisible placeholder used for `meaning="absent"` — instead of falling through to the
`meaning`→`name`→`role` text fallback. Only empty UNIT tokens are affected (a non-empty unit
keeps its symbol; a non-unit empty token keeps the existing fallback), so no currently-passing
paper changes shape. A unit whose symbol produces nothing is exactly the case where the fallback
is wrong. **Upstream**: worth filing against `brucemiller/LaTeXML`.

**Guards**: `06_cluster_regressions::cluster_siunitx_empty_unit_renders_invisible`
(`tests/cluster_regressions/siunitx_nothing_unit.tex`, via `convert_and_post_pmml_clean`) — the
presentation MathML contains no visible `>nothing<`, the empty unit is an `<m:mphantom>`, and the
quantity still renders.

### 115. `\widthof` &friends resolve as dimensions in every dimension context, not only calc expressions

**Perl** LaTeXML's `calc.sty.ltxml` defines `\widthof`/`\heightof`/`\depthof`/`\totalheightof`
as no-op stub primitives (`DefPrimitive('\widthof','')`) and only *evaluates* them inside calc's
own expression scanner (`readValue`, invoked by `\setlength`/`\addtolength`/`\setcounter`). But
box- and rule-length arguments — `\makebox[⟨len⟩]`, `\rule{⟨w⟩}{⟨h⟩}`, `\hspace{⟨len⟩}`,
`\parbox{⟨w⟩}…` — are read by the *base* dimension reader, which never routes through that
scanner. There the non-expandable `\widthof` stops the number scan, so `\makebox[\widthof{X}][r]{Y}`
comes out `width="0.0pt"` (Perl emits `Warning: Missing number (Dimension), treated as zero`). A
zero-width box gives its content no advance width, so it overlaps its neighbour. Real
pdflatex+calc gives the true width there — calc.sty L124-137 does `\let\widthof\wd` and boxes the
argument, so `\widthof{X}` = `\wd` of a box holding X, valid *everywhere* a `<dimen>` is read.
latexml-oxide reproduced Perl exactly (SHARED-FAILURE, both diverge from real LaTeX).

**Perl behavior**: `\widthof` etc. → 0 outside a calc expression; boxed content collapses and
overlaps. **Rust behavior**: `\widthof`/`\heightof`/`\depthof`/`\totalheightof` stay bare no-op
primitives (so their digestion/reversion is byte-identical to Perl — `\mathmakebox[\widthof{…}]`
parity is preserved), but the base dimension reader gains a calc-agnostic seam
(`gullet::set_internal_dimension_fn`, consulted by `read_register_value` before it gives up with
"Missing number"). calc installs a resolver that, when the *dimension reader* meets one of these
tokens, digests its `{box}` argument and returns the measured width/height/depth/total-height —
the LaTeXML analogue of calc's `\let\widthof\wd`. So they resolve in **every** dimension context
(makebox, framebox, rule, hspace, parbox), matching real LaTeX, while digestion is untouched.
calc's own `read_value` still intercepts them by token identity for calc expressions, so
calc-routed widths (`tests/graphics/calc.xml`) are unchanged — only the previously-broken
base-reader contexts change, from a wrong `0.0pt` to the measured width.

**Why**: a kernel-quality gap, not a TeX-semantics change — real LaTeX+calc already makes
`\widthof` a valid dimension in these contexts; both LaTeXML engines simply failed to. The fix
halos across every dimension-reading construct and every calc-using paper.

**Witnesses**: arXiv 2603.23669 (html_feedback#6869) — its siunitx `S`-column result tables box
every bold value as `\makebox[\widthof{\tablenum{…}}][r]{…}`; the bold number rendered on top of
its `± unc`. Rust `\widthof{WWWW}` now measures 41.1pt (real LaTeX: 41.1112pt).

**Upstream**: worth filing against `brucemiller/LaTeXML` (same gap upstream).

**Guards**: `cluster_sizing::widthof_in_base_dimension_reader::widthof_resolves_in_makebox_and_rule_length_arguments`
— `\makebox[\widthof{WWWW}]` and `\rule{\widthof{WWWW}}{…}` widths are nonzero and agree with the
calc-routed `\setlength\reflen{\widthof{WWWW}}` reference.

### 116. Unsorted bibliography styles number the References in citation order, not alphabetically

**Perl** LaTeXML's `MakeBibliography.pm` **always** `unisort`s the cited entries
(`getBibEntries` L357, `makeBibliographyList` L398) and assigns `[N]` in that
alphabetical order (`++$NUMBER`, L418) — regardless of the `\bibliographystyle`.
It reads the style's `sort` flag into `%STYLE` (L57) but never uses it, and even
maps `IEEEtran → sort='true'` (`IEEEtran.cls.ltxml` L331). So a paper with
`\bibliographystyle{unsrt}`/`{ieeetr}`/`{IEEEtran}` — whose `.bst` is UNSORTED —
gets an alphabetized list, mismatching the published PDF. latexml-oxide
reproduced this exactly (SHARED-FAILURE: both engines alphabetize where
pdflatex+bibtex number by first citation).

**Perl behavior**: every `\bibliographystyle` alphabetizes the References; the
inline `[N]` cites index that alphabetical list. **Rust behavior**: for an
UNSORTED style (bibtex's `sort='false'`: `unsrt`, `unsrtnat`, `ieeetr`,
`IEEEtran`, `abntex2-num` — its `presort` drops the sort key, abntex2-num.bst:2021-2022, 57cm) MakeBibliography numbers the entries by **first citation appearance**
— the document order of the inline `<ltx:bibref>`s (`citation_order`,
`make_bibliography.rs`), which is exactly bibtex's `\citation`-record order. The
list is then rendered in that same numbered order (the biblist follows
`entry.number`, not a second `unisort`). SORTED styles (`plain`, `alpha`, `abbrv`,
plainnat/…, and any unknown style) are unchanged — still alphabetical, identical
to Perl. Detection is from the `bibstyle` attribute name (plus an explicit
`sort='false'`, e.g. the bibunits `\bibstyle` path), GATED on a numeric list
(`citestyle="numbers"`, or absent → numeric): an author-year list is always
alphabetical by author regardless of the `.bst` sort flag. The engine's
`lookup_bibstyle_params` also gains `ieeetr`/`IEEEtran` → `numbers`, `false`
(Perl's base table omits them and its class binding alphabetizes IEEEtran) so the
real IEEE `.bst` behavior is matched.

**natbib/revtex arm** (html_feedback #5930/#6095): a revtex4-2 or `natbib` paper
with `\bibliographystyle{ieeetr}` lost the style name before it reached the node —
natbib's `[numbers]`/`nobibstyle` option does `\let\bibstyle\@gobble`, and its
author-year `\bibstyle` has no `\bibstyle@ieeetr` preset (Perl natbib.sty.ltxml
L85-92, which flags the loss as a known FUTURE gap). The kernel
`\bibliographystyle` now records `BIBSTYLE` (`\lx@record@bibstyle`) BEFORE
dispatching to the possibly-gobbled `\bibstyle`, so the name reaches the node in
every path. It records only the name, never `CITE_STYLE` — natbib owns
numbers-vs-author-year via its options (`plainnat` is numeric in the base table
but author-year under natbib), and the citestyle guard above keeps author-year
natbib lists alphabetical. `beginBibliography` still emits no `sort` attribute, so
the core node is unchanged except for the `bibstyle` name it already ought to
carry (`thebibliography`-based lists, which read no BIBSTYLE, are untouched).

**Why**: bibtex's own guarantee is that an unsorted `.bst` leaves the References
in citation order; IEEE figure captions bake in "[2], [3], [4]" that depend on it.
Matching pdflatex+bibtex is the ground truth (html_feedback #6294 asked for
exactly this), and Perl's alphabetization is the defect.

**Residual** (shared with Perl, out of scope): `\nocite{key}` is deferred to
end-of-document in both engines (`\nocite`→`@at@end@document`), so a mid-document
`\nocite`'d entry lands after the cited ones rather than at bibtex's `\nocite`
position; and `\nocite{*}` entries (no citation position) fall to the end in
`unisort` order rather than `.bib`-file order. The reported `\cite`-order case is
exact.

**Witnesses**: arXiv 2510.05438 (html_feedback#6294) — `\documentclass{IEEEtran}`,
`\bibliographystyle{IEEEtran}`, `\bibliography{…}`, 17 entries; oxide's numbered
order now equals the pdflatex+bibtex `.bbl` order key-for-key (Wu2021=[1] … Shi2011=[17]).
arXiv 2602.00643 (html_feedback#5930) — `\documentclass{revtex4-2}` +
`\bibliographystyle{ieeetr}`, 10 entries, likewise key-for-key vs bibtex.

**Upstream**: worth filing against `brucemiller/LaTeXML` (same alphabetize-everything gap).

**Guards**: `06_cluster_bibliography::cluster_bib_unsrt_citation_order`
(unsrt + IEEEtran number gamma/alpha/beta by citation order),
`cluster_bib_plain_stays_alphabetical` (sorted styles unchanged),
`cluster_bib_bibstyle_is_known_numeric` (engine maps IEEEtran → numeric),
`cluster_bib_natbib_numeric_citation_order` (the natbib/revtex arm),
`cluster_bib_natbib_numeric_sorted_stays_alphabetical` +
`cluster_bib_natbib_authoryear_stays_alphabetical` (the unsorted + numeric gates).

### 117. biblatex style/variant packages load the native biblatex binding

**Perl** LaTeXML ships **no** `biblatex.sty.ltxml` at all — biblatex is
unsupported upstream. latexml-oxide's `latexml_contrib/src/biblatex_sty.rs` is a
surpass-Perl native binding (see #62), reached when the document `\usepackage`s
**`biblatex`** or fires the `\addbibresource`/`\printbibliography` autoload. But
the biblatex ecosystem ships dozens of **style/variant** packages —
`biblatex-chicago`, `biblatex-apa`, `biblatex-ieee`, `biblatex-nature`,
`biblatex-science`, `biblatex-phys`, `biblatex-chem`, … — each of which in
reality does `\RequirePackage{biblatex}` and then only *configures* it. Their
configuration commands (`\DeclareFieldFormat`, `\DeclareFieldAlias`,
`\renewbibmacro`, …) run in the **preamble**, before any `\addbibresource`. With
no binding for the variant name, those commands were undefined the moment they
were used, and — separately — every biber-generated `.bbl` opens with the guard
`\@ifundefined{ver@biblatex.sty}{\@latex@error{Missing 'biblatex' package}…
\aftergroup\endinput}{}`, so the `.bbl` `\endinput`ed itself and the **entire
References list came out empty** (0 bibitems, a flood of preamble
`Error:undefined:`s).

**Rust behavior**: (1) `latexml_contrib::dispatch` routes any `biblatex-<style>.sty`
name to the biblatex binding — reproducing the `\RequirePackage{biblatex}` the
variant's real `.sty` performs. (2) The biblatex binding itself now sets
`\ver@biblatex.sty` (mirroring biblatex.sty's own `\ProvidesPackage{biblatex}`),
so the biber `.bbl` guard passes no matter how biblatex was loaded — including
the variant path, where the package machinery would otherwise only set
`\ver@biblatex-chicago.sty`. The variant's preamble customization commands
resolve (as the biblatex binding's no-op/gobble stubs), and the `.bbl` renders
its entries.

**Why**: matching the published PDF, whose bibliography is present, beats an
empty References list; and the variant packages genuinely reduce to
`biblatex` + configuration, so routing them there is faithful to what the real
`.sty` does. Perl produces nothing here, so this is purely additive.

**Witness**: arXiv 2605.11180 (html_feedback #6601) —
`\usepackage[authordate,backend=biber]{biblatex-chicago}` via a `biblatex-aer.tex`
helper full of `\DeclareFieldFormat`/`\renewbibmacro`; biber `.bbl` format 3.3.
Before: 13 errors, 0 bibitems. After: 0 errors, 56 rendered bibitems.

**Guard**: `06_cluster_bibliography::cluster_bib_biblatex_variant_loads_and_renders`
(`\usepackage{biblatex-chicago}` + preamble `\DeclareFieldFormat` + biber `.bbl`:
customization does not leak/error and the References populate).

### 118. amsrefs `{bibsection}` is a bound environment

**Perl** LaTeXML's `amsrefs.sty.ltxml` binds `{bibdiv}` and `{biblist}` but not
`{bibsection}`. In the real `amsrefs.sty` (L1251/L1265) it is the other way round:
`{bibsection}[⟨heading⟩]` is the primitive section-heading wrapper around
`{biblist}`, and `{bibdiv}` is *defined as* `{bibsection}` in article mode
(`\newenvironment{bibdiv}{\bibsection}{\endbibsection}`). So an amsrefs paper that
opens its references with `\begin{bibsection}` directly — common in AMS templates
— hit `Environment {bibsection} is not defined`: the `<ltx:biblist>` then floated
in an `<ltx:p>` (schema-invalid) and the **entire References list was lost**, with
every `\cite` left dangling. latexml-oxide reproduced this (SHARED gap — Perl
errors identically).

**Rust behavior**: `latexml_package::amsrefs_sty` binds `{bibsection}[Default:\refname]`
to the **same** `<ltx:bibliography>` container and digest hooks as `{bibdiv}`, with
the optional argument as the `<ltx:title>` (default `\refname` → "References",
which is also `begin_bibliography_clean`'s own fallback, so a no-argument
`\begin{bibsection}` and `\begin{bibdiv}` render identically). The entries digest
into `<ltx:bibentry>` and MakeBibliography converts them to `<ltx:bibitem>` exactly
as the `bibdiv` path already did.

**Why**: matching the published PDF — whose bibliography is present — beats an empty
References list, and `bibsection` is a real, load-bearing amsrefs environment, so
binding it is a faithful port of the package Perl's incomplete `.ltxml` omitted.

**Witness**: arXiv 2405.18501 (html_feedback #1393) — `\documentclass{amsart}`,
`\usepackage[numeric]{amsrefs}`, references in `\begin{bibsection}\begin{biblist}`.
Before: 2 errors, 0 bibitems. After: 0 errors, 6 rendered bibitems.

**Guard**: `06_cluster_bibliography::amsrefs_bibsection_environment_renders`
(`\begin{bibsection}` + `{biblist}` + `\bib` entries: References populate, entries
convert to `<ltx:bibitem>`, no stray `<ltx:bibentry>`).

### 119. `\verb` inside `\index{…}` renders as typewriter, not an empty `<verbatim/>`

**Perl** LaTeXML reads `\index`'s argument `SanitizedVerbatim` (`latex_constructs.pool.ltxml`
L4397 `DefMacro('\index SanitizedVerbatim', \&process_index_phrases)`), which re-tokenizes the
argument string. That collapses a `\verb`'s raw catcode-12 body back into control sequences
(`\delta`, not `\`,`d`,…) and leaves `\verb` with no mouth to scan its delimiter from, so at
`process_index_phrases` time `\verb` emits an empty `<verbatim/>` and its body leaks out
mis-tokenized (`\delta` → math-italic δ). A `|` delimiter additionally collides with the
makeindex encap separator that `process_index_phrases` splits on, losing everything after the
first `|` into a bogus `style=` attribute and raising `Error:expected:delimiter Verbatim argument
lost`. latexml-oxide reproduced Perl 0.8.8 byte-for-byte (SHARED-FAILURE). Real pdflatex passes
the characters through to the `.idx` and the index typesets `\delta` in typewriter (issue #354).

**Perl behavior**: `\index{\verb|\delta|}` → error + empty `<indexphrase/>` (the `|` eats the
phrase); `\index{\verb+\delta+}` → empty `<verbatim/>` + leaked math δ. **Rust behavior**:
`process_index_phrases` (`latex_constructs.rs`) intercepts a `\verb`/`\verb*` token in its scan
loop, consumes the whole `\verb<D>body<D>` run atomically — BEFORE the `!`/`@`/`|` split can see
the delimiter — `untex`+`Explode!`s the body back to catcode-OTHER literals (so the digested body
renders as typewriter text rather than re-expanding), and emits `\@internal@text@verb{star}{D}{body}`.
Both delimiter forms and `\verb*` now render `<verbatim font="typewriter">…</verbatim>` with no
error, and the interception composes with the `!` subentry split (`grp!\verb|sub|` → a plain `grp`
head + a verbatim `sub` subentry).

**Why**: a kernel-quality gap, not a TeX-semantics change — real LaTeX+makeindex typesets the
`\verb` body in typewriter; both LaTeXML engines simply lost it in the sanitized re-tokenization.
The fix is one locus (`process_index_phrases`) and halos across every `\verb`-in-`\index` paper.

**Witnesses**: issue #354 (split out of #347). `\index{\verb+\delta+}`, `\index{\verb|\delta|}`,
`\index{\verb*|a b|}`, `\index{grp!\verb|sub|}` — all render typewriter, 0 errors, where Perl 0.8.8
errors/loses the phrase.

**Upstream**: worth filing against `brucemiller/LaTeXML` (same SanitizedVerbatim/`\verb` gap).

**Guards**: `06_cluster_regressions::cluster_verb_in_index_renders_typewriter`.

### 120. A `\label` on a `\nonumber` eqnarray row references the equation number

**Perl** binds a `\label` placed right after `\begin{eqnarray}` (before the first
`\\`) to the environment's first row. When that row is `\nonumber`, LaTeXML gives
it an unnumbered id (`<ltx:equation xml:id="S0.Ex1">`) with no refnum, while the
number lands on a later numbered row (`S0.E1`). CrossRef's `generateRef` then finds
no refnum on the labelled row, walks its ancestors still empty, retries with
`show="title"`, and returns the nearest ancestor title — the **document element's**,
i.e. the paper title — as the visible `\ref` link text. pdflatex disagrees: it
steps the `equation` counter once at `\begin{eqnarray}`, so `\@currentlabel` is
already `1` when `\label` runs (before the `\nonumber` retraction), and the `.aux`
records `\newlabel{eqx}{{1}…}` → `\ref` is **1**. latexml-oxide reproduced Perl's
title-leak exactly (SHARED-FAILURE, verified same-host on 0.8.8).

**Rust behavior**: during Scan (`collect_common`, `group_sibling_refnum`), a
labelled `<ltx:equation>` that carries no refnum of its own and sits inside an
`<ltx:equationgroup>` inherits its refnum from the nearest numbered sibling row —
following-first (the counter, captured at `\label` time, points at the next number
to be shown), else preceding. Only the ObjectDB entry gains the number; the
document row still shows no `<ltx:tag>`, so nothing new is displayed. `\ref` then
renders "1" as an `ltx_ref_tag`, byte-identical to a normal numbered-equation ref
(same `title="In <context>"` breadcrumb tooltip both Perl and Rust already emit).

**Why**: matching the published PDF's cross-reference beats leaking the paper title
into the body; pdflatex's `\ref` value ("1") is the ground truth, and Perl's
title-fallback is the defect.

**Witness**: arXiv 2308.06222 (html_feedback #94) — a `revtex4-2` PRL whose
`\Eq{EqDefSSH}` (`\def\Eq#1{Eq.~(\ref{#1})}`) referenced the first `\bea\label{…}`
`\nonumber` equation and rendered the whole title "High-temperature
superconductivity induced by the Su-Schrieffer-Heeger electron-phonon coupling";
94 such refs, now all "1"…"N". pdflatex ground truth: `\newlabel{eqx}{{1}{1}…}`.
Shared upstream bug recorded as KNOWN_PERL_ERRORS #84.

**Guard**: `06_cluster_regressions::cluster_eqnarray_nonumber_label_ref_is_the_number`.

### 121. Alignment declarations in `\abstractname` are stripped from the abstract's `name`

**Perl** extracts the abstract heading via `getFrontmatterName` → `DigestText(\lx@abstract@name)`,
and `\lx@abstract@name` is `\format@title@abstract{\abstractname}` with `\format@title@abstract`
defined as the identity hook `#1` (`latex_constructs.pool.ltxml` L1146-1148). When a document
redefines `\renewcommand{\abstractname}{\centering {\large Abstract}}`, digesting the name runs
`\centering` — a `DefConstructor` (L1237) — whose **reversion** serializes back as the literal
text `\centering` when the digested box is flattened into the text-only `name=` attribute. So both
engines emit `<ltx:abstract name="\centeringAbstract">`, and the XSLT renders `<h6
class="ltx_title ltx_title_abstract">\centeringAbstract</h6>` (SHARED-FAILURE, verified same-host
on Perl 0.8.8: identical core XML and identical post-processed HTML). Font-size/series primitives
(`\large`, `\bfseries`) already produce no text leak — only the alignment *constructors* do.

**Rust behavior**: the `\format@title@abstract` hook — designed for exactly this ("# Redefine")
and used only during abstract-name extraction — neutralizes the alignment declarations inside a
group: `{\let\centering\relax\let\raggedright\relax\let\raggedleft\relax#1}`. The name digests to
the clean label `<ltx:abstract name="Abstract">`.

**Why**: the `name` is a plain-text label; an alignment declaration has no textual content and
must not leak its reversion into it. The fix mirrors LaTeXML's own `titlepage` precedent, which
does `Let('\centering', '\relax')` (L1168) for the same reason — alignment declarations should not
fire while frontmatter is being captured. It halos across every paper that decorates
`\abstractname` with `\centering`/`\raggedright`/`\raggedleft`, at one designated hook rather than
per-paper. Perl would benefit identically (upstream filing pending, owned by maintainer).

**Witness**: arXiv 2312.14226 (html_feedback #6870, aistats2024 class) — abstract heading rendered
`\centeringAbstract`; now "Abstract". Shared upstream bug recorded as KNOWN_PERL_ERRORS #87.

**Guard**: `06_cluster_frontmatter::frontmatter_abstract_centering_name`.

### 123. natbib citations of a numeric `.bbl` render as the bracketed number

**Perl** renders such citations as the raw citation key. When natbib is loaded in
its default author-year mode but the `.bbl` is numeric — plain `\bibitem{key}`
with no `[author(year)]` label, as `\bibliographystyle{unsrt}`/`plain` produce —
each `\cite` freezes an author-year `<ltx:bibref show="Authors Phrase1YearPhrase2">`
at digest time (natbib's mode isn't yet numeric, especially when the
`\bibliographystyle` sits AFTER the cites). The numeric `<ltx:bibitem>` carries a
`number`/`refnum` tag but no author/year metadata, only a `keytag`. Post-processing
`CrossRef.pm::make_bibcite` L542 keeps the author-year format
(`$show = 'refnum' unless … || $keytag;` — the `|| $keytag` guard is always
satisfied), so the citation prints the key (`alpha ()` / `alpha `). latexml-oxide
reproduced this exactly (SHARED-FAILURE, verified same-host on 0.8.8).

**Rust behavior**: in `CrossRef::make_bibcite` (`latexml_post/src/crossref.rs`, called by `fill_in_bibrefs`),
when a bibref's frozen `show` wants author-year yet EVERY cited entry is
numeric-only (a `number`/`refnum`, no real `authors`/`fullauthors`/`year`), the
citation collapses to natbib's numeric form: the bracketed number `[N]`, or `[N, M]`
for a multi-key `\cite` (each number a separate link). The brackets are added only
when the frozen author-year delimiters lived inside the bibref (`\cite`/`\citet`,
whose `show` has a Phrase after Year); `\citep`, whose parens are sibling text,
keeps its own delimiter.

**Why**: this mirrors natbib's own algorithm. On a numeric `.bbl`, real
pdflatex/bibtex write `\NAT@force@numbers` into the `.aux`, forcing numbers mode
globally so every `\cite` prints `[N]` regardless of a late `\bibliographystyle`.
The published PDF is the ground truth (`Text citing [1] and also [2] and both
[1, 2].`); Perl's raw-key output is the defect.

**Witness**: arXiv 2308.06262 (html_feedback #62) — a NeurIPS-2023 paper
(`neurips_2023.sty` loads natbib, `\bibliographystyle{unsrt}` after 263 `\cite`s,
numeric `main.bbl`). Every citation rendered `key ()`; now all render `[N]`/`[N, …]`,
byte-matching the golden pdflatex `.bbl`+`.aux`. Shared upstream bug recorded as
KNOWN_PERL_ERRORS #89.

**Guard**: `06_cluster_bibliography::cluster_bib_natbib_late_numeric_style_forces_numbers`.

### 124. Content injected into `\@maketitle` is recovered, not discarded

**Perl** discards `\@maketitle` wholesale. LaTeXML replaces the LaTeX kernel's
`\maketitle`→`\@maketitle` typesetting pipeline with its own frontmatter model:
`\maketitle` deposits the separately-captured title/author/date
(`\lx@frontmatterhere`) and then `\global\let\@maketitle\relax`
(`latex_constructs.pool.ltxml` L1105), with the source comment (L1094) admitting
"In case `\@maketitle` defines these — we can't yet emulate that." So content a
document appends to `\@maketitle` — a teaser figure, an epigraph, a banner — is
silently dropped, and any `\ref` to a `\label` inside it renders the raw internal
key ("LABEL:fig:teaser"). latexml-oxide reproduced this exactly (SHARED-FAILURE,
verified same-host on 0.8.8: both engines drop the figure).

**Rust behavior**: `\@maketitle` is predefined empty (so `\g@addto@macro\@maketitle`
appends cleanly — LaTeXML never reimplements the title *layout*, so `\@maketitle`
is otherwise undefined and appending to it warns "not expandable" and leaves a
self-reference), and `\maketitle` gains a `\lx@deposit@maketitle` step — after
`\lx@frontmatterhere`, before `\global\let\@maketitle\relax` — that deposits
`\@maketitle`'s accumulated content in a title-neutralized group
(`\let\@title\@empty\let\@author\@empty\let\@date\@empty\let\@thanks\@empty`). An
`\ifx\@maketitle\@empty` guard makes it a no-op for the vast majority of papers.
Injected definitions execute LaTeX-scoped (group-local, as real `\maketitle`'s
`\begingroup` does); injected content deposits.

**Why**: real pdflatex runs `\@maketitle`, so the teaser figure appears right
below the title and its `\ref` resolves to the figure number. Matching the
published PDF beats dropping the figure and leaking the internal label key. The
title-neutralization reuses the same technique as the `\format@title@abstract`
fix (#121): run the macro with the title-producing pieces neutralized so only the
injected content survives, rather than fragile token-parsing.

**Witness**: arXiv 2506.23854 (html_feedback #4281) — a teaser figure injected via
`\g@addto@macro\@maketitle{\begin{figure}…\label{fig:teaser}…\end{figure}}`; the
figure vanished and `\figref{fig:teaser}` rendered "Fig. LABEL:fig:teaser". Now the
figure renders (`xml:id="S0.F1"`) and the reference resolves to "Fig. 1". Shared
upstream bug recorded as KNOWN_PERL_ERRORS #90.

Second witness, a distinct authoring path into the same mechanism: arXiv 2606.25280
(html_feedback #6675) uses `titlepic.sty`, which does not *append* to `\@maketitle`
but *redefines* it wholesale (`\renewcommand\@maketitle{…{\centering\@titlepic\par}…}`)
with the teaser `\captionof{figure}`+`\label` held in `\@titlepic`. Because the
redefinition makes `\@maketitle` non-empty, the `\ifx\@maketitle\@empty` guard falls
through and `\lx@deposit@maketitle` runs it — so the figure survives and takes number 1
here too, while production ar5iv (Perl) still drops it. (Author-local `\def\name`
*inside* a redefined `\@maketitle` remains unsupported — that needs the author block, not
just the injected content; KNOWN_PERL_ERRORS #47.)

**Guard**: `06_cluster_frontmatter::frontmatter_maketitle_injected_figure_survives`
(append path); `06_cluster_frontmatter::frontmatter_titlepic_redefined_maketitle_figure_survives`
(titlepic redefine path).
### 122. A `font="bold"` wrapping an entire author name is unwrapped for coherence

**Perl** captures semantic creators from `\author`, not the class's visual title
layout. Classes like `neurips_2023` bold the whole author block with a block-level
`\bf` in their `\@maketitle` (`\begin{tabular}[t]{c}\bf…\@author\end{tabular}`),
which LaTeXML does not emulate. So when a paper `\textbf`s only *some* author name
lines and relies on that class `\bf` for the rest, LaTeXML renders the block
incoherently — bold on the `\textbf` lines, plain on the others. Both engines emit
`<ltx:personname><ltx:text font="bold">Name</ltx:text></ltx:personname>` on the
explicit-`\textbf` lines and a bare `<ltx:personname>Name</ltx:personname>` on the
rest (SHARED-FAILURE, byte-identical same-host on Perl 0.8.8).

**Rust behavior**: an `ltx:personname` `afterClose` handler (`unwrap_whole_name_bold`)
unwraps a personname whose sole meaningful child is a *pure* bold `<ltx:text>` (decoded
font series=bold, otherwise the default upright serif) — the case that would serialize
as exactly `font="bold"`. A trailing reference marker (`\footnotemark`/`\thanks` →
`<ltx:note>`) or a misused-`\\` `<ltx:break>` is skipped, not counted as content, so it
does not block the unwrap. Mixed styling (bold-italic, bold-sans, bold on only part of
the name) is left untouched. Every author then renders in the same weight.

**Why**: bolding a whole author name is presentational author-block styling, not
semantic content; a class that bolds the block does so uniformly, so partial bold is an
artifact of inconsistent source, never authorial intent (a corresponding/presenting
author is marked by a superscript, not by name-weight). Normalizing to plain restores
the coherence the reporter asked for and matches how ar5iv renders author names for the
overwhelming majority of classes. One general rule at the personname seam, not a
per-class binding. User-directed 2026-08-16 (chosen over the per-class "emulate the
class bold" alternative).

**Witness**: arXiv 2308.06262 (html_feedback #61, neurips_2023) — 7 authors rendered as
3 plain + 4 bold; now all 7 plain. Also arXiv 2507.06670 (acl) "Zhou Zhao", where a
`\textbf{Zhou Zhao} \footnotemark[2]` bold name kept its footnotemark marker sibling —
now unwrapped too. Shared upstream bug recorded as KNOWN_PERL_ERRORS #88.

**Guard**: `06_cluster_frontmatter::frontmatter_neurips_author_bold_coherent`.

### 125. A wrapper box merges its `class` onto a single block child, not overwrites it

**Perl**'s `insertBlock` (`TeX_Box.pool.ltxml` L489-493) absorbs a box (minipage,
parbox, …) onto its content when that content is a single block the context can
hold directly: it copies the box's attributes onto the child and unwraps the
wrapper. For `class` it uses `setAttribute(class => …)`, which **overwrites**.
LaTeXML has a separate `addClass` (used elsewhere in the same file, L887/892/896)
that merges the space-separated set — but `insertBlock` doesn't use it. So a
`lstlisting` (or a `minted` block, which routes through the same listings display)
that is the **sole** content of a `minipage` becomes `<listing class="ltx_minipage">`,
**losing `ltx_lstlisting`** — and with it the whitespace-preserving CSS keyed on
that class, so its indentation collapses. latexml-oxide reproduced this exactly
(SHARED-FAILURE, verified same-host: Perl 0.8.8 also emits `class="ltx_minipage"`).

**Rust behavior**: `insert_block` (`base_utilities.rs`) treats `class` as the
space-separated set it is — it `add_class`es the wrapper's class onto the child
instead of overwriting, so the child keeps its own semantic class and *gains* the
wrapper's: `<listing class="ltx_lstlisting ltx_minipage" vattach="…" width="…">`.
Every other absorbed attribute (`width`, `vattach`, …) is still `set_attribute`d as
before; only `class` merges. This is the same `addClass`-vs-`setAttribute`
distinction LaTeXML already draws, applied at the one site that forgot it.

**Why**: the child's class carries its rendering contract (`ltx_lstlisting` →
`white-space` handling for code); a wrapper that borrows the child's element must
not erase it. pdflatex shows the code indented; keeping `ltx_lstlisting` is what
preserves that in HTML.

**Scope note**: this is the *single-minipage-in-a-float* path. A float with
*multiple* side-by-side minipages takes the flex-figure layout, where the minipage
is a separate `ltx_figure_panel` wrapper and the listing already keeps its class —
so this fix and that path are independent. (The whitespace loss reported for the
flex path in html_feedback#6632, arXiv:2605.03143, was a *separate*, CSS-only
cause: arXiv's deployed `arxiv-html-papers-theme` layer sets a bare
`.ltx_listingline{white-space:normal}` that overrides ar5iv's `nowrap` by
cascade-layer order — not an engine issue.)

**Witness**: arXiv:2605.03143 (a single `\begin{minipage}…\begin{minted}` in a
`figure`); before, `<listing class="ltx_minipage">`, after,
`<listing class="ltx_lstlisting ltx_minipage">`.

**Upstream**: worth filing against `brucemiller/LaTeXML` (`insertBlock` should
`addClass`, not `setAttribute`, for the `class` key).

**Connected behavior**: the same merge preserves any absorbed block's own class,
not just listings — e.g. an algorithm float (`ltx_float_algorithm`) that is a
minipage panel now keeps `ltx_float_algorithm` alongside `ltx_minipage` instead of
being clobbered to bare `ltx_minipage` (golden `tests/complex/figure_mixed_content.xml`).

**Guard**: `cluster_sizing::listing_in_minipage_keeps_class::listing_sole_content_of_minipage_keeps_lstlisting_class`;
the `80_complex::figure_mixed_content_test` golden pins the algorithm-float case.

### 126. Numeric/superscript natbib `.bbl` labels the References with `[N]`, not author-year

**Perl**'s `\NAT@wrout` (`natbib.sty.ltxml` L609-620) formats each `\bibitem`'s
reference-list `refnum` from `CITE_STYLE`, but its numeric branch is gated on
`$style eq 'number'` (**singular**) — a value `CITE_STYLE` never holds (it is
`'numbers'`/`'super'`/`'authoryear'`). The only route to number style is the
empty-author/year fallback (L612: `$style = 'number' if IsEmpty($authors) ||
IsEmpty($year)`). So a pre-formatted numeric `.bbl` — the `thebibliography` /
`\bibitem` path, distinct from the `.bib` / MakeBibliography path in #116 — whose
`\bibitem[{Name(Year)}]{key}` label carries an author AND a year keeps an
author-year label (`Shor [1994]`) even though the inline `\cite` correctly shows
`[N]`. SHARED-FAILURE: Perl 0.8.8 emits the identical `Shor [1994]` (numbers) /
`Shor 1994` (super), verified same-host.

**Rust behavior**: `\NAT@wrout` (`natbib_sty.rs`) forces number style whenever
`CITE_STYLE` is `'numbers'` or `'super'` (as well as the empty author/year
fallback), so every entry's `refnum` is the bracketed entry number `[N]` (the bare
number in super mode, whose `CITE_OPEN`/`CLOSE` are empty) — consistent with the
inline `[N]` cites and the published PDF. `authoryear` mode is unchanged: an entry
with an author+year keeps `Author (Year)`, and only an empty author or year falls
back to the number.

**Why**: apsrev4-2 / `[numbers]natbib` is a numeric style; pdflatex+bibtex render
the whole reference list as `[N]`. Matching the PDF beats a partial author-year
list that contradicts its own inline `[N]` cites, and Perl's singular-`'number'`
guard is the defect.

**Witness**: arXiv 2410.05202 (html_feedback#4295) — `revtex4-2` / apsrev4-2, 57
entries. Before, entries with a year (`[1]` Shor(1994), `[4]` Reiher et al.(2017))
showed an author-year label while genuinely year-less entries (`[3]` Gidney and
Ekerå, empty `()`) already showed `[3]`; now all 57 are `[1]`…`[57]`, matching the
PDF.

**Upstream**: worth filing against `brucemiller/LaTeXML` (the singular-`'number'`
guard mislabels every numeric-mode `.bbl` with authors+years).

**Guard**: `06_cluster_bibliography::cluster_bib_natbib_bbl_numeric_refnum`
(numbers + super go numeric; the `authoryear` control keeps `Shor (1994)`).

### 127. minted honours `escapeinside`, so a `\label` inside code registers on its line

**Perl** ships **no** `minted` binding — it loads the raw `minted.sty`, which
(without shell-escape/pygmentize) processes the body as ordinary LaTeX. A
`\label` inside the code runs, but the code is not rendered as a listing and the
label bubbles to the document root with no line to attach to. latexml-oxide
instead provides a richer `minted` binding (`latexml_contrib::minted_sty`) routed
through the `listings` substrate, producing a real `<ltx:listing>` with numbered
`<ltx:listingline>`s.

**Rust behavior**: that binding used to *drop* minted's `[options]` — it called
`lst_process_display` without activating them — so `escapeinside=!!` never reached
the listings tokenizer. An inline `!$\label{line:x}$!` was emitted as literal code
characters, its `\label` never ran, the line label was never registered, and
`\ref{line:x}` rendered as an empty `ltx_missing_label` (earlier: the raw key
`LABEL:line:x`). `\begin{minted}[opts]{lang}` now feeds `opts` through
`\lstset{…}` — the same activation `\begin{lstlisting}` uses — so minted's
`escapeinside`/`mathescape` (shared verbatim with listings) take effect and the
`\label` attaches to its `<ltx:listingline>`; `\ref` resolves and links to the
code line. Minted-only keys (`linenos`, `fontsize`, …) are stored harmlessly and
ignored; the `{language}` arg is left unapplied (current language-agnostic
rendering is preserved).

**Why**: matching the published PDF, where a line label references and links to
its code line. This completes our Rust-only minted binding rather than diverging
from a Perl behavior (Perl has none to match).

**Residual (shared with Perl, out of scope)**: the reference shows the line's
`xml:id`, not its printed line number, because `\@lst@startline` uses
`RefStepID('lstnumber')` (id only, no refnum) in *both* engines
(`listings.sty.ltxml:1546`); giving the line a numeric refnum is a separate
listings-wide change.

**Witness**: arXiv:2308.03276 (html_feedback#1028) — `\begin{minted}[linenos,
escapeinside=!!]{python}` with `!$\label{line:world}$!`; before, `\ref{line:world}`
vanished (empty `ltx_missing_label`); now it registers on the line and links.

**Guard**: `06_cluster_regressions::minted_escapeinside_label_registers_on_the_code_line`.
### 128. IEEEtran multi-row author grid emits creators in row-major (reading) order

**Perl**: an IEEEtran conference `\author{}` can lay authors out as a 2-D grid where
`\and` starts a new COLUMN and a top-level `\\` a new ROW within a column —
`\IEEEauthorblockN{Zhao}…\\ \IEEEauthorblockN{Ding}… \and \IEEEauthorblockN{Chen}…\\
\IEEEauthorblockN{Kong}… \and …` (arXiv:2403.16405, 6 authors in 3 columns × 2 rows).
Each `\IEEEauthorblockN` emits its creator in token (declaration) order, i.e. down each
column, so the creator sequence is **column-major** (Zhao, Ding, Chen, Kong, Huang,
Zhang) — scrambling the **row-major reading order** the PDF and the arXiv
`citation_author` metadata show (Zhao, Chen, Huang, Ding, Kong, Zhang). Same-host Perl
LaTeXML 0.8.8 mis-handles the same grid (SHARED-FAILURE, Perl-origin).

**Rust behavior**: the IEEEtran `\author` dispatch (`ieeetran_cls.rs`) transposes a
genuine grid to row-major before emitting creators (`transpose_ieee_author_grid`). The
transpose is guarded TIGHTLY: it fires only for a REGULAR grid — ≥2 `\and` columns,
every column the SAME number of top-level `\\` rows, that count ≥2. A single-row `\and`
author list (no top-level `\\`) and `\\` used only INSIDE `\IEEEauthorblockA{…\\…}`
(nested, brace-depth > 0) are left in their declared order untouched.

**Why**: the reading/metadata order is the authoritative author sequence (it drives
citation, attribution, and search indexing); a column-major linearization silently
reorders authors 2…n−1. The tight regular-grid guard keeps every non-grid IEEE block
(the common case) byte-identical.

**Witness**: arXiv:2403.16405 (IEEEtran, 6 authors) — was Zhao, Ding, Chen, Kong, Huang,
Zhang; now Zhao, Chen, Huang, Ding, Kong, Zhang. Shared upstream bug recorded as
KNOWN_PERL_ERRORS #94.

**Guard**: `06_cluster_frontmatter::frontmatter_ieee_author_grid_transpose` (grid
transposed + a single-row control proven un-reordered).

### 129. Nested inline-math superscript author markers (`$^\text{$...$}$`) no longer desync math mode

**Perl**: in an author/affiliation block using `^`/`\textsuperscript` markers,
`\lx@add@authors` takes the marker (withsup) branch and `\let`s `^` onto
`\lx@request@frontmatter@annotation`, whose bare `{}` argument reads a single token.
For a marker whose operand is a control sequence carrying its own group —
`^\text{...}`, which real LaTeX math reads as `^{\text{...}}` (`\text` grabbing its
`{...}`) — the `{}` read grabs only `\text` and orphans the following `{...}`. Inside
the marker's own inline math that stray `{...}` leaves a brace-group frame on top of
the digestion stack, so the marker's closing `$` fires `\lx@end@inline@math` against
the brace group rather than the math: `Error:unexpected:\lx@end@inline@math Attempt to
end mode math`. Nested markers (`$^\text{$...$}$`, e.g. an icon built from
`\newcommand`-chained `$^\text{...}$`) cascade the error and merge every creator into
one garbled `<personname>`. **Same-host Perl LaTeXML 0.8.8 errs identically**
(SHARED-FAILURE, Perl-origin; KNOWN_PERL_ERRORS #95).

**Rust**: the two `^`-hijack wrappers (`\lx@sup@request@affiliation`,
`\lx@sup@setlabel@affiliation`; installed on the superscript catcode by
`\lx@let@superscript`, not Perl's `\let^`, which tex.web §1215 rejects — #313) are
primitives that read a FULL superscript operand
(`read_frontmatter_sup_operand`), mirroring `TeX_Math` `scriptHandler`: a braced
operand is taken whole, and a bare leading control sequence keeps its following
`{...}` group. So `\text{...}` — and any `$...$` nested inside it, at any depth — is
captured together and undigested, the surrounding inline math stays balanced, and the
now-empty `$...$` collapses so the marker links the author to the shared affiliation
instead of leaving a stray box. Numeric/char markers (`^1`) and already-braced markers
(`^{...}`, `\textsuperscript{...}`) read exactly as before.

**Why**: nested `$...$`/`\text{}` is a fundamental LaTeX capability authors do use
(icons-as-markers here); erroring and merging creators is never the right outcome. The
change is surgical — only a bare `^`-control-sequence-with-argument operand behaves
differently — and every existing frontmatter marker test is unaffected.

**Witness**: arXiv:2403.11905 (html_feedback#1021) — `\handPointerZ` =
`$^\text{\mouseOne}$`, `\mouseOne` = `$^\text{\faMousePointer}$` (double-nested icon
markers). Was 6× "Attempt to end mode math" + one merged personname; now 0 errors.

**Residual (separate, pre-existing)**: an author line whose ONLY marker is delivered
through a macro (`\handPointerZ`, no literal `^`) is still classified "no marker" and
merged into the previous `\and`-separated author — `\lx@add@authors` conflates `\and`
(new author) with `\\` (continuation) and keys classification on a LITERAL `^`. That is
the F2 author-classification gap (shared with Perl), independent of this math-mode fix,
and is why arXiv:2403.11905's own creators still merge even though the error cascade is
gone.

**Guard**: `06_cluster_frontmatter::frontmatter_nested_math_author_marker` (two clean
creators linked to a shared affiliation via `$^\text{$\star$}$` markers, 0 errors).

### 130. `\twocolumn[header]` scopes the spanning-header's font/alignment declarations

**Perl**: `\twocolumn[]` is `\ifx.#1.\else\par\noindent#1\fi\par`
(`latex_constructs.pool.ltxml` L1015) — the optional argument is spliced into the
stream **unscoped**. A font or alignment declaration inside the header
(`\centering`, `\Large`, …) therefore runs on into the body. Real LaTeX does not:
`\twocolumn[#1]` routes through `\@topnewpage`, which typesets `#1` in a box, so the
header's declarations are bounded to it. Perl rarely exhibits the leak because it
tends not to render the headers that trigger it — e.g. cvpr's
`\maketitlesupplementary` (`\twocolumn[\centering\Large … Supplementary Material …]`)
is undefined under Perl (no cvpr binding, raw loading off), so `\maketitlesupplementary`
errors and the `\Large` never runs.

**Rust**: our cvpr binding raw-loads the real `cvpr.sty`, so `\maketitlesupplementary`
*does* run and the header renders at `\Large` — but the unscoped splice then leaked
that `\Large` into the entire Supplementary Material section (every heading and
paragraph `font-size:144%`). We wrap the header in a group with its own `\par`
(`\ifx.#1.\else\par{\noindent#1\par}\fi\par` — `\noindent` inside the group so the
header paragraph keeps its non-indent), matching the box scope real LaTeX
gives `\@topnewpage`: the header keeps its `\Large`, the body returns to normal size.
This surpasses Perl's simplification. Witness html_feedback#6638 (arXiv:2511.14625v1).

**Guard**: `06_cluster_regressions::twocolumn_optional_header_font_does_not_leak_into_body`.

### 131. cleveref `\cref` names custom `\newtheorem` types by their heading

**Perl & Rust (shared limit)**: real cleveref patches LaTeX's `\@ynthm`/`\@xnthm`/`\@othm`
so that `\newtheorem{arch}{Architecture}` auto-registers "Architecture" as the cleveref
type name — `\cref{...}` then renders "Architecture 1" (as in the PDF). LaTeXML's
`\newtheorem` is a native primitive (`define_new_theorem`) that never routes through
those patches, so `\cref@arch@name` stays undefined. Both engines therefore emit the
type-tag empty (dropped by `removeEmptyElement`) and `\cref` degrades to a bare
"1" — the `creftype` component of `show="creftype~refnum"` resolves to nothing.

**Rust (surpass-Perl)**: two changes, in precedence order.

1. `\crefname`/`\Crefname` are now **real definitions** (`cleveref_sty.rs::cref_define_name`,
   a clean port of raw cleveref's `\@crefname` — the raw macros' `\toksdef`/`\expandafter`
   chains mis-consumed tokens here, so they had been no-op stubs). An explicit
   `\crefname{arch}{…}` therefore populates `\cref@arch@name` and takes precedence, exactly
   as in LaTeX. The cross-variant `\MakeUppercase` derivation (deriving `\Cref@…` from a lone
   `\crefname`) is not reproduced — provide `\Crefname` for the capitalised form — matching
   the `thmtools_sty.rs` `\declaretheorem[refname=]` precedent.
2. When no explicit name is set, the `creftype`/`creftypecap` formatters
   (`cleveref_sty.rs::cleverref_type_name`) fall back to the theorem heading
   `\lx@name@<type>` (which `define_new_theorem` already stores), so a bare
   `\newtheorem{arch}{Architecture}` renders "Architecture 1" — matching the PDF and
   exceeding Perl. Only the **singular** names get the fallback (cleveref's theorem patches
   set only `cref@<type>@name@preamble`, never a plural). The heading is emitted verbatim:
   cleveref's first-letter `capitalize` case transform is not reproduced, so a lowercase
   `\cref` under the default (non-`capitalize`) option keeps the heading's own case.

`\lx@name@<type>` is also defined by `\floatname`/`\newfloat` (`float_sty.rs`), so the
fallback additionally auto-names custom floats — matching real cleveref's float auto-naming,
so it is beneficial, not a leak; standard `figure`/`table`/`equation` keep their raw-cleveref
primary name (the fallback stays dormant).

Witness html_feedback#140 (arXiv:2305.10391v2 — `\usepackage[capitalize,nameinlink]{cleveref}`
+ `\newtheorem{arch}{Architecture}`).

Upstream context: cleveref's `\newtheorem` naming is a long-standing sore spot with newer
LaTeX. TeX Live 2025 broke it so that *every* `\cref` to a theorem prints the same name
([arXiv TeX Live FAQ](https://info.arxiv.org/help/faq/texlive.html), "cleveref"); arXiv's
workaround is `\AddToHook{env/<thm>/begin}{\crefalias{<counter>}{<thm>}}`. LaTeXML is immune
because it keys `creftype` on each theorem's own type (via `type_tag_formatter`), not on the
shared counter, so it yields distinct correct names and needs no workaround — standard names
(`theorem`, `proposition`, …) still come from cleveref's raw defaults; only non-standard names
(`arch`) use the heading fallback. `\crefalias` remains a harmless no-op here.

**Guards**: `06_cluster_regressions::cleveref_custom_theorem_cref_shows_heading_name`
(heading fallback) and `::cleveref_explicit_crefname_overrides_heading` (explicit name wins).

### 132. Main-file selection: a matching `.bbl` sibling outranks the pdf-`\includegraphics` heuristic

**Perl**: `Pack.pm::detect_source` orders its multi-candidate tie-breakers
(L188-213) as: max-likelihood → shallowest depth → **pdf-`\includegraphics`
(heuristic 2)** → **matching `.bbl` (heuristic 3)** → common name
(`main`/`ms`/`paper`) → alphabetical. When the true main **delegates all its
figures** to `\input`-ed section files — so it carries no direct
`\includegraphics` — but a bundled class **template / how-to / supplement** does
contain an example `\includegraphics{fig.png}`, heuristic 2 narrows the candidate
set to the decoy and the decisive `.bbl` tie-break never runs. Perl mis-selects
the decoy as the top-level file.

**Rust**: run the `.bbl`-sibling heuristic **before** the pdf-`\includegraphics`
heuristic (`main_tex.rs`). arXiv requires the compiled `<main>.bbl` to be bundled
(BibTeX is not re-run at conversion time), so a candidate with a **matching
`.bbl`** is the single strongest fingerprint of the real top-level file. When
more than one candidate carries a `.bbl` (e.g. 2506.05564, 2401.07129) the set
survives and the later heuristics (pdf-include, common-name, alphabetical) still
discriminate exactly as before, so the reorder only fires on the decoy class. A
133-paper blast-radius sweep (21 open html_feedback autotex issues + 22 closed +
90 random across 7 categories) showed **0 regressions**: the reorder changed a
pick only on the 8 witnesses below, always from decoy → real main.

This surpasses Perl on a **SHARED-FAILURE** cluster (Perl and the old Rust port
both mis-selected). Witnesses (all confirmed against production arXiv HTML titles
and vendored Perl `detect_source`): 2407.05010 (#1721, `New_IEEEtran_how-to.tex`),
2409.06957 (#6100, `iclr2025_conference.tex`), 2409.02543 (#5867, `supp.tex`),
2406.08688 (#5476, `IEEEtran_template.tex`), 2310.02368 (#4156, IEEE template),
2505.05625 (#4067, `supplementary.tex`), 2410.12672 (#2369, `iclr2025_conference.tex`),
2410.01562 (#2224, `template.tex`).

The companion **parity** fix — argument-anchoring the pdf-`\includegraphics`
probe (`has_pdftex_marker`) to Perl's exact regex — is *not* a divergence; it
restores fidelity where the old loose `contains` check false-positived
(`KNOWN_PERL_ERRORS.md#97`), recovering 2401.17263 (#442) and 2403.17719 (#859),
which Perl already selected correctly.

**Upstream**: not yet filed to `brucemiller/LaTeXML` (TODO — the same
heuristic-2-before-3 reorder would benefit Perl `Pack.pm`).

**Guards**: `main_tex::tests::bbl_sibling_outranks_pdf_include_marker` and
`::reorder_preserves_common_name_when_multiple_bbl`.

### 133. comment.sty detects `\end{comment}` mid-line, not only as a whole line

**RETRACTED 2026-09-05** (user directive: match pdflatex). comment.sty:186-193
compares each WHOLE line against `\end{name}` (`\ProcessCommentLine#1^^M{\def
\test{#1}\csarg\ifx{End…Test}\test`) with every special made innocent, so a
mid-line `…text.\end{comment}` is NOT an end: the comment runs to the end of
the file and pdflatex aborts with "File ended while scanning use of \next" and
no PDF — verified on the witness fixture `tests/cluster_regressions/
comment_midline_end.tex` and on each edge shape (leading spaces, trailing `%`,
text before or after: all abort; trailing spaces: end, TeX's line reader strips
them). The claim below that pdflatex keeps the 31-entry bibliography was wrong.
`comment_sty.rs` now ends only on such a line and reports TeX's error at EOF
(Perl's `^\s*\end{name}\s*$` is looser on leading spaces). Guards:
`00_tokenize::comment_test`, `06_cluster_bibliography::
comment_midline_end_runs_to_eof_like_pdflatex`,
`perfect_kernel_gemini::comment_self_terminating_hands_to_end`. Original text kept
for the record:

**Perl**: `comment.sty.ltxml`'s `defineExcluded` reads the body raw line-by-line
and stops only at a line that is *entirely* the end marker — `/^\s*\Q\end{name}\E\s*$/`
(L30). A comment whose closing sits at the end of a content line —
`…text.\end{comment}` — never matches, so the raw-line scan runs to **end of
file**, silently swallowing everything after the comment. The old Rust port
matched Perl exactly (`line.trim() == end_mark`).

**Rust**: match `\end{name}` **anywhere in a line** (allowing spaces between
`\end` and `{`, i.e. `\end {name}`), mirroring the in-tree verbatim.sty
`\verbatim@` scanner (`verbatim_sty.rs`), and drop the rest of that line (with an
`Info:unexpected:stuff`). Crucially, the scan looks only at the **code part** of
each line — an `\end` behind a `%` comment (`% […] \end{name}`) is NOT an end,
because comment.sty reads its body with `%` active as a comment. `comment_sty.rs`.

This surpasses Perl on a **SHARED-FAILURE**: both LaTeXML engines silently drop
the content. Witness **arXiv:2606.11493** — a proof wrapped in
`\begin{comment}…\(G(h_1)=0\).\end{comment}` swallowed the document's 31-`\bibitem`
`thebibliography`, so the paper rendered with **0** bibliography entries and **no
diagnostic** (R3b: silent bibliography loss); pdflatex compiles the same source
with the full 31-entry bibliography, and after the fix Rust does too. (comment.sty
itself is finicky about mid-line closings — reduced/synthetic cases can even
`Runaway` in pdflatex — so this is not a byte-for-byte pdflatex port but the
robust reading verbatim.sty already used, which recovers the real witness without
regressing the tokenize `comment` fixture's `%`-guarded edge cases.)

**Upstream**: not yet filed to `brucemiller/LaTeXML` (TODO — the same
mid-line-end scan would benefit Perl `comment.sty.ltxml`).

**Guard**: `06_cluster_bibliography::comment_midline_end_keeps_bibliography`.

### 134. `--urlstyle` defaults to `file`, not Perl's `server`

**Perl**: `Config.pm` L482 defaults `urlstyle` to `server`, so `latexml`'s
cross-reference URLs strip a trailing `index.<ext>` (a split document's landing
page links as `./`) out of the box.

**Rust**: `--urlstyle` accepts the same three values (`server` / `negotiated` /
`file`) and applies the identical `CrossRef::generateURL` transform (CrossRef.pm
L656-663), but **defaults to `file`** — full paths, nothing stripped. Passing
`--urlstyle=server` reproduces Perl's default exactly.

**Why**: latexml-oxide's north star is a self-contained artifact often viewed
straight off disk (`file://`), where `server`-style links (`./`, `dir/`) do not
resolve — a browser opening `file:///…/dir/` will not serve `dir/index.html`.
`file` is the safe default for that case (the very reason the issue reporter, the
BookML author, notes needing `--urlstyle=file` for `index.html`-buggy servers);
a real HTTP deployment opts into `server`/`negotiated`. Keeping `file` also
avoids changing the current cross-ref output for every split document. Approved
by the maintainer 2026-08-18 when resolving #656.

**Upstream**: n/a — a default choice, not a bug in Perl.

**Guard**: `cluster_xslt_split::urlstyle::default_is_file_style_full_paths` (and
the `server`/`negotiated` siblings prove the opt-in styles match Perl).

### 135. overpic renders a populated `<ltx:picture>` (not Perl's empty `tex=` stub)

**Decision:** The `overpic` environment emits a **populated** `<ltx:picture>` — the
`\includegraphics` background nested at the origin plus the body's `\put` overlays,
sized to the graphic (`unitlength = max(w,h)/100` in the default percent mode) —
rather than Perl's empty `<ltx:picture tex='…'/>`.

**Perl behavior** (`overpic.sty.ltxml`): emits an EMPTY `<ltx:picture>` carrying a
`tex=` attribute with the full overpic source, relying on the `PictureImages`
post-processor to render the whole thing (graphic + overlays) as one LaTeX+dvipng
image.

**Rationale:** that path is a dead end in Rust — `tex=` on `<ltx:picture>` is
suppressed unconditionally (divergence #21) and the LaTeXImages renderer is unwired
(Rust renders pictures as inline SVG from their CHILD elements). A faithful port
therefore renders NOTHING: no graphic, and `#body` (the overlays) is dropped. Since
Rust's `{picture}` + `\includegraphics` + `\put` already produce correct SVG, we
reproduce overpic.sty's OWN construction (`\OVP@picture` / `\OVP@calc@rel`) — box the
graphic, size a picture to it, `\put(0,0)` the graphic, run the body — routing
through the picture-nested-graphics SVG path (PR #675). Rust measures a boxed
`\includegraphics` (`\wd`/`\ht`/`\dp`) as pdfTeX does, which is what makes this
possible.

**Impact:** ~37 arXiv papers (44 html_feedback reports) whose overpic figures were
missing/blank now render the image + labels. Faithful to overpic.sty's percent-mode
coordinate math; the `grid` option (epic's `\grid`, no Rust binding) and the
`\Overpic` capital-O variant are unported (no report uses either).

**Upstream:** n/a — a Rust-rendering-model adaptation, not a Perl bug. Approved by
the maintainer 2026-08-18.

**Guard:** `cluster_package_guards::overpic_renders_graphic_and_overlays::{overpic_emits_populated_sized_picture_with_graphic_and_overlays, overpic_missing_natural_size_image_does_not_divide_by_zero}`.

### 136. `<?latexml nominal-font-size?>` PI persists a non-default `NOMINAL_FONT_SIZE`

**Decision:** When `NOMINAL_FONT_SIZE` differs from the 10pt default at document
finalization, emit a `<?latexml nominal-font-size="X"?>` processing instruction,
alongside the existing `<?latexml class=… package=…?>` metadata PIs. A default
(10pt) document emits nothing, so its output stays byte-identical.

**Perl behavior:** Perl never persists `NOMINAL_FONT_SIZE` — it is a digestion-only
value read by `Common/Font.pm::DEFSIZE` and then discarded. Post-processing has no
way to recover the body font size, so an external SVG sized in `em` units is scaled
against an assumed 10pt even when the class chose another size.

**Rationale:** an `em` is `NOMINAL_FONT_SIZE`pt, not always 10pt. a0poster (25),
BookML, and the `NNpt` class options move it off 10, and post-processing needs the
value to size font-relative external SVGs correctly. Exposing it as a PI (the
reporter's suggested form) makes it available without touching any element.
arXiv/html_feedback#683 (xworld21).

**Impact:** only documents whose class sets a non-default `NOMINAL_FONT_SIZE` gain
the PI. Emitting it also surfaced and fixed an `insert_pi` bug — a PI added after
the root element already exists was queued into the once-drained `pending` list and
silently lost; it is now inserted directly before the root, matching Perl
`Core/Document.pm::insertPI` ("a PI always lands before the root element").

**Upstream:** n/a — beyond-Perl (post-processing metadata Perl does not carry).
Approved by the maintainer 2026-08-18.

**Guard:** `cluster_sizing::delimiter_size_nominal_font::nominal_font_size_persisted_as_pi_only_when_non_default`.

### 137. `\subimport*`/`\subimport` accept an ABSOLUTE directory argument

**Perl behavior** (`import.sty.ltxml` L31-42, `\lx@append@path`): the directory
argument of `\subimport` is **always** `pathname_concat`'d onto the current lead
search path. For an absolute argument that yields `<lead>//abs/dir`, which never
resolves, so `\subimport*{/abs/dir/}{file}` reports `missing_file` — even though
real LaTeX (pdflatex, verified) opens the file. `\import`/`\import*`
(`\lx@set@path`) already special-case absolute paths; `\subimport` does not.

**Rust behavior**: `\lx@append@path` uses an absolute argument **verbatim**
(`pathname::canonical`, as `\lx@set@path` does), so `\subimport*{/abs/dir/}{file}`
resolves and matches pdflatex. A relative argument is still concatenated onto the
lead path (unchanged — the common `\subimport*{sub/}{file}` case), so no
currently-passing document changes: the fix only turns a `missing_file` failure
into a success (it cannot regress, since no document resolves an absolute
`\subimport` today).

**Why**: real LaTeX is the ground truth and it resolves the absolute path; both
LaTeXML engines diverging from `pdflatex` here is a file-resolution quality bug,
not a TeX-semantics choice. The reporter hit it converting a book whose chapters
`\subimport*` shared assets by absolute path. Approved by the maintainer 2026-08-19
(#697), conditional on the confirmed pdflatex behavior.

**Upstream**: Perl's `\lx@append@path` has the identical limitation and would
benefit from the same one-line fix — to be filed at `brucemiller/LaTeXML`.

**Guard**: `06_cluster_standalone_subfiles::subimport_absolute_path_resolves_like_real_latex`.

### 138. Subfigure panels with no explicit `{width}` share a row (not stacked full-width)

**Perl behavior** (`latex_constructs.pool.ltxml:3229-3349` `arrange_panels_and_breaks`,
`subcaption.sty.ltxml:104`, `subfig.sty.ltxml`): a panel's per-row width is its box
width. `\subcaptionbox` and subfig `\subfloat` carry no explicit `{width}`
(subcaption's `\subfloat` passes `{\columnwidth}`), so the panel box is sized to the
full float `\hsize`, and `arrange_panels` then gives each panel its own row. In a
two-column layout that renders every such panel at the full text width, stacked —
even though the PDF sets them side-by-side in one column. An explicit-width
`\begin{subfigure}{0.48\linewidth}` is unaffected (box = 0.48·\hsize → two per row).
Verified: Perl 0.8.8 emits `width=345.0pt` for the panels and stacks them, identical
to the pre-fix Rust.

**Rust behavior** (`latex_constructs.rs` `arrange_panels` + `sole_graphic_width`;
`subcaption_sty.rs` `subcaption_width_props`): when a panel is sized to the full
float width but wraps exactly one graphic narrower than the float, `arrange_panels`
uses the graphic's width for the per-row layout, so the panels share a row like an
explicit-width `{subfigure}{W}`. A `{0pt}` default width (subcaptionbox) no longer
pins the panel to zero. Only the row-arrangement threshold changes; the markup is
unchanged. Guarded to only ever *shrink* a full-width panel to its narrower content,
so it cannot widen or reflow a correctly-sized figure.

**Why**: the HTML is single-column, so a panel meant to sit beside its sibling in one
PDF column should share a row, not span the full text width. arXiv/html_feedback#6903
(two-column paper, subfigure figures rendering full-width). The content width is only
knowable post-digest (the graphic's box), so the fix lives in `arrange_panels`, not
the binding. Approved by the maintainer 2026-08-20.

**Upstream**: Perl's `arrange_panels_and_breaks` has the identical limitation and
would benefit from the same graphic-width fallback — to be filed at
`brucemiller/LaTeXML`.

**Guard**: `06_cluster_regressions::cluster_subfigure_panels_share_a_row_6903`.

### 139. A graphics `<img>` carries an explicit `aspect-ratio` (requested W/H)

**Perl behavior** (`Post/Graphics.pm` `setGraphicSrc`; `LaTeXML.css` flex rules): the
`<img>` gets `width`/`height` attributes (the *requested* size, from the
`\includegraphics` options and the file) plus a coarse
`ltx_img_{square,portrait,landscape}` class, but **no** `aspect-ratio`. The flex
subfigure CSS then caps `max-width` (`.ltx_flex_size_N .ltx_graphics`), which shrinks
the width and leaves the height attribute untouched — so a square figure renders as a
vertical ellipsoid (a sphere becomes an ellipse). brucemiller/LaTeXML#2392, still open;
Perl 0.8.8 distorts identically (verified: 288×476 for a 476² square under `flex_size_3`).

**Rust behavior** (`latexml_post/src/graphics.rs` `set_graphic_src`; owned `LaTeXML.css`):
`set_graphic_src` also emits `cssstyle="aspect-ratio:W/H"` from the requested
width/height (the same W/H it puts on the attributes), and the flex/minipage image
rules add `height:auto`. When `max-width` caps the width the browser recomputes the
height from that ratio, so the *requested* aspect ratio is preserved — not merely the
file's (which matters for `\includegraphics[width=…,height=…]` where they differ). All
five panels of the issue MWE render at ratio 1.000 (were 0.605 / 0.887).

**Why**: this is the design the upstream thread converged on (xworld21's
`aspect-ratio: @imagewidth / @imageheight` proposal, meeting brucemiller's requirement
that any width/height/min/max tweak preserve the *requested* ratio). Emitting the ratio
per-image is CSS-safe (inert unless a dimension is freed) and beyond Perl, which has not
implemented it. The coarse `ltx_img_*` classes still pick the flex layout.

**Upstream**: brucemiller/LaTeXML#2392 (open) — the same emission + `height:auto` would
fix Perl; not filed here.

**Guard**: `latexml_post::graphics::tests::set_graphic_src_emits_requested_aspect_ratio_2392`.

### 140. amsart up-front author/contact grid redistributes to the right author

**Perl behavior** (`ams_support.sty.ltxml` `\address`/`\email`; `Base_Utility.pool.ltxml`
`\lx@annotate@frontmatter@now` L510-530): a contact with no `label`/`labelseq`/`annotate`
attaches to the **single most-recent** creator. When a document declares every `\author`
before any `\address`/`\email` (a common amsart idiom, witness arXiv:2308.06214v1), the
most-recent author at every contact is the *last* one, so all addresses/emails bunch under
that last author. Perl 0.8.8 does the same (verified same-host, byte-identical).

**Rust behavior** (`base_utilities.rs::distribute_upfront_contacts`, a DOM pass in
`insert_frontmatter` between `coalesce_empty_creators` and `relocate_annotations`):
after creators are built, a *clean* pile is redistributed — the signature is that the
other N−1 authors carry no `ltx:contact` and the last author's `K` contacts split evenly
(`K = N·m`) into a role-periodic sequence (`role[i] == role[i+m]`); group *i* is then
handed to author *i* (`address i` → author i, `email i` → author i, …). All-same-role
piles (`m = 1`) work too.

**Why**: the input is genuinely ambiguous (amsart's PDF renders one flat block, no
pairing), so we distribute ONLY the unambiguous regular grid and otherwise keep Perl's
attachment verbatim. This mirrors the shared-email splitter's "distribute-when-clean,
else keep prior" rule (#52(j)). The gate's `N-1`-empty + periodicity conditions make the
interleaved idiom (each author immediately followed by its own contacts — already correct)
fail the check, so it is never disturbed (guard `tests/structure/amsarticle.tex`, whose
Joe-Blow/Frank-Zappa/Someone-Else block is interleaved and byte-unchanged).

**Upstream**: brucemiller/LaTeXML — the same redistribution would help Perl; not filed here.

**Guard**: `06_cluster_frontmatter::frontmatter_amsart_upfront_contact_distribution`
(up-front pile distributes; interleaved control untouched). KNOWN_PERL_ERRORS #104.

### 141. calc column widths (`p{(\columnwidth - N\tabcolsep) * \real{X}}`) evaluate, not collapse to 0pt

**Perl** LaTeXML's `calc.sty.ltxml` patches only the explicit length/counter setters
(`\setlength`/`\addtolength`/`\setcounter`/`\settowidth`/…) to run its expression scanner. A
`p{Dimension}`/`m`/`b`/`w`/`W` column width, however, is read by the *base* dimension reader,
which never routes through that scanner. When the width is a calc infix expression — the default
Pandoc emits for a relative-width table column, `>{\raggedright\arraybackslash}p{(\columnwidth -
N\tabcolsep) * \real{X}}` — the base reader meets `(`, cannot parse it, and warns `Missing number
(Dimension), treated as zero`. Every such column comes out `width="0.0pt"`; a zero-width `p{}`
cell wraps its text one character per line — the reporter's "river of characters with no
resemblance to the original". Real pdflatex+calc evaluates the width (calc patches length
scanning wherever a `<dimen>` is read). latexml-oxide reproduced Perl exactly (SHARED-FAILURE,
both diverge from real LaTeX; verified same-host — identical 0pt + identical warning).

**Perl behavior**: a Pandoc calc column width → `width="0.0pt"`; the whole table collapses.
**Rust behavior**: the base dimension reader's calc seam (`gullet::set_internal_dimension_fn`,
the same one #115 installs for `\widthof`) also fires on a leading `(`: it un-reads the paren and
runs calc's own expression parser (`read_expression_body`) on the live stream, returning the
evaluated dimension. The parser stops at the first non-`(+|-|*|/)` token, so it never
over-consumes; it is scoped to a leading `(` (the exact Pandoc idiom), so nothing that parsed
before changes. When `calc` is not loaded the seam is unset → byte-identical to before. So
`p{(\columnwidth - 4\tabcolsep) * \real{0.30}}` with `\columnwidth`=345pt now yields
`width="96.3pt"` instead of `0.0pt`.

**Why**: a kernel-quality gap, not a TeX-semantics change — real LaTeX+calc already evaluates a
length here; both LaTeXML engines simply failed to. The fix halos across every calc-relative
column width, i.e. essentially every Pandoc-authored table on arXiv.

**Witnesses**: arXiv 2606.08266v1 (html_feedback#6909) — an IEEEtran survey whose five-column
`p{(\columnwidth - 8\tabcolsep) * \real{…}}` failure-statistics table collapsed to zero-width
columns. Rust `(\columnwidth - 4\tabcolsep) * \real{0.30}` now measures 96.3pt (of 321pt avail).

**Upstream**: worth filing against `brucemiller/LaTeXML` (same gap upstream).

**Guards**: `06_cluster_regressions::cluster_pandoc_calc_colwidth_6909` — a two-column
`p{…*\real{0.30}}` / `p{…*\real{0.70}}` table has no `width="0.0pt"` and carries the expected
proportional 96.3pt / 224.7pt.

### 142. `\\[dimen]` optional glue preserved as a themeable `--ltx-break-space` CSS variable

**Perl** LaTeXML's `\lx@newline OptionalMatch:* [Glue]` constructor parses the optional length of
`\\[20pt]` (the extra vertical space LaTeX inserts at a forced line break) and then **drops it**:
it emits a bare `<ltx:break/>`, and `ltx:break` has no spacing attribute in the schema (`break_model
= empty`). So the author's requested gap is lost in HTML while the PDF keeps it. latexml-oxide
reproduced this exactly (SHARED-FAILURE; verified same-host — byte-identical core XML, both a bare
`<break/>`).

**Perl behavior**: `\\[20pt]` → `<break/>` (the 20pt is discarded).
**Rust behavior**: the constructor reads the optional `[Glue]` (`args[1]`) and, when non-zero, sets
`cssstyle="--ltx-break-space:<pt>"` on the break, so `\\[20pt]` → `<break
cssstyle="--ltx-break-space:20.0pt"/>` → `<br class="ltx_break" style="--ltx-break-space:20.0pt;">`.
Plain `\\` (no optional) and `\\[0pt]` stay bare. **No default CSS rule consumes the variable**, so
default rendering is byte-identical to Perl's bare break — the value is *preserved for a theme to opt
into* (e.g. ar5iv mapping it to a margin), not acted on.

**Why**: the same "preserve intent in the data model, let the theme decide" principle as the #721
image-sizing discussion — the engine stays faithful (emits the value, changes nothing visually) and
spacing policy lives in the theme layer. A default-inert attribute is a strict superset of the
parity output.

**Witnesses**: html_feedback #722 (a `\title{… \\[20pt] {\small …}}` whose 20pt gap vanished in HTML).

**Upstream**: worth raising against `brucemiller/LaTeXML` (the drop is upstream; a `--ltx-break-space`
convention or a break spacing attribute would let both engines carry it).

**Guards**: `06_cluster_regressions::cluster_break_optional_glue_722` — `\\[20pt]` carries
`--ltx-break-space:20.0pt` on its break and exactly one break in the document does (plain `\\` and
`\\[0pt]` stay bare).

### 143. The FIRST paragraph is `ltx_noindent` when `\parindent` is zero

**Perl** LaTeXML's `\par` (`TeX_Paragraph.pool.ltxml` L131-137) marks a paragraph `ltx_noindent`
via a *deferred* flag: the `\par` that closes paragraph N reads `next_para_class` (set by the
`\par` that closed N-1) and records a fresh flag for N+1, keyed on `\parindent==0` at close time.
The mechanism models the fact that a paragraph's indent is fixed by `\parindent` when it *begins*,
approximated by `\parindent` at the *previous* `\par`. But the very first body paragraph has no
prior `\par`, so it is never marked — even under `\setlength{\parindent}{0pt}`. It then inherits
the stylesheet's default first-line indent (`ltx-article.css` `.ltx_para > .ltx_p:first-child {
text-indent:2em }`), rendering visibly indented where pdflatex is flush-left. latexml-oxide
reproduced Perl exactly (SHARED-FAILURE; verified same-host — byte-identical XML, first `<para>`
un-classed in both).

**Perl behavior**: with `\parindent=0`, the first paragraph is un-classed → indented 2em by CSS;
the 2nd+ paragraphs are `ltx_noindent`.
**Rust behavior**: `\par`'s `after_digest` (`tex_paragraph.rs`) flags the closing paragraph a
first-paragraph candidate when `\parindent==0` and no deferred class applies; the constructor then
stamps `ltx_noindent` iff the paragraph is *structurally* first — no preceding `ltx:para` sibling
(`document::helpers::preceding_para_sibling`, shared with `prune_empty_para`). The structural test
matters: a state one-shot ("first `\par` seen") is consumed by a begin-document `\par` before the
first content paragraph under the no-dump / freshly-generated-dump sequence (the CI path), so the
first landing reverted there; the DOM position is robust to how many stray `\par`s fired. The stamp
fires only when `\parindent` is genuinely zero — where `ltx_noindent` is correct — so
default-`\parindent` documents are byte-identical to before, and only the first paragraph is
touched (later paragraphs keep the unchanged deferred mechanism).

**Why**: a kernel-quality off-by-one, not a TeX-semantics change — real LaTeX+`\parindent=0`
leaves the first line flush too. The fix halos across every `\parindent=0` document (manual
`\setlength`, `parskip`, KOMA `parskip=`, …). Completes #106's `parskip` surpass, whose first
paragraph was the residual gap.

**Witnesses**: issue #719 (reporter nasser1), the first-paragraph tail of #558/#106 (same
reporter). MWE: `\setlength{\parindent}{0pt}` + two paragraphs → both `ltx_noindent`.

**Upstream**: worth filing against `brucemiller/LaTeXML` (same off-by-one upstream).

**Guards**: `06_cluster_regressions::cluster_first_para_noindent_719` (first paragraph
`ltx_noindent` under `\parindent=0`; a control fixture confirms default `\parindent` marks no
paragraph); `cluster_package_guards::dump_gate_init::nodump_latex_branch_converts_healthily` (the
same via a `LATEXML_NODUMP=1` subprocess, sharing one raw kernel load with the other NODUMP guards —
guards the exact stray-`\par` path that broke the state-flag first landing);
`50_structure::parskip_test` (all three paragraphs `ltx_noindent`).

### 144. Verbatim contexts keep `~`/`^` ASCII under T1 (the fontmap accent stays for normal text)

**Background.** LaTeXML's `t1.fontmap` (and `t2a`/`t2b`/`t2c`) *deliberately* map slot 126 (`~`)
to `U+02DC` SMALL TILDE and slot 94 (`^`) to `U+02C6` MODIFIER LETTER CIRCUMFLEX — Bruce Miller,
commit `9ec6a4122` "Encodings (#2435)", 2024-11-20: "^ and ~ which should be accents". We KEEP
that mapping. Those slots are reached only by a *literal catcode-12* `~`/`^`: in normal text `~`
is active and `^` is superscript, and `\textasciitilde`/`\textasciicircum` emit ASCII U+007E/U+005E
directly (not via the slot). The one place the slot is hit is a **verbatim** context — `\verb`,
the `verbatim` environment, a `Verbatim`/`HyperVerbatim` argument (incl. `\href` and a Rhai
binding's verbatim arg), and `\url`/`\path` — where the intent is the *literal* character, so a
`.../~user` URL must stay ASCII (the reporter's Rhai `HyperVerbatim` URL came out `˜`).

**Perl behavior**: a verbatim/URL `~`/`^` under T1 font-decodes through the fontmap to the accent
glyphs `U+02DC`/`U+02C6` in the **displayed** text (SHARED-FAILURE — Perl does the same for `\verb`,
`\url`, and a `HyperVerbatim`/`Verbatim` constructor argument alike). What always stayed ASCII was
the href **attribute**: it is built by *reversion* of the catcode-12 tokens, which never touches
the fontmap — so #723's "`\href`/Semiverbatim is fine" was about the attribute, NOT the display.
(Verified same-host: Perl `\url{~a^b}` → `<ref … href="~a^b">˜aˆb</ref>`; a `DefConstructor('\x
HyperVerbatim {}')` → the same `˜aˆb` display.)
**Rust behavior**: every verbatim context now selects the identity `"ASCII"` fontmap for its run,
so `~`/`^` (and `` ` ``/`'`) stay ASCII in the display too, while the fontmap itself is untouched —
normal T1 text still follows Bruce. Four sites, all leaving the `typewriter` family intact (styling
unchanged): `Verbatim`/`HyperVerbatim` add `MergeFont(encoding => "ASCII")` in `before_digest`
(`base_parameter_types.rs`); `\verbatim@font` gains `\lx@fontencoding{ASCII}` (`latex_constructs/sect13.rs`,
covers the `verbatim` environment); `\@internal@{text,math}@verb`'s `font` clause gains `encoding =>
"ASCII"` (covers `\verb`); and `\UrlFont` (all `\urlstyle` variants) gains `\lx@fontencoding{ASCII}`
(`url_sty.rs`, covers `\url`/`\path` — whose displayed text is a separately-digested `\UrlFont`-
wrapped plain arg, so the reader's semiverbatim ASCII fontmap did not reach it). `\lx@fontencoding`
merges only the encoding and `\selectfont` merges only family/series/shape, so the ASCII encoding
survives the family switch. (The OmniBus and apacite `\doi` set their verbatim argument in the ASCII
encoding the same way: `omnibus_cls.rs`, `apacite_sty.rs`.) The sites call the digest primitive `\lx@fontencoding` that `\fontencoding`
stands for (Perl TeX_Fonts.pool.ltxml:169), never the public name, which a package may re-install as
the kernel's checking command (latex.ltx:10490-10492; luatexja's lltjfont.sty:549-551/573-574), and
which knows no `\T@ASCII`: qworld's 7 "Encoding scheme `ASCII' unknown" (59j; guard
`perfect_kernel_batch59::url_font_survives_a_checking_fontencoding`).

**Why**: verbatim wants the literal input character; the T1 slot's accent shape is right only for
the accent-command contexts (a standalone `\^{}`/`\~{}`) that Bruce was protecting, which do not
take the verbatim path. Scoping ASCII to verbatim reconciles both, and matches what pdflatex
extracts (a verbatim `~`/`^` under T1 → ASCII U+007E/U+005E, verified via pdftotext and via the
pdftex golden `ec.enc ∘ glyphtounicode.tex`).

**Fontmap drift tooling**: `tools/fontmap_drift.py` recomputes that pdftex golden for each shipped
text encoding and fails on un-allowlisted drift. Slots 94/126 are allowlisted there **as Bruce's
intentional accent choice** (with the `#2435` reason), documenting exactly why our fontmap differs
from `glyphtounicode` — alongside slot 127 (line-break hyphen `U+2010`) and T2 slots 14/15
(Cyrillic angle quotes `‹›`).

**Witnesses**: issue #723 (reporter xworld21) — a Rhai `HyperVerbatim` URL argument under T1 whose
`~` became `U+02DC`; the `\url`/`\path` follow-up came from Vincenzo's observation that "hyperref
works fine" (the attribute) while a constructor's verbatim arg did not (the display). MWE:
`\usepackage[T1]{fontenc}` + `\verb|a~b^c|` → `a~b^c`; `\url{http://g/~h^i}` → display `~h^i`.

**Upstream**: worth filing against `brucemiller/LaTeXML` (verbatim/`\url` display loses ASCII there
too, and Bruce can weigh the verbatim-vs-accent split).

**Guards**: `06_cluster_regressions::cluster_t1_hyperverbatim_ascii_723` (the reported Rhai
`HyperVerbatim` URL, ASCII, via a subprocess so the runtime binding loads);
`cluster_t1_verbatim_ascii_723` (`\verb` + `verbatim` env + `\url`/`\path` stay ASCII AND keep
`font="typewriter"`); `tools/fontmap_drift.py` (the fontmap values, with 94/126 allowlisted as
Bruce's accents).

### 145. nicematrix `\begin{<x>NiceMatrix}` renders as a real math array with `\CodeBefore` cell colors

**Background.** `nicematrix` has **no** Perl `LaTeXML/…/nicematrix.sty.ltxml` binding — it is a
Rust-contrib package (`latexml_contrib/src/nicematrix_sty.rs`). Its math matrix family
(`NiceMatrix`, `pNiceMatrix`, `bNiceMatrix`, `BNiceMatrix`, `vNiceMatrix`, `VNiceMatrix`) was
previously stubbed to an `<ltx:note role="nicematrix-placeholder">` plus `Error:undefined`, and the
matrix body was **discarded** — the entries (and any `\CodeBefore` cell coloring) vanished. Witness
arXiv 2410.00317 (a rigidity-matrix paper: 5× `bNiceMatrix[first-row,first-col]`, each with a
`\CodeBefore … \Body` block and `\rectanglecolor{blue!15}` marking the nonzero entries).

**Rust behavior (beyond-Perl).** Each `\begin{<x>NiceMatrix}[opts]` reduces to the amsmath matrix
flavour for its delimiter (`b→[]`, `p→()`, `B→{}`, `v→||`, `V→‖‖`, plain→none) via the shared
`\lx@ams@matrix` path (`amsmath_sty.rs`, `base_xmath.rs:1151`), so the entries render through the
real math-array engine as `ltx:XMDual > XMWrap > XMArg > XMArray`. Three nicematrix features map
onto LaTeXML's native math-table model:

- **`\CodeBefore … \Body` cell coloring.** The color commands (`\rectanglecolor`, `\cellcolor`,
  `\rowcolor`, `\columncolor`, `\arraycolor`) run during CodeBefore digestion and record fill
  rectangles (1-based, over the main matrix); a no-output `\lx@nicematrix@applycolors` constructor
  — run right after the matrix closes, mirroring colortbl's `\lxsetcellcolor` DOM write — paints
  `backgroundcolor` (schema `LaTeXML-math.rnc:330`) onto the covered `ltx:XMCell`s. Colors reuse
  `color_sty::parse_color` (xcolor `!`-algebra), so `blue!15` resolves DRY. Because digestion and
  construction are **separate phases** (the recorders run at digest, the color-walk at construct),
  each matrix's `\lx@nicematrix@applycolors` snapshots ITS rects+flags in its digest-time
  `properties` closure (before the next `\begin` clears the thread_local) and reads them back at
  construction. The MathML post-processor (`pmml_array`) then carries `XMCell/@backgroundcolor`
  onto the `m:mtd` as `mathbackground`, which the XSLT turns into the `--ltx-bg-color` theming
  variable the CSS paints — so the fill is visible in the HTML.
- **`[first-row,first-col]`** label lines are kept INSIDE the array (row 0 / col 0) and marked
  `thead='column'`/`'row'` (`LaTeXML-math.rnc:352`), rather than nicematrix's outside-the-brackets
  placement. (Accepted nuance; the semantic header role is preserved.)
- The tabular-like family (`\NiceArray`/`pNiceArray`/…/`NiceTabular*`/`NiceTabularX`, which take a
  `{colspec}`) still degrades to a placeholder/`\tabular` — no faithful colspec reduction yet.

**Bundled fix (not a divergence).** `color_sty::try_color_algebra` mixed `c!p` at `(100-p)%` of the
color instead of `p%` (`blue!15` came out dark `#2626FF` instead of light `#D9D9FF`), because it
passed `1.0 - pct_frac` to `Color::mix` whose `fraction` is the weight of *self*. Corrected to
`pct_frac`. This only surfaced through **direct** Rust callers of the algebra path
(nicematrix here, plus `soul`/`fancyvrb`/`colordvi`); `\color`/`\textcolor` were already correct
because xcolor's own decoder (`xcolor_sty::apply_mix_expr`) does the mix right.

**Limitation.** Only the *color* commands in `\CodeBefore` are interpreted; other decorations
(`\tikz`, `\SubMatrix`, …) are undefined and may emit `Error:undefined` (the matrix still renders).
Several Nice matrices in ONE display are handled — the color-walk paints the *last* matrix-XMDual,
i.e. the just-closed one (guard `cluster_nicematrix_multi_matrix_no_color_leak_6569`). A Nice matrix
NESTED in another's cell is not: the inner `\begin` clears the shared thread_local, so the outer
loses its `\CodeBefore` rects and both share one first-row/first-col flag pair (inner wins).

**Upstream / mirror**: the ar5iv `nicematrix.sty.ltxml` stub still errors here; mirror this upgrade
there for strict Rust↔ar5iv parity.

**Witnesses**: arXiv/html_feedback#6569 (witness arXiv 2410.00317).

**Guards**: `06_cluster_regressions::cluster_nicematrix_codebefore_6569` (the witness's first matrix:
`ltx:XMArray` present, exactly 6 `backgroundcolor` cells at the nonzero entries, `thead` on the
first-row/first-col labels, 0 errors, no `nicematrix-placeholder`).

### 146. A hand-typeset title that duplicates the structured `\title{}` is dropped

**Background.** LaTeXML's unified Frontmatter API captures `\title{}` into a semantic
`<ltx:title>` the moment it is seen — independent of `\maketitle` (unlike LaTeX, where `\title{}`
without `\maketitle` renders nothing), and the stylesheet renders that title as the visible
document heading. When a paper *also* hand-typesets its title as ordinary body text — a leading
centered display-font block, with `\title{}` set but `\maketitle` never called — the title appears
twice: once from the structured frontmatter, once as the author's "ink". Witness arXiv 2608.10928
(`\title{…}` + a `\begin{center}{\LARGE\bfseries …}` reproduction; no `\maketitle`).

**Perl behavior**: SHARED — Perl LaTeXML emits the structured `<ltx:title>` from `\title{}` without
`\maketitle` too, so it duplicates identically (verified on the witness). This is not a Rust-only
bug; the divergence below is a deliberate surpass-Perl.

**Rust behavior**: at frontmatter finalization (`\lx@frontmatter@fallback`, the no-`\maketitle`
path), once the structured `<ltx:title>` is in the tree, `maybe_dedup_leading_title_ink`
(`base_utilities.rs`) removes a **leading, non-sectional, centered, display-font** paragraph whose
normalized text **exactly** reproduces the structured title. Structure wins over ink: the semantic
`<ltx:title>` is kept; the redundant hand-typeset copy is dropped (empty wrappers pruned). It is the
mirror of `maybe_promote_leading_title` (which, when *no* structured title exists, promotes such a
block *into* the title) and reuses the same detection helpers.

**Why it's safe / precise.** Fires only for `\title`-without-`\maketitle` papers (the `\maketitle`
path disables the fallback), and only on a full normalized-text match against the structured title
in a leading display-font centered block. A paper that sets `\title` but doesn't hand-typeset keeps
its title; a leading centered block that *doesn't* match the title is left untouched; author/abstract
"ink" (no structured counterpart) is preserved. Never removes a title the PDF actually shows —
`\title{}` without `\maketitle` renders nothing in the PDF, so only the manual block was ever
visible there.

**Witnesses**: arXiv/html_feedback#6924 (witness arXiv 2608.10928).

**Guards**: `06_cluster_regressions::cluster_frontmatter_title_ink_dedup_6924` (structured `<title>`
kept, title text appears exactly once, hand-typeset author block preserved).

### 147. A leftover control sequence in a hyperref URL stays literal, not digested

**Background.** LaTeXML reads a hyperref `\url`/`\href` argument *semiverbatim*: it neutralizes the
specials to catcode-12 but keeps the backslash an escape (`hyperref.sty.ltxml:165-186`,
"Expand as we go!" / "let CS's through!"), so the argument is read with partial expansion and any
surviving control sequence is then handed to digestion. Real `url.sty` instead `\meaning`-stringifies
the whole argument (`\edef\Url@String{\expandafter\strip@prefix\meaning\Url@String}`), which turns
every leftover control sequence into inert characters. The difference bites on a **non-expandable
primitive** in a URL, e.g. `\url{https://ex/q=\def}`: digesting `\def` *executes* it — it reads the
following tokens as its name/parameter-text/body, consumes past the closing brace, truncates the URL
to `…q=` and raises errors. Escapes (`\%`, `\_`, `\^`, `\textasciitilde`, …) are unaffected because
they resolve to their character before/at digestion; only a genuine leftover CS misbehaves.

**Perl behavior**: SHARED-FAILURE — Perl LaTeXML digests the `\def` the same way and is in fact
*worse*. Verified same-host (Perl 0.8.8, rev `0d02309d`): `\url{https://www.google.com/search?q=\def}`
→ **byte-identical** `href="https://www.google.com/search?q="` in both engines, Perl raising **4**
errors to our 2. Not a Rust-only bug; the change below is a deliberate surpass. pdflatex keeps the
`\def` as literal link text (via `url.sty`'s `\meaning`), which is what an author expects.

**Rust behavior (beyond-Perl)**: after the semiverbatim read, any *surviving* control sequence is
recatcoded to `other` (`Token::as_other`) — url.sty's `\meaning` in one step — so `\def` becomes the
literal href text `\def` instead of executing. To keep the escapes resolving so realistic URLs are
unchanged, the reader now expands url.sty's escape set (`\%`, `\#`, `\&`, `\_`, `\~`,
`\textasciitilde`, `\^`, `\textasciicircum`, `\textbackslash`, `\\`) to their character *during* the
partial read (mirroring `url.sty`'s first pass), leaving only genuine leftovers as CS to stringify.
Two sites, kept consistent so `\url` and `\href` agree: `\lx@hyper@url` (`hyperref_sty.rs`, the
hyperref `\url`) and the `HyperVerbatim` parameter type (`base_parameter_types.rs`, backing `\href`).
Plain `url.sty` without hyperref already did this (it recatcodes the whole argument to `other`), so
this brings the hyperref path into line with it — and with pdflatex.

**Why it's safe / precise.** Only a leftover *control sequence* changes — every url.sty escape still
resolves (`\_`→`_`, `\^`→`^`, `\textbackslash`→`\`, …), literal specials pass through, and an
expandable `\macro` still expands during the read. The net effect on any URL that previously
converted cleanly is nil; the only behavior that changes is the one that previously *errored* and
lost data. Consistent with #144 (which fixed the T1 ASCII *display* of verbatim/URL text) — that
was the font side, this is the token side.

**Scope / limitation.** `\path` already stringified leftovers (plain `url.sty`'s `\lx@url@url`
recatcodes its whole argument to `other`), so it was never affected. `\nolinkurl` still digests a
leftover primitive (SHARED-FAILURE with Perl — both truncate `\nolinkurl{…n=\def}` to `n=`): it reads
through the generic `Semiverbatim` *parameter type*, so bringing it in line would mean changing that
shared reader — out of this ticket's scope, left as parity.

**Witnesses**: issue #723 rebuttal (reporter xworld21 / Vincenzo Mantova), comment 5380586287:
`\url{https://www.google.com/search?q=\def}` lost the `\def` and errored while pdflatex kept it.

**Upstream**: worth filing against `brucemiller/LaTeXML` — its `\url`/`\href` digest a leftover
primitive the same way; adopting `url.sty`'s `\meaning`-stringify would fix it there too.

**Guards**: `06_cluster_regressions::cluster_url_cs_verbatim_723` (distilled reproductions covering
Vincenzo's cases: `\def` stays literal in both `\url` and `\href`, escapes/`~`/`\textbackslash`
still resolve, 0 errors); `10_expansion::hyperurls_test` (the full escape matrix — `\#`, `\&`, `\_`,
`\%`, `\^`, `\~{}`, macro expansion, literal `^`/`$`/`{}` — all still resolve).

### 148. `\everypar` fires at paragraph start (tex.web `new_graf`), enabling correct algorithm2e line numbering

**Background.** algorithm2e's `linesnumbered` numbers each body line by setting
`\everypar`→`\nl` (which steps `AlgoLine` and typesets `\algocf@printnl`). The number
must be emitted when a line's paragraph starts — after any leading `everyparnl`-setter
(a KwInOut header sets it to `\relax` BEFORE its content, so Input/Output stay
unnumbered) but before a trailing one (a `\Comment*[r]` side comment resets it to
`\relax` AFTER the statement). Only the content-start moment distinguishes them.

**Perl behavior**: SHARED failure. Perl LaTeXML never fires `\everypar` on
horizontal-mode entry (`enterHorizontal`, Stomach.pm, is a plain mode switch); the
algorithm2e binding runs `\the\everypar` MANUALLY at **end-of-line**
(`algorithm2e.sty.ltxml` L171). By then a `\Comment*[r]` statement's `everyparnl` is
already `\relax`, so the statement **loses its line number** and the comment falls to
the next line. Verified on the witness (2602.20153) with same-host Perl.

**Rust behavior**: `stomach::enter_horizontal` fires `\the\everypar` on the
vertical→horizontal transition, faithful to tex.web `new_graf` (background/tex.web
L21117, `begin_token_list(every_par)`). It is guarded two ways — a no-op when
`\everypar` is empty (every ordinary paragraph, post-`\begin{document}`), and skipped
in the preamble/kernel-load where `\everypar` holds LaTeX3's unmodelled para-hook list
`\g__para_standard_everypar_tl` (guard: `\@nodocument` is `\relax`). A prerequisite fix:
`\begin{document}` now clears `\everypar` via `assign_register` (Perl `AssignRegister`),
not `assign_value`, so the register `\the\everypar` reads is actually emptied. The
algorithm2e binding then makes each listing line a real hmode entry (a per-line
`leave_horizontal_internal` seam) and moves line indentation to an end-of-line DOM
prepend so it does not enter hmode early. Result: statement lines carrying a trailing
`\Comment*[r]` KEEP their number; KwInOut headers and standalone comments stay
unnumbered — matching the pdflatex golden and surpassing Perl.

**Refinement (2026-09-03, batch 54l): only a *list* fires it.** tex.web §1091
`new_graf` runs from `main_control` in vertical mode of the current list; a
constructor's digested `{}` argument (`stomach::digest(tokens)`) is
macro-parameter text TeX never typesets as such. `ARG_DIGEST_DEPTH` counts open
isolated argument digestions and `fire_everypar` stays silent while it is
non-zero; `digest_next_body` (an environment body, `\hbox`/`\vbox` contents,
`DigestUntil`) resets it to 0 because the captured material IS a list. Before
this, an armed `\everypar` — latex.ltx `\@afterheading`'s
`{\setbox\z@\lastbox}`, left by ltugboat.cls:1214 `\aftergroup\@afterheading`
in `\@maketitle` — fired inside `\@@numbered@section`'s *type* argument and
reverted into it as `{}section` (counter `\c@{}section`, tag `ltx:{}section`;
lazylist, parnotes). Guard:
`perfect_kernel_batch54::everypar_does_not_fire_inside_a_constructor_argument`.

**Why it's safe.** `\everypar` firing is body-only and a no-op for normal paragraphs
(LaTeXML's list/item machinery does not populate `\everypar`, unlike real LaTeX);
inside a listing algorithm2e overrides `\everypar` with its own `\algocf@everypar`.
Full suite 2143/2143, tikz/streaming re-verified clean.

**Witnesses**: arXiv 2602.20153 (JUCAL, `\Comment*[r]`); the disjoint-decomposition and
generic-`\Fn` examples from the algorithm2e manual.

**Guards**: `50_structure::algorithm2e_linenumbers_test` (KwInOut unnumbered; a
`\Comment*[r]` statement numbered; body 1..N; nested indentation).

**Upstream**: to be filed at brucemiller/LaTeXML (endline-timed `\the\everypar` drops
the `\Comment*[r]` statement number).

### 149. Float body frames (`ruled`/`boxed`) land on the body, not the metadata `<tags>` — algorithm2e ruled family wired

**Background.** `addFloatFrames` (`float.sty.ltxml` L76-85) draws a float's frame from
two maps: `%float_outerframe` puts an outer rule on the `<float>` itself, `%float_innerframe`
puts an inner rule on the float's **body** — the first child that is not a caption
(`grep { getNodeQName !~ /^ltx:(?:toc)?caption$/ } childNodes`). `ruled` → outer `top` +
inner `topbottom`; `boxed` → inner `rectangle`. pdflatex draws both rules.

**Perl behavior**: SHARED failure. A `\refstepcounter`'d float emits `<ltx:tags>` as its
**first** child, and `<tags>` (`LaTeXML-block.rnc:325`, `element tags { tag+ }`) carries
**no attributes** — so `setAttribute($body, framed => …)` is silently schema-dropped and the
**inner rule is never drawn**. The outer `framed="top"` (set on the float) survives, so a
ruled float shows only its top rule; a `boxed` float shows **no frame at all** (boxed has no
outer rule). Verified same-host on Perl 0.8.8: `floatnames.tex` (newfloat `\floatstyle{ruled}`)
and a `[boxed]` algorithm2e MWE both emit only the outer `framed`, never the inner. Separately,
algorithm2e's own binding (`algorithm2e.sty.ltxml` L88-91) wires **only** the `box` family to a
frame; the `ruled` family (`ruled`/`algoruled`/`tworuled`/`plainruled`) is dropped by both
engines, so a default `\usepackage[ruled]{algorithm2e}` gets no rules.

**Rust behavior**: `add_float_frames` also skips `<ltx:tags>` when choosing the body, so the
inner `framed` lands on the real body element (`<listing>`, `<p>`, …) that pdflatex frames.
And the algorithm2e binding extends its `\algocf@style` dispatch: `box`→`boxed` (unchanged),
else `ruled`→`ruled`, so `[ruled]`/`[algoruled]`/… draw the top+body rules. Reach is
engine-level — every framed float (algorithm/algorithmicx, `newfloat`, `float.sty`,
algorithm2e boxed/ruled) now frames its body.

**Why it's safe.** `framed` is a generic `Backgroundable.attributes` decoration; the fix only
moves the *target* of an already-intended `setAttribute` from a metadata element that rejects
it to the body element that accepts it — no new markup shape, no change to the listings dialect
(an `lstlisting` serving as a ruled-float body is decorated exactly as any other body would be).
The outer-frame path is unchanged. Full suite green.

**Witnesses**: `floatnames.tex` (newfloat ruled), `algx.tex` (algorithmicx ruled),
`figure_mixed_content.tex` (algorithm floats), `various_colors.tex` (lstlisting ruled-float body);
`[boxed]`/`[ruled]` algorithm2e MWEs cross-checked against pdflatex goldens.

**Guards**: `50_structure::algorithm2e_frames_test` (ruled → `top`+`topbottom`, boxed →
`rectangle`, via `\RestyleAlgo`); the four re-blessed goldens above pin the general fix.

**Upstream**: to be filed at brucemiller/LaTeXML (inner float frame dropped onto `<tags>`; ruled
family unwired in algorithm2e).

### 150. `\floatname`/`\newfloat` also define float.sty's real `\fname@<type>` internal

**Background.** Real `float.sty` names a float's caption word `\fname@<type>`
(`float.sty` L34: `\newcommand\floatname[2]{\@namedef{fname@#1}{#2}}`; `\newfloat`
defaults it, L59). Documents reference that real internal directly — most visibly the
widely-copied `breakablealgorithm` recipe: `\textbf{\fname@algorithm~\thealgorithm}`.

**Perl behavior**: SHARED failure. LaTeXML *reimplements* float.sty with its own internal
`\lx@name@<type>` (`float.sty.ltxml` L36) and never defines `\fname@<type>`. So any
document touching the real internal leaks a raw, undefined `\fname@<type>` —
`<ltx:ERROR>\fname@algorithm</ltx:ERROR>` — and errors. Verified same-host on Perl 0.8.8
(witness arXiv 2408.07803).

**Rust behavior**: `\floatname` and `\newfloat` define **both** LaTeXML's `\lx@name@<type>`
(unchanged, drives our tag machinery) **and** real float.sty's `\fname@<type>`
(`float_sty.rs`). The `breakablealgorithm` caption then compiles to "Algorithm 1 …"
instead of leaking raw. Additive — no currently-passing document emits `\fname@<type>`,
so no existing output shape changes; it only converts the error to correct output.

**Why it's safe.** `\fname@<type>` is the *real* float.sty internal, so defining it makes
our float.sty emulation more faithful to the actual package, not less. Purely additive to
the `\floatname`/`\newfloat` bindings.

**Witnesses**: arXiv 2408.07803 (html_feedback #1998, `breakablealgorithm` recipe).

**Guards**: `50_structure::float_fname_internal_test` (`\floatname` sets `\fname@widget`;
`\newfloat` defaults `\fname@gizmo`).

**Upstream**: to be filed at brucemiller/LaTeXML (float.sty.ltxml should alias
`\fname@<type>` to its `\lx@name@<type>`).

### 151. `\tabto` (tabto-ltx) approximated as `\hfill` — right-justified algorithm comments flush inline

**Background.** `tabto` (package `tabto-ltx`) moves to a horizontal tab position.
`algpseudocodex` `\RequirePackage{tabto}` (sty L29) and right-justifies each `\Comment`
with `\tabto{\dimexpr\linewidth-\algpx@tmpLen}`, so a `\State … \Comment{…}` line shows
its comment flushed to the right margin (as pdflatex does).

**Perl behavior**: SHARED failure. LaTeXML has no `tabto` binding, so both engines
raw-load it. Raw `\tabto` measures the current line position with a `$$…$$` display-math
+ one-row `\halign` hack (reads `\predisplaysize`, tabto.sty L85-120). Our engine — with
no positional layout model — turns that hack into (a) a spurious empty display
`<equation/>` (see KNOWN_PERL_ERRORS #108) and (b) a paragraph break, so the comment
**stacks on its own line below the statement** instead of flushing right. Same-host Perl
raw-loads the identical hack.

**Rust behavior**: a `tabto.sty.ltxml`-equivalent binding (`tabto_sty.rs`) approximates
`\tabto{pos}` (and `\tabto*`, `\tab`) as `\hfill`. LaTeXML renders `\hfill` before inline
content as a `float:right`, so the comment stays in the statement's line box and flushes
right — matching the pdflatex golden. The dominant `\tabto` use IS this right-justify, so
the approximation is faithful in practice; a genuinely left-directed `\tabto{2cm}` would
also become a right-fill, but the raw `$$`-hack (break + empty equation) was strictly
worse. The length registers `\CurrentLineWidth` / `\TabPrevPos` are provided so
algpseudocodex's `\settowidth`/`\dimexpr` reads resolve.

**Why it's safe.** Replaces an unmodellable positional hack with the layout primitive
LaTeXML already renders correctly; no positional information was being honoured before.

**Witnesses**: arXiv 2511.21969 (Algorithm 1 `\State … \Comment` lines).

**Upstream**: to be filed at brucemiller/LaTeXML (raw `\tabto`'s `$$` measurement hack
emits an empty equation and breaks the line; a `\hfill` approximation renders correctly).

### 152. `\hbox to \hsize{…leader fill…}` emits `width="100%"`, not a frozen pt value

**Background.** A leader-fill separator — `\hbox to \hsize{\dashfill\hfil}` (where
`\dashfill`=`\cleaders\hbox{-~-}\hfill`), or `\hrulefill`/`\dotfill` — sizes a box to
the current line/column width and fills it with a repeating rule. pdflatex confines it
to the column.

**Perl behavior**: SHARED failure. LaTeXML's `\hbox` constructor derives an ABSOLUTE
pt `width` from the `to` spec (`TeX_Box.pool.ltxml`, `width => $props{width}`). Since the
generic article `\textwidth` defaults to `345pt` and two-column class widths aren't
modeled, `\hsize`=`345pt`, so the box freezes at `width="345.0pt"` — wider than a
narrower container (e.g. an algorithm), where it OVERFLOWS. Same-host Perl emits the
identical frozen pt and renders equally too-wide.

**Rust behavior**: when a `\hbox to <line-register>` (`\hsize`/`\linewidth`/
`\columnwidth`/`\textwidth`) has a body that is a horizontal LEADER FILL (the `\leaders`
whatsit is marked `hfill_leader`; `tex_box.rs`), the constructor emits a RELATIVE
`width="100%"` so the box fills its HTML container — matching the pdflatex golden in any
context. The resolved value is compared against the CURRENT register value, so a
`\hbox to \hsize` inside a narrowed parbox relativizes to that parbox too. A genuine
fixed `\hbox to 100pt`, and any non-leader body (crucially fancyvrb's `\hbox to
\linewidth{…text…}` verbatim lines, whose `345pt` is deliberate Perl parity —
`wisdom_fancyvrb_linewidth_box_parity`), are UNCHANGED.

**Why it's safe.** The leader-fill discriminator scopes the change to boxes whose whole
purpose is to span the line; text-bearing full-width boxes keep their pt width.

**Follow-up (stacking).** width:100% alone is not enough when TWO full-line separators flank a
centered label on ONE `nowrap` listingline (1510.02728's "Modified ellipsoid method" block, inside
`\begin{algorithm}`): as inline-blocks they lay side-by-side and sum to >200% width, overflowing the
listing. The fill-line box is therefore ALSO marked `class="ltx_leaderfill"` (`tex_box.rs`, on the
same `fill_line` gate), and both stylesheets set `.ltx_inline-block.ltx_leaderfill { display:block; }`
so each separator owns its line and they stack like the pdflatex golden. fancyvrb/`\hbox to 100pt`
still untouched (not fill-line).

**Witnesses**: arXiv 1510.02728 (`\hbox to \hsize{\dashfill\hfil}` "Modified ellipsoid
method" separators, 3 per algorithm; two flank the centered label). Guard
`cluster_hbox_to_hsize_leader_fills_width` (asserts `width="100%"` + `class="ltx_leaderfill"`).

**Upstream**: to be filed at brucemiller/LaTeXML (a `\hbox to \hsize` leader fill should
be a fluid full-width box, not a frozen pt value that overflows narrower containers).

### 153. algorithm2e ruled family draws the caption at the TOP of the frame

**Background.** algorithm2e's `ruled`/`algoruled`/`tworuled`/`plainruled`/`boxruled`
styles put the caption at the top of the frame: the real sty sets
`\@algocf@capt@ruled`=`top` (L2530) / `\@algocf@capt@boxruled`=`above` (L2540), and
`\algocf@makethealgo` lays the caption out before the body. pdflatex renders it there.

**Perl behavior**: SHARED failure. LaTeXML emits the float caption in standard order
(last child = bottom), so the ruled caption renders at the BOTTOM. Same-host Perl does
the same.

**Rust behavior**: for the ruled family, `after_construct` DOM-moves `<ltx:caption>` /
`<ltx:toccaption>` before the body (`float_sty::reposition_caption_top`, gated by a
`caption_pos="top"` property set from the resolved `\algocf@style`). DOM order drives the
XSLT render position, so the caption renders at the top. `plain`/`boxed` keep the caption
at the bottom (no reposition). The float content model
(`LaTeXML-para.rnc:196`, an order-free choice) stays schema-valid.

**Why it's safe.** Pure post-construction reorder of two elements for one style family;
`plain`/`boxed` are untouched, and a guard asserts the plain case does not reorder.

**Witnesses**: any `\RestyleAlgo{ruled}` algorithm. Guard
`cluster_algorithm2e_ruled_caption_at_top` (+ re-blessed `algorithm2e_{frames,
linenumbers}.xml`).

**Upstream**: to be filed at brucemiller/LaTeXML (ruled-family algorithm captions should
render at the top of the frame, per algorithm2e's `\@algocf@capt@ruled`).

### 154. Replaceable frontmatter tags keep only one entry (forward-port of upstream dedup)

**Background.** Some frontmatter tags are "replaceable" — only one per document: a later
`title`/`toctitle`/`subtitle`/`date`/`abstract`/`keywords` should REPLACE the earlier,
not stack. Later upstream LaTeXML added `%ReplaceableFrontmatterTags` +
`\@add@frontmatter@now` (`Base_Utility.pool.ltxml`), which empties `$$frontmatter{$tag}`
before pushing a replaceable entry.

**Perl behavior**: the VENDORED Perl (our ground truth) PREDATES that fix — its
`\lx@add@frontmatter@{now,until}` push unconditionally, so a document that re-adds a
replaceable tag keeps BOTH entries and emits DUPLICATE frontmatter (two `<title>`, two
`<abstract>`). Newer upstream Perl does NOT.

**Rust behavior**: `base_utilities.rs` adds `REPLACEABLE_FRONTMATTER_TAGS` and clears
`frontmatter{tag}` before the push in `\lx@add@frontmatter@now` (and `@until`, guarded
against same-tag re-entrancy so a nested/malformed `{abstract}` is not corrupted).
Non-replaceable tags — crucially `ltx:creator` — still accumulate (multi-author
frontmatter is preserved). A forward-port of the upstream fix; a surpass over the
vendored Perl.

**Refined (batch 56io).** Only entries with the SAME `name` as the new one are replaced. An
entry under another name is another element and stays: a bilingual document's `Abstract` and
`摘要` abstracts are two abstracts (beamertheme-mirage-doc lost its English one, which vendored
Perl keeps). So are a journal's `Received` / `Revised` / `Accepted` dates, which the dedup had
collapsed to the last (14 of 3,003 arXiv 2605 papers). Re-emissions (a second `\icmltitle`, which
has no name) are still replaced. Guard `class_census::two_named_abstracts`.

**Why it's safe.** Restores the single-entry semantics upstream Perl already adopted;
creators/notes are excluded so multi-valued frontmatter is unaffected, and the `@until`
re-entrancy guard leaves the malformed-nesting case exactly as before.

**Witnesses**: arXiv 2002.09766 (appendix `\twocolumn[\icmltitle{…}]` re-added
`ltx:title` → duplicate title + duplicated author block), 2511.21969 (nested
`{abstract}` env). Guard `cluster_frontmatter_replaceable_dedup`.

**Upstream**: already fixed upstream (`%ReplaceableFrontmatterTags`); this forward-ports
it into the vendored engine.

### 155. A `.bbl` preamble no longer emits a phantom empty `(N)` bibliography entry

**Background.** An ACM-Reference-Format-style `.bbl` (and others) places a preamble —
`\providecommand`/`\newcommand` macro definitions and a blank line — between
`\begin{thebibliography}` and the first `\bibitem`. The blank line is a `\par`; inside a
bibliography that is `\par@in@bibliography`, which (when the next token is not
`\par`/`\bibitem`) opens a keyless `\lx@bibitem` for the preamble content.

**Perl behavior**: SHARED failure. The keyless phantom `\lx@bibitem` renders as a spurious
empty first entry — `<ltx:bibitem xml:id="bib.bib1">` with a `(1)` refnum tag and a
whitespace-only `<ltx:bibblock>` — pushing the real references to `bib.bib2…`. Both engines
carry a digest-time prune (Perl #2409 / `latex_constructs` `\lx@bibitem` afterDigest) meant
to catch exactly this, but it only inspects the IMMEDIATELY-previous box; the preamble
whitespace boxes displace the phantom from that check, so it survives. Same-host Perl emits
the identical phantom (verified byte-identical on arXiv 2605.03143).

**Rust behavior**: a `Tag!("ltx:bibitem", after_close_late)` scrub (`latex_constructs.rs`)
removes any bibitem that has no non-empty `key` attribute AND whose `<ltx:bibblock>`s are all
whitespace — i.e. the auto-opened phantom. A real `\bibitem` always carries a key, so real
entries are never touched. A surpass over the shared Perl failure.

**Why it's safe.** The discriminator (no `key` + whitespace-only bibblocks) matches only the
auto-opened phantom; a citeable reference always has a key and real bibblock text. The real
entries keep their `xml:id`s and keys (cross-references key on the key, not the id).

**Witnesses**: arXiv 2605.03143 (ACM-Reference-Format `.bbl`, empty `(1)` before 23 real
entries). Guard `cluster_bib_preamble_no_phantom_entry`.

**Upstream**: to be filed at brucemiller/LaTeXML (the `.bbl`-preamble phantom bibitem should
be pruned; the existing digest-time guard misses it when whitespace intervenes).

### 156. Author-attached `\thanks` is a marked note with semantic class hooks, not an inline contact

**Background.** In real arXiv author blocks, `\author{Name\thanks{…}}` carries a small set of
distinct content kinds — correspondence ("Correspondence to X ⟨email⟩"), funding ("supported by
NSF grant…"), equal-contribution ("contributed equally"), present-address ("now at…"),
prior-publication/venue, and generic acknowledgement. pdflatex renders `\thanks` as a footnote:
a superscript mark on the author name + the content at the page bottom.

**Perl behavior**: SHARED readability gap. Creator-scope `\thanks` becomes
`<ltx:contact role="thanks">` (`Base_Utility.pool.ltxml` `\lx@add@thanks` →
`\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=thanks]`), which the shared HTML XSLT
renders INLINE next to the author — structurally identical to an affiliation, with no mark. Same
in Rust before this change. (Title-scope `\thanks` already becomes a marked note/pubnote; only
creator-scope was the inline contact.)

**Rust behavior**: creator-scope `\thanks` routes to `<ltx:note role="thanks"
class="ltx_note_frontmatter ltx_thanks_<kind>">` attached to the creator (`base_utilities.rs`
`\lx@add@thanks` else-branch). It reuses the existing `ltx:note` footnote template (a superscript
`ltx_note_mark` on the author + `ltx_note_outer`/`ltx_note_content`), so a theme can place it as a
margin/footnote note. `<kind>` is a **best-effort** keyword classifier (`classify_thanks`):
`correspondence` / `funding` / `contribution` / `address` / `note`. The class hooks
(`ltx_note_frontmatter`, `ltx_role_thanks`, `ltx_thanks_<kind>`) let theme designers style each
kind. Requires: adding `ltx:note` to `ltx:creator`'s content model
(`LaTeXML.model` + `LaTeXML-structure.rng`/`.rnc`) — else `open_element` auto-closes the creator
and the note detaches; and an XSLT addition rendering the creator's `ltx:note` child as a
name-sibling (`LaTeXML-structure-xhtml.xsl`). A surpass over the shared inline-contact behavior.

**Why it's safe.** The classifier only picks a CSS hook, never core semantics. The note attaches
to the same creator the contact did (verified: note is inside `<creator>`); title-scope
`\thanks` (a pubnote) and affiliation contacts are untouched. Only golden change:
`tests/structure/authors.xml` (contact → note). The `frontmatter_ieee_membership_no_phantom`,
`frontmatter_thanks_literal_mark_mix`, and `author_block_thanks_collapses_in_title_not_inline`
tests are element-agnostic and unchanged.

**Coalesce edge case.** `coalesce_empty_creators` (which drops nameless comma-split creators and
moves their annotations to the last real author) special-cased `ltx:contact`; it was extended to
move `ltx:note` too, so a trailing `\thanks` on a nameless creator — 1510.02728's
`\author{Sani,~\IEEEmembership{…} Vosoughi,~\IEEEmembership{…}%\thanks{…NSF…}}`, where the
membership pieces digest to empty and the `\thanks` strands on a phantom creator — is not dropped
with that creator (it lands on Vosoughi, as the contact did). Regression guard
`cluster_author_thanks_note_survives_empty_creator`.

**Witnesses**: arXiv 2512.24601 (`\thanks{Correspondence to …}` → `ltx_thanks_correspondence`),
1510.02728 (`\thanks{…supported by NSF…}` → `ltx_thanks_funding`). Guards `authors_test` and
`cluster_author_thanks_marked_note`.

**Upstream**: to be filed at brucemiller/LaTeXML (author-attached `\thanks` should render as a
marked footnote, not an inline affiliation-like contact; the content-kind class hooks are a
theme-facing extension).
### 157. `minted` renders Pygments syntax colors from a committed `_minted/` frozencache

**Perl behavior**: Perl LaTeXML ships no `minted` binding — the `minted` environment
errors out (no highlighting at all). Our binding already routes `minted` through the
`listings` substrate (bold-black keywords, no color), which is itself beyond Perl.

**Rust behavior**: when a paper is built with `\usepackage[frozencache]{minted}`, the
committed `_minted/` directory (sibling of the main `.tex`) already holds Pygments'
output on disk as plain LaTeX. We read it and re-emit the **actual Pygments colors**
(green bold keywords, teal italic comments, blue names, gray operators, purple
decorators, …). Any colored output surpasses Perl's error-out. `\begin{minted}`,
`\inputminted`, and `\mintinline` all take this path; on a miss they keep the exact
uncolored `listings` rendering, and with no `_minted/` present the feature is a strict
no-op.

**The frozencache on disk.** `default.style.minted` defines a `\PYG@tok@<class>` per
Pygments token class, each `\let\PYG@bf=\textbf` / `\let\PYG@it=\textit` and/or
`\def\PYG@tc##1{\textcolor[rgb]{r,g,b}{##1}}`. Each `<MD5>.highlight.minted` is a
`MintedVerbatim` body of `\PYG{<tokclass>}{<text>}` runs interleaved with literal
spaces and `\PYGZ*` escapes (`\PYGZbs`→`\`, `\PYGZus`→`_`, `\PYGZgt`→`>`, …).

**Method — content-match, not MD5-keying.** minted keys its cache by an MD5 over the
snippet + options; replicating that keying is fragile. Instead, each highlight file —
`\PYG` unwrapped and `\PYGZ*` resolved — yields the exact plain code of some snippet.
We normalize a block's raw body the same way (rstrip each line, drop blank edges) and
look it up in a `plaincode → lines` map built once per document (memoized by the
resolved `_minted/` dir, so a later document never reuses a stale cache). Compound
classes (`n+nf`, `l+m+mf`) are resolved like `\PYG@toks`: color from the LAST
sub-class that sets one, bold/italic accumulate. Blocks that use `escapeinside`
(their raw body carries `@…@` markers the highlight file lacks) simply miss and fall
back — acceptable, since the current path already handles `escapeinside`.

**Emitter.** Colored lines reuse the listings constructors so the output is
structurally identical to the substrate (same `<ltx:listingline>`, same
`ltx_lst_space` `white-space:pre` runs for indentation, same `<ltx:listing>`
container with base64 `data` provenance): the block body is a sequence of
`\@lst@startline{}` … `\@lst@endline`, each segment's chars mapped through the
listings special-char table (`<`→`\textless`, `_`→`\textunderscore`, …) and wrapped
in `\textbf`/`\textit`/`\textcolor[rgb]{…}` per its style. Two small helpers were
added to `listings_sty` — `lst_process_display_with` / `lst_process_block_with` —
that accept pre-built body tokens instead of re-parsing the source; the re-parsed
entry points delegate to them, so the uncolored path is byte-identical.

**Why it's safe.** Reading the host source tree's `_minted/` is in scope (like reading
a `.sty`; the ban is only on latexml-oxide's *own* embedded resources). The feature
activates only when `find_file("_minted/default.style.minted")` resolves, so
non-frozencache papers are untouched; a cache miss keeps the current listings output.

**Witness**: arXiv:2605.03143 (`\begin{minted}{ocaml|python}` blocks in
`sections/01-introduction.tex`, `02-a-taste-of-pact.tex`, `03-memo.tex`, plus many
`\mintinline{python}{…}`). Guard: `minted_frozencache_colors_from_pygments_cache_157`
(`06_cluster_regressions.rs`) — drives the real binary against a hand-built tiny
`_minted/` and asserts the `#008000`/`#0000FF` color spans, with a no-cache control
proving the strict no-op. Implementation:
`latexml_contrib/src/minted_frozencache.rs` + hooks in `minted_sty.rs`.

**Limitation.** `\mintinline` snippets that Pygments leaves as plain names/punctuation
(classes `n`, `p`) are correctly uncolored (faithful to Pygments); only `\mintinline`
itself takes the cache path, not the `\newmintinline`-generated aliases.

### 158. acmart affiliation parts break AFTER the comma, not before

**Background.** acmart's `\affiliation{\institution{}\city{}\state{}\country{}}` puts each
address part on its own source line. The real `acmart.cls` `\institution`/`\city`/… use
`\unskip`/`\ignorespaces` (`acmart.cls` L1679, L2879) so the inter-part source newlines do
not become spaces, and joins the parts with a `, ` separator.

**Perl behavior**: SHARED failure. `acmart.cls.ltxml` (L97-101) ports `\lx@acm@addresspart`
WITHOUT the `\unskip`/`\ignorespaces`, and with a `,~` (comma + non-breaking-space)
separator. So each source newline between `\institution{}` and `\city{}` leaks as a space
BEFORE the comma (serialized `…Institute</ltx:text>\n, <ltx:text>New York…`). On a wrap
the breakable space sits before the comma, pushing the comma to the START of the next line;
and the `~` forbids a break AFTER the comma, so long affiliations break mid-part instead.
Same-host Perl renders identically.

**Rust behavior**: `\lx@acm@addresspart` (`acmart_cls.rs`) appends `\ignorespaces` (after its
`\fi`) so the trailing source newline is gobbled — the comma binds directly to the preceding
part — and uses a `, ` (comma + a REGULAR breakable space) separator, so a wrap breaks AFTER
the comma, matching the pdflatex golden. Empty parts are still skipped.

**Why it's safe.** Scoped to acmart's address-part joiner; the only change is which side of the
comma the breakable space sits on (and that inter-part source newlines no longer leak).

**Witnesses**: arXiv 2605.03143 ("Basis Research Institute, New York, New York, USA").

**Upstream**: to be filed at brucemiller/LaTeXML (acmart address parts should `\ignorespaces`
between parts and break after the comma, per the real `acmart.cls`).

### 159. A shared single affiliation renders once below all authors, not stranded on author 1

**Background.** LLNCS-style markup `\author{A \and B \and C}` + one `\institute{…}` with NO
per-author `\inst` marker means the institute is shared by every author (pdflatex centers it
once below the author row). LaTeXML's frontmatter model has no document-level affiliation slot
(`ltx:contact` lives only inside `ltx:creator`, per the RelaxNG `contact` content model), and
its only mechanism for "shared" is per-author replication.

**Perl behavior**: SHARED failure. `\institute` → `\lx@add@affiliation[labelseq=affiliation]`
gives the affiliation the label `affiliation:1`; each author gets an auto sequence label
`author:N` (`\lx@add@frontmatter@now`). In `relocateAnnotations` (`Base_Utility.pool.ltxml`
L880-910) the affiliation matches no author by its own prefix, then the prefix-stripped
fallback (`$unlabeltable{$noprefix}`, L899-900) reduces `affiliation:1` to `1` and binds it to
the FIRST author's `author:1` — so the whole shared institute is stranded under author 1 only.
Perl 0.8.8 (installed and vendored) and pre-fix Rust produce byte-identical output.

**Rust behavior**: `relocate_annotations` (`base_utilities.rs`) two-part fix. (1) A creator's
own role-sequence label (`author:N`/`editor:N`/`translator:N`) is NOT indexed into the
prefix-stripped fallback table, so a shared `affiliation:1` no longer binds to `author:1` by
number. A genuine per-author affiliation still matches EXACTLY via `labeltable` (the
`affiliation:1` that `\inst{1}` requests), so `\inst`-targeted markup is unchanged. (2) The now
un-targeted shared affiliation — with any `\email`/`\url` that inherited its label — is gathered
onto ONE trailing name-LESS `<ltx:creator role="author">`, kept as the last child of the authors
container. The ar5iv theme's existing breakout rule then renders a last-position shared
affiliation once, full-width, centered, below the author row.

**Why it's safe.** Only the auto self-sequence labels leave the numeric fallback; exact-label
(`\inst`) attachment and genuine cross-prefix misuse recovery are untouched. Guarded by
`frontmatter_llncs_shared_affiliation_below_authors`.

### 160. OmniBus captures `\orcid` and no-ops the running-head registers `\lefttitle`/`\righttitle`

**Background.** Unbound bundled journal `.cls` files fall through to the generic OmniBus fallback
(`INCLUDE_CLASSES` defaults false), which supplies a shared frontmatter vocabulary. Many journal
classes spell frontmatter macros in their own way; three appear widely and unambiguously across
the sandbox-arxiv-2606 corpus: `\orcid{id}` (a stored ORCID identifier, e.g. pasj02
`\def\orcid#1{…ORCID: #1…}`; some classes use the `\orcid[name]{id}` form), and the running-head
registers `\lefttitle`/`\righttitle` (e.g. jfm `\lefttitle#1{\gdef\@lefttitle{#1}}`).

**Perl behavior**: SHARED failure — Perl's `OmniBus.cls.ltxml` defines none of these three, so each
is `Error:undefined` and its content is dropped. pdflatex renders them (the ORCID as a link, the
running heads in the page headers).

**Rust behavior**: OmniBus (`omnibus_cls.rs`) maps `\orcid[]{}` → the existing `\lx@add@orcid`
helper, so the identifier is captured as a real `<ltx:contact role="orcid">` with an `orcid.org`
link (a surpass-Perl content recovery, sibling of divergence #144's `scrartcl \titlehead`); and
no-ops `\lefttitle`/`\righttitle`, which are page-layout registers with no document-content
meaning (correctly dropped, never leaked). Only this verified, class-consistent subset is added;
the ambiguous/variant journal spellings (`\aff` — a **superscript reference marker** in jfm, NOT
an affiliation; `\contribution`, `\correspondence`, `\data`, `\ack`, `\reportnumber`) stay
unhandled pending per-class output review (tracked in `SYNC_STATUS.md`).

**Why it's safe.** The three names are currently undefined for OmniBus docs (they only affect docs
that use them, all of which error today), the ORCID mapping reuses the canonical helper, and the
running-head no-ops drop only presentational registers. Upstream the same to Perl's
`OmniBus.cls.ltxml`. Guarded by `omnibus_captures_orcid_and_drops_running_heads`.

**Witnesses**: arXiv 2402.19043 (WDM: 5 authors, one shared `\institute`, no `\inst`).
Guardrails that must NOT regress: 2608.11332 (shared `\email` under `\inst{1}`), 2603.23669
(two-author-per-creator dedup), 2606.00313 (`\thanks`-abuse affiliations).

**Upstream**: the ar5iv CSS comment already anticipates this ("Ideally latexml's schema should
evolve to handle this via differently organized markup") — the trailing-creator normalization is
that markup.

### 161. `DefPlain` skips blanks before its required `{` (undelimited-argument scanning)

**Perl behavior**: the `DefPlain` parameter type calls `readBalanced(0,1,1)` bare
(`Base_ParameterTypes.pool.ltxml` L34, `Gullet.pm` L441-452), whose `require_open`
branch reads ONE raw token — a space/newline before the brace errors
`Expected opening '{'` and the body braces then execute inline. Bites every
`\lstnewenvironment{x}[1][]` whose `{begin}{end}` bodies sit on following lines —
the standard documentation style (~148 TeX Live doc manuals; witnesses:
`ltxdockit.sty` L561/L565 via abraces-doc, `cnltx-example.sty` L1015 via
snotez/elements/carbohydrates manuals).

**Rust behavior**: `DefPlain` (`latexml_engine/src/base_parameter_types.rs`) runs
`skip_spaces()` before the balanced read.

**Why**: real TeX skips blank space tokens when grabbing an undelimited argument
(tex.web `macro_call`), and LaTeXML's own `{}` reader (`readArg` → `readNonSpace`)
skips them too — the bare `readBalanced(require_open)` in `DefPlain` is internally
inconsistent, not a defensible alternative semantic. `\def`/`\gdef` reach `DefPlain`
after `UntilBrace`, where the skip is a no-op, so `\def`-family semantics are
untouched.

**Witnesses**: TL2025 doc corpus — abraces-doc (10→? errors), snotez-manual,
elements-manual, carbohydrates_en, flashmovie/test-beamer-0. 10-line repro in the
guard test.

**Upstream**: approved 2026-08-31 as branch-contained (perfect_kernel); upstream
filing deferred by user directive ("contain all work to this branch only").

### 162. Listings raw-line capture keeps the first body line after a line-crossing argument probe

**Perl behavior**: `readListingsLines`-style capture (`listings.sty.ltxml`) discards its first
`readRawLine` unconditionally as "the remainder of the `\begin{…}` line". When the environment
was defined with an optional argument (`\lstnewenvironment{x}[1][]`) and none is given, the
`Optional` probe (`readNonSpace`) crosses the newline after `\begin{x}` and unreads the body's
first character — the "first line" then IS the body's first line, and Perl swallows it. Verified
same-host on Perl 0.8.8: a two-line body comes back holding only line two.

**Rust behavior**: `listings_read_raw_lines` (`listings_sty.rs`) discards the first raw line
only when the gullet pushback is empty (`gullet::pushback_is_empty`) — pushback pending means an
argument probe already advanced into the body, so the first raw line (pushback + line remainder)
is real content and is kept.

**Why**: content preservation — real listings drops only the `\begin`-line remainder; the first
body line is typeset. Every `\lstnewenvironment`-defined example environment in the TL doc corpus
(cnltx, ltxdockit, … — the standard "define an example env, body on following lines" style) lost
its first line in both engines. Known residual: `\begin{env} same-line-junk` (no optional given)
now keeps the junk as body line 1 instead of dropping it — pathological input real listings warns
about; accepted.

**Witnesses**: TL2025 doc corpus manuals via cnltx-example.sty / ltxdockit.sty; 10-line repro in
`cluster_package_guards::defplain_skips_blanks_before_brace` (guards #161 + #162 together).

**Upstream**: branch-contained per user directive 2026-08-31 (sibling of #161).

### 163. `\makeindex` allocates the `\@indexfile` write stream (kernel contract subset)

**Perl behavior**: `\makeindex` / `\makeglossary` are full no-ops
(`latex_constructs.pool.ltxml` L4531-4532). Real latex.ltx `\makeindex` also
`\newwrite`s `\@indexfile`, and raw doc.sty / l3doc.cls-style code then writes
`\protected@write\@indexfile{…}` directly — with the stream never allocated, every
such write errors `undefined \@indexfile` (and cascades: Perl 0.8.8 lands at 101
errors + 1 fatal on l3kernel's own `saveenv.tex`, same-host verified).

**Rust behavior**: `\makeindex` → `\ifdefined\@indexfile\else\newwrite\@indexfile\fi`
(`\makeglossary` likewise for `\@glossaryfile`). Everything else stays nooped: no
`\openout`, and NO kernel-style redefinition of `\index` — the semantic
`\index SanitizedVerbatim` constructor remains in charge. Writes to the
allocated-but-unopened stream go to the log, exactly as real TeX behaves with no
file open.

**Why**: kernel-contract subset restoration; content untouched (the write's payload
was never document content). 14 TL-doc bundles (l3kernel manuals, robustindex,
postnotes, …) clear their first-error.

**Witnesses**: saveenv/saveenv (2 errors → 1), l3kernel/l3styleguide,
robustindex/robustmanual. Guard: `cluster_package_guards::makeindex_allocates_indexfile`.

**Upstream**: branch-contained per user directive 2026-08-31 (sibling of #161/#162).

### 164. `\usepackage`/`\documentclass` record the kernel's `\@raw@opt@<name>.<ext>` raw-option list

**Perl behavior**: the binding-level package loader handles options internally and
never mirrors the modern kernel's raw-option record. `\ProcessKeyOptions` (ltkeys,
kernel 2022+) reads EXACTLY `\@raw@opt@\@currname.\@currext` (latex.ltx L19398) —
finding it free, it processes nothing, so every key-option package silently loses
its load-time options (verified same-host on Perl 0.8.8: `[flag]` bool option
stays false).

**Rust behavior**: `pass_options_with_raw` (content.rs) is the single writer, as
the kernel's `\@pass@ptions` is (latex.ltx L18509-18526): `\gdef` on the first
pass, `\g@addto@macro{,#2}` after. `\PassOptionsToPackage`/`\PassOptionsToClass`
record their argument tokens at pass time, with the first expanded once
(`\expandafter{#2}`, computed by an `\edef` of `\unexpanded\expandafter{…}`);
the loader's explicit `[…]` list and bindings' string lists are re-read with
standard catcodes. Batch 56ia moved the writes there from a load-time rebuild of
the x-expanded string list, which re-tokenized an expl3 value
(`linespread = \c__fdu_line_spread_fp`, fduthesis.cls:193-202 →
ctexbook.cls:336) into `\s`, `__fp`… and did not exist before the load. Guard:
`class_census::passed_options_keep_their_tokens`.

**Why**: kernel-contract restoration for the growing class of `\ProcessKeyOptions`
packages. Minimal 12-line repro: a local .sty with `\keys_define:nn` +
`\ProcessKeyOptions` loaded `[flag]` — flag=OFF before, ON after (matching
pdflatex).

**Witnesses**: codedescribe manual (`[strict,infograb]` → `\PkgInfo` alias never
installed; 10 TL-doc bundles' first error). Guard:
`cluster_package_guards::process_key_options_sees_load_options`.

**Upstream**: branch-contained per user directive 2026-08-31 (sibling of #161-#163).

**Class half (same entry)**: the kernel likewise records the raw `\documentclass`
option text in `\@raw@classoptionslist` (latex.ltx L18718, first class only —
`\ifx\@classoptionslist\relax` guard). Modern babel reads THAT list for global
language options (babel.sty L4199 `\bbl@foreach\@raw@classoptionslist`);
without it `[french]{article}` + `\usepackage{babel}` silently loads nil.ldf
(`\og`/`\fg` undefined — 4+ French TL doc bundles; Perl identical). Recorded in
the same `input_definitions` cls block, gated by a `@raw@classoptionslist_recorded`
once-flag. Guard: `cluster_package_guards::raw_classoptionslist_recorded`.

### 165. `\@currsize` defaults to `\normalsize` (begin-document invariant)

**Perl behavior**: class bindings define `\normalsize`/`\small`/… as font
primitives that never route through `\@setfontsize`, so `\@currsize` — which real
LaTeX guarantees is set once `\begin{document}` has run `\normalsize` — stays
permanently undefined (verified same-host, Perl 0.8.8). Raw packages restoring
the surrounding size via `\@currsize` error `undefined` (linguistics doc family:
linguex, covington, philex, drs, movement-arrows).

**Rust behavior**: `\@currsize` is pre-defined as the expansion indirection
`\normalsize` beside `\@setfontsize` (`latex_constructs.rs`); a class that does
route through `\@setfontsize` overwrites it with the exact size command, exactly
as in real LaTeX.

**Why**: kernel-invariant restoration. Witness: linguex-doc converts 6 errors →
**0 errors, 0 warnings**. The former residual ("inside `\small` the default
still reads `\normalsize`") is closed by batch 54v: a primitive whose font
directive is a pure size switch (`font => {size => N}`, every class binding's
`\tiny`…`\Huge`) `\let`s `\@currsize` to itself when it runs — latex.ltx:14103
`\@setfontsize`'s first act — so `\ifx\@currsize\small` chains (ltugboat's
`\SMC`, latex-doc-ptr.sty:203-215) see the identity as in real LaTeX
(`def_primitive`, dialect.rs; guard `perfect_kernel_batch54::size_switch_lets_currsize`).

**Upstream**: branch-contained per user directive 2026-08-31 (sibling of #161-#164).

### 166. `\@filelist` entries carry kernel catcodes (letters as LETTER)

**Perl behavior**: filelist names are exploded all-catcode-OTHER, so a
source-level delimited parse over `\@filelist` whose delimiter was typed as
letters never matches — hep-font.sty's
`\def\hepfont@get@class#1.cls#2\relax` + `\expandafter…\@filelist\relax`
class-detection idiom yields an EMPTY #1, and the mis-split desyncs the
surrounding conditional bookkeeping ("Missing \fi or \else, conditional fell
off end" at the .sty's EOF — the 13-bundle hep-* `expected:\fi` cluster).

**Rust behavior**: `\@addtofilelist` receives `ExplodeText!` tokens —
alphabetic chars catcode LETTER, the rest OTHER — exactly latex.ltx's
`\string@makeletter` (L1784-1789) storage rule.

**Why**: kernel-contract restoration. hep-font.sty: first-error cluster →
**0 errors**; hep-paper manual 370-warning cascade collapses to 2 unrelated
errors. Residual noted: a FAILED delimited match should raise TeX's "use does
not match its definition" instead of silently returning empty arguments —
separate hardening: the leading-delimiter case landed in 56iq (#295); an `Until:` miss stays quiet by design
(parameter.rs:440-452).

**Upstream**: branch-contained per user directive 2026-08-31 (sibling of #161-#165).

### 167. LuaTeX-only packages get first-class treatment (texlua bridge + piton/luacode/showexpl bindings)

**Perl behavior**: no `\directlua` support and no bindings for luacode, piton,
or a content-bearing showexpl — LuaTeX-only packages either abort fatally
(piton: "LuaLaTeX is mandatory", killing the rest of the document) or lose
their bodies (showexpl's noop stub).

**Rust behavior** (user directive 2026-08-31): `latexml_engine::lua_bridge`
runs a persistent per-conversion `texlua` (assumed present with TeX Live);
`\lx@directlua` evaluates chunks with LuaTeX-manual semantics and
`tex.print`/`sprint` output re-enters the input stream. luacode.sty maps its
whole API onto it ({luacode} bodies EXECUTE); piton.sty's environments render
their bodies as listings (coloring is presentation; running piton.lua's LPEG
highlighter through the bridge is a possible refinement); showexpl's
{LTXexample} emits the code listing AND executes the body as the result.
CRITICAL constraint: the engine never defines `\directlua`/`\luatexversion`
under their real names — those are the LuaTeX-detection probes (babel et
al.), and claiming them flips package ecosystems onto luatex code paths
(measured: 26 suite tests red).

**Witnesses**: nicematrix manual (piton fatal truncated it at ~4% depth; now
full 8,334 lines convert), luacode/showexpl guard tests.

**Upstream**: branch-contained per user directive (perfect_kernel).

### 168. Opt-in `luatex` profile (`\usepackage[luatex]{latexml}`)

**Perl behavior**: one fixed engine identity (pdfTeX-model); LuaLaTeX-authored
documents take dead branches (engine probes false, `\directlua` undefined,
fontspec surface unreachable).

**Rust behavior** (user decision 2026-08-31): the Rust-only latexml.sty option
`luatex` declares the DOCUMENT LuaLaTeX-authored for this conversion:
iftex-level probes (`\iftutex`/`\ifluatex`/`\ifpdftex`… — state-consulting
conditionals, so load order doesn't matter), `\directlua`/`\luaescapestring`
under their REAL names (bridged to texlua), `\luatexversion`, the common
LuaTeX parameter surface (`\hyphenationmin`, `\outputmode`, `\suppress*error`,
…) and TLT direction primitives. The default identity is untouched without the
option; the perfect-kernel harness auto-selects it for docs whose LaTeX oracle
required lualatex. fontspec gains v2 post-optional signatures; a minimal
unicode-math binding absorbs math-font selection (both presentation).

**Witnesses**: hvfloat corpus preambles (`\setmonofont{…}[…]` bare after
libertinus→otf), TL doc bundles oracle-classified lualatex (~300).

**Upstream**: branch-contained per user directive (perfect_kernel).

**l3sys (batch 54m, 2026-09-03).** expl3 freezes `\c_sys_engine_str` and the
`\sys_if_engine_<e>` conditionals at format-build time (expl3-code.tex:7846-7861,
`\cs_if_exist:NT \tex_luatexversion:D {luatex}`), before the profile defined
`\luatexversion`, so `\sys_if_engine_luatex:TF` stayed FALSE and polyglossia's
gloss-latin.ldf:125 took its XeTeX branch (`\newXeTeXintercharclass` ×12; hang,
sample). The profile now re-derives them (`\c_sys_engine_str` = luatex, the
five engine conditionals, `\c_sys_engine_exec_str`/`_format_str` = luatex /
lualatex per :7868-7916). `sys_if_engine_opentype` is left as derived. Guard:
`perfect_kernel_batch54::l3sys_engine_identity_under_luatex_profile`.

### 169. Raw loader keeps `\ProvidesPackage`'s `\ver@<file>` (fallback-only `\fmtversion`)

**Perl behavior**: `Package.pm` L2393 (raw loadTeXDefinitions) unconditionally
`Let`s `\ver@<file>` to `\fmtversion` after the load — clobbering the version
string the file's own `\ProvidesPackage` just recorded. Every downstream date
guard (`\GetFileInfo`, `\@ifpackagelater`, toptesi.cls L44-73's version
comparison) then reads the format date.

**Rust behavior**: `\fmtversion` is applied only as a FALLBACK when the file
declared nothing — the guarded idiom Perl itself uses in the ltxml-binding
branch (and we already used at `content.rs` L652-660). Real LaTeX keeps the
declared string.

**Why**: kernel-quality — real LaTeX semantics; unlocks every version guard,
not one package.

**Witnesses**: 12-doc toptesi cluster ("the sty file you are using has a date
of <empty>" abort → gone). Guard:
`cluster_package_guards::raw_provides_version_survives`.

**Upstream**: branch-contained per user directive (perfect_kernel);
Perl-filing candidate.

### 170. Text accents carry the kernel's ROBUST `\meaning` shape

**Perl behavior**: `DefAccent` (TeX_Character.pool.ltxml L92-100) makes `\u`
et al. protected primitives, so `\meaning\u` renders as `\protected …` (or the
bare CS). tikzmath's 4-character meaning sniff
(tikzlibrarymath.code.tex L22-46) then classifies an accent CS used as a
`\tikzmath` variable (`\tikzmath{\u=int(…);}`) as a KEYWORD and errors.

**Rust behavior**: `DefAccent!` builds a two-level structure — the real
(protected) accent in the space-suffixed CS, the user name a plain macro
expanding straight to it — so `\meaning\u` starts with `macro:` (pdflatex
ground truth: `macro:->\T1-cmd \u \T1\u `, a macro via the `\<enc>-cmd`
dispatch) AND the accent stays expandable inside `\csname` (the twemoji
`\@namedef{…\'e…}` idiom, guarded by `cluster_csname_accent` — a literal
`\protect` token in the body broke it; the suite caught the first attempt).

**Why**: kernel-fidelity to latex.ltx's `\DeclareTextAccent`/robust-command
shape; fixes the whole meaning-sniffing class, not tikzmath alone.

**Witnesses**: cahierprof-doc 501→0 errors, ipsum-doc 103→0,
colorblind_doc 502→19, tikz-mirror-lens 502→38. Guard:
`cluster_package_guards::accent_meaning_robust_shape`. Residual: math-symbol
CSes as tikzmath variables (`\angle`, sunpath) are a separate meaning-shape
family.

**Upstream**: branch-contained per user directive (perfect_kernel);
Perl-filing candidate.

**Batch 56cq — the composite step.** The inner accent macro now does what
T1's `\DeclareTextComposite` does (latex.ltx lowercases the slot into the
CHARACTER token): applied to one alphabetic letter whose NFC with the
combiner is a single character, it EXPANDS to that character
(`setup_binding_language::accent_composite`, the letter's catcode kept); an
empty or multi-letter group, a digit, or a pair with no precomposed form
still expands to `\lx@applyaccent`, the stomach primitive. bibleref-parse
.sty:495-507 `\brp@@expandcs` re-expands a control sequence with
`\expandafter` until a character comes back: the inert primitive looped for
the whole 420 s cap on the manual's German book names (`\brp@parse
{IK\"onige}`); OT1 pdflatex hangs the same way, T1 pdflatex finishes because
the composite ends the loop. Observable deltas: `\texttt{\~n \^a}` gives
`ñ â` (T1 pdflatex's glyphs; the former `~n ^a` was Perl's typewriter hack,
kept for the empty-group idiom `\~{}`), and a math `tex=` reversion shows
the composed letter. Guards
`accent_composite_expansion::{expand_until_character_loop_over_an_accent_terminates,
accent_on_a_letter_expands_to_the_precomposed_character}`.

### 171. Required-brace hunt expands `\protected` macros (scan_left_brace fidelity)

**Perl/previous Rust behavior**: while hunting the required `{` of a
<general text> (`\expanded`, `\write`, …), protected macros were not expanded
— `\expanded\pp` with `\protected\def\pp{{abc}}` errored `Expected opening
'{'`. Every `\xinttheexpr` (whose `\expanded\csname XINTexprprint…` lands on a
\protected macro) emitted one such error.

**Rust behavior**: the brace hunt fully expands, matching TeX's
`scan_left_brace` (plain `get_x_token`; protection inhibits expansion only
during body absorption). Live-probed: pdflatex typesets `abc` for the case
above. Same argument-scanning-fidelity family as #161.

**Witnesses**: ~16-doc xint cluster — sim-os-menus-doc, ipsum-doc,
tikz-bagua-en, commalists-tools-doc. Guard:
`cluster_package_guards::expanded_protected_brace_hunt`.

**Upstream**: branch-contained per user directive (perfect_kernel).

### 172. Alignment brace ledger follows tex.web, not Perl's retract-on-unread

**Perl/previous Rust behavior**: `$LaTeXML::ALIGN_STATE` bookkeeping retracts
braces on EVERY pushback (Gullet.pm L343-358) and `readBalanced` localizes the
ledger to 1000000 with an entry `--` compensator. Self-consistent for balanced
token flows, but expl3 brace-tricks — `\if_true: { \else: } \fi:` pairs whose
halves travel through different reads, as l3tl's delimited replace machinery
(`\tl_greplace_all:Nnn`) generates — drift the ledger permanently: a kept `{`
pushed back (retracted) is later consumed as an arg opener (compensated), so
the retraction never cancels. A later cell-top `&` then misses the column-end
check and errors `Stray alignment "&"` — one per l3doc `{function}` block,
across every l3doc manual (17+ bundles). **Perl shares the failure** (verified
2026-09-01 on the shrunk repro).

**Rust behavior**: tex.web's protocol exactly. Braces count when SCANNED
(get_next §342-344, token lists included §357); pushback of previously-scanned
tokens retracts (back_input §325 — `unread_one`/`unread_vec`); expansion
output enters with NO adjustment (`begin_token_list` — new
`gullet::unread_expansion`, used by the read-loop invoke pushes); a
closure-backed expandable that re-emits tokens it READ pre-retracts them
(`gullet::retract_scanned_braces`, tex.web §368's back_input around
`\expandafter` — applied to `\expandafter`, `\noexpand`, `\@ifstar`,
`\@ifnextchar`, `\@ifnext@n`, `\eqno`, `\cprotect`); and `read_balanced` no
longer localizes or compensates — scan_toks §473-482 touches nothing, the
already-counted opening `{` itself keeps the ledger ≥1 inside a balanced text
so `&` cannot fire mid-argument. Alignment-machinery resets stay (template
insert 1000000 ≈ §7099; cell-start 0; per-alignment save/restore ≈ §15289/
§15300 via the `digest_alignment_body` ledger frame).

**Why**: kernel-quality; pdflatex converts every affected manual cleanly.
First-principles directive (user, 2026-09-01): root causes from tex.web over
Perl emulation.

**Witnesses**: l3doc `{function}`+`{syntax}` 6-line repro (perfect_kernel/archive/CLUSTERS_2026-09-01.md);
gotham-user-cmds (101 strays), se2thesis (52), citation-style-language-doc
(31), unicode-math-input (30), zref-check-code (25). Guards:
`cluster_package_guards::alignment_ledger_expansion_pushback::*` (both
directions: the l3 drift AND the `\@ifstar` re-emission that a naive
no-adjust-everywhere breaks — cells.xml/graphrot goldens caught it).

**Upstream**: branch-contained per user directive (perfect_kernel); Perl
issue-worthy (KNOWN_PERL_ERRORS #81).

### 173. xkeyval key presets implemented (Perl stubs them)

**Perl behavior**: `\presetkeys`/`\gpresetkeys`/`\delpresetkeys`/`\gdelpresetkeys`/`\unpresetkeys`/`\gunpresetkeys` are warn-stubs ("Presetting keys is currently not supported.", xkeyval.sty.ltxml L452-475); no `\setkeys` call ever applies presets.
**Rust behavior**: the six front-ends are verbatim xkeyval.tex L363-403 (`RawTeX!` in `xkeyval_sty.rs`), and public `\setkeys` takes the real front-end (`\XKV@testopta{\XKV@testoptc\XKV@setkeys}`, xkeyval.tex L437) so the already-present `\XKV@usepresetkeys` hooks fire — every `\setkeys` applies the family's head presets before and tail presets after the user keys, excluding user-passed names.
**Why**: complete support over stubs; packages run key code only from presets (cntperchap.sty L75-79 `\gdef\@cps@@keymacro@@tracklevel` — "section level … is unknown" without it). Oracle = xkeyval.tex + pdflatex.
**Witnesses**: cntperchap_doc 6→3, cntperchap_example (TeX Live doc corpus).
**Residual**: `\XKV@s@tkeys` passes the full `\XKV@fams` list rather than per-family `\XKV@tfam` inside the preset loop — a multi-family `\setkeys` whose preset key name exists in an earlier family could dispatch there (single-family callers unaffected).

### 174. `\index` sort keys are brace-protected through the `\@indexphrase` re-parse (Perl truncates at `]`)

**Perl behavior**: `process_index_phrases` emits `\@indexphrase[<sortas>]{<phrase>}` with the sort key raw between `[`/`]` (latex_constructs.pool.ltxml L4351-4353); a `]` INSIDE the sort key (examdoc `\indc{gradetable[v]}` → `\index{gradetable[v]@…}`) truncates the constructor's optional-arg re-parse at the first `]` — key attribute cut short and the display phrase spilled as illegal `<ltx:indexmark>` children (`malformed:ltx:text`).
**Rust behavior**: the emitted expansion wraps the sort key in a brace group (`[{<sortas>}]`), which the optional reader treats as opaque; `clean_index_key` strips exactly one whole-span outer pair.
**Why**: KNOWN_PERL_ERRORS #83 sibling — real makeindex splits the out-of-band `.idx` string where a `]` is harmless.
**Witnesses**: exam/examdoc 39-error family → 0; pgfornament docs.

### 175. Unknown section types open the element of their `\@startsection` LEVEL (Perl warns and opens `ltx:section`)

**Perl behavior**: `\@@numbered@section`/`\@@unnumbered@section` sanitize the type name against the schema (latex_constructs.pool.ltxml:599-607): a name that is not a known sectioning tag (and not `app`) gets `Warn('malformed', …)` and falls back to `ltx:section`, whatever its level. `\DeclareSectionCommand`-defined headings (KOMA `\DeclareNewSectionCommand[level=2,…]{task}`, tudaexercise; `\newcommand\Proof{\@startsection{Proof}{5}…}`, 1608.04650) therefore all become warned `<ltx:section>`s, breaking the nesting of the real headings around them.
**Rust behavior**: `\@startsection` — the only place the level is visible — records a `SECTION_ELEMENT` mapping for a type it does not know (part −1, chapter 0, section 1, subsection 2, subsubsection 3, paragraph 4, subparagraph ≥5, classes.dtx conventions); `section_element_for_type` consults it before Perl's warned fallback, which remains for a type that never went through `\@startsection`.
**Why**: the fallback is Perl's own error-recovery path (a `malformed` warning is emitted), and the level is exactly the information LaTeX uses to place the heading — kernel-quality recovery, no TeX-semantics change; output changes only where Perl warned. Pairs with KNOWN_PERL_ERRORS #118 (the level is now read as a TeX <number>, so `\numexpr` levels work at all).
**Witnesses**: tudaexercise (`\task` → `<subsection>`, `Warning:malformed:ltx:task` gone), 1608.04650 `\Proof` → `<subparagraph>`.
**Guard**: `perfect_kernel_batch53::startsection_level_is_a_tex_number`, `koma_declaresectioncommand_heading_is_a_subsection`.
**Upstream**: not yet filed.

### 176. A zero-width `\vrule` is a strut, never an alignment border (Perl marks it a column rule)

**Perl behavior**: `\vrule` inside an alignment sets `isVerticalRule` whenever `h > 3 * w` (TeX_Box.pool.ltxml L811-814), so `\vrule height 12pt width 0pt` — the strut idiom — becomes a cell `border` (`border="ll"` on `\halign{\vrule height 12pt width 0pt#&\vrule#&#\cr &&a\cr}`); the `w == 0 → invisible` branch only runs outside alignments. Perl never meets the case through `\strut` because its `\strutbox` is void.
**Rust behavior**: `w == 0` is `invisible` + `alignmentSkippable` everywhere (tex_box.rs); the alignment peel and the normalize `isrule` test treat a box holding only struts as a strut (`Digested::is_strut`, `\hbox{\vrule … width\z@}` = latex.ltx:12596's `\strutbox`, which `\strut` `\unhcopy`s into every TeXbook `\halign{\strut#&…}` template cell). `\null` (`\hbox{}`) is NOT a strut and stays opaque to the peel, as in Perl.
**Why**: a zero-width rule has nothing to draw; with the real `\strutbox` (needed by fillwith/fullwidth coffins) every strutted halign row grew an empty bordered cell (halignatt.tex). Kernel-quality, engine-level, Perl would take the same fix.
**Witnesses**: halignatt.tex (TeXbook p.247 table), TL doc corpus fillwith/fullwidth manuals.
**Guard**: `perfect_kernel_batch54::zero_width_vrule_is_a_strut_not_a_border`, `53_alignment::halignatt_test`, `cells_test` (makecell `\null` opacity).
**Upstream**: not yet filed. KNOWN_PERL_ERRORS #131.

### 177. A source-tree relative package path falls back to its basename (Perl fails the load)

**Perl behavior**: `\usepackage{../tex/tikzpingus}` (tikzpingus-doc.tex:16 — the CTAN source layout where `doc/` and `tex/` are siblings) is looked up literally; in the installed TeX Live tree `../tex/` does not exist, so the package never loads and the manual's whole shape/layer error mass (354 lines) follows. pdflatex fails identically there.
**Rust behavior**: `require_package` (`binding/content.rs::source_tree_basename`) reroutes a request containing a directory separator to its basename when the literal path resolves nowhere AND the basename does (kpathsea), with an `Info:loading` note. A relative path that resolves (a paper's own `./mypkg`) still loads the local file.
**Why**: build-environment mismatch, not TeX semantics; the manual's intent is the installed package. Gated on a miss so arXiv-local packages (memory 0906.3507) are untouched.
**Witnesses**: 61 TL doc/latex manuals with `\usepackage{../…}` (tikzpingus, classicthesis, pgf-go, cora-macs …).
**Guard**: `perfect_kernel_batch54::relative_package_path_falls_back_to_basename`.
**Upstream**: not filed (environment-specific).

### 178. A shipped page advances `\c@page` (Perl leaves it at 1 forever)

**Perl behavior**: `\lx@newpage` is a bare `<ltx:pagination>` marker (TeX_Page.pool.ltxml:55); no output routine ever runs latex.ltx:15271's `\global\advance\c@page\@ne`, so `\value{page}` is 1 for the whole document and the standard pad-to-page idiom `\loop\ifnum\value{page}<N \null\clearpage\repeat` (knowledge.tex:803-809) never terminates (Perl hangs; Rust's box-cycle guard made it `Fatal:Stomach:Recursion`).
**Rust behavior**: `\lx@newpage` (tex_page.rs) emits the marker and then `\global\advance\c@page\@ne` — every `\clearpage`/`\cleardoublepage`/`\newpage`/`\eject`/`\supereject` counts a page, as `\@outputpage` does.
**Why**: counting the page markers is the honest page model available without a page builder; documents with no page break keep page 1, so output changes only where a break was written. Pairs with `\pagegoal`=`\vsize` (batch 54).
**Witnesses**: knowledge manual (P47), any `\null\clearpage` fill loop.
**Guard**: `perfect_kernel_batch54::clearpage_advances_the_page_counter`.
**Upstream**: not filed.

### 179. `\chapter` exists only where the class made a chapter counter (Perl defines it in every class)

**Perl behavior**: `DefMacroI('\chapter', undef, '\@startsection{chapter}…', locked => 1)` in the kernel pool (latex_constructs.pool.ltxml:557), so `\@ifundefined{chapter}` is FALSE under article/scrartcl. blindtext.sty:243 `\blinddocument`, hvfloat's 50 test documents, coseoul, xassoccnt take the chapter branch and error `undefined:\thechapter` (53 docs, sweep 30).
**Rust behavior**: after `\documentclass` loads its class, `\chapter` is `\let` undefined when `\c@chapter` is undefined (Perl's own "has chapters" probe, pool:690) — except under the OmniBus fallback for an unknown class, which may have chapters (it autoloads book.cls on `\thechapter`, arXiv:2602.10407).
**Why**: latex.ltx defines no `\chapter`; only report/book-like classes do. An arXiv paper never uses `\chapter` in a chapterless class (pdflatex would refuse to compile it).
**Witnesses**: coseoul/cosexamp, hvfloat ×50, modular, sillypage, xassoccnt_totalcounters_example.
**Guard**: `perfect_kernel_batch54::chapter_is_undefined_in_a_chapterless_class`, `06_cluster_regressions::cluster_omnibus_chapter_book_autoload`.
**Upstream**: not filed.

### 180. `\@trivlist` is the shared list opener (Perl neutralizes it to `\relax`) — PLANS P38

**Perl behavior**: `DefMacro('\@trivlist', '\relax', locked => 1)` (pool:1732), so a raw class or package that redefines `\list` alone — memoir.cls:4580 (latex.ltx:15848 verbatim; its `adjustwidth` is `\begin{list}`), autolist.sty:37-109 — opens nothing, while `\endlist` → `\endlx@list` expects the `\lx@list` frame: "Attempt to end mode internal_vertical" on every such list (digiconfigs, memman, memexsupp, MemoirChapStyles, dlfltxb ×3, biblatex-oxref ×4, shipunov autolist ×2, … 28 docs, sweep 30).
**Rust behavior**: `\@trivlist` = `\lx@trivlist@setup\lx@list`: starts the itemization unless the list's setup already ran `\usecounter` (which binds `itemcounter` in the current frame), then opens the `\lx@list` itemize+frame; `\endtrivlist` pops an `\lx@list` frame when it is on top (latex.ltx:15913 `\endlist` → `\endtrivlist`; 0802.2207 `mathtrivlist` pairs `\@trivlist` with `\endtrivlist`). The kernel `\@trivlist`'s `\@noitemerr` paths are not reproduced.
**Why**: kernel-quality: the two closers now close what the kernel's own opener opened; our `\list`/`\trivlist` bindings are unchanged.
**Witnesses**: digiconfigs 5→0 errors, memman 211→104.
**Guard**: `perfect_kernel_batch54::raw_list_opens_through_trivlist`.
**Upstream**: not filed.

### 181. A `\\` inside a brace group of an alignment cell is an in-cell break (Perl ends the row and truncates the table)

**Perl behavior**: `\lx@alignment@newline` (TeX_Tables.pool.ltxml:583-594, "VERY tricky (and mostly Wrong)") always emits the `\lx@hidden@cr` row terminator; real LaTeX's `{\ifnum0=`}\fi` guard (latex.ltx:16583-16594) is not modeled. `{Y \\} & C \\` ends the row inside the group, the cell's `}` is then read as the alignment's `\egroup`, the tabular is truncated to one cell (Perl: silently; Rust: "Attempt to close non-boxing group", "Attempt to end mode restricted_horizontal", stray `&` — 3 errors per such cell).
**Rust behavior**: with a small positive `align_group_count` (a user brace group of the current cell; the template mask parks the count at 1000000 between a `&` and the next column's `before-column` marker, so `S & \\` after an EMPTY cell still ends the row) `\\` expands to `\newline` — a break inside the cell; the `[dim]` is dropped.
**Why**: tabularray reads its body itself and makes exactly this `\\` a line break inside the cell (ProfSio.sty:2917 `\SetCell{l}{… \\ …}`, `{Consignes\\ \\ …}` cells — the ProfSio manuals, pdflatex-clean); for a plain tabular pdflatex errors "Misplaced \cr", and an in-cell break is the benign recovery. The row-ending path is untouched.
**Witnesses**: profsio ProfSio-doc-fr (390 errors), annexe71/72/73.
**Guard**: `perfect_kernel_batch54::newline_inside_a_cell_group_is_an_in_cell_break`.
**Upstream**: not filed.

### 182. A `\caption` with no float ancestor becomes the float of its `\@captype` (Perl errors twice per caption) — PLANS P16 ii

**Perl behavior**: `\@@caption` / `\@@toccaption` are `^^<ltx:caption>` float-up constructors (latex_constructs.pool.ltxml:3368-3370; the counters are rescued by `RescueCaptionCounters`, :3203-3214). A `\caption` whose `\@captype` was set by hand in a box that is not a float — tufte-common.def:1110-1133 `marginfigure` (`\begin{lrbox}…\begin{minipage}…\def\@captype{figure}`, then `\marginpar{\usebox{…}}`), the `\def\@captype{figure}` minipage idiom, tocbasic's `\captionaboveof{table}` at top level — finds no ancestor able to hold the caption and errors `<ltx:caption> isn't allowed in <ltx:block>` plus the `ltx:toccaption` sibling (pgfornament ornaments 40+40, memman 46+46, xltabular).
**Rust behavior**: in LaTeX such a caption IS its type's caption — "Figure 1:", the target of the following `\label` — so `\@@caption` builds that float: an `ltx:figure`/`ltx:table`/`ltx:float` at the nearest ancestor that admits one (the minipage's `inline-logical-block`, `Para.model`), carrying the counter tags and id the caption stepped, with the `ltx:caption` inside — the shape `\captionof` already gives (#89). The caption TAKES its counter values at digest time when no float environment is open (`before_float`'s scoped `lx@in@float`), so a later caption-less float cannot reuse its id. Only where no such float can be placed does it degrade to the inline shape Perl's own no-`\@captype` path emits — `ltx:text class="ltx_caption"` minus the `\lx@tag` pieces — with no toc entry. (Before batch 56gs the degrade was the only behaviour: unnumbered captions and dangling `\ref`s.) The float holds the caption, not the box's other content (built in a `_CaptureBlock_` at that point), and no `ltx:toccaption` (its constructor runs before the float exists; the list-of-figures entry falls back to the caption text).
**Why**: kernel-quality: the same document keeps its real `figure`/`table` captions tagged and toc-listed, and the box's caption is numbered and referable as in the PDF instead of dropped with two errors or left as text.
**Witnesses**: pgfornament ornaments, memman, xltabular-doc, the tufte-latex manuals.
**Guard**: `perfect_kernel_batch54::caption_outside_a_float_becomes_its_float`; `latex_via_exemplos_residue::caption_settype_declares_the_float_type`.
**Upstream**: not filed.

### 183. A native (closure) macro is never `\ifx`-equal to a token macro (Perl's `Equals` matches its `CODE(0x…)` text)

**Perl behavior**: `Common/Object.pm:80-95` `Equals` treats a CODE-ref expansion as equal to a token list whose string is the same `CODE(0x…)`, so a `\meaning` → `\scantokens` → `\ifx` round-trip (etoolbox's `\etb@ifscanable`, biditools.sty:792 `\bidi@ifscanable`) reports a native macro as reconstructable. `\ifpatchable*{\begin}` answers yes, and biditools' own patchcmd clone then `\def`s `\begin#1{…CODE(0x…)}` — a macro whose tail is the literal text `CODE(0x…)`; the `\begingroup` that `\begin` pushed is gone and the document errors ("Attempt to end mode internal_vertical" on a bare `x` body; crbox-doc/ghab-doc "close a group that switched to mode horizontal"). Rust carried the same hack (2023) in `ExpansionBody::PartialEq`.
**Rust behavior**: cross-variant `\ifx` is `false`: `\ifpatchable*{\begin}` answers no, biditools leaves the native `\begin`/`\end` alone, and the environments keep their groups. The Perl-golden fixture `expansion/etoolbox` line "begin is patchable" is re-blessed to "begin is not patchable".
**Why**: kernel-quality: a closure cannot be rebuilt from its `\meaning`; saying it can only lets patchers install broken macros. Same-shaped as the native etoolbox `\patchcmd` refusing closure bodies (etoolbox_sty.rs).
**Witnesses**: crbox/crbox-doc, ghab/ghab-doc (perfect-kernel corpus).
**Guard**: `perfect_kernel_batch54::biditools_env_patch_leaves_begin_end_intact`.
**Upstream**: not filed.

### 184. `\DeclareTextAccent` defines the accent (Perl ignores it)

**Perl behavior**: `\DeclareTextAccent{\cs}{enc}{slot}` is `ignoredDefinition` (Base_Utility), so greek-fontenc's `\accdasia`/`\accperispomeni`/… (lgrenc.def:439-470) never exist: teubner.sty:165 `\let\~\accperispomeni` makes `\~` undefined (teubner-doc 1→87 errors once the encoding is live), textalpha's breathings `\<`/`\>` error.
**Rust behavior**: `\<enc>\cs{#1}` appends the combining mark(s) the slot's standalone glyph stands for (the kernel accent combiner map plus a Greek diacritics table: varia, oxia/tonos, perispomeni, dialytika, psili, dasia and their composites); the bare `\cs` becomes the `\fi`-free encoding dispatcher. A bare command that already exists (the kernel accents, `DefAccent`'d natively) and its encoding slot are left alone, so the native accent path keeps its dotless-i and typewriter rules. The dispatcher shape shared with `\DeclareTextCommand`/`\ProvideTextCommand`/`\DeclareTextSymbol` is `…\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{…}{…}`: Perl's `…\else…\fi` shape hands the trailing `\fi` to an argument-taking text command as its argument.
**Why**: kernel-quality: LGR/Greek documents get their accents instead of an undefined-CS cascade. Since batch 56ji (#312) a kernel accent whose argument has no precomposed form takes the encoding's declared composite, which changes some Latin output as pdflatex does (T1 `\"{ab}` → ä).
**Witnesses**: teubner/teubner-doc, greek-fontenc (char-list, textalpha-doc), any babel-greek + polytonic text.
**Guard**: `perfect_kernel_batch54::declare_text_accent_defines_greek_diacritics`.
**Upstream**: not filed.

### 185. The trivial-recursion guard anchors on the invoking token (Perl: the definition's CS)

**Perl behavior**: `Expandable.pm:80-89` errors "Token \x expands into itself!" when a macro body begins with `$$self{cs}` — the Expandable's home CS. A `\let` alias shares the object, so invoking the alias of a legitimately self-headed body fires too: musixlyr.tex:709-722 stores `\cont@<verse>` bodies that begin with `\cont@<verse>`, snapshots them into `\der@kontext`, clears the original and expands the alias (recorder-fingering, undar-digitacion-doc 14 errors).
**Rust behavior**: the comparison is `t0 == self.cs && t0 == invoking token` (`get_current_token()`). `\def\x{\x}` invoked as `\x` still fires; through an alias the loop is caught one step later when `\x` itself expands.
**Why**: kernel-quality: TeX has no such guard at all; ours exists to turn a certain hang into an error, and it must not fire on a finite expansion.
**Witnesses**: recorder-fingering/recorder-fingering, undar-digitacion/undar-digitacion-doc.
**Guard**: `perfect_kernel_batch54::recursion_guard_anchors_on_the_invoking_token`.
**Upstream**: not filed.

### 186. Block content in a TikZ node is wrapped in `svg:foreignObject` (Perl: schema error)

**Perl behavior**: `pgfsys-latexml.def.ltxml:202` absorbs the node box straight into `svg:g`; a `\verb` or paragraph in the node text opens `ltx:p` and errors `<ltx:p> isn't allowed in <svg:g>` (makeshape/testsample 22, optikz_doc 46). `LaTeXML-misc.rnc:114` limits `svg:foreignObject` to `Inline.model` (its own comment: "eventually must adapt to put LaTeXML objects in foreignObject").
**Rust behavior**: `SVG.foreignObject.content = parent Flow.model`, and `find_insertion_point_qsym` auto-opens an `svg:foreignObject` (`_autoopened`/`_autoclose`) for a child an `svg:*` parent cannot hold but a foreignObject can — the shape `\lxSVG@includegraphics` already uses for non-SVG content and the sibling of the `ltx:inline-block` fallback (PLANS P37).
**Why**: kernel-quality: the node content is kept, well-formed, in the element SVG designed for it.
**Witnesses**: makeshape/testsample, optikz/optikz_doc.
**Guard**: `perfect_kernel_batch54::verbatim_in_a_tikz_node_gets_a_foreign_object`.
**Upstream**: not filed.

### 187. `\@ifundefined` leaves the probed name undefined (Perl defines it as `\relax`)

**Perl behavior**: `Base_Utility.pool.ltxml:23-31` probes with the
`\csname…\endcsname\relax` idiom, so after `\@ifundefined{foo}{}{}` the name
`\foo` IS defined (`\relax`) — the golden `t/expansion/definedness.xml`
records "After the @ifundefined: `\relax`, IS defined, IS cs-defined".
**Rust behavior**: no side effect, as latex.ltx:1729-1737 (`\ifcsname`, LaTeX
2020+) and pdflatex on the same test ("undefined, is NOT defined, is NOT
cs-defined"); the golden is re-blessed to pdflatex's answer.
**Why**: a reentrancy-guarded file loaded as `\@ifundefined{sentinel}
{\input file}{}` must find its sentinel undefined — polyglossia's
gloss-*.ldf:591 + babelsh.def:1 (19 languages) early-exited with
`\bbl@afterfi` undefined (KPE #158). Kernel-quality, engine-level, pdflatex
is the oracle.
**Witnesses**: hang/hang, hang/sample (TeX Live doc corpus).
**Guard**: `perfect_kernel_batch54::ifundefined_does_not_define_the_name`;
golden `tests/expansion/definedness.xml`.
**Upstream**: not filed.

### 188. Box constructors read their content as a live `HBoxContents` body (Perl: an isolated argument)

**Perl behavior**: `LaTeX.pool.ltxml` defines `\mbox{}`, `\@makebox[][]{}`,
`\@framebox[][]{}`, `\raisebox{}[][]{}` with a plain `{}` argument digested in
isolation under `mode => 'restricted_horizontal'`. Code that OPENS a box inside
the argument and CLOSES it after (ulem's `\UL@start` = `\setbox\UL@box\hbox
\bgroup…\bgroup`, `\UL@stop` = `\egroup\egroup`, driven by `\hss` →
`\UL@hskip` → `\afterassignment\UL@reskip`; examdesign.cls:186-200 around
`\makebox[.5in][r]{\hss}` at :1210) meets the wrong frame: "`\egroup` Attempt
to close a group that switched to mode restricted_horizontal" then a cascade
(examdesign examplea/b/c 101 errors each; Perl identical).
**Rust behavior**: the content parameter is `HBoxArgContents` — read and
digested in the SAME list as latex.ltx's `\mbox{#1}` = `\leavevmode\hbox{#1}`
(:16082) and `\@makebox`/`\@framebox`/`\raisebox`'s `\hb@xt@`/`\hbox`, with the
one-frame `readBoxContents` loop (tex.web §1083 `begin_box` pushes nest and
save level together, §1100 `package` pops both) and `bounded => true` so the
frame still scopes the box; `sizer` keeps the width/height computation. The
common shapes (`\makebox[2cm][r]`, `\fbox`, `\raisebox`, `\framebox`, math
`\fbox{$…$}`) keep their output. The reader (`read_box_arg_contents`,
batch 56ck) takes ONE macro argument, as Perl's `{}` and `\mbox[1]` do — a
braced group on the live gullet, or a single token digested as an isolated
list (Perl `readArg`'s `Tokens($token)`, so `\mbox\emph{x}` — rejected by
pdflatex, lenient in both engines — yields Perl's empty `<emph/>` with `x`
outside the box) — not the
`\hbox` primitive's forward scan to the next `{` (`read_box_contents`, kept
for `\hbox`/`\vbox`): that scan swallowed the `}` closing an enclosing group
on an unbraced argument (`\subsection{… ggg\\\mbox\qquad and packages}`,
ltnews issue 40: one leaked group per sectioning re-digest, the title cut
after the `\\`, "\end occurred inside a group at level 4").
**Why**: kernel-quality: the box body is a TeX list, not an argument; the
reader/frame shape is tex.web's. Engine-level (four constructors), Perl would
take the same change.
**Witnesses**: examdesign/examplea, examplea-b, examplec (TeX Live doc corpus).
**Guard**: `perfect_kernel_batch54::{box_constructor_content_is_a_live_hbox_body,
hbox_reader_is_one_frame}`, `mbox_argument_is_bounded::unbraced_box_argument_is_one_token`.
**Upstream**: not filed.

### 189. Sectioning units inside a list item or figure insert without a diagnostic (Perl: schema error)

**Perl behavior**: `Document.pm` openElement errors `<ltx:subsection> isn't
allowed in <ltx:item>` and inserts the nested node anyway.
**Rust behavior**: the sectioning-in-frontmatter leniency (`document.rs`,
witness 2311.06870) covers the whole sectioning family
(`section`…`subparagraph`) inside `ltx:item` and `ltx:figure`; the structure
is byte-identical to Perl's, without the error.
**Why**: LaTeX runs `\section`/`\paragraph` inside an `\item` or a float
body (the heading is set in the list's indentation), pdflatex is clean, and
both engines already build the nested node. Auto-closing the list/figure was
rejected: it produces a structure neither engine emits and mis-nests the
following items.
**Witnesses**: ddphonism/ddphonism, phonrule, prerex/prerex, pdfmarginpar
(TeX Live doc corpus).
**Guard**: `perfect_kernel_batch54::sectioning_unit_inside_item_or_figure_is_lenient`.
**Upstream**: not filed.

### 190. Math content in a ref-family element auto-opens an inline `ltx:Math` (Perl: schema error)

**Perl behavior**: `TeX_Math.pool.ltxml:42` autoOpens only `ltx:XMText`, so a
`\hyperref[l]{b}` or glossaries' `\glsdisp{k}{k}` reached in math mode
(glosmathtools.sty:74 `\ensuremath{\glsdisp…}`) errors `<ltx:XMTok> isn't
allowed in <ltx:glossaryref>` (sample_glosmathtools_en/fr 53 each).
**Rust behavior**: `find_insertion_point_qsym` auto-opens `ltx:Math
mode="inline"` + `ltx:XMath` (`_autoopened`/`_autoclose`) when a math node
(`ltx:XM*`) arrives in an element that can hold `ltx:Math` (Inline.model)
and is not one of the p/text/emph cascade cases — the shape `\text{$k$}`
already has; the math post-pass gives it `tex`/`text`/`xml:id`.
**Why**: kernel-quality: the content is math and the schema has the element
for it; the same rule family as #186 (foreignObject) and the inline-block
cell fallback.
**Witnesses**: glosmathtools/sample_glosmathtools_en, _fr.
**Guard**: `perfect_kernel_batch54::math_content_in_a_ref_gets_an_inline_math`.
**Upstream**: not filed.

### 191. `ProtectedXUntil` / `ProtectedExpanded` parameter types (Rust-only)

**Perl behavior**: `Parameters.pm` offers `XUntil:` and `Expanded` (full
expansion) and `ExpandedPartially` (e-TeX `\protected` deferred, `\the`
once). None of them touches LaTeX's `\protect`, so a binding that scans user
text with expansion (siunitx.sty.ltxml:1414 `\lx@SI@column@parse
XUntil:\lx@si@column@end`) expands a raw `\DeclareRobustCommand` body; with
Perl's primitive `\small` the loop never shows.
**Rust behavior**: two additional types run the read under latex.ltx's
`\protected@edef` regime (:1442-1454 `\let\protect\@unexpandable@protect`,
:1384) — `ProtectedXUntil:<delim>` and `ProtectedExpanded` — via
`state::with_let` (a temporary `\let` restored after the body) and
`gullet::read_x_until`, collapsing `\noexpand`'d tokens to their identity when
stored (tex.web §1149). Outside LaTeX the regime is absent and the read is
plain. The Perl spec names stay the stable boundary; the new names are
one-line aliases over the same readers, which are also callable from a binding
body.
**Why**: kernel-quality: an `\edef`-strength scan of LaTeX text without the
`\protect` regime is a pdflatex overflow (`\edef\x{\small}`), not a Perl
design choice; the reader must carry the regime, and the declared type keeps
reversion (`tex=`), `LXML_TRACE_ARGS` and the delimited-scan EOF policy that a
hand-rolled gullet loop in a `sub` body loses. KNOWN_PERL_ERRORS #178.
**Witnesses**: zugferd/zugferd-invoice (`S` cell opening with an unbraced
`\small` under scrartcl, `PushbackLimit`).
**Guard**: `perfect_kernel_batch54::s_column_unbraced_size_command_is_scoped`,
`state::reentrancy_tests::with_let_restores_the_prior_meaning`.
**Upstream**: not filed (the Perl binding cannot loop with its primitive size
commands).


### 192. `\VerbatimFootnotes` captures the footnote body live (Perl freezes it as a constructor argument)

**Perl behavior**: `latex_constructs.pool.ltxml:454-490` locks `\footnote`
and `\@footnotetext` to constructors with ordinary `{}` body parameters.
Consequently fancyvrb.sty:33-58 and fancybox.sty:647-668 cannot install their
`\afterassignment` reader, and `\verb` or a short-verbatim delimiter inside a
footnote is tokenized before its verbatim catcodes apply (two errors in the
minimal witness, and a fatal/cascade in the fancybox manual).
**Rust behavior**: the locked public footnote entry points dispatch through
private current-implementation macros. The package-scoped
`\VerbatimFootnotes` supplied by the existing fancyvrb/fancybox bindings
selects live-body constructors: `\afterassignment` consumes the original
opening brace, `\bgroup\aftergroup` turns its closing brace into the body
boundary, and `capture_body` digests the intervening tokens under the active
verbatim catcodes. Loading neither package does not define the public command.
**Why**: kernel-quality, user-approved with the queued perfect-kernel surpass
shapes on 2026-09-02. This preserves the package's content-bearing feature;
silently freezing the argument loses or executes verbatim source and can
truncate the surrounding document.
**Witnesses**: fancyvrb/fancyvrb-doc, fancybox/fancybox-doc, nicematrix.
**Guard**: `cluster_units::footnote::{verbatim_footnotes_allow_verb_in_footnote,
verbatim_footnotes_is_package_scoped}` and
`perfect_kernel_batch54::nicematrix_shortvrb_verbatim_footnotes`.
**Upstream**: not filed.

### 193. A conditional left open by a re-parsed numeric argument is closed by expansion (Perl leaks the if-frame)

**Perl behavior**: `Parameters.pm:78 reparseArgument` reads a `{Dimension}` /
`{Glue}` / `{Number}` argument in an isolated mouth; when the numeric scan stops
before a pending `\else…\fi` (the internal-register path of `Gullet.pm:961-998`
returns without `skip1Space`), `readingFromMouth` (`Gullet.pm:131-146`) discards
the leftovers unexpanded, so the `\ifx` frame stays on `if_stack`. The witness
`\parbox[t]{0pt}{s\hspace{\ifx\a\b\Lreg\else\a\U\fi}e}` then reports
`Extra \else` and, at `\end{document}`, "open conditionals".
**Rust behavior**: `Parameters::reparse_argument` records the if-depth before the
isolated mouth and, after the read, expands the remaining tokens of that mouth via
`read_x_token` until the depth is back — exactly what TeX's `get_x_token` after
`scan_dimen` does with the trailing `\fi` (tex.web §448/§461). A genuinely
unbalanced argument (`\hspace{\iftrue 3pt}`) warns `expected:\fi` and drops the
orphan frames instead of leaking them.
**Why**: kernel-quality; pdflatex converts the witness with 0 errors and a single
`\parbox`. The common path (no lingering conditional) is untouched.
**Witnesses**: typog-example (`tools/perfect_kernel/repros/expansion-primitives/parbox_dimen_conditional_double.tex`).
**Guard**: `cluster_package_guards::dimension_conditional_in_parbox_does_not_leak_or_duplicate`.
**Upstream**: not filed.

### 195. `\endgroup` against an open math frame inserts the missing `$` (Perl leaves the frame open)

**Perl behavior**: `Stomach.pm:367-369` `endgroup` sees `BOUND_MODE` bound on the
top frame, reports "Attempt to close a group that switched to mode math" and does
not pop ("maybe we'll recover?" — no recovery was ever written). The math frame
stays; every later `\endgroup`/`\egroup`/box end re-errors and the `<ltx:Math>`
leaks out of its paragraph. tikz calc's failure path produces exactly this: the
bailout `\tikz@cc@end$` (tikzlibrarycalc.code.tex:118-129) eats the injected `$`
and `\tikz@finish` (tikz.code.tex:2500-2508) re-inserts the orphaned one as real
inline math (`\begingroup $\endgroup`: pdflatex 1 / Perl 4 / Rust 4 errors; the
calc witness pdflatex 3 / Perl 24 / Rust 23).
**Rust behavior**: `stomach.rs::endgroup` follows tex.web §1063/§1064 `off_save`:
when the top frame is an inline or display math switch, it reports "Missing $
inserted", pushes `$` (or `$$`) and the closer back onto the input, and returns;
the `$` closes the math through the ordinary `\lx@dollar@default` path and the
`\endgroup` then matches its group. `}`/`\egroup` keep TeX's §1069 shape (the
brace is dropped, math stays open) and every non-math mode switch keeps Perl's
behaviour.
**Why**: kernel-quality; TeX's own recovery, one diagnostic instead of a cascade,
and the document tree stays well-formed.
**Witnesses**: `tools/perfect_kernel/repros/boxes-groups/endgroup_open_math_off_save.tex`,
`tools/perfect_kernel/repros/graphics-tikz/calc_fraction_partway_bailout_mode_frame.tex`;
kblocks-doc, ozguide, nathguide, chemobabel-en/ja carry the cascade.
**Guard**: `perfect_kernel_batch56::endgroup_against_open_math_inserts_the_missing_dollar`.
**Upstream**: not filed.

### 196. A math shift inside a group opened within math defers the math end to the group's end (Perl leaves the frame open)

**Perl behavior**: `$\bm{\hat{m}$} b` (kblocks-doc.tex:207; titlecaps' word-wrap
splits `$x^2$` across its `{…}` groups): the `$` reaches `endMode('math')` while
the bounded constructor's argument group is on top; Stomach.pm:367 reports
"Attempt to end mode math" and leaves the math frame, so `<ltx:Math>` leaks to the
document root and every later closer re-errors (pdflatex 0 / Perl 4 / Rust 4
before this entry).
**Rust behavior**: tex.web §1131 `off_save` — TeX inserts the missing `}`, closing
the group, and re-reads the `$`. `tex_math.rs::end_math_or_defer` (the
`before_digest` of `\lx@end@inline@math`/`\lx@end@display@math`) warns "Missing }
inserted" and, since the group is a constructor's own argument frame that its
digestion closes, defers the math end to that moment with the `\aftergroup`
mechanism; the enclosing math stays nested in its paragraph. It fires only where
`end_mode` already took its error branch (BOUND_MODE not bound on the top frame),
so every balanced `$…$`, `\mbox{$…$}`, `array`/`cases` and alignment path is
untouched. The plain-brace shape `${\hat m$}` (an error for pdflatex too) gets
the same warning instead of Perl's error.
**Why**: TeX's own recovery; one warning instead of a cascade; well-formed tree.
**Witnesses**: `tools/perfect_kernel/repros/boxes-groups/mal_math_bm_group_close.tex`
(and `mal_math_brace_close_plain.tex`, the malformed control); kblocks-doc 4 → 0,
titlecaps 10 → 3, then 10 → 0 with the re-defer below.
**Guard**: `perfect_kernel_batch56::math_end_inside_open_group_defers_to_group_end`.
**Upstream**: not filed.
**Refinement (batch 56u)**: the deferred `@atgroup` twin (`tex_math.rs::end_math_at_group`)
re-defers once more when the frame on top at the group's end is a REAL TeX group
(`groupInitiator` = `{`, `\bgroup`, `\begingroup` — the groups off_save would
close), and is a no-op when its math has already closed (a second `$` deferred in
the same group is TeX's re-opener). A constructor's bounded frame is not a TeX
group: there the benign "Attempt to end mode" stays (nicefrac, egpeirce). Guard:
`perfect_kernel_batch56::deferred_math_end_walks_real_groups`.

### 197. `TeXFileName` fully expands the filename tokens (Perl expands with `readXToken(0)`)

**Perl behavior**: Base_ParameterTypes.pool.ltxml:296-306 reads a filename with
`readXToken(0)`: macros expand, but a `\protected` macro stays unexpanded, so
`\openout\file=\pratendGeneratePrefixFile pratenddefaultcategory.tex`
(proof-at-the-end.sty:112/127, a protected `\NewDocumentCommand`) writes to a
file literally named `\pratendGeneratePrefixFile…` and the later
`\input{\pratendGeneratePrefixFile pratenddefaultcategory.tex}` cannot find it
(`Error:missing_file`; pdflatex, which expands with `\scan_file_name`'s full
expansion in the `\openout` filename scan, is clean).
**Rust behavior**: `base_parameter_types.rs` `TeXFileName` reads with
`read_x_token(…, Some(true))`, expanding protected macros too (tex.web §526
`scan_file_name` expands the filename with `get_x_token`, which is why TeX itself
sees the expansion). That covers an unbraced name. A braced name (`\input{…}`,
`\tex_input:D {…}`) follows TeX Live's braced scan (`scan_toks(false,true)`): it
stops at the matching `}`, keeps its spaces and a `\protected` macro, and keeps
its braces, which `\input` strips (batch 56is, KNOWN_PERL_ERRORS #256). The
witness below is unaffected: LaTeX's `\input{…}` goes through `\ltx@input`'s
`\csname` expansion (sect11.rs), not `TeXFileName`.
**Why**: TeX's own filename scan; a same-run write-then-read of a generated file
(the K7/VFS lane) depends on it.
**Witnesses**: proof-at-the-end/proof-at-the-end_demo (2 → 0).
**Guard**: `perfect_kernel_gemini::openout_then_input_same_run`.
**Upstream**: not filed.

### 198. Beamer frame bodies halve `#` twice but tolerate a lone `#` (real beamer errors)

**Perl behavior**: beamer.cls.ltxml:875 `readFrameBody` digests a `{frame}` body
directly, so `####1` written for a `\tikzset`/`\newcommand` inside a frame
(beamer-theme-albi-doc.tex:505) reaches `\def` as four parameter tokens and
errors (`Illegal parameter number`, 40 errors; pdflatex clean).
**Rust behavior**: `beamer_cls.rs::collect_frame_body` collects the body of a
non-fragile frame to the matching `\end{frame}` and `halve_frame_hashes` halves
`##` → `#` twice before replaying it, the way real beamer's default path does:
`\beamer@doseveralframes` (beamerbaseframe.sty:469/518) puts the body through
`\loop`'s `\def\iterate{…}` and then `\def\beamer@doifinframe{…}` (:527), two
replacement-text scans. Fragile frames skip the halving (they are written to a
`.vrb` file and re-read, `\beamer@doexternalframe` :476/553).
**Leniency**: at either `\def` level real TeX rejects a `#` not followed by `#`
or a digit ≤ 0, so a frame body must write `##`/`####`; Rust keeps a lone `#`
intact (`#1` stays `#1`), so the many existing manuals that write
`\newcommand\x[1]{#1}` inside a frame keep working. `containsverbatim` frames
(`\beamer@dosingleframe`, no halving in beamer) are equivalent under the
leniency.
**Why**: the `\def`-shaped collect is how beamer really processes a frame;
the leniency is a strict superset of the legal inputs.
**Witnesses**: beamer-theme-albi/beamer-theme-albi-doc (40 → 0).
**Guards**: `perfect_kernel_gemini::beamer_frame_hash_halving`,
`perfect_kernel_batch56::beamer_frame_single_hash_control`.
**Upstream**: not filed.

### 199. The kernel `{verbatim}` hands `\end{verbatim}` back to the current `\end` macro

**Perl behavior**: latex_constructs.pool.ltxml:1734 `\begin{verbatim}` is a
fused-cs constructor that reads the raw body AND swallows the terminating
`\end{verbatim}`, so a package that hooks `\begin`/`\end` sees the begin (the
fused cs is only reached from the `\begin` macro) but never the end.
knowledge.sty:1671-1685 pushes a scope area at `\begin{verbatim}` and pops it
at `\end{verbatim}`; Perl is nevertheless clean because it never defines
`\verbatim`/`\endverbatim`, and `\KnowledgeConfigureEnvironment?` (:3754) is
gated on both existing, so the area is never registered there.
**Rust behavior**: the dumped latex.ltx defines both (:15459-15460), the area
is registered, and the swallowed terminator left the stack as
`[verbatim::document]` at `\end{document}` (`Not allowed to close 'document'
here (at 'verbatim')`). `after_digest_verbatim` now unreads
`\end{verbatim}` (or `\end{verbatim*}`) ahead of the rest of the line so the
CURRENT `\end` macro runs, as latex.ltx:15438 `\def\@xverbatim#1\end{verbatim}
{#1\end{verbatim}}` does; the fused `\end{verbatim}` cs is a no-op macro so
the kernel `\end` adds nothing (the constructor already closed its group).
`\AfterEndEnvironment{verbatim}` hooks (`@environment@verbatim@afterend`) now
fire too, as in LaTeX.
**Why**: the terminator re-execution is LaTeX's own shape; every
`\begin`/`\end` hook pair stays balanced.
**Witnesses**: knowledge/knowledge (1 → 0). Golden re-baselined:
`tests/expansion/etoolbox.xml` gains the `\AfterEndEnvironment{verbatim}` line
"After End verbatim(outside)." after the verbatim — pdflatex prints it there;
Perl's golden (t/expansion/etoolbox.xml:70) lacks it.
**Guard**: `perfect_kernel_batch56::package_state_prtec_psfragx_knowledge`
(hooked-`\end` control: the hook fires once, after `</ltx:verbatim>`, and the
`\end` line's tail still reads).
**Upstream**: not filed.

### 200. `read_match` keeps a delimiter's adjacent space tokens (Perl always skips the run)

**Perl behavior**: Gullet.pm:604-620 `readMatch` shifts the next to-match
token and, when the token just matched was a space, SKIPS the whole following
space run in the input ("If this was space, SKIP any following!!!", :614) —
even when the next delimiter token is itself a space. expkv.tex:176 builds a
`\def` delimiter by `\detokenize` of control words, which yields TWO adjacent
space tokens (`…\ekv@mark  \ekv@nil…`); Perl's skip swallows the second
space of the input, the delimiter never matches, and the key loop runs off
(`\ekv@stop` undefined, then the PushbackLimit/EoF fatal; Perl 101 errors +
fatal on the same witness).
**Rust behavior**: `gullet.rs::read_match` skips the space run only when the
next token to match is NOT a space (Gemini round 3, 6d7501762d). tex.web §392
`macro_call` matches delimiter tokens one by one with no skipping, so a
two-space delimiter matches two input spaces there; pdflatex is clean.
**Why**: the change is behaviour-preserving except for delimiters with ≥2
adjacent spaces, where it follows TeX; every other case (single space + a
non-space, trailing space, surplus input spaces) is byte-identical to the
Perl port.
**Witnesses**: expkv-bundle/expkv (2 errors + fatal → 13 errors, no fatal: the
loop terminates and the document runs through to its next classes).
**Guards**: `perfect_kernel_gemini::expkv_ekvcsvloop_delimiter_adjacent_spaces`
(the two-space delimiter) and its control (a single-space delimiter still
collapses surplus input spaces, the Perl behaviour).
**Upstream**: not filed.

### 201. amsrefs `\bib` titles keep their case (Perl re-cases them like BibTeX)

**Perl behavior**: amsrefs entries go through the BibTeX pool, whose default
`BibTeX_title_case = capitalize1` (BibTeX.pool.ltxml:293-332) lowercases every
word after the first — including control-sequence NAMES, so `\AmS` becomes
`\ams` and `\LaTeX` `\latex` (undefined: amslatex-primer/amshelp, 3 errors).
amsrefs.sty.ltxml never overrides it.
**Rust behavior**: `amsrefs_sty.rs` sets `BibTeX_title_case = asis`. amsrefs
typesets `\bib` titles verbatim (pdflatex prints "Using AMS-LaTeX and Other
Tools Well", verified 2026-09-06); documents without amsrefs keep the BibTeX
`.bib` default `capitalize1` (the value is set at amsrefs load, so a document
using both gets `asis` for its `.bib` too).
**Goldens re-baselined**: `tests/structure/amsrefs_basic.xml` (titles "On
Examples", "A Book" keep their case) and the `06_cluster_bibliography` amsrefs
guard.
**Witnesses**: amslatex-primer/amshelp (3 → 0).
**Guard**: `perfect_kernel_batch56::sweep46_single_name_gaps` (amsrefs part).
**Upstream**: not filed.

### 202. An environment's closing tag closes only what its own replacement opened

**Perl behavior**: a constructor's `</ltx:X>` is a blunt `closeElement`
(Document.pm:803); when the element's content could not be held and auto-closed
it — a tcolorbox placed directly in a `{picture}` (pagelayout.cls:3656,
xebaposter), a `\subsection` inside a beamer `{block}` (`</ltx:theorem>`,
beamertheme-mirage), `\lxSVG@insertpicture`'s `svg:svg`/`ltx:picture` closes
(pgfsys-latexml.def.ltxml:106-107) — the close reports "Attempt to close
</ltx:X>, which isn't open" (pdflatex clean).
**Rust behavior**: a `CloseElement` op whose element was opened by an
`OpenElement` op of the same replacement runs `Document::close_element_if_open`
— the strict close (force-closing intervening descendants) whenever the element
is still on the ancestor chain, a no-op only when it is already gone (runtime
interpreter `replacement.rs::exec_ops` and the codegen `emit_ops`); closes
of elements the replacement did not open stay strict. A genuine
`\begin{a}…\end{b}` mismatch is caught by the environment-name check
before the closing tag runs, so nothing real is hidden. Witnesses:
pagelayout/example-template, example-text, xebaposter/poster,
beamertheme-mirage/mirage-poster-{en,zh}. Guard:
`perfect_kernel_batch56::environment_close_after_content_autoclose_is_not_an_error`.

### 203. fontenc loads afresh on every request (Perl loads its binding once)

**Perl behavior**: fontenc.sty.ltxml is loaded once (Package.pm:2328); the
second `\usepackage[<encs>]{fontenc}` is an option-clash Info and the new
encodings' `.def` files never load (montex: ctib.sty `[LCT,T1]` then
mls.sty:416 `[LMS,LMO,LMA,LMC,T1]` — `\MyTogrog` from lmcenc.def undefined).
**Rust behavior**: the real fontenc.sty un-marks itself at its end
(`\global\let\ver@fontenc.sty\relax`, `\let\opt@fontenc.sty\relax`), so
every request loads it again with the new options and `\@ifpackageloaded
{fontenc}` is false. `fontenc_sty.rs` declares `fontenc.sty_unmarks_itself`;
the loader (`content.rs` `already_handled`, `_load_binding`, the option-clash
check, `\ver@` relax) and `\@ifl@aded` honour it. Guard:
`perfect_kernel_batch56::fontenc_reloads_with_new_encodings`.

### 204. `\ifpdf` follows the output mode (Perl: static FALSE)

Extends #168: iftex.sty:272-291 answers `\ifpdf` TRUE on LuaTeX whenever the
output mode is PDF — always, LuaTeX's default — so a lualatex-oracle document
takes the PDF branch (tikzrput.sty defines `\rput` only there: pgfornament
ornaments/tikzrput). On pdfTeX it is `\ifnum\pdfoutput>0` (:290-291), and under
the `xetex` profile it is FALSE (XeTeX has no `\pdfoutput`). Since batch 56id
(the K6 ruling, #285) the pdfTeX persona reads `\pdfoutput` too, whose default
is set per document. The value is read when `\ifpdf` is tested, not frozen when
iftex loads as the real package does: our format loads iftex before a
document's own `\pdfoutput=1`. The one observable difference is a document that
changes `\pdfoutput` after loading iftex (real `\ifpdf` keeps the load-time
answer); `\pdftrue`/`\pdffalse` still override. Guards:
`perfect_kernel_batch56::{ifpdf_is_true_under_the_luatex_profile,
ifpdf_delegates_to_iftex}`, `class_census::ifpdf_follows_pdfoutput`.

### 205. A box keeps what it can hold; the rest is placed after it (`insertBlock`)

**Perl behavior**: `insertBlock` (TeX_Box.pool.ltxml:406-472) renames the
captured box content to the first candidate container that can hold all of it
(`inline-block`/`inline-logical-block`/`inline-sectional-block` in an inline
context, `block`/`logical-block`/`sectional-block`/`figure` otherwise) and, when
none can, to a hard `ltx:block` — after which every child the model rejects is
reported ("`<ltx:caption>` isn't allowed in `<ltx:block>`": a `\caption` inside
`\rotatebox{\parbox{…}}`, heria-proposal; `\printbibliography` inside mdframed,
biblatex-juradiss; pdflatex clean in each).
**Rust behavior**: the same container choice first; when no candidate holds
everything, the first candidate that holds the box's leading content is the
box, and the content it cannot hold is placed after the box in the nearest
FLOW ancestor (one that can hold a paragraph) that can hold it, with the
insertion point following — the outcome an autoclosing frame produces for the
same content written live. The climb never leaves a flow container (a drawing
or math box is another medium; hoisting past it strands the rest of the box),
where the model's verdict stands as in Perl. One rule, no per-tag cases
(`latexml_engine/src/base_utilities.rs::insert_block`). Guards:
`perfect_kernel_gemini::mdframed_block_bibliography_juradiss`,
`perfect_kernel_batch56::caption_in_inline_parbox_floats_to_figure`,
`tests/complex/figure_dual_caption` (a minipage of graphics + caption still
becomes the figure).



### 206. Array cells follow `\@classz`; `\endtabular` closes a raw scaffold; templates `\edef`-expand

**Perl behavior**: `\@array@bindings` always binds MATH cells; `\@tabarray` is
`\m@th\@@array[c]`; `\endtabular` = `\@tabular@after\lx@end@alignment\@end@tabular`
(no `$\egroup`); `ReadAlignmentTemplate` (Alignment.pm:895-921) reads the template
unexpanded.
**Rust behavior**: `\@arrayclassz`/`\@tabclassz` exist (latex.ltx:16645/16652) and
`\@array@bindings` + `\@array` pick text cells and the tabular container when
`\ifx\@classz\@tabclassz` (latex.ltx:16561 `\@tabular`), math cells otherwise
(:16550 `\array`); `\@array@bindings` records `lx@raw@array@open` (scoped to the
scaffold group, cleared by the constructor's `\@tabular@bindings`) and
`\endtabular` appends latex.ltx:16554's `$\egroup` exactly when set; the template
reader expands every expandable token as `\@mkpream`'s `\edef` does
(latex.ltx:16610-16644) except `\csname`/`\expandafter`/`\noexpand` (the over-read
protection nicematrix's `V{3cm}`-before-binding needs).
**Why**: one latex.ltx discriminator instead of a forced mode; binding-less
deluxetable copies (aguplus, sgame, mdwtab, fcolumn, plarray, tabularborder) close
balanced. KNOWN_PERL_ERRORS #202.
**Witnesses**: aguplus/aguplus (11 → 0), t-angles (unchanged, math path).
**Guards**: `perfect_kernel_batch56::{tabarray_cell_mode_follows_classz,
raw_tabular_scaffold_closes_and_template_expands}`, `perfect_kernel_batch54::
tabarray_is_the_full_array_setup`.

### 207. Groups still open at `\end{document}` are abandoned §1335-style

**Perl behavior**: `\end{document}` pops the open undo frames back to the document
frame (latex_constructs.pool.ltxml:350-374) with one warning; `boxing` and any
bounded primitive still digesting are left inconsistent, so an unbalanced
`\abstract{…` cascades into mode-close and malformed-XML errors.
**Rust behavior**: the same unwinding pops STACK frames (frame + `boxing`), discards
their `\aftergroup` lists (tex.web §1335 never runs them), reports "(\end occurred
inside a group at level N)" for plain groups, and sets `lx@end@document@open@groups`;
the `ltx:document` close is then lenient (`close_element_lenient`: descendants the
abandoned groups left open close with a Warning, not an Error); a `bounded => true`
primitive closes only the frame it opened (the opener writes its cs as the frame's group code, `lx@group@code`, as tex.web §274 `new_save_level` does; the closer checks it as §1068 checks `cur_group`), so one unwound by
`\end{document}` does not close the document frame; and
because the deferred body then carries the `\end{document}` whatsit — absorbed
when the frontmatter is inserted at finalization, which is when the document
actually closes — that constructor is idempotent (`lx@document@closed`) and a
frontmatter entry's own close is scoped (`close_element_if_open`, #202).
**Why**: pdflatex treats the unclosed group as a warning; the content is typeset.
A `\section` inside the open group ENDS the abstract as in the brace-less form:
the braced branch arms `\@startsection@hook` too, and because a `{` group is a
NESTED digest body (`tex_box.rs`'s `{` primitive; Perl nests the same way,
TeX_Box.pool.ltxml:30-41) the hook's `\lx@end@abstract` arrives inside that
nested body where the until-loop cannot see it — so the terminal primitive itself
acts (`until_terminal_inside_group`, stomach.rs): it abandons the groups opened
inside the body (§1335 shape, one Warning — LaTeX's `\@checkend` "\begin{X}
ended by \end{Y}", latex.ltx:15394, is the analogue) and flags the body's loop
(`lx@until@terminal@hit`), so the section lands at document level (batch 56aa).
KNOWN_PERL_ERRORS #203.
**Witnesses**: screenplay-pkg/screenplay-pkg (6 → 0).
**Guards**: `perfect_kernel_batch56::unbalanced_abstract_brace_unwinds_at_end`,
`perfect_kernel_batch54::braced_abstract_reads_its_body_incrementally`.

### 208. A locked `\abstract` honours the class's argument-taking signature

**Perl behavior**: `\abstract{…}` → `\lx@add@abstract[]{}` always (deferred, argument
form); a class's `\newcommand{\abstract}[1]` is dropped by the lock with no trace.
**Rust behavior**: the lock records the dropped definition's parameter count
(`<cs>:redefined@nargs`, from `\newcommand`'s locked bail in sect08.rs and from a
dropped raw `\def` in state.rs). `\abstract{…}` reads the group as ONE stored
argument routed to the deferred `\lx@add@abstract` when the class declared
`\abstract` with ≥1 argument (ryethesis.cls:344, apa7.cls:785, uwthesis.cls:805 —
~25 TL classes, all store-then-use at `\maketitle`/`\frontmatter`), and keeps the
incremental env-begin path (#P74, char-list) otherwise — the meaning of `\abstract`
at the call site, which is what real TeX keys on.
**Why**: `\abstract{\Gls{LI}…}` before `\newacronym{LI}` (ryethesis ryesample) is
valid under the class's own `\abstract`; a one-path capture cannot serve both the
live-catcode and the deferred cases (settled dead end: undigested capture snapshots
catcodes, so an inner `\makeatletter` breaks `\patch@level`).
**Witnesses**: ryethesis/ryesample (1 → 0; Perl 0 on the reduction).
**Guards**: `perfect_kernel_batch56::class_redefined_abstract_defers_its_argument`.
### 209. `\multicolumn` is locked like `\tabular`

**Perl behavior**: `\tabular`/`\endtabular` locked (latex_constructs.pool.ltxml:3667/
3671), `\multicolumn` unlocked (:3702): a raw package's latex.ltx-shaped
`\def\multicolumn` replaces the alignment binding and runs `\@mkpream`, which
needs the raw scaffold's `\@classz`/`\@acol` (KNOWN_PERL_ERRORS #205).
**Rust behavior**: `\multicolumn` carries `:locked`; a raw `\def` is dropped (the
lock records it as `\multicolumn:redefined`), a binding's redefinition (loaded
unlocked) wins. The rule: commands that ARE the alignment model's structure
(`\tabular`, `\endtabular`, `\multicolumn`) are locked; raw copies of latex.ltx's
`\halign`-preamble machinery cannot run against a model that has no preamble.
**Why**: the cell keeps its column and content (aguplus 2 → 0).
**Witnesses**: aguplus/aguplus.
**Guards**: `perfect_kernel_batch56::raw_multicolumn_redefinition_is_dropped`.

### 211. The alignment cell-head peek runs in internal vertical mode

**Perl behavior**: the whole `\halign` body, including the per-column peek that
expands the cell-head token, is digested in restricted_horizontal
(TeX_Tables.pool.ltxml:181/376).
**Rust behavior**: the peek (and `\noalign` material) runs under
`internal_vertical`, the cell body in restricted_horizontal — tex.web §15350/
§15510/§15532's mode split (`AlignPeekMode`, tex_tables.rs).
**Why**: a mode-conditional cell head (`\ifhmode\else\expandafter\hbox\fi…`, the
abntexto/OpTeX `<…>` idiom) keeps its box. KNOWN_PERL_ERRORS #206.
**Witnesses**: abntexto/abntexto (8 → 0).
**Guards**: `perfect_kernel_batch56::alignment_cell_head_peeks_in_internal_vertical_mode`.

### 212. A pgf graphics-state scope is an `<svg:g>`, not a TeX group

**Perl behavior**: pgfsys-latexml.def.ltxml:576-586 `\pgfsys@beginscope`/
`\pgfsys@endscope` = `\begingroup`/`\endgroup` plus the `<svg:g>`.
**Rust behavior**: the `<svg:g _scopebegin>` open/close only, as the pdf driver's
`q`/`Q` literals open no TeX group (pgfsys-common-pdf.def:37-38); pgf's own
`{pgfscope}` supplies the TeX grouping it needs (pgfcorescopes.code.tex:48-61).
**Why**: scopes straddling pgf's node boxes no longer interleave with box frames.
KNOWN_PERL_ERRORS #207.
**Witnesses**: `tools/perfect_kernel/repros/graphics-tikz/pgfsys_scope_straddle_hbox.tex`,
an empty `{msc}` chart; msc/msc and modernposter/demo improve (24→21, 15→14) but stay
open on the fused mode-frame family (DIFFICULT_CASES D12).
**Guards**: `perfect_kernel_batch56::pgf_scope_straddling_a_box_is_not_a_tex_group`.

### 213. `\eqno`/`\leqno` digest their tag

**Perl behavior**: TeX_Math.pool.ltxml:1239 collects the tag tokens with
`readXToken` up to a terminator net, then digests them as the constructor's
argument.
**Rust behavior**: `\lx@eqno EqnoTag` digests the tag as a bounded math sub-body
(tex.web §21745 `start_eq_no`), stopping before the display-end token and
retracting it; assignments in the tag execute in place.
**Why**: mhequ's `\@restoreMHComms` after `\eqno{…}`. KNOWN_PERL_ERRORS #208.
**Witnesses**: mhequ/mhequ-example.
**Guards**: `perfect_kernel_batch56::eqno_digests_its_tag_material`.

### 214. algpseudocodex.sty package binding

**Background.** `algpseudocodex.sty` extends `algorithmicx` / `algpseudocode` with indentation guides, single-line right-flushed comments, multiline `\LComment`s, and boxed algorithmic code blocks (`\BeginBox`, `\EndBox`, `\BoxedString`). On `perfect_kernel`, raw loading already renders `\Comment` on one line (`float:right`) with 0 Fatal; the binding provides full semantic support for `\LComment`, `\BeginBox`, `\EndBox`, and `\BoxedString` (which raise undefined control sequence errors under raw loading) and implements clean in-flow box wrappers.

**Perl behavior**: SHARED failure — Perl LaTeXML has no binding for `algpseudocodex.sty`.

**Rust behavior**: `latexml_contrib::algpseudocodex_sty` provides a clean binding:
1. Loads `algpseudocode` (and `algorithmicx`).
2. Honors package options `italicComments` (default `true`), `rightComments` (default `true`), `noEnd` (default `true`, suppressing `\EndWhile`, `\EndIf`, etc.), `commentColor` (default `gray`), and comment delimiter options `beginComment`, `endComment`, `beginLComment`, `endLComment`. Drops visual-only TikZ layout options (`indLines`, `spaceRequire`).
3. Supports `\Comment` and `\LComment` with configurable formatting and delimiters, placing single-line comments in-flow within the line's `<listingline>` and multiline `\LComment` on its own `<listingline>`. Defensively provides stubs if old `algorithmic.sty` was loaded first.
4. Preserves `\Statex` in-line break semantics (`<break/>` within the open line box) matching raw TeX varwidth-in-open-box execution.
5. Implements `\BeginBox`, `\EndBox`, and `\BoxedString`, translating box options (`draw=<color>`, `dashed`, `dotted`, `thick`, etc.) to structured CSS styles and classes on `<ltx:text class='ltx_framed ltx_algpx_boxed ...'>`, opening only on the pending→open transition so multi-line boxes do not double-open.
6. Declares dummy counters, lengths, and stubs for internal TikZ coordinate macros.

**Why it's safe.** Surpasses Perl on unbound package, eliminates undefined CS errors on `\LComment`/`\BeginBox`/`\EndBox`/`\BoxedString`, and preserves exact line and break structure.

**Witnesses**: arXiv 2511.21969. Guarded by `algpseudocodex_produces_clean_comments_and_boxes` and `statex_continues_the_open_line_box` in the `cluster_package_guards` binary.

### 215. The save-frame bookkeeping is exempt from `\globaldefs`

**Perl behavior**: State.pm:144-151 applies `\globaldefs` to every
assignment, including `pushStackFrame`/`beginMode`'s frame records, so a group
closed under `\globaldefs=1` leaves its records behind.
**Rust behavior**: the frame records (`groupInitiator`, `groupNonBoxing`,
`lx@frame@id`, `BOUND_MODE`/`MODE`/`INNER_BOX` at a mode switch, an
environment's `lx@group@code`, the alignment cell group, an until-body's
terminal record) bind through `assign_local_unconditional`, local to their
frame whatever `\globaldefs` or a `\global` prefix says.
**Why**: tex.web §1214 (`\globaldefs` only flags `prefixed_command`'s
assignments) vs §274/§282 (the save stack). KNOWN_PERL_ERRORS #209.
**Witnesses**: msc/msc (21 → 0, `repros/graphics-tikz/msc_declinst_tikz_scope_desync.tex`).
**Guards**: `perfect_kernel_batch56::globaldefs_does_not_globalize_the_save_stack`.

### 216. `\pgfmathpointintersectionoflineandarc` is closed form

**Perl behavior**: the raw pgf bisection runs over Perl's float trig and never
meets its exact-equality exit (KNOWN_PERL_ERRORS #210).
**Rust behavior**: a binding solves the ray/ellipse quadratic and returns the
parametric angle of the root on the arc (nearest to it otherwise — pgf's own
"best estimate" `\n`), then pgf's `\pgfpointadd{center}{\pgfpointpolar…}` as
before.
**Why**: the point set is the bisection's limit; no loop, no dependence on
trig self-consistency. Sub-point border coordinates of rounded-rectangle and
callout nodes may move from the bisection's approximation to the exact point.
**Witnesses**: zx-calculus/zx-calculus (Fatal → 0), arXiv 2201.09268 class.
**Guards**: `perfect_kernel_batch56::line_and_arc_intersection_is_closed_form`.

### 217. The document hooks run inline in the galley

**Perl behavior**: `\begin{document}`/`\end{document}` digest the hook stores
in isolated string mouths (latex_constructs.pool.ltxml).
**Rust behavior**: `begindocument/end` + etoolbox's `\AfterEndPreamble` store
are unread onto the main stream at the end of `\begin{document}`;
`\end{document}` expands to the `\AtEndDocument` list + the `enddocument`
hook + `\lx@finalize@document` (the `\@checkend`/`final_cleanup` half).
**Why**: latex.ltx:15255-15259 and etoolbox.sty:1774-1776 run them inline, so
a box or environment spanning the body reads the body as its contents.
KNOWN_PERL_ERRORS #211; extends the opener-side unread of batch 54.
**Witnesses**: modernposter/demo (14 → 0),
`repros/boxes-groups/atenddocument_hbox_egroup.tex`.
**Guards**: `perfect_kernel_batch56::atenddocument_closer_reaches_the_galley_box`.


### 218. The pst-grad binding loads `pst-grad.tex`

**Perl behavior**: `pst-grad.sty.ltxml` is `RequirePackage('pstricks')` and
nothing else, so `\psfs@gradient` and the `grad*` keys never exist.
**Rust behavior**: `pst_grad_sty.rs` additionally raw-loads `pst-grad.tex`
(pst-grad.sty:3), exactly what the wrapper does.
**Why**: with `\pst@object` dispatching for real (batch 56af) a raw object's
bracket `[fillstyle=gradient,…,addfillstyle=boxfill]` reaches pstricks.tex's
`fillstyle`/`addfillstyle` keys (:1333-1350): the first raised "Undefined fill
style" and left `\psk@fillstyle` at its `\let…\relax` default
(pstricks.tex:1337); the second's `\expandafter\noexpand\psk@fillstyle`
then froze that unexpandable NAME into the new definition — a
self-recursion at the next use (TeX loops the same way; the difference is
only that pdflatex has `\psfs@gradient`). The gradient fill styles are PostScript
strings the renderer ignores; the key surface is what matters.
**Witnesses**: arabi/big2 (0 → 2 after 56af → 0).
**Guards**: `perfect_kernel_batch56::pst_grad_binding_loads_the_gradient_fill_styles`.

### 219. The empty xkeyval family is searched and named by the header rule

**Perl behavior**: `KeyVals->new` drops empty keyset entries (KeyVals.pm:52)
and `keyval_qname` always doubles the `@` (KeyVal.pm:60-62), so keys defined
with `\define@key[psset]{}{…}` are unreachable.
**Rust behavior**: `keyval_qname` follows xkeyval.tex:83-88 (`\XKV@makehd`):
`<prefix>@<key>` for the empty family, `<prefix>@<family>@<key>` otherwise;
`KeyVals::new` keeps explicit empty entries and reserves `_anonymous_` for an
absent list.
**Why**: pst-xkey builds `\pst@famlist` as `,pstricks` so that `\psset` searches
the empty family first; pstricks.tex:808-810 (`precode`, `postcode`,
`exchange`) and pst-node.tex (`Xnodesep`… six keys) live there. The header rule
is also what keeps `\psset@@dash`-style delimited helpers from colliding.
KNOWN_PERL_ERRORS #212.
**Witnesses**: every pstricks load ("unknown KeyVals key 'precode'" ×3 warnings
gone); pst-node `Xnodesep` users.
**Guards**: `perfect_kernel_batch56::xkeyval_empty_family_is_searched_by_psset`,
`keyval::tests::keyval_qname_normalizes_empty_prefix`,
`keyvals::tests::keyvals_new_keeps_the_empty_family`.

### 220. A terminal `\read` below scroll mode halts the job (tex.web §484)

**Perl behavior**: `\batchmode`/`\nonstopmode`/`\scrollmode`/`\errorstopmode`
are no-ops and a `\read` from an unopened stream does nothing.
**Rust behavior**: the modes set a global interaction level (tex.web §71,
§1265); a `\read` from a stream that is not open is a Fatal when the level is
batch or nonstop, and stays a no-op in scroll/errorstop mode (no terminal).
iftex's `\Require<engine>` and `ifxetex` are the real definitions
(iftex.sty:42-52, 78-91; ifxetex.sty:4-5 requires iftex).
**Why**: `\batchmode\read -1 to …` is THE halt idiom of iftex and expl3's
`\msg_fatal`; without it a wrong-engine package keeps loading and loops on
undefined engine primitives (bidi.sty:354 on `\XeTeXcharclass`) until memory
runs out. KNOWN_PERL_ERRORS #213. Does not define any XeTeX primitive: those
names are engine-detection probes (the XeTeX analogue of the parked
`\directlua` family). Collateral by design: a LuaTeX-only package's
`\RequireLuaTeX` (cartonaugh, mandi, luaprogtable, luamathalign, junicodevf)
halts under the pdfTeX profile exactly as pdflatex would; lualatex-oracle
documents are converted under the `luatex` profile, where it passes
(cartonaugh-example: 0 errors).
**Witnesses**: the 23-doc `alloc_failed 3288334336` cluster of sweep 57
(handout, latexbangla, lshort-persian, texnegar-xetex-bidi-leaders-hrule…);
`repros/backend-persona/xetex_only_package_batchmode_read_halt.tex`.
**Guards**: `perfect_kernel_batch56::batchmode_terminal_read_halts_the_job`.
**Update (batch 56hr, user direction 2026-09-24)**: the default persona converts XeLaTeX
documents. The input is Unicode-native, so what XeTeX gives a document is already there:
UTF-8 text, and OpenType fonts and script switching via the fontspec/polyglossia bindings.
`\RequireXeTeX` and `\RequireTUTeX` therefore pass, as in Perl. The loop the halt guarded
against is fixed at its source: `\RequireXeTeX` installs XeTeX's inter-character
primitives (`\XeTeXcharclass`, `\XeTeXinterchartoks`, `\XeTeXinterchartokenstate`, and
latex.ltx:22025's `\newXeTeXintercharclass`) as argument-reading no-ops. They are
installed only once some package requests XeTeX, because amsmath, mathastext, minted2 and
others probe `\XeTeXcharclass` to detect XeTeX. Only the setter forms are emulated, not
`\XeTeXcharclass <n>` as a getter (zxjatype's `\int_compare`), and two TL packages drive the
primitives without calling `\RequireXeTeX` (unicode-bidi.sty:20-21, xetexko.sty:55-57), so
they remain exposed to the loop. bidi, xeCJK and mathspec, whose raw files check
the engine themselves or drive those primitives directly, get bindings that keep content
and drop the font and direction switches. arXiv 2605: 11 halting papers now convert, 10 of
them with 0 errors. TL manuals: 18 of the 28 halting ones now convert, latexbangla clean
where it previously hit the 6.3 GB `alloc_failed`. The other 10 fail as before, in
xepersian's and quran's own expl3 engine checks under the luatex profile, and in stex-doc's
missing archive. `\Require…` for an engine the persona is not (LuaTeX, pTeX, upTeX, VTeX …) still halts;
`\RequirePDFTeX`/`\RequireeTeX` pass as before.
Guard `perfect_kernel_batch56::xetex_only_packages_load_under_the_default_persona`.

### 221. `\@roman` & co. are latex.ltx token macros; a `{Type}` re-parse is isolated

**Perl behavior**: `\@arabic`/`\@roman`/`\@Roman`/`\@alph`/`\@Alph`/`\@fnsymbol`
are `{Number}` closures (latex_constructs.pool.ltxml:3053ff): one braced-or-single
token is read, then re-parsed as a number in a fresh mouth. When that token is
`\the` (texmate.sty:546 `\csname chessdiag\@roman\the\@diagramsbuilt\endcsname`),
the re-parse mouth runs dry and Perl's `readXToken` falls THROUGH to the outer
gullet, so it reads `\@diagramsbuilt` by accident and prints the right numeral.
**Rust behavior**: the six are latex.ltx:10206-10224's own bodies (`\number #1`,
`\romannumeral #1`, `\expandafter\@slowromancap\romannumeral #1@`, the `\ifcase`
alphabets with `\@ctrerr`, and `\@fnsymbol` with `\TextOrMath`); `\@slowromancap` is
defined (it was not). A `{Type}` re-parse stays isolated (`reading_from_mouth`).
**Why**: `#1` is one token and the NUMBER is scanned by the primitive from the
continuing stream — the kernel's own semantics, expandable inside `\csname`/`\edef`,
and serializable into the dump (CLAUDE rule 3). The isolated re-parse read `\the`
alone, errored "A <register> was supposed to be here", and the leaked count ended
the name scan early ("Extra \endcsname" ×33, texmate2manual 100-error storm).
Collateral by design: `\@alph{27}` is "Counter too large" as in LaTeX (the closure
extended silently). Guard `at_roman_family_expands_inside_csname`. Batch 56av.

### 222. An index entry is the `.idx` string `\printindex` re-reads

**Perl behavior**: `SanitizedVerbatim` (latex_constructs.pool.ltxml:4376-4394)
re-tokenizes the sanitized entry with `TokenizeInternal` — style catcodes, `@` a
LETTER — so `\index{\AB@\relax…}` (manyind mindsample.tex:208) is the one
undefined name `\AB@`; and every control word in the phrase, including an
undefined sort key `\A`, is digested (mindsample: 15 errors).
**Rust behavior**: the same style re-tokenize, but AFTER the `\protected@write`
expansion (`process_index_phrases`) an UNDEFINED control word still carrying an `@` is
re-read with the document's table (`@` OTHER) — the `.ind` is `\input` by
`\printindex` in the document body — so `\AB@` is the key `\AB` plus the
separator, while names the expansion consumes (tcolorbox's
`\kvtcb@doc@sortindex\idx@actual`) and defined survivors (robust commands, the
`\@internal@text@verb` marker) are untouched. The pre-expansion
(`process_index_phrases`) passes an undefined control word inert with
`\noexpand`, so the entry digests in order (manyind.sty:100/119's
`\protect\def \nwletre {…}…\nwletre` defines the name before its use); an
undefined control word in the SORT KEY is its literal characters — makeindex
sorts on the string `\AB`, it never typesets it.
**Why**: the entry is out-of-band text in real LaTeX (`\@sanitize`, `\write`,
makeindex, `\input`); the kernel emulates that pipeline in one pass, and the
catcodes of the re-read are the document's. Witness: manyind/mindsample
(oracle pdflatex-clean) 3 → 0. Guards `index_entry_passes_undefined_words_inert`,
`index_sort_key_undefined_word_is_text`; KNOWN_PERL_ERRORS #83 sibling. Batch 56bp.

### 223. A class-cleared option handler is undefined for a package's default option

**Perl behavior**: `Package.pm:2481-2486` tests `lookupDefinition('\ds@'.$option)`,
which returns the `\relax` primitive a class's `\ProcessOptions` left behind
(`\let\ds@landscape\relax`, latex.ltx:18582), so `\usepackage[landscape,a4paper]
{geometry}` runs `\relax` and geometry's `\DeclareOption*` default never sees the
option.
**Rust behavior**: `execute_option_internal` treats a handler `\let` to `\relax` as
undefined (`x_equals(cs, \relax)`), as latex.ltx:18589 `\@use@ption`'s
`\@ifundefined{ds@\CurrentOption}` does, so the package default runs. The option
value and `\CurrentOption` are the argument tokens (standard catcodes, braces
grouping; `option_argument_tokens`), not an all-OTHER explode.
**Why**: the kernel's own test; a package that declares `\DeclareOption*` is meant to
receive every option the class does not claim at that moment. Witness elzcards'
`[landscape,letterpaper,vmargin={0mm,0mm}]{geometry}` (root-causer 2026-09-09).
Guards `class_cleared_option_handler_reaches_the_package_default`,
`package_option_value_keeps_its_braces`. Batch 56bq.

### 224. A delimited argument keeps each interior group's own close token

**Perl behavior**: `Gullet.pm:661/674` (`readUntil`) pushes a canonical `T_END`
after `readBalanced` consumed a group's close, whatever character carried
catcode 2 — a `(…)` group scanned with `(`/`)` given catcodes 1/2 (chemfig.tex:
1315-1324 `\CF_grabsubmol`) comes back as `(…}`, which the next `\scantokens`
re-reads as a real end-brace (chemfig two-submol molecules; abntexto-exemplo).
**Rust behavior**: `read_balanced_with_close` returns the close token and
`read_until` stores it verbatim, as `macro_call` does (tex.web §392-398
`store_new_token(cur_tok)`); the single-group outer-brace strip stays
catcode-driven (§399-400, `right_brace_limit`), so `(A)` is still stripped.
**Why**: the argument keeps the braces it was scanned with; only exotic
catcode-2 characters differ from Perl, which fails those identically (SHARED).
Witnesses chemexec/chemnum/carbohydrates (25 → 6), abntexto-exemplo (Fatal → 0).
Guard `delimited_read_keeps_a_groups_own_close_token`. Batch 56bt.

### 225. `\afterpage` runs its body in place

**Perl behavior**: afterpage.sty.ltxml:20 `DefRegister('\afterpage' => Tokens())` —
the argument is dropped.
**Rust behavior**: `\afterpage{…}` is `\gdef\lx@afterpage@body{#1}\lx@afterpage@body`
(afterpage.sty:83 stores the body by `\gdef`, halving `##` to `#`, and typesets
it after the page; a web document has no page, so the body runs at once).
Only with the package loaded — an undefined `\afterpage` stays an error, as in
LaTeX and Perl (sesamath-doc-fr relies on a missing sesamath-doc.sty; SHARED).
**Why**: the content is part of the document; dropping it loses text.
Guard `afterpage_body_undoubles_parameter_hashes`. Gemini round 10 N5, merged
with the autoload removed.


### 226. `\@preamblecmds` is `\@notprerr` inside a running document

**Perl behavior**: latex_constructs.pool.ltxml:5460 `\@preamblecmds` = `Tokens()`
forever, `:5594` `\@onlypreamble` records nothing ("Don't bother enforcing this"),
and `\@notprerr` is undefined. Raw code that probes the kernel's own signal for
"inside a running document" — lppl.tex:44 `\ifx\@preamblecmds\@notprerr` — takes the
standalone branch, `\let\endLPPLicense\enddocument`, and ends the document at the
license (beameruserguide: 16 of 26 chapters lost, zero errors; SHARED).
**Rust behavior**: `\@notprerr` is defined (latex.ltx:9019) and `\document` `\let`s
`\@preamblecmds` to it globally at the latex.ltx:9521-9522 point (`\@preamblecmds`
is itself `\@onlypreamble`, :1228), so the probe reads as it does in LaTeX.
`\@onlypreamble` itself stays the loose flag-only guard (sect01.rs), not a `\do` list.
**Why**: the observable side effect raw code tests for is part of the kernel
contract; reproducing it costs one `\let` and no per-command bookkeeping.
Guard `preamblecmds_is_notprerr_inside_a_running_document`. Batch 56bx.

### 227. A `filecontents` capture ends at `\end{<current environment>}`

**Perl behavior**: latex_constructs.pool.ltxml:4278-4295 binds only
`\begin{filecontents*}`; the bare `\filecontents*` command that a wrapper
environment runs through `\csname filecontents*\endcsname` (latexdemo.sty:97-101
`DefineCode`) is undefined there — an error, but no loss.
**Rust behavior**: the bare command is a primitive; its raw capture ended at a
fixed `\end{filecontents[*]}`, which a wrapper's body never contains, so the rest
of the document was swallowed silently (latex4wp: 63% of the manual). The
terminator is now `\end{<\@currenvir>}` (latex.ltx:19047), the wrapper's own end.
**Why**: faithful to latex.ltx; the fixed marker was the Rust-only defect.
Guard `filecontents_inside_a_wrapper_environment_ends_at_the_wrappers_end`. Batch 56bx.

### 228. `blindtext` does not force `babel[english]`

**Perl behavior**: blindtext.sty.ltxml:21 `RequirePackage('babel', options => ['english'])`
so `\languagename` exists; babel's `\extrasenglish` then runs the English text that
blindtext.sty:342-354 registered, replacing the Latin `\blindtext@text` default
(:356-369) — `\blindtext` reads "Hello, here is some text without a meaning".
**Rust behavior**: no implicit babel; the engine defines `\languagename`
(tex_hyphenation.rs), so a document without babel gets the Latin default exactly as
pdflatex does (the shipped PDFs of assoccnt/xassoccnt), and one that loads babel
gets its own language's text as before.
**Why**: the oracle is the PDF; the forced language was a binding convenience with
a visible, wrong side effect. Guard `blindtext_default_is_latin_without_babel`. Batch 56bx.


### 229. A `.bib` field value keeps an escaped trailing space

**Perl behavior**: `Pre::BibTeX` trims every field value (`BibTeX.pm:273`
`s/\s+$//`), so `type = {Memo No.\ }` becomes `…No.\` and the entry-to-TeX
splice `\csname bib@field@default@type\endcsname{…No.\}` has `\}` eat the
delimiter — the runaway argument swallows the rest of the entry and the next
ones (asmejour ×23 `malformed:ltx:bib-part`). Real bibtex + pdflatex fail the
same way.
**Rust behavior**: the trim keeps one space when the trailing run follows an
odd number of backslashes, so the control space survives (`pre_bibtex.rs`).
**Why**: user-approved surpass (2026-09-17); the source is valid BibTeX and the
PDF shows the entries. Guard `bib_field_trailing_control_space_survives_trimming`.

### 230. A graphic named with its extension is also asked of kpsewhich

**Perl behavior**: `image_candidates` (Util/Image.pm:43-57) searches the
graphics/search paths; when nothing is found it consults kpsewhich only for an
EXTENSIONLESS name with `.png`/`.pdf` appended. A package-shipped asset
referenced by its full name — `\includegraphics[page=N]{openmoji-color-all.pdf}`,
five icon galleries, 97.8 % of the corpus's lost figures (~27,700) — is never
found although kpsewhich locates it.
**Rust behavior**: when the candidate list is empty and the name has an
extension, `kpsewhich <name>` supplies the candidate (`image.rs`).
**Why**: user-approved surpass (2026-09-17). Guard
`graphics_kpsewhich::extensioned_texmf_graphic_is_a_candidate`.

### 231. A `.bib` written by `filecontents` reaches MakeBibliography

**Perl behavior**: `filecontents` caches the file in State; MakeBibliography's
`getBibliographies` uses `pathname_find` (filesystem only), so the `.bib` is
"not found" — Perl admits the gap (MakeBibliography.pm:92-93). Six corpus
manuals ship their `.bib` this way.
**Rust behavior**: the virtual file store is consulted first and the data fed
as a literal source (`make_bibliography.rs`), in the same process (the
monolithic pipeline); a split `--whatsin=xml` post has no store and behaves as
before.
**Why**: user-approved surpass (2026-09-17). Guard
`filecontents_bib_feeds_the_bibliography`.

### 232. `\begin{document}` runs a user-redefined `\document`

**Perl behavior**: `\begin{x}` prefers the `\begin{x}` control sequence
(the document constructor) over `\x`, so `\renewenvironment{document}`
(ltnews.tex:236, l3news.tex:109 — a driver that `\input`s every issue, each a
complete document, inside a group) and `\def\enddocument{…\TO@enddocument}`
(tools-overview.tex:65) are ignored: the first inner `\end{document}` ends the
conversion (ltnews 3.9 % recall, l3news 12.6 %, tools-overview 1.7 %).
**Rust behavior**: for the `document` environment only, `\begin{document}` runs
`\document` and `\end{document}` runs `\enddocument` whenever that macro is the
document's own: not `\ifx`-equal to the ORIGINAL constructor (kept as
`\lx@orig@document`/`\lx@orig@enddocument`, since docmute/subfiles redefine the
`\end{document}` CS itself), not the kernel's post-begin re-let to `\@notprerr`,
and not a wrapper whose body contains the control sequence itself
(standalone.cls captures the kernel macro's body, which here is the unexpandable
alias — a self-loop). The renewed environment is grouped like any other
(`\begingroup … \enddocument\par\endgroup`; only the original `\document`
eats `\begin`'s group), so an issue's local redefinitions revert at its end.
Every ordinary document is unchanged (`sect01.rs`).
**Why**: user-approved surpass (2026-09-17); latex.ltx semantics. Not
generalized to other environments on purpose: a corpus-wide "user environment
outranks the binding" rule would turn every `\renewenvironment{abstract}`
into presentational markup. Guards `document_indirection::*`.
\n

The begin side's decision is recorded group-locally
(`\lx@document@indirected` inside the `\begingroup` it opens,
`\lx@document@direct` inside the constructor's group) and the end side closes
the group whenever one was opened — running `\enddocument` unless it is the
original, in which case the finalizer runs after the `\endgroup`. The
absorb-a-subfile idiom (exam-n.cls:1348 `\includequestion`: `\let\document
\@empty \let\enddocument\endinput \input{…}`) had its begin open a group its
end never closed (`\endinput` is a closure, not a user macro; sweep 82). Guard
`document_indirection::included_subfile_document_closes_its_group`. Batch 56cl.
The end side's three cases (batch 56cn): a user-macro `\enddocument` runs
(then `\par\endgroup` if the begin opened a group); a non-original closure
(`\endinput`) runs and the group closes; the ORIGINAL `\enddocument` takes
the finalizer alone even when the begin opened a group — a package that
wraps `\document` around the original (dblfnote.sty:210-211
`\let\dfn@document\document \def\document{\dfn@document …}`, etoolbox-style
patches) runs the constructor inside that group, so its mode frame sits
above it; in TeX the original never returns and `\end`'s `\endgroup` is
unreachable, and an explicit one met that frame (seven manuals in sweep
83: yafoot-man, guitartabs, zanabazr, recorder-fingering, …). Guard
`document_indirection::package_wrapped_document_finalizes_without_closing_the_begin_group`.

### 233. frontespizio typesets its title page inline

**Perl behavior**: no binding; the raw package runs in its default `write` mode,
which redirects every content macro to a generated `\jobname-frn.tex`
(frontespizio.sty:207-229) for a second pdflatex run and re-includes the result
as a graphic (:606-616) — an empty `<titlepage>` and all six shipped manuals at
0 % recall.
**Rust behavior**: `frontespizio_sty.rs` loads the raw package with the document's
options plus `nowrite,infront`, so the content macros store their values
(:260-309), and the environment's end runs `\preparefrontpage<shape>`
(:311-538), the package's own inline typesetting; xcolor is loaded for the
`suftesi` shape's colours.
**Why**: external compilation is out of scope (as shell-escape); the package
carries the inline route itself, and the PDF is the oracle. `Preambolo` /
`Preambolo*` material — the preamble of that external document — is executed
inline with its package loaders and its `\geometry` page layout neutralised
(frontespizio.sty:120-127 writes geometry into the external file itself;
toptesi-example-con-frontespizio's `\geometry{a4paper,…}` was undefined inline
and its argument leaked as body text, sweep 82). Guards
`frontespizio_inline::{standard_shape_title_page_is_typeset_inline,
preambolo_material_is_executed_inline}`. Batches 56cj, 56cl.

### 234. nomencl entries are glossary definitions listed in full (Perl: lost)

**Perl behavior**: `nomencl.sty.ltxml` raw-loads the package, whose two ends
only WRITE: `\nomenclature[prefix]{sym}{desc}` is a `\protected@write` of a
`\nomenclatureentry` line to `\jobname.nlo` (nomencl.sty:205-245) and
`\printnomenclature` `\@input@`s `\jobname.nls` (:277-282), the output of an
external makeindex run. No makeindex runs, so the list is silently absent —
0 errors, 17-37 % recall on the five shipped samples.
**Rust behavior**: `nomencl_sty.rs` rebinds the two ends onto the glossary
model: each entry is `<ltx:glossarydefinition inlist="nomenclature">` with the
phrases the written line carries — `sort` (prefix + symbol, the makeindex key),
`name`, `description` with `\nomeqref{\theequation}` appended as the written
line does (", see equation (N)" after `\nomrefeq`), and under `nomentbl` `unit`
and `note` — and `\printnomenclature` is an empty `<ltx:glossary
lists="nomenclature" role="nomenclature">` titled `\nomname`. MakeGlossary
(`make_index.rs::get_glossary_entries`) lists EVERY definition of a
`role="nomenclature"` glossary, where a glossary lists only the referenced ones
(Perl MakeIndex.pm:468 `next unless $refs`): a nomenclature has no `\gls`-style
reference — makeindex prints every written line. The list renders only an entry's label
and definition, so a `nomentbl` entry's `unit` and `note` phrases follow the description
as `<ltx:text class="ltx_glossary_unit">`/`ltx_glossary_note` rather than being dropped
(batch 56hn).
**Why**: the in-memory route glossaries already takes instead of makeindex;
the PDF is the oracle (all five samples print the full list).
**Witnesses**: nomencl/sample01…sample05 (TeX Live doc corpus).
**Guard**: `cluster_package_guards::nomencl_inline::{nomenclature_entries_become_glossary_definitions,
printnomenclature_lists_every_entry}`. Batch 56cl.


### 235. biblatex resources resolve by basename and with an implied `.bib` (Perl: no binding)

**Perl behavior**: no `biblatex.sty.ltxml`; the raw package fails to load. Its
`pathname_find` for a classic `\bibliography{}` would also miss a resource named
with a directory that does not exist beside the document.
**Rust behavior**: a resource that misses under its given name is looked up
again with `.bib` appended when it has no extension (biber tries the name, then
`name.bib`; a legacy `\begin{refsection}[refs]` names its resources that way),
and by its basename when the name carries a directory — `\addglobalbib
{../bibtex/bib/biblatex-apa-test-references.bib}` (biblatex-apa-test.tex:69,
apa6-test:444) mirrors the TeX Live SOURCE tree while the `.bib` ships beside
the `.tex`, and no search path climbs out of the document directory
(`make_bibliography.rs::bib_lookup_candidates`; the exact name always wins).
`\refsection[resources]`/`\newrefsection[resources]` register their resources
(biblatex.sty:10757/:10771), where the binding had dropped them.
**Why**: the PDF is the oracle — the shipped bibliographies are the documents.
biblatex-apa-test's 283-entry bibliography (16 % recall) and apa6-test's hinged
on it.
**Witnesses**: biblatex-apa/biblatex-apa-test, biblatex-apa6/biblatex-apa6-test.
**Guard**: `06_cluster_bibliography::{pathful_bib_resource_falls_back_to_its_basename,
refsection_optional_argument_registers_its_resources}`. Batch 56cs.

### 236. `\bibliography` is locked against a raw `\@input{\jobname.bbl}` redefinition (Perl: overridden, nothing rendered)

**Perl behavior**: `\bibliography Semiverbatim` (latex_constructs.pool.ltxml:3895) is
an ordinary macro; a class or package that redefines it — paper.cls:933 (sfee),
ltugboat.cls:1683, abntex2cite.sty:375, gbt7714.sty, the curve/fcavtex/
nxuthesis/thuthesis/uafthesis classes — writes `\bibdata` to the `.aux` and
`\@input{\jobname.bbl}`, a file only a bibtex run produces, and Perl renders no
bibliography (its `.bib` list lives only in the consumed argument; the `.aux` is
never re-read).
**Rust behavior**: the macro is `locked` (sect11.rs), as `\thebibliography`
already is: the raw redefinition is refused with an Info and the native route
runs — a shipped `.bbl` if present, else the `.bib` session at post. A wrapper
that extends the original loses its extras (a heading, single spacing), never
the list.
**Why**: bindings outrank raw; the wrapper's meaning IS "bibliography from
these `.bib`s", which the native macro renders and the wrapper cannot.
**Witnesses**: sfee/SFEE_author (Perl: Fatal), abntex2/abntex2cite (+ -alf),
unbtex/unbtex-example.
**Guard**: `06_cluster_bibliography::raw_bibliography_override_cannot_lose_the_bib_session`.
Batch 56cx.

### 238. `\meaning` and `\string` of a `\noexpand` marker print TeX's `\relax` / the shadowed name (Perl: `\special_relax`)

**Ground truth**: tex.web §358 — `\noexpand`'s marker makes the next token cmd
`relax`, chr `no_expand_flag`, with `cur_cs` the shadowed control sequence
(TeXbook, `background/texbook.tex:13195`: "that token is interpreted as if its
meaning were `\relax`"); `print_meaning` renders that as `\relax` (§266
`print_cmd_chr(relax, …)`), `\string` prints `cur_cs`'s name (§472
`sprint_cs(cur_cs)`), and `\let` stores the `relax` variant itself (§1221
`define(p, relax, no_expand_flag)`), so `\meaning` of the alias is `\relax` too.
pdflatex: `\expandafter\meaning\noexpand\foo` → `\relax`,
`\expandafter\string\noexpand\foo` → `\foo`,
`\expandafter\let\expandafter\x\noexpand\foo \meaning\x` → `\relax`. Perl
represents the marker as its internal `\special_relax` (Gullet.pm:315, the shadowed
token smuggled in the meaning slot) and prints that name in all three. The Rust
per-token family (`\special_relax\x01<shadowed>`, `Token::noexpand_shadowed`) is
the same triple made persistent (its transient-vs-persistent residual is K3,
`docs/perfect_kernel/KERNEL_CAPABILITIES.md`), and every other observable —
`\ifx` against `\relax` (false, `no_expand_flag` ≠ 256) and between two aliases
(true, both the same `relax` variant), `\edef` capturing the plain token (§369),
delimited-argument matching (§392), `\if`/`\ifcat` on an active char (§506) —
already matched pdflatex; the printing primitives now do too, for a directly
expanded marker and for its `\let` alias.
**Guard**: `cluster_package_guards::noexpand_marker_texweb::fourteen_probes_match_pdflatex`
(the whole probe, one `<p>` per case). Batch 56dd.

### 237. `class` values are NMTOKENs and `xml:id`s are XML Names at the emitter (Perl: invalid values pass through)

**Perl behavior**: `setAttribute` writes `class` and `xml:id` verbatim; listings'
language literal becomes `ltx_lst_language_{TeX}_LaTeX` / `_C++` / an unexpanded
`\lexer@cs`, raw `\lx@add@class` with `@` names and a `\par` leaking into a
class or an id (csvsimple-legacy) all reach the XML and fail RelaxNG (33
manuals / 3,488 lines; 2 manuals / 1,231 lines).
**Rust behavior**: `Document::set_attribute`, the one emitter every writer
(including `add_ss_values`) funnels through, keeps each `class` token's
NameChars and drops emptied tokens (`nmtokens_clean`), and passes an `xml:id`
that is not an XML Name through `clean_id`; valid values are untouched.
**Why**: user ruling 2026-09-18 ("we want our class values to be NMTOKENs"):
the XML must validate, and a cleaned class beats an invalid one; fixing every
producer would be a per-package chase.
**Witnesses**: tutodoc-en/-fr, book-of-common-prayer, tikzcodeblocks, tablor,
ualberta, gotham-doc, csvsimple-legacy, manptp (TeX Live doc corpus).
**Guard**: `cluster_schema::listing_language_class_is_an_nmtoken`,
`latexml_core document::attribute_cleaners::{class_values_become_nmtokens,
ids_are_xml_names}`. Batch 56cz.

### 239. `\makebox[w][s]` alignment emits `align="justified"` (Perl: schema-invalid `stretched`)

**Perl behavior**: `%makebox_alignment` (latex_constructs.pool.ltxml #2829)
maps `s => 'stretched'`, so `\makebox[w][s]{…}` (and `\framebox`, `\parbox[s]`,
etc.) emit `align="stretched"` — a value the schema `align` enum forbids
(`LaTeXML-common.rnc:216` = `left|center|right|justified`). pgf-periodictable's
manual alone carries 1419 such invalid attributes.
**Rust behavior**: `makebox_alignment` (`latex_constructs/mod.rs`) maps
`s => "justified"`. `[s]` justifies the box's inter-word glue to fill the
width, which the schema value `justified` denotes and `text-align:justify`
renders — so the output is both schema-valid and render-faithful. A CSS rule
`.ltx_align_justified { text-align:justify; }` (LaTeXML.css) is added because
the XSLT builds the class as `ltx_align_<align>` = `ltx_align_justified`
(distinct from the pre-existing `ltx_align_justify`).
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19);
`stretched` is not a legal `align` value and rendered as plain left before.
**Witnesses**: pgf-periodictable (1419), any `\makebox[…][s]`.
**Guard**: `picture_makebox_offset::makebox_stretch_is_justified`.
**Upstream**: not filed.

### 240. A block-box's Para.class content climbs out of a `<para>` that can't hold it (Perl: in-place rename → schema-invalid)

**Perl behavior**: `insertBlock` (`TeX_Box.pool.ltxml:449-519`) ends with
`my $tag = $allowed_candidates[0] || $filtered_candidates[0];
renameNode($container,$tag,1)` (the rename at `:512-513`) — a direct IN-PLACE
rename. When a block box
(`framed`'s `{shaded}`, a standalone `{minipage}`) whose content is Para.class
material (a `<float>`/`<table>`/nested `<para>`/`<subsection>`) is inserted while
a `<para>` is already open from preceding inline text, `<para>` (Block.model)
cannot hold the `<logical-block>`/`<sectional-block>` chosen, so Perl renames in
place and emits `<para>…<logical-block>` — schema-invalid (`LaTeXML-para.rnc:16`
`Para.class = para | logical-block`; `:56` `para_model = Block.model`).
**Rust behavior**: `insert_block` (`base_utilities.rs`) — when
`allowed_candidates` is empty but a candidate exists, climb from the context to
the nearest ancestor that can hold the tag and move the capture there (ending
the paragraph), then rename. The block ends up a valid child of the section/body.
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19).
Chosen over renaming to the inline variant (`inline-logical-block`, which IS
valid in Block.model) because that renders block content — a `<float>`/`<table>`
— inside an inline `<span>` (`LaTeXML-para-xhtml.xsl:72`), breaking fidelity;
the block form renders as a `<div>` (`:53`). Only the currently-invalid path is
touched (empty `allowed_candidates` in a non-inline context); the inline/`<p>`
path is unchanged.
**Witnesses**: tikz-network (88→1 jing lines), numerica (framed `{shaded}` /
`{minipage}` after inline text).
**Guard**: `perfect_kernel_batch56::logical_block_climbs_out_of_para`.
**Upstream**: not filed.

### 241. algorithm2e double-`\nl` line-number tags collapse to one (Perl: multiple `<tags>`, schema-invalid)

**Perl behavior**: a line that fires numbering more than once — `\LinesNumbered`
(auto `\nl` via `\everypar`) PLUS an explicit `\nl` — runs `\algocf@printnl`
each time, so `LaTeXML/lib/LaTeXML/Package/algorithm2e.sty.ltxml` emits a
separate `<ltx:tags>` per fire (2 in Perl, up to 3 in our content-start timing),
which the listingline model forbids (`tags?`, `LaTeXML-block.rnc:189`). Real
algorithm2e overprints the numbers with `\llap`/`\rlap` (`algorithm2e.sty:1647,
1654`) so only one is visible.
**Rust behavior**: a post-construct pass (`renumber_algo_lines`,
`algorithm2e_sty.rs`) keeps the FIRST `<ltx:tags>` per listingline and removes
the rest (`document.remove_node`), then renumbers the survivors 1..N. First, not
last, so the surviving tags keeps the schema-required leading position that the
endline indentation-prepend (#… batch 56eh) seats it at, ahead of any `<rule>`;
the visible number is corrected by the renumber. Done post-construct on the
stable DOM — an in-place remove inside `\algocf@printnl@i` corrupts the tree
(the rapid double-fire's mid-construction cursor state).
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19),
faithful to the PDF's single visible (overprinted) number.
**Witnesses**: algorithm2e manual (`algorithm2e_exnlsty`); any `\LinesNumbered`
+ explicit `\nl` document. algorithm2e manual 221→0 jing lines across 56eh/ei/eo.
**Guard**: `cluster_algorithm2e_double_nl_collapses_to_one_leading_tag`.
**Upstream**: not filed.

### 242. Frontmatter hoists above a content-free leading pagination (Perl: emitted in place → schema-invalid)

**Perl behavior**: `\title`/`\author`/`\date` queue frontmatter (`\lx@add@*` →
`queue_front_matter`); `\maketitle` flushes it at the CURRENT insertion point
(`\lx@frontmatterhere` → `insertFrontMatter`, `Base_Utility.pool.ltxml:824,918`).
When a content-free pagebreak precedes `\maketitle` — a class `\frontmatter`/
`\clearpage`/`\newpage` emitting `<ltx:pagination role="newpage"/>` before the
title — the frontmatter lands AFTER that pagination. The document model requires
frontmatter to lead (`LaTeXML-structure.rnc:34`:
`(FrontMatter.class|…|titlepage)*, (document.body.class|…)*`), so every following
`<title>`/`<creator>`/`<date>` is schema-invalid. Perl (native AND raw-class) does
the same and is invalid against its own schema (verified same-host, Perl LaTeXML
0.8.8).
**Rust behavior**: `\lx@frontmatterhere` (`base_utilities.rs`) gates on
`leading_document_nodes_content_free` — true when every `<ltx:document>` child
after the last `<ltx:resource>` is content-free (a self-closing `<ltx:pagination>`,
or an empty `<ltx:para>`/`<ltx:p>`/`<ltx:break>`). If so it flushes via
`insert_frontmatter_after_resources` (the `_Capture_`-after-last-resource machinery
`\lx@frontmatter@fallback` already uses), placing the frontmatter ahead of the
pagebreak so it leads. Otherwise it inserts at the current point unchanged.
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19). Safe
because the demoted node is a pagebreak — invisible in HTML, faithful to the PDF
(the `\clearpage` still opens the title on a fresh page). Genuine pre-title content
(a real `<para>`, an author masthead) or an undefined-command `<ltx:ERROR>` para
fails the content-free gate and keeps the Perl-parity in-place placement, so no
visible content is ever reordered.
**Witnesses**: amsmath/amsldoc, amsldoc-it/itamsldoc,
tkz-doc, tkzexample, tuda-ci/DEMO-TUDaPhD, bfh-ci/DEMO-BFHThesis, fbithesis/example,
commedit/sample, ifmslide/ifmman, istgame/istgame-doc, latex-refsheet/thesis,
se2thesis, biblatex-cheatsheet, tzplot/tzplot-doc — 14 s105 docs recovered to 0
rng-errors. NOT covered: a directly-built `<ltx:titlepage>` element after a leading
pagination (toptesi/* frontispiece — 4 docs; distinct mechanism, the titlepage is
its own content not queued frontmatter, tracked separately, cf. #233); and a leading
sequence whose intervening para is not provably empty at construct time
(gitinfo2/gitlog, 2 docs — conservatively declined rather than risk hoisting over
hidden content). A general document-finalization demotion pass would cover both
residuals uniformly (follow-up).
**Guard**: `frontmatter_hoists_above_content_free_leading_pagination` (+ the negative
control `frontmatter_does_not_hoist_above_a_leading_graphic`).
**Follow-up**: the two shapes this hoist cannot see (a directly-built `<titlepage>`; an empty-box paragraph whose `<ltx:text>` folds away only at paragraph close) are settled by #245's `\end{document}` pass.
**Superseded (mechanism)**: the placement logic now lives in #247's single `place_frontmatter` pass; the behavior and witnesses above are unchanged.
**Upstream**: not filed.

### 243. A `\put`-positioned box in a picture group is `<inline-block>`, not `<block>` (Perl: schema-invalid `<block>`)

**Perl behavior**: a `\put(x,y){\parbox…}`/`\makebox` inside a `picture` digests into
the positioned `<ltx:g transform>` group and calls `insertBlock`
(`TeX_Box.pool.ltxml:493`) with context `ltx:g`. `$inline = $is_svg ||
canContain($context,'#PCDATA')` is false for `ltx:g` (it holds no `#PCDATA`), so the
block candidate family is chosen; `g_model` has no block element, so it falls through
to an in-place rename to `<block>`. But `g_model` (LaTeXML-picture.rng) is inline-only
(`Picture.class | Inline.class | Misc.class | Meta.class`) — `<block>` (Block.class) is
NOT in it, `<inline-block>` (Misc.class) IS — so `<g>/<block>` is schema-invalid. Perl
(native AND raw-class) emits the same.
**Rust behavior**: `insert_block` (`base_utilities.rs`) treats a container that holds
`<inline-block>` but neither `<block>` nor `#PCDATA` as inline
(`can_contain_qsym(ctx,"ltx:inline-block") && !can_contain_qsym(ctx,"ltx:block")`), so
the capture is renamed IN PLACE to `<inline-block>` — valid in `g_model`, and the box
keeps its `\put` position inside `<g transform="translate(x,y)">` (no move). A
`<para>`/`<inline-block>` holds BOTH block and inline-block, so the clause is false
there and the #240 block-climb (`context_tag == "ltx:para"`) is untouched — this is a
SIBLING branch of the same `insert_block` candidate logic, not a generalization of the
climb (climbing would hoist the card text out of the picture and destroy the layout).
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19). A
`\parbox`/`\makebox` is an LR-box, semantically an inline-block; the in-place rename is
fidelity-safe (no reorder).
**Witnesses**: ticket/ex_flashcard + ex_flashcard_rm (88→0 each), ffslides/ffslides-doc
(135→0), elzcards/elzcards-examples (41→0), simplecd/examples (32→0), flacards/flacards_ex,
bytefield/bf-example, bookcover — 36 s105 docs (sub-cluster A), each block-only invalid.
NOT covered: `<block>` directly in a list (`itemize`/`enumerate`) — colorframed (RUST-ONLY
framed-in-list nesting) + tableaux (SHARED), 2 docs, distinct root cause (list nesting).
**Render**: a `\parbox` in a picture still renders as a `<foreignObject>` of the SAME
dimensions/position; only the inner wrapper flips `<div class="ltx_block">`→`<span
class="ltx_inline-block" style="width:…">` (and its `<p>`→`<span class="ltx_p">`) — a
100pt/56.9pt fixed-width box wraps text identically, so render-faithful. Two SVG goldens
re-blessed: `parbox_inside_a_picture_renders_as_a_foreign_object` (cluster_xslt_split),
`picture_nested_in_a_scaled_box_is_converted` (cluster_package_guards).
**Guard**: `put_parbox_in_picture_is_inline_block_not_block`.
**Upstream**: not filed.

### 244. A Para.class box in a Block.model context demotes to `<block>` (Perl: schema-invalid `<logical-block>`)

**Perl behavior**: a box-forming construct (`\begin{center}`/flushleft/flushright via
`aligningEnvironment`→`insertBlock`, or minipage/`\parbox`) whose digested content is
Para.class (a `<para>` auto-opens when a genuine block — a centered `tabular`, a display
`picture`, a nested minipage — sits mid-content) is inserted into a Block.model container
(titlepage/quote/figure/table/float/abstract/inline-block). `insertBlock`
(`TeX_Box.pool.ltxml:512`) picks `$allowed_candidates[0] || $filtered_candidates[0]`; with
no allowed candidate it renames in place to an invalid `<logical-block>` (`Block.model =
Block.class|Misc.class|Meta.class` has no Para.class; LaTeXML.rnc:37). Perl (native AND
raw-class) emits the same invalid nesting.
**Rust behavior**: `insert_block` (`base_utilities.rs`) — when the context holds `<block>`
but not the Para.class `final_tag`, AND `subtree_is_block_demotable` — emits `<block>` and
recursively demotes the Para.model content BOTTOM-UP: rename descendant `logical-block`/
`sectional-block` → `block` (KEEPING attributes — a minipage's `width`), unwrap descendant
`<para>` (its Block.model children float up, valid in a block). Bottom-up, container
renamed LAST, so no Para.class node is ever transiently moved into a `<block>` (which would
emit a `malformed` Error). The `subtree_is_block_demotable` gate LEAVES at Perl-parity any
container holding a Para.model-only element `<block>` can't take — a `<float>`, `<TOC>`, a
sectioning unit (stanli, webquiz's TOC branch) — rather than stranding it as a fresh error.
Sibling of #240's climb (which reorders content out of a `<para>`) and #243's inline
demotion; this is the in-place BLOCK demotion. SHARED Perl bug, surpass on the
schema-validity axis.
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19). Render-safe:
`.ltx_para`/`.ltx_block`/`.ltx_logical-block` are all `display:block` (LaTeXML.css:205)
with no distinguishing layout; the demotion preserves child order and attributes, dropping
only a redundant `<para>` grouping div + its auto-`xml:id`.
**Witnesses**: 11 s105 docs → 0 rng (tabularcalc_doc_{en,fr,vn}, short-math-guide,
pinoutikz, msc, listparskip, heria-proposal, fepslatex, kaytannollista, l2tabufr); 3
partially improved (l2tabu 2→1, webquiz 2→1, suanpan-l3 2→1 — residuals are separate
clusters); stanli unchanged (float content, correctly skipped — no regression). Also fixes
the `caption_in_inline_parbox` sibling (heria, inline-block context).
**Guard**: `para_class_box_in_titlepage_demotes_to_block_not_logical_block`.
**Upstream**: not filed.

### 245. Leading content-free nodes relocate past the frontmatter at `\end{document}` (Perl: frontmatter schema-invalid after a pagebreak)

**Perl behavior**: the document model is `(FrontMatter.class | SectionalFrontMatter.class |
Meta.class | titlepage)*, (document.body.class | BackMatter.class)*` (LaTeXML-structure.rnc:34)
and `Para.class` — hence the body group — includes `pagination` and `para`
(LaTeXML-para.rnc:18-20). Two constructs deposit a content-free body node BEFORE the
frontmatter and so invalidate every `<title>`/`<creator>`/`<date>`/`<titlepage>` after it:
(1) `\cleardoublepage` (memoir `\frontmatter`, gitinfo2/gitlog) = `\clearpage \hbox{}
\newpage` → `<pagination>, <para><p/></para>, <pagination>` ahead of the `\maketitle`
flush; (2) a `{titlepage}` environment after a `\clearpage` (toptesi frontispiece) — the
`<titlepage>` is built in place, not queued, and its `after_construct` `insert_frontmatter`
lands the `\title`/`\author` right behind it. Perl (native and raw-class) leaves both invalid.
**Rust behavior**: #242's construct-time hoist cannot see either: the empty box's
`<ltx:text>` is folded into its `<p>` only when the paragraph CLOSES
(`auto_collapse_children`), so at flush time the content-free gate sees an element and
declines; a titlepage is never queued. Both settle into the same final tree
`resource*, content-free+, frontmatter+, body*`, so `\lx@finalize@document` (sect02.rs,
after the `@at@end@document` fallback flush) runs
`relocate_leading_content_free_past_frontmatter` (`base_utilities.rs`): the leading run of
content-free nodes (per `node_is_content_free` — `<pagination>`, or an empty
`<para>`/`<p>`/`<break>` with no visible descendant) moves, in order, to just after the
contiguous run of frontmatter-only elements that follows it. "Frontmatter-only" is the
document model's first group, read from the schema — admitted by `ltx:document`, not by
`ltx:sectional-block` (model `document.body.class*`), and, to exclude the BackMatter.class
the document also admits, `titlepage` or something `ltx:bibliography`'s leading
`FrontMatter.class*, SectionalFrontMatter.class*` admits — so no element names are
hard-coded; a Meta.class element (allowed in both groups) ends the run, and a pagebreak
before a leading `thebibliography` stays put (guard). #242 stays as the construct-time fast path; this is
the finalization safety net that covers what it structurally cannot.
**Why**: kernel-quality / schema validity (user-approved surpass 2026-09-19). Render-safe:
the moved nodes carry no ink — an empty paragraph renders nothing, a `<pagination>`'s only
HTML trace is the 2em gap of `.ltx_pagination.ltx_role_newpage`, which now falls below the
title instead of above it (exactly #242's result); frontmatter and body keep their order. A
leading node with visible ink (a logo `<graphics>`, a `<rule>`, text) is not content-free
and blocks the pass, so a hand-typeset cover (toptesi-it's TeX-logo page) stays above the
`<titlepage>` — invalid but faithful; the pass never reorders visible content. Streaming:
a spilled document top simply yields no leading run (no-op).
**Witnesses**: gitinfo2, gitlog (3→0 rng-errors each); toptesi/FrontespizioScudo (3→0),
toptesi-example-{sss,triennale,con-frontespizio} (1→0); toptesi-it unchanged (visible
cover, correctly left).
**Guard**: `perfect_kernel_batch56::{frontmatter_relocates_past_a_leading_empty_box_pagebreak,
titlepage_relocates_past_a_leading_pagination, titlepage_stays_below_a_visible_leading_cover,
pagebreak_before_a_leading_bibliography_stays_put,
frontmatter_stays_below_a_leading_framed_empty_box}`.
**Superseded (mechanism)**: the placement logic now lives in #247's single `place_frontmatter` pass; the behavior and witnesses above are unchanged.
**Upstream**: not filed.

### 246. Abstract-only fallback: title-page layout wraps as `<titlepage>`, the abstract follows (Perl: abstract at the top; Rust had it trailing the body)

**Perl behavior**: with no `\maketitle`, queued frontmatter is flushed by
`\lx@frontmatter@fallback` (first `\section` via `\@startsection@hook`, or `\end{document}`),
which ALWAYS inserts at the document top after `ltx:resource[last()]`
(`Base_Utility.pool.ltxml:927-945`); the abstract-only case is merely deferred to that
fallback (`:830-834`). A manual whose only frontmatter is an `{abstract}` gets
`abstract, (hand-typeset cover as body), TOC, section` — schema-valid, cover below the abstract.
**Rust behavior (before)**: a beyond-Perl rescue for arXiv 1609.07638 (a hand-formatted
`\begin{center}` title above the abstract, no `\maketitle`) promoted the first centered
display-font paragraph to `<title>` and flushed the abstract at the CURRENT position so it
would not float above that title. When the body had emitted a cover block, `\tableofcontents`
or `\clearpage` before the first `\section`, the `<abstract>` landed after them —
schema-invalid (`Para.class |= TOC`, LaTeXML-structure.rnc:682) and later than the source
order. RUST-ONLY: same-host Perl validates all 16 witnesses. The promotion itself was also
unreliable: tikz-mirror-lens got `<title>FHZ</title>` (the author, from an author-first cover
whose `\Large{#1}` leaks onto the title paragraph), isosigns/bootstrapicons got
`<title>VERSION 2.1</title>` (a badge in a tcolorbox) — layout mis-read as metadata.
**Rust behavior (now)** (`\lx@frontmatter@fallback`, `base_utilities.rs`; user ruling
2026-09-20 — a regular document needs no body `logical-block(author)`; titlepage layout is
not semantic, the frontmatter metadata schema is): (1) `maybe_promote_leading_title` promotes
ONLY the unambiguous 1609.07638 shape — a plain `center`/paragraph block at document level
whose FIRST paragraph is centered, has letters, and is set in a display font no other
paragraph of the block shares (`first_plain_paragraph`/`collect_plain_paragraphs` descend
through `logical-block`/`para` only — never into a tcolorbox/parbox/tabular). FHZ
(two display paragraphs) and VERSION 2.1 (nested box) are declined; so is tipfr-doc's real
`tipfr.sty` (also in a tcolorbox — declined rather than coin-flipped; no title metadata beats
a wrong one). (2) `wrap_leading_layout_in_titlepage` moves the remaining leading run of
`pagination`/`para`/demotable `logical-block` children (after resources and the promoted
title, and BEFORE the abstract's own source position — `\lx@add@abstract`/`\lx@begin@abstract`
construct an empty `ltx:_Frontmatter_Capture_` marker where the abstract was queued — only
while no frontmatter has been placed yet; removed by `insert_frontmatter`, and any stranded
`*_Capture_` scaffolding is dropped by the core `finalize_rec` (the s107 sweep leaked the
marker in 8 docs whose abstract came AFTER `\maketitle`, e.g. lips' `\DocInput`-ed
`% \begin{abstract}`, or whose `\end{document}` was swallowed by an unbalanced conditional —
an end-of-document hook is not a reliable cleanup point, the core finalize is). The flush
point cannot be the bound, since a sectionless manual flushes at `\end{document}` and would
see its whole body as cover; and promotion/wrap run only on the FIRST flush — a LATE abstract's
preceding content is body, `insert_late_frontmatter` files it beside the existing frontmatter) into a new `<ltx:titlepage>` in its exact order — `titlepage_model = (FrontMatter.class
| SectionalFrontMatter.class | Block.class)*` is the schema's container for exactly this
layout — renaming `para`→`block` and demoting `logical-block` with #244's bottom-up
`demote_para_class_content`; the run stops at the first non-layout node (`<TOC>`, section,
float) and is skipped when nothing in it is visible. (3) The abstract is flushed right after
the wrapped titlepage, else after the last first-group element standing before its marker
with only content-free nodes between (a promoted `<title>`, a directly built `<titlepage>` —
`last_frontmatter_before_abstract_mark`, which also keeps eager and streaming builds
identical: an eager build's `{titlepage}` `after_construct` files a later-queued abstract
right behind the titlepage because digestion completed first, a streaming build reaches the
fallback with it still queued), else at the TOP like Perl (`insert_frontmatter_after_node` /
`insert_frontmatter_after_resources`). Result: `title?, titlepage?, abstract, TOC, …` — all
first-group, valid; the cover stays above the abstract in its exact order, and only trailing
body that cannot be frontmatter (a `<TOC>`, a document-level `<rule>`) sinks below the hoisted
abstract, as any frontmatter-leads transform must (Perl's direction too).
**Why**: schema validity + faithful semantics. The cover's visible order is preserved exactly
(it stays above the abstract, as in the PDF; `.ltx_para`/`.ltx_block` are both `display:block`),
no metadata is guessed, and the HTML front-matter order (title, authors, metadata, abstract,
body) is the XSLT's job from schema-valid core XML. Better than Perl on both counts (Perl
floats the abstract above the cover and never promotes).
**Not covered (SHARED, fidelity-blocked)**: pgf-spectraManual (abstract after visible ink
inside a `{titlepage}` whose `after_construct` already flushed title/creator) and five docs
whose leading VISIBLE para is a leaked/undefined command before `\maketitle` (iodhbwm
glossaries `\glsnonextpages`, psvectorian `\DeclareDocumentMetadata`, forest-doc
`\@escapeifif`, biocon noweb `\nwfilename`, cnbpaper 2-arg `\author`) — Perl leaks them
identically.
**Witnesses**: 13 → 0 rng-errors (toptesi-example-magistrale, tipfr-doc, schulmathematik, russ_doc, PRTEC24-template, pgf-interference-{en,de}, jourcl, isosigns-docs, derivative, democodetools, bootstrapicons-docs, axodraw2-man); 3 improved (tikz-mirror-lens 5→4, tikz-mirror-lens-PT 5→4, tikz-among-us 2→1 — residual = the animate `frame-count` attribute cluster); pgf-spectraManual 8→8 unchanged (SHARED, left); controls amsldoc 0→0, gitinfo2 3→0 and FrontespizioScudo 3→0 (56es) unchanged; 1609.07638 shape → `title, abstract, section`; the fixture shape → `title, titlepage, abstract, section`. Same-host Perl 0.8.8 validates the 13 too (parity on validity; ours keeps the cover above the abstract and the title as metadata).
**Guard**: `perfect_kernel_batch56::{abstract_only_fallback_floats_to_top,
promoted_title_stays_above_the_top_flushed_abstract,
promoted_title_remainder_becomes_titlepage_layout_before_the_abstract,
ambiguous_display_font_cover_is_titlepage_layout_not_a_title,
version_badge_in_a_box_is_not_promoted_to_a_title,
sectionless_body_after_the_abstract_is_not_cover_layout,
late_abstract_after_maketitle_leaves_no_marker_and_wraps_nothing}`; fixture `structure/promote_center_title`
(re-blessed to `title, titlepage, abstract, section`); `class_redefined_abstract_defers_its_argument`
(a class `\abstract{}` store, sectionless) unchanged.
**Superseded (mechanism)**: the placement logic now lives in #247's single `place_frontmatter` pass; the behavior and witnesses above are unchanged.
**Upstream**: not filed.

### 247. One frontmatter placement pass (consolidates #242, #245 and #246's anchor plumbing; zero behavior change)

**Perl behavior**: `\lx@frontmatterhere` places the queued frontmatter at the current point;
`\lx@frontmatter@fallback` places it at the top after `ltx:resource[last()]`; nothing else
(`Base_Utility.pool.ltxml:824-944`). Whatever precedes the placement stays where it is.
**Rust behavior**: `place_frontmatter(document, wrap_layout, anchor, on_stop)`
(`base_utilities.rs`) runs at every placement point — `\lx@frontmatterhere`, both
`\lx@frontmatter@fallback` branches, the `{titlepage}` environment's `after_construct` — and
classifies the LEADING REGION (root children between the resources and the bound: the
abstract's `ltx:_Frontmatter_Capture_` position marker when present, else everything built so
far): a first-group element (`is_frontmatter_group_element`) is the anchor candidate; a
content-free node (`node_is_content_free`) is moved after the placed frontmatter if it
preceded it; hand-typeset layout (`para` / demotable `logical-block` / `pagination` with
visible ink, `wrap_layout` = the abstract-only fallback only) is wrapped in `<titlepage>` in
its exact order; anything else ends the region and the pass places exactly as Perl does
(`on_stop`: current point for `\maketitle` and the titlepage flush, top for the fallback).
With nothing queued the content-free normalization around a built `<titlepage>` still runs,
so a `\clearpage` before a `{titlepage}` settles when the titlepage is built.
**Why**: the 2026-09-20 consolidation study (Opus 4.8, ~60 corpus docs, every pinning test,
the Perl baseline) found #242's `\maketitle` content-free gate, #245's `\end{document}`
relocation and #246's wrap/anchor helpers keyed on four slightly different notions of "what
may precede the frontmatter", interacting (a build-then-finalize two-step for the toptesi
frontispieces; gitinfo2 fixed only at finalize) and producing the s107 regressions. User
ruling: "consolidate only, no policy flip" — same behavior, less code. Deleted:
`leading_document_nodes_content_free`, `relocate_leading_content_free_past_frontmatter` (+ its
`\lx@finalize@document` call), `wrap_leading_layout_in_titlepage`,
`last_frontmatter_before_abstract_mark`, `insert_frontmatter_after_node`,
`insert_frontmatter_after_resources` (four code files 227+/287−, net −60 lines; 5 mechanisms → 1). Accepted superset on the `\maketitle` path: a built `<titlepage>` ahead of `\maketitle` now anchors the flush even when a content-free node sits between them (#242 placed at the current point there; no corpus doc has the shape, the anchored tree is the valid one). The pass drains `frontmatter_raw` first so `frontmatter_pending` and the flush agree under streaming at the `{titlepage}` `after_construct`, which has no digest hook of its own. Kept, not redundant:
the marker + `remove_frontmatter_marks` + core `finalize_rec` sweep (the bound), the
conservative title promotion (#246), `maybe_dedup_leading_title_ink` (#6924),
`insert_late_frontmatter`. NOT adopted (needs its own ruling): wrapping VISIBLE leading layout
on the `\maketitle` path — it would flip three fidelity controls
(`frontmatter_does_not_hoist_above_a_leading_graphic`, `titlepage_stays_below_a_visible_leading_cover`,
`frontmatter_stays_below_a_leading_framed_empty_box`) and cannot tell a genuine cover (9 docs)
from a leaked command's argument text (8 docs).
**Witnesses**: all 45 cataloged docs identical to s107 (rng counts and leading shapes) — the 56ep 14, 56es 6, 56et 16, P4 2, the fidelity/ERROR controls (toptesi-it 1→1, ltnews 1→1, l3news 1→1, pgf-spectraManual 8→8, oubraces 6→6, a4wide 4→4, eqnnumwarn 2→2, turnstile 3→3, lua-tikz3dtools 3→3); the only differences are 56et.1's marker-leak fix that s107 predates (forest-doc 4→3, lips 1→0). 22 targeted tests (14 guards + 5 fixtures + streaming eager≡streaming + title-ink dedup + class-abstract store); full suite 2993/2993; clippy + rustdoc clean.
**Guard**: the 14 `perfect_kernel_batch56` frontmatter guards of #242/#245/#246 unchanged; fixtures
`structure/{promote_center_title,faketitlepage,titlepage,svabstract,amsarticle}`;
`114_streaming_structure` (eager ≡ streaming); `06_cluster_regressions::cluster_frontmatter_title_ink_dedup_6924`.
**Upstream**: not filed.

### 248. animate's frame count rides as RDFa `property="ltx:frameCount" content="N"` (Perl: no animate binding)

**Perl behavior**: no `animate.sty` binding at all; the environment's frames are raw
interpretation.
**Rust behavior**: the `animate` contrib binding collapses an animation to one representative
frame and records the frame count on `<ltx:block class="ltx_animate">`. It used to do so with an
ad-hoc `frame-count` attribute registered only in the RUNTIME model
(`model::add_tag_attribute`), so it survived construction but every document with an animation
failed the static schema (`attribute "frame-count" not allowed here`: chessboard, liftarm,
movie15, tikz-among-us, tikz-mirror-lens ×2). The count now rides as RDFa —
`property`/`content` are `Common.attributes` on every element and `LaTeXML-common.xsl` passes
both through to HTML — so the metadata is kept and the XML validates.
**Why**: schema validity without losing the one piece of metadata the collapse leaves behind
(user preference: capture metadata). Nothing consumed `frame-count` downstream.
**Witnesses**: chessboard, liftarm, movie15, tikz-among-us, tikz-mirror-lens(-PT) — the
`frame-count` rng-error gone. **Guard**: the seven `perfect_kernel_batch56` animate guards now
assert `content="N"`.
**Upstream**: n/a (no Perl binding).

### 249. A degenerate SVG clip or xy path emits no `<svg:path>` (Perl: `<svg:path>` without `d`)

**Perl behavior**: `\lxSVG@drawpath@clipped`/`\lxSVG@discardpath@clipped`
(`pgfsys-latexml.def.ltxml:342-371`) and `xylatexml.tex.ltxml`'s path emitter set `d="#1"`
unconditionally; `setAttribute` skips an empty value (`Document.pm:1381-1382`), so a clip with no
path operations (`\path[clip];`) or an empty xy path yields `<svg:path/>` with no `d` —
schema-invalid (`d` is required). SHARED with Rust (`document.rs` skips empty values the same way).
**Rust behavior**: when `d` is empty the clipped constructors emit the `<svg:clipPath>` without
its path and skip the `<svg:use>` (an EMPTY clipPath clips everything away, as the empty PDF clip
does; the clip group stays), and `xy_emit_path` emits nothing (nothing to draw). The unclipped
path already had this guard (`?#1(...)()` in Perl, `!d.is_empty()` here).
**Why**: schema validity; render-neutral (an empty path draws nothing; an empty clip keeps
clipping everything).
**Witnesses**: circuitikz/circuitikzmanual, xytree/xytree-doc-en (`svg:path missing required
attribute d` gone). **Upstream**: not filed.

### 250. A sectioning unit inside a block-mode box floats out as a real section (Perl: `<sectional-block>` inside the enclosing section, schema-invalid)

**Perl behavior**: `insertBlock` (`TeX_Box.pool.ltxml:504-524`) tries the block candidates
(block, logical-block, sectional-block, figure); only `sectional-block` can hold a `<section>`,
so a `minipage[t]`/`framed`/`tcolorbox` whose body ran `\section` is renamed to
`<sectional-block>` in place — inside the enclosing `<section>`/`<chapter>`/`<quote>`, where the
schema admits no `sectional-block` (`document.body.class` only, LaTeXML-structure.rnc:23-47,99).
0 build errors, invalid tree. SHARED with Rust before this batch.
**Rust behavior**: in LaTeX a `\section` inside a box IS a section: `\@startsection` ran, the
counter advanced, and text after `\end{minipage}` belongs to the new unit. `insert_block`
(`base_utilities.rs`) now checks, before the candidate machinery, whether the captured content
holds a sectioning unit (schema-read: admitted by `ltx:sectional-block`, not by
`ltx:logical-block` — part…subparagraph, slide, sidebar; never a float/theorem/para) and whether
the insertion context has an auto-closeable path to an ancestor that admits it
(`sectioning_floatable`: walk up while the node admits the unit or `can_auto_close` — a
`<quote>`, a list `<item>`, a `<figure>` stop the walk and keep Perl's shape). If so every
built unit is MOVED to the point `find_insertion_point_qsym` chooses for it live — closing the
enclosing unit for a same-or-higher level, nesting a deeper one — and made the insertion node,
so trailing box content and the post-box text nest inside; non-sectioning lead content stays
where the box was; the box's `class` (`ltx_minipage`) and any attributes
`Sectional.attributes` admit (`framed`/`framecolor`/`cssstyle`) ride the first floated unit
(`width`/`vattach` are dropped, as the old `sectional-block` already dropped them). Numbering
needs no repair: the unit already carries its digest-time `<tags>`.
**Why**: user ruling 2026-09-20 — "we should not treat a section in a minipage as a schema
extension … convert the minipage into an annotation via class attribute to the usual
ltx:section … semantics usually is better." The box is presentation; the sectioning is
semantics. Valid, and faithful to LaTeX's own structure (visible order unchanged).
**Witnesses**: aguplus (quote-gated — the box sits in `\begin{quote}`, Perl shape kept by design, jing 1→1), fancyvrb-doc 5→4, latex-via-exemplos 23→5 (36 `<sectional-block>` → 0; residual is frontmatter `creator`/`title`), phonenumbers-de 1→0, phonenumbers-en 1→0, recorder-fingering 2→0, tasks-manual 1→0, unamth-template/tesis 2→0 (jing errors, s107 → this branch; the `<sectional-block>` count in every non-quote witness is 0). **Golden**: `tests/graphics/framed.xml` (a `\paragraph` in a
`framed` env now nests directly in its section, frame attributes on the paragraph).
**Guard**: `perfect_kernel_batch56::{section_in_a_block_minipage_floats_out_as_a_sibling_section,
subsection_in_a_block_minipage_nests_in_the_enclosing_section,
section_in_a_minipage_inside_a_quote_keeps_perl_parity, section_in_a_center_environment_floats_out_and_keeps_its_alignment_class}`.
**Upstream**: not filed.

### 251. A Fatal ends digestion and its diagnostics stay lossless: `hardYankProcessing`, `Fatal:` at the raise, `IGNORE_ERRORS` scopes (Perl: the same, then `die`)

**Perl**: `Fatal` (Common/Error.pm L297-L318) counts, prints, calls
`hardYankProcessing` (L320-348: stash `@LaTeXML::LIST` as `rescued_boxes`,
reset pushback / mouth stack / mouth) and sets `$LaTeXML::IGNORE_ERRORS = 1`, so
`Error`/`Warn`/`Info` (L362/L380/L396) drop every later record whole; the
`finishDigestion` retry (LaTeXML.pm L251-259, Core.pm L214-226) re-collects the
rescue and finds no input. **Rust** recovers a partial document instead of dying,
and until 2026-09-20 the salvage `digest_internal` kept reading the LIVE mouth
past a resource Fatal (`Timeout:{PushbackLimit,TokenLimit,Recursion,…}`) while
`emit_record` muted every Error record — so jlreq's `PushbackLimit` fired inside
the class, luatexja then loaded and the whole body was converted with
`\kanjiskip` and 7 more undefined names tallied in the summary but never logged
(asternote, hideanswer-doc, inlinelabel, jpnedumathsymbols-doc). The user
directive "we depend on the messages for establishing success" is met by making
the pipeline Perl-shaped rather than by admitting more post-Fatal messages:

1. **`core_interface::hard_yank_processing`**: on a resource Fatal,
   `digest_internal` rescues the completed bodies (`RESCUED_BOXES`, Perl's
   `rescued_boxes`) and `gullet::flush()`es the input; the converter's salvage
   pass re-collects the rescue and reads nothing more. The in-progress body is
   still not salvaged for `Timeout:Recursion` (2605.25400: reviving it
   re-entered the loop during build) — only the stomach box-cap path salvages
   pending lists, as before.
2. **`Fatal!` / `fatal!` write their `Fatal:` line at the raise** (Perl L312);
   `Error::log_fatal` at the converter/post sinks is idempotent per fatal.
   Before, only the sink logged, so a binding swallowing the `Err`
   (`let _ = digest(..)`, 11 sites) could leave a counted-but-unlogged phantom
   fatal (witness 1903.01633).
3. **`IgnoreDiagnosticsScope`** (`set_ignore_diagnostics`) is the one
   sanctioned silence — Perl's `IGNORE_ERRORS`, dropping line AND count
   together and skipping the cap bookkeeping (Error.pm L362 returns before
   the MAX_ERRORS check), RAII-restored and reset per conversion. The degraded
   no-dump expl3 raw-load (`expl3_sty.rs`, #651) uses it, together with the
   state-level `SUPPRESS_UNDEFINED_ERRORS`/`SUPPRESS_UNEXPECTED_ERRORS` the
   dump build sets, instead of restoring counters while its unconditional
   `Error:` lines had already reached the log.
4. **`emit_error` fires `Fatal:TooManyErrors` once** (the frozen post-latch
   count kept the crossing test true on every later call).
5. **A worker-thread panic ends like any Fatal** (`bin/latexml_oxide.rs`):
   `Fatal:panic:caught` + the `Conversion failed` verdict + exit 1 (s107
   texproposal aborted with a bare `worker thread panicked`, visible only to
   the exit code).

The post-Fatal Error mute itself stays (Perl `IGNORE_ERRORS`); with the yank,
what reaches it is teardown noise. Warnings are not muted (fail toward
flagging). Measured on s107 (2,371 logs): zero counted-but-unlogged Errors or
Warnings and zero fatal counters without a `Fatal:` line — the code defects were
latent; the observed gaps were the continuation (jlreq family) and the panic.
**Guards**: `perfect_kernel_batch56::a_resource_fatal_rescues_the_digested_bodies_and_reads_no_further_input`,
`common::error::tests::{fatal_macro_logs_its_line_at_the_raise_exactly_once,
ignored_diagnostics_scope_drops_line_and_count_together,
emit_error_fires_the_too_many_errors_fatal_once,
errors_are_silent_once_a_resource_fatal_is_latched}`,
`cluster_cli::worker_panic_ends_with_a_fatal_line_and_the_verdict`.
**Upstream**: not filed (Rust recovers where Perl dies).

### 252. `document.body` admits `ltx:subparagraph` (Perl's schema: `paragraph` but not `subparagraph` at document level)

**Perl** (`LaTeXML-structure.rnc:23`): `document.body.class = Para.model | paragraph |
subsubsection | subsection | section | chapter | part | slide | slidesequence |
sidebar | sectional-block` — every other sectional container (part, chapter,
section, subsection, subsubsection, paragraph, appendix) admits `subparagraph`;
`document.body` alone lists `paragraph` and omits it, an asymmetry with no semantic
ground (LaTeX permits a top-level `\subparagraph`). It bites when a class typesets a
heading through `\@startsection` at level `\@M` — smfart.cls:873 for the
`\tableofcontents` title — which both engines clamp to the deepest unit, so
`<subparagraph inlist="toc"><title>Contents</title>` lands as a `<document>` child
and the document is schema-invalid (smflatex/smf-edoc, smf-fdoc; identical tree in
Perl). **Rust**: `ltx:subparagraph` is added to `document.body` (user ruling
2026-09-22) in `latexml_core/resources/RelaxNG/LaTeXML.model` (`document.body`, and
the flattened `ltx:document`/`ltx:sectional-block`/`ltx:inline-sectional-block`
lines), `LaTeXML-structure.rnc:23` and `LaTeXML-structure.rng`. Pure widening: no
document that validated before becomes invalid. Not extended: sectioning units
inside `item`/`figure` (the 56fb carve-out; 14 docs, Perl-identical tree,
PDF-faithful — ruled "leave" the same day). **Guard**:
`perfect_kernel_batch56::a_top_level_subparagraph_is_admitted_by_the_document_body`.
**Upstream**: worth filing (schema asymmetry).

### 253. The locked kernel `\author` absorbs a raw class's trailing argument and appends per call when the class redefined `\author` (Perl: the surplus typesets, the creators collapse)

**Perl** (`latex_constructs.pool.ltxml:1076`, `\author[]{}` locked) and Rust
(`sect05.rs`) refuse a raw class's own redefinition of `\author`, so under
`[rawclasses]` a class that declares MORE arguments leaves them in the stream:
`cas-common.sty:895 \RenewDocumentCommand\author{O{} m O{}}` (els-cas
cas-sc/cas-dc-sample: `\author[1]{Name}[type=editor, orcid=…]`) typesets
`[type=editor, orcid=…]` as a `<para>` before the title, and
`cnbwp.cls:273 \def\author{\@ifnextchar[…}` → `\CNB@authorLong#1#2`
(`\author{Name}{Affiliation}` once per author) typesets every `{Affiliation}`
and, because the kernel `\author` dequeue-replaces, collapses three authors to
the last one. Both engines leak identically (SHARED); pdflatex honours the
class. User ruling 2026-09-22: surpass. Letting the class body run is NOT the
fix — verified: cas-common, cnbwp and aomart store names in class-private
accumulators (`\g_stm_prelimsau_seq`, `\CNB@authors`, `\@names`) laid out only
by their own `\maketitle`, which stays locked, so the authors vanish. **Rust**
(`base_utilities.rs`, `\lx@add@authors@adaptive`, `\lx@author@trailing`; the
kernel `\author` and its `inst_support`/`sv_support`/`llncs` twins carry the
tail): ONLY when `\author:redefined` is set (the lock recorded a refused
redefinition, `state.rs install_definition`) — (a) the trailing `[keyval]`
is absorbed: `orcid=` → `ltx:contact role=orcid`; every other key (the
cas-common `stm/author` family: `type`, `auid`, `bioid`, `alt`, `prefix`,
`suffix`, `role`, `style`, …) is production metadata and is dropped; a
trailing `{…}` → `ltx:contact role=affiliation`; (b) creators APPEND
(`add_authors_calls(stuff, replace=false)` — the same author-line parser
as `\lx@add@authors`, minus the dequeue), so a class calling `\author` once per
author keeps every creator with its affiliation. A document that never
redefined `\author` is byte-identical (`\@ifnextchar` is never reached).
Residual, documented: with a same-shape class redefinition, a brace group on
the same paragraph right after `\author{…}` reads as the class's affiliation
argument (`\par` stops `\@ifnextchar`). **Guards**:
`perfect_kernel_batch56::{raw_class_author_trailing_keyval_becomes_frontmatter,
raw_class_author_per_author_calls_accumulate_with_affiliations,
same_shape_author_redefinition_absorbs_nothing}`; the ptptex/quantumview
author guards pin the replacing path. **Bindings note**: the els-cas witnesses
actually load the contrib binding `cas_dc_cls.rs` (bindings precede raw), which
inherited inst_support's `\author[]{}` and leaked the same `[keyvals]`; it now
carries cas-common's `\author[marks]{name}[keyvals]` shape through the shared
`\lx@add@author@keyvals` (guard `cas_author_trailing_keyvals_are_frontmatter`).
**Upstream**: not filed (Perl leaks the
same way; a class-shape-aware `\author` would be the upstream fix).

### 254. `\usecounter` is counter-only (latex.ltx:16048); the list starts at `\@trivlist` (Perl: `\usecounter` starts the list)

**Perl** (`latex_constructs.pool.ltxml:1642`) defines `\usecounter{ctr}` as
`beginItemize('list', ctr, nolevel)` and its `\list` (`:1650`) calls
`\usecounter{}` itself when the setup body bound no counter — the list-start
hook lives in `\usecounter`. `beginItemize` does `Let('\item' => '\list@item')`
(`:1312`), so a raw package that uses `\usecounter` for what LaTeX says it
does — set `\@nmbrlist`, bind `\@listctr`, zero the counter — while running
its OWN list machinery has its `\item` clobbered: fancybox.sty:251 `\let\item
\Bitem` + :316 `\Benumerate` → `\usecounter{\@enumctr}`, an `\halign` list
whose items are `\cr`/`&`, rendered three nested `<ltx:item>` inside one
`<ltx:td>` (fancybox-doc.tex:542; 3 schema errors). The mechanism is
Perl-origin (probe: `\let\item\myitem \usecounter{enumi} \item` raises
`malformed:ltx:item` in both engines), but Perl's own fancybox binding never
reaches the raw code (it defines four boxes and errors on every
`B*` environment: 101 errors on the manual), whereas Rust loads fancybox raw
(`fancybox_sty.rs`, `noltxml`). **Rust** (`latex_constructs/sect06.rs`):
`\usecounter` is latex.ltx:16048 verbatim
(`\@nmbrlisttrue\def\@listctr{#1}\setcounter{#1}\z@`); `\list{}{}` runs its
setup body and ends in `\@trivlist` (latex.ltx:15848/15862); `\lx@trivlist@setup`
reads `\@listctr` and calls `begin_itemize("list", ctr, nolevel)` — with
`start` = the counter's current value + 1, so the value the setup body left
(zeroed by the real `\usecounter`, possibly moved by a following
`\setcounter` to continue a list) reaches the items instead of being reset
again. `itemize`/`enumerate`/`description` call `begin_itemize` directly and
are untouched; `enumitem` builds its own lists. **Guards**:
`perfect_kernel_batch56::{raw_halign_list_keeps_its_own_item,
list_setup_counter_value_survives_into_the_items}`, `tests/structure/itemize.tex`
(five `\begin{list}…\usecounter` shapes), the `endlist_*` guards. **Upstream**:
worth filing — a raw `\usecounter` outside `\list` starts a phantom list in
Perl.

### 255. A block captured in a drawing may hoist out of the box — but never out of a list (Perl: stays in the box, reported)

**Perl** places a block-level element that turns up inside a captured box
(`svg:g`/`svg:foreignObject` of a tikz node, an mdframed or tcolorbox skin)
where the model puts it: `adjustBackmatterElement`
(`latex_constructs.pool.ltxml:3936`) and `find_insertion_point`
(`Document.pm:960-1011`) autoclose only `canAutoClose` nodes and a drawing's
`svg:g` is not one, so an `ltx:bibliography` built inside a tikz node stays
there and is reported (`<ltx:bibliography> isn't allowed in <ltx:block>`).
**Rust** (`base_utilities.rs` `insert_block`, batch 56bq) hoists such a
block out of the drawing when no kept node carries text (a graphic in the box
is drawing content and stays): xebaposter's bibliography in a top-level tikz
node floats after the `svg:svg` (guard
`bibliography_in_a_drawing_floats_to_the_document`), juradiss's mdframed
bibliography and a parbox caption reach their figure. **Boundary (56fp)**: the
climb never crosses `ltx:item`/`ltx:inline-item`/`ltx:itemize`/`ltx:enumerate`
/`ltx:description` (or their inline forms). A bibliography arriving alone from
a tcolorbox inside an `\item` (biblatex-ext.tex:1301/1326) had climbed to the
subsection, closed the itemize under the remaining items and nested them (4
schema errors); it now stays in the box and is reported exactly as Perl reports
it, and the list keeps its five siblings. Guard
`perfect_kernel_batch56::bibliography_in_a_list_item_box_keeps_the_list`.
**Upstream**: none (Perl's placement is the conservative one).

### 256. A built document with no root element is a Fatal (Perl: status 2 with a bare XML declaration)

**Perl** writes the 39-byte `<?xml version="1.0" encoding="UTF-8"?>` when
`\begin{document}` was never digested — typically an undefined `\ifX`
auto-`\newif`ed to `\iffalse` (State.pm:532) whose skip ran to EOF and
swallowed the body (`Missing \fi or \else, conditional fell off end`) — and
reports status 2: errors, but "output". xwatermark-guide (its class is
missing, so etoolbox's `\ifdefTF` is undefined) and skeyval-pokayoke2 are the
corpus witnesses; both engines produce the same empty file. **Rust**
(`converter.rs` `note_rootless_document`, at the three build-success sites of
the full-document `convert` path — CLI, corpus harness, cortex_worker; the
editor's in-process fragment fallback is exempt, a snippet without
`\begin{document}` being a legitimate state) raises `Fatal:Document:Malformed`
when the built document has no root element and no Fatal is already on record
(a `TooManyErrors` or resource fuse explains the loss itself), so the run is
status 3 and the loss is named — the messages are the success signal (user rule 2026-09-22: every
Warning/Error/Fatal emitted and counted; an empty output without a Fatal is a
silent whole-document loss). The empty file is still written. Formats that
serialize digested boxes rather than a DOM (TeX/Box) are not checked. Guard
`perfect_kernel_batch56::document_without_a_root_is_a_fatal`. **Upstream**:
worth proposing — Perl's `Fatal` on an empty document would cost nothing.
Only a LaTeX document has a `\begin{document}` to miss, so the Fatal requires a loaded
document class (`document_class_filename`, set when `\documentclass` loads one; batch
56hh). A source that is all comments, like arXiv's `%auto-ignore` withdrawal
placeholders (53 papers of sandbox 2605, e.g. 2605.00131, and 28 of 2606), lost nothing. So does a
plain-TeX file that typesets nothing. Both stay clean, as in Perl. Guard
`comment_only_source_is_not_a_fatal`.

### 257. A `{titlepage}` built after body content is a layout paragraph, not a stranded `<titlepage>` (Perl: the element, schema-invalid)

**Perl**'s `{titlepage}` is `<ltx:titlepage>#body` with the Info "When using
titlepage, Frontmatter will not be well-structured"
(`latex_constructs.pool.ltxml:1167`); the Rust binding is the same constructor
(`sect05.rs`). `document_model` admits `ltx:titlepage` only in the leading
front group, so when body content precedes the environment — a leaked
preamble argument or undefined-command marker (chemexec_en.tex:119, stanli,
l2picfaq, pst-calendar-doc), a hand-typeset cover line (toptesi-it), genuine
prose — both engines emit a stranded `<titlepage>` after the first `<para>`:
`element "titlepage" not allowed here` (verified identical on the minimal
repro). **Rust** (`base_utilities.rs` `demote_stranded_titlepage`, run from the
environment's `after_construct` after the placement pass): when any preceding
root child is neither a resource, a frontmatter-group element nor content-free,
and every child of the titlepage is something `ltx:para` can hold, the element
is renamed in place to `ltx:para` with `class="ltx_titlepage"` — the same text
in the same order, no wrapper the schema forbids there. A titlepage at its
proper leading position keeps its element; one carrying frontmatter children (a
`\maketitle` unwound inside it) is left as Perl leaves it. Refines the #245
control `titlepage_stays_below_a_visible_leading_cover` ("invalid but
faithful" → valid and faithful). Guards
`perfect_kernel_batch56::stranded_titlepage_becomes_a_layout_paragraph`,
`titlepage_stays_below_a_visible_leading_cover`. **Upstream**: worth proposing.

### 258. A leading paragraph holding only undefined-command markers is content-free for the frontmatter hoist (Perl: the frontmatter strands behind it)

**Perl** typesets an undefined control sequence as an `<ltx:ERROR>` marker in
place; when the first such marker precedes `\maketitle` (a preamble command
whose package or engine is missing under `[rawstyles]`: pst-calendar-doc's
`\DeclareDocumentMetadata`, forest-doc's `\@escapeifif`, pmhanguljamo's
`\fontid`), the marker opens the body and every frontmatter element after it
is schema-invalid — or Perl produces no XML at all (7 of the 8 witnesses).
**Rust** (`base_utilities.rs` `text_outside_error_markers`,
`subtree_elements_all_invisible`): for the placement pass (#242/#247) a
`para`/`p` whose only element descendants are `ltx:ERROR class="undefined"` markers
(only that class carries a control-sequence name; `\lx@ERROR` markers and constructor
failures carry arbitrary text and stay visible) and whose text outside them is
whitespace is content-free — the marker's text is the control
sequence NAME, diagnostic ink that pdflatex never typeset — so it is a mover:
the frontmatter is placed above it and the marker follows, unchanged and still
reported as `Error:undefined`. A marker whose arguments were typeset as text
(`\foo{Real words}`) is visible content and the pass declines as before, matching
Perl's placement. Guard
`perfect_kernel_batch56::frontmatter_hoists_over_an_error_marker_only_paragraph`
(with the typeset-argument control). **Upstream**: none (Perl's placement is
conservative; the marker is a LaTeXML artifact either way).

### 259. pgfmath division by zero returns the dividend (Perl: dividend ÷ 0.00001)

**Perl**'s pgfmath accelerator (`pgfmath.code.tex.ltxml:255` `pgfmath_divisor`)
replaces a zero divisor by an epsilon, so `divide(x, 0)` is `x / 0.00001`,
`int(x / 0)` a six-digit integer, `mod(x, 0)` ≈ 0 — with a Warning. Real pgf
(`pgfmathfunctions.basic.code.tex:66-80`, `:271`) divides with TeX's `\divide`,
whose zero divisor sets `arith_error` and leaves the register unchanged
(tex.web §107 `x_over_n`, §1240 `do_register_command`), so `divide(x, 0)` is
`x`, `div`/`int` its integer part, `mod(x, 0)` is `x`, and pdflatex raises the
recoverable "You've asked me to divide 'x' by '0'". **Rust**
(`pgfmath_code_tex.rs` `pgfmath_divide`, used by the infix `/`, `divide`, `div`,
`\pgfmathdivide@` and `mod`) returns the dividend and emits Perl's Warning. The
epsilon value was not harmless: carbohydrates.sty:119-125 seeds a decoration's
step from `len / int(len / segmentlength)` with a segment length that degraded
to 0 (chemfig renamed `\CF@atom@sep`), pdflatex's step is ≈ 1 pt (16 automaton
steps), the epsilon step ≈ 1 sp walked pgf's automaton into the 5,000,000
pushback limit (carbohydrates_en, a pdflatex-clean manual; Perl never draws it).
`reciprocal`, `sec`, `cosec`, `cot` keep the epsilon (pgf: `reciprocal(0)` is 1
by the same mechanism; no witness). Guard
`perfect_kernel_batch56::pgfmath_division_by_zero_returns_the_dividend`.
**Upstream**: worth proposing (the epsilon is the port's invention).

### 260. A space token's character code is 32, an end-of-line space included (Perl: 10)

**Perl** tokenizes an end-of-line inside an argument as a SPACE token whose text
is `"\n"` (Mouth.pm:230, `PRESERVE_NEWLINES`) and `getCharcode` returns
`ord("\n")` = 10 (Token.pm:243), so `\if` (TeX_Logic.pool.ltxml:51) compares it
unequal to a typed space. tex.web §289 defines `space_token = 2^8·spacer + " "`
and §349 makes an end-of-line a space with `cur_chr := " "`: every space token
has character code 32. Character-processing packages depend on it —
stringstrings' `\@DiscardNextChar` (`\if\BlankSpace#1\else\@gobble#1\fi`,
stringstrings.sty:1556) and `\testmatchingchar` — so a `\@for` item that was a
lone newline (tikzviolinplots.sty:201 `\violinsetoptions` over an axis-option
list ending in `,` + newline) took the wrong branch, desynchronised the
conditional stack (`Error:unexpected:\else` ×4) and left the axis state
undefined, after which the KDE loop ran away (`Fatal:Timeout:Recursion`).
Perl fails identically at the character-code level but never runs raw
stringstrings. **Rust** (`token.rs` `get_charcode`): a `Catcode::SPACE` token
reports 32 whatever its text; the text stays `"\n"` (Perl's data model, the
`tex=` reversion). Callers of `get_charcode`: `\if` (tex_logic.rs), inputenc's and
cjk's byte reads, the counter-style char read, forest's grammar-char test; the numeric
backtick read (`read_normal_integer`, gullet.rs) builds its code from the token text
and received the same normalization. `\ifx` was already TeX-correct: `Token::eq`
ignores the text of SPACE tokens. Guards `charcode_tests::end_of_line_space_has_character_code_32`,
`perfect_kernel_batch56::end_of_line_space_compares_equal_to_a_space_in_if`.
**Upstream**: worth proposing to Perl LaTeXML.

### 261. A block or same-kind list arriving in a list before an `\item` gets an auto-opened item (Perl: placed in the list, schema-invalid)

**Perl** `insertBlock` (TeX_Box.pool.ltxml:449-519) renames its capture in place
(:512) without consulting the model's indirect routes, so a box opened in a list
before its first `\item` — framed.sty's `shaded` wrapping an `\item`
(colorframed-doc:176), a minipage of `\item`s beside a table minipage
(tableaux/exemples:71) — becomes `<block>` directly in `<itemize>`, whose model
is `item*`. And `computeIndirectModel` skips a same-tag route (the `$tag ne $kid`
guard, mirrored in `state.rs` `compute_indirect_model`), so an `itemize` opened
directly in an `itemize` (legal LaTeX after an empty item: iitem's `\Pseudo@item`,
lost in qworld because ltjsarticle.cls redefines the kernel `\@item`) is also
placed in the list. Both are SHARED schema errors. **Rust**: `insert_block`
(base_utilities.rs) moves a capture that a list container cannot hold through
`find_insertion_point`, which opens `item` → `para` exactly as for text arriving
before the first `\item`; `find_insertion_point`'s bridge table (document.rs)
gains a row `item → para` for a child whose tag equals the current list's. The
auto-opened item carries no tag; the next `\item` closes it. pdflatex reports
"Something's wrong--perhaps a missing \item" only for the empty-list case and
typesets the material as list content — the item is the closest valid shape.
Witnesses qworld (24 schema errors), colorframed-doc (6), tableaux/exemples (2).
Guards `perfect_kernel_batch56::block_in_a_list_before_an_item_gets_an_auto_item`,
`perfect_kernel_batch56::nested_list_before_an_item_gets_an_auto_item`.

### 262. The frontmatter always opens the document: preamble residue and visible body content before the flush follow it (Perl: placed at the current point, title stranded after the body)

**Perl** places a `\maketitle` flush at the current insertion point
(Base_Utility.pool.ltxml:918) and the fallback at the top; whatever precedes the
current point stays ahead of `<title>`. The document model puts the frontmatter group
before all `document.body.class` content (LaTeXML-structure.rnc:34), so any visible
node there — a paragraph the PREAMBLE typeset (an undefined command's marker and its
argument: engine primitives such as `\mubytein`/`\XeTeXgenerateactualtext`, a missing
sibling file's macros, an unbound class option), prose or a class-drawn cover before
`\maketitle`, a logo — leaves `<title>` stranded after the body: schema-invalid, and
SHARED in the 59 s112 manuals whose first jing error was a frontmatter element.
**Rust** (user-approved surpass, 2026-09-23), two layers in the one placement pass
(`place_frontmatter`, base_utilities.rs): (1) the `\begin{document}` constructor
(latex_constructs/sect02.rs) marks every child of a root the preamble already opened
as PREAMBLE residue (`_preamble`, internal). LaTeX forbids typesetting there
(`\@nodocument`, "Missing \begin{document}"), so residue is never body: the pass moves
it below the frontmatter like content-free nodes and never wraps it as a cover
titlepage. `\begin{document}` still issues no `\par` — preamble text and the first body
words share a paragraph as in TeX. (2) Visible body content before the flush no longer
stops the pass: the frontmatter goes to the head (after the resources and any leading
frontmatter-group run) and that content follows it in its own order. Every flush ends
the paragraph it interrupts, when every open element up to the root auto-closes, as
LaTeX's flush points do (`\maketitle` opens with `\par`, article.cls:203; `\section`;
`\end{document}`) — never a frontmatter container: an open `{titlepage}` stays open for
its `\unwind@titlepage` hook — and a `\maketitle` inside a `\parbox` leaves the box
content in the box.
Reordering is limited to moving the frontmatter group up; no content is dropped
(content_diff.py and golden-PDF recall unchanged on all 59 witnesses). Supersedes the
56ep/56es "never hoist over visible content" controls and the degrade-to-`ltx:text`
placement of a `\maketitle` in a box (now removed as unreachable); generalizes #258
(error-marker-only paragraph). Witnesses: 55 of the 59 s112 frontmatter-first-error
manuals become valid (a 56th, bmstu, with 56gg's heading page-break fix) (ean13isbn,
makebarcode, zwgetfdate, zwpagelayout, a4wide, hebdomon, turnstile ×2, aomart ×2,
beameruserguide, ltnews, l3news, …). Guards
`perfect_kernel_batch56::{preamble_residue_follows_the_frontmatter,
titlepage_around_maketitle_leaves_no_empty_titlepage, frontmatter_leads_a_leading_graphic,
frontmatter_leads_a_leading_framed_empty_box,
frontmatter_hoists_over_an_error_marker_only_paragraph}`,
`perfect_kernel_batch54::maketitle_inside_a_box_goes_to_the_head`; goldens revtex4_endpage
(now valid), aliceblog.

### 263. `\unwind@titlepage` re-homes what it unwraps through the content model (Perl: children spliced in place)

**Perl** (latex_constructs.pool.ltxml:1187-1193) closes the innermost open
`ltx:titlepage` and `unwrapNodes` it: the children are spliced into the parent with no
model check. When a class's own `\maketitle` opens a titlepage inside the document's
`{titlepage}` (jlreq.cls:5501-5527 under `\begin{titlepage}\maketitle\end{titlepage}`),
its empty centered paragraphs land as bare `<ltx:p>` under `<ltx:document>` —
schema-invalid (`element "p" not allowed here`). **Rust** (`sect05.rs`): after the
unwrap, each child the new parent cannot hold is wrapped in the element the model
auto-opens for it there (`can_contain_indirect`: `<p>` → `<para>`), the shape ordinary
paragraph flow gives it; admissible children are untouched. Witnesses
gckanbun/gckanbun-doc and kksymbols/kksymbols-doc (2 schema errors each → 0; Perl
Fatals on both under the pdfTeX identity). Guard
`perfect_kernel_batch56::unwound_titlepage_rehomes_its_paragraphs` (self-skips without
jlreq.cls).

### 264. A page break digested inside a section heading precedes the section (Perl: left between the headings)

**Perl** builds `<ltx:title>` and then `<ltx:toctitle>`; a `\clearpage` executed while
the title is digested — titlesec's `\titleformat{…}{\clearpage…}` runs its format
there (bmstu-iu8 IU8-02-construction.sty:26-27) — floats its `<ltx:pagination>` out of
the title, and it stands between the two headings: `element "toctitle" not allowed
here` (SHARED, verified on the minimal repro). **Rust** (`sect04.rs`
`precede_heading_with_its_pagebreaks`, both section constructors): a pagination found
among the section's leading heading run (tags/title/toctitle, before the last heading)
is moved in front of the section when its parent admits `pagination`, else after the
headings. In TeX the break comes before the heading, so the page-level shape is the
faithful one; a break after the headings (body content) is untouched. Content-free:
nothing visible moves. Witness bmstu-iu8/bmstu-example (4 schema errors → 0). Guard
`perfect_kernel_batch56::pagebreak_in_a_title_format_precedes_the_section`.

### 265. A class's dropped `\maketitle` body is deposited for the fields the frontmatter does not carry (Perl: dropped, the fields lost)

**Perl** locks the kernel `\maketitle` (latex_constructs.pool.ltxml:1099) and drops a
class's redefinition (State.pm:502-517); the title-page fields such a class keeps in
its own macros and prints only in its `\maketitle` body — degree, program, university,
city, subject, teacher — never reach the XML (ryethesis.cls:282, exam-n; SHARED, 0
errors, silent content loss). **Rust**: the lock keeps every dropped MACRO definition
as `\lx@dropped@<name>` (state.rs `install_definition`; `\newcommand`'s own locked
bail, sect08.rs). `\lx@deposit@maketitle` (sect05.rs) runs a captured argument-free
`\maketitle` body — a class's, or a document preamble's own `\renewcommand\maketitle`,
which the lock drops alike — in a group with `\@title`/`\@author`/`\@date`/`\@thanks` nulled (the
frontmatter carries them — no duplication), `\@maketitle` relaxed, not re-entrant, and
keeps the output only when it typesets something (a hard error in the replay still
propagates — Fatal stays Fatal); the replay's layout lands as block paragraphs after the
frontmatter — a class `{titlepage}` builds NO `ltx:titlepage` inside the replay (56gm:
the XSLT drops the document's title block whenever a titlepage exists, taking it to
carry the title, which the nulled replay never does — edmaths and ten other manuals
lost title/author/date from the HTML; `?#fields` in the `{titlepage}` constructor). The
replay also relaxes the SETTERS `\title`/`\author`/`\date` (ukbill.cls:497 typesets
`\textbf{\title}` for `\@title`; memoir's two-argument `\title` ate the replay's braces),
and the stores of a frontmatter ENVIRONMENT the class renewed but the lock kept read
given-and-empty (sect08.rs `record_dropped_environment_stores`: each `\setbox` target of
the dropped bodies an empty box, each `\def`-family target and the conventional
`\@<name>` empty — wkmgr.cls's `\ifvoid\abspagebox` "no abstract" warning, mcmthesis's
`\@abstract` undefined; the environment analogue of K11's `\lx@captured@stores`; a box
store is emptied only if it exists at deposit time). Caveat: a document that gives NO
abstract no longer gets such a class's own "no abstract" warning inside the replay. The
replay's class code met three genuine kernel/binding gaps, fixed at their source:
hyperref's `\NoHyper` macros, `MoveableBox` recognising a void box register by meaning
(expl3 `\box_use:N` = `\copy`), and unicode-math's `version=` key declaring the math
version. After 56gm every one of the 54 deposit-touched manuals has diagnostics at or
below its pre-56gj level. Only a
body whose every control sequence it would run is defined at deposit time is replayed
(`body_vocabulary_is_defined`, sect05.rs; measured on the 20 maketitle-redefining
manuals, arXiv document preambles not yet measured). A derivative class leaning on
internals our binding of its base class lacks (resphilosophica over the amsart binding)
keeps the lock's behaviour: that backfire retired an earlier generic replay. The
arguments of a no-op macro (parameters, empty expansion) are skipped by the scan, since
they are discarded unread. So a body drawing its title page inside eso-pic's
`\AddToShipoutPicture*{…}` replays its flow content, and the undefined tikz in the
picture never runs (batch 56ha: uantwerpendocs' exam `\@extrainfo` rules, the phdthesis
jury, contact and integrity blocks, the bamathesis copyright notice; 6 manuals gain
37-384 words, 0 lost, 0 errors, 0 validity changes over the 544 maketitle-dropping
manuals). The shipout picture itself (exam cover fields, bamathesis title page) stays
lost, as in both engines. A body whose `\maketitle` takes a plain optional argument
(uni-titlepage.sty:97 `\renewcommand*{\maketitle}[1][]`, KOMA's page number) is also
replayed (batch 56he). The kernel `\maketitle` is then a dispatcher that reads the
`[<options>]` for it (`\iflx@maketitle@options`, `\lx@maketitle@withopt`), so
they no longer leak as a paragraph. Bindings extend `\lx@maketitle@body`, not
`\maketitle`, because an appended token would stand between the peek and the `[`. Inside
both deposits the frontmatter setters fill the field a class body reads
(`\lx@deposit@setters`: `\title{…}` → `\gdef\@title{…}`, an optional `[…]`
skipped, a bare `\title` nothing, as the former `\relax`). The setters are
`\let`, because `\title` is locked and the lock ignores a `\def`. KOMA's re-targeted
setters join through `\g@addto@macro\lx@deposit@setters`. uni-titlepage's
WWUM/DHBW/UKoLa recall goes 51.4/64.0/77.8 → 100/100/100, 0 errors. Label text next to a nulled field stays ("by"), as it is in the PDF.
Witnesses: exam-n/template-master recall 36.5 → 100, ryethesis/ryesample 89.1 → 93.7;
18 other maketitle-redefining manuals unchanged; 0 errors, 0 validity changes, 0 words
lost. Guards `perfect_kernel_batch56::class_maketitle_body_deposits_its_fields`,
`class_maketitle_titlepage_keeps_the_title_block`,
`class_maketitle_deposit_relaxes_the_setters`,
`class_maketitle_reads_a_dropped_environment_store_as_given`,
`class_maketitle_deposit_skips_a_shipout_picture`,
`class_maketitle_deposit_threads_its_options`.
A replay that raises an error is dropped along with its diagnostics (batch 56hh). The
replay's premise, fields emptied, is a state the class never faces. PoS proceedings'
pos.sty:88 keeps its authors in an expl3 seq that the locked `\author` never fills.
Its `\printAuthors` (:171-190) pops the empty seq inside `\bool_do_until:nn` and
expands the `\q_no_value` quark: "Token \q_no_value expands into itself!" ×2 in 20
papers of arXiv sandbox 2605 (2605.02049) and 154 of 2606 (aaskaiid.sty too, 2606.25043),
where pdflatex and Perl are clean. The
replay runs under a `DiagnosticsHold` (logger.rs). It holds the formatted records;
`commit` writes them. `discard` drops them and restores the report counts, together
with the runaway guards beside them (the consecutive-error tracker, the too-many-errors
latch). So the log and the status agree, as with `IgnoreDiagnosticsScope`, and a
dropped replay cannot trip a cap that would mute the document's own errors. Only
diagnostics are rolled back: the replay's group undoes its local assignments (pos.sty's
seq pops), but a global one it made stays. A kept replay's own
diagnostics are written as before, and a Fatal still propagates after a commit. The
drop is noted as `Info:ignore:\maketitle`. The 10 deposit repros are unchanged.
Guards `class_maketitle_replay_that_errors_is_dropped`, `error.rs`
`diagnostics_hold_commits_or_discards_lines_and_counts_together`.
A K11-rerouted store (`frontmatter_stores.rs`) is nulled in the deposit, so a class default the document never replaced would reach neither the frontmatter nor the body. lion-msc.cls:196-203 is the witness: a default affiliation and address that `\@maketitle` prints. The kernel `\maketitle` therefore hands each unset, non-blank default to its frontmatter setter first (`\lx@store@defaults` in `\lx@maketitle@body`, batch 56ip). It runs only at `\maketitle`, where pdflatex prints the default. A setter the document does call replaces the default, and a second `\maketitle` finds the store set. The list of rerouted stores accumulates across a raw class and a raw class it `\LoadClass`es. Guard `class_census::k11_class_default_store`.
A K11 store that annotates a creator (an affiliation, address, e-mail, ORCID) or makes one (an
editor, a speaker) is read once, at `\maketitle`, where the author list exists (`harvest_stores`;
at the frontmatter fallback without a `\maketitle`; set later, it is handed on at once): a store set
twice gives its last value, an emptied one none, one set before `\author` meets the author list
(handed on when set, its affiliation found no creator and was dropped), and the annotating stores
go before the creator-making ones, so they stay the authors'. An affiliation store is printed once
under the whole author list, so it annotates every author (`annotate=all`). With no creator, an
annotating store is not handed on and the deposited `\@maketitle` keeps printing it
(courseoutline.cls:150); with no author but an editor or speaker store, the creator-making stores
go first, so the affiliation annotates that creator. A store set after `\maketitle` hands on the
value just given (an accumulating `\g@addto@macro` store is not repeated). Superscript marks in an
affiliation store label its entries only when the authors carry marks to match; otherwise the
entries annotate every author with the marks kept as text. A class default is read only by
`\maketitle`: without one, only the stores the document set are handed on, read at the frontmatter
fallback before its queue is digested (eager and streaming alike). Every other store is
handed on as it is set, as before (keywords set after
`\maketitle` for an abstract that prints them, pittetd.cls:503-527; an abstract at its place). The
deposit nulls exactly the stores handed on. An affiliation store whose value typesets nothing
(umich-thesis.cls:71, a warning) leaves no element; other kinds still leave an empty one (RED
`store_default_that_typesets_nothing_leaves_no_element`). The table also takes an
`[optional]{mandatory}` setter storing `#2` (bfhlayout.sty:738; an argument read up to a delimiter
is not a store), and `\department` (an affiliation). Witnesses: bfh-ci DEMO-BFHThesis (recall 83.0 →
92.5 %, both authors), york-thesis (`GRADUATE PROGRAM IN …`, :221), ndsu-thesis-2022 (department
moves to the affiliation), disser templates (`\institution` before `\author`); courseoutline
Outline unchanged. Guards `perfect_kernel_batch58::{department_and_institute_stores_reach_the_frontmatter,
store_set_twice_keeps_its_last_value, store_set_before_the_author_reaches_it,
store_affiliations_reach_every_author, store_without_an_author_stays_the_class_s,
store_set_after_maketitle_is_kept, editor_store_leaves_affiliations_to_the_authors,
store_default_needs_a_maketitle, accumulating_store_set_late_adds_its_value,
speaker_store_carries_the_affiliation, marked_store_affiliations_without_author_marks,
marked_store_affiliations_match_without_maketitle}`. Open: a late
affiliation store (after `\maketitle`) also annotates an editor already made, and adds to the
title page's value where the class prints only the first; an editor store whose class default
typesets nothing (`\relax`) makes an empty editor that takes the affiliation (mg58l: the same editor,
the affiliation lost); a `\section` before `\maketitle` reads the stores at the fallback, so the
later `\maketitle` hands on no class default (it stays in the deposited title page, as in pdflatex's
order).

The replay gate (`body_vocabulary_is_defined`, sect05.rs), since 59p: a `\newif` switch's dead branch is not checked
(uantwerpencoursetext.cls:474-477 `\if@copyright\backgroundsetup{…}\fi`: `\backgroundsetup` exists only under the
`copyright` option, and the rejection lost the CONFIDENTIAL notice) — `\unless` inverts the switch, and a switch that is
an assignment's operand (`\let\ifX\iftrue`, `\newif\ifX`) or that the body sets before testing it (`\Xtrue…\ifX`) is
checked on both branches (`live_branches`); a `\begin{E}…\end{E}` is one unit only for an environment whose binding marks
it `replay_gate_scoped_vocabulary:E` (tikz: `tikzpicture`; uantwerpenletter.cls:285-292, `\path` exists only inside it) —
any other environment's body is checked, `{titlepage}` and `{center}` included; the arguments of a definition marked
`replay_gate_skips_arguments:<cs>` are left to its own diagnostics hold (eso-pic's one-shot overlay, #413); a rejection
logs an `Info` naming the undefined control sequence. A deposit or replay is kept only when it typesets content — a
non-space character or an image anywhere in it (`typesets_content`; a whatsit's string is its reversion, so KOMA's,
boek's and rapport's emptied title pages, `\noindent\par\null`, read as text and left empty paragraphs). Both deposits
also empty titling's `\thetitle`/`\theauthor`/`\thedate` (a body that uses them typed the title twice). A replay
discarded for its errors takes back the error stubs standing in for them (`DiagnosticsHold::discard`,
`state::remove_error_stub`: kept, the document's own later use of the command was an uncounted `<ERROR>`). The groups
it leaves open stay open (RED `discarded_maketitle_replay_leaves_its_groups_open`, utexasthesis): closing them is a dead
end — oegatb.cls:160-176 leaves `spacing` open on purpose for `\AtEndDocument{\EndTextFormat}`. Guards
`perfect_kernel_batch59::{maketitle_replay_keeps_its_title_page_content, maketitle_replay_adds_nothing_the_frontmatter_has,
discarded_replay_takes_back_its_stubs}`.

### 266. A recatcoded 8-bit input byte is decoded where it enters, through its own inputenc declaration (Perl: the font map's upper half, applied to every character)

**TeX** has no Unicode: every character that reaches a font is a slot of its
encoding. inputenc makes each upper byte active (`\DeclareInputText{199}{\CYRZ}`,
cp1251.def:54) and even a text composite is a slot (`\DeclareTextComposite{\"}{T1}{y}
{184}`, t1enc.def:219, built by the `\lccode` trick of latex.ltx:9949). A package that
recatcodes the bytes to letters (russ.sty:58-63, so Cyrillic words can name commands)
sends byte 199 straight to the font, where T2A's letter block — laid out like cp1251 —
shows `З`. **Perl** (`FontDecodeString`, Package.pm:2895-2916) maps codes 128–255 through
the font map whenever the input encoding is 8-bit and the map is full. That reading is
ambiguous in a Unicode-native engine: codes 128–255 also arrive as Unicode from our own
bindings (`\flqq` → `«` U+00AB, the German `"`-shorthand's `ä`/`ß`, accent composites
`\"y` → `ÿ` U+00FF) and T1 puts `ń`/`ż`/`ß` in exactly those slots — tried here, it
turned l2tabu's »…« into ż…ń. **Rust**: the byte is decoded where its provenance is
known, in the mouth (`mouth.rs` `eight_bit_input_char`): a character 0x80–0xFF that IS
an input byte — on a file/content line that is not UTF-8 (so decoded as its bytes), or a
`^^xx` code (tex.web §355; its position is remembered for the line, so a peek and re-read
keeps it a byte) — never an engine string, never a character of a valid UTF-8 line (an
`\input` UTF-8 file keeps its `é` inside a cp1251 document), with catcode letter/other,
under a declared 8-bit `INPUT_ENCODING` and outside the pdfTeX byte mouth, becomes the
character its own inputenc declaration produces — the active definition followed to its
LICR name, then LaTeX's text-command dispatch `\<encoding><name>` for the font encoding
current when the byte is READ (TeX resolves it when the letter is typeset; the two agree
unless the encoding changes between). The catcode stays the byte's, so recatcoded
Cyrillic command names stay consistent. Downstream every code is Unicode, the font map
keeps its lower half only (unchanged), and no token carries provenance. Limitation:
tables keyed by character code (`\uccode`/`\lccode`, `\sfcode`, `\mathcode`, a `` \ifnum`З ``
comparison) now see the Unicode code point, where the pdfTeX setup keys them by byte —
`\MakeUppercase` over recatcoded Cyrillic keeps the case (before, it upper-cased the
mojibake). Rejected: a `composed` flag on `Token`
(the most primitive structure, for a problem that is the input's); composites emitting
TeX slots (faithful for composites, but binding-produced Unicode stays ambiguous).
Divergence from pdflatex: a latin1 byte 0xFF deliberately recatcoded to a letter under
T1 gives the typed `ÿ` where pdflatex prints slot 255 `ß`. Witness russ/russ_doc
(recall 69.9 → 82.7; `Çäðàâåé`-style mojibake → Cyrillic); 150 other 8-bit-input corpus
manuals unchanged. Guard `perfect_kernel_batch56::recatcoded_input_bytes_decode_through_their_declaration`
(its UTF-8 control line reads `Cafй na й.` without the line-provenance gate).
The raw-line readers decode the same way (`Mouth::read_raw_line_decoded`, `decode_eight_bit`): the `.bib` reader and, since batch 56ir, `{verbatim}` and verbatim.sty bodies. Their text reaches the output as typeset, including a first line that a wrapper's optional-argument check pushed back. Readers that WRITE a file (filecontents, `VerbatimOut`, `\writeverbatim`: `capture_raw_lines_until`) stay byte-exact, as TeX writes the bytes and decodes them when the file is read. Guard `perfect_kernel_batch56::verbatim_decodes_the_input_encoding`.

### 267. `\everypar` runs in front of the character that starts a paragraph (Perl: `\everypar` never fires)

**TeX** (tex.web §1090-1091): a character in vertical mode is backed up (`back_input`)
and `new_graf` inserts `\everypar` in front of it, so the hook reads the very token that
started the paragraph, its catcode already fixed. **Perl** declares the register
(TeX_Paragraph.pool.ltxml:42) and never fires it. **Rust** fires it at `new_graf`
(`stomach.rs` `fire_everypar`, for algorithm2e's `\nl` & co.), but digested it as an
isolated list BESIDE the already-absorbed character, so a hook that reads the paragraph's
first token found nothing: syntax.sty:264-274's grammar (`\everypar{…\catcode`\<\active
\gr@implitem}`, `\gr@implitem<#1> #2 `) typeset `¡ab¿` and its delimiter matched past the
line. Now a LETTER/OTHER character that starts a paragraph is backed up and `\everypar`
unread in front of it (`back_input_for_new_graf`); other paragraph starters
(constructors, `\leavevmode`) keep the in-place firing. Guard
`perfect_kernel_batch56::everypar_reads_the_token_that_started_the_paragraph`.

The list fired is the register LaTeX's paragraph hook runs, not what `\everypar` means now:
latex.ltx:9070 `\newtoks\everypar` and :9072-9077 name it by number in
`\g__para_standard_everypar_tl`. `latex_constructs_rust_only.rs` captures it as
`\lx@para@everypar`, which `pending_everypar` reads; outside a LaTeX document body nothing fires.
`\begin{document}` clears `\everypar` through its meaning, as latex.ltx:9498 does. A package's
`\def\everypar` (rlbicig.sty:62-64), a `\let\everypar\othertoks` in the body, and a preamble
chain through another register (arabicore.sty:123-128, rlbabel.def:121-125; no corpus witness yet) all
keep the paragraph list as pdflatex does (KNOWN_PERL_ERRORS #415). Guards
`perfect_kernel_batch58::{everypar_redefined_by_a_package_still_fires,
everypar_let_to_another_register_keeps_the_list,
everypar_chained_in_the_preamble_survives_the_document}`.

### 268. Resetting a counter skips a `UN` companion that was never allocated (Perl: assigns it, "not a register")

LaTeXML pairs each `\newcounter` counter with `\c@UN<ctr>` for unnumbered-item ids, and a
reset list names both (Package.pm:674). A counter allocated with a raw `\newcount`
(doc.sty:870 `\c@CodelineNo`) has no companion, but l3doc.cls:463
`\@addtoreset{CodelineNo}{part}` still resets it at every `\part`; Perl `ResetCounter`
(Package.pm:886) assigns the companion unconditionally and warns "\c@UNCodelineNo is
not a register" (Rust did the same; ltx-talk-code, 20 warnings once 56gj's deposit ran
its `\part`). Rust resets only an existing companion — the existence test Perl itself
applies in `RefStepID` (Package.pm:863). Guard
`perfect_kernel_batch56::reset_list_skips_a_missing_un_companion`.

### 269. The part internals `\@part`/`\@spart` are locked sectioning hooks (Perl: not defined; `\part` unlocked)

Perl leaves `\part` unlocked ("sometimes redefined as partition", latex_constructs.pool
.ltxml:558 — exam.cls:3303's `\def\part` is a question part) and defines no `\@part`; it
never loads KOMA-Script raw (OmniBus → article). Under raw class loading KOMA's
`\@namedef{part}{\scr@startpart{part}}` therefore replaced our `\part` and typeset
`<p><text font="sansserif bold">Part I</text></p>` — no `ltx:part` in scrartcl/scrbook/
scrreprt/cnltx-doc/toptesi documents (Axis 2b: `\part` 59.5 % coverage on s114). KOMA still
dispatches through LaTeX's part internals (`\SecDef\@part\@spart`, scrartcl.cls:4884,
its own `\@part` = `\scr@@startpart`, :4069-4071), as article.cls does (`\secdef\@part\@spart`,
:273-294). **Rust** defines `\@part[#1]#2` / `\@spart#1` → `\@startsection{part}{-1}…`,
LOCKED like `\section` (latex_constructs_rust_only.rs): the class's typesetting `\@part` is
dropped and the part is an `ltx:part`; `\part` stays unlocked, so exam's question parts are
untouched. Residual: ctex's localized part label ("第一部分") becomes the standard "Part I"
tag (beautybook-cn, pgfornament-han). 24 of the 31 part-short s114 manuals gain their
parts, validity unchanged (27/31), no diagnostic change. Guard
`perfect_kernel_batch56::koma_part_is_a_part`.

### 270. Beamer lists: an overlay spec is consumed, not acted on, and enumerate's lone `[template]` is kept (Perl: overlays wrapped in `actionenv`; a lone template dropped)

**Perl** (beamer.cls.ltxml:1110-1179): `beginBeamerItemize` starts the kernel item machinery and routes `\item` through `\beamer@item`, which wraps an overlaid item in `\begin{actionenv}<spec>` → `uncoverenv` → `<ltx:inline-block class="ltx_covered">`. `{enumerate} [BeamerAngled] OptionalUndigested` reads the first `[…]` unconditionally as the overlay (`readOptional` + reparse), so a sole `\begin{enumerate}[(a)]` loses its template and renders `1.` (pdflatex: `(a)`).
**Rust** (batch 56gu, `beamer_cls.rs`): the same item machinery and `\item` routing (`\beamer@item` consumes `\item<…>`, `\item[x]<…>`, `\item<…>[x]`), but the overlay is not acted on — the static output shows every item, as beamer's handout mode does (Rust's `*env` overlay environments are no-ops; no `ltx_covered` wrapper). Enumerate takes two undigested optionals and applies the first that is not an overlay as the label template, so `[(a)]` and `[<+->][(a)]` both give `(a)`, `[<+->]` alone keeps `1.` — pdflatex's labels. Before 56gu the list environments had no item machinery at all (one tagless item per list, specs leaked as `¡2-¿`).
**Guard**: `perfect_kernel_batch56::beamer_list_items_open_their_own_item`.
### 271. `ltx:quote` admits `para`, `logical-block` and `sectional-block` (Perl's schema: `Block.model` only)

**Perl** (`LaTeXML-block.rnc` `quote_model = Block.model`, with the upstream comment "should be Block.model, perhaps even Para.model?"): a quote holds bare `ltx:p` and block-level items but no paragraph or box container. The trees are the same in both engines. A minipage in a quote holding `\tableofcontents` becomes a `logical-block` (webquiz), and one holding `\section*` becomes a `sectional-block` (aguplus); both are schema-invalid. A `\noindent` paragraph in a quote loses its `ltx:para` and `ltx_noindent` class, because `openElement` auto-closes the paragraph container (tagpair/sample).
**Rust** (batch 56gw, user ruling 2026-09-23): `quote_model = (Block.class | Misc.class | Meta.class | para | logical-block | sectional-block)*`, set in `LaTeXML-block.rnc`/`.rng` and on the flattened `ltx:quote` line of `LaTeXML.model`. It is `Block.model` plus the three `Para.model` containers that caused the invalidity, not `Para.model` itself: taken literally, `Para.model` would drop `p`, `equation` and lists as direct quote children. Floats still escape a quote (they float to the page), and theorem, proof, TOC and bare sectioning units stay out; a `\section*` in a quote arrives inside its minipage's `sectional-block`. This is a pure widening, measured over the 383 quote-bearing s117 manuals: 25 change shape (a `para` is kept inside the quote), 0 lose words (`content_diff.py`), and validity goes from 369 to 371 (webquiz, aguplus) with 0 newly invalid. The precedent is #252. **Guard**: `cluster_schema::quote_holds_paragraph_and_box_blocks`. **Upstream**: worth filing, since it answers the rnc's own open question.
### 272. Meta content in math text floats out of the Math, and an equation's footnotes render in HTML (Perl: stranded in `XMText`; equation-level notes not rendered)

**Perl**: `\footnote`/`\index`/`\gls`/`\nomenclature` inside `\text{}` in math construct in the `ltx:text` that `\text` opens. That element admits them, so their `^` float (`floatToElement`, Document.pm:1052) stops there. `cleanup_XMText` (TeX_Math.pool.ltxml) then unwraps the `ltx:text` and leaves the marker as a direct `XMText` child. `XMText_model = (text | Inline.class | Misc.class)*` has no `Meta.class`, so the document is schema-invalid. The marker is also read into the formula: the footnote enters `text=` (`[b11footnote 1fn]`), and ryethesis's `\nomenclature` entries parse as a factor, `c² × [glossary]`. In an `align` cell, `addColumnToMathFork` (Base_XMath.pool.ltxml:880) clones any non-math element into the main branch's `XMText`, so the fork holds the footnote twice (a `.mf` copy). Separately, the XSLT renders "all of equation_model EXCEPT Meta" (LaTeXML-block-xhtml.xsl), so a footnote at equation level, a bare `\footnote` in `\[…\]` included, never reaches the HTML.
**Rust** (batch 56gx, user ruling 2026-09-23 "move them out of math"):
- `cleanup_math` first runs `float_meta_out_of_math` (`base_utilities.rs`). Every outermost `Meta.class` element (LaTeXML-meta.rnc:17) inside the Math's `XMText` moves, in document order, to just after the Math, at the nearest ancestor level that admits it: the `p` for inline math, the `equation` for a display, the `td` for an align cell. This is where a bare `\footnote` in math floats. A `text`/`XMText` wrapper the move leaves empty is removed, so it does not parse as an empty term.
- `add_column_to_math_fork` does not clone Meta content into the main branch.
- The XSLT's two equation-content selects add `ltx:note`, and an aligned row's own notes render in its right padding cell (`eq-right`'s `notes` parameter).

Measured on the 11 affected s117 manuals: schema errors 7 → 0 (ribbonproofs, sidenotesplus, ryethesis), 0 core-XML words lost, HTML 0 words lost and 9 docs gained (the pdfcomment manuals' equation annotations, LaTeX_for_Undergraduates, ryethesis). **Guards**: `cluster_schema::meta_in_math_text_floats_out_of_the_math`, `footnote_in_display_math_text_floats_to_the_equation`, `footnote_in_display_math_renders_in_html`.
### 273. Delimiter spaces and prefix scanning follow tex.web, not Perl's Gullet/Stomach (KNOWN_PERL_ERRORS #222, #223)

**Perl**: a space in a macro's prefix delimiter swallows every following space (`readMatch`, Gullet.pm:614-617). A `\relax` or a space after `\global`/`\long`/`\outer`/`\protected` clears the prefix (Stomach.pm:212/:234). **Rust**: a delimiter space matches exactly one space token (tex.web §392, batch 56gy). After one of TeX's prefixes, a `\relax` keeps it and a space is skipped (tex.web §1211/§404, batch 56hb), so catoptions' `\protected\relax\def` and `\global\sbox` behave as in pdflatex. The details, triggers and witnesses are in the KPE entries. **Guards**: `perfect_kernel_batch56::prefix_space_delimiter_matches_one_space`, `relax_after_a_prefix_keeps_the_prefix`.
### 274. `\verbatim@start` heads line 1 with a pending command, except `\relax` (Perl: never looks)

**Perl**: `\verbatim@start` = `\lx@verbatim@\verbatim@` (verbatim.sty.ltxml:76) reads raw lines and never inspects the token after it, so verbatimbox's `\verbatim\verbbox@inner` captures `\begin{verbbox}[\footnotesize]`'s bracket as verbatim text. On the witness this also gives 6 errors and empty boxes. **Rust** (batch 56hc, `verbatim_sty.rs` `\verbatim@start` / `read_verbatim_lines`): as verbatim.sty:107-112 does, a pending control sequence goes at the head of line 1's `\verbatim@addtoline` and runs inside `\verbatim@processline`. Per tex.web §506, `\if\noexpand#1\noexpand~` holds only for the active line end. So the bracket is consumed and the size applied (readarray.tex:440, verbatimbox, stackengine). The divergence from verbatim.sty itself is that a pending `\relax` (the `\verbatim@start\relax` idiom: curve2e `{Esempio}`, memoir, digiconfigs) is dropped rather than prepended, because our default processline stringifies line tokens and would print it; in TeX it is a no-op, so the rendering matches. **Guards**: `perfect_kernel_batch50::verbatim_start_relax_idiom`, `perfect_kernel_batch56::verbatim_start_runs_a_prepended_command`.

### 275. titling's pre/post hooks reach the output (Perl: kept but never read)

**Perl** (titling.sty.ltxml:28-33, 81-96, "our `\maketitle` doesn't do anything … we'll just punt"): `\pretitle`/`\posttitle`/`\preauthor`/… store their argument in `\@bspretitle`… and nothing reads it. A class that builds its title page in those hooks loses all of it. lion-msc.cls:232-318 puts the thesis type, degree, major, supervisors, affiliation colophon and abstract there, so lion-msc/minimal had 32.6 % recall, with 0 errors in both engines. **Rust** (batch 56hg, `titling_sty.rs`): the binding defines `\@maketitle` as titling.sty:166-180 does, each field between its pre/post hooks. The kernel `\maketitle` deposits `\@maketitle` with the fields emptied, because they are already in the frontmatter, and keeps the result only when it typesets something. So titling's default hooks, which are layout only (`\begin{center}\LARGE` …), add nothing, while a class's text in the hooks lands after the frontmatter. The page layout around the block (`\newpage\null\vskip 2em\vspace*{\droptitle}` … `\vskip 1.5em`) is left out: it has no content, and `\newpage\null` left a page break and an empty paragraph in every titling document. lion-msc/minimal: 24.3 → 70.3 % (`pdf_recall.py` on the core XML, 0 errors), lion-msc 86.5 → 86.9 %. **Guard**: `perfect_kernel_batch56::titling_hooks_reach_the_output`.

### 276. compsci's own `\code` is restored by a first-aid hook (Perl: doc.sty's identity `\code`, the doc body skipped)

**TeX Live 2025**: doc.sty:623 defines `\code` as the identity (`\@ifundefined{code}{\def\code#1{#1}}{}`), so compsci.sty:510-514's `\newcommand*\code` (verbatim typewriter through url's `\Url`) is refused: pdflatex stops with "Command \code already defined" and writes no PDF. **Perl** skips the redefinition with an Info and never reads the frankenstein manuals' doc bodies (it loads `\DocInput{X.sty}` as definitions). **Rust** reads the doc body as content, as pdflatex does, so with the identity `\code` the manuals EXECUTED the macros they document: `\cs\Wrapquotes` = `\code{\Wrapquotes}` ran titles.sty's quote wrapper, whose `\futurelet` look-ahead looped (`Fatal:Stomach:Recursion`, titles), and abbrevs, attrib, lips and moredefs errored or lost their text. Batch 56hj adds a `file/compsci.sty/after` hook (`latex_constructs_rust_only.rs` §11), the way the kernel's latex2e-first-aid-for-external-files.ltx patches external packages. It restores compsci's `\code`. The file still loads raw, and a manual that `\DocInput`s it still reads it as content: a registered binding would have taken over `\DocInput{compsci.sty}` itself (compsci's own manual fell to 1.5 % recall that way). The shipped PDFs, built before doc.sty took `\code`, show the names verbatim. Twelve frankenstein manuals: errors 204 → 1, warnings 390 → 1; PDF recall abbrevs 0 → 97.3 %, titles 0 → 97.7 %, moredefs 23.2 → 97.8 %, lips 73.9 → 95.1 %, compsci 90.4 → 97.6 %, the rest up by 0.1-2 points. The residual moredefs error (`\In` in a `\begin{macro}{\IfElement...\In}` argument) is a separate doc.sty mechanism. **Guard**: `perfect_kernel_batch56::compsci_code_is_verbatim`.

### 277. A vertical command in horizontal mode runs a redefined `\par` whole before reading its arguments (Perl: the macro's tail leaks into the command's scan)

**TeX** (tex.web §1094 `head_for_vmode`): a vertical command met in horizontal mode is backed up and `\par` is inserted in front of it, so a redefined `\par` macro runs its whole body before the command is read again and scans its arguments. **Perl** `Stomach::leaveHorizontal` runs `invokeToken(T_CS('\par'))`. For a macro `\par` that expands the macro, executes only its first primitive, and leaves the rest of the body in the gullet. The command's own argument scan (`\vskip`'s glue) then reads that rest. In Perl this is mostly harmless. In Rust, where `\everypar` fires (#267), it broke mdwtools syntax.sty grammars. The grammar `\par` (syntax.sty:264-273: `\parshape…\@@par \catcode`\<12 \everypar{…}`) lost `\@@par` to the glue scan ("Missing number"). The glue's stray digit then started a paragraph that fired the re-armed `\everypar`, whose `\gr@implitem` never found its catcode-12 `<` delimiter. That gave 11 errors in the repro, and arXiv 2605.07451 (`\grammarShrink` = `\vspace` between productions) ended in `Fatal:Timeout:PushbackLimit`. **Rust** (batch 56hk, `stomach.rs` `leave_horizontal`): when `\par` is expandable, its whole body is read from a mouth of its own and invoked against the current box list before the command continues. There is no group, so its assignments stand, and `\@@par`'s repack sees the paragraph. A primitive `\par` keeps the former single invocation. The one departure from §1094: a body that reads past its own end (`\@ifnextchar`, an unfilled argument) finds end of input rather than the vertical command behind it. None of the kernel's own `\par` redefinitions (latex.ltx `\@par`, `\@setpar`, verbatim, `\@doendpe`, tabular) does that, and lists keep the primitive `\par`. 2605.07451: 0 errors, 6 warnings (11 before 56go). **Guards**: `perfect_kernel_batch56::everypar_grammar_survives_vertical_material_between_productions`, `everypar_reads_the_token_that_started_the_paragraph`.

### 278. natbib's `\setcitestyle` ignores words it does not know (Perl: keyvals, an unknown word resets the style to authoryear)

**natbib.sty:303-335** walks the `\setcitestyle` list with `\@for` and tests each word against its own set (`round`, `square`, `angle`, `curly`, `semicolon`, `colon`, `comma`, `authoryear`, `numbers`, `super`, and `open=`/`close=`/`aysep=`/`yysep=`/`notesep=`). Any other word does nothing. **Perl** reads the argument as `RequiredKeyVals:natbib`, and its `setCitationStyle` fall-through (natbib.sty.ltxml:402-404) sets `CITE_STYLE` to authoryear for an unknown word. So `\setcitestyle{numbers,sort&compress}`, a package option passed here, loses `numbers`, and `colon` (natbib.sty:315-316, the `;` separator) is not known at all. The Rust port also warned "Encountered unknown KeyVals key" (2606.03886 and three more papers). **Rust** (batch 56ho, `natbib_sty.rs`): the list is split as natbib splits it, and only natbib's own words reach the style, `colon` included. **Guard**: `perfect_kernel_batch56::natbib_setcitestyle_ignores_unknown_words`.

### 279. algorithm2e's `{procedure}`/`{function}` are bound like `{algorithm}` (Perl: raw, the caption lost)

**algorithm2e.sty:2786-2830** opens `algocf@algorithm` for these environments with `algocf@procenvironment` true. Its `\algocf@setcaption` (:2428-2441) lets `\@caption` be `\algocf@caption@proc#1[#2]#3` (:2402), which assumes LaTeX's `\@dblarg` calling convention. Our `\caption` calls `\@caption{<type>}{<text>}` with no `[short]`, so the `[` scan ran to the end of the document (arXiv 2605.00743, 2605.06384: `Fatal:Mouth:EoF`). **Perl** raw-loads these environments too. It reaches the same scan, reports it (14 errors on the repro), and emits a flat float: no caption, and the listing flattened into text. **Rust** (batch 56hu, `algorithm2e_sty.rs`): the four environments share `{algorithm}`'s `DefEnvironment`, with `algocf@procenvironment`/`algocf@func` set and `\caption` let to `\lx@algocf@proccaption`. That caption follows `\algocf@caption@proc` + `\algocf@captionproctext` (:2775-2785):
- the name before the first `(` is declared a function keyword (`\SetKwFunction`, unless `nokwfunc`);
- the caption reads "Procedure Name(args)rest", dropping the parentheses when the arguments are empty;
- a trailing `\label` moves into the caption, as `\@caption@postlabel` does.

A `procnumbered` procedure takes the float counter. Otherwise the float gets an id and a `refnum` tag holding the name, as :2418 makes `\@currentlabel` the name, so `\ref` prints the name, as in pdflatex. **Guard**: `regress_2605_clusters::algorithm2e_procedure_caption_is_bound`; repro `repros/captions-floats/algorithm2e_procedure_caption.tex`.

### 280. `\import`/`\subimport` run the file ungrouped (Perl: a `{…}` around the input)

**import.sty:65-92**: `\@sub@import` closes its own group before `\@import` runs the `\input`/`\include` at the caller's level. The search paths are then restored by plain `\def`. **Perl** (import.sty.ltxml L44-47) wraps the input in `{…}`, so everything the imported file defines is popped at the `}`. That includes its `\newcommand`s (`\subimport{}{macros}` leaves the paper's macros undefined) and the packages it loads, whose loaded-flags are global. arXiv 2605.20598: `\subimport{}{…for_preamble.tex}` loads hyperref, which loads etoolbox. The main file's biblatex then found etoolbox "loaded" but `\newbool` gone, 92 cascade errors, `Fatal:TooManyErrors`. Perl survives only because it cannot load biblatex. **Rust** (batch 56hu, `import_sty.rs`): no group. `\lx@import@save`/`\lx@import@restore` bracket the input with the search paths (a stack, so imports nest), so sibling `\subimport{Chapter/}{…}` calls still start from the base paths (witnesses 2604.09744, 2603.04457). A `standalone` child keeps the bracket `standalone_sty.rs` opens itself (#65). **Guard**: `regress_2605_clusters::subimport_keeps_definitions`; repro `repros/loader/subimport_keeps_definitions.tex`.

### 281. xcolor: a repeat load processes its name sets, and active separators stand for themselves (Perl: neither)

Two gaps in the xcolor binding, both shared with Perl, where pdflatex is clean.
- **Name sets on a repeat load.** xcolor.sty:171-193 (`\XC@declarenames`) makes `dvipsnames`/`svgnames`/`x11names` keys whose code inputs the `.def`, on a repeat load too. Our `DeclareOption!` handlers are cleared to `\relax` by `ProcessOptions`, so the repeat-load recovery (#43) had nothing to run. `\usepackage[dvipsnames]{xcolor}` after tcolorbox had loaded xcolor defined no `Maroon`, and tikz's `\fill[Maroon]` failed its `\color@Maroon` probe: arXiv 2605.28926, one `/tikz/Maroon` unknown-key error per use, Fatal. **Rust** (batch 56hu, `xcolor_sty.rs`): durable `\ds@<names>` handlers are re-asserted after `ProcessOptions`, as `\ds@table` already was.
- **Active separators.** xcolor.sty:104-120 `\XC@edef`/`\XC@mdef` expand a colour name, model or expression with each of `! : - + ; " >` (and a model list's `/`) that is currently active standing for itself. Perl (xcolor.sty.ltxml:561) and our binding expanded it unprotected. Generated tables that make `!` an active `\itshape` turned `red!100.0!black` into font switches: arXiv 2605.30133, 101 "Can't find color named", Fatal. **Rust** (batch 56hu): `xc_expand` lets the active separators to their other-catcode selves in a scoped frame for every colour argument the binding expands.

**Guards**: `regress_2605_clusters::xcolor_reload_loads_dvipsnames`, `xcolor_expression_ignores_an_active_bang`; repros `repros/graphics-tikz/xcolor_reload_dvipsnames_tikz.tex`, `repros/unicode-catcodes/xcolor_active_bang_in_expression.tex`.

### 282. A package an autoload loads inside a group outlives the group (Perl: popped with it)

`def_autoload` (and OmniBus's `\lx@late@usepackage`, and the LaTeX-pool autoload) clears its trigger and sets the package's loaded-flag GLOBALLY. It then hoisted the load's meaning-delta to global so the package lives as long, which is the 1711.11576 fix. Issue #348 later restricted the shared hoist to conditionals, for #65's sibling subfiles, and that restriction reached the autoload callers too. So natbib autoloaded by a `\citep` inside `{\itshape …}` lost `\citep` at the `}`, while its lock (`\citep:locked`) survived. Every later `\citep` was then undefined and could not be redefined: arXiv 2605.08349, 2605.10423, 2605.14513 and 2605.04028, `undefined:\citep` ×100, Fatal. Perl fails the same way (OmniBus.cls.ltxml:49-51; 101 errors + Fatal on 08349 and 14513). **Rust** (batch 56hu, `state.rs`): the autoload callers hoist the whole load with `snapshot_top_frame_keys` + `hoist_top_frame_package_load`: its meanings, and its values, so natbib's citation style does not revert after the group either. The subfile bracket keeps the conditionals-only `hoist_top_frame_conditional_delta` (#65). The snapshot-diff is an approximation. A key the group had already bound is not promoted. Catcode, mathcode and the other code tables are not covered. A value the package sets that the group had not bound yet, such as the current font, would become global. The planned replacement runs the load as if at group level 0: set the group's frames aside for the load and restore them after. **Guard**: `regress_2605_clusters::autoloaded_package_outlives_the_group`; repro `repros/loader/autoload_in_group_survives_the_group.tex`.

### 283. An opt-in `xetex` profile, the counterpart of `luatex` (Perl: no engine personas)

The default persona is pdfTeX-model, and since #220's update it passes iftex's `\RequireXeTeX`. It cannot pass an engine test on l3sys, whose identity was frozen to pdftex when the format was built (expl3-code.tex:7846-7861). Examples:
- fduthesis.cls:69-71 and njuthesis.cls:62-63: `\sys_if_engine_xetex:F{\sys_if_engine_luatex:F{\msg_fatal…}}`;
- exam-zh.cls:22;
- fontspec's gate, inlined in xtufte-common.def:20-24, so no binding reaches it.

Each of these halted (TeX Live class census, 2026-09-24). Flipping the default persona is not an option: l3sys's engine is a single value, so `\sys_if_engine_pdftex` would turn false for the 436 pdflatex classes.

**Rust** (batch 56hx, `latexml_sty/mod.rs`): `\usepackage[xetex]{latexml}` (preload `[…,xetex]latexml.sty`) makes the document XeTeX-authored:
- iftex's `\ifxetex`/`\iftutex` read true and `\ifpdftex` false;
- l3sys reports `xetex` and `xelatex` (`set_l3sys_engine`, shared with `luatex`);
- `\XeTeXversion`/`\XeTeXrevision`, `\Uchar` and the inert inter-character primitives are defined;
- the Unicode-engine behaviours are on (`state::unicode_engine_profile`: non-ASCII letter catcodes, `\char` above 255, no utf8 byte mouth).

fontspec, unicode-math and xeCJK stay bindings, so the raw XeTeX font backends are never reached. Their bindings gained the expl3 internals that classes call directly (`\__fontspec_main_set…font:nn`, `\__um_setmathfont:nn`, the `unicode-math` key family, `\xeCJK_declare_node:n` and friends), and ulem's `\ULC@box`.

Census classes under the profile: xtufte-book, xtufte-handout, fduthesis-en, xduugtp and bitbeamer go from Fatal or errors to 0. fduthesis, njuthesis, exam-zh, xdupgthesis and xduugthesis go from Fatal to converting, with 1-8 residual errors.

`run_doc.sh` routes clean-xelatex-oracle documents to the profile, as it routes lualatex ones to `luatex`. The TL-manual oracle does not yet try xelatex, so that routing is inert there for now.

**Guards**: `class_census::xetex_profile_engine_identity`, `xetex_class_calls_binding_internals`.

### 284. A kernel length keeps latex.ltx's register allocation (Perl: renamed to its CS name)

latex.ltx allocates its lengths with `\newdimen`/`\newskip` (`\columnsep` is `\dimen124`), and the format dump records the allocation. The Rust pools that run after the dump re-`DefRegister!` about 54 of them to give them their default values. Perl's `DefRegisterI` (Package.pm:1346-1348), which `def_register` follows, sets the address to the CS name when no `address`/`allocate` option is given. So `\meaning\columnsep` printed `\columnsep`, where tex.web §1224 prints `\dimen124`. etoolbox's `\ifdefdimen`/`\ifdeflength` split `\meaning` for `\dimen`/`\skip` (etoolbox.sty:421-456), and tudscrbase's length store (tudscrbase.sty:160-170, 435-445) then raised "`\columnsep` is not a defined length" (tudscrartcl, tudscrbook, tudscrposter, tudscrreprt).

**Rust** (batch 56hx):
- `def_register`: a bare re-definition of a register that already has an allocated address keeps that address. The default value is still assigned, as Perl assigns when no address is given.
- `state::lookup_dimension`/`lookup_glue` resolve a CS name to its register's address (`register_value_key`), so name-keyed readers such as `lookup_dimension("\\textwidth")` in the float and graphics code read the live value.

**Guard**: `class_census::kernel_length_is_an_allocated_dimen`.

### 285. PDF output is the default; a source shipping EPS/PS figures keeps DVI (Perl: always DVI)

Perl sets `\pdfoutput=0` (pdfTeX.pool.ltxml:23) and `\ifpdf` FALSE, so every
document takes its DVI branches. pdflatex's own format sets `\pdfoutput=1`
(pdftexconfig.tex), and arXiv compiles a source with pdflatex unless it ships
EPS/PS figures, which AutoTeX runs through latex+dvips (a `\pdfoutput=1` in
the first lines still selects pdflatex). arXiv 2605 (30,079 sources): 95.8 % are
pdflatex ones; 1,692 test `\ifpdf`, 1,646 of them pdflatex papers that took
their DVI branch here; 925 set `\pdfoutput=1` themselves and still saw
`\ifpdf` FALSE. User ruling 2026-09-24 (K6).

**Rust** (batch 56id): `core_interface::establish_pdf_output_mode`, run for
each document where its source context is set, assigns `\pdfoutput` = 1 unless
the source directory tree (a bounded walk: depth 4, 20,000 entries) holds an
`.eps`/`.ps`/`.epsf`/`.epsi` file. The `luatex` profile is always PDF and the
`xetex` profile untouched. A document's own `\pdfoutput=…` still wins. With PDF
output, expl3 selects its pdftex backend (l3backend-pdftex.def), which needs
the pdfTeX primitives `\pdfcolorstackinit`, `\pdfmajorversion`,
`\pdfrunninglinkoff/on` (added; the dumps bind expl3's `\tex_…:D` aliases to
them, expl3-code.tex:674-675). pgf, graphicx and hyperref keep their bindings'
drivers.

**Known limitation**: a source with EPS figures that its author compiles with
pdflatex+epstopdf is taken as DVI, as arXiv's own cue would.

**Guards**: `class_census::ifpdf_follows_pdfoutput`,
`core_interface::tests::postscript_figures_select_dvi_output`.

### 286. The bibliography formatter prints subtitles, a host's publisher and place, and access dates (Perl: dropped)

The `.bib` reader files `subtitle` as `ltx:bib-subtitle` (BibTeX.pool.ltxml:571) and, for
`@inbook`/`@incollection`/`@inproceedings`, the host's `publisher`/`address` under
`ltx:bib-related` (:402-403, :437). Perl's formatter reads neither: MakeBibliography.pm has no
subtitle spec, and its incollection block reads only the direct `ltx:bib-publisher`/`ltx:bib-place`
(:742-744). The structure is captured and then not printed.

**Rust** (batch 56ie, `latexml_post::make_bibliography::get_fmt_spec`, shared by every `.bib`
bibliography): a `ltx:bib-subtitle` spec after each title ("Title. Subtitle."), the incollection
block's `ltx:bib-related/ltx:bib-publisher` and `ltx:bib-related/ltx:bib-place`, and a metadata
line for `ltx:bib-date[@role='accessed']` ("Accessed: …", biblatex's `urldate`). The `.bib` reader
also reads biblatex's field names (`journaltitle`, `location`, `urldate`, `titleaddon`,
`addendum`) for the biblatex binding's no-biber path. Witnesses: 78 biblatex manuals of sweep 120
(biblatex-ieee 75.1 → 78.3 %, biblatex-chicago/cms-notes-sample 66.2 → 74.9 %,
biblatex-apa-test 72.6 → 78.4 % recall). No existing golden carries these fields.

**Guard**: `cluster_cli::whatsinout::biblatex_bib_fields_reach_the_reference_list`.

**Crossref (batch 56jd).** The host publisher/place rows carry Perl's host-row step
`ltx:bib-related[@type][not(../ltx:bib-related[@bibrefs])]` (MakeBibliography.pm:732-734): an
entry with a `crossref` prints "See [parent]" and no host fields, as plain.bst's
`format.incoll.inproc.crossref` does ("In [1], pages 1–10"); the biblatex event rows are gated the
same way. Guard: `bibliography_crossref::crossref_child_sees_its_parent_and_skips_the_host_title`.

### 287. quotchap's quotations are epigraphs where they are written (Perl: lost)

quotchap typesets each `savequote` into a box that its redefined `\chapter` prints at the next
chapter head (quotchap.sty:97-132). `\chapter` is locked in both engines
(latex_constructs.pool.ltxml:557; sect04.rs), so that redefinition is ignored and the box is never
printed: the quotations and their authors are silently lost (quotchap manual, recall 44 %).

**Rust** (batch 56if, `latexml_contrib::quotchap_sty`): the package loads raw, then `savequote` is an
`ltx:quote class='ltx_epigraph ltx_quotchap'` at its source position — just before the chapter it
opens — and `\qauthor` its `ltx_epigraph_source` block (the epigraph binding's structure). The
optional box width is read and dropped.

**Guard**: `class_census::quotchap_savequote_epigraph`.

### 288. NFSS size state: `\fontsize`, `\selectfont` and `\f@size` carry the font size (Perl: fixed at 10)

Perl's `\fontsize{}{}` swallows its arguments and its `\f@size` is a constant `10`
(latex_constructs.pool.ltxml:5167, :5620); `\selectfont` merges family, series and shape only. So a
document's `{\fontsize{24}{28}\selectfont …}` keeps no size, `\f@size` reads 10 after `\large`,
and a raw class whose size commands run `\@setfontsize` (the `.clo` files: size11.clo,
KOMA's scrsize11pt.clo) never changes the text size. typearea then measures its good text width
at 10 pt instead of 10.95 pt and warns "Bad type area settings!" (765 KOMA-Script manuals, sweep
#121). Perl's `\baselinestretch` is the kernel's macro `1` (L1050); ours was a register.

**Rust** (batch 56ih): the kernel's mechanism, latex.ltx:10527-10528, 12585-12601 and 14103-14107.
- `\fontsize` is the kernel's, and `\set@fontsize` records `\f@size`, `\f@baselineskip` and
  `\f@linespread` and arms `\size@update`.
- The next `\selectfont` runs the update once: `\baselineskip`, `\strutbox`, and the current font
  takes `\f@size` points (`\lx@fontsize@merge`, the size half of `\selectfont`'s font choice).
- A `\selectfont` with no pending update (`\bfseries` after `\large`) leaves the size alone, as the
  kernel's deferral does. The update disarms itself before its `\hbox`, not after, so it cannot
  re-enter.
- `\@setfontsize` runs `\fontsize{#2}{#3}\selectfont` inside its `\@typeset@protect` guard.
- The class bindings' size switches (`font => {size => …}`) record `\f@size` through the font merge
  (content.rs `merge_font_ref`, #309) and end in `\selectfont` (dialect.rs).
- A preamble `\@setfontsize\normalsize…` sets `NOMINAL_FONT_SIZE`, as a0poster's binding does, so an
  11pt raw class's body text carries no `fontsize` and `\large`/`\small` read 110 %/91 %.
- `\baselinestretch` is a macro again.
- `\check@mathfonts` is the size half of the kernel's `\glb@settings` (Perl's is a no-op):
  `\tf@size`, `\sf@size` and `\ssf@size` for `\f@size`, from fontmath.ltx's table or
  `\calculate@math@sizes`. It runs at load and after each size update; the kernel runs it at math
  entry. Both come with the dump; without one (`LATEXML_NODUMP`) they, `\ifmath@fonts` and the
  script ratios are defined from latex.ltx (:10467, :10472-10489, :10742-10754). Needed once `\fontsize` reads its arguments: `\@textsuperscript` and the logos'
  `\fontsize\sf@size…` (latex.ltx:17643, :10076) met an undefined `\sf@size` (program-doc,
  uhrzeit-doc, memman in the A/B).

The binding classes still ignore the `10pt`/`11pt`/`12pt` option (Perl too).

**Guards**: `class_census::{fontsize_selectfont_size_state, raw_class_normalsize_nominal}`.

### 289. The bibliography formatter prints URLs where the style does, translators, and biblatex annotations (Perl: "Link", dropped)

Perl's `.bib` reader writes every `url` as `<ltx:bib-url href='…'>Link</ltx:bib-url>`
(BibTeX.pool.ltxml:740-741). MakeBibliography then prints "External Links: Link", so the address
survives only in the `href`. A `translator` is read into `ltx:bib-name[@role='translator']` (:560-562)
and never printed, since its FMT_SPEC has no row for it. biblatex's `annotation` field, the name
biblatex gives BibTeX's `annote`, is not read at all. biblatex's standard styles print all three:
"URL: …", "Trans. by …", and the annotation in annotating styles. Witnesses (sweep #121 recall):
biblatex-apa-test, the biblatex-chicago samples, and docsurvey's 126 annotated entries.

**Rust** (batch 56ii):
- `latexml_post::make_bibliography` shows the address itself in place of "Link" when the
  bibliography's style prints URLs (`style_prints_urls`: biblatex, which the binding now records as
  `bibstyle='biblatex'`, natbib's `plainnat`/`abbrvnat`/`unsrtnat`, plainnat.bst:285
  `format.url`, and abnTeX2's `abntex2-num`/`abntex2-alf`, abntex2-num.bst:993-1001, 57cm). The classic `plain`/`alpha`/`unsrt` keep "Link".
- A "Translated by …" row sits in the shared field block.
- `annotation` reads like `annote`, as a `role=annotation` note.

Only the URL change is style-gated; the translator row and the `annotation` field print for every
style, like the `note`/`annote` row before them (#286's pattern: captured `.bib` content is printed).
Recall on the witnesses: biblatex-apa-test 81.0 → 92.1 %, cms-dates-intro 83.0 → 97.8 %,
cms-dates-sample, cms-trad-sample and cms-notes-sample +2.6 to +3.5 points, docsurvey 78.0 → 92.7 %.

**Guard**: `cluster_cli::whatsinout::bib_urls_translators_and_annotations`.

### 290. versonotes' notes are margin notes where they are written (Perl: lost)

`\versonote{text}` writes its note to the `.aux` (versonotes.sty:42-45), and the next run's
shipout hook sets it on the facing verso page (:217-254). A single pass never reads it back, so in
both engines every note is lost (versonotes/sample, recall 61.5 %: 30 annotations). **Rust** (batch
56ij, `latexml_contrib::versonotes_sty`): the package loads raw, and `\versonote` is `\marginpar`, an
`ltx:note role='margin'` at the point the note is written. **Guard**: `class_census::versonote_is_a_margin_note`.

### 291. `\textcommabelow` is a combining accent (Perl: an `\ooalign` table)

latex.ltx:10097-10100 defines `\textcommabelow` as an `\ooalign` that overprints a raised comma.
Perl keeps that kernel default unless latin10.def is loaded (latin10.def.ltxml:22 then makes it
`DefAccent('\textcommabelow', "\x{0326}", ",")`). So `Mari\textcommabelow{s}`, BibTeX's usual
spelling of `ș`, becomes a two-row `<ltx:tabular>` whose text reads "Maris,". **Rust** (batch 56il,
`latex_constructs_rust_only.rs` §12) installs latin10's accent for every document:
`\textcommabelow{s}` is `ș` (U+0219 after NFC), and `ț`, `Ș` and `Ț` likewise. Witnesses: arXiv
2605.08338, 2605.21804. **Guard**: `perfect_kernel_batch56::fontenc_keeps_preloaded_encoding`.

### 292. nlctuserguide defines its glossary entries in the run (Perl: none defined)

Nicola Talbot's user guides (glossaries, glossaries-extra, datatool, mfirstuc, tracklang, …) declare
every command, option and term in one `\nlctuserguidegls{…}` block. The writer macros serialize each
item as a bib2gls record into `\jobname-gls.bib` (nlctuserguide.sty:2883-2892, 2928-3016), and
`\nlctuserguideloadgls` (:3062-3160) inputs the `.glstex` that the external bib2gls writes. TeX Live
ships no `.glstex`, and neither engine runs bib2gls, so no entry was ever defined: the command
summaries, term lists and every `\gls`/`\idx` reference text were lost (Perl has no binding).
**Rust** (batch 56il, `latexml_contrib::nlctuserguide_sty`) redefines the two writers to define the
entries in the run, as the `.glstex` would with its `\newglossaryentry`/`\newabbreviation` calls. The
record type becomes the category, the glossary follows the resource's `entry-type-aliases`, parents
are defined before children, and the icon entries come from `\symboldefinitions`. The glossary
definitions are emitted at `\begin{document}`, once every entry exists (#293), as the `.glstex`
defines everything before anything is typeset: a description may name an entry defined after it. The glossaries binding no longer stubs glossaries-extra's label-prefix
commands (glossaries-extra-bib2gls.sty:430-452), through which `\dgls`, and so `\idx`, resolves a
label. Recall against the shipped PDFs (LuaTeX persona, sweep #121 → 56il): glossaries-extra-manual
50.9 → 86.6 %, datatool-user 67.4 → 90.6 %, glossaries-user 68.0 → 91.1 %, mfirstuc-manual 79.3 → 96.9 %.
**Guard**: `class_census::nlctuserguide_entries_defined_in_run`.

### 293. glossaries: preamble entries are emitted at `\begin{document}` (Perl: at definition)

Perl's binding (glossaries.sty.ltxml:52-97) emits each entry's `<ltx:glossarydefinition>` from
`\@newglossaryentryposthook`, digesting its fields when the entry is defined. LaTeX typesets a field
only in `\printglossary`, once every entry exists. So a field that names a later entry, such as
`\newacronym{endc}{EN-DC}{E-UTRAN-\gls{nr} \gls{dc}}` before `\newacronym{nr}…`, raised "Glossary entry
`nr' has not been defined" in Perl and in Rust and lost its text. A `\gls` inside a field also runs
glossaries' `\ifglshasshort`/`\ifglshaslong` (glossaries.sty:2166-2180), which `\letcs` `\@glo@short`/
`\@glo@long` to the referenced entry's values, so the entry being defined carried them: endc's long
form read "Dual Connectivity". **Rust** (batch 56il, glossaries_sty.rs) snapshots the field values in
the posthook. For an entry defined in the preamble it queues the definition
(`\iflx@glossaries@defer`) and emits the queue at `\begin{document}`; that is where Perl places
preamble definitions anyway. Entries defined in the body are emitted at once. arXiv 2605 (all 281
glossaries papers): errors 854 → 774, 7 papers better status, none worse; LEDGER 2026-09-25, 56il.
**Guard**: `class_census::glossaries_preamble_forward_reference`.

### 294. siunitx v3 keys that change the printed number (Perl: not handled)

Perl's siunitx binding predates siunitx v3. It registers only the v2 key vocabulary
(siunitx.sty.ltxml:38-54) and lists `retain-unity-mantissa` and the rounding keys as "NOT YET
HANDLED" (:302-306), so v3 documents lost settings that change the printed number.
`\sisetup{locale=DE}\num{3.14}` gives "3,14" under pdflatex but "3.14" in Perl. **Rust** (batch
56in, siunitx_sty.rs):
- `locale` sets the decimal marker and the exponent and inter-unit products from siunitx's
  table (siunitx.sty:4956-5013; BR/FR/IT/SI, DE, PL, ZA, UK/US).
- `drop-zero-decimal` (and its deprecated form `zero-decimal-to-integer`) drops an all-zero
  decimal part unless an uncertainty follows; `minimum-decimal-digits` pads the decimal part.
  Both act on the value, never the exponent, in siunitx's order (:1000-1001).
- `uncertainty-mode` maps onto `separate-uncertainty` (:8676-8689).
- `retain-unity-mantissa = false` (v3 `print-unity-mantissa`) prints `\num{1e3}` as 10³.
- The v3 names of v2 keys are routed to them (`print-zero-exponent`, `print-zero-integer`,
  `drop-uncertainty`, `bracket-ambiguous-numbers`).
- The font-matching keys (`reset-text-series`, `text-series-to-math`, `propagate-math-font`, …)
  are accepted and ignored, as `reset-text-family` already was.

Keys that siunitx v3 itself removed or never had (`load-configurations`, `obeybold`, …) still
warn: pdflatex warns or errors on them too. The uncertainty conversions are Perl's, ported
(siunitx.sty.ltxml:329-375), with one difference: an integer value's separate uncertainty prints
as `123 ± 45`, as siunitx does, where Perl prints `123 ± 45.`. **Guard**: `class_census::siunitx_v3_keys`.

---
### 295. A macro call that does not match its `\def` is reported and ignored (Perl: reported, then expanded)

A `\def` parameter text's leading delimiter (`\def\lp\x{…}`) must be the next thing at a call.
When it is not, TeX reports "Use of \lp doesn't match its definition" and abandons the call
(tex.web §397-398); the macro does not expand. **Perl** reports "Missing argument" (Parameter.pm
:93-97), but `readArguments` goes on and the macro expands anyway. **Rust** (batch 56iq,
parameter.rs `Parameter::read`, expandable.rs) reports the miss with Perl's message and then
abandons the call, as TeX does. The macro expanding anyway had looped a self-calling macro to the
digestion fuse (frankenstein/titles via compsci's `\cs\def`). Before this batch the miss was
silent, which swallowed every expl3 expandable error: those go through `\???`, a `?`-delimited
macro (expl3-code.tex:11596-11606). A leading delimiter missed because an isolated token list ran
out (an index phrase, a constructor argument) is the artificial end of that list, not an improper
call: real TeX would read on. It stays quiet and the call proceeds (manyind.sty:98).

Measured: the 3,003-paper arXiv sample changes in one paper. 2605.24084 gains 51 errors with
identical words: `dot spacing/.store in=\dot@spacing` without `\makeatletter` redefines `\dot`,
and pdflatex reports the same mismatches. Of 29 TeX Live manuals with swallowed misses, 20 are
unchanged, and greektonoi, bfh-ci's SciPoster and guitar gain errors. euclideangeometry-man (curve2e's `\MV@c`),
chinesechess (a removed l3draw API) and mercatormap reach the 100-error limit, as pdflatex and
lualatex do. **Guard**: `perfect_kernel_batch56::macro_delimiter_mismatch_ignores_the_call`.

---
### 296. biber's `%` comments inside a `.bib` entry (Perl: the entry is lost)

biber's reader takes `%` to the end of the line as a comment between an entry's tokens
(`@Book{a2004,% see also 14.110`); bibtex 0.99d does not ("You're missing a field name"), and
neither does Perl's `skipWhite` (LaTeXML/lib/LaTeXML/Pre/BibTeX.pm:320-330), which loses the entry.
**Rust** (batch 56ir, pre_bibtex.rs `skip_white`): when the document uses biblatex (`BIBSTYLE` =
`biblatex`, or `biblatex.sty` loaded, which a split post session has from its preloads), `%` to
the end of the line is skipped there. A `%` inside a field value is untouched: values are read by
`parse_string`/`parse_balanced_braces`, which never skip white. It applies to biblatex whatever
its `backend=` option says. windycity recovers 27 entries; biblatex-iso690 and frankenstein recover
some too. **Guard**: `perfect_kernel_batch56::biber_bib_percent_comments_keep_the_entry`.

---
### 297. An inline pstricks framed box is framed text (Perl: an auto-opened picture swallows the paragraph)

Perl's `\psframebox` family always builds `<ltx:g framed='true'>` (pstricks_support.sty.ltxml:955-980).
Outside a `{pspicture}` that auto-opens an `ltx:picture` in the paragraph, and the rest of the
paragraph goes inside it. Each further box nests one level deeper. The post stage renders an unsized
SVG in which the text overprints and cannot wrap. **Rust** (batch 56ir, pstricks_sty.rs) keeps the
`ltx:g` inside a `{pspicture}` (the `\rput` / `\psframebox` drawings of ffslides). In running text
it emits `<ltx:text framed='rectangle'>`, an inline bordered box, as pdflatex draws it (pst-poker-doc,
sesamath-doc-fr, pst-pdf-example). Circle, oval, shadow and double frames are drawn as a rectangle
in running text. Fill and stroke colours are not modelled, and neither is `fillframe`, whose SVG
filter is not ported. **Guard**: `perfect_kernel_batch56::psframebox_keeps_its_body`.

---
### 298. `\rule[raise]` sizes its box as latex.ltx does (Perl: the raise is ignored)

Perl stores `\rule`'s Dimensions as its width and height (latex_constructs.pool.ltxml:4797-4799) and
ignores the raise. latex.ltx:16360-16368 `\@rule` builds `\vrule` with height h + raise and depth
−raise; hpack floors each at 0 (tex.web §653). **Rust** (batch 56is, sect12.rs) sizes the box that
way: `\rule[-2pt]{1pt}{1cm}` measures 26.45274pt high and 2.0pt deep, and `\rule[-2cm]{1pt}{1cm}`
measures 0pt high and 56.9055pt deep, as pdflatex prints. The attribute
strings are unchanged. Before this batch Rust stored only the strings, and the box measured 0×0.
bfhsciposter.cls:171's `\box_gresize_to_ht_plus_dp` then divided by zero (bfh-ci SciPoster, 11
errors). **Guard**: `perfect_kernel_batch56::rule_box_has_its_size`.

---
### 299. Bibliography given names follow the style; biblatex's second-tier fields are printed (Perl: always initials, fields dropped)

Perl's MakeBibliography always prints given names as initials (MakeBibliography.pm:555-566; its note
at :557 says this should be an option). biblatex's title units, host titles and events have no rows
there, so `subtitle`, `titleaddon`, `booksubtitle`, `maintitle`, `eventtitle`, `venue` and
`eventdate` are dropped. Witnesses: the biblatex-apa, biblatex-chicago (cms-*-sample) and fiwi
manuals, spines, and biblatex-examples.bib `salam`.

**Rust** (batch 56iu, round-12 P2/P3):
- **The given-name form comes from the style.**
  - **BibTeX:** the `.bst`'s first literal `format.name$` template with an `f` piece decides: `ff` is
    full, a single `f` is initials. Templates with no given-name piece are skipped, and so are calls
    whose result is compared with `"others"` (the "and others" test; achemso.bst:514, biochem.bst).
    A template held in a variable, or a missing `.bst`, falls back to Perl's initials. A survey of
    TL 2025 found achemso and biochem are the only two changed by the `"others"` rule.
    The abnTeX family (abntex2-*.bst, unbtex-*.bst, fcavtex.bst) reads `{, ff}` and abbreviates in
    its own code: its `#0`/`#1 'abnt.full.initials :=` default (abntex2-num.bst:2045) decides
    first (57cm.1; unbtex-example "L. C. Araujo", the PDF's "ARAUJO, L. C.").
  - **biblatex:** the resolved `author` name format decides first. A format that tests
    `\ifgiveninits` follows the option; otherwise `\namepartgiveni` is initials and `\namepartgiven`
    is full. Seeds and aliases follow biblatex.def:953, 989-993 and biblatex.sty:4494-4545. The
    `giveninits` option comes next; `initsonly` is always initials.
  - **Native styles:** for a style whose `.bbx` the binding does not load, its top-level
    `\ExecuteBibliographyOptions` and `\RequireBibliographyStyle` are still read, in file order
    (ieee.bbx:26-34 `giveninits`, reached from ieee-alphabetic.bbx:13).
- **Name lists:** they use Perl's separator rule, "A. X, B. Y, et al." (MakeBibliography.pm:568-584).
- **Second-tier fields:**
  - biblatex's subtitle and title addon are printed as two units, for the title, the book title and
    the main title.
  - The units are joined as `\newunit` joins them: ". " (biblatex.def:173), or " " after a unit that
    already ends in a mark (`.` `?` `!` `,` `;` `:` `…`), looking through closing brackets and quotes
    (biblatex.sty:2118-2130 `\blx@addpunct`, :2026, :1749-1754). A blank unit prints nothing.
    pdflatex + biber: "Second (2nd ed.) Addon", "In the U.S.A. Addon", "Ends with colon: Addon".
  - An `@inproceedings`' event, venue and date print once. An `@incollection` or `@inbook` event is
    nested in the book host and prints once. biblatex's standard drivers print no event for those
    two (standard.bbx:311/578/710); sbl.bbx:293 does.
- **Residuals:**
  - `eventdate` and `urldate` print in ISO form, which is biblatex's `eventdate=iso`. Its default
    `comp` form ("May 19–25, 1968") depends on the document language.
  - Hyphenated initials print "A." where biblatex prints "A.-T.".
  - Per-type options and name formats are not modelled.
  - geschichtsfrkl's private switch (`\ifbool{bbx:nurinit}`, geschichtsfrkl.bbx:105) is read as
    initials.
  - A title addon listed before its subtitle in the `.bib` prints first.
  - The book subtitle's `,` before the place's `(` ("Symposium, (…"), and the event date running
    into the editor with no punctuation.
  - A row's trailing `.` is unconditional, as in Perl, so a title ending in `?` prints "What is
    X?. A survey"; biblatex and BibTeX's `add.period$` print "What is X? A survey".

**Guards**:
- `bibliography_names_fields::{bst_full_name_template_spells_given_names_out,
  bst_initials_template_abbreviates_given_names, biblatex_default_spells_given_names_out,
  biblatex_giveninits_abbreviates_given_names, biblatex_style_decides_given_names,
  bst_name_list_with_others_is_comma_separated, biblatex_giveninits_keeps_percent_comment_entries,
  biblatex_second_tier_fields_are_printed, biblatex_title_units_and_events_print_once,
  classic_host_rows_are_unchanged}`;
- the 06_cluster_bibliography goldens (15 updated to the style's own name form).

---
### 300. catchfile reads the file as catchfile.sty does; newverbs' own code prints in the document encoding (Perl: raw catchfile, ASCII quotes)

Perl has no catchfile binding, so it raw-loads catchfile.sty. Rust (batch 56iv, round-12 P4/P5,
`latexml_contrib/src/catchfile_sty.rs`) ports catchfile.sty:224-309 literally. Only the file opener
is native (`\lx@catchfile@input`). It builds the name as the kernel's `\IfFileExists` does, so
`\CatchFileDef\x{\jobname.aux}{}` finds the file. It reads virtual files written by `filecontents` or
`\openout`. It opens the file with an opaque boundary: a delimited read stops at the file's end, as
TeX's does (tex.web §338).
- **Missing file:** it raises catchfile's own `\CatchFile@NotFound` package error (catchfile.sty:
  240-245), one Error.
- **`\CatchFileEdef`:** it is catchfile.sty:251-261 over the same file opener, since batch 56ix
  lets `\noexpand` carry its `\xdef` across the file's end (tex.web §367; KNOWN_PERL_ERRORS #259).
  A Latin-1 file decodes as in pdflatex (`[café][café ]`). Batch 56iv had stood in with a string
  copy of the file, which read non-UTF-8 bytes lossily.
- **`\CatchFile@File`:** it holds the resolved path, not the name as written.
- **An unbalanced caught file:** the partial argument goes back into the input, as Perl's readUntil
  recovery does (Gullet.pm:683-685), where TeX discards it (tex.web §396). pdflatex prints
  "Before. After."; Rust prints "Before. AB␤_After." (a control case in
  `binding_singletons_56::catchfile_expands_the_name_and_reads_filecontents` pins the actual output).

newverbs (`latexml_package/src/package/newverbs_sty.rs`) ports newverbs.sty:52-69 as written: the
command appends its `after` code to the `\verb@egroup` in force and calls the `\verb` in force (or
the given `[\verbcmd]`), so a package that redefines the `\verb`/`\verb@egroup` pair still works.
The before and after code print in the document encoding. Only the verbatim body keeps the ASCII
font of #144. pdflatex prints ‘‘x_y’’ for `\qverb`. The earlier output was ASCII `` `` `` ... `''`.

**Guards:** `binding_singletons_56::{catchfilebetweentags_uses_the_eof_protocol,
catchfile_expands_the_name_and_reads_filecontents, qverb_quotes_share_the_verbatim_font,
qverb_keeps_a_redefined_verb_pair}`, `perfect_kernel_batch54` catchfile guards,
`perfect_kernel_batch56::make_special_short_verb_is_defined`.

---
### 301. pstricks drawing objects draw into the picture (Perl: pt geometry in a px picture, several parameters lost)

Perl's pstricks_support.sty.ltxml draws `\psline`, `\psframe`, `\pscircle`, `\psarc`, `\pswedge`,
`\psdots`, `\pspolygon`, `\pscurve`, `\psgrid` and their relatives as `ltx:line`/`rect`/`circle`/…
elements. Rust had gobbling stubs instead: every such object vanished. **Rust** (batch 56iw, round-12
P1, `latexml_package/src/package/pstricks_support_sty.rs`) ports the constructors and reads the
parameters from the raw `\psset` state. It departs from Perl where Perl disagrees with pdflatex:

| # | Rust | Perl (pstricks_support.sty.ltxml) | pdflatex |
|---|---|---|---|
| 1 | points and radii in px | `ptValue` into the px picture (:663, :675, :701, :717, :733, :756, :784, :805): objects at 72.27 % size | objects meet the frame |
| 2 | `{->}` arrows kept (`terminators`, `arrowlength`) | the `psTerminators` hook (:495) receives the stomach, never sets them | draws the heads |
| 3 | `framearc` rounds corners | `arcValue` (:430-443) takes `ptValue` of a Float: 0 | rounds (pstricks.tex:2175-2184) |
| 4 | `\qline` stroked | `DefSimplePSConstructor` (:509-514) never attaches parameters: invisible | strokes |
| 5 | `\psgrid` reads `*[…]` | no `OptionalMatch:* []` (:826): options printed as text | draws the grid |
| 6 | negative units normalised | `c1-c0` (:693-694) gives a negative height, which SVG does not draw | draws |
| 7 | hatch fill styles do not paint the white default | `psGetFill` (:363-373) fills anything but `none` | hatch lines only |
| 8 | dash only for `linestyle=dashed` | `psGetDash` (:344-355) dashes a solid line that has `dash` set | dashes only dashed |
| 9 | raw `\pssetlength`/`\psaddtolength` | `Let` to `\setlength`/`\addtolength` (:641-651): a bare number is pt, with warnings | a bare number is in `\psunit` |
| 10 | `\newpsobject` keys kept as tokens | `Explode(ToString)` (:849-861): egameps 1 error, black lines | resolves the macros |
| 11 | object group `\begingroup`…`\endgroup` | `{`…`Digest(T_END)` (:479, :500) unbalances the alignment brace ledger (dspTricksManual 0 → 17 errors + Fatal) | no brace in the ledger |
| 12 | `angle1="0"` emitted | `trunc2` (:425-427) drops 0 | 0 is an angle |
| 13 | polar `(r;a)` and mixed `(A|B)` resolved; node, `(!…)`, `(*…)`, `(+…)` objects not drawn, `\rput` at the origin | `ReadPair` cartesian only (:103-105): a node reads past `)`, `(!…)` errors, `(1;45)` leaks as text | places every form |
| 14 | objects outside any picture not drawn | an auto-opened picture sized 0×0 (latex_constructs.pool.ltxml:4943-4950) that the SVG post draws overflow-visible (Post/SVG.pm:110), swallowing the paragraph | draws at the current point |
| 15 | `\degrees` without argument = 360 | stores undef (:653); angles ×360 | resets to 360 (pstricks.tex:763) |
| 16 | `\psarcn{->}` → `<-` | the `reverseArrow` regex (:452-459) gives `-<` | head at the clockwise end |
| 17 | any xcolor model parsed; bare names to hex (except black/white) | `setGraphParams` (:336-338) looks `[HTML]FF8000` up as a name: Error, black | the colour |
| 18 | `\SpecialCoor`/`\NormalCoor` are the raw pstricks modes (pstricks.tex:746-806), since `\pssetlength` is raw; the coordinate reader always accepts the `\SpecialCoor` forms, pstricks' default since :806 | no-ops (:1037-1038): pst-poly's `\NormalCoor` around an empty `\pssetlength` (pst-poly.tex:68-71) read past `\@nil` (hexgame 1 error) | under `\NormalCoor` reads cartesian coordinates only |

Residuals: an object outside any picture could draw at the current point through an auto-opened
picture sized from its box, as Perl does; that size is Perl's (batch 56iy, W12; the node box is
#319). `\multirput` is a no-op
and drops its text bodies; pstricks_sty.rs's `\psclip{}` no-op overrides the support file's
`{psclip}` on the LaTeX path. The SVG post does not draw arrow markers, arc strokes or dot fills
yet (latexml_post svg.rs). Colours pstricks defines itself (plain `\input pstricks`,
pstricks-color.tex:23-27 `\@newcolor`) are read from its own storage: `green` is #00FF00.

**Guards**: `pstricks_drawing::{pstricks_objects_draw_into_the_picture,
psset_lengths_read_bare_numbers_in_psunit, pstricks_objects_follow_the_parameter_state,
pstricks_coordinates_that_cannot_be_placed_draw_nothing, plain_input_pstricks_sizes_its_pictures,
pstricks_objects_read_macro_keys_colour_models_and_cells, pstricks_coordinate_modes_are_the_raw_ones,
hexgame_board_converts, pstricks_macro_colour_keys_in_tabularx_cells}`.

---
### 302. forest trees are nested inline lists with bounded key processing; chemnum's first use of a compound carries its id (Perl: no forest or chemnum binding)

Perl has no forest binding and no chemnum binding. **Rust** (batch 56iz, round-12 P6-P8,
`latexml_contrib/src/forest_sty.rs`, `chemnum_sty.rs`) models the parts of forest that carry
content and structure, and departs from forest.sty and pdflatex as follows:

- **Output shape:** every form (`{forest}`, `\Forest`, `\Forest*`) is an `ltx:inline-block`
  holding nested inline lists. forest draws a picture. pdflatex draws every form as an inline box,
  and the star only drops `\forest@group@env`'s group (forest.sty:8506-8514, :8528). So a tree in
  running text stays in its paragraph. The star is read and ignored: styles and names stay local
  to a tree for every form.
- **Node keys:** only `edge label`, `tier`, `phantom` and `name` are modelled. Node specs become
  `content=<spec>` (forest.sty:1423-1426) and are processed with pgfkeys' rules
  (pgfkeys.code.tex:357, 369, 507-520).
  - A phantom node hides its own edge label and its children's (forest.sty:7643-7651).
  - Everything else is layout and is ignored: the `+`/`'` forms (forest.sty:2413-2418),
    `.expanded`, `/.cd`, `content=`, `delay`, `where`, `replace by`, `draw`.
  - Afterthoughts are dropped, and `(config)` is not interpreted.
  - Edge labels keep only TikZ `node{…}` texts.
- **Styles:** `\forestset` records `.style`, `.append style`, `.prefix style` and `.try`, and
  `default preamble` (:3793, :6045-6049). Bare node keys in `\forestset` are not tree-wide
  defaults (:130).
- **Action characters are not executed** (:1598-1627). A tree whose text holds one is marked, and
  its labels keep the raw action text: forest-doc.tex:1784-1795 shows `@@[×1[f]]…` in one node
  where pdflatex draws 13. In a marked tree, `phantom` is not applied, so no text pdflatex prints
  is hidden. The marking compares by token (charcode and catcode), not by meaning as `\ifx` does
  (a control sequence `\let` to `[` does not match; forest.sty:1416). `\bracketResume` is `\relax`.
- **Key runaway:** one `Error` where pdflatex dies fatally ("TeX capacity exceeded", or 100
  errors). Style expansion stops after 64 nested styles. All keylist processing in a tree or
  `\forestset` stops when a work budget is spent: 1,000,000 units plus 10,000 per node (or
  1,000,000 per `\forestset` / per library's defaults), charged per key (name and value, again
  for every replay from an ancestor's `for tree`) and per token an expansion or definition copies.
  Inherited `for tree` keylists live on one stack shared by the tree, and the replay stops once
  the budget is spent. Measured needs: forest-doc ≤ 1,206 per node, milsymb 12,534 per tree,
  prooftrees.sty 34,260 per `\forestset`.
- **Other:** `\forestoption` and its relatives are `\relax`. `\useforestlibrary` records the
  library (forest.sty:173-177) but loads no library file. The bare `\forest … \endforest` form
  emits `<ltx:ERROR>` and discards the body; prooftrees and neoschool reach forest this way.
- **chemnum:** when `\cmpd{x.a}` is a compound's first use, its output is wrapped in
  `xml:id="cmpd.x"`, so a later `\refcmpd{x}` resolves (chemnum.sty:1324, 1358, 1370-1378).
  chemnum with hyperref places a point target instead. `\cmpdinit` is accepted without
  chemnum.sty:181-191's deprecation warning.

**Guards**: `forest_chemnum::*` (11), `perfect_kernel_batch56::forest_*`.

### 303. The Unicode-engine profiles typeset in the TU encoding (Perl: no TU encoding, OT1 throughout)

A LuaTeX or XeTeX format inputs tuenc.def and makes `TU` the default encoding (fonttext.ltx:57-68,
93); its slots are Unicode code points. Perl has no TU fontmap, so `\lx@fontencoding{TU}` falls back
to OT1 (TeX_Fonts.pool.ltxml:169-175), and text and `\char` decode through OT1 even in a document
that only lualatex or xelatex can compile: `\char`\{` prints "–", a typed `<` prints "¡".
**Rust** (batch 56ja) follows the Unicode format under the `luatex` and `xetex` profiles
(`latexml_sty` `install_unicode_format_encoding`, `tu_fontmap.rs`):

- The TU map is the identity on printable slots, with the TeX ligatures of the format's Latin
  Modern text fonts (tuenc.def:60 `+tlig;`, :100 `mapping=tex-text;`): `"` `'` `` ` `` are ” ’ ‘,
  and the quote ligatures (`non_typewriter_t1`, Perl TeX_Fonts.pool.ltxml:344 OT1/T1) also apply
  under TU. The typewriter map is the plain identity (tulmtt.fd loads no ligatures).
- A text symbol, accent or composite declared for TU at a slot above 255 is that code point
  (`decode_slot`; tuenc-greek.def:204 `\DeclareTextSymbol{\textalpha}\UnicodeEncodingName{"03B1}`),
  where the 8-bit rule gives no glyph.
- The `xetex` profile installs the same format encoding, after `\XeTeXrevision` so that tuenc.def
  takes its XeTeX branch.
- The pdfTeX-model default keeps OT1, as pdflatex. The TU map is registered for every profile,
  though, so an explicit TU selection under it (`\fontencoding{TU}`, `\UseTextSymbol{TU}`,
  fontenc's `[TU]`, the fontspec binding's `\latinencoding`=TU reached through babel's
  `\latintext`) now decodes through TU where Perl falls back to OT1. pdflatex has no TU
  (tuenc.def:49-57 defaults to T1); xelatex prints TU.

Measured against lualatex and xelatex (repro
`tools/perfect_kernel/repros/unicode-catcodes/tu_encoding_char_and_tlig.tex`): identical output.
Manual A/B over the 339 manuals with a clean lualatex/xelatex oracle: tipauni-example 80.1 → 93.4,
greek-fontenc char-list 64.6 → 65.8, pgfornament tikzrput 97.7 → 98.1, no recall loss; errors
375 → 375, warnings 15,829 → 15,902 (char-list +74, ijsra −1).

**Regression this batch introduced, on one manual (resolved in 56jf, #308)**: greek-fontenc char-list. TU routes textalpha's
Greek accents through tuenc.def's `\add@unicode@accent` (`\char"0313\relax`), and our case changer
(`lx_read_and_change_case`, latex_constructs/mod.rs) expands robust text commands and ignores
l3text's case-change equivalents (textalpha.sty:213 `\DeclareCaseChangeEquivalent`), so in
`\MakeUppercase{\>\`α}` it separates `\char` from its number: 74 "Missing number" warnings, and
`”0313` printed as text in the uppercase column where lualatex prints Ὰ. Before the batch the same
cells were empty or a stray combining mark. Red repro
`unicode-catcodes/case_change_greek_accent_char.tex`; SYNC_STATUS lead.

**Guards**: `unicode_format_encoding::*`, `perfect_kernel_batch56::luatex_csstring_primitive`.

### 304. A file that ends inside a definition inserts a `}` and the document goes on (Perl: the read stops at the file's end)

TeX, at the end of a file read while scanning a definition, closes the file, reports "File ended
while scanning definition", inserts a `}` and carries on in the enclosing input (tex.web §362 calls
§336 `check_outer_validity`; §338-339 `ins_list`): the definition ends at the file's end and the rest
of the document is kept. Perl's `readBalanced` stops at the mouth's end (Gullet.pm:470-472), then
errors (:525), and the definition is lost (`misdefined`). Rust stopped the same way for a file on
disk, one error more than pdflatex (the document's `}` then stray), and read a file held in memory
(filecontents, an `\openout` file) as a string, crossing its end: the `\edef` swallowed the rest
of the document. **Rust** (batch 56ja) marks every mouth that reads a file, on disk or in memory,
as a file level (`Mouth::is_file_level`, tex.web's `name>17`). A balanced read never crosses a
file level's end, and inside a definition it recovers as TeX does (`read_balanced_with_close`).
Scope, and where it stops short of TeX:

- Only an autoclose file level with an enclosing input recovers: an `\input` file, a filecontents
  or `\openout` file, catchfile's opener. A package or class file read through
  `reading_from_mouth` (not autoclose) and the main document keep Perl's stop.
- Batch 56jg (#310) extends the recovery to every scan status TeX has at a file end: token-list
  texts, macro arguments and `\halign` preambles. `\scantokens` stays crossable (the cprotect binding wraps text in it without
`\protect`, cprotect.sty:180-185), as does any `\everyeof{\noexpand}` read (batch 56ix).
Error counts equal pdflatex's on the six cases of `vfs_file_end` and the 56ix control.

**Guards**: `vfs_file_end::*`, `noexpand_input_ends::a_definition_still_runs_off_a_file_end`.

### 305. The format is read as initex reads it, and `\radical` is a primitive (Perl: `{` at catcode 1, `\radical` undefined)

`--init=latex.ltx` must log zero errors (CLAUDE.md parity rule 4). Since batch 56g made
`\errmessage` an Error, three kernel checks failed there; **Rust** (batch 56jb, worker W5) meets
them the way initex does:

- **Braces at catcode 12.** initex has `{`/`}` at catcode 12 (tex.web §232), and latex.ltx:98-101
  stops with "LaTeX must be made using an initex with no format preloaded" when `{` is already 1;
  latex.ltx:102-103 (plain.tex:11-12) then set them to 1/2. `ini_tex.rs` sets both to 12 after the
  bootstrap snapshot and before reading the format file, so the dump carries no brace records.
  Perl reads the format with `{` at 1 (State.pm:105-112) and logs the `\errmessage` as a Note
  (TeX_Debugging.pool.ltxml:71-74). A missing format file now fails the dump, as Perl's
  `DumpFile` does (TeX_Job.pool.ltxml:141-144).
- **`\radical`.** Perl lists it as not implemented (TeX_Math.pool.ltxml:28), so
  `\DeclareMathRadical` (latex.ltx:13650-13698) failed its `\meaning` test and fontmath.ltx:423
  reported `\sqrtsign` already defined. `tex_math.rs` defines the primitive: it scans the math
  field as tex.web §1151 does (blanks and `\relax` skipped; `\char`/`\mathchar`/`\delimiter` with
  their number) and hands it to the `\sqrt` constructor under the private `\lx@radical@sqrt`, so
  `\let\sqrt\sqrtsign` does not loop. The dump carries `\sqrtsign` = `\radical"270370\relax`, what
  pdflatex's `\meaning` prints. A field that opens with an implicit `\bgroup` is read as one token.
- **`\CurrentFile` family** (latex.ltx:19582-19585): defined by latex.ltx, not by the Base pool
  (tex_file_io.rs defined it, Rust-only), so plain TeX has it undefined, as pdftex; the dump
  carries it empty, as Perl's (latex_dump.pool.ltxml:2716-2719).

`tools/make_formats.sh` now fails when either init logs an `Error:`/`Fatal:` (the
release-dumps.yml pattern), so CI catches an init regression whenever the engine changes.

**Guards**: `dump_gate_init::*` (6).

### 306. A biblatex `.bbl` is read into bibentries and formatted like its `.bib`; a bibliography row adds no period after a mark (Perl: `.bbl` rebuilt as `\bibitem`s, periods unconditional)

- **`.bbl` reader.** Perl's ar5iv biblatex binding (biblatex.sty.ltxml:495-690) rebuilds a `.bbl`
  as a `\thebibliography` of `\bibitem`s, and so did Rust, dropping every field its layout did not
  know (subtitles, title addons, events). **Rust** (batch 56jc, worker W6) reads the `.bbl`'s
  `\entry`/`\name`/`\list`/`\field`/`\keyw`/`\verb` data into the BibTeX pool's `BibEntry` (field
  tables from blx-dm.def:473-634) and runs `\ProcessBibTeXEntry`, so a `.bbl` gives the same
  `ltx:bibentry` as its `.bib`, formatted by MakeBibliography with the `.bib` path's rules. biber's
  choices are kept: labels (`labelalpha` + `extraalpha`, used as the refnum, so `\cite` prints
  `[Knu84]`), the `.bbl`'s entry order (a biblatex bibliography of inline entries is printed whole
  with `sort='false'`), one datalist (the default refcontext's; biblatex prints one), `skipbib`
  and biblists. Filtered `\printbibliography` calls and per-refsection printing are not modelled.
  Inline bibentry ids are released before formatting (`PostDocument::release_id`), which also
  resolves amsrefs' bibliography links (amshelp: 24 of 25 dangling → 0).
- **Row periods.** Perl's `formatBibEntry` adds its "." / ". " unconditionally
  (MakeBibliography.pm:527-531): "What is X?. A survey", "Pub, Inc..". Rust drops the period after
  a mark by the bibliography engine's own rule, picked by the `bibstyle`: BibTeX's `add.period$`
  (no period after `.` `?` `!`, btxhak.tex:326-329) for a `.bst` or no style; biblatex's
  punctuation tracker (biblatex.sty:2118-2130, :2026, :1749-1754) for biblatex. A formula, or a
  reference CrossRef fills in later, counts as ending in no mark. Checked with pdflatex+bibtex and
  pdflatex+biber.

Measured (POST_MODE=mono, 340 bibliography manuals): recall up 0, down 0; mark-plus-period in
bibliographies 356 → 1. arXiv: the 7 papers of the 3,003 sample that ship a biblatex `.bbl` keep
their bibitem counts, 5 gain words (up to +50 %), labels are biber's; 2605.17646's duplicate
datalist is gone (58 → 29 bibitems). Residuals: the `.bib` path's alphabetic labels (it prints
`[1]`; porting biber's labelalpha would fix it), "Jr.", `maxbibnames`, apa's "&" in the
bibliography and `\parencite` (apa.cbx:496-497, :727; `\textcite` joins with "and", :50-55).

**Guards**: `bibliography_names_fields::{title_mark_takes_no_period_biblatex, title_mark_takes_no_period_bst, bbl_prints_what_its_bib_prints, bbl_keeps_bibers_alphabetic_labels_and_order, bbl_prints_the_default_refcontext_datalist, bbl_skips_its_biblists_and_skipbib_entries, bbl_format2_reads_as_its_bib, bare_thebibliography_twice_arms_once}`, unit test `make_bibliography::test_period_rule`.

### 307. Citation labels follow Perl's `make_bibcite`; the bibliography's XPath is real XPath (Perl: the same; cloned tag nodes)

Batch 56jd ports two Perl mechanisms the post stage had approximated. Both are faithful; this entry
records what changed visibly and what stays different.

- **Foreign-node XPath.** `PostDocument::findnodes_foreign` evaluated a hand-rolled subset of
  XPath that treated any predicate with `(` as satisfied and never matched `.//`. It now
  evaluates real XPath in the node's own document (`Context::from_node`, `ltx` bound), as Perl's
  `$XPATH->findnodes($path, $node)` does (Post.pm:1019-1021), and a failed evaluation raises
  `Error:post:xpath`. MakeBibliography's `[not(../ltx:bib-related[@bibrefs])]` host rows
  (MakeBibliography.pm:729-734) now apply, so a crossref'd entry prints "See [parent]" and no
  second "In <booktitle>". The Rust-only host publisher/place rows (#286) and the biblatex event
  rows carry the same step, so a crossref'd entry does not print half of its host again. The sort
  `ERROR` removal (Step 6) uses `.//` (the entry's subtree) where Perl uses `//`
  (MakeBibliography.pm:377); no binding emits `ltx:ERROR[@class='sort']`, so it changes no output.
- **The show walk** (`CrossRef::make_bibcite`, a port of CrossRef.pm:486-644): each citation link
  carries its entry's title as `title=` (:530-557; a missing citation's carries its key), the show
  roles are matched lower-cased with a trailing `s` stripped (:585), and
  `title`/`author`/`fullauthor`/`refnum`/`phraseN`/`year`/`number`/`super` render as Perl's.
  natbib author-year cites link only the year and merge same-author entries ("Smith, 2001a, b"),
  natbib `super` superscripts, and a crossref shows "The whole volume, Editor" (`do_crossref`,
  MakeBibliography.pm:638-642). Also as Perl: a bibref with no keys is removed; an entry without
  authors, full authors or a key demotes the whole citation to its refnum (:542); unknown show words
  are dropped (with an Info) instead of printed; `phraseN` indexes all element children. The `.bbl`
  route's title tag comes from MakeBibliography (#306). #59 and #123 are unchanged (#123's gate
  stays on natbib's capitalized `Author`/`Year`).

Kept divergences: the cloned labels of the `title`, `author` and `fullauthor` roles
(`\citetitle`, `\citeauthor`) are text, where Perl clones nodes: MakeBibliography builds the
title and authors tags as text (Perl clones child nodes, MakeBibliography.pm:473-475, :435-437), and
Scan stores their content as strings (scan.rs `bibitem_tag_props`; Perl Scan.pm:474-482 stores the
nodes). So `et al.` in a citation loses Perl's `ltx_bib_etal` wrapper. Perl's treatment of the
string "0" as false (a key, title or year suffix of "0") is not copied.

**Guards**: `bibref_show::*` (6), `bibliography_crossref::crossref_child_sees_its_parent_and_skips_the_host_title`.

### 309. NFSS text-font state follows every font switch; `\the\font` names the current font (Perl: codes only from `\fontfamily`…; `\the\font` = the last `\font` identifier or cmr10)

Perl keeps LaTeX's NFSS codes (`\f@family`, `\f@series`, `\f@shape`) only where `\fontfamily` and
its relatives write them, and its class bindings' size switches set the size alone
(article.cls.ltxml:111; latex_constructs.pool.ltxml:5622). So doc.sty's `\MacroFont`
(`\fontfamily\ttdefault … \small`, doc.sty:149-153), which relies on `\small`'s `\selectfont`
(latex.ltx:14103-14107), never applied cmtt, and a later `\selectfont` reset a plain `\tt`/`\bf`
to the stale codes: code printed `\ { }` through OT1 roman as “ – ˝. **Rust** (batch 56je, worker
W8), each checked against pdflatex with `\typeout` probes:

1. Every font merge writes the NFSS codes, locally and in text mode only (content.rs
   `sync_nfss_font_state`, reverse maps in `common/font.rs`); a code already in force that names
   the same font is kept (`pcr` survives `\ttfamily`). A raw `\font\x=cmtt10 \x` writes them too,
   where TeX leaves them to NFSS.
2. A class binding's size switch ends in `\selectfont`, pushed back into the input (dialect.rs), so
   whatever `\selectfont` means there runs (pgf's `\nullfont` in a picture stays).
3. `\the\font`, `\fontname\font` and `\fontdimen`/`\hyphenchar` of `\font` use the current
   font's identifier (`current_font_identifier`, tex_fonts.rs), defined once as the kernel names it
   (`\OT1/cmtt/m/n/10`), where Perl returns the last `\font` identifier or `\lx@default@font`
   (TeX_Macro.pool.ltxml:272, TeX_Fonts.pool.ltxml:41-43). ltxdoc's `\oc@ttf` (ltxdoc.cls:136) and
   short-math-guide's `\ttfont` print in typewriter.
4. `\emph`'s `\f@shape` toggle hook (latex_constructs.pool.ltxml:414-416) is subsumed by (1); plain
   `\em` (plain_base.pool.ltxml:604-609) and `neutralize_font` (notes, tags) go through the same
   sync, the latter as `\reset@font` does (latex.ltx:14122, :17658-17659).

Measured (worker W8): OT1 mis-glyphs in code, “ – ˝: moloch 690/554/554 → 11/0/0, achicago-bst
102/671/665 → 45/2/0, short-math-guide 496/29/26 → 19/3/0, source2e 2761/534/397 → 281/89/0;
typewriter runs up 3-15×; recall, errors and goldens unchanged. Left: `\fontname` names a scaled
base font (`cmr10 at 14.4pt`, pdflatex `cmr12`); the default codes are fixed to cmr/cmss/cmtt/bx.

**Guards**: `nfss_font_state::*` (9), `math_text_font_restore::*`.

### 308. `\MakeUppercase`/`\MakeLowercase` follow l3text's declarations (Perl: `latexChangeCase` expands everything)

Perl's `latexChangeCase` (latex_constructs.pool.ltxml:5520-5563) reads the argument with
`readXToken`, so every command is expanded before it is case-changed; it consults the exclusion
list only after `\protect` and builds `\@uclclist` last-wins (:5491-5500). The kernel's
`\MakeUppercase` has been l3text's `\text_uppercase:n` since 2022: `\text_expand:n`
(expl3-code.tex:36164-36384) and the case loop (:36622-36767). **Rust** (batch 56je, worker W10)
follows l3text in one loop: for each command, read unexpanded, it checks the exclusion list (bare
commands too, so `\label` keeps its key), l3text's expand-equivalents (`\l__text_expand_<cs>_tl`,
`\exp_not:n`), the case-change equivalent (`\l__text_case_<cs>_tl`, written by
`\DeclareCaseChangeEquivalent`, latex.ltx:22400), `\CaseSwitch`, the letter-like table (l3text's
constants plus `\@uclclist`, first-wins, :37921-37977), and otherwise expands one level and checks
again. Checking declarations before expanding any command approximates l3text, which leaves only
robust, protected and encoding commands unexpanded. An excluded command's arguments are taken up to
the first `{`, as `\__text_change_case_exclude:nnnNw` does. A command whose expansion opens with
`\@protected@testopt` (every `\newcommand`/`\newenvironment` optional-argument command, latex.ltx:1249;
ours carry `Parameter::testopt`) is kept unexpanded, as `\__text_expand_testopt:N` does
(:36319-36328), so its default is read when it is typeset and is not cased (batch 56jn, KNOWN_PERL_ERRORS #273).
The wrapper's group lets `\oe`/`\OE` as latex.ltx:22380-22393 does; Perl's `\def\i{I}\def\j{J}` (:5917) is
dropped, since the letter-like table maps a bare `\i`/`\j` and the `\def`s reached a stored command's body
(`\MakeUppercase{\foo}`, `\foo` = `L\i st \oe uvre`: "LIst œuvre", pdflatex "Lıst Œuvre").

Consequences that match pdflatex but change our output: textalpha's Greek accents drop in
uppercase (`\MakeUppercase{\>\`α}` → Ὰ; greek-fontenc char-list 84 warnings → 0); a user-redefined
`\v`/`\d`/`\b`/`\c`/`\r` stays unexpanded and its argument is case-changed;
`\renewcommand\th{\textsuperscript{th}}` + `\MakeUppercase{5\th}` gives `5Þ` (was `5TH`). Where we
differ from l3text: our exclusion list adds `\thanks`, `\@ensuremath`, `\NoCaseChange` (so `\thanks`
text keeps its case where pdflatex uppercases it) and lacks `\begin`, `\end`, `\babelshorthand`,
`\cite␣`; `\DeclareUppercaseMapping` (code-point mappings, lgrenc.def:875-896) stays a no-op; the
`\l__text_expand_*` tables exist only when expl3 is loaded (dump vs NODUMP runs differ).

**Guards**: `case_change_equivalents::*`.

### 310. A file ending inside a text, a macro argument or a preamble recovers as TeX does (Perl: the read stops at the mouth's end)

Batch 56ja (#304) gave a definition body TeX's file-end recovery. **Rust** (batch 56jg, worker W9)
models tex.web's scanner status (§305) for every balanced read at an autoclose file level's end
(the same scope as #304), with TeX's messages and recovery (§338-339):

| status | set by | message | recovery |
|---|---|---|---|
| defining | `\def`/`\edef` bodies (`read_definition_body`) | "File ended while scanning definition" | insert `}` |
| absorbing | `GeneralText`/`XGeneralText`, `\toks=`/`\everyX=`, the pdfTeX text reads (incl. `\pdfescapehex`…`\pdffiledump`, read with `scan_pdf_ext_toks` in pdfTeX) | "…scanning text of \X" | insert `}` |
| matching | macro arguments (`Expandable::read_call_arguments`) | "…scanning use of \X"; for a non-`\long` call whose runaway holds a `\par`, "Paragraph ended before \X was complete" (§396) | the call is dropped (§392); after a `\par` the text from the `\par` on is kept; the ended file is closed |
| aligning | the `\halign` preamble | "…scanning preamble of \halign" | insert `\cr}` |

Perl's `readBalanced` stops at the mouth's end (Gullet.pm:470-472), reports "ran out" (:525) and
runs the macro on the partial argument (:683), so a runaway argument now loses what Perl kept, as
pdfTeX does. The status is saved and restored by a guard type on every exit path and reset per
conversion; `\long` survives the dump. Error counts, messages and text equal pdflatex's on the
repros (`expansion-primitives/file_end_*`). Since batch 56jj (#313) `\message`, `\errmessage`,
`\mark`, `\marks` and `\special` read their text at absorbing status, and a primitive's own token
reads follow the status too. Not modelled (residuals, SYNC_STATUS): `\lstnewenvironment`/
`\newtcbinputlisting` bodies and LaTeX-level commands Rust implements as primitives or constructors
(e.g. `\setcounter`) keep Perl's stop (a primitive-level abort would be needed); TeX's `skipping`
status; the defining message does not name the macro; the inserted `\cr` is not TeX's frozen one; a
parameterless closure macro running during an enclosing macro's typed-argument read has its runaway
charged to the enclosing call; a Rust `DefMacro` binding of a `\long` LaTeX original is non-`\long`
(it keeps the text after a `\par`, where pdflatex drops it).

**Guards**: `scanner_status::*` (8), `vfs_file_end::*`, re-pinned
`binding_singletons_56::catchfile_expands_the_name_and_reads_filecontents` (pdflatex's "Before. After.").

### 311. siunitx units mixing macros with literal material are parsed as literal units; listing files are decoded (Perl: pre-power before its item; raw bytes)

- **siunitx mixed input.** `six_convert_units_from_tokens` stopped at the first token that was not a
  unit macro and dropped the rest, silently: `\unit{\watt\per(\square\meter\kelvin)}` gave "W^{-1}".
  It now hands the whole input to the literal parser, as siunitx's symbolic test does
  (siunitx.sty:6596-6613; Perl siunitx.sty.ltxml:903-927). In the literal branch a pre-power
  (`\square`, `\cubic`, `\raiseto`) raises the item after it (siunitx.sty:6633-6641, :6801), where
  Perl renders `\mathrm{^{2}}\mathrm{m}`; a braced item has its units resolved; `\cancel` prints
  nothing (:6698-6700). Output as pdflatex: W/(m² K). arXiv: 30 unit expressions in 22 papers regain
  content ("4500 km" → "4500 km/s", 2605.00984; "100 G" → "100 GOps/s", 2605.15423; 2602.18218's
  `\cancel` "meV cancel(^{-1})" → "1 meV/c²").
- **nomencl `nomentbl` unit column** is `\unit{#4}` (nomencl.sty:232), so a siunitx unit prints (a
  bare `\meter` is `\relax` outside `\unit`, Perl siunitx.sty.ltxml:1352).
- **Listing files.** A missing `\lstinputlisting`/`\tcbinputlisting` file is `Error:I/O`, as Perl
  (listings.sty.ltxml:322-334) and pdflatex ("Package Listings Error: File … not found",
  listings.sty:2074-2086); Rust warned. A listing file that is not UTF-8 is decoded
  (`mouth::decode_input_bytes`) where it came out empty; Perl slurps the raw bytes.

Measured (worker W14): 19 s123 manuals turn 182 missing-listing warnings into errors (hvextern 75,
out of scope; underoverlap/concepts 18 each from a `'#1'` substitution in dry.sty/with.sty);
recall unchanged. Guards: `package_leads_56::*` (4), `nomencl_nomentbl`.

### 312. `\input` re-reads a raw definitions file; a kernel accent takes the encoding's declared composite (Perl: skips the re-read; ignores `\DeclareTextComposite`)

- **`\input` re-read.** While reading definitions, `\input` of a raw file that was already read
  reads it again, as TeX does. Perl routes such an `\input` to InputDefinitions
  (Package.pm:2287-2288), which returns for a file already `_loaded` (:2363). A binding stays
  once-only, and the re-read never goes through binding dispatch or the `\@currname` directory
  resolution (`InputDefinitionOptions::reread`). Consequence: when a binding raw-loads a file under
  another name and then overrides its definitions (epstopdf_sty.rs:15 with grfext,
  beamer_cls.rs:1477, unicode_math_sty.rs:103), a later raw `\input` of that file re-runs it and
  overwrites those overrides, which Perl's skip would have kept. Witness: greek-fontenc
  test-tuenc-greek (tuenc-greek.def:126 re-inputs greek-fontenc.def for TU, so `\TU\greekscript`
  and the TU hiatus composites exist: babel-greek stays in TU under the luatex profile, `ΑΫΛΟΣ`).
  A re-declared text command is wrapped for its composites again (latex.ltx:9923-9932).
- **Kernel accents and declared composites.** A native accent (`\' \" \~` …) whose argument has
  no precomposed form looks up the composite the current encoding declares, keyed as the kernel
  keys it (latex.ltx:9933-9940, `\string\<enc>\<accent>-\string<first token>`, `\@empty` for an
  empty argument); the Rust composite keys and wrapper now have the kernel's spelling and shape
  (`\@text@composite` first, latex.ltx:9925-9932), so the dumped T1/OT1 composites are found. If
  the key exists it replaces the whole argument and the rest is dropped, as pdflatex (T1 `\"{ab}` →
  ä; OT1 → äb). A glyph composite is returned as its decoded character (a `tex=` attribute stays
  valid TeX); a command composite expands. The native precomposition applies only to letters the
  current font prints as themselves (LGR `\"i` → ϊ). Perl ignores `\DeclareTextComposite` (#184).

Measured (worker W11): 176 manuals (68 Greek, 8 babel-greek, 100 random) and 60 arXiv papers:
errors, fatals and warnings unchanged; 8 recall gains, 0 losses (test-athnum 78.4 → 100, usage
95.7 → 98.9, teubner-doc 96.8 → 98.4, hyperref-with-greek 92.4 → 93.7). Residuals: LGR `\~a` →
α̃ (pdflatex ᾶ; LGR's own `\~`, lgrenc.def:484-487), final sigma (KPE #148), a
`\textgreek{t'eqnh}` ligature (apprends-latex), babel language tags under the luatex profile
(luababel.def; red repro `babel-lang/luababel_language_tag_luatex.tex`). ἀͺ → ᾀ is
the CB fonts' iota ligature since batch 56jv (KNOWN_PERL_ERRORS #290).

**Guards**: `greek_text::*` (10), `perfect_kernel_batch54` (θ as pdflatex).

### 313. A primitive's token read crosses a file end; a token-list assignment takes an implicit brace; `\message` and `\mark` read their text as `scan_toks`; a definition's name that is no control sequence is "Missing control sequence inserted" (Perl: Fatal or "Missing argument"; a `{}` argument; any token defined)

**Rust** (batch 56jj, worker W13) follows tex.web where a primitive reads its own tokens:

- **A primitive's `get_token` crosses the end of an input level** (§362): every autoclose level
  with a parent (`\input` files, `\scantokens` pseudo-files, verbatim/listings line remainders,
  bibtex entry mouths). `gullet.rs` `read_primitive_token` serves the `Token` parameter
  (`\afterassignment`, `\aftergroup`, `\futurelet`), `\expandafter` (which now reads its own two
  tokens, §368) and `\let` (§1221). At `normal` status it crosses the level's end as `\noexpand`'s
  reader does. At `defining`/`absorbing`/`aligning` the end is the enclosing scan's runaway,
  reported where the primitive meets it and followed by TeX's inserted tokens (§336-339):
  `\edef\x{\expandafter` as a file's last line is one error and an empty `\x`, as pdflatex. At
  `matching` it stops and a macro's `Token` argument is `\relax` (see residual).
  `\let\c\@sptoken=\b` gives `\b`, as pdflatex (KNOWN_PERL_ERRORS #263).
- **`\string`, `\meaning` (§471), `\ifx` (§507) and `\ifdefined` read at `normal` whatever the
  enclosing scan** (`read_token_across_input_ends`, the `NormalToken` parameter), so inside an
  `\edef` body they cross a file end instead of meeting the definition's runaway.
- **A definition's name crosses a file end** (§1215 `get_r_token`): `RedefinableToken` /
  `read_redefinable_token` for `\def \gdef \edef \xdef \let \futurelet \chardef \mathchardef
  \countdef \dimendef \skipdef \muskipdef \toksdef \read \font` skips the spaces after the name.
- **A name that is neither a control sequence nor an active character is "Missing control
  sequence inserted"** (§1215, batch 56jp, worker W17): the token is read again (`back_input`) and the frozen
  `\inaccessible` is defined instead (a control sequence named `\inaccessible␣`, as §262 prints
  it), so `\gdef{}X{Y}` is one error and typesets "XY" (pdflatex). A `\noexpand`-marked name is
  the control sequence it marks (§358 sets `cur_cs`): `\expandafter\def\noexpand\foo{X}` defines
  `\foo`, where both engines defined the marker (`\special_relax`). Perl reads a plain `Token`
  (TeX_Macro.pool.ltxml:174-177, 184, 189), so `\gdef{` installs a macro on the Begin catcode and
  every later `{` reads a delimited argument (KNOWN_PERL_ERRORS #268; yquant-doc.tex:3832, an
  `\edef`-expanded `\pgfsyssoftpath@flushbuffers`: 1001 errors and a Fatal with #315's
  `\patchcmd` fix → 201, no Fatal). LaTeXML's own frontmatter idiom `\let^\lx@sup@…`
  (Base_Utility.pool.ltxml:729-737) needs the superscript catcode's key, which no TeX name
  reaches: Rust installs it with `\lx@let@superscript` (`base_utilities.rs`, a local `Let!` of
  `T_SUPER`). tcilatex's `\activesoff` is defined inside `\bgroup\makeactives…\egroup` as
  tcilatex.tex.ltxml:369-389 does (the binding had it at normal catcodes: 4 errors per `\FRAME`
  under babel). Frozen control sequences as names are not modelled (Rust has none). Residuals:
  `\def}` goes on to scan a parameter text where §475 says "Missing { inserted" and ends the
  definition (red repro `expansion-primitives/def_param_text_right_brace_storecmd.tex`,
  storecmd-guide `\hx{\def}`); guitar's `\changes` entry reaches `\glossary`, which digests
  `\def{…}` where latex.ltx:17726 gobbles it (red repro `index/glossary_digests_argument_guitar.tex`).
  Guards: `rtoken_patchcmd::{def_of_a_non_cs_defines_inaccessible,
  let_family_of_a_non_cs_reads_the_token_again, noexpand_marked_name_is_its_control_sequence,
  tcilatex_frame_under_babel_defines_active_punctuation}`.
- **A token-list assignment** (`read_tokens_value`, §1226-1227) finds its brace by expanding,
  skipping blanks and `\relax`; `\bgroup` counts as the left brace and a token register gives its
  value. Any other token is "Missing { inserted" (§403) and is kept as the value (Perl stores it
  silently).
  Witnesses 2605.02221, 2605.25087 (Paul Taylor's `diagrams.sty:15`, `\toks0=\bgroup}`): 1 error → 0.
- **`\message`, `\errmessage`, `\mark`, `\marks` and `\special` read `XGeneralText`**
  (`scan_toks(false, true)`): the brace is found by expanding, the text is expanded once (not a
  second time), and a file end is "File ended while scanning text of \X". An unbraced text is
  Rust's `read_balanced` recovery (error, token put back, empty text; Perl's `{}` argument takes
  the single token silently) where pdflatex inserts `{` and absorbs up to the next `}`, which would
  swallow the rest of a document after a top-level unbraced `\message`.
  Every TL use of `\message\x`/`\mark\x`/`\special\x` is a `\let` or an `\expandafter` chain
  (`gentombow.sty:641`, `qrcode.sty:3044`, `frhyphex.tex:13`).
- **`\meaning` names `\expandafter`, `\noexpand`, `\string`, `\meaning`** (also through a `\let`
  alias) while they are the kernel's closures (§296), where both engines printed `macro:…->CODE(<address>)`.

Perl: `Gullet.pm` `readToken` returns undef at the mouth's end, so `\expandafter`, `\string` and
`\meaning` there are a Fatal and the others "Missing argument Token" (KNOWN_PERL_ERRORS #262);
`\let` skips only an explicit space after `=` (#263); `readTokensValue` (Gullet.pm:824-842) takes
no implicit brace (#264); `\message`/`\errmessage` (TeX_Debugging.pool.ltxml:65,71) and `\mark`
(TeX_Marks.pool.ltxml:30) read a `{}` argument, and `\mark` does not expand its text (#265).

Measured (worker W13): 40 arXiv papers heavy in these primitives: 2 go from 1 error to 0, the rest
byte-identical once `xml:id`s are ignored; `--init` dumps byte-identical. Residual: a file end in a
`GeneralText`/`XGeneralText` brace hunt (`\uppercase`, `\lowercase`, `\detokenize`, `\message`,
`\errmessage`, `\mark`, `\marks`, `\special` as a file's last token: pdflatex 2 errors, Rust and
Perl 0; the hunt runs at the caller's status) and the `matching` status Rust sets around expandable primitives' typed
parameters (`\csname`, `\number`, `\readline`), which TeX never reads at `matching`; red repro
`expansion-primitives/uppercase_brace_hunt_file_end.tex`. Settled dead ends: making `read_token`
itself cross file ends (tried three times, see its doc); a runaway at `matching` in
`read_primitive_token` (it abandons `\csname`/`\number` calls whose tokens run to a file end).

**Guards**: `token_kernel_gaps::*` (18), `scanner_status::a_file_ending_inside_a_text_ends_the_text`.

### 314. `\shipout` is a primitive that emits its box in place (Perl: no `\shipout`; an error, or the box lost)

**Rust** (batch 56jk, worker W15): `\shipout` scans its box operand as `\setbox` does (tex.web
§1073 `leader_ship`, §1084 `scan_box`, shared as `tex_box.rs` `read_box_operand`) and, since
LaTeXML has no pages, puts the box where `\box` would put it in the current mode. LaTeX's
`\shipout` (latex.ltx:19911-19917, ltshipout) ends in `\tex_shipout:D \box_use:N \l_shipout_box`
(latex.ltx:19986), the expl3-code.tex:531 copy of the primitive, so the kernel's shipout hooks run
unchanged. LaTeXML never runs an output routine (`\clearpage` is `\lx@newpage`, `\output` never
fires) and its `\end{document}` (sect02.rs) never reaches `enddocument/afterlastpage`, whose
`\@kernel@after@enddocument@afterlastpage` (latex.ltx:20226-20269, in the dump) ships a "Temporary
page!" box — so only an explicit `\shipout` reaches the primitive, and a document without one is
unchanged. A void box gives LaTeX's "Ignoring void shipout box" warning (latex.ltx:19944).

**Perl** defines no `\shipout` (TeX_FileIO.pool.ltxml:257 is a placeholder). With a braced operand
(`\shipout\vbox{…}`) Perl's `\setbox` parks the `\afterassignment` token for the box body
(TeX_Box.pool.ltxml:170-172), so LaTeX's chain reaches `\tex_shipout:D`: `Error:undefined:\tex_shipout:D`,
then the box typeset in place (Perl 0.8.8, same host). With a register operand (`\shipout\box255`)
the token never fires (KNOWN_PERL_ERRORS #253) and the box is lost without a diagnostic (#266).
Rust fires the token for both since batch 56ir, so both errored; this batch removes the error and
keeps the box in place, as Perl already does for the braced form. Witness coverpage/SimpleSample
(the cover page `\shipout\box255`): status 2 → 0. Other kernel-`\shipout` users (morefloats,
ltnews32, pgfmorepages, bookestdoc-en, rvwrite-doc, blowup-ex1, elpres-example) are byte-identical;
TL's explicit body callers are CoverPage, grfpaste, pst-eps, memoize and blank-page idioms
(`\shipout\vbox{}`/`\hbox{}` emit nothing). Residuals: in horizontal mode a shipped `\hbox` runs into
the surrounding text (the guard pins "…continues.\nSavedAEnd."), where pdflatex makes separate
pages; a raw `shipout/background` hook user (draftwatermark.sty:187-190, via
`\__shipout_add_background_picture:n`, latex.ltx:20200) would add its overlay to an explicitly
shipped box; under pgfpages (`\pgfpages@interceptshipout`, pgfpages.sty:1017-1037) the box is held to
`\AtEndDocument` (:1131-1137) and so moves to the document's end; memoize's extern box
(memoize.sty:918) is duplicated independently, because `pdftexcmds_sty.rs:45` makes
`\pdf@primitive` a no-op that swallows `\shipout`; eso-pic's `\AddToShipoutPictureBG` material is not
emitted on a shipped page (pdflatex prints it).

`read_box_operand` (shared with `\setbox`) skips implicit spaces and `\let` aliases of `\relax`
(§404) but, like the old `\setbox`, invokes a non-box operand instead of TeX's "A <box> was supposed
to be here" (SYNC_STATUS).

**Guards**: `shipout_parskip::{shipout_emits_the_box_in_place, shipout_takes_every_box_operand}`
(each shipped text exactly once); repro `boxes-groups/shipout_box_register_simplesample.tex`.

### 315. A `\let` copy of a robust command gets its own body, and `\ifx` compares robust bodies; the NFSS font switches are robust; `\patchcmd` keeps the prefix (Perl: the wrapper is shared; plain-macro font switches; the prefix dropped)

**Rust** (`state.rs` `let_i`): `\let\x\cs` of a robust wrapper `\protect \cs␣` gives `\x` its own
wrapper `\protect \x␣` and copies the body to `\x␣`. TeX's `\let` copies the wrapper itself
(tex.web §1221), so a later `\DeclareRobustCommand\cs` changes what `\x` does — and loops on
`\let\origref\ref \DeclareRobustCommand\ref{\@ifstar\origref\origref}`, since the binding's `\ref`
is robust where latex.ltx's is not (witness arXiv 0810.0695, PlanarMain.tex's `\else` branch; since
K6 its `\pdfoutput=1` makes `\ifpdf` true, and the branch forced false still converts with 0 errors).
Two consequences: `\meaning\x` reads `\protect \x  ` where pdflatex reads `\protect \bfseries  `
(documented, not changed: sharing the wrapper brings the loop back), and `\ifx` of two robust wrappers
`\protect \A␣`/`\protect \B␣` with equal parameters compares the meanings of `\A␣` and `\B␣`
(`same_robust_body`, worker W17), so `\ifx\x\bfseries`, `\ifx\reset@font\normalfont` and, in the
preamble, amsthm's `\nonslanted` (`\let\@tempa\csname\f@shape shape\endcsname\ifx\@tempa\itshape`,
amsthm.sty:209-212) answer as pdflatex; in the body the shape switches are `\protected` macros (below)
and `\nonslanted` is a plain meaning comparison. The two bodies must be one definition (equal, and the same `cs`: the copied body
is the source's own `Rc<Expandable>`, named `\bfseries␣`, which the dump keeps), so two robust
commands declared apart with the same body stay unequal, as in TeX. A `\LetLtxMacro` copy
(letltxmacro.sty) and a `\NewCommandCopy`/`\DeclareCommandCopy`/`\RenewCommandCopy` copy (each its own
wrapper over a `\let` of the body) have the same state as `let_i`'s and compare true where pdflatex
says false (`\NewCommandCopy\zc\bfseries \ifx\zc\bfseries`: pdflatex F, Rust T; Rust before W17 said
true too, the switches being plain macros).

**Related (W17)**: the NFSS switches are robust in the preamble, as latex.ltx declares them
(`\fontencoding` :10490, `\usefont` :10518, `\fontseries` :12259, `\fontshape` :12436,
`\upshape`/`\slshape`/`\scshape`/`\itshape` :13794-13803, `\bfseries` :13923, `\mdseries` :13948,
`\rmfamily`/`\sffamily`/`\ttfamily` :13983-13993, `\normalfont` :14113, `\fontfamily` :14139), so
`\protected@edef` keeps them (KNOWN_PERL_ERRORS #269: titlecaps' `\titlecap`, doc's
`\SpecialEnvIndex` index entries, pythonimmediate's `\DescribeOption`). At `\begin{document}` the
begin-document constructor (sect02.rs) runs `\reinstall@nfss@defs` where latex.ltx's `\document`
runs `\@kernel@after@begindocument@before` (:9471, :12513-12514): in the body `\upshape`
`\slshape` `\scshape` `\itshape` (sect13.rs bodies) and `\ulcshape` `\swshape` `\sscshape`
(latex.ltx's, :12489-12511) are `\protected` macros, so a plain `\edef` keeps them too; family and
series switches stay wrappers. Like latex.ltx it replaces a preamble redefinition of those switches
(pdflatex: `\DeclareRobustCommand\itshape{\bfseries}` in the preamble is gone in the body).
`\normalshape` is a `\protected` macro in the preamble and the body (latex.ltx:12486-12488; Perl
latex_constructs.pool.ltxml:5161 lets it to `\upshape`, which copied the dump's robust wrapper, so a
plain `\edef` expanded it). The hook's `\init@series@setup` (latex.ltx:13902-13920) is not run: the
bound `\bfseries` does not read its `\bfseries@rm` family, but it also runs `\reset@font`, `\mdseries`
and `\let\seriesdefault\f@series`, so a preamble font switch stays in force in the body — shared
residual (base and same-host Perl alike): `\documentclass{article}\itshape\bfseries\begin{document}
Body text.` is upright CMR10 in pdflatex and bold italic in Rust. amsthm's `\nonslanted` is amsthm.sty's (Perl
amsthm.sty.ltxml:28 lets it to `\upshape`, which also set small caps upright). etoolbox's
`\patchcmd` keeps the macro's `\protected\long\outer` prefix, `[<prefix>]` replaces it, and only
the first match is replaced (etoolbox.sty:1358-1371; KNOWN_PERL_ERRORS #270).

**Guards**: `rtoken_patchcmd::{ifx_sees_let_copies_of_robust_font_switches,
font_switches_are_robust_in_protected_edef, body_shape_switches_are_protected, index_entry_keeps_ttfamily,
patchcmd_keeps_the_protected_prefix, patchcmd_prefix_option_and_single_replacement}`,
`perfect_kernel_batch56::deferred_math_end_walks_real_groups`.

---
### 316. Picture `\line` is a macro front for its constructor; `\bezier` is stroked (Perl: a constructor; no stroke)

Perl defines `\line` as `DefConstructor('\line Pair:Number {Float}', …)`
(latex_constructs.pool.ltxml:4992) and nothing else: `\line` is always the picture object, and
`\noindent\line{Left\hfil Right}` is three errors (Missing argument `Pair:Number`, `Float`) and a
`Fatal:misdefined` (`slopeToPicCoord`: "Can't call method 'getX'"), so the conversion fails, where pdflatex sets "Left … Right" across the line. Plain TeX's
`\line` is `\hbox to\hsize` (plain.tex:575), and papers use it outside pictures as a length or box
(witness 2306.13101, `\diagbox[height=2.5\line]{…}{…}`). **Rust** (`latex_constructs/sect13.rs`)
makes `\line` a `DefMacro` that peeks for `(`: with it, `\lx@pic@line Pair {DefaultUnits}` (Perl's
constructor, alias `\line`, its `{Float}` a `DefaultUnits` since #317); without it, `\hbox to \hsize`. Picture output is Perl's.

Perl's `\lx@pic@bezier` template (:5034-5038) has no `stroke`, so `\bezier`'s curve inherits the
`{picture}`'s `stroke='none'` and is invisible (KNOWN_PERL_ERRORS #271); the Rust `\lx@pic@bezier`
(Perl's constructor, sharing `\qbezier`'s properties) strokes it as `\qbezier` does. Both write
Perl's `displayedpoints` for a nonzero count only.

**Guards**: `node_box_append::{line_takes_box_free_arguments, line_outside_a_picture_is_a_full_width_box,
bezier_is_stroked_and_counts_its_points}`.

---
### 317. What follows the value in a braced typed argument stays in the input; assignments scan by the register's type; calc evaluates a braced length whole (Perl: dropped silently; `{Dimension}`; the first term)

**Rust** (batch 56jr, worker W18). A binding's braced or bracketed typed argument (`{Dimension}`,
`{Glue}`, `{Number}`, `{Float}`, `[Dimension]`, `CommaList:Number` items) is re-read by
`Parameters::reparse_argument` in a mouth of its own; Perl closes that mouth with whatever the scan
left (Gullet.pm:131-135; KNOWN_PERL_ERRORS #275). TeX never sees those braces — latex.ltx passes the
argument to `\setlength#1#2{#1 #2\relax}` (:10253) or `\global\csname c@#1\endcsname#2\relax`
(:10115-10122) — so the rest is read next. `parameter.rs` `read_braced` returns the tail (tokens a
look-ahead put back, the rest of the argument, and what a conditional cut by the scan still holds —
it is expanded to its `\fi` there, #193, one expansion at a time so nothing after the `\fi` is
expanded early: `\setlength{\xd}{\ifx\a\b 1pt\else 3\U\fi\the\xd}` typesets the new value, as
pdflatex, whose dimen scan stops at `3\U`, §455), and `ArgumentTails` holds it until the command's whole
argument list is read, then puts it back in the input: TeX grabs a macro's arguments before
scanning any of them (`\@rule[#1]#2#3`, latex.ltx:16360-16366). The tail is read right after the
command. The assignments `\setlength`, `\addtolength`, `\setcounter`, `\addtocounter` (and
calc's) instead read their value in the body with `read_braced_value` and put the tail back after
assigning — TeX's place (`xG`, `zY5`), silently, as pdflatex. A command that outputs a box or space
scans its length BEFORE the output in TeX (`\@hspace`, `\@imakebox`, `\@rule`, `\@iiiparbox`,
`\@iiiminipage`), so there the tail lands one step late — after the space or box (`\parbox`, a
macro, after its expansion's box), or at the start of a `minipage`/`tabular*` body. A handed-back
tail with more than spaces and `\relax` gets a `Warning:unexpected` ("Unexpected text after the
value in an argument of \hspace ('x'); it is read after \hspace"): pdflatex is silent, but such a
tail is almost always an authoring slip (a length expression without calc). Where a handed-back
tail would be misread it is dropped with that warning (`parameter::dropping_argument_tails`, an RAII
scope, as are the `ArgumentTails` and `in_braced_read` counts, so a panic caught by cortex_worker
leaves none set on the pool thread): a column type's, which the template reader took for more column
letters (`p{\textwidth-1pt}c` grew a `p` column of width `t`; TeX typesets it in every cell,
array.sty:189-191 `\@startpbox`), and one left at the head of an alignment row between rows
(`digest_alignment_column`), which opened the next row with a cell where the next `\noalign`
failed. booktabs' widths and spaces are unused and read untyped (`\cmidrule[lr]{1-2}
\cmidrule[lr]{3-4}` for `(lr)`: 2605.27476, 0 → 14 errors in the first cut, 2605.25272;
`\addlinespace[2pt plus 1pt]`), and the `\\[…]` of a tabular reads its length when it expands and
drops the rest, which sat in `\lx@hidden@cr`'s argument. An undefined control sequence the scan
met was reported and stubbed as it was expanded; TeX discards it (tex.web §370), so it is stripped
from the head of the tail (`is_error_stub`) and skipped by calc's post-scan: kept, it came back at
every use as `<ERROR>` text, or with calc as "invalid at this point" (2605.25073, `\hspace{6\@p@t}`
without USG.cls: 10 → 242 errors in the first cut; 2605.08378). `\setcounter`/`\addtocounter` of an
undefined counter read no value (latex.ltx:10115-10122 `\@nocounterr`, "AB C").

Types now follow TeX's scan, since a narrower one would turn the rest into text: `\setlength`/
`\addtolength` read by the register's own type (a `\newlength` is a skip: `plus`/`minus` kept, and
`\the\parskip` prints them; a `\dimen` register's `plus 1pt` is text, as in pdflatex); `\hspace`,
tabbing's `\\[…]`, soul's `\sodef` spacings, multirow's `[<vmove>]` read a `Glue` (`Glue` gained
Perl's `revert`, Number.pm:74-76, which it lacked: `x\hspace{10pt}y` kept its `\hskip 10.00002pt`);
revtex's `\rotatebox` and aastex's `\rotatefig` read a `Float` angle; multirow's `[<bigstruts>]`
(`[b2]`) is untyped; soul's `\capsdef` takes its five arguments (soul-ori.sty:697-700); hyperref's
`\hypercalcbp` evaluates `\dimexpr(#1)\relax` (hyperref.sty:364-366); floatflt's
`{floatingtable}` typesets its table argument (floatflt.sty:131-139) where Perl read it as a
`{Dimension}` and dropped it. A picture length (`\line`, `\vector`, `\circle`, `\dashbox`,
pict2e's `\oval[…]`) is the new `DefaultUnits` type, latex.ltx:16766-16767 `\@defaultunitsset`:
`\dimexpr<arg>\unitlength`, the rest discarded (`\remove@to@nnil`, :10537), so `\circle{1cm}` and
`\line(1,0){.5\linewidth}` are lengths (Perl's `{Float}` took 1 unit), a plain number stays exact.
A `fil` glue reverts with a space after its unit (`$x\hspace{\stretch{1}}l$` reverted to `…fill`
followed by the `l`, read as `filll`, tex.web §454). ntheorem's skip amounts, titling's `\droptitle`/
`\thanksmarkwidth`/`\thanksmargin` and authblk's `\affilsep` are skips, as their `.sty` allocates
them.

calc (`calc_sty.rs`; calc is loaded in most arXiv papers, about 78 % of the 56jr review sample,
through algorithmic, mathtools and others): a braced `{Dimension}`/`{Glue}` argument is evaluated as
a calc expression when the document loaded calc (`gullet::set_braced_length_fn`, gated per document by the
`calc_expressions` State flag, which now also gates the internal-dimension seam of #115/#141 — its
thread-local closure had outlived the document that loaded calc). latex.ltx hands these lengths to
`\setlength`, which calc redefines (calc.sty:86, :51-53): `\makebox[\widthof{ab}+\widthof{cdefgh}]`
is 38.6pt (pdflatex), not 10.6pt. It also applies to `{Dimension}` bindings whose LaTeX original
assigns directly — booktabs rule widths, `\tablewidth`, and the driver internals `\pgfsys@moveto`/
`lineto`/`curveto`/`rect`, `\lxSVG@*`, `\pIIe@*`, `\Gscale@box@dd` — where only input that is not a
plain length differs (a pgf coordinate is always `\the\pgf@x`); a 6000-path tikzpicture with calc
loaded ran in 4.39 s against 4.34 s (median of 5, release, 8 cores).
calc's reader reports a token it cannot parse after a term (calc.sty:144-160, 281-284: "`x' invalid at
this point") and consumes it, the rest staying in the input; it reads a parenthesized group in place
so parentheses nest (Perl reads to the first `)`, calc.sty.ltxml:183-184: hexgame.sty:77
`1.5\halfhexwidth*((\value{modulocounter})-1)` lost `-1)`, 4 "Illegal unit" warnings); and a braced
group is spliced in, as `\calc@pre@scan`'s undelimited argument (calc.sty:88-89; mhchem.sty:1020
`{2em}` in `\makebox[#7]`). calc's `!` sentinel, which pdflatex typesets after the error, is not. A
package may take calc's error over — picture.sty:79-120 `\let`s `\calc@error` to a handler that accepts
a `\unitlength` after a length (`\setlength\dimen@{#1\unitlength}`) and gobbles the rest to calc's `!`
and passes any other token to calc's own (:88-107). That handler runs on calc.sty internals the
binding lacks, so its decision is taken in `calc_post_scan`: while `\calc@error` is not calc's own
(`\lx@calc@error`), a `\unitlength` ends the expression silently and anything else is reported
(circledsteps under picture.sty, 2605.09094/2605.10684: 6 and 15 false errors in the first cut; its
circles now sized by their lengths, where the `{Float}` read gave r=0).

pgf 3.1's `\pgfsys@declarepattern` has fifteen arguments, a six-entry matrix before the code
(pgfcorepatterns.code.tex:126, 159-164; pgfsys-common-svg.def:677-690); Perl's binding reads nine
(pgfsys-latexml.def.ltxml:492), so the matrix's `{1.0}` was the code and `{0.0}` the flag. Its `.0`
now re-entered the input and was warned about; before, the call's unread rest ran the pattern code after
`\lxSVG@(un)coloredpattern`, which left `<svg:defs><svg:pattern>` open for it, so everything the
picture drew afterwards (a `\node`'s text) sat inside the invisible `<svg:defs>`. The binding takes
the fifteen arguments (the matrix is not applied) and the pattern constructors absorb their code and
close their elements (Perl's templates hold `#4`, pgfsys-latexml.def.ltxml:509-532). Setting a
pattern also sets the fill a path inherits (`pgf@svg@fillcolor`), since the Rust-only per-path colour
group of `\lxSVG@drawpath@unclipped` otherwise painted a pattern-filled path in pgf's last solid
colour: tikz/unit_tests_by_silviu now has Perl's 41 `url(#pgf…)` fills (38 before, with the drawing
hidden in `<svg:defs>`).

Only an argument list with a re-parsed parameter (an inner spec) opens a tail scope: every macro
call paid for it, +1.6 % instructions on 2605.29846, +0.14 % after (388.20e9 against 387.66e9).

Residuals: a keyval value's rest (`Parameter::reparse`, keyvals.rs) is still dropped; `\vspace` is
Perl's `\vskip #2\relax` (latex.ltx:9362-9372 sets `\sp@ce@skip` first, so TeX reads the tail
before the skip); a column type's tail is dropped (warned; 2605.19386's `;{1pt/1pt}` under the
arydshln stub); `\lx@@genfrac`'s `after_digest` reads its numerator after a re-inserted tail; a
constructor with a `DigestedBody` (`\@@tabularx`, `\@@supertabular@`) hands its width's tail back
after the whole body; a `\scantokens` mouth left open on top of a braced argument's own hides that
argument's rest from `take_rest_of_mouth` (the forced close drops it); `DefaultUnits` read bare (no
spec does) takes the rest of the CURRENT mouth only inside a braced read (`in_braced_read`, a depth
count, not the mouth's identity).

**Readers that took less than TeX** (batch 56ju, worker W18b). The TL-manual A/B of 56jq against
56jr (341 manuals) found seven manuals with new errors and four with new warnings. In each, a
binding read less than TeX, and the rest it left was now either handed back as text or reported by
calc. Counts are 56jt → 56ju, the same dumps:
- multido's Number variable (`\n=1+1`) started as `1.0`. Perl's `readFloat->revert` gives that
  (multido.sty.ltxml:71), and calc's `\yunitlength*\real{…}*\n` then reported the `.0`
  (bardiag.sty:667). It now starts as its text, as multido.tex:193-198 does (`\edef#3{#1}`), and
  `\fpAdd`/`\fpSub` port `\FPadd@` (:217-282) literally, so `2.00+-3.05` steps to `-1.05`,
  `-4.10`, as pdflatex prints them (Perl: `-4.1`); spaces count for nothing. This also fixed
  pst-eucl-docBG's 5 "Expected a relational token … Got `.`" errors. Manuals bardiag 10 → 0
  errors and bardiag2 4 → 0; `expansion/testmultido` reblessed to pdflatex's text.
- nicematrix parses its own preamble. Its `X[<keys>]` (nicematrix.sty:2788-2826) is now `X` after
  the `>{\centering\arraybackslash}` (or `\raggedright`/`\raggedleft`) its `c`/`l`/`r` key gives.
  Before, `X[c,m]` reached the template reader, whose `m` took `]` as its width (manuals
  nicematrix and nicematrix-french, 5 → 0 errors and 43 → 27 warnings each).
- tabularray: `\NewColumnType`/`\NewTblrColumnType` (and `…ColumnRowType`,
  tabularray.sty:3291-3334) are recorded, and the colspec translation expands their uses with their
  arguments. The translation also takes `|[<rule options>]`, `>`/`<` with their `[sep]`, and the
  predefined `j`, `t`, `h`, `f` (:3172-3181, 3336-3346). Before, it bailed on these and the whole
  inner spec became the template, where a `b` or `m` in the key text took a letter as its width
  (non-decimal-units 15 → 4 errors; logoetalab-doc 2 → 0). A `*{<count>}` that is not a literal is
  evaluated as the integer expression tabularray makes of it (:3361, `\prg_replicate:nn`), `*{\n+1}`
  and `*{1+1}` alike (batch 56ke; PixelArtTikz-doc-fr 2 → 0 errors).
- pstricks `\xdef`s an angle argument and reads it whole (pstricks.tex:790-802, 990-999
  `\special@angle`, SpecialCoor): `(x,y)` is its vector's angle, while a node's `(A)` and
  PostScript `! <code>` are unresolved, and the arc or wedge at such an angle is not drawn (as for
  an unresolved coordinate); text after a number is `\pst@checknum`'s "Bad number", 0 substituted
  (:534-563). A PostScript length `{! <code>}` takes its argument whole too (:1001-1004), is 0,
  and is warned about. Perl's `ReadPSAngle` (pstricks_support.sty.ltxml:198-215) reads neither
  form and draws from 0°. The rest came back as text, e.g. `(F)(A_1)` after pst-eucl's
  `\pstMarkAngle` arc (pst-eucl.tex:528), whose `A_1` was a "Script _" error (pst-eucl-docBG 28
  → 6 errors, with the multido item; its 16 "Missing argument PSAngle" were Perl's too).
- graphics' `GraphixDimension` evaluates through calc when calc is loaded. graphics.sty:555-568
  reads `\resizebox`'s lengths with `\setlength`, so perfectcut.sty:119
  `\resizebox{\width}{\ht\cut@boxi+\dp\cut@boxi}` is one expression (perfectcut: 134 → 4
  warnings).
- The pdfcomment binding loads what pdfcomment.sty:25, 1332-1350 loads (etoolbox, refcount, ifthen,
  calc, ifpdf, ifluatex). dataref-doc.tex:120 `\begin{minipage}{#1-2\fboxsep}` relies on its calc
  (54 → 2 warnings). An audit of the 553 bindings with a TL original found five more whose package
  loads calc and whose binding did not: diagbox (diagbox.sty:26), animate (animate.sty:23),
  savetrees (savetrees.sty:194, under its default `lists=tight`), breqn (breqn.sty:56) and jmlr.cls
  (:46). They now load it (`\linewidth-2cm` in a minipage is 288.1pt, not 345pt and a typeset
  `-2cm`).
- `\DeclareTextAccent` reads its slot inside `read_braced` and discards the rest: latex.ltx:9903-
  9904 stores the argument in the command it defines, to be scanned at use (vntex pd1supp.def:5-6
  `{\texthookabove}`; manual amsldoc-vi, 6 → 2 warnings).

The other 331 manuals are unchanged (errors, warnings, recall).
Not regressions:
- latexsheet-esmx's `p{\linewidth-\the\MyLen}` has no calc (its `\usepackage{calc}` is commented
  out), so the column's rest is dropped with the warning above.
- arydshln-man's stub column tails, as 2605.19386.

Open (shared with Perl or print layout): `\width`, `\height`, `\depth` and `\totalheight` are
`0pt` (sect12.rs:184-187; Perl graphics.sty.ltxml:61 "Need to arrange for \width,… to be
bound"), so perfectcut's `\resizebox{\width}{…}` delimiters have `xscale="0"`; tabularray's
`cmd=` key is not applied (non-decimal-units' cells keep their raw `1.2.3`); nicematrix's `X[S]`
(siunitx) key and its ragged2e forms (`\Centering`, nicematrix.sty:2403-2410) are not modelled.

**Guards**: `braced_quantity_tail::*` (26), `pstricks_drawing::hexgame_board_converts` (3 warnings);
repros `expansion-primitives/{braced_value_tail_after_assignment,braced_length_skip_keeps_stretch,
braced_length_tail_box_commands,calc_braced_length_expression,calc_braced_length_invalid_tail,
calc_error_redefined_picture_sty,picture_length_default_units,floatingtable_table_argument,
braced_length_undefined_cs,calc_braced_length_undefined_cs,braced_length_tail_between_rows,
setcounter_undefined_counter_value,multido_number_variable_as_written,nicematrix_x_column_keys,
tabularray_new_column_type,tabularray_rule_options_colspec,pstricks_angle_argument_read_whole,
calc_resizebox_length_expression,pdfcomment_loads_calc_and_ifthen,
declare_text_accent_slot_stays_stored}.tex`.

### 318. `\glossary` entries are index marks; a missing makeindex output is stood in for; MakeIndex builds every list; entries are read as makeindex reads them (Perl: `\glossary` dropped in text; "No file"; one list; default characters)

**Rust** (batch 56js, worker W19). Perl's side is KNOWN_PERL_ERRORS #281 and #282.

- **`\glossary`** (sect11.rs) reads its argument `SanitizedVerbatim`, as `\index` does, and yields an
  invisible `ltx:indexmark inlist="glo"`. latex.ltx's `\@wrglossary` is `\@wrindex` writing to the
  `.glo` file, and the file extension names the list, as `idx`, `toc` and `lof` do.
  - The mark is made only after `\makeglossary` (latex.ltx:17727-17736; doc.sty's `\RecordChanges`)
    has allocated `\@glossaryfile`. Before that, `\glossary` is latex.ltx:17742's `\@index`, which
    reads the entry and drops it (:17726).
  - guitar.dtx:397-399's `\changes`, recorded without `\RecordChanges`, `\protected@edef`s to
    `\def{\relax}`. Digesting it raised "Missing control sequence inserted" (repro
    `index/glossary_digests_argument_guitar.tex`).
- **The makeindex stand-in** (`\lx@makeindex@output`, latex_constructs_rust_only.rs §14). It runs in
  `\@input@`'s missing-file branch, only for `\jobname.ind` and `\jobname.gls`, and only when
  `\makeindex`/`\makeglossary` allocated the stream and the environment is defined. It digests what
  gind.ist/gglo.ist write around the entries: `\begin{theindex}` or `\begin{theglossary}`, and its
  postamble.
  - While `\theindex` is still our constructor, the stand-in is `\begin{theindex}\end{theindex}`.
    Our `ltx:index` is the placeholder, as for makeidx's `\printindex`.
  - When a package has replaced the environment, the stand-in runs it as LaTeX's `\begin` would.
    Examples are doc.sty:570-581 with hypdoc.sty:184-212 (the prologue and the columns), and
    makeglos.sty:10-12 (`\section*` plus `description`). The code is `begin_environment_by_macro` /
    `end_environment_by_macro`, shared with `\begin`/`\end` in sect01.rs. The stand-in then puts
    `<ltx:index lists="idx|glo" xml:id="<docid>.<list>">` after the environment, with no backmatter
    relocation.
  - Scope (coordinator ruling A, 2026-09-25): only this path. The kernel `{theindex}` DefEnvironment and
    makeidx's `\printindex` are unchanged. KOMA, memoir and imakeidx manuals are byte-identical apart
    from timestamps.
- **multicols** reads `{multicols}{Number}[][Dimension]` (multicol.sty:145, :172-186). doc.sty's
  `\begin{multicols}\c@IndexColumns[\index@prologue][\IndexMin]` had printed `[]` and raised two
  "Missing number" warnings.
- **Scan and MakeIndex** (scan.rs, make_index.rs). Scan registers one `INDEX` entry per list:
  `INDEX:<keys>` for `idx` (Perl's key, unchanged) and `INDEX@<list>:<keys>` for any other list.
  MakeIndex builds each `ltx:index` from the lists in its `lists` attribute, defaulting to `idx`. The
  webpage stylesheet's head keywords take only `idx` phrases.
  - Scan stores its titles, captions, tags, notes, anchor titles and declaration descriptions
    without their `ltx:indexmark`s, as Perl's `cleanNode` does (Scan.pm:206-214; the port had
    skipped it). A mark in a caption had reached the list-of-figures entry (`1Cap icap`) and one in
    a section title the links' tooltip (`1 Title isec`).
- **Reading an entry** (`process_index_phrases`, `IndexChars`, `absorb_index_verb_runs`; coordinator
  ruling B, 2026-09-25).
  - The makeindex characters are a parameter. `\glossary` uses the declared ones: doc.sty's
    `\levelchar`/`\actualchar`/`\encapchar`/`\quotechar` (doc.sty:521-524), where a macro whose body
    is a single character is a declaration. The `.glo` file needs a style that names its
    `\glossaryentry` keyword (gglo.ist), and `\changes` expands the characters before writing
    (doc.sty:627).
  - An `\index` entry uses the declared characters only when it shows them: spelled with the macros
    (doc.sty, hypdoc.sty:351, amsldoc.cls:88), or holding the declared actual character at top level
    (nlctdoc.cls:496-570). Otherwise it uses makeindex's defaults. The style is chosen per file,
    outside the document: tcolorbox's documentation library writes `@` for a default run inside
    ltxdoc manuals (keytheorems-doc, csvsimple-l3).
  - makeindex's quote character acts inside a `\verb` run too, in its delimiter slot and in its body.
    A quoted `\` before the quote stays a backslash (amsldoc.tex `"\"|`); an unquoted `\"` stays the
    escape.
  - `\string\verb` is the command.
  - A run whose head a macro supplied is `\protected@write`-expanded (`do_expand_partially`) inside
    the entry's frame, with control symbols and undefined words inert. A robust command is written by
    its name, without `\protect`.
  - When the rest of the entry after such a run does not balance, the run is read literally. doc.sty's
    `\LeftBraceIndex` closes its run inside a `{…}` that the re-read's `\iffalse…\fi` would hide.
  - A Fatal inside either stays Fatal.
  - The run is grouped, so its typewriter font stops at its end. This changes existing entries
    toward pdflatex: `tilda ( ~)`, `\; (espacement)`, and 354 entries of latex-via-exemplos.
  - An encap with arguments takes its command as the style (hypdoc's `hdpindex{main}` becomes
    `hdpindex`), and the style is read from the argument's tokens.
  - A `_`/`^` of the display that is active in the document (underscore.sty) is active.
  - Outside math, a `_`/`^` that reads as text is `\textunderscore`/`\textasciicircum`. This covers
    the character inside `\changes`'s `\@sanitize`, where an other `_` would print through OT1's slot
    as `˙`.
  - A subscript `_` stays a subscript.
  - A sort key's implicit braces (`\bgroup`/`\egroup`, doc.sty's brace entries) are text, like its
    undefined words.
  - An entry whose groups do not balance is discarded with `Warning:malformed:indexentry`
    (Perl pool:4332-4336, which the port had dropped). makeindex rejects such an entry too:
    ltmath.dtx v1.2i `\cs{\bslash}` is absent from the golden change history.
- **`\url`** (url_sty.rs, hyperref_sty.rs) is `protected`, as hyperref.sty:4801
  `\DeclareRobustCommand*` makes it, and as url.sty:166 `\Url@unmove` treats a moving argument.
  So are `\path` and every `\DeclareUrlCommand` command. `\changes`'s `\protected@edef` had
  expanded the reader (ltsect.dtx v1.1b).

**Measured** (228 TL manuals: every `\PrintIndex`/`\PrintChanges`/`\printindex` or
`unexpected:glossary` manual; the 56jp release → this batch, mono HTML, each manual's sweep preload, recall by
`pdf_recall.py --min-len 4`):

- No manual loses recall, and 49 gain.
- Distinct PDF words found go from 214,097 to 215,171.
- Errors go from 3,053 to 3,051 (guitar 1 → 0, paracol-man 3 → 0), and warnings from 13,643 to 9,573. `Warning:unexpected:glossary`
  goes from 4,079 to 0.
- Stream A cluster 1 (the 63 manuals with missing change-history or index-prologue words, plus
  source2e) gains 960 words: 54,992 → 55,952.
  Examples: biblatex-chem 76.5% → 99.7%, biblatex-ieee 81.1% → 99.5%, source2e 95.8% → 98.9% (127 s → 117 s),
  sunpath 88.7% → 96.8%, joinbox 78.4% → 83.0%.
- New diagnostics. robustsample raises 2 errors (residual below). compsci gains `Missing phrases in
  indexmark`: doc.sty writes its `|...|` entry unquoted, so the key is empty. lettre
  gains `Missing index see-also term`, because its index is built now.
- The arXiv `\glossary` witnesses (cs/9809003, math/9608214, nucl-th/9311001) call no
  `\makeglossary`. Their entries are dropped as in LaTeX, without the `unexpected:glossary`
  warnings. Their HTML text and head keywords are unchanged.

**Residuals** (SYNC_STATUS): doc.sty's code-line index is still missing. Its writers
(`\codeline@wrindex`, hypdoc `\HD@codeline@wrindex`, l3doc `\__codedoc_index_*`) write
`\indexentry` to `\@indexfile` themselves and never reach `\index`, so a dtx manual's index lists
only its `\index`-level (usage) entries. imakeidx documents get no index at all, before and after
this batch. robustglossary's `&`-column entries raise `Stray alignment` at the mark (robustsample, 2
errors; RED repro `index/glossary_ampersand_robustglossary.tex`). xindy styles' separators
(makeglos.xdy `:`) are read as text. `\@SpecialIndexHelper@` keys of control symbols (`\cn{\\}`)
remain garbled. A `\changes` whose nearest identified ancestor is the document (biblatex-ieee's
follow its bibliography in vertical mode) refers to the document itself, so its link repeats the
document title as its text. That is MakeIndex's convention: its refs ask for `show=typerefnum`
(Perl MakeIndex.pm:303-304), a document has none, and CrossRef falls back to the title
(CrossRef.pm:686). biblatex-ieee's change history shows its title 56 times where the PDF prints
page "4".

The re-read of the written entry is modelled only for `\string\verb`. amsldoc.cls:99-103's
`\string\texttt{#2}` prints `“texttt…` (RED repro `index/index_string_command_amsldoc.tex`), and
doc.sty's brace entries show their internals. `\verb*` in an entry shows no visible spaces. Under
hypdoc the encap style is `hdpindex`/`hdclindex`, not doc's `usage`/`main`; the CSS styles neither. A
body-grabbing `theindex` (environ) would not see an `\end{theindex}` in the stand-in.

**Guards**: `cluster_package_guards::doc_changes_index::*` (13). Repros
`tools/perfect_kernel/repros/index/{doc_changes_index_prologue, index_quote_verb_lshort,
doc_index_entry_writers, index_string_verb_amsldoc, glossary_list_env_makeglos,
glossary_running_text_keywords, changes_url_moving_argument, glossary_digests_argument_guitar,
changes_unbalanced_discarded, glossary_standin_in_item_and_cell,
indexmark_cleaned_from_titles_captions}.tex`; RED `index_string_command_amsldoc.tex` and
`glossary_ampersand_robustglossary.tex`.

### 319. A node's box follows Perl's `appendNodeBox`, one flat list in horizontal mode (Perl: a new list per append)

Perl records on every element the box that created it, and `openElementAt` (Core/Document.pm:1861)
extends the box of each auto-opened ancestor with the box of anything opened inside it
(`appendNodeBox`, :1685-1700); text arriving at an auto-opened node does the same
(`openText_internal`, :1136-1150), and `removeNode` takes a removed node's box out again
(`removeNodeBox`, :1704-1723). An auto-opened `ltx:picture` (latex_constructs.pool.ltxml:4943-4950)
and `svg:foreignObject` (TeX_Box.pool.ltxml:379-424) are sized from that box when they close, so it
decides every tikz node's foreignObject and every picture a drawing object opens outside `{picture}`.
**Rust** (batch 56jo, worker W12; `latexml_core/src/document.rs` `append_node_box`, `remove_node_box`,
`open_element_at_with_box`) ports it:

- Perl combines with `List($origbox, $box, mode => …)` (List.pm:31-55), which in horizontal mode
  flattens a horizontal list into its boxes. Rust keeps a node's horizontal box as one flat list the
  node owns and pushes onto it in place while no one else holds it (Perl builds a new list per
  append, O(n²) in a paragraph; a shared list is rebuilt, as Perl does). A box with no `mode` counts
  as horizontal: a Rust `Tbox` records none, where Perl's `Box()` records the `MODE` it was made in
  (`horizontal` for paragraph text). In every other mode the boxes nest in pairs, as in Perl.
- `replace_node` (and so `unwrap_nodes`) moves no box: Perl's `replaceNode` (:2011-2025) moves the
  replaced node into a document fragment by its first `replaceChild`, so its `removeNode` finds no
  parent element, and its closing `appendNodeBox` map runs over an emptied array. Only a node
  replaced by nothing leaves its parent's box. The `\@framebox` unwrap (latex_constructs.pool.ltxml
  :4729) inside an auto-opened node keeps the frame's size.
- `cleanup_math` unwraps a text-only Math with `replaceTree`'s boxes (TeX_Math.pool.ltxml:219;
  `Document::replace_node_as_tree`): the Math's box leaves the parent, each element piece is recorded
  with its own box (a text wrapper, made in the XMText here, with the parent's, as Perl makes it in
  the parent), and an `XMHint`'s spaces with none. An auto-opened `svg:foreignObject` left with no box
  at all records the box being absorbed (as text arriving there does): Perl turns that one into
  `svg:text` (TeX_Box.pool.ltxml:386-389), which needs no size, and the Rust foreignObject cleanup has
  no such branch (SYNC_STATUS).
- `append_tree` keeps a copied node's own box (Perl copies the `_box` attribute, :2110-2116).
- A math ligature's merged tokens take their box out of no enclosing box (`apply_math_ligature`):
  Perl's `removeNode` of each takes its box out of the enclosing boxes (:1197 → :1788-1789) and so
  out of an aligned cell Math's `tex` (`=414\times 0^{-3}` for `= 2.414 \times 10^{-3}`;
  KNOWN_PERL_ERRORS #272, 153 arXiv papers in the 56jo A/B). A renamed node (`rename_node`, Perl
  :2064) and a copied one (`remove_node_keeping_box`: `\sideset`'s nucleus, which Perl moves) likewise
  leave their box in place: an aligned cell keeps its `\mbox`, `\raisebox`, `\hbox`, `\resizebox`
  and `\sideset` (Perl drops the first three).
- With the node boxes' full widths, Perl's figure-panel heuristic (latex_constructs.pool.ltxml
  :3331-3343) starts the row after a standalone panel as full as that panel (a subfigure grid's
  caption line); Rust starts it empty, as pdflatex does, and puts a small panel it merges into the
  next block first in that block, where Perl appends it (KNOWN_PERL_ERRORS #274).

Sizes are Perl's in every probe, including `\node{\fbox{$x^2$}}` (foreignObject 23.52×20.67 px), `x
\put(0,0){A} $\quad\text{ab}\quad\text{cd}$ y` (50.74) and `\put`s nested in auto-opened text.
Residuals: `replace_tree`/`replace_tree_detach` do not take the old node's box out of the parent as
Perl's `replaceTree` does; a synthetic 500-deep nesting of `\put` pictures (their boxes `restricted_horizontal`, so
Perl's nested pairs) takes 25-31 % more memory and 55-70 % more time than before W12 (stress500 1.52 vs
1.22 GB, 3.8 vs 2.2 s; Perl 9.5 GB, 594 s); a constructor
template's `?#N` holds for "0", where Perl's string test does not (Constructor/Compiler.pm:166;
SYNC_STATUS).

**Guards**: `node_box_append::*` (13), `picture_sizing::auto_opened_picture_is_sized_from_its_box`,
`cluster_schema::empty_node_foreign_object_is_sized`, tests/graphics/xytest; repros
`graphics-tikz/picture_autoopen_*.tex`, `graphics-tikz/node_box_*.tex`.

### 320. glossaries' `\glsadd` leaves a location-only `ltx:glossaryref`, and the reference wraps survive glossaries-extra (Perl: the entry is not listed)

**Rust** (batch 56jt): `\glsadd` emits `<ltx:glossaryref inlist key show="none"/>` for a defined
entry (glossaries_sty.rs), so MakeIndex lists it as makeindex does every written entry
(KNOWN_PERL_ERRORS #276). `show="none"` is a reference with no text: CrossRef's
`fill_in_glossaryrefs` skips it (neither filled nor `ltx_missing`) and
`LaTeXML-inline-xhtml.xsl` renders nothing for it. In the preamble the reference is queued with its
list and label expanded, apart from the entry definitions, and carried into the first paragraph
that opens in the body (batch 56jt) by a one-shot `\everypar` that restores the one it replaced —
LaTeX's location for a preamble `\glsadd` is page 1 (glossaries-user.tex:25585-25590). Emitted at
`\begin{document}`, the reference (`Inline.class`) opened a paragraph LaTeX does not have: an empty
`ltx:para` when the body starts with vertical material, which shifted every later document-level
paragraph id. When no paragraph opens, or the `\everypar` is replaced before one does, the
references are emitted at `\end{document}`, where they open a trailing paragraph of their own. The
`\everypar` is restored only while it still holds just the one-shot, so an addition made to it
meanwhile is kept. References are lost when the first paragraph opens in a box whose content is
discarded, or when the `\everypar` digestion fails (`stomach.rs` `fire_everypar` swallows the
error). In the body `\glsadd` opens a
paragraph in vertical mode as LaTeX's `\@gls@adjustmode` (`\ifvmode\mbox{}\fi`, glossaries.sty:5294)
does. In math (batch 56jt) the reference is placed where a `^` float puts an element, at the
nearest enclosing level that admits it, after the formula (the `p` of inline math); a display
formula has no such level, so there none is made (SYNC_STATUS). The `\@gls@link` wrap in math is
the original alone plus that reference: the wrap there was an `XMText` atom of the formula,
spelling `\lx@glossaries@gls@link{…}` in `alttext`/`tex=` (KNOWN_PERL_ERRORS #276). After
glossaries-extra loads, the binding's `\@gls@link` wrap is re-applied over
glossaries-extra.sty:3465 and the `\glsadd` wrap moves to its `\@glsadd` (:3581), through
`\AddToHook{package/glossaries-extra/after}`.

**Perl** wraps only `\@gls@link`, so an entry added by `\glsadd`/`\glsaddall` is missing from the
list, and a glossaries-extra document lists no entry. Witnesses ualberta/ualberta (83.9 → 95.7 %),
glosmathtools/sample_glosmathtools_en/_fr (+1.5), iodhbwm (+1.1); the 38 corpus manuals with a
glossary definition keep their errors and warnings.

**Guards**: `stream_a_recall::{glsaddall_lists_every_entry,
glossaries_extra_lists_used_and_added_entries, gls_in_math_typesets_the_term_alone,
preamble_glsadd_opens_no_paragraph}`.

### 321. The bibliography formatter separates the fields of one block (Perl: run together)

**Rust** (batch 56jt, `latexml_post::make_bibliography::get_fmt_spec`, shared by every `.bib`
bibliography): an editor follows the author in the name block after ", "; a website's title and
its parenthesized type follow the name after a space, as does `software`'s type after its key; the
elements a row's path matches are joined — `ltx:bib-note` (`note`, `howpublished`, `addendum`,
`annote`) as units with ". " (`Formatter::Units`, BibTeX's `new.block`, biblatex's `\newunit`),
any other row's (`organization` + `institution` in `ltx:bib-organization`) as a list with ", "
(`Formatter::Any`); after an element whose printed text already ends in a mark only the space is
printed (a formula or a reference CrossRef fills later ends in none), and an
element with neither text nor child elements prints nothing (a `\citet` in a note is an empty
`ltx:bibref` until CrossRef fills it). The text of every field is unchanged.

**Perl** gives these rows `""` punctuation (MakeBibliography.pm:708, :752, :769, :781-787, :794) and
concatenates a row's elements (`do_any`, :550, :688), and LaTeXML.css adds no separator between
the `ltx_bib_*` spans: "George Frideric HandelAtlanta Symphony (Ed.)", "V. JacobsonModified
TCP…(Website)", "Note: NovemberhowLimanote" (KNOWN_PERL_ERRORS #280). Witnesses biblatex-ieee,
biblatex-chicago cms-notes-sample (88.6 → 90.4 %), cms-trad-sample, cms-dates-sample,
biblatex-caspervector; 311 bibliography manuals keep their error and warning counts.

**Guard**: `stream_a_recall::bibliography_fields_are_separated`.

### 322. A tabular template is read as its argument and parsed in a mouth of its own (Perl: read from the document, token by token)

**Rust** (batch 56jz, `latexml_core/src/alignment.rs` `read_alignment_template`): the template
is one balanced group (or a single token), as latex.ltx's `\@mkpream` takes `#1` and tex.web's
preamble ends at its `}`. It is parsed in a sourceless mouth, which the locator walk passes
over, so diagnostics point at the template's line in the document. A column type's argument
(`p{..}`, `@{..}`, `>{..}`, a `\newcolumntype`'s parameters) and a macro's expansion are read
inside the template and cannot run past its end. A column type that takes an argument, with
the template ended, is an error ("Missing argument of the column type 'p' at the end of the
tabular template; token ignored") and is dropped, as array drops it (array.sty:331, :414;
latex.ltx:16643-16644 reports "Missing p-arg"/"Missing @-exp").

**Perl** (Alignment.pm:895-921) reads the template from the document's input, counting braces,
so such a column type reads the template's `}` and scans on through the document, which is
lost ("Input ended while environment tabular was open", KNOWN_PERL_ERRORS #292).

The template's braces still count while it is parsed (the alignment brace counter is raised
around it), so an `&`-valued token inside a nested template does not end the enclosing cell. The
same reader serves `\multicolumn`'s template (`\lx@alignment@multicolumn`, tex_tables.rs:548):
`\multicolumn{1}{c@}{x}` keeps `x`, and `\multicolumn{3}p{3cm}` makes `3cm` the cell body, as
LaTeX's `\long\def\multicolumn#1#2#3` reads it (latex.ltx:16603).

**Not modelled**: for a missing `p`/`m`/`b` argument pdflatex also inserts `#` into the empty
template after the `&` its preamble builder added ("Missing # inserted in alignment preamble"),
so a second column exists there; here the column is dropped, and a row's extra cell takes the
default template. (`{c@}` gives pdflatex only "Missing @-exp in array arg".)

**Guards**: `regress_2605_clusters::tabular_template_missing_argument_stays_in_the_template`, `::multicolumn_template_missing_argument_keeps_the_cell`, `::nested_template_amp_does_not_end_the_enclosing_cell`.

### 323. The box commands start the paragraph, as LaTeX's `\leavevmode` does (Perl: `\mbox`, `\fbox`, `\colorbox`, `\fcolorbox`, `\textcolor` do not)

**Rust** (batch 56kb): `\mbox`, `\@makebox`, `\@framebox`, `\raisebox` (sect12.rs) and color's
`\fcolorbox` (like xcolor's) carry `enter_horizontal`; color's `\colorbox` and `\textcolor` open with
`\leavevmode` (color.sty:104, 163-164). A box at a paragraph's or an item's start starts the
paragraph: `\everypar` fires before the box (pdflatex `[EP]Box after.`, where Perl gives
`Box[EP]after.`), the space after it is kept, and `\ifvmode` after it is false. In math the step
is a no-op. (`\fcolorbox`'s body, `internal_vertical` in Perl, also ended the running paragraph
until 57r set it `restricted_horizontal`, the `\hbox` it is; KPE #336.) Same shape as #97 (`\hrulefill`/`\dotfill` keep the kernel's `\leavevmode`).

**Perl** has enterHorizontal only on `\makebox` and `\raisebox` (latex_constructs.pool.ltxml:
4658-4667, 4800-4802); the rest drop the space after the box (KNOWN_PERL_ERRORS #293).

**Witnesses**: matapli-doc (`\item \Verb+FiraSans+ et`), the biblatex manual (ltxdockit's `\sty`),
titlesec (`\fbox{2.9}` version marks), etoolbox-DE, arXiv 2605.21713 (`\colorbox` runs),
2605.21322 (`\makebox[0pt][c]` figure panels), 2605.18438, 2605.29836.

**Guards**: `mbox_argument_is_bounded::box_commands_start_the_paragraph`,
`::boxes_fire_everypar_first_and_stay_out_of_math`.

### 324. biblatex bibliographies: standard.bbx's entry-type aliases, an article's editors, a host's edition (Perl: book format, no article editor, no host edition)

**Rust** (batch 56kc):
- The biblatex binding's `\DeclareBibliographyAlias{alias}{type}` (a no-op before) defines the BibTeX
  reader's `\bib@entry@<alias>@alias` (Perl's alias mechanism, BibTeX.pool.ltxml:67-68, 121-123),
  resolving an aliased target at declaration, and the binding declares standard.bbx:740-752's list (the
  style's `.bbx` is not read for the native styles). Only a target that is an entry type of biblatex's
  data model (blx-dm.def:419-471) reaches the reader: raw-loaded styles also alias to their own drivers
  (chicago-notes.bbx:2261-2265 `{legal}{cite:legal}`), which are not types, and `*` is the formatter's
  existing fallback. Both the `.bib` session (it preloads the article's packages) and biber's `.bbl`
  (read in the document's session, where a preamble declaration also applies) go through it: `@review`
  is formatted as an article, `@bookinbook` as an inbook (its `booktitle` then finds the inbook
  handler), `@software` as a misc. A raw-loaded style's retyping to a real type now applies too
  (mla-strict.bbx:355-370 `misc`/`online`/`patent` → article), as that style prints them.
- A biblatex bibliography's article prints "Edited by …" after the journal and before the pages
  (make_bibliography.rs `get_fmt_spec`, gated on the bibliography's `bibstyle`); a `.bst` article prints
  no editor (plain.bst), so BibTeX bibliographies are unchanged.
- The incollection format prints the host's edition (`\bib@field@inbook@edition` /
  `@incollection@edition` file it under the host `bib-related`, where no row read it): "2 edition" of
  the Oxford English Dictionary entry, ungated — a `.bst`'s `incollection`/`inbook` print the edition too
  (plain.bst "…, third edition, 2007").
- In a biblatex bibliography the author-less label and sort key fall back to the host's editor as well
  (`label_editors`, host editor first, then the entry's own — tried in order, not joined): an aliased `@inreference` files its editor under the host, and biblatex labels
  by the editor wherever it is (biber "Grabowski (2022)", biblatex-apa-test 10.2:35d, which the aliases
  alone had turned into "(80)"). A `.bst` incollection keeps Perl's author-or-key label.

Not done: `editortype` as the editor's role (biblatex.def:2875-2892). Given in the reader it broke the
author-year label of author-less entries, `.bst` bibliographies and undecoded (ctex) roles; RED repro
`tools/perfect_kernel/repros/index-bib/biblatex_editortype_role.tex` (SYNC_STATUS).

**Perl**: KNOWN_PERL_ERRORS #294.

**Witnesses** (second review of 56kc, before → after, words with tags stripped; error, warning and fatal
counts unchanged everywhere): cms-notes-sample +112, cms-dates-sample +100, cms-trad-sample +87,
biblatex-apa-test +153, mla examples +175, caspervector +45, fiwi +23, cms-legal-sample 0 (the `cite:`
targets filtered), arXiv 2605.16053 +4 (`@software` gains its author), 2605.00219/.18215/.02768/.06336/
.09091 identical.

**Guards**: `bibliography_names_fields::biblatex_entry_type_aliases_take_the_standard_drivers`,
`::biblatex_alias_declarations_retype_only_to_entry_types`, `::biblatex_articles_print_their_editors`,
`::biblatex_inreference_keeps_its_editor_label_and_edition`, `::biblatex_label_prefers_the_host_editor`,
`::bst_incollection_prints_its_host_edition`.

### 325. A `\setlength` operand goes through a package's or document's own `\setlength` (Perl: read by the binding)

**Rust** (batch 56kf): latex.ltx hands the user lengths of eight commands to `\setlength`: `\hspace`
(`\@hspace` 9425), `\\[..]` (`\@newline` → `\@vspace@calcify`, 9254-9261), `\makebox` (`\@imakebox` 16101),
`\framebox` (16199), `\parbox` (width and height, 16252-16255), `minipage` (16308), `\rule` (16363-16365) and
`\raisebox` (16380-16390); graphics.sty:555-568 does the same in `\Gscale@box@dd`/`@dddd`, behind
`\resizebox`. Their bindings declare these arguments `SetlengthDimension`/`SetlengthGlue`
(base_parameter_types.rs; sect01.rs `\lx@newline`, sect12.rs, graphics_sty.rs; graphicx's
`GraphixDimension` takes the route explicitly). When `\setlength` is a macro, not the kernel's or
calc's primitive, a package or the document has redefined it, and such an argument is read through it:
`\setlength<register>{<argument>}` is digested (`read_through_redefined_setlength`, parameter.rs, from
`Parameters::reparse_argument`) and the register read back. The registers are its own
(latex_constructs_rust_only.rs), as `\sp@ce@skip` and `\@tempdima` are latex.ltx's: `\lx@braced@skip` for a
Glue, `\lx@braced@dimen` for a Dimension, whose scan stops before `plus` as `\@tempdima`'s does. What the
assignment leaves in the input is typeset where TeX typesets it, before the command's output, except
after `\\[..]`, where TeX typesets it after the break (inside `\vadjust`, latex.ltx:9260) and it lands
before the break here: an authoring slip (`\\[12pt w]`) under a redefined `\setlength`. A read inside
the redefinition takes the direct path. bxcalcux.sty's `\bxcx@decl@patch\setlength` then parses its
units (`\newcalcunit{tm}{0.05em}`: `\hspace{6tm}` is 3pt, `\makebox[20tm]` 10pt, `\\[2tm]` a 1pt break,
pdflatex's sizes). Plain TeX (miniltx's `\setlength` macro) keeps the direct read: the registers are
LaTeX-only.

A binding's plain `Dimension`/`Glue` stands for no `\setlength`, and a redefined one does not read it
(with calc loaded, #317's evaluator still reads every braced length as calc's operand; narrowing that
to the declared operands is K14 work). Taking every braced length through a redefined `\setlength` (the
first cut) recursed to the pushback limit inside a nested `\pgfpicture`, whose `\setlength` is
`\pgf@setlength` saved as its own original (pgfcorescopes.code.tex:238-241): `\pgfsetlinewidth{..}`,
which TeX reads through pgfmath, called it (arXiv 2605.15377, 2605.22769, pgf-blur's `blur shadow`,
tikzlibraryshadows.blur.code.tex:181). A declared operand met there still recurses (`\hspace` in the text
of a node inside a `\pgfpicture` nested directly in a picture): pdflatex dies there too ("TeX capacity
exceeded [grouping levels=255]"), so the Fatal is faithful. A tikz node's text is typeset with the
original `\setlength` restored (`\pgfinterruptpicture`, :751), in pdflatex as here. Not yet declared: the
`p{..}` column width (array.sty's `\@startpbox`, `\setlength\hsize{#1}`) and `\vspace` (a raw
`\vskip #2\relax` macro; latex.ltx:9361-9373 reads it through `\@vspace@calcify`).

**Perl** reads the length itself (KNOWN_PERL_ERRORS #296).

**Witnesses**: sample-bxcalc (13 errors and 26 warnings → 0 and 0: the "`t' invalid" errors and the
"m"/"s+2tm" text typeset after each command since 56jr); bxcalcux and bxcalcize byte-identical; arXiv
2605.15377 and 2605.22769 byte-identical to 56ke (the first cut's Fatal); 125 further arXiv papers
(40 adjustbox, 20 tikz-heavy, 3 realboxes, 60 random) byte-identical (56kf review).

**Guards**: `braced_quantity_tail::redefined_setlength_reads_a_braced_length`,
`::redefined_setlength_scans_a_dimension_as_a_dimen`, `::nested_pgfpicture_lengths_stay_off_setlength`;
repros `tools/perfect_kernel/repros/expansion-primitives/braced_length_redefined_setlength.tex`,
`braced_length_redefined_setlength_dimen.tex`, `braced_length_nested_pgfpicture.tex`.

### 326. The ar5iv profile's pushback limit is the binary's 5,000,000, and a box period is a loop only when its input repeats (Perl: 599,999; no stomach guard)

**Rust** (batch 56kh):
- **Pushback limit.** `ar5iv_sty.rs` sets `pushbacklimit=5000000`: Perl's ar5iv.sty.ltxml:16 has 599,999, and Rust had 650,000. The new value is the default of `latexml_oxide`/`cortex_worker` and matches TeX's `main_memory` (texmf.cnf:820, one word per token). User ruling 2026-09-26: the ar5iv limits are pragmatic heuristics meant to let a large manuscript convert.
  - A macro expansion copies its whole body into the flat pushback (gullet.rs `unread_expansion`). TeX instead pushes a pointer to the token list (tex.web §323 `begin_token_list`). So a finite long list trips a limit that TeX has no counterpart for.
  - pgf expands a whole soft path at once (pgfsyssoftpath.code.tex:66-75, 94-98, 122-131). A 9,000-sample smooth plot is about 650K tokens.
  - Cost: a pushback *runaway* now stops later, still at the limit or the timeout. A pgfmath-per-iteration loop took 2.3 s → 14.9 s; a recursive tikz call 0.6 s → 1.8 s.
- **Box-cycle guard.** The stomach's guard (`cycle_guard_record`, stomach.rs) keeps its box-fingerprint window and that window's uniform-run exemption. A detected box period `w` now becomes a Fatal only when the tokens read per box also repeat, with a period that is a multiple of `w` up to 128 (`SpanHistory::input_period`, cycle_guard.rs: eight repetitions over a 1024-span ring); the Fatal names that input period.
  - Every point of a pgf `plot[smooth]` digests the same content-free boxes: empty `\pgf@process` groups and a stray space (pgflibraryplothandlers.code.tex:37-77). An 11,000-point plot therefore read as a 6-box cycle and was a Fatal. Its coordinates vary in length, so it is not a cycle now.
  - Runaways that read the same input per period still stop at the 50K-box floor:
    - ` {}\x`, 0.3 s;
    - the alternating ` {} {} {}\y`/`\relax\x`, input period 12 over a 2-box window, 0.3 s;
    - 2508.07407's cloud loop, 1.5 s;
    - the `\loop`, `\@whilenum`, `\@whiledim`, pgf-loop and 2201.09268 arc-loop reductions, 0.2-1.8 s;
    - a `\loop` whose counter wraps every 33 iterations (input period 66), 0.5 s.
  - Cost: a runaway whose input never repeats (a `\@whiledim` re-reading `\the\dimen@`), or repeats with a period longer than 128 boxes, now falls to the count cap: 0.4 s → 8.8 s, 3.4 GB, under `--max-memory=8192`.
  - Dead ends:
    - Excluding content-free boxes blinded the guard to ` {}\x` and to the cloud loop.
    - XOR-ing the span into the box fingerprint defeated the uniform-run exemption: a finite `\loop` printing 60,000 x's through `\ifodd` branches became a Fatal. It also pushed combined periods past the 10-box window (the alternating runaway fell to the count cap, 3.2 s, 3.7 GB).
    - Dropping spaces under `\nullfont` is unfaithful: TeX appends glue for every space (tex.web §1041).
    - Hashing token content lets a counter defeat the cloud loop.

**Perl** has no stomach guard, and checks the pushback only in `readToken` (Gullet.pm:300-306), not in `readXToken`/`readBalanced`.
- Perl with ar5iv stops on the finite 9,000-sample plot: `Fatal:timeout:pushback_limit 599999` after 107 s (KNOWN_PERL_ERRORS #297).
- It converts `\edef\a{\a\a}` ×17.
- Without ar5iv, Perl's s9000 path matches ours byte for byte.

**Witnesses**:
- **Manual profile:**
  - tkz-grapheur-doc-en goes from a Fatal (27 words) to 0 errors and 11,871 words, in 108 s.
  - tkz-grapheur-doc-fr goes from 29 words to 17,749, in 135 s.
  - The 9,000-, 11,000- and integer-step plots all convert.
- **Unchanged:** the 3,000-sample plot is byte-identical.
- **arXiv:** the 3,003-paper A/B of 56kg+56kh (first design) is byte-identical on every paper.

**Guards**:
- `perfect_kernel_batch56::smooth_plot_past_the_box_cycle_floor`
- `::runaway_space_group_loop_is_still_a_loop`
- `::a_finite_loop_of_one_letter_is_no_digestion_loop`
- `::a_long_expansion_fits_the_pushback`
- `cycle_guard::tests::{constant_spans_repeat_with_any_box_period, spans_repeating_at_a_multiple_of_the_box_period_repeat, varying_spans_do_not_repeat}`
- `fatal_salvages_partial_document::recoverable_fatal_keeps_the_already_digested_document` (unchanged)

### 327. A box's measured size is its constructed size (Perl: `\framebox[w]` measured as an object address)

**Rust** (batch 56kj, theme 11, K18 step 1): the size code (`get_size`) reads only typed sizes, so box sizes are stored as `Dimension`s, as Perl stores them. The template renders them to the same attributes.

**Parity changes, not divergences** (Perl already behaves this way):
- `\parbox` keeps `width`/`totalheight` typed. Before, a 0.1pt round trip made `\parbox{1in}` 72.3pt; Perl and pdflatex give 72.26999pt.
- `\raisebox` gets Perl's `raisedSizer` (TeX_Kern.pool.ltxml:86-92), shared with `\raise`/`\lower`.
- The graphics boxes store Dimensions:
  - scaled boxes truncate, as Perl's `multiply` (Number.pm:107-109);
  - rotated boxes round, as Perl's `Dimension($float)` (Dimension.pm:40-47).
- A list breaks lines only at its own width (Perl `computeBoxesSize`, Font.pm:683), so `\makebox[1em]{aaa bbb ccc ddd}` measures 6.94444pt / 0pt, as in Perl and pdflatex.
- Golden `graphics/graphrot.xml` now equals Perl's own `LaTeXML/t/graphics/graphrot.xml` (lines 78, 125, 172, 256, 306, 312).

**Beyond Perl:**
- `\framebox[w]` stores its width typed (sect12.rs), so it measures w: latex.ltx:16196-16229 puts the frame inside the width. Perl stores the digested argument, which measures 1568748557.57892pt, an object address.
- `node_box_append::replacing_a_node_keeps_the_boxes`: the `\parbox{2cm}` node's picture is 97.93 (Perl 97.92). Its box is 63.70549pt, pdflatex's to the sp.
- rotman's isorot wheel (a `\savebox` holding `{sideways}`) is 93.14pt wide (Perl 163.75pt, pdflatex 101.99pt). The remaining gap is the document's 11pt font, which is measured here at 10pt.

**Open (K18 step 2):**
- `\raisebox`'s `[height][depth]`. latex.ltx reads them after the box, with `\height` etc. measuring it; here `\height` is a 0pt stub, as in Perl. RED repro `boxes-groups/raisebox_optionals_measure_the_box.tex`.
- `\resizebox{1em}` is 10.00002pt (Perl's); pdflatex's 10.00081pt comes through `\Gscale@div`.
- makecell and diagbox still store string sizes.

**Guards**:
- `perfect_kernel_batch56::{box_sizes_are_their_constructed_sizes, a_box_width_does_not_break_its_contents}`
- repro `boxes-groups/box_dimensions_measured.tex`

### 328. A box command's size arguments are read with the box set, `\width`/`\height`/`\depth`/`\totalheight` measuring it (Perl: `0pt` text, read before the box)

**Rust** (batch 56kl, theme 11, K18 step 2): the arguments follow latex.ltx's `\@begin@tempboxa` (16085-16094). The affected arguments are:
- `\makebox`'s and `\framebox`'s width;
- `\parbox`'s height;
- `\raisebox`'s `[height]` and `[depth]`;
- the lengths of `\Gscale@box@dd`/`@dddd`.

**How it works.** Each is declared `TempboxaDimension`, a `SetlengthDimension` with `after_box` (base_parameter_types.rs). Only a constructor whose last argument is the box honours it; a macro reads it at once.
1. `Parameters::read_arguments_and_digest` reads such an argument's raw tokens and leaves its slot empty.
2. It digests the box. A plain `{}` box is digested in a group, as TeX's `\hbox{`; `HBoxArgContents`/`VBoxContents` open their own.
3. Then `within_tempboxa` (parameter.rs), in a group:
   - puts the box in `\@tempboxa`;
   - digests `\lx@tempboxa@sizes` (latex.ltx:16088-16093 verbatim: `\def\width{\wd\@tempboxa}`…, internal dimensions, so `2\width` is exact to the sp);
   - re-parses each waiting argument through the `SetlengthDimension` path (calc, or a redefined `\setlength`).

This all happens before the constructor's `properties` and sizer read the arguments.

The `\raisebox` sizer applies `[height]`/`[depth]` over Perl's `raisedSizer`. Outside a box command the four stay Perl's `0pt` stubs (sect12.rs).

**Effect:**
- `\makebox[2\width]{x}` is 10.6pt wide (was 20pt).
- `\makebox[\value{c}pt]{\stepcounter{c}x}` sees the step.
- siamart's `\raisebox{0pt}[\height][0pt]` keeps the height and drops the depth (100 papers of the 3,003-paper 2605 sample; 2605.00332, 2605.01276).
- `tex=` reversions carry the true values.

**Open:**
- **`\raisebox`'s raise** (`\raisebox{-.5\height}{icon}`, 13 papers / 105 uses; 2605.18894, 2605.03941) is still read before the box. Its `yoffset` renders as `position:relative; bottom:` (LaTeXML-common.xsl:610-611), which reserves no space, so the true raise made icons overflow into the next table row. It waits for a render change. RED repro `boxes-groups/raisebox_raise_measures_the_box.tex`.
- **`\resizebox`:** `\resizebox{\width}{!}` (still `xscale="0"`) and `\resizebox*`, through `GraphixDimension`/`\Gscale@@box` (4 papers).
- **Picture and minipage heights:** `\pic@raisebox` (sect13.rs) and `{minipage}`'s `[height]` (latex.ltx's `\endminipage` → `\@iiiparbox`) still read the stubs.
- **Empty `[]`:** `\raisebox{d}[]{x}` now reads its empty `[height]` as 0pt with "Missing number" (56kj: pdflatex's size and the warning), where latex.ltx's `\@irsbox` (`\ifx\\#2\\`) treats it as not given. There are 0 uses in the sample. RED repro `boxes-groups/raisebox_empty_height_is_not_given.tex`.

**Perl** (KNOWN_PERL_ERRORS #298): `\width` etc. are `0pt` text, so `2\width` is 20pt, and the arguments are read before the box.

**Guard**: `perfect_kernel_batch56::box_size_arguments_measure_the_box`. Repros `boxes-groups/box_size_arguments_measure_the_box.tex`, `boxes-groups/raisebox_optionals_measure_the_box.tex`.

### 329. `\rotatebox[…]` turns about graphicx's point: the centre, then the keys in order (Perl: the reference point, `c` a letter)

**Rust** (batch 56km): `RotationOptions` (graphics_sty.rs) follows graphicx's `\Grot@box@kv` (graphicx.sty:226-242).
- **With `[…]`, even empty:** the point starts at the box's centre, `\width/2` and `(\height-\depth)/2` (truncating `\divide`). The `Grot` keys then apply in the order given:
  - `origin`'s letters set one axis each (`l`/`r` set x, `t`/`b`/`B` set y); the last letter on an axis wins, and `c` changes nothing.
  - `x=`/`y=` set a length.
- **Without `[…]`:** the reference point (`\Grot@box@std`), as for rotating.sty's environments, rotate.sty and makecell.

The bounding box is Perl's `rotatedProperties` corner formula (graphics.sty.ltxml:172-188), which is graphics.sty's `\Grot@box` (459-510).

**Before:** the keys were read and ignored.
- `origin=c` at 90° gave 12.8pt high and 0pt deep, where pdflatex gives 8.83/3.94pt. It was harmless while rotated boxes measured as text; 56kj's true sizes exposed it.
- Witnesses: 2605.02583's `valign=m` label raise; 2605.14476 and 2605.22405 node heights.

**Now:** pdflatex's sizes to the sp for `origin=c`/`r`/`l`/`b`/`B`/`rb`, `[]`, `[x=2pt]`, `[x=2pt,origin=c]` and `[origin=l,y=1.2em]`. At 45° we are within 0.0004pt: pdflatex's trig.sty sine is 5 digits (0.70709), while ours is exact.

**Perl** (KNOWN_PERL_ERRORS #299) starts at the reference point and reads `c` as the centre. So `origin=r`, `[]` and `[x=2pt]` turn about another point than pdflatex's. The sample has 20 uses in 3 papers: 2605.23694, 2605.30813, 2605.25220.

**Guard**: `perfect_kernel_batch56::rotatebox_turns_about_its_origin`. Repro `graphics-tikz/rotatebox_origin_keys.tex`.

### 330. A dimen register holds a dimension and a skip register a glue, whatever is assigned (Perl: the value as given)

**Rust** (batch 56kn): `Register::set_value` (register.rs) converts before any setter runs, so custom setters such as `\wd`/`\ht`/`\dp` and `\pagegoal` get the converted value too.
- A dimen register assigned a glue takes its width, as TeX does ("When a glue_val changes to a dimen_val, we use the width component", tex.web §429).
- A skip register assigned a dimension takes it as a glue with no stretch or shrink.

The case that matters is calc's `\setlength` (`\calc@assign@skip`, calc.sty:86, 51-53), which assigns every length as a skip:
- `\setlength{\unitlength}{2pt}` under calc left a glue in `\unitlength`, which the picture code read as 1pt. That affected 201 pictures in 6 of 14 probed arXiv papers: 2605.11190, 2605.05506, 2605.18952, 2605.10502, 2605.02197, 2605.15276.
- `\the\unitlength` after `{2pt plus 1pt minus 1fil}` is `2.0pt`, as in pdflatex.

In the same batch:
- The picture code reads `\unitlength` to the sp: one `base_utilities::unitlength_sp`, where ten sites had rounded it to 0.01pt.
- A picture's size is Perl's typed `picScale` Dimensions with depth 0.
- The picture `\makebox` sizes render at 0.1pt, as Perl's.

**Perl** (KNOWN_PERL_ERRORS #300) stores the glue as given: `[2.0pt plus 1.0pt minus 1.0fil]`.

**Guard**: `perfect_kernel_batch56::unitlength_set_through_calc_is_the_length`.

### 331. A non-vertical list's height and depth are floored at 0 (Perl: a one-item list reports its item's size)

**Rust** (batch 56ko): `List::compute_size` (list.rs) floors the height and depth of a list whose mode is not vertical at 0, as TeX's `hpack` does (tex.web §649, §720 for math lists). A list with no mode counts as horizontal, as it does in `compute_boxes_size`.

Lists of several items were already floored. A one-item list went through `compute_boxes_size`'s bare-box shortcut, which is Perl's `getSize` of a lone Box (Font.pm:648-649), and returned its item's size. The affected results, now pdflatex's:
- `\hbox{\rotatebox[origin=l,y=1.2em]{90}{Qg}}` was −11.99998pt deep;
- `\hbox{\box2}` with `\dp2=-5pt` was −5pt deep;
- `\raisebox{0pt}[5pt][-3pt]` had depth −3pt;
- the `\fbox`, `\resizebox` and `\scalebox` of such boxes.

A vbox keeps its last box's negative depth (§670).

**Perl** (KNOWN_PERL_ERRORS #301): `List()` returns the lone box itself (List.pm:41-44), so Perl measures it unfloored as well.

**Guard**: `perfect_kernel_batch56::a_one_item_hbox_has_no_negative_depth`, with the vbox as negative control. Repro `boxes-groups/hbox_single_child_negative_depth.tex`.

### 332. `\eminnershape` is latex.ltx's `\upshape` (Perl: empty)

latex.ltx's `\em` switches to `\eminnershape` when the current font is slanted (`\ifdim\fontdimen\@ne\font>\z@`, latex.ltx:14048-14052), and `\eminnershape` is `\upshape` (:14058): emphasis inside italic or slanted text is upright.

Perl defines `\eminnershape` empty (latex_constructs.pool.ltxml:5861), leaving the styling "to the ultimate output". Its `\font` reads the default font's parameters, whose slant is 0, so its `\em` always takes `\itshape`. Emphasis inside italic text then shows no change, and inside slanted text it becomes italic.

Since batch 56kr a font's parameters are its TFM's, so the slant of an italic or slanted font is non-zero and `\em` takes `\eminnershape`. Left empty, it would erase emphasis in both contexts. Rust restores latex.ltx's `\upshape`, which is pdflatex's output.

Trigger: `{\itshape a {\em b}} {\slshape c {\em d}}`. pdflatex sets b and d upright. Perl gives b italic (unchanged) and d italic. Rust gives b and d upright.

**Guard**: `perfect_kernel_batch56::a_font_without_its_own_metric_has_cmr_parameters`. Repro `fonts-nfss/fontdimen_font_without_its_own_metric.tex`.

### 333. The dependency scan reads a CR-only file by its lines (Perl: as one line)

`maybe_require_dependencies` / `require_dependencies_except` (content.rs) mirror Perl's `maybeRequireDependencies` (Package.pm:2776-2813). Its comment stripper `%[^\n]*\n` needs a `\n`. A class written with CR-only line ends, such as 2402.17342's sn-jnl.cls (1791 CRs, no LF), is one line to it. Perl's identical regex then strips no comment at all and loads every commented-out `\usepackage` (in that copy, `%%\RequirePackage[T1]{fontenc}` and `hypcap`). A file with a stray LF is stripped from its first `%` to that LF instead.

**Rust** (batch 56kt): before stripping, the scan turns `\r\n` and a lone `\r` into `\n`; TeX ends a line at either (web2c). A `\r\n` file scans as before, since the `\r` sat inside the stripped comment and counts as `\s` in the other patterns. Raw-loaded `.sty` files are unaffected: the executed-set gate already drops a package that never ran. The miss-handler and the unbound-class paths see the change.

**Guard**: `perfect_kernel_batch56::the_sn_jnl_class_loads_its_packages` (the sn-jnl binding's scan); witnesses 2402.17342 (CR-only copy) and 2606.00121 (the binding had loaded a hand-picked subset, so booktabs's `\toprule` was undefined; LEDGER 56kt).

### 334. A class's `\LoadClass` options are its own, and amsmath discards a class's `fleqn` (Perl: global options)

In LaTeX, `\documentclass` and `\LoadClass` both go through `\@fileswithoptions` (latex.ltx:18617-18635), but `\@classoptionslist` and `\@raw@classoptionslist` are set only while `\@classoptionslist` is `\relax` (latex.ltx:18716-18718): at the first class load, `\documentclass`. A class that class loads keeps `opts` to itself. amsmath's `\newif\if@fleqn` (amsmath.sty:64) starts false, and its display environments read only that switch. A class's own `fleqn` (article's `\input{fleqn.clo}`) therefore no longer applies once amsmath loads without `fleqn`.

**Perl** (KNOWN_PERL_ERRORS #304): `InputDefinitions` pushes every cls load's options onto `class_options` and redefines `\@classoptionslist` to them (Package.pm:2578-2581). webofc (2605.12407) and USG (2605.00042) do `\LoadClass[fleqn]{article}` and load amsmath without `fleqn`. amsmath's `ProcessOptions` takes the `fleqn` as a global option, and Perl writes `ltx_fleqn` where pdflatex centres. `\documentclass[french]{…}` over such a class reads `\@classoptionslist` as `fleqn`. With raw classes, the `\LoadClass` runs the article binding's `fleqn` handler and nothing takes it back.

**Rust** (batch 56kv):
- `load_class` (content.rs) counts class loads in progress. A class loaded while another loads is nested (`InputDefinitionOptions::nested_class`): a raw `\LoadClass`, a binding's base class, or the dependency scan's harvest. Its options stay its own (`\opt@<name>.cls`). They are not pushed onto `class_options`, and neither `\@classoptionslist` nor `\@raw@classoptionslist` changes.
- The amsmath binding removes `ltx_fleqn` as it defines `\if@fleqn`, before `ProcessOptions`. A global `\documentclass[fleqn]` or amsmath's own `fleqn` sets it again.
- The reset discards every earlier source of `ltx_fleqn`, not only a `\LoadClass`: `\usepackage{fleqn}` before amsmath, the elsarticle binding's unconditional fleqn (KPE #305; 15 A/B papers), crckapb. pdflatex centres amsmath's displays in each case (`5p` elsarticle + amsmath: x=141pt, without amsmath 63pt).
- Residual: amsmath leaves `eqnarray` alone, so after a class's `fleqn` pdflatex keeps it flush left while it centres `equation`, `\[` and `align`. `ltx_fleqn` is document-wide, so Rust now centres `eqnarray` there too. The XSLT reads `ltx_fleqn` from any ancestor (LaTeX-block-xhtml.xsl:614), so a per-`eqnarray` class could restore it.
- Without amsmath, a class's `\LoadClass[fleqn]{article}` still gives `ltx_fleqn` under raw classes, as pdflatex sets it flush left. Without raw classes, OmniBus has already loaded article, so the re-load does nothing, as in Perl.

**Guard**: `perfect_kernel_batch56::loadclass_options_are_the_class_own` (no preload and raw classes; with and without amsmath; `\@classoptionslist` keeps `french`). Repro `loader/loadclass_options_stay_local.tex`.

### 335. elsarticle's equations are flush left only in the `5p` and two-column `3p` layouts (Perl: always)

elsarticle.cls inputs `fleqn.clo` only for `\jtype` 5, or 3 with `twocolumn` (elsarticle.cls:1280, 1295). The default `preprint` (jtype 0), `1p` and one-column `3p` centre their equations; pdflatex sets `[3p,twocolumn]` flush left (x=84pt).

**Perl** (KNOWN_PERL_ERRORS #305): elsarticle.cls.ltxml:45 runs `RequirePackage('fleqn')` for every layout, and treats `1p`/`3p`/`5p` as ignorable.

**Rust** (batch 56kw): the `5p`/`3p`/`1p` options record the journal type in the class's declared order (elsarticle.cls:82-87), so the last declared wins (`[3p,5p]` is `3p`, centred in pdflatex). After article loads, fleqn loads only for `5p`, or `3p` with article's `\if@twocolumn`. A document that loads amsmath is centred in any layout (#334).

**Guard**: `perfect_kernel_batch56::elsarticle_fleqn_follows_the_journal_type` (default, `1p`, `3p`, `3p,twocolumn`, `5p`, `3p,5p`). Repro `loader/elsarticle_fleqn_follows_journal_type.tex`.

### 336. amsmath's tags follow only its own `leqno`/`reqno`, in its declared order (Perl: a class's `leqno` stays; `reqno` first)

amsmath.sty:53 `\newif\iftagsleft@` starts false, and amsmath's displays read only it. `leqno` and `reqno` set it, declared in that order (amsmath.sty:54-55), so `\ProcessOptions` lets a local `reqno` beat a global `leqno`. A class's own `leqno` (article's `leqno.clo` through a raw `\LoadClass[leqno]{article}`) no longer sets the tags. amsart passes its default `leqno` and any `reqno` on to amsmath (amsart.cls:159-162). acmart loads amsart with `reqno` (acmart.cls:282).

**Perl** (KNOWN_PERL_ERRORS #306): the amsmath binding declares `reqno` before `leqno` and never resets `ltx_leqno`. ams_core.cls.ltxml sets `ltx_leqno` by default without passing `leqno` to amsmath, and acmart.cls.ltxml loads amsart without `reqno`. So ACM papers number their equations on the left. `\documentclass[leqno]{article}\usepackage[reqno]{amsmath}` also numbers on the left.

**Rust** (batch 56kw):
- The amsmath binding declares `leqno` before `reqno`, and removes `ltx_leqno` as it loads, before `ProcessOptions`.
- `ams_core_cls.rs` and `amsbook_cls.rs` pass `leqno` (the default and the option) and `reqno` to amsmath with `\PassOptionsToPackage`.
- `acmart_cls.rs` passes `reqno` to amsart (2605.02222, 2605.24417).
- `ams_core_cls.rs`/`amsbook_cls.rs` run the default through `execute_options(&["leqno"])`, as amsart.cls:350 runs `\ExecuteOptions`.
- The Rust-only stubs for classes that pass `leqno` to amsmath now pass it too: siamart (siamart250211.cls:64 `\RequirePackage[leqno]{amsmath}`; 2605.02838, pdflatex `(0.1)` on the left), aomart (aomart.cls:75 `\LoadClass[11pt]{amsart}`), and imsart's `aap`/`aop`/`aos`/`aoas`/`sts`/`aihp` journals (imsart.cls:84-118; 2605.16086).

pdflatex tag sides, all now matched:
- `\LoadClass[leqno]{article}` + amsmath: right.
- `[leqno]{article}` + `[reqno]{amsmath}`: right.
- amsart: left.
- `[reqno]{amsart}`: right.
- `[leqno]{article}` + amsmath: left.
- acmart: right.
- amsbook: left; `[reqno]{amsbook}`: right.
- siamart, aomart, `[aos]{imsart}`: left; plain imsart: right.

**Guard**: `perfect_kernel_batch56::amsmath_tags_follow_its_own_options`. Repro `loader/amsmath_tagsleft_discards_class_leqno.tex`. The `complex/acm_aria.xml` golden drops the Perl golden's `class="ltx_leqno"` (the test has no equations).

### 337. A cropped raster's pixels are its own resolution over 72 (Perl: its resolution, unit ignored, over 72.27)

graphicx's `trim=` and `viewport=` lengths are bp. pdfTeX sizes a raster by its own resolution, a PNG's `pHYs` (per metre) or a JPEG's JFIF density (per inch or per centimetre), or 72 dpi when it states none.

**Perl** (KNOWN_PERL_ERRORS #307): `image_graphicx_complex` crops with `$idppt = (x-resolution // $dpi)/72.27` (Util/Image.pm:402-403). That divides bp by a pt's 72.27, and takes ImageMagick's `x-resolution` without its unit. ImageMagick reports a PNG's in pixels per centimetre, so a 300-dpi PNG (118.11 px/cm) is under-cropped 2.54 times over.

**Rust** (batch 56kx):
- `latexml_core::util::image::graphicx_crop_rect` ports the crop arithmetic (Util/Image.pm:404-418), with pixels per bp the raster's dpi over 72 or, for a rendered PDF/EPS, the render density over 72.
- `raster_resolution_dpi` reads the dpi as pdfTeX takes it: a PNG's per-metre value rounded, a per-centimetre JFIF value truncated (pdflatex sizes an 11811 px/m PNG at 300 dpi and a 118 px/cm JPEG at 299), 72 when none is stated.
- A `viewport` reaching past the image's left or top edge keeps only its overlap with the image; Perl's crop width `min($ww, $w-$x0p)` (L417) runs that far past the box's right or bottom edge.
- The displayed size applies every op but the crop to what the file holds: a crop that keeps the whole image, or a negative trim, sizes nothing (Perl shows the processed image at its own size; 2605.06510's colorbars, `trim=0cm -4cm 0cm 0cm`, were stretched 24 %).
- A natural-size vector figure's `em` box (graphicx_sty.rs) is its natural size under the same crop and turn, clamped, so a cropped figure is not stretched back into the page's box.
- A length in pt reaches the crop as bp through 72/72.27, so the rounding snaps within a millionth (`100.375pt` is 100.00000000000001bp and would otherwise gain a pixel row).

What stays Perl's:
- the crop with or without `clip` ("we'll just clip in all cases", L180-185; pdflatex paints the uncropped image over the box without `clip`);
- the crop before the turn;
- a new resource file for a transformed raster (Post/Graphics.pm:318-323);
- the `limitation` warning, trim/clip/angle dropped, for a non-raster (SVG) source.

A small PDF's vector render (beyond Perl, which only rasterizes) is cropped through its root `viewBox`, which spans the page in bp; `width`/`height` shrink with it (`crop_svg_inplace`). `reflect` is still not applied.

**Guard**: `latexml_post graphics::tests::{trim_and_viewport_crop_the_raster_into_its_own_file, a_crop_gets_its_own_file_and_a_zero_trim_none, crop_svg_narrows_the_view_box}`, `perfect_kernel_batch56::a_natural_figure_box_follows_its_crop_and_turn` and `latexml_core util::image::…::{graphicx_crop_rect_follows_perl_trim_and_viewport, raster_resolution_reads_png_and_jfif_units}`. Witness 2510.17772 (S3.F4: `trim=90 30 50 50` on a 3000×1500 300-dpi PNG keeps 2417×1167 pixels). Repro `graphics-tikz/includegraphics_trim_crops_the_raster.tex`.

### 338. A `\parbox` or minipage between paragraphs starts one (Perl: a block of its own)

latex.ltx's `\parbox` ends in `\@iiiparbox`, which begins with `\leavevmode` (latex.ltx:16249-16250): a parbox met in vertical mode starts a paragraph, the box is its first material, and the text after it continues that paragraph.

**Perl** (KNOWN_PERL_ERRORS #309): `\lx@parbox` (latex_constructs.pool.ltxml:4750-4764) begins `inline_internal_vertical` mode without `enterHorizontal`, and `insertBlock` at section level makes the box a `<para class="ltx_parbox">` of its own; the space after it is read in vertical mode and dropped, and the text opens a new paragraph.

**Rust** (batch 57d): `\lx@parbox` declares `enter_horizontal`, and a constructor declaring it records `in_paragraph` on its whatsit when that `\leavevmode` leaves TeX in a paragraph: invoked in vertical mode (a new one, tex.web §1091 `new_graf`) or in unrestricted horizontal mode (`\noindent\parbox…`); in a restricted box (`\rotatebox{90}{\parbox…}`, arXiv 2605.20645) or math it is a no-op (constructor.rs `digest_to_whatsit`). The parbox is built by `insert_block_in_paragraph` (base_utilities.rs), which after the capture and the sectioning float-out (#250) opens the `ltx:p` where text would open one (`text_opens_paragraph`, the model's auto-open route for `#PCDATA`): the box goes into it as an `<inline-block class="ltx_parbox">` and ` text after.` continues it. Not in a float, whose content is panels for `arrange_panels`, seen through the captures of any box or environment built in it (`in_panel_float`, `PANEL_FLOATS`: arXiv 2605.27134's parbox-topped minipage panels stay side by side, captioned parbox panels in a `{center}` stay sub-figures); not for an empty box; not for content no inline box holds (a bibliography, an index, a caption), which the hoist places after the box as before. Found by the K13 binding-conformance audit.

**`{minipage}`** (batch 57f): `\@iiiminipage` begins with `\leavevmode` too (latex.ltx:16305-16306); the environment declares `enter_horizontal` and is built by `insert_block_in_paragraph`, so a minipage between paragraphs is an inline block in the paragraph it starts, the text after it continues that paragraph, and minipages on one line sit side by side (`\hfill` pairs, a list's minipage of `\item`s beside a second one, 2605.19762's three `\subsubsection*` minipages in a `table*`) instead of stacking. **Lone float** (user ruling 2026-09-27, option C, shared by both boxes): a box whose whole content is one float becomes that float by Perl's fold when its line holds nothing else — `\captionof` in a minipage or a parbox gives `<figure class="ltx_minipage">` / `<figure class="ltx_parbox">` (for the parbox this restores Perl's shape over 57d's inline one); since 57g this is settled once the document is built (#339), and a float-box followed by text on its line stays inline. A box that is clearly a figure but holds more than the float (image + `\captionof`) becomes the float with its image, and lone-float boxes on one line form a row of panels: #339 (57g). Guards `perfect_kernel_batch56::{a_minipage_starts_the_paragraph, a_box_holding_only_a_float_becomes_it, para_class_minipage_in_figure_demotes_keeping_its_width}`, `cluster_schema::quote_holds_framed_logical_and_sectional_blocks` (the #271 admissions through `{framed}`); repros `boxes-groups/minipage_starts_the_paragraph` (RED 57d → GREEN), `captions-floats/box_holding_only_a_float_becomes_it`; goldens `alignment/vmode`, `digestion/dollar`; guards reshaped: list auto-item (the #261 branch now pinned by a `\vbox` case), titlepage demotion (a `{center}` body, whole element), quote (#271), lstnewenvironment, calc/pdfcomment minipages, nested/restricted parbox.

**Witnesses**: goldens `alignment/vmode` and `digestion/dollar` (the parbox moves into the section's paragraph). arXiv A/B (57c → 57d, 3,003 papers): tallies identical (errors 5,085, fatals 6, words 44,327,175); 28 papers change, each a parbox joining its paragraph with the same text (2605.15128 also keeps the spaces after its boxes; 2605.02364 only splits a whitespace node); witnesses 2605.27134 and 2605.20645 byte-identical to 57c. Manual net (1,000 manuals): 0 regressions, validity and words unchanged; 74 of the 75 changed manuals are parbox moves (hitreport-example, jourcl: a parbox holding one `tabular`, which now carries the box's attributes in the paragraph), memman's and swfigure's captioned parbox sub-figures intact.

**Guard**: `perfect_kernel_batch56::{a_parbox_starts_the_paragraph, a_parbox_in_a_restricted_box_starts_no_paragraph, parbox_panels_stay_panels, a_parbox_holding_a_section_floats_it_out, a_parbox_holding_a_bibliography_opens_no_paragraph}`; repros `tools/perfect_kernel/repros/boxes-groups/{parbox_starts_the_paragraph, parbox_in_a_restricted_box_starts_no_paragraph, parbox_holding_a_section_floats_it_out, parbox_holding_a_bibliography_opens_no_paragraph}.tex`, `captions-floats/parbox_panels_stay_panels.tex`. Settled dead-end: opening the `ltx:p` before `insert_block` (the first cut) changes every decision it makes from its context — a section in the box stayed in the `<p>` (#250 skipped), the float panels inside `{center}`/`{minipage}` captures became `<p>` panels (stacked, captions hoisted out of their sub-figures); opened before the hoist, it left an empty `<p/>` behind a bibliography the hoist moved out, or a `<block>` inside the `<p>`, and captions hoisted out of parbox sub-figures in a box inside a figure (memman, swfigure-examples).

### 339. A box that is clearly a figure becomes a `<figure>` holding its image; figure-boxes on one line form a row (Perl: a caption-only float beside the image, stacked)

`\captionof{figure}{…}` (caption.sty) sets the caption type; LaTeXML's binding (Perl caption.sty.ltxml, ours `\@captionof@`, caption_sty.rs) wraps only the caption in the float element, so a minipage holding an image and `\captionof` becomes a box with the image in a `<para>` beside a caption-only `<figure>` (KNOWN_PERL_ERRORS #312), and such boxes side by side on one line (`\hfill`) stack.

**Rust** (batch 57g, user rulings 2026-09-27): in `insert_block_as` (base_utilities.rs), a box whose content is one float holding only its caption (the shape `\captionof` makes) and figure material — graphics, pictures, tabulars, rules and their `para`/`p`/`\scalebox`/font wrappers, no prose — hands the material to the float in source order (`absorb_figure_material`; a float holding content of its own, `\begin{figure}[H]…`, takes nothing, so its order with the box's material stands). It applies to every box and alignment capture that reaches `insert_block` (`{center}` + `\captionof` + a tikzpicture: the tikz/consort-flowchart golden now holds its flowchart in the figure). Such a float-box is paragraph material like any box (57d/57f): it sits in the paragraph it starts as an inline block. Once the document is built (`settle_float_paragraphs`, latex_constructs, first step of `finish_document` and, per spilled segment, of `streaming_pass2`, core_interface.rs — eager and `--streaming` output are identical), a paragraph holding nothing but float-boxes — besides `\\` breaks and whitespace — gives way to them (`settle_float_paragraph`): one becomes its float with the box's class and sizes folded on (`<figure class="ltx_minipage"><graphics/><caption/></figure>`, Perl's fold), several become a row, an uncaptioned `<figure>` holding them as panels with the author's breaks, laid out by `arrange_panels` (the row's `after_close`) — the markup LaTeXML gives a figure float holding captioned minipages. A paragraph holding anything else (text, a box that is not a float) keeps its boxes inline, side by side (arXiv 2605.18920's table and algorithm boxes). Paragraphs of successive lines (a blank line between) settle separately; a `{center}` grid split by `\\` is several paragraphs of the capture and stays as 57f made it. The line's alignment (`align`, the `\raggedleft` classes) goes onto the float or row. A row takes over the settled paragraph's identity — its `xml:id` and id counters, minted during construction (`Document::wrap_nodes_as`) — so it is numbered alike in a streaming segment and the ids of its contents keep their prefix; a lone float keeps its own id, and its paragraph's number stays unused (settling after construction cannot give it back): later paragraph ids there differ from Perl's and 57f's (the `structure/autoref` golden; 4 of the A/B's changed papers: 2605.16962, 2605.17828, 2605.24961, 2605.29757).

**Settled dead-ends**: deciding at insertion (lifting a box that became a float out of the paragraph it had started, and grouping boxes by a paragraph serial numbered at `new_graf`) stacked a float-box beside a text box (2605.18920), dropped `\\` between figure-boxes, and appended panels past `arrange_panels`. Settling at the paragraph's close (its `after_close`) fired inside `{center}`'s own construction, which still held the paragraph, and dropped the center alignment (2605.03276 A2.F8-F10). Minting the row's own id when it settles numbered it under a streaming segment's parse wrapper (`fig13` for a section's `S1.fig2`), so eager and streamed output differed. The paragraph is the unit that knows what the line holds, and the finished document the time to ask it.

**Witnesses**: arXiv 2605.27134 S5.F8 (two parbox-in-minipage panels, each now a sub-figure holding its image and caption), 2605.06890 (`\captionof{table}` minipage panels, now `<table>` panels with the caption above the tabular, as written). A/B 57f → 57g (3,003 papers): tallies identical; 165 papers change, each with the same words and formulas (the `tex=` order moved in 12 where a `\captionof{table}` above its tabular now precedes it, as written); element deltas `logical-block` −154, `para` −264, `figure` −61 / `table` +71 (panels that carried a table caption are now tables). Manual net: 0 regressions; latex4wp's code-listing boxes beside their output boxes now sit side by side in one paragraph.

**Guard**: `perfect_kernel_batch56::{figure_boxes_become_figures_and_rows, figure_box_lines_settle_when_the_paragraph_closes, float_boxes_keep_their_line_alignment, a_noindent_lone_float_box_stands_beside_the_paragraph, logical_block_climbs_out_of_para}` (the last: #240's climb, now with text after the float), `116_adaptive_streaming::figure_box_rows_settle_alike_when_streamed` (eager and forced-spill streaming identical), `perfect_kernel_batch53::hvfloat_only_text_env_options_and_ifoddpage`, `latex_via_exemplos_residue::caption_settype_declares_the_float_type` (a float-box followed by text stays inline); repros `captions-floats/{figure_boxes_become_figures_and_rows, figure_box_lines_settle_when_the_paragraph_closes, float_boxes_keep_their_line_alignment, figure_box_rows_settle_alike_when_streamed}.tex`, `captions-floats/noindent_lone_float_box_leaves_empty_para.tex` (RED 57f → GREEN); goldens `tikz/consort-flowchart`, `structure/autoref` (a lone float's paragraph number unused).

### 340. `\orcidlink` starts the paragraph it begins (Perl: the link is built in vertical mode)

The real `\orcidlinkX` is `\href{…}{…}` (orcidlink.sty:69); hyperref's `\hyper@linkurl` begins with `\leavevmode` (hpdftex.def:411). LaTeXML's binding wraps the link in the mode-less constructor `\lx@orcidlink` (KNOWN_PERL_ERRORS #313), so at a paragraph's start the space after the link was dropped.

**Rust** (batch 57i, a K13 stage-2 finding): orcidlink.sty's `\orcidlinkX` begins with `\leavevmode`, the prologue of the `\href` it stands for. The shared `\lx@orcidlink` is unchanged: the frontmatter's `\lx@add@orcid` builds the same link inside author annotations, where no paragraph must open.

**Witnesses**: none in the corpus; found by the K13 audit. No paper of the 3,003-paper A/B changes (95 load orcidlink); links in author blocks, `\thanks`, captions, `l` cells and math convert byte-identically. Re-converted 2605.21922 (the `p{…}` cell witness above): byte-identical, 0 errors.

**Guard**: `perfect_kernel_batch56::orcidlink_starts_the_paragraph`; repro `boxes-groups/orcidlink_starts_the_paragraph.tex` (RED 57h → GREEN).

### 341. changepage's `{adjustwidth}` keeps its body in the flow (Perl with the raw package: a one-item `<itemize>`)

changepage.sty's `{adjustwidth}` is a `\list` (changepage.sty:110-139); Perl has no binding, and `--includestyles` builds it from the raw file as a tagless one-item `<itemize>`, in which a bibliography is malformed (`<ltx:bibliography> isn't allowed in <ltx:item>`, the MDPI template's shape). The margins are layout; the body is the document's own text.

**Rust** (batch 57l): the binding loads the package's macros raw and defines `{adjustwidth}`/`{adjustwidth*}` (and chngpage's `{adjustwidth}[3]`) as a transparent body in internal vertical mode whose last paragraph closes before the end, as the list's `\endtrivlist` does (latex.ltx:15915-15926). Under memoir, which provides changepage's commands (memoir.cls:12216) and whose `{adjustwidth}` is the same list (memoir.cls:11267-11297), the binding loads nothing and its environment replaces memoir's; memoir's `\checkoddpage` stays. `\ch@ngetext`, which `\changetext`/`\changepage` end with (changepage.sty:80-88, chngpage.sty:79-88, memoir.cls:11237-11245), halves `\columnwidth` under `\if@twocolumn`; one shared override (`changepage_sty.rs` `ONE_COLUMN_CH_NGETEXT`, installed by the changepage, chngpage and memoir bindings) keeps LaTeXML's one column, so `\linewidth` follows the changed `\textwidth`.

**Witnesses**: 2605.02723 (a `width=\linewidth` figure, 0pt before), 2305.09826 (`$$` inside), 2006.09676 (`{adjustwidth*}`).

**Guard**: `perfect_kernel_batch56::{adjustwidth_ends_its_paragraph, adjustwidth_margins_are_read_not_typeset, changepage_page_checks_are_the_packages, changepage_under_memoir_keeps_memoirs, chngpage_is_its_own_package, changetext_keeps_one_column, changetext_under_memoir_keeps_one_column}`.

### 342. apacite's reference-list layer runs verbatim, less three layout-only details

apacite.sty's formatting layer, strings and `\PrintOrdinal` run as the package's own TeX (apacite_sty.rs, batch 57m; Perl has no binding). Three details differ from the raw package, each without reference-list text:

- the `\hbox{}` after each abbreviation (`Vol.\hbox{}`, `pp.\hbox{}`, `et al.\hbox{}`, apacite.sty:2068-2156) is dropped: it only resets the space factor (texbook.tex:4464-4466), which the XML does not carry, and it would add an empty `<ltx:text xml:id>` under `ids`;
- `\Bem` is `\emph` before a brace (apacite.sty:1519) and the declaration `\em` otherwise, so a pre-2012 `.bbl`'s `{\Bem Journal}` (theapa.sty:69 `\let\Bem\itshape`) italicizes its group, where today's apacite would italicize one letter;
- `\nocitemeta` sets its flag at once instead of writing it to the `.aux` for the next run (:1137-1149): the asterisk prints when the call precedes the bibliography.

**Guard**: `perfect_kernel_batch56::{apacite_bem_reads_both_forms, apacite_old_bbl_keeps_its_declared_italics, apacite_bbl_formats_like_pdflatex}`.

### 343. A folded minipage or `\parbox` is a figure panel at its declared width

`insert_block` folds a minipage or `\parbox` that holds one node into that node, carrying the box's class and `width` (Perl TeX_Box.pool.ltxml:489-492). Perl's panel layout then treats a folded `<p>` as a standalone paragraph with a row of its own and sizes panels by their box (latex_constructs.pool.ltxml:3225-3227, 3277, 3286, 3335) — the natural width of the text, or of a rotated label.

**Rust** (batch 57o): a node carrying the `ltx_minipage` or `ltx_parbox` class and a `width` is a box TeX sets at that width (a `\vbox` whose `\hsize` is the declared width, latex.ltx:16249-16253, 16305-16312): `arrange_panels` sizes it by that width and does not give it a row of its own (`box_panel_width`, latex_constructs/mod.rs). Two `0.45\textwidth` text minipages share a row, as in the PDF (KPE #310). Residual: a source `\par` between rows is not seen, so a narrow label panel can join the previous row (RED `captions-floats/par_breaks_a_panel_row`). Such a box is never the container of a panel merge (Perl's reuse of an `ltx:block`): a label minipage eight times narrower than the picture minipage beside it is wrapped with it, not put inside it (57q, KPE #331; witness 2605.00042).

**Witnesses**: 2605.27134 S5.F8, 2605.17146 S4.F4, 2605.13435 S3.F2, 2605.05889 S4.T4.

**Guard**: `perfect_kernel_batch56::{minipage_text_panels_share_a_row, minipage_panel_single_child_width, panel_merge_keeps_boxes_closed}`.

### 344. An alignment switch inside a heading aligns the heading

`\centering`, `\raggedright` and `\raggedleft` set `\leftskip`/`\rightskip`/`\parfillskip` for the paragraph they end (latex.ltx:15409-15423), and a heading is one paragraph. Perl applies the switch to the element children that follow it (latex_constructs.pool.ltxml:1225-1235): inside a title that is an `align` on an inline `<text>`, which centres nothing and keeps the format's font from collapsing onto `<title>` — a titlesec `{\huge\bfseries\filcenter}` chapter renders at ≈517% (the size on the span multiplied by the title's own).

**Rust** (batch 57o): when the aligning node is an `ltx:title`, `ltx:toctitle` or `ltx:subtitle`, the switch aligns the heading itself (`ltx_align_center` — text alignment only, so a run-in heading stays inline — `ltx_align_left`, `ltx_align_right`; `apply_aligning_context`, latex_constructs/mod.rs). Kernel-wide: sectsty's `\allsectionsfont{\centering}`, `\section*{\centering Acknowledgments}` and `\refname{\centering …}` centre their heading, as in the PDF. Limitation: a switch in an inner group of the title (`\section{Plain {\centering inner} tail}`, which TeX's paragraph never sees) also aligns the heading.

**Witnesses**: 2605.19575 (`\large\bfseries\center`), 2605.22788, 2605.19519.

**Guard**: `perfect_kernel_batch56::{centred_title_format_keeps_its_size, titlesec_format_runs_once, pagebreak_in_a_title_format_precedes_the_section}`.

### 345. A DOI pubnote holding markup is not wrapped in a second link

LaTeXML-structure-xhtml.xsl builds a DOI pubnote's link as `https://doi.org/` + its `text()` (Perl's :508-514 the same). A pubnote whose content is an element — pnas-new's `\doi{\url{www.pnas.org/…}}`, a `<ref>` — gave nested anchors and a dead `https://doi.org/` link.

**Rust** (batch 57p): the template matches `ltx:pubnote[@role='doi'][not(*)]`; a DOI pubnote holding markup is rendered as any pubnote (its `<ref>` the one link).

**Guard**: `perfect_kernel_batch56::pnas_doi_is_the_classs_setter` (the core XML); witnesses 2605.03599, 2605.07504.

### 346. elsarticle's itemize labels as an itemize

Perl's elsarticle binding counts `{itemize}[]` in the enumerate counter (elsarticle.cls.ltxml:64), so every bullet list prints "1."/"(a)" labels (KNOWN_PERL_ERRORS #360).

**Rust** (batch 57y): the itemize counts in `@item` (`elsarticle_cls.rs`), labelled by `\labelitem<depth>` as elsarticle.cls:1143-1150 and the kernel do, its optional argument that level's label (`\@Itemize[#1]`); pdflatex the oracle. Item and list ids are the kernel itemize's.

**Guard**: `perfect_kernel_batch56::elsarticle_itemize_has_bullets`.

### 347. A `\let` copy of `\jobname` taken before the job holds the job's name

Perl installs the job's `\jobname` after the preloads (Core.pm:143 `initializeState`, then :152), as a fresh definition, so a copy a preload takes (`\let\myjob\jobname`) keeps the empty placeholder (TeX_Job.pool.ltxml `DefMacroI('\jobname', undef, Tokens())`). No Perl preload loads the LaTeX format, so expl3's own copies are taken after the install and are right there.

**Rust** (batch 57aa): a `latexml.sty` / `ar5iv.sty` preload loads the LaTeX format before the job, so `install_jobname` (`core_interface.rs`) re-points every binding still holding the placeholder to the job's definition (`state::rebind_let_copies`) — expl3's `\tex_jobname:D` and `\c_sys_jobname_str` (Perl's result for them), and any copy a user preload takes (TeX's result: `\jobname` is a primitive there, known before any code runs). A later install (the nested `.bib` session) re-points nothing.

**Guard**: `perfect_kernel_batch56::tex_jobname_is_the_jobname_under_a_preload`, `state::let_copy_tests::let_copies_follow_a_late_definition`.

### 348. Letters after leading digits join into one identifier

Perl's letters ligature (Base_XMath.pool.ltxml:443-458) reads back from the last letter while the fonts are `equals` and the text is `/^[0-9a-zA-Z]+$/`, and joins only when the run starts with a letter. A digit before `\mathrm` letters has an equal font, so it is read too and the whole run is refused: `10\mathrm{log}_{10}` is `l * o * g`, `2\mathrm{skew}` `s * k * e * w` (KNOWN_PERL_ERRORS #364) — the rendering is the same, the semantics (and a screen reader's reading) are not.

**Rust** (batch 57ae): the fonts compare as Perl's (`Font::perl_equals`), and when the run read back starts with digits, the letters after them join, the digits staying a number: `10`, `log`; `2`, `KL`; `5`, `mm`. A run starting with a letter joins whole, as Perl (`\mathrm{ab}2` is `ab2`, `\mathrm{a2b}` `a2b`). In the 3,003-paper A/B this keeps 71 formulas in 30 papers joined.

**Guard**: golden `tests/parse/math_lexemes.tex#letters_ligature_reads_back_through_a_digit`.

### 349. A spacing hint after a punctuation mark in a gathered row is padding

Perl's `filter_hints` (MathParser.pm:417-491) makes a hint of 10pt or more (`\quad`) a virtual PUNCT unless the node before it is a PUNCT, reading that node's own `role` (:482-483), and passes a hint after an OPEN on to the next node, again by the node's own `role` (:445). In a gathered/split/multline row's content branch the nodes are XMRefs without a role, so `a=b,\quad c=d` there has two punctuations in a row and `(\quad x)` a punctuation inside its fence; Perl leaves both unparsed, where inline they are `formulae@(a = b, c = d)` and `x` (KNOWN_PERL_ERRORS #365).

**Rust** (batch 57af): the OPEN and PUNCT tests read the realized role (`get_grammatical_role`, an XMRef's target's), so the row reads as it does inline and the `\quad` is the comma's `rpadding` (on the XMRef too, as Perl pads a content XMRef). The rest follows Perl: the phantom gate (:443), APPLYOP taking no space (:458-461), a pending space — negative too — as the next node's `lpadding` (:466). Not ported: keeping an XMHint that an XMRef references (:453-457), carrying XML comments onto the neighbouring node (:427-437), and Perl's precision in summed widths (Rust records each hint's width at 0.1pt, `\!` as `width="-1.7pt"`, so `\!\!` sums to `-3.4pt` where Perl's is `-3.3pt`).

**Guard**: golden `tests/parse/aligned_content_branch.tex#content_branch_reads_its_delimiters`.

### 350. An evaluation bar inside `\|…\|`, `\left|…\right|` or a nested group

Perl's `absExpression : <rulevar: local $forbidEvalAt = 1>` (MathGrammar:410) forbids the `f|_{x=0}`
evaluation bar (`moreFactors`, :259-262) anywhere inside an absolute value or norm, nested groups
included — a guard against reading one of a pair of identical bars as the evaluation bar
(`|\nabla a|_L|\nabla b|_L`).

**Rust** (batch 57am): the guard holds for the single-bar pairs where a bar can be misread (`|…|`,
`||…||`, `\big|…\big|`), and stops at a nested group, which pairs its own bars; `\|…\|` (one
token) and `\left|…\right|` (paired by the lexer) take an evaluation bar inside:
`\|u|_{\Gamma}\|`, `\left\|\left.\frac{\partial y}{\partial Z}\right|_{Z=0}\right\|`,
`\left|\mathcal{M}|_S\right|`, `|g(f|_{x=0})|` parse (2605.01526, 2605.04708, 2605.07463), where
Perl fails or reads `delimited-‖|`. And it is a preference, not a prune: the single-bar fences mark
the reading (`Meta::evaluation_bar_inside`, latexml_math_parser/src/semantics.rs) and the forest
keeps the trees with the fewest marks (`XM::prefer_fewest_evaluation_bars_inside`, semantics/tree.rs,
before the student pragmas), so a pair whose every reading nests the bar keeps it while the other
pairs of the formula read Perl's way — `|f(x)|_{0}^{1}|`, `\bigg|g\big|_{t=1}-h\bigg|\le C` is
|eval(g, t=1) − h| ≤ C, `|\nabla a|_L|\nabla b|_L+|f(x)|_0^1|` keeps |∇a|_L·|∇b|_L (2605.26054,
2605.02499; Perl fails). Only where no reading without the mark exists: before `+ c` the bar pair is
a conditional, as in Perl — `|f(x)|_0^1|+c` is `conditional@(((|f(x)|)_0)^1, + c)` (57al read
|eval(f(x), 0, 1)| + c), and `|f(x)|_0^1|+|\nabla a|_L|\nabla b|_L`
`conditional@(((|f(x)|)_0)^1, + |∇a|_L·|∇b|_L)` (Perl fails) — a residual, the price of Perl's rule. A conditional bar takes no condition with a bar reading of its own (Perl's
`ExpressionsNoBars`, MathGrammar:267-268) nor a relation missing an operand (`vertbar_modifier`).

**Guard**: `perfect_kernel_batch57::evaluated_at_stays_outside_absolute_bars` (repro
`math-parse/evaluated_at_stays_outside_absolute_bars`).

### 351. An OPFUNCTION applied to a group takes the scripts after it

Perl's `addEasyArgs` (MathGrammar:571-576) ends an OPFUNCTION's Factor at the group's close; a
script after it has no base, and the formula is unparsed (`\log(n)^2`, `\operatorname{Var}(X)_k`,
`\max(a,b)^2`, `\exp(x)^{-1}`).

**Rust** (batch 57am): the group application takes the scripts
(`scripted_opfunction_application`, latexml_math_parser/src/grammar/builder.rs): (log@(n))²,
(Var@(X))_k, (max@(a,b))², also after a factor (`\lambda^2\log(|A|)^2` λ²·(log@(|A|))²); a bare
OPFUNCTION before a group always takes it (`addEasyArgs`), never multiplies it. The rule it
replaces (`opfunction tight_term`) read `\log(n)^2` as log@(n²). About 359 formulas in 112 of
the 3,003 A/B papers (2605.24357, 2605.01408). Only an unscripted head: a scripted one reads as a
limit, its operand the scripted group however many scripts (`scripted_group_apply`) —
`\min_w(y-w)^2` is min_w@((y−w)²), `\min_j(y_j)_{j=1}^{n}` min_j@(((y_j)_{j=1})^n) (2605.04340,
2605.23087, 2605.19263) — a base subscript too: `\log_2(n)^2` is (log₂)@(n²), as before 57am (Perl
fails). Not a bar pair, which is no `addEasyArgs` group but an `aBarearg` (:330) that starts the
greedy bare argument: `\log|z|^{2}dz` is log@(|z|²·dz), as Perl (`group_apply`; `||x||` too,
#353); `\lvert…\rvert` is an OPEN/CLOSE pair, so `\log\lvert x\rvert^2` is (log@(|x|))². Not a
TRIGFUNCTION, whose argument is `trig_arg`: `\sin(x)^2` stays sin@(x²). Known cost: a power meant
for the group reads outside it — `\log(2\pi e)^{n}` (the Gaussian entropy's ½log((2πe)ⁿ|Σ|)) is
(log@(2πe))ⁿ, was log@((2πe)ⁿ) (Perl fails both).

**Guard**: golden `tests/parse/opfunction_arguments.tex#opfunction_group_application_takes_its_scripts`.

### 352. A differential anywhere inside an integral's operand

Perl licenses a differential `d` (MathGrammar `IntFactor`'s `diffd`, :643-646) only among the
top-level factors of an integral's operand (`addIntOpArgs`/`moreIntOpArgFactors`, :633-638): in a
fenced integrand, or under a big operator nested in the integral, the `d` is a plain unknown —
`\oint(P\,dx+Q\,dy)` is `contour-integral@(P * d * x + Q * d * y)`, `\int\sum_i f_i\,dx`
`integral@((sum _ i)@(f _ i * d * x))`.

**Rust** (user ruling 2026-09-28: "a clear semantic improvement"): a `d` in a clear integration
context reads as the differential wherever it stands in the integral's operand —
`contour-integral@(P * differential-d@(x) + Q * differential-d@(y))`,
`integral@((sum _ i)@(f _ i * differential-d@(x)))`. Outside an integrand it is to stay a plain `d`,
as Perl (`\int f=1,\qquad dx=2` is `d * x = 2`; Rust still reads `differential-d@(x) = 2` there). The faithful licensing port (repro
`math-parse/differential_needs_its_integrand`, RED) keeps this scope.

**Guard**: golden `tests/parse/integrals_and_differentials.tex#differential_is_no_big_operator` (the kept
differentials); the licensing port adds the fenced and nested-sum rows.

### 353. A norm written `||x||` is a bare item, as `\|x\|` is

Perl's `SINGLEVERTBAR SINGLEVERTBAR absExpression SINGLEVERTBAR SINGLEVERTBAR` norm (MathGrammar:294)
is a Factor, not an `aBarearg` (:329-330, only `VERTBAR absExpression VERTBAR`), so an OPFUNCTION
before it multiplies it: `\log||x||_2^2` is `logarithm * ((norm@(x)) _ 2) ^ 2`,
`\min_x||Ax-b||_2^2` `minimum _ x * …`, while `\log\|x\|^2` is `logarithm@((norm@(x)) ^ 2)` — the
same norm, split by the spelling of its bars.

**Rust** (batch 57am): every fence built from VERTBAR tokens — `|…|`, `\|…\|`, `\left|…\right|`,
the stretchy norms (kerned or not, `\log\left|\left|x\right|\right|_2^2` is `logarithm@(((norm@(x)) _ 2) ^ 2)`),
and `||…||` — is marked (`Meta::bar_fence`, latexml_math_parser/src/
semantics.rs) and is a bare item of a function's argument, never the group of its easy arguments:
`\log||x||_2^2` is `logarithm@(((norm@(x)) _ 2) ^ 2)`, `\min_x||Ax-b||_2^2`
`(minimum _ x)@(((norm@(A * x - b)) _ 2) ^ 2)`, `\arg\min_w||y-Xw||^2` likewise — as 57al read them;
an operator's bare argument too (`bare_abs`): `\nabla u||v||` is `nabla@(u * norm@(v))`, as
`\nabla u\|v\|` is (Perl `nabla@(u) * norm@(v)`).
Four single bars around a term are the norm, never nested absolute values (Perl's ordered choice,
:294 before :299). The mark is the fence's own, so a `split` row, whose bars read through XMRefs,
reads as the inline formula. A `\left`/`\right` double bar (`\left\|`, `\left\Vert`,
`\left\lVert`) is a bar pair as in Perl, which re-roles its ∥ a VERTBAR (TeX_Math.pool.ltxml:755):
the markup keeps its OPEN/CLOSE role, the lexer reads it as the stretchy bars (latexml_math_parser/
src/util.rs), so `\exp\left\|x\right\|^2` is `exponential@((norm@(x)) ^ 2)`, as Perl. A bare
`\lvert…\rvert`, `\lVert…\rVert` are OPEN/CLOSE delimiters, groups as in Perl
(`\log\lvert x\rvert\,y` is `logarithm@(absolute-value@(x)) * y`). The bar pairs are a nonterminal
of their own (`bare_abs`, grammar/builder.rs), which a function's group application
(`group_factor`) does not include, so each has one derivation after a function.

**Guard**: goldens `tests/parse/opfunction_arguments.tex#opfunction_group_application_takes_its_scripts`,
`tests/parse/bar_pairs.tex#left_double_bar_is_a_bar_pair`.

### 354. Three kerned `\left|` bars are one operator-norm delimiter

Perl reads stacked `\left|` bars (`\left\vert`, the `\vertii`/`\vertiii` idioms) with its bar grammar:
two around two are `SINGLEVERTBAR SINGLEVERTBAR absExpression …` (MathGrammar:294), `\vertii{x}` is
`norm@(x)` with a merged `‖`; three are a norm around an absolute value, `\vertiii{x}` is
`norm@(absolute-value@(x))`, kerned or not. (The `\left\lvert` spelling lexes by its name, no
`SINGLEVERTBAR`, and nests absolute values.)

**Rust** (task #263): two `\left|` around two `\right|` are one norm as Perl, kerned or not
(`stretchy_norm_fenced`, latexml_math_parser/src/semantics.rs; until 57am the pair needed the kern and
read `\left|\left|x\right|\right|` as |(|x|)|). Three bars stacked with a negative kern each side (the
kern folded into the bar's `rpadding`) are one delimiter pair, `operator-norm@(x)` with `⦀`
(`stretchy_triple_norm_fenced`) — the divergence; unkerned they are Perl's
`norm@(absolute-value@(x))`. The `\left\lvert` spelling lexes as the same stretchy bar (util.rs reads
its glyph), so it stacks too — a divergence: `\left\lvert\left\lvert x\right\rvert\right\rvert` is
`norm@(x)` (Perl `absolute-value@(absolute-value@(x))`), three unkerned `norm@(absolute-value@(x))`
(Perl three absolute values); the stack renders as ‖x‖, and |(|x|)| is no one's intent. A merged glyph takes the padding after its stack's last bar,
the kerns between the bars being the glyph's own (57am review round 7). Only single bars stack: a
`\left\|` inside a stack is a bar pair of its own (`\left|\!\left\|A\right\|\!\right|` is
`absolute-value@(norm@(A))`, as Perl). One level of Perl's `absExpression` nesting.

**Guard**: golden `math/norm_kerned_delims` (2211.13044 §S4.Ex17; `\vertiii{x}\le C\vertiii{y}` is
`operator-norm@(x) <= C * operator-norm@(y)`), golden `tests/parse/bar_pairs.tex#left_double_bar_is_a_bar_pair`.

### 355. A placeholder between `\left`/`\right` bars is its norm

Perl's `absExpression` needs a term (MathGrammar:410-412), so `\left\|\cdot\right\|_\infty` — the norm
of an argument slot — leaves the formula unparsed.

**Rust** (batch 57am): a placeholder between `\left`/`\right` bars is the norm or absolute value of
the slot, as the placeholders fenced in parentheses are (`f(\cdot)`, `\langle\cdot,\cdot\rangle`):
`\left\|\cdot\right\|_\infty` is `(norm@(cdot)) _ infinity`, `(A,\left\|\cdot\right\|_\infty)`
`vector@(A, (norm@(cdot)) _ infinity)` (a normed space, #371) — what 57al read through the OPEN/CLOSE placeholder rule
before the lexer made a `\left\|` pair bars (#353); a stack around the slot is its norm, kerned or not
(`\vertii{\cdot}` and `\left|\left|\cdot\right|\right|` are `norm@(cdot)`), three kerned bars its operator norm
(#354). The slot is a MulOp or BinOp (`\cdot`,
`\bullet`; `\left|\times\right|` is `absolute-value@(times)`) — `\left|+\right|`, `\left|=\right|` stay
unparsed, as in Perl. Sided bars only: between unsided ones, `|x|\cdot|y|` would offer a `|\cdot|`
reading of its own, so `\|\cdot\|` and `|\cdot|` stay unparsed, as in Perl.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bars_around_a_placeholder` (57am7 A/B: 78 formulas,
30 papers; 2605.00709, 2605.05645, 2605.07463).

### 356. A sided bra and ket around an operator holding bars are one quantum-operator-product

Perl's `ketExpression` forbids bars (MathGrammar:401), so between `\left\langle a\right|` and
`\left|e\right\rangle` an operator holding bars cannot be the middle of a braket: Perl reads
`\left\langle a\right|b\left|c\right|d\left|e\right\rangle` as `bra@(a) * b * absolute-value@(c) * d * ket@(e)`
and `\left\langle\Psi\right|\left|\Phi\right|\left|\Psi\right\rangle` as `delimited-⟨⟩@(Psi * norm@(Phi) * Psi)` —
a norm of two bars TeX paired apart.

**Rust** (batch 57ap): the sided bars fix the pairing — the `\right|` closes the bra, the last `\left|` opens the
ket, the pairs between are absolute values — and the bracket's middle is any expression, so both are the matrix
element Dirac notation writes: `quantum-operator-product@(a, b * absolute-value@(c) * d, e)`,
`quantum-operator-product@(Psi, absolute-value@(Phi), Psi)`. Unsided bars are unchanged.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the "divergence #356" rows).

### 357. An unresolved or unrecognized rewrite scope is reported once per conversion

Perl compiles each rewrite rule once and reports its scope as it compiles it (Rewrite.pm:49-55, :308-311): three
`\lxDeclare`s scoped to the same missing label log three `Error:misdefined:<rewrite> No id for label …`.

**Rust** (batch 57aq): once per distinct label or unrecognized scope per conversion (a State flag). A streaming
conversion compiles its rules once per fragment, where a per-rule report would repeat per fragment; the reading — the
rule applies nowhere, or unscoped — is Perl's.

**Guard**: `perfect_kernel_batch57::declaration_scopes_resolve_as_perl` (one error each).

### 358. In braces, a `\left|` pairs its `\right|` evaluation bar

Perl's `scriptFactorOpen` tries the set-builder bar before the bar pair in braces (MathGrammar:486-491, `FormulaNOBar
suchThatOp Formulae`), so `\left\{a\left|f\right|_{x=0}\right\}` is `conditional-set@(a, evaluated-at@(f, x = 0))` — the
`\left|` that TeX paired with the `\right|` read as the set-builder bar.

**Rust** (57ao and before; kept deliberately since 57ap/57ar): TeX's pairing — `set@(a * (absolute-value@(f)) _ (x = 0))`;
a `\left|` divider whose next item evaluates at a stretchy bar is refuted (`left_bar_pairs_an_evaluation_bar`). In
parentheses and brackets Perl pairs them too (`Factor` before `evalAtOp`, :257-263).

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the brace row).

### 359. A `\middle|` divides a conditional

Perl's conditional reads a VERTBAR (`evalAtOp ExpressionsNoBars`, MathGrammar:261-268); a `\middle|` is role MIDDLE
(etex `\middle`), so `P\left(A\middle|B\right)` is unparsed.

**Rust** (before 57ap; recorded 57ar): the `middle_bar` fence rules read it — `P@(conditional@(A, B))`, the `\middle|`
the author wrote to divide.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the `\middle|` rows).

### 360. A streamed conversion resolves a duplicated label scope to its first copy

Perl's `rewrite` fills its label map in document order (Rewrite.pm `getLabelID` over the document's `labels`), so
when a `\label` is carried twice — an author error LaTeX warns about as multiply defined — a
`scope=label:…` declaration scopes to the last copy; the eager Rust path does the same (`load_labels_for_rewrite`).

**Rust streaming** (57aq/57as): the shared label map (`core_interface.rs`, built before pass 2 from the pass-1
fragment index and the spine's own labelled nodes) keeps the first copy it meets, and a fragment resolves its own
copy before the shared one; the fragment index records no positions, so the document order across spilled segments
and the spine is not known there. Only a duplicated label differs; a label carried once resolves identically.

**Guard**: none (streaming vs eager on a duplicated label is not pinned; `114_streaming_math` covers single labels
through `tests/math/declaration_scope_labels.tex`).

### 361. A closing mark outside a script is presentation

Perl's `Subscript`/`Superscript` start rules alone end in `endPunct(?)` (MathGrammar:84-92), which wraps a
script's content in a one-item list (`NewList`, MathParser.pm:1463-1475): `x_{a,}` x _ (list@(a)). A formula
parsed with `Anything,` has its closing marks set aside first (MathParser.pm:657-670); an `XMArg` or `XMWrap`
parsed with plain `Anything` (MathGrammar:64-82, MathParser.pm:384-388) has no closing mark, so `\boxed{a+b.}`, `\frac{a,}{b}`,
`k\xrightarrow{a.}o` and `a\mathrel{=,}b` are unparsed in Perl.

**Rust** (rc59b; landed with the batch that records it): the grammar admits a closing mark in every container
(`statements end_punct => postfix_embellished`); a mark that closes a script builds Perl's list, any other is
presentation — `\boxed{a+b.}` a + b, `a\mathrel{=,}b` a = b — where the mark's value once chose `list@` for any
container (`list@(a + b)`; 763 formulas in 82 papers of the 57ar A/B, 709 of them `\boxed{… .}`). A mark followed by a
wide space reads `fragments@(…)` where the accidental one-item list swallowed what followed it (`a=b.\quad c=d`,
`\boxed{x=0,}\qquad y`: was `list@(…)`; Perl unparsed; `a=b\quad c=d` reads `formulae@(…)`, the `statements
end_punct` result feeding no `formulae` rule).

**Guard**: golden `tests/parse/trailing_punctuation.tex`.

### 362. A reversed-bracket pair is an open interval

Perl's grammar has no rule for a reversed bracket pair: `]0,1[` and `]0;1[` are unparsed (`isMatchingClose`,
MathParser.pm:1379-1384, over `%balanced` :1348-1356, pairs `[` with `]` only), and its `%enclose2` table
(:1368-1373) names comma pairs alone.

**Rust**: `interval_term` admits `rbracket expression punct expression lbracket` (`grammar/builder.rs`, the French notation
for an open interval; expression endpoints since 57bj, `]-1,1[`), named `open-interval` with a comma or a `;` — the reversed pair marks the interval, and `;` is the
French separator (`]0;1[`). Every other pair is named as Perl's tables name it: a comma pair an interval, any other
separator a `list` (57ba) — `[0;1]` is a closed interval in French papers but a vertical concatenation in machine-
learning ones (`[\mathbf{a};\mathbf{b}]`, 2605.00423), so the neutral `list`.

**Guard**: golden `tests/parse/fenced_lists.tex` (`(a;b]`, the `interval` action's `list`).

### 363. A function of a named fence takes the fence as one argument

Perl's grammar reads a function's arguments before it names any fence: `addEasyArgs` (MathGrammar:570-577) takes
`OPEN Argument (argPunct Argument)* balancedClose` for any balanced pair (`isMatchingClose`, MathParser.pm:1379, over
`%balanced` :1348-1356) and `ApplyDelimited` drops the delimiters. A delimiter pair that names a function of its
content is dropped with them, and a set-builder's bar or colon splits it:

| Input | Perl | Rust |
|---|---|---|
| `\log\lfloor x\rfloor` | `logarithm@(x)` | `logarithm@(floor@(x))` |
| `\log\lceil x\rceil` | `logarithm@(x)` | `logarithm@(ceiling@(x))` |
| `\log\lvert x\rvert` | `logarithm@(x)` | `logarithm@(absolute-value@(x))` |
| `\log\lVert x\rVert` | `logarithm@(x)` | `logarithm@(delimited-∥∥@(x))` |
| `\max\{x\mid x>0\}` | `maximum@(x * ket@(x) * 0)` | `maximum@(conditional-set@(x, x > 0))` |
| `\max\{x:x>0\}` | `maximum@(x colon x > 0)` | `maximum@(conditional-set@(x, x > 0))` |
| `\Pr\left(A\middle\|B\right)` | `Pr@(A, B)` | `Pr@(conditional@(A, B))` |
| `\max\langle a,b\rangle`, `\max_m\langle t,b_m\rangle` | `maximum@(a, b)` | `maximum@(delimited-⟨⟩@(list@(a, b)))` (an inner product; 57bf) |
| `\log\langle Z\rangle`, `\max\langle x\rangle` | `logarithm@(Z)` | `logarithm@(delimited-⟨⟩@(Z))` (an average; 57bf) |

**Rust** (57bb; `fenced_tuple_items`, semantics.rs): a function takes the arguments between a matched pair of grouping
delimiters — parens, brackets, braces; angle brackets are an inner product or an average, one argument whatever
they hold (57bf; 95 formulas in 38 papers of the A/B, 2605.02896 `\max\langle Q,X\rangle`, 2605.20551) — around a
grouping fence (`delimited-…`, `list`, `set`, an
interval, a paren `vector`); any other fence is one argument. `|x|` agrees with Perl either way (bars are an
`aBarearg`: `logarithm@(absolute-value@(x))`). An operator head, bare, scripted or nested, takes them alike (57bf, 57bh):
`\nabla_X[Y,Z]` — a covariant derivative of a Lie bracket — reads Perl's `(nabla _ X)@(Y, Z)`, two arguments where
57be kept `closed-interval@(Y, Z)` one; neither is a commutator. Kept as Perl's though not the intent (SYNC): a function
before an operator's application multiplies it, `\operatorname{stf}\nabla(a-b)` `stf * nabla@(a - b)`,
`\min_h\operatorname*{R}(h,P)` `minimum _ h * R@(h, P)` (57bh restored the parse 57bf lost).

**Guard**: golden `tests/parse/fenced_lists.tex` ("A function takes the arguments between any delimiters":
`\log\lfloor x\rfloor`, `\max\{x\mid x>0\}`, `\log\langle Z\rangle`), `tests/parse/operator_application.tex`
("An operator takes the arguments between delimiters").

### 364. A fenced conditional's given part and its joint part are lists

Perl's grammar reads a paren group's comma list before the conditional bar: `p(x,y|z)` is
`p * open-interval@(x, conditional@(y, z))` — the bar binds only the last item, and the pair is named an interval.
Its brace form reads the joint whole: `\{x,y|z\}` `conditional-set@(list@(x, y), z)`.

**Rust**: a fenced conditional takes the whole list on either side of the bar, `conditional@(list@(x, y), z)` — the
joint of x and y given z, as the notation means — and the given part is a list too, `p(x|y,z)`
`conditional@(x, list@(y, z))` as Perl (57bd: both had been renamed `open-interval` by `rename_fenced_lists`). An
unfenced `a,b|c` still reads `list@(a, conditional@(b, c))`, as Perl (`vertbar_modifier_listlhs`). A list on
both sides at once reads the joint given both, `p(x,y|z,w)` `conditional@(list@(x, y), list@(z, w))` (57bh; Perl
`open-interval@(x, conditional@(y, list@(z, w)))`); the brace form `\{x,y|z\}` as Perl. A list holding a relation
is Perl's `formulae` beside the bar as in a fence (`FormulaNOBar suchThatOp Formulae`, MathGrammar:487-491):
`\{x : a<1, b<2\}` `conditional-set@(x, formulae@(a < 1, b < 2))`; `\mathbb{E}[Y\mid A=1,X]`
`E@(delimited-[]@(conditional@(Y, formulae@(A = 1, X))))` (`E` an unknown applied, #18), which Perl garbles; a
paren conditional's list too, `\Pr(X=1,Y=2|Z=3)` `Pr@(conditional@(formulae@(X = 1, Y = 2), Z = 3))` (was `list@`).
Brackets and braces take a list on either side, and a brace's colon only (Perl's `suchThatOp`, MathGrammar:499-501):
`\{x,y|z,w\}` `conditional-set@(list@(x, y), list@(z, w))` and `\{x,y : x<y\}` as Perl, `[a,b|c]`
`delimited-[]@(conditional@(list@(a, b), c))` (Perl `closed-interval@(a, conditional@(b, c))`); `\{\Gamma\vdash A,B\}` stays
unparsed (Perl `set@(Gamma proves list@(A, B))`). Open: `\middle|` sets with a list (`\left\{x,y\middle|z\right\}`), a
multirelation item (`\{i : 1\le i\le n, i\ne j\}`, a `formula_list` limit), and a plain paren list holding a relation,
still `vector@(…)` where Perl says `formulae@(…)` (SYNC, "Application residuals after 57bf–57bh").

**Guard**: goldens `tests/parse/bar_pairs.tex` ("A list beside a conditional bar is the conditional's"),
`tests/parse/functions.xml` (`p(x,y|\Theta)`).

### 365. A script that is one float script alone is a script of an absent base

A script whose argument is a lone float script — `F^{{}^{\prime}}`, `H_{{}_{\mathrm{I}}}`, `(a-b)^{{}^{\prime}}` —
reaches Perl's parser as a one-node argument, which `parse_single` passes through unparsed (MathParser.pm:680-681): the
output keeps the raw `<XMApp role="FLOATSUPERSCRIPT">`, text `F ^ ^prime`. A float followed by more
(`x^{{}^{\prime}\prime}`) fails Perl's `rule 'Superscript'` (TeX_Math.pool.ltxml:501-516).

**Rust** (57bj; `lone_float_script`, grammar/builder.rs): the script argument may be one float script, built as
`NewScript(absent, ′)` (the shape of MathGrammar:79-80), so `F^{{}^{\prime}}\leq F` reads `F ^ ^ prime absent <= F` —
a parsed structure where Perl keeps raw markup, the float nested as written; a float followed by more stays unparsed, as
in Perl.

**Guard**: golden `tests/parse/rust_parse_additions.tex` ("A script may be a float script alone").

### 366. A colon list between delimiters is a list

Perl reads a colon between a fence's items as a metarelation over the whole content (`OPEN Formulae CLOSE` with a
`metarelopFormula`, MathGrammar:69, :118-125; one item fenced, MathParser.pm:1412-1415): `[a:b]`
`delimited-[]@(a colon b)`, `(x:y:z)` `x colon y colon z`; a fenced metarelation is a statement, so `[K:\mathbb{Q}]=2`
and `x\in[a:b]` are unparsed.

**Rust** (57bm; `colon_list`/`colon_terms`, grammar/builder.rs): a colon list between brackets (expression items) or of
three or more terms between parentheses is a `list`, as the two-item `(a:b)` already was (`fence` names a non-comma
separator `list`, the default of Perl's `Fence` tables, MathParser.pm:1405-1409): `[a:b]` `list@(a, b)`,
`[x_0:\dots:x_n]` `list@(x _ 0, dots, x _ n)`, `(x:y:z)` `list@(x, y, z)`, and a factor — `list@(K, Q) = 2`,
`x element-of list@(a, b)`, `c ^ (list@(1, t))`. Nothing in the syntax tells an index `[G:H]` from a range `[1:t]` or
a projective point, so the neutral `list`; a function takes the list whole, `\max[a:b]` `maximum@(list@(a, b))`
(Perl `maximum@(a colon b)`). A paren item may carry a sign, `(1:-1:0)` `list@(1, - 1, 0)`, but no sum: a sum between
colons in parentheses is a tensor double contraction (`(\nabla x:\nabla y-\nabla z:\nabla w)`, 2605.21445, 2605.01156,
2605.09779), no list; it stays unparsed, as does `(a+b:c:d)`, which Perl reads `a + b colon c colon d`. Witnesses
2605.14715, 2605.08808, 2605.01646, 2605.25087, 2605.00473 (~216 formulas / 52 papers were unparsed).

**Guard**: golden `tests/parse/fenced_lists.tex` ("A colon list between delimiters is a list").

### 367. Explicit space or a differential `d` ends a trig function's bare argument

Perl's trig bare argument is greedy (`trigBarearg : aTrigBarearg moreTrigBareargs`, MathGrammar:340-357): every
following atom, identifier, unknown or number joins it, whatever spacing the author wrote — `\sin\theta\,d\theta`
`sine@(theta * d * theta)`, `\sin\theta_W\,C_{uB}` `sine@(theta _ W * C _ (u * B))`, `\sinh r_u\,\mathbf a_u`,
`d\cos\theta_1 d\cos\theta_2` `d * cosine@(theta _ 1 * d) * cosine@(theta _ 2)`.

**Rust** (57bo; `trig_argument_juxtaposition`, `ends_with_space`, `is_differential_d`, semantics.rs): a juxtaposed
item does not join the argument after an item carrying explicit space — a positive `rpadding`, which
`filter_hints` folds from `\,`, `\:`, `\;`, `~`, `\ ` or a `\kern`/`\hspace` under 10pt onto the token or script
before it (a wider space, `\quad`, splits the formula as a PUNCT; a net zero or negative one, `\!`, is none) — nor
when it is the differential letter `d`; the greedy-argument prune (`leaves_a_trig_bare_argument`) does not fire
there: `sine@(theta) * d * theta` (with or without the space), `sine@(theta _ W) * C _ (u * B)`, `sine@(x) * y`
for `\sin x\;y`, `d * cosine@(theta _ 1) * d * cosine@(theta _ 2)`, `dA=\sin\theta d\theta d\phi` sin@(θ)·dθ·dφ.
Otherwise the argument stays Perl's greedy chain (`\sin 2x` sine@(2x)); `\sin kd` reads sin@(k)·d. Witnesses
2605.12033, 2605.03429, 2605.00265, 2605.10520, 2605.26406 (~90 formulas read wide across a space).

57bq (user direction 2026-09-29: prune a compound trig argument by the types inside it): one predicate,
`ends_trig_argument`, answers both actions, so exactly one reading survives. Besides space and `d` (a space before a
number sits in the number token's own text, `\cos\theta\;1`), an item ends the argument when it carries a mark no
scalar angle does (`non_scalar_mark`, read from the token's decoded `_font` and the built atom's shape): bold; a
calligraphic, script, blackboard, fraktur or sans-serif family; an upright Latin letter; a `\vec`/`\overrightarrow`,
`\hat`/`\widehat` or `\dot`/`\ddot` accent; a Leibniz derivative; a built atom holding a trig function; an adjoint
or transpose superscript (`\dagger`, `\top`, `\intercal`, upright or sans-serif `T`) — unless the argument so far
is coefficients only — numbers, π, a fraction or root of numbers (`\sin 2\mathcal P` sine@(2·P),
`\cos 2\pi\mathbf k\cdot\mathbf r`) — or already holds an item of the same mark (`\cosh\mathcal K\mathcal S`, an
all-upright document). A Leibniz fraction is read from its parsed parts (`\frac{d\theta}{dt}`,
`\frac{\partial^2u}{\partial x^2}`; its denominator a product or application, not `\frac{d_1}{d_2}`); a split
row's items through their XMRefs. `\cos\phi_m\vec e_{x_m}` cos@(φ_m)·e⃗_{x_m}, `\sin\theta\mathrm P_1` sin@(θ)·P_1,
`\sin\theta\dfrac{\partial Y}{\partial\theta}` sin@(θ)·∂Y/∂θ, `\cos\theta A^\dagger` cos@(θ)·A†; `\tilde`, `\bar`,
Greek and capital italic letters name angles too and join (`\cos\omega\tilde t`, `\cos\omega T`). Witnesses
2605.31180, 2605.28946, 2605.14528, 2605.15451, 2605.20574, 2605.09444, 2605.14924, 2605.26410. Known splits against
intent: `\cos\Omega\hat t` (2605.03417, a dimensionless t̂); a font switch around the whole formula,
`\mathbf{\cos\omega t}`, `\mathrm{\cos\omega t}` (the function keeps its operator font and Greek letters keep
theirs, so the DOM cannot tell it from `\cos\omega\mathbf t`). No evidence, greedy as Perl: `\cos\theta\sigma_x`,
`\sin\theta_k D_{x_k}f`, `(r\sin\theta\Omega)^2`, `\cos\omega_1X_1`.

57cj.7, 57cj.8 (the 57cj.6 and 57cj.7 reviews; `ends_with_space` → `trailing_padding`, `trig_letter_application`): the
space after the argument so far is its last token's, found down its right edge — through a product, a derivative's
operand and a bare application's argument however deep (`\sin\partial_x u\,v` sin@(∂_x u)·v, `\sin\partial_x 2\,\partial_y
u\,v` sin@(∂_x(2·∂_y u))·v, `\sin\log u\,v` sin@(log u)·v), and through a group, or an application to one, to its closing
delimiter: `\sin f(x)\,y` sin@(f(x))·y, `\sin\Gamma(x)\,y`, `\sin\log(x)\,y` sin@(log(x))·y, `\sin\exp\bigl(x\bigr)\,y`,
`\sin\log\left(x\right)\,y`, `\sin\log[x]\,y`, `\sin\max(a,b)\,y`, `\sin\partial_x\log(u)\,v`, where the argument took
y before; a space inside the delimiters ends nothing (`\sin\log(x\,)y` sin@(log(x)·y)). Formulas unparsed before now
parse: `\sin\log|x|\,y`, `\sin\log\|x\|\,y`, `\sin\exp x\,y`, `\sin\arcsin x\,y`, `\sin\operatorname{tr}A\,B`,
`\sin\ln\frac{x}{2}\,y`, `e^{i\sin\log x\,t}`. A letter's application to a group (#18) does not reach across the space
that ends the argument: `\cos\phi\,(1-x)` cos@(φ)·(1−x), `\sin k\,(x-y)` sin@(k)·(x−y), `\cos\omega\,(k)\,t` cos@(ω)·k·t
(`\cos\phi(1-x)` stays cos@(φ(1−x)), `\cos\omega(k)\,t` cos@(ω(k))·t — a dispersion phase may mean cos@(ω(k)t), but the
rule reads the space). Witness 2605.29683 A1.E17 `a\cos\phi\,\bigl(10-\cos(6\theta)\bigr)\,r^{6}`: cos@(φ@(10−cos 6θ)·r⁶)
before 57cj.7, cos@(φ@(10−cos 6θ))·r⁶ in 57cj.7, a·cos@(φ)·(10−cos(6θ))·r⁶ now; its delta A/B's other readings, all this
class: `\sin^{2}\beta\,\big(F(\dots)-F(\dots)\big)` (sin²)@(β)·(F−F) (2605.11097), `-q\cos\theta~(\bar{\mathcal F}=\dots)`
(2605.15566), `\cosh^{2}Z\,(dZ^{2}+d\varphi^{2})` (cosh²)@(Z)·(dZ²+dφ²) (2605.27600). A derivative inside the argument ends
where the argument ends (#374). Open (awaiting a ruling): a plain argument's coefficient run before a space ends there,
`\cos 2\,\theta` cos@(2)·θ, `\sin 2\pi\,ft` sin@(2π)·f·t, while inside a derivative it does not (#374).

57cj.9 (the 57cj.8 review; latent, its probes and Perl's own readings): the argument takes an OPFUNCTION's application as
an OPFUNCTION's argument does — Perl's `aTrigBarearg : preScripted['OPFUNCTION'] addOpFunArgs` (MathGrammar:341-343),
where the old item was one unscripted OPFUNCTION applied to one factor (`opfunction factor`), so Rust left unparsed what
Perl parses (`trig_arg += opfunction_application | bound_application | scripted_opfunction_application =>
trig_function_item`): `\sin\log_2 x` sin@(log₂(x)), `\sin\log^2 x`, `\cos\ln^2 x`, `\sin\max_i x_i` sin@(max_i(x_i)), `\sin\log 2x`
sin@(log(2x)), `\sin\log uv`, `\sin\operatorname{tr}AB`, `\sin\ln\frac{x}{2}y`, `\sin\log\log x`, `\sin\log\max(a,x)`,
`\sin\log\partial_x u`, `\sin\log|x|y`, and `\sin\log_2(x)` sin@(log₂(x)), which read sin@(log₂)·x (an unapplied OPFUNCTION
ending the argument takes what it applies to, `leaves_a_trig_bare_argument`); a group application takes its scripts
(#351: `\sin\log(x)^2` sin@((log x)²), was sin@(log(x²))). Its bare argument ends where the trig argument ends
(`crosses_within`, `ends_a_trig_argument_within`): `\sin\log u\,v` sin@(log u)·v, `\sin\log_2 x\,y`, and before a trig
function, as juxtaposed trig functions are separate factors (`\sin\log x\cos y` sin@(log x)·cos y, where alone `\log x\cos y`
is log@(x·cos y)); a mention of a bound head's variable continues it (`\sin\max_i u_i\,v_i` sin@(max_i(u_i·v_i)), the head's
scope following its variable, user ruling 2026-09-30). A letter's postfixed application to a group does not reach across
the space either (`trig_letter_postfixed`: `\cos\phi\,(1-x)!` cos@(φ)·(1−x)!, `\sin^2\phi\,(1-x)!`). A limit-type
operator's application the trig function takes (#390) keeps Perl's product when its operand crosses the argument's end
(`\sin\det A\,y` sin·det@(A·y)).

57cj.10 (the 57cj.9 review): the OPFUNCTION's bare chain in a trig argument is built without a trig function after its
first item (`trig_op_bare_arg`, `op_bare_plain_next`), so no tree derives log@(x·cos y) to refuse it — refused trees per
site multiplied: `\sin\log x\cos y+\cos\log x\sin y` 25 trees and an `ambiguous_math` warning, eight such terms the
5,001-tree cap; now one tree (`parse_tree_count_limits`). Space, `d` and type-mark ends stay refusals in the actions (the
grammar cannot see them): `\sin\log x\,dx+\cos\log x\,dx` enumerates 36 trees (9 before 57cj.9), a residual; the
parallel trig chain also grows the bocage, so `\sin^2\log\log x\,dy` (491 or-, 503 and-nodes) takes the tree route and warns
(14 trees, the same reading; 8 again in 57cj.11). Until 57cj.13 an argument
list kept a letter's application across the space (`\cos\phi\,(x,y)` read cos@(φ(x,y)); the #18 row's "an argument list keeps
the application", 2605.24758) — narrowed in 57cj.11 to a parenthesized list of variables; since 57cj.13 a tuple after a space
in a trig argument is a vector (`\cos\phi\,(x,y)` cos φ·(x,y), below), the argument list a derivative operand's rule (#374). The OPFUNCTION before a big operator is an item (#390: `\sin\log\det A`). Readings 57cj.9
changed by these rules: `\sin\log x\cdot y` sin@(log(x·y)), as `\log x\cdot y`; `\cos 2\pi i\mathbf k` cos@(2πi·k) and
`\cos\log 2\mathbf k` cos@(log(2k)) — a coefficients-only argument takes a marked item (the type-mark rule's exemption, with
the imaginary unit and a function of a constant now constants). With the physics package loaded `\sin` is an OPFUNCTION
(physics' own operator), so none of these trig rules apply there (`\sin x\,y` sin@(x·y)) — a residual (SYNC).

57cj.11 (the 57cj.10 review; latent, its probes): an argument list is a parenthesized comma list of variables only —
letters, scripted letters (not to a constant power), ellipses: `(x,t)`, `(x_1,\ldots,x_n)`, `(x',y)` (`is_an_argument_list`,
`is_a_variable`). Numbers, applications, powers, sums, brackets or another separator make a vector, interval or list that
the letter multiplies, as Perl reads it: `\cos\theta\,(1,0)+\sin\theta\,(0,1)` cos θ·(1,0)+sin θ·(0,1),
`\hat r=\sin\theta\,(\cos\phi,\sin\phi,0)+\cos\theta\,(0,0,1)`, `\cos\phi\,[x,y]`, `\cos\phi\,(x;y)`, `\cos\phi\,(x^2,y)` (57cj.10
read φ applied to each). A trig function's argument composes another trig function's application
(`trig_composed_arg`; Rust-only unparsed before — Perl's `aTrigBarearg` has no TRIGFUNCTION, MathGrammar:340-356, so Perl
multiplies, sine·cosine@(x)): `\sin\cos x` sin@(cos@(x)), `\cos\sin x+1`, `\sin^2\cos x` (sin²)@(cos@(x)), `\sin\cos^{-1}x`,
`\int\sin\cos x\,dx` ∫(sin(cos x)·dx), `\partial_x\sin\cos u`; the inner argument is the one that goes on and ends
(`\sin\cos xy` sin@(cos@(x·y)), `\sin\cos x\,y` sin@(cos@(x))·y, `\sin\cos x\cos y` sin@(cos@(x))·cos@(y)); an inner trig
function before a big operator takes a limit-type operator's application and multiplies a sum (#390: `\sin\cos\det A`
sin@(cos@(det A)), `\sin\sin\sup_x f`, `\sin\tan\lim_n a_n`, `\sin\cos\sum_i x_i` sin@(cos·∑…)). A function before an
operator's application inside the argument multiplies it, as outside (`\log\nabla u` log·∇u): `\sin\log\nabla u`
sin@(log·∇u), `\sin\cos\nabla u` sin@(cos·∇u) (Rust-only unparsed before; Perl sine@(logarithm)·∇u). The OPFUNCTION chain's
first item is built without a trig function at every nesting (`trig_op_bare_item`, `trig_opfunction_item_application`):
`\sin\log\log x\cos y` 7 → 3 trees (the 3 of `\sin\log\log x` alone; `\log\log x` has 2), four sites 3,182 → 81, five
under the cap (243); `\sin^2\log\log x\,dy` 14 → 8 trees, no warning. Why composition: sine·cosine@(x) multiplies a bare
head, which has no value; composition is the only meaningful reading (user ruling 2026-09-29: the mathematically correct
reading in context); latent, no corpus witness in the 3,003 A/B sources (the 57cj.11 review's mining: the only hits are
the operator-name lists of 2605.21236).

57cj.12 (the 57cj.11 review; latent, its probes): an argument list holds at least one variable and may hold numbers
beside it — an evaluation point (`(0,t)`, `(x,0)`, `(x,-1)`, `(x,\tfrac12)`) — and an accented letter is a variable
(`is_accented_letter`: an OVERACCENT or UNDERACCENT over a letter, which lexes as one ATOM: `\hat x`, `\dot q`, `\vec r`),
on all three paths (trig argument, derivative operand, postfix) until 57cj.13, which leaves the trig path a vector (below):
`\cos\phi\,(x,0)` read cos@(φ(x,0)) and `\cos\phi\,(\hat x,y)` cos@(φ(x̂,y)) in 57cj.12, cos φ·(…) since; `\partial_x u\,(0,t)=0` ∂_x(u(0,t))=0, `\partial_q L\,(q,\dot q,t)`, `\partial_t\psi\,(\vec r,t)` — as the bare
letter's `u\,(x,0)=g(x)` u@(x,0), `L\,(q,\dot q,t)` (57cj.11 read these products, an m13 → m14 regression; Perl multiplies
them all). Numbers alone (π among them) stay a vector: `\cos\theta\,(1,0)`, `\cos\phi\,(0,\pi)`; so do applications, sums
and powers (`\sin\theta\,(\cos\phi,\sin\phi,0)`). Residual: a numeric superscript is a power, so contravariant coordinates
read as a vector (`\cos\phi\,(x^1,x^2)` cos φ·(x¹,x²), `\partial_t u\,(x^1,x^2)`; telling an index from a power needs
the whole tuple). An ellipsis opens a trig argument, as it opens an OPFUNCTION's (`trig_arg += elideop`; `\ldots` already
did, a `factor_base` ID): `\cos\cdots x` cos@(⋯·x), `\sin\,\cdots`, `f(\sin\cdots)`, `\sin\cos\cdots x` sin@(cos@(⋯·x))
(Rust-only unparsed before; Perl cosine@(cdots·x), sine·cosine@(cdots·x)); after an item it still ends the argument
(`\cos\theta_1\cos\theta_2\cdots\cos\theta_n`, divergence #3). `\cos\cdots x` kept its twin cos@(⋯)·x as a second reading
(two unique parses; `\cos\ldots x` has one), refused since 57cj.13.

57cj.13 (the 57cj.12 review; latent, its probes; no corpus witness in the 3,003 A/B sources): a trig argument is an angle,
so a tuple after a space is a vector the trig value scales, whatever its items (`trig_letter_application`; decided
2026-09-30 by the main loop under the user's ruling "the mathematically correct reading in context", 2026-09-29, for the
user's review): `\mathbf v=\cos\alpha\,(v_x,0)+\sin\alpha\,(0,v_y)` cos α·(v_x,0) + sin α·(0,v_y), `\cos\theta\,(a,0)`,
`\cos\phi\,(x,0)`, `\cos\phi\,(x,y)` cos φ·(x,y), `\cos\phi\,(x,0)!` cos φ·(x,0)!, as Perl (57cj.10-57cj.12 applied φ to the
argument lists and evaluation points above). The argument list is a derivative operand's rule only (#374:
`\sin\partial_x u\,(x,0)` sin@(∂_x(u(x,0)))); without the space the letter applies (#18: `\cos\phi(x,y)` cos@(φ(x,y))). An
ellipsis opening the argument lets it go on as any first item does, a run of ellipses too (`trig_ellipses`), and its twin
cos@(⋯)·x is refused (`leaves_a_trig_bare_argument`, `is_an_ellipsis_run`): `\cos\cdots x\,y` cos@(⋯·x)·y, `\sin^2\cdots x`
(sin²)@(⋯·x), `\sin\cdots\cdots x` sin@(⋯·⋯·x), `\tan\cdots\cdots\cdots x` (57cj.12 read cos@(⋯)·x·y, (sin²)@(⋯)·x,
sin@(⋯)·⋯·x; Perl as now, but for the space: cos@(⋯·x·y)); `\cos\cdots x` one parse, four sites one unique of 16
enumerated (was 16 unique), as `\cos\ldots x` and `\cos a x`. After an addop or a MulOp the ellipsis is a bare head's
argument, not an elided operation's operand (`infix_apply_and_elide` refuses a trig function, OPFUNCTION or operator
before it): `a+\cos\cdots` a + cos@(⋯), `\sin\cdots-\cos\cdots` sin@(⋯) − cos@(⋯), `a+2\cos\cdots`, `a+\log\cdots`,
`a+\nabla\cdots`, as Perl (57cj.12 read a + cos + ⋯, which rendered a second `+` the source does not have). Residual: an
elided operation shows its operator before the ellipsis (`a+b\cdots` a + b + ⋯, `b_1+b_2\cdots+b_j`; Perl a + b·⋯; SYNC row
"Math-parse residuals of the 57cj train" (9)).

57cj.14 (the 57cj.13 review; latent, its probes; no corpus witness in the 3,003 A/B sources): the elided operation is
refused after every head with no argument of its own and after its bare application (`reaches_a_following_ellipsis`):
a trig function, OPFUNCTION or operator (57cj.13), and a big, limit-type or differential operator, bare or scripted
(`is_bare_big_operator`: SUMOP, INTOP, BIGOP, LIMITOP, DIFFOP) — `a+\sum\cdots` a + ∑@(⋯), `a+\sum_n\cdots`,
`x_1+\sum\cdots+x_n`, `\sin\cdots+\det\cdots` sin@(⋯) + det@(⋯), `a+\int_0^1\cdots`, `a+\oint\cdots`, `a+\prod\cdots`,
`a+\bigcup\cdots`, `a+\lim\cdots`, `a-\inf\cdots`, `a+\sup\cdots`, `a+\partial_x\cdots`, as Perl (57cj.13 read a + ∑ + ⋯).
After a head's bare application the ellipsis reads as it does with nothing before it (`ends_in_a_bare_ellipsis` no longer
refuses that product): it goes on the argument — `a+\cos\cdots\cdots` a + cos@(⋯·⋯), `a+\det A\cdots` a + det@(A·⋯),
`a+\sum_n a_n\cdots` a + ∑_n(a_n·⋯), as Perl — or, where a trailing ellipsis leaves the bare argument (user ruling
2026-09-29), multiplies the application: `a+\sin x\cdots` a + sin@(x)·⋯, `a+\log x\cdots`, `a+\nabla u\cdots`,
`a\cdot\log x\cdots` (a·log@(x))·⋯ (Perl a + sin@(x·⋯)); never a + sin x + ⋯, whose `+` the source does not have. A run of
ellipses opening a trig argument is one whatever the macros (`is_an_ellipsis_run` takes an ellipsis ID):
`\sin\ldots\cdots x` sin@(…·⋯·x) as `\sin\cdots\ldots x`, `\cos\ldots\cdots` cos@(…·⋯), `a+\cos\ldots\cdots`, as Perl
(57cj.13 read sin@(…)·⋯·x); across a space it ends (`\sin\ldots\,\cdots x` sin@(…)·⋯·x). Residual (SYNC (9)): a closed
factor keeps the elision (`a+b\cdots`, `a+\sin(x)\cdots`, `a+\Delta\cdots` a + … + ⋯); a limit-type operator's operand
that opens with a group is not one — it takes what follows, `a+\det(A)\cdots` a + det@((A)·⋯), as Perl and as
`\det(A)\cdots`, `\det(A)B` det@(A·B) alone (a LIMITOP is a big operator, Perl `bigop`, MathGrammar:717), where
`a+\gcd(a,b)\cdots`, `a+\max(a)\cdots`, `a+\log(x)\cdots` (OPFUNCTIONs) keep the elision.

57cj.15 (the 57cj.14 review; latent, its probes; no corpus witness in the 3,003 A/B sources): 57cj.14's grammar derived
a run with one leading ellipsis ID only (`trig_ellipses += ellipsis_id elideop`) while `is_an_ellipsis_run` accepted any
mix, so the refusal of sin@(run)·⋯ left runs with a later ID before an ELIDEOP no tree — `\cos\ldots\ldots\cdots`,
`\sin\ldots\ldots\cdots x`, `\sin\cdots\ldots\cdots x`, `a+\cos\ldots\ldots\cdots`, `\sin^2\ldots\ldots\cdots x`,
`\sin\dots\cdots\ldots\cdots x` were unparsed (57cj.13 read sin@(…·…)·⋯·x). `trig_ellipses` now derives every run that
ends in an ELIDEOP (`trig_ellipsis_ids`: a run ending in an ID, which an ELIDEOP continues), the IDs after its last
ELIDEOP going on as `trig_arg`'s bare items — one derivation per run: `\sin\ldots\ldots\cdots x` sin@(…·…·⋯·x),
`\cos\ldots\ldots\cdots` cos@(…·…·⋯), `\sin\cdots\ldots\cdots\ldots x` sin@(⋯·…·⋯·…·x), `\sin\cos\ldots\ldots\cdots x`, as
Perl. An OPFUNCTION's or operator's bare argument takes a run between two items likewise (#376), joined by MulOps
too, and so does an OPFUNCTION's chain inside a trig argument (`\sin\log x\cdots y` sin@(log@(x·⋯·y)), 57cj.16). A run
after an addop or a MulOp is one unit, a product of its ellipses, as here (57cj.16, `semantics::ends_an_elided_run`):
the elided operation (`infix_apply_and_elide`) takes one ellipsis, never a run's last, so `a+x\cdots\cdots+c` is
a + x·⋯·⋯ + c, `a+\cdots\cdots\cdots+c` a + ⋯·⋯·⋯ + c, `a+x\ldots\cdots+c` a + x·…·⋯ + c and `a+\sin x\cdots\cdots+c`
a + sin@(x)·⋯·⋯ + c, as each reads alone (the first two had no parse, refused both as a bare ellipsis and as the elided
addition; the last two read a + x·… + ⋯ + c, a + sin@(x)·⋯ + ⋯ + c; latent, no corpus witness).

57cj.17 (the 57cj.16 review; latent, no corpus witness in the 3,003 A/B sources): an ELIDEOP run between two items stays
inside a trig argument, as an ellipsis ID does and as in an OPFUNCTION's (`trig_elided`, `trig_elidable_arg`,
`semantics::trig_argument_elision`; the twin refused in `semantics::leaves_a_bare_argument`): `\sin x\cdots y` sin@(x·⋯·y),
as `\sin x\ldots y` sin@(x·…·y) — 57cj.16 split it, sin@(x)·⋯·y, so the reading hung on the macro (`\sin x\dotsb y` stayed
inside, an ID) and on the head (`\log x\cdots y` and `\sin\log x\cdots y` kept the run inside). Whatever the macros, joins and
head, as Perl: `\sin^2 x\cdots y`, `\sin a_i\cdots y`, `\cos_a x\cdots y`, `\sin x\cdot\cdots\cdot y`, `\sin x\times\cdots\times y`,
`\sin x\cdots\cdots y`, `\sin x\ldots\cdots y`, `\sin x\cdots y\cdots z`, `\sin\cdots x\cdots y`, `\sin x!\cdots y`, and a run
opening the argument joined by a MulOp (`\sin\cdots\cdot\cdots\cdot y` sin@(⋯·⋯·y), was sin@(⋯)·⋯·y); through a composition,
a differential operator and an OPFUNCTION's argument (`\sin\cos x\cdots y` sin@(cos@(x·⋯·y)), `\partial\sin x\cdots y`,
`\log\sin x\cdots y` log@(sin@(x·⋯·y)), `\log\sin x\cdot\cdots\cdot\cdots\cdot y`). What ends a trig argument ends it here: a
space before or after the run (`\sin x\,\cdots y`, `\sin x\cdots\,y`), a `d`, a symbol of another type; a trig function's
application after the run is a new factor (`\cos\theta_1\cdots\cos\theta_n`, divergence #3), a trailing ELIDEOP run leaves
(`\sin x\cdots`, `\sin x\cdots+b` sin@(x)·⋯) as user ruling 15 (2026-09-29) has it — "a trailing ellipsis leaves any bare
argument … an ellipsis stays inside only between two items" — while a trailing ellipsis ID stays (`\sin x\ldots` sin@(x·…), as
Perl), which departs from that ruling pending the user's answer to Q10 (does ruling 15 cover trig arguments, reversing Perl's
sin@(x·…)?; SYNC_STATUS "Math-parse residuals of the 57cj train" (10)), so an ID after the run is an item
(`\sin x\cdots\ldots` sin@(x·⋯·…), as `\sin x\ldots` sin@(x·…)), and an argument ending in an OPFUNCTION's bare application leaves the run to that chain (no
`trig_elidable_arg`: `\sin\log x\cdots y` sin@(log@(x·⋯·y)), `\sin\log x\cdots\dots` sin@(log@(x))·⋯·…, its trailing run);
an application to a group closes and the run goes on (`\sin\log(x)\cdots y` sin@(log(x)·⋯·y)), and so does an OPFUNCTION before
a big operator, whose operand takes it (no twin refused: `\sin\log\det A\cdots y` keeps sin@(log@(det A))·⋯·y, SYNC (13)). A
run holding an ellipsis ID before an ELIDEOP is the whole argument only at its end (`trig_mixed_ellipses`: `\cos\ldots\cdots`
cos@(…·⋯)); an item after it goes on through `trig_elided`, so no ID before an ELIDEOP has a second derivation
(`\sin\ldots\cdots\ldots\cdots\ldots\cdots\ldots\cdots x` 9 trees, as 57cj.16). Two refusals that saw no
twin now do, so one reading survives at every number of sites (SYNC_STATUS "Math-parse residuals of the 57cj train"
(10)): a trig function ending an OPFUNCTION's or operator's argument keeps its argument's chain where the OPFUNCTION's
does not go on (`semantics::trig_application_ending`, read by `leaves_a_trig_bare_argument`): `\log\sin x\ldots\ldots`
log@(sin@(x·…·…)), `\log\sin\cdots\cdots\cdots` log@(sin@(⋯·⋯·⋯)), `\exp\sin x\ldots\ldots`, `\log\cos x\ldots\ldots`, as Perl
(log@(sin@(x))·…·… alone, Perl's at a second site); and a MulOp continues a trig argument (`semantics::trig_argument_across_a_mulop`,
`trig_arg mulop factor_base`): `\sin x\cdot y` sin@(x·y) as before but one tree, `\sin\log\ldots\ldots\ldots\cdot c`
sin@(log@((…·…·…)·c)), `\sin\log a_i\cdot\ldots\cdot\cdots\cdot\ldots` read as `\sin\log x\cdot\ldots\cdot\cdots\cdot\ldots`
(the pragmas chose between two trees by the letters). The ellipsis grid (`~/data/pk_agents/math/grids/ellipsis_grid/`,
77,250 formulas): 0 lost against 57cj.13-57cj.16, 3,263 readings changed to Perl's, 160 changed otherwise — each the
reading of a shorter form (`\sin\log x\ldots\cdots\ldots` sin@(log@(x)·…·⋯·…) as `\sin\log x\ldots` sin@(log@(x)·…), `\log\sin
x\ldots\cdots` log@(sin@(x·…))·⋯ as `\sin x\ldots\cdots`); formulas with more than one surviving reading 14,598 → 5,570, every
`\log\sin`/`\sin\log` run-length and context inconsistency gone (`byhead.py`).

57cj.18 (the 57cj.17 review; latent, no corpus witness in the 3,003 A/B sources): a run a MulOp or BinOp opens before a
juxtaposed item is the run between two items too, so it stays inside the bare argument — trig function, OPFUNCTION and
operator alike: `\sin x\cdot\cdots y` sin@((x·⋯)·y), `\sin\ldots\cdot\cdots x`, `\cos\dots\times\cdots y`, `\sin x\ldots\cdot\cdots y`,
`\log\sin x\ldots\cdot\cdots y`, `\sin\log x\cdot\cdots y`, `\log x\cdot\cdots y` log@((x·⋯)·y), `\nabla u\cdot\cdots v`,
`\max_i a_i\times\cdots b`, `\sin x\cdot\cdots\ldots` sin@((x·⋯)·…), as Perl. A MulOp's right operand is a juxtaposed product
(`term mulop tight_term`), so the twin (sin@(x)·⋯)·y was built by `infix_apply_nary` with `⋯ y` as one operand, a split
`leaves_a_bare_argument` never saw: both readings survived, 2^n at n sites (`\sin\ldots\cdot\cdots x` 256 at eight), the forest's
order choosing. `semantics::leaves_a_bare_argument` now asks its question (`a_run_stays_inside`) at every junction the
juxtaposed product would have — the split, and each one inside the right's opening run up to its first other factor. A
trailing run after a trig argument that ends in an OPFUNCTION's or operator's bare application leaves both bare
arguments, whatever its first macro (`semantics::leaves_a_trig_argument_s_chain`; user ruling 15, 2026-09-29: "a trailing
ellipsis leaves any bare argument … an ellipsis stays inside only between two items"): `\sin\log x\ldots` sin@(log@(x))·…, as
`\sin\log x\cdots` sin@(log@(x))·⋯ and `\log x\ldots` log@(x)·…; `\sin\log x\ldots\cdots\ldots`, `\sin\log x\ldots\ldots`,
`\sin\exp x\ldots\cdots`, `\cos\log_2 x\ldots`, `\sin\max_i a_i\ldots\cdots`, `\sin\log x\cdot\ldots`, `\sin\log\ldots\ldots`
sin@(log@(…))·… as `\log\ldots\ldots` log@(…)·…, `\sin\cos\log x\ldots` sin@(cos@(log@(x)))·… (57cj.17 kept an ID-led run in
the trig argument, sin@(log@(x)·…·⋯·…), while a `\cdots`-led one left; Perl keeps both in the OPFUNCTION's, sin@(log@(x·…))). The
trig actions ask it of the same pair — `trig_argument_juxtaposition` and `trig_argument_across_an_operator` (`trig_arg mulop
factor_base`) refuse to join, `leaves_a_trig_bare_argument` and `trig_argument_across_a_mulop` refuse no stop. Between two
items the chain still takes the run (`\sin\log x\ldots y` sin@(log@(x·…·y))); after a big operator its operand takes it
(`\sin\log\det A\ldots`, SYNC (13), Q9) and after an application to a group the trig argument's trailing-run reading holds
(`\sin\log(x)\ldots` as `\sin x\ldots`, pending Q10). This change already applies a "yes" to Q10 (does ruling 15 cover
trig arguments?) for a trig argument ending in an OPFUNCTION's or operator's chain: a "no", which keeps `\sin x\ldots`
sin@(x·…), reverts `\sin\log x\ldots` to 57cj.17's sin@(log@(x)·…), and the 2,952 grid rows below with it. The ellipsis grid,
extended by 79,425 mixed-join rows (the joins before, inside and after the run differ; 156,675 formulas): 0 lost against 57cj.13-57cj.17, 1,076 readings changed to Perl's (the
forest-order picks of the twin), 2,952 changed otherwise — every one a trailing run after `\sin\log`/`\cos\log_2` reading by
ruling 15; no formula from one reading to more (`ambig.py`), rows with more than one reading 22,835 → 16,160, every one left a
big operator's (`\det`, `\sum_i`; SYNC (15)).

**Guard**: golden `tests/parse/rust_parse_additions.tex` ("Explicit space ends a trig function's bare argument",
"A symbol of another type ends a trig function's bare argument", its 57cj.7-review rows, "A trig function's argument
takes an OPFUNCTION's application", its 57cj.10- to 57cj.14-review rows); `tests/parse/ellipsis_products.tex` ("An
ellipsis after a function head", "An ellipsis after an operator head or a head's application", "An ellipsis in an
operator function's bare argument" 57cj.14-review rows, "A run of ellipses in a trig function's argument", "A run of
ellipses that ends a trig function's argument", "A run of ellipses after a trig function in an OPFUNCTION's argument", "A run a MulOp opens before a juxtaposed item", "A trailing run after an OPFUNCTION's chain in a trig function's argument"); `tests/parse/count_parses.tex` (57cj.7 to 57cj.17 reviews); `parse_tree_count_limits`
(`trig_composition`, `trig_argument_nested_opfunction_*`, `trig_argument_opened_by_an_ellipsis*`,
`ellipsis_after_a_trig_head_in_a_sum`).

### 368. A nested group inside a Dirac label keeps its own bars

Perl's `ketExpression` and `maybeBra` set `$forbidVertBar` with `local` (MathGrammar:373, :401), a dynamic scope that
nothing resets, so inside a bra's, ket's or bracket's label no bar reads as a ket, a bar pair, an evaluation bar or a
conditional — not even inside parentheses: `\langle a|(1-|u\rangle\langle u|)|b\rangle` reads `bra@(a) * (1 - ket@(u)
* bra@(u)) * ket@(b)`, the projector's bars ending the bracket.

**Rust** (57bp; `is_forbidden_dirac_label`, `holds_ket`, `holds_open_bra`, `holds_bar_reading`, semantics.rs): the
label check stops at a nested group — a fence or Dirac bracket pairs its own bars — so the parenthesized projector
stays inside the operator product: `quantum-operator-product@(a, 1 - ket@(u) * bra@(u), b)`, the matrix element the
author wrote (2605.05292, 2605.20326).

**Guard**: golden `tests/parse/bar_pairs.tex` ("A bra that a bracket closes is the bracket's").

### 369. Message text expands as `\write` expands it

Perl's `make_message` (latex_constructs.pool.ltxml:5586-5587) expands a `\GenericInfo`/`\GenericWarning`/`\GenericError`
text fully, so an `\unexpanded{…}` or `\the\toks` result is expanded again and a protected macro runs.

**Rust** (Gemini round 13; `make_generic_message`, base_utilities.rs): the text is expanded as `\edef` and
`\immediate\write` expand it (latex.ltx:8773-8799), so every `\Package*`/`\Class*`/`\@latex@*` message prints as
pdflatex's: `\unexpanded` text and `\the` results stay, a protected macro prints as its name (KPE #381).

**Guard**: `perfect_kernel_gemini::package_warning_keeps_unexpanded_text`.

### 370. `\hypertarget`/`\hyperdef` anchor their own text

Perl's `localized_anchor` (hyperref.sty.ltxml:238-258) wraps the first node an anchor may hold, walking from the insertion
point; mid-paragraph that is the running text before the target (KPE #382).

**Rust** (Gemini round 13; `anchor_own_text`, hyperref_sty.rs): an empty text is a bare anchor; where the insertion
point admits `ltx:anchor` and the text is all horizontal material, the anchor is inserted there holding the text, as
hyperref.sty:4834-4845 anchors `#3`; otherwise (a heading, `\hypertarget{n}{\section{T}}`, or a display in the text) Perl's
walk places it, and the display stays a block.

**Guard**: `perfect_kernel_gemini::hyperdef_anchor_holds_only_its_text` (with the display and heading controls).

### 371. A two-item paren pair is a vector unless a set is asked for

Perl's `Fence` (MathParser.pm:1390-1417) names a two-item comma pair between parentheses by `%enclose2` alone
(`'(@,@)' => 'open-interval'`, :1369), whatever surrounds it: `G=(V,E)`, `\xi=(\xi_1,\xi_2)`, a whole-formula
`(x,y)`, `a^{(i,j)}` and `\{(x_i,y_i)\}` all read `open-interval@(…)`.

**Rust** (57br, user ruling 2026-09-29; `rename_fenced_lists`, `operand_slots`, `paren_pair_meaning`,
semantics.rs): the pair is a `vector` — a pair, a point — unless the slot it fills holds a set: the right operand of
`∈`/`∉` (not a pair right of a pair: `(x,a)\in(\mathcal X,\mathcal A)` is componentwise, 2605.06977; `(t,x)\in(0,T)\times\Omega`
keeps its set slot), the left
of `∋`, either operand of a subset or superset relation (in a multirelation, by the relation beside it; a decorated
one by its base, `\in_{\mathcal A}`), and the operands of `×` (as written), `∪`, `∩`, `∖` (big or scripted,
`\bigcup_n`) or a power's base when that application fills such a slot — `x\in(0,1)`, `A\subset(0,1)`,
`x\in(0,1)^d`, `t\in(-\delta,\delta)\setminus\{0\}`, `x\in\bigcup_n(a_n,b_n)` read `open-interval`,
`(x,y)\in\mathbb R^2` reads `vector`. A function's argument pair is a `vector` (`u(x,\infty)`,
`\Pi(M^2,\infty)`, 2605.28015), unless it runs from −∞ to ∞ or to a number — the interval the function takes
(`S(-\infty,\infty)`, `f(-\infty,-3)`, 57bx; 2605.16086, 2605.01702; `u(-\infty,t)` stays a vector). Elsewhere an infinite endpoint on its own side makes an interval — `-\infty` (or
`\pm`/`\mp`) first, `\infty` (`+`, `\pm`, `\mp`) last: `(0,\infty)`, `C^1((0,\infty))`, `\mu((x,\infty))`, `B=(0,\infty)\times\mathbb R` — while
`(\infty,1)` (an (∞,1)-category, 2605.30648) and `(0,-\infty)` stay vectors. The name is given once, on the chosen parse,
reading relations and delimiters through XMRefs (Perl `p_getTokenMeaning(realizeXMNode(…))`, MathParser.pm:1090), so
it no longer depends on which derivation Marpa enumerated first (before: `G=(V,E)` vector but `\xi=(\xi_1,\xi_2)`
open-interval). Brackets, half-open pairs and `]a,b[` (#362) are unchanged. Accepted, read `vector`: a function's argument pair,
bare-letter function spaces among them (`C(0,\infty)`, `\mathcal L(0,\infty)`, `\mu(0,\infty)`; one from −∞ reads an
interval since 57bx, `C(-\infty,0)`, so `G(-\infty,0)=G(0,\infty)` names its two pairs differently; the number test
reads a single token, so `F(-\infty,\tfrac12)` stays a vector; a scripted or accented
head parses as a product, so `\Pi_1(M^2,\infty)` reads an interval); a set outside any relation and without an infinite endpoint — `\mu((a,b))`, a `∪` outside a relation, an integration
domain `\int_{(a,b)}` (2605.02556), an indicator's subscript `\mathbbm{1}_{(0,\delta_x)}` (2605.01709), a map's
codomain `\to(0,1)` (2605.01729) — a pair set equal to a pair `(p,q)=(1,\infty)`, and a pair in a gathered or split
row whose rows the parser joins by invisible times (the relation's operand is then a product). Witnesses: 2605.01424, 2605.00899, 2605.00335, 2605.03082, 2605.01633, 2605.00581.

**Guard**: golden `tests/parse/paren_pairs.tex`.

### 372. A float numbered by a phantom caption counts as captioned when floats collapse

Perl's `collapseFloat` (latex_constructs.pool.ltxml:3437-3464) merges a float holding exactly one inner float into it
unless both carry an `ltx:caption`, copying the inner's attributes over the outer's.

**Rust** (57bu; `collapse_float`, latex_constructs/mod.rs): a float numbered by `\phantomcaption`, a bare
`\subfloat{…}` or `\phantomsubcaption` — its own `ltx:tags`, no `ltx:caption` — is captioned in caption.sty's terms
(`\caption@refstepcounter`, caption.sty:392-395), so a numbered panel stays a panel with its number and its `\label`
(2605.28276's `\cref{fig:corridor}` dangled when the inner `labels` replaced the outer's). The outer float's tags
count beside an inner with tags of its own (a tagless inner — a `\caption` in a minipage or `\parbox`, which
`insert_block` makes a figure — holds the very caption whose counters the outer rescued, and collapses, as Perl;
57bu.2 A/B: 40 papers, 2605.01437 Fig. 1); an inner float's count only when they came from a sub-float counter (`after_float` records
`panel_number` for a `sub<type>` `\@captype`): a longtable's own step inside a table (2605.18937, ltablex) leaves the
inner float collapsible, as in Perl, and a caption-less inner float no longer takes the outer caption's counters
(KNOWN_PERL_ERRORS #395; 2605.26653): tagless, it collapses, and the outer `\label` stays on the table. The collapsed
float keeps the id of the element carrying its number, where Perl moves the inner id exactly when the inner has a
caption (collapseFloat :3449-3452; 57cp): an inner float numbered by its caption or by its tags alone — a caption-less
longtable's own step (`S1.tab1` → `S1.T1`), a tabularray tall table whose theme hides the caption — in an outer with
neither gives the float its id, and an outer holding the tags its minipage's caption stepped keeps its own (`S1.F1`,
where Perl gives `S1.F1.fig1`). A collapse also keeps both floats' labels, where Perl's attribute copy replaces the
outer's; it puts the inner content where the inner float stood, as Perl does (2605.04869, 2605.17547, 2605.20770,
2605.27546 read the panel's graphic after the figure caption), and an inner float that was one panel beside other
content leaves that content a panel (`ltx_figure_panel`): a tall table beside a tabular in a caption-less `table`, a
tabular beside its colorbar (2605.31204 A1.F2), a minipage beside a captioned one, stay side by side (57cp review 5).
The inner's caption stays where it stood, so a captioned *first* panel leaves its caption between the panels, which
the XSLT's figure grouping splits into rows (2605.01542 A3.F8, 2605.21713 A4.T15; unchanged by the panel class).

**Guards**: `perfect_kernel_gemini::{phantom_numbered_panel_keeps_its_place_and_labels,
phantom_numbered_outer_keeps_its_captioned_panel, collapsed_panel_content_stays_in_place,
inner_float_without_a_panel_number_collapses}`, `perfect_kernel_batch57::{captionless_float_takes_the_inner_number,
tabularray_tall_table_in_a_captionless_float}`.


### 373. A missing comma beside an ellipsis is supplied

Perl reads an item that starts (or ends) with a bare ellipsis as a product with it: `\{l_1,\cdots l_k\}`
`set@(l _ 1, cdots * l _ k)`, `k=0,1,\cdots L-1` `list@(k = 0, 1, cdots * L - 1)`, `\{30,40\ldots,80\}`
`set@(30, 40 * ldots, 80)` — the author's missing comma read literally.

**Rust** (57bv; user ruling 2026-09-29, split it off; `separate_ellipsis_items`, semantics.rs): in a comma container
(a list or formulae, a Fence-named set or pair, a head's arguments `\max(a_1,\ldots a_n)`), an item whose spine —
through unfenced invisible products, sums and relations — reaches an invisible product that starts with an ellipsis
gives it up as an item before it, and one ending in an ellipsis before a comma gives it up after it, with an
invisible comma (U+2063, Perl's `InvisibleComma` for a missing separator) between. Each reads as its comma'd twin
(`set@(l _ 1, cdots, l _ k)`, `k = list@(0, 1, cdots, L - 1)` once the enumeration attaches, a pair grown to three
items named as Fence names three). Not before a visible operator (`x_1,\cdots +x_n` elides a sum); a trailing
ellipsis only where one factor remains (`p_1p_2\cdots, q` elides the product) and never a decimal's digits
(`x=0.325\ldots`, `x, 0.3\ldots`). Residual: the distributed shape `x_1,\dots x_n\in\{0,1\}` (3 A/B
formulas; SYNC). Witnesses 2605.12837, 2605.12555, 2605.02211, 2605.00390, 2605.25695 (~288 formulas / 130 papers
of the 57bs A/B). **Guard**: `tests/parse/enumerations.tex` "A missing comma beside an ellipsis" (each row beside
its twin).

### 374. A differential operator takes one factor; a Leibniz quotient is one derivative

Perl's grammar lists DIFFOP among the big operators (`bigop : BIGOP | SUMOP | INTOP | LIMITOP | DIFFOP`,
MathGrammar:717), and `addOpArgs`/`moreOpArgFactors` (:605-618) take every factor after one, across a MulOp too:
`\partial F/\partial T` reads ∂@(F/∂@(T)), `\partial\Omega\times(0,T]` ∂@(Ω·(0,T]), `\nu\partial_x u\cdot\partial_x v`
ν·∂_x(u·∂_x v), `\partial^\rho G(x-y)c(y)` ∂^ρ@(G(x−y)·c(y)) (KNOWN_PERL_ERRORS #387).

**Rust** (57bx, then 57cj and 57cj.1; user ruling 2026-09-29: ∂ takes one factor, ∇ keeps its greedy bare argument): a
DIFFOP is no `bigop_application` head. It takes one factor (`diffop_application = diffop_head diffop_operand`, builder.rs;
`diffop_operand` is Perl's `Factor` list: a factor, an application, another differential operator's application, a
postfixed or interval term, a number with the factors juxtaposed after it — `numeric_monomial`, `\partial_x\frac12 u^2`
∂_x(½u²), `\partial_t 2\pi iu` — and an ellipsis before a differential operator's application, `\partial_i\ldots\partial_j u`
∂_i(…·∂_j u)), built by `differential_operator_apply` as a finished factor (`Meta::differential`, as `d x` already was):
`\partial_x u\cdot v` (∂_x u)·v, `g^{\mu\nu}\partial_\mu\varphi\partial_\nu\varphi` g·∂_μφ·∂_νφ, `\partial\Omega\times(0,T]` ∂Ω·(0,T],
`\partial^\rho G(x-y)c(y)` ∂^ρ(G(x−y))·c(y), `\partial_x\partial_y f(x)g(x)` ∂_x(∂_y(f(x)))·g(x) (`diffop_group_application`
nests), `\int\partial_x u\,dx` ∫(∂_x u·dx); `\partial\partial f`, `\partial\sin x\cdot y` keep their operand whole, and so does an ellipsis run between two
items in it (57cj.18a, the 57cj.18 review: `\partial\log x\ldots y` ∂(log(x·…·y)), `\partial\nabla u\cdots v` ∂(∇(u·⋯·v)),
`\partial\sin\log x\cdots y`, as Perl and as `\partial\log x\cdot y` ∂(log(x·y)); were ∂(log x)·…·y —
`semantics::a_run_stays_inside` now looks through the differential, as `leaves_a_bare_argument` does; a trailing run
still leaves, `\partial\log x\ldots` ∂(log x)·…; latent, no corpus witness). What no
factor may follow it takes at term level (`diffop_term`, a `bigop_operand`), so a factor-level operand cannot split it:
a big operator's application (`\partial_t\int_\Omega u\,dx` ∂_t(∫…); 57cj's factor-level route gave
`\partial_t\int u(y)v(y)w(y)\,dy` 42 trees, 8 now as before), a function before one (`\partial\log\sum_i x_i` ∂(log·∑…),
2605.30560), a bare function (`\partial\operatorname{\mu}`, 2605.29136), a bare operator (`\partial_t\nabla\cdot u` (∂_t∇)·u, as
`\nabla\cdot u` is ∇·u) and a bare differential operator, a chain (`\partial_x\partial_y` (∂_x)@(∂_y),
`\eta^{\alpha\beta}\partial_\alpha\partial_\beta`, `(\partial_t\partial_s-\partial_s\partial_t)\Phi=0`, `D^\alpha=\partial_1^{\alpha_1}\cdots
\partial_n^{\alpha_n}`; 2605.12948, 2605.21314, 2605.22252, 2605.26285, 2605.27600, 2605.28314, 2605.29990). A partial
derivative is the first item of an operator's, an OPFUNCTION's and a trig function's bare argument (Perl's `aBarearg`
has no DIFFOP): `\nabla\partial_x u` ∇@(∂_x u), `\sin\partial_x u` sin@(∂_x u), `\operatorname{dom}\partial\psi` dom@(∂ψ); a later
item ends it, `\nabla u\partial_x v` ∇@(u)·∂_x v, as Perl. An accented DIFFOP is one (util.rs lexes `\bar\partial` a
DIFFOP; Perl an ATOM): `\partial\bar\partial u=0` ∂(∂̄u) (2605.01526, 2605.15276, 2605.01646). A Leibniz slash quotient
is one derivative (`leibniz_quotient` in `infix_apply_nary`): the numerator is the last differential operator with the
factors after it, the denominator the leading differentials of the numerator's kind (∂ or d: `\int_0^1\partial^n
f/\partial x^n\,dx` keeps its `dx`) its order asks for (the numeric exponent on ∂, else 1; each denominator variable's
exponent; a symbolic order takes the run), and the rest multiplies the quotient — `T\,\partial F/\partial T` T·(∂F/∂T),
`h\,\partial f_A/\partial p_j` h·(∂f_A/∂p_j), `\partial^2 f/\partial x\partial y` ∂²f/(∂x·∂y), `\partial u/\partial x\,v`
(∂u/∂x)·v, `\partial\rho u/\partial t` ∂(ρu)/∂t; a `\frac` numerator over a differential reads the same way
(`regroup_leibniz_numerator`, parser.rs), and a `\frac` denominator under one gives each differential the factors up to
the next (`regroup_leibniz_denominator`: `\frac{\partial u_0}{\partial\Delta\psi}` ∂u_0/∂(Δψ), 2605.23203, 2605.28495,
2605.05995). A bare DIFFOP with nothing to take stays an operand (`\partial/\partial t`, `\partial_x+\partial_y`); a
pre-scripted one keeps Perl's greedy operand (no witness); `\nabla` is an OPERATOR and keeps its bare argument
(`\nabla u\cdot v` ∇@(u·v)). Witnesses: 2605.03741, 2605.08634, 2605.24774, 2605.00580, 2605.00105, 2605.29990, 2605.15405,
2605.21149, 2605.00581 (`\partial_{11}l(F(x),Y)f(x)`), and 57bx's 2605.13790, 2605.22408, 2605.24151
(`\partial\Omega\times(0,T)`), 2605.28300; the 57bw A/B XML applied a ∂ to a greedy slash quotient in 370 formulas / 123
papers, a product of derivatives in 807 / 90, a MulOp chain in 42 / 24, another product in 916 / 94. Residuals: a
scripted letter's group (`\partial_t\rho_t(\bm x)` (∂_tρ_t)·x, ~775 formulas) follows #18's scripted-letter reading; a
braced compound (`\partial_z{\delta B_y}` ∂_z(δ)·B_y, 2605.22729) and an order mismatch in a slash quotient
(`\partial f/\partial x\partial y` (∂f/∂x)·∂y) stay; a letter constant is no number (`\partial_x\pi r^2` ∂_x(π)·r²,
`\partial_\theta\frac1N\sum_n\ell_n` ∂_θ(1/N)·∑…, 57cj.1 review); iopart's `\rmd^n f/\rmd x^n\,\rmd x`
takes the integral's `\rmd x` (the d kind matches); a `\frac` denominator's constant joins the variable
(`\frac{\partial^2\phi}{\partial t^2c^2}` ∂(t²c²)); `\partial_t\nabla\cdot u` is (∂_t∇)·u, where Perl and 57ci read
∂_t(∇·u) (open, a ruling: equal as operators, but the tree differentiates the ∇ symbol). 57cj.1's numeric operands
(`numeric_monomial`, a fraction of numbers lexed `ATOM_NUMBER`, util.rs), the operand before a big operator
(`numeric_big_operand`, before a function before one too) and the unsubscripted ∂ a `\frac` denominator regroups
were the 57cj.1 and 57cj.2 reviews' (57cj.2, 57cj.3; latent, the reviews' probes); a derivative right after the number
joins it (`\partial_x 2\,\partial_y u` ∂_x(2·∂_y u), as Perl).
Residuals: a factor after an integral's differential stays in the integrand, as Perl (`\partial_t\int_0^x u(y)\,dy\,v`
∫(u·dy·v)); an accented letter is never applied (`\partial\tilde s(u,j)` ∂(s̃)·(u,j); Perl ∂(s̃·(u,j)), s̃ unapplied as well); `\partial_x 2\frac12 u`
reads (∂_x(2+½))·u through the mixed-number rule; `\nabla\partial_x 2\sum_i a_i` ∇@(∂_x 2)·∑; `\partial_x 2\nabla\sum_i a_i`
reads (∂_x 2)·∇·∑ (Perl ∂_x(2·∇·∑); no numeric route before an operator before a big operator);
`\mathbb{E}\partial_t\sum_i X_i` E·∂_t(∑…). OPEN, pending a ruling (57cj.3 review): 𝔼 before a derivative is a letter
since 57cj.3 (`\mathbb{E}\partial_t u\,v` E·∂_t u·v, `\nabla\mathbb{E}\partial_t u` ∇@(E)·∂_t u, as Perl); typed (57cj.2), an
OPFUNCTION's bare argument ended at the first derivative, so the Fisher information `\mathbb{E}\partial_\theta\log p\,
\partial_\theta\log p^\top` read 𝔼@(∂_θ log p)·∂_θ…; the question is whether an expectation's bare argument runs on over
later derivatives. Further residuals: before a big operator a number's operand is a term no bare argument takes, so
`\nabla\partial_x 2\sum_i a_i` is ∇@(∂_x 2)·∑… and `\sin\partial_x 2\sum_i a_i` sin·∂_x(2·∑…) (the function before a big
operator a factor of its own, as `\sin\sum_i a_i` sin·∑…); physics' braced `\dd{x}` lexes one ATOM a monomial takes
(`\int\partial_x 2uv\,\dd{x}` ∫(∂_x(2uv·dx))); `\partial_x 2\pi\,\partial_y u` is (∂_x(2π))·∂_y u (Perl ∂_x(2π·∂_y u)). The
differentiated number's preference is a soft pragma (`DifferentiatedNumbersTakeTheirFactors`, 57cj.4), counting its
sites (57cj.5): a refusal in `apply_invisible_times` (57cj.1–57cj.3) killed the last parse where the monomial cannot hold
what follows (`\partial_t 2\,\partial_x u\,\nabla\cdot v`, 57cj.3 review), and a pass-or-fail pragma let that one site make
every other split (`\partial_t 2\,\partial_x u\,\nabla\cdot v+\partial_y 3w` read (∂_y 3)·w, 57cj.4 review). A bare operator or
function after the monomial is none it takes, so no violation (57cj.6): `\partial_x 2u\,\nabla\cdot v` (∂_x(2u)·∇)·v,
`\partial_t 2\,\partial_x u\,\nabla\cdot\mathbf{v}=0` (∂_t(2·∂_x u)·∇)·v = 0 — counted as one, the two readings tied and the
tie-break chose by what stood before (57cj.5 review: the split after `\sin x`, `\log x`, `\nabla f`, the monomial after
`a`); and every differentiated number on the left factor's right edge counts, through a differential operator's
operand and a function's or operator's bare argument (`right_edge`): `\partial_x\partial_y 2u` ∂_x(∂_y(2u)),
`\partial_x 2\,\partial_y 3w\,\nabla` ∂_x(2·∂_y(3w))·∇, `\sin\partial_x 2\,\partial_y u` sin@(∂_x(2·∂_y u)),
`\nabla\partial_x 2\,\partial_y u` ∇@(∂_x(2·∂_y u)). A derivative in a bare argument ends where the argument ends (57cj.7;
57cj.6 review): its numeric monomial crosses no end the enclosing argument obeys — in a trig function's, explicit
space, a `d`, a type mark (`ends_trig_argument`, #367) or a function or operator after it (juxtaposed trig functions are
separate factors), in an OPFUNCTION's or operator's an OPFUNCTION or operator, unless a bound head's variable is
mentioned (`trig_derivative_item`, `operator_bare_apply`, `crosses_a_bare_argument_end`): `\sin\partial_x 2u\,v`
sin@(∂_x(2u))·v as `\sin\partial_x u\,v` sin@(∂_x u)·v, `\sin\partial_t 2\pi u\,v`, `\sin\partial_x 2u\mathbf v`, `\sin\partial_x 2u\cos v`,
`\log\partial_x 2u\log v` log@(∂_x(2u))·log v, `\nabla\partial_x 2u\,\log v`, `\max_i\partial_x 2u_i\log v` (but
`\max_i\partial_x 2u_i\log v_i` takes it); a function's or operator's bare argument in it crosses none of a trig
argument's own ends either, and ends where its own head ends it (57cj.8; 57cj.7 review): `\sin\partial_x\log u\,v`
sin@(∂_x(log u))·v, `\sin\partial_x 2\log u\,v`, `\sin\partial_x\log u\,v\,w` (one parse each, where
sin@(∂_x(log(u·v))) survived to the pragma, and a four-term sum held 16) — a mention of the variable a bound head binds
continuing it across the space (57cj.9; 57cj.8 review: `\sin\partial_x\max_i u_i\,v_i` sin@(∂_x(max_i(u_i·v_i))), the head's
scope following its variable, user ruling 2026-09-30, before #367's space). The principle: no end where everything under
the derivative before it is constant ends anything — ending there would differentiate a constant — so
`\sin\partial_x 2\,\partial_y u` is sin@(∂_x(2·∂_y u)), `\sin\partial_x 2\,u\,v` sin@(∂_x(2u))·v, and (57cj.8; 57cj.7 review)
`\sin\partial_x 2\pi\mathbf v` sin@(∂_x(2πv)), `\sin\partial_x 2\pi\,u\,v` sin@(∂_x(2πu))·v, `\log\partial_x 2\pi\log v`
log@(∂_x(2π·log v)), `\nabla\partial_x 2\pi\,\log v`, `\partial_x 2\pi\,\partial_y u` ∂_x(2π·∂_y u); judged down the whole path,
through a function's argument too (57cj.9; 57cj.8 review): `\sin\partial_x\log 2\,u\,v` sin@(∂_x(log(2u)))·v,
`\sin\partial_x 2\log 2\pi\,u\,v` sin@(∂_x(2·log(2πu)))·v, `\sin\partial_x\log\sqrt2\,u\,v`. A constant (`is_constant`, one notion
with the lexer's `is_numeric_constant`, 57cj.9) is a number; π unscripted or raised to a constant power (`\pi^2`, not
`\pi^a` or `\pi_a`: a field or a policy — `\partial_\mu 2\pi^a\partial^\mu\pi^a` (∂_μ(2πᵃ))·∂^μπᵃ, a regression of 57cj.8,
`\sin\partial_x 2\pi_a\,u\,v` sin@(∂_x(2π_a))·u·v); a sum, quotient, product, constant power, root or nth root of
constants; the imaginary unit `i` (`\mathrm{i}`, `\imath`) right after a constant in a product (57cj.9; 57cj.8 review: 25 of
25 sampled `2\pi i` in the delta A/B's 373 are the unit — `\sin\partial_t 2\pi i\,u\,v` sin@(∂_t(2πiu))·v, `\partial_x\frac{1}{2\pi
i}u` ∂_x(1/(2πi)·u); a leading `i` stays a letter, `\sin\partial_x i\,u\,v` sin@(∂_x i)·u·v); a group holding one; a function
applied to one (`\log 2`). A derivative of a bare π or π-power counts as no differentiated number (`differentiates_a_number`):
an integral's measure `\int f\,d\pi(x,y)` keeps dπ (2605.00545, 2605.28589; a first cut of 57cj.9 read d·π·(x,y) in 65
A/B formulas). The lexer reads a constant atom as the number `\frac12` is (`ATOM_NUMBER`: `\sqrt2`, `\frac{\pi}{2}`,
`\frac{\pi^2}{6}`, `\sqrt[3]{2}`, `\frac{1}{2\pi i}`; a Dual by its presentation, a reference through the idstore only;
`\frac{1}{\text{n}}` is none), and a number raised to a constant power leads a monomial (`numeric_power`: `\partial_x 2^3u`
∂_x(2³u), `\partial_x\sqrt2^3u`), so `\partial_x\sqrt2 u` is ∂_x(√2·u), `\partial_x\sqrt[3]{2}u` ∂_x(∛2·u) (were ∂_x(√2)·u). The count
reads into a function's or operator's bare argument only for a differentiated constant (an argument that has ended takes
nothing more). In a trig argument a derivative of a numeric constant — a number, a constant monomial a number leads, a
function's bare application to a constant — takes what follows it: neither the chain nor the product goes on after it
(`ends_in_a_differentiated_constant`, 57cj.9; 57cj.8 review NIT 9: `\sin\partial_t 2\pi\,f t\,v`, `\sin\partial_x 2\pi\,u\,v` one
parse, where 2–4 survived to the pragma). A bare π leads no monomial — `\partial_x\pi u` stays ∂_x(π)·u, ∂ taking one factor:
π is also a field (`\partial_\mu\pi^a\partial^\mu\pi^a`; all 77 ∂π formulas of the 9,420 corpus ∂ formulas are a field or a
policy, 57cj.8 review; Perl ∂_x(π·u)). A letter's application to a group across explicit space is no operand of a
derivative, nor a factor of its monomial (57cj.9; 57cj.8 review, latent: `\partial_x u\,(1-x)` (∂_x u)·(1−x),
`\sin\partial_x 2u\,(1-x)` sin@(∂_x(2u))·(1−x), `\partial_z\rho\,(\tfrac12|\nabla\theta|^2)` (∂_z ρ)·(½|∇θ|²) — 6 formulas in the
delta A/B, 2605.24151 and 2605.18945's measure `\differential^4\mathsf y\ (1-\dots)`; `\partial_x u(1-x)` stays ∂_x(u(1−x)), #18; outside a derivative's operand and a trig argument the
letter still applies, `k\,(x-y)` k@(x−y), a ruling pending). A trig argument's trailing space is its last token's (#367).
Residuals: `u\,\nabla\cdot v` groups as (u·∇)·v, where ∂_x(2u)·(∇·v) is intended (Perl ∂_x((2·u·∇)·v) and
∂_t(2·∂_x((u·∇)·v)) = 0, its greedy `bigop`); an argument list keeps a letter's application across the space
(57cj.10: `\partial_t u\,(x,t)` ∂_t(u(x,t)); 2605.24758) — a parenthesized list holding a variable (57cj.11, 57cj.12:
constants beside it an evaluation point, `\partial_x u\,(0,t)=0` ∂_x(u(0,t))=0, `\partial_q L\,(q,\dot q,t)`; 57cj.13, the
57cj.12 review: ∞ and a signed one, `\partial_x u\,(x,\infty)` (was (∂_x u)·open-interval), a lone `\cdot` or `\bullet` a
variable slot, the standard `\partial_x u\,(\cdot,t)`, a number approached from one side `(0^+,t)`, an accent over a
scripted letter `\partial_t f\,(\hat{x_1},t)` (`\hat{x^2}` a power); an unscripted e or i is a constant as π is, so
`\partial_t f\,(i,0)` is a vector and `(x,i)`, `(i,j)` hold a variable; 57cj.14, the 57cj.13 review: a subscripted or
primed e or i is a letter, an index or a basis vector, `\partial_t a\,(i_1,\ldots,i_k)` ∂_t(a(i_1,…,i_k)), `(i',t)`,
`(e_1,t)`, `(\mathbf e_1,t)` apply (57cj.13 read them vectors); raised to any other power it is the exponential or a power
of the imaginary unit, no variable (57cj.15, the 57cj.14 review, which admitted any script): `(e^{x},t)`, `(e^{i\theta},t)`,
`(\mathrm{e}^{x},t)`, `(i^n,t)`, `(i^j,t)`, `(e^2,t)` are vectors, as Perl's open intervals — the whole script chain
deciding, not its outer layer, so script orders TeX renders alike read alike (57cj.16, the 57cj.15 review): a letter if
any layer is a subscript or a script opening with a prime and no layer a constant power, `(e^x_k,t)` as `(e_k^x,t)`,
`(e'^x,t)` (TeX's one superscript `e^{\prime x}`) as `({e'}^x,t)` apply, `(e^2_k,t)`, `(e_1^2,t)` are vectors — and a prime
raised to a constant is that power (57cj.17, the 57cj.16 review; `semantics::raises_a_prime_to_a_constant`: `e'^2` is TeX's
`e^{\prime 2}`, (e′)², which the constant test could not see through the prime): `(e'^2,t)`, `(x'^2,t)`, `(e'^{-1},t)`,
`(i'^2,t)` are vectors as `(e_1^2,t)` and `(x^2,y)` are (57cj.16 applied them; Perl multiplies, f·open-interval), `(e'^x,t)`,
`(x'^n,t)` apply; latent, no primed-power tuple in the 3,003 A/B sources; an accent over a power takes a bare power's constant test,
`\hat{x^{-1}}`, `\hat{x^{1/2}}`, `\hat{x^\pi}` powers as `x^{-1}` is, `\hat{x^n}` a variable; the lone slot is `\cdot`,
`\bullet` or the wildcard `\ast` (`\Delta(\ast,\ast)`, 2605.02499, 2605.18079 — the only operator alone in a tuple in the
3,003 A/B sources, 52 hits), so `(\otimes,t)`, `(\star,t)`, `(\times,t)` are vectors as Perl reads every one — Perl's
`AnyOp` takes any operator alone before a PUNCT or CLOSE, MathGrammar:204-206, and singles none out as a slot); constants alone or a bracketed pair multiply: `\partial_x u\,(1,0)`
(∂_x u)·(1,0), `\partial_t u\,[a,b]` (∂_t u)·[a,b] — Perl multiplies inside the operand, ∂_x(u·(1,0)), and the one-factor
rule moves the tuple out. Intended but not done (latent, no corpus witness): a signed or scaled letter, an expression
item and a semicolon parameter list are evaluation points too — `\partial_x u\,(-x,t)` ∂_x(u(−x,t)), `\partial_x u\,(2x,t)`
∂_x(u(2x,t)), `\partial_x u\,(e^x,t)` (an application while e was a letter, 57cj.12), `\partial_x u\,(x,t;\lambda)`
∂_x(u(x,t;λ)) — and read (∂_x u)·(…) today. The rule is the derivative operand's: in a trig
argument a tuple after a space is a vector (#367, 57cj.13); a constant group a number leads, raised to a constant power, leads a
monomial (`numeric_group`: any scripted group there added a refused tree, two new `ambiguous_math` warnings in the corpus
∂ set), and a group's delimiters are no factor of a constant atom (57cj.10: `\partial_x(2\pi)^{-3}u` ∂_x((2π)⁻³·u),
`\partial_x\frac{1}{(2\pi)^3}u`); a function of a constant under ∂ before an OPFUNCTION or operator,
`\partial_x\log 2\log v` (∂_x log 2)·log v — ∂ takes one factor and the function's argument ends at a later OPFUNCTION, so no
reading takes both (the grid's 136 constant readings, `grid9/`); the reviewers' formulas `\sin\partial_x 2\log 2\pi\,u\,v`,
`\sin\partial_t 2\pi i\,u\,v`, `\sin\partial_t 2\pi\,f t\,v` read as intended but enumerate over 10 derivations (an
`ambiguous_math` warning).
Each differentiated number's two readings reach the root on the tree route, 2^n trees for n sites (six in a sum,
64; latent, the review's probe; a per-glade preference would drop the split a parent needs, so none); per-site
counting is unobservable in today's grammar (every forced site the reviews probed has a 0-violation reading) and is kept
for correctness. A `d` outside
an integral: `\partial_x 2u\,dx` ∂_x(2u·d·x) (as Perl) while `\partial_x u\,dx` is (∂_x u)·d·x. A `\frac` denominator holding a
subscripted ∂ followed by a non-∂ is a derivative, neither argument regrouped (57cj.6: `\frac{\partial fg}{\partial_x\partial_y u}`,
`\frac{\partial fg}{\partial x\,\partial_y u}`); a subscripted ∂ before another ∂ or at the end names no operand
(`\frac{\partial^2 fg}{\partial_x\partial_y}`, `\frac{\partial fg}{\partial_x\partial y}` regroup ∂(fg)). **Guards**: `tests/parse/integrals_and_differentials.tex` ("A
differential operator takes one factor", "A Leibniz quotient is one derivative", "A differential operator's term
operands", "A differential operator's numeric and fenced operands"), `tests/parse/ellipsis_products.tex` ("A run of
ellipses in the bare argument a differential operator takes"), `tests/parse/bigop_operands.tex`, `parse_tree_count_limits` ∂ rows.

### 375. A collapsed float takes no box geometry from a panel that stood beside other content; classes merge

Perl's `collapseFloat` (latex_constructs.pool.ltxml:3437-3464) merges a float holding exactly one inner float, when they
do not both have captions, by copying every attribute of the inner onto the outer (:3447-3449, `setAttribute`
overwrites) and splicing the inner's children in its place (KNOWN_PERL_ERRORS #388). When the inner float was a box — a
caption minipage — beside another panel, its `width`, `vattach` and `ltx_minipage`/`ltx_figure_panel` classes
describe that one panel, yet land on the whole float: a side caption's `0.3\linewidth` minipage gives its figure
`width="103.5pt"` around a wider panel, a subfigure of a caption minipage and a panel claims the caption's width, and the
parent's panel rows then set several `\linewidth` subfigures on one row.

**Rust** (57by; `collapse_float`, latex_constructs/mod.rs): when the outer holds content other than the inner float and
what `arrange_panels` counts as no panel (`is_panel_break_name`: tags, captions, breaks; notes, index marks), the
collapse copies no `width`, `height`, `depth` or `vattach` and none of the inner's `ltx_minipage`, `ltx_parbox`,
`ltx_figure_panel` classes. In both cases the class merges with the outer's own (`add_class`), as a wrapper box's does
(#125, KNOWN_PERL_ERRORS #92): a lone-minipage `algorithm` keeps `ltx_float_algorithm` with the minipage's geometry, where
Perl overwrote it with `ltx_minipage`. Labels, ids and captions merge as before. The parent's panel rows see the
collapsed float's own width, so three `0.4\linewidth` subfigures set two to a row, as pdflatex. Witnesses: 2605.03502
S3.F3 (103.5pt side caption), 2605.15932 S4.F3 (17.3pt subfigures, now a row each), 2605.17037 (algorithm class); 59
papers of the 57by delta A/B, every tally identical. Residual: a `\rule{\linewidth}{…}` panel's `width="100%"` is not
measured by the panel rows, so 2605.18774's rule moves between rows either way. **Guard**:
`perfect_kernel_batch57::collapsed_panel_keeps_the_float_geometry` (repro
`captions-floats/collapsed_panel_keeps_the_float_geometry`), `perfect_kernel_gemini::collapsed_panel_content_stays_in_place`.

### 376. Juxtaposed operator functions are separate factors; `\mathbb{E}` and `\mathbb{P}` are operator functions

Perl's bare argument (`barearg`, MathGrammar:553-558, for an OPFUNCTION through `addOpFunArgs`, and for an operator)
is greedy: it runs to the next relation or punctuation and takes every OPFUNCTION it meets with it, so
`\log x\log y\log z` is log@(x·log@(y·log@(z))), `\max_i a_i\log b_i` max_i@(a_i·log@(b_i)) and `\nabla x\log y`
∇@(x·log@(y)). A second function juxtaposed after the first's argument begins a new factor, as it is read (user rulings
2026-09-29). Perl's `\mathbb{E}` and `\mathbb{P}` are UNKNOWN letters: `\mathbb{E}X` is E·X, `\mathbb{E}[X|Y]` E·[X|Y],
`\mathbb{P}(A)` P·(A).

**Rust** (57cb): a bare argument's first item may be a function (`op_bare_item`: `\log\log x` log@(log@(x)),
`\max_x\log p(x)`, `\arg\min_w`); a later item may not be an OPFUNCTION (`op_bare_next`, builder.rs), so
`\log x\log y\log z` is log@(x)·log@(y)·log@(z), `\max_i a_i\log b_i` max_i@(a_i)·log@(b_i), `\ln x\ln(1+x)`
ln@(x)·ln@(1+x), `\log^3\omega\log y` (2605.16034), `\nabla x\log y` ∇@(x)·log@(y) (2605.11994). A trig function,
bare or applied, still continues the argument (`\log x\sin y` log@(x·sin@(y)), `\max_j 2\sin\frac{\theta}{2}`,
`\nabla u\sin v`, as Perl), and a bare function head before a bare item takes it (`\tau\log\mathbb{E}_{y'}\exp(x)`
τ·log@(𝔼_{y'}@(exp@(x)))). An ellipsis stays inside only between two items (`op_bare_elided`; an ID one,
`\ldots`/`\dots`, through `op_bare_next`, refused at the end by `operator_bare_apply`): `\log x\cdots y`
log@(x·⋯·y), `\log x_1\cdots\log x_n` and `\log x_1\ldots\log x_n` log@(x_1)·⋯·log@(x_n), and a trailing one leaves,
`\nabla u\cdots` ∇@(u)·⋯, `\nabla u\ldots` ∇@(u)·… (Perl ∇@(u·⋯)). A run of them is one ellipsis here, whatever the
macros (57cj.15, the 57cj.14 review; latent, no corpus witness): `op_bare_elided` goes on over an ELIDEOP and
`semantics::leaves_a_bare_argument` looks past the whole run to the application before it, so `\log x\cdots\cdots y`
log@(x·⋯·⋯·y), `\log x\ldots\cdots y`, `\log\ldots\ldots\cdots x` log@(…·…·⋯·x), `\max_i a_i\cdots\cdots a_n`,
`\nabla u\cdots\cdots v`, as Perl (57cj.14 read log@(x)·⋯·⋯·y); a trailing run leaves (`\log x\cdots\cdots`
log@(x)·⋯·⋯). The run may be joined by MulOps or BinOps, any mix with juxtaposition (57cj.16, the 57cj.15 review:
`op_bare_elided mulop elideop`, `binop`, and the twin `trig_op_bare_elided`, so the grammar derives every run the
refusal looks past): `\log x\cdot\cdots\cdot\cdots\cdot y` log@(x·⋯·⋯·y), `\max_i a_i\times\cdots\times\cdots\times b`,
`\exp x\otimes\cdots\otimes\cdots\otimes y`, `\log x\cdots\cdot\cdots y` log@(((x·⋯)·⋯)·y), as Perl (unparsed on 57cj.15,
log@(x)·⋯·⋯·y on 57cj.14). An OPFUNCTION's chain ending a trig function's argument goes on over the run the same way
(`semantics::trig_argument_ending_in_an_opfunction_application`): `\sin\log x\cdots y` sin@(log@(x·⋯·y)), `\cos\log_2 x\ldots\cdots y`,
as Perl, where both trees used to survive and the forest's order picked one per formula — a second site flipped
`\sin\log x\cdots y` to sin@(log@(x))·⋯·y; a space, a trig function's application after the run and a trailing run
leave it, as they end a trig argument (`\sin\log x\,\cdots\,y`, `\sin\log x\cdots\cos y`, `\sin\log x\cdots` keep
sin@(log@(x))·⋯…). A run a MulOp opens before a juxtaposed item is the same run (57cj.18, the 57cj.17 review): `\log x\cdot\cdots y`
log@((x·⋯)·y), `\nabla u\cdot\cdots v`, `\max_i a_i\times\cdots b`, where (log@(x)·⋯)·y survived beside it (the MulOp's right operand
`⋯ y` hid the junction; `semantics::a_run_stays_inside` at every junction), and a trailing run leaves the chain ending a trig
argument whatever its first macro (`\sin\log x\ldots` sin@(log@(x))·…, #367). Latent: none among the 951 two-ellipsis formulas of the 3,003 A/B sources; the 57cj.16 ellipsis
grid (`~/data/pk_agents/math/grids/ellipsis_grid/`, 77,250 formulas: 15 heads × 4 first items × 6 joins × 5 ellipsis
mixes × run lengths 1-3 × 3 tails × 5 contexts) parses every formula. A bare OPFUNCTION ending a product is a term
(`bare_opfunction_term`), not a factor another factor follows, and a bare head is a MulOp's or BinOp's operand
(`a\cdot\log^2`, `\log\circ\exp`, Perl-unparsed before): the product `tight_term opfunction` let every junction of an
n-function chain be split and refused (2^(n−1) trees; `semantics::leaves_a_bare_argument` still refuses the one split
left, a product whose last application's argument the right side would continue). A blackboard `E`/`P` — `\mathbb{E}`
by the document's decoded font, or the glyphs 𝔼/ℙ (`parser.rs::expectation_operators`, whose lexemes
`type_expectation_lexemes` relabels after lexing) — lexes as an OPFUNCTION: `\mathbb{E}X` 𝔼@(X),
`\mathbb{E}[X|Y]` 𝔼@(X|Y), `\mathbb{E}_{x\sim p}[f(x)]`, `\mathbb{P}(A)` ℙ@(A), `\mathbb{P}\{X\leq t,Y\leq s\}`
ℙ@(X ≤ t, Y ≤ s). Only the lexeme is typed; the token keeps its role, so a set (`x\in\mathbb{E}^3`,
`\sum_{\Gamma\in\mathbb{E}_h}`) or a lone letter carries no function role or operator spacing into the output, and a
numeric power among its scripts names a space and stays a letter (`\mathbb{P}^1\times\mathbb{P}^1`,
`\mathbb{P}^2\#\overline{\mathbb{P}^2}`, `\mathbb{P}_{\mathbb{C}}^2(V)`, `ℙ^{2}(\mathbb{C})`, `\mathbb{R}\times\mathbb{E}^3`). An
evaluation bar evaluates a bare or scripted function or operator head, alone or ending a product, as it does a tight
term (`eval_operand`: `\log^2|_{x=0}`, `\left.\max_i\right|_{i=1}`, `a\log^2|_{x=0}`, `\sin^2|_0^\pi`,
`\left.\nabla^2\right|_{x=0}`, `a\nabla|_0`, as Perl; `\log|_{x=0}`, `\sin|_0` and the `\left.…\right|` operator
forms were Rust-unparsed before 57cb). The standalone `latexmlmath_oxide` types 𝔼/ℙ too
(`type_expectation_operators`, from the serialized `font` of its finalized document). Residuals: a big operator after 𝔼 was a separate factor, as after any OPFUNCTION
(`\mathbb{E}\sum_i X_i` 𝔼·∑…; since 57cf 𝔼 takes it, divergence #379; `\log\sum_i` keeps Perl's reading); LIMITOP heads keep Perl's greedy nest (`\det A\det B`
det@(A·det@(B)), `\liminf`, `\limsup`); a letter-indexed power still reads 𝔼 as a function where a bare item follows
(`\mathbb{E}^n X`; ℙ only where applied since 57cd, `\mathbb{P}^n X` P^n·X), and a numeric one never does (`\mathbb{E}^2[X]` 𝔼²·[X]); a scripted 𝔼 before a scripted group
takes the script inside (`\mathbb{E}_x[X]^2` 𝔼_x@([X]²), #351's scripted-head limit, where `\mathbb{E}[X]^2` is
(𝔼@(X))²); `\lxDeclare[role=UNKNOWN]` does not opt a blackboard E/P out (`role=ID` does); a scripted ELIDEOP ending a bare
argument stays unparsed (`\log x\cdots^2`, `\nabla u\cdots_n`; Perl log@(x·⋯²), as 57ca5), where the ID one leaves
(`\log x\ldots^2` log@(x)·…²); a differential head takes no evaluation bar (`\partial|_0`, `\partial_x|_0`; ∂ is a
DIFFOP big operator, task #185); on the guard's 500-alternative test budget the sixteen-log formula keeps the cut
sample's reading, whose bars misread an evaluation bar (57ca5 left it unparsed; the production budget reads it). 57cd (the regressions of the 57cb train A/B): ℙ is an operator only where it is applied — a group after it, past its
scripts, an OPEN or a group built whole (physics `\qty(A)`, `parser::opens_a_group`; user ruling 2026-09-30):
`\mathbb{P}(A)`, `\mathbb{P}_x[\tau<\infty]`, `\mathbb{P}\{…\}`; otherwise a named matrix or space — `\mathbb{P}^{-1}\mathbb{Q}`
(2605.04101), `\mathbb{P}_{1,1}\mathbb{P}_{1,2}` (57cb: (P_{1,1})@(P_{1,2})) and `\mathbb{P}_1^\intercal\mathbb{P}_1\odot\beta`
(57cb: unparsed; both 2605.01199), `\mathbb{P}\text{-a.s.}` P·[-a.s.] (2605.20593), `\mathbb{P}X` P·X. A ℙ after an integral's differential is
the measure, a letter (`\int u\,\mathrm{d}\mathbb{P}(\omega)` differential-d@(P)·ω as `\mathrm{d}\mu(x)`, 2605.24620):
a `d` counts only in a formula with an INTOP (the lexer's XDIFFUNK rule), a DIFFOP except ∂, a token or built (physics
`\dd`) — `d\,\mathbb{P}(A)` is d·ℙ@(A), `\partial\mathbb{P}(A)` ∂@(ℙ@(A)). 𝔼 with nothing to take — the end, a
relation, punctuation, a close or an infix operator after its scripts, or an operator, which an OPFUNCTION's bare
argument does not take (`parser::is_an_argument`; 57cd review) — is a name: `\nabla\mathbb{E}=0` ∇@(E) = 0,
`\nabla\mathbb{E}\nabla f` ∇@(E)·∇@(f). (A big operator it takes since 57cf, divergence #379.) The lexeme is an EXPECTATION (`EXPECTATION:𝔼:5`, `type_expectation_lexemes`), an OPFUNCTION of its own
category: the grammar's `opfunction` is `plain_opfunction | expectation_head`, and the actions read it as an OPFUNCTION
(`semantics::lexeme_category`; so do CorTeX warning keys). The typing spells the lexeme from the token's
content, never a `meaning` or `name` (`\lxDefMath{\EE}{𝔼}[meaning=expectation]` expectation@(X)). An operator nests over
plain OPFUNCTIONs only (`plain_opfunction`, `scripted_plain_opfunction`) and takes an expectation's application whole,
an operator or a nest of operators only (`open_op_head expectation_application`; a nest over a function takes it as
any function's application): one derivation (57cd reviews: a refused nest, then a refused closed nest, per site made
`\nabla\mathbb{E}[X]+\nabla\mathbb{E}[Y]+\nabla\mathbb{E}[Z]+\nabla\mathbb{E}[W]` and `\nabla\log\mathbb{E}[X]+…` 16 trees
and a warning; 1 now):
`\nabla_{\mathbf x_t}\mathbb{E}[\mathbf x_1\mid\mathbf x_t]` (∇_x)@(𝔼@(x_1 | x_t)) (2605.00941),
`\operatorname*{argmax}_\theta\mathbb{E}(X_\theta)` (2605.03240), were (∇_x 𝔼)@(…); the application forms are an
OPFUNCTION's — a group with its scripts (`\nabla\mathbb{E}[X]^2` ∇@((𝔼@(X))²)), a scripted head's group or scripted
group, a bare argument, a postfixed group's operand (`\nabla\mathbb{E}(X)!` ∇@(𝔼@(X!))), a postfixed scripted group application
(`\nabla\mathbb{E}[X]^2!` ∇@((𝔼@(X))²!)). A group application ends
the operator's argument, as a group does (`is_closed_expectation_application`): `\alpha_t\nabla^{\top}_{x_t}
\mathbb{E}[x|x_t]\Sigma_t^{-1}` α_t·∇@(𝔼@(x | x_t))·Σ_t⁻¹ (2605.20593); a bare one keeps its greedy argument
(`\nabla\mathbb{E}XY` ∇@(𝔼@(X·Y))). A closed nest takes it as any bare argument and goes on:
`\nabla\log\mathbb{E}[X]Y` (∇@log)@(𝔼@(X)·Y), as `\log\mathbb{E}[X]Y` (`nest_is_open` in
`is_bare_operator_application`). Residuals (SYNC "𝔼/ℙ follow-ups"): before a factor the bare route is a refused
twin, as for any OPFUNCTION after an operator, and so is a scripted group's bare-item reading, as for any OPFUNCTION
(`\log[X]^2` 2 trees, four in a sum 16 and a warning, 57cc too) (`parse_tree_count_limits` rows
`operator_before_an_expectation_*`: 1, 1, 2, 2, 1 closed nest, and 1 for three sites in a sum); a scripted 𝔼's group
with scripts then a postfix after an operator is unparsed (`\nabla\mathbb{E}_x(X)^2!`), as a paren group holding a
fenced modifier (`\nabla\mathbb{E}(\to x)`, `\nabla\mathbb{E}(>0)`); a measure's scripts hide the differential before it
(`\int u\,d\mathbb{P}_x(\omega)` u·d·P_x·ω; 57cc d·(P_x)@(ω), neither the measure); a letter after the application multiplies (`\nabla_\theta\mathbb{E}_x[f(x)]g(y)`
…·g·y, where 57bl applies it after `\nabla(X)`); ℙ applied in one place and a matrix in another of the same formula is
read by place (`\mathbb{P}(\mathbb{P}^{-1}\mathbb{Q}-zI)^{-1}` (ℙ@(P⁻¹·Q − z·I))⁻¹, 2605.04101, a matrix product;
direction: one reading per formula); `\mathbb{P}\abs{X}` applies ℙ where `\mathbb{P}|X|` multiplies; a declared operator (`\DeclareMathOperator{\E}{\mathbb{E}}`) is no expectation and nests
(`\nabla\E[X]` (∇@E)@(X)); 𝔼 takes no operator (`\mathbb{E}\nabla f` E·∇@(f), an OPFUNCTION's `aBarearg`). 𝔼 takes a
following big operator since 57cf (divergence #379). Queued by the same rulings (SYNC): a scripted head's scope follows its bound variable
(`\min_{\mu}\tau\mathbb{E}_z\phi(…\mu…)`, 2605.02116, 2605.26653). A blackboard alphabet row
(`\mathbb{ABC…Z}`, short-math-guide:1131) enumerates 21 trees for its E (57cc: 100), a warning, reading
A·B·C·D·𝔼@(F·…·Z).
**Guards**: golden
`tests/parse/opfunction_arguments.tex#juxtaposed_operator_functions_are_separate_factors` (with the 57cb review rows) and its 57cd rows; `perfect_kernel_batch57::probability_applies_to_a_physics_group`;
`tests/parse/postfix_operands.tex` (`a\log\nabla^{2}!` a·log·(∇²)!, one product); `parse_tree_count_limits` rows
`three_logs`, `seven_logs`, `scripted_max_then_log`, `logs_across_an_ellipsis`,
`operator_before_an_expectation_{group,bare,then_a_factor}`, `operator_before_expectations_in_a_sum`;
`perfect_kernel_batch57::opfunction_chain_parses_past_the_tree_sampler` (repro
`math-parse/opfunction_chain_parses_past_the_tree_sampler`, now one reading per chain),
`perfect_kernel_batch56::{varmathbb_is_its_own_alphabet, varmathbb_survives_an_earlier_renewcommand}` (the role stays).

### 377. A bare conditional keeps its events in the condition; any other relation relates the conditional

Perl's bare bar is a factor continuation, `moreFactors : evalAtOp ExpressionsNoBars` (MathGrammar:252-268): the condition
is a bar-free list of plain expressions, and a relation after it relates the whole conditional — `y|x\sim N(0,1)` is
conditional@(y, x) ∼ N@(0, 1), but also `Y\mid X=x` conditional@(Y, X) = x and `\sum_{j|j=1,\ldots,n}` a relation
over a conditional. Rust's bar was statement-level only (`statement vertbar statements`): the condition took every
relation after it, `y|x\sim N` conditional@(y, x ∼ N), and after 57bv.1's refusal of a statement-list condition the
index-list shapes were unparsed (2605.05396, 2605.13128, 2605.19519, 2605.03152).

**Rust** (57cc; user ruling 2026-09-29): an *event* — an undecorated equality, inequality, order, membership or
inclusion (`is_event_relation`: `=`, `≠`, `<`, `>`, `≤`, `≥`, `≪`, `≫`, `∈`, `∉`, `∋`, the subset/superset family) —
states what is given and stays in the condition, as the statement-level bar reads it: `Y\mid X=x` conditional@(Y, X = x),
`T-s\mid T>s`, `y|x_1=a_1,\ldots,x_n=a_n`, `\sum_{j|j=1,\ldots,n}`. Any other relation (`\sim`, `\approx`, `\propto`,
`\equiv`, an arrow, a decorated relation) relates the whole conditional through the factor-level `conditional_head` /
`conditional_formula` (builder.rs; Perl's reading): `y|x\sim N(0,1)`, `\theta|y\propto p(y|\theta)p(\theta)`,
`\gamma_g\mid\theta\stackrel{\mathrm{iid}}{\sim}\mathrm{Bernoulli}(\theta)`, `x|y\to 0`, and mixed ones keep the event
inside, `WT\mid W=w\sim\mathrm{Gamma}(a,b)` conditional@(W·T, W = w) ∼ Gamma@(a, b), `U\mid A=a,B=b\sim V`
conditional@(U, formulae@(A = a, B = b)) ∼ V (2605.18724; Perl splits the condition at the comma,
formulae@(conditional@(U, A) = a, B = b ∼ V)); the conditioned is an expression (`W_t-W_s\mid\mathcal F_s\sim\mathcal
N(0,t-s)`, `X-\mu\mid\sigma\sim t_\nu`; Perl binds the term, minus@(W_t, conditional@(W_s, F_s))), the chain goes on
through any relation (`X_n\mid\mathcal F\sim\mu_n\to\mu`), a METARELOP after it is a statement (`\Gamma\mid\Delta\vdash e:\tau`
conditional@(Γ, Δ) proves e colon τ, `a\mid b\iff a\le b`, `0\mid n\leftrightarrow n=0` 2605.29001, as Perl), and a
conditional on each side of a relation or METARELOP is two, an event between them too (`x|y\sim z|w` (x|y) ∼ (z|w),
`3\mid n^2\Rightarrow 3\mid n`, `a|b=c|d`, `a\mid b\iff c\mid d`, as Perl; `vertbar_modifier` refuses a relation over a
conditional as its left side), and a list before a relation over a conditional is no distributed relation
(`a,b\mid c\sim d` formulae@(a, conditional@(b, c) ∼ d), `\Delta;\Gamma\mid\Phi\vdash P`). The two readings complement
each other — `vertbar_modifier` refuses a condition holding a non-event (`condition_holds_a_non_event`),
`conditional_relation` an event relation, `conditional_over_a_list` a relation list that `relation_pairs` owns — one
derivation each. The factor-level conditional is no `formula`, so fences and set-builders keep their bar (`P(A|B=b,C=c)`,
`\{x\mid x>0\}`), and it enters lists (`y_i|\theta_i\sim P,\quad i=1,\ldots,n` formulae@(conditional@(y_i, θ_i) ∼ P, i =
list@(1, …, n)), `y|x\sim N,\ z|w\sim M`). Changes: divisibility before `\equiv` reads the relation over a conditional (`n\mid m\equiv 0`). Residuals (SYNC_STATUS
"Enumeration residuals"): every bare conditional before a relation keeps a refused twin in the forest — an event the
factor-level one, a non-event the statement-level one — so raw tree counts roughly double per conditional (`Y|X=x` 2,
`y|x_1=a_1,\ldots,x_n=a_n` 3 → 6, `x|y(x)\sim p(x|y(x))` 8 → 16, three subscript conditionals 144 → 2,304) and some pass
the 10-tree `ambiguous_math` threshold with their reading unchanged (a conditional in a list of three or more items,
`y_i|\theta_i\sim P,\ i=1,\ldots,n` 11); fix direction: split events from other relations in the grammar so neither twin
is built. Unparsed, where 57cbb read the statement-level bar over them (nonsense readings): a conditional after a
relation (`x=y|z\sim N`, `y^{(k)}_t:=y_t|z_t=k\sim\mathrm{Beta}(…)`, `A\perp B\mid C\sim D`, `\alpha\Rightarrow y|x\sim N`,
`\Gamma,x:A\mid\Delta\vdash e:B`), a bare conditional as a list's first item (`y|x=1,\ z|w\sim N`: `list_apply` refuses a
conditional item, as it does for `Y\mid X=x,\quad i=1,\ldots,n`, unparsed before 57cc too), a placeholder condition
(`\Gamma\mid\cdot\vdash e:\tau`). **Guards**: golden
`tests/parse/bar_pairs.tex#a_bare_bar_binds_its_adjacent_factors`;
`perfect_kernel_batch57::conditional_ends_before_the_next_statement` (repro
`math-parse/conditional_ends_before_the_next_statement`).

### 378. A plain run beside an attached enumeration attaches

Perl's `maybeColRHS` (MathGrammar:165-172) attaches every comma run after a relation: `a=b,c,d` is a = list@(b, c, d),
which divergence #37 (KNOWN_PERL_ERRORS #37) refuses — a bare list is no relation operand — keeping the run as items,
except an enumeration holding an ellipsis (57bv, `i=1,\ldots,n` i = list@(1, …, n)). So a plain run beside an attached
enumeration read as loose items: `i=1,\ldots,4,\,j=0,1,2` list@(i = list@(1, …, 4), j = 0, 1, 2), where Perl reads
formulae@(…, j = list@(0, 1, 2)).

**Rust** (57cc; user ruling 2026-09-29, both directions; `attach_in_container`, semantics.rs): a plain run attaches as
the values of the relation before it in its own segment when another relation of the container took an enumeration on
its right, the relation takes a right run (`takes_right`), the run bridges no chain to the next relation, and each item
continues the relation's value (`continues_progression`: a number after a number, the same letter under scripts, a plain
letter after plain letters): `i=1,\ldots,4,\,j=0,1,2` formulae@(i = list@(1, …, 4), j = list@(0, 1, 2)),
`i=1,\ldots,n;\ j=1,2`, `i=1,\ldots,n,\ \sigma=x,y,z`, and before the enumeration `d=0,1,\,k=1,\ldots,K` formulae@(d =
list@(0, 1), k = list@(1, …, K)) (2605.18167, ~10 formulas), `\kappa_j^k\sim U[-1,1],\quad j=1,2,3,\quad k=1,\ldots,K`.
A signed number continues numbers (`i=1,\ldots,n,\ j=-1,0,1` j = list@(−1, 0, 1)). The run attaches whole or not at all:
kept as items when any item continues nothing (`i=1,\ldots,n,\ x=0, y`, `i=1,\ldots,n, j=1,2, k`), without an
enumeration beside it (`a=b,c,d`, `i=1,2`), and beside a left-attached one, which ranges no relation (`x_1,\ldots,x_n\ge
0,\ k=1,2` keeps `k = 1, 2` loose; Perl k = list@(1, 2); no corpus case). **Guards**: golden
`tests/parse/enumerations.tex#a_plain_run_beside_an_enumeration_attaches`.

### 379. An expectation takes the big operator after it

Perl's `\mathbb{E}` is an UNKNOWN letter, and an OPFUNCTION's bare argument takes no big operator (`aBarearg`,
MathGrammar:323-331), so `\mathbb{E}\frac1n\sum_i X_i` is E·(1/n)·∑… (latexmlc, 0 errors); since 57cb (divergence #376)
Rust read 𝔼@(1/n)·∑…, the expectation of the coefficient alone.

**Rust** (57cf; user ruling 2026-09-30, "follow the mathematical meaning": an expectation of a sum written without
parentheses is of the whole sum): an expectation takes the summation-like big operator after it (BIGOP, SUMOP, INTOP,
LIMITOP — Perl's `bigop`, MathGrammar:717, less DIFFOP, so a physics `\dd t` stays outside) — alone,
`\mathbb{E}_{x\sim\rho}\sum_a` (2605.06977 A3.Ex64.m2), `\varepsilon\mathbb{E}\int_0^T` (2605.13204 S2.Ex6.m1),
`\mathbb{E}\sup_g\int` (2605.03300 S3.E17.m2), `\mathbb{E}\prod_{s=1}^4` (2605.22025 A3.Ex75.m1), `\mathbb{E}_t\inf_z`
(2605.07165 S3.Ex134.m2); after its bare argument's coefficients, `\mathbb{E}\frac1n\sum_{i=1}^n[X_i]` 𝔼@((1/n)·∑…)
(2605.02116 A3.Ex163.m2), `\eta\mathbb{E}_{x\sim\rho}\gamma\sum_a` η·𝔼@(γ·∑…) (2605.06977 A3.Ex74.m1); as a function's
or operator's whole bare argument, `\max_\pi\mathbb{E}_{\tau\sim\pi}\sum_t` max_π@(𝔼_τ@(∑…)) (2605.11975 S6.E24.m1,
2605.11742 S4.E12X.m1), `\operatorname{Tr}\mathbb{E}\prod` (2605.02768 S1.Ex1.m1), `\nabla\mathbb{E}\frac1n\sum_i X_i`
∇@(𝔼@((1/n)·∑…)). The grammar derives these shapes once, as the product they are for any OPFUNCTION, and the product's
action reads it (`product_before_a_big_operator`, `function_before_a_big_operator` →
`expectation_takes_the_big_operator`: the deepest expectation the factor before the big operator ends in takes it), so
no rule and no refused tree is added; after an operator, which nests over no expectation, one term-level rule takes it,
where the big operator's operand runs over the term (`open_op_head expectation_before_a_big_operator bigop_operand`,
`operator_takes_an_expectation_s_big_operator`: `\nabla_\theta\mathbb{E}_x\sum_i f_i` (∇_θ)@(𝔼_x@(∑…)),
`\nabla_\theta\mathbb{E}_{x\sim\mathcal{D}}\int\pi_\theta(y\mid x)\cdot\frac{…}{…}dy`, 2605.09853; a first cut inside
the tight-term `expectation_application` split the operand at every factor, `\nabla\mathbb{E}\int_0^T u(t)v(t)\,dt` 21
trees and a warning; 57cf review). Where the grammar has no reading for an expectation — before a big operator in a trig
function's argument, after a closed nest or a MulOp in a bare argument (`\sin\mathbb{E}\sum_i X_i`,
`\nabla\log\mathbb{E}\sum_i X_i`, `\log x\cdot\mathbb{E}\sum_i X_i`) — it reads as the letter Perl reads, rather than
leaving the formula unparsed: a formula no typed stream parses is parsed again with every expectation also read as
its letter (M3, parser.rs `spell_letter_readings`: the lexeme `EXPECTATION:𝔼.letter`, whose grammar category
`expectation_letter` presents the UNKNOWN lexeme the lexer gave it, `semantics::expectation_as_letter`), and the
counting soft prune `ExpectationLettersAreFallbacks` keeps the readings with the fewest letter readings — per glade in
the ASF traverser (`keep_fewest_letter_readings`), at the root on every route — one expectation at a time:
`\sin\mathbb{E}\sum_i X_i+\mathbb{E}Y` keeps 𝔼@(Y), `\log x\cdot\mathbb{E}\sum_i X_i=\mathbb{E}X` 𝔼@(X) (the untyped retry
of 57cf.1–57ch.1 read every expectation of the formula a letter). As lexed first, then with the fence retry's null
delimiters (`\sin\mathbb{E}\sum_i X_i|0\rangle`, 57ch review). A stream a typed reading parses takes no letter twin:
offered in every formula, the twin doubles the tree iterator's trees per expectation (probed:
`\nabla\mathbb{E}[X]+…+\nabla\mathbb{E}[W]` 16 trees); offered only before a big operator (M3 as committed), it still
added 7 `ambiguous_math` in the delta A/B with no reading changed (2605.02116, 2605.03300, 2605.13204, 2605.29267) and
lost the retry's parse in contexts no list foresees (`\nabla_\theta\mathbb{E}_x\log\sum_y p_\theta(x,y)`,
`\sin\mathbb{E}_x f(x)`; M3 review). The gate is exact, not only cheaper: `ExpectationLettersAreFallbacks` ranks before
every other pragma, so a typed reading beats every letter reading whenever one exists — computing the letter readings
only when none does is the ungated category's result, evaluated lazily (M3.1 A/B: 0 readings changed over the 3,003
papers). Residuals: inside a letter retry each expectation still doubles the tree iterator's trees
(`\sin\mathbb{E}\sum_i X_i=\mathbb{E}[A]\mathbb{E}[B]\mathbb{E}[C]\mathbb{E}[D]` 16 trees and a warning; 0 corpus hits);
the retry's letter readings are Perl's, not always good (`\sin\mathbb{E}_x f(x)` sine@(E_x·f)·x splits f(x),
`\sin\nabla\mathbb{E}\sum_i X_i` sine·∇@(E)·∑…, and against the rulings: `\cos\mathbb{E}\mathbb{E}\sum_i X_i` cos@(E@(E))·∑…,
intended cos@(𝔼@(𝔼@(∑…))) — 𝔼 before ∑ takes the sum; `\nabla_\theta\mathbb{E}_x\log\sum_y p_\theta(x,y)` (∇_θ)@(E_x)·log·∑…,
intended (∇_θ)@(𝔼_x@(log ∑…)); `\log x\cdot\mathbb{E}\mathbb{P}(A)\sum_i X_i` log@(x·E)·P@(A)·∑…, intended log(x)·𝔼@(ℙ(A)·∑…)
— golden rows pinning today's retry readings, M3.2); `latexmlmath_oxide` runs no fence or letter retry.
`\nabla\mathbb{E}_x\mathbb{E}_y\sum_i Y_i` reads ∇@(𝔼_x@(E_y·∑…)) (the retry: ∇@(E_x·E_y)·∑…); both lack the nested
𝔼_x@(𝔼_y@(∑…)) the formula means, which needs an operator's route over a chain of expectations. The pragma
`BigopPreferWiderAbsorption` reads through an unfenced bare OPFUNCTION or operator application
(`pragmatics::ends_in_a_bigop_application`), so `\mathbb{E}\sum_i X_i\cdot c` is 𝔼@(∑(X_i·c)), as `\log\sum_i x_i\cdot c`
log·∑(x_i·c). As they read: a group closes the expectation (`\mathbb{E}[X]\sum_i Y_i` 𝔼@(X)·∑…), a sum ends the big
operator (`\mathbb{E}\sum_i X_i+c`), a MulOp is no juxtaposition (`\mathbb{E}X\cdot\sum_i Y_i`, 2605.28612 keeps a sum
outside with `\times`), another OPFUNCTION keeps Perl's product (`\log\sum_i x_i`, `\min_\theta\frac1n\sum_i`, and inside
the argument `\mathbb{E}_{z_j}\min_\mu\frac{\tau}{m}\sum_j` 𝔼@(min_μ@(τ/m)·∑…), 2605.02116 A5.Ex283.m1), an unapplied ℙ is
a letter (`\mathbb{P}\sum_i X_i`). **Guards**: golden `tests/parse/opfunction_arguments.tex` (the 57cf block);
`parse_tree_count_limits` rows `expectation_before_a_big_operator`, `expectation_coefficient_before_a_big_operator`,
`expectation_midterm_before_a_big_operator`, `expectation_nested_before_a_big_operator`,
`operator_before_an_expectation_before_a_big_operator`.

### 380. A limits head's bare argument takes a later application that mentions its bound variable

Perl's bare argument is greedy (`barearg`, MathGrammar:321-337, :553-558): `\max_i a_i\log b_i` is max_i@(a_i·log@(b_i)),
but so is `\max_x f(x)\log y`; divergence #376 (57cb) made every later OPFUNCTION a separate factor, splitting both.

**Rust** (57cg; user ruling 2026-09-30, "a scripted operator's scope follows its bound variable"): the bare argument of
a head whose subscript binds a variable takes a later OPFUNCTION's application that mentions one free
(`bound_application`, `semantics::bound_argument_extends`; the product that would leave it outside is refused,
`bound_head_takes` in `leaves_a_bare_argument`): `\max_i a_i\log b_i` max_i@(a_i·log@(b_i)),
`\min_{\mu\in\mathbb R}\tau\mathbb{E}_z\phi((z-\mu)/\tau)+\mu` min_μ@(τ·𝔼_z@(φ(…)))+μ (2605.02116 A5.Ex275.m1 and 28 more),
`\min_{\eta>0}\eta\ln\eta` (2605.18694), `\max_{\boldsymbol\gamma\in H}4n\exp(\ldots\boldsymbol\gamma\ldots)` (2605.26653),
`\max_{S\ni M}e\ln|S|` (2605.09953), `\min_{1\le i\le N}2\arccos(q_i)` (2605.17033). A head binds its subscript when its
scripts are limits — TeX's `\mathop` without `\nolimits` (`\max`, `\min`, `\gcd`, `\Pr`: plain.tex:1073-1074, :1083-1084;
LaTeXML's `scriptpos`, math_common.pool.ltxml:742, :759-764), a starred operator name (`\operatorname*`,
`\DeclareMathOperator*`, amsopn.sty.ltxml:23-29, :50-53) — and, by the same ruling's meaning, the named `argmin`/`argmax`
and an expectation (`\mathbb{E}_x f(x)\log g(x)` 𝔼_x@(f(x)·log g(x))); a `\nolimits` head's script is a base or a
parameter (`\log_a b`, `\operatorname{rank}_K`), an operator's its variable of differentiation (`\nabla_x`): neither
binds. The subscript binds a letter, a chain's interior operands (`\max_{a\le x\le b}` x, `\max_{1\le i<j\le n}` i and
j; 57cg.1), a single relation's first operand holding one (`\max_{i\ne j}` i), each item of a list, a scripted letter by
its base (`\max_{x'}f(x')\log x'`); a mention is a free occurrence in the same font (a script's too, `\log b_i`), not
where an inner head or big operator rebinds the plain letter (`\max_{z\in A}|f(z)|\max_{z\in A}|g(z)|`, 2605.24231); a
scripted inner binder rebinds nothing (`\mathbb{E}_{y\sim p}\tau\log\mathbb{E}_{y'}\exp(y\cdot y')` mentions y,
2605.02116). With no mention the argument ends as #376 reads it (`\max_x f(x)\log y`, `\max_{k\notin\{i,j\}}\frac{\pi_k}{\pi_d}
\exp(-\kappa)`, 2605.06831). The innermost head binding a mentioned variable takes it (`\max_i a_i\max_j b_{ij}\log c_j`
max_i@(a_i·max_j@(b_ij·log@(c_j)))), through an operator's argument too (`\operatorname*{argmin}_w a_w\cdot\max_i
b_{iw}\cdot\log c_i` argmin_w@(a_w·max_i@(b_iw·log c_i)), 57cg.1). Each limits head's argument has its own start
(`limits_bare_application`), so a formula with no candidate item derives once; k OPFUNCTIONs after one head's argument
are k+1 derivations, limits heads in a row compound (Catalan: three 5, four 14), and separate sites multiply — n terms
each with a head and a later OPFUNCTION are 2^n trees whether or not anything is mentioned (`x+\max_i a_i\log c+…` with 4
terms 16 trees and an `ambiguous_math` warning; 12 terms 4,096 in 0.76 s; 14+ reach the 5,000-tree cap, still read
right), and every subscripted operator pays a refused twin (`limits_operator_head` takes any `operator postsubarg`:
`\nabla_x f(x)\log x` 2 → 3 trees). The direction is the one recorded for #377's conditional twins: split the grammar so
the refused reading is never derived. Residuals: the scope ends at the first later OPFUNCTION that does not mention the
variable, so a later mention stays outside (`\max_i a_i\log c\log b_i` max_i@(a_i)·log c·log b_i — the scope should reach
the last mention, a formula-level check); a variable is its base letter (`\max_{x'}f(x')\log x` extends; the whole
scripted identifier — base, script, font — should bind and rebind). **Guards**: golden `tests/parse/opfunction_arguments.tex` ("A scripted operator's
scope follows its bound variable"); `parse_tree_count_limits` rows `scripted_max_then_log`, `bound_variable_*`,
`limits_head_*`, `nolimits_scripted_chain`, `starred_operator_bound`, `bound_head_as_a_first_item`, `nested_bound_heads`,
`bound_witness_min_mu`.

### 381. (withdrawn in 57ch.1) A `<` relation in a `\langle…\rangle` Dirac label

57ch let a plain `<`/`>` relation stand in a `\langle…\rangle` label, where Perl's `$forbidLRAngle` (MathGrammar:402,
:708) forbids it; its one witness, `\langle p_\alpha\mid\alpha<\gamma\rangle` (2605.09161), is an ordinal-indexed
sequence, not an inner product. Withdrawn on review: the label refuses the relation again
(`semantics::holds_plain_angle_relation`), as Perl, and the formula stays unparsed; a sequence-builder reading is open
(SYNC_STATUS "Dirac shapes"). The rest of 57ch's Dirac work is Perl parity, not a divergence: labels are Perl's
`ketExpression : Formulae` (:401-404; `|n,l\rangle` ket@(list@(n, l)), `|F=1,m_F=1\rangle` ket@(formulae@(…))), and a bra
directly before a ket stays a bra, Perl's `maybeBra` fallback (:373-379; `\langle x||y\rangle` bra@(x)·ket@(y), 2605.29622).
**Guards**: golden `tests/parse/bar_pairs.tex` ("A Dirac label is a formula or a list", "A bra directly before a ket
stays a bra").

### 382. amsmath's `\lvert` opens a ket and `\rvert` closes a bra

amsmath's `\lvert`/`\rvert` are an OPEN and a CLOSE (amsmath.sty.ltxml:1150-1152), so Perl reads `\lvert\psi\rangle` as a
group, delimited-|⟩@(ψ), and leaves `\langle\psi\rvert H\lvert\psi\rangle` unparsed.

**Rust** (57ch; 2605.02211, ~49 formulas): they are a ket's opening and a bra's closing bar (`ket_bar`, `bra_bar`,
`braket_bar`): `\lvert\psi\rangle` ket@(ψ), `\langle\psi\rvert` bra@(ψ), `\langle\psi\rvert H\lvert\psi\rangle`
quantum-operator-product@(ψ, H, ψ), `\langle\phi\rvert\psi\rangle` inner-product@(φ, ψ), `\lvert\psi\rangle\langle\psi\rvert`
ket@(ψ)·bra@(ψ); never a conditional's divider, and a group of its own (`\lvert x\rvert\lvert y\rvert` |x|·|y|) as before.
**Guards**: golden `tests/parse/bar_pairs.tex` (the `\lvert` rows).

### 383. A copied picture's svg ids are renamed with their references

Perl writes a copied pgf picture's svg ids once per copy (KNOWN_PERL_ERRORS #391): the driver fixes them at digestion
and `\copy` repeats the whatsit, so the XML repeats an ID-typed attribute.

**Rust** (57cl, 57cl.1): each outermost `svg:svg` root, when it closes, makes the svg `id`s under it unique in the document
(`Document::record_svg_ids`, triggered by the `svg:svg` close hook in tex_box.rs; `Document::svg_ids` is the set of ids
handed out): an id already handed out (or held by an `xml:id`) takes the next `radix_alpha` suffix (`pgfcp1` →
`pgfcp1a`, a per-base counter), and every reference under the root — `url(#X)` in any attribute, `href`/`xlink:href`
`#X` (the latter a literal prefixed name until finalize) — binds to the latest preceding definition of X under the root;
a reference to a definition in another picture is left alone. A walked element carries a `_svgid` mark (stripped at
finalize): the math parser moves a picture by re-creating it (`append_tree` copies attributes), closing a new `svg:svg`
each time, and the mark tells that move from a TeX copy — so a picture in math keeps Perl's ids (57cl keyed the registry
by node address and renamed each re-creation, leaving `xlink:href` dangling). `append_clone` (MathFork) makes a second
copy instead; a fix to `append_clone_aux` copying the svg `id` as Perl does must drop the mark from the clone. A picture
keeps Perl's ids unless an earlier element holds them (a relocated footnote copy can); nothing is dropped (content_diff: suanpan-l3, thuaslogos unchanged). Modelled on Perl's own
`appendClone` id map (Document.pm:1920-1956). Witnesses: suanpan-l3 (9,480 of 9,481 jing lines), thuaslogos-doc-english,
thuaslogos-doc-dutch (4 each; copies nested in one picture) — +2 schema-valid manuals. Residual: an `xml:id` recorded
after a picture that holds the same svg id is not renamed (the check is one-way); `append_clone_aux` turns an svg `id`
into an `xml:id` (SYNC_STATUS). **Guards**: `perfect_kernel_batch57::copied_pictures_keep_unique_svg_ids`,
`perfect_kernel_batch57::rebuilt_math_pictures_keep_their_svg_ids`; repros
`graphics-tikz/copied_picture_keeps_unique_svg_ids`, `graphics-tikz/rebuilt_math_picture_keeps_its_svg_ids`.

### 384. tabularray tables are reduced to the kernel's `tabular`/`array`

Perl has no tabularray binding beyond `\tblr` → `\tabular` (ar5iv's `tabularray.sty.ltxml`), which hands the key-value
inner spec to the column-template parser; raw-loading tabularray.sty (its l3 table builder measures and typesets
boxes) is not a route either engine can take.

**Rust** (56-series, 57co, 57cp; tabularray_sty.rs): every tblr-family environment is the kernel's alignment.

- *Specs.* The inner spec is a colspec or a key list by its first key name (tabularray.sty:4045-4057). The colspec is
  read by characters (`\NewColumnType{é}`) and translated to a classic template (`Q`/`X` by their alignment,
  `t`/`h`/`f` as `p`, `\NewColumnType`s expanded); a spec without one has its columns inferred (a wide `l` template
  the text alignment prunes); a text table gets 16 margin columns for ragged rows, continuing the last column (`l`
  with none: the initial `halign=j`, :3908-3910). A type tabularray does not know is its error ("Unknown Column
  type S!", :3389-3410) for the first such type, where the template ends (the rest, stray text in tabularray, is
  dropped). Table commands (`\NewTblrTableCommand`, tabularray's `\hline[…]`/`\cline[…]`/`\pagebreak`/`\nopagebreak`)
  mean their body inside a table only. `\SetTblrInner[]{…}` (an explicitly empty list) applies to no environment.
- *Math.* A table in math mode is an `array` with math cells and no margin (math alignments are not pruned), as wide
  as its widest row, read from the body ahead of time (tabularray's `+b`: a row wider than the colspec, a
  colspec-less table) — the table's own cells, not an unbraced environment's in a cell — up to the first `\end{…}`
  that closes no `\begin{…}` of the body (an environment wrapping the table), at most 65,536 tokens. A long or tall
  one there is an `\hbox` holding its box; its `tex=` holds the reduction, not the source.
- *Long and tall tables* are one box placed where they are written (:6384-6474): the caption (the kernel's
  `\caption`, saved at the document's start so longtable's row caption cannot take it), notes and remarks go into a
  `table` inside the binding's own box (`{lx@tblr@box}`, the kernel's `insert_block`, as wide as the table:
  tabularray sets caption and notes at the table's width, :6480-6508). The box stands as the table between
  paragraphs (ruling C); in a float without a caption of its own the two are one float (`collapse_float`, with the
  table's id: a caption-less `figure` holding one becomes `<figure xml:id="S1.T4" inlist="lot">`), beside other
  content the table stays one panel of it, and beside the float's own `\caption`, before or after, each keeps its
  number; elsewhere (quote,
  abstract, algorithm, threeparttable, footnote, cell, `\resizebox`, math) it stays in place, where a float opened
  directly moved out. A long table ends the paragraph around it (:6550 `\par`), a tall one is a box in the line.
- *Captions and templates.* `label=none` or an empty `caption-tag` prints the text alone; an empty `firsthead`/
  `caption`/`caption-text`/`lastfoot`/`note`/`note-tag`/`note-text`/`remark`/`remark-tag`/`remark-sep`/`remark-text`
  template (a theme's too) drops what it prints; a numbered table whose caption does not print still carries its
  number for `\ref`. The List of Tables line follows `entry` (:6459-6478): none for `entry=none`, one for a
  `label=none` table (the counter as it stands) and for a hidden caption; a line with no text goes only on a table
  that has a float for other reasons; an empty `caption-lot` unlists a table unless the document's own
  `firsthead`/`lastfoot` template — or its `caption` template under a head that uses it: tabularray's own
  `firsthead` (:6276-6279) or a document head calling `\UseTblrTemplate{caption}` (njuthesis.cls:1597-1602) —
  calls `\caption` or `\captionof{table}` (not starred; directly, as a `\let` copy or `\csname caption\endcsname`,
  or through the macros it calls to four levels, looked up when the table is typeset; not seen: a star passed as a
  macro's argument or after its arguments, a `\let` copy taken before a package redefines `\caption`, an
  `\endcsname` from a macro) and the table is numbered (tblr-extras' `caption`
  library writes the line from its own head, which prints nothing under `label=none`, tblr-extras.sty:47-50). The
  binding keeps each template's state — empty, tabularray's own, or the document's with its code — and a template
  the document redeclares, `empty` included, is the document's. `\SetTblrTemplate` with a name the
  element has no template of is an error (:5703-5718). A document template prints as tabularray's own would: a
  custom `caption-tag`/`caption-sep` as the kernel's "Table N:" (tabularray-abnt's `quadro` theme prints "Quadro N —"
  on its own counter), a formatting-only `caption` template with the tag, tblr-extras' head under `label=none` the
  caption text (it prints nothing) and its `entry=none` line not at all (it lists "none"); a long or tall table in a
  `figure`/`subfigure` with a caption of its own is a `table` inside it.
- *Libraries* are reduced: amsmath's `+matrix`…`+cases` are amsmath's own matrices in math and a text table in text
  (delimiters and options dropped); tikz's `tblrtikzbelow`/`tblrtikzabove` overlays are dropped with a warning (no
  cell nodes to draw on); siunitx's `S`/`s` columns are centered columns (their `\TblrNum`/`\TblrUnit` cell
  formatting, :8416-8425, is not applied).

Not modelled: cell/row/column styles (`cell{…}={…}`, `row{…}`, fonts, colors, `cmd=`), `\SetCell[c=…,r=…]` spans,
`rowhead`/`rowfoot` repetition, the `rownum`/`colnum`/`rowcount`/`colcount` counters, `evaluate=`, the List of Tables
line without text of a table with no float of its own (a caption-less long table). Witnesses: the tabularray manual
(26 errors before 57co → 7 → 1 after 57cp), panda, pegmatch, circularglyphs, PixelArtTikz, tblr-extras,
2605.06284. **Guards**: `perfect_kernel_batch57::{tabularray_libraries_and_public_variables,
tabularray_long_table_caption_notes,tabularray_table_commands_and_key_lists,tabularray_unknown_column_type,
tabularray_table_placement,tabularray_math_tables,tabularray_undefined_theme,tabularray_list_of_tables_entries,
tabularray_box_is_as_wide_as_its_table,tabularray_template_names_and_environment_lists,
tabularray_tall_table_in_a_captioned_float,tabularray_caption_library_lists_its_tables,
tabularray_tall_table_in_a_captionless_float,tabularray_template_calling_caption_is_listed}`,
`perfect_kernel_batch54::tabularray_*`, `perfect_kernel_batch56::tblr_*`.

### 385. A block that cannot be demoted in a scaled box becomes an inline container

Perl's `insert_block` renames a capture that cannot stay a paragraph to its first block candidate that holds the
content (`ltx:block`, `ltx:logical-block`, `ltx:sectional-block`, `ltx:figure`; TeX_Box.pool.ltxml:502-513),
allowed in the context or not: a `{minipage}` holding a float, a theorem or a `\section*` inside `\resizebox`,
`\scalebox` or `\rotatebox` (an `ltx:inline-block`, which holds none of those) becomes `inline-block >
logical-block` or `> sectional-block`, a schema error.

**Rust** (57cp, base_utilities.rs `insert_block_as`): where the parent holds neither the final tag nor `ltx:block`
(so no demotion), the capture is the first of `ltx:inline-block`, `ltx:inline-logical-block`,
`ltx:inline-sectional-block` that the parent holds and that holds all its content; otherwise Perl's rename. Nothing
moves: the element is renamed in place. Documents without such a box are unchanged (the 57cp manual net: only the
tabularray/abntex set moved). Witnesses: tabularray's tall table in `\resizebox` (the tabularray manual), the 57cp
review's kern.tex cases (valid pdflatex input; jing rejected three on 57cpe). **Guard**:
`perfect_kernel_batch57::minipage_blocks_in_a_scaled_box_are_inline`; repro
`boxes-groups/minipage_blocks_in_a_scaled_box_are_inline`.

### 390. A function takes a limit-type operator's application

Perl's OPFUNCTION takes no big operator as its argument (`aBarearg`, MathGrammar:323-331), and a LIMITOP — `\det`,
`\sup`, `\inf`, `\lim`, `\dim`, `\liminf` — is one (`bigop`, :717), so a function before one multiplies it:
`\log\det A` reads logarithm * determinant@(A), `\sin\det A` sine * determinant@(A) (KNOWN_PERL_ERRORS #400).

**Rust** (57cj.9; 57cj.8 review; `function_takes_a_limit_operator`, semantics.rs): an OPFUNCTION or trig function, bare or
scripted, or an operator's nest over one, right before a limit-type operator's application — or a derivative's of one —
takes it as its argument: `\log\det\Sigma` log@(det@(Σ)), `\frac12\log\det(2\pi e\Sigma)` (1/2)·log@(det@(2πeΣ)),
`\cos\sup_t u` cos@(sup_t(u)), `\log\inf_x u`, `\sin\partial_x\det A` sin@(∂_x(det A)), `\nabla_x\log\det(A)`
((∇_x)@(log))@(det@(A)), `\operatorname*{arg\,max}_s\log\det(L_s)`, `\mathbb{E}\log\det\Sigma` 𝔼@(log@(det Σ)),
`\max_{|\alpha|\le m}\ \sup_{x,v}(\dots)` (max_α)@(sup_{x,v}(…)), `\Im\lim_{\varepsilon\to0}` Im@(lim…). Delta A/B 57cj.8 →
57cj.9: 243 formulas in 55 of the 3,003 papers read so, most `\log\det\Sigma` in ML papers (2605.00130, 2605.26554, 2605.02883
`\log{\det}_\epsilon`, 2605.05081 and 2605.28059 max-before-sup, 2605.28932 `\Im\lim`; earlier witnesses 2605.03984,
2605.24401, 2605.25592, 2605.14289; 4 of the 9,420 corpus ∂ formulas). The grammar derives the product once, as for any big operator, and the action reads it — no rule
is added (the precedent of `expectation_takes_the_big_operator`, #379). A sum or an integral stays a factor of its own
(`\log\sum_i x_i` log·∑…, as Perl), and a trig function keeps Perl's product when the operator's operand crosses its
argument's end (`\sin\det A\,y` sin·det@(A·y), `\cos\sup_t u\,v`, `\sin\lim_n a_n\,b`; #367). 57cj.10 (57cj.9 review): a
function's bare application to a function takes it too, inside (`take_a_limit_operator`): `\log\log\det A` log@(log@(det A)),
`\min_\theta\log\det\Sigma_\theta` min_θ@(log@(det Σ_θ)), `\log\max_i\det A_i`, `\log\exp\sup_x f` (2605.14289); a trig
function's argument takes an OPFUNCTION before a big operator (`\sin\log\det A` sin@(log@(det A)); a sum or an integral the
product, `\sin\log\sum_i x_i` sin@(log·∑…); Rust-only unparsed before, Perl sine@(logarithm)·…). Not a word that qualifies
the operator (`qualifies_the_limit_operator`, 57cj.11 narrowing 57cj.10's `is_a_limit_qualifier`): `\arg` — or
`\operatorname{arg}`, its spelling (57cj.13, the 57cj.12 review: `\operatorname{arg}\sup_x f` arg·sup_x f as Perl, was
arg@(sup…)) — right before an infimum or a supremum — `\arg\inf f(\theta)` means one arg-inf, so it keeps Perl's argument·inf@(f(θ)) (2605.30648,
2605.16560; the take read the complex argument of an infimum) — and `\operatorname{ess}` before every limit-type operator
(`\operatorname{ess}\sup`, `\operatorname{ess}\limsup`, and since 57cj.12 `\operatorname{ess}\lim_{x\to a}f(x)`, `\operatorname{ess}\det A`
ess·det(A)): ess is never a function of a value, only a qualifier (the 57cj.11 review; 57cj.11 read ess@(lim…), ess@(det A)).
Before any other limit-type operator `\arg`, the complex argument, is a function and takes it: `\bar\theta=\theta-\arg\det M_q`
argument@(det M_q) (the strong-CP phase), `\exp(i\arg\det U)`, `\arg\lim_{z\to0}f(z)` (57cj.10 read products). Before a
minimum or a maximum — OPFUNCTIONs, no limit-type operator — both words apply, as Perl: `\operatorname{ess}\max_x f`
ess@(max_x f), `\arg\min_x f(x)`. The qualifier is judged at the head right before
the operator, so an outer one does not stop the take: `\arg\min_x\log\det\Sigma_x` argument@(min_x@(log@(det Σ_x))) (57cj.10
argument@(min_x@(log))·det Σ_x). Residual (SYNC): one arg-min/arg-inf operator is not modelled — `\arg\min_x f(x)` reads
argument@(min_x(f(x))) (older; 1,824 Math `tex=` in the 3,003 A/B papers by the 57cj.10 review's count, 2605.31100).

**Guard**: golden `tests/parse/bigop_operands.tex` ("A function takes a limit-type operator's application"),
`tests/parse/operator_application.tex`, `tests/parse/opfunction_arguments.tex` (`\log\det A`).

### 391. A note is its mark in the line it sits in; a note text or margin note is nothing

**Perl** sizes only the kernel's `\lx@note` by its mark (`sizer => '#mark'`, latex_constructs.pool.ltxml:460); every
other note constructor — `\lx@notemark`, `\lx@notetext` (:467, :478), `\marginpar` (:3487), plain's `\footnote{}{}`
(plain_constructs.pool.ltxml:147), amsppt's and revtex4's notes (amsppt.sty.ltxml:268/:290, revtex4_support.sty.ltxml
:123/:145) — is measured by its arguments: its body, and for `\footnotemark` the counter's name.

**TeX**: a note's text is an insertion, set out of the line (tex.web §1100 `begin_insert_or_adjust`), and so is a
margin note (a float); in the line a footnote is its mark. The HTML agrees: a note renders as its mark with the text in
a popup (`.ltx_note_content{display:none}`, LaTeXML.css:494).

**Rust** (58b): `note_size_in_line` (the `mark`, or plain's `prenote`; nothing without one) for every note that puts a
mark in the line — `\lx@note`, `\lx@note@live`, `\lx@notemark`, plain `\footnote{}{}`, amsppt `\footnote`/`\footnotemark`,
revtex4 `\endnote`/`\endnotemark`; `out_of_line_size` (nothing) for the texts and margin notes — `\lx@notetext`,
`\lx@notetext@live`, `\lx@marginpar`, amsppt `\footnotetext`, revtex4 `\endnotetext`, deluxetable `\tablenotetext`,
pdfcomment's notes, mdpi `\endnote`, elsart `\thanks`/`\note`, mn2e queries, acmart `\Description`, slides
`\addtime`/`\settime`. Not all of these are insertions in the PDF: mn2e's queries print inline (`\aquery` =
`\mbox{\largerm {[#1]}}`, texmf-dist/doc/latex/mnras/LEGACY/mn2e.cls:1469; `\hbox{A\aquery{…}B}` 138.7pt for 13.5pt), as elsart's `\thanks` prints a mark; the HTML
shows each as a † with its text in a popup, so they measure as nothing — a render-driven surpass, not TeX's insertion.
Measured with its body a note holding a `\par` added its paragraphs' lines to the line; measured
as its `tags` it counted every registered tag form (hyperref + cleveref: a `\resizebox`d tabular with a footnote at half
size); `\hbox{A\footnotemark B}` was 50.69pt for pdflatex's 19.07pt. Witnesses 2605.18427 S3.T4 (natural width 802.6
→ 556.7pt, pdflatex 521.7), 2605.14713 S2.T1 (aa `\tablefoot`). Residuals: a note still breaks a measured line; plain
TeX's `\footnote` (plain.tex `\insert\footins`) is measured with its body; a minipage footnote's text, which LaTeX sets
inside the box (`\@mpfootins`), is not counted (consistent with the popup). Guard
`perfect_kernel_batch58::notes_measure_as_their_mark`.

### 392. A size switch sets the leading; `\begin{document}` runs `\normalsize`

**TeX/LaTeX**: `\@setfontsize#1#2#3` (latex.ltx:14103-14107) is `\fontsize{#2}{#3}\selectfont`; its `\size@update`
sets `\baselineskip` to #3 × `\baselinestretch` (and `\normalbaselineskip`, `\strutbox`). Every size command of the
standard classes goes through it (size10.clo:48-86), and `\document` runs `\normalsize` (latex.ltx:9497).

**Perl**: the class bindings' size commands only set the font size (`font => {size => …}`, article.cls.ltxml), and
`\begin{document}` runs no size command, so `\baselineskip` keeps whatever the preamble set, for every size.

**Rust** (58c): a pure size switch (`binding/def/dialect.rs`) is followed by `\fontsize{<size>}{<leading>}` before its
`\selectfont`, the leading being size10.clo's for that size (`size10_leading`; 1.2 × the size otherwise), and
`\begin{document}` digests `\normalsize`. With sizing by lines (58b), a preamble `\setlength{\baselineskip}{1.5cm}`
had stacked a `\small` tikz node's lines 1.5cm apart (witness 2605.29990 S4.F7.pic1: 99.93pt for pdflatex's 39.26; now
36.57). A parameterless size switch is robust (`\small` is a macro for the primitive `\lx@size@small`, as size10.clo's size commands expand), so
the preamble idiom `\expandafter\def\expandafter\normalsize\expandafter{\normalsize …}` and `\appto\normalsize` keep the
switch (an unexpandable primitive made the new `\normalsize` call itself: 2605.03010, 2605.19623, 2605.30610); the primitive is `\lx@size@<name>`, not `\<name>␣` (the name `\DeclareRobustCommand` writes to), so `\NewCommandCopy`/`\LetLtxMacro` + a robust redefinition calling the copy works as in pdflatex — and a plain `\let` copy, which loops in pdflatex (input stack overflow), converts; the
nominal size is whatever `\normalsize` gives at `\begin{document}`; `\linespread` (latex.ltx:10525) and setspace's
`\setstretch`/`\onehalfspacing`/`\doublespacing` (stretch by `\@ptsize`), its options and environments set `\baselinestretch` (Perl: no-ops); a floating environment's text starts in the body font, size and leading (latex.ltx `\@floatboxreset` = `\reset@font\normalsize`: figure/table, `\@float`, float.sty, rotating, acmart, jhep, aas plates, wrapstuff, an algorithm2e not placed `[H]` — not sub-floats, wrapfig/floatflt, supertabular, listings; setspace single-spaces it, setspace.sty:418-422); a note's text is single-spaced under setspace (the leading only, LaTeX's `\footnotesize` is not applied: a `\footnote` is at the text-default size, `neutralize_font`, its leading the surrounding size's — FN 12pt, `\small` 11pt, for pdflatex's 9.5pt). Residuals: setspace also single-spaces float.sty `[H]` floats and wrapstuff, which call `\@floatboxreset` directly and keep the stretch in pdflatex; a floating `lstlisting[float]` is not reset (its caption is at normal size in pdflatex).
Residuals: the bindings' sizes are size10.clo's whatever the class option (at 11pt pdflatex's leading is 13.6pt, ours
12pt; beamer's default is size11); class bindings with their own size tables (amsart `\Large` 14 in pdflatex, 18 here;
slides, IEEEtran, acmart) get size10's leadings. Guards `perfect_kernel_batch58::{size_switches_set_the_leading,
size_commands_can_be_appended_to, line_spacing_commands_set_the_stretch}`.


### 393. A BINOP keeps its juxtaposed operand whole

Perl reads a BINOP as a MulOp (`MulOp : BINOP addOpDecoration`, MathGrammar:688), so it takes one factor on its right
and a juxtaposition after it multiplies the result: `a\boxast b c` ⁢(⧆(a,b),c), `KX\mathbin{\|}(I-K)X` ‖(KX, I−K)·X. A
BINOP is a binary operator of unknown meaning: `\mathbin{…}`, the stmaryrd box operators, every
`\DeclareMathSymbol{…}{\mathbin}` symbol of a raw-loaded package (sect08.rs), a raw `\mathchar`/`\mathchardef` of
class 2 (latexml_core common/mathchar.rs) and a `\Udelimiter` of class 2 (latex_constructs_rust_only.rs).

**Rust** (57cj.19.2-57cj.19.8, `infix_apply_nary`): a BINOP's right operand is the juxtaposed product whole — ⧆(a, b c),
‖(KX, (I−K)X) — as before 57cj.19 (the one-factor regroup is a MULOP's only: `a\cdot b c` (a·b)·c, as Perl). Perl's own
grammar says "BINOP can never really be satisfactory; it comes from something marked as \mathbin; we don't know any more
about it" (MathGrammar:690-691), and the corpus agrees: the 57cj.19.1 A/B (km191, m23 → m24, 3,003 papers) that gave a
BINOP the one-factor rule changed 11 formulas in 6 papers, every one to a wrong reading — concatenations
`KX\mathbin{\|}(I-K)X` (2605.31129) and `[WX_i\mathbin{\|}WX_j\mathbin{\|}w_{ij}]` ‖(‖(WX_i, W)·X_j, w_ij) (2605.31315,
2605.08689), a map arrow set as a `\mathbin` (2605.26237), an inner product (2605.25490), temporal operators (2605.30618).
Before an operand that opens with an integrand — a factor that is no differential — and holds an integral's differential
after it — a `d`-kind differential's application, not a differential operator's (`KX\mathbin{\|}Y\partial_t u` stays ‖(KX, Y·∂_t u), `\int f\boxast g\,\partial_t u`
⧆(f, g·∂_t u)): a bare `d`, which the lexer offers as one only in an integral's operand (`util::in_an_integral_operand`;
`a\boxast b\,dx` stays ⧆(a, b·d·x)), or a bound differential, one anywhere — iopart's `\rmd`, elsart's `\d` (meaning `differential-d`),
physics' `\dd`/`\differential` (meaning `differential`), braced or raised too (`\dd{x}`, `\dd[3]{x}`, `\dd^2 x`; not a
variation, `\variation x`, which stays in the operand where Perl splits it off) — a BINOP takes the integrand's factors
before the differentials, its juxtaposed operand, as a large MULOP does (#396; user ruling 2026-10-01, which replaced
57cj.19.3-57cj.19.8's one factor, Perl's): `\int f\boxast g\,dx` ∫((f⧆g)·dx), `\int f\boxast g h\,dx` ∫((f⧆(g h))·dx) (was
∫((f⧆g)·h·dx), Perl's), `\int f\mathbin{\#}g h\,dx\,dy` ∫(#(f, g h)·dx·dy), `\int_X f\boxast g\,d\mu(x)` ∫((f⧆g)·dμ·x), `\int f\boxast g\,dx\,(1+h)`
(`integrand_split`, `integrand_before_differentials`; 57cj.21; A/B km21: 0 corpus readings changed, latent); outside an integral a bound differential closes the operand too,
`a\mathbin{\#}b c\,\dd x` #(a, b c)·dx (was (a#b)·c·dx). An operand that opens with a differential closes no integrand
and stays whole, a bare `d` or a bound one (57cj.19.6-57cj.19.8): `a\mathbin{\#}\dd\omega\,\eta` #(a, dω·η) (an exterior
derivative; Perl #(a, d(ω·η))), `a\mathbin{\#}\dd\omega\,\eta\,\dd x` #(a, dω·η·dx) (Perl #(a, d(ω·η·dx))),
`\int f\mathbin{\#}\dd x\,g\,\dd y` ∫(#(f, dx·g·dy)) (Perl ∫(#(f, d(x·g·dy)))), `a\mathbin{\#}\dd x\,\dd y` #(a, dx·dy), where
Perl's one-factor rule splits the braced and `\qty`-fenced physics forms (`\int f\mathbin{\#}\dd{x}\,\dd{y}` ∫(#(f, dx·dy)), Perl
∫((f#dx)·dy); `\int f\mathbin{\#}\qty(\dd x)\,g` ∫(#(f, dx·g)), Perl ∫((f#dx)·g); `\dd[3]{x}\,g\,\dd y`, `\dd{x}\,g\,\dd{y}`,
`\qty(\dd x)\,g\,\dd y` read Perl's split on 57cj.19.7) and
a bare `d`: `\int f\boxast dx\,g` ∫(⧆(f, dx·g)), `\int f\boxast dx\,dy` ∫(⧆(f, dx·dy)), `\int f\boxast dx\,g\,dy`
∫(⧆(f, dx·g·dy)) (Perl ∫((f⧆dx)·g), ∫((f⧆dx)·dy), ∫((f⧆dx)·g·dy)); an integrand before the BINOP is no matter,
`\int dx\,f\boxast dy\,g` ∫(⧆(dx·f, dy·g)) (Perl ∫((dx·f⧆dy)·g)), and `\int dx\,f\boxast g\,dy` ∫((dx·f⧆g)·dy), as Perl. A raised bare `d` before its variable is
the volume element since 57cj.20 (#395), a differential as `\dd^2` and iopart's `\rmd^2` are: `\int f\boxast g\,d^2x`
∫((f⧆g)·(d²)@(x)), and an operand that opens with it stays whole, `\int f\boxast d^2x\,g\,dy` ∫(⧆(f, (d²)@(x)·g·dy)); Perl's
`IntFactor` has no raised `diffd` (MathGrammar:643-651: `diffd ATOM_OR_ID`, the atom right after a bare `d`) and reads
∫((f⧆g)·d²·x), ∫(⧆(f, d²)·x·g·dy) (57cj.19 read ∫(⧆(f, g·d²·x)) and Perl's split).
Around a big operator a BINOP is a MulOp, as Perl: before
one (`a\boxast\sum_i b_i c_i` ⧆(a, ∑(b_i c_i)); unparsed before 57cj.19.2) and in its wider operand (`\sum_i a_i\boxast b_i`
∑(a_i⧆b_i), was ⧆(∑a_i, b_i); `pragma_bigop_prefer_wider_absorption`'s `is_product`). A trailing ellipsis after a trig
argument leaves it (user ruling 15): `\sin x\boxast\cdots` ⧆(sin x, ⋯), Perl sin@(x⧆⋯), as `\sin x\cdot\cdots` sin@(x)·⋯.
Q11 (user ruling 2026-10-01) gives the large MULOPs (⊗, ⊙ and their circled and boxed family) the same whole operand, #396.
Guards `tests/parse/product_chains.tex`
("A BINOP, a different operator of the same meaning, a coloured operator"), `tests/parse/physics_package.tex` (the physics
differentials).

### 394. `\font` fires a pending `\afterassignment` token

tex.web §1269: after an assignment, the token saved by `\afterassignment` is put back into the input — after the whole
`\font`, so `\def\foo{[AA]}\afterassignment\foo\font\x=cmr10 at 12pt {\x Hello}` typesets "[AA]Hello". Perl's `\font`
(TeX_Fonts.pool.ltxml:82-119) never calls `AfterAssignment`, so the token stays pending: Perl drops it, and Rust (before
58g) emitted it wherever the next assignment fired it — a stray `<p>[AA]</p>` at the end of the document. Rust's
`\font` fires it after installing the font (tex_fonts.rs). Users: luatexko.sty:495-499 (`\hangulfont`, `\hanjafont`,
`\fallbackfont`), ptb-fonts.tex:45 (`\applyfontcode`). Guard `perfect_kernel_batch58::font_name_is_defined_before_its_size`.

### 395. An integral's raised `d` before its variable is the volume element

Perl's `IntFactor` reads a differential only from a bare `d` and the atom after it, scripts on the atom included
(`diffd ATOM_OR_ID addScripts`, MathGrammar:643-647), so a raised `d` is a letter's power: `\int d^3x\,f` integral@(d³·x·f),
`\int d^nx\,f(x)`, `\int\mathrm{d}^2r` — the volume element of physics papers (`d^4x\sqrt{-g}`, `d^dx`, `d^3k`), read as a
power of a variable `d` times `x`. A bound differential raised the same way is one (iopart's `\rmd^3x` (differential-d ^ 3)@(x),
physics' `\dd[3]x`), as are Perl's.

**Rust** (57cj.20, 57cj.20.1; `raised_differential_d`, `differential_d_power`, `differential_power_apply`): an integral's `d`
(the lexer's `XDIFFUNK`, only in an integral's operand, `util::in_an_integral_operand`) raised to a count before a bare or post-scripted variable is the
differential's power applied to it, `(differential-d ^ 3)@(x)`, the markup of `\rmd^3x`: `\int\!d^3x\,f` ∫((d³)@(x)·f),
`\int d^2x\,d^2y\,f`, `\int d^nx_1\,f`, and as an integrand's last factor `\int f\,d^2\mathbf{r}`. A count
(`is_a_power_count`) is a number, a single-character letter other than the transpose `T` (a named one is not: `d^\alpha x` stays d^α·x), or a sum, difference or product of counts
(`d^{d-1}x`, `d^{2N}x`); not a power: a prime (`d'x`), a star (the codifferential `d^*`), a dagger, a negative number
(`d^{-1}x`), a group (`d^{(2)}x`, an order as `x^{(2)}` is), a symbol (`d^\infty`, the Haar measure's `\mathrm{d}^\natural h`,
2605.01646), an accented letter (`d^{\hat n}`), the transpose (`\exp(-d^{\mathsf T}x)`), a subscript (`d_1x`, `d^2_xy`), and — a residual — a
subscripted count, whose script the lexer spells as one atom (`\int{\rm d}^{n_{\mathrm{lin}}}\theta_{\mathrm{lin}}` d^{n_lin}·θ,
2605.21436), and a decorated measure in an operand (the Haar measure `\int…\,\mathrm{d}^\natural h` d^♮·h, 2605.01646: the integral
keeps no differential), as Perl reads both. Only in
an integral's operand as the tokens show it — an INTOP before the `d` at its level with no relation, arrow or wide
punctuation between (`util::in_an_integral_operand`, the lexer's test for every letter `d` since 57cj.21, SYNC (17); where Perl reads
`diffd` at all, `moreIntOpArgFactors`, MathGrammar:633-638): a dimension stays a letter's power before the
integral sign, `L_3^2d^2h^2\int_t^{t+h}…` d²·h² (2605.07939), `\leq 9\tilde L_f^2d^2h\sum\int…` (2605.26800), and after a
relation, `\int f\le C d^2 n`, `\int f=O(d^2n)`; the 57cj.20 review found the corpus rows read (d²)@(…) on 57cj.20. A relation
inside the integral's limits or a script is no end (the lexer makes it an atom): `\int_{s<u<t}d^2x\,f` keeps (d²)@(x); nor is one
inside a group closed before the `d` (`\int\mathbb{1}\{x\le y\}\,d^2x` (d²)@(x) since 57cj.21, was d²·x). Ranked with the bare differential
(`LetterDsBeforeVariablesAreDifferentials`). Corpus: 660 formulas in 79 of the 3,003 A/B papers read a `d ^ n * x` product
(m58c; 2605.29990 `\int d^{4}y`, 2605.21314 `\int\!d^{4}x`, 2605.23046 `\int\mathrm{d}^{2}r=\int_{-\infty}^{+\infty}\mathrm{d}^{2}r`,
2605.30421 `\int d^{d}xd\tau`); the km20 A/B (m58g → m35) changed 476 readings in 51 papers this way. A fraction's numerator is
a formula of its own with no INTOP, so `\int\frac{d^3k}{(2\pi)^3}` keeps d³·k, as `\frac{dk}{2\pi}` keeps d·k. Guard
`tests/parse/integrals_and_differentials.tex` ("An integral's differential takes a scripted variable").

### 396. A large product operator keeps its juxtaposed operand whole

Perl reads every MULOP with `moreFactors` (MathGrammar:252-258: `MulOp Factor`, each `ApplyNary`d on the product so far), so
an explicit MulOp takes one factor on its right and a juxtaposition after it multiplies the result: `a\otimes 2b` (a⊗2)·b,
`2\Lambda_1\otimes\Lambda_1\otimes 2\Lambda_1\otimes\Lambda_1` (((2Λ₁⊗Λ₁⊗2)·Λ₁)⊗Λ₁, `g\otimes w\otimes\sigma'(\gamma_i)` (g⊗w⊗σ′)·γᵢ,
`a\otimes_k DB` (a⊗_k D)·B. Rust read so from 57cj.19 (SYNC "Math-parse residuals" (14)) until the ruling.

**Rust** (user ruling Q11, 2026-10-01: juxtaposition binds tighter than a large MULOP; 57cj.20.Q11, `is_a_large_mulop`,
`infix_apply_nary`, `apply_invisible_times`): a large product operator — ⊗ `\otimes` (tensor-product), ⊙ `\odot`
(direct-product) and the circled and boxed operators of the same size, ⊘ `\oslash`, ⊚ `\circledcirc`, ⊛ `\circledast`,
⊠ `\boxtimes`, ⊡ `\boxdot`, stmaryrd's ⦸ `\varobslash`, and (the Q11 scope ruling, 2026-10-01; 57cj.21) the semidirect products
⋉ ⋊ ⋋ ⋌, the coproduct ∐ (`\amalg`, U+2210), the circles ○ `\bigcirc` and ◯ `\varbigcirc`, mathabx's box product □ `\square` and
`\pluscirc` (2605.11552 `\pi_1(T^3)\rtimes\pi_1(N_0)` (π₁T³)⋊(π₁N₀), 2605.15276, 2605.27086 `\operatorname{Diff}(M)\ltimes\Omega^2_{\mathrm{sym}}(M)`;
2605.12221 a control; A/B km21: 9 readings in 7
papers, 7 better, 1 neutral, 1 worse where a `\text{ and }` connective joins the operand, 2605.22466), bare or decorated — keeps
its juxtaposed operand whole, as a BINOP does (#393):
(2Λ₁)⊗Λ₁⊗(2Λ₁)⊗Λ₁ (2605.17901), g⊗w⊗(σ′·γᵢ) (2605.01702), `P\odot P\odot P_\theta(x|y)` P⊙P⊙(P_θ·(x|y)) (2605.00423),
`c\boxtimes T^{(r,n)}c` (2605.29990), `R\otimes_{\mathbb C}\mathbb C G` R⊗_ℂ(ℂG) (2605.14864), `a\otimes_k DB` a⊗_k(DB). In a bare
argument's chain, where the grammar joins the next item to the chain so far (`trig_arg factor_base`, `op_bare_arg
op_bare_next`), the large MULOP's application takes the juxtaposed item into its last operand: `\sin x\otimes y z`
sin@(x⊗(y z)), `\log a\otimes b c`, `\nabla u\otimes v w`, `\sin x\otimes 2\pi t` sin@(x⊗(2πt)) (Perl sin@((x⊗y)·z)) — not a
differential, and an ellipsis only between two items (`\log x\otimes y\cdots z` log@(x⊗(y·⋯·z)); a trailing run leaves,
`\log x\otimes y\cdots` log@(x⊗y)·⋯, user ruling 15).
Before an integral's differentials — or the letter `d` of one, so the letter twin meets the differential's reading at the
same shape (`\int_0^1 f\otimes g h\,dx` read ∫(f⊗(g·h·d·x)) in the first cut) — it takes the integrand's factors before them,
one juxtaposed operand:
`\int f\otimes g h\,dx` ∫((f⊗(g h))·dx) (Perl ∫((f⊗g)·h·dx); a BINOP too since 57cj.21, #393); an operand that opens
with a differential stays whole, `\int_X F(x)\otimes d\nu(x)` ∫(F(x)⊗(dν·x)) (2605.25146). `\cdot` keeps Perl's one-factor
chain grouping (57cj.19), `\circ` (f∘g)·x, and `\times`, `\star`, `\ast` their one factor (the semidirect products, ∐, the circles,
mathabx's □ and `\pluscirc` took one factor too until the Q11 scope ruling, 57cj.21: the 57cj.20.Q11 plan check counted 4
corpus formulas with a semidirect product that read better whole, 2605.11552, 2605.12221, 2605.15276, 2605.27086). The plan check's corpus count (km201 B outputs, 3,003 papers):
⊗ 344 formulas in 43 papers and ⊙ 60 in 25 split a juxtaposed operand at a term's level, ⊠ 35 in 2, and 11 inside a bare
argument (2605.17112, 2605.25134). Accepted ambiguity: an operator before a trailing group, `{\mathrm{tr}}\otimes
I_{\mathcal X}(a)` (2605.21982), now reads tr⊗(I_X·(a)); a Kronecker product before a vector, `A\otimes B\,x` A⊗(B·x), where
(A⊗B)x may be meant (latent, no corpus formula); math braces are transparent, so `{a\otimes b}c` and `\boldsymbol{a\otimes b}c`
read a⊗(b·c) (`\mathord{a\otimes b}c` keeps (a⊗b)·c); a mixed number in a bare argument's chain joins as a product,
`\log x\otimes 2\frac34` log(x⊗(2·¾)) beside `x\otimes 2\frac34` x⊗(2+¾). Guards `tests/parse/product_chains.tex` ("A large product operator keeps its
juxtaposed operand", "The semidirect products, the coproduct and the circles"), `tests/parse/large_mulops_mathabx.tex`.

### 397. A `pspicture` measures its declared size

`\begin{pspicture}(x0,y0)(x1,y1)` is a box x1−x0 wide and y1−y0 tall in pstricks (pstricks.tex `\pspicture`). Perl sets only
the rendering attributes (`pxwidth`/`pxheight`, pstricks_support.sty.ltxml:532-535), so it measures 0pt — `\settowidth`,
`\resizebox`, adjustbox and centring arithmetic read nothing, and floatrow gobbles a floatbox whose object is one
(floatrow.sty:366-369: frsample10's pstricks figures and their `\footnotetext` were lost). Rust gives the box its declared
size as typed cached dimensions (`pspicture_properties`, pstricks_support_sty.rs), as `{picture}` is typed (sect13.rs);
corners in either order give a positive box (pst-code-pspicture.tex:151-160); the rendered attributes are unchanged. A
`[shift=…]` is not applied (depth stays 0; Perl ignores it too). Guards
`perfect_kernel_batch58::{floatrow_keeps_an_object_it_measures_empty, pspicture_has_its_declared_size}`.


### 398. An integral's letter `d` stays a differential past a sign, a colon and narrow punctuation

Perl reads a letter `d` as a differential only among an INTOP's arguments (`addIntOpArgs`/`moreIntOpArgFactors`,
MathGrammar:620-647): a product of `IntFactor`s, which ends at an additive operator, a relation, any punctuation and never
enters a group. So `\int_M a+b\,dV` reads b·d·V, `\int|\mathrm dP-\mathrm dQ|` d·P−d·Q, `\int f\,P(y,dx')` and
`\int K\tilde\mu(ds,de)` read their measures' `d`s as letters.

**Rust** (user ruling (17), 2026-10-01; 57cj.21, `util::in_an_integral_operand`, applied by the lexer to every letter `d`, which
then offers its differential lexeme `XDIFFUNK`/`XDIFFID` only there): an INTOP before the `d` at its level, with no relation,
arrow or wide punctuation (`,\quad`, a `\qquad`) between at that level. The `d` keeps its differential reading — as before 57cj.21, whatever the grammar's tree for the integral (`\int f-g\,dx` reads
integral@(f) − g·dx, the Frobenius integral (∫H) : Q·dx: a differential outside its integral's tree, a residual) — past a sign (697 sum-integrand
sites in 129 of the 3,003 A/B papers: `\int_{\bar M}Q_t(w-1)+Qw_t\,\mathrm{d}V_{0,0}` 2605.17001, `\int|\mathrm{d}P-\mathrm{d}Q|`
2605.15207, `\int_s^t-V\,\mathrm{d}W_\tau-\frac12V^2\,\mathrm{d}\tau` 2605.17850), past a colon, a product in an operand (the Frobenius
`\int_\Omega\mathbf{H}:\mathbf{Q}_t\,d\mathbf{x}`, 2605.24758, 2605.12883, 2605.19415), past narrow punctuation (mostly a comma typed for `\,`,
`\frac1L\int_0^L\mathbf u\cdot\mathbf e_x,dx`, 2605.21567, 2605.04013), and into groups (the measures `\int f\,P(y,dx')` 2605.08485,
`\int_E K\tilde\mu(ds,de)` 2605.20593). A group closed before the `d` is skipped whole (parentheses, brackets, a `\left|…\right|` pair; a plain `|…|` does not pair
by position, `|\int f\,dx|\,d\pi` keeps differential-d@(π)): `(\int_0^1 f\,dx)\,d\pi` d·π, and a
relation inside it ends nothing (`\int\mathbb{1}\{x\le y\}\,d^2x` (d²)@(x)). Letters: a `d` before the integral sign, after a
relation, arrow or wide punctuation at the integral's level, in a script, or bound by a big operator in between until a sign
ends that operator's operand — the letter, a list item, or a relation's, relation chain's or condition's operand other than
its last (`\int f\,dx+\sum_{d=1}^D d\,w_d` d·w_d, `\sum_{1\le d\le D}`, `\sum_{d\mid n}`, `\sum_{d,e}`; `\sum_{i\le d}` binds `i`). A/B km21 (m58i → m49): 120 readings in 35 of the 3,003 papers lost a differential — 11 right, 92 ruling-conformant,
17 residual (SYNC (17)). The design review found the ADDOP stop of Perl wrong in
all but one corpus case (2605.26800's second `dh`, pinned) and the colon stop wrong in all 28. Guard
`tests/parse/integrals_and_differentials.tex` ("A letter d is a differential only inside an integral's operand").

### 399. A toc copy is digested with notes, `\label`, `\index` neutralized (Perl: digested as the heading, its notes stepping twice)

**LaTeX** typesets a heading or caption once; its toc/lof copy is written untypeset
(`\addcontentsline` → `\protected@write`, latex.ltx:17351-17362; `\@caption`, :17401-17412), with
`\label`, `\index`, `\glossary` gobbled (:17359-17361). **Perl** digests the toc copy as typeset
material (latex_constructs.pool.ltxml:627-628, :3187-3190), so a `\protect\footnote` in a heading
steps the counter twice and a caption's note appears twice; every later note's number shifts
(KNOWN_PERL_ERRORS #417). **Rust** digests the copy under `\lx@toc@copy@neutralize` (the note
constructors, the current footnote hooks, `\label`, `\index`, `\glossary` gobbled by plain `o m`/`s o m` macros — LaTeX's ltcmd expandable
`\@gobble@om` left a constructor argument's frame open, 2605.13648), inside
`\lx@format@toctitle@@`'s group and around the direct `\@@toccaption` callers (listings,
algorithm2e). Witnesses 2605.15775, 2605.09284, 2605.19033 (20 of the 3,003 A/B papers). Guards
`perfect_kernel_batch58::{note_in_a_title_or_caption_steps_once,
footnotemark_in_a_heading_pairs_with_its_text, optional_label_after_a_caption_converts}`. Open: with
a `\tableofcontents`, pdflatex typesets the toc line's note as well, so an article's later notes
number one higher there (the TOC LaTeXML builds shows no note).

### 400. The German and French babel bindings define `\date<lang>` as their `.ldf` does (Perl: raw-loads the `.ldf`)

LaTeXML maps babel's `german`/`germanb`/`ngerman`/`ngermanb` and `french`/`frenchb`/`francais`
`.ldf` files to Rust bindings (`german_sty.rs`, `ngerman_sty.rs`, `french_ldf.rs`; the German raw
load made `\mdqoff` nondeterministic, latexml_package lib.rs). Their `\date<lang>` was an empty
stub, so `\today` stayed English under `[german]`, `[ngerman]` and `[french]` (Perl, which loads the
`.ldf`, prints the German date after `\selectlanguage{german}`; its babel reports the `german` and
`ngerman` options unknown). The bindings now define `\month@german`/`\month@ngerman` and the
dates as germanb.ldf:124-130 and ngermanb.ldf:107-113 do, and french.ldf:332-345's
`\StartBabelCommands*{french}{date}` strings (`\month<roman>name`, `\today` via `\FB@date`) with
`\FB@date`, `\FBdatebox`, `\FBdatespace` and `\frenchdate`. Guard
`perfect_kernel_batch58::today_follows_the_babel_language`.


### 403. Number scanning keeps the token after the signs in hand; a skipped conditional reports where it started (Perl: re-reads the token per kind; reports where the skip began)

Perl's `readNumber` (Gullet.pm:852-858, :896-906) puts the token after the signs back and reads
it again for each step (`readNormalInteger`, then `readRegisterValue('Number', …, 1)` with
coercion), and `readDimension` likewise (:964). TeX's `scan_int`/`scan_dimen` (tex.web §440, §448)
test the token in hand. The Rust
`read_number`/`read_dimension` (gullet.rs, `scan_optional_signs`, `normal_integer_from`,
`register_value_from`) follow TeX — one read per number instead of up to six, the l3fp loop of
pgf-interference −11.9 % instructions — with two behaviours kept from Perl: a register read
without a value consumed its tokens, so the next kind reads a fresh token; and an undefined
control sequence, whose own read installs its error stub (`\iffoo` `\let` to `\iffalse`,
State.pm:537-545), is put back once so it is read and expanded again, in `read_number` and `read_dimension` alike,
and `read_factor` reads a digit run's end token again as Perl's `readFactor` does (Gullet.pm:876-880)
(`gullet::became_expandable`; guards `perfect_kernel_batch59::{undefined_conditional_in_a_number_expands,
undefined_conditional_in_a_dimension_expands, undefined_conditional_after_digits_expands}`). One diagnostic differs:
"Missing number" names the token TeX met (`\ca= &` in a cell: `Next token is T_ALIGN[&]`, Perl
the cell-end macro its second read produced). The conditional skip's "conditional fell off end"
error names the conditional's own start (its `IfFrame` locator), where Perl reports the position
at which the skip began (Conditional.pm:124); the stack itself is a typed `State::if_stack`
(Perl's `if_stack` Value, edited in place, never grouped).

### 404. pgfmath results take the form live pgf gives them; `?:` and `&&`/`||` follow pgf's precedences (Perl: Perl `eval`'s numbers and precedences)

Perl's pgfmath binding evaluates an expression with Perl `eval` (pgfmath.code.tex.ltxml:363-382)
and prints every number as a real (`10.0`, `7.0`); its precedences are Perl's (a right-nested
`?:`). The Rust evaluator follows pgf (pgfmathparser.code.tex:898-916; probes against
lualatex/pgf TL2025): `?:` binds looser than `&&`/`||`, which bind looser than the comparisons,
and chained ternaries group to the left (`1?2:3?4:5` is 4); a result keeps the form pgf leaves it
in — an integer literal (`7`, `(7)`, `0x1F` 31), a comparison, a logic operator or an integer
function (`int`, `sign`, …) prints as an integer, arithmetic, units, registers and real functions
as a real (`2+3` 5.0, `-1` -1.0), and `?:`, `ifthenelse` and `array` return their chosen operand's
form; a user-declared function's result keeps the form its body leaves `\pgfmathresult` in, and its
integer arguments are passed as integers (a non-numeric result is reported once and read as 0). Perl prints `[10.0][7.0][2.0][1.0]` for `3>2?10:20`, `(7)`, `1?2:3?4:5`,
`ifthenelse(2>1,1,0)`; pgf and Rust `[10][7][4][1]`. The integer form matters downstream: an
`\ifnum` on a result read `1.0` as `1` followed by text (pgf-spectra, `ifthenelse` in
`\pgfmathsetmacro`). Guards `pgfmath_golden_tests::ternary_and_logic_follow_pgf_precedence`,
`perfect_kernel_batch59::pgfmath_ternary_after_a_comparison`. Not yet followed: `random(m,n)` is an
integer in pgf; pgf passes `.5` and `1.` through unchanged; an integer result is printed in canonical
form (`07`, `+5` print `7`, `5`).

### 405. What code appends to `\@author` after `\author` is the author block, kept (Perl: dropped)

LaTeX's `\@author` is read only by `\@maketitle` (article.cls:247), so a template that grows it after
`\author` — `\g@addto@macro\@author{\\\textsuperscript{1}Affiliation one}` from its `\address`,
`\correspondence`, `\email` (the hindawi template, hindawi_latex_template.tex:57-62;
cjs-rcs-article.cls:361 `\affil`) — prints those rows in the author block. Perl's locked `\author`
digests the creators at once and `\maketitle` empties `\@author` unread
(latex_constructs.pool.ltxml:1076-1079, :1099-1107), so the rows are lost. The Rust `\author` records
what it stored (`\lx@author@handed`), and at `\maketitle` `\lx@author@tail` (frontmatter_stores.rs)
hands what follows that value, token for token, to the creators that call made, as unlabelled
author-block rows (`role=authorblock`); an appending `\author` (a class redefined it,
cjs-rcs-article's `\author`/`\affil` pairs) hands each call's tail before storing the next
(`\lx@author@flush`). A class binding that builds `\@author` itself (revtex, acmart) never records
a value, and a tail holding `\and` (more authors) is left alone. Guards
`perfect_kernel_batch59::{author_block_appended_after_author_is_kept,
author_block_follows_each_author, author_block_after_a_replaced_author,
author_block_orphan_mark_is_kept}`. Open: the template's `\addresslink` marks (`\let` to
`\textsuperscript`) are not recognized as author marks, so its rows annotate every author; a marked
row is placed after the unmarked ones; a tail holding `\and` is dropped (pdflatex prints more
authors); a stored value with `\and` gives its tail to every creator of the call (LaTeX prints it
under the last `\and` group only); JATS output drops `authorblock` contacts (LaTeXML-jats.xsl:188);
dtk.cls:374 builds `\@author` without `\author`.

### 406. A later `\maketitle` typesets the authors set after the last one (Perl: the first `\maketitle` disables `\author`)

LaTeX's `\author` redefines `\@author`; each `\maketitle` typesets the authors set at that moment.
article.cls's `\maketitle` then disables `\author`, but a class's own `\maketitle` may leave it live:
KOMA ≥ 3.12 (scrbook.cls:3263-3290) and udesoftec, which typesets its title page from
`\AfterEndPreamble` (udesoftec.cls:1230-1241) with the class default `\author{Max Mustermann}`
(:266), before the document's `\author`. **Perl**'s `\maketitle` disables `\author` unconditionally
(latex_constructs.pool.ltxml:1099-1111), so a later `\author` is typeset as body text and the default
author stays the only creator.

**Rust** (59e): `\maketitle` leaves `\author` live where the class's own does (`\maketitle:redefined`,
`\lx@maketitle@cleanup`), and an `\author` after a flush marks the authors that flush digested as
superseded; the next `\maketitle` that holds new authors drops them first thing
(`\lx@maketitle@supersede`), with their pending annotation stubs, restarts the counts the new batch is
numbered and labelled by (`supersede_digested_authors`, base_utilities.rs), and hands the class's
stores (`\@address`, `\@email`, which its `\@maketitle` reads again) to the new authors — a store set
after the new `\author` was handed to them when it was set and is not handed twice; one set before it
went to the replaced authors and is handed again. A store handed when it was set comes before the
re-handed ones (Alice's email before the address the class prints first). The latest `\maketitle`
wins, as for the title (#154). A `\thanks` in the new `\author` is its author's note: the digest of
an author line makes `\thanks` a note whatever its meaning, and the line split matches an alias of a
delimiter's own definition only, so an `\and` disabled by the last `\maketitle` matches itself — by meaning it split at
every `\relax` alias (`\protect`, a disabled `\thanks`; 59d, Rust-only: Perl matches delimiters by token,
#36). Open, shared with Perl:
under KOMA ≥ 3.12, which keeps `\thanks` live after `\maketitle` (scrbook.cls:3264), a body `\thanks`
after the title is inline text (pdflatex: a footnote). Without a later `\maketitle` nothing typesets the new authors again, and they
are added — as in Perl when a fallback placed the frontmatter; after a `\maketitle` Perl has disabled
`\author`, and the name is body text. Guards `perfect_kernel_batch59::{author_after_a_digesting_maketitle,
author_after_a_fallback_flush_is_added, author_superseded_keeps_contacts_with_owners,
superseding_maketitle_rehands_class_stores, thanks_in_a_superseding_author,
store_set_before_a_superseding_author, thanks_after_a_late_author_stays_the_documents,
relax_aliases_do_not_split_a_late_author}`; repro
`sectioning-frontmatter/class_default_author_is_replaced` (udesoftec: one creator, the document's).

### 407. An abstract ends the paragraph it interrupts (Perl: the text around it stays one paragraph)

LaTeX's `abstract` begins by ending the paragraph it interrupts: `\quotation`'s `\list` (`\@trivlist`,
article.cls:377-387), the titlepage variant's `\titlepage`, the twocolumn variant's `\section*`. Text after
`\end{abstract}` is a new paragraph. **Perl**'s `\begin{abstract}` is plain `\lx@begin@abstract`
(latex_constructs.pool.ltxml:1130), so `foo \begin{abstract}x\end{abstract} bar` keeps "foo bar" one
paragraph (after the abstract); in Rust, where text before an abstract is its cover (#246), "bar" moved ahead
of the abstract with it.

**Rust** (59f): `\begin{abstract}` and the kernel `\abstract`'s braced and brace-less forms begin with `\par`
(sect05.rs); a class's argument-taking `\abstract` (`\lx@add@abstract`) does not. Guard
`perfect_kernel_batch59::abstract_ends_the_open_paragraph` (repro
`sectioning-frontmatter/abstract_ends_the_open_paragraph`); witness: none (review of 59e).

### 408. A document's `\newcommand` wins over what the fallback class defined (Perl: ignored as a redefinition)

For a class with no binding both engines load OmniBus, a fallback that defines the commands many classes have —
among them convenience aliases such as `\let\abst\abstract` (OmniBus.cls.ltxml:216). **Perl** then treats a
document's `\newcommand{\abst}` as a redefinition and ignores it (`isDefinableLaTeX`,
latex_constructs.pool.ltxml:2512-2526; only an Info), so `\abst{a}` opens an abstract inside a formula and the
document's |a| is lost. Yet a document that compiles with its real class and `\newcommand`s a name shows that class
leaves it undefined.

**Rust** (59g): what OmniBus defines carries `DefinitionOrigin::Fallback`; `State::assign_internal` records the
names whose current meaning a fallback installed (`fallback_meanings`, until anything else assigns them), and
`is_definable_latex` treats such a name as definable when the document itself defines it (`DefinitionOrigin::Document`)
— for `\newcommand`, `\providecommand`, `\newenvironment` and `\@ifdefinable` — unless the OmniBus definition is
locked. OmniBus's semantic frontmatter commands are locked (user ruling 2026-10-01): `\doi`, `\orcid`, `\email`,
`\emailaddr`, `\ead`, `\address`, `\affil`, `\affiliation`, `\keywords`, `\kword`, `\kwd`, `{keyword}`, `{keywords}`,
`\classification`, `\correspondingauthor`, `\subtitle`, `\titlenote`, `\dedicated` keep their markup against any
later definition — a document's presentational `\newcommand{\orcid}` (2605.08630), `\providecommand{\keywords}`, a
package's deferred `\providecommand{\doi}` (apacite), a raw `\def\doi` (imsart.sty), a document's
`\renewcommand{\doi}` (unlike Perl, which lets those replace them); conveniences and aliases (`\abst`, `\etal`,
`\corref`) and common words (`\volume`, `\issue`, `\journal`) yield. arXiv A/B (3,003 papers, 59f2 -> 59g4):
2605.05619 errors 52 -> 0 and words 39,866 -> 53,885, 2605.13204 errors 6 -> 0 (its own `\corref`), 2605.13443 its own
upright `\etal`. Open: a package the paper ships (`\usepackage{mymacros}`) is a raw File, not the document, so its
`\newcommand` still yields (RED `singletons/omnibus_alias_shadows_a_local_package`); a group-local reassignment clears
the fallback mark for good. A raw class or package loaded beside
OmniBus (apa7.cls, whose `\doi` OmniBus reads as a URL) still yields to it, as raw files yield to bindings
(guard `perfect_kernel_batch56::omnibus_doi_reads_as_a_url`). A census (TeX Live 2025 and ~23,000 arXiv sources) found `\abst` defined as an abstract setter by
jpsj2.cls:841 and ptptex.cls:617 and shipped classes (jpsj3, pasj0x), which load raw; papers that define it as an
absolute value (2605.05619, 2605.05645, 2501.01582); and one paper that uses it undefined as an abstract
(astro-ph0003107, `PASJ95` not shipped), which the alias still serves. Witness 2605.05619 (its own
`siamart250211.cls`; 12 formulas). Guards `perfect_kernel_batch59::omnibus_alias_yields_to_newcommand`,
`perfect_kernel_batch56::omnibus_doi_reads_as_a_url` (the locked `\doi`).

### 409. A raw float end's `\@endfloatbox` closes only a box that was opened (Perl: it pops the float binding's frame)

latex.ltx:17538 `\@endfloatbox` ends with `\egroup\color@endbox`, closing the `\color@vbox` + `\vbox\bgroup`
that `\@xfloat` opens (:17478-17481). Both engines bind `{@float}`/`{@dblfloat}` as environments that open one
mode frame and never run `\@xfloat` (latex_constructs.pool.ltxml:3142-3155), yet the dump keeps the raw
`\@endfloatbox`, so a raw float end that calls it on the binding's frame — floatrow.sty:421/:436
`\float@end`/`\float@dblend` for every `\DeclareNewFloatType` — reports 3 errors per float ("Attempt to close a
group that switched to mode internal_vertical"). **Perl** keeps latex.ltx's raw `\@endfloatbox` from its dump
(latex_dump.pool.ltxml:1993) and binds none, so it errs the same.

**Rust** (59h): `begin_float` records the float's frame (`lx@float@frame`), and `\@endfloatbox` (sect09.rs) leaves
out its closers (the hook `\lx@endfloatbox@close`, after latex.ltx's own tokens, which `\patchcmd`/`\appdef`
still meet) when that frame is on top; a box a raw opener made on top of it (examplep.sty:1794-1807:
`\color@vbox\normalcolor\vbox\bgroup`…`\@floatboxreset`) is closed as LaTeX closes it. Open: changebar's
`\cb@end@float` opens a new raw box after `\@endfloatbox`, before the binding's end (changebar.sty:1234-1266).
Witness kaytannollista-latexia (55 floatrow floats, 165 -> 0 errors, XML otherwise identical). Guards
`perfect_kernel_batch59::{floatrow_new_float_type_closes_its_box, raw_float_box_keeps_its_closers}`.

### 410. A tcolorbox listing box is a listings environment that reads its own begin line (Perl: raw tcblistings through a temp file)

tcolorbox's listings library defines `\newtcblisting`/`\NewTCBListing` environments as xparse environments
(tcblistingscore.code.tex:300-343) that write the body to the listing file (`\tcbverbatimwrite`, :279-280;
tcolorbox.sty:2726-2737) and typeset it with `\lstinputlisting` (tcblistings.code.tex:45) inside a box whose frame
runs the box's phantom code — `hypertarget`, `label`, `step`, `index` (tcolorbox.sty:844-855, run by
`\tcb@set@@phantom` :1154-1161). **Perl** raw-loads tcolorbox and the library (tcolorbox.sty.ltxml:27), and the read
of that file fails (probe `tools/perfect_kernel/repros/expl3/tcb_listing_star_is_a_boolean.tex`).

**Rust** (`tcolorbox_sty::tcb_xparse_listing`) delegates the environment to `\lstnewenvironment`:
- its `[n][]` arity carries the mandatory arguments and one leading optional; every other argument (`s`, `t<c>`,
  `o`/`O`/`d`/`D`/`g`/`G`, `!`-modified) is read off the `\begin` line by listings' raw reader into a stand-in the
  options name (`tcb_grab_begin_line`, brace-aware like ltcmd; one that runs past the line is a counted Warning and
  its rest is listing text) — absent, the stand-in is the default or `\c_novalue_tl`;
- the options reach `\tcbset` as tokens; with begin-line arguments they resolve twice, first absent, then with the
  values read, so a global side effect in them (`code={\stepcounter…}`) runs twice;
- the phantom code runs at the start of the listing's first `listingline` (an empty listing gets one empty line for
  it): a `listing` holds only lines, and a `\label` there names the line.
Witnesses jsonparse-doc (7 -> 1 jing errors), 2605.21082 (two `label=`s on their listings), tutodoc-en/-fr
(`\begin{tdoclatex}<\tdoctcb{code}>` is `listing only`: its body is no longer typeset). Guards
`perfect_kernel_batch59::{tcb_listing_keeps_its_hypertarget, tcb_listing_label_names_its_listing,
tcb_listing_begin_line_edge_cases}`, `perfect_kernel_batch56::tcb_bang_leading_optional_reaches_options`.

### 411. An index entry is set in the neutral font of the text encoding, one brace level in (Perl: OT1, at the alignment's level)

Perl's `\@index` (latex_constructs.pool.ltxml:4409-4412) digests the entry after `neutralizeFont()`, which sets
the default font with the default encoding, OT1; and it digests the entry at the alignment level of the
surrounding text. LaTeX writes the entry to the `.idx` (`\@wrindex`, latex.ltx:17720-17726) and typesets it in
the index, in the document's encoding; the entry's `&` is other there (`\@sanitize`, :1778). **Perl** under
`[T1]{fontenc}`: `\index{a<b>c}` reads `a¡b¿c` in the phrase and its key; `\index{y&z}` in a tabular cell ends
the cell (KNOWN_PERL_ERRORS #423).

**Rust** (59k): the neutral font keeps the current text encoding (family, series, shape, size and colour still
reset, as Perl's), the entry's `&` is `\&`, and the alignment level is raised by one while the entry is digested,
as the argument's braces raise TeX's `align_state` (tex.web §358) — a tabular inside the entry still ends its own
cells. Witness robustsample (robustsample.tex:45/:59). Guard `perfect_kernel_batch59::index_entry_ampersand_is_literal`.

### 412. A g-brief letter's sender and addressee are frontmatter creators (Perl: lost with the first-page style)

g-brief.cls and g-brief2.cls print the sender's name, address, phone, internet, bank details and the recipient's
address only in `\ps@firstpage`'s `\@oddhead`/`\@oddfoot` (g-brief.cls:306-360, g-brief2.cls:304-426), set by
`\thispagestyle{firstpage}` at the start of `{g-brief}`. LaTeXML never typesets a page style (`\thispagestyle` is a
no-op, latex_constructs.pool.ltxml:998), so **Perl** (raw classes, 0 errors) and Rust before 59o drop every field:
beispiel2 kept 60.7 % of its PDF's words.

**Rust** (59o; user ruling 2026-10-01: page furniture stays dropped, a letter's sender block and recipient are kept
as frontmatter): the contrib bindings `g_brief_cls.rs`/`g_brief2_cls.rs` load the class raw and, at `\begin{g-brief}`,
add a `creator` of role `sender` (`personname` from `\Name`, then one `contact` per foot block — its lines joined by
`\\`, a blank store left out — named by the class's label where it prints one, `\adresstext`, and unnamed where it
prints none: the head's street and town and the recipient, so the kernel's default "Address:" is emptied in these
documents) and one of role `addressee` (`postal_note`, `address`). The fields' values are taken at the letter (the
frontmatter queue digests later, after a later `\Name`/`\Adresse`); g-brief.cls's bank block only with bank, BLZ and
account all set, as the class prints it; a return address only when the document set one; no sender creator when
nothing is set; `\HTTP` is role `http`, not `url` (a scheme-less value would become a relative link). The structure XSLT
renders the two roles in place as blocks, each label apart from its value (the plain `creator` template renders
nothing outside a title). Limitations: only the first letter of a serial document (RED
`sectioning-frontmatter/gbrief_serial_letters_each_have_their_frontmatter`; a document has one frontmatter); fields set
inside `{g-brief}` are missed; a label's ligatures (`e--Mail\/:`) are untypeset in `name` (RED
`sectioning-frontmatter/contact_name_label_keeps_its_ligatures`). Not taken: typesetting the page style (its `picture`
layout is the furniture the ruling drops), a cross-class "first-page style carries frontmatter" rule (only g-brief puts
fields there), K11's store table (these stores are read in the body too). Witnesses g-brief/beispiel2 (HTML recall
60.7 → 97.6 %), g-brief/beispiel (XML 71.4 → 91.4 %), cv/ApplicationLetter (85.7 → 96.7 %). Guard
`perfect_kernel_batch59::gbrief_letter_sender_and_addressee_are_frontmatter`.

### 413. A one-shot eso-pic overlay inside a class's title page is the title page's picture (Perl: dropped)

eso-pic's `\AddToShipoutPicture*` adds picture code to the next page shipped out, then clears it (eso-pic.sty:140-145,
:176-196); unstarred, to every page. **Perl**'s eso-pic binding (ar5iv-bindings eso-pic.sty.ltxml:21-23) makes every
overlay a no-op, so a class that draws its title page that way (uantwerpendocs exam/letter/coursetext: course, exam
session, rooms, lecturer, recipient, subject) loses it (PERL-ORIGIN, 0 errors).

**Rust** (59p; user ruling 2026-10-02: the one-shot title-page overlay is kept whole — its text, the university logo,
colour fills and an exam's blank name/docket boxes): a starred overlay digested inside a class's `\maketitle` replay
(`lx_depositing_frontmatter_fields`, #265) becomes a page-sized `picture` there — origin at the lower-left corner,
`\unitlength` 1pt, as the shipout hook's `\put(0,\ESO@yoffsetI)` places it — under its own diagnostics hold, kept only
when it raised no error (kdgcoursetext's overlay needs XeTeX fonts: dropped alone, the replay kept). Every-page overlays
(the exam's "Examiner Version" watermark) and a starred one anywhere else stay dropped as page furniture (2026-10-01).
An overlay that typesets nothing is dropped too (no empty page-sized picture). eso-pic's page positions are its `\put`s
(eso-pic.sty:32-38; `\AtStock*` = `\AtPage*` outside memoir): lni.cls:544-553 sets its DOI under
`\AtPageLowerLeft`; `\AtText*` (margins, twoside) stay no-ops. The overlay's tikzpicture is a picture nested in the page
picture: rendered upright since #414. Witnesses
(XML recall): uantwerpencoursetext-example 86.3 → 98.6 %, uantwerpenexam-example1 83.5 → 96.3 %, -example2 84.2 → 95.6 %,
uantwerpenletter-example 92.4 → 100 %, uantwerpenbamathesis-example 88.5 → 93.3 % (with #265's gate rules); phdthesis1/2
and kdgcoursetext unchanged. Guard `perfect_kernel_batch59::maketitle_replay_keeps_its_title_page_content`.

### 414. A picture nested in a picture keeps the enclosing picture's orientation (Perl: flipped about its mid-height)

**TeX** sets a nested `\begin{picture}` (or a tikzpicture) `\put` in a picture as a box with its lower-left corner at the
`\put` point: its content sits in the enclosing picture's y-up coordinates. **Perl**'s SVG post-processor flips every
nested `ltx:picture` again (`SVG.pm:191-197` `convertPicture`: `translate(0,h) scale(1,-1)`, mirroring it about its
mid-height: a `\line` at its bottom drawn at its top), and wraps a nested picture that is already SVG (a tikzpicture's
`svg:svg`, y-down) in a `foreignObject` flipped once more (`convertNode`'s else branch) — upside-down (PERL-ORIGIN).
**Rust** (59p, `latexml_post` svg.rs `convert_picture`): a nested picture of LaTeXML primitives is a plain group; one
that is already SVG is flipped back once and kept as SVG. Witness: uantwerpenexam-example1's title page (a tikzpicture in
eso-pic's page picture, #413) rendered mirrored, its fills and docket boxes lost in the `foreignObject`. In the same
module, a picture's text is black unless it says otherwise, as Perl's font stack starts (`SVG.pm:82`
`{ fill => 'black' }`): the picture's `fill="none"` is inherited, and every `\put(…){label}` was invisible in Rust's HTML
(a Rust-only loss, parity restored). Repro graphics-tikz/picture_put_tikzpicture_keeps_orientation; guards
`cluster_xslt_split::picture_svg_on_the_live_dom::{label_after_a_nested_picture_renders_in_the_svg,
tikzpicture_in_a_picture_keeps_its_orientation}`.
