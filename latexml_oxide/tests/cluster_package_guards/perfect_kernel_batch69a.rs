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
