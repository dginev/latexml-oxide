# Tactical Wisdom: Internal System Insights

Specialized analyses that led to correct patches. These are tactical insights
about the internals of latexml-oxide — not general skills, but specific
knowledge about how the system works that can prevent future mistakes.

---

## 1. DefMacro! double-packing: compile-time vs runtime pack_parameters

A `DefMacro!` whose expansion is pre-compiled via `compile_expansion!`
(`tokenizeable.rs:31`) packs `##`→`#`(PARAM) / `#1`→ARG at build time; at runtime
`def_macro()` → `Expandable::new()` (`expandable.rs:225`) packs it **again**
unless `nopack_parameters: true`, re-reading the packed `#\hfil` as an
alignment-cell marker → `Error:misdefined:expansion` (witness `\displaylines`).
**Fix:** every `DefMacro!` branch using `compile_expansion!` sets
`nopack_parameters: true`. (`DefConstructor!`/`compile_tokenize!` and
`DefPrimitive!`/closures don't pack — this is `DefMacro!`-specific; check it for
any new compile-time expansion path.)

---

## 2. Font::merge() must NOT call specialize()

`specialize(text)` classifies Unicode properties of *rendered text*. A Rust
`merge()` wrongly called `specialize(font_name)` on the filename "cmb10" — its
digits hit the "Other Symbol" branch, silently resetting `series` bold→medium
(`\font\mybf=cmb10 \mybf Hello` lost its bold). Perl's `merge()` takes
`specialize` only as an explicit optional arg, never by default. **Fix:** remove
`specialize()` from `merge()`; call it only at `TBox::new()` (`tbox.rs:131`) with
real text. Never pass it a font/file/CS name.

---

## 3. Catcode::CS vs Catcode::ESCAPE distinction

`ESCAPE` (catcode 0) is the backslash INPUT character; `CS` is the catcode of a
formed control-sequence token (`\foo` tokenizes to ONE `CS` token, not `ESCAPE` +
letters). **Test CS/active tokens with `cc.is_active_or_cs()`, never
`cc == Catcode::ESCAPE`.**

---

## 4. RegisterType::PartialEq trap: Number == CharDef

`RegisterType`'s custom `PartialEq` treats `CharDef` as equal to `Number` (char
defs are numerically valued), so `register != RegisterType::Number` does NOT
exclude `CharDef`. **Use `matches!(register, RegisterType::Number)`, not `==`/`!=`.**

---

## 5. at_letter catcode restore: None vs Some(OTHER)

`at_letter()` saves `@`'s old catcode via `lookup_catcode('@')`, which returns
`None` when `@` uses its default catcode OTHER (absent from the table). Restoring
with `assign_catcode('@', None)` is a no-op, so `\makeatother` left `@` a LETTER
forever. **Fix:** `unwrap_or(Catcode::OTHER)` on restore. General lesson: a state
lookup returning `None` often means "default value," not "no value" — decide what
`None` means in context.

---

## 6. Sizer string parsing: `#property_name` vs `#digit`

Perl `Whatsit::computeSize` (Whatsit.pm L257-260) parses a sizer string as
`(#\w+)*` — each `#token` is an ARG lookup if numeric, else a PROPERTY lookup.
Rust's `IntoOption<Option<SizingClosure>> for &str` (traits.rs) only handled
`#digit`, so `sizer => "#alignment"` failed `"alignment".parse::<usize>()` and
silently `unwrap_or(1)`'d — measuring arg 1 (the optional `[]`) instead of the
alignment property, returning (0,0,0) → `normalize_prune_columns` dropped the
column (nested tabtab_test lost its 3rd column). **Fix:** parse each `#word` as
numeric (arg) or alphabetic (property); supports compound `#1#2`. Lesson: verify
the `default` in any `parse::<usize>().unwrap_or(default)` over user strings —
silent fallbacks mask bugs for months.

---

## 7. `align_group_count` (`$ALIGN_STATE`): scan-level only, retract on unread

`$ALIGN_STATE` (TeX §309) must be incremented/decremented exactly once per `{`/`}`
as it passes `readToken`/`readXToken`, and retracted when tokens are unread — the
stomach's group machinery is a separate concern. Nested `\vbox{\halign{...}}`
inside an outer `\halign` mis-fired `handle_template` (agc >0 when it should be 0)
from two bugs: (1) `unread_one` didn't adjust agc for BEGIN/END (Perl `unread`,
Gullet.pm L340-359, always does), so `skip_spaces` reading `{` then unreading it
via `unread_one` double-incremented on re-read; (2) `stomach::bgroup()/egroup()`
adjusted agc, but Perl's (Stomach.pm L327-342) do NOT. **Fix:** add agc adjustment
to `unread_one` for BEGIN/END; remove it from `bgroup()`/`egroup()`.

---

## 8. Rust macros cannot dispatch on type — Vec<Token> vs Token vs &[Token]

`macro_rules!` expands on syntax before types are known, so unlike Perl's runtime
`Tokens()`, one `Tokens!()` invocation can't conditionally handle `Token` /
`Vec<Token>` / `Tokens` / `&[Token]`. When building token sequences from mixed
static + dynamic (`.revert()`) sources, fall back to imperative `Vec<Token>` +
`extend()`/`push()`, then `Tokens::new(toks)` — don't fight the macro:
```rust
let mut toks: Vec<Token> = vec![T_CS!("\\hbox"), T_BEGIN!()];
toks.extend(content.revert());
toks.push(T_END!());
stomach::digest(Tokens::new(toks))?;
```

---

## 9. arena::with pattern for zero-allocation string access

The interner stores strings in a thread-local arena. `arena::to_string(sym)`
heap-allocates a new `String`; `arena::with(sym, |s| …)` (and `with2` for two
symbols) borrows the `&str` for the closure's duration, zero-alloc. Prefer `with`
whenever the use is short-lived (comparisons, `set_property`, formatting) — in
per-token/column/row hot paths the allocations add up. Use `to_string` only for an
owned `String` that outlives the scope (e.g. a `HashMap<String,…>` key).

---

## 10. Porting RawTeX() blocks: copy bravely and exactly

Port Perl `RawTeX()` as Rust `RawTeX!()` with the identical TeX string — even
large blocks. The code is already debugged in Perl and the macro feeds it through
the same tokenizer/expander, so fidelity is free; "Rustifying" or selectively
porting pieces introduces subtle divergences that surface as unrelated test
failures later. The cost of porting is copy-paste; the cost of NOT is missing
definitions.

---

## 11. Parameter prototype conventions: `{}` vs named parameter types

In a LaTeXML prototype string, `{}` means "read a Plain balanced group"; a named
type (`Token`, `Number`, `Variable`) is its bare name, NOT braced. So
`DefMacro!("\\foo Token", …)` reads a Token param, while `"\\foo {Token}"` reads a
group and treats "Token" as literal body. `def_parser.rs` distinguishes `{}`
(Plain reader), `[]` (Optional), `Token` (named type from PARAMETER_TYPES),
`[Number]` (Optional with inner Number reparse). Perl `DefMacro('\foo{}',…)` ≡
Rust `"\\foo {}"`. If an arg reads wrong, check `{}`-vs-named-type in the prototype.

---

## 12. normalize_sum_sizes: per-column-index arrays, not flat lists

Perl `normalize_sum_sizes` (Alignment.pm L500-664) uses per-column-index arrays
(`$colwidths[$j]` collects column j across all rows, then `max`). Rust used a flat
one-entry-per-cell list, which broke on ragged column counts and colspan>1. When
porting Perl array-accumulation, verify the indexing: `$foo[$j]` in a nested loop
is per-index accumulation (`Vec<Vec>::push_at_index`), NOT flat `Vec::push`. The
Rust rewrite of normalize.rs also had to add the missing Perl semantics:
**vattach** height/depth split (top=all depth, bottom=all height, middle=split at
math axis; Rust had `cached_depth=0`); **lspaces/rspaces** → cell width +
`lpadding`/`rpadding`; **border padding** `0.4*UNITY` per border; **first/last-row
strut** only for non-LaTeX alignments; **colspan>1** distributes excess width
across spanned columns.

---

## 13. close_node_with_strictness: walker tracks walker node, not target

`close_node_with_strictness` (document.rs) walks up from `self.node` toward
`node`, auto-closing intermediates — the loop condition must test the WALKER `n`,
not the TARGET `node` (invariant across iterations). Perl (Document.pm L952-970):
`while (($t = $n->nodeType) != XML_DOCUMENT_NODE && !$n->isSameNode($node)) {… $n
= $n->parentNode}`. The bug used `node.get_type()` in both init and loop body
instead of `n.get_type()` — a one-char (`node` vs `n`) difference causing wrong
termination. **Fix:** both → `n.get_type()`. When porting Perl loops with `$node`
(target) + `$n` (walker), watch which variable each condition uses.

---

## 14. close_to_node ifopen parameter: suppress error when true

`close_to_node` (document.rs) declared `_ifopen` (unused). Perl (Document.pm
L910-925) uses `$ifopen` to suppress the "not open" error when closing a node
absent from the open path: `if (!$ifopen) { Error('malformed', …, "…isn't open")
}`. Without it, every "close if open" call to a non-open node emits a spurious
error. **Fix:** rename `_ifopen`→`ifopen`, guard the error with `if !ifopen`.
Lesson: an `_`-prefixed param may be *accidentally* unimplemented, not
intentionally unused — the `_` masks missing functionality.

---

## 15. DefKeyVal machinery: default resolution and setKeysExpansion guard

Bare keys (`sensitive,`) weren't getting defaults despite a correct
`DefKeyVal!("LST","sensitive","","true")`. Default resolution happens during
KeyVals parsing (`add_value` with `use_default=true`) via key
`KEYVAL@default@KV@LST@sensitive` (correct); a second `lstActivate` path used the
WRONG key `KEYVAL@LST@sensitive@default` (dead code, removed — never matched in
Perl either). **Actual root cause:** `\@lstdefinelanguage` ignored the base-language
params — Perl's `$keyvals->setValue('language', Tokens(@base))` triggers recursive
language-chain activation (`[LaTeX]{TeX}→[common]{TeX}→[primitive]{TeX}`); without
it `sensitive,` from `[primitive]{TeX}` never reaches the context. (Related Perl
quirk: `lstClearLanguage` clears class `'textcs'` but texcs words use `'texcss'` —
a typo that lets them survive the clear.) **setKeysExpansion guard:** Rust adds
`state::has_meaning(...)` before emitting `\qname@default`; Perl emits it
unconditionally → undefined-CS errors for bare keys without defaults (e.g.
`a4paper` via `DeclareOptionX`). Rust falls back to `\qname{}`.

---

## 16. Star (`*`) in CS names causes infinite compile loop

**Date:** 2026-03-15

`DefMacro!`/`Let!` proc macros infinite-loop (OOM 14GB+) when the CS-name string
contains `*` or `{}` — the compile-time tokenizer (`latexml_codegen`) treats them
as special/param patterns. Both are valid in TeX CS names (`\eqnarray*`,
`\begin{foo}`). **Workaround:** wrap the CS in `T_CS!()`, bypassing string
tokenization:

```rust
DefMacro!(T_CS!("\\IEEEeqnarray*"), "{}", T_CS!("\\eqnarray*"));
Let!(T_CS!("\\endIEEEeqnarray*"), T_CS!("\\endeqnarray*"));
```

(Refactor TODO: fix the internal tokenizer to handle `*` in CS names without
looping, so the raw string first-arg works.)

---

## 17. Sizer inference from reversion: silent incorrect sizing

All math boxes (`\hbox{$...$}`) returned an identical size regardless of content:
`dialect.rs::infer_sizer()` inferred a sizer from a Constructor's reversion tokens
when none was set. For body-capturing constructors (`\lx@begin@inline@math`) the
reversion is `$`, so it measured the literal `"$"` glyph, not the math body.
**Fix:** `infer_sizer()` returns `None` when no explicit sizer is set (Perl never
infers from reversion) — `compute_size()` then uses the "body" property. Perl's
`Whatsit::computeSize` consults reversion only as a last resort (body → sum args →
reversion); Rust was short-circuiting that cascade.

---

## 18. METRIC_MAP vs STDMETRICS key mismatch: math fonts fall back to cmr

Math chars used cmr (serif) instead of cmmi metrics (no italic correction):
`METRIC_MAP` mapped `"math_medium_italic"`→`"cmmi"` but `STDMETRICS` keys cmmi10
data as `"cmm"`, so `get_metric_for_name("cmmi")` missed and fell back to cmr.
**Fix:** `METRIC_MAP` value `"cmmi"`→`"cmm"`. STDMETRICS keys drop the trailing
'i' (cmm, not cmmi) while METRIC_MAP used the TFM-filename convention — ensure
METRIC_MAP values match STDMETRICS keys.

---

## 19. enterHorizontal uses inplace assignment, NOT beginMode

`\vbox{hop}` should give width=\hsize (469.75pt) but Rust gave natural char width
(5.55pt). Perl's `enterHorizontal` (Stomach.pm L418) does `assignValue(MODE =>
'horizontal', 'inplace')` — NOT `beginMode` — so BOUND_MODE stays
'internal_vertical' in the SAME frame. `endMode('internal_vertical')` →
`leaveHorizontal_internal` then passes `MODE eq 'horizontal' AND BOUND_MODE =~
/vertical$/`, firing `repackHorizontal`, which groups char boxes into
`List(mode=>'horizontal')`; `List()` (List.pm L53-54) sets `width=\hsize`. The
distinction is critical: `assignValue(MODE,'inplace')` modifies the SAME frame's
BOUND_MODE; `beginMode` pushes a NEW frame (`pushStackFrame` + `MODE,'local'`)
hiding the parent's. **Fix:** `predigest_box_contents` matches Perl's frame scope
and calls `repack_horizontal_in_list` when MODE went horizontal inplace (guarded
to simple TBoxes, not Whatsits like tabular).

## 20. Whatsit::get_arg() is 1-based: get_arg(0) always returns None

`Whatsit::get_arg(n)` (whatsit.rs L108-116) is 1-based (matches Perl
`$whatsit->getArg(1)`): `n==0` returns None, else `self.args.get(n-1)`.
0-based-assuming code silently gets None for the first arg → `unwrap_or(0.0)`
(e.g. `\turnbox{90}{hello}` always angle=0). The pattern
`get_arg(0).map(…).unwrap_or(default)` always uses the default. **Fix:**
`\turnbox`/`{turn}`/`{rotate}`/`\lx@diagheads` shifted to 1-based. (OptionalKeyVals
params NOT provided don't occupy an arg slot — `novalue=true` — so don't shift
indices.) Grep `get_arg(0)` to catch these.

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

`DefConstructor` bodies run at CONSTRUCTION time, but constructors are digested in
sequence before ANY is constructed — so a register read in the body sees the LAST
digested constructor's value, not this one's (xy-pic SVG constructors read
`\X@c`/`\Y@c` at construction → all-zero coordinates). **Fix:** capture at digest
time via `properties => sub[args] {…}` (returns a `SymHashMap<Stored>`), read in
the body from `props.get("key")`:

```rust
DefConstructor!("\\foo", sub[document, _args, props] {
    let val = props.get("bar_val");                          // read at construction
}, properties => {
    let val = state::lookup_register("\\bar", Vec::new())?;  // captured at digest
    stored_map!("bar_val" => format!("{}", val))
});
```

Applied to all 19 xy SVG constructors + `\pic@makebox@` (no other critical sites).

---

## 24. catcode checks vs defined_as: Perl is inconsistent

Replacing `get_catcode() == Catcode::BEGIN` with `defined_as(T_BEGIN!())`
regressed because Perl uses DIFFERENT checks per function — raw catcode
(`$$token[1] == CC_BEGIN`) vs meaning (`defined_as`, via `\let`-chain resolution) —
and mixes them:

| Function | Perl check | Matches `\bgroup`? |
|----------|-----------|-------------------|
| readArg | CC_BEGIN catcode | No |
| readBoxContents | defined_as(T_BEGIN) | Yes |
| readBalanced (require_open) | CC_BEGIN \|\| Equals(meaning, T_BEGIN) | Yes (dual) |
| readDelimited / readTokensValue / readUntilBrace | CC_BEGIN catcode | No |

**Match each Perl function's exact check** — `defined_as` is not universally correct.

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
- `dump_loader.rs` (was: false → true) — file since split into
  `latex_dump.rs` / `plain_dump.rs`; kept as the historical call-site list
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

## 38. `\vspace` is the faithful DefMacro; a "doesn't fire" mode-gated primitive means suspect the UPSTREAM mode-setter

**RESOLVED (2026-08-05).** `\vspace` is the faithful
`DefMacro!('\\vspace OptionalMatch:* {}', '\\vskip #2\\relax')`
(`latex_constructs.rs:9241`), matching Perl `latex_constructs.pool.ltxml:4692`.
An earlier port kept it a no-op `DefPrimitive` stub, fearing a `moderncv/cs_cv.tex`
paragraph-break regression from `\vskip` auto-`\par` in horizontal mode; that
diagnosis was WRONG and the feared landmine never fired (the fix doesn't touch
`\vskip`, so `82_moderncv::cs_cv_test` stays green).

**The real bug (witness arXiv:2302.11635, IEEEtran `figure*` with
`\hrulefill\vspace*{4pt}` between minipage rows):** the captioned minipages after
the `\vspace*` came out inside the leader's `<ltx:p>` as schema-invalid
`<caption>`-in-`<block>` (4 `malformed:` errors) where Perl makes them separate
`<ltx:figure>`s. Root cause: Rust was `internal_vertical`, not `horizontal`, at
the `\vskip`, so `leaveHorizontal` *correctly* declined to fire (`hmode+vskip:
head_for_vmode`, gated on horizontal mode, tex.web L21160). The defect was that
LaTeXML's `\hrulefill` dropped the kernel's leading `\leavevmode` (latex.ltx
L643); `\hrule` is vertical-mode, so nothing entered horizontal mode. Perl
survived because `\hfill`'s `enterHorizontal` (`inplace`) persists past `\leaders`
(a `bounded` constructor); Rust's `bounded` reverts it. **Fix:** restore the
kernel definition — `\hrulefill` → `\leavevmode\leaders\hrule\hfill\kern\z@` (and
`\dotfill` likewise), `plain_constructs.rs`,
[OXIDIZED_DESIGN #97](OXIDIZED_DESIGN_DIVERGENCES.md). 2302.11635: 4 errors → 0,
10 `<figure>`/0 `<block>` (Perl-identical). Guard:
`50_structure::vspace_closes_leader_para_test`.

**Method takeaway:** when a mode-gated primitive "doesn't fire," suspect the
UPSTREAM mode-setter, not the primitive — and check the **real LaTeX kernel
definition** (latex.ltx); LaTeXML's `.pool` macros sometimes drop
`\leavevmode`/`\kern\z@`, and the port should follow the kernel.

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
**Corrected 2026-07-20: only TWO branches still need porting** (single-X-token
read, BEGIN-unwrap). The third — `.`/undef → `\lx@delimiterdot` — **is already
implemented** in `base_parameter_types.rs`'s `DefParameterType!(TeXDelimiter)`
(the `None` / `END` / `"."` match arms), together with an END-peek fallback Perl
does not have (witness arXiv:1207.4709). The in-code comment said "3 branches
missing" too and has been corrected alongside this line.

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
after the `ltx:resource` block; caught by `the `20_digestion` `rebox` fixture (`tests/digestion/rebox.{tex,xml}`)`.
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
`rewrite.rs:1034/:1043` (XMArg→inner-id transfer) is a no-op, and swapping in the
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
calls mutate sibling/parent nodes outside any RefCell). Raising it to 8192 only
moved the false-positive threshold higher — the real fix (below) replaced the
heuristic outright.

**Bound on the shared-`RefCell` guarantee (don't overclaim):** the identity
cache is NOT total — `set_unlinked` (on `unlink_node`) and `import_node` call
`forget_node`, evicting the pointer (deliberate: a freed C node's address can be
reused, so a stale wrapper would mis-identify it). After eviction, re-wrapping
the same pointer mints an INDEPENDENT `RefCell`, so two such handles to an
unlinked node are not mutually exclusive. The old `strong_count` heuristic was
equally blind to this (two independent `Rc`s, each low-count), so `try_borrow_mut`
neither introduces nor worsens it — it's the same inherent C-wrapping footgun.
**Why no conflict actually occurs:** document construction is single-threaded
(State is thread-local, one Document per conversion) and the builder mutates one
node at a time without re-entering a live borrow. The only place a real
re-entrant mutable borrow can arise is the Rhai constructor trampoline (#248 /
SYNC_STATUS §3) — and there, failing LOUDLY is correct.

**RESOLVED (dginev `libxml` fork 0.3.14, landed).** `node_ptr_mut` now uses
`try_borrow_mut` (catches genuine re-entrancy, ignores benign clone count);
`NODE_RC_MAX_GUARD`/`set_node_rc_guard` are deprecated no-ops and the
`set_node_rc_guard(8192)` call in `Document::new` is gone
(`latexml_core/src/document.rs:258`). An in-repo-only mitigation (key `idstore`
by `xmlNodePtr` and re-wrap on lookup) could shave counts but never removes the
issue — the cache-clone + deep-sharing reality always exceeds a small guard, so
the fork fix was the right layer. Do NOT reintroduce a `strong_count` guard: it
false-positives on the 16-paper cluster (witnesses 0805.2376 dcpic, 1407.0452
emulateapj).

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

   **UPDATE (2026-07-05 commit review of #46 `2b1ebe2492`).** The anchor was
   later moved from `getSize` to the font's TFM *quad* (em value), so the
   `em × --ltx-fo-*` box geometry is exact for every font. CAVEAT surfaced by
   the review and NOT yet resolved: that same `font-size` is *inherited by the
   foreignObject's visible content* (`.ltx_foreignobject_content` sets no
   own font-size), so text now renders at the quad, not the design size —
   cmtt10 emits 10.5pt vs 10pt design (~+5%), cmr7 ~+14%. The geometry win and
   the text-size drift are coupled through one attribute; splitting them
   (geometry off the quad, text off the design size) is the open follow-up.
   The "so `\small` contexts emit 8pt" line above describes the *intent* for
   text size; #46 optimized for geometry, so the two are momentarily at odds.

Debugging recipe: bisect with `\setbox0=\vbox` probes against reference Perl
(`perl -I LaTeXML/lib LaTeXML/bin/latexml`, `--debug=size-detailed`) AND
pdflatex ground truth; both engines deliberately over-estimate, so chase
*divergence from Perl*, not from TeX.

## #61 `\usepackage[...]` option values must be stored with LETTER catcode — kvoptions `\equal`/`\ifx` validation is catcode-sensitive

`\opt@<name>.<ext>` (the stored `\usepackage[opt=val]` option list, read back by
kvoptions/keyval `\ProcessKeyvalOptions` → `\setkeys`) was built with
`Explode!` in `latexml_core/src/binding/content.rs::before_input_handle_options`
(the `\opt@…` `def_macro`). `Explode!` emits every char as `CharToken` = OTHER
(catcode 12). Real LaTeX keeps the option tokens as read, so alphabetic chars are
LETTER (catcode 11). The divergence is invisible until a package **compares** a
`\DeclareStringOption` value: `ifthen`'s `\equal` (and a bare `\ifx`) compare
replacement text *including catcodes*, so `\equal{\axp@bibliography}{common}`
returned FALSE for a catcode-12 "common" vs the source's catcode-11 "common" —
even though `\meaning` shows an identical `macro:->common`, and even though a
plain `\def\x{common}` compares equal.

**Fix:** use `ExplodeText!` (alphabetic → LETTER, others → OTHER) for the
`\opt@…` body — the exact same fix already applied a few lines up to
`\@currname`/`\@currext` (which broke kvoptions's `\ifx\@currext\@pkgextension`
the same way; witnesses cond-mat/9611206, math/9904040). One-liner, but broad
reach: every package that validates a string option via `\equal`/`\ifx`.

**Diagnosis recipe** when a package spuriously rejects a *passed* option while
its *defaults* work: the tell is "default value → `\equal` YES, passed value →
`\equal` NO". Confirm with `\edef\x{\detokenize{val}}\ifthenelse{\equal{\x}{val}}`
— if that is NO, the comparison is catcode-sensitive and the stored value is
catcode-12. Do NOT "fix" `\equal` (it is faithfully catcode-sensitive, matching
pdflatex); fix the value's catcode at the storage site. Witness: apxproof
`bibliography=common` (gdsm.tex, KNOWN_PERL_ERRORS #44); regression fixture
`tests/keyval_options/optcatcode*`.

## #62 A figure of bare `\includegraphics` that can't be measured wraps its rows by FILENAME LENGTH — the unmeasured graphics box falls back to summing its argument boxes (the path string)

`arrange_panels_and_breaks` (latex_constructs) partitions a multi-image float
into rows by inserting `<ltx:break>` when the accumulated panel WIDTH exceeds the
float width — a faithful port that reproduces the PDF's per-row arrangement when
the source gives no explicit `\\`. The per-panel width is
`getNodeBox($child)->getWidth` — the **measured** graphics box, NOT the requested
`width=` on the element.

The trap: `image_graphicx_sizer` (`latexml_core/src/util/image.rs`) measures the
image file via `read_image_dimensions`, which — like Perl's `imgsize` — reads
PNG/JPEG/EPS only, **not PDF/SVG**. On a miss it early-returned WITHOUT setting
`cached_width`. `Whatsit::get_width` then fell through to `compute_size`, which
for a `\includegraphics` whatsit sums its ARGUMENT boxes — and the Semiverbatim
path argument is one of them, so the box width became the **rendered width of the
filename string**. A figure of 12 uniform `width=0.245\textwidth` PDF panels then
wrapped 3/3/2/3/1 (widths tracked `figures/WPO1d_J.pdf`=99pt …
`figures/WPO10d_w0density.pdf`=141pt — monotonic in path length). Witness
arXiv:2409.16471 fig 2.

Same-host Perl is NOT a useful oracle here: Perl measures PDFs via **ImageMagick**
(`Util::Image::image_size`, `pdf:use-cropbox`), which is an OPTIONAL dep. Without
`Image::Magick` installed (this host), Perl's `image_graphicx_size` bails at
`return unless $w && $h` (L226) and the sizer returns `Dimension(0)` (L272) → all
panels width-0 → the `$child_width == 0` heuristic MERGES them into one
`<ltx:block>` → a single row. So the three outcomes were: pdflatex/Perl-with-IM =
3 rows of 4; Perl-without-IM = one merged row; Rust = filename-length garbage.

**Fix** — emulate pdfTeX, NOT Perl. pdfTeX's built-in reader takes a PDF's
CropBox (its default, verified against `pdftex.def` + `\the\wd` under pdflatex)
or MediaBox, and an SVG's viewBox — with no external tool. `image_graphicx_sizer`
does the same in pure Rust: on a raster-reader miss it calls `natural_size_pt`
(shared reader `read_pdf_page_box` = CropBox→MediaBox, also used by
`LaTeXML::Post::Graphics`; `read_svg_size_pt` = width/height/viewBox), then
applies the graphicx transform IN POINTS (`graphicx_box_pt`). Measured pdflatex
truth that this matches: with an explicit `width=`, the box width IS the request
(`0.245\textwidth → 84.52332pt`) and the natural size only fills in the height
via the aspect ratio; only bare / `scale=` / `height=`-only inclusions actually
need the file read. When even the byte reader can't see the box (page dict
compressed into an object stream — where pdfTeX's full parser still would), fall
back to the requested `width=` (else 0), and ALWAYS set `cached_width` so
`compute_size` never sums the filename.

This reproduces pdfTeX / Perl-WITH-ImageMagick (and the PDF) with no ImageMagick
runtime dep — a portability + fidelity win. Reach is corpus-wide but NARROW:
`width=` figures get an identical box width either way, so only no-explicit-width
PDF/SVG figures change; the golden suite is untouched (every test graphic is a
measurable `.png`/`.jpg`). Regression tests: `figure_panel_native` (native
CropBox path) + `figure_panel_unmeasured` (the `width=` fallback / filename-bug
guard). Verified: fig 2's 12 panels → uniform 84.52pt → breaks after g4/g8 → 3
rows of 4; a bare full-size PDF → one panel per row; `scale=0.16` → wraps by the
scaled natural size.

## #63 Compiled bindings ALWAYS beat a raw `.sty` — the precedence gate is `input_definitions`'s `_loaded` check, NOT `find_file_aux`'s `notex` gate

**The trap:** reading `find_file_aux` in isolation suggests `--includestyles` lets a
raw on-disk `.sty` override a compiled binding. It does not, and reasoning from
that function alone will produce a wrong answer (it did twice in the 2026-07-19
issue #307 investigation, once in a comment nearly posted to a reporter).

`find_file_aux`'s binding fast-path (`content.rs:2703`) IS gated on `notex`, and
`require_package` (`content.rs:1744`) does clear `notex` under `INCLUDE_STYLES`.
But that pair answers a *different* question — "does this name resolve at all?",
as asked by `\IfFileExists` / `\openin` — and never decides package-load
precedence. Precedence is decided one layer up, in `input_definitions`
(`content.rs:101`), a stepped ladder mirroring Perl `Package.pm`:

* **Step 1/2** — binding dispatch (`is_binding`, `content.rs:481`/`:505`); on
  success sets `{filename}_loaded`.
* **Step 3** — fallback binding via version-suffix stripping (`content.rs:615`).
* **Step 4** — raw TeX (`content.rs:680`), guarded by
  `else if lookup_bool("{filename}_loaded") → None` (`content.rs:698-702`).
  Only when NO binding loaded does it call `find_file(…, notex: false)`, which
  is the sole route to a disk `.sty`.

So `notex`/`INCLUDE_STYLES` only controls whether Step 4 is *eligible*; for any
package that HAS a binding, Step 4 is already unreachable. Mirrors Perl's
`if/elsif` flow (`Package.pm:2118-2125`), which `return`s on binding success.

**Evidence** (shipped 0.7.4 asset, `\usepackage{latexml}` with a real
`latexml.sty` — which contains `\newif\iflatexml\latexmlfalse` — on `--path`):
both with and without `--includestyles` the log prints `(Loading latexml_sty.rs…)`
and the document takes the TRUE branch. Corollary: a compiled binding needs no
path and can never be "not found", so **never advise a user to drop a raw `.sty`
somewhere to change binding behavior** — it has no effect.

**Consequence for triage:** when a `\if<pkg>` conditional takes the wrong branch,
the cause is never "the raw `.sty` won". It is that the `\usepackage` was never
*executed* — e.g. it lives in an `\input`ed file that failed to resolve. Witness:
`\iflatexml` is defined only by `latexml.sty` (Rust
`latexml_package/src/package/latexml_sty/mod.rs:239`; Perl
`latexml.sty.ltxml:27` — neither predefines it), so a bare `\iflatexml` errors
`undefined` and falls into `\else` in BOTH implementations, byte-identically.
That is parity, not a bug.

## #64 "Perl recovers where Rust loops" can mean **Perl never had the macro** — inherited-kernel-macro leaks are a whole bug class

Rust raw-loads `latex.ltx` into the kernel dump; **Perl LaTeXML does not**. So a
control sequence can be *fully defined and TeX-faithful* in Rust while being
plain `undefined` in Perl. When such a CS is a **raw TeX implementation of
something LaTeXML models structurally**, digesting it is worse than not having
it: LaTeXML's constructs do not implement the low-level machinery (`align_state`,
`\lastbox`, `\futurelet`-driven brace juggling) the kernel body relies on.

**Canonical case (2026-07-20, arXiv:2605.23849).** `\kbordermatrix` uses the
documented `\bordermatrix` idiom `\let\\\@arraycr` inside its own `\ialign`.
The kernel's `\@arraycr` (latex.ltx L16583-16585) is

```tex
\protected\def\@arraycr{${\ifnum0=`}\fi\@ifstar\@xarraycr\@xarraycr}
\def\@xarraycr{\@ifnextchar[\@argarraycr{\ifnum0=`{\fi}${}\cr}}
```

— the `$`/brace pair exists purely to keep TeX's `align_state` balanced while
`\halign` scans for `\cr`. LaTeXML has no `align_state`, so the `$`s are digested
as real mode switches, re-opening an inline-math frame the alignment's
column-*after* template can no longer balance:
`Error:unexpected:\halign Attempt to close a group that switched to mode math`,
then a runaway to the token limit (~149 s, 0 formulae). Perl "completed in 0.4 s"
only because `\@arraycr` was undefined and it **skipped the whole matrix**.

**Fix shape** — retract the entry point to LaTeXML's own model, exactly as Perl
already does for the tabular sibling (`latex_constructs.pool.ltxml:3612`,
`Let('\@tabularcr','\lx@alignment@newline')`):

```rust
Let!("\\@arraycr", "\\lx@alignment@newline");
```

Aliasing the *entry point* retracts the whole `\@xarraycr`/`\@argarraycr` chain,
and `\lx@alignment@newline` already reads the same `*` and `[dim]` arguments.
Result: 0 errors / 1.9 s / 985 formulae, vs Perl's 3 errors / 52.7 s — same 985
`Math` and 8 `XMArray` counts, so structure is preserved, not degraded.

**Three transferable rules.**

1. **A Perl "0 errors" that comes from an `undefined` is not a target to match —
   it is a construct Perl dropped.** Compare *structure counts* (formulae,
   arrays, sections), not just error counts, before calling Perl the better
   result. Here Perl's 3 errors WERE the whole bordered matrix going missing.
2. **Bisect by hand-expanding the suspect macro.** Substituting `\@arraycr`'s
   body inline was clean in both engines while the macro was not — that one
   experiment moved the fault from "deep mode/frame accounting" (two people's
   prior hypothesis, plus a reverted fix attempt) to "one inherited kernel
   definition", and it is cheap to run.
3. **Look for siblings whenever you find one — then check each for a
   consumer.** The retraction list is a deliberate seam: `\@tabularcr` and `\+`
   were already there; `\@arraycr` was the missing third. `latex.ltx` has exactly
   four sites using this ``\ifnum0=` `` trick — `\@arraycr`, `\@tabularcr`,
   `\@eqncr`, `\hline` — and `\hline`/`\@xhline` are already bound by both
   engines.

   **But `\@eqncr` must NOT be retracted, and this was measured, not guessed.**
   Synthetically it looks identical (`\let\\\@eqncr` in a raw `\halign`: Rust 15
   errors vs Perl 1, same as `\@xtabularcr` at 13 vs 1). The difference is that
   `\@eqncr` has a **real consumer that depends on the chain**:
   `latexml_contrib/src/equations_sty.rs` redefines `\@@eqncr` — the kernel
   `\@eqncr`→`\@yeqncr`→`\@xeqncr`→`\@@eqncr` path is how it emits its column
   padding *and* `\@eqnnum`/`\stepcounter{equation}`. Retracting the entry point
   skips all of it: on an `eqaligntwo` the equation **numbers disappear and the
   remaining ones renumber** (verified by diff; error count stayed 0 both ways,
   so an error-count gate would have missed the regression entirely). The
   array/tabular continuations (`\@xarraycr`, `\@argarraycr`, `\@xtabularcr`,
   `\@argtabularcr`) are likewise left alone: they are unreachable once the entry
   points are retracted, `\@xtabularcr` is itself redefined by `tabls.sty`, and
   the only demonstrated harm needs a `\let\\\@xtabularcr` nobody writes.

   So the rule is narrower than "retract the family": retract a kernel CS **only
   where LaTeXML fully models the construct and nothing consumes the kernel
   chain**. Diff the *output*, not the error count, before deciding.

Neutrality argument worth reusing: the change is observable **only** by documents
that name `\@arraycr` (no Rust binding and no `.ltxml` references it) — measured
at **6 of 6,000** 2605 papers, three via the direct `\let`. See
`docs/known_crashes/kbordermatrix_halign_math/`.

## 48. Patching an EXISTING definition in place → `Scope::InPlace`, never `Scope::Global`

**Symptom / trigger:** the BookML `LookupDefinition(cs).push*/unshift*` idiom
(Rhai #321) — and any code that clones an installed def, splices a hook, and
re-installs it. The naive re-install uses `Scope::Global`.

**Why Global is wrong:** Perl does NOT re-assign when BookML runs
`push(@{ $$def{beforeConstruct} }, sub{…})`; it mutates the shared blessed
def-hash **in place**, which never touches the save stack. `Scope::Global`
instead collapses every frame down to the locked base and wipes lower-frame
undo, **promoting a locally-bound def to global**. Harmless *only* because the
sole user (BookML) patches already-global defs (`\hrule`/`\vrule`/`\rule`); a
patch applied to a def bound inside a group would wrongly survive group exit
(flagged by @xworld21, PR #333 r3623947537).

**Fix — and the reusable fact:** Perl `State.pm:175` has a fourth scope,
`'inplace'` ("Special case for `\box` & friends"): replace the front binding in
its own frame, add no undo entry (or seed at the locked base if never bound).
That is Knuth's "same level" / xworld21's tentative `scope => 'definition'`.
Ported as **`Scope::InPlace`** (state.rs enum + `assign_internal` arm; the
`\globaldefs` override deliberately does NOT re-scope it — Perl's guard is
`$scope ne 'global' && $scope ne 'local'`). The Value-table fast path
`assign_value_inplace` (mode changes, WISDOM #19) was the pre-existing witness
that this scope existed; #48 just lifts it to a first-class scope so the Meaning
table (definitions) can use it. Semantics pinned by
`state::reentrancy_tests::inplace_scope_keeps_the_bindings_level` (proves it is
neither Global nor Local across a `push_frame`/`pop_frame` boundary).

## #65 Post-processing `PostDocument::findnodes` with NO context node evaluates ONLY absolute paths — a relative axis silently matches nothing (and cross-doc node copies need a target-doc namespace)

Two bugs, one witness (split HTML pages losing their `LaTeXML.css`/`ltx-book.css`
`<link>`s — GitHub #341; Perl `latexmlc --format=html5 --splitat` links them on
every page).

**Bug 1 — the relative-axis trap.** `latexml_post::PostDocument::findnodes(xpath)`
(no context node) calls rust-libxml's `evaluate_checked`, which leaves the XPath
**context node unset**. libxml2 then resolves only **absolute** paths (`//…`,
`/…`); a **relative** location path — `descendant::…`, `.//…`, `child::…` —
matches **NOTHING**, unlike XML::LibXML's `$doc->findnodes` (which evaluates from
the document node). This silently broke `Post::Document::newDocument`'s
`descendant::ltx:resource` / `.//processing-instruction('latexml')` copies into
each split sub-document, `Split`'s `descendant::ltx:navigation`, CrossRef's
`descendant::ltx:glossaryref` / `descendant::*[@decl_id or @meaning]`, and the
graphics/documentclass PI reads — all returned empty for years.

**Fix:** `findnodes_at` binds the **root element** as the context node when none
is given, so relative axes resolve. You **cannot** bind the *document node* —
rust-libxml's `node_evaluate` **SIGSEGVs** on a document-node context (unguarded
FFI in the fork; see [[rust-libxml-null-nodeptr-segfault-fix]]). Consequence:
nodes **outside** the root element — `<?latexml …?>` PIs that precede it — are
not descendants of the root, so a relative PI query still needs the **absolute**
`//processing-instruction('latexml')` form (used at the 4 PI call sites +
`newDocument`). Guard: `document::tests::findnodes_resolves_relative_axes_without_context_node`.

**Bug 2 — cross-document namespace SIGSEGV (was hidden behind Bug 1).** Once the
resources were actually *found*, `add_nodes`→`add_xml_node` cloned them with
`parent.new_child(source.get_namespace(), …)` — reusing the **source** document's
`xmlNs` object inside the **target** sub-document. That cross-doc namespace
pointer SIGSEGVs / corrupts the tree when either doc is freed. **Fix:** resolve
the namespace in the **target** document (find-or-create a decl with the same
URI, preferring the default/empty prefix), mirroring the `NodeData::Element`
path. Any cross-document node copy (split, MathImages fork, bibliography merge)
must reconcile namespaces into the destination — never carry the source's `xmlNs`.

Both fixes are general (they repair every affected caller). Same pass also ported
`newDocument`'s missing `addDate` (Perl Post.pm L774) and removed a
duplicated class-copy block. Guard: `13_split_css_links`.

## #66 A named scope is a self-terminating region marker — `activate_scope`/`is_scope_active`, not a bare frame-depth test

**When:** code must ask *"am I inside a bracket LaTeXML itself opened?"* — the
`standalone` child preamble (`standalone_sty.rs`, after its `bgroup()`),
`import.sty`'s `\lx@activate@subfile@scope` `{…}` — so a package loaded there
survives the pop (OXIDIZED_DESIGN #65, KNOWN_PERL_ERRORS #55, issue #311).

**Mechanic:** `activate_scope(subfile_scope_here())` marks `StashActive`
with **`Scope::Local`** (`state.rs`, Perl `State.pm:683`), so the frame that set
it undoes it: no matching "deactivate" for a call site to forget, and it no-ops
when already active, so nesting is free. Test activity with `is_scope_active`, i.e. Perl's inline
`$$self{stash_active}{$scope}[0]` (State.pm L682): the truthiness of the FRONT
value, never key presence. Deactivation **overwrites the front value with
`false`** rather than removing the key — a plain global assign (State.pm L701),
and a global assign replaces rather than layers, leaving exactly one value. So
the front value is the whole state. (Delete is not on offer anyway: the table
rides the generic undo machinery, whose per-frame pop counts a removed key would
desynchronise.) Guards:
`state::reentrancy_tests::{scope_activity_tracks_value_not_presence,
a_deactivated_scope_can_be_reactivated, scope_activation_is_bounded_by_its_group}`. Named scopes are the engine's own
vocabulary for a region of state (Perl's are counter- and label-derived,
`section:4` / `label:foo`, State.pm L965-975) — reach for one before inventing a
boolean Value plus a manual restore.

**Trap 1 — activity ALONE is not the test; the name must carry the frame
depth.** `StashActive` is `Scope::Local` at the bracket's frame, so a bare "is the
region active?" reads true at every *deeper* frame — an author's
`{\usepackage{…}}` written inside a subfile preamble then gets hoisted too, and
Rust drops an error below Perl. Hence `subfile:<depth>`, Perl's own
`section:4`/`label:foo` shape. A bare `get_frame_depth() > 0` is worse still:
it matches any author group anywhere. It also matches a group the AUTHOR wrote, which must keep failing
(OXIDIZED_DESIGN #65) — and the two are indistinguishable by anything cheaper:
same file, `inPreamble` true, `\currentgrouplevel` 1. That is *why* the region has
to be named. Guard:
`06_cluster_regressions::author_written_group_around_usepackage_still_loses_the_package`.

**Trap 2 — hoist CONDITIONALS, not everything.** The hooks read `\ifX`; a
package's ordinary macros must stay scoped, or a second sibling subfile's
same-named `\newcommand` becomes a silent no-op and renders the first sibling's
body (worse than Perl). **Trap 3 — `require_package` is idempotent** (`input_definitions`'s
`already_handled` `_loaded`/`_raw_loaded`/`_load_attempted` check in `content.rs`;
the binding-vs-raw precedence half is WISDOM #63), so
"clear and re-load" does *not* reinstall: hoist the installed defs instead
(`snapshot_top_frame_meaning_keys` + `hoist_top_frame_meaning_delta`, Meaning-only
— Value/Catcode/register state stays frame-local; OXIDIZED_DESIGN #65). See also
WISDOM **`48.`** — `Scope::InPlace` for patching an existing definition, in the
plain-numbered series, NOT `#48`: it says don't promote out of the frame, #66
says do, but only past a bracket that is ours.

## #67 A `.tex`/`.xml` suite must give every pair its own `#[test]` (`tex_tests!`) — the runtime `latexml_tests_internal` glob SIGABRTs on the SECOND conversion in one thread

`latexml_tests_internal(dir, …)` (`latexml_oxide/src/util/test.rs`) globs a
directory and converts every `.tex`/`.xml` pair **inside a single `#[test]`
fn**, i.e. one libtest thread. The engine's roots are `#[thread_local]`
*attribute* statics that are torn down per test by `reset_thread_engine`
(WISDOM: `feedback_test_oom_leak`), so the second `initialize_singletons()` in
the same thread re-runs the engine's `Let!`s against interner ids from the first
one. That trips `slice::get_unchecked`'s precondition inside `state::let_i` — a
**non-unwinding** panic, so it is a SIGABRT of the whole test binary, not a
catchable failure, and the backtrace points at engine bootstrap with no trace of
the fixture that actually triggered it.

The tell is that the crash appears the moment a directory gains its *second*
fixture and vanishes when it is removed — and that the same second document
converts perfectly through the CLI. Two copies of the SAME passing `.tex`
reproduce it, which is what separates "my new fixture is broken" from "this
harness never ran two documents".

`tex_tests!` (`latexml_codegen/src/testable.rs`) is the fix and the tree-wide
convention: it emits one `#[test]` per pair at compile time, and libtest gives
each its own thread. Every other directory-wide suite (`65_graphics`,
`51_structure_rhai`, …) already uses it; `00_contrib` was the last runtime-glob
holdout and only ever held one fixture, so the defect stayed latent until
issue #347 added `tests/contrib/cprotect.{tex,xml}`. **Adding the second pair to
a directory is what surfaces this** — if a suite still calls
`latexml_tests_internal`, convert it rather than debugging the abort.

Note the compile-time glob: a new pair needs the test target rebuilt (touching
the suite's `.rs` suffices; `cargo clean` is the sledgehammer) or it is simply
not discovered.

## #68 Porting a Perl `while (defined($x = read()))` loop: a post-loop test on `$x` means "the read ran out", NOT "we never read"

Perl's idiomatic reader loop leaves the loop variable holding the value that
*ended* the loop, and the code after it discriminates on that:

```perl
while (defined($token = $gullet->readXToken(1))) {
  ...
  last if $terminal and Equals($token, $terminal);
  last if $initdepth > scalar(@{ $$self{boxing} }); }
push(@LaTeXML::LIST, Box()) unless $token;       # Stomach.pm:130
```

`unless $token` is true in exactly one case: the `while` condition failed, i.e.
**the input ran out**. It is false for every `last`, because `$token` still holds
a live Token. Rust's `while let Some(token) = ...` **drops** that binding at the
end of the loop, so the discriminator has to be reconstructed — and it is easy to
reconstruct the wrong one.

`digest_next_body` (`stomach.rs`) reconstructed it as `found_token` — "did we ever
read a token" — set inside the loop body. That agrees with Perl only on a body
that was empty from the very start, and silently disagrees on the case the code
exists for: read some content, *then* hit EOF. The correct shape is a flag that
starts `true` and is cleared at each `break`:

```rust
let mut ran_out = true;
while let Some(token) = read()? { ...; ran_out = false; break; ... }
if ran_out { push_box_list(Digested::from(Tbox::default())); }
```

**Why it mattered.** That trailer box is what makes `readDigested`
(`Base_ParameterTypes.pool.ltxml` L374) safe: it does
`push(@list, digestNextBody()); pop(@list);` to strip the closing-brace box. With
no trailer on the EOF path, the `pop` removed a box of **real content**. A single
runaway `.bib` field (a bare `%` — literal data to BibTeX, a comment to TeX) then
emptied an entire bibliography that same-host Perl renders in full, while
reporting *fewer* errors than Perl. Fixed 2026-07-26 with
`55_bibtex::runaway_field_costs_only_its_own_entry`; the companion mouth-boundary
half is #69 below.

**Method.** When porting any Perl loop, ask what the loop variable holds *after*
each exit path before translating a post-loop condition on it. The `last`-vs-
condition-failure distinction is invisible in the Rust rewrite and produces a
silent behavioural narrowing — no compile error, no test failure on well-formed
input, and damage only on malformed input, which is exactly where error recovery
is judged.

## #69 A balanced read must not cross out of a self-contained mouth

`read_balanced` (`gullet.rs`) crosses a mouth boundary when the exhausted mouth is
autoclose and not a file — a deliberate surpass-Perl divergence for xint's
`\edef\X{\scantokens{…}}`, where the matching `}` really does live in the parent.
Perl never crosses: `Gullet.pm` L465-472 reads `$$self{mouth}->readToken()`, the
current mouth only, and `last`s at its end.

"Literal mouth" is NOT evidence of a continuation. `\ProcessBibTeXEntry` replays
each entry through `Mouth::new` too (Perl `BibTeX.pool.ltxml` L165-166), and there
a runaway argument crossed into the wrapper and consumed every following
`\ProcessBibTeXEntry` *and* `\end{bibtex@bibliography}`.

The property is now explicit — `open_mouth_with(mouth, autoclose,
BalancedBoundary::{Transparent,Opaque})` — rather than inferred from
`autoclose && !File`. Declare **Opaque** whenever the mouth carries a
self-contained input; reserve Transparent for token-level injections
(`\scantokens`, RawTeX). Related: #68 above.

## #70 Byte-index scanners over `&str` are a standing hazard of this port — scan by CHARACTER (`CharCursor`), and remember the bug is in the ADVANCE, not the slice

`&str` carries one compiler-enforced invariant — valid UTF-8. A `usize` used to
index it carries **none**. So every `&s[a..b]` is an unchecked assertion that
both ends are char boundaries, and when it is wrong the program does not return
an error, it **panics** — aborting the whole document.

That is unusually dangerous *here* specifically, and it will recur: Perl strings
are sequences of **characters** (`pos`, `substr`, `\G` have no boundary concept),
so every hand-rolled `as_bytes()` + `i += 1` scanner translated from Perl
introduces an invariant the original never had. Silently, with no type-level
trace, and passing every ASCII fixture forever.

**The rule.** The panic fires at the slice; the defect is always in the advance:

| advance | safe? |
|---|---|
| scan to an ASCII delimiter | **always** — an ASCII byte is never a UTF-8 continuation byte |
| by `char::len_utf8()` | **always** |
| a fixed count past an unclassified byte | **never** |

That is why a walker can be correct for years and break on one edit: three of the
four arms in `recase_title` only ever *stopped* at ASCII, and the fourth
(`\<char>`) advanced one byte past the backslash — right for `\&`, fatal for
`\“`. Witness 2605.22125, found by the 2026-07-26 sandbox sweep, not by tests.

**What to use.** `latexml_core::util::char_cursor::CharCursor` — a thin wrapper
over `char_indices` (Rust guidelines `anti-index-over-iter` /
`perf-iter-over-index`; do NOT invent a newtype, std already has the iterator).
Its value is what it withholds: no advance-by-byte-count exists, so `slice_from`
is infallible and the class is unrepresentable. Deliberately NOT an `Iterator` —
`by_ref().take_while()` consumes the item that fails the predicate, desyncing
`peek`/`pos` and making a mark meaningless.

**Convert even the arms that are provably safe.** All three `bibtex.rs` walkers
were converted, though only one had ever panicked: "happens to be safe" is
exactly the state `recase_title` was in.

**Testing.** Enumerate rather than sample where the space is small: 4 UTF-8
widths x scanner position x mode is complete coverage and needs no proptest
dev-dependency (which would touch the license inventory and `cargo-deny`).
Assert across CASE FORMS when the function re-cases — a literal `contains(c)`
check fails on `Uppercase` turning `a` into `A`, and that is the test being
wrong, not the code.

Guards: `bibtex::tests::recase_title_handles_every_character_width_at_every_position`,
`util::char_cursor::tests::*`. Related: [[#68]] above (the same "port a Perl
loop, inherit an invariant Perl never had" family).

## #71 A `Tokens` flattened with `Display` and re-tokenized WELDS control words — the tokenizing sinks take `TeXString`, so a bare `String` cannot reach them

TeX **consumes** the space that terminates a control word, so it is gone as data
by token time: `\v S` tokenizes to `[\v][S]`, and concatenating those token
strings gives `\vS` — a control sequence that exists in no LaTeX. `untex()`
re-emits the space; `Display`/`to_string()` deliberately does not.

This is **not** a divergence. Perl is identical (`Core/Tokens.pm:61 toString`
joins the token strings, `Core/Token.pm:306` returns a CS name with no trailing
space) and its comment — which our port copies verbatim — already says the
result is "NOT for creating valid TeX (use revert or UnTeX for that!)". Perl
protects itself by author discipline alone. That failed three times here, each
found by a user-visible failure years later: `\bib@@names` (PR #399),
`dcolumn`/`overpic` (PR #400), and `\bib@synthesize@mr` (issue 410 —
`MRREVIEWER = {Dragomir \v{Z}. \Dbar okovi\'{c}}` → `undefined:\Dbarokovi`;
witness 2605.11579, 5 welds over 36 bibitems, Perl 0). *Re-measure that count
before quoting it: OXIDIZED_DESIGN **#80** now digests only the CITED entries, so
the `\Dbar` one (`KacNilpotentorbits`, `biblo.bib` L2059) is no longer read at
all. Other accented `MRREVIEWER`s **are** in cited entries (`BM93`
`{Fran\c cois\ Digne}`, `Nek03` `{Marcos\ Mari\~no}`, …), so the shape still
occurs — only the tally moved. The guard fixture, not the paper, is what pins
this.*

**The rule.** All three mouth entry points — `mouth::tokenize`,
`mouth::tokenize_internal`, `mouth::tokenize_bib_literal` — and therefore
`Tokenize!`/`TokenizeInternal!`/`bib_tex_tokens` take `impl Into<TeXString>`
(`latexml_core::tokens`). Its value, like `CharCursor`'s in [[#70]], is what it
withholds — there is `From<&'static str>` and nothing else:

| you have | you write |
|---|---|
| a TeX literal | nothing; `&'static str` converts implicitly (~125 sites untouched) |
| a `Tokens` | `t.untex_string()` |
| a `format!` of literal TeX around safe pieces | `TeXString::assembled(…)`, whose doc names the obligation |

`s!(…)` returns `String` and `&some_var` is a short-lived borrow, so neither
coerces: a welded flatten fails to compile (E0277) or fails the borrow check
(E0521 "`'1` must outlive `'static`"). **Do not add `From<String>`.**

**Not every flatten is a weld** — the audit matters. A string that becomes a CS
*name* (`T_CS!(cs.to_string())`), an `ExplodeText!` char run, a `.parse()`, or a
comparison cannot weld, and those keep `to_string()`. So does `Display for
Tokens` itself: 500+ sites use it for keys and names, and making it TeX-correct
would silently change all of them.

**Method — census a call site family without breaking the build.** Replace/shadow
the trait method with a *deprecated inherent* one (`impl Tokens { #[deprecated]
pub fn to_string(&self) }`): an inherent method wins over `ToString`, so every
call site warns while `format!("{t}")` stays silent. Collect the warnings
workspace-wide, classify, then remove the shim. Measured 547 sites, of which 18
reached a TeX-source sink and 6 could weld.

Guards: `tokens::texstring_guard_tests` (compile-time — `String`/`&String` must
NOT be `Into<TeXString>`, `&'static str` must be; plus a `compile_fail`
doctest), `bib_name_space_form_accent_survives_reversion`,
`bib_mr_reviewer_accent_survives_reversion`. Related: [[#70]] (the same
"withhold the operation and the class disappears" shape).

## #72 A bibliography sub-conversion's FATAL carries no fatal-severity message — it surfaces on the parent as a trailing `error bibliography/convert`

**The trap.** Mining a corpus run by clustering on **fatal messages** buckets
these documents as *"no fatal recorded"* — measured, **~80 documents** in one
sweep. Their conversion really did die; the death simply has no `Fatal:` line to
cluster on. Cluster on the **status code** (`Status:conversion:3`) or on the
document's last `Error:` instead, and treat `bibliography/convert` as a
fatal-class marker.

**Why it is built this way, and why it is right.** `bib_session.rs` imports the
**post-phase** diagnostic macros (`latexml_post::{Error, Info}`), which are plain
reporters with none of the too-many-errors escalation the engine-side `Error!`
carries. The recursive `.bib` session runs *inside* post-processing, so an
inner-session failure must not itself become a document Fatal — and Perl's
`convertBibliography` does the same, returning empty-handed
(`MakeBibliography.pm` L240-242). The `Err` arm therefore emits
`Error!("bibliography", "convert", "Recursive bibliography conversion failed: …")`
and returns `None`.

**What DOES cross the boundary** — do not confuse the two. Perl's `MergeStatus`
(`MakeBibliography.pm` L237, `Common/Error.pm` L669-686) adds the inner session's
tally to the outer document's, so a `.bib` that raises *errors* makes the
document an error document. Rust gets that for free by sharing the live core
State (the counters never left). It is only the *fatal severity* of a sub-session
collapse that is deliberately downgraded.

**Consequence for guards.** The ordinary `convert_and_post` test helper gates only
the CORE stage, so a post-stage error flood passes every bibliography guard
silently — 17 errors on one fixture, 203 on its witness, all green. Use
`convert_and_post_clean` for anything in this path (see OXIDIZED_DESIGN #73).

## #73 `raw_tex` is the only binding-side path that TOKENIZES a CS name — so binding-authored expl3 TeX needs the expl3 catcode regime

**The trap.** `RawTeX!` → `stomach::raw_tex` builds a `Mouth` and reads it with
the **ambient** catcode table (`at_letter: true` is the only override). Under the
document regime `_` is SUB, so a CS name written in a raw string terminates at its
first `_`: `\edef\c_sys_shell_escape_int{0}` is `\edef\c` with parameter text
`_sys_shell_escape_int` and body `0`. The intended constant is never defined, and
a **short, real** CS is silently rebound — here LaTeX's cedilla accent `\c`, so
every later `Fran\c cois` rendered "Fran0cois" with **zero errors** while Perl
rendered "François" (issue 421; `expl3_sty.rs`, witness arXiv 2605.11579). The
same shape threatens any `\c…`/`\v…`/`\u…`-prefixed expl3 name.

**The rule.** Binding-authored expl3-syntax raw TeX must run inside the
`\ExplSyntaxOn` regime — `expl3_sty.rs::with_expl_catcodes` saves the ambient
`:`/`_` catcodes, sets LETTER, and restores on both the success and error paths.
Do NOT hardcode the restore to OTHER/SUB: the caller may itself be an expl3
package (that mistake is the older half of this family, in
`docs/parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md`).

**The cheaper escape.** `T_CS!`, `Let!` and `parse_prototype`
(`def_macro_noop`, `def_macro_identity`, `def_primitive_noop`, …) build the CS
name as a **string** and never reach the tokenizer — `def_parser.rs::CS_RE`
admits `[a-zA-Z@_]+(?::[a-zA-Z]*)?` for exactly this reason. Prefer them for
expl3 names; reach for `raw_tex` only when you need real TeX control flow.

**Diagnosing it.** The symptom is a *wrong glyph*, not an error — grep the
output for the corrupted rendering, and probe `\meaning\<cs>` (a rebound accent
reads `macro:_…->…`). To ask whether a `\c_sys_*`-style constant is really
defined, use `\number\csname …\endcsname`, not bare
`\csname …\endcsname`: an `\int_const:Nn` chardef ≥ 256 expands to a glyph the
font lacks and renders as *nothing*, which reads exactly like "undefined" and
sent this investigation down a wrong path once.

---

## #74 The kernel dump is a usable "what will the format define" ORACLE — and it is NOT the same question as "what does `latex.ltx` define"

Loading `LaTeX.pool` on demand needs a membership test that runs *before* the
load: "load it and see if that helped" is not available, because it would drag
the LaTeX kernel into genuinely plain-TeX documents on their first undefined CS.
The reflex is to scan the host's `latex.ltx`. That is the wrong question.

`resources/dumps/latex.YYYY.dump.txt` is **not** a YYYY artifact. It is
generated by our *current* code inside a pinned TL-YYYY container, so its key
set is "what this engine's bootstrap of that TeX Live produces", which is
exactly what a caller wants to know. A `latex.ltx` scan answers a different
question and misses whatever our own layer contributes.

Mechanically the scan is cheap and safe: dump rows are
`<table>\t<key>\t<data>`, so `M\t`-prefixed rows yield the CS names with one
`splitn` per line and no record parsing, no arena interning, and no State
mutation (`dump_reader::collect_meaning_keys`). ~22k names on TL2025, built
lazily — never at startup.

Three properties any such "load a format from the undefined path" hook needs,
all learned the hard way elsewhere in this port:

1. **Claim the single attempt BEFORE the load, in State, not in a
   `thread_local`.** `input_definitions` sets `<name>.pool_loaded` only *after*
   the pool body runs, so that flag cannot stop the load re-entering itself
   through its own undefined CSes. A State value also resets per session, which
   a process-global flag does not — a test binary runs many conversions on one
   thread and the second one would silently lose the mechanism.
2. **Retry by pushback, not by returning the token.** `unread_one(token)` +
   `continue` (gullet) / + `return Ok(Vec::new())` (stomach) — returning a
   now-expandable token from `read_x_token` leaks it unexpanded to whatever
   asked for a fully-expanded head.
3. **Hoist the load's frame delta to global** (`snapshot_top_frame_meaning_keys`
   / `hoist_top_frame_meaning_delta`). Unlike the eager `TeX.pool` triggers,
   which in practice fire at top level, this can fire at any group depth, and a
   pop that takes the format away while `LaTeX.pool_loaded` survives is the
   silent-infinite-loop shape of witness 2606.21610.

And it must be inert where undefined CSes are *expected*: `LATEXML_INI_MODE`
(dump-build — otherwise a previous run's dump leaks into the next one) and
`SUPPRESS_UNDEFINED_ERRORS` (bulk raw loads with forward references).
See `latexml_engine/src/latex_kernel.rs`; the defect it fixes is
`KNOWN_PERL_ERRORS.md` #64.

## 79. Discarding DOM means FREEING it: rust-libxml unlink is not a destructor, and `append_tree` copies

Two facts compose into this port's largest single memory sink (~1.4 MB per
math formula; most of a 63.7 GB eager peak on a 19.8 MB book):

1. **`unlink()` / `unbind_node()` / `unlink_node()` are aliases and none
   frees a doc-owned node.** The wrapper's `Drop` is a no-op while
   `node->doc` is set, and `xmlFreeDoc` reclaims only what is reachable from
   the root — an unlinked-but-never-reattached subtree is freed by NOBODY.
2. **`Document::append_tree` re-CREATES every node** (faithful to Perl
   `appendTree`), so every `replace_tree`/parse-replacement abandons its
   entire SOURCE tree. Perl's refcount GC collects those; Rust must free.

The discard primitive is `Document::discard_subtree` → fork
`Node::free_subtree` (libxml 0.3.17): frees the C subtree immediately and
NEUTRALIZES every registered wrapper (shared `node_ptr` nulled). The
`set_rust_owned`-and-drop route is a trap: any stray clone in a long-lived
collection (`constructed_nodes`, an idstore epoch) defers the free past the
owning document's `xmlFreeDoc` — then `xmlFreeNode` reads the freed doc's
dictionary (SIGSEGV in `xmlDictOwns`; observed at the 113 gate).

Rules when freeing at a discard site:
* **COPY FIRST, free after** — the replacement can sit INSIDE the tree being
  discarded (`parse_single`'s single-child shortcut; tex_box's
  foreignObject-to-grandchild replace).
* **Ids are the caller's business, not the primitive's**: the copy re-records
  the SAME id strings, so unrecording at free time would kill the fresh
  entries. Unrecord BEFORE the copy.
* **Dedupe garbage by detached root** (`common::xml::detached_root`) — moved
  originals live inside the built tree; freeing both "roots" is a double
  free, and a chain ending at a Document node means NOT YOURS to free.
* **Purge ptr-keyed registries first** (`node_boxes`): freed addresses are
  reused, and a stale entry mis-associates a box with an unrelated new node
  (surfaced as phantom `<text font="italic">` wrappers).

Heaptrack discipline that found it: profile the STREAMING run (fragment
docs die before exit, so alive-at-exit ≈ true leak with stacks); an eager
run's exit snapshot conflates the live final DOM with the leak.

---

## 80. A font selected by FAMILY reaches its glyphs only through `\selectfont`'s family-as-encoding branch — so a `*_fontmap.rs` can be fully populated, registered, and still dead

`DeclareFontMap` parks every map in ONE namespace keyed by *encoding*
(`<enc>_fontmap`; a `family =>` option only ever *refines* it as
`<enc>_<family>_fontmap`). But several packages select their font by family and
carry an encoding nothing maps: `bbding`'s `\dingfamily` is
`\fontencoding{U}\fontfamily{ding}\selectfont`, and no `u.fontmap` exists in
either implementation. The only path from `ding` to `ding_fontmap` is Perl's
own hack in `\selectfont` (`latex_constructs.pool.ltxml` L5207-5209):

```perl
elsif (LoadFontMap($family)) {
  # Special case hack: Tentatively treat family as the encoding! (typically "U" encoding)
  MergeFont(encoding => $family); }
```

Rust lacked that branch, so `ding_fontmap.rs` was **dead code** — correct
table, registered loader, and no reachable caller. Every `\@chooseSymbol{N}`
fell to the OT1 fallback and emitted OT1 slot N's *text* character.

**Method, two durable parts:**

1. **To decide whether a declared fontmap is live, grep for who ever SETS its
   encoding** — `MergeFont!(encoding => …)` — not for its loader registration.
   A `pub mod x_fontmap;` plus a `("x","fontmap",…)` row plus a populated table
   look exactly like a working feature. Ask what assigns the encoding.
2. **This failure mode is invisible to every log-based signal.** The witness
   converted at `Status:conversion:0` with zero `Error:`/`Warning:` lines and
   telemetry `errors:0`, while 28 cells of its two main results tables read `%`
   and `!` instead of ✗ and ✓ — an OT1 slot's text character is ordinary
   content, so nothing downstream can flag it. The only signals that catch it
   are a glyph-level diff against Perl and a `.tex`/`.xml` golden.

**Trap when reaching for ground truth:** `pdftotext` on the pdflatex output
prints the *same* wrong characters (`! " # $ %`), because symbol fonts like
`ding` ship no `ToUnicode` map and extraction falls back to raw slot codes
33-37. So the PDF's text layer is NOT usable ground truth for a symbol font —
it agrees with the bug. Compare glyphs against Perl, or read the rendered page.

Witness 2503.04421. Guards: `latexml_oxide/tests/fonts/bbding.{tex,xml}` and
`tests/116_bbding_family_fontmap.rs`. The same primitive also lacked Perl's
`reported_unrecognized_font_*` report-once guards (`already_reported` in
`latexml_engine/src/base_utilities.rs`), which had turned one unrecognized
family into 28 identical `Info` lines.

Settled dead end: Perl's `DeclareFontMap` `(uppercase|lowercase|digit)_mathstyle`
options are a *separate* unported gap — Rust writes `OMS_uppercase_mathstyle`
(`tex_fonts.rs`) and `amsb_fontmap.rs` records a dropped
`uppercase_mathstyle => { family => 'blackboard' }` in a comment, but nothing
reads either key. Same defect class, not fixed here.

## 81. Before optimizing a per-item cost, check whether the item COUNT is a degenerate trigger — and a level-test with no floor is one

The 131 MB witness's 70-minute streamed conversion looked like "per-segment
costs × 459,579 segments"; two plausible per-segment optimizations (cache the
Marpa grammar, skip the spill walk) profiled at ~0 % because the real defect
was upstream: the soft-RSS yield trigger is a LEVEL test (`rss > watermark`,
`stomach.rs`), and a document whose irreducible resident floor sits above the
watermark latches it permanently — yielding at every legal seam, 24,051,712
times, producing 5.5 KB segments. A 1024-box accumulation floor (waived again
at `watermark + (fuse−watermark)/2` so the pathological-footprint valve
survives) cut yields ~16,000× and segments ~76×. Guards:
`115_soft_yield_floor` (red-tested), `soft_yield_floor_waiver_boundaries`.

Method, both directions: (1) when a per-item cost dominates, ask what sets the
item count before optimizing the item — the count may be one latched predicate;
(2) the attribution instrument must exist before the argument — every
`telemetry::phase()` guard sat on the eager path, so streamed runs reported
Digest = 0 µs and the 70 minutes could only be attributed by paired control
runs. One phase-guarded run then settled it: MathParse 41 %, Build 29 %,
Digest 22 % — and killed two more speculative optimizations. Companion waste
pattern, same fix-shape as #79: anything GENERATED then UNDONE is pure tax —
the spill intermediates carried 51.2 % indentation that pass 2 re-parsed into
~40 M text nodes and deleted (`spill_flat` now skips both halves; the deleted
`strip_indentation_whitespace` also orphaned every node it unlinked, the #79
trap).

## 82. Streaming byte-identity is a MECHANICAL ORACLE for construction-time live-state reads — and the audited class is contained to one fixed site

A `DefConstructor` body runs at CONSTRUCTION (XML-build) time, not digest time.
The eager path builds after the whole document is digested (so a live
`lookup_font()`/`lookup_value()` there sees the document's FINAL state); the
streaming path builds mid-document (sees LOCAL state). **So any constructor-body
read of mutable STATE that shapes output diverges eager-vs-streaming, and the
`114_streaming_*::streaming_matches_eager_on_*` sweep catches it — but ONLY where
a fixture exercises the construct.** #504 (`tex_glue::dimension_to_spaces` sizing
a faked space off the live font, [OXIDIZED_DESIGN #96]) was found only after a
fixture was added; the mechanism was blind to it until then. Fixture coverage is
the gap, not the sweep.

**Audit method (reusable).** Two complementary passes:
- *Static:* classify every `lookup_font()`/`lookup_value()` call by enclosing
  scope — a `sub[document, …]` constructor body is a suspect; `properties`/
  `sizer`/`after_digest`/`getter` closures and `DefPrimitive` bodies are
  digest-time (live font is CORRECT there); free helper fns need per-caller
  classification (the dangerous shape is a helper reading live font that is
  CALLED FROM a constructor — exactly `dimension_to_spaces`). A ~40-line Python
  walk-back-to-nearest-header script does this in one pass.
- *Dynamic:* a probe battery — each construction-time-sensitive construct
  (spacing, `\rule`, `\phantom`, fills, math spacing, `p{}` intercol, `\kern`)
  followed by a font-size shift (the #504 witness pattern) — run through the
  sweep, which reports the first diverging byte.

**Finding (2026-08-04).** Of 85 `lookup_font()` sites, ZERO are output-shaping
reads in a constructor body (the one flagged was a `properties` closure false
positive). Every other read is digest-time-correct: unit resolution
(`em_value`/`mu_to_pt`/`convert_unit`), font decode (`decode`/`merge_font`/
`font_decode`), and colour helpers all inside `DefPrimitive`/`properties`
bodies. #504's `dimension_to_spaces` was the SOLE instance. Guard:
`114_streaming_cluster_regressions` over
`tests/cluster_regressions/streaming_construction_time_spacing.tex` (one fixture
covering the whole class) + `faked_space_is_sized_by_the_font_it_was_digested_in`.

## #75 A TeX list-value (SEARCHPATHS-like) must live in the group-scoped value table, not a plain State field — TeX grouping then does its save/restore for free

**When:** porting a Perl `AssignValue(FOO => [...])` list — search paths, graphics
paths, any `@FOO`. Perl's value table is group-scoped and **local-by-default**
(`State.pm:152,169`), so `}` reverts it and an `\import`-style `{…}` wrapper is
the ONLY revert mechanism the Perl binding needs.

**Trap:** storing it as a plain `State` field (a bare `VecDeque`) takes it OUT of
the group stack — `pop_frame` reverts only the ten grouped tables. Two band-aids
then accrete to fake the scoping: an explicit save/restore stack in the binding
(`import.sty`'s former `\lx@save@paths`/`\lx@restore@paths`), and a guard around
package loading that snapshot-restores — which wipes a package's OWN additions
(#561).

**Fix:** route it through `lookup_value`/`assign_value("FOO", …, scope)` exactly
as `GRAPHICSPATHS` already does (`get_graphics_paths`, `state.rs`). Reads see the
current group's value; import writes go **local** (reverted by the `{…}`), a
package's persistent add goes **global**. Both band-aids disappear. `SEARCHPATHS`
was the last plain-field holdout; the two are now consistent. Guards:
`cluster_cli::dir_prefixed_package_loading::*`, `06_cluster_standalone_subfiles::subimport_sibling_calls_do_not_accumulate_search_paths`.

**Corollary — a zero-arg binding loader that drops the directory needs
`\@currname`, not a search-path guard.** `\usepackage{DIR/pkg}` dispatches to a
basename-keyed binding (dir stripped) that raw-loads its own basename; resolve it
by raw-loading the `\@currname` request (`DIR/pkg`) directly — as Perl does (no
binding → raw-loads the path) — not by injecting `DIR/` into SEARCHPATHS (which
also grants non-LaTeX auto-sibling resolution the `import` package exists to
provide). Witness 2510.09534 (`AISTATS/aistats2026`).

## 83. Deferred CLI flags stay hard parse errors, not accept-and-warn stubs (option C)

**Decision (2026-07-09, issue #191; salvaged from the retired `ISSUE_AUDIT.md`).**
The authoritative CLI spec is Perl `getopt_specification` (`Common/Config.pm`,
~82 canonical omni options; the `latexmlc` union). A flag whose engine feature
does **not** yet exist is left as a clap *"unexpected argument"* ERROR — never an
accept-and-warn stub — because a strict parser must never report misleading
success for an absent feature. Each flag is wired only when its feature lands
(e.g. `--includestyles`→`INCLUDE_STYLES`, `--timestamp`/`--icon`→ the XSLT
`TIMESTAMP`/`ICON` params, `--nographicimages`→ gate the Graphics phase). The
parser-library question the issue raised is settled: **clap 4 derive**.

Durable deferrals (feature absent by design or blocked, so the flag stays a hard
error): `--profile`/`--mode` → planned as **TOML** profiles deserialized into the
clap option struct, not Perl `.opt` ([`OXIDIZED_DESIGN_FUTURE_WORK.md`](OXIDIZED_DESIGN_FUTURE_WORK.md)
"TOML profiles"); `--validate`/`--novalidate` waits on the rust-libxml fork's safe
RelaxNG interface (`Post::Document::validate()` is a stub); `--svg` is **redundant**
— the HTML5 XSLT already renders `<ltx:picture>` as inline `<svg>`, so the
standalone `svg.rs` post-processor would only produce divergent, unverified output;
`--output` is an intentional non-goal (we keep `--destination`/`--dest`). Everything
else absent (mathimages/dvipng pipeline, jats/html4/tex/box outputs, crossref/
bibliography/index cluster, DTD-gated `--omitdoctype`, daemon mode) is a feature
gap kept as a hard error, revisited per-flag as the feature lands.

## 84. Keyval keys are read unexpanded; a binding assembles conditional key lists OUTSIDE the read

keyval.sty reads each `key=value` item as a delimited macro argument (`\KV@do#1,`),
so the KEY is never expanded and a `{…}` inside it is opaque. Perl
`KeyVals.pm:273-286` (and this port until batch 54h) expanded while scanning and
cut the key at the first `}`/`,`/`=` — enumitem's shortlabel `\setlist[test]
{\@risp,…}` with `\@risp` = `\fbox{\parbox…}` was unfolded into box code and cut
at an inner brace (verifica ×5). `read_keyword_from` (keyvals.rs) now uses
`read_token()` with a brace-depth counter; only a key that STARTS with
`\savevalue`/`\gsavevalue` (xkeyval.tex:140-146 `\XKV@ifcmd`) is expanded.

**Method for bindings**: a key list must not carry `\ifx…\fi` in key position
(mathtools' `\lx@mt@smallmatrix`, `\multlined`): decide before the read —
`\ifx/#2/\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{…}{…}` or an
`\edef` of the whole list handed over with `\expandafter{…}`.

Sibling rule: a leftover-key store (`\XKV@rm`) is fetched ONE STEP
(`lookup_expandable(cs)?.invoke(true)`, xkeyval.tex:497/618), never
`do_expand` — a value may name a macro defined only when its key code finally
runs (chessboard.sty:1439 `trimarea=\board`, `\board` \edef'd at :1087).
Guards: `perfect_kernel_batch54::{keyval_key_is_brace_aware,
setrmkeys_keeps_leftover_values_unexpanded, xkeyval_usevalue_*}`.
