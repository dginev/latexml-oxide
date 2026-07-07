# Oxidized Design — Type System & Tactical Insights

[← OXIDIZED_DESIGN.md](OXIDIZED_DESIGN.md) · Where Rust types beat stringly-typed Perl (behavior-neutral), plus hard-won internal pitfalls.

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
