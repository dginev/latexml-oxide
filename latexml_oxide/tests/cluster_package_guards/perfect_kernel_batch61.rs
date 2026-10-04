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
