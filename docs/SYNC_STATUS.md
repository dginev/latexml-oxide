# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "Ranked worklist" below and take the top unblocked row.** That is the
whole intent of this file; everything after it is supporting detail.

| section | what it is | when you read it |
|---|---|---|
| **Ranked worklist** | every open item, ordered, with size + where the detail lives | **first, always** |
| Current status | suite count, the last session, release state | to orient |
| Open items | the detail behind the ranked rows | when you pick that row |
| Standing policies | rules that constrain *how* you fix things | before adding a CLI flag, a stub, or a divergence |
| Parked families | pointers to extracted docs | only when starting that family |
| Reference | stable facts, not active work | when something surprises you |

Three rules that keep this file honest:

1. **Verify a status label before acting on it — and before deleting it.** Check the
   **named guard test** in the tree, or `gh issue view <N>` / `gh pr view <N>`.
   **SHA-ancestry does not work** as a check — the repo squash-merges, so a branch
   SHA quoted here is never an ancestor of `main`.
2. **This is the BRIEF ACTIONABLE LIST.** Day-by-day logs live in `git log` and
   `docs/archive/`. When you close an item, delete it here and lift anything
   worth re-reading into `docs/archive/SYNC_SESSIONS_YYYY-MM.md`.
3. **Keep it under ~500 lines.** When a section outgrows ~100 lines it has become
   its own subject — give it a doc under `docs/` and leave a one-line pointer.

*Last compaction: 2026-10-04 — the run-329 narratives (61r-62f) moved to `archive/SYNC_SESSIONS_2026-10.md`, the parked math rows to `math/MATH_PARSE_RESIDUALS.md`, landed rows removed; before: 2026-10-02 — the K13 stage-2 findings and the rc131/rc132 + 57cn–59l rows moved verbatim to `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`, their open residuals kept here. Prior: 2026-09-27 (closed R1, R2, R5 and the 2026-08-02 status snapshot to `archive/SYNC_SESSIONS_2026-09.md`); 2026-09-03 (949 → ~320 lines; 1276 → 953); 2026-08-18 (1462 → ~890); 2026-07-25 (1979 → ~500).*

---

## Ranked worklist — start here

Ordered by importance (2026-10-04 review): closing the perfect-kernel goal, then run 329 and its residuals (they seed
the coming arXiv success-rate task, not started), the rulings that block rows, the work safe to do while the run
finishes (T1, T2, F1, F2), then the older parity rows. Within a tier: **does it reproduce today** → **is a real user
affected** → **is it unblocked** → **effort**.

| # | item | state | size | detail |
|---|---|---|---|---|
| **R8** | **Perfect-kernel program** (branch `perfect_kernel`; the TL-manual corpus; generalized capabilities K1–K18 in [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md)) — goal bar met at sweeps #146-148 (G1–G5); close status in [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) | **CLOSING**: bar confirmed (user 2026-10-04); ar5iv-css PR #54 open; run 329 completes | program | [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) |
| **G** | **Run 336** (full arXiv, 2,947,191 papers; 62zu `c0e4393f39` to 2026-10-07 16:35Z, then `9569ca27a7`) — COMPLETE 2026-10-10 07:10Z; vs run 306 (snapshot 2026-09-17): error 12.86 % → 6.32 %, fatal 0.87 % → 0.27 %; 203,366 papers out of error, 20,570 out of fatal; 16,827 worse into error/fatal; no_problem → warning 290,846 (the warning-visibility shift); E/F rerun **run 338** (e57be39640, complete 2026-10-12): of run 336's 186,160 error / 8,019 fatal, 42,143 left error and 2,509 left fatal (corpus error 6.32 % → 4.89 %, fatal 0.27 % → 0.19 %); 142 error → fatal to triage; **run 340** (E/F rerun on 1bd0c44f59: 64zf + 65b + 65c, complete 2026-10-12 15:19Z): of run 338's 144,214 error / 5,652 fatal, 2,282 left error and 305 left fatal (corpus error 141,799, fatal 5,634); 287 error → fatal — 204 TooManyErrors (107 = longtable undefined under aa/pasj00/emulateapj, a 65b regression, batch 65h), 59 cortex never_completed (dispatcher lease clock started at batch fetch, fixed in CorTeX a5bbbee5, not yet deployed), 18 PushbackLimit; **runs 344/345** (E/F rerun on f1811254dc: 66b-66e, complete 2026-10-14 05:04Z): corpus error 141,799 → 105,835 (3.62 %), fatal 5,634 → 4,625 (0.158 %), OK + warning 96.22 % of 2,922,325; error → OK 5,966, error → warning 30,248, fatal → OK 81, fatal → warning 678, fatal → error 544; 294 error → fatal, of which a local reconversion confirms 118 (Fatal on 66e1, Error on 64zf: TooManyErrors 68, Mouth EoF 24, Recursion 14, PushbackLimit 9; `~/data/pk_agents/main/ef344/fatal118.tsv`) and the rest are fleet artifacts (OOM under host memory pressure, timeouts) | **TRIAGED** (2026-10-10): 1,571 clusters of the 16,827 (`~/data/pk_agents/main/run336/worse_306_336.tsv`, `agent_reports/2026-10-10_run336_*`, `_rc336_*`). On head already clean: CCSXML EOF (64d), imageprocessing (AppArmor), most `never_completed`. Faithful (pdflatex errs too, `\errmessage` an Error since 56g): ctex fandol 107, l3backend twice 71, acro `class` 52, `\DeclareMathOperator{\free_var}` 155, `\verb` misuse 183, JINST.cls under TL 2025 39. Fixed in 64h (~2,500 papers): `\hyper@makecurrent` patchable 129 (with the kernel's `\theH<counter>` names, OD #480), `pdftex` option / PostScript-only K6 cue 47, CJK stray bytes 746 + byte-mouth `.bib` 4, tcilatex `\thechapter` 537 + plain `\input #1` 2, aipproc `\tablehead` ~428, elsart wrappers 367, algorithmicx nested listings 134, spverbatim 107, nomencl prefix 12, `\diaghead` pair. Fixed in 64k (OD #482): raw classes' refused `\author`/`\abstract` replayed for their flags, stores kept the kernel's (JINST/JHEP/CECS 340, iosart2x 71), the Foundation Compositio family (224) and JINST/CECS bindings (semantic contacts, keywords, MSC, dates; no second title page), each reading the class file the document asked for (a path-prefixed `JINST-Sample-files/JINST`, a renamed `JINST_mod`) — A/B over 2,918 papers (64h4 → 64k16, the 238 the email rounds touch re-run on the final 64k22): with errors 809 → 252, with Fatals 72 → 2, no creator or abstract lost; a labelled email line of a marked author block is now its author's contact, or the block's (OD #52(m); addresses on several authors 66 → 4). Open: the 31 renamed JHEP3 copies (QCEMDA, AKRSX, jhepLike…) need a class route by content; `blkarray` `\begin{block}{[…]c}` raises `\else` errors (2112.09933, reached once its frontmatter stopped failing); a Semiverbatim URL contact's `~` prints as the OT1 tilde accent `˜` (`\urladdr{…/~y}`, Perl the same; RED sectioning-frontmatter/url_contact_keeps_tilde). Fixed in 64qb (KPE #563-567, OD #483): paspconf bound to aaspp (+ its Q&A, `\deg`, keywords as RDFa) and RevTeX 3's keyless `\reference` (~860), a caption2 binding (843), amsmath's `\macc@` chain with `\skewchar` a font register (817), `\tablebreak` ends the row (38), `\sidecaption[pos]` (186), jmlr2e `\keywords{}` (7), `Digested::to_attribute` ported (a skip in an attribute is its spaces, not `\enskip` text; Perl List/Whatsit `toAttribute`); A/B 332 papers: with errors 301 → 37, none worse. Fixed in 64p (run 336's loops, KPE #475/#568, OD #315): a class's `\renewenvironment{thebibliography}` that a binding's hook runs unlocked is refused, the lock read as a value as Perl reads it (wiley2sp & co, 132), and a class's own bibliography list — `{thebibliography}` `\let` to it (ajour, cjour, IEEE-Con-Sys-mag) or its `{references}` — gives way to the kernel's at the first `\bibitem` (RED: a class end printing text after its list prints it before the entries; a class bibliography built on `{enumerate}` leaves a group open — both synthetic; OD #484, KPE #569); `\tag{\thesubeqn}` (74); mathfixs' autobold `\let` of a robust switch (33); a document's `\mathaccentV` accent (21; its `tex` attribute now reads `\mathaccent 28766{G}` where 62d's named reading gave a `.bbl`'s `\hat{G}` — a `\mathaccentV` reversion is open). RED: keyval runs a key given no value that has no default with an empty value (keyval.sty:52-56 errors and skips it; csquotes `[english]` loops, 5; `macro-state/keyval_bare_key_without_default`). Fixed in 64qa (KPE #570-573, OD #485): World Scientific `\body`/`\bodymatter` and table rules (ws-procs9x6, ws-procs961x669, ws-rv9x6; ~1,078), iopart `\ioptwocol` (1,058), revtex `\rev@citealpnum` (1,023), svmult `\toctitle`/`\tocauthor` (960), mdpi `\history`/`\changeurlcolor` (765), WileyNJD `\state` (701), the ptephy author block (204 → 27 papers with errors), quantumarticle tikz (61 → 4). Open: epsfig `\epsfbox` a no-op (KPE #574, its own batch with an A/B); mdpi's other history fields gated by status like `\history` (display-toggle rule); the ws classes' packages (rotating, dcolumn, url, chapterbib, cite[super,nospace]: bindings load what the class loads). Fixed in 64s (OD #486, KPE #575): ar5iv's `\today` is the paper's source date (the newest TeX source entry time of its zip/tar), `\relax` only for an undated bundle; ptapap's `\today` parse 95 (1801.05985), `<date>` filled on 43 of 60 sampled `\today` papers; open: ptapap `\let\oldfnum@figure=\fnum@figure` undefined (92 papers). Fixed in 64ra (KPE #576-581, OD #487-491): `$$` is display math in IEEEtran `{IEEEproof}`, lineno `{linenomath}`/`{linenumbers*}`, OmniBus `{frontmatter}` and autart `{pf}` (macros as the classes define them, not boxed DefEnvironments; ~1,600 IEEEproof, ~110 linenomath, ~200 frontmatter, ~90 autart papers), IEEEproof titled by its optional argument with IEEEtran's own QED switch (`\IEEEQEDhere`/`\IEEEQEDoff`/new `\IEEEQEDhereeqn`, one tag "∎(1)"), the class QED symbols sized as their boxes so a cell holding only the QED is kept, and `\IEEEeqnarray`/`IEEEeqnarraybox` columns built from the IEEEtrantools column table (≤1k papers); A/B: IEEEproof 60 → 1 papers with errors, linenomath 40 → 3, `\endlinenomath` 30 → 1, frontmatter 39 → 12, autart 26 → 4, IEEEeqnarray 80 → 8, 30 extra-tab eqnarray papers byte-identical. IEEEtrantools' own IEEEproof/QED commands for non-IEEEtran classes landed in 67x (`block-model/ieeetrantools_ieeeproof_any_class` GREEN); open:, IEEEeqnarray `[decl]`/struts (its rule rows landed in 67x), 64 more mode-less `"#body"` DefEnvironments (census in `agent_reports/2026-10-10_payload_64ra.md`). Fixed in 64w (OD #492): an axodraw v1 binding (not in TL; 1,507 papers with `\Text` undefined, 149 as the first error) evaluating v1's PostScript procedures (gluons, photons, arcs, arrows, blobs, boxes, ovals, curves, axes; PostScript-arithmetic arguments over dvips-header constants) into the picture model, `\Text` at `\unitlength`s unscaled, colordvi loaded as the colordvi-aware v1 does; it also answers papers shipping their own axodraw.sty (drawn now, errors equal), while axodraw2 (raw TL file, kept off the versioned fallback) and axodraw4j stay raw. A/B vs e1ec58c24e: 149 witnesses with errors 149 → 10, errors 872 → 94; 63 shipping, 16 axodraw4j, 8 axodraw2 papers unchanged in errors and warnings. Port gap closed alongside: the SVG of an `ltx:arc` keeps its stroke attributes, as SVG.pm `convertArc` copies them (L385-387) — every arc, pstricks' `\psarc` too, had been invisible. Open RED: a picture label mixing text and math overprints in the SVG (graphics-axodraw/picture_label_mixed_text_and_math, hep-ph0509118); a math label in a `\vcenter{\hbox{picture}}` is a bare `<msub>` without `<math>` (graphics-axodraw/picture_math_label_in_vcenter_box); `\hspace*` before a picture is dropped (graphics-axodraw/picture_after_hspace_star); colordvi's `\Red{…}` sets no color (fonts-nfss/colordvi_named_text_color); an article `\author` block's second name after `\\[12pt]` is read as an affiliation (sectioning-frontmatter/article_author_block_second_author_after_spaced_break, gr-qc9906006). Fixed in 64u (OD #493-496, KPE #582): bindings for journal styles neither in TL nor shipped — l-aa/laa/l-aa-ps as the `aa` class with `\thesaurus` codes, argument-less `\keywords` and `$^n$`-marked institutes (843), epl2/epl (740), OSA opex3/ol2/osajnl/osajnl2 with appending authors and email lines and `\homepage` placed by mark, spelled name or group through the new kernel `\lx@add@contact@placed` (759), REVTeX 2.1 `{title}`/`{instit}`/`\receipt`/`\nonum` (750); A/B vs 64ra: l-aa 126 papers with errors 126 → 5, EPL 142: 117 → 1, OSA 622 shipping no style: 622 → 7 and 129 shipping one: title 40 → 129 papers, creators 7 → 129 (1104.0806's private abbreviations lost: 5 errors), REVTeX 120: 86 → 9; no keyword, abstract or affiliation text lost, no email moved off its owner. No generic fallback for a missing `\usepackage` (user ruling 2026-10-10). Open: under `\documentstyle[aps]{revtex}` the OmniBus overlay gives an `\author{A and B}` group's address to B only (RED `revtex3_omnibus_author_group_address`; ruling asked); `\documentstyle[l-aa]{article}`; marks that are a paper's own macros (1201.5125). Fixed in 64y (OD #497-499, KPE #583-585): an environment frame closed by `}`/`\endgroup` (M1, ≈2,060 run-336 papers) ends as tex.web §1064 `off_save` does, the diagnostic following the group kind the begin code leaves (none | semi_simple | box; 71 environments audited); 17/17 witnesses clean (139 → 0 errors), 150 of the cluster 141 → 82 erroring (1172 → 642); 0 regressions over 300 random + 100 theorem/list-heavy + 70 reviewer papers; REVTeX 3 figure list entries (cond-mat9607130 1 → 0) and natbib `\bibpostamble` (2009.08029 1 → 0). RED follow-ups in `boxes-groups/`: `atbegin_hook_group_above_env_frame` (2001.02707), `bare_theorem_ended_by_enclosing_end`, `amsthm_bare_proof_closed_by_brace`/`amsthm_proof_closed_by_endgroup` (the proof is a constructor, so a group closer silently swallows the document), `footnote_bare_quote`, `macro_env_ended_by_inner_bare_end`, `bare_doublespace_closed_by_brace` (2310.08233, setspace's no-op bare `\doublespace`); 2106.02397's float.sty binding no-op `\float@end` under floatrow; `osa,floats` REVTeX 3 figures. Fixed in 64zb (OD #500): a locked binding owns its calls (D1, superseding the arity rule below) — a document's parameterless redefinition of a binding with a braced signature is the meaning for every use without an argument (pgffor `\foreach \i/\text`, `\@for\text:=`, `\def\title{…}` as a value), a `{`/`[`/`*` call still runs the binding (kept as `\lx@locked@<cs>`), a redefinition with parameters (robust and ltcmd forms included) stays refused, `@` names and 0-parameter readers stay absolute; A/B vs 8542e1b1a5: M5 cluster 369 papers errors 11,882 → 2,500 (140 fewer, 125 to zero, none more), census 314/320 identical, restyle 100/100, random 298/300 (2 fewer errors); witnesses 2405.18512 45 → 0, 1406.2544 31 → 0, 1511.05179 1 → 0. Accepted residual: an unbraced parameterless forwarder runs the author's code (`\mathcal C` after `\def\mathcal{\mathscr}`; 1806.09639, 1904.10871 `\sqrt` hack). RED: `macro-state/m5_locked_name_in_expansion_only_context` (`\lx@locked@<cs>` in `\meaning`/`\typeout`), `sectioning-frontmatter/paragraph_title_trailing_gobbler_open_group` (2604.23335, pre-existing); m5_e2/m5_d2 deferred (rulings 4/5). Fixed in 64zc (OD #501-503, KPE #586): an email or web address keeps its `_` and `~` in the font its class sets it in — typewriter (`\lx@address@urlfont`: REVTeX 4.0, AASTeX 5.x, amsart/amsproc/amsbook, llncs, JHEP, jheppub, iopart, aa, elsarticle, cas, imsart) or roman (`\lx@address@textfont`, user ruling 2026-10-11: REVTeX 4.1/4.2, AASTeX 6+, emulateapj, svjour3/svmult, quantumarticle, jmlr, pasj00, ptephy), the author block's contacts made of a class's `\email` call too, `\~{}` in a typewriter address a `~` of its run, the link of a split address its whole text, `\i`/`\j` kept in the ASCII encoding (1001.1076 "aiwen˙fan"; RED `url_contact_keeps_tilde` green); A/B vs 64u over 175 sampled underscore-email papers (7 months of run 336), 100 random and 31 review papers: no Error/Fatal/Warning count changes, papers with a `˙` email/url contact 60 → 0, every XML change on those sets on an email/url contact (font, or `˙`/`˜`/`ˆ` → `_`/`~`/`^`), no address left split into a one-character link. Beyond contacts, the accent of an empty letter is the current font's (OD #503): 6 of 13 sampled `\~{}` papers change in the body, each a run joined or a typewriter `~` (1001.1367, 1001.1793, 1301.6104, 1301.6221, 1901.07113, 2101.11048). Open: a binding's Semiverbatim address freezes `$`/`^`, so an address's math prints as text and its link holds the `$`s (RED `sectioning-frontmatter/semiverbatim_address_freezes_math`; 1601.02576, 1601.02586, 1601.00888; Perl the same, the class of KPE #515); hyperref's `\url{http://x.org/\'{\i}}` prints the accent's expansion in its text and link (RED `sectioning-frontmatter/url_accent_command_leaks_expansion`). Fixed in 64zd (OD #504-505, KPE #574 fixed, #587): REVTeX 3's own options declared in revtex3_support (revtex.sty v3.0/v3.1 `\ds@` and the distribution's option files; `eqsecnum`, `preprint`, `osa`), its numbering as aps.sty ("II A", "II A 1", appendix "A 1", A1, an unnumbered appendix stepping the counter; seg A-1, rmp "II.A") and marked author lists split (OD #52(n): `\cite`, math surnames, "by", bare asterisks); `\compat@loadpackages` loads OmniBus provide-only (`ProvideOnlyGuard`: every binding definition during the load — OmniBus's and the bindings it requires — declines a name defined before it, environments whole, article not re-run, `extends` wrappers such as its `\bibitem` install anyway, TeX `\def`/`\let` untouched); bindings for the 2.09 option styles OmniBus used to stand in for: World Scientific `sprocl`/`mprocl`/`stwol` (appending `\author`/`\address`, `\abstracts`) and AASTeX substyles `aas2pp4`/`aaspptwo`/`apjpt4`/`aj_pt4` → aaspp (AASTeX 3's `{planotable}` ending its own data, astro-ph9502038); author lists (OD #52(n)): glued initials before capitals surnames (`M.CACCIA`), a doubled `\& and` separator, a name line with a `\thanks` between affiliations opening the next author when the block's first names carry one (hep-lat9404004); sprocl addresses led by an author's mark go to that author; AASTeX's obsolete `\authoremail` before `\author` the email of the author it spells, not a creator of its own (astro-ph9803018); packages OmniBus requires under provide-only are unmarked when it ends, so a later `\usepackage` loads them whole; epsfig `\epsfbox` includes its graphic; aas `{references}` titled; A/B against 44ff2ac956 (64zc) over 150 REVTeX 3, 40 REVTeX 2 `{instit}`, 149 other-class missing-option papers, 400 more option-stratified and 46 of the bound style families in `agent_reports/2026-10-10_payload_64zd.md`. Open: marked affiliations then an unmarked group's: the unmarked one reaches the marked authors too (RED `sectioning-frontmatter/marked_affiliations_then_unmarked_group_leaks`; revtex4 the same); an article author block's plain name line between plain affiliation lines stays an affiliation (RED `sectioning-frontmatter/article_plain_name_line_between_affiliations`); `\input` of a binding in a group a second time finds its definitions gone (RED `loader/input_binding_in_group_twice`, cond-mat9610182; TeX re-reads, Perl the same; low priority); hep-th9707026's glued `Sahng-kyoon.Yoo` list stays one creator; the ws classes' psfig/epsfig load (RED `singletons/ws_procs_psfig`, now unblocked by KPE #574). Fixed in 64x (OD #506-511, KPE #588-597): titlepage `{centering}`; natbib `\bibitem` outside any list reports Lonely `\item` (Perl's `malformed:ltx:bibitem`, once per `\bibitem`; 1108.2593 40 Errors kept, structure recovered) and among a list's own items keeps its place (1411.7266 10 → 0); implicit-bibliography headings absorbed or untitled (no contents entry); biblist reopen with the phantom prune and emptied-list sweep at the bibliography's and document's close (147 → 6 phantoms over 127 papers, the 6 holding figure panels); only a TeX `\item`/`\bibitem` steps the counter (ACM `.bbl`-preamble ids now `bib.bibN`, user ruling); a `.bbl` bibliography after an implicit one prints again (KPE #596); AMS-TeX `\end`/`\predefine`; `\endabstract` locked against files; jheppub `\abstract`; PoS without JHEP journal macros + `\pos`. A/B 127 papers vs 8c7ed11a23: Errors 1986 → 1276, none above base, every drop a shape pdflatex prints cleanly (`scratch_64x/ab/drops10.txt`). Accepted residual: 2205.07302 keeps its words but loses its `\enquote` marks as `<ERROR>` (KPE #589, RED index-bib/bibitem_list_local_definition, deferred to K21). Open RED: kernel `\bibitem` in the body silent (KPE #595; Warning per ruling, batch 64zf stage 2); natbib `\bibitem` among an enumerate's items (cond-mat0003103, 1 Error; K21); a paragraph after a lonely natbib `\bibitem` with no heading between becomes a keyless entry (pdflatex: a paragraph; RED index-bib/natbib_lonely_bibitem_then_paragraph). Fixed in 64ze (OD #512-517, KPE #598-599; K22 census binding gaps): natbib entry interface (`\BibitemOpen`/`\BibitemShut`/`\bibinfo`/`\bibfield`; 80 → 2 papers with errors; a lonely bibliography spends no `\BibitemShut` stop); svmult/svmono (`{svgraybox}`, `\svhline`, `{prob}`/`{sol}`, `\motto`/`\chapauthor` via the after-heading hook, svmono as a book; 167 → 14); aipproc `\pacs`; svjour `\preprint`; interact/World Scientific `\bm`; IEEEAerospace; quantumarticle hyperref + REVTeX compatibility (PACS/preprint as RDFa, natbib numbers; 46 → 5); elsarticle bare `\frontmatter`; mn2e abbreviations per shipped copy (217 → 77); kernel: fallbacks yield to the author's shipped files, `\noalign\\` (2309.15248). A/B 1,530 documents vs e57be39640: no rises, 679 papers fewer with errors. Open: pasj00/PASJ95 absent-class emulation (~190 papers; own batch); quantumarticle `plainnat` default; svmult 2018 info boxes; RED `noalign_let_tabularcr`, `svmult_frontmatter_chapter_numbered`, `aipproc_nested_pacs` (0709.4622). Queued from the run-336 Error/Fatal ranking (`agent_reports/2026-10-10_run336_error_fatal_ranking.md`, triage `rc_undefA`/`rc_undefB`/`rc_struct`): 64q binding gaps behind first-error undefined commands (ws `\bodymatter` ~1,078, iopart `\ioptwocol` ~1,058, revtex4 `\rev@citealpnum` ~1,023, svmult `\toctitle` ~786, mdpi `\history` ~765, paspconf ~860, caption2 ~843, amsmath `\macc@` ~817, WileyNJD `\state` ~701, ptephy ~204); 64r roots behind stray `&`/`_`/`^` (`\lefteqn` as a 3-column span ≤2.2k, `$$` in boxed binding environments ≤1.7k, `IEEEeqnarray` column spec ≤1k, redefined `\span`, a braced amsmath body, `\tablebreak`, `\sidecaption`, jmlr2e `\keywords`). 64l pgfkeys native dispatch (~17), 64m ar5iv preload loads the format early (`\salta`, 84), 64n alignment cell-head peek (61), 64o delimited argument past an unmatched `}` (Fatal → Error), 64p loops (`\renewenvironment` lock 132, `\tag`/`\theequation` 76, `\let` wrapper 33, `\mathaccentV` 21, keyval 5). Ruled 2026-10-10 (queued 64t-64w): `&` as text in stored-but-untypeset metadata, bindings for l-aa/laa, epl2, OSA opex3/ol2/osajnl2 and REVTeX 2 `{instit}` (plus a generic raw-loading improvement), `\left`/`\right`/`\middle` unexpandable (62 IfLimit in xymatrix), an axodraw v1 SVG binding (1,507). Ruled later 2026-10-10: an environment frame closed by a plain `}`/`\endgroup` recovers as tex.web `off_save` does (M1, ~2,060, unparked: 64y); a locked binding yields only to a redefinition with a different number of mandatory arguments, after minimal repros show the rule well-founded (forwarding 0-argument redefinitions such as `\renewcommand\section{\@startsection…}` excluded; M5, ~160); a heading between `\bibitem`s is an inline sectional block inside the one bibliography (~750); a missing `\usepackage` file gets no OmniBus fallback, only `Warning:missing_file` (pdflatex fails too: 1210.7592 `opex3.sty` not found; shipped copies raw-load); REVTeX 3.0's real options declared in revtex3_support; an implicitly opened bibliography takes a preceding heading-like line as its title, else none (no `\refname`) | program | [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) close status |
| **KC** | **Kernel capability program** (user 2026-10-11): the open gaps grouped by the missing mechanism behind several of them — K24 one audited A/B comparator (P0), K20 gullet expansion fidelity (four tex.web algorithms: column ends by command code, `\left`/`\middle`/`\right` primitives, §395 extra `}`, peeking macros as one unexpandable step; absorbs queued 64n/64o/64v), K21 environments as TeX does them (`Streamed` vs `Captured`; absorbs the `#body` audit), K22 binding provenance and ownership (FileIdentity, owned sets, corpus-copy audit; version drift and author-shipped copies), K23 the author-block event model; next tier C3 placement policy (11,012 `malformed:` first errors), C6 content-aware emptiness, C7 string values for attributes | **ORDERED**: K24 LANDED (`ab_audit`, p0 `1e47551e6c`); manual sweep #149 → K20 G4 → G2 → G3 → G1 → K21, K22's read-only census/audit in parallel → K23 shadow mode | program | [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md) K20–K24; evidence [`KERNEL_CAPABILITY_GAPS_2026-10-11.md`](perfect_kernel/KERNEL_CAPABILITY_GAPS_2026-10-11.md) |
| **MS** | **TeX's math-group recovery** (main session 2026-10-12, task #204) — **65n landed for inline formulas outside alignments** (OD #553): a math shift meeting a `{`/`\begingroup` group opened in the formula inserts its closer and ends the formula there (§1193 + §1064-1065; 2007.11971 13 → 1 errors, 179 → 0 run-together runs, text words 5,268 → 9,865), a `\par` or a DefEnvironment's end in it is "Missing $ inserted" (§1046-1047), a `}` at its math shift ends it when it closes a construct digested as a unit (a box body, an argument digested on its own — content first by main-session decision: `\mbox{$x} after` keeps "after" as text where pdflatex swallows it into the box) and is deleted otherwise (a plain `{…}` group included, as TeX: `{\bf $x} y$ after.` keeps "after." text), one Error "Extra }, or forgotten $" either way (§1069; 1710.02165 29 → 21); display math stays tolerant of `\par` (no §1047, by decision); since 66w (user ruling 2026-10-14) the `\par` keeps an inline formula open when a math-only token precedes the next `$` (a formula split by a blank line, Perl's content; hep-th9402135). REDs dollar_closes_open_script_group, par_in_inline_math_inserts_dollar, stray_brace_then_dollar_keeps_text GREEN. **Open**: (1) a formula in an alignment keeps the old recoveries — a cell's `$` may be its template's (math9805003's undefined `{Sb}` in an `align*` lost the alignment when its row `$` closed the `\begingroup`); an author's cell formula needs the template's `$` told from the author's (RED paragraph-mode/cell_end_in_inline_math_inserts_dollar; 1210.0368 and 1907.09124, whose deferrals sit in alignments, unchanged); run 363 adds 1906.07043 (iopart + breqn, which loads amsmath: the "Old form `\cases`" error leaves `$x>l_0$` inside `{equation}`'s alignment, where a lone `$` does not end the display as §1197 "Display math should end with $$" does — 101 Errors + TooManyErrors where pdflatex gives 9 and a PDF); (2) display math has no §1047 (a blank line in `equation` is common), so its stray `}` keeps 65a's inserted `$` (f2 `\[ x } y \]`, a13); (3) DONE: boxes-groups/mbox_math_stray_brace_ends_box_not_math GREEN by the unit-construct rule (round 3; round 2's "partner outside" rule lost a1/a2/a16, reverted); its residual, a forgotten `$` in a plain group (`Before {\bf see $x} more text.`) runs "more text." as math to the `\par`, as pdflatex — possible refinement by `$`-parity in the rest of the paragraph (odd: delete; even: end at the `}`) (RED paragraph-mode/forgotten_dollar_in_a_plain_group_keeps_the_text); (4) the `}`-meets-`\left`-group deletion in inline formulas (65d round 7, h1/d2/e1-e4) is unblocked by (§1065 at `$`) and not done (RED paragraph-mode/brace_meeting_left_group_in_inline_math_is_deleted); (5) hep-lat9811036 / nucl-th0401019 run text together from another open construct, not this one | **PARTLY DONE 65n**; (1) next | M | OD #553, #530; `agent_reports/2026-10-12_payload_65n.md`, `_65d.md` rounds 5-7 |
| **XP** | **A detail-less XPath failure in post is a resource failure, not 100+ Errors** (runs 362/363): on the fleet, post's `ltx:XMath` lookups failed "(no libxml2 error detail)" 87-501 times per paper (latexml_post/src/document.rs:959, :1031) until `TooManyErrors` — 8 of run 363's 35 E→F (1008.4986, 1712.03782, 1910.13564, 2101.04438, 2208.07188, 2305.17269, 2401.10096, 2609.06115); each converts locally with 1-14 Errors and no such failure. A trivial relative path cannot fail syntactically, so a NULL result without a libxml2 error is libxml2's own allocation failure inside a long-lived child (C `malloc`, outside the Rust allocator hook that turns an allocation failure into `Fatal:oom`). Fix: route it to the same cause-carrying `Fatal:oom` path (one Fatal, the child recycled, retried per the 2026-10-15 reused-child ruling) instead of an Error per node; keep the Error for failures that carry a libxml2 message. Mechanism CONFIRMED (68m step 1, 2026-10-16; evidence ~/data/pk_agents/main/scratch_356a0c47/68m_step1/RESULTS.md): exhausting glibc malloc under RLIMIT_AS makes rust-libxml =0.3.22 `node_evaluate_checked` return exactly "XPath evaluation failed (no libxml2 error detail)", code 2 (XML_ERR_NO_MEMORY), domain 12, message None, on libxml2 2.15 and 2.9.14, where a bad XPath (`ltx:XMath[`) gives "Invalid expression" (code 1207); a fresh-process `ulimit -v` sweep of 2101.04438 (1.2-7.0 GB) never reproduces it (the Rust side fails first), so it needs a reused child. Step 2 (the fix) not started | **OPEN** | S | runs 362/363 bullet below; memory feedback_rulings_2026_10_15 |
| **BD** | **A `\DeclareBoldMathCommand` delimiter after `\left`/`\right`** (65j round 3 review): bm.sty's `\bm@define` (:307-316) `\xdef`s `\blangle` to a bold `\delimiter` (`\bm@delimiter`, :416), so `\DeclareBoldMathCommand\blangle{\langle}` then `$\left\blangle x\right\rangle$` is a stretchy bold ⟨ in pdflatex (0 errors); ours defines `\blangle` as `\bm{\langle}`, a constructor `\left`'s delimiter scan cannot read: Warning "Missing delimiter (. inserted)" (naming `\bm` on the 65 bundle, `\lx@bm@bold` on 65j), a null `\left` and a non-stretchy bold ⟨ after it. Witness 1205.4484 (`\boldlangle`/`\boldrangle`/`\boldlvert`/`\boldrvert`). RED `boxes-groups/bm_declared_bold_delimiter_after_left` | **OPEN**, RED only | S-M | OD #544; `agent_reports/2026-10-12_payload_65j.md` round 3 |
| **A2** | **Fidelity audit round 2 residuals** (silent content/markup losses in "successful" run-336 papers): acmart comma list (2b), an.cls `\abstract{}` (9), entics `\thanksref{ALL}` (10), `$(a)$` marks (15), sectsty (11), title-`\cite` tooltip (12), llncs check (16) | **OPEN**; 63j-63m landed 10 classes (63m: group affiliations 14, names past `\\` 13) | small-medium each | §"Fidelity audit round 2" |
| **A3** | **arXiv html_feedback subgoal** (user 2026-10-08): 5,970 reader reports → 4,819 papers → 44 problem classes; each class verified on head, persisting defects ranked HF1-HF13 — tcolorbox listings one line high (HF1, 37 papers; root-caused: the listings block has no sizer), merged authors of the report set (HF2, 118), binding gaps vs shipped classes (HF3), acmart `\Description` raw TeX and labels on its hidden note (HF4), box font sizes (HF5), deferred typesetting (HF6), error clusters (HF7), pgf text anchors vs amstext (HF11), `\includestandalone` and TikZ residuals (HF12), table pruning (HF13) | **OPEN**; baseline sweep + 4 classes verified (raw-latex-leak 12/15 fixed, tcolorbox 6 fixed + 5 viewer, tikz 6 fixed + 1 viewer, table 10 fixed + 3 viewer, of 15 each); 63m landed HF2 shapes; 63n landed HF1 (listing sizes) and HF4 (`\Description` digested, labels on the float) | medium each | [`perfect_kernel/HTML_FEEDBACK.md`](perfect_kernel/HTML_FEEDBACK.md) |
| **A4** | **Full arXiv fidelity audit** (user 2026-10-10): a high-performance Rust analysis script over the whole arXiv conversion output for silent content/markup losses (empty frontmatter elements, words lost against the source, ERROR nodes, unparsed-math ratio, misplaced elements), ranked into fix classes — A2's method at corpus scale | **QUEUED, LAST**: after the run-336 Error/Fatal subgoal and the queued payloads reach diminishing returns | large | — |
| **G1** | **Run-329 open residuals** — 17 PushbackLimit Fatals (csvsimple + siunitx `S`), TooManyErrors / never_completed, the `gullet.rs` `handle_template` latent panic, the runtime-bindings `.rhai` fallback, amsppt-as-plain, `.aux` re-read, eplain, calc's `!`, singles | **NEXT** (arXiv success-rate task) | medium each | §"Run-329 and sandbox open residuals" |
| **D1** | **Rulings**: RULED 2026-10-04 — implement tex.web §392 `\par` in a non-`\long` argument (1001.1670) and restore Perl's `Error:malformed` for DIVERGENCES #189 (prerex section-in-figure): 62g landed (#189: 7 in-scope manuals now report it — biblatex, keytheorems-doc, pdfmarginpar, phonrule-doc, prerex, srdp-mathematik, zx-calculus — so G1 counts 7), 62h landed (§392, DIVERGENCES #442 / KPE #472-473). Still pending: `\obeylines` `^^M`; `{framed}` item shape; biblatex `cite.<n>@<key>` surpass anchors (D15); R7 port-or-drop | RULING | small-medium | rows cited |
| **T1** | **Repro suite move**: `tools/perfect_kernel/repros` (1,469 files) → `latexml_oxide/tests/repros` header-contract trials (release CI + `LATEXML_FULL_TESTS=1`); drain `tools/` | **OPEN**, not started (user 2026-10-02) | large | memory `project_repro_suite_move` |
| **T2** | **Test API stage 2**: 37 test files still spawn binaries (`Command::new`; cluster_cli 33, cluster_xslt_split 13 …) → the in-process convert API; five `[[bin]]`s remain | **OPEN** (stage 1 landed 61a) | medium | `~/data/pk_agents/main/HANDOFF.md` (61a) |
| **F1** | **Per-encoding text commands**: textcomp's symbols now dispatch on the encoding (62r, KPE #489; repro `fonts-nfss/textmu_follows_the_encoding` GREEN). Residual: the kernel letters keep Perl's definedness gate (a T1 fontmap slot shared by `\DH`/`\DJ`), and `\<E>-cmd` chains ignore `\cf@encoding` | **PARTIAL** | medium | §Font-Selection Chain Residuals |
| **F2** | Small RED repros: hyperref pdfinfo values expanded as by `\pdfstringdef`; `\vadjust` queue scoped to its list (60k, TeX nest; low return); the rest under §"Perfect-kernel RED repros" | **OPEN**, safe now | small each | §Perfect-kernel RED repros |
| **R3** | **Bibliography-absence campaign** (PR #444) — **291 recovered / 20 338 entries**. Remaining unblocked: **R3d tab-mark parameter scan vs cell read** | **R3d next** (12 papers left, unblocks alignment macro `&` splits) | medium | Open items §R3, [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md) |
| **R8b** | **forest.sty full support** (side goal, user 2026-09-05; heavily used on arXiv) — native semantic tree since 56iz (nested inline lists, labels as TeX, node keys; DIVERGENCES #302); open: drawing/layout fidelity | **OPEN**, recorded | large | [`perfect_kernel/DIFFICULT_CASES.md` §D10](perfect_kernel/DIFFICULT_CASES.md) |
| **R4** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified (1 error with `--preload=article.cls`, 0 without). Pool load reordering | medium | Open items §R4 |
| **R6** | **`Collector::rescan` Refactor for Generated Backmatter** | **OPEN**; `Scan` owns `ObjectDB` by value; generated backmatter lacks full relations/labels | medium | Open items §R6 |
| **R7** | Presentation-MathML **F5** Linebreaker | **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision | family | Open items §R7 |

---

## Current status

The corpus measure is the perfect-kernel scoreboard ([`PERFECT_KERNEL.md`](PERFECT_KERNEL.md), sweeps of the 2,374
TL manuals) and the per-batch arXiv A/B; the cortex reruns of sandboxes 2605/2606 (runs 325-328, 2026-10-04) and run 329's validations are in
`archive/SYNC_SESSIONS_2026-10.md`; the last full sandbox rerun before them (2026-08-02, 60,505 docs) in
`archive/SYNC_SESSIONS_2026-09.md`.

---

## Open items — detail for the ranked rows

### R3 — Bibliography-Absence Campaign (PR #444 Residuals)
- **R3d: Alignment Parameter Scan vs Cell Read Distinction (`suppressed_tab_marks`):**
  - *Symptom:* An unescaped `&` inside a delimiter-fenced macro argument splits the alignment row and truncates the document (and bibliography). 12 papers remain affected.
  - *Mechanism:* `tex.web` §394 `macro_call` suppresses tab marks while scanning parameters. `SuppressedTabMarks` in [`latexml_core/src/common/local_assignments.rs:194`](../latexml_core/src/common/local_assignments.rs#L194) fixed `physics.sty`'s `\mqty` (14 papers), but applying it globally to `Parameters::read_arguments` regressed 5 tests (`cells_test`, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content.
  - *Action:* Distinguish macro parameter scanning from alignment cell reading in `Parameters::read_arguments` so tab marks inside `{...}` do not split outer cells.
- **R3b: No-Diagnostic Chase Candidates (~6 left):**
  - Remaining papers with real `\cite` calls but empty bibliography: `2605.14990`, `2606.05629` (math-in-body silent drop), `2606.10056`, `2606.17491`, `2606.00231`, `2605.29754`.
- **R3g: amsrefs Bare `\begin{biblist}` (4 papers):**
  - `\begin{biblist}` without `{bibdiv}` wrapper $\to$ `malformed:ltx:biblist`. Requires `BACKMATTER_ELEMENT` route.

### R4 — `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`)
- **Symptom:** `--preload=article.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel` (clean without preload or with `LATEXML_NODUMP=1`).
- **Mechanism:** `\@pushfilename` changes meaning mid-load: `article` is pushed before `LaTeX.pool` loads, using a pre-pool `\@pushfilename` that does not touch `\g__hook_name_stack_seq`. The pool installs expl3's `\@popfilename`, which pops an empty seq and errors.
- **Resolution:** A TeX-side repair or re-synchronizing the sequence at the point `LoadPool('LaTeX')` executes.

### R6 — `Collector::rescan` Refactor for Generated Backmatter
- **Symptom:** Generated subtrees (Bibliographies, Indexes, Glossaries) lose ObjectDB relations, labels, and fragids because `Scan` owns `ObjectDB` by value (`latexml_post/src/scan.rs:49`) and cannot rescan generated nodes.
- **Resolution:** Refactor `Scan` to borrow or take/restore `ObjectDB`, enabling clean rescanning of generated nodes and removing fragile ad-hoc registrations.

### R7 — Presentation-MathML F5 Linebreaker
- **Status:** The only remaining pMML gap from the MathML line audit (F17 is closed). A full linebreaker algorithm gap requiring a port-or-drop scope decision before coding.

---

## Secondary residuals & unranked active items

### Open residuals of closed rows (R1, R2, R5; details in `archive/SYNC_SESSIONS_2026-09.md`)
- **R1:** the 117-paper `TooManyErrors:MaxLimit(100)` cluster the seeds came from (2026-08-02 rerun) has not been
  re-measured; its seeds `2605.22927`, `2606.11121`, `2606.01136`, `2605.10685` are clean since 56ks3/56kt. The same
  rerun's `Stomach:Recursion` cluster (55 papers, MIXED) had three REAL-by-count witnesses never followed up:
  `2605.17696` (Rust 144 / Perl 56 errors), `2606.05321` (35 / 15), `2606.08524` (94 / 50).
- **sn-jnl (DIVERGENCES #333):** newer class copies (2404+, e.g. 2605.00003 L1280-1321) define `\toprule`/`\midrule`/
  `\botrule`/`\cmidrule` themselves instead of loading booktabs; neither Perl nor the binding runs the class code, so a
  paper relying on them without its own `\usepackage{booktabs}` keeps `\toprule` undefined (parity).
- **Equation layout (DIVERGENCES #334):** `eqnarray` after a class's `fleqn` + amsmath is flush left in pdflatex, centred
  here (`ltx_fleqn` is document-wide). `document_class_filename` (content.rs) is overwritten by nested class loads, and
  xkeyval's fallback reads it (`xkeyval_sty.rs:1524`). elsarticle's `\ifpreprint` is always false here; elsarticle.cls:112
  makes `preprint` the default, and `5p`/`3p`/`1p`/`final` clear it (:71-87).
- **Cropping (DIVERGENCES #337):** `reflect` is not applied; a negative trim (padding) is clamped, as in Perl; anisotropic
  resolutions crop by the x one; the job key holds the full options, so `[trim=X,clip,width=3cm]` and `[…,width=5cm]`
  crop twice into identical files.


### Font-Selection Chain Residuals
1. **`\cal ABC` collapses to one `<mi>`**: Drops `class="ltx_font_mathcaligraphic"` (Perl emits three `<mi>` elements, Rust one containing `𝒜ℬ𝒞`). Token grouping and class diverge.
2. **`\DeclareTextCommand`/`\ProvideTextCommand` lack encoding-dispatch chain**: `latex_constructs.rs:6525/6544` bind bare `\cs` to first-encoding expansion permanently.
3. **Minor font registration gaps**: `\DeclareTextSymbol` decodes eagerly instead of installing deferred `CharDef`; `\DeclareErrorFont` is a bare no-op where Perl defines argument as `\relax`.
4. **A preamble font switch stays in force in the body** (W17, DIVERGENCES #315; shared with Perl): latex.ltx's `\@kernel@after@begindocument@before` also runs `\init@series@setup` (latex.ltx:13902-13920: `\reset@font`, `\mdseries`, `\let\seriesdefault\f@series`), which Rust does not run. `\documentclass{article}\itshape\bfseries\begin{document}Body text.` — pdflatex upright CMR10; Rust and Perl `<text font="bold italic">`.

### Fragid Parity Open Items
1. **`associateNode` (Post.pm L508-585) unported**: Generated MathML/OpenMath nodes carry no `xml:id`, so `convertedIDs` and pmml↔cmml parallel cross-linking do not exist.
2. **`in_page_id` lacks `labelids` and `split_from_id` branches**: Affects `--splitnaming=label*`.
3. **`strip_ref_display_fragids`** (crossref.rs:131): Matches `//ltx:ref//*[@fragid]` wholesale; narrow to IDs absent from the ObjectDB.
4. **`make_sub_collection_documents` returns `vec![]`** (collector.rs:141): `--splitindex`/`--splitbibliography` drop entries past first initial.

### Corpus Triage Quick Wins
- **A paragraph's trailing whitespace: Perl and Rust split its text into different nodes** (56jy; RED repro `tools/perfect_kernel/repros/block-model/paragraph_trim_text_node_split.tex`): both trim with `s/\s+$//` (all Unicode whitespace; TeX_Paragraph.pool.ltxml:184-196, `document.rs` `trim_node_right_whitespace`) and only the last text node. Perl appends the space after a formula with XML::LibXML `appendChild` (Document.pm:1148-1153), which never merges, and a formula's own text with `appendTextNode` (Document.pm:2123, :2133), which merges into the text before it; Rust's `add_child` (xmlAddChild) merges the space, and `replace_node_as_tree` keeps the formula's text a node of its own. `\item $\,$ ` gives Perl `<p>U+2006</p>` (its golden `t/parse/artefacts.xml`), us `<p/>`; `\item A $\,$\par` Perl `A`, us `A `. `cleanup_Math` runs as the Math element closes in both (Document.pm:1239-1248, tex_math.rs:756), so ordering is not the cause. Cosmetic; a faithful fix appends text without merging, which touches many goldens. Where the trim empties the last text node, Rust frees it and Perl keeps it empty, which Perl's `isVAttached` then counts (KNOWN_PERL_ERRORS #311).
- **Test-suite hygiene (57z)**: the logger echoes through `eprint!` (`logger.rs` `stderr_echo`), so `cargo test` captures every test's log lines with its output (a direct `std::io::stderr()` write bypassed libtest's capture and leaked all of them to the terminal); every passing test that prints `Error:`/`Fatal:` exercises an error path and pins it by count (inventory 2026-09-28: 84 of 3,634 tests; incidental fixture errors removed, loose `> 0`/`contains` checks made exact); the streaming sweeps require each swept fixture to convert error-free (the math sweep has its dispatcher; intentional-error and preload-needing fixtures are excluded); `latexml::util::test::error_count` counts every `Error:<category>:` (`missing_file`, `I/O` were missed). Found by the inventory: LANDED 57aa `expl3/tex_jobname_is_the_jobname` (the dump wrote `\tex_jobname:D`, a `\let` of `\jobname`, by value — empty; now, as Perl's `Lt`, a `\let` copy identical to its source is an alias record re-`\let` at load; copies a preload takes before the job's `\jobname` exists are re-pointed at install, `core_interface::install_jobname`: `\c_sys_jobname_str` is the job name with and without a preload; RUST-ONLY; DUMP_DESIGN "Format"); open: RED `boxes-groups/nicefrac_text_argument_is_text` (nicefrac's parts are text in `\mbox`, the binding digests them in math; PERL-ORIGIN); forest binding gaps `\@escapeifif` (forest.sty:203) and `\forest@file@copy` (:8430), used by forest-index.sty (RUST-ONLY, stub binding), and `\lst@InstallKeywords` (lstmisc.sty:618, lstdoc.sty:178; SHARED) — pinned by `perfect_kernel_batch56::forest_docinput_lstenv_writefile_gobbles_doc_percent`; the parked pTeX-format registers (`\hour`, `\jfam`, `\kanjiskip`, §D9) behind jsarticle/jarticle/otf; tests that still mutate the process env (race under `cargo test`): `117_post_2gib_handoff.rs:82` (`LATEXML_POST_MEM_PARSE_LIMIT`), `118_streaming_split_parity.rs:142-216` (`LATEXML_POST_STREAM_SPLIT`/`_THRESHOLD`), `identity.rs:113` (`SOURCE_DATE_EPOCH`) — migrate to thread-local overrides as `datatool_sty`/`pgfkeys_code_tex::set_native_override`; RED `loader/package_options_keep_protected_macros` (a robust or e-TeX `\protected` macro in a package option is expanded on the option path: pdflatex stores `k=\protect \lxr`, `j=\lxp`; Rust `k=\protect \protectR`, `j=P`; the chinesechess guard `package_options_are_stored_by_protected_xdef` no longer discriminates).
- **simplebnf-doc warns `unexpected:\fatslash … should only appear in math mode`** (sweep #130, 0 → 1; new since sweep #129): stmaryrd's `\fatslash` (loaded `\AtBeginDocument` with `[only,fatslash]`) inside the manual's tcolorbox `example` re-typesetting (`lx-example-1.tmp`); a minimal `bnf` + stmaryrd probe is clean in both builds and Perl, so the trigger is the example's re-read. Likely 57n's requireMath for every DefMath constructor meeting a text-mode re-read; confirm against Perl on the manual before fixing.
- **An index/glossary entry is digested at its mark** — the `&` case LANDED 59k (a literal `&`; KPE #423, DIVERGENCES #411; robustsample 2 → 0; `index/glossary_ampersand_robustglossary` GREEN). Open (ARCHITECTURE_THEMES §4a): TeX only writes the entry and re-reads it at `\printindex`; a phrase inspected after `do_expand_partially` (mod.rs) has lost `\begin`/`\(`, so arrays in index entries stay unhandled.
- **biblatex `editortype` is not the editor's role** (56kc review; RED repro `tools/perfect_kernel/repros/index-bib/biblatex_editortype_role.tex`): every editor prints "(Ed.)" where biber prints "vocalist Billie Holiday" / "Kilo, Karl, comp." (biblatex.def:2875-2892 `editorstrg`). Giving the role in the BibTeX reader (tried in 56kc) broke the author-year label of author-less entries ("(Hitchcock, 2000)" → "(52)", cms-dates-sample; the fallbacks at make_bibliography.rs ~1076/1212/1354/3640 read only `@role='editor'`), changed `.bst` bibliographies, and exploded an undecoded (ctex) `editortype` byte string. Shape: in the biblatex binding, keep the editor for label/sort/head, carry the type separately, from the decoded field value.
- **A template with no column raises "Extra alignment tab" on every row** (56jz root-causer; SHARED with Perl, Alignment.pm:136-145/240): `\begin{tabular}{}` (or `{|}`) gives an error per row start even without `&`; TeX's preamble always has a column (tex.web:15367-15375, "Missing # inserted" :15468), array adds one for an empty preamble with "Empty preamble: `l' used" (array.sty:332, 412). Shape: in `read_alignment_template`, no columns and no repeated columns → one `l` column plus that error. Witnesses platex-tools/plextarray, plextcolortbl (plext's `\begin{tabular}<t>{…}` direction argument, pTeX family, parked).
- **Measured box sizes, K18 step 2** (step 1 landed in 56kj; repro `tools/perfect_kernel/repros/boxes-groups/box_dimensions_measured.tex` GREEN): `\raisebox`'s raise measures the box since 61v, with `yoffset` rendered as `\raise` (KPE #466; repro `boxes-groups/raisebox_raise_measures_the_box.tex` GREEN). A `\makebox` in script math measures its text at the script size (RED repro `boxes-groups/makebox_in_script_math_text_size.tex`, 2605.18608). `\resizebox{\width}{!}{…}` is `xscale="0"` and `\resizebox*` behaves as `\resizebox`: `\Gscale@@box`'s `GraphixDimension` sizes are not yet read with the box set, as 56kl's `TempboxaDimension` are (4 papers). `\includegraphics[height=1em]` measures 10.1178pt (pdflatex 10.00002pt): its size goes through whole pixels at 100 DPI (RED repro `graphics-tikz/includegraphics_size_not_pixel_rounded.tex`). An empty `\raisebox{d}[]{x}` reads 0pt with Missing number where latex.ltx treats it as not given (0 uses; RED repro `boxes-groups/raisebox_empty_height_is_not_given.tex`). `\resizebox{1em}{!}{x}` measures 10.00002pt, Perl's, where pdflatex gives 10.00081pt through graphics.sty's `\Gscale@div`. makecell (`makecell_sty.rs:192-193`), diagbox (`diagbox_sty.rs:288-289`) and rotate.sty's `\rotate[u|f]` (`rotate_sty.rs:43-45`, Perl rotate.sty.ltxml:44-46 stores Dimensions) still store string sizes, which `compute_size_and_cache`'s complete-size bypass (56kq) cannot read, so the box is measured and reports its unrotated content; rotating.sty shares `rotated_properties` and is typed. Still unported from Perl's `computeSizeStore` (Box.pm:266-297): a partly specified size keeping its given parts over the computed ones, and `isEmpty` (`docs/archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md` U2/S5). `set_width`/`set_height`/`set_depth` (`latexml_core/src/lib.rs`) write only the declared value, where Perl's `setWidth` etc. (Box.pm:211-230) also update the computed copy, and `fobj_get_size` (`tex_box.rs:75-118`) prefers a stored computed size and never computes a missing one (Perl `$whatsit->getSize`, TeX_Box.pool.ltxml:410): a `\ht` assigned after a size was computed stays stale for a foreignObject unless the box's size is complete (56kq's bypass then applies). Needs a ruling: `\ ` is 0.5em wide (Perl's constant, TeX_Character.pool.ltxml:28-30) where TeX uses the font's interword space (`\spaceskip` if nonzero, tex.web §1041-1044): a node with a control space is 0.167em per `\ ` wider than pdflatex. Also open: a detector for a String-valued size key in `compute_size_and_cache`, and an audit of box constructors without a sizer (79 `sizer =>` sites against 945 `DefConstructor!`). See `docs/perfect_kernel/KERNEL_CAPABILITIES.md` K18.
- **babel-french high punctuation** (residual of Gemini round 13, KPE #383): a space inside a closed group (`{\bfseries gras }?`) stays; a negative skip is removed where TeX removes only one above 1sp.
- **K13 stage-2 findings: open residuals** (57h audit, `tools/perfect_kernel/binding_conformance.sh`; the landed rounds
  57i-57y, 57ad, 57ah, 57aj and Gemini round 13 are verbatim in `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`;
  verify each against the real `.sty` before fixing). K13 stage 3 (environments' `\endX`, classes, the allowlist) not started.
  - *setspace/titlesec*: RED `block-model/setspace_display_math_in_doublespace`; titlesec — an alignment switch in the
    before-code aligns an inner `<text>` (#344), a `leftmargin` after-code ends the title where TeX starts the next
    paragraph with it, `explicit` as a global class option unseen, `uppercase` (`\ttl@case`) unread, keyed
    `\titleformat{name=\section,numberless}` defines a junk macro (13 A/B papers: googledeepmind.cls, mystyle.cls); RED
    `sectioning-frontmatter/unnumbered_section_takes_the_title_font` (`\@@unnumbered@section`, Perl
    latex_constructs.pool.ltxml:666-670; ~70 A/B papers; needs tufte's `\titleformat` `[after]` first).
  - *floats/captions*: RED `captions-floats/par_breaks_a_panel_row`; RED
    `captions-floats/continuedfloat_minipage_subcaption_continues_the_letters` (no `\caption@subtypehook` pre-step,
    caption.sty:642); under subfig a third continued part keeps the first continuation's sub-float id suffix (`S0.F1a.sf4`
    under `S0.F1b`); `[caption=false]{subfig}` still gets `\caption@ContinuedFloat`'s check; caption's
    `continuedfloat`/`continued<type>` option sets (caption.sty:526-528) unapplied; RED
    `captions-floats/longtable_starred_caption_keeps_its_text` (KPE #363); longtable ignores `\LTcaptype`; floatfig/JHEP
    `floatingfigure`/`floatingtable` and elsart `algorithm` typed without a source; RED `captions-floats/subref_prints_the_letter`;
    `subfloat_sty.rs:29-30` `\fnum@sub…` = `(\thesub…)` gives "((a))" under `labelformat=simple|empty` (2605.01267,
    2605.10017, 2605.05371; `subfig_sty.rs:128` moved to `\lx@subfig@fnum` in Gemini Q8 — re-verify which still
    double-wraps); jfm.cls/interact.cls `\doi` setters (KPE #326); the XSLT DOI-pubnote predicate (DIVERGENCES #345) has no
    HTML guard; `\rotcaption*`/`\rotcaptionof` undefined, `\controtcaption` unnumbered (57l); a `\rule{\linewidth}` panel's
    `width="100%"` is unmeasured by the panel rows (57by, 2605.18774; KPE #389).
  - *loader/kernel*: neither the loader nor `\@ifpackageloaded` honours a `\ver@<pkg>.sty` mark (2605.20200; memoir
    `\EmulatedPackage`; honouring it broke 2605.08402's `\listoffigures*` on the 57bu A/B; the loader half retried in
    66m, KPE #668, awaits a ruling); Rust's `\@addtoreset` omits
    Perl's `defCounterID … unless \the<ctr>@ID` (latex_constructs.pool.ltxml:3044; 57ac).
  - *bibliography*: natbib's `\bibpostamble`; KOMA's `\setbibpreamble` without natbib; apacite's citation layer is natbib's
    (`\cite` textual, `\citeA` abbreviates).
  - *anchors and hooks*: RED `block-model/hyperdef_starts_the_paragraph`, RED
    `block-model/hypertarget_heading_a_paragraph_keeps_the_space` (both shared); math in a new anchor takes ids chained from
    the anchor's (`m.m1`); RED `macro-state/afterendpreamble_code_is_followed_by_ignorespaces` (RUST-ONLY); RED
    `macro-state/atbegindocument_in_the_body_runs_now` (fire `\document`'s hooks with `\hook_use_once:n`, then drop 57r's
    `\AfterEndPreamble` flag).
  - *layout and lists*: RED `boxes-groups/rotatebox_starts_the_paragraph` (shared); RED `fonts-nfss/ulem_custom_mark_is_rendered`
    (41 of 30,079 2605 papers; the class mapping is a design call); RED `block-model/noindent_paragraph_in_a_minipage_takes_its_id`
    (same defect as 58g's RED `noindent_paragraph_in_minipage_numbers_inside` — merge the two); RED
    `fonts-nfss/textcircled_of_a_box_circles_its_text`; RED `list-structure/enumitem_ref_clears_the_parent_prefix`
    (enumitem.sty:551-556); RED `list-structure/endtrivlist_after_a_heading_starts_a_paragraph` (61p review; OD #435
    residual); RED `string-mouth/verbatim_sty_starred_shows_visible_spaces` (verbatim.sty's `{verbatim*}` without its
    visible-space setup; SHARED); level-only `\setlist[1]` (re-verify after 57u's KPE #354); `\optc[x]` in a message prints `\optc [x]` (KPE #381
    residual). Settled dead end: a separate counter family for kernel lists fixes the label but changes Perl's ids.
  - *57ah residuals*: svn-multi's keywords are pdflatex's pass-2 `.aux` values; savetrees does not load titlesec/geometry for
    `sections`/`margins`; `\MakeUppercase{\authors}` leaves the kernel `\and`'s " and " lowercase; varioref has no
    per-language range words; RED `sectioning-frontmatter/amsart_uppercasenonmath_textcase_keeps_the_title` (textcase
    `\altucnm`, SHARED); RED `macro-state/kernel_Ref_is_a_capitalised_reference`.
  - *math (PARKED 2026-10-02; verbatim in the archive)*: `math-parse/left_delimiter_outside_the_map` GREEN since 65d
    (golden `tests/parse/left_right_delimiters`), RED `math-parse/differential_needs_its_integrand`, RED
    `math-parse/lxdeclare_matches_a_font_styled_symbol`; the 57ag route-dependence and grammar residuals (2605.11385 S2.E18,
    2605.02221, 2605.30618, `x_1,x_2,x_3\in X`, `\otimes_k E:=F`, `[n]\setminus\cup_k C_k` 71 formulas/12 papers,
    `<r_i>_n^*`, `x^{\prime,}`, `x_{a,\qquad}`, `f(x)+\phantom{g(x)}+h(x)`, `gathered` `vattach`, `ams/mathtools`
    S10.Ex76's `\qquad`, `\sum_k\beta\log\varepsilon_k`, `\max_i\max_j a_{ij}`, `\operatorname*{argmin}_z`); 57ae's
    `\DeclareMathOperator{\Tr}{Tr}` lacks Perl's `name="Tr"`.
  - Kept as is: MnSymbol/fdsymbol `\not` (2605.05506; `\mathrel` wrapping is a dead end: ≠ → `not@(=)`), fontawesome
    `\faicon`, the superset bindings (nameref `\Nameref`, subfigure `\subref` `*`, lineno `\linerefp`); walker limits
    (subfig `\DeclareSubrefFormat`, apacite `\BCAY`, soul `\caps`, eufrak `\mathfrak`).
- **A block box at a paragraph start splits the paragraph** (56kb review; SHARED): `\rule`, `tabular` first in a paragraph (latex.ltx 16361, 16560 start with `\leavevmode`) come out as separate blocks, and the text after them ("et B.") in a paragraph of its own. `\parbox` (16250) fixed in 57d and `minipage` (16306) in 57f (KNOWN_PERL_ERRORS #309, DIVERGENCES #338).
- **Cells at a column rule or `@{..}`: alignment and padding classes** (56jy review; RED repro `tools/perfect_kernel/repros/alignment/column_rule_edge_padding.tex`): a leading arydshln `:` right-aligns the first `c` cell (Perl `align="left"` with `ltx_nopad_l`; pdflatex centres it), and the cell keeps the rule's U+2002 U+200A gap as leading text, doubling the CSS padding; `c@{\,}c` marks no `ltx_nopad_l` on the right cell where Perl does. Shared with Perl: a `c` column before `:`/`;` reads `align="right"` `thead="row"` (Perl's right-to-left cell scan stops at the class command, TeX_Tables.pool.ltxml:505-520), and the `@{..}` material itself is dropped.
- **Scanner-status residuals** (batch 56jg, DIVERGENCES #310): commands Rust implements as primitives/constructors (`\setcounter`, `\lstnewenvironment`, `\newtcbinputlisting`) need a primitive-level abort for TeX's matching recovery; `skipping` is not modelled.
- **A file end in a `GeneralText`/`XGeneralText` brace hunt is not an error** (batch 56jj residual, DIVERGENCES #313): `\uppercase`/`\lowercase`/`\detokenize`/`\message`/`\errmessage`/`\mark`/`\marks`/`\special` as a file's last token — pdflatex 2 errors ("File ended while scanning text of \uppercase", "Missing { inserted") and "A abc B", Rust and Perl 0 and "A ABC B". Order: stop setting `matching` around expandable primitives' typed parameters (`Expandable::read_call_arguments`, `definition/expandable.rs:325-340`; `\csname`, `\number`, `\readline`), then let `read_x_token` call `recover_at_file_end` at non-normal status; the brace hunt (`skip_filler`) also runs at the caller's status and skips no `\protected` expansion, where `read_tokens_value` hunts with protected expansion — one hunt for both. Red repro `tools/perfect_kernel/repros/expansion-primitives/uppercase_brace_hunt_file_end.tex`.
- **A braced typed argument's rest: the residuals** (batch 56jr resolved the silent loss, OXIDIZED_DESIGN_DIVERGENCES #317 / KNOWN_PERL_ERRORS #275: the rest re-enters the input after the command, assignments scan by the register's type, calc — loaded in most arXiv papers — evaluates a braced length whole; an undefined control sequence the scan met is discarded, as TeX does, and a rest met between alignment rows is dropped with a warning: the review regressions 2605.25073, 2605.08378, 2605.27476, 2605.25272 are fixed; batch 56ju fixed the TL-manual A/B regressions, each a binding that read less than TeX — multido's Number variable, nicematrix `X[<keys>]`, tabularray `\NewColumnType` and rule options, pstricks angles read whole, `\resizebox` lengths through calc, calc loaded by the pdfcomment/diagbox/animate/savetrees/breqn/jmlr bindings as by their packages, `\DeclareTextAccent`'s stored slot). Open: `\width`/`\height`/`\depth`/`\totalheight` are `0pt` inside `\resizebox` (sect12.rs:184-187, shared with Perl; perfectcut's delimiters get `xscale="0"`); tabularray's `cmd=` key is unapplied; a box or space command's tail comes one step late — `A\hspace{1em\foo}B` gives "A xB" with a warning (pdflatex "Ax B"), inside a `minipage`/`tabular*` body for those; `\vspace` is Perl's `\vskip #2\relax` (latex.ltx:9362-9372 sets `\sp@ce@skip` first); a keyval value's rest (`Parameter::reparse`, keyvals.rs) is still dropped; a column type's rest (`p{3cm\foo}`) is dropped with a warning where TeX typesets it in each cell (2605.19386 `;{1pt/1pt}` under the arydshln stub); a `DigestedBody` constructor's (`\@@tabularx`) comes after the whole body; `\lx@@genfrac`'s `after_digest` (amsmath_sty.rs) reads its numerator after a re-inserted rest. Repros `tools/perfect_kernel/repros/expansion-primitives/braced_*`, guards `braced_quantity_tail::*`.
- **`\ifx` ignores `\protected`/`\long`/`\outer`** (W17 final review, pre-existing; shared with Perl `Core/Definition/Expandable.pm:116-120`): `latexml_core/src/definition/expandable.rs:107-110` `PartialEq` compares only parameters and expansion, where tex.web §507 also compares the command code and eTeX's `protected_token`. `\def\a{x}\protected\def\b{x}\long\def\c{x}` + `\ifx\a\b`, `\ifx\a\c`: pdflatex "FF", Rust and Perl "TT". RED repro `tools/perfect_kernel/repros/macro-state/ifx_ignores_macro_prefixes.tex`. The fix has to keep `x_equals` callers that compare binding copies (`\bibitem`/`\restoring@bibitem`, `\document`/`\lx@orig@document`) equal.
- **`\shipout`/`\setbox` residuals** (batch 56jk, DIVERGENCES #314): `read_box_operand` invokes a non-box operand (`\shipout A` typesets "A", `\shipout\hrule` a rule) where TeX §1084 reports "A <box> was supposed to be here" and backs the token up; a shipped `\hbox` in horizontal mode runs into the surrounding text (emit it as its own unit, as a `\vbox` yields an inline-block); `box_prefix_tokens` (`tex_box.rs`) reads `\everyhbox` before the `\afterassignment` token, where TeX reads the after-token first (§1083 then §1269 back_input; TeXbook p.279) — Perl the same (TeX_Box.pool.ltxml:170-174), a KNOWN_PERL_ERRORS candidate once a pdflatex probe pins it; `repros.sh:63` parses `% status: RED (…) -> GREEN (…)` as RED (98 repros use that form), so their regression check is off.
- **doc.sty's code-line index and the index-list residuals** (batch 56js, DIVERGENCES #318): a dtx manual's index lists only its `\index`-level (usage) entries — doc.sty's `\codeline@wrindex` (doc.sty:855-862), hypdoc's `\HD@codeline@wrindex` (hypdoc.sty:332-343) and l3doc's `\__codedoc_index_page_hc:nn`/`_codeline_hc:nn` (l3doc.cls:1899-1914) write `\indexentry{…}{…}` to `\@indexfile` themselves, which is allocated but never opened, so the text goes to the log (source2e: 44,246 `\indexentry` lines). A `\write` to an index stream could become an `ltx:indexmark` read by the same makeindex reader. Also open: imakeidx documents get no index at all (imakeidx.sty:147-156 redefines `\index` to write the file itself; e.g. assoccnt_doc, atableau); robustglossary's `&`-column `\glossary` entries raise `Stray alignment "&"` at the mark (robustindex/robustsample, 2 errors; pdflatex prints them through its own `\glossaryentry` table; RED repro `tools/perfect_kernel/repros/index/glossary_ampersand_robustglossary.tex`); xindy styles' separators are read as text (makeglos.xdy `:`); `\@SpecialIndexHelper@` keys of control symbols stay garbled (amsldoc `\cn{\\}`). The re-read of the written entry (a `\` plus letters make a control word when `\printindex` inputs the `.ind`) is modelled only for `\string\verb`: amsldoc.cls:99-103's `\string\texttt{#2}` prints `“texttt…` — RED repro `tools/perfect_kernel/repros/index/index_string_command_amsldoc.tex`; one re-read step after the entry's expansion (skipping the absorbed `\@internal@text@verb` arguments) would also fix doc.sty's `\LeftBraceIndex`/`\RightBraceIndex` displays. `\verb*` in an entry shows no visible spaces; under hypdoc the encap style is `hdpindex`/`hdclindex` rather than doc's `usage`/`main`. A list-of-figures entry loses the space after its number (`1Cap`; Perl and pdflatex `1 Cap`): Scan stores `caption`/`toccaption` as text (scan.rs `captioned_handler`), which drops the tag's `close`; storing the `cleanNode` copy as a node, as Perl does, would keep it (repro `index/indexmark_cleaned_from_titles_captions.tex`).
- **A `\gls`/`\glsadd` inside display math lists nothing** (batch 56jt residual, DIVERGENCES #320): the location-only `ltx:glossaryref` of a formula goes to the nearest enclosing level that admits it, after the formula; inline math has its `p`, but `ltx:equation` admits no inline element, so an entry named only in display math (`\begin{equation} \gls{E}=mc^2 \end{equation}`) is missing from the list. `\aftergroup` carries tokens past `$…$`/`\[…\]` (probed), but not safely out of an alignment cell.
- **`{filecontents}{name}` overrides a `name.tex` on disk** (batch 56jt residual, KNOWN_PERL_ERRORS #279): since `\openout`/`{filecontents}` name the file `name.tex`, the source's contents shadow a `name.tex` beside the document, where LaTeX's default (no `[overwrite]`) keeps the disk file ("File `name' already exists on the system. Not generating it from this source.", latex.ltx:19004). Probe: `democode.tex` = "Disk version." + `\begin{filecontents}{democode}Source version.\end{filecontents}` + `\input{democode}` — pdflatex and the pre-56jt binary print "Disk version.", now "Source version.". No fix yet: the store must consult the disk before a non-`[overwrite]` write.
- **Fixture exception to the 0-warning rule** (batch 56jt): `stream_a_recall::a_citation_of_another_lists_item_is_missing_here` asserts its one warning, "Missing bibkeys: supp" — the faithful diagnostic (Perl MakeBibliography.pm:342-343; bibtex warns "I didn't find a database entry"), since the key it guards is cited and absent from that bibliography's files by construction.
- **biblatex refsections share one citation list** (batch 56jt residual, KNOWN_PERL_ERRORS #277): each bibliography now reads its own `@files`, but every `\printbibliography` fallback lists every cited key its files hold and warns "Missing bibkeys" for the rest, where biber lists and checks only its refsection's citations (RED repro `index-bib/biblatex_refsections_share_citations.tex`: 2 warnings, Globalentry in the first list); `\printbibliography[section=N]` after the refsections reads the pooled resources (biblatex-apa6-test).
- **babel language tags are lost under the `luatex` profile** (batch 56ji residual): babel loads `luababel.def`, so `xml:lang` is not set; `babel_support_sty.rs:243-254` also forces LGR for Greek (latent). Red repro `tools/perfect_kernel/repros/babel-lang/luababel_language_tag_luatex.tex`.
- **Rust `replace_tree`/`replace_tree_detach` keep the replaced node's box in its parent** (W12 residual, document.rs): Perl's `replaceTree` (Document.pm:2079-2091) runs `removeNode` on the attached old node, so its box leaves the parent's (and each auto-opened ancestor's) box before `appendTree` adds the copy's; Rust's copy-then-detach adds the copy's box without removing the old one. Callers: the math parser (`replace_tree_deferred`), the `svg:foreignObject` cleanup (tex_box.rs). Only `cleanup_math` has Perl's `replaceTree` boxes (`Document::replace_node_as_tree`).
- **A text-only `svg:foreignObject` is not renamed to `svg:text`** (W12 review probe t5, pre-W12): Perl's foreignObject `afterClose` renames it (TeX_Box.pool.ltxml:386-389); the Rust port (tex_box.rs, its comment says it does) has no such branch. A tikz `\node[draw,align=center]{A\\BB}` gives Perl two `<svg:text>` lines, Rust a 10.38×9.46 and a 19.6×9.46 foreignObject, and its cells lack Perl's `ltx_nopad_l` class; `$\phantom{H}$` in a node gives Rust a foreignObject holding nullfont spaces.
- **The foreignObject width override is stale** (W12 review, `latexml_engine/src/tex_box.rs:885-918`): an auto-opened `svg:foreignObject` wrapping a width-bearing `ltx:inline-block` takes that block's `width` attribute in place of its box's width, on the premise "appendNodeBox creates Lists that sum widths incorrectly", which W12 retired (DIVERGENCES #319). It makes `\node[draw]{\fbox{\parbox{2cm}{Hello world}}}` 78.73 px wide (Perl 88.15). Retire it in its own A/B batch.
- **`\node{x $\text{ab}$ y}` is 37.08 px wide, Perl 49.63** (W12 review probe t1, pre-W12): Perl measures an undefined `\text` (no amsmath) as its literal name inside the Math.
- **The pgf shading node `\node{YAY!\pgfuseshading{msh2}}` is 227.16 px wide, Perl 230.62** (unit_tests_by_silviu): Perl keeps a space before the shading picture that pdflatex does not set (`\wd` of `\hbox{YAY!\pgfuseshading{msh2}}` = 123.99pt = "YAY!" 23.61pt + shading 100.38pt); the ~1.7pt Rust still lacks is the Y–A kerns across separate text boxes.
- **Figure panels under `geometry` are measured at the class `\textwidth` but arranged at `\Gm@tw`** (pre-existing; `latex_constructs/mod.rs` `after_float` floatwidth override, OXIDIZED_DESIGN #99): in 2605.03152 (`[margin=1in]{geometry}`, S3.F2) the `0.41\textwidth` subfigures are 141.5pt wide (0.41 × 345pt) against a 469.8pt row. The W12 fixup's panel-row change (KNOWN_PERL_ERRORS #274) turned its rows from 2 | 2 (56jk, pdflatex's layout, reached through Perl's row-width carry-over) into 3 | 1; pdflatex (0.41 × 469.75pt) sets two per row, separated by the source's blank line, which the rows ignore (Perl alike).
- **A merged number's box is wider than its glyphs** (W12 fixup review, pre-existing): the math ligature's composite box is a math-mode `List` of the merged tokens, which adds ~3mu. `\node{$2.414$}` is a 33.82 px foreignObject (Perl 31.52; pdflatex 22.78pt), `x \put(0,0){$2.414$} y` a 45.74 px picture (Perl 43.43).
- **A declared UTF-8 character is case-changed through its LICR** (batch 56jw residual): pdfTeX's `\MakeUppercase{ά}` keeps the character (its lead octet is `\protected`, utf8.def:180), maps the code point to U+0386 and typesets that LICR, "΄Α"; the case changer expands `ά`'s LICR `\ensuregreek{\acctonos\textalpha}` (lgrenc.dfu:125) and textalpha drops the tonos of `\acctonos`, "Α" (repro `unicode-catcodes/case_change_text_command_escape.tex`). Keeping the declared character and casing its code point (tried in 56jw) gives "΄Α" but loses the accent drop of the Greek locale, which l3text does on code points (`\__text_change_case_upper_el:nnnnn`, expl3-code.tex:37019-): char-list's babel-greek `\TestUppercase` lines lost 20 words (recall 74.9 → 71.6 %). Both belong together: the code-point path with the `el` rules and `\DeclareUppercaseMapping` (a no-op in sect08.rs).
- **Binding closures that assign as they expand: the unflagged rows** (batch 56jw audit, KNOWN_PERL_ERRORS #286): a number scan still runs `\IfFileExists`/`\InputIfFileExists` (sect13.rs, robust in latex.ltx:9667/9710), `\AtEndOfPackage` (sect05.rs), `\ProvidesPackage`/`\ProvidesFile`, `\@startsection` (sect04.rs), the `\e@alloc` allocators, amsthm `\swapnumbers`, paralist `\setdefaultenum`/`\setdefaultitem`, xkeyval `\setrmkeys`/`\key@ifundefined`/`\disable@keys`/`\DeclareOptionX*`, multido `\fpAdd`/`\fpSub`, contrib biblatex `\DeclareCiteCommand`/`\DeclareMultiCiteCommand` and chemnum `\resetcmpd` (tabularray's `\lx@tblr@env`/`\lx@tblr@portrait` set the after-table state and raise the unknown-type/theme errors while expanding — internal, called only from the environment), whose originals reach an unexpandable token before they act. None has a corpus witness; each wants its repro before `peeks_by_futurelet`. ntheorem's `\theoremheaderfont` and siblings are macros where ntheorem.sty:628-634 declares token registers.
- **A constructor template's `?#N` holds for "0"** (W12 review; `latexml_codegen/src/constructable.rs:301-310` `emit_bool`): Perl's conditional is the argument's string (Constructor/Compiler.pm:166), false for "0" and "". `\qbezier`/`\bezier` work around it (DIVERGENCES #316); the generic fix changes many goldens (its own batch).

- **Residuals of batches 57cn–59l** (rc132 schema roots, then the scoreboard batches; LANDED prose verbatim in
  `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`, one row per 59 batch in `perfect_kernel/LEDGER.md`). Open:
  - *hyperref anchors* (57cn.1, 57cp, 58i): `\hyper@anchor`/`\hyper@anchorstart`/`\hyper@anchorend` still gobblers
    (hyperref_sty.rs:693; fdudoc.cls:921-922; multibibliography/amsrefs/bibtopicprefix `cite.<key>`); a vertical-mode
    `\Hy@raisedlink{\hypertarget{v}{}}` opens a paragraph (hyperref.sty:2098-2102; tudscrmanual.cls:524, reledmac
    `\edlabel`); an anchor in math loses its name (`$a\hypertarget{m}{}b$`); a stale `\@currentHref` names memoir pagenote
    anchors (`target-star-.3`, ucalgmthesis); glossaries-extra `\printunsrtglossary` re-digests a label's anchor into its
    `typerefnum` (`glo..fooa`); `\Hy@SaveSpaceFactor` family undefined; RED `singletons/anchor_at_paragraph_start_keeps_the_space`. A `\hypertarget` around a block in vertical mode (a `\parbox`, after `\item`, in `quote`/`minipage`/`p{}` cells) puts its destination in the paragraph after the block, one block late (59m, KPE #424: exact placement before it would need a vertical-mode destination, which `\hypertarget{n}{\section{T}}`'s in-title convention rules out).
  - *tabularray* (57cp; `evaluate=`/`expand=` landed 59n): rules (`hlines`, `vlines`, `hline{…}`) are not drawn, so an
    all-empty table is pruned whole (RED `alignment-bindings/tblr_all_empty_table_keeps_its_rules`; the manual's
    `\makeEmptyTable` demo, tabularray.tex:2859-2870); a math table without a colspec is sized before `evaluate=`/`expand=` run (RED `alignment-bindings/tblr_math_columns_follow_the_expanded_body`); custom `caption-tag`/`caption-sep`
    ("Quadro 2 —", tabularray-abnt); a long/tall table in a captioned `figure`/`subfigure` nests a `table`; document
    templates print as tabularray's own (tblr-extras head, `label=none`); a math table's `tex=` holds the reduction;
    `\SetCell[c=…,r=…]` spans, `rownum`/`colnum`/`rowcount`/`colcount`; `cmd=`/font styles; the LoT line of a caption-less
    long table. Settled (pdflatex errors too): `\caption{\url{…#1}}`, floats in minipages, `{l→}`.
  - *floats/captions*: RED `captions-floats/{two_captions_in_one_float_are_numbered_apart (KPE #396),
    subfig_subfloats_restart_in_each_figure, float_in_a_subfloat_row_keeps_the_parent_number (KPE #395 part),
    captionsetup_type_in_a_figure_minipage, captionless_longtable_is_not_listed, label_after_a_continued_caption,
    label_after_a_caption_with_a_note_names_the_float (SHARED), cleveref_optional_label_type_names_the_label (2605.13648),
    filecontents_graphic_has_its_size}`, RED `boxes-groups/caption_minipage_in_a_table_cell`; text a `\refstepcounter`
    rebinding typesets is dropped; floatrow — a footnote ignores `\def\@mpfn{mpfootnote}` (floatrow.sty:476-478), a
    `\floatfoot` in a plain `table` and a `\DeclareNewFloatType` float are lost (our float environments shadow
    `\float@end`, :114-127, :421-424), a floatbox caption lacks `ltx_centering` and a zero-size `<rule>` follows the body
    (:392); float placements are unchecked (`[!H]`); a tcolorbox listing's title/number is not printed (59i).
  - *box measurement* (58b-58e; K15/K18): `\vsplit` never splits a paragraph into lines (reledmac
    2-titles_in_line_numbering_with_notes 16 numbered lines vs 28); `\penalty` leaves no item, no interline glue or
    `\splittopskip`; `{x\par y}` measures one line more; `\lastbox` removes any last item; a note counts as a line break,
    plain `\footnote` is measured with its body, a minipage footnote's text is uncounted (DIVERGENCES #391); size10.clo
    leadings whatever the class size; a footnote keeps the surrounding leading; RED `alignment/booktabs_rules_add_their_spacing`;
    RED `boxes-groups/negative_kern_overprints` (2605.14271); 2605.02221's `\rTo{\pi}` sequences now parse as fragments;
    reledmac — `\patchcmd{\endminipage}` fails (reledmac.sty:7643-7654), `\footnote` in `\pstart` splits the line.
  - *fonts* (58f, 58g, 58k, 59d): a raw font's `\fontdimen` reads cmr10's, `\fontcharwd`/`\fontcharht` the undecoded char;
    the lcircle map is unchecked; `\ding` 7.5pt vs 5.71pt; a standard-metric font's missing char (KPE #408); `\f@encoding`
    inside a raw font's group; RED `fonts-nfss/math_char_without_glyph_keeps_its_box` (`t1_guillemet_ligatures` GREEN 60s); T2A ј/і as
    Latin j/i (surpass candidate); a redefined `\everymath` is not run (latex.ltx:10580).
  - *lists*: RED `list-structure/stale_list_counter_reset_gives_I0_ids` (Perl-origin; 2605.04012); a `\list` on the
    enclosing counter inside `\lx@trivlist@setup` gets `i0`; RED `block-model/noindent_paragraph_in_minipage_numbers_inside`
    (2605.21713; same defect as the K13 row's `noindent_paragraph_in_a_minipage_takes_its_id`).
  - *frontmatter* (58m, 58q, 59d-59g): RED `sectioning-frontmatter/{store_set_again_after_maketitle_keeps_the_title_page_value,
    deposit_of_only_punctuation_is_dropped, abstract_store_keeps_the_document_abstract,
    store_default_that_typesets_nothing_leaves_no_element, ijcai_markers_in_an_appending_author}`; bfh-ci
    Factsheet/ProjektProposal lose their department (a footer-layer semantic note, kept under the 2026-10-01 ruling);
    `\addresslink`-style marks `\let` to `\textsuperscript` are not author marks; dtk.cls `\Author`; RED
    `singletons/omnibus_alias_shadows_a_local_package`; g-brief (59o, #412): only a serial document's first letter gets
    its sender/addressee (RED `sectioning-frontmatter/gbrief_serial_letters_each_have_their_frontmatter`), a contact
    label's ligatures stay untypeset in `name=` (RED `sectioning-frontmatter/contact_name_label_keeps_its_ligatures`); acmart — the year without `\copyrightyear`/`\acmYear` is the
    conversion's, a document's own `\footnotetextcopyrightpermission` lands in the first paragraph, the cc logo/link and
    `authorversion` statement are unported, an unknown `\setcopyright` is silent.
  - *bibliography and notes*: `\printbibliography[type=…]`/`[check=…]` all print under the first call (biblatex_sty.rs
    `\biblatex@printbibliography`; `~/data/pk_agents/rc132/repro/typed_print.tex`; the biblatex manual's 244 missing
    words); endnotes `\theenmark` = `\theendnote` (PERL-ORIGIN endnotes.sty.ltxml:29; cms-noteref-demo "18" ×18); enotez
    `split=section` — a `\chapter` starts a split, a pre-heading mark's `\endnotetext` lists in the mark's split, 0.10d
    internals overridden; biblatex-ext-oa prints no open-access mark (RED `boxes-groups/xsavebox_saved_box_prints_its_content`);
    Ruled STAY (D15): biblatex `cite.<n>@<key>` (cms-dates-intro 80, cms-trad-appendix 4) and
    gost `back:` (770) dangling links — the surpass option needs a ruling
    (`~/data/pk_agents/main/agent_reports/2026-10-02_biblatex_idref_rootcause.md`).
  - *kernel*: RED `singletons/register_value_absorbs_as_its_text` (`digested.rs:378`; KPE #403); RED
    `macro-state/generic_command_hooks_run`; the loader ignores `\ver@<pkg>.sty` (memoir `\EmulatedPackage{setspace}`
    loads the setspace binding; see the K13 row); Perl's open-conditional report at end of input
    (Core.pm:229-233) is unported (the `\end{document}` one landed in 65c, sect02.rs); the raw
    babel `.ldf` load stays parked on the `\mdqoff` note (latexml_package lib.rs).
  - *pgfmath/TikZ* (59a-59c): RED `graphics-tikz/{tikzmath_function_draws, pgfmath_width_of_formatted_text}` (both SHARED);
    `random(m,n)` integer, `.5`/`1.` pass through (DIVERGENCES #404); a missing `:` in a chained ternary is silent;
    `\pgfmathsetlength`'s leftover tokens run outside pgf's group; perf levers L1 and L6 landed (59q) and G4 is
    met since s136 (pgf-interference-en ~170 s alone); further levers are optional margin
    (`~/data/pk_agents/main/agent_reports/2026-10-03_g4_pgf_interference_levers.md`).
  - Out of scope (recorded): luatexja `\fontencoding{T1}` prints "T1" (`\luadef` predicate no-ops); biblatex-ext's last
    error, a bibliography inside a tcolorbox (user 2026-10-02); cms-notes-intro (lualatex-unclean).
- **Recall-tail residuals, sweep #130 (rc131; LANDED 57cl-57cm.2 verbatim in the archive)** — golden mismatches are
  curated in `tools/perfect_kernel/golden_reference.tsv` (dinbrief, geradwp, JACoW A4/Letter; `candidate-mismatch` is a
  flag only). Open: RED `graphics-tikz/xml_id_after_a_picture_repeats_its_svg_id` (one-way check); RED
  `graphics-tikz/eqnarray_copy_in_a_picture_keeps_its_references` (`append_clone_aux`, document.rs: copy the svg id as Perl,
  Document.pm:1942-1951, dropping `_svgid` from the clone so `record_svg_ids` renames it); **abntex2cite** (413 missing
  words at s134, the top in-scope S3 doc; -alf 102): `abnt-verbatim-entry` blocks (post, ~+11 %), `\autoref` babel names
  (`\HyLang@DeclareLang`), `reprinted-from`, `conference-*` (abntex2-num.bst:1003-1014), `\hiddenbibitem` (`@hidden`,
  abntex2-num.bst:1379-1390), URL/citation-order rules keyed on the style name (unbtex `unbtex-alf-pt` prints "Link" ×9:
  key them off the `.bst`'s `format.url`/`presort`), and the presentation list (upper-case `alf` author, "e" and ";"
  separators, a multi-key `\citeonline[note]`, authorless entries, `abnt-full-initials=yes`, num "(1)" brackets
  (`\citebrackets`, :213, :232-233), repeated authors (:995-999, :1037-1043), `alf` list labels, note field order, the math
  `tex=` brace group) — repros `~/data/pk_agents/rc131/abntex2/repro/r1–r8`, `review57cm/`; **jacow-collaboration**
  (92.7 %): author continuation lines glued at `\\` (`add_authors_calls`, base_utilities.rs:1235-1239 = Perl
  Base_Utility.pool.ltxml:702-703, PERL-ORIGIN; keep the delimiter as `<break/>`; `~/data/pk_agents/rc131/repro/author_glue.tex`).
  Ruled out: suanpan-l3's last jing line (a second `\title`/`\maketitle` replaces the first — double `\maketitle` keeps
  the latest title, user 2026-10-01).

### Perfect-kernel RED repros without a row elsewhere

The open RED repros of `tools/perfect_kernel/repros/` that no other row names; headers re-checked 2026-10-02 (scope:
PERFECT_KERNEL.md → Scope; a witness whose oracle is unclean is a crash canary). Each row: defect, classification, repro path.

- **`\multirow` in a `p{}` column spans no rows** (PERL-ORIGIN): the `p{0mm}` cell gets no `rowspan` where the `c`-column copy gets `rowspan="2"`, so the rotated label inflates its row (2605.23257 S5.T4 169.5pt vs pdflatex 116.2; A2.T7). Both bindings put `rowspan` on the alignment lookup's current column (`multirow_sty.rs:18-45`), which misses inside a p cell's box. Repro `alignment-bindings/multirow_in_p_column_spans.tex`.
- **`\resizebox` of a zero-size box** (PERL-ORIGIN, K18 step 2 residue): a 0-width box gets `xscale="6553600"` where graphics.sty's `\Gscale@div` reports "Division by 0" and uses 1. Repro `boxes-groups/resizebox_zero_size_division.tex`. (The `pspicture` half landed in 58i, DIVERGENCES #397: `graphics-tikz/pspicture_size_is_typed` GREEN.)
- **`\obeylines` makes the newline an `<ltx:break/>`, not `\par`** (PERL-ORIGIN; OUT OF SCOPE): Perl/Rust let the active `^^M` be `\@break` (`plain_base.rs:467-470`), latex.ltx:575-576 `\obeyedline` (= `\par`), so `\let\par=\cr \obeylines` ends no row. 6 errors, first `Error:unexpected:&`. A faithful fix changes goldens everywhere: ruling needed. Repro `alignment/metre_obeylines_halign.tex`.
- **A `\def` whose parameter text meets `}` swallows text to the next `{`** (SHARED; kernel): tex.web §474-475 inserts "Missing {" and ends the definition at the `}`; `\def … UntilBrace` (`tex_macro.rs:91`) reads on, so storecmd-guide loses two sentences. 2 errors vs pdflatex 6. Repro `expansion-primitives/def_param_text_right_brace_storecmd.tex`.
- **A `\lastbox` loop counts 2 title lines, pdflatex 1** (SHARED): `\lastbox` pops whatever item is last (`tex_box.rs`; Perl TeX_Box.pool.ltxml:596) and paragraphs have no line boxes; needs K15's list tail record. Repro `expansion-primitives/unpenalty_lastbox_sidenotes.tex`.
- **`\keys_set:nn {siunitx}` finds no l3keys module** (SHARED): the binding declares keyval `SIX` keys, so currency.sty:90's raw `\keys_set:nn {siunitx}{round-precision=2}` gets l3keys' unknown-key error (1 `Error:errmessage`; Perl prints it uncounted). Fix: declare the `siunitx` l3keys module over the binding's keys, routed as `\sisetup`. Repro `expl3/l3chk_siunitx_keys_set_l3keys_empty.tex`.
- **chessboard under beamer overlays** (RULED residue, DIFFICULT_CASES §D14, Perl's single-pass overlays kept): since sweep #149 status 2 with 4 errors and no Fatal (two skak `\errmessage mainline`, an `\EatNumberA` runaway, an `\end{frame}` mode error; the document is valid), first `mainline: black, not white, to move (e4)`; in scope. Kept (`accepted_residuals.tsv`: `errors 4`). Repro `expl3/s41msg_chessboard_beamer_only_gamestate.tex`.
- **fourier-orns ornaments print their `futs` slot characters** (SHARED): no `futs` glyph map in `latexml_core/src/common/font.rs`, so the raw preload prints "A 1 B M C U D e". Fix: map `futs` to the Unicode ornaments. Repro `fonts-nfss/fourier_orns_ornaments_are_their_glyphs.tex`.
- **forest-doc's `\DocInput` example: binding gaps** (OUT OF SCOPE, R8b): 3 errors (`\lst@InstallKeywords` SHARED; `\@escapeifif`, `\forest@file@copy` from the forest binding, a native tree since 56iz). Repro `graphics-tikz/forest_docinput_lstenv_optarg_absent.tex`.
- **A raw `\item[…]` label is re-digested for the `typerefnum` tag** (PERL-ORIGIN, latex_constructs.pool.ltxml:1384-1386): `\typerefnum@<counter>` digests `Revert($tag)`, so `\item[\n]` gives `Error:undefined:\n` and `\item[\(]` opens math. 5 and 2 errors (Perl 61 and 43). Fix: build the typerefnum from the formatted tag. Repros `index-bib/item_tag_makelabel_typerefnum.tex`, `index-bib/item_tag_math_cascade_downstream.tex`.
- **An `lstlisting` in a `\parbox` argument is dropped silently** (CONTROL; RUST-ONLY): the pre-tokenized argument puts the body on the begin line, which the reader drops; 0 errors and an empty `<listing/>` (pdflatex 7 errors, no PDF). Needs a diagnostic only when the reader's source is a pre-tokenized argument. Repro `kernel-alignment/pcol_lstlisting_parbox_CONTROL.tex`.
- **`\RequirePackage` inside a group raises no error** (OUT OF SCOPE): pdflatex says "Loading a class or package in a group" (latex.ltx:18699-18704) and loses `{multicols}` at the group's end; Rust loses it silently. Fix: the group-level check in `\RequirePackage`/`\usepackage` (`latex_constructs/sect05.rs:180`). Repro `loader/multicols_afterpackage_group.tex`.
- **A product before a big operator stays unparsed** (RUST-ONLY): `a\cdot b\sum_i c_i`, `\frac1n\sum_i x_i\cdot\frac1m\sum_j y_j` — 4 `Warning:unparsed_math`; Perl parses all four. Not root-caused (the wide/narrow big-operator pragma or a missing `term mulop term bigop_application` path); a bare OPFUNCTION as the MulOp's right operand alike (`a\cdot\log\sum_i x_i`, `a\cdot\mathbb{E}\sum_i X_i`; Perl (a·log)·∑…, 57cf review), and `a/b\sum_i x_i`, `a\times b\int f\,dx`. Repro `math-parse/product_before_a_big_operator.tex`.
- **An `\index` phrase is re-tokenized live** (SHARED; with the `&` glossary row above): SanitizedVerbatim (`base_parameter_types.rs:692-716`) re-tokenizes `\index`'s argument with normal catcodes, so `\index{a\def{b}c}` runs `\def` at the mark (1 error). Repro `string-mouth/index_phrase_primitive_retokenize.tex`.
- **An \item inside {framed}** (RUST-ONLY shape; GREEN on errors): the boxed `\item` keeps the outer numbering (`S0.I1.i2`) but sits in a framed `<itemize>` inside item 1's paragraph, not as the list's own second `<item>` (Perl: 2 `Error:malformed:ltx:item`). Ruling needed on the shape: a framed sibling item or the nested framed list. Repro `boxes-groups/mal_item_framed_capture.tex` (colorframed-doc, OUT OF SCOPE).
- **hyperref's PDF information values are recorded unexpanded** (SHARED, Perl hyperref.sty.ltxml:133 `ToString`):
  `pdftitle=\mytitle` gives `<rdf content="\mytitle"/>`; hyperref expands with `\pdfstringdef` (resphilosophica.cls:311
  `pdftitle=\@title`). Digesting the value would duplicate frontmatter notes (`\thanks`); needs a `\pdfstringdef`-like
  expansion. RED `sectioning-frontmatter/hyperref_pdfinfo_expands_macros`. The same for an AASTeX 7 address recorded as
  `schema:email` metadata (`\email{\url{…}}` → `content="\url{…}"`; RED
  `sectioning-frontmatter/aastex7_email_markup_recorded_as_text`, OXIDIZED_DESIGN_DIVERGENCES #479).
- **A `{abstract}` left open at `\end{document}` empties the document** (SHARED; 65c review side finding): the deferred
  abstract body (`\lx@add@frontmatter@until … \lx@end@abstract`) runs to the end, holding the finalize whatsit and the
  rest of the document, and is never placed — `<document/>` with or without a following `\section` (Perl: resources
  only, 1 error); pdflatex prints every word with 1 error. RED `sectioning-frontmatter/unclosed_abstract_at_end_document_keeps_the_document`.
- **`\boldsymbol{… $ and $ …}` leaves the math open** (RUST-ONLY; 2110.01778): amsbsy.sty:28-34 typesets the argument
  in `\hbox{$…$}`, so the inner `$…$` is text; ours closes the outer math at the inner `$` and the math runs to
  `\end{document}`. RED `boxes-groups/boldsymbol_text_dollars_inside_math`.
- **hyperref's `\hyperpage`/`\hyperindexformat` are unbound** (SHARED; 1703.08480): a hyperref `.ind`
  (`\hyperindexformat{\see{X}}{20}`) leaks a `{` group and leaves `{theindex}` open. RED
  `index/hyperref_hyperindexformat_see_in_theindex`.
- **An alignment left open at `\end{document}` drops its last cell's content** (SHARED, low priority; 65c review
  probes r16 `c & d` → `<td/>`, r21 `c&=d` loses `=d`): invalid input (pdflatex prints nothing); no repro.
- **A plain `\\` in a `\multicolumn` paragraph cell is recovered with no Warning** (RUST-ONLY diagnostic; Perl ends
  the outer row, KPE #626): without a `\centering`/`\raggedright` in the cell, `\\` is `\@tabularcr`, whose `\cr`
  lands inside the omitted span's `\vtop` group: pdflatex 101 errors, the first "Missing } inserted", no PDF. Ours
  (65p, OXIDIZED_DESIGN_DIVERGENCES #559) makes it the line break the legal `\centering` case gives, 0 errors and 0
  warnings; it wants one Warning. Telling the cases apart needs `\centering`/`\raggedright`/`\raggedleft` to
  `\let\\\@centercr` as latex.ltx does (Perl's do not either), and the Warning must also come from the
  `\tabularnewline` let: a `>{\raggedright\arraybackslash}` prefix makes `\\` the same `\@tabularcr` error.
  RED `kernel-alignment/multicolumn_pcol_plain_newline_unwarned`.
- **A `>{…}` prefix macro of a paragraph column takes the wrong arguments** (SHARED; synthetic, 65p review probe q6):
  `\newcommand\two[2]{[#1|#2]}` with `>{\two}p{3cm}` reads `[|x]y`, pdflatex `[|]xy`: array.sty:106-110
  `\insert@pcolumn` puts `\relax` before `\ignorespaces` for `p`/`m`/`b`, `Template::add_column` appends only the
  `\insert@column` `\ignorespaces` (array.sty:97-101); and a `\multicolumn{2}{>{\two}l}{gh}` reads `[|g]h`, pdflatex
  `[|gh]` (array.sty:267-278 reads the body as one `\@sharp`). RED `kernel-alignment/pcol_prefix_argument_takes_relax`.
- **A formula holding only a space ends the paragraph's `--`/`---` ligatures** (RUST-ONLY; 67w side finding, Perl
  0.8.8 and pdflatex ligature every line): after `500$\,$hPa` (the formula written as a space) the rest of the
  paragraph keeps `1989--2008`; `$x$` does not do it. WileyNJD-v2's `\uns` (`\ifmmode\,\else$\,$\fi`, 2208.08472) is
  the witness; the binding's `\uns` uses `\thinspace` until the kernel is fixed. RED
  `fonts-nfss/ligature_after_space_only_formula`.
- **Header re-grades pending**: `luatex-profile/xetexprobe_xevlna_shared` needs `% preload: [xetex]` (then libertinus-otf.sty:215 `\XeTeXtracingfonts` undefined is a new RED); `luatex-profile/babelmodifier_greek_polutoniko` drops its `[luatex]` line (lualatex fails too; pdfTeX 0 errors); `luatex-profile/zugferdtabular_loop` needs the witness's `unit=hour`; `graphics-tikz/calc_scbox_babel_frozen_bang_stale_oracle` becomes CONTROL.

### Fidelity audit round 2 (2026-10-08) — findings and their state

The second audit of "successful" run-336 conversions (80 papers, seed 337, on 63h5; notes
`~/data/pk_agents/main/scratch_356a0c47/fidelity2/notes.txt`, audit.tsv): authors right in 65/69 (round 1: 52/73).
Each class is a silent content or markup loss unless marked otherwise; each landed fix has a repro under
`tools/perfect_kernel/repros/` and a guard in `perfect_kernel_batch63`. Open rows are the worklist (row **A2**).

| # | finding | witnesses | state | next step |
|---|---|---|---|---|
| 1 | `\IEEEmembership` dropped from the author line | 2201.01230, 2306.15457, 2408.09035 | **DONE 63l** (KPE #534) — name-line grades are membership contacts | residual: grades after names misread past a `\\` stay dropped (row 13) |
| 2 | merged creators: `{\ and}`, `\ddag~and` | 1508.01140, nlin0101056 | **DONE 63l** (KPE #532) | — |
| 2b | merged creators: acmart comma list in one `\author` (`Error:frontmatter:merged_creators`) | 2212.06542 | **OPEN** | acmart `\author` binding: split a comma-separated name list (PDF prints them as one block) |
| 3 | eqnarray/align row holding only `\label` dropped with its number | 1011.4399, 1305.3072, hep-th9412215, hep-ph0208046 | **DONE 63j** (KPE #526) | nit: a kept eqnarray row takes a following unnumbered `&=&` row as its continuation |
| 4 | `\ref` (and `\eqref`/`\autoref`/`\cref`/`\nameref`) in a section title empty in the TOC and `title=` tooltips (RUST-ONLY) | 1011.3492, 1111.3672 | **DONE 63k** (fillInTitle port) | residual: a `\cite` in a title reads empty in tooltips (row 12) |
| 5 | IEEE `and~Name` keeps "and" in the last name | 2011.10474, 2408.09035 | **DONE 63j** (KPE #528) | — |
| 6 | `\fnmsep` leaves a stray comma in the name | astro-ph0001054, astro-ph0611016 | **DONE 63l** (KPE #533) | — |
| 7 | PoS/JHEP `\author` calls accumulate, only the last kept | astro-ph0611258 | **DONE 63k** (KPE #529) | — |
| 8 | algorithmic `\REQUIRE`/`\ENSURE` labels replaced by "0:" | 2201.01230 | **DONE 63j** (KPE #527; algpseudocode too) | residuals: algpseudocodex drops the label (RED `list-structure/algpseudocodex_require_shows_its_label`); `\ref` to a labelled line prints the label |
| 9 | raw class `\abstract{…}` (an.cls) a plain paragraph, no `ltx:abstract` | astro-ph0611016 | **OPEN** | an.cls binding or a raw-class `\abstract{}` command form → the abstract frontmatter |
| 10 | entics.cls (raw) `\thanksref{ALL}` prints its key in the name | 2212.08177 | **OPEN** | entics `\thanks[key]`/`\thanksref` → the frontmatter annotation API (elsart-style) |
| 11 | sectsty `\subsectionfont{…\textit}` applies the font to the first letter only (presentation) | 1508.01140 | **OPEN**, low | sectsty font hooks take the whole title |
| 12 | a `\cite` in a section title reads empty in tooltips (63k residual) | — | **OPEN**, low | tooltip text of a nested bibref without double-counting missing-citation warnings |
| 13 | names continued past a `\\` in an author line read as an affiliation line | 2408.01902, 2408.02464, 2308.07107 | **DONE 63m** | IEEE grades read past a continued `\\` (`names_continue`), compsoc `\IEEEcompsocitemizethanks` a note of the names line, a line of two or more names continues the names; guards `ieee_compsoc_names_continue_past_a_break`, `ieee_grade_comma_before_a_break_continues_names`, `author_names_line_and_address_line` |
| 14 | an affiliation line under a group of names goes to the group's last name only | 0911.0568 (LPT Orsay group), JHEP/PoS group blocks | **DONE 63m** | `\lx@add@affiliation[annotate=N]` for the group's N names (OD #459); a bare address line gives each address to the name it spells; guards `author_group_affiliation_goes_to_each_name`, `author_names_line_and_address_line` |
| 15 | `$(a)$Department …` marked affiliation lines not linked (0 affiliations) | 1011.0516 | **OPEN** | read a `$(a)$` lead as the line's mark |
| 16 | llncs `\author{A\\Univ\\\email{x}}` reads "Univ" as a person | review probe (rv63k/llncs1) | **OPEN**, check | confirm llncs semantics (`\\` there separates names) before changing |

### Run-336 open residuals (63a, 2026-10-07)

Mined from run 336's first ~35k papers (post-fix reconversions on the fleet worker; the env-gap papers are re-queued at
completion, row G). **63a** (landed): the `Fatal:Stomach:Recursion` cluster's fixable roots — arxbj `{longlist}` as the
class's own `\list` (KPE #517), etex's register-block allocators (KPE #516), the autoref tag's `~` bound to the kernel
space where it is digested (1601.02132), and arxbj's IMS markup through a binding shared with arximspdf
(`ims_support`); guards `perfect_kernel_batch63::*`. Open:
- **65c** (landed; RUST-ONLY): the content of environments left open at `\end{document}` (and inside a runaway
  `\abstract{…` ended by a section) is kept: every body digesting inside an abandoned group ends as at end of input
  with Perl's dummy trailer (stomach.rs `abandon_stack_frame`; Stomach.pm:111,130), where the trailer split used to
  take each enclosing body's last real box (the inner environment, or the rest of the document). The `ltx:document`
  close is deferred to the end of the build so every still-open element's close hooks run (`close_abandoned_document`,
  OD #207); the warning prints Perl's detail lines (initiator + locator) and closes and reports open conditionals; the
  end-of-document internals revert to nothing in `tex=`; `cleanup_math`'s real-math test is a linear walk
  (`has_real_math`; 1907.09124 265 s → 5 s). Witnesses powerdotDE (recall 26.3 → 97.8%), csvsimple-legacy (88.9 →
  99.6%), 1907.09124, 1902.00853. Guards `perfect_kernel_batch65c::*`. The regained content of arXiv papers whose
  environment or math an upstream defect leaves open surfaced as `malformed:*` placement errors; since 65i the open
  construct gives way to the structure after it (OD #542: 1507.01637 27 → 25; hep-lat9811036 stays `TooManyErrors` on
  its own alignment errors; 2110.01778's cause is RED `boxes-groups/boldsymbol_text_dollars_inside_math`).
- **64zf** (landed; diagnostics honesty, stage 1 — user directive 2026-10-11): Error/Fatal emitted at Perl's
  severity where Perl reports, and where only TeX errs a Warning when the recovery yields good markup, else an Error.
  `.bib` undefined macros at the entry's line with the `.bst`'s `MACRO`s (OD #518), overflow clamps (OD #519),
  setspace switches as checked no-ops (OD #520), listings begin-line text (OD #521), an unknown colour model, the
  open conditional at `\end{document}` (65c's report, guarded here), Perl's `Info:misdefined:utf-8`, picture `\line`
  outside a picture, booktabs widths, `\IEEEQEDhereeqn` in a row, the CLI's unusable inputs as a Fatal with a log;
  addenda OD #207/#316/#317/#481/#489; KPE #600-#602. Guards `perfect_kernel_batch64zf::*`,
  `119_final_status_report::unusable_archive_is_a_fatal_conversion`. Open REDs: sectioning-frontmatter/
  section_title_errors_reported_twice (task: the toc digestion re-raises the heading's diagnostics, shared with
  Perl), loader/plain_input_defining_documentstyle_routes_plain (hep-lat9301012), babel-lang/
  shorthandoff_quote_in_english_is_no_shorthand, expansion-primitives/uppercase_brace_hunt_file_end,
  alignment/tabularcalc_pcell_ifnum_cr, and the setspace REDs boxes-groups/{lrbox_doublespace_group_end,
  abstract_begingroup_doublespace_group_end, plain_group_around_itemize_doublespace,
  doublespace_end_in_a_nested_plain_group}. Stage 2 (the remaining Error/Fatal classes) is open.
- **65d** (C5 step G2; OD #530, KPE #607): `\left`/`\middle`/`\right` are TeX's unexpandable `left_right` command —
  `\left` the constructor Perl has, `\middle` a primitive dispatching to `\lx@delim@middle` — with tex.web §1160
  `scan_delimiter` (`tex_math::scan_delimiter`: `get_x_token` past blanks and `\relax`, a delimiter by meaning — a
  `\delcode` character, `\delimiter`/`\Udelimiter` or a `\let` copy with its code scanned there, a math symbol — else
  the Warning "Missing delimiter (. inserted)"), the `\left` group's K9 code checked by `\middle`/`\right` ("Extra
  \right." Error at the formula's group with the delimiter kept, user ruling 2026-10-12; §1064 `off_save` Warning
  against a document `{`/`\begingroup`), group ends inside it reported as Warnings ("Extra }, or forgotten \right.";
  "Missing \right. inserted" at a formula's or an argument's end, which now closes the formula instead of running on to
  the next `$`), a delimiter outside the map made stretchy as Perl does; rounds 3-4 (reviews r1, r3): the closers
  `off_save` inserts are owed by the group it closed and the debt follows the stack at every pop, so the author's
  `}`/`\endgroup` is deleted (§1069) where it meets a group it cannot close instead of setting the rest of the paragraph
  as math; `\endgroup`, `$` and a DefEnvironment's own end inside an open `\left` get `\right.` first (§1065 at the
  mismatching primitive); a brace where a delimiter belongs takes TeX's path ("Missing delimiter", the brace read
  again); a parameterless command is a delimiter only when it typesets a symbol, its trial digestion silent. Rounds 5-6:
  argument groups carry a group code, and the `}` a delimiter scan puts back is deleted at the formula's frame (Error,
  as TeX and Perl; REDs stray_brace_at_the_formula_level_is_deleted and
  right_in_an_argument_brace_group_after_the_inserted_brace green); every other stray `}` keeps 65a's recovery until row
  MS (§1046-1047). Open: RED boxes-groups/right_inside_an_argument_drops_the_argument. Closes WISDOM #41's `\left` row
  without an `undigested` parameter. braket's `\Set` no longer splits at a `\left|` (KPE #607, 2107.02628). amsmath's
  `\env@cases` defined (amsmath.sty:1121-1126; zhlineskip's `{originalcases}`, whose unmatched `\right.` otherwise
  cascades under the group check). A/B vs 65a (`scratch_65d/ab*`): 109 run-336 witnesses (48 X1 IfLimit + 60
  `unexpected:\right` + 2605.21750): Fatals 48 → 0, errors 3541 → 826, papers with errors 106 → 21; 200 `\left`-rich
  papers and 150 random: XML identical (194 + 141 same, 10 equivalent or ids-only, 1 timing-only), diagnostics unchanged
  (one random paper −1 ambiguity warning). 1710.02165, whose unclosed `\left` swallowed the rest of the paper, gains 4
  sections, 9 figures and its bibliography. Witnesses parens.tex:43 and 1207.4709 byte-identical; golden
  `tests/math/sampler` `\left\relax(` now stretchy, as Perl. Guards `perfect_kernel_batch65d::*`, golden
  `tests/parse/left_right_delimiters`. Residuals (OD #530): the `\right` delimiter digests after the group's
  `\aftergroup` tokens (= Perl); a stray `}` in math inserts `$` instead of TeX's "Extra }, or forgotten $", and a `$`
  meeting a document `{` defers the math end (pre-existing).
- **65b** (landed; C5 step G4):
  - **Column ends by command code** (`is_column_end`; the `\halign` preamble incl. `\tabskip`; OD #522, KPE #603).
  - **Frozen `\span`** for `\multicolumn`/`\multispan`/`\sp@n` (user ruling 2026-10-11, one Warning per document;
    OD #523).
  - **Frozen `\cr`** at a preamble's file end (OD #524).
  - **Misplaced `\noalign`** reported per site, its body typeset as an ordinary group with rule markers inert (OD #525,
    KPE #191 extended).
  - **aa `{longtab}`** wrapper (KPE #604, 1611.08428) and **aastex `\tablewidth{0pt}`** expandable (KPE #605,
    1805.09353).

  W1 (179 `\span`-redefining papers, run 336): errors 1730 → 839, papers with errors 138 → 57. W2: 280 → 218. Manual
  net 797: 0 XML changes. Guards `perfect_kernel_batch65::*`.

  Open:
  - ~~RED `class-bindings/aalongtable_lt_array`~~: fixed in 65h (OD #541, KPE #612).
  - Math readings of ambiguous formulas can flip between builds with no input change (2205.06656, 2503.00718,
    1708.09355). Each is deterministic per binary; likely an interning-order tie-break in the math parser.
- **65e** (RUST-ONLY regressions of run 338 against run 336; triage 2026-10-12): l-aa's plain `\matrix{…\cr}` (64u:
  aa's amsmath gave the environment-only form; aa's `\@currenvir` dispatch, KPE #608); crckapb's `\keywords` ends at
  the abstract's `\end` (64zd: read to EOF with the class `\abstract` refused); a `\par`/`\vskip` in an item's inline
  sectional block closes no paragraph (64d; OD #532); collcell takes the template's `\ignorespaces` as its delimiter
  and drops the trim (63v); the OmniBus theorem set declares only free names, each stub firing once (64y-amplified;
  OD #533, KPE #609; siamltex's theorems declared as its class does); a refused setter's replay that errs is rolled
  back by an assignment journal (64k; OD #534); ptephy's `$^{n}$` affiliation list goes to the names its marks label (64qa:
  2203.01270 83 MB/6.9 GB/357 s → 1.8 MB/3 s; its legend lines after the names a document pubnote each — only when every line after the first `\\` is led
  by a footnote symbol, else the whole list stays names: `ptephy_name_lines_after_break_stay_names`,
  `ptephy_textsuperscript_legend_line`);
  hyperref loads letltxmacro (OD #535, ruling Q5). Shared (`show_unanswered_marks`, base_utilities.rs): an author's
  superscript symbol mark nothing answers is shown, as the PDF prints it, a math `*` as `∗` and a text one
  (`\textsuperscript{1,*}`) as `*` (unless it placed one of that author's contacts that drops it; a mark the author's
  email shows as well stays on the name too, 1910.00401); residual: a `$^{*}$` the kernel shows itself (keepsup, an
  email line) keeps ASCII `*` where pdflatex prints `∗` (1910.00401 email, probe hb); four older guards flipped to the PDF's reading (batch61 `author_thanks_and_footnote_keep_their_math`,
  `marked_affiliation_wrappers_and_legends`; batch63 `author_mark_comma_in_math`; batch64
  `marked_block_email_line_owner`). Run-338 Fatal →
  witnesses: astro-ph9609122, astro-ph9809009, astro-ph9503122, astro-ph0002500, astro-ph9901229, astro-ph9610167,
  astro-ph0003044, astro-ph9702104, astro-ph0003143, astro-ph0003144, 1806.03273, 1912.00582, 2604.02352, 1308.6566,
  0811.3048, cs0511055, 2106.06828, 1911.11856 → 0 errors; 1805.06896 2, 2304.12197 3, 2307.13461 47, hep-ex0406049 30,
  0705.0894 2, 2511.04367 7, 1811.04820 1, 2607.24171 3 (all Error, below or at 64c/64h). Guards
  `perfect_kernel_batch65e::*`. Open: `ltx:appendix isn't allowed in ltx:inline-sectional-block` after an item's
  bibliography (5 of the C witnesses; `list-structure/bare_itemize_end_leaves_appendix_in_block`, witness
  1805.06896: a bare `\itemize`'s `\end{itemize}` leaves the outer list open — GREEN in 65i, the open list giving way
  to the section); the set's amsthm load replaces a
  document's own `\newenvironment{proof}` (shared with Perl); a clean replay of a document setter still empties the
  stores its body `\gdef`s (RED `sectioning-frontmatter/replayed_document_setter_keeps_its_store`: "Written by ."
  where the PDF has "Written by Alice."; the #482 store reset keeps class title-page copies out of the frontmatter,
  and telling a body-read store from a title-page one needs the class's `\maketitle` reads, not the journal).
  Not in scope (lost identically in 65a): a composite mark `$^{2*}$` or `$^{1\dagger}$` (2307.15337, 2604.24425) and
  cas `\fnmark[$\dagger$]` (2511.01302, 2607.29394) lose their symbol. Also composite: under OSA a `$^{2\#}$` mark shows only its
  symbol half on the name (c2h), and a `$^{2\%}$` address mark places its email on the block, not its author (both
  as in 65a).
- **65a** (landed; RUST-ONLY): sweep #149's regressions against #148. A `\title`/`\author` the 64d seal turns away
  runs the raw class's own definition in place when it provably keeps no value (fail-safe store classifier); an
  `\author` the class hands only to page furniture becomes the author of its sectional unit (ruling 2026-10-11); heading
  names (`\contentsname`, `\refname`, `\indexname`, `\acknowledgmentsname` …) are digested in their own group, so an
  unbraced font switch no longer leaks into the document; insert_block keeps Perl's container order, with a guarded
  closing-caption hoist out of a rotate/scale box; ASCE grades (`M.ASCE`) qualify the name. OD #526-#529. Witnesses
  recipebook/LongSample (recipe titles and attributions back), heria/heria-proposal (schema-valid), ascelike/ascexmpl,
  1810.08269. Guards `perfect_kernel_batch65a::*`. Residuals: a late class `\author` that both stores and prints is
  dropped (= base); the figure-panel wrap for refused caption hoists (task #193; RED
  `captions-floats/subcaption_labels_in_scaled_panels_stay_on_their_panels`).
- **65h** (RUST-ONLY; run 340's Error→Fatal regressions, 287 papers against run 338):
  - **`\LT@array`**: the longtable binding is split as longtable.sty splits it (`\longtable` = `\begingroup\LT@array`,
    `\endlongtable` closes the group), so aalongtable.sty's and anlongtable.sty's `\longtable` build their tables
    (81 papers; OD #541, KPE #612).
  - **pasj00/pasj01/pasj02 without the shipped class** keep the class's `{longtable}` through the longtable binding,
    with the class's `\noalign` `\caption`/`\label` that gobble a following `\\` (pasj00.cls:1245-1270), so the head
    row after `\caption{…}\label{…}` is kept (astro-ph0407419, astro-ph0701701; OD #541).
  - **longtable caption row** (OD #103): a caption's row with an `&` after the caption is kept, not dropped as the
    caption's alone (`\caption{…}\label{…}` with no `\\` before the head row lost it).
  - **Misplaced-alignment cascade**: the first misplaced `\noalign`/`\omit`/`\span` in a group is the Error, the rest
    of that group Warnings; at top level the group is the document body (OD #540). This is open for the main session:
    it supersedes 65b's per-site Errors.
  - The 107 longtable papers: 107 Fatal → 0 Fatal, 2147 errors (64ze) → 350, 73 papers at 0 errors, none worse than
    64ze. Of the other 121 non-infra papers, 56 are no longer Fatal. Guards `perfect_kernel_batch65h::*`.

  Open, each a RED repro and none fixed here:
  - **rccol + natbib** (64ze): `Fatal:Timeout:PushbackLimit` in all 18 PushbackLimit papers.
    - Witnesses 1101.1265, 0812.2309, 1104.5326, 1106.3121, 1202.3790, 1212.1181, 1304.2825, 1307.3777, 1309.0372,
      1404.0550, 1501.04632, 1509.07583, 1610.05509, 1610.09398, 1801.07259, 1810.01491, 2207.03232, 2609.37147.
    - 64x converts them.
    - RED `alignment-bindings/rccol_with_natbib_pushback_loop`.
  - **Environments open at `\end{document}`** (65c): their kept content holds the bibliography or sections, and every
    `malformed:*` placement is an Error. Of the 21 sampled TooManyErrors papers, 19 reached Fatal at 65c.
    - Witnesses 1611.05714 (64ze: no bibliography, 1 error), 1507.01637, 2412.20775, 1806.06901, 1101.1959,
      2007.11971, 1210.0368, 1003.1368, physics0505113.
    - Near-threshold papers (64ze 96-99 errors): 1211.3050, 2210.11107, 1305.0086, astro-ph0303604, math9805003.
    - cs0009011 ends in `Fatal:Document:Generic("add_child`.
    - RED `list-structure/open_description_holds_the_bibliography`.
  - **standalone sub-file preamble**: xcolor's tabular hooks are installed without their definitions, so every later
    tabular is no alignment.
    - Witness 2512.09430: 64ze 84 errors, 65b+ Fatal, one misplaced `\toprule` per table.
    - RED `loader/standalone_subfile_xcolor_tabular_hooks`.
- **65i** (RUST-ONLY + one recorded surpass; run-340 Error→Fatal regressions):
  - **Constructs the document ended inside of** (65c's cluster: an upstream defect leaves a list, formula, script,
    alignment or `\fbox` open, and since 65c everything after it arrives inside it): such a construct is
    `lx@unended` and its elements auto-close (Perl's per-node `_autoclose`), so the next section, theorem or
    bibliography closes it instead of being held as `malformed:*` (OD #542; Perl errs as 65c did). Run 340's 228
    error→fatal papers (65bundle → 65i): 41 Fatal → OK (all 18 PushbackLimit, cs0009011's add_child, 2512.09430, 21
    TooManyErrors), none worse; no text word outside math lost in the 42 changed papers whose base XML parses
    (2512.09430 loses only the names in its `<ERROR>`s). Cluster vs 64ze: 1611.05714 1 → 1,
    1210.0368 30 → 30, physics0505113 75 → 75, cs0009011 13 → 13, 1507.01637 25 → 25, 1806.06901 52 → 52; a few
    errors above 64ze where 64ze dropped the rest of the paper: 2007.11971 9 → 13, nucl-th0401019 89 → 91, 1204.0071
    88 → 90, math9805003 96 → 97, 1101.1959 55 → 56. **A formula open at the end is the `$` deferral, not the end**:
    tex_math.rs `end_math_or_defer` defers the math end of `$S^{x$` to the script group's end, where TeX inserts the
    `}` (RED paragraph-mode/dollar_closes_open_script_group, row MS); 65i only closes such a formula at the next
    structure, so the text after it is kept but run together (2007.11971: 2,453 script-font words, 7,157 run-together
    pseudo-words by the review's count; 1210.0368 908 runs of 20+ letters; 1907.09124 148; math9805003 25) — no word
    gain is claimed for these. Fidelity given up: an unended `center` or `minipage` no longer marks the section that
    closes it (`ltx_centering`/`ltx_minipage`, OD #542). 65e's open item, a bare `\itemize` leaving the
    outer list open (RED → GREEN `list-structure/bare_itemize_end_leaves_appendix_in_block`): 1805.06896 2 → 0,
    2304.12197 3 → 0, 2511.04367 7 → 0, 2307.13461 47 → 42, hep-ex0406049 30 → 29, 0705.0894 2 → 2. Still Fatal (64ze 99, 97 errors), each the honest
    errors of regained content, their upstream defects not reduced: 1211.3050 (`align`s left open,
    Configurations.tex:288, ObtainingConf.tex:1207), 2210.11107 (4 stray `&` of a tabular left open,
    main_arxiv.tex:2979, after a run of `\acro{\smaller …}` errors in math; `\smaller` is undefined in pdflatex too).
  - **rccol `R{}{}` with natbib** (`Fatal:Timeout:PushbackLimit` since 64ze, 18 papers): the template reader empties
    `\@temptokena` before a column-type rewrite as array.sty:361 `\NC@do` does (natbib's `\appdef` left it filled), and
    an rccol binding gives `R` the result of its `\edef` rewrite, the raw cell code formatting the number as pdflatex
    does (KPE #615; 1101.1265, 0812.2309, 2609.37147: 64x 1 error → 0). **Known gap** (the kernel guard's fixture,
    kernel-alignment/column_rewrite_reads_temptokena_empty, converts with 3 Warnings, pinned): the template reader
    expands a column type and does not run it, so a rewrite written for array.sty's search — assignments to
    `\@temptokena` ending in `\NC@find`, array.sty's own `*` shape (array.sty:372-379) — leaves
    `Unrecognized tabular template` for `\@temptokena`, `\expandafter` and `\NC@find`; no pdflatex-valid rewrite that
    reads `\@temptokena` avoids them (array.sty's `\@mkpream` rejects an expandable `\the` in a `\newcolumntype` body,
    "Illegal pream-token"). Closing it means running a rewrite's assignments in the reader.
  - **Standalone sub-file preamble**: `AddToMacro` in a package loaded inside the bracket is local, so xcolor's
    tabular hooks end with its definitions (OD #543, KPE #614; 2512.09430 84 → 5 errors, 6,635 → 14,455 words).
  - Guards `perfect_kernel_batch65i::*` (65e's `par_in_item_bibliography_keeps_the_block` now on a list that ends,
    repro list-structure/bib_blank_lines_in_closed_list); repros list-structure/open_description_holds_the_bibliography,
    list-structure/open_math_script_holds_the_section, boxes-groups/unended_fbox_holding_figures,
    kernel-alignment/column_rewrite_reads_temptokena_empty, alignment-bindings/rccol_with_natbib_pushback_loop,
    loader/standalone_subfile_xcolor_tabular_hooks (all GREEN); RED paragraph-mode/dollar_closes_open_script_group
    (row MS).
- **65m** (recorded surpass + Perl-origin fixes; run 340's first-error cluster `Script _ can only appear in math mode`,
  6,357 papers, root-cause table `~/data/pk_agents/main/run340/clusters/underscore_rootcauses.md`): text TeX only
  stores, writes to an auxiliary file, or gobbles is not digested as typeset content.
  - **Written copies** (A1+A2, ≈2,750 papers): natbib's `\bibitem` label (natbib.sty:822, .aux only) and a caption's
    or heading's `[short]` form (.lof/.toc) are digested with `tex_math::WRITTEN_COPY` set (`\lx@written@copy` in
    `\lx@toc@copy@neutralize`; natbib's `\NAT@@wrout`), where a text-mode `_`/`^` is its character without the Error
    (OD #550, KPE #620).
  - **Names and gobbled arguments** (A3-A6, B1-B3): imsart's `\bdoi` gobbles (imsart.sty:2583); feynmp's `{fmffile}`,
    cas's `\bio`, aastex's `{interactive}` read their file arguments Semiverbatim; achemso's `\email` is the author's
    `ltx:contact`; a raw class's rerouted `\doi` store is a detokenized string; `\the<ctr>@ID` splices the counter's
    name as characters (OD #551, KPE #621). **A7**: cleveref's `\label[type]{key}` after `\caption`, a procedure's caption (algorithm2e) or
    `\captionof` (OD #552).
  - Witnesses (base 138cf5b167 → 65m, Errors): 23 of the table's 26 → 0; 2102.09024 3 → 1 (`\captionof`),
    1511.07613 3 → 2 (imsart `\setattribute`, `\endbibitem`), 2303.02355 28 → 7. 25 more found by grep (natbib
    labels, `\caption[…_…]`, `\newcounter{…_…}`): 17 with errors, all → 0. 60 random natbib/`\caption[` papers:
    byte-identical XML. Visible changes: imsart entries lose their DOI (as the PDF), 2404.11739's floats get their
    labels (`labels="LABEL:["` before), 1803.03591's e-mails are the authors' contacts.
  - **Open**: aastex's `\tablenotetext{^a}` digests its mark as text (deluxetable_sty.rs:128; aastex631.cls:5382 sets
    it `$^{…$#1$}$`: 2303.02355's 6 left); a `.bbl` call of a raw class's `\doi` adds a second DOI pubnote
    (1710.01977); cleveref's `[type]` is dropped, so `\cref` says "table" where the PDF says "Appendix Table"
    (2404.11739). Guards `perfect_kernel_batch65m::*`; repros index-bib/natbib_bibitem_label_is_aux_only,
    index-bib/natbib_authoryear_label_is_aux_only, captions-floats/caption_short_form_is_lof_only,
    captions-floats/cleveref_label_type_after_caption, captions-floats/cleveref_label_type_after_procedure_caption,
    captions-floats/cleveref_label_type_after_captionof, macro-state/counter_name_with_underscore,
    class-bindings/imsart_bdoi_is_gobbled, class-bindings/cas_bio_photo_file_is_not_typeset,
    class-bindings/achemso_email_is_an_address, class-bindings/aastex_interactive_arguments_are_not_typeset,
    graphics-tikz/feynmp_fmffile_name_is_not_typeset, sectioning-frontmatter/raw_class_doi_store_is_a_string (all
    GREEN).
- **65f** (RUST-ONLY + one recorded surpass; run-338 Error→Fatal kernel/mode clusters):
  - **Plain TeX documents** (`state::plain_tex_document`): the on-undefined LaTeX autoload no longer fires there, so a
    LaTeX kernel command is undefined as in TeX and Perl (1504.02513, 1402.1807, 1310.8121, 2405.04530 `\verbatim`
    Fatal → 0 E; math9810192 101 E Fatal → 55 E, its `gtmacros.tex` is missing; 1901.09683 Fatal → 1 E). Perl's
    TeX.pool triggers are held undefined there, so generic probes see plain TeX, and a use still loads the format
    (OD #536, KPE #610). graphics' epstopdf hook is guarded as pdftex.def:681-700 (1710.04722, 1802.06281 Fatal → 0 E).
  - **PDF/DVI by included figures** (OD #537): 1012.0815 Fatal → 3 E (= 64g), 1002.1751 → 0 E. DVI when every
    included figure resolves PostScript-only, epstopdf conversions or load notwithstanding, or pstricks is loaded;
    oracle = the Producer of arXiv's PDF ("dvips + GPL Ghostscript"): 1806.00093, 1806.00188, 1806.00284, 1806.00370
    flip from PDF (65a) to DVI; the epstopdf-loading EPS-only 1804.02991, 2104.01314, 1209.6096, 2210.10273, 1701.08490
    stay DVI, as 65a had them.
  - **Cycle guard**: a Mouth token's fingerprint carries its source position (OD #538; 1009.1698 Fatal → 1 E, = 64u).
    A sixteenth open file level is pdftex's "TeX capacity exceeded, sorry [text input levels=15]" Fatal (a
    self-`\input` macro ran to the watchdog's `Fatal:timeout:wallclock` at `--timeout`, ~0.75 MB of log a second; now
    0.2 s). The gullet checks the cooperative deadline too (`Fatal:Timeout:Convert`, which mattered only where no
    watchdog is armed: the test harness, `latexml::api`); open: under a watchdog sharing the deadline the two race, and
    a run may end in the cooperative Fatal with its XML or in the watchdog's with none (OD #538; grace: task #205).
  - **`\endinput`** through a macro on a line ending in `%` no longer reads the next line (comments off; atbegshi in
    plain TeX).
  - A/B 65a-rel → 65f (681 papers: 150 plain, 125 mode-flip, 66 pstricks, 98 EPS, 99 PDF-figure, 143 random): 639
    identical; plain 2078 → 1829 errors. The 22 PostScript files submitted as `.tex` change both ways; among real TeX
    sources 4 better (hep-lat9207024 Fatal → 26 E, 8k → 42k words), 4 worse by 1-8 `Error:undefined` (plain papers
    whose missing macro files LaTeX used to stand in for: hep-ph9209216 mtexsis, hep-th9109002 jnl). Guards
    `perfect_kernel_batch65f::*`, `preclass_kernel_autoload::plain_document_leaves_kernel_cs_undefined`,
    `converter::tests::figure_output_cue_of_sources`.
  - Open: math0305287, math0111222 stay Fatal: TL 2025 pstricks under plain TeX loads expl3 (pst-code-pspicture.tex:76
    `\ExplSyntaxOn`) and LaTeX with it, plus Perl's braced-`\input` LaTeX load (KPE #611). J (1911.11856, 2106.06828)
    is TeX-faithful once `\LetLtxMacro` is defined: both 0 E with letltxmacro loaded (65e's hyperref change).
- **65l** (RUST-ONLY binding gap; OD #545-546): the MnSymbol binding declares every name of MnSymbol.sty — the 1,307
  `\Decl@Mn@Char`/`\Decl@Mn@Alias` names, the 55 `\Decl@Mn@Op`/`\Decl@Mn@Int` operators, the `symbols` font's
  `\t…`/`\d…` operators, delimiters (`\lsem`, `\langlebar`, … fence after `\left`), brace pieces, accents and the
  package's own declarators — with unicode-math's code point, else TeX4ht's for the font slot, roles as the sibling
  symbol bindings; a name already defined keeps its definition, a slot-and-class sibling of one is `\let` to it
  (`\nin` → `\notin`). The stub's `Warning:missing_file` is now an `Info` (13,470 run-340 papers carried it: a status
  confound, OD #546). Run 340's first errors `\nin` 175, `\udots` 173, `\sumint` 165, `\cupdot` 130, `\strokedint`
  113, `\lsem` 108. A/B 65bundle-rel → 65l on the 80 witnesses (`run340/stubs/MnSymbol_{err,ok}.txt`): err set 40 → 10
  papers with errors, 78 → 39 errors, the 10 left non-MnSymbol; ok set 0 → 0 errors, words and Math identical; XML
  changes are `<ERROR>` → symbol, `\checkmark` amssymb's again (the stub overrode it), `\lcurvearrowright` ↶ → ↷ (the
  stub had MnSymbol's `l` family reversed). Round 2 (review): TeX4ht's `&gt;` decoded (`\gtr`), the footlines and
  `\upmodels`/`\downmodels` (2604.25507), `\ocirc`, `\filledlargestar` as MnSymbol.pdf draws them, a name a document
  declares with `\Decl@Mn@Char`/`\Decl@Mn@Open` takes its slot's glyph from the tables (else an Info), `\slimits@`
  left to amsmath. Guards `perfect_kernel_batch65l::*`, `mnsymbol_sty::tests::*`; repros `class-bindings/mnsymbol_*`.
- **65j** (regression of the 65 bundle; OD #544, KPE #616): `\left`/`\middle`/`\right` in a `\bm` argument stand at
  `\bm`'s level, as bm.sty's token scan emits them (bm.sty:162-217), their delimiter bold (`\bm@delimiter`, :416).
  `\bm` is a protected macro: without a top-level delimiter command the argument is the bounded `\lx@bm@bold` (Perl's
  constructor, byte-identical: 30 of 30 bm-loading control papers); with one, the runs between the commands are bolded
  apart and `\lx@bm@delimiter` bolds the delimiter's element; a run's leading script stays on the fence (2107.08152,
  1703.10645). Witness 2204.13665 (`\bm{\left\langle} #1 \bm{\right\rangle}`): 65a 0 E/521 W, bundle 587 E, 65j 0 E/227 W,
  words 200,003 → 202,457. Exposure: `\bm|\boldsymbol|\hm{\left|\right|\middle` in 81 of 400k sampled sources (≈600
  corpus-wide), 38 with bm loaded (≈280). A/B of 30 of them (65a / bundle / 65j): bundle errors on 9 (1-587), 65j
  0 E everywhere (2312.16075's 1 E is 65a's), no paper worse than 65a in errors, warnings or words. Guards
  `perfect_kernel_batch65j::*`; repro `boxes-groups/bm_left_right_at_the_outer_level`. RED side finding
  `boxes-groups/formula_end_after_extra_brace_in_left_group` (`${\left( } x$` runs to `\end{document}` on 65a and
  the 65 bundle alike; row MS's `}`-meets-`\left`-group class).
  Round 2: the commands that box or group their argument — amsbsy's `\boldsymbol`/`\pmb`, the `\bm` of RevTeX
  (Perl's bold `\bm`; RevTeX itself asks for bm.sty), the World Scientific classes and interact.cls — share the
  mechanism (bm_sty.rs `bold_math`, `BoldArgument::Grouped`): paired `\left…\right` in one argument stays the
  bounded constructor (byte-identical), an unpaired one (pdflatex: "Missing \right. inserted", ⟩ lost) is set at the
  outer level with one Warning per command instead of the bundle's "Attempt to close a group that switched to mode
  math". Repros `boxes-groups/{bm_revtex,bm_ws_class,bm_interact,boldsymbol}_unpaired_left_right`.
  Round 3: a paired `\left…\right` in a bm.sty `\bm` argument stays the bounded `\lx@bm@bold` too, byte-identical
  to the bundle (round 2 split it: `\bm{\left(A+B\right)^\mathsf{T}}` lost its superscript; the 8 A/B papers whose
  `tex` had changed match the bundle again). On the unpaired path a script's argument is bolded whole — a command
  takes its parameters' arguments (`\right\rangle_\mathrm{L}`), as bm.sty expands it (`\bm@expand`); one that
  cannot be told keeps the rest of the run, with a Warning — and a prime is a bold `^{\prime}`. Guards
  `perfect_kernel_batch65j::{bm_paired_left_right_keeps_its_superscript, bm_script_command_argument_after_unpaired_right,
  bm_prime_after_unpaired_right_is_bold, bm_script_argument_that_cannot_be_told_warns}`. RED row **BD**
  (`\DeclareBoldMathCommand` delimiter after `\left`, 1205.4484).
- **65r** (bindings load what their TeX Live files load, round 2; OD #561-564, KPE #627-629): amsrefs loads url,
  rkeyval, textcmds and mathscinet (`\cprime`, `\germ`, `\scr`, `\url`, `\DefineSimpleKey`; `lite` skips the last two),
  `\bibselect` reads its `.ltb` database (cited entries printed, `\bibselect*` all; the author's `.bbl` when no
  database ships), `{biblist}[…]`/`{biblist*}`, `\resetbiblist`, `\BibSpecAlias`, `\bysame`, `\ocite`; a repeat
  `\usepackage[…]{hyperref}` after a class binding's own hyperref load takes its new options through `\hypersetup`
  (`<file>_keyval_setter`), so `colorlinks` loads color (revtex4, elsarticle, …); PoS loads color as PoS.cls does;
  thm-restate loads thmtools (the mutual-require cycle is an Info, as LaTeX's `\ver@` ends it); elsarticle `1p`/`3p`/`5p`
  load geometry (one-column layouts give it their text width); xy's frame feature requires curve (xypdf-fr.tex:60:
  `\usepackage{xypic}` curved arrows, `\crvi`); revtex defines `\sanitize@url`. Run-340 first-error estimates: amsrefs
  ~1,860, colorlinks ~860, thm-restate ~477, xy curve ~380, `\sanitize@url` ~375, elsarticle geometry ~287. Witness A/B
  65int2 → 65r (`~/data/pk_wt/scratch_65r/ab/`, ≥4 Error + 4 OK/Warning papers per item, 61): 32 of 34 Error papers
  lose errors (28 to 0), none gains; no OK/Warning paper changes beyond tmp-path noise, save 1907.13203's placeholder
  `pdftitle`/`pdfauthor` now RDFa. Next first errors: revtex `\@AF@join` (2006.04679, 1702.04290, 1510.07897,
  2508.01050, from JASA.cls/ujp.sty); thmtools `\declaretheorem{theorem}` prints no "Theorem" name (shared with 65int2).
  Guards `perfect_kernel_batch65r::*`, `06_cluster_bibliography::amsrefs_bibselect_{database,star_database}`; repros
  `stub-dependencies/*` (7), `index-bib/amsrefs_bibselect_*`.
- **65k** (stub bindings load what their packages load; OD #547-548, KPE #618-619): mdframed loads zref-abspage,
  needspace and color (mdframed.sty:72-77) and its `framemethod` package (tikz/pst-node), `\mdfsetup` reads the keys;
  backref loads kvoptions/kvsetkeys/rerunfilecheck and defines its language hooks, `\ifbackrefparscan`, `\backref`,
  `\backrefalt`; hyperref's `backref`/`pagebackref` options load backref; mathpartir loads keyval and defines its
  separators; epstopdf loads pdftexcmds (`\ifpdf`); mciteplus its switches, counters and setters; arydshln `{Array}`/`{Tabular}`/`{Longtable}`; nag its rule
  commands. Class bindings: pnas-new (`\dates`, `\dropcap`, the stored sections, `[framemethod=tikz]{mdframed}`,
  jabbrv), achemso (caption/graphicx/url/xkeyval, `\SectionsOn`/`\AbstractOn`, the TOC-graphic lengths). A/B
  65bundle → 65k over the run-340 witness samples (`~/data/pk_wt/scratch_65k/`, 40 Error + 40 OK/Warning papers per
  stub, 560): Error papers 266 → 226 (40 Error → OK/Warning: mciteplus 39 → 14, mdframed 39 → 29, arydshln 37 → 35,
  mathpartir 39 → 37, nag 37 → 36; tabu and backref unchanged); no OK/Warning
  paper has more errors or fewer words or bibitems (1804.04674's figures renumbered by caption's `\caption*`). Loaded
  through hyperref's option (or `\PassOptionsToPackage`), backref's `missing_file` message is an Info, a direct
  `\usepackage{backref}` keeps the Warning (main-session decision; Perl is silent, the lists are generated navigation;
  witness 2203.07162 back to 0 warnings). arydshln witnesses 2502.17812, 2306.04970; nag 1303.5654. Guards
  `perfect_kernel_batch65k::*`; repros `stub-dependencies/*`. Not stub gaps (remaining first errors): wiley-article's
  `\keywords` is defined inside the class's `wiley@abstract` body, which the locked `{abstract}` binding bypasses
  (2009.12523, 2007.02789; landed in 68n, OXIDIZED_DESIGN_DIVERGENCES #634); lmcs's `\titlecomment` under `\documentclass{LMCS}` (upper-case name, no binding: 1103.0510);
  ulem's `\ULforem`; tabu's `|[width]` rule spec (1907.07986).
- **65p** (RUST-ONLY, Perl 0 errors; OD #559, KPE #626): a `\multicolumn` over a paragraph column (`p{}`, `m{}`, `b{}`,
  tabularx `X`) lets `\\` to `\lx@newline` inside its cell box instead of rewriting the body's brace-depth-0 `\\`
  tokens, so a nested `\begin{tabular}…\end{tabular}` keeps its rows (the rewrite made them line breaks: "Extra
  alignment tab", "\noalign cannot be used here" at the next `\hline`, or, for a one-column table, rows lost with no
  diagnostic), and a math alignment in the cell (`pmatrix`, `cases`, `array`, `aligned`) keeps its rows; the `\let`
  precedes the column's `>{…}` tokens (`Cell::prefix_at`); `X`/`J` cells are spliced as `\vtop{` (no more "Missing
  number" Warning and `width="0.0pt"`). Witnesses
  2607.14806 4 → 0 errors, 2010.13561 29 → 0, 1808.05579 18 → 0; 1610.00974 and 1805.01525 byte-identical. Sample of 30
  seeded random papers with a `p`/`m`/`b`/`X` `\multicolumn` (`~/data/pk_wt/scratch_65p/samp.tsv`): 28 byte-identical, 2112.08542
  loses `width="0.0pt"` on its `X` cells, 2410.18046 a temporary path. Guards `perfect_kernel_batch65p::*`; repros
  `kernel-alignment/multicolumn_*`. Open: two RED repros, rows in "Perfect-kernel RED repros without a row elsewhere"
  (a plain `\\` in a multicolumn paragraph cell recovered with no Warning; a `>{…}` prefix's arguments).
- **65n** (task #204; OD #553): TeX's math-group recovery for an inline formula outside alignments — tex_math.rs
  `end_math_or_defer` inserts the closer of a `{`/`\begingroup` group opened in the formula at its `$` (Error "Missing }
  inserted", §1193 + §1064-1065) instead of deferring the math end to that group's end; stomach.rs `insert_dollar_sign`
  ends the formula at a `\par` or a DefEnvironment's end ("Missing $ inserted", §1046-1047); a stray `}` at the formula's
  math shift ends the formula when it closes a construct digested as a unit (a box body, an argument digested on its
  own: `stomach::formula_ends_with_its_unit`; the DECISION `\mbox{$x}` repro GREEN) and is deleted otherwise, one Error "Extra }, or forgotten $" (§1069). Witnesses 65int2 → 65n: 2007.11971 13 → 1 errors, 2,643 → 43 warnings, run-together runs 179 → 0, text words
  outside math 5,268 → 9,865; 1710.02165 29 → 21, its swapped paragraph set right; 2008.11762 14 → 15 (a pre-existing
  `$` desync now ends at its `\par`). Unchanged: 1210.0368, 1907.09124 (deferrals inside alignments, row MS (1)),
  math9805003, hep-lat9811036, nucl-th0401019. 40 random papers (seed 65204) byte-identical. Guards
  `perfect_kernel_batch65n::*`; 65c/65d/65i guards updated to TeX's counts, 56/64y to the "Extra }" message. Round 2:
  the 70-paper A/B (witnesses + random, seed 65204) byte-identical to round 1. REDs left: row MS (1)
  cell_end_in_inline_math_inserts_dollar, (4) brace_meeting_left_group_in_inline_math_is_deleted. Round 3 (main-session
  decision): round 2's "partner outside" rule reverted to the unit-construct rule (it lost a1/a2/a16 and gained nothing);
  new RED (3) forgotten_dollar_in_a_plain_group_keeps_the_text.
- **65o** (SHARED, PERL-ORIGIN; OD #556-557, KPE #624-625, completes KPE #547's `\lefteqn` residual): eqnarray's
  alignment as latex.ltx defines it. A first-column `\lefteqn` is one cell (`\rlap`, latex.ltx:15800) that overprints:
  at normalization it spans the structurally empty cells after it (`Cell::overprint`, `normalize_overprint_spans`; TeX
  does not span, tex.web §791), so `\lefteqn{lhs}&&\nonumber\\` keeps Perl's one cell over the row with 0 errors, a
  cell with content (or an `\index`/anchor) keeps its column, and the row is classed `ltx_eqn_lefteqn` in the XML
  (Perl Base_XMath:956, dead there). eqnarray has the fourth, number column (:15764) a `\cr`-ended row may fill
  (1804.05059), its trailing empty cells dropped (eqnarray only), and latex.ltx's "Too many columns in eqnarray
  environment" where a `\\`-ended row reaches it. Run 340's `&` cluster R1 (about 2,050 papers) + R4 (about 71): the
  32 witnesses (R1 list, R4, 2205.04522, 2411.04274) go from 178 errors on 31 papers to 0, no words lost, warnings
  equal or fewer (nucl-th9512020's `\hat`-in-text and math0407427's `\span` warnings gone); 40 seeded random eqnarray
  papers: 39 byte-identical, cond-mat0601437 2 → 0 errors (a `\lefteqn` row). Guards `perfect_kernel_batch65o::*`;
  repros `alignment/{lefteqn_*,eqnarray_number_column*,eqnarray_empty_cell_keeps_later_column}` GREEN. Open (RED):
  `alignment/eqnarray_cr_row_unnumbered` and `eqnarray_cr_row_column_count` (a raw `\cr` skips `\@@eqncr`: TeX leaves
  the row unnumbered and counts on into the next row's "Too many columns"; we, as Perl, number it and count afresh);
  `alignment/lefteqn_overprint_keeps_template_material` (the `\multicolumn{1}{l}` drops column 1's `@{…}` and `|`,
  which TeX's `\rlap` keeps); `alignment/eqnarray_side_content_only_cell` (KPE #625: a cell holding only an `\index`
  or anchor measures under 1pt and is never absorbed, and an eqnarray row join drops a floated indexmark; rule
  proposed: size never decides dropping content, only `Digested::is_skippable` does).
- **65s** (KPE #630, #631, #632): the AMS bindings provide what the real packages define and documents call:
  amsmath's internals (`\bBigg@` on the scaled-delimiter path, `\big@size`, `\Let@`/`\restore@math@cr`/`\default@tag`,
  `\maketag@@@`, `\@mathmeasure`, `\spread@equation`, `\start@align`, `\intertext@`, `\mdots@`, `\ltx@label`), amsfonts'
  `\newsymbol` (through `\DeclareMathSymbol`; the msbm table is now also the `U`/`msb` font map), and amscd's connector
  read as an undelimited argument (spaces after `@` skipped) plus `\atdef@` and `\CDat`. Run-340 estimates: amsmath
  internals ~1,140 papers, amscd ~838, `\newsymbol` ~300 (run340/clusters/undefined_rootcauses.md rows 3, 5, 7).
  Guards `perfect_kernel_batch65s::*`; repros `ams-internals/*`.
- **65t** (KPE #633): the AASTeX binding defines the commands of the AASTeX 6.x class files that papers shipping
  those classes call: `\reportnum` and `\journalinfo`/`\submitted` as pubnotes, the edit and layout switches
  (`\turnoffedit*`, `\listofchanges`, `\suppressAffiliations`, `\NewPageAfterKeywords`, `\setwatermarkfontsize`), the
  author-limit count, `\tablefontsize`, `\abovedeluxetableskip`, `\fullcollaborationName`, `\lambdabar`, and the journal
  names `\aas`, `\dps`, `\jaavso`, `\maps`. Run-340 estimate ~1,700 papers (undefined_rootcauses.md row 1, aastex
  sub-family). Guard `perfect_kernel_batch65t::aastex6x_class_commands_are_defined`; repro
  `class-bindings/aastex6x_class_commands`.
  The World Scientific bindings (ws_journal_cls.rs, ws_procs_cls.rs; no Perl binding) load the figure packages the ws
  classes input (epsfig, epsf; ws-ijmpa.cls:551-555, whose psfig.sty is not in TeX Live), and define `\pub{received}{revised}` as pubnotes,
  `\cdash`, `{alphlist}`/`{arabiclist}`/`{romanlist}`/`{itemlist}` and ws-rv9x6's `\if@addchapnum`. Run-340 estimate
  ~1,500 papers (`\psfig`/`\epsfig` ~650, `\pub` ~500, `\cdash` ~120, lists ~150, `\if@addchapnum` 113). Guards
  `perfect_kernel_batch65t::{ws_journal_class_commands_are_defined, ws_rv9x6_defines_addchapnum}`; repro
  `class-bindings/ws_journal_class_commands`.
- **65v**: the jfm and mdpi bindings (contrib; no Perl binding) define what the versions of those author-shipped classes
  define and papers call (union of 119 sampled copies each, the class-family rule), and follow the paper's own copy
  where the versions conflict (`shipped_class_text`, `class_text_loads` in latexml_package/src/shipped_class.rs, as
  mn2e's journal abbreviations). jfm: `\ifCUPmtlplainloaded` (the template preamble's font checks; 597 first errors),
  the class's `\checkfont` block (amsbsy, upmath when shipped, `\upi`), the `lineno` option, `\aunote`, `\corresau` (a
  `role=corresponding` note), `\oneappendix`, and the author conveniences (`\Rey`, `\Pran`, `\ls`, `\ns`, `\p`,
  `\bnabla`, `\mathsfbi`, …) as `DefinitionOrigin::Fallback`, so a paper whose copy lacks one keeps its own
  `\newcommand`; by the copy: caption (35 of 119), `\le`/`\ge` as ⩽ ⩾ (73 of 119, jfm.cls:1361), `\mathsfbi` bold-italic
  Helvetica (newer) or slanted cmss (older). mdpi: by the copy, the packages its versions load (caption, lastpage,
  float, enumitem, mathpazo, setspace, epstopdf, upgreek, subfig; a 2013 copy loads no enumitem, so the paper's
  `[shortlabels]` load is the first) and `\conflictofinterests` (the statement under "Conflicts of Interest:", or the
  2012-2015 title form); as fallback guesses, the title-page setters as frontmatter notes (`\doinum` once accepted,
  `\secondnote`/`\thirdnote`, `\conference`, `\featuredapplication`, `\simplesumm`), `\addhighlights` (the switch; the
  block is the paper's own redefinition), `\acknowledgements`, `\publishersnote`, the page and layout setters,
  `\highlighting`, `\cellWidtha`…`\cellWidthz`, `\PreserveBackslash`, `\isAPAandChicago`. Run-340 estimate ~1,800 jfm +
  ~1,400 mdpi papers' first error (undefined_rootcauses.md row 1). Residuals: captions keep ": " where these classes print
  "Figure 1." (their own `\@makecaption`, or caption's `labelsep=period`, which our caption binding declares but does not
  apply); `\oneappendix` keeps the appendix letter. Guards
  `perfect_kernel_batch65v::*` (with `perfect_kernel_batch64qa::mdpi_history_is_frontmatter`: lastpage's `LastPage`
  label ids the document); repros `class-bindings/{jfm,mdpi}_class_commands`.
- **65w**: six more class bindings define what their versions define and papers call (the class-family rule, from
  run-340 first errors and a census of the shipped copies). lmcs (contrib): `\titlecomment` a `role=titlenote` note,
  `\ACMCCS`/`\amsclass` classifications named by `\ACMCCSname`/`\amsclassname`, `\rsuper`/`\lsuper` (the newer copies'
  braced `\MakeLowercase{#1}`), `\qEd`, `{iteMize}{label}`/`{desCription}` as `{list}` (the enumitem the binding loads
  would read the label as a key; ~900). lipics (contrib): `{claimproof}` ending ◁, at its own end or `\claimqedhere`
  (other proofs end ∎: the class's grey ◀ for every proof is not reproduced, our `\qed` ignoring `\qedsymbol` as Perl's does),
  `\claimqedhere`, `\lipicsEnd`,
  `\proofsubparagraph`, the `(i)`/`(a)`/`(1)` enumerates, `{exercise}`, `{note}` and the starred theorems, `\flag`'s
  image, and the 2016-2019 setters as `DefinitionOrigin::Fallback` (`\EventShortName`, `\serieslogo`, `\DOIPrefix`,
  `\DOI` whose DOI, when set, becomes a pubnote once at `\maketitle`, `\volumeinfo`'s event as a `role=event` note;
  ~700). atlasdoc (contrib): the document-data setters store as atlasdoc.cls does (`\journal` too), and at `\maketitle`
  the title header the class options pick gives the notes, each prefix the note's label: with `preprint`/`cernpreprint`
  the journal reference, or else "Submitted to: <journal>", DOI and preprint number; with `HEPDATA` the preprint number
  and arXiv id; with `BOOK` (unless `PAPER`/`CONF`/`PUB`) none; otherwise "<\subprepend> <journal>" ("Journal:");
  `\arXivId`/`\HepDataRecord` store; `\AtlasPaperWebpage`
  from `\AtlasRefCode`, `\AtlasDate` as the date, the cover-page commands no-ops (~700). spie (contrib, which the loader
  gives spieman): `\linkable` and v1.00's `\supscr`/`\supscrsm` as Fallback (~650). arximspdf: `\doiurl`/`\arxivurl`
  (moved from the arxstspdf-only loader; 92 of 100 arximspdf copies define them) and `\surl`, `\setattribute`/
  `\getattribute`, `\tsup`/`\tsub`, `\OrigBibText…\endOrigBibText` (~700). aa: the online floats are set at
  `\AtEndDocument` as the class sets them, in both arities, told apart by the first argument (a number is the v7 form's
  N: `\onltab{N}{table}` numbered N, the counter stepped to N; v8.x's `\onltab{table}` keeps the number it reserved where
  called; v9.x defines none), `\longtabL{N}{…}` in place numbered N, `\AANum`, `\psj` (Fallback), and hyperref at the
  document's start when the paper's copy loads it (`class_text_loads`; ~530). Guards `perfect_kernel_batch65w::*`; repros
  `class-bindings/{lmcs,lipics,atlasdoc,spieman,arximspdf}_class_commands`, `class-bindings/aa_online_floats{,_v8}`.
- **65x**: seven more class bindings define what their versions define and papers call (run-340 first errors; census of
  the shipped copies, `scratch_356a0c47/census_65x.txt`). PoS: `\PoS{<identifier>}` the contribution's identifier as a
  `role=pubid` pubnote (PoS.cls :1395, a second call undefined as in the class; ~360). dmtcs-episciences (contrib):
  `\publicationdetails{volume}{year} {issue}{number}{id}` the head's "DMTCS vol. 19:3, 2017, #12" and the license line
  as pubnotes, the copyright as its date, and the class's number sets with `\primes` (~320). svmult: the 2018+ additions
  as `DefinitionOrigin::Fallback` — `\orcidID` an ORCID contact (as llncs's), `\ethics`/`\ethicssubhead`, the titled
  boxes (`{important}`, `{warning}`, `{tips}`, … : the title bold over the text), `\biblstarthook` under the
  bibliography heading (through `\lx@bibliography@preamble`), `\subsubruninhead`, `{petit}`, `{refguide}`,
  `\leftcaption`/`\rightcaption`, `\leftfigure`/`\rightfigure`, `\samenumber` (~350). iopart: the 55 journal
  abbreviations of the 2011-2014 iopart.cls and `\MLST`, which Perl's list predates, as Fallback (~200); every journal
  abbreviation keeps the class's trailing space (KPE #634). acmart: the 2.x additions as Fallback —
  `\noindentparagraph`, `\anon`, `\AtBeginMaketitle` (run as `\maketitle` begins, through the kernel's new
  `\lx@maketitle@begin`), the `acmcp` setters (stored), the translated title/subtitle/keywords as notes and
  `{translatedabstract}`; `\acmBadgeR`/`\acmBadgeL` take their arguments (Perl's `\acmBadgeL` printed them). IEEEtran:
  `{LaTeXdescription}`, `\yesnumber`, `\IEEEstrut`, `\IEEEsettopmargin`, `\IEEEeqnarraystrutmode`. elsarticle:
  `\fnmark[…]`/`\cormark[…]` as `\fnref`/`\corref`, `\nonumnote`, `\newpageafter` (Fallback). Residual: svmult's
  `\subfigures` (3 of 120 sampled; its `merk` counter machinery). Guards `perfect_kernel_batch65x::*`; repros
  `class-bindings/{pos,dmtcs,svmult,acmart,ieeetran}_class_commands`,
  `class-bindings/{iopart_journals,elsarticle_marks}`.
- **65y**: symbol and unit packages define the names papers call (run-340 first errors still undefined at the int7
  head, per package, `scratch_356a0c47/r7/still.txt`).
  SIunits (440): its own `\addunit`, `\addprefix`, `\usk`/`\unitskip`, `\per`, `\power`, `\squared`…`\rpfourth`,
  `\SIminus`, `\SIunits[…]`, `\SI@fstyle`, `\degreecelsius` (as ℃) and `\upmu`, its 198 units and prefixes (a table of
  SIunits.sty:195-543; the `derived`/`derivedinbase` sets and begin-document `\micro`/`\square`/`\gray` by its options;
  `\meter`/`\deka` aliases; `\degree`/`\arcminute`/`\arcsecond` as glyphs), all by `\providecommand` over the siunitx
  shim's `\relax` placeholders, so a unit outside `\si` keeps its symbol (2103.01586's `$221\usk\meter$` lost it; OD #565). constants (253): ported whole, `\Cl` recording at once (no `.aux`
  round trip), `\null` as `\relax`. newtxmath (400): `\widebar`, `\Angstrom`, `\bigtimes`/`\varprod`, `\vv`,
  `\mapsfrom`, `\transp`, `\lBrack`/`\rBrack`, `\uppartial`, `\lambdabar`, `\smallsum`, `\bigcupdot`, `\sumint`, `\cdotB`,
  `\re@DeclareMathSymbol` (names defined first keep theirs, as `\DeclareMathSymbol` refuses them; `\vv*{x}{i}` the
  two-argument `\vvstar`). gensymb `\celsius` (214, KPE #635); eucal `\EuScript` + options (229); libertine `\useosf`
  and the type1 figure-style commands (233); fourier's math symbols (163; names amssymb defined first keep theirs);
  stmaryrd `\oast`/`\ocircle` (118); mathtools `\newcases`/`\renewcases` as `\lx@ams@cases` (93); amsbsy `\binrel@`,
  `\binrel@@` (its content keeps its own class: no box measures math spacing here), `\math@atom` (161). Residual: a `\Cr` before its
  `\Cl` prints LaTeX's first-run "C^st ??" with a warning (no `.aux` round trip; 1307.4858); the
  parser does not pair `⦀` fences (RED `math-parse/fourier_vert_fences`, math parked); `\newcases` variants revert as
  `\begin{cases}` (the shared kernel reversion, base_xmath.rs:1471, as Perl's for `dcases` too); acronym's first-use
  double space after `\AC@placelabel` is Perl's (acronym.sty.ltxml:85-86; structure/acro.xml), kept; not taken (below the bar): amsmath's
  `\@mathmeasure` without amsmath (34), mathtools' `\delimsize` outside a paired delimiter (11). Guards `perfect_kernel_batch65y::*`; repros
  `stub-dependencies/{siunits_unit_commands,gensymb_celsius,eucal_euscript,libertine_figure_styles,fourier_math_symbols,`
  `newtxmath_symbols,stmaryrd_oast,constants_labelled_constants}`, `ams-internals/{amsbsy_binrel_math_atom,mathtools_newcases}`.
  A/B (262 run-340 witnesses, `ab_65y`): errors 701→101, Error status 230→33; the 18 word-count drops are
  error-recovery tokens leaving the math (text outside math equal) and 1307.4858's forward `\Cr`.
- **65z**: package bindings define the commands and internals their real `.sty` defines that papers, or the raw code
  they load, call (run-340 first-error "undefined" cluster, row 7). caption: `\subcaptionsetup` (`\captionsetup[sub…]`),
  `\captionlistentry` (the caption's counter step for a `\label`, no caption), `\captiontext*` (the float's label and
  text, the panel "(a)" in a subcaption group; with text, "Figure 2: text", the label's separator as a caption's). bigstrut/multirow: `\bigstrutjot`. natbib: the `\hyper@nat…` hooks
  (kotex extends them). titlesec: `\newpagestyle`/`\renewpagestyle`/`\widenhead` load titleps; `\ttl@straightclass{X}`
  is the kernel heading at X's level (`\lx@startsection`, as `\titleclass`'s commands now: a paper's own `\@startsection`
  is not called). titling: `\thetitle`/`\theauthor`/`\thedate` set by `\protected@xdef` as `\title`/`\author`/`\date`
  run (titling.sty:245-268, chained onto new kernel slots `\lx@title@after`/`\lx@author@after`/`\lx@date@after`, which
  the revtex4 and authblk `\author` call too; a name whose slot never fired is set at `\begin{document}` and at
  `\maketitle` (for a field changed since) from `\@title`/`\@author`/`\@date` — amsart/mnras with the fields in the body, IEEEtran over its
  placeholder), not at load, nor re-`\let` at `\maketitle` (KPE #636: `\title{\thetitle}` looped). Residual (as the
  int7 head): `\theauthor` empty under llncs, acmart, elsarticle, svjour3, aastex, whose `\author` stores no `\@author`
  and calls no slot (RED `package-bindings/titling_setter_llncs`). array: `\col@sep`, `\@acol`, `\NC@list`, `\extratabsurround` (raw eqnarray.sty, ltxutil.sty,
  dgruyter). authblk: the state `\author`/`\affil` keep (`\AB@las`, `\AB@au@str`, `\AB@thenote`, …), which papers'
  copies of `\affil` read. IEEEtrantools: `\bstctlcite`, `\IEEEstrut`, `\yesnumber`, `\IEEEsettextheight`,
  `\IEEEsettopmargin` (IEEEtran: `\IEEEsettextheight`). accents: `\cc@accent` (a `\hat` written to the .bbl) and
  `\cc@style` (`\widebar`). acronym: `\@iaci` (the `\acrodefindefinite` article, which now also defines
  `\fn@<acr>@IS/IL`), `\@firstupper`, `\texorpdfstring` at begin-document (PERL-ORIGIN: acronym.sty.ltxml calls
  `\@iaci`/`\@firstupper` undefined). crop: `\CROP@size`, the stock size, the corner marks. enumitem: `\labelindent`,
  `\enit@format`, `\enit@type` set per list (0/1/2). ulem: `\ULforem`/`\normalem` (switching `\em`), `\ULset`,
  `\UL@setULdepth`. footmisc: `\DefineFNsymbolsTM`. listings: `\lst@CheckVersion` (shipped lstpatch.sty skips itself),
  `\lst@mode`/`\lst@Pmode`, `\lst@token`, `\lst@SaveOutputDef`. amsthm: `\thm@headsep`, `\dth@everypar`,
  `\displaymath@qed`. feynmf/feynmp: `\fmfkeep` in a graph keeps it for `\fmfreuse`; `{fmffile}` sets `\thefmffile`.
  Witnesses (2-3 per name) in the code comments and repro headers. Residuals: an `\affil` copy's extra text (an
  `\email` written to `\AB@affillist`) is not in the frontmatter; `\@itemlabel` is still the kernel no-op; footmisc's
  symbol sets are read, not used (`\fnsymbol` is native); `{fmffile}` opened in the preamble and closed in the body
  (hep-ph0206138) is an environment-nesting error; the raw alignment code that calls array's internals
  (eqnarray.sty, revtex's ltxutil.sty, dgruyter) next reaches the rest of array's/latex.ltx's preamble builder
  (`\@mkpream`: `\prepnext@tok`, `\@classx`, `\@classz`, `\d@llarbegin`, `\NC@do`), undefined; IEEEtrantools' non-eqnarray
  half (`{IEEEproof}`, `\IEEEQED…`, `{IEEEitemize}`, `\IEEEPARstart`), which the IEEEtran class binding has, is
  missing from the package binding (2211.06691, 2310.15756). Guards `perfect_kernel_batch65z::*`; repros
  `package-bindings/*`.
  Integration 8 (65y + 65z, four review rounds): bundle A/B 912 papers (`ab_65int8`; 65y/65z witnesses + int7 sample)
  errors 3056→2138, Error status 499→175, Fatal 1→1, bibliography/tables equal, the 18 word drops error tokens leaving
  math; manual net 797 manuals 0 diffs, 30 repros fixed, 0 newly failing.
- **66b** (run-340 Fatal origins still Fatal on the int8 head; sample of 273 over the classes in
  `~/data/pk_agents/main/run340/fatal/`: Stomach:Recursion 22/25, PushbackLimit 22/25, Mouth:EoF 24/24, Malformed 24/25,
  Timeout:Recursion 19/25, IfLimit 11/23 still Fatal). Three mechanisms, 14 of the 22 Recursion witnesses Fatal → 0
  errors (~215 papers by the sample rate): the kernel's caption constructor is `\lx@@caption` with `\@@caption` its
  alias, since the Cambridge UP classes keep `\caption` under `\@@caption` for `\contcaption` (fac/ijsipaper/mscs; OD
  #566, KPE #637); `\newcommand`/`\renewcommand`/`\providecommand` with an optional argument also define latex.ltx's
  internal `\\cs` that the amsart idiom `\edef\cs{\@dblarg\csname\string\cs\endcsname}` calls (bcp.cls `\abtitle`,
  cmfne.cls; OD #567, KPE #638; review r2: every `\@xargdef` declarator, `\newenvironment` and `\DeclareRobustCommand`
  too); a locked robust binding locks its inner `\cs␣` against the files, so a class's copy of cite.sty's
  `\@namedef{cite }` no longer takes over the kernel `\cite` (cmfne, asme2e; OD #568); image fragments gobble
  `\lx@@caption` as well as `\@@caption` (latex_images.rs, LaTeXImages.pm:411). Guards
  `perfect_kernel_batch66b::*`; repros `captions-floats/cup_contcaption_lets_caption`,
  `macro-state/{newcommand_optional_internal_cs,locked_robust_inner_cite}`. Residual: AMS-TeX `\documentstyle{X}`
  never raw-loads a paper-local `X.sty` (Perl the same, `notex => 1`), so amams.sty's `\title#1` meets amsppt's
  `\title … \endtitle` and the file ends in the argument (math0402285, math0101270, math9906120; ~33 papers; raw-loading
  needs an A/B against gr-qc9507042's fic.sty, which the amsppt fallback fixed); gtmacros papers ship no gtmacros.tex
  (math9804003, math9902122, math9905203: harness).
- **66c** (run-340 `\endgroup` cluster, the empty-trigger frame: 937 papers have an `\endgroup` error whose frame was opened
  by no token, half of a 40-paper sample mdpi papers closing `\end{paracol}`): the mdpi binding follows the copy the
  paper ships — a copy that loads paracol (mdpi.cls:55) opens `\begin{paracol}{2}\switchcolumn` (the first page's side
  column) in its `\AfterEndPreamble`, after its own `\maketitle` (2105.08579's :880-887; 1809.00601's :908-912 via
  `\maketitlen`), so the binding loads paracol and opens it there too, and the template's `\end{paracol}` closes it (raw
  paracol.sty converts cleanly: the columns flow); a copy without paracol (1307.3961's 2013 one) and a paper shipping
  none are unchanged. And (R3 of the study, OD #569) a lock against the files holds for their `\let` as for their
  `\def` (`state::let_i`: arabtex's `\let\endabstract\endquotation`, 2401.14559), except a `\let` that only clears the
  name (projlib's `\cs_undefine:N \endabstract`: the homework/jwjournal manuals), whose binding comes back when the
  file's new meaning is refused; mn2e_support's `{keywords}` locks both halves against the files (mn2e-breakabs.sty's
  `\endkeywords`, 1403.3615, 1607.03149; plus mn2e.cls:992 `\ifSFB@keywords`). Residual (pre-existing, 65int8 too):
  `\usepackage{arabtex}` sets the whole body italic (its `\all@wcmd` font wrappers; pdflatex cmr10) — RED follow-up. Witnesses 2105.08579, 2201.05363. Guards `perfect_kernel_batch66b::{mdpi_*, file_let_*,
  refused_*_of_a_cleared_name_restores_the_binding, mn2e_*}`. A/B over 74 mdpi papers (`ab_66c`: 45 of 65v's, 29 of the empty-trigger
  list) vs 66b1: errors 154 → 100, Error status 37 → 17, none Fatal; the only word changes are error text gone
  (`{paracol} 2 \switchcolumn` printed by the undefined environment). Residual: 1905.04581's own `\begin{paracol}`
  after pgfplots figures raises paracol's `\ifinner` "Not in outer par mode" (2 errors; a mode left inner upstream,
  not reproduced minimally). The rest of the `\endgroup` study
  (`run340/clusters/endgroup_rootcauses.md`): R1 an eqnarray cell `a}{b` (latex.ltx:15760-15765 `{##}` templates; the
  gullet's before-column marker already resets align_state as tex.web §324, but real braces would put `{…}` into every
  eqnarray `tex` attribute: needs a hidden template group, ~86 papers), R2 diagrams stub body read raw inside an
  alignment cell (~51; the stub is a binding since 67d), R4/R5 singletons; the parked mode-switch family (29/56) beyond mdpi.
- **66d** (run-340 Fatal origins round 2, `~/data/pk_agents/main/run340/fatal/fatal_rootcauses.md`: 110 papers still
  Fatal after 66b, 16 mechanisms). Seven fixed, red/green with repros: R1 the caption-label hoist closes its
  `\ifx.#5.` before the caption is digested (latex.ltx's `\@firstoftwo` idiom; a stray `\fi` or open `\if` in a caption
  took the wrapper's `\else`/`\fi`; OD #570, KPE #641); R3 AMS-TeX `\Bbb`/`\bold` are font switches, amsfonts `\bold` is
  `\mathbf` (KPE #639); R4 AMS-TeX's `\abstract` drops the LaTeX kernel lock and amsppt.sty's state for amsppt1.tex exists
  (OD #571, KPE #642); R5 a driver with `\begin{document}` vetoes the preamble it inputs (main_tex.rs and cortex_worker.rs;
  OD #573); R6 `{filecontents}` ends at its `\end{…}` anywhere in a line (OD #572); R10 the kernel `\shorttitle`/
  `\shortauthor` are stubs a `\newcommand` replaces (OD #574); R13 `\lbrack`/`\rbrack` are macros (KPE #640); and, from
  the witness reconversion, amsppt's `\topcaption`/`\botcaption#1#2\endcaption` (amsppt.sty:820-836; q-alg9505010,
  q-alg9507011) and amstex.tex's `\Err@`. Bundle witness reconversion (user 2026-10-13: every arXiv id in
  `35feb675fa..` and the comments of touched code, 88 papers through the CLI's main-file pick, `~/data/pk_agents/main/
  wit66`, 65int8f vs 66d3): 38 Fatal → converted (Fatal 50 → 7, words 0.81M → 1.76M), 6 fewer errors (mdpi, mn2e,
  arabtex), 39 identical (incl. 2406.14142, 1905.05350, 2307.13586, 2606.02077), cond-mat0001395 changed for the better
  (jpsj.sty's raw cite copy no longer takes `\cite`: 0 → 51 `bibref` citations, the "?" marks gone); manuals latex4wp,
  gaceta, homework, jwjournal byte-identical. Errors 766 → 1006 only because the R1 papers now convert and show their
  authors' stray conditionals (1904.11818 594, 2012.08626 112, 2210.17505 97, 1602.01884 20; pdflatex reports each).
  Still erroring: AMS-TeX `\tag 1.2$'$` (1012.3760 Fatal → TooManyErrors, 1108.6264 3, 1112.1423 4; RED
  `amstex/amstex_tag_with_math_text`, next — fixed in 66e); 1905.04581 (paracol `\ifinner`, known); unchanged pre-existing errors in
  2304.05575 (`\BAmulticolumn`), gr-qc9405068/gr-qc9406008 (the author's `\beginabstract` pair), gr-qc9507042
  (merged creators), math0012150 (active `@`), 2401.14559 (3), and the AMS-TeX amams/gtmacros Fatals of the 66b residual.
  Bundle A/B (1009 papers, `~/data/pk_agents/main/ab_66bundle`, 66cr2 vs 66d2): identical tallies (E 2241, F 1), 12
  papers' content changed. Six lose the running-head paragraph the kernel `\shorttitle`/`\shortauthor` had leaked into
  the body (numapde 1908.02022, 2011.03580, 2301.02516; jnsao 2011.04474, 2001.07609; jpp-AAS 2305.03275; titles and creators
  unchanged); five print amsfonts `\bold` bold (1210.0557, 1811.00558, 1903.00970, 2005.01953, 2303.01678); and
  2210.02246 had lost cmfne's end-of-document author lines — 66c r3 skipped the `\@xargdef` internal of a refused
  definition, which the replayed `\@dblarg` setter calls; restored (OD #567, guard
  `perfect_kernel_batch66b::refused_dblarg_setter_reaches_its_internal`). Manual net (65int8f vs 66d4, `mnet_66d4`): 0
  regressions, 13 repros fixed; 4 manuals changed, all from 66c r4 sending every locked binding down `\let`'s plain
  path, which leaves plain `:locked` to `\let` — bera.sty:12-23 `\let\LaTeX=\fveLaTeX` (robust) replaced the `\LaTeX`
  logo constructor with the package's typeset boxes (enumitem, colorspace, mathador-doc; tikzbrickfigurines-doc's node widths). The skip
  now covers a locked inner only; the 4 manuals are byte-identical to 65int8f again (OD #568, guard
  `perfect_kernel_batch66b::robust_let_over_a_locked_constructor_keeps_it`); no witness or A/B paper's logo count moved.
  Guards `perfect_kernel_batch66d::*`,
  `main_tex::tests::{driver_vetoes_the_preamble_it_inputs, wrapper_without_begin_document_keeps_its_main}`. Open:
  R2 aastex `D` vs preloaded dcolumn (~50), R7 xargs robustness (~21), R8 `\enddocument` tail (~21), R9 hyperref
  `\H@refstepcounter` (~12), R11 qbordermatrix IfLimit, R12 shipped core-file copies, R14-R16 singletons.
- **66e** (AMS-TeX `\tag`, the 66d "next"): `\tag` was `\let` to `\eqno` (Perl too, KPE #643), whose tag is math up to
  the first `$`, so `\tag 1.2$'$` closed the display ("Missing $" cascade). amstex.rs `amstex_tag` reads the tag text as
  amstex.tex:597-605 does — unexpanded and brace-balanced up to the display's `$$` or, in a row (:629-640, 718-739, 797;
  a `\multline` tag follows `\endmultline`, :903, a display tag), to `\\`/`\cr`/the environment's `\end…` — and sets it
  with amsmath's `\tag` markup (`\tag*` for a quoted `"…"`, `$…$` under `\TagsAsMath`, amstex.tex:590-593, whose amsppt
  no-op stubs are gone). With no equation row (a plain `$$` display, also `$$\align` = amsmath's `aligned`) the tags
  float to the display (amstex.rs `\lx@equation@settag@float`, in the AmSTeX pool so a plain `\input amstex` + `\input
  amsppt.sty` document without the LaTeX pool has it; `\documentstyle` makes the route LaTeX, whose amsfonts
  `\DeclareSymbolFont` loads LaTeX.pool); one built in an `aligned` row is tied to it, and `rearrange_lone_ams_aligned`
  gives it to that row's equation. A row's tag ends at every row ender (`\displaybreak`, `\allowdisplaybreak`,
  `\intertext`, `\vspace`, a token meaning `\cr`/`\crcr`/`\\` or a macro expanding to one), the row test needs an
  alignment opened in the display's own math, and a delimited `{…}` argument loses its braces (`\tag{"A"}$$` quoted). In
  the `$$` form a `\par` abandons the call with TeX's runaway error and a `}` closing an outer group with TeX's two
  (tex.web §395-396: "extra }", then the inserted `\par`'s runaway; the input goes on with `\par }`); a row's tag (a
  LaTeX equation row's too) takes no argument, so there a `\par` outside the tag's own groups, or such a `}`, ends the
  tag cell without error; a `\par` inside a group of the tag that closes stays in its text (`\tag{3\par}\\`), and a
  row's or environment's end read inside an open group of the tag (`\tag{5\\ c&=d\endalign$$`) stops the scan at once:
  TeX's "Missing } inserted" per open group (tex.web §1126-1127; the runaway error if a `\par` was read in a group
  still open), the tag keeps its text with its groups closed, and the rest is read again, so no row, enclosing group
  (`{\it …}`, `\vbox{…}`), later paragraph or `\enddocument` is lost (r3 read them into the tag; r4 still did when an
  enclosing group's `}` came first); the document's end (`\enddocument`, `\bye`) is never read into a tag (a row end;
  in the `$$` form TeX's "Forbidden control sequence", the call dropped), and a `$` closing inline math in a tag is never
  the start of `$$` (`\tag{$a$$b$}\\`); the shapes where TeX errs and ours recovers are OD #575. In a LaTeX equation row a braced
  `\tag{…}`/`\tag*{…}` is amsmath's (an `\input amstex` or `\NoBlackBoxes`-autoloaded LaTeX document read `a=b` into the
  tag before 66e r3). r2 A/B (66d6 → 66e r2, 122 AMS-TeX zips): E 721 → 692, F 5 → 5, no paper worse, the 17 r1
  regressions (math9412232, hep-th9308082, hep-th9206021, hep-th9111004, 1102.5649, …) back to 66d6's counts or better;
  200 LaTeX `\tag` zips unchanged. Witnesses (66d6 → 66e, `wit66`): 1112.1423 E 4 → 0, 1108.6264 E 3 → 1 (merged
  creators), 1012.3760 still E 101 + TooManyErrors, but its first error moved from line 134 to 904 (lines 1-902: E 7 →
  1, all 60 tags text, 57 on their row/equation vs 51): the next class is `\Sb … \\ … \endSb` after `\input pictex` (RED
  `amstex/amstex_pictex_sb_newline`). A/B over 30 AMS-TeX `\tag` papers: no paper worse; math0001138 and math-ph0001014
  E 2 → 0. Residual: `$$\gather … \tag 1\\ … \endgather$$` keeps both tags on the one display equation (amsmath's
  `gathered` is not split into rows); AMS-TeX tags sit on the left in the PDF (`\TagsOnLeft`, amstex.tex:590), ours on
  the right; `\intertext` inside `$$\align` errs (RED `amstex/amstex_align_intertext`, before 66e too); `$$\gather` in a
  plain document errs "`\ams@return@opt@arg` undefined" (amsmath_sty.rs `\lx@ams@start@box` lets it to `\@empty`, which
  only the LaTeX pool defines; math9412232 and 6 more of the 122, 36 errors, before 66e too; RED
  `amstex/amstex_plain_gather_return_opt_arg`); an unbraced inner alignment in a row tag (`\tag 1\matrix a\cr
  b\endmatrix\\`) stops the tag at the inner `\cr` and errs; an active character let to `\\` (`\let~\\`) does not end a
  row tag (`amstex_tag_row_end`'s `Catcode::CS` gate); a `\par` in `$$\align … \endalign$$` raises no error (TeX's
  `\align` reads its body as an argument: "Paragraph ended before \align was complete"; before 66e too); an amsppt
  `\footnote{` whose `}` never comes reads to the file's end, the later paragraphs inside the `<note>` (amsppt_sty.rs
  `\footnote[]{}` reads `\long`, amsppt.sty:804/:814 `\nextiv@##1` does not; RED
  `amstex/amsppt_footnote_argument_runs_past_par`); an alignment open at `\enddocument` drops its open row's last cells
  without an error (RED `amstex/amstex_align_unclosed_at_enddocument`, before 66e too); a display open at `\enddocument`
  takes `\enddocument` into its formula's `tex` (RED `amstex/amstex_display_unclosed_at_enddocument`, before 66e too). Guards
  `perfect_kernel_batch66e::*`.
- **66f** (sandbox regressions, cortex runs 341/342 on 7a93c36531 vs the 2026-10-04/05 statuses; reports
  `agent_reports/2026-10-13_sbx2605_regressions.md`, `_sbx2606_regressions.md`): of 412 (2605) + 444 (2606) OK/Warning →
  Error/Fatal, 375 + 416 are intended signal (BibTeX undefined macros and parse errors, OD #518; `merged_creators`, 63e),
  3 + 4 fleet-only OOM/xpath; the rest root-caused. Fixed: `\begin{document}` opens `\jobname.aux` as latex.ltx's
  `\document` (OD #576; jpsj3/bio/lamuphys/caosp `\enddocument` re-reads; 2605.24927, 2605.04825, 2605.28673,
  2605.20125, 2606.02098, 2606.24241, 2606.25242, cond-mat9607109, astro-ph9805185); `\mathversion` expands its name
  (KPE #646; XCharter/figureversions, 2605.11518, 2605.12374, 2605.22763); an unnumbered appendix unit in a float
  (OD #189; 2605.02378, 2605.25534); authblk `\affil` robust (KPE #647; 2605.03796, body 15 → 7,748 words); SNmult.cls →
  svmult binding (2605.13070); url `\path` only if undefined (KPE #644; 2605.13185); jmlr `\jmlrissue`/`\jmlrpages`
  (2605.20066); IEEEtran `\appendices` through a copy of the kernel `\appendix` (2605.23316, Fatal → 0); nicematrix
  `[opacity=…]` color keys (2605.25123); braket `\set` splits only at `|` (KPE #645; 2605.31379). Every witness 0 errors,
  text equal or recovered. From 2606: natbib `\citealp`/`\citealt` and the capitalised forms through private copies of
  `\citep`/`\citet` (natbib.sty:702-730; since D1 a document's `\renewcommand{\citep}{\oldcitep*}` owned them,
  2606.16139 E 11 → 0, 2606.30583 E 69 → 0); calc's `<optional equals>` (2606.05909); appendix.sty internals (KPE #648,
  2606.09094); amsart `\@classname` (read in amsart/amsproc/amsbook's own load: the loading class) and `\subjclassname@<year>`
  (KPE #649, 2606.20943); IEEEtran's author pre-scan keeps a
  run whose `\else`/`\fi` close an `\if` from a block argument whole (2606.17718; residual: an empty affiliation contact
  from the skipped branch); `\hypertarget`: hyperref.sty:4805-4811 (nesting off) puts `#2` after the anchor in every mode; our mode split keeps
  Perl's anchor-holds-its-text in text, and in math an empty anchor with `#2` placed after the `\ifmmode` test, so a
  brace-less `\frac` takes its own arguments (hyperref's `#2` precedes its `\fi`; a `\relax` first keeps a row-initial
  call in `align`/`aligned`/`array` on it; OD #578; 2606.15438);
  braket's split never inside `\left…\right`, the count never below zero (2606.06185, KPE #645). Each 2606 witness
  E → 0. Guards `perfect_kernel_batch66f::*`. RED from its review (pre-existing on 66e1):
  `package-bindings/hypertarget_math_anchor_keeps_its_id` (a math anchor's `xml:id` is generated, so `\hyperlink`
  dangles, and the anchor parses as an operand), `fonts-nfss/mathversion_normal_restores_the_normal_version`,
  `sectioning-frontmatter/unnumbered_unit_in_float_pins_the_float_id` (Ax1.F1 for A1.F1),
  `sectioning-frontmatter/chapter_star_in_float_is_an_inline_block` (`malformed:ltx:chapter` in report),
  `package-bindings/biblatex_natbib_capitalized_cite_commands` (`\Citet` undefined; blx-natbib.def:49). Open (2605 report): C3 nomencl glossary in a float, C6 standalone sub-preamble loads, C7
  `\mathcolor` group, C8 bigfoot listing in a footnote, C11 JASA revtex4-2 internals (`\@AF@join`, `\@email`), C12
  2.09-style `\ds@<opt>` handlers (KPE-class, Perl Package.pm:2470), babel's on-demand Lithuanian `\fnum@table` loop
  (2605.29738; our kernel lacks `\fnum@figure`/`\fnum@table`), revtex4-2 `\frontmatter@footnote` (2605.04155); from
  2606: N1 scrhack/floatbytocbasic patching our primitive `\newfloat` (2606.04906, 2606.11358), N6 the expl3 scope-exit
  heuristic firing on a package that sets `_` a letter itself (etextools, 2606.06323), N7 NFSS shape probes
  `\try@load@fontshape`/`\DeclareFontShape` no-ops (2606.07112, Perl the same), N10 a sectioning unit in `ltx:quote`
  (2606.19241) and an `\item[]` excluded from the OD #189 item bridge (2606.00118) — both await a ruling.
- **Runs 346/347** (E/F rerun on `3ad0bbe622`, 66f-66h; vs runs 344/345 end): E 105,835 → 101,776, F 4,625 → 4,478;
  E→OK 1,290, E→W 2,814, F→E 171, F→W 97, E→F 126 — none a regression (agent_reports/2026-10-14_ef347_triage.md: 113
  not reproducible locally, fleet contention at the 5632 MB child cap / 180 s on an E/F-only rerun of the heaviest papers;
  13 long-standing slow or memory-heavy). Open from it: libmarpa `marpa_g_new` aborts on allocation failure with no
  Fatal line (SIGABRT; fix in the marpa-asf fork: an overridable OOM hook → `Fatal:oom:alloc_failed`), XPath OOM floods as
  one `Fatal:oom`, a 2.7-3.1 GB peak RSS for 4-10 s papers. Runs 348/349 (E/F rerun on `a7f5e7308e`) started 13:15Z.
- **67d** (Paul Taylor's `diagrams`, not in TeX Live, in arXiv's tree; run 349: `\diagramstyle` 467, `\newarrow` 441,
  `\rTo` 362 papers undefined): a native binding (latexml_contrib/src/diagrams_tex.rs, after amscd's `{CD}`; surface
  from the package manual v3.97 and the 100 witnesses' use) where Perl's ar5iv stub put `<ERROR>{diagram}</ERROR>`:
  `{diagram}`/`\diagram` is a math array of objects and labelled arrows (display in text, inline in a `c` cell or LR
  box, `[LaTeXeqno]` numbered, `\\` = the package's `\cr`), `\newarrow` draws each direction with the Unicode arrow
  of its components, `\pile`, text arrows, plain TeX `\input diagrams` (OD #589). An arrow reads the source's
  arguments: inside a diagram `(x,y)`, `<`, `>` too, outside one not (`$X\rTo (0,1)$` keeps its pair); a `.` and a
  positional `{g}` after `^f` as the source reads them; arrows and `\diagram` are mode-tested when digested, so one
  at an alignment cell's start is in the cell's maths (`&\rTo` in `align*`, math0404431); a horizontal arrow in text
  is the package's Error ("Commutative Diagram: "$" inserted before horizontal map"), a vertical one outside a
  diagram, in maths or text, its other Error ("vertical map ignored outside diagram"), a diagonal one none, as
  pdflatex; each arrow is kept (in text set as a formula); the `amstex` option (package option, `\diagramstyle[…]`
  or `\diagramstyle{…}`) loads amscd's `{CD}` (math0104206, math0403203). A/B of 102 witnesses vs 66x1: Errors
  468 → 86, Fatals 0 → 0, words +53k, 1,089 `<ERROR>{diagram}` → 1,267 diagram arrays (the 99 harness-run papers:
  1,083 → 1,250); no paper with more Errors/Fatals; 13 with 2-71 fewer words, all the printed arguments of the
  undefined `\newarrow`/`\diagramstyle`/`{CD}` (`@>>>`) and `<ERROR>` macro names.
  Witnesses math0001049, math-ph0001006, math0003019, math-ph0007001, math0010298, math0012073, math0102115,
  math0104065, math0104175, math0104206, math0105166, math0109209, math0203179, math0306018, math0310311,
  math0403203, math0404431, cs0204036; repros `package-bindings/diagrams_*`; guards `perfect_kernel_batch67d`.
  Open: the pullback corners (`\SEpbk`, a visible ⌟; CSS-placed corner proposed for the end-of-goal CSS review) and
  `\HonV`/`\VonH` draw nothing; `eqno=`, `\diagraminline` and the dotted `.` are not implemented; a vertical
  arrow's left label leaves its cell unparsed (`ltx_math_unparsed`, as amscd's `@VfVV`).
- **Fleet TeX tree** (2026-10-14, host change, no code): the cortex fleet reads the distro TeX Live
  (`/usr/share/texlive`, worker.env sets no TEXMF*), which lacked whole collections the vendor TL 2025 of the gates and
  A/Bs has, so some clusters were fleet-only (`{fmffile}` 298, `\Tree` 302 papers in run 349). Installed at the same TL
  version (2025.20260124-1, 0 upgrades): texlive-metapost, -humanities, -music, -games, -formats-extra, -xetex, and
  Debian's separate `feynmf`. The live dumps are unaffected (their source stamps name latex.ltx/expl3, unchanged).
  The full apt list (texlive-full is not needed) and the host check: `performance/CORTEX_WORKER_HARNESS.md` "Host TeX
  Live for arXiv coverage". Before calling a cluster a code bug, compare the distro `kpsewhich` with the vendor one
  for its package. E/F rerun
  on 4f131f2b84 started 20:13Z (baseline 2026-10-14 19:25:48.785430).
- **67g** (run 349 Error→Fatal regression 2109.06028): pict2e keeps `\@tempdimd`, which it allocated before v0.4b and
  which TeX Live's own ellipse.sty/longfbox.sty still use (TL 2025 pdflatex fails on both), loads trig as pict2e.sty:123
  does, and binds the graph-code layer (`\pIIe@add@nums`/`\pIIe@add@CP`/`\pIIe@addtoGraph`) ellipse.sty drives
  (OD #590; `perfect_kernel_batch67g`). 2109.06028: 643 errors, then the same-error cap's Fatal once 66l-67e removed the
  alignment errors that had interleaved, → 0. Run 349: 23 papers with undefined `\CalculateSin`, 18 with undefined
  `\pIIe@addtoGraph`. With it, 66w's look-ahead gives up at a `\catcode` (the tokens after it are not the ones TeX
  reads; `par_before_a_catcode_change_ends_the_formula`) and steps over only a `_` or `^` after a backquote
  (`par_before_a_backquoted_{formula,ref}_ends_the_formula`).
- **67j** (frequency-first, run 349 "undefined"): native bindings get their class's own user macros. espcrc (2004
  v1.2): `\readRCS` reads the `$Id$` stamp (espcrc1.sty:27-31), `\runtitle`/`\runauthor` store the running heads
  (:156-159). svjour `epjCONF` (svjour.cls:78-81 reads svepjCONF.clo): `\session-title` stored (svepjCONF.clo:234).
  imsart: `\let\endbibitem\relax` (imsart.sty:2514), every structured entry drops its `[author]` key list (:2585-2769,
  printed before every entry until now), `{binproceedings}`/`{bmanual}`, `\setattribute`/`\getattribute` (:464-466),
  `\thanksmark` (mark dropped). siamltex: `\slugger` stored (siamltex1213.cls:184-204). amsppt `\tenpoint`/`\eightpoint`
  (amsppt.sty:153-214), AmSTeX `\goth` (amstex.tex:1509). tcilatex: the binding (the 2.5 copy, chosen over a shipped
  3.0-5.0 tcilatex.tex as bindings are) gains `\TCItag` = amsmath's `\tag` (5.0 :776-783) and 4.0's `\U`/`\UNICODE`
  placeholders. In math `\eightpoint`/`\tenpoint` change the size only (the math families are resized, letters stay
  math italic: `\textfont\@ne\eighti`); the roman is text-only (1306.2523: 718 upright letters in r0 → 0). A row whose
  only content is its tag (Scientific Word's `… \nonumber \\ && \TCItag{27}`, amsmath's `\notag \\ \tag{29}`) is
  kept with its tag, where Perl drops it (KNOWN_PERL_ERRORS #682, OXIDIZED_DESIGN_DIVERGENCES #591; nlin0106009's 11
  such tags, lost in r0, are back). imsart's `\thanksmark` is locked. KNOWN_PERL_ERRORS #681; repros
  class-bindings/{espcrc_*,svjour_epjconf_*,imsart_structured_*,siamltex_slugger_*},
  amstex/amsppt_point_sizes_and_goth, package-bindings/tcilatex_tcitag_*, alignment/ams_tag_only_row_keeps_number;
  guards `perfect_kernel_batch67j`. A/B (82 papers, the 22 witnesses + ≤10 per family from the run-349 log scan): 275 →
  155 Errors, none worse. Every `\TCItag` in the 13 tcilatex papers is emitted as a tag. The words lost are the ERROR
  text, the `$Id$` math, the running heads, slug and session lines, "[author]", the `\U{…}` argument text (now the
  PDF's square), and tag text that sat inside the formula as math tokens and is now one `<tag>`. RED repro
  alignment/multline_notag_then_tag: multline ignores its `\tag` and `\notag` (always (n); hep-th0210191; Perl alike).
  Residual in those papers, other mechanisms: plain TeX's
  `\settabs\+` under the LaTeX route (alg-geom9410018, hep-th9306053), amsppt internals papers copy
  (`\textonlyfont@`, `\ifsyntax@`, `\eightrm`: funct-an9706006, math0002200), imsart `{supplement}` without
  imsart.sty (1602.02055).
- **67l** (frequency-first, run 349 "undefined": `\State` 560, `\For` 470, `{algorithm}` 544, `\newfloat` 365):
  which algorithm and float definitions a paper gets when a binding stands in for the file that loads them. A stand-in
  binding takes the algorithm packages from the paper's copy of the file it stands in for
  (`shipped_class::require_shipped_algorithm_packages`, the dependency scan kept to `ALGORITHM_PACKAGES`:
  `require_dependencies_only`, content.rs; OD #592): icml2016-2025 and versioned ICML names (papers edit `algorithmic`
  out of their copy for algpseudocode, which the distributed pair bailed: 1706.03643, 2002.06473, 2106.05074),
  WileyNJD-v2/v5 (`algorithm, algorithmicx, algpseudocode`, a copy algorithm2e: 1810.03033, 2106.06046, 2402.13183),
  bmvc2k (algorithmic or algpseudocode by year: 2210.06270, 2210.01640), pnas-new (2402.10236), edited neurips and
  siamltex copies (2402.18905, 2210.03478); without a copy, ICML keeps the distributed pair, the others none. REVTeX
  4.1/4.2 and AASTeX 6+ define the class's `\newfloat@ltx`/`\newfloat@float`, which preambles let `\newfloat` to
  before algorithm.sty (all 25 sampled `\newfloat` papers: 2210.07980, 2210.08566, 2002.07183; KNOWN_PERL_ERRORS #683).
  autart gets the class's own `{algorithm}` float (autart.cls:568-577; 2106.14831, 2402.17000). The scan also drops a
  comment ending a file without a newline. Repros package-bindings/icml_style_copy_without_algorithmic,
  class-bindings/{revtex_float_repair_before_document,wileynjd_class_copy_loads_algorithm_packages,
  bmvc2k_class_copy_loads_algorithmic,autart_algorithm_float}; guards `perfect_kernel_batch67l`. A/B vs 67k1 (144
  papers: the run-349 `\State`/`\For`/`{algorithm}` sample, 31 more ICML papers 2017-2026, 7 Wiley, 3 bmvc2k,
  8 neurips): 3,118 → 279 Errors, Fatal 1 → 0, 83 better, 2 worse, both a decoy main the harness picked (the ICML
  template's example file, which needs the algorithm packages the paper's copy no longer loads: pdflatex 16 and 15
  errors; the real mains 14 → 0 and 15 → 0); revtex/aastex (31): 73 → 2. Words lost: 2301.00557's float now comes from
  its own algorithm2e alone (its copy loads no algorithm package). Residual, the paper's own combination: algorithmic
  loaded before algpseudocode (1706.04918, 2506.05791, 2506.13672, 2506.01215) bails algorithmicx as in Perl, where
  pdflatex errs too ("Command \algorithmic already defined", then each `\State`). Not covered: IEEEtran (a Perl
  binding) ignores an edited copy's algorithm packages (2506.04683). Perl's versioned fallback answers a paper's
  icml20xx.sty with icml.sty.ltxml and forces the pair too (KNOWN_PERL_ERRORS #684): the ICML part is beyond Perl.
  Open from the review: a nested dependency scan reads as "no copy" (`require_dependencies_filtered` returns `false`
  under the re-entrancy guard, so ICML forces the pair over a shipped copy loaded from an unbound class's scan); the
  pnas-new, siamltex and icml2019 raw-read paths have no guard; autart's caption period sits outside the bold
  (autart.cls:568-569 `{\bfseries #1.}`).
- **67m** (fleet throughput, run 351): an allocation that fails when the address space is spent no longer wedges the
  worker. `custom_alloc_error_hook` (cortex_worker.rs, latexml_oxide.rs) formatted a backtrace under the fleet's
  `RUST_BACKTRACE=1`; symbolizing it allocated while holding the standard library's backtrace lock, the failure
  re-entered the hook, and the re-entry waited forever on that lock — with the stderr lock held, so the watchdog's
  `Fatal:timeout` line blocked too and its 180 s exit never came. Children sat parked (conversion and watchdog
  threads both in a futex wait, 3-5 GB resident) until the harness's 360 s no-CPU-progress SIGKILL, which re-leases
  the paper as a retryable death: ≥574 such kills in 4 h of run 349/351 (journald dropped ~1M lines per 30 s, so a
  lower bound), 17 of 53 children parked at one sample. A re-entry that did not deadlock recursed instead: one child
  printed 6,048 nested `Fatal:oom:alloc_failed … 1 bytes (align 1)` lines in a millisecond (reading `RUST_BACKTRACE`'s
  value). The hooks now allocate nothing and take no lock (`watchdog::write_stderr_unlocked`, a stack-formatted
  `write(2)`; no backtrace), a re-entry ends the process at once and another thread's failure waits for the first's
  exit (`watchdog::enter_alloc_failure_hook`), and they end with `watchdog::exit_now` (`_exit`). The watchdog writes
  its two Fatal lines unlocked too. The harness child also registers pericortex's per-task input directory
  (`cortex_task*`, a `TempDir` its loop drops) for the exit sweep: every timeout and OOM exit had left one behind —
  565,483 of them, 434 GB, in the fleet's `/opt/cortex-scratch` since 2026-07. Stuck-child witnesses (fleet 4f131f2b84,
  symbolized by rebuilding that commit with `STRIP=none` and the same embedded dumps): 1804.07640, and the tasks
  33865595 (math parse, `XM` clone under ASF traversal) and 32779664 (`SymHashMap::insert`); 1804.07640 converts in
  13 s standalone, so the state is a long-lived child's. Guards (need `--features cortex`, which CI and the default
  suite do not enable) `cortex_worker_exit_reports::{an_allocation_failing_inside_the_hook_still_ends_in_one_fatal_oom,
  a_watchdog_timeout_is_reported_as_fatal_timeout, an_allocation_past_rlimit_as_is_reported_as_fatal_oom}` (the
  `alloc_exhausted` test fault spends the address space through the raw allocator first). Residual: a hook whose sweep
  itself fails to allocate exits without finishing it.
- **Runs 364/365** (E/F rerun of run 363 on 026cf18f87: 68c Part B, 68a; COMPLETE 2026-10-16 12:20:31Z):
  E 79,627 → 77,932, F 3,971 → 3,971 (success 97.14 % → 97.20 % of 2,922,325). Transitions vs run 363: E→OK 362,
  E→warning 1,330, E→F 33, F→warning 3, F→E 30. E→F: `timeout` 22 + `Timeout:Convert` 2 and `oom` 4 (fleet contention,
  retried per the 2026-10-12 ruling), `TooManyErrors` 5 — all five row **XP** (98-501 detail-less `post:xpath` Errors each:
  1809.02905, 2007.00624, 2104.14033, 2602.10769, hep-ph9803283); no conversion regression. Cluster rescan of the run-353
  Error/Fatal set (latest zip per paper, `~/data/pk_agents/main/scratch_356a0c47/attr_now/`): `undefined` 53,902 → 43,535
  papers, the structural categories flat (`unexpected` 23,505 → 23,064, `malformed` 8,564 → 8,551), the greedy
  clean-paper ranking unchanged at its top (`_` 2,035, `ltx:bibitem` 1,109, `&` 1,096, `\errmessage` 1,081, `\noalign`
  1,005); 24 papers now stop at mathtools internals (`\MT_start_cases:nnnn`, `\MH_else:`) after 67u's
  `\MHInternalSyntaxOn` (all Error before).
- **Runs 362/363** (E/F rerun of run 361 on 58956d026a: 67w, 68f, 68b; COMPLETE 2026-10-16 08:34:36Z):
  E 81,301 → 79,627, F 3,994 → 3,971 (success 97.08 % → 97.14 % of 2,922,325). Transitions vs run 361: E→OK 293,
  E→warning 1,386, E→F 35, F→OK 1, F→warning 17, F→E 40. E→F: `timeout` 22 + `Timeout:Convert` 2, `oom` 2,
  `TooManyErrors` 9. Reconverted locally on 1ca233a402, 58956d026a and 026cf18f87 (`~/data/pk_agents/main/ab_66bundle/
  results_e363{a,b}.tsv`): the 2 `oom` and 8 of the `TooManyErrors` convert with 1-14 Errors and no Fatal, identical on
  all three. Those 8 are not conversion failures: on the fleet, post's XPath evaluation of `ltx:XMath` failed 87-501 times
  "(no libxml2 error detail)" (latexml_post/src/document.rs:959) until the error cap — a libxml2-internal failure (C
  allocation under a long-lived child's address space, which the Rust allocator hook does not see), not reproducible
  standalone; queued as a retryable resource failure (row **XP** below). One is a regression: 1906.07043 (iopart + breqn)
  6 → 101 Errors + TooManyErrors since 68b, because breqn now loads amsmath as breqn.sty:46 does, and amsmath's `\cases`
  (an "Old form" error, as pdflatex gives with the paper's own iopart.cls) leaves the cell's `$x>l_0$` inside
  `{equation}`'s alignment, where the `$` does not end the display as TeX's §1197 ("Display math should end with $$")
  does; pdflatex: 9 errors and a PDF. That is row **MS**'s open item (1), now with this witness.
- **Runs 360/361** (E/F rerun of run 359 on 1ca233a402: 67y, 68d, 67x, 67z; COMPLETE 2026-10-16 04:30:35Z):
  E 86,301 → 81,301, F 4,011 → 3,994 (success 96.91 % → 97.08 % of 2,922,325). Transitions vs run 359: E→OK 894,
  E→warning 4,100, E→F 36, F→OK 4, F→warning 19, F→E 30. E→F: `timeout` 30 (two writers and the gates shared the
  host), `TooManyErrors` 3, SIGABRT 1 (math0407433, again as in run 357), SIGSEGV 1 (2112.13654), `Timeout:Convert` 1;
  every reconvertible one converts locally with no Fatal and the same output on 595ce52d9e and 1ca233a402 (fleet
  contention, not a regression; `~/data/pk_agents/main/ab_66bundle/results_ref361.tsv`).
- **Runs 358/359** (E/F rerun of run 357 on 595ce52d9e: 67q, 67r, 67p, 67s, 67u, 67ps; COMPLETE 2026-10-16
  00:15:50Z): E 89,367 → 86,301, F 4,091 → 4,011 (success 96.80 % → 96.91 % of 2,922,325). Transitions vs run 357:
  E→OK 480, E→warning 2,602, E→F 26, F→OK 1, F→warning 63, F→E 42. E→F: `timeout` 14 + `Timeout:Convert` 1, `oom` 6,
  `TooManyErrors` 4, `PushbackLimit` 1. Reconverted locally (595ce52d9e and 4a7853352a agree on every one): 9 of the
  12 non-timeout ones convert with no Fatal (fleet contention), 3 are regressions of this bundle against 1a95291ef6:
  2104.09383 (373 Errors → PushbackLimit, 0 words), 2201.11056 (6 Errors, 22,367 words → `oom:alloc_failed` from
  gullet pushback growth, 0 words), 2410.09116 (74 Errors → TooManyErrors, 16,027 → 3,634 words) — root-causing
  (`~/data/pk_agents/main/ab_66bundle/results_ref359{,b}.tsv`).
- **68f** (the three run-359 regressions, root-caused in agent_reports/2026-10-16_rc359_regressions.md): (A)
  `sect08.rs::record_dropped_environment_stores` read the name a dropped frontmatter environment's `\gdef` defines as
  the first control sequence after it, so rmmcart.cls:418's
  `\expandafter\gdef\expandafter\abstracts@ve\expandafter{\@grabstract}` recorded `\let\expandafter\@empty`, which the
  `\maketitle` deposit (sect05.rs `\lx@deposit@maketitle`) ran and its broken groups leaked: every later `\csname`
  construction failed (67o1's 373 Errors) and 67s's amsthm proof lock looped a tikz-cd into PushbackLimit (RUST-ONLY,
  since 56gm). The target is now read as TeX reads it (`dropped_store_target`: the `\expandafter` chain skipped, no name
  for a `\csname`-built one) and only a store is nulled (`nullable_store`: undefined, `\relax`, or a token-bodied macro
  — never a primitive, register, Rust-bodied kernel command or locked definition). (B) The binding dependency scan
  (`content.rs::require_dependencies_filtered`) loaded sn-jnl.cls:311's program.sty although 2201.11056's
  sn-jnl-setup.sty sets `\ver@program.sty` in `\AddToHook{class/sn-jnl/before}` (latex.ltx:18750 `\@ifl@aded` skips
  it): program's active `|` broke pgf and the gullet ran to `oom`. The scan now skips a candidate whose
  `\ver@<name>.<ext>` is set while our records say it is not loaded, with an Info naming it
  (`marked_loaded_unrecorded`; scan-only by the 2026-10-16 ruling, DIVERGENCES #605, KNOWN_PERL_ERRORS #780 — Perl
  Package.pm:2328-2330 tests only its records). (C) siunitx_sty.rs declared `S`/`s` at load only (Perl
  siunitx.sty.ltxml:1379-1399), so cellspace.sty:174-177's later `\newcolumntype{S}` won and 2410.09116's `{llSS}`
  tables gave 74 Errors (101, TooManyErrors, once 67u moved program's load); siunitx.sty:9348-9373 (and
  siunitx-v2.sty:5624-5625) re-declare them at begin-document, which the binding now does as the loaded release
  does: v3 `S` only (`table-column-type`, :9101), with its warning `Tabular column type "S" overwritten with siunitx
  definition.` also for an `S` declared before siunitx, a document's own `s` left alone; the `[=v2]` rollback `s` and
  `S`, silently (KNOWN_PERL_ERRORS #781; v2's cellspace-to-`C` move not ported). 2410.09116 stays a target
  although TL 2025 pdflatex aborts it at 100 errors (_main.tex:194: cellspace 2022 replaces `\@startpbox`, array 2025
  calls `\@startpbox@action`, array.sty:196, 388; clean with cellspace removed). Witnesses (67int4 → 68f, HTML,
  `--preload=ar5iv.sty`): 2104.09383 269 Errors + Fatal → 0 (0 → 3,932 words, recall 0 → 87.4 %; 67s1 269 + Fatal),
  2201.11056 Fatal `oom` → 5 Errors (13,817 words; 67s1 6 Errors, the extra lost words its undefined
  `\figurecaptionfont`), 2410.09116 102 Errors + Fatal → 0 (3,674 → 14,176 words; the 39 lost are the citation keys of
  missing citations, now resolved); the 67u sn-jnl witnesses 2010.12415, 2102.03176, 2104.02781 and the store-nulling
  manuals wkmgr1, wkmgr2, mcmthesis-demo, rpsample (resphilosophica) byte-identical. Repros
  sectioning-frontmatter/dropped_abstract_store_expandafter_gdef, loader/scan_skips_document_marked_loaded_package,
  package-bindings/siunitx_column_after_cellspace RED (67int4) → GREEN, package-bindings/siunitx_v3_document_s_column
  and siunitx_v2_rollback_columns RED (round 1) → GREEN; guards `perfect_kernel_batch68f` (8). A/B vs
  67int4-rel over 306 papers (100 pop67o, 100 siunitx of which 39 with a later column package, 100 with a shipped class
  renewing `{abstract}` or a `\ver@` marker, the 6 witnesses): only the three witnesses differ, 303 byte-identical;
  the sn-jnl net (30) unchanged.
- **68c** (binding gaps G3 B, author-file and symbol package bindings; bindgaps/g23_macros.tsv rows jmlr2e, spconf,
  dgruyter, icml2025/2026, biblatex, article, mathpazo, mathastext, wasysym, amsmath). The loader rule of round 1 (a
  shipped `.sty` that is another package than its TeX Live namesake loads raw: physics, proof, `myproof.sty`) is held
  on pk/68c-loader for a user ruling. Bindings, macros at their real file:line: jmlr2e borrows amsthm (a paper's
  `\let\proof\relax` + `\usepackage{amsthm}` gets amsthm's proof; 40 of 43 `{proof}` papers) and gains
  `{conjecture}`/`{axiom}`, `{example}` on its own counter (jmlr2e.sty 2022-10-17:383-396; 0907.5309, 1302.6584);
  spconf `\secondlinename`/`\thirdlinename`/`\nameplus`/`\nameplusplus`/`\secondname` append authors
  (`\lx@add@authors@append`), `\threeauthors`/`\fourauthors` (the 2006-07 copies :172-196; 1701.07847, 0708.3022);
  supertabular `\ST@newpage` (supertabular.sty:62-63, :515-543) and breakurl's character lists under `\ifpdf` false,
  after its xkeyval and ifpdf (breakurl.sty:35-38, :71-104, :186-187), for dgruyter.sty:1704-1706/:2052-2058
  (1503.04650, 1711.03341); article `\ps@headings`, `\ps@myheadings`, `\@makecaption` (article.cls:131-168, :480-489;
  KPE #760; 0711.0180, 1010.5412); mathpazo `\mathbold`, `\PazoBB` (mathpazo.sty:72, :101-103; KPE #761; 0807.0088);
  mathastext's user commands as no-ops with their arguments and their `\Mathastext…` names (:837-858, :964-968),
  `\MTDeclareVersion`/`\Mathastext[<name>]` declaring a math version and `\MTversion` selecting it with
  `\mathversion` (:2411-2420, :2585-2588, :2673-2697), `\Mathsf`/`\Mathtt` (:701-702; 1712.02869); wasysym 2.4
  `\wasyparagraph`/`\Paragraph` (§), `\wasyeuro`/`\euro` (`noeuro`), `\wasycmd`/`\applecmd`, `\longs`, `\wasyfamily`,
  `\textwasy` (wasysym.sty:47-55, :90-91, :262-270; KPE #762; 1001.4255, 1103.0853); biblatex `\RN`/`\Rn`
  (biblatex.sty:3948-3960), `\DeclareNolabel`/`\DeclareNoinit` (:13988), `\DeclareRefcontext` (:10428),
  `\BiblatexSplitbibDefernumbersWarningOff` (:16652), `.bbl` `\set`/`\inset`/`\annotation` (:8360-8378, :7943; a set
  entry prints as one more `<bibentry type="set">`, its members beside it; 2011.12274, 1707.02797, 1110.6541,
  2001.10285); icml `\Notice@String`, `\ICML@appearing`, `\ICML@preprint` — the paper's copy's where it ships one
  (`shipped_class::shipped_copy_text`; workshop lines 2606.28747, 2604.14572, 2507.02987, 2506.09099), else the
  version's as ICML distributed it (2015-2019, 2021-2026), empty for a version whose copies disagree (icml2020: PMLR
  108, 119 or empty) or an unknown one, so `accepted` never leaves the review notice nor invents a proceedings line —
  `\ificmlshowauthors`, `\ificml@noticeprinted`, the copy's (else the version's) accepted/preprint options;
  eso-pic/url/caption as the copy loads them, forloop and the affiliation counters as the copy (else 2026, resp.
  2017+) has them (icml2026.sty:124-177, :434, :504-518; 2601.22575, 2601.23143, 2601.04575); amsmath
  `\ifinalign@`/`\ifingather@`/`\ifst@rred`, `\intkern@` (amsmath.sty:1280-1290, :653; 1606.07128). Repros
  package-bindings/{jmlr2e_relaxed_proof_then_amsthm,jmlr2e_conjecture_and_axiom,spconf_second_line_names,
  spconf_threeauthors,dgruyter_patches_supertabular_and_breakurl,mathpazo_mathbold,mathastext_user_commands,
  wasysym_new_characters,biblatex_roman_and_config_commands,icml2026_notice_from_own_printaffiliations},
  class-bindings/article_page_styles_and_makecaption, ams-internals/amsmath_switches_and_intkern RED (67int6) → GREEN;
  package-bindings/{icml2024_accepted_notice_is_proceedings,icml2020_accepted_notice_unknown_is_empty,
  icml2025_customised_copy_notice,mathastext_version_selects_math_version} RED (round 1) → GREEN; guards
  `perfect_kernel_batch68c` (17). A/B vs 67int6 over 472 papers (372 row witnesses with the 43 loader-census papers, 100 of pop67o): Errors 1,332 →
  1,080, papers with Errors 362 → 188, Fatals 4 → 4, none more; per row, Errors 67int6 → 68c: jmlr2e 29 → 21, spconf
  28 → 14, dgruyter 47 → 19, article 48 → 27, mathpazo 28 → 2, mathastext 29 → 2, wasysym 27 → 2, biblatex 44 → 5 and
  19 → 2, icml2026 50 → 11, icml2025 30 → 27 (2505.21317's copy loads forloop), amsmath 89 → 84; physics (211),
  proof (21) and the census papers at 67int6's, the held rule's. Every word lost is former error text, `ſ`/`§`/`€`
  in place of the command names, or jmlr2e's examples numbered on their own counter as the copies number them
  (2311.05501, 2412.10837, 2411.09644); the 100 pop67o identical. Binding nets vs 67int6 (jmlr2e, spconf, dgruyter,
  mathpazo, mathastext, wasysym, biblatex, amsmath, article, breakurl, icml2023-2025): no document worse. RED: macro-state/counter_alias_macro_to_register
  (`\newcommand*{\c@thmlemma}{\c@theorem}`, math0701082's own theorem.sty, which the paper reaches only under the
  held loader rule). Residuals: wasysym `\textwasy`/`\wasyfamily` text is the raw letters, not the wasy glyphs;
  jmlr2e's claimed `{proof}` is amsthm's italic "Proof." ∎ where jmlr2e prints a bold "Proof" ■ (before 68c too);
  a math version mathastext declares changes no font (sect13.rs `\mathversion`), and `\MTversion{normal}` keeps a
  bold (KPE #687). Not this payload's: the jmlr2e `{assumption}` (84; four numbering forms in the copies),
  `{thm}`/`{cor}`/…, spconf `\names`/`\nametwo`/`\myname` (several arities), aas_macros journal additions (`\cqg`,
  `\prx`, …), the icml2025 long tail (`\printAffiliationsAndNoticeArxiv`, …) — each a copy's own addition, the pending
  user decision on a shipped copy's extra definitions (the ICML notice strings read from a copy are definitions the
  binding makes anyway, the copy's text of them); amsmath's public commands the witnesses lack (`\eqref`, `\dfrac`,
  `{align*}`: amsmath is not loaded there — ncc's, breqn's or the class's own dependency, not the binding); stix and
  the amsmath `\collect@body`/`\make@display@tag` internals not started.
- **Run 357** (E/F rerun of run 353 on 1a95291ef6: 67n, 67o; COMPLETE 2026-10-15 18:15:14Z in 4 h 05 min under host
  load): E 89,338 → 89,367, F 4,145 → 4,091 (success 96.80 % → 96.80 %). Transitions vs run 353: E→OK 4, E→F 43, F→E 76,
  F→OK 21. E→F: `timeout` 33 (contention: two writers and their builds shared the host), `TooManyErrors` 6, `oom` 2,
  SIGABRT 1, Document 1; all 10 non-timeout ones convert locally on the same commit with no Fatal (Errors 0-90). The
  fleet logs of the TooManyErrors papers (hep-th9703095, 1607.00237) are 68 and 101 × `Error:post:xpath XPath
  evaluation failed (no libxml2 error detail)` plus EPS "every converter failed": a memory-pressure signature in a
  fleet child. Candidate (open): treat a detail-less libxml2 XPath failure like an allocation failure (retryable,
  67o).
- **Run 353** (E/F rerun of run 351 on ba8dff38f3: 67d, 67j, 67l, 67m; COMPLETE 2026-10-15 05:52:14Z in 3 h 27 min,
  against 5 h 27 min for run 351 with the 67m deadlock): E 93,718 → 89,338, F 4,171 → 4,145 (success 96.65 % → 96.80 %).
  Transitions vs run 351: E→OK 4,360, F→OK 46, F→E 71, E→F 91 (`oom:alloc_failed` 59, `timeout:wallclock` 11,
  `TooManyErrors` 14, `crash:SIGABRT` 3, `Timeout:PushbackLimit` 2, Recursion 1, Convert 1).
- **67o** (fleet, user ruling 2026-10-15 "both"): an allocation failure in a grown harness child is retried, and children
  recycle earlier. 53 of run 353's 59 E→F `oom:alloc_failed` papers convert cleanly in a fresh process under the same
  5632 MB `RLIMIT_AS` (hep-ph0006359, math0111204, math0305067: 3.1-3.5 GB peak RSS); the allocator's reservations had
  grown the long-lived child's address space, and the recycle fired only within 1.5 GiB of the cap. Before 67m these
  children deadlocked and the harness's kill re-leased the paper; 67m made the failure clean and final. Now
  `begin_task` writes the child's address space as the paper starts (`space <KiB>`), `exit_report::retryable` tags an
  `oom:alloc_failed` retryable when that start was past a fresh worker's `FRESH_ADDRESS_SPACE_KB` (2.7 GB; an unknown
  start stays final), and `recycle_after_task` also recycles past `RECYCLE_ADDRESS_SPACE_KB` (3 GiB). Raising the cap
  instead is ruled out (worker.env: 8192 once spiked a sweep to 207 GB). Guards
  `cortex_worker_exit_reports::{an_allocation_failing_in_a_grown_worker_is_retryable,
  an_allocation_past_rlimit_as_is_reported_as_fatal_oom}` (`CORTEX_WORKER_TEST_GROW_MB` reserves address space in a
  debug child), `exit_report` unit tests. Open: measure the respawn cost of the 3 GiB recycle on the next rerun (spawn
  count and wall time vs run 353).
- **Run 351** (E/F rerun of run 349 on 4f131f2b84 with the fleet's added TeX Live collections; COMPLETE 2026-10-15
  01:41:54Z): E 100,336 → 93,718, F 4,418 → 4,171 (success 96.42 % → 96.65 % of 2,922,325). Transitions vs run 349:
  E→OK 6,658, F→OK 207, F→E 153, E→F 113 (`oom:alloc_failed` 56, `timeout:wallclock` 15, `ParamSpec:Expected` 14,
  `TooManyErrors` 18, `Timeout:PushbackLimit` 6, `crash:SIGABRT` 3). The OOM/timeout share is the run's slow tail: the
  67m deadlock parked OOM children for the harness's 360 s kill and their papers were re-leased. The 14 `ParamSpec`
  are a host-TL exposure, not a code change: eplain.tex (texlive-formats-extra, installed 2026-10-14) borrows `\cite`
  as scratch — btxmac `\edef\cite{\the\catcode`@}` then `\let\@oldatcatcode=\cite` (eplain.tex:346-348) — which our
  locked `\cite` ignores ("Ignoring redefinition of \cite"), so `\catcode`@ = \@oldatcatcode` (:666) misses its
  number and `\def\@Fornoop#1\@@#2#3` (:721) is read as `\def\@` (math0308041, math0612670, 0705.2148, 0802.3843,
  1207.1752, 1401.4197; fixed by 67n below). Per-paper cost (perf-measure 2026-10-15, CPU seconds: `perf_event_paranoid=4`
  on this host blocks `instructions:u`): no regression since the 2026-10-07 pass — 39 random 2609 papers HEAD/BASE
  median 1.000, 20 heavy fleet Fatals 0.989; 8 of the 9 fleet OOM papers peak under 3.9 GB and 8 of the 9 fleet
  timeouts finish in 5-140 s standalone, so the tail deaths are fleet limits and contention. Open from it: 1902.02884
  times out since 65n (math-group recovery at `partie_geometrique.tex:6430` `\begin{remark}`: "Missing $ inserted",
  nine "Missing \endgroup inserted", then a stall; stomach.rs `insert_dollar_sign`/`off_save_closer`, tex_math.rs
  `end_math_or_defer`); math0701165 costs +30 % CPU and +1.13 GB (4.87 GB peak) since 67d, which now converts its
  diagrams (3,152 → 3,196 formulae). Ops: the fleet's journald drops ~1M lines per 30 s (every child streams its whole
  log), and `/opt/cortex-scratch` holds 565,483 leaked `cortex_task*` input trees (434 GB, since 2026-07) awaiting a
  user-approved cleanup.
- **67n** (run 351's 14 E→F `ParamSpec:Expected`; user ruling 2026-10-15 "extend D1 to robust"): a robust locked
  binding owns its calls by what it wraps. The kernel `\cite` (sect11.rs, `robust => true, locked => true`, OD #88)
  is the `\protect \cite␣` wrapper, whose own parameter list is empty, so `state::owns_calls` said no and eplain's
  btxmac scratch `\edef\cite{\the\catcode`@}` (eplain.tex:346) was refused ("Ignoring redefinition of \cite"): its
  `\let\@oldatcatcode=\cite` (:348) copied the binding, `\catcode`@ = \@oldatcatcode` (:666) read no number, `@`
  became catcode 0 and `\def\@Fornoop#1\@@#2#3{}` (:721) raised Fatal `ParamSpec`. All six witnesses take the LaTeX
  route before `\input eplain` (AMS-TeX's `\documentstyle{amsppt}`; eplain's own `\usepackage` for 1207.1752 and
  1401.4197), which loads the locked `\cite`; Perl's `\cite` is not locked (math0308041: Perl 2 Errors, 0 Fatal).
  `owns_calls` now judges a robust wrapper (`robust_wrapper_body`) by its inner `\cs␣`'s meaning and `call_openers`
  takes the `[`/`*` openers from it, so the scratch value is the meaning for `\let` and every use without an
  argument, while `\cite{x}`/`\cite[p.~3]{x}` after eplain's own `\def\cite{\@getoptionalarg\@cite}` (:445) stay the
  binding's citations (OD #500 amended, OD #88 note). Witnesses (67m1 → 67n, Errors/Fatals): math0308041 1/1 → 0/0,
  math0612670 1/1 → 0/0, 0802.3843 1/1 → 1/0 (`frontmatter:merged_creators`), 1207.1752 1/1 → 7/0, 1401.4197 1/1 → 4/0,
  0705.2148 1/1 → 0/1 (side finding (a)). Repros macro-state/{m5_robust_cite_edef_scratch_eplain,
  m5_robust_cite_calls_after_value}, m5_robust_cite_value_in_group RED → GREEN; guards `perfect_kernel_batch67n`.
  A/B vs 67m1 (the M5 census's 95 `\def`/`\renewcommand`/`\let\cite` papers + 30 random natbib-free papers citing
  with `\cite[…]{…}`): 125/125 byte-identical XML (606 Errors, 3 Fatal timeouts, 8,727 `<cite>` both sides); 4 take
  the new path, parameterless forwarders whose every use is a call (`\renewcommand\cite{\citep}` 2306.00600,
  `{\citeasnoun}` 2006.07780/2006.13934, a cite.sty-style `\def\cite{…\@ifnextchar[…}` 1806.11139).
  Side findings, not chased: (a) 0705.2148 now ends in a Rust-only `Fatal:Timeout:PushbackLimit` while reading
  amssym.tex, which needs eplain, amstex, amssym and `\documentstyle` together (Perl 1 Error at eplain.tex:50, pdftex
  clean): RED amstex/eplain_amstex_amssym_pushback_limit. (b) `has_latex_cue` (latexml_oxide/src/converter.rs:136-146)
  counts `\usepackage`/`\RequirePackage`/`\include` as LaTeX cues, while the test `state::set_plain_tex_document`
  documents (state.rs, arXiv AutoTeX's) names only `\documentclass`, `\documentstyle` and `\begin{document}`; eplain
  provides `\usepackage` in plain TeX, so the plain eplain papers 1207.1752 and 1401.4197 take the LaTeX route and keep
  their `\usepkg@options@…` Errors (routing unchanged here; needs a decision).
- **Runs 354/355** (full sandbox reruns of 2605/2606 on 1a95291ef6 = 67n + 67o, the baseline before the rawclasses
  lever). 2605 (run 354) vs its 2026-10-13 snapshot: E 1,443 → 1,332, F 32 → 29; 10 papers worse: 5 fleet-contention
  `timeout:wallclock` Fatals (2605.04030, 2605.15781, 2605.20426, 2605.28027, 2605.14200: 40-72 s locally against the
  fleet's 180 s, with local measurement runs sharing the unpinned host), and two code classes, fixed as 67q. 2606
  (run 355) vs 2026-10-13: E 1,402 → 1,298, F 42 → 27; 2 worse: 2606.12944 (`\path`, 67q) and 2606.01369 (a
  wall-clock timeout).
- **67q** (run 354 regressions): (1) the arXiv hyperref fallback (67a, OXIDIZED_DESIGN #588) supplied `\href` but not
  url.sty, which the injected hyperref loads; an elsarticle-num `.bbl` defines `\path` only when `\href` is undefined, so
  its `{\path{doi:…}}` entries were undefined (2605.07373, 2605.08166, 2605.12003). The fallback now also loads url.sty
  provide-only (`\path`, `\urlstyle`, `\urldef`, `\Url` where still undefined). Guard
  `perfect_kernel_batch67a::url_path_kept_for_a_bbl_after_supplied_href`. (2) mathpazo.sty:63/67 declares the `upright`
  symbol font, so `\symupright` exists; flexisym's mathpazo.sym:49 reads it under `\usepackage[mathpazo]{flexisym}`
  (breqn), read since 66i. Our binding, like Perl's, declared none (2605.15826, 2605.30950). Guard
  `perfect_kernel_batch67q::mathpazo_declares_the_upright_symbol_font`. Residual (not changed): our kernel's
  `\DeclareSymbolFont` makes `\sym<name>` a `\let` to `\z@` (sect08.rs), where latex.ltx's `\new@mathgroup` allocates
  a `\chardef` family number (`\the\symupright`: 0.0pt vs pdflatex's 4). (3) From the rawclasses regression study
  (agent_reports/2026-10-15_rc_rawcls_regressions.md, R3): hyperref.sty:7302 keeps the original `\maketitle` as
  `\HyOrg@maketitle`; pittetd.cls's `\AtBeginDocument{\let\maketitle\HyOrg@maketitle}` left `\maketitle` undefined
  (1509.06375: 101 errors then Fatal). Our binding (and Perl's) saved none; now `\let`. Guard
  `perfect_kernel_batch67q::hyperref_keeps_the_original_maketitle`.
- **67r** (rawclasses study R1): in maths an old font command switches the math alphabet to the end of the group, as
  latex.ltx's `\@fontswitch` (14312-14321) does by letting `\math@bgroup` to `\relax`: `{\rm LP-HF}` is all upright,
  `{\cal H}` a calligraphic H. Ours and Perl's (latex_constructs.pool.ltxml:5200, :5315; KNOWN_PERL_ERRORS #685) ran the
  alphabet with its own argument (`\mathrm{L}P-HF`, `\mathcal{}H`, "Missing sub/superscript argument", "Extra
  \right"). `\lx@math@alphabet@switch` merges the font the alphabet's binding carries (`Definition::declared_font`,
  kept on primitives and constructors); another `#2` is TeX's `#2\relax` (OXIDIZED_DESIGN_DIVERGENCES #593). Under the
  rawclasses lever the `tex=` with an empty alphabet argument of 10 ptptex/jpsj2/paper witnesses go 3,214 → 0 (gr-qc0605108
  919 → 0, cond-mat0510381 296 → 0), errors and words unchanged. A/B without the lever (67o1; 401 papers, 392 converted:
  sources calling `\DeclareOldFontCommand` or amsart/revtex/aastex with `{\rm …}` in maths, and 100 of pop67o): 0 gain
  Errors or Fatals, cond-mat0205657 loses its one Error, 33 change `tex=` (`\mathrm{o}ut` → `{\rm out}`), and the 24
  word drops are maths tokens merging (`g`·`a`·`s` → `gas`; every maths character and text word kept). Guards
  `perfect_kernel_batch67r::{old_font_command_switches_the_math_alphabet,
  old_font_command_switches_package_and_declared_alphabets}`. Round 2 (review): an alphabet reached through a
  parameterless macro or `\protect` is a switch too (`\newcommand\Bbbx{\mathbb}` as `#3`: round 1 gave
  `tex="{\mathbb{}RZ}"` with both letters italic; guard `old_font_command_alphabet_through_a_macro`); the `{\cal AB}`
  `tex=` round-trip is a recorded residual in OD #593; round-2 A/B on the same 401 papers (392 converted): every
  output byte-equal to round 1's, so vs 67o1 still 0 gain Errors or Fatals; the 10 rawclasses witnesses byte-equal
  too. RED side finding, not fixed: `\unboldmath` and
  `\mathversion{normal}` do not undo `\boldmath` (both engines; `Font::merge` of `forcebold: false` is a no-op while the
  bold math font forces its series), KNOWN_PERL_ERRORS #687, repro fonts-nfss/boldmath_undone_by_unboldmath_and_group.
- **67p** (raw TeX Live classes under the coming rawclasses lever, study 2026-10-15_rawclasses_lever.md §2): two
  binding gaps. (1) xkeyval: the `OptionalAngle` reader returned the same empty value for `<>` and for no `<…>`, so
  `\ProcessOptionsX[sap]<>` (sapthesis.cls) and `\DeclareOptionX[sap]<>` took the default family
  `\@currname.\@currext` and the class's `\DeclareOptionX*` rejected `english`/`oneside`/`binding=…`; an absent
  `<…>` is now `None` and `<>` the empty family (header `<prefix>@`, xkeyval.tex:83-87, as Perl's front end :330
  keeps it), and `\key@ifundefined` searches the empty family too (xkvutils.tex:110-126, :228-237). Perl's
  `KeyVals` drops the empty family (KPE #212, extended). (2) theorem.sty (theorem_sty.rs): `\theorem@bodyfont`,
  `\theorem@style`, `\theorem@headerfont` defined as theorem.sty:62-81 does; `\theoremstyle` (grouped),
  `\theorembodyfont`, `\theoremheaderfont` (once only) are theorem.sty's; the binding's `\newtheorem` hands
  `define_new_theorem` the theorem.sty sequence `\th@<style> \the\theorem@bodyfont \@thm{<counter>}{<name>}` as the
  theorem's body font (a binding style's font in place of its `\th@`, unless a class redefined it; the current style
  font before a class style that sets its font in `\@begintheorem`), and `\@thm` ends it. theorem.sty does not load
  amsthm, so the binding leaves the amsthm it borrows unloaded (unless the paper had loaded it) and a paper's later
  `\usepackage{amsthm}` takes `\newtheorem` (amsthm.sty:77; amsthm_sty.rs `\lx@amsthm@newtheorem`) and the styles
  back, as the ifacconf template needs (KPE #686). Round 2 (review): the no-op that ends the sequence is a
  `\let\@thm\@gobbletwo` local to the theorem's group (round 1's global no-op `\@thm` silently dropped the heads and
  counter steps of raw `\th@plain\@thm{claim}{Claim}` environments; guard `theorem_raw_thm_keeps_heads`); the borrow
  is the kernel's `require_package_borrowed` on `state::mark_unloaded` (factored out of `end_provide_only`), used by
  ntheorem's binding too (ntheorem.sty loads only ifthen); the definition-time style limits are in KPE #686. Round-2
  A/B vs 67o1-rel, the 200 above + 30 ntheorem users (from a 29.7k-zip scan; 8 also load amsthm or theorem.sty), 218
  converted: Errors 1,324 → 1,322, no paper worse, the same six changes, and byte-equal tallies with round 1 on all
  190 shared papers; the 30 ntheorem papers unchanged. Open (a ruling): a borrowed binding's definitions block the
  ones LaTeX would take, as the real package never defines them — a document's `proof` in the
  `\@ifpackageloaded{amsthm}` false branch, and under the rawclasses lever cc.cls's ccproof.sty:56
  `\newenvironment{proof}` ("Ignoring redefinition"), so 1507.02955's proofs have no □ (base the same; round 1 had
  them only because its global no-op `\@thm` let ccqed.sty:74's wrapper leak `\@qedtrue`). Perl golden tests/theorem/theorem.xml: the
  last theorem is italic, as pdflatex sets it (CMTI10). Witnesses, rawclasses lever (`--path=…/rawcls/rhai
  --preload=lxrawclasses.sty`), Errors 67o1 → 67p: 2503.03277 2 → 0, 2403.20315 1 → 0, 1507.02955 4 → 1 (the
  `\currentaddress` left; creators 3 both, theorems 38 now slanted under small-caps heads as thcc.sty sets them);
  default profile unchanged for the three. A/B vs 67o1-rel, 190 papers (150 pop67o xkeyval/theorem.sty users + 50
  grepped; 10 no main): Errors 1,316 → 1,314, no paper worse; 6 change, each checked against pdflatex: 2211.14033 and
  2512.06733 (ifacconf + amsthm: `\theoremstyle` undefined / `{proof}` undefined gone, amsthm heads), 1911.07501 and
  1911.10645 (ifacconf: italic heads, upright bodies, SFTI/SFRM in the PDF), 1911.07591 (fundam's `\th@plain` no
  longer sets the title and abstract italic), math0505657 (`\theorembodyfont{\slshape}` survives
  `\theoremstyle{plain}`, SFSL); 2603.11260/2603.11265 take amsthm's heads (with the period) and its ∎. Repros
  package-bindings/{xkeyval_empty_family_processoptionsx (+ xkvemptyfam.cls), theorem_bodyfont_internals,
  theorem_user_style_and_headerfont, theorem_then_amsthm_takes_over} RED → GREEN; guards `perfect_kernel_batch67p`.
  RED side finding, not fixed (kernel, Perl shares): a `\newtheorem`'s shared counter named by a macro
  (ccthm.sty `[\cc@thmcnter]`; latex.ltx `\@othm` expands it, `define_new_theorem` takes `ToString`): 1507.02955's
  38 theorems unnumbered under the lever, repro package-bindings/newtheorem_shared_counter_named_by_a_macro.
- **67s** (rawclasses regression study R2 and R5, agent_reports/2026-10-15_rc_rawcls_regressions.md): theorem and
  proof machinery under raw classes. (1) smfart.cls:1194, smfbook.cls:1281 and ijmart.cls:336 `\renewenvironment{proof}`
  over a `\trivlist` replaced amsthm's `<proof>` (86 papers under the lever: smfart 72, ijmart 11, smfbook 3); amsthm_sty.rs
  now locks `\proof`/`\endproof` against the class and package files, the document's own `{proof}` stays its own
  (OXIDIZED_DESIGN #596, KNOWN_PERL_ERRORS #690). (2) `\newtheorem{theo}[\smf@thm]{…}` (smfthm.sty:52-56, ccthm.sty
  `[\cc@thmcnter]`): `define_new_theorem` took the counter's macro name literally, so every theorem raised
  `\the\smf@thm` undefined; it now expands the name as latex.ltx's `\@othm` does through `\csname` (KNOWN_PERL_ERRORS
  #691; 21 papers in both arms). Witnesses 1711.11547, 1902.02049, 2108.12000 (smfart), 1507.02955 (cc). Repros
  package-bindings/amsthm_proof_kept_against_a_class_renewenvironment, newtheorem_shared_counter_named_by_a_macro (67p
  RED → GREEN); guards `perfect_kernel_batch67s`. apxproof, which wraps the binding's proof to move a `theoremrep`'s
  proof to the appendix, had its binding yield the lock while it loads (2502.18805; since 67ps the lock lets its
  wrappers through). Witnesses under the lever (67o1 →
  67s, Errors / `<proof>`): 1711.11547 1/0 → 0/14, 1902.02049 1/0 → 0/30, 2108.12000 0/0 → 0/24, 2412.15231 (smfbook)
  1/0 → 0/256, 2306.11126 (smfbook) 0/0 → 0/44, ijmart 1001.1175 0 → 19, 1107.1667 0 → 8, 2207.06393 0 → 9 proofs;
  1507.02955 (cc) keeps its 4 theorem.sty Errors (67p) and loses 76 counter warnings. Word losses are generated text
  only: the `\the\smf@thm` error strings, the trivlist items' "item" autoref tags, the class's " — " after the proof
  head. A/B without the lever vs 67o1 (400 papers: 300 amsthm `{proof}`/`\newtheorem[..]` users, many shipping a
  `.sty`/`.cls` that redefines `proof`, + 100 from pop67o; plus 17 apxproof users): 0 worse; 9 gain proofs (1202.3460,
  1210.2844, 1609.05867, 2309.14926, math0003189, math0007097, math0203159, math0302254, math0412216: a shipped class's
  `\renewenvironment{proof}`; 2503.05183, first counted here, keeps GT.cls's trivlist: its copy is defined in
  `\AtBeginDocument`, OXIDIZED_DESIGN #596), 2309.14926 (shipped smfart) 1 → 0 Errors; the apxproof users are
  byte-identical. Residual: the proof head's font and punctuation are the binding's, not the class's (isamms.cls's
  small caps, smfart's " — ").
- **67u** (binding gaps G1, run 353's undefined-only papers whose missing commands come from a package the class, or a
  package it loads, requires; bindgaps/g1_deps.tsv): bindings load what the real file loads, in its order and under its
  options. Class bindings (dependency, real file:line, witnesses): sn-jnl → program (sn-jnl.cls:311/621 of the older
  copies, through the class scan; 2010.12415, 2102.03176, 2104.02781; its `\(`…`\)` programbox kept the kernel's inline
  maths under the class, OXIDIZED_DESIGN #597, approved by the main session 2026-10-15; the proof mark is the class's
  begin-document `\opensquare` □, sn-jnl.cls:1571-1575 of 1907.11468's copy, over program's `\blacksquare` — round 2,
  1907.11468, 2210.16458; the same hook's `\let\proof\spiproof`, an amsthm copy over program.sty:928's `\def\proof`, puts
  the binding's `<ltx:proof>` back — integration with 67ps, whose lock lets program's own-head `\proof` through); bmvc2k → subfigure/subcaption/subfig/caption as the
  paper's copy names them (bmvc2k.cls:121; 1507.05726, 1605.07146; shipped_class.rs `SUBFIGURE_PACKAGES`,
  `require_shipped_packages`); ieeeaccess → color, spotcolor unless `jtehm` (ieeeaccess.cls:121-131; 1901.05553,
  1712.00216); agujournal2019 → xcolor, url, lineno, indentfirst, ragged2e (:91-112; 1803.00457, 1902.01137);
  WileyMSP-template → graphicx, gensymb (:5-10; 2003.01904); oup-authoring-template → multirow, url, mathrsfs
  (TL 2025's copy :2426-2430, 2102.02669's :2391-2395; 2102.02669); cas-sc/cas-dc → cas-common's `\MSC`/`\JEL`/`\PACS`/`\orcidauthor`/`\stmAuthorSetup`/
  `\FullWidth` (cas-common.sty:415-423, 774, 1599-1610, 2067; `\MSC` 1909.01564, 2407.03849; `\orcidauthor`
  2104.12510, 2304.13458, 2407.18715, each after its own `\author` — 2106.08963 swaps the two arguments, the author's
  error, so its link reads orcid.org/Andrew S. Nencka); ws-rv9x6 → makeidx, rotating, url
  from the copy (ws-rv9x6.cls:1524-1528; 0706.1148); revtex4-1/4-2 → lineno under the `linenumbers` option, at the
  document's start (revtex4-1.cls:6186-6193, revtex4-2.cls:6207-6214; 1209.2541, 2003.07846; KNOWN_PERL_ERRORS #697);
  quantumarticle → the `amsfonts`/`amssymb`/`amsmath` options' packages, geometry, caption (quantumarticle.cls:155-168,
  282-284, 1508-1512 — caption at the class's `\AtEndPreamble`, with the class's etoolbox (:27), round 2;
  1802.10344, 1612.06828, 1910.03969; #698); aa → answers (verbatim's `{comment}`) when the copy
  loads it (aa.cls:249 of the v8 copies; 1501.03426; #699); atlasdoc → its font option's packages, newtxtext+newtxmath by
  default (atlasdoc.cls:52-54, 516-546; 2207.12246); ceurart → colortbl, dcolumn, stfloats (:88; 2212.08130); ejpecp →
  graphicx, mathtools, microtype, latexsym, dsfont unless `NODS`, geometry (ejpecp.cls:81-106; 1607.07914, 1111.6458);
  WileyNJD-v2 `APA` → NJDapacite, which is apacite.sty, as the apacite binding with `natbibapa` (WileyNJD-v2.cls:63,
  2722-2723; 1811.00964); jmlr → graphicx, url, xcolor[x11names] (jmlr.cls:150-154; 1902.03035); sigma → latexsym, amscd,
  epsfig, graphics, ifthen, color (sigma.cls:20-28; 0902.2977). Package chains the grep attributed to the class:
  amsart/article/IEEEtran → graphics through lscape.sty:35 and psfrag.sty:35 (129 papers; 0710.2354, 0801.1195; #692),
  → color through showkeys `[color]` (showkeys.sty:51-56; 73; 1010.3777; #692), → thm-patch through thmtools
  (`\addtotheorem…hook`, run by `define_new_theorem` inside the theorem and, as thm-patch.sty:178-217 wraps amsthm's
  `proof` when it exists, the `[proof]` hooks by the binding's proof; thmtools marks `thm-patch.sty_loaded`, so a later
  `\usepackage{thm-patch}` reads nothing over it — round 2; 69; 1710.06939; #695), → `\varlim@`
  (amsopn.sty:92-97, mis-attributed to amstex; 23; 1608.08273; #696); article → mhsetup through mathtools
  (mathtools.sty:37, the binding's TODO; 33; 1202.0045; #693); acmart → centernot through newtxmath (newtxmath.sty:36;
  21; 2002.09451; #694, a centernot binding: `\not`). The xcolor exclusion comments in agujournal2019_cls.rs and
  jmlr_cls.rs were stale (a repeat `[table]` load applies its options since 2605.00310) and are replaced. With program
  loaded two interactions surfaced and are fixed: adjustbox's trimclip, finding no driver for our empty `\Gin@driver`,
  reads pgf at once (pdflatex: tc-pdftex.def, no pgf), and pgf broke on program's active `|` (2202.10115, 2507.09618 +6
  Errors) — adjustbox_sty.rs read its file with `|` other (removed in 68h, which names the driver); program.sty:1009's `\index` (`\@sanitize` with nothing
  to end the group here) turned the rest of 2306.09847 into text and lost its bibliography — program_sty.rs keeps the
  kernel's `\index`/`\makeindex`. `shipped_class_text` now also reads a `{filecontents}` copy (the VFS), as the
  dependency scan does. Skipped, author's missing `\usepackage` (pdflatex errs too): mn2e → natbib (mn2e.cls:1201-1203
  loads it only under `usenatbib`, which none of the 128 witnesses passes (many have it commented out); `\citeauthoryear`
  then errs at each `\cite` in pdflatex: 0706.0055), acmart → colortbl (no witness of 47 loads colortbl or passes
  `table`: 1807.07984), article → forloop/multirow/color with no chain, the 24 revtex papers calling lineno without the
  option. Discounted (TL-2025 version limit): mnras → atbegshi (63; 1605.02911, 1609.06406: the `\Hy@FixNotFirstPage`
  hack relies on pre-2020 hyperref loading atbegshi; TL 2025 hyperref does not, pdflatex with the paper's mnras.cls
  leaves `\AtBeginShipout` undefined). Repros class-bindings/{snjnl_class_copy_loads_program,
  bmvc2k_class_copy_loads_subfigure, ieeeaccess_loads_color_and_spotcolor, agujournal2019_loads_lineno_ragged2e_xcolor,
  wileymsp_loads_gensymb, oup_loads_multirow, cas_sc_classification_lines, ws_rv9x6_class_copy_loads_makeidx,
  revtex_linenumbers_option_loads_lineno, quantumarticle_options_load_ams_geometry_caption, aa_class_copy_loads_answers,
  atlasdoc_default_fonts_load_newtxmath, ceurart_loads_colortbl, ejpecp_loads_mathtools_dsfont,
  wileynjd_apa_option_loads_apacite, jmlr_loads_xcolor, sigma_loads_amscd} and stub-dependencies/{lscape_psfrag_load_graphics,
  showkeys_color_option_loads_color, mathtools_loads_mhsetup, newtxmath_loads_centernot, thmtools_theorem_hooks_run,
  amsopn_varlim_builder} RED → GREEN; guards `perfect_kernel_batch67u`. A/B: final candidate vs the parent (67s1-rel; 67o1-rel where the r1 candidate matched it), 606 papers (up to 12
  witnesses per row, all 138 sn-jnl → program, 100 from pop67o): Errors 1,278 → 432, Fatal 3 → 2, error-free 65 → 496;
  pop67o 343 → 242. Word losses, over the 539 papers both arms converted: 366 lose some words, 82 more than 15, 20 at
  least 1 % of their words; each is error-recovery text or text pdflatex hides: the unloaded commands' names and
  arguments (`\definecolor cvprbluergb…`, `\cellcolor color1!33`, `\scalebox .4 \includegraphics krebes.eps`,
  WileyNJD's raw `\APACyear`/`\BBA`, the APA title printed twice; bibitems unchanged, every reference text kept) and
  aa's `{comment}` bodies — three aa papers lose a `<tabular>` that sits inside `\begin{comment}` (1911.00393,
  1708.03873, 1612.05281) — plus the sn-jnl proof mark (∎ in 257 proofs of the 138 sn-jnl papers; □ since round 2,
  as pdflatex prints). The review diffed 13 more (WileyNJD 2009.09092, 2106.13685, 2101.02028, 2012.15339; aa
  1708.03873, 1704.08487, 1505.02291, 1509.09223, 1911.00393, 2605.31060; ieeeaccess 2304.03771, 2205.04113; amsart
  1406.4669) and scanned every paper losing more than 15: no author text lost. 1709.10492 timed out once under load (170 s) and converts in 112 s alone. Binding nets
  (binding_nets run_net.sh, label 67u) for the 23 bindings touched vs 67s1: Errors fall in 19 (ieeeaccess 40 → 0,
  mathtools 55 → 13, ejpecp 53 → 26, atlasdoc 19 → 3), none worse after the adjustbox/program fixes (the sn-jnl net's
  2010.12415 2 → 7 at the first candidate, 0 now). Round-2 repros class-bindings/{snjnl_class_open_square_qed,
  quantumarticle_caption_at_end_preamble}, stub-dependencies/thmtools_then_thm_patch_proof_hooks (RED on 67u r1).
- **Residual of 67u — our empty `\Gin@driver` sends trimclip to its PGF fall-back** (SHARED, KNOWN_PERL_ERRORS #797;
  FIXED in 68h: graphics_sty.rs gives the driver file graphics.cfg would, and the `|` workaround is gone): trimclip.sty:24 `\def\tc@driver{tc-\Gin@driver}` names no file here, so :252-258 input tc-pgf.def and pgf at
  adjustbox's load, where pdflatex reads tc-pdftex.def and no pgf. 67u's adjustbox_sty.rs reads the file with `|` other
  (program.sty's active `|` broke pgf: 2202.10115, 2507.09618), a workaround; the root is the driver name graphics
  leaves empty.
- **67ps** (the integrated 67p + 67s review, agent_reports/2026-10-15_67ps_integrated_review.md, NO-SHIP finding 1 and
  the borrowed-definitions rule): (1) the borrow (`require_package_borrowed`, theorem.sty's and ntheorem.sty's bindings)
  sets no proof lock — 67s had set it during the borrow, so a theorem.sty-based class's `\def\proof` was refused — and
  what it defines carries `DefinitionOrigin::Borrowed`, which yields to any later definition, the document's or any
  file's (`is_definable_latex`; `fallback_meanings` now records the origin), as in LaTeX, where amsthm is not loaded;
  the files it loaded (amsgen) are unloaded with it, `mark_unloaded` also clears `_raw_loaded` and `\opt@<file>`, and
  amsthm's real load after a borrow keeps a proof put in place meanwhile and only provides its old-version font commands
  (KNOWN_PERL_ERRORS #686). (2) Under a real amsthm the proof lock holds only against a copy of amsthm's interface (an
  optional argument with a non-empty default first, `state::files_lock_holds`/`takes_optional_first`), titles the kept
  proof from that default (`record_redefined_default`: thuthesis's `\thu@proof@name`), lets a redefinition with its own
  head through (head text and end mark kept), and lapses once the protected `\@proof` is no longer in force (amsart, the
  document's `\let\proof\undefined`, then a package's copy `[1][Nachweis]`); 0705.0545 (the same `\let`, then nccthm's
  `\newcommand*{\proof}[1][]`: 1 Error → 0) goes through on the copy rule, its default being empty; apxproof needs no
  lift of the lock any more (OXIDIZED_DESIGN #596). Round 4 (re-review agent_reports/2026-10-15_67ps_r3_review.md): a
  robust copy (`\DeclareRobustCommand\proof[1][\proofname]`) is judged by its inner `\proof␣`; a borrowing binding
  claims the borrowed names its real package defines too (`state::claim_borrowed_meanings`: ntheorem's `[amsthm]`
  proof, which a later package's `\newenvironment{proof}` had replaced; theorem.sty's skips; KNOWN_PERL_ERRORS #686);
  `install_definition` probes the copies lock only where a lock against the files is set; and the test harness fails
  a guard whose fixture a binding shadows through the versioned-name fallback without reading it
  (`util::test::assert_fixtures_not_shadowed`; `perfect_kernel_batch67s::package_redefined_proof_keeps_the_binding`'s
  `myproof.sty` had loaded the proof.sty binding, so it is `relaxproofpkg.sty` now). Repros package-bindings/{theorem_borrowed_proof_yields_to_a_class_def,
  theorem_borrowed_proof_yields_to_a_package_newenvironment, amsthm_proof_lock_lets_a_hardcoded_head_through,
  amsthm_proof_copy_titled_by_its_default, amsthm_proof_lock_lapses_after_the_document_undefines_it,
  amsthm_proof_lock_passes_an_empty_default, ntheorem_amsthm_option_keeps_its_proof,
  amsthm_proof_lock_holds_against_a_robust_copy, theorem_borrow_leaves_amsgen_unloaded,
  amsthm_reload_keeps_theorem_font_commands} RED → GREEN; guards `perfect_kernel_batch67ps` (16), with 67s/67p guards tightened. A/B vs 67o1 (613 papers: the 67s and 67p sets +
  0705.0545): Errors 1838 → 1813, Fatals 5 → 5, 0 papers gain an Error or Fatal; vs the parent a03442572a 12 differ: 9
  papers where a theorem.sty/ntheorem document or class defines its own proof now print it as pdflatex does, without
  `<proof>` (2005.04068 22 Errors → 0, 2201.05175, math0505657 "Preuve.", 1911.07591 fundam.cls:169, math-ph0505075,
  math0505275, 0807.2196, 1802.03066, 2505.07298), 2 take the document's own QED mark, 1 nondeterministic (2210.15126).
  Under the raw-class lever 1507.02955's 19 proofs are ccproof.sty's theorems with their □, headed "Proof 1" where
  pdflatex prints "Proof." (the class style's `\@begintheorem`, KNOWN_PERL_ERRORS #686); the smfart/ijmart/smfbook
  witnesses are unchanged. Binding nets amsthm, tabls, amsart, amsbook, llncs, svjour3, elsarticle, siamart: 0 worse
  than 67o1 or 67s1.
- **68d** (loader fixes from the 2605+2606 `missing_file` study, agent_reports/2026-10-15_missing_file_study.md): (1)
  a path-prefixed class whose basename names a binding (`\documentclass{lib-acm/acmart}`, shipped copy in lib-acm/) is
  that binding, loaded as the request's own file, as Perl's FindFile finds `<class>.cls.ltxml` in its installation
  directory without the request's directory (Package.pm:2125-2128, Util/Pathname.pm:340-341): no `missing_file`
  warning, no "missing file" in the verdict (cortex no longer files it as a warning), no dependency scan of the
  shipped copy, which had loaded babel (no language: “?” quote glyphs), amsart, xstring, totpages, environ, framed,
  zref and draftwatermark over the acmart binding (`content.rs::class_binding_for_path`; `fallback_request` keeps the
  path for the family bindings that read the copy, JINST); 43 tasks (2605.00109, 2605.25901, 2606.20651). (2)
  `\usepackage`, `\RequirePackage`, `\LoadClass(WithOptions)` and `\documentclass` strip a lone brace group from the
  list and from each name as latex.ltx's delimited arguments do (a package name up to four groups after its argument's
  braces: the list at `\@fileswith@pti@ns` latex.ltx:18714 and `\zap@space` :18695, each item at `\reserved@b`
  :18727-18734 and `\@onefilewithoptions` :18740; a class name two; tex.web §400), and the PI spells the argument as
  Perl's, without its braces and with its whitespace (2605.12897 `{makecell}`, 2606.09700 `{multirow}`). (3)
  `\IfFileExists`/`\InputIfFileExists` take a binding for its file except a stand-in for an author's file TeX Live
  never distributed (`latexml_core::binding::author_files::AUTHOR_FILE_STAND_INS`, 224 names):
  `\IfFileExists{arxiv.sty}` is false unless the paper ships it (2606.27281; Perl shares it for its own stand-ins,
  KNOWN_PERL_ERRORS #770). A file TeX Live once distributed answers as one (ruling 2026-10-15: `FORMER_TEX_LIVE`, 21
  names with the years of TL 2007-2024's file lists — html.sty, psfig, rsfs.sty, luximono.sty, scrpage/scrpage2.sty,
  slashbox.sty, undertilde.sty, cropmark.sty, aastex.cls/.sty and aastex62/63/631.cls, elsart*.cls, phyzzx), as does
  LaTeXML's own (`LATEXML_OWN`). Guards: every entry is a registered binding kpathsea does not find, on one list only;
  on a full TeX Live tree, every registered `.cls`/`.sty`/`.tex` binding kpathsea does not find is on a list (which
  added jmlr2e, aistats2026, colt2024-2026, uai2025, l4dc2026, neus2025, pasj00-02, asme2ej, bmvc2k, SCIS2024,
  agutexSI2019, agujournal2019, ws-p8-50x6-00 and colm2025_conference as stand-ins). (4) jfm's `lineno` option
  (lineno-FLM.sty at the document's start) is declared only when the paper's copy declares it (2605.03843's v1.25 has
  none). (5) breqn's stub notice says "only minimally stubbed". Recorded, not changed: `\RequirePackage{setup}` after
  `\usepackage{style/setup}` (KNOWN_PERL_ERRORS #771). Repros loader/{subdir_class_is_its_binding (+
  lib-acm/acmart.cls), braced_package_name, iffileexists_author_file_stand_in, iffileexists_former_tex_live,
  jfm_v125/jfm_old_copy_has_no_lineno_option} RED (67int1) → GREEN; guards `perfect_kernel_batch68d`, the batch 61
  alternate guard now expects no warning for `misc/ieeetran`. A/B vs 67int2-rel, 147 papers (the 43 subdirectory-class
  tasks, the 4 other witnesses, 100 from pop67o): Errors 506 → 506, Fatals 6 → 6, Warnings 1,509 → 1,460, no paper
  worse; XML changes (15): the acmart papers lose the scan's babel (proper quotes, `~` a no-break space again) and the
  PIs of the witnesses; 1710.04900 (harvard.sty's `\IfFileExists{html.sty}`, no html.sty in the source) is unchanged,
  keeping the html binding and its hyperref, as TeX of its era found html.sty.
- **67x** (binding gaps G2, the IEEEtran family; bindgaps/g23_macros.tsv rows IEEEtran local 354 / tl 138, IEEEtran4PSCC
  185, ieeeconf 67; KNOWN_PERL_ERRORS #710-#712, Perl's IEEEtran.cls.ltxml has the same gaps). IEEEtran binding
  (ieeetran_cls.rs), as IEEEtran.cls V1.8b (TL 2025) writes them: the compsoc executors `\@IEEEcompsoconly`,
  `\@IEEEnotcompsoconly`, `\@IEEEcompsocconfonly`, `\@IEEEcompsocnotconfonly` (:456-462), `\@IEEEnormalsizeunitybaselineskip`
  and `\CLASSINFOnormalsizeunitybaselineskip` per size option (:606-697, :976), `\sublargesize` (:639-708), `\@IEEEtunefonts`/
  `\@IEEEsetfontdimens` (:1074, interword tuning: empty), the IED parameters `\IEEEiednormlabelsep`, `\IEEEiedmathlabelsep`,
  `\IEEEiedtopsep`, `\IEEEiedlistdecl`, `\ifIEEEnocalcleftmargin`, `\ifIEEEnolabelindentfactor` (:2085-2153), `{LaTeXitemize}`/
  `{LaTeXenumerate}` (:2219-2222), `\thesectiondis` … `\theparagraphdis` (:2571-2587; `\appendices` sets `\thesectiondis` as
  :5765-5774 does; `\appendix` clears it and numbers the subsections `\Alph`, :5734-5736), `\@IEEEsaveoriginalsectiononce` run by `\appendix`/`\appendices` (the kernel's otherwise) with
  `\@IEEEprocessthesectionargument`/`\@IEEEdestroythesectionargument` (:5695-5713; a document's `\suppl` copy, 1907.08915), the page styles `\ps@headings`, `\ps@IEEEtitlepagestyle`, `\ps@IEEEpeerreviewcoverpagestyle` with
  `\@IEEEheaderstyle`/`\@IEEEfooterstyle`, `\@IEEEpubid`, `\if@IEEEusingpubid`, `\@IEEEpubidpullup` (:4228-4347, :6127-6150; page
  furniture, set nowhere), `\IEEEnoauxwrite` (:4448), `\IEEEbibitemsep` (:4484), the author-block styles and
  `\@IEEEauthorblockconfadjspace` (:4552-4561), `\IEEEtitletopspace(extra)` (:4748), `\@IEEENORMtitlevspace`/
  `\@IEEEMINtitlevspace` (:4969-4978), `\IEEEquantizevspace` (:4983-5203: arguments read as written, the object set once,
  the measuring and the space not modelled), `\IEEEquantizedisabletitlecmds` (:5208), `\@IEEEaftertitletext` and
  `\@IEEEspecialpapernotice` (:5219, :6101; empty — `\IEEEaftertitletext`/`\IEEEspecialpapernotice` stay frontmatter notes),
  `\@ifmtarg` (:5668). Older versions and variants, as fallbacks a document's definition replaces: V1.5-1.6c
  `\ifcenterfigcaptions` (V1.6b :1966-1972), `\ifnocalcleftmargin` (:1512), `\@specialpapernotice` (:4379),
  `\ifuseRomanappendices` (:4075, false: V1.8b's `\Alph` default; `\appendices` reads it beside `romanappendices`); V1.7a's
  begin-document `\proof` (:4690) for a document that left `{proof}` undefined or `\relax` (V1.8b's error case), let as
  a fallback by a primitive the hook calls, so a document's own begin-document `{proof}` replaces it; IEEEtran4PSCC.cls's `\thanksto` (:4793-4804)
  as the frontmatter's thanks note (`\lx@add@pubnote@thanks`); the SIU template's `{ozet}`/`{IEEEanahtar}` (2011/11/03 "V1.8
  based on V1.7a" :1659-1662, :3745-3790), the Turkish abstract and keywords set in the body under "Özetçe"/"Anahtar
  Kelimeler" (the English ones stay the frontmatter's), the abstract bold and the keywords bold italic as the template's
  two-column form sets them (:3746, :3765). IEEEtran/IEEEtrantools shared IEEEeqnarray (ieeetrantools_sty.rs):
  `\@IEEEeqnnumcols` set from the column specification (IEEEtrantools.sty:1352, :2498/:2535), `\IEEEeqnarrayseprow`,
  `\IEEEeqnarrayseprowcut`, `\IEEEeqnarrayrulerow`, `\IEEEeqnarraydblrulerow`, `\IEEEeqnarraydblrulerowcut` (:1566-1690;
  IEEEtran.cls:3247-3360) as spanning leader-rule rows in an `{IEEEeqnarraybox}` (open, KPE #710 residual: in an
  `{IEEEeqnarray}` a rule row keeps its number but not its rule and a separator row is removed with its number, as
  `{eqnarray}` rearranges rows; RED `alignment/ieeeeqnarray_rule_rows_keep_rules_and_numbers`; ruled 2026-10-15: such
  rows stay numbered rows with their rules, as pdflatex numbers them — kernel-wide for the eqnarray family, batch 68e
  with its own A/B, low paper impact: the 32 sampled papers with rule rows use an `{IEEEeqnarraybox}`; IEEEtrantools'
  `\PackageError` under IEEEtran stays silent). Open: an IEEEtran appendix subsection heading shows its refnum "A-A"
  where pdflatex prints `\thesubsectiondis` "A." (IEEEtran.cls:2586; pre-existing). IEEEtrantools under any
  class but IEEEtran (ieeetrantools_sty.rs; under IEEEtran the package only raises its error, IEEEtrantools.sty:73-78, and
  the class binding's IEEEproof, lists and `\IEEEauthorrefmark` stay): the
  IEEEproof family with its QED switches (:2726-2747) and the IED lists with their parameters (:1000-1140), moved from the
  class binding into shared definers both call (`define_ieeeproof`, `define_ied_lists`), `[redeflists]` (:1122-1129),
  `\IEEEauthorrefmark{<n>}` as the package writes it (:2721-2723, the footnote symbols * † ‡ …), `\IEEEPARstart` (its
  text, as the class binding's); RED `block-model/ieeetrantools_ieeeproof_any_class` (64ra) GREEN. ieeeconf (ieeeconf_cls.rs): `\appendices` sets `\thesectiondis`
  (ieeeconf.cls :4103-4117) without the subsection numbering IEEEtran's `\appendix` sets, the RBO copy's `\ieeefootline` (:4602-4630) a `role=footline` publication note. Witnesses before → after
  (Errors; 68d2-rel vs candidate): `\ps@headings` 1805.00151 1 → 0, 1805.02797 1 → 0, 1811.10132 1 → 0, 1908.05569 1 → 0,
  1910.06140 1 → 0; compsoc executors 2202.10460 5 → 0, 2202.12377 5 → 0, 2203.08603 5 → 0, 2204.11491 5 → 0, 2401.11581 5 → 0;
  rule rows cs0608041 2 → 0, 0911.1713 2 → 0, 1004.2757 3 → 0, 1112.1125 3 → 0, 0908.3162 2 → 0, 1312.4895 3 → 0;
  `\@IEEEtunefonts` 0705.1682 1 → 0, 0705.1922 1 → 0, 0801.1002 1 → 0, 1912.02718 1 → 0; V1.6 names 0704.1833, cs0501064,
  cs0504030, 0805.0012, 2009.04715, cond-mat0207222 1 → 0, 0705.3677 2 → 0; `\thanksto` 1601.01675, 1603.05188, 1710.09738,
  1803.06921, 2304.11418 1 → 0; `{ozet}` 1504.02975, 1807.04577, 1910.03090 2 → 0; `\sublargesize` 1208.3665 1 → 0, 2103.13318
  3 → 0; `\IEEEbibitemsep` 2007.01482, 2106.14760 1 → 0; `\thesectiondis` 1605.01233 1 → 0, 1702.07429 2 → 1 (empheq's
  `\empheqlbrace`); `\@ifmtarg` 1510.02032 1 → 0; `\IEEEquantizevspace` 2601.10876 5 → 0; `\proof` 2106.12111, 2208.13860, 2310.09933, 2310.09935, 2503.20066, 2509.09277 1 → 0; `\@IEEEappendixsavesection`
  1907.08915 1 → 0;
  ieeeconf 1811.05306, 1811.01547, 1905.11167, 1909.12483 2 → 0, 1803.09650, 1807.07870, 1810.07122, 1610.00529, 1807.11264
  1 → 0. Word drops are the error text gone. Binding nets vs 67int2: IEEEtran 30 docs 0 worse (0710.3817, 0805.3118 1 → 0),
  IEEEtrantools 25 0 worse (12 errors in 8 docs → 0: 1304.3949 3, 1609.02977 3, 1407.6872,
  1710.09177, 1802.00899, 1905.00616, 2002.01226, 2003.00712 1 → 0), ieeeconf 26 0 worse (1603.06163, 1610.00529, 1807.11264 1 → 0, 1811.01547 2 →
  0). Not taken (authors' additions to their copies, not a version's interface): `\eg`/`\ie`/`\etal`/`\etc`/`\vs` (11),
  `{refsize}` (11, each copy its own size), `\teaser`/`\IEEEteaser` (8), `\name`/`\address`/`\orcid` (7), `\footernote` (4),
  `\IEEEabstract`/`\IEEEkeyword` (3); version conflicts: the SIU copy's Turkish `\refname`/`\figurename`/`\IEEEkeywordsname`,
  V1.6's Roman-appendix default; `\@makecaption` (10) is undefined under article too (a kernel gap, not IEEEtran's); `\I`
  (2) is not defined by IEEEtran.cls; hyperref internals (`\Hy@raisedlink`, `\hyper@anchorstart`/`end`, `\@IEEEtheHrefequation`,
  1 each). Repros class-bindings/{ieeetran_internals_documents_call, ieeetran_eqnarray_rule_rows, ieeetran_v16_names,
  ieeetran_pscc_thanksto, ieeetran_siu_ozet, ieeetran_v17_proof_at_begin_document,
  ieeeconf_v16_title_internals_and_footline}, block-model/ieeetrantools_lists_marks_any_class RED (68d) → GREEN;
  guards `perfect_kernel_batch67x` (15; review round: `ieeetran_v17_proof_yields_to_document_begin_document_proof`,
  `ieeetrantools_under_ieeetran_keeps_class_author_marks`, `ieeetran_appendix_clears_sectiondis`). Seen, not changed: the class binding's `\IEEEauthorrefmark{<n>}` prints the
  numeral `n` as a superscript (beyond-Perl, 2404.07847) where IEEEtran.cls:4539-4546 prints the footnote symbol
  (`{2}` → †) outside the `transmag` option (whose mark is the numeral), as the IEEEtrantools one now does.
- **Fleet packaging (68d study, not a loader bug)**: the fleet image's distro TeX Live (`/usr/share/texlive`) has no
  `asymptote.sty`, which TL 2025 has and the probe reads raw without a warning: 8 + 5 `missing_file:asymptote` tasks in
  2605/2606 (2605.00095, 2605.03389, 2606.02087); a packaging fix for the worker image.
- **67y** (binding gaps G2: class bindings lacking commands their class defines; bindgaps/g23_macros.tsv rows amsart,
  revtex4-2, revtex4-1, kluwer, elsarticle, elsart, egpubl). Per binding (commands, real file:line, witnesses):
  amsart (ams_support_sty.rs `amsart_class_internals`, all three AMS classes) — `\except`/`\for`/`\forany` and the
  `toc`/`lof`/`lot`/`runhead` switches (amsart.cls:63-96, :305, :1143-1145; `\newswitch`/`\setTrue`/`\setFalse` real,
  no longer no-ops; 1304.2934, 1601.05128, 1912.08145), `\abstractbox`/`\@setabstract`/`\@setabstracta` with
  `\@setabstract` run at `\maketitle` (:837, :856-864; the box it sets the frontmatter `<ltx:abstract>`, one set after
  the title where it ends: 1512.02361, 2602.22909; 2303.07306, 2201.00740), `\@keywords`/`\@subjclass` empty (:554/:564; 2510.22858), `\copyrightyear`/`\copyrightholder` (:687-689), `\@tocwrite`/`\@tocwriteb`/`\@tocline`/`\l@figure`/
  `\l@table` (:1189-1241; 2004.14791, 1805.00244, 2503.14638), `\@defaultbiblabelstyle`, `\bibliofont` (:1244, :1262;
  2105.14892); `{proof}` after `\let\proof\relax` + `\usepackage{amsthm}` by the loader below (1510.08833 and 5 more).
  revtex (revtex4_support_sty.rs; `define_revtex41_internals` for 4.1/4.2) — `\doi`/`\doibase` now for 4.2 too
  (revtex4-2.cls:1986-1988; 2010.08759, 2301.10789), `\@@startlink`/`\@@endlink` an `<ltx:ref>` pair (:1990-2009;
  1211.1037), `\onecolumngrid@push`/`@pop`, `\onecolumn@grid@setup`, `\set@vsize` (ltxgrid.sty, layout no-ops;
  2311.06373, 2609.36113), `\let@environment` (ltxutil.sty:97; 1101.2683), `\widetext@top`/`@bot` (:7459; 2101.04636),
  `\frontmatter@title@above/format/below`, `\frontmatter@finalspace` (aps4-2.rtx:212-214; 2006.03287), `\REV@text`
  (:7547; 2202.11665), `\altaddress` (:7520), `\clo@groupedaddress`… (:6283-6308; 2402.07499), `\prep@math@patch`,
  `\ltx@footnote@pop`, `\setup@hook`, `\setfloatlink`, `\theequation@prefix`. kluwer (kluwer_cls.rs) — named
  references through natbib, borrowed (kluwer.cls never loads it: `\@ifpackageloaded{natbib}` false; `\cite` =
  `\citep`, `\cite*` = `\citeyear` (:2125), `\inlinecite` = `\citet`, `\opencite` = `\citealp`, `\shortcite` =
  `\citeyearpar`; all `\cite` under `numreferences`; kluwer.cls:2007-2145; 1306.4881, 1408.6262, 0802.0687), endnotes
  (borrowed endnotes.sty; :1909-1975; 0910.2393), `\lsim`/`\gsim`/`\nequiv`/`\dC`…`\dQ` as ≲ ≳ ≢ ℂ ℙ ℕ ℝ ℤ ℚ (:162-176;
  astro-ph0309194, math0212055; OXIDIZED_DESIGN_DIVERGENCES #598), `\I`, `{subequation}`, `\qed`, `{kaprotate}`,
  `\nomenclature`/`\nomen` and `\abbreviations` notes with their bold headings (:950-960), `{motto}`
  (a quote), `\tabcapfont`/`\figcapfont`/`\figtabdot`, `\crline`, `\rcline`… (as `\cline`), `\filedate`.
  elsarticle (elsarticle_cls.rs) — TL v3.4c `\tnotemark`, `\myfooter`/`\myfooterfont` (`[#1]#2␣`, :1022),
  `\abstracttitle` (`\@elsarticleabstitle`, :672, which names the frontmatter abstract; `\abstractname` stays the
  document's), `\keywordtitle`, `\resetTitleCounters` + `tnote`/`fnote`/`cnote`, `\absbox`/`\keybox`,
  `\Columnwidth`, `\elsLabel`/`\Newlabel`/`\elsRef`, `\comma`, `\@eadauthor`, `\pprinttitle`, `\hashchar`,
  `\elsparagraph` (2203.02441, 2209.12222, 2204.07076, 2401.05905); v1.21's (100 of 225 shipped copies) `\email`,
  `\correspondingauthor`, `{nomenclature}`, `{deflist}`/`\defitem`/`\defterm`, `\normalMode` (ends the appendices) as
  `Fallback` guesses (1305.6441, 2103.08145, 1303.2653, 2404.00783); the AAS journal abbreviations (`\apj`, `\mnras`…)
  from aas_macros' binding, provide-only, where the paper's copy has them pasted in (`class_code_contains`; 0812.4232,
  1504.02326, 1807.02064, 2003.05953, 2401.03908); `\@journal` stored by `\journal`, empty until then (:1302;
  elsart_support_core_sty.rs; 1212.6083, 1007.3192, 0812.4577). elsart (elsart_support_sty.rs, elsart_cls.rs) — `{pf}`/`{pf*}`
  an `<ltx:proof>` pair (elsart.cls:477-483; 0708.4409), `\cty`/`\cny`/`\jr`, `\@blstr`, `\ext@algorithm`,
  `\ftype@figure`, `\output@glob@notes`, `\if@nameyear`/`\if@natbibloaded` (0907.1629 adnd.sty), the MACMILLAN elsart5
  copies' `\KEY` and `\nolinenumbers` (0801.1765, 0904.0552), `\@journal` "Elsevier Science" until `\journal`
  (v2.18:28); the differential `\d` (Perl elsart_support.sty.ltxml:48) only where the paper's copy redefines it
  (`\if@symbold\else\def\d{\,\mathrm{d}}\fi`, v2.18:1382, 98 of 150 copies) or ships none, else the kernel's accent
  (elsart5/elsart4 copies; cond-mat0610502, 0706.1711), and the `symbold` option that keeps the accent. egpubl (egpubl_cls.rs) — `\p@ConfYear` stored by
  `\ConfYear` (egpubl.cls:986-987 v4.0; 1409.5623), `\shortcite` (:2063), `\CCScat` (:816; 1102.4258),
  `\EuroVisTutorialEditors`/`\EuroVisPanelsTutorialsEditors`/`\EuroVisGDxDREditors` (2604.14791),
  `\copyrightTextTitPag` (the frontmatter copyright, its leading `\copyright` left to the frontmatter's ©; 2404.02634;
  OXIDIZED_DESIGN_DIVERGENCES #599)/`\copyrightTextRunPag`, `\p@journalSubmissionText`,
  `\pStartPage`, and the single copies' selectors (`\PaperL2P`, `\ArxivPaper`…, `Fallback`). Kernel: the loader
  reloads a file a class binding marked loaded (`<file>:class_marked`: amsart's amsthm, ams_support_sty.rs) whose
  `\ver@<file>` the document let to `\relax` or `\undefined`, and `\@ifl@aded` answers false
  (`state::unmarked_by_document`; KNOWN_PERL_ERRORS #720); no other file (user ruling 2026-10-14: no general loader
  `\ver@` rule); the ltcmd declarators replace a `Fallback` guess or a kernel `Stub` as `\newcommand` does
  (`\lx@if@fallbackdefined`, latex_constructs_rust_only.rs; 2509.10728); `\addcontentsline` adds a list the node is
  not in yet (amsart's `\@tocwriteb` gave `inlist="toc toc"`; KNOWN_PERL_ERRORS #725). KNOWN_PERL_ERRORS
  #720-#725, OXIDIZED_DESIGN_DIVERGENCES #598-#599. Not ported: REVTeX's title-block production internals that documents replay for a second title page
  (`\titleblock@produce`, `\@author@finish`, `\frontmatter@author@produce@*`, `\title@column`, `\print@toc`: need
  ltxfront's author storage, which the frontmatter API replaces; 2105.06662, 2503.21669, 2604.15540), `\@makecaption`
  (undefined for every class, article's too), amsart's `\@secnumpunct` (set only inside amsart's `\@sect`; 1611.09402) and `\sectionname`… (:320-322, a running
  head in 1504.08082: a defined empty `\sectionname` put a space before every section number),
  elsart's `\@thebibliography` (a journal style's own `{thebibliography}` over it; 0907.1629). Repros
  class-bindings/{amsart_class_internals, revtex4_2_class_internals, revtex4_1_class_internals, kluwer_class_commands,
  elsarticle_class_internals, elsarticle_v1_21_commands, elsart_class_commands, elsart5_macmillan_key,
  egpubl_class_commands, kluwer_cite_star_and_title_notes, elsarticle_class_details,
  elsarticle_copy_aas_journal_macros, elsart_d_follows_the_shipped_copy}, loader/{amsart_reloads_amsthm_after_unmarking_it,
  amsart_reloads_amsthm_after_undefining_its_version, ltcmd_declarators_replace_a_fallback_guess} RED → GREEN; guards
  `perfect_kernel_batch67y` (17).
- **Residual of 67y — a font-size change after a formula rewrites its `tex=`** (RUST-ONLY, pre-existing): `$a\quad b$`
  then a top-level `\footnotesize` gives `tex="a\hskip 10.00002ptb"` (the XMath is unchanged); amsart's `\bibliofont`
  before `\bibliography` now triggers it (2105.14892, 2307.14222, 2609.15658, 2312.03234: every `\quad`). RED repro
  math-parse/quad_tex_after_a_later_size_change.
- **Residual of 67y — a `\par` in a frontmatter note pulls the formula after it into the styled text before it**
  (RUST-ONLY, pre-existing): `\lx@add@frontmatter{ltx:note}{{\bfseries Head:}\par\noindent $x$ -- v}` gives
  `<text font="bold">Head:<Math…/></text> – v`; kluwer's "Nomenclature:" heading (kluwer.cls:958-960, on a line of its
  own in the PDF) is run in for it. RED repro sectioning-frontmatter/frontmatter_note_par_then_math.
- **67z** (binding gaps G2, journal classes A: bindgaps/g23_macros.tsv rows sn-jnl, sigma, arxbj, informs, autart,
  asme2ej, aipproc, sagej, oup-authoring-template, ejpecp; every class but oup/ejpecp is one papers ship). Per binding
  (commands, real file:line, witnesses): sn-jnl (sn_jnl_cls.rs; 1802.08831's copy) — `\nomail` (:550; 2111.06962), the
  title/table/figure fonts `\Titlefont`, `\tablebodyfont`, `\figurecaptionfont`… (:443-461, :1231-1235; 2210.01472,
  2307.08421, 2402.15353), `{tablenotes}`, `\tnote` (:1280-1292; 2102.13454), booktabs for the copies that define its
  rules themselves (2006.16202 :1292-1309), `\sidewaystablefn` a no-op (:1361; 1802.08831, 2012.15392 — the kernel's
  `{table}` keeps the footnote), `\artnote`/`\miscnote` (:631-638; 2203.15424, 2605.14802) and
  `\intern`/`\equalsup`/`\equaladv`/`\equalcontribution` (2603.28334 :575-600, 2307.08214, 2501.01157, 2206.10381)
  frontmatter notes (a copy's optional-argument form with the copy's default text: 2606.24824's `\intern`, 2504.21788's
  `\equalcont`, `class_code_argument`), `\ackno` frontmatter acknowledgements (2404.03266 :639-646), `\presentaddresstxt`, the `\if…`
  switches, `\Newlabel`/`\jmkLabel` (2307.08421 :491), `{unenumerate}` (2303.09648 :1084-1116), `{biography}` with its
  bold `\author` (:1529; 2310.14485), `\orcidlogo`, the copies' `\abs`/`\argmin` only where the copy has them
  (2206.00975 :317-318); the begin-document `\let\proof\spiproof` now with or without program.sty (a preamble
  `\let\proof\relax`: 2211.03599, 2501.00469). sigma (sigma_cls.rs; 0705.3628 v3.0, 1404.0720 v1.7, 2609.01661 v1.8) —
  `\EmailDD`/`\EmailMarked`/`\URLaddressDD`/`\URLaddressMarked` notes, empty ones dropped (:77-83; 0706.0314,
  0711.1671), `\mail` where the copy has it (v1.8 :80), the class's theorem set on their own counters with the starred
  twins and `{conjecture}` from v1.2 (:161-178; 1301.4541, 1404.0720; OmniBus's fallback had numbered them by section on
  one counter; the v1.0/1.1 copies' set without them, nlin0510055), `\tsep`/`\bsep` (:182-191;
  0705.3628). arxbj (arxbj_cls.rs; 1106.2467's copy) — the VTeX URL and attribute helpers moved from arximspdf_cls.rs
  into ims_support_sty.rs, which both load (`\arxivurl`, `\doiurl`, `\surl`, `\setattribute`, `\tsub`; 1206.6586),
  `\docsubty` (:2259), `{keyword}[class=…]`/`\kwd` (:2490-2541; elsart's lost every entry) in ims_support_sty.rs
  with arximspdf's, whose collecting form a later block replaced (1107.3284's and 1309.4864's AMS codes): each block
  kept, the AMS one the subject classification, joined by `; ` (arxbj) or `, ` (arximspdf), under the class's headings,
  `\eqntext` (:1273; 1002.0175), `\startlocaldefs`, `\setvaluelist`/`\getitemvalue` (:392-502; 0811.0643), `\legend`,
  `{supplement}` with `\sname`/`\stitle`/`\slink`/`\sdatatype`/`\sdescription` (:2543-2598; 1302.0163), vtexthm's
  starred twin of every `\newtheorem`/`\newproclaim` on borrowed amsthm (:1302-1345; 1205.2947, 1307.5990), in
  ims_support_sty.rs, so arximspdf has it too (its arximspdf.cls:1287-1337; `{hy*}` 1107.3284). informs
  (informs_cls.rs; 1501.01126's informs3, 2309.11120's informs4) — the 133 `\BF…` bold symbols as
  `\mathbf`/`\boldsymbol` (:98-247; 1501.01126, 2008.02154), `\fs`, `\HD`/`\up`/`\down`/`\updown`, `\halmos`, `\n`
  (:391-913; 1402.4593, 1011.5677), `\SUBJECTCLASS`/`\AREAOFREVIEW`/`\MSCCLASS`/`\ORMSCLASS` classifications
  (:1001-1020; 1009.2653), `{Trivlist}`, the class's `\proof{head}` (:1829-1830; amsthm's took the head as text, "Proof.
  Proof.", 229 of the 456 inventory papers: 1301.2609) with theorem.sty in amsthm's place (:6; 2008.02211's
  `\let\proof\relax` + `\usepackage{amsthm}`), `\NatBibNumeric` a no-op, `{descr}`, the title/theorem fonts and hooks,
  `\ECRepeatTheorems` (:2680-2696; 1704.09011), the spacing switches, `\ECDisclaimer` dropped. autart (autart_cls.rs;
  1112.0879's copy) — elsart's `{keyword}` with `\PACS`/`\MSC` (elsart_support_core_sty.rs `define_keyword_markup`,
  shared; :1248-1258; 1106.2518, 1409.4624), `\sep`, `\e`, `\half`/`\threehalf`/`\quart`, `\Rset`…`\Zset`, `\nuc`,
  `\pol`, `\cv` (:722, :1429-1456; 1207.4104, 1611.06692), the theorem set as the copy declares it (:545-580; `{case}`
  2001.10125, a copy's `{assumption}` 1912.06343, separate counters in 0807.2769's, `secthm` 1909.02848). asme2ej
  (asme2ej_cls.rs; 1212.4243's copy) — `{acknowledgment}` acknowledgements (:1218; 1011.2269), `{nomenclature}`/`\entry`
  (:1209-1216; 1403.0296), `\qed` (:1246; 1610.03468), the fixed fonts (:301-311). aipproc (aipproc_cls.rs) — aipxfm.sty
  recorded loaded as the class loads it (:405; a paper's `\usepackage{aipxfm}` read the shipped file raw: 1010.4521,
  0705.1061; KNOWN_PERL_ERRORS #730), the designer commands (:212-216, 336-386), a copy's journal abbreviations where it
  has them (0809.4021 :1042-1124). sagej (sagej_cls.rs; 2101.10072's copy) — `{sm}`, `{credit}`, `{conflict}`, `{das}`
  statements (:464; 2505.13309, 2406.06038, 2204.02460; 1412.4039), each with `{funding}`/`{dci}` named by the heading
  the class prints ("Supplemental material", 2503.14633's copy "Supplemental Material", "Author contributions",
  "Declaration of conflicting interests", "Data Accessibility Statement", "Funding"; the XHTML prints `@name` as the
  heading, which had shown "sm." and "dci."), `{biog}`/`{biogs}` (:480; 1801.00336), `\sagesf`,
  `{refsize}`, `\titlesize`, `\absbox`, `\journalclass`, `\Highlights`, `\twitter`, and the rest of the class's
  `\RequirePackage` line (ifthen, rotating, calc, textcase, color: 2204.02460). oup-authoring-template (TL 2025) —
  `\vol` (:1054; 2104.15079), `\boxedtext` a frontmatter note led by its heading, the copy's constant ("Key Messages",
  "Significance statement") or the newer copies' first argument (:1117-1119; 2604.25257 :1271-1272; 2306.17210),
  `\secsize`… (:1676; 2606.05448's :1660), the `\sffamilyfont…` switches (:143-150; 2505.18036), `\@numbibtrue`; a
  newer copy's `\otherabstract` (a further abstract) and `\figalttext` (the image's description, as acmart's
  `\Description`, float_description.rs; the class drops it: OXIDIZED_DESIGN_DIVERGENCES #604; 2608.22901, 2608.14294,
  2609.06413) where it has them (2602.07066 :1232, :1935). ejpecp (TL 2025) — the class's theorem set within
  sections with its styles (:400-458; it had numbered `{theorem}` through; 1405.0815, 1406.6650), `\ARXIVID`/`\HALID`
  notes (:150-151, :316-328; 1104.1060, 2103.00338), `\ACKNO` (:152; 1103.0392), `\ARXIVPASSWORD`. Repros
  class-bindings/{sigma,snjnl,informs,autart,asme2ej,arxbj,sagej, ejpecp,oup_authoring_template}_class_commands,
  aipproc_paper_loads_aipxfm RED (68d2) → GREEN; guards `perfect_kernel_batch67z` (10). A/B vs 68d2-rel over 400
  witnesses + 100 pop67o (488 converted): Errors 1,291 → 354, error-free 61 → 410, Fatals 8 → 8, no paper gains Errors
  or Fatals; the 92 papers with fewer words lose error-recovery text, numbers the class does not print (sigma's and
  autart's statements on the class's counters, OmniBus's had numbered them by section on one counter), informs's
  duplicated "Proof." and amsthm's ∎, and the then-dropped alt texts (2608.22901, 2608.14294, 2609.06413); the 9 aipproc
  papers whose main file the harness missed (aipcheck.tex) 21 → 0 Errors. Binding nets vs 67int2: 0 worse in all nine
  (arxbj has none); Errors 127 → 18. Side finding, RED: class-bindings/elsart_support_theorem_tag_zero (under
  elsart_support — arxbj, elsart — a `\newtheorem` statement is tagged "Remark 0", its refnum 1). Not taken: the
  modified copies' one-paper commands (sn-jnl's astronomy block 2108.07431/2507.08592, informs `{observation}` in
  `\TheoremsNumberedThrough`, oup `{methods}`/`\absection`, sigmasthesis.cls 2510.12158 — another class). Round 2
  (review fixes): repros class-bindings/{arximspdf_keyword_blocks_and_starred_theorems, snjnl_author_notes,
  sagej_v2503/sagej_sm_heading_follows_the_copy, snjnl_v2606/snjnl_notes_take_the_copys_default,
  oup_v2602/oup_figalttext_is_the_images_description} RED (67z) → GREEN, guards `perfect_kernel_batch67z` (15); the
  `{biography}` `\author` a primitive's before-digest code. Residuals: informs's `\TheoremHeaderFont`/`\ExampleHeaderFont`
  do not reach the heads (informs3.cls:1810-1845 sets them per style, TH and EX, `\def\theorem@headerfont{\normalfont
  \TheoremHeaderFont}`; theorem_sty.rs keeps one global `\theorem@headerfont`, and informs4's two fonts differ, so a
  global `\gdef` would set examples sans: pdflatex's small caps against our bold, PDF fidelity only); a loader rule
  running a shipped copy's own simple one-line definitions for names its binding leaves undefined (the modified copies'
  long tail across all ten classes) is a pending user decision; class-bindings/elsart_support_theorem_tag_zero stays RED;
  sn-jnl with no shipped copy loads none of the class's packages (rotating…); the same visible-`@name` flaw in bindings
  outside this batch: sn_jnl_cls.rs `{declarations}` (`name='declarations'`, an environment sn-jnl.cls does not define)
  and jmlr_cls.rs/jmlr2e_sty.rs `\acks` (`name='acknowledgments-disclosure-of-funding'`). Ruled 2026-10-15: arximspdf's
  borrowed amsthm also gives it `{proof}`, which the class lacks (pdflatex errs); kept, as it keeps the author's proof.
  The arximspdf/arxstspdf row from a 24-paper sample — `\ccline` (7 arxstspdf papers), `\freefootnotetext`,
  `\write@toc@…contentsline`, `\maxcsec`, `\setvaluelist` (1203.0565) — landed in 68k; open: arximspdf's own
  `{aligned}`.
- **68k** (binding gaps G2 C, class bindings whose papers ship their own copy; bindgaps/g23_macros.tsv rows gretsi,
  PoS, JHEP3, ws-procs9x6/ws-procs961x669/ws-ijmpe, svmult, pnas-new, interact, mdpi + Definitions/mdpi, svjour3,
  arxstspdf/arximspdf, siamart/siamonline; each command checked against the witnesses' copies, `copies.py` tallies in
  the batch report). Per binding (commands, real file:line, witnesses): gretsi (gretsi_cls.rs) follows the paper's copy
  where the releases conflict — v1.2 (2008, 45 of 62 papers, 1001.1912's copy) `\titre`, `\auteur` = `\author` of
  `\coord{first}{last}{mark}` names (:154-157), `\adresse`/`\address` of `\affil{mark}{lines}` (:158-166),
  `\resumefrancais`/`\frenchabstract`, `\resumeanglais`/`\englishabstract` (:106-122, the English one in
  `{texteenanglais}`, English under babel), `\legende` (:22-58); v1.3 (2023, 2304.10248's; the default) `\auteurs` of
  `\auteur{first}{last}{email}{mark}` (:96-97, an author each with e-mail), `\affils`, `\resume`/`\abstract` (:78-87),
  its packages (babel `[french,english]` with the `english` option's language, algorithm2e, booktabs; :35-75); the
  stub's `\auteur{}{}{}{}` (name + address/email notes holding the e-mail and the mark) is gone; names' marks request
  the affiliations so labelled (1001.1912, 0706.1914, 1311.3768, 1512.06999, 2206.05928, 2304.10248, 2310.15635). PoS
  (pos_cls.rs) `\@firstaubox`, `\PoSspecialurl` (`\relax` from `\begin{document}`, PoS.cls :452, :1416-1428),
  `\PoSpaper@url` (v1.2.6 :656, :704-709; 0912.1441, 1606.09248, 1812.08139; KPE #811). JHEP3 (jhep3_cls.rs)
  `\arXivid` (v3.1.4 :1708; 0704.3719, 0709.3309) and `\@firstaubox` (:748; KPE #810). The AAS journal abbreviations
  a copy pastes in (`\apj`, `\mnras`, `\aap`…) for PoS (16 of 92 copies: 0810.0108, 1708.06808), svmult (7:
  1711.01112, 1801.09940's svmult_aa.cls), svjour3 (9: 0812.0839, 1111.3868) and mdpi (2212.11355, 2211.06541,
  2510.22959's mdpi_mod.cls) through one helper, `shipped_class::provide_shipped_aas_macros` (elsarticle's 67y code
  moved there). World Scientific: ws_journal_cls.rs (every WS class) `\titlefont`, `\authorfont`, `\tablefont`,
  `{tabnote}[heading]`, `{tabfootnote}`, `\tabmark`, `{otherinfo}`, `\citelow`, `\figurebox` (ws-ijmpe v1.0 :58-71,
  :334-340, :740-763, :951, :667; 0711.3504, 0712.0330, 0811.0496, 1302.0454); ws_procs_cls.rs `\eref`…`\Tref`
  (ws-procs9x6 :984-993; 0704.0369, 1605.04791), `\figurecaptionfont`, `\wstoc` (a contents line, kept out),
  `\mastertoc` (`\tableofcontents`), `{unnumlist}`, `{algorithm}`/`{assumption}` (:918-921), `\aindx` as the copy writes
  it (`\index{author}{…}`, ws-rv9x6 `\index[aindx]{…}`). svmult (svmult_cls.rs; v5.4 0912.1479's, KPE #812) `\ts`,
  `\if@runhead`, `\svparindent`, `\verbatimindent`, `\twocaptionwidth`, `\mtaddtocont`, `\hyperhrefextend`, `\subfigures`
  (the counter restored at the figure's end through `\AddToHook{env/figure/end}`), `{abbrsymblist}`, `\greeksym` (the
  Symbol font's Greek letter, OD #623), v1.1's `\micro` (Euler's upright Greek μ, U+03BC), v5.5's
  `\circledmark`/`\formtmp`, v5.15's `\Description` (the float's description, as acmart's; 2602.21993). pnas-new (pnas_new_cls.rs) `\bibsplit`, `\Firstpage`,
  `\Parasplit`/`\Endparasplit` (layout: nothing), `\dataavail` (a titled paragraph; v1.47 :468-492, :688; 2208.11618,
  2602.17158). interact (interact_cls.rs; v1.05 1711.05866's) `{abbreviations}`, `{jelcode}`/`{pacscode}` (and v4.0's
  `{classcode}`) as classifications, the upright Greek `\ualpha`…`\upartial` and bold `\ubalpha`… (:167-226), `{tabnote}`,
  `\jvol`/`\jnum`/`\jmonth`/`\jyear`/`\jname` stores, `\history`, `\figwidth`, `\x`, v4.0's `\colrule`/`\botrule`
  (1901.07015, 2306.12924, 1709.03559, 1709.02637). mdpi (mdpi_cls.rs) `\conferencetitle`, `\contributed`,
  `\fourthnote`/`\fifthnote`, `\emails`, `\orcidauthorONE`…/`\orcidONE`… (2017) and `\orcidA`…`\orcidZ` (since 2018,
  were no-ops for A-F) as the author's ORCID, `\conflictofinterest`, `\useofartificialintelligence`,
  `\linksupplementary` (`\doinum` now stores `\@doinum`; round 2: else the copy's default DOI), `\bio`, `\justifyorcenter`, `\highlight`, `\CorrStatement`,
  `\dist` (1709.06513, 1506.06852, 1707.09764, 2311.18672, 2208.12066). svjour3 (svjour3_cls.rs) `\lastand` (an `\and`,
  KPE #813; 1907.12906), the economics edition's `\JEL` (1402.2599 :314-317, 6 copies) and the 2023 edition's
  `{funding}`/`{dataavailability}`/`{conflict}` (2411.13175 :1133-1159). arxstspdf/arximspdf (arximspdf_cls.rs)
  `\ccline` (a `\cline` per range), `\bhline`/`\thline`/`\fhline`/`\lhline`, `\freefootnotetext`, `\relateddois` (a note,
  its `\relateddoi` DOI links), `\shref`, `\write@toc@…contentsline`, `\maxcsec`…, and `\setvaluelist`/`\getitemvalue` moved
  from arxbj_cls.rs into ims_support_sty.rs for both
  (1210.0385, 1011.1160, 1010.0304, 1203.0565, 1205.5627). siamart/siamonline (siamart_cls.rs) `\proofbox`, `\slugger`
  (page furniture), `\siamprelabel`, `{DOI}`, `{AM}`, `{MSCcode}`, `{ACM}`, `{romannum}`, `{remunerate}`,
  `\appendixnotitle`, siamonline's `{Code}` and `\dedication` (2006.12120's siamart220329 :491-930, :1426; 1805.01916,
  1901.02925, 2306.16314). Class-interface commands only; a copy without one leaves it to the paper
  (`DefinitionOrigin::Fallback`). Repros class-bindings/{gretsi_v12/,gretsi_v13_frontmatter,pos_aas/,jhep3_arxivid,
  ws_procs_class_commands,ws_ijmpe_table_notes,svmult_aas/,pnas_layout_and_dataavail,interact_class_commands,mdpi_aas/,
  svjour3_statements,svjour3_jel_classification,arxstspdf_class_commands,siamart_class_commands} (RED 67int10 →
  GREEN); guards `perfect_kernel_batch68k` (12). A/B vs 67int10 over 400 row witnesses + 100 pop67o papers: errors
  1,374 → 691, 342 papers fewer, none more; every word drop an undefined command's name or its arguments no longer
  typeset (`\wstoc`'s title and authors, `\apj` → "ApJ"). Round 2 (review 2026-10-16): mdpi `\linksupplementary`
  without the paper's `\doinum` links under the default the copy computes (2208.12066's :523-530
  `\doinum{10.3390/\@doiabbr\@pubvolume\twodigits\@issuenum\fivedigits\@articlenumber}`): the binding inputs the journal
  table the copy inputs (`Definitions/journalnames.tex`, its `\@doiabbr`), executes the copy's `\ExecuteOptions`, keeps
  `\@pubvolume`/`\@issuenum`/`\@articlenumber` as the class does and ports `\cutdigits`/`\twodigits`/`\fourdigits`/
  `\fivedigits`; without the table, a journal it names or numeric values, the file's name and a Warning (19 papers:
  2208.12066 → 10.3390/s23001234, 1901.06933, 2607.17752, `preprints` 2111.09774 → 10.3390/11020074); gretsi abstracts
  in the class's order, Résumé first, the last call of each kept, added when the title is made (`\lx@maketitle@begin`;
  2205.13867, 2206.05928, 2304.07234, 2411.07577); v1.3 `\auteurs` keeps the text between its `\auteur` calls with the
  name before it, past separators (2508.03934); svmult `\if@runhead` from the version's option (`runningheads` v1.x,
  `norunningheads` v5.x), `\subfigures` prefixed only outside `\if@numart` (:1918-1920; `\title*` 1a), `numart`/`book`
  options; interact `\ubvarrho`/`\ubvarsigma` (:223-224); sv_support: an `\institute` piece opened by `\\` after `\and`
  still leads with its mark (1810.10167: Yaohua Hu's institute 2 was shared). Repros class-bindings/{mdpi_doi/,
  gretsi_v12/gretsi_v12_abstract_order,gretsi_v13_abstract_order,svmult_v11/,svmult_aas/svmult_norunningheads_subfigures,
  svjour3_marked_pieces_after_a_break}; guards `perfect_kernel_batch68k` (18). Open: the fuzzy name label of a creator
  (`Tag ltx:creator`, base_utilities.rs; Perl Base_Utility.pool.ltxml:870-876 the same) keeps the commas of the marks
  its name shows (`Simon Lepage1,2` → `fuzzy:Simon_Lepage1` and a stray `2`), and the stray piece is an accidental mark
  request: it takes a shared mark's affiliation from the author showing it alone (2508.03934: David Picard loses LIGM to
  Simon Lepage) and supplies the requests marked labels lack elsewhere (jheppub `\affiliation[1]` with `$^{1,2}$`
  names, 1912.00557; svjour3 1810.10167); dropping the commas alone, sampled over 60 comma-mark papers, fixed 7, broke
  1912.00557 and moved 1507.03278 — needs one rule (shown marks as the requests for labelled affiliations), pending a
  ruling; PoS/mdpi merged authors split at `\ ` separators (1808.07470, 1909.00451, 2502.15737) and svjour3's empty
  creator for an unmarked `\institute` with several authors (sv_support) — frontmatter rows of their own; pnas-new
  `\theabstract` (the class's stored abstract; 2602.21299 reprints it); arximspdf's own `{aligned}` (:1209);
  `\@makecaption` from a paper's hangcaption.sty
  (interact 2208.03568, svjour3 1702.06262: G1). Not this payload's (a copy's private one-line edits, pending user
  decision (a)): JHEP3 `\mathph` (0901.2133), `\jcap` (0901.3074), JHEP3MaxThesisMod's thesis commands (1210.6500); PoS
  `\ReportNo` (1310.3045, 1012.2709's PoSmod), `\Preprint`/`\PREPRINT` (1601.04101, 1406.7652, 1712.05633), PoS_mod's
  `\speakershort`; svmult `\emailbrack` (1707.04674), svmult12 `\Scriptsize`, svmultivs_gm's `\ContactAuthor…`/
  `\Institution…`/`\authoremails` (7 papers), svmult6 `\chapterauthors`/`\acknowledgments`/`\shortauthname`, v5.5
  `{articlededication}` (2 copies) and `{contribution}` (1); mdpi 2009 `\arttype`/`\journal`/`\keywords`/`\pagerange`/
  `\sysname`/`\issn`/`\logo`/`\logosize` (0912.5334), `\academiceditor`, mdpimod's `\lastrevision`/`\titlenote`,
  `{Axiom}`, `{Conjecture}`, mdpi_pre's `\nameofjournal`, my-mdpi's `\bio`; svjour3 `\preprintnumbers` (2501.18289),
  `\makesecondtitle`, `\authormark`/`\issue`, svjour3mod's `{code}`/`\arxivnumber`/`\journalref`, svjour3VISbi's
  `\doi`, `\citetop`/`\doinum`/`\papernum`, `\MCS`, `\journalinfo`, `\rmGamma`…, `{das}`, `{datavail}`; interact
  `{published}`, `\subjclassname`, `{github_repo}`, 2404.11552's `\Ac`…, `{graphicabstract}`; siamart `\R`/`\F`/`\N`/`\eps`
  (template shortcuts in 3-7 copies), `\fundingl` (2111.11645), siamonline `\disclaimer`, `{subjects}`,
  `\supplementary`, `\correspondings`, `{setup}`; ws-procs9x6adap's `\citelow` copy. Excluded by the brief: svmult/
  svjour3 `\spn@wtheorem` (68g's `\spnewtheorem`).
  Integrated with 68j as int11 (gates 5,593/5,593; bundle A/B vs 67int10 over 1,038 papers — the 68j and 68k witnesses +
  186 of pop67o: Errors 2,906 → 1,749, 562 papers fewer, none more, Fatals 8 → 8; every word drop, by bag-of-words diff,
  is error text, an error's invisible-times product in a formula, or `\wstoc`'s TOC-only arguments; nets 136/136 0 worse).
- **68n** (class-defined keywords reach the frontmatter API; the run-353 cluster of papers whose only errors were
  `undefined:\keywords`, 373 papers, + R9 of the rawclasses lever, absorbing 67t; OXIDIZED_DESIGN_DIVERGENCES #634,
  KNOWN_PERL_ERRORS #835). K11 (frontmatter_stores.rs) serves the table's keywords setters in two more shapes, each by a
  check on what the class defined: (1) a one-argument `\keywords` a raw class's refused abstract defines for its body —
  wiley-article.cls:446-489's `\NewEnviron{wiley@abstract}` (nbdt-article.cls:430-473, and its renamed copies
  wiley-networks, wiley-article_final, zadpreprints, 1-article, wiley), astlb.cls:199-205's `{pazhabstract}` reached
  from `\endabstract` → `\maketitle` → `\@maketitle` (pazhastl, pazhb, pazhb_eng, pazha2, pazh_mod, pazhastlm,
  astrobul), mtt.cls:176-181's own `\renewenvironment{abstract}` (mttr) — found by `abstract_setter_definitions` and
  defined for the abstract's body only by `\lx@abstract@class@setters` (the head of `\lx@begin@abstract`,
  base_utilities.rs): the frontmatter's keywords, or, in an abstract kept in place as a supplement's, a paragraph in
  place with it; a refused `\renewenvironment` now keeps its code as `\lx@dropped@<name>`/`\lx@dropped@end<name>`
  (sect08.rs); (2) a `{keywords}`/`{keyword}` whose begin code only typesets a label (`label_only_begin_code`:
  paper.cls:621-627, acmconf.cls:340, pasa, biometrika, irmaems, ieice) becomes `\lx@class@keywords`: the environment
  is the frontmatter's keywords named by the label, `\keywords{…}` the keywords of its argument, a bare `\keywords
  text` the class's own code. Witnesses before → after (base 67int11-rel): 1706.00599, 1801.08881, 1202.2889,
  astro-ph0306137, 1507.06302, 2103.05801 each 1 Error → 0 with `<keywords>` (pdflatex prints them in the abstract);
  rawclasses lever, paper.cls: 9 of 10 papers (0707.1307, 0712.4042, 0808.0240, 0809.0271, 0812.2713, 0901.1497,
  1004.5246, 1007.3042, 1008.0145) run-in `<paragraph>` → `<keywords name="Keywords:">`, 0711.1008 has its
  `{keywords}` commented out. A/B vs 67int11-rel over the 373 cluster papers + 100 of pop67o + 150 papers on shipped
  raw classes defining a `{keywords}`: cluster Errors 374 → 43 (330 papers to 0, 331 gain `<keywords>`, none worse),
  pop67o identical (1902.02884 times out in both), shipped 18 gain `<keywords>`, errors identical, the only word
  drops the label moved into the keywords' name. Repros sectioning-frontmatter/class_keywords/* RED (67int11) → GREEN
  (three CONTROL); guards `perfect_kernel_batch68n` (22 with round 3). Not served (listed): the "article" row (36 papers: 25 with the
  document's own `\renewenvironment{abstract}{…\providecommand\keywords{\par\medskip\noindent\textit{Keywords:}…}}`,
  a parameterless run-in keywords inside an abstract the kernel's lock refuses — 1205.4818, 1403.4689, 2405.19523 —
  as hsrbericht.cls:348 in TeX Live; 5 shipping an arXiv template named `natbib.sty`/`neurips_202x.sty` whose
  `\keywords` our bindings of those names shadow — 2308.08611, 2403.02576, 2410.06567, 2411.12876; 1 defining it
  nowhere, 2403.06358); cup-pan.cls:134-147 and its copy mytemplate (6 papers: 2008.06930, 2102.01732, 2004.14479)
  patch the locked `\abstract` with `\preto` then `\appto`, and the refused-definition record keeps only the last
  patch, which lacks the `\keywords`; pazh's `\englishkeywords`; 2305.04260's `\DeclareObjectType` error (another
  cluster). Label-only shapes, in the versions read (TeX Live 2025 where it ships the class, else the papers' copies):
  converted — paper.cls (TL), pasa, biometrika, irmaems, ieice, emsprocart's label-only copies (1202.4161, 1006.0632,
  1610.08545, 0803.2841, 1203.6509), lni's older copies `\fontsize{9}{10}\selectfont\noindent{\bfseries Keywords:}`
  (1604.03313, 1810.05361, 2108.00996, 2110.01641); not taken (each stays the class's): a flag (tADR2e/rQUF2e/gGAF2e/
  tCPH2e `\global\keywordstrue`; TL 2025's lni.cls:561-565 and its copies 2403.01613, 2402.18227, which also `\def\and`),
  a box (ar-1col, iau, ar2e, biom, IOS-Book-Article, iosart2x, ifacconf, autart `\keyword`), a list (fundam, basi,
  ragtime, cpc-hepnp, fac), a definition (cidarticle `\def\and`; elsarticle-like `\sep`), a conditional (emsprocart's
  `\if@twocolumn\section*{\keywordname}` copies 0803.2060, math0612455; icmart; mnras-like), an optional argument
  (vmsta), centering environments (sf2a-conf), jmlr.cls:235 `\bgroup\leftskip 20pt…{\bfseries Keywords:}` would match
  but jmlr has a binding. Round 2 (review findings): the label becomes the keywords' `name` as its text only
  (`LabelText`; round 1 put a group label's `\noindent`/`\vskip` into the name and its `\bfseries` onto the keywords
  text of 10 of 18 converted shipped papers — emsprocart 1202.4161/1006.0632/1610.08545, ieice 2002.00666/1604.03276/
  2509.21881, lni 1604.03313); Part A names the keywords by the class's label too (`label_before_argument`: astlb "Key
  words:", pazh "Keywords:"; wiley's `\allcaps` and mtt's `\item` keep the default); `\keywords{…}` is the keywords of
  its group only when `\par`, a blank line or `\end…` follows it, else the class's code runs (`\keywords {alpha}, beta`
  kept together); `\end<name>` outside the environment is the class's own end code; a supplement's in-place abstract
  typesets its keywords with the class's own definition when that only typesets label and argument (kwmtt.cls:
  pdflatex's "Keywords: english words"), else as a paragraph of the class's label where its code shows one and the
  keywords; a found setter that does not use its argument (beilstein.cls:854, fbithesis.cls:272) is the class's own
  definition, not the keywords. Guards `perfect_kernel_batch68n` 11 → 20. Round 3 (re-review): where the class's label
  cannot be read (mtt.cls:181's language-chosen `\mtt@keywords`) the in-place keywords have no label — round 2 printed
  the English default "Keywords:" (with a double gap) where pdflatex prints "Ключовi слова:" (1310.0585, 1310.0589,
  1212.3890); the main frontmatter `<keywords>` of such a class carries the default name (its language-selected label
  cannot be read without running class code), a known limitation. A class macro's skip (`\quad`) or a `\kern` between two
  label pieces is one space of the name (round 2: `\textbf{Key}\quad\textbf{words}:` "Keywords:", base "Key words:").
  Guards 20 → 22 (`group_label_pieces_parted_by_quad_and_kern`,
  `supplement_abstract_keywords_without_a_readable_label_have_none`).
- **68g** (llncs theorem numbering, the 68a residual KNOWN_PERL_ERRORS #743): llncs_cls.rs installs the class's
  theorem set with llncs.cls's own code (:1142-1174, TL 2025 v2.26; the same in every shipped copy v2.2-v2.26, checked
  over 18 versions from 1,250 arXiv llncs papers): `{theorem}` by section only under `envcountsect` (reset by section
  under `envcountreset`, by chapter otherwise), the rest through `\spn@wtheorem` over `\@spynthm`/`\@spxnthm`/`\@spothm`
  mapped onto `\spnewtheorem` (the theorem counter under `envcountsame`), `{definition}`'s body italic as the class
  sets it. Was "Theorem 0.1", "Fact 0.2" under every option (Perl the same); now pdflatex's numbers under default,
  envcountsame, envcountsect, both and envcountreset. Repros class-bindings/llncs_theorem_numbering_{default,envcountsame,
  envcountsect,envcountsame_envcountsect,envcountreset} (RED 026cf18f87 → GREEN); guards `perfect_kernel_batch68g` (5),
  `perfect_kernel_batch68a::llncs_class_internals` flipped ("Fact 2", "Assumption 3"). A/B vs 026cf18f87 over 480
  papers (400 llncs, 41 of them with a numbering option; 6 witnesses; pop67o): Errors 605 → 605, Fatals 4 → 4, words
  unchanged; 357 identical, 50 differ only in theorem tags/refnums/ids, 64 also in `{definition}`'s italic body (font
  attributes, italic box widths), 1 (1610.02500, class fac) a math-parse nondeterminism (a base rerun equals ours).
  Witnesses: 0906.4121 "Fact 3.1" kept, its lemmas/definitions now "Lemma 2.1", "Definition 3.1" as the PDF;
  1003.2851, 2303.08726, 2202.10090 (envcountsame+envcountsect "Definition 2.1", "Problem 2.2"), 1312.1526, 1104.3211,
  1212.0139, 1808.10364, 1412.6065, 2308.02825 checked against pdflatex. llncs net vs 67int8 and vs 026cf18f87: 0
  worse. Round 2 (the 68g review): `\spnewtheorem` is the class's dispatch (llncs.cls:1061, block :1060-1125) over the
  `\lx@spnewtheorem` primitive, whose branches refuse an env already defined (`\@ifdefinable`, an Info) and define
  `\<env>name`; `\@spothm` follows the paper's copy of llncs.cls (v2.26 without one; OXIDIZED_DESIGN_DIVERGENCES #614
  corrected): v2.16+ make the env an alias counter of the base (`register_counter_alias`, shared with aliascnt_sty.rs),
  v2.24+ reset it by the section ("Hypothesis 1, Conj 2" in each section, v2.20 "Hypothesis 3, Conj 4"), an undefined
  base is the class's "No theorem environment" Error. hyperref's `\autoref` names an alias by its own
  `\<type>autorefname`/`\<type>name` (none if neither, as pdflatex): "Lemma 2", "Definition 3" under envcountsame
  (round 1 "Theorem 2", "Theorem 3"), a paper's `\spnewtheorem{obs}[theorem]` "Observation 4", an aliascnt lemma
  without names "2" (was "Theorem 2"). A `\newcounter` of an alias (`\let\c@case\relax\newcounter{case}`, 2303.08726)
  gives it its own register again (round 1: every case stepped the theorem counter, "Lemma 25" for the PDF's "Lemma
  23"). Guards `perfect_kernel_batch68g::{llncs_autoref_envcountsame{,_v214_copy}, llncs_spothm_section_reset{,_v220_copy},
  llncs_spnewtheorem_errors, llncs_alias_newcounter, aliascnt_autoref_names_the_alias}`, refs asserted in the five
  numbering guards. A/B vs 68g r1 over 568 papers (the round-1 480 and 95 hyperref+`\autoref` or aliascnt papers of
  pop67o): Errors 1,169 → 1,169, Fatals 6 → 6, words unchanged; 542 identical, 17 differ only in `\autoref` names (all
  the env's, as pdflatex: 2202.10090, 1312.1526, 1504.06105, 2412.05162, 2601.04573 "Nonexample 9", 2102.01037's
  aliascnt `{thm}` " 3.4" with amsthm's `\thmname`), 2303.08726 in its case tags and the lemmas after them (now the
  PDF's). Ruled: a paper's own `\newtheorem{x}{X}[section]` keeps "X 1.1" (the PDF's "X 11", llncs.cls:1058
  `\edef\@thmcountersep{}`, OXIDIZED_DESIGN_DIVERGENCES #615). Open: latex.ltx `\@definecounter` keeps an existing
  `\the<ctr>` ("Command `\thecase' already defined -- not changed"); the kernel's `\newcounter` redefines it, as Perl's
  NewCounter (Package.pm), so 2303.08726's cases print "Case 1, Case 2" where the PDF prints the enclosing lemma's
  "Case 22, Case 22" (repro class-bindings/llncs_alias_newcounter). Round 3 (the 68g re-review): the branches title the
  env through `\csname <env>name\endcsname` (llncs.cls:1073/1082/1096/1122; `\renewcommand\lemmaname{Lemme}` "Lemme 2"
  in title and `\autoref`, r2 title "Lemma 2"), and the alias defines aliascnt.sty :54-57's `\AC@cnt@<env>`/`\cl@<env>`
  (a paper's own `\aliascntresetthe{lemma}`: no Error); guards `perfect_kernel_batch68g::{llncs_renamed_theorem_title,
  llncs_aliascntresetthe}`.
- **68a** (binding gaps G2, journal classes B: bindgaps/g23_macros.tsv rows aa, acmart, aastex631/aastex63, mnras,
  llncs, jfm, tac, aamas; each macro checked against TeX Live's class or the witnesses' shipped copies, 1,035 witness
  papers unpacked: a name defined by at least a version of the class went in, a one-line definition only some authors
  added to their copy did not). Per binding (commands, real file:line, witnesses): aa (aa_cls.rs, aa_support_sty.rs) —
  `{theorem}` (2509.20613's v9.1 :1556; 1601.05309, 1805.03673; Perl lacks it too, KNOWN_PERL_ERRORS #740), `\arxiv`
  with `\arxivprefixe`/`\arxivprefixesep` (:2058-2062; 1604.01307, 1809.05581, 2103.09505; #740), v7's `\if@referee`/
  `\if@onecolumn` set by the `referee`/`onecolumn` options (1201.4282 :85-88; 1208.0304, 1501.05323, 1608.08833), the
  language editor's `\LEt` on the `lenote` counter (:165-173, provided at begin-document; 1607.03898), `\topics` as a
  frontmatter note (:837; 1810.01624), `\etalname`, `\VSpaceAfterTabCaption` (2504.01061), `\yearCop` (2609.03060), and
  the A&A language editor's v9.1 copy's five journals `\adndt`, `\jpb`, `\jpcrd`, `\raa`, `\zphys` (7 identical copies,
  2008.09582 :1902-1961), provide-only. acmart (acmart_cls.rs, TL 2025 v2.16) — environ loaded where styles are read raw
  (:288; 121 of 128 copies; #741), `\acmConference@shortname`/`@name`/`@date`/`@venue` kept by `\acmConference` with the
  class's placeholder conference (:1582-1600; 30 papers: 2604.15990, 2604.21043), the `\@received` history (:1859-1872;
  2210.01206), `\@subtitle` (:1611-1612; 2608.16886), `\ACM@format`/`\ACM@format@nr` and by them `\@titlefont`,
  `\@authorfont`, `\@affiliationfont`, `\@proofnamefont`, `\@proofindent` (:51-56, :2515-2539, :2659-2687, :3490-3508;
  2006.07812, 2201.00614, 1810.00952, 2608.06906), `\acksname` (:3570; 2208.13741), the title-block registers
  `\author@bx@sep`, `\author@bx@wd`, `\num@authorgroups`, `\@ACM@title@width`, `\mktitle@bx` (:1613-1614, :1767, :2225,
  :2758-2761; 2606.30549, 2607.17095, 2608.24300, 2608.20335). aastex63/aastex631 (aas_support_sty.rs, aastex_cls.rs;
  2107.07519's aastex631.cls, 1907.08642's aastex63.cls) — `\ifsuppressAffiliations` (:7324-7325; 39 papers, 2108.09109,
  2001.00952), the edit switches' flags `\ifturnoffedit`/`one`/`two`/`three` (:5790-5809; 2109.04933, 2609.30834),
  `\authorcomment` (:5843-5850, honouring them; 2004.09658, 2205.09859, 2303.08830: `\turnoffedit` drops the notes, as
  the PDF does), `\resetapptablenumbers` (:7210-7216; 2308.05494), `\currtabletypesize` kept by `\tabletypesize`
  (:1928-1932; 2202.09386), `\savetablenum`, `\tablereferences`, `\deluxetablecaption`, `\tabcaption` (:1926, :5391,
  :4780, :1609-1611; 1910.11401, 2403.12118, 2408.17407), `\onecolfootnote` (:7316), `\BeforeBegin` (:581; 2111.03992),
  `\mdash` (:7141), `\apjsecfont` (:219), `\ackbox` (:1205), `\ifnumlines`, `\ifpreprint`/ `\ifpreprinttwo` set by their
  options (:167-171, :253; 2301.00027, 2608.12156), `\aspconf` provide-only in the class binding (:6164; 2307.09521;
  aas_macros_sty.rs untouched). mnras (mnras_cls.rs, TL 2025 v3.2) — `\doi`, `\eprint` with `\@eprint@arXiv`/`@dblp`,
  `\mniiiauthor` (:1399-1479; 1609.03154, 2008.09832, 1709.09375, 1507.08002; #742), v3.0's `\apspr` provide-only
  (2404.04493). llncs (llncs_cls.rs, TL 2025 v2.26) — `\if@envcntreset`/`@envcntsame`/ `@envcntsect`/`\if@runhead` set
  by their options (:25-36; pscproc2.sty:19, 0905.3107, 2303.08726), `\@spothm` and `\spn@wtheorem` by the numbering
  option (:1086-1097, :1155-1167; 0906.4121, 1003.2851, 2202.10090; #743), the language switches (:79-197; 0908.0239,
  1812.03939; `\switcht@albion` restores the binding's own names, OXIDIZED_DESIGN_DIVERGENCES #613), `\@subtitle`,
  `\institutename` (empty: the institutes are the frontmatter's; 2112.05140), `\phantomsection` (:561; 1709.06172). jfm
  (jfm_cls.rs; 1702.03412's copy) — the caption-legend symbols `\fullcirc`…`\full` and `\rmGamma`…`\rmOmega`
  provide-only (:1380-1405, :1419-1429; 1806.08341, 2004.10578), `\thesubequation` and `\returnthesubequation`
  (:641-649; cond-mat0107563), `\useAMSsubequations`, `\norule`, `\followon`, `\fulloutfootnote`, `\indexsize`,
  `\boldit`, `\halflineskip` (2304.12657). tac (tac_cls.rs; 2401.05974's 2023 copy, 0708.1921's 2015 copy) —
  `\mathrmdef`, `\mathbfdef`, `\mathopdef`, `\mathsfdef`, `\mathcaldef`, … with `\xspace` where the copy loads xspace
  (:651-678; 45 papers: 0709.0837, 0711.0674), `\refs`/`{refs}`, `{references}`/`{references*}` as bibliographies
  (:545-578; 0812.1674, 0902.2554, 1202.6034), `\dedication` a frontmatter note (:311; 0812.1674), `\cauthor`,
  `\endproofbox`, `\proofof`, `\state`/`\staterm`/`\thirdleveltheorems` (:148-163; 1106.1983), `{remarks}`/`{examples}`
  at begin-document where the paper has none (:466-470; 0907.2460; a paper's own `\newtheoremrm{examples}…[section]`
  replaces them, as tac.cls's `\def`-based `\newtheorem` lets it: 2407.21129). aamas (aamas_cls.rs; 2602.10598's copy) —
  `\doi{SUFFIX}`: `\@doi` = 10.65109/SUFFIX, a DOI pubnote from the preamble (:1435; 65 papers: 2602.03799), defined
  only where the paper's copy defines it or none is shipped (r2: the 2021 v1.78 copies of 2510.21200, 2602.05888 and
  2608.28553's commented-out lines have none, and a reference list's own `\providecommand{\doi}` prints). Repros
  class-bindings/{aa_class_commands_v7_v91, acmart_conference_parts_and_title_fonts, aastex63x_table_and_edit_commands,
  mnras_reference_links, llncs_class_internals, jfm_legend_symbols_and_subequations, tac_class_commands, aamas_doi} RED
  (67int6) → GREEN; guards `perfect_kernel_batch68a` (8). A/B vs 67int6-rel over the 1,035 witnesses + 100 pop67o:
  witnesses' Errors 1,993 → 964, error-free 56 → 536, 504 papers with fewer Errors, none with more, Fatals 0 → 0; the
  310 with fewer words lose error-recovery text (`\Hom` → Hom, the `\mathrmdef` arguments), `\mniiiauthor`'s second
  branch (1507.08002, 1806.09815), the notes `\turnoffedit` drops (2205.09859, 2303.08830, as their PDFs), a DOI read as
  one word, appendix tables renumbered by `\resetapptablenumbers` (2601.08948 B5 → B1), 1608.08833's one-column equation
  forms (its `[onecolumn]`); pop67o 100: Errors 402 → 401, Fatals 5 → 5. Binding nets vs 67int6 (aa, aamas, aastex,
  aastex62, aastex63, aastex631, acmart, jfm, llncs, mnras, tac, emulateapj, aas_macros): 0 worse in all; Errors 177 →
  78. Not taken (pending the loader-rule decision: one-line definitions only some authors added to their copy): aa's
  journals `\jaavso` 14, `\rnaas` 8, `\maps` 7, `\lrsp` 6, `\pnas`, `\aspc` (108 names in 187 uses); aastex's
  `\acceptjournal` 30 (one author's copies, "Added by Bin Ren"), `\plotthree`, the heliophysics journals `\frass`,
  `\jgra`, `\angeo`…; mnras's `\psj` 13, `\mnrasl` 6, `\kms`; acmart's `{remark}` 11, a variant's `\acmshortauthors`/
  `\acmshorttitle` 6 (2603.20062, 2609.01519; a copy without environ), `\authorwithoutinstuition` 5; llncs's
  `\etal`/`\ie`/`\eg`; jfm's `\Nu`, `\Ma`, `\Ri`, `\Da`; fairmeta's whole row (per-variant `\code`/`\project` with
  conflicting labels, `\authoremails`, `\github`; its `\addtolist` internals). Not this payload's: `\@makecaption` (aa
  12, mnras 5 papers: xtab's `\topcaption`, papers' own caption code; no class defines it in LaTeXML — an article/
  kernel gap). Not ported (main-session ruling 2026-10-16): acmart's title-block procedures `\@mktitle`, `\@mkauthors`, `\@printendtopmatter`,
  `\@mkauthorsaddresses`, `\@@authornotemark` (2-4 papers each, supplementary title pages: 2609.39019, 2605.14703), the
  one-paper storage macros `\@acmDOI`, `\@acmVolume`…; the old aamas20xx/aa-family classes the binding rows also list
  (aamas2017.cls `\numberofauthors`, aadebug.cls `\corr`…). Residuals: AASTeX's `\edit<n>` prints its text whatever
  `\turnoffedit<n>` says (the class drops it; kept by ruling 2026-10-16); llncs's default theorems ignore
  `envcountsame`/`envcountsect` (numbered within the section on their own counters; KNOWN_PERL_ERRORS #743, both
  engines; ruled to its own payload 68g, which flips the llncs guard's "Fact 0.2"/"Assumption 0.3"); the llncs language
  switch's divergence OXIDIZED_DESIGN_DIVERGENCES #613 accepted (ruling 2026-10-16); tac's theorems stay theorem
  environments (the class's are `\subsection` headings). Side finding (pre-existing; RED row, ruling 2026-10-16): a paper's
  `\newtheorem{proposition}{Proposition}[section]` over a `\newtheorem{proposition}[theorem]…` (a binding's, or the
  paper's own after `\let\proposition\relax`) numbers from a counter that reads 0 (tac's 2407.21129: "Proposition
  2.0"; pdflatex "Proposition 1.1" on the article repro macro-state/newtheorem_over_shared_counter_theorem, RED, not
  root-caused). Round 2 (review of 68a): aa's `\arxiv`, `\arxivprefixe`, `\arxivprefixesep` and `\etalname` (v8.2+
  only: 14 and 13 of 139 copies lack them, 1201.4282's v7.0) and aas_support's AASTeX 6.2+ commands (`\mdash`,
  `\tabcaption`, `\BeforeBegin`, `\authorcomment`, `\apjsecfont`, `\onecolfootnote`, `\resetapptablenumbers`,
  `\savetablenum`, `\tablereferences`, `\deluxetablecaption`; none in AASTeX 5.2's aastex.cls, 1001.0026) are
  fallback guesses (`DefinitionOrigin::Fallback`), so a paper's own `\newcommand` of them wins (it was ignored, an
  Info); `\authorcomment`'s label follows the paper's copy, else the class name ("(Author~note: …)" in aastex62/63,
  2205.09859 :5731; numbered in aastex631 and the aastex63 copies of 2312.04630, 2409.02158, 2504.08889, 2512.11572);
  tac's `\dedication` is a pubnote (`\lx@add@pubnote[role=dedication]`, as aa's and svjour's). Repros class-bindings/
  {aamas_doi_older_copy, aamas_doi_2026_copy, aa_class_newer_commands_yield, aastex5_newer_commands_yield,
  aastex63_authorcomment_label, aastex63_copy_numbered_authorcomment}; guards `perfect_kernel_batch68a` (13). Not
  guarded the same way: the registers aa v9.1 adds (`\VSpaceAfterTabCaption`, `\yearCop`) and AASTeX's `\ackbox`,
  `\ifnumlines` (an allocation does not yield to a paper's own; no witness).
- **67w** (binding gaps G2: the Wiley class family; bindgaps/g23_macros.tsv rows WileyNJD-v2, WileyMSP-template,
  WileyASNA-v1, WileyNJDv5, and WileyNJD-v1 through the same binding). One family binding
  (`wileynjd_cls::wiley_njd_family`, serving WileyNJD-v1/-v2, WileyNJDv5, wileyNJDv5 and, no longer through a
  `LoadClass{WileyNJD-v1}`, WileyASNA-v1), the copy the paper ships deciding where versions conflict (box
  environments, theorem set and styles, subfig, amsrefs, apacite), else the class name (`version_from_text`, unit
  test `wileynjd_cls::tests::wiley_version_from_text_batch67w`). Line numbers are those of 1712.08341's
  WileyNJD-v2.cls (312 shipped copies, same bodies), 2302.02310's WileyNJDv5.cls, 1804.11014's WileyASNA-v1.cls,
  1511.08547's WileyNJD-v1.cls. `\presentaddress` (v2:1781, v5:5066, ASNA:1195; a present-address note; 156+68+38 papers
  of run 353: 1701.02349, 1705.09846, 1712.08341); `\abstract[2][]` (v2:1857, every copy; `\abstract[Summary]{…}` names
  the abstract, the kernel's `\abstract` read "[Summary]" as text in 219 of 490 sampled papers; 1701.02349); the 47
  theorems in the class's three amsthm styles and its redefined `plain` (v2:2902-3030, v5, ASNA; v1:4481-4568 its 20,
  with `{example}`), the styles as the paper's copy defines them (`copy_theorem_styles`; without a copy the version's:
  v2 bodies upright, v5's `plain` and WBstyleone italic, ASNA's WBstyleone italic, v1's WB styles italic with
  bold heads and its `plain` upright with an italic head; 2302.02310's v5 theorems and lemmas italic, 1902.09230's own
  `{Algorithm}` upright, as pdflatex sets them), WBstylethree
  left current (v2:3005: a paper's own `\newtheorem{thm}` upright as pdflatex sets it, 1712.08341's copy; the binding
  had ended with an invented `\theoremstyle{plain}`):
  `{assumption}` 44+23 papers (1803.11411, 1808.10367), `{eexample}` (1806.09518), `{property}`, `{result}`, `{LWrule}`,
  each its own counter as in pdflatex (1807.03475's "Lemma 1, Theorem 1", formerly OmniBus's "Theorem 2.2");
  `{tablenotes}`/`\tnote` (v2:2816-2828; 1808.05540, 1808.09739, 1910.13464); `{biography}{photo}{text}` (v2:3445,
  IEEEtran's float shape; 1705.00674, 1812.10266); `{ack}[title]` (v2:3393, named acknowledgements; 2205.06045,
  2412.02681); `{boxtext}` (v2:466, ASNA:269; v5:592 `[1]`) and `{boxwithhead}` (v5:601) as framed.sty `{framed}` blocks
  (1811.09505, 2309.00837, 2604.17508); `\bmsection`/`\bmsubsection`/`\bmsubsubsection` (v5:5762-5764, `\section` as it
  is when called: star, optional title and appendix numbering; the former `\bmsection{}` read `*` as the title,
  2405.03603, 2511.08532); `\doiheadtext` (v2:1820, a DOI note; 1803.06134); the page-furniture stores
  `\historydates`, `\fundinginfohead(text)`, `\artid`, `\journalname`/`\journallogo`/`\jurl` (v1 of 2020), `\uns`
  (v2:1180, `\thinspace` out of math for the class's `$\,$`, which ends the paragraph's `--` ligatures in the
  kernel: 2208.08472's 27 en dashes), `\subtitlefont`; `\fundingAgency`/`\fundingNumber` ("Agency, Number." in `\fundingInfo`, v2:1877; ASNA:1300
  bold and italic in running text, 70 papers: 1806.06754, 1810.06326); `\openaccess` (v1 2010.09463, a licence
  pubnote); ASNA's journal abbreviations (ASNA:2722-2788, `\aaa`: 1909.01905); the class's loads of listings and
  alltt (all versions; `\lstset`: 1802.08557), `[labelformat=empty]{subfig}` (ASNA:250, v1:256; 1511.08547) and v1's
  amsrefs (1511.08547's reference styles, :33-43, only under APA/AMS/ACS/AMA/VANCOUVER; `\bib`/`{bibdiv}`: 9
  papers; their APA style is amsrefs', apacite only for v2 and later). WileyMSP-template (wileymsp_template_cls.rs):
  `\dedication` (:34, a dedication pubnote; 158 papers, nearly all the template's empty `\dedication{}`: 2003.09099),
  `\threesubsection` (:49, "Head: " run in italic; 87 papers: 2009.06328). OXIDIZED_DESIGN_DIVERGENCES #601 (table
  notes' leading item), #602 (`\doiheadtext`), #603 (a `\section` in `{boxtext}` is the box head). Not done: v1's `\everymath{\rm}\let\text\mathrm` in theorem bodies (ruled 2026-10-15:
  left out, `UPRIGHT_MATH` — ported, it reads `A_{ij}` as one `\rm` identifier with the subscript "ij"; math in a v1
  theorem is italic where the PDF sets it upright: 1511.08547, 1707.00398, 1709.02675), v5's
  `[apaciteclassic]{NJDapacite}` (the binding loads apacite with `natbibapa`; no witness harmed), `\xleftrightarrow` (v2/v5 via amsmath's
  `\ext@arrow`, which the binding renders as →; 2211.16474, 2309.00837), one-copy names (`{Corollary}`… 2002.01818,
  `\cd`/`\ci` 2408.07575, `\E` 2407.00139, `\maketitlesup` 2510.16163: not in the class as shipped elsewhere), and
  wiley-article.cls (#212: Overleaf's own Wiley template, raw, no binding; its `\keywords` is defined inside the
  class's abstract body, which the kernel's locked `{abstract}` bypasses: 2009.12523, 2007.02789; landed in 68n without
  a binding, the K11 rule serving every class of the shape). Repros
  class-bindings/{wileynjd_v2_class_commands, wileynjdv5_box_environments, wileyasna_funding_in_text,
  wileymsp_dedication_threesubsection, wileynjd_v1_loads_amsrefs} RED (67int3) → GREEN, {wileynjd_theorem_styles,
  wileynjdv5_theorem_styles, wileynjd_uns_keeps_en_dashes, wileynjd_v1_reference_styles, wileynjd_v1_apa_is_amsrefs}
  RED (67w) → GREEN (67w r2); guards `perfect_kernel_batch67w`.

- **68q** (`\unvbox` puts a vertical box's list back; study agent_reports/2026-10-16_perf_ram_study.md (b)): tex.web
  §1110 `unpackage` splices the box's list and drops the box, so a `\lastbox` (§1080) peel loop
  `\loop \setbox\pb=\vbox{\unvbox\pb \global\setbox\lb=\lastbox}\ifvoid\lb … \repeat` takes the lines one by one and
  ends. `\unvbox`/`\unvcopy` returned the `\vbox`/`\vtop` whatsit itself (Perl `Whatsit::unlist`,
  TeX_Box.pool.ltxml:725-742), `\lastbox` took it whole and left an empty `\vbox{}` that the next round took again, for
  ever: infernce.sty's `\@adjustPremises` under every semantic.sty `\inference` (31 of the 44-paper fleet
  `oom:alloc_failed` sample are this loop; ~315 corpus-wide is an extrapolation, 31/44 × 448; KNOWN_PERL_ERRORS #840).
  tex_box.rs `vlist_of` splices into a vertical box's own list (`VBoxContents`, marked `lx@vbox@list` in
  base_utilities.rs `predigest_list_in_mode`; `\unvcopy` as copies) between a pair of boundary marks
  (`\lx@splice@begin`/`\lx@splice@end`, round 2, replacing round 1's line-only `\par` mark): the end mark closes what
  the list opened and left open (the paragraph the box's end finished, §1086; a minipage's or `\parbox`'s `ltx:p`; a
  `\noindent` `ltx:para`), the list-tail primitives see through both; on the main vertical list, in a float's, note's
  or minipage's body, inside a mode-setting environment in the vbox, in `\vadjust`/`\insert` material and in math the
  box goes back whole (OXIDIZED_DESIGN_DIVERGENCES #637, awaiting R6 and a ruling). OOM sample (44, `ulimit -v
  8912896`, default `--max-memory`) 67int10b-rel → 68q: 37 killed/Fatal → 6 (31 more complete, 38 of 44, at 0.2-1.2 GB
  in 1-19 s; 29 of the 31 new completions load semantic; 1209.2208 PushbackLimit in a `table*`/`{kaprotate}` and
  hep-th0209063 IfLimit unchanged; 1410.2418, 2311.11318, 2402.17444 still out of memory, no semantic.sty; math0301370
  OOM → Timeout:Convert). A/B over 200 pop67o papers: 198 byte-identical, 2006.07243 one "Missing number" Warning fewer
  (its local amsmath copy asmath.sty `\measure@` peels the alignment), 2210.15126 nondeterministic `CODE(0x…)` text in
  both. Repros boxes-groups/unvbox_lastbox_peel_loop (p1 RED R7: no interline glue, pinned at `[1:h,2:h,3]`),
  boxes-groups/unvbox_splice_boundary (round 2: minipage, `\parbox`, minipage then `\hbox`, `\noindent` + minipage,
  `\unvcopy` twice, a splice in a splice — each one `ltx:p` in round 1), package-bindings/semantic_inference_premises,
  box_primitives_lastbox cases 5-7 and box_primitives_unpack cases 4, 7 RED → GREEN; latexml_oxide/tests/digestion/box.xml
  re-blessed (four lines in one box, not two nested blocks), alignment/arraycr_halign.xml (`tex=` one `\vbox` fewer);
  guards `perfect_kernel_batch68q::{unvbox_lastbox_peel_loop, unvbox_splice_closes_what_its_list_opened,
  semantic_inference_premises}`, `box_primitives::{lastbox, unpack}`, `perfect_kernel_batch58::floatrow_*` (4
  re-pinned: the caption spliced into floatrow's `\vtop` is `vattach="top"`, and a `\floatfoot` after it is a child of
  the caption's `<figure>` panel — the panel opens when the `\vtop`'s captured block is placed, after the marks; text
  order the PDF's; accepted by the main session 2026-10-16). Open: raw Paul Taylor `diagrams` loops (math0301370,
  `\input{taylor_diagrams}`; probe scratch_68q/wm03/d2.tex `$$\begin{diagram} A & \rTo & B \end{diagram}$$`, in both
  binaries): our `end_graf` keeps a line where TeX's paragraph is null (§1096), so the package's `\reformatmatrix`
  `\lastbox` loop keeps finding an hbox — not 68q's. Open RED (68q review side finding 5, not 68q's): floatrow's
  `\floatfoot` in a plain `figure` (not a `\floatbox` object) is dropped by 67int11 and 68q alike, pdflatex prints it
  (captions-floats/floatrow_floatfoot_in_plain_figure; review probes fr2 "Plain foot.", fr3 "TwoFoot"). Open RED (int12
  nets, sigma 0907.2394 errors 0 → 8): easybmat's `\@easy@buildbox` peels the `\halign`'s rows with `\lastbox` after
  `\unvbox`; our alignment is one whatsit, not TeX's row hboxes (§799), so the whole alignment is taken as a row and
  unhboxed → 2 `malformed` per BMAT; the matrix itself was LOST before 68q (the box went back whole, the loop ended at
  once) and is kept now (boxes-groups/easybmat_lastbox_alignment_rows).
  Integrated as int12 with 68p's perf items 1-4 and 6 (gates 5,596/5,596; bundle A/B vs 67int11: 298 random pop67o
  papers tallies identical; 161 fleet `oom:alloc_failed` + OOM-sample papers Fatals 46 → 15, words 6.52 M → 9.00 M, the
  9 with more Errors all Fatal → completed; nets 135/136 0 worse, sigma 0907.2394 the RED above).

- **68i** (binding gaps G3 C, TeX Live package bindings; bindgaps/g23_macros.tsv rows tocbibind, amsopn, datetime,
  rotating, paralist, todonotes, standalone, ccaption, enumitem; each macro checked against the TL 2025 file, or the
  witnesses' shipped classes for enumitem). tocbibind (tocbibind_sty.rs) — `\tocbibname`/`\settocbibname` (:115-132;
  46 papers: 0710.3210, 1106.3946, 1401.4404, 1412.4869, 1505.05782) heading the bibliography where tocbibind sets
  `\bibsection` (:154-198, written as `\<unit>*{\tocbibname}`; the class's name by its chapters, OXIDIZED_DESIGN_DIVERGENCES
  #620), `\settocname`/`\setlofname`/`\setlotname`/`\setindexname` (:124-131; 0710.3210, 2010.07703, 2012.02570),
  `\tocfile{<title>}{<ext>}` (:263-271, a TOC of `<ext>`; 1605.05950, 1712.01817, 2204.09786, 2408.05806, 2409.09967),
  `\tocsection`, `\toc@section`/`\toc@chapter`/`\toc@headstar`, `\toc@start`/`\toc@finish` (:107-114, :250-262,
  :314-316; etoc's tocbibind style, 2412.08175), `\tocotherhead` (:102, formerly empty); `\simplechapter`/`\tocchapter`
  not ported (no witness). amsopn (amsopn_sty.rs) — `\newmcodes@` (:28-42, the pdfTeX branch; 1511.04387, 1712.08220,
  1802.09883, 1809.00159, 1908.01362), `\@declmathop` (:51-53; 2101.09853), `\varliminf@`/`\varlimsup@` (:105-108;
  2005.13511); `\varlim@` was 67's (KNOWN_PERL_ERRORS #696). The row's `\DeclareMathOperator` (21) and `\operatorname`
  (3) are not amsopn's: 1502.03077 (`a msmath` option typo, pdflatex errs too), 2110.09723, 2201.10869, 2208.02596
  (libertinust1math.sty:75 loads amsmath, the libertinust1math binding does not: next payloads' row), 2405.12464 and
  2410.06995 (no amsmath at all, pdflatex errs too); the rest convert clean at 67int9. datetime (latexml_contrib
  datetime_sty.rs; English, monolingual) — the date counts and `\dateformat`/`\THEDAY`/`\THEMONTH`/`\THEYEAR`
  (:564-565, :175-180; 2608.06092), the format selectors verbatim with `\dateseparator`, `\twodigit` and the day of the
  week while `\ifshowdow` (`\showdowtrue`/`\showdowfalse`, `dayofweek`/`nodayofweek`; :75-148; 2209.06700),
  `\ordinaldate` (:64-74, fmtcount's English `\ordinalnum` with `\fmtord`, raised by default, `level`/`raise`;
  1601.00929, 1706.09444, 1804.06840), `\monthname`/`\shortmonthname` via datetime-defaults.sty's English names,
  `\testifleapyear`/`\computedayofyear`/`\computedayofweek`/`\dayofweekname`/`\shortdayofweekname`/
  `\thisdayofweekname` (:613-731; 26 papers: 1308.5288, 1409.5169, 1409.7716, 2112.11386, 2203.14410), `\today` in the
  current format (:733-736) unless ar5iv keeps it `\relax` for an undated source (ruling 64s), `\pdfdate` (:789-829; 17
  papers: 1005.0302, 1203.1927, 1209.4035, 1211.7272, 1212.4375), `iso`'s separators; `\textdate` keeps its
  approximation (fmtcount's spelled numbers), `oclock` and the language modules not ported. rotating (rotating_sty.rs) —
  `\if@rot@twoside`/`\rot@LR` with `figuresleft`/`figuresright` (:31-58; 2306.14600, 2403.02437), `\rotdriver` reading
  its driver and loading none (:64; 42 papers: 0704.0929, 0704.1238, 0704.2885, 0802.2495, 0908.2043;
  OXIDIZED_DESIGN_DIVERGENCES #619). paralist (paralist_sty.rs) — `{inparablank}`/`{asparablank}` under `defblank`
  (:380-399; 15 papers: 1205.4564, 1306.3906, 1801.06340, 1804.00914, 2005.05886; 1707.03997), `\if@empty`,
  `\setdefaultleftmargin` setting the indents, `\defaultleftmargin`, `\defaultitem`, `\defaultenum` (which calls
  `\setdefaultitem`, as the file does) with their deprecation warnings (:75-77, :400-448; 24 papers: 1209.4819,
  1301.6228, 1308.4189, 1403.1528, 1406.4053; 0907.0929, math0106001). todonotes (todonotes_sty.rs) — `\todostyle`
  (:581-583; 27 papers: 2407.14906, 2409.03694, 2410.02385, 2411.05495, 2501.02553), the begin-document `\phantomsection`
  when hyperref has none (:46-49; 12 papers: 1308.6029, 2103.09181, 2208.10521, 2305.03446, 2402.03164),
  `\if@todonotes@disabled` set by `disable` (:159-160; 2606.13443). standalone (standalone_sty.rs) — `\ifstandalone`,
  `\ifstandalonebeamer`, `\onlyifstandalone`, `\IfStandalone` (:26-39; 1310.0809, 2010.10238, 2011.05064, 2208.05046,
  2403.07821), `{standalone}` (:707-709; 2310.18279), `\standaloneconfig` setting `mode`/`subpreambles` (:296-304; 17
  papers: 1905.04614, 1910.10882, 2210.09924; its other keys unexpanded, so 1703.03769's `\buildjobname` is never run),
  and an `\input` sub-file's `\standaloneenv` marking nothing (:802; 2505.21582). ccaption (latexml_contrib
  ccaption_sty.rs) — the options, `\captionwidth`/`\changecaptionwidth`/`\normalcaptionwidth` (:76-82; 24 and 20
  papers: 0803.1137, 0806.4323, 0811.1937, 0905.4349, 1207.5269), `\centerlastline`/`\flushleftright` (:66-72;
  1706.01843), `\precaption`/`\postcaption`/`\midbicaption` (:91-96; 0711.3402, 1001.1088, 1408.3003),
  `\hangcaption`/`\indentcaption`/`\normalcaption` setting their switches, `\bitwonumcaption` (:204-217; 1302.5179),
  `\bionenumcaption`, `\bicaption` in the file's five-argument form (formerly four, keeping one caption) and
  `\bicontcaption` (:219-259), `\contcaption` under the continued float's number (:138-141, formerly the next number),
  and under `[subfigure]` `\subbottom`/`\subtop`, `\contsubbottom`/`\contsubtop`/`\contsubfigure`/`\contsubtable`
  numbering on from the previous float's subfloats and `\subconcluded` (:309-312, :382-430; 0706.2285, 0907.2099,
  0907.2405), the count kept after every subfloat through subfigure_sty.rs's new `\lx@subfloat@done`. enumitem — the
  `{enumerate*}` row (28 papers) is the class bindings loading enumitem without the `inline` their classes give it:
  lmcs (lmcs.cls:37-38; 1611.06929, 1711.09946, 1808.05051, 1904.11378, 2007.10842), Interspeech (Interspeech2024.cls:62,
  Interspeech.cls:69; 2406.16145, 2408.04363, 2505.19937, 2609.16274), INTERSPEECH2021.sty:54 (2104.05980), pnas-new
  (pnas-new.cls:433; 2508.21277) and wlpeerj (wlpeerj.cls:134, which the binding did not load; 2402.14602) now pass it;
  2408.11792's `[in line]` errs in pdflatex too; the row's `\e` (5) is not enumitem's (no definition in the file;
  2110.09723 is libertinust1math's `\DeclareMathOperator`). KNOWN_PERL_ERRORS #800-#807, OXIDIZED_DESIGN_DIVERGENCES
  #619-#620. Repros package-bindings/{tocbibind_names_and_tocfile, amsopn_internals, datetime_names_and_pdfdate,
  rotating_rotdriver, paralist_blank_lists, todonotes_todostyle, standalone_in_main_document,
  ccaption_continued_and_bilingual, enumitem_inline_under_lmcs, enumitem_shortlabels_under_lmcs} RED (67int9) → GREEN;
  guards `perfect_kernel_batch68i` (13). A/B vs 67int9-rel over 502 papers (the rows' 402 witnesses, 100 of pop67o): Errors 705 → 323, papers with
  Errors 380 → 56, Fatals 4 → 4, none gains an Error; per row, Errors: tocbibind 66 → 6, amsopn 46 → 26 (the
  libertinust1math and no-amsmath papers above), datetime 61 → 11 (papers' own: `\THEDAy`, unloaded `\newdateformat`),
  rotating 54 → 10 (unloaded `{sideways}`), paralist 44 → 1, todonotes 40 → 0, standalone 40 → 2, ccaption 57 → 0,
  enumitem 39 → 9; the 100 pop67o byte-identical. Words fall only where `<ERROR>` text left (8 `\newmcodes@` papers:
  1511.04387, 1712.08220, 1802.09883, 1908.01362, 2307.12224, 2309.05133, 2512.10779, 2609.04177) and in 2504.05198,
  whose sub-files' `\onlyifstandalone{\bibliography…}` calls are gobbled as in pdflatex: of the base's four
  bibliographies (152 items) the three sub-file copies of main.bbl go (3 × 38 = 114 duplicate items; the appendix's call
  rendered none), the main one (38 items, before the appendix as in the PDF) stays. Binding nets (tocbibind, amsopn,
  datetime, rotating, paralist, todonotes, standalone, ccaption, enumitem, pnas-new) vs 67int9: no document worse,
  Errors 524 → 471. Rulings (main session, 2026-10-16): `\rotdriver` loading no driver file (OXIDIZED_DESIGN_DIVERGENCES
  #619), tocbibind's name told by the class's chapters (#620), datetime's `\today` in its format with the source date,
  and the class bindings loading enumitem with their classes' `inline` accepted. Round 2: lmcs passes `shortlabels` too
  (the newer lmcs.cls:45-47 in 2003.01491, :51-56 in 2604.16477; the union of the versions), so `[(a)]` numbers "(a)",
  "(b)" (repro package-bindings/enumitem_shortlabels_under_lmcs, guard `enumitem_shortlabels_under_lmcs`); pnas-new's
  and wlpeerj's `inline` is the union with their newer copies (pnas-new v1.44/v1.45 and wlpeerj v1.2 load enumitem
  without it: 1812.09361, 2311.05803, 1710.01085); `\tocsection` numbers `\tocfile`'s heading in a class without
  chapters (pdflatex "1 List of Algorithms"; the bibliography's number is OXIDIZED_DESIGN_DIVERGENCES #620's residual);
  ccaption's load warning names only what stays approximated (`\bottomcaption`, `\legend`, the caption styling);
  subfigure's `\lx@subfloat@done` is empty without ccaption. Open: `\todo[<style>]` takes its style key as the note's
  mark (`mark="mine"`; 19 papers' `\todo[inline]` alike, before 68i too; todonotes_sty.rs:15 `\todo` is `\lx@note{todo}`,
  whose `[<mark>]` takes todonotes' option list; repro package-bindings/todonotes_style_key, RED); enumitem's short
  label numbers a reference without the label's punctuation (`\ref` "a" for pdflatex "(a)"; Perl alike,
  KNOWN_PERL_ERRORS #808; repro package-bindings/enumitem_shortlabel_ref, RED).
- **68j** (binding gaps G3 D, TeX Live package bindings, the tail; bindgaps/g23_macros.tsv rows newpxmath, listings,
  eulervm, nicematrix, nameref, mathalpha, sidecap, tabls, cryptocode, old-arrows, libertinust1math; each macro checked
  against the TL 2025 file). newpxmath (newpxmath_sty.rs) — newtxmath.sty's Palatino twin, so the binding carries the
  newtxmath binding's set (txfonts' symbols, `txfonts_sty::def_txfonts_symbols`, without loading txfonts — newpxmath.sty
  never does, so `\@ifpackageloaded{txfonts}` stays false and `\varv`/`\varw`/`\vary` undefined, as pdflatex — and
  newtxmath's own layer, now `newtxmath_sty::def_newtx_math_symbols`, which re-declares `\coloneq` ≔, `\eqcolon` ≕ and
  `\varg` an italic g unconditionally, as :1541-1542/:1858 and newtxmath.sty:1973-1974/:2238 do; the newtxmath binding
  stops loading txfonts too, keeping its names, KNOWN_PERL_ERRORS #825):
  the upright Greek `\Gammaup`… and `\upGamma`… (:1662-1743; 1709.08558, 1902.06558, 2001.06387, 2305.12589), `\nequiv`
  (:1374), `\fint` (:967), `\mapsfrom` (:1313), `\uppartial` (:1860), `\transp`, `\Angstrom`, `\circleddot`, `\cdotB`,
  `\vv`, and, new in the shared set (newtxmath.sty too), `\laplac` (:745 / newtxmath.sty:1166, U+29E0; 2603.09013) and
  the italic Greek `\deltait`… (:1817-1856 / :2262-2301; 2508.05134); `\widebar` stays newtxmath's only (newpxmath has
  none); amsmath, etoolbox, xkeyval, centernot loaded as newpxmath.sty:23-33 does. listings (listings_sty.rs) —
  `\lst@InterruptModes`/`\lst@ReenterModes`/`\lst@PrintToken`/`\lst@CalcLostSpaceAndOutput` (no mode stack, nothing
  pending), `\lst@whitespacetrue/false`, `\lst@boxtrue/false`, `\lst@label`, `\lst@aboveskip`, `\lst@frametextsep`,
  `\lst@width`, `\lst@FVConvert`, `\lst@Init`/`\lst@DeInit`, `\lst@AddToHook`/`…Exe`/`…AtTop` creating the hooks and
  listings.sty's own `\lsthk@<name>` hooks (mmacells.sty's inline cells, 9 errors each: 1705.02863, 1712.09863,
  1806.04503, 2012.00279, 2012.07851; 2012.14984, 1904.01009, 2210.07578, 2504.10323, 2505.04744, 2608.14263,
  1704.06338). eulervm (eulervm_sty.rs) — `\mathbold` (:58), a bounded bold upright alphabet (27 papers; 0710.3183,
  1111.7285): the kernel's `\DeclareMathAlphabet` makes a switch that runs past its argument (KNOWN_PERL_ERRORS #823,
  open; repro macro-state/declare_math_alphabet_scopes_its_argument RED). nicematrix (nicematrix_sty.rs) — `\iddots`
  when undefined (:146 `\ProvideDocumentCommand`, U+22F0; 26 papers, 2108.05847, 2210.04594), `\Iddots` now ⋰ (was
  `\ddots`), nicematrix's own (`\lx@nicematrix@Iddots`, :1632) whatever a paper made `\iddots`. nameref (nameref_sty.rs) — `\strip@period` (:70; 17 papers, 1310.1737, 1805.08920), `\label@hook` (:88;
  2511.17271), `\T@nameref`/`\@namerefstar` (:361-372; 2404.09236, 2503.18999, 2511.11546); the row's `\N`/`\Z`/`\C`
  are the papers' own. sidecap (sidecap_sty.rs) — `\sidecaptionsep` (:39-40; 1302.0875), its options (`origragged`,
  :118) and ragged2e at `\begin{document}` (:156-172; `\RaggedRight` 18 papers, 1112.5588, 1812.02282; 2406.14861 and
  2009.09454 never load sidecap; golden graphics/sidecap_vpos.xml gains the ragged2e processing instruction). tabls (tabls_sty.rs) — `\tablinesep`, `\arraylinesep`, `\extrarulesep` (:16-18; 22
  papers, 1011.3468, hep-th0309263). old-arrows (raw) — stmaryrd_sty.rs `\stmry@if` is stmaryrd.sty:40's
  (`\let#2=\@undefined\iftrue#1#2`): the stub gobbled `\def\varmapsfrom` so old-arrows.sty:489-492 ran
  `{\varleftarrow\mapsfromchar}` at its load (19 papers, 2006.11831, 2102.02198; 2406.00395, 2406.02375 unchanged).
  libertinust1math (libertinust1math_sty.rs) — loads xkeyval, etoolbox, amsmath (:7, :75; the 68i residual 2110.09723,
  2201.10869, 2208.02596) and defines its own `\mathbcal`, `\mathbfit`, `\widebar`, `\widecheck`, `\lesssim`,
  `\gtrsim`, `\blacksquare`, `\varkappa`, `\eqdef`, `\cupdot`, `\bigcupdot`, `\oiint` when undefined. Not done:
  cryptocode `\@pc@lb` (20 papers; cryptocode.sty:950-957 redefines `\halign` to set it and its line-numbering `\\`, and
  the amsmath binding's alignments never reach `\halign` — also every pseudocode line after the first loses its number,
  silently; needs a kernel mechanism; repro kernel-alignment/cryptocode_halign_redefinition_numbers_lines RED);
  mathalpha `\mathbcal`/`\mathbscr` (25 papers, discounted: the pre-1.14 mathalfa interface — TL 2025 mathalpha.sty:1107
  defines them only under `oldbold`, and TL 2025 pdflatex errs undefined on 1607.07843, 1612.01166, 1612.06674,
  1703.06879, 1709.03549). KNOWN_PERL_ERRORS #820-#825. Repros package-bindings/{newpxmath_symbol_set,
  listings_mode_and_hook_internals, eulervm_mathbold, nicematrix_iddots, nameref_internals, sidecap_ragged2e_and_sep,
  tabls_line_separations, stmaryrd_if_old_arrows, libertinust1math_amsmath_and_names, newtxmath_coloneq_over_mathtools}
  RED (67int10) → GREEN, {newpxmath_coloneq_over_mathtools, nicematrix_iddots_own_definition} RED (68j round 1) →
  GREEN; guards `perfect_kernel_batch68j` (12). A/B vs 67int10-rel over 359 papers (the rows' 265 witnesses, 100 of pop67o, 6 without a
  main file): Errors 1216 → 755, papers with Errors 291 → 88, Fatals 5 → 5, none gains an Error; per row, Errors:
  newpxmath 75 → 4 (not newpxmath's: `\oren`, `\DeclareQuotePunctuation`, `\lst@InstallKeywords`, `\textgreekfont`),
  listings 191 → 2, eulervm 27 → 0, nicematrix 26 → 0, nameref 110 → 74, sidecap 27 → 4, tabls 28 → 0, old-arrows 38 → 0,
  libertinust1math 23 → 0; of the pop67o papers 92 byte-identical, 2 (1803.02290, 1911.12144, sidecap papers) gaining only the
  `<?latexml package="ragged2e" options="NewParameters"?>` line. Words fall by `<ERROR>` text, leaked label keys
  and `=15pt` text, and merged bold letters (34 papers) — except that round 1's newpxmath binding turned `\coloneq` ≔ into
  ":-" (2103.00564 ×34, 2508.05134, 2603.09013, 2212.01087; the txfonts binding's value), fixed in round 2; warnings rise
  only as math parse warnings over formulas whose ERROR atoms are now symbols (2305.12589, 2507.05434, 2605.03875). The
  base binary crashed once (exit 139 in tikz.code.tex) on 2505.23260, clean on a rerun. Binding nets (newpxmath,
  newtxmath, listings, eulervm, nicematrix, nameref, sidecap, tabls) vs 67int10: no document worse. Round 2 re-A/B over
  60 papers (the newpxmath and nicematrix rows' witnesses, the `\coloneq` papers): Errors 67int10 106 → round 1 9 →
  round 2 9; the 6 papers that change from round 1 change only `\coloneq`/`\eqcolon` (":-" → ≔ in 2103.00564 ×34,
  2108.04582 ×33 — undefined under 67int10 —, 2508.05134, 2603.09013, 2212.01087; 2304.05713, newtxmath after
  mathtools, ×93 and ≕ ×2, pre-existing); the newtxmath papers 2303.02628, 2303.11093, 1812.07002 are byte-identical to
  67int10.
- **68b** (binding gaps G3, core package bindings: bindgaps/g23_macros.tsv rows hyperref, breqn, natbib, url,
  blkarray, pict2e, amsthm; each macro verified against the TL 2025 file). hyperref (hyperref_sty.rs) — `\Hy@Info`,
  `\Hy@Warning`, `\Hy@Error`, `\Hy@WarningNoLine`, `\Hy@InfoNoLine` (:130-135; 1310.6851, 1712.03838, 1901.10472),
  `\Hy@ReturnAfterFi`… (:357-361), `\HyperRaiseLinkHook`/`\HyperRaiseLinkDefault`, `\Hy@SaveSpaceFactor`/
  `\Hy@RestoreSpaceFactor` (:2078-2090; 2403.10429, 2501.18381, 2510.01943, 2602.21138), `\thispdfpagelabel`
  (:6312-6314; 1810.05756, 2002.05778), `\if@hyper@item`/`\if@skiphyperref` (:6655-6658; 1806.02596),
  `\hyper@refstepcounter` (:6727-6733 without `\HyCnt@ProvideTheHCounter`/`\Hy@ifnotmeasuring`, the anchor the
  kernel's; 2102.05485, 2303.09820, 2405.07080, 2505.07323), `\hyper@link` (hpdftex.def:396; 1403.6472, 1502.05411,
  1509.00838, 1612.08986, 1907.10062), `\@extra@b@citeb` (:7443; hep-ph0412159, hep-ph0703293, 2202.05188),
  `\hyperpage`/`\hyperindexformat`/`\hyperxindexformat`/ `\nohyperpage` (:7886-7901, :8019-8068, the page numbers as
  text, OXIDIZED_DESIGN_DIVERGENCES #608; 0712.1142, 0804.2154, 0907.3749, 1011.1558, 1310.3213, 1707.07140,
  2312.17745, quant-ph0409125), `\@refstar`/`\@pagerefstar`/ `\@namerefstar` (:8126-8134; 2204.02601),
  `\psdmapshortnames`/`\psdaliasnames` under `psdextra` (psdextra.def:75; 1801.02767, 1710.02246, 1809.07440,
  1908.10510, 1908.03268). natbib (natbib_sty.rs) — `\NAT@biblabel`, `\NAT@biblabelnum` (:622-623; 1409.8404,
  1508.04752, 1604.03470, 1605.02776, 1607.05855, 1610.04415), `\NAT@bibsetnum`/`\NAT@bibsetup` and `\@bibsetup` by
  the style (:627-645, :658-668; 1102.3365, 1103.0594, 1312.5975, 1804.04445, 1911.00031), `\NAT@force@numbers`
  (:974-981, natbib's error under author-year citations, as pdflatex; 2508.10404), `\@extra@b@citeb`/`\@extra@binfo`
  (:1104-1105), `\NAT@open`/`\NAT@close`/`\NAT@sep`/`\NAT@cmt`/`\NAT@aysep`/ `\NAT@yrsep` from the style's values
  (:239-300; 1906.11206), `\ifNAT@numbers`/`\ifNAT@super` set by the options and `\bibpunct` (:235-245, :320-324
  locally, `\bibpunct` :294-297 `\global`ly). breqn (latexml_contrib breqn_sty.rs; completed, not raw-loaded,
  OXIDIZED_DESIGN_DIVERGENCES #607: flexisym's mathcode macros would strip the formulas' meaning) — `{dgroup}` as a
  subequation group, `{dgroup*}` (:1771-1784; 0903.2087, 1008.1366, 1107.4667, 1111.2451, 1201.6197, 1302.6047,
  1303.0187, 1305.0701, 1306.0788), `{dsuspend}` (:2027-2041; 1606.07330), `{dseries}`/`{dseries*}` (:1724-1748;
  1201.6197, 1607.06055, 1706.02708, 1811.11912), the `breqn` keys with `label` (taking `labelprefix`)/`number` and
  `\breqnsetup` (:57, :383-436; 2008.13194), `{dgroup}`'s options (:1793, a `label` the group's; 2607.02409), the
  display parameters with breqn's types (:179-198, `\eqbreakdepth` a count, `\eqindent` the macro `C`,
  `\eqbinoffset`/`\eqdelimoffset`/`\eqstyle`; 2008.13194, 1805.03313), amsmath and graphicx loaded as breqn.sty:47-48
  does; `\Biggg` (1907.10288) is not breqn's with amsmath loaded (:1523-1528). url (url_sty.rs) — `\Url@FormatString`
  (:127-133, text in the url font, OXIDIZED_DESIGN_DIVERGENCES #608; 1902.08702, 1905.04294, 1905.04768, 1908.01711,
  1908.10377, 1910.08562), `\UrlTildeSpecial` (:63; 0907.0729). blkarray (blkarray_sty.rs) — `\BAmulticolumn`
  (:934-947; 1202.0500, 1204.2176, 1301.3189, 1311.0327, 1401.4921, 1407.4836), `\BAnewcolumntype` (:1677-1683, the
  template's `\NC@rewrite@<c>`; 1302.2340, 1402.5950, 1408.0807, 1702.06723, 2005.02853), `\BAextrarowheight` (:1063;
  1312.6738, 1503.04543, 1503.08375), `\BAnoalign` (:1005-1012; 1912.05110). pict2e (pict2e_sty.rs) — `\arc`/`\arc*`
  (:667-684; 1201.4907, 1503.03277, 1503.03676, 1507.03747, 1611.00631, 2001.00020, 0811.3832), and `\pIIe@arc`'s mode
  now pict2e's (0 a line to the arc's start, 1 a new subpath, :643-646; the filled sector starts at its centre and
  closes there once). amsthm (amsthm_sty.rs) — `\ifmeasuring@`, `\iffirstchoice@` (:294-295, where amsmath has not
  made them; setstack.sty's copy of `\text@`: 0803.1550, 0806.2059, 0812.2552, 1001.4940, 1105.1277).
  KNOWN_PERL_ERRORS #750-#753. Repros package-bindings/{hyperref_internals_called_by_papers,
  breqn_display_environments, breqn_group_label_and_parameters, natbib_label_and_list_internals,
  url_format_string_and_tilde, blkarray_public_commands, pict2e_arc, amsthm_firstchoice_conditional}; guards
  `perfect_kernel_batch68b` (8). Not this payload's (verified): hyperref row's `\chapter`, `\xspace`, `\textcolor`,
  `\ding`, `\citeauthoryear`, `\hypersetup`/`\texorpdfstring`/`\hyperlink`/`\nameref`/`\autoref` (called before or
  without `\usepackage{hyperref}`), `\x`/`\y`/`\z`/`\F`/`\For` (the papers' own), `\PrerenderUnicode`/`\unichar`
  (ucs.sty:591, `utf8x`; 50 papers, the inputenc/ucs binding's), `\CJKkern` (CJK), `\flqq`/`\frqq`/`\glqq`/`\grqq`/
  `\cyr` (hyperref sets them only in PDF strings), `\bbl@info` (babel), `\harvarditem` (harvard), `\tocsection`
  (amsart); natbib row's `\bibsep` (2003.02474, 2008.04185: cas-common.sty raw before natbib is loaded),
  `\citeauthoryear`/`\citep`/`\Citet` without natbib, `\N`/`\NA`/`\Doi`/`\bibAnnoteFile` (the papers' or their
  `.bst`'s). Residuals: `\hyper@link{cite}{cite.<key>}` links to hyperref's anchor name, which our bibliography items
  do not carry (`ltx:ref idref="cite.<key>"` dangles; kept, the dangling-links ruling); blkarray's own `>{…}`/`<{…}`
  preamble tokens are read only where array is loaded; amsthm's `\QED@stack`/`\math@qedhere` (a document's own
  `\qedhere` over amsthm's token stack, 1801.02767, 1710.02246, 1809.07440, 1908.10510, 1908.03268) need the QED stack
  as amsthm's macro, not this binding's native one (the proof lock's area, left); under breqn's amsmath an old-form
  `\cases{…\cr}` (1008.1366) is amsmath's "Old form" error in pdflatex too (amsmath.sty:1077-1117 `\matrix@check`,
  which gobbles `\env@cases`; pdflatex then errs on each `&`/`\cr`), and our `\cases`, as Perl's, then leaves its
  alignment open: 1 → 13 errors there, words 11,629 → 16,621 — faithful, no surpass (main-session ruling);
  `{dgroup*}`'s options are read and dropped (a `label` there names the first equation in pdflatex, `\grp@setnumber`
  :1794 not run; no witness); our amsmath binding never defines `\iffirstchoice@` (amstext.sty:43), so setstack with
  amsmath and no amsthm still errs (side finding, not this payload's). Not started (brief rows after the top six):
  amsthm `{proof}`/`\QED@stack`, color/xcolor/graphicx (`\color@block`, `\color@box`, `\definecolor`, `\@testcolor`,
  `\Grot@box@std`, `\Gread@parse@vp`), caption `\DeclareCaptionSubType` and `\captionsetup` keys, the `\Gin@driver`
  root behind trimclip's PGF fallback, cleveref `\crefalias`, the ORCID-URL normalisation of `\lx@add@orcid`.
- **68h** (the 68b tail: caption keys, ORCID URLs, `\crefalias`, `\Gin@driver`). caption (caption_sty.rs) — the
  `\captionsetup` keys and the package options (caption.sty:171-172, the v1 options `bf`, `small`… :114-147) format
  the float captions as caption3.sty:1661-1715 does, through the kernel's new caption hook `\lx@format@caption@@`
  (sect09.rs, longtable_sty.rs): `labelsep` (:774), `labelformat` (:733), `name` (:1347-1350), `figurename`/`tablename`
  (:1336-1345), `textformat` (:791), `font`/`labelfont`/`textfont` and their `+` forms (:842-858), an empty text with
  no separator (:1680-1683); typed `\captionsetup[type]` settings stay the type's (they also set the general keys
  before); `\DeclareCaptionFont` (:830, the predefined fonts :860-950), `\DeclareCaptionTextFormat` (:787) and the
  `endash`/`newline`/`gobble` separators (:776-785) as caption3 defines them; `\DeclareCaptionSubType` (caption3.sty:
  1774-1806; run 336: 39 first errors; 1202.5818, 1604.05412, 2002.03129, 2106.01798, 1706.00437, 2405.16148, 1907.05468,
  2512.12259), moved from subcaption_sty.rs's no-op, its counter made with subcaption's sub-type ids (subcaption and
  subfig register theirs: `\caption@subtypelist`, subfig.sty:261-263 `\sf@counterlist`). OXIDIZED_DESIGN #617. Kernel
  (base_utilities.rs) — `\lx@add@orcid` takes the iD of an orcid.org address (2104.12510; OXIDIZED_DESIGN #616).
  cleveref (cleveref_sty.rs) — raw cleveref's `\crefalias` (cleveref.sty:1277-1278) replaces the no-op; `cref_type`
  reads a type's alias, else its counter's (1810.01223, 1706.03576, 2210.04778, 2506.14950). graphics (graphics_sty.rs)
  — `\Gin@driver` is the driver file the options or graphics.cfg choose (pdftex.def for PDF output): trimclip reads
  tc-pdftex.def and no pgf (2202.10115, 2306.09847, 2507.09618), and adjustbox_sty.rs's `|`-as-other workaround is
  removed. Side fixes the A/B found: bmvc2k_cls.rs defines the class's colours (bmvc2k.cls:268-278, :716;
  2210.10486's `labelfont={color=bmv@captioncolor}`, and its 13 `bmv@sectioncolor` errors), cas_dc_cls.rs loads
  setspace under `[review]` (cas-sc.cls/cas-dc.cls:49, :67; 2512.10464). KNOWN_PERL_ERRORS #795-#797. A/B vs
  67int8-rel over 304 papers (the 39 `\DeclareCaptionSubType` papers, 95 caption-key users, 26 `\crefalias` users, 43
  adjustbox users, 4 ORCID, 100 pop67o): E 2,963 → 2,931, F 5 → 5, error-free 129 → 149; the `\DeclareCaptionSubType`
  papers E 160 → 141, error-free 1 → 20. Binding nets (caption, graphics, graphicx, acmart, cas-sc, ceurart, svmult,
  revtex4/4-1/4-2, bmvc2k) vs 67int7: 0 worse. Repros
  captions-floats/{caption_keys_set_the_label_and_separator, declarecaptionsubtype_numbers_the_subtype},
  sectioning-frontmatter/orcid_given_as_url_is_the_id, package-bindings/{cleveref_crefalias_names_the_type,
  adjustbox_clipping_driver_is_pdftex}; guards `perfect_kernel_batch68h` (8). Faithful error counts that rise:
  `\DeclareCaptionSubType` after subfigure.sty is caption's "counter already defined" error, once per type as in
  pdflatex (1706.00437, 1907.05468: 2 errors, as pdflatex; 1705.07909, 1807.06322). Residuals: sub-captions keep
  their packages' formats (the general keys do not reach them); `\caption*` takes no font and no text format; a sub-type
  `\DeclareCaptionSubType` makes after subcaption has no `{sub<type>}` environment (subcaption.sty:109-111
  `\ForEachCaptionSubType`); `\captionof{subfigure}` in a minipage still reads the caption as the sub-figure's width
  (pre-existing, 2002.04564). Not started: amsthm `\QED@stack`/`\math@qedhere` (the binding's proof begin and end,
  amsthm_sty.rs:236, :297, use its native QED stack: the proof-lock area; 1801.02767, 1710.02246, 1809.07440,
  1908.10510, 1908.03268), color/xcolor/graphicx internals (`\color@block`, `\color@box`, `\@testcolor`,
  `\Grot@box@std`, `\Gread@parse@vp`). Round 2 (review agent_reports/2026-10-16_68h_review.md): the package options of
  caption, subcaption and subfig reach `\captionsetup` as written, and `\ProcessOptions` reads a package's option list
  as latex.ltx:18557's `\protected@edef\@curroptions` (content.rs; it fully expanded it, so `[figurename=\textsc{Fig}]
  {caption}` gave 2 undefined-command errors and a "\edefn sc\selectfontFig 1" label; pdflatex 0, "Fig 1");
  `figurename=`/`tablename=` are added to babel's `\captions<language>` as newfloat.sty:179-188 `\newfloat@setname`
  does (under `[english]{babel}` they were reset to "Figure"); the empty-caption test is caption3.sty:1713-1748
  `\caption@ifempty` (a `\label`, `\index`, `\glossary` or a leading `\ignorespaces` is no text: `\caption{}\label{f}`
  after the hoist, `\caption{\label{f}}`); caption v1's size options replace `font=` (caption.sty:124-129); mrm's
  `\orcid` and apa7's `\addORCIDlink` take an orcid.org address's iD (OXIDIZED_DESIGN #616). More residuals:
  `\captionsetup[longtable]` is not read (longtable's captions take the `table` settings); `\caption{{}}` counts as
  empty (the kernel's label hoist strips the group; pdflatex "Figure 1:"); subcaption's/subfig's separators
  (`labelsep=` under `[sub]`) do not reach the sub-captions. Repros captions-floats/{caption_option_values_unexpanded,
  caption_names_survive_babel, caption_empty_text_ignores_label}, sectioning-frontmatter/orcid_inline_link_takes_the_id;
  guards `perfect_kernel_batch68h` (+5). Round 3 (re-review agent_reports/2026-10-16_68h_r2_review.md): caption's
  v1-keyword match reads `\CurrentOption`'s body (caption_sty.rs `current_option_text`; caption3.sty:374-399 passes
  the list on by `\@expandtwoargs`, a `\protected@edef`), so `\DeclareRobustCommand\rb{\laterdefined}` +
  `[figurename=\rb]{caption}` no longer runs `\rb` at package load (1 Error → 0, pdflatex 0); subfig keeps its expanded
  key read, as subfig.sty:125 and :217 `\edef` the list (pdflatex errors there too). A `\<type>name` of `\protect` and
  a robust or xparse command is a name noun (base_utilities.rs `is_name_noun` follows `\protect\<cs>` hops through
  distinct macros — a hop back to one is a loop, fuse 32 (int10, re-review finding 1: five hops, pdflatex "Ex 1"); it stopped
  at the first, so `\renewcommand\figurename{\protect\rb}` gave the tag "1", pdflatex "Fx 1"; KPE #194). Repros
  captions-floats/{caption_option_key_is_read_unexpanded, protected_robust_name_is_a_name_noun},
  loader/processoptions_reads_as_protected_edef (+ poq68h.sty; the direct `\ProcessOptions` guard: pdflatex's list
  except `[beta=\textbf {x}]` and `[o=\oa ]`, which pdflatex prints with a `\protect`, the kernel's `\textbf` and an
  optional-argument `\newcommand` not being `\protect`-wrapped macros here); guards `perfect_kernel_batch68h` (+3). Integration (int10 nets): with caption's keys in effect, papers' caption setups name their class's colours and fonts, which two class bindings lacked — ieeeaccess `accessblue` (ieeeaccess.cls:121-133, the PANTONE 3015 C spot colour's CMYK alternate 1 0.3 0 0.2, color having no `spotcolor` model) and `\figcapfont` & co. (:75-79; 2405.14505 0 → 9 Errors, 2205.15416 0 → 20), IEEEojcsys `ojcolor` (IEEEojcsys.cls:3346-3348, `\AtBeginDocument`, after loading color) and its caption fonts (:4669-4672; 2206.03535 7 → 14): now defined as the classes do; nets ieeeaccess 0/28 and IEEEojcsys 0/20 worse than 67int9; repros class-bindings/{ieeeaccess,ieeeojcsys}_caption_class_colour, guards `perfect_kernel_batch68h::{ieeeaccess,ieeeojcsys}_caption_class_colour`.
- **subfig passes a key's value unexpanded** (open, latent; 68h r3 re-review finding 4, probe
  scratch_356a0c47/rev68h3/p/s1.tex): subfig.sty:217 and :220 `\edef` the option list before its keys see it, so a
  value macro is expanded at load; since 68h r2 subfig_sty.rs hands the value on as written (OD #617). No visible
  effect today — `labelformat=\lf` with `\lf` redefined after loading prints "(a)" in both, as the sub-caption format
  is fixed — but it is an unruled divergence. Fix: expand the value as subfig's `\edef` does, keeping the key read.
- **A stored `\ProcessOptions` list drops `\protect` before a robust or optional-argument command** (open, low impact;
  68h r3 re-review finding 4): pdflatex's `\protected@edef` (latex.ltx:18557) writes `[beta=\protect \textbf {x}]` and
  `[o=\protect \oa ]`; ours writes `\textbf {x}` and `\oa` — the kernel's `\textbf` and a `\newcommand` with an optional
  argument are not `\protect`-wrapped macros here. Pinned as-is in
  `perfect_kernel_batch68h::processoptions_reads_as_protected_edef` (repro loader/processoptions_reads_as_protected_edef).
- **caption v1 keywords given by a protected or robust macro** (open, small; 68h r3 re-review finding 5, probes
  rev68h3/p/c5.tex, c8.tex): `\protected\def\kw{bf}` with `[\kw]{caption}` makes pdflatex's label bold (keyval's
  `\csname` expands a protected macro), ours stays plain; `\DeclareRobustCommand\kw{bf}` gives pdflatex 2 errors, ours 0.
- **`\ProcessKeyOptions` sees an expanded option list** (open, side finding of the 68h r2 re-review, probe
  scratch_356a0c47/rev68h2/probes/a2.tex): ltkeys' `\ProcessKeyOptions` reads `\@raw@opt@<file>` (latex.ltx:19398,
  :19457-19470), which `\@pass@ptions` stores as the argument tokens (:18521-18525), so pdflatex's
  `\usepackage[name=\textsc{Y}\nm]{myl3}` sets the key to `\textsc {Y}\nm`; ours to `\textsc {Y}Bob`: the loader's
  explicit `[…]` list reaches content.rs `pass_options` already expanded and its raw record is re-tokenized from that
  text (`option_argument_tokens`; `\PassOptionsToPackage` keeps its tokens through `pass_options_with_raw`). Fix: carry
  the `\usepackage`/`\RequirePackage` argument tokens to `pass_options_with_raw`.
- **A local `my<binding>.sty` loads the binding instead** (open, SHARED with Perl, side finding of the 68h r2
  re-review, probe a2.tex): without raw interpretation content.rs `find_file_fallback` (Perl Package.pm:2186-2221
  `FindFile_fallback`, the `rw|my|preprint` prefix) maps `\usepackage{mypgf}` to the pgf binding even though the
  document ships mypgf.sty, and logs "Interpreted as versioned package, falling back to pgf.sty" (content.rs:985-1000,
  the fallback step before the raw one), so the local file's `\showmypgf` is undefined (pdflatex loads mypgf.sty). A
  ruling is needed: keep Perl's binding-wins order (witnesses 1206.0536 mysvjour3 → svjour3, astro-ph0005021), or
  prefer the document's own file over a prefix-stripped fallback (the version-suffix case unchanged).
- **Residual of 68h — trimclip's PGF fall-back under `[dvipdfmx]`/`[dvipdf]`** (open; review
  agent_reports/2026-10-16_68h_review.md finding 8): adjustbox ships tc-pdftex.def, tc-dvips.def, tc-xetex.def and
  tc-luatex.def only, so with `\Gin@driver` = dvipdfmx.def/dvipdf.def trimclip.sty:252-258 inputs tc-pgf.def and pgf,
  and `\clipbox{1pt 2pt 3pt 4pt}{Clipped text}` becomes an `ltx:picture` (an svg:clipPath over the box, the text only
  in an svg:foreignObject) where pdftex/dvips give an `ltx:text`; pdflatex with `[dvipdfmx]` reads tc-pgf.def too
  (the content is the clip). Before 68h every driver mode took this path. Probe
  scratch_356a0c47/rev68h/p3/g_dvipdfmx.tex.
- **67b** (frequency-first): a kluwer.cls binding (`latexml_contrib/src/kluwer_cls.rs`; the class is in TeX Live, so
  these papers fell back to OmniBus): `{article}`, `{opening}` (its end makes the title, kluwer.cls:707-738), one
  author per `\author` with an `\institute` going to the authors since the previous one (:766-769), `\surname`,
  `\newdisplay`, `\newproof`/`{pf}`, the frontmatter setters. Run 349: `{article}` 384, `{opening}` 344, `\surname`
  335, `\newdisplay` 128 papers; 40-paper A/B E 169 → 58, error-free 0 → 24, none worse
  (`perfect_kernel_batch67b`). Open: `\author{A (1) and B (2)}` with parenthesised marks stays one creator (the
  merged_creators cluster).
- **67a** (frequency-first; ruled 2026-10-14 "provide-only fallback"): under the ar5iv preload a paper that uses
  `\url`/`\href` with neither url.sty nor hyperref loaded gets hyperref's definitions at `\begin{document}`, as arXiv's
  build (which loads hyperref into every paper) gives them — only the ones still undefined (OD #588;
  `perfect_kernel_batch67a`). Cluster: 812 `\url` + 473 `\href` papers live in run 349.
- **66z** (frequency-first, ruled 2026-10-14: the image-conversion Error cluster, ~1,240 papers live in run 349): a
  MetaPost figure (`prologues:=0`, fonts left to dvips in `%*Font:` comments) is converted from a copy holding the fonts
  dvips would download — Type1 programs via `pdftex.map` + kpathsea, TeX names bound, MetaPost's `fshow` (OD #587;
  `latexml_post/src/postscript_fonts.rs`). 40/40 sampled failing figures were MetaPost (≈2/3 of the cluster's papers);
  12-witness HTML A/B: 10 papers E → 0 (math0102124 38 figures, cs0005010 34), none worse. Rest of the cluster:
  dvips-prolog fragments (`startTexFig`, `@endspecial`, `xxxprolog.ps` blocked by `-dSAFER`), re-encoded fonts (`ptmr8r`),
  `.eps` files that are TeX sources.
- **66y** (K1 roots, ruled 2026-10-14 "park K1, fix the roots"): an `array` outside math is an inline formula of its
  own, its cells math as latex.ltx `\@arrayclassz` sets them (1611.02392 E 24 → 0; KPE #677), and `\raisebox` in a
  picture reads its box in text mode (0902.1321 E 101 + F → 0; KPE #678); `perfect_kernel_batch66y`. The other K1
  roots (`\cite*` under the locked `\cite`, qtree, gtart) are one-paper causes, parked under the frequency-first rule.
- **66x** (K9 study row L, ruled 2026-10-14 "fix the identified mode bugs, Perl wording"): the `\lx@list` frame of
  `\list`/`\@trivlist` is marked a bare environment's, so a `}`/`\endgroup` meeting it ends the list (an Info) and then
  its own group — iopconf `\references`, jT `resume`, INFORMS `\Trivlist` no longer swallow the rest of the document into
  the last item (0902.3464: 28 sections inside list items → 0, 1611.07992 9 → 0, 2106.10067 19 → 11; 5 witnesses E → 0; KPE
  #676, OD #586; `perfect_kernel_batch66x`). Row B (a preamble `\begingroup` around
  `\begin{document}`, ~38 papers, 1 Error, content kept) stays open: faithful unwinding pops the document frame and
  the author's group beneath it, then re-establishes the document frame (latex.ltx:9490 `\document`'s `\endgroup`).
- **66w** (K1 cluster of run 345, ruled 2026-10-14: narrow OD #553): a `\par` read in an inline formula looks ahead,
  unexpanded, to the next `$` (stomach.rs `formula_continues_past_par`, `gullet::peek_upcoming_tokens` over a scratch
  mouth): a math-only token before it (a script mark, a DefMath symbol, a requireMath definition, tex.web §1046's
  math-only primitives) keeps the formula open across the blank line, the `\par` reverting as nothing (Perl's content);
  otherwise §1047 ends it as 65n did. hep-th9402135 101 errors + Fatal TooManyErrors → 22 errors, 0 Fatal (eleven split
  formulas whole; left: `$\bull$` with a blank line in `\bull`'s body, as pdflatex, and `\References` non-long `\par`
  errors); hep-ph9809328, hep-ph9802283, hep-ph9706250, hep-ph9706246, 0903.4382 fewer errors or set-right formulas,
  still capped elsewhere; the 65n witnesses (2007.11971, 1710.02165, 2008.11762, 1210.0368, 1907.09124, math9805003,
  hep-lat9811036, nucl-th0401019) and the other K1 papers byte-identical (39-paper A/B vs 66t2). KPE #679; guards
  `perfect_kernel_batch66w::*`. Round 1 (review blocker): any doubt ends the formula — a command's `*`, `[…]` and
  `{…}` arguments are not looked into (`\cite[p.~2]{smith_2001}`, `\ref*{eq_1}`, `\citep[e.g.][]{a_b}`), a command
  it cannot follow (verbatim/semi-verbatim/delimited parameter, a code-bodied macro such as `\verb`, one reached
  through a macro's body such as `\lstinline`) or a stop by meaning (`\medskip`'s `\vskip`, `&`, `\cr`) gives up
  (`stomach.rs lookahead_command`); the look-ahead is lazy and bounded (2,000 tokens, 16,000 characters), and starts
  after the judged `\par` (an `\obeylines` line end continues as a blank line does: hep-ph9809328, hep-ph9706250).
  Not done:
  §1046's re-opening itself (66i draft, parked).
- **66v** (66m E1, ruled 2026-10-14): the floatrow binding records float.sty/rotfloat.sty loaded, as floatrow.sty:34-37
  marks them (`\ver@float.sty`), so a later `\usepackage{float}` no longer replaces floatrow's `\float@end` (2010.00977,
  2003.11778, 2011.10793, 2105.04869; KPE #668; `perfect_kernel_batch66v::floatrow_marks_float_loaded`). No general
  loader `\ver@` rule (it kept out memoir's and floatrow's semantic-layer bindings).
- **66t** (K9 study row H, 54 papers losing their whole body): a bare `\endX` ends the mode only when the top frame
  is an environment's (any: amsmath ends `\equation*` with `\endequation`); a box's frame gets one Warning and no frame work, as
  LaTeX's list-ending `\endquotation` does none — an IEEE Open Journal `\IEEEkeywords` `\vbox\bgroup` box ended by
  `\par\endquotation\egroup` no longer loses its frame and the body (2301.00752 words 0→16,663, 2602.19492 0→6,164;
  KPE #674, OD #584; `perfect_kernel_batch66t`). K9 Stage 1's general frame-ownership rule and Stage 2 stay open.
- **66s** (K9 study row G, 74 papers, all 2511+): a Semiverbatim identifier field's TeX escapes and grouping braces
  are not part of the identifier (bibtex.rs `identifier_data`): MathSciNet's DOI `…3\{_}10` is balanced for BibTeX
  (it counts the escaped brace) but the Semiverbatim reader took `\{` for a control symbol and met a stray `}`;
  `\{_}`/`{\_}`/`\_` now read `_`, the DOI link matches the entry's `url`; values holding a control word (`\href`)
  pass as before (2601.01900 E 2→0, 2511.01040 E 6→0, 2601.01772 E 2→0;
  `06_cluster_bibliography::bib_doi_with_escaped_brace`).
- **66r** (two quick wins from the K9 mode-switch study, `agent_reports/2026-10-14_k9_mode_switch_study.md`): the
  contrib emlines binding's `\emline` lost its two stray `}` (Rust-only typo; ~40 papers, 0911.1421 had 78 Errors), and
  `\twocolumn[…]` tests its header in Rust instead of inside an `\ifx`, so a header spanning the document's own
  conditional (achemso, price.sty; ~46 papers) keeps it balanced (KPE #672; `perfect_kernel_batch66r`). The study's
  K9 Stage 1 (H, 54 papers losing their whole body) / Stage 2 (L 126, B 38) design and six ruling questions are open.
- **66q** (K1 report row W2): natbib's internals `\NAT@citetp`/`\NAT@citexnum` defined over the binding's citation
  commands, selected by `\NAT@ctype`/`\ifNAT@swa`/`\ifNAT@par`/`\ifNAT@full` (natbib.sty:685-749), so a document's own
  citation commands built on them cite (2104.01159 E 35→0 with its 62 bibitems back, 2105.06268 E 10→0; KPE #671;
  `perfect_kernel_batch66q::natbib_internals_serve_user_cite_commands`).
- **66o** (`\endgroup` cluster E3): a natbib `\bibitem` label piece's math shifts are balanced within its brace
  groups (1208.0605's `{Mantz}, $\, {Ebeling}, \& {Rapetti}`; revtex's `\citenamefont {Sasaki$}`, 1205.3745,
  1508.02141) — natbib.sty:818-823 only writes the label to the .aux, but the binding typesets it as tags, and the open
  math ran past the `\bibitem` into the bibliography; 6/6 witnesses lose that error (KPE #670;
  `perfect_kernel_batch66o::natbib_unbalanced_dollar_in_label`).
- **66n** (root-causer on 2308.07459's raw ytableau): an `\hrule` is an alignment's border only between its rows;
  in an open cell it is the cell's (its box's) material — Perl adds a border row and `add_line` restarts the row's
  columns, so every ytableau row with more than two cells lost its middle cells, and after a `\none` cell the next cell
  left math (KPE #669; `perfect_kernel_batch66n::rule_in_a_cell_box_keeps_the_row`).
- **66m** (the low-risk mechanisms of the arXiv `\endgroup` cluster, report
  `agent_reports/2026-10-14_endgroup_cluster.md`): the algorithm2e binding no longer loads float, which algorithm2e.sty
  does not, so floatrow's `\float@end` stays (1811.04370, 2007.08784, 1609.05867, 1910.09976, 1807.05059; OD #580);
  `\lstinline` robust, so ifthen's `\equal` keeps it unexpanded (1406.5453, 1605.05104; OD #581); IEEEtran's
  convenience `\keywords` a `Fallback` the document's `\newcommand` replaces (2006.10005, 2008.05854, 2011.04315; OD
  #582, KPE #664); `{plate}` undefined for AASTeX 6+ and emulateapj, whose classes have none (1909.00812, 2006.16258,
  2305.06275; KPE #666); emulateapj's `\enddeluxetable` closes a still-open data (1506.07185, 1705.08932; KPE #665); a
  brace-less `\abstract`'s capture frame is a bare environment's, so an enclosing `\endgroup` ends it (2305.17823; KPE
  #667). Guards `perfect_kernel_batch66m`. The loader's `\ver@` half was ruled 2026-10-14 (floatrow's marks only, no
  general loader rule) and landed as 66v. Open from the same report: E2, E7, E10,
  E11 (E3: 66o).
- **66l** (the rest of K1's wrong-mode roots from the same report): array.sty `w`/`W` columns keep the alignment's
  mode, as array.sty:463-474 (2407.14297; KPE #660); `\lx@marginpar` digests in `inline_internal_vertical`, so a
  `$…$` note called from a formula is a formula (2002.06461; KPE #661); forest's `content format` (`plain content`,
  `math content`: forest.sty:3944-3946) is applied to a node's content (2305.14297); what OmniBus requires as a guess
  (inst_support, aas_macros) loads `Fallback`, so a document's `\newcommand{\inst}` wins (2603.21740, cas-sc; OD #408);
  an eqnarray.sty binding maps `equationarray[pos]{cols}` onto the eqnarray alignment over the specification's columns,
  cells `$\displaystyle…$` (1605.07858, 2507.02169); a `\multicolumn` cell of an amsmath math alignment (`aligned`,
  `gathered`, `split`, `multline`: `template_math`) is text, as `\omit` drops the template's `$` (1808.07192; KPE #662). Open, shared with Perl (KPE #663): a mid-paragraph `\footnote` or (since 66l) `\marginpar` in a measured
  `\vbox` counts as two lines — the parameter digester's mode repair repacks the line around the note
  (`notes_measure_as_their_mark` case 3 pinned at 18.83pt).
  Guards `perfect_kernel_batch66l::*`. Open: `\opt@` string re-tokenized (2407.02071), ytableau after `\omit`
  (2308.07459), two xy-pic gaps (1012.0388, 1103.1294); eqnarray.sty `\eqnnum`/`\yesnumber`, fleqn and `[pos]` not
  modelled.
- **66k** (K1's word-loss roots, root-causer report `agent_reports/2026-10-14_k1_wordloss_triggers.md`: in 19 of the
  20 papers our engine read the `_`/`^` in text where pdflatex reads it in math): `\cancel`/`\bcancel`/`\xcancel` are
  robust, as cancel.sty:70-78, so an aligned cell's start (§791) does not expand them before the template's `$`
  (1801.01765, 2404.04195, 2505.00286, 2505.10361, 2512.10194); pstricks keeps graphics' boxing `\scalebox` (2104.00361,
  2504.03865; KPE #658; 1208.6481 still 0 errors); graphicx's `alt`/`actualtext` are read undigested, as graphicx.sty
  never typesets them (2605.23217, KPE #659). Guards `perfect_kernel_batch66k::*`. The report's other roots: 66l.
- **66j** (2605/2606 sandbox regressions, root-causer reports `agent_reports/2026-10-13_sbx2605_regressions.md`,
  `…_sbx2606_regressions.md`): an option no `\DeclareOption` declared runs its `\ds@<option>` when defined, latex.ltx's
  `\@process@pti@ns` (C12; 2.09-style classes: gji.cls 2605.19951, biom.cls `usenatbib` 2606.12015; KPE #657); revtex's
  `\@AF@join` for a class's own `\@email` (C11, JASA.cls: 2605.18495, 2605.23416; KPE #655); `\mathcolor` adds no group
  and restores the color it found, as mathcolor.ltx:20-29, so a `\left` and `\right` in two calls pair (C7, 2605.11792);
  the loader's expl3 scope-exit cleanup fires on expl3's own `\l__kernel_expl_bool`, not on `_` being a letter, so a
  package that restores its own catcodes keeps them (N6, etextools: `/` stayed 8; 2606.06323). Review r1: the `\ds@`
  lookup is the kernel `\ProcessOptions`'s only (not the key=value processors'); option handlers a binding defines but
  never processes are retired as processed ones are — REVTeX 3's under a revtex4 class (`retire_option_handlers`; a later
  package's `[draft]` ran revtex's no-op), hyperref's and minted's key names — and preview/floatflt/floatfig process
  their options as their packages do. Guards `perfect_kernel_batch66j::*`. RED: `loader/etextools_translations_begin_document` (2606.06323's second error,
  etextools + translations at `\begin{document}`); C3 (nomencl in a float) still open.
- **66g** (payload, 2605/2606 cluster C5): an input's end closes only a `\left` group opened in that input — `\left`
  records the input it was read in (`gullet::reading_context`), so physics' `\left( \lx@xmarg{…}{#1} \right)` keeps its
  pair when `#1` (`\cosh(a\over 2)`, `\expval{a \over b}` in `aligned`) ends (OD #530; 2605.02768, 2606.01486 E 1 → 0;
  65d's witnesses unchanged). Guards `perfect_kernel_batch66g::*`.
- **66h** (run 345's Error → Fatal regressions, 118 confirmed against 64zf, root-causer report
  `agent_reports/2026-10-14_ef345_regressions.md`, per-paper `~/data/pk_agents/main/ef344/attribution.tsv`; and three
  2605/2606 sandbox regressions): the caption wrapper's branch is a control sequence, not a braced argument, so a
  one-token box argument's `}` no longer splits it (K3, 16 of 17: astro-ph0107461 and more back to their pre-66d Errors,
  OD #570); `\right.` is inserted once per `\left` group (OD #530) and svjour3's `\institute` drops its `\at` sentinel (K5a,
  9: 1404.4148, 1705.07573, 1911.12144, 1912.03170, 2107.00750, 2107.11151, 2205.01215, 2210.16010, 2305.18835; KPE
  #652); the AMS-TeX front matter is amsppt's (K2b, 7 AGT papers: math0103177; KPE #650); amstex.tex's `\RIfM@`/`\RIfMIfI@`
  (K2c, 6 of 7 Michor diag.tex papers: math9611223; KPE #651); revtex `\@email`/`\@homepage` (K4a, 7: 0710.1022,
  1201.4230; KPE #655); amsrefs `\strip@MRprefix`/`\MRhref`, `msc-links` loading hyperref and `{bibchapter}` (K4b/K4c, 6:
  0806.1269, 1612.01009, 1706.05540; KPE #656); lipics loads thm-restate only on its option, read from the class's own
  option list too (K4d, 2604.22493; 2211.04601 still restatable), and follows the class's `autoref` (aliascnt sibling
  counters unless `thm-restate`, :1119-1236) and `numberwithinsect` options (2604.22493's keytheorems
  `sibling=corollary`: 6 Errors → 0; over the 36 lipics papers in the A/B population only it changed counts, words
  identical; `\autoref` names the sibling, "Proposition 2" for "Theorem 2" in 2103.15659; tags follow
  `numberwithinsect`, "Problem 2.1" for "Problem 1" in 2406.18299). From the sandboxes: the article, report and book
  bindings define `\fnum@figure`/`\fnum@table` as the composed tag (2605.29738 Fatal → 0, OD #579, KPE #654); revtex
  `\frontmatter@footnote` (2605.04155, KPE #655); float.sty's `\newfloat` a macro scrhack can patch
  (2606.04906, 2606.11358; KPE #653). Guards `perfect_kernel_batch66h::*`; A/B over pop.tsv (1,002 papers)
  byte-identical in every fingerprint, and over the
  E → F papers with a main file (75 of 121): papers with a Fatal 72 → 34, words 660k → 1.12M. Open from the report: K6
  (2601.10843, 2602.04048): `\start@align` re-enters a re-dressed `{alignat}`; starting the binding's own alignment
  instead (66h r1) only turned the Fatal into an Error over the same truncated document (review 66h: an empty
  `<document/>` on the repro), so it was reverted — `\start@align` must open the alignment in a group whose `\egroup` ends
  it, with `\black@`/`\totwidth@` (amsmath.sty:1364/:1316) defined (RED `alignment/start_align_under_redressed_alignat`);
  K1 (31, 65n's
  §1047 `\par`-closes-math without §1046's math-token-reopens-math: `$` parity flips; repro
  `paragraph-mode/script_in_text_after_par_closed_formula_reenters_math` GREEN by 66w, OD #553 narrowed: the `\par`
  keeps the formula open when math follows it. Batch 66i, §1046 for sub/superscript marks,
  is parked: over 1,235 A/B papers errors 9,542 → 8,361 and Fatals 40 → 32, but 20 papers lost >1 % of their words
  where an earlier wrong-mode bug puts a script in text — 2104.00361 `\scalebox{1}{$b_1$}` in `\substack` (the inner `$`
  closes the outer math), several at binding-built strings — so those go first; root-causer report
  `agent_reports/2026-10-14_k1_wordloss_triggers.md`),
  K2a (19, missing macro files: shared), K2d (4, Paul Taylor's diagrams.tex not in TL, now converting 2k-11k words; a binding since 67d), K5b
  (2, L3 papers' unshipped physics.sty), 5 singletons; 1309.7562's `$$` meeting an open `{` (13 Errors on every binary,
  pdflatex 1); `\bibselect` does not read a `filecontents` `.ltb` (`Info:missing_file`).
- **63b** (landed): nameref's begin-document reset of `\ref`/`\pageref`/`\Ref` under its own hook label (KPE #519;
  2503.08060, 1908.01329, 1811.01873, which also needed nameref's `\NR@setref`), a nested alignment's rows run no
  enclosing row hook (KPE #518; arrays, `\halign`, TikZ matrices, tabbing), the 63a tag `~` binding narrowed to the
  tags a reference prints,
  and the stomach loop detector tells register values apart by amount (Rust-only: `digested.rs` `fingerprint_into`
  hashed a `RegisterValue` by kind, so PiCTeX's dots, each `\raise`d differently, read as a loop; 0801.0709). Guards
  `perfect_kernel_batch63::{preamble_ref_redefinition_reset_at_begin_document,
  begin_document_ref_redefinition_survives_nameref, nested_alignment_inherits_no_eqnarray_row_hook,
  pictex_finite_dots_are_not_a_loop}` and `digested::tests::cycle_fingerprint_distinguishes_register_amounts`;
  trade-off in OXIDIZED_DESIGN_DIVERGENCES #326.
- **63l**: author-line residuals of fidelity audit round 2, all Perl-origin: "~and" and `{\ and}` separate authors
  (KPE #532; nlin0101056, 1508.01140, both a merged_creators error before); `\fnmsep` leaves no comma in a name
  (KPE #533; astro-ph0001054, astro-ph0611016); `\IEEEmembership` is the author's membership contact (KPE #534;
  2306.15457, 2508.00603, 2408.00327; name lines only, not in conference mode unless `\IEEEoverridecommandlockouts`, a document's own definition kept). A/B
  540 papers vs 63k (80 `\IEEEmembership` papers of 2408 among them): 65 change, 0 more errors or warnings (57 gain
  grades only, none on another author). Open from the round: entics.cls `\thanksref{ALL}` prints its key (2212.08177, a raw-class
  two-pass mark), an.cls `\abstract{…}` a plain paragraph (astro-ph0611016), llncs `\author{A\\Univ\\\email{x}}`
  reads Univ as a person. Guards `perfect_kernel_batch63::{author_and_tied_or_grouped_splits,
  author_fnmsep_leaves_no_comma, ieee_membership_is_a_contact, ieee_membership_in_a_block_list,
  ieee_membership_outside_the_names_stays_dropped, ieee_membership_own_definition_kept,
  ieee_membership_conference_override}`.
- **63k**: titles' refs and JHEP-family authors (fidelity audit round 2):
  - RUST-ONLY: a `\ref`/`\eqref`/`\autoref`/`\cref`/`\nameref` in a section title was empty wherever the title is
    reused (TOC entries, `title=` tooltips): Perl's `CrossRef::fillInTitle` (CrossRef.pm:882-904) was never ported.
    `crossref.rs` now fills a copy of the stored title before each use (`entry_text_children`, `filled_value_text`),
    the ObjectDB staying read-only; a title whose refs lead back to itself is used as stored the second time (Perl
    recurses without end). 1011.3492, 1111.3672. Open: a `\cite` in a title reads empty in the tooltip ("cite []";
    the TOC is filled by `fill_in_bibrefs`).
  - RUST-ONLY: `\nameref` showed the target title's number ("2 Target"); Perl drops the title's tag
    (CrossRef.pm:774-777). A `\nameref` nested in a title drops it too (KPE #531, where Perl keeps it).
  - RUST-ONLY: a display copy (TOC entry, ref text) named its `xml:` attributes bare: a `\foreignlanguage` title's
    copy was `<text lang="de">`, which the schema refuses (`PostDocument::clone_subtree`).
  - JHEP/JHEP3/PoS `\author` appends, one author per call (KPE #529; astro-ph0611258 keeps its three authors).
  - An author block's email line is the email, its label the contact's `name` (the class's own name when the author
    wrote none: amsart's "Email address: "); a group's shared line gives each
    address to the name it spells (KPE #530; 0911.0568, 0911.0082). A `$^a$` inside a `\footnote`/`\thanks` no
    longer turns an author block into a marked list (`has_author_marks`; 0911.0568's affiliation line read as a
    person). Open: a group's shared affiliation still goes to its last name only.

  A/B vs 63j: XML 454 papers, 51 change with 0 more errors or warnings (JHEP/PoS papers gain their authors:
  0711.2174 1 → 16, 0811.2355 1 → 15, 1211.0565 1 → 14); HTML 151 papers, 22 change, 0 worse (empty TOC refs 1011.3492
  4 → 0; tooltips fill their refs, "Proof of Theorem 3.2." in 0909.5610, and a title's `\\` reads as a space as in
  Perl). Guards `perfect_kernel_batch63::{ref_in_section_title_fills_toc_and_tooltip, nameref_drops_the_section_number,
  jhep_author_calls_accumulate, author_email_line_label_is_its_name, author_class_email_line_keeps_its_name,
  toc_copy_keeps_its_language}`.
- **63j**: three silent classes from fidelity audit round 2, all Perl-origin:
  - numbered eqnarray, align and gather rows holding only `\label` keep their number and label (KPE #526). Before,
    the row was dropped and the `\ref` printed "( )". hep-ph0208046: 12 missing labels → 0, equation numbers gap-free 1-59;
    hep-th9608109 7 → 0; 1011.4399, 1305.3072, hep-th9412215 all resolve.
  - algorithmic's `\REQUIRE`/`\ENSURE` show their labels (Require:/Input:, Ensure:) rather than "0:", and
    algpseudocode's `\Require`/`\Ensure` theirs rather than line numbers 1, 2; `\Statex`/`\item[]` lines go
    unnumbered as pdflatex prints them (KPE #527; 2201.01230, 1406.5162, 1811.08330, 2507.12875, 2602.10387,
    2309.03769, 1606.06256). Open: algpseudocodex's contrib binding still drops the label (RED repro
    list-structure/algpseudocodex_require_shows_its_label).
  - " and~" separates authors (KPE #528; 2011.10474, 2408.09035, 1406.6147).

  A/B 388 papers vs 63i: 19 change, each one of these, with 0 more errors or warnings. Guards
  `perfect_kernel_batch63::{eqnarray_empty_labelled_row_keeps_number, align_empty_labelled_row_keeps_number,
  unnumbered_empty_labelled_row_stays_unresolved, algorithmic_labelled_item_shows_its_label,
  algpseudocode_require_shows_its_label, algpseudocode_statex_keeps_its_bracket_text,
  algorithmic_empty_and_bracketed_item_labels, author_and_tied_to_the_last_name_splits}`.
- **63i**: affiliations linked by the marks authors show (OD #159, KPE #525). An institute labelled by a leading
  letter or number mark goes to the authors whose names show it. revtex lists all the authors first, then the marked
  `\affiliation`s (2011.01984, 2301.08449, 1510.01235, 0808.2763, 1205.4587, 1509.05545, 2601.16864). svjour3 marked
  list pieces (1406.6147, 2405.03406, 1709.04623, 2003.05787) now link per author. Before, every group author had every
  affiliation, or all were shared. Interleaved revtex groups keep the group rule, and an aa numbered-lines
  `\institute` stays shared (astro-ph0305539). A/B 384 papers vs 63h: 11 change, all to the marks; 0 more errors or
  warnings. Guards `perfect_kernel_batch63::{revtex_marked_affiliations_link_by_mark,
  svjour3_marked_institutes_link_by_mark, institute_with_numbered_lines_is_no_marked_list,
  revtex_interleaved_mark_with_symbol_keeps_the_group_rule, revtex_interleaved_group_with_unanswered_mark,
  revtex_affiliation_with_inner_superscript_is_no_marked_line, revtex_marked_affiliations_with_symbol_marks_link_by_mark,
  svjour3_marked_institutes_with_thanksref_link_by_mark, svjour3_symbol_marked_institute_pairs_by_position,
  revtex_mathrm_mark_interleaved_group_keeps_the_group_rule, revtex_mathrm_marked_affiliations_link_by_mark,
  revtex_textsuperscript_interleaved_group_keeps_the_group_rule, revtex_interleaved_typo_mark_keeps_the_group_rule}`
  (marks read alike on both sides; a line's first mark labels it). Open RED: affiliation_line_inner_superscript_is_its_text.
  Fidelity audit round 2 (80 papers, 63h5): authors 65/69 right
  (round 1 52/73); next classes in HANDOFF (eqnarray label-only row, IEEE "and~", `\ref` in a TOC title, PoS
  `\author`, algorithmic labels).
- **63h**: a labelled annotation relocated into a title (the AAS title footnote `\title{..\altaffilmark{1}}` +
  `\altaffiltext{1}`, 0704.0478, 1001.2402; elsarticle `\tnoteref`/`\tnotetext`) is the title's frontmatter note, not a
  schema-invalid `<contact>` (KPE #524, OD #461; both papers 1 → 0 jing errors). Orphaned frontmatter annotations go
  where the evidence names (OD #159):
  - svjour3 `\at` institutes go by surname to every author they name, in any order (a shared institute's e-mail
    stays shared);
  - as many numbered institutes as authors pair by position;
  - a sole author's orphans are that author's;
  - otherwise they are shared, and none is dropped;
  - the prefix-stripped label fallback serves creators only.

  svjour3's unlabelled institute pieces are numbered and its `\author` pieces split at " and " (OD #460).
  A/B vs 63g:
  - 64 svjour3 papers: 34 change and none is worse; a script checked each `\at` link against its source. Changes:
    - stranded institutes reach their authors (1205.2208, 1811.03214, 2105.03148, …) or become shared;
    - 3 merged creators split (1406.5162, 2105.12728, 1709.07063 errors 1 → 0).
  - 318 other papers: only the two title papers and kluwer astro-ph0302424 (its sole author keeps the institute)
    change. No paper has more errors or warnings.

  Guards `perfect_kernel_batch63::{aastex_title_altaffilmark_is_a_note, elsarticle_title_tnote_is_a_note,
  svjour3_epj_one_institute_shared, svjour3_sole_author_keeps_the_institute, llncs_sole_author_keeps_the_institute,
  svjour3_two_institutes_pair_by_position, svjour3_at_institutes_link_by_surname_in_any_order,
  svjour3_at_piece_naming_several_authors, svjour3_name_only_piece_keeps_the_institute_shared,
  elsarticle_lettered_addresses_no_one_cites_stay_shared, author_svjour3_and_piece_splits,
  elsarticle_orphan_fntext_stays_off_the_title}`. Open RED: svjour3_marked_institutes_link_by_mark,
  svjour3_name_only_piece_shares_the_next_institute, author_svjour3_mbox_list_splits,
  author_collaboration_name_with_and_stays_one.
- **63g**: svjour3's EPJ style (svepjc3.clo) links an author to its institutes and notes by key (OD #460) —
  `\thanksref{addr1,e1}` requests them, an institute's `\label` labels its affiliation, `\thankstext{e1}` is a labelled
  thanks note. Before, the keys printed as marks and nothing linked (2304.02920; also 2406.12029, 2406.12545, whose
  e-mail notes now reach their authors). A title's `\thanksref` note stays in the title. An institute's own
  `\thanksref` keeps its note, which goes to a trailing creator when no author cites it (OD #159). An amsrefs `biblist`
  prints all its entries, cited or not, each bibliography its own (OD #57; 1012.2719 29 → 34 references). Guards
  `perfect_kernel_batch63::svjour3_epj_*`, `06_cluster_bibliography::amsrefs_*`. The revtex shape of the same audit row
  (2011.01984, 2301.08449: all `\author`s, then `$^{a}$`-marked `\affiliation`s) is the marked-affiliation linking still
  open for per-author classes (OD #459). Open RED repro: index-bib/amsrefs_case_distinct_keys_both_kept (two `\bib`
  keys differing only in case merge into one entry, silently).
- **63f**: frontmatter fidelity follow-ups (run-336 audit): acmart's `\received` keeps its whole history — received,
  revised (several), labelled — each its own date via the frontmatter key `accumulate` (KPE #522; 2307.05988); initials
  glued to a surname (`A.G.Bogdanchikov`, `Yu.M.Shatunov`; not `St.`/`Mt.`/`Ft.`) read as a name and a `\vspace`'s length
  is no name text (hep-ex0105093 5 → 38 creators; hep-ph9306209 recovers D.P. Roy and Sridhar K.); a name's closing
  initial (one capital) or suffix keeps its period (KPE #523). A/B 318 papers vs 63e: 3 papers change, each a recovery
  (hep-ex0105093, hep-ph9306209, 2307.05988). Next (63g): amsrefs inline biblist kept whole
  (1012.2719), svjour3 `\thanksref`.
- **63e** (landed): author lists (the run-336 fidelity audit's 21 of 73 papers with merged or lost co-authors, all
  silent; KPE #521, OD #459): per-author classes (aastex/emulateapj, amsart, revtex, llncs/aa, IEEE blocks, labelled
  authblk) parse their `\author` as an author block; lines without marks find later names ("and" lines, unfinished
  lists, names after styled affiliations); `\newline`, " \& ", `\altaffilmark` after the comma; names marked before
  them are found by layout and by the affiliation below that answers their mark (Perl drops them all); a per-author
  class's affiliation-line `\thanks`/`\altaffilmark` stays with its author; a merged author is
  `Error:frontmatter:merged_creators`. A/B on 318 papers (the 80-paper fidelity sample + 238 random): 56 papers'
  authors split as LaTeX typesets them, none with more errors, no merged author left. 67 guards
  `perfect_kernel_batch63::author_*`; open RED repros (sectioning-frontmatter/, listed in OD #459): shared-label names
  on lines of their own, unanswered labels, "and"-joined marked names, name-shaped affiliations at an `\and` group's
  head, stacked unmarked names in per-author blocks, amsart `\address` after a split `\author`, glued initials (63f).
- **63d** (landed): pinlabel figure labels (85 papers of run 336's first 265k had `\labellist`/`\pinlabel` undefined):
  pinlabel.sty read raw with its `\ps@begin` placing the picture through graphicx on both driver branches,
  `\pdfximage` keywords + `\pdfximagebbox` (OD #458), `\openin` closes the stream first (KPE #520), PDF object streams
  with `/Filter` first. A/B on the 100 papers with them undefined by then (63b → 63d): errors 843 → 5, no Fatal, none with more errors, 26 with
  more images, one with fewer (1310.1838's unshipped figure: pinlabel's own "not found" text, as pdflatex prints it); the reader's `/Filter`-first object streams reach every Illustrator PDF figure, A/B'd on these papers only.
- **Fleet address-space ceiling → `Fatal:TooManyErrors`** (63c, deployed 2026-10-07 16:35Z): 113 of run 336's first
  253 TooManyErrors papers carry only `Error:post:xpath` "XPath evaluation failed (no libxml2 error detail)" on
  `ltx:XMath`, 15 more were tipped over by it (math0005012, 1209.0448, 1310.4783). They convert with 0 errors in a fresh
  process under the same 5632 MiB `RLIMIT_AS`, on 62zt and 62zu alike (`VmPeak` 3.6 GB for 1106.3786). Live harness
  children sat at ~1 GB RSS but 4.8-5.6 GB of address space (mimalloc reservations), so the RSS recycle never fired
  and libxml2's `malloc` failed. 63c recycles a harness child when its `VmSize` nears the cap
  (`docs/performance/CORTEX_WORKER_HARNESS.md`); after the swap children held 3.1 GB of address space on average
  (4.2 GB max) where they had held 4.8 GB (5.6 GB max). Re-queue the affected tasks at completion (row G). Open: post-processing
  still reports such an OOM as ~100 `Error:post:xpath` → `Fatal:TooManyErrors`; on a detail-less XPath failure under
  memory pressure (`latexml_post/src/document.rs` ≈959/1031) emit one latched resource `Fatal` instead. Also open:
  `fatal/cortex never_completed_with_retries` (150 in the first 145k), likely the same ceiling killing workers.
- **Mouth EoF** (10, runaway delimiters, singletons): 3 × `\input amstex` (amsppt-as-plain, below); `\NOT` 2002.09653
  2212.04182; `\endcaption` 1210.4309 2407.08262; `\@checkend` 1404.3085; ` at ` 0811.2708; `\@nil` 1402.2014.
- **Document Malformed** (8): 0901.1457 ships the AIP proceedings kit and a template driver is taken as the main file.
- **never_completed** (53): fleet pressure; 3 of 4 convert standalone, 1007.0100 is a clean 180 s timeout.
- **IMS bindings**: imsart (contrib) keeps a partial copy of the `\b*` markup — fold it into `ims_support`; arximspdf's
  `{longlist}` is a structural environment where arxbj's is now the class's labelled `\list`.

### Run-329 and sandbox open residuals (61r-62f, 2026-10-04)

Open items from the cortex reruns of sandboxes 2605/2606 (61r) and from mining run 329 (62a-62f). The fixed narratives
(61r's validation, the stop of run 329 and the 62a-62f fixes with their numbers, witnesses and guards) are archived in
[`archive/SYNC_SESSIONS_2026-10.md`](archive/SYNC_SESSIONS_2026-10.md); first-error and Fatal cluster tables of run 329
in `~/data/pk_agents/main/scratch_g/run329/clusters62c/`.

From 61r (sandboxes 2605/2606): a tabular
`\\[<len>]` with a non-positive length is `\setlength`-calcified in latex.ltx (:16586-16602, `\@vspace@calcify`) but read
by TeX's scan here; latex.ltx 2025's `\vspace` is calcified too (:9254/9362) while ours is `\vskip#2\relax`. Open, each pdflatex-clean, ids in
`~/data/pk_agents/main/hostb_61r4.tsv`:
  - calc's `!` protocol (`\calc@next`, calc.sty:52/147/153): linegoal's `\LNGL@set!` runs inside the expression
    (2605.05695).
  - Pre-broken papers now Fatal: 2605.05881 (`\pgfutil@emu@@unpack` Mouth EoF; 83 errors on 56il), 2605.10232 (tikz-cd
    cells holding `tblr` tables with `\&`; PushbackLimit since 57cp; 23 errors on 56il).
  - Latent panic: `gullet.rs` `read_token`'s `handle_template(data.borrow_mut(), …)` (RefCell already borrowed), Fatal
    on 2606.05500 under both images until 61s (KPE #462) removed the makecell cascade that reached it; the full paper
    on `latexml_oxide.61r6-rel` still reproduces it, no truncation of it does.
  - `runtime-bindings` builds only: a versioned name whose stripped name has only a runtime `.rhai` binding is found by
    loading it (`find_file_fallback`), without the document's options; the caller's load with them is then skipped.
    Fix: ask the rhai tier whether `<name>.rhai` exists instead of loading it.
  - Singletons (1-8 errors each, 56il-57ck window unless noted): Missing control sequence inserted (2605.26277,
    2606.00866, 2606.20186); listings input files not found (2605.15569, 2605.12863); delimited user macros
    (`\edef\__figfile` read through a neutralized Semiverbatim, KPE #465, 2606.13798; `\def\\ell` reaching
    authblk's `\affil` text, which TeX typesets where `\\` is local — authblk.sty:156, `\@maketitle`'s tabular —
    2605.28517; `\\` Match:X in neurips' author tabular, minimal shape clean, 2605.12000); 2605.04969 (sectioning in a box, 59j); 2605.07684 (`\ContinuedFloat` after a table); 2605.10068 (`#`
    reaches the stomach, 61m); 2605.11285, 2605.31585 (58h counter steps: `\the\cmdKV@…`, `\iffirstchoice@`);
    2606.15122 (utf8 keyboard character, 58q); 2605.19122 (`_Capture_` close); 2605.29722 (`_` outside math);
    2606.30845 (`\capitalizethefirst`); 2606.11726 (2 undefined counters in a plain-TeX paper now converted whole).

62s: the 2609 templates' new author and title macros, from each template's source (2609 cluster study): neurips_2026
`\workshoptitle` + tracks (113 papers), acmart `\correspondingauthor` + `\if@ACM@anonymous` (76+8), aa `\corrauth` (58),
spconf `\sthanks` (45), revtex's `\move@AU`/`\move@AF`/`\@affiliation` for openjournal (33), wacv via the cvpr binding
with `\thetitle` the title's copy (21; wrapped once when cvpr and wacv both load, and for iccv.sty:477-479), natbib's
`\@ifxundefined`/`\NAT@sectionbib` for iau/JFM (24), acmart `\if@ACM@balance` (11), neurips' `preprint` switch and
tracks' `\@trackname` and `\@noticestring` with the year's own ordinal, location and track wording — also for a style
requested under a directory, whose fallback records the request (`content.rs`, Perl's `\@currname`); no venue note of
the binding's own, as the paper's style copy decides what its first page prints, so a camera-ready workshop paper's
workshop name is not output (DIVERGENCES #448; a workshop-only note awaits a ruling) — acmart's `balance`
options, aa `\aa@emailfont`, newtxmath's `\up<letter>` names (`\upmu`, 15). Guards
`perfect_kernel_batch61::templates_of_2609_keep_their_author_and_title_macros`,
`perfect_kernel_batch61::neurips_notice_names_the_year_of_its_style`. Known limit: a preamble `\newcommand`
of one of these names is ignored like any binding macro's (no 2609 bundle is hurt; older-template papers could be). The 2609 study's other findings:
the expl3 "Mismatched LaTeX support files" errors are the dump regression below (62u), not the harness; 945 EPS
failures: AppArmor `gs` denies `/opt/cortex-scratch` (open).

62t: REGRESSION since 62b fixed — a caption opening with a conditional (`\caption{\ifdefined\x …\fi …}`) ended in
"Extra \else" + Fatal EoF scanning for `\endcaption` (2609.27590, 2609.28438): `\@dblarg` always gives a `[short]` and
`\@caption@@@`'s `\ifx.#2.#3\else#2\fi` took the conditional as its second token; the list entry is now chosen by the
detokenized `[short]` without skipping either text, as are longtable's and listings' (KNOWN_PERL_ERRORS #468). The 2609
classes' commands: llncs `\doi` (a single `\url` link under hyperref; 2609.04690; #492), sagej `\affilnum` (2609.04585), the
appendices' `\Roman` numbering of ieeeconf (`\ifuseRomanappendices`, default on; 2609.16300) and IEEEtran
(`romanappendices`; KNOWN_PERL_ERRORS #491), subfigure's captions chosen the same way as the
kernel's (#468), acronym `\Acp`/`\Aclp`/`\Acfp`/`\Aclu`/`\Acfip` (let to the lower-case forms, the capital not
rendered) and `\acfip` (#490). Guard `perfect_kernel_batch61::captions_and_2609_class_commands_keep_their_text`.
OPEN (design, user 2026-10-05): a paper and its supplement are two top-level documents — revtex's supplement idiom
restarts the title block (`\frontmatter@init` + lets of `\title`/`\author`/`\maketitle` to `\frontmatter@…`,
revtex4-2.cls:2115-2140,3127-3140; 2609.07332, TooManyErrors Fatal) — and the frontmatter API has one title block, so
lets to the binding's `\title` replace the paper's title and append the supplement's authors (the draft is set aside
locally, recorded in the main handoff register); the frontmatter API needs extending for a second document.
62t (part 2): the 2609 classes load what their class files load — lipics-v2021 (array, subcaption, comment with the
`{CCSXML}` exclusion, multirow, tabularx, …, and at the document's start xcolor unless the paper has it plus the class's
named colours, soul's `\textsolittle`; 2609.13401 64 → 44 errors, 2609.10114, 2609.13485 13 → 0), fairmeta (2609.11172),
WileyNJDv5 (and the WileyNJD-v2/WileyASNA-v1 copies the binding serves, which load the same) with caption
(2609.06025 2 → 0), bmvc2k's xcolor and T1 with its `\addauthor` mail read verbatim and set in sans (2609.06007 4 → 0),
lmcs's theorem set with its `defC`/`thmC` styles, tikz, xparse, mathtools (2609.11893 10 → 0), informs3's
`\TheoremsNumbered*`/`\EquationsNumbered*`/`\ECSwitch` (2609.08001), MnSymbol `\llangle`/`\rrangle` (2609.07645);
`\tracingmacros`/`\tracingcommands` stored as numbers so `\the` reads them back (Perl TeX_Debugging.pool.ltxml:214-225;
was silently empty). Guard `perfect_kernel_batch61::the_2609_classes_load_their_packages_and_commands`. The bindings'
named witnesses, re-converted: 2502.11299 25 → 0, 2305.19985 14 → 2, 2305.14448 2 → 0, 2511.16624 1 → 0, the other fifteen
unchanged (lipics 2311.17226, 2211.04601, 2606.01187; Wiley 2203.16535, 2406.06228, 2407.00139, 2504.02281; fairmeta
2412.06264, 2508.07407, 2509.24704, 2605.29955; lmcs 1607.01886, 1607.04128, 1709.06170; bmvc2k's xcolor reload
2605.00310). `\ECSwitch` without a `\TheoremsNumbered*` is undefined, as under pdflatex (the
class defines no default). Residuals:
math-mode `^`/`_` in 2609.13401; a raw `_` in an email (`\lx@add@email`) prints OT1's dot accent (a catcode-12 `_` in
roman OT1, as pdflatex would — classes that set the address with url.sty, in typewriter or under T1 need the binding to
say so, as bmvc2k's now does); the amsthm binding ignores a `\newtheoremstyle` head spec (lmcs's `thmC` drops the
note's parentheses, ours keeps them) and informs3's `\mdseries\scshape` head font (ours bold, :1857-1859); thm-restate does not load thmtools' `\declaretheorem` (2606.01187); 2609.25833's
98 errors are arabtex (`\setcode`, `\RL`); amsart's `\@xsetfontsize` internals (2609.37833) landed in 62z. GENERALIZATION
(open): these bindings hand-copy their class's package list, which drifts by class version and drops options
(fairmeta's `[numbers,sort&compress]{natbib}`); Perl's own dependency scan of the shipped class
(`require_dependencies_except`, as sn_jnl_cls.rs uses it) with per-class exceptions is the general form — needs its own
A/B over each class's papers.

62zl: iopart's `\address` adds its affiliation on each call, as iopart.cls prints one block per call (:240-248),
through `\lx@add@affiliation@marked`; Perl's `\lx@add@affiliations` dequeued the ones before, keeping only the last
(KNOWN_PERL_ERRORS #511; 21 of the 41 2609 iopart papers call it more than once, 2609.01831 lost three of four). In a
marked affiliation list (`affiliation_calls`) an unmarked line continues the affiliation before it, as in
`\lx@add@authors` (each `\\` line had been its own affiliation, the unmarked ones placed by position; 2609.19448) —
not across `\and` (an entry boundary there too), nor a footnote-symbol legend (`* Equal contribution`, `\dag …`),
which stays apart (llncs `\institute`, 2609.06094); address lines are placed as runs once the list is
read (`place_address_runs`, explicit `author:N` labels): where the list puts addresses under its affiliations (one
affiliation, or a run above another that is no footnote-symbol legend; 2609.19448), one address under an affiliation one author requests is theirs and
any other run continues the affiliation; else the runs are the block's — by order when there is one address per
author, else on the shared creator below the authors (2609.06094); a one-address line is kept as written
(`\href{mailto:…}` stays a link), an `\author` call's tail is placed so only at `\maketitle` when that call made
every author, and a marked iopart `\address` before `\author` keeps its affiliations (`email_line_calls`, shared with
`\lx@add@authors`; an email command's wrapper is no longer nested in `\lx@add@email`; DIVERGENCES #52(j)); the mark splitter cuts only outside
groups, and a line opening with a font command or group that holds the marks (`\textit{$^1$A, $^2$B}`, `{\small …}`,
`\small{$^1$A, $^2$B}. Email: …`, `{\color{blue} $^1$A, …}`) is split inside it, the wrapper (a group with its opening
declarations and their arguments) repeated on each piece and what follows split at its own marks, while a
`\thanks{…}` holding the list is unwrapped (2609.00995, 24896: both had regressed under a depth-only rule,
caught by the A/B) (OXIDIZED_DESIGN_DIVERGENCES #52(f)). A new
apa7 binding (contrib) reads apa7.cls itself for its modes, packages and bibliography choice (none by default, :179-216
— the dependency scan had loaded both biblatex and apacite, 17 malformed bibitems in 2609.02899) and maps its author
block onto the frontmatter API: `\authorsnames[marks]{names}` with `\authorsaffiliations{…}` numbered in order, linked
through the kernel's author-line parser (affiliations given first are kept until the names, as apa7.cls builds the
title block from stored lists at `\maketitle`, :780-890); `\author` (the class's one-name `\authorsnames`), `\authornote`, `\note`, `\abstract{}`, `\keywords{}`
(`\lx@add@keywords`); the student page's course, instructor and due date and the journal masthead's journal, volume,
copyright and number, as the modes print them; `\addORCIDlink` as the kernel ORCID link; `\doi` a link to the record, where apacite's own sets url-styled text only (semantic markup before
PDF fidelity, user 2026-10-06); `mask` hides the author identity as the PDF does (:131; user 2026-10-06); the kernel's `\appendix`
stands over the class's (:1041-1075, whose `\section` redefinition the kernel refuses: "Appendix 9" for "Appendix A",
2609.00670). 4 papers, 32 errors -> 0 (2609.00670, 02899, 12869, 19448). RED `marked_affiliation_list_email_line_leaves_no_empty_contact` (in a
marked list an `\email{…}` line had left an empty affiliation contact; GREEN with the shared `email_line_calls`). Guards `perfect_kernel_batch61::{iopart_each_address_adds_an_affiliation,
marked_affiliations_inside_a_font_group_keep_the_font, marked_affiliation_continuation_lines_stay_with_it,
marked_affiliation_wrappers_and_legends, marked_affiliation_review_shapes, apa7_author_block_maps_onto_the_frontmatter}` (and `omnibus_doi_reads_as_a_url`
now on apa6, OmniBus's class, with apa7's linked DOI beside it).

62zm: a bibliography right after a unit that heads it takes its place (user 2026-10-06; sect11.rs
`absorb_bibliography_heading`, the bibliography's `after_close`): titled as it is, or an otherwise empty unit when the
bibliography has no title or the `\bibitem`s opened it in — Pandoc's `\section{References}` over its
`{CSLReferences}` list (2609.02899), `\section*{References}` before `{thebibliography}`; Perl keeps two headings, the
first over nothing (KNOWN_PERL_ERRORS #512, OXIDIZED_DESIGN_DIVERGENCES #456). 62zn: `\bibitem`s in
`{enumerate}`/`{itemize}`/`{description}` no longer err at the list's `\end` (`\endgroup`, shared with Perl,
KNOWN_PERL_ERRORS #513): the auto-open opens the group that `\end` closes. Open: what the list sets before its first
`\bibitem` is undone with the list's group (RED repro sectioning-frontmatter/bibitems_in_a_list_keep_the_lists_settings).

62zk: the informs binding (informs3 and its h/a/noheader copies, informs4 and its copies, informs5) gains the class
API its papers use — the journal switches and the review options (informs4.cls:9-54; informs3 declares twelve journals
and no anonymous-review options), the stored metadata and running heads and feet (:948-985, read back as
`\theRUNTITLE`, `\theJOURNAL`), `\argmax`/`\argmin` (:889-890), `\FUNDING` (:1023), `\FIGURE`/`\TABLE` with their notes
and a grouped body (:2229-2259, 2449-2470), the hanging lists `{henumerate}`/`{hitemize}` (:1702-1763) — and reads `\AUTHOR` and `\AFF` as the class does, one argument each, the
affiliation under its author (:1198-1206): `\AUTHOR{name}{aff}` had swallowed the next `\AFF`, leaving the affiliations
as loose notes (2609.37380). A marked author list links its marked `\AFF` lines by their marks through two kernel
macros, `\lx@add@authors@append` (the author-line parsing without `\author`'s replacing) and
`\lx@add@affiliation@marked` (2609.17368, 22690, 28084); `affiliation_calls` splits a line that starts with a mark
and names several marked institutions at its marks (`$^1$Univ A; $^2$Univ B`), as the author-block parser already
did, and drops a trailing `;` as it dropped a `,` (OXIDIZED_DESIGN_DIVERGENCES #52(f)): 7 of the A/B's 600
non-informs papers (picked for marker-led lines with several marks) changed, all for the better (2609.12549, 16472, 17549, 21045, 21169, 21347, 29760). Under
`blindrev`/`dblanonrev` the class's notice replaces the author block and the history, acknowledgment and author bio
print nothing and `\RUNAUTHOR` names no one, as in the PDF (:987-995, 1071-1076, 1225-1233, 1952; user 2026-10-06);
the binding reads which class was asked
for (`fallback_request`, now also recorded for a prefix fallback such as `informs3noheader`) for informs3's
"blinded" wording, informs3noheader's empty notice and informs3's MOOR acknowledgment. No 2609 paper sets the blind
options (2609.23739 has them commented out); older informs4 copies without `dblanonrev` word the notice as informs3
does, and print the MOOR acknowledgment under `blindrev` as informs3 does; they get informs4's current behavior
(2609.10587, 17368, 25924, 28947, 28263). The notice is a frontmatter note, shown only on hover, where the PDF prints
a bold line in the author slot. 8 papers' errors (2609.10587, 17368, 22690,
23739, 24605, 25924, 37380, 38842). Open: `\argmax` (`\mathop{\rm arg\,max}`, and `\operatorname*{arg\,max}` alike)
parses as the product "arg × max" (math stream); the residual informs diagnostics (2609.28451's own informs3 copy
comments out `\ECSwitch`'s `\ECHowTheorems` call, :2753, so its undefined `\ECHowTheorems` is ours alone; the
`THkey`/`EXkey` theorem styles, whose head is the optional argument alone, revert to plain with a warning, 21433,
28451); `\RUNTITLE`/`\RUNAUTHOR`/`\MANUSCRIPTNO` stay
notes though the class prints them only in the page heads (page furniture). RED
`sectioning-frontmatter/authblk_ieeemembership_after_comma_affiliation_twice` (IEEEtran + authblk: `\author{A. Author,
\IEEEmembership{Member, IEEE}}\affil{Univ X}` attaches the affiliation twice — Rust-only — and drops "Member, IEEE",
as Perl does); RED `affiliation_before_any_author_is_kept` (an `\AFF` before any `\AUTHOR` is lost without a warning;
Perl removes an unlabelled pending annotation silently too); `author_symbol_mark_links_its_affiliation` (fixed by 62zp); `iopart_each_address_adds_an_affiliation` (fixed by 62zl); `split_before_affiliation_marks`, now reached from
`affiliation_calls`, ignored brace depth (`affiliation_marks_inside_a_font_group_keep_the_font`, fixed by 62zl) and took an ordinal superscript for a mark
(`affiliation_line_superscript_ordinal_is_not_a_mark`, fixed by 62zp). Kernel author-mark linking
(`author_suffix_marks_after_commas_link_affiliations`, SHARED with Perl, 2609.25924; `author_prefix_marks_first_line_is_names`,
Perl drops the authors): fixed by 62zp.

62zt: the 2609 frontmatter digest cluster (`\lx@add@frontmatter@now`/`\lx@frontmatterhere`/`\lx@personname`, 9
papers; root-cause by agent, verified): the author splitter keeps an unbraced `\begin{name}...\end{name}` inside a piece
whole (a `{center}`/`{flushleft}`/`{flushright}` around the whole block or an `\and` group is unwrapped first) and
splits at a superscripted separator (SHARED with Perl; OXIDIZED_DESIGN_DIVERGENCES paren-split entry); a note's
`\\` is the plain one, as `\@footnotetext`'s `\@parboxrestore` gives it (`\thanks` in a `p{}` cell, 2609.32661);
IEEEtran's `\begin{@IEEEauthorhalign}`/`\end{@IEEEauthorhalign}` are `\relax` as in Perl (`\authorrowbreak`,
2609.02116); achemso's `{suppinfo}` is its "Supporting Information Available" section (2609.03387, 10829). Guard
`perfect_kernel_batch61::frontmatter_digest_cluster_2609`. Open: RED `ieeetran_authorrowbreak_leaves_no_empty_affiliation` (one
empty affiliation contact; Perl merges the two authors); a block-wide `{center}` followed by more tokens (`\begin{center}Ann
\and Bob\end{center}\thanks{...}`) is not unwrapped, its authors kept together; 2609.13401's frontmatter errors follow a
math-mode leak earlier
in sections/mdps.tex, not this cluster.

62zs: a sectioning unit in a float body opens an `ltx:inline-sectional-block` there — numbered, labelled and identified
in sequence as pdflatex numbers it, schema-valid — and the float's id is fixed before the unit steps its counters
(S1.F1, not S2.F1); user ruling 2026-10-06 after a demo (38 2609 papers, 117 errors; OXIDIZED_DESIGN_DIVERGENCES
#189). Guard `perfect_kernel_batch54::sections_inside_floats_keep_number_id_and_ref`.

62zr: MathSciNet's `\cprime`/`\Cprime`/`\cdprime`/`\Cdprime`/`\cftil` provided within `{bibtex@bibliography}` only,
where free, for `.bib` entries copied without the export's `@preamble` (15 2609 papers, 2609.00627, 2609.11266; user
ruling 2026-10-06; OXIDIZED_DESIGN_DIVERGENCES #78). Guard
`06_cluster_bibliography::bib_mathscinet_fields_without_preamble`. Rulings 2026-10-06 recorded: `\foreach` over a
locked variable keeps Perl's lock (already bounded, PERFECT_KERNEL "Engine"); an eqnarray cell's `+} {` relying on
the `{##}` template is an accepted residual; a `\section` in a float: ruled after the demo, landed in 62zs.

62zq: an author's `\thanks` and `\footnote` read an ordinary argument, as LaTeX's and a title's do — Perl's
Semiverbatim `\lx@add@thanks`/`\lx@add@note` printed their math as text (1,391 2609 papers have math in a `\thanks`;
KNOWN_PERL_ERRORS #515) — and a note's content in a marked author line keeps its superscripts as text
(`\lx@frontmatter@savesups`/`\lx@frontmatter@plainsups`), save a leading mark authors request on a note that is one
legend, which labels the note, the legend going to them (`note_body`; 2609.00885, 2609.19600); an author's requests are
its marks outside its notes (`author_mark_operands`). RED `thanks_with_several_legends_goes_to_each_legends_authors`
(2609.39576: several legends in one `\thanks` stay whole with its author); `\dag` is not among the footnote symbols
`rewrite_symbol_superscripts` shows (an author's `$^{\dag}$` is consumed where `$^{*}$` is shown). Guard
`perfect_kernel_batch61::author_thanks_and_footnote_keep_their_math`. Open from its review: RED
`sectioning-frontmatter/thanks_affiliation_list_keeps_superscripts_in_its_text` (the thanks-abuse idiom, 2606.00313:
every `$^{n}$` in the `\thanks` goes through `\lx@affiliation@withsup`, so `10 m$^{2}$ labs.` loses its "2"; the same
on 62zp).

62zp: marks glued to a comma in a line that does not open with a mark are the name's before it
(`marks_before_glued_commas`), a `\thanks` glued among them too (2609.37343); an author block's first line is its names even when the marks lead them, and so is a
later `\and` group's first line once the first line did, when no author before it requests its mark and no
affiliation has been read; a
footnote-symbol author mark, still shown, also requests the affiliation of its symbol (`\lx@frontmatter@symbolmark`,
creators only; `\inst{*}` too), so informs4's `\AFF{$^*$Corresponding author}` goes to its author (2609.38842); an
ordinal suffix superscript after a numeral (`5\textsuperscript{th}`, `21$^{st}$`) is shown, not taken for a mark
(`rewrite_ordinal_superscripts`; 2609.01831). OXIDIZED_DESIGN_DIVERGENCES #52(k). Guard
`perfect_kernel_batch61::author_marks_link_their_affiliations`. Guards
`perfect_kernel_batch61::{informs_class_api, informs_blind_review_hides_the_authors,
informs_author_marks_link_affiliations, informs_hanging_lists, marked_affiliation_lines_split_at_their_marks}`.
Open from the 62zp review: RED `sectioning-frontmatter/author_symbol_legend_line_links_its_starred_author` (Rust-only;
Perl links it): in an article `\author`, a `$^*$Corresponding author` line continues the affiliation before it
instead of going to the author whose mark carries `*` (`$^{1,*}$`), because `rewrite_symbol_superscripts` makes its
`$^*$` a visible superscript before the lines are split; `author_thanks_and_footnote_keep_their_math` (SHARED,
KNOWN_PERL_ERRORS #515; fixed by 62zq); RED `author_concatenated_digit_marks_link_each_affiliation`
(2609.37343: `$^{23}$` for affiliations 2 and 3 labels `affiliation:23`, so they sit on a nameless creator; Perl
gives the first and drops the second); `author_and_groups_sharing_a_prefix_mark_are_names` (`\and` groups whose
names open with the mark of an author before them; GREEN in 63e, guard `perfect_kernel_batch63::author_and_groups_sharing_a_prefix_mark_are_names`; Perl drops both authors).

62zj: the jmlr binding defines the ML4H copy of jmlr.cls's track machinery (`\mlhtrack`, the `\ifmlh…` switches; :83-142,
its "no track" `\AtBeginDocument` error left out; 8 papers); IEEEoj's front-matter setters (`\authornote`/`\corresp` as
notes; the dates and `\doiinfo` as dates and a DOI pubnote, as the first page prints them, :3457-3458 — IEEEtj's
too, which were no-ops; an empty `\doiinfo{}` prints none, :3458, 2609.27083; the published and current dates under the
class's names; IEEEoj.cls:3447-3451, 3713-3717; 3 papers); when the lock refused a raw class's `\author`
(`\author:redefined`), author content reads the author markup still undefined there — `\fnms`/`\snm`/`\inits`/
`\prefix`/`\particle`/`\suffix`/`\degs`/`\roles`/`\orcid`, which IOS-Book-Article.cls and econsocart.cls define
only inside their own `\author` — with its OmniBus meaning, undefined again after, a class's or the document's own
definition standing (base_utilities.rs `\lx@author@markup@begin`; 5 papers; Rust-only exposure through
`localrawclasses`, OXIDIZED_DESIGN_DIVERGENCES #444; `\editor`/`\translator` content does not take it yet); amsthm `\@upn` (KNOWN_PERL_ERRORS #509);
newtxmath's `\re@DeclareMath…` helpers (KNOWN_PERL_ERRORS #510). Open from the same root-cause report
(`~/data/pk_agents/main/agent_reports/2026-10-06_rc62zj_template_clusters.md`): the informs binding's class API (fixed by 62zk), an apa7 binding (`\authorsnames`/`\authorsaffiliations`/`\authornote`/`\addORCIDlink`; 4 papers), IEEEtran
`\@IEEEabskeysecsize` (6 papers, fixed by 62zo: the class internals a document copies, KNOWN_PERL_ERRORS #514; open RED repros from it:
list-structure/enumitem_declares_labelindent, sectioning-frontmatter/ieeekeywords_blocks_are_all_kept and
ieeekeywords_redefined_with_quotation_in_one_column), MathSciNet `MRREVIEWER` macros (`\cftil`, `\cprime`…; 6 papers,
NEEDS A RULING), vmsta2's local `\author` redefinitions under the lock (1 paper, deferred). The IEEEojcsys binding still maps `\authornote` to
`role=authornote` and `\receiveddate` to a note, and lacks the other setters (not yet checked against IEEEojcsys.cls). Guards
`perfect_kernel_batch61::{jmlr_reads_the_ml4h_track, ieeeoj_front_matter_setters,
raw_class_author_markup_reads_within_author_content, amsthm_upn_and_newtxmath_redeclare}`.

62zi: acmart reads `acmart-preload-hook.tex` before its packages, with the class's warning, and defines the ACM palette
(acmart.cls:44-47, 673-680; KNOWN_PERL_ERRORS #507; 2609.29962 298 errors → 0); the biblatex binding defines biblatex's
message commands `\blx@error`/`\blx@warning`/`\blx@info` and variants, which shipped style files call
(biblatex.sty:135-155, 1353-1370; KNOWN_PERL_ERRORS #508; 2609.32652, 58 papers in 2609) — which lets acmauthoryear.bbx
print a "bibmacro missing" warning pdflatex does not, the binding's `\ifbibmacroundef` always answering undefined (RED
`index-bib/ifbibmacroundef_knows_the_standard_bibmacros.tex`). Guards
`perfect_kernel_batch61::{acmart_reads_its_preload_hook_and_palette, biblatex_message_commands_are_defined}`.

62zh: the conversion binaries remove a run's temporary trees however they exit. `process::exit` skips destructors, so
the `latexml_oxide` CLI left each archive's unpacked sources (a `TempDir`) in the temp directory after every archive
conversion — clean, or stopped early by a Fatal — until a per-user `/tmp` quota filled during the 62zg A/B; its exits
after the unpacking now go through `exit_removing` (latexml_oxide.rs). The exits no caller sees — the watchdog's
timeout (124) and memory ceiling (137), the allocation-failure hooks — first sweep a registry
(`latexml_core::watchdog::remove_on_exit`/`remove_registered_paths`): the CLI's archive and archive-output staging
directory, cortex_worker's per-job sources, destination and output zip until the job hands it on, and post's page-spill
directory register there, so a timed-out paper no longer leaves its tree in a fleet worker's scratch directory for the
life of the container. The watchdog and the allocation-failure hooks claim the exit before they sweep: a thread
finishing meanwhile parks (`park_if_exiting`, bounded at a minute, then exits with the claimed code) instead of exiting
with its own code or handing on output built over the deleted trees. Residual: the page-render children
(`LATEXML_RENDER_JOBS` > 1, off by default) have no watchdog of their own and are not killed by the parent's watchdog
exit — orphans over a deleted spill, which can recreate a swept directory; what kills the process from outside still
leaves the trees — the harness's SIGKILL of an unresponsive worker (cortex_worker.rs), a cgroup or
systemd-oomd kill — and so does a panic in a `panic = "abort"` (maxperf) build; for the fleet, a per-slot `TMPDIR` the
harness wipes on respawn would cover these. The image-size unit tests' `lxsize-*` fixtures remove themselves (image.rs `Fixture`).
Residual: the graphics-cache unit tests leave one small `gcache-shared-<nanos>` directory per test process
(graphics_cache.rs, a process-lifetime `static`). Guards `cluster_cli::whatsinout::archive_sources_are_removed_at_exit`,
`watchdog::tests::registered_paths_are_removed_before_an_exit`.

62zg: the 2609 `\endgroup`-cluster triage (report `~/data/pk_agents/main/agent_reports/2026-10-06_endgroup_triage.md`,
groups A-K). Landed: (A) an author block in any alignment stays whole — the splitter's list gains `{tabular*}`,
`{tabularx}`, `{tabulary}`, `{longtable}`, `{array}` (KNOWN_PERL_ERRORS #502; 2609.34061, 34965, 39374, 39909);
(B, Rust-only) `\lstinline{…}` and minted's frozencache `\mintinline` retract their `{` delimiter from the alignment
ledger only when the gullet did not count the closing `}` (a pre-tokenized argument keeps its END catcode;
`settle_verbatim_brace_ledger`, listings_sty.rs; 2609.04372, 29962); (D) `\captionsetup{type=}` digests
`\caption@settype`, setting `\@captype` (KNOWN_PERL_ERRORS #504; 2609.06557, 32742), and a caption that becomes the float
of its box (OXIDIZED_DESIGN_DIVERGENCES #182) now carries its list entry — `\@@toccaption`, which runs before that float
exists, hands it on (`lx@unplaced@toccaption`; the review caught the list of tables repeating one entry and dropping the
next); a sub-float's caption (`\subcaption`, `\@captype` `sub<type>`) becomes its parent type's element, not
`ltx:float`; and `float_to_element`'s closing form (`^^`) takes only open nodes — a text node's previous sibling that
could hold the float (a minipage holding a sub-figure) was chosen and "closed", an "isn't open" error over an unchanged
tree (2609.15558, 33998; the old binary hit it with `\def\@captype{figure}` too), nor — once the walk skips them — end
it at a Math or rule between such minipages and enter the earlier one (OXIDIZED_DESIGN_DIVERGENCES #455). Residual:
sub-figures captioned before their figure's `\caption` take the figure counter as it then is — "0a", then the PREVIOUS
figure's number in each later group — where pdflatex, writing the label at shipout, prints "1a", "2a" (RED
`captions-floats/subcaption_before_caption_numbers_its_figure.tex`); (E) physics' operators read no
size (KNOWN_PERL_ERRORS #503; 2609.12812, 39468; the operatorP comment's witnesses 2003.02721, 2605.20398 re-converted);
(G) no global biblatex `\sort`, and `\DeclareSortingTemplate`/`\DeclareSortingScheme` read their `{<spec>}`
(biblatex.sty:14796), which that stub used to swallow (KNOWN_PERL_ERRORS #505; 2609.39511); (I) `{IEEEeqnarraybox*}` and the box's
`[<decl>][<pos>][<width>]` options, its column glue dropped from the expanded specification (:2280 `\edef`), shared by the IEEEtrantools and IEEEtran
bindings (`define_eqnarraybox`; KNOWN_PERL_ERRORS #506; 2609.32652). Guards `perfect_kernel_batch61::{
author_in_tabular_star_stays_whole, lstinline_in_text_in_align_cell, captionsetup_type_sets_the_caption_type,
captionsetup_type_caption_keeps_its_list_entry, subcaptions_in_minipages_then_the_caption,
subcaption_boxes_around_math_then_the_caption, caption_after_framed_text_keeps_the_reading_order,
physics_operator_leaves_a_sized_bracket,
physics_operator_bracket_across_a_row_break, biblatex_leaves_sort_to_the_document,
ieeeeqnarraybox_star_with_options_and_glue}`. RED from the review: `\text{\lstinline|w|}`'s tex= carries the listing's
internal setup macros (`tools/perfect_kernel/repros/singletons/lstinline_in_text_reverts_as_written.tex`); a text-mode
`{IEEEeqnarrayboxm}` opens no inline math, so its Math swallows the following text
(`alignment-bindings/ieeeeqnarrayboxm_in_text_keeps_its_math.tex`). Open: (C) arabtex's asect.sty:297 `\let\endabstract\endquotation`
(reached through alatex.sty's 2.09 `\begin`) closes the frontmatter `\abstract` with a quotation's end (2609.06402,
10153) — locking `\endabstract` does not help, since `\let` assigns past a lock in both engines (State.pm:380-384
`assignMeaning`; state.rs checks locks only in `install_definition`); (F) `\foreach \x/\text` against a locked kernel
macro and (J) an eqnarray cell `+} {` relying on the `{##}` template need rulings; (H) `\underaccent{$\text{.}$}`
deferred. A `\captionsetup{type=figure}` minipage with `\subcaptionbox`es converts clean but places the sub-figures
beside the captioned figure, not inside it. The `latexml_oxide` CLI left each archive's unpacked sources in the temp
directory (`process::exit` skips the `TempDir`'s drop), which filled a per-user `/tmp` quota during this A/B — fixed in 62zh.

62zf: elsarticle declares the class's other conditionals (elsarticle.cls:40-45 `\iflongmktitle`, `\ifdoubleblind`,
`\ifnonatbib`, `\ifnopreprintline`, `\ifuseexplthreefunctions` — the last stays false, the class's expl3 helpers it
selects being unemulated) and their options: the Elsevier journal styles
(jasr.sty, cnf.sty, jcomp.sty) test them inside the `\if@twocolumn` branches of their `\maketitle`, and undefined they
unbalanced TeX's skip of the false branch, so the other branch's body ran at load (`\finalMaketitle` undefined, an
unbalanced `}`, stray `\else`/`\fi`). The class's `\emailauthor{<email>}{<name>}`/`\urlauthor` (:211-235) are a
frontmatter note "email (name)", as the first page prints them — `\ead`'s contact would land on whichever author
precedes the call; the styles' `\KWD` (defined in their own `\keyword`, which the binding's {keyword} bypasses) is a
no-op. 2609.23725, 23732, 27838, 39431, 09773, 31741, 39502: 4-6 errors → 0. `nonatbib` leaves natbib out, as the class
does (:1242; KNOWN_PERL_ERRORS #501; 2609.05849, 19225, 20719, 23461, 26003). Guards
`perfect_kernel_batch61::{elsarticle_journal_styles_load, elsarticle_nonatbib_leaves_natbib_out}`.

62ze: the mnras binding takes mnras.cls's own settings: its enumerate labels "(i)", "(a)" (:930-940), the bibliography
heading "REFERENCES" (:1312, 1339), with `usenatbib` the author-year punctuation "(Draine 2011)" (:1336 `\bibpunct`),
with `usedcolumn` the `d`/`.`/`,` columns on dcolumn (:1349-1354); the mn binding declared that option as `usedcolum`
(Perl's typo, KNOWN_PERL_ERRORS #500), so dcolumn never loaded. 216 mnras + 18 rasti papers in 2609. Guard
`perfect_kernel_batch61::mnras_lists_citations_and_columns`.

62zd: rasti.cls (RAS Techniques and Instruments, mnras.cls under another name: v3.0 defines the same commands) loads
as the mnras binding (rasti_cls.rs): all 18 rasti papers in 2609 at 0 errors (2609.08700, 09329, 27622, 29860, 35676
had 5-33). cas-common's `stm/mktitle` title-page keys are declared (2609.00281). The box capture's backmatter lift
stops at a float, as at an alignment (base_utilities.rs `insert_block`, OXIDIZED_DESIGN_DIVERGENCES #205): an appendix
section or a bibliography in a minipage in a table was lifted past it with no diagnostic, leaving the table empty and
its caption at the top level; it stays in the table and errors as Perl's does (A/B over 867 papers: 8 improved; 2609.05763,
a cas paper's abbreviation list in a framed table, now in its table with Perl's `ltx:glossary`-in-`ltx:block` error
instead of moved after an emptied table). OPEN (third stop on one climb —
consolidate): the climb's stops are per-tag (list, alignment, float), and a box in a `quote` still leaks — the lift
moves the insertion point past the open quote, so the text after the box escapes it (RED
repros/boxes-groups/bibliography_in_minipage_in_quote_keeps_the_quote.tex); Perl's `floatToElement` restores the
insertion point after floating past an open container, the likely single rule, but the xebaposter/juradiss hoists
(surpass) must keep working — needs its own A/B. The mnras-binding gaps rasti papers inherited landed in 62ze. RULED (applied 62zs/64d): a `\section` inside a float (38 papers in 2609, 117 errors; 29 have no
other error: 2609.03590, 00943, 05680) is valid TeX — `\@startsection` needs only vertical mode — and could open in an
`ltx:inline-sectional-block` in the float (schema-valid, the appendix as its `ltx:section` stand-in; prototyped, 0
errors, jing-clean) — applied in 62zs for floats (ruling 2026-10-06) and in 64d for list items (ruling 2026-10-09;
OXIDIZED_DESIGN_DIVERGENCES #189, `perfect_kernel_batch54::sectioning_unit_inside_item_or_figure_opens_a_block`). Guards `perfect_kernel_batch61::{rasti_loads_as_mnras, appendix_in_a_minipage_stays_in_its_float,
cas_common_helpers_can_be_renewed}`.

62zc: class bindings that skipped what the class loads or defines. optica-article's binding runs the class file's own
dependency scan (`require_dependencies_except`, as sn-jnl's, without soul — the `\else` arm of its `\ifpdf`): array,
tabularx, multirow, newtxmath were missing (2609.00899, 05706, 06191, 10235, 01145; all 24 optica papers in 2609 at 0
errors); its `fontenc[T1]` now prints `"`, `<`, `>` as pdflatex does (9 papers' text, e.g. 2609.12212's bibliography). cas-dc/cas-sc define cas-common's
name parsers, printers and page styles (no-ops; the frontmatter keeps e-mails and notes itself), so a paper's
`\RenewDocumentCommand\firstname` or `\ps@cas` no longer errors (2609.16168, 16199, 36345, 00634, 20010). The general
form — every class binding scanning its class file — stays open (GENERALIZATION above). Guards
`perfect_kernel_batch61::{optica_article_loads_its_class_packages, cas_common_helpers_can_be_renewed}`.

62zb: three kernel gaps from the 2609 recheck. `\input@path` searched for files kpathsea does not find, after the local
paths and kpathsea, a `/` added as l3file does (l3file's `\file_full_name:n`, expl3-code.tex:12585-12612; content.rs
`find_on_input_path`, OXIDIZED_DESIGN_DIVERGENCES #454):
71 papers in 2609 set it, 2609.02998's class style (and the natbib its `\citep`s needed) was missing. `\pdfstartlink`
reads its rule, attr and action specs (pdftex.rs `LinkSpecification`, KNOWN_PERL_ERRORS #499): the bare no-op typeset
`attr {…} goto name {…}`, 57 papers in 2609 use it (AAAI forbids hyperref; 2609.00161). aa's `\pmatrix`/`\cases` take
amsmath's environment form when `\@currenvir` names them, as `\matrix@check` does (KNOWN_PERL_ERRORS #498; 2609.02726,
2609.04318). Guards `perfect_kernel_batch61::{aa_cases_and_pmatrix_take_the_environment_form,
pdfstartlink_consumes_its_spec, input_path_finds_a_style_in_a_subdirectory}`.

62za: `\minipage`/`\endminipage` locked against class and package files (`:locked@files`, state.rs `is_name_locked`;
sect12.rs, OXIDIZED_DESIGN_DIVERGENCES #453), the document's own definitions free as in Perl: iucr.cls's `\endminipage`
(kernel internals, to drop the footnote rule) broke every minipage and, through tikz's `\pgfutil@endminipage`, every
`text width` node: 2609.07722 and 2609.02106, 265 errors each, Fatal → 0; memoir's own save-and-restore is retired. From
the 2609 tally (cortex bundles of worker a01c8944b4: 3,374 Error, 64 Fatal), class gaps the class files fill:
aaai2027.sty's `\corresponding` and aaai's `\equalcontrib`, defined only inside their `\@maketitle` with their footnotes,
now carry the note on each author (kernel stubs, `DefinitionOrigin::Stub`, which the document's or a raw file's own
`\newcommand` replaces, as copernicus.cls:1673's; sect05.rs; 223 AAAI-27 papers); egpubl (argument-less
`\PrintedOrElectronic`/`\BibtexOrBiblatex`/`\biberVersion`, whose argument swallowed the paper's `\ifpdf`; `\ConfName`; the
remaining editor setters kept as the others; the paper-type, page-number and licence selectors; `\excludecomment{CCSXML}`;
`\teaser` a figure at `\maketitle`, as `\@maketitle` sets it with `\def\@captype{figure}`, after the abstract since the
schema keeps top matter ahead of figures; `\EGyear` only stores the year `\ConfYear` keeps), bmvc2k (geometry,
`\BMVA@blfootnote`, `\bmvaEtAl`, two-argument `\runninghead`), sn-jnl `\unnumbered`/`\numbered`, acmart's section fonts
(`\@secfont`, patched by pvldb.sty): 2609.00420/00555 1 → 0, 2609.00994 23 → 0, 2609.00732 13 → 0, 2609.00981/08090 2 → 0,
2609.05015/05615 1 → 0, 2609.00548/02328 1 → 0. Open: acmart's teaser rewrite (acmart_cls.rs `\lx@relocate@teaser`)
never matches, since the abstract precedes the teaser in the built document, and moving the teaser ahead of the abstract
would break the schema (`ltx:document` takes its top matter first) — retire or rethink; raw-loaded copernicus.cls drops `\affil` and its equal-contribution note, both typeset only by its
`\maketitle` body, which is not replayed (`\citati@nbyarticlenumber`): 2609.05557, 07363, 08383, 12496, 13482, 17175,
21633, 36077; 931 papers' first error
`imageprocessing:failed_to_convert` (not examined); `\cprime` in MathSciNet `MRREVIEWER`/`FJOURNAL` fields of cited
`.bib` entries (40 papers; Perl's MathReview synthesis digests the reviewer too; the always-on stub was retracted by
maintainer decision 2026-07-27, latex_constructs_rust_only.rs — needs a ruling); the tally's `_` cluster (216 papers) is
mostly gone on HEAD (splncs04 `\doi{…_14}` in the `.bbl`, now clean); Perl-origin, pdflatex-clean: `\lefteqn{…} & &` in
eqnarray (Perl's `\lefteqn` spans 3 columns, so the tabs the idiom writes are extra, 2609.03020: fixed in 65o, 10 → 0
errors, OD #556) and an etoolbox
`\AtBeginEnvironment{table}{\begingroup…}` + `\AfterEndEnvironment{table}{\endgroup}` pair (Package.pm:1918-1925 binds the
environment's mode into the hook's group, 2609.01057). Guards `perfect_kernel_batch61::{class_endminipage_does_not_replace_the_environment,
document_endminipage_definition_is_kept, class_bindings_2609_clusters}`.

62z: amsart's size machinery in ams_support (amsart.cls:169-219, 258-296): `\@xsetfontsize`, `\@currsizeindex`,
`\@adjustvertspacing`, `\@mainsize`/`\@ptsize`, and each size option's `\@typesizes` (10pt the default), which a class
built on amsart redefines its size commands with (m2an.cls:241): 2609.37833 undefined `\@xsetfontsize` then Fatal
PushbackLimit → 0 errors; the class bindings declare the size options as the classes do (ams_support keeps them no-ops,
as Perl, so a package does not run the document's size again), so `\LoadClass[12pt]{amsart}` sets the 12pt sizes. `\label` no longer digests `\@currentlabel` into a `LABEL@` value nothing reads (sect11.rs):
read lazily after a list whose `label=` redefined `\theenumi`, it reached the document's self-referential `\theenumi`
(KNOWN_PERL_ERRORS #496, OXIDIZED_DESIGN_DIVERGENCES #452; Perl hangs): 2609.13327 Fatal Recursion → 0. mathtools' paired delimiters follow the package
(`\delimsize` in a group, `\<size>l`/`\<size>r`; KNOWN_PERL_ERRORS #497, OXIDIZED_DESIGN_DIVERGENCES #451, ams
`mathtools` golden: three `tex` attributes): 2609.17447 861 errors, Fatal → 0. A/B over 551 papers (62y5 → 62z10, before the `\@ptsize`
restore): no output changed. Open (RED `tools/perfect_kernel/repros/math-parse/sized_bars_do_not_merge_into_a_norm.tex`, math
parked): two adjacent sized bars of different sizes merge into one `‖` (`\Bigl\lvert\bigl\lvert y…`), which
`\abs[\bigg]{\abs[\big]{y}}` now reaches; `\reDeclarePairedDelimiterInnerWrapper` still writes wrappers the X forms do
not read. Guards
`perfect_kernel_batch61::{amsart_xsetfontsize_sizes, label_after_a_relabeled_list_does_not_loop,
paired_delimiter_size_and_delimsize}`.

62y: four 2609 Fatal witnesses. (1) aastex7/aastex701 load rotating before their own `\rotate` (aastex701.cls:11416,
12196; aastex631 and earlier do not); the binding now does too (aastex_cls.rs, by the requested name), so a
deluxetable's `\rotate` stays its no-op instead of rotating's `{rotate}` environment, which the versioned fallback's
dependency scan of the class file loaded after the binding (Perl's versioned fallback scans nothing, Package.pm:2226):
2609.09266 Fatal → 2 errors (its .bib's ADS `\lt`, `\CID`). `rotatetable(*)` (:12212) set as its body: 2609.01052 → 0.
(2) `\left`/`\right`/`\middle` read a delimiter character by its `\delcode` (tex.web §1160): `DelimiterToken` +
`stomach::digest_as_delimiter`, where a math-active `(` (`\mathcode`(="8000`, active meaning `\left(`) is the
parenthesis (KNOWN_PERL_ERRORS #494), for `\bigl(` too (`TeXDelimiter`, one token); a math-active character whose active
meaning is undefined self-inserts as the plain character, where it was dropped (Perl: Γ, KNOWN_PERL_ERRORS #495):
2609.40266 Fatal Recursion → 0. (3) autobreak bound (latexml_contrib): lines as the package reads them (a line end inside
braces stays in its line, a leading `,`/`.`/`;`/`:` joins the line before), the first non-empty one the left-hand
side, the rest after the tab, the width-driven breaks left out; the raw package needed
amsmath's `\start@align`/`\collect@body`, which the native `align` never runs: 2609.08470 Fatal → 0 (and 2609.04368,
which loads autobreak unused, loses csquotes' failed patch of the raw package's `\collect@body`, 1 → 0, as pdflatex).
A/B over 551 papers (the 157 aastex + every 20th of the first 8000 2609 ids, 62x8 → 62y2): 2 improved, 0 regressed,
1 output changed (2609.01052). Open:
2609.13328's `\def\year{\pdfprimitive\year}` (an ieeeaccess `\def\year#1` workaround) loops, as `\pdfprimitive` is `#1`
(pdftex.rs:611, Perl pdfTeX.pool.ltxml:208 likewise); a faithful one needs the engine's primitive meanings kept in
State past the TeX pools (before plain_bootstrap) — one paper, deferred. Remaining 2609 Fatal witnesses (root causes in
`~/data/pk_agents/main/HANDOFF.md`): 2609.07018 (nested tikzpicture under
`\widetilde`), 2609.14043 (tikz-timing). Guards
`perfect_kernel_batch61::{aastex7_rotate_and_rotatetable, math_active_delimiter_is_its_character,
math_active_character_without_meaning_is_itself, autobreak_lines_after_the_left_side}`.

62x: AASTeX's table columns (aas_support_sty.rs), ported from aastex701.cls (TL 2025 copy). `D` decimal columns
(:12010): after `\decimals` a cell's first word splits at its first `.` (`\lookfordecimal`, :11980), both parts in math
(a sign is a minus), the point only before a fraction, the rest of the cell kept; without `\decimals` the cell stays whole
(each deluxetable resets `\decimals`, :11330; Perl's split uses an undefined `\lx@alignment@align`, KNOWN_PERL_ERRORS
#493). `C`/`L`/`R` are the class's math cells
(:8857-8859), with `$` active in every table (:8849, tabular's and deluxetable's bindings) and doing nothing there, as in
the class's `\nodata` (:8268-8269); `\centerwidetable` (:13497) and the `\movetabledown`/`\movetableright` registers
(:11756, 13690) defined. OXIDIZED_DESIGN_DIVERGENCES #450. Kernel: a `$$` display opens and closes on a math shift by
meaning (tex.web §1138, §1197; tex_math.rs `next_is_math_shift`), so the active `$` pairs in a `p{}` cell; a
`split_tokens_delimited` math span closes as it opens, by meaning.
2609.05675 102 → 0 errors (was Fatal), 2609.00308 102 → 0 (was Fatal), 2609.06985 63 → 0, 2609.04324 12 → 0,
2609.06112 2 → 0; A/B over the 157 aastex papers in the first 8000 2609 ids (62w3 → 62x2): 5 improved, 0 regressed,
10 outputs changed — the 5, 3 more whose `C`/`L` cells are now one math expression (2609.00140, 2609.00283,
2609.07836: `-16.44$\pm$0.07` → `−16.44±0.07`), and 2 whose `\nodata` keeps the class's surrounding spaces. Open: a
`\colhead` over a `D` column covers its integer half only (the class spans both, :12027-12028 `\CheckNumberAndSwitch`;
no witness, `\twocolhead` is the documented form), and the class's `d` (:12011) is the hidden pair, which this binding,
like Perl's, prints as `D`. Guards `perfect_kernel_batch61::aastex_decimal_and_math_columns`,
`perfect_kernel_batch61::active_math_shift_pairs_for_display_math`.

62w: the ar5iv profile's `iflimit` 16M → 48M (ar5iv_sty.rs). Five 2609 pgfplots/tikz papers ended `Fatal:Timeout:IfLimit`
with no error before it; measured without the limit (a release probe build counting conditionals), three are finite and
complete: 2609.10563 (19M, 48 s), 2609.16075 (21M, 45 s), 2609.07725 (39M, 78 s); so do 2605.27177 (raw mhchem, 56 s)
and 2605.04377 (98 s). Any runaway still stops at the worker's 180 s per-document timeout (the dispatcher lease is
240 s); the limit only ends a faster one sooner with its label, and one between ≈89k and ≈267k cond/s now ends at
the timeout. Outcomes that change label: 2609.30783 (lirseg, a memory runaway) now ends at the box-list memory budget
(`Fatal:Stomach:MemoryBudget`, which rides the memory cap: 8 GB probe, 5.25 GiB in the cortex fleet), and 2609.08966 (pgfplots
`shader=interp`, slow) and 2605.12601 at the 180 s timeout — performance items (open). Guard
`perfect_kernel_batch61::ar5iv_profile_sets_its_runaway_limits`.

62u (REGRESSION 2026-10-05, critical): 12,144 of 38,624 2609 papers ended as Error with expl3's "Mismatched LaTeX
support files" + "Cannot run piped system commands" (expl3.sty:64-78). Cause: the worker's `LATEXML_DUMP_DIR`
(`resources/dumps`) held dumps built from `/usr/local/texlive/2025` (L3 2025-11-06), copied there from a gates dump
dir on 2026-10-04 21:28, while the worker reads packages from `/usr/share/texlive` (L3 2026-01-19); every engine
revision fails the same way with that dump (61m17..62s11), all pass with a dump built from the runtime tree. Fix: the
dumps regenerated by `tools/make_formats.sh` in the worker's environment; a dump now records the format files it was
built from (`# source` header: `latex.ltx` + `expl3-code.tex` / `plain.tex`, size + CRC-32) and every loader passes
over one this TeX tree does not have (`dump_paths::dump_matches_tree`: `resolve_versioned_in_dir`, the embedded
dumps, `LATEXML_DUMP_PATH`), the files looked up by tree subpath so a document's own `expl3-code.tex` never
counts; when none matches, `latexml::format_dumps` builds the format in process, once per tree and build, in a
directory with no TeX files, under a lock, into a per-user cache (`~/.cache/latexml-oxide/formats/`,
`$LATEXML_FORMAT_CACHE`), keeps it only if the build logged no error, and uses it, else the base branch (DUMP_DESIGN
"Tree match", DIVERGENCES #449). Dev dumps from before the stamps are passed over: regenerate them with
`tools/make_formats.sh`. Guard `dump_gate_init::a_dump_built_from_another_tree_is_not_used` (an older tree's forged
dumps — bogus stamps, L3 date 2000-01-01 — as the only dumps allowed: expl3's mismatch on the old engine; now both
formats built into the cache with this tree's stamps and loaded from there, on any TeX Live; the document's own
`expl3-code.tex` never read; matching dumps load and build nothing). Perf audit: the stamps cost a CLI process +0.6M
instructions (+0.09%, 78.2 → 78.9 ms on a one-line article, within noise) and `cortex_worker` once per child
(DUMP_DESIGN "Tree match"). 62v: per format, builds that have not succeeded are counted
(`.attempts.<kind>`, so a build dying with its process counts), two in a row mark it `.failed.<kind>` for a day, and a
process that waited while a build failed does not retry it at once; a dump on disk wins over the cache's, the embedded
one comes after it; a failed build is a counted `Warning:dump:build_failed` in each conversion's log; the embedded dumps'
`$TMPDIR` cache is per user and owner-checked (`latexml_core::util::private_files`, unit-tested).
Rerun: run 332 (error/errmessage), 1,532 of whose papers converted during a dump-less minute of a second regeneration
and need regenerating again (dumps must be swapped atomically under a running fleet).

62r: textcomp's symbols are TS1 text symbols that dispatch on the encoding (KPE #489): Greek `\textmu` and babel
greek's `\figurename` print μ, not µ; a document's per-encoding `\DeclareTextCommand` takes effect.

62q: a captioned minipage or parbox in a float is the float its caption numbers (DIVERGENCES #447, KPE #488): its
panel takes the caption's id, tags, list and `\label`s, a table's are tables — side-by-side captioned minipages read
"See 1 and 2", not "2 and 2" (shared with Perl; the common idiom, and 1601.03744's tabular cells).

62p: graphicx's pdftex driver chain at `\begin{document}` (pdftex.def → epstopdf-base → pdftexcmds → iftex, when
`\@curroptions` is not empty) defines `\ifpdf` afresh (KPE #487): a paper's `\let\ifpdf\relax` no longer leaves
JINST's `\label` a stray `\fi` (1310.6454, a Fatal). `\ProcessOptions` and the package loader keep `\@curroptions`
as latex.ltx does, key=value processors leave it (40 bindings say so: `ProcessKeyOptions!`/`key_options_processed`);
amsmath passes `namelimits` to amsopn. 31 synthetic probes agree with pdflatex on iftex/pdftexcmds loading.

62o: a captioned minipage in a tabular cell is a figure panel in the cell (KPE #486): the box placement no longer climbs
out of an alignment (it left `<td>` open in the figure, 5 malformed errors, 1601.03744), and caption material left in a
box in running text becomes a figure panel in an inline logical block (labels: 62q).

62n: tocloft keeps the kernel's lists (DIVERGENCES #446): interpreted raw for its `\cft…` parameters, its
`\tableofcontents`/`\listoffigures`/`\listoftables` replaced by the kernel's under its own condition and times —
they ran `\@starttoc` on a `.toc` LaTeXML never writes, losing the `<TOC>` (SciPost.cls; shared with Perl).

62m: the ACM SIG classes (sig-alternate, sigchi; shipped, run raw since 62k) keep their authors: `\alignauthor` is of the
`\and` family, its brace group and `\affaddr` read as their content (DIVERGENCES #445; open: 1609.00045, 1608.06253), and a missing `figure`/`table` counter is made
once the class has loaded (KPE #485, sig-alternate.cls:699 `\@ifundefined{figure}`). 1605.02827 18 → 1 errors,
2003.09061 24 → 0 (4 creators with affiliations, OmniBus merged them), 1607.07514 22 → 0, 1906.01122 17 → 0,
1707.05754 30 → 0.

62l: a PASJ binding (`latexml_contrib` `pasj00_cls.rs`, also pasj01/pasj02): the shipped class is interpreted raw and
the kernel `\caption` (`\lx@caption`) put back over PASJ's own, which bypassed `\@caption` (captions without number,
dangling labels since 62k); `\@maketitle` emptied; `\KeyWords`, `\affil`, `\altaffiltext`, `\email`, `\orcid`,
`\Received`/`\Accepted` go to the frontmatter; the binding splits the one `\author` itself (commas, `\&`, `\\`; a
name's marks follow its comma). 11 PASJ papers: 0 errors
but 1601.03744 (3, a shared kernel defect, RED), captions back (≥ 62j4), keywords and affiliations gained. RED:
`\caption[<number>]`, captions in minipages in a tabular.

62k: the arXiv profile interprets a shipped class without a binding raw (`localrawclasses` in ar5iv.sty; DIVERGENCES
#444; a binding route, Perl's prefix alternate, Rust's case step or a path-prefixed class's basename binding, still
wins): OmniBus-class papers 1,151 → 220 errors and 2 → 89 error-free (127), random run-329 error papers 4,645 → 2,714
and 45 → 155 error-free (383), general sample 752 → 691 errors (873), no body text lost. With it: extsizes bindings
(`\@ptsize` in points), `\documentstyle` binding-first probes and a raw `.cls` probe, tocbibind conditionals (KPE
#482), ragged2e `\LaTeX*` saves under `newcommands` (KPE #481), IEEEtran `\ifCLASSINFOpdf` from `\pdfoutput` (KPE
#480, 22 papers), and four kernel paths raw classes reach: an empty `\@startsection` type (KPE #483), a re-let
`\@startsection` recursing through `\@sect` (`\lx@startsection`), the `\@maketitle` deposit's diagnostics hold, the
locked `\NAT@wrout` (KPE #484). Open, worse than OmniBus (DIVERGENCES #444, RED repros): a paper's `\let\ifpdf\relax`
that graphicx's driver undoes in TeX (JINST, Fatal), JINST's locked-`\author` flag, raa crossed groups,
`\alignauthor`/`\newauthor` vs `\lx@personname`, `\@ifundefined{figure}` seeing the kernel's environment
(sig-alternate).

62j: elsarticle `\jtype` (KPE #478) and txfonts `\varv`/`\varw`/`\vary` (KPE #479) fixed; the run's one panic,
1111.1991 (Rust-only: an auto-opened `ltx:picture` sized the `{picture}` whatsit being absorbed and stored the size
memo on it — "RefCell already borrowed"), fixed by `set_memo_property` (a busy box keeps no memo; any other re-entrant
property write is an Error, not a panic). Residuals: `Digested::compute_size` answers 0 for any box it cannot borrow
mutably — one being absorbed too, unless all three of its sizes are requested (1111.1991 passes because the
`{picture}` carries them), not only the astro-ph0310145 self-reference; a `\parbox` after a text `\put` sizes the
auto-opened picture 204.16×22.56 where Perl gives 7.3×5.96 (pre-62j); 0709.1145's `\LT@array`.

From the 62i review: amsmath `multline` drops a `\tag` and numbers the equation (pdflatex "(Q)"; the counter then
runs one ahead) — RED `repros/alignment-bindings/multline_tag_replaces_the_number.tex`.

From the 62h in-scope manual pass (each pdflatex-clean, so Rust-only; both were Fatals on 62g1 and stay failures):
- typog-example: Rust's `\fontname\font` gives the fallback `cmss9` for an NFSS family (`Inter-LF`, T1Inter-LF.fd)
  where pdflatex gives `Inter-Regular-lf-t1 at 9.0pt`, so the document's `\projectoutfontname#1-#2-#3\relax`
  (typog-example.tex:263-264, inside an `\edef`) finds no `-`. 62g1: the argument ran to the end of the file (Fatal);
  62h: "Paragraph ended before \projectoutfontname" ×11 (tex.web §395's "extra }" at the `\edef`'s close is not
  implemented), then the run times out near line 1813.
- chessboard_and_beamer: skak's `\mainline` under beamer `\only<n>` reports "black, not white, to move", then
  `\EatNumberA(#1.#2)` (skak.sty:1319) finds no `.` — 62g1 Fatal at the end of the file, 62h 4 errors.

From run 329 (62a-62c):
- `\jobname.aux` re-read: FIXED 66f — `\begin{document}` opens the `.aux` as latex.ltx's `\document` does (OD #576);
  lamuphys.sty:1240-1247, caosp.sty:916, jpsj3.cls:90-96 and bio.cls `\input \jobname.aux` at `\enddocument`
  (cond-mat9607109, astro-ph9805185, 2605.24927, 2605.04825, 2605.28673, 2605.20125: 1 error → 0).
- Still Fatal after 62b, each erring under pdflatex too: 1409.3401 (pdflatex emergency stop), 2105.00771 (101 = 101), cs0702042 (citesort.sty missing from TL; Rust
  then 101 math-alignment errors, pdflatex 3).
- Rust-only, still open: CJK GB `\@inpenc@undefined` (1007.1512); singles 1205.5844 (`\@journal`), 1711.06710
  (`\@rticle@options`), 1811.00686, 1907.03566, 2004.12109 (memoir font command), 2105.02164, 2203.13766, 2207.02360,
  2306.10394, 2409.00304.
- AMSTeX `\documentstyle{amsppt}` documents still get the LaTeX format (Perl reads them plain). Kept plain they measured
  worse: 64 sampled 1996-99 amsppt papers, 192 → 319 errors, 1 → 2 Fatal (math9806005 0 → `TooManyErrors`; plain
  AMSTeX lacks `\ams@return@opt@arg`, `\rightpoint`; math9603201, math9709201, math9801043, math9803037). Fix the
  plain-AMSTeX gaps first, then classify them plain (`converter.rs` `compile_route`); sample and A/B in
  `~/data/pk_agents/main/scratch_g/run329/amsppt/`.
- `\input eplain` (pre-existing, tex clean): `Error:undefined:\auxfile`, then `Fatal:ParamSpec` "Parameters for `\@`
  not in order" (Perl: the one error).

From 62d/62e: Open, pre-existing (review probes `scratch_g/review62e/r3/r4.tex`, `r5.tex`): with
`linesnumbered` the first statement right after `\caption` takes no number; `{quote}`/`{center}`/`{flushleft}` end no
algorithm line, so the text after them stays on their line; a statement inside `\href` breaks inside the link
(`ltx:ref` does not auto-close). Not fixed: missing journal classes (raa `\pagerange`/`\volnopage`, ws-* `\bodymatter`,
iopart `\ioptwocol`, old A&A `\thesaurus`), text-mode `_`/`&`/`^`, and the Fatals `TooManyErrors` (49),
`never_completed` (24), `PushbackLimit` (21 then, 86 by 126k papers; fixed for harvmac by 62f below).

From 62f: Side risk, open: any genuine `\@ifnextchar` autoload in a plain
  document still replaces the document's macros through the dump apply.
- 62i fixed six of the 17 PushbackLimit Fatals left after 62f: 2108.13640 (siunitx `S` cell read past csvsimple's
  `\relax`, KPE #474), 2407.10582 + 2505.05474 (Rust-only: a document's own `\emails`/`\affiliations` taken for an
  IJCAI author-block marker, `has_ijcai_section_marker`), 2309.08676 + 2502.21053 (`\DeclareEmphSequence`:
  `\selectfont` defined no font identifier, KPE #477), 2408.12869 (`\tag` text mentioning `\theequation`, KPE #475),
  2305.06365 (`\let\@@cite\cite`, KPE #476 / DIVERGENCES #443; its revtex `\@AF@join`/`\twocolumn@sw` stay
  undefined). 0902.2281 and 1007.3028 are AMSTeX documents (`\input amstex` + amsppt: label.def needs amstex.tex's
  `\W@`, `\eat@`), the amsppt-as-plain residual. 2408.12869 then shows 3 `<ltx:XMArray>` in `<ltx:figure>` errors.
- Open, still Fatal: 2302.07191, 2501.17908,
  2509.10120 (error before run 329, Fatal now); unclassified: 1807.10890, 1807.08405, 2107.07104, dg-ga9410001,
  2406.19307. A `\noexpand`-ed undefined control sequence raises "undefined" in csvsimple's
  space trim where TeX treats it as `\relax` (`g62f/c_m.tex`, a CSV cell `\foo` with `\foo` undefined).


### ar5iv tracker residuals (frozen sweep: `archive/AR5IV_DIAGNOSTICS_2026-08-14.md`)
- **Close-out pending:** the screened issues (the ~48 already 0-error + the 16 fixed by PR #306) are still OPEN on
  dginev/ar5iv (2026-09-27) but for #503 and #555 (closed 2026-07-19); #546, #550, #598 went 0-error on 2026-07-20; close via a maintainer batch list after an ar5iv redeploy, spot-checking each reported
  symptom — never post unilaterally.
- **Rust-only:** #472 `2311.06609` siamart paper-local `code` env (`list`+`tabbing`+`\mathcode`+`\mynewline`, inline
  `$…$`), 82 vs Perl 39 (the 07-18 reclassification; 17 in the archive's 10×-parallel table); #556 `2508.07407` tikz `calc`-coordinate cloud loop, contained (31 KB salvage;
  `docs/reproducers/tikz_calc_node_recursion_2508.07407.tex`).
- **Unclassified since 2026-07-20:** #520 `2412.06264` 337 `\or` at `\end{document}` (deferred-content `\ifcase` leak);
  `2305.05665` 33 macro-generated `unexpected:_` (not axessibility, not ar5iv-only); #558 `2301.12995` 16
  `\@end@tabular`; #473 `2310.17416` 9 errors after the blkarray binding, whose `block` `(`/`[` sub-region delimiters are
  dropped (blkarray_sty.rs:14-21).
- **Grew since the sweep:** #576 `2511.16624` — 4 → 2 errors after the fairmeta binding, now 9 in cortex 2026-08-26 (8×
  `\noalign`, `{subfigure}` undefined): a new cause, untriaged.
- **Shared, surpass candidate:** `2604.16007` acmart `\@ACM@balancefalse`/`\@ACM@pbalancefalse` undefined (acmart.cls:123
  `\define@boolkey`; neither binding defines them).

---

## Standing policies & method

### Methodology & the cortex cross-join
- Working method: **re-triage LARGE-error papers** (the single-error tail is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose` → fix the divergence.
- Cortex API: `http://127.0.0.1:8000/api`. Endpoints: `/api/reports/<corpus>/oxidized-tex-to-html/<severity>` and `/api/corpus/<corpus>/tex_to_html/document/<id>`.

### CSS themes — `ar5iv.css` vs `LaTeXML.css`
- `ar5iv.css` is actively developed (`~/git/ar5iv-css`). Base `LaTeXML.css` is a faithful copy of upstream Perl LaTeXML's default theme; bugs in base CSS route upstream to `brucemiller/LaTeXML`.
- Sizing intent gap (witness #721, ar5iv#83): Preserve absolute (`7in`) vs relative (`\textwidth`) intent in `image.rs:186` instead of flattening both to `pt`.

### Algorithm Markup + CSS Unification (0.7.7 Target)
- Part 2 landed (line numbers, captions, CSS mirrors).
- Shared markup class: Assign every algorithm listing a shared marker class (regardless of surrounding env) so one generic CSS rule targets it in both `LaTeXML.css` and `ar5iv.css`. Details in [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md).

### CLI options policy (Option-C) + `validate()`
- Wire only options whose engine feature works end-to-end; strict clap parser (no accept-and-warn stubs).
- `--validate`: STUB today (`latexml_post/src/document.rs:1717`). Requires safe RelaxNG bindings in `rust-libxml` published to crates.io before wiring.

---

## Parked families — pointers to dedicated docs

| family | doc |
|---|---|
| Environment markup class (`ltx_env_<name>`) | [`parity/ENV_MARKUP_DESIGN.md`](parity/ENV_MARKUP_DESIGN.md) (Phase 2 / Post-Release) |
| Beyond-Perl performance levers (BP-1…BP-6) | [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| Content-MathML & math parser gaps | [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| Deep deferred families (`.bst`, xy-pic, etc.) | [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) |
| Stage 4 WASM bring-up plan | [`release/WASM_COMPATIBILITY_PLAN.md`](release/WASM_COMPATIBILITY_PLAN.md) |
| Streaming core DOM design | [`archive/STREAMING_CORE_DESIGN_2026-07-29.md`](archive/STREAMING_CORE_DESIGN_2026-07-29.md) |
| Two-pass streaming split | [`archive/STREAMING_POST_DESIGN_2026-07-06.md`](archive/STREAMING_POST_DESIGN_2026-07-06.md) |
| Multi-document streaming post-join | [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) |

---

## Reference & stable notes

- **SVG picture path:** `<ltx:picture>` is converted on the live page DOM by `latexml_post::svg::SVG` (the faithful `SVG.pm` port) before MathML/XSLT — Perl's chain order — since batch 56bz (2026-09-17); the former regex fragment table + post-XSLT placeholder splice in `post.rs` is deleted. The libxml2 `PostDocument` use-after-free that kept the DOM port benched was fixed by the idcache relinking in `PostDocument::drop` (#480) on libxml 0.3.21; `crossref.rs`'s live-DOM `replace_node` was the production precedent. Guards `cluster_xslt_split::picture_svg_on_the_live_dom::*`.
- **Picture `\unitlength` sizing:** Inkscape `.pdf_tex` pictures ignore `\unitlength` during core `{picture}` sizing, producing degenerate (sub-pixel) outer SVGs; the picture-nested figure still renders because the XSLT emits it inside an `overflow="visible"` `<foreignObject>` at its own size (the former string-path `DEGENERATE_SVG_PX` special case went with batch 56bz). The core-side `\unitlength` gap is open.
- **Primitive layer:** Audited faithful (2026-06-20); core arithmetic, glue, conditionals, and token tables match Perl byte-for-byte.
