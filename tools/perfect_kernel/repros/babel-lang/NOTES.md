# babel-lang — Checkpoint 1 (2026-09-03, binary b54r)

Topic: babel/polyglossia/language + font-encoding roots under the pdflatex/lualatex oracle.
KPE #161 (`\ProcessOptions` reads rewritten `\opt@babel.sty`) and #158 (`\@ifundefined`)
already landed in b54r — verified: `greek.polutoniko` is now GREEN (greek.ldf raw-loads,
no stub). Do not re-find those.

## Candidate ranking (in-scope, by docs x error-lines)

### R1  babel-language `.ldf` STUB shadows the real installed file   [RUST-ONLY]  TOP
babel_lang_stubs.rs registers `spanish.ldf`,`czech.ldf`,`vietnamese.ldf`,… via lib.rs
so find_file resolves them to a minimal Rust stub (allocates `\l@<lang>` + empty
captions/extras hooks) EVEN WHEN the real .ldf is on disk (full TL2025). The stub skips
the real .ldf's `\DeclareOption{<modifier>}{}` / `\bbl@declare@ttribute`, so babel's
own leftover `\DeclareOption{<opt>}{\bbl@error{unknown-package-option}}` (babel.sty:4214)
survives to `\ProcessOptions` (babel.sty:4300) and fires. greek has NO stub → raw-loads →
already works. Perl has NO lang stubs (raw .ldf load) → Perl CLEAN. pdflatex CLEAN.
  - unamthesis/UNAMThesis  `mexico`            pdflatex  (2 err)
  - udepcolor/udepcolor-doc-ES  `mexico`,`es-noshorthands`  lualatex  (2 err)
  - csbulletin/csbulletin  czech attribute `split`  pdflatex  (1 err)
Repros (RED under b54r, Perl+pdflatex 0 err): babel_option_mexico.tex,
babel_attribute_split_czech.tex.  probe: es-noshorthands confirmed same family.
Mechanism cites:
  - babel.sty:4130-4140 `\bbl@load@language`/`\InputIfFileExists{#1.ldf}`
  - babel.sty:4210-4227 `\bbl@unkopt` → `\DeclareOption{opt}{unknown-package-option}`
  - babel.sty:4297-4300 `\bbl@tempb` (loads .ldf) then `\ProcessOptions`
  - spanish.ldf:66-88 `\es@genoption` → `\DeclareOption{mexico}{}` + rewrites `\opt@babel.sty`
  - czech.ldf:328 `\bbl@declare@ttribute{czech}{split}` (registers `czech-split` in \bbl@attributes)
  - babel.sty:1512-1538 `\languageattribute` → `unknown-attribute` if not in \bbl@attributes
Diverging Rust site: latexml_package/src/package/babel_lang_stubs.rs (install_lang_stub) +
  registration table latexml_package/src/lib.rs:341-403.
Fix plan: make the stub raw-load the REAL `<lang>.ldf` from disk when it exists (kpse
  FindFile the real file, `\input` it), and fall back to the minimal-hook stub ONLY when
  the file is genuinely absent (the original "minimal TeXLive" intent). Faithful because
  Perl raw-loads the real .ldf. Guard: 0 errors + `//ltx:text[@xml:lang='es']` present
  (mexico) and czech doc has body text; assert `\bbl@attributes` contains `czech-split`.
Risk: MED (raw .ldf load may surface new raw-load gaps in individual languages; greek
  proves the path works). Expected gain: ~3 residue docs + broad spanish/czech/… corpus.

### R2  hang: polyglossia intercharclass `\newXeTeXintercharclass` undefined under [luatex]  [RUST-ONLY?]
latex.ltx:22018-22028 defines `\newXeTeXintercharclass` ONLY inside
`\ifx\XeTeXcharclass\@undefined\else…\fi`. Under the `[luatex]` profile b54r never defines
`\XeTeXcharclass`, and latex.ltx already ran at format-dump time, so the def path was
skipped. The batch-54 l3sys engine-identity fix (latexml_sty/mod.rs:139-165) newly makes
`\sys_if_engine_luatex:TF` TRUE, so polyglossia gloss-latin.ldf:125 now takes its
XeTeX/luatex intercharclass branch and calls the undefined `\newXeTeXintercharclass`; the
`\g_polyglossia_latin_*_class` l3 int allocations are downstream cascade.
  - hang/hang    lualatex  (17 err)
  - hang/sample  lualatex  (37 err)  -> 54 error lines, 2 docs (biggest single win)
Fix plan: in the `[luatex]` profile block (latexml_sty/mod.rs DeclareOption "luatex"),
  replicate latex.ltx:22021-22033 with the engine known: define `\XeTeXcharclass` (no-op
  scanning its <number> args), `\xe@alloc@intercharclass` (countdef), the
  `\e@alloc@intercharclass@top` chardef, and `\newXeTeXintercharclass` allocator; plus the
  `\XeTeXinterchartoks`/`\XeTeXinterchartokenstate` no-ops polyglossia then uses (verify the
  full chain in gloss-latin.ldf — intercharclass spacing has no XML meaning → no-ops).
  Mirrors the sibling no-ops already in that block (`\attributedef`, `\initcatcodetable`).
Risk: MED (chain length — needs every intercharclass primitive polyglossia touches).
  NOT parked: `\XeTeXcharclass` is a typographic primitive, not an engine-detection probe.
  Needs Perl classification (Perl has no luatex profile; likely RUST-ONLY).

### R3  paresse `\GA@parse@UTFviii@a` undefined   [SHARED]  (surpass in scope: pdflatex clean)
paresse-utf8.sty:203-204 `\global\let\GA@parse@UTFviii@a=\parse@UTFviii@a`. `\parse@UTFviii@a`
/`@b` are utf8.def internals (utf8.def:253-265) used by the real `\DeclareUnicodeCharacter`.
LaTeXML (BOTH Rust utf8_def.rs AND Perl utf8.def.ltxml) reimplements `\DeclareUnicodeCharacter`
natively and never defines `\parse@UTFviii@a`, so the `\let` grabs an undefined token →
`\GA@parse@UTFviii@a` undefined → error when used at paresse-utf8.sty:225,248.
  - paresse/paresse-eng  pdflatex  (3 err)
  - paresse/paresse-fra  pdflatex  (6 err: +\og \fg \ieme downstream)
Repro (RED b54r; Perl ALSO fails 7 err incl same token; pdflatex 0 err): paresse_ga_parse_utfviii.tex
Fix plan: in latexml_package/src/package/utf8_def.rs, RawTeX the utf8.def:253-265 defs of
  `\parse@UTFviii@a`,`\parse@UTFviii@b` (+ `\UTFviii@two@octets`/`three`/`four@octets`) verbatim
  (pure uccode/count arithmetic, no engine deps). Then paresse's `\let` grabs a real macro and
  its `\csname u8:…\endcsname` byte-mapping runs. Guard: 0 errors + body text `Bonjour`.
Risk: LOW (adds inert kernel internals). Expected gain: 2 docs. Since SHARED, also fixes
  Perl-parity note (record in KNOWN_PERL_ERRORS as a surpass).

## Lower-priority / adjacent (in babel-lang but deeper or off-topic)
- greek-fontenc/test-lgrenc, textalpha-doc  `\@tabbing@"`,`\@tabbing@<`  lualatex (3 err each)
    babel-greek LGR active-char shorthand inside tabbing; `\@tabbing@`+active `"` built
    dynamically (no static source). Deep shorthand mechanism — defer.
- greek-fontenc/char-list, char-list-alphabeta  `\patch`  lualatex (1 err each) — `\patch`
    source not yet pinned (many packages define it); needs a probe.
- asmeconf/asmeconf-template  `vietnamese.noencoding`  lualatex (1 err) — DIRECT
    `\usepackage[vietnamese.noencoding]{babel}` is CLEAN under b54r (probe_vietnam), so this
    is an asmeconf class-option-passing path, NOT the R1 stub family. Separate investigation.
- dvdcoll/dcexample  `\HyPsd@UTFviii`  pdflatex (6 err) — hyperref PDF-string UTF8 decoder
    (hyperref.sty), a HYPERREF topic, not babel-lang. Reassign.

## PARKED (Japanese/Chinese/Korean — NOT root-caused)
| doc | first error | engine | errs | reason |
|---|---|---|---|---|
| beamertheme-mirage/mirage-beamer-zh | `\uselanguage` (translator.sty:20) | lualatex | 5 | ctexbeamer (Chinese) |
| beamertheme-mirage/mirage-poster-zh | `\uselanguage` (translator.sty:20) | lualatex | 15 | ctexbeamer (Chinese) |
| cjk-ko/cjk-ko-doc | `\CJKspace` | pdflatex | 101 | kotex/CJK (Korean) |
| gckanbun/kanshi-sample | `\epTeXinputencoding` | lualatex | 395 | pTeX (Japanese) |
| gckanbun/whole-vert-sample | `\epTeXinputencoding` | lualatex | 413 | pTeX (Japanese) |
| kksymbols/kksymbols-doc | `\epTeXinputencoding` | lualatex | 392 | pTeX (Japanese) |
| sduthesis/sduthesis-demo | `inputencoding utf8` keyboard char | lualatex | 1 | ctexbook (Chinese) |
Note: the residue `\uselanguage` roots are translator.sty (beamer l10n), NOT the format-level
hyphenation `\uselanguage`; both witnesses are ctexbeamer → parked. mirage-*-en passed.

## Dead ends
- Direct `\usepackage[vietnamese.noencoding,english]{babel}` is clean under b54r — the asmeconf
  error is not reproduced this way (class-option path, not the stub family).
