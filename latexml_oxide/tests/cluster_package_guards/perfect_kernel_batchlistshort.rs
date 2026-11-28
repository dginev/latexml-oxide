//! Guards for LISTSHORT: latex.ltx's list macros with arguments are not `\long` — `\list#1#2` (:15848),
//! `\usecounter#1` (:16048), `\@mklab#1` (:15963) — so a `\par` read as an argument ends the call, "Paragraph ended
//! before \list was complete" (tex.web §392, §396), the `\par` put back and the call dropped. A stray `\list` and a
//! blank line after `\begin{thebibliography}` (1310.5566, revtex4) opened a list that nested every entry: 38
//! "<ltx:bibitem> isn't allowed in <ltx:bibblock>". Perl nests the same (KNOWN_PERL_ERRORS #1348); OXIDIZED_DESIGN_DIVERGENCES #1173.
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// A whole element: its tag, identifying attributes, the element.
type Element = (&'static str, &'static [&'static str], &'static str);
/// One repro: its name, its source, the exact Error and Warning counts, each "Paragraph ended before \X was complete"
/// command with its count, the `<itemize>` count, and whole elements.
type Row = (
  &'static str,
  &'static str,
  usize,
  usize,
  &'static [(&'static str, usize)],
  usize,
  &'static [Element],
);

const ROWS: &[Row] = &[
  // The witness's shape (1310.5566): the stray `\list` opens no list, both entries in the bibliography's own. Red
  // (INT081TA): 1 Error:malformed "<ltx:bibitem> isn't allowed in <ltx:p>", the entries in an `<ltx:itemize>`.
  (
    "list_argument_ended_by_a_paragraph_in_a_bibliography",
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/list_argument_ended_by_a_paragraph_in_a_bibliography.tex"
    ),
    1,
    0,
    &[(r"\list", 1)],
    0,
    &[(
      "biblist",
      &[],
      concat!(
        r#"<biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags>"#,
        r#"<bibblock> First entry.</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag>"#,
        r#"<tag role="refnum">2</tag></tags><bibblock> Second entry.</bibblock></bibitem></biblist>"#,
      ),
    )],
  ),
  // The siblings: `\list`'s second argument, a `\list` in an enumerate item, `\usecounter`, `\@mklab`; a blank line
  // after a list's arguments is the control. Red (INT081TA): 1 Error:malformed, 2 Warnings, B..I in two stray lists.
  (
    "list_family_arguments_ended_by_a_paragraph",
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/list_family_arguments_ended_by_a_paragraph.tex"
    ),
    4,
    0,
    &[(r"\list", 2), (r"\usecounter", 1), (r"\@mklab", 1)],
    1,
    &[(
      "document",
      &[],
      concat!(
        r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/>"#,
        r#"<resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><p>A</p></para>"#,
        r#"<para xml:id="p2"><p>B</p></para><para xml:id="p3"><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1">"#,
        r#"<tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags>"#,
        r#"<para xml:id="S0.I1.i1.p1"><p>C</p></para><para xml:id="S0.I1.i1.p2"><p>D</p></para></item></enumerate>"#,
        r#"</para><para xml:id="p4"><p>E</p></para><para xml:id="p5"><p>F</p></para><para xml:id="p6"><p>G</p></para>"#,
        r#"<para xml:id="p7"><itemize><item xml:id="S0.I2.i1"><tags><tag>-</tag><tag role="typerefnum">1st item</tag>"#,
        r#"</tags><para xml:id="S0.I2.i1.p1"><p>H</p></para><para xml:id="S0.I2.i1.p2"><p>I</p></para></item>"#,
        r#"</itemize></para></document>"#,
      ),
    )],
  ),
];

/// Repros list-structure/list_argument_ended_by_a_paragraph_in_a_bibliography (witness 1310.5566) and
/// list-structure/list_family_arguments_ended_by_a_paragraph: each "Paragraph ended" Error by its command, the exact
/// counts, no stray list, whole elements.
#[test]
fn list_macro_arguments_end_at_a_paragraph() {
  for (name, tex, errors, warnings, ended, itemizes, elements) in ROWS {
    let (log, xml) = convert(tex, false);
    for (cs, n) in *ended {
      let message = format!("Paragraph ended before {cs} was complete");
      assert_eq!(
        log.matches(&message).count(),
        *n,
        "{name}: {message}\n{log}"
      );
    }
    assert_eq!(error_count(&log), *errors, "{name}: {log}");
    assert_eq!(warning_count(&log), *warnings, "{name}: {log}");
    assert_eq!(xml.matches("<itemize").count(), *itemizes, "{name}:\n{xml}");
    for (tag, attrs, element) in *elements {
      assert_element(&xml, tag, attrs, element);
    }
  }
}
