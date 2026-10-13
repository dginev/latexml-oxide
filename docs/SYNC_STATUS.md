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
| **G** | **Run 336** (full arXiv, 2,947,191 papers; 62zu `c0e4393f39` to 2026-10-07 16:35Z, then `9569ca27a7`) — COMPLETE 2026-10-10 07:10Z; vs run 306 (snapshot 2026-09-17): error 12.86 % → 6.32 %, fatal 0.87 % → 0.27 %; 203,366 papers out of error, 20,570 out of fatal; 16,827 worse into error/fatal; no_problem → warning 290,846 (the warning-visibility shift); E/F rerun **run 338** (e57be39640, complete 2026-10-12): of run 336's 186,160 error / 8,019 fatal, 42,143 left error and 2,509 left fatal (corpus error 6.32 % → 4.89 %, fatal 0.27 % → 0.19 %); 142 error → fatal to triage; **run 340** (E/F rerun on 1bd0c44f59: 64zf + 65b + 65c, complete 2026-10-12 15:19Z): of run 338's 144,214 error / 5,652 fatal, 2,282 left error and 305 left fatal (corpus error 141,799, fatal 5,634); 287 error → fatal — 204 TooManyErrors (107 = longtable undefined under aa/pasj00/emulateapj, a 65b regression, batch 65h), 59 cortex never_completed (dispatcher lease clock started at batch fetch, fixed in CorTeX a5bbbee5, not yet deployed), 18 PushbackLimit | **TRIAGED** (2026-10-10): 1,571 clusters of the 16,827 (`~/data/pk_agents/main/run336/worse_306_336.tsv`, `agent_reports/2026-10-10_run336_*`, `_rc336_*`). On head already clean: CCSXML EOF (64d), imageprocessing (AppArmor), most `never_completed`. Faithful (pdflatex errs too, `\errmessage` an Error since 56g): ctex fandol 107, l3backend twice 71, acro `class` 52, `\DeclareMathOperator{\free_var}` 155, `\verb` misuse 183, JINST.cls under TL 2025 39. Fixed in 64h (~2,500 papers): `\hyper@makecurrent` patchable 129 (with the kernel's `\theH<counter>` names, OD #480), `pdftex` option / PostScript-only K6 cue 47, CJK stray bytes 746 + byte-mouth `.bib` 4, tcilatex `\thechapter` 537 + plain `\input #1` 2, aipproc `\tablehead` ~428, elsart wrappers 367, algorithmicx nested listings 134, spverbatim 107, nomencl prefix 12, `\diaghead` pair. Fixed in 64k (OD #482): raw classes' refused `\author`/`\abstract` replayed for their flags, stores kept the kernel's (JINST/JHEP/CECS 340, iosart2x 71), the Foundation Compositio family (224) and JINST/CECS bindings (semantic contacts, keywords, MSC, dates; no second title page), each reading the class file the document asked for (a path-prefixed `JINST-Sample-files/JINST`, a renamed `JINST_mod`) — A/B over 2,918 papers (64h4 → 64k16, the 238 the email rounds touch re-run on the final 64k22): with errors 809 → 252, with Fatals 72 → 2, no creator or abstract lost; a labelled email line of a marked author block is now its author's contact, or the block's (OD #52(m); addresses on several authors 66 → 4). Open: the 31 renamed JHEP3 copies (QCEMDA, AKRSX, jhepLike…) need a class route by content; `blkarray` `\begin{block}{[…]c}` raises `\else` errors (2112.09933, reached once its frontmatter stopped failing); a Semiverbatim URL contact's `~` prints as the OT1 tilde accent `˜` (`\urladdr{…/~y}`, Perl the same; RED sectioning-frontmatter/url_contact_keeps_tilde). Fixed in 64qb (KPE #563-567, OD #483): paspconf bound to aaspp (+ its Q&A, `\deg`, keywords as RDFa) and RevTeX 3's keyless `\reference` (~860), a caption2 binding (843), amsmath's `\macc@` chain with `\skewchar` a font register (817), `\tablebreak` ends the row (38), `\sidecaption[pos]` (186), jmlr2e `\keywords{}` (7), `Digested::to_attribute` ported (a skip in an attribute is its spaces, not `\enskip` text; Perl List/Whatsit `toAttribute`); A/B 332 papers: with errors 301 → 37, none worse. Fixed in 64p (run 336's loops, KPE #475/#568, OD #315): a class's `\renewenvironment{thebibliography}` that a binding's hook runs unlocked is refused, the lock read as a value as Perl reads it (wiley2sp & co, 132), and a class's own bibliography list — `{thebibliography}` `\let` to it (ajour, cjour, IEEE-Con-Sys-mag) or its `{references}` — gives way to the kernel's at the first `\bibitem` (RED: a class end printing text after its list prints it before the entries; a class bibliography built on `{enumerate}` leaves a group open — both synthetic; OD #484, KPE #569); `\tag{\thesubeqn}` (74); mathfixs' autobold `\let` of a robust switch (33); a document's `\mathaccentV` accent (21; its `tex` attribute now reads `\mathaccent 28766{G}` where 62d's named reading gave a `.bbl`'s `\hat{G}` — a `\mathaccentV` reversion is open). RED: keyval runs a key given no value that has no default with an empty value (keyval.sty:52-56 errors and skips it; csquotes `[english]` loops, 5; `macro-state/keyval_bare_key_without_default`). Fixed in 64qa (KPE #570-573, OD #485): World Scientific `\body`/`\bodymatter` and table rules (ws-procs9x6, ws-procs961x669, ws-rv9x6; ~1,078), iopart `\ioptwocol` (1,058), revtex `\rev@citealpnum` (1,023), svmult `\toctitle`/`\tocauthor` (960), mdpi `\history`/`\changeurlcolor` (765), WileyNJD `\state` (701), the ptephy author block (204 → 27 papers with errors), quantumarticle tikz (61 → 4). Open: epsfig `\epsfbox` a no-op (KPE #574, its own batch with an A/B); mdpi's other history fields gated by status like `\history` (display-toggle rule); the ws classes' packages (rotating, dcolumn, url, chapterbib, cite[super,nospace]: bindings load what the class loads). Fixed in 64s (OD #486, KPE #575): ar5iv's `\today` is the paper's source date (the newest TeX source entry time of its zip/tar), `\relax` only for an undated bundle; ptapap's `\today` parse 95 (1801.05985), `<date>` filled on 43 of 60 sampled `\today` papers; open: ptapap `\let\oldfnum@figure=\fnum@figure` undefined (92 papers). Fixed in 64ra (KPE #576-581, OD #487-491): `$$` is display math in IEEEtran `{IEEEproof}`, lineno `{linenomath}`/`{linenumbers*}`, OmniBus `{frontmatter}` and autart `{pf}` (macros as the classes define them, not boxed DefEnvironments; ~1,600 IEEEproof, ~110 linenomath, ~200 frontmatter, ~90 autart papers), IEEEproof titled by its optional argument with IEEEtran's own QED switch (`\IEEEQEDhere`/`\IEEEQEDoff`/new `\IEEEQEDhereeqn`, one tag "∎(1)"), the class QED symbols sized as their boxes so a cell holding only the QED is kept, and `\IEEEeqnarray`/`IEEEeqnarraybox` columns built from the IEEEtrantools column table (≤1k papers); A/B: IEEEproof 60 → 1 papers with errors, linenomath 40 → 3, `\endlinenomath` 30 → 1, frontmatter 39 → 12, autart 26 → 4, IEEEeqnarray 80 → 8, 30 extra-tab eqnarray papers byte-identical. Open: IEEEtrantools' own IEEEproof/QED commands for non-IEEEtran classes (RED `block-model/ieeetrantools_ieeeproof_any_class`), IEEEeqnarray `[decl]`/struts/`\IEEEeqnarrayrulerow`, 64 more mode-less `"#body"` DefEnvironments (census in `agent_reports/2026-10-10_payload_64ra.md`). Fixed in 64w (OD #492): an axodraw v1 binding (not in TL; 1,507 papers with `\Text` undefined, 149 as the first error) evaluating v1's PostScript procedures (gluons, photons, arcs, arrows, blobs, boxes, ovals, curves, axes; PostScript-arithmetic arguments over dvips-header constants) into the picture model, `\Text` at `\unitlength`s unscaled, colordvi loaded as the colordvi-aware v1 does; it also answers papers shipping their own axodraw.sty (drawn now, errors equal), while axodraw2 (raw TL file, kept off the versioned fallback) and axodraw4j stay raw. A/B vs e1ec58c24e: 149 witnesses with errors 149 → 10, errors 872 → 94; 63 shipping, 16 axodraw4j, 8 axodraw2 papers unchanged in errors and warnings. Port gap closed alongside: the SVG of an `ltx:arc` keeps its stroke attributes, as SVG.pm `convertArc` copies them (L385-387) — every arc, pstricks' `\psarc` too, had been invisible. Open RED: a picture label mixing text and math overprints in the SVG (graphics-axodraw/picture_label_mixed_text_and_math, hep-ph0509118); a math label in a `\vcenter{\hbox{picture}}` is a bare `<msub>` without `<math>` (graphics-axodraw/picture_math_label_in_vcenter_box); `\hspace*` before a picture is dropped (graphics-axodraw/picture_after_hspace_star); colordvi's `\Red{…}` sets no color (fonts-nfss/colordvi_named_text_color); an article `\author` block's second name after `\\[12pt]` is read as an affiliation (sectioning-frontmatter/article_author_block_second_author_after_spaced_break, gr-qc9906006). Fixed in 64u (OD #493-496, KPE #582): bindings for journal styles neither in TL nor shipped — l-aa/laa/l-aa-ps as the `aa` class with `\thesaurus` codes, argument-less `\keywords` and `$^n$`-marked institutes (843), epl2/epl (740), OSA opex3/ol2/osajnl/osajnl2 with appending authors and email lines and `\homepage` placed by mark, spelled name or group through the new kernel `\lx@add@contact@placed` (759), REVTeX 2.1 `{title}`/`{instit}`/`\receipt`/`\nonum` (750); A/B vs 64ra: l-aa 126 papers with errors 126 → 5, EPL 142: 117 → 1, OSA 622 shipping no style: 622 → 7 and 129 shipping one: title 40 → 129 papers, creators 7 → 129 (1104.0806's private abbreviations lost: 5 errors), REVTeX 120: 86 → 9; no keyword, abstract or affiliation text lost, no email moved off its owner. No generic fallback for a missing `\usepackage` (user ruling 2026-10-10). Open: under `\documentstyle[aps]{revtex}` the OmniBus overlay gives an `\author{A and B}` group's address to B only (RED `revtex3_omnibus_author_group_address`; ruling asked); `\documentstyle[l-aa]{article}`; marks that are a paper's own macros (1201.5125). Fixed in 64y (OD #497-499, KPE #583-585): an environment frame closed by `}`/`\endgroup` (M1, ≈2,060 run-336 papers) ends as tex.web §1064 `off_save` does, the diagnostic following the group kind the begin code leaves (none | semi_simple | box; 71 environments audited); 17/17 witnesses clean (139 → 0 errors), 150 of the cluster 141 → 82 erroring (1172 → 642); 0 regressions over 300 random + 100 theorem/list-heavy + 70 reviewer papers; REVTeX 3 figure list entries (cond-mat9607130 1 → 0) and natbib `\bibpostamble` (2009.08029 1 → 0). RED follow-ups in `boxes-groups/`: `atbegin_hook_group_above_env_frame` (2001.02707), `bare_theorem_ended_by_enclosing_end`, `amsthm_bare_proof_closed_by_brace`/`amsthm_proof_closed_by_endgroup` (the proof is a constructor, so a group closer silently swallows the document), `footnote_bare_quote`, `macro_env_ended_by_inner_bare_end`, `bare_doublespace_closed_by_brace` (2310.08233, setspace's no-op bare `\doublespace`); 2106.02397's float.sty binding no-op `\float@end` under floatrow; `osa,floats` REVTeX 3 figures. Fixed in 64zb (OD #500): a locked binding owns its calls (D1, superseding the arity rule below) — a document's parameterless redefinition of a binding with a braced signature is the meaning for every use without an argument (pgffor `\foreach \i/\text`, `\@for\text:=`, `\def\title{…}` as a value), a `{`/`[`/`*` call still runs the binding (kept as `\lx@locked@<cs>`), a redefinition with parameters (robust and ltcmd forms included) stays refused, `@` names and 0-parameter readers stay absolute; A/B vs 8542e1b1a5: M5 cluster 369 papers errors 11,882 → 2,500 (140 fewer, 125 to zero, none more), census 314/320 identical, restyle 100/100, random 298/300 (2 fewer errors); witnesses 2405.18512 45 → 0, 1406.2544 31 → 0, 1511.05179 1 → 0. Accepted residual: an unbraced parameterless forwarder runs the author's code (`\mathcal C` after `\def\mathcal{\mathscr}`; 1806.09639, 1904.10871 `\sqrt` hack). RED: `macro-state/m5_locked_name_in_expansion_only_context` (`\lx@locked@<cs>` in `\meaning`/`\typeout`), `sectioning-frontmatter/paragraph_title_trailing_gobbler_open_group` (2604.23335, pre-existing); m5_e2/m5_d2 deferred (rulings 4/5). Fixed in 64zc (OD #501-503, KPE #586): an email or web address keeps its `_` and `~` in the font its class sets it in — typewriter (`\lx@address@urlfont`: REVTeX 4.0, AASTeX 5.x, amsart/amsproc/amsbook, llncs, JHEP, jheppub, iopart, aa, elsarticle, cas, imsart) or roman (`\lx@address@textfont`, user ruling 2026-10-11: REVTeX 4.1/4.2, AASTeX 6+, emulateapj, svjour3/svmult, quantumarticle, jmlr, pasj00, ptephy), the author block's contacts made of a class's `\email` call too, `\~{}` in a typewriter address a `~` of its run, the link of a split address its whole text, `\i`/`\j` kept in the ASCII encoding (1001.1076 "aiwen˙fan"; RED `url_contact_keeps_tilde` green); A/B vs 64u over 175 sampled underscore-email papers (7 months of run 336), 100 random and 31 review papers: no Error/Fatal/Warning count changes, papers with a `˙` email/url contact 60 → 0, every XML change on those sets on an email/url contact (font, or `˙`/`˜`/`ˆ` → `_`/`~`/`^`), no address left split into a one-character link. Beyond contacts, the accent of an empty letter is the current font's (OD #503): 6 of 13 sampled `\~{}` papers change in the body, each a run joined or a typewriter `~` (1001.1367, 1001.1793, 1301.6104, 1301.6221, 1901.07113, 2101.11048). Open: a binding's Semiverbatim address freezes `$`/`^`, so an address's math prints as text and its link holds the `$`s (RED `sectioning-frontmatter/semiverbatim_address_freezes_math`; 1601.02576, 1601.02586, 1601.00888; Perl the same, the class of KPE #515); hyperref's `\url{http://x.org/\'{\i}}` prints the accent's expansion in its text and link (RED `sectioning-frontmatter/url_accent_command_leaks_expansion`). Fixed in 64zd (OD #504-505, KPE #574 fixed, #587): REVTeX 3's own options declared in revtex3_support (revtex.sty v3.0/v3.1 `\ds@` and the distribution's option files; `eqsecnum`, `preprint`, `osa`), its numbering as aps.sty ("II A", "II A 1", appendix "A 1", A1, an unnumbered appendix stepping the counter; seg A-1, rmp "II.A") and marked author lists split (OD #52(n): `\cite`, math surnames, "by", bare asterisks); `\compat@loadpackages` loads OmniBus provide-only (`ProvideOnlyGuard`: every binding definition during the load — OmniBus's and the bindings it requires — declines a name defined before it, environments whole, article not re-run, `extends` wrappers such as its `\bibitem` install anyway, TeX `\def`/`\let` untouched); bindings for the 2.09 option styles OmniBus used to stand in for: World Scientific `sprocl`/`mprocl`/`stwol` (appending `\author`/`\address`, `\abstracts`) and AASTeX substyles `aas2pp4`/`aaspptwo`/`apjpt4`/`aj_pt4` → aaspp (AASTeX 3's `{planotable}` ending its own data, astro-ph9502038); author lists (OD #52(n)): glued initials before capitals surnames (`M.CACCIA`), a doubled `\& and` separator, a name line with a `\thanks` between affiliations opening the next author when the block's first names carry one (hep-lat9404004); sprocl addresses led by an author's mark go to that author; AASTeX's obsolete `\authoremail` before `\author` the email of the author it spells, not a creator of its own (astro-ph9803018); packages OmniBus requires under provide-only are unmarked when it ends, so a later `\usepackage` loads them whole; epsfig `\epsfbox` includes its graphic; aas `{references}` titled; A/B against 44ff2ac956 (64zc) over 150 REVTeX 3, 40 REVTeX 2 `{instit}`, 149 other-class missing-option papers, 400 more option-stratified and 46 of the bound style families in `agent_reports/2026-10-10_payload_64zd.md`. Open: marked affiliations then an unmarked group's: the unmarked one reaches the marked authors too (RED `sectioning-frontmatter/marked_affiliations_then_unmarked_group_leaks`; revtex4 the same); an article author block's plain name line between plain affiliation lines stays an affiliation (RED `sectioning-frontmatter/article_plain_name_line_between_affiliations`); `\input` of a binding in a group a second time finds its definitions gone (RED `loader/input_binding_in_group_twice`, cond-mat9610182; TeX re-reads, Perl the same; low priority); hep-th9707026's glued `Sahng-kyoon.Yoo` list stays one creator; the ws classes' psfig/epsfig load (RED `singletons/ws_procs_psfig`, now unblocked by KPE #574). Fixed in 64x (OD #506-511, KPE #588-597): titlepage `{centering}`; natbib `\bibitem` outside any list reports Lonely `\item` (Perl's `malformed:ltx:bibitem`, once per `ibitem`; 1108.2593 40 Errors kept, structure recovered) and among a list's own items keeps its place (1411.7266 10 → 0); implicit-bibliography headings absorbed or untitled (no contents entry); biblist reopen with the phantom prune and emptied-list sweep at the bibliography's and document's close (147 → 6 phantoms over 127 papers, the 6 holding figure panels); only a TeX `\item`/`\bibitem` steps the counter (ACM `.bbl`-preamble ids now `bib.bibN`, user ruling); a `.bbl` bibliography after an implicit one prints again (KPE #596); AMS-TeX `\end`/`\predefine`; `\endabstract` locked against files; jheppub `\abstract`; PoS without JHEP journal macros + `\pos`. A/B 127 papers vs 8c7ed11a23: Errors 1986 → 1276, none above base, every drop a shape pdflatex prints cleanly (`scratch_64x/ab/drops10.txt`). Accepted residual: 2205.07302 keeps its words but loses its `\enquote` marks as `<ERROR>` (KPE #589, RED index-bib/bibitem_list_local_definition, deferred to K21). Open RED: kernel `\bibitem` in the body silent (KPE #595; Warning per ruling, batch 64zf stage 2); natbib `\bibitem` among an enumerate's items (cond-mat0003103, 1 Error; K21); a paragraph after a lonely natbib `\bibitem` with no heading between becomes a keyless entry (pdflatex: a paragraph; RED index-bib/natbib_lonely_bibitem_then_paragraph). Fixed in 64ze (OD #512-517, KPE #598-599; K22 census binding gaps): natbib entry interface (`\BibitemOpen`/`\BibitemShut`/`\bibinfo`/`\bibfield`; 80 → 2 papers with errors; a lonely bibliography spends no `\BibitemShut` stop); svmult/svmono (`{svgraybox}`, `\svhline`, `{prob}`/`{sol}`, `\motto`/`\chapauthor` via the after-heading hook, svmono as a book; 167 → 14); aipproc `\pacs`; svjour `\preprint`; interact/World Scientific `\bm`; IEEEAerospace; quantumarticle hyperref + REVTeX compatibility (PACS/preprint as RDFa, natbib numbers; 46 → 5); elsarticle bare `\frontmatter`; mn2e abbreviations per shipped copy (217 → 77); kernel: fallbacks yield to the author's shipped files, `\noalign\\` (2309.15248). A/B 1,530 documents vs e57be39640: no rises, 679 papers fewer with errors. Open: pasj00/PASJ95 absent-class emulation (~190 papers; own batch); quantumarticle `plainnat` default; svmult 2018 info boxes; RED `noalign_let_tabularcr`, `svmult_frontmatter_chapter_numbered`, `aipproc_nested_pacs` (0709.4622). Queued from the run-336 Error/Fatal ranking (`agent_reports/2026-10-10_run336_error_fatal_ranking.md`, triage `rc_undefA`/`rc_undefB`/`rc_struct`): 64q binding gaps behind first-error undefined commands (ws `\bodymatter` ~1,078, iopart `\ioptwocol` ~1,058, revtex4 `\rev@citealpnum` ~1,023, svmult `\toctitle` ~786, mdpi `\history` ~765, paspconf ~860, caption2 ~843, amsmath `\macc@` ~817, WileyNJD `\state` ~701, ptephy ~204); 64r roots behind stray `&`/`_`/`^` (`\lefteqn` as a 3-column span ≤2.2k, `$$` in boxed binding environments ≤1.7k, `IEEEeqnarray` column spec ≤1k, redefined `\span`, a braced amsmath body, `\tablebreak`, `\sidecaption`, jmlr2e `\keywords`). 64l pgfkeys native dispatch (~17), 64m ar5iv preload loads the format early (`\salta`, 84), 64n alignment cell-head peek (61), 64o delimited argument past an unmatched `}` (Fatal → Error), 64p loops (`\renewenvironment` lock 132, `\tag`/`\theequation` 76, `\let` wrapper 33, `\mathaccentV` 21, keyval 5). Ruled 2026-10-10 (queued 64t-64w): `&` as text in stored-but-untypeset metadata, bindings for l-aa/laa, epl2, OSA opex3/ol2/osajnl2 and REVTeX 2 `{instit}` (plus a generic raw-loading improvement), `\left`/`\right`/`\middle` unexpandable (62 IfLimit in xymatrix), an axodraw v1 SVG binding (1,507). Ruled later 2026-10-10: an environment frame closed by a plain `}`/`\endgroup` recovers as tex.web `off_save` does (M1, ~2,060, unparked: 64y); a locked binding yields only to a redefinition with a different number of mandatory arguments, after minimal repros show the rule well-founded (forwarding 0-argument redefinitions such as `\renewcommand\section{\@startsection…}` excluded; M5, ~160); a heading between `\bibitem`s is an inline sectional block inside the one bibliography (~750); a missing `\usepackage` file gets no OmniBus fallback, only `Warning:missing_file` (pdflatex fails too: 1210.7592 `opex3.sty` not found; shipped copies raw-load); REVTeX 3.0's real options declared in revtex3_support; an implicitly opened bibliography takes a preceding heading-like line as its title, else none (no `\refname`) | program | [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) close status |
| **KC** | **Kernel capability program** (user 2026-10-11): the open gaps grouped by the missing mechanism behind several of them — K24 one audited A/B comparator (P0), K20 gullet expansion fidelity (four tex.web algorithms: column ends by command code, `\left`/`\middle`/`\right` primitives, §395 extra `}`, peeking macros as one unexpandable step; absorbs queued 64n/64o/64v), K21 environments as TeX does them (`Streamed` vs `Captured`; absorbs the `#body` audit), K22 binding provenance and ownership (FileIdentity, owned sets, corpus-copy audit; version drift and author-shipped copies), K23 the author-block event model; next tier C3 placement policy (11,012 `malformed:` first errors), C6 content-aware emptiness, C7 string values for attributes | **ORDERED**: K24 LANDED (`ab_audit`, p0 `1e47551e6c`); manual sweep #149 → K20 G4 → G2 → G3 → G1 → K21, K22's read-only census/audit in parallel → K23 shadow mode | program | [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md) K20–K24; evidence [`KERNEL_CAPABILITY_GAPS_2026-10-11.md`](perfect_kernel/KERNEL_CAPABILITY_GAPS_2026-10-11.md) |
| **MS** | **TeX's math-group recovery** (main session 2026-10-12, task #204) — **65n landed for inline formulas outside alignments** (OD #553): a math shift meeting a `{`/`\begingroup` group opened in the formula inserts its closer and ends the formula there (§1193 + §1064-1065; 2007.11971 13 → 1 errors, 179 → 0 run-together runs, text words 5,268 → 9,865), a `\par` or a DefEnvironment's end in it is "Missing $ inserted" (§1046-1047), a `}` at its math shift ends it when it closes a construct digested as a unit (a box body, an argument digested on its own — content first by main-session decision: `\mbox{$x} after` keeps "after" as text where pdflatex swallows it into the box) and is deleted otherwise (a plain `{…}` group included, as TeX: `{\bf $x} y$ after.` keeps "after." text), one Error "Extra }, or forgotten $" either way (§1069; 1710.02165 29 → 21); display math stays tolerant of `\par` (no §1047, by decision). REDs dollar_closes_open_script_group, par_in_inline_math_inserts_dollar, stray_brace_then_dollar_keeps_text GREEN. **Open**: (1) a formula in an alignment keeps the old recoveries — a cell's `$` may be its template's (math9805003's undefined `{Sb}` in an `align*` lost the alignment when its row `$` closed the `\begingroup`); an author's cell formula needs the template's `$` told from the author's (RED paragraph-mode/cell_end_in_inline_math_inserts_dollar; 1210.0368 and 1907.09124, whose deferrals sit in alignments, unchanged); (2) display math has no §1047 (a blank line in `equation` is common), so its stray `}` keeps 65a's inserted `$` (f2 `\[ x } y \]`, a13); (3) DONE: boxes-groups/mbox_math_stray_brace_ends_box_not_math GREEN by the unit-construct rule (round 3; round 2's "partner outside" rule lost a1/a2/a16, reverted); its residual, a forgotten `$` in a plain group (`Before {\bf see $x} more text.`) runs "more text." as math to the `\par`, as pdflatex — possible refinement by `$`-parity in the rest of the paragraph (odd: delete; even: end at the `}`) (RED paragraph-mode/forgotten_dollar_in_a_plain_group_keeps_the_text); (4) the `}`-meets-`\left`-group deletion in inline formulas (65d round 7, h1/d2/e1-e4) is unblocked by (§1065 at `$`) and not done (RED paragraph-mode/brace_meeting_left_group_in_inline_math_is_deleted); (5) hep-lat9811036 / nucl-th0401019 run text together from another open construct, not this one | **PARTLY DONE 65n**; (1) next | M | OD #553, #530; `agent_reports/2026-10-12_payload_65n.md`, `_65d.md` rounds 5-7 |
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
    `\EmulatedPackage`; honouring it broke 2605.08402's `\listoffigures*` on the 57bu A/B); Rust's `\@addtoreset` omits
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
  (2009.12523, 2007.02789); lmcs's `\titlecomment` under `\documentclass{LMCS}` (upper-case name, no binding: 1103.0510);
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
  alignment cell (~51), R4/R5 singletons; the parked mode-switch family (29/56) beyond mdpi.
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
#444; a binding route, Perl's prefix alternate or Rust's case/basename steps, still wins): OmniBus-class papers 1,151 →
220 errors and 2 → 89 error-free (127), random run-329 error papers 4,645 → 2,714 and 45 → 155 error-free (383),
general sample 752 → 691 errors (873), no body text lost. With it: extsizes bindings (`\@ptsize` in points), `\documentstyle` binding-first probes
and a raw `.cls` probe, tocbibind conditionals (KPE #482), ragged2e `\LaTeX*` saves under `newcommands` (KPE #481),
IEEEtran `\ifCLASSINFOpdf` from `\pdfoutput` (KPE #480, 22 papers), and four kernel paths raw classes reach: an empty
`\@startsection` type (KPE #483), a re-let `\@startsection` recursing through `\@sect` (`\lx@startsection`), the
`\@maketitle` deposit's diagnostics hold, the locked `\NAT@wrout` (KPE #484). Open, worse than OmniBus (DIVERGENCES
#444, RED repros): a paper's `\let\ifpdf\relax` that graphicx's driver undoes
in TeX (JINST, Fatal), JINST's locked-`\author` flag, raa crossed groups, `\alignauthor`/`\newauthor` vs
`\lx@personname`, `\@ifundefined{figure}` seeing the kernel's environment (sig-alternate).

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
- `\jobname.aux` re-read: lamuphys.sty:1240-1247 and caosp.sty:916 redefine `\enddocument` to `\input \jobname.aux`
  under `\if@filesw` (true in Perl and Rust), a file pdflatex has written by then and LaTeXML never writes:
  `Error:missing_file` (cond-mat9607109, astro-ph9805185; 0.7.6 clean).
- Still Fatal after 62b, each erring under pdflatex too: 1409.3401 (pdflatex emergency stop), 2105.00771 (101 = 101), cs0702042 (citesort.sty missing from TL; Rust
  then 101 math-alignment errors, pdflatex 3).
- Rust-only, still open: CJK GB `\@inpenc@undefined` (1007.1512); singles 1205.5844 (`\@journal`), 1711.06710
  (`\@rticle@options`), 1811.00686, 1907.03566, 2004.12109 (memoir font command), 2105.02164, 2203.13766, 2207.02360,
  2306.10394, 2409.00304, astro-ph9805185 and cond-mat9607109 (`.aux` input).
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
