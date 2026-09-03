# index-bib topic — Checkpoint 1 candidate list (2026-09-03, binary b54r)

Errors = ANSI-stripped `^Error:|^Fatal:`. Counts from sweep #35 logs
(`/home/deyan/data/perfect_kernel_s35/<bundle>/<name>/<name>.log`).

## Mechanism buckets, ranked by docs x error-lines

### A. glossaryref/ref holds a bare math token in math mode  [TOP, repro written]
- glosmathtools/sample_glosmathtools_en (53), sample_glosmathtools_fr (53) = 106 lines / 2 docs
- Error: `<ltx:XMTok>/<ltx:XMApp> isn't allowed in <ltx:glossaryref>`
- glosmathtools puts a gls-link inside `\ensuremath` (glosmathtools.sty:74), so the
  glossaryref (Inline.model) is opened in math mode and its display content is a
  math token -> inserted as a bare <ltx:XMTok>. document.rs can't auto-open the
  Math>XMath path (neither Math nor XMath is autoOpen, in Perl OR Rust).
- GENERAL: `$\hyperref[s]{b}$` gives the identical error in `<ltx:ref>`.
- Class: SHARED (Perl TeX_Math.pool.ltxml:42 = Rust tex_math.rs:465 both autoOpen
  only ltx:XMText). pdflatex 0. Surpass-Perl in scope.
- Fix site: latexml_core/src/document.rs find_insertion_point_qsym (add a localized
  auto-open of <ltx:Math> inside a ref-family inline element when inserting a math
  leaf, mirroring the inline-block / foreignObject rules at document.rs:3099-3134).
  Math IS in Inline.model (LaTeXML.model:13) so `ref>Math>XMath>XMTok` is valid.
- Repro: glossaryref_math_glosmathtools.tex (RED, 1 err).

### B. hyperref binding: undefined hyperref internals  [repro written for \autopageref]
- abntex2/abntex2cite (1) `\autopageref`; ucalgmthesis/sample-thesis (5)
  `\hyper@makecurrent`; biblatex/biblatex (4, lualatex) `\hyper@normalise`;
  biblatex-chicago/cms-dates-intro (2) `\@baseurl`.  = ~12 lines / 4 docs
  (plus feeds the cms-noteref cascade: \hyper@linkstart/\hyper@linkend/\@baseurl).
- The Rust hyperref binding (hyperref_sty.rs) is a LoadDefinitions emulation that
  does NOT raw-load hyperref.sty, so hyperref internals not explicitly emulated are
  undefined. `\autopageref` (hyperref.sty:8183) is user-level and simple;
  `\hyper@makecurrent`/`\hyper@normalise` are deeper plumbing.
- Class: SHARED for \autopageref (Perl hyperref.sty.ltxml has \autoref L367 but NOT
  \autopageref). pdflatex 0.
- Fix site: latexml_package/src/package/hyperref_sty.rs — add \autopageref (+ star)
  faithful to hyperref.sty:8183-8190 (page ref, hyperlinked; star = no link).
- Repro: autopageref_hyperref_abntex2cite.tex (RED, 1 err).

### C. biblatex `\blx@*` internals undefined  [entangled multi-root; NOT minimal-reproducible alone]
- biblatex-chicago/cms-noteref-demo (65, lualatex) `\blx@refpatch@sect`
  (+54 cascade `expected:{` + cmsendnotes \@enotes/\@doanenote + hyperref internals);
  biblatex-sbl/biblatex-sbl (7) `\blx@xsanitizeafter`; biblatex-sbl-ibid (3)
  `\blx@opt@loccittracker@false`; sbl-paper (2) `\blx@key@bibcheck`;
  biblatex-apa-test (6) `\glet`; biblatex-juradiss (6) `\AtDataInput`.
- biblatex.sty DOES define these (e.g. \blx@refpatch@sect at biblatex.sty:11077).
  A MINIMAL biblatex doc (`refsection=section`) and a MINIMAL biblatex-chicago doc
  (`noteref=section`) both load CLEAN under b54r. So each residue doc aborts the
  raw-load at a doc-specific interaction (fontspec/lualatex, cmsendnotes, xr-hyper,
  csquotes autostyle, \externaldocument). Needs per-doc isolation at Checkpoint N.
- Fix site: kernel/engine so the raw biblatex.sty load completes (contrib
  biblatex_sty.rs is only the .bbl pipeline, not a biblatex.sty replacement).

### D. natbib/nmbib `\NAT@*` internals + cascade
- nmbib/nmbib-sample (22, pdflatex) `\NAT@reset@parser`, `\NAT@star@cite`,
  `\NAT@sort`, + alignment/tabular/math-`_` cascade. natbib.sty:780 defines
  \NAT@reset@parser but nmbib's multibibliography redefinition path leaves it
  undefined. Multi-root; Checkpoint N.

### E. tufte cite-in-sidenote `\@for` loop var undefined  [repro written]
- tufte-latex/sample-book (1), sample-handout (1) `\@temp@bibkeyx` = 2 docs
- tufte \cite -> \@tufte@normal@cite -> \sidenote(=\optparams{\@tufte@sidenote}) ->
  \@footnotetext; the \@for\@temp@bibkeyx loop var is undefined when the deferred
  sidenote body runs. NOT the broad footnote/\@for path (plain
  \footnote{\@for\x:=a,b\do{[\x]}} works). Divergence is \optparams / \@tufte@sidenote
  arg handling or a re-expansion of the deferred footnote body. pdflatex 0.
- Repro: tufte_cite_forvar_undefined.tex (RED, 1 err). Root TBD at Checkpoint N.

### F. bibarts \fi/\iffalse torn in \edef\write  [string-mouth overlap]
- bibarts/bibarts (16), bibarts/ba-short (5) — `Error:unexpected:fi` (the
  \iffalse..\fi idiom torn at the \edef string-mouth boundary). Already characterized
  SHARED in the repo's edef_write_ifmmode_bibarts_CONTROL.tex. Overlaps string-mouth topic.

### G. misc single-doc package-internal undefined
- biblatex-trad/biblatex-trad (10) `\ltd@title@title`; achemso-demo (4)
  `\mciteSubRef` + mciteplus_doc (1) `\@openbib@code` (mciteplus); esindex (1)
  `\spanishdatedel`; biblatex-iso690 (1) `\uv`; biblatex-cheatsheet (3) hypdestopt
  package error; biblatex-cv/cv (1) `\highlightname`.

### H. bibliography element placement (schema)
- xebaposter/poster (1) `<ltx:bibliography> isn't allowed in <ltx:block>`.

Note: robustindex/robustsample|robustmanual|multisample s35 errors are from the
pre-batch-54l binary and were fixed in 54l/54m (repo repros GREEN) — excluded.

## Repros written (all RED under b54r, all pdflatex 0)
1. glossaryref_math_glosmathtools.tex   (bucket A, 106 lines/2 docs, SHARED)
2. autopageref_hyperref_abntex2cite.tex (bucket B, hyperref binding, SHARED)
3. tufte_cite_forvar_undefined.tex       (bucket E, 2 docs)

# ============================================================
# Checkpoint N (2026-09-03, binary b54t) — A+B landed upstream
# ============================================================

## ROOT (iodhbwm): \marginnote drops a macro-argument layer -> `#` leak -> glossaries flood
- Witness: iodhbwm/iodhbwm (96x "Glossary entry `index-N-opt' has not been defined"
  + 48x "# (catcode PARAM) should never reach Stomach" + 1 \AfterBeginDocument + 1 color).
- Chain: iodhbwm.tex:222 \Option{load-preamble} -> skdoc.cls:631 \Options ->
  \marginnote{ \clist_map_inline:Nn\l_tmpa_clist{ \index@option*{####1} ... } }.
  `####1` (FOUR hashes) is calibrated to marginnote.sty's REAL macro depth:
  \marginnote -> \@dblarg\@mn@marginnote -> \@ifnextchar[ -> \@mn@@marginnote ->
  \@mn@@@marginnote (marginnote.sty:319-343) passes the body through one more
  macro-argument layer than a bare \marginpar. Our binding
  (latexml_package/src/package/marginnote_sty.rs:38-73) expands \marginnote DIRECTLY
  to \marginpar and stubs \@mn@marginnote/\@mn@@marginnote/\@mn@@@marginnote as noops
  (marginnote_sty.rs:83-85) -> body is one layer short -> after xparse halves
  ####1->##1, a literal `#1` PARAM survives into the \clist_map_inline item body ->
  reaches the Stomach (stomach.rs:1905), AND skdoc's \@index@@ (skdoc.cls:921)
  runs \newglossaryentry{index-#1-opt} (name defined under a leaked-# key) so every
  later \gls{index-<opt>-opt} is "not defined".
- Class: SHARED. Perl marginnote.sty.ltxml L37-40 ALSO shortcuts \marginnote->\marginpar
  (per the Rust binding's own comment), so Perl leaks identically. pdflatex 0 -> in scope.
- Repro: marginnote_hashdepth_skdoc.tex (RED rust=2, pdflatex=0);
  CONTROL marginnote_hashdepth_bare_CONTROL.tex (pdflatex ALSO rejects `####1` without
  marginnote's real depth: `##1` is the correct count for a bare command).
- Fix (latexml_package/src/package/marginnote_sty.rs LoadDefinitions): replace the
  \marginnote->\marginpar DefMacro (L38-73) with a faithful port of marginnote.sty:319-343:
    \marginnote = \@dblarg\@mn@marginnote
    \@mn@marginnote[#1]#2  -> \@ifnextchar[ dispatch to \@mn@@marginnote (L322-333)
    \@mn@@marginnote[#1]#2[#3] -> \@mn@@@marginnote (L336-343)
    \@mn@@@marginnote[#1]#2[#3] -> \marginpar{<font/raggedright wrapping> #2}
      (fold the current binding's \mn@parboxrestore\marginfont\raggedrightmarginnote
       + optional [left] handling into this terminal step)
  This restores the exact macro-argument depth that `####1` (and any package
  calibrated to real marginnote) expects. VALIDATED: overriding \marginnote with this
  chain in the doc drops rust 2->0 on the repro and 6->0 (all glossaries+PARAM) on the
  minimal skdoc doc, entries then correctly named index-alpha-opt/index-beta-opt.
- Guard (cluster_package_guards / perfect_kernel_batchNN): marginnote_hashdepth_skdoc.tex
  -> 0 errors AND output contains `[alpha]`/`[beta]` (or the note element) with NO
  literal `#`. Risk: LOW-MED (touches only the marginnote binding; \@ifnextchar/\@dblarg
  are already supported — the realchain override used them and ran clean). Expected
  corpus gain: iodhbwm 146->~2; also unblocks any skdoc-documented package
  (skdoc is a doc class; other TeXdoc manuals use \Options/\Option).
- Residual (separate roots, out of this fix): iodhbwm also emits 1x \AfterBeginDocument
  undefined (scrlfile.sty; \AfterBeginDocument is a scrlfile hook alias for
  \AtBeginDocument-ish) and 1x "Can't find color named ''".
- Dead ends: `\clist_map_inline:Nn` itself is fine (2-hash test -> [a][b]); bare
  `\DeclareDocumentCommand`+clist_map is fine; \marginpar/\footnote/\fbox do NOT add
  the extra layer (they reject `####1` in pdflatex too) -- ONLY marginnote does.

## LEAD (tufte \@temp@bibkeyx, root #2 — characterized, not yet pinned)
- The `\@for\@temp@bibkeyx` loop var is undefined only in the REAL tufte cite path
  (tufte-common.def:898 in \@tufte@normal@cite inside \sidenote, and/or :934 in
  \@tufte@print@citations "puts the citations in a margin note"). Error frame is
  "Anonymous String" -> a re-tokenized/isolated mouth (matches the deferral hypothesis).
- NOT reproduced by any isolated piece: plain \footnote{\@for...}, \@footnotetext{\@for...},
  \marginpar{\@for...}, \optparams passthrough (optparams.sty just forwards #3, no \edef),
  and \@for+\ifthenelse{\equal{..}{\@temp@bibkeyx}} at top level -- ALL clean. So the
  trigger is the specific COMBINATION (likely \@tufte@citations accumulation +
  \@tufte@print@citations margin note, or the natbib-\cite interaction), where the
  sidenote/citation body is expanded in an isolated mouth AFTER the \@for group ended.
  Next: trace \@tufte@print@citations + \@tufte@infootnote@cite (\g@addto@macro
  \@tufte@citations) and whether our engine re-tokenizes that accumulated body.
- Repro (still valid under b54t): tufte_cite_forvar_undefined.tex (4 lines, rust=1, pdflatex=0).

# ============================================================
# Checkpoint N+1 (2026-09-03, binary b54w) — marginnote root LANDED
# ============================================================
# iodhbwm re-verified on b54w: 146 -> 2 (residual = 1 \AfterBeginDocument [scrlfile]
# + 1 empty-color-name; both separate roots, out of scope here).

## ROOT (tufte \@temp@bibkeyx): \nocite defers its key UNEXPANDED to end-of-document
- Witness: tufte-latex/sample-book, sample-handout. Chain: tufte \cite ->
  \@tufte@normal@cite -> \sidenote -> \@tufte@sidenote -> (bidi hook) \@footnotetext is
  \renewcommand'd to \marginpar{...}, and \@tufte@print@citations (tufte-common.def:934)
  runs \marginpar{ \@for\@temp@bibkeyx:=\@tufte@citations\do{ ... \bibentry{\@temp@bibkeyx} } };
  bibentry.sty:64 \bibentry{#1} -> \nocite{#1}.
- Rust site: latex_constructs.rs:10016 `\nocite{}` captures its key via `.revert()`
  (UNEXPANDED) and PushValues `\lx@mark@nocite{<raw key>}` onto @at@end@document
  (deferred to \end{document}). The key `\@temp@bibkeyx` is a transient \@for loop var; by
  the time the deferred token is processed it is out of scope. Inside a \marginpar (deferred
  digestion) the deferred token additionally gets EXPANDED -> "\@temp@bibkeyx is not defined"
  (frame "Anonymous String" = the re-tokenized deferred mouth). Real TeX writes \citation{#1}
  via \protected@write\@auxout, which EXPANDS the key at the call site (while \@temp@bibkeyx
  is still key1).
- Isolation ladder (all under b54w): \@for over empty macro -> OK; \marginpar{\@for over macro
  list} -> OK; \bibentry in \@for WITHOUT marginpar -> OK; \bibentry/\nocite in \@for INSIDE
  \marginpar -> RED; \@nameuse-only inside -> OK; \nocite-only inside -> RED. So the trigger is
  precisely \nocite (deferred raw key) + a deferred \marginpar body + a transient \@for key.
- Class: SHARED design (Perl latex_constructs.pool.ltxml:4214 also PushValues the RAW $_[1]);
  the error is Rust-manifested (Perl likely yields bibrefs="\@temp@bibkeyx" without erroring).
  pdflatex 0 and expands the keys -> fix SURPASSES both (correct bibrefs="key1", no error).
- Fix (latex_constructs.rs:10016, \nocite DefMacro): expand the key argument (protected
  expansion, matching \protected@write\@auxout) BEFORE building the deferred
  \lx@mark@nocite{<keys>} token list, instead of deferring the raw .revert()ed token.
  VALIDATED: \expandafter\nocite\expandafter{\@temp@bibkeyx} -> clean + bibrefs="key1";
  overriding \nocite to \edef its key drops the full tufte doc 1->0 with bibrefs="key1".
- Repro: nocite_deferred_forvar_tufte.tex (kernel-only: \nocite in \marginpar+\@for;
  RED rust=1, pdflatex=0). Guard (cluster_package_guards): 0 errors AND output contains
  <ltx:bibref ... bibrefs="key1"> (NOT bibrefs="\@temp@bibkeyx").
- Risk: LOW-MED. Only \nocite's handler; expanding a cite-key list is what TeX does. Watch:
  `\nocite{*}` (star) must survive expansion unchanged (it does -- `*` has no expansion), and
  keys that are plain identifiers are expansion-idempotent.
- Expected corpus gain: tufte-latex sample-book + sample-handout (1 each -> 0); any doc that
  \nocite/\cite's a bibkey held in a transient loop/temp macro inside a deferred box.
- Dead ends: not the footnote/sidenote/optparams deferral (plain \footnote/\@footnotetext/
  \optparams all clean); not \@for empty-list detection; not \marginpar deferral alone;
  not \g@addto@macro (\@tufte@citations accumulation is a red herring -- the failing loop
  fires with a single expanded key).

# ============================================================
# Checkpoint N+2 (2026-09-03, binary b54w) — \nocite root LANDED
# ============================================================

## ROOT (nmbib-sample): \citeall falls through to nmbib's raw natbib-internal engine
- Witness: nmbib/nmbib-sample (22 errors). Isolated trigger: \citeall{key} (nmbib-sample.tex:20,
  and every other \citeall). \citep/\citet/\citenum/\citealn (undefined OR resolved) are all CLEAN
  in isolation — our natbib EMULATION intercepts them as high-level <ltx:cite> constructors.
- Mechanism: nmbib (\RequirePackage{natbib}, nmbib.sty:43) REIMPLEMENTS natbib's low-level citation
  engine. \citeall (nmbib.sty:343) -> \@citeall -> \@@@citeall (nmbib.sty:347), a full-citation
  expander parallel to nmbib's \NAT@citexnum (nmbib.sty:141); both OPEN with \NAT@reset@parser
  (natbib.sty:780), then \NAT@sort@cites (natbib.sty:1122), \NAT@reset@citea (natbib.sty:598),
  \@cite, \NAT@parse (natbib.sty:761), \NAT@num/name/date, \@ifnum, \NAT@ctype/\NAT@cmprs/
  \NAT@ifcat@num. \citeall is nmbib-specific and is NOT intercepted by our emulation, so it runs
  nmbib's raw expander and hits undefined internals (first: \NAT@reset@parser). The `_`/tabular/
  \end{table} errors downstream are the malformed text-mode citation output cascading.
- Missing surface: nmbib reaches 63 \NAT@* internals; natbib_sty.rs (LoadDefinitions, ~20) provides
  a small subset. The 57 missing include the parser (\NAT@parse, \NAT@reset@parser), state
  (\NAT@num, \NAT@name, \NAT@date, \NAT@all@names), sort (\NAT@sort, \NAT@sort@cites, \NAT@sort@cites@),
  assembly (\NAT@citex, \NAT@citexnum, \NAT@def@citea, \NAT@open/@close, \NAT@reset@citea) + \@cite/\@ifnum.
- Class: SHARED. Perl natbib.sty.ltxml emulates high-level too (8 \NAT@*, no \NAT@reset@parser/
  \NAT@citexnum/\NAT@parse); no nmbib.sty.ltxml exists -> Perl hits the same undefined internals.
  pdflatex 0 -> in scope (surpass), but LOW ROI (single niche corpus doc).
- Fix (recommended): a SMALL nmbib binding (latexml_package/src/package/nmbib_sty.rs) that EMULATES
  \citeall/\citeall* (and \@citeall/\@@citeall/\@@@citeall) as a high-level <ltx:cite> constructor
  the way natbib \citet/\citep are emulated, so they do NOT fall through to \@@@citeall/\NAT@citexnum.
  \citealn is already [\citenum{#1}] (nmbib.sty:338) and works. Also emulate \multibibliography/
  \multibibliographystyle as no-ops or bib-list producers if needed for full-doc parity.
  Do NOT port the 57 \NAT@* internals: that duplicates natbib.sty, yields TEXT not <ltx:cite>, and
  still cascades (the `_`/tabular errors are the text output). VALIDATED: \DeclareRobustCommand
  \citeall[1]{\citet{#1}} -> 0 errors + <ltx:cite bibrefs="Markey:Tame_the_BeaST">.
- Repro: citeall_natbib_internals_nmbib.tex (RED rust=10, pdflatex=0). Guard: 0 errors AND output
  has <ltx:cite ... bibrefs="Markey:Tame_the_BeaST">.
- Risk of the fix: LOW (adds a binding; \citeall currently produces nothing usable). Corpus gain:
  nmbib-sample 22 -> ~0 (a couple of residual alignment errors may remain if any are non-cascade).
- Dead ends: \citep/\citet/\citenum/\citealn (undefined or resolved) all clean; \multibibliography
  with missing .aux is clean; the trigger is specifically \citeall (and the multibib rendering path,
  which also uses nmbib's \NAT@citexnum).

# ============================================================
# Checkpoint N+3 (2026-09-03, binary b54y) — nmbib \citeall root LANDED
# ============================================================

## ROOT (biblatex-sbl/sbl-paper, smallest \blx@* doc = 2 errors): binding omits internals a raw
## biblatex style .def reaches
- Witness: biblatex-sbl/sbl-paper (sbl-paper.tex:229 \printbibliography). First errors:
  \blx@key@bibcheck (biblatex.sty:9643) + \blx@printbibliography (biblatex.sty:9820), both at the
  \printbibliography call.
- Correction to the family framing: the Rust biblatex binding does NOT raw-load biblatex.sty. It
  STANDS IN for it (biblatex_sty.rs:706-727: "biblatex.sty is provided by a native binding, not
  interpreted raw") and emulates a SUBSET, defining its own \printbibliography[] (biblatex_sty.rs:1995
  -> \biblatex@printbibliography native worker). So this is NOT a raw-load abort -- it is a binding
  coverage gap.
- Mechanism: the biblatex-sbl STYLE raw-loads biblatex-sbl.def, whose line 663
  `\renewrobustcmd*{\printbibliography}` OVERRIDES the binding's \printbibliography with a copy of
  biblatex's REAL definition (\begingroup \blx@key@bibcheck{bibliography} ...
  \@ifnextchar[{\blx@printbibliography}{\blx@printbibliography[]}). That body reaches \blx@key@bibcheck
  (biblatex.sty:9643, the check= option handler) and \blx@printbibliography (biblatex.sty:9820, the
  render worker) -- neither defined by the binding. Confirmed by \meaning: style=sbl swaps
  \printbibliography to the raw body, style=authoryear keeps the binding's native macro (CLEAN).
- Class: RUST-ONLY. Perl has no biblatex.sty.ltxml and RAW-loads biblatex.sty, so both internals are
  defined there (raw-load reaches 9643/9820); the biblatex-sbl.def override then works. The gap is
  specific to the Rust native-binding design (emulation incomplete where a raw style .def reaches
  biblatex internals). pdflatex 0 (\printbibliography w/o biber only warns).
- Fix (latexml_contrib/src/biblatex_sty.rs): define the two internals the raw .def reaches:
    \blx@key@bibcheck#1  -> faithful biblatex.sty:9643 body
        (\ifcsdef{blx@bibcheck@#1}{\letcs\blx@bibcheck{blx@bibcheck@#1}}{}) or a no-op gobble
        (the check filter is cosmetic in our output)
    \blx@printbibliography[#1] -> the binding's native \biblatex@printbibliography[#1]
  so the sbl-overridden \printbibliography routes to native rendering. VALIDATED: providing these two
  (bibcheck no-op, printbibliography -> native) -> 0 errors on the minimal sbl doc.
- Repro: blx_printbibliography_sbl.tex (RED rust=2, pdflatex=0). Guard: 0 errors AND \printbibliography
  yields <ltx:bibliography> (empty w/o biber is acceptable).
- Risk: LOW (two internals routed to an existing native worker / no-op). Corpus gain: sbl-paper 2->0;
  and \blx@printbibliography/\blx@key@bibcheck are reached by any biblatex-<style>.def that copies the
  real \printbibliography (helps biblatex-sbl.tex, -ibid, and similar style-.def overrides).
- Note: the LARGER family members fail on DIFFERENT internals (biblatex-sbl.tex \blx@xsanitizeafter,
  -ibid \blx@opt@loccittracker@false, apa-test \glet, juradiss \AtDataInput, trad \hyper@normalise,
  achemso \mciteSubRef) -- each a distinct binding/hyperref/kernel gap, to be pinned separately.
- Dead ends: not a raw-load abort of biblatex.sty (binding stands in for it); not sbl.bbx/sbl.cbx
  (they don't touch \printbibliography); style=authoryear is clean (keeps the native \printbibliography).

# ============================================================
# WAVE 15 — Checkpoint 1 (2026-09-03, binary b54t): rest of the
# biblatex/hyperref internals family (12 assigned roots)
# ============================================================

META-MECHANISM (unifies most roots): a NATIVE Rust binding STANDS IN for a raw
.sty/.cls (biblatex, mciteplus, amsart; hyperref is Perl-emulated too) and emulates
a SUBSET. A DERIVED raw file — a biblatex style .bbx/.cbx/companion .sty, or a derived
.cls — is raw-loaded and reaches package INTERNALS the binding omits. Per bucket the
class-level fix = extend that binding's internal surface (the user's preferred shape),
NOT per-name whack-a-mole. Empirically grounded: `\usepackage{biblatex}` bare is CLEAN
in Rust (binding stands in) but Perl raw-loads biblatex.sty and FAILS UNIVERSALLY at
`\ProcessLocalKeyvalOptions` (kvoptions.sty.ltxml gap, biblatex.sty:7113) = 3 errors on
ANY biblatex doc. So the Rust biblatex binding is already AHEAD of Perl; it only lacks
the raw-style-reached internal surface.

## BUCKET 1 — biblatex.sty internals reached by raw STYLE files  [CLASS-LEVEL, top value]
Fix site: latexml_contrib/src/biblatex_sty.rs (the `LoadDefinitions!` body, extending the
existing "Declaration-only biber/data-model and setup hooks" surface ~L1044). Class:
SHARED (both engines fail on these docs; Perl fails EARLIER/differently at kvoptions, so
these exact "undefined:\blx@*" lines are Rust-manifested). pdflatex 0 -> surpass in scope.
Roots (real def : raw file that reaches it):
- \blx@refpatch@sect  biblatex.sty:11077  <- cmsendnotes.sty:121,126,135 (biblatex-chicago)
    cms-noteref-demo (65 total; only 1 is refpatch — the other 64 are the cmsendnotes
    ENDNOTES machinery \@enotes/\@doanenote/\@endanenote/\if@enotesopen +hyperref link
    internals; \blx@refpatch@sect alone will NOT clear this doc — separate endnotes root).
- \blx@xsanitizeafter biblatex.sty:1216   <- sbl.cbx  (biblatex-sbl, 7: also \blx@nocite@do,
    \blx@ifdata, \blx@blxinit, \abx@missing@entry, + landed \blx@printbibliography/\blx@key@bibcheck)
- \AtDataInput        biblatex.sty:8985   <- standard-dw.bbx:389 (biblatex-juradiss, 6: also
    \blx@safe@actives, \OnManualCitation [biblatex-dw], + 2 schema cascade + etoolbox toggle)
- \ResetDataInheritance biblatex.sty:14566, \DefaultInheritance :14474 <- windycity.bbx:741-742
    (windycity, 7: also \except, \clearfield, \DeclareCitePunctuationPosition,
    \DeclareAutoPunctuation, \idemcites — all biblatex.sty user/decl commands)
Semantics note: most are declaration/hook/data-model directives that only shape BIBER's
.bbl (\AtDataInput=\csgappto a bblitem hook; \DefaultInheritance/\ResetDataInheritance=
biber inheritance; \clearfield/\except/\DeclareCitePunctuationPosition/\DeclareAutoPunctuation=
formatting) — cosmetic for our pipeline (we render biber's .bbl). They can be
faithful-SIGNATURE no-op gobbles (EXACT arg counts, from biblatex.sty, so following tokens
aren't misparsed). \blx@xsanitizeafter (biblatex.sty:1216 `\protected\def#1#2{...}` — runs
#1 after x-sanitizing #2) and \blx@ifdata/\blx@blxinit need faithful bodies (they gate real
control flow). REPRO: blx_datainherit_windycity.tex (rust=5, perl=3, pdflatex=0).

## BUCKET 2 — hyperref.sty internals reached by raw packages  [CLASS-LEVEL, 4 sub-families]
Fix site: latexml_package/src/package/hyperref_sty.rs. Class: SHARED — hyperref.sty.ltxml
EXISTS (Perl emulates hyperref like the Rust binding) and ALSO omits these internals; Perl
fails IDENTICALLY (verified rust=2/perl=2 on \hyper@normalise). pdflatex 0 -> surpass in scope.
Sub-families (each its own faithful semantics):
- URL-normalise: \hyper@normalise, \url@ (url.sty), \hyper@linkurl, \Hurl  <- \fnurl
    (biblatex.tex:170-171 `\DeclareRobustCommand{\fnurl}{\hyper@normalise\fnurl@}`,
    `\fnurl@[1]{\footnote{\url@{#1}}}`) / btxdockit \href. Docs: biblatex(2), biblatex-trad(3).
    \hyper@normalise = hyperref's url-arg catcode sanitizer (hyperref.sty); the binding
    emulates HIGH-level \url/\href but not the low-level arg-normaliser other cmds build on.
- Anchor/link: \hyper@makecurrent, \hyper@linkstart, \hyper@linkend  <- pagenote (ucalgmthesis
    class config) at chapter1:37 \pagenote. sample-thesis(5) [+ 2 misdefined:# from sample-thesis.ent].
    Minimal trigger is class-config-entangled (bare hyperref+pagenote did NOT reach it).
- pdfstring: \HyPsd@UTFviii, \HyPsd@ConvertToUnicode, \HyPsd@LetUnexpandableSpace,
    \HyPsd@AMSclassfix  <- \pdfstringdef body with active latin1 bytes. dvdcoll(4). ENTANGLED
    with [latin1]inputenc active-char expansion — the binding's \pdfstringdef=\gdef#1{#2}
    (hyperref_sty.rs:955) doesn't run the \HyPsd@ converters; the byte path invokes them.
- Driver: \Hy@driver + \@{link,url,cite,file,menu,run,anchor}bordercolor  <- hrefhide.sty
    driver-check. hrefhide(9) [+ hrefhide "Producing not a pdf file" package error — hrefhide
    aborts because \Hy@driver is undefined]. Entangled with driver detection.
REPRO: hyper_normalise_url_biblatexmanual.tex (rust=2, perl=2, pdflatex=0). The other 3
sub-families are each entangled with their host package/encoding — pin separately at CP N.

## BUCKET 3 — per-package binding gaps (not biblatex/hyperref core)
- \mciteSubRef  mciteplus.sty:781-782 (`\def\mciteSubRef{\@ifnextchar[{\@mciteSubRef}
    {\@mciteSubRef[\mcitetrackID]}}`, `\@mciteSubRef[#1]#2{\ref{\@mcitereflabelprefix:#1:#2}}`).
    The Rust mciteplus binding (latexml_contrib/src/mciteplus_sty.rs, 54 lines of no-op stubs)
    OMITS the user-facing \mciteSubRef/\@mciteSubRef/\mcitetrackID/\@mcitereflabelprefix.
    Class: RUST-ONLY (Perl has no mciteplus.sty.ltxml -> raw-loads mciteplus.sty -> defined ->
    verified perl=0). achemso-demo(1 of 4; also \bibnote [achemso_cls.rs gap], {scheme} float,
    \caption cascade — so the \mciteSubRef fix alone won't clear achemso-demo).
    Fix: mciteplus_sty.rs defines the 4 macros faithfully (route \@mciteSubRef -> \ref).
    REPRO: mcitesubref_mciteplus_achemso.tex (rust=1, perl=0, pdflatex=0).
- \author@andify  amsart.cls (frontmatter and-ify).  <- resphilosophica.cls:75
    `\LoadClass{amsart}`, :323 `\author@andify\authors`. Rust amsart_cls.rs binding stands in
    for amsart.cls and OMITS \author@andify/\@dedicatory (SAME stands-in pattern as biblatex).
    Class: likely RUST-ONLY. rpsample(1 of 4; also \@dedicatory + 2). Fix: amsart_cls.rs surface.
- \headlessfullcite/\headlesscite/\shortrefcite  biblatex-chicago chicago-notes.cbx / .sty —
    user-facing cite commands not emulated. cms-notes-sample(6, lualatex). Biblatex-chicago
    binding-surface gap (cite family). Pin at CP N.

## PARKED (report to orchestrator, NOT index-bib scope)
- \glet  apa.cbx:643-646 is INSIDE `\ifdef\luatexversion{\directlua{require'apa'}\glet...}{}`
    — a LuaTeX-ONLY branch. Our [luatex] profile DEFINES \luatexversion=121
    (latexml_package/src/package/latexml_sty/mod.rs:138), so \ifdef -> TRUE, entering a
    \directlua{require'apa'} block we can't execute; \glet (never defined by biblatex/etoolbox
    — assumed present under lualatex) + \BLTXAPAifInParensTF (from apa.lua) stay undefined, +
    a Fatal recursion loop. biblatex-apa(6+Fatal). This is the luatex-detection-probe family
    (luatex-profile topic), not a biblatex-internal gap. Defining \glet alone does NOT fix it.

## TOP-3 REPROS (all RED under b54t, all pdflatex 0)
1. blx_datainherit_windycity.tex        BUCKET 1 (biblatex internals) rust=5 perl=3
2. hyper_normalise_url_biblatexmanual.tex BUCKET 2 URL-normalise    rust=2 perl=2 SHARED
3. mcitesubref_mciteplus_achemso.tex     BUCKET 3 mciteplus         rust=1 perl=0 RUST-ONLY

## RECOMMENDED CHECKPOINT-N ORDER (one root/checkpoint, highest class-value first)
CP2: BUCKET 1 biblatex internal surface (windycity repro; clears windycity 7->0, and the
     same surface unblocks biblatex-sbl [+5], juradiss [+3], cms-noteref [+1]). Best ROI.
CP3: BUCKET 2 URL-normalise (\hyper@normalise + \url@ + \hyper@linkurl + \Hurl; 2 docs).
CP4: BUCKET 3 \mciteSubRef (trivial, RUST-ONLY, Perl is the oracle).
Then, as separate roots: hyperref anchor/link (sample-thesis), pdfstring (dvdcoll),
driver (hrefhide); cmsendnotes ENDNOTES machinery (cms-noteref — the real 54-error cost);
\author@andify (amsart binding); biblatex-chicago cite family (cms-notes-sample).

## DEAD ENDS (one line each)
- Bare `\usepackage{biblatex}`: Rust CLEAN, Perl 3 errors (kvoptions) — binding is ahead, not behind.
- \glet is NOT etoolbox (no \glet in etoolbox.sty/.def) nor biblatex.sty — apa.cbx assumes it under lualatex only.
- Bare hyperref+pagenote did NOT reach \hyper@makecurrent — the reach needs ucalgmthesis's pagenote config.
- \HyPsd@UTFviii is not a clean hyperref-surface gap: entangled with [latin1]inputenc active-byte -> \pdfstringdef.
