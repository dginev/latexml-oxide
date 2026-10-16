//! Red/green guards for batch 68g: llncs's theorem set numbers as llncs.cls numbers it under each numbering option
//! (default, envcountsame, envcountsect, both, envcountreset; llncs.cls:1142-1174, TL 2025 v2.26, alike in every
//! shipped copy v2.2-v2.26). The expected numbers are pdflatex's on each repro (KNOWN_PERL_ERRORS #743).
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion of a class-bindings repro, squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/class-bindings/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// Every theorem's visible tag and refnum, in document order: `"Theorem 1/1"`, …
fn theorem_tags(xml: &str) -> Vec<String> {
  let mut tags = Vec::new();
  for chunk in xml.split("<theorem ").skip(1) {
    let chunk = chunk.split_once("</tags>").map_or(chunk, |(tags, _)| tags);
    let tag = |open: &str| {
      chunk
        .split_once(open)
        .and_then(|(_, rest)| rest.split_once("</tag>"))
        .map(|(t, _)| t.to_string())
    };
    let shown = tag("<tags> <tag>").unwrap_or_default();
    let refnum = tag(r#"<tag role="refnum">"#).unwrap_or_default();
    tags.push(format!("{shown}/{refnum}"));
  }
  tags
}

/// The theorems' tags, `"Theorem 1/1, Lemma 1/1, …"`, are `expected`.
fn expect_tags(xml: &str, expected: &str) {
  assert_eq!(theorem_tags(xml).join(", "), expected, "{xml}");
}

/// The default: `{theorem}` and every other env on its own counter, none within the section ("Theorem 3" in
/// section 2), the unnumbered `{claim}` and the paper's own `\spnewtheorem` alike.
#[test]
fn llncs_numbering_default() {
  let xml = clean!("llncs_theorem_numbering_default.tex");
  expect_tags(
    &xml,
    "Theorem 1/1, Lemma 1/1, Theorem 2/2, Lemma 2/2, Definition 1/1, Fact 1/1, MyY 1/1, Claim/, Theorem 3/3, Corollary 1/1, Remark 1/1, Fact 2/2",
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" labels="LABEL:t2" xml:id="Thmtheorem3"> <tags> <tag>Theorem 3</tag> <tag role="refnum">3</tag> <tag role="typerefnum">Theorem 3</tag> </tags>"#),
    "{xml}"
  );
  // The class's `{definition}` has an italic body (llncs.cls:1160 `{\bfseries}{\itshape}`; Perl's binding upright).
  assert!(
    xml.contains(r#"<title class="ltx_runin"><tag><text font="bold" xml:id="Thmdefinition1.1">Definition 1</text></tag></title> <para xml:id="Thmdefinition1.p1"> <p xml:id="Thmdefinition1.p1.1"><text font="italic" xml:id="Thmdefinition1.p1.1.1">e</text></p> </para> </theorem>"#),
    "{xml}"
  );
}

/// envcountsame: every env of the set, and a paper's `\spn@wtheorem{fact}`, numbers with the theorem counter.
#[test]
fn llncs_numbering_envcountsame() {
  let xml = clean!("llncs_theorem_numbering_envcountsame.tex");
  expect_tags(
    &xml,
    "Theorem 1/1, Lemma 2/2, Theorem 3/3, Lemma 4/4, Definition 5/5, Fact 6/6, MyY 1/1, Claim/, Theorem 7/7, Corollary 8/8, Remark 9/9, Fact 10/10",
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="Thmtheorem10"> <tags> <tag>Fact 10</tag> <tag role="refnum">10</tag> <tag role="typerefnum">Fact 10</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountsect: each env on its own counter within the section ("Lemma 0.1" before the first section; 0906.4121's
/// "Fact 3.1").
#[test]
fn llncs_numbering_envcountsect() {
  let xml = clean!("llncs_theorem_numbering_envcountsect.tex");
  expect_tags(
    &xml,
    "Theorem 0.1/0.1, Lemma 0.1/0.1, Theorem 1.1/1.1, Lemma 1.1/1.1, Definition 1.1/1.1, Fact 1.1/1.1, MyY 1/1, Claim/, Theorem 2.1/2.1, Corollary 2.1/2.1, Remark 2.1/2.1, Fact 2.1/2.1",
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="S2.Thmfact1"> <tags> <tag>Fact 2.1</tag> <tag role="refnum">2.1</tag> <tag role="typerefnum">Fact 2.1</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountsame + envcountsect: one theorem counter, within the section.
#[test]
fn llncs_numbering_envcountsame_envcountsect() {
  let xml = clean!("llncs_theorem_numbering_envcountsame_envcountsect.tex");
  expect_tags(
    &xml,
    "Theorem 0.1/0.1, Lemma 0.2/0.2, Theorem 1.1/1.1, Lemma 1.2/1.2, Definition 1.3/1.3, Fact 1.4/1.4, MyY 1/1, Claim/, Theorem 2.1/2.1, Corollary 2.2/2.2, Remark 2.3/2.3, Fact 2.4/2.4",
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="S2.Thmtheorem4"> <tags> <tag>Fact 2.4</tag> <tag role="refnum">2.4</tag> <tag role="typerefnum">Fact 2.4</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountreset: each env on its own counter, reset by the section but printed without it.
#[test]
fn llncs_numbering_envcountreset() {
  let xml = clean!("llncs_theorem_numbering_envcountreset.tex");
  expect_tags(
    &xml,
    "Theorem 1/1, Lemma 1/1, Theorem 1/1, Lemma 1/1, Definition 1/1, Fact 1/1, MyY 1/1, Claim/, Theorem 1/1, Corollary 1/1, Remark 1/1, Fact 1/1",
  );
}
