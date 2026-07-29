# Oxidized Design — Intentional Divergences from Perl

[← OXIDIZED_DESIGN.md](OXIDIZED_DESIGN.md) · Deliberate breaks with Perl behavior, numbered. Code comments reference these as `OXIDIZED_DESIGN #N`.

> **Numbering note:** the `### N` numbers are load-bearing (referenced from `.rs` comments) and are kept verbatim. `#16` and the math-grammar entries `#7–#18` live in [OXIDIZED_DESIGN_MATH.md](../math/OXIDIZED_DESIGN_MATH.md); in particular the code-referenced **`#18` is the f(x) "Speculative function application"** entry there, *not* the "Source-Level Bindings" `#18` below.
>
> **`#76` is a RETIRED number, not an omission** — its entry was consolidated into `#74` and the number was deliberately not reused (see the placeholder in sequence below). Next free number: **#83**.

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

**Why this is safe.** A paper with an external `.bib`/`.bbl` carries no inline
`ltx:bibentry` in the main document at this point in the pipeline, so the extra
scan contributes nothing and the entry map is byte-identical. The scan runs
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

**Decision:** `CrossRef::fill_in_bibrefs` (`latexml_post/src/crossref.rs`) searches
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

**`require_package` hoists the load's meaning-delta past the bracket**
(`snapshot_top_frame_meaning_keys` + `hoist_top_frame_meaning_delta`), as
`tex.rs::def_autoload` already did for the mirror-image autoload failure (witness
1711.11576) — note that one is UNGATED, since an autoload fires from arbitrary
body depth and has no bracket to be inside. We keep LaTeX's *invariant*, not its *enforcement*: refusing the
load would discard divergence #63.

**Only our own brackets.** An author's group keeps real LaTeX's verdict —
`{\usepackage{amsthm}}` leaves `\theoremstyle` undefined in pdflatex and in Perl,
so hoisting there would emit *fewer* errors than Perl on an authoring mistake.
What separates them is *where* the bracket was opened, so the region is named
`subfile:<frame depth>` — Perl's own `section:4` / `label:foo` convention
(State.pm L965-975) — and activated by `standalone_sty.rs` right after its
`bgroup()` and by `import_sty.rs`'s `\lx@save@paths` inside the `{…}`. Activity
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

Two shapes have no image to use. The author's annotation is never dropped, so
it goes to the next best host — the enclosing float — as `aria:describedby`,
which supplements the name rather than replacing it, so the caption survives
either way. Both **`Warn!`**, since the result is second-best and the author
can act on it:

* **no `ltx:graphics` in the float** — a figure built from tabular, text or
  TikZ content (which `t/complex/acm_aria` is), a `table` float, or an empty
  one. There is no image to be an alternative to.
* **more than one** — a `\Description` is scoped to the whole float, so on a
  multi-panel figure it describes the ensemble. Making it panel 1's `@alt`
  would assert that one sentence is the alternative for one panel, a claim the
  author never made. The review says "the first image"; we narrow that to the
  case where "first" is also "only", where it is unambiguous.

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
