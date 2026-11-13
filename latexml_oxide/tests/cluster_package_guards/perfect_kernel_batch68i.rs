//! Red/green guards for batch 68i (binding gaps G3 C): TeX Live package bindings — tocbibind's heading names and
//! `\tocfile`, amsopn's internals, datetime's day of the week, ordinals and `\pdfdate`, rotating's `\rotdriver`,
//! paralist's blank lists and deprecated commands, todonotes' `\todostyle`, standalone inside a main document,
//! ccaption's continued and bilingual captions, and enumitem's inline lists and short labels under the classes that
//! load them.
//!
//! Every repro (`tools/perfect_kernel/repros/package-bindings/`) is checked against pdflatex (the lmcs ones with
//! 1611.06929's and 2003.01491's shipped lmcs.cls beside them).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The no-break space of `~` and of an item's reference names.
const NBSP: char = '\u{a0}';

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $name
    ))
  };
}

/// The XML of `tex` converted under ar5iv, which must give no error and exactly `warnings` warnings.
fn convert_checked(tex: &str, warnings: usize) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  xml
}

/// tocbibind.sty:125-132 and :250-316: `\settocname`/`\setlofname` rename the lists, `\tocfile{<title>}{<ext>}` is a
/// list of `<ext>` headed `<title>` — after `\tocsection` (:313-315), in a class without chapters, under a numbered
/// section — and `\settocbibname` heads the bibliography (pdflatex "Table of Contents", "Figures", "1 List of
/// Algorithms", "2 Intro", "3 Literature"; 67int9: five undefined, "References"; 68i: the list titled, unnumbered, and
/// "1 Intro"). The bibliography's number is not modelled (OXIDIZED_DESIGN_DIVERGENCES #620). 0710.3210, 1605.05950,
/// 1712.01817.
#[test]
fn tocbibind_names_and_tocfile() {
  let xml = convert_checked(repro!("tocbibind_names_and_tocfile.tex"), 0);
  assert_element(
    &xml,
    "TOC",
    &[r#"lists="toc""#],
    r#"<TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Table of Contents</title></TOC>"#,
  );
  assert_element(
    &xml,
    "TOC",
    &[r#"lists="lof""#],
    r#"<TOC lists="lof" scope="global"><title>Figures</title></TOC>"#,
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>List of Algorithms</title><TOC lists="loa" scope="global"/></section>"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>Literature</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, Title.
</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// tocbibind.sty:115-123: in a class with chapters the bibliography is `\bibname`'s, under natbib too (pdflatex
/// "Bibliography"; 67int9 "References", natbib's binding defining `\refname`).
#[test]
fn tocbibind_book_bibliography_is_bibname() {
  let xml = convert_checked(
    r"\documentclass{book}
\usepackage{natbib}
\usepackage{tocbibind}
\begin{document}
\chapter{Intro}
Text \cite{a}.
\begin{thebibliography}{9}
\bibitem[Author(2000)]{a} A. Author, Title.
\end{thebibliography}
\end{document}
",
    0,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>Bibliography</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2000</tag><tag role="authors">Author</tag><tag role="refnum">Author (2000)</tag><tag role="key">a</tag></tags><bibblock> A. Author, Title.</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// amsopn.sty:28-53 and :105-108: `\newmcodes@`, `\varlimsup@` and `\@declmathop` (pdflatex "max-flow(G), lim_n a_n,
/// coim f"; 67int9: three undefined). 1511.04387, 2005.13511, 2101.09853.
#[test]
fn amsopn_internals() {
  let xml = convert_checked(repro!("amsopn_internals.tex"), 0);
  assert_element(
    &xml,
    "XMTok",
    &[r#"name="coim""#],
    r#"<XMTok name="coim" role="OPFUNCTION" scriptpos="post">coim</XMTok>"#,
  );
  assert_element(
    &xml,
    "XMApp",
    &[r#"role="BIGOP""#],
    r#"<XMApp role="BIGOP" scriptpos="post1" xml:id="p1.m2.1.1.1.2"><XMTok name="overline" role="OVERACCENT" stretchy="true">¯</XMTok><XMTok role="OPFUNCTION" scriptpos="post">lim</XMTok></XMApp>"#,
  );
}

/// datetime.sty:64-187, :613-736 and :789-829: the long date with the day of the week and the ordinal, the day-of-week
/// names, `\pdfdate`, a named date without the day of the week, and the numeric formats — the pdflatex text (67int9:
/// `\dayofweekname`, `\ordinaldate`, `\pdfdate`, `\showdowfalse`, `\shortdayofweekname` undefined). The one warning is
/// the binding's partial-port notice; `\today` stays empty, as ar5iv leaves it for an undated source (ruling 64s).
/// 1308.5288, 1005.0302, 1601.00929, 2209.06700.
#[test]
fn datetime_names_and_pdfdate() {
  let xml = convert_checked(repro!("datetime_names_and_pdfdate.tex"), 1);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    &format!(
      r#"<p xml:id="p1.1">A: .
B: Friday, 16<sup xml:id="p1.1.1">th</sup> October, 2026.
C: 20261016123400.
D: 16{NBSP}October{NBSP}2026.
E: 3<sup xml:id="p1.1.2">rd</sup> February, 2001; Sat.
F: 2026/10/16; 16/10/26; October 16, 2026.</p>"#
    ),
  );
}

/// rotating.sty:52-64: `\rotdriver{dvips}` and the two-sided switch (67int9: both undefined); the sideways box is kept.
/// 0704.0929, 2306.14600.
#[test]
fn rotating_rotdriver() {
  let xml = convert_checked(repro!("rotating_rotdriver.tex"), 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Before <inline-block angle="90" depth="0.0pt" height="18.3pt" innerdepth="0.0pt" innerheight="6.9pt" innerwidth="18.3pt" width="6.9pt" xtranslate="-5.7pt" ytranslate="-5.7pt" xml:id="p1.1.1"><p xml:id="p1.1.1.1">Side</p></inline-block> after.</p>"#,
  );
}

/// paralist.sty:380-448 under `defblank`: `{inparablank}` and `{asparablank}` lists without labels, and the deprecated
/// `\defaultleftmargin` with its warning, as pdflatex warns (67int9: three undefined). 1205.4564, 1707.03997,
/// 1209.4819.
#[test]
fn paralist_blank_lists() {
  let xml = convert_checked(repro!("paralist_blank_lists.tex"), 1);
  assert_element(
    &xml,
    "inline-itemize",
    &[],
    r#"<inline-itemize xml:id="S0.I1"><inline-item xml:id="S0.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags><text xml:id="S0.I1.i1.1">apples, </text></inline-item><inline-item xml:id="S0.I1.i2"><tags><tag role="typerefnum">2nd item</tag></tags><text xml:id="S0.I1.i2.1">pears</text></inline-item></inline-itemize>"#,
  );
  assert_element(
    &xml,
    "itemize",
    &[],
    r#"<itemize xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I2.i1.p1"><p xml:id="S0.I2.i1.p1.1">First paragraph.</p></para></item><item xml:id="S0.I2.i2"><tags><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I2.i2.p1"><p xml:id="S0.I2.i2.p1.1">Second paragraph.</p></para></item></itemize>"#,
  );
}

/// todonotes.sty:46-49 and :581-583: `\todostyle` declares a style key and an empty `\phantomsection` stands in for
/// hyperref's (67int9: both undefined); the note keeps its text and its number (pdflatex "Text more." with the margin
/// note "Fix this."). A note given the style key (`\todo[mine]`) is numbered as any since 77v (repro
/// package-bindings/todonotes_style_key, guard `perfect_kernel_batch77v::todonotes_style_key`). 2407.14906, 1308.6029.
#[test]
fn todonotes_todostyle() {
  let xml = convert_checked(repro!("todonotes_todostyle.tex"), 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text<note inlist="todo" mark="1" role="todo" xml:id="todo1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">ToDo 1</tag></tags>Fix this.</note> more.</p>"#,
  );
}

/// standalone.sty:26-39, :296-304 and :707-709 in a main document: `\ifstandalone` false, `\onlyifstandalone` gobbles,
/// `\IfStandalone` takes its second branch, `{standalone}` keeps its body, `\standaloneconfig` sets the mode, and an
/// `\input` sub-file's `\standaloneenv` marks nothing (pdflatex "Main. Main again. Kept. Sub body."). 1310.0809,
/// 1905.04614, 2208.05046, 2310.18279, 2505.21582.
#[test]
fn standalone_in_main_document() {
  let xml = convert_checked(repro!("standalone_in_main_document.tex"), 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">Main.\nMain again.\nKept.\nSub body.</p>",
  );
}

/// ccaption.sty:66-96, :138-141, :204-217 and :309-430 under `[subfigure]`: the caption-shape commands, a continued
/// figure's subfloat numbered on from the previous float's ("1(c)") under that float's number ("Figure 1: Continued."),
/// and a bilingual caption numbered in both languages (67int9: six undefined, "Figure 2: Continued."). The one warning
/// is the binding's stub notice. 0907.2405, 1302.5179, 0803.1137.
#[test]
fn ccaption_continued_and_bilingual() {
  let xml = convert_checked(repro!("ccaption_continued_and_bilingual.tex"), 1);
  assert_element(
    &xml,
    "figure",
    &[r#"labels="LABEL:fc""#],
    r#"<figure labels="LABEL:fc" xml:id="S0.F1.sf3"><tags><tag><text fontsize="80%" xml:id="S0.F1.sf3.1">(c)</text></tag><tag role="refnum">1(c)</tag></tags><p xml:id="S0.F1.sf3.2">C</p><toccaption><tag close=" ">(c)</tag>Third</toccaption><caption><tag close=" "><text fontsize="80%" xml:id="S0.F1.sf3.3">(c)</text></tag><text fontsize="80%" xml:id="S0.F1.sf3.4">Third</text></caption></figure>"#,
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1a""#],
    r#"<figure inlist="lof" xml:id="S0.F1a"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure labels="LABEL:fc" xml:id="S0.F1.sf3"><tags><tag><text fontsize="80%" xml:id="S0.F1.sf3.1">(c)</text></tag><tag role="refnum">1(c)</tag></tags><p xml:id="S0.F1.sf3.2">C</p><toccaption><tag close=" ">(c)</tag>Third</toccaption><caption><tag close=" "><text fontsize="80%" xml:id="S0.F1.sf3.3">(c)</text></tag><text fontsize="80%" xml:id="S0.F1.sf3.4">Third</text></caption></figure><toccaption><tag close=" ">1</tag>Continued.</toccaption><caption><tag close=": ">Figure 1</tag>Continued.</caption></figure>"#,
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Tableau 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Tableau 1</tag></tags><toccaption><tag close=" ">1</tag>Short</toccaption><caption><tag close=": ">Table 1</tag>English caption.</caption><toccaption><tag close=" ">1</tag>Court</toccaption><caption><tag close=": ">Tableau 1</tag>French caption.</caption><p xml:id="S0.T1.1">x</p></table>"#,
  );
}

/// lmcs.cls:37-38 loads enumitem with `inline`: `{enumerate*}` is an inline list (pdflatex "Either (a) red, (b) green
/// or none."; 67int9: undefined). 1611.06929, 1711.09946.
#[test]
fn enumitem_inline_under_lmcs() {
  let xml = convert_checked(repro!("enumitem_inline_under_lmcs.tex"), 0);
  assert_element(
    &xml,
    "inline-enumerate",
    &[],
    &format!(
      r#"<inline-enumerate xml:id="S0.I1"><inline-item xml:id="S0.I1.i1"><tags><tag>(a)</tag><tag role="autoref">item{NBSP}(a)<text xml:id="S0.I1.i1.1"/></tag><tag role="refnum">(a)</tag><tag role="typerefnum">item{NBSP}(a)</tag></tags><text xml:id="S0.I1.i1.2">red, </text></inline-item><inline-item xml:id="S0.I1.i2"><tags><tag>(b)</tag><tag role="autoref">item{NBSP}(b)<text xml:id="S0.I1.i2.1"/></tag><tag role="refnum">(b)</tag><tag role="typerefnum">item{NBSP}(b)</tag></tags><text xml:id="S0.I1.i2.2">green</text></inline-item></inline-enumerate>"#
    ),
  );
}

/// The newer lmcs.cls also passes `shortlabels` (2003.01491's lmcs.cls:45-47, 2604.16477's :51-56), the binding the
/// union of the versions: `\begin{enumerate}[(a)]` numbers "(a)", "(b)" (pdflatex with 2003.01491's class; 68i
/// "(1)", "(2)" and an unknown-key warning).
#[test]
fn enumitem_shortlabels_under_lmcs() {
  let xml = convert_checked(repro!("enumitem_shortlabels_under_lmcs.tex"), 0);
  // The reference number is the short label, as pdflatex's `\ref` "(a)": enumitem makes the short label `label=`
  // (enumitem.sty:660-681; KNOWN_PERL_ERRORS #808, fixed 77u; repro package-bindings/enumitem_shortlabel_ref).
  assert_element(
    &xml,
    "enumerate",
    &[],
    &format!(
      r#"<enumerate xml:id="S1.I1"><item xml:id="S1.I1.i1"><tags><tag>(a)</tag><tag role="autoref">item{NBSP}(a)<text xml:id="S1.I1.i1.1"/></tag><tag role="refnum">(a)</tag><tag role="typerefnum">item{NBSP}(a)</tag></tags><para xml:id="S1.I1.i1.p1"><p xml:id="S1.I1.i1.p1.1">red</p></para></item><item xml:id="S1.I1.i2"><tags><tag>(b)</tag><tag role="autoref">item{NBSP}(b)<text xml:id="S1.I1.i2.1"/></tag><tag role="refnum">(b)</tag><tag role="typerefnum">item{NBSP}(b)</tag></tags><para xml:id="S1.I1.i2.p1"><p xml:id="S1.I1.i2.p1.1">green</p></para></item></enumerate>"#
    ),
  );
}

/// ccaption without `[subfigure]`: the caption-shape commands are defined and the option's skipped definitions leave
/// no conditional open (a draft's `\iftrue` in them left "Missing \fi" at the load: 0803.1137, 1403.7561, 1706.01843).
/// The one warning is the binding's stub notice.
#[test]
fn ccaption_without_subfigure_option() {
  let xml = convert_checked(
    r"\documentclass{article}
\usepackage{ccaption}
\captionwidth{.75\textwidth}\changecaptionwidth
\begin{document}
\begin{figure}x\caption{\centerlastline A.}\end{figure}
\end{document}
",
    1,
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.1">x</p><toccaption><tag close=" ">1</tag>A.</toccaption><caption><tag close=": ">Figure 1</tag>A.</caption></figure>"#,
  );
}

/// todonotes.sty:159-160: `\if@todonotes@disabled` is false unless `disable` (67int9: undefined, its `\else` and `\fi`
/// stray). 2606.13443.
#[test]
fn todonotes_disabled_switch() {
  let xml = convert_checked(
    r"\documentclass{article}
\usepackage{todonotes}
\makeatletter
\if@todonotes@disabled\newcommand{\fix}[1]{#1}\else\newcommand{\fix}[1]{[#1]}\fi
\makeatother
\begin{document}
\fix{Draft}
\end{document}
",
    0,
  );
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">[Draft]</p>"#);
}
