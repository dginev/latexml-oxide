# Silent-loss inventory — blocks the 98 % claim (user 2026-10-18)

The living worklist of conversions that lose authored content or its markup with 0 errors: a false success. Moved
whole from [`SYNC_STATUS.md`](../SYNC_STATUS.md) on 2026-11-09 (SYNC1b; the section had outgrown that file's ~100-line
rule), in its one-fact-per-line form: one row is one `###` section, one fact per bullet. The user's framing
(2026-10-18): "We should address all of [the High Priority items that were not Error status] before claiming 98 % is
reached".

**Every row is open unless it says otherwise** (a `State:` of closed, accepted or no action). Landed rows and the
landed history of open ones are in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md). The same
rules as SYNC_STATUS: verify a status against its named guard before acting on it; writers hand the main session a
SYNC note rather than editing this file.

"We should address all of [the High Priority items that were not Error status] before claiming 98 % is reached": a
conversion that loses authored content or its markup with 0 errors is a false success. Source:
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

- ptptex `[preprint]` double abstract: the class's own `[preprint]` `\maketitle` runs (ptp-prep.clo:111-119), prints `\@abst` in its block and clears it (:27-34);
- State: open residual (not new in 77i; pinned by count in `perfect_kernel_batch77i::abstract_store_cleared_by_the_class_title_page_is_kept`, repro `sectioning-frontmatter/abstract_store_cleared_by_the_class_title_page_is_kept.tex`): the store is handed on, but the block the class's running `\maketitle` typesets is not the deposit's, so its label and store are not taken out with it
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "ptptex `[preprint]` double abstract".

### #6a labels

- #6a labels: a label read off the class's code only when it is plain text — one with an accent (`R\'esum\'e : \@resumen`, `\textbf{Mots-cl\'es :} \@keywords`: `label_item` refuses the accent macro) stays an orphan paragraph and the entry takes the default name ("Abstract", "Keywords: ");
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "#6a labels".

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
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "biblatex `\printbibliography` filters".

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
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "`\@@underline` is defined under the dump (record `M \@@under".

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
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a split name's mark that a line or other text follows".

### a combined legend mark is labelled by its first label only

- a combined legend mark is labelled by its first label only: `\author{Ann Able$^{1}$and Bob Baker$^{1,2}$\thanks{$^{1}$…}\thanks{$^{1,2}$Bob Baker is with Univ Y.}}` (2604.27101, ieeeconf) gives Ann Able Bob Baker's legend too (`\lx@sup@setlabel@affiliation` keeps the first of `clean_frontmatter_labels`; int32a the same)
- State: open (77n r4; authblk's `\affil{$^{1,2}$…}` leaves the marked path for authblk's run instead, `queued_authors_carry_entry_mark`)

### names parted by a separator inside a layout or declaring group are one creator

- names parted by a separator inside a layout or declaring group are one creator: `{\large \bf Yuxuan Yang$^{1,2}$ \quad Dugang Liu$^{1*}$ \quad …}` (2603.20775, flagged `merged_creators`), `{\centering\bfseries A\textsuperscript{…}\quad B…\\ C…\par}{\centering\small $^1$…\par}` (2610.21682: one creator holding every name and affiliation, 0 Errors — silent), `\textbf{Alain Rakotomamonjy\up{2}\hfill Patrick Gallinari\up{1,2}}` (2209.14855, silent), `{David McAllister$^{1}$… ~ Songwei Ge$^{2*}$ ~ …}` (2406.09417: one tie between spaces, no run, flagged)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "names parted by a separator inside a layout or declaring gro".

### a row of addresses parted by space under a row of names parted by space (`\author{Cat Cole \quad Dan Dorn \\ Univ C \quad Univ D}`, each address printed under the name …

- a row of addresses parted by space under a row of names parted by space (`\author{Cat Cole \quad Dan Dorn \\ Univ C \quad Univ D}`, each address printed under the name above it) gives Univ C to the row's last name and makes "Univ D" a creator, 0 Errors (Perl the same)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a row of addresses parted by space under a row of names part".

### names parted by horizontal space under symbol-labelled address lines (77p-77y; OD #52(o)) — the residual

- **names parted by horizontal space under symbol-labelled address lines (77p-77y; OD #52(o)) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a legend's "Corresponding author:" before a bare address is no e-mail label (2609.24299);
- Also open (recorded 77p r4-77y, not fixed since): a starred legend in a numbered block goes to the last author (2608.22149's `{\small $^{*}$Equal contribution.}` on Yuchen Cui, 2605.03042's `$^\dagger$Project Leader \quad $^*$Corresponding Author` on Shuai Li);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "names parted by horizontal space under symbol-labelled addre".

### a linked two-word text whose last word is a link word, alone in its `\and` column, reads as a name

- a linked two-word text whose last word is a link word, alone in its `\and` column, reads as a name: `\author{Ann Able \\ Univ A \and Bob Baker \\ Univ B \and \href{…}{Source Code}}` makes a creator "Source Code" (no corpus witness, probes a5, a6 — "Online Demo" too; and a three-word one whose last word is a link word that is also a surname, "Full Source Code", "Google Scholar Page", "Hugging Face Link", the link column first or last: probes s1, s3, s4, s7 of the rev77y3 re-review's finding 5 — or its plural, "Other Useful Links", the rev77y4 re-review's probe s20 — no author block linking such a text among 519,672 papers of 2025-2026, int38b the same, 77y r2 URL contacts; 2606.06924 prints "Datasets  Source Code" on one line under its e-mail line, both URL contacts of the shared creator as its PDF shows, the rev77y2 re-review's finding 6; int38b the same), 0 Errors
- State: open (77y r2, the 77y review's finding 1): `link_words_beyond_a_surname` keeps a two-word text a name for "Lucy Page" alone in its column (`perfect_kernel_batch77p`'s linked-names guard) and cannot tell it from "Source Code" without a word list; options: a column holding a linked text alone is the block's link unless the text spells the link's target (`x.org/lp` for Lucy Page), or accept.
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a linked two-word text whose last word is a link word, alone".

### a name alone under a row's merged symbol lines over a one-word generic institution noun or a connective-led institution of its own is lost

- a name alone under a row's merged symbol lines over a one-word generic institution noun or a connective-led institution of its own is lost: `Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics` gives "Cat Cole" to the shared creator as an affiliation and "Physics" to Ann Able and Bob Baker (pdflatex prints it under Cat Cole only);
- State: open (77y, accepted cost of `continues_the_line`, which reads `Shanghai Jiao Tong \\ University` as one institution, probe k12; the hyphenated `Max-Planck-Institut` FIXED in int41, guard `perfect_kernel_batch77y::names_split_under_symbol_lines_reads_the_line_under`).
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a name alone under a row's merged symbol lines over a one-wo".

### in a per-author `\author` that adds to its block, a link column (`\href{…}{Project Page} \\ Univ B \\ contact@x.org`) goes to the current author

- in a per-author `\author` that adds to its block, a link column (`\href{…}{Project Page} \\ Univ B \\ contact@x.org`) goes to the current author: a raw class appending to `\author` puts the URL, Univ B and the e-mail on Ann Able, where pdflatex prints the column beside her (the rev77y4 re-review's finding 3, probes x7, x8; int38b and 77y r2 a creator "Univ B"), no corpus witness, 0 Errors
- State: open (77y r4): `LinkOwner::Current`/`\lx@add@email` for `block_links`/`block_lines` when `replace` is false; option: the block's (`LinkOwner::Block`, `AddressPlacement::Shared`) once a per-author block forms a shared creator.

### an organization link whose name is exactly an author's spelling reads as that author's handle

- an organization link whose name is exactly an author's spelling reads as that author's handle: `Hua Wei \\ \href{https://www.huawei.com}{Huawei}` and `Hua Wu \\ {HW}` give a URL contact on the name, not an affiliation (the rev77y4 re-review's finding 4, probes h20, h21; int38b and 77y r2 affiliations), the link still on the name it prints under, 0 Errors
- State: open (77y r4, `link_prints_a_handle`'s exact-spelling test; no corpus witness).

### a name alone under a row's merged symbol lines with a place under it is lost

- a name alone under a row's merged symbol lines with a place under it is lost: `Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Palo Alto, CA` gives "Cat Cole" and "Palo Alto, CA" to the shared creator (the rev77y2 re-review's probe k10; int38b and 77y r2 the same; 77y r1 an author), 0 Errors
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a name alone under a row's merged symbol lines with a place ".

### a place line with no comma under a welded symbol row's labelled institution goes to the shared creator

- a place line with no comma under a welded symbol row's labelled institution goes to the shared creator: `$^\dagger$Department of Physics \\ University A \\ Palo Alto \\ $^\ddagger$School of Law …` gives "Palo Alto" to the block, not to the † name (the rev77y2 re-review's probe w9; 77y r1 Ann Able's, int38b the block's), 0 Errors
- State: accepted (77y r3, the rev77y2 re-review's finding 7): `reads_as_place` needs a comma (`Muncie, IN 47306`), so a bare place is no `weldable_line`, and an unreadable owner goes to the block (the accepted fallback).

### a brace-list address group and a later address under three names swap two owners

- a brace-list address group and a later address under three names swap two owners: 2606.06924's `\{laign,yehj\}@…, huhr@…` gives `yehj@` (Han-Jia Ye's) to Haoran Hu and `huhr@` (Haoran Hu's) to Han-Jia Ye (int38b and 77y r1-r3 the same), 0 Errors
- State: open (pre-existing; the rev77y2 re-review's finding 8): `spelled_owners` scores "yehj" and "huhr" against the three names' keys — not yet root-caused; the swap puts each address on a name the PDF does not print it under.

### in an unmarked block a row of names after an affiliation line stays an affiliation line of the names above

- in an unmarked block a row of names after an affiliation line stays an affiliation line of the names above: `Ann Able \\ Univ A \\ Bob Baker~~~~Cat Cole \\ Univ B` gives Ann Able the affiliations "Univ A", "Bob Baker Cat Cole", "Univ B" (two authors lost; 0 Errors; int34a and Perl the same, the r3 re-review's probes p19, p27, where r3 had made Cat Cole alone a creator)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "in an unmarked block a row of names after an affiliation lin".

### an author mark with a footnote symbol before the number, `$^{*1}$`, requests no affiliation

- an author mark with a footnote symbol before the number, `$^{*1}$`, requests no affiliation: `Ann Able$^{*1}$, Bob Baker$^{1}$ \\ $^1$Univ A` gives Ann Able none (the 77p r4 re-review's probe k1);
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "an author mark with a footnote symbol before the number, `$^".

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

- **an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o), user ruling R10) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): an icon-only link (2604.10930);
- an address printed twice is two contacts (2604.10667, a ruling if one should go).
- Also: the trade's second witness 2609.02349's `xiahou@` (on Junping Zhang; accepted residuals, user ruling 2026-10-20);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o)".

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
- State: open (77n r5; guard `perfect_kernel_batch77n::authblk_superscript_inside_a_name_stays` pins it). The first half of this row closed in 77n r6: an authblk name kept on `\lx@add@creator` (`Ann Able$^{1}$\textsuperscript{equal contrib.}`) requests the affiliations its marks pick (`author_label_marks`), so the run passes it over (guard `authblk_textual_superscript_after_mark_is_shown`)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a run always reaches its last author (Perl's `new`, Base_Uti".

### authblk's note numbering (`\AB@thenote`, authblk.sty:114-167) is not modelled

- authblk's note numbering (`\AB@thenote`, authblk.sty:114-167) is not modelled: an unlabelled `\author` after a labelled one shares its note (`\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}`: pdflatex "Ann1 and Bob1 / 1Univ X / 1Univ Y", Univ X under Bob too; we give Bob Univ Y only), and an `\affil` inside a later `\author` steps it (`\author[1]{Ann}\affil[1]{Univ X}\author{Bob \affil{Univ Y}}`: pdflatex "Ann1 Bob2 / 1Univ X / 2Univ Y"; we give Ann Univ Y as well, 77n r4 review probe e3);
- State: open (77n r6; int32a and Perl's `annotate=new`, authblk.sty.ltxml:57, the same; guard `perfect_kernel_batch77n::authblk_labelled_author_shares_the_run` pins it): number the notes as authblk does — an `\author` steps `\AB@thenote` after an `\affil`, an unlabelled one takes it — and label each unlabelled `\author`/`\affil` by it
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "authblk's note numbering (`\AB@thenote`, authblk.sty:114-167".

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

- aistats paired column: an address line starting "and" is read as an author separator by the kernel `\author` line reading, so 2402.12508's `Department of Mathematics \\ and Computer Science \\ University of Basel` makes a creator "Computer Science" (both rows since 77o r3; up to 32 paired papers carry such a line; 2310.02702 reads right);
- State: open (77o r2 review finding 6, r3 finding 1). Fix when taken: the binding knows line 1 of each column is the name — add the name, then each address line as an affiliation or e-mail explicitly, instead of the kernel's guess
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "aistats paired column".

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
- State: RULED 2026-10-21: the separator is the binding's (default ": " as Perl), and a copy's caption format (`\fnum@<type>`) is the binding's too — only its declared text names (`\figurename`) reach the label (OD #731 rule 14); no raw `\@makecaption` is read. The 7 doubled separators of 78h r1 are gone (78h r2) — closed
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "caption separator vs a copy's own label".

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

- K11's line rule (OD #662) after 79a r2 — a store printed in one line with another value of the class's, or glued to its text, stays the class's: (a) a store kept the class's when the class is loaded (another value in its line printing text then) is not judged again at `\maketitle`, so a document that empties that value still leaves the store in the class's line;
- State: open (accepted unless a paper shows (a) or (d))
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K11's line rule (OD #662) after 79a r2 — a store printed in ".

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
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A raster graphic's explicit size comes out at 72/72.27 of it".

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

- Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` with no breakpoint, tight TikZ, two 4.5cm sub-floats in a 252pt column, a nested `\subfloat` beside a 4.5cm one with ~1pt to spare in a 246pt column — base the same, review rev78x5 finding 2, `rev78x5/p/m3.tex` fourth figure): 1108.5543 Fig 9, 2009.04593 Fig 5, 1712.01536 Fig 2, 1703.09165 (2 floats), 2406.05437 Fig 12;
- State: accepted residual (user 2026-10-22: overflow)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` wi".

### Panel grid markup

- **Panel grid markup** (grid design payload 3): the XML says a float's rows only implicitly (the runs between `ltx:break`s);
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Panel grid markup".

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
- No error is reported.
- int62a, int63a and 78v alike (older than 78v);
- synthetic, no census paper.
- Fix: read the label from what it typesets (expand macros, drop an unexpandable `\relax`/`\protect`) before matching it.
- Review rev78v2 finding 6a, widened by rev78v3 finding 3 (probes `rev78v3/q/pl.tex`, `pra.tex`, `prc.tex`, `rev78v2/p/sh3.tex`);
- RED repro `sectioning-frontmatter/author_mark_relax_before_its_label`
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "An author's mark whose label holds any control sequence prin".

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
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Author marks".

### A formula silently degrades to `ltx_math_unparsed` under load

- A formula silently degrades to `ltx_math_unparsed` under load: 1001.0023 `S9.E2.m1.pic1.m2` once in a census run (FM r9 writer's set-mode census), never in 3+1+2+2 reruns;
- the log identical, so no warning — the math parser's 2 s pruned-only budget (`parser.rs:235`) is the only time-based path found (revfms2a2_9 finding 7).
- A time budget that changes output without a diagnostic hides regressions in A/Bs: emit a Warning when the budget cuts a parse, or make the cut deterministic
- State: open

### Complete breakage at 100 % (user 2026-10-24)

- **Complete breakage at 100 % (user 2026-10-24)**: a missing image, a `\ref` or `\cite` with no target, where pdflatex resolves it.
- Not ours (pdflatex or the arXiv PDF breaks too): a file not shipped, xr without the other `.aux`, a stale shipped `.bbl` (ruling 2026-10-24: 2509.06285, 2205.10368, 2402.06576, 2311.01729, 2205.00263, 2004.00833, 2507.01293, 1806.02261), the harness\'s main-file guess, a document\'s own `\def\label`, a shipped namesake of a TL package (ruling 2026-10-17 H1: 0802.3014\'s and 1210.0342\'s pretex `units.sty`).
- Method: a fresh sample per round (`one_side.sh`, `scratch_356a0c47/brk<NN>/`), `scratch_356a0c47/fms2b_census/scan.py` over the core XML (a `graphics` with no `candidates`, a `labelref` with no label, a `bibref` with no `bibitem` — skip `show="nothing"`), then `classify.py` against the source (a file of that name shipped, a `\label`/`\bibitem` in a file this document reads — strip comments and post-`\end{document}` text, exclude xr\'s other documents);
- a label a macro makes classifies UNDEF, so read those too.
- Residuals and the brk94/brk96/brk100/brk102 tails: [`perfect_kernel/OPEN_ROWS.md`](OPEN_ROWS.md) § "Breakage residuals".
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Complete breakage at 100 % (user 2026-10-24)".

### FM S2b: the observer's writer on by default for revtex4_support (OD #837 "S2b r1") — the residual

- **FM S2b: the observer's writer on by default for revtex4_support (OD #837 "S2b r1") — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left 2608.23944 (an uncarried symbol line under ruling 2 in a one-author group: a ruling) and 2602.16003;
- open (ruling): the mark-picked path does not show a line's mark (219 guards).
- State: OPEN (residual)

### K11 store residuals after 95d

- **K11 store residuals after 95d** (OD #939-#941; the scorer's four lost-words papers 2607.06973, 2608.26053, 2307.16262, 1202.2889 fixed; 95d-r2: a setter's own store only when no running code reads it, a date's label in an `\else` branch, an unlabelled date unnamed, fancyhdr styles read not run; 95d-r3: a date's label read in any code the title page reaches, a label phrase ending at a `;` or `,`, the label before a store's emptiness test, a role's default only where its word is printed right before the store): unsupported shapes, each a one-line input — (a) a footer token register selected only at the class's top level or in `\AtBeginDocument` (`\pagestyle{firststyle}` outside `\maketitle`), or set by `\fancyfoot[L]{\the\correspondingauthor}` under `\pagestyle{fancy}` outside `\fancypagestyle` (fancyhdr's fields stay no-ops), stays unread;
- (b) the same classes' `\paperurl` / `\reportnumber` registers (deepseek.cls:113-114, googlecloud.cls:81-82) are not in the table;
- (c) a macro store a `\ps@` style prints under another name (`\def\correspondingauthor#1{\gdef\@corr{#1}}` + `\def\ps@firststyle{\def\@oddfoot{\@corr}}`) is no store (its readers are no title-page code);
- (d) a footer toks value set after `\maketitle` (pdflatex prints it at shipout) is not read;
- (e) a register printed under a switch the class leaves false (`\ifshowca … \the\correspondingauthor\fi` + `\showcafalse`) still makes the note — `printed_register` knows only the `\if\relax\the\X\relax` test (RED `toks_store_printed_under_a_false_switch_is_no_note`, rev95d S4; fix: read the printing branch live or dead, as `reads_of_branches`);
- (f) a preamble `\fancypagestyle{firststyle}{\fancyfoot[L]{Preprint}}` that drops the register still makes the note — the style is read at the class's end (RED `toks_store_page_style_the_document_redefines_is_no_note`, rev95d S5; fix: re-read `\ps@<style>` at `harvest_stores`);
- (g) a setter's own store no macro reads, or one read by a page style the class never selects, is still rerouted — kept for astlb.cls:361-372's `\@shorttitle`, printed only by a running head its top level sets (1202.2889's `<toctitle>`) —, so a dead store (`\def\keywords#1{\gdef\@keywords{#1}}`, nothing reading `\@keywords`) is an invented item;
- whether an uncalled macro runs is not read off the code (`read_where_nothing_runs` takes any reader the never-running code reaches: cesj.sty's unselected `\cejpmaketitle`, physics0510010);
- (h) a label printed through a macro that reads the store (ujp.sty:107/:421 `\udkpacs~\udkpacsnum`, `\def\udkpacsnum{\@pacs}`; 1906.11531), or in a group that is a command's argument with layout the label reader does not take (CSML.cls:227-229's `\@footnotetext{\textit{\ACMCCSname:}\enspace\@ACMCCS.}`, the stand-in tpccs.cls:12; 1608.04592, 1602.00476), is not read: the classification is unnamed where the class prints "PACS", "2012 ACM CCS:" (review rev95d-r2 finding 3; charter-correct, better hidden than mislabelled);
- (i) `\fancycenter{a}{b}{c}` called in a style body outside a field (pdflatex typesets it where `\pagestyle` runs the style) prints nothing — the style runs nothing (int124a and Perl the same);
- (j) an unlabelled date among labelled ones is handed on as it is set, the labelled ones at `\maketitle` (`labelled_api` defers only a named one), so it comes first: medima's shape with `\@finalform` printed bare gives the final-form date before "Received";
- (k) a raw class's own store only an uncalled macro reads (`\def\makecover{… \@keywords …}`, never called) is still an invented item — the raw package's test (`read_where_nothing_runs`) is not applied to classes, where it lost bfhthesis's `\@department` (read through a `\@titlefooterleft` a scrlayer layer prints) and dmtcs's keywords (0905.1102) (RED `raw_class_store_only_an_uncalled_macro_reads_is_no_item`, rev95d p9);
- (l) a role's default kept because its word is printed right before the store (`ClassReaders::prints_default_name_before`: wiley2sp.cls:296-302's `\receivedname\ignorespaces\@received`, 1211.4450, 1306.2559; zaa.cls:513-515's `\AtEndDocument` hook `Received \rm\@received`, 1902.07328, the date also printed there, the class path's double print of (k)) leaves that printed label in the title-page deposit ("Received:" over nothing, as before 95d-r2; `perfect_kernel_batch95d::a_date_with_no_label_read_keeps_its_default_only_where_printed`), and a date stored right after a `;` or `,` (no label of its own: `\@revised; \@accepted`, iaus.cls's `No. \@volume, \@pubyear`, 0707.3527) leaves that separator there, the unnamed date first by (j);
- (m) a raw package's table store that no macro reads is still rerouted (`read_where_nothing_runs` takes it for one a running head its top level sets), so `\def\keywords#1{\gdef\@keywords{#1}}` with nothing reading `\@keywords` invents `<keywords name="Keywords: ">` and a received date (RED `raw_package_store_no_macro_reads_is_no_item`, review rev95d-r2 finding 2, probe qn; a static scan of the 712-paper package population found no real paper);
- (n) pazha2.cls's `\submitted` date ("Received December 15, 2022" in 2408.01858, "Received January 21, 2025" in 2607.07582) is in no output: `submitted` is not in STORE_SETTERS, its class is read raw and the refused `\endabstract` that runs `\maketitle` never prints the title page (one-line input: `\def\submitted#1{\gdef\@submitted{#1}}` printed only by a refused `\@maketitle` reached through `\endabstract`);
- OLDER than 95d — int123a and int124a lose it too
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K11 store residuals after 95d".

### 95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synthetic only, no census paper changed this way)

- **95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synthetic only, no census paper changed this way)** — (S1, a content loss on its shape) the `\else`/`\or` whole-test removal (`frontmatter_stores.rs` `line_label_before` ~:5302-5315, `end = past` ~:5349-5353, `enclosing_conditional` ~:5492) checks only that nothing precedes the store in its branch, so text printed after the store inside the test leaves the title page with it and is printed nowhere: `Received: \if@twocolumn\else\@received\ (preprint version)\fi` loses "(preprint version)" (base and r2 keep it; a static scan of the 9,502 class-shipping census zips found no such shape) — fix: take the whole test only when the store's branch holds just the store and layout and the test is the store's own emptiness test, else the r2 path (the `\ifcase`/`\or` variant too);
- reverting that arm alone restores r2 and turns only the witness-less guard `a_date_labelled_before_its_emptiness_test_is_named` RED;
- repro `sectioning-frontmatter/raw_class_date_branch_with_trailing_text_keeps_it.tex`.
- (S2, markup) the `;`/`,` clause split (~:5372-5385, meant for pazha2's date clauses) cuts every store's label: `Index terms, keywords: \@keywords` → `name="keywords:"` + a stray "Index terms" paragraph — split only after a clause holding a store or value, or only for date/classification calls, and say so in OD #940;
- repro `raw_class_label_with_a_comma_stays_whole.tex`.
- (N1) `text_right_before` reads every branch of a name macro (a German `\recname` keeps the English default; base the same).
- Pre-existing, from the review: 2109.14785 (ectj.cls:352 commented out) emits dates it never prints — a real-paper witness for (m) above;
- 1409.1462 loses the printed "(January 2015)" set by `\received` inside `\affil` (base and r3; content loss);
- 2210.05326 a stray "Received:" paragraph.
- Review `agent_reports/2026-10-31_rev95d_r3.md`
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synt".

### 95e residuals (OD #938)

- **95e residuals (OD #938)** — the several-address `\email` give-back (2109.07363 three creators again, 0 E) leaves: (a) `\address{$1.$ …}` labels unread, so the reading's upfront grid gives the entries by position, its pile lacking the listed call (`answer_listed_emails` runs after `Resolver::distribute_upfront`): 2109.07363 Liu {1}, Tao {2} where the PDF prints {1,2} for both (RED `amsart_dollar_numbered_addresses_under_shared_marks`), synthetic reversed marks both wrong where int123a had one wrong (RED `amsart_dollar_numbered_addresses_under_reversed_marks`, rev95e S1);
- fix: the call one contact in the grid's pile, as today's, or the `$1.$` label read;
- (b) a listed call's address spelling no one is the call's author's (`owner:call`: 2109.07363 `ferrytau@` on Wei, Tao by the list's order);
- (c) the fail-safe keeps every listed call whole when one cannot be written out (RED `amsart_several_address_emails_one_kept_whole_keeps_all_whole`, rev95e S3; = int123a);
- (d) revtex `\author{Ann Able $\&$ Bob Baker}` stays one creator with no `merged_creators` Error (= int123a)
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95e residuals (OD #938)".

### Unlanded payload branches (inventory 2026-10-31; all work reintegrates into perfect_kernel, user 2026-10-31)

- **Unlanded payload branches (inventory 2026-10-31; all work reintegrates into perfect_kernel, user 2026-10-31)** — each kept on its branch for a recorded reason, none silently: `pk/66p` 41122534fa (a sub/superscript mark outside math, tex.web §1046; ruled unlanded, `feedback_rulings_error_fatal_policy`);
- if not, retire with its residual rows.
- **RETIRED 2026-11-01** (row "K9 row F residual": 264 Errors in 29 of its 62 witnesses on the pushed tip).
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Unlanded payload branches (inventory 2026-10-31; all work re".

### K9 row F residual: a macro that closes and reopens the author block's environment (66u RETIRED 2026-11-01, user ruling 2026-10-31)

- **K9 row F residual: a macro that closes and reopens the author block's environment (66u RETIRED 2026-11-01, user ruling 2026-10-31)** — `\author{\begin{tabular}{c}Ann\end{tabular}\hskip1em\begin{tabular}{c}Bob\end{tabular}}`-style blocks (sig-alternate `\alignauthor`, hand-made `\end{tabular}…\begin{tabular}` breaks) read as `\and` by pdflatex leave a run of Errors: on 66u's 62 code/test witnesses (`pk/66u` `perfect_kernel_batch66u`; job tmp `w66u.txt`, `m66u/run.sh`) the pushed tip (int125c `8a87df771c`) gives 264 Errors in 29 papers, 33 error-free (int123a 272 / 32; the bundle fixed 2405.18462) — `unexpected:\hbox` 66, `\lx@personname` 52, `\endgroup` 52, `\@@tabular` 52, `\@end@tabular` 39, `&` 3;
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K9 row F residual: a macro that closes and reopens the autho".

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

- **Frontmatter on the digested representation — the freeze and the refactor's batches 90a-95c (task #423; now `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, row 0b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `amsart_email_of_a_two_name_call_spelling_an_accented_name` (2112.10604) and `revtex_email_spelling_its_holders_given_name`;
- REDs' witnesses 0711.3633, 0809.3090 (`santos@` off Santos Bravo Yuste);
- amsart's routed `\thanks` (the row "A routed amsart note").
- open RED `raw_class_store_trailing_line_after_last_mark` (quant-ph0101015's rspublic.cls store line after the last mark);
- left: 1304.1574 (`Computer Science` read as names), llncs's 1901.07288 (a trailing-mark `\institute` whose entries span two lines), 1505.06588 and 2203.08964 (scorer source-reading limits, ours the PDF's).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Frontmatter on the digested representation — the freeze and ".

### A routed amsart note belongs to the last live author at `\maketitle`; needs a change tag and a ruling

- **A routed amsart note belongs to the last live author at `\maketitle`; needs a change tag and a ruling** (92a review rev92a_r3 s1; main-session decision: defer to the ams written-family work): amsart.cls:516 keeps each `\thanks` in `\thankses`, run at `\maketitle`, and its `\email` (:508) appends to `\addresses`, so a `\thanks{Correspondence: \email{x}}` between two authors, or before every `\author`, prints x after the last author's address.
- RED catalog repros `class-bindings/amsart_thanks_between_authors_email`, `amsart_thanks_before_every_author_email`
- State: DEFERRED (ams written family)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A routed amsart note belongs to the last live author at `\ma".

### Compound marks: one rule for the superscript path, the formula path's runs and the legend's label (94a r9 residual; OD #928 round 9, OD #927)

- **Compound marks: one rule for the superscript path, the formula path's runs and the legend's label (94a r9 residual; OD #928 round 9, OD #927)**: the three readers part a mark of several symbols differently — the superscript path at commas only (`apply::show_legend_mark`'s `ltx:sup` loop: `Ann Able$^{*\dagger}$` under a `$^{*}$Corresponding author` legend shows "*†" beside the note's ∗, the star twice, once as ASCII `*`; w01), the formula path by `token_runs` and `repeated_symbols` (`$^{1,*\spadesuit}$`'s ∗ taken with the comma before it, "1♠"), the reading's labels by `Entry::compound` and `mark:parted` (`**` read as `*,*`, so a `$^*$` legend is the `**` carrier's, w11; the kernel then hides the `**`: `\author{Ann Able$^{1,**}$, Bob Baker$^{1}$\\ $^1$Univ X\\ $^*$Corresponding author}` prints "1,∗∗", ours no mark, as 93a), and a `$^{*\dagger}$Corresponding author` legend reaches Bob Baker, who carries only `*` (`Ann Able$^{1,*\dagger}$ and Bob Baker$^{1,*}$`, w10, since 93a).
- Constraints any rule keeps: 2602.19190 (a run of distinct symbols labels a line as a whole), 2606.15609 (`$^{*\spadesuit}$` requests each symbol), a repeated symbol a mark of its own (94a r9 guard `a_repeated_symbol_beside_a_legend_is_its_own_mark`).
- A partial take also drops the comma before the taken symbol: `$^{1,**\dagger}$` under a `**` legend gives "1†" where pdflatex prints "1,∗∗†", and `$^{1,\dagger\ddagger}$` under `†` gives "1‡".
- Remove the comma only when the whole comma-delimited run is taken (`apply.rs` ~l.258; rev94a r9 r12).
- RED catalog repros: `author_names_compound_mark_superscript_beside_its_legend`, `author_names_compound_legend_goes_to_a_partial_carrier`, `author_names_kernel_repeated_symbol_beside_a_legend_hidden` (the kernel path's `**` hidden; rev94a r9 a03, siblings a01, a05, a10, a11)
- State: OPEN (refactor)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Compound marks: one rule for the superscript path, the formu".

### A trailing answered name takes the unmarked in-place line, before its own (94a r9 residual P3)

- **A trailing answered name takes the unmarked in-place line, before its own (94a r9 residual P3)**: `ANN ABLE$^1$ and BOB BAKER and CID COLE$^1$ \affil{$^1$Univ A} \affil{Univ B}` (a class printing `\affil` in place) gives Univ B to BOB BAKER and CID COLE, CID COLE's Univ B before his Univ A;
- RED catalog repro `raw_class_affil_run_trailing_answered_name_takes_the_unmarked_line`
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A trailing answered name takes the unmarked in-place line, b".

### IEEEtran's `annotate=new` affiliation block after a marked one, contact order (94a r9 residual P4; since 93a)

- **IEEEtran's `annotate=new` affiliation block after a marked one, contact order (94a r9 residual P4; since 93a)**: `\IEEEauthorblockN{Ann Able$^1$ and Bob Baker}\IEEEauthorblockA{$^1$Univ X}\IEEEauthorblockA{Univ Y}` gives Ann Able Univ Y before Univ X (the print's order X, Y);
- the same with `\IEEEauthorrefmark{1}` (review rev94a r8 z11, z07).
- RED catalog repro `ieee_affil_new_block_after_a_marked_name_contact_order`
- State: OPEN

### A raw class's line whose mark only a later `\author` requests stays the class's line (94b r7 residual; it was 94b r4's S1)

- **A raw class's line whose mark only a later `\author` requests stays the class's line (94b r7 residual; it was 94b r4's S1)**: in an appending class (capa-like `\let\@author\@empty\renewcommand\author[2][]{\g@addto@macro\@author{#2 }}`), `\author{Ann Able\inst{1}}\address[\inst{2}]{Univ B, City}\author{and Bob Baker\inst{2}}\maketitle` prints "(2) Univ B, City" where it is called and Bob Baker gets no contact: at the call no author requests 2, so the line is in doubt — the class's, digested once (`frontmatter_stores.rs` `class_contact`, OD #936 round 7).
- The three tries to decide it later (94b r5 a replay after the title, r6 a call kept beside the printed line and lifted) each reordered the body or digested the line twice ("Body 2." for pdflatex's 1);
- no census paper takes the shape (300 setter census, cpc-hepnp 60, the 3,000 sample).
- Review rev94b r4-r6 probes `p/m/later`, `p/n` aug/tc/two/l3, `p/u` u1-u3, `p/w` w1-w2, `p/s/s4`.
- RED catalog repro `raw_class_optional_mark_a_later_author_requests`
- State: OPEN (deferred)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A raw class's line whose mark only a later `\author` request".

### Per-entry wraps of nested containers, and an `\href` around entries (94b r7 review residuals; synthetic, triage 2026-10-31: RED catalog, no fix round)

- **Per-entry wraps of nested containers, and an `\href` around entries (94b r7 review residuals; synthetic, triage 2026-10-31: RED catalog, no fix round)**: (1) a container inside a kernel text font command with more entries after its group loses the container's font (`\textbf{\emph{$^1$…\\ $^2$…}} $^3$…`: bold kept, italic lost; `emitter/plan.rs` `entry_forms`/`container_wrap`);
- (2) `events/fonts.rs` `font_declaration` matches by name, so a redefined `\bfseries` (`\quad`) is copied into every entry (the B2 class: what prints once copied into each entry) — compare meanings (the S3 `x_equals` pattern);
- (3) the allow-list between nested containers (`plan.rs` ~:681-687) refuses a text font command between containers, `\relax`, `\fontseries{b}\selectfont`, `\LARGE`/`\huge` (plain cut, = base);
- (4) `\href` around the entries (± a font command) → a nameless creator holding the contact and two names merged (pre-existing since 94a r9).
- RED catalog repros `raw_class_affil_text_font_container_then_more_entries`, `raw_class_affil_redefined_font_declaration_between_containers`, `raw_class_affil_text_font_command_between_containers`, `raw_class_affil_href_around_entries_nameless_creator` (review rev94b r7 probes p/z/bt, rds, ebun, ihref; p/x/href)
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Per-entry wraps of nested containers, and an `\href` around ".

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

- **frontmatter notes read as the class prints them (92e, 92g) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): the version-dependent shapes left are elsarticle's `\journal` (copies up to v3.3 print "Preprint submitted to X", 2609.40131; TL 2025's v3.4c none) and elsart's ("Article published in …" when `\volume` is set);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "frontmatter notes read as the class prints them (92e, 92g) —".

### An unmarked address after a name list goes to the last name only (iopart, informs4; pre-existing, Perl's default)

- **An unmarked address after a name list goes to the last name only (iopart, informs4; pre-existing, Perl's default)**: `\author{A, B and C}\address{X}` (iopart) and `\AUTHOR{A, B}\AFF{X}` (informs4) give X to C (B) alone, where the print sets X under all of them.
- For iopart this is Perl's: `\lx@add@affiliations` (Base_Utility.pool.ltxml:742) attaches to the last creator by default.
- Candidate: the same default 91a's just binding opts into (`annotate=all`/`run`: an unmarked address belongs to the run of names it follows), declared per binding;
- found by 91a r2's sibling audit (the other `\lx@add@affiliation@marked` callers already annotate or reach it only through a marked branch)
- State: DEFERRED to frontmatter refactor stage 6/7

### Frontmatter correctness scorer (2026-10-27)

- **Frontmatter correctness scorer (2026-10-27)**: our creators against the paper's own print (pdflatex TL 2025, pages 1-2, sources and outputs only — no arXiv API, user 2026-10-27), on 3,000 random arXiv papers (`~/data/pk_agents/main/fm_scorer/`: `score2.py`, `conv2.sh <id> <bin> <outroot>`, `view.py`; seed 20261027).
- a hand-typeset title page with no `\author` and no creators 6.9 % (178; all 8 sampled print their authors; Perl the same — a beyond-Perl capability, needs a ruling);
- residuals in `parity/DEFERRED_FAMILIES.md` "Abstracts a class stores or queues";
- open (RED repros `class-bindings/llncs_hand_marks_with_several_institutes`, `llncs_author_line_brace_list_email`, `llncs_marked_block_ambiguous_spelling_is_no_place`): several institutes with hand-typed name marks (2207.06953), an author-line `\email` brace list split inside its braces, or glued to the last name (1609.01220), the general author-block reader's `addresses_in_order` (`run_in_author_order`, author_block/affiliations.rs) placing an address several names spell by its place (`lee@` under Ann Lee, Bob Lee, Cid Cole goes to Cid Cole; `row_address_owners` has the fix) — a later batch with its own A/B, the reader being every class's;
- the r5 re-review's picked-path residuals (OD #901: a "Corresponding author" note on an unmarked author taking an unspelled picked address, RED `llncs_corresponding_note_does_not_override_inst_marks`; a shared line's word read as the e-mail of the note beside it, RED `llncs_shared_note_line_word_is_no_email`; one unspelled address alone in a call under several picked authors stays every picked author's — charter §4 says the block, awaiting a ruling);
- Largest unclassified buckets: an `\altaffiltext{n}` no name requests (13 papers) — many are the title's note (`\title{…\altaffilmark{1}}`, 0801.1670, 0808.0471, 1006.1591, 1010.0340: "Based on observations …"), which today's XML keeps in `<title>` and the reading sends to the block: the reading needs the title as an owner of a labelled entry, and the comparator and writer with it, before anything is written;
- labelled contacts by `\altaffilmark` (9 papers; 0707.2093, 0804.2539, 0911.4695 to read against their PDFs).
- State: OPEN (rulings)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Frontmatter correctness scorer (2026-10-27)".

### dev giant-list census (2026-11-01) — the residual

- **dev giant-list census (2026-11-01) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): BESIII `(A)`/`(C)` sub-labels deferred (R6, the difficulty rule);
- M3 PoS's one `\@author` (1711.04132, R9, settled by Z14);
- scorer limits L1-L4 (1201.5677, 1904.03257, 2510.23732).
- Also L4 math in a source name (2310.11912) and L5 a colon heading kept in the source name (2206.00684);
- aa's `X Collaboration\thanks{…}: R.~Abuter\inst{8}` keeps the collaboration an author (2206.00684, 97f optional).
- State: OPEN (residual)

### Holdout10k v3 (2026-11-01; spent: any further read is a fresh holdout v4, read once at the end — the plan's rule, user 2026-11-06; the rules below stay the judge's)

- **Holdout10k v3 (2026-11-01; spent: any further read is a fresh holdout v4, read once at the end — the plan's rule, user 2026-11-06; the rules below stay the judge's)**: **Papers with an Error or Fatal in their frontmatter are excluded from every count (user 2026-11-01: "completely exclude papers with errors in frontmatter from this work, they confound the results. Errors elsewhere are OK to keep")** — `score_tiers.py` `frontmatter_error`, status `fm_error`: an Error/Fatal located in the main file before the first sectioning command, in a class or package the preamble loads, or in a file input before that point, or an `ltx:ERROR` inside a frontmatter element;
- `fm_scorer/draw2/holdout10k.txt` from `draw3.py` (seed 20261102) — 10,000 papers stratified by (in fleet run 336's error list or not) × submission year at the universe's shares (6.14 % run-336 error papers, universe 6.15 %), drawn among unseen papers only: every id of the first draw's exclusions, the 3,000 sample, dev20k, devh10k and `seen.txt` (222,086 ids any session converted, read or named — pk_agents, pk_wt, the job scratch, memory and the repo; fleet-scale error dumps nobody read paper by paper left out, the first draw's own exclusions having been 26.8 % run-336 error papers);
- `reserve3.txt` (2,000, same strata) is `replace.py`'s pool.
- The bar (row above): 98 % per creator AND per paper, each with its paper-resampled 95 % interval (`score_tiers.py` `paper_bootstrap`, report lines "Per creator" / "Per paper" / "Bar"; test `test_2026_11_01_d3_reports_per_paper_and_a_paper_resampled_interval`), and every residual lost word counted by a `Warning:frontmatter:unclaimed` (user 2026-11-04, replacing "0 papers with lost words"; FIDELITY_CHARTER §4 amended).
- Measured at milestones only, headline numbers only;
- no shape of it picks work.
- Dev = dev20k + devh10k.
- Writers' A/Bs: the class-stratified `ab_66bundle/popint125a_s10k.tsv` (10,574 of 28,439 papers, every class; `scripts/stratify_pop.py`) plus the touched binding's papers;
- the full population with `fm_checks.py` once per bundle **Dev re-measure (2026-11-04, pushed `ccd3b7067e`, binary int129a; holdout still sealed):** dev20k per creator **97.98 %** (38,187 of 38,974; paper-resampled 97.64-98.31 %), per paper **97.33 %** (7,865 papers; 97.00-97.63 %), **68** lost-words papers (int126b: 97.22 / 96.67 / 81);
- **Switch rule (user 2026-11-01):** v3 stays sealed until dev (dev20k + devh10k + the long-tail dev set) reaches 98 % on both measures;
- then v3 is measured, and if short it is spent (becomes dev) and a fresh v4 (10k, same stratification) is the final judge.
- Holdout reports may show bucket totals by class frequency in dev (50+ / 5-49 / 1-4-paper classes), never papers or class names.
- State: OPEN (the exit bar's judge)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Holdout10k v3 (2026-11-01; spent: any further read is a fres".

### a marked author list merged by a name the split test does not know (97a) — the residual

- **a marked author list merged by a name the split test does not know (97a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a `NON_NAME_WORDS` word in a surname keeps a mark-covered list whole (1212.4143);
- reversed marked names `Surname, I.\altaffilmark{1}` (1903.10014);
- a `(B)`-led institute makes a nameless extra creator.
- State: OPEN (residual)

### dev20k `aff_missing` census (int125b, 2026-11-01; 451 creators / 146 papers: 390 / 114 classed, 61 / 32 long tail, ≈43 scorer limits)

- **dev20k `aff_missing` census (int125b, 2026-11-01; 451 creators / 146 papers: 390 / 114 classed, 61 / 32 long tail, ≈43 scorer limits)** — **F1** (110 / 29) the family's writer gate is shut while the reading is right (inst_support 69 / 14 — the names census's M3; the jheppub family jheppub/jcappub/jinstpub 30 / 9; undeclared policies 11 / 6: aas_support, spie, emulateapj, PoS; `inst_support_sty.rs:11`, `jheppub_sty.rs:11`, `latexml_contrib/src/jcappub_sty.rs:14`, `jinstpub_sty.rs:18`, `policy.rs:122-127`, gate `writer/mod.rs:45-71`; witnesses 2302.00492, 2303.05792, 1607.07434, 1509.06963, 0807.0164; fix: each family through the OD #837 S2b gate, then `written: true`);
- **R1** (58 / 16) a raw class's in-place affiliation setter with no line break is never read (iopjournal `\affil` = `{\fontsize{8}{10}\selectfont\raggedright #1}`, 28 / 9: 2607.12158, 2609.08534; `frontmatter_stores.rs:1972-2010` `in_place_contact_setter`, `sets_a_line` :1989; caosp/iau/ETHpaper stores HARD);
- **W2** (31 / 8, MED) the writer declines "names differ" silently where today's output makes address lines persons and the reading is right (2410.09484, 2507.05173, 2105.11228, 1009.4816, 2512.19748; `emitter/plan.rs:1421`, `repart.rs:242-400`, `apply` :1375 returns false without a reason line);
- **J1** (11 / 3) contrib jfm makes `\affiliation` a document note (`latexml_contrib/src/jfm_cls.rs:302-305`; 2211.03175, physics0607157, 1411.5202; full fix needs F1's inst_support gate);
- **W3** (10 / 5) one unexplained contact difference declines the whole paper — ORCID keyed `httpsorcidorg0000…` vs `0000…` (`plan::contact_key` :256 not using `dom::orcid_id` :674; 2503.10804, 1903.03238, 2512.20289; EASY) and url/e-mail owner changes (2608.26140, 2407.19670, 2501.11284: per-item decline is a design question);
- **T1** (9 / 3, HARD) marked affiliations inside an IEEE `\thanks` (1903.05481, 1410.6355 — one `\thanks` holding `\newline` lines; 2411.05212 FIXED 100c: a line of marked `\thanks` after `\`, OD #957);
- **U1** (8 / 2) an unmarked affiliation over a `\\` becomes one contact per line (2509.16245, 1304.1996; `blocks.rs:752` marked only);
- **D1** (6 / 2) raa `\institute{… {\small Received …}}` (2102.01352, 2110.11117).
- SL-sym (12 / 3) symbol and parenthesised marks ignored (cond-mat0611096, 2601.21839, 2404.11930);
- SL-mail (9 / 2) an e-mail line inside `\affiliation` read as affiliation text;
- SL-legend (llncs legend folded into the institute).
- Per-paper data: job tmp `devaff/`
- State: open (worklist; 97c takes W1, L1, SV1)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev20k `aff_missing` census (int125b, 2026-11-01; 451 creato".

### dev20k `name:unmatched` census (int125b, 2026-11-01; 284 creators / 103 papers: 198 / 44 ours, 86 / 59 scorer limits)

- **dev20k `name:unmatched` census (int125b, 2026-11-01; 284 creators / 103 papers: 198 / 44 ours, 86 / 59 scorer limits)** — by creators: **M1** (61 / 10, EASY) a whole-block `tabular`/`minipage`/`\parbox` author block with e-mail lines stays ONE creator, mostly silent: the re-part cannot write an e-mail item (`observer/emitter/repart.rs:183-215` `item_source`, declines :449/:522; fix `Made::Email` at `plan.rs:46` + an `apply.rs` arm; witnesses 2605.09461, 2507.07465, 2609.36484, 2603.10436, 2608.20401; repro `\author{\begin{tabular}{c}Ann Able$^{1}$ \quad Bob Baker$^{2}$\\ $^{1}$Univ A \quad $^{2}$Univ B\\ \texttt{ann.able@a.org} \quad \texttt{bob.baker@b.org}\end{tabular}}`; an address no name spells needs a ruling: a nameless creator) — FIXED 100c for the addresses a name spells (OD #958, `Made::Email`: 2507.07465, 2603.10436, 2608.20401, 2609.39183);
- an address no name spells (3B: 2605.09461, 2609.36484) still declines the re-part, row "100c";
- **M5** (23 / 6) six writer declines on boxed or grid blocks (2107.12423 `\resizebox` size read as name ink `0pt11304960` — FIXED 100c, `\Gscale@@box` keyed, OD #958; 2603.24245 `{\color{red}*}` legend; 2003.04668, 2101.00440, 1708.03035, 2402.14804);
- **M3** (21 / 6) the inst_support family unwritten (egpubl, aprim10: 1906.07409, 2503.03953, 1802.10123; `inst_support_sty.rs:11` `written: false`; the OD #837 S2b gate);
- **M9** (21 / 6) no creator (SolarPhysics `{opening}`, memsait, renamed ICML copies routed raw — a routing ruling);
- **M7** (15 / 5) one per-author `\author` holding several names (authblk `\quad`, acmart `\hspace`, fairmeta silent: 2412.03072, 1901.08289, 2510.01161);
- **M2** (9 / 1) a `\thanks` after a layout box (2604.15336);
- **M8** (9 / 2) reversed `Surname, I.` split at the comma (1409.4999, 2501.09580; the re-part only refines, `repart.rs:317`);
- **M10** sdl `\names{}` (1905.00862);
- **M6** (3 / 1, EASY) acmart `\bfseries\large` before the names breaks the `\\` continuation (2608.14026; `acmart_cls.rs:246-253`, `names_list_continues`);
- **M11** `\textsc{\char13}` prints ﬂ (2003.10147, cmcsc10 '013 is a quote).
- Scorer limits: S1 duplicate source `\author`s (supplements, 32 / 9: 2003.12024), S4 names the source parts merged by the scorer (28 / 27), S3 extra text in the source name (14 / 11), S2 name text (Math, 8-bit encodings; 12 / 12).
- Side finding: renamed ICML copies print their layout check ("marginparsep has been altered…") because article leaves `\marginparsep`/`\marginparpush`/`\topmargin` at 0 (`latex_constructs/sect05.rs:896-910` = Perl; TeX sets them in size10.clo:95-168; 2502.01411, 1903.03536, 2110.08350).
- Per-paper classes: job tmp `devnames/classes.tsv`
- State: open (worklist)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev20k `name:unmatched` census (int125b, 2026-11-01; 284 cre".

### dev lost-words census (int126a-pre, 2026-11-01; dev20k + devh10k, 171 flagged papers: 101 real, 69 scorer artefacts, 1 deferred)

- **dev lost-words census (int126a-pre, 2026-11-01; dev20k + devh10k, 171 flagged papers: 101 real, 69 scorer artefacts, 1 deferred)** — every paper loses exactly the same words on int125b, 97b-rel and int126a-pre (97c-97f neither fixed nor broke one).
- → the 2026-11-01 ruling: unclaimed printed text at the block as a note with a counted Warning (FIDELITY_CHARTER §4), plus API routing for the large families;
- **F** smaller (daj, quantum e-mails, 1605.04229 deferred as pathological, 2603.24729 `$^{\textrm{…}}$` lines, INTERSPEECH 2103.17142).
- Real, by mechanism: **A** (69, HARD as a family, RUST-ONLY in most) a raw class's, package's or document's own stores that only its title-page code prints, refused or replayed with the fields emptied — copernicus/authblk-code classes (`\AB@affillist` never read: 0712.1312, 1401.4706, 1709.00722, 2310.11002 — its JOSS sub-family, a copy of authblk's own `\affil` over the binding's (2310.11002, 2504.08081; oasics 1709.00722), FIXED 102a by the binding's lock, OD #959; the kernel `\author` overwrites copernicus.cls:1349's `\@author`, the replay empties it `sect05.rs:1431-1435`), raa (abstract and keywords lost whole: 1404.3644, 1304.5354 — `\RAAabstract` store, deposit dropped for 3 held errors `sect05.rs:1390-1399`), sig-alternate `\titlenote` (`\tntoks`, printed only by the class's `\abstract`: 1206.6940, cs0405006), ~30 more classes with their own setters (aaskaii `\emailAdd`, cccconf, cup-journal, ametsoc `\authors`, ptapap, memsait, putex);
- On the 3k sample (97f r2 run): lost-words papers 1 → 2 (2402.17260's `\Y4Sdecays`, a delimited `\def` the scorer does not expand — scorer limit), D3 unchanged 98.23 %.
- Artefacts left (26): glued/split kept text 8, accent splits 6, aastex7 hidden e-mails 3, label words 2, one-offs 7.
- State: open (98b; A needs the net)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev lost-words census (int126a-pre, 2026-11-01; dev20k + dev".

### the unclaimed-text net (int127a; FIDELITY_CHARTER §4) — the residual

- **the unclaimed-text net (int127a; FIDELITY_CHARTER §4) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): deferred cccconf 1702.07637 and the 1-2-word losses below the threshold;
- stores at their class default (tADR2e 1509.08973, cup-journal 2302.11911) no candidates, the placeholder ruling pending.
- Open: R, routing a class carrying authblk's code without the package through the authblk binding (copernicus 0712.1312, esapub2005 1401.4706: the net's notes today);
- N3, note lines split only at direct-child breaks (2607.26558);
- left: a `\email` the class sends to an unprinted `\@thanks` (1304.5354, astro-ph0609644, astro-ph0612749).
- State: OPEN (residual)

### 99b: a raw class's title-page affiliation or address (OD #955; int127a, 100a) — the residual

- **99b: a raw class's title-page affiliation or address (OD #955; int127a, 100a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED `run_in_lines_a_run_places_keep_their_order`;
- the "Deceased" ruling and rev99b2 F2 (its item (f), int127a's next round).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "99b: a raw class's title-page affiliation or address (OD #95".

### 100a: hand-printed letter notes and unmarked note lines after a marked list (99a's Cut)

- **100a: hand-printed letter notes and unmarked note lines after a marked list (99a's Cut)** — OPEN, RED.
- Tentative ruling (main session 2026-11-02): a hand-printed `x)` some name's mark requests is that line's entry label, read by the Cut's entry reader (marks win);
- Deferred from 100a r2: only `a)` leads a line — `b)`…`g)` stand mid-line (source newlines), so the reading is of mid-line hand-printed letter labels, a new reader beside the Cut's line-leading one, in `author_block/affiliations.rs` (batch 100b's file).
- hep-ex0106079 (moriond.sty `\address{… $^{12}$ YerPhI, Yerevan, Armenia.\\ a) also at UCEH … g) on leave of absence of YerPhI …}`, moriond_web.tex:160-170): the Cut reads superscript marks only, so the `a)`–`g)` notes are the text of entry 12 and its five YerPhI authors each hold all seven, while Abreu (`$^{6,a}$`) and the other requesters hold none;
- sibling (review rev100a c4): an unmarked trailing `Both authors contributed equally.` joins the last entry's affiliation.
- a trailing line no mark leads and no name requests continues the last entry only if it reads as an address part, a note (`classify_thanks`) goes once to the block.
- RED repros `displaced_hand_lettered_notes_after_the_marked_entries_go_to_their_requesters`, `displaced_unmarked_trailing_note_line_is_the_blocks`.
- **rev100a2 residuals (SHIP; synthetic):** a displaced store with one unrequested symbol line (`$^\bigstar$Deceased`) keeps the whole store in the body, its requested lines losing their carriers (RED probe `rev100a/s/dsu08`; fix = per-entry doubt);
- the "opens with Deceased" exception reads `$^\dagger$Deceased Estates Unit, Univ C` as a note (probe `rev100a2/v/v5`; decide on the first clause, "Unit" an institution word);
- a requested ♮/♭ "Deceased" line is an affiliation while ★ is a note (`is_footnote_symbol_operand` narrower than `prints_a_mark`);
- OD #955 (f): a math `$^*$` gives `mark="*"` while the sup prints ∗, and the U+2217-only maps in `post.rs:836` and `lexer.rs:387` are unnamed there.
- State: OPEN (RED)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100a: hand-printed letter notes and unmarked note lines afte".

### 100c: two Link fixes the observer read right and its writer declined (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`)

- **100c: two Link fixes the observer read right and its writer declined (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`)** — FIXED: (6) a line of marked notes only after a marked block's names line and its `\\` (`\\ \thanks{$^{1}$…}`, also `\\[3.0ex]`, `\\*[len]`, a spacer `\\`) joins the names line, its notes the names' (`author_block/preprocess.rs` `notes_line_joins_names`, `notes.rs` `notes_only_line`; IEEEtran's `grade_only_line` now "one grade, then `notes_only_line`"; OD #957, KNOWN_PERL_ERRORS #1206 — Perl shares it; witnesses 2609.17106, 2411.05212, 2312.10807).
- An unmarked block's line of unmarked notes stays the document's (86d's reading; control repro `notes_only_line_after_unmarked_names`), and so do notes after an `Emails: …` line, whose list joined was no list (1911.03356; control `notes_only_line_after_an_emails_line_stays`).
- The `author_mark_position` / `has_author_marks` disagreement (`frontmatter/classify/marks.rs:56` / `frontmatter/fallback.rs:114`, a `^` inside `\thanks`) stays, cited at the preprocess.
- RESIDUALS, pre-existing, base too: the re-part's written affiliations drop the block's `\itshape` (base keeps `<text font="italic">` where the re-part does not run);
- an e-mail row continuing a marked legend line becomes a creator with no name (RED `marked_legend_email_row_is_no_creator`, sibling of 3B);
- a marked "John Smith Laboratory" affiliation merges the names with a `merged_creators` Error (RED `marked_affiliation_named_laboratory_merges_names`).
- **Open:** (cells) the resolver's `joins_previous` (`resolver/lines.rs:668`, rule `name:layout-joined`) joins unmarked names in adjacent `tabular` columns into one name (`Ann Able & \mbox{\ \ } & Bob Baker`, 1302.4168; also the affiliations below them, `Univ A & Univ B`), so the block stays one creator (RED repro `boxed_block_names_in_two_columns`);
- a cell boundary should part names (EASY in the resolver; changes readings beyond the boxed blocks, so its own A/B).
- (3B, deferred) an e-mail no name spells in a boxed block is a block-level contact (FIDELITY_CHARTER §4, 2026-11-01), which the writer has no form for short of OD #159's nameless creator — the re-part declines ("a block e-mail no name spells"), today's one creator stays (2605.09461, 2609.36484; RED repro `boxed_block_emails_no_name_spells`);
- needs a block-level contact in the writer's `dom::Front` (only creators today) and the API.
- (exposed) the e-mail item lifts the re-part's last decline on boxed blocks, so readings that decline masked now ship (no content lost, the scorer better or equal on each): 2609.37429 (`\FormatAuthor{name}{mail}{inst}` nested `tabular`s side by side — the second author read as a continued line of the first's affiliation, "EPFL Zihan Hu zihan.hu@epfl.ch EPFL"; declined since r2 by `joined_rows`, the reading's continuation across two nested `tabular`s in one cell still open), 2603.19216 (planstyle's `\affil` after a boxed `\author`: the reading's `api:annotate:run` gives it to the last name only; 7 names now right, 6 without the shared affiliation), 1302.4168 (names joined across cells, now declined);
- proposed one fail-safe for the main session: a boxed block's re-part writes only when every Name event of the block that reads as a person on its own (`may_name_a_person`) is exactly one reading creator's name — never part of another name or of an item (it would subsume `joined_across_cells`).
- (Kind) `\IEEEauthorrefmark{n}` prints the n-th footnote symbol (∗, †, …; IEEEtran.cls:4528-4530 `\@fnsymbol`), but the reader keys it as `n`, so legend notes `\thanks{$^*$Equal contribution}` / `$^\dagger$Corresponding…` answer no request (2312.10807: document pubnotes since r2, no longer the last author's);
- the charter's printed-label rule wants the symbol key, which would make them the refmarked authors' legends.
- (S2) 2603.19216's shared `\affil` after a boxed `\author` goes to the last name only (the reading's `api:annotate:run`; siblings 1107.1546, 1107.1548; all 7 creators right, no content lost).
- (q10) an unmarked note alone on its line under a marked names line (`Ann Able$^{1}$, Bob Bell$^{1}$ \thanks{$^{1}$Univ A}\\ \thanks{Funded by X.}`) is the last name's, int127a the same;
- FIDELITY_CHARTER R1 wants the block's (RED repro `notes_only_line_unmarked_under_a_marked_names_line`).
- (S3) revtex `B.~in der Baker$^{2}$, b.baker@two.edu$^{*}$`: the address's `∗` is read as the name's request, so the name prints `<sup>2,∗</sup>` and the made e-mail no mark — the made e-mail item would need to carry its mark (resolver attribution + writer; RED repro `author_marked_email_piece_mark_moves_to_the_name`; `perfect_kernel_batch97b::marked_address_piece_is_no_person` pins today's).
- (6') a line after `\\` holding a note and text (`\\ \thanks{$^{1}$Univ A.} $^{2}$Univ B, City`) stays a line read by the mark inside its note, the note the document's (RED repro `notes_and_text_line_after_the_names`, synthetic).
- (3C) 2603.24245's legend welded over `\quad` into `\small{{{Code: \url{}}}}` (a template placeholder) — the Cut residuals.
- **rev100c2 residuals (SHIP; synthetic):** B1's person-row decline counts a `\quad` gap as a row edge, so `\quad`-separated place names ("San Diego", "New York", "Los Alamos") decline a reading round 1 had right (RED probes `rev100c2/probes/b6.tex`, `b3`; only line and cell edges should count);
- unrequested-note pubnotes are appended after the block's calls, out of print order (probe `m7`);
- OD #957 should say 2312.10807's `$^{*}$` is printed by the names via `\IEEEauthorrefmark`.
- State: open (3B, cells, exposed, Kind, S2, q10, S3, 6', 3C; held FIXED 101b)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100c: two Link fixes the observer read right and its writer ".

### Z14's REDs and follow-ups (revZ14r2, SHIP)

- **Z14's REDs and follow-ups (revZ14r2, SHIP)** — OPEN.
- repro `sectioning-frontmatter/z14_affiliation_label_word_left_red.tex`.
- (2) **The narrowed heading row needs a person cue on every piece** (`labels.rs` `later_line_is_names`): `Cat Cole$^1$\\ Authors: Ann Able and Bob Baker\\ $^1$Univ A` and `Editor: A. Able, Bob Baker` read one creator with the row as an affiliation, as int143b;
- repro `z14_heading_row_plain_names_red.tex`.
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z14's REDs and follow-ups (revZ14r2, SHIP)".

### Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had their two fix rounds, so these go to the next reading batch)

- **Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had their two fix rounds, so these go to the next reading batch)** — OPEN.
- repro `sectioning-frontmatter/z14r3_address_glued_to_a_surname_red.tex`.
- repros `plain_address_glued_by_a_dash_red.tex`, `plain_address_glued_by_a_comma_red.tex`.
- repro `z14r3_same_line_text_under_two_marks_red.tex`, with the OPEN `entry_address_owners` row (2403.12766).
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had thei".

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

- **Z3: spans from the digested list, the rehearsal's isolation (int143b; OD #978) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED `writer_second_digestion_says_an_error_twice_red`;
- `#[ignore]`d REDs of `perfect_kernel_batchz3`: `affiliation_cut_environments_by_spelling_red`, `affiliation_cut_group_siblings_stay_uncut_red`, `rehearsal_no_lua_loses_a_cut_red`.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z3: spans from the digested list, the rehearsal's isolation ".

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

- **A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, as P1)** — `latexml_engine/src/l3fp_native/`: `\__fp_parse_operand:Nw` at the doors' shape answers with what its run leaves, from a port of l3fp's arithmetic, or runs as defined (a flag, a tuple, `nan`, `rand`, a conditional in the argument, a tab mark that ends a cell, any l3fp name changed since the install, an expression past 400 levels);
- State: [`performance/PERFORMANCE.md`](../performance/PERFORMANCE.md)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, ".

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

- **R1: the title-page check's cost, the last regression of #151 (user 2026-11-06; OD #953 "R1"; `performance/PERFORMANCE.md` "the pk/P1 levers")** — DONE: no run of the check's reads more than 8 M tokens (`gullet::read_budget`), what it leaves out named in a counted `Warning:frontmatter:check_budget`;
- RED `check_store_argument_loop_times_out_red` green (`perfect_kernel_batchr1::a_looping_stored_argument_is_budgeted`).
- SASnRdisplay's `Warning:unexpected:\end{document}`, a `\ifx` left open since between #149 and P1, not root-caused
- State: [`performance/PERFORMANCE.md`](../performance/PERFORMANCE.md)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "R1: the title-page check's cost, the last regression of #151".

### A second `\author` in article keeps the first call's contact

- **A second `\author` in article keeps the first call's contact** (Z2's review) — `\author{Ann$^1$ \\ $^1$Univ A\thanks{First note.}}\author{Bob}`: the PDF has Bob alone;
- ours Bob with the affiliation Univ A and the pubnote First note, on the base too.
- Synthetic;
- no arXiv witness.
- RED `sectioning-frontmatter/article_second_author_call_keeps_a_block_of_the_first_red`, no guard | RED | Before the frontmatter's single engine reads it (plan Z-series) or on a witness
- State: `tools/perfect_kernel/repros/sectioning-frontmatter/`

### Z19 list setters (DEL3, OD #982)

- **Z19 list setters (DEL3, OD #982)** — the S2b writer's last `=2` path closed: a block calling a binding's list setter (`\institute` of sv_support, inst_support, aa, llncs inside `\author`) is the emitter's — the setter's call written as it is after the items, its entries the list the names' marks request (`emitter/emit.rs` `list_setter_runs`, `without_list_setters`);
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z19 list setters (DEL3, OD #982)".

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

- **100b: one Kind for the addresses inside affiliation entries, one Link chain (OD #956; int129a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): HARD, deferred: an address inside a line of other words (about 151 dev20k papers);
- 2009.04283 pathological (DEFERRED_FAMILIES);
- TENTATIVE ruling E and the only-carrier step (2403.17706, 2411.17073);
- RED an `@` in `\href`'s URL read as a printed font;
- REDs `address_family_text_symbol_marked_lines_pile_on_last` (cmp-lg9710008), `address_family_rlap_name_split_takes_spelled_address` (hep-ph9905377), `address_family_unspaced_list_one_letter_last_stays_whole`, a tie after a list comma read as `˜`;
- int129a's: an authblk entry of initials-labelled address lines takes out only its first (2505.20974).
- Its witnesses: the HARD shape 0704.2085, 1902.04228, 2111.01631, 2608.22602, 1805.00710 (`ee13d015, ramkrishna@…`);
- the `@` in `\href`'s URL `address_family_href_url_at_is_no_printed_font`;
- the tie `address_family_tie_in_email_list_prints_a_space`.
- RESIDUALs: names welded into an entry by an upstream misparse (2506.06908's `Jiawei Yan, …` under Chenzhi Xu; 2504.14657, jmlr's `\AND`: `bellabruin5711@` the block's);
- a one-address line with an ORCID glued after it is declined whole (2601.09524, 2308.02269 `inenaga@`, `nakashima.yuto.003@`);
- (S2) the own-call step runs in today's engine only (jmlr's `\AND` 1504.01446, 2509.23923);
- (N1) addresses left to the block whose author is identifiable (1912.12193 `xi@`, astro-ph9809079 ö read as oe, 2308.02269's `arim@`/`koba@`, 1410.0127's `bcz@`);
- 2501.13021's `author1@` after the misparsed creator's own two;
- 1611.10059 (elsarticle) loses typewriter on the address the `\ead` copy merges with.
- Also: the HARD shape's fail-safe witness 2401.08092 (the writer declines a re-part needing a block creator);
- (S2) compare 2002.00057;
- not read (ruled out): surname-first initials (2512.18755).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100b: one Kind for the addresses inside affiliation entries,".

### Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs 368 sandbox-arxiv-2605, 369 sandbox-arxiv-2606)

- **Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs 368 sandbox-arxiv-2605, 369 sandbox-arxiv-2606)** — error-free coverage did not regress: 2605 95.47 → 96.88 % (E 1,332 → 923, F 29 → 16), 2606 95.64 → 96.98 % (E 1,298 → 897, F 27 → 22), baselines runs 354/355 (2026-10-15, `1a95291ef6`) snapshotted first.
- State: OPEN (triage)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs".

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

- **103b residual: a collaboration heading over an `\and` list covers only the first name** — `\author{GRAVITY+ Collaboration\thanks{…}: R.~Abuter\inst{8} \and …}` (aa, 2206.00684, the 98a witness), `Euclid Collaboration: M.~Walmsley\orcid{…}\thanks{…}\inst{…} \and …` (aa 2503.15310, aaEC 2507.12116): the binding split reads the heading as the remark of the first piece (`split_author_calls`, OD #946), so only the first name carries `contact role="collaboration"` (2206.00684: 1 of 115);
- nothing is lost (the heading's `\thanks` a pubnote).
- Charter §4: the collaboration is a contact on every name it heads.
- Minimal input: llncs `\author{XYZ Collaboration: Ann Able\inst{1} \and Bob Baker\inst{2} \and Cat Cole\inst{1}}` — RED repro class-bindings/`llncs_collaboration_heading_covers_every_name` (no guard).
- Options: carry a heading parted by its colon from the first name to every later `\and` piece of the same call (in scope of the split, a brief patch), or leave the first name's only.
- State: OPEN (review rev103b S3, record only)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "103b residual: a collaboration heading over an `\and` list c".

### 103c: a legend line read as an affiliation is its carriers' note (census129 cluster 6 / mechanism 4, Bundle 1c; OD #961)

- **103c: a legend line read as an affiliation is its carriers' note (census129 cluster 6 / mechanism 4, Bundle 1c; OD #961)** — FIXED: at the frontmatter flush, after placement, the Link step and the observer's writing (`insert::kind_legend_contacts`), a creator's affiliation contact whose whole text is legends — correspondence, a contribution, a death — names and addresses, no institution (`notes::entry_legend_kind`: pieces parted at marks, brackets, `,:;` and line breaks), is renamed in place to `<note role="thanks" class="ltx_note_frontmatter ltx_thanks_<kind>">`, carriers and e-mails unchanged, its `mark` one that prints — the printed superscript, else a label key a printed mark gave (`affiliation:K`, not an entry's place: `_seqlabel`), a carrier's name shows or requests, or its binding prints before the line (`_labelprints`: authblk, jheppub/jcappub, cas), the carrier's own among several;
- Residuals (one-line inputs): IEEEtran's `\IEEEauthorblockA{\IEEEauthorrefmark{1}Corresponding author}` was a note marked "1", the number the names showed, where IEEEtran prints `*` in both (2205.00125, 2410.12184 `§`, 2504.18719, 2507.23474, 2603.14786 `‡`) — fixed by 106b (OD #968: the binding prints the symbol, the note's mark is it);
- `$^{\ddagger}$Work done while at Google` on the line route keeps `ltx_thanks_note`, the CSS kind reader reading other text than `legend_note_kind` (review rev103c2 N1, class only);
- `\affil[*]{The authors wish it to be known that, in their opinion, the first two authors should be regarded as Joint First Authors.}` stays an affiliation (a comma parts the piece; 1703.07724);
- `\icmlaffiliation{x}{Work done before joining Google DeepMind}` (2603.23994), `\affil[2]{Corresponding Author: amir[dot]norouzi[at]lancium[dot]com}` (2605.00769), a legend beside a code link (2403.03314), a funding legend (1604.05657) stay affiliations;
- `\author[1]{A}\author[1]{B}\affil[1]{Univ}\affil[ ]{Corresponding author: A (a@x)}` stays B's affiliation, the run's last author (2412.14916; unmarked, and the Link question, not the Kind);
- a compound legend `$^{\dagger\ddagger}$Project lead` the observer writes gets `mark="†"` (`show_legend_mark`, 77p's shape), while the Kind step keeps a label's key whole (2602.19190's `$^{*\dag}$Project lead: Yi Yang`, `*†`);
- `\thanks{Work done during an internship at Microsoft Research Asia.}` is `ltx_thanks_note` (2110.15064, 2410.24211).
- RED, synthetic: an authblk label that prints nothing visible (`\affil[~]`, `\affil[\phantom{*}]`) is written into a note's mark as its source text (`mark="˜"`) — `_labelprints` should read the label's printed text (repro sectioning-frontmatter/`legend_kind_invisible_authblk_label_prints_no_mark`; none of the A/B papers).
- Pre-existing, pinned as base: in the svjour guard's repro (`legend_kind_svjour_institute_sequence_stays`) the `\institute{… \and C. Cole (\email{…}) \at Org Inc.}` name "C. Cole" appears nowhere in the XML (pdftotext prints it) — a content loss of svjour's `\at` institute names, base too.
- Open for the user: IEEE legend notes take the number the names show ("1") where IEEEtran prints ∗/§/‡ (2205.00125, 2410.12184) — the binding's `\IEEEauthorrefmark` form, not this step.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "103c: a legend line read as an affiliation is its carriers' ".

### 104b: a raw class's `\and` institute store (OD #791, #955) — the residual

- **104b: a raw class's `\and` institute store (OD #791, #955) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): OPEN, RED catalog repros, one paper each: comsis2 2207.09249, caosp 1401.6818 (`displaced_numbered_institutes_also_printed_after_the_bibliography`), a numbered store whose last entry carries the names' legend lines.
- RED `raw_class_numbered_institutes_with_legend_lines` (2207.09249).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104b: a raw class's `\and` institute store (OD #791, #955) —".

### int131fix: the Bundle 1 A/B's regressions — the residual

- **int131fix: the Bundle 1 A/B's regressions — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: a recovered Fatal in a field leaves `place_keeper` (`\date{\deep}`), 2110.07418;
- RED `author_orcid_link_wrapped_comma_parts_no_names`.
- State: OPEN (residual)

### Token-stack RAII (int131fix follow-up)

- **Token-stack RAII (int131fix follow-up)** — `invoke_token` (stomach.rs) pushes `token_stack` and pops it on its normal exits only;
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Token-stack RAII (int131fix follow-up)".

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

- **104c: marked runs the Cut left whole (OD #964) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED `revtex_nucleus_marks_spaced_in_one_line_red`;
- open (rev104c3): a nucleus mark after "and", an isotope inside a line;
- deferred: an unmarked line before the marked ones (2205.14113).
- Also open, readings before 104c (recorded, no guard): a tabular of marked entries then `\\ Joint Lab` gives "Joint Lab" to the last entry only;
- REDs: `revtex_isotope_mark_after_and_is_text_red`, `revtex_math_number_mark_before_st_floor_red` (catalog only), `svjour3_institute_unmarked_lead_lines_red` (2205.14113), `revtex_unmarked_first_affiliation_line_red` (1903.07834), `aa_institute_and_inside_a_macro_group_red` (2407.09799).
- `$^1$ Univ A, Lab for ${}^{3}$He Physics` parted at the isotope;
- `$3$\textsuperscript{rd} floor` loses "rd".
- Outside the Cut's reading: llncs's `\mailsa` line glued to the last entry (1604.03221), a second author block (2006.15775), ieeeconf's `\:\:$^{2}$` (2110.11491), authblk's `\affil[\hspace{-1ex}]` label (2502.06575), a legend on an entry (2609.38081).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104c: marked runs the Cut left whole (OD #964) — the residua".

### 104d: link lists — the residual

- **104d: link lists — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): residual 2008.13733 (a correspondent of several names), L3's other witnesses (2607.00453);
- RED L4, a `|` between addresses (1507.04180), L5, ijcai `\emails` with a TeX group (`ijcai_emails_group_list_is_spelled`, 2403.12370), P1, aastex631's `\correspondingauthor` placement;
- OPEN P2, cas's `{augroup}` undefined.
- P1's RED `aastex631_correspondingauthor_after_authors_email_is_the_named_authors`.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104d: link lists — the residual".

### 105a: the net's lines below its floor (OD #953) — the residual

- **105a: the net's lines below its floor (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): RED: class-default placeholders no candidates (2302.11911, 1509.08973; the placeholder ruling);
- code the net does not walk (cccconf 2408.14758, 2206.02389, 2512.09276, 1702.07637 deferred; basm 1601.01922).
- REDs `net_class_default_placeholder_is_counted_red`, `net_non_english_month_line_red`;
- revtex's `\@author@finish` under the revtex binding prints `\alt@affil@toks` (aeb_emulateapj 1410.3797, openjournal 2604.08196).
- Each of (b)-(h) needs its own mechanism or binding: (b) a store whose name is computed or passed (cup-journal 2508.08363, 2607.18758, 2412.08333, 2503.22083, 2510.05654, 2510.18398; ptapap 1912.07241, 1612.02970; compositionality 1904.09322, 2603.03774; JUSTC 2409.17603; opendatalab 2601.09733, 2601.21821, 2605.10341);
- (d) a candidate the sentinel run does not print (2205.05046, 2412.08184, 2205.01391);
- (e) memsait's handed `\@opening` (astro-ph0102098, astro-ph9909366, astro-ph0005436);
- (f) printed only from the `.aux` (1206.2980, 2310.08896);
- (g) a K11 or kernel field losing part of its value (2304.08300, 2312.02510, 1909.06690, cond-mat9610136, cond-mat9812194, cond-mat9906132, 2004.13105, 2408.12585, 2410.15356);
- (h) outside raw-class stores (`\offprints{…\email{…}}` 1511.07638, 0805.4214, 1311.4988; econsocart 2203.00070; icrc29 astro-ph0507347; 2308.16621; single words of kept fields 1709.00722, 2409.01067, 2205.11450, 1908.07849, astro-ph9912176);
- a Chinese line whose ideographs all appear in the title or abstract goes with no Warning (rev105a2 nit, suspected).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "105a: the net's lines below its floor (OD #953) — the residu".

### 106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade lines — the residual

- **106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade lines — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): a late revtex `\author{A}\affiliation{X}` after `\maketitle`;
- Also open (one-line inputs): amsart `\author{Name\\ \today}` dateline as an affiliation (1910.12289);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade line".

### 106a: an address line kept as affiliation text (OD #966) — the residual

- **106a: an address line kept as affiliation text (OD #966) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: 2410.09681's template line, a line break inside a font wrapper (2311.04121), a phrase-led `\altaffiliation`;
- RED (rev106a2) the older route places by order.
- REDs: `addr_line_authblk_two_carriers_line_marks_red` (the older route), `addr_line_authblk_thanks_requested_mark_red` (a `\thanks`-requested mark unread by `author_mark_keys`);
- left: an address inside a legend note (2312.10429, 2501.04244).
- Also open: a phrase-led `\altaffiliation{Correspondence should be sent to a, b and c.}` (1209.3452);
- tagged runs `a@x (Matvey)` (2512.10975);
- `{a@mails., b@}tsinghua.edu.cn` (2602.07769);
- an affiliation the observer's writer makes after the flush (mn2e, 2008.08137);
- a template `{name.surname}@ 1polytechnique.edu` (2009.04283);
- e-mail contacts holding lists or names (2004.13933, 2005.04027, 2510.11067, 1512.01552, 2404.09958);
- class contacts other than affiliations (elsarticle 2310.13481, a `\thanks` 2509.07695, amsart `\address` glued to `\email` 2301.00107);
- unspelled addresses to the block (2608.22602's `chris241`, `chand158`);
- recorded: 1310.7913's unmarked `Corresponding Authors: a, b` names the e-mails rather than becoming a note (#961 needs a printed mark).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106a: an address line kept as affiliation text (OD #966) — t".

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

- **106t: the title-page check's second source (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: quantumarticle's `\ead` (KPE #1208), ceurart's `\orcidauthor`;
- RED (rev106t2 S1) a binding selector's undeclared dropped branch;
- That RED: `check_binding_selector_untaken_branch_unwanted` (fancyhdr's `\iffloatpage`).
- dmtcs `\publicationdata`'s dates counted, the content fix open;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106t: the title-page check's second source (OD #953) — the r".

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
- Also open (one line each): a template line `Emails: name.surname@polito.it, …` (2401.15236);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107c: an address, legend or URL line kinded affiliation (OD ".

### 107d: the title-page check where no `\maketitle` of ours runs (OD #953) — the residual

- **107d: the title-page check where no `\maketitle` of ours runs (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `check_title_page_relaxed_maketitle_no_false_warning_red` (S1), a sibling-directory `\input` read as a package's (S2).
- Open, one dev3 paper: nj_style's `\ead` written to the `.aux`, printed on pdflatex's second run (1401.7920).
- State: OPEN (residual)

### 107e: a marked run cut whatever parts it (OD #954) — the residual

- **107e: a marked run cut whatever parts it (OD #954) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: 2605.25195's legend (the writer declines its URL line's owner);
- REDs `cut_run_wrapped_entry_then_comma_address_red`, `cut_run_group_entry_then_comma_address_red`.
- Also `cut_run_legend_after_wrapped_entry_ending_in_a_group_red`, `cut_run_nested_wrapper_per_entry_red`.
- Also open: the kernel alone welds a legend line into the entry above it after any run (base the same);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107e: a marked run cut whatever parts it (OD #954) — the res".

### 107f: e-mail ownership (OD #970, #972, #966) — the residual

- **107f: e-mail ownership (OD #970, #972, #966) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): open: 2511.18493's `minhnnhat@` goes to Anh-Minh Phan;
- RED `addr_line_sole_author_symbol_series_orphan_is_blocks_red` (`mark_series`' symbol branch does not fire).
- Also open: a note holding a list for its carriers is not parted;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107f: e-mail ownership (OD #970, #972, #966) — the residual".

### 107g: the Bundle 5 fix round (OD #970, #966, #897) — the residual

- **107g: the Bundle 5 fix round (OD #970, #966, #897) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): REDs `email_list_mark_line_names_non_carriers_no_spelling_red`, `email_list_mark_line_spelled_out_of_place_no_order_red`, `email_own_call_printed_under_both_stays_on_both_red` (2502.15891, 2307.00704, 2405.01799), `author_block_repeated_affiliations_land_on_the_last_red` (2307.07324, 2001.07542).
- Also `author_tie_space_inside_three_word_name_red`, `email_own_call_in_an_entry_tie_moves_red`, `addr_line_glued_word_before_a_font_observer_route_red`.
- Open: 2010.09850's two institutes in one `\parbox` are one affiliation as on base (the shape of 2509.02924, no rule tells them apart);
- 2604.19098's `\parbox` of address lines stays in the affiliation as on base (the option: `@` as a third reason a wrapper is transparent);
- RED `email_own_call_last_author_every_author_addressed_stays_red` and its twin `email_own_call_last_author_every_author_addressed_stays_twin_red` (`last` too broad; 0 rows in 3 dev runs);
- not blocking: 1103.0889 `Faranda, Davide` mis-split, `v.lucarini@` on "Davide";
- out of scope, unchanged: 1406.2912's legend tail on the Hanoi affiliation, 0807.2167's `$^1$NIKHEF` welded into a name, 2601.12698's dropped mark.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107g: the Bundle 5 fix round (OD #970, #966, #897) — the res".

### 108a: authors merged under authblk (2610.06709; OD #52(h)) — the residual

- **108a: authors merged under authblk (2610.06709; OD #52(h)) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: `\footnote` under a sole authblk author (2601.20363, 2605.13069; RED `authblk_footnote_line_under_sole_author_red`);
- 2505.05785 (RED `email_spelled_part_of_a_hyphen_name_red`);
- REDs `authblk_institution_line_before_affil_red`, `authblk_legend_swallows_contact_lines_red`, `authblk_title_suffix_one_name_red`, `authblk_small_hspace_in_name_red`.
- 2606.04547, 2412.13509;
- RED `authblk_link_macro_under_names_stays_merged_uncounted_red` (2601.07891);
- the trade, recorded: "as many left, in order" requires the order to hold, so out-of-order lists go to the block (2606.04646, 2605.26032, 2102.08716);
- int140a's open: 2511.07187 (`Name -- \url{…}\\ …`: 3 of 4 authors read as affiliation lines, no self-check fires).
- Also int140a's open: cpc-hepnp's `1;1)` could part into its affiliation and its note instead of counting a lost mark (#431);
- an e-mail keeps a glued mark in its text, `cak29@mail.aub.edu *` (2609.25833), `*archnalakhani@` (1201.5255).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "108a: authors merged under authblk (2610.06709; OD #52(h)) —".

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

- **97a review should-fixes (rev97a, SHIP, 2026-10-31)** — (S1, synthetic; FIXED 97b, OD #944: a piece that is an address and no name is no marked name, GREEN `perfect_kernel_batch97b::marked_address_piece_is_no_person`) a marked e-mail piece is made a person: `names_split.rs` `marked_names` counts any piece `reads_as_affiliation` declines as a marked name, and `frontmatter/classify/marks.rs:8-11` declines every text holding `@`, so `\author{A.~Able$^{1}$, B.~in der Baker$^{2}$, b.baker@two.edu$^{*}$}` (revtex4-2) gives three creators, the third `<personname>b.baker@two.edu<sup>*</sup></personname>` (95b r3: one creator);
- the same for a marked URL, ORCID or place name the shape test reads as a name;
- fix: a piece the e-mail reader would take is no name, the line then a reported merge;
- no census paper's changed creators hold an `@` (RED `author_marked_email_piece_is_no_person`);
- (N1) the merge message "One author holds N names" is a lower bound read as exact (`name_text.rs:549-554` `names.max(_mergednames)`: 2508.04092 says 2 of 4, 1012.4815 3 of 4, 1212.4143 6 of 7) — word it "at least N" or take the larger of the marked and comma-piece counts;
- (P2, older) 2312.16289's e-mail contact carries raw TeX in its label, `name="${}^{\ast}$: Corresponding author: "` (int123a the same)
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97a review should-fixes (rev97a, SHIP, 2026-10-31)".

### 97b: a take-out removes what it read and nothing else TeX prints

- **97b: a take-out removes what it read and nothing else TeX prints** (OD #944; the int125b A/B's `fm_checks.py` losses against int123a) — FIXED: aa's `\mail` labelled as the shipped copy prints it (a copy naming `\mailname`, v4.03-v7.0, keeps "Correspondence to "; a storing copy, v8 on, none; `\mailname`'s `\it` grouped, no italic on later items);
- Residuals, each a one-line input: (a) a shape-split names line keeps the binding's `before` — `\documentclass{revtex4}…\author{Ann Able, Bob Baker and Cat Cole}` before="  " (int123a the same; "and" printed nowhere);
- (b) two prints of one address with two labels keep one — `\institute{Univ X \\ email: x@y.org}\mail{x@y.org}` under a printing aa.cls gives one contact "Correspondence to ", the institute's "email:" nowhere (95c's first-print order);
- (c) 2508.08560: a raw package's `\affiliation` store whose ★⧫♠♣ e-mail legend follows two `\vspace{0.3cm} $ $ \\` lines is one affiliation of the last name, its marks stripped — the store-affiliation line reading, not a take-out (RED `store_affiliation_email_legend_after_spacing_lines`);
- (e) a label part the deposit takes out of the first place its tokens stand, not the branch that runs: jpsj2.cls:789-818 prints `KEYWORDS:\hskip.5em\parbox[t]{…}{\@kword}` in five option branches, so the printed "KEYWORDS:" stays a body paragraph beside the keywords' name (int125b the same; 416 papers of the bundle population, 0704.0082);
- (d) the 95d r3 RED rows `raw_class_date_branch_with_trailing_text_keeps_it`, `raw_class_label_with_a_comma_stays_whole` stay RED (other mechanisms: the whole-test removal, the clause split)
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97b: a take-out removes what it read and nothing else TeX pr".

### 97b review should-fixes (rev97b, SHIP, 2026-11-01)

- **97b review should-fixes (rev97b, SHIP, 2026-11-01)** — (S1, synthetic) the store-label reader knows which arguments print only for the commands its table names (`frontmatter_stores.rs` `argument_of`, `PRINTED_ARGUMENTS`), so a printed label is still lost through a raw file's own wrapper (`\newcommand{\mymailto}[1]{\href{mailto:#1}{#1}}`, `Correspondence: \mymailto{\@corremail}`), `\hyperlink{corr}{\@corremail}` or `{\color{black}\@corremail}` (int125b the same);
- parameter types cannot decide it (`\url` and `\href`'s target both Semiverbatim);
- fix: read a raw-file macro with parameters through its body, `\hyperlink`'s 2nd and `\hyperref[…]`'s argument print, a `\color` inside the store's group is layout (RED `raw_package_note_label_before_a_wrapper_macro_names_it`);
- (S2, synthetic) rev97a S1's siblings: a marked address piece led by a label word (`E-mail: b.baker@two.edu$^{*}$`, 1504.06050's line shape) or a marked URL (`www.two.edu/~baker$^{*}$`) still reads as a marked name (`names_split.rs:173-179` `only_addresses`; int125b the same; RED `author_marked_labelled_address_piece_is_no_person`);
- fix: skip a leading label word ending in `:`, count URL-shaped words as addresses;
- (N2) the `in_unprinted_argument` doc comment still claims a `\write`'s text, but `argument_of` looks only at the token before the group (`\write\@auxout{…}`, `\marks3{…}` never match) — OD #944 corrected, the comment waits on the next writer in that file;
- (N3) the guard name `aa_mail_storing_copy_has_no_label` asserts the kernel default `name="Email: "` on an address pdflatex prints nowhere;
- (N4) `argument_of` skips one `[…]` only (`\parbox[t][3cm][s]{w}{\@x}`);
- (P1, older) HTML shows a note's role name before its text (`<span class="ltx_note_type">email: </span>`, LaTeXML-meta-xhtml.xsl:77-83, bold in CSS): 2506.10896 renders "email: Correspondence: …" — a role name the class never prints (FIDELITY_CHARTER §4);
- (P2, older) printed words around a store still go with it without a Warning ("Write to the author", a trailing "(preferred)", a leading "Mail"; the conservation rule of brief 97b part B);
- (P3, older) a given-back `\mail` address goes to the last creator, not the one it spells (1005.3784 Woodcraft; astro-ph0107243 `faraggiana@`; int123a the same);
- (P4, older) 1504.06050's per-address marks a/a/b collapse to one `a,b` on a nameless creator (appolb, no TL oracle)
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97b review should-fixes (rev97b, SHIP, 2026-11-01)".

### 97c: an affiliation the reading already has reaches its author

- **97c: an affiliation the reading already has reaches its author** (OD #945; dev20k's largest archival shape `own:aff_missing`, 364 creators / 119 papers) — FIXED: (W1) a names block's mark-led entry of several events is cut by its whole span (resolver/blocks.rs `labelled_entries`; `$^{1}$Univ of Science and Technology~~~~$^{3}$Univ C` was Ann's whole: 2111.07950, 1503.06795, 2504.05692, 2008.02300, 2404.01856; W1b 2212.01433, 2608.06741), a digit mark glued inside a word only when the source glues it (events/lexer.rs `glued_in_source`: `NLP$^2$CT` the word's, `Russia\break $^2$Nuclear` an entry, hep-ph9809222);
- Residuals, each a one-line input: (a) two group-wrapped entry lines do not cut — `\author{Ann Able$^{1}$ \quad Bob Baker$^{2}$ \quad Cy Cole$^{3}$ \\ \textnormal{$^{1}$Univ A \quad $^{2}$Univ B} \\ \textnormal{$^{3}$Univ C}}` gives Ann "Univ A 2Univ B" (2609.38081; 97b the same);
- (b) llncs legend line first — `\institute{\textsuperscript{*}Equal contribution \\ Univ A, Town}` continues the legend, Ann "Equal contributionUniv A, Town", Bob none;
- (c) svproc/svjour sole institute with an `\email` — `\author{Ann Able \and Bob Baker}\institute{Univ A, Town A\\ \email{ann@x.org}}` the block's, address included (1705.10638, 2010.10850, 2207.12084, 2307.06258);
- under numbered institutes hand-typed `${}^{1}$` marks answer no piece today (2608.25509: affiliations the block's, three of five addresses off their authors; the reading is right, sv_support is no written family);
- (e) the scorer folds an llncs legend into the institute (SL-legend): 2603.18089, 2608.22772, 2207.13038 score `aff_unexplained` though the XML matches pdflatex  Review rev97cd (NO-SHIP on 97c alone): C1 (svproc addresses on every author of their institute) and C2 (2608.25509) are fixed by 97e's sv_support writer — on 97c + 97e none of the seven C1 papers shares an address, each gains its affiliations, and 2608.25509 holds every affiliation and the two spelled addresses on their authors, its three initials-only addresses (`ngr@`, `ygt@`, `frt@`) at the block (base: by position; initials as evidence is the refactor's placement order) — so SV1 lands only with 97e (OD #945 amended);
- C3 synthetic, RED `author_digit_mark_glued_after_an_entry_opens_the_next` (the in-word digit glue takes a requested label);
- C4 `_markpiece` is binding data.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97c: an affiliation the reading already has reaches its auth".

### 97d: a collaboration named in the author line is a contact (OD #946) — the residual

- **97d: a collaboration named in the author line is a contact (OD #946) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left, one-line inputs: (a) aa's own `\and` split;
- (g) RED `author_collaboration_name_with_and_stays_one`;
- (j)'s RED `author_team_piece_mark_labels_its_own_line`.
- Also left: (a) 2512.08803 (`XYZ Collaboration: A.~Able\inst{1}` one name);
- (h) `\affiliation{(WA98 Collaboration)}` an affiliation (0708.2630), awaiting a ruling;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97d: a collaboration named in the author line is a contact (".

### 97f: giant lists in a size environment; AASTeX 6.x's shared affiliations — the residual

- **97f: giant lists in a size environment; AASTeX 6.x's shared affiliations — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md)): left: (a) sub-labelled marks (RED `author_sub_labelled_marks_read_as_digits`, ruling deferred);
- RED `aa_collaboration_heading_before_colon_is_a_name`;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97f: giant lists in a size environment; AASTeX 6.x's shared ".

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
- Also left: an affiliation-only `\IEEEauthorblockA{X, Y\\ …}` parted at its first line's commas (words kept);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "98b: four lost-words mechanisms (OD #949-#952) — the residua".

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

- **97e r2 residuals (review rev97e; OD #948)** — (N1) 1108.2557 (ws-ijmpd, inst_support): the e-mail line printed under the addresses is read into entry b, so all three addresses go to Geng (97c: on no one) — the reading's line cut;
- (N2) the paper-wide corresponding-author fallback is gone where an e-mail call's group is empty (`resolver/post.rs` `answer_sequence`, 97e r1's scoping): the address falls to the block, fail-safe;
- `events::lexer::letter_spaced_word_space` reads `Smith A B\enspace Jones C` (two names, the last word one letter) as one name — no witness yet;
- (key) a hand-typed mark equal to an elsarticle key whose place prints otherwise, `\author[a]{Cy Cole$^{*}$}` under `\address[*]{…}`: today's and the reading give Cy Cole the line, pdflatex prints "c current address" — a hand mark is read against the key, not the place letter the class prints (`\theaffn`; keys in order agree, 0907.5014), RED note in repro class-bindings/`elsarticle_key_labelled_address_is_no_mark`;
- (note) a `\footnotetext[n]` after `\maketitle` the reading pairs with a name's `\footnotemark[n]` is a note today's holds only after the writer runs (its relocation followed the S2b writer, deleted in DEL4), so the paper declines whole — pre-existing in every family (`article` with `\renewcommand\thefootnote{\fnsymbol{footnote}}`), reached in elsarticle by its arabic marks since r2: 1108.0201, 0909.2782 (r1 had split its e-mails onto their authors);
- fix: the writer leaves a text built after the names' flush to today's relocation (neither wanted nor compared), then a whole-population A/B;
- (keys) the other bindings labelling an entry by a key (`label={#1}`: imsart, ecai, ceurart, icml, sn-jnl, bmcart, beilstein, interspeech, vmsta, quantumarticle; `UNDECLARED`, their own `\author`) are not audited for printed keys — a class printing the entry's place, not the key, declares `keyed_labels` as elsart_support_core does;
- (pre-existing) the fms2a2 `addr` source (`1. University A … 2. University B`) gives every author every line, against FIDELITY_CHARTER §4 R1/R2;
- marks left in the names of 1201.0008, 2307.11481, 2506.22621
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97e r2 residuals (review rev97e; OD #948)".

### int126a review (rev126a; OD #947, #948)

- **int126a review (rev126a; OD #947, #948)** — B1 and B3 FIXED in 98a (a collaboration heading a printed separator parts from the first name is an entry, 2206.00684; a control word in an `\altaffilmark` list is no request label, astro-ph0105133; guards `perfect_kernel_batch98a`).
- Open for a ruling: the heading `XYZ Collaboration\thanks{…}: Ann Able` is written as a creator of its own (Perl's and the old reader's `\and` entry, 97b's output), not OD #946's collaboration contact — svjour's, llncs's and aa's `\and` pieces never reach the author block's collaboration reader (row 97d (a));
- Pre-existing, recorded only, each unchanged int126a → 98a: (a) 1708.04166 (PoS): `serena.benatti@…` held by 5 Padova authors, not the one it spells;
- (b) 1909.05984: G. Vianello holds the collaboration contact twice;
- (c) marks left in names — hep-ph0301204 "1 , 2 G. Ricco" (the comma split, row 97f (b)), 0907.5014 "Denes Molnar a,b";
- (d) 1212.4143 still one creator.
- elsart's `\author{XYZ Collaboration: Ann Able}` already gives the contact.
- Noted, no output change: `\altaffilmark{1,~2}` requests a stray `altaffil:˜` beside `altaffil:1`/`altaffil:2` (the observer's raw text spells the tie `˜`), answered by nothing.
- Residuals, recorded (no fix in 98e): 2506.19040 `drohr@mit.edu` copied to the other two `*` authors beside their own addresses (suspected `frontmatter/observer/resolver/owners.rs:666` `spelled_owners` reading "drohr" as spelling no one);
- BESIII sub-lettered marks (`75A,75C`, `29A`, `63A,63C`) keep the mark in the name, the name given the whole numbered entry, not its lettered part (33-36 names a paper; R6, tentative: sub-entries at Stage 4a);
- 1609.03822's `\pounds` label is not split;
- nameless creators holding a stray copy: 2307.16667 [5], 2312.09590 [3], 2109.02295 [4];
- `&&`/`\nonumber` affiliation lines (eqnarray residue; pdflatex errs) keep the block one entry (2208.09667, 2308.02280: pathological — defer);
- numbered lines set in a `\parbox` inside one `\affiliation` land once at the block (review rev98e fs3: pathological — defer; a one-column `tabular`'s rows are each author's since 104c, 0911.3234, OD #964);
- 2112.13115's second department keeps its line's printed `and` at its head ("and Department of Physics …", the marked list keeps a line's text before its mark);
- the observer reads no Mark event from a nucleus formula (`${\,\!}^1$`, `\mbox{${\ }^{1}$}`: 98e r1's `_unsplit` cut never fired on them; the binding's token reader now covers them, the observer's `observer::marks` does not — a second engine's reading of the same spelling, open);
- the kernel `\author` block's unrequested line keeps no printed label (`\author{Ann Able$^{1}$, …\\ $^{1}$Alpha …\\ $^{b}$Also at Eps College}` → a nameless creator "Also at Eps College", 98a the same: the `\lx@affiliation@withsup` pending creator relocate_annotations leaves unanswered, not the writer's cut — review rev98e's sp2)
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "int126a review (rev126a; OD #947, #948)".

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

- **Unobserved frontmatter calls under a written author block (95b r3 residual, OD #942)** — the writer tells an unwritten family's call apart only when a watched setter's expansion made it (`ledger::FRONTMATTER_SETTERS`);
- any other API call under a written block is written as before, whoever made it: a binding's other frontmatter commands (`\thanks`, `\IEEEauthorblockA`, elsarticle's `\cortext`, revtex's `\altaffiliation`), the document's own `\lx@add@…`, and a raw file's store handed on at `\maketitle` (`frontmatter_stores::harvest_stores`, where no setter's expansion is open — a raw setter is the class's family anyway, so only a raw style's store under an unwritten class family matters).
- One-line minimal input of the class, no census paper showing it: `\documentclass{svjour3}\usepackage{authblk}` with a raw style's `\def\address#1{\gdef\@address{#1}}` printed by the class.
- Census: no paper of the 95b populations or the 419 authblk papers under other classes changed but 2609.05466.
- Options: (a) the harvest names the raw file for every hand-on (`\lx@store@from`, frontmatter_stores.rs);
- (b) watch the bindings' other contact commands by name;
- (c) leave
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Unobserved frontmatter calls under a written author block (9".

### svjour3 `\institute`: the names before `\at` are no entry (pre-existing; 95b r3)

- **svjour3 `\institute`: the names before `\at` are no entry (pre-existing; 95b r3)** — svjour3.cls:506 `\def\at{\\\let\and=\iand}` prints the names before `\at` over the institute they share;
- State: OPEN (the reading; the writer declines)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "svjour3 `\institute`: the names before `\at` are no entry (p".

### 95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none a regression or a loss)

- **95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none a regression or a loss)** — (S1, moot since DEL5 deleted the setter watch) a setter's input mark can stay open after its expansion ends: `state.rs` `input_unread_since` (~:2436-2449) compares the mouth's depth and read position but not which mouth, and token-list digestion runs on `Mouth::default()` (position (0,0,0,0), mouth.rs:164-175), so a mark taken at the end of a digested list reads open in a later default-mouth context;
- (S2) the watch switches on in any written-family paper that defines one of the setter names itself — the `document`/`fallback` owner counts as an unwritten family (`ledger.rs` `watch_unwritten_setters` ~:494-505): article + authblk + `\newcommand{\email}[1]{\texttt{#1}}` (review probe `rev95b3/q1`, control `q2`) costs +0.28 % instructions and marks earlier calls `unobserved`;
- creators unchanged;
- fix: exclude non-binding owners from the trigger, or correct the comment and OD #942.
- (S3) svjour3's own `\mail` (sv_support_sty.rs:147-150 → `\lx@add@contact[role=address]`) is not in `ledger::FRONTMATTER_SETTERS` (~:282), so under authblk its item is still written;
- the list hand-copies `store_setters.rs:22` / `rehearsal.rs:271-281` — derive it from those tables.
- (S4) the row above's minimal input is unchecked (svjour3 never prints a raw `\@address`) and its scope differs from OD #942's wording: one statement in both and a pdflatex-clean RED repro.
- (S5) the census ran in default mode, so it cannot count papers where the watch was on: 9 of the 419 mixed papers have authblk under an unwritten class (2005.13982, 2106.12864, 2108.03328, 2110.11194, 2211.06456, 2305.02433, 2311.07372, 2502.07150, 2609.05466) — re-run them with the census switch (deleted in DEL4; moot).
- Nits: `writer/mod.rs:259`'s outcome "declined: … beside a re-part" is also given for an unwritten call's creator and for unpaired names;
- `\inst` is unwatched while a raw `\inst` store is rerouted to an API call (store_setters.rs:36).
- Review `agent_reports/2026-10-31_rev95b_r3.md`
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none".

### AMS contact setters under raw amsart derivatives (next batch; 86c r1, reverted)

- **AMS contact setters under raw amsart derivatives (next batch; 86c r1, reverted)**: a raw class read over the AMS binding re-dresses `\address`, `\curraddr`, `\email`, `\urladdr` as stores only its own `\@maketitle` prints (stacs.cls:294-298, stacs_proc.cls, rmmcart.cls; m2an.cls, edpsmath.cls, pcms-l.cls:481 `\csname @address\number\addresscount@`), and our deposit of that `\@maketitle` is dropped, so the addresses are printed nowhere (0902.1737, 0802.2833).
- Repros (no guard): `sectioning-frontmatter/ams_raw_class_address_setters_stored` (RED) and the six above;
- A lock on the four against the files (`"{cs}:locked@files"`) keeps them, but alone it imposes the binding's argument meaning on classes whose arguments differ — reverted until it ships with: (1) a stacs/stacs_proc binding: `\address[ref]{…}` an address with no name (the key is matched against `\author[ref]`, never printed; the lock made it a visible `<contact name="uniroma">`, 0902.1737's `lab1`–`lab4` in 0802.2833), and `\author[ref]{short}{long}` (stacs.cls:256) the long name as the person (today a stray long-name paragraph);
- (2) a pcms-l `\auth` binding (pcms-l.cls:203, reached through the paper's `\renewcommand{\auth}{\secdef\@auth\@sauth}`, pcmslmod-modified.tex:89), else the address goes to a nameless creator ahead of all (math0505518, math0602226);
- (3) OD #803's empty-shell drop extended to the deposit's footnotes and pubnotes whose content became contacts (m2an_mod.cls:118 `\footnote{\box\@b@ddrss@}`: an empty `<note role="footnote" mark="2">` in 2502.17950, 2309.08130, 2411.09498, math0701227; an empty `<pubnote/>` in 2305.18054, 2604.25825);
- (4) the lock with a wrapper opt-in that lets a file's redefinition through when it still calls the binding through a `\let`-saved copy (epigamath.sty:227-232, 2206.02674, 2304.03539, 2305.12241; MyCommA.sty:530-533, 2602.16384), where S1: the call must be real — at brace depth 0 and outside a conditional, not a mention (`sectioning-frontmatter/{ams_raw_class_address_setter_conditional, ams_raw_class_address_setter_deferred_call}`, review probes `p_cond`, `p_wrapstore`);
- S2: a wrapper installed with `\let\address\wr@address` takes the same test on `let_i`'s `:locked@files` path (`ams_package_let_wrapper_keeps_both`, `p_letwrap`);
- S3: a class's `\AtBeginDocument{\renewcommand\address…}` runs at the document's origin (OD #596) and bypasses the lock — fix or record (`ams_raw_class_address_atbegindocument`, `p_atbegin`).
- Boundaries the lock must keep: the document's own redefinition (1212.1375, 2206.01924; `ams_document_address_redefinition_stays`), a package's wrapper (`ams_package_wrapping_setters_keeps_both`).
- the r1 code is branch `pk/86c-r1-backup` (state.rs `calls_locked_meaning`, ams_support_sty.rs).
- Its census (`ab_66bundle/pop86c.tsv`, 880 AMS-derived/redefining papers + a 292-paper amsart sample): 14 changed, also 1909.04033, 2001.07042, 2609.01974 and 2411.00367 (d-omega3.sty: E 18 → 9, `\organization`/`\orcid` still lost).
- Review `agent_reports/2026-10-28_review_86c_r1.md` (B1–B3, S1–S3);
- from the r2 re-review: B1's visible `name` label is what Perl itself gives 0902.1737 (it never reads stacs.cls, 1 missing file, and keeps `\address[ref]`'s key as the contact name), so the stacs binding goes beyond Perl rather than restoring parity;
- its guard must assert `@name` (the `creators()` helper cannot see it);
- the lock paths that held in r1 (`\let\address\relax` + `\newcommand`, a `\renewcommand` in a class option; probes `p_relaxnew`, `p_option`) are its boundaries;
- 2305.18054's thanks note loses its "∗" mark and its two-name `\author`'s address attaches to the last name only
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "AMS contact setters under raw amsart derivatives (next batch".

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

- float.sty's `\restylefloat{<type>}` of a kernel float (figure, table) records the style only (float_sty.rs `record_float_style`), so without caption the caption keeps the format it had: a ruled figure "Fig. 1: text" under IEEEtran (kernel ": " under article) where float.sty's `\floatc@ruled` prints "Fig. 1 text" (review rev78y probe3/rs.tex);
- State: OPEN (pre-existing; found at 78y r2)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "float.sty's `\restylefloat{<type>}` of a kernel float (figur".

### acmart copies older than TL 2025's v2.16 (v1.55-era, review rev78y acmcls/cs12.cls) set sigchi's small caption fonts on figures only, not tables; acmart_cls.rs ports the …

- acmart copies older than TL 2025's v2.16 (v1.55-era, review rev78y acmcls/cs12.cls) set sigchi's small caption fonts on figures only, not tables;
- acmart_cls.rs ports the TL version's.
- Fonts only, so behind the class caption fonts knob (caption_sty.rs `CLASS_CAPTION_FONTS`, off by default): no output difference unless the knob is set.
- 78y r2
- State: OPEN (knob-only; per-version port not planned)

### A preload's options do not reach a package another preload already loaded, in either order, so until 78z the class caption fonts knob could not be combined with ar5iv.sty

- A preload's options do not reach a package another preload already loaded, in either order, so until 78z the class caption fonts knob could not be combined with ar5iv.sty: `--preload=[classcaptionfonts]latexml.sty --preload=ar5iv.sty` sets the knob but loses every option ar5iv.sty passes to latexml.sty (`ids`, `rawstyles`, `rawclasses`, `magnify`/`zoomout`, the limits — `\documentclass{paper}` falls to OmniBus with `Warning:missing_file` where ar5iv alone reads paper.cls raw; the magnify PIs and `<p>`/autoref ids gone), and `--preload=ar5iv.sty --preload=[classcaptionfonts]latexml.sty` sets no knob (no `\ds@classcaptionfonts` runs).
- State: the class caption fonts knob FIXED 78z (guards `perfect_kernel_batch78z::{ar5iv_classcaptionfonts_option_sets_the_knob, ar5iv_classcaptionfonts_option_keeps_ar5iv_options}`); the general repeat-preload behaviour OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A preload's options do not reach a package another preload a".

### Determinism (det354 B2)

- Determinism (det354 B2): the `xmlns:*` declarations on the root `<document>` are in the order of the model's namespace table (`common/model.rs:1244-1254`, a `SymHashMap` at `:55`, → `document.rs:774-830`), not the order the document first uses them, as Perl declares them (Document.pm:1393-1394,1874): `\usepackage{fontawesome,tikz}` then `\faTimes … \tikz\draw (0,0)--(1,1);` declares `xmlns:svg` before `xmlns:aria`, though `aria` is used first (trigger det354 `repro/ns.tex`; Perl declares `svg` only).
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Determinism (det354 B2)".

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

### Before the flip: the switch-on leftovers — open items (the row's history, items 1-39, in the archive)

- The whole row (DEL1/DEL2, LO1-LO4, E1, the content check, the readiness re-measures, the flip's gate and no-harm pass) verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Before the flip: the switch-on leftovers".
- Left: (3) **1812.07638's `Contributors:` heading** (a line between two names rows) read as the six editors' affiliation: a role heading is its names row's label, which the API has no kind for yet (options: a note of the block, the row's creators' contact of a heading role, or left as today).
- (3c) an in-block `$^1$Univ (ECT$^*$) and …` splits the entry at the symbol with either switch (a symbol glued inside open brackets in the lexer's `glue_in_word_marks` fixes it, and moves the switch-off output).
- (5) **BESIII's sub-labels** (R6, tentatively ruled): 2204.09614, 2404.06718 leave 48 and 51 words silent (`68A,68C` answered by entry 68).
- (6) **1810.03464's rehearsal loops** past its read budget (vldb's `\normalsize` in `\email{…}`; trigger in its own row).
- (7) **The tabular grade** `…, Fellow, IEEE\end{tabular}` reads `\end{tabular}` into the grade (E=2, off too).
- (9) open part: the switch-off cost +0.18 to +0.22 % `instructions:u` over LO1 on 1507.02068, 2103.04797, 1901.01540 (from LO2) — to bisect (`kit/perf_bisect.sh` over 3205f4ba81..df0926bd9b); the rest of (9), the tests under `=2`, done before the flip.
- **The residual, flipped with as named REDs (user 2026-11-09):** `docs/parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`.
- Open beside it (REDs): the author-field Errors silenced since the flip (the PRIORITY RED: 18 A/B papers, revDEL4b) are raised again by ERR2 (below; OD #983);
- a block whose `\footnote` counter steps inside `\author` (14 A/B papers) goes whole as one creator "Ann Able  and Bob Baker" where Perl's article writes two, so that Warning's "as Perl's whole-block creator" is false there;
- a `\stepcounter` of another counter inside `\author` is lost (0 vs Perl/pdflatex 1; 0 A/B papers; repro `observer_rehearsal_loses_an_author_stepcounter.tex`);
- DEL3's list-setter port (OD #982), synthetic only (2 of 73,251 papers have the shape, neither these cases; probes `scratch_revDEL3b/rp/`): R1 a top-level svjour `\institute` *after* the block takes over — the block's Univ X / Lab Y lost with 2 orphan Warnings, where pdflatex prints the block's list (`\maketitle` sets it last; fix: clear the epoch's affiliation calls up to the flush), R2 an entry of the written setter call no mark asks for is dropped with an orphan Warning (`sv_unref_in`: pdflatex prints "2 Lab Y, City"; base and off keep it on a nameless creator; `sv_typed_sup` the same);
- the ORCID logo of an icon-only `\href` is lost at `=2` (OD #960; int153m `=2` vs off: 2507.12319 4→0 logos, 2406.08258 2→0; pre-existing at the flip, not DEL3);
- the ejpecp e-mail notes the class queues besides its creators are lost at `=2` ("holds no place … (ltx:note); it is lost", counted; binding net ejpecp 3 of 30 worse: 1809.03082, 1310.4722, 2609.22952 — rows in the residual TSV);
- NE5's `declaration_reads_past` writes six A/B papers' blocks whole that the candidate before it split (0809.5152, 0906.2789, 1110.6375, 1404.5602, 2404.18592, 2503.21719; 0 distinct words lost, three also drop a queued `ltx:date`, counted) — narrow it to a definition whose replay actually reads past;
- `resolver/mod.rs` takes every held Error in an IJCAI block as a section marker, not only those at `\affiliations`/`\emails` (revX7, unprobed);
- NE8's IJCAI cut is picked by an `@` in a held section's printed text, not by the split's own delimiter (`resolver/blocks.rs` `address_section`; revX5, synthetic `scratch_revX5/syn/a1.tex`: e-mails inside `\affiliations` with no `\emails` → at `=2` the held `\affiliations` Error off also raises, and "Univ B \affiliations"; fix: cut at the marker whose held token is `\lx@ijcai@affilsplit`'s `Until:\emails` delimiter, read from its definition; no stage2 paper reaches it);
- a frontmatter TikZ picture with no text inside `\href` (`\author{Ann\\ \href{https://e.org}{\tikz\fill (0,0) circle (0.1);}}`, or in `\thanks`) is dropped with no Warning, off and `=2` (NE7; needs a kept contact/note or a counted Warning);
- 1602.03850-shaped names-only macro overlaps written as each name's text;
- two unrecorded calls in one flush;
- 1901.02012's sequenced e-mails counted lost, not owned;
- AIMS's `\email` label printed at the end (1903.05770);
- an `\emph`-wrapped class label leaves an empty wrapper;
- a label's font not carried into `name`;
- `E-mail: \email{…}` with a class `\email` printing "E-mail:" loses one;
- `content::reads_past`'s end-of-run rule (a run ending at a call none of whose arguments it holds) has no guard — no input reaching it was found (a piece a macro prints spans its arguments, `\newcommand\place[1]{Princeton}` + `$^1$Univ \place{x}, $^2$Univ B`; a mark's span is grown by `arguments_end` first), so it stays a backstop (NE8);
- since the flip a `\stepcounter` in an author field is lost: `\author{Ann Able\stepcounter{probe}\ifx\foo\undefined\gdef\foo{ONE}\else\gdef\foo{TWO}\fi \and Bob}` with `Value \foo{} and \theprobe.` in the body gives "Value ONE and 0." at `=2` (0 errors, 0 warnings: the `\gdef` kept, the step lost), where Perl, pdflatex and off print "1" (revDEL4 P1; repro sectioning-frontmatter/`observer_rehearsal_loses_an_author_stepcounter`, pinned as today's in `perfect_kernel_batchfms2a1::the_rehearsal_leaves_nothing_behind`);
- an author field's undefined commands, whose Errors the rehearsal held and discarded, are raised once since ERR2 (OD #983): in place where a piece holds them (after a name, past its marks and notes; an address whose local part an undefined command glues a word's ink onto — ink of the same word on both sides of it before the `@`, a Name the ink table cannot align failing closed on its span — is never read as one, any other read as without the Error, so `\author{Ann Able\undefinedprobecs \and Bob\email{b@x.org}}` gives Perl's 2 Errors and creators Ann Able and `Bob<ERROR>\email</ERROR>b@x.org`), else by the flush's net (a piece written as text);
- the flip A/B's 19 papers: every lost `Error:undefined` back except the artefacts off raised and pdflatex does not (kr.sty's `\affiliations` in 2405.06973, 2602.18351, 2604.26688; 2405.13308's `\@thefnmark`/`\@undefined` from a digestion that lost both names), creators never fewer than DEL5's, 0 words lost, stage-2 1,434 unchanged;
- the `\fi` chains of 1603.08231 (18) and 2112.06251 (12) are Perl's split at `\and` (KNOWN_PERL_ERRORS #1218; pdflatex 0), correctly absent.
- ERR2's REDs left by its last review (revERR2c, SHIP): one undefined command after `\footnotemark` writes two `<ERROR>` stubs while logging its one Error (`\author{Ann Able\footnotemark\foo{} \and Bob Baker}`; Perl and pdflatex one; ERR2-introduced, 0 of the 358 sample papers; suspected `emit.rs` `block_names` taking the held command into `apart` while the note piece grows over it by `content.rs` `past_held_undefined`) — the next batch;
- the span fallback's `@` test is end-exclusive (`b\foo@x.org` keeps the invented `Baker1b@x.org`, `b\foo{}@x.org` declines);
- pre-existing on the base: a superscript mark glued before an address is read into it (`Bob Baker\textsuperscript{1}b@x.org` → e-mail `Baker1b@x.org`), and `\author{Ann Able, ann@x.org\footnotemark \and Bob Baker}` gives one creator (TeX 2, Perl 3; no word lost).
- Reviews: revNE1, revNE2, revNE2r2, revX1, revX2, revCL9b, revCL10 (and the fix rounds' re-reviews).
- Reports: `agent_reports/2026-11-07_{F1,F1r2,revF1,revF1r2,N1,revN1,LO1,revLO1,N2,revN2,N2r2,LO2,revLO2,LO2r2,revLO2r2,tests_sort}_report.md`, `2026-11-09_{NE1,NE2,NE1w,NE2w,NE2b,NE2c,NE3,CL5,CL6,CL7,CL8,CL8b,CL9,CL9b,CL9c,CL10,revNE1,revNE2,revNE2r2,revX1,revX2,revCL9b,revCL10}_report.md`, `2026-11-08_{T1,revT1,N2r3,revN2r3,LO3,revLO3,LO4,revLO4,RD1,P2,revP2,P2r2,revP2r2,RC1,revRC1,revG1,revG2,revG3,revBC1,revG5,revH1,revH2,revH3,revI1,revI2,revI3,revI4,revI5,revI6,revI7,revI8,cw78,ts9}_report.md`

### Open residuals lifted from landed rows (2026-11-09)

Each landed row is verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](../archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1); these are its open facts.

- **Bundle int147b (LO2 + LO2r2, the second switch-on leftovers; T1, the tests at `=2` part 1; N2r2 + N2r3, the content check's fix rounds; each reviewed SHIP (revT1 NO-SHIP …**:
  - Readiness (`=2` vs off on LO2r2-rel, scored; dev20k pending): dev3 per creator 98.89 / 98.88 %, per paper 98.51 / 98.57 %;
- **Bundle int133a (98d + 105a; code `9a716bc81b`)**:
  - the 9 frontmatter changes all 105a gains (e-mail contacts 1305.0854, 2001.07747; PACS codes 1311.3004; Chinese address lines 2206.01007, 2601.14329) but 2108.12618, whose kept template note ("Instructions for Creating an FnT Article") gains its "2018" (placeholder ruling pending).
- **98d: `\end{X}` over a `\begingroup` opened inside X (kernel; SYNC "The unclaimed-text net" R0; OD #965, KNOWN_PERL_ERRORS #1207)**:
  - More in the archived row.
