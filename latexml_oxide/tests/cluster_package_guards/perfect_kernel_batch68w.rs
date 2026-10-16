//! Red/green guards for batch 68w: the first-error clusters `malformed:ltx:bibitem` (a heading inside the author's
//! `{thebibliography}`) and `unexpected:_` (lipics `\author`, thmtools key=value theorem options, and one-liners).
//! Each repro's pdflatex reference text is in its header.
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

fn count(xml: &str, part: &str) -> usize { xml.matches(part).count() }

/// `\begin{thebibliography}{99}\section*{References}\bibitem…` (1702.06834, 2402.15720, 2110.15342): one bibliography,
/// titled as the heading, holding both entries, no empty one before it.
#[test]
fn heading_inside_thebibliography_makes_one_bibliography() {
  let xml = clean!("index-bib/heading_inside_thebibliography.tex");
  assert_eq!(count(&xml, "<bibliography "), 1, "{xml}");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="biba"> <title>References</title> <biblist> <bibitem key="a" xml:id="biba.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author, J. 1 (2000). </bibblock> </bibitem> <bibitem key="b" xml:id="biba.bib2"> <tags> <tag>[2]</tag> <tag role="refnum">2</tag> </tags> <bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> <para xml:id="p2"> <p xml:id="p2.1">After.</p> </para>"#,
  ]);
  assert!(!xml.contains("<section"), "{xml}");
}

/// Topical headings between groups of entries (hep-th0109182): the environment's own References heading stays over
/// them, each group is a bibliography of its own, and the entries are numbered on through the one list.
#[test]
fn heading_inside_thebibliography_topical_groups_number_on() {
  let xml = clean!("index-bib/heading_inside_thebibliography_topical.tex");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist/> </bibliography> <bibliography inlist="toc" xml:id="biba"> <title>General Background</title> <biblist> <bibitem key="a" xml:id="biba.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author, J. 1 (2000). </bibblock> </bibitem> </biblist> </bibliography> <bibliography inlist="toc" xml:id="bibb"> <title>Special Topics</title> <biblist> <bibitem key="b" xml:id="bibb.bib2"> <tags> <tag>[2]</tag> <tag role="refnum">2</tag> </tags> <bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> <bibliography inlist="toc" xml:id="bibc"> <title>More</title> <biblist> <bibitem key="c" xml:id="bibc.bib3"> <tags> <tag>[3]</tag> <tag role="refnum">3</tag> </tags> <bibblock> C. Author, J. 3 (2002). </bibblock> </bibitem> </biblist> </bibliography> <para xml:id="p2"> <p xml:id="p2.1">After.</p> </para>"#,
  ]);
}

/// revtex4-1 (natbib's `\bibitem`, 2008.08910): no "Lonely \item" for the entries after the heading.
#[test]
fn heading_inside_thebibliography_revtex_natbib_entries_stay_in_the_list() {
  let xml = clean!("index-bib/heading_inside_thebibliography_revtex.tex");
  assert_eq!(count(&xml, "<bibliography "), 1, "{xml}");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="biba"> <title>References</title> <biblist> <bibitem key="a" xml:id="biba.bib1"> <tags> <tag role="number">1</tag> <tag role="refnum">[1]</tag> <tag role="key">a</tag> </tags> <bibblock> A. Author, J. 1 (2000). </bibblock> </bibitem> <bibitem key="b" xml:id="biba.bib2"> <tags> <tag role="number">2</tag> <tag role="refnum">[2]</tag> <tag role="key">b</tag> </tags> <bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> </document>"#,
  ]);
}

/// A beamer frame inside `{thebibliography}` (1301.0681): the frame holds the bibliography, no empty one before it.
#[test]
fn heading_inside_thebibliography_beamer_frame_holds_it() {
  let xml = clean!("index-bib/heading_inside_thebibliography_frame.tex");
  assert_eq!(count(&xml, "<bibliography "), 1, "{xml}");
  assert_all(&xml, &[
    r#"<subsection xml:id="id2"> <bibliography inlist="toc" xml:id="biba"> <title>References</title> <biblist> <bibitem key="a" xml:id="biba.bib1">"#,
    r#"<bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> </subsection> </document>"#,
  ]);
}

/// A heading inside `{thebibliography}` that no `\bibitem` follows leaves nothing pending past the environment's end: a
/// later lonely `\bibitem` is reported (pdflatex: "Lonely \item") and the next bibliography numbers from 1.
#[test]
fn heading_inside_thebibliography_no_entry_after_leaves_nothing_pending() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/heading_inside_thebibliography_no_entry_after.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Lonely \\item"), "{log}");
  let xml = squash(&xml);
  assert_all(&xml, &[
    r#"<bibitem key="z" xml:id="biba.bib1"> <tags> <tag role="number">1</tag> <tag role="refnum">[1]</tag>"#,
    r#"<bibitem key="b" xml:id="bibb.bib1"> <tags> <tag role="number">1</tag> <tag role="refnum">[1]</tag>"#,
  ]);
}
