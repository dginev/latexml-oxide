# Open rows — detail for the SYNC_STATUS worklist

Living worklist (no date in the name): the open-row detail sections moved verbatim from `docs/SYNC_STATUS.md` on
2026-10-23 (rule 3 there: a section past ~100 lines is its own doc). Start at SYNC_STATUS "Ranked worklist"; a row
here is open unless its status says otherwise; when a row closes, delete it here and note it in the month's
`docs/archive/SYNC_SESSIONS_YYYY-MM.md`.

### Corpus Triage Quick Wins
- **AMS-TeX papers read through their own amams.sty: the errors left are our emulation's** (RUST-ONLY, opened 78i r2;
  Perl never reads the style, KPE #1051; `tex` converts math9906120 clean). Since 78i (OD #825) the Annals style is
  read raw and the papers have their content (the 78i bullet's word counts); what errs is our AMS-TeX
  display emulation under the style's own machinery. math9906120: 102 Errors and Fatal:TooManyErrors from
  JohanssonArchive.tex:250 on — 43 "Use \Bbb only in math" and 43 "Script ^/_ can only appear in math mode", after
  the `\@@multline`/`\tag` errors at :250 where amams's `\everydisplay{\lookforbreak}` (amams.sty:585-590: the
  display read to its closing `$$` as a macro argument, split at `\splitmath`) meets amstex's `\multline`.
  math9806089: 13 (no longer Fatal) — 9 from amams's `\split` (lines 3979, 4444) over amstex.tex internals the AmSTeX
  pool does not have (`\ifinany@`, `\next`, `\@end@split`, two stray `&`), 4 from boxedeps.tex, which neither the
  paper nor TeX Live ships (harness). math0101270: 8 — `\scriptsize` undefined from our amsmath binding's
  `\smallmatrix` (`style=\scriptsize`, amsmath_sty.rs:560; Zagier.tex:811, plain has no `\scriptsize`) and 7
  "Missing sub/superscript argument" in expanded strings (not root-caused). math0402285: 0 (78i r2: the kernel tests
  with `\lx@ifdefined`, which amams.sty:1099's three-argument `\ifdefined` cannot reach). Fix shape: amstex.tex's
  display layer (`\multline`, `\split`, `\ifinany@`) as tex.web reads it under a document `\everydisplay`.
- **mn_thesis.cls is bound as `mn` (the prefix alternate)** (72b review finding 1): 0909.2812's mn_thesis.cls:36
  loads mnphd.cls, whose :22 `\LoadClass{report}`; pdflatex 0 errors, six chapters. The class loader's prefix fallback
  (Perl Package.pm:2700-2716, the same alternate there) picks `mn` (MNRAS) by name for both, so the thesis gets MNRAS's
  frontmatter guesses and 12 Errors for the thesis class's own macros (`\adviser`, `\graduationdate`, `\makecopyright`,
  `{dedication}`, `\doublespacing`…); its chapters come from report, which the dependency scan of the shipped classes
  loads (72b r2: no chapterless Warning). Options for the main session: a prefix alternate only where the shipped class
  declares that class (`\ProvidesClass`/`\LoadClass` of it), else OmniBus; or read such a class raw under `rawclasses`.
  Classification: SHARED with Perl.
- **jmlrbook's chapters get jmlr's article Warning** (72b review r2 finding 8): jmlrbook.cls:277-279 redefines
  `\LoadClass` so combine's inner class is jmlr, then `\jmlrbookcommands` (jmlr.cls:950-959) switches `\chapter` to
  `\bookchapter`: a jmlrbook chapter is legitimate. jmlr_cls.rs answers the class and its `\chapter` is always the
  article's `\artchapter` stand-in (OXIDIZED_DESIGN #735), so each jmlrbook document with chapters gets one false
  "Class jmlr Error: Chapters not permitted in articles" Warning; the chapters themselves convert. `\jmlrbookcommands`
  (jmlr.cls:950-956) also makes `\appendix` the book's `\bookappendix` (:936-949: `\thechapter` `\@Alph`, chapter and
  section counters reset), which letters the appendix chapters ("Appendix A"), where ours is jmlr's article
  `\artappendix` (:720-729), lettering sections within the last chapter (72b review r3 finding 5). Fix: a jmlrbook
  binding (or jmlr_cls.rs keyed on the requested class) that runs the book commands: `\chapter` the kernel's, no
  Warning, `\appendix` lettering chapters. No arXiv witness counted yet. Classification: RUST-ONLY (Perl has no jmlr
  binding).
- **A `\documentclass` in the body is not "Can be used only in preamble"** (72b review r3 finding 3, probe
  rev72b/r3/dc2): latex.ltx:18621 `\@onlypreamble\documentclass` (and :18625 for `\documentstyle`) makes a body call
  `\@notprerr`, pdflatex 1 error "LaTeX Error: Can be used only in preamble."; ours raises none (0 Errors) and, as TeX,
  loads no class (the first class's chapters kept, OXIDIZED_DESIGN #179; a second one in the preamble is latex.ltx:18618
  `\@twoclasseserror`). Perl errs here: its `\documentclass` has `beforeDigest => onlyPreamble('\documentclass')`
  (latex_constructs.pool.ltxml:75, :155-160, "The current command '\documentclass' can only appear in the preamble"),
  which sect01.rs's `\documentclass` lacks (`latex_constructs::only_preamble` exists, mod.rs). Fix: the same
  `before_digest`, a one-line port (it adds one Error to every merged-file paper with a body `\documentclass`).
  Classification: RUST-ONLY.
- **combine's `book`/`report`/`memoir` option is ignored** (72b review r2 finding 8): combine.cls:85-88 picks the
  inner class (`\c@lclass`, default article, :29), but combine_cls.rs:11 always loads article, so under
  `\documentclass[book]{combine}` `\chapter` is undefined and a chapter reads #734's stand-in with the chapterless
  Warning, where TeX has book's chapters. Fix: load the option's class (`LoadClass!` of `book`/`report`/`memoir`),
  as combine.cls does. Witness 2310.14794 (combine, article). Classification: RUST-ONLY. FIXED 77v: combine_cls.rs
  declares combine's options, passes the others to `\c@lclass` and loads it (`colclass=` too); guard
  `perfect_kernel_batch77v::combine_book_option_loads_book`.
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
    `\titleformat{name=\section,numberless}` defines a junk macro (13 A/B papers: googledeepmind.cls, mystyle.cls);
    `sectioning-frontmatter/unnumbered_section_takes_the_title_font` GREEN 78a (`\@@unnumbered@section` applies
    `\lx@format@title@font@@`, Perl latex_constructs.pool.ltxml:666-670, and titlesec's after-code,
    `\lx@format@title@after@@`, OXIDIZED_DESIGN_DIVERGENCES #785, so tufte's `\begin{fullwidth}` format closes).
  - *floats/captions*: RED `captions-floats/par_breaks_a_panel_row`; GREEN 78o
    `captions-floats/continuedfloat_minipage_subcaption_continues_the_letters` (`\subcaption` pre-steps the float's
    counter as `\caption@subtypehook` does, caption.sty:642; OD #853); under subfig a third continued part keeps the first continuation's sub-float id suffix (`S0.F1a.sf4`
    under `S0.F1b`); `[caption=false]{subfig}` still gets `\caption@ContinuedFloat`'s check; caption's
    `continuedfloat`/`continued<type>` option sets (caption.sty:526-528) unapplied; longtable ignores `\LTcaptype`; floatfig/JHEP
    `floatingfigure`/`floatingtable` and elsart `algorithm` typed without a source; RED `captions-floats/subref_prints_the_letter`;
    `subfloat_sty.rs:29-30` `\fnum@sub…` = `(\thesub…)` gives "((a))" under `labelformat=simple|empty` (2605.01267,
    2605.10017, 2605.05371; subfig's no longer double-wraps, verified on int40c in 78a: `<caption><tag close=" ">(a)</tag>`,
    refnum `1a`; guard `perfect_kernel_gemini` "((a))"); jfm.cls/interact.cls `\doi` setters (KPE #326); the XSLT DOI-pubnote predicate (DIVERGENCES #345) has no
    HTML guard; `\rotcaption*`/`\rotcaptionof` undefined, `\controtcaption` unnumbered (57l); a `\rule{\linewidth}` panel's
    `width="100%"` is unmeasured by the panel rows (57by, 2605.18774; KPE #389).
  - *loader/kernel*: neither the loader nor `\@ifpackageloaded` honours a `\ver@<pkg>.sty` mark (2605.20200; memoir
    `\EmulatedPackage`; honouring it broke 2605.08402's `\listoffigures*` on the 57bu A/B; the loader half retried in
    66m, KPE #668, awaits a ruling); Rust's `\@addtoreset` omits
    Perl's `defCounterID … unless \the<ctr>@ID` (latex_constructs.pool.ltxml:3044; 57ac).
  - *bibliography* (77v): natbib's `\bibpostamble` is typeset where natbib does (in the last entry, before
    `\endlist`: pdflatex "A. Author, Title. Post text.", 64y); KOMA's `\setbibpreamble` without natbib FIXED 77v
    (koma_script.rs, guard `perfect_kernel_batch77v::koma_setbibpreamble_is_printed`); apacite's citation layer is
    apacite's since int17 (`\cite` parenthetical, `\citeA` textual) — open: its first citation lists every author
    ("Author, Brown, & Cole, 2001"; ours "Author et al.") and `\citeyear` prints "(2001)" (ours "2001").
  - *anchors and hooks*: RED `block-model/hyperdef_starts_the_paragraph`, RED
    `block-model/hypertarget_heading_a_paragraph_keeps_the_space` (both shared); math in a new anchor takes ids chained from
    the anchor's (`m.m1`); RED `macro-state/afterendpreamble_code_is_followed_by_ignorespaces` (RUST-ONLY); RED
    `macro-state/atbegindocument_in_the_body_runs_now` (fire `\document`'s hooks with `\hook_use_once:n`, then drop 57r's
    `\AfterEndPreamble` flag).
  - *layout and lists*: RED `boxes-groups/rotatebox_starts_the_paragraph` (shared); RED `fonts-nfss/ulem_custom_mark_is_rendered`
    (41 of 30,079 2605 papers; the class mapping is a design call); RED `block-model/noindent_paragraph_in_a_minipage_takes_its_id`
    (same defect as 58g's RED `noindent_paragraph_in_minipage_numbers_inside` — merge the two); ~~RED
    `fonts-nfss/textcircled_of_a_box_circles_its_text`~~ GREEN since 78l (KPE #361); `list-structure/enumitem_ref_clears_the_parent_prefix`
    (enumitem.sty:551-556) GREEN since 77u (verified on int40c in 78a: "(a).i"); RED `list-structure/endtrivlist_after_a_heading_starts_a_paragraph` (61p review; OD #435
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
- **FIXED int27 (silent-loss #20) — a `\gls`/`\glsadd` inside display math lists nothing** (batch 56jt residual, DIVERGENCES #320; the location-only reference now goes in an `ltx:text` of the `ltx:equation`, after the formula): the location-only `ltx:glossaryref` of a formula goes to the nearest enclosing level that admits it, after the formula; inline math has its `p`, but `ltx:equation` admits no inline element, so an entry named only in display math (`\begin{equation} \gls{E}=mc^2 \end{equation}`) is missing from the list. `\aftergroup` carries tokens past `$…$`/`\[…\]` (probed), but not safely out of an alignment cell.
- **Fixture exception to the 0-warning rule** (batch 56jt): `stream_a_recall::a_citation_of_another_lists_item_is_missing_here` asserts its one warning, "Missing bibkeys: supp" — the faithful diagnostic (Perl MakeBibliography.pm:342-343; bibtex warns "I didn't find a database entry"), since the key it guards is cited and absent from that bibliography's files by construction.
- **biblatex refsections share one citation list** (batch 56jt residual, KNOWN_PERL_ERRORS #277): each bibliography now reads its own `@files`, but every `\printbibliography` fallback lists every cited key its files hold and warns "Missing bibkeys" for the rest, where biber lists and checks only its refsection's citations (RED repro `index-bib/biblatex_refsections_share_citations.tex`: 2 warnings, Globalentry in the first list); `\printbibliography[section=N]` after the refsections reads the pooled resources (biblatex-apa6-test).
- **babel language tags are lost under the `luatex` profile** (batch 56ji residual): babel loads `luababel.def`, so `xml:lang` is not set; `babel_support_sty.rs:243-254` also forces LGR for Greek (latent). Red repro `tools/perfect_kernel/repros/babel-lang/luababel_language_tag_luatex.tex`.
- **Rust `replace_tree`/`replace_tree_detach` keep the replaced node's box in its parent** (W12 residual, document.rs): Perl's `replaceTree` (Document.pm:2079-2091) runs `removeNode` on the attached old node, so its box leaves the parent's (and each auto-opened ancestor's) box before `appendTree` adds the copy's; Rust's copy-then-detach adds the copy's box without removing the old one. Callers: the math parser (`replace_tree_deferred`), the `svg:foreignObject` cleanup (tex_box.rs). Only `cleanup_math` has Perl's `replaceTree` boxes (`Document::replace_node_as_tree`).
- **A text-only `svg:foreignObject` is not renamed to `svg:text`** (W12 review probe t5, pre-W12): Perl's foreignObject `afterClose` renames it (TeX_Box.pool.ltxml:386-389); the Rust port (tex_box.rs, its comment says it does) has no such branch. A tikz `\node[draw,align=center]{A\\BB}` gives Perl two `<svg:text>` lines, Rust a 10.38×9.46 and a 19.6×9.46 foreignObject, and its cells lack Perl's `ltx_nopad_l` class; `$\phantom{H}$` in a node gives Rust a foreignObject holding nullfont spaces.
- **The foreignObject width override is stale** (W12 review, `latexml_engine/src/tex_box.rs:885-918`): an auto-opened `svg:foreignObject` wrapping a width-bearing `ltx:inline-block` takes that block's `width` attribute in place of its box's width, on the premise "appendNodeBox creates Lists that sum widths incorrectly", which W12 retired (DIVERGENCES #319). It makes `\node[draw]{\fbox{\parbox{2cm}{Hello world}}}` 78.73 px wide (Perl 88.15). Retire it in its own A/B batch.
- **`\node{x $\text{ab}$ y}` is 37.08 px wide, Perl 49.63** (W12 review probe t1, pre-W12): Perl measures an undefined `\text` (no amsmath) as its literal name inside the Math.
- **The pgf shading node `\node{YAY!\pgfuseshading{msh2}}` is 227.16 px wide, Perl 230.62** (unit_tests_by_silviu): Perl keeps a space before the shading picture that pdflatex does not set (`\wd` of `\hbox{YAY!\pgfuseshading{msh2}}` = 123.99pt = "YAY!" 23.61pt + shading 100.38pt); the ~1.7pt Rust still lacks is the Y–A kerns across separate text boxes.
- **Figure panels under `geometry` are measured at the class `\textwidth` but arranged at `\Gm@tw`** (pre-existing; `latex_constructs/mod.rs` `after_float` floatwidth override, OXIDIZED_DESIGN #99): in 2605.03152 (`[margin=1in]{geometry}`, S3.F2) the `0.41\textwidth` subfigures are 141.5pt wide (0.41 × 345pt) against a 469.8pt row. The W12 fixup's panel-row change (KNOWN_PERL_ERRORS #274) turned its rows from 2 | 2 (56jk, pdflatex's layout, reached through Perl's row-width carry-over) into 3 | 1; pdflatex (0.41 × 469.75pt) sets two per row, separated by the source's blank line, which the rows ignore (Perl alike). FIXED (78x, OD #885: the panels' stated widths are taken at the geometry's width too; 2605.03152 S3.F2 2 | 2; guard `perfect_kernel_batch78x::geometry_text_width_sets_the_panel_rows`).
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
  - *tabularray* (57cp; `evaluate=`/`expand=` landed 59n): ~~rules (`hlines`, `vlines`, `hline{…}`) are not drawn, so
    an all-empty table is pruned whole~~ (FIXED 78p, OD #855: the rule keys and a rowspec's `|` are cell borders; open:
    the body's `\SetHline`/`\SetHlines`/`\SetVline`/`\SetVlines` stay no-ops, rule widths, dashes and colours are not
    rendered); a math table without a colspec is sized before `evaluate=`/`expand=` run (RED `alignment-bindings/tblr_math_columns_follow_the_expanded_body`); custom `caption-tag`/`caption-sep`
    ("Quadro 2 —", tabularray-abnt); a long/tall table in a captioned `figure`/`subfigure` nests a `table`; document
    templates print as tabularray's own (tblr-extras head, `label=none`); a math table's `tex=` holds the reduction;
    ~~`\SetCell[c=…,r=…]` spans~~ (FIXED 78l, OD #834; `cell{i}{j}={r=…,c=…}` spans open), `rownum`/`colnum`/`rowcount`/`colcount`; `cmd=`/font styles; the LoT line of a caption-less
    long table. Settled (pdflatex errors too): `\caption{\url{…#1}}`, floats in minipages, `{l→}`.
  - *floats/captions*: RED `captions-floats/{two_captions_in_one_float_are_numbered_apart (KPE #396),
    subfig_subfloats_restart_in_each_figure, float_in_a_subfloat_row_keeps_the_parent_number (KPE #395 part),
    captionsetup_type_in_a_figure_minipage, captionless_longtable_is_not_listed, label_after_a_continued_caption,
    label_after_a_caption_with_a_note_names_the_float (SHARED), filecontents_graphic_has_its_size}`, RED `boxes-groups/caption_minipage_in_a_table_cell`; text a `\refstepcounter`
    rebinding typesets is dropped; floatrow — a footnote ignores `\def\@mpfn{mpfootnote}` (floatrow.sty:476-478); a
    `\floatfoot` in a plain `figure`/`table` or a `\DeclareNewFloatType` float is FIXED int27 (the float's end sets the
    pending foot, floatrow_sty.rs `\lx@flrow@plainfoot`; guards `perfect_kernel_batch58::floatrow_floatfoot_in_{plain_figure,
    new_float_type}_keeps_its_text`; the new-type float itself was already kept); a floatbox caption lacks `ltx_centering` and a zero-size `<rule>` follows the body
    (:392); float placements are unchecked (`[!H]`); (a tcolorbox listing's title/number: FIXED 77m, silent-loss #16).
  - *box measurement* (58b-58e; K15/K18): `\vsplit` never splits a paragraph into lines (reledmac
    2-titles_in_line_numbering_with_notes 16 numbered lines vs 28); `\penalty` leaves no item, no interline glue or
    `\splittopskip`; `{x\par y}` measures one line more; `\lastbox` removes any last item; a note counts as a line break,
    plain `\footnote` is measured with its body, a minipage footnote's text is uncounted (DIVERGENCES #391); size10.clo
    leadings whatever the class size; a footnote keeps the surrounding leading; RED `alignment/booktabs_rules_add_their_spacing`;
    RED `boxes-groups/negative_kern_overprints` (2605.14271); 2605.02221's `\rTo{\pi}` sequences now parse as fragments;
    reledmac — `\patchcmd{\endminipage}` fails (reledmac.sty:7643-7654), `\footnote` in `\pstart` splits the line.
  - *fonts* (58f, 58g, 58k, 59d): a raw font's `\fontdimen` reads cmr10's, `\fontcharwd`/`\fontcharht` the undecoded char;
    the lcircle map is unchecked; `\ding` 7.5pt vs 5.71pt; a standard-metric font's missing char (KPE #408); `\f@encoding`
    inside a raw font's group; RED `fonts-nfss/math_char_without_glyph_keeps_its_box` (`t1_guillemet_ligatures` GREEN 60s); a redefined `\everymath` is not run (latex.ltx:10580).
  - *lists*: RED `list-structure/stale_list_counter_reset_gives_I0_ids` (Perl-origin; 2605.04012); a `\list` on the
    enclosing counter inside `\lx@trivlist@setup` gets `i0`; RED `block-model/noindent_paragraph_in_minipage_numbers_inside`
    (2605.21713; same defect as the K13 row's `noindent_paragraph_in_a_minipage_takes_its_id`).
  - *frontmatter* (58m, 58q, 59d-59g): RED `sectioning-frontmatter/{deposit_of_only_punctuation_is_dropped,
    store_default_that_typesets_nothing_leaves_no_element, ijcai_markers_in_an_appending_author}` (the store-timing
    pair GREEN 77i); bfh-ci
    Factsheet/ProjektProposal lose their department (a footer-layer semantic note, kept under the 2026-10-01 ruling);
    `\addresslink`-style marks `\let` to `\textsuperscript` are not author marks; dtk.cls `\Author`; RED
    `singletons/omnibus_alias_shadows_a_local_package`; g-brief (59o, #412): only a serial document's first letter gets
    its sender/addressee (RED `sectioning-frontmatter/gbrief_serial_letters_each_have_their_frontmatter`), a contact
    label's ligatures stay untypeset in `name=` (RED `sectioning-frontmatter/contact_name_label_keeps_its_ligatures`); acmart — the year without `\copyrightyear`/`\acmYear` is the
    conversion's, a document's own `\footnotetextcopyrightpermission` lands in the first paragraph, the cc logo/link and
    `authorversion` statement are unported, an unknown `\setcopyright` is silent.
  - *bibliography and notes*: `\printbibliography[type=…]`/`[check=…]` all print under the first call (biblatex_sty.rs
    `\biblatex@printbibliography`; `~/data/pk_agents/rc132/repro/typed_print.tex`; the biblatex manual's 244 missing
    words); endnotes `\theenmark` = `\theendnote` (PERL-ORIGIN endnotes.sty.ltxml:29; cms-noteref-demo "18" ×18; FIXED 77v,
    KNOWN_PERL_ERRORS #1013); enotez
    `split=section` — a `\chapter` starts a split, a pre-heading mark's `\endnotetext` lists in the mark's split, 0.10d
    internals overridden; biblatex-ext-oa's open-access mark (xsavebox: `boxes-groups/xsavebox_saved_box_prints_its_content` GREEN 78a,
    `\pdfxform`/`\pdfrefxform`, OXIDIZED_DESIGN_DIVERGENCES #788);
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
  (92.7 %): author continuation lines glued at `\\` (`add_authors_calls`, base_utilities/author_block/dispatch.rs:259-264 = Perl
  Base_Utility.pool.ltxml:702-703, PERL-ORIGIN; keep the delimiter as `<break/>`; `~/data/pk_agents/rc131/repro/author_glue.tex`).
  Ruled out: suanpan-l3's last jing line (a second `\title`/`\maketitle` replaces the first — double `\maketitle` keeps
  the latest title, user 2026-10-01).
- **Frontmatter side findings from 78f / int48** (for FM S2; unchanged from int47a unless stated): 2409.17846's names
  keep a trailing comma before `\orcidlink` ("Qin Tan,", "You Wu,"); an iopart `\author{A and B}\address{…}` gives the
  address to B only (rev78f2 probe i1); IOS (`infor.cls`) `\address` labels `aff.a`–`aff.d` land on the corresponding-author
  note and on the document instead of their affiliations (2406.15038; hidden, no `\ref` uses them; int48 moved
  `aff.b`–`aff.d` from the note to the document and gave the note the step's refnum).

### Perfect-kernel RED repros without a row elsewhere

The open RED repros of `tools/perfect_kernel/repros/` that no other row names; headers re-checked 2026-10-02 (scope:
PERFECT_KERNEL.md → Scope; a witness whose oracle is unclean is a crash canary). Each row: defect, classification, repro path.

- **A footnote's numeric `[<n>]` is not marked `\thefootnote`, so symbol marks leave a mark and its text apart**
  (PERL-ORIGIN, KPE #1060; **FIXED 78o**, OD #851): `[n]` prints `\thefootnote` at `n` (read undigested, so
  `[\value{footnote}]` is the counter), article's/report's/book's title block marks in `\@fnsymbol`, its `\thanks` and
  `\footnote` step the counter (78o r2) and it resets the counter, a mark and a text pair by printed mark, else by
  counter value when one is in the title block or neither prints a mark — across the counter reset, by value only
  between notes naming their number and by printed mark unless both are numbered from the counter (78o r3). Witnesses
  2507.05750, 2412.20504, 0903.0014, 2006.14582, 2201.11183, 2111.13260, 2606.03847; guards `perfect_kernel_batch78o`.
  Residuals: the frontmatter digests the fields in the order the preamble set them, so a title `\thanks` set after
  `\author` steps after the authors' (the title's first in pdflatex); a `[<n>]` that is not a number
  (`\footnotemark[$\dagger$]`, pdflatex "Missing number, treated as zero") is printed as given with no diagnostic, as in
  Perl — and stays so (main-session decision 2026-10-22, at 78u's integration: no diagnostic for now, Option 3; the Warning
  first decided false-alarmed). The faithful rule is the open part of this row: warn only when the definition in force is
  LaTeX's counter-setting `\@xfootnotemark`/`\@xfootnotenext`/`\@xfootnote` (the PASJ/emulateapj bindings defining their
  class's versions, aa/arximspdf no longer passing text marks through `\footnotemark[…]`). 78u's census A/B found that 15 of the 19 papers it would warn
  are PASJ papers whose class reads the mark as text (pasj00.cls:2237-2239, pasj01.cls:2308-2310
  `\protected@edef\@thefnmark{#1}`; so do emulateapj.cls:558-570 and ecai.cls:480's author `\footnotemark[\dagger]`),
  which our locked `\footnotemark` does not see, and three binding paths pass text marks through `\footnotemark[…]`
  (aa's `\tablefootmark`, arximspdf's `\freefootnotetext`, a cleveref format's `\lx@therefnum@@`): the Warning is not
  landed, a rule is open (78u report; RED repro package-bindings/footnote_mark_not_a_number_prints_as_given).
  `[\numexpr…]` is a number since 78u (KPE #1135). 0707.3986 is classified, not a residual: RR.sty:141
  `\footnotetext[0]` under RR.sty:445's `\fnsymbol` prints no mark, ours too; its TL 2025 PDF's "0" is pdflatex's
  recovery from RR.sty's own errors (the `\xdef` of the accented theme name, :160-168: "Missing } inserted", "Too many
  }'s" close `\makeRR`'s group early, and `\thefootnote` is article's `\@arabic` again at :477).
- **cleveref's reference forms: the review rev78j3's later items** (RUST-ONLY, 78j's forms, OD #845; no RED repro yet):
  (a) a document multi format's conjunction prints inside the link — the form is whole in the link, pdflatex links only
  `#1`, so 2609.28438's `{ and~(#2#1#3)}` puts " and " inside the second `<a>` ("Problems (P) and (P1)"); (b) internal
  ids shift where a form holds math or a paragraph (2412.09235's math and para ids: the forms in `ltx:tags` take ids
  first), and in a tabular's minipages a label type's tags take their place in insertion order (2609.28438: the tags'
  order in `ltx:tags` and the ids after them shift, the HTML identical); (c) `\eqref` of a `\tag{$\star$}` walks to its parent for context ("⋆ ‣ 1", Perl's `generateRef`, not a
  format issue); (d) a math tag form's alttext reads `{{1}}` (the number passed as one group, `cref_apply_format`'s
  `{{<number>}}`).
- **A cleveref format holding an undefined command raises nothing** (RUST-ONLY, 78j's forms, OD #845; found by rev78j4
  probes n5, g5; no RED repro yet): `cref_text_token` (cleveref_sty.rs) reads a command with no definition as not text,
  so `\crefformat{figure}{#2\undefinedfoo Fig.~#1#3}`, or `#2\color{red}Lem.~#1#3` without xcolor, builds no form and
  each reference prints cleveref's default silently, where pdflatex raises "Undefined control sequence" at every
  reference that prints the format (n5 1, g5 2 for two references). Fix direction: an undefined command is text that
  errs — build the form, its held Error replayed per printing reference.
- **A `\newtheorem{<env>}[<counter>]` checks its counter only at the first `\begin{<env>}`** (SHARED, Perl's
  `\newtheorem` reads the counter lazily; found by rev78o probe a1): pdflatex errs at the definition ("No counter
  'theorem' defined", the environment left undefined). Since acmart makes its theorems `\AtEndPreamble` (78o, OD #854),
  `\documentclass{acmart}\newtheorem{remark}[theorem]{Remark}` is silently "Remark 1.2" on the class's theorem counter
  in ours, an error in pdflatex; with no counter at all (`[foo]`) ours and Perl err at the `\begin{remark}`.
- ~~**algorithm2e's `\Indp`/`\Indm` do not indent the lines after them**~~ (SHARED, binding, pre-existing; found by
  rev78o probe g1): algorithm2e.sty:1626-1629 `\Indp` advances `\leftskip` by `\algoskipindent` until `\Indm` (pdflatex
  "indented;" one level in); ours and Perl set the line at the block's level (int52a the same). **FIXED 78r** (KPE
  #1107): a signed offset in levels that a block saves and restores (r2; r1's push/pop of the blocks' stack set the
  flush-left `\Indm … \KwIn … \Indp` idiom's body a level in, 2102.06989); guards
  `perfect_kernel_batch78r::{algorithm2e_indp_indm_indent_the_lines_between,
  algorithm2e_indm_before_the_input_keeps_the_body_at_the_margin, algorithm2e_indm_in_a_block_ends_with_the_block}`.
- **A body `\footnotetext` pairs with a mark printing the same number far away** (PERL-ORIGIN rule, Perl's
  printed-mark `relocateFootnote`; found by rev78o2): 2509.17784's `\footnotetext[\value{footnote}]{Results are based
  on Gemini 2.0…}` (main.tex:134, "1" since 78o r2 reads `[<n>]` as the counter) moves into the appendix's COAT
  `\protect\footnotemark[1]` (sections/appen_exp.tex:290), where pdflatex prints the text on its own page and the
  appendix mark alone. Int52a left it a text (its mark printed ""). A pairing bounded by position (the nearest mark
  before the text) would be the general fix; open.
- **A title page's literal `\footnotemark[<n>]` copies a body text of the same number** (PERL-ORIGIN, Perl's
  `relocateFootnote` pairs every mark printing the text's mark; found by rev78o2; int52a and 78o r3 alike): report
  (title page) `\author{A\footnotemark[1]}`, body `x\footnotemark{} \footnotetext{two}` — pdflatex "A1" with no
  text, "x1" and "1 two"; ours puts "two" in A's mark and in the body's (duplicated). The literal `[<n>]` keeps its
  pairing across the counter reset on purpose (2505.14146); telling an intended title answer from a body one needs the
  position rule above. Open.
- **The frontmatter observer labels `[\arabic{footnote}]` with the counter before the title's notes** (FM S2a.2;
  found by rev78o2): the printed mark steps with the title's `\thanks` (`\lx@title@footnote@step`, 78o r2), the
  observer's rehearsal evaluates the argument against the unstepped counter, so the two agree for a bare
  `\footnotemark` and a literal `[<n>]` only (OD #851). Fixed in FM S2a.2 (the rehearsal reads `[<n>]` with the steps
  the block's notes made; guard `perfect_kernel_batchfms2a2::footnotemark_number_reads_the_stepped_counter`).
- **amsmath's alignment dimens `\totwidth@`, `\alignsep@` (and `\black@`) are undefined** (SHARED, binding; KPE #1087;
  **FIXED 78o**, amsmath.sty:1309-1317, :1364 in amsmath_sty.rs; 0912.5438 156 → 124 Errors, its tail back). Guard
  `perfect_kernel_batch78o::amsmath_alignment_dimens_totwidth_alignsep_are_defined`.
- **algorithm2e's `noline` blocks never give back their indentation** (SHARED, KPE #1062; **FIXED 78o**, OD #852:
  `\algocf@Noline` pops its level). Guards `perfect_kernel_batch78o::algorithm2e_*`.
- **amsmath displays do not run `\everydisplay`** (SHARED, binding; found by rev78g3, witness 2212.07486): amsmath's
  `{align}`/`{gather}` family start with `$$` (`\start@align`, `\start@gather`), so TeX runs `\everydisplay` (tex.web
  §1145) at every one; the bindings build the display without it (amsmath_sty.rs:443, the `\if@display` note), while
  `\[…\]` and `{equation}` run it (tex_math.rs `begin_mode`). cs839.sty:23-30 resets its `\labelrel` counter `relctr` in
  `\everydisplay`, so in amsmath displays the relation labels keep counting ("r1 r2 r3" for pdflatex's "r1 r1 r1"; Perl
  the same). RED `ams-internals/amsmath_display_runs_everydisplay`.

- **`\halign` never inserts `\everycr`** (SHARED, kernel; found by 78a; **FIXED 78b**, KNOWN_PERL_ERRORS #1019):
  tex.web §774/§799 insert `\everycr` after the preamble's `\cr` and after every row's; tex_tables.rs `insert_everycr` does
  for the `\halign` primitive (binding-built alignments stand for `\ialign`-style macros and run none). It unblocked
  AMS-TeX's active `@` (KNOWN_PERL_ERRORS #1018, landed with it: math0005042 28 errors → 0, no Fatal). Repros
  `kernel-alignment/everycr_*`, `amstex/amstex_active_at_reads_the_arrows` (GREEN); guards `perfect_kernel_batch78b::*`.

- **subcaption sets the main caption at 90 %** (PERL-ORIGIN; found by rev78a2; **FIXED 78c**, KNOWN_PERL_ERRORS #1021:
  the sub-captions alone are `\small`, the guards re-pinned to the normal size): subcaption_sty.rs's
  `\fnum@font@float`/`\format@title@font@float` are `\small` (Perl subcaption.sty.ltxml:34-35), so with subcaption loaded
  every figure's and table's label and caption text read `fontsize="90%"`; pdflatex sets them in the text font (CMR10,
  checked in jfm and in plain article + subcaption). Pinned as it is in
  `perfect_kernel_batch78a::{jfm_copy_caption_yields_to_caption_loaders,
  jfm_copy_caption_yields_to_caption_loaders_after_floatrow}` (repro
  `class-bindings/jfm_caption/jfm_copy_caption_yields_to_subcaption`); open, small: drop the two `\small`s, then
  re-pin the guards and the captions-floats goldens. 78c r2: a `\usepackage[<options>]{caption}` after subcaption (the
  binding's caption load is the first) runs its options as caption3.sty:416-429 does (`caption.sty_keyval_setter`;
  witness 1602.00134). Open (KPE #1021): sub-captions stay `\small` whatever `\captionsetup[sub]{font=…}`,
  subcaption's `font=` or caption's global `font=small` (→ `smaller`, 8 pt) say (caption_sty.rs `passthrough`).

- **svmult's main-matter `\appendix` starts the appendices** (PERL-ORIGIN, svmult_cls.rs:56 `\@appendix` + `\gdef\@chapapp
  {\appendixname}`; found by rev78a3; **FIXED 78c**, KNOWN_PERL_ERRORS #1026: the main-matter `\appendix` is `\par`, the
  sections are `\@arabic\c@section` without the chapter's reset until a `\title`, svm_app now "Chapter 2 After", "3 N"):
  svmult.cls:713's `\appendix` is only `\par` — the main matter's sections keep
  counting (78a r3 keeps that) and its chapters stay "Chapter N"; here a later `\backmatter\chapter{After}` prints
  "Appendix A", "A.1" where pdflatex prints "Chapter 2 After", "3 N" (probe `svm_app`; int40c, 78a r1-r3 the same; none
  of the 8 corpus svm papers with `\appendix` changes). Open, small: svmult's `\appendix` a `\par` outside the back
  matter, its back-matter `\bppendix` as 78a r3 ports it.

- **An svmult volume of several contributions numbers only the first** (GENUINE-RUST-ONLY kernel shape, predates 78c;
  rev78c finding 6, KNOWN_PERL_ERRORS #1026 open): the kernel's `\lx@maketitle@cleanup` (sect05.rs) lets `\title` and
  `\maketitle` be `\relax` after the first `\maketitle`; svmult's (svmult.cls:2141-2152) does not, so each contribution
  is its own chapter. Probe svm1 (`\title{First}`, `\title*{Second}`, `\title{Third}`, each `\maketitle`d): pdflatex
  "1.1, 1.2 | 1, (1) | Chapter 2, 2.1", ours "1.1, 1.2, 1.3, 1.4", the later titles paragraphs ("*SecondB", "ThirdC");
  int43a the same. Not a binding line: several titles in one document need per-contribution titles in the frontmatter
  model (the double-`\maketitle` ruling keeps one title); RULED 2026-10-21: deferred, kept recorded (rare on arXiv).

- **`\cref` to an `\item[…]` of a description inside an enumerate prints no name** (GENUINE-RUST-ONLY since 77u r3, base
  printed the item's own label; review rev77u2 finding 4): the refnum is now the enclosing enumerate item's ("2", KNOWN_PERL_ERRORS
  #1003), but the item's cleveref forms are built for its own list's type (`\lx@make@tags{@desci}`, dialect.rs
  `ref_step_item_counter`), which has no `\crefname`, so `\cref{d}` prints "2" where pdflatex prints "item 2" (cleveref's
  `\cref@currentlabel` keeps the enclosing item's type, as `\@currentlabel` its number). Fix: the `creftype…` tags of an
  `\item[…]` that takes another counter's label are that counter's type (not one line: the role-less and `typerefnum`
  tags stay the item's own). RED `list-structure/cref_to_a_description_item_in_an_enumerate`.
- **biblatex-only `.bib` fields are digested under natbib/BibTeX** (GENUINE-RUST-ONLY, predates 77a; 77a r5 review
  finding 3): 47 field handlers of latexml_engine/src/bibtex.rs, none in Perl's BibTeX.pool — the block at :1962-2172
  (19 defaults: `journaltitle`, `location`, `urldate`, `titleaddon`, `addendum`, `maintitle`, `mainsubtitle`,
  `maintitleaddon`, `booksubtitle`, `booktitleaddon`, `issuetitle`, `issuesubtitle`, `eventtitle`, `eventtitleaddon`,
  `venue`, `eventdate`, `editora`, `editorb`, `editorc`; 27 per-type variants: `inbook|incollection@location`,
  `inbook|incollection|inproceedings@maintitle|mainsubtitle|maintitleaddon`, the book/proceedings host variants) and
  `annotation` (:2337) — exist for the biblatex `.bib` route but are defined on every route, so natbib + plainnat
  prints content the PDF does not have: pdflatex "Ann Able. Paper. J Real, 3:1–2, 2001." 0 errors; we print
  "Main Work Title . Paper . Addon Words . J Real, \undefinedmacro Jt 3, pp. 1–2. Note: An addendum text Accessed:
  2020-01-01" and `Error:undefined:\undefinedmacro` (Perl keeps the fields raw, `\bib@field@default@default`).
  **FIXED 78i** (OD #823; quality ledger #155): the handlers moved next to the `shorthand`/`labelalpha` handlers the
  biblatex binding defines after its `LoadPool!("BibTeX")` (biblatex_sty.rs); witnesses 2602.10008, 2606.11822; GREEN
  `index-bib/natbib_biblatex_only_fields_are_raw_data` (77a r5 review probe n7) and its biblatex twin.

- **etna's `\remark` ends by the class's `\@endtheorem`, which the native `\endtheorem` never runs** (SHARED, Perl 2; 70b
  E4): etna.cls:288-326 defines `\remark` as `{…\@begintheorem…}` (its `}` closes `\@begintheorem`'s `\bgroup`, its
  own `{` stays open) and `\endremark` as `\endtheorem`, i.e. latex.ltx's `\@endtheorem` = the class's
  `\egroup\ignorespacesafterend`; ours is the theorem binding's native end, so the `{` stays open ("\endgroup Attempt
  to close non-boxing group", pdflatex 0). Not local: the fix is a theorem end that consults a class's `\@endtheorem`
  (or the class's `\@begintheorem`/`\@endtheorem` pair as a unit), a theorem-machinery change. Witnesses 2409.06465,
  2603.20879. Repro `boxes-groups/etna_remark_ends_by_class_endtheorem`.
- **acmart's theorems shadow an author's command of the same name** (SHARED, KPE #1086; **FIXED 78o**, OD #854: the
  class's `\AtEndPreamble` + `\@ifundefined` per theorem, `acmthm` read). Witnesses 2510.04987, 2306.01662; guards
  `perfect_kernel_batch78o::acmart_*`.
- **A `$\phantom{$-$}$` toggle in a `{cases}` cell never ends its in-math text** (SHARED, Perl 9; 69c side finding): the author's
  `$`s in a math cell toggle through `\lx@dollar@in@mathmode` (tex_math.rs), whose boxing-level bookkeeping
  (`MATH_ALIGN_$_BEGUN`) loses the text's end across `\phantom`'s argument; 9 Errors and the rest of the display
  mangled (pdflatex 0). Witness 1912.00050 (sigma.cls). Repro `alignment/cases_cell_dollar_phantom_toggle`.
- **2305.03559's clevethm.sty `\setlist[enum#1, 1]{enumerate, label=…}`** (69c side finding): 90 Errors
  "'enum…' is neither a list nor a level" (enumitem_sty.rs:801) for the lists its `\CreateTheoremLists` makes with
  `\newlist`; a plain `\newlist` + `\setlist[name, 1]` in a macro is clean, so the cause is in clevethm.sty's own
  wrappers (its :217/:237 keyval warnings). Not shrunk.
- **amsart's caption takes display math** (SHARED?; 68z C11): amsart.cls `\@makecaption` sets the caption in a `\vbox`
  paragraph, so `$$…$$` there is display math (pdflatex 0 errors); the kernel's `\lx@@caption` digests in text mode
  (latex_constructs/sect09.rs), `$$` is an empty pair and the scripts between err (2 Errors). Needs a display inside
  `ltx:caption` (schema) or an amsart caption mode. Witness 0803.4182. Repro `captions-floats/amsart_caption_display_math`.
- **An `\@ifnextchar`/`\@ifstar`-headed macro at a cell head reads the `&`** (SHARED; 68z review): the cell-head
  look-ahead stops at a `\newcommand` optional-argument macro only (gullet.rs `read_x_token_cell_head`), so
  `\def\nx{\@ifnextchar[{\nxa}{\nxa[d]}}` with `\nx[x] & b1` prints "<x|" / "> b1" (pdflatex "<x|>" / "b1"). Fix:
  stop at every closure that peeks by `\futurelet`, with an opt-out for the `\noalign`-headed bindings (`\cmidrule`,
  `\toprule`) — KNOWN_PERL_ERRORS #880 residual. Repro `kernel-alignment/cell_head_ifnextchar_user_macro_reads_the_tab`.
- **A `\cite` note in display math is set in text** (SHARED, Perl restricted_horizontal; 68z C6): latex.ltx:17798
  `\@cite` keeps the current mode, so `$$ \cite[\text{I}.2.11^\prime]{Mac} $$`'s `^` is a script; `\lx@@cite`
  (latex_constructs/sect11.rs) digests the note in text: 1 Error. Needs a Math in the cite's note. Witness math0402383.
  Repro `index-bib/cite_note_in_display_math`.
- **`\multirow` in a `p{}` column spans no rows** (PERL-ORIGIN; **FIXED 78a**, KNOWN_PERL_ERRORS #1015: the empty p cell
  below counted as content, `isSkippable`, so the span was truncated; a row span now covers a cell holding only a p box
  with a skippable body, the `paragraph_cell_body` property `Digested::is_empty_paragraph_cell` reads; guard
  `perfect_kernel_batch78a::multirow_in_a_paragraph_column_spans_its_rows`, 2605.23257's table 122.9 → 62.3pt): the `p{0mm}` cell gets no `rowspan` where the `c`-column copy gets `rowspan="2"`, so the rotated label inflates its row (2605.23257 S5.T4 169.5pt vs pdflatex 116.2; A2.T7). Both bindings put `rowspan` on the alignment lookup's current column (`multirow_sty.rs:18-45`), which misses inside a p cell's box. Repro `alignment-bindings/multirow_in_p_column_spans.tex`.
- **`\resizebox` of a zero-size box** (PERL-ORIGIN, K18 step 2 residue): a 0-width box gets `xscale="6553600"` where graphics.sty's `\Gscale@div` reports "Division by 0" and uses 1. Repro `boxes-groups/resizebox_zero_size_division.tex`. (The `pspicture` half landed in 58i, DIVERGENCES #397: `graphics-tikz/pspicture_size_is_typed` GREEN.)
- **`\obeylines` makes the newline an `<ltx:break/>`, not `\par`** (PERL-ORIGIN; OUT OF SCOPE): Perl/Rust let the active `^^M` be `\@break` (`plain_base.rs:467-470`), latex.ltx:575-576 `\obeyedline` (= `\par`), so `\let\par=\cr \obeylines` ends no row. 6 errors, first `Error:unexpected:&`. A faithful fix changes goldens everywhere: ruling needed. Repro `alignment/metre_obeylines_halign.tex`.
- **A `\def` whose parameter text meets `}` swallows text to the next `{`** (SHARED; kernel): tex.web §474-475 inserts "Missing {" and ends the definition at the `}`; `\def … UntilBrace` (`tex_macro.rs:91`) reads on, so storecmd-guide loses two sentences. 2 errors vs pdflatex 6. Repro `expansion-primitives/def_param_text_right_brace_storecmd.tex`.
- **A `\lastbox` loop counts 2 title lines, pdflatex 1** (SHARED): `\lastbox` pops whatever item is last (`tex_box.rs`; Perl TeX_Box.pool.ltxml:596) and paragraphs have no line boxes; needs K15's list tail record. Repro `expansion-primitives/unpenalty_lastbox_sidenotes.tex`.
- **`\keys_set:nn {siunitx}` finds no l3keys module** (SHARED): the binding declares keyval `SIX` keys, so currency.sty:90's raw `\keys_set:nn {siunitx}{round-precision=2}` gets l3keys' unknown-key error (1 `Error:errmessage`; Perl prints it uncounted). Fix: declare the `siunitx` l3keys module over the binding's keys, routed as `\sisetup`. Repro `expl3/l3chk_siunitx_keys_set_l3keys_empty.tex`.
- **chessboard under beamer overlays** (RULED residue, DIFFICULT_CASES §D14, Perl's single-pass overlays kept): since sweep #149 status 2 with 4 errors and no Fatal (two skak `\errmessage mainline`, an `\EatNumberA` runaway, an `\end{frame}` mode error; the document is valid), first `mainline: black, not white, to move (e4)`; in scope. Kept (`accepted_residuals.tsv`: `errors 4`). Repro `expl3/s41msg_chessboard_beamer_only_gamestate.tex`.
- **fourier-orns ornaments print their `futs` slot characters** (SHARED): no `futs` glyph map in `latexml_core/src/common/font.rs`, so the raw preload prints "A 1 B M C U D e". Fix: map `futs` to the Unicode ornaments. Repro `fonts-nfss/fourier_orns_ornaments_are_their_glyphs.tex`.
- **forest-doc's `\DocInput` example: binding gaps** (OUT OF SCOPE, R8b): 3 errors (`\lst@InstallKeywords` SHARED; `\@escapeifif`, `\forest@file@copy` from the forest binding, a native tree since 56iz). Repro `graphics-tikz/forest_docinput_lstenv_optarg_absent.tex`.
- **A raw `\item[…]` label is re-digested for the `typerefnum` tag** (PERL-ORIGIN, latex_constructs.pool.ltxml:1384-1386): `\typerefnum@<counter>` digests `Revert($tag)`, so `\item[\n]` gives `Error:undefined:\n` and `\item[\(]` opens math. 5 and 2 errors (Perl 61 and 43). Fix: build the typerefnum from the formatted tag. Repros `index-bib/item_tag_makelabel_typerefnum.tex`, `index-bib/item_tag_math_cascade_downstream.tex`.
- **An `lstlisting` in a `\parbox` argument is dropped** (CONTROL; RUST-ONLY; not silent since 64zf — it raises 1 Error, silent-loss #27): the pre-tokenized argument puts the body on the begin line, which the reader drops; an empty `<listing/>` (pdflatex 7 errors, no PDF). Needs a diagnostic only when the reader's source is a pre-tokenized argument. Repro `kernel-alignment/pcol_lstlisting_parbox_CONTROL.tex`.
- **`\RequirePackage` inside a group raises no error** (OUT OF SCOPE): pdflatex says "Loading a class or package in a group" (latex.ltx:18699-18704) and loses `{multicols}` at the group's end; Rust loses it silently. Fix: the group-level check in `\RequirePackage`/`\usepackage` (`latex_constructs/sect05.rs:180`). Repro `loader/multicols_afterpackage_group.tex`.
- **A product before a big operator stays unparsed** (RUST-ONLY): `a\cdot b\sum_i c_i`, `\frac1n\sum_i x_i\cdot\frac1m\sum_j y_j` — 4 `Warning:unparsed_math`; Perl parses all four. Not root-caused (the wide/narrow big-operator pragma or a missing `term mulop term bigop_application` path); a bare OPFUNCTION as the MulOp's right operand alike (`a\cdot\log\sum_i x_i`, `a\cdot\mathbb{E}\sum_i X_i`; Perl (a·log)·∑…, 57cf review), and `a/b\sum_i x_i`, `a\times b\int f\,dx`. Repro `math-parse/product_before_a_big_operator.tex`.
- **An `\index` phrase is re-tokenized live** (SHARED; with the `&` glossary row above): SanitizedVerbatim (`base_parameter_types.rs:692-716`) re-tokenizes `\index`'s argument with normal catcodes, so `\index{a\def{b}c}` runs `\def` at the mark (1 error). Repro `string-mouth/index_phrase_primitive_retokenize.tex`.
- **An \item inside {framed}** (RUST-ONLY shape; GREEN on errors): the boxed `\item` keeps the outer numbering (`S0.I1.i2`) but sits in a framed `<itemize>` inside item 1's paragraph, not as the list's own second `<item>` (Perl: 2 `Error:malformed:ltx:item`). Ruling needed on the shape: a framed sibling item or the nested framed list. Repro `boxes-groups/mal_item_framed_capture.tex` (colorframed-doc, OUT OF SCOPE).
- **hyperref's PDF information values are recorded unexpanded** (SHARED, Perl hyperref.sty.ltxml:133 `ToString`):
  **FIXED 78i** (OD #824, KPE #1050; 1210.7805): the text keys are expanded as `\pdfstringdef` expands them when
  `\hypersetup` sets them, `\thanks` and the note commands gobbled, the text recorded; GREEN
  `sectioning-frontmatter/hyperref_pdfinfo_expands_macros`, `…/hyperref_pdfinfo_drops_notes_and_markup`. Still open:
  an AASTeX 7 address recorded as `schema:email` metadata (`\email{\url{…}}` → `content="\url{…}"`; RED
  `sectioning-frontmatter/aastex7_email_markup_recorded_as_text`, OXIDIZED_DESIGN_DIVERGENCES #479).
- **FIXED int26 (OXIDIZED_DESIGN_DIVERGENCES #754, KNOWN_PERL_ERRORS #980) — a `{abstract}` left open at `\end{document}` empties the document** (SHARED; 65c review side finding): the deferred
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
  the witness. **FIXED 78a**: the text libxml merged into the formula's space node left the parent current, so no close
  ligated it; that node is now current (document.rs `open_text_internal`, as Perl's `setNode`), and the binding's `\uns`
  is the class's again. Guard `perfect_kernel_batch78a::text_after_a_space_only_formula_is_ligatured` (repro
  `fonts-nfss/ligature_after_space_only_formula` GREEN).
- **bibentry's `\nobibliography*` builds the bibliography where it is called** (RUST-ONLY shape: Perl 0.8.8 does not
  load bibentry — Error undefined `\nobibliography`; 68s review finding 3): bibentry.sty `\BR@nobib` reads the `.bbl`
  inside a group with `{thebibliography}` redefined to record entries only; ours builds a full `<ltx:bibliography>`
  at the call, and bibentry's `\AtEndDocument` Warning says no `\bibliography` followed. 2010.07703
  (texpreamble.tex:820, in the preamble) gets its 806 bibitems at the head of the XML, before the cover minipage,
  instead of under `\part{Listings and Bibliography}` (thesis.tex:458-480); the title-ink dedup passes over a
  bibliography ahead of the cover as a workaround (`leading_centered_paragraphs`, OXIDIZED_DESIGN_DIVERGENCES #146).
  RED `index-bib/nobibliography_star_prints_nothing`. **FIXED 78k** (OD #830, KPE #1059): a bibentry binding
  (latexml_contrib bibentry_sty.rs) reads the `.bbl` in store mode, so `\nobibliography*` builds no list and
  `\bibentry` prints the stored entry; guard `perfect_kernel_batch78k::nobibliography_star_prints_nothing`. The
  `leading_centered_paragraphs` workaround (OD #146) stays (base_utilities.rs, not touched in 78k).
- **memoir's `{adjustwidth}`/`{adjustwidth*}` are tagless one-item lists** (RUST-ONLY shape: memoir raw-loaded; Perl
  0.8.8 has no memoir binding, OmniBus; 68s review finding 2): memoir.cls:11267-11297 builds the margin-moving list,
  so a cover set in it is `<itemize><item>` without a tag, and the title-ink dedup, which never enters a list, keeps
  0812.2309's cover copy (article.tex:120-143, `\calccentering` + `{adjustwidth*}`): its title shows on both
  covers plus the structured `<h1>`. Fix in the binding: the transparent body of
  `latexml_contrib/src/changepage_sty.rs` (`{adjustwidth}`/`{adjustwidth*}`, OXIDIZED_DESIGN_DIVERGENCES #341), today
  defined only when changepage is loaded. RED `class-bindings/memoir_adjustwidth_star_is_not_a_list`.
- **sn-jnl's swapped `\href{text}{url}` without the `pdflatex` option** (RUST-ONLY by OXIDIZED_DESIGN_DIVERGENCES #654,
  68w r2): sn-jnl.cls:1462-1467 makes `\href` breakurl's `\burlalt` (first argument shown, second linked); the binding
  keeps hyperref's order, which 10 of 12 sampled papers write (2405.04967, 2207.12959, 2603.09685), so the template's
  `\href{here.}{https://…/Datasets_-_RMSC/…}` (2401.05832) shows its URL as the link text, 2 `Error:unexpected:_`.
  Needs a ruling: keep it (hyperref's reading, as pdflatex with `[pdflatex]` errs), or a per-call reading of which
  argument is the URL (a text test, against the 2026-10-16 principled-first rule). RED
  `class-bindings/sn_jnl_href_swapped_order`.
- **thm-restate's `restate=` key in a theorem's optional argument** (RUST-ONLY gap of 68w's thmtools keys; 2108.11527,
  2602.09462 −243 words of key text): thm-restate.sty's `restate=<name>` (the theorem stored for `\<name>` /
  `\<name>*` later) is not one of the keys `thmtools_use_keys` reads (`label`, `name`, `continues`), so the argument is
  the note: "(restate=…)" in the title, the restated copy undefined. FIXED 77v: the key stores the theorem as
  `{restatable}` does (KNOWN_PERL_ERRORS #1006; repro package-bindings/thmtools_restate_key, guard
  `perfect_kernel_batch77v::thmtools_restate_key`).
- **A restated theorem prints a reference, not its statement** (SHARED, Perl the same for `{restatable}`:
  thm-restate.sty.ltxml:28 `\lx@thm@restate{}` = "See \ref{restatable:#1}"; 77v review finding 1): thm-restate's
  `\<name>`/`\<name>*` (thm-restate.sty:103-184 `thmt@restatable`: `\thmt@stored@<name>` re-typesets the collected
  body) restate the theorem in full, its head and statement again, numbered as the original; ours print "See ⟨ref⟩" —
  the restated statement text is lost against the PDF wherever a paper restates (`{restatable}` and the `restate=` key;
  2108.11527, 2602.09462, 2402.14202). **FIXED 78i** (KNOWN_PERL_ERRORS #1048): the store collects the body, `\<name>*`
  replays it numbered as the original, `\<name>` as a new theorem, and `{restatable*}` (r2; 1702.04225) prints its
  theorem with the later `\<name>`'s number by reference; guards `perfect_kernel_batch78i::{restated_theorem_prints_its_statement,
  restatable_star_takes_the_later_number}`, `perfect_kernel_batch77v::thmtools_restate_key` (the restated statement).
- **`\ifinner` is false in a display's array cell** (SHARED, Perl the same; 77k review Q3): an array cell's math is a
  formula of its own, so TeX's `\ifinner` is true there, inside `\[…\]` too (pdflatex "aI"; tex.web §1138-1145); we
  and Perl report false ("aD"). `\ifmmode` false at a cell's first token is TeX's own (it expands that token looking for
  `\omit` before inserting the template; Perl says true there). Effect: pstricks sets a box in a display's array cell
  in `\displaystyle` where pdflatex uses `\textstyle` (KPE #989 `\lx@ps@mathflag`). Fix: the alignment cell's math
  mode is the inline one. RED `kernel-alignment/ifinner_in_a_display_array_cell`.
- **A `}` meeting a formula a macro opened inside a unit ends the formula** (RUST-ONLY kernel recovery; 77k r2 review
  finding 1): TeX deletes the brace (tex.web §1069 `extra_right_brace`) and the macro's own `$` closes the formula;
  our OD #553 rule (`formula_ends_with_its_unit`, stomach.rs ~:3542-3565) ends the formula at the unit's `}` and reads
  the brace again, so the groups after it are off by one — `$\psframebox{$q$}$ after text.` gives 7 errors and
  " after text." as math, where latex gives 2 errors and text. Fix: delete the brace (TeX's branch) when the formula
  was opened by a macro that supplies its own closing `$` (e.g. a flag `\lx@ps@mathbox` binds and the `deleted` test
  reads). Reach: none in 115,456 zips. RED `boxes-groups/extra_brace_meeting_a_macro_formula_ends_it`; the guard
  `perfect_kernel_batch77k::pstricks_math_box_keeps_its_group` pins the count only. Related NITs (77k r2 review
  2-3): the inner formula of an unwrapped pstricks box keeps `tex="\textstyle{…}"` (KPE #989's inserted style);
  REVTeX 3 prints every `\preprint` where aps.sty v3.0:961-965 / v3.1:965-968 keep only the last
  (`\def\@preprint` each call; cond-mat9601115, 2 of 305 preprint-style papers, base the same).
- **A document-level `\def\@abstract{…}` read only by a refused class `\maketitle` is no `ltx:abstract`** (residual by
  user ruling 2026-10-19; HF14, 2609.12818): the document sets the class's store itself, so no setter call reaches K11
  (class setters only) or OD #767's off-table rule (a setter the document calls); the text survives inside the
  title page's tcolorbox picture (Perl drops it). Not rerouted — K11 stays class-setters only.
- **blkarray side labels dropped** (RUST-ONLY binding gap, base too; 77k review probe b3): `\Left{label}{delim}` and
  `\Right{delim}{label}` in a block's preamble (blkarray.sty:1998-2003) print their label beside the block's bracket;
  the binding drops delimiters and labels alike (OD #771), so "M" and "N" are lost with 0 errors. Needs a place for a
  label spanning the block's rows (a cell of its own beside them, or an annotation of the rows). RED
  `alignment-bindings/blkarray_block_side_labels_are_dropped`.
- **espcrc's `\address[<key>]` / `\addressmark[<key>]` do not link an author to the keyed address** (RUST-ONLY binding
  gap, rev78s; Perl prints the key, KPE #1122): espcrc1.sty:159-162 marks the author with the `\address[<key>]` named, and
  espcrc_sty.rs reads and drops both keys, so the marked authors have no `<contact>` where the PDF ties each to its
  address (astro-ph0207534, hep-ph0610439; hep-lat0409072 the witness of the key no longer printed). Next: store
  key → address at `\address[<key>]` and attach that contact at `\addressmark[<key>]`. No repro yet (review probe
  `rev78s/p/r_espcrc_addressmark_key.tex`).
- **wrapfig's `\float@restyle` hook is not ported** (RUST-ONLY, rare; rev78s): wrapfig.sty:286-298 hooks float.sty's
  `\float@restyle`, so a `\restylefloat{<type>}` after wrapfig makes `wrap<type>` for a type declared before it; here
  only `\newfloat` makes it (float_sty.rs) and `\restylefloat` is a no-op (float_sty.rs:11). Found reviewing the
  `wrap<type>` hook of 2411.16612; reach unmeasured. No repro yet.
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
| 2b | merged creators: acmart comma list in one `\author` (`Error:frontmatter:merged_creators`) | 2212.06542 | **FIXED 63x** (KPE #549; HF2) | acmart `\author` binding: split a comma-separated name list (PDF prints them as one block) |
| 3 | eqnarray/align row holding only `\label` dropped with its number | 1011.4399, 1305.3072, hep-th9412215, hep-ph0208046 | **DONE 63j** (KPE #526) | nit: a kept eqnarray row takes a following unnumbered `&=&` row as its continuation |
| 4 | `\ref` (and `\eqref`/`\autoref`/`\cref`/`\nameref`) in a section title empty in the TOC and `title=` tooltips (RUST-ONLY) | 1011.3492, 1111.3672 | **DONE 63k** (fillInTitle port) | residual: a `\cite` in a title reads empty in tooltips (row 12) |
| 5 | IEEE `and~Name` keeps "and" in the last name | 2011.10474, 2408.09035 | **DONE 63j** (KPE #528) | — |
| 6 | `\fnmsep` leaves a stray comma in the name | astro-ph0001054, astro-ph0611016 | **DONE 63l** (KPE #533) | — |
| 7 | PoS/JHEP `\author` calls accumulate, only the last kept | astro-ph0611258 | **DONE 63k** (KPE #529) | — |
| 8 | algorithmic `\REQUIRE`/`\ENSURE` labels replaced by "0:" | 2201.01230 | **DONE 63j** (KPE #527; algpseudocode too) | residuals: `\ref` to a labelled line prints the label (algpseudocodex's label FIXED 77s) |
| 9 | raw class `\abstract{…}` (an.cls) a plain paragraph, no `ltx:abstract` | astro-ph0611016 | **FIXED 77b** (silent-loss #23) | an.cls binding or a raw-class `\abstract{}` command form → the abstract frontmatter |
| 10 | entics.cls (raw) `\thanksref{ALL}` prints its key in the name | 2212.08177 | **FIXED 77t** (OD #790) | an entics/entcs binding reads the shipped class raw with elsart's frontmatter interface (esl.dtx's: `\author[keys]`, `\address[label]`, `\thanks[key]`, `\thanksref`); 2212.08177 and 1812.00040 (entcs) each author holding address and notes; guard `perfect_kernel_batch77t::entics_thanksref_names_its_note` |
| 11 | sectsty `\subsectionfont{…\textit}` applies the font to the first letter only (presentation) | 1508.01140 | **FIXED** (verified on int40c in 78a: `\subsectionfont{\normalfont\textit}` sets the whole numbered title italic; the starred half 78a, OD #785) | sectsty font hooks take the whole title |
| 12 | a `\cite` in a section title reads empty in tooltips (63k residual) | — | **OPEN**, low | tooltip text of a nested bibref without double-counting missing-citation warnings |
| 13 | names continued past a `\\` in an author line read as an affiliation line | 2408.01902, 2408.02464, 2308.07107 | **DONE 63m** | IEEE grades read past a continued `\\` (`names_continue`), compsoc `\IEEEcompsocitemizethanks` a note of the names line, a line of two or more names continues the names; guards `ieee_compsoc_names_continue_past_a_break`, `ieee_grade_comma_before_a_break_continues_names`, `author_names_line_and_address_line` |
| 14 | an affiliation line under a group of names goes to the group's last name only | 0911.0568 (LPT Orsay group), JHEP/PoS group blocks | **DONE 63m** | `\lx@add@affiliation[annotate=N]` for the group's N names (OD #459); a bare address line gives each address to the name it spells; guards `author_group_affiliation_goes_to_each_name`, `author_names_line_and_address_line` |
| 15 | `$(a)$Department …` marked affiliation lines not linked (0 affiliations) | 1011.0516 | **FIXED 77t** (OD #52(p)) | a parenthesized math lead whose mark an author requests labels its line (`parenthesized_mark_line`); 1011.0516's eight authors hold their institutes; residual: the speaker's `\speaker{Etsuko Itou}$^a$` mark after the command requests for no creator (Osaka not on her), its e-mail on the shared creator; guard `perfect_kernel_batch77t::parenthesized_math_marks_label_affiliations` |
| 16 | llncs `\author{A\\Univ\\\email{x}}` reads "Univ" as a person | review probe (rv63k/llncs1) | **FIXED 77t** (OD #792) | llncs.cls:1019-1036 prints `\@author` as one centred paragraph, `\\` a line break, and its `\and` inline (llncs.cls:1013-1018): a line reading as an affiliation is the names' since the last line under names (an affiliation or an e-mail line), across the whole `\author` (in a list without `\inst` marks; 77t r2: 2604.11038's Simon Fraser University on its three authors, the rev77t review's L1, L2, L4); residual: an unmarked `\institute` with no `\inst` requests stays on the block's shared creator (as before); guards `perfect_kernel_batch77t::{llncs_author_line_break_affiliation, llncs_author_line_break_affiliation_rows, llncs_author_line_break_affiliation_per_name, llncs_author_and_is_inline, llncs_author_e_mail_line_ends_a_row}` |
| 17 | newer vmsta copies (`style/vmsta.cls`, `\fnms` name parts) print `\thanksref{cor1}` keys in the names | 1805.02511, 1810.09681, 1912.07948, 2001.02487 | **OPEN** (77t residual; unchanged from the base; 78e made these copies' e-mails and addresses the authors' contacts, OD #802, the key stays) | the class defect rank 92 fixed for entics: `\thanksref` naming its `\thanks[key]` note in these copies (elsart_support_core's frontmatter interface, as entics_cls.rs) |
| 18 | llncs `\author{Ann Able\\ University of X\\ Stanford, CA 94305}`: the address line with no institution word reads as names, creators "Stanford" and "CA 94305" | review probe rev77t/L5 | **OPEN** (77t residual; as on the base) | an address line under an affiliation line goes on with it, not as names |
| 19 | a class numbering its `\institute` entries (llncs's `\institutename`, preprint.cls) with `\institute` before `\author`: the entries annotate every author (`annotate=all`), the numbers unread | the rev77t review | **OPEN** (77t residual) | `class_numbered_institutes` reads the queued authors' marks, none queued yet: number the entries for the authors still to come |
| 20 | an llncs `\author` line printing a link under the names (`\texttt{\href{https://3dlg-hcvc.github.io/EgoFun3D/}{…}}`) reads as a name: a creator holding the URL text | 2604.11038 | **OPEN** (77t r2 residual; as on the base) | a link line under llncs names a URL contact of the row, as the marked block places one (`url_contact_call`) |

### Open rows from the main-stream handoff register (drained 2026-10-17)

The main stream's `~/data/pk_agents/main/HANDOFF.md` was drained here and deleted (user 2026-10-17). The inventory is
in `agent_reports/2026-10-17_handoff_audit.md` and `.tsv`, with the original line numbers. Each row is open unless it
says otherwise.

**Rulings to apply (user 2026-10-17)**
- **Box family stage R6, ADOPTED as its own batch.** An `\hbox` in vertical mode becomes its own line:
  `<p>C1</p><p>C2</p>`, then a separate "After" paragraph, with no logical-block wrapper
  (agent_reports/2026-10-02_box_family_design.md:38-44; KPE #432).
  - The batch needs an A/B (high breadth).
  - It unblocks the queued R1 vertical splice (KPE #434, "elsewhere it needs R6").
- **68f's scan-only `\ver@` skip, CONFIRMED** (OD #605). The dependency scan skips a package that the class's own setup
  already marked loaded. This is distinct from the general loader `\ver@` rule, rejected 2026-10-14. Witness: 2201.11056
  (OOM → 5 E); 0 of 30 sn-jnl net papers worse.
- **66u (K9 row F, author tabular-break split; approved 2026-10-14).** 8 review rounds, never landed (pk/66u at
  1b7360cc1e; worktree ~/data/pk_wt/66u; `briefs/66u_r9.md` never launched).
  - Ruling: re-measure r8 on head. If it rebases cleanly, run a fresh r9 writer before FM S2.
  - Otherwise park it here, keeping the branch.
- **#260 / H1, RESOLVED.** The binding always wins over a shipped copy of a TeX Live file. 68c Part A
  (`shipped_file_is_another_package`) is not landed, and branch pk/68c-loader (a02f9cbecc) is deleted. Author-file
  stand-ins read their copy provide-only (72a, OD #731).

**Kernel**
- **`\stop` is `\let` to `\endinput`** (latex_constructs/sect13.rs:868): the rest of the line is kept and the job
  continues. Perl closes the mouth (`closeMouth(1)`, latex_constructs.pool.ltxml:5750), and latex.ltx:15343 ends the job.
  - RUST-ONLY, LOW. Witness: elzcards (out of scope; `\PKGERROR` then `\stop`,
    agent_reports/2026-10-02_elzcards_geometry_rootcause.md:13-14).
  - Also open from that report: the kernel paper-size swap under `landscape`.
- **`LATEXML_NODUMP=1` broke every guard** in the gates env on 64h4 and 64qb (`Warning:recursion:LaTeX.pool`,
  `\AddToHook`/`\NewDocumentCommand` undefined; agent_reports/2026-10-10_payload_64qb.md:186-190).
  - This is CLAUDE.md's strict-LoadFormat NODUMP branch.
  - Re-verify on head. A fix is a payload.
- **`\newcommand{\span}` gives only an Info, and `\DeclareMathOperator{\span}` is silent,** where LaTeX's `\@notdefinable`
  errs.
  - Populations: 385 `\def`, 215 `\renewcommand`, 39 `\newcommand`, 39 `\DeclareMathOperator`, 10 `\let`,
    8 `\providecommand` (scratch_65b/main_q1/span_forms.tsv).
  - Apply the 2026-10-11 TeX-only-error ruling: a Warning where the recovery keeps good markup.
- **`\lefteqn` ending a cell** should end at the cell end rather than by look-ahead (68z r2).
- **Before fixing rc_struct3 X2,** audit which plain-only registers the LaTeX format carries
  (agent_reports/2026-10-17_rc_struct3.md).
- **Cooperative deadline grace:** write partial output on a `Fatal:Timeout:Convert` instead of none (65f r3 option).
  PAYLOAD.
- **`%!PS`-header detection for the PDF/DVI figure cue,** and pstricks under plain TeX and under expl3 (65f open;
  math0305287, math0111222).

**Bindings**
- **Missing-file scan of arXiv-only gaps** (scratch_356a0c47/rawcls_main/missing_all.tsv): sw20lart 260, ioplppt 205
  (191 E/F; a binding candidate), thmsupp, mncite, aip, ichep, fbssuppl, qcdparis, npbproc. ptptex 827 is tracked at
  its own row.
- **jinstpub.sty** (JINST; not in TL) across `\keywords`/`\affiliation`/`\includegraphics`: about 600 papers
  (agent_reports/2026-10-14_undefined_cs_attribution.md). H1 applies: grow a binding from a released version if one
  exists. **FIXED 78e** (OD #801): the undefined commands were gone on int45a (0 Errors on 2006.05543, 2303.10224), the
  frontmatter still raw (affiliations an itemize, the e-mail a paragraph); latexml_contrib jinstpub_sty.rs is jheppub's
  binding with JINST's extras, from the 2015 and 2020 releases the corpus ships (23 of the 30,000-paper sample); its
  e-mails go to the authors they spell, not by position (78e r2).
- **68j r2 residuals.**
  - newtxmath keeps txfonts names that TL 2025 newtxmath.sty lacks (`\coloneqq`, `\eqqcolon`, `\mathcent`, clockwise
    integrals). This is pre-existing and Perl-shared: a leniency vs pdflatex.
  - The mathtools binding's `\coloneq` is ≔, while mathtools.sty:508 makes it ':-' (`\MATHT@colondash`). Perl is the
    same (mathtools.sty.ltxml:729).
  - newtxmath then pxfonts loses `\varmathbb`.
- **svjour3:** an unmarked `\institute` adds an extra empty creator (68k; pre-existing, sv_support).
- **Design question (67l):** centralize the shipped-copy algorithm scan in the loader (beyond Perl). The reviewer's
  recommendation was not recorded; this is now adjacent to 72a's copy reader.
- **2609.14043:** tikz-timing's `tikztimingtable` gives "No shape named timing@dstart/dend", then `\p`/`\x`/`\draw`
  undefined at :390, then EoF (a deep package; low).
- **2609.07018 (amsart):** `\widetilde\clock`, with `\def\clock{\begin{tikzpicture}…\end{tikzpicture}}` inside a tikz node
  (main.tex:18278-18283), gives a boxing-group mismatch and an undefined `\endscope`. amsmath's `\mathaccentV` takes the
  accentee as a braced macro argument (pdflatex is fine). Not bisected.

**Bibliography**
- **64zf stage 2** (the remaining Error/Fatal classes):
  - the kernel's lonely `\bibitem` (KPE #595);
  - AAS keyless `\reference`, silent;
  - the natbib author-year incompatibility message (0906.4717);
  - the missing `\item`;
  - natbib's soft egroup.
- **64x r7 follow-ups:**
  - AAS label fidelity per class and cite mode (AASTeX6+/emulateapj numeric `[N]` vs unlabelled entries);
  - citations to unlabelled entries;
  - the AAS `\references` title;
  - ACM sigconf numeric shown as author-year under ar5iv (FIXED 78c, `\citestyle{acmnumeric}`: `[N]` with
    `show="Number"`; re-verified on int45a by the sel78d study).
- **64u ruling-4 check, DONE.** The `\address` cluster (765 run-336 papers): 38 of 40 sampled ship no
  opex3/ol2/osajnl2, and TL 2025 lacks them. pdflatex on 1210.7592 gives "File opex3.sty not found", then an emergency
  stop. Papers that ship the style (0901.2165, 0901.2997, 0707.0735, 0901.2793) raw-load clean, and we already log
  `Warning:missing_file`, so there is no generic change.

**Alignments and floats (REDs to file)**
- **Longtable caption rows (65h r2) — re-probed int27:** the `\multicolumn{3}{c}{\caption{…}}\\` and one-column `{l}`
  shapes keep every word (the caption's row holds nothing else; pdflatex errs "Misplaced \noalign" on the
  `\multicolumn` one, ours stays silent — parity residual). FIXED int27: `x & \caption{…}\\` lost the cell before the
  caption with 0 errors; the row now keeps it, with LaTeX's "Misplaced \noalign" (longtable_sty.rs
  `\lx@longtable@caption@row`; guard `perfect_kernel_batch65h::longtable_caption_after_a_cell_keeps_the_cell`, repro
  captions-floats/longtable_caption_after_a_cell_keeps_the_cell). Open: the pasj fallback `\caption[N]` writes N as
  the toccaption.
- **1310.1926:** the emulateapj head line lands in tbody (68y r2).

**Frontmatter (re-probe each on head first; check against FM S2/S5)**
- **Fidelity round-1 residuals** (scratch_356a0c47/fidelity/audit.tsv):
  - amsrefs source-order numbering.
- **Singletons:**
  - (a) subcaption `{subfigure}` sets no `\linewidth`, so `\includegraphics[width=\linewidth]` gives 345pt. SHARED;
    subcaption.sty:70, 99; 2407.02944, html_feedback #6128; repro scratch_59s/listings_issues/repro/subfiglw.tex.
  - (b) The author splitter cuts a `\\` inside `\begin{center}` in `\author` (2609.17357).
  - (c) Alt text from a digested acmart `\Description` lacks the `` ligature (KPE #535 residual).
  - (d) A dangling author mark with no affiliation is dropped silently (Perl too;
    `\author{\bfseries Ann\textsuperscript{1}}`). 47 JMLR-sample papers keep `@` in a personname, plus other shapes.
  - (e) `\newcommand{\tab}{&}` in an author tabular gives a tabular personname on every binary (unpinned).
  - (f) cs0204041 loads the hyperref binding twice through a local myHyperref.sty.
- **59p class sweep** (scratch_59p/cls/, 299 classes). M3 S5 (in-place display) may subsume both:
  - (a) webquiz: a deposit dropped for showing nothing loses its paragraph end, so `\AtBeginDocument` text and the
    document's emptied `\maketitle` center run into "Body text." in one `<p>`. In pdflatex the center ends the paragraph.
  - (b) tudaleaflet: a kept replay of bullets and rules with every field emptied, because a bullet counts as ink.
- **2301.13443 tmlr.sty:** `\name … \\ \email\vspace{2pt} … \\\vspace{2pt} \addr $^{1}$Texas A\&M…` — the `\addr`
  institutions become creators (pre-existing). The repro needs the paper's tmlr.sty. PAYLOAD.

**Listings**
- **2605.03143:** 1 of 2 leading spaces is lost before an `@\tikzmark{…}@` listings escape. Mechanism unknown
  (scratch_59s/listings_issues/2605.03143). PAYLOAD.

**Sweep #148/#149 side findings (re-probe each)**
- **(a) Converter internals in Math `tex=`:** `\lxSVG@transformcm` / `\lxSVG@begingroup@{transform=matrix(…)}` leak in
  the mhchem and zx-calculus manuals (s148).
- **(b) Dimensions leak as text** in 7 s148 manuals: fncychap "24pt 28pt", pgfornament-han-doc, latexcheat-esmx,
  pldocverb, biblatex-gb7714-2015, nathguide, floatrow-rus. 5 are genuine per P0 r5.
- **(c) `\divide` by 0:** mathfam256, scsnowman-sample, bxbase-ja and chemobabel gained
  `Error:misdefined:0 Illegal \divide by 0` from s147 to s148.
- **(d) Inline-block widths:** pgf-interference has 8 inline-block widths going 1225.2pt → 0pt in s149.
- None of these was checked in agent_reports/2026-10-11_sweep149_triage.md.

**Repro harness (input for row T1)**
- **33 GREEN repros fail their `% expect:`** in the 2026-10-02 expectation check
  (`~/data/pk_agents/main/scratch_repro_sweep/{expect_all.tsv,green_fails.txt,red_meets.txt}`):
  - babel poly/sysengine junk before Latin;
  - blx_printbibliography_sbl/datainherit without `<bibliography>`;
  - blx_citechain `\citeshorthand` empty;
  - clefval `\csname` value;
  - isomath `\vectorsymbol` not Math;
  - pgfmath stray "1.0";
  - beamer size9/colorbox/`circle]`;
  - labelformat without tag;
  - ryethesis without titlepage;
  - ejpecp logo math;
  - kotex U+02D9;
  - ctex `\draw_linewidth:n`;
  - montex ₮;
  - picture_direct_tcolorbox;
  - examdesign "True".
- **5 RED repros meet their line:**
  - math-parse differential_needs_its_integrand and product_before_a_big_operator;
  - calc_scbox, multicols_afterpackage and xetexprobe_xevlna, which are SHARED pins and become CONTROL.
- **`tools/perfect_kernel/repros.sh`** `errs()`/`first()` read the class with `[A-Za-z_/]+`, so `Error:I/O:` counts
  (fixed 2026-10-17).

**Stability (STABILITY_WITNESSES.md watch list)**
- **Unreproduced segfault:** 2310.12904, once, in the 65e r3 review.
- **Intermittent SEGFAULT under host load:** 64zf-rel on couleurs-fr-doc-prefix, the 65a binary on eigo. Standalone
  reruns are clean.

**Queued studies (main-session queue, 2026-10-17)**
- **70c:** done on pk/70c (OXIDIZED_DESIGN_DIVERGENCES #750; the 70c bullet below).
- **rc_harness:** re-open every "discount" or "harness limit" row: mnras→atbegshi 63, bull-ppt 98, mathalfa 25, the
  rc_struct2/3/4 harness papers, and row G's clusters.
  - User ruling 2026-10-17: TL 2025 build failure means skip the PDF checks only; there is no discount.
  - So row G's "Faithful (pdflatex errs too)" clusters are targets: ctex fandol 107, l3backend twice 71, acro `class`
    52, `\DeclareMathOperator{\free_var}` 155, `\verb` misuse 183, JINST.cls under TL 2025 39.
  - So are mathalpha's old names `\mathbcal`/`\mathbscr` (25 papers; mathalpha.sty:1107 defines them only under
    `oldbold`).
  - Each needs a check of the XML content against the `.tex`.
- **rc_struct5 kernel batch (Z1 V1 C2 K2 D3 D4) and structure batch (K1 A1 G1 F1)**
  (agent_reports/2026-10-17_rc_struct5.md). Main-session rulings:
  - S1: pgfplots `shader=interp` declares the driver capability honestly, so pgfplots takes its own flat-shading
    fallback.
  - G2: glossaries emits definitions for used entries only, as `\printglossary` prints.
  - H2: epsbox — obtain the original source first, else a row.
  - F1: an `[H]` algorithm in an inline box follows the P1 precedent (`ltx:inline-logical-block` around the float).
- **rc_struct6** (briefs/rc_struct6.md; scratch_356a0c47/proj98/rc6_papers.tsv): the largest sole-cause clusters of
  the int20 residual. `_` 59, `\errmessage` 29, `&` 27, Match 23, `\noalign` 21, section 22, `}` 16.
- **patchcmd study** (briefs/patchcmd_study.md; user idea `\lx@patched@<name>`; covers rc_struct5 E1).
- **First-mechanism cascade study:** `\endgroup` 131 any / 5 sole, `\lx@begin@alignment` 54, TooManyErrors 46,
  `misdefined:#` 33 (int19 census).
- **physics `\fbraces`/`\lparen`/`\rparen`** (physics.sty:158, :31-32; witness 2005.03021): the int20 open (d) row.
