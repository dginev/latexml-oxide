//! Red/green guards for batch 69a: bibliographies and headings inside a `\list`-based container (`{quote}`,
//! `{quotation}`, `{verse}`, a class's bibliography list), and small package shapes from the rc_struct2 study
//! (2026-10-16). Each repro's pdflatex reference text is in its header.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion of a repro (`<topic>/<file>`), squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

fn assert_all(xml: &str, parts: &[&str]) {
  for part in parts {
    assert!(xml.contains(part), "{part}\n{xml}");
  }
}

/// The AAAI template's `\begin{quote}\begin{small}` bibliography (2203.09193, 1904.12066, 1610.09996, 2109.09861,
/// 2403.09260, 1609.06434; astro-ph0410583): the quote is closed and, empty, dropped; the bibliography is the
/// document's, its entries at the quote's `\small`.
#[test]
fn bibliography_in_a_quote_closes_the_quote() {
  let xml = clean!("index-bib/bibliography_in_quote.tex");
  assert_all(&xml, &[
    r#"<para xml:id="p1"> <p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p> </para> <bibliography inlist="toc" xml:id="bib"> <title fontsize="90%">References</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock><text fontsize="90%" xml:id="bib.bib1.1"> A. Author, Journal 1 (2000) 1. </text></bibblock> </bibitem> </biblist> </bibliography> </document>"#,
  ]);
  assert!(!xml.contains("<quote"), "{xml}");
}

/// A `{verse}` with text before and after its bibliography (1504.06728): the text before stays in the verse, the text
/// after follows the bibliography.
#[test]
fn bibliography_in_a_verse_keeps_the_text_around_it() {
  let xml = clean!("index-bib/bibliography_in_verse_after_text.tex");
  assert_all(&xml, &[
    r#"<para xml:id="p1"> <quote role="verse" xml:id="p1.1"> <p xml:id="p1.1.1">Lines.</p> </quote> </para> <bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author, J. 1. </bibblock> </bibitem> </biblist> </bibliography> <para xml:id="p2"> <p xml:id="p2.1">More.</p> </para> </document>"#,
  ]);
}

/// `\begin{quotation}\section*{}` (0712.1985) and a numbered, labelled `\subsection` in a `{quote}`: each quote holds an
/// inline sectional block with the unit; the subsection keeps its number and label; the text after the quote stays in
/// the section's paragraph.
#[test]
fn section_in_a_quotation_opens_a_block() {
  let xml = clean!("sectioning-frontmatter/section_in_quotation.tex");
  assert_all(&xml, &[
    r#"<para xml:id="p1"> <quote xml:id="p1.1"> <inline-sectional-block xml:id="p1.1.1"> <section xml:id="Sx1"> <title/> <para xml:id="Sx1.p1"> <p align="center" xml:id="Sx1.p1.1">Abstract</p> </para> <para xml:id="Sx1.p2"> <p xml:id="Sx1.p2.1"><text fontsize="90%" xml:id="Sx1.p2.1.1">We study things. </text></p> </para> </section> </inline-sectional-block> </quote> </para>"#,
    r#"<quote xml:id="S1.p1.2"> <inline-sectional-block xml:id="S1.p1.2.1"> <subsection inlist="toc" labels="LABEL:q" xml:id="S1.SS1"> <tags> <tag>1.1</tag> <tag role="refnum">1.1</tag> <tag role="typerefnum">§1.1</tag> </tags> <title><tag close=" ">1.1</tag>Inner</title> <para xml:id="S1.SS1.p1"> <p xml:id="S1.SS1.p1.1">Quoted text.</p> </para> </subsection> </inline-sectional-block> </quote> <p xml:id="S1.p1.3">After.</p> </para> </section>"#,
  ]);
}

/// Kluwer's `{chapthebibliography}` (`\section*{References}` + `\list` + `\item[]`; hep-th0212109, cond-mat0112379,
/// cond-mat0110649, cond-mat0002157): one bibliography titled References with both entries, no empty section or list.
#[test]
fn kluwer_chapthebibliography_closes_its_list() {
  let xml = clean!("index-bib/kluwer_chapthebibliography.tex");
  assert_all(&xml, &[
    r#"</section> <bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author, Journal 1 (2000) 1. </bibblock> </bibitem> <bibitem key="b" xml:id="bib.bib2"> <tags> <tag>[2]</tag> <tag role="refnum">2</tag> </tags> <bibblock> B. Author, Journal 2 (2001) 2. </bibblock> </bibitem> </biblist> </bibliography> </document>"#,
  ]);
  assert_eq!(xml.matches("<section ").count(), 1, "{xml}");
  assert!(!xml.contains("<itemize"), "{xml}");
}

/// `\multirow{2}{*}{\ib}` with a one-argument `\ib` (2507.07731): the macro takes multirow's closing `\strut`
/// (multirow.sty:174-177), not the box's brace; in an `array` the delimiter leaves no rule in the formula's `tex=`
/// (2501.19081).
#[test]
fn multirow_text_is_strut_delimited() {
  let xml = clean!("package-bindings/multirow_text_one_argument_macro.tex");
  assert_all(&xml, &[
    r#"<tr xml:id="p1.1.1"> <td align="left" border="b" rowspan="2" xml:id="p1.1.1.1"><text xml:id="p1.1.1.1.1">InstructBLIP</text></td> <td align="left" xml:id="p1.1.1.2">a</td> </tr> <tr xml:id="p1.1.2"> <td align="left" border="b" xml:id="p1.1.2.1">b</td> </tr> </tbody> </tabular>"#,
    r#"tex="A=\begin{array}[]{cc}\hbox{\multirowsetup$\mathbf{0}$}&amp;x\\&#10;&amp;y\end{array}""#,
  ]);
  assert!(!xml.contains("vrule"), "{xml}");
}

/// `\arrayrulecolor\hhline{---~~~}` (1910.02206): the argument runs to the first brace and takes that group
/// (colortbl.sty:156-164), so no row of the spec's text appears between the two rows; `\doublerulesepcolor[rgb]{…}` too.
#[test]
fn arrayrulecolor_reads_to_the_brace() {
  let xml = clean!("package-bindings/colortbl_arrayrulecolor_unbraced.tex");
  assert_eq!(xml.matches("<tr ").count(), 2, "{xml}");
  assert_all(&xml, &[
    r#"<td align="center" xml:id="p1.1.1.6">f</td> </tr> <tr xml:id="p1.1.2"> <td align="center" border="b t" xml:id="p1.1.2.1">g</td>"#,
  ]);
  assert!(!xml.contains("---"), "{xml}");
}

/// `\text{ kg m$^{-3}$}` in a dcolumn `D` cell (1302.5691; fixed by 68w's `lx_dc_inner_math`): the unit's superscript
/// is a formula inside the text.
#[test]
fn dcolumn_text_box_dollar_is_a_math_shift() {
  let xml = clean!("package-bindings/dcolumn_text_box_dollar.tex");
  assert_all(&xml, &[
    r#"<XMText xml:id="p1.m2.1.1.3"> kg m<sup xml:id="p1.m2.1.1.3.1"><text xml:id="p1.m2.1.1.3.1.1">-3</text></sup></XMText>"#,
  ]);
}

/// `\usepackage[amsstyle]{cases}` (2508.08751): the explanation column is a formula and the right-hand side
/// `\textstyle` (cases.sty:228-233).
#[test]
fn cases_amsstyle_explanation_is_math() {
  let xml = clean!("package-bindings/cases_amsstyle_explanation_math.tex");
  assert_all(&xml, &[
    r#"<td align="left" xml:id="S0.E1.2"><Math mode="inline" tex="\textstyle\eta=x" text="eta = x" xml:id="S0.E1.m1">"#,
    r#"<td align="left" xml:id="S0.E1.3"><Math mode="inline" tex="\text{in }[0,T]\times\Omega^{\epsilon}," text="[in ] * closed-interval@(0, T) * Omega ^ epsilon" xml:id="S0.E1.m2">"#,
    r#"<td align="left" xml:id="S0.E2.3"><Math mode="inline" tex="\text{on }\Gamma^{\epsilon}" text="[on ] * Gamma ^ epsilon" xml:id="S0.E2.m2">"#,
  ]);
}

/// A braced `cases` in an `\inferrule` premise (2604.09165): only the premises' own `\\` separates them
/// (mathpartir.sty:133-146), so the cases keep both rows. The stub's one missing_file Warning stays.
#[test]
fn mathpartir_splits_only_its_own_premise_breaks() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/mathpartir_premise_braced_rows.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  let xml = squash(&xml);
  assert_all(&xml, &[
    r#"tex="\frac{o={\begin{cases}a&amp;\text{if }b\\&#10;c&amp;\text{otherwise}\end{cases}}\quad d}{e}\quad\textsc{Step}""#,
    r#"<XMRow xml:id="S0.Ex1.m1.2.3.2.2.2"> <XMCell align="left" xml:id="S0.Ex1.m1.2.3.2.2.2.1"> <XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.5">c</XMTok> </XMCell> <XMCell align="left" xml:id="S0.Ex1.m1.2.3.2.2.2.2"> <XMText xml:id="S0.Ex1.m1.6">otherwise</XMText> </XMCell> </XMRow>"#,
  ]);
}

/// `A & \multicolumn{3}{|c|}{B}` in a three-column `{NiceTabular}` (2505.05139): the span reaches nicematrix's guard
/// column (nicematrix.sty:2236-2243) silently; a row with a cell too many runs the guard's template, one Error.
#[test]
fn nicematrix_guard_column_takes_a_span() {
  let xml = clean!("package-bindings/nicematrix_guard_column.tex");
  assert_all(&xml, &[
    r#"<td align="center" border="l r" colspan="2" thead="column" vattach="top" xml:id="p1.1.1.2">B</td> </tr> </thead>"#,
  ]);
  let (log, xml) = convert_with(
    r"\documentclass{article}\usepackage{nicematrix}\begin{document}
\begin{NiceTabular}{ccc} a & b & c & d \\ e & f & g \\ \end{NiceTabular}\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Too much columns"), "{log}");
  assert!(xml.contains(">d</td>"), "{xml}");
}

/// A never-used `\newacronym` whose long form holds `&` (2602.09131): the stored definition reads the character.
#[test]
fn glossary_entry_ampersand_is_stored_text() {
  let xml = clean!("package-bindings/glossaries_untypeset_ampersand.tex");
  assert_all(&xml, &[
    r#"<glossaryphrase key="cisa" role="long">Cybersecurity &amp; Infrastructure Security Agency</glossaryphrase>"#,
    r#"<glossaryphrase key="cisa" role="description">Cybersecurity &amp; Infrastructure Security Agency</glossaryphrase>"#,
  ]);
}

/// `\DeclareMathOperator{\N^1}{N^1}` after `\newcommand{\N}` (2301.04262): `\N^1` is the operator (the `^1` its
/// parameter text, amsopn.sty:50-53).
#[test]
fn declaremathoperator_parameter_text() {
  let xml = clean!("package-bindings/declaremathoperator_parameter_text.tex");
  assert_all(&xml, &[
    r#"tex="\operatorname{N^{1}}(x)+\operatorname*{Lim}_{n}a_{n}""#,
    r#"<XMApp name="N" role="OPFUNCTION" scriptpos="post" xml:id="p1.m1.1"> <XMTok role="SUPERSCRIPTOP" scriptpos="post3"/> <XMTok role="UNKNOWN">N</XMTok> <XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok> </XMApp>"#,
  ]);
}
