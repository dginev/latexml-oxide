# `latex_base.pool.ltxml` ↔ `latex_base.rs` line audit

Strict line-by-line walk of the 865-line Perl `latex_base.pool.ltxml`
against `latex_base.rs`. Goal: confirm every Perl entry is in the
matching Rust file, in the same source order, with the same shape.

**Status legend**:
* ✅ PARITY — Perl entry has Rust counterpart in expected location.
* ↻ ORDER — entry exists in Rust but in a different sibling file.
* 📁 FILE — entry placed correctly relative to file structure.
* ⚠ DIVERGE — entry differs in semantics or shape.
* ❌ MISSING — Perl entry has no Rust counterpart.
* 🔵 RUST_ONLY — Rust entry without Perl source.

## Phase 1 — Perl L1-150 (C.0 Preliminaries & Shorthands)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 35 | `Let '\@pushfilename' '\lx@pushfilename'` | latex_base.rs:58 | ✅ |
| 36 | `Let '\@popfilename' '\lx@popfilename'` | latex_base.rs:59 | ✅ |
| 38 | `\@ehc` "I can't help" | latex_base.rs:62 | ✅ |
| 40 | `\@gobble{}` (Tokens()) | latex_base.rs:65 | ✅ |
| 41 | `\@gobbletwo{}{}` | latex_base.rs:66 | ✅ |
| 42 | `\@gobblefour{}{}{}{}` | latex_base.rs:67 | ✅ |
| 43 | `\@firstofone{}` | latex_base.rs:73 (token-list "#1" form, see comment L68-72) | ✅ ⚠ shape |
| 44 | `Let '\@iden' '\@firstofone'` | latex_base.rs:74 | ✅ |
| 45 | `\@firstoftwo{}{}` | latex_base.rs:75 | ✅ ⚠ shape |
| 46 | `\@secondoftwo{}{}` | latex_base.rs:76 | ✅ ⚠ shape |
| 47 | `\@thirdofthree{}{}{}` | latex_base.rs:77 | ✅ ⚠ shape |
| 48-49 | `\@expandtwoargs{}{}{}` (closure) | latex_base.rs:82-90 | ✅ |
| 50-52 | `\@makeother{}` (closure) | latex_base.rs:93-104 | ✅ |
| 55-64 | RawTeX block: `\@namedef`/`\@nameuse`/`\@cons`/`\@car`/`\@cdr`/`\@carcube`/`\nfss@text`/`\@sect` | latex_base.rs:25-37 (TeX!) | ✅ ↻ position |
| 66-72 | RawTeX: `\obeycr`/`\@gobblecr`/`\restorecr` | latex_base.rs:107-113 (TeX!) | ✅ |
| 73-90 | RawTeX: `\rem@pt`/`\strip@pt`/`\strip@prefix`/`\@sanitize`/`\@onelevel@sanitize`/`\dospecials` | latex_base.rs:115-133 (TeX!) | ✅ |
| 92-114 | `\nfss@catcodes` | latex_base.rs:135-160 | ✅ |
| 116 | `\@height` ("height") | latex_base.rs:163 | ✅ |
| 117 | `\@width` ("width") | latex_base.rs:164 | ✅ |
| 118 | `\@depth` ("depth") | latex_base.rs:165 | ✅ |
| 119 | `\@minus` ("minus") | latex_base.rs:166 | ✅ |
| 120 | `\@plus` ("plus") | latex_base.rs:167 | ✅ |
| 121 | `\hb@xt@` ("\hbox to") | latex_base.rs:168 | ✅ |
| 122 | `\hmode@bgroup` ("\leavevmode\bgroup") | latex_base.rs:169 | ✅ |
| 124 | `\@backslashchar` (T_OTHER('\\')) | latex_base.rs:171 | ✅ |
| 125 | `\@percentchar` (T_OTHER('%')) | latex_base.rs:172 | ✅ |
| 126 | `\@charlb` (T_LETTER('{')) | latex_base.rs:173 | ✅ |
| 127 | `\@charrb` (T_LETTER('}')) | latex_base.rs:174 | ✅ |
| 129 | `\@vpt` (T_OTHER('5')) | latex_base.rs:177 | ✅ |
| 130 | `\@vipt` (T_OTHER('6')) | latex_base.rs:178 | ✅ |
| 131 | `\@viipt` (T_OTHER('7')) | latex_base.rs:179 | ✅ |
| 132 | `\@viiipt` (T_OTHER('8')) | latex_base.rs:180 | ✅ |
| 133 | `\@ixpt` (T_OTHER('9')) | latex_base.rs:181 | ✅ |
| 134 | `\@xpt` ("10") | latex_base.rs:182 | ✅ |
| 135 | `\@xipt` ("10.95") | latex_base.rs:183 | ✅ |
| 136 | `\@xiipt` ("12") | latex_base.rs:184 | ✅ |
| 137 | `\@xivpt` ("14.4") | latex_base.rs:185 | ✅ |
| 138 | `\@xviipt` ("17.28") | latex_base.rs:186 | ✅ |
| 139 | `\@xxpt` ("20.74") | latex_base.rs:187 | ✅ |
| 140 | `\@xxvpt` ("24.88") | latex_base.rs:188 | ✅ |
| 142-153 | `\vpt`/`\vipt`/`\viipt`/`\viiipt`/`\ixpt`/`\xpt`/`\xipt`/`\xiipt`/`\xivpt`/`\xviipt`/`\xxpt`/`\xxvpt` (LaTeX 209 size aliases) | latex_base.rs:190-201 | ✅ |

### Phase 1 findings

* **Strong PARITY**. All Perl L31-153 entries have Rust counterparts
  in proper source order at latex_base.rs L57-201.
* **⚠ shape** divergence on `\@firstofone`/`\@firstoftwo`/`\@secondoftwo`/
  `\@thirdofthree`: Rust uses token-list form `"#1"` etc. instead of
  Perl's closure `sub { $_[1] }`. Documented in latex_base.rs:68-72:
  matches Perl latex.ltx's `\long\def\@firstofone#1{#1}` end-state
  (via raw-load), AND lets these CSes survive dump-only mode dump
  loading. Validated as intentional.
* **🔵 Rust-only entry**: `Let!("\\@empty", "\\lx@empty")` at
  latex_base.rs:22 is not in Perl latex_base.pool.ltxml directly —
  the alias is for `\lx@empty` from Base_Schema (TeX pool). `\@empty`
  is also defined via raw-load of latex.ltx in Perl. Functionally
  equivalent.
* **↻ position**: The Perl L55-64 RawTeX block (with `\@namedef` etc.)
  is at latex_base.rs L25-37 — placed BEFORE the L40-49 macro block
  (Rust L65+). Perl has them after L40-52. This is a minor ordering
  divergence; doesn't affect semantics since the entries are
  independent.

## Phase 2 — Perl L150-350 (C.1.3 Fragile, C.3 Sentences, C.4 Sectioning, C.5 Page Styles)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 177-237 | RawTeX block: `\@ignorefalse`/`\@ignoretrue`, `\zap@space`, `\@unexpandable@protect`, `\x@protect`/`\@x@protect`, `\@typeset@protect`, `\set@display@protect`/`\set@typeset@protect`, `\protected@edef`/`\protected@xdef`/`\unrestored@protected@xdef`/`\restore@protect`, `\@nobreakfalse`/`\@nobreaktrue`, conditionals (`\ifv@`, `\ifh@`, `\ifdt@p`, `\if@pboxsw`, `\if@rjfield`, `\if@firstamp`, `\if@negarg`, `\if@ovt`/`\if@ovb`/`\if@ovl`/`\if@ovr`), dimens (`\@ovxx`/`\@ovyy`/`\@ovdx`/`\@ovdy`/`\@ovro`/`\@ovri`), `\if@noskipsec` true | latex_constructs.rs (C.1.3 area) | ↻ MISPLACED |
| 255 | `\fmtname` "LaTeX2e" | latex_constructs.rs:2960 | ↻ MISPLACED |
| 256 | `\fmtversion` "2018/12/01" | latex_constructs.rs:2961 | ↻ MISPLACED |
| 261 | `Let '\@@par' '\par'` | latex_constructs.rs:2991 | ↻ MISPLACED |
| 262 | `\@par` (`\let\par\@@par\par`) | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 263 | `\@restorepar` (`\def\par{\@par}`) | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 268 | `NewCounter('footnote')` | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 269 | `\thefootnote` (`\arabic{footnote}`) | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 270 | `NewCounter('mpfootnote')` | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 271 | `\thempfn` (`\thefootnote`) | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 272 | `\thempfootnote` (`\arabic{mpfootnote}`) | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 273 | `\footnotesep` register Dimension(0) | latex_constructs.rs (C.3) | ↻ MISPLACED |
| 287 | `\appendixname` "Appendix" | latex_constructs.rs:9059 (also at L3406 — DUP) | ↻ MISPLACED + ⚠ DUP |
| 288 | `\appendixesname` "Appendixes" | latex_constructs.rs (C.4) | ↻ MISPLACED |
| 294 | `\contentsname` "Contents" | latex_constructs.rs:3416 | ↻ MISPLACED |
| 295 | `\listfigurename` "List of Figures" | latex_constructs.rs (C.4) | ↻ MISPLACED |
| 296 | `\listtablename` "List of Tables" | latex_constructs.rs (C.4) | ↻ MISPLACED |
| 300 | `NewCounter('tocdepth')` | latex_constructs.rs (C.4) | ↻ MISPLACED |
| 309 | `\columnsep` Dimension(0) | latex_constructs.rs:3879 | ↻ MISPLACED |
| 310 | `\columnseprule` Dimension(0) | latex_constructs.rs:3880 | ↻ MISPLACED |
| 311 | `\mathindent` Dimension(0) | latex_constructs.rs (C.5) | ↻ MISPLACED |
| 312 | `NewCounter('secnumdepth')` | latex_constructs.rs (C.5) | ↻ MISPLACED |
| 317-331 | RawTeX: `\@ifl@t@r`/`\@parse@version@`/`\@parse@version`/`\@parse@version@dash` | latex_constructs.rs (C.5) | ↻ MISPLACED |
| 343-347 | `\sectionmark`/`\subsectionmark`/`\subsubsectionmark`/`\paragraphmark`/`\subparagraphmark` | latex_constructs.rs:4196+ | ↻ MISPLACED |

### Phase 2 findings

* **Massive ↻ MISPLACED cluster**: All Perl L177-347 entries are in
  Rust `latex_constructs.rs`, NOT `latex_base.rs`. Per CLAUDE.md
  "Every `\foo` defined in `LaTeXML/blib/lib/LaTeXML/Engine/<file>.pool.ltxml`
  must be defined in `latexml_engine/src/<file>.rs`" — these
  should ALL move to `latex_base.rs`.
* **⚠ Duplicate**: `\appendixname` defined twice in
  latex_constructs.rs (L3406 AND L9059). Perl also has it twice
  (`latex_base.pool.ltxml:287` AND `latex_constructs.pool.ltxml:5783`),
  so the Rust duplication is parity-faithful — but at the Perl level,
  the latex_constructs entry overrides the latex_base one. Should
  the Rust order match? Probably fine: latex_constructs.rs loads
  AFTER latex_base.rs, so the L9059 entry wins, which mirrors Perl.
* **Fragile-Commands cluster (L177-237)**: This is a large RawTeX
  block; relocating it requires careful scoping (the
  `\protected@edef`/`\@unexpandable@protect` chain depends on
  loading order and is referenced by many later definitions).
  Defer migration; flag as ↻ for tracking.

## Phase 3 — Perl L350-550 (C.8 Defining, C.9 Floats, C.11 Files/Boxes)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 357 | `\@tabacckludge {}` (`\csname\string#1\endcsname`) | latex_constructs.rs (C.8) | ↻ MISPLACED |
| 359-360 | `\DeclareTextAccent DefToken {}{}` (closure: `ignoredDefinition`) | latex_constructs.rs (C.8) | ↻ MISPLACED |
| 361-362 | `\DeclareTextAccentDefault{}{}` (closure) | latex_constructs.rs (C.8) | ↻ MISPLACED |
| 365-366 | `\DeclareTextComposite{}{}{}{}` (closure) | latex_constructs.rs (C.8) | ↻ MISPLACED |
| 367-368 | `\DeclareTextCompositeCommand{}{}{}{}` (closure) | latex_constructs.rs (C.8) | ↻ MISPLACED |
| 391 | `\flushbottom` (DefPrimitive undef) | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 392 | `\suppressfloats[]` (DefPrimitive undef) | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 394 | `NewCounter('topnumber')` | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 395 | `\topfraction` "0.25" | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 396 | `NewCounter('bottomnumber')` | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 397 | `\bottomfraction` "0.25" | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 398 | `NewCounter('totalnumber')` | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 399 | `\textfraction` "0.25" | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 400 | `\floatpagefraction` "0.25" | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 401 | `NewCounter('dbltopnumber')` | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 402 | `\dbltopfraction` "0.7" | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 403 | `\dblfloatpagefraction` "0.25" | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 404-414 | float separators/extents `\floatsep`/`\textfloatsep`/`\intextsep`/`\dblfloatsep`/`\dbltextfloatsep`/`\@fptop`/`\@fpsep`/`\@fpbot`/`\@dblfptop`/`\@dblfpsep`/`\@dblfpbot` (DefRegister Glue) | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 415-417 | `Let \topfigrule \relax`, `\botfigrule`, `\dblfigrule` | latex_constructs.rs (C.9) | ↻ MISPLACED |
| 454-456 | `\DeclareRobustCommand` DefPrimitive | latex_constructs.rs (C.13) | ↻ MISPLACED |
| 457-486 | RawTeX block: `\newsavebox`/`\savebox`/`\sbox`/`\@savebox`/`\@isavebox`/`\@savepicbox`/`\@isavepicbox`/`\lrbox`/`\endlrbox`/`\usebox` | latex_constructs.rs (C.13) | ↻ MISPLACED |
| 516-554 | `\PackageError`/`\PackageWarning`/`\PackageWarningNoLine`/`\PackageInfo`/`\ClassError` (RawTeX `\gdef`/`\def`) | latex_base.rs:266-308 (TeX!) | ✅ |

### Phase 3 findings

* **C.8 / C.9 / C.13 clusters all misplaced**: ~25 entries should
  be in latex_base.rs but are in latex_constructs.rs.
* **PARITY for `\PackageError`/etc**: The error/warning RawTeX block
  IS in latex_base.rs:266-308 — correctly placed (Perl L516-554).

## Phase 4 — Perl L550-865 (file end: PackageError finish, math chardefs, registers, hooks)

| Perl L | Symbol | Rust file:line | Status |
|--------|--------|----------------|--------|
| 550-594 | `\ClassWarning`/`\ClassWarningNoLine`/`\ClassInfo`/`\@latex@error`/`\@latex@warning`/`\@latex@warning@no@line`/`\@latex@info`/`\@latex@info@no@line` (RawTeX cont.) | latex_base.rs:266-352 (TeX!) | ✅ |
| 601-606 | `\@xxxii`, `\@Mi`-`\@Miv` mathchardef | latex_base.rs:363-368 | ✅ |
| 607 | `\@fontenc@load@list` (`\@elt{T1,OT1}`) | latex_base.rs:369 | ✅ |
| 609-620 | `\@vpt`-`\@xxvpt` redefinitions (string form) | latex_base.rs:177-188 (T_OTHER+string mixed form, see Phase 1) | ⚠ shape (Perl L609-620 OVERRIDES L129-140 with all-string form; Rust matches L129-140) |
| 622-625 | `\@tempa`, `\@tempb`, `\@tempc`, `\@gtempa` | latex_base.rs:373-376 | ✅ |
| 627-628 | `\defaultscriptratio` ".7", `\defaultscriptscriptratio` ".5" | latex_base.rs:378-379 | ✅ |
| 630-803 | Big RawTeX block: `\loop`, ~80 register declarations (`\@ydim`, `\@arstrutbox`, etc.), `\@sqrt`, conditionals (`\if@filesw`, `\if@partsw`, `\@tempswa*`, `\@tempcnta`/`\@tempcntb`, `\@tempdim*`, `\@tempbox*`, `\@tempskip*`, `\@temptokena`, `\@flushglue`, `\if@afterindent`, `\rootbox`, eq-related, iteration helpers `\@whilenum`/`\@iwhilenum`/`\@whiledim`/`\@iwhiledim`/`\@whilesw`/`\@iwhilesw`, `\@nnil`/`\@fornoop`/`\@for`/`\@forloop`/`\@iforloop`/`\@tfor`/`\@tf@r`/`\@tforloop`/`\@break@tfor`/`\@removeelement`, `\remove@to@nnil`/`\remove@angles`/`\remove@star`/`\@defaultunits`, math/list flags (`\ifmath@fonts`, `\@labels`, `\if@inlabel`, `\if@newlist`, `\if@noparitem`, `\if@noparlist`, `\if@noitemarg`, `\if@nmbrlist`), `\glb@settings`) | latex_base.rs:385-549 (TeX!) | ✅ |
| 809 | `\loggingall` (DefMacroI Tokens()) | latex_base.rs:557 | ✅ |
| 829 | `\hook_gput_code:nnn {}{}{}` "" | latex_base.rs:579 | ✅ |
| 830 | `\NewHook{}` "" | latex_base.rs:580 | ✅ |
| 831 | `\NewReversedHook{}` "" | latex_base.rs:581 | ✅ |
| 832 | `\NewMirroredHookPair{}{}` "" | latex_base.rs:582 | ✅ |
| 833 | `\ActivateGenericHook{}` "" | latex_base.rs:583 | ✅ |
| 834 | `\DisableGenericHook{}` "" | latex_base.rs:584 | ✅ |
| 835 | `\AddToHook{}[]{}` "" | latex_base.rs:585 | ✅ |
| 836 | `\AddToHookNext{}{}` "" | latex_base.rs:586 | ✅ |
| 837 | `\ClearHookNext{}` "" | latex_base.rs:587 | ✅ |
| 838 | `\RemoveFromHook{}[]` "" | latex_base.rs:588 | ✅ |
| 839 | `\SetDefaultHookLabel{}` "" | latex_base.rs:589 | ✅ |
| 840 | `\PushDefaultHookLabel{}` "" | latex_base.rs:590 | ✅ |
| 841 | `\PopDefaultHookLabel` "" | latex_base.rs:591 | ✅ |
| 842 | `\UseHook{}` "" | latex_base.rs:592 | ✅ |
| 843 | `\UseOneTimeHook{}` "" | latex_base.rs:593 | ✅ |
| 844 | `\ShowHook{}` "" | latex_base.rs:594 | ✅ |
| 845 | `\LogHook{}` "" | latex_base.rs:595 | ✅ |
| 846 | `\DebugHooksOn` "" | latex_base.rs:596 | ✅ |
| 847 | `\DebugHooksOff` "" | latex_base.rs:597 | ✅ |
| 848 | `\DeclareHookRule{}{}{}{}` "" | latex_base.rs:598 | ✅ |
| 849 | `\DeclareDefaultHookRule{}{}{}` "" | latex_base.rs:599 | ✅ |
| 850 | `\ClearHookRule{}{}{}` "" | latex_base.rs:600 | ✅ |
| 851 | `\IfHookEmptyTF{}{}{}` "#3" | latex_base.rs:601 | ✅ |
| 852 | `\IfHookExistsTF{}{}{}` "#3" | latex_base.rs:602 | ✅ |
| 853 | `\MakeTextLowercase` "\lowercase" | latex_base.rs:603 | ✅ |
| 854 | `\MakeTextUppercase` "\uppercase" | latex_base.rs:604 | ✅ |
| 856 | `\if@includeinrelease` (DefConditional) | latex_base.rs:607 | ✅ |
| 857 | `Let \@kernel@after@enddocument \@empty` | latex_base.rs:608 | ✅ |
| 858 | `Let \@kernel@after@enddocument@afterlastpage \@empty` | latex_base.rs:609 | ✅ |
| 859 | `Let \@kernel@before@begindocument \@empty` | latex_base.rs:610 | ✅ |
| 860 | `Let \@kernel@after@begindocument \@empty` | latex_base.rs:611 | ✅ |
| 861 | `Let \conditionally@traceon \@empty` | latex_base.rs:612 | ✅ |
| 862 | `Let \conditionally@traceoff \@empty` | latex_base.rs:613 | ✅ |

### Phase 4 findings

* **Strong PARITY** for L550-865. The error-message RawTeX (L516-594),
  math chardefs (L601-608), temp registers (L622-628), big RawTeX
  registers/iteration/lists block (L630-803), `\loggingall`,
  expl3 hook stubs (L829-854), and kernel conditionals/Lets
  (L856-862) are ALL correctly placed in latex_base.rs.
* **⚠ shape divergence** (L609-620): Perl explicitly redefines
  `\@vpt`-`\@xxvpt` with all-string form here, OVERRIDING the
  earlier L129-140 mixed form. Rust only mirrors L129-140's mixed
  form. Functionally negligible (T_OTHER('5') and "5" digest
  identically).
* **🔵 Rust-only** (latex_base.rs:631): `\nofiles` defined as
  `\@fileswfalse`. Not in Perl source. Used by raw `latex.ltx`
  load — Rust pre-defines as a stub. Move to `_rust_only.rs`?
  Actually it's a small one-liner; reasonable as documented stub.

## Cumulative parity health (Perl L1-L865, 100% of latex_base.pool.ltxml)

* **Phase 1** (L1-150): ✅ Strong PARITY for C.0 Preliminaries.
* **Phase 2** (L150-350): ↻ MASSIVE MISPLACED cluster — entire
  C.1.3 Fragile, C.3 Sentences, C.4 Sectioning, C.5 Page Styles
  blocks live in `latex_constructs.rs`. Should relocate to
  `latex_base.rs`.
* **Phase 3** (L350-550): ↻ MISPLACED — C.8 Defining Commands,
  C.9 Floats, C.13 Boxes (the `\DeclareRobustCommand`/
  `\newsavebox`/`\sbox`/etc cluster).
* **Phase 4** (L550-865): ✅ Strong PARITY — error-msg infra,
  math chardefs, big RawTeX register block, hooks, kernel Lets.

## Pending parity work (post-audit)

✅ **Phase 2/3 latex_base reverse-migration 100% COMPLETE**
(commits 1b0dc204c..426c64b68):

### All migrations:
* C.1.3 Fragile Commands RawTeX (Perl L177-237) — `\protect` chain,
  conditionals, dimens (commit `751bfb2a2`)
* C.3.1 \fmtname/\fmtversion (L255-256)
* C.3.2 \@@par/\@par/\@restorepar (L261-263)
* C.3.3 footnote counters + \footnotesep (L268-273)
* C.4.2 \appendixname/\appendixesname (L287-288)
* C.4.3 \contentsname/\listfigurename/\listtablename (L294-296)
* C.4.4 NewCounter('tocdepth') (L300)
* C.5.1 \columnsep/\columnseprule/\mathindent (L309-311)
* C.5.1 NewCounter('secnumdepth') (L312)
* C.5.2 \@ifl@t@r/\@parse@version* RawTeX (L317-331)
* C.5.4 \sectionmark family (5 entries) (L343-347)
* C.8.1 \@tabacckludge/\DeclareTextAccent family (L357-368)
* C.9.1 float infrastructure (~22 entries) (L391-417)
* C.13 \DeclareRobustCommand DefPrimitive (L454-456)
* C.13 Savebox RawTeX block — \newsavebox/\savebox/\sbox/etc (L457-486)

### Architecture pattern

For Perl-latex_base entries the dump doesn't capture:
* **Primary**: defined in `latex_base.rs` (NODUMP path / strict file-name parity)
* **Dump-path coverage**: mirrored in `latex_constructs_rust_only.rs`
  (loaded after dump). NewCounter/DefRegister are idempotent so
  dual-definition is safe.

### Tests (continuously green throughout)

50_structure 45/0, 30_encoding 26/0, 53_alignment 29/0, 56_ams 7/0.
