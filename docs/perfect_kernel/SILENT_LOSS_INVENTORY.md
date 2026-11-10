# Silent-loss inventory — blocks the 98 % claim (user 2026-10-18)

The living worklist of conversions that lose authored content or its markup with 0 errors: a false success. Moved
whole from [`SYNC_STATUS.md`](../SYNC_STATUS.md) on 2026-11-09 (SYNC1b; the section had outgrown that file's ~100-line
rule), in its one-fact-per-line form: one row is one `###` section, one fact per bullet. The user's framing
(2026-10-18): "We should address all of [the High Priority items that were not Error status] before claiming 98 % is
reached".

**Every row is open unless it says otherwise** (a `State:` of closed, accepted or no action). **An open row stays
here whole** (SYNC1c, 2026-11-09): only closed rows moved to [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md),
and an open row's "Full row (verbatim)" pointer names the verbatim copy there, never a part missing here. The same
rules as SYNC_STATUS: verify a status against its named guard before acting on it; writers hand the main session a
SYNC note rather than editing this file.

Source:
`agent_reports/2026-10-18_silent_loss_inventory.md` (27 items, numbered as there); each row is fixed, or at least made
an honest Error, before the 98 % report or the E/F rerun. Measured on all three targets (error-free log, authored text
present, semantic element) — `feedback_three_metrics_honest_tests`.

**Status 2026-10-19 (int31, 9482a8e7db):** every numbered row is fixed (#21 in part: what remains prints extra
entries, it loses none), closed, or an honest Error; the unnumbered
rows left open below lose no authored text (duplication, mis-split, hover-only placement, a wrong reference number) and
do not block the E/F rerun.

**Moved 2026-11-08 (DOC1):** the landed frontmatter rows of this table are in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md), verbatim; a landed row with a live residual left a one-line row here holding only the residual, marked "the residual".

### 5 — raw-class off-table title-page stores (#5) — the residual

- **raw-class off-table title-page stores (#5) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): the vmsta 2018+ record ruling (75a row (e));
- nowfnt's cover author block beside the creators (OD #767, row "nowfnt creators").
- State: OPEN (residual)

### 18 — amsrefs `\bib` keys differing in case merge, one entry lost

- amsrefs `\bib` keys differing in case merge, one entry lost
- State: honest Error int27 (`scan_bibentries`); keeping both open

### ptptex `[preprint]` double abstract

- ptptex `[preprint]` double abstract: the class's own `[preprint]` `\maketitle` runs (ptp-prep.clo:111-119), prints `\@abst` in its block and clears it (:27-34); the frontmatter's `ltx:abstract` comes from the value the setter stored (`lx_store_value_<name>`, OD #154 addendum), and the class's printed block stays in the body — the abstract text twice where pdflatex prints it once (0906.3938, 0907.2968; base int29a the same)
- State: open residual (not new in 77i; pinned by count in `perfect_kernel_batch77i::abstract_store_cleared_by_the_class_title_page_is_kept`, repro `sectioning-frontmatter/abstract_store_cleared_by_the_class_title_page_is_kept.tex`): the store is handed on, but the block the class's running `\maketitle` typesets is not the deposit's, so its label and store are not taken out with it
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "ptptex `[preprint]` double abstract".

### #6a labels

- #6a labels: a label read off the class's code only when it is plain text — one with an accent (`R\'esum\'e : \@resumen`, `\textbf{Mots-cl\'es :} \@keywords`: `label_item` refuses the accent macro) stays an orphan paragraph and the entry takes the default name ("Abstract", "Keywords: "); a heading-style label before an abstract the kernel's `\abstract` owns (`\begin{center}{\bf Abstract}\end{center}\@abstract`, `\textbf{Abstract:} \@abstract`, `{\bf Abstract.} \@abstract`) is not a rerouted store's and stays too; and an unlabelled class abstract store is named "Abstract", which the XSLT renders as a heading (LaTeXML-structure-xhtml.xsl:146-150) pdflatex never prints — the base and Perl convention
- State:
  - open residual (base too; review probes `x/q2`, `x/ab1`-`x/ab3`, `n/d_nolab` of 77i r4): an accented label needs `label_item` to read accents and the label check to compare folded letters; the heading-style shapes need the kernel `\abstract`'s deposit to take the class's heading out.
  - Also (77i r5 re-check, synthetic, sample reach 0): with the deposit dropped, a label before a case-changing argument (`Conference: \MakeLowercase{\@va}`) is lost with its line while the value is kept as a pubnote (probe `rev77i5/n/p2c`; guard `store_values_typeset_with_quotes_are_printed` pins it); a store the title page only reads (`pdfsubject`) stays a visible `name="Abstract"` abstract beside a document abstract of another name/language (`n/p3`, base identical: `frontmatter_release_store` drops only on the same name and language); a document `{abstract}` before and another after `\maketitle` keep only the second (`n/p4`, plain `article` and base the same — the kernel's replacement rule; pdflatex prints both)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "#6a labels".

### nowfnt creators

- nowfnt creators: `\author[n]{Haikin, Marina}` (nowfnt.cls:581, BibTeX name order "Surname, First", split by the class's `\nowfnt@splitauthor`) becomes two creators "Haikin" and "Marina" (8 creators for 4 authors), and `\affil[n]{…}` (:599) reaches no creator — 2108.12618, 2308.02698 (base too);
- the cover's `\maintitleauthorlist` block is dropped when it only repeats the creators' names (2308.02698) and kept as an `ltx:pubnote` with line breaks between the authors when it carries more (2108.12618's affiliations and e-mails; main-session decision, OD #767, 77i r3)
- State: open

### bullsrsl correspondence

- bullsrsl correspondence: the corresponding-author e-mail (`\correspondance`, bullsrsl.cls:475-477, printed after `\BSRSL@CorrespIntro`, a label macro the class defines only inside its `\maketitle`, :747-765 "Correspondence to:") reaches the frontmatter as an `ltx:pubnote` without that label, which renders as a hover-only popup (LaTeXML.css:107-120) — 2401.09121, 2410.19120 (77i r2/r3; base lost it).
- Needs: a label read from a macro the title page defines before printing the store, and routing to the corresponding creator's `ltx:contact role=email` (the creator carrying `[corresponding]`): not a brief patch with the existing store techniques (77i r3)
- State: open

### 21 — biblatex `\printbibliography` filters

- biblatex `\printbibliography` filters: every list printed every entry
- State:
  - PARTLY FIXED 77a (OD #762, KPE #983): `type`/`subtype`/`keyword`/`category` and their `not*` forms narrow the `.bbl` list in the order given, ANDed, a repeated key again (`type`/`subtype` twice is biblatex's error); category membership is decided once the document is digested, so a late `\addtocategory` counts, and a category list's entries are deferred there and built only if kept (77a r4: no entry digested that no list keeps, none built again in the hidden list, which holds only unprinted entries; an entry two lists keep is typeset twice, as in pdflatex); `\bibbycategory` per category under its `\defbibheading` title; each list is its own refsection's, and a refsection's citations link to its own list (`inlist`/`lists` `refsection<n>`; 77a r4, witnesses 2308.00515, 2507.16315); "Empty bibliography" as biblatex; an entry has one label number in every list of its refsection, read from the list's own copy (its datalist place; print order with `defernumbers=true`, `resetnumbers`), a `shorthand` entry its shorthand and no number, on the `.bib` route too (77a r6; biblatex only: under natbib a `.bib` `shorthand`/`labelalpha` field is raw data and no label, 77a r5); biblatex's `\bibliographystyle` is ignored with biblatex's error as a Warning (blx-compat.def:13-17), so a leftover call after `\printbibliography` no longer turns the list into a `.bst`'s and drops its alphabetic labels (77a r6); the entries no list printed are built hidden, so their citations and `\fullcite` resolve (2405.19762); arXiv 2406.00136 21 + 21 → 4 + 17 with pdflatex's labels, 2406.11014.
  - Witnesses: TL `doc/latex/biblatex/examples/13-19,94`, 2406.00136.
  - Reach: 1 of a 6,000-paper arXiv sample; 18 of the 38,734 papers of 2405+2406 use a filter or `\addtocategory`, all with a `.bbl`.
  - **Open (unmodelled)**: a category filled by code the binding never runs (hooks, bibmacros, drivers, formats, cite-command code, a parameter name) is not filtered — the list prints unfiltered, an Info says so; `check=`/`\defbibcheck`, `filter=`/`\defbibfilter`, `segment=`, `sorting=` per list, `omitnumbers`, `prefixnumbers`/`labelprefix`, biblatex's "Split bibliography detected" Warning, and filters on the no-`.bbl` (`.bib`) route; the `.bib` route numbers a `style=alphabetic` list where biber prints alphabetic labels, and ignores the package-level `sorting=` (`sorting=none` lists alphabetically; 77a r6 review probe br/b_none) (both predate 77a).
  - **Open (a)**: refsections opened at headings (`refsection=chapter|section`) are not counted — each `\printbibliography` prints every refsection's list and their citations misroute (arXiv 2406.01695: 8 `\printbibliography` → 96 lists, 4,888 items, 566/566 citations outside their chapter's list; minimal trigger = 77a r4 review probe q3: `report`, `\usepackage[refsection=chapter]{biblatex}`, two `\chapter`s each citing and calling `\printbibliography[heading=subbibliography]` with biber's two-refsection `.bbl` — pdflatex prints each chapter's own list, we print both lists twice); model biblatex's `\blx@refpatch@sect` heading patch (biblatex.sty `\blx@patchsection@switch`, a no-op in the binding) so each heading bumps the refsection and its `CITE_UNIT`.
  - **Open (b)**: the hidden list digests whole entries, so the errors their data raises are raised although pdflatex never typesets them (pinned: `biblatex_filtered_out_entries_errors`, 2 errors, pdflatex 0; review probe e3).
  - Faithful route (77a r4 review): build hidden-list entries label-only (labelnumber, labelalpha/extraalpha/shorthand, labelname, labeldate/extradate, labeltitle) and digest whole entries only for keys a full-entry citation typesets (`\fullcite`, `\footfullcite`, the `\citetitle`/`\citeurl` fields)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "biblatex `\printbibliography` filters".

### an IEEE affiliation block whose refmarked entries a tie run parts is one entry

- an IEEE affiliation block whose refmarked entries a tie run parts is one entry: `\IEEEauthorblockA{\IEEEauthorrefmark{1}Silicon Austria Labs … \\\IEEEauthorrefmark{2}Johannes Kepler University, Linz, Austria~~~\IEEEauthorrefmark{3}Infineon Technologies …}` gives "Johannes Kepler University, Linz, Austria 3Infineon Technologies" one affiliation and the mark-3 authors none (2603.03918; int34a the same), 0 Errors
- State:
  - open (77p r3, review item 7): reach 1 of 84,990 papers of 2603-2605 (3 blocks part refmarked entries by a run; the `\;\;` ones, 2603.12023 and 2604.07032, already split);
  - the run is inside the `\IEEEauthorblockA` argument, which `normalize_hspace_separators` (the top level of `\author`) does not read — the IEEE affiliation reader would take the mark-led rule

### the aero.cls variant whose `\maketitle` prints `\@title`/`\@author` (2509.08607, 2511.17720) keeps its `\cor` glyph in the personname, "Dario Izzo(🖂)", and the paper's …

- the aero.cls variant whose `\maketitle` prints `\@title`/`\@author` (2509.08607, 2511.17720) keeps its `\cor` glyph in the personname, "Dario Izzo(🖂)", and the paper's e-mail and address stay body paragraphs (`\noindent\cor{} dario.izzo@esa.int`), as int34a
- State: open (77p r3, OD #775): the kernel `\author` path is left as it is (review item 1); reading the variant's `\cor` as a request needs the e-mail the paper sets in its body

### `Error:frontmatter:merged_creators` separators (77d, 77n; OD #765) — the residual

- **`Error:frontmatter:merged_creators` separators (77d, 77n; OD #765) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open S6 shapes (class list `\author`s: aipproc 0911.5408, 1411.0869, physics0611155; acmart 2105.00164; quantumarticle 2404.17677; IEEE `\IEEEauthorblockN` with `\IEEEauthorrefmark` 1509.06333; fairmeta 2509.15733), 1706.05664 (pasj01 `\author`), 1911.02873's first two names (a mark and jfm `\ns`, flagged `merged_creators` again).
- State: OPEN (residual)

### `\@@underline` is defined under the dump (record `M \@@underline PA \underline`, latex.ltx:16368's `\let`) but undefined under `LATEXML_NODUMP=1`

- `\@@underline` is defined under the dump (record `M \@@underline PA \underline`, latex.ltx:16368's `\let`) but undefined under `LATEXML_NODUMP=1`: the two `LoadFormat` branches disagree on that name (77g review finding 8)
- State: FIXED (main session): latex_base.rs carries latex.ltx:16368's `\let\@@underline\underline` (KNOWN_PERL_ERRORS #987; guards `dump_gate_init::nodump_latex_branch_converts_healthily`, `perfect_kernel_batch77g::kernel_primitive_standins_mean_their_primitive`); the NODUMP branch still lacks latex.ltx's other saved copies (77j/77l review, meaning probe dump vs `LATEXML_NODUMP=1`): `\let` copies `\@@italiccorr` (:1192), `\@@math@bgroup`/`\@@math@egroup` (:14322-14323), `\@@startpbox`/`\@@endpbox` (:16757-16758), `\@@fileswith@pti@ns` (:18881), `\@@write` (:20427); `\def`s `\@@enc@update` (:10502), `\@@sverb` (:15486), `\@@eqncr` (:15790), `\@@line` (:16403), `\@@unprocessedoptions` (:18923) — open, NODUMP only (the arXiv runs use the dump)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "`\@@underline` is defined under the dump (record `M \@@under".

### merges now flagged by 77d's mark-as-comma re-read (`flag_merged_creators`; XML unchanged, the content was merged silently before)

- merges now flagged by 77d's mark-as-comma re-read (`flag_merged_creators`; XML unchanged, the content was merged silently before): binding nets 67int34 vs 67int29 13 documents 0 → 1-2 `Error:frontmatter:merged_creators` — names merged with each other (2508.17434 aaai2026/listings, 1804.05141 cryptocode, 1706.10121 and 2207.11582 gretsi, 2112.09858 jfm, 2602.02920 listings, 1912.09565 sagej, population 1906.03608 `\And`-less list), a name merged with its affiliation (1102.4258, 2608.09604 egpubl: "V. Zobel INRIA Grenoble Rhône-Alpes") or with a note ("José C. Príncipe Equal Contribution", 2410.14697 neurips_2024)
- State: open — split by shape (egpubl affiliation lines, neurips equal-contribution notes, gretsi/aaai separators)

### author names parted by `\ ` alone with a `\\` line of more names (`\author{Ann Able \ Bob Baker \\ Cat Cole \ Dan Dorn}`, the article `\author` and every per-author …

- author names parted by `\ ` alone with a `\\` line of more names (`\author{Ann Able \ Bob Baker \\ Cat Cole \ Dan Dorn}`, the article `\author` and every per-author split): "Ann Able Bob Baker" is one creator and the second line its affiliation, two authors lost with 0 Errors;
- ecai's 1606.03244 has the shape (ecai keeps such a block whole since 77n r2, one creator holding the five names)
- State: open (77n r1 review finding 1): a `\\` line that reads as names after a line of `\ `-parted names continues them (`names_continue`), and `\ ` between two name-shaped runs separates them

### an author's mark is shown after its ORCID link

- an author's mark is shown after its ORCID link: `\author{A$^{1}$\orcidlink{…} and B$^2$\orcidlink{…}}` (2405.16615, authblk; the article `\author` the same) gives "A[iD]1" where pdflatex prints "A1[iD]"
- State:
  - open (77n r1 review finding 2; RED repro sectioning-frontmatter/author_mark_before_orcid_link): a split name's marks are read as affiliation requests (`\lx@sup@request@affiliation`) and an unanswered one is appended to the personname (`show_unanswered_marks`);
  - keeping its place needs a slot where the request was read (after the math it sits in: `\aftergroup`), filled or removed at insertion, without spending an `xml:id`

### a split name's mark that a line or other text follows

- a split name's mark that a line or other text follows: the same placement on every name the author-line splitter reads — `Ann Able$^{a}$ (ann@x.org)` gives "Ann Able (ann@x.org)ᵃ", and one before a `\\` is dropped (`\author{Ann Able$^{1}$\\ ann@x.org \and Bob Baker$^{2}$}`: "Ann Able", int32a too)
- State: open (77n r3 review probes m2, m7-m10; the same mechanism as the ORCID row above). authblk's single marked `\author` takes the split only when its marks end what it prints (`author_marks_end_name`, 77n r4; guard `perfect_kernel_batch77n::authblk_mark_before_a_line_stays_in_place`); the article `\author` lists still do, and so does an authblk name whose mark holds a comma (`author_call`'s comma test: `\author{Bob Baker$^{1,\dagger}$ (bob@x.org)}` gives "Bob Baker (bob@x.org)†" where pdflatex prints "Bob Baker1,† (bob@x.org)"; int32a moves both marks, 77n r7)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a split name's mark that a line or other text follows".

### a combined legend mark is labelled by its first label only

- a combined legend mark is labelled by its first label only: `\author{Ann Able$^{1}$and Bob Baker$^{1,2}$\thanks{$^{1}$…}\thanks{$^{1,2}$Bob Baker is with Univ Y.}}` (2604.27101, ieeeconf) gives Ann Able Bob Baker's legend too (`\lx@sup@setlabel@affiliation` keeps the first of `clean_frontmatter_labels`; int32a the same)
- State: open (77n r4; authblk's `\affil{$^{1,2}$…}` leaves the marked path for authblk's run instead, `queued_authors_carry_entry_mark`)

### names parted by a separator inside a layout or declaring group are one creator

- names parted by a separator inside a layout or declaring group are one creator: `{\large \bf Yuxuan Yang$^{1,2}$ \quad Dugang Liu$^{1*}$ \quad …}` (2603.20775, flagged `merged_creators`), `{\centering\bfseries A\textsuperscript{…}\quad B…\\ C…\par}{\centering\small $^1$…\par}` (2610.21682: one creator holding every name and affiliation, 0 Errors — silent), `\textbf{Alain Rakotomamonjy\up{2}\hfill Patrick Gallinari\up{1,2}}` (2209.14855, silent), `{David McAllister$^{1}$… ~ Songwei Ge$^{2*}$ ~ …}` (2406.09417: one tie between spaces, no run, flagged)
- State:
  - PARTLY FIXED 84a (OD #897, `space_parted_names` inside a wrapper and a declaring group): 2603.20775 three creators, its Error gone; 2209.14855's `\hfill` parts Rakotomamonjy and Gallinari; 2406.09417's first line four creators (a group holding only marks, `{$^{2}$}`, is the name's marks).
  - Open: 2406.09417's second line, where a brace the author left open makes one group of `{$^{1}$ ~ Aleksander Holynski{$^{1}$} ~ Angjoo Kanazawa{$^{1}$}}` inside `\textbf{Alexei A. Efros…}` (one creator of three names, silent before and after); 2610.21682 (`{\centering\bfseries …\\ …\par}`) not in the local corpus, unchecked; a `\\` inside a wrapper still does not part names (77p; HF14 M2(c)); Perl merges the same.
  - 2406.09417's single ` ~ ` between spaces is no separator (user ruling 2026-10-19, 77p r2: the spacing-run ruling covers runs of two or more), so its four names wait on this row (the group) and stay flagged.
  - Also in a marked block: a second row of names whose later names carry no marks stays one creator with the address after it, `… Guanheng Chen$^{2}$ He Zhu Xun Wu Shaohan Huang Furu Wei \\ Microsoft Research` → "He Zhu Xun Wu Shaohan Huang Furu Wei Microsoft Research" (2607.18110, the r3 re-review; int34a the same)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "names parted by a separator inside a layout or declaring gro".

### a row of addresses parted by space under a row of names parted by space (`\author{Cat Cole \quad Dan Dorn \\ Univ C \quad Univ D}`, each address printed under the name …

- a row of addresses parted by space under a row of names parted by space (`\author{Cat Cole \quad Dan Dorn \\ Univ C \quad Univ D}`, each address printed under the name above it) gives Univ C to the row's last name and makes "Univ D" a creator, 0 Errors (Perl the same)
- State:
  - open (77p r2; pinned by `perfect_kernel_batch77p::space_parted_names_negatives_pin_residuals`): pair the address row's space-parted pieces with the names by position when the counts agree.
  - Since 77p r3 a spacing run on such a row parts nothing (the names-line rule), so `Ann Able~~~Bob Baker \\ Univ A~~~~Univ B` gives "Univ A Univ B" as one affiliation of both names — no invented creator, the pairing still lost; since 77p r4 also when the institutions read as names (`Ann Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins`, `\textbf` names over `\textit` institutions: `row_of_names`, guard `perfect_kernel_batch77p::institution_row_under_names_is_no_names`), and since 77p r5 over an e-mail line the addresses do not spell (guard `institution_row_over_an_email_line_is_no_names`).
  - Still read as names going on, creators "Carnegie Mellon", "Johns Hopkins", …: a row in another count than the names (`Ann Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins~~~~Google DeepMind`, the r4 re-review's probe v4) and a row with a legend line under it (`… \\ \textsuperscript{*}Equal contribution`, probe v2) — no corpus case among 933 papers; the count and the line under the row are all `row_of_names` reads, markup does not tell institutions from names (2607.07318, 2603.16856).
  - The r5 trade (recorded 77p r6): a real second row of names over an e-mail line with fewer addresses than the names of both rows, none spelling a piece of the row, reads as one affiliation of the names above, as in int34a (`Ann Able~~~~Bob Baker \ Cat Cole~~~~Dan Dory \ 	exttt{ann.able@x.edu}`, the r5 re-review's probes n1-n4, n10: Cat Cole and Dan Dory lost as authors) — v3 (institutions) and n1 (names) have the same structure, so no address rule parts them; no case among the 1083 A/B papers (2602.13551 has an affiliation line between)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a row of addresses parted by space under a row of names part".

### names parted by horizontal space under symbol-labelled address lines (77p-77y; OD #52(o)) — the residual

- **names parted by horizontal space under symbol-labelled address lines (77p-77y; OD #52(o)) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a legend's "Corresponding author:" before a bare address is no e-mail label (2609.24299); one `\textsuperscript{\textdagger}` in run-parted rows merges 7 + 5 names with no Error (2601.11514, probe s4); a legend line holding a mark after its text reads as names (2602.09443); a comma list of symbols in a name's mark or leading a line (probes x1, x3, y1-y3, q1-q3).
- Also open (recorded 77p r4-77y, not fixed since): a starred legend in a numbered block goes to the last author (2608.22149's `{\small $^{*}$Equal contribution.}` on Yuchen Cui, 2605.03042's `$^\dagger$Project Leader \quad $^*$Corresponding Author` on Shuai Li); a marked block's legend line `\\ \textsuperscript{*}\textit{Equal contribution.}` lands as an affiliation of the last author (2509.20748); a line of several tie-joined entries is paired by its last mark (2609.24797's `$^{\triangle}$EPFL,~$^{\bigcirc}$OpenEuroLLM,` on Jenia Jitsev; probe f1); a second row of marked names styled otherwise than the first, after a names line ending in a comma, reads as an affiliation (2601.18077: two authors lost; 2609.35807 the same shape); a marked address line in a numbered block doubles its mark inside the address (probe f4); the numbered block's x12 weld welds a note line too (probes x1-x3); a mark-only piece before a `\quad`-parted link is a creator "‡" (probe h3).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "names parted by horizontal space under symbol-labelled addre".

### a linked two-word text whose last word is a link word, alone in its `\and` column, reads as a name

- a linked two-word text whose last word is a link word, alone in its `\and` column, reads as a name: `\author{Ann Able \\ Univ A \and Bob Baker \\ Univ B \and \href{…}{Source Code}}` makes a creator "Source Code" (no corpus witness, probes a5, a6 — "Online Demo" too; and a three-word one whose last word is a link word that is also a surname, "Full Source Code", "Google Scholar Page", "Hugging Face Link", the link column first or last: probes s1, s3, s4, s7 of the rev77y3 re-review's finding 5 — or its plural, "Other Useful Links", the rev77y4 re-review's probe s20 — no author block linking such a text among 519,672 papers of 2025-2026, int38b the same, 77y r2 URL contacts; 2606.06924 prints "Datasets  Source Code" on one line under its e-mail line, both URL contacts of the shared creator as its PDF shows, the rev77y2 re-review's finding 6; int38b the same), 0 Errors
- State: open (77y r2, the 77y review's finding 1): `link_words_beyond_a_surname` keeps a two-word text a name for "Lucy Page" alone in its column (`perfect_kernel_batch77p`'s linked-names guard) and cannot tell it from "Source Code" without a word list; options: a column holding a linked text alone is the block's link unless the text spells the link's target (`x.org/lp` for Lucy Page), or accept.
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a linked two-word text whose last word is a link word, alone".

### a name alone under a row's merged symbol lines over a one-word generic institution noun or a connective-led institution of its own is lost

- a name alone under a row's merged symbol lines over a one-word generic institution noun or a connective-led institution of its own is lost: `Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics` gives "Cat Cole" to the shared creator as an affiliation and "Physics" to Ann Able and Bob Baker (pdflatex prints it under Cat Cole only); also `\\ Research`, `\\ Polytechnique`, `\\ at Univ C`, `\\ University`, `\\ Institute` (the rev77y4 re-review's finding 1, probes c1, c3, c4, c8, k27, k30; int38b and 77y r3/r4 the same, 77y r2 right), no corpus witness, 0 Errors
- State: open (77y, accepted cost of `continues_the_line`, which reads `Shanghai Jiao Tong \\ University` as one institution, probe k12; the hyphenated `Max-Planck-Institut` FIXED in int41, guard `perfect_kernel_batch77y::names_split_under_symbol_lines_reads_the_line_under`).
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a name alone under a row's merged symbol lines over a one-wo".

### in a per-author `\author` that adds to its block, a link column (`\href{…}{Project Page} \\ Univ B \\ contact@x.org`) goes to the current author

- in a per-author `\author` that adds to its block, a link column (`\href{…}{Project Page} \\ Univ B \\ contact@x.org`) goes to the current author: a raw class appending to `\author` puts the URL, Univ B and the e-mail on Ann Able, where pdflatex prints the column beside her (the rev77y4 re-review's finding 3, probes x7, x8; int38b and 77y r2 a creator "Univ B"), no corpus witness, 0 Errors
- State: open (77y r4): `LinkOwner::Current`/`\lx@add@email` for `block_links`/`block_lines` when `replace` is false; option: the block's (`LinkOwner::Block`, `AddressPlacement::Shared`) once a per-author block forms a shared creator.

### an organization link whose name is exactly an author's spelling reads as that author's handle

- an organization link whose name is exactly an author's spelling reads as that author's handle: `Hua Wei \\ \href{https://www.huawei.com}{Huawei}` and `Hua Wu \\ {HW}` give a URL contact on the name, not an affiliation (the rev77y4 re-review's finding 4, probes h20, h21; int38b and 77y r2 affiliations), the link still on the name it prints under, 0 Errors
- State: open (77y r4, `link_prints_a_handle`'s exact-spelling test; no corpus witness).

### a name alone under a row's merged symbol lines with a place under it is lost

- a name alone under a row's merged symbol lines with a place under it is lost: `Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Palo Alto, CA` gives "Cat Cole" and "Palo Alto, CA" to the shared creator (the rev77y2 re-review's probe k10; int38b and 77y r2 the same; 77y r1 an author), 0 Errors
- State: open (77y r3, the rev77y2 re-review's finding 4; no corpus witness): the names split (`names_at`) accepts a line of names over an institution (not one its words go on into, `continues_the_line`) or over an e-mail address it spells (`addresses_spelling`, probe k15), not over a place — `Google DeepMind \\ London, UK` (probe k11) has the same shape and is no author; options: a place under a line of a given name and a surname only, or accept.
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a name alone under a row's merged symbol lines with a place ".

### a place line with no comma under a welded symbol row's labelled institution goes to the shared creator

- a place line with no comma under a welded symbol row's labelled institution goes to the shared creator: `$^\dagger$Department of Physics \\ University A \\ Palo Alto \\ $^\ddagger$School of Law …` gives "Palo Alto" to the block, not to the † name (the rev77y2 re-review's probe w9; 77y r1 Ann Able's, int38b the block's), 0 Errors
- State: accepted (77y r3, the rev77y2 re-review's finding 7): `reads_as_place` needs a comma (`Muncie, IN 47306`), so a bare place is no `weldable_line`, and an unreadable owner goes to the block (the accepted fallback).

### a brace-list address group and a later address under three names swap two owners

- a brace-list address group and a later address under three names swap two owners: 2606.06924's `\{laign,yehj\}@…, huhr@…` gives `yehj@` (Han-Jia Ye's) to Haoran Hu and `huhr@` (Haoran Hu's) to Han-Jia Ye (int38b and 77y r1-r3 the same), 0 Errors
- State: open (pre-existing; the rev77y2 re-review's finding 8): `spelled_owners` scores "yehj" and "huhr" against the three names' keys — not yet root-caused; the swap puts each address on a name the PDF does not print it under.

### in an unmarked block a row of names after an affiliation line stays an affiliation line of the names above

- in an unmarked block a row of names after an affiliation line stays an affiliation line of the names above: `Ann Able \\ Univ A \\ Bob Baker~~~~Cat Cole \\ Univ B` gives Ann Able the affiliations "Univ A", "Bob Baker Cat Cole", "Univ B" (two authors lost; 0 Errors; int34a and Perl the same, the r3 re-review's probes p19, p27, where r3 had made Cat Cole alone a creator)
- State:
  - open, narrowed (85a r2, OD #900): a row that a gap (`\\ \\`, `\vspace{…}\\`, a positive `\\[len]`), the names' styling or the alternation opens after a name's lines is a names line, parted into its creators, each with the lines under it (`perfect_kernel_batch85a::row_of_names_opens_the_next_author`); left: the row with none of these, as above (RED `author_block_row_of_names_with_no_signal`; no person's cue, no gap, no styling — the same reading as a cue-less `Bob Baker \\ Univ B`; 77p r4: `row_of_names` joins only the line right after the names line; pinned by `perfect_kernel_batch77p::institution_row_under_names_is_no_names`).
  - Also a row of names under a single name (`\textbf{Ann Able}\\[2pt] \textbf{Bob Baker}~~~\textbf{Cat Cole}`, the r4 re-review's probes t1/t2: Bob Baker and Cat Cole an affiliation of Ann Able; `row_of_names` needs two names on the names line; int34a the same)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "in an unmarked block a row of names after an affiliation lin".

### an author mark with a footnote symbol before the number, `$^{*1}$`, requests no affiliation

- an author mark with a footnote symbol before the number, `$^{*1}$`, requests no affiliation: `Ann Able$^{*1}$, Bob Baker$^{1}$ \\ $^1$Univ A` gives Ann Able none (the 77p r4 re-review's probe k1); 2609.16161's `Sebastian Zhao\blfootnote{…}$^{*1}$ \enspace\enspace Minseo Kim$^{*1}$ …` leaves its three starred authors without UC Berkeley, visible on three creators since 77p split the names (int34a the same, merged), 0 Errors
- State: open (77p r5, recorded): `split_glued_symbol` (`rewrite_symbol_superscripts`) splits a number before the symbol (`$^{1*}$`), not one after it; the same paper's `{$^{1}$~UC Berkeley\qquad $^{2}$~ICSI\qquad $^{3}$~LBNL}` is one entry (the layout-group row); and a symbol glued to the last number of a list, `$^{1,2*}$`, is no part of the split either: its labels are 1 and "2*", so 2605.03042's Shuai Li misses Shanghai Innovation Institute, which goes to the shared creator, and shows no star (the r8 re-review's A/B, recorded in 77y)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "an author mark with a footnote symbol before the number, `$^".

### a horizontal skip that opens a line of an unmarked block parts it as a separator

- a horizontal skip that opens a line of an unmarked block parts it as a separator: `The University of Hong Kong\\ \hspace{-12pt}\texttt{\small \{bhzhang, liujh, xjqi, hso\}@eee.hku.hk}` makes the e-mail line a names line, its pieces creators "{bhzhang", "liujh", "xjqi", "hso}@eee.hku.hk" (2603.27558; int34a the same), 0 Errors
- State:
  - open (77p r5 A/B, recorded): OD #52(i)'s `\hspace` → `\quad` rewrite reads a skip at a line's start as a group separator;
  - a negative or line-opening skip separates nothing, but a positive one can stand between author groups (`Ann Able \\ Univ A \\ \hspace{2cm} Bob Baker \\ Univ B`), so the rule needs care

### a mark wrapped in a user macro is not read as a mark

- a mark wrapped in a user macro is not read as a mark: `\newcommand*{\affmark}[1][*]{\textsuperscript{#1}}` then `Thanh-Tung Nguyen\affmark[1]~~ Viktor Schlegel\affmark[1,3]~~ …\\ \affaddr{\affmark[1]AICS Singapore / TaiPei}` (2304.13998): the block reads as unmarked, "3]" and "2]" junk creators (the comma in the optional argument), the affiliations not by the marks, 0 Errors;
- 2602.23952's `\affmark[1,2,3]` the same, invented creators "2", "3]", "2]" (int34a too; 77p r5 recovers about six of its authors; the r5 re-review)
- State: open (77p r1 question 3; main-session decision 2026-10-19: a later follow-up): a macro whose expansion is exactly `\textsuperscript{#1}` (its optional default the mark too) reads as a mark, as a `\let` alias does (`mark_command_meant`, OD #52(o))

### \footnotemark[1] after an author name in an ICLR block prints "1 footnote 1 footnote Footnote footnotes Footnotes 1 footnote 1" inside the personname (\textbf{Sungil …

- `\footnotemark[1]` after an author name in an ICLR block prints "1 footnote 1 footnote Footnote footnotes Footnotes 1 footnote 1" inside the personname (`\textbf{Sungil Son}\textsuperscript{1}\footnotemark[1]`, 2610.20409; `\textbf{Ying-Tu Chen\textsuperscript{\normalfont{1},}\footnotemark[2]}`, 2604.24532; 2603.12193, 2509.10377, 2403.00801; int33a the same, the names merged then)
- State: open (77p A/B)

### an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o), user ruling R10) — the residual

- **an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o), user ruling R10) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): an icon-only link (2604.10930); 2607.22153's `eeyxm@` the block's; a "Contact:" label is no e-mail label (2606.28166); a split name in a third names row (2602.23649); 2605.08769's `xuchd6@` the block's (R10); an address printed twice is two contacts (2604.10667, a ruling if one should go).
- The r6 trade, recorded: an address the r5 rules placed by the names left over stays on the last name when another line holds an unspelled address (2605.30274's `zhangmin186@`).
- Also: the trade's second witness 2609.02349's `xiahou@` (on Junping Zhang; accepted residuals, user ruling 2026-10-20); the order path gives no address to an author another address spells when the block lists one per author at most (2609.34459's `mayhhu@`); a star after a line break and a space stays text, a legend's mark (2608.23637's `\\ *$\S$$\P$ …`).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o)".

### an unmarked author block's brace-list emails (`\{nlp2ct.guanhua, nlp2ct.chuyue, …, xqsongangie\}@gmail.com\\ \{derekfw, lidiasc\}@um.edu.mo`, 2605.15019) go to the last …

- an unmarked author block's brace-list emails (`\{nlp2ct.guanhua, nlp2ct.chuyue, …, xqsongangie\}@gmail.com\\ \{derekfw, lidiasc\}@um.edu.mo`, 2605.15019) go to the last author, but for an address that spells a name
- State: open, an accepted trade-off (77n r5 brief; r3: `NLP\textsuperscript{2}CT Lab` is the word's since `rewrite_in_word_superscripts`, so the block is no longer a marked one — its four junk creators gone and the affiliation on every author, the marked branch's per-position emails lost)

### authblk `\affil{1}` inside `\author` (`\author{Khalid Mahmud Labib\affil{1}, Saad Waheed\affil{1}, …}` then `\affil{Dynamical Systems and Signals Lab (DSSL)\\ …}`, …

- authblk `\affil{1}` inside `\author` (`\author{Khalid Mahmud Labib\affil{1}, Saad Waheed\affil{1}, …}` then `\affil{Dynamical Systems and Signals Lab (DSSL)\\ …}`, 2605.01689): pdflatex prints no "1" and the DSSL block under every author;
- we give each author a "1" contact and the DSSL block to the last author only (int32a's and Perl's `annotate=new`, authblk.sty.ltxml:57)
- State: open (77n r4; guard `perfect_kernel_batch77n::authblk_affil_after_affils_in_author_reaches_them` pins it): read an `\affil` inside `\author` that prints nothing as no affiliation, so the run reaches every author — a further departure from Perl's `annotate=new`, a ruling first

### a capitalised "And"/"AND" between two names is no separator

- a capitalised "And"/"AND" between two names is no separator: `\author{Ann Able And Bob Baker}` is one creator with 0 Errors — the merge check reads "And" as a name word (int32a and Perl the same; Perl's only word separator is " and ", Base_Utility.pool.ltxml:678);
- glued after a mark, `Ann Able$^{1}$And Bob Baker$^{2}$` (77n r4/r5 review probes a1, a6), it is one creator flagged `Error:frontmatter:merged_creators`
- State: open (77n r6 keeps the flag, fail toward flagging — r5 read the "And" as a separator after the mark and merged the line silently; guard `perfect_kernel_batch77n::mark_glued_to_capital_and_is_flagged`)

### a run always reaches its last author (Perl's `new`, Base_Utility.pool.ltxml:530), answered or not — the label-answered break applies only once the run has reached an …

- a run always reaches its last author (Perl's `new`, Base_Utility.pool.ltxml:530), answered or not — the label-answered break applies only once the run has reached an unanswered author (77n r6, so a run answered throughout is the whole run's, as ptephy prints it): `\author{Craig M$^{\mathrm{c}}$Rae}\author{Ann Able$^{1}$}\affil{$^1$Univ X}\affil{Univ Y}` (2505.09041's shape) gives the answered last author Ann Able Univ Y as well as Univ X
- State:
  - open (77n r5; guard `perfect_kernel_batch77n::authblk_superscript_inside_a_name_stays` pins it).
  - The first half of this row closed in 77n r6: an authblk name kept on `\lx@add@creator` (`Ann Able$^{1}$\textsuperscript{equal contrib.}`) requests the affiliations its marks pick (`author_label_marks`), so the run passes it over (guard `authblk_textual_superscript_after_mark_is_shown`)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a run always reaches its last author (Perl's `new`, Base_Uti".

### authblk's note numbering (`\AB@thenote`, authblk.sty:114-167) is not modelled

- authblk's note numbering (`\AB@thenote`, authblk.sty:114-167) is not modelled: an unlabelled `\author` after a labelled one shares its note (`\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}`: pdflatex "Ann1 and Bob1 / 1Univ X / 1Univ Y", Univ X under Bob too; we give Bob Univ Y only), and an `\affil` inside a later `\author` steps it (`\author[1]{Ann}\affil[1]{Univ X}\author{Bob \affil{Univ Y}}`: pdflatex "Ann1 Bob2 / 1Univ X / 2Univ Y"; we give Ann Univ Y as well, 77n r4 review probe e3); a marked author before an `\author[1]` shares its note too (`\author{Ann$^{1}$}\author[1]{Bob}\affil{$^1$Univ X}\affil{Univ Y}`: pdflatex "Ann11 and Bob1 / 11Univ X / 1Univ Y"; we give Ann Univ X only, 77n r6 review probe f2 — Bob's Univ X fixed in 77n r7)
- State: open (77n r6; int32a and Perl's `annotate=new`, authblk.sty.ltxml:57, the same; guard `perfect_kernel_batch77n::authblk_labelled_author_shares_the_run` pins it): number the notes as authblk does — an `\author` steps `\AB@thenote` after an `\affil`, an unlabelled one takes it — and label each unlabelled `\author`/`\affil` by it
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "authblk's note numbering (`\AB@thenote`, authblk.sty:114-167".

### a symbol legend after a run reaches authors who do not show its symbol

- a symbol legend after a run reaches authors who do not show its symbol: `\author{Ann$^{1}$}\author{Bob}\author{Cat$^{\dagger}$}\affil{$^1$X}\affil{Y}\affil{W}\affil{$^\dagger$Corresponding author}` — pdflatex shows † on Cat only, int32a and Perl's `annotate=new` give the legend to Cat only, 77n r6 to Bob and Cat, r7 to Ann, Bob and Cat (77n r7 review probe x7; s1, z1, g05 since r6, the ∗ legend on Ann; guard `perfect_kernel_batch77n::authblk_symbol_mark_then_text_shown_once` pins it).
- No paper in 823 A/B conversions or the 2603-2605 scans
- State: open (77n r7; reach 0): when an author of the run shows the legend's symbol, the legend is those authors' only (`base_utilities/defs/frontmatter_api.rs` legend read ~:596, extent ~:719)

### authblk

- authblk: an affiliation a mark picks (`\affil{$^1$X}`) is listed on its authors after the run's unlabelled ones (`\author{Ann$^1$ and Bob$^2$}\affil{$^1$X}\affil{$^3$Z}` gives Ann Z, X): the marked contact is placed at the end, when the labels are matched (`relocate_annotations`), the run's at once
- State: open (77n r3; guard `perfect_kernel_batch77n::authblk_typo_mark_affil_runs` pins the order): place a label-matched contact at its queue position among its author's contacts

### author names parted by a mark and spacing alone, with no separator (pre-77d, synthetic)

- author names parted by a mark and spacing alone, with no separator (pre-77d, synthetic): `Ann Able$^{1,}$ Bob Baker$^{2}$` is one creator "Ann Able Bob Baker", both marks after it, with 0 Errors (1911.02873's shape, there flagged as its marks stay between the names);
- `Ann Able\hspace{1em}and\hspace{1em}Bob Baker` makes "and" an affiliation of Ann (the `\hspace` read as an author/affiliation line break before the name split);
- a pair merged with only glued text to show it ("Ann Ableand Bob Baker", a shape no rule splits) is not flagged — "Ableand Bob" reads as "Ferdinand Smith"
- State: open

### HF14 conference-template frontmatter (77o; OD #772) — the residual

- **HF14 conference-template frontmatter (77o; OD #772) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open (HTML_FEEDBACK HF14): M2 spacing-run separators / `\let`-alias marks, M4 authblk replacing authors, aero.cls, 2609.12818.
- State: OPEN (residual)

### aistats paired column

- aistats paired column: an address line starting "and" is read as an author separator by the kernel `\author` line reading, so 2402.12508's `Department of Mathematics \\ and Computer Science \\ University of Basel` makes a creator "Computer Science" (both rows since 77o r3; up to 32 paired papers carry such a line; 2310.02702 reads right); the same kernel line reading turns a marked line under a paired column into a creator (`Apple$^\ddag$`) and glues a name to its first line when the next column's line is marked — 2202.05722 ("Apple ‡" a creator, two names flagged `merged_creators`), 2401.02609, 2503.08203 (77o r3 review probe m_trail)
- State: open (77o r2 review finding 6, r3 finding 1). Fix when taken: the binding knows line 1 of each column is the name — add the name, then each address line as an affiliation or e-mail explicitly, instead of the kernel's guess
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "aistats paired column".

### aistats in-place reading and e-mail splitting, small (77o r3 review nits 2-4; corpus reach 0 for each)

- aistats in-place reading and e-mail splitting, small (77o r3 review nits 2-4; corpus reach 0 for each): a column holding a test primitive (`\ifx\Uni\relax …`) is not read in place, so a macro defined in the block is undefined (probe c_ifx; r2 printed it by luck);
- a mark glued to an address in a block line (`ann@a.org$^*$, bob@b.org`) stays in the e-mail contact ("ann@a.org*"; probe e_mark);
- a `\gdef` after the block wins over the one before the row (probe c_gdef; deferred frontmatter digestion, r2 the same)
- State: open: protect only a test primitive's operands; reject an `email_addresses` item with a skipped superscript inside it; digest the row in place (FM S5)

### aistats

- aistats: a marked first row followed by an unmarked row leaks the second row's address upward (review probe `r_secondrow_marked`: Ann gets "Univ C" before "Univ A"; her marked affiliation is attached later, so `annotate=run` still counts her new)
- State: open, 0 of 2,843 aistats papers (77o r2 review finding 7)

### aistats `in_place` residuals

- aistats `in_place` residuals: a macro with parameters defined inside the `\twocolumn[…]` block (`\def\Uni#1{…}`, probe `g_args`: Error undefined, pdflatex "Univ of Here") or a local `\let` of a primitive (`\let\Uni\relax`, probe `g_letprim`: Error) is not read in place
- State: open (77o r2 review finding 4); no corpus witness

### aip-cp `\author[aff1]`/`\affil[aff1]` marks are not linked

- aip-cp `\author[aff1]`/`\affil[aff1]` marks are not linked: the authors are creators without affiliations and the `\affil` list reaches the page as the class title page's deposit paragraph (1506.04050, 1604.08112; int37a the same)
- State: open: needs a mechanism for a class's mark-keyed author/affiliation lists (`\g@addto@macro` accumulators, not K11 stores)

### natbib forks that declare their own names stay read raw (`hbnatbib` 16, `ctextemp_natbib` 13, `NAR-natbib` 9, `natbib_padeu` 8, `rnatbib` 7, …

- natbib forks that declare their own names stay read raw (`hbnatbib` 16, `ctextemp_natbib` 13, `NAR-natbib` 9, `natbib_padeu` 8, `rnatbib` 7, `wsnatbib`/`igsnatbib`/`dmnatbib` 6 each, …) and so do the 399 shipped natbib copies that declare nothing: their citations still print "?" unless the fork has a binding (NJDnatbib, bmvc2k_natbib, ws-rv-har)
- State: open (77w residual; census `~/data/pk_agents/main/scratch_356a0c47/natbib_copies/natbib_copies_all.tsv`): a route by content (rule 0 (b)'s command-set test) is the candidate, a user ruling first

### author-block gaps that renamed copies reach (int40) — the residual

- **author-block gaps that renamed copies reach (int40) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): aamas and spie FIXED;
- egpubl (2312.16264's arxiv.cls) and bytedance_seed open: binding work in each family.
- State: OPEN (residual)

### caption separator vs a copy's own label

- caption separator vs a copy's own label: the separator is the binding's model of the class's `\@makecaption` (`\format@title@<type>`'s close), so a routed copy's (or a raw class's) `\fnum@<type>` that carries its class's punctuation meets the binding's (or the kernel's ": "): "Fig. 1.—.— " (1106.2565 emulateapj-rtx4.cls:911 + the emulateapj binding's ".— ", astro-ph0204283), "Figure 1..— " (1505.06804), "Table 1:: " (astro-ph0601469, astro-ph0603093), "Figure 1.: " (astro-ph0101121), "Fig. 1.—: " (1402.7103, PDF "Fig. 1.— " from proto.cls:1226 `#1\ #2`) — 7 of the 78h A/B's 322 papers, worse than int47a's binding label
- State:
  - RULED 2026-10-21: the separator is the binding's (default ": " as Perl), and a copy's caption format (`\fnum@<type>`) is the binding's too — only its declared text names (`\figurename`) reach the label (OD #731 rule 14); no raw `\@makecaption` is read.
  - The 7 doubled separators of 78h r1 are gone (78h r2) — closed
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "caption separator vs a copy's own label".

### a stand-in's copy under the binding's own name (rule 0: a shipped svjour3.cls, aas_macros.sty) keeps the binding's text names (OD #731 rule 1); the 2026-10-21 ruling's …

- a stand-in's copy under the binding's own name (rule 0: a shipped svjour3.cls, aas_macros.sty) keeps the binding's text names (OD #731 rule 1);
- the 2026-10-21 ruling's yielding is the route's (rule 14), though TeX prints the copy's text there too
- State: RULED 2026-10-21: a copy under the binding's own name overrides nothing (rule 1 unchanged; guard `perfect_kernel_batch78h::self_named_copy_supplies_no_text_names`) — closed

### the aas_macros binding's journal names lag AASTeX 7's

- the aas_macros binding's journal names lag AASTeX 7's: it prints "ApJ" for `\apjl`, "J. Quant.
- Spec.
- Radiat.
- Transf." for `\jqsrt`, "Acta Astron." for `\actaa` (aas_macros_sty.rs:25/70/16), where TL aastex701.cls:8314/8352/8359 prints "ApJL", "JQSRT", "AcA" — a class-family binding gap (one binding serves every version: union of features);
- routed AASTeX 6.1+ copies print the class's names since 78h (2107.03400, 2007.00749), an AASTeX document under the binding does not
- State: open (found 78h r2 review)

### the aa binding's itemize labels are article's "•", where aa.cls:2054-2056 sets a bold "–" at the first two levels and `$\bullet$` at the third (2503.15310's aaEC.cls and …

- the aa binding's itemize labels are article's "•", where aa.cls:2054-2056 sets a bold "–" at the first two levels and `$\bullet$` at the third (2503.15310's aaEC.cls and 2505.14783's copies printed it under 78h r1) — the llncs gap 78a closed (KNOWN_PERL_ERRORS #1017)
- State: RULED 2026-10-21: typography, the binding's default stays (list labels are the binding's, OD #731 rule 14) — closed

### the svjour3 binding numbers the class's predefined theorems within sections ("Theorem 5.1") without the `envcountsect` option, where svjour3.cls:902-911 numbers them …

- the svjour3 binding numbers the class's predefined theorems within sections ("Theorem 5.1") without the `envcountsect` option, where svjour3.cls:902-911 numbers them consecutively unless it is given (1803.06644's PDF "Theorem 1"; its routed svjour3-arxiv.cls copy's `\@thmcountersep` stays the binding's since 78h r2, OD #731 rule 14)
- State: open (78h r2, the rev78h review's finding 1): check sv_support_sty.rs's predefined theorems (Perl sv.cls.ltxml L189-223) against svjour3.cls's `envcountsect`/`envcountsame` branches

### pazh_engl.cls:467's title-page journal line ("Astronomy Letters, 2004, accepted") lost under the aa binding's locked `\maketitle` (astro-ph0402416, routed by the copy's …

- pazh_engl.cls:467's title-page journal line ("Astronomy Letters, 2004, accepted") lost under the aa binding's locked `\maketitle` (astro-ph0402416, routed by the copy's `\ProvidesClass{aa}`)
- State: open, minor: a publication note

### a jinstpub/jheppub copy whose own `\maketitle` adds lines loses them

- a jinstpub/jheppub copy whose own `\maketitle` adds lines loses them: 2103.11983's copy prints "Published in JINST as DOI: 10.1088/1748-0221/16/07/P07016" from its edited title page;
- the binding owns `\maketitle` (jheppub's the same exposure; of 121 jinstpub copies the rev78e review read, 4 edit what the title page prints: 3 the header line, kept since 78e r2 as a preprint pubnote, OD #801, and this one)
- State: open, minor (78e r2 residual, the rev78e review's finding 3): a copy-aware rule would read the copy's `\maketitle` for lines the stock one lacks

### jinstpub's (and jheppub's, as Perl's) `\collaboration` and `\proceeding` are pubnotes, which LaTeXML.css shows only on hover (LaTeXML.css:107-119)

- jinstpub's (and jheppub's, as Perl's) `\collaboration` and `\proceeding` are pubnotes, which LaTeXML.css shows only on hover (LaTeXML.css:107-119): 2311.16063's "on behalf of the DUNE collaboration" and 1610.01994's Pixel2016 conference line were visible lines read raw (the rev78e review's finding 10)
- State: open, presentation (CSS; review with the ar5iv-css port at the goal's end)

### `\thanks{\label{fn:c}\email{x}}` (aa probe)

- `\thanks{\label{fn:c}\email{x}}` (aa probe): the `\label` relabels the note, and the e-mail lands on a creator without a name instead of its author (as int45b; rev78e2)
- State:
  - **STOPPED 78f, not brief and no target**: pdflatex (TL 2025) errs on the shape itself — `\thanks{\label{…}…}` in an `\author` under aa.cls (41 Errors), article (101) and llncs: "Use of \@xfootnotemark doesn't match its definition", no PDF — and 0 of the 26,323 sel78d census papers write `\thanks{\label`;
  - left as is

### iopart `\ead` (78f; OD #806, #808, #809, KPE #1031) — the residual

- **iopart `\ead` (78f; OD #806, #808, #809, KPE #1031) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): two `\ead`s inside an `\address` unchanged;
- `\AA rrestad` (2311.12551's own `\AA`, read by the mark reader) open, recorded.
- State: OPEN (residual)

### tlp

- tlp: a `\documentclass{tlp}` (and a renamed copy routed to it, new_tlp.cls: 1405.1833, 1405.2564, 2008.02015, 2008.04008, 2008.06692) reads the author block's affiliation lines after `\\` as more names — 1405.1833 4 → 6 creators, "KU Leuven E-Mail: …" and "transnational University of Limburg E-Mail: …";
- the probe also raises a false "Can't find binding or file for 'tlp.cls'" Warning, OmniBus loaded before tlp_cls.rs (rev77w probe_tlp)
- State: open (77w review finding 2): the raw tlp.cls `\author` lines go through the kernel author splitter; not a few lines on tlp_cls.rs

### 2209.10785: cidr.cls (an acmart copy) comments out acmart's `natbib=true` default; the acmart binding's natbib meets the document's `\usepackage[numbers]{natbib}`, …

- 2209.10785: cidr.cls (an acmart copy) comments out acmart's `natbib=true` default;
- the acmart binding's natbib meets the document's `\usepackage[numbers]{natbib}`, LaTeX's "Option clash" Error where pdflatex loads natbib once
- State: open, stays an Error (main-session decision 2026-10-20; 77w residual)

### adassconf.sty (astro-ph0101121) declares nothing of its own; its first declaration is html.sty's embedded `\ProvidesPackage{html}` (:1374, after 391 definitions), so it …

- adassconf.sty (astro-ph0101121) declares nothing of its own;
- its first declaration is html.sty's embedded `\ProvidesPackage{html}` (:1374, after 391 definitions), so it routes to the html binding;
- converts cleanly (+17 words)
- State: open, left as is (main-session decision 2026-10-20; review finding 7)

### 1802.07182 (aistats)

- 1802.07182 (aistats): the affiliations written as a second `\aistatsauthor` row become creators "University of Cambridge1", "Invenia Labs2", "Cambridge", "UK" (the author's misuse of the setter; its e-mail list is the block's, one contact an address since 77o r3)
- State: open, author misuse (77o r2 review finding 3)

### 1202.1568 (aistats2eM)

- 1202.1568 (aistats2eM): an e-mail list with the domain after the braces (`{seungyeon.kim@, fli@cc., …}gatech.edu`) is not read as an e-mail list and stays inside the affiliation (content kept)
- State: open (77o r2 report)

### 2203.06997 (aistats, paired)

- 2203.06997 (aistats, paired): a `~\\` spacer line before a column's e-mail line makes the kernel `\author` line reading take the e-mail for an affiliation ("Richard E. Turner: University of Cambridge; ret26@cam.ac.uk" both affiliations)
- State: open (77o r3 A/B; content kept)

### `\hyperref[s1]{}` with empty text

- `\hyperref[s1]{}` with empty text: a class deposit holding only it is kept (a `<ref labelref>`, which post fills with "1");
- pdflatex prints nothing.
- Perl does the same in body text
- State: open (77o r2 review finding 10)

### `\shortstack` eats the space after its argument (`A \shortstack[r]{x} Body.` → `…</inline-block>Body.`)

- `\shortstack` eats the space after its argument (`A \shortstack[r]{x} Body.` → `…</inline-block>Body.`): the trailing `OptionalMatch:* [Dimension]` of the prototype reads past it;
- Perl the same
- State: open, KNOWN_PERL_ERRORS #996

### a `{titlepage}` whose class `\@maketitle` prints a paragraph after the title and author

- a `{titlepage}` whose class `\@maketitle` prints a paragraph after the title and author: the paragraph is re-homed before the title (`<para>…Printed cover text…</para><title>T</title>`), the PDF prints it after them;
- base, 77o r1-r3 alike
- State: open (77o r2 review finding 9; guard `perfect_kernel_batch56::unwound_titlepage_rehomes_a_printed_paragraph` asserts the paragraph alone)

### K11 counts a page style the class defines as printing its stores though the class never selects it (`ClassReaders::printed`)

- K11 counts a page style the class defines as printing its stores though the class never selects it (`ClassReaders::printed`): zzfmb.cls's `\def\ps@jhead{\def\@oddhead{\hfil\@journal}…}`, never `\pagestyle{jhead}`, makes the class's `\@journal` default ("Journal of Default Placeholders") a `<pubnote role="journal">` pdflatex never prints — invented visible content (int68a the same).
- Direction: a page style prints only when `\pagestyle`/`\thispagestyle` select it (the class's, its `\maketitle`'s, or the document's) or it is the one the class starts with.
- Repro class-bindings/title_page_typesetting/address_in_a_vbox_with_a_class_line (zzfmb.cls).
- Review rev79a finding 4
- State: open

### K11's line rule (OD #662) after 79a r2 — a store printed in one line with another value of the class's, or glued to its text, stays the class's

- K11's line rule (OD #662) after 79a r2 — a store printed in one line with another value of the class's, or glued to its text, stays the class's: (a) a store kept the class's when the class is loaded (another value in its line printing text then) is not judged again at `\maketitle`, so a document that empties that value still leaves the store in the class's line; (b) a line whose other value the document leaves empty hands the store on and leaves the class's text around it in the body — zzfmf.cls's `{\itshape \@issueinfo; DOI \@doi\par}` without `\issueinfo` leaves "; DOI" (the control of `perfect_kernel_batch79a::a_store_in_one_line_with_a_document_value_stays_the_class_line`; int68a the same): the label before a store is taken out with it only for an abstract, keywords or a classification (`labelled_api`); (c) a store printed outside any group is never in a class's line (the group bound kept from 68x); (d) `glued_to_class_text` exempts only `\href`'s first argument as a link target, so WileyASNA-v1.cls's `\burlalt{http://dx.doi.org/\@doi}{…}` keeps `\@doi` the class's (1 census paper).
- Review rev79a findings 2, 4, Q1
- State: open (accepted unless a paper shows (a) or (d))
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K11's line rule (OD #662) after 79a r2 — a store printed in ".

### Design

- Design: decide a raw class's title-page line membership from TeX's own output — the speculative title-page run (`before_title_page_runs`) — instead of reading the class's code (`prints_in_a_line_with_another_value`, frontmatter_stores.rs).
- The code-reading rule keeps accreting exceptions (68x's shape test; 79a r1's printed-text rule; r2's sentence full stop, skip dimensions, running-head arguments; r3's head definitions; rev79a3's head optional arguments).
- Needs `harvest_stores` after the speculative run (an ordering change in sect05.rs's `\maketitle`).
- Principled-before-heuristics (user 2026-10-16);
- schedule with FM S2b (the observer reads print events), not as a one-off.
- 79a r2 writer's pattern report
- State: OPEN (design; task #367)

### natbib merged citations (`\cite{a,*b}` under `mcite`/`merge`/`elide`, REVTeX 4.1/4.2 unless `nomerge`)

- natbib merged citations (`\cite{a,*b}` under `mcite`/`merge`/`elide`, REVTeX 4.1/4.2 unless `nomerge`): since 78n the keys are a and b, but the merged entry is not folded into the one before it (natbib.sty:841-860 `\NAT@b*@key`), so the citation prints "[1, 2]" for pdflatex's "[1]", the bibliography keeps b as an entry of its own and every later entry is numbered one higher per merge;
- the `[pre][post]` notes are not added to the entry (`\nat@keyopt@open`/`\nat@keyopt@shut`).
- 1808.00247, 1509.05768, 1206.6986, 1209.2422;
- KPE #1081, guard `perfect_kernel_batch78n::revtex_merged_citation`
- State: open (78n r1-r3 residual; a wrong reference number, content kept)

### `\nonumber` inside an amsmath inner block of an `{equation}` (`aligned`, `split`) removes the equation's number; amsmath's `\let\print@eqnum\@empty` is local to the …

- `\nonumber` inside an amsmath inner block of an `{equation}` (`aligned`, `split`) removes the equation's number;
- amsmath's `\let\print@eqnum\@empty` is local to the block, so pdflatex prints it (2407.10017 `eq:chainMapHam`, PDF 41; int54a the same; found by 78r's `.aux` check, OD #865)
- State: open

### IEEEeqnarray

- IEEEeqnarray: a `\label` written before `\IEEEyessubnumber` on its row names the pre-advanced number in pdflatex (`\@currentlabel` as it is then), ours the row's sub-equation — common where the label opens the environment and the text writes `(\ref{X}a)`, which prints "(23aa)" here for the PDF's "(23a)" (r1→r2 A/B: 1608.03745 `Eq:MMSE_Qvar`, 1705.04514 `modulation`, 1705.10635, 1806.01087, 2003.10007, 2012.02697 `eq:dss`; probe `scratch_79d/r2/eq/lb.tex`).
- Fix direction: such labels name the environment's parent number — the `ltx:equationgroup` labelled with that number's reference forms — needing the label keys a row holds before the back-out (sect07.rs `\lx@eqnarray@label`) and `rearrange_eqnarray` to move them;
- not bounded for 79d r2.
- State: OPEN

### Cases.sty's `{subnumcases}` leaves the numbering in sub-equations (2001.04863: every number from (3) on "3x", r1 and r2 alike); pre-existing (int72a the same)

- Cases.sty's `{subnumcases}` leaves the numbering in sub-equations (2001.04863: every number from (3) on "3x", r1 and r2 alike);
- pre-existing (int72a the same)
- State: OPEN

### IEEEeqnarray*: a `\nonumber` *before* `\IEEEyesnumber`/`\IEEEyessubnumber` on a row still wins (`sect07.rs:270-272`

- IEEEeqnarray*: a `\nonumber` *before* `\IEEEyesnumber`/`\IEEEyessubnumber` on a row still wins (`sect07.rs:270-272`: the starred environment retracts the row at once, so the tags lose `preset` and the later command takes its non-IEEE branch);
- pdflatex prints (1) (2) (2a), ours (1) with the labels unresolved (review rev79d3 finding 2, probe `rev79d3/eq/v.tex`);
- fix: defer the retraction in an IEEE row of the starred environment too;
- pre-existing (r2 the same), 0 corpus uses found
- State: OPEN

### 1806.02261 (106 IEEEeqnarrays)

- 1806.02261 (106 IEEEeqnarrays): 5 printed equation numbers where the PDF has ~114 — the log's Errors start at `\smaller` undefined, then "Missing {" and a non-boxing group; flagged, not silent, but large (79d r3 report); pre-existing (r2 the same)
- State: OPEN

### REVTeX 3 (aps.sty:473-487)

- REVTeX 3 (aps.sty:473-487): `\nonumber` and `eqnarray*` redefine `\label` to an error or a warning, so pdflatex leaves such labels "??";
- ours since 78r resolves an `eqnarray*` label to the next number (the label's content kept, a reference pdflatex does not print; review rev78r finding 6)
- State: open — main-session decision 2026-10-22: keep the resolved number (no visible loss), not PDF-literal "??"

### `\RequirePackage{aas_macros}` before an AASTeX 6+ `\documentclass`

- `\RequirePackage{aas_macros}` before an AASTeX 6+ `\documentclass`: pdflatex 55 errors (an unclean oracle);
- ours prints the class's journal names (review rev78r finding 6)
- State: no action (main-session decision 2026-10-22: the oracle is unclean)

### algorithm2e

- algorithm2e: an algorithm's last line, closed by the environment's end rather than by `\;`, ignores the `\Indp`/`\Indm` offset (`\lx@prepend@indentation` runs only from `\lx@algo@endline`; algorithm2e_sty.rs env template): `\Indp Bb\; Cc \end{algorithm}` or `\DontPrintSemicolon … \Return Gg \end{algorithm}` sets Cc / "return Gg" at the margin, pdflatex one level in;
- 2508.06088's three algorithms end with `\Return` at the margin (pdflatex x 165.1, one level in).
- int54a and 78r alike;
- review rev78r2 finding 1.
- Fix: give the open last line its indentation before the environment's group ends (a no-op on an empty last line)
- State: open

### algorithm2e `\DontPrintSemicolon` / `\PrintSemicolon` are assigned globally (algorithm2e_sty.rs, Perl algorithm2e.sty.ltxml:131-136 the same) where algorithm2e.sty's are …

- algorithm2e `\DontPrintSemicolon` / `\PrintSemicolon` are assigned globally (algorithm2e_sty.rs, Perl algorithm2e.sty.ltxml:131-136 the same) where algorithm2e.sty's are a local `\renewcommand`, so a later algorithm in the same document loses every printed ";" (pdflatex "Hh;", ours "Hh"; int54a, 78r alike; review rev78r2 finding 4; SHARED)
- State: open

### cleveref `\label[type]{…}` after a `\caption` inside a minipage panel of a figure loses its type

- cleveref `\label[type]{…}` after a `\caption` inside a minipage panel of a figure loses its type: the panel (or the folded figure) carries the label but keeps `creftype` "fig." — `\begin{figure}\begin{minipage}{.4\linewidth}\caption{A}\label[appendix]{mp}\end{minipage}…` gives "fig.
- 1" where pdflatex prints "appendix 1" (0 errors; a subfigure's works, "appendix 2a");
- the type's tags have no target when `\lx@cleverref@labeltype` runs (cleveref_sty.rs:61-72 `continue`s) and the panel fold moves the labels, not the type's tags.
- int54a and 78j alike;
- review rev78j5 finding 2.
- Fix: carry the type's tags with the labels when the panel/fold merges them, or settle the type once its owner exists
- State: open

### A `\begingroup` left open in held text raises an Error TeX does not

- A `\begingroup` left open in held text raises an Error TeX does not: an `\autoref` name `\begingroup Fig`, `\creflabelformat{equation}{\begingroup(#2#1#3)}` — pdflatex 0 errors (only "\end occurred inside a group at level 1"), ours 1 ("Attempt to end mode restricted_horizontal");
- `\crefformat{figure}{\begingroup#2Fig.~#1#3}` — pdflatex 0 errors and the author's "Fig. 1", ours 1 ("leaves 1 group(s) open") and cleveref's default "fig.
- 1".
- 78j r5 (r4 raised 2);
- review rev78j5 finding 3.
- Direction: when only `\begingroup` frames are left open, `digest_in_own_group` unwinds them without an Error and keeps the form
- State: open

### subfigure's `\subref{b}` prints "1(b)" where pdflatex prints "(b)"

- subfigure's `\subref{b}` prints "1(b)" where pdflatex prints "(b)": subfigure.sty:412-420 `\subref` reads the `sub@b` label its `\label` writes (the panel's own number), the binding's `\subref` is `\ref` (subfigure_sty.rs, Perl subfigure.sty.ltxml:181 the same);
- since 78t numbers every panel more papers reach it (review rev78t finding 7).
- Fix: `\subref` shows the panel's own tag (its `ltx:tag` without the figure's number)
- State: open

### A `\label` after a raw `\refstepcounter` in a paragraph (a user environment's counter, 1505.02540: 8 references print "2", the section's) names the enclosing unit

- A `\label` after a raw `\refstepcounter` in a paragraph (a user environment's counter, 1505.02540: 8 references print "2", the section's) names the enclosing unit: no schema element there can carry the label and the number (`ltx:para`/`ltx:p` have neither `labels` nor `ltx:tags`).
- Proposed: `ltx:para` takes `labels` and an optional leading refnum-only `ltx:tags` (the 78g `ltx:picture` mechanism; the XSLT drops `ltx:para/ltx:tags`);
- reach ≥ 24 papers (78t item 5)
- State: open, awaiting the schema ruling

### algorithm2e sets a caption's text in a box placed at the style's caption position (`\global\sbox\algocf@capbox`, algorithm2e.sty:2305-2336: the bottom for …

- algorithm2e sets a caption's text in a box placed at the style's caption position (`\global\sbox\algocf@capbox`, algorithm2e.sty:2305-2336: the bottom for `plain`/`boxed`, the top for the ruled family), so a `\label` in the caption's text is written there, not at its source position: a key labelled in a `plain` algorithm's caption text and again after its last line names the algorithm in pdflatex, the line here (2511.10625 `alg:general_astar`: pdflatex 1, 78t 13; int58a 1).
- Fix: a caption-text `\label` of an algorithm2e float counts as written at the caption's position (OD #877 residual)
- State: open

### Sub-figures in a `tabular` cell (`\begin{tabular}{ccc}\subfigure[]{…} & \subfigure[]{…} \\ …`) are lifted out of the table and their rows guessed from their widths, not …

- Sub-figures in a `tabular` cell (`\begin{tabular}{ccc}\subfigure[]{…} & \subfigure[]{…} \\ …`) are lifted out of the table and their rows guessed from their widths, not the table's rows (1309.3581, astro-ph0205132: neither the PDF's grid; int58a and 78t alike);
- their stated widths are not evaluated at the class's real geometry either (`note_stated_widths` does not enter an alignment's cells: 1911.11519 S3.F3, `{subfigure}{\hsize}` in `p{0.45\textwidth}` cells under elsarticle's geometry, 4+3 in the PDF, 3+1+3 here; 2502.05562 S5.F4)
- State: open

### An adjustbox's (or `\resizebox`'s) shell around sub-floats renders a blank oversized row

- An adjustbox's (or `\resizebox`'s) shell around sub-floats renders a blank oversized row: the sub-floats leave the box (`absorb_figure_material`), and the box stays behind as an `ltx:p` panel holding `<inline-block width="1393.3pt">` with an empty `ltx:p` (review rev78x3 finding 4, probe `q4.tex` Figs 1-2: `\begin{adjustbox}{width=\linewidth,center}` of four sub-figures; the XSLT gives it `style="width:1393.3pt"`, `.ltx_inline-block` is `display:inline-block`, LaTeXML.css:250).
- Pre-existing (int68a the same for the environment form);
- 78x keeps the four sub-floats one row after it (`note_panel_hbox`).
- Direction: drop a box left with no content once its floats are lifted out, or fold its scale onto them
- State: open

### A raster graphic's explicit size comes out at 72/72.27 of it, rounded up to a whole pixel

- A raster graphic's explicit size comes out at 72/72.27 of it, rounded up to a whole pixel: `image_graphicx_sizer`'s pixel branch (`latexml_core/src/util/image.rs`, `apply_graphicx_ops` with `units_per_bp = DPI/72.27`) multiplies graphicx's bp values (`parse_graphicx_options`, `to_bp`) by pixels per pt, as Perl does (Util/Image.pm:162-163 `to_bp`, :237 `ceil($a1 * $dppt)` with `$dppt = DPI/72.27`; a Perl-origin defect, KNOWN_PERL_ERRORS #1142): `\includegraphics[width=114.6pt,height=1cm]{x.png}` is 114.187pt (158px) where TeX's box is 114.6pt, so two such panels stay one row of REVTeX `[preprint,twocolumn]`'s 229pt column where pdflatex breaks them (review rev78x3 finding 6; probes `rvpt.tex`, `rvpt2.tex`).
- The vector branch is TeX's (`graphicx_box_pt`, 78z).
- Direction: an explicit `width=`/`height=` is the box's size in pt, as TeX's; reach every raster with a stated size (box and row widths shift by ≤ 0.4 %)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A raster graphic's explicit size comes out at 72/72.27 of it".

### `\adjustbox{max width=\columnwidth}{\subfloat[a]{…}\subfloat[b]{…}}` in a float (revtex4-1 twocolumn)

- `\adjustbox{max width=\columnwidth}{\subfloat[a]{…}\subfloat[b]{…}}` in a float (revtex4-1 twocolumn): the two sub-figures land outside the scaled `ltx:inline-block`, which keeps an empty `<p><text/></p>`, and come out `P \| PP` where pdflatex sets one line holding one adjustbox;
- the figure counter steps twice, so the next float is numbered one too high (refnums `5`, `5a`… where pdflatex's `.aux` reads `\numberline {4}`).
- int69b, 78x r4 and r5 alike (pre-existing): adjustbox digests its content twice — fix in the adjustbox/subcaption interaction.
- Review rev78x5 finding 1, probe `rev78x5/p/m3.tex` (third figure)
- State: OPEN (RED)

### Sub-figures lifted out of their `figure*` into a paragraph of floats settled into a row (`settle_float_paragraph`) beside the float's minipage remnant (2202.03884 …

- Sub-figures lifted out of their `figure*` into a paragraph of floats settled into a row (`settle_float_paragraph`) beside the float's minipage remnant (2202.03884 S4.SS1.fig1: `{subfigure}{0.33\columnwidth}`s with `{center}` and `\resizebox` bodies around pgfplots `\input`s; the PDF 2+1, int67a 2+2, 78x r2 one per row, 78x r3 as int68a): the lift is the defect (the sub-figures belong to S4.F5);
- the row is set at the registers' widths (OD #885)
- State: open

### Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` with no breakpoint, tight TikZ, two 4.5cm sub-floats in a 252pt column, a nested `\subfloat` beside a 4.5cm one …

- Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` with no breakpoint, tight TikZ, two 4.5cm sub-floats in a 252pt column, a nested `\subfloat` beside a 4.5cm one with ~1pt to spare in a 246pt column — base the same, review rev78x5 finding 2, `rev78x5/p/m3.tex` fourth figure): 1108.5543 Fig 9, 2009.04593 Fig 5, 1712.01536 Fig 2, 1703.09165 (2 floats), 2406.05437 Fig 12; and the `figure_mixed_content` fixture's 0.56/0.42 minipages with `\hspace{8px}` (8.03pt), one overfull line in pdflatex (`\showbox`: both in one `\hbox`, glue set −1.0), two rows under the rule (review rev78x finding 10); and a subfigure.sty body whose stated `\hspace` makes its row overfull (two `\subfigure[]{\includegraphics[width=0.47\textwidth]{…}\hspace{0.5cm}}`: pdflatex one overfull line, glue set −1.0; int69b one row, 78x r4 two, the body's glue counted: review rev78x4 `n4.tex` F4)
- State: accepted residual (user 2026-10-22: overflow)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` wi".

### Panel grid markup

- **Panel grid markup** (grid design payload 3): the XML says a float's rows only implicitly (the runs between `ltx:break`s); missing are a panel's stated width (Perl subcaption.sty.ltxml:60-66 stores `width` and never emits it), spans outside a `tabular`, the column a panel takes across rows, gallery or matrix (int61a sets 1 of 10 labelled matrices as the PDF).
- Options (study Q4): (a) today's elements plus attributes — `panelwidth`/`panelspan` on a panel, `panelrows`/`panelcols` and `ltx_panels_gallery`/`ltx_panels_matrix` on the float (LaTeXML.model `figure_attributes`; the XSLT maps them to CSS custom properties as `@backgroundcolor`); (b) a row wrapper element; (c) a table grid.
- Sizing S1/S2/S3, a two-column class's `\columnwidth`, matrix detection by heuristic or `tabular` only (design study task #360); CSS in LaTeXML.css and ar5iv-css at the goal's end
- State:
  - RULED 2026-10-22: today's elements plus attributes (`panelwidth`/`panelspan` on panels, `panelrows`/`panelcols` + `ltx_panels_gallery`/`ltx_panels_matrix` on the float, XSLT → CSS custom properties); sizing keeps the panels' relative sizes within a figure (no per-image cap that skews them — precomputed sizing values as CSS custom properties; design proposal owed) — after the grid rule payload.
  - MARKUP FIXED (79c, OD #887): `annotate_panel_grid` (after `arrange_panels` and `collapse_float`) states `panelrows`/`panelcols`/the class on floats of 2+ panels, `panelwidth` where the width is the real geometry's and stated (`panel_width_is_stated`; never a guess), a matrix only on a `tabular` of panels or 2+ rows agreeing panel for panel (widths within 0.01 of the line: 0.0-0.01 score alike on the oracle, 0.02 adds a false matrix) with spanning rows, and sub-captions on none of the panels or on exactly one whole row (79c r2); nested groups their own; schema `Panel.attributes` in `Common.attributes` (panels are any element) and `PanelGrid.attributes` on figure/table/float; XSLT `--ltx-panel-rows/cols` on the float's element, `--ltx-panel-width/span` on its `div.ltx_flex_cell`.
  - Oracle (453): study matrix 42 → ours 20 matrix / 22 gallery (16 text-labelled matrices), study gallery 411 → 405 gallery / 5 matrix (grids captioned on one row only) / 1 none; rows 429, shapes 422 unchanged.
  - A/B int71a → 79c (1,907): E 6,680 = 6,680, F 20 = 20, W 59,351 = 59,351, words equal; with the markup stripped 1,855 papers byte-identical, the 52 others the label item's (row above).
  - Sizing RULED 2026-10-23: the float states `panelline` (its line ÷ the PDF's running-text column) and `panelfill` (widest row's Σ `panelwidth`); a panel renders at `panelwidth × panelline` of the theme column ÷ max(1, `panelfill × panelline`); no CSS rule reads them yet (goal-end CSS).
  - 79c r2 (review rev79c): a `tabular` cell counts only for a graphic 24pt+ wide and tall (13 data tables lose the matrix), sub-captions allow a matrix only on exactly one whole row (6 partial-caption grids → galleries), a beside-content collapse drops the inner's widths, a `width=`-only graphic panel states its stated width, the grid attributes typed numeric and the XSLT emits numbers only; guards `perfect_kernel_batch79c::*`, `integration::panel_grid_markup_states_only_numbers`.
  - Residuals: text-labelled matrices (label rows are `ltx:p` panels), a `\resizebox` of measured content states no width (its stated width is not on the element), a raster inside a sub-float body keeps KPE #1142's rounding in its sub-float's width; a float held in a minipage — FIXED 79f (the page outside the box, 0.5 as the PDF); a `\captionof` figure gets no grid markup (not a float environment; Perl alike); class pages declared — FIXED 79f (OD #887).
  - LANDED int74 (61039b657e + 140d474145)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Panel grid markup".

### The natural `cssstyle` is the size pdflatex prints a graphic at (`first_natural_size`, util/image.rs; 79a), but post renders the candidate it finds most desirable …

- The natural `cssstyle` is the size pdflatex prints a graphic at (`first_natural_size`, util/image.rs; 79a), but post renders the candidate it finds most desirable (`latexml_post/src/graphics.rs:321-371`), which may be another file of another shape: 2107.02597's gnuplot figures (`\DeclareGraphicsExtensions{.pdf,.eps,.png,.jpg,.jpeg}`, a 640x480 png beside each eps) render the png (4:3) at the eps's `36.135em x 25.294em` (aspect 1.43), about 7 % out of proportion (int68a the same, `cssstyle` A = B).
- Options: post renders the candidate the box is sized from when it can convert it (an eps through gs), or the `cssstyle` keeps the box's width and lets the height follow the rendered file.
- Review rev79a finding 5
- State: open

### `\Gin@extensions` is seeded with pdftex.def's list (`graphics_sty.rs`, pdftex.def:597-600) under every driver option, though `\Gin@driver` names the option's driver …

- `\Gin@extensions` is seeded with pdftex.def's list (`graphics_sty.rs`, pdftex.def:597-600) under every driver option, though `\Gin@driver` names the option's driver (`graphics_driver()`): `\usepackage[dvips]{graphicx}` loads dvips.def in TeX, whose list is `.eps,.ps,.eps.gz,.ps.gz,.eps.Z,.mps` (dvips.def:202), so pdflatex takes `q.eps` (40.15pt) beside `q.png` where ours (int68a, 79a) takes the png (144.54pt);
- likewise `[dvipdfmx]`/`[xetex]`.
- Fix: seed by `graphics_driver()` from each driver file's list.
- A corpus-wide shift for every `[dvips]` paper with a raster and a vector of one basename: its own A/B. Review rev79a2 finding 3, probe `/home/deyan/.claude/jobs/356a0c47/tmp/rev79a2/g/gh.tex`
- State: open (ranked: after the panel grid payloads)

### graphicx's natural-size keys are not read by the sizer

- graphicx's natural-size keys are not read by the sizer: `bb=`/`bbllx=`…, `natwidth=`/`natheight=` and `hiresbb` set the natural size in TeX (Perl reads none of them either): `\includegraphics[bb=0 0 10 10]{e.eps}` of a 40bp EPS is 10.03749pt in pdflatex, ours 40.15pt (int67a 28.908pt).
- Witness cond-mat0610021 (`bb=0in 0in 2in 2in`; Fig 11's sub-figure then breaks where TeX sets one overfull `\hbox`).
- Review rev78z finding 5
- State: OPEN

### A float LaTeX defers writes its `\newlabel` when placed, so a later `\label` of the same key loses to it

- A float LaTeX defers writes its `\newlabel` when placed, so a later `\label` of the same key loses to it: `\begin{figure}[p]\caption{Deferred}\label{k}\end{figure}\section{Two}\label{k}See \ref{k}.` — pdflatex "See 1.", ours, int58a and Perl "See 2." (review rev78t2's lb2; OD #877, KPE #1131);
- rare
- State: open

### An author's mark whose label holds any control sequence prints nothing

- An author's mark whose label holds any control sequence prints nothing: the mark reader reads the label from the operand's unexpanded text (`author_macros.rs:198` `clean_frontmatter_labels(&operand.to_string(),…)`), which no list answers, so `\textsuperscript{\relax 3}` loses its "3" (pdflatex "Ann Alpha3"), `\textsuperscript{\one}` and `\textsuperscript{\protect 2}` their "1" and "2", `\textsuperscript{\relax 1\thanks{a}}` and `\textsuperscript{\protect 2\thanks{b}}` keep the note but lose the label, as do `1\protect\protect\thanks{b}`, `{\protect}1\thanks{f}` and `\protect\protect\thanks{c}`, and `1,\protect\one\thanks{n}` gives `<sup>1</sup>` where pdflatex prints "1,1".
- No error is reported. int62a, int63a and 78v alike (older than 78v); synthetic, no census paper.
- Fix: read the label from what it typesets (expand macros, drop an unexpandable `\relax`/`\protect`) before matching it.
- Review rev78v2 finding 6a, widened by rev78v3 finding 3 (probes `rev78v3/q/pl.tex`, `pra.tex`, `prc.tex`, `rev78v2/p/sh3.tex`); RED repro `sectioning-frontmatter/author_mark_relax_before_its_label`
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "An author's mark whose label holds any control sequence prin".

### A note macro reached through four macros (`\fnM→\fnN→\fnO→\fnP{8\footnotemark}`) inside an author's mark loses both its "8" and the footnotemark (pdflatex "Di Delta8†")

- A note macro reached through four macros (`\fnM→\fnN→\fnO→\fnP{8\footnotemark}`) inside an author's mark loses both its "8" and the footnotemark (pdflatex "Di Delta8†"): `makes_a_note`/`prints_beside_a_note` (frontmatter/classify/preprocess.rs) look three deep at most, so the macro is neither taken out nor read in place and stays in a label nothing answers.
- int62a, int63a and 78v alike;
- synthetic.
- Fix: past the depth bound, fall back to taking the macro out whole (the int62a behaviour for a note macro).
- Review rev78v2 finding 6b;
- RED repro `sectioning-frontmatter/author_mark_four_deep_note_macro_chain`
- State: open

### Author marks

- Author marks: inside a note macro's body, a `\protect` before another note-making macro (`\newcommand\fnL{\footnotemark}`, `\newcommand\fnB{7\protect\fnL}`, `Lu L\textsuperscript{\fnB}`) is not label-safe, so the macro is taken out whole and its "7" lands in the name text: `L7[mark]` where 78v r2 gave `L[mark]<sup>7</sup>` (pdflatex "L7§").
- `prints_beside_a_note` (`frontmatter/classify/preprocess.rs:199-202`) lets a `\protect` through only before a note command, where `split_mark_notes` (`:71-74`) also takes one before a note-making macro.
- Markup only, nothing printed lost; 78v r3 = int63a; synthetic, no census paper.
- Fix: accept a `\protect` before a `makes_a_note` macro in the body loop as `split_mark_notes` does, a row in `perfect_kernel_batch78v::author_protect_thanks_and_note_macro_inside_a_mark`, OD #881 "before a note command or note macro".
- Review rev78v3 finding 1 (probe `rev78v3/q/prb.tex`)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Author marks".

### A formula silently degrades to `ltx_math_unparsed` under load

- A formula silently degrades to `ltx_math_unparsed` under load: 1001.0023 `S9.E2.m1.pic1.m2` once in a census run (FM r9 writer's set-mode census), never in 3+1+2+2 reruns;
- the log identical, so no warning — the math parser's 2 s pruned-only budget (`parser.rs:235`) is the only time-based path found (revfms2a2_9 finding 7).
- A time budget that changes output without a diagnostic hides regressions in A/Bs: emit a Warning when the budget cuts a parse, or make the cut deterministic
- State: open

### Complete breakage at 100 % (user 2026-10-24)

- **Complete breakage at 100 % (user 2026-10-24)**: a missing image, a `\ref` or `\cite` with no target, where pdflatex resolves it.
- Method: a fresh sample per round (`one_side.sh`, `scratch_356a0c47/brk<NN>/`), `scratch_356a0c47/fms2b_census/scan.py` over the core XML (a `graphics` with no `candidates`, a `labelref` with no label, a `bibref` with no `bibitem` — skip `show="nothing"`), then `classify.py` against the source (a file of that name shipped, a `\label`/`\bibitem` in a file this document reads — strip comments and post-`\end{document}` text, exclude xr\'s other documents); a label a macro makes classifies UNDEF, so read those too.
- Fixed 79g–80r (int80–int101: image paths and names, xr, `\label` redefinitions, pgf images, front matter groups, file names, alignments, listings, `\import`, subfig `\subref`, ctable, cleveref's braced comma, subeqnarray's `\slabel`, a longtable's labels, autonum, `\newlabel` records, theoremref, acronym's options, caption's `\captionbox`, nicematrix's `\tabularnote`, `\pdfstrcmp`, a guessed theorem that refused the document's `\newcommand`, a longtable's macro-built label key, a `\cref` list held in a macro, siunitx's unit commands, aa.cls v9.1's `\longtab`, a `\label` after an eqnarray's last row, a column end met while a false branch is skipped (the hide-column `\iffalse` template), `\maketitle`'s call of a redefined `\twocolumn` (the teaser figure), acronym's protected commands in a siunitx `S` cell, iucr.cls's `\hcaption`, amsmath's `\collect@body`, `\everypar` ahead of a horizontal command (tex.web §1090), a redefined `\newtheorem` start; each with its witnesses and A/B in [`archive/SYNC_SESSIONS_2026-10.md`](../archive/SYNC_SESSIONS_2026-10.md) int80–int105).
- Sweeps: brk81 (3,000 papers: 186 dangling `\ref`) → brk86/brk89 re-runs (`\ref` 1,060 → 310) → brk94 (2,000 fresh papers on int94a: 61 with breakage; 3 OURS, 49 UNDEF, 9 ABSENT) → brk96 (2,000 fresh papers on int96a: 59 with breakage; 12 OURS, 45 UNDEF, 2 ABSENT) → brk100 (2,000 fresh papers on int100c: 78 with breakage, 17 with OURS keys — 3 fixed, 6 not ours, 7 Fatal papers and a minted cache) → brk102 (2,000 fresh papers on int101a: 71 with breakage, 9 with OURS keys — none cheap: a redefined theorem start macro, a hide-column `\iffalse`, amsart's stored `\address` labels, error papers; the hide-column one fixed 80n) → brk104 (1,200 fresh papers on int101a: 44 with breakage, 5 with OURS keys — 2 fixed 80n, 2 not ours, dgruyter.sty's raw `\@array` fixed 80r) → brk106 (1,200 fresh papers on int102a: 38 with breakage, 6 with OURS keys — 1 fixed 80o, 1 stale `.bbl`, 4 error papers).
- Not ours (pdflatex or the arXiv PDF breaks too): a file not shipped, xr without the other `.aux`, a stale shipped `.bbl` (ruling 2026-10-24: 2509.06285, 2205.10368, 2402.06576, 2311.01729, 2205.00263, 2004.00833, 2507.01293, 1806.02261), the harness\'s main-file guess, a document\'s own `\def\label`, a shipped namesake of a TL package (ruling 2026-10-17 H1: 0802.3014\'s and 1210.0342\'s pretex `units.sty`).
- Residuals and the brk94/brk96/brk100/brk102 tails: [`perfect_kernel/OPEN_ROWS.md`](OPEN_ROWS.md) § "Breakage residuals".
- State: OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Complete breakage at 100 % (user 2026-10-24)".

### FM S2b: the observer's writer on by default for revtex4_support (OD #837 "S2b r1") — the residual

- **FM S2b: the observer's writer on by default for revtex4_support (OD #837 "S2b r1") — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left 2608.23944 (an uncarried symbol line under ruling 2 in a one-author group: a ruling) and 2602.16003;
- open (ruling): the mark-picked path does not show a line's mark (219 guards).
- State: OPEN (residual)

### K11 store residuals after 95d

- **K11 store residuals after 95d** (OD #939-#941; the scorer's four lost-words papers 2607.06973, 2608.26053, 2307.16262, 1202.2889 fixed; 95d-r2: a setter's own store only when no running code reads it, a date's label in an `\else` branch, an unlabelled date unnamed, fancyhdr styles read not run; 95d-r3: a date's label read in any code the title page reaches, a label phrase ending at a `;` or `,`, the label before a store's emptiness test, a role's default only where its word is printed right before the store): unsupported shapes, each a one-line input — (a) a footer token register selected only at the class's top level or in `\AtBeginDocument` (`\pagestyle{firststyle}` outside `\maketitle`), or set by `\fancyfoot[L]{\the\correspondingauthor}` under `\pagestyle{fancy}` outside `\fancypagestyle` (fancyhdr's fields stay no-ops), stays unread; (b) the same classes' `\paperurl` / `\reportnumber` registers (deepseek.cls:113-114, googlecloud.cls:81-82) are not in the table; (c) a macro store a `\ps@` style prints under another name (`\def\correspondingauthor#1{\gdef\@corr{#1}}` + `\def\ps@firststyle{\def\@oddfoot{\@corr}}`) is no store (its readers are no title-page code); (d) a footer toks value set after `\maketitle` (pdflatex prints it at shipout) is not read; (e) a register printed under a switch the class leaves false (`\ifshowca … \the\correspondingauthor\fi` + `\showcafalse`) still makes the note — `printed_register` knows only the `\if\relax\the\X\relax` test (RED `toks_store_printed_under_a_false_switch_is_no_note`, rev95d S4; fix: read the printing branch live or dead, as `reads_of_branches`); (f) a preamble `\fancypagestyle{firststyle}{\fancyfoot[L]{Preprint}}` that drops the register still makes the note — the style is read at the class's end (RED `toks_store_page_style_the_document_redefines_is_no_note`, rev95d S5; fix: re-read `\ps@<style>` at `harvest_stores`); (g) a setter's own store no macro reads, or one read by a page style the class never selects, is still rerouted — kept for astlb.cls:361-372's `\@shorttitle`, printed only by a running head its top level sets (1202.2889's `<toctitle>`) —, so a dead store (`\def\keywords#1{\gdef\@keywords{#1}}`, nothing reading `\@keywords`) is an invented item; whether an uncalled macro runs is not read off the code (`read_where_nothing_runs` takes any reader the never-running code reaches: cesj.sty's unselected `\cejpmaketitle`, physics0510010); (h) a label printed through a macro that reads the store (ujp.sty:107/:421 `\udkpacs~\udkpacsnum`, `\def\udkpacsnum{\@pacs}`; 1906.11531), or in a group that is a command's argument with layout the label reader does not take (CSML.cls:227-229's `\@footnotetext{\textit{\ACMCCSname:}\enspace\@ACMCCS.}`, the stand-in tpccs.cls:12; 1608.04592, 1602.00476), is not read: the classification is unnamed where the class prints "PACS", "2012 ACM CCS:" (review rev95d-r2 finding 3; charter-correct, better hidden than mislabelled); (i) `\fancycenter{a}{b}{c}` called in a style body outside a field (pdflatex typesets it where `\pagestyle` runs the style) prints nothing — the style runs nothing (int124a and Perl the same); (j) an unlabelled date among labelled ones is handed on as it is set, the labelled ones at `\maketitle` (`labelled_api` defers only a named one), so it comes first: medima's shape with `\@finalform` printed bare gives the final-form date before "Received"; (k) a raw class's own store only an uncalled macro reads (`\def\makecover{… \@keywords …}`, never called) is still an invented item — the raw package's test (`read_where_nothing_runs`) is not applied to classes, where it lost bfhthesis's `\@department` (read through a `\@titlefooterleft` a scrlayer layer prints) and dmtcs's keywords (0905.1102) (RED `raw_class_store_only_an_uncalled_macro_reads_is_no_item`, rev95d p9); (l) a role's default kept because its word is printed right before the store (`ClassReaders::prints_default_name_before`: wiley2sp.cls:296-302's `\receivedname\ignorespaces\@received`, 1211.4450, 1306.2559; zaa.cls:513-515's `\AtEndDocument` hook `Received \rm\@received`, 1902.07328, the date also printed there, the class path's double print of (k)) leaves that printed label in the title-page deposit ("Received:" over nothing, as before 95d-r2; `perfect_kernel_batch95d::a_date_with_no_label_read_keeps_its_default_only_where_printed`), and a date stored right after a `;` or `,` (no label of its own: `\@revised; \@accepted`, iaus.cls's `No. \@volume, \@pubyear`, 0707.3527) leaves that separator there, the unnamed date first by (j); (m) a raw package's table store that no macro reads is still rerouted (`read_where_nothing_runs` takes it for one a running head its top level sets), so `\def\keywords#1{\gdef\@keywords{#1}}` with nothing reading `\@keywords` invents `<keywords name="Keywords: ">` and a received date (RED `raw_package_store_no_macro_reads_is_no_item`, review rev95d-r2 finding 2, probe qn; a static scan of the 712-paper package population found no real paper); (n) pazha2.cls's `\submitted` date ("Received December 15, 2022" in 2408.01858, "Received January 21, 2025" in 2607.07582) is in no output: `submitted` is not in STORE_SETTERS, its class is read raw and the refused `\endabstract` that runs `\maketitle` never prints the title page (one-line input: `\def\submitted#1{\gdef\@submitted{#1}}` printed only by a refused `\@maketitle` reached through `\endabstract`); OLDER than 95d — int123a and int124a lose it too
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K11 store residuals after 95d".

### 95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synthetic only, no census paper changed this way)

- **95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synthetic only, no census paper changed this way)** — (S1, a content loss on its shape) the `\else`/`\or` whole-test removal (`frontmatter_stores.rs` `line_label_before` ~:5302-5315, `end = past` ~:5349-5353, `enclosing_conditional` ~:5492) checks only that nothing precedes the store in its branch, so text printed after the store inside the test leaves the title page with it and is printed nowhere: `Received: \if@twocolumn\else\@received\ (preprint version)\fi` loses "(preprint version)" (base and r2 keep it; a static scan of the 9,502 class-shipping census zips found no such shape) — fix: take the whole test only when the store's branch holds just the store and layout and the test is the store's own emptiness test, else the r2 path (the `\ifcase`/`\or` variant too); reverting that arm alone restores r2 and turns only the witness-less guard `a_date_labelled_before_its_emptiness_test_is_named` RED; repro `sectioning-frontmatter/raw_class_date_branch_with_trailing_text_keeps_it.tex`.
- (S2, markup) the `;`/`,` clause split (~:5372-5385, meant for pazha2's date clauses) cuts every store's label: `Index terms, keywords: \@keywords` → `name="keywords:"` + a stray "Index terms" paragraph — split only after a clause holding a store or value, or only for date/classification calls, and say so in OD #940; repro `raw_class_label_with_a_comma_stays_whole.tex`.
- (N1) `text_right_before` reads every branch of a name macro (a German `\recname` keeps the English default; base the same).
- Pre-existing, from the review: 2109.14785 (ectj.cls:352 commented out) emits dates it never prints — a real-paper witness for (m) above; 1409.1462 loses the printed "(January 2015)" set by `\received` inside `\affil` (base and r3; content loss); 2210.05326 a stray "Received:" paragraph.
- Review `agent_reports/2026-10-31_rev95d_r3.md`
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synt".

### 95e residuals (OD #938)

- **95e residuals (OD #938)** — the several-address `\email` give-back (2109.07363 three creators again, 0 E) leaves: (a) `\address{$1.$ …}` labels unread, so the reading's upfront grid gives the entries by position, its pile lacking the listed call (`answer_listed_emails` runs after `Resolver::distribute_upfront`): 2109.07363 Liu {1}, Tao {2} where the PDF prints {1,2} for both (RED `amsart_dollar_numbered_addresses_under_shared_marks`), synthetic reversed marks both wrong where int123a had one wrong (RED `amsart_dollar_numbered_addresses_under_reversed_marks`, rev95e S1); fix: the call one contact in the grid's pile, as today's, or the `$1.$` label read; (b) a listed call's address spelling no one is the call's author's (`owner:call`: 2109.07363 `ferrytau@` on Wei, Tao by the list's order); (c) the fail-safe keeps every listed call whole when one cannot be written out (RED `amsart_several_address_emails_one_kept_whole_keeps_all_whole`, rev95e S3; = int123a); (d) revtex `\author{Ann Able $\&$ Bob Baker}` stays one creator with no `merged_creators` Error (= int123a)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95e residuals (OD #938)".

### Unlanded payload branches (inventory 2026-10-31; all work reintegrates into perfect_kernel, user 2026-10-31)

- **Unlanded payload branches (inventory 2026-10-31; all work reintegrates into perfect_kernel, user 2026-10-31)** — each kept on its branch for a recorded reason, none silently: `pk/66p` 41122534fa (a sub/superscript mark outside math, tex.web §1046; ruled unlanded, `feedback_rulings_error_fatal_policy`); `pk/68c` 66eb0995f9 + `pk/68c-loader` (a shipped `.sty` that is another package; Part A not landed by the 2026-10-17 H1 ruling, the binding wins); `pk/68p2` e78ea52da6 (address-space memory fuse: PARKED, STABILITY_WITNESSES cluster K); `pk/88a` a754a94eac + `pk/88a-r3-wip` (e-mail spelling: held for the symmetric whole-token redesign, task #422); `pk/88c` 377529655a (font-switched marks: dropped 2026-10-29, folded into the digested-representation refactor stage 3); `pk/66u` 1b7360cc1e (K9 row F, the author tabular-break split — `author_block_reopenings`: a macro that closes and reopens the author block's environment read as `\and`; on its 370-paper family Errors 4,387 → 379, error-free 2 → 318 at 2026-10-14; r8 gates green on its 2026-10-15 base).
- **RULED 2026-10-31** (user): after the int125a bundle lands, rebase; if clean, one fresh round (r9); if not, retire with its residual rows.
- **RETIRED 2026-11-01** (row "K9 row F residual": 264 Errors in 29 of its 62 witnesses on the pushed tip).
- Trial merge against int125a (`git merge-tree`): NOT clean — `latexml_engine/src/base_utilities.rs`, where its 1,261 added lines live, was split into `base_utilities/` since (modify/delete), plus 12 conflict hunks (state.rs 5, error.rs, tex_debugging.rs, docs, the test list); so the retire path: re-derive its family from the branch's `perfect_kernel_batch66u` witnesses and repros (its scratch tables are gone), measure E/F on the pushed tip, and record what is left as a residual row with those numbers
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Unlanded payload branches (inventory 2026-10-31; all work re".

### K9 row F residual: a macro that closes and reopens the author block's environment (66u RETIRED 2026-11-01, user ruling 2026-10-31)

- **K9 row F residual: a macro that closes and reopens the author block's environment (66u RETIRED 2026-11-01, user ruling 2026-10-31)** — `\author{\begin{tabular}{c}Ann\end{tabular}\hskip1em\begin{tabular}{c}Bob\end{tabular}}`-style blocks (sig-alternate `\alignauthor`, hand-made `\end{tabular}…\begin{tabular}` breaks) read as `\and` by pdflatex leave a run of Errors: on 66u's 62 code/test witnesses (`pk/66u` `perfect_kernel_batch66u`; job tmp `w66u.txt`, `m66u/run.sh`) the pushed tip (int125c `8a87df771c`) gives 264 Errors in 29 papers, 33 error-free (int123a 272 / 32; the bundle fixed 2405.18462) — `unexpected:\hbox` 66, `\lx@personname` 52, `\endgroup` 52, `\@@tabular` 52, `\@end@tabular` 39, `&` 3; top 1106.3195 66, astro-ph0507026 55, 0810.5607 30, astro-ph9904014 / astro-ph0007224 / 1101.0179 / 0812.4013 6.
- The branch's mechanism (`author_block_reopenings`, `pk/66u` 1b7360cc1e: Errors 4,387 → 379, error-free 2 → 318 on its 370-paper family at 2026-10-14) lives in the pre-split `base_utilities.rs` (now `base_utilities/`) and does not rebase (modify/delete + 12 conflict hunks); re-implement it on the current layout as a fresh batch with the branch's witnesses and repros as the red set
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K9 row F residual: a macro that closes and reopens the autho".

### Bundle int126a/int126b (97c-98e) — the residual

- **Bundle int126a/int126b (97c-98e) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a REVTeX `superscriptaddress` `\affiliation` with an unmarked first line lands at the block (1507.03321, 2312.15705);
- 2506.19040's `drohr@` copied to the other `*` authors;
- BESIII `(A)` sub-entries (R6);
- no Mark event for nucleus formulas;
- 2112.13115's leading "and";
- sp2 (an unanswered line's label dropped);
- 1609.03822's `\pounds` label unsplit.
- State: OPEN (residual)

### Frontmatter on the digested representation — the freeze and the refactor's batches 90a-95c (task #423; now `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, row 0b) — the …

- **Frontmatter on the digested representation — the freeze and the refactor's batches 90a-95c (task #423; now `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, row 0b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `amsart_email_of_a_two_name_call_spelling_an_accented_name` (2112.10604) and `revtex_email_spelling_its_holders_given_name`; amsart's routed `\thanks` (the row "A routed amsart note").
- REDs' witnesses 0711.3633, 0809.3090 (`santos@` off Santos Bravo Yuste); open RED `raw_class_store_trailing_line_after_last_mark` (quant-ph0101015's rspublic.cls store line after the last mark); left: 1304.1574 (`Computer Science` read as names), llncs's 1901.07288 (a trailing-mark `\institute` whose entries span two lines), 1505.06588 and 2203.08964 (scorer source-reading limits, ours the PDF's).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Frontmatter on the digested representation — the freeze and ".

### A routed amsart note belongs to the last live author at `\maketitle`; needs a change tag and a ruling

- **A routed amsart note belongs to the last live author at `\maketitle`; needs a change tag and a ruling** (92a review rev92a_r3 s1; main-session decision: defer to the ams written-family work): amsart.cls:516 keeps each `\thanks` in `\thankses`, run at `\maketitle`, and its `\email` (:508) appends to `\addresses`, so a `\thanks{Correspondence: \email{x}}` between two authors, or before every `\author`, prints x after the last author's address.
- We read the author current at the call, as the binding routes it: between two authors the first's (today's and Perl's), before every author the block's, the whole call however many addresses it holds (today's a nameless creator, Perl's lost; GREEN `amsart_thanks_before_every_author_two_addresses`, review rev92a_r4 S1).
- Moving it changes written output against Perl.
- RED catalog repros `class-bindings/amsart_thanks_between_authors_email`, `amsart_thanks_before_every_author_email`
- State: DEFERRED (ams written family)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A routed amsart note belongs to the last live author at `\ma".

### Compound marks: one rule for the superscript path, the formula path's runs and the legend's label (94a r9 residual; OD #928 round 9, OD #927)

- **Compound marks: one rule for the superscript path, the formula path's runs and the legend's label (94a r9 residual; OD #928 round 9, OD #927)**: the three readers part a mark of several symbols differently — the superscript path at commas only (`apply::show_legend_mark`'s `ltx:sup` loop: `Ann Able$^{*\dagger}$` under a `$^{*}$Corresponding author` legend shows "*†" beside the note's ∗, the star twice, once as ASCII `*`; w01), the formula path by `token_runs` and `repeated_symbols` (`$^{1,*\spadesuit}$`'s ∗ taken with the comma before it, "1♠"), the reading's labels by `Entry::compound` and `mark:parted` (`**` read as `*,*`, so a `$^*$` legend is the `**` carrier's, w11; the kernel then hides the `**`: `\author{Ann Able$^{1,**}$, Bob Baker$^{1}$\\ $^1$Univ X\\ $^*$Corresponding author}` prints "1,∗∗", ours no mark, as 93a), and a `$^{*\dagger}$Corresponding author` legend reaches Bob Baker, who carries only `*` (`Ann Able$^{1,*\dagger}$ and Bob Baker$^{1,*}$`, w10, since 93a).
- Constraints any rule keeps: 2602.19190 (a run of distinct symbols labels a line as a whole), 2606.15609 (`$^{*\spadesuit}$` requests each symbol), a repeated symbol a mark of its own (94a r9 guard `a_repeated_symbol_beside_a_legend_is_its_own_mark`).
- A partial take also drops the comma before the taken symbol: `$^{1,**\dagger}$` under a `**` legend gives "1†" where pdflatex prints "1,∗∗†", and `$^{1,\dagger\ddagger}$` under `†` gives "1‡".
- Remove the comma only when the whole comma-delimited run is taken (`apply.rs` ~l.258; rev94a r9 r12).
- RED catalog repros: `author_names_compound_mark_superscript_beside_its_legend`, `author_names_compound_legend_goes_to_a_partial_carrier`, `author_names_kernel_repeated_symbol_beside_a_legend_hidden` (the kernel path's `**` hidden; rev94a r9 a03, siblings a01, a05, a10, a11)
- State: OPEN (refactor)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Compound marks: one rule for the superscript path, the formu".

### A trailing answered name takes the unmarked in-place line, before its own (94a r9 residual P3)

- **A trailing answered name takes the unmarked in-place line, before its own (94a r9 residual P3)**: `ANN ABLE$^1$ and BOB BAKER and CID COLE$^1$ \affil{$^1$Univ A} \affil{Univ B}` (a class printing `\affil` in place) gives Univ B to BOB BAKER and CID COLE, CID COLE's Univ B before his Univ A; pdflatex prints Univ B after the names, CID COLE answered by Univ A. Today's API walk (`frontmatter_api.rs` `reached_unanswered`) and the reading's (`Resolver::record_run`) start at the trailing name and stop only once past an unanswered one: the premise "when the last author is answered, every author of the run is" fails with an unmarked name between answered ones. authblk's `\author{Ann Able$^{1}$}\author{Bob Baker}\author{Cid Cole$^{1}$}\affil{$^1$Univ A}\affil{Univ B}` the same (review rev94a r8 u04, z14).
- RED catalog repro `raw_class_affil_run_trailing_answered_name_takes_the_unmarked_line`
- State: OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A trailing answered name takes the unmarked in-place line, b".

### IEEEtran's `annotate=new` affiliation block after a marked one, contact order (94a r9 residual P4; since 93a)

- **IEEEtran's `annotate=new` affiliation block after a marked one, contact order (94a r9 residual P4; since 93a)**: `\IEEEauthorblockN{Ann Able$^1$ and Bob Baker}\IEEEauthorblockA{$^1$Univ X}\IEEEauthorblockA{Univ Y}` gives Ann Able Univ Y before Univ X (the print's order X, Y);
- the same with `\IEEEauthorrefmark{1}` (review rev94a r8 z11, z07).
- RED catalog repro `ieee_affil_new_block_after_a_marked_name_contact_order`
- State: OPEN

### A raw class's line whose mark only a later `\author` requests stays the class's line (94b r7 residual; it was 94b r4's S1)

- **A raw class's line whose mark only a later `\author` requests stays the class's line (94b r7 residual; it was 94b r4's S1)**: in an appending class (capa-like `\let\@author\@empty\renewcommand\author[2][]{\g@addto@macro\@author{#2 }}`), `\author{Ann Able\inst{1}}\address[\inst{2}]{Univ B, City}\author{and Bob Baker\inst{2}}\maketitle` prints "(2) Univ B, City" where it is called and Bob Baker gets no contact: at the call no author requests 2, so the line is in doubt — the class's, digested once (`frontmatter_stores.rs` `class_contact`, OD #936 round 7).
- The three tries to decide it later (94b r5 a replay after the title, r6 a call kept beside the printed line and lifted) each reordered the body or digested the line twice ("Body 2." for pdflatex's 1); no census paper takes the shape (300 setter census, cpc-hepnp 60, the 3,000 sample).
- Review rev94b r4-r6 probes `p/m/later`, `p/n` aug/tc/two/l3, `p/u` u1-u3, `p/w` w1-w2, `p/s/s4`.
- RED catalog repro `raw_class_optional_mark_a_later_author_requests`
- State: OPEN (deferred)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A raw class's line whose mark only a later `\author` request".

### Per-entry wraps of nested containers, and an `\href` around entries (94b r7 review residuals; synthetic, triage 2026-10-31: RED catalog, no fix round)

- **Per-entry wraps of nested containers, and an `\href` around entries (94b r7 review residuals; synthetic, triage 2026-10-31: RED catalog, no fix round)**: (1) a container inside a kernel text font command with more entries after its group loses the container's font (`\textbf{\emph{$^1$…\\ $^2$…}} $^3$…`: bold kept, italic lost; `emitter/plan.rs` `entry_forms`/`container_wrap`); (2) `events/fonts.rs` `font_declaration` matches by name, so a redefined `\bfseries` (`\quad`) is copied into every entry (the B2 class: what prints once copied into each entry) — compare meanings (the S3 `x_equals` pattern); (3) the allow-list between nested containers (`plan.rs` ~:681-687) refuses a text font command between containers, `\relax`, `\fontseries{b}\selectfont`, `\LARGE`/`\huge` (plain cut, = base); (4) `\href` around the entries (± a font command) → a nameless creator holding the contact and two names merged (pre-existing since 94a r9).
- RED catalog repros `raw_class_affil_text_font_container_then_more_entries`, `raw_class_affil_redefined_font_declaration_between_containers`, `raw_class_affil_text_font_command_between_containers`, `raw_class_affil_href_around_entries_nameless_creator` (review rev94b r7 probes p/z/bt, rds, ebun, ihref; p/x/href)
- State: OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Per-entry wraps of nested containers, and an `\href` around ".

### A `\thanks` opening with `\label` sits on a nameless creator (94a r9 residual P5; 94b r4 and on)

- **A `\thanks` opening with `\label` sits on a nameless creator (94a r9 residual P5; 94b r4 and on)**: llncs's `\author{Ann Able\thanks{\protect\label{fn1}Funded.}\inst{1} \and Bob Baker\inst{2}}` keeps "Funded." on a creator of no name after the two authors, not on Ann Able: inside a frontmatter item `\label` is `\lx@set@frontmatter@label` (Base_Utility.pool.ltxml:330, :565), so the note is an annotation labelled `LABEL:fn1` that no entry requests.
- Perl drops the text with a Warning (KNOWN_PERL_ERRORS #1204).
- Review rev94a r8 pl/p02.
- RED catalog repro `llncs_thanks_label_on_a_nameless_creator`
- State: OPEN

### A `\footnotemark` after a name's mark invents a superscript (94a r9 residual P6)

- **A `\footnotemark` after a name's mark invents a superscript (94a r9 residual P6)**: a class printing `\affil[2]{…}` in place, `BOB BAKER\textsuperscript{2}\footnotemark \affil[2]{Univ B}` with a later `\footnotetext`: the name holds the footnote then `<sup>∗</sup>`, the 2 lost and a ∗ nothing prints;
- unlabelled `\affil{Univ B}` `<sup>2,∗</sup>`, `\footnotemark[3]` `<sup>‡</sup>` (review rev94a r8 x02, x13, x14).
- RED catalog repro `raw_class_affil_footnotemark_after_a_mark_invents_a_sup`
- State: OPEN

### Marks a document macro prints, or picture keys, are not split (94a r9 residual P7; content kept)

- **Marks a document macro prints, or picture keys, are not split (94a r9 residual P7; content kept)**: `\newcommand\sa[1]{\textsuperscript{#1}}` and `\author{ANN ABLE\sa{1} \affil[1]{Univ A} BOB BAKER\sa{2} \affil[2]{Univ B}}` (a class printing `\affil[k]` in place) stay one creator holding both lines;
- the names boxed the same;
- a tikz circled key `\affil[\ctwo]{…}` (review rev94a r8 y01, y05, x10).
- RED catalog repro `raw_class_affil_marks_by_macro_not_split`
- State: OPEN

### A mark formula rewritten into a superscript keeps its italic digits (94a r9 residual P8; since 93a)

- **A mark formula rewritten into a superscript keeps its italic digits (94a r9 residual P8; since 93a)**: `Ann Able$^{1,\dagger\dagger}$` → `<sup><text font="italic">1</text></sup>`, pdflatex's digits upright (math digits are upright).
- Pinned as the known residual by the rv11 and rv15 rows of `perfect_kernel_batch94a::a_legend_of_several_symbols_shows_its_mark` and the rows of `a_repeated_symbol_beside_a_legend_is_its_own_mark` (update them with the fix);
- no catalog repro of its own, since those guard rows pin it
- State: OPEN

### just.cls binding (91a; cpc-hepnp.cls under the PhiPsi15 name) — the residual

- **just.cls binding (91a; cpc-hepnp.cls under the PhiPsi15 name) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): 91c open: the cpc-hepnp binding's house mark convention (an item ending in `)` a footnote mark, `;` a separator; the `;n)` residual, 341 papers; RED `class-bindings/just_semicolon_email_marks`), then a structural fail-safe for an unmarked multi-line address under marked names and the 611-paper A/B.
- State: OPEN (residual)

### nature-pre / nature_mod binding: the class's own list environments (92f) — the residual

- **nature-pre / nature_mod binding: the class's own list environments (92f) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: a paragraph a relaxed setter prints after `\maketitle` (article.cls:199/:233 let `\date`, `\author` `\relax`) between the list and the abstract keeps the abstract above the list (2608.12563, 2102.01782, 2111.09589).
- Also 2412.11966's author line (the same shape).
- Pre-existing (rev92f): 1408.2522's copy prints "Received …;
- Accepted …" from its own `\@maketitle` (nature_mod.cls:59-63), lost;
- `\author{…$^{1,2 \ast}$}` under article or TL nature keeps only `<sup>1</sup>`.
- State: OPEN (residual)

### A body `\ref` to an `\institute` entry's `\label` prints nothing (found 2026-10-30)

- **A body `\ref` to an `\institute` entry's `\label` prints nothing (found 2026-10-30)**: aa's `\institute{Univ X\label{ia} \and Univ Y\label{ib}}` keys `\inst{\ref{ib}}` correctly (the observer's `ref:` keys, OD #913), but a body "Univ Y is institute `\ref{ib}`." gives `<ref labelref="LABEL:ib"/>` with no target — our HTML an empty `ltx_missing_label` span, Perl's the red key "LABEL:ib";
- pdflatex prints 2 (the label is the institute counter's, which is why the same `\ref` inside `\inst` prints the number).
- The affiliation contact carries no label or refnum.
- Content loss shared with Perl;
- reach to be sized (body refs to institutes).
- RED repro `tools/perfect_kernel/repros/class-bindings/aa_body_ref_to_institute_label.tex`.
- State: OPEN

### frontmatter notes read as the class prints them (92e, 92g) — the residual

- **frontmatter notes read as the class prints them (92e, 92g) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): the version-dependent shapes left are elsarticle's `\journal` (copies up to v3.3 print "Preprint submitted to X", 2609.40131; TL 2025's v3.4c none) and elsart's ("Article published in …" when `\volume` is set); aa's `\mail` went to 97b.
- The e-mail inside an aa/svjour `\offprints` note ("O. Kochukhov, e-mail: …", 0903.3512) sat on the last author named before it while placing it by the note's name waited for aa/llncs as written families: that landed (int123a; `written: true` in `aa_cls.rs`, `llncs_cls.rs`, `sv_support_sty.rs`), and the reading places a note's address by the chain (`perfect_kernel_batch92a::a_publication_note_email_is_placed_by_the_chain`, a reading assert); the written XML on 0903.3512 is not re-checked (DOC1c, docs only).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "frontmatter notes read as the class prints them (92e, 92g) —".

### An unmarked address after a name list goes to the last name only (iopart, informs4; pre-existing, Perl's default)

- **An unmarked address after a name list goes to the last name only (iopart, informs4; pre-existing, Perl's default)**: `\author{A, B and C}\address{X}` (iopart) and `\AUTHOR{A, B}\AFF{X}` (informs4) give X to C (B) alone, where the print sets X under all of them.
- For iopart this is Perl's: `\lx@add@affiliations` (Base_Utility.pool.ltxml:742) attaches to the last creator by default.
- Candidate: the same default 91a's just binding opts into (`annotate=all`/`run`: an unmarked address belongs to the run of names it follows), declared per binding;
- found by 91a r2's sibling audit (the other `\lx@add@affiliation@marked` callers already annotate or reach it only through a marked branch)
- State: DEFERRED to frontmatter refactor stage 6/7

### Frontmatter correctness scorer (2026-10-27)

- **Frontmatter correctness scorer (2026-10-27)**: our creators against the paper's own print (pdflatex TL 2025, pages 1-2, sources and outputs only — no arXiv API, user 2026-10-27), on 3,000 random arXiv papers (`~/data/pk_agents/main/fm_scorer/`: `score2.py`, `conv2.sh <id> <bin> <outroot>`, `view.py`; seed 20261027). int111d, 2,578 scored papers: 84.3 % clean; name defects 4.3 % (several names in one creator 40 names, e-mail/ORCID in a name 29 papers, affiliation as a name 18, missing author 18, IEEE membership in a name 5, `\author` but no creator 4); a hand-typeset title page with no `\author` and no creators 6.9 % (178; all 8 sampled print their authors; Perl the same — a beyond-Perl capability, needs a ruling); block-level contacts only 4.5 % (may be FIDELITY_CHARTER §4's \"else the block\", e.g. llncs's one shared `\institute`).
- Validated: missing-author flags 17/19 papers right, simulated recall 95 % (name vanished) / 91 % (name in a contact), 40 random clean papers clean by hand; per-class flag samples all real.
- Structured classes 93-99 % clean, `article` 74 %.
- Worklist by papers: names parted by space alone (84a, OD #897, LANDED as int112c: gates 7,336/7,336, inits 0, dumps unchanged; A/B popint110u 14,994 papers, 53 changed, E 23,650 → 23,642 (16 better), F 64 → 64, +4,225 words, none worse — 1111.1264's 20 → 28 is its memory abort, int111d alone 17/26 across runs; nets 136/136; 26 witnesses and the 14 html_feedback "front matter" missing-author issues reachable locally or by download all right, #7192 a submission with no id); eptcs (6/6 sample papers: `\institute`/`\email` inside `\author` folded into the name; 84b, OD #898, LANDED as int113b: gates 7,340/7,340, nets 136/136; the 6 witnesses 1407.7928, 1408.5969, 1909.07676, 2109.08295, 2406.16409, 2409.15014 right, census A/B of all 73 eptcs papers 18 changed and all better, E 1,247 → 1,247, F 1 → 1, none worse; open: mark-labelled institute/e-mail blocks, 2304.02524) and lmcs (2/2: `\lmcsorcid` in the name; 84c, OD #899, LANDED as int114a (gates 7,343/7,343, nets 136/136): lettered and written-letter addresses linked to their authors, ORCIDs as contacts, `\email` lists split by owner, `\lmcsheading` dates; witnesses 2306.08266, 2402.09138, 1103.0510, cs0701140, 1202.6352, 2001.03894, 2012.13129, 2305.14448, 2305.19985, 2402.11987 right, 2108.06944, 1703.06021, 1607.01886, 1607.04128 unchanged; A/B popint110u + all 40 census lmcs papers, 15,034 papers: 45 changed — every one an LMCS paper, all better — E 24,166 → 24,163, F 63 → 64 the load timeout of 1908.03764, clean in isolation on both binaries); llncs's one `\institute` (20/56 sample papers: a nameless creator under the names, no author affiliated; 85b, OD #901, KPE #1195): the sole institute (no `\and`) is every author's unless the class's own `\inst{n}` picks it (hand-typed `$^1$` and paper mark macros such as `\printfnsymbol{1}` pick nothing: 2607.02596, 1804.05870, 2007.07423, 2102.12877, 2403.11865), shaped in the flush's queue (`share_row_contacts`); its `\email` calls, picked or shared, are lines under the row, each decided on its own (the address the author it spells, else the in-order one, else the one marked corresponding author owning no other, else the block; one address in printed braces read, 2410.04091; a call the reader cannot part the block's, 2206.10050; no e-mail ever inherits `row:shared`; a line holding an address outside the calls — `{\tt \{…\}@…}`, `\texttt`, `\url`, `\href{mailto:…}`, a `\urldef`'d macro — read by the address reader, each address its owner's, an unread one the block's as written: 19 census papers r2 had put in every author's affiliation, 0712.2737, 1103.2027, 1307.8067, 1405.0417, 1408.0384, 1503.02007, 1504.01099, 1511.00725, 1705.06516, 1706.02124, 1708.06860, 1805.11291, 1809.11127, 2103.08389, 2207.07506, 2407.18381, 2408.03209, 2411.00887, 2504.06600; a call's own label with it, `Corresponding author:`, `(Corresponding author)`, 2410.06985, 2609.00838, 2608.24697, 2509.05139; a template address no one's); an `\institute` inside `\author` taken out of the last name and winning over an outer one; 20 witnesses right (1002.2793, 1307.8067, 1405.1167, 1509.00818, 1609.01220, 1809.11127, 1910.11599, 2011.00616, 2101.06781, 2106.04289, 2205.06740, 2207.04574, 2207.11464, 2306.03360, 2405.11276, 2502.08785, 2511.12796, 2603.26486, 2604.20229, and #159's 2402.19043); 2207.06953 (two institutes, hand-typed `$^1$` marks) left the block's; census A/B of all 690 llncs papers + witnesses (716 converted, r3, every creator's name and contacts against int114a): 217 changed, 181 more authors affiliated, 162 nameless creators gone, 154 with e-mail ownership changed, no author given an address or label not theirs nor losing one that is (each address to its owner, or in order by charter §4; r5 on the `\inst{1}`-picked path, the marks deciding the contact's authors: a picked line the reader cannot part, or a unit no one owns, stays its picked authors', int114a's placement — 2109.05583's `{firstname.lastname}@…`, 2307.04136's `xfy_73.buaa.edu.cn`, 2609.24668's `[…]rouen.fr` — while an address of a group written out that no one spells is the block's, decision 3 of 2026-10-22: 2510.12629's `kgwo0528@`; r5 creators otherwise equal r3's), E 1,104 → 1,103, F 3 → 3 (1409.0098's W 3 → 4 a log-line split); the eptcs census population (73 papers, `row_address_owners` shared) byte-identical; open (RED repros `class-bindings/llncs_hand_marks_with_several_institutes`, `llncs_author_line_brace_list_email`, `llncs_marked_block_ambiguous_spelling_is_no_place`): several institutes with hand-typed name marks (2207.06953), an author-line `\email` brace list split inside its braces, or glued to the last name (1609.01220), the general author-block reader's `addresses_in_order` (`run_in_author_order`, defs/frontmatter_api/email_lines.rs since Z18j) placing an address several names spell by its place (`lee@` under Ann Lee, Bob Lee, Cid Cole goes to Cid Cole; `row_address_owners` has the fix) — a later batch with its own A/B, the reader being every class's; the r5 re-review's picked-path residuals (OD #901: a "Corresponding author" note on an unmarked author taking an unspelled picked address, RED `llncs_corresponding_note_does_not_override_inst_marks`; a shared line's word read as the e-mail of the note beside it, RED `llncs_shared_note_line_word_is_no_email`; one unspelled address alone in a call under several picked authors stays every picked author's — charter §4 says the block, awaiting a ruling); rescored 2026-10-28 on int114a (`fm_scorer/run_int114a/`): papers with a name defect 110 → 84 (4.3 → 3.3 %), clean 84.3 → 85.3 %, merged names 35 → 17 papers, contact in a name 33 → 28, affiliation as a name 18 → 15, missing authors 18 (unchanged: the next shape); rescored 2026-10-28 on int115a (85a + 85b, `fm_scorer/run_int115a/`): papers with a name defect 84 → 73 (2.8 %; nameless creators 125 → 106, missing authors found elsewhere 24 → 17, none new); the two tiers (`score_tiers.py`, validated: R1 precision 75/80 and 20/20 on the final rules, R2 12/12, simulated recall 97-99 %): readability violations 112 of 2,416 scored papers on int114a and int115a alike (19 printed words lost — root-caused, 11 mechanisms, `agent_reports/2026-10-28_rc_lost27.md`; 86a, OD #902, KPE #1196: imsart's `{keyword}`/`\kwd` blocks keep every entry and their MSC/AMS classification under its printed heading — the HTML's `<h6>` shows the scheme — OmniBus's `\kwd` accumulates, an amsart document's own stored `\subjclass`/`\keywords` reach the frontmatter; witnesses 1410.4462, 2407.13970, 2605.27137, 1401.3408, 2004.06211, 1009.1929, kms 1109.3766, 2207.08173; A/B 567 papers vs 85a, 84 changed, all in keywords/classification, E 1,850 → 1,849, none worse; 86b, OD #904, KPE #1198: bindings for the PoS template's pos.sty (`\author*` the speaker with the copy's "Speaker" note, labelled so only a starred author takes it, labelled affiliations, `\emailAdd`, `\onbehalf`, `\FullConference`) and rmaa.cls (`\altaffilmark`/`\altaffiltext` as the AAS markup, an unmarked one the block's; `\affil` in an `\author`; the Spanish abstract before the English; the keywords; `\shorttitle` the toc title); witnesses 2409.20413, 2512.14592, 2602.22379, 0903.4819, astro-ph0208329, 1301.7157, astro-ph0105072; census A/B of all 31 pos.sty and 8 rmaa papers vs int116a, 38 changed, all better, E 52 → 52, F 1 → 1, plus 345 `\altaffiltext` papers of OmniBus/aguplus/AASTeX byte-identical; 86d, OD #906, KPE #534 extended: the `\maketitle` replay gate no longer runs what `\ifx`/`\ifdefined`/`\string`/`\meaning` only compare or show (zaa.cls:166 `\ifx\@dedication\undefined`: 1902.07328's addresses and MSC line kept; cms-tdr 2307.11186, 2507.13469 still refused at an `\ifthenelse` dead branch; `\noexpand`'s operand and a non-`\csname` `\expandafter\ifx` comparand are still checked, refused by the gate), a replayed body's call of the kernel `\maketitle` makes no second frontmatter, and IEEEtran's binding takes out a full stop closing the block right after a grade (2405.17600's grade; 2405.10825, 2112.14209, 2409.11755, 2510.15773's last-line authors, which had become every author's affiliation), only where no note changes hands by it, by the shared reader's own mark tests (`author_mark_position` on the names, `leads_with_mark` on the notes): fully unmarked, the notes after the grade stay the document's (2112.14209's 6, 2409.11755's 1, 2510.15773's 1 + 3 `<pubnote>`s, as on the base; 86d's first round had given them to the last author by position, as Perl does and as both still do with no stop, P1, awaiting a ruling); a label on the names (`$^{1}$`, not a symbol such as `$^{*}$`) and a note led by one, read by their marks as on the base (2405.17600, a `\let` alias of `\textsuperscript` too); any other block as the base reads it (marks on the names alone, or symbol-only name marks over a numbered note: a correspondence note of a non-last author, RED `author_ieee_correspondence_note_of_a_non_last_author`, the observer's S2c); A/B of `86d-rel` (the gates' build) vs 501e8ba4f1, 9,329 papers (3,634 shipping a `.cls` that tests `\ifx…\undefined` in a title-page file, 1,800 IEEE `\IEEEmembership`, 544 whose `\author` closes with a stop, 3,351 raw-class): 8 changed, none worse, E 8,592 → 8,592, F 25 → 25 — better creators on 2405.17600, 2405.10825, 2112.14209, 2409.11755, 2510.15773 and 1902.07328's addresses kept, 2605.22853's unmarked notes after a closing stop the document's (the base's last author's), 2406.18171 copernicus's manuscript `\baselinestretch` 1.4 after its replayed title (as pdflatex); no other paper changed (r86d10, r86d11); earlier rounds' noise, rerun alone: 2210.01917 (timeout Fatal both sides), 1309.5800, 1203.4558 (round 2), 1101.3742 (round 1's A/B); residuals: zaa's orphan "Abstract." line (RED `class_title_page/maketitle_replay_label_of_refused_abstract_store`) and its two empty title/author slot paragraphs, a `{\em …}` grade before the stop (1810.08035), a mark between the grade and the stop (RED `author_ieee_membership_mark_before_closing_stop`); 81 pieces out of print order, most set below the abstract though printed above it (86g, OD #903: an abstract given after `\maketitle`, or without `\maketitle` after in-place setters, follows the title-page text printed before it — the class's own title page by provenance, cover lines that are no body prose — wrapped as a `ltx_titlepage_continued` `<titlepage>` whose HTML keeps the title; a float, TOC, body prose or body list keeps the abstract's place; an abstract queued before `\maketitle`'s flush (not `_late`) stays ahead of it, the rest of the abstract's run follows it, save the fail-safe: a run holding an entry from a raw class's store its title page may print (`_titlepage`, above the cover, below it or at the page foot) is not moved, int116a's order (residual: such a class's cover lines stay after the abstract, 86g review probes b01, b03, b04, f1, f2, f3); a deposit's `\insert` material (braced or `\bgroup`) is set after the title page as its own vertical list, and the title page's speculative runs read it as printed (a store there is the title page's, its label taken out with it); witnesses 2011.04405, 2102.01904, 2109.06762, 2203.09121, 2208.05119, 2309.13734, 2412.11664, 2304.00893, 0905.1102, 1404.2263, 1609.06335, 1908.08044, 1302.5352, 1702.07128, 2309.14687, 1207.1161, 1303.3004, astro-ph0210637 right; open: 2403.15998, ifacconf's `{frontmatter}` box holding the cover, the abstract's marker and the keywords); 12 owners wrong on certain evidence), the strict count (moving across the abstract counts as out of order; 70 if forgiven); archival 94.45 → 94.59 % of 5,838 structurally decided creators exactly right (llncs 74.4 → 79.0 %), its failures an address of another author copied with a numbered institute (114: aa 70, llncs 33 — `\inst{n}` with several institutes, 1811.05243, 2001.09649, 0901.4618), affiliation missing (138: one 55-author `just` paper 1603.03230 — 54 of its 55 right since 91a, OD #912 —, article 28) and unmatched creators (44); rulings decided 2026-10-28 by the user's difficulty rule (easy, pragmatic in scope; high-difficulty recovery deferred with witnesses, `parity/DEFERRED_FAMILIES.md`): hand-typeset title pages DEFERRED, llncs's one shared `\institute` on every author IN SCOPE (85b), OPEN_ROWS FM-24/FM-6/FM-31/KRN-28 IN SCOPE; exit bar ACCEPTED by the user 2026-10-28, two tiers — **readability, every paper**: every printed frontmatter piece is in the XML (nothing lost) and none sits on the wrong person or outside the frontmatter (nothing shuffled), a piece whose owner is uncertain staying at the block as written; 0 genuine violations on the 3,000-paper sample (narrowed by the user 2026-10-31 to **0 papers with lost words**; pieces out of print order and wrong owners are tracked as residual rows and no longer gate done) and the html_feedback frontmatter catalogue, every flag triaged — a family deferred with its witnesses in `parity/DEFERRED_FAMILIES.md` is excluded from the bar (user 2026-10-29), its witnesses kept for later work; **archival**: each creator and contact records how it was decided (provenance, as a schema attribute — user 2026-10-29), and at least 98 % of structurally decided creators are exactly right (lowered from 99.5 % by the user 2026-10-31, at diminishing returns; measured by the fail-closed scorer of 95a r2) — per creator AND per paper (the mean of each paper's share right), each read with a 95 % interval from resampling whole papers, on the v3 holdout (user 2026-11-01; row "Holdout10k v3"), name and ownership, inferred ones carrying no promise; path: 85a/85b → the scorer's two tiers (R1 printed words kept, R2 ownership on certain evidence only, archival precision per class) → readability violations first → provenance → per-class gates to 98 %; a data-driven labelling study for hand-formatted blocks is proposed, not built (`archive/AUTHOR_BLOCK_LABELING_DESIGN_2026-10-28.md`, superseded by the one engine); e-mail/ORCID/affiliation inside a name; the article-class second author swallowed into the first's affiliation (9 of the 18 missing authors; 85a, OD #900: after a name's lines, a line that reads as a name and nothing else starts the next author when a gap before it (a spacer line or a positive `\\[len]`), the names' styling, or an institution under a name with a person's cue (an initial, a particle) says so — `opens_next_author`; a stacked run of names keeps its lines, and after a gap no line under rows of names opens an author (2604.18394's "CUHK MMLab"); a row of names on an opened line is parted; the JASA template's `Name\hspace{.2cm}\\` block is parted by `name_groups` with the names line; witnesses 1702.01413, 0801.1297, 1608.08479, 2201.06821, 1509.03383, 1709.07791, 2408.11742, gr-qc9906006, 1701.05964, 2004.03971, 2503.11250, 1504.05438 right, 1908.10448 right but for Zach Shahn, 1702.07694 as int114a; census A/B of 400 `article` papers whose `\author` holds 3+ line breaks and no marks: 12 changed, all better, E 1,261 → 1,261, W 8,316 → 8,316, F 2 → 2; the 27 papers with an unmarked `\hspace` before the first `\\`: 16 changed vs int114a, all better; scorer subset of the 233 sample papers with such blocks: article papers flagged 10 → 4, missing authors 8 → 1; r4, the union of the reviewers' populations (1,071 papers: the 254 whose first `\\` has an option, 53 with later `\\[len]` gaps, 320 with plain `\\ \\` gaps, the 27 `\hspace` papers, the census 400, the scorer 233, the witnesses): 59 changed vs int114a, none worse, every creator compared with its contacts in order, E 1,720 → 1,720, W 18,165 → 18,164 (1603.00681's time-bounded ambiguous-math warning), F 5 → 5; reviewer probe shapes: one guard per distinct path (`perfect_kernel_batch85a`, 8), near-duplicates as unguarded catalog repros, the unhandled ones as RED repros (OD #900's residuals); open: a cue-less name over an institution with no gap or styling stays a line — no text tells `Zach Shahn \\ IBM Research,` (1908.10448) from `Max Planck \\ Institute for …` (also 0710.4320's `Meijun Zhu`, 2305.13188's three later authors; RED `article_plain_name_line_between_affiliations`); an eponym over an institution reads as a person (`L.D. Landau \\ Institute …`, RED `author_block_eponym_or_place_over_institution`); a cued name under a name-shaped place or over a head-noun institution stays a line (RED `author_block_next_name_under_a_place_or_whole_institution`); a wider gap among uniform ones parts no authors, nor does a cue-less name set with the same gaps as a place (`\\[3pt] Inst \\[3pt] Cat Cole \\ Inst`, RED `author_block_uniform_gaps_hide_a_wider_gap`); an organisation after a gap with a line under it opens an author (`\\ \\ Google DeepMind \\ London, UK`, RED `author_block_organisation_after_a_gap`, no corpus witness); a block opening with a `\ ` line read it as an empty names line and appended its lines after the next creator's own (solv-int9710025; GREEN since 98b, OD #949: a line before the block's first that prints nothing is no names line); a name line with a note or address after a wide space stays a line (RED `author_block_name_line_with_trailing_text`, 1908.06437's Håvard Rue after a line ending in "and" is a sibling); other shapes: `Name, Inst, e-mail` on each line, 2002.00268; a name and its institution on the line after a names list, 1409.4942 (JAC2003); adassconf's typesetting `\author` called twice keeps the last, astro-ph0101447; names stacked under names, with no line between, stay lines of the first as Perl reads it, 1702.07694's Peter I. Frazier and Shane G. Henderson, deferred, RED `author_block_stacked_names_each_a_creator`).
- Scope: names only, not ownership of affiliations/e-mails.
- Design evidence for segmenting on rehearsed boxes (user question 2026-10-27; the token level stays per the 2026-10-25 ruling until then): 84a's `space_parted.rs` re-derives by spelling what digestion knows — glue widths (`wide_length` reads `1em` as 10pt), raised marks, `{\quad}` as glue, `\(…\)` as math, `\rlap,` as a zero-width comma (hep-ph0411288) — and an author's own spacing macro is a spelling it cannot anticipate; a speculative digestion of each queued call into boxes, segmented on the boxes and cut back on the boxes' source tokens (the observer's rehearsal, #367's title-page run), would read them all; the repro guards (84a: 10) are the safety net for that change.; 86i (OD #154): a second abstract pdflatex prints where it stands is kept, as the vendored Perl keeps it — the replacement of a same-named `{abstract}` (upstream's `%ReplaceableFrontmatterTags`, removed upstream in #2767) dropped amsart's English abstract with its keywords and MSC (1006.5288) and an elsarticle paper's only abstract for an empty later one (2404.04417); titles, dates and keywords still replace (bindings re-add a keyword list as it grows); an earlier abstract repeated with the same text is dropped, the later kept; `\abstract{…}` before `\maketitle`, queued in Perl, is dequeued by the next abstract of its name, language and store until a flush (sn-jnl's template above the paper's, 2305.00493; Perl's dequeue ignores the name and runs after `\maketitle` too, losing printed abstracts the base kept); each later late abstract follows the cover lines printed before it, each late marker linked to the abstract its own command added, and late abstracts keep their source order — a move carries the later ones waiting after it, so one whose own move is refused (prose, a float, a list, a TOC before its marker) or a markerless one stays right after the one before, checked after every move — a violation is a placement bug, which a debug build fails on and a release build reports (`Warning:unexpected:late_abstract_order`) and repairs (`place_later_abstract`, `LATE_MARK_ORDINAL`, `take_late_mark`, `keep_late_abstracts_in_source_order`, OD #903); residuals in `parity/DEFERRED_FAMILIES.md` "Abstracts a class stores or queues"; A/B of 3,278 papers (the 2026-10-29 two-abstract census sets, every smfart/cedram/ceurart paper, 3,000 of popint110u, the witnesses) against int117a: 7 changed, all frontmatter, +1,188 words, E 1,721 → 1,721, F 4 → 4, none worse — 4 recover printed abstracts (1006.5288, 2404.04417, 0908.4597, quant-ph0404078), 3 keep an abstract pdflatex does not print, as Perl does (2406.04383 ceurart's `\verbatimwrite` abstract; 2108.09753, 2412.03249 elsarticle's `\absbox`, the class storing the last: open, a per-binding "stores its abstract" convention); guards `perfect_kernel_batch86i` (5), `cluster_frontmatter_replaceable_dedup` flipped to two abstracts | \n | — | **FM S2b third family: aas_support** (OD #837): the per-family gate as for ams.
- Census int110p (the 688 aas papers of the S2a.2 r9 census, `fms_aas/`): 634 aas_support papers, 30 unclassified (4.7 %), 81 classified, 2 Fatals.
- Largest unclassified buckets: an `\altaffiltext{n}` no name requests (13 papers) — many are the title's note (`\title{…\altaffilmark{1}}`, 0801.1670, 0808.0471, 1006.1591, 1010.0340: "Based on observations …"), which today's XML keeps in `<title>` and the reading sends to the block: the reading needs the title as an owner of a labelled entry, and the comparator and writer with it, before anything is written; labelled contacts by `\altaffilmark` (9 papers; 0707.2093, 0804.2539, 0911.4695 to read against their PDFs).
- Fixed: 82k — an API entry its call labels, placed by the names' marks, is ruling 1 (`ruling1:request`; 0707.2093's `Woods\altaffilmark{5,6}`, today's gives the last author); 82l — a prefixed mark label keeps its prefix on each piece (`altaffil:*`/`altaffil:**`, 0911.4695's swapped corresponding-author notes); census int110s: 22 unclassified (3.5 %); 82m — the ledger records the title and an entry only the title's marks request is the title's note (`api:title-note owner:title`; the comparator counts a note in the title as the document's); census int110u: 8 → (comparator 82n) 7 unclassified (1.1 %); 82o (a request of a separator is a mark, astro-ph0601584), 82p (emulateapj's title marks are the number given, 1111.4983), 82r (R12: an unmarked line under a run of names rows is every name's, 1511.01102), 82s (an address beside an institution keeps the institution, astro-ph0202139), comparator 82q (2003.01448, 1903.08299, 0910.4381) — census rerun on the 82s build next, then the seeded precision sample; design row (entry identity, owner kinds, group-level labels) in OPEN_ROWS; 82t (a footnote text only the title's mark shows is the title's note, 1111.4983), 82u (AASTeX 6.1's one-argument `\collaboration`, 1802.09023's lost author; KPE #1193), 82v (a `\correspondingauthor` naming several authors names each), 82w (every written family's observer guard checks the XML via the writer's outcome; the plan skips empty items; `{\,\!}^{…}` is a mark), 82x (an e-mail placed by position goes to the author it spells, FIDELITY_CHARTER §4 — 92 of 688 aas papers; today's and the reading alike). Census int111b: 2 unclassified (0.3 %: 1501.05343 collaboration placement — decided 2026-10-28 by the difficulty rule, the label stays at the block — and astro-ph0202139 FM-24 — in scope, a hand-written `1.` is the line's label); precision sample seed 110: 30/34 on int110w, the 4 wrong (multi-name correspondents) agree since 82v — gate met pending those two rulings
- State: OPEN (rulings)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Frontmatter correctness scorer (2026-10-27)".

### dev giant-list census (2026-11-01) — the residual

- **dev giant-list census (2026-11-01) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): BESIII `(A)`/`(C)` sub-labels deferred (R6, the difficulty rule);
- M3 PoS's one `\@author` (1711.04132, R9, settled by Z14);
- scorer limits L1-L4 (1201.5677, 1904.03257, 2510.23732).
- Also L4 math in a source name (2310.11912) and L5 a colon heading kept in the source name (2206.00684);
- aa's `X Collaboration\thanks{…}: R.~Abuter\inst{8}` keeps the collaboration an author (2206.00684, 97f optional).
- State: OPEN (residual)

### Holdout10k v3 (2026-11-01; spent: any further read is a fresh holdout v4, read once at the end — the plan's rule, user 2026-11-06; the rules below stay the judge's)

- **Holdout10k v3 (2026-11-01; spent: any further read is a fresh holdout v4, read once at the end — the plan's rule, user 2026-11-06; the rules below stay the judge's)**: **Papers with an Error or Fatal in their frontmatter are excluded from every count (user 2026-11-01: "completely exclude papers with errors in frontmatter from this work, they confound the results. Errors elsewhere are OK to keep")** — `score_tiers.py` `frontmatter_error`, status `fm_error`: an Error/Fatal located in the main file before the first sectioning command, in a class or package the preamble loads, or in a file input before that point, or an `ltx:ERROR` inside a frontmatter element; an unlocated error (`at Anonymous String`) is kept (test `test_2026_11_01_errors_in_the_frontmatter_exclude_the_paper`); the report gives one "Excluded" line and the numbers stay combined.
- `fm_scorer/draw2/holdout10k.txt` from `draw3.py` (seed 20261102) — 10,000 papers stratified by (in fleet run 336's error list or not) × submission year at the universe's shares (6.14 % run-336 error papers, universe 6.15 %), drawn among unseen papers only: every id of the first draw's exclusions, the 3,000 sample, dev20k, devh10k and `seen.txt` (222,086 ids any session converted, read or named — pk_agents, pk_wt, the job scratch, memory and the repo; fleet-scale error dumps nobody read paper by paper left out, the first draw's own exclusions having been 26.8 % run-336 error papers); `reserve3.txt` (2,000, same strata) is `replace.py`'s pool.
- The bar (row above): 98 % per creator AND per paper, each with its paper-resampled 95 % interval (`score_tiers.py` `paper_bootstrap`, report lines "Per creator" / "Per paper" / "Bar"; test `test_2026_11_01_d3_reports_per_paper_and_a_paper_resampled_interval`), and every residual lost word counted by a `Warning:frontmatter:unclaimed` (user 2026-11-04, replacing "0 papers with lost words"; FIDELITY_CHARTER §4 amended).
- Measured at milestones only, headline numbers only; no shape of it picks work.
- Dev = dev20k + devh10k.
- Writers' A/Bs: the class-stratified `ab_66bundle/popint125a_s10k.tsv` (10,574 of 28,439 papers, every class; `scripts/stratify_pop.py`) plus the touched binding's papers; the full population with `fm_checks.py` once per bundle **Dev re-measure (2026-11-04, pushed `ccd3b7067e`, binary int129a; holdout still sealed):** dev20k per creator **97.98 %** (38,187 of 38,974; paper-resampled 97.64-98.31 %), per paper **97.33 %** (7,865 papers; 97.00-97.63 %), **68** lost-words papers (int126b: 97.22 / 96.67 / 81); long tail per creator **91.78 %** (2,567 of 2,797; 89.17-93.96 %), per paper **89.99 %** (567 papers; 87.69-92.16 %), **67** lost-words papers (int126b: 88.34 / 86.98 / 86); runs `fm_scorer/run_dev20k_int129a`, `run_lt_int129a` (score2.py then score_tiers.py, then `--report`).
- **First measure (2026-11-01, pushed `80fe12a9d1`, binary int126b; headline only):** per creator **96.54 %** (18,750 of 19,421; paper-resampled 95.63-97.29 %), per paper **96.48 %** (3,938 papers; 95.92-96.98 %), **46 papers with lost words**, readability violations 155 (46 lost + 80 out of print order + 29 R2), 75 papers excluded for errors in their frontmatter; giants (6 papers over 200 creators) hold 1.2 % of the wrong.
- Against the bar (98 % both, 0 lost-words papers): 1.46 / 1.52 points and 46 papers short.
- The tuned 3k sample reads 98.53 % on the same code — a 2-point dev→holdout gap (the overspecialization signal); dev20k (96.08 % on int125b) is the honest steering set.
- **Switch rule (user 2026-11-01):** v3 stays sealed until dev (dev20k + devh10k + the long-tail dev set) reaches 98 % on both measures; then v3 is measured, and if short it is spent (becomes dev) and a fresh v4 (10k, same stratification) is the final judge.
- Holdout reports may show bucket totals by class frequency in dev (50+ / 5-49 / 1-4-paper classes), never papers or class names.
- Long-tail evidence (dev int125b): 1-4-paper classes 63.97 % per creator, 5-49 83.48 %, 50+ 95.96 % — 6 % of papers hold 25.6 % of the wrong creators.
- **Dev baseline on the same binary (int126b):** dev20k per creator 97.14 % (37,861 of 38,974), per paper 96.56 % (7,865 papers), 100 lost-words papers, 123 excluded — per paper within 0.1 point of the holdout (dev20k is the honest steering set; the 3k sample is over-tuned).
- **Long-tail dev set** (`draw2/longtail3k.txt`, `draw_lt.py` seed 20261103: 3,000 papers of classes with <50 dev papers, 837 classes, 483 unseen in dev; pdf texts 2,590 ok) on int126b: per creator 88.34 % (2,471 of 2,797), per paper 86.98 % (567 papers), **86 lost-words papers** (6× dev20k's rate), 22 excluded — the holdout's rare buckets (5-49: 92.17 %, 1-4/unseen: 85.24 % per creator) behave alike; only general mechanisms reach it.
- **Baselines restated on the current scorer (2026-11-01, same int126b outputs; the "before" of the next bundle):** dev20k per creator **97.22 %** (37,892 of 38,974), per paper **96.67 %**, 81 lost-words papers; long-tail 88.34 % / 86.98 % (D3 unchanged), 86 lost-words papers.
- Three scorer source-reading limits were fixed, each checked against its PDF (`score_tiers.py`; runs `fm_scorer/run_{dev20k,lt}_int126b_s5`): (1) an authblk key written twice holds both entries (`\affil[1,3*]{…}` then `\affil[1]{priyadarshi@…}`; 9 dev20k papers to 0 wrong: 2212.11454, 2006.03117, 2301.13111, 2010.16159, 2503.07893, …), guarded by test `test_2026_11_01_authblk_key_written_twice`; (2) after an e-mail label a spaced local-part list is one address group (`e-mail address: digiaco, paffuti@…`, hep-lat9906024), guarded by test `test_2026_11_01_labelled_local_part_list`; (3) a revtex `\thanks` contact naming its holder among others is a joint note, its addresses the note's (`Corresponding authors. a@x, b@x` under each, 2501.08373, 1407.5407, 1803.05275; one spelling only another creator still counts against its holder, 1209.2676), guarded by test `test_2026_11_01_thanks_contact_is_a_note`.
- The rest of census127 §4 stays as scorer limits.
- State: OPEN (the exit bar's judge)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Holdout10k v3 (2026-11-01; spent: any further read is a fres".

### a marked author list merged by a name the split test does not know (97a) — the residual

- **a marked author list merged by a name the split test does not know (97a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a `NON_NAME_WORDS` word in a surname keeps a mark-covered list whole (1212.4143);
- reversed marked names `Surname, I.\altaffilmark{1}` (1903.10014);
- a `(B)`-led institute makes a nameless extra creator.
- State: OPEN (residual)

### dev20k `aff_missing` census (int125b, 2026-11-01; 451 creators / 146 papers: 390 / 114 classed, 61 / 32 long tail, ≈43 scorer limits)

- **dev20k `aff_missing` census (int125b, 2026-11-01; 451 creators / 146 papers: 390 / 114 classed, 61 / 32 long tail, ≈43 scorer limits)** — **F1** (110 / 29) the family's writer gate is shut while the reading is right (inst_support 69 / 14 — the names census's M3; the jheppub family jheppub/jcappub/jinstpub 30 / 9; undeclared policies 11 / 6: aas_support, spie, emulateapj, PoS; `inst_support_sty.rs:11`, `jheppub_sty.rs:11`, `latexml_contrib/src/jcappub_sty.rs:14`, `jinstpub_sty.rs:18`, `policy.rs:122-127`, gate `writer/mod.rs:45-71`; witnesses 2302.00492, 2303.05792, 1607.07434, 1509.06963, 0807.0164; fix: each family through the OD #837 S2b gate, then `written: true`); **R1** (58 / 16) a raw class's in-place affiliation setter with no line break is never read (iopjournal `\affil` = `{\fontsize{8}{10}\selectfont\raggedright #1}`, 28 / 9: 2607.12158, 2609.08534; `frontmatter_stores.rs:1972-2010` `in_place_contact_setter`, `sets_a_line` :1989; caosp/iau/ETHpaper stores HARD); **W1** (40 / 12) → 97c; **W2** (31 / 8, MED) the writer declines "names differ" silently where today's output makes address lines persons and the reading is right (2410.09484, 2507.05173, 2105.11228, 1009.4816, 2512.19748; `emitter/plan.rs:1421`, `repart.rs:242-400`, `apply` :1375 returns false without a reason line); **L1** (24 / 6) → 97c; **SV1** (19 / 6, RUST-ONLY) → 97c; **J1** (11 / 3) contrib jfm makes `\affiliation` a document note (`latexml_contrib/src/jfm_cls.rs:302-305`; 2211.03175, physics0607157, 1411.5202; full fix needs F1's inst_support gate); **W3** (10 / 5) one unexplained contact difference declines the whole paper — ORCID keyed `httpsorcidorg0000…` vs `0000…` (`plan::contact_key` :256 not using `dom::orcid_id` :674; 2503.10804, 1903.03238, 2512.20289; EASY) and url/e-mail owner changes (2608.26140, 2407.19670, 2501.11284: per-item decline is a design question); **T1** (9 / 3, HARD) marked affiliations inside an IEEE `\thanks` (1903.05481, 1410.6355 — one `\thanks` holding `\newline` lines; 2411.05212 FIXED 100c: a line of marked `\thanks` after `\`, OD #957); **U1** (8 / 2) an unmarked affiliation over a `\\` becomes one contact per line (2509.16245, 1304.1996; `blocks.rs:752` marked only); **D1** (6 / 2) raa `\institute{… {\small Received …}}` (2102.01352, 2110.11117).
- Scorer limits: SL-mid (17 / 6) `find_marks` backs a `\textsuperscript` mark over its space so `$^1$A \textsuperscript{2}B` is one source affiliation (`score_tiers.py:~1300-1311`, :1742, :1751; 2303.05807) — FIXED 2026-11-01 (spaces are the mark's only inside its math; test `test_2026_11_01_sl_mid_*`): 4 dev papers right (2303.05807 3 → 0, 2308.12032 6 → 0, 2504.01503, 2505.22016), and two whose int125b output the old reading hid now score honestly wrong (2109.02066 lost `Institute of Computing Technology, Beijing` and entry 2; 2501.09538's Takamura no affiliation); dev20k 96.06 → 96.08 %, devh10k 93.05 → 93.03 %; SL-sym (12 / 3) symbol and parenthesised marks ignored (cond-mat0611096, 2601.21839, 2404.11930); SL-mail (9 / 2) an e-mail line inside `\affiliation` read as affiliation text; SL-legend (llncs legend folded into the institute).
- Per-paper data: job tmp `devaff/`
- State: open (worklist; 97c takes W1, L1, SV1)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev20k `aff_missing` census (int125b, 2026-11-01; 451 creato".

### dev20k `name:unmatched` census (int125b, 2026-11-01; 284 creators / 103 papers: 198 / 44 ours, 86 / 59 scorer limits)

- **dev20k `name:unmatched` census (int125b, 2026-11-01; 284 creators / 103 papers: 198 / 44 ours, 86 / 59 scorer limits)** — by creators: **M1** (61 / 10, EASY) a whole-block `tabular`/`minipage`/`\parbox` author block with e-mail lines stays ONE creator, mostly silent: the re-part cannot write an e-mail item (`observer/emitter/repart.rs:183-215` `item_source`, declines :449/:522; fix `Made::Email` at `plan.rs:46` + an `apply.rs` arm; witnesses 2605.09461, 2507.07465, 2609.36484, 2603.10436, 2608.20401; repro `\author{\begin{tabular}{c}Ann Able$^{1}$ \quad Bob Baker$^{2}$\\ $^{1}$Univ A \quad $^{2}$Univ B\\ \texttt{ann.able@a.org} \quad \texttt{bob.baker@b.org}\end{tabular}}`; an address no name spells needs a ruling: a nameless creator) — FIXED 100c for the addresses a name spells (OD #958, `Made::Email`: 2507.07465, 2603.10436, 2608.20401, 2609.39183); an address no name spells (3B: 2605.09461, 2609.36484) still declines the re-part, row "100c"; **M4** (30 / 5) a collaboration or team piece in the names line (1708.04166, 1711.07097, 1311.7305, 1905.11825, hep-ph0502212; also 2212.06489's 581-name BESIII line, `\center BESIII Collaboration\\~\\`) — FIXED 97d (OD #946, row "97d"); **M5** (23 / 6) six writer declines on boxed or grid blocks (2107.12423 `\resizebox` size read as name ink `0pt11304960` — FIXED 100c, `\Gscale@@box` keyed, OD #958; 2603.24245 `{\color{red}*}` legend; 2003.04668, 2101.00440, 1708.03035, 2402.14804); **M3** (21 / 6) the inst_support family unwritten (egpubl, aprim10: 1906.07409, 2503.03953, 1802.10123; `inst_support_sty.rs:11` `written: false`; the OD #837 S2b gate); **M9** (21 / 6) no creator (SolarPhysics `{opening}`, memsait, renamed ICML copies routed raw — a routing ruling); **M7** (15 / 5) one per-author `\author` holding several names (authblk `\quad`, acmart `\hspace`, fairmeta silent: 2412.03072, 1901.08289, 2510.01161); **M2** (9 / 1) a `\thanks` after a layout box (2604.15336); **M8** (9 / 2) reversed `Surname, I.` split at the comma (1409.4999, 2501.09580; the re-part only refines, `repart.rs:317`); **M10** sdl `\names{}` (1905.00862); **M6** (3 / 1, EASY) acmart `\bfseries\large` before the names breaks the `\\` continuation (2608.14026; `acmart_cls.rs:246-253`, `names_list_continues`); **M11** `\textsc{\char13}` prints ﬂ (2003.10147, cmcsc10 '013 is a quote).
- Scorer limits: S1 duplicate source `\author`s (supplements, 32 / 9: 2003.12024), S4 names the source parts merged by the scorer (28 / 27), S3 extra text in the source name (14 / 11), S2 name text (Math, 8-bit encodings; 12 / 12).
- S1 and S4 FIXED in the scorer 2026-11-01 (`score_tiers.py` `once_each`: a later source author spelling an earlier one's whole name is that author again unless our output keeps two; `split_names` + `WIDE_SPACE`/`_name_shaped`: a wide space parts names that each read as a name, as the converter's 84a does — such an `\author` is several names, so the paper's ownership is no longer read as structured; tests `test_2026_11_01_s1_*`, `test_2026_11_01_s4_*`, 73/73): dev20k on int125b 95.95 → 96.06 % (37,647 of 39,190; 8 S1 papers fewer wrong — 2003.12024 12 → 0, 2206.03879 11 → 1 — and 15 S4 papers out of the structured count, 0 papers worse); devh10k 92.96 → 93.05 %.
- S3 FIXED for collaboration remarks the same day (`name_of` + `COLLAB_REMARK`: a trailing `for the X Collaboration` / `on behalf of …` / `(The X Collaboration)` is no part of the source name — the 2026-11-01 collaboration ruling; test `test_2026_11_01_s3_*`): int125b's welded names now score wrong, as the ruling says (1407.6541, 2601.19431, 1212.4278, 2111.04743; 97d fixes them).
- Side finding: renamed ICML copies print their layout check ("marginparsep has been altered…") because article leaves `\marginparsep`/`\marginparpush`/`\topmargin` at 0 (`latex_constructs/sect05.rs:896-910` = Perl; TeX sets them in size10.clo:95-168; 2502.01411, 1903.03536, 2110.08350).
- Per-paper classes: job tmp `devnames/classes.tsv`
- State: open (worklist)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev20k `name:unmatched` census (int125b, 2026-11-01; 284 cre".

### dev lost-words census (int126a-pre, 2026-11-01; dev20k + devh10k, 171 flagged papers: 101 real, 69 scorer artefacts, 1 deferred)

- **dev lost-words census (int126a-pre, 2026-11-01; dev20k + devh10k, 171 flagged papers: 101 real, 69 scorer artefacts, 1 deferred)** — every paper loses exactly the same words on int125b, 97b-rel and int126a-pre (97c-97f neither fixed nor broke one).
- Real, by mechanism: **A** (69, HARD as a family, RUST-ONLY in most) a raw class's, package's or document's own stores that only its title-page code prints, refused or replayed with the fields emptied — copernicus/authblk-code classes (`\AB@affillist` never read: 0712.1312, 1401.4706, 1709.00722, 2310.11002 — its JOSS sub-family, a copy of authblk's own `\affil` over the binding's (2310.11002, 2504.08081; oasics 1709.00722), FIXED 102a by the binding's lock, OD #959; the kernel `\author` overwrites copernicus.cls:1349's `\@author`, the replay empties it `sect05.rs:1431-1435`), raa (abstract and keywords lost whole: 1404.3644, 1304.5354 — `\RAAabstract` store, deposit dropped for 3 held errors `sect05.rs:1390-1399`), sig-alternate `\titlenote` (`\tntoks`, printed only by the class's `\abstract`: 1206.6940, cs0405006), ~30 more classes with their own setters (aaskaii `\emailAdd`, cccconf, cup-journal, ametsoc `\authors`, ptapap, memsait, putex); → the 2026-11-01 ruling: unclaimed printed text at the block as a note with a counted Warning (FIDELITY_CHARTER §4), plus API routing for the large families; **B** (8, EASY) `coalesce_empty_creators` drops a block opening with a spacer line (`~\\`), silently (`insert.rs:233-283`, `:382`; 0812.2273, hep-lat0105010; Perl keeps the lines); **C** (9) `\IEEEmembership` under authblk (1803.04687) and `\IEEEpeerreviewmaketitle` (1909.01874); **D** (4) `\thanks` inside `\inst{}` (2303.08174, 1808.09829); **E** (3) svjour3 text before `\at` (1705.01399) — B-E → batch 98b; **F** smaller (daj, quantum e-mails, 1605.04229 deferred as pathological, 2603.24729 `$^{\textrm{…}}$` lines, INTERSPEECH 2103.17142).
- Scorer fixed the same day, each from a red test (`test_2026_11_01_lw_*`, `…abstract*`, `…units_and_break_points*`; 84/84): line-break spaces `\\[-0.8ex]`, font switches `\fontfamily{ptm}`, SPIE `\supit`, nested superscript marks (a superscript holding a word stays text: 2603.24729), math letters as atoms with control words whole and `\text{}` kept, `\symbol{126}`, `\input{…}`, `\allowbreak{}`, a Math's `text` attribute (siunitx units spelled one `\mathrm` letter at a time: 1811.04931), an abstract inside `<titlepage>` (cernphprep: 1609.06062); and per the ruling the abstract (environment, raa `\abstract{}`, ptephy `\abst{}`) is a checked piece.
- On the 171 census papers: 43 of 69 artefacts cleared, 101 of 101 real kept.
- On the 3k sample (97f r2 run): lost-words papers 1 → 2 (2402.17260's `\Y4Sdecays`, a delimited `\def` the scorer does not expand — scorer limit), D3 unchanged 98.23 %.
- Artefacts left (26): glued/split kept text 8, accent splits 6, aastex7 hidden e-mails 3, label words 2, one-offs 7.
- Per-paper data: job tmp `devlost/verdicts.tsv` **Abstract-piece artefacts fixed (census127, 2026-11-01):** an abstract heading (`\subsubsection{Abstract}`, 1205.6867), `\import{dir}{file}` (2507.10897), acronym/glossary commands printing their expansion (`\acp{LLM}`, 2403.15886), display math `\[…\]` (2607.23302), an unknown command's separate arguments (aa `\ion{N}{ii}` → "N ii", 1606.01066); test `test_2026_11_01_abstract_piece_artefacts`. dev20k on int126b: lost-words papers 100 → 81, readability violations 323 → 274, D3 unchanged.
- State: open (98b; A needs the net)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev lost-words census (int126a-pre, 2026-11-01; dev20k + dev".

### the unclaimed-text net (int127a; FIDELITY_CHARTER §4) — the residual

- **the unclaimed-text net (int127a; FIDELITY_CHARTER §4) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): deferred cccconf 1702.07637 and the 1-2-word losses below the threshold;
- stores at their class default (tADR2e 1509.08973, cup-journal 2302.11911) no candidates, the placeholder ruling pending.
- Open: R, routing a class carrying authblk's code without the package through the authblk binding (copernicus 0712.1312, esapub2005 1401.4706: the net's notes today);
- N3, note lines split only at direct-child breaks (2607.26558);
- left: a `\email` the class sends to an unprinted `\@thanks` (1304.5354, astro-ph0609644, astro-ph0612749).
- State: OPEN (residual)

### 99b: a raw class's title-page affiliation or address (OD #955; int127a, 100a) — the residual

- **99b: a raw class's title-page affiliation or address (OD #955; int127a, 100a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED `run_in_lines_a_run_places_keep_their_order`; the "Deceased" ruling and rev99b2 F2 (its item (f), int127a's next round).
- Open (a): xiaomiev's `\affiliation[#1]{#2}` through `\addtolist` (2604.26839), ifacconf's dispatcher `\address` (1111.1311 + 25 hidden long-tail papers), a `tcolorbox` title page (alta 2609.39929), a document-defined `\address` (1804.02826).
- Open (b): a name showing a legend's symbol in its own superscript keeps it beside the note's mark (2602.03759; legend notes 2609.15125, 2604.08878, 2607.12158); Kind: an e-mail line inside an affiliation (2607.25849); (c) R8 order: a run-in `\email` precedes the affiliations (2412.06969, 2503.13373); (d) a displaced call's leading font switch reaches its first entry only; a class's own label printed outside the store stays in the body; sifrev's `\split@rg` numbering (1704.03333, base too); a `$^10$` typo labels no line (2608.19687).
- Open (100a): authblk's `\affil[$\dagger$]{Deceased}` an affiliation of its carrier; a displaced store with one line in doubt leaves every line the class's text (`displaced_in_doubt`); hep-ex0106079's notes.
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "99b: a raw class's title-page affiliation or address (OD #95".

### 100a: hand-printed letter notes and unmarked note lines after a marked list (99a's Cut)

- **100a: hand-printed letter notes and unmarked note lines after a marked list (99a's Cut)** — OPEN, RED. hep-ex0106079 (moriond.sty `\address{… $^{12}$ YerPhI, Yerevan, Armenia.\\ a) also at UCEH … g) on leave of absence of YerPhI …}`, moriond_web.tex:160-170): the Cut reads superscript marks only, so the `a)`–`g)` notes are the text of entry 12 and its five YerPhI authors each hold all seven, while Abreu (`$^{6,a}$`) and the other requesters hold none; sibling (review rev100a c4): an unmarked trailing `Both authors contributed equally.` joins the last entry's affiliation.
- Tentative ruling (main session 2026-11-02): a hand-printed `x)` some name's mark requests is that line's entry label, read by the Cut's entry reader (marks win); a trailing line no mark leads and no name requests continues the last entry only if it reads as an address part, a note (`classify_thanks`) goes once to the block.
- Deferred from 100a r2: only `a)` leads a line — `b)`…`g)` stand mid-line (source newlines), so the reading is of mid-line hand-printed letter labels, a new reader beside the Cut's line-leading one, in `defs/frontmatter_api/affiliations.rs` (moved from author_block/ by Z18j; batch 100b's file).
- RED repros `displaced_hand_lettered_notes_after_the_marked_entries_go_to_their_requesters`, `displaced_unmarked_trailing_note_line_is_the_blocks`.
- **rev100a2 residuals (SHIP; synthetic):** a displaced store with one unrequested symbol line (`$^\bigstar$Deceased`) keeps the whole store in the body, its requested lines losing their carriers (RED probe `rev100a/s/dsu08`; fix = per-entry doubt); the "opens with Deceased" exception reads `$^\dagger$Deceased Estates Unit, Univ C` as a note (probe `rev100a2/v/v5`; decide on the first clause, "Unit" an institution word); a requested ♮/♭ "Deceased" line is an affiliation while ★ is a note (`is_footnote_symbol_operand` narrower than `prints_a_mark`); OD #955 (f): a math `$^*$` gives `mark="*"` while the sup prints ∗, and the U+2217-only maps in `post.rs:836` and `lexer.rs:387` are unnamed there.
- State: OPEN (RED)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100a: hand-printed letter notes and unmarked note lines afte".

### 100c: two Link fixes the observer read right and its writer declined (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`)

- **100c: two Link fixes the observer read right and its writer declined (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`)** — FIXED: (6) a line of marked notes only after a marked block's names line and its `\\` (`\\ \thanks{$^{1}$…}`, also `\\[3.0ex]`, `\\*[len]`, a spacer `\\`) joins the names line, its notes the names' (`author_block/preprocess.rs` `notes_line_joins_names`, `notes.rs` `notes_only_line`; IEEEtran's `grade_only_line` now "one grade, then `notes_only_line`"; OD #957, KNOWN_PERL_ERRORS #1206 — Perl shares it; witnesses 2609.17106, 2411.05212, 2312.10807).
- An unmarked block's line of unmarked notes stays the document's (86d's reading; control repro `notes_only_line_after_unmarked_names`), and so do notes after an `Emails: …` line, whose list joined was no list (1911.03356; control `notes_only_line_after_an_emails_line_stays`).
- The `author_mark_position` / `has_author_marks` disagreement (`frontmatter/classify/marks.rs:56` / `frontmatter/fallback.rs:114`, a `^` inside `\thanks`) stays, cited at the preprocess.
- (3A) a boxed block's re-part writes an e-mail item, the reading's address (one spelled out of a brace list) in its row's fonts (`observer/writer/{plan,repart,apply}.rs` `Made::Email`, `plan::block_address_tokens`; a domain word the list prints once counts as printed); (3D) `\Gscale@@box` among the lexer's `KEYED_CONSTRUCTORS` (OD #958; witnesses 2609.39183, 2507.07465, 2107.12423, 2603.10436, 2608.20401).
- A fail-safe: the re-part declines a name the reading joined across an alignment's cells (`repart.rs` `joined_across_cells`; 1302.4168).
- Guards `perfect_kernel_batch100c` (6); updated `perfect_kernel_batch97b::marked_address_piece_is_no_person` (two creators and B's e-mail, 0 errors, where one creator + `merged_creators` was the fail-safe).
- A/B (`pop100c.tsv`, 1,199: witnesses + M1, the 39 popint125a papers with a notes-only line after `\\` in `\author`, the 153 with a boxed block, a 1k control; base int127a): 1,187 converted, 18 frontmatter changes, E 2,042 → 2,042, F 4 → 4, none worse; re-A/B of the changed papers and the dev reach on the final binary (`pop100c_r3.tsv`, 26): 24 changed, every one read, no creator merged or lost, no shared address grown (fm_checks).
- Scorer D3 on the dev20k + devh10k reach (203 papers, int127a → 100c3; readability violations 5 → 5): 73.15 → 80.23 % per creator (455 → 499 of 622), 59.87 → 71.44 % per paper; long-tail reach (20) 73.33 → 73.33 %.
- Review rev100c fixes (r2): (B1) the re-part declines a made item that joins two or more Name events of the block one of which is a whole row (a line, a cell, a spaced-apart piece — no comma's piece) that may on its own name a person (`repart.rs` `joined_rows`/`whole_row`; 2609.37429's `\FormatAuthor` pair, Zihan Hu inside Chiesa's affiliation — today's one creator again; RED repro `boxed_block_two_people_in_one_cell`); (S1) on a line of notes only, only the notes a name's mark requests join the names, the others are taken out of the block as document pubnotes (`preprocess.rs` `notes_the_names_request`, `\thanks` → `\lx@add@pubnote@thanks`), the resolver reads them the block's (`note:unrequested`, `resolver/blocks.rs`) and the writer holds such a block note where today's prints the pubnote (`plan::held_as_pubnote`): 2609.17106's funding note, 2312.10807's `$^{*}$`/`$^{\dagger}$` legends (no longer Alois Knoll's), a `$^{\dagger}$` in such a note no name's mark (review probe q11).
- Guards `perfect_kernel_batch100c::{a_note_no_name_requests_stays_the_documents, a_person_in_a_row_an_item_joins_is_never_a_contact}`.
- Re-A/B of round 1's changed papers + witnesses (`pop100c_r3.tsv`, 26, int127a → 100c10): E 6 → 6, F 0 → 0, no word loss; against round 1 only 2609.37429 (declined), 2609.17106 and 2312.10807 (the notes) and 2310.15990 (its unrequested `$^*$` funding line now stays a line, as int127a) differ.
- Regression fixed (101c): B1 also declined a marked entry whose first row may name a person (`$^{\#}$\,Hellebore Capital Management\\ 63 Avenue …`), so 1507.08137's four authors shipped as one creator + a `merged_creators` Error (int128b); the first row of an item a mark labels is exempt, the line the mark makes the entry (`repart.rs` `reads_as_a_person`; guard `perfect_kernel_batch101c::a_marked_entrys_first_row_that_reads_as_a_name_stays_the_entry`).
- Review rev101c: SHIP (the exemption is narrow: a marked first row that IS a person is declined earlier or never re-parted; 2609.37429's decline holds).
- RESIDUALS, pre-existing, base too: the re-part's written affiliations drop the block's `\itshape` (base keeps `<text font="italic">` where the re-part does not run); an e-mail row continuing a marked legend line becomes a creator with no name (RED `marked_legend_email_row_is_no_creator`, sibling of 3B); a marked "John Smith Laboratory" affiliation merges the names with a `merged_creators` Error (RED `marked_affiliation_named_laboratory_merges_names`).
- **Open:** (cells) the resolver's `joins_previous` (`resolver/lines.rs:668`, rule `name:layout-joined`) joins unmarked names in adjacent `tabular` columns into one name (`Ann Able & \mbox{\ \ } & Bob Baker`, 1302.4168; also the affiliations below them, `Univ A & Univ B`), so the block stays one creator (RED repro `boxed_block_names_in_two_columns`); a cell boundary should part names (EASY in the resolver; changes readings beyond the boxed blocks, so its own A/B).
- (3B, deferred) an e-mail no name spells in a boxed block is a block-level contact (FIDELITY_CHARTER §4, 2026-11-01), which the writer has no form for short of OD #159's nameless creator — the re-part declines ("a block e-mail no name spells"), today's one creator stays (2605.09461, 2609.36484; RED repro `boxed_block_emails_no_name_spells`); needs a block-level contact in the writer's `dom::Front` (only creators today) and the API.
- (exposed) the e-mail item lifts the re-part's last decline on boxed blocks, so readings that decline masked now ship (no content lost, the scorer better or equal on each): 2609.37429 (`\FormatAuthor{name}{mail}{inst}` nested `tabular`s side by side — the second author read as a continued line of the first's affiliation, "EPFL Zihan Hu zihan.hu@epfl.ch EPFL"; declined since r2 by `joined_rows`, the reading's continuation across two nested `tabular`s in one cell still open), 2603.19216 (planstyle's `\affil` after a boxed `\author`: the reading's `api:annotate:run` gives it to the last name only; 7 names now right, 6 without the shared affiliation), 1302.4168 (names joined across cells, now declined); proposed one fail-safe for the main session: a boxed block's re-part writes only when every Name event of the block that reads as a person on its own (`may_name_a_person`) is exactly one reading creator's name — never part of another name or of an item (it would subsume `joined_across_cells`).
- (Kind) `\IEEEauthorrefmark{n}` prints the n-th footnote symbol (∗, †, …; IEEEtran.cls:4528-4530 `\@fnsymbol`), but the reader keys it as `n`, so legend notes `\thanks{$^*$Equal contribution}` / `$^\dagger$Corresponding…` answer no request (2312.10807: document pubnotes since r2, no longer the last author's); the charter's printed-label rule wants the symbol key, which would make them the refmarked authors' legends.
- (S2) 2603.19216's shared `\affil` after a boxed `\author` goes to the last name only (the reading's `api:annotate:run`; siblings 1107.1546, 1107.1548; all 7 creators right, no content lost).
- (q10) an unmarked note alone on its line under a marked names line (`Ann Able$^{1}$, Bob Bell$^{1}$ \thanks{$^{1}$Univ A}\\ \thanks{Funded by X.}`) is the last name's, int127a the same; FIDELITY_CHARTER R1 wants the block's (RED repro `notes_only_line_unmarked_under_a_marked_names_line`).
- (S3) revtex `B.~in der Baker$^{2}$, b.baker@two.edu$^{*}$`: the address's `∗` is read as the name's request, so the name prints `<sup>2,∗</sup>` and the made e-mail no mark — the made e-mail item would need to carry its mark (resolver attribution + writer; RED repro `author_marked_email_piece_mark_moves_to_the_name`; `perfect_kernel_batch97b::marked_address_piece_is_no_person` pins today's).
- (held) the writer holds a block note as today's pubnote only for `note:unrequested`; a note on an address line (`note:address-line`, the fms2a2r6/r8 `ADDRESS_LIST`/`OWNERLESS` shapes, `\author{Ann Able and Bob Baker\\ Univ A, City\\ E-mails: x1@example.org, \\ bob.baker@example.org\thanks{Corresponding author.}}`) still declines; widened, those shapes are written right (e-mails each author's, no nameless creator) — its own A/B. FIXED 101b: the token reader reads those lines as address lines, their notes the document's pubnotes (`address_line_notes`), and `held_as_pubnote` holds `note:address-line` too; `ADDRESS_LIST` and `OWNERLESS_ONE` now `same`, row "101b".
- (6') a line after `\\` holding a note and text (`\\ \thanks{$^{1}$Univ A.} $^{2}$Univ B, City`) stays a line read by the mark inside its note, the note the document's (RED repro `notes_and_text_line_after_the_names`, synthetic).
- (3C) 2603.24245's legend welded over `\quad` into `\small{{{Code: \url{}}}}` (a template placeholder) — the Cut residuals.
- **rev100c2 residuals (SHIP; synthetic):** B1's person-row decline counts a `\quad` gap as a row edge, so `\quad`-separated place names ("San Diego", "New York", "Los Alamos") decline a reading round 1 had right (RED probes `rev100c2/probes/b6.tex`, `b3`; only line and cell edges should count); unrequested-note pubnotes are appended after the block's calls, out of print order (probe `m7`); OD #957 should say 2312.10807's `$^{*}$` is printed by the names via `\IEEEauthorrefmark`.
- State: open (3B, cells, exposed, Kind, S2, q10, S3, 6', 3C; held FIXED 101b)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100c: two Link fixes the observer read right and its writer ".

### Z14's REDs and follow-ups (revZ14r2, SHIP)

- **Z14's REDs and follow-ups (revZ14r2, SHIP)** — OPEN.
- (1) **An affiliation line's addresses come out but its label word stays**, glued across the `\\`: 2002.06945 reads `…Imperial College London, UKEmail` and declines (output byte-identical, correct); sibling `\IEEEauthorblockN{Ann Able, Bob Baker \IEEEauthorblockA{Univ A\\ Email: ann@a.org, bob@b.org}}` (pdflatex clean) is written at Z14, declined at Z14r2, equal to int143b.
- Fix: take the label word with the addresses, as `address_runs` does for a names row; repro `sectioning-frontmatter/z14_affiliation_label_word_left_red.tex`.
- (2) **The narrowed heading row needs a person cue on every piece** (`labels.rs` `later_line_is_names`): `Cat Cole$^1$\\ Authors: Ann Able and Bob Baker\\ $^1$Univ A` and `Editor: A. Able, Bob Baker` read one creator with the row as an affiliation, as int143b; repro `z14_heading_row_plain_names_red.tex`.
- (3) `given_back` in `ab_audit fmshadow` carries no paper id per row: add one, so that the papers it fires on (3 in popZ01, 6 each on dev3 and devh10k) can be read
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z14's REDs and follow-ups (revZ14r2, SHIP)".

### Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had their two fix rounds, so these go to the next reading batch)

- **Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had their two fix rounds, so these go to the next reading batch)** — OPEN.
- (1) **A surname glued to its address goes with it** (`resolver/lines.rs:477-486` `names_without_addresses`, this round): `\author{Ann Able(ann@a.org)}` reads as `Ann`; the writer declines, so the output is int143b's (Z14r2 wrote `Ann Able` + e-mail).
- Fix: cut where the address starts inside its word, stepping back only over glued punctuation (a bar, an em dash, an opening parenthesis), never letters; repro `sectioning-frontmatter/z14r3_address_glued_to_a_surname_red.tex`.
- (2) **Addresses glued by a dash or a comma** (earlier rounds; int143b the same): `Ann Able---ann@a.org` gives name `Ann`, e-mail `Able—ann@a.org` (`labels.rs` `plain_addresses` ~:1125), and the guard `an_address_glued_to_a_dash_leaves_the_name` pins the e-mail text with its dashes; `Ann Able,ann@a.org` makes a second creator `ann@a.org`; repros `plain_address_glued_by_a_dash_red.tex`, `plain_address_glued_by_a_comma_red.tex`.
- (3) **Lines with the same text under two marks are one entry** (entries keyed by text; the any-order join can't use one twice, the take-out declines): Ann's spelled address stays in Cat Cole's affiliation; `an_only_carriers_address_beside_others_is_not_written` pins int143b's fallback as green; repro `z14r3_same_line_text_under_two_marks_red.tex`, with the OPEN `entry_address_owners` row (2403.12766).
- Report `agent_reports/2026-11-07_revZ14r3_report.md`
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had thei".

### Z14: the reading's repairs before the flip (int144b) — the residual

- **Z14: the reading's repairs before the flip (int144b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left by creators on dev3: 2305.13191 -10 (flexisym's digits are `\mathchar`s), 1509.04564 -4, 0807.4720, 1404.5693;
- RED `a_mark_past_a_marked_lists_leading_run_is_an_event_red` (2411.03172, 2510.06608, 2602.11758);
- the role heading's REDs.
- Also left at -2 each: 1901.10843, 2311.16860 (`Alexander New`: a surname read as an institution word), 2409.13798, 2510.01302, cs0011034.
- Also 2003.02512: the reading has its four authors, the writer declines, today's merged creator and its honest Error kept.
- State: OPEN (residual)

### Z4a: the emitter behind its switch (int144b; OD #980) — the residual

- **Z4a: the emitter behind its switch (int144b; OD #980) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left for Z17: an address in an entry's run with no separator (astro-ph0610446, cond-mat0610708), a legend behind a printed label (2107.01943);
- a conditional a cut ends inside;
- a document note's own API calls (0807.4720).
- State: OPEN (residual)

### Z4b: `=2` reaches `=1`, the safety net (int145b; OD #980) — the residual

- **Z4b: `=2` reaches `=1`, the safety net (int145b; OD #980) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left before the flip: a block past the positions or the read budget scores as Perl's (dev3 -513 creators, devh10k -607 and -7).
- State: OPEN (residual)

### Z3: spans from the digested list, the rehearsal's isolation (int143b; OD #978) — the residual

- **Z3: spans from the digested list, the rehearsal's isolation (int143b; OD #978) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED `writer_second_digestion_says_an_error_twice_red`; `#[ignore]`d REDs of `perfect_kernel_batchz3`: `affiliation_cut_environments_by_spelling_red`, `affiliation_cut_group_siblings_stay_uncut_red`, `rehearsal_no_lua_loses_a_cut_red`.
- Open: the rehearsal has no read budget yet (`rehearse`, `emitter::plan::cut` to take `gullet::read_budget` when R1 lands); a file only a rehearsal reads stays in `RAW_READ_PATHS`; package-level thread-locals not read one by one; the marked-lines path keeps a printed mark on `$\rm ^{1}$` lines (`aastex_affil_font_switched_marks_cut_with_their_lines`, Z6's); an entry cut at a mark a plain group's switch opens loses it (`{\it $^3$Univ Z}`, base the same: `block_line_mark_in_its_wrapper_whatever_prints_it`).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z3: spans from the digested list, the rehearsal's isolation ".

### Z2: event gaps 3, 4, 5, 7, 10 (int143b) — the residual

- **Z2: event gaps 3, 4, 5, 7, 10 (int143b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs with no guard `ordinal_in_a_group_or_with_a_font_keeps_its_letters`, `ordinal_names_on_two_lines_are_two_names`, `event_a_par_by_another_name_is_a_par_break`, `event_declaration_a_kernel_macro_building_a_name`, `event_declaration_through_ifthenelse`;
- REDs with no code: a package's macro defined before `\AtBeginDocument` unread, an unbalanced Declaration span.
- RED, no guard: `ieee_author_after_maketitle_prints_nothing` (`\author{…}\maketitle\author{…}`: the PDF and Perl print the first, ours both).
- State: OPEN (residual)

### Z1: event gaps 1, 2, 6, 8, 9 — the residual

- **Z1: event gaps 1, 2, 6, 8, 9 — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): for Z17: most held papers are the observer's own errors (dev3 120 held errors in 32 papers; about 630 API calls in 353 papers);
- a block's AASTeX 7 `\email` still a printing Contact (gap 6);
- a raw class's font switch read as a declaration.
- The study's held-error papers (21 `Script _ can only appear in math mode`, the rehearsal digesting the API's private options in text mode): 2603.28168, 1405.3507, 2201.04449, 2309.13984.
- State: OPEN (residual)

### A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, as P1)

- **A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, as P1)** — `latexml_engine/src/l3fp_native/`: `\__fp_parse_operand:Nw` at the doors' shape answers with what its run leaves, from a port of l3fp's arithmetic, or runs as defined (a flag, a tuple, `nan`, `rand`, a conditional in the argument, a tab mark that ends a cell, any l3fp name changed since the install, an expression past 400 levels); the doors stay the format's macros. fp1000 -97.5 %; pgf-interference-en/-de 150 → 33.6 s and 151 → 33.5 s, -80 % instructions, XML byte-identical: **G4 closed**.
- Self-check 0 mismatches in 971,116 calls over the 346 manuals that reach l3fp (94.3 % native), which also agree base, native and raw on E, F, W and XML; pdflatex oracle 36 families, 0 mismatches; A/B 2,495 papers 0 changed.
- OD #977, `performance/PERFORMANCE.md` 2026-11-06 | landed on the branch | speed left: `\exp_args:*`, `\arabic`, `\box_wd:N`, `\pgfkeysvalueof`, conditionals, `\numexpr` in an expression; native only over expl3 2025-11-06 and 2026-01-19, other years keep the macros; a document's redefinition of the kernel's macros l3fp uses (`\exp_after:wN`) or of primitives is not watched
- State: [`performance/PERFORMANCE.md`](../performance/PERFORMANCE.md)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, ".

### l3kernel/source3: the XML varies with the streaming yield (found by P2's comparison, not caused by it)

- **l3kernel/source3: the XML varies with the streaming yield (found by P2's comparison, not caused by it)** — a big document's yields fall by RSS (`stomach.rs` soft-RSS branch), so 7, 8 or 9 segments are staged from run to run under load, and the output moves with them: a paragraph's closing newline (`types of functions.\n</p>`) and a `\,`'s `rpadding` (`1.5pt` or `1.7pt`, `1\,\text{in}`, l3fp.dtx:1570).
- Base P1 alone, 8 runs under load: 2 hashes (3 and 5);
- P2 8 runs: 1 of those 2;
- idle, one hash.
- Text and errors equal | open, not a regression | a yield must not change the output: what crosses the boundary (a font's size, a trailing space)
- State: [`perfect_kernel/MANUAL_REGRESSIONS.md`](MANUAL_REGRESSIONS.md)

### Interpreter hot paths, `pk/P1` (user 2026-11-06: "dramatic speedups for pgf-interference and other pgf work, spanning the entire latexml-oxide")

- **Interpreter hot paths, `pk/P1` (user 2026-11-06: "dramatic speedups for pgf-interference and other pgf work, spanning the entire latexml-oxide")** — byte-identical on the benchmarks and on 1,873 arXiv papers: fp1000 -22.1 %, fill1000 -19.1 %, addplot500 -22.9 %, loading pgf -24.5 %.
- pgf-interference-en/-de are still marginal: 171 s and 181.7 s on loaded SMT siblings, so the native l3fp evaluator (`pk/P2`) is what closes them.
- Each lever, its figures and the measured dead ends: `performance/PERFORMANCE.md` 2026-11-06 | levers in, gates pending | a native l3fp evaluator (`agent_reports/2026-11-06_brief_P2_l3fp.md`);
- `-Zplt=yes` to measure on idle cores
- State: [`performance/PERFORMANCE.md`](../performance/PERFORMANCE.md)

### R1: the title-page check's cost, the last regression of #151 (user 2026-11-06; OD #953 "R1"; `performance/PERFORMANCE.md` "the pk/P1 levers")

- **R1: the title-page check's cost, the last regression of #151 (user 2026-11-06; OD #953 "R1"; `performance/PERFORMANCE.md` "the pk/P1 levers")** — DONE: no run of the check's reads more than 8 M tokens (`gullet::read_budget`), what it leaves out named in a counted `Warning:frontmatter:check_budget`; uantwerpenexam-example2 322.7 G → 178.9 G (#149: 224.7 G) by the budget firing, its `\extrainfo` and title page unchecked, that Warning its one difference, the XML byte-identical.
- The third source digests every stored argument, as before R1: a skip when the print held its words lost a word uncounted (`perfect_kernel_batchr1::a_thanks_behind_a_macro_in_a_stored_argument_is_counted`).
- RED `check_store_argument_loop_times_out_red` green (`perfect_kernel_batchr1::a_looping_stored_argument_is_budgeted`).
- A/B `popZ01` vs P1: 1,873 papers, XML identical, every `frontmatter:` Warning and Info line identical.
- A failed reading context closes its mouths by depth, the mouth in reading counted (`reading_from_mouth`, `mouth_depth`; same-named mouths left the document's text to the next close).
- | DONE (R1, R1r2) | The rehearsal and the stores' sentinel runs have no budget yet (plan, "Before Z4"); SASnRdisplay's `Warning:unexpected:\end{document}`, a `\ifx` left open since between #149 and P1, not root-caused
- State: [`performance/PERFORMANCE.md`](../performance/PERFORMANCE.md)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "R1: the title-page check's cost, the last regression of #151".

### A second `\author` in article keeps the first call's contact

- **A second `\author` in article keeps the first call's contact** (Z2's review) — `\author{Ann$^1$ \\ $^1$Univ A\thanks{First note.}}\author{Bob}`: the PDF has Bob alone;
- ours Bob with the affiliation Univ A and the pubnote First note, on the base too.
- Synthetic;
- no arXiv witness.
- RED `sectioning-frontmatter/article_second_author_call_keeps_a_block_of_the_first_red`, no guard | RED | Before the frontmatter's single engine reads it (plan Z-series) or on a witness
- State: `tools/perfect_kernel/repros/sectioning-frontmatter/`

### Z19 list setters (DEL3, OD #982)

- **Z19 list setters (DEL3, OD #982)** — the S2b writer's last `=2` path closed: a block calling a binding's list setter (`\institute` of sv_support, inst_support, aa, llncs inside `\author`) is the emitter's — the setter's call written as it is after the items, its entries the list the names' marks request (`emitter/emit.rs` `list_setter_runs`, `without_list_setters`); the CL2b stay-out (`LIST_SETTER_IN_BLOCK`) removed.
- Population 2 of 73,251 (cond-mat0605195 byte-identical; 2502.07804 creators equal, ORCID as the `=2` `\href` item); the 1,434 stage2 ids unchanged; cost −0.04…+0.002 %.
- DEL3b (review revDEL3): the names keep the marks they print (2502.07804's `${}^{1}$`, B1), and the written call is the flush's last, replacing a list queued before the block or an earlier call of it (`replaced_by_setter`, `Ledger::replaced`; S1).
- Guards `perfect_kernel_batchdel3` (8), `perfect_kernel_batchcl2::a_list_setter_in_an_author_block_is_read_by_the_emitter` (renamed, re-expected: no stay-out Warning).
- **DEL4 (2026-11-09): done** — `write_observed_frontmatter`, `read_back_frontmatter` and their closure, `Policy::written` and the census (`LATEXML_FM_SHADOW`, `set_fm_shadow_override`, `<dest>.fmshadow.json`, `ab_audit fmshadow`) deleted (OD #837 "Deleted (DEL4)"): skipping the writer changed 0 of the 1,434 stage2 ids and 0 of 900 E0 papers, nothing ported; −6.4k engine lines, −4.8k `ab_audit`; cost −0.02…−0.34 % (cost.sh's four arXiv documents).
- Kept: the ledger's JSON `fm_reading_json` as the guards' view of the reading (≈15 guard modules read it); the core setter watch (`origin::watch_frontmatter_setter_calls`, `state::open_frontmatter_setter`) now has no caller turning it on (follow-up: delete in latexml_core)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z19 list setters (DEL3, OD #982)".

### Bundle int153a — THE FLIP: `LATEXML_FM_EMIT=2` (the single frontmatter engine) is the default; `=0` keeps the old driver until the deletion pass (FLIP1; user 2026-11-09 …

- **Bundle int153a — THE FLIP: `LATEXML_FM_EMIT=2` (the single frontmatter engine) is the default; `=0` keeps the old driver until the deletion pass (FLIP1; user 2026-11-09 "i accept the flip"). The no-harm pass (OD #981): NE1 across-cell decline, NE2/NE2b/NE2c/NE2d counter-moved and counter-stepping-mark declines, NE3 `&` only with digested cells, NE4 annotations from the call's tokens, NE5 `declaration_reads_past` + `conditional_cut` (the Fatals 2505.04436, 1511.09009), NE6 a mark's span takes its arguments + `reads_past` over optional/verbatim parameters (2603.11001), NE7 a copied picture keeps its SVG ids (Perl `appendClone`; tikz-shields-doc jing 8→0), NE8 an IJCAI `\emails` section ends the reading + symmetric `conditional_cut`, NE9 the IJCAI flag only for a block holding its markers + an address opening with a call ends the affiliation before it; CL6-CL10 the content no-harm fixes; PF9/PF9b the `=2` checks proportional to what they inspect; reviews revX4-revX7 SHIP; code `e537b16abf`)** — gates 8,311 of 8,311 (4 skipped), cortex 10 of 10, fmt, clippy and doc 0, dumps plain 0 / latex 0 errors.
- **A/B** 28,354, int152c (off) vs int153m (`=2`): 5,397 changed (frontmatter 4,254), E 46,080 → 45,800 (116 better, 1 worse: cs0607047 1→2, its `\def\\KL` erring at each `\\`, faithful), F 172 → 171 (none worse);
- 1710.01722 a timeout equal alone (base, `=2` and off all 180 s under load);
- the large word drops are deduplication (0 distinct words lost: hep-ex0107028, nucl-ex0002003, 1001.4730, 1204.5689, 2406.08304, 1811.08969, hep-ex0106079, 2109.09437);
- 2606.14105 (BESIII, 770 creators off) written whole since int153d ("unstamped: out of positions", counted).
- **The `=2` stage** (1,434): 4 creators changed vs int152c at `=2`;
- 0 newly erroring at `=2` vs off;
- the keep-better net's 6 "equal off" the same as int153d's (1307.5879 ruling, 1409.0847 deferred, the rest scorer-wrong);
- nextest off 8,311 of 8,311.
- **The 37 `=2`-only Error papers**: 0 still newly erroring (bundle dumps).
- **Cost** vs int152c: `=2` −1.10 % to +0.42 % (2103.04797), off −0.12 % to +0.36 %.
- **Nets** 136, 1 worse: ejpecp (3 of 30, the class's queued e-mail notes lost at `=2`, counted — RED).
- ejpecp: CLOSED in bundle int154a (CR5 on int154v: 0 words lost on all 7 ejpecp papers).
- **Sweep #167**: G1-G4 0 (G2 1→0 by NE7), G5 58.
- **Re-measure rd8** (int153m, `=2`, per creator / per paper): devh10k 98.60/98.05;
- dev3_10k 99.08/98.73;
- dev20k 98.64/98.32;
- longtail3k 94.21/93.75 vs off 93.05/91.81;
- full suite on the pushed tree (`631dfd6c86`, the 77p pin and docs after the chain's gate tree) 8,311 of 8,311.
- **flip200, judged once** (hand labels, `=2` vs off): MAIN per creator 81.64 % vs 75.72 %, per paper 69.21 vs 68.83 %, fully right 124 vs 122 of 200;
- STRICT 76.89 vs 71.05 %;
- extra creators 54 vs 60.
- **REDs at the flip** (SYNC "Before the flip" item 39, `FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`): the content residual (248 rows incl. 2406.07496's Correspondence line and the three ejpecp nets);
- NE5's decline writing six A/B blocks whole (0809.5152, 0906.2789, 1110.6375, 1404.5602, 2404.18592, 2503.21719);
- the IJCAI `@` cut;
- the textless TikZ drop;
- 77p's ruled grouping (Dan Dorn at UBC alone, user 2026-11-09).
- Since, in bundle int154a: the content residual re-triaged (CR1, CR5, CR9) and its real classes fixed or listed in "Bundle int154a — the residual"; NE5's six blocks split by CR7 except 0809.5152, 1110.6375, 0906.2789; the IJCAI `@` cut synthetic only (RED catalog); the textless TikZ drop closed (CR8); 77p's grouping still open.
- The push gate's cargo-deny met RUSTSEC-2026-0459 (bundled SQLite in `libsqlite3-sys` 0.38.2, published the day of the flip): `latexml_post` takes `rusqlite` 0.41.1 (`libsqlite3-sys` 0.39.1), full suite 8,311 of 8,311.
- Method notes: probes compare on the bundle's own dumps;
- a writer touching shared frontmatter code runs the whole guards binary.

### Bundle int139c (Bundle 5: 107a-107g) — the residual

- **Bundle int139c (Bundle 5: 107a-107g) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): scorer false losses left unfixed: 1808.00588, 2410.15681.
- State: OPEN (residual)

### Bundle 5's plan (landed as int139c) — the residual

- **Bundle 5's plan (landed as int139c) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): side RED `email_list_unspelled_entries_legend_becomes_creator_red`.
- State: OPEN (residual)

### 100b: one Kind for the addresses inside affiliation entries, one Link chain (OD #956; int129a) — the residual

- **100b: one Kind for the addresses inside affiliation entries, one Link chain (OD #956; int129a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): HARD, deferred: an address inside a line of other words (about 151 dev20k papers); 2009.04283 pathological (DEFERRED_FAMILIES); TENTATIVE ruling E and the only-carrier step (2403.17706, 2411.17073); RED an `@` in `\href`'s URL read as a printed font; REDs `address_family_text_symbol_marked_lines_pile_on_last` (cmp-lg9710008), `address_family_rlap_name_split_takes_spelled_address` (hep-ph9905377), `address_family_unspaced_list_one_letter_last_stays_whole`, a tie after a list comma read as `˜`; int129a's: an authblk entry of initials-labelled address lines takes out only its first (2505.20974).
- Its witnesses: the HARD shape 0704.2085, 1902.04228, 2111.01631, 2608.22602, 1805.00710 (`ee13d015, ramkrishna@…`); the `@` in `\href`'s URL `address_family_href_url_at_is_no_printed_font`; the tie `address_family_tie_in_email_list_prints_a_space`.
- RESIDUALs: names welded into an entry by an upstream misparse (2506.06908's `Jiawei Yan, …` under Chenzhi Xu; 2504.14657, jmlr's `\AND`: `bellabruin5711@` the block's); a one-address line with an ORCID glued after it is declined whole (2601.09524, 2308.02269 `inenaga@`, `nakashima.yuto.003@`); (S2) the own-call step runs in today's engine only (jmlr's `\AND` 1504.01446, 2509.23923); (N1) addresses left to the block whose author is identifiable (1912.12193 `xi@`, astro-ph9809079 ö read as oe, 2308.02269's `arim@`/`koba@`, 1410.0127's `bcz@`); 2501.13021's `author1@` after the misparsed creator's own two; 1611.10059 (elsarticle) loses typewriter on the address the `\ead` copy merges with.
- Also: the HARD shape's fail-safe witness 2401.08092 (the writer declines a re-part needing a block creator); (S2) compare 2002.00057; not read (ruled out): surname-first initials (2512.18755).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100b: one Kind for the addresses inside affiliation entries,".

### Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs 368 sandbox-arxiv-2605, 369 sandbox-arxiv-2606)

- **Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs 368 sandbox-arxiv-2605, 369 sandbox-arxiv-2606)** — error-free coverage did not regress: 2605 95.47 → 96.88 % (E 1,332 → 923, F 29 → 16), 2606 95.64 → 96.98 % (E 1,298 → 897, F 27 → 22), baselines runs 354/355 (2026-10-15, `1a95291ef6`) snapshotted first.
- 965 papers entered error-free, 137 left: 40 are the `frontmatter:merged_creators` self-check reporting pre-existing merges (the Oct-31 binary reports the same); the rest for triage — `latex:(hypdvips)` 14 (2605.04427, 2606.08053; FIXED in 101a, "Rust Error Fixes": hyperref's `\Hy@driver` follows the output mode), `\errmessage` 10 (2605.16389, 2606.14185), month words june/july/sept as `unexpected` ~9 (2605.00162, 2606.01474), `\algocf@capseparator` 3 (2605.26474, 2606.16211), wall-clock timeouts 9 (load: re-time alone first).
- Lists: job tmp `rerun2605/`, `rerun2606/` `clusters.txt`.
- The fleet was found dead (its `WORKER_BIN` deleted, harness unable to spawn workers) and was rebuilt (`build_fleet.sh` at `3e4819bfee`), its distro-tree dumps regenerated (backup `fleet_dumps_backup_2026-11-02`), and restarted.
- State: OPEN (triage)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs".

### Author blocks routed through user macros — example collection (user 2026-11-02: no approach decided; collect examples, data and tests first)

- **Author blocks routed through user macros — example collection (user 2026-11-02: no approach decided; collect examples, data and tests first)** — name formatter wrapping a nested `tabular`: 2609.37429 `\FormatAuthor` (two calls in one cell, printed side by side; 100c declines it to base);
- mark macros: 2304.13998, 2602.23952 `\affmark[..]`, `\sa{1}` (RED `raw_class_affil_marks_by_macro_not_split`);
- affiliation macros: nucl-th0303060, 2405.10528, 1307.5171 `\affiliation{\LPC}`.
- Add witnesses as they appear.
- State: COLLECTING

### 101b: an address line carrying a note (2605.23553) — the residual

- **101b: an address line carrying a note (2605.23553) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED: an address line's notes are appended after the block's calls (`dispatch.rs:1844`, `affiliations.rs:200`).
- Repro `address_line_note_order_after_a_later_note`.
- State: OPEN (residual)

### 102a: a copy of authblk's `\affil` no longer replaces the binding's (OD #959) — the residual

- **102a: a copy of authblk's `\affil` no longer replaces the binding's (OD #959) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open variant (iii), an ACL/NAACL `\email` built from authblk's `\affil` code (2406.11632, 1904.03969; RED `authblk_email_from_affil_code`), and variant (ii);
- RED a `\let`-alias wrapper loses its typography (`authblk_affil_copy/wrapper_keeps_its_font`).
- RESIDUAL (the binding's): 2604.08855's `\author[2\ensuremath\mathparagraph]{…}` asks for no `¶` label, so `\affil[$\mathparagraph$]{Corresponding author}` goes to a nameless creator (OD #159).
- State: OPEN (residual)

### 103b: name tails inside `personname` — the residual

- **103b: name tails inside `personname` — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED (S) `\samethanks`/`\footnotemark[n]` keeps a mark-only note;
- left: a remark inside the name's own group (2406.15355, 1206.2546), an iD printed as digits after a name's comma;
- RED an icon glued to a bare comma (OD #960 "Open").
- REDs: (S) `author_samethanks_mark_is_the_same_note`;
- the bare-comma icon `author_orcid_after_bare_comma_is_the_names`;
- the leading-icon test reading only the first name `author_orcid_leading_a_later_name_is_its_own`.
- State: OPEN (residual)

### 103b residual: a collaboration heading over an `\and` list covers only the first name

- **103b residual: a collaboration heading over an `\and` list covers only the first name** — `\author{GRAVITY+ Collaboration\thanks{…}: R.~Abuter\inst{8} \and …}` (aa, 2206.00684, the 98a witness), `Euclid Collaboration: M.~Walmsley\orcid{…}\thanks{…}\inst{…} \and …` (aa 2503.15310, aaEC 2507.12116): the binding split reads the heading as the remark of the first piece (`split_author_calls`, OD #946), so only the first name carries `contact role="collaboration"` (2206.00684: 1 of 115); nothing is lost (the heading's `\thanks` a pubnote).
- Charter §4: the collaboration is a contact on every name it heads.
- Minimal input: llncs `\author{XYZ Collaboration: Ann Able\inst{1} \and Bob Baker\inst{2} \and Cat Cole\inst{1}}` — RED repro class-bindings/`llncs_collaboration_heading_covers_every_name` (no guard).
- Options: carry a heading parted by its colon from the first name to every later `\and` piece of the same call (in scope of the split, a brief patch), or leave the first name's only.
- State: OPEN (review rev103b S3, record only)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "103b residual: a collaboration heading over an `\and` list c".

### 103c: a legend line read as an affiliation is its carriers' note (census129 cluster 6 / mechanism 4, Bundle 1c; OD #961)

- **103c: a legend line read as an affiliation is its carriers' note (census129 cluster 6 / mechanism 4, Bundle 1c; OD #961)** — FIXED: at the frontmatter flush, after placement, the Link step and the observer's writing (`insert::kind_legend_contacts`), a creator's affiliation contact whose whole text is legends — correspondence, a contribution, a death — names and addresses, no institution (`notes::entry_legend_kind`: pieces parted at marks, brackets, `,:;` and line breaks), is renamed in place to `<note role="thanks" class="ltx_note_frontmatter ltx_thanks_<kind>">`, carriers and e-mails unchanged, its `mark` one that prints — the printed superscript, else a label key a printed mark gave (`affiliation:K`, not an entry's place: `_seqlabel`), a carrier's name shows or requests, or its binding prints before the line (`_labelprints`: authblk, jheppub/jcappub, cas), the carrier's own among several; a legend that prints no mark stays the affiliation it was, whatever its route (a binding's key, icml's `\icmlaffiliation{intern}{…}`, 2106.09776; an entry's place, svjour's `\institute{… \and …}` `affiliation:2`, 1507.04774: review rev103c2 B-A) (an unmarked note is unreachable on the rendered page; review rev103c B1, main-session ruling 2026-11-04: 18 papers, 2412.14916, 2305.02090, 2004.11569, 2011.02402); a name in the legend must spell one of its carriers (a nameless block creator's, any author's: 2609.08556), so "Equal contribution, Google DeepMind" stays an affiliation (review rev103c S2); `classify_thanks` reads project roles (project lead, joint/co-first, core contribution, senior author) and an opening "Work done during/while/as" as contributions only where every piece of the text is a legend — one gate for every reader of the kind, so `$^{\ddagger}$Core Contributor Network, Paris` stays an affiliation on the line route and the observer too (review rev103c S1) (SYNC 97c (d) closed).
- Routes of the 87 dev20k papers: authblk `\affil[x]` 45 (43 now notes), author-block lines 10 (9), IEEE blockA 6 (6), mark-led `\thanks` 4 (3), icml 2 (0), llncs/svjour `\institute` 2 (2), others 18 (12).
- Witnesses now notes: 2608.22772, 2310.15190, 2412.14916, 2603.21906, 2305.02090, 2609.03342, 2609.18514, 2609.19719, 2409.05386, 2410.12405, 2504.19724, 2406.19953, 1611.06677; mixed lines stay affiliations (2609.34327, 1702.02684, 2505.10257, 2404.02132, 2512.00940, 2207.11193, 2303.06042).
- A/B 103c vs int130a (1,881 papers: the 87 + witnesses, 800 phrase-bearing and 1,000 control of int125a_s10k): 129 papers held a legend as an affiliation, 14 still do; 125 changed (114 legend → note only, 5 a `\thanks` note's kind class, 6 legends the observer now writes as notes, the name's mark moved onto the note); E 1,883 = 1,883, F 5 = 5, words +6, `fm_checks` 0 creators merged/lost, 0 e-mails moved, word types lost only the role label "Affiliation" (1206.2592).
- Re-A/B after review rev103c (the 125 changed + 2602.19190 vs int130a): 107 changed, 104 a marked note, 0 empty marks, E 11 = 11, F 0 = 0, `fm_checks` 0 merged/lost creators, 0 e-mails moved, 0 word types lost; vs r1 only the 18 unmarked legends (back to base) and two `\thanks{Work done during an internship at …}` CSS kinds (`ltx_thanks_note` again, 2110.15064, 2410.24211).
- Re-A/B after review rev103c2 (the same 126 papers): vs int130a 105 changed, E 11 = 11, F 0 = 0, words +6, `fm_checks` 1-7 clean; vs r2 3 changed — 2106.09776 and 1507.04774 back to base byte for byte, 2301.13111's second author's note marked "2" as her name prints it — every non-symbol mark checked against pdftotext (1610.05499, 1812.09922, 2301.13111, 2301.13661, 2303.06763, 2305.16688, 2311.06016, 2409.14212, physics0307102 as printed).
- Guards `perfect_kernel_batch103c` (four, `a_legend_whose_label_prints_no_mark_stays_an_affiliation` with repros `legend_kind_icml_keyed_legend_stays` and `legend_kind_svjour_institute_sequence_stays`, the multi-label repro `legend_kind_authblk_multilabel_legend_marks_each_carrier` in the first; controls `legend_kind_institutions_named_with_role_words_stay`, `legend_kind_symbol_led_institutions_stay`; `creator_notes` pins 92b's and 95b's notes), unit `classify_thanks_reads_project_roles_and_the_works_circumstances`; 11 guards updated to the marked note or the new kind, the unmarked legends of 61, 63, 77p and 77y as on base (OD #961).
- Residuals (one-line inputs): IEEEtran's `\IEEEauthorblockA{\IEEEauthorrefmark{1}Corresponding author}` was a note marked "1", the number the names showed, where IEEEtran prints `*` in both (2205.00125, 2410.12184 `§`, 2504.18719, 2507.23474, 2603.14786 `‡`) — fixed by 106b (OD #968: the binding prints the symbol, the note's mark is it); `$^{\ddagger}$Work done while at Google` on the line route keeps `ltx_thanks_note`, the CSS kind reader reading other text than `legend_note_kind` (review rev103c2 N1, class only); `\affil[*]{The authors wish it to be known that, in their opinion, the first two authors should be regarded as Joint First Authors.}` stays an affiliation (a comma parts the piece; 1703.07724); `\icmlaffiliation{x}{Work done before joining Google DeepMind}` (2603.23994), `\affil[2]{Corresponding Author: amir[dot]norouzi[at]lancium[dot]com}` (2605.00769), a legend beside a code link (2403.03314), a funding legend (1604.05657) stay affiliations; `\author[1]{A}\author[1]{B}\affil[1]{Univ}\affil[ ]{Corresponding author: A (a@x)}` stays B's affiliation, the run's last author (2412.14916; unmarked, and the Link question, not the Kind); a compound legend `$^{\dagger\ddagger}$Project lead` the observer writes gets `mark="†"` (`show_legend_mark`, 77p's shape), while the Kind step keeps a label's key whole (2602.19190's `$^{*\dag}$Project lead: Yi Yang`, `*†`); `\thanks{Work done during an internship at Microsoft Research Asia.}` is `ltx_thanks_note` (2110.15064, 2410.24211).
- Re-review rev103c3 SHIP.
- RED, synthetic: an authblk label that prints nothing visible (`\affil[~]`, `\affil[\phantom{*}]`) is written into a note's mark as its source text (`mark="˜"`) — `_labelprints` should read the label's printed text (repro sectioning-frontmatter/`legend_kind_invisible_authblk_label_prints_no_mark`; none of the A/B papers).
- Pre-existing, pinned as base: in the svjour guard's repro (`legend_kind_svjour_institute_sequence_stays`) the `\institute{… \and C. Cole (\email{…}) \at Org Inc.}` name "C. Cole" appears nowhere in the XML (pdftotext prints it) — a content loss of svjour's `\at` institute names, base too.
- Open for the user: IEEE legend notes take the number the names show ("1") where IEEEtran prints ∗/§/‡ (2205.00125, 2410.12184) — the binding's `\IEEEauthorrefmark` form, not this step.
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "103c: a legend line read as an affiliation is its carriers' ".

### 104b: a raw class's `\and` institute store (OD #791, #955) — the residual

- **104b: a raw class's `\and` institute store (OD #791, #955) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): OPEN, RED catalog repros, one paper each: comsis2 2207.09249, caosp 1401.6818 (`displaced_numbered_institutes_also_printed_after_the_bibliography`), a numbered store whose last entry carries the names' legend lines.
- RED `raw_class_numbered_institutes_with_legend_lines` (2207.09249).
- Explicit-mark stores, each its own shape, not `\and` numbering: 1105.3133 appolb, 2212.04765 ETHpaper, 2507.18198 tlp, 2605.21951 lumia, 2410.14846 and 2609.04385 openjournal, 2205.03755 bioinfo, 2209.11470 anstrans, 2512.21849 antgroup, 2608.19687 pas, bound 2602.03671 llncs and 2606.24132 aa; known and left 2609.39929 (alta), 2604.26839 (xiaomiev).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104b: a raw class's `\and` institute store (OD #791, #955) —".

### int131fix: the Bundle 1 A/B's regressions — the residual

- **int131fix: the Bundle 1 A/B's regressions — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: a recovered Fatal in a field leaves `place_keeper` (`\date{\deep}`), 2110.07418;
- RED `author_orcid_link_wrapped_comma_parts_no_names`.
- State: OPEN (residual)

### Token-stack RAII (int131fix follow-up)

- **Token-stack RAII (int131fix follow-up)** — `invoke_token` (stomach.rs) pushes `token_stack` and pops it on its normal exits only; the recovery arms now clear the stack (`stomach::clear_token_stack`), but an error caught below them (a speculative digestion that discards its `Err`, a binding's `catch`) still leaves its entries, and a later digestion can trip `MAXSTACK` early with a ghost `Stomach:Recursion`.
- Fix: an RAII pop guard beside `local_current_token_guard`, every exit of `invoke_token` popping once.
- Witnesses 2410.07647/2105.02599 (fixed by the clear); 2502.10606 (a speculative title-page run's discarded `Stomach:Recursion`, 106s at int135b) fixed by `stomach::StomachMark`, which the speculative runs now take and put back (row 106s) — a binding's own `catch` is still uncovered.
- Pre-existing, from review rev135c (synthetic): an `Err` inside `invoke_conditional` (a recursion while reading a test) leaks the localized `if_frames` entry (local_assignments.rs:68), so `open_conditional_count() == 0` (stomach.rs fragment yields under memory pressure) never holds again — `StomachMark` could truncate it too; speculative runs advance `if_count` toward `iflimit`; a false test opened in one rehearsed block and closed in the next takes the wrong branch (base the same).
- State: OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Token-stack RAII (int131fix follow-up)".

### `\string` of the null control sequence (int131fix follow-up)

- **`\string` of the null control sequence (int131fix follow-up)** — tex.web §263 prints `\csname\endcsname` (with `\escapechar`) for the null CS;
- ours prints `\` (`\edef\x{\expandafter\string\csname\endcsname}\show\x` gives `\x=macro:->\`).
- `define_optional_internal` names its internal as TeX's `\string` does by its own test (int131fix, OD #567);
- every other `\string`/`\meaning`/`\detokenize` of the null CS is still the divergence.
- State: OPEN

### 104a: IEEE address blocks — the residual

- **104a: IEEE address blocks — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `ieee_shared_block/email_line_marked_addresses`, `ieee_shared_block/explicit_empty_block`, `ieee_shared_block/mixed_grid_column_major`;
- not this row: 0907.0548, 2402.00243.
- State: OPEN (residual)

### 104c: marked runs the Cut left whole (OD #964) — the residual

- **104c: marked runs the Cut left whole (OD #964) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED `revtex_nucleus_marks_spaced_in_one_line_red`; open (rev104c3): a nucleus mark after "and", an isotope inside a line; deferred: an unmarked line before the marked ones (2205.14113).
- REDs: `revtex_isotope_mark_after_and_is_text_red`, `revtex_math_number_mark_before_st_floor_red` (catalog only), `svjour3_institute_unmarked_lead_lines_red` (2205.14113), `revtex_unmarked_first_affiliation_line_red` (1903.07834), `aa_institute_and_inside_a_macro_group_red` (2407.09799).
- Also open, readings before 104c (recorded, no guard): a tabular of marked entries then `\\ Joint Lab` gives "Joint Lab" to the last entry only; `$^1$ Univ A, Lab for ${}^{3}$He Physics` parted at the isotope; `$3$\textsuperscript{rd} floor` loses "rd".
- Outside the Cut's reading: llncs's `\mailsa` line glued to the last entry (1604.03221), a second author block (2006.15775), ieeeconf's `\:\:$^{2}$` (2110.11491), authblk's `\affil[\hspace{-1ex}]` label (2502.06575), a legend on an entry (2609.38081).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104c: marked runs the Cut left whole (OD #964) — the residua".

### 104d: link lists — the residual

- **104d: link lists — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): residual 2008.13733 (a correspondent of several names), L3's other witnesses (2607.00453); RED L4, a `|` between addresses (1507.04180), L5, ijcai `\emails` with a TeX group (`ijcai_emails_group_list_is_spelled`, 2403.12370), P1, aastex631's `\correspondingauthor` placement; OPEN P2, cas's `{augroup}` undefined.
- Also L3's 2607.12952 (`\email` under Gurwell's call, unprinted), 2609.27403 (`\email[show]{\parbox…}` swallowing the next lines); L5's 2311.12521; P1's RED `aastex631_correspondingauthor_after_authors_email_is_the_named_authors`.
- Also: `\correspondingauthor{Chengchao Yuan}` over `\author{Chengchao Yuan (袁成超)}` stays an orphan correspondent (the name's parenthetical, 2306.15659, 2511.16056).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104d: link lists — the residual".

### 105a: the net's lines below its floor (OD #953) — the residual

- **105a: the net's lines below its floor (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED: class-default placeholders no candidates (2302.11911, 1509.08973; the placeholder ruling); code the net does not walk (cccconf 2408.14758, 2206.02389, 2512.09276, 1702.07637 deferred; basm 1601.01922).
- REDs `net_class_default_placeholder_is_counted_red`, `net_non_english_month_line_red`; revtex's `\@author@finish` under the revtex binding prints `\alt@affil@toks` (aeb_emulateapj 1410.3797, openjournal 2604.08196).
- Each of (b)-(h) needs its own mechanism or binding: (b) a store whose name is computed or passed (cup-journal 2508.08363, 2607.18758, 2412.08333, 2503.22083, 2510.05654, 2510.18398; ptapap 1912.07241, 1612.02970; compositionality 1904.09322, 2603.03774; JUSTC 2409.17603; opendatalab 2601.09733, 2601.21821, 2605.10341); (d) a candidate the sentinel run does not print (2205.05046, 2412.08184, 2205.01391); (e) memsait's handed `\@opening` (astro-ph0102098, astro-ph9909366, astro-ph0005436); (f) printed only from the `.aux` (1206.2980, 2310.08896); (g) a K11 or kernel field losing part of its value (2304.08300, 2312.02510, 1909.06690, cond-mat9610136, cond-mat9812194, cond-mat9906132, 2004.13105, 2408.12585, 2410.15356); (h) outside raw-class stores (`\offprints{…\email{…}}` 1511.07638, 0805.4214, 1311.4988; econsocart 2203.00070; icrc29 astro-ph0507347; 2308.16621; single words of kept fields 1709.00722, 2409.01067, 2205.11450, 1908.07849, astro-ph9912176); a Chinese line whose ideographs all appear in the title or abstract goes with no Warning (rev105a2 nit, suspected).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "105a: the net's lines below its floor (OD #953) — the residu".

### 106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade lines — the residual

- **106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade lines — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a late revtex `\author{A}\affiliation{X}` after `\maketitle`; a refmark ≥ 10 among other-form marks loses its roman numeral (rev106b N2); a legend note's math `∗` beside `*` (N3); `\(…\)`/`\ensuremath` unseen by `printed_refmarks` (N4, suspected).
- Open: a sentence holding a URL (2310.15190).
- Also open (one-line inputs): amsart `\author{Name\\ \today}` dateline as an affiliation (1910.12289); a suit-led legend line read into the last name (2402.14804); a publication note after an affiliation's break (2406.05205); llncs mixed URL lines (1608.02693); the observer writer's re-part shows an unanswered refmark's number (`ieee_affil_block_after_a_marked_block_of_two`, synthetic, pinned in 94a); 2409.20550's `\thanks{\IEEEauthorrefmark{1} Corresponding author.}` stays a document note; two calls of one author printing one text the class prints once (REVTeX's two `\email{x}`, 2402.07705; two `\thanks{…}`, 2302.04271) — a class merge no binding declares; aastex `\correspondingauthor{Name\altaffiliation{1}}` beside `\author{Name\altaffiliation{1}}` (2307.05833).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade line".

### 106a: an address line kept as affiliation text (OD #966) — the residual

- **106a: an address line kept as affiliation text (OD #966) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: 2410.09681's template line, a line break inside a font wrapper (2311.04121), a phrase-led `\altaffiliation`; RED (rev106a2) the older route places by order.
- REDs: `addr_line_authblk_two_carriers_line_marks_red` (the older route), `addr_line_authblk_thanks_requested_mark_red` (a `\thanks`-requested mark unread by `author_mark_keys`); left: an address inside a legend note (2312.10429, 2501.04244).
- Also open: a phrase-led `\altaffiliation{Correspondence should be sent to a, b and c.}` (1209.3452); tagged runs `a@x (Matvey)` (2512.10975); `{a@mails., b@}tsinghua.edu.cn` (2602.07769); an affiliation the observer's writer makes after the flush (mn2e, 2008.08137); a template `{name.surname}@ 1polytechnique.edu` (2009.04283); e-mail contacts holding lists or names (2004.13933, 2005.04027, 2510.11067, 1512.01552, 2404.09958); class contacts other than affiliations (elsarticle 2310.13481, a `\thanks` 2509.07695, amsart `\address` glued to `\email` 2301.00107); unspelled addresses to the block (2608.22602's `chris241`, `chand158`); recorded: 1310.7913's unmarked `Corresponding Authors: a, b` names the e-mails rather than becoming a note (#961 needs a printed mark).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106a: an address line kept as affiliation text (OD #966) — t".

### 106s: the title-page check (OD #953) — the residual

- **106s: the title-page check (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED: a raw class printing its title page from `\begin{document}` (cup-journal.cls:1013-1014, 2302.11911);
- REDs (rev106s2) `\let\thanks\@gobble` a false alarm, `\let\thanks\footnote` (talk.cls:413) a silent loss;
- a `\thanks` in the third of several still recurses;
- a class word equal to a document word counted the document's;
- the placeholder ruling.
- Those REDs: `check_class_let_thanks_gobble_red`, `check_class_let_thanks_footnote_red`, `check_shipped_class_text_is_not_the_authors_red`, `check_document_sty_macro_body_counts_red`.
- The sibling: opendatalab's `\let\oldmaketitle\maketitle` (2601.09733).
- State: OPEN (residual)

### 106c: a tail inside the personname (OD #967) — the residual

- **106c: a tail inside the personname (OD #967) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: `\samethanks`/`\footnotemark[n]` keeps a mark-only note (2110.07431, 2402.06922);
- RED `grade_given_back_per_creator_red`, `grade_macro_names_and_membership_one_block_red` and kin.
- The kin: `grade_macro_block_beside_written_grade_red`;
- a lone acronym a collaboration contact, `collab_for_lone_acronym_is_no_group_red`;
- the `\samethanks` repro `author_samethanks_mark_is_the_same_note`.
- State: OPEN (residual)

### 106t: the title-page check's second source (OD #953) — the residual

- **106t: the title-page check's second source (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: quantumarticle's `\ead` (KPE #1208), ceurart's `\orcidauthor`; RED (rev106t2 S1) a binding selector's undeclared dropped branch; silent: aomart's `\subject`, daj's thanks (1510.08733, 2112.09352).
- That RED: `check_binding_selector_untaken_branch_unwanted` (fancyhdr's `\iffloatpage`).
- Also: the kernel's author splitter parts a name at a comma inside an optional argument's brackets (`\author{Ann Able\ead[label=e1,mark]{a@x}}`, 2101.10506, repro `check_binding_plumbing_is_no_dropped_text`), a content fix; dmtcs `\publicationdata`'s dates counted, the content fix open; silent: a raw file's macro inside a command's argument (europhys `\Pacs` cond-mat9610136, cond-mat9812194, cond-mat9906132; cpcHACK 0706.0578), non-UTF-8 sources (0906.3407).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106t: the title-page check's second source (OD #953) — the r".

### 107a: an address spells its author by transliteration — the residual

- **107a: an address spells its author by transliteration — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: a list after a depth-0 break inside one font wrapper (2408.05517, 2609.38967, 1805.08616).
- RED `email_list_unspelled_entries_legend_becomes_creator_red` unchanged.
- State: OPEN (residual)

### 107b: a names list not parted (OD #897 "107b") — the residual

- **107b: a names list not parted (OD #897 "107b") — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): `\mbox{S.~Burke-Spolaor,$^{12,13}$}` keeps the comma (1809.10744);
- a `\shortstack` and the tabular/minipage blocks stay one author;
- REDs `author_row_of_name_shaped_institutions_red`, `author_space_tie_between_two_names_red`.
- RED `author_sub_lettered_mark_shown_on_name_red` (1202.2048).
- Also 1505.05811's empty-mark `${}^{\mathsf{}}$` continuation lines go to the block creator, no word lost (F4).
- State: OPEN (residual)

### 107c: an address, legend or URL line kinded affiliation (OD #966, #961, #969) — the residual

- **107c: an address, legend or URL line kinded affiliation (OD #966, #961, #969) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: a run a role tag follows (2503.14354, 2210.14319).
- Left beside a fixed defect: 2602.07392 (aastex631's second affiliation on the last author only), 2503.04130 (`aff_foreign`).
- Also open (one line each): a template line `Emails: name.surname@polito.it, …` (2401.15236); initials tags `a@x (X.X.); b@y (M.Y.)` (2002.08904); a sentence `Correspondence … should be addressed to F. B. at x@y.` (2208.12334); authblk `\affil[ ]{Email}` (2009.11285); PoS speaker e-mail on the mark-1 entry (1412.5940); a marked `$^\ast$~Correspondence: a@x` line outside a wrapper stays a legend note; an institution after "work done during …" keeps 103c's reading; a parenthesized list `(a, b)@x` stays unread; a ✉ line the writer parts leaves "🖂 Corresponding author" in the affiliation; authblk `…USA; a@x. b@y` stays whole; a correspondence note from the author block's route carries no mark; `In memory of …, and supported by …` stays one dedication.
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107c: an address, legend or URL line kinded affiliation (OD ".

### 107d: the title-page check where no `\maketitle` of ours runs (OD #953) — the residual

- **107d: the title-page check where no `\maketitle` of ours runs (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `check_title_page_relaxed_maketitle_no_false_warning_red` (S1), a sibling-directory `\input` read as a package's (S2).
- Open, one dev3 paper: nj_style's `\ead` written to the `.aux`, printed on pdflatex's second run (1401.7920).
- State: OPEN (residual)

### 107e: a marked run cut whatever parts it (OD #954) — the residual

- **107e: a marked run cut whatever parts it (OD #954) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: 2605.25195's legend (the writer declines its URL line's owner); REDs `cut_run_wrapped_entry_then_comma_address_red`, `cut_run_group_entry_then_comma_address_red`.
- Also `cut_run_legend_after_wrapped_entry_ending_in_a_group_red`, `cut_run_nested_wrapper_per_entry_red`.
- Also open: the kernel alone welds a legend line into the entry above it after any run (base the same); plain isotope forms `$^{1}$Lab for $^{3}$He Studies` still cut (base the same); an entry led by two marks labelled by its first only (2411.03010); the unmarked last name revtex prints the head's number on holds nothing (2109.01501's Oshiyama, 1903.07834's Matos-Abiague); the kernel cut shows no mark and repeats no lead declaration (2603.19086, 2609.20163); consecutive marked/unmarked `\affiliation` calls (gr-qc0611116); PoS `\footnotesize` group in `\author` (1509.01480); 2309.12239's two unspelled addresses on the last name (ADDR-b); pre-existing: a stray space before a comma after a wrapper, a legend line directly under the names becomes extra creators.
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107e: a marked run cut whatever parts it (OD #954) — the res".

### 107f: e-mail ownership (OD #970, #972, #966) — the residual

- **107f: e-mail ownership (OD #970, #972, #966) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: 2511.18493's `minhnnhat@` goes to Anh-Minh Phan; 2602.23777's `nnwang@`/`dongsli@`, 2310.05782 (a tie elimination could part); 2001.00550's "Name - address" lines keep "Marco Cerezo -".
- RED `addr_line_sole_author_symbol_series_orphan_is_blocks_red` (`mark_series`' symbol branch does not fire).
- Also open: a note holding a list for its carriers is not parted; initials tags `a@x (X.X.)` (2002.08904); a list in reverse author order stays on the block (2508.07950's `wzy218@`, `jhma@`); a local part with no separator spells the surname it ends with (`rickyang@`); 2009.10916's two institute e-mails stay both Bo Li's under the supplement; the tie elimination would not part 2501.11284's `yyzhang@` (three Zhangs).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107f: e-mail ownership (OD #970, #972, #966) — the residual".

### 107g: the Bundle 5 fix round (OD #970, #966, #897) — the residual

- **107g: the Bundle 5 fix round (OD #970, #966, #897) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `email_list_mark_line_names_non_carriers_no_spelling_red`, `email_list_mark_line_spelled_out_of_place_no_order_red`, `email_own_call_printed_under_both_stays_on_both_red` (2502.15891, 2307.00704, 2405.01799), `author_block_repeated_affiliations_land_on_the_last_red` (2307.07324, 2001.07542).
- Also `author_tie_space_inside_three_word_name_red`, `email_own_call_in_an_entry_tie_moves_red`, `addr_line_glued_word_before_a_font_observer_route_red`.
- Open: 2010.09850's two institutes in one `\parbox` are one affiliation as on base (the shape of 2509.02924, no rule tells them apart); 2604.19098's `\parbox` of address lines stays in the affiliation as on base (the option: `@` as a third reason a wrapper is transparent); RED `email_own_call_last_author_every_author_addressed_stays_red` and its twin `email_own_call_last_author_every_author_addressed_stays_twin_red` (`last` too broad; 0 rows in 3 dev runs); not blocking: 1103.0889 `Faranda, Davide` mis-split, `v.lucarini@` on "Davide"; out of scope, unchanged: 1406.2912's legend tail on the Hanoi affiliation, 0807.2167's `$^1$NIKHEF` welded into a name, 2601.12698's dropped mark.
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107g: the Bundle 5 fix round (OD #970, #966, #897) — the res".

### 108a: authors merged under authblk (2610.06709; OD #52(h)) — the residual

- **108a: authors merged under authblk (2610.06709; OD #52(h)) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: `\footnote` under a sole authblk author (2601.20363, 2605.13069; RED `authblk_footnote_line_under_sole_author_red`); 2505.05785 (RED `email_spelled_part_of_a_hyphen_name_red`); 2606.04547, 2412.13509; REDs `authblk_institution_line_before_affil_red`, `authblk_legend_swallows_contact_lines_red`, `authblk_title_suffix_one_name_red`, `authblk_small_hspace_in_name_red`.
- RED `authblk_link_macro_under_names_stays_merged_uncounted_red` (2601.07891); the trade, recorded: "as many left, in order" requires the order to hold, so out-of-order lists go to the block (2606.04646, 2605.26032, 2102.08716); int140a's open: 2511.07187 (`Name -- \url{…}\\ …`: 3 of 4 authors read as affiliation lines, no self-check fires).
- Also int140a's open: cpc-hepnp's `1;1)` could part into its affiliation and its note instead of counting a lost mark (#431); an e-mail keeps a glued mark in its text, `cak29@mail.aub.edu *` (2609.25833), `*archnalakhani@` (1201.5255).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "108a: authors merged under authblk (2610.06709; OD #52(h)) —".

### 108b: the silent lost words counted (OD #953) — the residual

- **108b: the silent lost words counted (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open RED catalog `check_raw_setter_input_own_file`, `check_store_read_only_in_first_page_foot`;
- a glued `\textbf{Lunar}\includegraphics{logo}` fails the source gate.
- State: OPEN (residual)

### 108b replay gate: a discarded print need not obey it

- **108b replay gate: a discarded print need not obey it** — OPEN.
- `sect05.rs` "not replayed: it uses …, which is undefined here" refuses a class's `\maketitle` for the kept replay and for the check's print;
- 61 of 10,000 dev3 logs (~17,000 arXiv papers).
- Letting the discarded print run ignore the gate recovers the print (untested: 2408.05088, `check_raw_setter_replay_refused`).
- Counted meanwhile by `unclaimed_store`

### 108b content: keep what mechanism A only counts

- **108b content: keep what mechanism A only counts** — OPEN, a ruling.
- Content losses against Perl (it keeps them, with Errors, apparently by not reading the raw class): 2305.00867 (Perl E=14), 1011.2770 (E=58), 2605.02209 (E=11), repro `check_raw_setter_store_gated` (E=0).
- Rust's run is clean and loses them, now counted, not kept

### 108b `\newcommand`: a class's `\newcommand{\@maketitle}` ignored under the kernel's empty one

- **108b `\newcommand`: a class's `\newcommand{\@maketitle}` ignored under the kernel's empty one** — OPEN, found here.
- cjpsuppl.cls:199 (hep-ph0402063): "Ignoring redefinition (\newcommand) of `\@maketitle`", no `\lx@dropped@` record, so the class's title page (`\@authori`, `\@addressi`) never runs: the authors lost;
- the words reach only `Info:frontmatter:unread_store`.
- In pdflatex the class loads no article and the name is free.
- Minimal: a class with no `\LoadClass` and `\newcommand{\@maketitle}{Lunar}`

### 97a review should-fixes (rev97a, SHIP, 2026-10-31)

- **97a review should-fixes (rev97a, SHIP, 2026-10-31)** — (S1, synthetic; FIXED 97b, OD #944: a piece that is an address and no name is no marked name, GREEN `perfect_kernel_batch97b::marked_address_piece_is_no_person`) a marked e-mail piece is made a person: `names_split.rs` `marked_names` counts any piece `reads_as_affiliation` declines as a marked name, and `frontmatter/classify/marks.rs:8-11` declines every text holding `@`, so `\author{A.~Able$^{1}$, B.~in der Baker$^{2}$, b.baker@two.edu$^{*}$}` (revtex4-2) gives three creators, the third `<personname>b.baker@two.edu<sup>*</sup></personname>` (95b r3: one creator); the same for a marked URL, ORCID or place name the shape test reads as a name; fix: a piece the e-mail reader would take is no name, the line then a reported merge; no census paper's changed creators hold an `@` (RED `author_marked_email_piece_is_no_person`); (N1) the merge message "One author holds N names" is a lower bound read as exact (`name_text.rs:549-554` `names.max(_mergednames)`: 2508.04092 says 2 of 4, 1012.4815 3 of 4, 1212.4143 6 of 7) — word it "at least N" or take the larger of the marked and comma-piece counts; (P2, older) 2312.16289's e-mail contact carries raw TeX in its label, `name="${}^{\ast}$: Corresponding author: "` (int123a the same)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97a review should-fixes (rev97a, SHIP, 2026-10-31)".

### 97b: a take-out removes what it read and nothing else TeX prints

- **97b: a take-out removes what it read and nothing else TeX prints** (OD #944; the int125b A/B's `fm_checks.py` losses against int123a) — FIXED: aa's `\mail` labelled as the shipped copy prints it (a copy naming `\mailname`, v4.03-v7.0, keeps "Correspondence to "; a storing copy, v8 on, none; `\mailname`'s `\it` grouped, no italic on later items); the writer's take-out of an entry it empties keeps the entry's printed label on its e-mail contacts (`holder_label`; not a role default), or gives the entry back when no contact would carry it (`give_back_entries`: 1005.3784, astro-ph0107243); a K11 store's label read before its printed occurrence, not in `\href`'s target, through `\href`/`\textcolor` text, and a note store's label leading its text (2506.10896 "Correspondence:"); a marked-list split's printed separators in `before` (hep-ph9710228 ", " / " and "; 2003.03705, 2307.12736, 2401.13225, nucl-ex0410037, 2410.11607).
- Witnesses 0805.1001, 1005.3784, 1102.1556, 1505.05452, astro-ph0107243, astro-ph0107266, astro-ph0108379, astro-ph0609025, astro-ph9907206, 1209.2606 (no copy); storing 2007.14739, 2012.11607, 2606.29508.
- Guards `perfect_kernel_batch97b` (five).
- Residuals, each a one-line input: (a) a shape-split names line keeps the binding's `before` — `\documentclass{revtex4}…\author{Ann Able, Bob Baker and Cat Cole}` before="  " (int123a the same; "and" printed nowhere); (b) two prints of one address with two labels keep one — `\institute{Univ X \\ email: x@y.org}\mail{x@y.org}` under a printing aa.cls gives one contact "Correspondence to ", the institute's "email:" nowhere (95c's first-print order); (c) 2508.08560: a raw package's `\affiliation` store whose ★⧫♠♣ e-mail legend follows two `\vspace{0.3cm} $ $ \\` lines is one affiliation of the last name, its marks stripped — the store-affiliation line reading, not a take-out (RED `store_affiliation_email_legend_after_spacing_lines`); (e) a label part the deposit takes out of the first place its tokens stand, not the branch that runs: jpsj2.cls:789-818 prints `KEYWORDS:\hskip.5em\parbox[t]{…}{\@kword}` in five option branches, so the printed "KEYWORDS:" stays a body paragraph beside the keywords' name (int125b the same; 416 papers of the bundle population, 0704.0082); (d) the 95d r3 RED rows `raw_class_date_branch_with_trailing_text_keeps_it`, `raw_class_label_with_a_comma_stays_whole` stay RED (other mechanisms: the whole-test removal, the clause split)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97b: a take-out removes what it read and nothing else TeX pr".

### 97b review should-fixes (rev97b, SHIP, 2026-11-01)

- **97b review should-fixes (rev97b, SHIP, 2026-11-01)** — (S1, synthetic) the store-label reader knows which arguments print only for the commands its table names (`frontmatter_stores.rs` `argument_of`, `PRINTED_ARGUMENTS`), so a printed label is still lost through a raw file's own wrapper (`\newcommand{\mymailto}[1]{\href{mailto:#1}{#1}}`, `Correspondence: \mymailto{\@corremail}`), `\hyperlink{corr}{\@corremail}` or `{\color{black}\@corremail}` (int125b the same); parameter types cannot decide it (`\url` and `\href`'s target both Semiverbatim); fix: read a raw-file macro with parameters through its body, `\hyperlink`'s 2nd and `\hyperref[…]`'s argument print, a `\color` inside the store's group is layout (RED `raw_package_note_label_before_a_wrapper_macro_names_it`); (S2, synthetic) rev97a S1's siblings: a marked address piece led by a label word (`E-mail: b.baker@two.edu$^{*}$`, 1504.06050's line shape) or a marked URL (`www.two.edu/~baker$^{*}$`) still reads as a marked name (`names_split.rs:173-179` `only_addresses`; int125b the same; RED `author_marked_labelled_address_piece_is_no_person`); fix: skip a leading label word ending in `:`, count URL-shaped words as addresses; (N2) the `in_unprinted_argument` doc comment still claims a `\write`'s text, but `argument_of` looks only at the token before the group (`\write\@auxout{…}`, `\marks3{…}` never match) — OD #944 corrected, the comment waits on the next writer in that file; (N3) the guard name `aa_mail_storing_copy_has_no_label` asserts the kernel default `name="Email: "` on an address pdflatex prints nowhere; (N4) `argument_of` skips one `[…]` only (`\parbox[t][3cm][s]{w}{\@x}`); (P1, older) HTML shows a note's role name before its text (`<span class="ltx_note_type">email: </span>`, LaTeXML-meta-xhtml.xsl:77-83, bold in CSS): 2506.10896 renders "email: Correspondence: …" — a role name the class never prints (FIDELITY_CHARTER §4); (P2, older) printed words around a store still go with it without a Warning ("Write to the author", a trailing "(preferred)", a leading "Mail"; the conservation rule of brief 97b part B); (P3, older) a given-back `\mail` address goes to the last creator, not the one it spells (1005.3784 Woodcraft; astro-ph0107243 `faraggiana@`; int123a the same); (P4, older) 1504.06050's per-address marks a/a/b collapse to one `a,b` on a nameless creator (appolb, no TL oracle)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97b review should-fixes (rev97b, SHIP, 2026-11-01)".

### 97c: an affiliation the reading already has reaches its author

- **97c: an affiliation the reading already has reaches its author** (OD #945; dev20k's largest archival shape `own:aff_missing`, 364 creators / 119 papers) — FIXED: (W1) a names block's mark-led entry of several events is cut by its whole span (resolver/blocks.rs `labelled_entries`; `$^{1}$Univ of Science and Technology~~~~$^{3}$Univ C` was Ann's whole: 2111.07950, 1503.06795, 2504.05692, 2008.02300, 2404.01856; W1b 2212.01433, 2608.06741), a digit mark glued inside a word only when the source glues it (events/lexer.rs `glued_in_source`: `NLP$^2$CT` the word's, `Russia\break $^2$Nuclear` an entry, hep-ph9809222); (L1) llncs's one `\institute` cut at its marks (shape (c)) keeps the sole institute every author's for its unmarked piece (`affiliation_calls`'s `unmarked_attr`, `_share=authors`), hand-typed numbers labelling its other pieces (`_markpiece`: 1905.07852; 2603.18089, 2608.22772, 2207.13038, 2609.31726); (SV1) contrib svproc no longer overrides `\institute` with a document note, sv_support's split stands, as Perl's (2101.04527, 2504.04157, 2206.03859, 1911.11805, 2409.01900).
- A/B int125a 28,354 papers: 70 changed, no creator merged or lost, no frontmatter word lost; scorer 3,000: archival 98.18 → 98.36 %, lost-words 0, R2 7 → 5; dev 146 papers: 81 creators to right, 0 from.
- Guards `perfect_kernel_batch97c` (five); changed `perfect_kernel_batchfms2a2r8::a_text_mark_glyph_is_no_footnote_symbol` (written now, XML pinned), `r8_shapes_leave_the_xml_unchanged`.
- Residuals, each a one-line input: (a) two group-wrapped entry lines do not cut — `\author{Ann Able$^{1}$ \quad Bob Baker$^{2}$ \quad Cy Cole$^{3}$ \\ \textnormal{$^{1}$Univ A \quad $^{2}$Univ B} \\ \textnormal{$^{3}$Univ C}}` gives Ann "Univ A 2Univ B" (2609.38081; 97b the same); (b) llncs legend line first — `\institute{\textsuperscript{*}Equal contribution \\ Univ A, Town}` continues the legend, Ann "Equal contributionUniv A, Town", Bob none; (c) svproc/svjour sole institute with an `\email` — `\author{Ann Able \and Bob Baker}\institute{Univ A, Town A\\ \email{ann@x.org}}` the block's, address included (1705.10638, 2010.10850, 2207.12084, 2307.06258); under numbered institutes hand-typed `${}^{1}$` marks answer no piece today (2608.25509: affiliations the block's, three of five addresses off their authors; the reading is right, sv_support is no written family); (d) "†Project lead" read as an affiliation of its mark's names (2609.18514, 2609.19719: `classify_thanks` knows no project role) — FIXED 103c (OD #961, row 103c); (e) the scorer folds an llncs legend into the institute (SL-legend): 2603.18089, 2608.22772, 2207.13038 score `aff_unexplained` though the XML matches pdflatex  Review rev97cd (NO-SHIP on 97c alone): C1 (svproc addresses on every author of their institute) and C2 (2608.25509) are fixed by 97e's sv_support writer — on 97c + 97e none of the seven C1 papers shares an address, each gains its affiliations, and 2608.25509 holds every affiliation and the two spelled addresses on their authors, its three initials-only addresses (`ngr@`, `ygt@`, `frt@`) at the block (base: by position; initials as evidence is the refactor's placement order) — so SV1 lands only with 97e (OD #945 amended); C3 synthetic, RED `author_digit_mark_glued_after_an_entry_opens_the_next` (the in-word digit glue takes a requested label); C4 `_markpiece` is binding data.
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97c: an affiliation the reading already has reaches its auth".

### 97d: a collaboration named in the author line is a contact (OD #946) — the residual

- **97d: a collaboration named in the author line is a contact (OD #946) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left, one-line inputs: (a) aa's own `\and` split; (g) RED `author_collaboration_name_with_and_stays_one`; (h) revtex's `\collaboration` (2412.18491); (j) 1603.09150's team legend.
- (a)'s witness 1912.05868; (j)'s RED `author_team_piece_mark_labels_its_own_line`.
- Also left: (a) 2512.08803 (`XYZ Collaboration: A.~Able\inst{1}` one name); (b) the IEEE block reader (1111.2718; 0907.0548's "Brigidaon behalf …"); (c) the reading's heading over later lines, its remark line under one name (the writer declines); (d) `\author{Ann Able, University of Toronto and Bob Baker for the CMS Collaboration}`; (e) emulateapj `$^{n}$` marks under `\address{$^{n}$ …}` not linked (0910.4381); (f) PoS 1708.04166's `E-mail:` line to all 15 names; (h) `\affiliation{(WA98 Collaboration)}` an affiliation (0708.2630), awaiting a ruling; (i) PoS's `\author{Fermilab Lattice, MILC, and HPQCD Collaborations}` three creators (hep-lat0509169); (j) 1508.03930's `$^b$` line; (k) an unmarked `\and` group led by a collaboration over a names line; (l) a bracketed team under a name.
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97d: a collaboration named in the author line is a contact (".

### 97f: giant lists in a size environment; AASTeX 6.x's shared affiliations — the residual

- **97f: giant lists in a size environment; AASTeX 6.x's shared affiliations — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: (a) sub-labelled marks (RED `author_sub_labelled_marks_read_as_digits`, ruling deferred); (b) unlabelled lines under `$^{N}$` names, a mark after the comma; RED `aa_collaboration_heading_before_colon_is_a_name`; (e) PoS M3 (1711.04132); an unmarked e-mail line in a marked `\affil`; (h) a nested `\author` (2012.14750).
- (b)'s witness 1202.0629; the unmarked e-mail line's 2008.05143.
- Also: (a)'s witnesses 1507.02068, 1512.08197, 2201.02056, 2410.11607's `75A,75C`; an `\altaffilmark{1$\dagger$}` request matches no label (2006.10291's N. Fraija); (c) aa's `XYZ Collaboration: R.~Abuter\inst{1}` one name (2206.00684); (d) a `{\it $^{1}$ Uni One\\ …}` group keeps a trailing `<break/>` and a leading space (every BESIII affiliation); (f) a `{\it …}` group glued after its line breaks is one entry (2007.02542: 347 of 490 names no affiliation); (g) a marked entry glued after a comma inside one `\affil` is not cut (0909.3044).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97f: giant lists in a size environment; AASTeX 6.x's shared ".

### 99a: one Cut (OD #954) — the residual

- **99a: one Cut (OD #954) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): an unrequested entry once at the block (hep-ph9805510, 2608.11300);
- IEEEtran's symbol-labelled lines (2308.05546);
- REDs `cut_acmart_uncut_call_after_a_cut_list_red`, `cut_acmart_closing_part_composes_unprinted_text_red`;
- N-b a conjunction dropped between entries;
- 2009.04283;
- 2204.02412.
- Pathological, deferred: REDs `cut_acmart_template_address_line_red`, `inblock_glued_repeated_label_is_entry_text_red`;
- int127a's RESIDUAL: a cut entry shows no label once its name's mark is hidden as answered (`hide_answered_marks`; 2406.07529, 2207.02356, 2309.15702, 2405.17878).
- State: OPEN (residual)

### 98b: four lost-words mechanisms (OD #949-#952) — the residual

- **98b: four lost-words mechanisms (OD #949-#952) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: pathological 2008.12867 (RED `svjour3_institute_environment_before_at_is_lost`).
- Also left: an affiliation-only `\IEEEauthorblockA{X, Y\\ …}` parted at its first line's commas (words kept); a `\at` name with a remark the fuzzy owner misses (`Lun Ai (corresponding author)`, 2405.06724) stays on the block's creator, with its text; an itemize list whose later names come after its first `\at` (2306.07927, 2605.09054) is the first author's whole, as int126a (`\lx@sv@institute@` reads the first `\at` only); `\institute{\begin{itemize}\item A \at X \and \item B \at Y \end{itemize}}` not split at its `\and` (synthetic, no paper known); the list's items reach no author by their marks (`\item[\textsuperscript{1}] …`: the list is one contact).
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "98b: four lost-words mechanisms (OD #949-#952) — the residua".

### field stamps overflow on a large block (2410.11607) — the residual

- **field stamps overflow on a large block (2410.11607) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): 1810.03464 is a rehearsal over its read budget (vldb-modify.cls's `\normalsize`; the leftovers row's item 6).
- State: OPEN (residual)

### R2 binding-API data: ijcai `\emails`, cas `\address[label]` (88b; OD #908, KPE #1200) — the residual

- **R2 binding-API data: ijcai `\emails`, cas `\address[label]` (88b; OD #908, KPE #1200) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): residuals with options in OD #908: marked groups kept whole on the block (2105.02711), `wulk@` unspelled (2308.08563), a group in markup on one author (2501.02356).
- Also: the one leftover address to the corresponding author (2304.11989 `scs_lzg@`, 2505.12894's plural legend; a user-ruling step, kept; two narrower refinements for the user);
- ijcai's one unmarked `\affiliations` line on the first author only (Perl's `labelseq=author`).
- State: OPEN (residual)

### amsart: the frontmatter setters after `\maketitle` are still the binding's (pre-existing on every binary; 92a review rev92a_r2 P2)

- **amsart: the frontmatter setters after `\maketitle` are still the binding's (pre-existing on every binary; 92a review rev92a_r2 P2)**: amsart.cls:621-629 `\@cleartopmattertags` lets `\maketitle`, `\title`, `\author`, `\address`, `\email` and the other setters `\relax` after the first `\maketitle`, so `\title{Old}\maketitle Text. \title{New}\author{Ann Able}\address{University X, Town}\email{ann@x.org}\maketitle More.` prints "Text.
- New Ann Able University X, Town ann@x.org More." as body text under the title "OLD".
- We print "New Ann Able" in the body and put the address and the e-mail on a nameless creator;
- Perl prints the same body text and loses them.
- RED catalog repro `class-bindings/amsart_setters_after_maketitle_print_as_text`;
- not 92a's mechanism, so no guard
- State: OPEN

### FM S2b: ams_support written (81a-81f, 82a-82e; OD #837 "S2b next family") — the residual

- **FM S2b: ams_support written (81a-81f, 82a-82e; OD #837 "S2b next family") — the residual** (landed: 82b `written: true`, 82e; its Bundle A/B and nets superseded by the later bundles; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): 1610.06222 open (marks in an address's own names line).
- State: OPEN (residual)

### FM S2b: IEEEtran written (95b, OD #942) — the residual

- **FM S2b: IEEEtran written (95b, OD #942) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left 2602.04239 (an ORCID line kept as an affiliation), 2603.10979 (affiliations in `\thanks` legends), 2311.14729.
- State: OPEN (residual)

### FM S2b: acmart written (95b, OD #942) — the residual

- **FM S2b: acmart written (95b, OD #942) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left 2010.12245, 2607.06101.
- State: OPEN (residual)

### FM S2b: authblk written (95b, OD #942) — the residual

- **FM S2b: authblk written (95b, OD #942) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left 2609.33086, 1208.5147, 2603.11001.
- State: OPEN (residual)

### 97e: every remaining observer-writer family written (OD #948) — the residual

- **97e: every remaining observer-writer family written (OD #948) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left 1509.06963 (jcappub, declined), 2009.04166.
- Also: the svjour/elsart collaboration headings wait on OD #946's contact reaching the per-author bindings' `\and` pieces;
- a re-parted entry's `\textit` mark kept in its text where the source typed `//` (1306.1933).
- State: OPEN (residual)

### 97e r2 residuals (review rev97e; OD #948)

- **97e r2 residuals (review rev97e; OD #948)** — (N1) 1108.2557 (ws-ijmpd, inst_support): the e-mail line printed under the addresses is read into entry b, so all three addresses go to Geng (97c: on no one) — the reading's line cut; (N2) the paper-wide corresponding-author fallback is gone where an e-mail call's group is empty (`resolver/post.rs` `answer_sequence`, 97e r1's scoping): the address falls to the block, fail-safe; `events::lexer::letter_spaced_word_space` reads `Smith A B\enspace Jones C` (two names, the last word one letter) as one name — no witness yet; (key) a hand-typed mark equal to an elsarticle key whose place prints otherwise, `\author[a]{Cy Cole$^{*}$}` under `\address[*]{…}`: today's and the reading give Cy Cole the line, pdflatex prints "c current address" — a hand mark is read against the key, not the place letter the class prints (`\theaffn`; keys in order agree, 0907.5014), RED note in repro class-bindings/`elsarticle_key_labelled_address_is_no_mark`; (note) a `\footnotetext[n]` after `\maketitle` the reading pairs with a name's `\footnotemark[n]` is a note today's holds only after the writer runs (its relocation followed the S2b writer, deleted in DEL4), so the paper declines whole — pre-existing in every family (`article` with `\renewcommand\thefootnote{\fnsymbol{footnote}}`), reached in elsarticle by its arabic marks since r2: 1108.0201, 0909.2782 (r1 had split its e-mails onto their authors); fix: the writer leaves a text built after the names' flush to today's relocation (neither wanted nor compared), then a whole-population A/B; (keys) the other bindings labelling an entry by a key (`label={#1}`: imsart, ecai, ceurart, icml, sn-jnl, bmcart, beilstein, interspeech, vmsta, quantumarticle; `UNDECLARED`, their own `\author`) are not audited for printed keys — a class printing the entry's place, not the key, declares `keyed_labels` as elsart_support_core does; (pre-existing) the fms2a2 `addr` source (`1. University A … 2. University B`) gives every author every line, against FIDELITY_CHARTER §4 R1/R2; marks left in the names of 1201.0008, 2307.11481, 2506.22621
- State: OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97e r2 residuals (review rev97e; OD #948)".

### int126a review (rev126a; OD #947, #948)

- **int126a review (rev126a; OD #947, #948)** — B1 and B3 FIXED in 98a (a collaboration heading a printed separator parts from the first name is an entry, 2206.00684; a control word in an `\altaffilmark` list is no request label, astro-ph0105133; guards `perfect_kernel_batch98a`).
- Pre-existing, recorded only, each unchanged int126a → 98a: (a) 1708.04166 (PoS): `serena.benatti@…` held by 5 Padova authors, not the one it spells; (b) 1909.05984: G. Vianello holds the collaboration contact twice; (c) marks left in names — hep-ph0301204 "1 , 2 G. Ricco" (the comma split, row 97f (b)), 0907.5014 "Denes Molnar a,b"; (d) 1212.4143 still one creator.
- Open for a ruling: the heading `XYZ Collaboration\thanks{…}: Ann Able` is written as a creator of its own (Perl's and the old reader's `\and` entry, 97b's output), not OD #946's collaboration contact — svjour's, llncs's and aa's `\and` pieces never reach the author block's collaboration reader (row 97d (a)); elsart's `\author{XYZ Collaboration: Ann Able}` already gives the contact.
- Noted, no output change: `\altaffilmark{1,~2}` requests a stray `altaffil:˜` beside `altaffil:1`/`altaffil:2` (the observer's raw text spells the tie `˜`), answered by nothing.
- **The bundle A/B's blocker FIXED in 98e (review revab126, OD #947 round 3):** REVTeX's group rule copied a whole multi-label block onto every author — a one-argument line wrapper is now a line (`\centerline`/`\leftline`/`\rightline`, a whole-line or mark-led `\mbox`/`\hbox`: D0 hep-ex0409040 377,251 → 10,085 words, hep-ex0612013 428,795 → 11,807, each author its own institute; 1412.5812, 0911.2853), a block of several requested labels no mark the binding reads leads lands once at the block (`addresses:block`, `_unsplit=true` for the observer; 0807.4407, 2208.09667, 2308.02280 once on the block, every author's before), and the writer's block-line cut gives a line no name requests to the block once instead of declining the paper (2007.02542: 143 holders of mark 1 the whole list, 347 none → every author its lines); guards `perfect_kernel_batch98e`.
- **Review rev98e's findings FIXED in 98e r2 (OD #947 round 3):** a sub-lettered mark goes by its number — a digit run is cut only under single-digit numbering and an entry's own leading number is never parted (`parted_label`, `part_glued_labels`: 2007.02542's 36 sub-lettered authors had two wrong institutes each, R. Baldini Ferroli `23A` Beihang and Beijing Petrochemical; 1507.02068 `48A` → `4`, `8`); a cut block line no name requests keeps its printed label (`b Also at …`; ruling tentative, the user away); the fail-safe fires only for two or more lines led by requested labels (a line holding marks inside its text, `Joint Institute, \textsuperscript{1}Univ A and \textsuperscript{2}Univ B`, is the group's copy on each again) and a mark spelled otherwise is the mark it prints, read by one shared reader of a line's leading label (`relocate::spaced_mark_at`, used by `notes::leads_with_mark`, `leading_line_label`, `requested_mark_lines`; REVTeX's `spaced_marks_as_marks` writes such marks as `$^{1}$`): `${\,\!}^1$`, `${}^1$`, `$~^1$`, `${^1}$`, `\( ^{1} \)`, `\mbox{${\ }^{1}$}` — 2112.13115 each author its departments (98e r1 none), 0807.4407 and 1211.5032 each author its lines, a `tabular` block (0911.3234) once on the block; guards `perfect_kernel_batch98e` (`a_sub_lettered_mark_goes_by_its_number`, `the_fail_safe_fires_only_for_labelled_lines`, `a_mark_on_a_spacing_nucleus_leads_its_line`).
- Residuals, recorded (no fix in 98e): 2506.19040 `drohr@mit.edu` copied to the other two `*` authors beside their own addresses (suspected `frontmatter/observer/resolver/owners.rs:666` `spelled_owners` reading "drohr" as spelling no one); BESIII sub-lettered marks (`75A,75C`, `29A`, `63A,63C`) keep the mark in the name, the name given the whole numbered entry, not its lettered part (33-36 names a paper; R6, tentative: sub-entries at Stage 4a); 1609.03822's `\pounds` label is not split; nameless creators holding a stray copy: 2307.16667 [5], 2312.09590 [3], 2109.02295 [4]; `&&`/`\nonumber` affiliation lines (eqnarray residue; pdflatex errs) keep the block one entry (2208.09667, 2308.02280: pathological — defer); numbered lines set in a `\parbox` inside one `\affiliation` land once at the block (review rev98e fs3: pathological — defer; a one-column `tabular`'s rows are each author's since 104c, 0911.3234, OD #964); 2112.13115's second department keeps its line's printed `and` at its head ("and Department of Physics …", the marked list keeps a line's text before its mark); the observer reads no Mark event from a nucleus formula (`${\,\!}^1$`, `\mbox{${\ }^{1}$}`: 98e r1's `_unsplit` cut never fired on them; the binding's token reader now covers them, the observer's `observer::marks` does not — a second engine's reading of the same spelling, open); the kernel `\author` block's unrequested line keeps no printed label (`\author{Ann Able$^{1}$, …\\ $^{1}$Alpha …\\ $^{b}$Also at Eps College}` → a nameless creator "Also at Eps College", 98a the same: the `\lx@affiliation@withsup` pending creator relocate_annotations leaves unanswered, not the writer's cut — review rev98e's sp2)
- State: OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "int126a review (rev126a; OD #947, #948)".

### authblk: a label holding a command before its number

- **authblk: a label holding a command before its number** (`\author[$\dagger$1]{Ann Able}`, pdflatex "Ann Able†1") — `labels::parted_label` reads a source-spelled label to its first command (2405.16051), so the label is nothing: Ann Able holds no affiliation, entry 1 and the `$\dagger$` entry stay in the block (int124b alike; `$^{\dagger}$1` alike);
- `\author[1$\dagger$]` keeps `1` from 95b on, the dagger's entry in the block.
- Synthetic (review rev95b);
- repro class-bindings/`authblk_label_opening_with_a_command` (RED, no guard).
- Options: read each label piece a command prints as a mark (the `\affil[$\dagger$]` label's own reading)
- State: OPEN

### The take-out reads a replaced `\author` call's block items

- **The take-out reads a replaced `\author` call's block items** (`Resolver::take_out_entry_addresses`, post.rs ~1664): the creators' prints are the live creators' (`self.live()`), the block's every item, a replaced call's (IEEEtran's anonymous-review `\author{…}` before the real one, 2602.12633's shape) included — an address in such a call's block entry would be taken out and owned.
- Output identical to int124b on the census;
- no repro (a replaced call holding a block e-mail is not cheap to build).
- Fix: skip the block items whose source is a replaced call
- State: OPEN (review rev95b)

### authblk residuals read on the 95b census (output identical to int124b)

- **authblk residuals read on the 95b census (output identical to int124b)**: 2106.15893 — `\author[$^{1,2}$]{…}`, `\author[$^{\star,1}$]{…}`, `\affil[$^1$]{…}`: a label spelled as a formula's script is not read, every entry in the block;
- 1411.2438 — one `\affil[ ]{…}` shared by all three authors sits on the last only;
- 2008.01011 — `\affil[$\S$]{…}` stays in the block.
- Each a one-line input of its source's shape
- State: OPEN (review rev95b)

### Writer cosmetics on the 95b census (review rev95b N2)

- **Writer cosmetics on the 95b census (review rev95b N2)**: 2507.03771 — authblk `\affil[*]{Corresponding authors: \color{blue}yujiewang@sjtu.edu.cn\color{black}, …}`: the take-out leaves "Corresponding authors", its colon trimmed with the cut addresses (pdflatex prints "Corresponding authors: …");
- 2603.23991 — IEEEtran `Qiong Wu$^{1,2,}$$^\ast{}$`: the name keeps "1,2," as the source prints it, its `∗` hidden as answered by the legend now hers ("∗Corresponding author is Qiong Wu.").
- Neither is a separator-trim one-liner of the 95c F6 rule: the first needs the cut to keep a label's own colon, the second is the source's comma
- State: OPEN

### Unobserved frontmatter calls under a written author block (95b r3 residual, OD #942)

- **Unobserved frontmatter calls under a written author block (95b r3 residual, OD #942)** — the writer tells an unwritten family's call apart only when a watched setter's expansion made it (`ledger::FRONTMATTER_SETTERS`); any other API call under a written block is written as before, whoever made it: a binding's other frontmatter commands (`\thanks`, `\IEEEauthorblockA`, elsarticle's `\cortext`, revtex's `\altaffiliation`), the document's own `\lx@add@…`, and a raw file's store handed on at `\maketitle` (`frontmatter_stores::harvest_stores`, where no setter's expansion is open — a raw setter is the class's family anyway, so only a raw style's store under an unwritten class family matters).
- One-line minimal input of the class, no census paper showing it: `\documentclass{svjour3}\usepackage{authblk}` with a raw style's `\def\address#1{\gdef\@address{#1}}` printed by the class.
- Census: no paper of the 95b populations or the 419 authblk papers under other classes changed but 2609.05466.
- Options: (a) the harvest names the raw file for every hand-on (`\lx@store@from`, frontmatter_stores.rs); (b) watch the bindings' other contact commands by name; (c) leave
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Unobserved frontmatter calls under a written author block (9".

### svjour3 `\institute`: the names before `\at` are no entry (pre-existing; 95b r3)

- **svjour3 `\institute`: the names before `\at` are no entry (pre-existing; 95b r3)** — svjour3.cls:506 `\def\at{\\\let\and=\iand}` prints the names before `\at` over the institute they share; sv_support's `\lx@sv@institute` keeps them as an affiliation ("A. Able" in the block beside Univ X: 2609.05466's "N. Cuellar", "A. Pereira"; int124b and r3 alike), and the reading numbers them as entry 1. sv_support is written since 97e (OD #948); the writer declines an affiliation writing one of the paper's names, or printing another author's name, that the reading gives an author (`emitter::plan::{writes_the_name, prints_the_name}`; 2609.05466, 1506.02396, 1908.03628), so today's still prints.
- Fix in sv_support's reading: the names before `\at` the entry's carriers (each matched to its author by surname, as the print sets them), then svjour3's own gate.
- Repros class-bindings/`svjour3_institute_names_at_with_authblk`, `svjour3_at_pieces_joined_are_not_written`
- State: OPEN (the reading; the writer declines)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "svjour3 `\institute`: the names before `\at` are no entry (p".

### 95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none a regression or a loss)

- **95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none a regression or a loss)** — (S1, moot since DEL5 deleted the setter watch) a setter's input mark can stay open after its expansion ends: `state.rs` `input_unread_since` (~:2436-2449) compares the mouth's depth and read position but not which mouth, and token-list digestion runs on `Mouth::default()` (position (0,0,0,0), mouth.rs:164-175), so a mark taken at the end of a digested list reads open in a later default-mouth context; it can only cost a write (a written item kept as today's, or the paper declined), never write an unwritten family's item; fix: a per-mouth serial in `InputMark` (the per-context `ctx_serial`, gullet.rs:4936-4941); unconfirmed (probe: svjour3 + authblk, `\email{..}` last in a digested argument, a later unwatched API call, the `.fmshadow.json` `binding`); addressed by RC1r2: a mark carries a serial its own mouth's watch holds (`gullet::PushbackWatch`), so a mark of a mouth since closed reads as read (guard `gullet::token_limit_tests::input_marks_nest_and_keep_no_copy_of_the_pushback`), the probe not run.
- (S2) the watch switches on in any written-family paper that defines one of the setter names itself — the `document`/`fallback` owner counts as an unwritten family (`ledger.rs` `watch_unwritten_setters` ~:494-505): article + authblk + `\newcommand{\email}[1]{\texttt{#1}}` (review probe `rev95b3/q1`, control `q2`) costs +0.28 % instructions and marks earlier calls `unobserved`; creators unchanged; fix: exclude non-binding owners from the trigger, or correct the comment and OD #942.
- (S3) svjour3's own `\mail` (sv_support_sty.rs:147-150 → `\lx@add@contact[role=address]`) is not in `ledger::FRONTMATTER_SETTERS` (~:282), so under authblk its item is still written; the list hand-copies `store_setters.rs:22` / `rehearsal.rs:271-281` — derive it from those tables.
- (S4) the row above's minimal input is unchecked (svjour3 never prints a raw `\@address`) and its scope differs from OD #942's wording: one statement in both and a pdflatex-clean RED repro.
- (S5) the census ran in default mode, so it cannot count papers where the watch was on: 9 of the 419 mixed papers have authblk under an unwritten class (2005.13982, 2106.12864, 2108.03328, 2110.11194, 2211.06456, 2305.02433, 2311.07372, 2502.07150, 2609.05466) — re-run them with the census switch (deleted in DEL4; moot).
- Nits: `writer/mod.rs:259`'s outcome "declined: … beside a re-part" is also given for an unwritten call's creator and for unpaired names; `\inst` is unwatched while a raw `\inst` store is rerouted to an API call (store_setters.rs:36).
- Review `agent_reports/2026-10-31_rev95b_r3.md`
- State: OPEN (residual)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none".

### AMS contact setters under raw amsart derivatives (next batch; 86c r1, reverted)

- **AMS contact setters under raw amsart derivatives (next batch; 86c r1, reverted)**: a raw class read over the AMS binding re-dresses `\address`, `\curraddr`, `\email`, `\urladdr` as stores only its own `\@maketitle` prints (stacs.cls:294-298, stacs_proc.cls, rmmcart.cls; m2an.cls, edpsmath.cls, pcms-l.cls:481 `\csname @address\number\addresscount@`), and our deposit of that `\@maketitle` is dropped, so the addresses are printed nowhere (0902.1737, 0802.2833).
- A lock on the four against the files (`"{cs}:locked@files"`) keeps them, but alone it imposes the binding's argument meaning on classes whose arguments differ — reverted until it ships with: (1) a stacs/stacs_proc binding: `\address[ref]{…}` an address with no name (the key is matched against `\author[ref]`, never printed; the lock made it a visible `<contact name="uniroma">`, 0902.1737's `lab1`–`lab4` in 0802.2833), and `\author[ref]{short}{long}` (stacs.cls:256) the long name as the person (today a stray long-name paragraph); (2) a pcms-l `\auth` binding (pcms-l.cls:203, reached through the paper's `\renewcommand{\auth}{\secdef\@auth\@sauth}`, pcmslmod-modified.tex:89), else the address goes to a nameless creator ahead of all (math0505518, math0602226); (3) OD #803's empty-shell drop extended to the deposit's footnotes and pubnotes whose content became contacts (m2an_mod.cls:118 `\footnote{\box\@b@ddrss@}`: an empty `<note role="footnote" mark="2">` in 2502.17950, 2309.08130, 2411.09498, math0701227; an empty `<pubnote/>` in 2305.18054, 2604.25825); (4) the lock with a wrapper opt-in that lets a file's redefinition through when it still calls the binding through a `\let`-saved copy (epigamath.sty:227-232, 2206.02674, 2304.03539, 2305.12241; MyCommA.sty:530-533, 2602.16384), where S1: the call must be real — at brace depth 0 and outside a conditional, not a mention (`sectioning-frontmatter/{ams_raw_class_address_setter_conditional, ams_raw_class_address_setter_deferred_call}`, review probes `p_cond`, `p_wrapstore`); S2: a wrapper installed with `\let\address\wr@address` takes the same test on `let_i`'s `:locked@files` path (`ams_package_let_wrapper_keeps_both`, `p_letwrap`); S3: a class's `\AtBeginDocument{\renewcommand\address…}` runs at the document's origin (OD #596) and bypasses the lock — fix or record (`ams_raw_class_address_atbegindocument`, `p_atbegin`).
- Boundaries the lock must keep: the document's own redefinition (1212.1375, 2206.01924; `ams_document_address_redefinition_stays`), a package's wrapper (`ams_package_wrapping_setters_keeps_both`).
- Repros (no guard): `sectioning-frontmatter/ams_raw_class_address_setters_stored` (RED) and the six above; the r1 code is branch `pk/86c-r1-backup` (state.rs `calls_locked_meaning`, ams_support_sty.rs).
- Its census (`ab_66bundle/pop86c.tsv`, 880 AMS-derived/redefining papers + a 292-paper amsart sample): 14 changed, also 1909.04033, 2001.07042, 2609.01974 and 2411.00367 (d-omega3.sty: E 18 → 9, `\organization`/`\orcid` still lost).
- Review `agent_reports/2026-10-28_review_86c_r1.md` (B1–B3, S1–S3); from the r2 re-review: B1's visible `name` label is what Perl itself gives 0902.1737 (it never reads stacs.cls, 1 missing file, and keeps `\address[ref]`'s key as the contact name), so the stacs binding goes beyond Perl rather than restoring parity; its guard must assert `@name` (the `creators()` helper cannot see it); the lock paths that held in r1 (`\let\address\relax` + `\newcommand`, a `\renewcommand` in a class option; probes `p_relaxnew`, `p_option`) are its boundaries; 2305.18054's thanks note loses its "∗" mark and its two-name `\author`'s address attaches to the last name only
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "AMS contact setters under raw amsart derivatives (next batch".

### Frontmatter e-mail ↔ name spelling

- Frontmatter e-mail ↔ name spelling: an address that spells an author with an umlaut written out (`ü` as `ue`, `duerrschnabel@` for Dürrschnabel, 1911.11496) is not matched, so it goes unspelled (to the block).
- The spelling helper is shared by today's output and the FM S2 shadow.
- Fix: fold the German umlaut transliterations (ä→ae, ö→oe, ü→ue, ß→ss) into the comparison when the plain fold fails.
- FM S2a.2 r6 decision 3 sample
- State: open

### Graphics search order

- Graphics search order: LaTeX's `\IfFileExists` tries the bare name in the working directory first (kpathsea, latex.ltx:1095-1102) and only then the `\input@path`/`\graphicspath` entries (`\@iffileonpath`, latex.ltx:9698), where `image_candidates` (`util/image.rs:36-41`, Perl Image.pm:46-48 alike) lists `\graphicspath` entries first and SOURCEDIRECTORY last: with `\graphicspath{{../up/}{./}}` and a `u.pdf` in both, pdflatex reads `./u.pdf`, ours `../up/u.pdf` (for the extensionless name and the `.eps`-retry alike).
- Older than 78v (int63a the same).
- Also `graphics_type_rank` ranks `.MPS` among the uppercase types, which pdftex.def:599 does not list (no practical effect).
- Review rev78v3 findings 2 and 4 (probe `rev78v3/g/sub/g3.tex`)
- State: open

### float.sty's `\restylefloat{<type>}` of a kernel float (figure, table) records the style only (float_sty.rs `record_float_style`), so without caption the caption keeps …

- float.sty's `\restylefloat{<type>}` of a kernel float (figure, table) records the style only (float_sty.rs `record_float_style`), so without caption the caption keeps the format it had: a ruled figure "Fig. 1: text" under IEEEtran (kernel ": " under article) where float.sty's `\floatc@ruled` prints "Fig. 1 text" (review rev78y probe3/rs.tex); the plain style's ": " happens to match. float.sty's own `\newfloat` types get the style's format (float_sty.rs `\lx@float@newfloat`, Perl float.sty.ltxml:50-53, whose plain format puts a space before the text and whose `boxed` style reads " " where float.sty prints ": ").
- Fix: give an unstarred `\restylefloat` the `\newfloat` type's style format.
- Also: a `\restylefloat` in the document body is not seen, since the class format is installed at `\begin{document}` keeping only the types restyled by then (caption_sty.rs `class_makecaption`, `\lx@class@floatsty@kept`): under IEEEtran `\restylefloat{figure}` before a second figure leaves "Fig. 2.
- " where pdflatex prints "Fig. 2: ", and `\restylefloat*{table}` after a `\restylefloat{table}` leaves "TABLE II: " where pdflatex prints "TABLE II" over the text (78z probe `scratch_78z/rs/rs.tex`; the preamble forms are right, guard `perfect_kernel_batch78y::ieeetran_restylefloat_table_keeps_float_sty_format`).
- 78y r2
- State: OPEN (pre-existing; found at 78y r2)
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "float.sty's `\restylefloat{<type>}` of a kernel float (figur".

### acmart copies older than TL 2025's v2.16 (v1.55-era, review rev78y acmcls/cs12.cls) set sigchi's small caption fonts on figures only, not tables; acmart_cls.rs ports the …

- acmart copies older than TL 2025's v2.16 (v1.55-era, review rev78y acmcls/cs12.cls) set sigchi's small caption fonts on figures only, not tables;
- acmart_cls.rs ports the TL version's.
- Fonts only, so behind the class caption fonts knob (caption_sty.rs `CLASS_CAPTION_FONTS`, off by default): no output difference unless the knob is set.
- 78y r2
- State: OPEN (knob-only; per-version port not planned)

### A preload's options do not reach a package another preload already loaded, in either order, so until 78z the class caption fonts knob could not be combined with ar5iv.sty

- A preload's options do not reach a package another preload already loaded, in either order, so until 78z the class caption fonts knob could not be combined with ar5iv.sty: `--preload=[classcaptionfonts]latexml.sty --preload=ar5iv.sty` sets the knob but loses every option ar5iv.sty passes to latexml.sty (`ids`, `rawstyles`, `rawclasses`, `magnify`/`zoomout`, the limits — `\documentclass{paper}` falls to OmniBus with `Warning:missing_file` where ar5iv alone reads paper.cls raw; the magnify PIs and `<p>`/autoref ids gone), and `--preload=ar5iv.sty --preload=[classcaptionfonts]latexml.sty` sets no knob (no `\ds@classcaptionfonts` runs).
- Mechanism: `core_interface.rs:1085-1092` puts a preload's options into `opt@<name>.<ext>` only, `InputDefinitionOptions.options` stays empty, so binding/content.rs `apply_repeat_options` returns at its first test (`content.rs:602`); the repeat load is not re-read.
- Perl alike in effect (Core.pm:333-335 passes `options =>`; a repeat load only reports the "Option clash" Info, Package.pm:2586-2590, and is not re-read, Package.pm:2357).
- Relaxing every declared `\ds@` handler after `\ProcessOptions` is Perl's (Package.pm:2479-2481, ported at `content.rs:3339-3342`; latex.ltx:18566-18568 empties only the used ones) and not the cause.
- Fix for the knob (78z): ar5iv.sty declares `classcaptionfonts`/`noclasscaptionfonts` and passes the one given to latexml.sty with its own options (`ar5iv_sty.rs` `pass_options`), so `--preload=[classcaptionfonts]ar5iv.sty` sets the knob and keeps ar5iv's options (OD #884).
- Pre-existing; reachable since 78y r2 (review rev78y2 finding 1, probes `k1_*`, `k2_*`)
- State: the class caption fonts knob FIXED 78z (guards `perfect_kernel_batch78z::{ar5iv_classcaptionfonts_option_sets_the_knob, ar5iv_classcaptionfonts_option_keeps_ar5iv_options}`); the general repeat-preload behaviour OPEN
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A preload's options do not reach a package another preload a".

### Determinism (det354 B2)

- Determinism (det354 B2): the `xmlns:*` declarations on the root `<document>` are in the order of the model's namespace table (`common/model.rs:1244-1254`, a `SymHashMap` at `:55`, → `document.rs:774-830`), not the order the document first uses them, as Perl declares them (Document.pm:1393-1394,1874): `\usepackage{fontawesome,tikz}` then `\faTimes … \tikz\draw (0,0)--(1,1);` declares `xmlns:svg` before `xmlns:aria`, though `aria` is used first (trigger det354 `repro/ns.tex`; Perl declares `svg` only).
- Stable today across int32a–int64a, since the symbols are created at init — a latent binary-to-binary instability, and golden churn on every SVG+aria document when changed.
- Fix: a document-order walk, with the prefixes of streaming-spilled content sorted after it; or sort by prefix (simpler, not Perl's).
- Not in 78w (main-session decision 2026-10-22)
- State: open
- Full row (verbatim): [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Determinism (det354 B2)".

### appendix.sty

- appendix.sty: `\appendix` inside `{appendices}` does not restart the letters (pdflatex "A" again for the next section, ours "B": the kernel's `\@appendix` is `\relax` there; 2112.09636);
- pinned as ours by `perfect_kernel_batch79d::cleveref_appendix_inside_appendices_names_appendix` (review rev79d2 finding 8)
- State: OPEN (pre-existing)

### cleveref's `\appendix` type under TeX Live's LaTeX first aid

- cleveref's `\appendix` type under TeX Live's LaTeX first aid: TL 2025's first aid (`\firstaid@cref@updatelabeldata`) drops cleveref's appendix aliasing, so TL 2025 pdflatex prints "section" after `\appendix` where first-aid-off pdflatex prints "appendix" (the oracle: the witnesses refer by `\ref`, so their PDFs never print the type; 2503.13751, pdfTeX-1.40.25 / TL 2023, only dates the kernel);
- 79d follows the latter (main-session decision 2).
- Decide by the build's TeX Live year if arXiv's builds move to a first-aid kernel (79d r2 writer's question 2)
- State: open, low

### Two adjacent `\vec`s (and `\mathbf`s) merge into one bold identifier by the kernel's `\mathbf` merge rule

- Two adjacent `\vec`s (and `\mathbf`s) merge into one bold identifier by the kernel's `\mathbf` merge rule: 0812.2480 `e^{-i\vec{k}\vec{r}}` → one `<XMTok font="bold">kr</XMTok>`, 2305.06325 `\vec{g}\vec{z}` → "gz", `\vec{z}\vec{p}` → "zp";
- `$\mathbf{k}\mathbf{r}$` gives "kr" in base (probe `rev79b/kr/`).
- Unlike the accepted `\vec{kr}` ruling, two vectors written separately lose their implied product;
- renders the same, a parser-semantics question (review rev79b (`agent_reports/extracted_356a0c47/2026-10-23T024112_a8b3383dff3c35201.md`, probes `~/.claude/jobs/356a0c47/tmp/rev79b/`), finding 3)
- State: OPEN, needs a ruling; math parsing parked (no change)

### euler.sty:47-57 and fourier.sty:77-87/104-114 declare the capital Greek letters `\mathord` (class 0) from their upright fonts, so in TeX `\mathbf{\Gamma}` keeps the …

- euler.sty:47-57 and fourier.sty:77-87/104-114 declare the capital Greek letters `\mathord` (class 0) from their upright fonts, so in TeX `\mathbf{\Gamma}` keeps the medium-weight Euler/`futm` letter (and `\mathit{\Gamma}` stays upright; mathptmx's, mathpazo's and eulervm's `\mathord` `\up…` the same);
- ours follows the kernel's `\mathalpha` rule (`declare_capital_greek(…, None)`: `\mathbf` bold, `\mathit` italic), as Perl and int75a do — no regression, no fixed-upright `LettersClass` (review rev79e finding 3, probes `~/.claude/jobs/356a0c47/tmp/rev79e/`)
- State: open, low (not a regression)

### mathastext's Greek options (`symbolgreek`, `eulergreek`, `LGRgreek`, `selfGreek`, `itGreek`) are not modelled — its binding is a no-op for the capital Greek letters (79e …

- mathastext's Greek options (`symbolgreek`, `eulergreek`, `LGRgreek`, `selfGreek`, `itGreek`) are not modelled — its binding is a no-op for the capital Greek letters (79e r2 report);
- no census paper combines mathastext with the italic classes;
- a `\mathord` upright letter (mathptmx's `\up…`, amstex, euler) under `\mathit` turns italic here where TeX keeps it upright (no fixed-upright rule; the row above's gap)
- State: open, low

### The kernel's `\DeclareMathSymbol` gives a `\mathalpha` (class 7) symbol role `ID`, where Perl's (latex_constructs.pool.ltxml:2697-2705) gives none

- The kernel's `\DeclareMathSymbol` gives a `\mathalpha` (class 7) symbol role `ID`, where Perl's (latex_constructs.pool.ltxml:2697-2705) gives none: a raw class/package copy's capital Greek letters (and any `\mathalpha` glyph) parse as `ID`, a binding's (`declare_capital_greek`, the kernel's `\Gamma`) as `UNKNOWN`.
- Predates 79e (55h);
- a parser role, math parsing parked (79e r1 writer's question 2, review rev79e decision 2: leave it, track here)
- State: open, math parked

### 25, 26, 27 — `--splitindex`/`--splitbibliography` (non-default), alignment open at `\end{document}`, lstlisting in `\parbox`

- `--splitindex`/`--splitbibliography` (non-default), alignment open at `\end{document}`, lstlisting in `\parbox`
- State:
  - not silent, re-checked on head 2026-10-19: #26 is an honest Error (a `tabular` left open, `c & d`: 4 Errors; an `align`, `c&=d`: 3 — "Attempt to close a group that switched to mode internal_vertical"; invalid input, pdflatex errs);
  - #27 has raised 1 Error since 64zf (CONTROL `kernel-alignment/pcol_lstlisting_parbox_CONTROL`, pdflatex 7);
  - #25 needs a non-default option the arXiv runs never pass — open, low reach, not blocking

### Before the flip: the switch-on leftovers (one batch after the safety net's; all `LATEXML_FM_EMIT=2` only, the pre-flip default output unaffected; `=2` the default since …

- The whole row, live: its history (DEL1/DEL2, LO1-LO4, E1, items 1-39) sits beside its open items and REDs; its verbatim copy in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Before the flip: the switch-on leftovers".
- **Before the flip: the switch-on leftovers (one batch after the safety net's; all `LATEXML_FM_EMIT=2` only, the pre-flip default output unaffected; `=2` the default since item (38))** — OPEN.
- **DEL1: `=0` is gone from the tests and the switch is fixed at `=2`** (`fm_emit_mode` a constant, `LATEXML_FM_EMIT` unread; OD #980): the `red_at_two` guards asserting only `=0` deleted with their orphaned fixtures, the guards that assert something at `=2` first keep it and lose their `=0` tail, the explicit `=0`/mode-1 halves removed, `expect(off, two)` → `two` (the RED rows kept in `FRONTMATTER_REDS_AT_EMIT2.tsv`, marked).
- **DEL2: the switch and the old driver are deleted** (OD #980): `fm_emit_mode`/`set_fm_emit_override` gone, every `=0` arm collapsed (emitter, resolver `emitting`/`whole_entries`, rehearsal, shadow `reads_now`, ledger overflow, `frontmatter_stores`), `add_authors_calls_sectioned`/`split_author_calls`/`line_kinds.rs` and every `author_block/`/`fallback.rs` item only they reached removed by the compiler (the IJCAI split kept as `author_macros.rs` `ijcai_split_call`); byte-identical on the int152c stage2 ids; the e0 fence's pins lowered.
- LO1 (OD #980, guards `perfect_kernel_batchlo1`) did: a link filling a name slot is a name (2602.22747); a superscript after an emitted entry's label is its text (1812.07638's `(ECT$^*$)`); an address decides only its own ink (2203.10777, 1210.0352, 1605.03019); a list of local parts opening its line is its authors' addresses (2003.14069); a creator call's `annotations=` are its creator's labels, written once (`k_annot2`).
- LO2 (OD #980, guards `perfect_kernel_batchlo2`) did (3b), (8), (10)-(12): a row line's address spelling one author is that author's e-mail, the rest the row's, cut from the tokens inside the ink (2203.10777; `r_pres_spelled`); a note after a labelled entry is its holders' (2504.09745); `annotate={made}` is `annotate=all` where the call made every queued creator (`annotate_made_reaches_every_name`); the first-mark fact a key on the pending entry, `emitting` = `=2`, `FieldCreator::labels` cleaned as `cleanFrontmatterLabels`, one write path for the ledger's `generation`.
- Left: (3) **1812.07638's `Contributors:` heading** (a line between two names rows) read as the six editors' affiliation: a role heading is its names row's label, which the API has no kind for yet (options: a note of the block, the row's creators' contact of a heading role, or left as today).
- (3c) an in-block `$^1$Univ (ECT$^*$) and …` splits the entry at the symbol with either switch (a symbol glued inside open brackets in the lexer's `glue_in_word_marks` fixes it, and moves the switch-off output).
- (5) **BESIII's sub-labels** (R6, tentatively ruled): 2204.09614, 2404.06718 leave 48 and 51 words silent (`68A,68C` answered by entry 68).
- (6) **1810.03464's rehearsal loops** past its read budget (vldb's `\normalsize` in `\email{…}`; trigger in its own row) — FIXED by PC3 (int155a, OD #983: a class's own `\email` read unexpanded; 1 → 7 creators).
- (7) **The tabular grade** `…, Fellow, IEEE\end{tabular}` reads `\end{tabular}` into the grade (E=2, off too).
- (9) **The tests under `=2`:** 701 of 8,173 fail on int147a with LO2 (before T1, which re-expects 31: item 16); N2r3 alone 712 of 8,173, 139 with the content check's Warning in their output; each read and fixed or re-expected before the flip.
- LO2r2 (OD #980): a note after a block's labelled entries, or on a line under one, is the block's, one pubnote as Perl (2504.09745); a row line's cut ends at its address's own first token, every ink before it kept, and every address must end the line; a spelled label (`{\rm Ed Office}`) cleaned on both sides; the ledger's pre-scans run only at `=2` (`Ledger::change_each`); the switch-off output stays byte-identical, but its cost is +0.18 to +0.22 % `instructions:u` over LO1 on 1507.02068, 2103.04797, 1901.01540 (from LO2; revLO2r2 names only small per-call off-path code: `ledger.rs` `record_api_call`/`note_field_stamp` `rposition`, `record_clear`, `blocks.rs` `labelled_entries`' count, `queue_digest.rs:1516`) — to bisect (`kit/perf_bisect.sh` over 3205f4ba81..df0926bd9b).
- (13) From revN2 (SHIP), N2r2 and N2r3 did (OD #980, guards `perfect_kernel_batchn2r2`, `perfect_kernel_batchn2r3`): the rehearsal's names no longer reach the output (2106.06418's svjour3 `\email{yjansson@kth.se \and tony@kth.se}` wrote `\lx@fm@obs@andtony@kth.se` at `=2`; 1401.5951's note text, 2106.11154's link target: the bound constructors revert as their commands, a verbatim argument's `\and` parts its addresses); the content check counts a shared item as often as the inks print its text, at least its most copies on one creator and at most its copies (the ruling on N2r2's question (a): a line printed under each author and kept as one shared item is no loss, a word lost beside a shared item printed once still is, `m1.tex`; each print taken once, the longest entries first, cond-mat0305063; a print read by its words, 2311.03657), a contact's `name` label is never its text's (2307.11138), a block written whole exempts only its own creator, the Warning counts its words, every word counts with a mark excluded by its event; under `=2` the flush reads every paper as the census does, so the check reads a paper's creator calls with no author block in production too (llncs's `\institute`, 1401.5951: 0 findings without `LATEXML_FM_SHADOW=1` before), and the `=2` XML is byte-identical with and without SHADOW (287 dev3 papers, N2r2-rel).
- Measured at `=2` without SHADOW (dev3 10k / devh10k, papers flagged lost, excess: 108/157, 159/204; N2r2 547/158, 543/202; N2 68/155, 104/193; read 20 each: lost real 10 + 2 unsure / 6 + 3 unsure, excess real 12 / 16 — the excess's real ones mostly an address a note keeps that the emit also writes as a contact).
- Real losses it finds: jheppub's `\note` after a name dropped (1403.1099, 2001.06318, 2308.04217), an `\email` in a `\thanks` (2511.14713), a second author's lines read as the first's affiliation (2209.10660, 2006.14932, 1808.07977), ORCIDs in `\inst{…}` (2405.10871).
- Left, counted as false-alarm classes: (13a) a shared item whose text no run of prints spells (a grade inside a names line, `Member, IEEE`: 2310.13917, 2606.23126; a label inside its line, `(e-mail: …)`: 1003.2485) — proposal: count a shared item's prints by the reading's entries (their spans), not by matching its text; (13b) an accent whose base the rehearsal prints as a Name of its own (`Bo$\check{c}$ní`, 1603.04461, 0805.0350, 0911.1045, 2310.11912, cond-mat0103427, 0705.1518; `\cb{t}`, 2302.10027; `\'\i`, 1306.2567): one word needs the events composed; (13c) what the rehearsal doesn't record: an ORCID icon's drawing (a `tikzpicture`'s path data as a Contact's text: 2502.20521, 2411.08097; dev3 14 / devh 17 papers) and a label a class prints with no `name` (`ORCID: `, a class's `Email: ` inside a note: 2307.11138, 2402.16530, 2405.08537, 1602.07570; ~9 papers); (13d) one ink read two ways: local parts in `\href`s inside braces (2505.15472, 2108.00192, 2411.09694), an ordinal (`24\textsuperscript{th}`: 2202.05365, 1806.03501, 1402.0794), a mark or label glued (1509.04564, 1510.04162, 2512.16476), a domain list (2501.11453), `\underline{ }` in an address (1404.5693); (13e) a class's `\and` printing "and" (dev3 10 / devh 16 papers: 1709.02726, 1703.10540, 2404.11208); 1401.5951's thanks as TeX source.
- SolarPhysics' raw `\author` (1808.10103, 1306.3180): deferred — off writes the same (0 creators, the authors in body paragraphs, nothing lost); the structure diagnostic's (bar 4).
- From revN2r3 (SHIP): (13f) RED, a lost line with a shared item's words is covered by its copies (`Prints::take`, `resolver/placed.rs:470-499`: `\author{Ann Able \quad Bob Baker\\ IDSIA \and Cat Cole \and Dan Dorn\\ \small{IDSIA {\{cat, dan\}@idsia.ch}}}` drops Cat and Dan's IDSIA with no Warning; `scratch_revN2r3/t/q1e.tex`, `q1h`): count prints per column of the item's holders, or from the reading's entry spans (13a); (13g) RED, a contact's default kernel label (`Affiliation: `, `Email: `) counted as printed ink (`placed.rs:384-386`; `t/q4f.tex`): add a label's words only where `labelled()` stripped them off the text; (13h) jheppub's `\note` lost at `=2`, kept off (1403.1099, 2001.06318 `Corresponding author.`; 2308.04217 "Speaker at …"): done by LO3 (OD #980, guard `perfect_kernel_batchlo3`: a block's Contact event of kind note is written as its contact command; dev3 + devh reach 0 papers, none of their six jhep-family papers has a `\note`; at `=2` 670 of 8,181 tests fail on LO3, seven fewer than on its base, `batch66h::revtex_frontmatter_footnote_on_two_authors`, `batch78e::jinstpub_frontmatter_is_jheppub_s`, `batch78y::jinstpub_with_class_caption_fonts` and four `batch78f::jheppub_note_*`, none new).
- LO4 did the leftover: the one `\emailAdd` address left over goes to the author marked as the corresponding author at `=2` as off, the switch-off step's facts read on the reading (OD #980 LO4 (b), `Resolver::listed_corresponding`; 1901.08773, 2510.11030; guard `perfect_kernel_batchlo4::a_listed_address_left_over_is_the_marked_authors`, six `batch78f` repros' creators at `=2` = off).
- Still `=2` ≠ off on the sequenced lists (other steps, each a `batch78f` repro): a legend `\affiliation[*]{Corresponding author}` an affiliation, not its carriers' note (`jheppub_emailadd_corresponding_author_star_legend`, `_note_owners_are_a_set`, `_two_marked_authors_stay_the_blocks`; the addresses right); `\emailAdd{zq1@x.org, bbaker@x.org, wy7@y.org}` one call, not read by the address (`jheppub_emailadd_one_call_lists_the_authors`); `\emailAdd{\mbox{a, b}}` given to the author one address spells (`_markup_list_kept_whole`); iopart's `\ead` under its address's mark (`iopart_ead_under_its_address_mark`); (13i) an address keeping its `}` (2410.15681, 1401.5951), a false lost/excess pair.
- From revLO2r2 (SHIP): (14) **an `\href`/`\url`/`\email` address in brackets on a row line** (`Dresden, Germany (\href{mailto:bob.baker@a.org}{bob.baker@a.org})`) leaves `Dresden, Germany (` with the `)` lost at `=2` (LO1 `=2` lost the `)` too; off keeps both): the address is its own event, so `resolver/blocks.rs:711-733` must strip the opener as `emitter/repart.rs:199-261` does.
- (15) **two addresses on one row line** (`… (bob.baker@a.org); Leipzig (bob@b.org)`): `=2` gives each author the whole line where off gives each their own e-mail (LO1 `=2` the same); `batchlo2::a_row_address_cut_keeps_its_ink`'s `two` block pins the `=2` shape, worse than off.
- Pre-existing at `=2`: `Dresden, Germany\footnote{Visiting.} (…)` gives Bob alone the note (off a pubnote); an unlabelled `\and` column's thanks is Ann's (off a pubnote); `$^{*}$\thanks` on Ann's mark drops the `*` (counted).
- Also left: a note inside an entry with ink after it a nameless note as before; `annotate=all` for a raw tail under `\and` columns (pdflatex prints it under the last column only; `=2` = off = Perl, a fidelity question).
- (16) **The tests at `=2`** (policy, user 2026-11-08: measure reach, port first; guards for shapes with no measured reach stay named REDs failing only at `=2`, fixed after the flip): 701 of 8,173 fail at `=2` on int147a (sort `tests_sort/sort_int147a.tsv`: X 415, W 230, T 54, P 2); **623 of 8,250 on int150a** (`tests_sort/int150b/sort_int150b.tsv`, report `agent_reports/2026-11-08_tests_sort2_report.md`: X 336, W 229, T 56, P 2; 87 of int147a's pass now, 9 newly fail since int147a with unchanged bodies — content-check "more often" `100b::an_address_note_in_an_affiliation_is_its_email`, `77t::llncs_author_e_mail_line_ends_a_row`, `77t::llncs_author_line_break_affiliation`; "missing" `85b::llncs_listed_local_parts_keep_their_braces`, `z2::a_declarations_lead_is_invocations_not_positions`, `z2::a_font_switch_declares_nothing_and_a_name_is_no_reason`, `z3::a_declarations_span_is_a_balanced_run`; X `100b::an_address_line_holding_another_call_stays_whole` (an empty orcid contact deleted), `z14::an_entry_whose_cut_fails_is_given_back` (an e-mail becomes an address); single causes: the content-check Warning 124 tests, the plain-text Warning 83, the writer outcome 42; the 9 read (`agent_reports/2026-11-08_ts9_report.md`, fixtures and PDFs `scratch_ts9/`): **4 the check's false counts** (the XML = off = the PDF; the printed side misses an authblk `\affil{…\thanks{}}` note's words, an llncs `\email` inside `\author`, a braced list's trailing plain address — `resolver/placed.rs:225`, `resolver/words.rs:65-88`; 54 of stage2's 1,434 `=2` papers raise "more often", 24 "missing"); **3 content losses at `=2` older than int147b, newly caught by the check** — a name invoking a macro the block defines loses its expansion (`z2`, `z3`: "Ann 1" off, "Ann" at `=2`; and a font size leaking onto the rows after it), `emitter/emit.rs:1678` `declarations()` replayed before the creators while the invocation's text never reaches the name; **1 order** since int149a (`100b` orcid: the cut e-mail written after `\orcidID`, `emit.rs:1094` / `emitter/address_cut.rs`; de-nesting kept); **1 right at `=2`** (`z14`: each author's own e-mail, a per-mode pin; its contact name the source `e--mail:`, not the printed `e–mail:`)).
- T1 read batch63 and batch77p's 101 X rows: 31 as right at `=2` (re-expected per mode, `perfect_kernel_batcht1::expect(off, two)`), 70 wrong at `=2` (`tests_sort/T1.tsv`), by mechanism: **G 12** a block's link line goes to the row above (`resolver/blocks.rs:454-456`, `R12:row-above`; the block, or its `\and` column, ruling 2026-10-20); **D 10** an address cut from a labelled/linked line loses its link and font (`emitter/emit.rs:1039-1048`); **P 6** a link printing an institution becomes a URL (`events/mod.rs:1425`; old `author_block/links.rs:163` `link_prints_an_institution`); **J 6** e-mail elimination per line (`blocks.rs:127-131`, R10), an owner spelled in a later group ignored, a role mailbox placed by position; **F2 5** a name and an institution split by word reading (rev94a B1; `\author{Ann Able, John Center}` loses an author); **A 4** a prefix-marked names line after the first read as an affiliation (`blocks.rs:289`); **C 3** class conventions (cas affiliation before `\author`, `api.rs:761-777`; svjour3 numbered institutes, `post.rs:1410-1420`); **R 3** an unmarked line glued to a symbol-led one (`blocks.rs:571-586`); **K 3** rows of institutions read as names (`labels.rs:383`); **B 4** answered marks left on names (incl. `Bob Baker‡`); **N 2** a link's label as the contact name (`insert.rs:1017-1022`); **E 2** `\emph`/`\em` lose italic; **F1** a bare `\dag` raised to `<sup>` (pdflatex baseline); singles incl. content losses (`author_markup_title_code_forms` " b Given", aero `(🖂)`, icon rules, `symbol_led_affiliation_run`).
- Nit at the flip: `ieee_membership_own_definition_kept`'s header and OD #967 ("a document's own `\IEEEmembership` definition keeps its meaning (63l)") become false.
- (17) **The content check's other real losses, off against `=2` (LO3):** `=2` only — (a) done by LO4 (OD #980 LO4 (c), guard `perfect_kernel_batchlo4::an_email_call_in_a_note_is_its_owners`: a binding's `\email` in a note its owner's e-mail, OmniBus's locked `\email` the binding's; 2511.14713 `=2` creators = off, its Warning gone): an `\email` in a `\thanks` (2511.14713, siamart: `\thanks{…, \email{wning@vt.edu}, \email{amiedlar@vt.edu}.}`; minimal `\author{Bob Baker\thanks{Univ B, \email{cy@b.org}, \email{dy@b.org}.}}` in siamart, both lost): the rehearsal reads OmniBus's locked `\email` as the document's own (`ledger::email_meaning`, `EmailMeaning::Own`; `resolver/words.rs:316` records it for 1804.09455), the reading keeps a note's links that spell no one when there are several (ruling 2026-10-20 #4, `Resolver::note`), and the emitted note mutes the API call (`\lx@frontmatter@itemonly`), so the address prints nowhere; off and Perl give it to the note's owner, as the API routes it (options: the binding's meaning read as a binding's and its links the note owner's, as the API; the link printed in the note's text, as ruling #4 says; left).
- (b) a second author's stack after `\\[2ex]` read as the first's affiliation (2209.10660, article: `\author{Ann Able\\ Univ A\\ ann@a.org \\[2ex] Bob Baker\\ Univ B\\ bob@a.edu}`; off parts the stacks at the spaced break, `=2` reads "Bob Baker" as an affiliation and keeps his lines once): the Break event records no skip, a fact the rehearsal would need first.
- Both — (c) 2006.14932, 1808.07977: the second and third authors' lines are the first's affiliations with either switch (`Anna Duyunova,\\ …\\ Valentin Lychagin,\\ …`; all-caps `ZHIFENG ZHANG`); `=2` only writes a repeated affiliation once, which the check counts.
- (d) 2405.10871 (llncs `\inst{1,\dag,\orcid{0000-…}}`): no ORCID with either switch, and the `\institute` list is cut with neither: off gives it whole (every institute, the legends, `{spbakas@iu.edu}`) to the seven authors whose `\inst` lists `1`, the others' marks left on their names; `=2` gives it whole to all 22, their names clean (an excess the check counts).
- (18) From revLO3 (SHIP): a contact command in a span printing several events (a macro holding a name and `\note{…}`, two `\note`s, `Dan Dorn\emailAdd{…}`) wrote the whole macro per item at `=2`: done by LO4 (OD #980 LO4 (a), guard `perfect_kernel_batchlo4::a_contact_command_in_a_macro_is_its_text`, its text, counted); left: an address so written loses its typewriter font (the text path has no font for a contact command's ink; P3's cut inside a macro's expansion fixes it).
- LO4's reach (all three): dev3 + devh `=2` (1,317 papers) byte-identical to LO3-rel `=2`.
- (19) From revLO4 (SHIP), synthetic REDs: an `\email` in one author's `\thanks` spelling another author goes to the note's owner at `=2` (`resolver/blocks.rs:1131-1134` `routed`, `post.rs:775-778`; `scratch_revLO4/ps/q3_siam_spelled.tex`, `q3_siam_names_bob`, `q3_revtex_names_bob`) — ruled (main session 2026-11-08): the charter's spelling rule decides, the spelled author, as off; an apart `\note`'s counter number marks its author (`p/q1_apart_num.tex`: the page marks Ann; off and `=2` give Cat, Perl Ann) — number apart notes in `listed_corresponding` and `place.rs:236-258`; a macro span's address written as plain text loses its typewriter font with no Warning (`emitter/emit.rs:1061-1069`).
- (20) P2 (OD #980 P2, guards `perfect_kernel_batchfmp2`; the readiness measure's NT, C and B, `agent_reports/2026-11-08_RD1_report.md`) did, `=2` only: cas's one unlabelled `\affiliation` every author's (T1's C for cas; 2506.17280, 2106.02306); a role note the note off writes — a `\thanks` an affiliation entry only where `\lx@add@thanks` makes one (a superscript number), never a `\footnote`, else its mark's carriers' legend (1107.5730, 2205.02029, 2110.12892), the label that prints carried to the Kind step for a legend (authblk `\affil[*]`, 2605.31330, 2201.11910; a numbered `\thanks` legend); a name cut out of a font wrapper holding its mark and note, and a note its name's tokens print written once (T1's B; 2309.13682, 2212.02936; revLO4's REVTeX `\newcommand{\ann}{Ann Able\thanks{…}}`).
- Left: a legend whose address the reading leaves in its text waits for E1's cut (2605.31330's `(Email: …), Le Hong Trang` and 2411.08381's `Correspondence to: a and b` still affiliations; 1310.0379, 2403.10709, 2601.11860, 2511.06673 their role note right, the addresses left in its text; P2r2 ported the note kind strictly (off's order) and kept one `=2` reading: a mark-led note printing e-mail addresses, no legend, stays the reading's entry, each address its spelled carrier's (1610.05419, 2405.06709; guard `a_thanks_note_printing_addresses_keeps_its_entry`); RED, no guard: `\thanks{$^{a}$Department …}` (a letter mark, no address) a note on its carriers where LO4 `=2` read an affiliation, = off); 2602.07769 (`\IEEEauthorblockA{\textsuperscript{1}…\\ \textsuperscript{2}…}` inside an article `\author`, the entries unsplit, marks on the names: KEY, not B); 2305.16982's `xuta@microsoft.com` the block's (an address list's elimination, J).
- (21) From revP2r2 (SHIP), synthetic REDs: RP2r2's rule 6 (a mark-led note printing addresses keeps `=2`'s entry reading; ruled) decides by "prints an address", not the text (`resolver/blocks.rs:1267`; `scratch_revP2r2/probes/q8_email_only`, `q7_ieee_noaddr`, `q6_footnote_addr`): apply it only where the text, addresses removed, reads as an institution, and say in OD #980 that it reaches footnotes; an unspelled address on a symbol-led note to a nameless creator, where the charter gives it every carrier (`q3b_stray_addr`, pre-existing); the digit test broader than `starts_with_affiliation_mark` (`q1f_zero_brace`, = the PDF; a benign departure to name); the letter-mark `\thanks{$^{a}$Department…}` a note as off (LO4 `=2`: an affiliation) — all for the kind rule table.
- (22) **E1 (RD1's 42; P1b + P1br2, OD #980 P1b, guard `perfect_kernel_batchp1e`):** RT1's rows 54-58 ported onto the tokens (`events::InkTable` beside the block's events, `emitter/address_cut.rs`), row 57a's decline per entry; P1br2 (review revP1b): the label the Kind step for an address line reads before an address goes with its run (`address_line_cut` in the cut, `labelled`: 2305.02090, 2404.05566, 2107.10505, 1510.03749, 1807.09187, 2008.08137), a marked `\affil[*]` stays its carriers' legend note (1812.03816), row 56's wrapper and the font each `@` prints in (2608.15371, 2405.06709, 2608.14026, 1312.5468), a mark after the cut kept (S1); 27 of RP1's 28 must-keep `=2` = off (1705.06453 keeps a ` -` the reading's cut leaves), RP1's four dev regressions = LO4 `=2`; the 152 keep-better papers at `=2` against LO4 `=2`: 141 identical, 11 changed, none worse.
- Left (still worse than off, the reading not taking the address): 1703.06483 (one underscore address left), 2010.15509, 2609.34378 (`Co-correspondence:`), math0603155, 1812.07023 (a `{…}@x` list read as a url); 2201.10239 an affiliation's `b` mark lost in the cut (P1b's; its correspondence note now right); synthetic: `\affil[1]{Univ X, Contact e-mail: a@x}` keeps `Contact` (the Kind step declines the line, = off), `Correspondence to: a@x` on a line keeps its label (the Kind step reads no label there, = off).
- P1c (review revP1br2, guard `perfect_kernel_batchp1c`): the label decision is the take-out's (`resolver::labels::take_labels`), the emitter's rewrite of the reading gone; a whole label line cut keeps the entry's other breaks (2604.06805, 2509.07459).
- Left: 2510.10393 keeps the `;` before its cut address (`Switzerland;\linebreak\tiny{…}`, off drops it), 1703.06483's third unlabelled address (`tayyaba\_ jabeen92@…`) stays on Shi Jin's affiliation without its label — neither falls out of the move; an icon in a cut address's link or wrapper goes with the e-mail (RED `emit_a_cut_never_takes_an_icon`, review revP1br2 S3: needs the link's characters' own positions).
- (23) From revKEY1 (SHIP; OD #980 KEY1, guards `perfect_kernel_batchkey1`), RED: a name's mark printing a macro's letter beside a number (`$^{\a 1}$`, `\def\a{{a}}`) is read as the one label "a1", so the address `$^a$` leads is no name's (hep-ph0011101: off and `=2` give it to Peris by the annotate default, a nameless creator in the minimal; repro `macro_letter_in_a_name_mark_is_its_own_label`): the mark's labels should be `a` and `1`, as it prints.
- (22) From RC1, RC1fix, RC1r2 (the content invariant at `=2`, OD #980): the input mark is a per-mouth low-water watch (`gullet::PushbackWatch`, no copy, no `unsafe`; guards `gullet::token_limit_tests::{an_input_mark_tells_a_refilled_slot_from_a_peek,input_marks_nest_and_keep_no_copy_of_the_pushback}`, `perfect_kernel_batchrc1::*`); left: a letter command (`Gro{\ss}e`, a control word) parts a word of an unread call's text (`ledger::content_text`), a lost word counted as two; the accent fold in the placed check (`resolver::placed`) has no fixture — a stand-in copy's unrecorded creator call (`Ledger::unrecorded`) was not reached by synthetic copies of arxiv.sty or jheppub.sty (their `\author` replaced), and two inside calls past the check's allowance not by revtex's `\collaboration`; `emitter::emit::emit_commands` builds the unrecorded texts with the emitter off too (2103.04797: every queued creator call's), which only the `=2` check reads.
- (22) NL1 (OD #980 NL1, guards `perfect_kernel_batchnl1`; rule table RT3, `agent_reports/2026-11-08_ruletable_namesline.md`) ported, `=2` only: a marked piece is a name whatever its words (F2: 1901.10843, 0904.4122, 2103.02641, 2311.16860), a collaboration remark the names' contact read whole across notes, note references and boxes, and a line speaking for one under one name (COL: 0711.1544, 1212.4278, 1310.6507), glued initials are no bare surname (NS: 2409.13569), a comma one command draws inside a word parts no names (CB: 2507.13598).
- NL1r2 (revNL1's B1, S1-S4): a remark's text joined as the source joins it and its own span after the name's note (`lines::printed_join`, `remark_start`; 0711.1544, 0804.0020, 1201.5130), a kern backing up no space (`\babar`), a break a command prints after its box a break, C5's contact at the line's span (1502.03246); **K** ported behind "the line above holds one labelled entry" (the main session's decision: 2501.04046, hep-ph0511130, `Cat Cole \\ Physics` kept), with an address several names spell the block's unless the corresponding author is one of them (`perfect_kernel_batch64::marked_block_email_line_owner`, which K's reading reached through the old writer).
- Residual, no guard: **cs0011034** (`$^1$Dept … and $^2$Centre for Computational Linguistics\\ K.U.Leuven`, two entries above, as Cat Cole's) keeps a junk creator "K.U.Leuven" at `=2`; one-entry / two-entry is the only fact the reading holds to part them.
- The concession's give-back of trailing marks (`repart::give_back_ink`) has no live shape after N1-N3 (its three batch94a guards read the reading's names; the prefix-mark in-doubt branch guarded) — dead-code removal is the main session's call.
- **1208.2323** (item 3, "(The OGLE Collaboration)" every author's affiliation) unchanged, its site unmapped.
- RED, no guard: an ordinal superscript in a name (`J. M$^{\underline{\mbox{a}}}$ Ib\'a\~{n}ez$^1$`, "Mª", astro-ph9802333) ends the name with either switch (repro `emit_an_ordinal_superscript_inside_a_name_is_its_text`).
- (24) From the int150a fix rounds' reviews (revG1-revG5, revBC1, revH1-revH3; all SHIP), REDs, synthetic, no arXiv witness, each a probe in the review's scratch: a marked `\thanks` piece no name's mark asks for (`$^{\dagger}$deceased`) goes to a nameless creator at `=2` (off: the `\thanks` owner's affiliation; intended: the owner's note); the `*` piece after `$^{2}$Univ B.` is Bob's, though Ann carries the `*` (off the same; revG5 1); a bare `Smith\\Univ` reads as one word in the content check (RC1e), and a lost word a macro also prints (h4i) is missed (OD #980); authblk's `\thanks{…$x^2$}` rehearses as "x2" with no Mark, a counted false W=2 (RC1f), as an `\email` in a creator call's `\thanks` warns "appears more often" (revH2 2); a user `\inst` macro in a creator call's `\thanks` writes the name as the note (revH2 1); a `\thanks{…\protect\label{}}` in a creator call keeps no `labels=` (off too, revH2 3); an institution column between named columns is a creator (h2h), a CJK names row leaves a trailing university a creator (h2k), a `\textbf{and}` between juggles is dropped with W=0 (h3a), a bare group or an `\hskip` between a juggle's close and reopen loses the next column's affiliation (h3f, h3e; the check's blindness there is FS2's); NL1f's n10 places the `\thanks` note after the collaboration contact (off before) and n9's `$^{1}$` is lost (off too); unlabelled non-block orphans are dropped at `=2` with a Warning (revBC1 3, OD #159).
- Parity gap, outside the frontmatter: our raw-line readers (`Mouth::read_raw_line`) and the native l3fp readers do not drop a marker token as every Perl reader does, which ruled Perl's marker out for the input mark (RC1r2, OD #980).
- (25) IO3b (OD #980 IO3b, guard `perfect_kernel_batchio3b`): a note making a creator (0807.4720, amsart `\thanks{… \author{Bob}\address{…}\email{…}}`) is a rehearsal fact (`\lx@fm@obs@add` → `EventKind::ApiCall` "ltx:creator") and is digested as Perl's, each contact the newest creator's; `=2` = off on it.
- Left (synthetic, amsart, `scratch_IO3c/u/`): a later note with no creator after it (`\thanks{Again \email{both@x.org}}`) is still answered by the first flush, Ann's at `=2`, Bob's off and Perl (`u8.tex`); `\thanks{X \author{}\address{Univ B}\email{bob@b.org}}` orders Ann's contacts differently at `=2` (`u5.tex`); the content check counts an excess for a note's contact in `\setbox0\hbox{…}` or in a `\title`'s `\thanks` (`u2.tex`, `u10.tex`; W=1 at `=2`, 0 off, the output = off); no shape found where such a note's contact is unplaceable (no guard for the count).
- (26) From the int151 fix rounds' reviews (revI1, revI2; CK2b, FB2b9 SHIP), REDs, synthetic, no arXiv witness, probes in `scratch_revI2/pr/`: a `\vskip` written by a macro between two tabulars (`\newcommand\sk{\vskip 2mm}`, h03) is read as a column though pdflatex stacks them — the row break tests the literal token only (`rehearsal.rs` `prints_a_row_break`); `\\ *[2mm]` (g14): the binding and Perl (`latex_constructs.pool.ltxml:254` `OptionalMatch:*`, `Gullet.pm:604`) skip no space before the star, TeX's `\@ifstar` does, so "Bob Two" becomes a nameless creator's affiliation and "[2mm]" is printed — counted (W=2), not recovered; the take-out matches cut words against all the addresses' words, so `{a,b}@mail.org` holds the domain twice and the spare copy could swallow a label word "mail" (`resolver/mod.rs:1310-1324`; f1 = base, W=0).
- (27) From CK3's reviews (revI3, revI4; CK3, CK3b SHIP; probes `scratch_revI3/p/`, `scratch_revI4/p/`), synthetic: false flags (the fail-toward-flagging side) — `@{\hspace{2mm}}` in a column spec counts "2mm" (t3), `\tabularnewline[2mm]` counts "2mm" (r12); real losses counted but split — `\O rsted`, `\L ukasz` as "rsted", "ukasz" (r6), a glued `>{Prof}c` as "Dept Prof" (t8); not counted by design — "via"/"or" lost (t1b, t15): stop words, which the content words leave out as the scorer does (`insert.rs:148` `net_content_words`); **quiet, pre-existing** — a spec given by a macro (`{\mycols}`) never shows its printed text (t5).
- (28) From CK4/CK5 and their review (revI5, SHIP; probes `scratch_revI5/p/`), synthetic: CK4 reads a raw address's `~` as a space (85b) and CK5 records two rehearsal facts — a nested `\lx@add@email`'s `name=` label (`Event::name`) and llncs's queued `\email` creator call — so the three guards' false counts are gone; **on real papers nothing moved**: the 1,434 stage2 papers at `=2` raise "more often" 54 → 54, "missing" 24 → 24 (`scratch_main/ck5recount/`), so their Warnings come from other mechanisms (a sample to class, real vs false, before the flip).
- Left: a false "missing" for a user `\renewcommand\email[1]{}` under llncs (q1a, CK5's walk keeps the binding's contact, `events/mod.rs:1726`); `\email{ann@x.org;~bob@y.org}` in `\institute` adds a second contact `~bob@y.org` at `=2` (u3, u4; flagged "more often"); `\email{Dept~ann@x.org}` one contact "Dept~ann@x.org" (u1, off too); llncs `\thanks{\email{…}}` a false "missing" (r2); **quiet, pre-existing** — `\phantom{\email{ann@x.org}}` places the invisible address and puts Bob's "University of Y" on Ann (q1b).
- (29) **The content check on real papers at `=2` — flip blockers** (`agent_reports/2026-11-08_cw78_report.md`; 20 of the 78 Warning papers of the 1,434 read against off and pdflatex): "more often" is **11 of 12 real duplicates off does not have** — an e-mail lifted from a `\thanks` into a contact while the note keeps it (0807.0895, 1410.4175, 2009.03378, 2604.08687, 2410.14846; FX3), an `\href{orcid}{\usebox{\orcid}…Name}` whose text, the author's name, becomes the orcid contact (2410.07791, 2507.22485, 2602.05817), a nameless llncs creator holding both e-mails (2009.10916), a revtex `\riera` macro read as a second creator (cond-mat9310042), a `\href{mailto:email@domain}{…}` target never printed made the contact (2607.28418); 1 false (1001.0337, mem's printed "e-mail:" label).
- "missing" is 6 of 8 false (A: a class-printed label left uncounted, `words.rs:98-139`; B: words pdflatex never prints — cleveref internals, `\orcidlink`'s argument; C: glyphs in a title note's math; D: identical text printed twice held once, `placed.rs:218-237`; E: an aastex63 `\email` on the affiliation's line, `lost_inks`), 2 real: 2111.06447 (svjour3) dropped sibo.cheng@imperial.ac.uk at `=2` — fixed by FX5 (an entry not written leaves its inner e-mail its own item; `perfect_kernel_batchfx5`; its content-check Warning left is mechanism D), 2609.22862 a second title page lost off too.
- (30) Outside the frontmatter (math parked; found by FX3b's two runs of one binary): **an ambiguous formula's XMath ids vary from run to run of the same binary** — 2308.08199 `S6.E3.m2.10` vs `.m2.9`, 0909.4443 `S3.Ex21.m3.14` vs `.m3.7`, both raising `ambiguous_math`; a noise source for every A/B (classed as noise when the paper reruns alone identically), cause not traced (the ASF's choice among equal parses, or the id counter's order).
- (31) From FX3/FX3b/FX4 and their reviews (revI7 NO-SHIP → FX3b; revI8 SHIP; probes `scratch_revI7/probe/`, `scratch_revI8/probe/`): **FX3** (`emitter/emit.rs` `in_a_written_note`) writes a note's address once at `=2` — the note keeps it and no contact is made **when the note is on the contact's own owner** (42 papers of the 1,434, off's outcome); a note on another creator leaves the contact, the only tie to its owner (2410.20285 Antonis, main-session ruling, charter §4), and the check counts that copy; **FX4** (`emit.rs:661-674`) writes an ORCID link around the name once — the link in the personname, no orcid contact, Perl's authblk (`authblk.sty.ltxml:51`) and off's shape (2410.07791, 2507.22485, 2602.05817, 2111.12794); "more often" on the 1,434: 54 → about 21 (FX3b's 25 less FX4's 4, measured apart; the bundle's stage2 confirms).
- Left, synthetic: `\href{orcid}{Ann} Able` gives a creator "Able" and a nameless one holding "orcid: Ann" (a5); an icon link before the name a nameless ORCID creator where off gives it to Ann (a7); **one ORCID link around "Ann Able and Bob Baker" a merged creator and an Error at `=2`, off two creators** (a4); FX3b's owner key is the reading's holder, taken before the whole-block remap (`emit.rs:1766` vs `:2388-2398`; no probe triggers it); `=2` writes `\hspace{1mm}` after an icon as U+2005 where off writes U+2003.
- TG1 restored the block-written-whole guard as a unit test (`resolver::placed::whole_tests::a_whole_block_exempts_only_its_own_creator`); **quiet, synthetic**: the exemption is a per-word allowance, not per creator, so a word printed in a block written whole and placed in excess by another creator goes uncounted (no document shape found).
- (32) **Readiness re-measure on int150a** (`53d3d04906`, `scratch_main/readiness/run3.sh`, `diff3/`; per creator / per paper exactly right, `=2` vs off on one binary): dev3 10k **99.07 / 98.70** vs 98.88 / 98.57; devh 10k **98.59 / 98.03** vs 98.31 / 97.80; longtail 3k **94.21 / 93.75** vs 93.05 / 91.81; dev20k **98.67 / 98.36** vs 98.39 / 98.15 — `=2` above off on every set, above 98 % per creator and per paper on the three dev sets.
- Papers worse at `=2` than off: 12 + 14 + 3 + 27 = 56 (int147a: 140).
- **The 152 keep-better papers: 4 lost their `=2` advantage** (now equal to off, worse than int147a's `=2`): 2306.04726, 1307.5879, 1409.0847, hep-ph0011101 — a regression between int147a and int150a, being root-caused (`scratch_kb4/`). int151's fixes (FX3-FX5) are not in this measure.
- (33) **15 of the 56 worse-at-`=2` papers read** (`agent_reports/2026-11-08_w56_report.md`, outputs `scratch_w56/p/`): 11 real `=2` defects, 1 both wrong, 2 ties, 1 the scorer's; only 3 of the 11 raise a frontmatter Warning.
- **Content lost at `=2` (pre-flip):** an accent whose argument is a control word, `Chiriv\`\i` → "Chiriv`" (1709.03335; `emitter/emit.rs:351-387` `past_what_it_set` has no `\i`/`\j` case — confirmed); svjour3 `\institute` nested in `\and{…}` unread, an affiliation lost (cond-mat0605195); the rehearsal budget's fallback writes a block holding a tabular inside a name — 4 new Errors, 8 names lost (1511.05911, `stomach.rs:1285`, `:1718`); an `\orcidB{}` SVG path's text dissolving the line, two authors lost (2409.13798, the `by_text` fallback `emit.rs:752`); **new Errors at `=2`**: 3× `readBalanced ran out` from the emitter re-reading a cut token slice (1111.2718).
- **Wrong owner or kind** (after the flip unless cheap): marks not linked to their legend lines, every line on every author (2305.13191, 2602.07769, 1202.3425 — revtex's grouping overridden); names or a group author read as affiliations (2506.08710 `\footnotemark[1]~~` in `\small{}`, 1111.6740 `\and The HerM33es Team`); a `\texttt` e-mail line glued to an affiliation (math0603155); a shared address kept beside the split ones (astro-ph0310059).
- (34) **The flip's gate (user 2026-11-08):** 98 % per creator and per paper at `=2` on dev3_10k, devh10k and dev20k (met on int150a: 99.07/98.70, 98.59/98.03, 98.67/98.36) and flip200 judged once; longtail3k "good enough for the flip already at 93%" — only `=2` ≥ off, no regression; no harm (the `=2` content losses and new Errors of item (33) fixed); no keep-better paper lost (item (32): KB1 glued mark, KB2 the e-mail cut; 1307.5879's correspondence note kept by ruling); the `=2` guards re-expected per mode or named REDs; then the user's yes.
- Wrong-owner cases, longtail and synthetic REDs continue after the flip.
- Graceful degradation (block-level output) is the accepted failure mode; no detection mechanism without recovery (user 2026-11-08).
- Bundle chain `stage2.sh` now also lists papers newly erroring at `=2` vs off and keep-better papers fallen back to off.
- (35) **int152's pre-flip fixes** (reviews revJ1-revJ3 SHIP; revJ2 blocked KB2 → KB2b): KB1 a glued mark "a1" split into its printed labels at `=2` (hep-ph0011101, `resolver/labels.rs:1567-1587`); KB2/KB2b the e-mail cut gives a phrase-completing entry back whole **only when its holders are exactly the address's owners** (2306.04726, 2110.05460, 2602.09324 whole; 2004.11812 cut as the base; an address printed in a formula written as text, 1409.0847), shared entries cut; CL1 an accent over a control word keeps its letter (1709.03335); CL3 the whole-block fallback writes a tabular juggle (vldb `\alignauthor`) as a line break — 0 Errors, every name kept (1511.05911, 1810.03464).
- Left, synthetic: a juggle macro printing words after its `\end{tabular}` loses them in the fallback (off too; `fallback.rs:1478-1496`, emit the rest of the expansion — revJ3 c1); a juggle written straight into `\author` errs off and at `=2` (c2); a shared entry holding one address per holder goes whole on every holder, markup worse than the base (KB2b, `address_cut.rs:854-874`, k11); a block-held whole entry may keep its e-mail twice (suspected); an unmatched or unmarked address still goes to the last author (KEY1's annotate default, revJ1); one author's e-mail on a nameless creator (revJ1 p5); the KB2 shared entry's co-holders lack the address (1409.0847, off's outcome — after the flip).
- (36) More of int152 (reviews revJ4, revJ5 SHIP): E1 a declaration's span carried to its argument's matching close (`field_notes.rs` `balanced_reach`; 1111.2718, 0907.0548: 9 Errors → 0 at `=2`); CL2b a binding's list setter (a setter whose definition clears its list: the `\institute` of sv_support, inst_support, aa, llncs) called inside an author block declines the paper — block-level, every word kept (cond-mat0605195 = off; 0 of the 1,434 decline); CL4 a TikZ picture gives the frontmatter reader only its text nodes (`util_text.rs` `ink_text_printed`; 2409.13798's icon path text gone).
- Left, synthetic or deferred: a `\gdef` inside a call's own group lost at `=2` — `Error:undefined` (`field_notes.rs` `field_invocation`'s take-back, revJ4 1); CL2b's check also compares the document's `\author` tokens against the setters' names (`ledger.rs` ~796, outside the `batche0` pin's regex — revJ4 2); the declined paper's content check counts a false excess (cond-mat0605195); a TikZ picture outside a link still reads its drawing at `=2` — a name or mark drawn as a node lost (`events/mod.rs:1765`, `:832`; revJ5 `t/tkname.tex`, `tkmark.tex`); 2409.13798's names inside the address line (`scratch_CL4/r/sib_first.tex`) and svjour's faithful `\institute` reading — after the flip.
- (37) **The tests at `=2` before the flip (G1-G6, test-only, user 2026-11-08 "reach the flip sooner"):** every guard failing at `=2` on int151 (615) re-expected per mode where `=2` is surely right (`expect(off, two)`, 14) or skipped at `=2` as a named RED (`red_at_two(why)`, `perfect_kernel_batcht1.rs`; 597 rows in `docs/parity/FRONTMATTER_REDS_AT_EMIT2.tsv`: test, then what `=2` gets wrong — 58 `old engine only` (the shadow comparator, a writer outcome; deleted with the old driver), 65 `unverified:` (not checked against the PDF; triage after the flip)); off expectations unchanged; each group's modules pass off and at `=2`.
- At the flip each `expect` becomes its `two` and each `red_at_two` guard an `#[ignore = "RED at =2: …"]` keeping the right expectation.
- (38) **The flip (FLIP1, 2026-11-08):** `LATEXML_FM_EMIT=2` is the default — `writer::emit::fm_emit_mode()` returns 2 when the variable is unset or unreadable, the one place it is read (every engine site calls it); `LATEXML_FM_EMIT=0` keeps the old driver, byte-identical to the pre-flip default, until Z18-Z19 delete it.
- The tests' switch helpers (`perfect_kernel_batcht1::fm_emit` and its copies in `06_cluster_frontmatter.rs`, `cluster_cli.rs`, `bin/ab_audit/fmshadow/tests.rs`) default to 2 as the engine does; guards pinning a mode set it explicitly (`set_fm_emit_override`, or `LATEXML_FM_EMIT` in a child's env).
- Checked: 20 dev3 papers byte-identical both ways (unset = the base at `=2`, `=0` = the base unset); the frontmatter guard groups pass unset and at `=0`.
- Next, in a bundle of their own: the deletions, the `expect` → `two` collapse and `red_at_two` → `#[ignore]`.
- (39) **The flip's no-harm pass (2026-11-09; OD #981).** Run before the flip on the flip candidate, paper by paper on the four dev sets: (a) **Errors** — 37 papers erred at `=2` only (stray alignment `&` in a re-digested cut, 14; counter out of order; a label's `_`; the IJCAI split; `\,` initials; "representing"; a cut control word; an open `\endgroup`) → 0 of 37 (NE1, NE2/NE2b/NE2c, NE3); (b) **content**, a whole-document word diff (`kit/emit/docwords.py`; the creators' words had missed it): `=2` swallowed body footnotes (1312.3323, pnastwo's `\affil#1#2` misbound → a TeX group left open; CL6), dropped affiliations, pubnotes, a `\contributor` line, printed labels ("Email address for correspondence:") and used marks ("50A,50C" → "A,C"), and gave 2201.02056's authors 48 wrong "Also at" affiliations (CL7, CL8/CL8b, CL9b/CL9c, CL10).
- A general "decline on any lost word" (CL9) closed 13 of 222 and threw away right readings — dropped.
- **The residual, flipped with as named REDs (user 2026-11-09):** `docs/parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`.
- Open beside it (REDs): the author-field Errors silenced since the flip (the PRIORITY RED: 18 A/B papers, revDEL4b) are raised again by ERR2 (below; OD #983); a block whose `\footnote` counter steps inside `\author` (14 A/B papers) goes whole as one creator "Ann Able  and Bob Baker" where Perl's article writes two, so that Warning's "as Perl's whole-block creator" is false there; a `\stepcounter` of another counter inside `\author` is lost (0 vs Perl/pdflatex 1; 0 A/B papers; repro `observer_rehearsal_loses_an_author_stepcounter.tex`); DEL3's list-setter port (OD #982), synthetic only (2 of 73,251 papers have the shape, neither these cases; probes `scratch_revDEL3b/rp/`): R1 a top-level svjour `\institute` *after* the block takes over — the block's Univ X / Lab Y lost with 2 orphan Warnings, where pdflatex prints the block's list (`\maketitle` sets it last; fix: clear the epoch's affiliation calls up to the flush), R2 an entry of the written setter call no mark asks for is dropped with an orphan Warning (`sv_unref_in`: pdflatex prints "2 Lab Y, City"; base and off keep it on a nameless creator; `sv_typed_sup` the same); the ORCID logo of an icon-only `\href` is lost at `=2` (OD #960; int153m `=2` vs off: 2507.12319 4→0 logos, 2406.08258 2→0; pre-existing at the flip, not DEL3); the ejpecp e-mail notes the class queues besides its creators are lost at `=2` ("holds no place … (ltx:note); it is lost", counted; binding net ejpecp 3 of 30 worse: 1809.03082, 1310.4722, 2609.22952 — rows in the residual TSV); NE5's `declaration_reads_past` writes six A/B papers' blocks whole that the candidate before it split (0809.5152, 0906.2789, 1110.6375, 1404.5602, 2404.18592, 2503.21719; 0 distinct words lost, three also drop a queued `ltx:date`, counted) — narrow it to a definition whose replay actually reads past; `resolver/mod.rs` takes every held Error in an IJCAI block as a section marker, not only those at `\affiliations`/`\emails` (revX7, unprobed); NE8's IJCAI cut is picked by an `@` in a held section's printed text, not by the split's own delimiter (`resolver/blocks.rs` `address_section`; revX5, synthetic `scratch_revX5/syn/a1.tex`: e-mails inside `\affiliations` with no `\emails` → at `=2` the held `\affiliations` Error off also raises, and "Univ B \affiliations"; fix: cut at the marker whose held token is `\lx@ijcai@affilsplit`'s `Until:\emails` delimiter, read from its definition; no stage2 paper reaches it); a frontmatter TikZ picture with no text inside `\href` (`\author{Ann\\ \href{https://e.org}{\tikz\fill (0,0) circle (0.1);}}`, or in `\thanks`) is dropped with no Warning, off and `=2` (NE7; needs a kept contact/note or a counted Warning); 1602.03850-shaped names-only macro overlaps written as each name's text; two unrecorded calls in one flush; 1901.02012's sequenced e-mails counted lost, not owned; AIMS's `\email` label printed at the end (1903.05770); an `\emph`-wrapped class label leaves an empty wrapper; a label's font not carried into `name`; `E-mail: \email{…}` with a class `\email` printing "E-mail:" loses one; `content::reads_past`'s end-of-run rule (a run ending at a call none of whose arguments it holds) has no guard — no input reaching it was found (a piece a macro prints spans its arguments, `\newcommand\place[1]{Princeton}` + `$^1$Univ \place{x}, $^2$Univ B`; a mark's span is grown by `arguments_end` first), so it stays a backstop (NE8); since the flip a `\stepcounter` in an author field is lost: `\author{Ann Able\stepcounter{probe}\ifx\foo\undefined\gdef\foo{ONE}\else\gdef\foo{TWO}\fi \and Bob}` with `Value \foo{} and \theprobe.` in the body gives "Value ONE and 0." at `=2` (0 errors, 0 warnings: the `\gdef` kept, the step lost), where Perl, pdflatex and off print "1" (revDEL4 P1; repro sectioning-frontmatter/`observer_rehearsal_loses_an_author_stepcounter`, pinned as today's in `perfect_kernel_batchfms2a1::the_rehearsal_leaves_nothing_behind`); an author field's undefined commands, whose Errors the rehearsal held and discarded, are raised once since ERR2 (OD #983): in place where a piece holds them (after a name, past its marks and notes; an address whose local part an undefined command glues a word's ink onto — ink of the same word on both sides of it before the `@`, a Name the ink table cannot align failing closed on its span — is never read as one, any other read as without the Error, so `\author{Ann Able\undefinedprobecs \and Bob\email{b@x.org}}` gives Perl's 2 Errors and creators Ann Able and `Bob<ERROR>\email</ERROR>b@x.org`), else by the flush's net (a piece written as text); the flip A/B's 19 papers: every lost `Error:undefined` back except the artefacts off raised and pdflatex does not (kr.sty's `\affiliations` in 2405.06973, 2602.18351, 2604.26688; 2405.13308's `\@thefnmark`/`\@undefined` from a digestion that lost both names), creators never fewer than DEL5's, 0 words lost, stage-2 1,434 unchanged; the `\fi` chains of 1603.08231 (18) and 2112.06251 (12) are Perl's split at `\and` (KNOWN_PERL_ERRORS #1218; pdflatex 0), correctly absent.
- ERR2's REDs left by its last review (revERR2c, SHIP): one undefined command after `\footnotemark` writes two `<ERROR>` stubs while logging its one Error (`\author{Ann Able\footnotemark\foo{} \and Bob Baker}`; Perl and pdflatex one; ERR2-introduced, 0 of the 358 sample papers; suspected `emit.rs` `block_names` taking the held command into `apart` while the note piece grows over it by `content.rs` `past_held_undefined`) — the next batch; the span fallback's `@` test is end-exclusive (`b\foo@x.org` keeps the invented `Baker1b@x.org`, `b\foo{}@x.org` declines); pre-existing on the base: a superscript mark glued before an address is read into it (`Bob Baker\textsuperscript{1}b@x.org` → e-mail `Baker1b@x.org`), and `\author{Ann Able, ann@x.org\footnotemark \and Bob Baker}` gives one creator (TeX 2, Perl 3; no word lost).
- **After bundle int154a** (this row's REDs; the bundle's own residual in "Bundle int154a — the residual"):
- the author-field Errors silenced since the flip: CLOSED by ERR2 (OD #983); the 6 rows with 0 Errors where the old engine printed some are correct (CR9: pdflatex 0, the old engine read `\author` outside the class's `\@maketitle`).
- the footnote-counter whole block: CLOSED by CR7 (Perl's `\lx@add@authors` split, OD #988), but where CR7d's fail-safe keeps the base's whole block.
- the ORCID logo of an icon-only `\href`: CLOSED (2507.12319, 2406.08258; Z18g `frontmatter_fields::orcid_icon_link`, CR5).
- the ejpecp e-mail notes: CLOSED (CR5: 0 words lost on all 7).
- NE5's six whole blocks: CR7 split 1404.5602, 2503.21719, 2404.18592; 0809.5152 and 1110.6375 whole by the fail-safe; 0906.2789 one creator.
- the textless TikZ picture in `\author` or `\thanks`: CLOSED by CR8 (`perfect_kernel_batchcr8::{textless_picture_in_author_is_a_note,textless_picture_in_thanks_stays_in_the_note}`); inside `\href` it was already kept (CR5).
- ERR2's two `<ERROR>` stubs after `\footnotemark` and the end-exclusive `@` test: CLOSED by DEL5b.
- NE8's IJCAI `@` cut: synthetic only, RED catalog (CR5).
- Still open: the lost `\stepcounter`; DEL3's R1/R2; revX7's IJCAI held-Error suspect; the glued superscript mark (`Baker1b@x.org`) and `ann@x.org\footnotemark` one creator.
- Reviews: revNE1, revNE2, revNE2r2, revX1, revX2, revCL9b, revCL10 (and the fix rounds' re-reviews).
- Reports: `agent_reports/2026-11-07_{F1,F1r2,revF1,revF1r2,N1,revN1,LO1,revLO1,N2,revN2,N2r2,LO2,revLO2,LO2r2,revLO2r2,tests_sort}_report.md`, `2026-11-09_{NE1,NE2,NE1w,NE2w,NE2b,NE2c,NE3,CL5,CL6,CL7,CL8,CL8b,CL9,CL9b,CL9c,CL10,revNE1,revNE2,revNE2r2,revX1,revX2,revCL9b,revCL10}_report.md`, `2026-11-08_{T1,revT1,N2r3,revN2r3,LO3,revLO3,LO4,revLO4,RD1,P2,revP2,P2r2,revP2r2,RC1,revRC1,revG1,revG2,revG3,revBC1,revG5,revH1,revH2,revH3,revI1,revI2,revI3,revI4,revI5,revI6,revI7,revI8,cw78,ts9}_report.md`

### An image before a URL, mark or address on a frontmatter line is dropped silently (CR8c; OD #989)

- **Known silent loss, open:** 2503.08161's `hf-logo.png`, an image before a URL on its line, is dropped with 0 frontmatter Warnings (revCR8c finding 2, confirmed on both binaries); its logo on the first marked line after bold names is kept since CR8c (`polyu-logo.png` on its 4 authors; guard `perfect_kernel_batchcr8::image_on_the_first_marked_line_is_kept`).
- RED repro `sectioning-frontmatter/fm_image_before_url_is_a_note.tex`, no guard.
- Drop site: `events::place_pictures` (`observer/events/mod.rs` ~2645-2656) places a picture only on a names line, at a line end or inside a later line's text; one that leads a URL, mark or address piece has no place and is not written.
- Why the content check cannot see it: an image has no words, so the word count (`resolver::placed`) has nothing to miss.
- A naive count of Layout images false-flags the icons a URL contact writes (guard 77y, 2505.21327); a sound Warning needs span coverage — every picture's span inside some written item's span.
- Reverted attempts: CR8c's note in place and at the line's end kept the image but merged 2505.21327's two `{\github\ \url{…}} \\ {\huggingface\ \url{…}}` URL lines into one affiliation (markup lost, +1 W); the Warning attempt's patch is `/home/deyan/data/pk_wt/scratch_CR8c/image_warning_attempt.patch`.
- Fix: place the picture as a note at its line (the picture ruling, user 2026-11-10 "A note, kept whole") or count it by span coverage.
- State: open (main session at the CR8b/CR8c merge `08460a0ea6`).

### Bundle int155a — the residual (OD #983, #987, #988, #980, #985, #989, #991)

- Landed: PB/PBb (the push's B1/B2), Z18k/Z18k2/Z18k3 (eptcs + OmniBus rerouted), CR10b (CR9's glue, `\fcolorbox` line, nameless `\author`'s setters), Z18l (aas rerouted), PC3 (pazh `\address`, `\@nil`), Z18m/Z18m2 (`author_block/` deleted), Z18o (the bundle's own REDs F1, F2, R1 and the runaway guard), revCR10c's REDs recorded. Open facts, one per line; repros under `tools/perfect_kernel/repros/`.
- pazh `\address` false Errors (10 papers) and 1802.01191 `\@nil`: FIXED in PC3 (OD #983 — a title-code definee reading arguments is no held undefined command a piece takes alone, `Field::held_undefined_at`; a class's own `\email` reads its argument unexpanded, `UnexpandedSemiverbatim`; RCcut: neither was a cut); neurips_2024.sty unread (2401.16452, HARD).
- Z18o's Known RED R-A (introduced, synthetic, 0 of the 1,166 aas census papers): an `annotate=new` entry continuing its own `\author` block loses its owner, silently (`aastex_affil_email_line_continuing_own_author_block_red`); R-B, N1, N2 and the flush-path `\affil{Univ C; zz@x.org}` (`aastex_affil_unmarked_line_address_under_marked_names_red`) → Z18o2.
- 6 aaai2026 papers W +1 (the content check's "more often"; the block's rehearsal lacks the title code's definitions) with E 1 → 0 — accepted by the main session (OD #983).
- OD #983 REDs (revPB/revPBb, synthetic): a reader macro's first use as the reading point (`pbb_reader_first_use_in_a_measuring_group`); the inline juggle path's `\hfil\linebreak[0]\hfil`; the stale-copy side of `copy_kept_meaning` unguarded; a `\linebreak` inside a title-tabular name a row break (`title_tabular_linebreak_inside_a_name`); a counter-stepping title-code mark lost after the first author (`title_code_counter_mark_first_author_only`).
- OD #988: the wordless-piece fallback never uses a piece up (revPBb S1, no failing probe).
- OD #987 REDs (revCR10c): a `\def`'d old block name loses its grades (`ieeetran_def_old_block_name_loses_grades`, 1 W); a wrapped unclosed names block spliced (`ieeetran_wrapped_unclosed_names_block`).
- OD #991: 2304.02524's e-mail marks `$^{a,b}$` a counted block note (source plus placed addresses); a marked bracket-only piece a creator (`llncs_author_marked_bracket_remark_is_no_name`, revZ18k3 R1).
- OD #980 (revCR10b): 1007.2355's `\speaker` control-space glue (counted); the `{a, b}@x` content-check false Warning; `\fcolorbox` around a name loses its frame. Pre-existing, Perl the same: a setter inside `\author` adds rather than replaces (two `<abstract>` with `\begin{abstract}` too); amsart `\date` inside `\author` written though unprinted; authblk `\affil`'s `\\` a `<break/>` where pdflatex prints ", ".

### Bundle int154a — the residual (OD #983-#990)

- **Pushed with these REDs (user 2026-11-10, "Push now, fix next"; push checks `agent_reports/2026-11-10_PC1_report.md`, `PC2_report.md`):**
- B1, a content loss on a real paper: 1511.09009 loses "Seung-won Hwang, Wei Wang" (the flip kept all four names in one merged creator; a Warning counts the 5 words; ‡ and § become †); CR7's Perl split (OD #988) — its fail-safe checks false owners, not a name the reading found and the split loses: fall back whole there too. FIXED in bundle int155a (PB/PBb, OD #988): written whole, all four names with ‡ and §.
- B2, false `Error:undefined` lines pdflatex does not print (Perl-origin, the KNOWN_PERL_ERRORS #1219 family; ERR2 raises them, the flip had silenced them): the pazh/Astronomy Letters class (`\address` defined only inside its `\@maketitle`, pazhastl.cls:112; the e-mail cut re-digests a piece outside it) on 1108.4222, 1202.2889, 1206.0451, 1207.2226, 1301.4791, 1307.2761, 2103.05801, 2408.01858, 2412.06362, 2607.07582; aaai2026 copies whose `\@maketitle` reads `\@author` only through `\def\theauthors{…}` (`definitions_at_author` names.rs:485 skips `\def` bodies; 2508.00545, 2508.04047, 2508.04749, 2508.05996, 2508.08170); neurips_2022 copies whose `\@maketitle` the binding keeps its own of (`title_code_author_definitions` names.rs:464 reads the live one only; 2205.14141, 2205.14539); neurips_2024.sty never read (2401.16452 1 → 4, HARD, deferred). Repros `scratch_PC2/rep/c1.tex`, `c2.tex`, `n22/n1.tex`. OD #983's exclusion to cover all four.
- B2 in bundle int155a (PB/PBb, OD #983): FIXED for aaai2026's `\theauthors` (2508.00545, 2508.04047, 2508.04749, 2508.05996, 2508.08170, 2508.21091, 2607.29185: E 1 → 0, pdflatex 0) and neurips_2022's kept `\@maketitle` and juggles (2205.14141, 2205.14539, 2205.11640); pazh `\address` FIXED in PC3 (the e-mail piece no longer takes it from its `{1}`; 1108.4222 … 2607.07582 E 1 → 0); OPEN for neurips_2024.sty (never read, 2401.16452).
- 1802.01191 +1 `Error:undefined:\@nil` (Rust only, Perl none, pdflatex none): FIXED in PC3 — the rehearsal's Semiverbatim pre-expansion of the own `\email`'s argument ran `\fontsize`'s `\@defaultunits` ahead of its assignment (RCcut; OD #983 (6)). 2206.01959 "Script _" (ejpecp: the e-mail text re-digested with normal catcodes; Perl raises it too, pdflatex not).
- 1003.2281: the corresponding-author note a nameless creator (after CR3/CR7 gained it 1 → 6 creators).
- Cost: 1507.02068 at the +0.5 % bar (+0.496 % over 3 reps), the growth in the frontmatter from the DEL1-5/Z18a/ERR2/DEL5b stretch (ERR2's flush-wide meaning recorder × 418 creators the likely lever).

The bundle's history: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#bundle-int154a--the-deletion-pass-and-the-real-paper-reds). Open facts, one per line; repros under `tools/perfect_kernel/repros/`.

- **Closed by this bundle:** CR1 class A (frozen setters in an author block or call: CR2/CR2b, cond-mat9801109, 1411.6693 by CR3b), class B (svjour3 `(\email{…}) \at`: CR2, 2206.00320, 1307.4424), class C (shipped imsart copies: CR3, 1905.11425, 2101.10506); ejpecp's e-mail notes (CR5: 0 words lost on all 7); the ORCID icon-only logo (2507.12319, 2406.08258; Z18g `orcid_icon_link`); the textless TikZ picture in `\author`/`\thanks` (CR8, `perfect_kernel_batchcr8`); `\includegraphics` beside a name (CR8b); the silenced author-field Errors (ERR2, OD #983); the footnote-counter whole block (CR7, OD #988); 1411.6693's nested date (CR3b); imsart `\ead`'s `$\sim$` (CR4; repro `imsart_ead_url_reads_a_tex_argument.tex`); IEEEtran + authblk names (CR6), nested names blocks (CR10a, 1511.03982).
- NE5's six whole blocks: CR7's split un-merged 1404.5602 (1→4, right), 2503.21719 (1→3); 2404.18592 1→2; 0809.5152 and 1110.6375 stay whole by CR7d's fail-safe (revCR7c B1, 1110.6375: `$^\ast$,\thanks{$^\ast$Permanent address…}` → the next author; B2, 0809.5152: two `$^{a}${\em …}` address lines become authors); 0906.2789 still one creator (1→1 under CR7).
- CR7c S2: 2503.21719's `\\` line of names read as an affiliation (Perl does the same); since PB (int155a) the block is written whole — all 5 PDF names in one creator, 0 words lost.
- CR7: should the other declines (names_fail, cells, mark_dropped, names_overlap, the unread block) take Perl's split too — measure first.
- CR7: 1302.3028 writes an empty "affiliation: E-mail:".
- CR7c: a mark digit glued in cicp names ("Lixiu Dong1"; pre-existing); no B1 guard (no pdflatex-clean synthetic).
- CR7 S2 recorded: 1003.2281, cs0404029 hand their notes on the block creator (charter §4's #159 amendment allows).
- revCR7d synthetic REDs (OD #988): S1 a note-led later piece; S2 an address line whose mark follows its text; S3 the raw-split fallbacks.
- CR9's glued-break class: FIXED by CR10b (int155a, OD #980 `kept_breaks`) on 2212.03638, 2211.03175, 0904.3899, cond-mat0204589, 1311.0012; 1007.2355 is another path (PoS `\speaker`'s control space in the reading's text, counted; OD #980 Known RED).
- CR9: `\fcolorbox` e-mail line in `\author`: FIXED by CR10b (2211.07344, 2305.15057; OD #989); left the content check's false "missing" Warning on a `{a, b}@x` line, boxed or not (OD #980 Known RED).
- CR9: jheppub's abstract inside `\author`: FIXED by CR10b (1211.2861, OD #985: a nameless `\author`'s setters recovered).
- CR5: H's rebuilt 134 rows / 436 words are mostly artefacts or `=2` right; the IJCAI `@` cut is synthetic only (RED catalog).
- CR10a: 2605.20445's Umer Asgher kept, but as an affiliation (an unclosed outer block) — CR10c.
- CR10a: `grade_text` is a local copy of the engine's `grade_words`; `\authorblockN`/`\authorblockA` to be normalised once at `author_block_body` — CR10c, which also fixes OD #987's labelled-path RED (`\authorblockN` alias, synthetic).
- CR4: `$\dag$` in an e-mail (2604.07356).
- CR4: `\date` directly inside `\title` (synthetic).
- CR4b: amsart's brace-list split warns (pre-existing).
- CR4: REDs `date_nested_twice_never_runs_red.tex`, `imsart_address_block_fallback_run_{q1,q2}_red.tex` (OD #985/#986).
- CR2b: cond-mat9809277's empty minipage contact (Perl parity; trigger `\vskip -0.5in $ $` after `\pacs`).
- Z18d: S6 separators after an entry's last call stay, as in the old split (2605.21246, 2210.09234, 1408.0510; OD #984).
- Z18d: q1 a control-symbol macro in a list counted missing; q6 a later template call on the last author, no Warning.
- Z18d: P2 a `\footnote` in a list `\email` leaks `\lx@fm@obs@fnnote` into the address (RED `class-bindings/llncs_list_email_note_name_leaks_into_address.tex`, pre-existing).
- Z18i: S-b a joined run under one `\and` column dealt in print order (RED `class-bindings/article_author_url_pieces_owner_spelling.tex`, no guard); revZ18i's other REDs in OD #984.
- Z18g: `\path` delimiter ink printed as em dashes (pre-existing; W=2 on 1504.01099).
- DEL5b: P1 a superscript mark glued before an address read into its local part (`Bob Baker\textsuperscript{1}b@x.org` → `Baker1b@x.org`; pre-existing).
- DEL5b: P2 the `\and` boundary lost after `ann@x.org\footnotemark` (one creator; TeX 2, Perl 3; pre-existing).
- revDEL5b finding 3: a second use of an undefined command keeps no stub (RED `sectioning-frontmatter/emitter_undefined_command_second_use_keeps_no_stub.tex`, OD #983; `f776ddb597`).
- DEL3: R1 a top-level svjour `\institute` after a block with a list setter loses the block's "Lab Y"; R2 an unrequested entry of the written setter call dropped with an orphan Warning (synthetic; OD #982).
- revX7: `resolver/mod.rs:1396` takes every held Error in an IJCAI block as a section marker (suspect, unprobed).
- 2606.14105 (BESIII, 770 creators off): one creator, "unstamped: out of positions", counted (pre-existing since int153d).
- The e0 fence's `ONE_ENGINE_REACH` counted only `author_block` exports; three modules were named `email_lines` (classify/, author_block/, frontmatter_api/) — DONE at Z18m/Z18m2: `author_block/` deleted, its last items in the old reader's files of their own (`classify/old_reader/`, `defs/author_macros/ijcai.rs`), which `MOVED_OLD_READER_FILES` lists; the reverse fence counts every name they export but `text_tokens` (exempt in `OLD_READER_NAMESAKES`: three items share the name) and reads only the one engine's files (`perfect_kernel_batche0::the_old_readers_names_never_count_as_the_one_engines_own`); the two `email_lines` left named by their parents (`classify::email_lines`, `frontmatter_api::email_lines`).
- e0 fence hardening (review revZ18m2 S1, pre-existing, partly closed by Z18m2): an old-reader export moved out of a fenced file into a library file stops counting and nothing fires (the old-reader decision is by file; the ceiling is `<=`, so it only falls); fix in its own batch — pin the full list of the old reader's 81 exports, make `OLD_READER_CODE_LINES` exact, assert `OLD_READER_NAMESAKES` equals the shared names (N1), rename the two one-engine `text_tokens` helpers and drop the exemption, and split an `old_reader` row from `TOKEN_READS`' `classify` (515).
- Private-item rustdoc has pre-existing unresolved links and unclosed tags (not gated) — the cleanup phase.
- Z18k (no commit; WIP `agent_reports/2026-11-10_Z18k_pending/Z18k_wip.patch`): eptcs worse 1101.4424 (`\email{a \qquad b}` glued `\qquadtassiana98@`, font lost — fix on the tokens in the reading's address reader); 1709.04714, 1904.04371, 1908.10743, 2108.10490 +1 creator; 1210.2456 2 → 4; 1504.02441, 2510.13425 unread — Z18k2, after the push.
- Z18k's gains to keep: omnibus `\authors` whole to `\lx@add@author@split` with `\affil{n}` as the `\inst` mark (17 of 154 changed: 1810.04324, 2512.09915, 1304.1392, 1304.1412); eptcs as printed columns (26 of 73 changed: 2304.02524, 1610.08171).
- Z18k decided (charter §4 "owner uncertain stays at the block", KPE #1195): 2402.13328's unmarked list → numbered affiliations on the block.
- State: open

### Open residuals lifted from landed rows (2026-11-09)

Each landed row is verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1); these are its open facts.

- **Bundle int147b (LO2 + LO2r2, the second switch-on leftovers; T1, the tests at `=2` part 1; N2r2 + N2r3, the content check's fix rounds; each reviewed SHIP (revT1 NO-SHIP …**:
  - Readiness (`=2` vs off on LO2r2-rel, scored; dev20k pending): dev3 per creator 98.89 / 98.88 %, per paper 98.51 / 98.57 %; devh10k 98.42 / 98.31 %, 97.75 / 97.80 %; longtail3k 93.86 / 93.22 %, 93.12 / 92.15 %.
- **Bundle int133a (98d + 105a; code `9a716bc81b`)**:
  - A/B over `popint125a` (28,354 papers) vs int132a: 22 changed (9 frontmatter), E 46,393 → 46,393, F 173 → 173; the one word-loss row 1710.01722 the known B-side load timeout; the 9 frontmatter changes all 105a gains (e-mail contacts 1305.0854, 2001.07747; PACS codes 1311.3004; Chinese address lines 2206.01007, 2601.14329) but 2108.12618, whose kept template note ("Instructions for Creating an FnT Article") gains its "2018" (placeholder ruling pending).
- **98d: `\end{X}` over a `\begingroup` opened inside X (kernel; SYNC "The unclaimed-text net" R0; OD #965, KNOWN_PERL_ERRORS #1207)**:
  - Round 2 (review rev98d S1): the branch also refuses when the open environment is not X or a group above X records another (LaTeX's `\@badend`); re-A/B, both sides on `vendor_dumpsint132a`: 40 changed vs int132a1 (E 4,215 → 4,214), vs round 1 only 2101.04970, now byte-identical to int132a1 (ntheorem's `\theoremprework{\begin{changemargin}…}` raw group above `\end{fabsmfabstract}`).
  - Open: the bare `\X … \endX` form over a `\begingroup` (no witness; `\endX` keeps batch 66t's Warning); X's layout state (`\leftskip`, `\centering`, `\raggedright`) does not reach a paragraph ended between `\end{X}` and the class's `\endgroup` (TeX keeps it; no witness has content there; OD #965); raa's `\abstract{}` is still the class's body paragraphs, not `<ltx:abstract>` (R routing); 2101.04970's `Can't close environment fabsmfabstract` + `Attempt to end mode` (pdflatex 0: ntheorem's `\theorempostwork{\end{changemargin}}` runs inside `\endX` in TeX, after our `\end{X}`'s check; pre-existing).
  - Review rev98d2 (round 2 SHIP): an inner X the branch already ended, then a raw environment's frame or a second `\end{X}` against the outer X, is refused where TeX closes the still-open inner group (synthetic, better than base: pdflatex 0, base 5 / 3 E, round 2 3 / 1 E; RED boxes-groups/`end_inner_ended_x_over_a_raw_environment_red`, `end_nested_x_over_begingroups_closed_twice_red`).
  - State: landed in bundle int133a; the open facts and REDs above remain
