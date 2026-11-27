# latexml-oxide 0.8.1: the full list of changes

The release summary will be the `## [0.8.1]` section of [`CHANGELOG.md`](../../CHANGELOG.md). This file keeps every
entry the release's pull requests wrote, with its guard tests and design references (`OXIDIZED_DESIGN_DIVERGENCES #N`
is [`docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md`](../parity/OXIDIZED_DESIGN_DIVERGENCES.md)); a new entry goes at the
top of its group.

## Changes

### Unreleased

  - **KOMA-Script, versioned SIAM and dmtcs title notes print their fnsymbol marks and step the footnote counter, as the
    classes do.** A binding now answers for the class it actually serves — its own name, the request a versioned or prefix
    fallback answered with it (`siamltex1213`, `siamonline220329`), or the base a raw class file `\LoadClass`es while leaving
    the title block alone — so `scrartcl`/`scrarticle`, `scrreprt`, `scrbook`, `siamart`/`siamonline` and `dmtcs-episciences`
    title notes are marked ∗, †, ‡ … where they were unmarked or arabic, and `\footremember`/`\footrecall` keep their notes
    (OXIDIZED_DESIGN_DIVERGENCES #851). Guard `perfect_kernel_batchomnt`.
  - **autart's title page keeps no stray separators and no split `\thanks`.** The ", ", "and" and `\and` a template
    types between and after its `\author` and `\ead` calls, which autart.cls prints between the names, were text after
    the creators; they are now dropped as the names line's separators, while any other text there stays
    (1610.05434, 2602.06569). `\thanks[label]{…}` reads its label, so the note is the title's or the author's whose
    `\thanksref` names it, as elsart's binding links them, where the `[` was a note and the label and text ran into the
    title page, and `\thanksref` printed the key (2505.13064, 1105.0629); an unlabelled `\thanks` stays a document
    note. Guard `perfect_kernel_batchblockplace::autart_names_line_and_thanks_labels`.
  - **A `trim`/`viewport` crop of a rasterized PDF, EPS or PS keeps the region the PDF shows.** The crop was cut at
    72 pixels per inch whatever density the figure was rendered at, so at the default 120 dpi a `trim` removed 60 %
    of what it asked for, and a large figure rendered at the 2048-pixel cap (26 dpi for a 5669 bp page) lost 2.8 times
    too much (ar5iv issue 220, 2011.13011). It is now cut at the render density, the same value the render and its cache
    key use; a PNG or JPEG keeps its own resolution (OXIDIZED_DESIGN_DIVERGENCES #337). Guard
    `latexml_post graphics::tests::a_vector_render_is_cropped_at_its_density`.
  - **A delimited argument that runs off the end of the document no longer ends the conversion (`Fatal:Mouth:EoF`).**
    As in LaTeXML's Perl original, the tokens read go back to the input, the miss is an Error ("Missing argument
    Until:…") and the macro runs on an empty argument, so the text after the call is kept. A plain `{}` argument
    and a token argument that find no input left at the end of the document are Errors too ("Missing argument {}",
    "Missing argument Token"), and so is every other required argument (a `\label`, `\ref`, `\cite`, `\section` or
    `\mbox` argument: "Missing argument Semiverbatim", "Missing argument Undigested", "Missing argument
    HBoxArgContents"), which then runs on an empty argument as in Perl. A scanner that calls itself again
    after each miss (an open fancyvrb `{VerbatimOut}`, l3prg's map break over an undefined prop, a macro whose other
    arguments are undelimited) stops at the error cap, `Fatal:TooManyErrors`, keeping the text before it — except a
    `\futurelet` peek loop (it ends in `Fatal:Timeout:Recursion`) and a scanner whose re-reads double the put-back (it
    ends in `Fatal:Timeout:PushbackLimit`, which keeps nothing). An alignment left open to the end of the
    input still drops its last cell's text, as the Perl original does, and a Warning now counts it ("An alignment
    still open at the end of input drops its last cell's text, N word(s): …"). Guards
    `error_boundary::perfect_kernel_error_boundary` (the `Until:`, `{}`, `Token`, `Semiverbatim`, `Undigested`,
    `HBoxArgContents` and open-alignment rows),
    `perfect_kernel_batch48::until_miss_at_true_eof_errors_like_perl`,
    `perfect_kernel_batch52::until_miss_at_eof_reaches_the_error_cap`.
  - **Four bindings no longer read a delimited argument to the end of the file (`Fatal:Mouth:EoF`).** algpseudocodex
    loads algorithmicx and its own `pseudocodex` language, as the package does, with its keywords and blocks, so a
    nested `\Call` no longer runs ifthen's `\equal` loop off the end and its commands no longer give way to
    algpseudocode's (2512.12052); amsppt's `\subheading` takes an optional `\nofrills` and one undelimited argument
    (1412.5856, alg-geom9709005; KNOWN_PERL_ERRORS #1344); longtable's `\longtable` carries the real body's prefix and
    an `\LT@output` is defined, so the "longtable in twocolumn" preamble patch finds the `\LT@err` and `\@outputpage`s
    it splits at (1409.6572), and a two-column document's longtable logs longtable.sty's Error as an Info, the table
    kept; jmlr's `\addr` text ends at the `}` of `{\addr …}` (2409.18592). Guard
    `perfect_kernel_batcheofbind::eofbind_rows`.
  - **bmvc2k's institutions, bytedance_seed's affiliation and contribution lists, wlpeerj's affiliation and
    corresponding e-mail lines and sn-jnl's present address sit under the names, before the abstract, where each class
    prints them.** They were notes after the abstract, Perl's place for every note; each binding now declares them under
    the names (`\lx@names@note`), wlpeerj's e-mail after the affiliations whatever the order of the calls, and
    bytedance_seed's correspondence line, printed after the abstract, stays there (1507.06535, 2506.15211, 2007.11779;
    OXIDIZED_DESIGN_DIVERGENCES #1074). On the page, a contribution or present-address note with no mark shows its text
    as a line, where it was a lone `†` (bytedance_seed, elife's `\contrib`, 2311.00616; #1063). Guard
    `perfect_kernel_batchblockplace::a_block_contact_note_sits_under_the_names`.
  - **A Fatal during digestion keeps the text digested before it; the run stays Fatal.** A delimited argument that
    runs off the end of the input (`Fatal:Mouth:EoF`), an unbalanced definition (`Stomach:Misdefined`), a parameter or
    group Fatal, or a binding's own `Fatal()` wrote a 39-byte document, because a `document` environment is one
    body; the boxes of every level pending at the Fatal are now built, and the input after it stays unread. The
    levels are kept, not closed: an unfinished footnote, `\author{…}`, formula or list leaves its text in the running
    paragraph. The loop and resource guards are excluded: `Timeout:Recursion` and the input-level limit keep nothing
    (2605.25400), the stomach's runaway guards drop the runaway level, and the resource Fatals still end the run where
    they fire (OXIDIZED_DESIGN_DIVERGENCES #251). Guards:
    `fatal_salvages_partial_document::a_digestion_fatal_keeps_the_text_before_it`,
    `30_script_bindings::a_rhai_fatal_is_logged_once`.
  - **arydshln's dashed rules are drawn dashed.** A translation of upstream LaTeXML PR 2889 by Christopher Smolen:
    `\hdashline`, `\firsthdashline`, `\lasthdashline`, `\cdashline{a-b}` and the `:` and `;{dash/gap}` column rules
    are borders of the cells they rule, as `\hline`, `\cline` and `|` are, and each dashed side also gets the class
    `ltx_border_<side>_dashed`, which LaTeXML.css draws dashed (a side with a solid rule too stays solid). A `c`
    column before `:` is centred (was right-aligned, its cell a header guess), a leading `:` is the first column's
    left border with no gap left as text, a math array's `tex=` is its source (it held `\lx@add@cssclass`), and
    colortbl's `\arrayrulecolor` colours the dashed rules and is no longer undone when arydshln loads after colortbl
    (1909.02323's grey rules); `\dashgapcolor` and `\nodashgapcolor` exist, as in the package, only under colortbl;
    `\xleaders` stays TeX's after arydshln, and the binding is no longer reported as a stub
    (OXIDIZED_DESIGN_DIVERGENCES #1025, KNOWN_PERL_ERRORS #291). Guards
    `perfect_kernel_batcharydshln::{arydshln_dashed_borders_reach_the_html, arydshln_colortbl_rows}`, the
    `contrib/arydshln` golden.
  - **icml's notice legends are the notes of the names that show their mark.**
    `\printAffiliationsAndNotice{\icmlEqualContribution}` gave a note no author held, and each `equal` author an empty
    mark-only `*`; now each such author holds the legend as a note marked `*` ("Equal contribution"). Each legend of
    the notice (`\icmlEqualContribution, \icmlProjectLead`) goes to the names whose `\icmlsetsymbol` mark it opens with,
    one naming an institution too ("†Work done during internship at AWS AI", 2310.02777), one of a symbol holding a TeX
    special too (`\icmlsetsymbol{interns}{\#}`, 2306.03504). A notice holding a mark no `\icmlsetsymbol` label
    printed, a second mark the cut cannot reach (after `\quad`, `\\` or `\protect`, in a group, in math), or a
    conditional, an assignment or a footnote, stays whole, as before. `\icmlEqualContribution` ends with the space
    every icml20xx.sty gives it ("Equal contribution First", 2502.09860). 2602.04557, 1904.07793 (the
    corresponding author's e-mail kept), 2106.09776 (OXIDIZED_DESIGN_DIVERGENCES #1063, #954). Guards
    `perfect_kernel_batchdateicml::{icml_notice_legends_are_the_marked_names_notes, icml_notice_read_by_meaning}`.
  - **AAAI's `\affiliations` lines are the authors' affiliations.** aaai21-aaai2027's `\affiliations` redefines itself
    as its store, `\affiliations_`, which the raw class's title page prints; its lines stayed a block under the names.
    Each line now goes to the names whose marks request its label, an unlabelled institution to every author, as do
    the lines of a raw file's plain `\affiliations` store its title page prints (rlj.sty: 2605.26012); under
    `[submission]`, whose title page prints no affiliation, nothing moves. For every raw class whose title page prints
    such a store (aaai, rstransa, iau, alta, eph; caosp's numbered `\institute` entries stay as they were): a closing
    line of e-mails after the last label is the whole block's, not that label's — each address to the name it spells,
    else one per author in order where the spelled ones agree, else the block's, a lone address without its list's
    comma — unless the lines put each label's e-mails under it; a line (a document macro's text too) printing a list
    of addresses the reader cannot read is the author block's line as it prints, the rest still the authors'; an
    address inside a line of other words stays with its entry.
    2501.02471, 2511.12066, 2212.00442, 2508.05165, 2404.14949, 2511.11817, 2511.02997; 2211.16869, 2212.12190,
    2408.10349, 2412.03107 (OXIDIZED_DESIGN_DIVERGENCES #767, formerly #955). A raw class's
    affiliation store that no author carries (naverlabseurope.cls's `\affiliations` beside its own `\authors`,
    2408.05088) is counted in `Warning:frontmatter:unclaimed_store` instead of dropped silently. Guards
    `perfect_kernel_batchaaailabel::aaai_affiliations_rows`, `perfect_kernel_batch108b::a_store_of_a_refused_replay_is_counted`.
  - **The MSC edition of an AMS class's or amsppt's `\subjclass` follows its codes.** The call's year, or the
    author's own `\subjclassname` (or amsppt `\subjclassyear`) that the class discards, stands when every code is in
    that edition; codes that fit one edition name it; a printed year the codes refute gives the edition current at the
    source's date, or the nearest one that fits (undated, the oldest). The name and the scheme follow, the scheme alone
    where the class prints the author's own label (`\subjclass[2011]` after a renewal); each correction is an
    `Info:frontmatter:msc_edition`, each code in no edition an `Info:frontmatter:msc_unknown_code`. The table is the
    full list of MSC1991, 2000, 2010 and 2020 codes (identifiers only, with attribution). 1501.02464: 2020 → 2010 (its
    renewal); 0904.0450: 2020 → 2000; 2111.04526: 2010 → 2020 (57K12); 1204.4468: 2000 → 2010; dg-ga9604003:
    2020 → 1991 (OXIDIZED_DESIGN_DIVERGENCES #1168). Guard `perfect_kernel_batchmscinfer::mscinfer_rows`.
  - **An AMS class's `\subjclass` sets `\subjclassname` at the call, as amsart.cls does.** Each call names its
    edition's "Mathematics Subject Classification", over a `\subjclassname` the preamble renewed, which the class never
    prints (1501.02464 and 2503.20773 named the renewal's year); an edition the class does not know (`\subjclass[]`,
    `\subjclass[1985]`) keeps the name in force with the class's Warning "Unknown edition (…)" (1001.1600,
    1601.08172). A renewal after the call is still the name `\maketitle` prints (0811.1789). KNOWN_PERL_ERRORS #1341;
    guard `perfect_kernel_batchthmedpsres::thmedpsres_rows`.
  - **An `\halign` in a TikZ node's text or formula is set as outside a picture: a Young diagram or an `\ooalign`
    symbol in a tikz-cd cell or label keeps its place.** Inside a picture `\halign` was the SVG matrix everywhere, so
    ytableau's `\vtop{…\halign…}` or an `\ooalign` in a node's text opened SVG groups in the node's content: the
    formula lost the alignment, or closed early with malformed-markup Errors (2604.27868 reached the error limit, a
    Fatal, and lost more than half its text).
    Text leaving a picture (`\pgfinterruptpicture`: nodes, labels, pins, edge nodes, node parts, `\pgftext`) now has
    TeX's `\halign` back, as tikz uninstalls its own commands there; tikz's `align=` lines and pgf matrices keep the SVG
    matrix, and `\lxSVG@halign` reverts as `\halign`. 1502.07025: 147 Errors → 1; 1901.06584: 57 → 0; 2604.27868: 1001
    Errors and a Fatal → 0; 2409.06632: 28 → 0; 2409.04903: 20 → 0 (OXIDIZED_DESIGN_DIVERGENCES #1167,
    KNOWN_PERL_ERRORS #1340). Guards `perfect_kernel_batchhalignint::text_leaving_a_picture_sets_halign_as_tex`,
    `perfect_kernel_batchpanic397::a_nested_ooalign_in_node_math_reverts`.
  - **thmtools' list filters, edpsmath's keywords, amsart's default MSC edition, and an `\item[…]` label that steps
    its own counter.** Under thmtools, thm-listof's `\showtheorems`, `\ignoretheorems`, `\onlynamedtheorems` and
    `\setlisttheoremstyle` are defined and `\listoftheorems[…]` reads its keys (`ignoreall`, `show`, `ignore`,
    `showall`, `onlynamed`, `title`), so a list holds only the theorems it names (2408.09583's "List of Models" listed
    every theorem); `\onlynamedtheorems` lists the whole set, with a Warning. edpsmath's `\keywords` (EDP Sciences,
    shipped with the paper) is the frontmatter's keywords, named "Keywords and phrases.": they were a footnote of the
    body (1901.04804) or, under `[proc]`, which prints none, lost silently (1110.0287; FIDELITY_CHARTER §2). Under the
    AMS classes a `\subjclass` without a year names the 2020 edition, as amsart.cls does, at each call (was 1991, or
    the year an earlier call gave; KNOWN_PERL_ERRORS #1337). `\item[\refstepcounter{\@listctr}…]`, which made its
    tags from itself without end, converts (1704.06431: `Fatal:Stomach:Recursion`, now 0 errors; KNOWN_PERL_ERRORS
    #1338). Guard: `perfect_kernel_batchthmedpsres::thmedpsres_rows`.
  - **A list's `\linewidth` is its line, as LaTeX sets it.** `\list` and every environment that is a list in LaTeX
    (itemize, enumerate, description, quote, quotation, verse, changepage's `{adjustwidth}`) now set latex.ltx's list
    registers: `\leftmargin` from `\@list<depth>`, the line narrowed by both margins, `\@totalleftmargin`; the standard
    classes set `\leftmargini`..`\leftmarginvi`, `\labelsep` and `\labelwidth` (article.cls:322-339) in ems of their
    size, and a center, theorem or proof clears the margins. Before, every margin was 0pt (as in Perl), so a
    `width=\linewidth` graphic in a list was 7-20 % too wide, and MDPI's `{adjustwidth}{-\extralength}{0cm}`
    full-width figures kept the text's width (`\extralength` is now the class's 4.61cm). A footnote, a float, a
    minipage or a `p` cell inside a list sets its own line; panels in an `{adjustwidth}` share its line (2405.20768,
    2408.06972); enumitem's stated `leftmargin`/`rightmargin` and its `wide` are read; a label's leading glue nets
    against `\labelsep`, as LaTeX's `\@item` cancels it (a syntax.sty grammar's tags open with no space, 2605.07451);
    a footnote's or a `p` cell's paragraphs in a list are unspaced. An A/B of 212 papers that use lists with `\linewidth` or `{adjustwidth}`
    moved 22, with equal errors, warnings and words: 19 in widths, heights, `panelwidth`/`panelline` and SVG geometry
    only; 1703.00699's figure is one row, as in the PDF; 2306.09542 sets one more line mark in a narrower list item;
    1508.03599's proof heads lose a leading space. With 39 enumitem `wide` users and the witnesses, 252 papers moved 55,
    errors and words equal: the 22 above as they were, 2605.07451 unchanged, 31 `wide` users' lists
    `--ltx-enum-leftmargin:0pt` (1810.12214's `\setlist{leftmargin=5em,…,wide}` too; 2503.04577's description heads
    also without a leading en space), 2404.15303's table heights, 1907.01194's blank `\item[~]` labels dropped as an
    `\item[]`'s. A description label's opening `\hspace\labelsep` is the skip that nets away, its own `~` or
    `\quad` kept, and an itemize or enumerate nested in a description labels with its own label, not the
    description's bold one (a paralist `{compactenum}`, 1406.1992). `\list` resets `\makelabel` after the class's
    `\@list<depth>`, as LaTeX's does: a class that sets it there (tlp.cls) no longer gives every bullet and number a
    zero-width box (1210.0368: none of its tags, 34 on the base), a kernel `\list` inside a description is not bold
    (2206.12224's `\item[-]`), a document's own two-argument `\makelabel` no longer reaches list items, and an
    enumitem `\newlist{…}{description}` list without a `font`/`format` key is bold. With tlp.cls papers, enumitem
    `\newlist` users and documents with a `{list}` and a `{description}`, 769 papers moved 87, errors, Fatals and
    warnings equal or fewer, two words more (0907.4547's "Proof: The"); of them, the 60 of 411 above byte-identical
    but 1210.0368.
    Guard `perfect_kernel_batchqa14k2::list_registers_set_the_line_width` (OXIDIZED_DESIGN_DIVERGENCES #29).
  - **Rhai bindings: a register getter's or a `fontTest`'s `Fatal()` stops the conversion, a rewrite's keeps the
    document, and ligature code may assign values.** A getter's Fatal was logged but dropped (the register read unset
    and the whole document was built), and so was a `fontTest`'s Fatal raised while a `\footnote` closed the text before
    it; each is now raised, printed once, by the core step after it, as Perl dies there: in digestion or the build it
    stops the conversion; in a `DefRewrite` rule it ends the rewrites, and the document is post-processed with status 3
    (a rule's own `Fatal()` lost the whole document to an extra `Error:document:convert`). A `fontTest` or a
    math-ligature matcher that assigns (`AssignValue`, `DefLigature`, `DefMathLigature`) panicked ("RefCell already
    borrowed", "checkin_value should only be called after checkout_value") and lost the document
    (OXIDIZED_DESIGN_DIVERGENCES #1028). Two changes reach every conversion: a logged Fatal from any rewrite rule,
    native or Rhai, now keeps the document, and the math ligatures retry every remaining matcher after a match, as Perl
    does (`Document.pm:1174-1183`). Without a script binding, the output measured byte-identical (the four
    `docs/examples` fixtures, 2103.04797, math0211159), at most +0.06 % instructions. Guards:
    `30_script_bindings::a_rhai_fatal_is_logged_once`, `perfect_kernel_batchw3rd::rhai_thin_wrappers`.
  - **Tests: guards read whole elements.** The shared test helpers (`latexml_oxide/src/util/test.rs`) gain
    `xml_elements`, `element_count`, `assert_all_elements`, `assert_no_element` and `markup_text`, which match an
    attribute at its start (`id="p1"` is not `xml:id="p1"`), skip comments, CDATA and processing instructions, and read
    an element whole; `assert_no_element` also fails on a matching start tag left open. Listing guards pin whole
    `<listingline>` elements. `LATEXML_TEST_SHADOW=<absolute dir>` logs every `xml_element` call that disagrees with the
    new reading (0 on the whole suite) and fails when it cannot write. No conversion output changes.
  - **amsppt (AMS-TeX preprint) references keep their fields.** Each `\ref` field is digested as amsppt's `\makerefbox`
    boxes it, so field macros that assign (math0610862's `\noOf`) no longer misfire: math0610862 and math0205237 no
    longer end in `Fatal:TooManyErrors` (101 Errors → 2; +1,985 and +2,144 words), and math9204232's 5 references are
    back. A field that prints only a rule (`\bysame`) is kept; a `\ref` left open is closed by its next `\par`, `\ref`
    or `\endRefs` with AMS-TeX's Error and every field kept; `\newline` in a reference is a line break; a field marker
    inside a group the field opened no longer invents a picture; the `\paper` italic no longer leaks over the journal
    and year. Guards: `perfect_kernel_batchamsppt`.
  - **hyperref keeps the kernel's `\refstepcounter` on a format from 2024-11-01, as hyperref.sty does, and sets
    cleveref's label data through its `hyperref/fixcleveref` plug.** A document's
    `\let\H@refstepcounter\refstepcounter` no longer loops (2609.30372: a `Timeout:Recursion` Fatal, now no Fatal);
    with cleveref loaded before hyperref only cleveref's own load-order Error remains, as in pdflatex (1811.08194,
    1909.08399), and its typed `\refstepcounter[thm]{myc}` no longer prints `thm]myc`. Such documents now run cleveref's
    own `\refstepcounter`, as pdflatex does (1811.08194: +7.3 % instructions, the cost the other load order already
    paid). Guard: `perfect_kernel_batchloopfix::hyperref_leaves_refstepcounter_to_the_kernel`.
  - **Fewer `unsafe` blocks (issue 937, phase 2 step 1).** The glibc heap statistics (`mallinfo2`) and mimalloc's page
    purge (`mi_collect`) are each called from one safe function in `heap_trim.rs` (`c_heap_mb`, `collect_rust_heap`);
    their other call sites are safe code: 69 → 65 `unsafe` keywords. Output byte-identical; instructions and syscalls
    unchanged.
  - **An `\item[label]` in `{thebibliography}` is an entry labelled as written,** as LaTeX prints it (asl.cls author
    groups, backref's `\item[\@biblabel{1}]`, `\item[]`, footnote marks): the label was printed twice ("[1] [[A1]] …")
    and each later entry, and its citations, numbered one too high, as in Perl (KNOWN_PERL_ERRORS #1335). Witnesses
    1005.4193, 0705.0395, 1211.3435, quant-ph0409153 (`\cite{audenaert}` now [25], as the PDF).
  - **manyfoot, nccfoots and bigfoot are bound.** A declared series' `\footnote<X>` is a footnote marked as the series
    prints it (its counter and `\thefootnote<X>` style, `[<n>]`, a minipage too); `\footnotetext<X>` and
    `\Footnotetext<X>{<mark>}` are footnote texts marked as given, never taken for the text of a main `\footnotemark` of
    the same number; bigfoot's series too. The `\endminipage` Warning on every manyfoot or bigfoot load is gone, and
    manyfoot's begin-document definitions (`\MFL@fudgefactor`, `\MFL@paraskip`, `\defaultfootnoterule`) are kept for the
    packages that read them. Witnesses 2511.01971 (an e-mail line in `\author`, lost before), 2609.01974. Guard:
    `footnote_series::footnote_series_rows`.
  - **Identical images keep their own paths.** Two `\includegraphics` of different files with the same bytes each
    link their own file, as Perl does (`Post/Graphics.pm` L289-300), also under `--sitedirectory`; before, both linked
    the first file's path, so a page silently depended on another folder's file. Converted images and generated `xN`
    copies still share one file per content (OXIDIZED_DESIGN_DIVERGENCES #24).
  - **One longtable caption.** A `\caption[]{… (continued)}` in `\endhead` no longer shows as a second caption at the
    table's foot, and a `\endfoot` caption after `\endlastfoot` no longer moves to the head (Perl does both;
    KNOWN_PERL_ERRORS #1333).
  - **`\usepackage[svgpath={figs/}]{svg}` finds its SVGs,** reading the option as svg.sty's `\svgpath{{…}}`
    (svg.sty:546-556). As in pdflatex, the option holds one path: `svgpath={{a/}{b/}}` is not a list.
  - **Split pages no longer overwrite each other or leave the destination.** Under `--splitnaming=label`/
    `labelrelative`, two labels that name the same file (`dup` and `dup/`, `./x` and `x`, or `index` beside
    `index.html`) wrote both pages to one file, losing one (1502.05726 lost two appendix subsections); the second page
    now takes its id (`A1.SS2.html`), with a Warning. A label that would put its page outside the destination
    (`\label{../esc}`) is an Error, and the page is written inside under its id. Under `labelrelative` an unlabelled
    page is named relative to its parent's id, as in LaTeXML (`sec/SS1.html`, was `sec/S1.SS1.html`). One page namer
    serves both split paths (OXIDIZED_DESIGN_DIVERGENCES #1164).
  - **Split pages are named in Perl's order, each page before its children,** on both split paths: when two labels
    give the same file, the earlier page in the document keeps it and the later one takes its id (Perl writes both to
    one file, which keeps the later page). Under `--splitnaming=labelrelative`, a page whose label climbs out of its
    parent's folder (`\label{../up}`) now gets links that resolve. Collision and escape messages print page paths
    relative to the destination, EPUB included, and print a bare destination's directory as `.`.
  - **Eleven fewer `unsafe` sites (issue 937, phases 1 and 3):** the cortex log mirror, the crash-cause record, `getpid`
    and the watchdog's stderr line use safe `rustix` calls; `getrusage` has one call site; `--init` no longer writes
    `LATEXML_INI_MODE` into the process environment (an in-process switch; the variable is still read). 69 `unsafe`
    keywords remain (80 before). Output is byte-identical; user instructions −0.4/−0.5 %, task-clock and system-call
    counts unchanged.
  - **`\graphicspath` and `svgpath` entries that are absolute on Windows (`C:/figs/`) are searched as written.** They
    were recognized by a leading `/`, so a drive-letter entry was joined under the source directory and its figures
    went missing; both now ask `pathname::is_absolute`, Perl's `pathname_is_absolute`. Guard:
    `perfect_kernel_batch79g::an_absolute_graphics_path_is_searched_as_written` (red on Windows).
  - **The graphics cache delivers its images by copy, not hard link, and prunes to 90 % of its cap.** An output image
    edited in place no longer rewrites the cache entry and every other document's copy; an insertion no longer walks
    the whole cache (only past the cap, by a usage tally). Output byte-identical. Guards:
    `graphics_cache::tests::{an_output_edited_in_place_leaves_the_cache_and_its_siblings,
    a_prune_leaves_the_low_water_mark_and_a_later_insertion_walks_nothing}`.
  - **Three fewer `unsafe` sites (issue 937, phase 5):** the `unsafe impl Send/Sync` markers on `State` and `Stored` are
    gone; the ratchet `unsafe_sites_ratchet::unsafe_sites_only_fall` falls 80 → 77. Output byte-identical.
  - **cortex_worker: `--max-rss-mb` sets the cooperative memory fuse and the streaming watermarks for standalone and
    pool workers, as it already did for harness children; `LATEXML_RSS_CAP_BYTES` no longer overrides the flag.** A
    worker's memory messages name `--max-rss-mb`, or no flag, where they advised `--max-memory`, which `cortex_worker`
    does not accept; `tools/benchmark_canvas.sh` passes the fleet child's 5376 MiB. Guards: `cortex_worker`
    `cli_tests::max_rss_mb_sets_the_fuse_and_the_watermarks_in_every_worker_mode`,
    `cortex_worker_untrusted_bindings::a_standalone_workers_max_rss_mb_sets_its_fuse`, `stomach::memory_cap_tests::
    {the_fuse_fatal_names_the_knob_that_set_it, the_box_cap_fatals_name_the_knob_that_set_it}`,
    `watchdog::tests::the_oom_fatal_names_the_knob_that_set_it`.
