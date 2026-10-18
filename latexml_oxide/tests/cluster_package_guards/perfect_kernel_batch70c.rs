//! Red/green guards for batch 70c: a `\bibitem` label's reference forms are TeX's `\b@key`, typeset only where a
//! `\cite` prints them (OXIDIZED_DESIGN_DIVERGENCES #750; the user ruling of 2026-10-18 on text TeX writes to an
//! auxiliary file). Under mn2e/mnras without natbib (`\def\@biblabel#1{}`) a `\citeauthoryear` label errs only where a
//! `\cite` prints it (projection 67int23b: 0812.1037, 1105.2318, 1306.6339, 1805.00040).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The conversion of a repro with `errors` errors, no warning and no Fatal; returns the XML.
fn converted(tex: &str, errors: usize) -> String {
  let (log, xml) = convert_counted(tex);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (errors, 0),
    "{log}"
  );
  assert!(!log.lines().any(|l| l.starts_with("Fatal:")), "{log}");
  xml
}

const UNCITED_ENTRY: &str =
  "<bibitem key=\"a\" xml:id=\"bib.bibx1\"><bibblock> Abbott B., 2017, ApJ\n</bibblock></bibitem>";

const CITED_ENTRY: &str = "<bibitem key=\"a\" xml:id=\"bib.bibx1\"><tags><tag role=\"autoref\"><ERROR class=\"undefined\" xml:id=\"bib.bibx1.1\">\\citeauthoryear</ERROR>Abbott et al.2017<text xml:id=\"bib.bibx1.2\"/></tag><tag role=\"refnum\"><ERROR class=\"undefined\" xml:id=\"bib.bibx1.3\">\\citeauthoryear</ERROR>Abbott et al.2017</tag></tags><bibblock> Abbott B., 2017, ApJ\n</bibblock></bibitem>";

/// mnras.cls:1636 prints no label and the paper never cites the key: pdflatex 0 errors, and no tag.
#[test]
fn mnras_bibitem_label_uncited_is_never_typeset() {
  let xml = converted(
    include_str!("../../../tools/perfect_kernel/repros/index-bib/mnras_bibitem_label_uncited.tex"),
    0,
  );
  assert_element(&xml, "bibitem", &[r#"key="a""#], UNCITED_ENTRY);
}

/// `\nocite` writes `\citation` and prints nothing: pdflatex 0 errors.
#[test]
fn mnras_bibitem_label_nocite_prints_nothing() {
  let xml = converted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/mnras_bibitem_label_nocite_only.tex"
    ),
    0,
  );
  assert_element(&xml, "bibitem", &[r#"key="a""#], UNCITED_ENTRY);
}

/// A `\cite` before the list prints the key's label: pdflatex's one error kept, and the citation's text (the refnum and
/// autoref forms); the uncited entry b has no tag.
#[test]
fn mnras_bibitem_label_cited_before_keeps_its_error_and_text() {
  let xml = converted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/mnras_bibitem_label_cited_before.tex"
    ),
    1,
  );
  assert_element(&xml, "bibitem", &[r#"key="a""#], CITED_ENTRY);
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="b""#],
    "<bibitem key=\"b\" xml:id=\"bib.bibx2\"><bibblock> Baker C., 2001, MNRAS\n</bibblock></bibitem>",
  );
}

/// A `\cite` after the list (an appendix) prints the label there: one error, the entry's reference forms filled in.
#[test]
fn mnras_bibitem_label_cited_after_the_list() {
  let xml = converted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/mnras_bibitem_label_cited_after.tex"
    ),
    1,
  );
  assert_element(&xml, "bibitem", &[r#"key="a""#], CITED_ENTRY);
}

/// Control: an article's `[N]` labels are printed in the list, cited or not; only the cited entry has the refnum.
#[test]
fn numeric_bibitem_labels_printed_refnum_only_cited() {
  let xml = converted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/bibitem_numeric_label_cited_and_uncited.tex"
    ),
    0,
  );
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="a""#],
    "<bibitem key=\"a\" xml:id=\"bib.bibx1\"><tags><tag>[1]</tag><tag role=\"refnum\">1</tag></tags><bibblock> Abbott B., 2017, ApJ\n</bibblock></bibitem>",
  );
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="b""#],
    "<bibitem key=\"b\" xml:id=\"bib.bibx2\"><tags><tag>[2]</tag></tags><bibblock> Baker C., 2001, MNRAS\n</bibblock></bibitem>",
  );
}

/// Control: with `usenatbib` natbib parses the label itself, every entry with its author-year tags.
#[test]
fn mnras_usenatbib_bibitem_label_is_natbibs() {
  let xml = converted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/mnras_usenatbib_bibitem_label_cited.tex"
    ),
    0,
  );
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="b""#],
    "<bibitem key=\"b\" xml:id=\"bib.bib2\"><tags><tag role=\"number\">2</tag><tag role=\"year\">2001</tag><tag role=\"authors\">Baker</tag><tag role=\"refnum\">Baker (2001)</tag><tag role=\"key\">b</tag></tags><bibblock> Baker C., 2001, MNRAS\n</bibblock></bibitem>",
  );
}
