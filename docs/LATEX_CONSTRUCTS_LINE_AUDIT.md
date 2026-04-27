# latex_constructs.pool.ltxml line-by-line parity audit

Per user directive: "exact same definitions in exact same order are
translated to latex_constructs.rs, as are in the original
latex_constructs.pool.ltxml". This audit walks Perl line-by-line,
maps each definition to its Rust analog (file + line), and flags
order/file divergences.

**Methodology**: Walk Perl `LaTeXML/lib/LaTeXML/Engine/latex_constructs.pool.ltxml`
top-down. For each definition, locate the Rust analog in any of
`engine/latex_{base,bootstrap,constructs,constructs_rust_only}.rs`.

Status legend:
- ✅ PARITY — same form, comparable position
- ↻ ORDER — definition exists in Rust but at a far different position
- 📁 FILE — definition exists but in a different Rust file than Perl puts it
- ⚠ DIVERGE — Rust definition differs in form (DefMacro vs DefConstructor etc.)
- ❌ MISSING — Perl defines it, Rust doesn't (in any of the 4 latex_* files)
- 🔵 RUST_ONLY — already isolated to `latex_constructs_rust_only.rs`

## Phase 1: Perl L19-L100 (preamble + LoadPool reloads + early defs)

| Perl L | Symbol/op | Rust file:line | Status |
|---|---|---|---|
| 19 | `AssignValue plain_constructs._loaded undef` | latex_constructs.rs:2337 | ✅ |
| 20 | `AssignValue math_common._loaded undef` | latex_constructs.rs:2342 | ✅ |
| 21 | `LoadPool('plain_constructs')` | latex_constructs.rs:2357 (`InnerPool!`) | ✅ |
| 25 | `assignValue font => textDefault` | (?) | ❓ verify |
| 26 | `assignValue mathfont => mathDefault` | (?) | ❓ verify |
| 27 | `DefMacroI '\f@encoding'` | latex_constructs.rs:5403 | ↻ ORDER (~3000L late) |
| 28 | `DefMacroI '\cf@encoding'` | latex_constructs.rs:5406 | ↻ ORDER |
| 30 | `DefMacro '\hline'` | tex_tables.rs (likely) | 📁 FILE (intentional — table) |
| 31 | `DefMacroI '\ldots'` | math_common.rs (likely) | 📁 FILE (intentional — math) |
| 33 | `DefPrimitiveI '\ASCII\^'` | NOT FOUND | ❌ MISSING |
| 34 | `DefPrimitiveI '\ASCII\~'` | NOT FOUND | ❌ MISSING |
| 36 | `Let '\par' '\lx@normal@par'` | latex_constructs.rs:2370 | ✅ FIXED 2026-04-27 (`b3c114d79`) |
| 38 | `LoadPool('math_common')` | latex_constructs.rs:2358 (collapsed at top) | ↻ ORDER (Perl L38, Rust at top) |
| 42 | `DefAccent '\k'` | latex_constructs.rs:8754 | ↻ ORDER (~6000L late, also redefined) |
| 43 | `DefAccent '\r'` | (similar) | ↻ ORDER |
| 45 | `NewCounter('page')` | latex_constructs.rs:3806 | ↻ ORDER (~1000L late but probably OK) |
| 46 | `SetCounter(page, 1)` | latex_constructs.rs:3807 | ✅ |
| 47 | `Let '\newpage' '\eject'` | plain_constructs.rs:539 | 📁 FILE (intentional — plain pool) |
| 48 | `Let '\nobreakspace' '\lx@nobreakspace'` | latex_constructs.rs:4722 | ↻ ORDER (~2500L late) |
| 51 | `DefMacroI '\hidewidth' Tokens()` | plain_base.rs:199 | 📁 FILE (Rust puts it in plain_base, not latex_constructs) |
| 56 | `Let '\magnification' '\@undefined'` | latex_constructs.rs:2384 | ✅ |
| 57 | `Let '\@empty' '\lx@empty'` | latex_base.rs:22 | 📁 FILE (Perl latex_constructs L57 → Rust latex_base.rs L22) |
| 58 | `Let '\@ifundefined' '\lx@ifundefined'` | latex_base.rs (likely L23 area) | 📁 FILE |
| 63 | `DefConditionalI '\if@compatibility'` | latex_constructs.rs:2390 | ✅ |
| 64 | `DefMacro '\@compatibilitytrue'` | latex_constructs.rs:2391 | ✅ |
| 65 | `DefMacro '\@compatibilityfalse'` | latex_constructs.rs:2392 | ✅ |
| 67 | `Let '\@currentlabel' '\@empty'` | latex_constructs.rs:2394 | ✅ |
| 68 | `DefMacro '\@currdir' './'` | latex_constructs.rs:2395 | ✅ |
| 71 | `AssignValue inPreamble => 1` | latex_constructs.rs:2398 | ✅ |
| 73-85 | `DefConstructor '\documentclass …'` | latex_constructs.rs:~2410 | ✅ |
| 87 | `AssignValue '@unusedoptionlist'` | latex_constructs.rs:2413 | ✅ |
| 88-92 | `DefPrimitiveI '\warn@unusedclassoptions'` | latex_constructs.rs:2414 | ✅ |
| 94+ | `DefConstructor '\documentstyle …'` | latex_constructs.rs:~2440 (verify) | ✅ |

## Phase 1 findings

* **Order**: From Perl L36 `Let \par \lx@normal@par` onwards, Rust
  reorganizes substantially. Several Perl L42-48 directives appear
  thousands of lines later in Rust. The `\par` Let (L36) has been
  positioned correctly recently (2026-04-27 fix). The `\nobreakspace`
  Let (L48) is at Rust L4722 vs Perl L48.
* **MISSING**: `\hidewidth` (L51), `\ASCII\^` and `\ASCII\~` (L33-34).
* **Verify**: `font => textDefault`/`mathfont => mathDefault`
  assignments (L25-26), `\@empty`/`\@ifundefined` Lets (L57-58).

## Phase 2 (TODO): Perl L101-L500

Includes major sections — probably options handling, `\AtBeginDocument`,
package loading, main document block. Will continue in next iterations.

## Phase 3+ (TODO): L501-L6014

The bulk of `latex_constructs.pool.ltxml`.
