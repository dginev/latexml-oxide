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

## malformed:ltx family (8 docs) + threeparttablex — grouped by fix class

Ranked by docs × errors. isorot/rubik already covered by root 2.

### Group A (b) — block-level listing element in a HORIZONTAL capture — 2 docs, 8 err — SHARED — TOP
Docs: algpseudocodex (2), coloredtheorem-doc (6). Repro: listingline_in_minipage.tex.
`\begin{algorithmic}` (block-level <ltx:listing>/<ltx:listingline>) inside a `minipage`
(algpseudocodex.tex:73) or a colorbox/theorem body (coloredtheorem). The box is captured
as <ltx:p class=ltx_minipage> / <ltx:text> (horizontal); _CaptureBlock_ fails "Did not find
a block-like candidate in ltx:p" (latexml_engine/src/base_utilities.rs:4146) so listingline
can't open in <ltx:p>, then the capture close fails (Group B symptom). Perl identical.
Fix class (b) document.rs: the _CaptureBlock_ block-repackaging must SYNTHESIZE a block when
a p-capture receives block-level content (generalizes base_utilities.rs:4146, which only
finds an EXISTING block); OR add listing's block parent to `can_contain_indirect` auto-open
from <ltx:p> in find_insertion_point_qsym (document.rs:2985 — same helper as the svg:foreignObject
and inline-Math auto-opens). Structural guard: 0 err + //ltx:listing/ltx:listingline present,
no <ltx:p> parent of listingline.

### Group C (a) — mismatched <ltx:picture> open/close — 2 docs, 4 err — SHARED
Docs: pagelayout example-template, example-text. Repro: pagelayout_picture_close.tex.
pagelayout.cls (raw) `\LoadClass[multi=picture]{standalone}` (pagelayout.cls:227) +
`\template[..]{layout}{\text{..}}` emit a </ltx:picture> with no matching open (standalone
multi=picture wrapping / draft tikz guide). Perl identical.
Fix class (a): a pagelayout (or standalone multi=picture) class binding for \template/\text/
\newtemplate/\placeholder that emits balanced structure. Complex class; MED risk.
Guard: 0 err + the template text present, no stray picture close.

### Group B (b) — capture closes over an un-auto-closeable verbatim/listing descendant — 1 doc, 2 err — SHARED
Doc: testnumberedblock. Repro: numberedblock_verbatim_capture.tex. (Also the 2nd error of
every Group A doc.) `numVblock` (numberedblock.sty raw) captures verbatim into a numbered box
(_CaptureBlock_); on close the open `verbatim` descendant is not auto-closeable → close fails.
Perl identical.
Fix class (b) document.rs: force-closing a capture boundary must hard-close its
non-auto-closeable descendants (verbatim, listingline) — generalizes close_node_internal/
close_to_node at a _CaptureBlock_ boundary. Guard: 0 err + //ltx:verbatim inside the box.

### Group F (a) — \caption without a float context — 1 doc, 1 err — PERL-ORIGIN
Doc: threeparttablex. Repro: threeparttable_caption_captype.tex.
threeparttable.sty:110 `\@ifundefined{@captype}{\def\@captype{table}}{}` (measuredfigure:126
-> figure) lets \caption work outside a float. Rust threeparttable_sty.rs:16 and Perl
threeparttable.sty.ltxml:31 bind `{threeparttable}`/`{measuredfigure}` as bare `#body`,
omitting the \@captype setup → \caption -> \@@generic@caption -> "outside any known float".
Fix class (a): complete the binding — before_digest sets \@captype (table/figure) if undefined
(faithful port of threeparttable.sty:110/126). PERL-ORIGIN (threeparttable.sty.ltxml:31,36).
LOW risk. Guard: 0 err + //ltx:caption inside the threeparttable/tabular block.

### Group E (a/b) — bibliography inside a poster SVG box — 1 doc, 1 err
Doc: xebaposter/poster. `posterbox` (baposter-style, raw) renders as <svg:g> with a captured
block; `\bibliography` inside lands in <ltx:block> in SVG (context <svg:g><svg:g>
<ltx:_CaptureBlock_><ltx:p><ltx:block>). Fix (a) xebaposter/baposter binding, or (b) relocate
bibliography to the document body. 1 doc; lower priority.

### Group D — REASSIGN — screenplay-pkg — 1 doc, 6 err — mode-frame family, NOT document-model
First error `\lx@add@frontmatter@until Attempt to close a group that switched to mode
internal_vertical` (mode-switch frame family, no longer parked). The `<ltx:section> isn't
allowed in <ltx:section>` malformed errors are a downstream cascade of the frontmatter/abstract
mode break (context <ltx:document><ltx:abstract><ltx:section>). Recommend the mode-frame agent.

TOP-3 repros written: listingline_in_minipage.tex (A), pagelayout_picture_close.tex (C),
numberedblock_verbatim_capture.tex (B); plus threeparttable_caption_captype.tex (F, easy PERL-ORIGIN win).
Note A+B are the same document-model capture root (block content in a horizontal capture;
capture-close over un-auto-closeable descendants) — fix together in document.rs.

## Groups A+B — document-model capture class root — IMPLEMENTATION SPEC

Repros: listingline_in_minipage.tex (A), numberedblock_verbatim_capture.tex (B).

### Mechanism (code path)
`insert_block` (latexml_engine/src/base_utilities.rs:3927 = Perl `insertBlock`, TeX_Box.pool.ltxml)
handles every \parbox/minipage/box capture: opens `<ltx:_CaptureBlock_>` with the box attrs
(class/width/vattach), `document.absorb(contents)` (:3996), collects nodes,
`close_to_node(&container,true)` + `close_node(&container)` (:3999-4000), then repackages the
capture — unwrap single child / merge attrs (:4020-4076) / rename to a block candidate from
[block,logical-block,sectional-block,figure] or inline-block set (:4110-4148), else Warn
"Did not find a block-like candidate" + rename to ltx:block (:4145).

- A error #1 `<ltx:listingline> isn't allowed in <ltx:p>`: fires DURING absorb, in
  `find_insertion_point_qsym` (latexml_core/src/document.rs:2973). can_contain(p,listingline)=F;
  can_contain_indirect=none; the auto-close loop (:3019 `while can_auto_close(node) && …`) stops
  at the non-auto-closeable `<ltx:p>`/inline-block/`<ltx:listingline>`; only the ancestor
  `<ltx:listing>` can host a listingline (schema). No candidate → Error at :3203, then "does it
  anyway" (returns self.node → inserts listingline in the p).
- B error `_CaptureBlock_ Closing … descendents do not auto-close. Descendants: verbatim/listingline`:
  fires when insert_block closes the capture — `close_to_node` (document.rs:~1407) /
  `close_node_with_strictness` (:1476) walk `self.node`→container collecting `!can_auto_close`
  nodes into `cant_close`; non-empty → Error (:1449 / :1508); then close_node_internal closes
  ANYWAY. can_auto_close (document.rs) = text/comment, or element w/o `_noautoclose` and with
  `_autoclose` or model autoClose=true; verbatim/listingline have neither.

### Schema (LaTeXML.model)
- `ltx:listing` (a Block) contains ONLY `ltx:listingline`. `ltx:listingline` is contained ONLY
  by `ltx:listing`; allows inline content only (no block, no listing, no p).
- `ltx:p`/`ltx:inline-block` allow inline/Misc — NOT listing/listingline.
- `ltx:verbatim` is in `Misc` (allowed inline: in p/text/inline-block/listingline).
- `ltx:_CaptureBlock_` is a transient wrapper, always renamed by insert_block.

### What Perl does (same-host, BOTH repros) — EXACT PARITY
Perl emits the IDENTICAL errors AND produces a STRUCTURALLY IDENTICAL tree:
- A: `<para class=ltx_minipage><listing><listingline l1><inline-block class=ltx_minipage>
  <listingline l2>…</listingline></inline-block></listingline><listingline l3>…` — i.e. l2 is
  NESTED inside l1's inline-block, not a sibling. Both engines. (Only cosmetic diffs: width em vs pt.)
- B: `<inline-block><inline-block><inline-block vattach=bottom><verbatim>…</verbatim>` — verbatim
  correctly inside the box in BOTH; only Perl leaks a `\verbbox@inner` literal + an extra
  `ltx_nopad_l` class. The insert/close "happens anyway" in both.
Divergence entry: "Perl emits the same close/placement diagnostics and builds the same tree; the
box boundary legitimately force-closes, so the diagnostic is spurious — Rust drops it (surpass on
error count, identical structure)."

### The rule
B (close rule — SAFE surpass, LOW risk). A `_CaptureBlock_` (and the box it becomes) is a HARD box
boundary: tex.web box completeness means nothing stays open across a completed box, so any
non-auto-closeable descendant is force-closed by the box, not an error. Rule: when the close TARGET
`node` is `ltx:_CaptureBlock_` (i.e. the close originates from insert_block:3999-4000), treat
intervening descendants as force-closeable — do NOT accumulate `cant_close` / do NOT emit the
"descendents do not auto-close" Error (close_node_internal already closes them). Implement in
`close_to_node` + `close_node_with_strictness` (document.rs): `if get_node_qname(node)=="ltx:_CaptureBlock_"
{ /* suppress cant_close error */ }`, OR have insert_block mark the container so the close site
recognises the box boundary. Faithful mechanism = box completeness (tex.web §640-ish \vpack/\hpack:
a box's contents are finished when the box is). Clears B (numberedblock 2→0) AND the `_CaptureBlock_
Closing` half of algpseudocodex (2→1) and coloredtheorem (~2 of 6).

A (placement, error #1) — NOT a clean document-model rule; RISKIER. Root: the algorithmicx line box
(inner `_CaptureBlock_`→inline-block) fails to close between `\State`/`\Statex` lines when the
algorithmic is inside a minipage — the algpseudocode `\everypar`-on-hmode handoff item, NOT the
document model. Options: (a) fix the algpseudocode/algorithmicx binding to close each line box
before the next listingline (real root; un-nests l2 to a sibling → surpasses Perl); (b) document
relocation — when `<ltx:listingline>` arrives and an ancestor `<ltx:listing>` can host it past box
artifacts, force-close to the listing and open as sibling (surpasses Perl) — RISK MED/HIGH: would
truncate a legitimately-open `\parbox` inside a line. RECOMMEND (a) via the algpseudocode binding;
do NOT pursue (b). B's fix already halves algpseudocodex without touching A.

### Regression guards (capture-path bar to run green)
- `listing_sole_content_of_minipage_keeps_lstlisting_class` — cluster_sizing.rs:861
  (fixture cluster_regressions/listing_in_minipage_keeps_class.tex): single-node unwrap + class MERGE.
- `parbox_nested_math_converts_to_presentation_mathml` — cluster_sizing.rs:655.
- `hphantom_braceless_minipage_does_not_swallow_endminipage` — 06_cluster_regressions.rs:1175.
- `memoir_keeps_native_endminipage` — cluster_package_guards.rs:6260.
- `fancybox_verbatim_layer_raw` — cluster_package_guards.rs:4507 (verbatim-in-box; closest to B).

### Guard assertions
- B (numberedblock_verbatim_capture.tex): 0 Error lines AND `count(//ltx:verbatim)=1` inside an
  `<ltx:inline-block>` (verbatim survives the box close); no `_CaptureBlock_` string in the output.
- A (listingline_in_minipage.tex): with the algpseudocode binding fix — 0 Error lines AND
  `count(//ltx:listing/ltx:listingline)>=2` with `count(//ltx:listingline//ltx:listingline)=0`
  (listinglines are SIBLINGS, not nested). With B-only fix: assert exactly 1 Error remains (the
  placement), i.e. the `_CaptureBlock_ Closing` second error is gone.

## Group C — pagelayout unmatched </ltx:picture> — SHARED — SPEC

Repro: pagelayout_picture_close.tex (RED: <ltx:picture> Attempt to close, isn't open;
pdflatex 0 !; same-host Perl emits the identical picture-close error + a Perl-only
\Gin@draftfalse undefined).

### (1) standalone multi=picture — how it's bound
standalone.cls:190-191 `\sa@clsoption{multi}[true]{... \AtBeginDocument{\standaloneenv{#1}}}`
→ `\standaloneenv{picture}` (:523) → `\@standaloneenv{picture}` (:629) which redefines the
`picture` env to wrap it with `\preview\sa@varwidth` … `\endpreview` (keeping the ORIGINAL
picture open/close). Rust standalone_cls.rs raw-loads the class (InputDefinitions noltxml) and
NEUTRALISES the wrapper: `def_macro_noop("\\@standaloneenv{}")` (standalone_cls.rs:14) — so
`picture` stays the plain LaTeX env. Perl standalone.cls.ltxml:20 raw-loads too. => the
multi=picture wrapper is a NO-OP here; it is NOT the source of the open/close imbalance.

### (2) where the picture is opened/closed — pagelayout's OWN renderer
pagelayout.cls (raw, no binding). `\template[o]{name}{body}` (:3639) → `\pal@rendertemplate`
(:3627) → `\page[o]{...}` (:3080) → `\pal@standardpage[o]{body}` (:3099). `\pal@standardpage`
opens `\begin{picture}(\paperwidth,\paperheight)` (:3115) as a FULL-PAGE canvas, inserts the
page content `#2` + `\pal@putgrid`/`\pal@putbleed`/… (each a `\put{...\begin{tikzpicture}...}`),
then `\end{picture}` (:3153). The `\text{..}` placeholder becomes a `\put`-positioned pgfpicture
containing a `\minipage` (block flow). LaTeXML's `<ltx:picture>` is an inline/Misc element that
cannot hold block flow, so it auto-closes the picture to place the `<ltx:p>`/inline-block; the
`\begin`/`\end{picture}` nesting desyncs and the `\end{picture}` at :3153 closes a picture that
is no longer open → Error at latexml_core/src/document.rs:1298 (close_element, "isn't open"),
Currently in #document. The close is pagelayout's OWN `\end{picture}`, NOT standalone's wrapper.
Rust splits into several `<picture>` (p1.pic1 empty self-close, p2, p3 with the minipage/text);
Perl captures the whole body into one `<picture tex="\begin{picture}…\end{picture}">` + siblings —
both imbalanced, both emit the close error. Text present in both.

### (3) fix class — (a) a pagelayout binding (NOT a standalone gap)
Standalone's multi= wrapper is already neutralised and is not involved. The picture is opened and
closed by pagelayout's `\pal@standardpage`, and the imbalance is intrinsic: LaTeXML has no page
layout, so a full-page `picture` canvas hosting block flow cannot round-trip. Faithful fix = a
`latexml_package/src/package/pagelayout_cls.rs` binding that raw-loads the class (InputDefinitions,
like standalone_cls.rs) then overrides the page renderer to DROP the picture canvas and emit the
template content as block flow:
  - redefine `\page[]{}` / `\pal@standardpage[]{}` (+ `\pal@doublepage`/`\pal@frontcover`/
    `\pal@backcover`) to digest `#2` as block flow WITHOUT `\begin{picture}`/`\end{picture}`, OR
  - redefine `\template[]{}{}`→ digest `#3` directly and `\text{}`→ its arg as a paragraph,
    bypassing `\page`.
The picture canvas is pure page positioning (semantically empty for LaTeXML); the `\text{...}`
placeholder text + any real graphics (tikz→svg) are the content to keep. MED risk (deep class, 2
docs: example-template, example-text; the pagelayout-manual*/quickstart/example-* others may reach
the same path — re-verify). Precedent: eso-pic/background page-canvas packages are similarly reduced.

### (4) guard assertions
0 Error lines AND the placeholder text present in a block:
`//ltx:p[contains(.,'generic template')]` (or the template body as flow), AND no
"<ltx:picture> Attempt to close" error. Structural: the template content is NOT lost and no stray
`<ltx:picture>` close remains.

Dead end: treating it as a standalone multi=picture gap — the wrapper is a no-op (standalone_cls.rs:14);
the open/close is pagelayout's `\pal@standardpage`, so a standalone-side change would not help.

## Group A REAL CAUSE — algpseudocodex \Statex leaves the code box open — SHARED — SPEC

Repro: algpseudocodex_statex_line.tex (RED 2 err with b54x; pdflatex 0 !; same-host Perl
identical 2 errors + identical nested tree). CORRECTED trigger: `\State` FOLLOWED BY `\Statex`
— NOT minipage-dependent (the doc's minipage is incidental; a bare algorithmic reproduces it).
Controls that stay CLEAN: `\State`+`\State` (sibling listinglines), `\Statex` alone,
`lstlisting` in a minipage.

### (1) how a line is opened / closed (binding)
algorithmicx_sty.rs: `\algorithmic`→`\lx@setup@algorithmicx` re-lets `\item`→`\lx@algorithmicx@item`
(:65). `\lx@algorithmicx@item[]`→`\@ifnextchar\nointerlineskip{}{\lx@algorithmicx@@item}` (:70).
`\lx@algorithmicx@@item` (:90) opens `<ltx:listingline>`; its `before_construct`
`maybe_close_element("ltx:listingline")` (:114) closes the PRIOR line. `\lx@algorithmicx@endlist`
(:76) closes the last line then `</ltx:listing>`. `maybe_close_element` (document.rs:1389) →
`is_closeable` → closes ONLY if every intervening node auto-closes; else NO-OP.

### (2) algorithmicx.sty line ending + why it fails
`\State` = `\algdef{SL}[STATE]{State}{0}{}` (algorithmicx.sty:577); `\Statex` = `\item[]` (:632).
algpseudocodex wraps EACH line's content in `\begin{varwidth}[t]{…}` (algpseudocodex.sty:185),
which LaTeXML captures (insert_block) as `<ltx:inline-block class=ltx_minipage>`. The box is
closed by `\algpx@endCodeCommand` = `\end{varwidth}…` (algpseudocodex.sty:192), `\pretocmd` to
`\State \While \For \ForAll \Loop \Repeat \Until \If …` (:782-…). `\Statex` is NOT in that list,
so after a `\State` its varwidth box stays OPEN when `\Statex` opens the next line. In real TeX
the list `\item` implicitly ends the box (\par/\@item) → pdflatex clean; in LaTeXML the inline-block
is a live element and the non-strict `maybe_close_element("ltx:listingline")` can't close through
it, so line 2 opens inside line 1's box `<ltx:p>` → "listingline isn't allowed in <ltx:p>", and the
enclosing capture close then reports the open listingline descendant. Perl identical (SHARED); the
nested tree is byte-for-byte the same (line 1's varwidth box holds line 1's text AND line 2).

### (3) the fix (binding) + CONTROL
Fix in algorithmicx_sty.rs, `\lx@algorithmicx@@item` (and `\lx@algorithmicx@endlist`) before_construct:
replace the non-strict `maybe_close_element("ltx:listingline")` with "return to the enclosing
`ltx:listing`" — locate the ancestor `ltx:listing` and close all its open descendants (the leftover
per-line `<ltx:inline-block>` code box + the prior `<ltx:listingline>`) via `close_to_node(listing,true)`,
so every new line opens as a DIRECT child of `ltx:listing`. This is the binding analogue of the
package's own end-of-line (`\algpx@endCodeCommand`/`\ALG@endline`): a new algorithmic line always
ends the previous line. The force-close of the per-line box must be SILENT (a new line is a hard
line boundary) — mirror the Group B `_CaptureBlock_` box-completeness rule, or mark the varwidth
code box `_autoclose`. CONTROL preserved: for `\State`-opened lines the box is ALREADY closed by
`\algpx@endCodeCommand`, so "return to listing" == the current `maybe_close_element("ltx:listingline")`
(no output change; two-`\State` and non-algpseudocodex algorithmic stay identical, sibling lines).

### (4) guard assertions
algpseudocodex_statex_line.tex (and listingline_in_minipage.tex): 0 Error lines AND
`count(//ltx:listing/ltx:listingline) = 2` with `count(//ltx:listingline//ltx:listingline) = 0`
(the two lines are SIBLINGS, not nested). CONTROL guard: a plain `\State`+`\State` algorithmic
still yields two sibling `<ltx:listingline>` with 0 errors (unchanged).

Dead ends:
- "minipage triggers it" — WRONG: bare `\State`+`\Statex` reproduces (per-line varwidth box, not the
  user minipage, is the box). The minipage in the doc is incidental.
- Patching `\Statex` with `\algpx@endCodeCommand` in the binding — `\algpx@endCodeCommand` is
  algpseudocodex-only (absent in plain algorithmicx); fix belongs at the package-agnostic line-open.

## FAMILY — class frontmatter internals reached by raw classes/packages

Top-3 repros (by errors): frontmatter_ifbeamertemplateempty.tex (beamer, 43),
frontmatter_authorgroup.tex (quantumview, 8), frontmatter_amsart_internals.tex (resphil, 4).
All verified RED with b54x; same-host Perl run for classification.

### GROUP beamer_cls.rs — \ifbeamertemplateempty — SHARED (43 err, 1 doc) — TOP
Doc: beamer-theme-albi-doc. Caller: beamerthemeAlbi.sty:224,301,689
`\ifbeamertemplateempty{name}{empty}{nonempty}`. Real def beamerbasetemplates.sty:26
`\def\ifbeamertemplateempty#1#2#3{\def\beamer@ifdo{#3}\expandafter\ifx\csname beamer@@tmpl@#1\endcsname\relax\def\beamer@ifdo{#2}\fi\expandafter\ifx\csname beamer@@tmpl@#1\endcsname\beamer@@empty\def\beamer@ifdo{#2}\fi\beamer@ifdo}` —
a CONTROL-FLOW gate: run #2 if template `\beamer@@tmpl@<name>` is undefined or empty, else #3.
beamer_cls.rs stands in for beamer but omits this beamerbasetemplates internal. Perl beamer.cls.ltxml
also omits it → SHARED (Perl 3 err on the repro).
FIX (beamer_cls.rs): define `\ifbeamertemplateempty` with the REAL body (RawTeX/DefMacro, 3 args) —
it gates flow, so a real body is required (not a no-op); ensure `\beamer@@empty` exists (beamer's
`\def\beamer@@empty{}`). Guard: 0 err AND the frame body present (`//ltx:p` or a frame element),
no `\fi` cascade.

### GROUP ams_support_sty.rs — \author@andify / \@dedicatory / \@setabstract — RUST-ONLY (4 err, 1 doc)
Doc: rpsample (resphilosophica). resphilosophica.cls:75 \LoadClass{amsart} -> amsart_cls.rs binding
(loads ams_core, high-level `\lx@add@*` frontmatter; `\dedicatory`->`\lx@add@contact` at
ams_support_sty.rs:126). resphilosophica then RAW-redefines \maketitle/\@maketitle (resphil.cls:331,352)
which reach amsart's raw \maketitle internals — \author@andify\authors (:323), \ifx\@empty\@dedicatory /
\@dedicatory (:358-361), \@setabstract (:259,364). Real amsart.cls defs: \author@andify :803 (andify
the \authors list), \let\@dedicatory=\@empty :552, \@setabstract :856. The Rust binding omits all three.
Perl gives 0 err (RUST-ONLY) — Perl also lacks the internals, so Perl's amsart/\maketitle path must not
reach resphil's raw layout (locked/no-op maketitle); either way Rust must reach 0.
FIX (ams_support_sty.rs, beside \dedicatory:126) — these are LAYOUT of already-captured frontmatter,
so init/no-op:
  Let!("\\@dedicatory", "\\@empty");    // amsart.cls:552 — \ifx\@empty\@dedicatory then takes empty branch
  DefMacro!("\\author@andify", "");      // amsart.cls:803 — no-op; authors captured by \lx@add@creator
  DefMacro!("\\@setabstract", "");       // amsart.cls:856 — non-relax no-op; resphil:259 \ifx\@setabstract\relax
                                         //   is then FALSE so \@setabstracta is not needed; :364 no-op
Frontmatter still emitted by the binding's \lx@add@* (author/dedicatory/abstract captured at their call
sites); the raw \maketitle internals run inert (no duplication). Guard: 0 err AND
//ltx:creator (author) present AND //ltx:abstract present, no double-render.

### GROUP quantumarticle (quantumview) — \@authorgroup — SHARED (8 err, 1 doc)
Doc: quantumview-template (class quantumview, RAW, self-contained — derived from quantumarticle, NOT
\LoadClass). quantumview.cls:661 \renewcommand{\author}[2][]{…\internal@elseauthor{#1}{#2}}; :673-680
\internal@elseauthor inits the author-group list `\ifcsdef{@authorgroup}{}{\csdef{@authorgroup}{}}` then
`\listxadd{\@authorgroup}{\the@authorcounter}`, consumed by \maketitle's affiliation loop
`\forlistloop{..}{\@authorgroup}` (quantumarticle.cls:1169). RED root: the etoolbox init `\csdef{@authorgroup}{}`
does not leave `\@authorgroup` defined for `\listxadd`/`\forlistloop` (the `\csname author…\@authorgroup…
\endcsname` cascade). Perl fails WORSE (33 err) → SHARED. This is an etoolbox author-LIST emulation gap
(`\csdef`/`\ifcsdef`/`\listxadd`/`\forlistloop` on `\@authorgroup`/`\@authors`), NOT a single missing
internal — needs an etoolbox-interaction fix (ensure `\csdef{cs}{}` defines `\cs` so `\listxadd{\cs}` works),
or a quantumview shim initializing `\@authorgroup`/`\@authors` to empty lists. Lower priority; deeper than
the other two. Guard: 0 err AND //ltx:creator present.

### jourcl \@abstract — UNDER-INVESTIGATED (4 err, 1 doc)
jourcl.cls:240 \def\abstract#1{\def\@abstract{#1}}, :241 \newcommand{\pabstract}{\@abstract}; doc uses
\abstract{\lipsum[1]} (preamble) then \pabstract (body). Minimal repro (jourcl + \abstract{..} + \pabstract)
is CLEAN in Rust — the doc's failure (\@abstract undefined + \else Extra cascades, log:222-247) has a trigger
I could not isolate in <15 lines. Note for follow-up: likely \@abstract used in jourcl's cover-letter/\ifempty
machinery before \abstract{} runs, or \abstract shadowed. Faithful direction: init \let\@abstract\@empty.

## quantumview \@authorgroup — ROOT = locked \author, NOT an etoolbox bug — SHARED (8 err, 1 doc)

Repros: frontmatter_authorgroup_locked_author.tex (RED), etoolbox_list_primitives.tex (GREEN guard).

Investigation result: the etoolbox list primitives are CORRECT. Verified in isolation vs pdflatex
(all 0-err, matching output): \csdef, \ifcsdef, \csundef, \listadd, \listgadd, \listxadd, \listcsadd,
\listcsgadd, \forlistloop, \ifinlist — including \csdef{name}{} then \listxadd{\name} then \forlistloop
(etoolbox.sty:877 \csdef, :1675 \listadd, :1683 \listxadd, :1690 \listcsadd, :1725 \ifinlist). The
isolation doc etb_qview.tex that reproduces the failure differs by ONE thing: it goes through
\renewcommand{\author}.

REAL ROOT: the LaTeX kernel `\author` is `locked => true` in BOTH engines
(latexml_engine/src/latex_constructs/sect05.rs:737-739
`DefMacro!("\\author[]{}", r"\def\@shortauthor…\lx@add@authors…", locked => true)`; Perl
latex_constructs.pool.ltxml:1079 `locked => 1`). sect05.rs:736 even comments "our \author is locked so
renewcommand can't add it." quantumview.cls:661 `\renewcommand{\author}[2][]{…\internal@elseauthor…}`
therefore does NOT take effect, so `\internal@elseauthor` (quantumview.cls:673-680) — which runs
`\ifcsdef{@authorgroup}{}{\csdef{@authorgroup}{}}` + `\listxadd{\@authorgroup}{…}` — NEVER runs, and the
raw `\maketitle` affiliation loop `\forlistloop{\@addaffiliation{…}}{\@authorgroup}`
(quantumarticle.cls:1169) hits an undefined `\@authorgroup`. Proof: r_renew/r_def show `\renewcommand{\author}`
/`\def\author` are inert; q_a/q_c/q_d show the etoolbox machinery works when reached directly.
Perl fails WORSE (33 err) because its `\author` is likewise locked → SHARED, by design. pdflatex clean.

FIX (validated, class-level — NOT etoolbox, NOT unlocking \author): unlocking `\author` is unsafe (the
lock deliberately protects the frontmatter capture in both engines). Instead a `quantumview_cls.rs`
binding (InputDefinitions the raw quantumview.cls) initializes the author-group lists empty so the raw
`\maketitle` loop is a safe no-op, while the binding's locked `\author`->`\lx@add@creator` captures the
authors. Validated: q_fix.tex = quantumview + `\csdef{@authorgroup}{}\csdef{@authors}{}` before \maketitle
→ 0 errors, 3 `<ltx:creator>` still emitted. So the binding load should:
  TeX!(r"\makeatletter \csdef{@authorgroup}{} \csdef{@authors}{} \makeatother");  // or Let! to \@empty
(quantumarticle proper shares the machinery; if a doc uses `\documentclass{quantumarticle}` with the same
author-group path, the same init belongs in quantumarticle_cls.rs.)
Guard: etoolbox_list_primitives.tex stays GREEN (etoolbox correctness regression bar), AND
frontmatter_authorgroup_locked_author.tex → 0 errors with //ltx:creator present.

Dead end: an etoolbox `\csdef`/`\listxadd`/`\forlistloop` fix — the primitives are correct (guard proves it);
the divergence is entirely the locked-`\author` override resistance.

## FAMILY — singleton undefined class/package internals (survey, grouped by binding)

Each 1 doc unless noted. "role": control-flow (needs a real gating body) vs layout (no-op/render).

### GROUP doclicense_sty.rs — \doclicenseThis — binding gap (beautynote, 6 err) — CONFIRMED RED
Repro: singleton_doclicenseThis.tex. doclicense.sty:222 `\newcommand{\doclicenseThis}{…\begin{center}
\begin{minipage}…\doclicenseImage…}` — LAYOUT (renders the license block). doclicense_sty.rs binding
omits it. FIX: define `\doclicenseThis` in doclicense_sty.rs to emit the license text/reference (or a
faithful port of the minipage body). Guard: 0 err AND the license text present in output.

### GROUP subfiles_sty.rs — \ifSubfilesClassLoaded — binding gap (sshrc-insight, 4 err) — CONFIRMED RED
Repro: singleton_ifSubfilesClassLoaded.tex. subfiles.sty:171 `\newcommand\ifSubfilesClassLoaded{%
\expandafter\ifx\csname ver@subfiles.cls\endcsname\relax\@secondoftwo\else\@firstoftwo\fi}` — CONTROL
FLOW (is subfiles the document CLASS?). subfiles_sty.rs redefines \documentclass to no-op (subfiles_sty.rs:4)
but omits this test. FIX: define `\ifSubfilesClassLoaded` in subfiles_sty.rs with the real body
(`\@ifundefined{ver@subfiles.cls}\@secondoftwo\@firstoftwo`); when subfiles is used as a PACKAGE (not the
class) it takes the second branch. Guard: 0 err AND the second-branch text present.

### GROUP listings_sty.rs — \lst@XConvert — binding gap (tagpdf, 6 err)
listings.sty:211 `\def\lst@XConvert{\@ifnextchar\bgroup\lst@XConvertArg\lst@XConvert@}` (+ :212
\lst@XConvertArg) — CONTROL FLOW (a delimited character-conversion chain, `#1\@nil`). listings_sty.rs
emulation omits it. tagpdf reaches it. FIX: port the `\lst@XConvert`/`\lst@XConvertArg`/`\lst@XConvert@`
delimited chain into listings_sty.rs (real body — it drives listing char conversion). Guard: 0 err.

### GROUP hyperref_sty.rs — \Hy@driver / \hyper@makecurrent / \HyPsd@UTFviii — emulation gaps (3 docs)
hyperref_sty.rs is an EMULATION (InputDefinitions commented out, :73). Internals reached by raw
hyperref-dependent packages:
 - \Hy@driver — hrefhide.sty:154 `\ifx\Hy@driver\hrefhide@driver\relax` — CONTROL FLOW (driver-identity
   check). Define `\Hy@driver` to a stable sentinel so the \ifx is well-defined (else branch → hrefhide
   warns, continues). hrefhide, 9 err. (A bare `\ifx\Hy@driver\relax` is already clean in Rust; the real
   trigger is the compare vs \hrefhide@driver — verify.)
 - \HyPsd@UTFviii — dvdcoll/pdfnotiz.sty:263 — hyperref PDF-STRING UTF8 handler; CONTROL FLOW inside
   pdfstring encoding. Define as a stub/no-op (pdfstrings are metadata). dvdcoll, 4 err.
 - \hyper@makecurrent — ucalgmthesis, 5 err — hyperref anchor-current internal; CONTROL FLOW. Stub.
FIX: add these hyperref internals to hyperref_sty.rs (sentinel \Hy@driver + no-op pdfstring/anchor
internals). Guard per doc: 0 err. (Group is version-sensitive — port only the reached internals.)

### GROUP kernel/class-error — \ClassErrorNoLine — (asmeconf, asmejour; 2 docs, 2 err each)
asmeconf.cls:650-653 `\IfFontExistsTF{…otf}{}{\ClassErrorNoLine{\ClassName}{\FontWarning}}` — the
font-MISSING else-branch. asmeconf/asmejour do NOT define \ClassErrorNoLine; it fires only when
\IfFontExistsTF returns false. ROOT: Rust's \IfFontExistsTF returns false (fonts "not found") so the
else-branch runs — same class as \TBWarning/\@currsize font-detection. FIX (primary): \IfFontExistsTF
should detect texmf .otf fonts → true → branch never taken. FIX (fallback): define \ClassErrorNoLine as a
\ClassError-style stub (real error macro without the line number). Guard: 0 err (no spurious class error).

### PARKED / DEFERRED (out of this family's scope)
 - \Gm@lmargin (geometry, stocksize) — PAGE LAYOUT (geometry.sty:137 \let\Gm@lmargin\@undefined; computed
   during \geometry layout). SKIP (page-layout parked).
 - \pIIe@mode (bxeepic) — pict2e GRAPHICS DRIVER mode (p2e-*.def). Graphics-driver detection; defer.
 - \ekv@stop (expkv-bundle) — expkv VM DELIMITER marker (expkv.tex:105 `#1\ekv@stop`); expl3-ish gullet
   quark, not a missing def. Defer.
 - \headlessfullcite (biblatex-chicago, lualatex) — biblatex .cbx citation command (chicago-notes.cbx);
   biblatex is config-driven / out of scope. Defer.
 - \diffd (diffcoeff) — generated by diffcoeff's expl3 \diffdef machinery (diffcoeff.sty:924); if the
   expl3 generation doesn't run, \diffd isn't created. diffcoeff-expl3 issue; defer.
 - \IfEq (heria) — numeric compare (xstring \IfEq / fragoli); heria doesn't visibly load xstring — its own
   prerequisite. Needs which package supplies \IfEq. Defer.
 - \mathhyphen (rec-thy, lualatex) — math-mode hyphen; defined in cryptocode (NOT loaded by rec-thy, which
   loads pict2e/xifthen). rec-thy's own prerequisite / a math-symbol the binding could provide. Defer.
 - \nmid (rbt-mathnotes) — ALREADY defined in amssymb_sty.rs:521 (`def_math_sym("\\nmid",…)`). Not a binding
   gap: rbt-mathnotes must not be loading amssymb (or a load-order/unicode-math path). Verify the doc's
   amssymb load; likely a rbt-mathnotes prerequisite, not a Rust fix. Defer.
 - \pkgname (labyrinth, lualatex, 13 err) — the DOC's own convenience macro (labyrinth.tex:41,63 uses
   \pkgname{labyrinth}) with no \def in the doc/sty — a doc-support prerequisite (ltxdoc-style). Raw doc's
   own missing def; defer.

Actionable this family (by value): doclicense \doclicenseThis (6), listings \lst@XConvert (6), hyperref
group (9+5+4), subfiles \ifSubfilesClassLoaded (4), \ClassErrorNoLine (2 docs). All binding gaps or
font-detection; the rest are parked/raw-prerequisite/expl3.

## FOLLOW-UP 1 — faithful bodies for the actionable singleton gaps (one block per binding)

subfiles_sty.rs — \ifSubfilesClassLoaded (real body, control flow):
  DefMacro!("\\ifSubfilesClassLoaded", r"\@ifundefined{ver@subfiles.cls}\@secondoftwo\@firstoftwo");
  (subfiles.sty:171; when subfiles is a PACKAGE, ver@subfiles.cls is undefined → second branch.)

listings_sty.rs — \lst@XConvert: the chain IS reached (tagpdfdocu-patches.sty:65-69 defines
  \lstrenewenvironment via \lst@UserCommand(=\gdef) with body `\let\lst@arg\@empty \lst@XConvert{#1}\@nil
  \expandafter\lstnewenvironment@\lst@arg{#1}{#2}`). It ONLY NEEDS TO EXIST + consume its delimiter — the
  real case-conversion is pointless for LaTeXML, and `\lstnewenvironment@` must stay a NO-OP so the patch
  does NOT rebind lstlisting to latex-lab blockenv (the reason listings_sty.rs:2404 omits it). VERIFIED
  minimal fix (0 err):
    RawTeX!(r"\def\lst@XConvert#1\@nil{}\long\def\lstnewenvironment@#1#2#3{}");
  (Do NOT port the real listings.sty:211-236 chain — it would feed the blockenv rebind the comment warns of.)

doclicense_sty.rs (contrib, currently a 16-line stub that no-ops ALL sub-macros incl. \doclicenseText/
  \doclicenseLongText/\doclicenseImage/\doclicenseURL/\doclicenseName) — \doclicenseThis is the only
  missing wrapper. Consistent minimal fix (VERIFIED 0 err): def_macro_noop("\\doclicenseThis")?;
  (doclicense.sty:222 body is pure LAYOUT — center + minipages of \doclicenseImage + \doclicenseLongText,
  all already no-op'd. SURPASS option: un-stub \doclicenseText/\doclicenseLongText/\doclicenseURL/
  \doclicenseName to emit `\href{url}{name}` and make \doclicenseThis → \doclicenseLongText; larger change.)

hyperref_sty.rs (emulation) — three internals reached by raw hyperref-dependent packages:
  - \Hy@driver: SENTINEL = "hpdftex". hrefhide.sty:154 `\def\hrefhide@driver{hpdftex}\ifx\Hy@driver
    \hrefhide@driver\relax\else\PackageError…`; real hyperref.sty:2555 `\def\Hy@driver{hpdftex}` under
    pdfTeX. Fix: DefMacro!("\\Hy@driver", "hpdftex"); → the \ifx matches → no driver error. VERIFIED (the
    \Hy@driver error clears; a residual `ocgcolorlinks` note is a separate hrefhide option in the real doc).
  - \HyPsd@UTFviii: hyperref.sty:1788 — PDF-string UTF8 octet setup (bookmark encoding). LaTeXML emits no
    pdfstrings → def_macro_noop("\\HyPsd@UTFviii")?;
  - \hyper@makecurrent: hyperref.sty:6832 `\def\hyper@makecurrent#1{…}` — builds the current anchor name;
    LaTeXML uses its native id/label system → def_macro_noop("\\hyper@makecurrent{}")?;

## FOLLOW-UP 2 — \ClassErrorNoLine root = \IfFontExistsTF always-false

Docs: asmeconf-template, asmejour-template (oracle=lualatex, run with the luatex preload → \ifac@fontspec
TRUE → the fontspec font-check branch runs). asmeconf.cls:650-655
  `\IfFontExistsTF{TexGyreTermesX-regular.otf}{}{\ClassErrorNoLine{\ClassName}{\FontWarning}}` (×4 fonts,
  + \ClassWarningNoLine ×2). Under pdflatex \ifac@fontspec is FALSE (asmeconf.cls:640-642 loads
  inconsolata/newtxmath instead), so the checks are never reached → pdflatex clean.

ROOT: Rust \IfFontExistsTF is hard-wired to the FALSE branch — fontspec_sty.rs:79
  `DefMacro!("\\IfFontExistsTF{}{}{}", "#3")` (comment :75-78: "No OpenType font resolves in this engine →
  false branch. Witnesses: neoschool{,-fr}, beamerthemeCelestia{,-fr}"). So every asmeconf font check takes
  the else branch → \ClassErrorNoLine, which is undefined (no real def anywhere — asmeconf assumes fonts
  present so it never fires under a full TL). Real \IfFontExistsTF = fontspec `\fontspec_font_if_exist:nTF`,
  a LuaTeX/XeTeX font lookup; the fonts ARE in TeX Live (kpsewhich resolves texgyretermes-math.otf,
  texgyreheros-regular.otf, Inconsolatazi4-Regular.otf, lmroman10-regular.otf; TexGyreTermesX-regular.otf is
  the one gap on this host).

FAITHFUL FIX (fontspec_sty.rs): replace the always-#3 with a real lookup via the existing kpathsea resolver
  find_file (latexml_core/src/binding/content.rs:3084, the same helper that resolves .sty) — return #2 if
  find_file(#1) resolves the font file in texmf, else #3. Apply to \IfFontExistsTF (:79) and
  \fontspec_font_if_exist:nTF (:151). fontspec accepts a filename (`Foo.otf`, asmeconf's case) — resolve as-is;
  for a bare family name, a fallback (append .otf/.ttf) is optional.
  ALSO define the missing kernel-ish error/warning macros so a genuinely-absent font degrades to a message
  (not "undefined CS"): \ClassErrorNoLine{}{}  → \ClassError-style (no line helptext); \ClassWarningNoLine{}{}
  → \ClassWarning-style. (These are legit `…NoLine` variants; asmeconf/asmejour + others reach them.)
RISK: MED — the always-false was DELIBERATE (4 witnesses neoschool{,-fr}, beamerthemeCelestia{,-fr} that
  chose the false/fallback path). A real lookup returns TRUE for TL-present fonts → those witnesses now take
  the TRUE branch; RE-VERIFY all four don't regress (they likely load a fallback on "missing" — confirm the
  found-font path is handled). If they regress, narrow the change (e.g. only for the `.otf`-filename form).
GUARDS: \IfFontExistsTF{lmroman10-regular.otf}{YES}{NO} → YES; {nonsense-font-xyz.otf}{YES}{NO} → NO;
  asmeconf repro (luatex preload) → the \ClassErrorNoLine root cleared for present fonts (0 font-check errors
  on full-TL CI). NOTE: asmeconf has ADDITIONAL separate roots (\setoperatorfont, \affiliation undefined) —
  the font fix clears only the font-check root; asmeconf reaches 0 total only once those are also fixed.

## FAMILY — Error:latex:(pkg) package errors (all oracle-clean → Rust-side gaps)

### TOP — "Unknown math version" — 5 docs, 36 err — SHARED, clean fix
Docs: objectz/ozguide (oz, 28), askmaps (sans, 3), iwonamath (iwonacondensed, 3), zed-csp/csp2e+zed2e
(zed, 1+1). Repros: mathversion_declared.tex (RED), mathversion_undeclared_control.tex (RED, must STAY).
Mechanism: `\DeclareMathVersion{}` = def_primitive_noop (latexml_engine/src/latex_constructs/sect08.rs:715)
— does NOT register the name; `\mathversion{}` (sect13.rs:1058-1067) accepts only bold/normal and errors
`Unknown math version '<x>'` otherwise. oz.sty:34 `\DeclareMathVersion{oz}` then :70 `\mathversion{oz}`;
iwonamath.sty:110 `\DeclareMathVersion{\l__iwonamath_versionname_tl}` (expl3 var — register the EXPANDED
name). Perl is identical (latex_constructs.pool.ltxml:2658 `\DeclareMathVersion` undef, :5290 errors) → SHARED.
FIX: `\DeclareMathVersion{name}` (sect08.rs:715) → register the (edef-expanded) name, e.g.
AssignMapping!("MATH_VERSIONS", name => true); `\mathversion{}` (sect13.rs:1058) → accept "bold"/"normal"
AND any name in MATH_VERSIONS (custom version = accept, no font change — LaTeXML has no custom math fonts);
only Error when the name is neither. Guard: mathversion_declared.tex → 0 err; CONTROL
mathversion_undeclared_control.tex (+ the existing mathversion_unknown_version_errors.tex `\mathversion{wobble}`)
→ still exactly 1 Error. Risk LOW. Gain 5 docs (objectz alone 28 err).

### scanpages \GenericError "Must be processed with pdf[la]tex!" — 1 err — SHARED-by-design (POLICY)
scanpages.sty:22-23 `\ifpdf\else\@latex@error{Must be processed with pdf[la]tex!}\@eha`. Rust
ifpdf_sty.rs:7 `DefConditional!("\\ifpdf")` is FALSE by design (comment "always false in LaTeXML"; Perl
`\newif\ifpdf\pdffalse`). So the \else error fires; Perl identical → SHARED. pdflatex clean (pdf mode true).
FIX = POLICY DECISION: LaTeXML emulates pdfTeX output, so \ifpdf arguably should be TRUE. Flipping it is a
broad change (every \ifpdf branch); flag for the surpass-Perl call, NOT a quick binding fix. tidyres's
\GenericError "Not in outer par mode" is a DIFFERENT root (a float/marginpar reached not in outer par mode —
mode-tracking; lualatex; 1 err) — separate.

### Verona "Command \sidegraphics ... defined only with the 'sidebar' option" — 4 err
beamerthemeVerona.sty:184 `\newcommand<>{\sidegraphics}[3][]{…}` (beamer OVERLAY-newcommand, sidebar branch)
vs :191 `\else\def\sidegraphics#1{\PackageError{Verona}{…defined only with the 'sidebar' option}}`. The doc
is beamer-verona-SIDEBAR (sidebar option ON) so :184 should run, but the FALLBACK (:192) fired → the sidebar
conditional was not set in Rust (theme-option not detected) OR `\newcommand<>` is unsupported so the real def
was lost. FIX: ensure the Verona sidebar theme-option sets its conditional (and/or beamer `\newcommand<>`
overlay-newcommand is honoured) so the real \sidegraphics is installed. Guard: beamer-verona-sidebar repro →
0 err, \sidegraphics not the PackageError stub. (beamer binding; verify \newcommand<> support.)

### colorspace "Unknown spot color" — 2 err — pdftex-primitive gap
colorspace.sty:38-41 `\def\spc@unknown#1#2{\@ifundefined{#1}{\PackageError{colorspace}{Unknown #2}…}{}}`;
colorspace's spot-color path uses pdftex primitives (`\pdffeedback colorstackinit` :35, `\pdfextension`).
When those are absent in Rust the spot-color CS isn't built → \spc@unknown → "Unknown spot color". colorspace
is raw (no binding). FIX: provide the pdftex color-stack primitives colorspace needs (or a colorspace binding
that defines \definespotcolor to register the color). Deeper; note. 1 doc.

### tikz/pgf — tikz-binding-deep, DEFER
zx-calculus "Giving up on this path. Did you forget a semicolon?" (42 err) — a tikz PATH-PARSE failure in the
pgf/tikz path grammar (zx-calculus uses heavy custom tikz). braids "No shape named `strands-3-s'" (4 err) —
braids defines custom pgf SHAPES the pgf binding doesn't register (\pgfdeclareshape path). Both are tikz/pgf
emulation gaps (shape registry / path grammar), high-effort; defer to a tikz-focused pass. All oracle-clean
(Rust gaps, not document errors).

## FOLLOW-UP (1) — Verona \sidegraphics = \usetheme option-drop, NOT \newcommand<> — SHARED (4 err)
Repro: verona_sidebar_option.tex (RED). beamer_cls.rs ALREADY implements the overlay definers
`\newcommand<>`/`\renewcommand<>`/`\newenvironment<>` (beamer_cls.rs:733-800, DefPrimitive
`\lx@beamer@defcmd@angle {}[][]{}`; handles `[3][]` with the "overlay dropped, default unsupported"
policy) — so `<>` is NOT the gap. The gap: `\usetheme[opts]{name}` (beamer_cls.rs:446) DROPS `_opts`
and `\ProcessOptionsBeamer` (:506) is a no-op. Verona:29/36 `\newif\ifbeamer@sidebar`/`\beamer@sidebarfalse`;
:43 `\DeclareOptionBeamer{sidebar}{\beamer@sidebartrue}`; :45 `\ProcessOptionsBeamer`; :174
`\ifbeamer@sidebar…\newcommand<>{\sidegraphics}[3][]{…}\else\def\sidegraphics#1{\PackageError{Verona}{…}}`.
Since the sidebar option never fires, `\ifbeamer@sidebar` is false → the fallback stub → the doc's 4
`\sidegraphics<1>{..}{..}` calls error. Perl no-ops `\usetheme` (beamer_cls.rs:516 note) → SHARED; pdflatex clean.
FIX (beamer_cls.rs): (a) `\usetheme[opts]{name}` → `\PassOptionsToPackage{opts}{beamertheme<name>}` before
`require_package`; (b) `\ProcessOptionsBeamer` → `\setkeys{\@currname}{<opts passed to this theme>}` (the
options are already routed to `\define@key{\@currname}` via `\DeclareOptionBeamer`→`\beamer@dokv`,
beamer_cls.rs:524). Then Verona's `sidebar` key fires `\beamer@sidebartrue` DURING theme load (before the
:174 branch), the real overlay `\sidegraphics` installs via the working `\newcommand<>`. Guard:
verona_sidebar_option.tex → 0 err AND \sidegraphics emits a graphics/tikz node (not "Package Verona Error");
CONTROL: no-sidebar Verona → \sidegraphics stub still errors on use (genuine). Risk MED (touches \usetheme
option routing for all themes — re-verify Albi/Berkeley/sidebar size-option docs).

## FOLLOW-UP (2) — tidyres "Not in outer par mode" = \ifinner wrong at main galley — SHARED (1 err)
Repro: ifinner_main_galley.tex (RED, plain \begin{paracol}). NOT a genuine float-in-box: tidyres \ressection
(tidyres.sty:82) → \begin{paracol}{2}; paracol.sty:1995-1996 `\def\pcol@zparacol[#1]#2{\par \ifinner\@parmoderr\fi…}`
GUARDS outer par mode (paracol drives the output routine, valid only outside a box). ROOT = Rust's mode state:
at the main document galley, AFTER `\par`, MODE is `internal_vertical` (an INNER mode), so `\ifinner`
(latexml_engine/src/tex_logic.rs:110 — true for restricted_horizontal|internal_vertical|math) is wrongly TRUE
→ `\@parmoderr` → the error (routed via base_utilities.rs:5552 make_generic_message). VERIFIED: `Main:\ifinner`
(horizontal, text before) → OUTER (correct); `\par\ifinner` at body top → INNER (wrong); inside `\parbox` → INNER
(correct, must STAY). LaTeXML does not distinguish TeX's OUTER main-vertical/horizontal galley from the INNER
internal-vertical/restricted-horizontal of a real box. Perl is IDENTICAL (TeX_Logic.pool.ltxml:127-128 same
regex; Perl errors on paracol too) → SHARED, pdflatex clean.
FIX (deep, mode-model): `\ifinner` must be FALSE at the top-level galley and TRUE only inside a real box
(\hbox/\vbox/\parbox/minipage/_CaptureBlock_) or nondisplay math. Two shapes: (a) track a box-capture depth
(hbox/vbox/parbox/minipage/insert_block push a frame) — `\ifinner` = depth>0 || mode==math; main galley depth 0
→ false. (b) give the main document body a distinct OUTER `vertical`/`horizontal` mode, reserving
internal_vertical/restricted_horizontal for boxes; `\ifinner` checks the latter. Risk MED-HIGH (mode-model
change; `\ifinner` has many callers — must keep box-interior true, e.g. isorot \@rotcaption, and the guard
mathversion/box tests). Guard: ifinner_main_galley.tex → 0 err; `\par\ifinner` at body top → OUTER;
`\parbox{}{...\ifinner...}` → INNER (unchanged). Unblocks every \begin{paracol} doc (tidyres + others).

## REPORT (1) — beamer \usetheme[opts]{name} option routing — LANDABLE SPEC (Verona, 4 err)
Real beamer: \usetheme[opts]{names} -> \beamer@calltheme{opts}{names}{beamertheme}
(beamerbasethemes.sty:18-23) = `\@for name:=names\do{\usepackage[{opts}]{beamertheme<name>}}` — the
options are ordinary PACKAGE options. \ProcessOptionsBeamer (beamerbaseoptions.sty:15-24) then does
`\edef\@tempa{}\@for\CurrentOption:=\@classoptionslist\do{\@ifundefined{KV@\@currname @\CurrentOption}{}
{\edef\@tempa{\@tempa,\CurrentOption,}}}\edef\@tempa{\noexpand\setkeys{\@currname}{\@tempa\@ptionlist{\@currname.\@currext}}}\@tempa`
— it EDEF-EXPANDS the passed option list + matching class options, then `\setkeys{\@currname}{…}`.
\DeclareOptionBeamer/\beamer@dokv/\ExecuteOptionsBeamer in Rust (beamer_cls.rs:524-526) already match
(`\define@key{\@currname}`/`\setkeys{\@currname}`).

Rust gaps (2): (i) \usetheme[]{} (beamer_cls.rs:446) + the four siblings (\usecolortheme/\usefonttheme/
\useinnertheme/\useoutertheme, :458/:472/:484/:494) all DROP `_opts`; (ii) \ProcessOptionsBeamer (:506) is a no-op.

FIX (beamer_cls.rs): (i) replace each `\use*theme` DefPrimitive with the real \beamer@calltheme routing so
opts flow as package options — RawTeX:
  \def\beamer@calltheme#1#2#3{\@for\beamer@themename:=#2\do{\usepackage[{#1}]{#3\beamer@themename}}}
  \newcommand*\usetheme[2][]{\beamer@calltheme{#1}{#2}{beamertheme}}   (+ 4 siblings with
  beamercolortheme/beamerfonttheme/beameroutertheme/beamerinnertheme). (Rust `\usepackage[opts]{pkg}` DOES
  populate `\@ptionlist{pkg.sty}` — VERIFIED: probe showed \@currname=beamerthemeVerona, \@currext=sty,
  \@ptionlist=sidebar.) (ii) replace \ProcessOptionsBeamer no-op with the real beamerbaseoptions.sty:15-24
  body ABOVE (the \edef expansion is REQUIRED — a bare `\setkeys{\@currname}{\@ptionlist{…}}` fails because
  \setkeys does not pre-expand \@ptionlist; VERIFIED: hardcoded `\setkeys{\@currname}{sidebar}` cleared the
  error, unexpanded \@ptionlist did not, the \edef body did).
VERIFIED end-to-end: \usetheme[sidebar]{Verona} + both fixes -> Verona `\sidegraphics` PackageError GONE
(the real overlay \sidegraphics installs via the already-working \newcommand<>). Residual on the Verona repro
is a SEPARATE parked pgf "No shape named `graphic'" (tikz remember-picture named node), not this root.
CONTROL: \usetheme{Albi} (size options via pgfkeys, no matching \DeclareOptionBeamer key) — IDENTICAL 1 err
(\ifbeamertemplateempty, a separate landing root) WITH and WITHOUT the fix -> unchanged. Perl no-ops \usetheme
(beamer_cls.rs:516) -> SHARED; pdflatex clean.
Guard: verona_sidebar_option.tex -> the "Command \sidegraphics ... 'sidebar' option" PackageError is gone
(count 'Command .sidegraphics' = 0); Albi guard (beamer_size_option_is_recorded, and an Albi convert) unchanged.
Risk MED (touches \usetheme routing for all themes; re-verify landed beamer witnesses: Albi, size options).

## REPORT (2) — \ifinner mode-model DESIGN (plan, not a patch) — SHARED
Sites that put the galley into a mode \ifinner mis-reads:
- Init: MODE=BOUND_MODE=`vertical` (stomach.rs:512-513; Perl Stomach.pm:48-49) — the PREAMBLE (outer).
- \begin{document}: `begin_mode_opt("internal_vertical", true)` (sect02.rs:54) = Perl
  `beginMode('internal_vertical',1)` (latex_constructs.pool.ltxml:314) — the BODY galley, internal_vertical
  WITHOUT a frame (noframe=true, level 0). This is the mode paracol.sty:1996 `\ifinner\@parmoderr` trips on.
- Boxes: \hbox/\vbox/\parbox/minipage/math call begin_mode WITH a frame (noframe=false, stomach.rs:987
  push_stack_frame). Plain `{...}` groups call bgroup()/push_stack_frame WITHOUT begin_mode (mode unchanged).
The MODE STRINGS do not encode tex.web §211's outer/inner SIGN: `internal_vertical` serves BOTH the main
galley (should be OUTER vmode) AND \vbox (INNER -vmode); `horizontal` serves BOTH the main galley AND box
interiors (should be restricted_horizontal / -hmode). VERIFIED both errors: main-galley VERTICAL (after \par)
-> \ifinner INNER (wrong; pdflatex OUTER); box-interior HORIZONTAL (\parbox{}{\par…}) -> \ifinner OUTER
(wrong; pdflatex INNER). Plain `{...}` group -> OUTER (correct, both). \ifinner (tex_logic.rs:110) and Perl
(TeX_Logic.pool.ltxml:127) share the identical MODE-string regex -> SHARED; pdflatex clean.

PLAN — Shape A (box-frame depth; RECOMMENDED, LOW-MED): add a "box/math nesting depth" that increments in
begin_mode_opt when `!noframe` sets a bound (inner) mode, decrements on the matching end_mode/egroup.
`\ifinner` := depth>0 (this IS tex.web §211's sign, encoded as a depth). Document body (noframe) -> depth 0 ->
OUTER (fixes paracol); \parbox/minipage/\hbox/\vbox/math (frame) -> depth>0 -> INNER (fixes box-interior);
plain `{...}` groups (no begin_mode) -> depth unchanged -> correct. Does NOT touch mode strings, box-interior
digestion, \par/paragraph handling, or the many `ends_with("vertical")` checks (stomach.rs:1755/1793,
leave_horizontal, FRAGMENT_YIELD) — only the \ifinner predicate (and optionally an \ifhmode/\ifvmode refine,
but those are already string-correct). Perl parity: same counter in Stomach.pm beginMode + TeX_Logic.pool.
The signal already half-exists: begin_mode's noframe flag + BOUND_MODE local-binding frame level
(`is_value_bound("BOUND_MODE", Some(0))`, stomach.rs) distinguishes the body's frameless bind from a box's
framed bind — a counter formalizes it.
Shape B (tex.web OUTER vertical/horizontal pair): give the body distinct outer `vertical`/`horizontal`,
reserve internal_vertical/restricted_horizontal for boxes; \ifinner checks the inner forms. Faithful to
§211 but HIGH risk: \begin{document} mode change + audit of every internal_vertical/ends_with("vertical")
site (paragraph build in the body relies on internal_vertical semantics; FRAGMENT_YIELD:1793 gates on it).
NOT recommended.
Guard (either shape): ifinner_main_galley.tex (paracol) -> 0 err; `\par\ifinner` at body top -> OUTER;
`{\par\ifinner}` group -> OUTER; `\parbox{}{\par\ifinner}` -> INNER; nondisplay `$\ifinner$` -> INNER.

## b55c suite re-run + shifted first errors

### repros.sh captions-floats --bin b55c — still-RED (real bugs), rest landed
Landed (rust 0): adjmulticol, caption_isorot_sidewaystable, caption_prepareslc, frontmatter_amsart_internals,
frontmatter_authorgroup(_locked), frontmatter_ifbeamertemplateempty, ifinner_main_galley, mathversion_declared,
ntheorem_thm_topsepadd, numberedblock_verbatim_capture, singleton_doclicenseThis, singleton_ifSubfilesClassLoaded,
threeparttable_caption_captype, titlesec_titlewidth. Still RED (as expected):
- algpseudocodex_statex_line (1) + listingline_in_minipage (1) — algpseudocodex line box (parked to the binding).
- caption_in_parbox_float (2) — rubik parbox-in-float capture (the deferred float_to_element-vs-capture root).
- pagelayout_picture_close (1) — pagelayout multi=picture standalone binding (deferred).
- verona_sidebar_option (4) — the Verona ROOT is FIXED (no more \sidegraphics stub); it SHIFTED to the parked
  pgf "No shape named `graphic'" (tikz remember-picture named node).
Intentional control (correctly RED): mathversion_undeclared_control (1, \mathversion{wobble}).
(jourcl has no repro file — never seeded.)

### Shifted first errors — NONE is a fancyhdr binding gap (the binding is complete)
- thesis-gwu \fancyhf: NOT fancyhdr. thesis-gwu.cls:146 `\input{required-packages}` FAILS — `required-packages.tex`
  is NOT shipped in the TL bundle (absent from the doc root and tex/; `kpsewhich required-packages.tex` = MISSING),
  so pdflatex `\input` would ALSO fail. Cascades to \fancyhf/\fancyfoot/\fancypagestyle/\newglossarystyle/\sodef/
  \cftchapnumwidth undefined (s36 log:26 "Can't find … 'required-packages'"). Broken/incomplete doc → SKIP (not a
  Rust gap). fancyhdr_sty.rs already defines \fancyhf/\fancyhead/\fancyfoot/\fancypagestyle/\headrulewidth/… and
  ports \f@nch@initialise (RawTeX :67-100).
- fancyhdr surface completeness audit: the binding covers the public surface; the ONLY missing public commands
  (none reached by these docs) are — `\fancycenter[][]{}{}{}` (4.0 centered h/f; faithful = no-op, header layout);
  `\fancyhdrsettoheight[2]` (fancyhdr.sty:438, measures a header/foot box into length #1; faithful = `\setlength{#1}\z@`
  or a def_macro_noop of `[2]`); `\fancypagestyleassign[2]` (fancyhdr.sty:753, 4.0 internal; no-op `{}{}`). Adding
  these completes the surface but fixes no doc in this set.
- codebox-doc-en \pkg/\url: NOT fancyhdr. `\documentclass{ctxdoc-en}` → ctxdoc-en.cls (shipped in doc/latex/codebox/)
  → `\LoadClass{l3doc}`. l3doc.cls:565 `\DeclareRobustCommand\pkg{\textsf}` DEFINES \pkg. Undefined \pkg means the
  ctxdoc-en/l3doc class chain didn't load in Rust (l3doc is a heavy expl3 doc-class; ctxdoc-en is a doc-dir cls) —
  an l3doc/class-load issue, not a binding-surface gap. Needs an l3doc-load diagnosis (own root).
- sduthesis-demo "inputencoding utf8": PARKED — oracle=lualatex, error trace is \luatexattributedef / \ltj@@attr@zero
  / \luafunction / \primitive (luatexja engine primitives = the parked CJK/luatexja family). Out of scope.

## FAMILY — pgf LuaTeX branch \pgfutil@luaescapestring — RUST-ONLY under [luatex] (4 docs)
Docs: neoschool, neoschool-fr, beamerthemeCelestia, beamerthemeCelestia-fr (the \IfFontExistsTF witnesses;
after that fix they progress to this error). Repro: pgf_luatex_graphdrawing.tex (RED [luatex]).
Engine test: pgfutil-common.tex:867-882 `\let\pgfutil@ifluatex\iffalse ... \ifx\csname directlua\endcsname
\relax\else\let\pgfutil@ifluatex\iftrue\fi`, then `\pgfutil@ifluatex \let\pgfutil@directlua\directlua
\pgfutil@directlua{tex.enableprimitives('pgfutil@',{'luaescapestring'})} \else \def\pgfutil@directlua#1{}
\def\pgfutil@luaescapestring#1{}\fi`. Under [luatex] \directlua is DEFINED (=\lx@directlua, engine identity)
-> LUATEX branch -> the lua `tex.enableprimitives` that would define \pgfutil@luaescapestring is NOT evaluated
(LaTeXML no-ops \directlua) -> \pgfutil@luaescapestring stays undefined. Reached by pgf's luamath/graphdrawing
libraries (pgflibrarygraphdrawing.code.tex:146; pgflibraryluamath.code.tex:126) — neoschool.cls:1481
`\usetikzlibrary{graphs,graphdrawing,...}` + a `\graph[..layout]`. In REAL lualatex the Lua runs and defines
it -> clean; RUST-ONLY (Lua unavailable). Rust pgfutil_common_tex.rs (11 lines) just
InputDefinitions('pgfutil-common', noltxml) — raw-loads the .tex, so the luatex branch runs and leaves it undefined.
FIX (faithful, keeps the luatex identity — do NOT touch \directlua/\luatexversion): in pgfutil_common_tex.rs,
AFTER the InputDefinitions, add pgf's OWN non-lua fallback:
  RawTeX!(r"\def\pgfutil@luaescapestring#1{}");   // = pgfutil-common.tex:882 / pgflibraryluamath.code.tex:68
This defines the lua entry point as its TeX equivalent for ALL pgf lua paths (shellescape, luamath, graphdrawing).
VALIDATED: gd_use [luatex] 2 err -> 0 err + 2 <svg:svg> (the \graph renders as plain tikz; the lua layout is
dropped, which is the expected LaTeXML limitation, not an error). Siblings `\pgfmath@usepgfmathlua`/
`\pgfmathsetseed`/`\pgf@sys@luaimage…` are the same lua-branch pattern but are NOT reached by these witnesses
(the single def clears all 4); add the same TeX no-op only if a future witness reaches them.
CONTROL: under pdflatex preload \directlua is UNDEF, so pgf's `\else` branch already `\def`s
\pgfutil@luaescapestring#1{} identically -> the binding's def is a redundant same no-op -> pdflatex unchanged.
Guard: pgf_luatex_graphdrawing.tex ([luatex]) -> 0 errors AND an <svg:svg>; pdflatex-preload control unchanged.
Risk LOW. Gain 4 docs.

## s36 second-error + full-log mining — document-model/frontmatter/float/caption tally

Docs (log files) per message class (of 2374 logs), MY lane:
| docs | class | status |
|------|-------|--------|
| 153 | "Attempt to close" (group/box/mode) | KERNEL-ALIGNMENT lane (not mine) |
|  52 | "isn't allowed in" (all) | see breakdown |
|  22 | "Did not find a block-like" (_CaptureBlock_ repackage) | capture-box family (partly covered) |
|  12 | malformed:ltx:caption / :toccaption | isorot/rubik caption-in-block (root 2, deferred) |
|  12 | malformed:ltx:_CaptureBlock_ close | B close rule (LANDED 55c) |
|  10 | @captype (caption context) | ⊂ caption-outside-float |
|   8 | "Use of \caption outside any known float" | TOP UNCOVERED (7 beyond threeparttablex) |
|   7 | malformed:ltx:section | screenplay + section-in-block (mode-frame/deferred) |
|   4 | malformed:ltx:picture | pagelayout (deferred) |
|   2 | float/figure isn't allowed in ltx:quote | UNCOVERED, coherent |
| "isn't allowed in" pairs (occurrences): XMTok-in-item 893(2 docs), XMTok-in-abstract 67(1), toccaption/caption-in-block 64/64(12), float-in-quote 10(2), section-in-block 8(1), bibliography-in-block 5(xebaposter).

### TOP class #1 — "Use of \caption outside any known float" — 8 docs — HETEROGENEOUS (not one root)
Docs: achemso-demo, biblatex-gb7714-2015, frankenstein/lips, jmlr/pmlr-sample, oup-authoring-template,
storecmd-guide, threeparttablex (LANDED), xtufts/sample-handout. The common caption contexts ALL WORK in
b55c (verified minimally: caption in table / in {..}group-in-table / in minipage-in-table / longtable /
ltablex-tabularx / float.sty \newfloat — all 0 err). Each remaining doc reaches the error via a
class/doc-SPECIFIC float mechanism: achemso's `scheme` is an expl3 custom float (achemso.sty has no
newfloat/@captype — the expl3 float declaration doesn't set \@captype in LaTeXML); storecmd/oup are
Elsevier/journal-class tabular contexts. NOT a single root — each needs its own float-declaration
investigation (per-doc). Reproduced: `\documentclass{achemso}` + `\begin{scheme}\caption{}` → 2 err.

### TOP class #2 — "<ltx:figure> isn't allowed in <ltx:quote>" — 2 docs — COHERENT — ROOT-CAUSED
Docs: isorot/rotman (the sidewaysfigure/figure-in-quote example), bashful/bashful. Repro: float_in_quote.tex.
A figure/table float built inside a `quote` (or any Block container: list/box) lands in <ltx:quote>. Schema
(LaTeXML.model): Block:=(…,ltx:quote) excludes floats, but Para:=(…,ltx:figure,ltx:float,…,ltx:table) — the
quote's ancestor <ltx:para> CAN hold the float. Real LaTeX floats ESCAPE their environment to the page.
Rust figure/table ctors (latex_engine/src/latex_constructs/sect09.rs:212/224/234/242) emit "<ltx:figure …>"
/ "<ltx:table …>" with NO `^` float-up marker; Perl (latex_constructs.pool.ltxml:3394) is identical → the
float is placed in the quote → schema error. SHARED (Perl builds the same <quote><figure> tree + same error);
pdflatex clean → surpass.
FIX: add the `^` (floatToElement, single caret = move-without-close) prefix to the figure/table/float
DefEnvironment patterns — sect09.rs:212 "^<ltx:figure …>", :224 (figure*), :234/:242 (table/table*), and the
float.sty `{float}` ctor; Perl parity in latex_constructs.pool.ltxml:3394+. floatToElement('ltx:figure') from
inside <ltx:quote> walks quote(no)→para(yes) and relocates the float to the para (float-escape), then restores.
Transparent for the normal case (a top-level figure's para already accepts it → no move). Guard:
float_in_quote.tex → 0 err AND `//ltx:para/ltx:figure` (figure is a child of para, NOT of quote); CONTROL: a
plain top-level `\begin{figure}\caption{}\end{figure}` → unchanged (0 err, figure in para). Risk MED (touches
the core figure/table ctor — re-verify a figure-heavy doc's structure). Gain 2 docs + any float-in-list/box.
