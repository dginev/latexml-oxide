# latexml-oxide 0.8.0: the full list of changes

The release summary is the `## [0.8.0]` section of [`CHANGELOG.md`](../../CHANGELOG.md). This file keeps every entry
the release's pull requests wrote, with its guard tests and design references (`OXIDIZED_DESIGN_DIVERGENCES #N` is
[`docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md`](../parity/OXIDIZED_DESIGN_DIVERGENCES.md)). A fix that lands between
the release candidates and 0.8.0 adds its entry at the top of the list below.

## The perfect-kernel campaign (2026-08-31 to 2026-11-13)

Most of the work of September and October merged into main at once (`6df4632fb1`, "Merge branch 'perfect_kernel'"),
so the list below holds one entry for it. The campaign's record:
[`docs/archive/PERFECT_KERNEL_2026-11-13.md`](../archive/PERFECT_KERNEL_2026-11-13.md) (the goal bar and the closing
sweep), [`docs/perfect_kernel/KERNEL_CAPABILITIES.md`](../perfect_kernel/KERNEL_CAPABILITIES.md) (the engine
capabilities it added: the `texlua` bridge and the engine persona, the pTeX/upTeX profile, the pdfTeX byte reader,
lthooks as the one hook store, the frontmatter event model) and the OXIDIZED_DESIGN_DIVERGENCES entries from #161 on.

## New bindings

Bindings for 198 more packages and classes than 0.7.6, none of which Perl LaTeXML ships (`latexml_package/src/lib.rs`
and `latexml_contrib/src/lib.rs` against the 0.7.6 tag). Four were retired, their packages now loading from their own
sources: alphabeta, atableau, catoptions and textalpha.

**Classes (68):**
aero, aguplus, alggeom, amsdtx, amsldoc, an, apa, apa6, apa7, appolb, asmejour, beilstein, bmcart, brief, CECS,
cimento, cjs-rcs-article, colt2025, colt2026, compositio, entcs, entics, epl, epl2, eptcs, examdesign, extarticle,
extbook, extreport, g-brief, g-brief2, gtpart, hha, hld, ijsra, iucr, jcm, jhrs, JINST, jpconf, just, kluwer,
knittingpattern, l-aa, l-aa-ps, l4dc2026, laa, lni, ltnews, ltxdockit, midl, modernposter, moduli, neus2025, pasa,
pasj00, pasj01, pasj02, quantumview, rasti, resphilosophica, rmaa, smfart, SNmult, uiucthesis, uspatent, vmsta, webofc.

**Packages (130):**
aaai, aas2pp4, aaspptwo, abntex2cite, acl, adjmulticol, aistats, aistats2e, aistats2eM, aj_pt4, ajmacros,
algpseudocodex, animate, apjpt4, assoccnt, autobreak, autonum, axodraw, axodraw2, bfhlayout, bibentry, bidi,
bmvc2k_natbib, caption2, cclayout, centernot, clefval, cmpj, cmpj2, cmpj3, codehigh, coling, collcell, colm, conll,
corl, crossreftools, cuted, datatool, dgruyter, dhucs, directory, eacl, emnlp, emnlp-ijcnlp, enotez, eqnarray, etoc,
figbib, floatrow, frontespizio, gauss, hvfloat, hypdestopt, iclr, ifoddpage, imakeidx, ims_support, imsart, isorot,
jinstpub, jurabib, keytheorems, latex-lab-testphase-block, latex-lab-testphase-minipage, ltxtable, lua-widow-control,
luacode, mathspec, mprocl, multibib, naacl, naaclhlt, newverbs, nips, NJDnatbib, nlctuserguide, nmbib, nomentbl, ol2,
opex3, osajnl, osajnl2, pdfcomment, pdfmanagement, pdfmanagement-testphase, phy-ab.braket, piton, polyglossia, pos,
projlib-author, quotchap, ragslab, rccol, robustindex, schooldocs, scrbase, scrkbase, scrlfile, scrlogo, setstack,
shapepar, siamproceedings, spotcolor, sprocl, srdp-tables, stwol, tacl, tagpdf, tagpdf-base, tex-label, textpos,
theoremref, thumbs, titleps, tkz-euclide, tmlr, tocbasic, tocloft, tone, underscore-ltx, unicode-math, updatemarks,
usenix, versonotes, wacv, wrapstuff, ws-rv-har, xeCJK, xkeymask.

## Changes

  - **Run 397's misplaced elements, as DG ruled 2026-11-19.** An undefined BibTeX macro (`month = june`; ~190
    papers) is a Warning, as bibtex has it (OXIDIZED_DESIGN_DIVERGENCES #1096). A theorem or proof in an abstract (86
    papers) is dissolved into it: its title a run-in bold text, its paragraphs, lists and equations in the abstract,
    its id kept (1908.02189, 0804.0728; #1097); an abstract still open at `\end{document}` (math0006149) keeps its
    theorems. nomencl's list in a framed box in a table keeps its box: a
    bold heading and the entries as a description (1410.5511; #1098). A bibliography inside a float or a heading
    argument floats out after it (cond-mat0108156, hep-ph0004227; #1099). Guards `perfect_kernel_batchplacefix::*`.

  - **UnicodeMath output (`--unicodemath`).** Perl's `--unicodemath`/`--nounicodemath` are exposed: each formula's
    MathML carries an `<annotation encoding="application/x-unicodemath">` (`a+b^2=\frac{x}{y}` gives `𝑎+𝑏²=𝑥/𝑦`),
    after the Content MathML and before the TeX annotation. As with `--cmml` and `--mathtex`, Presentation MathML stays
    on in HTML (Perl's `--unicodemath` alone replaces it, and the page then shows the TeX; OXIDIZED_DESIGN #1093);
    with `--nopmml` and no `--cmml` the string is the only math output, as in Perl. `latexmlmath_oxide --unicodemath`
    prints the string (`--unicodemath=FILE` writes it to FILE). The converter now writes scripts as Perl's does:
    sub/superscript codepoints where every character has one (`∫₀¹`, `𝑎⁻¹`, `𝑥'`), `┬`/`┴` for limits
    (`∑┬{𝑖=1}┴𝑛`), a prescript before its base. Unlike Perl, the annotation of `$0$` is kept (KNOWN_PERL_ERRORS
    #1289), `latexmlmath_oxide` does not XML-escape `<` and `&` (#1290), and an nth root reads its degree and
    radicand in order (#1291). **Without the flag too**, the page `<title>` and a reference's `title=` tooltip read a
    title's math through the same converter, so their scripts now read as Perl's: a section `On $x^2$, $a_{i+1}$ and
    $\sum_{i=1}^n y_k$` gives `1 On 𝑥², 𝑎_{𝑖+1} and ∑_{𝑖=1}^𝑛{𝑦_𝑘}`, and hep-ex0309075's page `<title>`
    `𝜋^0⁢𝑒^+⁢𝑒^-` becomes `𝜋⁰⁢𝑒⁺⁢𝑒⁻`. An equation tag in a `title=` reads the same way, its scripts and fonts kept
    (`\tag{$\mathrm{P_1}$}` gives `Equation P₁`, where Perl's `textContent` gives `P1`; OXIDIZED_DESIGN #1094).
    Guards `cluster_cli::unicodemath_output::*`, `unicode_math::tests::*`.

  - **Four more run-397 Error roots where TeX is clean.** Under amsmath, `\[`/`\]` are `\begin{equation*}`/
    `\end{equation*}` as amsmath.sty:2950-2951 makes them, so a mixed pair (`\[ … \end{equation*}`) no longer errs
    (math0306159, math0401181, math0508341), and a `\tag` in `\[` is printed (2008.12392; OXIDIZED_DESIGN_DIVERGENCES
    #1091). etoolbox's `\robustify` of a command that is a macro in LaTeX and a native binding here (`\ref`, `\uline`,
    `\it`, amsmath's `\sum`, `\uparrow`) leaves it alone: `\robustify{\ref}` had lost every reference (1812.10085),
    the others erred "not a macro" (1508.06145, 1702.00233); a TeX primitive or a register keeps etoolbox's Error
    (#1092). The native pgfkeys accessors install only over TeX Live's `pgfkeys.code.tex`, so a paper's own pgf 2.00
    copy runs as written (1108.0463: 270 Errors → 1; 1010.2550, 0710.3603). doublespace's `\setstretch`
    (doublespace.sty:66; physics0003041, quant-ph0005132). Guards `perfect_kernel_batchclustfix3::{
    amsmath_display_brackets, robustify_of_a_native_binding, pgfkeys_natives_skip_a_pgf2_pgfkeys,
    doublespace_setstretch}`.
  - **hyperref's and atbegshi's commands on first use under the ar5iv preload.** arXiv's build loads hyperref into every
    paper, so the preload provides, the first time a paper uses one while undefined (preamble or body), hyperref's
    `\texorpdfstring` (a title keeps its TeX string; the PDF string was printed beside it after an Error, hep-th0702068),
    its `\hypersetup`, `\hyperref`, `\hyperlink`, `\hypertarget`, `\phantomsection`, `\autoref`, `\nameref` and
    `\pdfbookmark` (math0202134, 0807.4242, 0803.2070), and the shipout hooks of the atbegshi that hyperref 6.77a-7.01m
    loaded (`\AtBeginShipout`, `\AtBeginShipoutNext`, `\AtBeginShipoutBox` and their siblings; 1503.07687's class,
    2309.06611). The hooks print nothing, except what pdflatex prints on a page (`\AtBeginShipoutUpperLeft` and
    `\AtBeginShipoutAddToBox` inside a hook, `\AtBeginShipoutFirst`), kept once as a publication note of its words (2309.06611's IEEE
    copyright, 2302.00362's conference line). Nothing is predefined, a paper's own or loaded definitions win, and
    nothing changes without the preload. Guards `perfect_kernel_batchhyperprov::*`; OXIDIZED_DESIGN_DIVERGENCES #588.

  - **Five crash and Fatal roots of the 2026 arXiv run.** A counter whose reset list leads back to it
    (`\newcounter{subsubsection}[subsubsection]`, 2308.05200) is reset once per step, with a Warning (an Error where TeX's
    own lists loop too), instead of a silent stack overflow; an amsrefs list under an active `@` (1810.11524) reads its records instead of looping, the
    bibliography mouth reading `@` as data; an alpha label of a year like `1--250, 2023` no longer panics (2308.14725,
    2607.14650), and its year part is BibTeX's, the last two characters (`{99}` gives `KNU99`, not Perl's `KNU`); `\DeclareMathAlphabet{ \mathcal}` (1810.09714, 1906.05222, 2011.04603, 2206.00709, 2209.01842) and
    `\urldef{\wikisurvey2011}` (1312.0976) read their command as TeX does, the first token, so they define it instead of
    a Fatal (xcolor's `\extractcolorspec`, `\DeclareFixedFont`, `\DeclareOldFontCommand` and
    `\DeclareCommandCopy` share the reader). Guards
    `perfect_kernel_batchcrash397b::*`; OXIDIZED_DESIGN_DIVERGENCES #1090.

  - **hyperref's anchors under ar5iv's `\href`, PASJ's `\timeform` without its class, psfig's `\psfigurepath`, `\sep`
    in revtex4's `\keywords`.** The `\href` the ar5iv preload provides now brings hyperref's no-op anchor layer, so a
    shipped algorithm2e.sty 3.6 that takes it for hyperref numbers its lines without three Errors each (cs0602002); a
    pasj00 paper without the class sets `\timeform{2.5'}` as the class prints it, and has `\emissiontype`, the class's
    abbreviations (`\arcsec`, `\micron`, `\fd`) and `\draft` (astro-ph0201169, astro-ph0204505, 0706.0253); `\psfigurepath{a:b}` puts its directories on the graphics search path, so the
    figures resolve (cond-mat0012297; cond-mat0010435's absolute author paths lose only the Error); an elsart-style `\sep` in revtex4's `\keywords` is elsart's
    separator (cond-mat0309530). Each was an `Error:undefined` (an estimated ~230, ~180, ~125 and ~200 papers of the
    2026 arXiv run). Guards `perfect_kernel_batchclustfix2::*`; OXIDIZED_DESIGN_DIVERGENCES #1086, #1087, #1088, #1089.

  - **A source archive's link cycle, a recursion through an argument expansion, and `\verb` in a footnote.** A
    `.tar` source whose links point at its own top (`a -> .`, kept since they stay inside it) hung the search for the
    main file past `--timeout` (the CLI) or into the fleet's timeout: it walks real directories only now
    (`main_tex.rs`, `cortex_worker.rs`). A persistent `cortex_worker` thread no longer logs the archive Warnings of a
    job that failed before its report as the next paper's. An expansion recursing through a balanced read (a macro
    whose `\@ifundefined` argument expands the macro again; the CCJNL397 shape under mfirstuc) overflowed the
    conversion thread's stack, SIGABRT with no log; it is counted by the expansion-depth guard and ends in
    `Fatal:Timeout:Recursion` (TeX: "capacity exceeded"). A `\verb` in an argument digested without its braces
    (`\footnote{a \verb+x+ b}`) reports TeX's "\verb illegal in argument", as pdflatex does (Perl is silent); under
    bigfoot, whose footnotes take `\verb`, the note is read live and the verb is kept verbatim (2310.18996: 0 Errors,
    was its URLs re-read as text with their `+`). Guards `perfect_kernel_batchharden397::*`, `cortex_worker`
    `archive_tests::*`,
    `perfect_kernel_batch56::verb_inside_pretokenized_argument_unreads_instead_of_running_away`;
    OXIDIZED_DESIGN_DIVERGENCES #1085.

  - **astron.sty's author-year labels, the 1998 aipproc's `\affiliationmark`, elsart.sty's switches, `\verb` in an
    argument.** A `\bibitem[\protect\astroncite{A}{Y}]` label under astron.sty or its copies (EslabStyle, ISOStyle,
    EAS), whose `\cite` the locked kernel `\cite` refuses, prints "A, Y" as the copy's `\citefull` defines it (was
    `Error:undefined:\astroncite`; 0812.2799, astro-ph0001290, astro-ph0008362); the 1998 aipproc copy's
    `\affiliationmark` is its superscript (astro-ph0011502); elsart.sty's options set its `\if@seceqn`, `\if@secthm`,
    `\if@secfloat`, `\if@nameyear`. A `\verb` inside an argument ends at the argument's `}` with TeX's "\verb illegal
    in argument" and the text after it is kept; it read on to the line's end and lost the rest of a table. Under
    `\documentstyle` it reads to its delimiter as tokenized, as latex209.def's `\verb` does, with no error
    (cond-mat0203143 14 errors to 0, its three figures back; math0303012's 23 `\verb` errors to 0). Guards
    `perfect_kernel_batchnext397::*`; OXIDIZED_DESIGN_DIVERGENCES #88,
    #1083, #1084, #1085.
  - **A paper's own old algorithm2e.sty, and forest's libraries.** A shipped algorithm2e.sty of 1996-2005, which
    defines `\algocf@capseparator` and (before 2004) `\algocf@typo` (release 1.16 also `\algorithmcfname`) not at all, captions as that
    copy prints: "Algorithm 1: text", but "Algorithm 1 text" under release 1.16's `ruled` (was an `Error:undefined`
    each; ~200 papers of the 2026 arXiv run, cs0008002, 0711.3765, math0511378). forest's `\useforestlibrary` loads
    the library file, as forest.sty does, except `linguistics` and `edges`, which the binding stands in for:
    prooftrees 0.9.2's `ext.tagging` declares the socket prooftrees plugs into (was `Socket
    'tagsupport/forest/setup' undeclared!`; 2606.20837). Guards `perfect_kernel_batchalgo397`.

  - **Four class and citation gaps of the arXiv run, an Error per use each.** A pasj00/01/02 paper that does not ship
    its class keeps the class's own `\SetRunningHead`, `\FigureFile` (its figures were the error's text),
    `\lesssim`/`\gtrsim`, solar symbols (`\MO`, `\LO`, `\RO`) and `\authorcite`/`\yearcite`
    (OXIDIZED_DESIGN_DIVERGENCES #1082). The 1998 aipproc.cls's `\author{name}` and `\affiliation{text}` are the
    author and their affiliation (were a "Missing keyval arguments" error each; #1083). A class's starred `\cite*`
    (EslabStyle, ISOStyle) is one citation (the `*` was the key; #88). elsart's `amsthm`, `seceqn`, `secthm` options
    are the class's `\if@…` switches a journal style tests (#1084). Guards `perfect_kernel_batchclustfix::*`.

  - **Rhai bindings: Perl's hook bodies port 1:1, and every diagnostic is located.** The `Stomach` handle has
    Perl's `bgroup`, `egroup`, `begingroup`, `endgroup`, `beginMode`, `endMode`, `enterHorizontal` and
    `leaveHorizontal`. The boxes a definition's `beforeDigest`/`afterDigest` hook returns are kept, as Perl keeps
    them (were dropped); any other value is dropped with one `Warning:deprecated` per binding load, as a primitive
    body's; a parameter type's hooks run for effect only, as Perl's.
    A closure value in a `properties` map is called with `(stomach, args…)`, as Perl calls a CODE value (was
    printed as `Fnanon…`). A script's `Error()`, `Warn()` and `Info()` name the `.rhai` line and column as its
    `Fatal()` does; a failure inside script functions lists each call it passed through; a failing `--preload`ed
    binding is in the conversion's log (was only on stderr), once, and a message logged where no mouth reads a
    source has no `at locator.rs` line. The README, guide and `docs/examples` inline constructors declare
    `enterHorizontal: true`, so the space after one that opens a paragraph is kept. Guards
    `30_script_bindings::{rhai_hooks_port_perls_stomach_methods_and_values, a_rhai_failure_names_its_function_calls,
    a_failing_preloaded_binding_is_logged, a_rhai_diagnostic_names_its_binding_file}`.

  - **Three crashes of run 397 fixed.** A box width stated through a register read with arguments
    (`\makebox[1.5\fontcharwd\font`m]{…}`) in a figure of subfigure panels no longer panics: the panel-grid
    rule skips such registers, and pgfmath errs on one in a coordinate, as TeX does (2105.01581). An
    alignment whose own cell's malformed math closes the math around it (nested `\ooalign` in a tikz label,
    1901.06584) is reverted from its definition with a Warning instead of panicking. titlesec's before-code
    gets the author's title, not the kernel's font wrapper, so ccjnl.cls's `\capitalisefmtwords` no longer
    overflows the stack (2208.03460, 2208.09785, 2209.08255). Guards `perfect_kernel_batchpanic397`.

  - **A Rhai binding's diagnostic names its `.rhai` file.** A failure while a binding loads is one
    `Error:script:<file>.rhai` (was `Error:unexpected:` with the text twice), located at the line and
    column in that file and then at the `\usepackage`; a body's error and a `Fatal(…)` carry the same
    locator; types read as Rhai's (`string`, `int`, …). Every `Fatal:` line has one space before its
    message, as Perl's. Each external program a binding runs is named by an `Info:script:Command` line;
    `--help` and `docs/USAGE.md` say how a `.rhai` binding is found and how
    `LATEXML_DISABLE_SHELL_ESCAPE` refuses commands. Guard
    `30_script_bindings::a_rhai_diagnostic_names_its_binding_file`.

  - **Dependency cleanup before v0.8.0.** `once_cell`'s `Lazy` is now `std::sync::LazyLock` throughout, and the direct
    `once_cell` dependency is gone from every crate; `deny.toml`'s advisory notes are corrected. No output change.
    For a Rust binding crate: `latexml_engine::prelude` now re-exports `LazyLock` in place of `Lazy`.

  - **An author's printed address list no longer crashes the frontmatter reader.** A list whose
    local parts share a suffix (`\{abcde,\,xy\}.nju@gmail.com`) under marked names made the
    address-mark reader step by the length of a local part the text does not spell, into the
    thin space `\,` prints, and panic (2609.00833, a Fatal). It now steps past the next separator
    only. Guard `perfect_kernel_batch106a::a_brace_list_local_part_the_text_does_not_spell`.

  - **TeX Live's package manuals convert clean (the perfect-kernel campaign).** Every one of the
    1,552 in-scope TeX Live documentation bundles converts with no Error but of a ruled-out
    kind, valid against the schema (or only in a ruled class), at least 95 % word recall against
    its source (or a gap classified as no conversion loss) and in under 180 s at 8 GB; of the
    1,602 manuals, 1,592 convert clean, recall mean 96.89 %, median 99.5 % (closing sweep #186,
    merged as `6df4632fb1`). The work is kernel fidelity under raw interpretation of the
    packages' own `.sty`/`.cls` files. Goal bar and basis: `docs/archive/PERFECT_KERNEL_2026-11-13.md`.
  - **An author mark finds its note by the number it shows.** A footnote mark answers the note
    that prints the same number, never one that merely shares its glyph, and the class bindings
    compute the title marks and footnote steps their class prints. A note with no mark shows its
    text with no invented dagger, and a symbol legend under the names belongs to the names that
    print its symbol. Guards `perfect_kernel_batchfmmarks::{shared_notes_are_a_marked_copy_per_carrier,
    a_mark_on_a_name_is_a_copy_of_its_note}`, `perfect_kernel_batchinvdag::a_date_note_and_an_abstract_thanks_invent_no_glyph`
    (OXIDIZED_DESIGN_DIVERGENCES #990, #1052–#1055, #1057–#1064, #1068; #895).
  - **Contacts printed in a title note reach their authors.** IEEEtran/ieeeconf `\thanks` are
    routed by kind: a note that names authors who "are with …" gives them their affiliation and
    e-mail, a "Corresponding author: …" note goes to the author it names, and funding or other notes stay
    title notes. spconf/Interspeech `\address` lines go to the names showing their mark, and Interspeech.cls
    `\email` lines to the authors their local parts spell. elsarticle keys reach their entries;
    a `\date` that holds the block's labelled lines becomes their contacts, and its other notes
    become title notes (2605.04795). Every case the rule cannot place completely keeps the note
    whole. Guards `perfect_kernel_batchieeeroute::ieee_is_with_notes_route_to_the_named_authors`,
    `perfect_kernel_batchspconf::{spconf_marked_address_lines_go_under_the_names,
    interspeech_email_line_goes_to_its_authors}`, `perfect_kernel_batchdateleg::a_date_its_legend_is_taken_from_is_void`
    (OXIDIZED_DESIGN_DIVERGENCES #1067, #1071–#1073; #895, #897).
  - **A note the class prints under the names sits before the abstract.** A binding declares
    such a note (`\lx@names@note`, checked per class against its title page); every undeclared
    note keeps Perl's place. Guard `perfect_kernel_batchblockplace::a_block_contact_note_sits_under_the_names`
    (OXIDIZED_DESIGN_DIVERGENCES #1074; #895).
  - **A ragslab.sty binding; achemso and sn-jnl print their classes' marks.** ragslab (the
    FAIR/Meta template's package) prints its names, marked affiliations, contributions, abstract
    and metadata lines as frontmatter. The achemso and sn-jnl bindings (classes Perl does not
    ship) follow the same note and mark rules, and a paper's own copy of the class in a
    subdirectory keeps its title marks. Guard `06_cluster_frontmatter::frontmatter_ragslab_marks_link_affiliations`
    (OXIDIZED_DESIGN_DIVERGENCES #1065, #1066, #1070; #895).
  - **Author notes in the margin.** The base stylesheet's hover handles draw ⓘ, never a
    footnote-mark glyph that collides with printed marks. On wide ar5iv screens marked notes,
    the title's publication metadata included, stack in the side margin, and a shared note's
    later marks highlight their margin note; on narrow screens each mark pops up its own copy
    (OXIDIZED_DESIGN_DIVERGENCES #1069, #1075; ar5iv-css #67–#74; #895).
  - **Notes a document adds to a class's title-page footnote list print.** acmart, elsarticle,
    REVTeX, amsaddr and icml keep lists of title-page footnotes (`\g@addto@macro` targets) that
    the bindings never defined, so a paper's additions were lost. The bindings now define them
    where the files do, and each list runs where its class prints it, each `\footnotetext` a
    title note (KNOWN_PERL_ERRORS #1283; 2402.08073's icml "Correspondence to:" line). OmniBus binds
    `\orcid` (2606.13132), and pdfcomment's binding allocates soulpos's registers. Guards
    `perfect_kernel_batchregbind::{bindings_define_the_macros_their_files_define,
    bindings_print_the_title_page_lists_their_files_print}` (#891).
  - **icml's `\icmlsetsymbol` leaves the command it is named after alone.** It defined
    `\<name>` as well, so `\icmlsetsymbol{dagger}{$\dagger$}` made `\dagger` call itself; it now
    defines only `\@icmlsymbol<name>`, as icml2026.sty:465-467 does (2606.08980). Guard
    `perfect_kernel_batchicmlsym::an_icml_symbol_named_after_a_command_keeps_the_command`
    (OXIDIZED_DESIGN_DIVERGENCES #1076; #897).
  - **`\scalebox` reads its factors as numbers** (`{Float}[Float]{}`, as Perl), so a scale
    prints as given (`0.75`, was `0.8`) and a mark set in `\scalebox{0.75}{\Letter}` spells its
    glyph, not "0.75✉" (2606.29928, 2605.23320). An empty or zero `[yscale]` falls back to the
    x-scale. A scale with a tail TeX does not read (old pstricks' `\scalebox{x y}`, cs0606057)
    drops the tail with a counted Warning. Guards
    `perfect_kernel_batchmarkscale::{scalebox_mark_spells_its_glyph_not_its_scale,
    a_scale_factors_tail_is_dropped_with_a_warning}` (OXIDIZED_DESIGN_DIVERGENCES #1077; #897).
  - **A raw class's labelled title-page field is a publication note.** A store the class's
    title page prints right after its own label, alone on its line (a "Course:" line), becomes
    `<ltx:pubnote name="<label>">`, where its text was lost or misplaced. Guard
    `perfect_kernel_batch77i::offtable_store_typeset_by_the_title_page` (Bruce Miller Q3.13; #884).
  - **A unit with no title shows its publication notes, subtitle and dates in the HTML.** They
    were in the XML but rendered only beside a title (shared with Perl,
    `LaTeXML-structure-xhtml.xsl:480/:751/:790`); a titleless unit's head block now renders
    them in `maketitle`'s order (2006.12592, 1504.03712, els-mrw's `\articletag` papers).
    Guard `perfect_kernel_batch61::untitled_head_metadata_reaches_the_html`
    (OXIDIZED_DESIGN_DIVERGENCES #262; #881).
  - **llncs and svjour3 keyword lists print their class's separator** (upstream LaTeXML
    #2263). llncs `\keywords{Alpha \and Beta}` printed "Alpha Beta"; it prints "Alpha · Beta",
    as llncs.cls:906 defines `\and` there, and svjour3's `\keywords`, `\subclass`, `\PACS`,
    `\CRclass` and `\JEL` print its `$\cdot$` (2004.08441, 2105.07969). Perl shares the bug
    (KNOWN_PERL_ERRORS #1272). **apacite's full cites** (upstream #711): `\fullcite`,
    `\fullciteNP`, `\fullciteA`, `\fullciteauthor`, `\fullciteauthorA`, `\citeauthorNP` and
    `\shortciteauthorA`, in classic citation mode only, as apacite.sty. Guard
    `perfect_kernel_batchqa76a::class_keyword_lists_print_their_and` (#871).
  - **amsppt's heads hold their bodies, its theorem-like blocks take ids, and `\roster` keeps its
    items.** `\head`, `\subhead`, `\subsubhead` and `\specialhead` open their unit until the next
    head at their level (Perl amsppt.sty.ltxml:94-140), where each closed after its title;
    `\proclaim`, `\definition`, `\remark`, `\example` and `\demo` take ids from one `theorem`
    counter (math9201249: `Thmx1`…`Thmx15`); `\Refs` is back matter. Every `\roster` was an
    empty list (Perl too, KNOWN_PERL_ERRORS #1270): its items are kept, with amsppt's
    `\item"label"`/`\item[n]` (53 of 63 census papers recover 1,178 items; math0309287).
    Guard `perfect_kernel_batchamsppt::amsppt_heads_and_theorems` (KNOWN_PERL_ERRORS #1269,
    OXIDIZED_DESIGN_DIVERGENCES #456; #868).
  - **A preamble-only command used in the body raises LaTeX's Error.** `\begin{document}`
    walks `\@preamblecmds` where latex.ltx does (:9521-9522): each listed command used later
    gives "LaTeX Error: Can be used only in preamble" as pdflatex, then runs as Perl recovers;
    before, 6 of them erred and `\documentclass`, `\DeclareTextCommand`, `\listfiles` or a
    second `\begin{document}` passed silently. A body package's `\@onlypreamble` counts, and
    nothing errs inside a package load or a standalone/subfiles sub-file's preamble. A missing
    class's fallback no longer leaves `\@currext` = `cls` in the body. Guard
    `error_boundary::perfect_kernel_error_boundary` (OXIDIZED_DESIGN_DIVERGENCES #226; Bruce
    Miller QA90; #867, #884, #891).
  - **Native pgfmath computes when digested, not inside TeX's optional-space scan.**
    `\pgfmathparse` and the `\pgfmath<fn>@` functions computed when expanded, so after a unit
    (`\dd=2pt%⏎\pgfmathparse{\dd+3pt}`) they read the old value: 8.0 for pdflatex's 5.0
    (tex.web §443-§448). They now define `\pgfmathresult` when digested (2606.14412: 55 Errors →
    0, its curve where the PDF draws it; arrows.meta Stealth tips in pdflatex's proportions).
    Perl shares the bug (KNOWN_PERL_ERRORS #1274). Guard
    `perfect_kernel_batchdtl1::batchdtl1_pgfmath_rows` (Bruce Miller QA11; #872).
  - **`\parbox` keeps the page's `\textwidth` and `\columnwidth`.** It set them to the box's
    width (Perl too, KNOWN_PERL_ERRORS #1273), so `\parbox{0.44\textwidth}{\includegraphics[width=0.44\textwidth]{…}}`
    gave 66.8pt for pdflatex's 151.8pt (1412.0314, 1711.09469); it sets only `\hsize`,
    `\linewidth` and `\@totalleftmargin`, as latex.ltx's `\@iiiparbox`. Only floats modelled
    on `\@xfloat` reset `\hsize`, so a subfloat (subcaption, subfig, subfigure) keeps the
    enclosing width (2010.06608, 2007.14228). Guard
    `perfect_kernel_batchqa14a::parbox_sets_only_the_line_width` (Bruce Miller QA14; #873).
  - **`\leaders` with a rule is the leader itself** (issue 805). `\hrulefill`-style leaders
    were an `ltx:text` around an `ltx:rule`; they are `<ltx:rule class="ltx_leader">`
    (`ltx_filled_leader` in a box of known width), as in TeX a rule after `\leaders` is never
    typeset on its own (tex.web §1078, §1084), and a binding hook that closes the text first
    no longer raises `Error:malformed:ltx:text` (BookML). Perl shares the defect
    (KNOWN_PERL_ERRORS #1282). Guard `perfect_kernel_batch58::leaders_rule_keeps_the_paragraph` (#883).
  - **`\not` moves its operand as Perl does.** A nested `\not` at any depth, and `\not\not=`,
    give `≠`/`∉` where they gave a bare `／`, a plain `=` and an invisible times, and a
    cancelled operand keeps its size (1009.4552, 1611.10101, 1701.02906). Guard
    `perfect_kernel_batchnotcancel::not_cancel_moves_its_operand` (#887).
  - **fbox and listings frames have their padding; `array[t]`/`array[b]` align by their
    baseline** (upstream LaTeXML master `0db6f47e`). `\fboxsep` pads `\fcolorbox` and text
    `\boxed`, a listing frame takes its padding from `framesep`, and `frame=tb` and the other
    letter forms are drawn (upstream LaTeXML #2820); a `frame=` that names no side draws none, as
    listings.sty (OXIDIZED_DESIGN_DIVERGENCES #1038). `array[t]`/`[b]` give `<mtable
    align="baseline1">`/`"baseline-1"` (upstream #2875); `\lozenge` and `\blacklozenge` have
    pdflatex's width (upstream #2845). A `.zip` destination keeps an existing log, its member named by
    `--log`, and EPUB keeps its log outside the container (upstream #2855, OXIDIZED_DESIGN_DIVERGENCES
    #1037). Guards `perfect_kernel_batchups1::{fboxsep_pads_fcolorbox_boxed_and_listing_frames,
    mtable_align_is_the_first_or_last_baseline, lozenge_widths_are_msam_glyph_widths}` (#845).
  - **Faster conversions, byte-identical output.** A performance pass: typical arXiv papers
    take a median 10.4 % fewer instructions (5.8–26.7 %), the issue-790 book 18 % fewer with its
    peak memory 6.7 → 4.3 GB, and 2308.04512 (447 pages) 16 % fewer; every output file is
    byte-identical. The levers (one math grammar per streamed conversion, cheaper `\global`
    undo, shared math subtrees, token-path and XPath allocations) are in
    `docs/performance/PERFORMANCE.md` (2026-11-15). libxml 0.3.24 and libmarpa-asf-sys 0.3.1
    from crates.io (#885).
  - **Large math documents post-process in linear time** (issue 802). Each XMDual collapse
    renamed its references with a whole-document XPath, quadratic in the number of collapses
    and failing past libxml2 2.15's 10,000,000-node limit ("XPath evaluation failure", then
    `Fatal:TooManyErrors`); one idref index now serves every collapse (2,000 collapses:
    259 → 24 G instructions). Shared with Perl (KNOWN_PERL_ERRORS #1248). Guard
    `perfect_kernel_batchi802::collapsed_dual_references_take_the_branch_id` (#865).
  - **A math-parse use-after-free is gone** (issue 790). A discarded math subtree could hold
    the document's insertion point, so the next formula walked freed memory: the issue's book
    exited with SIGSEGV. As Perl's `removeNode` (Document.pm:1783-1792), a discard moves the
    point out first. Guards `02_unit_replace_tree::discard_subtree_keeps_the_insertion_point_out`,
    `06_cluster_math::cluster_discard_keeps_the_insertion_point_out` (#876).
  - **`\includegraphics[page=N]` of a vector PDF, N ≥ 2, renders with mutool.** The output was
    sought under the source page's number while mutool numbers its own output pages from 1, so
    such a page always fell back to pdftocairo (a different size) and left a stray
    `.mutool_svg1.svg` in the destination. SVG cache keys are salted once, so every SVG is
    re-rendered by one converter (2305.12780, 2512.04096). Guards
    `graphics::tests::vector_svg_of_page_two_is_page_two`,
    `graphics_cache::tests::svg_keys_are_salted_and_png_keys_are_not` (#886).
  - **`cortex_worker` leaves no temporary file behind.** Each slot's child runs with its own
    `TMPDIR` (and ImageMagick's), emptied before every respawn, so a killed conversion's files
    go with it; the graphics cache removes a failed install's tmp file and prunes old ones.
    Output is unchanged. Guards `cortex_worker_exit_reports::a_respawned_slot_starts_with_an_empty_tmpdir`,
    `graphics_cache::tests::{a_failed_install_leaves_neither_tmp_nor_dims,
    a_prune_reclaims_the_tmp_files_of_killed_installs}` (#888).
  - **`cortex_worker` built for AVX2 no longer crashes on its first connect.** The bundled
    libzmq allocated 64-byte-aligned queue chunks with plain `malloc`; `.cargo/config.toml` now
    passes `-DHAVE_POSIX_MEMALIGN=1 -DZMQ_CACHELINE_SIZE=64` (CorTeX KNOWN_ISSUES L-3; #819).
  - **A note several authors share prints once.** The XML keeps a copy of a shared author note
    (an authblk `\affil[*]`, a legend line's `*`) on every author, each with its `mark`, as Perl
    does; the HTML prints the first copy (class `ltx_note_shared`) and, at each other author, its
    mark linked to it. A marked note sits right after its author's name, in
    `ltx_annotated_personname`. An unmarked author note is no one author's: IEEEtran's `\thanks`
    prints no mark, so its funding or "manuscript received" note keeps its pop-up and is not
    shown under a name. Only an author's copies merge, and never an image-only note. Guards
    `integration::a_shared_author_note_prints_once_and_its_repeats_are_linked_marks`,
    `witnessed_css_delta::shared_author_note_delta_stays_present` (OXIDIZED_DESIGN_DIVERGENCES #1052).
  - **subcaption papers convert error-free on the current TeX Live 2026.** TL 2026's first aid
    for subcaption v1.6b adds to caption's `\caption@subtypehook`, which the caption binding did
    not define, so every subcaption paper there (lipics-v2021 among them) logged an
    `Error:undefined`. The binding now defines it, and entering a sub-type runs it once as
    caption.sty does: a sub-float, `{subcaptiongroup}`, `\subcaption` or `\phantomsubcaption`, in
    the sub-type's group, keeping what it typesets. Guards
    `caption_subtype_hook::{first_aid_adds_to_the_subtype_hook, subtype_hook_runs_inside_the_sub_float,
    subtype_hook_runs_once_per_sub_type, subtype_hook_material_is_kept}` (#889).
  - **The full test suite passes on Windows.** An EPUB manifest names a resource in a
    subdirectory `sub/x.css`, not `sub\x.css`. An image in the working directory is no longer
    listed twice when the directory's 8.3 short name (`RUNNER~1`) differs from its long name. The
    Windows full-suite job's test-side portability gaps are closed too (path separators,
    case-insensitive `kpsewhich` hits, `file:` URLs); the release workflow runs that suite on
    every tag. Guards `cluster_cli::whatsinout::format_epub_ar5iv_preload_carries_the_css_param`,
    `perfect_kernel_batch56::eps_bounding_boxes_are_read_as_bytes`
    (`docs/release/WINDOWS_SUITE_TRIAGE_2026-11-16.md`; #889).
  - **An author's `\\` lines print under the name** (issue 783). `\author{name\\school\\dept}`
    makes each line a contact, which LaTeXML.css showed only in a hover pop-up behind a blue star.
    Each such contact now carries class `ltx_author_line`, and an author whose contacts are all
    lines or notes shows the lines under the name, as the PDF does; an author with a class's own
    `\affiliation` or `\email` keeps the pop-up, and a `\thanks` or `\footnote` keeps its mark. Offered upstream. Guards
    `06_cluster_frontmatter::frontmatter_author_lines_783`,
    `witnessed_css_delta::author_line_delta_stays_present` (OXIDIZED_DESIGN_DIVERGENCES #1051).
  - **A numeric citation says so** (issue 857). A `<cite>` that shows only numbers or labels
    (`[1]`, `[3, 4]`, `[1, Section 7]`, `[Kho00]`, a superscript citation) carries the class
    `ltx_cite_numeric`; an author-year or textual one (`Bekki (2013)`, `Smith [2]`) does not, so a
    stylesheet can keep a numeric citation on one line without doing so for long author-year
    ones. The form is the one the citation printed, after the bibliography is read (a numeric
    `.bbl` under natbib, an entry with no author). Guard `bibref_show::cite_numeric_class_rows`
    (OXIDIZED_DESIGN_DIVERGENCES #307).
  - **LaTeX's kernel hooks run where LaTeX runs them** (Bruce Miller QA47; #856). Code a document or package adds
    with `\AddToHook` to the hooks of `\end{document}` (`env/document/end`, and the document's own
    code in `enddocument/afterlastpage`, `/afteraux`, `/end`), of `verbatim`/`verbatim*`
    (`env/verbatim/begin|end`), of `\label` (`label`) and of `\include` (`include/before`,
    `include/<file>/before|end|after`, `include/excluded`) was dropped; it now runs there,
    and so do etoolbox's `\AfterEndDocument` and atveryend's commands. A verbatim begin hook reads
    `verbatim` in `\@currenvir`, as in LaTeX. The kernel's first aid for the raw cleveref.sty is
    not registered under LaTeXML's cleveref binding, which replaces the code it repairs. Guard
    `perfect_kernel_batchqa47a::batchqa47a_command_hook_rows` (OXIDIZED_DESIGN_DIVERGENCES #1048).
  - **The abstract, `\selectfont` and file hooks run too** (Bruce Miller QA47; #869). The
    abstract runs `env/abstract/begin` and `env/abstract/end` (latex.ltx:15354-15362, :15386),
    the begin hook before the abstract's name is read, so a hook's `\renewcommand\abstractname`
    is honoured; `\selectfont` runs its `selectfont` hook (latex.ltx:12580); `file/<name>/before`
    and `file/<name>/after` run at every `\input` and load, not once. A family register as a font
    value reads its family, as tex.web §577: `\textfont9=\textfont1` prints pdflatex's "ABCD"
    where it printed "A1 B1 CD" (2511.21986; KNOWN_PERL_ERRORS #1271, shared with Perl). Guard
    `perfect_kernel_batchqa47a::batchqa47a_command_hook_rows` (rows `abstract_env_hooks_run`,
    `abstract_begin_hook_runs_before_the_name`, `abstract_end_hook_once_in_a_constructor_abstract`,
    `selectfont_hook_runs`, `file_hooks_run_at_every_input`, `file_hooks_order_around_a_package`,
    `font_family_register_takes_a_family_font`; OXIDIZED_DESIGN_DIVERGENCES #1048).
  - **The Rhai cheat sheet is a three-way API: Perl | Rust | Rhai, every Perl and Rust cell filled.**
    `CHEATSHEET.md`'s middle column, formerly the compile-time macro alone, names the Rust a compiled
    binding uses for each call: the macro where there is one, then the function, method, type, field or
    trait item the Rhai call itself reaches, each a rustdoc intra-doc link (the sheet is now also rendered as
    `script_bindings::cheatsheet`, so `cargo doc` fails on a renamed item). The Rhai helpers Perl has no single
    call for show the Perl that does the same; four Perl calls point at the Rhai expression that already does
    them (`DefExpandable`, `Float`, `GenerateID`, `TokensI`). Gate
    `script_bindings::tests::api_reference_is_up_to_date` (no empty Perl or Rust cell).
  - **26 more Package.pm calls in the Rhai dialect, with Perl's names and meanings**: `CleanIndexKey`,
    `CleanClassName`, `CleanBibKey`, `NormalizeBibKey`, `CleanURL`, `ComposeURL`, `TrimmedCommaList`,
    `HasKeyVal`, `DisableKeyVal`, `AfterAssignment`, `Deprecated` (Warns once per key), `DigestLiteral`,
    `DefAutoload`, `FontDecode`, `FontDecodeString`, `LoadFontMap`, `MaybePeekLabel`, `MaybeNoteLabel`,
    `CheckOptions`, `Stringify`, `gullet.ReadParameters`, the I-forms `DefConditionalI`, `DefEnvironmentI`,
    `DefMathI`, `DefRegisterI`, and `DocType`, which Warns that no DTD is read (RelaxNG only,
    OXIDIZED_DESIGN_DIVERGENCES #1) and registers its document namespaces. 17 Perl calls remain outside the
    dialect, listed at the end of `CHEATSHEET.md`. Guard
    `script_bindings::tests::package_pm_gap_calls_mean_what_perl_does`.
  - **Text set before `\maketitle` stays above the title** (QA91, Bruce Miller's probe). A
    preprint number, a copyright line or a journal heading the document typesets before
    `\maketitle` came out after the title, the authors and the abstract; it now stays ahead of
    them, as the PDF prints it, wherever the class's title page prints the title at
    `\maketitle`. A class that prints the title and the authors where they are set (iopart)
    keeps its order: text between them and `\maketitle` still follows them, and so does a
    leftover mark that prints no word (a stray `]`, an affiliation mark `*1`). Guard
    `perfect_kernel_batch56::text_before_maketitle_stays_before_the_title`
    (OXIDIZED_DESIGN_DIVERGENCES #262).
  - **A class's title page reads the date or the title as a number** (DEPN, Bruce Miller's
    probe). A class whose title page computed with `\@date` (`\@tempcnta=\@date`,
    `\advance\@tempcntb\@date`, `\ifnum`, `\numexpr`) printed "Exam session 0" with a
    "Missing number" Warning, or lost its title-page block; it now prints "Exam session 2024"
    as pdflatex. Guard `perfect_kernel_batch56::deposit_reads_a_field_as_a_number`
    (OXIDIZED_DESIGN_DIVERGENCES #265).
  - **A paper's own pre-2.0 `lni.cls` keeps LaTeX's `\author`** (#859). The lni binding gave
    every lni paper the class's 2.0 author command (authblk's four arguments), so a paper
    shipping an older class raised an Error, printed its author block a second time and gained
    an ORCID link it never wrote (2203.09493, 1705.02823, 2210.04484). Guard
    `perfect_kernel_batch68u::lni_authors_carry_email_orcid_and_affiliation`.
  - **Every box's width, height and depth are CSS custom properties: `--ltx-box-width`,
    `--ltx-box-height`, `--ltx-box-depth`** (Bruce Miller Q2.4, DG
    2026-11-14). A minipage, `\parbox`, `\makebox`, `p{}` cell, transformed box or wrapfigure wrote
    `style="width:<w>"`; the HTML now writes `style="--ltx-box-width:<w>"`, which LaTeXML.css (and
    ar5iv-css) applies at the inline style's rank, so pages render as before. A section set inside a
    minipage or `\parbox` keeps the box's width the same way (it was dropped). A rule's width and
    MathML's stay `width:`; the XML's `width` attribute is unchanged. Custom stylesheets: an
    `!important` override such as `.ltx_minipage { width: 100% !important }` works as before; a rule
    that matched `[style*="width"]` still matches; one that keyed on `style^="width:"` must now read
    the property, and a stylesheet may reuse its value (`width: min(var(--ltx-box-width), 100%)
    !important`) or switch every box width off (`:where([style*="--ltx-box-width:"])
    { --ltx-box-width: initial !important }`). The widths now need LaTeXML.css or ar5iv-css: they
    are lost under `--nodefaultresources` without a stylesheet carrying the rule, and in readers
    that ignore our CSS or lack `var()`/`:where()`. A box's height and depth go the same way (DG
    2026-11-14): `style="height:<h+d>;vertical-align:<-d>"` is now
    `style="--ltx-box-height:<h>;--ltx-box-depth:<d>"`, TeX's own values, from which LaTeXML.css
    derives the height and the drop below the line (a `\raisebox{y}[h][d]` keeps its shift); a
    rule's height and MathML's stay. Switch every box height off with
    `:where([style*="--ltx-box-height:"]) { --ltx-box-height: initial !important }`, or every depth
    with `--ltx-box-depth: 0pt` the same way. OXIDIZED_DESIGN #1045; guards
    `perfect_kernel_batchqab57::section_in_a_box_keeps_the_box_width`,
    `perfect_kernel_batchqab57::box_heights_ride_the_custom_properties`.
  - **Under the OmniBus fallback, the document's own definitions win over its guesses** (Bruce
    Miller Q3.12, DG 2026-11-14: "the document wins"). A `\newcommand`, `\renewcommand`, `\def`,
    `\DeclareRobustCommand`, `\let` or `\NewDocumentCommand` of a name OmniBus had guessed
    (`\doi`, `\corref`, `\etal`) replaces the guess, as in pdflatex; `\providecommand` keeps it.
    OXIDIZED_DESIGN #408 (extended); guard
    `perfect_kernel_batchqab57::omnibus_guesses_yield_to_the_document`.
  - **latexml.sty takes `pdfoutput` and `nopdfoutput` options** (Bruce Miller Q4.2). They set
    `\pdfoutput` to 1 or 0 over the mode LaTeXML guesses from the source's cues, e.g.
    `--preload=[nopdfoutput]latexml.sty` for a latex+dvips paper. Default output is unchanged.
    OXIDIZED_DESIGN #1046; guard `perfect_kernel_batchqab57::latexml_pdfoutput_options_override_the_mode`.
  - **The document body uses LaTeX's initialized hooks** (Bruce Miller QA47; #862). `\begin{document}` now runs
    lthooks' initialization, as LaTeX does: a body `\UseHook` expands to the hook's code (an
    `\edef` holds the code, not `\hook_use:n`), and code added in the body is built into its hook
    at once. Most documents convert faster (median −6.6 % instructions over 56 papers); acro papers,
    which add to `enddocument` at every acronym use, cost +0.6…+5.6 % — TeX's own rebuild per add
    (300 body adds to one hook: pdflatex 5.25G instructions, LaTeXML 7.81G). Guard
    `perfect_kernel_batchqa47a::batchqa47a_command_hook_rows` (OXIDIZED_DESIGN_DIVERGENCES #1048,
    KNOWN_PERL_ERRORS #1249).
  - **No default `rowspacing="0pt"` on a MathML table** (issue 695). An `array`, `aligned` or
    other multi-row table carried `rowspacing="0pt"`, which MathML Core does not have (Chrome
    ignores it) but Firefox and MathJax honour, setting the rows touching. It is no longer
    written, so they use their default row gap; an author's own row spacing (`\arraystretch`,
    `\jot`) is still written. `columnspacing` and `columnalign`, which LaTeXML.css reads, stay.
    Guard `aligned_mtable_columnspacing_zero::aligned_emits_zero_columnspacing_and_css_resets_mtd_padding`.
  - **A bibliography link drops TeX's escapes** (issue 808). A `.bib` `url` written
    `https://doi.org/10.1162/tacl\_a\_00276` linked to that string, backslashes and all; the
    href now reads `\_ \% \# \& \$ \~ \{ \}` as the character, while a style that prints the URL
    still prints it as TeX does. Guard `06_cluster_bibliography::bib_url_escapes_leave_the_href`
    (OXIDIZED_DESIGN_DIVERGENCES #1039).
  - **An equation's number stays beside its formula at any page width** (issue 810). LaTeXML.css
    gave the tag column no width, so on a narrow page a long formula ran under its number. A
    one-row equation is now centred in the full line, as TeX centres it, and moves aside only as
    far as its number needs; a multi-row table (`align`, `gather`, `eqnarray`) gives its numbers a
    column of their own, centring its rows in the line less that column (10-24 px left at 16 px).
    An overlong formula scrolls with the page instead. In a box narrower than formula and number
    (a `p{2cm}` cell, a narrow minipage) the number still sits behind the box's scroll, where pdflatex
    sets it beside the formula, as before this change (red repro
    `equation_tag_hidden_in_a_narrow_minipage_red.tex`). Offered upstream. Guards
    `browser_render_eqn_tags::equation_tags_stay_beside` (Chrome and Firefox, local) and
    `witnessed_css_delta::equation_tag_and_mathjax_deltas_stay_present`.
  - **Consecutive display equations are not spread apart under MathJax** (issue 475). MathJax
    gives its display container a 1em margin, which inside our equation table cannot collapse
    with the table's own display skip, so two displays sat about three times as far apart as
    natively (MathJax 2.1 in Chrome: 53 px against 18). LaTeXML.css now zeroes that margin in
    an equation cell, in a page's table or a minipage's (21 px). Offered upstream. Guard
    `witnessed_css_delta::equation_tag_and_mathjax_deltas_stay_present`.
  - **A `\bigskip`, `\medskip` or `\vspace` between two paragraphs keeps its gap** (Bruce Miller QA26; #854). The
    second paragraph carries `cssstyle="--ltx-skip-before:12.0pt"` (consecutive skips add up to their
    net, as in TeX), and LaTeXML.css adds it to the paragraph gap (ar5iv-css mirrors it); Perl drops
    the amount. Not after a heading, across a heading, a float or a sibling environment (theorem, proof), in a
    title page, box or float, or for a `\vspace` inside a paragraph; a `quote`, `center` or list inside
    the paragraph does not block it.
    OXIDIZED_DESIGN #1050. Guard `perfect_kernel_batchqab8::paragraph_skips_go_on_the_next_paragraph`.
  - **An aligned left column meets its relation in Chrome** (issues 809, 755). MathML Core
    dropped `columnalign`, and Chrome set the left column of an `aligned` (inline, or in
    `gather*`) flush left in its cell, so the `=` came up to 21 px late. LaTeXML.css now
    right- or left-aligns a cell by its `columnalign` (`justify-items`); the markup is
    unchanged. Guard
    `browser_render_aligned_math_spacing::aligned_left_column_is_right_aligned_against_the_relation`
    (Chrome, local).
  - **`/` has no operator space** (issue 789). TeX's `/` is an Ord, and MathML Core puts it in
    category K (zero space), but our dictionary (Perl's) had it in B and Chrome spaces it
    0.222 em each side. A visible Core-K operator (`/`, `\backslash`, `_`) is now
    `<mo lspace="0em" rspace="0em">` and an Ord to the space walk, so `\backslash` loses its
    0.222 em too; a kept `\mathrel{/}` or `\mathpunct{/}` keeps TeX's Rel or Punct space.
    Guard `perfect_kernel_batchmml2::core_k_operators_have_no_operator_space`.
  - **Small PDF figures with a raster image keep their vector labels** (after PR 816 by
    amachino — thank you). A PDF figure of at most 500 KiB that embeds a raster image
    (a heatmap with axes and labels) was rasterized whole to PNG; it now goes to the
    vector-SVG path like a vector-only PDF (mutool, then pdftocairo), keeping the text as
    glyphs and the image at its own resolution, with the PNG path as the fallback. A
    figure that draws a raster larger than its own pixels keeps the PNG path, where a
    browser would blur it
    (a 1×1 raster never counts: one pixel cannot blur). Guard `graphics::tests::small_raster_pdf_keeps_vector_labels_unless_enlarged`.
  - **The binding macros mean what Perl's same-named API means** (#843). `Digest!` leaves
    the horizontal mode its digestion entered (Package.pm:393-402, through the new `binding_digest`) and `RawTeX!`
    ends in vertical mode (Package.pm:996). **BREAKING (library):** `AssignRegister!` takes Perl's arguments,
    `(cs, value[, scope[, parameters]])`, `cs` a name or a Token — the third argument, formerly the
    parameters, is now the scope.
    `LookupRegister!`, `AssignRegister!` and the core `lookup_register` give Perl's one Warning
    (`The control sequence \X is not a register`, Package.pm:1367-1369); the macros on an undefined cs too
    (`warn_not_register`; the core `lookup_register` stays quiet on an undefined cs, which kernel code probes). `Dimension!` Warns on a spec that is neither a dimension nor a number (the value is
    unchanged, 0). `InstallDefinition!` takes `(definition[, scope])` (it did not compile). `AddToMacro!` on an
    undefined cs is TeX's Error and defines it with the addition, as latex.ltx does (OXIDIZED_DESIGN #1036). Guards `perfect_kernel_batchmacsync::*`.
  - **The bundled CSS follows upstream LaTeXML master (`0db6f47e`).** A listings line
    number is `user-select: none` (upstream `95a77fc0`, #2850), so copying a numbered
    `lstlisting` copies the code without its numbers; dark mode colours `body`, not
    `:root` (upstream `ee4b3640`). `ltx-cv.css`, which the moderncv binding requests
    (Perl `moderncv.cls.ltxml:25`), and the opt-in `LaTeXML-epub.css`
    (`--css=LaTeXML-epub.css`, `\newpage` as a page break) are embedded byte-identical to
    Perl's: both were `missing_file` Warnings with a dangling `<link>`. Mirrored in
    ar5iv-css. Guard `single_binary_smoke::class_and_opt_in_stylesheets_are_served_embedded`.
  - **An fvextra `breaklines` Verbatim wraps and fits a phone** (issue 431, issue 702).
    The bundled CSS's verbatim `white-space:pre` also reached a breaklines line's
    `\parbox`, so its markup newlines rendered as blank rows (a one-row line 60 px, now
    15), and PR #718's `.ltx_framed_verbatim.ltx_break .ltx_parbox { max-width:100% }`
    capped nothing, the line around the parbox carrying an inline width: at 375 px the
    frame scrolled (scrollWidth 468 > 330) and the guard's page was 488 px wide, now 375.
    Lines and xleftmargin's wrapper are capped at their box, a breaklines line is a flex
    row whose parbox shrinks to it (also through the `ltx_text` an id keeps around a
    numbered indented line's parbox), and a framed verbatim is a block; inline `\verb`
    keeps its spaces and wraps (`pre-wrap`), so a long one no longer widens a narrow page,
    except in a table cell, which stays unwrapped as the cell is. A breaklines line's text
    wraps and keeps its spaces (`        return   x` had rendered `return x`), and an
    `xleftmargin` Verbatim sets its margin beside each line, one row per line (issue 853). Witnesses
    2605.00468 and 2605.01024 (their lines' total height 41,496 → 13,650 px and
    1,476 → 432 px at 1280 px). Mirrored in ar5iv-css. Guards
    `witnessed_css_delta::verbatim_whitespace_delta_stays_present`,
    `browser_render_verbatim_narrow` (opt-in).
  - **`\linewidth` in a `p`/`m`/`b` table cell is the column's width** (issue 778). A
    cell of a paragraph column set `\hsize` to the column's width but kept the page's
    `\linewidth`, so `\includegraphics[width=\linewidth]` in a `m{0.3\textwidth}` cell was
    as wide as the page. The cell now sets `\linewidth\hsize`, as array's `\@startpbox` /
    `\@arrayparboxrestore` do (a `\multicolumn` over such a column and longtable's `p`
    columns alike); Perl sets `\hsize` alone. Guard
    `perfect_kernel_batch60::paragraph_cell_linewidth_is_the_column_width`.
  - **A prescript wraps its base's post-scripts** (issue 824). `${}^{95}\mathrm{MoO}_4$`
    and mhchem's `\ce{^{95}MoO4^{2-}}` read the 4 as a pre-subscript (`_ 4 ^ 95 MoO`,
    `scriptpos="pre2"`): the grammar applied the prescript first and the post-script over
    it. The post-scripts now bind first and the prescript wraps them, `^ 95 (MoO _ 4)`, as
    Perl's `preScripted` (MathGrammar:430-439); a prescripted, post-scripted `\log` or
    `\sin` is applied, `(^ a log _ 2)@(x)` (was `_ 2 ^ a log * x`, or unparsed). The isotope's 6 `\lx@end@inline@math`
    errors the issue reports no longer occur since batch 50's calc `\widthof` fix. Guards
    `perfect_kernel_batch50::calc_widthof_in_math_measures_own_box` and
    `rust_parse_additions_test`; `sampler_test` and `mathtools_test` now match Perl's goldens.
  - **mhchem formulas are chemistry math** (issue 824, stage 1). `\ce{^{95}MoO4^{2-}}` was one
    `<mtext>MoO</mtext>` with text scripts, an en dash for the minus and the isotope in an
    `<mpadded>`. mhchem still parses `\ce`; its formula output hook now makes each element an
    `<mi>` (`Mo`, `O`), the isotope and atomic number prescripts of the first element, the count a
    subscript of the last, and a closing charge (minus U+2212) a superscript of the whole formula:
    `<msup><mrow><mmultiscripts><mi>Mo</mi>…<mn>95</mn></mmultiscripts><msub><mi>O</mi><mn>4</mn></msub></mrow><mrow><mn>2</mn><mo lspace="0em" rspace="0em">−</mo></mrow></msup>`:
    the elements side by side (no invisible times), the charge sign tight as in the PDF. This first
    markup is tentative and awaits Bruce Miller's feedback; stage 2 adds MathML `intent`. DG: "We
    expect further improvements are both possible and needed."
    A text-mode `\ce` keeps its text font (bold in `\textbf`). Arrows, `+`, bonds, states and
    amounts are unchanged. Guard `mhchem_formula::mhchem_formula_units_are_chemistry`
    (OXIDIZED_DESIGN_DIVERGENCES #1040).
  - **A sign TeX sets as an ordinary atom is tight in MathML** (issue 824). TeX makes a binary
    operator an Ord when it starts a list, follows another operator, relation, opening or
    punctuation, or precedes a relation, closing or punctuation, or ends the list (tex.web
    §728-729): `f(0+)`, `a+b+`, `x^{2+}`, a charge `2-`, the `−` of `a+-b` have no space around the
    sign. A browser spaced each as an infix operator (0.222em each side: MathML Core gives a trailing
    `+` or `−` the infix spacing). The MathML space walk now applies TeX's Bin-to-Ord rule to signs and marks such a
    sign `lspace="0em" rspace="0em"`; a sign that stays binary (the `+` of `a+-b`) and a leading one
    (Core's prefix form) are unchanged. Guard
    `perfect_kernel_batchmml2::core_k_operators_have_no_operator_space` (OXIDIZED_DESIGN_DIVERGENCES #1033).
  - **amsmath's arrow fills keep their heads** (issue 824's sibling). `\arrowfill@` and
    `\ext@arrow`'s fill were a plain `\to`, so mhchem's `\ce{A <=> B}` read "→→" and every
    chemarr, extarrows and extpfeil arrow was →. The relation is now the heads amsmath.sty draws
    (`\arrowfill@#1#2#3#4`, :971-976): an end that is just the `\relbar`/`\Relbar` fill is dropped,
    a left/right pair is its composed arrow (↔, ⇔, ⟷, ⟺, ↦), and the named fills
    (`\rightarrowfill@` …) no longer leave their style argument behind. `\ext@arrow` sets the fill in an
    `\hbox`, as amsmath.sty:1014 does, so a package fill that opens its own `$…$` (halloweenmath)
    no longer ends the formula, and an active `|` (ltxguide.cls's `\MakeShortVerb{\|}`) no longer
    errs when amsmath loads. Witness 2405.01836; TL manuals halloweenmath-man, moreenum-doc. Guard
    `arrowfill_heads::arrowfill_keeps_its_heads` (repros `tools/perfect_kernel/repros/arrowfill/`).
  - **`--format=epub` writes an EPUB 3 container** (issue 807). `--format=epub` or an `.epub`
    destination selected the EPUB stylesheet but wrote one bare XHTML file. It now writes the zip
    Perl `latexmlc --dest=x.epub` does: `mimetype` first and stored, `META-INF/container.xml`, and
    under `OPS/` the content (`<name>.xhtml`), its CSS and images, and `content.opf` (manifest,
    spine, `properties="mathml nav"`); a document without a TOC gets Perl's undisplayed
    navigation TOC (`check_TOC`). The conversion log is written beside it, as for a single
    document; without `--dest` the container is `<source>.epub`. With `--split`, every page post
    wrote (subdirectories included) enters the manifest and spine in Split's order, the root
    first and each page before its children, as Perl's `@docs`. Guards
    `whatsinout::format_epub_writes_an_epub3_container`,
    `whatsinout::format_epub_split_pages_follow_splits_order` (in `cluster_cli`) and
    `manifest::epub::tests::url_id_follows_perl`.
  - **`--css`, `--javascript` and the icon link the copy** (issue 807). Each page linked the
    path as given, so an absolute `--css` linked the source file (the EPUB pointed outside
    itself) and a relative one was copied to its basename but linked at its own path; the
    bundled stylesheets linked `<dest dir>/LaTeXML.css` from a split sub-page. Every link is now
    where Perl's `copyResource` puts the file, relative to the page (XSLT.pm:71-125): an
    absolute or escaping path at the top level, a contained relative path at that path, `../`
    from a sub-page; the `@import`ed files still follow. So `--preload=ar5iv.sty
    --css=<path>/ar5iv.css` carries and links ar5iv.css in an EPUB. With `epubcheck` installed
    the EPUB tests validate their output. Guards
    `cli_css_resource_copy::cli_resources_link_their_copies_from_every_page` and
    `whatsinout::format_epub_ar5iv_preload_carries_the_css_param` (in `cluster_cli`).
  - **`revert_as => 'context'` reverts by the dual branch, as Perl does** (#821, thanks
    @nschloe). Both readers now take the branch from the locals stack: a no-argument
    `DefMathI` (Package.pm L1849-1854), whose reversion the port had fixed to the CS at
    definition time, and the `\lx@dual` reversion (Base_XMath.pool.ltxml L243), which read
    a state value nothing set. So `ltx:Math`'s `tex` (presentation branch) got the user's
    CS and `content-tex` was never written: `\lxDefMath{\plus}{+}` gave `tex="a\plus b"`
    where Perl gives `tex="a+b"` + `content-tex="a\plus b"`. Golden `math/declare` moves
    to Perl's attributes, except that `\foo{x}` with an unused `#1` reverts as TeX prints
    it, `\mathbf{foo}`.
  - **nicematrix's `{NiceTabular}` keeps its column and row features** (issue 736). `p[c]{5em}`-style columns
    read their `l`/`c`/`r` and `t`/`m`/`b` keys instead of misreading `[c]` as columns (2607.15452: 12 errors → 0); a
    `\Block{i-j}` cell spans its columns and rows, and a `\\` in its text breaks a line of the block, not the row
    (2206.12224); the body's two-color `\rowcolors` no longer swallows the next cell's `$` (2602.03088: 39 errors →
    0). Guard `perfect_kernel_batchi736::nicematrix_body_features_keep_the_table`.

  - **The log locator names the file with its extension** (#794). A diagnostic's
    location line read `at t; line 10 col 21 - …`, the file stem only; it now reads
    `at t.tex; line 10 col 21 - …`, as Perl's `getShortSource` prints it, so a problem
    matcher written for Perl's logs can find the file (a `.bib`'s own diagnostics name
    `refs.bib` likewise when the reader has its path; an extension-less file reads `README.`,
    as in Perl). The message layout is Perl's
    (the message, then `at …`, then detail lines). Guard
    `locator::tests::display_names_the_file_with_its_extension`; eleven guards that
    pinned the stem now pin the file name.

  - **A failing stylesheet is a Fatal, exits 1 and writes no output** (#782). A bad
    XPath, an `xsl:message terminate="yes"` or a missing `xsl:import` logged
    `Error:post:convert`, exited 0 and wrote the untransformed XML under the `.html`
    name. A MathML or XSLT step that fails now stops post-processing with
    `Fatal:post:convert` and nothing is written, as a die in Perl's ProcessChain does:
    with `--split`, the pages this run rendered before the failing one are removed, serially
    and under `LATEXML_RENDER_JOBS` (a page an earlier run left in the destination stays);
    `api::convert_to_html` returns `Err` with the log. A core Fatal that keeps its partial
    document is still post-processed and returned, as in Perl. A stylesheet that cannot be
    found stays an Error, as in Perl. Guards `xslt_failure_is_fatal` (in
    `cluster_xslt_split`), `api::tests::a_post_fatal_is_an_err` and
    `api::tests::a_core_fatal_keeps_its_rescued_document`.

  - **Split pages keep the document's processing instructions** (#795). With `--split`,
    only the main page carried the `<?latexml …?>` PIs; the other pages lost them, so a
    custom stylesheet reading them saw nothing there. Each page now gets a copy before its
    root, as Perl's `newDocument` does (the streaming split already did). Guard
    `document::tests::split_pages_carry_the_document_pis`.

  - **An option given twice keeps its last value** (#530). `--urlstyle=file
    --urlstyle=server` (and any other single-valued option or flag repeated) stopped
    with "cannot be used multiple times"; the last one now wins, as with Perl's
    GetOpt::Long, so a base option list can be overridden by appending to it. Repeatable
    options (`--css`, `--path`, `--preload`, ...) still accumulate. Guard
    `boolean_flag_last_wins_tests::repeated_options_are_last_wins`.

  - **`--xsltparameter` takes Perl's `name:value`, and a malformed one is an error** (#803).
    `--xsltparameter=FOO:bar` was silently ignored; it now sets `$FOO`, as in Perl's
    `latexmlc`/`latexmlpost` (a `\w+` name, then `:`). `name=value` keeps working (it also
    carries `{uri}name` names). A value with neither form stops the run with a usage error
    (exit 2), as `latexmlpost` does (Perl's `latexmlc` only warns, LaTeXML.pm L574-578);
    the library entry point logs an Error. Guard
    `xsltparameter_tests` (in the binary's unit tests).

  - **A `--stylesheet` on disk is used, even when it is named like a built-in one** (#806).
    `--stylesheet=./LaTeXML-html5.xsl` loaded the embedded copy instead of the local file,
    because the embedded stylesheets were matched by file name alone. A path that names an
    existing file now loads that file, and so does an engine-named stylesheet it imports from
    beside it; `urn:x-LaTeXML:XSLT:` imports and the default stylesheet always come from the
    binary, whatever `resources/XSLT/` or `LaTeXML-*.xsl` files sit in the working or source
    directory. A `%` in a plain path is a character (`/w/100%/LaTeXML-html5.xsl`); only a
    `file:` URL is percent-decoded. Guards `xslt::local_stylesheet_tests`,
    `xslt_custom_stylesheet::stray_engine_sheets_do_not_replace_the_default`.
  - **The format builds error-free on the LaTeX kernel of November 2026.** That
    kernel reads `glyphtounicode.tex` with expl3's `\file_get:nnN` while the format
    is made (latex.ltx:22952-22957). An `\input` met while definitions are read ran
    the file in its own loop, so the reader that set `\everyeof` never got the
    file's tokens: it scanned on through latex.ltx with expl3 syntax off, and
    `--init=latex.ltx` raised 137 errors. Such an `\input` now joins the reader's
    input stream with the `\everyeof` tokens at its end, as TeX's does, and in a
    document no binding answers an `\input` under `\everyeof` either:
    `\file_get:nnN{ltluatex}` and `{pstricks}` read the files (7 and 16 Errors
    before). Guard `perfect_kernel_batch51::file_get_reads_the_file_whole`.
  - **`tools/make_formats.ps1` gates the dumps as `make_formats.sh` does.** The
    Windows mirror had drifted: it now fails on any `Error:`/`Fatal:` of an
    `--init` run (`LATEXML_INIT_DEBUG=1`), pins `TEXMFROOT` to the detected TeX Live
    (the dual-TL guard), and honours `CARGO_TARGET_DIR`. It used to keep a dump
    built with errors without a word.
  - **Windows uses the linked libkpathsea it was built with.** The check that the
    linked library reads the same TeX Live as the `kpsewhich` on PATH compared
    paths as strings: kpsewhich names the root `C:/texlive/2026`, the library its
    files `c:/texlive/2026/…`, so every conversion fell back to the subprocess
    backend (about 270 ms a file lookup). Paths now compare as Windows compares
    them. Guard `path_is_under_compares_as_the_file_system_does`.
  - **Windows diagnostics name their source file.** A location's source was taken
    for a URL whenever it held a `:`, which every Windows path does, so messages
    read `at C:\…\t.tex.index.tex; line 3` where Linux says `at t.tex; line 3`. Perl's
    rule is restored (`Locator.pm:55`: a protocol is more than one character).
    Guard `a_windows_path_is_no_url`.
  - **Windows keeps the input's directory in its plain form.** The input was
    canonicalized to Windows' verbatim form (`\\?\C:\…`), which then reached
    `\graphicspath` lookups (no file found), EPS candidates (listed twice) and
    progress lines. New `pathname::canonicalize` drops the prefix where the plain
    path names the same file.
  - **`\directlua` output on Windows.** texlua writes its stdout in text mode
    there, so each `\n` arrived as `\r\n`: the frames' byte counts no longer
    matched, a `\r` ended every `tex.print` line, and the next frame could
    desync. The bridge now counts the bytes that reach the pipe and drops the
    added `\r`.
  - **The no-dump expl3 reload no longer mutes its diagnostics silently** (#825). When
    `expl3.sty` re-reads `expl3-code.tex` raw (no kernel dump), the records the scope
    drops are now tallied (`IgnoreDiagnosticsScope::dropped`) and reported: a Warning
    naming the error and fatal records dropped, or an Error when the reload stopped early,
    which leaves the expl3 functions it had yet to define missing (mhchem's `\ce` on
    TL 2026). On TL 2025 a `LATEXML_NODUMP=1` run now warns of the 5015 records
    the reload mutes. Guards `common::error::tests::an_ignoring_scope_tallies_what_it_drops`,
    `expl3_sty::tests::a_muted_expl3_reload_is_counted`,
    `dump_gate_init::nodump_latex_branch_converts_healthily`.
  - **Builds and converts on a machine without TeX Live.** The `kpathsea` crate's build
    stopped when it found neither `libkpathsea` nor `kpsewhich`; `.cargo/config.toml` now sets
    its opt-out (`KPATHSEA_SKIP_TOOLCHAIN_CHECK`), as the release workflows' TeX-less legs did.
    Such a binary has no format dump (making one needs TeX Live), and the no-dump branch broke
    on commands our own code calls where Perl's does not: a small article stopped at
    `Fatal:TooManyErrors` (101 Errors), and `LATEXML_NODUMP=1` broke the same way where TeX
    Live is. Now the base carries `\@stpelt`, `\if@ignore`, `\@ensuredmath` and
    `\IfDocumentMetadataTF` (the text after a list, `\stepcounter` and `\ensuremath` were
    lost); the kernel autoload, with no dump to ask, loads the format on the first undefined
    command of a LaTeX document (latexml.sty's `\AddToHook`, an `\IfFileExists` choosing the
    class); our expl3 extensions, hyperref's document-properties store, mathtools' expl3 block
    and `\DocumentMetadata`'s tagging packages wait for an expl3 to extend, while `\ExplSyntaxOn`
    without one is expl3's catcode switch (`\usepackage{array}` raised 2 Errors); a missing
    `expl3.sty` is reported; and, as in Perl, no
    Warning is raised for the missing resolver itself (the log's `Info:kpathsea:backend` line
    names it; it was one Warning on every conversion there). The article now
    converts with no Error, and the no-dump body is byte-identical to the dump branch's. A
    new CI job builds and converts it on a runner without TeX Live (`tools/notex_smoke.sh`).
    Guards `dump_gate_init::nodump_latex_branch_converts_healthily`,
    `preclass_kernel_autoload::latex_route_pre_documentclass_kernel_cs_autoloads`;
    OXIDIZED_DESIGN_DIVERGENCES #1095.

  - **`page=` on an SVG, PNG or JPEG no longer re-encodes it** (#791). A single-page
    format has no page to select (pdfTeX ignores `page=` on a PNG), so
    `\includegraphics[page=1]{fig.svg}` now copies `fig.svg` as it is, with its
    `width`/`height`, exactly as without `page=`. Before, it went through ImageMagick
    into a rewritten `x1.svg` without its size attributes. Guard
    `graphics::tests::a_single_page_source_ignores_page`.

  - **A Rhai `Fatal()` is logged once** (#792). The script's caller now returns the
    Fatal the binding raised instead of wrapping it in a second `Fatal:Document:Generic`
    line; this holds at a binding's top level, in a binding another binding loads, and
    for `Error()` past MAX_ERRORS. A `Fatal()` inside a body (macro, constructor, hook)
    now ends the conversion, as Perl's `Fatal` dies, instead of being contained as an
    `Error:script:` line. A Fatal the script catches never stands in for a later error,
    which stays its own `Error:script:` line. A Fatal ending digestion prints its one
    `Fatal:` line, no longer repeated by a `Warning:recovery:digest_internal` (a
    timeout or memory Fatal keeps its "not recovering" warning).
    Guard `30_script_bindings::a_rhai_fatal_is_logged_once`.

  - **Rhai `Let(a, b, scope)`** (#796). A binding can now write a global `\let`
    (`Let("\\alias", "\\target", "global")`), as Perl's `Let($t1, $t2, $scope)`;
    the 2-arg form stays local. Each side is a string, taken as a control-sequence
    name, or any `Token`, as in Perl: `Let(T_ACTIVE("~"), "\\nobreakspace")`,
    `Let(T_MATH(), …)`, `Let("\\x", T_OTHER("a"))`. New token constructors, as Perl's
    Token.pm exports them: `T_LETTER`, `T_OTHER`, `T_ACTIVE`, `T_COMMENT`, `T_MARKER`,
    `T_ARG`, `Token(string, catcode)`, the constants `T_BEGIN()` … `T_CR()` and
    `CC_ESCAPE()` … `CC_ARG()`, and `Tokens(…)` (up to four arguments, or an array). `T_CS` now returns a single `Token`, not a one-token `Tokens`;
    `Digest`, `Expand`, `ExpandPartially`, `UnTeX`, `AtBeginDocument`, `AtEndDocument`
    and `absorb` take either. `XEquals`, `IsDefined`, `LookupMeaning`,
    `LookupDefinition` and `AssignMeaning` take the same token arguments as `Let`.
    `Let` now goes through `state::let_definition`, as the `Let!` macro does, so a
    provide-only binding load declines it. Guards `script_bindings::tests::let_takes_a_scope`,
    `script_bindings::tests::let_takes_any_token`.

  - **The Rhai binding API is Perl's binding API.** A runtime binding (`.rhai`, no Rust
    toolchain, no recompile) writes Perl LaTeXML's Package.pm calls under Perl's names, with Perl's
    arguments and meanings: typed values (`Number`, `Dimension`, `Glue`, `Token`, `Tokens`, boxes,
    arrays and maps) both ways, Perl's hook and body arguments, Perl's option tables and
    diagnostics. The calls whose names or results changed are marked **BREAKING** below with
    their migration; where an entry names a deprecated form, that form keeps working in 0.8.0
    with a `Warning:deprecated` once per binding load and is removed in the release after 0.8.0;
    the other breaking changes take effect in 0.8.0 (#829, #849, #860, #870).
  - **Rhai digestion hooks take Perl's arguments.** A `beforeDigest`/`beforeDigestEnd` hook receives the stomach, a
    constructor's, environment's or `DefMath`'s `afterDigest`/`afterDigestBegin`/`afterDigestBody` hook
    `|stomach, whatsit|` and a `properties` closure `|stomach, x, …|`, each argument as `whatsit.getArg` gives it
    (Constructor.pm:99-117). A primitive's or parameter type's `afterDigest` now receives the stomach (Perl's argument);
    a body that called whatsit methods on the dummy it used to get now raises `Error:script`. The old forms
    (`afterDigest: |whatsit|`, `properties: |x|`) keep working through 0.8.0 with a `Warning:deprecated` once per
    binding load and are removed in the release after 0.8.0. A failing `afterDigestBegin`/`afterDigestBody`/`beforeDigestEnd` hook names itself in its `Error:script`
    (was "afterDigest"/"beforeDigest"). `RefStepCounter`'s documented result is Perl's `tags` and `id`. Guards
    `perfect_kernel_batchrhaihook::*`; repro `rhai-bindings/afterdigest_stomach_first` turns green.
  - **Rhai values print as their TeX** (#849). **BREAKING:** `LookupMeaning` returns the meaning, as Perl's
    and `LookupMeaning!` — a `Definition` handle (which `AssignMeaning` installs) or a Token, `()` when undefined —
    where it returned the printed string `Stored[??]`; a string comparison keeps working in 0.8.0 with a
    deprecation Warning (removed in the release after 0.8.0). `ToString` of a `Definition` is Perl's (the CS and its parameters, was `DefinitionProxy`),
    `ToString` of an array, and an array in a string, print each Token, Tokens, box and value handle as its TeX (were
    Rust type paths, with math Errors), and `LookupString` (the core `lookup_string` behind `LookupString!`) of a
    stored Token or Tokens is its text (was the Rust Debug form). Guard
    `perfect_kernel_batchw3rg::rhai_values_print_as_perl`.

  - **The Perl binding names Rhai lacked** (#849). `DefMacroI`, `DefPrimitiveI` and
    `DefConstructorI` take Perl's I-form arguments (a `Token` or a name read as `coerceCS`, a parameter spec or `()`,
    each body kind, the option map) and lower to the core calls their prototype forms make; `IfCondition`, `Box`,
    `List`, `tokens.unlist`, `token.getCatcode`/`getString`/`getCSName` and `box.getFont`/`whatsit.getFont` are
    registered. `IfCondition` of a non-conditional reads its Error as Perl's (`got '\relax'`).
    Guard `perfect_kernel_batchw3rg::rhai_perl_i_forms_and_accessors`. `IfCondition(cs, [args…])` tests a
    conditional on the given arguments, as Perl's `IfCondition($if, @args)`, and never reads the input (the core
    `if_condition` read an argument-taking conditional's arguments from the document); an argument not given is
    Perl's undef. The I-forms' `coerceCS` takes Tokens as their text, and `\csname X\endcsname` needs Perl's space
    after `\csname`. `SetCondition` of a control sequence that is no conditional errs as Perl's (it `\let` a
    parameterless primitive such as `\relax` to `\iftrue`). `LookupMeaning`'s value keeps, in 0.8.0, the
    `.is_empty()`/`.len()` of the string it was, Warned (removed in the release after 0.8.0). Guard `perfect_kernel_batchw3rg::rhai_review_rows_round_h`.

  - **A Rhai name that is a Perl API name behaves as Perl's** (#829). A `DefPrimitive`
    body's returned box, or array of boxes, is the primitive's output, as a Perl primitive's
    (`|| Digest(T_LETTER("A"))` printed nothing before); another returned value is dropped with a
    deprecation Warning (Perl errs on it; end such a body with `return;`). `MergeFont` and every
    `font:` option take all of Font.pm's merge keys — `color`/`background` by name (as
    `LookupColor`), `opacity`, a size name (`large`), `scale`, `forcebold`/`forcefamily`/
    `forceseries`/`forceshape`, `emph` — where all but family/series/shape/size were dropped
    silently; an unknown key Warns. A base colour name (`red`) needs no colour package
    (dvipsnam.def is read only once one defines `\DefineNamedColor`). Perl's `Warn`/`Error`/`Info`/`Fatal(cat, obj, where, msg,
    details…)` form is read when `where` is `()` or a handle. `SetCounter`/`AddToCounter` take a
    Number, `StepCounter` its `noreset`, `PushValue`/`UnshiftValue` up to four values, `Revert` a
    string, `LookupDimension` a Token, Tokens, box or Dimension; `ptValue`/`pxValue` clamp the
    precision to 0..5. `Number(string)` and the core `Number::from(String)` are one reader,
    Perl's `int()`: `"1e3"` is 1000, and a string not wholly a number (`"12abc"`) reads its numeric
    prefix with Perl's `perl:warn` Warning (the core gave 0 silently). `ExplodeText` (and `roman`, the kernel's `ExplodeText!`) makes only ASCII
    letters LETTER, as Perl Token.pm and latex.ltx's `\char@if@alph` (`é` is OTHER).
    **BREAKING:** `CounterValue` returns a Number (its `+`/`-`, comparisons and equality with an
    int or float, in either order, and `roman`/`Roman` of it keep working in 0.8.0 with a
    deprecation Warning, removed in the release after 0.8.0; use `.valueOf()`); `node.getAttribute`
    of an absent attribute is `()` (`defined(…)` tests it; `== ""` still holds, Warned);
    `document.setAttribute(key, value)` is deprecated for Perl's `(node, key, value)`. Each of
    these deprecated forms works in 0.8.0 with its Warning and is removed in the release after 0.8.0.
    Guards `cluster_package_guards::perfect_kernel_batchw3rg`,
    `perfect_kernel_batchw3rg::rhai_perl_names_review_rows`.

  - **BREAKING: Rhai Def* option bags follow Perl's option tables** (#829). Each of
    `DefMacro`, `DefPrimitive`, `DefConstructor`, `DefEnvironment`, `DefMath` and
    `DefConditional` checks its bag against Perl's key table (Package.pm `CheckOptions`);
    a key outside it is one `Error:misdefined` ("… does not accept options:bogusopt"),
    where it was ignored. `DefPrimitive`'s `nargs`, `DefMath`'s `mode`, `bounded`,
    `requireMath`, `forbidMath`, `isPrefix`, `DefMacro`'s `alias` and the
    environment-only hooks on `DefConstructor` and the Rust-only `texGroup` are now such
    keys, as in Perl. Newly honoured: `locked`, `scope` and `nargs` on
    `DefConstructor`/`DefEnvironment`, `robust` on `DefConstructor` only,
    `font` and `alias` on `DefPrimitive`, `font`, `reversion`, `sizer`, `decl_id`,
    `operator_scriptpos` and `scope` on `DefMath`, `nopackParameters` on `DefMacro`.
    A Perl key the core cannot carry yet (`outer`/`long` on `DefPrimitive`/`DefConstructor`,
    `attributeForm`, a closure `skipper`, a string `sizer`) is a `Warning:unsupported`.
    `DefConditional` takes an option bag and has its test-less `\newif` form
    (`DefConditional("\\iffoo")`, with `SetCondition`); its test receives Tokens.
    Guard `cluster_package_guards::perfect_kernel_batchw3ra`.

  - **BREAKING: Rhai values keep their type both ways** (#829). `LookupValue`
    and `LookupMapping` return the value as it was stored (a bool, an int, a float,
    `Token`, `Tokens`, a digested handle; a list as an array), where they returned its
    string form; a missing key is `()`, where `LookupMapping` returned `""`. `AssignValue`
    and `AssignMapping` take any of these values, where they took only a string
    (`AssignValue(k, Tokenize(…))` was "Function not found"); `()` stores undef, and
    removes a mapping key. A whatsit property or counter property read from a script
    keeps its type too. Guard `cluster_package_guards::perfect_kernel_batchw3rb`.
  - **BREAKING: the Rhai gullet reads return Tokens.** `ReadArg`, `ReadUntil` and
    `ReadOptional` return `Tokens`, keeping the catcodes the source gave, where they
    returned an untokenized string; `ReadOptional` without a `[…]` is `()`, where it was
    `""`. A `Token` or `Tokens` in a string context (`"(" + ReadArg() + ")"`) is its TeX
    source. New `ReadToken`, and a `Gullet` handle with Perl's methods (`readArg`,
    `readUntil`, `readOptional`, `readToken`, `skipSpaces`).
  - **A Rhai macro or reversion closure may return a `Token` or `Tokens`**, as a Perl macro
    sub returns tokens; it expanded to the type's name (`Token`) before.
  - **Rhai argument shapes as Perl's:** `Digest` also takes a string (TeX source), `DigestText`
    also a `Token` or `Tokens`, and `AssignCatcode` also takes a scope; the 0.7.6 forms are
    unchanged. **BREAKING:** `LookupCatcode` of a character the table does not hold is `()`
    (Perl undef), where it was 12.
  - **New Rhai calls:** `LookupMathcode`/`AssignMathcode`, `LookupSFcode`/`AssignSFcode`,
    `LookupLCcode`/`AssignLCcode`, `LookupUCcode`/`AssignUCcode`,
    `LookupDelcode`/`AssignDelcode` (each Assign* with an optional scope), and
    `LookupMappingKeys` (sorted, as Perl's). A code the table cannot hold (a catcode
    above 15, a code outside 0..65535) is an Error naming the call, not dropped.
  - **Rhai diagnostics as Perl's:** `Fatal(cat, obj, msg)` logs `Fatal:<cat>:<obj>`, where it
    logged `Fatal:Internal:Unexpected` (#792); `Warn`/`Error`/`Info`/`Fatal` take Perl's
    trailing details (a string or an array of strings, one line each).
  - **Rhai options Perl accepts, honoured** (#829): `Tag`'s `afterOpen`/`afterClose`
    hooks (and `:early`/`:late`; another key is an Error); `NewCounter`'s `idprefix`/`idwithin`/
    `nested`; `RequirePackage`/`LoadClass` option maps (`options`, `type`, `notex`, `after`,
    `withoptions`, …; `LoadClass`'s `notex`/`searchpaths_only` reach no class-load option yet
    and only Warn; `RequirePackage`'s obsolete `raw` Warns as Perl's); `ProcessOptions(#{inorder,
    keysets})`; `AtBeginDocument`/`AtEndDocument` of a closure; an array of hook closures
    (`afterDigest: [f, g]`), each run; `DefLigature`'s `fontTest` closure, given the text's
    `Font` (`getFamily`, `getSeries`, `getShape`, `getEncoding`, `getLanguage`, `getSize`).
  - **Rhai calls lower to their macro's core call:** `RefStepCounter` steps through
    `\refstepcounter`'s current meaning, as `RefStepCounter!` and TeX (`RefStepCounter(c, true)`
    steps directly; Perl 0.8.8 steps the kernel counter, KNOWN_PERL_ERRORS #412); `AssignMeaning` to an undefined token undefines; a scope string other than
    local/global is a named scope, as the macros read it, where it was dropped to the default.
  - **A Rhai closure takes any number of arguments, as a Perl sub:** missing ones are `()`,
    extra ones dropped (a `DefConditional` test, a `Tag` hook), where the count had to match.
  - **Rhai arrays and maps are stored as lists and hashes** (`AssignValue`, `AssignMapping`,
    `PushValue`): `let o = LookupValue(k); o.push(x); AssignValue(k, o)` round-trips, where
    the array was stored as its printed text. A `Token`/`Tokens` given where a string is
    read (a `properties` value, an option) is its ToString, as Perl's (`\foo bar` reads
    `\foobar`), where it was the Rust type name.
  - **New Rhai calls, thin wrappers over the macros' core calls** (#829):
    `Invocation`, `AddToMacro`, `DeclareFontMap`/`DeclareFontMapMultichar`, `LoadPool`,
    `IsEmpty`, `DigestIf`, `ProcessKeyOptions`, `LookupRegisterOrDefault`, `FindFile`, `Input`,
    `CleanLabel`/`CleanID`, `SetCounter`, `roman`/`Roman`, `UTF`, `Explode`/`ExplodeText`,
    `BeginItemize`, `Equals`, with Perl's argument forms.
  - **BREAKING: Rhai hooks take Perl's arguments, and the #716 renames** (#829). A hook
    receives the whatsit as Perl passes it: `afterDigest: |whatsit|`, `beforeConstruct`/`afterConstruct:
    |document, whatsit|`, a sizer `|whatsit|`, and a reversion closure `|whatsit, arg1, …|` — **a reversion
    written `|arg1, …|` must add the leading `whatsit`**. The whatsit has Whatsit.pm's `getArg(n)` (the
    argument as a constructor body receives it), `getProperty(key)`, `getProperties()` and `setProperty(key,
    any value)`. A constructor body receives Perl's `%props` as a trailing map (`|document, x, props|`).
    `openElement` takes an attribute map and returns the new node; `closeElement` and `maybeCloseElement`
    return the node closed, or `()`. `DefAccent` takes Perl's options map (`#{below: true}`; it checks no key,
    as Perl's). `AssignValue(k, 7)` reads back through `LookupNumber`. These old names stay in
    0.8.0 as aliases, each logging `Warning:deprecated:<name>` once per binding load, and are
    removed in the release after 0.8.0:

    | old | use |
    |---|---|
    | `whatsit()` | the hook's `whatsit` argument |
    | `whatsit.argString(n)` | `whatsit.getArg(n)` |
    | `whatsit.propertyString(key)` | `whatsit.getProperty(key)` |
    | `document.getProperty(key)`, `document.getProperties()` | the constructor body's trailing properties map |
    | `assign_value(k, v)`, `assign_global(k, v)` | `AssignValue(k, v, "local")`, `AssignValue(k, v, "global")` |
    | `lookup_value(k)` | `LookupValue(k)` |
    | `AssignString`, `AssignNumber`, `AssignBool` | `AssignValue(k, v[, scope])` |
    | `AssignMeaning(cs, "\\other")` (a string meaning) | `Let(cs, other[, scope])`, or `AssignMeaning(T_CS(cs), LookupDefinition(other))` |
    | `DefAccent(a, c, s, below)` | `DefAccent(a, c, s, #{below: true})` |

    Guards `cluster_package_guards::perfect_kernel_batchw3rd::{rhai_perl_forms_716,
    rhai_deprecated_aliases_warn_once}`.
  - **BREAKING: Rhai bodies receive Perl's arguments; `GetKeyVal` returns the value** (#829). A
    `DefMacro` (and `DefColumnType`, `DefConditional` test) closure is Perl's `|gullet, args|`, a `DefPrimitive`
    closure `|stomach, args|` (`stomach.getGullet()`), each argument `Tokens`, a `Token`, a `KeyVals` or `()` for an
    omitted optional. A closure declaring no parameter is Perl's plain `sub {…}`. One declaring no leading handle —
    1 to as many parameters as the definition's arguments — keeps receiving TeX strings in 0.8.0 (removed in the release after 0.8.0), with
    `Warning:deprecated:DefMacro |args|` (`DefPrimitive |args|`, `DefColumnType |args|`, `reversion |args|`) once per
    binding load; the two forms cannot be told apart by arity, so a Perl-form closure declares its handle and every
    argument, `|gullet, x|` (`|gullet, _x|` where Perl writes `my ($gullet) = @_`); fewer parameters is the old form,
    with a Warning naming the fix. A body that only concatenates its arguments
    (`"\\x{" + a + "}"`) migrates by adding the handle, one that calls string methods (`a.len()`) by `UnTeX(a)`. A
    whatsit in a string is its reversion, as Perl's `ToString($whatsit)`. `GetKeyVal(kv, key)` returns the value as
    Perl's (KeyVals.pm:31) — Tokens as read, a box once digested, `()` when missing — where it returned a string
    (`""` when missing): test it as Perl's `if ($v)` with `!!v` (`!v` for `unless`; Perl's truthiness: `()`, `""`,
    `"0"`, `0` false, every Token, Tokens, box, array and map true) or `defined(v)` (Perl's `defined`), and wrap a
    string use in `ToString(…)`; an old test against a string (`v != ""`, `"" != v`) compares the string forms, `()`
    as `""` on either side as Perl's `undef eq ''`, with `Warning:deprecated:value == string` / `() == string` once
    per load (removed in the release after 0.8.0) — so `v == ()` is no longer the defined test for a string (`"" == ()` is true). `GetKeyVals` gives those values (a repeated
    key's as an array).
    `ToString` takes any value; `document.absorbString` takes any value (its string; `()` adds nothing). New
    `DefParameterType(name, |gullet, extra…| …, #{…})` with Perl's `$parameter_options` (`nargs` accepted unread).
    `FindFile`'s `raw` Warns deprecated and is dropped (Package.pm:2079); `AddToMacro` keeps the macro's lock (Package.pm:2547);
    `DeclareFontMap`'s `*_mathstyle` is a font map, stored as the `Font` (another value Warns unsupported). Guards
    `cluster_package_guards::perfect_kernel_batchw3rd::{rhai_closure_args_and_parameter_types,
    rhai_deprecated_aliases_warn_once}`.
  - **BREAKING: Rhai registers and values are typed: `Number`, `Dimension`, `Glue`, `MuGlue`** (#829).
    Perl's constructors (`Dimension("3pt")`, `Dimension(65536)` in sp, `Glue("1pt plus 2fil minus 1pt")`,
    `MuGlue("3mu plus 1mu")`, `Number(5)`) and methods (`valueOf`, `ptValue`, `toString`, `add`, `subtract`,
    `multiply`, `divide`, `negate`, `smaller`, `larger`, Glue's `getStretch`…), Rhai's `+ - * /` and comparisons
    over the same arithmetic (Glue sums as TeX's, the higher fill order winning); a handle is true (`!!v`). A spec
    the constructor cannot read (`Dimension("3 furlongs")`) is an Error and 0, where Perl reads it silently.
    `DefRegister(cs, value[, #{readonly, getter, setter, address, allocate}])` takes any of them (or `Tokens`), with
    Perl's option table (another key is `Error:misdefined`); a string value is read for the type `cs` already has,
    else by its form (`"1pt plus 2pt"` is now a Glue register, where it printed `0.0pt`). New `LookupRegister(cs[,
    params])`, `AssignRegister(cs, value[, scope[, params]])`, `LookupDimension(x[, noerror])`. Return types that
    change: `LookupRegisterOrDefault` gives the typed value (was its TeX string), `LookupValue`/`GetKeyVal` of a
    stored Number, Dimension or Glue the handle (a Number was an int), and a closure's `{Number}`/`{Dimension}`/
    `{Glue}` argument the handle (was its TeX string). Migration: a string use works unchanged (`"" + v`, `${v}`,
    `ToString(v)`); a test against a string Warns `deprecated` once per load (use `ToString(v) == "3.0pt"`; the
    string test is removed in the release after 0.8.0); an
    int use takes `v.valueOf()`. `MuGlue("3mu plus 1mu")` no longer reconverts its stretch as points. Guards
    `cluster_package_guards::perfect_kernel_batchw3rc::{rhai_typed_registers, rhai_typed_registers_error_side}`.
  - **Rhai registers as Perl's** (#829). `LookupRegister` of an undefined control
    sequence Warns `expected:register` as of a non-register (Package.pm:1365-1367; the `LookupRegister!` macro's
    lookup); a fresh register's unitless string (`DefRegister("\\c", "5")`) is an Error and 0, where it was silently
    5sp. Glue and MuGlue `+`/`-` are TeX's `\advance` (`Glue - Dimension` keeps the stretch, `MuGlue + MuGlue` keeps
    the higher fill order), where Perl's Glue.pm/MuGlue.pm differ (OXIDIZED_DESIGN_DIVERGENCES #1030).
  - **The Rhai dialect is documented for newcomers** (#829): a guide,
    `docs/parity/RHAI_BINDINGS_GUIDE.md` (loading, values, the Perl closure form, truthiness and `defined`,
    diagnostics, the deprecated-name migration); a three-column cheat sheet, `CHEATSHEET.md` beside `API.md`
    (Perl | Rust macro | Rhai, generated from `api_doc.rs` and checked with it, its empty cells listed as the
    dialect's gaps); worked examples under `docs/examples/` that CI converts with no Error or Warning
    (`30_script_bindings::docs_examples_convert_clean`). `script_bindings_plan.md` keeps the design notes.
  - **Rhai `AssignMeaning` is Perl's `$STATE->assignMeaning`** (#829): `AssignMeaning(token,
    meaning[, scope])` stores a meaning, a `LookupDefinition` handle, a `Token` (`T_LETTER("a")`, as `\let\x=a`) or
    `()` (undefined), as the `AssignMeaning!` macro does, where it was a deprecated `Let`. A `LookupDefinition`
    handle keeps the definition it found, so `AssignMeaning` after a redefinition installs the old one, as Perl's.
    A string meaning, the old `AssignMeaning("\\a", "\\b")`, still lets in 0.8.0 and Warns `deprecated` once per
    load, naming `Let` and the handle form (removed in the release after 0.8.0). Guard `cluster_package_guards::perfect_kernel_batchw3rd::rhai_perl_forms_716` (p5).

  - **etoc binding (#760).** `\localtableofcontents` now lists the enclosing
    chapter's or section's units, to the depth `\etocsetnexttocdepth` sets, and
    `\tableofcontents` stays the document's contents; `\invisibletableofcontents`
    prints nothing, as in etoc. A `<ltx:TOC>` whose `scope` is a unit's `xml:id`
    lists that unit's children (the schema's documented meaning; Perl reads only
    `global`).
  - **`{verbatim}` honours upquote (#801).** With `\usepackage{upquote}`, the
    `verbatim` and `verbatim*` environments now print `'` and `` ` `` upright, as
    fancyvrb's `Verbatim` and `\verb` already did and as pdflatex prints all of
    them. Without upquote they keep the typewriter font's curly quotes.
  - **The standard classes' size options set `\textwidth` (#779).** In one column,
    `article`, `report` and `book` now set `\textwidth` as their size file does
    (`size1x.clo`/`bk1x.clo`): the paper's width less 2in, at most 345pt (10pt),
    360pt (11pt) or 390pt (12pt). Before, every size got 10pt's 345pt, so a
    `\begin{minipage}{\textwidth}` in a 12pt article was too narrow. Under
    `\documentstyle`, book's width is `bk1x.clo`'s 4.5in (10pt) or 5in. `\@ptsize`
    now follows the size option too. Both read the options the class was given,
    so a class passing its options to `article` (elsarticle, llncs) reaches them;
    llncs keeps its own 12.2cm.
  - **`\import@path` is defined (#772).** import.sty's own macro now holds the
    current import directory, as the package sets and restores it, so a document
    that reads it (`\href{\import@path index.tex}{…}`) no longer gets an
    undefined-macro error; a nested `\import` empties it, as import.sty does. Also
    drops a stray space the import commands typeset before the imported file.
  - **caption's `justification=` and `skip=` reach the caption (#787).** A
    `\captionsetup{justification=raggedright}` (or `centering`, `raggedleft`,
    `justified`, and ragged2e's `RaggedRight` &c.) is the `<caption>`'s `text-align`
    (`cssstyle`), in place of the class a `\centering` in its float gives it, and
    `skip=`, `aboveskip=`, `belowskip=` its margins on the sides caption puts them,
    `position=auto` read as caption reads it; `\captionsetup[longtable]{…}`
    applies to longtable captions. Perl ignores both.
  - **colortbl's `\arrayrulecolor` colours the table rules** (#773). Each rule
    takes the colour colortbl draws it in (a `|` the one set as its table or
    `\multicolumn` starts, else the one current when it is drawn): `\hline`,
    `\cline`, the `|` rules, `\multicolumn`'s, `\hhline`'s and booktabs' rules.
    A cell carries it per side as `cssstyle="border-top-color:#BFBFBF"`, and a
    colour set inside a tabular or array ends with it. Black, and a table without
    `\arrayrulecolor`, add nothing. (Perl ignores it.)
  - **BREAKING (library): `post::PostOptions::nodefaultresources` is now `default_resources`,
    with the meaning inverted.** A caller that sets the old field no longer compiles; set
    `default_resources: !old_value`. The double negative (`nodefaultresources: false`) is gone.
    The positive name is Perl's option name (`Config.pm:120` `defaultresources!`),
    and the XSLT step still receives its negation as `noresources`, as
    `LaTeXML.pm:584` passes it. The CLI flags `--defaultresources` and
    `--nodefaultresources` are unchanged.
  - **New `api::convert_to_html_with(tex, &HtmlOptions)`.** `HtmlOptions` holds the
    settings of the library's HTML entry point, and its `Default` is what
    `api::convert_to_html(tex)` does, so that function is unchanged. The first
    setting is `default_resources`; set it to `false` (the `--nodefaultresources`
    equivalent) to get the page without links to the bundled CSS/JS and with
    nothing written to the working directory. Construct `HtmlOptions` with
    `..Default::default()`, so a setting added later leaves callers compiling. The
    tests use it through `util::test::convert_to_html_page`, so a test run no
    longer leaves `LaTeXML.css`/`ltx-*.css` in `latexml_oxide/`.
  - **A symbolic link among the cached format dumps is never trusted on macOS.**
    `private_to_user` rejected a link by its mode bits, which are `0o777` on
    Linux but follow the umask on macOS, so a link to another file passed there.
    It now rejects links by file type. Guard
    `only_this_users_files_no_one_else_can_write_are_trusted`, red on macOS
    before.
  - **`--init` no longer segfaults intermittently on Linux.** `LATEXML_INI_MODE`
    was set after the kpathsea prewarm thread started, so glibc's `setenv`
    reallocated `environ` under that thread's `getenv` (7 of 200
    `--init=latex.ltx` runs on Linux aarch64), failing `tools/make_formats.sh`.
    It is now set before the thread exists (STABILITY_WITNESSES Cluster J).
  - **TikZ matrix and tikz-cd cells sit on their last row's baseline (#785).**
    `openTikzAlignment` placed a matrix at `y = h + d`; Perl
    (`pgfsys-latexml.def.ltxml` L952-975) subtracts the last row's depth, read
    from the row sizes `Alignment.pm` passes to `openContainer` (L321-323). Rust
    passed none, so every cell of a tikz-cd diagram or `\matrix` sat one
    last-row depth above its arrows. The alignment now passes
    `rowheights`/`rowdepths`/`columnwidths` to the container callback as Perl
    does. Guard `picture_sizing::tikz_matrix_places_last_row_baseline_at_origin`.

  - **The XSLT recursion cap now applies in the statically linked release
    binaries.** `xsltMaxDepth = 1000` (Perl `XML::LibXSLT->max_depth(1000)`) was
    written through `dlsym`, which cannot see a global of a statically linked
    libxslt, so those binaries kept libxslt's default of 3000. A stylesheet that
    recursed between 1000 and 3000 levels deep now aborts in post-processing, as
    in Perl and in dynamically linked builds. Guard
    `xslt::max_depth_tests::transform_aborts_past_perls_recursion_depth`.
  - **BREAKING (default): packages and classes without a binding are read raw, under ar5iv's runaway
    limits.** `latexml_oxide`, the library API (`latexml::api`) and cortex's generic profile now raw-load
    an unbound `.sty`/`.cls` from the TeX tree, as `--includestyles` (Perl `INCLUDE_STYLES` +
    `INCLUDE_CLASSES`) did; a compiled binding still wins. They also take runaway limits: 480,000,000
    conditionals (`iflimit`, ten times ar5iv.sty's, since the count accumulates over a long document) and
    1,299,999 absorbed boxes (`absorblimit`, ar5iv.sty's), beside the existing 5,000,000
    pushback cap and the source-scaled token budget. Measured on a stratified 697-paper arXiv draw without
    the ar5iv preload: error-free papers 84.5 % → 95.7 %, Fatal papers 2 → 0, errors −71 %, words +1.0 %,
    no systematic slowdown; 40 TeX Live manuals error-free 22.5 % → 90 %. Format builds (`--init`)
    and the in-process `Converter`/`Config::default` keep Perl's default.
    **To get Perl LaTeXML's behaviour: `--noincludestyles`** (new, the negative of `--includestyles`; the
    last one wins), or the existing `--preload='[norawstyles,norawclasses]latexml.sty'` — bindings only, an
    unknown class served by OmniBus; latexml.sty's `iflimit`, `absorblimit` and `pushbacklimit`
    options set those limits, and `iflimit=0,absorblimit=0` lifts the two (a limit option of 0
    now lifts it, as in Perl). Guards
    `perfect_kernel_batchrawdefw::{cli_default_raw_loads_unbound_package_and_class,
    cli_noincludestyles_restores_perls_binding_only_loading, latexml_sty_iflimit_overrides_the_front_end_default}`
    (OXIDIZED_DESIGN_DIVERGENCES #1080).

  - **A class that copies `\author` or `\title` before redefining it no longer loops.** tocbase.cls
    (shipped with aic.cls and its siblings) does `\let\orig@author\author`, then redefines `\author`
    to collect the names and calls `\orig@author{\toc@authors}` at `\end{frontmatter}`. The copy
    replayed the class's later definition, so the author list contained itself
    (`Fatal:Stomach:Recursion`). A `\let` copy of a frontmatter setter taken before the class
    redefines it is now the setter as it stood, as in TeX, and restoring the setter from such a copy
    brings back the kernel's setter in the `\let`'s own scope. Witnesses 1811.06821, 1811.11810,
    1904.05269, 1905.07135, 2011.14174, 2311.03341, 1808.03157, 1911.12471, 2110.09373, 2110.14889,
    2204.12683, 2406.17657 (OXIDIZED_DESIGN_DIVERGENCES #1078). Guards
    `perfect_kernel_batchsetcopy::{a_copy_taken_before_the_redefinition_is_the_kernels_setter,
    a_copy_in_an_end_code_takes_the_class_list_once, a_copy_taken_after_the_redefinition_still_replays,
    a_restored_setter_takes_a_later_redefinition_afresh, a_restored_setter_drops_the_class_definition,
    a_restore_in_a_group_ends_with_it}`.

  - **A command that sets text at a paragraph's start keeps the space after
    it.** `\object{M31} is a galaxy.` (aa), `\url{http://x.org} is a site.`
    (llncs) and an amsppt `\cite` gave `M31is`, `x.orgis`, `[1]is` (as Perl
    does): the constructor stayed in vertical mode, where the space is dropped.
    Those bindings, and the others whose text TeX sets there (ams `\URL`,
    html's `\hyperrefhyper`, `\lxWithClass`, revtex's `\text`, a glyph with no
    Unicode, a text `\hphantom`), now enter horizontal mode
    first, Perl's `enterHorizontal`. fancybox's `\shadowbox`, `\doublebox`,
    `\ovalbox` and `\Ovalbox` also no longer split a paragraph in two
    (`Mid \doublebox{C} para.`). OXIDIZED_DESIGN_DIVERGENCES #1079; guards
    `perfect_kernel_batchinlinector`.

  - **Untrusted input stays in the job.** Document Lua code could run programs or write files outside the job; it
    is now sandboxed, under TeX's own defaults for untrusted input (no shell escape, paranoid writes, no write through
    a symbolic link, on Unix no backslash in a written name): a refused call never takes effect, and is reported as
    an Error on a best-effort basis (the bridge accepts only frames carrying a per-interpreter nonce, so a refusal
    record cannot be forged, but a chunk that exhausts its C stack or reads its own pipe can suppress its own record;
    a chunk lists at most 20 refusals, then one summary Error); each conversion has its own interpreter with a reduced environment, restarted (with an Error)
    when it ends mid-chunk and killed by a timeout or memory exit. A tar archive's links are unpacked only when
    they resolve inside the source directory, the others skipped with a counted Warning; the `zip` floor is 9.7.0. An image whose path relative to the source leaves the destination directory is copied under a
    generated `xN` name inside it, as Perl does. Guards
    `perfect_kernel_batchseclua::{batchseclua_lua_runs_no_program, batchseclua_lua_writes_stay_in_the_job_directory,
    batchseclua_image_copy_stays_in_dest, batchseclua_sandbox_helpers_are_held_at_boot,
    batchseclua_lua_writes_follow_no_symlink, batchseclua_lua_interpreter_restarts_and_owns_its_pipe,
    batchseclua_tar_links_stay_inside_the_source, batchseclua_lua_frames_are_authenticated,
    batchseclua_lua_backslash_names_are_refused, batchseclua_lua_refusals_are_bounded,
    batchseclua_lua_interpreter_ends_with_the_timeout}`.
