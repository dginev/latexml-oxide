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
