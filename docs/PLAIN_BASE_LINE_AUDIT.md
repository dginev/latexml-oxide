# `plain_base.pool.ltxml` ↔ `plain_base.rs` line audit

Strict line-by-line walk of the 622-line Perl `plain_base.pool.ltxml`
against `plain_base.rs` (751 lines).

**Status legend**:
* ✅ PARITY — Perl entry has Rust counterpart in expected location.
* ↻ ORDER — entry exists in Rust but in a different sibling file.
* 📁 FILE — entry placed correctly relative to file structure.
* ⚠ DIVERGE — entry differs in semantics or shape.
* ❌ MISSING — Perl entry has no Rust counterpart.
* 🔵 RUST_ONLY — Rust entry without Perl source.

## Phase 1 — Perl L1-200 (Plain TeX, Special Chars, Alignment, Appendix B p.344-347)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 25 | `\magnification` Number(1000) (DefRegister) | plain_base.rs:43 | ✅ |
| 29 | `\hideoutput` (DefMacroI Tokens()) | plain_base.rs:55 | ✅ |
| 30-45 | `\loggingall` BIG body (`\tracingstats\tw@ \tracingpages\@ne ...`) | plain_base.rs (omitted; latex_base.rs:557 redefines as empty) | ⚠ DIVERGE shape (Perl-effective behavior matches via override) |
| 46 | `\tracingall` (`\showoverfull\loggingall`) | plain_base.rs:48-53 (different body) | ⚠ DIVERGE shape |
| 47-65 | `\tracingnone` BIG body | plain_base.rs:54 (empty None) | ⚠ DIVERGE shape |
| 70-71 | `\#` (DefPrimitive Box) | plain_base.rs:81 (DefMacro `\ifmmode\lx@math@hash\else\lx@text@hash\fi` + DefPrimitive `\lx@text@hash` + DefMath `\lx@math@hash`) | ⚠ DIVERGE shape (intentional WISDOM #44) |
| 72-73 | `\&` (DefPrimitive Box, ADDOP/and) | plain_base.rs:82,93,99 (trio split) | ⚠ DIVERGE shape (intentional) |
| 74-75 | `\%` (DefPrimitive Box, POSTFIX/percent) | plain_base.rs:84,94,100 | ⚠ DIVERGE shape (intentional) |
| 76-77 | `\$` (DefPrimitive Box, OPERATOR/currency-dollar) | plain_base.rs:87,95,101 | ⚠ DIVERGE shape (intentional) |
| (n/a) | `\_` underscore (Perl: TeX.pool.ltxml) | plain_base.rs:88-91,96,103 | 🔵 (Rust adds for parity with `\#`/`\&` family) |
| 80 | `\*` (DefMathI INVISIBLE TIMES) | plain_base.rs:107 | ✅ |
| 86-106 | DefMathRewrite XMWrap→XMTok concat | plain_base.rs:112-140 | ✅ |
| 108 | `\i` (DefPrimitiveI dotless-i, robust) | plain_base.rs:147 | ✅ |
| 109 | `\j` (DefPrimitiveI dotless-j, robust) | plain_base.rs:148 | ✅ |
| 118 | `Let \ialign \halign` | plain_base.rs:157 | ✅ |
| 121-122 | `\oalign{}` (DefMacro) | plain_base.rs:160-163 | ✅ |
| 123-127 | `\@@oalign{}` (DefConstructor, alignmentBindings('l')) | plain_base.rs:164-172 | ✅ |
| 131-132 | `\ooalign{}` (DefMacro) | plain_base.rs:175-178 | ✅ |
| 133-137 | `\@@ooalign{}` (DefConstructor) | plain_base.rs:179-187 | ✅ |
| 139-145 | `\buildrel Until:\over {}` (DefConstructor) | plain_base.rs:189-198 | ✅ |
| 147 | `\hidewidth` (DefMacroI Tokens()) | latex_constructs.rs (rust comment notes intentional move at plain_base.rs:199) | ↻ MISPLACED |
| 152 | RawTeX `\outer\def^^L{\par}` | plain_base.rs:206 | ✅ |
| 153 | `\dospecials` (DefMacro) | plain_base.rs:207-210 | ✅ |
| 158-169 | chardef block (`\active`, `\@ne`, `\tw@`, `\thr@@`, `\sixt@@n`, `\@cclv`, `\@cclvi`, `\@m`, `\@M`, `\@MM`) | plain_base.rs:214-224 (TeX!) | ✅ |
| (n/a) | mathchardef `\cdotp`/`\ldotp`/`\intop`/`\ointop` | plain_base.rs:225-228 | 🔵 RUST_ONLY (Perl has them at L5487-5492 of `latex_constructs.pool.ltxml` — moved up to plain_base for early availability) |
| 174-195 | register allocations RawTeX block | plain_base.rs:234-255 | ✅ |

### Phase 1 findings

* **Predominantly PARITY** with several documented intentional shape
  divergences:
  * `\tracingall`/`\loggingall`/`\tracingnone` — Rust has simpler
    bodies than Perl. Effective behavior matches because Perl
    `latex_base.pool.ltxml` redefines them later. Could be made
    Perl-faithful at a later cleanup pass.
  * `\#`/`\&`/`\%`/`\$`/`\_` — Rust splits each into a
    DefMacro+DefPrimitive+DefMath trio (math/text dispatch).
    Documented as intentional WISDOM #44 idiom; Perl uses
    Box-auto-XMTok-promotion which has no direct Rust API.
* **↻ MISPLACED**: `\hidewidth` (Perl L147) lives in
  `latex_constructs.rs` per Rust comment. Per strict-parity rule,
  should be in `plain_base.rs`.
* **🔵 Rust-only consolidation**: mathchardef `\cdotp`/`\ldotp`/
  `\intop`/`\ointop` defined in plain_base.rs:225-228 but Perl
  defines them later in `latex_constructs.pool.ltxml:5487-5492`.
  Need to verify if Rust-side has duplicate in latex_constructs.rs.

## Phase 2 — Perl L200-400 (Appendix B p.347-352)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 200-203 | `\wlog{}` (DefPrimitive, locked) | plain_base.rs:259-262 | ✅ |
| 207-218 | `\new*` (count/dimen/skip/muskip/box/help/toks/read/write/fam/language) RawTeX | plain_base.rs:266-278 (TeX!) | ✅ |
| 222 | `\newinsert Token` (DefPrimitive closure) | plain_base.rs:283-285 | ✅ |
| 226 | `\maxdimen` Dimension(16383.99999*UNITY) | plain_base.rs:291 | ✅ |
| 227 | `\hideskip` Glue('-1000pt plus 1fill') | plain_base.rs:292 | ✅ |
| 228 | `\centering` Glue('0pt plus 1000pt minus 1000pt') | plain_base.rs:293 | ✅ |
| 229 | `\p@` Dimension(UNITY) | plain_base.rs:294 | ✅ |
| 230 | `\z@` Dimension(0) | plain_base.rs:295 | ✅ |
| 231 | `\z@skip` Glue(0,0,0) | plain_base.rs:296 | ✅ |
| 234 | `\@` (DefConstructor empty) | plain_base.rs:299 | ✅ |
| 237 | RawTeX `\newbox\voidb@x` | plain_base.rs:302 | ✅ |
| 244 | `\smallskipamount` | plain_base.rs:315 | ✅ |
| 245 | `\medskipamount` | plain_base.rs:316 | ✅ |
| 246 | `\bigskipamount` | plain_base.rs:317 | ✅ |
| 247 | `\normalbaselineskip` | plain_base.rs:318 | ✅ |
| 248 | `\normallineskip` | plain_base.rs:319 | ✅ |
| 249 | `\normallineskiplimit` | plain_base.rs:320 | ✅ |
| 250 | `\jot` | plain_base.rs:321 | ✅ |
| 251 | `\lx@default@jot` (LookupRegister `\jot`) | plain_base.rs:325 | ✅ |
| 252 | `\interdisplaylinepenalty` Number(100) | plain_base.rs:326 | ✅ |
| 253 | `\interfootnotelinepenalty` Number(100) | plain_base.rs:327 | ✅ |
| 255 | `\magstephalf` "1095" | plain_base.rs:329 | ✅ |
| 257-261 | `\magstep{}` (closure) | plain_base.rs:330-340 | ✅ |
| 267-302 | RawTeX font setup `\font\tenrm=cmr10` etc. + `\textfont*=*`, `\newfam*` | plain_base.rs:346-372 | ✅ |
| 303-360 | RawTeX mathcodes block (`\mathcode\^^@` through `\mathcode\^^?`, plus ASCII chars `*`/`+`/`,`/`-`/etc.) | math_common.rs:425+ (assign_mathcode calls) | ↻ MISPLACED |
| 367 | `AssignValue NOMINAL_FONT_SIZE => 10` | plain_base.rs:385 | ✅ |
| 369-374 | `\mit` (DefPrimitiveI declarative + closure shadowed) | plain_base.rs:397-401 (closure with require_math) | ⚠ shape (Rust merges Perl's two forms) |
| 376 | `\frenchspacing` (DefPrimitiveI) | plain_base.rs:403 | ✅ |
| 377 | `\nonfrenchspacing` | plain_base.rs:404 | ✅ |
| 378-379 | `\normalbaselines` | plain_base.rs:405-408 | ✅ |
| 380 | `\space` Tokens(T_SPACE) | plain_base.rs:409 | ✅ |
| 381 | `\lq` "`" | plain_base.rs:410 | ✅ |
| 382 | `\rq` "'" | plain_base.rs:411 | ✅ |
| 383 | `Let \empty \lx@empty` | plain_base.rs:412 | ✅ |
| 384 | `\null` `\hbox{}` | plain_base.rs:413 | ✅ |
| 385 | `Let \bgroup T_BEGIN` | plain_base.rs:414 | ✅ |
| 386 | `Let \egroup T_END` | plain_base.rs:415 | ✅ |
| 387 | `Let \endgraf \par` | plain_base.rs:416 | ✅ |
| 388 | `Let \endline \cr` | plain_base.rs:417 | ✅ |
| 390 | `\endline` (DefPrimitiveI undef) | plain_base.rs:419 | ✅ |
| 393 | `\\\r` `\<cr>==\<space>` | plain_base.rs:422 | ✅ |
| 394 | `Let T_ACTIVE("\r") \par` | plain_base.rs:423 | ✅ |
| 396 | `Let \\\t \\\r` | plain_base.rs:425 | ✅ |

### Phase 2 findings

* **Predominantly PARITY** for L200-400. ~30 entries match Rust
  in source order at plain_base.rs:259-425.
* **↻ MISPLACED**: Plain TeX mathcodes block (Perl L303-360) lives
  in `math_common.rs:425+` in Rust, not `plain_base.rs`. Per
  strict-parity rule, should be in plain_base.rs. Need to verify
  that `math_common.pool.ltxml` doesn't ALSO define them — if
  shared between Perl files, keep math_common.rs; otherwise migrate.
* **⚠ shape**: `\mit` — Rust merges Perl's two forms (declarative
  L369 + closure L371-374) into a single closure with
  `require_math => true`. Functionally equivalent.

## Phase 3+ (TODO)

* Phase 3: Perl L400-622 (final blocks)
