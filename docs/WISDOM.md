# Tactical Wisdom: Internal System Insights

Specialized analyses that led to correct patches. These are tactical insights
about the internals of latexml-oxide — not general skills, but specific
knowledge about how the system works that can prevent future mistakes.

---

## 1. DefMacro! double-packing: compile-time vs runtime pack_parameters

**Discovery:** The `Error:misdefined:expansion` warning fired on every document
for `\displaylines` (an alignment-template macro with `##\hfil` in its body).

**Analysis:** `DefMacro!("\\displaylines{}", r"..##..")` compiles the expansion
at build time via `compile_expansion!` in `tokenizeable.rs` line 31, which calls
`pack_parameters()`. This converts `##` → single `#` (PARAM) and `#1` → ARG(1).
The packed tokens are stored in the compiled binary.

At runtime, `def_macro()` → `Expandable::new()` (line 225 of `expandable.rs`)
calls `pack_parameters()` **again** unless `nopack_parameters: true` is set.
The second packing sees `#` (PARAM) followed by `\hfil` (CS) — the `#` is now
an alignment cell marker, not a parameter — and fires the warning.

**Fix:** All `DefMacro!` branches using `compile_expansion!` must set
`nopack_parameters: true` in `ExpandableOptions`. This is specific to
`DefMacro!` — `DefConstructor!` uses `compile_tokenize!` (no packing), and
`DefPrimitive!` uses closures/strings (no packing).

**Key insight:** Any macro whose expansion is pre-compiled at build time must
skip runtime packing. Check this whenever adding new compile-time expansion paths.

---

## 2. Font::merge() must NOT call specialize()

**Discovery:** `\font\mybf=cmb10` followed by `\mybf Hello` did not produce
`<text font="bold">Hello</text>` — the bold series was silently reset to medium.

**Analysis:** `Font::merge()` was calling `specialize(font_name)` with the font
filename "cmb10". `specialize` examines Unicode properties of its argument text.
"cmb10" contains digit characters which fall into the "Other Symbol" branch,
which resets `series` to "medium" and `shape` to "upright".

In Perl, `merge()` has an **optional** `specialize` parameter (passed explicitly,
e.g. `merge(specialize => $text)`). It is NOT called by default. In Rust, someone
added `specialize(font_name)` in the merge code path, which was incorrect —
font filenames are not rendered text.

**Fix:** Remove `specialize()` from `merge()`. `specialize()` should only be
called at `TBox::new()` time (tbox.rs line 131) with actual rendered text content.

**Key insight:** `specialize()` is a text-classification function, not a font-
metadata function. Never call it with font names, filenames, or CS names.

---

## 3. Catcode::CS vs Catcode::ESCAPE distinction

**Discovery:** Token matching code using `cc == Catcode::ESCAPE` failed because
control sequence tokens have `Catcode::CS`, not `Catcode::ESCAPE`.

**Analysis:** `ESCAPE` (catcode 0) is the backslash character itself — it's the
input character catcode. `CS` is the catcode of a fully-formed control sequence
token (e.g. `\foo`). When the tokenizer reads `\foo`, it produces a single token
with catcode `CS`, not a token with catcode `ESCAPE` followed by letter tokens.

**Key insight:** Use `cc.is_active_or_cs()` to test for CS/ACTIVE tokens.
Never compare `cc == Catcode::ESCAPE` when looking for control sequences.

---

## 4. RegisterType::PartialEq trap: Number == CharDef

**Discovery:** `register == RegisterType::Number` matched CharDef registers
due to custom `PartialEq` implementation.

**Analysis:** The `PartialEq` impl for `RegisterType` treats `CharDef` as equal
to `Number` (since char defs are numerically-valued). This means `if register !=
RegisterType::Number` does NOT exclude CharDef.

**Fix:** Use `matches!(register, RegisterType::Number)` pattern matching instead
of `==`/`!=` operators to distinguish the variants correctly.

---

## 5. at_letter catcode restore: None vs Some(OTHER)

**Discovery:** `\makeatletter` made `@` a letter, but `\makeatother` didn't
restore it — `@` stayed as LETTER permanently.

**Analysis:** `at_letter()` saves the old catcode with
`saved = state::lookup_catcode('@')`. When `@` isn't in the catcode table
(using default catcode OTHER), `lookup_catcode` returns `None`. The restore
function then calls `state::assign_catcode('@', saved)` where `saved` is `None`,
which is a no-op — it doesn't set the catcode back to OTHER.

**Fix:** Use `unwrap_or(Catcode::OTHER)` when restoring — `None` means "was
using default OTHER catcode", so restore to OTHER explicitly.

**Key insight:** State lookups returning `None` for defaults is a common pattern.
Always consider what `None` means in context — it might mean "default value"
rather than "no value".

---

## 6. Sizer string parsing: `#property_name` vs `#digit`

**Discovery:** Nested tabulars (tabtab_test) lost their 3rd column — the inner
tabular whatsit reported width=0, causing `normalize_prune_columns` to remove it.

**Analysis:** `\lx@begin@alignment` has `sizer => "#alignment"`. In Perl,
`Whatsit::computeSize` (Whatsit.pm L257-260) parses sizer strings with
`$sizer =~ /^(#\w+)*$/` — each `#token` is checked: if numeric, `getArg($n)`;
if alphabetic, look up `$$props{$name}`.

In Rust, `IntoOption<Option<SizingClosure>> for &str` (traits.rs) only handled
`#digit` — `"alignment".parse::<usize>()` failed and defaulted to arg 1 via
`unwrap_or(1)`. So `sizer => "#alignment"` was silently computing size of arg 1
(the optional `[]` arg) instead of the alignment property.

**Fix:** Rewrote the sizer string parser to match Perl: parse each `#word` as
either numeric (arg lookup) or alphabetic (property lookup). Supports compound
patterns like `#1#2` as well.

**Key insight:** Any time `parse::<usize>().unwrap_or(default)` is used on user-
provided strings, verify the default makes sense. Silent fallbacks can mask bugs
for months — the sizer was returning (0,0,0) which just happened to trigger
column pruning instead of panicking.

---

## 7. `align_group_count` (`$ALIGN_STATE`): scan-level only, retract on unread

**Discovery:** Nested `\vbox{\halign{...}}` inside an outer `\halign` caused
outer column-end tokens (`&`, `\cr`) to not fire `handle_template`, because
`align_group_count` was >0 when it should have been 0.

**Root cause (two bugs):**

1. **`unread_one` didn't adjust agc.** Perl's `unread()` sub (Gullet.pm L340-359)
   always adjusts `$ALIGN_STATE` for `{` and `}` tokens, whether unreading one
   or many tokens. Rust had `unread_one` (no adjustment) and `unread_vec` (with
   adjustment). Functions like `skip_spaces()` → `read_non_space()` read `{` via
   `read_token()` (incrementing agc), then unread it via `unread_one` (no
   decrement). The `{` would be re-read later, double-incrementing.

2. **`stomach::bgroup()/egroup()` adjusted agc.** Perl's bgroup/egroup
   (Stomach.pm L327-342) do NOT touch `$ALIGN_STATE`. It's tracked only at the
   scan level (in `readToken`/`readXToken`). The Rust code had
   `increment_align_group_count()` in `bgroup()` and `decrement_align_group_count()`
   in `egroup()`, causing double-counting for every `{`/`}` pair.

**Fix:** Added agc adjustment to `unread_one` for BEGIN/END tokens. Removed agc
tracking from `bgroup()`/`egroup()`.

**Key insight:** `$ALIGN_STATE` is a scan-level concept (TeX §309). It must be
incremented/decremented exactly once per `{`/`}` token as it passes through
`readToken` or `readXToken`, and must be retracted when tokens are unread.
The stomach's group machinery is a separate concern.

---

## 8. Rust macros cannot dispatch on type — Vec<Token> vs Token vs &[Token]

**Discovery:** Attempting to create unified macros that accept both single tokens
and token sequences leads to compilation errors because Rust's `macro_rules!`
operates on syntax patterns, not types.

**Analysis:** Unlike Perl where `Tokens()` can accept scalars, arrays, or objects
and figure it out at runtime, Rust macros expand before type information is
available. The `Tokens!()` macro works via `Into<Vec<Token>>` trait, which works
for types that implement the conversion. But you can't write a single macro
invocation that conditionally handles `Token`, `Vec<Token>`, `Tokens`, and
`&[Token]` differently — the macro expander doesn't know the types.

**Workaround:** When building token sequences from mixed sources (static tokens
plus dynamic `Vec<Token>` from `.revert()` etc.), use explicit `Vec<Token>`
construction with `extend()` instead of trying to stuff everything into
`Tokens!()`:
```rust
let mut toks: Vec<Token> = vec![T_CS!("\\hbox"), T_BEGIN!()];
toks.extend(content.revert());
toks.push(T_END!());
stomach::digest(Tokens::new(toks))?;
```

**Key insight:** When the `Tokens!()` macro doesn't cooperate with a particular
type, fall back to imperative `Vec<Token>` construction. Don't fight the macro.

---

## 9. arena::with pattern for zero-allocation string access

**Principle:** Prefer `arena::with(sym, |s| ...)` over `arena::to_string(sym)`
when you only need a `&str` reference temporarily.

**Analysis:** The string interner stores all interned strings (SymStr/SymbolU32)
in a thread-local arena. `arena::to_string(sym)` resolves the symbol and
allocates a new `String` on the heap. `arena::with(sym, |s| ...)` borrows
the string directly from the arena with zero allocation — the `&str` lives
only for the duration of the closure.

**When to use each:**
- `arena::with(sym, |s| ...)` — when the result depends on `&str` and can
  be computed inline (e.g., comparisons, `set_property(key, val)`, formatting)
- `arena::with2(s1, s2, |a, b| ...)` — when you need two symbols resolved
- `arena::to_string(sym)` — only when you need an owned `String` that outlives
  the current scope (e.g., storing in a `HashMap<String, ...>`)

**Key insight:** Every `arena::to_string` is a heap allocation. In hot paths
(per-token, per-column, per-row), this adds up. The `with` pattern is always
preferable when the string use is short-lived.

---

## 10. Porting RawTeX() blocks: copy bravely and exactly

**Principle:** Perl `RawTeX()` calls should be ported as `RawTeX!()` in Rust with
the exact same TeX string content. Even very large blocks of raw TeX code should
be copied over directly — the TeX layer should always match the Perl exactly
unless there is a specific technical problem (e.g. Rust string escaping).

**Why:** The TeX code in `RawTeX()` blocks is already debugged and tested in Perl.
It defines internal macros, counters, lengths, and environments at the TeX level.
Attempting to "Rustify" these blocks or selectively port pieces introduces
subtle divergences. The Rust `RawTeX!()` macro feeds the string through the
tokenizer/expander just as Perl does, so fidelity is essentially free.

**Key insight:** Do not be intimidated by large `RawTeX()` blocks. The cost of
porting them is just copy-paste; the cost of NOT porting them is missing
definitions that later cause test failures in seemingly unrelated places.

---

## 11. Parameter prototype conventions: `{}` vs named parameter types

**Principle:** In LaTeXML's Perl prototype strings, `{}` means "read a Plain
balanced group". A named parameter type (like `Token`, `Number`, `Variable`)
is identified by its bare name in the prototype, NOT wrapped in braces.

**Example:** `DefMacro!("\\foo Token", ...)` reads one Token parameter.
Writing `DefMacro!("\\foo {Token}", ...)` would read a Plain balanced group
and the word "Token" would be literal body content, not a parameter type.

**Analysis:** The prototype parser (`def_parser.rs`) distinguishes between:
- `{}` → Plain parameter reader (reads balanced `{...}` group)
- `[]` → Optional parameter reader (reads `[...]` if present)
- `Token` → named parameter type (looked up in PARAMETER_TYPES table)
- `[Number]` → Optional parameter with inner Number reparsing

When porting from Perl, be careful: `DefMacro('\foo{}', ...)` in Perl is
equivalent to `DefMacro!("\\foo {}", ...)` in Rust — the `{}` is a parameter
spec, not literal braces.

**Key insight:** If a macro argument isn't being read correctly, check whether
the prototype has `{}` (Plain reader) when it should have a named type, or
vice versa. The `{}` braces in prototypes always mean "read a balanced group".

---

## 12. normalize_sum_sizes: per-column-index arrays, not flat lists

**Discovery:** Alignment column widths were computed incorrectly — nested
tabulars and multi-row tables had wrong column dimensions, causing incorrect
CSS width/height attributes.

**Analysis:** Perl's `normalize_sum_sizes` (Alignment.pm L500-664) uses
per-column-index arrays: `$colwidths[$j]` collects all width values for
column j across all rows, then computes `max(@{$colwidths[$j]})`. The Rust
implementation was using a flat list — one entry per cell across all rows —
which broke when rows had different column counts or when colspan>1 cells
needed width distributed across multiple columns.

Additional Perl semantics that were missing in Rust:
- **vattach height/depth split:** Perl computes per-alignment height/depth
  based on `vattach` property (top = all depth, bottom = all height,
  middle = split with math axis approximation). Rust set `cached_depth = 0`.
- **lspaces/rspaces:** Perl adds left/right space dimensions to cell width
  and sets `lpadding`/`rpadding` properties. Rust ignored these entirely.
- **Border padding:** Perl adds `0.4*UNITY` (0.4 * 65536 scaled points) for
  each border. Rust was missing this.
- **First/last row strut:** Perl conditionally applies strut height to first
  row and strut depth to last row only for non-LaTeX alignments. Rust applied
  strut to all rows.
- **colspan>1 distribution:** Perl distributes wide cells' excess width
  equally across spanned columns. Rust didn't handle this.

**Fix:** Complete rewrite of normalize.rs to match Perl Alignment.pm semantics:
per-column-index arrays, separate rowdepths, vattach split, border padding,
strut special-casing, colspan distribution, lspaces/rspaces propagation.

**Key insight:** When porting array-accumulation patterns from Perl, verify
the indexing structure. `$foo[$j]` in a nested loop is per-index accumulation,
not flat append. A flat `Vec::push` is fundamentally different from
`Vec<Vec>::push_at_index`.

---

## 13. close_node_with_strictness: walker tracks walker node, not target

**Discovery:** `close_node_with_strictness` (document.rs) was using
`node.get_type()` for the loop variable `t`, where `node` is the *target*
node being closed. This should be `n.get_type()` where `n` is the *walker*
node traversing up the tree.

**Analysis:** The function walks up the DOM tree from `self.node` toward
`node`, auto-closing intermediate elements. The loop condition
`t != Some(NodeType::DocumentNode) && &n != node` should track whether the
*walker* has reached the document root, not whether the *target* is the
document root (which is invariant across iterations).

Perl (Document.pm L952-970):
```perl
my $t;
while (($t = $n->nodeType) != XML_DOCUMENT_NODE && !$n->isSameNode($node)) {
  ...
  $n = $n->parentNode; }
```

The bug was in both the initialization (`let mut t = node.get_type()`) and
the loop body update (`t = node.get_type()` instead of `t = n.get_type()`).

**Fix:** Changed both to `n.get_type()`.

**Key insight:** When porting Perl loops with multiple node references
(`$node` = target, `$n` = walker), be very careful about which variable
is used in loop conditions. A single-character difference (`node` vs `n`)
can cause completely wrong loop termination.

---

## 14. close_to_node ifopen parameter: suppress error when true

**Discovery:** `close_to_node` in document.rs declared `_ifopen: bool`
(prefixed with underscore = unused). The Perl version uses `$ifopen` to
suppress the "not open" error when closing a node that isn't actually in
the current open-element path.

**Analysis:** Perl (Document.pm L910-925):
```perl
if (!$ifopen) {
  Error('malformed', $qname, $self, "Attempt to close $qname, which isn't open"); }
```

When `$ifopen` is true (the caller says "close this *if* it's open"), reaching
the document root without finding the node is not an error — it just means
the node wasn't open. Without this guard, every "close if open" call to a
non-open node would emit a spurious error.

**Fix:** Renamed `_ifopen` to `ifopen` and added the `if !ifopen` guard
before the error emission.

**Key insight:** Underscore-prefixed parameters (`_foo`) in Rust suppress
unused-variable warnings. When porting from Perl, check whether each
"unused" parameter is *intentionally* unused or *accidentally* not yet
implemented. The `_` prefix can mask missing functionality.

---

## 15. DefKeyVal machinery: default resolution and setKeysExpansion guard

**Discovery:** Bare keys like `sensitive,` in listings language definitions
weren't getting their default values applied, despite `DefKeyVal!("LST",
"sensitive", "", "true")` being correctly called.

**Analysis:** The default resolution happens in TWO places:

1. **During KeyVals parsing** (`add_value` with `use_default=true`):
   `keyval_get(keyval_qname(prefix, keyset, key), "default")` — uses key
   `KEYVAL@default@KV@LST@sensitive`. This is the CORRECT path.

2. **During `lstActivate`** (dead fallback, now removed):
   `LookupValue("KEYVAL@LST@sensitive@default")` — WRONG key pattern
   (doesn't include KV prefix). This never matched in Perl either but was
   carried over as dead code.

The actual root cause was `\@lstdefinelanguage` ignoring the base language
parameters (`_base_dialect`, `_base_language`). In Perl, `$keyvals->setValue
('language', Tokens(@base))` inserts a `language` key into the keyvals that
triggers recursive language chain activation. Without this, `[LaTeX]{TeX}
→ [common]{TeX} → [primitive]{TeX}` never fires, so `sensitive,` from
`[primitive]{TeX}` never reaches the processing context.

**Related discovery:** `lstClearLanguage` in Perl clears class `'textcs'`
but texcs words use class `'texcss'` — a Perl typo/quirk that allows texcs
words to survive the clear across the language chain.

**setKeysExpansion guard:** Rust adds `state::has_meaning(...)` before
emitting `\qname@default`. Perl unconditionally emits `\qname@default`
which causes undefined-CS errors for bare keys without registered defaults
(e.g., `a4paper` via `DeclareOptionX`). The Rust guard falls back to
`\qname{}`, which is more robust.

**Key insight:** When debugging default-value resolution in keyvals, check:
(a) that `DefKeyVal`/`define()` stored the default under the correct
`KEYVAL@default@{qname}` key, (b) that the KeyVals parser (`add_value`)
successfully retrieves it, (c) that the calling code doesn't introduce a
different key naming convention.

---

## 16. Star (`*`) in CS names causes infinite compile loop

**Date:** 2026-03-15

The `DefMacro!` and `Let!` proc macros enter an infinite loop (OOM kill at
14GB+) when the control sequence name contains special characters like `*`
(star) or `{}` (braces). For example:

```rust
// BROKEN — causes infinite compile loop:
DefMacro!("\\IEEEeqnarray*{}", "\\eqnarray*");
Let!("\\endIEEEeqnarray*", "\\endeqnarray*");
```

The compile-time tokenizer in `latexml_codegen` interprets `*` and `{}`
as special tokens or parameter spec patterns and gets stuck in an infinite
matching loop. Both `*` and `{}` are valid in TeX control sequences (e.g.
`\eqnarray*`, `\begin{foo}`).

**Workaround:** Always use the `T_CS!()` wrapper.

```rust
DefMacro!(T_CS!("\\IEEEeqnarray*"), "{}", T_CS!("\\eqnarray*"));
Let!(T_CS!("\\endIEEEeqnarray*"), T_CS!("\\endeqnarray*"));
```

**Refactoring needed:** `DefMacro!` and `Let!` should accept `T_CS!("\\foo*")`
as the first argument, bypassing string tokenization entirely. The internal
tokenizer should also be fixed to handle `*` in CS names without looping.

---

## 17. Sizer inference from reversion: silent incorrect sizing

**Symptom:** All math boxes (`\hbox{$...$}`) return identical size `5.00002pt x 7.5pt + 0.55554pt` regardless of math content.

**Root cause:** `dialect.rs::infer_sizer()` inferred a sizer from the Constructor's reversion tokens when no explicit sizer was specified. For body-capturing constructors like `\lx@begin@inline@math`, the reversion is `$` (T_MATH). The inferred sizer measured the literal string `"$"` with the current font, producing the `$` character's glyph size instead of the math body content size.

**Fix:** `infer_sizer()` now returns `None` when no explicit sizer is set, matching Perl's behavior where sizer is never inferred from reversion. The Whatsit's default `compute_size()` then correctly uses the "body" property.

**Key insight:** In Perl, `Whatsit::computeSize()` has explicit fallback: use body if available, else sum all args, else use reversion. The reversion is only consulted as a last resort. Rust's `infer_sizer` was short-circuiting this cascade.

---

## 18. METRIC_MAP vs STDMETRICS key mismatch: math fonts fall back to cmr

**Symptom:** Math character widths (e.g., italic 'a') don't include italic correction. All math characters use cmr (serif) metrics instead of cmmi (math italic) metrics.

**Root cause:** `METRIC_MAP` mapped `"math_medium_italic"` → `"cmmi"` but `STDMETRICS` used `"cmm"` as the key for cmmi10 data. The `get_metric()` function tried `"cmmi10"` → not found, then `get_metric_for_name("cmmi")` → not found, then fell back to `"cmr"`.

**Fix:** Changed `METRIC_MAP` value from `"cmmi"` to `"cmm"` to match the `STDMETRICS` key. Now `get_metric_for_name("cmm")` finds the correct cmmi metrics.

**Key insight:** The STDMETRICS key naming convention uses the base without the trailing 'i' (cmm, not cmmi), but METRIC_MAP was using the TFM filename convention (cmmi). Always ensure METRIC_MAP values match STDMETRICS keys.

---

## 19. enterHorizontal uses inplace assignment, NOT beginMode

**Context:** Understanding why `\vbox{hop}` should produce width=\hsize (469.75pt) but Rust was producing the natural character width (5.55pt).

**Root cause:** Perl's `enterHorizontal` (Stomach.pm line 418) uses `assignValue(MODE => 'horizontal', 'inplace')` — NOT `beginMode('horizontal')`. The comment says: "SAME frame as BOUND_MODE!" This means BOUND_MODE stays as 'internal_vertical' when MODE changes to 'horizontal'. When `endMode('internal_vertical')` calls `leaveHorizontal_internal`, the condition `MODE eq 'horizontal' AND BOUND_MODE =~ /vertical$/` PASSES because BOUND_MODE was never changed. This triggers `repackHorizontal`, which groups character boxes into a horizontal `List(@para, mode => 'horizontal')`. Perl's `List()` constructor (List.pm line 53-54) sets `width = \hsize` when `mode eq 'horizontal'`.

**Fix:** `predigest_box_contents` now calls `begin_mode`/`end_mode` matching Perl's `readBoxContents` frame scope. After `invoke_token`, checks if MODE was changed to 'horizontal' inplace, and if so, calls `repack_horizontal_in_list` to group character boxes into a horizontal sub-List with width=\hsize. Guard: only repacks when body contains simple TBoxes (not Whatsits like tabular).

**Key insight:** The distinction between `assignValue(MODE, 'inplace')` and `beginMode(mode)` (which calls `pushStackFrame` + `assignValue(MODE, 'local')`) is critical. The former modifies the SAME frame's BOUND_MODE scope; the latter creates a NEW scope that hides the parent's BOUND_MODE.

## 20. Whatsit::get_arg() is 1-based: get_arg(0) always returns None

**Context:** `\turnbox{90}{hello}` always produced angle=0. Debug output showed `get_arg(0)` returning None. The `\turnbox` constructor used 0-based indexing for arg access.

**Root cause:** `Whatsit::get_arg(n)` (whatsit.rs line 108-116) uses 1-based indexing to match Perl's `$whatsit->getArg(1)` convention:
```rust
pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    if n == 0 { return None; }
    match self.args.get(n - 1) { ... }
}
```
Code written with 0-based assumption silently gets None for the first arg, triggering `unwrap_or(0.0)` fallbacks.

**Fix:** Changed all `get_arg(0)` to `get_arg(1)`, `get_arg(1)` to `get_arg(2)`, etc. in `\turnbox`, `{turn}`, `{rotate}`, and `\lx@diagheads`. Also confirmed: OptionalKeyVals parameters that are NOT provided do NOT occupy an arg slot (novalue=true), so they don't shift the indices.

**Key insight:** Always use 1-based indexing with `get_arg()`. The pattern `get_arg(0).map(...).unwrap_or(default)` is a silent bug — it always uses the default. To catch these: grep for `get_arg(0)` in the codebase.

---

## 8. Math rewrite rules run BEFORE grammar parsing

**Discovery:** The DefMathRewrite mechanism (via `.latexml` files in Perl, `*_src.rs` files in Rust) fires during the "Rewriting" phase in `core_interface.rs`, which happens BEFORE the Marpa grammar parses the XMath tree. This means rewrite rules can change the XMTok structure (e.g., setting `role="ID"` or `role="FUNCTION"`) and those changes INFLUENCE how the grammar parses the expression.

**Why it matters:** The post-finalize UNKNOWN→ID conversion that was added as a workaround does NOT achieve the same effect. By the time it runs, the grammar has already parsed the expression using `role="UNKNOWN"`. Setting role to ID after parsing is cosmetic — it doesn't change the parse tree structure.

**Correct approach:** For tests that need `role="ID"` on single-letter tokens, create a `*_src.rs` file in `latexml_contrib` that uses `DefMathRewrite!` to set roles BEFORE parsing. This matches Perl's `.latexml` mechanism and actually changes how the math is parsed.

**Example:** `simplemath_src.rs` already demonstrates this pattern:
```rust
add_math_rewrite("a", "ID")?;  // sets role="ID" before parsing
add_math_rewrite("f", "FUNCTION")?;  // enables function application
```

**Key insight:** The rewriting phase is a meaningful pre-parse step, not a post-processing cosmetic. Changing roles before parsing changes the parse tree.

## 21. Floating pre-scripts: POST→FLOAT kludge and grammar rules

**Discovery:** In `{}_a^b\sum_c^d x`, the `_a` creates FLOATSUBSCRIPT (empty base `{}`), but `^b` creates POSTSUPERSCRIPT (base is the FLOAT result, which is non-empty). Perl's `parse_kludgeScripts_rec` preprocesses the token stream: when a FLOATSUBSCRIPT is followed by POSTSUPERSCRIPT (or vice versa), both are treated as pre-scripts on whatever follows, with the POST script getting forced `'pre'` position WITHOUT setting `_wasfloat`.

**Why it matters:** The `_wasfloat` flag controls level bumping. When two scripts share the same empty `{}` base (e.g., `{}_a^b`), they should be at the SAME level (both `pre1`). But when each has its own empty base (e.g., `{}_a{}^b`), they should be at DIFFERENT levels (`pre1` and `pre2`). Perl achieves this because POST scripts don't set `_wasfloat`.

**Rust approach:** Instead of pre-processing the token stream, the Marpa grammar has dedicated rules:
- `prescripted_bigop`: floating scripts wrapping bigops as pre-scripts
- `prefix_script_pre`: semantic action that forces "pre" position without `_wasfloat` (matching Perl's `NewScript($base, $script, 'pre')` for POST scripts)
- `prescripted_factor_post_r/l`: POST scripts used as pre-scripts on factors (only valid when FLOAT-wrapped)
- Recursive chaining via `scripted_factor_l2 += floatsubarg scripted_factor_l2` for 3+ float chains

---

## 12. alignsafeOptional: alignment token interception during parameter parsing

**Problem:** `\begin{aligned}` nested inside `\begin{align}` loses 85% of content. All errors cascade from "Attempt to end mode `inline_math` in `math`". The inner aligned's content `& D` gets intercepted by the outer alignment.

**Root cause:** `\aligned[]` reads its optional arg using standard `[]` parameter parsing. During the `read_x_token` call to check for `[`, the gullet's alignment check intercepts `&` from the content. Since the inner alignment hasn't been set up yet, `handle_template` fires for the OUTER alignment, injecting the outer after-template `$` into the inner alignment's token stream. This `$` triggers `\lx@end@inline@math` inside the inner alignment, corrupting the mode stack.

**Fix (3 parts):**
1. **`\aligned`/`\alignedat`**: Implement Perl's `alignsafeOptional` — read optional arg with `local_align_group_count(1000000)` to disable alignment token interception during arg reading.
2. **`\lx@begin@alignment`**: Remove spurious `SkipSpaces` parameter (Perl has none). SkipSpaces also triggers `read_x_token` which intercepts alignment tokens.
3. **`eqnarray_bindings`**: Remove spurious `Let(T_MATH, '\lx@dollar@in@mathmode')` — Perl doesn't set this.

**Key insight:** Any `read_x_token` call inside an alignment column can trigger `handle_template`. Parameter parsing (SkipSpaces, optional `[]`, etc.) calls `read_x_token`. If the content after the macro contains alignment tokens (`&`, `\cr`), they'll be intercepted by the outer alignment's template. Perl avoids this with `$LaTeXML::ALIGN_STATE = 1000000` (our `local_align_group_count`).

## 22. Babel OOM: undefined macros → \<ltx:ERROR/\> self-expansion → infinite loop

When babel 3.x calls `\selectlanguage{french}`, it triggers `\bbl@provide@locale`
which calls `\babelprovide{french}` if `\csname datefrench\endcsname` is `\relax`.
The `\babelprovide` path reads `.ini` files and uses many internal macros that our
engine doesn't define. Our error recovery for undefined macros creates them as
`<ltx:ERROR/>` — a string that, when expanded again, creates more error tokens.
Some babel macros accumulate lists that include undefined macros, creating chains
of error-recovery expansions that consume 14-26GB of memory.

**Root causes identified:**
1. `\bbl@languages` undefined → error recovery → self-referential expansion
2. `\babelprovide` ini-loading path hits multiple undefined internal macros
3. `\bbl@iflanguage` fails because `\l@<lang>` registers aren't defined

**Fixes applied (emulating Perl's precompiled kernel):**
- Pre-define `\bbl@languages{}` before babel loads
- Pre-define `\captionslang` + `\datelang` for 27 common languages
- Pre-define `\l@lang` registers for 13 common languages
- Clear `\@fontenc@load@list` after babel loads (comma leak fix)

**Fundamental fix needed:** Precompiled kernel dump (infrastructure E) that
pre-loads all kernel state, or fix error recovery to NOT create self-referential
expansions for undefined macros.

---

## 23. DefConstructor state lookups: digest time vs construction time

**Discovery:** xy-pic SVG constructors produced zero coordinates because register
values (\X@c, \Y@c, etc.) were read at construction time instead of digest time.

**Analysis:** `DefConstructor` bodies (`sub[document, args, props] { ... }`) run at
CONSTRUCTION time (when XML is built). But multiple constructors are digested in
sequence before any are constructed. A register read at construction time sees the
value from the LAST digested constructor, not the one being constructed.

**Fix pattern:** Use `properties => sub[args] { ... }` to capture register values
at digest time. The callback returns a `SymHashMap<Stored>` that becomes the
whatsit's properties. The constructor body reads from `props.get("key")`.

```rust
// WRONG: reads register at construction time
DefConstructor!("\\foo", sub[document, _args, _props] {
    let val = state::lookup_register("\\bar", Vec::new())?; // WRONG
});

// RIGHT: captures register at digest time
DefConstructor!("\\foo", sub[document, _args, props] {
    let val = props.get("bar_val"); // Read from properties
}, properties => {
    let val = state::lookup_register("\\bar", Vec::new())?;
    stored_map!("bar_val" => format!("{}", val))
});
```

**Scope:** Applied to all 19 xy SVG constructors + `\pic@makebox@`. Audit found
no other critical instances in the engine codebase.

---

## 24. catcode checks vs defined_as: Perl is inconsistent

**Discovery:** Replacing `get_catcode() == Catcode::BEGIN` with `defined_as(T_BEGIN!())`
caused regressions because Perl uses DIFFERENT check patterns in different functions.

**Analysis:** Perl's Token has both `$$token[1] == CC_BEGIN` (raw catcode check)
and `$token->defined_as(T_BEGIN)` (meaning check via `\let` chain resolution).
Perl uses them inconsistently:

| Function | Perl check | Matches `\bgroup`? |
|----------|-----------|-------------------|
| readArg | CC_BEGIN catcode | No |
| readBoxContents | defined_as(T_BEGIN) | Yes |
| readBalanced (require_open) | CC_BEGIN \|\| Equals(meaning, T_BEGIN) | Yes (dual) |
| readDelimited | CC_BEGIN catcode | No |
| readTokensValue | CC_BEGIN catcode | No |
| readUntilBrace | CC_BEGIN catcode | No |

**Fix:** Match each Perl function's exact check pattern. Never assume `defined_as`
is universally correct — check the Perl source for each specific function.

---

## 18. Rewrite system: internal DOM ≠ serialized XML

**Discovery (Session 58):** \lxDeclare wildcard patterns failed because the
XPath matched the serialized XML structure, not the internal DOM structure.

**Analysis:** The serializer transforms the DOM during output:
- Internal: `<XMApp role="POSTSUBSCRIPT"><sub_content/></XMApp>` (base token is a SIBLING)
- Serialized: `<XMApp><XMTok role="SUBSCRIPTOP"/><base/><sub/></XMApp>` (3 children)

XPath queries in rewrite rules run on the INTERNAL DOM (before serialization).
Attributes like `role="SUBSCRIPTOP"` and `scriptpos="post1"` exist only in the
serialized form. The internal DOM uses `role="POSTSUBSCRIPT"` on the XMApp
with `scriptpos="1"` (just the position number).

**Debugging approach that worked:**
1. List all unique attribute values: iterate all `*[@role]` nodes and collect
   via `get_property("role")` into a BTreeSet.
2. Compare XPath results: test the same XPath pattern with Python's lxml (which
   operates on serialized XML) vs our libxml2 (which operates on internal DOM).
3. Inspect actual node attributes: use `node.get_attributes()` to see the raw
   HashMap of attribute names and values.

**Key insight:** Always verify attribute values in the internal DOM before
writing XPath predicates. The serializer may synthesize, rename, or transform
attributes. Use `get_attributes()` debug prints rather than assuming the
serialized XML reflects the internal representation.

---

## 19. XPath nested predicates: known limitation

**Discovery (Session 58):** `ltx:XMApp[child::*[text()='x']]` returns 0 matches
in our libxml2 XPath evaluator, even though the elements exist.

**Analysis:** Predicates that check child attributes or text content within a
parent predicate (`[child::*[@role='SUBSCRIPTOP']]`) fail silently. Boolean
attribute checks (`[child::*[@role]]`) work, and top-level text comparisons
(`*[text()='x']`) work, but combining them in nested predicates doesn't.

The `xml:` namespace prefix also has quirks: `@xml:id` works in some contexts
but `@xml:id='S1'` (value comparison) may fail depending on the context.

**Workaround:** Match broadly with XPath (e.g., `*[@role='POSTSUBSCRIPT']`)
and apply fine-grained filtering in Rust code using `node.get_property()`,
`node.get_content()`, `node.get_next_sibling()`, etc.

**Key insight:** Treat our XPath as limited — use it for coarse selection and
do precise matching in Rust. Don't trust complex nested predicates.

---

## 20. Scope vs content Select: shared select_count hazard

**Discovery (Session 58):** Scoped rewrite rules with `select_count=2` (for
subscript wildcard wrapping) caused scope Selects to fail because the scope
Select tried to collect 2 sibling section nodes.

**Analysis:** `RewriteOptions::select_count` is shared across ALL clauses in
a Rewrite rule. When a rule has [Scope, Xpath, Attributes] clauses, the Scope
compiles to a Select that uses `select_count` — but this count was meant for
the inner Xpath Select, not the Scope Select.

In Perl, `nnodes` is stored per-clause in the pattern array `[$xpath, $nnodes, @wilds]`.
In Rust, it's a single shared field.

**Fix:** Distinguish scope Selects from content Selects by checking if the
XPath contains `xml:id` or `@id=`. Scope Selects always use nmatched=1.

**Key insight:** When porting Perl structures where each clause has its own
metadata (like `nnodes`), verify that shared fields in Rust don't create
cross-clause interference.

---

## 21. afterConstruct vs afterDigest timing: gullet state

**Discovery (Session 58):** `\thesection@ID` expanded in afterConstruct
always returned "S7" (the last section) regardless of the declaring section.

**Analysis:** `afterDigest` runs during the digestion phase — the gullet has
the correct current state (current section, counters, etc.). `afterConstruct`
runs during the construction phase — all digestion is complete, so the gullet
state reflects the end-of-document state.

For \lxDeclare, the `decl_id` (computed in afterDigest) correctly has the
section prefix (S1.XMD1, S2.XMD1, etc.). But the `scope` (derived from
`\thesection@ID` in afterConstruct) always sees the last section.

**Fix:** Derive the scope from the `decl_id` prefix rather than re-expanding
`\thesection@ID` in afterConstruct.

**Key insight:** Any TeX state query (counter values, section IDs, font state)
in afterConstruct reflects end-of-document state. Store needed values in
afterDigest as whatsit properties, then use them in afterConstruct.

---

## 22. DefEnvironment scope: after_digest vs after_digest_body timing

**Discovery (Session 108):** `\caption` inside `\begin{floatingfigure}` emitted
`Error:undefined:\@captype` even though `before_float` had set `\@captype` via
local-scope `def_macro` and the body could read it (`\@ifundefined{@captype}`
inside the env body reported "DEF:figure").

**Analysis:** Three hooks in DefEnvironment run at different frame-lifecycle
points:

1. `before_digest` — runs at digest time, in the env's frame. State assigned
   here is visible to the body.
2. `after_digest` — runs at digest time, **while the env frame is still
   active**. State from `before_digest` is still visible.
3. `after_digest_body` — runs at digest time, **after the env frame has
   popped**. State assigned with local scope in `before_digest` is GONE.

The engine's `{figure}` / `{table}` envs use `after_digest` for `after_float`
because `after_float` does `digest(\@captype)` — which needs the local binding
from `before_float`. `floatflt` / `floatfig` were using `after_digest_body` and
hit this exact bug on sandbox paper 0810.1610.

**Fix:** Use `after_digest` for hooks that read frame-local state. Use
`after_digest_body` only for hooks that operate on the whatsit's body
structure (e.g. `rotating_sty`'s `rotated_properties` scan, which inspects
the body DOM without looking up TeX state).

**Key insight:** Match the hook to the data you're reading:
- Reading TeX state (counters, registers, macros, \@captype) → `after_digest`
- Operating on the whatsit's body nodes in isolation → `after_digest_body`

Rust-specific: Perl's `afterDigest` in `DefEnvironment` is effectively Rust's
`after_digest`; Perl's `afterDigestBody` (rarely used) matches Rust's
`after_digest_body`. When porting Perl code that uses `afterDigest`, keep
`after_digest` in Rust unless there's a specific reason (body-structure
modification) to defer until after frame pop.

---

## 32. parse_parameters(..., init_flag): true at runtime, false at compile-time

**Discovery:** Strict-Perl `LoadFormat` mutual exclusivity (active 2026-04-26)
depends on dump-provided Expandables reading arguments correctly when
`_base.rs` is skipped. Initial flip-attempts surfaced "Missing argument {}"
errors the moment any dump-provided Expandable tried to read an argument —
e.g. `\@gobble{x}` said `x` was missing.

**Analysis:** `def_parser::parse_parameters(proto, cs, init_flag)` has an
`init_flag` parameter that controls whether each `Parameter` runs its
`.init()` method. `init()` looks up the type's reader via the
`PARAMETER_TYPES` mapping (populated by `base_parameter_types.rs`). With
`init_flag=false`, no lookup happens; every `Parameter` keeps the default
mock reader that returns `Ok(ArgWrap::None)` and emits a
"Please define a real reader" warning. At invocation, the mock returns
None for each arg → `checked_value` throws "Missing argument {}".

The `false` was historically correct for callers that run at compile time
(macros expanded before state init). But every RUNTIME path silently shipped
broken `Parameters`. Four call sites needed the fix:

- `dump_reader.rs` (was: false → true)
- `dump_loader.rs` (was: false → true)
- `dump_codegen.rs` codegen template (was: emitting false → now emits true)
- `latex_constructs.rs::\DeclareTextFontCommand` (was: false → true)

**Key insight:** When in doubt, `parse_parameters(..., true)` for runtime.
Only use `false` when the resulting `Parameters` are consumed at
compile-time or before state initialization. The mock reader's warning
will surface at INVOCATION, not at definition time — so defective sites
go undetected for a long time.

**Sentinel:** If a dump-loaded or runtime-declared definition produces a
"mock_reader: Please define a real reader, this is a mock fallback!"
warning followed by "Missing argument {}", the root cause is an
`init_flag=false` in the declaration path.

---

## 33. Dump round-trip: nargs alone is insufficient for parameter fidelity

**Discovery:** Early strict-Perl `LoadFormat` PoC (the D0 effort that
preceded the active 2026-04-26 mission) hung `00_tokenize` for 34+
minutes at 300% CPU even AFTER landing all the `init_flag=true` and
None-body-serialization fixes. Root cause traced to parameter-type
flattening in the dump round-trip.

**Analysis:** The v1 E-entry format recorded only `nargs` (a count), and
`dump_reader` rebuilt `Parameters` via `"{}".repeat(nargs)` — flattening
everything to Plain. For most CSes this is fine, but parameter types that
affect argument-READ behavior diverge:

- `DefToken` (reads a single token, not a balanced group)
- `Optional` (reads `[…]`, with or without default value)
- `Semiverbatim` (disables specified catcodes during reading)
- `Until:<delim>` (reads tokens up to a delimiter; delimiter may contain braces)
- `Match:<toks>` (matches specific token sequence; may contain braces)

Round-tripped as Plain, each of these silently reads the WRONG thing. The
`DefToken {}{}` signature of `\@ifnextchar` becomes `{}{}{}` — now user
code `\@ifnextchar[{yes}{no}` tries to parse `[` as a balanced group.
Livelock follows (tokenize pipeline can't recover).

**Fix (v2 format, commit fc45e068):** Add a 5th tab-separated field to E
entries that carries `Parameters::stringify()`. Reader prefers `<proto>`,
falls back to `"{}".repeat(nargs)` when proto fails to parse.

**Residual gap:** `Parameters::stringify` produces `"Until:\end{verbatim}"`
for delimited-with-brace params; `parse_parameters`'s `PARAMSPECT_CHECK_RE`
stops at `{`, so the tail mis-parses as a separate nested Plain with inner
type "verbatim". Tests still pass because:
- the v3 structured Parameter sub-line encoding (commit `3e1f89eb2`)
  carries `(name, spec, extra)` per Parameter, bypassing
  `parse_parameters` for catcoded delimiters. See
  `archive/DUMP_FORMAT_PERL_ANALYSIS_2026-04-30.md`.
- the v2 reader falls back gracefully when v3 sub-lines are absent.

**Key insight:** `Parameters::stringify` is NOT a true inverse of
`parse_parameters`. The active strict-Perl `LoadFormat` dump install
relies on the v3 structural encoding to keep `Until:`/`Match:` /
`DefToken` parameters faithful through the dump round-trip.

**Sentinel:** When a dump-loaded CS invokes with unexpected input
interpretation — e.g. `\@ifnextchar[` reads `[{yes}` as arg #1 — check
whether the CS's prototype includes a non-Plain parameter type that
round-tripped as Plain.

---

## 34. The \makeatletter autoload doesn't fire during `--init` raw-load

**Discovery:** During D0 d.1 investigation I kept expecting `latex_base.rs`
to be loaded during `--init=latex.ltx`, because `tex.rs` installs
`\makeatletter` as an autoload trigger (expands to `\@load@latex@pool
\makeatletter`). An env-gated `eprintln!` at the top of `latex_base.rs`'s
`LoadDefinitions!` block never fired during `--init`. Yet the dump still
captured `\documentclass`, `\@ifnextchar`, etc. — leading to a puzzling
"how does the LaTeX kernel get into the dump if `_base.rs` doesn't run?"

**Analysis:** Two mechanisms deliver LaTeX-kernel CSes into state at
`--init` time:

1. **Raw latex.ltx processing** (what `--init` explicitly does). When the
   tokenizer hits `\long\def\@ifnextchar#1#2#3{…}` mid-file, the engine's
   `\def` primitive installs the token-based Expandable directly — no
   `.pool.ltxml` dispatch needed. Most kernel macros are defined this way.

2. **Autoload trigger** (what *should* load `_base.rs`). When the
   tokenizer hits a `\makeatletter` invocation (not the `\def`
   redefinition), it expands the autoload DefMacro → `\@load@latex@pool`
   primitive fires → dispatches to `LaTeX.pool` → loads `latex.rs` →
   loads `_bootstrap`, `_base`, old dump, `_constructs`.

The subtle part: in `--init` mode, latex.ltx's `\makeatletter` is
REDEFINED early (line ~15 of latex.ltx: `\def\makeatletter{\catcode`\@11…}`)
BEFORE it gets INVOKED anywhere. After the redefinition the autoload is
gone — so `\@load@latex@pool` never fires.

That's why our dump contains most of the kernel (from raw `\def`s) but
misses 20 `_base.rs`-only CSes like `\@tempa`, `\xpt`, `\MakeTextLowercase`:
those CSes have NO corresponding `\def` in raw latex.ltx, and the
autoload path that would define them via `_base.rs` never fires.

**Fix:** D0 d.1 landing (commit ddee6952) explicitly calls
`latex_base::load_definitions()` from `ini_tex.rs` right after the
bootstrap snapshot. The surgical preload puts `_base.rs`'s closures/mocks
into state before raw-load starts; any of them that latex.ltx's raw
`\def` later overrides gets replaced with the tokens version (which is
what we want); the ones latex.ltx doesn't touch stay as-is and end up
in the dump via the diff.

**Key insight:** Autoload triggers only fire on LOOKUP, not on
redefinition. If a CS you expect to trigger autoload gets `\def`-ined
before any invocation, the autoload is dead code. This is Perl parity —
Perl LaTeXML has the same subtlety — but it's easy to miss when
tracing the Rust side in isolation.

**Sentinel:** If `_base.rs` or any `.pool.ltxml`-backed module seems not
to be loading, check whether the autoload trigger CS gets `\def`-ined
before invocation in the source TeX. Either invoke it explicitly
earlier, or surgically preload the module.


## 35. Perl silent-coerce vs Rust panic — a recurring parity trap

**Discovery:** A sweep through `.expect(...)` / `.unwrap()` sites turned
up ten distinct cases (9 fixes across the cycle) where Rust panicked
on input Perl silently handled. The common thread: Perl's implicit
numeric / boolean / truthy coercion lets "bad" input flow through as
`0` / `""` / `false`; Rust's strict Result/Option propagation turns the
same input into a crash.

**Why it matters:** Real-world documents contain surprising tokens
(stray `#0`, user-redefined section macros passing non-numeric level,
undefined length registers, rowspan typos). Perl emits a diagnostic and
continues; our port used to abort the whole conversion.

**Examples that landed this session:**
- `Number::from(String)` / `Float::from(String)` panicking on
  non-numeric input → `.unwrap_or(0)` / `.unwrap_or(0.0)` (matches
  Perl's `Number("abc")` + arithmetic → 0).
- `Dimension::spec_to_f64` panicking on `"pt"` (SPEC_RE allows empty
  numeric capture).
- `\setlength{\undefined}` panicking via `.expect("Variable must have
  a Register definition.")` → Perl's `return unless $defn && …`.
- `\@startsection` panicking if level arg isn't numeric.
- `rowspan="abc"` panicking in alignment header heuristic.
- `Mouth::has_more_input` panicking on `fill_buf()` I/O error.
- `List` font walk panicking on one box's font-resolution error.
- `clean_id` stripping idiom via wrong capture name (`$inner` vs
  `$label`) — silent data loss rather than crash, but same class.
- `input()` quote-unwrap `while` loop checking unchanged variable →
  infinite loop on `\input{"file"}`.

**How to spot next time:**
1. Grep `.expect(` in the crate you're auditing.
2. Cross-reference each site with its Perl counterpart — look for
   `$x || 0` / `defined $foo ? ... : ...` / `return unless $defn`.
3. If Perl has a fallback path and Rust has a panic path, fix to
   match Perl. Add a regression test if the path is plausibly reachable.

**Sentinel:** When the comment on a `.expect(...)` starts with
"should never", "has no reason to fail", or "TODO: handle malformed
values here", treat it as a parity gap to investigate, not a
design assertion.

## 36. `rebuild_idstore_from_dom()` timing: finalize-only, not Rewriting-entry

**Context:** The post-processor's `idstore` maps `xml:id` → libxml2
`Node`. Historically, upstream passes (math-parser `replace_tree`,
various `unbind_node()` sites) dropped xml:id-bearing subtrees
without calling `unrecord_id`, leaving dangling-Node entries that
later passes could dereference and SIGSEGV (originally seen on
arxiv:1605.08055; fixed in `337c1ef52` by adding
`rebuild_idstore_from_dom()` at `finalize()` entry before
`prune_xmduals`). Cycle 72 audited the specific call sites
(parser.rs:456/639/690/856, rewrite.rs:522) and confirmed they
now all have proper `unrecord_node_ids` / `remove_node` cascade
coverage — so the rebuild at finalize is belt-and-suspenders
pending 10k-sandbox re-verification on 1605.08055 (see
SYNC_STATUS.md D3b [~] entry).

**Wisdom:** do NOT also call `rebuild_idstore_from_dom` at the start
of the Rewriting phase. Tried in session 128, broke `split_test`.
When the DOM has duplicate xml:ids (rare but possible during
math-parse), `findnodes` visits in document order so the FIRST-
OCCURRENCE node wins the cache entry, but the prior idstore state may
have had the LAST-OCCURRENCE node — which some rewrites depend on.
Finalize is late enough that those rewrites have already fired, so
the rebuild there is safe; at Rewriting-entry it isn't.

## 37. `Document::safe_unlink` is mandatory for node drops

**Context:** `libxml::tree::Node::unlink()` detaches a node from its
parent but leaves any xml:id entries in the post-processor's idstore
pointing at the now-orphaned subtree. Subsequent `dref_by_id` calls
return nodes that may have been freed, producing SIGSEGV.

**Wisdom:** every raw `node.unlink()` site in latexml-oxide must route
through `Document::safe_unlink` unless one of these safe patterns
applies:
- **save-and-reparent** (`unlink` then immediately `add_child` /
  `add_prev_sibling` / `append_tree`) — the id survives the move.
- prior `unrecord_node_ids(node)` walk.
- text / non-element nodes only (no xml:id possible).
- routed through guarded `document.remove_node` / `document.replace_node`.

`safe_unlink` is the id-cache-invalidating wrapper: recurse via
`remove_node_aux` to `unrecord_id` the subtree, then call `unlink`.
The audit of every site in `latexml_core` / `latexml_post` /
`latexml_math_parser` is complete (round-17 cycles 51–58); new drops
should use the guardian by default.

## 38. `\vspace` kept as no-op stub; faithful port triggers moderncv paragraph-break regression

**Context:** Perl `latex_constructs.pool.ltxml` L4692 defines
`DefMacro('\vspace OptionalMatch:* {}', '\vskip #2\relax');` — a pure
token-replacement macro. Rust `latex_constructs.rs:7206` instead has
`DefPrimitive!("\\vspace OptionalMatch:* {}", None)` (empty body,
silently drops the argument).

**Why the divergence:** a prior port attempted the faithful DefMacro
wiring and regressed the `moderncv/cs_cv.tex` test — `\vskip` in Rust
digested as vertical-mode glue triggered an implicit `\par` when
encountered in horizontal mode, breaking paragraphs that moderncv
intended to keep intact. Perl's `\vskip` binding apparently produces
a Whatsit without the paragraph-break side effect.

**Wisdom:** **do NOT** flip `\vspace` to Perl-matching DefMacro as a
naive Def*-parity fix. The kind swap is load-bearing — it hides a
deeper asymmetry in Rust's `\vskip` horizontal-mode handling. The
proper path to parity is:
1. Port `\vskip` so its horizontal-mode digestion matches Perl (no
   auto-\par).
2. Then restore `\vspace` to `DefMacro!('\\vspace OptionalMatch:* {}', '\\vskip #2\\relax')`.

Verify fix against `moderncv/cs_cv.tex` + any other `\vspace`-using
regression tests before landing. Without step 1, step 2 breaks moderncv.

## 40. `\#`/`\&`/`\%`/`\$` Def*-kind mismatch is intentional mode-split

**Context:** Perl `plain_base.pool.ltxml` L70-76 defines each as a
single `DefPrimitive` with a sub body that emits `Box('#', undef,
undef, T_CS('\#'), role => '…', meaning => '…')` and similar. The Box
carries role/meaning that double as text-mode and math-mode markers,
converted downstream by the math parser / post-processor.

Rust `plain_base.rs:62-68` instead uses `DefMacro` with `\ifmmode`
dispatch into mode-specific helpers: `\lx@text@hash` (DefPrimitive
emitting a text Box) and `\lx@math@hash` (DefMath emitting an XMath
token directly).

**Wisdom:** do NOT "fix" this Def*-kind mismatch by collapsing to a
single Perl-matching DefPrimitive. The Rust split is a genuine
semantic improvement — it emits proper XMath tokens in math mode at
stomach level, rather than relying on post-processing to promote a
text Box into a math token. Reverting loses mode-precision.

If the Def*-parity audit flags these, the right resolution is to
record them as an intentional divergence in OXIDIZED_DESIGN.md, not
to kind-flip.

**Same direct-emission improvement in texvc_sty.rs (30 entries).**
Perl `texvc.sty.ltxml` defines MediaWiki's math subset as simple
expansion aliases: `DefMacroI('\N', undef, '\mathbb{N}')`,
`DefMacroI('\darr', undef, '\downarrow')`, etc. Rust redefines these
as direct DefMath emissions with explicit semantic markup:
`DefMath!("\\N", None, "\u{2115}", role => "ID", meaning =>
"natural-numbers")`. Both produce the same visible math symbol
(ℕ, ↓, etc.), but Rust's version carries `role`/`meaning`
attributes that Perl's alias-chain loses by the time it reaches
MathML output. All 30 texvc DP mismatches fit this shape — do NOT
kind-flip; the Rust version is strictly more informative for
accessibility/semantic consumers of the XML. Same categorization
applies to any package binding where the audit shows `Perl=DefMacroI
→ Rust=DefMath` for a symbol-alias CS.

## 41. Math-mode Def*-kind mismatches are usually structural, not parity bugs

**Context.** The Def*-parity audit (`tools/audit_def_parity.py`) flags
math-mode CSes whose Rust kind differs from Perl's. Most are structural
adaptations for missing Rust ParameterTypes, not parity bugs.

**The four intentional/blocked cases:**

| CS | Perl | Rust | Root cause |
|----|------|------|------------|
| `\mathchar` | `DefPrimitive('\mathchar Number', …decodeMathChar…Box)` | `DefConstructor("\\mathchar Number", "<ltx:XMTok …>", after_digest => …)` | Rust emits `<ltx:XMTok>` directly; Perl emits a Box the post-processor promotes. Rust is the more precise shape — kind-flip would regress output. |
| `\left` / `\lx@right` | `DefConstructor('\left TeXDelimiter', "#1", …)` | `DefMacro!("\\left XToken", sub { …manual \delimiter<Number> handling… })` | Rust's `TeXDelimiter` ParameterType is incomplete — see detailed plan below. Current DefMacro workaround at `tex_math.rs:836` peels `\delimiter` + reads number + decodes glyph manually. |
| picture primitives (`\line`/`\vector`/`\oval`/`\qbezier`/`\lx@pic@bezier`) | `DefPrimitive('\\line Pair:Number {Float}', …)` | `DefMacro!` unpacking `Match:( Until:, Until:) {Float}` into 3 args + forwarding to `\lx@pic@XXX{}{}{}` DefConstructor | Rust lacks `Pair:Number` as a ParameterType. Same functional parity, different factoring. |
| amsmath `\aligned` / `\alignedat` | `DefConstructor('\aligned alignsafeOptional {}', …)` | `DefPrimitive!` with explicit `local_align_group_count(1000000)` + manual `gullet::read_optional` + unread | Rust lacks `alignsafeOptional`. Plain `[]` would trip handle_template's `&`-interception inside nested alignments. See `amsmath_sty.rs:1168`. |

**Wisdom:** do NOT flip these to Perl-matching kinds naively. Each is
load-bearing. The proper path to parity for any of them goes through
porting the missing ParameterType first, then migrating the call sites.

### ParameterType port candidates (ROI-ordered)

Engine/ alone has **23 call sites** using these three ParameterTypes
(grep `Pair:Number|PairList|TeXDelimiter|alignsafeOptional`). Package
bindings add more.

- **TeXDelimiter** — 10+ entries (tex_math `\left`/`\lx@right` 2,
  revsymb `\biglb`/`\bigrb`/`\Biglb`/… 8, plus others). Highest ROI.
  Already partially exists at `base_parameter_types.rs:693` (per Perl
  PR#2596) — enhancement needed, not new port. Plan below.
- **Pair:Number** (+ `PairList`) — 5-10 entries (picture primitives
  + engine call sites). Medium ROI.
- **alignsafeOptional** — 2-4 entries (amsmath `\aligned`/`\alignedat`).
  Lowest ROI but simplest port (reads `[…]` with alignment-safe
  wrapping).

### TeXDelimiter enhancement plan (current truth, cycle 64 verified)

**Rust already has `TeXDelimiter`** at `base_parameter_types.rs:693`
and it's used successfully by `\big`/`\Big`/`\bigg`/`\Bigg` at
`math_common.rs:962-964`. The current implementation uses
`gullet::read_arg(ExpansionLevel::Partial)` (braced arg). The `\left` /
`\lx@right` / revsymb `\biglb` family bypass it via DefMacro because
the reader differs from Perl's in two dimensions:

**Dimension 1 — reader shape (3 branches missing vs Perl
`TeX_Math.pool.ltxml:709`):**
```perl
$gullet->skipFiller;
my $token = $gullet->readXToken(0);               # single X-token, not read_arg
if ($token && $token->getCatcode == CC_BEGIN) {   # BEGIN-unwrap once
  $gullet->unread($gullet->readBalanced(1));
  $gullet->skipFiller;
  $token = $gullet->readXToken(0); }
$token = T_CS('\lx@delimiterdot') if !defined($token) || ToString($token) eq '.';
my ($delim) = $STATE->getStomach->invokeToken($token);  # ← see dim 2
return $delim;
```
All three branches need porting (single-X-token read, BEGIN-unwrap,
`.`/undef → `\lx@delimiterdot`).

**Dimension 2 — `undigested => 1` is architectural, not a macro flag.**

- `ArgWrap` (`latexml_core/src/definition/argument.rs:24`) has no
  `Digested` variant.
- `Parameter` (`latexml_core/src/parameter.rs:48`) has no
  `undigested: bool` flag.
- The existing `digested_reversion` hook on Parameter fires only on a
  code path that reader-produced Digested values never currently reach.

Closing this is the real blocker for `\left\delimiter<Number>`:
without `invoke_token` being called from the reader, `\delimiter`'s
number-reading primitive never consumes the following `<Number>`, so
it dangles — which is exactly what `tex_math.rs:836`'s DefMacro
workaround manually peels back. To add `undigested` semantics, either:
- **(a)** extend `ArgWrap` with a `Digested(Box<Digested>)` variant +
  plumb through `be_digested` as identity when already Digested
  (cross-cutting across every arg-pipeline site), OR
- **(b)** add `Parameter.undigested: bool` + a bypass-re-digestion
  branch in the digestion-of-args phase (less invasive).

**Scope: one full dedicated session touching latexml_core.** Partial
progress via reader-only port (3 branches without `invoke_token`) is
possible but closes ZERO DP audit entries — the call-site migrations
need BOTH reader and `undigested` to work, since `\left\delimiter<num>`
still breaks without the digested path. Cycle 64 verified this.

**Prerequisites (confirmed exist):** `stomach::invoke_token`
(`stomach.rs:776`), `gullet::skip_filler` (`gullet.rs:1203`),
`gullet::read_x_token` (`gullet.rs:503`), `gullet::read_balanced`
(`gullet.rs:716`), `\lx@delimiterdot` (`tex_math.rs:1184`).

**Call sites to migrate once architecture is in place:**
- `tex_math.rs:836` `\left` — replace DefMacro+manual peel with
  `DefConstructor!("\\left TeXDelimiter", "#1", …)`.
- `tex_math.rs:1192` `\lx@right` — same.
- `revsymb_sty.rs:14-21` 8 `\biglb`/`\bigrb`/`\Biglb`/`\Bigrb`/
  `\bigglb`/`\biggrb`/`\Bigglb`/`\Biggrb` — each becomes
  `DefConstructor('\X TeXDelimiter', '#1', …)`.

**Expected outcome:** 1097/0/0 tests green, DP audit shows 10+ entries
cleared, `tex_math.rs:836` workaround removed, revsymb `\biglb` family
collapses back to audit-clean DefConstructor form.

### Broader takeaway

For a Def*-kind mismatch audit, expect a sizable fraction to be
structural adaptations (mode-splits, direct XML emission,
parameter-type gaps), not parity bugs. Read the Perl body first; if
the Rust shape is more precise or solves a missing-feature gap, the
mismatch is likely intentional and belongs in OXIDIZED_DESIGN.md
rather than a fix queue.

## 42. AmSPPT DefConstructor→DefMacro "shim" pattern

**Context:** Perl's `amsppt.sty.ltxml` ports Plain AMS-TeX typesetting
primitives with full XML-structured DefConstructor definitions —
e.g. `DefConstructor('\specialhead Until:\endspecialhead',
"<ltx:chapter inlist='toc' xml:id='#id'>#tags<ltx:title>#1</ltx:title>", bounded=>1, properties=>…)`.

Rust's `amsppt_sty.rs` instead provides **LaTeX-equivalent aliases**:
`DefMacro!("\\specialhead", "\\section*")`, and similar for
`\proclaim`, `\definition`, `\remark`, `\example`, `\demo`, `\roster`,
`\footnote`, etc. (10+ DP audit mismatches from this pattern).

**Wisdom:** amsppt is Plain AMS-TeX (pre-LaTeX); Rust pragmatically
reuses LaTeX's section/environment machinery via aliases rather than
reimplementing the XML-structuring DefConstructors. For arXiv content
(where amsppt is rare), "close enough to LaTeX" output is acceptable
and the full port isn't justified by usage frequency. Do NOT kind-
flip these entries — the flip alone loses semantic content; the flip
plus porting bodies is a multi-day effort justified only by
documented amsppt-in-arXiv evidence.

## 43. `\hook_use:n{begindocument}` dispatch is a Rust-only compensator

**Context:** Perl LaTeXML treats l3hooks as a block of no-op stubs
(`latex_base.pool.ltxml` L829-855) — no hook storage, no dispatch, no
ordering engine. `\hook_use:n` in Perl is a no-op that swallows its
argument.

**Wisdom:** the `latex_constructs.rs:2501` `\hook_use:n{begindocument}`
dispatch is NOT a parity gap — it is a Rust-only compensator for our
raw `expl3-code.tex` load path (active when the dump doesn't short-
circuit it). That path really does define `\hook_use:n` and enqueues
hook code against it; Perl doesn't load `expl3-code.tex` so doesn't
need the dispatch. Keep the gate; removing it silently regresses
the raw-load path. Any future "clean up expl3 support" pass must
preserve this compensator or replace the raw-load path first.

## 44. `DefMacro(sub{…})` vs `DefPrimitive(sub{…})` are NOT interchangeable

**Correction to an over-broad recent pattern** (several 2026-04-23
breadcrumbs claimed a blanket equivalence — wrong).

The two kinds agree on the **shape of the Perl body** (a sub that may
have side effects and may return tokens), but they differ on **when
and how the gullet sees the CS**:

| Property | `DefMacro(sub{})` | `DefPrimitive(sub{})` |
|---|---|---|
| Expandable? | yes (gullet-level) | no (stomach-level) |
| `read_x_token` over the CS | fires the sub, substitutes return | returns the CS token as-is |
| Inside `\edef` / `\protected@edef` | sub runs, return inlined into definition | CS frozen as-is in the body |
| `\ifx \cs \other` | compares expansion | compares primitive identity |
| `\expandafter \cs` | triggers one expansion step | unchanged |
| Side-effect timing | gullet-time (before stomach) | stomach-time |

**Operational takeaway.** A Rust `DefPrimitive!(cs, sub{…})` is only a
safe port of a Perl `DefMacro(cs, sub{…})` **if every call-site of the
CS occurs in a non-expansion context** — i.e., the CS is always invoked
at stomach time, never peeked by `read_x_token`, never captured by
`\edef`, never compared via `\ifx`. For most state-mutating package
helpers (e.g. `\DefineNamedColor`, `\lx@unactivate`,
`\set@deluxetable@template`, `\lx@makecell@head`) the invariant does
hold in practice — but the correctness is per-CS, not by-pattern.

For gullet-reactive helpers (`\xspace` reads the next token; `\xglobal
Token` peeks and decides; `\pgf@circ@stripdecimals Until:…` slices an
argument stream) the distinction is observable and the two kinds are
**not equivalent** in general. Those cases can still work because:
- the outer protocol (what tokens follow the CS) dictates whether the
  stomach-time consumption order matches the gullet-time expansion
  order, AND
- the CS is never placed inside a protected `\edef` or `\ifx` capture.

When triaging a Perl `DefMacro(sub{})` → Rust `DefPrimitive(sub{})`
mismatch, the right breadcrumb is **not** "WISDOM #41" (that entry is
about math-mode structural ParameterType adaptations). The correct
triage is:
1. Name the gullet contexts that could observe the CS (calls from
   `\edef`, `\ifx`, `\expandafter`, anything peeking with `readXToken`).
2. Confirm none of them fire for this CS in practice (grep, or a
   comment in the surrounding code that documents the invocation
   contract).
3. If confirmed, the DefPrimitive port is safe; otherwise it is a real
   parity gap and needs a genuine DefMacro / sub-with-token-return.

**Audit-tool consequence.** The Def*-parity audit surfaces every
`DefMacro → DefPrimitive` mismatch. Most pass the per-CS test, but
dismissing them all by pattern is unsafe. When in doubt, err toward
keeping Perl's kind and porting the sub body as a DefMacro with
gullet-token return.

**A FOURTH gullet context the triage above missed: ALIGNMENT column-scan
(added 2026-05-31).** If the `sub{}` reads a **non-brace DELIMITED argument**
(`(…)`/`[…]`/`<…>` via `phys_read_arg`/`readBalanced`-style) whose content can
contain `&` or `\\`, and the CS may appear **inside an alignment** (`eqnarray`,
`align`, `\halign`, matrix), then DefPrimitive is NOT safe: the alignment's
column reader (`digest_alignment_column`) scans the row for `&`/`\\` at
STOMACH time, and a digestion-time primitive hasn't yet consumed its
delimited body — so the alignment grabs the body's `&`/`\\` as its own column/
row separators, splitting the construct and orphaning its fences. A DefMacro
grabs the delimited body at EXPANSION time (before the column scan), like Perl.
Witness: `\mqty(a&b\\c&d)` inside an `eqnarray` (2007.06211) — Perl 0, Rust 11
(`\lx@begin@alignment … mode-switch … due to \lx@begin@inmath@text` + Unbalanced
`\right`). Fix: `physics_sty.rs` `\lx@physics@mat` reverted to `DefMacro`
(commit 6721f53232). The OTHER physics quantity constructs (`\quantity`/`\qty`,
`\lx@physics@fenced`→`\pqty`/`\abs`/`\norm`/`\order`, `\evaluated`,
`\lx@physics@operator/operatorP`, `\lx@physics@diff`) keep their deliberate
DefPrimitive (this entry's ~16-flip rationale) because their delimited body is a
single EXPRESSION with no `&`/`\\` — only the MATRIX family carries alignment
separators, so only it needs the macro kind. **Triage step 1 must therefore add:
"…and if the sub reads a delimited (non-brace) arg that can hold `&`/`\\`, can
the CS occur inside an alignment?"** See [[project_physics_mat_defmacro_not_primitive]].

## #45 Rust `mode => "text"` auto-implies `enter_horizontal => true`

When porting a Perl `DefConstructor` that carries
`mode => 'restricted_horizontal', enterHorizontal => 1`, the Rust
equivalent is `mode => "text"` alone — do NOT add
`enter_horizontal => true` on top. The translation happens in
`latexml_core/src/binding/def/dialect.rs:331-355`:

```rust
// Perl: mode => 'text' becomes restricted_horizontal + enterHorizontal
let mut needs_enter_horizontal = options.enter_horizontal;
let mode = if options.mode.as_deref() == Some("text") {
  needs_enter_horizontal = true;
  Some("restricted_horizontal".to_string())
} else {
  options.mode
};
```

This applies to `DefConstructor`, `DefEnvironment`, and `DefMath`
(three sites in `dialect.rs`).

**When the explicit flag IS required:** Perl entries that carry
`enterHorizontal => 1` with *no* `restricted_horizontal` mode (so
Rust uses `mode => "restricted_horizontal"` verbatim, or no mode at
all). Examples: `\ref`, `\lx@bibitem`, `\lx@bibnewblock`, `\@@bibref`,
`\lx@@verbatim`.

**Parity-sweep triage:** When scanning Perl for enterHorizontal gaps,
filter out entries that already have `mode => 'restricted_horizontal'`
on the same or an adjacent line — the Rust `mode => "text"` picks
up the flag automatically, and any explicit `enter_horizontal =>
true` on such a call is a harmless no-op that adds visual noise.

## #46 `NewCounter(..., idprefix => 'X')` silently decays to empty prefix when routed through `\newcounter`

**Finding (cycles 225/226):** Three Rust bindings had counter
declarations that lost their Perl `idprefix => '<prefix>'` option:

- `aas_support_sty`: `\@appendix` reset — `new_counter("equation",
  "section", None)` (was missing Perl's `idprefix => 'E'`)
- `subfig_sty`: subfigure/subtable — routed through `RawTeX!
  ("\\newcounter{subfigure}[figure]")` (raw `\newcounter` has no
  `idprefix` keyword; the LaTeXML option is lost)
- pre-existing subfigure_sty and subfloat_sty already correct

**Mechanism:** Perl's `NewCounter(...)` takes idprefix as a keyword
and wires it into LaTeXML's id-registry. LaTeX's `\newcounter`
takes only `[within]`; no way to express idprefix. So when a Rust
port uses `RawTeX!("\\newcounter{X}[Y]")`, the counter is created
without an idprefix → document IDs fall back to empty. The collision
surfaces on the *second* instance of the parent (e.g. second
appendix, or second figure with subfigures) since the first counter
value has no prefix-namespace separation.

**Detection pattern:**
```
for each Perl file with `idprefix =>`:
  count Perl idprefix occurrences
  count Rust idprefix=>"..." occurrences in same-named binding
  if Perl > Rust → audit
```

**Fix template:** replace `RawTeX!("\\newcounter{C}[W]")` with
`NewCounter!("C", "W", idprefix => "P")`; or convert a bare
`new_counter("C", "W", None)` to
`new_counter("C", "W", Some(NewCounterOptions { idprefix: "P", ..Default::default() }))`.
See commits `8fb8bf569`, `d79d1a2e4` for concrete examples.

**Don't over-apply:** theorem/spnewtheorem counters delegate to
`define_new_theorem` which builds `idprefix => "Thm{name}"` itself;
those bindings show as "deficits" in counted grep but aren't.

## #47 `rust-libxml` `Node::clone` is Rc refcount bump; `_Node::drop` may call `xmlFreeNode`

**Finding (cycle 236, 2026-04-23):** `latexmlpost_oxide` was SIGSEGVing
on `$X$` plus an ar5iv preload. Root cause: the `rust-libxml`
crate models `Node` as `Rc<RefCell<_Node>>`. When a `_Node` is dropped
with `unlinked == true`, the `Drop` impl calls `xmlFreeNode` on the
raw pointer. For nodes that are conceptually *doc-owned* (still
reachable via the document tree, or still referenced by the idcache /
objectDB) but temporarily held in a local `Node` handle, letting the
local drop fire invokes `xmlFreeNode` on memory that `xmlFreeDoc` will
free again at program end → UAF.

**Symptom shape:** segfault at process teardown or at the drop of a
post-processing phase's working set — not during the XML emission
itself. The stdout XML/HTML reaches disk before the crash fires.

**Fix pattern:** the `DocOwnedNode` RAII wrapper
(`latexml_post/src/doc_owned_node.rs`) holds a `ManuallyDrop<Node>`
that never runs the inner drop. Use it at exactly the sites where a
`Node` handle is extracted and then dropped, but the underlying
libxml2 allocation must remain live for the Doc to free:
- `PostDocument::drop` idcache teardown
- `math_processor::process_math_node` after `preremove_nodes` +
  `remove_nodes` of the xmath subtree

**What *not* to do:** scattered `mem::forget(node.clone())` ad-hoc.
That works but masks intent and leaks the wrapper-level Rc counts on
every call path. The RAII wrapper has one construction site, makes
ownership explicit at the type level, and is what
`safe_unlink`-adjacent reasoning should live under.

**Upstream fix path:** expose `_Node::set_linked()` from rust-libxml so
callers can toggle the "I own the allocation" flag without going
through `ManuallyDrop`. Until then `DocOwnedNode` is the local
workaround.

**Related:** WISDOM #37 `Document::safe_unlink` is mandatory; this
entry is the complement — unlinking is not always safe when the Doc
still has the node under management.

**Reproducer:** `docs/known_crashes/min_xmath_xmlid.tex` plus
`--preload=ar5iv.sty` triggers the old crash on 5/5 runs with the
pre-fix binary.

## #48 Scan/default_handler's Perl→Rust size asymmetry demands a `<Math>`-skip

**Finding (cycle 239, 2026-04-23):** `latexml_post::scan::Scan` was
registering every XMTok/XMApp/XMRef/XMWrap/XMDual inside `<Math>` into
the ObjectDB, making Scan dominate post-processing wall time for
math-heavy papers (arXiv:0705.0790: 11.4 s of 17.8 s total, 65K nodes
registered, ~98% of which have no downstream use).

**Why it's a *Rust-specific* problem:** Perl LaTeXML's core does not
emit `xml:id` on XM* nodes at all. Its `Scan::default_handler`
short-circuits on `$id` being undef, so the inner-math tree is
effectively skipped. The Rust port via ar5iv's `_ID_counter__` pattern
*does* emit xml:id on every descendant (needed for XMRef idref
resolution inside the math tree), so the literal Perl port of
`default_handler` dutifully processes every one.

**Fix shape (commit `0bc04e3eb`):** add an explicit `ltx:Math` branch
in the dispatch that registers the outer Math element's id and then
*returns without `scan_children`*. XMRef still resolves because
`PostDocument::idcache` is built at parse time (not via Scan) and
retains every xml:id. Only the ObjectDB entries for XM* descendants
are skipped, which is correct because cross-reference / index /
bibliography don't target math-internal ids.

**Secondary cleanup in `default_handler`:** move `collect_common`
*inside* the `if id.is_some()` — previously it ran for every node,
built a `ScannedProps`, and discarded it on id miss. Pure perf.

**When this is an intentional divergence from Perl and when not:** the
Math handler is *structurally Rust-specific* — Perl has nothing
to port. Note it in the code comment as a divergence, not a bug.
The `collect_common`-guard is a literal Perl parity improvement
(mirrors Perl Scan.pm L272-283).

**Don't over-apply:** other subtrees *may* legitimately carry
downstream-needed xml:ids (e.g. `ltx:figure`, `ltx:note`); those are
handled by their own dispatch branches and already register properly.
The Math skip is load-bearing specifically because XM* descendants are
an ar5iv-preload artifact.

## #49 Indirect-model memoisation must keep the max desirability, not the first

**Symptom observed as:** `paralists_test` failing in the test-harness
while the CLI binary (`latexml_oxide`) passed — `inparaenum` item
bodies wrapped in `<picture xml:id="…pic1">` under test, but not via
the bin. Earlier drafts of this entry blamed a harness vs binary
divergence; that was wrong.

**Actual root cause:** `latexml_core::common::model::compute_indirect_model_aux`
memoised `desc[kid][start]` on *first visit* and skipped any later
path. In the LaTeXML schema both `ltx:text` (autoOpen 1.0) and
`ltx:picture` (autoOpen 0.5) are valid containers for `#PCDATA`, and
`ltx:text` itself lists `ltx:picture` among its allowed children. When
the recursion explored `ltx:text → ltx:picture → #PCDATA` before the
direct `ltx:text → #PCDATA` child (which happens whenever the
`HashSet`-backed `contents(ltx:text)` iteration yields `ltx:picture`
first), it inserted `desc[#PCDATA][ltx:text] = 50` — the path
desirability after picture's 0.5 attenuation — and blocked the 100
score from the direct child. In the outer ranking loop at
`state.rs::compute_indirect_model` the stored 50 tied with
`desc[#PCDATA][ltx:picture] = 50`, and alphabetical sort put picture
first, so `imodel[inline-item][#PCDATA] = ltx:picture`. Process hash
seed determined iteration order, so the bin and test binaries picked
different outcomes on the same input.

**Fix:** Replace the "skip if already present" memoisation with a
"skip only if prior ≥ current" check so max-desirability wins
regardless of iteration order (model.rs ~L790). Remove WISDOM #49's
old claim and the corresponding paralists ignore entry in
`testable.rs`.

**Reproducer (historical):**

```bash
# Pre-fix: either the bin or the test binary would produce the picture
# wrap depending on the process hash seed.
LATEXML_SAVE_ACTUAL=1 cargo test --tests -p latexml paralists_test --include-ignored
diff /tmp/latexml_actual_paralists.xml latexml_oxide/tests/structure/paralists.xml
```

**When to apply:** Any auto-open regression where two openable tags
compete for the same child (e.g. `ltx:text` vs `ltx:picture`,
`ltx:p` vs `ltx:para` in `_CaptureBlock_`). Check that the indirect
model returns the *maximum*-scoring intermediate, not the
first-inserted one. Add a sorted tag iteration if determinism is
needed beyond desirability ranking.

## #50 Vendor-class size/layout `\PackageError` / `\GenericError` is moot in XML→HTML output — silence them

**Meta-principle.** LaTeXML and our Rust port produce *structured XML
and derivative formats* (HTML, MathML, ePub, JATS). We never produce
PDF — we don't run line-breaking, page assembly, justification, or
typesetter-grade dimension reconciliation.

Class / package files vendored by publishers (revtex, IEEEtran,
AISTATS, ACM, Springer Nature, etc.) routinely include defensive
`\PackageError{X}{...exceeds size limitations...}` / `\GenericError`
calls that fire when the typeset PDF would overflow a column,
header, or page region. These guards exist to alert the AUTHOR that
their PDF will look wrong. In our paradigm, the guards are
load-bearing on dimension semantics we cannot — and should not —
faithfully reproduce. We compute box dimensions heuristically (font
metric × char count, paragraph wrap at `\hsize`), and the heuristic
is necessarily off from real TeX. So:

* If we COMPUTE the dimension to match real TeX exactly, the guard
  fires when the PDF would overflow — and we emit an `Error:` that
  the conversion is otherwise fine on.
* If we compute it differently (we always do), the guard fires
  spuriously in cases where real TeX would have been silent — also
  an `Error:`.

Either way, errors emitted by these guards are signal-free
diagnostics about a typesetting outcome that never happens in our
pipeline.

**Rule:** when a vendor class fires `\PackageError`/`\GenericError`
whose message is *purely about size, layout, position, or page-fit*
("exceeds size", "too long", "too wide", "too tall", "breaks the
line", "doesn't fit", "running heading", "overflows", etc.),
**silence or downgrade the error**. Match Perl LaTeXML's behaviour
when we know it: Perl often gets the dimension different too and
also silently passes the guard. The signal we care about is
*semantic* (missing macros, malformed structures, undefined refs),
not *typographic*.

**How to apply:** classify the message text in our `\GenericError` /
`\PackageError` handlers. A regex over the message body (case-
insensitive, matching the size/layout phrases above) routes the
emission to `Info:` or `Warn:` instead of `Error:`. Do not gate on
the calling class — every publisher class has its own variants of
the same guard.

**Why not "fix" the dimension computation instead?** Each
publisher's guard tests a different combination of `\wd`, `\ht`,
`\dp`, `\baselineskip`, `\hsize`, `\textwidth`. Matching real-TeX
output for every one of them would require porting line-breaking and
page assembly. That's a multi-year undertaking with no semantic
output value.

**Witnesses:**
* aistats2026.sty `\ifdim\ht\autrun>10pt` → `\PackageError{Document}
  {Running heading author exceeds size limitations}` (12 papers in
  stage-1 of the 100k warning corpus, including arXiv:2602.11863).
* aistats2026.sty's analogous `\ifdim\wd\titrun>\textwidth` running
  title check.
* Springer Nature `sn-jnl.cls` `RunningHead` length checks.
* IEEEtran.cls `\ifclassoptioncomsoc` runninghead asserts.
* revtex4_* `\altaffiliation` width checks (related).

## #51 `listings.sty.ltxml` binding flattens upstream `\lst@tagmode` machinery — leaves three latent gaps

**The rule.** The Perl LaTeXML `listings.sty.ltxml` binding is a deep
simplification of the actual `lstmisc.sty` `tag=` / `usekeywordsintag` /
`markfirstintag` mechanism: it never models `\lst@tagmode`,
`\lst@gkeywords@sty`, `\lst@ifusekeysintag`, or `\lst@iffirstintag`. The
binding flattens "tag mode" into a flat regex-driven delimiter walk.

**How to apply.** When a listings issue surfaces on a real paper, do
not assume the Perl binding is authoritative; cross-check
`/usr/share/texlive/texmf-dist/tex/latex/listings/lstmisc.sty` first.
Three concrete divergences worth knowing:

1. `tag=**[s]<>` registration. Upstream enters `\lst@tagmode` so the
   inner content is processed with `\lst@ifkeywords\iftrue` and (when
   `usekeywordsintag=true`) restyled. The Perl binding instead emits
   one `\@listingGroup` span and lets the recursive `lstProcess_internal`
   keep ID matching active. The lex-sort of delim keys in
   `lstProcess_internal` (Perl `sort keys %$delimiters`) makes `<`
   shadow `<!--` in the regex alternation, which is the only reason the
   commentstyle never fires for inline XML comments — preserve this
   sort order in the Rust port (see `listings_sty.rs::lst_process_internal`).

2. `usekeywordsintag` / `markfirstintag` are `DefKeyVal('LST', …)` only.
   The Perl source comment is explicit: `NOT YET HANDLED; I don't even
   understand it`. Don't try to model them from the binding; if needed,
   port the upstream `\lst@AddToHook{Output}{…}` machinery directly.

3. `\@onefilewithoptions` re-option-processing (latex.ltx:15512). Both
   Perl LaTeXML and the Rust port short-circuit on `_loaded` flags, so
   `\usepackage{xcolor}` followed by `\usepackage[dvipsnames]{xcolor}`
   does not load `dvipsnam.def`. pdflatex DOES load it via the modern
   `opt@handler@xcolor.sty` mechanism (DeclareKeys-based). This is a
   deeper parity gap than listings — track separately if a paper-class
   of "late option" bugs grows beyond the listings shadow workaround.

**Why this layout.** Faithfully porting `\lst@tagmode` would require
modeling TeX modes inside Listings, which the Perl binding deliberately
sidestepped — every node in the listings tree would need a mode
stack. For now, mirror the Perl binding's simplifications and only
upgrade when a corpus paper actually exercises one of the gaps.

**Witnesses.**
* `arXiv:2602.15149` — `\lstdefinestyle{xmlstyle}{...commentstyle=\color{ForestGreen}...}`
  with `\usepackage{xcolor}` + later `\usepackage[dvipsnames]{xcolor}`.
  Fixed in the Rust port by faithfully matching Perl's delim-sort
  ordering and registering `tag=<>` as a 2-token split (commit
  `5b8a4f9aca` listings: faithful XML tag / commentstyle parity from Perl).
* `tests/tikz/various_colors.tex` — `moredelim=**[is]…{@}{@}` exposes
  the latent `alsoletter` default (`@$_` bundled with the alphabet)
  ID_RE greedy-eat bug that prevents propagating the `**` recursive
  flag through to `\lst@@delim` / `\lst@@moredelim`. Tracked.

## #52 `FindFile` interpret-mode raw-search is paths-only (NO kpsewhich)

**The rule.** When `INTERPRETING_DEFINITIONS=1` (we're inside a raw-load
context, e.g. one `.sty` file's body invokes `\RequirePackage{foo}`),
the Perl `FindFile` raw-search step (Package.pm L2117-2119) calls
`pathname_find($file, paths => $paths)` — **local paths only**, no
kpsewhich. The ltxml-fallback step (L2120-2123, `FindFile_fallback`)
fires next, BEFORE the unconditional kpsewhich at L2131-2136.

Practical effect: when a raw `.sty` calls `\RequirePackage{<name>}` and
`<name>.sty` ships in TeX Live (but not the user's search paths), the
Perl flow tries the local-paths search (fails), then the ltxml fallback
(which strips trailing version suffixes — `caption3` → `caption`,
`svjour3` → `svjour`, `mn2e` → `mn`, etc.), succeeds with the binding,
and never reaches kpsewhich. The fallback ALWAYS wins over the TL raw
file when the unsuffixed binding exists.

The Rust port previously called `find_file(..., search_paths_only:
options.searchpaths_only)` for the interpret-mode step in
`binding/content.rs::input_definitions`. `searchpaths_only` defaults
to false, so kpsewhich fired and returned the TL raw — short-circuiting
the fallback. Symptom on `caption3`: floatrow.sty raw-loaded caption3.sty
directly, and the hand-port stub `\DeclareCaptionFormat{}{}` missed its
optional `[#1#2#3\par]` bracket → 3+ PARAM-token leaks per
`\DeclareCaptionFormat` call plus a cascade of `\caption@*` undefineds.

**Why this matters beyond `caption3`.** Stage-13 sample showed five
distinct papers hitting `Error:misdefined:#` (6 PARAM each, identical
shape — caption3 cluster). Every "version-suffixed package that has a
binding for its unsuffixed name" follows the same code path; the rule
governs whether the Rust binding or the TL raw file wins.

**Implementation guard.** Step-2 must use `search_paths_only: true`.
Step-4 (the second raw-search, after fallback didn't catch it) must
drop the `!interpreting` gate — Perl's kpsewhich block (L2131-2136)
has no interpreting gate either, and dropping it preserves the
"interpret-mode + no fallback → kpsewhich the raw" path.

**Witness.** arXiv:2506.13435 (caption-package paper, Rust=28→2 after
fix); arXiv:2506.19291 (floatrow → caption3, Rust=30→2). Commit
`feb8832a2b binding/content: Step-2 raw-search paths-only, drop
interpreting gate in Step-4`.

**Why this is more elaborate than it needs to be (parity tax).** The
simpler model — `direct binding → fallback binding → paths (local +
kpsewhich)` — would resolve every realistic arXiv input correctly,
including the caption3 case above. Perl's 5-step ladder only diverges
from the simple model in one scenario: `interpreting=1` AND the raw
`<file>.sty` is present on local paths AND a fallback binding exists.
Perl picks the local raw (Step 2); the simple model would pick the
fallback binding. That divergence exists so a user can drop a custom
raw `<file>.sty` on `--path` and override our fallback binding — an
override pattern we have never observed in arXmliv corpora.

A second latent reason: Perl's `$interpretable =
LookupMapping('INTERPRETABLE_SOURCES', $file)` lets specific files
force-interpret raw even when global `interpreting=0`, AND it
explicitly suppresses Step 3 fallback (`!$interpretable` on L2120).
The Rust port doesn't honor `INTERPRETABLE_SOURCES` today; collapsing
Step 2 would silently violate this gate if we ever wire it up.

We keep the full 5-step order for strict Perl parity per CLAUDE.md
("Perl code is the ground truth"). If a future failure looks like
"Step 2 fired and we lost the fallback binding we wanted," tighten
Step 2 (as `feb8832a2b` did) — do not delete it.

## #53 expl3 intarrays ride `\fontdimen` of `cmr10 at <Nsp>` — consolidate the dump

**The trick.** expl3's `\int_array_new:Nn` allocates an integer
array of N slots by abusing `\font`: it instantiates `cmr10` at a
unique-per-intarray tiny `at <N>sp` size (~1/65k pt — the size is
just a fingerprint), then stores each slot in the new font instance's
`\fontdimen<idx>` register. A fully-initialized expl3 + LaTeX kernel
writes **~89,000 such slots** across ~22 intarrays
(`\c__fp_*_intarray`, `\c__codepoint_*_intarray`, `\g__regex_*_intarray`,
`\c_initex_cctab*`, etc.). They surface in our state Value table
under composite keys like `fontdimen_fontinfo_cmr10 at 15sp_<idx>`.

**The dump-size hit.** Before consolidation, `dump_writer` emitted
one `V\tfontdimen_fontinfo_cmr10 at <Nsp>_<idx>\tD\t<val>` record per
slot — **~4 MB / ~40% of `latex.YYYY.dump.txt`**. The PERL_LOADFORMAT
audit had originally measured 3094 such records; the actual count had
grown ~30× by 2026-05-15 (one paragraph in the audit was stale).

**The fix (commit `81176ba689`, 2026-05-15).** `dump_writer` now
groups V entries by `(font, size)` prefix and emits a single `IA`
record per dense intarray: `IA\t<prefix>\t<len>\t<rle>` where
`<rle>` is a comma-list of `v` or `vxn` runs. `dump_reader` parses
`IA`, RLE-decodes, and emits the same per-slot V assignments at
indices 1..=len — runtime state post-replay is identical.
**Backward compatible**: dump_reader still loads existing
V-record-only dumps via the unchanged `V` arm. Non-dense intarrays
fall back to individual V records (the dump-build log warns).

**Measured TL2025 impact:** 89,294 V → 15 IA + 63 V fallbacks. Dump
size 7.4 MB → 3.7 MB (-49%). Entry count 110,691 → 21,475 (-81%).
`cargo test --tests`: 1196/0/0 → 1220/0/0 (after 25 new unit tests
covering RLE round-trip, IA load semantics, and V-record backward
compat).

**Perl's framing.** Perl LaTeXML's `latex_dump.pool.ltxml` uses
`Im(<cs>, FD(<real_cs>, 'fontinfo_cmr10 at 0.0003pt'))` + an
RLE-array Hash inside a `V('fontinfo_...', {'data'=>[(15)x32,...]})`
record. Same compactness, different syntax. Our `IA` schema is the
adaptation to our tab-separated text format.

**When the IA path doesn't apply.** Non-dense intarrays (indices not
1..N) skip the IA emit and fall back to individual V records. We saw
exactly one in TL2025 — `fontdimen_fontinfo_cmr10 at 14sp` with 9
sparse slots. If a future expl3 release adds more sparse intarrays,
the fallback handles it; the only cost is a few extra V records.

## #55 `OmniBus` is a LAST-RESORT fallback for *unknown* classes — never a dependency

**The principle (user directive 2026-05-28).** `OmniBus.cls` exists so
that a `\documentclass{<thing-we-have-no-binding-for>}` still produces
*something* — it bundles a broad, generic grab-bag (frontmatter macros
`\email`/`\affil`/`\address`/`\keywords`/`\shorttitle`/…, theorem +
natbib autoloads, a `\bibitem` override, `{frontmatter}`/`{mainmatter}`/
`{backmatter}` envs, AAS/elsevier-ish coverage). That grab-bag is the
right move when we know *nothing* about the class. It is the WRONG base
for a class binding we *do* have a `.rs` for: pulling in OmniBus means
the binding inherits ~600 lines of generic guesses it never asked for,
and — crucially — those guesses can actively break the document. A known
binding must `LoadClass!("article")` (the real base most journal classes
build on) and then load *exactly* its own specific needs.

**Why it actively breaks things (the witnessed failure).** OmniBus
eagerly pre-loads helpers (e.g. journal-class bindings layered
`RequirePackage!("amsthm")` on top of `LoadClass!("OmniBus")`). Eager
amsthm broke the ubiquitous `\let\proof\relax`\,+\,`\usepackage{amsthm}`
idiom: the paper's explicit `\usepackage{amsthm}` no-ops (already loaded),
so amsthm's `\let\proof\@proof` never re-runs after the paper cleared
`\proof` → `Error:undefined:{proof}` (witness 1707.03222 svproc,
1612.03054 imsart; both convert cleanly in Perl, which does NOT pre-load
amsthm). OmniBus *itself* already provides *lazy* amsthm autoload (the
theorem-env stubs at omnibus_cls.rs L399+), so the eager preload was both
redundant and harmful. The deeper lesson: every generic provision OmniBus
makes is a potential clash with what the real class/paper does.

**Decisive finding (2026-05-28 audit).** ALL 51 `_cls.rs` files that do
`LoadClass!("OmniBus")` are for classes Perl LaTeXML has **no binding
for** (`grep` of `LaTeXML/lib/.../Package/*.cls.ltxml` → zero matches).
Perl handles every one via its *automatic* fallback
(`Package.pm:LoadClass` L2700-2716): warn `missing_file` → load OmniBus →
`maybeRequireDependencies($class,'cls')` (dep-scan the raw `.cls` for
`\RequirePackage`/`\usepackage`, load each binding). Rust mirrors this
exactly in `binding/content.rs::load_class` (L1962-2067, incl.
`maybe_require_dependencies`). So **a hand-rolled `*_cls.rs` that just does
`LoadClass!("OmniBus")` is functionally what Rust does anyway if the file
didn't exist** — except registering the stub SKIPS the dep-scan of the
real `.cls` (the `<name>.cls.ltxml_loaded` flag short-circuits L2009),
usually a *regression* vs. letting the fallback run.

**User guidance (2026-05-28, refined — supersedes the "switch to article"
plan above).** Codifying "no binding → OmniBus stub" is a **shortcut**: OK
to lean on today, NOT acceptable long-term. Converting those stubs to
`LoadClass!("article")` + hand-derived specifics is *also* a shortcut
(still a hand-rolled binding for a class Perl has no binding for). The
**principled fix is to add NO new binding files and instead improve the
raw interpretation of reading the original `.sty`/`.cls`** so the automatic
OmniBus+dep-scan+raw-read fallback simply works. Therefore:
  * **Do NOT** build a `journal_support` mega-helper or otherwise invest
    in making the OmniBus-stub pattern "nicer" — that entrenches the
    shortcut. (The svproc→article+sv_support conversion `ce6ecb16c7` is
    fine to keep — sv_support is a *real* Perl support pkg — but it is NOT
    a template to replicate across the other 50.)
  * Existing OmniBus stubs are tolerated as-is short-term. De-risking
    them (e.g. dropping eager `RequirePackage!("amsthm")`, which breaks
    `\let\proof\relax`+`\usepackage{amsthm}`) is a fine bounded cleanup.
  * For a NEW class-related error: prefer avoiding a stub and fixing the
    raw `.cls`/`.sty` read path so the fallback covers it. Keep/extend a
    stub only when raw interpretation genuinely can't yet.
  * **Autoload-shadowing trap (strong reason to DELETE a stub).** OmniBus
    registers *lazy autoload triggers*: `\subjclass`/`\curraddr`→ams_support,
    `\citet`/`\citep`→natbib, `\begin{theorem}`→amsthm, `\mathfrak`/`\mathbb`
    →amsfonts, `\thechapter`→book (omnibus_cls.rs L542-587 + L404-444). A
    stub that hand-rolls one of these CSes (e.g. `\subjclass{}` as a
    frontmatter macro) **shadows the trigger**, so the autoload never fires
    and everything that package would have defined (e.g. `\bysame` from
    ams_support) stays undefined. Witnessed: birkjour/mcom-l stubs →
    `undefined:\bysame`. Deleting the stub restores the autoload chain and
    matches Perl. So: a one-error CONVERR on an ams/natbib/theorem macro
    under an OmniBus-loading stub is very often this — delete, don't patch.

Concrete wins applying this (2026-05-28): deleted `fundam_cls.rs`
(`{keywords}`), `mcom_l_cls.rs` (mcom-l/proc-l/tran-l, `\bysame` via
amsart dep-scan), `birkjour_cls.rs` (`\bysame` via `\subjclass`-autoload
un-shadowing). Each → 0 errors, matches Perl, removes a stub.

**Reference.** `latexml_package/src/package/omnibus_cls.rs` (the grab-bag),
`binding/content.rs::load_class` (the automatic fallback + dep-scan — the
*legitimate* OmniBus path). Companion: [[feedback_prefer_raw_load]],
[[feedback_perl_parity_bindings]], [[feedback_no_papering]].

---

## #54 TeXLive year detection uses `kpsewhich -var-value=SELFAUTOPARENT`, NOT `--version`

**The gotcha.** The naive way to detect the installed TeXLive year
is `kpsewhich --version`. **Don't.** That command returns the
`kpathsea` library version string ("kpathsea version 6.4.1, Copyright
2023…"), which is shipped IDENTICALLY across TL2023, TL2024, and
TL2025. Using it as a discriminator silently picks the wrong dump.

**The right way.** `kpsewhich -var-value=SELFAUTOPARENT` returns the
TeXLive install root, e.g. `/usr/local/texlive/2025`. The last path
segment is the year. Code:

```text
TL_YEAR="$(kpsewhich -var-value=SELFAUTOPARENT 2>/dev/null \
  | sed -n 's:.*/\([0-9]\{4\}\)$:\1:p')"
```

**Distro-package fallback.** Debian/Ubuntu's `texlive` package puts
TL into `/usr/share/texlive` (no year subdirectory), so
SELFAUTOPARENT returns `/` and the year-extracting `sed` matches
nothing. Fallback: `pdflatex --version` prints "(TeX Live YYYY)" in
its first three lines — parseable. Sibling commit `395615c0d4`
landed this two-step strategy in both `tools/make_formats.sh` (the
dump-build path) and `latexml_engine::dump_paths::detect_ambient_texlive_year`
(the runtime path).

**Why it matters.** The whole versioned-dump infrastructure
(commit `946ff9b7d0`, branch `distribution-include-bytes-bundling`)
selects which `resources/dumps/{plain,latex}.YYYY.dump.txt` to embed
at build time and which to prefer at runtime. If the year detection
is wrong, an embedded TL2025 dump might be replayed against a TL2023
binary or vice versa — silent semantic divergence in raw-loaded
package state. The bug class is exactly what the original audit
("Distribution follow-up") warned about: "different raw-load
semantics" across years.

**Reference.** `latexml_engine/src/dump_paths.rs::detect_ambient_texlive_year`,
`tools/make_formats.sh:60`, `resources/dumps/texlive.YYYY.version`
(the stamp file lets us record which TL produced each dump).

---

## 45. Namespaced attributes must promote their namespace to a *document* namespace

**Discovery:** The `--source-map` feature emits `data:sourcepos` (in LaTeXML's
`data:` namespace) on elements. It appeared in the core ltx XML but was silently
**dropped during post-processing** — 0 `data-sourcepos` in the HTML — while the
analogous `aria:labelledby` (acm_aria test) survived and converted fine.

**Analysis:** Two kinds of namespace exist in the model — *code* namespaces
(`RegisterNamespace`, used in binding code) and *document* namespaces (declared
as `xmlns:prefix` on the output root). `Document::finalize` →
`apply_document_namespace_declarations` declares `xmlns:prefix` on the root **only
for document namespaces that are actually used** (a literal `prefix:…` attribute
exists). The post XSLT's `copy_foreign_attributes` (`LaTeXML-common.xsl`) then
copies only attributes that are *in a namespace* (`namespace-uri() != ''`),
converting `data:`-prefixed ones to `data-…`. `aria` is a document namespace (it
appears in the RelaxNG schema, `common.attrs.aria`), so it gets declared on the
root and its literal attr resolves into the namespace on serialize. `data` was a
**code-only** namespace → never declared on the root → the literal `data:sourcepos`
stayed namespace-less (unprefixed attributes are namespace-less per XML rules) →
`copy_foreign_attributes` skipped it.

**Fix:** `Document::set_attribute`'s namespaced branch now mirrors Perl
`Core/Document.pm::setAttribute`, whose `getDocumentNamespacePrefix($ns, 1)`
**promotes** the prefix's namespace to a document namespace on first use:
`model::register_document_namespace(prefix, Some(ns_uri))` before the literal set.
Finalize then declares `xmlns:prefix` on the root and the attribute resolves +
converts. General over any prefix (implements the old `decodeQName` TODO);
idempotent for namespaces that are already document namespaces (`aria`, `xlink`),
so it is parity-neutral (verified on structure/complex/tikz).

**Key insight:** Setting `node.set_attribute("prefix:local", …)` (libxml
`xmlSetProp`) only *binds* the namespace if the prefix is already in scope.
For an attribute namespace to survive to output (and the post XSLT), its prefix
must be a **document** namespace so `apply_document_namespace_declarations`
declares it on the root. Promote on first use — do not rely on the prefix being
in scope at set time (finalize declares it, after construction).

## #56 Pregenerated bindgen bindings are platform-locked: `\u{1}` link_names pin ELF symbol spelling

**Symptom (macOS probe 2026-06-07):** the whole workspace compiles on
macos-15 arm64, then the final `latexml_oxide` link dies on exactly one
undefined symbol: `xsltMaxDepth` — *without* the Mach-O leading
underscore. The Homebrew dylib **does** export `_xsltMaxDepth`
(llvm-nm-verified on the arm64 bottle); the linker was simply told to
look for the wrong spelling.

**Mechanism:** crates that ship a bindings.rs pregenerated by bindgen
*on Linux* (rather than running bindgen in build.rs) carry
`#[link_name = "\u{1}xsltMaxDepth"]` on **statics** — the `\u{1}`
escape means "raw symbol, do not decorate", which hardcodes the ELF
name and bypasses Mach-O's `_` prefix. Functions get no `link_name`
(bindgen trusts the platform C ABI for them), so only *statics* break,
and only at final-binary link time, and only on non-ELF targets.

**Fixes:** (a) consumer-side — resolve the global at runtime with
`libc::dlsym(RTLD_DEFAULT, c"name")`, which applies the platform's own
decoration (this is what `latexml_post::xslt::set_xslt_max_depth` does
now; works identically on ELF and Mach-O); (b) upstream — drop the
`link_name` attribute from statics (plain `extern "C"` statics get
per-platform decoration), or generate bindings at build time.

**Audit state:** `libxslt` 0.1.3 has 12 such statics (we referenced
only `xsltMaxDepth`). `libxml` 0.3.12 has them only on glibc-internal
`__isoc99_*scanf` symbols in its *fallback* `default_bindings.rs`; its
build.rs regenerates real bindings per-platform, so it does not bite.
`kpathsea_sys` bindings: statics-free in the referenced surface.
When adding any new `-sys`-style dependency, grep its bindings for
`link_name = "\u{1}` + `static` before assuming portability.

## #57 Validate resolver changes by byte-comparing format dumps across backends — ls-R order cannot emulate kpathsea ranking

**Context (2026-06-07, release-dumps work):** kpathsea 0.3's
subprocess backend fronts `kpsewhich` with an `ls-R` basename cache.
Generating `latex.ltx` dumps with the linked vs subprocess backends on
identical code and diffing them exposed a silent resolution divergence
no test had caught: the subprocess dump was 756 entries smaller and
its text encoding was **IL2 (Czech)** — the cache had resolved
`fonttext.cfg` to `tex/cslatex/base/` instead of `tex/latex/base/`.

**The general lesson:** TL ships duplicate basenames whose winner is
decided by kpathsea's *path-spec ranking*, which raw `ls-R` order
cannot reproduce with ANY single-pass tie-break — first-wins picks
csLaTeX's `fonttext.cfg` (cslatex < latex alphabetically); Perl's
last-wins picks antomega's `hyphen.cfg` (lambda > generic). The
correct cache design **evicts ambiguous basenames** and lets them
fall through to a direct (memoized) `kpsewhich` call.

**The method:** a format dump is a deterministic, high-coverage
witness of every file resolution the kernel load makes — the embedded
`__file_seen_*` markers are a literal file-load ledger, and CS-name
diffs localize the divergence (font-shape names flagged the encoding
swap immediately). Byte-compare dumps across backends (expect identity
modulo the `texsys.aux_contents` timestamp record) before trusting any
file-resolution change. Upstream regression test:
rust-kpathsea `lsr_cache_agrees_with_cli_on_shadowed_basenames`.

## #58 macOS libmalloc exposes latent use-after-free that glibc hides — the `node.get_type().is_none()`-after-`add_child` trap

**Context (2026-06-08, issue #217 macOS port):** the full test suite
crashed nondeterministically *only* on macOS (worker threads), with a
node read as a garbage libxml2 type (`EntityDecl`/17,
`DOCBDocumentNode`/21 — types LaTeXML never builds) → `get_node_qname`
panic, plus SIGSEGV/SIGBUS. Linux was clean under **valgrind AND ASan**
(TL2025+TL2026, full-binary, `--test-threads=16`), and the bug was a
Heisenbug (symbol/`MallocScribble` builds masked it).

**Root cause:** a genuine use-after-free. In
`document.rs::open_text_internal`, after `point.add_child(&mut node)`
libxml2 **merges adjacent text nodes** — it appends the new text to
`point`'s existing last text child and **frees the just-created
`node`**. The merge was detected with `node.get_type().is_none()`,
which *reads the freed node*. That read is **benign on glibc** (the
freed slot keeps its old/None `type`, so the merge is detected) but
**unsound on macOS libmalloc**, which recycles/scribbles the freed slot
so `get_type()` returns garbage → the check fails → the freed node is
installed as `self.node`, corrupting the current insertion point (one
bad `set_node` cascaded to dozens of corrupt reads → crash).

**Two load-bearing lessons:**
1. **macOS's system allocator (libmalloc) surfaces latent UAFs that
   glibc's lazy bin-reuse silently tolerates** — and Linux valgrind/ASan
   miss them when the freed memory is never read on the Linux path (here
   the read *was* on the path, but glibc made it benign and valgrind
   only flags reads of memory it knows is freed-then-read with a *bad*
   outcome — the stale-but-valid read passed). When a bug is macOS-only
   and Linux-tooling-clean, suspect allocator-exposed UAF, not just TLS.
2. **Never detect a libxml2 text-merge by reading the merged node.**
   `add_child`/`add_next_sibling` of a text node can free it. Detect via
   **pointer identity** instead: after the add, the text is the parent's
   last child either way — if it *is* the original node it was appended
   (live), else it was merged+freed. `libxml::Node`'s `PartialEq`
   compares the stored `xmlNodePtr` *without dereferencing*, so
   `parent.get_last_child() == Some(&node)` is UAF-safe and
   allocator-independent. Audit any `X.add_*sibling/add_child(&mut t)`
   followed by a read of `t` for the same trap (fixed:
   `open_text_internal`, `swap_comment_text_if_needed`).

**Diagnostic technique that cracked it:** an lldb backtrace on the
brew-texlive CI leg + a `#[track_caller]` `set_node` tracer pinned every
corrupt assignment to one site (`open_text_internal`'s post-`add_child`
`set_node`). The `#[global_allocator]=mimalloc` is bin-only and never
touches libxml2's `xmlNode`s (no `xmlMemSetup`), so it is NOT in the
recipe — the system **libmalloc** is the exposer.
## #59 Rust XPath context evaluates from the root ELEMENT — Perl-relative document paths silently miss

Perl `Document::findnodes($xpath, $node)` defaults `$node` to
`$$self{document}` — the **document node** (parent of the root
element). A Perl binding xpath like
`'ltx:document/ltx:resource[last()]'` therefore matches the root
`<ltx:document>` and steps into its children.

The Rust `Document::findnode/findnodes(xpath, None)` path ends at the
cached libxml `Context`, whose default evaluation node is effectively
the **root element** — so the same relative path looks for an
`ltx:document` *child of* `<ltx:document>` and returns nothing, with
no error. The miss is silent: code that falls back (e.g. "append at
end of root") produces structurally-wrong-but-valid XML.

**Rule:** when porting a Perl binding xpath that starts with a
relative step naming the root element (`ltx:document/...`), translate
it to the absolute form (`/ltx:document/...`). Paths starting `.//`
or `//` are unaffected.

**Witness:** PR-2767 port, `\lx@frontmatter@fallback` — frontmatter
(title/creator) landed at the *end* of `<ltx:document>` instead of
after the `ltx:resource` block; caught by `20_digestion::rebox_test`.
Fixed in `base_utilities.rs` by using
`/ltx:document/ltx:resource[last()]`.

## #60 libxml string accessors silently fail on namespaced `xml:id`/`xml:lang` — and a *masked* broken accessor is not automatically a bug

`xml:id` is stored by libxml2 NAMESPACED — local name `"id"` in the XML
namespace (`http://www.w3.org/XML/1998/namespace`), NOT a literal attribute
named `"xml:id"`. rust-libxml's string-keyed API matches the *literal* local
name, so the whole `*_attribute("xml:id")` family silently misfires:
`get_attribute("xml:id")` → always `None`, `has_attribute("xml:id")` → always
`false`, `remove_attribute("xml:id")` → silent no-op. **Writes and
serialization are fine** (`set_attribute("xml:id", …)` namespaces correctly);
only string-keyed reads/checks/removes break. Correct form:
`get_attribute_ns("id", XML_NS)` / `has_attribute_ns` / `remove_attribute_ns`
(`XML_NS = latexml_core::common::xml::XML_NS`, in the engine prelude). The same
footgun hits `xml:lang` (local `"lang"`) — and no other prefixed attribute in
the workspace.

**The non-obvious half:** most of the ~53 broken sites are *masked* — paired
with a working `_ns` call, guarded by another always-false check that never
lets the dead block run, or carrying an `.or_else(get_property("id"))`
fallback. **Do NOT blanket-"correct" them.** At least one mask is load-bearing:
`rewrite.rs:1242` (XMArg→inner-id transfer) is a no-op, and swapping in the
`_ns` accessor makes wildcard `1`/`n` tokens acquire `xml:id`s **Perl does not
emit**, regressing `simplemath`/`declare`. Only migrate a site when a
*confirmed* Perl divergence is traced to it. New code uses the ns-aware form
from day one; `tools/lint_xmlid_accessor.sh` (+ `xmlid_lint_baseline.txt`,
wired into pre-push + CI) ratchets against NEW string-keyed `xml:` accessors.

**Witnesses (all fixed 2026-06-08):** `rename_node_internal` dropped `xml:id`
across the equation→equationgroup rename (2311.01600 dangling `\Pr` refs);
`rearrange_lone_ams_aligned` read empty `eq_id`; `get_node_language` read
`xml:lang` as `None` → non-English math used `.`/`,` English conventions. Full
analysis: `archive/XMLID_ACCESSOR_AUDIT_2026-06-08.md`.

---

## 46. "Can not mutably reference a shared Node" is a false-positive guard, not a real aliasing check

**Discovery:** The cortex 10k cross-join flagged a 16-paper cluster
(`document/convert`, e.g. `0805.2376` dcpic commutative diagrams, `1407.0452`
emulateapj deluxetable) erroring `Can not mutably reference a shared Node` —
papers Perl converts cleanly. Re-running the **current** binary on `1407.0452`
(0 errors) and `0805.2376` (0 shared-Node errors; its 32 errors are
`\begindc`/`\obj` = host lacks the dcpic package, shared with Perl) confirms the
cluster is **already gone** — the cortex run used a pre-fix binary. The live fix
is `Document::new` raising `NODE_RC_MAX_GUARD` 2 → 8192 (`document.rs:~137`).

**Analysis — why the guard is the wrong invariant.** `libxml::Node` is
`Rc<RefCell<_Node>>` wrapping a raw `xmlNodePtr`. `node_ptr_mut`
(`libxml-0.3.13/src/tree/node.rs:180`, reached by every `&mut self` mutator —
`set_attribute`×135, `add_child`×47, `set_content`, `unlink_node`, …) gates
mutation on `weak_count == 0 && strong_count <= NODE_RC_MAX_GUARD`. But
`strong_count` counts **live `Node` clones**, which is NOT an active aliasing
conflict:
- libxml's own `document.nodes` cache holds **one persistent clone per node**
  (`_wrap` returns the cached clone), so every node already sits at strong_count
  ≥ 1 before anything else.
- latexml_core bookkeeping adds more **legitimate** persistent/transient clones:
  `idstore: HashMap<String, Node>` (one per `xml:id`'d node), `pending: Vec<Node>`,
  `constructed_nodes` / `localized_constructed_nodes: Vec<Vec<Node>>`. So any
  id'd node being mutated is already at ≥ 3 (cache + idstore + self), tripping the
  default guard of 2; deep legitimate sharing (dcpic arrow grids, XMDual content
  reuse) holds **thousands** of simultaneous clones during absorb.

None of that is a borrow conflict. The **real** safety mechanism is
`RefCell::borrow_mut()` on line 190. For a node **linked** in the tree, all
handles to it resolve to ONE shared `RefCell` (the per-document `nodes` cache,
keyed by `xmlNodePtr`, hands back a clone of the existing wrapper — see
`_wrap`/`ptr_as_option`), so `try_borrow_mut` serializes access and detects a
genuine active aliased borrow. The `&mut self` receiver is a second layer
(compiler-enforced exclusive access to the handle). `strong_count <= GUARD` is a
redundant THIRD layer that is simultaneously **over-strict** (false-positives on
benign clones — what bit the 16 papers) and **under-protective** (it never
actually prevents the real hazard: once you extract the `*mut xmlNode`, raw C
calls mutate sibling/parent nodes outside any RefCell). Raising it to 8192
doesn't fix the theory — it just moves the false-positive threshold higher.

**Bound on the shared-`RefCell` guarantee (don't overclaim):** the identity
cache is NOT total — `set_unlinked` (on `unlink_node`) and `import_node` call
`forget_node`, evicting the pointer (deliberate: a freed C node's address can be
reused, so a stale wrapper would mis-identify it). After eviction, re-wrapping
the same pointer mints an INDEPENDENT `RefCell`, so two such handles to an
unlinked node are not mutually exclusive. The old `strong_count` heuristic was
equally blind to this (two independent `Rc`s, each low-count), so `try_borrow_mut`
neither introduces nor worsens it — it's the same inherent C-wrapping footgun.
The fix lives in the dginev `libxml` fork (0.3.14): `node_ptr_mut` now uses
`try_borrow_mut`; `NODE_RC_MAX_GUARD`/`set_node_rc_guard` became deprecated
no-ops. After latexml-oxide bumps to 0.3.14, drop the `set_node_rc_guard(8192)`
call in `Document::new`.

**Why no conflict actually occurs:** document construction is single-threaded
(State is thread-local, one Document per conversion) and the builder mutates one
node at a time without re-entering a live borrow. The only place a real
re-entrant mutable borrow can arise is the Rhai constructor trampoline (#248 / SYNC_STATUS §3) — and there, failing LOUDLY is correct.

**Recommended structural fix (in the dginev `libxml` fork — published 0.3.13, not
checked out locally, no `[patch]`):** replace the `strong_count` heuristic in
`node_ptr_mut` with the real invariant —
```rust
match self.0.try_borrow_mut() {
  Ok(b) => Ok(b.node_ptr),           // no active aliased borrow ⇒ safe
  Err(_) => Err("… node is actively borrowed …".into()),
}
```
This is strictly sounder (catches the genuine re-entrancy, ignores benign clone
count), eliminates the false-positive class entirely, and makes
`NODE_RC_MAX_GUARD` / the 8192 band-aid dead code (remove after the fork bump).
A purely in-repo mitigation (key `idstore` by `xmlNodePtr`/`usize` and re-wrap on
lookup, so it stops holding a persistent clone) shaves a few counts but CANNOT
remove the issue "in theory" — the cache-clone + deep-sharing reality always
exceeds a small guard. **Sequencing:** keep 8192 load-bearing until the fork fix
lands, then delete the guard plumbing. Do NOT lower the guard while the heuristic
exists (regresses the 16 papers).

## 41. Frontmatter fallback DOM surgery: three construction-time traps

Context: `base_utilities.rs` `\lx@frontmatter@fallback` + `maybe_promote_leading_title`
(the beyond-Perl "keep abstract below a hand-formatted title block" ordering fix
and the "promote a leading centered display block to `<ltx:title>`" heuristic for
`\title`-less papers, e.g. arXiv 1609.07638). Three traps bite any code that
manipulates the live document DOM *during construction* (inside a `DefConstructor`
sub), not at serialize/finalize time:

1. **A RELATIVE-context `findnode`/`findnodes` returns nodes detached for child
   traversal.** `document.findnode(".//ltx:p", Some(&ctx))` yields a node whose
   `get_content()` works but whose `get_child_nodes()` is **empty** and on which a
   further relative XPath finds nothing — a rust-libxml shared-node artifact. An
   ABSOLUTE query (`/ltx:document/...`, `None` context) returns a live node that
   traverses correctly. Rule: fetch ONE anchor with an absolute query, then walk
   the DOM by hand (`get_child_nodes()` recursion) for everything downstream.

2. **The human-readable `fontsize`/`font` attributes do not exist yet at
   construction time.** The `<ltx:text>` carries `_font` (an interned Font id) +
   `_fontswitch="true"`; `fontsize="144%"` etc. are derived from `_font` in a
   later finalize pass (`Font::relative_to`). To test "larger than body" at
   construction, decode `_font` via `document.decode_font(&id)` → `Font::get_size`
   and compare to `NOMINAL_FONT_SIZE` (mirroring `font::defsize`): `size >
   nominal*1.1` is the analogue of `fontsize > 110%`.

3. **Creating a default-namespace LTX element.** `insert_element_before(pt,
   "ltx:title", None)` emits a stray prefixed `<ltx:title>`. Mirror
   `open_element_internal`'s default path: create with a BARE tag (`"title"`) then
   `set_namespace(root.get_namespace())` so it serializes as `<title>` in the
   document's default namespace. Move children with a true move
   (`child.unbind(); parent.add_child(&mut child)`) — preserves xml:ids, unlike
   `append_clone` which clones + remaps.

## 47. Box-sizing estimation: the `\par` repack seam, list padding, and the foreignObject em basis

*(2026-07-03, from the arXiv 2605.02240 tcolorbox arc — frames drawn from our
measured content `\vbox` were both grossly too tall and clipping their content.)*

tcolorbox (raw-loaded real `.sty`) draws its pgf frame from the dimensions WE
measure for the content `\vbox`, so every estimator gap becomes a visible
frame/content mismatch. Three traps, all in the sizing pipeline:

1. **A `\par` digested in an isolated box list repacks NOTHING and defuses
   later repacks.** `stomach::digest` isolates the box list
   (`new_local_box_list`), so an extra `Digest!("\par")` in a
   `before_digest_end` hook sees an empty list AND resets MODE — the real
   repack seam (`repack_horizontal`, fired by `\par` before_digest or
   `leave_horizontal_internal`) then never collects the trailing horizontal
   boxes into a width-carrying `List`. Result: paragraph text is measured as
   ONE long line (952pt tall boxes from `\hsize`-relative nonsense). Perl has
   no such hooks on {itemize}/{enumerate}/{description} — they were a
   Rust-only addition, removed in e0ec51fe87.

2. **Sizing properties ride whatsit properties, and lists carry real glue.**
   `compute_size_and_cache` (lib.rs BoxOps) adds
   `padtop`/`padbottom`/`padleft`/`padright` from the whatsit's properties
   after computing content size. Perl's `beginItemize` returns
   `padtop = padbottom = \topsep + \parskip + \partopsep` — and the five glue
   registers (`\topsep` 8pt, `\partopsep` 2pt, `\itemsep`/`\parsep`/
   `\lx@default@itemsep` 4pt) have REAL defaults in the pool. Zeroed registers
   or a missing pad ⇒ every list under-measures by ~2×`\topsep`+glue, and
   `\preitem@par` must be the CURRENT upstream DefMacro (real `\par` +
   `\vskip\itemsep\vskip\parsep` between items) or each item measures as a
   single unbroken line. Probe parity is byte-exact when right:
   `\setbox0=\vbox{...}\typeout{\the\ht0 \the\dp0}` matches reference Perl to
   the sp.

3. **foreignObject `--ltx-fo-*` em variables need the `font-size:<N>pt` term
   in the SAME style attribute** (Perl TeX_Box.pool L427-430). Without it the
   browser resolves the em vars against inherited 16px instead of the TeX em,
   inflating the CSS container ~20% past the drawn frame (content runs through
   the border). The size must come from the whatsit's live font — the same
   source as the em divisor — so `\small` contexts emit 8pt etc.

Debugging recipe: bisect with `\setbox0=\vbox` probes against reference Perl
(`perl -I LaTeXML/lib LaTeXML/bin/latexml`, `--debug=size-detailed`) AND
pdflatex ground truth; both engines deliberately over-estimate, so chase
*divergence from Perl*, not from TeX.
