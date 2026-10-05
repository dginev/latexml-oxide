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
    let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
    let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
    let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
  let (log, xml) = latexml::util::test::convert_files_with(
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
