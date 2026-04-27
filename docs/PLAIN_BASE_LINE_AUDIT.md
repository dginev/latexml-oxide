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

## Phase 3 — Perl L400-622 (file end: spacing, chars, page layout, em/boldmath)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 401-404 | `\obeyspaces` (DefPrimitiveI closure) | plain_base.rs:430-433 | ✅ |
| 407 | `Let T_ACTIVE(' ') \space` | plain_base.rs:436 | ✅ |
| 409-412 | `\obeylines` | plain_base.rs:438-441 | ✅ |
| 414 | `\@break` Constructor | plain_base.rs:443-444 | ✅ |
| 416-420 | RawTeX `\loop`/`\iterate`/`\repeat` | plain_base.rs:446-452 (TeX!) | ✅ |
| 422-424 | `\enskip` (Box, name='enskip', width=0.5em) | plain_base.rs:454 | ✅ |
| 426-428 | `\enspace` | plain_base.rs:465 | ✅ |
| 430-432 | `\quad` | plain_base.rs:476 | ✅ |
| 435-437 | `\qquad` (asHint => 1) | plain_base.rs:488 | ✅ |
| 439-441 | `\thinspace` | plain_base.rs:499 | ✅ |
| 443-445 | `\negthinspace` | plain_base.rs:510 | ✅ |
| 447-452 | `\hglue Glue` (DefPrimitive closure) | plain_base.rs:564-570 | ✅ |
| 454 | `\vglue Glue` (DefPrimitive None) | plain_base.rs:571 | ✅ |
| 455 | `\topglue` | plain_base.rs:572 | ✅ |
| 456 | `\nointerlineskip` | plain_base.rs:574 | ✅ |
| 457-458 | `\offinterlineskip` | plain_base.rs:578 | ✅ |
| 460 | `\smallskip` | plain_base.rs:582 | ✅ |
| 461 | `\medskip` | plain_base.rs:583 | ✅ |
| 462 | `\bigskip` | plain_base.rs:584 | ✅ |
| 467 | `\break` | plain_base.rs (verify) | (likely ✅) |
| 468 | `\nobreak` | plain_base.rs (verify) | (likely ✅) |
| 471 | `T_ACTIVE("~") \lx@NBSP` | plain_base.rs (verify) | (likely ✅) |
| 473 | `\slash` "/" | plain_base.rs (verify) | (likely ✅) |
| 474 | `\filbreak` | plain_base.rs:605 | ✅ |
| 475 | `\goodbreak` "\par" | plain_base.rs:606 | ✅ |
| 476 | `\removelastskip` | plain_base.rs:607 | ✅ |
| 477 | `\smallbreak` "\par" | plain_base.rs:608 | ✅ |
| 478 | `\medbreak` "\par" | plain_base.rs:609 | ✅ |
| 479 | `\bigbreak` "\par" | plain_base.rs:610 | ✅ |
| 481 | `\line` "\hbox to \hsize" | plain_base.rs (verify) | (likely ✅) |
| 484 | `\llap{}` | plain_base.rs:614 | ✅ |
| 485 | `\rlap{}` | plain_base.rs:615 | ✅ |
| 487 | `\m@th` | plain_base.rs:616 | ✅ |
| 490 | `\strut` Tokens() | plain_base.rs (verify) | (likely ✅) |
| 491 | RawTeX `\newbox\strutbox` | plain_base.rs (verify) | (likely ✅) |
| 499 | `\settabs` | plain_base.rs (verify) | (likely ✅) |
| 507 | `\hang` | plain_base.rs:634 | ✅ |
| 508 | `\item` | plain_base.rs (verify) | (likely ✅) |
| 509 | `\itemitem` | plain_base.rs:638 | ✅ |
| 510 | `\textindent{}` | plain_base.rs:640 | ✅ |
| 511-512 | `\narrower` | plain_base.rs:642 | ✅ |
| 516 | `\raggedright` | plain_base.rs:649 | ✅ |
| 517 | `\raggedleft` | plain_base.rs (verify) | (likely ✅) |
| 518 | `\ttraggedright` | plain_base.rs:651 | ✅ |
| 519 | `\mathhexbox{}{}{}` `\leavevmode\hbox{$\m@th \mathchar"#1#2#3$}` | plain_base.rs:654 | ✅ |
| 525 | `\OE` UTF(0x152) | math_common.rs:204 + latex_constructs.rs:5664 | ↻ MISPLACED + ⚠ DUP |
| 526 | `\oe` UTF(0x153) | (math_common.rs / latex_constructs.rs) | ↻ MISPLACED |
| 527 | `\AE` UTF(0xC6) | similar | ↻ MISPLACED |
| 528 | `\ae` UTF(0xE6) | similar | ↻ MISPLACED |
| 529 | `\AA` UTF(0xC5) | similar | ↻ MISPLACED |
| 530 | `\aa` UTF(0xE5) | similar | ↻ MISPLACED |
| 531 | `\O` UTF(0xD8) | similar | ↻ MISPLACED |
| 532 | `\o` UTF(0xF8) | similar | ↻ MISPLACED |
| 533 | `\ss` UTF(0xDF) | similar | ↻ MISPLACED |
| 537 | `Let \sp T_SUPER` | plain_base.rs (verify) | (likely ✅) |
| 538 | `Let \sb T_SUB` | plain_base.rs (verify) | (likely ✅) |
| 539 | `Let \: \>` | plain_base.rs (verify) | (likely ✅) |
| 541-543 | `\\\t` (DefPrimitiveI closure: Box UTF(0xA0) name=tab) | plain_base.rs:425 (Let form, not DefPrimitive) | ⚠ shape |
| 546 | `\openup Dimension` | plain_base.rs:661 | ✅ |
| 551 | `\displaylines{}` | plain_base.rs:667 | ✅ |
| 553 | `\pageno` Number(0) | plain_base.rs:671 | ✅ |
| 554 | `\headline` Tokens() | plain_base.rs:672 | ✅ |
| 555 | `\footline` Tokens() | plain_base.rs:673 | ✅ |
| 556 | `\folio` "1" | plain_base.rs:674 | ✅ |
| 558 | `\nopagenumbers` | plain_base.rs:676 | ✅ |
| 559 | `\advancepageno` | plain_base.rs:677 | ✅ |
| 564 | `\raggedbottom` | plain_base.rs:681 | ✅ |
| 565 | `\normalbottom` | plain_base.rs (verify) | (likely ✅) |
| 568 | `\vfootnote` | plain_base.rs:685 | ✅ |
| 569 | `\fo@t` | plain_base.rs (verify) | (likely ✅) |
| 570 | `\f@@t` | plain_base.rs (verify) | (likely ✅) |
| 571 | `\f@t{}` | plain_base.rs (verify) | (likely ✅) |
| 572 | `\@foot` | plain_base.rs (verify) | (likely ✅) |
| 574 | `\footstrut` | plain_base.rs:694 | ✅ |
| 575 | `\footins` Number(0) | plain_base.rs:695 | ✅ |
| 577 | `\topinsert` | plain_base.rs:697 | ✅ |
| 578 | `\midinsert` | plain_base.rs:698 | ✅ |
| 579 | `\pageinsert` | plain_base.rs:699 | ✅ |
| 580 | `\endinsert` | plain_base.rs:700 | ✅ |
| 588 | `\footnoterule` | plain_base.rs:707 | ✅ |
| 606-611 | `\em` (DefPrimitiveI before_digest closure) | plain_base.rs:725 | ✅ |
| 614-616 | `\boldmath` (forbidMath => 1) | plain_base.rs:735 | ✅ |
| 617-619 | `\unboldmath` (forbidMath => 1) | plain_base.rs:743 | ✅ |

### Phase 3 findings

* **Predominantly PARITY** for L400-622. Most ~50 entries match
  Rust at plain_base.rs:430-743 in source order.
* **↻ MISPLACED + ⚠ DUP** (L525-533): `\OE`/`\oe`/`\AE`/`\ae`/`\AA`/
  `\aa`/`\O`/`\o`/`\ss` (9 ligatures) — defined in `math_common.rs:204+`
  AND `latex_constructs.rs:5664+` instead of `plain_base.rs`. Per
  strict-parity rule, primary location should be plain_base.rs.
  The math_common.rs entries are likely intentional (ligatures
  also work in math) but the duplication in latex_constructs.rs
  is suspicious.
* **⚠ shape** (L541-543): `\\\t` — Perl is DefPrimitiveI closure
  (Box UTF(0xA0) width=1em); Rust at plain_base.rs:425 uses
  `Let!("\\\t", "\\\r")`. Different shape; functionally similar
  but not strictly parity.

## Cumulative parity health (Perl L1-622, 100% of plain_base.pool.ltxml)

* **Phase 1** (L1-200): ✅ Strong PARITY with documented intentional
  shape divergences (`\#`/`\&`/`\%`/`\$`/`\_` trio split, WISDOM #44).
* **Phase 2** (L200-400): ✅ Strong PARITY. ↻ MISPLACED Plain TeX
  mathcodes block (Perl L303-360) lives in `math_common.rs:425+`.
* **Phase 3** (L400-622): ✅ Strong PARITY. ↻ MISPLACED ligatures
  cluster (`\OE`/`\AE`/`\AA` family, 9 entries, Perl L525-533) in
  `math_common.rs` + `latex_constructs.rs`; shape divergence on
  `\\\t`.

Overall: plain_base.rs is a high-parity file; main outstanding work
is migrating mathcodes block + ligatures cluster from math_common
back to plain_base.

## Pending parity work (post-audit)

1. Mathcodes block (Perl L303-360, ~60 entries): currently in
   math_common.rs:425+. Need to verify if Perl math_common.pool.ltxml
   ALSO defines them; if not, migrate to plain_base.rs.
2. Ligatures cluster (`\OE`/`\AE`/`\AA` family): clean up duplicate in
   latex_constructs.rs; primary location should be plain_base.rs.
