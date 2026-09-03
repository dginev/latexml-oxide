# expansion-primitives — NOTES (Wave 15, Checkpoint 1)

Binary: /home/deyan/data/pk_bin/latexml_oxide.b54r  | preload [rawstyles,rawclasses]latexml.sty
Residue: /home/deyan/data/pk_agents/w14/first_errors.tsv

## Scope decision
Core e-TeX/pdfTeX expansion primitives are HEALTHY (probe_primitives.tex, GREEN):
\numexpr \dimexpr \ifcsname \ifincsname \detokenize \unexpanded \pdfstrcmp
\currentgrouplevel \lastnodetype all evaluate correctly. No Rust-only primitive
DIVERGENCE found. The only undefined in that probe is \pdf@strcmp (pdftexcmds
package wrapper, not a primitive). => primitive-divergence hunt = DEAD END.

## Ranked (a)+(b) candidates that belong to this topic

R1  \pdfoutline  tools/tools-overview (1 err)  — pdfTeX primitive. SHARED. ROOT-CAUSED, has fix. LOW.
R2  \GenericError/\ifpdf  scanpages/scanpages-doc (1) — engine-identity. SHARED. Risk HIGH (policy).
R3  \GenericError  numspell/numspell (12) — xstring \fullexpandarg/\StrChar leading-space. Needs isolation; borders string-mouth.
    \GenericError  tidyres/tidyres-doc (1) — "Not in outer par mode" => boxes-groups topic, NOT mine.

## Report-only (never fix here)
\csstring        abntexto (11), abntexto-uece (2)  — PARKED engine-detection probe. lualatex oracle.
\epTeXinputencoding gckanbun/whole-vert (413), gckanbun/kanshi (395), kksymbols (392) — PARKED e-pTeX. Japanese.
\digitalasset    aastex/aastex701-sample (1) — aastex701.cls:13638 class internal (owner: sectioning-frontmatter).

## Not-mine primitives (hand to owner topic)
\luatexattributedef  bxcoloremoji-ja(100)/bxjscls(61)/bxcjkjatype(44x2)/bxjaprnind(44)/kanbun(39) — LuaTeX primitive (luatexja); owner luatex-profile.
\newXeTeXintercharclass  hang/sample(37), hang/hang(17) — XeTeX primitive (polyglossia); owner luatex-profile/babel.
\pdfmeta_xmp_xmlns_new:nn  zugferd x2 (4 each) — l3pdf/expl3 namespace, NOT a primitive; owner expl3.

## Root cause R1 — \pdfoutline  [repro: pdfoutline_tools-overview.tex, RED]
Mechanism: pdfTeX primitive `\pdfoutline <attr spec> <action spec> [count N] <general text>`
writes a PDF bookmark, zero typeset output. Undefined in the Rust engine.
Perl: pdfTeX.pool.ltxml:179-180 only COMMENTS `\pdfoutline`/`\pdfdest` (never DefPrimitive).
Rust mirrors that comment verbatim at latexml_engine/src/pdftex.rs:385-386 — so absent in both.
CLASSIFICATION: SHARED (Perl errors identically; both leave the arg spec as body junk).
pdflatex oracle = 0 errors. Surpass approved.
Extra symptom: a bare noop is NOT enough — repro shows `attr`/`user` leaked as body text,
so the fix MUST consume the outline spec.
FIX: latexml_engine/src/pdftex.rs, LoadDefinitions! block at line 385. Add a
DefParameterType!(OutlineSpecification, reader => …) mirroring OpenAnnotSpecification
(same file L412) that discards: optional `attr <GeneralText>`; an action spec
(`user <GeneralText>` | `goto` … `num/name/file/page` + optional GeneralText);
optional `count <Number>`; then the final `<GeneralText>` title. Then
`def_primitive_noop("\\pdfoutline OutlineSpecification")?;`. (Same one-liner also
covers `\pdfdest` if a witness appears; leave commented for now — no residue doc.)
GUARD: pdfoutline_tools-overview repro → 0 errors AND output contains "Body text."
and does NOT contain the literal "attr" or "user".
RISK: LOW.  CORPUS GAIN: 1 doc (tools-overview) to clean-first-error.

## Root cause R2 — scanpages \ifpdf  [repro: ifpdf_scanpages.tex, RED]
Mechanism: scanpages.sty:22-23 `\ifpdf\else \@latex@error{Must be processed with pdf[la]tex!}\@eha`.
`\ifpdf` (ifpdf → iftex) is FALSE in both engines. Perl: iftex.sty.ltxml:27
`DefConditional('\ifpdf')` (default false) and ifpdf.sty.ltxml:19 `\newif\ifpdf\pdffalse`.
CLASSIFICATION: SHARED (Perl fails identically). pdflatex oracle = 0 (real pdfTeX PDF mode).
This is an engine-IDENTITY policy: flipping \ifpdf true would flip \ifPDFTeX-consistent
DVI/PDF branches everywhere (\pdfliteral etc.). RISK HIGH — do NOT flip globally.
Deferred; owner = engine-identity policy decision, not a local fix.

## Root cause R3 — numspell \StrChar/\fullexpandarg  [repro: strchar_numspell.tex = narrowing probe]
Mechanism: numspell-magyar.sty:380 `\fullexpandarg \StrChar{\thenumspell}{1}[\numspell@firstletter]`.
Error "There is not ' ' in uppercase!" => firstletter = SPACE, systematically (all 12 numbers),
so `\thenumspell` gains a LEADING SPACE under Rust that pdflatex does not have.
strchar_numspell.tex proves the SIMPLE path (`\def\thenumspell{egy}`) is CLEAN in Rust
(first=[e] rest=[gy]) — so the bug is in the compound spelling chain
(numspell@num@spell@hu / \g@addto@macro accumulation / \fullexpandarg full-expansion of
`\'{a}`-accented fragments), NOT in bare \StrChar.
xstring has NO Perl or Rust binding (loads raw both sides) => if Perl converts clean, this is
RUST-ONLY full-expansion machinery; needs same-host Perl run to confirm + deeper isolation
(babel+magyar chain). BORDERS string-mouth topic. Not fixed this checkpoint.
DEAD END: minimal `\def\thenumspell{egy}` does NOT reproduce.
