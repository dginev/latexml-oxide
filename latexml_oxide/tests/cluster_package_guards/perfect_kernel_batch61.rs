//! Red/green guards for perfect-kernel phase-61 batches: G3 residual slices 7a/7b (T1/T2 ASCII slots, the Unicode
//! profiles' OpenType flag, `\pdfsetmatrix`'s matrix, copyright lines a class prints only in a page foot).
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements},
};

/// 61c: T1 and T2A/T2B/T2C put `\textasciicircum`/`\textasciitilde` in slots 94/126 (t1enc.def:141-142,
/// t2aenc.def:83,88); the fontmaps decoded them as the spacing accents ˆ ˜ (Perl t1.fontmap.ltxml:30,34 alike), so a
/// `\string^` (Verbatim, listings, l3doc) printed a modifier letter. pdflatex prints `^ ~`; LY1 keeps its accents there
/// (ly1enc.def:99,106). Witness precattl ("edef" under l3doc's T1). Repro fonts-nfss/t1_ascii_slots_print_ascii.
#[test]
fn t1_ascii_slots_print_ascii() {
  for enc in ["T1", "T2A", "T2B", "T2C"] {
    let tex = include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/t1_ascii_slots_print_ascii.tex"
    )
    .replace("[T1]", &format!("[{enc}]"));
    let xml = assert_elements(&tex, RAW, (0, 0), &[]);
    assert_element(
      &xml,
      "p",
      &[],
      r#"<p>R &lt;a&gt; ^ ~ <text font="typewriter">T &lt;a&gt; ^ ~</text></p>"#,
    );
  }
}

/// 61d: the `luatex`/`xetex` profiles are OpenType engines: `\sys_if_engine_opentype` (expl3-code.tex:7863-7865,
/// `\cs_if_exist_p:N \tex_Umathcode:D`) froze false at format time and l3doc.cls:436-453 took its T1 + lmodern branch
/// (broydensolve, joinbox, ltx-talk, precattl, saveenv). The pdfTeX model stays an eight-bit engine. lualatex and
/// xelatex print "Unicode engine", pdflatex "Eightbit engine". Repro luatex-profile/unicode_profiles_are_opentype_engines.
#[test]
fn unicode_profiles_are_opentype_engines() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/luatex-profile/unicode_profiles_are_opentype_engines.tex"
  );
  for (preload, expected) in [
    (
      "[luatex,rawstyles,rawclasses]latexml.sty",
      "<p>Unicode engine</p>",
    ),
    (
      "[xetex,rawstyles,rawclasses]latexml.sty",
      "<p>Unicode engine</p>",
    ),
    (RAW, "<p>Eightbit engine</p>"),
  ] {
    let (stderr, xml) = convert_with(tex, Some(preload));
    assert_eq!(
      (error_count(&stderr), warning_count(&stderr)),
      (0, 0),
      "{stderr}"
    );
    assert_element(&xml, "p", &[], expected);
  }
}

/// 61e: `\pdfsetmatrix {<matrix>}` reads its matrix (pdfTeX manual, pdftex.tex:3253-3264); Perl pdfTeX.pool:223
/// read nothing and the matrix printed (synthslant-gauge "1 0 .05 1" in each cell). pdflatex prints "ABC". Repro
/// fonts-nfss/pdfsetmatrix_reads_its_matrix.
#[test]
fn pdfsetmatrix_reads_its_matrix() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/pdfsetmatrix_reads_its_matrix.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "p", &[], "<p>ABC</p>");
}

/// 61f: a copyright line a class prints only in a page foot is a frontmatter note (user ruling 2026-10-01): ltnews's
/// `\@indicia` (`\ps@titlepage`, ltnews.cls:461-488; 42 issues of the LaTeX News) and knittingpattern's `\cpyrght`
/// (`\fancyfoot[R]`, knittingpattern.cls:60-68). Both engines dropped them (SHARED). Repros
/// sectioning-frontmatter/{ltnews_copyright_is_a_note, knittingpattern_copyright_is_a_note}.
#[test]
fn class_copyright_lines_are_frontmatter_notes() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ltnews_copyright_is_a_note.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  let logo = r#"<text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text>"#;
  assert_element(
    &xml,
    "note",
    &[r#"role="copyright""#],
    &format!(
      "<note role=\"copyright\">{logo}\u{a0}News, and the {logo} software,\nare brought to you by the {logo} Project \
       Team;\nCopyright 2007, license LPPL.\n</note>"
    ),
  );
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/knittingpattern_copyright_is_a_note.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="copyright""#],
    r#"<note role="copyright">©2010 Hugh Griffiths</note>"#,
  );
}

/// 61g: OT4 keeps OT1's accents in slots 94/126 (ot4enc.def:54,56); its fontmap decoded 94 as `_` and 126 as ASCII
/// `~` (Perl ot4.fontmap.ltxml:30 alike). pdflatex prints "aˆb˜c". Repro fonts-nfss/ot4_accent_slots_are_accents.
#[test]
fn ot4_accent_slots_are_accents() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/ot4_accent_slots_are_accents.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "p", &[], "<p>a\u{02C6}b\u{02DC}c</p>");
}

/// 61g: a document's authors reach the HTML without a title (OXIDIZED_DESIGN_DIVERGENCES #434): exam-n prints its
/// `\author` as "Author: …" (exam-n.cls:1327), LaTeXML keeps it as a `creator` (`\author` is locked), and the structure
/// XSLT rendered authors only from the title template, so a titleless document lost the name (SHARED). Repro
/// sectioning-frontmatter/authors_without_a_title_reach_the_html.
#[test]
fn authors_without_a_title_reach_the_html() {
  let (stderr, html) = super::perfect_kernel_batch46::convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authors_without_a_title_reach_the_html.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "div",
    &[r#"class="ltx_authors""#],
    "<div class=\"ltx_authors\">\n<span class=\"ltx_creator ltx_role_author\">\n<span class=\"ltx_personname\">Frieda \
     Bloggs\n</span></span></div>",
  );
}

/// 61k: a brief.cls letter's sender is its frontmatter (user ruling 2026-10-01, as g-brief's, DIVERGENCES #412): the
/// class prints `\maakbriefhoofd`'s name and address and the `\voetitem` foot only in `\ps@firstpage` (brief.cls:285-294,
/// :437-468), which LaTeXML never typesets (SHARED). A foot label's line break is a space in the contact's name. Repro
/// sectioning-frontmatter/brief_letter_sender_is_frontmatter.
#[test]
fn brief_letter_sender_is_frontmatter() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/brief_letter_sender_is_frontmatter.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "creator",
    &[r#"role="sender""#],
    r#"<creator role="sender"><personname>WG 13</personname><contact role="address">Werkgroep 13<break/>de De Facto Standaard</contact><contact name="fax:">12345 abc</contact><contact name="telefoon privé:">080-448664</contact></creator>"#,
  );
}

/// 61k: an `\input` issued while a raw package is read keeps the current catcodes (TeX's `\input`): CoverPage.sty:58-70
/// makes `@` the escape character and inputs `\jobname.BibTeX.txt`, so that `@article{…}` runs `\article`; the
/// definitions mouth forced `@` back to a letter and the cover page printed "title undefined" (Perl alike). pdflatex:
/// "Title: Some Waste of Paper. Source: in: Irreality Journal." Repro loader/input_from_a_package_keeps_the_catcodes.
#[test]
fn input_from_a_package_keeps_the_catcodes() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/input_from_a_package_keeps_the_catcodes.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Title: Some Waste of Paper. Source: in: Irreality Journal. See also B<text font="smallcaps">ib</text>T<text yoffset="-3.0pt">E</text>X entry below.</p></para>"#,
  );
}

/// 61k: keyval reports an unknown key through `\KV@errx` (keyval.sty:41-42), which a package may redefine to do
/// nothing for its own `\setkeys` — CoverPage.sty:128 ignores a BibTeX record's other fields so. The native
/// `\setkeys` stays silent then, and still warns when `\KV@errx` is keyval's (or undefined).
#[test]
fn keyval_unknown_keys_follow_kv_errx() {
  let quiet = r"\documentclass{article}
\usepackage{keyval}
\makeatletter\define@key{F}{a}{A=#1}
\begin{document}
{\def\KV@errx#1{\relax}\setkeys{F}{a=1,b=2}}
\end{document}
";
  let (stderr, _) = convert_with(quiet, Some(RAW));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  let (stderr, _) = convert_with(&quiet.replace(r"\def\KV@errx#1{\relax}", ""), Some(RAW));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 1),
    "{stderr}"
  );
  assert!(stderr.contains("unknown KeyVals key 'b'"), "{stderr}");
}

/// 61m: a bfh-ci title page's foot is frontmatter (user ruling 2026-10-03): bfhlayout.sty prints `\department`,
/// `\institute` and `\titlefooterright` only in the title page's scrlayer footer (bfhlayout.sty:735-756), which
/// LaTeXML never typesets (SHARED). The three warnings are the class's own. Repro
/// sectioning-frontmatter/bfh_title_footer_is_frontmatter.
#[test]
fn bfh_title_footer_is_frontmatter() {
  let xml = super::perfect_kernel_batch57::assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bfh_title_footer_is_frontmatter.tex"
    ),
    RAW,
    (0, 3),
    &[
      "`\\@startsection' has been changed",
      "Unexpected definition of \\@sect",
      "You are using pdfLaTeX",
    ],
    &[],
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="department""#],
    r#"<note role="department">Applied Physics Unit</note>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="titlefooter""#],
    r#"<note role="titlefooter">Project Homepage</note>"#,
  );
}

/// 61m: a raw package a binding reads keeps its title-page setters raw (`store_setters::mark_raw`): bfhthesis.cls's
/// `\@maketitle` prints bfhlayout.sty's `\department`/`\institute` under the authors, so the kernel hands them to the
/// creators as affiliations; read by bfhlayout_sty.rs, they were taken for the binding's setters and lost
/// (DEMO-BFHThesis 95.2 -> 85.7 %), and the binding's footer notes are not repeated for a handed store. pdflatex:
/// "Anne Author", "▶ Technik und Informatik", "▶ Mikro- und Medizintechnik". Repro
/// sectioning-frontmatter/bfh_thesis_stores_stay_affiliations.
#[test]
fn bfh_thesis_stores_stay_affiliations() {
  let xml = super::perfect_kernel_batch57::assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bfh_thesis_stores_stay_affiliations.tex"
    ),
    RAW,
    (0, 3),
    &[
      "`\\@startsection' has been changed",
      "Unexpected definition of \\@sect",
      "You are using pdfLaTeX",
    ],
    &[],
  );
  assert_element(
    &xml,
    "creator",
    &[],
    &r#"<creator role="author"><personname>Anne Author</personname><contact name="Affiliation:~" role="affiliation">Bern University of Applied Sciences</contact><contact name="Affiliation:~" role="affiliation">Technik und Informatik</contact><contact name="Affiliation:~" role="affiliation">Mikro- und Medizintechnik</contact></creator>"#
      .replace('~', "\u{a0}"),
  );
  assert!(!xml.contains("<note role=\"department\""), "{xml}");
}

/// 61m: resphilosophica prints the `\thanks` text under "Acknowledgments" (`\enddoc@text`,
/// resphilosophica.cls:436-449); the frontmatter thanks note carries that name instead of "Thanks:" (user ruling
/// 2026-10-03). pdflatex: "Acknowledgments Supported by a grant." Repro
/// sectioning-frontmatter/resphilosophica_thanks_are_acknowledgments.
#[test]
fn resphilosophica_thanks_are_acknowledgments() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/resphilosophica_thanks_are_acknowledgments.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "pubnote",
    &[r#"role="thanks""#],
    "<pubnote name=\"Acknowledgments\u{a0}\" role=\"thanks\">Supported by a grant.</pubnote>",
  );
}

/// 61m: tex-label's `\labels` keywords, printed only in the page foot (tex-label.sty:30-32), are the frontmatter's
/// keywords, each page's after the last (user ruling 2026-10-03). Repro
/// sectioning-frontmatter/tex_label_keywords_are_frontmatter.
#[test]
fn tex_label_keywords_are_frontmatter() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/tex_label_keywords_are_frontmatter.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Labels:\u{a0}\">sample page, demo, tex-labels; second page, demo, labels</keywords>",
  );
  // Its own entry: an author's `\keywords` (amsart) stays beside it.
  let xml = assert_elements(
    r"\documentclass{amsart}
\usepackage{tex-label}
\keywords{alpha, beta}
\begin{document}
\labels{gamma}
Text.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  for keywords in [
    "<keywords name=\"Key words and phrases:\u{a0}\">alpha, beta</keywords>",
    "<keywords name=\"Labels:\u{a0}\">gamma</keywords>",
  ] {
    assert!(xml.contains(keywords), "{keywords}\n{xml}");
  }
}

/// 61m: imakeidx's named indexes (user ruling 2026-10-03): `\index[name]{…}` files the entry in list `name` and
/// `\printindex[name]` prints it under the title `\makeindex[name=,title=]` gave; the default index stays the
/// kernel's. pdflatex + makeindex: "Index / apple", "Subject Index / pear", "Author Index / Smith, John". Repro
/// index/imakeidx_named_indexes.
#[test]
fn imakeidx_named_indexes() {
  let (xml, log) = super::cluster::convert_and_post_contrib_logging(
    "../tools/perfect_kernel/repros/index/imakeidx_named_indexes.tex",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, expected) in [
    (
      "idx",
      r#"<index fragid="idx" xml:id="idx"><title>Index</title><indexlist><indexentry fragid="idx.apple" xml:id="idx.apple"><indexphrase key="apple">apple</indexphrase><indexrefs><text> </text><ref idref="p1" show="typerefnum">p1</ref></indexrefs></indexentry></indexlist></index>"#,
    ),
    (
      "idx.subject",
      r#"<index fragid="idx.subject" lists="subject" xml:id="idx.subject"><title>Subject Index</title><indexlist><indexentry fragid="idx.subject.pear" xml:id="idx.subject.pear"><indexphrase key="pear">pear</indexphrase><indexrefs><text> </text><ref idref="p1" show="typerefnum">p1</ref></indexrefs></indexentry></indexlist></index>"#,
    ),
    (
      "idx.authors",
      r#"<index fragid="idx.authors" lists="authors" xml:id="idx.authors"><title>Author Index</title><indexlist><indexentry fragid="idx.authors.SmithJohn" xml:id="idx.authors.SmithJohn"><indexphrase key="Smith, John">Smith, John</indexphrase><indexrefs><text> </text><ref idref="p1" show="typerefnum">p1</ref></indexrefs></indexentry></indexlist></index>"#,
    ),
  ] {
    assert_element(&xml, "index", &[&format!(r#"xml:id="{id}""#)], expected);
  }
}

/// 61m: an `\index` entry built by expl3 code holds control sequences whose names are not letters at the re-read
/// (`\bool_if:nT`): TeX expands them in the `\protected@write`, before the `.ind` re-read. The SanitizedVerbatim
/// re-read split them first — `\bool` undefined and `Script _` per entry (genealogy-profiles.sty:536-541, 300 errors
/// once imakeidx's named entries were kept). pdflatex + makeindex: "Z, 1 / W, 1" nested under Z.
#[test]
fn index_entry_keeps_expl3_names_whole() {
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
\ExplSyntaxOn
\cs_new:Nn \my_add:n { \index { #1 \bool_if:nT { \c_true_bool } { ! W } } }
Text\my_add:n { Z }
\ExplSyntaxOff
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "indexmark",
    &[],
    r#"<indexmark><indexphrase key="Z">Z</indexphrase><indexphrase key="W">W</indexphrase></indexmark>"#,
  );
}

/// 61m: the `\write` of an index entry is expanded again at shipout under `\let\protect\noexpand` (latex.ltx:20913
/// `\@outputpage`), so no `\protect` reaches the `.idx`: tikz-ext-manual's `\indexCommandO` writes
/// `\index{…\protect\string\protect#1}` as `\string\pgftext`. Kept, the `\pgftext` ran as the entry was read (501
/// `\egroup` errors once imakeidx's default entries reached the kernel's `\index`). pdflatex + makeindex: `.idx`
/// `\indexentry{\texttt {\string \fbox }}{1}`, index "\fbox, 1".
#[test]
fn index_entry_protect_is_consumed_at_shipout() {
  let xml = assert_elements(
    r"\documentclass{article}
\newcommand*{\indexCommandO}[1]{\index{\protect\texttt{\protect\string\protect#1}}}
\begin{document}
Text\indexCommandO{\fbox} more.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "indexmark",
    &[],
    r#"<indexmark><indexphrase key="\fbox"><text font="typewriter">\fbox</text></indexphrase></indexmark>"#,
  );
}

/// 61m: an entry typed in the source is read after `\@sanitize` made `\` other (latex.ltx `\index`, imakeidx.sty:164-168),
/// so its control words are written to the `.idx` verbatim and run only when the index is typeset; a control sequence
/// that was a token already (a macro-built entry) is expanded by the `\protected@write`. egpeirce-doc's visual index
/// (`\index[visual]{i@\ontop{…}\shk{1}…}`) ran its pstricks graphs inside the write expansion and timed out. pdflatex +
/// makeindex: `.idx` `\indexentry{a\early}{1}` and `\indexentry{bN}{1}`, index "aY, 1 / bN, 1".
#[test]
fn index_entry_typed_in_source_is_written_verbatim() {
  let xml = assert_elements(
    r"\documentclass{article}
\def\early{\ifx\protect\relax Y\else N\fi}
\newcommand\idx[1]{\index{#1}}
\begin{document}
Typed\index{a\early} and built\idx{b\early}.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>Typed<indexmark><indexphrase key="aY">aY</indexphrase></indexmark> and built<indexmark><indexphrase key="bN">bN</indexphrase></indexmark>.</p></para>"#,
  );
}

/// 61m: a named index lists its name's characters under any encoding: read as typeset text, an LGR document's
/// `\printindex[subject]` listed "συβθεςτ" and missed the marks' `inlist="subject"` (empty index).
#[test]
fn imakeidx_list_name_is_its_characters() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage[english,greek]{babel}
\usepackage{imakeidx}
\makeindex[name=subject,title=Subject]
\begin{document}
a\index[subject]{pear}
\printindex[subject]
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "index",
    &[],
    // The title is typeset text: LGR prints the Latin letters as Greek, as pdflatex does.
    r#"<index lists="subject" xml:id="idx.subject"><title>Συβθεςτ</title></index>"#,
  );
}

/// 61n: a binding that reads its own raw file passes no options to that read: the `\usepackage` recorded them, and
/// passing them again appended them to `\opt@<name>.<ext>` a second time, so `\ProcessOptions` ran each twice (eight
/// contrib bindings: imakeidx, doclicense, figbib, abntex2cite, refstyle, jurabib, directory). pdflatex: "Options:
/// noautomatic."
#[test]
fn binding_raw_read_records_its_options_once() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage[noautomatic]{imakeidx}
\begin{document}
\makeatletter
Options: \csname opt@imakeidx.sty\endcsname.
\makeatother
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>Options: noautomatic.</p></para>"#,
  );
}

/// 61o: crossreftools' label data (`\r@<label>`) and list of labels (`.lla`) come from a second LaTeX run; the binding
/// is that second pass per label (backward references; the list built after the document is read), and its `\label`
/// wrapper is put back around cleveref's. pdflatex (second run): "1 foo", "Shown text gen"; "See Shown text, Name
/// text, Shown text, Name text." Repro singletons/crossreftools_label_data_and_list.
#[test]
fn crossreftools_label_data_and_list() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/crossreftools_label_data_and_list.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "TOC",
    &[r#"class="ltx_listoflabels""#],
    r#"<TOC class="ltx_listoflabels"><toclist><tocentry><ref labelref="LABEL:foo">1 foo</ref></tocentry><tocentry><ref labelref="LABEL:gen">Shown text gen</ref></tocentry></toclist></TOC>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="Ch1.p1""#],
    r#"<para xml:id="Ch1.p1"><p>Name text See <ref labelref="LABEL:gen"/>, <ref labelref="LABEL:gen">Name text</ref>, Shown text, <ref labelref="LABEL:gen">Name text</ref>.</p></para>"#,
  );
}

/// 61o: the `\maketitle` replay gate counts what the class's title-page body itself defines (uiucthesis.cls:134-147
/// `\newcommand{\thesis@small}`, `\newdimen\thesis@dim`), and uiucthesis' uppercase title and author copies, made
/// by its own `\title`/`\author` that LaTeXML's frontmatter setters replace, are empty: the page is replayed.
/// pdflatex: "Submitted in partial fulfillment of the requirements / for the degree of Doctor of Philosophy in Food
/// Science / … Urbana-Champaign, 1994". Repro sectioning-frontmatter/uiucthesis_title_page_is_replayed.
#[test]
fn uiucthesis_title_page_is_replayed() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/uiucthesis_title_page_is_replayed.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  for line in [
    "Submitted in partial fulfillment of the requirements",
    "for the degree of Doctor of Philosophy in Food Science",
    "University of Illinois at Urbana-Champaign, 1994",
  ] {
    let p = format!(r#"<p><text fontsize="120%">{line}</text></p>"#);
    assert!(xml.contains(&p), "{p}\n{xml}");
  }
  // The title is the frontmatter's, once.
  assert_eq!(
    xml.matches("<title>Coffee Consumption</title>").count(),
    1,
    "{xml}"
  );
}

/// 61o: crossreftools without cleveref: the package's own `\label` wrapper is the only one (the binding re-wraps only
/// cleveref's `\label`; re-wrapping its own looped forever), a label's name is listed as written (`sec_a`), and a
/// backward `\crtrefcounter` reads the counter from the data's anchor (`section`). pdflatex (second run): "Counter:
/// section; number: 1." (its list typesets the `_` and errors).
#[test]
fn crossreftools_without_cleveref() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{hyperref}
\usepackage{crossreftools}
\begin{document}
\crtlistoflabels*
\section{A}\label{sec_a}
Counter: \crtrefcounter{sec_a}; number: \crtrefnumber{sec_a}.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "TOC",
    &[r#"class="ltx_listoflabels""#],
    r#"<TOC class="ltx_listoflabels"><toclist><tocentry><ref labelref="LABEL:sec_a">1 sec_a</ref></tocentry></toclist></TOC>"#,
  );
  assert!(xml.contains("<p>Counter: section; number: 1.</p>"), "{xml}");
}

/// 61p: `\trivlist\item[label]` keeps its label and its text (KNOWN_PERL_ERRORS #456): the itemization begins in the
/// current group, so `\item` stays bound until the environment ends. pdflatex: "Alpha one" / "Beta two" (Beta bold).
/// Witness webquiz's `heading` (webquiz.tex:49-51). Repro list-structure/trivlist_item_keeps_its_label.
#[test]
fn trivlist_item_keeps_its_label() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/trivlist_item_keeps_its_label.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag>Alpha</tag></tags><para xml:id="S0.I1.ix1.p1"><p>one</p></para></item></itemize><itemize class="ltx_trivlist" xml:id="S0.I2"><item xml:id="S0.I2.ix1"><tags><tag><text font="bold">Beta</text></tag></tags><para xml:id="S0.I2.ix1.p1"><p>two</p></para></item></itemize></para>"#,
  );
}

/// 61p: `{trivlist}` keeps both items in one list, the first tagged. pdflatex: "T x", then "y". Repro
/// list-structure/trivlist_environment_keeps_its_list.
#[test]
fn trivlist_environment_keeps_its_list() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/trivlist_environment_keeps_its_list.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag>T</tag></tags><para xml:id="S0.I1.ix1.p1"><p>x</p></para></item><item xml:id="S0.I1.ix2"><tags><tag/></tags><para xml:id="S0.I1.ix2.p1"><p>y</p></para></item></itemize></para>"#,
  );
}

/// 61p: a `\par` before a trivlist's first `\item` does not end the list (csquotes.tex:77-84's `{quotesample}`: a
/// blank line after its settings): a `\par` ends a paragraph, never a list open inside it (`\lx@normal@par`).
/// pdflatex: two items, then "After.".
#[test]
fn trivlist_par_before_the_first_item() {
  let xml = assert_elements(
    r"\documentclass{article}
\newenvironment{quotesample}{\trivlist\leftskip\parindent\small}{\endtrivlist}
\begin{document}
\begin{quotesample}
\newcommand{\x}{y}

\item First item.
\item Second item.
\end{quotesample}
After.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag/></tags><para xml:id="S0.I1.ix1.p1"><p><text fontsize="90%">First item.</text></p></para></item><item xml:id="S0.I1.ix2"><tags><tag/></tags><para xml:id="S0.I1.ix2.p1"><p><text fontsize="90%">Second item.</text></p></para></item></itemize><p>After.</p></para>"#,
  );
}

/// 61p: a class's `\@verbatim` that opens a trivlist (verbatim.sty:64; oblivoir's memucs-setspace.sty:588-591) has it
/// ended with the verbatim (verbatim.sty:88 `\endverbatim` = `\endtrivlist…`): left open, every later heading of the
/// oblivoir manuals nested in it (tzplot-doc, kotex-utf-doc errors). pdflatex: the verbatim, then "After.".
#[test]
fn verbatim_trivlist_ends_with_the_verbatim() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{verbatim}
\makeatletter
\def\@verbatim{\the\every@verbatim\trivlist \item \relax\verbatim@font}
\makeatother
\begin{document}
Before.
\begin{verbatim}
  code
\end{verbatim}
After.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Before.</p><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag/></tags><para xml:id="S0.I1.ix1.p1"><verbatim font="typewriter">code</verbatim></para></item></itemize><p>After.</p></para>"#,
  );
}

/// 61p: lists nest past the six `@item` levels the kernel declares (doc.sty's `\@doc@env` opens one trivlist per
/// documented name; frankenstein, source2e): the deeper levels are declared as they are reached ("\c@@itemvii is
/// not a register" otherwise).
#[test]
fn item_levels_past_six_are_declared() {
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
\begin{trivlist}\item[1]\begin{trivlist}\item[2]\begin{trivlist}\item[3]\begin{trivlist}\item[4]
\begin{trivlist}\item[5]\begin{trivlist}\item[6]\begin{trivlist}\item[7]\begin{trivlist}\item[8] deep
\end{trivlist}\end{trivlist}\end{trivlist}\end{trivlist}\end{trivlist}\end{trivlist}\end{trivlist}\end{trivlist}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1""#],
    r##"<item xml:id="S0.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1"><tags><tag>8</tag></tags><para xml:id="S0.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.I1.ix1.p1"><p>deep</p></para></item>"##,
  );
}

/// 61p review: trivlists that end in every way a source ends them, each clean and each closing only what it began:
/// unended before a `\section`, inside an `{itemize}` or a footnote (closed with them, as Perl's `_autoclose` list); the
/// paragraph before one ended first (latex.ltx `\@trivlist`); a bare pair in a `\list` item; bare pairs at the top
/// level (the list state they bound put back); doc.sty's several begun and one ended per group. pdflatex: each clean.
#[test]
fn trivlist_cases_end_cleanly() {
  let doc = |body: &str| {
    format!(
      "\\documentclass{{article}}\n\\newenvironment{{pf}}{{\\trivlist\\item[\\textbf{{Proof.}}]}}{{\\endtrivlist}}\n\
       \\begin{{document}}\n{body}\n\\end{{document}}"
    )
  };
  // An unended trivlist ends with its section.
  let xml = assert_elements(
    &doc(r"\section{A}\trivlist\item[Proof.] x \section{B} y"),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    r##"<para xml:id="S1.p1"><itemize class="ltx_trivlist" xml:id="S1.I1"><item xml:id="S1.I1.ix1"><tags><tag>Proof.</tag></tags><para xml:id="S1.I1.ix1.p1"><p>x</p></para></item></itemize></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="S2.p1""#],
    r##"<para xml:id="S2.p1"><p>y</p></para>"##,
  );
  // The heading puts back the list state too: the lists after it are its own (`\lx@trivlist@end@group`).
  let xml = assert_elements(
    &doc(
      r"\section{A}\trivlist\item[Proof.] x \section{B}\begin{itemize}\item c\end{itemize}\begin{enumerate}\item d\end{enumerate}",
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="S2.p1""#],
    r##"<para xml:id="S2.p1"><itemize xml:id="S2.I1"><item xml:id="S2.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S2.I1.i1.p1"><p>c</p></para></item></itemize><enumerate xml:id="S2.I2"><item xml:id="S2.I2.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S2.I2.i1.p1"><p>d</p></para></item></enumerate></para>"##,
  );
  // An unended trivlist ends with the itemize it is in.
  let xml = assert_elements(
    &doc(r"\begin{itemize}\item a \trivlist\item[b] c\end{itemize} d"),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>a</p><itemize class="ltx_trivlist" xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.ix1"><tags><tag>b</tag></tags><para xml:id="S0.I1.i1.I1.ix1.p1"><p>c</p></para></item></itemize></para></item></itemize><p>d</p></para>"##,
  );
  // An unended trivlist ends with its footnote.
  let xml = assert_elements(&doc(r"x\footnote{\trivlist\item[n] z} y"), RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "note",
    &[r#"xml:id="footnote1""#],
    r##"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag>n</tag></tags><para xml:id="S0.I1.ix1.p1"><p>z</p></para></item></itemize></note>"##,
  );
  // The paragraph a trivlist interrupts ends first.
  let xml = assert_elements(
    &doc(r"as follows. \begin{pf}body\end{pf} after"),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>as follows.</p><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag><text font="bold">Proof.</text></tag></tags><para xml:id="S0.I1.ix1.p1"><p>body</p></para></item></itemize><p>after</p></para>"##,
  );
  // A bare pair in a list item ends only itself.
  let xml = assert_elements(
    &doc(r"\begin{list}{}{}\item a \trivlist\item[b] c\endtrivlist d\item e\end{list} f"),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize><item xml:id="S0.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>a</p><itemize class="ltx_trivlist" xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.ix1"><tags><tag>b</tag></tags><para xml:id="S0.I1.i1.I1.ix1.p1"><p>c</p></para></item></itemize><p>d</p></para></item><item xml:id="S0.I1.i2"><tags><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I1.i2.p1"><p>e</p></para></item></itemize><p>f</p></para>"##,
  );
  // Bare pairs put the list state back: the itemize after them is not nested in them.
  let xml = assert_elements(
    &doc(
      r"\trivlist\item[P1] a\endtrivlist \trivlist\item[P2] b\endtrivlist \begin{itemize}\item c\end{itemize}",
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag>P1</tag></tags><para xml:id="S0.I1.ix1.p1"><p>a</p></para></item></itemize><itemize class="ltx_trivlist" xml:id="S0.I2"><item xml:id="S0.I2.ix1"><tags><tag>P2</tag></tags><para xml:id="S0.I2.ix1.p1"><p>b</p></para></item></itemize><itemize xml:id="S0.I3"><item xml:id="S0.I3.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I3.i1.p1"><p>c</p></para></item></itemize></para>"##,
  );
  // Three begun and one ended in a group (doc.sty): the group ends the others.
  let xml = assert_elements(
    &doc(
      r"\begingroup\trivlist\item[A]\trivlist\item[B]\trivlist\item[C] body\endtrivlist\endgroup After.",
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag>A</tag></tags><para xml:id="S0.I1.ix1.p1"><itemize class="ltx_trivlist" xml:id="S0.I1.ix1.I1"><item xml:id="S0.I1.ix1.I1.ix1"><tags><tag>B</tag></tags><para xml:id="S0.I1.ix1.I1.ix1.p1"><itemize class="ltx_trivlist" xml:id="S0.I1.ix1.I1.ix1.I1"><item xml:id="S0.I1.ix1.I1.ix1.I1.ix1"><tags><tag>C</tag></tags><para xml:id="S0.I1.ix1.I1.ix1.I1.ix1.p1"><p>body</p></para></item></itemize></para></item></itemize></para></item></itemize><p>After.</p></para>"##,
  );
}

/// 61p review: `{verbatim*}` ends at its `\end{verbatim*}`: the end line was matched with the name unescaped, its `*`
/// a quantifier, so the rest of the document was read as verbatim (RUST-ONLY; Perl quotes it, verbatim.sty.ltxml:91).
/// pdflatex: the starred verbatim, then the list with its own verbatim, then "H".
#[test]
fn starred_verbatim_ends_at_its_end() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{verbatim}
\begin{document}
A
\begin{verbatim*}
  two
\end{verbatim*}
\begin{itemize}\item in item
\begin{verbatim}
four
\end{verbatim}
\item next
\end{itemize}
H
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>A</p><verbatim font="typewriter">two</verbatim><itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>in item</p><verbatim font="typewriter">four</verbatim></para></item><item xml:id="S0.I1.i2"><tags><tag>•</tag><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I1.i2.p1"><p>next</p></para></item></itemize><p>H</p></para>"##,
  );
}

/// 61q (61p's arXiv A/B): a font switch in a trivlist's label ends with the label — latex.ltx sets it in a box
/// (latex.ltx:16028 `\sbox\@tempboxa{\makelabel{#1}}`) — while a global assignment in it persists. Digested ungrouped,
/// `\item[\scshape Proof.]` set every later word of the proof in small caps (2605.27137, 2605.03300, 18 of 3,003
/// papers). pdflatex: "Proof." in small caps, the body upright, then "After."; "x G7" (the `\gdef` and the counter
/// reach the body, the local `\def` and the colour do not).
#[test]
fn trivlist_label_font_ends_with_the_label() {
  let xml = assert_elements(
    r"\documentclass{article}
\newenvironment{pf}{\trivlist\item[\hskip\labelsep\scshape Proof.]}{\endtrivlist}
\begin{document}
\begin{pf}The body stays upright.\end{pf}
After.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag><text font="smallcaps">Proof.</text></tag></tags><para xml:id="S0.I1.ix1.p1"><p>The body stays upright.</p></para></item></itemize><p>After.</p></para>"##,
  );
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{xcolor}
\newcounter{thm}
\begin{document}
\begin{trivlist}\item[\gdef\foo{G}\def\baz{L}\setcounter{thm}{7}\color{red}x] \foo\thethm\ifdefined\baz\ local\fi\end{trivlist}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag><text color="#FF0000">x</text></tag></tags><para xml:id="S0.I1.ix1.p1"><p>G7</p></para></item></itemize></para>"##,
  );
}

/// 61r (sandbox 2606): a box or an alignment opened by any catcode-1 character closes on any catcode-2 character
/// (tex.web §403 `scan_left_brace`, §1068 `handle_right_brace`). Since 60j the openers took any catcode-1 character
/// but the closers knew only `}`, so `\hbox<…>` under `\catcode`\<=1 \catcode`\>=2` (2606.11726's plain-TeX macros)
/// erred at every `>` (101 errors, then Fatal); before 60j the box reader skipped to the next `{`, keeping 251 of the
/// paper's 115,678 words. pdflatex: "A in box B x C", the alignment, "after".
#[test]
fn box_closes_on_any_end_group_character() {
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
\catcode`\<=1 \catcode`\>=2
A \hbox<in box> B \vbox<\hbox<x>> C
\halign<#\hfil&\hfil#\cr a&b\cr> after
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>A in box B <inline-block vattach="bottom"><p>x</p></inline-block> C</p><tabular><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">a</td><td align="right" class="ltx_nopad_r">b</td></tr></tabular><p>after</p></para>"##,
  );
}

/// 61r (sandbox 2605/2606): `\setlength`/`\addtolength` read their register and value as latex.ltx:10253-10254 does,
/// from one stream — `#1`, a space, `#2`. `\setlength{\oddsidemargin 0.5cm}\setlength{\evensidemargin 0.5cm}` (six
/// arXiv papers: 2605.02145, 2605.06074, 2605.08367, 2605.24444, 2605.29037, 2606.03162) sets both margins, where
/// 56jr's argument tails handed `#1`'s rest back after `#2` and the next `\setlength` read "0" as its register ("A
/// <variable> was supposed to be here", a stray "5cm" typeset). The ordinary forms keep their reading: a tail after
/// the value is typeset after (`xG`), a skip keeps its `plus`, a braced `\relax` is skipped. pdflatex:
/// "odd=14.22636pt, even=14.22636pt"; "xG 3.0pt plus 1.0pt 5.0pt plus 1.0pt 5.0pt".
#[test]
fn setlength_reads_its_register_and_value_as_one_stream() {
  let xml = assert_elements(
    r"\documentclass{article}
\setlength{\oddsidemargin 0.5cm}
\setlength{\evensidemargin 0.5cm}
\begin{document}
odd=\the\oddsidemargin, even=\the\evensidemargin
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>odd=14.22636pt, even=14.22636pt</p></para>"##,
  );
  let xml = assert_elements(
    r"\documentclass{article}
\newlength\mylen\def\foo{x}
\begin{document}
\setlength{\parindent}{2pt\foo}G \setlength{\mylen}{3pt plus 1pt}\the\mylen\ \addtolength{\mylen}{2pt}\the\mylen\ \setlength{\relax\mylen}{5pt}\the\mylen
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>xG 3.0pt plus 1.0pt 5.0pt plus 1.0pt 5.0pt</p></para>"##,
  );
}

/// 61r (sandbox 2606.14467): a versioned package name falls back to its binding, which runs once. The fallback probe
/// ran the binding to see whether it existed and the caller ran it again: hyperref's `\let\H@refstepcounter
/// \refstepcounter` then saved its own wrapper, `\H@refstepcounter` expanded to itself, and since 58h's counter steps
/// go through `\refstepcounter`, the first `\section` looped (Fatal, no output). pdflatex: "1 Introduction", "See
/// Section 1.".
#[test]
fn versioned_package_fallback_runs_its_binding_once() {
  let xml = assert_elements(
    r"\begin{filecontents*}[overwrite]{hyperref.2.0.sty}
\ProvidesPackage{hyperref.2.0}
\RequirePackage{hyperref}
\end{filecontents*}
\documentclass{article}
\usepackage{hyperref.2.0}
\begin{document}
\section{Introduction}\label{s:intro}
See Section~\ref{s:intro}.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" labels="LABEL:s:intro" xml:id="S1"><tags><tag>1</tag><tag role="autoref">section 1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Introduction</title><para xml:id="S1.p1"><p>See Section <ref labelref="LABEL:s:intro"/>.</p></para></section>"##,
  );
}

/// 61r: the versioned-name fallback asks the binding registry without running the binding, and matches names
/// case-insensitively, as the dispatchers do: `jhep_2024` falls back to `jhep.cls`, which loads the `JHEP.cls`
/// binding (its `\JHEP` issue note).
#[test]
fn versioned_class_fallback_matches_the_binding_case_insensitively() {
  let xml = assert_elements(
    r"\documentclass{jhep_2024}
\title{A title}
\JHEP{12}
\begin{document}
\maketitle
Text.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="jhep-issue""#],
    r#"<note role="jhep-issue">12</note>"#,
  );
}

/// 61r (sandbox 2606.26406; #556 2508.07407): pgf's `@` arithmetic runs on the sp grid, as TeX's register arithmetic
/// does, so the cloud shape's border-anchor binary search ends (KNOWN_PERL_ERRORS #461: in floating point rounded to
/// five decimals its midpoint could round up to the interval's end forever, Fatal). pdflatex: the cloud and the line.
#[test]
fn cloud_anchor_search_converges() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{shapes.symbols}
\begin{document}
\begin{tikzpicture}
\node[shape=cloud, draw, minimum width=3.4cm, align=center] (c) at (0,0) {A \\ B};
\draw (c.south east) -- (2,-2);
\end{tikzpicture}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "picture",
    &[r#"xml:id="p1.pic1""#],
    r##"<picture height="102.93" width="147.95" xml:id="p1.pic1"><svg:svg height="102.93" overflow="visible" version="1.1" viewBox="0 0 147.95 102.93" width="147.95"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,102.93) matrix(1 0 0 -1 0 0) translate(68.94,0) translate(0,79.02)"><svg:path d="M 21.15 7.41 C 18.75 16.37 9.28 23.64 0 23.64 C -9.28 23.64 -18.75 16.37 -21.15 7.41 C -23.69 14.5 -31.83 19.74 -39.34 19.12 C -46.85 18.5 -54.02 11.99 -55.36 4.58 C -57.07 7.09 -60.79 8.31 -63.65 7.3 C -66.52 6.3 -68.66 3.03 -68.43 0 C -68.66 -3.03 -66.52 -6.3 -63.65 -7.3 C -60.79 -8.31 -57.07 -7.09 -55.36 -4.58 C -54.02 -11.99 -46.85 -18.5 -39.34 -19.12 C -31.83 -19.74 -23.69 -14.5 -21.15 -7.41 C -18.75 -16.37 -9.28 -23.64 0 -23.64 C 9.28 -23.64 18.75 -16.37 21.15 -7.41 C 23.69 -14.5 31.83 -19.74 39.34 -19.12 C 46.85 -18.5 54.02 -11.99 55.36 -4.58 C 57.07 -7.09 60.79 -8.31 63.65 -7.3 C 66.52 -6.3 68.66 -3.03 68.43 0 C 68.66 3.03 66.52 6.3 63.65 7.3 C 60.79 8.31 57.07 7.09 55.36 4.58 C 54.02 11.99 46.85 18.5 39.34 19.12 C 31.83 19.74 23.69 14.5 21.15 7.41 Z" style="fill:none"/><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 -5.19 -9.46)"><svg:g class="ltx_tikzmatrix" transform="matrix(1 0 0 -1 0 18.91)"><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 9.46)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_r" transform="matrix(1 0 0 -1 0 0)"><svg:foreignObject height="9.46" overflow="visible" style="--ltx-fo-width:0.75em;--ltx-fo-height:0.68em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="10.38">A</svg:foreignObject></svg:g></svg:g><svg:g class="ltx_tikzmatrix_row" transform="matrix(1 0 0 1 0 18.92)"><svg:g class="ltx_tikzmatrix_col ltx_nopad_r" transform="matrix(1 0 0 -1 0.29 0)"><svg:foreignObject height="9.46" overflow="visible" style="--ltx-fo-width:0.71em;--ltx-fo-height:0.68em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="9.8">B</svg:foreignObject></svg:g></svg:g></svg:g></svg:g><svg:path d="M 47.4 -16.74 L 78.74 -78.74" style="fill:none"/></svg:g></svg:svg></picture>"##,
  );
}

/// 61r (sandbox): calc reaches only the arguments TeX hands it. Two bindings scanned what their package never scans:
/// subcaption's `{subfigure}` takes minipage's `[pos][height][inner]{width}` (subcaption.sty:70-108; bound
/// `[]{Dimension}` it read `[` as the width, calc erred and "0pt][c]0.47" was typeset: 2605.06598, 2605.21425,
/// 2606.16001), and soul's `\setul` only stores its arguments (soul-ori.sty:833-836; "red" was scanned as a length:
/// 2605.19108). pdflatex: each clean.
#[test]
fn calc_reaches_only_the_arguments_tex_hands_it() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{graphicx,subcaption,calc}
\begin{document}
\begin{figure}
\begin{subfigure}[c][0pt][c]{0.47\textwidth}
x
\end{subfigure}
\end{figure}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="fig2""#],
    r##"<figure placement="c" xml:id="fig2"><p>x</p></figure>"##,
  );
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{soul,calc}
\begin{document}
\setul{red}{2pt}
Text.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  );
}

/// 61r (sandbox 2606.21036): amsmath's `\\[<len>]` is `\noalign{\vskip#1\relax}` (amsmath.sty:1181-1182), TeX's own
/// scan, not calc's: under calc `\\[-\belowdisplayskip\vspace{-1em}]` raised calc's "`\vskip' invalid at this point".
/// What follows the length is dropped with its warning (TeX typesets it between the rows). pdflatex: clean.
#[test]
fn align_newline_length_is_a_plain_scan() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{amsmath,calc}
\begin{document}
\begin{align}
a &= b \\[-\belowdisplayskip\vspace{-1em}] c &= d
\end{align}
\end{document}",
    RAW,
    (0, 1),
    &[],
  );
  assert_element(
    &xml,
    "equationgroup",
    &[r#"xml:id="S0.EGx1""#],
    r##"<equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle a=b" text="a = b" xml:id="S0.E1.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=b" text="absent = b" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle c=d" text="c = d" xml:id="S0.E2.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle c" text="c" xml:id="S0.E2.m1"><XMath><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=d" text="absent = d" xml:id="S0.E2.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>"##,
  );
}

/// 61r review: under calc, the widths latex.ltx and its packages hand `\setlength` stay calc expressions — a `p{}`
/// column (latex.ltx:16755, array.sty:191 `\setlength\hsize{#1}`), `tabular*` (latex.ltx:16558), tabularx
/// (tabularx.sty:56). A narrowing of calc to `Setlength*` operands typed "-1cm" and "-2cm" into the tables. pdflatex:
/// clean, the `p` column 0.3\textwidth-2\tabcolsep = 91.5pt.
#[test]
fn calc_widths_in_tables_are_evaluated() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{calc}
\usepackage{tabularx}
\begin{document}
\begin{tabular}{p{0.3\textwidth-2\tabcolsep}l}
alpha & beta\\
\end{tabular}

\begin{tabular*}{\textwidth-1cm}{ll}
gamma & delta\\
\end{tabular*}

\begin{tabularx}{\textwidth-2cm}{lX}
eps & zeta\\
\end{tabularx}

\parbox{\textwidth-2cm}{eta}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  for (id, whole) in [
    (
      "p1",
      r##"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left" vattach="top"><inline-block vattach="top" width="91.5pt"><p>alpha</p></inline-block></td><td align="left">beta</td></tr></tbody></tabular></para>"##,
    ),
    (
      "p2",
      r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">gamma</td><td align="left">delta</td></tr></tbody></tabular></para>"##,
    ),
    (
      "p3",
      r##"<para xml:id="p3"><tabular vattach="middle"><tbody><tr><td align="left">eps</td><td align="left"><inline-block vattach="top"><p>zeta</p></inline-block></td></tr></tbody></tabular></para>"##,
    ),
    (
      "p4",
      r##"<para xml:id="p4"><p><inline-block class="ltx_parbox" vattach="middle" width="288.1pt"><p>eta</p></inline-block></p></para>"##,
    ),
  ] {
    assert_element(&xml, "para", &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 61s (sandbox 2606.05500; KNOWN_PERL_ERRORS #462): an array opened without an environment group gives `$` back when
/// it ends. `\array`'s bindings re-let `$` for its math cells before the array's group opens, so after a bare
/// `\array…\endarray` the closing `$` opened a text box; makecell's math branch (makecell.sty:131-133) is that shape,
/// and every `\makecell` in a `>{$}l<{$}` cell cascaded. pdflatex: x, the a/b stack, y; the a/b cell.
#[test]
fn bare_array_gives_dollar_back() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{makecell}
\begin{document}
$x\hbox{$\array{l}a\\b\endarray$}y$

\begin{tabular}{>{$}l<{$}}
\makecell[l]{a\\b}
\end{tabular}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="x\hbox{$\begin{array}[]{l}a\\&#10;b\end{array}$}y" text="x * Array[[a], [b]] * y" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell></XMRow><XMRow><XMCell align="left"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow></XMArray><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"#,
  );
  // The cell's Math; the empty marked-as-math texts beside it (makecell's `\null`s in the `>{$}…<{$}` cell) are not
  // what this guards.
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p2.m1.m1""#],
    r#"<Math mode="inline" tex="\begin{array}[c]{@{}l@{}}a\\&#10;b\end{array}" text="Array[[a], [b]]" xml:id="p2.m1.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell></XMRow><XMRow><XMCell align="left"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow></XMArray></XMath></Math>"#,
  );
}

/// 61s (sandbox 2606.15832; OXIDIZED_DESIGN_DIVERGENCES #437): a paragraph column's width is read in its cells, as TeX
/// reads `\@startpbox{#1}`'s `\setlength\hsize{#1}` (array.sty:189-191). A `p|` column that no row reaches does not
/// scan its `|`, and calc widths are evaluated where the cell is set. pdflatex: "a b"; x, y, z in 42.7pt, 86.3pt and
/// 85.4pt boxes.
#[test]
fn paragraph_column_width_is_read_in_its_cells() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{array,calc}
\begin{document}
\begin{tabular}{|c|c|p|}
a & b\\
\end{tabular}

\begin{tabular}{p{2cm-5mm}|m{\linewidth/4}|b{3cm}}
x & y & z\\
\end{tabular}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  for (id, whole) in [
    (
      "p1",
      r#"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="center" border="l r">a</td><td align="center" border="r">b</td></tr></tbody></tabular></para>"#,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left" border="r" vattach="top"><inline-block vattach="top" width="42.7pt"><p>x</p></inline-block></td><td align="left" border="r" vattach="middle"><inline-block vattach="middle" width="86.3pt"><p>y</p></inline-block></td><td align="left" vattach="bottom"><inline-block vattach="bottom" width="85.4pt"><p>z</p></inline-block></td></tr></tbody></tabular></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 61s (sandbox 2606.05563; KNOWN_PERL_ERRORS #463): inside tabularx, `X` is tabularx's own column (tabularx.sty:90, :157-158 `\TX@newcol`),
/// though the document defines an `X` with an argument for its other tables. The document's `X[1]` read `|` as the
/// width of every tabularx X cell. pdflatex: "a b", then "c" centred in 2cm.
#[test]
fn tabularx_x_is_its_own_inside_tabularx() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{array,tabularx}
\newcolumntype{X}[1]{>{\centering\arraybackslash}p{#1}}
\begin{document}
\begin{tabularx}{\textwidth}{|l|X|}
a & b\\
\end{tabularx}

\begin{tabular}{X{2cm}}
c\\
\end{tabular}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  for (id, whole) in [
    (
      "p1",
      r#"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left" border="l r">a</td><td align="left" border="r"><inline-block vattach="top"><p>b</p></inline-block></td></tr></tbody></tabular></para>"#,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left" vattach="top"><inline-block vattach="top" width="56.9pt"><p align="center">c</p></inline-block></td></tr></tbody></tabular></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 61s (KNOWN_PERL_ERRORS #463): xltabular's `X` is tabularx's own column too (xltabular.sty:28 `\TX@newcol`); the
/// document's `X[1]` took `|` as its width and the cell typeset the `|` left over by the lazy width scan. pdflatex:
/// "d e".
#[test]
fn xltabular_x_is_tabularx_own() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{array,tabularx,xltabular}
\newcolumntype{X}[1]{>{\centering\arraybackslash}p{#1}}
\begin{document}
\begin{xltabular}{\textwidth}{|l|X|}
d & e\\
\end{xltabular}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular><tr><td align="left" border="l r">d</td><td align="left" border="r"><inline-block vattach="top"><p>e</p></inline-block></td></tr></tabular>"#,
  );
}

/// 61t (sandbox 2606.15113; KNOWN_PERL_ERRORS #464): `\pgfmath@smuggleone` smuggles its whole argument, as
/// pgfmathutil.code.tex:295-296 does. `\pgfmathtruncatemacro{\y0}` defines `\y` delimited by `0\expandafter` and never
/// expands it; the binding took the argument's first token and expanded `\y` without its `0` ("Missing argument", once
/// per loop pass). pdflatex: the nodes 0 and 1, then 3.
#[test]
fn smuggleone_carries_its_whole_argument() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\foreach \x in {0,1} {
  \pgfmathtruncatemacro{\y0}{2*\x + 1}
  \pgfmathtruncatemacro{\cc}{mod(\x,2)}
  \node at (\x,\cc) {\cc};
}
\end{tikzpicture}
\pgfmathtruncatemacro{\z}{7/2}\z
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><picture height="57.51" width="55.51" xml:id="p1.pic1"><svg:svg height="57.51" overflow="visible" version="1.1" viewBox="0 0 55.51 57.51" width="55.51"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,57.51) matrix(1 0 0 -1 0 0) translate(8.07,0) translate(0,9.07)"><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 -3.46 -4.46)"><svg:foreignObject height="8.92" overflow="visible" style="--ltx-fo-width:0.5em;--ltx-fo-height:0.64em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 8.92)" width="6.92">0</svg:foreignObject></svg:g><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 35.91 34.91)"><svg:foreignObject height="8.92" overflow="visible" style="--ltx-fo-width:0.5em;--ltx-fo-height:0.64em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 8.92)" width="6.92">1</svg:foreignObject></svg:g></svg:g></svg:svg></picture><p>3</p></para>"##,
  );
}

/// 61u (sweep #148): a document declaring pTeX gets the format's registers, as plcore.ltx:127-137 allocates them
/// (`\Cht`, `\Cdp`, `\Cwd`, `\Cvs`, `\Chs`, lowercase twins, `\cHT`), by `\NeedsTeXFormat{pLaTeX2e}` (jsarticle.cls:14)
/// or `\epTeXinputencoding` (jlreq.cls:493). jsarticle.cls:771-775 and jlreq.cls:1306-1310 set them unallocated; since
/// 61r's `\setlength` typesets a non-register target's value as TeX does, `\setlength\Cvs{\baselineskip}` assigned
/// `\baselineskip` a missing number, and `\divide\textheight\baselineskip` (jsarticle.cls:876) divided by zero (30
/// pLaTeX manuals, +1 error each).
#[test]
fn platex_format_registers_are_allocated() {
  for declaration in [
    r"\NeedsTeXFormat{pLaTeX2e}\documentclass{article}",
    r"\documentclass{article}\epTeXinputencoding utf8",
  ] {
    let xml = assert_elements(
      &format!(
        r"{declaration}
\setlength\Cvs{{\baselineskip}}
\setlength\Chs{{2pt}}
\begin{{document}}
\the\Cvs, \the\Chs, \the\baselineskip.
\end{{document}}"
      ),
      RAW,
      (0, 0),
      &[],
    );
    assert_element(
      &xml,
      "para",
      &[r#"xml:id="p1""#],
      r#"<para xml:id="p1"><p>12.0pt, 2.0pt, 12.0pt.</p></para>"#,
    );
  }
}

/// 61v (user ruling 2026-10-04; repro boxes-groups/raisebox_raise_measures_the_box; KNOWN_PERL_ERRORS #466): `\raisebox`
/// reads its raise with the box set, as latex.ltx's `\@irsbox` does (`\setlength\@tempdima{#1}` after `\@begin@tempboxa
/// \hbox{#3}`, :16378-16393), so `\height`, `\depth` and `\width` measure the box: `\raisebox{-.5\height}{x}` lowers x by
/// half its height. The raise was read first, with `\height` the text `0pt` (and, under calc, an error: 2606.06643 ×54).
/// pdflatex: [5.2778pt][2.15277pt][2.15277pt], [10.00002pt][5.00002pt][5.0pt], [5.00002pt][6.24998pt][0.0pt],
/// [10.5556pt][2.15277pt][2.15277pt].
#[test]
fn raisebox_raise_measures_the_box() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/raisebox_raise_measures_the_box.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  for (id, whole) in [
    (
      "p1",
      r#"<para xml:id="p1"><p>[5.2778pt][2.15277pt][2.15277pt]</p></para>"#,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><p>[10.00002pt][5.00002pt][5.0pt]</p></para>"#,
    ),
    (
      "p3",
      r#"<para xml:id="p3"><p>[5.00002pt][6.24998pt][0.0pt]</p></para>"#,
    ),
    (
      "p4",
      r#"<para xml:id="p4"><p>[10.5556pt][2.15277pt][2.15277pt]</p></para>"#,
    ),
    (
      "p5",
      r#"<para xml:id="p5"><p>A<text yoffset="-2.2pt">x</text>B</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 61v (cortex rerun on 61u, 2605.18869): a column type expanded outside a preamble builds nothing. Since 61s
/// `\NC@rewrite@X` is tabularx's own column inside a tabularx; an `\edef` meeting it there unwrapped the missing
/// template and panicked (Fatal). In TeX it only yields tokens (`\NC@find p{…}`); pdflatex errs on this abuse
/// ("Undefined control sequence"), the guard pins only that nothing panics and the table survives; the missing error is a
/// known gap (the binding's `X` yields no `\NC@find` tokens to fail on), not the intended diagnostic count.
#[test]
fn column_type_outside_a_preamble_builds_nothing() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{tabularx}
\makeatletter
\begin{document}
\begin{tabularx}{\textwidth}{lX}
\setbox0\hbox{\edef\y{\NC@rewrite@X}}b & c
\end{tabularx}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left">b</td><td align="left"><inline-block vattach="top"><p>c</p></inline-block></td></tr></tbody></tabular></para>"#,
  );
}

/// 62a (stopped full-arXiv run 329; repro loader/latexml_preload_keeps_plain.tex): a plain TeX document stays plain
/// under the latexml.sty preload. The preload's `\AddToHook` autoloaded the LaTeX format for every document; a plain
/// one then had LaTeX's `\end`, and its closing `\end` raised "`\endgroup` Attempt to close a group that switched to
/// mode vertical" (27 of 16,333 papers: 0911.4241, 1001.3079, hep-th9310069). Plain `\eject`'s page count no longer
/// names `\c@page` either (`\count0`). pdftex: PLAIN; "Hello world.".
#[test]
fn plain_document_stays_plain() {
  let xml = assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/loader/latexml_preload_keeps_plain.tex"),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "para", &[], "<para><p>PLAIN</p></para>");
  let xml = assert_elements("Hello world.\n\\eject\\end\n", RAW, (0, 0), &[]);
  assert_element(&xml, "para", &[], "<para><p>Hello world.</p></para>");
}

/// 62a (stopped full-arXiv run 329): a Semiverbatim argument's definitions keep their names. Its pre-expansion expanded
/// the name of a font switch's `\edef\f@series{…}` to the letter `m`, and neutralized a `\def~`'s active `~`
/// ("Missing control sequence inserted"; 1212.6174, 1303.4395, 1711.09355, 1011.4121). The JHEP link text now keeps
/// its fonts; a `\newcommand`/`\renewcommand` in it keeps its name and body. pdflatex: the URL with a tiny ∼; "See JHEP 1005 and [1707.09588]".
#[test]
fn semiverbatim_definitions_keep_their_names() {
  let xml = assert_elements(
    r"\documentclass{amsart}
\begin{document}
\title{T}\author{A}
\urladdr{\def~{{\tiny$\sim$}}http://www.math.mcgill.ca/~louigi/}
\maketitle
Theory.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "contact",
    &[r#"role="url""#],
    r#"<contact name="URL: " role="url">http://www.math.mcgill.ca/˜louigi/</contact>"#,
  );
  let xml = assert_elements(
    r"\documentclass{JHEP3}
\title{T}\author{A}\abstract{B}
\begin{document}
See \href{http://dx.doi.org/10.1007/X}{{\em JHEP} {\bf 1005}} and
[\href{https://arxiv.org/abs/1707.09588}{{\ttfamily 1707.09588}}].
Defined \href{http://x.org/d}{\newcommand*{\yy}[1][d]{#1}\renewcommand\yy{Z}}.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "ref",
    &[r#"href="http://dx.doi.org/10.1007/X""#],
    r#"<ref href="http://dx.doi.org/10.1007/X"><text font="italic">JHEP</text> <text font="bold">1005</text></ref>"#,
  );
  // LaTeX's definers keep their star, name, options and body (`\yy` was expanded, "undefined").
  assert_element(
    &xml,
    "ref",
    &[r#"href="http://x.org/d""#],
    r#"<ref href="http://x.org/d"/>"#,
  );
}

/// 62a (stopped full-arXiv run 329; the K6 ruling's DVI cue, stream F): a class option naming a DVI driver makes the
/// document a latex+dvips one. 0908.4150's `\documentclass[12pt,dvips]{article}` ships no figures, so it was compiled
/// as pdflatex would and l3backend stopped on "Backend request inconsistent with engine". latex: DVI.
#[test]
fn dvips_class_option_is_dvi() {
  let xml = assert_elements(
    r"\documentclass[12pt,dvips]{article}
\usepackage{graphicx}
\usepackage{ifpdf}
\begin{document}
\ifpdf PDF\else DVI\fi
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>DVI</p></para>"#,
  );
}

/// 62a (stopped full-arXiv run 329): an AMSTeX document keeps amsmath's `\cases … \endcases`. With the document now
/// plain, the amsfonts binding's `\DeclareSymbolFont` and the graphics binding's `\providecommand` (`\input psfig.sty`)
/// autoloaded LaTeX mid-load, and the latex dump replaced amsmath's `\cases` with plain's `\cases{…}`: "Stray alignment
/// "&"" and "`\lx@end@gen@cases` Attempt to close a group that switched to mode display_math" (1409.5819, 1→89 errors).
/// Perl never loads LaTeX for either. As in Perl, the `\noalign` row's empty cell reads "otherwise". tex: two cases.
#[test]
fn amstex_cases_stay_amsmath() {
  let xml = assert_elements(
    r"\input amstex
\input psfig.sty
$$\cases a=1,&\quad x>0,\\
\noalign{\medskip}
a=0,&\quad x=0.
\endcases$$
\bye
",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "XMApp",
    &[],
    r#"<XMApp><XMTok meaning="cases"/><XMRef idref="id1"/><XMRef idref="id2"/><XMText><text font="italic">otherwise</text></XMText><XMRef idref="id3"/><XMRef idref="id4"/></XMApp>"#,
  );
}

/// 62a (stopped full-arXiv run 329): a class shipped with the source and run as OmniBus has its `\LoadClass` and
/// `\RequirePackage` lines scanned (Perl `maybeRequireDependencies`); an option naming one of the class's own macros
/// stays an inert string, as in Perl. easychair.cls:361-366's `\LoadClass[\@PaperFormat,…]{report}` (an `\ifthesis`
/// branch the scan does not see) raised "undefined" for each macro (2011.11995, 2211.09353, 2607.12736). pdflatex:
/// "Hello.". Under the production preload (`[rawclasses]` would run the class raw); the OmniBus fallback's
/// missing-binding warning is the one warning.
#[test]
fn scanned_class_options_naming_macros_stay_inert() {
  let (log, xml) = latexml::util::test::convert_files_with(
    "\\documentclass{shipped}\n\\begin{document}\nHello.\n\\end{document}\n",
    &[(
      "shipped.cls",
      r"\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{shipped}
\def\@PaperFormat{letterpaper}
\newif\ifthesis
\ifthesis
  \LoadClass[\@PaperFormat,twoside]{report}
\else
  \LoadClass[\@PaperFormat,twoside]{article}
\fi
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Hello.</p></para>"#,
  );
}

/// 62a (stopped full-arXiv run 329): `\textcircled`'s argument is typeset in a box (omsenc.def:62-64 `\ooalign`'s
/// `\hbox`), restricted horizontal even in math, so in `$\textcircled{$C$}_1$` the inner `$` opens a formula. Digested
/// in the surrounding math mode, it closed the outer one and the `_1` raised "Script _ can only appear in math mode"
/// (1009.5713, 53 errors). latex: two circled C₁, C₂; Perl is clean (it circles the raw tokens). The circled text
/// "$C$" (Perl's too) is a pinned residual, not the target.
#[test]
fn textcircled_argument_is_text_in_math() {
  let xml = assert_elements(
    r"\documentclass{amsart}
\begin{document}
glue $\textcircled{$C$}_1$ and $\textcircled{$C$}_2$ along.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\textcircled{$C$}_{1}" text="circled-$C$ _ 1" xml:id="p1.m1"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok meaning="circled-$C$" role="UNKNOWN">$C$⃝</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
  );
}

/// 62a (review round 3): `\clearpage` advances the page number's register, which in a LaTeX document is whatever
/// `\c@page` names (`\count0` from the dump, latex.ltx:14891; a counter of its own on the NODUMP branch), not `\count0`
/// as such: 62a's `\count0` left the NODUMP `\thepage` at 1 and made the pad-to-page loop of OD #178 Fatal. A plain
/// document advances `\count0` (`\pageno`). pdflatex: "A", then "B 2" on page 2.
#[test]
fn clearpage_advances_the_register_c_at_page_names() {
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
\makeatletter\newcount\mypage \let\c@page\mypage \c@page=1 \makeatother
A\clearpage B \thepage
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "p", &[], "<p>AB 2</p>");
}

/// 62b (stopped full-arXiv run 329): verbatim.sty loaded after comment.sty redefines `{comment}` (verbatim.sty:90-97
/// `\def\comment`), so an indented `\end{comment}` ends it; the comment binding's `\begin{comment}` control sequence,
/// which `\begin` prefers, kept comment.sty's whole-line rule, and the comment ran to the end of the file (2607.07115,
/// 2607.23269). pdflatex: "Before. After." (under the ar5iv preload Rust and Perl drop the space).
#[test]
fn verbatim_comment_after_comment_sty() {
  let xml = assert_elements(
    "\\documentclass{article}\n\\usepackage{comment}\n\\usepackage{verbatim}\n\\begin{document}\nBefore.\n   \
     \\begin{comment}\n   hidden text\n   \\end{comment}\nAfter.\n\\end{document}\n",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "p", &[], "<p>Before. After.</p>");
}

/// 62b (stopped full-arXiv run 329): acro's `patch/longtable` `\patchcmd`s longtable.sty's `\endlongtable`, which the
/// longtable binding's is not, so acro errs "Patching `longtable' failed" (2310.14606, 2606.11983); the binding turns
/// the key off, whichever package loads first (the patch only silences acronyms in repeated heads). pdflatex: clean.
#[test]
fn acro_skips_its_longtable_patch() {
  for packages in [
    "\\usepackage{longtable}\\usepackage{acro}",
    "\\usepackage{acro}\\usepackage{longtable}",
  ] {
    let xml = assert_elements(
      &format!(
        "\\documentclass{{article}}{packages}\n\\DeclareAcronym{{ai}}{{short=AI,long=artificial intelligence}}\n\
         \\begin{{document}}\nUse \\ac{{ai}}.\n\\begin{{longtable}}{{ll}}a&b\\\\\\end{{longtable}}\n\\end{{document}}\n"
      ),
      RAW,
      (0, 0),
      &[],
    );
    assert_element(
      &xml,
      "tr",
      &[],
      r#"<tr><td align="left">a</td><td align="left">b</td></tr>"#,
    );
  }
}

/// 62b (stopped full-arXiv run 329; OXIDIZED_DESIGN_DIVERGENCES #440): in math, `\emph` typesets its argument as text
/// (latex.ltx `\DeclareTextFontCommand` → `\nfss@text`), as `\textit` does: `$\emph{x}+1$` is the emphasized text "x"
/// plus 1 (Perl: an empty `<XMText/>` times a token x), and in `$\emph{P$\bar{3}$m1}$` the inner `$` opens a formula
/// (2502.18190: an error under run 306, `TooManyErrors` since). pdflatex: clean.
#[test]
fn emph_in_math_is_text() {
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
A $\emph{x}+1$ and $\emph{P$\bar{3}$m1}$ B.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\emph{x}+1" text="[x] + 1" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMText><emph font="italic">x</emph></XMText><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "emph",
    &[r#"class="ltx_markedasmath""#],
    r#"<emph class="ltx_markedasmath" font="italic">P<Math mode="inline" tex="\bar{3}" text="bar@(3)" xml:id="p1.m2.m1"><XMath><XMApp><XMTok font="upright" name="bar" role="OVERACCENT" stretchy="false">¯</XMTok><XMTok font="upright" meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math>m1</emph>"#,
  );
  // In an italic theorem the emphasis is upright in math as in text (leaving math restores the text font, the
  // emphasis is toggled again), and a nested `\emph` toggles back.
  let xml = assert_elements(
    r"\documentclass{article}
\newtheorem{thm}{Theorem}
\begin{document}
\begin{thm}Let $\emph{G}$ and \emph{H} and $\emph{a \emph{b} c}$.\end{thm}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p><text font="italic">Let <emph class="ltx_markedasmath" font="upright">G</emph> and <emph font="upright">H</emph> and <emph class="ltx_markedasmath" font="upright">a <emph font="italic">b</emph> c</emph>.</text></p>"#,
  );
}

/// 62b (stopped full-arXiv run 329): caption3.sty replaces subfig v1.3's `\sf@subfloat` at begin document with one
/// built on raw subfig internals (caption3.sty:1376-1387; 2003.01262 ships v1.8h, whose replacement is reproduced
/// here), which the native subfig binding lacks ("undefined \sf@ifpositiontop", TooManyErrors); its own
/// `\sf@subfloat` comes back at `begindocument/end`. pdflatex: the sub-figure "a: One" in "Figure 1: Both.".
#[test]
fn native_subfloat_survives_captions_subfig_patch() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{subfig}
\makeatletter
\AtBeginDocument{\let\sf@subfloat\my@NEW@subfloat}
\def\my@NEW@subfloat{\begingroup\sf@ifpositiontop{\maincaptiontoptrue}{\maincaptiontopfalse}%
  \let\sf@oldlabel=\label\let\label=\subfloat@label\ifmaincaptiontop\else\advance\@nameuse{c@\@captype}\@ne\fi
  \refstepcounter{sub\@captype}\setcounter{sub\@captype @save}{\value{sub\@captype}}%
  \@ifnextchar[{\sf@@subfloat}{\sf@@subfloat[\@empty]}}
\makeatother
\begin{document}
\begin{figure}
\subfloat[One]{X}
\caption{Both.}
\end{figure}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1.sf1""#],
    r#"<figure xml:id="S0.F1.sf1"><tags><tag>(a)</tag><tag role="refnum">1a</tag></tags><p>X</p><toccaption><tag close=" ">a</tag>One</toccaption><caption><tag close=" ">(a)</tag>One</caption></figure>"#,
  );
}

/// 62b (stopped full-arXiv run 329): a column type's `\let\newline\\` (1901.05279's `>{\centering\let\newline\\…}m{…}`)
/// made the in-cell `\\` of a `\multicolumn` body, which returned `\newline` by name, expand into itself
/// (`Fatal:Timeout:Recursion`); it is the kernel's `\lx@newline` now. pdflatex: "a" and "b" on two lines.
#[test]
fn in_cell_newline_survives_a_let_newline() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{array}
\newcolumntype{C}[1]{>{\centering\let\newline\\\arraybackslash\hspace{0pt}}m{#1}}
\begin{document}
\begin{tabular}{C{3em}C{3em}}
\multicolumn{2}{C{6em}}{a \newline b}\\
\end{tabular}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "td",
    &[],
    r#"<td align="left" vattach="middle"><inline-block vattach="middle" width="60.0pt"><p align="center">a</p><p align="center">b</p></inline-block></td>"#,
  );
}

/// 62b (stopped full-arXiv run 329): an eqnarray row `& & + …` after a `\lefteqn{…}` row (one cell) asked the math
/// parser for a column pair past the row's cells, a slice panic (2211.01040, `Fatal:panic`). pdflatex: clean.
#[test]
fn eqnarray_continuation_after_lefteqn_does_not_panic() {
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
\begin{eqnarray}
\lefteqn{D = a} \nonumber\\
& & + b\, dx
\end{eqnarray}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "equationgroup",
    &[],
    r#"<equationgroup class="ltx_eqn_eqnarray" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle D=a+b\,dx" text="D = a + b * d * x" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">D</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMApp></XMath></Math><MathBranch><tr><td align="left" colspan="3"><Math mode="inline" tex="\displaystyle D=a" text="D = a" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">D</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok></XMApp></XMath></Math></td></tr><tr><td/><td/><td align="left"><Math mode="inline" tex="\displaystyle+b\,dx" text="+ b * d * x" xml:id="S0.E1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMath></Math></td></tr></MathBranch></MathFork></equation></equationgroup>"#,
  );
}

/// 62b (stopped full-arXiv run 329): `\cline`'s rule constructor is private (`\lx@cline`), so a raw `\@cline` with
/// latex.ltx's `#1-#2\@nil` signature (an author's, array.sty's, colortbl.sty's) no longer reads past the end of the
/// file for `\@nil` (1902.04834, `Fatal:Mouth:EoF`). pdflatex: the table with a partial rule.
#[test]
fn cline_survives_a_raw_at_cline() {
  let xml = assert_elements(
    r"\documentclass{article}
\makeatletter
\def\@cline#1-#2\@nil{\omit\@multicnt#1\advance\@multispan\m@ne
  \ifnum\@multicnt=\@ne\@firstofone{&\omit}\fi\@multicnt#2\advance\@multicnt-#1%
  \advance\@multispan\@ne\leaders\hrule\@height\arrayrulewidth\hfill\cr
  \noalign{\nobreak\vskip-\arrayrulewidth}}
\makeatother
\begin{document}
\begin{tabular}{|c|c|c|}
a & b & c\\ \cline{2-3}
d & e & f
\end{tabular}

Pages 1-2.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "tbody",
    &[],
    r#"<tbody><tr><td align="center" border="l r" thead="row">a</td><td align="center" border="r">b</td><td align="center" border="r">c</td></tr><tr><td align="center" border="l r" thead="row">d</td><td align="center" border="r t">e</td><td align="center" border="r t">f</td></tr></tbody>"#,
  );
}

/// 62b (stopped full-arXiv run 329): `\caption` hands `\@caption` its `[short]` through `\@dblarg`, as latex.ltx does,
/// so a package's `\let\@caption` to a `#1[#2]#3` macro (3parttable's `\TPT@caption`, cond-mat0307356) finds its `[`
/// instead of scanning to the end of the file (`Fatal:Mouth:EoF`). pdflatex: "Caption text." in the table.
#[test]
fn caption_passes_its_short_caption() {
  // The kernel's `\caption` and the caption package's (caption_sty.rs) alike.
  for package in ["", "\\usepackage{caption}"] {
    let xml = assert_elements(
      &format!(
        "\\documentclass{{article}}{package}\n\\makeatletter\n\\long\\def\\my@caption#1[#2]#3{{Caption #3}}\n\
         \\makeatother\n\\begin{{document}}\n\\begin{{table}}\n\\makeatletter\\let\\@caption\\my@caption\\makeatother\n\
         \\caption{{text.}}\n\\end{{table}}\n\\end{{document}}"
      ),
      RAW,
      (0, 0),
      &[],
    );
    assert_element(
      &xml,
      "table",
      &[],
      r#"<table xml:id="tab1"><p>Caption text.</p></table>"#,
    );
  }
}

/// 62b (stopped full-arXiv run 329): book.cls is two-sided by default (book.cls:86-88, :119), so a document's
/// `\if@twoside` takes its first branch (hep-ph0207204's other branch was unbalanced: `Fatal:Document:Malformed`).
/// pdflatex: "two".
#[test]
fn book_is_two_sided() {
  let xml = assert_elements(
    r"\documentclass{book}
\makeatletter
\if@twoside
\def\x{two}
\else
\def\x{one{}
\fi
\makeatother
\begin{document}
\x
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "para", &[], r#"<para xml:id="p1"><p>two</p></para>"#);
}

/// 62b (stopped full-arXiv run 329): `\dimendef` and the other shorthands are local unless `\global` (tex.web §1224), so
/// pgfplots' grouped `\dimendef\rb=5` (pgfutil-common.tex:611-620) leaves a document's `\rb` macro alone (2003.08372:
/// 662 "Missing $", TooManyErrors). pdflatex: U = [x]^+, "Text after.".
#[test]
fn grouped_dimendef_is_local() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{amsmath}
\newcommand{\lb}{\ensuremath{\left[}}
\newcommand{\rb}{\ensuremath{\right]}}
\begin{document}
\begingroup \dimendef\rb=5 \endgroup
\begin{align}
U &= \lb x \rb^+
\end{align}
Text after.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math tex="\displaystyle U=\left[x\right]^{+}" text="U = (delimited-[]@(x)) ^ +" xml:id="S0.E1.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">U</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="delimited-[]"/><XMRef idref="S0.E1.m3.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">[</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m3.1">x</XMTok><XMTok role="CLOSE" stretchy="true">]</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="plus" role="ADDOP">+</XMTok></XMApp></XMApp></XMath></Math>"#,
  );
}

/// 62b review: supertabular hands `\@caption` its short caption braced (`\@xdblarg`'s `[{#2}]`), so a `]` in the caption
/// does not end it ("Dimensions [mm] …" was cut and leaked into the table). pdflatex: clean.
#[test]
fn supertabular_caption_keeps_its_brackets() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{supertabular}
\begin{document}
\tablecaption{Dimensions [mm] of parts}
\begin{supertabular}{ll}
a & b\\
\end{supertabular}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Dimensions [mm] of parts</toccaption><caption><tag close=": ">Table 1</tag>Dimensions [mm] of parts</caption><tabular><tr><td align="left">a</td><td align="left">b</td></tr></tabular></table>"#,
  );
}

/// 62b review: under book's two-sided default `\@endpart` makes the blank verso a counted page (a second `newpage`
/// marker) without `\null`'s empty paragraph (raw book-based classes call it: hpsdiss, nddiss2e, suftesi). pdflatex:
/// "A" on page 3, "B" on page 5.
#[test]
fn book_endpart_blank_verso_has_no_empty_paragraph() {
  let xml = assert_elements(
    r"\documentclass{book}
\begin{document}
\part{One}
A
\makeatletter\@endpart\makeatother
B
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "part",
    &[],
    r#"<part inlist="toc" xml:id="Pt1"><tags><tag>Part I</tag><tag role="refnum">I</tag><tag role="typerefnum">Part I</tag></tags><title><tag close=" ">Part I</tag>One</title><toctitle><tag close=" ">I</tag>One</toctitle><para xml:id="Pt1.p1"><p>A</p></para><pagination role="newpage"/><pagination role="newpage"/><para xml:id="Pt1.p2"><p>B</p></para></part>"#,
  );
}

/// 62b review: an in-cell `\\[2pt]` passes its length on to the break and nothing after it is read again: tabularray's
/// `{c\\[2pt] [d]}` keeps the text "[d]" (the second read took it as another length). pdflatex: "c", then "[d]".
#[test]
fn in_cell_break_length_is_read_once() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{p{4em}}
{c\\[2pt] [d]}\\
\end{tblr}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "td",
    &[],
    r#"<td align="left" vattach="top"><inline-block vattach="top" width="40.0pt"><p>c</p><p>[d]</p></inline-block></td>"#,
  );
}

/// 62c (user ruling 2026-10-04; OXIDIZED_DESIGN_DIVERGENCES #441): a `\patchcmd` on latex.ltx's output routine that
/// misses succeeds without patching — LaTeXML runs no output routine. acl2019.sty:455's
/// `\patchcmd\@combinedblfloats{\box\@outputbox}{\unvbox\@outputbox}{}{\errmessage{patch failed}}` misses TL 2025's
/// body (16 ACL/EMNLP papers of run 329: 1811.00207, 1907.04380, 1909.00156, 2004.13897; pdflatex 2025 errs too).
/// So does a miss on LaTeXML's empty stub (`\@floatplacement`, whose latex.ltx body the search names); a hit still
/// patches, and a miss elsewhere still fails.
#[test]
fn output_routine_patch_miss_succeeds() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{etoolbox}
\makeatletter
\patchcmd\@combinedblfloats{\box\@outputbox}{\unvbox\@outputbox}{\def\one{OK}}{\errmessage{patch failed}}
\patchcmd\@floatplacement{\global\@topnum}{\relax}{\def\two{OK}}{\errmessage{patch failed}}
\patchcmd\@emptycol{\vbox{}}{\vbox{}\relax\relax}{\def\three{hit}}{\errmessage{patch failed}}
\def\mine{abc}
\patchcmd\mine{xyz}{q}{\def\four{patched}}{\def\four{missed}}
\begin{document}
\one, \two, \three, \four. \texttt{\meaning\@emptycol}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p>OK, OK, hit, missed. <text font="typewriter">macro:-&gt;\vbox {}\relax \relax \penalty -\@M </text></p>"#,
  );
}
