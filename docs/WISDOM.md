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

## 9. Porting RawTeX() blocks: copy bravely and exactly

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
