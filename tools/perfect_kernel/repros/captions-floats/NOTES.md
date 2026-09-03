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

## ROOT 2 — <ltx:caption>/<ltx:toccaption> not allowed in <ltx:block> — SHARED, in scope

Repros: caption_isorot_sidewaystable.tex (isorot, RED), caption_in_parbox_float.tex (rubik, RED).
Both: same-host Perl emits the identical 2 errors → SHARED; pdflatex 0 !.

Mechanism (common root): `\caption`/`\toccaption` is constructed while the insertion
point is inside a box element (`<ltx:block>` from a minipage, or
`<ltx:p class=ltx_parbox>`+`<ltx:block>` from a parbox) built via `_CaptureBlock_`
(latexml_engine/src/base_utilities.rs:4146 warns "Did not find a block-like candidate").
At construction time the enclosing float is NOT a live ancestor of the detached
capture, so the caption constructor's `^^` float-up
(latex_constructs.pool.ltxml:3368 `\@@caption PBoxContents "^^<ltx:caption>#1</ltx:caption>"`
= Compiler.pm:118-120 → `float_to_element('ltx:caption', true)`,
Rust latexml_core/src/document.rs:5826) finds no container and returns None; the
subsequent `openElement` then errors at document.rs:3203 ("isn't allowed in <ltx:block>").
Rust `float_to_element` is a faithful port of Perl `floatToElement`
(Core/Document.pm:1052) — hence identical failure = SHARED.

Two source shapes:
- isorot: isorot.sty raw-loads (NO binding, Perl or Rust). `\sidewaystable`→`\@rotfloat`
  →`\@xrotfloat` = `\@float{table}` + `\begin{lrbox}\rot@float@box` + `\begin{minipage}`
  (isorot.sty:139-147, 223-226). The minipage capture hosts the caption.
- rubik: plain kernel `\parbox` FOLLOWING other float content (rubikexamples.tex:295-303).
  (A parbox that is the figure's SOLE child is absorbed → clean; needs preceding content.)

Fix decision: (a) wrapper binding — VERIFIED. The bound `rotating` package
(rotating.sty.ltxml `DefEnvironment('{sidewaystable}[]', "<ltx:table ...>#tags#body</ltx:table>")`)
puts `#body` DIRECTLY inside the float (no lrbox/minipage capture), so the caption is
built with the float as a live ancestor. Tested `\usepackage{rotating}` +
sidewaystable+caption → 0 errors, `<ltx:caption>` correctly inside `<ltx:table>`.
LaTeXML chose this idiom deliberately (rotating.sty.ltxml comment: the raw minipage
"puts the caption where it can't be").

Fix plan (root 2 = isorot): create `latexml_package/src/package/isorot_sty.rs` (register in
package dispatch) mirroring rotating.sty.ltxml for the float surface:
`sidewaysfigure[]`, `sidewaysfigure*[]`, `sidewaystable[]`, `sidewaystable*[]`
(DefEnvironment → `<ltx:figure>`/`<ltx:table>` with `#tags#body`, mode internal_vertical,
before_digest before_float(...)+rotatedPage(hsize=textheight), after_digest after_float,
after_digest_body rotated_properties(body,90)); plus `\rotcaption{}`→`\caption{\turnbox{90}{#1}}`,
`\controtcaption`, and the option/`\rotdriver`/`\clockwise`/`\counterclockwise`/
`\figuresright`/`\figuresleft` no-ops isorot declares. Reuse rotating_sty.rs helpers.
(Also add Perl `isorot.sty.ltxml` for oracle parity.) Guard asserts: 0 errors AND
`//ltx:table/ltx:caption` (or figure) present — caption is a child of the float, not a block.
Risk: LOW-MED (new binding, but a near-copy of a proven one; isorot's non-float
machinery is typesetting-only). Expected gain: isorot/rotman (3 errors → the 2 caption
errors + likely the figure-in-quote at rotman.tex is a separate check).

Deferred sub-case (rubik / plain-kernel parbox-in-float): NOT fixable by a wrapper
binding. Would require the caption's `^^` float_to_element (or a document.rs leniency)
to escape the `_CaptureBlock_` boundary up to the enclosing float — architecturally
invasive (breaks capture isolation) and SHARED with Perl. Recommend a separate root:
option (b) only if the capture graft can expose the float as a float-up target. 1 doc
(rubikexamples), 4 errors.

Dead ends:
- Minimal `\parbox{..}{\caption{..}}` as figure's SOLE child: CLEAN (parbox absorbed into
  figure). Not a reproducer — need content before the parbox.
- "Just insert-anyway + suppress error" (section-in-item style): would leave `<ltx:caption>`
  inside `<ltx:block>` (schema-invalid) — fails the structural-correctness bar.

## ROOT 2 refinement — isorot binding as a DELTA over rotating_sty.rs

rotating.sty and isorot.sty share the SAME environment/command names; bind isorot by
reusing rotating's bound surface and adding only isorot's extras. Commands isorot.sty
defines beyond rotating's BOUND surface (rotating_sty.rs already binds sideways/turn/
rotate/turnbox/sidewaysfigure(*)/sidewaystable(*)/rotcaption):
- `\rotdriver#1`            isorot.sty:35  — declare dvi->PS driver (rotating.sty:64
                                             `\input{#1.def}`). LaTeXML no-op `{}`.
- `\clockwise`             isorot.sty:53  — set positive-angle sense CW. (rotating: an
                                             OPTION; here a COMMAND.) no-op.
- `\counterclockwise`      isorot.sty:56  — set sense CCW. no-op.
- `\figuresleft`          isorot.sty:45  — sideways-float sense. (rotating: option.) no-op.
- `\figuresright`         isorot.sty:49  — sideways-float sense. (rotating: option.) no-op.
- `\rotatedirection{}`    isorot.sty:81  — set sense from arg (clockwise/…). no-op `{}`.
- `\controtcaption`       isorot.sty:257-258 — continued rotated caption (no counter step).
                                             Faithful: `\let\controtcaption\rotcaption`
                                             (or `\rotcaption`-shape; cf. elsart `\contcaption`).
- `\rotcapfont`           isorot.sty:218 — rotated-caption font. no-op.
- options `errorshow`/`debugshow` isorot.sty:21-22 + `\DeclareOption*`->graphics. Declare/pass.
- `\RequirePackage{graphicx}` (33) + `\RequirePackage{lscape}` (34) — lscape gives `landscape`.
Internals `\@rotfloat`/`\@xrotfloat`/`\end@rotfloat`/`\@rotdblfloat` (isorot.sty:139-213)
are NOT reached once sidewaysfigure/sidewaystable are bound directly — omit them.
So isorot_sty.rs = RequirePackage!("rotating") (reuse its sideways*/rotcaption/turn/…)
  + RequirePackage!("lscape") + the ~8 no-op/alias deltas above.

## ROOT 5 — \mult@@cols (adjmulticol) — SHARED, in scope

Repro: adjmulticol_multcols.tex (RED: \mult@@cols undefined + \endmulticols mode error;
pdflatex 0 !; same-host Perl identical 2 errors).
Mechanism: adjmulticol.sty raw-loads (NO binding, Perl or Rust). It
`\RequirePackage{multicol}` then `\adjmulticols#1#2#3` (adjmulticol.sty:110) ->
`\adjmult@cols` (:143) -> `\adjmult@@cols#1[#2]` (:147) which adjusts `\linewidth`,
`\let\page@sofar=\adjmc@page@sofar`, then calls `\mult@@cols#1[#2]` (:151) = multicol.sty:172.
`\endadjmulticols` (:196) -> `\endmulticols`. It reuses multicol internals: `\mult@@cols`
(:151), `\col@number`, `\premulticols`, `\page@sofar`, `\balance@columns`,
`\mult@firstbox`/`\mult@box` (:189), `\multicol@leftmargin` (:181), `\endmulticols` (:200),
`\mult@footnotetext`/`\multi@column@out`. multicol_sty.rs binds `{multicols}` only at the
`<ltx:pagination role="start/end_#1_columns"/>` marker level (Perl multicol.sty.ltxml:21
identical: "basically styling, we can ignore the effects") and exposes NONE of those
internals -> `\mult@@cols` undefined; `\endmulticols` then hits the wrong mode.

Fix decision: (i) adjmulticol needs its OWN binding — NOT (ii) exposing `\mult@@cols`'s
real body. `\mult@@cols` (multicol.sty:172) is a page-layout OUTPUT ROUTINE (`\vsplit`,
column balancing, `\mult@box`) that LaTeXML does not and cannot emulate by design —
LaTeXML emits pagination markers, not columns. So a faithful `\mult@@cols` body is
infeasible and off-model; the fix is a thin adjmulticol binding mapping to multicol's
semantic output.

Fix plan: create `latexml_package/src/package/adjmulticol_sty.rs` (register in dispatch)
+ Perl `adjmulticol.sty.ltxml` for oracle parity:
  DefEnvironment!("{adjmulticols}{}{}{}",
    "<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>",
    mode => "internal_vertical");
  DefEnvironment!("{adjmulticols*}{}{}{}",  ...same... );
#1 = column count; #2 = inner margin, #3 = outer margin — both GOBBLED (page layout,
ignored, exactly as multicol drops its own effects). No heading optional (adjmulticols has
none; its `[\premulticols]` is internal). `\DeclareOption*`->multicol + RequirePackage!("multicol").
Guard asserts: 0 errors AND `//ltx:pagination[@role="start_2_columns"]` +
`//ltx:pagination[@role="end_2_columns"]` wrap the body `<ltx:p>` (multicolumn content
present), NO `<ltx:block>` leak.
Risk: LOW (2-env thin binding, copy of a proven multicol pattern). Gain: adjmulticol/sample
(4 errors -> 0), 1 doc.

Dead ends:
- Exposing multicol's `\mult@@cols`/`\endmulticols`/`\balance@columns` with real bodies:
  off-model (LaTeXML has no output routine / column balancer); would need the TeX page
  builder. Rejected.
