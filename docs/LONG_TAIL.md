# The arXiv long tail (v0.8.2 and later)

**What this is (DG 2026-11-23).** Work specific to the long tail of arXiv articles: every task here affects fewer than
200 arXiv articles (of ~2.8M) by its own evidence — a census count, a short witness list — or is synthetic-only (a
minimal repro, no corpus witness). It is out of scope for v0.8.0 and v0.8.1, whose work is
[`SYNC_STATUS.md`](SYNC_STATUS.md) row R ([`GOAL_v_0_8_1.md`](GOAL_v_0_8_1.md)).

Rules:

1. **Same format as row R:** one `  - ID — task; weight` bullet per task (weight: easy 1, medium 3, hard 5),
   continuation lines indented four spaces; an id is stable across both lists.
2. **What belongs here:** a task whose reach is shown under 200 articles, or which is synthetic-only. User-reported
   issues, tooling/CI/performance items and tasks with no reach evidence stay in row R until measured. A task whose
   reach turns out to be 200 or more (a census, a new witness cluster) moves back to row R with the evidence.
3. **New long-tail findings are added here,** not to row R. Writers never edit this file; the main session folds
   their SYNC notes in.
4. **Closing:** a closed task moves, in conclusion form, to the month's `archive/SYNC_SESSIONS_YYYY-MM.md`.
5. **Parked work:** branch `pk/REGTAILD` (amsthm's `[\relax]` parent counter, sidecap's `SCfigure` running the
   class's `\figure`, mathtools' robust paired delimiters; REGTAIL papers 2208.14960, 2309.00854, 2409.18804,
   1510.03282, 1511.05217, 2006.04669, 2202.13838, 2408.12455) is finished and unmerged, for v0.8.2: origin
   `pk/REGTAILD` at `b0f34364ecb` (on `int/towards_v0_8_1` `35a85b96669`), gates green (8786 tests), A/B 502 papers:
   E 7031 → 5071, 11 changed, 0 worse; OD #1156 and KPE #1328 reserved; guards `perfect_kernel_batchregtaild::*`.

**The long-tail list** (222 items on 2026-11-26: weight 341, 15 without a weight — counted as the `  - ID`
bullets below; moved from row R by DG's ruling of 2026-11-23, in row R's order):

  - ODR1 — OD #973's pgfmath residuals (moved from OD by the 2026-11-14 rationalization): string operands, units after array primaries, grouping, out-of-bounds indexing (repro `graphics-tikz/pgfmath_string_and_units_in_arithmetic_red`; every case's pgf-vs-ours values: OPEN_ROWS ODR1); 3
  - ODR2 — OD #976's tabularray `preto`/`appto` unwarned losses (inline expl key, the inner key's two-argument form, empty `preto`/`appto`, the `\SetTblrInner` order; repros `alignment-bindings/tblr_inline_expl_command_in_a_key_red`, `tblr_preto_in_a_two_argument_cell_key_red`; detail OPEN_ROWS ODR2); 1
  - ODR3 — OD #1019's mark-led names: REDs `article_mark_led_unanswered_trailing_mark_uncounted_red` and the two siblings; 1
  - ODR4 — OD #982: a top-level `\institute` after the author block is not cleared (revDEL3b R1, RED); 1
  - ODR5 — KPE #1267 (formerly OD #327) box sizes, K18 step 2: `\raisebox`'s `[height][depth]` with `\height` a 0pt stub, `\resizebox{1em}` 10.00002pt vs pdflatex 10.00081pt, makecell/diagbox string sizes (RED repro `boxes-groups/raisebox_optionals_measure_the_box`; detail OPEN_ROWS ODR5); 3
  - REGC — regression C (DG 2026-11-15, ruling (b)): GREEN since REGCDE r2 — at `\begin{document}` every `\@preamblecmds` command except Perl's 7 loaders is `\@notprerr` (latex.ltx:9521-9522; `loader/preamble_only_command_is_notprerr.tex`, OD #226). Left: two synthetic REDs, catalogue only by the pragmatic bar — `loader/loader_is_not_notprerr_red.tex` (the ruling's own cost: scrlttr2's `\ifx\RequirePackage\@notprerr` in the body) and `loader/declaremathoperator_is_preamble_only_red.tex` (amsopn's `\@onlypreamble\DeclareMathOperator`); 1
  - SR5 — SR5 (QA60 site 8): registers set only at \maketitle lose their notes uncounted (RED net_title_note_on_the_authors_set_at_maketitle_is_lost_red); 3
  - MATHR — The math silent-diagnostic RED row; 1
  - MTARGDELIM — (MTARG reviews, synthetic, no arXiv witness) residual shapes of an argument-taking class `\@maketitle`
    (OD #265), each losing title-page text that only an Info `frontmatter:class_text` counts, where the ruling wants a
    Warning: (a) a delimited `\@maketitle` (`\def\@maketitle#1.{…}`) whose replayed `\maketitle` is dropped for an
    error gets no deposit (repro `sectioning-frontmatter/maketitle_delimited_argument_class_keeps_its_text_red.tex`);
    (b) a class `\maketitle` that calls `\@maketitle` only when `\@title` is set replays on emptied fields, typesets
    nothing and is kept, so no deposit runs (fix: decide from the speculative run on the real fields,
    `note_title_page_print`, whether it called `\@maketitle`); (c) an `\@latex@error` inside `\@maketitle` on an empty
    title errs again in the fallback. Also: a second `\maketitle` prints nothing where pdflatex repeats the title page,
    and `\lx@saved@maketitle` stays defined after `\maketitle` (harmless). rc2 kept (a)-(c)'s text with the open-group
    Warning; 3
  - TBLR — tabularray first-token, \g__tblr_table_count_int, tblr-only \hline REDs; 1
  - QA11AR — QA11A's residuals (PR #872; QA11A, revQA11A): `\pgfmathsetlength{+…}`'s quick path `\unskip`s the space typeset before the call in pgf, ours and Perl keep it (RED `graphics-tikz/pgfmath_setlength_quick_path_unskips_red`); the same path skips spaces before putting back its leftover tokens, pgf the other way round, output identical (pgfmath_code_tex.rs:2783); `\pgfmathpostparse` not ported (the raw parse calls it before returning); a pgfmath primitive inside an `\edef` (`\edef\x{\pgfmathparse{1+2}}`) raises no Error where pdflatex errs and stops (Undefined + Incomplete `\iffalse`; base 1 E; RED `graphics-tikz/pgfmath_parse_in_edef_errs_red`; reach ≈ 0); 1
  - QA76AR — QA76A's review residuals (PR #871; revQA76A): apacite `\citeA`/`\citeNP`/`\citeyearNP`/`\shortcite*` defined in every citation mode where apacite.sty defines them in classic only (:258-584) — RED in QA76Ab; svmult v5.12 `\keywords{A \and B}` after `\maketitle`: pdflatex 19 `!` (kernel tabular `\and`), ours 0 E — RED; unchecked: svproc/webofc/wileymsp keyword `\and`, natbibemu mode; 1
  - QA90R — QA90's residuals (#867): latex.ltx's token preamble-only macros (`\makeindex`, `\makeglossary`, `\DeclareTextCommandDefault`, …) stay silent in the body (`loader/makeindex_in_the_body_red.tex`; the loader runs latex.ltx's filename-stack macros in the body; reach ~0), as do body token macros listed in `\@preamblecmds` (`\makeindex`, a document's `\@onlypreamble\myonly`: `loader/document_onlypreamble_macro_in_the_body_red.tex`); the walk's recovery (Perl's: Error, then run) is wrong for commands whose setting later commands read — `\includeonly` in the body drops `\include{cb}` where pdflatex ignores it and prints both (`loader/includeonly_in_the_body_red/`): (a) defer, (b) a small list of commands that only report the Error; 1
  - QA91R — QA91's and DEPN's residuals (#859): the wrap reads the title and author marks, one answer per block, so a class printing its own title store, or the title and the authors apart, is placed wrong (`sectioning-frontmatter/text_before_maketitle_where_a_class_prints_its_own_title_store_red`, `…_where_title_and_authors_print_apart_red`); `\date{Spring 2024}` printed twice in the deposit ("Spring 2024A=0", gullet.rs:2670 `unread(value)`); a float between the paragraphs before `\maketitle` splits them; `\maketitle` inside `{titlepage}` leaves the cover text after the title (base too); stray `*1`, `]`, `,` more visible at the top (QAB4b); 3
  - QA17 — `\iffontchar`/`\fontchar*` read `<font>` as a plain Token (etex.rs:620, :253; tex.web §577): `\textfont1 "80` typesets `”80`, `\nullfont` answers Y; the NFSS current font has no file, so the answer is always TRUE, and the identifier's file ignores the encoding (tex_fonts.rs:858; latex.ltx `\get@external@font`:12538 gives `ecrm1000`) — fix: a `FontToken` + `\nullfont` = empty (EASY) together with the identifier's file from `\get@external@font` in `pickup_font` (MED), since the first alone makes T1 wrong; limit: the font-package family stays CM (sect13.rs:907); RED `fonts-nfss/qa17_iffontchar_red.tex` (synthetic; pdflatex `[a:N][b:N][c:N][d:Y][e:cmbxti10]`); reach 0 author uses in 35,942 papers (12 bundled package copies); 3
  - QA58R — #858's residuals (QA24, QA58, QA64; QAB57): `\parbox[t][3cm]{4cm}{…}` and `minipage[t][2cm]` lose their height in the XML, Perl too (`block-model/parbox_and_minipage_height_dropped_red`); a framed box holding a `\section` draws its frame as an empty box above it (`block-model/framed_box_section_frame_collapses_red`; main too); a shipped `\usepackage{mydefs}` that defines `\email` still loses to OmniBus's guess; a document's hook definition refused under OmniBus (`singletons/omnibus_document_hook_definition_refused_red`); a latexml option in the document after `--preload=ar5iv.sty` (`backend-persona/latexml_option_after_ar5iv_preload_red`); 1
  - IEREG — IEEEtran `\IEEEauthorrefmark{1}\the\toks` / `\csname annmail\endcsname` address line: 1 Error, an invented e-mail "0", the address lost, counted (the token e-mail line reader; `ieeetran_authorblocka_register_address_line_red`; P6f); 1
  - BLET — beamer `\let\sep\and` inside the `\institute` list: 1 Error, a raw `\sep` left in the text, base and P6f2 alike (`beamer_institute_let_of_and_inside_the_list_red`; P6f2); 1
  - BAND — beamer's `\and` that pdflatex prints as a line break inside parentheses becomes the word "and", counted by its Warning (`\institute{\inst{1}Univ A (North \and South) …}`; revP6f p1); 1
  - ORCR — acmart `\orcid` between two `\author` calls ends P6e2's run of names, the shared block's first e-mail on the last name (`acmart_orcid_between_shared_names_email_on_last_red`; revP6e2 finding 1; no corpus witness); 1
  - ACOE — the content check's false "placed more often than printed" Warning for an address printed under two own calls (`acmart_own_email_call_stays_whatever_it_spells` pins W=1 on int168a, int169a, P6e2): a RED catalog repro; 1
  - MARKTXT — a mark's text kept in a name: PoS's first name (`pos_leading_mark_kept_in_the_first_name_red`), an unanswered trailing mark uncounted (`article_mark_led_unanswered_trailing_mark_uncounted_red`), nosuchclass "1 Ann Able", `$^1$Ann \and $^1$Bob`, an unanswered mark moved after its name (CL170 R2); 3
  - ISO — an unrequested isotope line loses its mark (`article_unrequested_isotope_line_loses_its_mark_red`; MLN1); 1
  - ROW1 — a mark-led second names row of one name read as an affiliation, a silent partial loss (`article_mark_led_second_row_of_one_name_read_as_affiliation_red`; revMLN1 2); 1
  - PTPM — ptptex `\author` marks after a comma consumed, uncounted (CL170 R1; probe `scratch_CL170/probes/ptp_a.tex`); 1
  - UNWE — D2's unwrapped e-mail command never runs its other effects (`raw_class_store_email_command_effects_lost_red`; revP23e 3); 1
  - XYZC — `\author{Ann\inst{1}, Bob\inst{2} \and the XYZ Collaboration}`: the institutes' e-mails on a nameless creator (`aa_comma_names_and_collaboration_emails_on_the_block_red`; revP23b 3); 1
  - LLCM — llncs's comma-parted marked list: the e-mail line the block's (`llncs_comma_parted_marked_list_email_line_is_the_blocks_red`; P23b); 3
  - SVMH — svjour3: a macro holding names and the separator, the e-mail line the block's (`svjour3_marked_list_macro_holding_names_and_separator_red`; P23b); 1
  - RAWL — a raw class's in-place `\email` label repeated per contact / its prose (`raw_class_in_place_email_label_under_numbered_institute_red`; revP23c S1); 1
  - AACM — aa `Bob\inst{2}$^{,}$\inst{1}` keeps the comma in the name (revP23c l4); 1
  - LLGL — llncs `\institute` legend line glued into the last institute (revP23c l5); 1
  - AFNR — two identical `\affiliation{…\footnote{…}}` write two pubnotes, no Warning (`aastex6_footnote_in_repeated_affiliation_doubled_red`; AAS1); 1
  - AAGR — AAS1's guard: a repeated affiliation content that must NOT be reused (`\stepcounter`/`\footnotemark`, A A B A) (revAAS1 NIT); 1
  - GTXT — GeneralText/XGeneralText arguments revert without braces (`\mark{x}` → `\mark xb`; Perl `\mark{x}`; RED `reversion/mark_general_text_reverts_without_braces_red`); 1
  - PGU — pgfmath unit conversion in scaled points (`2bp` 2.0075 vs 2.00749), pdfTeX's `px`/`nd`/`nc` and `mu` units (RED `pgfmath_units_as_tex_reads_them_red`); `08+1` octal (RED `pgfmath_octal_bad_digit_red`); 3
  - PGCNT — a count register then a dimension (`2\myc\mydim`: pdflatex 15.0; RED `pgfmath_count_then_dimension_drops_the_dimension_red`) (revPGR1); 1
  - QAHK — a `cmd/` hook's owner read from its label: a package's body hook a Warning, a document's hook labelled with a loaded package an Info (REDs `command_hook_a_package_adds_in_the_body_red`, `command_hook_a_document_label_names_a_package_red`; one fix: record `\@currname` when the code is stored) (revQA47Ab); 1
  - GPAR — group-paragraph geometry REDs: `\leavevmode{ \par}x` (P1), `\noindent{ }\par` (P2), a group's `\strut`/`\vspace`/leader or a blank piece after `\small` (P3, PE2), a glue-only formula (PE1), a forced break's last line (PE3) (RG1c, revRG1c); 3
  - PBXL — a marked list printed entirely inside one `\parbox` is one entry (`cut_list` splits at the top level only; RED `ieee_marked_list_inside_one_parbox_is_one_entry_red`) (revP4c S1); 1
  - CALCR — calc's `read_term` does not skip `\relax` before `*`/`/`: a stray "2+1" printed (`\setcounter{a}{\value{b}\relax*2+1}`; revGA1e S2); 1
  - FNMK — footnote marks: `[\relax 2]`/`[\value{nosuch}2]` read as digits, a braces-only tail silent (`footnotemark_relax_then_digits_reads_digits_red`), a blank `[]` (`footnotemark_blank_is_tex_zero_red`), `\ifnum` with a missing number (`ifnum_missing_number_inserts_equals_red`) (GA1f, revGA1f); 1
  - P4FR — P4f's residuals: llncs `\inst` names + `$^n$` institute, beamer, unmarked names + marked institute (REDs `llncs_inst_names_marked_institute_marks_hidden_red`, `beamer_marked_institute_marks_hidden_red`, `llncs_unmarked_names_marked_institute_marks_lost_red`) (P4f); 1
  - MHAF — REVTeX: an `\affiliation` whose marked lines a document macro holds (`\affiliation{\Insts}`) is one nameless creator, where the inline twin reads the lines under their names (`revtex_macro_held_affiliation_lines_nameless_red`; CST2); 1
  - CSNL — a mark macro built by `\csname…\endcsname` in a name (`A.~Smith\csname mA\endcsname`): 2 Errors on REVTeX, 1 on IEEEtran ("\let should not appear between \csname and \endcsname") and the marks lost; pdflatex 0 (`revtex_ieeetran_csname_built_mark_macro_errs_red`; revCST2); 1
  - TYST — a typed `*` beside `\,` / `\ ` in a name is read as a mark by the rehearsal the `\,` triggers (nameless creator), where the gate reads typed characters as text; option (a): the rehearsal reads them as text too (`revtex_typed_star_beside_thin_or_control_space_nameless_red`; CST2b); 1
  - IEFL — IEEEtran: `\IEEEauthorblockN{A. Smith*, B. Jones}` over `$^1$`/`$^2$` lines gives A line 1 only, where the PDF prints one block under both names (`ieeetran_typed_star_name_takes_first_line_only_red`; CST2b); 1
  - P4DEC — a macro of font declarations (`\newcommand\affilfont{\small\itshape}`) is carried to the first cut piece only (int165a carried it to all; restoring that brings back the printing-macro duplication, revP4b4a2 B1) (`llncs_institute_marks_macro_declarations_lead_not_carried_red`); `FONT_DECLARATIONS` repeats LAYOUT's head and LAYOUT lacks `\tt`/`\LARGE`/`\huge`/`\Huge`/`\normalcolor` (revP4b4f NIT); 1
  - P4IAL — IEEEtran's `\IEEEauthorblockA{\centering…}` first cut piece keeps the alignment (`ieeetran_authorblocka_centering_marks_first_entry_keeps_alignment_red`); 1
  - P4SET — the lead scan stops at `\setlength{…}{…}`, a later `\footnotesize` reaching no piece (`llncs_institute_setlength_lead_stops_carried_declarations_red`); 1
  - P4OWN — a minipage's one marked line kept whole on a nameless creator (`llncs_institute_minipage_one_line_marks_owners_red`); 1
  - ACMDUP — acmart `\country{$^1$…}` with an entry no author requests: the requester of the one before it holds both, twice (`acmart_country_unrequested_entry_duplicates_red`; revP4b4f); 1
  - IEADR — short marked e-mails kept one affiliation of the first name (`ieeetran_authorblocka_marked_short_email_list_whole_on_first_red`; P4b4f); 1
  - LGADR — a grouped marked address line `\{$^{1}$ann, $^{2}$bob\}@x` cut apart (`llncs_institute_grouped_marked_addresses_cut_apart_red`, int165a too); 1
  - MBAL — every multibib list holds every cited entry, each numbered from [1] (`index-bib/multibib_lists_hold_every_entry_red`); 1
  - IEGN — a grade inside a note, biography or the body dropped, as Perl (`check_binding_macro_drops_inside_an_entry_red`); 1
  - IEGW — a grade in a plain list's `\IEEEauthorblockA` raises 2 false content-check Warnings (`ieee_membership_grade_in_a_plain_affiliation_call_red`); 1
  - IEGF — a font-wrapped name and grade print the grade inside the name and keep the contact (`ieee_membership_grade_inside_a_font_wrapped_name_red`); 1
  - PSFM — a graphic measured in `\settowidth` then printed is counted twice (`reversion/psfrag_measured_graphic_counted_twice_red`); 1
  - P5BA — a bracketed collaboration call or column with its own affiliation: the affiliation a nameless creator or the first author's (3 REDs, synthetic; the "for the" form pre-existing; revP5b); 1
  - P6BF — an inline "… and Bob Baker for the X Collaboration" remark: the emitter's plain-text fallback, 2 counted Warnings, content right (`author_names_inline_remark_markup_fallback_red`; P6b); 1
  - P5SV — svjour3's `queued_author_name_prints`/`queued_author_names_spelled` read the names by tokens (`svjour_institute_piece_naming_a_macro_held_author_red`; P5c); 1
  - OSAN — OSA classes: an author name by `\csname` lost as "_" with a false Error; a name macro with arguments loses its space (`osa_author_name_by_csname_is_lost_red`, `osa_author_name_macro_with_arguments_loses_its_space_red`; P5c); 1
  - UNCR — a wordless `\thanks` written as its own binding command (REVTeX's after `\author`) goes the contact route and is lost uncounted (`revtex_wordless_thanks_after_author_uncounted_red`; revUNC); 1
  - UNCD — a wordless note counted twice when two readers meet it (`author_wordless_notes_counted_twice_red`; revUNC); 1
  - LOGOX — `\thanks{Hello \LaTeX}`: a false "excess" content-check Warning, the rehearsal's note text missing the logo (`author_thanks_logo_false_excess_warning_red`); 1
  - TP3H — a raw class's title page reading > 8 M tokens per run: a store only the run decides lost on the walk's fallback (the `\thanks` label), only the budget Warning naming it (`title_page_over_the_read_budget_loses_a_store_label_red`; revTP3); 1
  - AMSV — amsart-derived classes: `\@setcopyright` unbound in our amsart, the class's store ("Course: MATH 410") lost, counted (`ams_class_maketitle_with_unbound_internal_loses_store_red`; TP3, BM probe `ams`); 1
  - AUTP — a document macro WITH parameters holding names stays by text, counted (the end of its arguments unknown when the call ends the field) (`article_author_macro_with_parameters_holding_names_red`; AUT1); 1
  - AUTL — a leading mark after a comma (`, $^{1}$Bob`) lost, counted (`omnibus_author_leading_mark_after_a_comma_is_lost_red`; AUT1); 1
  - FNTN — `\fontname\font` under NFSS prints the CM name (Perl the same, `cmr10`); author reach 0 of 2,027 census papers; needs `.fd` → TFM in `\selectfont` (ROW0A, typog-example); 3
  - GINS — `\lx@Gin@include`/`\lx@Gin@setfile` local where `\includegraphics` is global (since batch 70a; graphicx loaded in the body or a group loses the graphic, Perl keeps it); EASY: `scope => Global` on both (ROW0A; the repro `gin_body` is not in the tree). Measured on rc2 2026-11-23: graphicx `\RequirePackage`d inside a body group — pdflatex itself stops ("Can be used only in preamble", an undefined control sequence) and we raise the same two Errors; `\AtBeginDocument{\RequirePackage{graphicx}}` converts clean. So no well-formed document loses its graphic: v0.8.1, low priority; 1
  - CTOV — CT1ar's count calls a space-separated repeat glued: `$^a$Dept of Physics $^a$Univ X` over-counted "a", XML right (`ctlabel_space_separated_repeat_over_counted_red`; DG accepted the over-count); 1
  - CTIC — an isotope after a comma (`, $^{3}$He Lab`) cut as a second entry of label 3, its "3" lost uncounted (`ctlabel_isotope_after_a_comma_uncounted_red`; revCT1ar); 1
  - CTSF — `$1$St Floor`: a math number leading a line read as a mark, its "1" lost uncounted (CT1a's St Floor repro; CT1ar); 1
  - P28G — guard rows for P28m2's generic take-back outside IEEEtran (an svjour3/kernel `\author` replaced: the nameless creator and its note dropped) and the `\let` copy defined only after `\author` (revP28m2's l1 probe); 1
  - P28C — `\ifblind Anonymous Authors\else <blocks>\fi` (literal or in a macro): a nameless creator with the words as its affiliation, where the base named it (`ieeetran_author_blind_conditional{,_in_a_macro}_red`; revP28m S2); 1
  - TABUN — tabu's private `|` let is inherited by a tabular nested in a tabu cell: `|[1pt]` accepted where TeX errs "Illegal pream-token" (`tabu_vline_style_in_a_nested_tabular_red`; revTABU2 N1); 1
  - TABUE — a `\tabularnewline` inside a list in a plain tabular cell: the recovery from the unbalanced group drops the table and every word after it (8 E, only "Before." kept; pdflatex errs but prints all) (`row_end_inside_a_list_in_a_cell_red`; revTABU2 N2); 3
  - TABUC — `{thebibliography}` in a p-column cell: `<ltx:bibliography> isn't allowed in <ltx:block>`, the content kept; pdflatex 0 errors (`block-model/bibliography_in_a_table_cell_red`; TABU2c); 1
  - TABUF — `{center}`/`{flushleft}` in a tabu cell keep their line breaks where TeX ends the row (tabu's `\centering` ends with `\arraybackslash`; pdflatex 104 `!`), 0 E (`tabu_center_env_backslash_red`; TABU2); 1
  - IEEO — IEEEtran preprocessing residuals (P4cR2): a refmark inside a font switch (`ieee_nested_block_bold_refmark_spliced_red`), a mark after a box (`ieee_nested_block_mark_after_box_spliced_red`), marks after `\\[2pt]`/`\newline` (`ieee_nested_block_after_spaced_break_spliced_red`) (revP4cR2b); an inner block inside a group stays nested (`ieee_nested_affiliation_block_in_group_red`); the ordinal content-check false pair (`ieee_ordinal_floor_line_names_unnumbered`, 2 W); the Cut gives an unmatched address in an inline marked list to the last entry's author; two sibling blocks after one names block reach the last author only (revP4cR2 q14); a block inside the outer's `\thanks` read as an affiliation (q1); 1
  - F2 — Small RED repros (`\vadjust` scope; OPEN_ROWS' RED catalogue); 1
  - MEMW — memoir: `\after<ext>title` text before the TOC title (RED `memoir_after_list_title_text_precedes_the_title_red`), `\cft<ext>beforelisthook` text after the list (RED `memoir_list_before_hook_text_red`), the `\newlistof` wrap not idempotent (an old ob-toclof), the chapter-style case not carried into list titles (MEM1; `2026-11-13_brief_revMEM1.md`); 1
  - LEGAB — author-block legend lines (`\textsuperscript{$^*$ prose}`, Perl the same: RED `author_line_superscript_wrapped_legend_lost_red`, 2502.05252) and a mark item's symbol tail not split before label matching (REDs `author_list_mixed_mark_note_becomes_nameless_creator_red`, `authblk_author_mark_list_mixed_symbol_note_becomes_nameless_creator_red`; 2509.15733, 2606.27251); reach 1 of 35,942, DG 2026-11-13 post-merge by reach (`2026-11-13_LEG0_report.md`); 2
  - BRKC — a breakable tcolorbox inside a minipage or float is not broken by TeX, yet its overflow is a Warning (REDs `tcolorbox_breakable_box_in_minipage_overflow_red`, `tcolorbox_breakable_box_in_float_overflow_red`; revSBX1); 1
  - FORC — article `\and for correspondence: …` read as both authors' affiliation, 0 E (revSBX1 probe n3; CAU1 too); 1
  - GATHER — linenoaa's `{gather}` on amsmath internals (RED `amsmath_start_gather_internals_red`; no sandbox witness; SBX1); 1
  - WRAPN — user wrappers around marked affiliation lines (`\inst`/`\myaff` around `$^n$` lines, `\textsc{\textsuperscript…}` in `\thanks`, `{array}` marks): lost, duplicated or misordered, counted (pre-existing; probes scratch_revSBX2F/p/u4_inst, x3_authblk, v8_tsup, t7_array, x2_myaffsuf; revSBX2F); 2
  - TKAR — `takes_arguments` reads "hands on an optional argument" from the spelling (RED `thanks_mark_list_parameterless_macro_ending_in_bracket_red`); FWEND — `Field::whole`'s `\end` spelling → the frames (SBX2F); 1
  - GRPRD — a call reading across `\and` moves words into the next name (RED `article_author_declaration_reading_across_and_moves_words_red`); a call defining then printing its argument leaves a later macro undefined (RED `article_author_call_defining_then_printing_its_argument_red`); `field_position_encloses` compares positions only (NIT) (revGRP1, `2026-11-13_brief_revGRP1.md`); 2
  - APCOL — `\author{X Project \and Ann Smith, Group}` makes Ann an affiliation (RED `author_project_column_takes_the_next_column_as_affiliation_red`; plain words on the base too); `\author{Ann \and Bob, \TeX\ Users Group \and Cat}` one creator (LOGO1, revLOGO1); 1
  - IFGEO — #845 (the upstream sync to `0db6f47e`) left as upstream ships it: `ifgeo10` also sizes some non-ifsym glyphs (wasysym `\CIRCLE`, marvosym `\SquareSteel`) at 1em (KPE #1245; RED `fonts-nfss/ifgeo_fallback_sizes_non_ifsym_glyphs_red`; a 20-paper census changed no output); 1
  - DELC — `\delcode` holds 24 bits: the delcode table as an integer table (the dump's DELCODE_TABLE, tex_math.rs:2336's −1 reading) (`macro-state/delcode_holds_24_bits_red`; #829, W3RB). INTWIDTH #1 (Gemini 2026-11-25, CONFIRMED,
    Rust-only) adds shapes: a char > 255 aliases a lower slot, `-1` reads back as 65535, `\delimiter\number\delcode`
    prints Φ…Ψ; 1
  - XYOPT — `\xyoption{<feature>}` without `[all]` loses one space mid-paragraph through `load_tex_definitions`; Perl keeps both (pre-existing; #843); 1
  - MODCV — moderncv 2.x: our binding, like Perl's, predates it — `\name` and `\makecvtitle` raise Errors, `\cventry` drops two fields (`class-bindings/moderncv_name_style_makecvtitle_red`; #844); 3
  - ARRF — arrow heads (#838, OD #81): an uncomposed head pair or a `\let` alias of a head reads as a list (`arrowfill/arrowfill_uncomposed_pair_red`); mathtools' `\xLongleftarrow` undefined (`arrowfill/mathtools_xLongleftarrow_red`, pre-existing); 1
  - ARCR — colortbl and rules (#835, OD #1025): a `!{}` rule's own colour (`alignment-bindings/arrayrulecolor_bang_rule_own_colour_red`); the error boundary of colortbl's `\noalign` test (`arrayrulecolor_mid_cell_misplaced_noalign_red`, `arrayrulecolor_noalign_and_before_multicolumn_red`); `\hhline` colour codes; the kernel template reader drops a `|` directly before `!{…}` (Perl too; `tabular_bar_then_bang_rule_between_columns_red`); a math array's `\multicolumn` `tex=` loses the template (`math_array_multicolumn_tex_keeps_the_template_red`); 3
  - RAWREF — a `\label` after a step whose algorithm2e line was dropped or never built lands on `<document>`, where pdflatex prints the line number (synthetic, `scratch_RAWREF/p/p1,p5,p7.tex`); settled: `raw_refstep_objects` (document.rs:235, :7867-7869, :8010) is no dangling path (revI790A's suspicion) — the stored `Node` is libxml's cached wrapper, nulled by `free_subtree`, and `document.rs:7871` filters a null parent (an LD_PRELOAD quarantine shim over 48 algorithm2e papers: 351 listingline frees, 0 moves); 1
  - LOSTA — revLOSTF: `classify/title_code.rs:252` `reads_arguments` decides by syntax — `\let\address\textsuperscript` stays undefined in the rehearsal though it reads an argument (probe `scratch_revLOSTF/p3/astlet.tex`; rule: run a definition whose definee's MEANING reads arguments, `get_num_args() > 0`); symbol-only `\thanks` notes (`\thanks{$\star$}`, `\thanks{\dag}`) dropped by `note_key`'s `retain` (`emitter/plan.rs:45`; counted; apply `symbol_key` in the note arm; probe `p2/symnote.tex`); two rehearsals without `author_markup_begin` (`separators.rs:211` `read_argument`, `emitter/emit.rs:2787` `rehearsed_with_expansions`) — prefix or justify; the astlb repro's two residuals (notes on the last name; the notes' italic lost) lack a `_red.tex`, and its `holds` should pin the notes' owner; 3
  - AMSRO — amsppt's `\roster` (#868; AMSPPTb, revAMSPPT): an unbraced nested roster gives the outer `\item` back where amsppt.sty:744 makes it plain's (invented "(3)"; `amstex/amsppt_unbraced_nested_roster_outer_item_red.tex`; synthetic); a `\runinitem` first item loses its run-in layout (no word lost); the `\endroster` inside an unclosed `{…}` (math0309287) is fixed (#963, OD #1117, guard `perfect_kernel_batchregfixp::endroster_ends_the_roster_inside_a_group`); 3
  - LOADFR — loader frames (revQAB3b): a non-Fatal `Err` mid-load (`load_tex_definitions(...)?`, `hook?` in content.rs) skips `pop_load_frame`, then `load_class` falls back to OmniBus with the frame still pushed (suspected; a drop-guard or a single exit); a package loaded ONLY in a subfiles sub-preamble loses its begin-document setup (synthetic: `\usepackage{cleveref,glossaries}` + `\newacronym` there); 1
  - NOTRES — NOTCANCEL's residuals (revNOTCANCEL; no witness, pre-existing): `Document::insert_math_token` returns the parent (`document.rs:3046`) where Perl returns the new token (`Document.pm:673`) — a `\not` carrying an `xml:id` would hand it to the cancel `XMApp` (census the 3 callers that use the value first: `tbox.rs:253`, `document.rs:1728/:1734`, `math_common.rs:686`); `sideset_wrap` (`amsmath_sty.rs:3777-3784`, Perl `appendChild`) and `base_xmath.rs:1783` (its comment says "Move" but it calls `append_tree`) copy where Perl moves; Replace never frees the node it discards (WISDOM #79 — a future free must skip nodes the callback re-attached, `rewrite.rs:694`); math RED rows (parked): `\not\left(x\right)` unparsed (Perl `delimited-(̸)`), `\overset{\not}{…}` / `\mathop{\not}…` leave `／` unrewritten, `\not\xhookrightarrow` keeps `font="italic"`; 3
  - ACHEMSO-ORDER — a waiting author's held pieces (alternative affiliation, e-mail) come before an affiliation given after the next `\author`, where the PDF lists the affiliation first (`\author{A}\altaffiliation{P}\author{B}\affiliation{X}`, revACHEMSO q14); order only; 1
  - ACHEMSO-BODY — achemso's `\author`/`\email`/`\phone`/`\fax` are `\@onlypreamble` (:346, :521, :530, :539): in the body pdflatex errs and prints the text inline; the binding attaches them at once with no Error, a phone even without an e-mail (revACHEMSOb h3/h6); whether to raise TeX's Error is a policy call; 1
  - SHIPOUTNOTE — (HYPERPROV final review; synthetic shapes, no corpus witness — 2309.06611, 2403.15239, 2302.00362 and
    1612.04433 are right) the page-notice note of `ar5iv_sty.rs` `\lx@arxiv@shipout@note`: (a) material with a picture
    (a tikz node) is noted from `ink_text_printed`'s font-encoded strings, so `\url{…_…~…}`, `\_`, `\textbackslash`, math
    and fonts garble — take the text nodes' digested boxes and route on the digested result, not the source test; (b) a
    picture-mode `\put(20,-30){\makebox(0,0)[l]{…}}` still notes a positioned picture; (c) material registered in the
    preamble is digested there, so `\AtBeginShipoutFirst{\parbox{12cm}{Pre one\par Pre two}}` reads "Pre onePre two"
    with a false `frontmatter:unclaimed` Warning (base glues too; registered in the body it is right) — digest at
    registration only to test for text, emit the tokens; (d) a notice behind a macro of the paper's inside the hook's
    argument (`\AtBeginShipout{\mynotice}`) is not seen by the token scan. Post-release; 3
  - ROBUSTIFYERR — (CLUSTFIX3 review, OD #1092; synthetic, no corpus witness) etoolbox's `\robustify` is a no-op where
    TeX errs "not a macro" on a LaTeX `\mathchardef` that is a `DefMath` here (`\usepackage{etoolbox}\robustify{\alpha}`;
    `\infty`, `\sum`/`\int` without amsmath) and on e-TeX's/pdfTeX's own primitives (`\middle`): pdflatex 1 Error, base 1,
    now 0, output unchanged. Fix: add pdflatex's e-TeX/pdfTeX primitive names to `LATEX_PRIMITIVE_COMMANDS`
    (`etoolbox_sty.rs`, `\meaning`-checked), and keep the Error for a `DefMath` whose LaTeX meaning is `\mathchar"…`
    (fontmath.ltx), unless amsmath made it a macro. Post-release; 1
  - TITLESECRED — (TITLESEC reviews, OD #1143) titlesec's hold on `\section`…`\subparagraph` misses: `\titleclass` after a
    release does not take the command back (titlesec.sty:133-139); a release inside a group outlives it (the lock's
    `\lx@dropped@<sec>` is global); only a body naming `\@startsection` releases (`\renewcommand\section{\mysection}`, a
    `\let` restore and `\secdef` bodies do not; pre-existing); a hand-written redefinition copying the claimed body with
    tokens added still counts as titlesec's (needs provenance from the lock, not token containment); and sectsty under a
    class whose binding loads article (amsart, acmart, IEEEtran, mnras, revtex4-2, emulateapj, asmejour, subfiles)
    releases where pdflatex errs. RED repros in sectioning-frontmatter/ (`titlesec_*_red`, `sectsty_under_amsart_keeps_titlesec_red`); 3
  - HYPERFAMILY — (INTERIM397g, OD #588) one family member's first use defines its whole family (`\phantomsection` then
    `\providecommand\nameref` is skipped) — arXiv's hyperref build behaves the same; the `ar5iv_sty.rs` comment "what
    pdflatex without the package sees" overclaims for the family's other names. Doc fix. With the provided anchor layer
    (#923, F1ANCHORTEST), a runtime `\@ifundefined{Hy@raisedlink}` / `\ifdefined\hyper@anchor` takes the hyperref branch
    in a paper without hyperref (suftesi.cls:3021 then skips its `\let\texorpdfstring\@firstoftwo`;
    interfaces-hypbmsec.sty:79), as arXiv's hyperref build does; 0 arXiv papers use them; a package testing
    `\@ifundefined{hyper@anchor}` in a later `\AtBeginDocument` hook sees hyperref loaded (no witness); 1
  - AAS5-HYPERREF — (LOOPFIX review, OD #1100) under AASTeX 5.x aas_support leaves hyperref marked loaded (as Perl), so
    after `aastex_cls.rs` restores the kernel's `\refstepcounter` a document's own `\usepackage{hyperref}` does not
    re-install the dispatch, and cleveref's hyperref branch (cleveref.sty:2044-2054) sets no `\cref@currentlabel`:
    `\documentclass{aastex}` + algorithm/algpseudocode + cleveref, `\label` in an `algorithmic` line, `\cref` → 2
    Errors (`\cref@currentprefix` expands into itself; base 0, pdflatex 0). aastex + cleveref in 0 of 35,942 census papers.
    Fix: `unmark_loaded("hyperref.sty")` for AASTeX 5.x after aas_support (A/B AASTeX 5 papers: `\@ifpackageloaded`
    turns false). Post-release; 3
  - PHYS-PROTECT — (LOOPFIX review) `\svmqty` (physics_sty.rs:1714, physics.sty:108) and `\trace`/`\Trace` (with the
    `\tr`/`\Tr` aliases, physics.sty:353-354/371-372) are `\DeclareDocumentCommand`s still unprotected here; no loop
    found (`\edef\x{\svmqty{a}}` clean). Protect them with the others. Post-release; 1
  - PHYS-COTH — physics.sty:346 redefines `\coth` as its other hyperbolic functions; doing so here moves the
    complex/physics golden (TRIGFUNCTION). Also undefined: `\homework`, `\braces`, `\fbraces`, `\argopen`,
    `\argclose`, `\opbraces`, `\trigbraces`. Post-release; 1
  - FMTOWNEDIT — (LTXOWN review, OD #1142) a paper's copy of a format-owned module that is not byte-identical is
    replaced even when the author edited it on purpose: TL's babel.sty plus `\gdef\mybabelextra{EXTRA}` gives an
    undefined `\mybabelextra` (pdflatex prints EXTRA); a private `xparse.sty` defining `\authorsnote` loses it (2 E).
    Loud (the `shipped_copy` Warning and the Errors), accepted by the ruling; 0 of 207 reach papers. Option: skip only a
    copy older than the tree's (its `\Provides…` date); 1
  - PHANTOMLEX (after PHANTOM; synthetic, no corpus witness) — a dual over an `XMWrap` beside an edge phantom is lexed
    by its whole text, phantom glyph included (`get_token_meaning`, latexml_math_parser/src/data.rs:347), so rules keyed
    on lexeme text fail where base parses: `$\mbox{$|\phantom{x}$} x \mbox{$|\phantom{x}$} = 1$` (base
    `absolute-value@(x) = 1`), `\mathopen{\langle\vphantom{X}} a, b \mathclose{\rangle\vphantom{X}}`,
    `\mathopen{[\vphantom{X}} a, b \mathclose{)\vphantom{X}}`, `\int f \mbox{$d\phantom{x}$}x` — kludged, content
    kept. Fix: lex a phantom dual as its item (as `item_beside_phantoms` does for an XMArg), covering OPEN/CLOSE
    pairing, VERTBAR, XDIFF, EXPECTATION. Also: `op_base_is_mo` descends a single-child `<mrow>` with no phantom (Perl
    never descends an mrow); `\mathop{\vphantom{X}\nabla} f` gains an `lspace` TeX does not set and
    `\mathop{\vphantom{X}f}` renders `<mi>f</mi>⁡` (base `<mo>`); guard row for repro line B
    (`\mbox{$\to\vphantom{X}$}`). Probes `/home/deyan/.claude/jobs/356a0c47/tmp/rv5/`; 3
  - PICKILLGLUE — (PICNEST review, OD #47) in a picture the explicit-space keep (`document.rs` `open_text`) decides by
    the previous sibling, where TeX decides by the next item: a drawing object kills the glue before it (latex.ltx
    `\@killglue` :16785-16805, pstricks.tex:1564 `\pst@killglue`). So `\put(0,40){word\line(1,0){10}\ more}` drops the
    space TeX keeps, and `\rput(1,1){\psline(0,0)(1,1)}\ \rput(2,1){…}` keeps one TeX kills, nesting the next object
    (depth +2 per item, as on the base). Fix: drawing objects remove a trailing whitespace-only text before they insert
    (`\@killglue` in the bindings or at insertion), and the keep goes back to "always text"; RED repros for both
    shapes. No corpus witness in ~2,700 pstricks/tikz papers; 3
  - PDFLASTOBJ — (PDFPRIM review) `\pdflastobj`, `\pdflastannot` and `\pdflastlink` are constant 0 (`pdftex.rs` ~:378):
    `\pdfobj`/`\pdfannot`/`\pdfstartlink` never advance them, so `\immediate\pdfobj{…}\pdfdest struct \pdflastobj\space
    name{d} fit` raises "struct identifier must be positive" where pdflatex is clean. Fix: advance them as `\pdfximage`
    advances `\pdflastximage` (:445). No witness (hypdestopt keeps its own counters); 1
  - FIXFONTTEX — (PDFPRIM review) a font identifier reached through an alias reverts in `tex=` under its target's NFSS
    name: `\DeclareFixedFont{\bigf}{OT1}{cmr}{bx}{n}{14pt}` then `$\bigf w$` gives `tex="\OT1/cmr/bx/n/14w"` (base
    `w`); any `\let` alias of `\the\font` the same. Fix: revert by the invoking token (`sect08.rs` ~:1022-1040). 0 of
    27 sampled `\DeclareFixedFont` papers show it in HTML; 3
  - TEXTFONTUNSET — (PDFPRIM) `read_font_ident` on an unset `\textfont<n>` returns the `\textfont` token where TeX
    uses the null font; 1
  - FONTNAMENFSS — (PDFPRIM, pre-existing) `\fontname` reports the NFSS shape, not the TFM: `\bfseries\itshape`
    gives `OT1/cmr/bx/it/10` (pdflatex `cmbxti10`), no `.fd` size substitution (`cmbx14`, pdflatex `cmbx12 at
    14.4pt`), no " at <size>" for a non-design size (`phvbc7t at 11.0pt`), and no "Font shape … undefined" Warning; 3
  - PDFPRIMUNDEF — (PDFPRIM) `\pdffontexpand` and `\pdfnobuiltintounicode` are undefined (ours and Perl;
    pdftex.web :17651, :35413); LuaTeX's `\pdffeedback version` answers 0 where lualatex gives 140; 1
  - ONLINECITE — (LOOPFIX review) `revtex4_support_sty.rs:430` `\onlinecite` → `\citealp` by user-level name, where
    revtex4-2.cls:7069 `\let\onlinecite\rev@citealp` (the OD #1100 class; no loop witness yet). Post-release; 1
  - CYCLESTOMACHDRAIN (after CYCLEWIN; synthetic) — the stomach's cycle guard has no backlog gate (it passes 0 to
    `CycleGuard::push`), so a finite recursion printing a fixed string per pushed-back item into one long paragraph
    reads as a cycle: `\def\eat#1{\ifx#1\relax\else abcdefghijkl\expandafter\eat\fi}` over a 60,000-token `\edef`
    is `Fatal:Stomach:Recursion` window 12 (base clean at window 10; the 9-letter variant Fatals on base too); pdflatex
    clean. Repro `expansion-primitives/cycle_guard_box_drain_period_twelve.tex`. Fix: pass the gullet's pushback length
    as the stomach guard's backlog so the same span gate applies; 3
  - CYCLEREFILL2 (pre-existing, base fails the same) — at window 2 `span_checks`' one-check floor looks back 256 reads but
    only 200 are certified repetition, so a period-2 list refilled in chunks of 240+ tokens (`\def\x#1{}`, a chunk of
    150 `\x a`, loaded 3,000 times) is a false `Fatal:Timeout:Recursion` window 2 (pdflatex clean; window 3 is clean).
    Fix: `detect` requires `max(w·REPEAT, CHECK_EVERY + 1)` repeated items. Probes
    `/home/deyan/.claude/jobs/356a0c47/tmp/rv2/src/refill2.tex`; 1
  - REGNOARGS — (no witness) `state::lookup_dimension_cs` (a single token), `lookup_register_quiet` and Rhai's
    `LookupRegister` can still run a getter-with-parameters register (`\fontcharwd`) without its arguments, the class
    #917 fixed in the panel-width scan and pgfmath: make the `\fontchar*` getters total (as `\dimexpr`) or have
    `Register::value_of` decline when arguments are missing; 1
  - RESETLIST — counter reset lists (#925, OD #1090): `sync_reset_list_macro` (`counter/dialect.rs:130`) rebuilds
    `\cl@<ctr>` from LaTeXML's stored list after every change, so a document's `\def\cl@X{}` / `\let\cl@chapter\@empty`
    comes back at the next `\@addtoreset`/`\numberwithin`/`\newcounter{…}[X]` and `tex_reset_list` can raise a false
    cycle Error: `\@cons` onto the macro's current body (latex.ltx:10156) and filter it for `\@removefromreset`
    (CLRESYNC; synthetic); `\stepcounter` into a cycle reports both sect08's Error and the cascade's Warning
    (CYCLEONCE); `\lx@stepcounter@within` reads LaTeXML's list, not TeX's macro (STEPWITHIN-CL; not probed); the
    two-counter id cycle (`\numberwithin{section}{equation}` where the class nests equation ids in section's →
    Fatal:Timeout:PushbackLimit; IDCYCLE, loud, no witness): record each counter's id-within and walk it in
    `define_counter_ids`; 3
  - CLUST397T — run 397's next Error tier (CLUST397c; Perl-parity, pdflatex clean): `(LayAureo)` 47; dcolumn in a math
    `array` ~40; emulateapj's `\tabcaption` ~5; bisect `ltx:float` in `inline-block` (1110.3898) and
    `\lx@add@frontmatter@now` (astro-ph0010381); tabularht's `\endinput` after `\@array` (44, HARD); `(newunicodechar)`
    after CJKutf8 (40, HARD, defer); 3
  - CRASH397L — run-397 crashes left to root-cause (the same on every binary compared then; recheck on rc2): stack
    overflows 2209.08800, 2403.08513 (rc=134; since #924 an unbounded argument expansion is a Fatal); recursion
    2411.07443 (listings), 2411.07667 (`\collect@box`); math0012097's endless "Ignoring redefinition of \title" at
    CRclassif3.tex:40 (~3 GB stderr); 2101.07941's wall clock (LOOPFIX). rc2 2026-11-23: 2209.08800, 2403.08513 and math0012097 convert clean (math0012097 because `amsp2.sty` is now skipped as a missing file; recheck if raw loading under `\documentstyle` returns). 2411.07667 fixed on `int/towards_v0_8_1` by LISTINLINE (an inline listing runs `\lst@Init`/`\lst@DeInit`, OD #1139). Left: 2411.07443 (`\@listings@inline` nested 98 deep in the first `lstlisting`, Perl the same, unexplained), 2101.07941 (wall clock; deferred by difficulty, DG 2026-11-23); 3
  - LSTTEXRECON — (LISTINLINE review, pre-existing) an `\lstinline` in math reconstructs `tex=` from binding internals
    (`\mbox{\lstinline{{\lst@@@set@language…\@listingGroup{ltx_lst_identifier}…}}}`) where BM's rule is the author's
    user-level macros; 3
  - PLACEB4 — (PLACEFIX, OD #1099) a block-local reference list (elsarticle CPC program summary in the abstract
    2503.17910; IEEE CSM sidebars 2402.17043, 2502.16577; hep-ex0405026's minipage) stays in place as DG ruled but keeps
    its "`<ltx:bibliography>` isn't allowed" Error: the schema allows a `biblist` only in back matter. Ruled design
    (DG 2026-11-19 "keep in place as a plain list"): a non-sectional list whose items Scan registers as BIBLABEL targets so
    citations still resolve. ~22 so far; 3
  - PLACEG3 — a nomenclature inside an inline frontmatter container (`{IEEEkeywords}` 1606.00288; 2111.12783, 2402.06758)
    keeps its Error: a paragraph is not allowed there; ~8; 3
  - PLACEB1 — piparticle(-final).cls:648-666 `{theaffiliation}` does `\let\institution\bibitem` inside `\blfootnote`, so
    a whole `{thebibliography}` opens in the footnote (1001.2197, 1510.06283; 38 so far; Perl the same,
    latex_constructs.pool.ltxml:4116-4129). General fix: `\bibitem` in a `\list`-based environment other than
    thebibliography acts as `\item[\@biblabel{n}]` + anchor; keep astro-ph9806175, 2103.15495, 0801.2060; 3
  - PLACENEST — (PLACEFIX final review; synthetic, 0 of 86 witnesses) a theorem dissolved into an abstract splices every
    non-`ltx:para` child as it is, so a nested `{proof}`/`{lemma}` or a `{figure}` inside it lands directly in
    `ltx:abstract` (schema-invalid, no Error; base raised "isn't allowed"). Fix in `document.rs`
    `dissolve_theorem_in_abstract`: dissolve nested theorems/proofs first; if a remaining child cannot go in the target
    (`can_contain_qsym`), keep the theorem with base's Error. Probes: an abstract holding a theorem with a proof inside,
    and a theorem with a figure inside (both pdflatex-clean); 3
  - ABS-BLOCKPAR — (ABSFIX re-review; synthetic, no witness changed) a late-closed abstract's first paragraph is not
    ended by a block environment that follows without a blank line (`Short abstract text.` then `\begin{center}…`, or
    `\begin{theorem}` on the next line): the S1 display exemption narrowed `split_after_first_paragraph`'s `internal_par`
    arm to "followed by a vertical skip". Fix: an `internal_par` ends it unless it borders a display. Base is worse on
    all three probes (`\abstract`+center/theorem without a blank line, closed by the bibliography or the end); 3
  - ABS-SHORTFIRST — a short first line (a PACS line) set before the real abstract is kept as the whole abstract and the
    rest moved (`\abstract \noindent PACS: 11.15\par We show…`; no witness among the 33 swallows); 3
  - ABS-BIBFORMS — net A covers `\begin{thebibliography}`/`\bibliography` only: `\printbibliography`, a bare
    `\thebibliography`, `{references}` and a binding-constructor `thebibliography` do not end a command-opened abstract; 1
  - SVMULT-PAREND — modern svmult's `\abstract{` with its `}` lost: TeX errs "Paragraph ended before \@abstract was
    complete"; we give only the open-group Warning; 1
  - ABS-MOVECOUNT — the moved-words Warning glues words across block edges ("Body theorem." then "Text." counts 2); 1
  - BLOCKPLACE-ZONE — (DG 2026-11-17: general, DRY, reusable) the opt-in landed (`\lx@names@note`, OD #1074: a note moves under the names only where its binding declares it, so round 1's three `_zone=footer` declarations are gone); what remains is the engine rule for a names-declared command used inside an author's footnote (`\AUTHORS{Ann\footnote{Univ X \EMAIL{a@x}}}` shape): a note made while a name's note is digested is the footer's, as a contact inside a note already is (`frontmatter_api.rs`, the contact-in-note rule). No witness among the declared classes yet.
  - SPCONF-TYPEDNUM — a typed `2 Univ B` under a `$^2$` name continues the line above (synthetic only; probes `REVSPCONFb/{e,f,h}.tex`).
  - JHEP-ADDRLIST-ORDER — jheppub's `\lx@add@contact@list` order step leaves the unspelled third address of a 3-for-3 list unplaced (probe `\author{Ann Able,}\author{Bob Baker,}\author{Cy Coe}\emailAdd{ann.able@a.org, bb@b.org, zz@a.org}`); found by SPCONF round 1, whose nameless block creators SPCONFb removed.
  - ANDSTORE — `\author{\store}` with a store opening with `\and` (`\newcommand\toc@authors{\and Christian Elbracht}`, no copy) invents the word "and": one creator "and Christian Elbracht", 1 Warning, where pdflatex sets the name alone (an empty first column); main 8b298f9c0d6 the same; RED catalog `sectioning-frontmatter/and_store/direct_store.tex`; fix in the author split: a leading empty name before `\and` is no name; 2
  - MARKORDER — notes' document order differs from the PDF's mark order ("Ann Able † ∗" vs PDF "∗ †"; HTML NEITHER ×5): `elsarticle_fntext_shared_by_two_names_is_their_marked_note`, `author_thanks_shared_by_footnotemark_marks_kept`, `ne2c_mark_after_footnote_glued`, `ne2b_counter_moved_between_notes`, `ne2e_mark_past_spacing_is_its_names` (+ revPRESBXb S2 RED, the WRONGFIX folded ∗ vs PDF text * order swap); engine order; low priority.
  - TABCELLMARK — a prefix mark opening the next tabular cell is carried back across the cell to Ann, silently (FMRG suite).
  - SHGIVEN-TEXTSC — `\textsc` next name (probe s4): a spacing-separated mark is a counted loss instead of carried.
  - AUTHRESET — `article_replaced_author_steps`: ‡ § where pdflatex prints * † (pre-existing wrong glyphs the old model hid).
  - REGCDE-RES — loader residuals: (a) a package reloaded in the body with NEW options gets no reopen of the preamble commands; (b) a body `\DeclareUnicodeAccent` drops "TU" that lualatex prints; (c) RED `loader/loader_is_not_notprerr_red.tex` (Perl's 7 loaders Error-then-load, scrlttr2.cls:2313); amsmath never adds `\DeclareMathOperator` to `\@preamblecmds` (RED `loader/declaremathoperator_is_preamble_only_red.tex`, pdflatex 3 `!`, ours 0); an undefined command used twice gives 1 Error (pdflatex 1 per use); a sub-file with no `\begin{document}` keeps its text with 0 E (pdflatex 1 Error, drops it; census first); an unknown textcomp option is `Info:unexpected` (pdflatex Error).
  - REGABBOLD — a `$` nested in an `\hbox` does not inherit `\bm`/`\boldsymbol`'s bold (`\ext@arrow` inside `\bm` loses `font="bold"`; RED `arrowfill/bold_ext_arrow_in_hbox_red.tex`); NIT: make the `ext_arrow_hmode_fill` guard row whole-element.
  - LEADPANEL — a rule leader between two panels on one line splits the float's panel row (RED `leader_between_panels_one_row_red`; LEAD805 covers the other two REDs).
  - PHANTOM2 — a second `\phantomsubcaption`, or a `\subcaption` after one, in the same group (a minipage, or a figure with one image and (a)(b) labels) re-runs `\caption@subtypehook` where pdflatex runs it once (`\setcaptionsubtype*` sets `\@subcaptype` for the group, caption.sty:305-327; our gate reads `\@captype`, which `\phantomsubcaption` never sets) — fix: a group-local "sub-type entered" flag set by `\phantomsubcaption` and `\lx@subcaption@type`, read by `in_caption_subtype`, the `\phantomsubcaption` gate and `\subcaption` (not `\@captype`, which drives numbering), plus a two-`\phantomsubcaption` row in `subtype_hook_runs_once_per_sub_type` (revWINSUITEb, probes `/home/deyan/.claude/jobs/356a0c47/tmp/revWINSUITEb/p1.tex`, `p6.tex`); matters only for a non-idempotent hook (the first aid's `\let` and zref-clever's property are idempotent); `\subcaptionbox` runs it once where pdflatex runs it twice (its trial pass, caption.sty:465-489 — documented, not mirrored, OXIDIZED_DESIGN_DIVERGENCES #853); 1
  - MHCHEMPROT — REGCHEM's residual (revREGCHEM): a protected user macro hiding a math shift (`\NewDocumentCommand{\nd}{}{$\alpha$-\ce{SiC}}`, `\protected\def\pd{…}`, then `\ce{\nd}`) is not detected by `shifts_math` (`latexml_contrib/src/mhchem_sty.rs:~286` skips protected macros — load-bearing for `\ensuremath`), so it still errs "Missing $ inserted" (pdflatex 0); and the scan does not know the mode, so `$` inside `\ifmmode…\else$…$\fi` or a user `\mbox{$…$}` declines a formula that would have worked (content kept, 0 E); possible fix: follow protected macros taking only `\ifmmode`'s true branch (units are set in math) — probe `/home/deyan/.claude/jobs/356a0c47/tmp/revREGCHEM/p2/p.tex`; no witness yet; 1
  - ICMLCORR — revREGBINDb NIT (synthetic): a document `\gdef`ing `\icmlcorrespondingauthor@text` without calling
    `\icmlcorrespondingauthor` leaves `\lx@icml@corr@built` undefined, so the line becomes a pubnote without pdflatex's
    "Correspondence to: " (icml2026.sty:530; icml_support_sty.rs:167). Fix: an undefined built-macro means the document replaced it.
  - REHEARSEARG — revREGREHEARSEb (synthetic): a run-time-built definition a paper's macro passes as a title-code command's ARGUMENT
    (`\instref{#1}{\expandafter\gdef\csname aff#1\endcsname{#2}}`) reads as inside the command's run and is not the block's; the
    body's `\csname aff1\endcsname` is empty, no diagnostic (RED `sectioning-frontmatter/title_code_command_argument_building_a_definition_red.tex`;
    a kernel run's argument loses the same way on CUT999). Fix: a title-code run drops only the names its own body names.
  - REGTAIL — the v0.8.0 regression check's residue after REGFIX M-U (int/regfix6; run 372 → 399, tier 2): of the 497
    papers clean on run 372, 192 still err (343 before that round, 0 newly erring, 12 Fatal), ~45 classes of 2-9:
    `unexpected:\endgroup` 9 (2110.03080, 1907.13475, 2502.02871), `expected:<dimen>` 7 (xy `\ar` labels, 1504.04186,
    1809.00682), `Fatal:oom:alloc_failed` 6, `unexpected:}`/`&` 6 each (ANONSTR), `expected:Match` 5, `\end{center}` 5,
    `\lx@personname` 5 (1207.5684, 1906.11649), `toccaption` 5 (solarphysics, svjour3; 1510.03282, 1511.05217),
    `TooManyErrors` 5, `\ordinal` 5 (1112.4346), `\@metaauthor` 5 (snapshotmfo.cls), `\email` 4 (cccconf.cls), old pgf
    1.x trees' `expected:conditional` 4 (0704.2264, cond-mat0610559), pgfmath overflow (2007.09944, 2201.10213),
    `\@end@tabular` 4 (0708.3763, 1107.1010), `\omit`/`\noalign` (2601.10115, 2609.04552, 2408.07095), `\relax` in
    `\csname` (2205.09553, 2208.14960); traced: THEAPABCA, NOLTAAUTHOR, IEEEDVI, CREFCAPS, EXPANDCUT, NEURIPSACK,
    TBLRADJ. From REGFIX G-L still: aastex631 2211.00793, 2502.03242's `\the@toks`. Unexplained on rc1 code (T2R):
    jcappub + translations `\spacefactor` (2008.10659, 2008.12721), 2507.12047 (`keyvals.rs:313`), thmtools
    prelude.sty:664 (2110.06326), `\@no@sparg`/`\@citea` (2209.13209), TooManyErrors 1507.00508, 1210.5701 (xy frames,
    Dimension too large), 1310.2513 (extraipa), 1701.06055 (`\RIfM@`), 2406.05605 (`\enquote` in a `.bbl`),
    circuits' wall clock (2506.09686), astro-ph0012068,
    `\rcite` recursion (cond-mat9505034), 2402.13372 (flaky?); Theorems.sty's `\newtheorem{\theoremcommand}` (2) Re-tallied on INT081G (2026-11-23): 190 of 192 still err, E 6321 → 5996, 0 worse; REGTAILA fixed 5 (1907.13475,
    2110.03080, 2203.16677, 2410.11591, 1504.04186), REGTAILC 4 (2105.10205, 2502.09129, 1210.5701, 2503.14426) and
    REGTAILB 7 (2406.05605, 2408.05715, 2309.07288, 1701.06055, 2402.13372, 1310.2513, cond-mat9505034; archive). Traced, open:
    `\endgroup` later in the log (0708.3763, 2012.08010, 1908.02166, 2006.02670, 2307.08421, 2307.12529, 2604.21274),
    `expected:<dimen>` (2410.03560, 2412.03297, 2604.08953, 2609.20331; 1809.00682 is pdflatex-faithful),
    kluwer's typo'd `\bibitem[\protect\citeauthoryear…]` runs natbib's
    `#3(@)(@)\@nil` to EOF (astro-ph0012068, `kluwer_cls.rs:138`), 1507.00508 (`\let\OLDsection*\section*`; HARD),
    2209.13209 (6 GB pushback after nameref redeclares `\Ref`; likely legacy-TL); 5
  - COPYTHENREAL — (COPYROUTE residual, OD #1118) a deferred request of a form `loads_file` misses runs after rule W's
    scope, so the real file is never read: `\AtEndOfPackage{\RequirePackageWithOptions{csquotes}}` in a copy (26 Errors,
    base the same); a true copy followed by the real request leaves the real load a no-op (TeX reads the file over the
    copy). Fix: keep the request pending until `\begin{document}`; 3
  - COPYSMALL — (COPYROUTE residual, OD #1118) an author's file of four names or fewer passes on one shared name, so
    `\ProvidesPackage{tikz-cd}` with `\def\ar{…}`, or a `{tcolorbox}` file renewing `\arraystretch`, is read as the real
    file and the document's later real load is a no-op; 3
  - NOLTAAUTHOR — nolta.cls's `\AUTHOR{…}` stores `\author{Name}{1}\orcid{…}` in `\@Mauthor`, run only by its
    `\@maketitle`; the lock refuses the class's `\author`, so there is no creator (the name a block note) and
    `Error:undefined:\current@author` (2404.06828, 2509.07487, 2604.20333, 2605.00366, 2605.05978; census reach ≈0): a
    thin binding, or a kernel rule that a raw class's stored `\author` calls printed by its title page are the creators;
    3
  - EXPANDCUT — a parameterless macro reading through its expansion in an author (`\def\contact{\xdef\@thefnmark{}
    \@footnotetext}`) is cut from its argument: 1909.05295 (svmult) a creator "and \@undefined", Stefan Bruckner lost,
    "footnote" in an e-mail; `\author{\textsc{Ann Bee}\contact{a@b.org}, Univ A\\ \textsc{Cy Dee}\contact{c@d.org}, Univ
    B}` merges Cy Dee into Ann Bee's affiliation; rims02e's `\affil` reroute swallows `\lx@author@markup@run`'s
    `\let\mail\@undefined` tail (math0408269). A whatsit made by a macro's expansion should cover the invoking token; 3
  - IEEEDVI — IEEEtran papers with `\includegraphics{x.eps}` throughout: the figure cue (converter.rs ~305
    `figure_output_cue`) picks DVI, `\ifCLASSINFOpdf` is false (ieeetran_cls.rs ~170) and graphicx never loads (3
    papers; 2112.07932 ships `-eps-converted-to.pdf`, so arXiv ran pdflatex): the cue should not pick DVI where the DVI
    branch cannot load graphicx; 3
  - THEAPABCA — theapa.sty read raw: after a `\cite`, `\BCAY` calls `\BCA`, defined only inside `\@@cite`, as our
    `\bibitem` digests the `\protect\BCAY{…}` label theapa's `\@biblabel` gobbles (:429) (1109.6618, 1111.0053,
    1405.3608, 2111.08066, 2304.01300; Perl 8 errors, it never reads theapa.sty): honour a raw `\@biblabel`, or bind
    theapa; 3
  - CREFCAPS — (ruling pending; (a), build the capitalised name lazily on its first use, recommended) a cleveref
    capitalised name with a leading command (`\MakeUppercase{\color{black} Section}`) is typeset at every `\label`
    (cleveref_sty.rs ~913-1000, CREF_TAG_ROLES): "Can't find color BLACK" though `\Cref` is unused (2402.05459,
    2404.01185, 2605.26927, 2606.11815; 2212.00537 errs in pdflatex too); 3
  - NEURIPSACK — `\usepackage[preprint]{neurips_2025}` with `\begin{ack}Thanks $$x_i=1$$\end{ack}` (or `\ack` as a
    command) gives `_` in math mode, the TL file too and Perl's same `{ack}` (2510.08734): model environ's
    `\NewEnviron{ack}`; 3
  - TBLRADJ — a `tblr` inside `adjustbox` inside `table*` (2502.02871), untraced; 3
  - GATHERCAP — `\lx@ams@gather@close` scans 8 tokens, not items: `…\black@\totwidth@ \noalign{\vskip 4pt}\egroup $$`
    overflows into the raw cascade (6 Errors), `\iftag@\fi` before the `\egroup` gives a false Warning (synthetic; every
    TL and corpus form is `\math@cr \black@\totwidth@ \egroup`): scan balanced items, skip `\black@<arg>`, empty
    conditionals and `\relax` silently; 1
  - XARGSSPACE — an xargs later optional argument peeks past spaces and drops them where no `[` follows, which
    `\xargs@ifnextchar` (xargs.sty:64-87) puts back: `\ry{i} \ry{i}[j]` prints "SieSij", pdflatex "Sie Sij"; Perl the
    same (KNOWN_PERL_ERRORS #1304); 1
  - CAPTIONV1 — caption3 v1 residues (REGFIXR2 review): a consumed v1 option list applies `labelformat` only
    (`[subfloat]{labelformat={simple},labelsep=colon}` prints "a", pdflatex "a:"); a caption-less `\subfloat` under
    `[subfigure]` warns "Unused" (pdflatex not), and raw v1.8h errs on `\phantomcaption`; `\captionsetup[figure]`
    under raw floatrow warns "Unused"; a document's `\usepackage{caption3}` reaches the binding while a raw package's
    `\RequirePackage{caption3}` reads the shipped copy; 3
  - KERNELSMALL — kernel residuals found by this round's reviews: a `\let` copy of `\underline` prints as the primitive
    under `\meaning` (ULMEANING; pdflatex `macro:->\protect \underline  `; tex_debugging.rs ~286-290 keys on the token's
    name, so `\patchcmd\oldul` or `\ifdefmacro\oldul` misread a saved copy; fix: give `\@@underline`, latex_base.rs
    ~536, its own primitive stand-in); `\ignorespacesafterend` leaves the space after `\end{…}` (IGNSPACESEND:
    "AFTERTEXT  next."); a named-equation hack (`\eqname` through a patched `\maketag@@@`) prints nothing (EQNAMEHACK;
    pdflatex "NAMEA"); `\[…\]\noindent X` marks the whole paragraph `ltx_noindent`; a `\let` (or `\LetLtxMacro`) copy of
    an optional-argument command keeps the old body after the original's redefinition, where LaTeX's calls `\\cs` by
    name (TWOOPTLET: `\LetLtxMacro\twc\tw` then `\renewcommandtwoopt\tw`, pdflatex "Rpqr", ours "Tpqr"); the
    `if_next('[')` peeks other than `\@testopt`'s (`pdftex.rs:399`, `base_parameter_types.rs:2027` optional_key_vals) do
    not skip spaces (Perl-shaped, no witness); `\@testopt`, like `\@ifnextchar`'s closure, may not halve `##` in the
    chosen branch (suspected); 3
  - INPUTNAMEBYTE — `\input{x<0x81>y}` under latin1 is a missing file whose name holds `\protect\GenericError…`
    (pdflatex: a garbled name, then Fatal); revtex4-1 without hyperref defines `\ifpdf`/`\ifxetex` (undefined in TeX);
    emulateapj answers `\@ifpackageloaded{iftex}`/`{hyperref}` yes (pdflatex no); an undefined byte inside `{verbatim}`
    drops its Error (in one shape the line too); inputenc's Errors come twice in `\caption`, natbib's `\bibitem[…]`,
    `\texorpdfstring` and `\nameref`, where pdflatex's first run gives one; 1
  - TIKZCDARROWS — a file declaring `{tikz-cd}` that loads `\usetikzlibrary{arrows}` gives 14 Errors at
    pgflibraryarrows.code.tex:36 ("I did not find the pgf library 's'"), base too, 0 under its own name; pdflatex
    clean; 1
  - SAFEACTIVES — the URL readers (`\url`'s, `HyperVerbatim`) do not set babel's safe actives, as `\hyper@normalise`
    does (`\Hy@safe@activestrue`, hyperref.sty:4627, defined :362): ngerman `A\footnote{\href{http://x.org/"o}{s}}`
    gives href `http://x.org/ö`, pdflatex `http://x.org/"o`; main the same (int/regfix7 review); 1
  - C1UTF8 — after `[utf8]{inputenc}` a C1 control code point (U+0080-U+009F) read fresh is catcode other
    (`utf8_def.rs:243`) and prints silently, where pdflatex errs "Unicode character (U+0085) not set up for use with
    LaTeX"; main the same; 1
  - INPENCLOOP — `set_input_encoding`'s loop, like Perl's (inputenc.sty.ltxml:48-51), skips 0x1F and 0x7F, which
    inputenc.sty:83-84 covers (`\^^N\^^_`, `\^^?\^^ff`); 1
  - EQNUMTD (post-0.8.0; Perl too, KNOWN_PERL_ERRORS candidate) — `LaTeXML-block-xhtml.xsl:551/579` `eqnumtd` tests
    `ancestor-or-self::ltx:tr`, which also matches an enclosing tabular's row, so an `equation`+`aligned` in a tabular
    cell prints its tag on every row. Minimal: `\begin{tabular}{|p{3in}|}{\begin{equation}\begin{aligned} x &= 1\\
    y &= 2\end{aligned}\end{equation}}\\\end{tabular}`; test only for a row inside the equation's own MathFork; 1
  - TITLEPAGEUNWRAPIDS (synthetic) — unwrapping a stranded `{titlepage}` around a float (`field_ink.rs:22
    demote_stranded_titlepage`) drops the titlepage's own `xml:id`/`labels` (the plain demotion keeps them on the
    renamed paragraph); no construct seen that labels a titlepage (`\label` inside attaches to the document); 1
  - ANONSTR — the "at Anonymous String" `&`/`}` error classes are separate mechanisms (REGFIXK sample): 2512.14155 (a
    stray `&`), 2107.00457 (an extra alignment tab), 2003.01824 (`Extra }` with arydshln), 2207.06210 (unclassified):
    one root-causer each; 3
  - FMREPLAY — the class-setter replay's residuals (REGFIXG, REGFIXI): a replay drops its local change to an existing
    switch (REPLAYSWITCH; bullsrsl `\setboolean{BSRSL@affilsempty}`: "Affiliation is empty", 19 papers, 2309.01084,
    2411.00793; clashes with `perfect_kernel_batch56::class_maketitle_replay_that_errors_is_dropped`); a replay's
    records held in toks registers survive the deposit (REPLAYTOKS: a class helper appending authors to a toks its
    `\maketitle` prints prints each twice; synthetic,
    `\def\@author[#1]#2{\global\auth@toks\expandafter{\the\auth@toks\par AUTH: #2}}` + `\the\auth@toks` in `\maketitle`;
    fix in `note_replay_records`, sect05.rs ~3345); bullsrsl surnames read as affiliations (BSRSLSURNAME;
    `\lx@author@trailing@mand`, author_macros.rs ~148, since Oct 8) and its "Corresponding author(s):" label not kept
    (BSRSLCORRESP; now counted: 38 papers' unclaimed Warning; option: carry `\BSRSL@CorrespIntro` into the frontmatter);
    a delimited-argument kept call makes the block decline (RED
    `sectioning-frontmatter/author_delimited_argument_kept_call_declines_red`); a note-printing call's globals read
    outside the block are still lost (no witness); with authblk a `\footnotemark` inside a robust command drops its mark
    (counted); contacts on a `\\` line land in a nameless creator; a counted `\orcidlink` wrapper gives Count 1
    (pdflatex 2); JATS/TEI `[@class='ltx_phantom']` misses `ltx_align_left ltx_phantom`; 5
  - TITLEFIGTRUEFLOAT (after TITLEFIG) — one true float in the cover region blocks the whole move: `\maketitle`, two
    `\captionof{figure}` teasers in a `center`, a `\begin{figure}[t]…\end{figure}`, then `{abstract}` — the PDF prints
    both teasers before the abstract, ours leaves them after it (as base). Options: let a true float pass after the
    abstract as notes do, or keep the fail-safe (`cover_order.rs` refuses a region holding a float); 3
  - TITLEFIGHNOFLOAT (synthetic) — `is_in_place_float` reads `[H]` from `placement` even when float.sty is not loaded,
    where LaTeX errors and the float floats; 1
  - TITLEFIGSTREAM (suspected) — the acmart and vgtc close hooks look only at root children
    (`deposit_in_place_floats`, `place_teasers_after_head`); under `--streaming` with a small fragment budget a spilled
    deposit would leave the teaser after the abstract (base order). Confirm with a small-budget `--streaming` run; 1
  - FILENAMESPACE (after REGTAILB) — a leading space before a braced file name is not skipped, where `\@firstofone`
    skips it: `\IfFileExists{ {a}}{yes}{no}` gives "no" (pdflatex "yes"); `gullet.rs` `expand_as_file_name` and
    `sect11.rs` `strip_leading_group` (which also duplicates the token helper for `\includeonly`); 1
  - FNUMINDIRECT (after REGTAILB) — a list label that reaches its own `\fnum@<ctr>` indirectly still loops:
    `\item[\csname fnum@fc\endcsname]` → `Fatal:Timeout:Recursion` (`dialect.rs` `label_calls_fnum` matches tokens
    only); pre-existing; 1
  - SIXFONTLIST (after REGTAILA) — siunitx's `detect-*` font is not yet carried onto `\SIlist`/`\SIrange`/`\ang`
    numbers (pdflatex bolds every one under detect-weight), `\numlist`/`\numrange` (pdflatex bolds 1 and 3 of
    `\numlist{1;2;3}`, 4 of `\numrange{4}{5}`), or the unit of `\SI`/`\qty` under `mode=text` (pdflatex bolds the "m");
    `\boldmath \num{6.5}` is bold where `reset-math-version=true` sets it medium (siunitx.sty:5208-5226); the ± and
    parentheses of an uncertainty are not bolded (pdflatex SFBX/CMBSY). Probes `tmp/rev2REGTAILA/p/d2.tex`, `d3.tex`; 3
  - SIXFAMILYRESET (after REGTAILA, markup only) — with `detect-shape` but not `detect-family`, `\lx@six@font`'s shape
    branch keeps the text family that siunitx resets to `\familydefault` in text mode (siunitx.sty:5355-5370):
    `{\sffamily\itshape\num[detect-shape,mode=text]{8.5}}` is `sansserif italic` where pdflatex prints CMTI10. Take the
    family only under `'f'`; 1
  - ENDITEMIZEREDEF — `\def\enditemize{\par After list.}` silently drops "After list." (the locked itemize ignores the
    redefinition, Info only), against counting every lost word; 1
  - BOXSIZEPARBOX (after REGTAILC, pre-existing) — `\setbox0\hbox{\footnotesize\parbox{1cm}{aaa bbb ccc ddd eee}}\the\ht0`
    is 14.27777pt vs pdflatex 13.22223pt (interline glue at 8pt); also a space's width under `\Huge`, `\Big(` box size; 3
  - BACKMATTERRAW (after BACKMATTERPART) — memoir (its runtime binding reads memoir.cls raw) and scrbook keep their own
    `\backmatter`, so their back matter still nests inside the last `ltx:part`; a document's `\renewcommand\backmatter`
    loses the close too (as Perl). Trigger: `\documentclass{scrbook}\begin{document}\part{A}\chapter{x}\backmatter
    \chapter{B}\end{document}`. Fix: call `\lx@begin@backmatter` from those classes' `\backmatter`; 3
  - AMSBACKNAME (cosmetic) — amsbook's `\backmatter` does `\let\chaptername\relax`, so pdflatex prints "B Afterword"
    for an appendix after it; we print "Appendix B Afterword". Trigger: repro
    `sectioning-frontmatter/amsbook_backmatter_closes_the_appendix_and_part.tex`; 1
  - SCANINDEXXML (after 0.8.0, multi-document sites) — Rust's Scan parent fallback (`scan.rs` after `doc.parent_id`)
    lacks Perl's second guess, an `index.xml` in a containing directory (`Scan.pm:117-124`). Split's pages no longer
    use the fallback; 1
  - STREAMNODISPLAY (after 0.8.0, suspected) — `stream_split.rs` has no equivalent of `split.rs::get_pages`'s
    `ltx_nodisplay` filter, so a hidden bibliography (a `\fullcite`-only one under `--splitat`) may become a page only
    on the streamed path. Probe first; 1
  - LUAWINCOLON (after 0.8.0, Windows, suspected) — `lua_bridge.rs:200` `out_name_ok`: a `:` that is not a drive letter
    (`a:b`, an alternate data stream) gets lualatex's refusal values and a Warning; kpathsea's paranoid check rejects a
    `:` only in position 2, so lualatex probably writes it and the policy makes it an Error. Empty/NUL/non-string names
    get nil + Warning where lualatex returns fopen's 3 values or "bad argument". Needs a Windows lualatex; 1
  - LINENOAUTHORS (after 0.8.0, content) — lineno/lineno's two extra `\author` lines ("Uwe Lück", "Karl Wette") become
    affiliation contacts; pre-existing, separate from MANFM's `~` fix. Perl itself is not clean on this manual under the
    sweep preload (23 missing-argument Errors near the end); 1
  - AMSWRAPSETAUTHORS (after 0.8.0, synthetic only) — MANFM keeps the `\@setauthors` an amsart-based class ends its load
    with (`class/after`) and runs only what the document appends. A wrapper class that `\LoadClass{amsart}` and then
    appends to (`\g@addto@macro\@setauthors{\par Communicated by …}`) or redefines `\@setauthors` with extra text has
    that text dropped (pdflatex prints it; base kept it as a note). No TL or arXiv class seen (12 real-class papers: 8
    identical, 4 only lose a junk nameless creator). Route: keep the definition amsart's binding ends with (`\@empty`)
    and skip only a derived class's setter that fails a sanity check; 1
  - LUAFONTIDS (after 0.8.0) — the texlua `font` mirror holds one font (the current one, id 1): `font.id(<real font>)`
    answers -1 with a Warning where lualatex gives its id (e.g. 2 for `\tenln`); hand out mirror ids per named font; 1
  - LUALANG (after 0.8.0) — `lang.new()` succeeds in lualatex (polyglossia.lua:99, luababel.def:274) but is an engine-gap
    Warning plus a Lua error under texlua; a `lang` mirror is its own batch; 1
  - LUASPRINTSPACE (after 0.8.0) — a trailing space in what a chunk `tex.sprint`s is dropped: `\luaexec{tex.sprint("[c] ")}d`
    gives `[c]d`, lualatex `[c] d` (also `luacode*`); the bridge's tokenizing of chunk output; 1
  - NBSPREVERT (after 0.8.0) — Rust's `\lx@nobreakspace` box (`general.rs:118`) reverts to itself; Perl's
    (Base_Utility.pool.ltxml:53-55) reverts to `\nobreakspace`. Since MANFM2 the 28 default frontmatter labels end in it;
    no output showed the difference. Align with Perl; 1
  - FMTIESTILDE (after 0.8.0, suspected) — ties LaTeXML adds still use the document's `~`: `frontmatter_api/email_lines.rs:103`
    (`named_email_calls` pushes `T_ACTIVE('~')`), `defs/frontmatter_fields.rs` `\lx@row@email@named` (`name={#2~}`),
    `frontmatter_stores.rs:4219` (the off-table label); under `\def~{\verb~}` they would meet OD #1136's Error; not
    triggered by `Emails: …` lines. Spell them with `\lx@nobreakspace` with a RED repro first; 1
  - PDFEXTFONTID (after 0.8.0) — `\pdfextension fontattr`/`includechars` (and `\pdfincludechars`) read their font as one
    token, so a `\textfont1`-style font identifier leaks the "1"; add a font-identifier reader; 1
  - PDFCOLORSTACKNOACT (after 0.8.0) — `\pdfcolorstack<n>` with no action: TeX errors, we read nothing and say nothing;
    add the Error; 1
  - PDFFONTFEEDBACK (after 0.8.0) — pdfTeX's `\pdffontname`/`\pdffontobjnum`/`\pdffontsize` expand to nothing (as Perl,
    pdfTeX.pool:97-100), so `\count255=\pdffontobjnum\font\relax` gives "Missing number" where pdflatex gives a number;
    one shared answer for both these and `\pdffeedback fontname`/`fontobjnum`/`fontsize`, which answer 0 (lualatex
    typesets "10.0pt" for `fontsize`; a bare 0 is no dimension): an integer for name and objnum, the font's size with
    `pt` for size (pdftex.web:11359-11369). Reach: microtype-pdftex.def:254 if ever raw-loaded, pdfbase.sty:520-526; 1
  - PDFOPTSPACE (after 0.8.0, advisor 2026-11-22) — one optional-space helper (`skip_one_space(true)`, gullet.rs) at
    pdftex.web's four sites: obj `reserveobjnum` (:34321), annot `reserveobjnum` (:34900), dest, `newwindow`/`nonewwindow`
    (`A\pdf_object_new:n{foo}B` gives "A B", pdflatex "AB"; ordinary documents make 0 such calls, `\DocumentMetadata`
    tagging 23 in pdflatex, 0 here as our tagpdf stubs that path); `read_action_spec` (pdftex.rs) to `scan_action`
    (:34774: `struct`, page/name/num order) and `\pdfdest struct <n>`; `\pdffontattr` on the pdfTeX path (:35375); "Color
    stack action is missing" as an Error; `lastximagepages` before `lastximage`; a non-font operand of `read_font_ident`
    ("Missing font identifier" + back up, as TeX); 1
  - EMPTYBOXSPACE — (issue 991 review, pre-existing) an author space after an empty box that stays an empty `<m:mrow>`
    is lost: `$\mbox{}\,x$` (beside an invisible times) has no space where TeX sets the thin space, as the spacing walk
    skips an empty row's `_rpadding`. Beside a visible operator (`$\mbox{}\,=b$`, `$\mbox{}\,-b$`) the box is the
    absent operand's placeholder and keeps it (OD #1150). Also `$\mbox{}\,\mbox{}=b$` parses as an invisible times and
    the `\,` is lost; 1
  - BOXTEXTSPACELIKE — (issue 991 review, pre-existing) a box holding only space is a whitespace `<m:mtext>` and
    loses its width: `$\mbox{\hspace{1cm}}=b$` is `<m:mtext>&#xA0;</m:mtext>`, the 1cm gone. The operator after it
    keeps its spacing (MathJax 3.2.2 gives `mtext` the ordinary-atom class, `MmlNodes/mtext.js:35`; Chrome 158 sets
    `=`/`+` infix after a whitespace `mtext`). `$\mbox{ }=b$` stays an NBSP `mtext` while `$\mbox{ }+b$` becomes the
    placeholder; 1
  - TEXTPHANTOM — (issue 991 review, pre-existing) `$\text{\phantom{XX}}=b$` is `<m:mtext class="ltx_phantom">XX</m:mtext>`,
    which renders a visible "XX" where TeX leaves blank space; 1
  - CONTSIGNEDGE (after issue 991, synthetic) — the continuation-sign reading's edges: `&\left.{} + j\right)`,
    `&\left.\right. + c` and `&\left.\big. + c\right)` read unary where TeX sets the sign binary (`{}` leaves no node; an
    Inner atom; a `\big.` box after the fence); a left cell ending in prose counts as an operand (`f &= x &\text{if }&
    -1<x<0` reads `(absent − 1) < x < 0`); `B ={} & -C` may count as an empty left cell (suspected); the
    expectation-respelling retry may still see `CELLADDOP` (suspected); a leading `\quad`/`\hskip` before `\left.+`
    loses its lspace in the unary reading (2505.11002, 2012.13407; the row-edge rule); `&\left. + c \right|` warns
    (pre-existing). OD #1151 "Not covered"; 3
  - EMPTYBOXSTYLE — (issue 991 review, pre-existing) `$\textcolor{red}{\mbox{}}=b$` loses its colour `mstyle`;
    `$\colorbox{red}{}=b$` loses TeX's 6pt red box and its `tex=` reads `\hbox{\pagecolor{red}}`; `$\fbox{}=b$` is 0 wide
    where TeX's is 2\fboxsep + 2\fboxrule ≈ 6.8pt; 3
  - CELLLONESIGN — an amsmath cell that is only a sign (`&+ \\`) takes no grammar parse and stays a bare `<mo>`; 1
  - FIGLOCK (after REGTAILD, synthetic) — our kernel `{figure}`/`{table}` are not locked as Perl's are
    (latex_constructs.pool.ltxml:3409), so a binding macro that expands to `\figure[…]` runs a raw class's own
    `\figure` (`\def\figure{\@float{figure}\setbox0\vbox\bgroup}`); only sidecap did (fixed on `pk/REGTAILD`). Options:
    lock `\figure`/`\table` as Perl does (A/B over raw classes) or audit the bindings; 1
  - FNT1 — 2108.12618: each author split into two creators (nowfnt). Reach (census 2026-11-24): 1 of the 30k sample ≈ 93 papers; 3
  - PSTL — pstool's `\pstool{f}{\psfrag…}` fragment uncounted (1608.03745; `reversion/psfrag_pstool_fragment_uncounted_red`). Reach (census 2026-11-24): 2 of the 30k sample ≈ 186 papers; 1
  - P5BN — a framed collaboration call with its own note (`\fnref`/`\footnote`) a nameless creator or the collaboration's creator holding it (`elsarticle_author_framed_collaboration_with_a_note_red`; 2301.12970). Reach (census 2026-11-24): 2 of the 30k sample ≈ 186 papers; 1
  - AMSTEX-ENUM — (LOOPFIX) inlined-AmS-TeX papers past the `\sum@` loop hit `malformed:ltx:enumerate` (#PCDATA, Math
    and text in an `enumerate`) from amsppt `\roster`/`\item` (math0612243, math0112238, math0303294, math0404149,
    math0404221, math0404272, math0405116, math0406482 TooManyErrors; 0705.4126, math0112286, math0211398, math0405158,
    math0404207). Reach (census 2026-11-24): 1 of the 30k sample ≈ 93 papers; 3
  - L3EXP — (LTXOWN) a 2015 paper shipping l3experimental (l3regex.sty, l3str.sty, l3str-convert.sty,
    l3tl-analysis.sty) is Fatal (Timeout:Recursion or TooManyErrors in l3str*.sty): 1412.8552, 1503.01348, 1510.04725,
    1510.01841, 1511.02193 (all better after LTXOWN, e.g. 1412.8552 1001 → 69 Errors). Not format-held, so outside
    DG's closed set (OD #1142); the paper-era legacy dump is the tool. Reach (census 2026-11-24): 0 of the 30k sample ≈ 0 papers; 3
  - REALBOXBG — (LISTINLINE review) realboxes' `\Colorbox`/`\Fcolorbox` keep their text but lose the background colour:
    collectbox digests the box before `\colorbox` (bound as `\color@box`, color.sty:158) sets the colour, so
    `\Colorbox{yellow}{x}` gives a plain `<text>` (2411.07667's ~390 `\myinline` boxes); a constructor around the
    collected box would carry `backgroundcolor`. Silent (no Warning); guard comments name it. Reach (census 2026-11-24): 0 of the 30k sample ≈ 0 papers; 3
  - SPCONF-NAMEPLUS — spconf `\nameplus` leaves `<sup>1</sup>` in the affiliation; 1309.1501 (its doubled mark). Reach (census 2026-11-24): 0 of the 30k sample ≈ 0 papers.
  - SPCONF-EMAIL — spconf `\email` needs an after-authors `\maketitle` hook; 2405.13379's e-mails stay a note (the note now sits under the names, `\lx@names@note`, af9dc7b87af). Reach (census 2026-11-24): 2 of the 30k sample ≈ 186 papers.
  - NOMTHOOK — article with no `\maketitle`: we print † where pdflatex prints 2 (pre-existing; ≤ 56 papers); RED pinned. Reach (census 2026-11-24): 2 of the 30k sample ≈ 186 papers.
  - SUBCAPLIST — subcaption's `\subcaptionlistentry` and `\subcaptiontext` (subcaption.sty:253-254) are unbound (undefined control sequences; 2404.12080, 2503.09244 use the first): bind them faithfully — enter the sub-type (running `\caption@subtypehook`) and `\captionlistentry[sub\@captype]` / `\captiontext` under `\@captype` = `sub<type>` (WINSUITEb; minimal: `\begin{figure}\begin{minipage}{.3\textwidth}\subcaptionlistentry{x}\end{minipage}\caption{F}\end{figure}`). Reach (census 2026-11-24): 0 of the 30k sample ≈ 0 papers; 1
  - LMCSQED — the lmcs binding's `\qEd` is `\ensuremath{\square}` (`lmcs_cls.rs` ~149), where LMCS.cls:551-553 draws its own
    box: a paper that redefines `\square` (1305.0576's ldiagxy.tex:207, a delimited diagram macro) breaks every `\qEd`
    display (217 errors, more since `\[` is `{equation*}`; 0 with `\qEd` overridden). Give `\qEd` the class's box under
    an internal name; repro `tmp/END399/T3W/repros/1305.0576/qEd_square.tex` (tier 3 A/B, pre-existing; 1202.6352). Reach (census 2026-11-24): 1 of the 30k sample ≈ 93 papers; 1
  - TIKZCDHANG — a tikz-cd diagram whose cell has `\hspace{1em}\rar[mapsto]` loops under calc after an earlier cell's
    calc error (1611.01788 thesis.tex:5400/:5434; both the Nov 17 binary and the release candidate; pdflatex is fatal
    there too, 8 errors): find the loop site (`perf record` at 20 s), then end in a bounded Error. The wall-clock Fatal
    leaves no XML and no log, so the paper's 56k words are lost — a salvage question too. rc2 2026-11-23: still the wall clock at 180 s (59 Errors before it). Deferred, decided by difficulty (DG 2026-11-23). Reach (census 2026-11-24): 1 of the 30k sample ≈ 93 papers; 5
  - TITLEFIGPANELS (shape, pre-existing) — a teaser whose panels are `\subcaptionbox`es (2007.15591, vgtc) stays a
    paragraph of four figures (now inside the continued title page, as a `block`), not one panelled teaser figure; a
    subfigure.sty `\captionof{subfigure}` minipage then `\captionof{figure}` inside `\twocolumn[{\maketitle\begin{center}…}]`
    is not moved to the cover (base, TITLEFIG rounds 1 and 2 alike). Reach (census 2026-11-24): 1 of the 30k sample ≈ 93 papers; 3
  - IJMEMAILCOLON (after 0.8.0, nit) — an ijmart e-mail contact gets `name="e-mail:: "`: the class's `\emailaddrname`
    already ends in ":" and the binding appends ": ". Reach (census 2026-11-24): 1 of the 30k sample ≈ 93 papers; 1
  - AAAIAFFIL — (moved from row R 2026-11-26: 11 of 3000 aaai21+ papers, ≈ 60 corpus-wide) a `\thanks`/`\footnote`
    inside aaai's `\affiliations` loses its mark and its place: the deposit (`sect05.rs` `deposit_tokens_of`) leaves
    the kernel `\thanks` (= Perl `latex_constructs.pool.ltxml:1128`, a mark-less pubnote) where TeX prints
    `\footnotemark` at the call (latex.ltx:17213-17216); 0 Warnings (2110.07444, 2312.01324, 2502.18479, 2410.04191).
    PERL-ORIGIN inside the OD #265 surpass. P1 (a note made in place at the call, mark-less, plus an Info) is easy;
    P2 (the glyph from the counter state) is hard. Also: sect05.rs ~:4058-4061 says the deposits `\let\thanks\@gobble`;
    only the replay does; 3
