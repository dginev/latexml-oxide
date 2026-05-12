# `plain_bootstrap.pool.ltxml` ↔ `plain_bootstrap.rs` line audit

Strict line-by-line walk of the Perl bootstrap pool against the Rust
sibling. The Perl source is 45 lines; this is a single-phase audit.

**Status legend**:
* ✅ PARITY — Perl entry has Rust counterpart in expected location.
* ↻ ORDER — entry exists in Rust but in a different sibling file.
* 📁 FILE — entry placed correctly relative to file structure.
* ⚠ DIVERGE — entry differs in semantics or shape.
* ❌ MISSING — Perl entry has no Rust counterpart.
* 🔵 RUST_ONLY — Rust entry without Perl source.

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| (n/a) | INITEX lccode/uccode/sfcode loop for letters | plain_bootstrap.rs:35-42 | 🔵 RUST_ONLY (documented) |
| 19-27 | `\TeX` Constructor (CSS-styled T<e>X) | plain_bootstrap.rs:45-50 | ✅ |
| 32 | `\alloc@{}{}{}{}{}` (locked) | plain_bootstrap.rs:55 | ✅ |
| 33 | `\ch@ck{}{}{}` (locked) | plain_bootstrap.rs:56 | ✅ |
| 37-40 | `\newif DefToken` DefPrimitive (locked) | plain_bootstrap.rs:63-65 | ✅ |
| 43 | `\leavevmode` DefPrimitive (enterHorizontal) | plain_bootstrap.rs:69 | ✅ |

## Findings

* **Strong PARITY** for all 5 substantive Perl entries. Direct
  semantic translations — `\TeX` Constructor body matches, `\alloc@`
  / `\ch@ck` token-list bodies preserved, `\newif` calls
  `def_conditional` matching Perl's `DefConditionalI`, `\leavevmode`
  enters horizontal mode.
* **🔵 Rust-only INITEX init**: The lccode/uccode/sfcode loop for
  letters at plain_bootstrap.rs L35-42 is a Rust-only addition.
  Documented in-place (L10-34): mirrors plain.tex L112-113 INITEX
  setup that is missing from the dump-build snapshot. Without this,
  `\MakeUppercase` produced lowercase output under the dump path.
  Justified divergence — keep.
* **No misplacements** — all Perl entries have correctly-placed
  Rust counterparts.
* **Order is preserved**: file structure mirrors Perl's section
  comments exactly.

## Cumulative parity health

`plain_bootstrap.pool.ltxml` is **fully parity** with one
well-documented Rust-only addition. The bootstrap layer is the
cleanest of the audited files.
