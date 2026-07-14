# Oxidized Design — Intentional Divergences from Perl

[← OXIDIZED_DESIGN.md](OXIDIZED_DESIGN.md) · Deliberate breaks with Perl behavior, numbered. Code comments reference these as `OXIDIZED_DESIGN #N`.

> **Numbering note:** the `### N` numbers are load-bearing (referenced from `.rs` comments) and are kept verbatim. `#16` and the math-grammar entries `#7–#18` live in [OXIDIZED_DESIGN_MATH.md](../math/OXIDIZED_DESIGN_MATH.md); in particular the code-referenced **`#18` is the f(x) "Speculative function application"** entry there, *not* the "Source-Level Bindings" `#18` below.

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

### 55. amsrefs inline bibliographies are collected (upstream drops them whole)

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

**Why this is safe.** A paper with an external `.bib`/`.bbl` carries no inline
`ltx:bibentry` in the main document at this point in the pipeline, so the extra
scan contributes nothing and the entry map is byte-identical. The scan runs
*after* the external documents, so a key defined both externally and inline
resolves to the inline one — matching upstream's own last-source-wins loop.

**Measured.** All 40 amsrefs papers in sandboxes 2605+2606 went from 0 rendered
references (100% loss, every citation dangling) to **1,482 references rendered
with zero dangling citations**. Witness 2605.01646 (23 entries), 2605.00783,
2605.03852.


### 56. A malformed `.bib` entry resyncs at the next `@` (upstream abandons the file)

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
