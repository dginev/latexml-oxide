//! Red/green guards for perfect-kernel phase-61 batches: G3 residual slices 7a/7b (T1/T2 ASCII slots, the Unicode
//! profiles' OpenType flag, `\pdfsetmatrix`'s matrix, copyright lines a class prints only in a page foot).
use latexml::util::test::{assert_element, convert_files_with, convert_with, xml_element};

use super::{
  perfect_kernel_batch46::{convert_with_then, error_count, warning_count},
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
/// "Hello.". Since 62k the production profile (ar5iv's `localrawclasses`) runs such a class raw, as TeX does, and the
/// scan is reached without it (no preload), where the OmniBus fallback's missing-binding warning is the one warning.
#[test]
fn scanned_class_options_naming_macros_stay_inert() {
  const SHIPPED: &str = r"\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{shipped}
\def\@PaperFormat{letterpaper}
\newif\ifthesis
\ifthesis
  \LoadClass[\@PaperFormat,twoside]{report}
\else
  \LoadClass[\@PaperFormat,twoside]{article}
\fi
";
  for (preload, diagnostics, para) in [
    (
      Some("ar5iv.sty"),
      (0, 0),
      r#"<para xml:id="p1"><p xml:id="p1.1">Hello.</p></para>"#,
    ),
    (None, (0, 1), r#"<para xml:id="p1"><p>Hello.</p></para>"#),
  ] {
    let (log, xml) = convert_files_with(
      "\\documentclass{shipped}\n\\begin{document}\nHello.\n\\end{document}\n",
      &[("shipped.cls", SHIPPED)],
      preload,
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      diagnostics,
      "{preload:?}: {log}"
    );
    assert_element(&xml, "para", &[], para);
  }
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

/// 62d: arximspdf/arxstspdf's `\printead*[text]{e1}` (arximspdf.cls:599-603) reprints the addresses `\ead[label=e1]`
/// recorded, which `\ead` already adds as contacts. The binding's no-op list held `"printead*"`, the prototype
/// `\printead` + a literal `*`, which replaced the plain one: every `\printead{e1}` raised "Missing argument Match"
/// (30 aoas/aos/sts papers of run 329: 1107.4843, 1011.3351, 1205.6055).
#[test]
fn arximspdf_printead_takes_star_option_and_labels() {
  let xml = assert_elements(
    r"\documentclass[aos]{arximspdf}
\begin{document}
\begin{frontmatter}
\title{A Title}
\begin{aug}
\author{\fnms{Jane} \snm{Doe}\ead[label=e1]{jd@x.org}}
\address{Dept. of Statistics\\ \printead{e1}\\ \printead*{e1}\\ \printead[mail]{e1}}
\end{aug}
\end{frontmatter}
Text.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "contact",
    &["role=\"email\""],
    r#"<contact name="e-mail: " role="email">jd@x.org</contact>"#,
  );
  assert_element(
    &xml,
    "contact",
    &["role=\"address\""],
    "<contact name=\"Address:\u{a0}\" role=\"address\">Dept. of Statistics</contact>",
  );
}

/// 62d: arximspdf's `{pf}`/`{pf*}` (arximspdf.cls:1428-1434): a run-in "Proof."/"<name>." and the class's automatic
/// `\@qed` □ at the end (:1417-1426), which `\noqed` drops once. The binding mapped them to an amsthm `{proof}` the class
/// never loads (12 papers of run 329: 1205.6055, 1104.1047 with 22 `pf` and no `\qed`).
#[test]
fn arximspdf_pf_proofs_end_with_the_class_qed() {
  let xml = assert_elements(
    r"\documentclass[aos]{arximspdf}
\begin{document}
\begin{pf}
Trivial.
\end{pf}
\begin{pf*}{Proof of the claim}
Also trivial.\upqed
\end{pf*}
\noqed
\begin{pf}
No box.
\end{pf}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><proof><title class="ltx_runin">Proof.</title><para xml:id="p1"><p>Trivial.<Math mode="inline" tex="\square" text="square" xml:id="p1.m1"><XMath><XMTok name="square" role="UNKNOWN">□</XMTok></XMath></Math></p></para></proof><proof><title class="ltx_runin">Proof of the claim.</title><para xml:id="p2"><p>Also trivial.<Math mode="inline" tex="\square" text="square" xml:id="p2.m1"><XMath><XMTok name="square" role="UNKNOWN">□</XMTok></XMath></Math></p></para></proof><proof><title class="ltx_runin">Proof.</title><para xml:id="p3"><p>No box.</p></para></proof></document>"#,
  );
}

/// 62d: the rest of what the IMS classes define and their papers use: `\tablewidth` and `\tabnotetext`/`\tabnoteref`
/// with an explicit mark (arximspdf.cls:1729-1790), `{sidewaystable}` under the `rotating` option (:164, :1838-1848; 1304.4448), and
/// arxstspdf's `\doiurl`/`\arxivurl` (arxstspdf.cls:2902-2975; 0903.0664).
#[test]
fn arxstspdf_tables_and_links() {
  let xml = assert_elements(
    r"\documentclass[aos,rotating]{arxstspdf}
\begin{document}
\begin{sidewaystable}
\setlength{\tablewidth}{\textwidth}
\caption{Data}
\begin{tabular}{l}A\tabnoteref[a]{n1}\end{tabular}
\tabnotetext[a]{n1}{A note.}
\end{sidewaystable}
See \doiurl{10.1214/09-AOS1} and \arxivurl{math.PR/0603300}.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "table",
    &[],
    "<table angle=\"90\" depth=\"0.0pt\" height=\"550.0pt\" inlist=\"lot\" innerdepth=\"36.0pt\" innerheight=\"6.8pt\" innerwidth=\"550.0pt\" width=\"42.8pt\" xml:id=\"S0.T1\" xtranslate=\"-253.6pt\" ytranslate=\"-253.6pt\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Data</toccaption><caption><tag close=\": \">Table 1</tag>Data</caption><tabular class=\"ltx_figure_panel\" vattach=\"middle\"><tbody><tr><td align=\"left\">A<sup>a</sup></td></tr></tbody></tabular><break class=\"ltx_break\"/><p class=\"ltx_figure_panel\"><sup>a</sup>A note.</p></table>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>See <ref class="ltx_href" href="http://dx.doi.org/10.1214/09-AOS1">10.1214/09-AOS1</ref> and <ref class="ltx_href" href="http://arxiv.org/abs/math.PR/0603300">math.PR/0603300</ref>.</p></para>"#,
  );
}

/// 62d: amsmath.sty:754-758 makes each `\DeclareMathAccent` accent `\mathaccentV{<name>}<family><slot>` (`\hat` is
/// `\mathaccentV{hat}05E`); the binding keeps LaTeXML's own accents, so the call arrives only written out, in a revtex
/// bibnote's `.bbl` (`\protect\mathaccentV {hat}05E{D}`; 2008.11212, 1309.7027, 2004.12163, 1811.07295). It reads as
/// the named accent; an accent name LaTeXML lacks keeps its base.
#[test]
fn mathaccent_v_reads_as_the_named_accent() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
$\protect\mathaccentV{hat}05E{D}+\mathaccentV{nosuchaccent}05E{x}$
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok font="italic" role="UNKNOWN">D</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath>"#,
  );
}

/// 62d: ragged2e.sty:292-297's `\newenvironment{justify}{\trivlist\justifying\item\relax}{\endtrivlist}` defines the
/// commands `\justify`/`\endjustify`, which papers call bare as a switch ("undefined \justify" in 16 papers of run 329:
/// 2204.13885, 1903.04078, 1909.10090), and its text is a paragraph of its own (the no-op ran "text.More" together;
/// witness 2406.15288).
#[test]
fn ragged2e_justify_command_and_environment_are_paragraphs() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{ragged2e}
\begin{document}
Before.
\justify
Some text.
\begin{justify}\bfseries More text.\end{justify}
After.
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>Before.</p></para><para xml:id="p2"><p>Some text.</p></para><para xml:id="p3"><p><text font="bold">More text.</text></p></para><para xml:id="p4"><p>After.</p></para></document>"#,
  );
}

/// 62d: INTERSPEECH2021.sty:50-54 (and 2022's) require graphicx, amssymb, amsmath, bm, textcomp, booktabs and
/// caption, and :69-70 make `\vec`/`\mat` bold; bound as plain spconf, a paper's `\includegraphics` was undefined
/// (2103.14512, 2211.09381, 2106.13419; 15 papers of run 329).
#[test]
fn interspeech_style_loads_its_packages() {
  let xml = assert_elements(
    r"\documentclass[a4paper]{article}
\usepackage{INTERSPEECH2021}
\title{A Title}
\name{Jane Doe}
\address{Somewhere}
\begin{document}
\maketitle
\begin{figure}[t]
\centering
\includegraphics[width=0.5\linewidth]{nofile}
\caption{A figure.}
\end{figure}
$\vec{x}$
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "graphics",
    &[],
    r#"<graphics class="ltx_centering" graphic="nofile" options="width=172.5pt,keepaspectratio=true" xml:id="S0.F1.g1"/>"#,
  );
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMTok font="bold italic" role="UNKNOWN">x</XMTok></XMath>"#,
  );
}

/// 62e (KNOWN_PERL_ERRORS #471): `[algo2e]` renames algorithm2e's environment, so algorithm.sty's `\newfloat{algorithm}`
/// holds the statements in a plain float; the line machinery closed a listingline that was not open and opened lines a
/// float cannot hold (5 errors; 2004.01608, 2406.10356). The first block opens an auto-closing listing.
#[test]
fn algorithm2e_statements_outside_a_listing() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage[algo2e,ruled]{algorithm2e}
\usepackage{algorithm}
\begin{document}
\begin{algorithm}[ht]
\caption{Demo}
Initialize $x$\;
\For{$i=1$}{
  update $x$\;
}
\end{algorithm}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "listing",
    &[],
    "<listing class=\"ltx_lst_numbers_left\"><listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}update <Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"algorithm1.m3\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></listingline><listingline>end for</listingline><listingline/></listing>",
  );
}

/// 62e (KNOWN_PERL_ERRORS #471): `\\` or a block macro inside a list item of an algorithm breaks the line inside the
/// item instead of closing the listingline across it (2004.03005, 1709.07249), and the lines after the list split
/// again (the wrappers the document opened for the list bar nothing; 2203.03384).
#[test]
fn algorithm2e_line_split_inside_an_item_is_a_break() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage[ruled]{algorithm2e}
\begin{document}
\begin{algorithm}[ht]
\caption{Demo}
\begin{description}
\item[Init] \ \\
Choose the constants.
\end{description}
\begin{enumerate}
\item \ForEach{$x$}{count $x$}
\item Place the elements.
\end{enumerate}
last\;
\For{$i$}{inner\;}
\end{algorithm}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "listing",
    &[],
    "<listing class=\"ltx_lst_numbers_left\" framed=\"topbottom\"><listingline><inline-block><description xml:id=\"S0.I1\"><item xml:id=\"S0.I1.ix1\"><tags><tag><text font=\"bold\">Init</text></tag><tag role=\"typerefnum\">item Init</tag></tags><para xml:id=\"S0.I1.ix1.p1\"><p><break/>Choose the constants.</p></para></item></description></inline-block></listingline><listingline><inline-block><enumerate xml:id=\"S0.I2\"><item xml:id=\"S0.I2.i1\"><tags><tag>1.</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">item\u{a0}1</tag></tags><para xml:id=\"S0.I2.i1.p1\"><p><text font=\"bold\">foreach</text> <emph font=\"italic\"><Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"S0.I2.i1.p1.m1\"><XMath><XMTok role=\"UNKNOWN\">x</XMTok></XMath></Math></emph> <text font=\"bold\">do<break/></text>count <Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"S0.I2.i1.p1.m2\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math><break/>end foreach<break/></p></para></item><item xml:id=\"S0.I2.i2\"><tags><tag>2.</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">item\u{a0}2</tag></tags><para xml:id=\"S0.I2.i2.p1\"><p>Place the elements.</p></para></item></enumerate></inline-block></listingline><listingline>last;</listingline><listingline><text font=\"bold\">for</text> <emph font=\"italic\"><Math mode=\"inline\" tex=\"i\" text=\"i\" xml:id=\"algorithm1.m1\"><XMath><XMTok role=\"UNKNOWN\">i</XMTok></XMath></Math></emph> <text font=\"bold\">do</text></listingline><listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}inner;</listingline><listingline>end for</listingline><listingline/></listing>",
  );
}

/// 62e (KNOWN_PERL_ERRORS #471): `\par` in restricted horizontal mode does nothing (tex.web §1094, §1096), so a
/// comment's closing `\par` inside `\text{}` or `\mbox{}` ends no algorithm line (2010.03983); a `\parbox`'s own
/// `\\` breaks its text.
#[test]
fn algorithm2e_par_in_a_box_ends_no_line() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{amsmath}
\usepackage[ruled]{algorithm2e}
\begin{document}
\begin{algorithm}[H]
update \mbox{fluid\par update}\;
see \fbox{\parbox{3cm}{first\\second}}\;
\[ z=1 \text{ \tcp{Fluid update}} \]
next line\;
\caption{Demo}
\end{algorithm}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "listing",
    &[],
    "<listing class=\"ltx_lst_numbers_left\" framed=\"topbottom\"><listingline>update fluidupdate;</listingline><listingline>see <text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\">first<break/>second</text>;</listingline><listingline><equation xml:id=\"S0.Ex1\"><Math mode=\"display\" tex=\"z=1\\text{ {\\hbox{{{\\hbox{// }}}}{{Fluid update\\hfill}}}}\" text=\"z = 1 * [ // Fluid update ]\" xml:id=\"S0.Ex1.m1\"><XMath><XMApp><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">z</XMTok><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok><XMText> <text font=\"typewriter\">// </text><text font=\"typewriter\">Fluid update </text></XMText></XMApp></XMApp></XMath></Math></equation>next line;</listingline><listingline/></listing>",
  );
}

/// 62e (KNOWN_PERL_ERRORS #471): with `linesnumbered`, a line behind a list item takes no number (`\@item` empties
/// `\everypar`); the tags floated out onto the item, three on one item (2001.00288, 1804.09120).
#[test]
fn algorithm2e_numbered_lines_inside_an_item() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage[ruled,linesnumbered]{algorithm2e}
\begin{document}
\begin{algorithm}[H]
first\;
\begin{enumerate}
\item \ForEach{$x$}{count $x$\; more $x$\;}
\item Place the elements.
\end{enumerate}
last\;
\caption{Demo}
\end{algorithm}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "listing",
    &[],
    "<listing class=\"ltx_lst_numbers_left\" framed=\"topbottom\"><listingline><tags><tag><text font=\"bold\">1</text></tag></tags>first;</listingline><listingline><inline-block><enumerate xml:id=\"S0.I1\"><item xml:id=\"S0.I1.i1\"><tags><tag>1.</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">item\u{a0}1</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p><text font=\"bold\">foreach</text> <emph font=\"italic\"><Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"S0.I1.i1.p1.m1\"><XMath><XMTok role=\"UNKNOWN\">x</XMTok></XMath></Math></emph> <text font=\"bold\">do<break/></text>count <Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"S0.I1.i1.p1.m2\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math>;<break/>more <Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"S0.I1.i1.p1.m3\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math>;<break/>end foreach<break/></p></para></item><item xml:id=\"S0.I1.i2\"><tags><tag>2.</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">item\u{a0}2</tag></tags><para xml:id=\"S0.I1.i2.p1\"><p>Place the elements.</p></para></item></enumerate></inline-block></listingline><listingline><tags><tag><text font=\"bold\">2</text></tag></tags>last;</listingline><listingline/></listing>",
  );
}

/// 62e (KNOWN_PERL_ERRORS #471): a line is reachable only through inline wrappers and the wrappers the document opened
/// directly in it, so a `\par`/`\\` inside a minipage, a `\vbox`, a table cell or a footnote breaks that box's text and
/// stays in it; a list's end ends the line (paralist's display lists too), but not inside a `\parbox`; math ends no
/// line (`{algomathdisplay}`'s `;`, a caption's `$a\\b$`; 2507.17199, 2602.19085); a caption's `\\` is a break
/// (1412.0600).
#[test]
fn algorithm2e_boxes_math_and_list_ends() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{paralist}
\usepackage[ruled]{algorithm2e}
\begin{document}
\begin{algorithm}[H]
\begin{minipage}{3cm} mini one\par mini two \end{minipage}\;
\vbox{vb one\par vb two}\;
\begin{tabular}{p{4cm}}\begin{itemize}\item cell item\end{itemize} cell tail\end{tabular}\;
note\footnote{fn one\par fn two}\;
\begin{compactitem}\item compact\end{compactitem}
after compact\;
\begin{algomathdisplay}x=1\end{algomathdisplay}
\parbox[t]{5cm}{\begin{itemize}\item boxed item\end{itemize}}\;
\caption{Demo $a\\b$ first\\second}
\end{algorithm}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "listing",
    &[],
    "<listing class=\"ltx_lst_numbers_left\" framed=\"topbottom\"><listingline><inline-block class=\"ltx_minipage\" vattach=\"middle\" width=\"85.4pt\"><p>mini one<break/>mini two</p></inline-block>;</listingline><listingline><inline-block vattach=\"bottom\"><p>vb one<break/>vb two</p></inline-block>;</listingline><listingline><tabular vattach=\"middle\"><tbody><tr><td align=\"left\" vattach=\"top\"><inline-block vattach=\"top\" width=\"113.8pt\"><itemize xml:id=\"S0.I1\"><item xml:id=\"S0.I1.i1\"><tags><tag>\u{2022}</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p>cell item</p></para></item></itemize><p>cell tail</p></inline-block></td></tr></tbody></tabular>;</listingline><listingline>note<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote 1</tag></tags>fn one<break/>fn two</note>;</listingline><listingline><inline-block><itemize xml:id=\"S0.I2\"><item xml:id=\"S0.I2.i1\"><tags><tag>\u{2022}</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I2.i1.p1\"><p>compact</p></para></item></itemize></inline-block></listingline><listingline>after compact;</listingline><listingline><equation xml:id=\"S0.Ex1\"><Math mode=\"display\" tex=\"x=1;\" text=\"x = 1\" xml:id=\"S0.Ex1.m1\"><XMath><XMDual><XMRef idref=\"S0.Ex1.m1.1\"/><XMWrap><XMApp xml:id=\"S0.Ex1.m1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp><XMTok role=\"PUNCT\">;</XMTok></XMWrap></XMDual></XMath></Math></equation><inline-block><itemize xml:id=\"S0.I3\"><item xml:id=\"S0.I3.i1\"><tags><tag>\u{2022}</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I3.i1.p1\"><p>boxed item</p></para></item></itemize></inline-block>;</listingline><listingline/></listing>",
  );
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\" \"><text font=\"bold\">Algorithm\u{a0}1</text></tag>Demo <Math mode=\"inline\" tex=\"a\\\\&#10;b\" text=\"a * b\" xml:id=\"algorithm1.m2\"><XMath><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp></XMath></Math> first<break/>second</caption>",
  );
}

/// 62f: `\hphantom`'s brace peek was LaTeX's `\@ifnextchar`, so in a plain TeX document it autoloaded the LaTeX format
/// mid-document and the dump replaced the document's own macros: a `\def\ref` became LaTeX's `\ref`, and harvmac's
/// `\refs` (harvmac.tex:186-191, `\hphantom` then `\edef` of a label `\lref` defines as `\ref\X`) re-expanded the label
/// forever (`Timeout:PushbackLimit`; 64 plain harvmac papers of run 329: hep-th0505019, hep-th9210021, hep-th9805158,
/// gr-qc9306023). tex: "A REF. B REF."; "Text [1].".
#[test]
fn hphantom_in_plain_tex_loads_no_latex() {
  let xml = assert_elements(
    "\\def\\ref{REF}\nA \\ref. \\hphantom{x} B \\ref.\n\\bye\n",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para><p>A REF. <text class="ltx_phantom">x</text> B REF.</p></para>"#,
  );
  let xml = assert_elements(
    "\\input harvmac\n\\lref\\GiddingsYU{S.~B.~Giddings, Hierarchies.}\nText \\refs{\\GiddingsYU}.\n\\bye\n",
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para><p>Text [<text class="ltx_phantom">[1]</text>1<text class="ltx_phantom"/>].</p></para>"#,
  );
}

/// 62i (run-329 PushbackLimit Fatals): a siunitx `S` cell's number ends at a `\relax`, where siunitx's own collector
/// stops (siunitx.sty:5641-5664) — the rest of the cell runs as it comes. csvsimple ends every row's command with one
/// (csvsimple-legacy.sty:289-293 `\csv@body\relax`); the binding read the cell on with expansion, unrolling csvsimple's
/// next-line loop without end — its `\read` never ran (2108.13640, `PushbackLimit`; Perl reads alike, KPE #474).
/// pdflatex: the three rows IN 5.1 / PV 4.4 / MEAN 9.6. Under the production preload, as the witness.
#[test]
fn csvsimple_rows_in_a_siunitx_column_end_at_the_relax() {
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\usepackage{siunitx}
\usepackage{csvsimple}
\def\imagenet{IN}\def\pvpower{PV}\def\mean{MEAN}
\begin{document}
\csvloop{file=summary.csv, head to column names=false,
 column names={model=\model,mae=\mae,texmodel=\texmodel},
 tabular={lS}, command=\texmodel & \mae}
\end{document}
",
    &[(
      "summary.csv",
      "model,mae,texmodel\nimagenet,5.1,\\imagenet\npvpower,4.4,\\pvpower\nmean,9.6,\\mean\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(log.matches("Fatal:").count(), 0, "{log}");
  // csvsimple-legacy loads shellesc, which reports the disabled shell escape (as pdflatex does).
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains("Package shellesc Warning: Shell escape disabled"),
    "{log}"
  );
  for (row, (name, number)) in [("IN", "5.1"), ("PV", "4.4"), ("MEAN", "9.6")]
    .into_iter()
    .enumerate()
  {
    let id = format!("p1.1.{}", row + 1);
    let m = row + 1;
    assert_element(
      &xml,
      "tr",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<tr xml:id="{id}"><td align="left" thead="row" xml:id="{id}.1">{name}</td><td align="left" xml:id="{id}.2"><Math mode="inline" tex="{number}" text="{number}" xml:id="p1.m{m}"><XMath xml:id="p1.m{m}.1"><XMTok meaning="{number}" role="NUMBER">{number}</XMTok></XMath></Math></td></tr>"#
      ),
    );
  }
}

/// 62i: `\affiliations`/`\emails` split an author block IJCAI-style only as separators — at its top level, undefined
/// or a no-op there. A document's own macro of that name is author text, as in LaTeX; taken for a marker, the split
/// handed the block back unsplit, without end: `\emails` before any `\affiliations` (2407.10582) and `\affiliations`
/// inside `\thanks` (2505.05474), both `PushbackLimit`. pdflatex: "A. Author" over "a@b.c"; "Beichen Wen" with the
/// footnote "The authors are with S-Lab".
#[test]
fn ijcai_marker_names_a_document_defines_are_author_text() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\newcommand*{\emails}{\texttt{a@b.c}}
\title{T}
\author{A. Author \\ \emails}
\begin{document}
\maketitle
Hello.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    // The contact's name ends in a no-break space.
    &format!(
      r#"<creator role="author"><personname>A. Author</personname><contact name="Affiliation:{}" role="affiliation"><text font="typewriter" xml:id="id1">a@b.c</text></contact></creator>"#,
      '\u{a0}'
    ),
  );
  let (log, xml) = convert_with(
    r"\documentclass{article}
\def\affiliations{The authors are with S-Lab}
\title{T}
\author{Beichen Wen\thanks{\affiliations}}
\begin{document}
\maketitle
Hello.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Beichen Wen</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">The authors are with S-Lab</note></creator>"#,
  );
}

/// 62i: `\selectfont` defines `\curr@fontshape/\f@size` as the font, as latex.ltx's `\pickup@font` does, so `\em`
/// under `\DeclareEmphSequence` (latex.ltx:14048-14069) stops rotating once the font changed — with every name
/// undefined both sides of its `\ifx` were `\relax` (2309.08676, 2502.21053; `PushbackLimit`; KPE #477). pdflatex:
/// "A", "b" italic, "c" bold italic, "d" bold upright (`\emreset`), "e".
#[test]
fn em_under_declare_emph_sequence_stops_at_a_new_font() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\DeclareEmphSequence{\itshape,\bfseries}
\begin{document}
A {\em b {\em c {\em d}}} e
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">A <text font="italic" xml:id="p1.1.1">b <text font="bold" xml:id="p1.1.1.1">c <text font="upright" xml:id="p1.1.1.1.1">d</text></text></text> e</p>"#,
  );
}

/// 62i: a `\tag` text that mentions `\theequation` past its first token gets the counter's value, as amsmath's
/// `\df@tag` does (amsmath.sty:1224-1227); the binding's one-step `\expandafter` chain made `\theequation` call
/// itself (2408.12869, `\tag*{(\theequation)$_i$}`; Perl out of memory, KPE #475). pdflatex: "(0)i".
#[test]
fn tag_text_mentioning_theequation_gets_the_counter() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{equation}
  x = y \tag*{(\theequation)$_i$}
\end{equation}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "tags",
    &[],
    r#"<tags><tag>(0)<sub xml:id="S0.Ex1.1"><text font="italic" xml:id="S0.Ex1.1.1">i</text></sub></tag><tag role="refnum">(0)<sub xml:id="S0.Ex1.2"><text font="italic" xml:id="S0.Ex1.2.1">i</text></sub></tag></tags>"#,
  );
}

/// 62i: a document that saves `\cite` as `\@@cite` (a name LaTeX leaves free) keeps citing: the bindings call
/// LaTeXML's constructor as `\lx@@cite`, so they no longer call themselves through the saved copy (2305.06365,
/// revtex4-2 + natbib, `PushbackLimit`; Perl alike, KPE #476; OXIDIZED_DESIGN_DIVERGENCES #443).
#[test]
fn a_cite_saved_as_at_at_cite_still_cites() {
  let kernel = r#"<p xml:id="p1.1">A <cite class="ltx_citemacro_cite">[<bibref bibrefs="x" separator="," yyseparator=","/>]</cite> B</p>"#;
  let natbib = r#"<p xml:id="p1.1">A <cite class="ltx_citemacro_cite"><bibref bibrefs="x" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","><bibrefphrase>(</bibrefphrase><bibrefphrase>)</bibrefphrase></bibref></cite> B</p>"#;
  for (package, expected) in [("", kernel), (r"\usepackage{natbib}", natbib)] {
    let (log, xml) = convert_with(
      &format!(
        r"\documentclass{{article}}{package}
\makeatletter\let\@@cite\cite
\renewcommand\cite[1]{{\@@cite{{#1}}}}\makeatother
\begin{{document}}
A \cite{{x}} B
\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(error_count(&log), 0, "{log}");
    assert_eq!(log.matches("Fatal:").count(), 0, "{log}");
    assert_element(&xml, "p", &[], expected);
  }
}

/// 62j: elsarticle's journal layout is the class's `\jtype` macro (elsarticle.cls:71-87: `\def\jtype{0}`, `\xdef`'d
/// by `preprint`, `1p`, `3p`, `5p`); journal styles test it (ecrc.sty:7 `\ifnum\jtype=1`). The binding kept it as an
/// internal value only, so ecrc's test raised "undefined" and a relational-token error (170 run-329 papers,
/// 1011.4942). `\ifpreprint` likewise follows the options (ycviu.sty:60 tests it).
#[test]
fn elsarticle_journal_layout_is_the_jtype_macro() {
  // `\jtype` and `\ifpreprint` per option set (elsarticle.cls:71-87, 112: `preprint` by default; the later declared
  // option wins). pdflatex: "0P", "3F", "0P" (review), "1F", "0F" (`[final]`: preprint's 0, then final's false).
  for (options, expected) in [
    ("", "0P"),
    ("3p", "3F"),
    ("review", "0P"),
    ("1p,preprint", "1F"),
    ("final", "0F"),
  ] {
    let (log, xml) = convert_with(
      &format!(
        r"\documentclass[{options}]{{elsarticle}}
\begin{{document}}
\jtype\ifpreprint P\else F\fi
\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(error_count(&log), 0, "{log}");
    assert_element(
      &xml,
      "p",
      &[],
      &format!(r#"<p xml:id="p1.1">{expected}</p>"#),
    );
  }
}

/// 62j: txfonts' (and newtxmath's, which loads it) variant letters `\varv`, `\varw`, `\vary` — no codepoints of their
/// own, left out by Perl (txfonts.sty.ltxml:379-381) and undefined — are set as the letters they are (153 run-329
/// papers, astro-ph0410697). pdflatex: v w y in their variant shapes.
#[test]
fn txfonts_variant_letters_are_their_letters() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{txfonts}
\begin{document}
$\varv\varw\vary$
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains(r#"tex="\varv\varw\vary""#), "{xml}");
  for letter in ["v", "w", "y"] {
    let name = format!(r#"name="var{letter}""#);
    assert_element(
      &xml,
      "XMTok",
      &[&name],
      &format!(r#"<XMTok font="italic" {name} role="UNKNOWN">{letter}</XMTok>"#),
    );
  }
}

/// 62j: a box sized while it is being absorbed keeps its size without the memo: an auto-opened `ltx:picture` (a
/// `\put` in text) closed by a `\par` inside the `{picture}` that follows measured that whatsit and stored the size on
/// it while `Document::absorb` held it — "RefCell already borrowed", a panic (1111.1991; Perl clean, Box.pm:291-293).
/// The memo store now gives way (`set_memo_property`). Perl: the outer picture 7.3 by 5.96.
#[test]
fn a_picture_measured_while_absorbed_does_not_panic() {
  // The busy `{picture}`'s own size reaches the outer one through the memo-less path. Perl: 7.3×5.96, 76.49×41.51.
  for (size, outer) in [
    (
      "0,0",
      r#"<picture height="5.96" width="7.3" xml:id="pic1">"#,
    ),
    (
      "50,30",
      r#"<picture height="41.51" width="76.49" xml:id="pic1">"#,
    ),
  ] {
    let (log, xml) = convert_with(
      &format!(
        r"\documentclass{{article}}
\usepackage{{subfig}}
\begin{{document}}
\begin{{figure}}
a\put(0,0){{x}}
\begin{{picture}}({size})
\hskip1cm\subfloat[v]{{y}}
\put(0,0){{y}}
\end{{picture}}
\end{{figure}}
\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(error_count(&log), 0, "{log}");
    assert_eq!(log.matches("Fatal:").count(), 0, "{log}");
    assert!(xml.contains(outer), "{size}: {xml}");
  }
}

/// 62k: the production profile (`ar5iv.sty`) interprets a class the paper ships and no binding covers raw, as TeX does
/// (`localrawclasses`, OXIDIZED_DESIGN_DIVERGENCES #444), instead of OmniBus's guesses: its journal macros are defined.
/// Run 329: ~35% of the erroring papers ran on OmniBus over a shipped class (webofc `\woctitle`, RAA `\pagerange`,
/// PASJ `\KeyWords`, …; 1301.7514). A class in TeX Live without a binding stays on OmniBus. pdflatex: "pp. 1–2 Hello."
#[test]
fn a_shipped_class_without_a_binding_is_interpreted() {
  let (log, xml) = convert_files_with(
    "\\documentclass{shippedjournal}\n\\begin{document}\n\\pagerange{1--2} Hello.\n\\end{document}\n",
    &[(
      "shippedjournal.cls",
      r"\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{shippedjournal}
\LoadClass{article}
\newcommand\pagerange[1]{pp.~#1}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">pp.\u{a0}1\u{2013}2 Hello.</p></para>",
  );
}

/// 62k: extsizes' `extarticle`, `extreport` and `extbook` are the standard classes with more body sizes; their bindings
/// pass the other options on. Without them they fell to OmniBus, whose guesses blocked a raw class built on one
/// (opticajnl.cls `\LoadClass{extarticle}` then lost its `\journal`, 2403.09007). `\@ptsize` is the body size in
/// points, as the classes set it (extarticle.cls:51-58), not the standard classes' 0-2.
#[test]
fn extsizes_classes_are_the_standard_ones() {
  for class in ["extarticle", "extreport", "extbook"] {
    let (log, xml) = convert_with(
      &format!(
        "\\documentclass[14pt,twocolumn]{{{class}}}\n\\begin{{document}}\nHello \\csname @ptsize\\endcsname.\n\\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    assert!(
      xml.contains(&format!(
        r#"<?latexml class="{class}" options="14pt,twocolumn"?>"#
      )),
      "{class}: {xml}"
    );
    assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">Hello 14.</p>"#);
  }
}

/// 62k: IEEEtran's `\ifCLASSINFOpdf` is true in PDF output only (IEEEtran.cls:552-557 `\ifcase\pdfoutput`), so a
/// source that ships EPS figures (`\pdfoutput` 0, the K6 ruling) takes its preamble's
/// `\ifCLASSINFOpdf…\else\usepackage[dvips]{graphicx}\fi` branch; the binding set it true always, as Perl does, and 22
/// run-329 papers had `\includegraphics` undefined (0902.1911). pdflatex: "D" with `\pdfoutput=0`, "P" without.
#[test]
fn ieeetran_pdf_flag_follows_pdfoutput() {
  for (setting, expected) in [(r"\pdfoutput=0", "D"), ("", "P")] {
    let (log, xml) = convert_with(
      &format!(
        r"{setting}\documentclass{{IEEEtran}}
\begin{{document}}
\ifCLASSINFOpdf P\else D\fi
\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_element(
      &xml,
      "p",
      &[],
      &format!(r#"<p xml:id="p1.1">{expected}</p>"#),
    );
  }
}

/// 62k: `\documentstyle{mn}` finds the `mn` binding with the arXiv profile's `localrawclasses` set: its probes ask the
/// binding registry, as Perl's FindFile prefers a binding whatever `notex` says (Package.pm:2126-2129). They missed it
/// and fell to OmniBus — `\ifoldfss` undefined, `{keywords}` lost (astro-ph0008081, astro-ph0105519).
#[test]
fn documentstyle_finds_its_binding_with_local_raw_classes() {
  let (log, xml) = convert_with(
    r"\documentstyle{mn}
\begin{document}
\title{T}\author{A}\maketitle
\begin{keywords}stars\end{keywords}
Hello \ifoldfss x\fi.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">stars</keywords>",
  );
}

/// 62k: tocbibind's conditionals are tocbibind.sty's (:38-69) — `\if@doto…` true unless a `not…` option clears it,
/// `\if@bibchapter` from the class's chapters — which tocloft's `\tableofcontents` reads (tocloft.sty:105-114); a
/// missing `\if@bibchapter` left a stray `\fi` (SciPost.cls loads both; 1811.09408, 2105.01655, 2203.11601; Perl
/// defines none, KPE #482).
#[test]
fn tocloft_reads_tocbibind_conditionals() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{tocloft}
\usepackage[nottoc,notlot,notlof]{tocbibind}
\begin{document}
\tableofcontents
\section{A}x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The kernel's contents list (62n, tocloft_sty.rs), its entries filled in post-processing.
  assert_element(
    &xml,
    "TOC",
    &[],
    r#"<TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Contents</title></TOC>"#,
  );
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>A</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">x</p></para></section>"#,
  );
}

/// 62k: ragged2e saves LaTeX's own commands as `\LaTeXcentering` & co. (ragged2e.sty:298-311), which classes restore
/// (sbc20.cls:523 `\let\centering\LaTeXcentering`); undefined before, `\centering` became undefined (2205.12270; Perl
/// lacks them too, KPE #481).
#[test]
fn ragged2e_saves_latex_commands() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage[newcommands]{ragged2e}
\let\centering\LaTeXcentering
\begin{document}
{\centering Centered.\par}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The same markup `\centering` gives without ragged2e.
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para align="center" xml:id="p1"><p xml:id="p1.1">Centered.</p></para>"#,
  );
  // Without `newcommands` (`originalcommands` is the default, ragged2e.sty:119) nothing is saved, as in pdflatex.
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{ragged2e}
\begin{document}
\ifdefined\LaTeXcentering saved\else unsaved\fi
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">unsaved</p>"#);
}

/// 62k: the K6 path — a source that ships EPS figures has `\pdfoutput` 0, so IEEEtran's `\ifCLASSINFOpdf` is false and
/// the template's `\else\usepackage[dvips]{graphicx}` branch defines `\includegraphics` (0902.1911).
#[test]
fn ieeetran_eps_source_loads_dvips_graphicx() {
  let (log, xml) = convert_files_with(
    "\\documentclass{IEEEtran}\n\\ifCLASSINFOpdf\n\\else\n\\usepackage[dvips]{graphicx}\n\\fi\n\\begin{document}\n\\includegraphics{fig}\n\\end{document}\n",
    &[(
      "fig.eps",
      "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\nshowpage\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &[],
    r#"<graphics candidates="fig.eps" cssstyle="width:1.004em; height:1.004em" graphic="fig" xml:id="p1.g1"/>"#,
  );
}

/// 62k: with `localrawclasses` a shipped class still takes a binding reached by Perl's prefix alternate, or by the
/// case-insensitive or basename steps Rust adds (`class_binding_alternate`; DIVERGENCES #444): `IEEEtranTCOM.cls` and
/// `misc/ieeetran.cls` keep the IEEEtran binding and are not read (2105.02087). The stub classes error if read raw.
#[test]
fn a_shipped_class_with_an_alternate_binding_keeps_it() {
  const READ_RAW: &str =
    "\\ProvidesClass{stub}\\errmessage{the shipped class was read raw}\\LoadClass{article}\n";
  for (class, file) in [
    ("IEEEtranTCOM", "IEEEtranTCOM.cls"),
    ("misc/ieeetran", "misc/ieeetran.cls"),
  ] {
    let (log, xml) = convert_files_with(
      &format!(
        "\\documentclass{{{class}}}\n\\begin{{document}}\n\\begin{{IEEEkeywords}}\nstars\n\\end{{IEEEkeywords}}\n\\end{{document}}\n"
      ),
      &[(file, READ_RAW)],
      Some("ar5iv.sty"),
    );
    // The one warning is the alternate's notice: "Can't find binding for class … (using IEEEtran)".
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 1),
      "{class}: {log}"
    );
    assert!(log.contains("(using IEEEtran)"), "{class}: {log}");
    assert_element(
      &xml,
      "keywords",
      &[],
      "<keywords name=\"Index Terms:\u{a0}\">stars</keywords>",
    );
  }
}

/// 62k: `\@startsection` with an empty type — amsart's `\@starttoc` heading, copied into journal classes; exframe.sty:538's
/// problems, with an empty level too — is an unnumbered `subparagraph`, which closes nothing above it. Kept empty,
/// `RefStepID`'s `NewCounter` defined `\the` and `\p@` themselves, and every later `\the` printed 0: theorems lost their
/// fonts and `\lx@thistheorem` (1102.4889, 94 errors; KPE #483). pdflatex: "Contents", "Question 1. Text."; and the
/// problem heading inside section 1, section 2 after it.
#[test]
fn an_empty_section_type_defines_no_counter() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsthm}
\newtheorem{question}{Question}
\begin{document}
\makeatletter\begingroup\@startsection{}{10000}{0pt}{12pt}{6pt}{\centering\scshape}{Contents}\endgroup\makeatother
\begin{question}
Text.
\end{question}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "subparagraph",
    &[],
    r#"<subparagraph inlist="toc" xml:id="S0.SS0.SSS0.P0.SPx1"><title>Contents</title><theorem class="ltx_theorem_question" inlist="thm theorem:question" xml:id="Thmquestion1"><tags><tag>Question 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Question 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmquestion1.1">Question 1</text></tag><text font="bold" xml:id="Thmquestion1.2">.</text></title><para xml:id="Thmquestion1.p1"><p xml:id="Thmquestion1.p1.1"><text font="italic" xml:id="Thmquestion1.p1.1.1">Text.</text></p></para></theorem></subparagraph>"#,
  );
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\section{First}
A.
\makeatletter\@startsection{}{}{0pt}{0pt}{1ex}{\bfseries}*{Problem}\makeatother
B.
\section{Second}
C.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>First</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">A.</p></para><subparagraph xml:id="S1.SS0.SSS0.P0.SPx1"><title>Problem</title><para xml:id="S1.SS0.SSS0.P0.SPx1.p1"><p xml:id="S1.SS0.SSS0.P0.SPx1.p1.1">B.</p></para></subparagraph></section>"#,
  );
}

/// 62k: a raw class's `\@maketitle` deposit is speculative — it runs with the title fields emptied, outside the class's
/// own `\maketitle`, which Perl never runs — so its diagnostics are held like the class-body replay's, and a deposit
/// that errors is dropped with them. eptcs.cls:114-130's `\@maketitle` prints `\copyrightholders`, which its
/// `\maketitle` (:73-89) provides just before the call; the deposit alone raised "undefined" for page furniture
/// (1309.1271, 1405.5596).
#[test]
fn an_erroring_maketitle_deposit_is_dropped() {
  let (log, xml) = convert_files_with(
    "\\documentclass{ept}\n\\title{T}\\author{A}\n\\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    &[(
      "ept.cls",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{ept}\LoadClass{article}
\renewcommand\maketitle{\par\begingroup
  \providecommand{\copyrightholders}{\authorrunning}%
  \def\@makefnmark{\rlap{\@textsuperscript{\normalfont\@thefnmark}}}%
  \@maketitle\endgroup}
\def\@maketitle{\noindent\copyright~\copyrightholders\par{\Large\@title}\par}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("The \\@maketitle deposit was dropped"),
    "{log}"
  );
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// 62k: a class that re-lets `\@startsection` to a latex.ltx-style worker ending in `\@sect` (cup-journal.cls:1066-1076)
/// reaches the kernel dispatcher: our `\@sect` and its siblings route to `\lx@startsection`, the dispatcher under its
/// own name, so `\@sect` → the class's worker → `\@sect` no longer recurses to a Fatal (2112.11969). The one warning is
/// the worker reading the locked `\section`'s empty skip as a dimension.
#[test]
fn a_relet_startsection_reaches_the_kernel_dispatcher() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\makeatletter
\newcommand\cup@startsection[6]{%
 \if@noskipsec \leavevmode \fi
 \par \@tempskipa #4\relax
 \@afterindenttrue
 \ifdim \@tempskipa <\z@ \@tempskipa -\@tempskipa \@afterindentfalse\fi
 \if@nobreak \everypar{}\else
     \addpenalty\@secpenalty\addvspace\@tempskipa\fi
 \@ifstar{\@dblarg{\@sect{#1}{\@m}{#3}{#4}{#5}{#6}}}%
         {\@dblarg{\@sect{#1}{#2}{#3}{#4}{#5}{#6}}}}
\let\@startsection\cup@startsection
\makeatother
\begin{document}
\section{Introduction}
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title><tag close=" ">1</tag>Introduction</title>"#,
  );
}

/// 62k: the natbib binding's `\NAT@wrout` is its bibitem tag builder, locked: a raw class's redefinition — natbib's own
/// job in TeX, writing `\bibcite` to the aux (basi.cls:601) — left every bibitem without `<tags>` (1109.3388; KPE #484).
/// pdflatex: "Smith (2001).".
#[test]
fn a_raw_nat_wrout_keeps_the_bibitem_tags() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage[authoryear]{natbib}
\makeatletter
\renewcommand\NAT@wrout[5]{\if@filesw{\let\protect\noexpand\let~\relax\immediate
  \write\@auxout{\string\bibcite{#5}{{#1}{#2}{{#3}}{{#4}}}}}\fi\ignorespaces}
\makeatother
\begin{document}
\citet{k}.
\begin{thebibliography}{1}
\bibitem[Smith(2001)]{k} J. Smith, A paper, 2001.
\end{thebibliography}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tags",
    &[],
    r#"<tags><tag role="number">1</tag><tag role="year">2001</tag><tag role="authors">Smith</tag><tag role="refnum">Smith (2001)</tag><tag role="key">k</tag></tags>"#,
  );
}

/// 62l: the PASJ classes (shipped with every paper, not in TeX Live) are interpreted raw under a binding that puts the
/// kernel `\caption` back — PASJ's own calls its `\@makecaption` directly, bypassing `\@caption`, so its captions were
/// paragraphs without their number and their labels dangled (1310.7069, 1505.02769) — and routes `\KeyWords`,
/// `\altaffiltext` and the dates to the frontmatter (lost under OmniBus and raw alike). pdflatex: "Figure 1. Stars.",
/// "See Figure 1.".
#[test]
fn pasj_captions_and_frontmatter_are_semantic() {
  const PASJ01: &str = r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{pasj01}\LoadClass{article}
\def\altaffilmark#1{\textsuperscript{\normalfont#1}}
\def\altaffiltext#1#2{\protected@xdef\@affil{#2}}
\long\def\KeyWords#1{\def\@keywords{#1}}
\def\Received#1{\def\rdate{#1}}
\def\Accepted#1{\def\adate{#1}}
\def\email#1{\def\@email{#1}}
\def\caption{%
   \ifx\@captype\@undefined
      \@latex@error{\noexpand\caption outside float}\@ehd
      \expandafter\@gobble
   \else
      \expandafter\@firstofone
   \fi
   {\@ifnextchar[\@caption@with@option\@caption@without@option}}
\def\@caption@with@option[#1]{%
   \protected@edef\@currentlabel{#1}%
   \@makecaption{\csname\@captype name\endcsname~#1}}
\def\@caption@without@option{%
   \refstepcounter\@captype
   \@makecaption{\csname fnum@\@captype\endcsname}}
\long\def\@makecaption#1#2{\par\noindent #1. #2\par}
";
  let (log, xml) = convert_files_with(
    r"\documentclass{pasj01}
\title{T}
\author{A. Name\altaffilmark{1}}
\altaffiltext{1}{Observatory}
\KeyWords{stars: winds}
\begin{document}
\maketitle
\begin{figure}
\caption{Stars.}\label{f}
\end{figure}
See Figure~\ref{f}.
\end{document}",
    &[("pasj01.cls", PASJ01)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "figure",
    &[],
    r#"<figure inlist="lof" labels="LABEL:f" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Stars.</toccaption><caption><tag close=": ">Figure 1</tag>Stars.</caption></figure>"#,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A. Name</personname><contact name=\"Alternate Affiliation:\u{a0}\" \
     role=\"altaffiliation\">Observatory</contact></creator>",
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">stars: winds</keywords>",
  );
  // One `\author` lists every author; `\\` breaks its rows, `\&` precedes the last (pasj00.cls:73-104), and the template
  // puts a name's marks after its comma: each separates authors, not an affiliation line, and the marks stay with the
  // name before them (0707.3867, 0704.3654, 2504.06663).
  let (log, xml) = convert_files_with(
    r"\documentclass{pasj01}
\Received{2001 May 1}
\Accepted{2001 June 1}
\title{T}
\author{A. One,\altaffilmark{1} {\'A}. Two\altaffilmark{1} \\ C. Three,\altaffilmark{2} \& D. Four\altaffilmark{2}}
\altaffiltext{1}{First Observatory}
\altaffiltext{2}{Second Observatory}
\begin{document}
\maketitle
Text.
\end{document}",
    &[("pasj01.cls", PASJ01)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creator = |before: &str, name: &str, affiliation: &str| {
    format!(
      "<creator {before}role=\"author\"><personname>{name}</personname><contact name=\"Alternate \
       Affiliation:\u{a0}\" role=\"altaffiliation\">{affiliation}</contact></creator>"
    )
  };
  // A mark takes only its own argument: the brace group opening the next name stays with it (`{\'A}. Two`).
  assert_eq!(xml.matches("<creator ").count(), 4, "{xml}");
  let after = "before=\"\u{2003}\u{2003}\" ";
  assert_element(
    &xml,
    "creator",
    &[],
    &creator("", "A. One", "First Observatory"),
  );
  for (name, affiliation) in [
    ("Á. Two", "First Observatory"),
    ("C. Three", "Second Observatory"),
    ("D. Four", "Second Observatory"),
  ] {
    // The whole creator, flattened as `assert_element` compares (it finds only the first `<creator>`).
    let flat = latexml::util::test::normalize_markup(&xml);
    let expected = latexml::util::test::normalize_markup(&creator(after, name, affiliation));
    assert!(flat.contains(&expected), "{name}: {expected}\n{xml}");
  }
  for (role, name, date) in [
    ("received", "Received", "2001 May 1"),
    ("accepted", "Accepted", "2001 June 1"),
  ] {
    assert_element(
      &xml,
      "date",
      &[&format!("role=\"{role}\"")],
      &format!("<date name=\"{name}\u{a0}\" role=\"{role}\">{date}</date>"),
    );
  }
  // A mark takes its optional argument too: PASJ's `\thanks[<mark>]{…}` (pasj01.cls:2019) after a comma stays with the
  // name before it.
  let (log, xml) = convert_files_with(
    r"\documentclass{pasj01}
\title{T}
\author{A. One,\thanks[*]{Star note} B. Two}
\begin{document}
\maketitle
Text.
\end{document}",
    &[("pasj01.cls", PASJ01)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>A. One</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Star note</note></creator>"#,
  );
  let second = latexml::util::test::normalize_markup(
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>B. Two</personname></creator>",
  );
  assert!(
    latexml::util::test::normalize_markup(&xml).contains(&second),
    "{second}\n{xml}"
  );
}

/// 62m: the ACM SIG classes' `\alignauthor` (sig-alternate.cls, sigchi.cls), which opens each author's column, separates
/// authors as `\and` does: split before digestion, its raw definition — a tabular juggle for the class's title page —
/// never runs inside a name, and each column's `\\` lines are the name, its `\affaddr` and its `\email`. sigchi writes the
/// column as one brace group, which is read as its content. Since the arXiv profile runs these shipped classes raw
/// their authors were lost (1605.02827, 2003.09061, 1906.01122). pdflatex: two author columns.
#[test]
fn acm_alignauthor_opens_each_author() {
  let creator = |before: &str, name: &str, id: usize, affiliation: &str, email: &str| {
    format!(
      "<creator {before}role=\"author\"><personname>{name}</personname><contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">{affiliation}</contact><contact name=\"Email:\u{a0}\" role=\"email\"><text \
       font=\"typewriter\" xml:id=\"id{id}\">{email}</text></contact></creator>"
    )
  };
  for (class, source, authors) in [
    (
      "acmsig.cls",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{acmsig}
\LoadClass{article}
\def\alignauthor{\end{tabular}\hskip 1em\begin{tabular}[t]{c}}
\def\affaddr#1{{\small #1}}
\def\email#1{{\ttfamily #1}}
\def\numberofauthors#1{}
",
      r"\alignauthor Ann Alpha\\ \affaddr{Inst One}\\ \email{ann@one.org}
\alignauthor Bob Beta\\ \affaddr{Inst Two}\\ \email{bob@two.org}",
    ),
    (
      "chi.cls",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{chi}
\LoadClass{article}
\def\alignauthor#1{\end{tabular}\hskip 1em\begin{tabular}[t]{c}#1}
\def\affaddr#1{{\small #1}}
\def\email#1{{\ttfamily #1}}
",
      r"\alignauthor{Ann Alpha\\ \affaddr{Inst One}\\ \email{ann@one.org}}\\
\alignauthor{Bob Beta\\ \affaddr{Inst Two}\\ \email{bob@two.org}}",
    ),
  ] {
    let name = class.trim_end_matches(".cls");
    let (log, xml) = convert_files_with(
      &format!(
        "\\documentclass{{{name}}}\n\\title{{T}}\n\\author{{{authors}}}\n\\begin{{document}}\n\\maketitle\n\\end{{document}}\n"
      ),
      &[(class, source)],
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    assert_eq!(xml.matches("<creator ").count(), 2, "{class}: {xml}");
    assert_element(
      &xml,
      "creator",
      &[],
      &creator("", "Ann Alpha", 1, "Inst One", "ann@one.org"),
    );
    let flat = latexml::util::test::normalize_markup(&xml);
    let bob = latexml::util::test::normalize_markup(&creator(
      "before=\"\u{2003}\u{2003}\" ",
      "Bob Beta",
      2,
      "Inst Two",
      "bob@two.org",
    ));
    assert!(flat.contains(&bob), "{class}: {bob}\n{xml}");
  }
  // A brace group anywhere but after `\alignauthor`/`\affaddr` keeps its braces: one author, commas and all.
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\title{T}\n\\author{{Smith, Jr., John}}\n\\begin{document}\n\\maketitle\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Smith, Jr., John</personname></creator>"#,
  );
}

/// 62m: once the document class has loaded, the kernel makes the missing counter of its float environments (latex.ltx
/// defines neither, a class both): a raw class that makes its counter only where the environment is new — sig-alternate.cls:699
/// `\@ifundefined{figure}{\newcounter{figure}}` — saw ours and made none, so `\thefigure` was undefined at every caption
/// (1605.02827, 1607.07514; KPE #485). pdflatex: "Figure 5: Stars.", "Figure 6: Moons.", "Table 1: Planets.".
#[test]
fn a_raw_class_testing_the_figure_environment_has_its_counter() {
  let (log, xml) = convert_files_with(
    "\\documentclass{fc}\n\\begin{document}\n\\setcounter{figure}{4}\n\\begin{figure}\\caption{Stars.}\\end{figure}\n\
     \\begin{figure}\\caption{Moons.}\\end{figure}\n\\begin{table}\\caption{Planets.}\\end{table}\n\\end{document}\n",
    &[(
      "fc.cls",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{fc}
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\setlength{\textwidth}{6.5in}\setlength{\textheight}{8in}
\pagenumbering{arabic}
\@ifundefined{figure}{\newcounter{figure}}{}
\def\fps@figure{tbp}\def\ftype@figure{1}\def\ext@figure{lof}\def\fnum@figure{Figure \thefigure}
\def\figure{\@float{figure}}\def\endfigure{\end@float}
\long\def\@makecaption#1#2{#1: #2\par}
\def\fps@table{tbp}\def\ftype@table{2}\def\ext@table{lot}\def\fnum@table{Table \thetable}
\@ifundefined{table}{\newcounter{table}}{}
\def\table{\@float{table}}\def\endtable{\end@float}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // Made once the class has loaded, so the body sets it: "Figure 5", "Figure 6", "Table 1", with article's prefixes.
  let float = |kind: &str, ext: &str, tag: &str, n: &str, prefix: &str, text: &str| {
    format!(
      "<{kind} inlist=\"{ext}\" xml:id=\"section0.{prefix}{n}\"><tags><tag>{tag} {n}</tag><tag role=\"refnum\">{n}</tag><tag \
       role=\"typerefnum\">{tag} {n}</tag></tags><toccaption><tag close=\" \">{n}</tag>{text}</toccaption><caption><tag \
       close=\": \">{tag} {n}</tag>{text}</caption></{kind}>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &[],
    &float("figure", "lof", "Figure", "5", "F", "Stars."),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"section0.F6\""],
    &float("figure", "lof", "Figure", "6", "F", "Moons."),
  );
  assert_element(
    &xml,
    "table",
    &[],
    &float("table", "lot", "Table", "1", "T", "Planets."),
  );
}

/// 62n: tocloft is interpreted raw for its `\cft…` parameters, with the kernel's lists put back where it puts its
/// `\tableofcontents`, `\listoffigures` and `\listoftables` (tocloft.sty:118-140, 536-538, 638-640), which run `\@starttoc` and
/// read a `.toc` LaTeXML never writes: only their heading was left, the `<TOC>` lost (SciPost.cls loads it; 1811.09408).
/// A document's `\cft…` settings change nothing here. pdflatex: the contents and figure lists.
#[test]
fn tocloft_keeps_the_kernel_lists() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{tocloft}
\renewcommand{\cftsecleader}{\cftdotfill{\cftdotsep}}
\setlength{\cftbeforesecskip}{2pt}
\begin{document}
\tableofcontents
\listoffigures
\section{A}x
\begin{figure}\caption{F}\end{figure}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "TOC",
    &[],
    r#"<TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Contents</title></TOC>"#,
  );
  assert_element(
    &xml,
    "TOC",
    &["lists=\"lof\""],
    r#"<TOC lists="lof" scope="global"><title>List of Figures</title></TOC>"#,
  );
}

/// 62n: the kernel's lists come back under tocloft's own condition and at its own times, so what TeX prints stays: with
/// `titles` tocloft leaves the lists alone, and a document's own `\tableofcontents` (or a patch of the kernel's) stands;
/// a `\renewcommand` in a begin-document hook added after tocloft runs after the restore and wins (minitoc's wrapper
/// shape). pdflatex: "Overview …"; "This report has no tables."; "Read this first." before the contents.
#[test]
fn tocloft_restores_where_tocloft_replaces() {
  let toc = r#"<TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Contents</title></TOC>"#;
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage[titles]{tocloft}
\renewcommand{\tableofcontents}{\section*{Overview}Overview paragraph text.}
\begin{document}
\tableofcontents
\section{Alpha}x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<TOC"), "{xml}");
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section xml:id="Sx1"><title>Overview</title><para xml:id="Sx1.p1"><p xml:id="Sx1.p1.1">Overview paragraph text.</p></para></section>"#,
  );
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{tocloft}
\AtBeginDocument{\renewcommand{\listoftables}{\section*{No tables}This report has no tables.}}
\begin{document}
\tableofcontents
\listoftables
\section{Alpha}x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "TOC", &[], toc);
  assert!(!xml.contains("lists=\"lot\""), "{xml}");
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section xml:id="Sx1"><title>No tables</title><para xml:id="Sx1.p1"><p xml:id="Sx1.p1.1">This report has no tables.</p></para></section>"#,
  );
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage[titles]{tocloft}
\usepackage{etoolbox}
\pretocmd{\tableofcontents}{\noindent Read this first.\par}{}{}
\begin{document}
\tableofcontents
\section{Alpha}x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "TOC", &[], toc);
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para class="ltx_noindent" xml:id="p1"><p xml:id="p1.1">Read this first.</p></para>"#,
  );
  // A preamble redefinition is overridden by tocloft's hook, so by the restore after it (TeX prints tocloft's list);
  // a later package's begin-document hook runs after the restore, filed under tocloft's own label, and wins.
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\usepackage{tocloft}
\usepackage{zlatehook}
\renewcommand{\listoffigures}{\section*{My figures}Custom LOF text.}
\begin{document}
\tableofcontents
\listoffigures
\listoftables
\section{Alpha}x
\end{document}",
    &[(
      "zlatehook.sty",
      r"\ProvidesPackage{zlatehook}
\AtBeginDocument{\renewcommand{\listoftables}{\section*{Late pkg tables}Late package text.}}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("Custom LOF text"), "{xml}");
  assert_element(
    &xml,
    "TOC",
    &["lists=\"lof\""],
    r#"<TOC lists="lof" scope="global"><title>List of Figures</title></TOC>"#,
  );
  assert!(!xml.contains("lists=\"lot\""), "{xml}");
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section xml:id="Sx1"><title>Late pkg tables</title><para xml:id="Sx1.p1"><p xml:id="Sx1.p1.1">Late package text.</p></para></section>"#,
  );
}

/// 62o: a minipage captioned as a figure inside a tabular cell is a figure panel in the cell — an inline logical block
/// (whose model holds a figure; a cell holds none) around a `figure class="ltx_figure_panel"` with its caption — as the
/// same minipage between paragraphs is a panel. The box placement climbed out of the cell to put the caption in the
/// figure and moved the insertion point past the `<td>` the alignment still owed: 5 malformed errors, both captions in
/// one figure (1601.03744; KPE #486). pdflatex: "Figure 1: Left.", "Figure 2: Right.".
#[test]
fn captions_in_minipages_in_a_tabular_are_panels() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\begin{figure*}
\begin{tabular}{cc}
\begin{minipage}{0.4\textwidth}\caption{Left.}\label{a}\end{minipage} &
\begin{minipage}{0.4\textwidth}\caption{Right.}\label{b}\end{minipage}
\end{tabular}
\end{figure*}
See \ref{a} and \ref{b}.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr xml:id="fig1.1.1"><td align="center" xml:id="fig1.1.1.1"><inline-logical-block class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="fig1.1.1.1.1"><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:a" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Left.</toccaption><caption><tag close=": ">Figure 1</tag>Left.</caption></figure></inline-logical-block></td><td align="center" xml:id="fig1.1.1.2"><inline-logical-block class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="fig1.1.1.2.1"><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:b" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">2</tag>Right.</toccaption><caption><tag close=": ">Figure 2</tag>Right.</caption></figure></inline-logical-block></td></tr>"#,
  );
  // The panel keeps the box's content in source order, the caption between the text, or first.
  for (body, panel) in [
    (
      r"Above.\caption{Mid.}Below.",
      r#"<figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p class="ltx_figure_panel" xml:id="S0.F1.1">Above.</p><toccaption><tag close=" ">1</tag>Mid.</toccaption><caption><tag close=": ">Figure 1</tag>Mid.</caption><p class="ltx_figure_panel" xml:id="S0.F1.2">Below.</p></figure>"#,
    ),
    (
      r"\caption{Top caption.}Body below caption.",
      r#"<figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Top caption.</toccaption><caption><tag close=": ">Figure 1</tag>Top caption.</caption><p xml:id="S0.F1.1">Body below caption.</p></figure>"#,
    ),
  ] {
    let (log, xml) = convert_with(
      &format!(
        "\\documentclass{{article}}\n\\begin{{document}}\n\\begin{{figure}}\n\\begin{{tabular}}{{c}}\n\
         \\begin{{minipage}}{{0.4\\textwidth}}{body}\\end{{minipage}}\n\\end{{tabular}}\n\\end{{figure}}\n\\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{body}: {log}"
    );
    assert_element(&xml, "figure", &["class=\"ltx_figure_panel\""], panel);
  }
  // A math `array` is an alignment too: the caption stays in its cell, a panel in the cell's text.
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\begin{figure}
$\begin{array}{cc}
\begin{minipage}{0.3\textwidth}\caption{Arr.}\end{minipage} & y \\
z & w
\end{array}$
\end{figure}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "XMCell",
    &[],
    r#"<XMCell align="center" xml:id="S0.F1.m1.1a.1.1.1"><XMText xml:id="S0.F1.m1.1a.1.1.1.1"><inline-logical-block class="ltx_minipage" vattach="middle" width="103.5pt" xml:id="S0.F1.m1.1a.1.1.1.1.1"><figure class="ltx_figure_panel" xml:id="S0.F1.m1.1"><toccaption><tag close=" ">1</tag>Arr.</toccaption><caption><tag close=": ">Figure 1</tag>Arr.</caption></figure></inline-logical-block></XMText></XMCell>"#,
  );
}

/// 62p: graphicx's pdftex driver loads epstopdf-base at `\begin{document}` (pdftex.def:681-701), which requires
/// pdftexcmds, and that iftex, when `\@curroptions` is not empty (epstopdf-base.sty:151-182) — here graphicx's
/// `pdftex`; iftex defines `\ifpdf` afresh. A paper's `\let\ifpdf\relax` before it held in Rust only, and JINST's
/// `\label` (reduced: `\iftrue\ifpdf…\else…\fi\fi`) left a stray `\fi` that closed the caption's hack: a Fatal, no
/// output (1310.6454; KPE #487). pdflatex: "Figure 1: Second.", "See 1.".
#[test]
fn graphics_pdftex_driver_chain_restores_ifpdf() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\newif\ifpdf \pdftrue
\let\ifpdf\relax
\usepackage[pdftex]{graphicx}
\makeatletter
\newcommand{\name}[1]{{\iftrue\ifpdf\pdfdest name{#1} fith\else\special{html:x}\fi\fi}}
\let\old@label\label
\def\label#1{\name{ref-#1}\old@label{#1}}
\makeatother
\begin{document}
\begin{figure}
\caption{Second.}
\label{MCcut}
\end{figure}
See \ref{MCcut}.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "caption",
    &[],
    r#"<caption><tag close=": ">Figure 1</tag>Second.</caption>"#,
  );
}

/// 62p: the chain follows the last options processing, as in TeX (pdflatex, each case): an option-less graphicx leaves
/// `\@curroptions` empty, so epstopdf-base loads no pdftexcmds and a paper's own `\ifxetex` stands; an explicit `dvips`
/// driver has no epstopdf load at all; amsmath leaves amsopn's `namelimits` (amsmath.sty:49-51, 91-92), so iftex loads
/// and defines `\ifxetex` (false) afresh; a key=value processor leaves `\@curroptions` as it was — caption's
/// `font=small` adds nothing (caption3.sty:399), xcolor keeps fontenc's `T1` (latex.ltx:19379).
#[test]
fn the_pdftex_driver_chain_follows_the_last_options() {
  for (packages, result) in [
    (r"\usepackage{graphicx}", "R:XE; NOIFTEX; NOPTC."),
    (
      r"\usepackage[dvips]{graphicx}\usepackage[T1]{fontenc}",
      "R:XE; NOIFTEX; NOPTC.",
    ),
    (
      r"\usepackage{graphicx}\usepackage{amsmath}",
      "R:NOXE; IFTEX; PTC.",
    ),
    (
      r"\usepackage{graphicx}\usepackage[font=small]{caption}",
      "R:XE; NOIFTEX; NOPTC.",
    ),
    (
      r"\usepackage{graphicx}\usepackage[T1]{fontenc}\usepackage{xcolor}",
      "R:NOXE; IFTEX; PTC.",
    ),
  ] {
    let tex = format!(
      r"\documentclass{{article}}
\newif\ifxetex \xetextrue
{packages}
\begin{{document}}
\makeatletter
R:\ifxetex XE\else NOXE\fi; \@ifpackageloaded{{iftex}}{{IFTEX}}{{NOIFTEX}}; \@ifpackageloaded{{pdftexcmds}}{{PTC}}{{NOPTC}}.
\makeatother
\end{{document}}"
    );
    assert_elements(&tex, "ar5iv.sty", (0, 0), &[(
      "p",
      "p1.1",
      &format!(r#"<p xml:id="p1.1">{result}</p>"#),
    )]);
  }
}

/// 62q: a captioned minipage in a float is the figure its caption numbers. The box is a group in LaTeX
/// (`\@iiiminipage`), and the `\@currentlabel` its `\caption` sets is what a `\label` after it names, so two such
/// minipages side by side are Figures 1 and 2, each `\ref` its own number (pdflatex: "See 1, 2, 3."). The float took the
/// last caption's counters and every `\label` in it, so each `\ref` read the last number — SHARED with Perl, whose
/// `insertBlock` captures the box into an id-less block that `floatToLabel` climbs past (TeX_Box.pool.ltxml:449-519,
/// Document.pm:1098-1127). Each panel now holds its caption's id, tags and label; the float keeps a caption of its own;
/// a table's captioned minipages are tables (`insert_block` never picks one). DIVERGENCES #447.
#[test]
fn captioned_minipages_are_their_own_floats() {
  let doc = |body: &str| {
    format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n{body}\nSee \\ref{{a}}, \\ref{{b}}, \\ref{{c}}.\n\\end{{document}}"
    )
  };
  let left = r"\begin{minipage}{0.4\textwidth}X\caption{Left.}\label{a}\end{minipage}\hfill";
  let right = r"\begin{minipage}{0.4\textwidth}Y\caption{Right.}\label{b}\end{minipage}";
  assert_elements(
    &doc(&format!(r"\begin{{figure}}{left}{right}\end{{figure}}")),
    "ar5iv.sty",
    (0, 0),
    &[(
      "figure",
      "fig1",
      r#"<figure xml:id="fig1"><figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:a" vattach="middle" width="138.0pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.1">X</p><toccaption><tag close=" ">1</tag>Left.</toccaption><caption><tag close=": ">Figure 1</tag>Left.</caption></figure><figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:b" vattach="middle" width="138.0pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p xml:id="S0.F2.1">Y</p><toccaption><tag close=" ">2</tag>Right.</toccaption><caption><tag close=": ">Figure 2</tag>Right.</caption></figure></figure>"#,
    )],
  );
  assert_elements(
    &doc(&format!(
      r"\begin{{figure}}{left}{right}\caption{{Overall.}}\label{{c}}\end{{figure}}"
    )),
    "ar5iv.sty",
    (0, 0),
    &[(
      "figure",
      "S0.F3",
      r#"<figure inlist="lof" labels="LABEL:c" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:a" vattach="middle" width="138.0pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.1">X</p><toccaption><tag close=" ">1</tag>Left.</toccaption><caption><tag close=": ">Figure 1</tag>Left.</caption></figure><figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:b" vattach="middle" width="138.0pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p xml:id="S0.F2.1">Y</p><toccaption><tag close=" ">2</tag>Right.</toccaption><caption><tag close=": ">Figure 2</tag>Right.</caption></figure><toccaption><tag close=" ">3</tag>Overall.</toccaption><caption><tag close=": ">Figure 3</tag>Overall.</caption></figure>"#,
    )],
  );
  assert_elements(
    &doc(
      r"\begin{table}\begin{minipage}{0.4\textwidth}\caption{TL.}\label{a}\begin{tabular}{c}1\end{tabular}\end{minipage}\hfill
\begin{minipage}{0.4\textwidth}\caption{TR.}\label{b}\begin{tabular}{c}2\end{tabular}\end{minipage}\end{table}",
    ),
    "ar5iv.sty",
    (0, 0),
    &[(
      "table",
      "tab1",
      r#"<table xml:id="tab1"><table class="ltx_figure_panel ltx_minipage" inlist="lot" labels="LABEL:a" vattach="middle" width="138.0pt" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>TL.</toccaption><caption><tag close=": ">Table 1</tag>TL.</caption><tabular vattach="middle" xml:id="S0.T1.1"><tbody><tr xml:id="S0.T1.1.1"><td align="center" xml:id="S0.T1.1.1.1">1</td></tr></tbody></tabular></table><table class="ltx_figure_panel ltx_minipage" inlist="lot" labels="LABEL:b" vattach="middle" width="138.0pt" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>TR.</toccaption><caption><tag close=": ">Table 2</tag>TR.</caption><tabular vattach="middle" xml:id="S0.T2.1"><tbody><tr xml:id="S0.T2.1.1"><td align="center" xml:id="S0.T2.1.1.1">2</td></tr></tbody></tabular></table></table>"#,
    )],
  );
  // A box around the minipage (`\fbox`, `lrbox` + `\usebox`) makes no panel of it: the float keeps the number and
  // the `\label`, formed under the id the float holds for the caption (`insert_block_in_paragraph`), never the
  // document's (62q review: `\fbox`, `\colorbox`, adjustbox, tcolorbox, `lrbox` labelled `<document>`).
  let xml = assert_elements(
    r"\documentclass{article}
\begin{document}
\begin{figure}\fbox{\begin{minipage}{0.4\textwidth}X\caption{Boxed.}\label{fb}\end{minipage}}\end{figure}
\newsavebox{\lr}
\begin{figure}\begin{lrbox}{\lr}\begin{minipage}{0.4\textwidth}Y\caption{Saved.}\label{lr}\end{minipage}\end{lrbox}\usebox{\lr}\end{figure}
See \ref{fb} and \ref{lr}.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "figure",
        "S0.F1",
        r##"<figure inlist="lof" labels="LABEL:fb" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.1"><text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle" xml:id="S0.F1.1.1"><inline-block class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.F1.1.1.1"><p xml:id="S0.F1.1.1.1.1">X</p></inline-block></text></p><toccaption><tag close=" ">1</tag>Boxed.</toccaption><caption><tag close=": ">Figure 1</tag>Boxed.</caption></figure>"##,
      ),
      (
        "figure",
        "S0.F2",
        r#"<figure inlist="lof" labels="LABEL:lr" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p xml:id="S0.F2.1"><text xml:id="S0.F2.1.1"><inline-block class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.F2.1.1.1"><p xml:id="S0.F2.1.1.1.1">Y</p></inline-block></text></p><toccaption><tag close=" ">2</tag>Saved.</toccaption><caption><tag close=": ">Figure 2</tag>Saved.</caption></figure>"#,
      ),
    ],
  );
  let document_tag = &xml[xml.find("<document").unwrap()..];
  let document_tag = &document_tag[..document_tag.find('>').unwrap()];
  assert!(!document_tag.contains("labels="), "{document_tag}");
}

/// 62r: textcomp's symbols are TS1 text symbols, each its encoding's dispatcher (`declare_bound_text_symbols`), so an
/// encoding declared later prints its own: LGR's `\textmu` is μ (lgrenc.def:184), where the fixed TS1 primitive
/// printed µ — in Greek text, babel greek's `\figurename` and lgrenc.dfu's UTF-8 μ (KPE #489; Perl alike,
/// textcomp.sty.ltxml:152). A document's `\DeclareTextCommand`/`\ProvideTextCommand` for one encoding takes effect
/// there and nowhere else. pdflatex, each case.
#[test]
fn textcomp_symbols_follow_the_encoding() {
  for (preamble, body, result) in [
    (
      r"\usepackage[LGR,T1]{fontenc}\usepackage[english,greek]{babel}",
      r"μα \textmu{} m \figurename",
      "μα μ μ Σχ\u{1f75}μα",
    ),
    (
      r"\usepackage[LGR,T1]{fontenc}",
      r"A:\textmu{} B:{\fontencoding{TS1}\selectfont\textmu} C:{\fontencoding{LGR}\selectfont\textmu} D:{\fontencoding{OT1}\selectfont\textmu}",
      "A:µ B:µ C:μ D:µ",
    ),
    (
      r"\usepackage[T1]{fontenc}\DeclareTextCommandDefault{\foo}{D}\DeclareTextCommand{\foo}{T1}{T}\DeclareTextCommand{\textmu}{T1}{MU}",
      r"A:\foo{} B:{\fontencoding{OT1}\selectfont\foo} C:\textmu{} E:{\fontencoding{OT1}\selectfont\textmu}",
      "A:T B:D C:MU E:µ",
    ),
    (
      r"\usepackage[LGR,T1]{fontenc}\ProvideTextCommand{\textmu}{LGR}{X}\ProvideTextCommandDefault{\textmu}{Z}\ProvideTextCommand{\textdegree}{LGR}{DEG}",
      r"A:{\fontencoding{LGR}\selectfont\textmu} B:\textmu{} C:{\fontencoding{OT1}\selectfont\textmu} D:\textdegree{} E:{\fontencoding{LGR}\selectfont\textdegree}",
      "A:μ B:µ C:µ D:° E:ΔΕΓ",
    ),
    // An encoding file's own construction of a bound symbol is the binding's character (62r review): t2aenc.def:70
    // builds ‰ as `\%\char 24`, ot4enc.def:106 £ as an italic `$`; OT1's dumped `\textsterling` is the same `$`;
    // LY1's slot 1 is € and 143 − (texnansi.enc).
    (
      r"\usepackage[T2A,OT4,LY1,T1]{fontenc}",
      r"A:{\fontencoding{T2A}\selectfont\textperthousand} B:{\fontencoding{OT4}\selectfont\textsterling} C:{\fontencoding{OT1}\selectfont\textsterling} D:{\fontencoding{LY1}\selectfont\texteuro\textminus}",
      "A:‰ B:£ C:£ D:€−",
    ),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\n{body}\n\\end{{document}}"
    );
    assert_elements(&tex, "ar5iv.sty", (0, 0), &[(
      "p",
      "p1.1",
      &format!(r#"<p xml:id="p1.1">{result}</p>"#),
    )]);
  }
}

/// 62s: the 2609 templates' new author and title macros, each from its template's own source (2609 cluster study):
/// neurips_2026.sty:81-82 `\workshoptitle` (113 papers), acmart.cls:1683 `\correspondingauthor` (76, an envelope
/// contact), aa.cls:812 `\corrauth` (58), spconf.sty:181 `\sthanks` (45), revtex4-1.cls:2147-2250's `\move@AU`,
/// `\move@AF`, `\@affiliation` under a class's own `\affiliation` (openjournal.cls:433, 33), wacv.sty:497's
/// `\thetitle` (21, the title's copy after `\maketitle`; the same wrapper once when cvpr and wacv both load),
/// natbib.sty:154 `\@ifxundefined` (iau.cls:4176, 24), neurips' `preprint` option and its
/// tracks' `\@trackname` with the year's own ordinal, `\@noticestring` (2609.00038), acmart's `balance`
/// options, aa.cls:569 `\aa@emailfont` (2609.17322), newtxmath.sty:2160 `\upmu` (2609.07528).
#[test]
fn templates_of_2609_keep_their_author_and_title_macros() {
  let cases: [(&str, &str, &str, &str); 13] = [
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage[preprint]{neurips_2026}\workshoptitle{Agents}\title{T}
\begin{document}\maketitle
W:[\makeatletter\@workshoptitle\makeatother]
\end{document}",
      r#"<p xml:id="p1.1">W:[Agents]</p>"#,
    ),
    (
      "creator",
      "",
      r"\documentclass[sigconf]{acmart}\begin{document}\title{T}\author{Ann Lee}\correspondingauthor\maketitle
Body.
\end{document}",
      "<creator role=\"author\"><personname>Ann Lee</personname><contact role=\"corresponding\">\u{2709}</contact></creator>",
    ),
    (
      "creator",
      "",
      r"\documentclass{aa}\begin{document}\title{T}\author{Ann Lee\inst{1}\corrauth{ann@x.org}}\institute{Inst}\maketitle
Body.
\end{document}",
      r#"<creator role="author"><personname>Ann Lee</personname><contact name="Corresponding author: " role="corresponding">ann@x.org</contact><contact name="Affiliation: " role="affiliation">Inst</contact></creator>"#,
    ),
    (
      "creator",
      "",
      r"\documentclass{article}\usepackage{spconf}\title{T}\name{Ann Lee\sthanks{Corresponding author.}}\address{Inst}
\begin{document}\maketitle
Body.
\end{document}",
      r#"<creator role="author"><personname>Ann Lee</personname><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">Corresponding author.</note></creator>"#,
    ),
    (
      "creator",
      "",
      r"\documentclass{revtex4-1}\makeatletter
\renewcommand\affiliation[1]{\move@AU\move@AF\begingroup\@affiliation{#1}}
\makeatother
\begin{document}\title{T}\author{Ann Lee}\affiliation{Perimeter Institute}\maketitle
Body.
\end{document}",
      "<creator role=\"author\"><personname>Ann Lee</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Perimeter Institute</contact></creator>",
    ),
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage{wacv}\title{My Title}
\begin{document}\maketitle
T:[\thetitle]
\end{document}",
      r#"<p xml:id="p1.1">T:[My Title]</p>"#,
    ),
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage{natbib}
\begin{document}\makeatletter
R:\@ifxundefined\NAT@sectionbib{X}{Y}
\makeatother
\end{document}",
      r#"<p xml:id="p1.1">R:X</p>"#,
    ),
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage{cvpr}\usepackage{wacv}\title{Twice Loaded}
\begin{document}\maketitle
T:[\thetitle]
\end{document}",
      r#"<p xml:id="p1.1">T:[Twice Loaded]</p>"#,
    ),
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage[preprint,position]{neurips_2026}
\begin{document}\makeatletter
A:\if@preprint P\else\if@neuripsfinal F\else S\fi\fi\if@anonymous Y\else N\fi [\@trackname]
\makeatother
\end{document}",
      r#"<p xml:id="p1.1">A:PN[40th Conference on Neural Information Processing Systems (NeurIPS 2026). Position Paper Track.]</p>"#,
    ),
    (
      "creator",
      "",
      r"\documentclass{aa}\makeatletter
\renewcommand*{\corrauth}[1]{\thanks{Corresponding author: {\aa@emailfont #1}}}
\makeatother
\begin{document}\title{T}\author{Ann Lee\corrauth{ann@x.org}}\institute{Inst}\maketitle
Body.
\end{document}",
      r#"<creator role="author"><personname>Ann Lee</personname><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">Corresponding author: <text font="typewriter" xml:id="id1.1">ann@x.org</text></note></creator>"#,
    ),
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage[dblblindworkshop,final]{neurips_2026}\workshoptitle{Agents in the Wild}
\begin{document}\makeatletter
N:[\@noticestring]
\makeatother
\end{document}",
      r#"<p xml:id="p1.1">N:[40th Conference on Neural Information Processing Systems (NeurIPS 2026). Workshop: Agents in the Wild.]</p>"#,
    ),
    (
      "p",
      "p1.1",
      r"\documentclass[sigconf,balance=false,pbalance]{acmart}
\begin{document}\title{T}\author{A}\maketitle
\makeatletter B:\if@ACM@balance Y\else N\fi\if@ACM@pbalance Y\else N\fi\makeatother
\end{document}",
      r#"<p xml:id="p1.1">B:NY</p>"#,
    ),
    (
      "p",
      "p1.1",
      r"\documentclass{article}\usepackage{newtxmath}
\begin{document}
$\upmu\upGamma$
\end{document}",
      "<p xml:id=\"p1.1\"><Math mode=\"inline\" tex=\"\\upmu\\upGamma\" text=\"upmu * upGamma\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok name=\"upmu\" role=\"UNKNOWN\">\u{3bc}</XMTok><XMTok name=\"upGamma\" role=\"UNKNOWN\">\u{393}</XMTok></XMApp></XMath></Math></p>",
    ),
  ];
  for (tag, id, tex, element) in cases {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62t: a caption opening with a conditional keeps its text (REGRESSION since 62b: `\@dblarg` makes the caption its own
/// `[short]`, and `\@caption@@@`'s `\ifx.#2.#3\else#2\fi` took the conditional as its second token — "Extra \else", then
/// Fatal at the end of the file scanning for `\endcaption`; 2609.27590, 2609.28438; KNOWN_PERL_ERRORS #468), with `[]`
/// still listing the caption; and the 2609 classes' own commands: llncs.cls:909-911's `\doi`, a link under hyperref
/// (2609.04690), sagej.cls:271's `\affilnum` (2609.04585), the appendices' `\Roman` numbering of ieeeconf.cls:4096-4118
/// (`\ifuseRomanappendices`, 2609.16300) and IEEEtran.cls:5752-5775 (`romanappendices`; KNOWN_PERL_ERRORS #491), a
/// subfigure caption opening with a conditional (#468), and acronym's plural capitalized forms and `\acfip` (#490).
#[test]
fn captions_and_2609_class_commands_keep_their_text() {
  let appendices = |preamble: &str| {
    format!(
      "{preamble}\n\\begin{{document}}\n\\section{{Intro}}\nA.\n\\appendices\n\\section{{Proof}}\nB.\n\\end{{document}}"
    )
  };
  let cases: Vec<(&str, &str, String, &str)> = vec![
    (
      "caption",
      "",
      String::from(
        r"\documentclass{article}\begin{document}
\begin{table}\caption{\ifdefined\undefinedthing X\fi A.}\end{table}
\end{document}",
      ),
      r#"<caption><tag close=": ">Table 1</tag>A.</caption>"#,
    ),
    (
      "toccaption",
      "",
      String::from(
        r"\documentclass{article}\begin{document}
\begin{figure}\caption[]{\iftrue Y\fi B.}\end{figure}
\end{document}",
      ),
      r#"<toccaption><tag close=" ">1</tag>YB.</toccaption>"#,
    ),
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{llncs}\begin{document}
D:\doi{10.1007/978-3-030}
\end{document}",
      ),
      r#"<p xml:id="p1.1">D:https://doi.org/10.1007/978-3-030</p>"#,
    ),
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{llncs}\usepackage{hyperref}\begin{document}
D:\doi{10.1007/978-3-030}
\end{document}",
      ),
      r#"<p xml:id="p1.1">D:<ref class="ltx_url" font="typewriter" href="https://doi.org/10.1007/978-3-030">https://doi.org/10.1007/978-3-030</ref></p>"#,
    ),
    (
      "personname",
      "",
      String::from(
        r"\documentclass{sagej}\begin{document}\title{T}\author{Ann Lee\affilnum{1}}\affiliation{\affilnum{1}Inst}
\maketitle
Body.
\end{document}",
      ),
      r#"<personname>Ann Lee<sup xml:id="id1">1</sup></personname>"#,
    ),
    (
      "appendix",
      "A1",
      appendices(r"\documentclass{ieeeconf}"),
      r#"<appendix inlist="toc" xml:id="A1"><tags><tag>Appendix I</tag><tag role="refnum">I</tag><tag role="typerefnum">Appendix I</tag></tags><title><tag close=" ">Appendix I</tag>Proof</title><toctitle><tag close=" ">I</tag>Proof</toctitle><para xml:id="A1.p1"><p xml:id="A1.p1.1">B.</p></para></appendix>"#,
    ),
    (
      "appendix",
      "A1",
      appendices(r"\documentclass{ieeeconf}\useRomanappendicesfalse"),
      r#"<appendix inlist="toc" xml:id="A1"><tags><tag>Appendix A</tag><tag role="refnum">A</tag><tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix A</tag>Proof</title><toctitle><tag close=" ">A</tag>Proof</toctitle><para xml:id="A1.p1"><p xml:id="A1.p1.1">B.</p></para></appendix>"#,
    ),
    (
      "appendix",
      "A1",
      appendices(r"\documentclass[romanappendices]{IEEEtran}"),
      r#"<appendix inlist="toc" xml:id="A1"><tags><tag>Appendix I</tag><tag role="refnum">I</tag><tag role="typerefnum">Appendix I</tag></tags><title><tag close=" ">Appendix I</tag>Proof</title><toctitle><tag close=" ">I</tag>Proof</toctitle><para xml:id="A1.p1"><p xml:id="A1.p1.1">B.</p></para></appendix>"#,
    ),
    (
      "caption",
      "",
      String::from(
        r"\documentclass{article}\usepackage{subfigure}\begin{document}
\begin{figure}\subfigure[\ifdefined\undefinedthing A\fi B]{X}\end{figure}
\end{document}",
      ),
      r#"<caption><tag close=" "><text fontsize="80%" xml:id="S0.F1.sf1.3">(a)</text></tag><text fontsize="80%" xml:id="S0.F1.sf1.4">B</text></caption>"#,
    ),
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{article}\usepackage{acronym}\begin{document}
\begin{acronym}\acro{CNN}{convolutional network}\end{acronym}
I:\Acfip{CNN}.
\end{document}",
      ),
      r#"<p xml:id="p1.1">I:<glossaryref inlist="acronym" key="CNN" show="long-plural"/><text font="italic" xml:id="p1.1.1"> </text>(<glossaryref inlist="acronym" key="CNN" show="short-plural"/>).</p>"#,
    ),
    // (`S: ` — the space is Perl's `\@acfp`, acronym.sty.ltxml:133, which acronym.sty:677 does not have.)
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{article}\usepackage{acronym}\begin{document}
\begin{acronym}\acro{CNN}{convolutional network}\end{acronym}
P:\Acp{CNN}; Q:\Aclp{CNN}; S:\Acfp{CNN}.
\end{document}",
      ),
      r#"<p xml:id="p1.1">P:<glossaryref inlist="acronym" key="CNN" show="long-plural"/>; Q:<glossaryref inlist="acronym" key="CNN" show="long-plural"/>; S: <glossaryref inlist="acronym" key="CNN" show="long-plural"/> (<glossaryref inlist="acronym" key="CNN" show="short-plural"/>).</p>"#,
    ),
  ];
  for (tag, id, tex, element) in cases {
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
      assert_eq!(lines, 0, "jing:\n{xml}");
    }
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62x: AASTeX's table columns (aas_support_sty.rs). A `D` column is two columns (aastex701.cls TL :12010; Perl
/// aas_support.sty.ltxml:353-372, whose `\lx@alignment@align` no Perl file defines: KNOWN_PERL_ERRORS #493): after
/// `\decimals` a cell's first word splits at its first `.` (`\lookfordecimal`, :11980), both parts in math, the point
/// only before a fraction and everything after it kept; without `\decimals` — which each deluxetable resets, :11330 —
/// the cell stays whole. `C`/`L`/`R` are math
/// cells (:8857-8859) in which `$` — active in every table, :8849 — does nothing, as in the class's `\nodata`
/// (:8268-8269), while it still shifts to math in an ordinary column, in a plain `tabular`, and after the table. Witness
/// 2609.05675 (`{llDDDCLll}`: 96 "Extra alignment tab" then Fatal TooManyErrors, now 0 errors), 2609.00308, 2609.06985.
#[test]
fn aastex_decimal_and_math_columns() {
  let decimal = r"\documentclass{aastex701}\begin{document}
\begin{deluxetable}{lDl}
\tablehead{\colhead{Name} & \twocolhead{Value} & \colhead{Note}}
\decimals
\startdata
A & 12.345 & x \\
B & 7 & y \\
C & 1.2.3 & z \\
D & . & w \\
E & -1.25 & v \\
F & 12. & u \\
\enddata
\end{deluxetable}
\end{document}";
  let undecimal = r"\documentclass{aastex701}\begin{document}
\begin{deluxetable}{lD}
\tablehead{\colhead{Name} & \twocolhead{Value}}
\startdata
A & 12.3 \\
\enddata
\end{deluxetable}
\end{document}";
  let math = r"\documentclass{aastex701}\begin{document}
\begin{deluxetable}{lCD}
\tablehead{\colhead{Name} & \colhead{$D$} & \twocolhead{Value}}
\decimals
\startdata
A & 69.0 \pm 3.2 & 12.345 \\
B & 284.5^{\bf *} & 7 \\
C & $z_0$ & 1.5 \\
\enddata
\end{deluxetable}
\end{document}";
  let two_tables = r"\documentclass{aastex701}\begin{document}
\begin{deluxetable}{lD}
\tablehead{\colhead{Name} & \twocolhead{Value}}
\decimals
\startdata
A & 12.345 \\
\enddata
\end{deluxetable}
\begin{deluxetable}{lD}
\tablehead{\colhead{Name} & \twocolhead{Value}}
\startdata
B & 7.25 \\
\enddata
\end{deluxetable}
\end{document}";
  let plain = r"\documentclass{aastex701}\begin{document}
\begin{deluxetable}{lc}
\tablehead{\colhead{Name} & \colhead{$D$}}
\startdata
A & $x^2$ and text \\
\enddata
\end{deluxetable}
\begin{tabular}{lCR}
a & $b_1$ & \nodata \\
\end{tabular}
After: $y_1$ and \$5.
\end{document}";
  let cases: [(&str, &str, &str, &str); 8] = [
    (
      decimal,
      "tbody",
      "",
      "<tbody><tr xml:id=\"tab1.1.2\"><td align=\"left\" border=\"t\" xml:id=\"tab1.1.2.1\">A</td><td align=\"right\" border=\"t\" class=\"ltx_norightpad\" xml:id=\"tab1.1.2.2\"><Math mode=\"inline\" tex=\"12\" text=\"12\" xml:id=\"m1\"><XMath xml:id=\"m1.1\"><XMTok meaning=\"12\" role=\"NUMBER\">12</XMTok></XMath></Math></td><td align=\"left\" border=\"t\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.2.3\">.<Math mode=\"inline\" tex=\"345\" text=\"345\" xml:id=\"m2\"><XMath xml:id=\"m2.1\"><XMTok meaning=\"345\" role=\"NUMBER\">345</XMTok></XMath></Math></td><td align=\"left\" border=\"t\" xml:id=\"tab1.1.2.4\">x</td></tr><tr xml:id=\"tab1.1.3\"><td align=\"left\" xml:id=\"tab1.1.3.1\">B</td><td align=\"right\" class=\"ltx_norightpad\" xml:id=\"tab1.1.3.2\"><Math mode=\"inline\" tex=\"7\" text=\"7\" xml:id=\"m3\"><XMath xml:id=\"m3.1\"><XMTok meaning=\"7\" role=\"NUMBER\">7</XMTok></XMath></Math></td><td align=\"left\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.3.3\"/><td align=\"left\" xml:id=\"tab1.1.3.4\">y</td></tr><tr xml:id=\"tab1.1.4\"><td align=\"left\" xml:id=\"tab1.1.4.1\">C</td><td align=\"right\" class=\"ltx_norightpad\" xml:id=\"tab1.1.4.2\"><Math mode=\"inline\" tex=\"1\" text=\"1\" xml:id=\"m4\"><XMath xml:id=\"m4.1\"><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMath></Math></td><td align=\"left\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.4.3\">.<Math mode=\"inline\" tex=\"2.3\" text=\"2.3\" xml:id=\"m5\"><XMath xml:id=\"m5.1\"><XMTok meaning=\"2.3\" role=\"NUMBER\">2.3</XMTok></XMath></Math></td><td align=\"left\" xml:id=\"tab1.1.4.4\">z</td></tr><tr xml:id=\"tab1.1.5\"><td align=\"left\" xml:id=\"tab1.1.5.1\">D</td><td align=\"right\" class=\"ltx_norightpad\" xml:id=\"tab1.1.5.2\"/><td align=\"left\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.5.3\"/><td align=\"left\" xml:id=\"tab1.1.5.4\">w</td></tr><tr xml:id=\"tab1.1.6\"><td align=\"left\" xml:id=\"tab1.1.6.1\">E</td><td align=\"right\" class=\"ltx_norightpad\" xml:id=\"tab1.1.6.2\"><Math mode=\"inline\" tex=\"-1\" text=\"- 1\" xml:id=\"m6\"><XMath xml:id=\"m6.1\"><XMApp xml:id=\"m6.1.1\"><XMTok meaning=\"minus\" role=\"ADDOP\">-</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math></td><td align=\"left\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.6.3\">.<Math mode=\"inline\" tex=\"25\" text=\"25\" xml:id=\"m7\"><XMath xml:id=\"m7.1\"><XMTok meaning=\"25\" role=\"NUMBER\">25</XMTok></XMath></Math></td><td align=\"left\" xml:id=\"tab1.1.6.4\">v</td></tr><tr xml:id=\"tab1.1.7\"><td align=\"left\" border=\"b\" xml:id=\"tab1.1.7.1\">F</td><td align=\"right\" border=\"b\" class=\"ltx_norightpad\" xml:id=\"tab1.1.7.2\"><Math mode=\"inline\" tex=\"12\" text=\"12\" xml:id=\"m8\"><XMath xml:id=\"m8.1\"><XMTok meaning=\"12\" role=\"NUMBER\">12</XMTok></XMath></Math></td><td align=\"left\" border=\"b\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.7.3\"/><td align=\"left\" border=\"b\" xml:id=\"tab1.1.7.4\">u</td></tr></tbody>",
    ),
    (
      undecimal,
      "tbody",
      "",
      "<tbody><tr xml:id=\"tab1.1.2\"><td align=\"left\" border=\"b t\" xml:id=\"tab1.1.2.1\">A</td><td align=\"right\" border=\"b t\" xml:id=\"tab1.1.2.2\">12.3</td></tr></tbody>",
    ),
    (
      math,
      "tr",
      "tab1.1.2",
      "<tr xml:id=\"tab1.1.2\"><td align=\"left\" border=\"t\" xml:id=\"tab1.1.2.1\">A</td><td align=\"center\" border=\"t\" xml:id=\"tab1.1.2.2\"><Math mode=\"inline\" tex=\"69.0\\pm 3.2\" text=\"69.0 plus-or-minus 3.2\" xml:id=\"m2\"><XMath xml:id=\"m2.1\"><XMApp xml:id=\"m2.1.1\"><XMTok meaning=\"plus-or-minus\" name=\"pm\" role=\"ADDOP\">±</XMTok><XMTok meaning=\"69.0\" role=\"NUMBER\">69.0</XMTok><XMTok meaning=\"3.2\" role=\"NUMBER\">3.2</XMTok></XMApp></XMath></Math></td><td align=\"right\" border=\"t\" class=\"ltx_norightpad\" xml:id=\"tab1.1.2.3\"><Math mode=\"inline\" tex=\"12\" text=\"12\" xml:id=\"m3\"><XMath xml:id=\"m3.1\"><XMTok meaning=\"12\" role=\"NUMBER\">12</XMTok></XMath></Math></td><td align=\"left\" border=\"t\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.2.4\">.<Math mode=\"inline\" tex=\"345\" text=\"345\" xml:id=\"m4\"><XMath xml:id=\"m4.1\"><XMTok meaning=\"345\" role=\"NUMBER\">345</XMTok></XMath></Math></td></tr>",
    ),
    (
      math,
      "tr",
      "tab1.1.4",
      "<tr xml:id=\"tab1.1.4\"><td align=\"left\" border=\"b\" xml:id=\"tab1.1.4.1\">C</td><td align=\"center\" border=\"b\" xml:id=\"tab1.1.4.2\"><Math mode=\"inline\" tex=\"z_{0}\" text=\"z _ 0\" xml:id=\"m7\"><XMath xml:id=\"m7.1\"><XMApp xml:id=\"m7.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">z</XMTok><XMTok fontsize=\"70%\" meaning=\"0\" role=\"NUMBER\">0</XMTok></XMApp></XMath></Math></td><td align=\"right\" border=\"b\" class=\"ltx_norightpad\" xml:id=\"tab1.1.4.3\"><Math mode=\"inline\" tex=\"1\" text=\"1\" xml:id=\"m8\"><XMath xml:id=\"m8.1\"><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMath></Math></td><td align=\"left\" border=\"b\" class=\"ltx_noleftpad\" xml:id=\"tab1.1.4.4\">.<Math mode=\"inline\" tex=\"5\" text=\"5\" xml:id=\"m9\"><XMath xml:id=\"m9.1\"><XMTok meaning=\"5\" role=\"NUMBER\">5</XMTok></XMath></Math></td></tr>",
    ),
    (
      plain,
      "tr",
      "tab1.1.2",
      "<tr xml:id=\"tab1.1.2\"><td align=\"left\" border=\"b t\" xml:id=\"tab1.1.2.1\">A</td><td align=\"center\" border=\"b t\" xml:id=\"tab1.1.2.2\"><Math mode=\"inline\" tex=\"x^{2}\" text=\"x ^ 2\" xml:id=\"m2\"><XMath xml:id=\"m2.1\"><XMApp xml:id=\"m2.1.1\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok></XMApp></XMath></Math> and text</td></tr>",
    ),
    (
      plain,
      "tr",
      "p1.1.1",
      "<tr xml:id=\"p1.1.1\"><td align=\"left\" thead=\"row\" xml:id=\"p1.1.1.1\">a</td><td align=\"center\" xml:id=\"p1.1.1.2\"><Math mode=\"inline\" tex=\"b_{1}\" text=\"b _ 1\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok><XMTok fontsize=\"70%\" meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math></td><td align=\"right\" xml:id=\"p1.1.1.3\"><Math mode=\"inline\" tex=\"~\\cdots\" text=\"cdots\" xml:id=\"p1.m2\"><XMath xml:id=\"p1.m2.1\"><XMTok lpadding=\"3.3pt\" name=\"cdots\" role=\"ELIDEOP\">⋯</XMTok></XMath></Math></td></tr>",
    ),
    (
      plain,
      "p",
      "p1.2",
      "<p xml:id=\"p1.2\">After: <Math mode=\"inline\" tex=\"y_{1}\" text=\"y _ 1\" xml:id=\"p1.m3\"><XMath xml:id=\"p1.m3.1\"><XMApp xml:id=\"p1.m3.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok><XMTok fontsize=\"70%\" meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math> and $5.</p>",
    ),
    (
      two_tables,
      "tr",
      "tab2.1.2",
      "<tr xml:id=\"tab2.1.2\"><td align=\"left\" border=\"b t\" xml:id=\"tab2.1.2.1\">B</td><td align=\"right\" border=\"b t\" xml:id=\"tab2.1.2.2\">7.25</td></tr>",
    ),
  ];
  for (tex, tag, id, element) in cases {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
      assert_eq!(lines, 0, "jing:\n{xml}");
    }
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62x: a `$$` display opens and closes on a math shift by meaning — tex.web §1138 `init_math` and §1197 test
/// `cur_cmd=math_shift` — so a `$` made active and `\let` to the math shift (aastex's tables, `\let$\savedollar`)
/// pairs as a catcode-3 `$` does (tex_math.rs `next_is_math_shift`). pdflatex: a display `x^2` between the two lines.
#[test]
fn active_math_shift_pairs_for_display_math() {
  let tex = r"\documentclass{article}
\let\savedollar=$
\begingroup\catcode`\$=\active \global\let$=\savedollar\endgroup
\begin{document}
\catcode`\$=\active
Text $$x^2$$ more and $y$.
\end{document}";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Text</p><equation xml:id=\"S0.Ex1\"><Math mode=\"display\" tex=\"x^{2}\" text=\"x ^ 2\" xml:id=\"S0.Ex1.m1\"><XMath xml:id=\"S0.Ex1.m1.1\"><XMApp xml:id=\"S0.Ex1.m1.1.1\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok></XMApp></XMath></Math></equation><p xml:id=\"p1.2\">more and <Math mode=\"inline\" tex=\"y\" text=\"y\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok></XMath></Math>.</p></para>",
  );
}

/// 62y: aastex7/aastex701 load rotating before their own `\rotate` (aastex701.cls:11416, 12196), so a
/// deluxetable's `\rotate` stays the no-op it is here (deluxetable_sty.rs) rather than rotating's `{rotate}`
/// environment, which the class file's dependency scan had loaded after the binding; `rotatetable` (:12212) sets its
/// body. Witnesses 2609.09266 (Fatal TooManyErrors → 2 errors, both its .bib's), 2609.01052.
#[test]
fn aastex7_rotate_and_rotatetable() {
  let tex = r"\documentclass{aastex701}\begin{document}
\begin{deluxetable*}{ll}
\rotate
\tablecaption{Cap\label{t1}}
\tablehead{\colhead{A} & \colhead{B}}
\startdata
x & y \\
\enddata
\end{deluxetable*}
\begin{rotatetable}
Turned text.
\end{rotatetable}
\end{document}";
  let cases: [(&str, &str, &str, &str); 2] = [
    (
      tex,
      "tr",
      "S0.T1.2.2",
      "<tr xml:id=\"S0.T1.2.2\"><td align=\"left\" border=\"b t\" xml:id=\"S0.T1.2.2.1\">x</td><td align=\"left\" border=\"b t\" xml:id=\"S0.T1.2.2.2\">y</td></tr>",
    ),
    (tex, "p", "p1.1", "<p xml:id=\"p1.1\">Turned text.</p>"),
  ];
  for (tex, tag, id, element) in cases {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62y: a `\left`/`\right` delimiter is its character (tex.web §1160 `scan_delimiter`: by `\delcode`, never
/// `\mathcode`), so a math-active `(` whose active meaning is `\left(` gives one delimiter instead of recursing
/// (`stomach::digest_as_delimiter`; Perl's `TeXDelimiter` recurses too, KNOWN_PERL_ERRORS #494), and so for `\bigl(`, which is
/// `\left(` (the `TeXDelimiter` parameter). pdflatex: f(x) = 1/2 (a+b), [x], and the big parentheses. Witness 2609.40266 (Fatal Recursion → 0 errors).
#[test]
fn math_active_delimiter_is_its_character() {
  let tex = r"\documentclass{article}
\newcommand*\autoop{\left(}
\newcommand*\autocp{\right)}
\AtBeginDocument{%
  \mathcode`( 32768 \mathcode`) 32768
  \begingroup\lccode`\~`(\lowercase{\endgroup\let~\autoop}%
  \begingroup\lccode`\~`)\lowercase{\endgroup\let~\autocp}}
\begin{document}
Text $f(x) = \frac{1}{2}(a+b)$ and $\left[ x \right]$.

Big $\bigl( x \bigr)$ and $\Big( y \Big)$.
\end{document}";
  let cases: [(&str, &str, &str, &str); 2] = [
    (
      tex,
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">Text <Math mode=\"inline\" tex=\"f\\left(x\\right)=\\frac{1}{2}\\left(a+b\\right)\" text=\"f@(x) = (1 / 2) * (a + b)\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.3\"><XMApp xml:id=\"p1.m1.3.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMApp xml:id=\"p1.m1.3.1.2\"><XMTok font=\"italic\" role=\"UNKNOWN\">f</XMTok><XMDual xml:id=\"p1.m1.3.1.2.2\"><XMRef idref=\"p1.m1.1\" xml:id=\"p1.m1.3.1.2.2.1\"/><XMWrap xml:id=\"p1.m1.3.1.2.2.2\"><XMTok role=\"OPEN\" stretchy=\"true\">(</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.1\">x</XMTok><XMTok role=\"CLOSE\" stretchy=\"true\">)</XMTok></XMWrap></XMDual></XMApp><XMApp xml:id=\"p1.m1.3.1.3\"><XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok><XMApp xml:id=\"p1.m1.3.1.3.2\"><XMTok mathstyle=\"text\" meaning=\"divide\" role=\"FRACOP\"/><XMTok fontsize=\"70%\" meaning=\"1\" role=\"NUMBER\">1</XMTok><XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok></XMApp><XMDual xml:id=\"p1.m1.3.1.3.3\"><XMRef idref=\"p1.m1.2\" xml:id=\"p1.m1.3.1.3.3.1\"/><XMWrap xml:id=\"p1.m1.3.1.3.3.2\"><XMTok role=\"OPEN\" stretchy=\"true\">(</XMTok><XMApp xml:id=\"p1.m1.2\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp><XMTok role=\"CLOSE\" stretchy=\"true\">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math> and <Math mode=\"inline\" tex=\"\\left[x\\right]\" text=\"delimited-[]@(x)\" xml:id=\"p1.m2\"><XMath xml:id=\"p1.m2.2\"><XMDual xml:id=\"p1.m2.2.1\"><XMApp xml:id=\"p1.m2.2.1.1\"><XMTok meaning=\"delimited-[]\"/><XMRef idref=\"p1.m2.1\" xml:id=\"p1.m2.2.1.1.2\"/></XMApp><XMWrap xml:id=\"p1.m2.2.1.2\"><XMTok role=\"OPEN\" stretchy=\"true\">[</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m2.1\">x</XMTok><XMTok role=\"CLOSE\" stretchy=\"true\">]</XMTok></XMWrap></XMDual></XMath></Math>.</p>",
    ),
    (
      tex,
      "p",
      "p2.1",
      "<p xml:id=\"p2.1\">Big <Math mode=\"inline\" tex=\"\\bigl(x\\bigr)\" text=\"x\" xml:id=\"p2.m1\"><XMath xml:id=\"p2.m1.2\"><XMDual xml:id=\"p2.m1.2.1\"><XMRef idref=\"p2.m1.1\" xml:id=\"p2.m1.2.1.1\"/><XMWrap xml:id=\"p2.m1.2.1.2\"><XMTok fontsize=\"120%\" role=\"OPEN\" stretchy=\"false\">(</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p2.m1.1\">x</XMTok><XMTok fontsize=\"120%\" role=\"CLOSE\" stretchy=\"false\">)</XMTok></XMWrap></XMDual></XMath></Math> and <Math mode=\"inline\" tex=\"\\Big(y\\Big)\" text=\"y\" xml:id=\"p2.m2\"><XMath xml:id=\"p2.m2.2\"><XMDual xml:id=\"p2.m2.2.1\"><XMRef idref=\"p2.m2.1\" xml:id=\"p2.m2.2.1.1\"/><XMWrap xml:id=\"p2.m2.2.1.2\"><XMTok fontsize=\"160%\" role=\"OPEN\" stretchy=\"false\">(</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p2.m2.1\">y</XMTok><XMTok fontsize=\"160%\" role=\"CLOSE\" stretchy=\"false\">)</XMTok></XMWrap></XMDual></XMath></Math>.</p>",
    ),
  ];
  for (tex, tag, id, element) in cases {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62y: a math-active character whose active meaning is undefined self-inserts (`lookup_digestable_definition`) as
/// the plain character, not a mathcode "8000 decoded as a math character — dropped here before, Γ in Perl
/// (KNOWN_PERL_ERRORS #495). braket's own `\Pr`-style idiom reaches it: braket.sty:73 defines `\SetVert`, which our braket
/// binding (and Perl's) does not, so the active `|` has no meaning. pdflatex: P( A | B ). Witness 1602.01342.
#[test]
fn math_active_character_without_meaning_is_itself() {
  let tex = r"\documentclass{article}
\usepackage{braket}
{\catcode`\|=\active
 \gdef\Pr#1{\mathrm{P}\left(\:{\mathcode`\|32768\let|\SetVert #1}\:\right)}}
\begin{document}
$\Pr{A|B}$
\end{document}";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\"><Math mode=\"inline\" tex=\"\\mathrm{P}\\left(\\&gt;{A|B}\\&gt;\\right)\" text=\"P@(conditional@(A, B))\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.3\"><XMApp xml:id=\"p1.m1.3.1\"><XMTok role=\"UNKNOWN\">P</XMTok><XMDual xml:id=\"p1.m1.3.1.2\"><XMApp xml:id=\"p1.m1.3.1.2.1\"><XMTok meaning=\"conditional\"/><XMRef idref=\"p1.m1.1\" xml:id=\"p1.m1.3.1.2.1.2\"/><XMRef idref=\"p1.m1.2\" xml:id=\"p1.m1.3.1.2.1.3\"/></XMApp><XMWrap xml:id=\"p1.m1.3.1.2.2\"><XMTok role=\"OPEN\" stretchy=\"true\">(</XMTok><XMTok font=\"italic\" lpadding=\"2.2pt\" role=\"UNKNOWN\" xml:id=\"p1.m1.1\">A</XMTok><XMTok role=\"MIDDLE\" stretchy=\"false\">|</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" rpadding=\"2.2pt\" xml:id=\"p1.m1.2\">B</XMTok><XMTok role=\"CLOSE\" stretchy=\"true\">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></p>",
  );
}

/// 62y: autobreak (latexml_contrib autobreak_sty.rs): the body's first non-empty line is the left-hand side and the rest
/// follows the alignment tab (autobreak.sty:283-299), a line end inside braces stays in its line (:163) and a leading
/// `,` joins the line before (:166-197), the width-driven breaks left out; the raw package needed
/// amsmath internals the native `align` never runs. Witness 2609.08470 (17 environments, Fatal TooManyErrors → 0).
#[test]
fn autobreak_lines_after_the_left_side() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\usepackage{autobreak}
\begin{document}
\begin{align*}
\begin{autobreak}
\beta(g) =

+ \frac{199}{18} g^{5}
- 3 g^{3}
\end{autobreak}
\end{align*}
\begin{align*}
\begin{autobreak}
F = \frac{a}{
b}
+ c
\end{autobreak}
\end{align*}
\begin{align*}
\begin{autobreak}
K
, = x
+ y
\end{autobreak}
\end{align*}
After.
\end{document}";
  let cases: [(&str, &str, &str, &str); 3] = [
    (
      tex,
      "equationgroup",
      "S0.EGx1",
      "<equationgroup class=\"ltx_eqn_align\" xml:id=\"S0.EGx1\"><equation xml:id=\"S0.Ex1\"><MathFork><Math tex=\"\\displaystyle\\beta(g)={}+\\frac{199}{18}g^{5}-3g^{3}\" text=\"beta@(g) = (+ (199 / 18) * g ^ 5) - 3 * g ^ 3\" xml:id=\"S0.Ex1.m3\"><XMath xml:id=\"S0.Ex1.m3.2\"><XMApp xml:id=\"S0.Ex1.m3.2.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMApp xml:id=\"S0.Ex1.m3.2.1.2\"><XMTok font=\"italic\" name=\"beta\" role=\"UNKNOWN\">β</XMTok><XMDual xml:id=\"S0.Ex1.m3.2.1.2.2\"><XMRef idref=\"S0.Ex1.m3.1\" xml:id=\"S0.Ex1.m3.2.1.2.2.1\"/><XMWrap xml:id=\"S0.Ex1.m3.2.1.2.2.2\"><XMTok role=\"OPEN\" stretchy=\"false\">(</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"S0.Ex1.m3.1\">g</XMTok><XMTok role=\"CLOSE\" stretchy=\"false\">)</XMTok></XMWrap></XMDual></XMApp><XMApp xml:id=\"S0.Ex1.m3.2.1.3\"><XMTok meaning=\"minus\" role=\"ADDOP\">-</XMTok><XMApp xml:id=\"S0.Ex1.m3.2.1.3.2\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMApp xml:id=\"S0.Ex1.m3.2.1.3.2.2\"><XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok><XMApp xml:id=\"S0.Ex1.m3.2.1.3.2.2.2\"><XMTok mathstyle=\"display\" meaning=\"divide\" role=\"FRACOP\"/><XMTok meaning=\"199\" role=\"NUMBER\">199</XMTok><XMTok meaning=\"18\" role=\"NUMBER\">18</XMTok></XMApp><XMApp xml:id=\"S0.Ex1.m3.2.1.3.2.2.3\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok><XMTok fontsize=\"70%\" meaning=\"5\" role=\"NUMBER\">5</XMTok></XMApp></XMApp></XMApp><XMApp xml:id=\"S0.Ex1.m3.2.1.3.3\"><XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok><XMTok meaning=\"3\" role=\"NUMBER\">3</XMTok><XMApp xml:id=\"S0.Ex1.m3.2.1.3.3.3\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok><XMTok fontsize=\"70%\" meaning=\"3\" role=\"NUMBER\">3</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math><MathBranch><td align=\"right\" xml:id=\"S0.Ex1.1\"><Math mode=\"inline\" tex=\"\\displaystyle\\beta(g)={}\" text=\"beta@(g) = absent\" xml:id=\"S0.Ex1.m1\"><XMath xml:id=\"S0.Ex1.m1.2\"><XMApp xml:id=\"S0.Ex1.m1.2.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMApp xml:id=\"S0.Ex1.m1.2.1.2\"><XMTok font=\"italic\" name=\"beta\" role=\"UNKNOWN\">β</XMTok><XMDual xml:id=\"S0.Ex1.m1.2.1.2.2\"><XMRef idref=\"S0.Ex1.m1.1\" xml:id=\"S0.Ex1.m1.2.1.2.2.1\"/><XMWrap xml:id=\"S0.Ex1.m1.2.1.2.2.2\"><XMTok role=\"OPEN\" stretchy=\"false\">(</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"S0.Ex1.m1.1\">g</XMTok><XMTok role=\"CLOSE\" stretchy=\"false\">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning=\"absent\"/></XMApp></XMath></Math></td><td align=\"left\" xml:id=\"S0.Ex1.2\"><Math mode=\"inline\" tex=\"\\displaystyle+\\frac{199}{18}g^{5}-3g^{3}\" text=\"(+ (199 / 18) * g ^ 5) - 3 * g ^ 3\" xml:id=\"S0.Ex1.m2\"><XMath xml:id=\"S0.Ex1.m2.1\"><XMApp xml:id=\"S0.Ex1.m2.1.1\"><XMTok meaning=\"minus\" role=\"ADDOP\">-</XMTok><XMApp xml:id=\"S0.Ex1.m2.1.1.2\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMApp xml:id=\"S0.Ex1.m2.1.1.2.2\"><XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok><XMApp xml:id=\"S0.Ex1.m2.1.1.2.2.2\"><XMTok mathstyle=\"display\" meaning=\"divide\" role=\"FRACOP\"/><XMTok meaning=\"199\" role=\"NUMBER\">199</XMTok><XMTok meaning=\"18\" role=\"NUMBER\">18</XMTok></XMApp><XMApp xml:id=\"S0.Ex1.m2.1.1.2.2.3\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok><XMTok fontsize=\"70%\" meaning=\"5\" role=\"NUMBER\">5</XMTok></XMApp></XMApp></XMApp><XMApp xml:id=\"S0.Ex1.m2.1.1.3\"><XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok><XMTok meaning=\"3\" role=\"NUMBER\">3</XMTok><XMApp xml:id=\"S0.Ex1.m2.1.1.3.3\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok><XMTok fontsize=\"70%\" meaning=\"3\" role=\"NUMBER\">3</XMTok></XMApp></XMApp></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>",
    ),
    (
      tex,
      "equationgroup",
      "S0.EGx2",
      "<equationgroup class=\"ltx_eqn_align\" xml:id=\"S0.EGx2\"><equation xml:id=\"S0.Ex2\"><MathFork><Math tex=\"\\displaystyle F=\\frac{a}{b}{}+c\" text=\"F = a / b + c\" xml:id=\"S0.Ex2.m3\"><XMath xml:id=\"S0.Ex2.m3.1\"><XMApp xml:id=\"S0.Ex2.m3.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">F</XMTok><XMApp xml:id=\"S0.Ex2.m3.1.1.3\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMApp xml:id=\"S0.Ex2.m3.1.1.3.2\"><XMTok mathstyle=\"display\" meaning=\"divide\" role=\"FRACOP\"/><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp><XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok></XMApp></XMApp></XMath></Math><MathBranch><td align=\"right\" xml:id=\"S0.Ex2.1\"><Math mode=\"inline\" tex=\"\\displaystyle F=\\frac{a}{b}{}\" text=\"F = a / b\" xml:id=\"S0.Ex2.m1\"><XMath xml:id=\"S0.Ex2.m1.1\"><XMApp xml:id=\"S0.Ex2.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">F</XMTok><XMApp xml:id=\"S0.Ex2.m1.1.1.3\"><XMTok mathstyle=\"display\" meaning=\"divide\" role=\"FRACOP\"/><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp></XMApp></XMath></Math></td><td align=\"left\" xml:id=\"S0.Ex2.2\"><Math mode=\"inline\" tex=\"\\displaystyle+c\" text=\"+ c\" xml:id=\"S0.Ex2.m2\"><XMath xml:id=\"S0.Ex2.m2.1\"><XMApp xml:id=\"S0.Ex2.m2.1.1\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>",
    ),
    (
      tex,
      "equationgroup",
      "S0.EGx3",
      "<equationgroup class=\"ltx_eqn_align\" xml:id=\"S0.EGx3\"><equation xml:id=\"S0.Ex3\"><MathFork><Math tex=\"\\displaystyle K,{}=x+y\" text=\"formulae@(K = x + y, absent = x + y)\" xml:id=\"S0.Ex3.m3\"><XMath xml:id=\"S0.Ex3.m3.5\"><XMDual xml:id=\"S0.Ex3.m3.5.1\"><XMApp xml:id=\"S0.Ex3.m3.5.1.1\"><XMTok meaning=\"formulae\"/><XMApp xml:id=\"S0.Ex3.m3.5.1.1.2\"><XMRef idref=\"S0.Ex3.m3.1\" xml:id=\"S0.Ex3.m3.5.1.1.2.1\"/><XMRef idref=\"S0.Ex3.m3.2\" xml:id=\"S0.Ex3.m3.5.1.1.2.2\"/><XMRef idref=\"S0.Ex3.m3.4\" xml:id=\"S0.Ex3.m3.5.1.1.2.3\"/></XMApp><XMApp xml:id=\"S0.Ex3.m3.5.1.1.3\"><XMRef idref=\"S0.Ex3.m3.1\" xml:id=\"S0.Ex3.m3.5.1.1.3.1\"/><XMRef idref=\"S0.Ex3.m3.3\" xml:id=\"S0.Ex3.m3.5.1.1.3.2\"/><XMRef idref=\"S0.Ex3.m3.4\" xml:id=\"S0.Ex3.m3.5.1.1.3.3\"/></XMApp></XMApp><XMApp xml:id=\"S0.Ex3.m3.5.1.2\"><XMTok meaning=\"equals\" role=\"RELOP\" xml:id=\"S0.Ex3.m3.1\">=</XMTok><XMWrap xml:id=\"S0.Ex3.m3.5.1.2.1\"><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"S0.Ex3.m3.2\">K</XMTok><XMTok role=\"PUNCT\">,</XMTok><XMTok meaning=\"absent\" xml:id=\"S0.Ex3.m3.3\"/></XMWrap><XMApp xml:id=\"S0.Ex3.m3.4\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok></XMApp></XMApp></XMDual></XMath></Math><MathBranch><td align=\"right\" xml:id=\"S0.Ex3.1\"><Math mode=\"inline\" tex=\"\\displaystyle K,{}\" text=\"K\" xml:id=\"S0.Ex3.m1\"><XMath xml:id=\"S0.Ex3.m1.2\"><XMDual xml:id=\"S0.Ex3.m1.2.1\"><XMRef idref=\"S0.Ex3.m1.1\" xml:id=\"S0.Ex3.m1.2.1.1\"/><XMWrap xml:id=\"S0.Ex3.m1.2.1.2\"><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"S0.Ex3.m1.1\">K</XMTok><XMTok role=\"PUNCT\">,</XMTok></XMWrap></XMDual></XMath></Math></td><td align=\"left\" xml:id=\"S0.Ex3.2\"><Math mode=\"inline\" tex=\"\\displaystyle=x+y\" text=\"absent = x + y\" xml:id=\"S0.Ex3.m2\"><XMath xml:id=\"S0.Ex3.m2.1\"><XMApp xml:id=\"S0.Ex3.m2.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok meaning=\"absent\"/><XMApp xml:id=\"S0.Ex3.m2.1.1.3\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok></XMApp></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>",
    ),
  ];
  for (tex, tag, id, element) in cases {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62z: amsart's size machinery (ams_support_sty.rs, amsart.cls:169-219, 258-296): `\@xsetfontsize\cs N` sets `\cs` at the
/// Nth of the size option's `\@typesizes` (normalsize 6), as a class built on amsart redefines its sizes (m2an.cls:241).
/// Under 11pt, `\small` is 10pt (91%) and `\large` 12pt (110%). Witness 2609.37833 (undefined `\@xsetfontsize`, then Fatal
/// PushbackLimit; now 0 errors).
#[test]
fn amsart_xsetfontsize_sizes() {
  let tex = r"\documentclass[11pt]{amsart}
\makeatletter
\renewcommand\normalsize{\@xsetfontsize\normalsize 6\@adjustvertspacing}
\DeclareRobustCommand{\small}{\@xsetfontsize\small 5\@adjustvertspacing}
\DeclareRobustCommand{\large}{\@xsetfontsize\large 7\@adjustvertspacing}
\makeatother
\begin{document}
Body text. {\small Small text.} {\large Large text.}
\end{document}";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Body text. <text fontsize=\"91%\" xml:id=\"p1.1.1\">Small text.</text> <text fontsize=\"110%\" xml:id=\"p1.1.2\">Large text.</text></p>",
  );
  // A size given to the class alone (`\PassOptionsToClass`, as a class's `\LoadClass[12pt]{amsart}`) is the class's,
  // which the class binding declares (ams_core_cls.rs): `\@mainsize` 12, `\small` 10.95pt (110% of the 10pt base).
  let class_option = r"\PassOptionsToClass{12pt}{amsart}
\documentclass{amsart}
\makeatletter
\DeclareRobustCommand{\small}{\@xsetfontsize\small 5\@adjustvertspacing}
\begin{document}
Body \@mainsize. {\small Small text.}
\end{document}";
  let (log, xml) = convert_with(class_option, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Body 12. <text fontsize=\"110%\" xml:id=\"p1.1.1\">Small text.</text></p>",
  );
  // The class's own size wins over the document's: ams_support declares the size options as no-ops, as a package's
  // `ProcessOptions` would otherwise run `11pt` again after the class settled `12pt` (pdflatex: Body 12).
  let class_size_wins = r"\PassOptionsToClass{12pt}{amsart}
\documentclass[11pt]{amsart}
\makeatletter
\renewcommand\normalsize{\@xsetfontsize\normalsize 6\@adjustvertspacing}
\DeclareRobustCommand{\small}{\@xsetfontsize\small 5\@adjustvertspacing}
\makeatother
\begin{document}
\makeatletter Body \@mainsize. {\small Small.}\makeatother
\end{document}";
  let (log, xml) = convert_with(class_size_wins, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Body 12. <text fontsize=\"91%\" xml:id=\"p1.1.1\">Small.</text></p>",
  );
  // The size option's `\@ptsize` (amsart.cls:274) survives the article binding's reset (ams_core_cls.rs): pdflatex
  // prints 12[2].
  let ptsize = r"\documentclass[12pt]{amsart}
\begin{document}
\makeatletter Body \@mainsize[\@ptsize].\makeatother
\end{document}";
  let (log, xml) = convert_with(ptsize, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Body 12[2].</p>",
  );
}

/// 62z: `\label` does not digest `\@currentlabel` (`\the<ctr>`, unexpanded as in Perl) into a value nothing reads
/// (sect11.rs): read after a list whose `label=` redefined `\theenumi` locally, it reached the document's own `\theenumi`
/// → `\RegularTheEnumi` → `\theenumi` loop. Perl hangs (KNOWN_PERL_ERRORS #496). pdflatex: the label is the section's.
/// Witness 2609.13327 (Fatal Recursion → 0 errors).
#[test]
fn label_after_a_relabeled_list_does_not_loop() {
  let tex = r"\documentclass{article}\usepackage[inline]{enumitem}\begin{document}
\def\RegularTheEnumi{\theenumi}\renewcommand{\theenumi}{\RegularTheEnumi}
\section{S}A \begin{enumerate*}[label=(\roman*)] \item two\end{enumerate*} \label{s:a}
\end{document}";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" labels=\"LABEL:s:a\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">§1</tag></tags><title><tag close=\" \">1</tag>S</title><para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\">A <inline-enumerate xml:id=\"S1.I1\"><inline-item xml:id=\"S1.I1.i1\"><tags><tag>(i)</tag><tag role=\"refnum\">(i)</tag><tag role=\"typerefnum\">item\u{a0}(i)</tag></tags><text xml:id=\"S1.I1.i1.1\">two</text></inline-item></inline-enumerate></p></para></section>",
  );
}

/// 62z: mathtools' paired delimiters (mathtools.sty:874-990, mathtools_sty.rs): `\delimsize` is the size in a group,
/// a sized delimiter takes `\<size>l`/`\<size>r` (`\relax` when undefined) and may use `\delimsize` itself, `[\big]` is
/// `\bigl`/`\bigr`. Perl put the size itself before the delimiter: with `\delimsize` in the delimiters, `[\cbig]` read
/// the delimiter's `\delimsize` as its argument (here `\big\big.\rangle`, silently). pdflatex: ⟨O⟩ big, |x|, |y| big, |z|,
/// {x | x>0}. Witness 2609.17447 (861 errors, Fatal TooManyErrors → 0).
#[test]
fn paired_delimiter_size_and_delimsize() {
  let tex = r"\documentclass{article}
\usepackage{mathtools}
\providecommand{\delimsize}{\relax}
\newcommand{\cbig}[1]{\big#1}
\DeclarePairedDelimiterX{\mb}[1]{\delimsize\langle}{\delimsize\rangle}{#1}
\DeclarePairedDelimiter{\abs}{\lvert}{\rvert}
\DeclarePairedDelimiterX{\set}[2]{\{}{\}}{#1 \;\delimsize\vert\; #2}
\begin{document}
$\mb[\cbig]{O}$ and $\abs{x} + \abs[\big]{y} + \abs*{z}$ and $\set[\big]{x}{x>0}$.
\end{document}";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\"><Math mode=\"inline\" tex=\"\\big\\langle O\\big\\rangle\" text=\"delimited-⟨⟩@(O)\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.2\"><XMDual xml:id=\"p1.m1.2.1\"><XMApp xml:id=\"p1.m1.2.1.1\"><XMTok meaning=\"delimited-⟨⟩\"/><XMRef idref=\"p1.m1.1\" xml:id=\"p1.m1.2.1.1.2\"/></XMApp><XMWrap xml:id=\"p1.m1.2.1.2\"><XMTok fontsize=\"120%\" name=\"langle\" role=\"OPEN\" stretchy=\"false\">⟨</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.1\">O</XMTok><XMTok fontsize=\"120%\" name=\"rangle\" role=\"CLOSE\" stretchy=\"false\">⟩</XMTok></XMWrap></XMDual></XMath></Math> and <Math mode=\"inline\" tex=\"\\lvert x\\rvert+\\bigl\\lvert y\\bigr\\rvert+\\left\\lvert z\\right\\rvert\" text=\"absolute-value@(x) + absolute-value@(y) + absolute-value@(z)\" xml:id=\"p1.m2\"><XMath xml:id=\"p1.m2.4\"><XMApp xml:id=\"p1.m2.4.1\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMDual xml:id=\"p1.m2.4.1.2\"><XMApp xml:id=\"p1.m2.4.1.2.1\"><XMTok meaning=\"absolute-value\"/><XMRef idref=\"p1.m2.1\" xml:id=\"p1.m2.4.1.2.1.2\"/></XMApp><XMWrap xml:id=\"p1.m2.4.1.2.2\"><XMTok name=\"lvert\" role=\"OPEN\" stretchy=\"false\">|</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m2.1\">x</XMTok><XMTok name=\"rvert\" role=\"CLOSE\" stretchy=\"false\">|</XMTok></XMWrap></XMDual><XMDual xml:id=\"p1.m2.4.1.3\"><XMApp xml:id=\"p1.m2.4.1.3.1\"><XMTok meaning=\"absolute-value\"/><XMRef idref=\"p1.m2.2\" xml:id=\"p1.m2.4.1.3.1.2\"/></XMApp><XMWrap xml:id=\"p1.m2.4.1.3.2\"><XMTok fontsize=\"120%\" name=\"lvert\" role=\"OPEN\" stretchy=\"false\">|</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m2.2\">y</XMTok><XMTok fontsize=\"120%\" name=\"rvert\" role=\"CLOSE\" stretchy=\"false\">|</XMTok></XMWrap></XMDual><XMDual xml:id=\"p1.m2.4.1.4\"><XMApp xml:id=\"p1.m2.4.1.4.1\"><XMTok meaning=\"absolute-value\"/><XMRef idref=\"p1.m2.3\" xml:id=\"p1.m2.4.1.4.1.2\"/></XMApp><XMWrap xml:id=\"p1.m2.4.1.4.2\"><XMTok role=\"OPEN\" stretchy=\"true\">|</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m2.3\">z</XMTok><XMTok role=\"CLOSE\" stretchy=\"true\">|</XMTok></XMWrap></XMDual></XMApp></XMath></Math> and <Math mode=\"inline\" tex=\"\\bigl\\{x\\;\\big|\\;x&gt;0\\bigr\\}\" text=\"conditional-set@(x, x &gt; 0)\" xml:id=\"p1.m3\"><XMath xml:id=\"p1.m3.3\"><XMDual xml:id=\"p1.m3.3.1\"><XMApp xml:id=\"p1.m3.3.1.1\"><XMTok meaning=\"conditional-set\"/><XMRef idref=\"p1.m3.1\" xml:id=\"p1.m3.3.1.1.2\"/><XMRef idref=\"p1.m3.2\" xml:id=\"p1.m3.3.1.1.3\"/></XMApp><XMWrap xml:id=\"p1.m3.3.1.2\"><XMTok fontsize=\"120%\" role=\"OPEN\" stretchy=\"false\">{</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" rpadding=\"2.8pt\" xml:id=\"p1.m3.1\">x</XMTok><XMTok fontsize=\"120%\" role=\"MIDDLE\" rpadding=\"2.8pt\" stretchy=\"false\">|</XMTok><XMApp xml:id=\"p1.m3.2\"><XMTok meaning=\"greater-than\" role=\"RELOP\">&gt;</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok meaning=\"0\" role=\"NUMBER\">0</XMTok></XMApp><XMTok fontsize=\"120%\" role=\"CLOSE\" stretchy=\"false\">}</XMTok></XMWrap></XMDual></XMath></Math>.</p>",
  );
}

/// 62za: `\minipage`/`\endminipage` are locked against class and package files (`:locked@files`, sect12.rs): a raw
/// file's own `\endminipage` (iucr.cls:3303-3315, dropping the footnote rule) closes the kernel minipage's box groups
/// and reads `\@mpargs`, none of which this environment opens; tikz copies it as `\pgfutil@endminipage` when it loads,
/// so every `text width` node, and every later minipage, ended in mode errors. pdflatex: the node's two lines, the
/// minipage. The node's leading empty `<p>` is an artifact pdflatex does not show, left unpinned. Witness 2609.07722
/// (265 errors, Fatal → 0).
#[test]
fn class_endminipage_does_not_replace_the_environment() {
  let tex = r"\begin{filecontents*}[overwrite]{endmp62za.sty}
\def\endminipage{\par\unskip
  \ifvoid\@mpfootins\else\vskip\skip\@mpfootins\normalcolor\unvbox\@mpfootins\fi
  \@minipagefalse\color@endgroup\egroup
  \expandafter\@iiiparbox\@mpargs{\unvbox\@tempboxa}}
\end{filecontents*}
\documentclass{article}
\usepackage{endmp62za}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\node[draw, text width=30mm, align=center] (a) {First line\\ second line};
\end{tikzpicture}

\begin{minipage}{3cm}Inside.\end{minipage} After.
\end{document}";
  assert_elements(tex, "ar5iv.sty", (0, 0), &[
    (
      "p",
      "p1.pic1.1.2",
      "<p xml:id=\"p1.pic1.1.2\">First line</p>",
    ),
    (
      "p",
      "p1.pic1.1.3",
      "<p xml:id=\"p1.pic1.1.3\">second line</p>",
    ),
    (
      "p",
      "p2.1",
      "<p xml:id=\"p2.1\"><inline-block class=\"ltx_minipage\" vattach=\"middle\" width=\"85.4pt\" xml:id=\"p2.1.1\"><p xml:id=\"p2.1.1.1\">Inside.</p></inline-block> After.</p>",
    ),
  ]);
}

/// 62za: the lock leaves the document's own definition alone, as LaTeX and Perl do (`is_name_locked`): pdflatex prints
/// the document's meaning.
#[test]
fn document_endminipage_definition_is_kept() {
  let tex = r"\documentclass{article}
\def\endminipage{the document definition}
\begin{document}
\texttt{\meaning\endminipage}
\end{document}";
  assert_elements(tex, "ar5iv.sty", (0, 0), &[(
    "p",
    "p1.1",
    "<p xml:id=\"p1.1\"><text font=\"typewriter\" xml:id=\"p1.1.1\">macro:-&gt;the document definition</text></p>",
  )]);
}

/// 62za: 2609 class-binding gaps, each what the class (or style) defines — egpubl.cls: argument-less version selectors
/// (`\PrintedOrElectronic` read the next `\ifpdf` as its argument), `\ConfName`, `\excludecomment{CCSXML}` (:820), and
/// `\teaser`, set by `\@maketitle` under the title block as a figure (`\def\@captype{figure}`, :766-774); bmvc2k.cls: geometry (:222), `\BMVA@blfootnote` (:516), `\bmvaEtAl` (:559), two-argument `\runninghead`
/// (:553); sn-jnl.cls `\unnumbered` (:878); acmart.cls's `\@secfont` (TL :3338, patched by pvldb.sty:35); and
/// aaai2027.sty's `\corresponding`/`\equalcontrib`, defined only inside its `\@maketitle` with their footnotes
/// (sect05.rs kernel stubs; `aaaimini62za.sty` here mimics it), which a raw file's own definition replaces. All pdflatex-clean with the shipped class files. Witnesses
/// 2609.00732, 2609.00994, 2609.00981, 2609.08090, 2609.05015, 2609.00548, 2609.00420 (223 AAAI-27 papers).
#[test]
fn class_bindings_2609_clusters() {
  let eg = assert_elements(
    r"\documentclass{egpubl}
\usepackage{comment}
\BibtexOrBiblatex
\electronicVersion
\PrintedOrElectronic
\ifpdf \usepackage[pdftex]{graphicx} \else \usepackage[dvips]{graphicx} \fi
\usepackage{hyperref}
\ConfName{Pacific Graphics}
\title{A Title}
\author{A. Author}
\teaser{\centering A teaser.\caption{The teaser.}\label{fig:teaser}}
\begin{document}
\maketitle
\begin{abstract}
Abstract text.
\begin{CCSXML}
<concept_id>10010147.10010371</concept_id>
\end{CCSXML}
\end{abstract}
Body, see Figure~\ref{fig:teaser}.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "figure",
        "S0.F1",
        "<figure class=\"ltx_teaserfigure\" inlist=\"lof\" labels=\"LABEL:fig:teaser\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><p align=\"center\" xml:id=\"S0.F1.2\">A teaser.</p><toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>The teaser.</toccaption><caption class=\"ltx_centering\"><tag close=\": \">Figure 1</tag>The teaser.</caption></figure>",
      ),
      (
        "abstract",
        "abstract1",
        "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">Abstract text.</p></abstract>",
      ),
    ],
  );
  // title, author, abstract (top matter, which the schema keeps ahead of every figure), teaser; no teaser note
  let order: Vec<usize> = [
    "<title>",
    "<creator ",
    "<abstract ",
    "<figure class=\"ltx_teaserfigure\"",
  ]
  .iter()
  .map(|tag| {
    eg.find(tag)
      .unwrap_or_else(|| panic!("{tag} missing:\n{eg}"))
  })
  .collect();
  assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}\n{eg}");
  assert!(!eg.contains("role=\"teaser\""), "{eg}");

  let bm = assert_elements(
    r"\documentclass{bmvc2k}
\geometry{margin=1in}
\title{A Title}
\addauthor{A. Author}{a@b.c}{1}
\addinstitution{An Institution}
\runninghead{Author}{Title}
\begin{document}
\maketitle
\makeatletter\BMVA@blfootnote{Equal contribution.}\makeatother
Body of \bmvaEtAl.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      ("text", "id1", "<text xml:id=\"id1\">Title</text>"),
      (
        "note",
        "footnote1",
        "<note role=\"footnote\" xml:id=\"footnote1\"><tags><tag role=\"autoref\">footnote\u{a0}<text xml:id=\"footnote1.1\"/></tag><tag role=\"typerefnum\">footnote</tag></tags>Equal contribution.</note>",
      ),
      (
        "para",
        "p1",
        "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Body of <text font=\"italic\" xml:id=\"p1.1.1\">et al..</text></p></para>",
      ),
    ],
  );
  assert_element(
    &bm,
    "toctitle",
    &[],
    "<toctitle><text xml:id=\"id1\">Title</text></toctitle>",
  );
  assert_elements(
    r"\documentclass[sn-basic]{sn-jnl}
\usepackage{amsmath}
\unnumbered
\begin{document}
\title{A Title}
\author{A. Author}
\maketitle
\section{Introduction}
Body.
\numbered
\section{Methods}
More.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "section",
        "Sx1",
        "<section inlist=\"toc\" xml:id=\"Sx1\"><title>Introduction</title><para xml:id=\"Sx1.p1\"><p xml:id=\"Sx1.p1.1\">Body.</p></para></section>",
      ),
      (
        "section",
        "S1",
        "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">§1</tag></tags><title><tag close=\" \">1</tag>Methods</title><para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\">More.</p></para></section>",
      ),
    ],
  );
  assert_elements(
    r"\documentclass[sigconf,nonacm]{acmart}
\usepackage{textcase}
\makeatletter
\expandafter\def\expandafter\@secfont\expandafter{\@secfont\MakeTextUppercase}
\makeatother
\begin{document}
\title{A Title}
\author{A. Author}
\maketitle
\section{Introduction}
Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "section",
      "S1",
      "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">§1</tag></tags><title><tag close=\". \">1</tag>Introduction</title><toctitle><tag close=\" \">1</tag>Introduction</toctitle><para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\">Body.</p></para></section>",
    )],
  );
  assert_elements(
    r"\begin{filecontents*}[overwrite]{aaaimini62za.sty}
\def\@maketitle{\begingroup
  \def\corresponding{\footnote{Corresponding author.}}%
  \def\equalcontrib{\footnote{These authors contributed equally.}}%
  {\LARGE\bf \@title\par}{\large\bf \@author\par}\endgroup}
\end{filecontents*}
\documentclass{article}
\usepackage{aaaimini62za}
\title{A Title}
\author{A. Author\corresponding, B. Author\equalcontrib}
\begin{document}
\maketitle
Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "note",
        "id1",
        "<note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" role=\"thanks\" xml:id=\"id1\">Corresponding author.</note>",
      ),
      (
        "note",
        "id2",
        "<note class=\"ltx_note_frontmatter ltx_thanks_contribution\" role=\"thanks\" xml:id=\"id2\">These authors contributed equally.</note>",
      ),
    ],
  );
  // A raw file's own definition replaces the kernel stub (`DefinitionOrigin::Stub`), as copernicus.cls:1673's
  // `\newcommand\equalcontrib[1]` (its equal-contribution note on the affiliations) does in pdflatex.
  assert_elements(
    r"\begin{filecontents*}[overwrite]{copmini62za.sty}
\newcommand\equalcontrib[1]{\textsuperscript{#1}These authors contributed equally to this work.}
\end{filecontents*}
\documentclass{article}
\usepackage{copmini62za}
\begin{document}
A note: \equalcontrib{1,2}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">A note: <sup xml:id=\"p1.1.1\">1,2</sup>These authors contributed equally to this work.</p>",
    )],
  );
}

/// 62zb: aa.cls loads amsmath (`\RequirePackage[tbtags,fleqn]{amsmath}`), whose {pmatrix}/{cases} start with
/// `\matrix@check` (amsmath.sty:1077-1082): the environment when `\@currenvir` names it, the old Plain-TeX form
/// otherwise. Perl's aa binding keeps only the plain form (for the old aa.cls without amsmath, astro-ph/0002145), so
/// `\begin{cases}` read its first cell as an argument (2609.02726, 2609.04318; Perl errors the same way). pdflatex
/// (the paper's aa.cls): the two environments; the plain `\pmatrix{…}` row is the old class's form.
#[test]
fn aa_cases_and_pmatrix_take_the_environment_form() {
  assert_elements(
    r"\documentclass{aa}
\begin{document}
\begin{equation}
E=\begin{cases}1, & x>0,\\ 0, & \text{otherwise}.\end{cases}
\end{equation}
\begin{equation}
A=\begin{pmatrix} 1 & 2\\ 3 & 4\end{pmatrix}
\end{equation}
\begin{equation}
B=\pmatrix{1 & 2\cr 3 & 4}
\end{equation}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "equation",
        "S0.E1",
        "<equation xml:id=\"S0.E1\"><tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags><Math mode=\"display\" tex=\"E=\\begin{cases}1,&amp;x&gt;0,\\\\&#10;0,&amp;\\text{otherwise}.\\end{cases}\" text=\"E = cases@(1, x &gt; 0, 0, [otherwise])\" xml:id=\"S0.E1.m1\"><XMath xml:id=\"S0.E1.m1.5a\"><XMApp xml:id=\"S0.E1.m1.5a.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">E</XMTok><XMDual xml:id=\"S0.E1.m1.5a.1.3\"><XMApp xml:id=\"S0.E1.m1.5a.1.3.1\"><XMTok meaning=\"cases\"/><XMRef idref=\"S0.E1.m1.1\" xml:id=\"S0.E1.m1.5a.1.3.1.2\"/><XMRef idref=\"S0.E1.m1.3\" xml:id=\"S0.E1.m1.5a.1.3.1.3\"/><XMRef idref=\"S0.E1.m1.5\" xml:id=\"S0.E1.m1.5a.1.3.1.4\"/><XMRef idref=\"S0.E1.m1.7\" xml:id=\"S0.E1.m1.5a.1.3.1.5\"/></XMApp><XMWrap xml:id=\"S0.E1.m1.5a.1.3.2\"><XMTok role=\"OPEN\" stretchy=\"true\">{</XMTok><XMArray xml:id=\"S0.E1.m1.5a.1.3.2.2\"><XMRow xml:id=\"S0.E1.m1.5a.1.3.2.2.1\"><XMCell align=\"left\" xml:id=\"S0.E1.m1.5a.1.3.2.2.1.1\"><XMDual xml:id=\"S0.E1.m1.1\"><XMRef idref=\"S0.E1.m1.2\" xml:id=\"S0.E1.m1.1.2\"/><XMWrap xml:id=\"S0.E1.m1.1.3\"><XMTok meaning=\"1\" role=\"NUMBER\" xml:id=\"S0.E1.m1.2\">1</XMTok><XMTok role=\"PUNCT\">,</XMTok></XMWrap></XMDual></XMCell><XMCell align=\"left\" xml:id=\"S0.E1.m1.5a.1.3.2.2.1.2\"><XMDual xml:id=\"S0.E1.m1.3\"><XMRef idref=\"S0.E1.m1.4\" xml:id=\"S0.E1.m1.3.2\"/><XMWrap xml:id=\"S0.E1.m1.3.3\"><XMApp xml:id=\"S0.E1.m1.4\"><XMTok meaning=\"greater-than\" role=\"RELOP\">&gt;</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok meaning=\"0\" role=\"NUMBER\">0</XMTok></XMApp><XMTok role=\"PUNCT\">,</XMTok></XMWrap></XMDual></XMCell></XMRow><XMRow xml:id=\"S0.E1.m1.5a.1.3.2.2.2\"><XMCell align=\"left\" xml:id=\"S0.E1.m1.5a.1.3.2.2.2.1\"><XMDual xml:id=\"S0.E1.m1.5\"><XMRef idref=\"S0.E1.m1.6\" xml:id=\"S0.E1.m1.5.2\"/><XMWrap xml:id=\"S0.E1.m1.5.3\"><XMTok meaning=\"0\" role=\"NUMBER\" xml:id=\"S0.E1.m1.6\">0</XMTok><XMTok role=\"PUNCT\">,</XMTok></XMWrap></XMDual></XMCell><XMCell align=\"left\" xml:id=\"S0.E1.m1.5a.1.3.2.2.2.2\"><XMDual xml:id=\"S0.E1.m1.7\"><XMRef idref=\"S0.E1.m1.8\" xml:id=\"S0.E1.m1.7.2\"/><XMWrap xml:id=\"S0.E1.m1.7.3\"><XMText xml:id=\"S0.E1.m1.8\">otherwise</XMText><XMTok role=\"PERIOD\">.</XMTok></XMWrap></XMDual></XMCell></XMRow></XMArray></XMWrap></XMDual></XMApp></XMath></Math></equation>",
      ),
      (
        "equation",
        "S0.E2",
        "<equation xml:id=\"S0.E2\"><tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags><Math mode=\"display\" tex=\"A=\\begin{pmatrix}1&amp;2\\\\&#10;3&amp;4\\end{pmatrix}\" text=\"A = matrix@(Array[[1, 2], [3, 4]])\" xml:id=\"S0.E2.m1\"><XMath xml:id=\"S0.E2.m1.2\"><XMApp xml:id=\"S0.E2.m1.2.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">A</XMTok><XMDual xml:id=\"S0.E2.m1.2.1.3\"><XMApp xml:id=\"S0.E2.m1.2.1.3.1\"><XMTok meaning=\"matrix\"/><XMRef idref=\"S0.E2.m1.1\" xml:id=\"S0.E2.m1.2.1.3.1.2\"/></XMApp><XMWrap xml:id=\"S0.E2.m1.2.1.3.2\"><XMTok role=\"OPEN\" stretchy=\"true\">(</XMTok><XMArray xml:id=\"S0.E2.m1.1\"><XMRow xml:id=\"S0.E2.m1.1.1\"><XMCell align=\"center\" xml:id=\"S0.E2.m1.1.1.1\"><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMCell><XMCell align=\"center\" xml:id=\"S0.E2.m1.1.1.2\"><XMTok meaning=\"2\" role=\"NUMBER\">2</XMTok></XMCell></XMRow><XMRow xml:id=\"S0.E2.m1.1.2\"><XMCell align=\"center\" xml:id=\"S0.E2.m1.1.2.1\"><XMTok meaning=\"3\" role=\"NUMBER\">3</XMTok></XMCell><XMCell align=\"center\" xml:id=\"S0.E2.m1.1.2.2\"><XMTok meaning=\"4\" role=\"NUMBER\">4</XMTok></XMCell></XMRow></XMArray><XMTok role=\"CLOSE\" stretchy=\"true\">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></equation>",
      ),
      (
        "equation",
        "S0.E3",
        "<equation xml:id=\"S0.E3\"><tags><tag>(3)</tag><tag role=\"refnum\">3</tag></tags><Math mode=\"display\" tex=\"B=\\pmatrix{1&amp;2\\cr 3&amp;4}\" text=\"B = matrix@(Array[[1, 2], [3, 4]])\" xml:id=\"S0.E3.m1\"><XMath xml:id=\"S0.E3.m1.2\"><XMApp xml:id=\"S0.E3.m1.2.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">B</XMTok><XMDual xml:id=\"S0.E3.m1.2.1.3\"><XMApp xml:id=\"S0.E3.m1.2.1.3.1\"><XMTok meaning=\"matrix\"/><XMRef idref=\"S0.E3.m1.1\" xml:id=\"S0.E3.m1.2.1.3.1.2\"/></XMApp><XMWrap xml:id=\"S0.E3.m1.2.1.3.2\"><XMTok role=\"OPEN\" stretchy=\"true\">(</XMTok><XMArray xml:id=\"S0.E3.m1.1\"><XMRow xml:id=\"S0.E3.m1.1.1\"><XMCell align=\"center\" xml:id=\"S0.E3.m1.1.1.1\"><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMCell><XMCell align=\"center\" xml:id=\"S0.E3.m1.1.1.2\"><XMTok meaning=\"2\" role=\"NUMBER\">2</XMTok></XMCell></XMRow><XMRow xml:id=\"S0.E3.m1.1.2\"><XMCell align=\"center\" xml:id=\"S0.E3.m1.1.2.1\"><XMTok meaning=\"3\" role=\"NUMBER\">3</XMTok></XMCell><XMCell align=\"center\" xml:id=\"S0.E3.m1.1.2.2\"><XMTok meaning=\"4\" role=\"NUMBER\">4</XMTok></XMCell></XMRow></XMArray><XMTok role=\"CLOSE\" stretchy=\"true\">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></equation>",
      ),
    ],
  );
}

/// 62zb: `\pdfstartlink [rule spec] [attr spec] action spec` consumes its spec (pdfTeX manual §8.12), as `\pdfdest`
/// and `\pdfoutline` do: a bare no-op left `attr {…} goto name {impact.sec:in_tro}` in the text, its `_` an error
/// (AAAI papers, which may not load hyperref, link so: 2609.00161). pdflatex: "See Section ?? and a link."
#[test]
fn pdfstartlink_consumes_its_spec() {
  assert_elements(
    r"\documentclass{article}
\begin{document}
\section{Intro}\label{sec:in_tro}
\pdfdest name {impact.sec:in_tro} xyz\relax
See \pdfstartlink attr {/Border [0 0 0]} goto name {impact.sec:in_tro}\relax Section~\ref{sec:in_tro}\pdfendlink{} and
\pdfstartlink height 2pt depth 1pt user {/Subtype /Link /A << /S /URI /URI (https://x.org/a_b) >>}\relax a link\pdfendlink.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "p",
      "S1.p1.1",
      "<p xml:id=\"S1.p1.1\">See Section <ref labelref=\"LABEL:sec:in_tro\"/> and\na link.</p>",
    )],
  );
}

/// 62zb: a file kpathsea does not find is looked for under each `{<dir>}` of `\input@path`, a `/` added to an entry
/// lacking one (l3file's `\file_full_name:n`, expl3-code.tex:12585-12612; content.rs `find_on_input_path`): a paper
/// keeping its style in `styles/` (2609.02998, whose natbib every `\citep` needed; 71 papers in 2609). pdflatex (with
/// the directory): the style's text, for `{styles/}` and `{styles}` alike.
#[test]
fn input_path_finds_a_style_in_a_subdirectory() {
  assert_elements(
    r"\begin{filecontents*}[overwrite]{styles/style62zb.sty}
\ProvidesPackage{style62zb}
\newcommand\fromstyle{Defined in the styles directory.}
\end{filecontents*}
\documentclass{article}
\makeatletter
\def\input@path{{styles/}}
\makeatother
\usepackage{style62zb}
\begin{document}
\fromstyle
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">Defined in the styles directory.</p>",
    )],
  );
  // an entry without its `/`
  assert_elements(
    r"\begin{filecontents*}[overwrite]{styles/style62zc.sty}
\ProvidesPackage{style62zc}
\newcommand\fromstyle{Defined in the styles directory.}
\end{filecontents*}
\documentclass{article}
\makeatletter
\def\input@path{{styles}}
\makeatother
\usepackage{style62zc}
\begin{document}
\fromstyle
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">Defined in the styles directory.</p>",
    )],
  );
}

/// 62zc: optica-article's binding loads what the class file loads (its own dependency scan, as sn-jnl's): array's
/// `>{$}l<{$}`, tabularx and multirow were missing (2609.00899, 2609.05706, 2609.06191, 2609.10235). soul, the `\else`
/// arm of the class's `\ifpdf` (optica-article.cls:46-54), is not loaded, so the paper's own `\st` stays its own. The
/// class here is a filecontents stand-in for the shipped one (:33 `\RequirePackage{tabularx,multirow,array}`).
/// pdflatex: the two-row table, then "a + s_t".
#[test]
fn optica_article_loads_its_class_packages() {
  assert_elements(
    r"\begin{filecontents*}[overwrite]{optica-article.cls}
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{optica-article}
\LoadClass{article}
\RequirePackage{tabularx,multirow,array}
\RequirePackage{ifpdf}
\ifpdf\RequirePackage{microtype}\else\RequirePackage{microtype}\RequirePackage{soul}\fi
\end{filecontents*}
\documentclass{optica-article}
\newcommand{\st}{s_t}
\begin{document}
\begin{tabularx}{\textwidth}{>{$}l<{$} X}
\multirow{2}{*}{x} & First \\
 & Second \\
\end{tabularx}

Then $a+\st$.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "tabular",
        "p1.1",
        "<tabular class=\"ltx_guessed_headers\" vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr xml:id=\"p1.1.1\"><td align=\"left\" rowspan=\"2\" thead=\"row\" xml:id=\"p1.1.1.1\"><text class=\"ltx_markedasmath\" xml:id=\"p1.1.1.1.1\">x</text></td><td align=\"left\" xml:id=\"p1.1.1.2\"><inline-block vattach=\"top\" xml:id=\"p1.1.1.2.1\"><p xml:id=\"p1.1.1.2.1.1\">First</p></inline-block></td></tr><tr xml:id=\"p1.1.2\"><td align=\"left\" xml:id=\"p1.1.2.1\"><inline-block vattach=\"top\" xml:id=\"p1.1.2.1.1\"><p xml:id=\"p1.1.2.1.1.1\">Second</p></inline-block></td></tr></tbody></tabular>",
      ),
      (
        "p",
        "p2.1",
        "<p xml:id=\"p2.1\">Then <Math mode=\"inline\" tex=\"a+s_{t}\" text=\"a + s _ t\" xml:id=\"p2.m1\"><XMath xml:id=\"p2.m1.1\"><XMApp xml:id=\"p2.m1.1.1\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMApp xml:id=\"p2.m1.1.1.3\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">s</XMTok><XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">t</XMTok></XMApp></XMApp></XMath></Math>.</p>",
      ),
    ],
  );
}

/// 62zc: cas-common.sty's name parsers, e-mail/URL/ORCID/first-page-note printers and page styles are defined (as
/// no-ops: this binding's frontmatter keeps the e-mails and notes itself), so a paper's own `\RenewDocumentCommand` of
/// them, or its `\ps@cas`, no longer errors — ltcmd refuses to renew an undefined command (2609.16168, 16199, 36345,
/// 00634, 20010); and its title-page layout keys exist (`\keys_set:nn {stm/mktitle}{nologo}`, cas-common.sty:1723,
/// 2609.00281). pdflatex (TL's cas-dc.cls): the title page and "Body."
#[test]
fn cas_common_helpers_can_be_renewed() {
  assert_elements(
    r"\documentclass{cas-dc}
\ExplSyntaxOn
\RenewDocumentCommand \firstname {} { \seq_use:Nn \l_stm_au_seq { ~ } }
\RenewDocumentCommand \emailauthor { m m } { #1 }
\RenewDocumentCommand \printorcid { } { }
\keys_set:nn { stm / mktitle } { nologo }
\ExplSyntaxOff
\begin{document}
\title [mode = title]{A Title}
\author{A. Author}
\maketitle
\makeatletter\ps@cas\makeatother
Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Body.</p>")],
  );
}

/// 62zd: the box capture's backmatter lift stops at a float, as at an alignment (base_utilities.rs `insert_block`): an
/// appendix section in a minipage in a `table` was lifted past it, leaving the table empty and its caption at the
/// document's top level, with no diagnostic. It stays in the table now, and errors there as Perl's does (a sectioning
/// unit in a float errors, OXIDIZED_DESIGN_DIVERGENCES #189 ruling 2026-10-04; 2609.05763's glossary). pdflatex: the
/// heading, the table and its caption.
#[test]
fn appendix_in_a_minipage_stays_in_its_float() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\section{Intro}
Text.
\appendix
\section{First}
Body.
\begin{table}[h]
\begin{minipage}{\textwidth}
\section{Notation Table}\label{sec:tab}
\centering
\begin{tabular}{cc} a & b \\ \end{tabular}
\end{minipage}
\caption{Notation.}
\end{table}
\end{document}",
    Some("ar5iv.sty"),
  );
  // the schema-invalid appendix-in-block Perl reports too: exactly that error and the capture's warning
  assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
  for message in [
    "Error:malformed:ltx:appendix <ltx:appendix> isn't allowed in <ltx:block>",
    "Warning:malformed:_CaptureBlock_ Did not find a block-like candidate in ltx:table",
  ] {
    assert!(
      log.lines().any(|l| l.starts_with(message)),
      "{message}\n{log}"
    );
  }
  assert_element(
    &xml,
    "table",
    &["xml:id=\"A2.T1\""],
    "<table inlist=\"lot\" placement=\"h\" xml:id=\"A2.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><block class=\"ltx_minipage\" vattach=\"middle\" width=\"345.0pt\" xml:id=\"A2.T1.1\"><appendix inlist=\"toc\" labels=\"LABEL:sec:tab\" xml:id=\"A2\"><tags><tag>Appendix B</tag><tag role=\"refnum\">B</tag><tag role=\"typerefnum\">Appendix B</tag></tags><title><tag close=\" \">Appendix B</tag>Notation Table</title><toctitle><tag close=\" \">B</tag>Notation Table</toctitle><para align=\"center\" xml:id=\"A2.p1\"><tabular vattach=\"middle\" xml:id=\"A2.p1.1\"><tbody><tr xml:id=\"A2.p1.1.1\"><td align=\"center\" xml:id=\"A2.p1.1.1.1\">a</td><td align=\"center\" xml:id=\"A2.p1.1.1.2\">b</td></tr></tbody></tabular></para></appendix></block><toccaption><tag close=\" \">1</tag>Notation.</toccaption><caption><tag close=\": \">Table 1</tag>Notation.</caption></table>",
  );
}

/// 62zd: rasti.cls (RAS Techniques and Instruments) is mnras.cls under another name (v3.0 defines the same commands)
/// and loads as the mnras binding (rasti_cls.rs): raw, or as OmniBus without its file, `\newauthor` was undefined and
/// the author block's groups unbalanced (2609.08700, 2609.09329; 18 papers in 2609 at 0 errors).
#[test]
fn rasti_loads_as_mnras() {
  let xml = assert_elements(
    r"\documentclass[fleqn,usenatbib]{rasti}
\title{A Title}
\author[A. Author et al.]{A. Author,$^{1}$ B. Author$^{2}$
\newauthor C. Author$^{1}$
\\
$^{1}$First Institute\\
$^{2}$Second Institute}
\pubyear{2026}
\begin{document}
\maketitle
Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Body.</p>")],
  );
  assert_element(
    &xml,
    "pubnote",
    &["role=\"pubyear\""],
    "<pubnote role=\"pubyear\">2026</pubnote>",
  );
}

/// 62ze: mnras.cls's own settings the binding lacked: its enumerate labels (:930-940, "(i)"), its bibliography heading
/// "REFERENCES" (:1312, 1339), with `usenatbib` its punctuation (:1336 `\bibpunct{(}{)}{;}{a}{}{,}`, "(Draine 2011)"),
/// and with `usedcolumn` its `d` column on dcolumn (:1349-1354); the mn binding declared that option as `usedcolum`
/// (Perl's typo, KNOWN_PERL_ERRORS #500). Witnesses 2609.09329, 2609.08700 (rasti, which loads as mnras). pdflatex:
/// "(i) First. (ii) Second.", the table, "REFERENCES".
#[test]
fn mnras_lists_citations_and_columns() {
  let xml = assert_elements(
    r"\documentclass[usenatbib,usedcolumn]{mnras}
\title{A Title}
\author{A. Author}
\begin{document}
\maketitle
\begin{enumerate}
\item First.\label{it:a}
\item Second.
\end{enumerate}
See item~\ref{it:a} and \citet{dr11} \citep{dr11}.
\begin{tabular}{d{2}}
1.5 \\
\end{tabular}
\begin{thebibliography}{}
\bibitem[Draine(2011)]{dr11} Draine B. T., 2011, Physics of the ISM.
\end{thebibliography}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "enumerate",
        "S0.I1",
        "<enumerate xml:id=\"S0.I1\"><item labels=\"LABEL:it:a\" xml:id=\"S0.I1.i1\"><tags><tag>(i)</tag><tag role=\"autoref\">item (i)<text xml:id=\"S0.I1.i1.1\"/></tag><tag role=\"refnum\">(i)</tag><tag role=\"typerefnum\">item (i)</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p xml:id=\"S0.I1.i1.p1.1\">First.</p></para></item><item xml:id=\"S0.I1.i2\"><tags><tag>(ii)</tag><tag role=\"autoref\">item (ii)<text xml:id=\"S0.I1.i2.1\"/></tag><tag role=\"refnum\">(ii)</tag><tag role=\"typerefnum\">item (ii)</tag></tags><para xml:id=\"S0.I1.i2.p1\"><p xml:id=\"S0.I1.i2.p1.1\">Second.</p></para></item></enumerate>",
      ),
      (
        "tabular",
        "p1.1.1",
        "<tabular vattach=\"middle\" xml:id=\"p1.1.1\"><tbody><tr xml:id=\"p1.1.1.1\"><td align=\"char:.\" xml:id=\"p1.1.1.1.1\"><Math mode=\"inline\" tex=\"1.5\" text=\"1.5\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMTok meaning=\"1.5\" role=\"NUMBER\">1.5</XMTok></XMath></Math></td></tr></tbody></tabular>",
      ),
    ],
  );
  // `\bibpunct{(}{)}{;}{a}{}{,}`: no comma between author and year ("(Draine 2011)")
  assert_element(
    &xml,
    "cite",
    &["class=\"ltx_citemacro_citep\""],
    "<cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"dr11\" separator=\";\" show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase> </bibrefphrase></bibref>)</cite>",
  );
  assert_element(&xml, "bibliography", &["xml:id=\"bib\""], "<bibliography inlist=\"toc\" xml:id=\"bib\"><title>REFERENCES</title><biblist><bibitem key=\"dr11\" xml:id=\"bib.bib1\"><tags><tag role=\"number\">1</tag><tag role=\"year\">2011</tag><tag role=\"authors\">Draine</tag><tag role=\"refnum\">Draine (2011)</tag><tag role=\"key\">dr11</tag></tags><bibblock> Draine B. T., 2011, Physics of the ISM.
</bibblock></bibitem></biblist></bibliography>");
}

/// 62zf: elsarticle.cls:40-45's conditionals (`\iflongmktitle`, `\ifdoubleblind`, …), set by its options: an Elsevier
/// journal style (jasr.sty, cnf.sty, jcomp.sty) tests them inside the `\if@twocolumn` branches of its `\maketitle`, and
/// undefined they unbalanced TeX's skip of the false branch, so the other branch's body ran at load
/// (`\finalMaketitle` undefined). Also the class's `\emailauthor{<email>}{<name>}` and `\urlauthor` (:211-235, frontmatter
/// notes "email (name)", as the first page prints them) and the styles' `\KWD` (defined in their `\keyword`, which the binding's
/// {keyword} bypasses). Witnesses 2609.23725, 23732, 27838, 39431, 09773, 31741, 39502. pdflatex: the title block,
/// "first; second", "Body.".
#[test]
fn elsarticle_journal_styles_load() {
  let xml = assert_elements(
    r"\begin{filecontents*}[overwrite]{jstyle62zf.sty}
\if@twocolumn
  \def\maketitle{\iflongmktitle\getSpaceLeft\else\twocolumn[\finalMaketitle]\printFirstPageNotes\fi}
\else
  \def\maketitle{\iflongmktitle\getSpaceLeft\else\finalMaketitle\printFirstPageNotes\fi}
\fi
\long\def\finalMaketitle{\MaketitleBox}
\def\printFirstPageNotes{}
\def\keyword{\def\KWD{\par\noindent Keywords:~}\global\setbox\keybox=\vbox\bgroup\hsize=\textwidth\noindent\ignorespaces}
\def\endkeyword{\egroup}
\end{filecontents*}
\documentclass[times,authoryear]{elsarticle}
\usepackage{jstyle62zf}
\begin{document}
\begin{frontmatter}
\title{A Title}
\author{A. Author}
\emailauthor{a.author@example.org}{A. Author}
\urlauthor{https://example.org}{A. Author}
\begin{keyword}
\KWD first \sep second
\end{keyword}
\end{frontmatter}
Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "note",
        "id1",
        "<note role=\"email\" xml:id=\"id1\"><text font=\"typewriter\" xml:id=\"id1.1\">a.author@example.org</text> (A. Author)</note>",
      ),
      (
        "note",
        "id2",
        "<note role=\"url\" xml:id=\"id2\"><text font=\"typewriter\" xml:id=\"id2.1\">https://example.org</text> (A. Author)</note>",
      ),
      ("p", "p1.1", "<p xml:id=\"p1.1\">Body.</p>"),
    ],
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords: \">\nfirst, second\n</keywords>",
  );
}

/// 62zf: elsarticle's `nonatbib` option sets `\ifnonatbib` (elsarticle.cls:80) and leaves natbib out (:1242-1244), and
/// `\biboptions` then selects nothing: those papers load biblatex or apacite (2609.05849, 19225, 20719, 23461, 26003).
/// pdflatex: "NN:YES; none.".
#[test]
fn elsarticle_nonatbib_leaves_natbib_out() {
  assert_elements(
    r"\documentclass[nonatbib]{elsarticle}
\biboptions{sort}
\makeatletter
\begin{document}
NN:\ifnonatbib YES\else NO\fi; \@ifpackageloaded{natbib}{natbib}{none}.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">NN:YES; none.</p>")],
  );
}

/// 62w: the ar5iv profile's `iflimit` reaches the engine (ar5iv_sty.rs → latexml.sty's keyval, `set_if_limit`): 48M,
/// which finite pgfplots/mhchem papers need (2609.07725 counts 39M conditionals, 2609.10563 19M; 2605.27177 converts).
/// Read on the conversion thread before its engine is released.
#[test]
fn ar5iv_profile_sets_its_runaway_limits() {
  let (log, _xml, limit) = convert_with_then(
    "\\documentclass{article}\n\\begin{document}\nText.\n\\end{document}\n",
    Some("ar5iv.sty"),
    |_| latexml_core::state::if_limit(),
  );
  assert_eq!(
    (error_count(&log), warning_count(&log), limit),
    (0, 0, 48_000_000),
    "{log}"
  );
}

/// 62t: the 2609 classes load what they load and define what they define (bindings stand in for the class, so its
/// packages load from them or not at all): lipics-v2021.cls:514-1098's packages and its `{CCSXML}` exclusion
/// (2609.13401, 2609.10114, 2609.13485), fairmeta.cls:17-69's (2609.11172), WileyNJDv5.cls:357-376/413/620-621's (2609.06025),
/// bmvc2k.cls:149's xcolor (a paper's own `[table]` still loading colortbl) and its `\addauthor` mail set in sans under
/// the class's T1, `_` and all (2609.06007), lmcs.cls:696-761's theorem set with its `thmC` styles (2609.11893),
/// informs3.cls's `\TheoremsNumberedThrough` and e-companion `\ECSwitch` renumbering sections, theorems and equations
/// (2609.08001),
/// MnSymbol's `\llangle`/`\rrangle` (2609.07645; the binding warns that it is a stub); and `\tracingmacros` stored as
/// a number, which `\the` reads back (Perl TeX_Debugging.pool.ltxml:214-225; arabtex's aedpatch.sty:29-30, 2609.25833).
#[test]
fn the_2609_classes_load_their_packages_and_commands() {
  let loaded = |class: &str, probe: &str| {
    format!(
      "\\documentclass{{{class}}}\n\\begin{{document}}\n\\makeatletter\nP:{probe}\n\\makeatother\n\\end{{document}}"
    )
  };
  let cases: Vec<(&str, &str, String, &str, usize, &str)> = vec![
    (
      "p",
      "p1.1",
      loaded(
        "lipics-v2021",
        r"\@ifpackageloaded{array}{a}{}\@ifpackageloaded{subcaption}{s}{}\@ifpackageloaded{comment}{c}{}\@ifpackageloaded{multirow}{m}{}\@ifpackageloaded{tabularx}{x}{}
\begin{CCSXML}
<ccs2012><concept_id>10003752</concept_id></ccs2012>
\end{CCSXML}",
      ),
      r#"<p xml:id="p1.1">P:ascmx</p>"#,
      0,
      "",
    ),
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{lipics-v2021}\begin{document}
Y:\textcolor{lipicsYellow}{y}
\end{document}",
      ),
      r##"<p xml:id="p1.1">Y:<text color="#FCC712" xml:id="p1.1.1">y</text></p>"##,
      0,
      "",
    ),
    (
      "tabular",
      "p1.1",
      String::from(
        r"\documentclass{lipics-v2021}\usepackage[table]{xcolor}\begin{document}
\begin{tabular}{m{1cm}}\rowcolor{lipicsYellow} a\end{tabular}
\end{document}",
      ),
      "<tabular vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr backgroundcolor=\"#FCC712\" xml:id=\"p1.1.1\"><td align=\"left\" vattach=\"middle\" xml:id=\"p1.1.1.1\"><inline-block backgroundcolor=\"#FCC712\" vattach=\"middle\" width=\"28.5pt\" xml:id=\"p1.1.1.1.1\"><p xml:id=\"p1.1.1.1.1.1\">a</p></inline-block></td></tr></tbody></tabular>",
      0,
      "",
    ),
    (
      "p",
      "p1.1",
      loaded(
        "fairmeta",
        r"\@ifpackageloaded{placeins}{p}{}\@ifpackageloaded{titlesec}{t}{}\@ifpackageloaded{subcaption}{s}{}\@ifpackageloaded{setspace}{h}{}",
      ),
      r#"<p xml:id="p1.1">P:ptsh</p>"#,
      0,
      "",
    ),
    (
      "p",
      "p1.1",
      loaded(
        "WileyNJDv5",
        r"\@ifpackageloaded{multirow}{m}{}\@ifpackageloaded{caption}{c}{}\@ifpackageloaded{tabularx}{x}{}\@ifpackageloaded{dcolumn}{d}{}",
      ),
      r#"<p xml:id="p1.1">P:mcxd</p>"#,
      0,
      "",
    ),
    (
      "creator",
      "",
      String::from(
        r"\documentclass{bmvc2k}\title{T}\addauthor{Ann Lee}{ann_lee@x.org}{1}\addinstitution{Inst}
\begin{document}\maketitle
C:\textcolor{red}{R}
\end{document}",
      ),
      "<creator role=\"author\"><personname>Ann Lee</personname><contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"sansserif\" xml:id=\"id1\">ann_lee@x.org</text></contact></creator>",
      0,
      "",
    ),
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{bmvc2k}\title{T}\addauthor{Ann Lee}{ann_lee@x.org}{1}\addinstitution{Inst}
\begin{document}\maketitle
C:\textcolor{red}{R}
\end{document}",
      ),
      r##"<p xml:id="p1.1">C:<text color="#FF0000" xml:id="p1.1.1">R</text></p>"##,
      0,
      "",
    ),
    (
      "theorem",
      "S1.Thmthm2",
      String::from(
        r"\documentclass{lmcs}\begin{document}\section{S}
\begin{defi}A definition.\end{defi}
\begin{exa}An example.\end{exa}
\end{document}",
      ),
      "<theorem class=\"ltx_theorem_exa\" inlist=\"thm theorem:exa\" xml:id=\"S1.Thmthm2\"><tags><tag>Example 1.2</tag><tag role=\"autoref\">\u{a0}1.2<text xml:id=\"S1.Thmthm2.1\"/></tag><tag role=\"refnum\">1.2</tag><tag role=\"typerefnum\">Example 1.2</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S1.Thmthm2.2\">Example 1.2</text></tag><text font=\"bold\" xml:id=\"S1.Thmthm2.3\">.</text></title><para xml:id=\"S1.Thmthm2.p1\"><p xml:id=\"S1.Thmthm2.p1.1\">An example.</p></para></theorem>",
      0,
      "",
    ),
    (
      "section",
      "S1",
      String::from(
        r"\documentclass{informs3}\TheoremsNumberedThrough\begin{document}
\begin{assumption}A1.\end{assumption}
\ECSwitch
\section{Proofs}
B.
\end{document}",
      ),
      r#"<section inlist="toc" xml:id="S1"><tags><tag>EC.1</tag><tag role="refnum">EC.1</tag><tag role="typerefnum">§EC.1</tag></tags><title><tag close=" ">EC.1</tag>Proofs</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">B.</p></para></section>"#,
      0,
      "",
    ),
    (
      "tabular",
      "p1.1",
      String::from(
        r"\documentclass{bmvc2k}\usepackage[table]{xcolor}\title{T}\addauthor{Ann Lee}{ann_lee@x.org}{1}
\addinstitution{Inst}
\begin{document}\maketitle
\begin{tabular}{m{1cm}}\rowcolor{red} a\end{tabular}
\end{document}",
      ),
      "<tabular vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr backgroundcolor=\"#FF0000\" xml:id=\"p1.1.1\"><td align=\"left\" vattach=\"middle\" xml:id=\"p1.1.1.1\"><inline-block backgroundcolor=\"#FF0000\" vattach=\"middle\" width=\"28.5pt\" xml:id=\"p1.1.1.1.1\"><p xml:id=\"p1.1.1.1.1.1\">a</p></inline-block></td></tr></tbody></tabular>",
      0,
      "",
    ),
    (
      "theorem",
      "Thmtheorem1",
      String::from(
        r"\documentclass{informs3}\TheoremsNumberedThrough\EquationsNumberedThrough\begin{document}
\ECSwitch
\section{Proofs}
\begin{theorem}T.\end{theorem}
\begin{equation}x=1\end{equation}
\end{document}",
      ),
      "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" xml:id=\"Thmtheorem1\"><tags><tag>Theorem EC.1</tag><tag role=\"refnum\">EC.1</tag><tag role=\"typerefnum\">Theorem EC.1</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"Thmtheorem1.1\">Theorem EC.1</text></tag><text font=\"bold\" xml:id=\"Thmtheorem1.2\">.</text></title><para xml:id=\"Thmtheorem1.p1\"><p xml:id=\"Thmtheorem1.p1.1\"><text font=\"italic\" xml:id=\"Thmtheorem1.p1.1.1\">T.</text></p></para></theorem>",
      0,
      "",
    ),
    (
      "equation",
      "S1.E1",
      String::from(
        r"\documentclass{informs3}\TheoremsNumberedThrough\EquationsNumberedThrough\begin{document}
\ECSwitch
\section{Proofs}
\begin{equation}x=1\end{equation}
\end{document}",
      ),
      "<equation xml:id=\"S1.E1\"><tags><tag>(EC.1)</tag><tag role=\"refnum\">EC.1</tag></tags><Math mode=\"display\" tex=\"x=1\" text=\"x = 1\" xml:id=\"S1.E1.m1\"><XMath xml:id=\"S1.E1.m1.1\"><XMApp xml:id=\"S1.E1.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math></equation>",
      0,
      "",
    ),
    (
      "theorem",
      "S1.Thmthm1",
      String::from(
        r"\documentclass{lmcs}\begin{document}\section{S}
\begin{thmC}[Smith]A theorem.\end{thmC}
$a\coloneqq b$
\end{document}",
      ),
      // (The note keeps its parentheses, which `thmC`'s head spec drops: the amsthm binding ignores head specs.)
      "<theorem class=\"ltx_theorem_thmC\" inlist=\"thm theorem:thmC\" xml:id=\"S1.Thmthm1\"><tags><tag>Theorem 1.1</tag><tag role=\"autoref\">\u{a0}1.1<text xml:id=\"S1.Thmthm1.1\"/></tag><tag role=\"refnum\">1.1</tag><tag role=\"typerefnum\">Theorem 1.1</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S1.Thmthm1.2\">Theorem 1.1</text></tag><text font=\"bold\" xml:id=\"S1.Thmthm1.3\"> </text>(Smith)<text font=\"bold\" xml:id=\"S1.Thmthm1.4\">.</text></title><para xml:id=\"S1.Thmthm1.p1\"><p xml:id=\"S1.Thmthm1.p1.1\"><text font=\"italic\" xml:id=\"S1.Thmthm1.p1.1.1\">A theorem.</text></p></para></theorem>",
      0,
      "",
    ),
    (
      "XMWrap",
      "p1.m1.2.1.2",
      String::from(
        r"\documentclass{article}\usepackage{MnSymbol}\begin{document}
$\llangle x\rrangle$
\end{document}",
      ),
      "<XMWrap xml:id=\"p1.m1.2.1.2\"><XMTok name=\"llangle\" role=\"OPEN\" stretchy=\"false\">\u{27ea}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.1\">x</XMTok><XMTok name=\"rrangle\" role=\"CLOSE\" stretchy=\"false\">\u{27eb}</XMTok></XMWrap>",
      1,
      "MnSymbol.sty is only minimally stubbed",
    ),
    (
      "p",
      "p1.1",
      String::from(
        r"\documentclass{article}\begin{document}
\tracingmacros=0 \tracingcommands=0
\edef\x{\the\tracingmacros/\the\tracingcommands}
T:[\x]
\end{document}",
      ),
      r#"<p xml:id="p1.1">T:[0/0]</p>"#,
      0,
      "",
    ),
  ];
  for (tag, id, tex, element, warnings, message) in cases {
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{tex}\n{log}"
    );
    assert!(log.contains(message), "{tex}\n{log}");
    if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
      assert_eq!(lines, 0, "jing:\n{xml}");
    }
    let attrs: Vec<String> = if id.is_empty() {
      vec![]
    } else {
      vec![format!("xml:id=\"{id}\"")]
    };
    let attrs: Vec<&str> = attrs.iter().map(String::as_str).collect();
    assert_element(&xml, tag, &attrs, element);
  }
}

/// 62s: a neurips style's notice names the year it was requested as (neurips_2026.sty:391-403), for papers whose own
/// `\@maketitle` prints it — under a directory (`Styles/neurips_2026`, 2609.20831), with a suffix
/// (`neurips_2025_custom`), 2025's `dandb` track wording, 2016-2021 with the location, a year's preprint and submission
/// text — and no venue note of the binding's own: the paper's copy of the style decides what its first page prints
/// (DIVERGENCES #448; 2609.24814, 2609.15128).
#[test]
fn neurips_notice_names_the_year_of_its_style() {
  let cases: [(&str, &str); 7] = [
    (r"[main,final]{Styles/neurips_2026}", r"\@noticestring"),
    (r"[final,dandb]{neurips_2025}", r"\@noticestring"),
    (
      r"[final]{neurips_2025_custom}",
      r"\@neuripsordinal/\@neuripsyear",
    ),
    (r"[final]{neurips_2020}", r"\@noticestring"),
    (r"[final]{neurips_2016}", r"\@noticestring"),
    (r"[preprint]{neurips_2023}", r"\@noticestring"),
    (r"{neurips_2026}", r"\@noticestring"),
  ];
  let printed = [
    "40th Conference on Neural Information Processing Systems (NeurIPS 2026).",
    "39th Conference on Neural Information Processing Systems (NeurIPS 2025) Track on Datasets and Benchmarks.",
    "39th/2025",
    "34th Conference on Neural Information Processing Systems (NeurIPS 2020), Vancouver, Canada.",
    "30th Conference on Neural Information Processing Systems (NIPS 2016), Barcelona, Spain.",
    "Preprint. Under review.",
    "Submitted to 40th Conference on Neural Information Processing Systems (NeurIPS 2026). Do not distribute.",
  ];
  for ((package, shown), text) in cases.into_iter().zip(printed) {
    let tex = format!(
      "\\documentclass{{article}}\\usepackage{package}\n\\title{{T}}\\author{{A}}\n\\begin{{document}}\\maketitle\n\\makeatletter N:[{shown}]\\makeatother\n\\end{{document}}"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    assert_element(
      &xml,
      "p",
      &[r#"xml:id="p1.1""#],
      &format!(r#"<p xml:id="p1.1">N:[{text}]</p>"#),
    );
    assert!(!xml.contains("<pubnote"), "{tex}\n{xml}");
  }
}

/// 62zg: an author block in any alignment stays whole — Perl's splitter (Base_Utility.pool.ltxml:693) keeps only
/// `{tabular}`, `{minipage}` and `\halign` content whole, so a `tabular*` author was split at its row ends, leaving
/// the alignment open across the pieces (2609.34061, 34965, 39374, 39909; KNOWN_PERL_ERRORS #502). pdflatex: "A" over
/// "X".
#[test]
fn author_in_tabular_star_stays_whole() {
  let xml = assert_elements(
    r"\documentclass{article}
\title{T}
\author{\begin{tabular*}{\textwidth}{l}A \\ X\end{tabular*}}
\begin{document}
\maketitle
Text.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Text.</p>")],
  );
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname><tabular vattach=\"middle\" xml:id=\"id1\"><tbody><tr xml:id=\"id1.1\"><td align=\"left\" xml:id=\"id1.1.1\">A</td></tr><tr xml:id=\"id1.2\"><td align=\"left\" xml:id=\"id1.2.1\">X</td></tr></tbody></tabular></personname></creator>",
  );
}

/// 62zg: `\lstinline{…}` whose argument was tokenized before (inside `\text{…}`) leaves the alignment ledger as it
/// found it: its closing `}` keeps the END catcode the gullet already counted, so retracting the opening `{` as well
/// left the cell at −1 and the row's `\\` was never recognised (2609.04372, 29962). pdflatex: two rows.
#[test]
fn lstinline_in_text_in_align_cell() {
  assert_elements(
    r"\documentclass{article}
\usepackage{amsmath,listings}
\begin{document}
\begin{align*}
  & a \text{\lstinline{x}} \\
  & b
\end{align*}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "XMApp",
        "S0.Ex1.m2.1.1",
        "<XMApp xml:id=\"S0.Ex1.m2.1.1\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMText class=\"ltx_lst_identifier ltx_lstlisting\" xml:id=\"S0.Ex1.m2.1.1.3\">x</XMText></XMApp>",
      ),
      (
        "equation",
        "S0.Ex2",
        "<equation xml:id=\"S0.Ex2\"><MathFork><Math tex=\"\\displaystyle b\" text=\"b\" xml:id=\"S0.Ex2.m2\"><XMath xml:id=\"S0.Ex2.m2.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMath></Math><MathBranch><td xml:id=\"S0.Ex2.1\"/><td align=\"left\" xml:id=\"S0.Ex2.2\"><Math mode=\"inline\" tex=\"\\displaystyle b\" text=\"b\" xml:id=\"S0.Ex2.m1\"><XMath xml:id=\"S0.Ex2.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMath></Math></td></MathBranch></MathFork></equation>",
      ),
    ],
  );
}

/// 62zg: `\captionsetup{type=figure}` is caption's `\caption@settype` (caption.sty:283-313), which sets `\@captype`:
/// a `\subcaptionbox` and a `\caption` in a minipage read it, and unset they built `\c@\@captype` (2609.06557, 32742;
/// KNOWN_PERL_ERRORS #504). The caption becomes the minipage's figure, its list entry with it (OXIDIZED_DESIGN_DIVERGENCES
/// #182). pdflatex: "(a) Left.", "(b) Right.", "Figure 1: Both.".
#[test]
fn captionsetup_type_sets_the_caption_type() {
  assert_elements(
    r"\documentclass{article}
\usepackage{graphicx,caption,subcaption}
\begin{document}
\noindent\begin{minipage}{\textwidth}
\captionsetup{type=figure}\centering
\subcaptionbox{Left.}{\rule{1cm}{1cm}}
\subcaptionbox{Right.}{\rule{1cm}{1cm}}
\caption{Both.}
\end{minipage}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "figure",
        "S0.F1.sf2",
        "<figure align=\"center\" inlist=\"lof\" xml:id=\"S0.F1.sf2\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf2.1\">(b)</text></tag><tag role=\"refnum\">1b</tag></tags><rule height=\"28.5pt\" width=\"28.5pt\"/><toccaption><tag close=\" \">b</tag>Right.</toccaption><caption><tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F1.sf2.2\">(b)</text></tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf2.3\">Right.</text></caption></figure>",
      ),
      (
        "figure",
        "S0.F1",
        "<figure align=\"center\" inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.1\">Figure 1</text></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Both.</toccaption><caption><tag close=\": \"><text fontsize=\"90%\" xml:id=\"S0.F1.2\">Figure 1</text></tag><text fontsize=\"90%\" xml:id=\"S0.F1.3\">Both.</text></caption></figure>",
      ),
    ],
  );
}

/// 62zg: a minipage's `\caption` after `\captionsetup{type=table}` is "Table 2" of the list of tables, as the
/// `\captionof` one before it is "Table 1": the caption becomes the minipage's float, and its list entry — which
/// `\@@toccaption` reaches before that float exists — goes into it (OXIDIZED_DESIGN_DIVERGENCES #182; review of 62zg:
/// the list read "1 Via captionof." twice). pdflatex: List of Tables "1 Via captionof.", "2 Via captionsetup.".
#[test]
fn captionsetup_type_caption_keeps_its_list_entry() {
  assert_elements(
    r"\documentclass{article}
\usepackage{caption}
\begin{document}
\listoftables
\noindent\begin{minipage}{\textwidth}
\centering A\captionof{table}{Via captionof.}
\end{minipage}

\noindent\begin{minipage}{\textwidth}
\captionsetup{type=table}\centering B
\caption{Via captionsetup.}
\end{minipage}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        "<table align=\"center\" inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Via captionof.</toccaption><caption><tag close=\": \">Table 1</tag>Via captionof.</caption></table>",
      ),
      (
        "table",
        "S0.T2",
        "<table align=\"center\" inlist=\"lot\" xml:id=\"S0.T2\"><tags><tag>Table 2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><toccaption><tag close=\" \">2</tag>Via captionsetup.</toccaption><caption><tag close=\": \">Table 2</tag>Via captionsetup.</caption></table>",
      ),
    ],
  );
}

/// 62zg: physics' operators (`\opbraces{ m g o d() }`, `\trigbraces{ m o d() }`, physics.sty:279-304) read no size: a
/// following `\Bigg[` is the paper's own bracket, which the size read took, and then read `[x\Bigg]` as the power, or
/// across a row break (2609.12812, 39468; KNOWN_PERL_ERRORS #503). pdflatex: "a = N exp[x]", "b = ln[y]".
#[test]
fn physics_operator_leaves_a_sized_bracket() {
  assert_elements(
    r"\documentclass{article}
\usepackage{amsmath,physics}
\begin{document}
\begin{align} a &= N\exp\Bigg[ x \Bigg] \\ b &= \ln\big[ y \big] \end{align}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "Math",
        "S0.E1.m2",
        "<Math mode=\"inline\" tex=\"\\displaystyle=N\\exp\\Bigg[x\\Bigg]\" text=\"absent = N * exponential@(x)\" xml:id=\"S0.E1.m2\"><XMath xml:id=\"S0.E1.m2.3\"><XMApp xml:id=\"S0.E1.m2.3.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok meaning=\"absent\"/><XMApp xml:id=\"S0.E1.m2.3.1.3\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">N</XMTok><XMDual xml:id=\"S0.E1.m2.3.1.3.3\"><XMApp xml:id=\"S0.E1.m2.3.1.3.3.1\"><XMRef idref=\"S0.E1.m2.1\" xml:id=\"S0.E1.m2.3.1.3.3.1.1\"/><XMRef idref=\"S0.E1.m2.2\" xml:id=\"S0.E1.m2.3.1.3.3.1.2\"/></XMApp><XMApp xml:id=\"S0.E1.m2.3.1.3.3.2\"><XMTok meaning=\"exponential\" role=\"OPFUNCTION\" scriptpos=\"post\" xml:id=\"S0.E1.m2.1\">exp</XMTok><XMWrap xml:id=\"S0.E1.m2.3.1.3.3.2.1\"><XMTok fontsize=\"260%\" role=\"OPEN\" stretchy=\"false\">[</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"S0.E1.m2.2\">x</XMTok><XMTok fontsize=\"260%\" role=\"CLOSE\" stretchy=\"false\">]</XMTok></XMWrap></XMApp></XMDual></XMApp></XMApp></XMath></Math>",
      ),
      (
        "Math",
        "S0.E2.m2",
        "<Math mode=\"inline\" tex=\"\\displaystyle=\\ln\\big[y\\big]\" text=\"absent = natural-logarithm@(y)\" xml:id=\"S0.E2.m2\"><XMath xml:id=\"S0.E2.m2.3\"><XMApp xml:id=\"S0.E2.m2.3.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok meaning=\"absent\"/><XMDual xml:id=\"S0.E2.m2.3.1.3\"><XMApp xml:id=\"S0.E2.m2.3.1.3.1\"><XMRef idref=\"S0.E2.m2.1\" xml:id=\"S0.E2.m2.3.1.3.1.1\"/><XMRef idref=\"S0.E2.m2.2\" xml:id=\"S0.E2.m2.3.1.3.1.2\"/></XMApp><XMApp xml:id=\"S0.E2.m2.3.1.3.2\"><XMTok meaning=\"natural-logarithm\" role=\"OPFUNCTION\" scriptpos=\"post\" xml:id=\"S0.E2.m2.1\">ln</XMTok><XMWrap xml:id=\"S0.E2.m2.3.1.3.2.1\"><XMTok fontsize=\"120%\" role=\"OPEN\" stretchy=\"false\">[</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"S0.E2.m2.2\">y</XMTok><XMTok fontsize=\"120%\" role=\"CLOSE\" stretchy=\"false\">]</XMTok></XMWrap></XMApp></XMDual></XMApp></XMath></Math>",
      ),
    ],
  );
}

/// 62zg: the witnesses' shape — a sized bracket that a row break splits, in `align` and in `eqnarray` (`\tr`): each row
/// keeps its half, `\\` ends the row as in pdflatex. 12 errors before (the size read took `\Bigg`, the power read the
/// `[…]` across the `\\`; 2609.12812, 39468). The 5 warnings are the cells holding half a bracket pair, which no math
/// parse spans (math parsing is a separate stream).
#[test]
fn physics_operator_bracket_across_a_row_break() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsmath,physics}
\begin{document}
\begin{align} a ={}& N\exp\Bigg[ x \\ &\quad + y \Bigg] \end{align}
\begin{eqnarray} t &=& \tr\big[A \\ & & B\big] \end{eqnarray}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 5), "{log}");
  assert_eq!(log.matches("Warning:unparsed_math:").count(), 5, "{log}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"S0.E2.m2\""],
    "<Math class=\"ltx_math_unparsed\" tex=\"\\displaystyle\\quad+y\\Bigg]\" xml:id=\"S0.E2.m2\"><XMath xml:id=\"S0.E2.m2.1\"><XMTok lpadding=\"10.0pt\" meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok><XMTok fontsize=\"260%\" role=\"CLOSE\" stretchy=\"false\">]</XMTok></XMath></Math>",
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"S0.E4.m1\""],
    "<Math class=\"ltx_math_unparsed\" mode=\"inline\" tex=\"\\displaystyle B\\big]\" xml:id=\"S0.E4.m1\"><XMath xml:id=\"S0.E4.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">B</XMTok><XMTok fontsize=\"120%\" role=\"CLOSE\" stretchy=\"false\">]</XMTok></XMath></Math>",
  );
}

/// 62zg: a paper's own `\newcommand{\sort}` — biblatex defines `\sort` only inside `\DeclareSortingTemplate`
/// (biblatex.sty:14810), and the binding's global stub (Perl L638) refused it, so `e^{\sort}` lost its exponent and
/// the stub ate the `}` (2609.39511, 97 errors; KNOWN_PERL_ERRORS #505). pdflatex: "Body e^s.".
#[test]
fn biblatex_leaves_sort_to_the_document() {
  // `\DeclareSortingTemplate[<locale>]{<name>}{<spec>}` (biblatex.sty:14796) reads its specification, which the stub
  // used to swallow (review of 62zg). pdflatex: "Body.".
  assert_elements(
    r"\documentclass{article}
\usepackage{biblatex}
\DeclareSortingTemplate{mysort}{\sort{\field{presort}} \sort[final]{\field{sortkey}} \sort{\field{year}}}
\begin{document}
Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Body.</p>")],
  );
  assert_elements(
    r"\documentclass{article}
\usepackage{biblatex}
\newcommand{\sort}{s}
\begin{document}
Body $e^{\sort}$.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">Body <Math mode=\"inline\" tex=\"e^{s}\" text=\"e ^ s\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">e</XMTok><XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">s</XMTok></XMApp></XMath></Math>.</p>",
    )],
  );
}

/// 62zg: `{IEEEeqnarraybox*}` (IEEEtrantools.sty:2157, the box without the `\jot` padding) reads its
/// `[<decl>][<pos>][<width>]` options (:2166-2173) and a column specification whose inter-column glue (`,` `/`, :2370-
/// 2420) makes no columns — under the IEEEtrantools package and the IEEEtran class alike (2609.32652; KNOWN_PERL_ERRORS
/// #506). pdflatex: "h = [a ··· b].".
#[test]
fn ieeeeqnarraybox_star_with_options_and_glue() {
  for class in ["{article}\n\\usepackage{IEEEtrantools}", "{IEEEtran}"] {
    assert_elements(
      &format!(
        r"\documentclass{class}
\begin{{document}}
\begin{{equation}}
 h = \left[
 \begin{{IEEEeqnarraybox*}}[][c]{{,c/c/c,}}
 a & \cdots & b
 \end{{IEEEeqnarraybox*}}
 \right].
\end{{equation}}
\end{{document}}"
      ),
      "ar5iv.sty",
      (0, 0),
      &[(
        "XMArray",
        "S0.E1.m1.2",
        "<XMArray role=\"ARRAY\" vattach=\"middle\" xml:id=\"S0.E1.m1.2\"><XMRow xml:id=\"S0.E1.m1.2.1\"><XMCell align=\"center\" xml:id=\"S0.E1.m1.2.1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok></XMCell><XMCell align=\"center\" xml:id=\"S0.E1.m1.2.1.2\"><XMTok name=\"cdots\" role=\"ELIDEOP\">\u{22ef}</XMTok></XMCell><XMCell align=\"center\" xml:id=\"S0.E1.m1.2.1.3\"><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMCell></XMRow></XMArray>",
      )],
    );
  }
}

/// 62zg: `\captionsetup{type=figure}` around two minipages with `\subcaption`s, then the `\caption` (2609.33998's
/// teaser): each sub-caption becomes an `ltx:figure` in its minipage (`sub<type>` maps to its parent type, as
/// `{subfigure}` does), the caption the figure after the paragraph. Floating that figure up chose the minipage — a
/// previous sibling of the text node after `\end{minipage}`, which can hold a figure but is not open — and reported
/// "Attempt to close inline-logical-block, which isn't open" (`float_to_element`'s closing form now takes only open
/// nodes; 2609.15558 alike). Known residual: sub-figures captioned before their figure's `\caption` take the number
/// the figure counter has then — "0a"/"0b" here, the previous figure's number in a later group — where pdflatex, which
/// writes the label at shipout, prints "1a"/"1b" (RED repro captions-floats/subcaption_before_caption_numbers_its_figure).
/// pdflatex: "(a) Left.", "(b) Right.", "Figure 1: Both.".
#[test]
fn subcaptions_in_minipages_then_the_caption() {
  assert_elements(
    r"\documentclass{article}
\usepackage{caption,subcaption}
\begin{document}
\begin{center}
\captionsetup{type=figure}
\begin{minipage}[t]{0.48\textwidth}\centering A\subcaption{Left.}\label{a}\end{minipage}%
\hfill
\begin{minipage}[t]{0.48\textwidth}\centering B\subcaption{Right.}\label{b}\end{minipage}
\caption{Both.}\label{c}
\end{center}
After \ref{a}, \ref{c}.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "figure",
        "S0.F0.sf2",
        "<figure align=\"center\" inlist=\"lof\" labels=\"LABEL:b\" xml:id=\"S0.F0.sf2\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F0.sf2.1\">(b)</text></tag><tag role=\"refnum\">0b</tag></tags><toccaption><tag close=\" \">b</tag>Right.</toccaption><caption><tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F0.sf2.2\">(b)</text></tag><text fontsize=\"90%\" xml:id=\"S0.F0.sf2.3\">Right.</text></caption></figure>",
      ),
      (
        "figure",
        "S0.F1",
        "<figure align=\"center\" inlist=\"lof\" labels=\"LABEL:c\" xml:id=\"S0.F1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.1\">Figure 1</text></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Both.</toccaption><caption><tag close=\": \"><text fontsize=\"90%\" xml:id=\"S0.F1.2\">Figure 1</text></tag><text fontsize=\"90%\" xml:id=\"S0.F1.3\">Both.</text></caption></figure>",
      ),
    ],
  );
}

/// 62zg (review): a sibling that cannot auto-close between the two minipages (a Math; a rule or a graphic alike) must not
/// stop the caption's closing walk and send the figure into the first minipage — the whole block holds the paragraph,
/// then the figure after it. pdflatex: "(a) Left. x (b) Right." over "Figure 1: Both.".
#[test]
fn subcaption_boxes_around_math_then_the_caption() {
  assert_elements(
    r"\documentclass{article}
\usepackage{caption,subcaption}
\begin{document}
\begin{center}\captionsetup{type=figure}
\begin{minipage}[t]{0.3\textwidth}\centering A\subcaption{Left.}\end{minipage}
$x$
\begin{minipage}[t]{0.3\textwidth}\centering B\subcaption{Right.}\end{minipage}
\caption{Both.}
\end{center}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "logical-block",
      "id1",
      "<logical-block xml:id=\"id1\"><para xml:id=\"p3\"><p align=\"center\" xml:id=\"p3.1\"><inline-logical-block class=\"ltx_minipage\" vattach=\"top\" width=\"103.5pt\" xml:id=\"p3.1.1\"><para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\">A</p></para><figure align=\"center\" inlist=\"lof\" xml:id=\"S0.F0.sf1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F0.sf1.1\">(a)</text></tag><tag role=\"refnum\">0a</tag></tags><toccaption><tag close=\" \">a</tag>Left.</toccaption><caption><tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F0.sf1.2\">(a)</text></tag><text fontsize=\"90%\" xml:id=\"S0.F0.sf1.3\">Left.</text></caption></figure></inline-logical-block><Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"m1\"><XMath xml:id=\"m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math><inline-logical-block class=\"ltx_minipage\" vattach=\"top\" width=\"103.5pt\" xml:id=\"p3.1.2\"><para xml:id=\"p2\"><p align=\"center\" xml:id=\"p2.1\">B</p></para><figure align=\"center\" inlist=\"lof\" xml:id=\"S0.F0.sf2\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F0.sf2.1\">(b)</text></tag><tag role=\"refnum\">0b</tag></tags><toccaption><tag close=\" \">b</tag>Right.</toccaption><caption><tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F0.sf2.2\">(b)</text></tag><text fontsize=\"90%\" xml:id=\"S0.F0.sf2.3\">Right.</text></caption></figure></inline-logical-block></p></para><figure align=\"center\" inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.1\">Figure 1</text></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Both.</toccaption><caption><tag close=\": \"><text fontsize=\"90%\" xml:id=\"S0.F1.2\">Figure 1</text></tag><text fontsize=\"90%\" xml:id=\"S0.F1.3\">Both.</text></caption></figure></logical-block>",
    )],
  );
}

/// 62zg (review): inside a real figure, a `\caption` after inline material that cannot auto-close (framed text, Math, a
/// cite) closes the paragraph before it, so the material after the caption follows it: before, the caption's float-up
/// stopped at the framed box (a previous sibling, which is not open) and "C after" was merged into the paragraph BEFORE
/// the caption. Perl places it after the caption too (OXIDIZED_DESIGN_DIVERGENCES #455). pdflatex: "A B text", the
/// caption, "C after".
#[test]
fn caption_after_framed_text_keeps_the_reading_order() {
  assert_elements(
    r"\documentclass{article}
\begin{document}
\begin{figure}
\fbox{A} \hfill \fbox{B} text
\caption{Two}
\fbox{C} after
\end{figure}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "figure",
      "S0.F1",
      "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><p class=\"ltx_figure_panel\" xml:id=\"S0.F1.1\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F1.1.1\">A</text>  <text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F1.1.2\">B</text> text</p><toccaption><tag close=\" \">1</tag>Two</toccaption><caption><tag close=\": \">Figure 1</tag>Two</caption><p class=\"ltx_figure_panel\" xml:id=\"S0.F1.2\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F1.2.1\">C</text> after</p></figure>",
    )],
  );
}

/// 62zi: biblatex's message commands (biblatex.sty:135-155, 1353-1370) exist for the style files a paper ships to call
/// — undefined, each call was an error (2609.32652's acmauthoryear.cbx; 58 papers in 2609 ship such files). pdflatex:
/// "Body.".
#[test]
fn biblatex_message_commands_are_defined() {
  assert_elements(
    r"\documentclass{article}
\usepackage{biblatex}
\makeatletter
\newcommand{\checkstyle}{\blx@info{style checked}\blx@info@noline{again}}
\makeatother
\begin{document}
\checkstyle Body.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Body.</p>")],
  );
}

/// 62zi: acmart.cls:44-47 reads `acmart-preload-hook.tex` before its packages, with its class warning (2609.29962
/// passes `svgnames` to xcolor there: 293 `DarkViolet` errors), and :673-680 defines the ACM palette (`ACMPurple`, 17
/// papers in 2609). pdflatex: the warning, "Text violet and purple." in color.
#[test]
fn acmart_reads_its_preload_hook_and_palette() {
  let (log, xml) = convert_with(
    r"\begin{filecontents*}[overwrite]{acmart-preload-hook.tex}
\PassOptionsToPackage{svgnames}{xcolor}
\end{filecontents*}
\documentclass[acmsmall,screen,nonacm]{acmart}
\colorlet{mylink}{ACMPurple}
\begin{document}
Text {\color{DarkViolet}violet} and {\color{mylink}purple}.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("I am loading acmart-preload-hook.tex"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <text color=\"#9400D3\" xml:id=\"p1.1.1\">violet</text> and <text color=\"#4D00D9\" xml:id=\"p1.1.2\">purple</text>.</p>",
  );
}

/// 62zj: a raw class that defines its author markup only inside its own `\author` (IOS-Book-Article.cls:1146-1184,
/// econsocart.cls:2095-2160): the locked kernel `\author` reads the content instead, and — only because the lock refused
/// that `\author` — `\fnms`/`\snm`/`\orcid` take their OmniBus meanings there, undefined again after, while the
/// document's own `\roles` stands (2407.04130; 2609.06231, 13776, 15113, 28673, 06865: undefined before). Under a class
/// whose `\author` is not refused, an undefined `\snm` errors as in pdflatex. pdflatex: "James Golike Ed", "Text U.".
#[test]
fn raw_class_author_markup_reads_within_author_content() {
  let cls = r"\LoadClass{article}
\def\author{\@ifnextchar[{\author@optarg}{\author@optarg[]}}
\def\author@optarg[#1]#2{\begingroup\def\fnms##1{##1}\def\snm##1{##1}\def\roles##1{##1}\def\orcid##1{}\xdef\@author{#2}\endgroup}";
  let (log, xml) = convert_files_with(
    r"\documentclass{iosmock}
\newcommand\roles[1]{\textsc{#1}}
\title{T}
\author{\fnms{James} \snm{Golike}\roles{Ed}\orcid{0000-0002-1825-0097}}
\begin{document}
\maketitle
\makeatletter Text \ifx\snm\@undefined U\else D\fi.
\end{document}",
    &[("iosmock.cls", cls)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "personname",
    &[],
    "<personname>James Golike<text font=\"smallcaps\" xml:id=\"id1\">Ed</text></personname>",
  );
  assert_element(
    &xml,
    "contact",
    &["role=\"orcid\""],
    "<contact role=\"orcid\"><ref class=\"ltx_orcid\" href=\"https://orcid.org/0000-0002-1825-0097\" title=\"ORCID 0000-0002-1825-0097\"><svg:svg class=\"ltx_orcidlogo\" height=\"1em\" version=\"1.1\" viewBox=\"0 0 72 72\" width=\"1em\"><svg:path d=\"M72,36 C72,55.884375 55.884375,72 36,72 C16.115625,72 0,55.884375 0,36 C0,16.115625 16.115625,0 36,0 C55.884375,0 72,16.115625 72,36 Z\" fill=\"#A6CE39\"/><svg:g fill=\"#FFFFFF\" transform=\"translate(18.868966, 12.910345)\"><svg:polygon points=\"5.03734929 39.1250878 0.695429861 39.1250878 0.695429861 9.14431787 5.03734929 9.14431787 5.03734929 22.6930505 5.03734929 39.1250878\"/><svg:path d=\"M11.409257,9.14431787 L23.1380784,9.14431787 C34.303014,9.14431787 39.2088191,17.0664074 39.2088191,24.1486995 C39.2088191,31.846843 33.1470485,39.1530811 23.1944669,39.1530811 L11.409257,39.1530811 L11.409257,9.14431787 Z M15.7511765,35.2620194 L22.6587756,35.2620194 C32.49858,35.2620194 34.7541226,27.8438084 34.7541226,24.1486995 C34.7541226,18.1301509 30.8915059,13.0353795 22.4332213,13.0353795 L15.7511765,13.0353795 L15.7511765,35.2620194 Z\"/><svg:path d=\"M5.71401206,2.90182329 C5.71401206,4.441452 4.44526937,5.72914146 2.86638958,5.72914146 C1.28750978,5.72914146 0.0187670918,4.441452 0.0187670918,2.90182329 C0.0187670918,1.33420133 1.28750978,0.0745051096 2.86638958,0.0745051096 C4.44526937,0.0745051096 5.71401206,1.36219458 5.71401206,2.90182329 Z\"/></svg:g></svg:svg></ref></contact>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text U.</p>",
  );
  let (log, _xml) = convert_with(
    r"\documentclass{article}
\title{T}
\author{\fnms{James} \snm{Golike}}
\begin{document}
\maketitle
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 2, "{log}");
  // The summary lists undefined macros in no fixed order; each is asserted by its own error line.
  assert!(
    log.contains("Error:undefined:\\fnms ") && log.contains("Error:undefined:\\snm "),
    "{log}"
  );
}

/// 62zj: the ML4H copy of jmlr.cls (:83-142) adds `\mlhtrack{…}` and its `\ifmlh…` switches, which the jmlr binding
/// now defines (2609.00435 and 7 more ML4H papers: undefined before). pdflatex: "Statements. Text.".
#[test]
fn jmlr_reads_the_ml4h_track() {
  assert_elements(
    r"\documentclass[pmlr]{jmlr}
\mlhtrack{findings}
\title{T}
\author{\Name{A. Author}\Email{a@b.c}}
\begin{document}
\maketitle
\ifmlhneedsstatements Statements.\fi\ifmlhdemo Demo.\fi{} Text.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Statements. Text.</p>")],
  );
}

/// 62zj: IEEEoj.cls's front-matter setters (:3447-3451, 3713-3717, 4877): the author note and the corresponding
/// author as notes, the dates and the DOI as the first page prints them (:3457-3458 "Received …; accepted …", "Digital
/// Object Identifier 10.1109/…"), a template's placeholder included (2609.01380, 05811, 11359: undefined before).
/// pdflatex: "Received 12 March, 2025; revised 2 April, 2025; accepted XX Month, XXXX; Date of publication 30 April,
/// 2025; date of current version 1 May, 2025.", the DOI line, the notes; a repeated setter prints its last value.
/// IEEEtj's copies likewise.
#[test]
fn ieeeoj_front_matter_setters() {
  let xml = assert_elements(
    r"\documentclass{IEEEoj}
\receiveddate{12 March, 2025}
\reviseddate{2 April, 2025}
\accepteddate{XX Month, XXXX}
\publisheddate{30 April, 2025}
\currentdate{1 May, 2025}
\doiinfo{OJ.2024.1111111}
\doiinfo{OJ.2024.0000000}
\begin{document}
\title{T}\author{A. Author}
\authornote{Draft note.}
\authornote{Funded by project X.}
\corresp{Corresponding author: A. Author}
\maketitle
Text.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "note",
        "id1",
        "<note role=\"note\" xml:id=\"id1\">Funded by project X.</note>",
      ),
      (
        "note",
        "id2",
        "<note role=\"corresponding\" xml:id=\"id2\">Corresponding author: A. Author</note>",
      ),
      ("p", "p1.1", "<p xml:id=\"p1.1\">Text.</p>"),
    ],
  );
  assert_element(
    &xml,
    "date",
    &["role=\"received\""],
    "<date name=\"Received\u{a0}\" role=\"received\">12 March, 2025</date>",
  );
  assert_element(
    &xml,
    "date",
    &["role=\"accepted\""],
    "<date name=\"Accepted\u{a0}\" role=\"accepted\">XX Month, XXXX</date>",
  );
  assert_element(
    &xml,
    "date",
    &["role=\"revised\""],
    "<date name=\"Revised\u{a0}\" role=\"revised\">2 April, 2025</date>",
  );
  assert_element(
    &xml,
    "date",
    &["role=\"published\""],
    "<date name=\"Date of publication\u{a0}\" role=\"published\">30 April, 2025</date>",
  );
  assert_element(
    &xml,
    "date",
    &["role=\"current\""],
    "<date name=\"Date of current version\u{a0}\" role=\"current\">1 May, 2025</date>",
  );
  assert_element(
    &xml,
    "pubnote",
    &["role=\"doi\""],
    "<pubnote name=\"DOI:\u{a0}\" role=\"doi\">10.1109/OJ.2024.0000000</pubnote>",
  );
  // The overridden first values are gone, not kept beside the last ones.
  assert!(
    !xml.contains("Draft note.") && !xml.contains("1111111"),
    "{xml}"
  );
  // IEEEtj's copies of the setters, which were no-ops (ieeetj.cls:3439-3440 prints them, 2405.01673, 2609.27083;
  // 2603.04284's copy has those lines commented out); an
  // empty `\doiinfo{}` prints no DOI line (ieeetj.cls:3440 `\ifx\@doiinfo\@empty`; 2609.27083).
  let (log, xml) = convert_with(
    r"\documentclass{ieeetj}
\receiveddate{12 March, 2025}
\doiinfo{}
\begin{document}
\title{T}\author{A. Author}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "date",
    &["role=\"received\""],
    "<date name=\"Received\u{a0}\" role=\"received\">12 March, 2025</date>",
  );
  assert_eq!(
    xml_element(&xml, "pubnote", &["role=\"doi\""]),
    None,
    "{xml}"
  );
}

/// 62zj: amsthm.sty:153 `\providecommand\@upn{\textup}` (a raw class's theorem head calls it: econsocart, 2609.06865)
/// and newtxmath.sty:695-706's `\re@DeclareMath…` helpers (vmsta2.cls:236, 2609.02595) — both absent from the
/// bindings, Perl's too (KNOWN_PERL_ERRORS #509, #510). pdflatex: "Theorem 2.", x̄.
#[test]
fn amsthm_upn_and_newtxmath_redeclare() {
  assert_elements(
    r"\documentclass{article}
\usepackage{amsthm}
\makeatletter
\begin{document}
Theorem \@upn{2}.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[("p", "p1.1", "<p xml:id=\"p1.1\">Theorem 2.</p>")],
  );
  assert_elements(
    r#"\documentclass{article}
\usepackage{newtxmath}
\makeatletter
\DeclareSymbolFont{lmoperators}{OT1}{lmr}{m}{n}
\re@DeclareMathAccent{\bar}{\mathalpha}{lmoperators}{"16}
\makeatother
\begin{document}$\bar{x}$\end{document}"#,
    "ar5iv.sty",
    (0, 0),
    &[(
      "XMApp",
      "p1.m1.1.1",
      "<XMApp xml:id=\"p1.m1.1.1\"><XMTok name=\"bar\" role=\"OVERACCENT\">\u{af}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMApp>",
    )],
  );
}

/// 62zk: the informs3/informs4 class API (informs4.cls:9-54, 889-890, 948-1023, 1198-1233, 2229, 2449): `\AUTHOR`
/// and `\AFF` take one argument each, the affiliation under its author (they were read as `\AUTHOR{name}{aff}`, which
/// swallowed the `\AFF`, 2609.37380); the stored metadata and running heads read back (`\theRUNTITLE`, `\theJOURNAL`);
/// `\FUNDING`; `\FIGURE`/`\TABLE` with their notes; `\argmax`; the journal switches (2609.10587, 17368, 22690, 23739,
/// 24605, 25924, 37380, 38842: undefined before). pdflatex (informs4.cls of 2609.38842): the two authors with their
/// affiliations, "Funding: Grant 42.", "Body Short T and Operations Research.", "arg maxx f.", "Figure 1 Fig caption
/// IMG Note. Fig note text.", "Table 1 Tab caption cell Tab note text." (the body's `\small` stays in its group,
/// informs4.cls:2470), "OPRE".
#[test]
fn informs_class_api() {
  let xml = assert_elements(
    r"\documentclass[opre,nonblindrev]{informs4}
\TheoremsNumberedThrough
\begin{document}
\RUNTITLE{Short T}
\TITLE{Long Title}
\ARTICLEAUTHORS{%
\AUTHOR{Ann Author}
\AFF{Dept A, Univ A, \EMAIL{ann@a.edu}}
\AUTHOR{Bob Builder}
\AFF{Dept B, Univ B}
}
\ABSTRACT{Abstract text.}
\FUNDING{Grant 42.}
\maketitle
\JOURNAL{Operations Research}
\LRHFirstLine{lrh1}\RRHSecondLine{rrh2}
Body \theRUNTITLE{} and \theJOURNAL.

$\argmax_x f$.
\begin{figure}
\FIGURE{IMG}{Fig caption}{Fig note text.}
\end{figure}
\begin{table}
\TABLE{Tab caption}{\small\begin{tabular}{c}cell\end{tabular}}{Tab note text.}
\end{table}
\begin{figure}
\FIGURE{IMG2}{Second caption}{}
\end{figure}
\makeatletter\if@OPRE OPRE\fi\if@MNSC MNSC\fi\makeatother
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "note",
        "id2",
        "<note role=\"funding\" xml:id=\"id2\">Grant 42.</note>",
      ),
      (
        "p",
        "p1.1",
        "<p xml:id=\"p1.1\">Body Short T and Operations Research.</p>",
      ),
      (
        "p",
        "S0.F1.2",
        "<p class=\"ltx_figure_panel\" xml:id=\"S0.F1.2\"><text font=\"italic\" xml:id=\"S0.F1.2.1\">Note.</text>\u{2002}Fig note text.</p>",
      ),
      (
        "p",
        "S0.T1.2",
        "<p class=\"ltx_figure_panel\" xml:id=\"S0.T1.2\">Tab note text.</p>",
      ),
      ("p", "p3.1", "<p xml:id=\"p3.1\">OPRE</p>"),
    ],
  );
  // Each affiliation under its own author; the second author's `before` is the kernel's author separator
  // (`\lx@author@sep`, a `\qquad`).
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Ann Author</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept A, Univ A, ann@a.edu</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Builder</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept B, Univ B</contact></creator>",
  );
  // An empty note prints nothing: the second figure holds its caption and body only.
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><toccaption><tag close=\" \">2</tag>Second caption</toccaption><caption><tag close=\": \">Figure 2</tag>Second caption</caption><p xml:id=\"S0.F2.1\">IMG2</p></figure>",
  );
}

/// 62zk: informs's blind-review options (`blindrev`, `dblanonrev`; informs4.cls:48-50, 987-995, 1225-1233, 1952) print
/// the class's notice in place of the author block and drop the history, the acknowledgment and the author bio, as the
/// PDF does (user 2026-10-06: honor the option that toggles the display, as the PDF does). The informs3 family words the
/// notice "blinded" and prints a MOOR paper's acknowledgment anyway (informs3.cls:1034-1036, 1266-1274);
/// informs3noheader prints no notice (2609.04127 informs3noheader.cls:1032-1035). No 2609 paper sets the options.
#[test]
fn informs_blind_review_hides_the_authors() {
  let body = r"\begin{document}
\TITLE{Long Title}
\RUNAUTHOR{Ann Author}
\ARTICLEAUTHORS{\AUTHOR{Ann Author}\AFF{Dept A}}
\HISTORY{Received 2024.}
\maketitle
Text.
\ACKNOWLEDGMENT{We thank X.}
\AUTHORBIO{Ann is a bio.}
\end{document}";
  let xml = assert_elements(
    &format!(r"\documentclass[moor,dblanonrev]{{informs4}}{body}"),
    "ar5iv.sty",
    (0, 0),
    &[(
      "note",
      "id1",
      "<note role=\"authors\" xml:id=\"id1\">(Authors\u{2019} names are not included for peer review)</note>",
    )],
  );
  assert_eq!(xml_element(&xml, "creator", &[]), None, "{xml}");
  assert_eq!(
    xml_element(&xml, "note", &["role=\"history\""]),
    None,
    "{xml}"
  );
  assert_eq!(xml_element(&xml, "acknowledgements", &[]), None, "{xml}");
  // Nothing names the author: not the running head (informs4.cls:1071-1076), the affiliation or the bio.
  for hidden in ["Ann Author", "Dept A", "is a bio"] {
    assert!(!xml.contains(hidden), "{hidden}: {xml}");
  }
  let xml = assert_elements(
    &format!(r"\documentclass[moor,blindrev]{{informs3}}{body}"),
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "note",
        "id1",
        "<note role=\"authors\" xml:id=\"id1\">(Authors\u{2019} names blinded for peer review)</note>",
      ),
      (
        "acknowledgements",
        "acknowledgements1",
        "<acknowledgements inlist=\"toc\" name=\"Acknowledgments\" xml:id=\"acknowledgements1\">We thank X.</acknowledgements>",
      ),
    ],
  );
  assert_eq!(xml_element(&xml, "creator", &[]), None, "{xml}");
  // informs3noheader has no binding of its own: the one warning is the prefix fallback's.
  let (log, xml) = convert_with(
    &format!(r"\documentclass[opre,blindrev]{{informs3noheader}}{body}"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Can't find binding for class informs3noheader (using informs)"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text.</p>",
  );
  assert_eq!(xml_element(&xml, "creator", &[]), None, "{xml}");
  assert_eq!(xml_element(&xml, "note", &[]), None, "{xml}");
}

/// 62zk: the superscript marks of an informs author list link each marked `\AFF` line to the authors carrying its mark
/// (kernel `\lx@add@authors@append` / `\lx@add@affiliation@marked`). Before, every affiliation went to the last author
/// (2609.17368: all six `\AFF` lines under Zhou Xu); one `\AFF` naming several marked institutions is split at its
/// marks (2609.22690). pdflatex (informs4.cls of 2609.38842) prints the names with their marks and the affiliations as
/// written.
#[test]
fn informs_author_marks_link_affiliations() {
  let affiliation = |name: &str| {
    format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{name}</contact>")
  };
  let (log, xml) = convert_with(
    r"\documentclass[opre,nonblindrev]{informs4}
\begin{document}
\TITLE{T}
\ARTICLEAUTHORS{\AUTHOR{Sunkanghong Wang$^{a,e}$, Zhengzhong You$^{b}$,\\ Lijun Wei$^{e,*}$, Zhou Xu$^{a}$}
\AFF{$^a$Univ A}
\AFF{$^b$Univ B}
\AFF{$^e$Univ E}
\AFF{$^*$Corresponding author}}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let sep = "before=\"\u{2003}\u{2003}\" ";
  assert_eq!(
    creators_of(&xml),
    vec![
      format!(
        "<creator role=\"author\"><personname>Sunkanghong Wang</personname>{}{}</creator>",
        affiliation("Univ A"),
        affiliation("Univ E")
      ),
      format!(
        "<creator {sep}role=\"author\"><personname>Zhengzhong You</personname>{}</creator>",
        affiliation("Univ B")
      ),
      format!(
        "<creator {sep}role=\"author\"><personname>Lijun Wei</personname>{}{}</creator>",
        affiliation("Univ E"),
        affiliation("Corresponding author")
      ),
      format!(
        "<creator {sep}role=\"author\"><personname>Zhou Xu</personname>{}</creator>",
        affiliation("Univ A")
      ),
    ],
    "{xml}"
  );
  let (log, xml) = convert_with(
    r"\documentclass[opre,nonblindrev]{informs4}
\begin{document}
\TITLE{T}
\ARTICLEAUTHORS{\AUTHOR{Huikang Liu\textsuperscript{1},
Zhengchao Wang\textsuperscript{2}}
\AFF{\textsuperscript{1}Shanghai Jiao Tong University;
\textsuperscript{2}The University of Sydney}}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators_of(&xml),
    vec![
      format!(
        "<creator role=\"author\"><personname>Huikang Liu</personname>{}</creator>",
        affiliation("Shanghai Jiao Tong University")
      ),
      format!(
        "<creator {sep}role=\"author\"><personname>Zhengchao Wang</personname>{}</creator>",
        affiliation("The University of Sydney")
      ),
    ],
    "{xml}"
  );
}

/// 62zk: informs's hanging lists `{henumerate}` and `{hitemize}` (informs4.cls:1702-1763) are enumerate and itemize
/// with another indentation (2609.21433: undefined before). pdflatex (informs4.cls of 2609.38842): "1. First", "• Bullet".
#[test]
fn informs_hanging_lists() {
  assert_elements(
    r"\documentclass[mnsc,sglanonrev]{informs4}
\begin{document}
\begin{henumerate}
\item First
\end{henumerate}
\begin{hitemize}
\item Bullet
\end{hitemize}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "enumerate",
        "S0.I1",
        "<enumerate xml:id=\"S0.I1\"><item xml:id=\"S0.I1.i1\"><tags><tag>1.</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">item\u{a0}1</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p xml:id=\"S0.I1.i1.p1.1\">First</p></para></item></enumerate>",
      ),
      (
        "itemize",
        "S0.I2",
        "<itemize xml:id=\"S0.I2\"><item xml:id=\"S0.I2.i1\"><tags><tag>\u{2022}</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I2.i1.p1\"><p xml:id=\"S0.I2.i1.p1.1\">Bullet</p></para></item></itemize>",
      ),
    ],
  );
}

/// 62zk: an affiliation list's line that starts with a mark and names several marked institutions is split at its
/// marks, one affiliation per mark (`affiliation_calls`, as `\lx@add@authors` splits its marker-led lines); before,
/// the last mark labelled the whole line, so Ann had no affiliation and Bob both (2609.21347, 22690). A line with
/// text before its mark stays whole. iopart's `\address` reaches it through `\lx@add@affiliations`. pdflatex (iopart):
/// "Ann Able1 and Bob Baker2", "1 Univ A; 2 Univ B", "Present address: 2 Univ C".
#[test]
fn marked_affiliation_lines_split_at_their_marks() {
  let affiliation = |name: &str| {
    format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{name}</contact>")
  };
  let (log, xml) = convert_with(
    r"\documentclass{iopart}
\begin{document}
\title{T}
\author{Ann Able$^1$ and Bob Baker$^2$}
\address{$^1$Univ A; $^2$Univ B\\ Present address: $^2$Univ C}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators_of(&xml),
    vec![
      format!(
        "<creator role=\"author\"><personname>Ann Able</personname>{}</creator>",
        affiliation("Univ A")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}{}</creator>",
        affiliation("Univ B"),
        affiliation("Present address: Univ C")
      ),
    ],
    "{xml}"
  );
}

/// The `<creator>` elements of `xml`, in document order, each whole.
fn creators_of(xml: &str) -> Vec<String> {
  let mut found = Vec::new();
  let mut from = 0;
  while let Some(at) = xml[from..].find("<creator") {
    found.extend(xml_element(&xml[from + at..], "creator", &[]));
    from += at + 1;
  }
  found
}

/// 62zl: iopart's `\address` is used "once for each address" (iopart.cls:240-248), each call printing its block, so
/// each adds its affiliation, linked by its mark; Perl's `\lx@add@affiliations` dequeued the ones before, keeping only
/// the last (KNOWN_PERL_ERRORS #511; 21 of the 41 2609 iopart papers, 2609.01831 lost three of four). Repro
/// sectioning-frontmatter/iopart_each_address_adds_an_affiliation. pdflatex (iopart.cls of 2609.01831): "1 Univ A",
/// "2 Univ B".
#[test]
fn iopart_each_address_adds_an_affiliation() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/iopart_each_address_adds_an_affiliation.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators_of(&xml),
    vec![
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ B</contact></creator>",
    ],
    "{xml}"
  );
}

/// 62zl: a marked affiliation list wrapped whole in a font command or a group is split inside it, the wrapper (a
/// group with its opening declarations) repeated on each piece; the splitter cut inside the group, and every piece
/// after the first left the font (62zk review). Repro
/// sectioning-frontmatter/affiliation_marks_inside_a_font_group_keep_the_font. pdflatex (iopart.cls of 2609.01831):
/// "1 Univ A, 2 Univ B" all italic; "1 Univ A; 2 Univ B" all small.
#[test]
fn marked_affiliations_inside_a_font_group_keep_the_font() {
  let affiliated = |xml: &str| -> Vec<String> {
    creators_of(xml)
      .iter()
      .map(|c| xml_element(c, "contact", &[]).unwrap_or_default())
      .collect()
  };
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/affiliation_marks_inside_a_font_group_keep_the_font.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    affiliated(&xml),
    vec![
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\" xml:id=\"id1\">Univ A</text></contact>",
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\" xml:id=\"id2\">Univ B</text></contact>",
    ],
    "{xml}"
  );
  let (log, xml) = convert_with(
    r"\documentclass{iopart}
\begin{document}
\title{T}
\author{Ann Able$^1$ and Bob Baker$^2$}
\address{{\small $^1$Univ A; $^2$Univ B}}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    affiliated(&xml),
    vec![
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id1\">Univ A</text></contact>",
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\" xml:id=\"id2\">Univ B</text></contact>",
    ],
    "{xml}"
  );
}

/// 62zl: in a marked affiliation list an unmarked line continues the affiliation before it, as an unmarked line
/// continues an entry in `\lx@add@authors`; each `\\` line had been its own affiliation, the unmarked ones placed by
/// position, not by mark (2609.19448). Repro sectioning-frontmatter/marked_affiliation_continuation_lines_stay_with_it.
/// pdflatex (iopart.cls of 2609.01831): "1 Dept A", "Univ A", "2 Dept B", "Univ B".
#[test]
fn marked_affiliation_continuation_lines_stay_with_it() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/marked_affiliation_continuation_lines_stay_with_it.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators_of(&xml),
    vec![
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept A<break/>Univ A</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept B<break/>Univ B</contact></creator>",
    ],
    "{xml}"
  );
}

/// 62zl: apa7.cls is read for its modes and packages, and its author block maps onto the frontmatter API:
/// `\authorsnames[marks]{names}` (:798-806) and `\authorsaffiliations{…}` (:808-812), numbered in order, link each
/// name to the affiliations its marks number; `\authornote` is a note, `\abstract`/`\keywords` the abstract and
/// keywords (:785-787); under `mask` (:131) no author identity prints (2609.00670, 02899, 12869, 19448: 32 errors
/// before). The one warning is pgf's `svg.path` library, which the class loads for its ORCID icon (:312), as Perl warns.
/// pdflatex: "Ann Able1, 2 and Bob Baker2", "1Univ A", "2Univ B", "Author Note", "Correspondence to Ann.".
#[test]
fn apa7_author_block_maps_onto_the_frontmatter() {
  if !latexml::util::test::kpse_has("apa7.cls") {
    return;
  }
  let body = r"\title{A Title}
\authorsnames[{1,2},2]{Ann Able, Bob Baker}
\authorsaffiliations{{Univ A},{Univ B}}
\authornote{Correspondence to Ann.}
\abstract{Abstract text.}
\keywords{alpha, beta}
\begin{document}
\maketitle
Body, see \ref{a1}.
\appendix
\section{Extra}\label{a1}
More.
\end{document}";
  let pgf_warning =
    "The conditional \\pgf@lib@svg@relative is being defined but doesn't start with \\if";
  let (log, xml) = convert_with(
    &format!(r"\documentclass[man]{{apa7}}{body}"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains(pgf_warning), "{log}");
  let affiliation = |name: &str| {
    format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{name}</contact>")
  };
  assert_eq!(
    creators_of(&xml),
    vec![
      format!(
        "<creator role=\"author\"><personname>Ann Able</personname>{}{}</creator>",
        affiliation("Univ A"),
        affiliation("Univ B")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}</creator>",
        affiliation("Univ B")
      ),
    ],
    "{xml}"
  );
  // apa7.cls builds the title block from its stored lists at `\maketitle` (:780-890), so affiliations given before
  // the names are linked the same (round 6: lost).
  let names = "\\authorsnames[{1,2},2]{Ann Able, Bob Baker}\n";
  let affiliations = "\\authorsaffiliations{{Univ A},{Univ B}}\n";
  let swapped = body.replace(
    &format!("{names}{affiliations}"),
    &format!("{affiliations}{names}"),
  );
  assert_ne!(swapped, body);
  let (swapped_log, swapped_xml) = convert_with(
    &format!(r"\documentclass[man]{{apa7}}{swapped}"),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    (error_count(&swapped_log), warning_count(&swapped_log)),
    (0, 1),
    "{swapped_log}"
  );
  assert_eq!(creators_of(&swapped_xml), creators_of(&xml));
  assert_element(
    &xml,
    "note",
    &["role=\"authornote\""],
    "<note role=\"authornote\" xml:id=\"id1\">Correspondence to Ann.</note>",
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">alpha, beta</keywords>",
  );
  // The kernel's appendix stands over apa7.cls's (:1041-1075), whose `\section` redefinition the kernel refuses:
  // lettered, as pdflatex's "Appendix A" (2609.00670 read "Appendix 9").
  assert_element(
    &xml,
    "appendix",
    &["xml:id=\"A1\""],
    "<appendix inlist=\"toc\" labels=\"LABEL:a1\" xml:id=\"A1\"><tags><tag>Appendix A</tag><tag role=\"autoref\">Appendix\u{a0}A<text xml:id=\"A1.1\"/></tag><tag role=\"refnum\">A</tag><tag role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix A</tag>Extra</title><toctitle><tag close=\" \">A</tag>Extra</toctitle><para xml:id=\"A1.p1\"><p xml:id=\"A1.p1.1\">More.</p></para></appendix>",
  );
  let (log, xml) = convert_with(
    &format!(r"\documentclass[man,mask]{{apa7}}{body}"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators_of(&xml), Vec::<String>::new(), "{xml}");
  for hidden in ["Ann Able", "Univ A", "Correspondence"] {
    assert!(!xml.contains(hidden), "{hidden}: {xml}");
  }
  // `\author` is the class's one-name `\authorsnames` (:766), hidden under `mask` too; the student page's course,
  // instructor and due date (`stu`, :1292-1294) print either way; an unnumbered affiliation under unmarked names
  // belongs to every author, marks of its own kept as written.
  let (log, xml) = convert_with(
    r"\documentclass[stu,mask]{apa7}
\title{T}
\author{Ann Able}
\affiliation{Univ A}
\course{PSY 101}
\professor{Dr. Zed}
\duedate{May 5}
\begin{document}
\maketitle
Body.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(creators_of(&xml), Vec::<String>::new(), "{xml}");
  assert!(!xml.contains("Ann Able"), "{xml}");
  for (role, text) in [
    ("course", "PSY 101"),
    ("professor", "Dr. Zed"),
    ("duedate", "May 5"),
  ] {
    assert_eq!(
      xml_element(&xml, "note", &[&format!("role=\"{role}\"")])
        .map(|note| note.contains(&format!(">{text}</note>"))),
      Some(true),
      "{role}: {xml}"
    );
  }
  let (log, xml) = convert_with(
    r"\documentclass[man]{apa7}
\title{T}
\authorsnames{Ann Able, Bob Baker}
\authorsaffiliations{{$^1$Univ A}}
\begin{document}
\maketitle
Body.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  let shared = "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id1\">1</sup>Univ A</contact>";
  assert_eq!(
    creators_of(&xml),
    vec![
      format!("<creator role=\"author\"><personname>Ann Able</personname>{shared}</creator>"),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}</creator>",
        shared.replace("id1", "id2")
      ),
    ],
    "{xml}"
  );
}

/// 62zl: the marked affiliation shapes the A/B of the brace-depth rule turned up, each linked by its marks: marks
/// inside a leading `\small{…}` with text after it (2609.00995: the text stays with the last piece); a `\thanks{…}`
/// line holding the list (2609.24896: unwrapped, as `\lx@add@thanks` reads it, not repeated on each piece); and,
/// after a marked entry, an address line or a footnote-symbol legend, which stays apart rather than continuing it
/// (llncs `\institute`, 2609.06094).
#[test]
fn marked_affiliation_wrappers_and_legends() {
  let contacts = |xml: &str| -> Vec<Vec<String>> {
    creators_of(xml)
      .iter()
      .map(|creator| {
        let mut found = Vec::new();
        let mut from = 0;
        while let Some(at) = creator[from..].find("<contact") {
          found.extend(xml_element(&creator[from + at..], "contact", &[]));
          from += at + 1;
        }
        found
      })
      .collect()
  };
  let affiliation = |name: &str| {
    format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{name}</contact>")
  };
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\textsuperscript{1}, Bob Baker\textsuperscript{2}\\
\small{\textsuperscript{1} Univ A, \textsuperscript{2} Univ B}. Email: x@y.z}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // `\small` is a declaration: what follows its group is small too, as in the PDF.
  assert_eq!(
    contacts(&xml),
    vec![
      vec![affiliation(
        "<text fontsize=\"90%\" xml:id=\"id1\"> Univ A</text>"
      )],
      vec![affiliation(
        "<text fontsize=\"90%\" xml:id=\"id2\"> Univ B. Email: x@y.z</text>"
      )],
    ],
    "{xml}"
  );
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able$^{1}$, Bob Baker$^{2}$\\
\thanks{$^1$Univ A. $^2$Univ B.}}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    contacts(&xml),
    vec![vec![affiliation("Univ A.")], vec![affiliation("Univ B.")]],
    "{xml}"
  );
  let (log, xml) = convert_with(
    r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1,*} \and Bob Baker\inst{2}}
\institute{$^1$ Univ A \\ $^2$ Univ B \\ \email{a@x.y, b@x.y} \\ * Equal contribution}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = contacts(&xml);
  // Ann holds her affiliation and her address alone: each address of the line goes to its author in order
  // (`email_line_calls`), the legend is not folded into Univ B. (Where the legend goes, an unmarked line placed by
  // position, is not pinned: it is not an affiliation.)
  assert_eq!(
    found[0],
    vec![
      affiliation(" Univ A"),
      "<contact name=\"Email:\u{a0}\" role=\"email\">a@x.y</contact>".to_string()
    ],
    "{xml}"
  );
  let bob: Vec<&String> = found[1]
    .iter()
    .filter(|c| !c.contains("Equal contribution"))
    .collect();
  assert_eq!(
    bob,
    vec![
      &affiliation(" Univ B"),
      &"<contact name=\"Email:\u{a0}\" role=\"email\">b@x.y</contact>".to_string()
    ],
    "{xml}"
  );
  assert_eq!(xml.matches("Equal contribution").count(), 1, "{xml}");
}

/// 62zl review shapes, each linked by its marks: a `$^\dag$` mark still labels its affiliation (`\dag` is a legend
/// symbol only at a line's start, not a footnote-symbol superscript); `\and` ends an entry (an unmarked line after it
/// is no continuation), while `\quad` continues one; an address line gives each address to its author in order; marks
/// after a wrapper split too; a group's opening `\color{…}` is no text before the mark and is repeated with its
/// argument on each piece.
#[test]
fn marked_affiliation_review_shapes() {
  let contacts = |tex: &str| -> Vec<String> {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    creators_of(&xml)
  };
  let creator = |before: bool, name: &str, rest: &str| {
    let sep = if before {
      "before=\"\u{2003}\u{2003}\" "
    } else {
      ""
    };
    format!("<creator {sep}role=\"author\"><personname>{name}</personname>{rest}</creator>")
  };
  let affiliation = |name: &str| {
    format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{name}</contact>")
  };
  let email =
    |addr: &str| format!("<contact name=\"Email:\u{a0}\" role=\"email\">{addr}</contact>");
  assert_eq!(
    contacts(
      r"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able$^{1}$, Bob Baker$^{\dag}$\\ $^1$Univ A\\ $^\dag$Univ C}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation("Univ A")),
      creator(true, "Bob Baker", &affiliation("Univ C"))
    ]
  );
  // `\and` ends Ann's entry: "Univ C" is an entry of its own (placed by position, so not pinned here), not folded
  // into Univ A; `\quad` continues Bob's.
  let found = contacts(
    r"\documentclass{iopart}
\begin{document}
\title{T}
\author{Ann Able$^1$ and Bob Baker$^2$}
\address{$^1$ Univ A \and Univ C}
\address{$^2$ Univ B \quad Univ D}
\maketitle
Text.
\end{document}",
  );
  assert_eq!(
    found[0],
    creator(false, "Ann Able", &affiliation(" Univ A"))
  );
  assert!(
    found[1].contains(&affiliation(" Univ B\u{2003}Univ D")),
    "{found:?}"
  );
  assert_eq!(
    found
      .iter()
      .map(|c| c.matches("Univ C").count())
      .sum::<usize>(),
    1,
    "{found:?}"
  );
  // A declaration then a content command opening an author line (`\large\textbf{Ann Able}$^1$`) is no affiliation
  // line: `\textbf{…}` is content, not a declaration's argument (round 2: the authors were lost).
  assert_eq!(
    contacts(
      r"\documentclass{article}
\begin{document}
\title{T}
\author{\large\textbf{Ann Able}$^1$, \textbf{Bob Baker}$^2$\\ $^1$Univ A\\ $^2$Univ B}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation("Univ A")),
      creator(true, "Bob Baker", &affiliation("Univ B"))
    ]
  );
  // An empty leading group holds no mark: the whole line decides, so these stay affiliation lines.
  assert_eq!(
    contacts(
      r"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able$^1$, Bob Baker$^2$\\ {}$^1$Univ A\\ {}$^2$Univ B}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation("Univ A")),
      creator(true, "Bob Baker", &affiliation("Univ B"))
    ]
  );
  // One address under a marked affiliation is that affiliation's authors' (round 2: by order it went to author 1).
  assert_eq!(
    contacts(
      r"\documentclass{iopart}
\begin{document}
\title{T}
\author{Ann Able$^1$, Bob Baker$^1$ and Cy Dee$^2$}
\address{$^1$ Univ A}
\address{$^2$ Univ B\\ cy@x.y}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation(" Univ A")),
      creator(true, "Bob Baker", &affiliation(" Univ A")),
      creator(
        true,
        "Cy Dee",
        &format!("{}{}", affiliation(" Univ B"), email("cy@x.y"))
      )
    ]
  );
  // A mark with a control sequence labels the email as it labels the affiliation (round 3: the label was retokenized
  // into `\mathrma`, an undefined-command error, and digested, which mangled its `\`).
  for (ann, bob) in [
    (r"$^{\mathrm{a}}$", r"$^{\mathrm{b}}$"),
    (r"\textsuperscript{\it a}", r"\textsuperscript{\it b}"),
    (r"$^\text{a}$", r"$^\text{b}$"),
  ] {
    assert_eq!(
      contacts(&format!(
        r"\documentclass{{iopart}}
\usepackage{{amsmath}}
\begin{{document}}
\title{{T}}
\author{{Ann Able{ann} and Bob Baker{bob}}}
\address{{{ann} Univ A\\ ann@x.y}}
\address{{{bob} Univ B\\ bob@x.y}}
\maketitle
Text.
\end{{document}}"
      )),
      vec![
        creator(
          false,
          "Ann Able",
          &format!("{}{}", affiliation(" Univ A"), email("ann@x.y"))
        ),
        creator(
          true,
          "Bob Baker",
          &format!("{}{}", affiliation(" Univ B"), email("bob@x.y"))
        )
      ],
      "{ann}"
    );
  }
  // One address under an affiliation two authors share continues it, as printed (round 3: each author got it).
  assert_eq!(
    contacts(
      r"\documentclass{iopart}
\begin{document}
\title{T}
\author{Ann Able$^1$, Bob Baker$^1$ and Cy Dee$^2$}
\address{$^1$ Univ A\\ ann@x.y}
\address{$^2$ Univ B}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation(" Univ A<break/>ann@x.y")),
      creator(true, "Bob Baker", &affiliation(" Univ A<break/>ann@x.y")),
      creator(true, "Cy Dee", &affiliation(" Univ B"))
    ]
  );
  // A run of address lines after a list's affiliations, one for each author, goes by order (round 3: both took the
  // last affiliation's label).
  assert_eq!(
    contacts(
      r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{2}}
\institute{$^1$ Univ A \\ $^2$ Univ B \\ \email{ann@x.y} \\ \email{bob@x.y}}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &format!("{}{}", affiliation(" Univ A"), email("ann@x.y"))
      ),
      creator(
        true,
        "Bob Baker",
        &format!("{}{}", affiliation(" Univ B"), email("bob@x.y"))
      )
    ]
  );
  // Rounds 4-5. An address claims an author only where the list says whose it is: under an affiliation of a list
  // that puts addresses under its affiliations, one address of its one author's. Several under one author's
  // affiliation, or one under an affiliation several share, continue it, as printed (the global email count had made
  // a creator with no name for `bob2@x.y`; by order among a shared mark's authors, 2609.06094's addresses went to
  // the wrong ones).
  let iopart = |authors: &str, addresses: &str| {
    contacts(&format!(
      "\\documentclass{{iopart}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{authors}}}\n{addresses}\n\\maketitle\nText.\n\\end{{document}}"
    ))
  };
  let shared = |rest: &str| format!("<creator role=\"author\">{rest}</creator>");
  assert_eq!(
    iopart(
      r"Ann Able$^1$, Bob Baker$^2$ and Cy Dee$^2$",
      r"\address{$^1$ Univ A\\ ann@x.y}\address{$^2$ Univ B\\ bob@x.y\\ cy@x.y}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &format!("{}{}", affiliation(" Univ A"), email("ann@x.y"))
      ),
      creator(
        true,
        "Bob Baker",
        &affiliation(" Univ B<break/>bob@x.y<break/>cy@x.y")
      ),
      creator(
        true,
        "Cy Dee",
        &affiliation(" Univ B<break/>bob@x.y<break/>cy@x.y")
      )
    ]
  );
  assert_eq!(
    iopart(
      r"Ann Able$^1$ and Bob Baker$^2$",
      r"\address{$^1$ Univ A\\ ann@x.y}\address{$^2$ Univ B\\ bob@x.y\\ bob2@x.y}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &format!("{}{}", affiliation(" Univ A"), email("ann@x.y"))
      ),
      creator(
        true,
        "Bob Baker",
        &affiliation(" Univ B<break/>bob@x.y<break/>bob2@x.y")
      )
    ]
  );
  // A list whose addresses only follow its affiliations gives them to the authors in order when there is one for
  // each, else to the shared creator below the authors (#159) — 2609.06094's shape: two authors request mark 2, two
  // addresses, a legend after. (The legend on the last author is the unmarked-row placement of before, not pinned
  // as intended.) A footnote-symbol legend after the run, `$^\dagger$ Corresponding author`, is no affiliation the
  // run sits above (round 6: Ann's address went to Bob).
  assert_eq!(
    contacts(
      r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{2} \and Cy Dee\inst{2}}
\institute{$^1$ Univ A \\ $^2$ Univ B \\ \email{ann@x.y, bob@x.y} \\ * Equal contribution}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation(" Univ A")),
      creator(true, "Bob Baker", &affiliation(" Univ B")),
      creator(
        true,
        "Cy Dee",
        &format!(
          "{}{}",
          affiliation("* Equal contribution"),
          affiliation(" Univ B")
        )
      ),
      shared(&email("ann@x.y, bob@x.y"))
    ]
  );
  assert_eq!(
    contacts(
      r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{2,\dagger}}
\institute{$^1$ Univ A \\ $^2$ Univ B \\ \email{ann@x.y, bob@x.y} \\ $^\dagger$ Corresponding author}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &format!("{}{}", affiliation(" Univ A"), email("ann@x.y"))
      ),
      creator(
        true,
        "Bob Baker",
        &format!(
          "{}{}{}",
          affiliation(" Univ B"),
          affiliation(" Corresponding author"),
          email("bob@x.y")
        )
      )
    ]
  );
  // Before any author, a marked `\address` keeps its affiliations, for the authors to come (round 5: dropped).
  assert_eq!(
    contacts(
      r"\documentclass{iopart}
\begin{document}
\title{T}
\address{$^1$ Univ A}
\address{$^2$ Univ B}
\author{Ann Able$^1$ and Bob Baker$^2$}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation(" Univ A")),
      creator(true, "Bob Baker", &affiliation(" Univ B"))
    ]
  );
  // An `\href{mailto:…}{…}` address is read by what it prints and kept as the link (round 4: `mailto:a@xa@x`).
  assert_eq!(
    contacts(
      r"\documentclass{iopart}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able$^1$ and Bob Baker$^2$}
\address{$^1$ Univ A\\ \href{mailto:ann@x.y}{ann@x.y}}
\address{$^2$ Univ B}
\maketitle
Text.
\end{document}"
    )[0],
    creator(
      false,
      "Ann Able",
      &format!(
        "{}{}",
        affiliation(" Univ A"),
        email(r#"<ref class="ltx_href" href="mailto:ann@x.y">ann@x.y</ref>"#)
      )
    )
  );
  // An `\author` call's tail comes before the later authors: its lines are its own author's rows, as before (round
  // 4: by the marks of the authors queued so far, Ann's address went to Bob too and Bob's to Ann).
  let (log, xml) = convert_files_with(
    r"\documentclass{myc}
\begin{document}
\title{T}
\author{Ann Able$^1$}
\affil{$^1$ Univ A\\ ann@x.y}
\author{Bob Baker$^1$}
\affil{$^1$ Univ A\\ bob@x.y}
\maketitle
Text.
\end{document}",
    &[(
      "myc.cls",
      r"\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{myc}
\LoadClass{article}
\renewcommand\author[1]{\ifx\@author\@empty\gdef\@author{#1}\else\g@addto@macro\@author{\and #1}\fi}
\let\@author\@empty
\newcommand\affil[1]{\g@addto@macro\@author{\\#1}}",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (each `Univ A` row goes to both authors by its mark, as before)
  let row = |text: &str| format!("<contact role=\"authorblock\">{text}</contact>");
  let rows = |own: &str| format!("{}{}{}", row(own), row(" Univ A"), row(" Univ A"));
  assert_eq!(creators_of(&xml), vec![
    creator(false, "Ann Able", &rows("ann@x.y")),
    creator(true, "Bob Baker", &rows("bob@x.y"))
  ]);
  // A presentational wrapper stays on each address; only an email command's is dropped (round 3: `\small` was lost).
  assert_eq!(
    contacts(
      r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{2}}
\institute{$^1$ Univ A \\ $^2$ Univ B \\ \small{ann@x.y, bob@x.y}}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &format!(
          "{}{}",
          affiliation(" Univ A"),
          email("<text fontsize=\"90%\" xml:id=\"id1\">ann@x.y</text>")
        )
      ),
      creator(
        true,
        "Bob Baker",
        &format!(
          "{}{}",
          affiliation(" Univ B"),
          email("<text fontsize=\"90%\" xml:id=\"id2\">bob@x.y</text>")
        )
      )
    ]
  );
  // A mark's first label is the one it sets (`$^{1,3}$`): Bob's request for 3 finds no affiliation, a residual shared
  // with Perl (Base_Utility.pool.ltxml:565-570 keeps the first label), though pdflatex prints Univ A as his too.
  assert_eq!(
    contacts(
      r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{3}}
\institute{$^{1,3}$ Univ A \\ ann@x.y}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &format!("{}{}", affiliation(" Univ A"), email("ann@x.y"))
      ),
      creator(true, "Bob Baker", "")
    ]
  );
  // More addresses than authors: one contact of the block, on the shared creator, without the `\email{…}` wrapper
  // (round 2: an empty contact beside it; round 5: on the last author).
  assert_eq!(
    contacts(
      r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{2}}
\institute{$^1$ Univ A \\ $^2$ Univ B \\ \email{a@x.y, b@x.y, c@x.y}}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(false, "Ann Able", &affiliation(" Univ A")),
      creator(true, "Bob Baker", &affiliation(" Univ B")),
      format!(
        "<creator role=\"author\">{}</creator>",
        email("a@x.y, b@x.y, c@x.y")
      )
    ]
  );
  assert_eq!(
    contacts(
      r"\documentclass{iopart}
\begin{document}
\title{T}
\author{Ann Able$^1$, Bob Baker$^2$ and Cy Dee$^3$}
\address{\textit{$^1$Univ A, $^2$Univ B} $^3$Univ C}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &affiliation("<text font=\"italic\" xml:id=\"id1\">Univ A</text>")
      ),
      creator(
        true,
        "Bob Baker",
        &affiliation("<text font=\"italic\" xml:id=\"id2\">Univ B</text>")
      ),
      creator(true, "Cy Dee", &affiliation("Univ C"))
    ]
  );
  assert_eq!(
    contacts(
      r"\documentclass{iopart}
\usepackage{xcolor}
\begin{document}
\title{T}
\author{Ann Able$^1$ and Bob Baker$^2$}
\address{{\color{blue} $^1$Univ A, $^2$Univ B}}
\maketitle
Text.
\end{document}"
    ),
    vec![
      creator(
        false,
        "Ann Able",
        &affiliation("<text color=\"#0000FF\" xml:id=\"id1\"> Univ A</text>")
      ),
      creator(
        true,
        "Bob Baker",
        &affiliation("<text color=\"#0000FF\" xml:id=\"id2\"> Univ B</text>")
      )
    ]
  );
  assert_eq!(
    contacts(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/marked_affiliation_list_email_line_leaves_no_empty_contact.tex"
    )),
    vec![
      creator(
        false,
        "Ann Able",
        &format!("{}{}", affiliation(" Univ A"), email("a@x.y"))
      ),
      creator(
        true,
        "Bob Baker",
        &format!("{}{}", affiliation(" Univ B"), email("b@x.y"))
      )
    ]
  );
}

/// The top-level units of a document, each as `<qname>:<title text without its tag>`, with `+biblist` on a
/// bibliography holding entries and its labels, if any.
fn units_of(xml: &str) -> Vec<String> {
  let unit = regex::Regex::new(
    r#"(?s)<(section|subsection|chapter|appendix|bibliography)\b([^>]*)>\s*(?:<tags>.*?</tags>\s*)?(?:<title>(.*?)</title>)?"#,
  )
  .unwrap();
  let tag = regex::Regex::new(r"(?s)<tag\b[^>]*>.*?</tag>").unwrap();
  let labels = regex::Regex::new(r#"labels="([^"]*)""#).unwrap();
  unit
    .captures_iter(xml)
    .map(|c| {
      let title = c.get(3).map_or(String::new(), |t| {
        tag.replace_all(t.as_str(), "").trim().to_string()
      });
      let mut shown = format!("{}:{title}", &c[1]);
      if &c[1] == "bibliography" {
        if let Some(l) = labels.captures(&c[2]) {
          shown.push_str(&format!(" [{}]", &l[1]));
        }
        if xml[c.get(0).unwrap().end()..]
          .split("</bibliography>")
          .next()
          .is_some_and(|b| b.contains("<biblist>"))
        {
          shown.push_str(" +biblist");
        }
      }
      shown
    })
    .collect()
}

/// The element children of the first bibliography, by local name (`["tags", "title", "biblist"]`).
fn bibliography_children(xml: &str) -> Vec<String> {
  let start = xml.find("<bibliography").expect("a bibliography");
  let end = xml[start..]
    .find("</bibliography>")
    .map_or(xml.len(), |e| start + e);
  let tag = regex::Regex::new(r"<(/?)([A-Za-z_][-\w:.]*)[^>]*?(/?)>").unwrap();
  let mut children = Vec::new();
  let mut depth = 0i32;
  for c in tag.captures_iter(&xml[start..end]) {
    if &c[1] == "/" {
      depth -= 1;
      continue;
    }
    if depth == 1 {
      children.push(c[2].to_string());
    }
    if &c[3] != "/" {
      depth += 1;
    }
  }
  children
}

/// 62zm: a bibliography right after a unit that heads it takes that unit's place (user 2026-10-06), or the document
/// has two headings, the first over nothing — Pandoc's `\section{References}` over the `{CSLReferences}` `\bibitem`s
/// that open the bibliography (2609.02899; Perl the same, KNOWN_PERL_ERRORS #512), `\bibitem`s in an `{enumerate}`
/// under any heading (the PDF prints that heading, not `\refname`), `\section*{References}` before `{thebibliography}`
/// and a bibliography with an empty `\refname` (both printed twice; OXIDIZED_DESIGN_DIVERGENCES #456). Not a unit
/// with other content and another title, nor an empty unit before a `{thebibliography}` titled otherwise. Repro
/// sectioning-frontmatter/bibliography_section_titled_as_it_becomes_it.
#[test]
fn bibliography_takes_the_place_of_its_heading_unit() {
  let units = |body: &str| -> Vec<String> {
    let tex = format!(
      "\\documentclass{{article}}\n\\begin{{document}}\nText \\cite{{a}}.\n\n{body}\n\\appendix\n\\section{{Extra}}\nMore.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, None);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    units_of(&xml)
  };
  let tail = "appendix:Extra";
  // The repro: Pandoc's list of `\bibitem`s under `\section{References}\label{references}`.
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bibliography_section_titled_as_it_becomes_it.tex"
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(units_of(&xml), vec![
    "bibliography:References [LABEL:references LABEL:refs] +biblist",
    tail
  ]);
  assert_eq!(bibliography_children(&xml), ["tags", "title", "biblist"]);
  // `\bibitem`s opening the bibliography under any heading take it, as the PDF prints that heading over them — in
  // `{enumerate}` and `{itemize}` too, which ended in an `\endgroup` error (62zn, KNOWN_PERL_ERRORS #513; repro
  // sectioning-frontmatter/bibitems_in_a_list_environment_close_without_error).
  for list in ["enumerate", "itemize", "description"] {
    let tex = format!(
      "\\documentclass{{article}}\n\\begin{{document}}\nText \\cite{{a}}.\n\n\\section{{Literature}}\\label{{lit}}\n\\begin{{{list}}}\n\\bibitem{{a}} Able, A title.\n\\end{{{list}}}\nAfter the list.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, None);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{list}: {log}"
    );
    assert_eq!(
      units_of(&xml),
      vec!["bibliography:Literature [LABEL:lit] +biblist"],
      "{list}"
    );
    assert_element(
      &xml,
      "bibitem",
      &["key=\"a\""],
      "<bibitem key=\"a\" xml:id=\"bib.bib1\"><tags><tag>[1]</tag><tag role=\"refnum\">1</tag></tags><bibblock> Able, A title.</bibblock></bibitem>",
    );
    // the text after the list is a paragraph after the bibliography
    let after = xml.split("</bibliography>").nth(1).unwrap_or_default();
    assert!(after.contains("<p>After the list.</p>"), "{list}: {xml}");
  }
  // The auto-open's group also ends the lists' redirection: a later list of that kind is itself again, and a second
  // such bibliography opens and ends too (without it both met an `\endgroup` error and the later lists nested into the
  // bibliography's list).
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
Text \cite{a}.

\section{Refs}
\begin{enumerate}
\bibitem{a} Able.
\end{enumerate}
\section{Later}
\begin{enumerate}
\item one
\item two
\end{enumerate}
\begin{itemize}
\item three
\end{itemize}
After.
\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S2\""],
    "<section inlist=\"toc\" xml:id=\"S2\"><tags><tag>2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">§2</tag></tags><title><tag close=\" \">2</tag>Later</title><para xml:id=\"S2.p1\"><enumerate xml:id=\"S2.I1\"><item xml:id=\"S2.I1.i1\"><tags><tag>1.</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">item\u{a0}1</tag></tags><para xml:id=\"S2.I1.i1.p1\"><p>one</p></para></item><item xml:id=\"S2.I1.i2\"><tags><tag>2.</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">item\u{a0}2</tag></tags><para xml:id=\"S2.I1.i2.p1\"><p>two</p></para></item></enumerate><itemize xml:id=\"S2.I2\"><item xml:id=\"S2.I2.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S2.I2.i1.p1\"><p>three</p></para></item></itemize><p>After.</p></para></section>",
  );
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
Text \cite{a}.

\section{Refs}
\begin{enumerate}
\bibitem{a} Able.
\end{enumerate}
\section{Refs2}
\begin{enumerate}
\bibitem{b} Baker.
\end{enumerate}
After.
\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(units_of(&xml), vec![
    "bibliography:Refs +biblist",
    "bibliography:Refs2 +biblist"
  ]);
  assert_element(
    &xml,
    "bibitem",
    &["key=\"b\""],
    "<bibitem key=\"b\" xml:id=\"biba.bib1\"><tags><tag>[1]</tag><tag role=\"refnum\">1</tag></tags><bibblock> Baker.</bibblock></bibitem>",
  );
  assert_eq!(
    units(
      r"\section{Literature}\label{lit}
\begin{list}{}{}
\bibitem{a} Able, A title.
\end{list}"
    ),
    vec!["bibliography:Literature [LABEL:lit] +biblist", tail]
  );
  // `\section*{References}` before `{thebibliography}`, and a heading over a bibliography with no title of its own.
  for (heading, refname) in [
    (r"\section*{References}", ""),
    (r"\section{Sources}", r"\renewcommand\refname{}"),
  ] {
    assert_eq!(
      units(&format!(
        r"{heading}
{refname}
\begin{{thebibliography}}{{9}}
\bibitem{{a}} Able, A title.
\end{{thebibliography}}"
      )),
      vec![
        format!(
          "bibliography:{} +biblist",
          if refname.is_empty() {
            "References"
          } else {
            "Sources"
          }
        ),
        tail.to_string()
      ],
      "{heading}"
    );
  }
  // The unit's other content goes before the entries.
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\section{References}
Listed in order of citation.
\begin{thebibliography}{9}
\bibitem{a} Able, A title.
\end{thebibliography}
\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(units_of(&xml), vec!["bibliography:References +biblist"]);
  assert_eq!(bibliography_children(&xml), [
    "tags", "title", "para", "biblist"
  ]);
  let bibliography = &xml[xml.find("<bibliography").unwrap()..];
  assert!(
    bibliography.find("Listed in order of citation.").unwrap()
      < bibliography.find("<biblist>").unwrap(),
    "{xml}"
  );
  // Not a unit with other content and another title, nor an empty unit before a bibliography titled otherwise.
  assert_eq!(
    units(
      r"\section{Conclusions}
Done.
\begin{list}{}{}
\bibitem{a} Able, A title.
\end{list}"
    ),
    vec![
      "section:Conclusions",
      "bibliography:References +biblist",
      tail
    ]
  );
  assert_eq!(
    units(
      r"\section{Sources}
\begin{thebibliography}{9}
\bibitem{a} Able, A title.
\end{thebibliography}"
    ),
    vec!["section:Sources", "bibliography:References +biblist", tail]
  );
  // Nor a unit with content before an untitled bibliography — the section, or its last subsection (round 1: the
  // conclusion became the bibliography).
  assert_eq!(
    units(
      r"\section{Conclusion}\label{sec:conc}
We conclude something important.
\renewcommand\refname{}
\begin{thebibliography}{9}
\bibitem{a} Able, A title.
\end{thebibliography}"
    ),
    vec!["section:Conclusion", "bibliography: +biblist", tail]
  );
  assert_eq!(
    units(
      r"\section{Discussion}
Intro.
\subsection{Limitations}
Some limits.
\renewcommand\refname{}
\begin{thebibliography}{9}
\bibitem{a} Able, A title.
\end{thebibliography}"
    ),
    vec![
      "section:Discussion",
      "subsection:Limitations",
      "bibliography: +biblist",
      tail
    ]
  );
}

/// 62zo: a document redefining IEEEtran's `\abstract`/`\IEEEkeywords` (or a "Note to Practitioners") copies the class's
/// internals — the abstract/keywords size `\@IEEEabskeysecsize` and the leading-break gobbler `\@IEEEgobbleleadPARNLSP`
/// (IEEEtran.cls:5263-5270, 5357-5379) — and sets the IED lists' label indents, dimens there (:2031-2059): all were
/// undefined in the binding, as in Perl's, whose empty `\IEEElabelindent` made `\IEEElabelindent\parindent` an
/// assignment to `\parindent` (KNOWN_PERL_ERRORS #514). Witnesses 2609.05249, 07516, 12903, 13235, 16206, 36487.
#[test]
fn ieeetran_internals_a_document_copies() {
  let (log, xml) = convert_with(
    r"\documentclass[journal]{IEEEtran}
\makeatletter
\newenvironment{manuscriptabstract}{\normalfont\@IEEEabskeysecsize\bfseries
  \textit{\abstractname:}\nobreakspace\relax\@IEEEgobbleleadPARNLSP}{\par}
\def\IEEEkeywords{\normalfont\@IEEEabskeysecsize\bfseries\textit{\IEEEkeywordsname:}\ \relax\@IEEEgobbleleadPARNLSP}
\def\endIEEEkeywords{\par}
\IEEEilabelindent\IEEEilabelindentB
\IEEElabelindent\parindent
\makeatother
\begin{document}
\title{T}
\maketitle
\begin{manuscriptabstract}
\\ \par Abstract text.
\end{manuscriptabstract}
\begin{IEEEkeywords}
alpha, beta
\end{IEEEkeywords}
\section{Body}
Body text. Indents \the\IEEEilabelindent, \the\IEEEilabelindentB{} and \the\IEEElabelindent, \the\parindent.
\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The leading `\\ \par` is gobbled, the blocks are in the abstract/keywords size, and the `{IEEEkeywords}` group
  // ends it: the body after is in the normal font (round 1: `\small` leaked to the end of the document).
  let paragraphs: Vec<String> = regex::Regex::new(r"(?s)<p>.*?</p>")
    .unwrap()
    .find_iter(&xml)
    .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join(" "))
    .collect();
  assert_eq!(paragraphs[..2], [
    r#"<p><text font="bold italic" fontsize="90%">Abstract:<text font="upright"> Abstract text.</text></text></p>"#,
    r#"<p><text font="bold italic" fontsize="90%">Index Terms:<text font="upright"> alpha, beta</text></text></p>"#,
  ]);
  assert!(paragraphs[2].starts_with("<p>Body text. Indents "), "{xml}");
  // `\IEEEilabelindent` took `\IEEEilabelindentB` (1.3\parindent), `\IEEElabelindent` took `\parindent`, and
  // `\parindent` itself stayed as it was.
  let indents = regex::Regex::new(r"Indents ([\d.]+pt), ([\d.]+pt) and ([\d.]+pt), ([\d.]+pt)\.")
    .unwrap()
    .captures(&xml)
    .expect("the indents paragraph");
  assert_eq!(&indents[1], &indents[2], "{xml}");
  assert_eq!(&indents[3], &indents[4], "{xml}");
  let pt = |i: usize| indents[i].trim_end_matches("pt").parse::<f64>().unwrap();
  assert!((pt(2) - 1.3 * pt(4)).abs() < 0.001, "{xml}");
}
