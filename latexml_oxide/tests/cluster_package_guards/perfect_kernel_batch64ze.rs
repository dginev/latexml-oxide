//! Red/green guards for perfect-kernel batch 64ze: names a class (or natbib) defines that its binding lacked, ranked
//! by the K22 census of shipped class copies and verified against run 336's first errors. Repros
//! `tools/perfect_kernel/repros/class-bindings/*.tex`, `index-bib/natbib_bibitemshut_without_preamble.tex` and
//! `kernel-alignment/noalign_tabularcr.tex`.
use latexml::util::test::{assert_element, xml_element};

use super::perfect_kernel_batch46::{convert_files_with, convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

macro_rules! repro {
  ($topic:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $topic,
      "/",
      $name,
      ".tex"
    ))
  };
}

/// natbib.sty:880-958: apsrev4-1 entries pasted without the `.bbl` preamble meet natbib's `\BibitemOpen`,
/// `\BibitemShut`, `\bibinfo` and `\bibfield`, and each entry ends with the stop its `\BibitemShut{NoStop}` asked for
/// (1008.0439; base: 3 errors).
#[test]
fn natbib_bibitemshut_ends_each_entry() {
  let xml = convert_clean(repro!("index-bib", "natbib_bibitemshut_without_preamble"));
  assert_eq!(xml.matches("<bibitem ").count(), 2, "{xml}");
  assert_element(&xml, "bibblock", &[], "<bibblock>First note. </bibblock>");
  // `J.~A.`: a no-break space
  assert!(
    xml.contains(
      "<bibblock>J.\u{a0}A. Soules, Am. J. Phys. <text font=\"bold\" xml:id=\"bib.bib2.1\">58</text>, 1195 (1990). \
       </bibblock>"
    ),
    "{xml}"
  );
}

/// svmult.cls:182-208, 1296-1298, 1731-1733: `{svgraybox}` shaded under `graybox`, `\svhline` a table rule, `{prob}`
/// numbered without a caption (referred to by its number alone) and `{sol}` headed by its number (0801.0527,
/// 1511.08212, 1509.02295; base: 5 errors).
#[test]
fn svmult_graybox_svhline_and_problems() {
  let xml = convert_clean(repro!("class-bindings", "svmult_graybox_svhline_prob"));
  assert_element(
    &xml,
    "para",
    &["backgroundcolor=\"#D9D9D9\""],
    r##"<para backgroundcolor="#D9D9D9" cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle" xml:id="Ch0.S1.p1"><p xml:id="Ch0.S1.p1.1">Boxed TEXT.</p></para>"##,
  );
  assert_element(
    &xml,
    "tbody",
    &[],
    r#"<tbody><tr xml:id="Ch0.S1.p2.1.1"><td align="left" border="b t" xml:id="Ch0.S1.p2.1.1.1">a</td><td align="left" border="b t" xml:id="Ch0.S1.p2.1.1.2">b</td></tr></tbody>"#,
  );
  assert_element(
    &xml,
    "theorem",
    &[],
    r#"<theorem class="ltx_theorem_prob" inlist="thm theorem:prob" labels="LABEL:p1" xml:id="Thmprob1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmprob1.1">1</text></tag></title><para xml:id="Thmprob1.p1"><p xml:id="Thmprob1.p1.1">Problem TEXT.</p></para></theorem>"#,
  );
  assert_element(
    &xml,
    "para",
    &["class=\"ltx_noindent\""],
    r#"<para class="ltx_noindent" xml:id="Ch0.S1.p3"><p xml:id="Ch0.S1.p3.1"><ref font="bold" labelref="LABEL:p1"/>Solution TEXT.</p></para>"#,
  );
}

/// svmult.cls:822-836 (v5.4): without `graybox`, `{svgraybox}` warns and sets a quotation (0801.0527; base: 1 error).
#[test]
fn svmult_svgraybox_without_graybox_warns() {
  let (log, xml) = convert_with(
    repro!("class-bindings", "svmult_svgraybox_without_graybox"),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains("Environment \"svgraybox\" not available"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><quote xml:id="p1.1"><p xml:id="p1.1.1">Boxed TEXT.</p></quote></para>"#,
  );
}

/// svmult.cls:538-541, 570-571, 634-635, 649: `\motto` and `\chapauthor` are set under the next chapter's title, inside
/// it — a starred chapter sets only the motto, the authors wait for the next numbered head — the authors a creator of
/// the chapter, the motto a quotation (0911.4644, 1010.1304; base: 3 errors).
#[test]
fn svmult_motto_and_chapauthor_open_the_chapter() {
  let xml = convert_clean(repro!("class-bindings", "svmult_motto_chapauthor"));
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Chx1\""],
    r#"<chapter xml:id="Chx1"><title>Preface</title><para xml:id="Chx1.p1"><quote xml:id="Chx1.p1.1"><p class="ltx_align_left" xml:id="Chx1.p1.1.1"><text font="italic" fontsize="90%" xml:id="Chx1.p1.1.1.1">A motto TEXT.</text></p></quote><p xml:id="Chx1.p1.2">Preface TEXT.</p></para></chapter>"#,
  );
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Ch1\""],
    r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><creator role="author"><personname>Ann Author and Bob Bauer</personname></creator><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Chapter TEXT.</p></para></chapter>"#,
  );
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Ch2\""],
    r#"<chapter inlist="toc" xml:id="Ch2"><tags><tag>Chapter 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Chapter 2</tag></tags><title><tag close=" ">Chapter 2</tag>Second</title><toctitle><tag close=" ">2</tag>Second</toctitle><para xml:id="Ch2.p1"><quote xml:id="Ch2.p1.1"><p class="ltx_align_left" xml:id="Ch2.p1.1.1"><text font="italic" fontsize="90%" xml:id="Ch2.p1.1.1.1">Second motto.</text></p></quote><p xml:id="Ch2.p1.2">Second TEXT.</p></para></chapter>"#,
  );
}

/// svmono.cls is a book like its twin svmult.cls: front matter, a numbered chapter with its section, back matter
/// (1508.02595, 0801.1245; base: 6 errors, svmono read as the journal class svjour3).
#[test]
fn svmono_is_a_book() {
  let xml = convert_clean(repro!("class-bindings", "svmono_is_a_book"));
  assert_eq!(xml.matches("<chapter ").count(), 3, "{xml}");
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Chx1\""],
    r#"<chapter xml:id="Chx1"><title>Preface</title><para xml:id="Chx1.p1"><p xml:id="Chx1.p1.1">Preface TEXT.</p></para></chapter>"#,
  );
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Ch1\""],
    r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Chapter TEXT.</p></para><section inlist="toc" xml:id="Ch1.S1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title><tag close=" ">1.1</tag>First</title><para xml:id="Ch1.S1.p1"><p xml:id="Ch1.S1.p1.1">Section TEXT.</p></para></section></chapter>"#,
  );
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Chx2\""],
    r#"<chapter xml:id="Chx2"><title>Glossary</title><para xml:id="Chx2.p1"><p xml:id="Chx2.p1.1">Glossary TEXT.</p></para></chapter>"#,
  );
}

/// aipxfm.sty:266-269: `\let\pacs\classification` (cond-mat0701310; base: 1 error).
#[test]
fn aipproc_pacs_is_the_classification() {
  let xml = convert_clean(repro!("class-bindings", "aipproc_pacs_is_classification"));
  assert_element(
    &xml,
    "classification",
    &[],
    "<classification name=\"Classification:\u{a0}\">71.10.Li, 74.72.-h</classification>",
  );
}

/// svjour.cls:61-62, svjour3.cls:392-393: `\preprint{<number>}` a preprint publication note (1801.06113; base: 1
/// error).
#[test]
fn svjour_preprint_is_a_pubnote() {
  let xml = convert_clean(repro!("class-bindings", "svjour_preprint_is_a_pubnote"));
  assert_element(
    &xml,
    "pubnote",
    &[],
    "<pubnote name=\"Preprint:\u{a0}\" role=\"preprint\">CERN-TH-2024-001</pubnote>",
  );
}

/// interact.cls:134-138: the class's own bold-math `\bm` (2310.17051; base: 1 error).
#[test]
fn class_defined_bm_is_bold_math() {
  let xml = convert_clean(repro!("class-bindings", "class_defined_bm"));
  assert_element(
    &xml,
    "XMTok",
    &["font=\"bold italic\""],
    r#"<XMTok font="bold italic" role="UNKNOWN">x</XMTok>"#,
  );
}

/// IEEEAerospaceCLS.cls:267-272, 403-408, 541-549: `\acknowledgements` and `\thebiography` open unnumbered sections,
/// `{biographywithpic}{<name>}{<file>}` is the IEEEtran biography (2601.06000; base: 3 errors). The unnumbered
/// headings take the section font (78a; the class's is `\large\bfseries\scshape`, :489-490, the binding's IEEEtran
/// small caps).
#[test]
fn ieeeaerospace_acknowledgements_and_biography() {
  let xml = convert_clean(repro!(
    "class-bindings",
    "ieeeaerospace_acknowledgements_biography"
  ));
  assert_element(
    &xml,
    "section",
    &["xml:id=\"Sx1\""],
    r#"<section xml:id="Sx1"><title font="smallcaps">Acknowledgements</title><para xml:id="Sx1.p1"><p xml:id="Sx1.p1.1">Thanks TEXT.</p></para></section>"#,
  );
  // The whole biography; the graphic's `candidates` path depends on where the output is written.
  let section = xml_element(&xml, "section", &["xml:id=\"Sx2\""]).expect("the biography section");
  let section = regex::Regex::new(r#" candidates="[^"]*""#)
    .unwrap()
    .replace(&section, "")
    .into_owned();
  assert_eq!(
    section,
    "<section xml:id=\"Sx2\"><title font=\"smallcaps\">Biography</title><float class=\"biography\" xml:id=\"Sx2.tab1\"><tabular \
     xml:id=\"Sx2.tab1.1\"><tr xml:id=\"Sx2.tab1.1.1\"><td xml:id=\"Sx2.tab1.1.1.1\"><graphics graphic=\"example-image\" \
     options=\"width=71.13188pt,height=91.04881pt\" xml:id=\"Sx2.g1\"/></td><td xml:id=\"Sx2.tab1.1.1.2\"><inline-block \
     xml:id=\"Sx2.tab1.1.1.2.1\"><p xml:id=\"Sx2.tab1.1.1.2.1.1\"><text class=\"ltx_font_bold\" \
     font=\"italic\" xml:id=\"Sx2.tab1.1.1.2.1.1.1\">Ann Author</text>Bio TEXT.</p></inline-block></td></tr></tabular></float></section>",
    "{xml}"
  );
}

/// quantumarticle.cls:197-205, 1392-1495: a REVTeX journal option provides `\pacs` and `\preprint`, which print
/// nothing — their codes kept as RDFa metadata (`dcterms:subject`, `dcterms:identifier`) — the journal abbreviations,
/// natbib (`numbers`) and REVTeX's `\textcite`/`\onlinecite` (1705.03968, 2006.05524; base: 5 errors).
#[test]
fn quantumarticle_revtex_compatibility() {
  let xml = convert_clean(repro!("class-bindings", "quantumarticle_revtex_compat"));
  assert_eq!(xml.matches("<classification").count(), 0, "{xml}");
  assert_eq!(xml.matches("<pubnote").count(), 0, "{xml}");
  assert_element(
    &xml,
    "rdf",
    &["property=\"dcterms:subject\""],
    r#"<rdf property="dcterms:subject">03.67.-a</rdf>"#,
  );
  assert_element(
    &xml,
    "rdf",
    &["property=\"dcterms:identifier\""],
    r#"<rdf property="dcterms:identifier">QQ-2020-1</rdf>"#,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A. Author</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Somewhere</contact></creator>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">Body TEXT in Phys. Rev. Lett., see <cite class="ltx_citemacro_citet"><bibref bibrefs="a" separator="," show="Authors Phrase1NumberPhrase2" yyseparator=","><bibrefphrase>[</bibrefphrase><bibrefphrase>]</bibrefphrase></bibref></cite> and <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," show="Number" yyseparator=","/>]</cite>.</p></para>"#,
  );
}

/// Each `\pacs` adds its codes beside the author's own hyperref `pdfkeywords`, none replacing another (review 64ze round
/// 3, `/tmp/rev64ze3/q/q1.tex`).
#[test]
fn quantumarticle_pacs_keep_the_authors_keywords() {
  let xml = convert_clean(
    "\\documentclass[aps,onecolumn]{quantumarticle}\n\\usepackage{hyperref}\n\\hypersetup{pdfkeywords={quantum, \
     entanglement}}\n\\begin{document}\n\\title{T}\n\\author{A. Author}\n\\maketitle\n\\pacs{03.67.-a}\n\\pacs{42.50.Dv}\n\
     Body TEXT.\n\\end{document}\n",
  );
  let rdf = regex::Regex::new(r"<rdf [^>]*(?:/>|>[^<]*</rdf>)").unwrap();
  let found: Vec<&str> = rdf.find_iter(&xml).map(|m| m.as_str()).collect();
  assert_eq!(
    found,
    vec![
      r#"<rdf property="dcterms:subject">03.67.-a</rdf>"#,
      r#"<rdf property="dcterms:subject">42.50.Dv</rdf>"#,
      r#"<rdf about="" content="quantum, entanglement" property="dcterms:subject"/>"#,
    ],
    "{xml}"
  );
}

/// elsarticle.cls:1303: a bare `\frontmatter` is the environment's empty begin macro (1111.1604; base: 1 error).
#[test]
fn elsarticle_bare_frontmatter_is_a_no_op() {
  let xml = convert_clean(repro!("class-bindings", "elsarticle_bare_frontmatter"));
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>A. Author</personname></creator>"#,
  );
}

/// mn2e.cls:1575-1686 of the 1696-line v2.2 copy: the AAS journal abbreviations, a fallback when the paper ships no
/// copy (0711.3034; base: 2 errors).
#[test]
fn mn2e_journal_abbreviations() {
  let xml = convert_clean(repro!("class-bindings", "mn2e_journal_abbreviations"));
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> Smith J., 2001, ApJ, 550, 1; Jones K., 2002, MNRAS, 330, 2\n</bibblock>",
  );
}

/// A document's own abbreviation, with no mn2e.cls shipped: the binding's fallback yields to it (0901.1150 `ApJL`).
#[test]
fn mn2e_document_abbreviation_wins_without_a_shipped_class() {
  let xml = convert_clean(
    "\\documentclass{mn2e}\n\\newcommand{\\apjl}{ApJL}\n\\begin{document}\nIn \\apjl{} and \\apj.\n\\end{document}\n",
  );
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">In ApJL and ApJ.</p>"#);
}

/// A shipped mn2e.cls without the block (the common 1574-line v2.2 copy, here its stand-in) defines no abbreviation:
/// the document's own stands and the others stay undefined, as in pdflatex.
#[test]
fn mn2e_shipped_copy_without_the_block_defines_none() {
  let (log, xml) = convert_files_with(
    "\\documentclass{mn2e}\n\\newcommand{\\apjl}{ApJL}\n\\begin{document}\nIn \\apjl{} and \\apj.\n\\end{document}\n",
    &[(
      "mn2e.cls",
      "\\ProvidesClass{mn2e}[2001/02/06 v2.2 LaTeX2e MN document class]\n\\LoadClass{article}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(log.contains("Error:undefined:\\apj "), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">In ApJL and <ERROR class="undefined" xml:id="p1.1.1">\apj</ERROR>.</p>"#,
  );
}

/// A shipped mn2e.cls that defines the abbreviations itself gives the document those lines, as the copy writes them,
/// with the internal macro they are built on: stand-ins for the 1695-line v2.2 copy's `\def\apj{ApJ}%` (0804.2668) and
/// another copy's `\def\reff@jnl#1{{\rm#1\/}}` (1209.3072).
#[test]
fn mn2e_shipped_copy_with_the_block_gives_its_own() {
  let (log, xml) = convert_files_with(
    "\\documentclass{mn2e}\n\\begin{document}\nIn \\apj{} and \\mnras.\n\\end{document}\n",
    &[(
      "mn2e.cls",
      "\\ProvidesClass{mn2e}[2001/02/06 v2.2 LaTeX2e MN document class]\n\\LoadClass{article}\n\\def\\apj{ApJ}%\n          \
       % Astrophysical Journal\n\\def\\reff@jnl#1{{\\rm#1\\/}}\n\\def\\mnras{\\reff@jnl{MNRAS (copy)}}%\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">In ApJ and MNRAS (copy).</p>"#,
  );
}

/// `\noalign\\` in a text `tabular` with the kernel's `\@tabularcr`: an empty noalign and the row end, as latex.ltx's
/// brace makes it, pdflatex clean (2309.15248; base: 11 errors).
#[test]
fn noalign_followed_by_tabularcr() {
  let xml = convert_clean(repro!("kernel-alignment", "noalign_tabularcr"));
  assert!(xml.contains("After TEXT."), "{xml}");
  assert_element(
    &xml,
    "tbody",
    &[],
    r#"<tbody><tr xml:id="p1.1.1"><td align="left" xml:id="p1.1.1.1">a</td><td align="left" xml:id="p1.1.1.2">b</td></tr><tr xml:id="p1.1.2"><td align="left" border="t" xml:id="p1.1.2.1">c</td><td align="left" border="t" xml:id="p1.1.2.2">d</td></tr></tbody>"#,
  );
}

/// The kernel `\@tabularcr` reached past `\relax`, as `\tabularnewline`, or in a bare `\tabular…\endtabular`: pdflatex
/// clean, so no error (review 64ze round 2, `/tmp/rev64ze2/na/{t6,n_tnl,n_bare}`).
#[test]
fn noalign_followed_by_tabularcr_forms() {
  for table in [
    "\\begin{tabular}{|l|l|}\na & b \\\\\n\\noalign\\relax\\\\\nc & d \\\\\n\\end{tabular}",
    "\\begin{tabular}{|l|l|}\na & b \\\\\n\\noalign\\tabularnewline\nc & d \\\\\n\\end{tabular}",
    "\\tabular{ll}\na & b \\\\\n\\noalign\\\\\nc & d \\\\\n\\endtabular",
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n{table}\nAfter TEXT.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), 0, "{tex}\n{log}");
    assert_eq!(warning_count(&log), 0, "{tex}\n{log}");
    assert!(xml.contains("After TEXT."), "{tex}\n{xml}");
  }
}

/// The other `\\`s open with no brace (the kernel's and array.sty's `\@arraycr`, longtable's, amsmath's `\math@cr`):
/// `\noalign\\` is TeX's "Missing { inserted", one error each, and the document goes on (pdflatex: 6, 41, 106, 5
/// errors).
#[test]
fn noalign_followed_by_other_crs_is_missing_brace() {
  for (preamble, table, id, paragraph) in [
    (
      "",
      "$\\begin{array}{ll}\na & b \\\\\n\\noalign\\\\\nc & d \\\\\n\\end{array}$",
      "p1.1",
      r#"<p xml:id="p1.1"><Math mode="inline" tex="\begin{array}[]{ll}a&amp;b\\&#10;\cr\\&#10;c&amp;d\\&#10;\end{array}" text="Array[[a, b], [], [, ], [c, d]]" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMArray role="ARRAY" vattach="middle" xml:id="p1.m1.1.1"><XMRow xml:id="p1.m1.1.1.1"><XMCell align="left" xml:id="p1.m1.1.1.1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="left" xml:id="p1.m1.1.1.1.2"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow><XMRow xml:id="p1.m1.1.1.2"><XMCell colspan="2" xml:id="p1.m1.1.1.2.1"/></XMRow><XMRow xml:id="p1.m1.1.1.3"><XMCell xml:id="p1.m1.1.1.3.1"/><XMCell xml:id="p1.m1.1.1.3.2"/></XMRow><XMRow xml:id="p1.m1.1.1.4"><XMCell align="left" xml:id="p1.m1.1.1.4.1"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell><XMCell align="left" xml:id="p1.m1.1.1.4.2"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow></XMArray></XMath></Math>After TEXT.</p>"#,
    ),
    (
      "\\usepackage{amsmath}",
      "\\begin{align}\na & b \\\\\n\\noalign\\\\\nc & d\n\\end{align}",
      "p1.1",
      r#"<p xml:id="p1.1">After TEXT.</p>"#,
    ),
    (
      "\\usepackage{array}",
      "\\begin{tabular}{|l|l|}\na & b \\\\\n\\noalign\\\\\nc & d \\\\\n\\end{tabular}",
      "p1.2",
      r#"<p xml:id="p1.2">After TEXT.</p>"#,
    ),
    (
      "\\usepackage{longtable}",
      "\\begin{longtable}{|l|l|}\na & b \\\\\n\\noalign\\\\\nc & d \\\\\n\\end{longtable}",
      "p1.1",
      r#"<p xml:id="p1.1">After TEXT.</p>"#,
    ),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n{preamble}\\begin{{document}}\n{table}\nAfter TEXT.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), 1, "{tex}\n{log}");
    assert!(
      log.contains("Error:expected:{ Missing { inserted"),
      "{tex}\n{log}"
    );
    assert_element(&xml, "p", &[&format!("xml:id=\"{id}\"")], paragraph);
  }
}

/// The author's own `.sty` outranks a binding's fallback guess (K22): mn2e's abbreviations, installed as a fallback when
/// no mn2e.cls is shipped, yield to the paper's `mystyle.sty` (0901.4335's `mn2e_journal_style.sty`).
#[test]
fn mn2e_fallback_yields_to_the_authors_own_style() {
  let (log, xml) = convert_files_with(
    "\\documentclass{mn2e}\n\\usepackage{mystyle}\n\\begin{document}\nRefs: \\araa, \\nat, \\mnras.\n\\end{document}\n",
    &[(
      "mystyle.sty",
      "\\newcommand{\\araa}{ARAA}\n\\providecommand{\\nat}{Nat}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">Refs: ARAA, Nat, MNRAS.</p>"#,
  );
}

/// The same for a World Scientific one-variant fallback (sprocl's `\id`, ws_procl_sty.rs): the paper's own style's
/// `\newcommand{\id}` stands.
#[test]
fn ws_fallback_yields_to_the_authors_own_style() {
  let (log, xml) = convert_files_with(
    "\\documentclass{article}\n\\usepackage{sprocl}\n\\usepackage{mymacros}\n\\begin{document}\nThe unit \
     \\id.\n\\end{document}\n",
    &[("mymacros.sty", "\\newcommand{\\id}{\\textbf{1}}\n")],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">The unit <text font="bold" xml:id="p1.1.1">1</text>.</p>"#,
  );
}

/// natbib's deferred stop and a lonely `\bibitem` (64x): an entry's `\BibitemShut{NoStop}` is spent by the next entry
/// of its list (natbib.sty:864) or the list's end (:1080). A `\bibitem` in no list opens a bibliography of its own (one
/// Error each, LaTeX's "Lonely \item"), which a heading, `\end{document}` or the end of the environment around it closes
/// — none a list end, so no "." is printed there, and the first bibliography's pending stop does not reach the second
/// (pdflatex `/tmp/rev64x10/pdf_stop/stop1`, `stop`: "Able, A. 2011, ApJ", "Cole, C. 2013, ApJ").
#[test]
fn natbib_lonely_entries_spend_no_stop() {
  let entry = |key: &str, text: &str| {
    format!(
      "\\bibitem[{key}]{{{}}} {text}\\BibitemShut{{NoStop}}\n",
      &key[..1]
    )
  };
  let bibblock = regex::Regex::new(r"(?s)<bibblock>.*?</bibblock>").unwrap();
  for (body, errors, blocks) in [
    (
      format!(
        "\\section{{Summary}}\nText.\n\n{}\n\\section{{Appendix}}\nMore text.\n",
        entry("a(2011)", "Able, A. 2011, ApJ")
      ),
      1,
      vec!["<bibblock> Able, A. 2011, ApJ\n</bibblock>"],
    ),
    (
      format!(
        "\\section{{Summary}}\nText.\n\n{}\n\\section{{Appendix}}\nMore text.\n\n{}",
        entry("a(2011)", "Able, A. 2011, ApJ"),
        entry("c(2013)", "Cole, C. 2013, ApJ")
      ),
      2,
      vec![
        "<bibblock> Able, A. 2011, ApJ\n</bibblock>",
        "<bibblock> Cole, C. 2013, ApJ\n</bibblock>",
      ],
    ),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage{{natbib}}\n\\begin{{document}}\n{body}\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), errors, "{tex}\n{log}");
    assert_eq!(
      log.matches("Error:malformed:ltx:bibitem").count(),
      errors,
      "{log}"
    );
    let found: Vec<String> = bibblock
      .find_iter(&xml)
      .map(|m| m.as_str().to_string())
      .collect();
    assert_eq!(found, blocks, "{tex}\n{xml}");
  }
}

/// The same lonely entry closed by the end of the environment around it (`{small}`, its saved end): no stop either.
#[test]
fn natbib_lonely_entry_closed_by_its_environment_spends_no_stop() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{natbib}\n\\begin{document}\nText.\n\\begin{small}\n\\bibitem[Able(2011)]{a} \
     Able, A. 2011, ApJ\\BibitemShut{NoStop}\n\\end{small}\nAfter TEXT.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_element(
    &xml,
    "bibblock",
    &[],
    r#"<bibblock><text fontsize="90%" xml:id="bib.bib1.6"> Able, A. 2011, ApJ</text></bibblock>"#,
  );
  assert!(xml.contains("After TEXT."), "{xml}");
}
