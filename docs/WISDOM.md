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

## 13. DefKeyVal machinery: default resolution and setKeysExpansion guard

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

## 13. Star (`*`) in CS names causes infinite compile loop

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

**Workaround:** For `Let!`, use the `T_CS!()` wrapper. For `DefMacro!`, use
`\csname...\endcsname` form or runtime `def_macro()` calls:

```rust
DefMacro!("\\csname IEEEeqnarray*\\endcsname{}", "\\csname eqnarray*\\endcsname");
Let!("\\csname endIEEEeqnarray*\\endcsname", "\\csname endeqnarray*\\endcsname");
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
