# Latex Constructs Translation Order Audit

## Headline numbers

* Perl `LaTeXML/lib/LaTeXML/Engine/latex_constructs.pool.ltxml` — **6014 lines**
* Rust `latexml_package/src/engine/latex_constructs.rs` — **9296 lines**

Rust is **54% larger** than Perl. The user has flagged this as bloat. Some
of it is legitimate (Rust's `DefMacro!` macro + `sub[...]` helpers expand
to more characters per definition than Perl's terse `DefMacro` calls), but
the gap is bigger than the per-line factor would predict. There IS extra
content.

## Order divergences found in the first 50 Perl lines

Compared Perl L19-L48 (the file's preamble + LoadPool reloads) against
Rust:

| Perl L | Symbol/op | Rust position | Issue |
|---|---|---|---|
| 19 | `AssignValue plain_constructs._loaded undef` | rs:2337 | OK (state::assign_value) |
| 20 | `AssignValue math_common._loaded undef` | rs:2342 | OK |
| 21 | `LoadPool('plain_constructs')` | rs:2357 | OK (InnerPool!) |
| 23 | `assignValue font => textDefault` | NOT FOUND (?) | Possibly missing |
| 24 | `assignValue mathfont => mathDefault` | NOT FOUND (?) | Possibly missing |
| 27 | `DefMacroI \f@encoding` | rs:5431 (~3000L LATER) | OUT OF ORDER |
| 28 | `DefMacroI \cf@encoding` | rs:5434 | OUT OF ORDER |
| 30 | `DefMacro \hline` | tex_tables.rs:418 | WRONG FILE |
| 31 | `DefMacroI \ldots` | math_common.rs:814 | WRONG FILE |
| 33 | `DefPrimitiveI \ASCII\^` | NOT FOUND | MISSING |
| 34 | `DefPrimitiveI \ASCII\~` | NOT FOUND | MISSING |
| 36 | `Let \par \lx@normal@par` | rs:2370 (just added) | **FIXED** by b3c114d79 |
| 38 | `LoadPool('math_common')` | rs:2358 (collapsed earlier) | OUT OF ORDER (Perl runs this AFTER L36 Let; Rust runs both InnerPools at top) |
| 41 | `DefAccent \k` | rs:8754 (~6000L LATER) | OUT OF ORDER (also redefined in t1enc_def, t5enc_def, t1enc_sty, cp852_def) |
| 42 | `DefAccent \r` | likely similar | OUT OF ORDER |
| 44 | `NewCounter('page')` | rs:3806 | LATER but probably OK |
| 45 | `SetCounter(page,1)` | (need to find) | TBD |
| 46 | `Let \newpage \eject` | NOT FOUND (?) | Possibly missing |
| 47 | `Let \nobreakspace \lx@nobreakspace` | rs:4722 | LATER (~2500L) |

## Why the order matters (per user directive)

When the order is wrong:
* Lookups during dump-load phases see stale or default bindings.
* Re-Lets that depend on prior `Let` order silently no-op.
* The recently-fixed `Let \par \lx@normal@par` is the textbook example —
  Rust was missing it entirely, and box_test (and 50+ other tests) fail
  silently because document-body `\par` resolves to the chain instead
  of the Constructor.

## Plan

This audit needs to be pursued line-by-line through all 6014 Perl lines.
That's at least a few iteration cycles of careful work. Approach:

1. **Phase 1 (immediate)**: catalog Perl L1-L500 line-by-line, identify
   each Rust analog (its file + line + form), flag DIVERGENCE/MISSING.
   Fix any high-leverage missing/wrong-order ones (like the `\par` one).
2. **Phase 2**: continue through L501-L3000.
3. **Phase 3**: L3001-L6014.
4. **Phase 4**: identify Rust content that DOESN'T appear in Perl —
   either move to a more appropriate file, delete (if dead), or document
   as intentional divergence.

## Bloat hypotheses (Rust > Perl by 3300 lines)

Without the audit, candidates include:
* Long Rust comments documenting Perl line numbers (legitimate but
  voluminous).
* Helper functions inlined that Perl pulls from `LaTeXML::Package` lib.
* Possibly stale code from earlier iterations that's been superseded.
* DOM-walker helpers that need real Rust code where Perl uses Perl's
  XML libs at runtime.

Will be confirmed during the audit.
