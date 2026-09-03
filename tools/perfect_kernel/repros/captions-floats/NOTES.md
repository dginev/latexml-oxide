# Topic: captions-floats (wave-15) — NOTES

Binary: /home/deyan/data/pk_bin/latexml_oxide.b54t
Residue: /home/deyan/data/pk_agents/w15/first_errors36.tsv

## Candidate list (ranked by docs × errors, topical fit)

| rank | first error | docs | errs | class | fix site |
|------|-------------|------|------|-------|----------|
| 1 | `\caption@prepareslc` undefined | 9 (all hep-*) | 8×1, hep-math 40 | SHARED | caption_sty.rs |
| 2 | `<ltx:toccaption>/caption> not allowed in <ltx:block>` | 2 (rubik, isorot/rotman) | 4,3 | ? structural | doc/document.rs (rot/framed float) |
| 3 | `\titlewidth` undefined | 1 (titlesec.tex) | 5 | SHARED | titlesec_sty.rs |
| 4 | `\thm@topsepadd` undefined | 1 (dlfltxbcodetips) | 5 | SHARED | ntheorem_sty.rs |
| 5 | `\mult@@cols` undefined | 1 (adjmulticol/sample) | 4 | ? (multicol internal) | multicol_sty.rs |
| 6 | `\caption` outside float | 1 (threeparttablex) | 1 | ? structural | later |
| 7 | `\TBWarning` undefined | 1 (latex-doc-ptr) | 5 (mixed) | OFF-TOPIC | see note |

hep-* docs (rank 1) all load `\usepackage{...}{hep-paper}` → hep-paper loads
hep-bibliography.sty, whose L108-109 does
`\AtBeginDocument{\@ifpackageloaded{caption}{\g@addto@macro\caption@prepareslc{...}}{}}`.

## ROOT 1 — \caption@prepareslc (TOP, 9 docs) — SHARED, in scope

Repro: caption_prepareslc.tex  (RED: Error:undefined:\caption@prepareslc; pdflatex 0 !)
Mechanism:
  - caption3.sty:1595  `\providecommand*\caption@prepareslc{}` (defined empty; used
    at :1575 and appended at :1594 `\g@addto@macro\caption@prepareslc}`).
  - hep-bibliography.sty:108 appends to it at begin-document when caption loaded.
  - Rust caption_sty.rs is a full EMULATION (does NOT raw-load caption3.sty) and
    omits `\caption@prepareslc`; so `\g@addto@macro\caption@prepareslc` → undefined.
Perl: LaTeXML/lib/LaTeXML/Package/caption.sty.ltxml — no prepareslc, no caption3
  raw-load → Perl fails identically = SHARED.
Fix plan: caption_sty.rs `LoadDefinitions!{}` — near the other caption3 internals
  (after `\DeclareCaptionListFormat`, ~L189) add
    `DefMacro!("\\caption@prepareslc", "");`  // caption3.sty:1595 \providecommand*
  (empty hook body; short is fine — `*` in the source). This is the faithful
  completion of the emulation for the sibling-package caller.
Guard: repro loads caption + `\g@addto@macro\caption@prepareslc{\relax}`; assert
  0 errors AND `<ltx:p>` with body text present.
Risk: LOW. Expected gain: up to 9 docs clear their first error (hep-math still has
  40 downstream errors — see \tempa; the other 8 are single-error).

## ROOT 3 — \titlewidth (titlesec.tex) — SHARED, in scope

Repro: titlesec_titlewidth.tex (RED: \titlewidth undefined + 2× <variable>; pdflatex 0 !)
Mechanism: titlesec.sty:1039-1041 unconditional
  `\newdimen\titlewidth \newdimen\titlewidthlast \newdimen\titlewidthfirst`.
  titlesec.tex:1780 `\titleformat{\section}[block]{...\addtolength{\titlewidth}{2pc}...}`
  executes at `\section`. titlesec_sty.rs (164-line emulation) omits all three dimens.
Perl: titlesec.sty.ltxml has no \titlewidth (only \wordsep) → SHARED.
Fix plan: titlesec_sty.rs `LoadDefinitions!{}` — add
    `DefRegister!("\\titlewidth"      => Dimension::new(0));`
    `DefRegister!("\\titlewidthlast"  => Dimension::new(0));`
    `DefRegister!("\\titlewidthfirst" => Dimension::new(0));`
Guard: repro above; assert 0 errors AND a `<ltx:section>` present.
Risk: LOW. Expected gain: 1 doc (titlesec.tex) — clears \titlewidth + the 4
  cascading <variable> errors.

## ROOT 4 — \thm@topsepadd (dlfltxbcodetips) — SHARED, in scope

Repro: ntheorem_thm_topsepadd.tex (RED: \thm@topsepadd undefined + <variable>; pdflatex 0 !)
Mechanism: ntheorem.sty:714-715 unconditional `\newskip\thm@topsep \newskip\thm@topsepadd`.
  dlfltxbcodetips.sty:102-106 ("code stolen from ntheorem.sty") uses
  `\thm@topsepadd \theorempostskipamount` / `\advance\thm@topsepadd\partopsep`.
  ntheorem_sty.rs binding defines \theorem*skipamount registers but omits these two.
Perl: ntheorem.sty.ltxml also omits (grep thm@topsep = none) → SHARED.
Fix plan: ntheorem_sty.rs `LoadDefinitions!{}` — beside the \theorem*skipamount block
  (~L53) add
    `DefRegister!("\\thm@topsep"    => Dimension::new(0));`
    `DefRegister!("\\thm@topsepadd" => Dimension::new(0));`
Guard: repro above; assert 0 errors AND a `<ltx:document>` (body text present).
Risk: LOW. Expected gain: 1 doc (dlfltxbcodetips) clears its first error + 4 cascades
  (note dlfltxb bundle has 5 docs; the sample used [amsmath] option — verify siblings).

## Deferred / later-checkpoint roots
- ROOT 2 toccaption-in-block (rubik, isorot): caption emitted inside a `<ltx:block>`
  produced by a rotated/framed float wrapper — structural schema violation. Needs
  document.rs auto-open/relocate analysis. HIGHER value (2 docs + XML structure).
- ROOT 5 \mult@@cols (adjmulticol): multicol.sty:172 `\def\mult@@cols#1[#2]`;
  adjmulticol.sty:151 raw-calls `\mult@@cols`. multicol_sty.rs emulation omits it.
- ROOT 6 threeparttablex `\caption outside float`: structural.
- ROOT 7 \TBWarning (latex-doc-ptr): OFF caption/float. `\TBWarning` (ltugboat.cls)
  reached only via the ELSE fallthrough of latex-doc-ptr.sty's `\@currsize` font-size
  cascade (L200-217) — Rust's `\@currsize` not matching `\ifx\@currsize\LARGE...`
  triggers `\SMC@unknown@warning`→`\TBWarning`. Real root = `\@currsize` state, not a
  missing macro. Doc also fails on endnotes.sty `\@enotes*` + missing .ent. Recommend
  reassign to a font-state / endnotes topic.
