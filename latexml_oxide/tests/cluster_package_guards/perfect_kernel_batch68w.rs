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

/// lipics-v2021's five-part `\author{name}{affiliations}{email}{orcid}{funding}` (lipics-v2021.cls:766-800; 2607.26691,
/// 2603.11908, 2307.12098, 2608.03845) through the frontmatter API, and `\relatedversiondetails`/`\supplementdetails`
/// with their keys and their URL a URL (:919-935, :962-985; 2112.14457): no author text left in the body.
#[test]
fn lipics_author_five_parts_reach_the_frontmatter() {
  let xml = clean!("class-bindings/lipics_author_five_parts.tex");
  assert_all(&xml, &[
    r#"<creator role="author"> <personname>Ann Author</personname> <contact name="Affiliation: " role="affiliation">Uni X, Austria</contact> <contact name="Affiliation: " role="affiliation">Inst Y, Germany</contact> <contact name="URL: " role="url">https://ann.example.org/~ann_a</contact> <contact name="Email: " role="email">ann_author@x.at</contact> <contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0001-0000-0000" title="ORCID 0000-0001-0000-0000">"#,
    r#"<creator before=" " role="author"> <personname>Bob Builder</personname> <contact name="Affiliation: " role="affiliation">Uni Z</contact> </creator>"#,
    r#"<note role="funding" xml:id="id1"><text font="italic" xml:id="id1.1">Ann Author</text>: ERC grant 1</note>"#,
    r#"<note role="related-version" xml:id="id2"><text font="italic" xml:id="id2.1">Full Version</text>: <ref class="ltx_href" font="typewriter" href="https://arxiv.org/abs/2101.00001_v2">arXiv</ref></note>"#,
    r#"<note role="supplement" xml:id="id3"><text font="italic" xml:id="id3.1">Software</text>: <ref class="ltx_href" font="typewriter" href="https://github.com/a/sui_analysis_code">Scripts</ref></note>"#,
    r#"<ref class="ltx_url" font="typewriter" href="https://github.com/b/c_d">https://github.com/b/c_d</ref><break/> archived at <ref class="ltx_href" href="https://archive.softwareheritage.org/swh:1:dir:abc;origin=https://x.org">"#,
    r#"<para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para> </document>"#,
  ]);
  assert_eq!(count(&xml, r#"role="email""#), 1, "{xml}");
}

/// Under lipics' `anonymous` option (lipics-v2021.cls:192, :791-794, :809) the author is "Anonymous author" with the
/// affiliation "Anonymous affiliation", and nothing else of it.
#[test]
fn lipics_anonymous_author_hides_the_author() {
  let xml = clean!("class-bindings/lipics_anonymous.tex");
  assert_all(&xml, &[
    r#"<creator role="author"> <personname>Anonymous author</personname> <contact name="Affiliation: " role="affiliation">Anonymous affiliation</contact> </creator> <para xml:id="p1">"#,
  ]);
  assert!(!xml.contains("Ann") && !xml.contains("Grant"), "{xml}");
}

/// thmtools' keys in a theorem's optional argument (thm-kv.sty:434-437, `label` :485-487, `name` :488-496, `continues`
/// :511-516; 2002.09876, 2409.14881): the label on the theorem, no key text in a title, a note that is no key list kept,
/// the continued example numbered by a reference to the one it continues and its counter not stepped.
#[test]
fn thmtools_keyval_optional_argument_sets_label_name_continues() {
  let xml = clean!("package-bindings/thmtools_keyval_optional_argument.tex");
  assert!(
    !xml.contains("label=") && !xml.contains("continues=") && !xml.contains("name="),
    "{xml}"
  );
  assert_all(&xml, &[
    r#"<theorem class="ltx_theorem_example" inlist="thm theorem:example" labels="LABEL:ex:a_1" xml:id="Thmexample1"> <tags> <tag>Example 1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">Example 1</tag> </tags> <title class="ltx_runin"><tag><text font="bold" xml:id="Thmexample1.1">Example 1</text></tag><text font="bold" xml:id="Thmexample1.2">.</text></title>"#,
    r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" labels="LABEL:thm:main_1" xml:id="Thmtheorem1"> <tags> <tag>Theorem 1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">Theorem 1</tag> </tags> <title class="ltx_runin"><tag><text font="bold" xml:id="Thmtheorem1.1">Theorem 1</text></tag><text font="bold" xml:id="Thmtheorem1.2"> </text>(Main result)<text font="bold" xml:id="Thmtheorem1.3">.</text></title>"#,
    r#"<text font="bold" xml:id="Thmtheorem2.2"> </text>(Smith, 2001)<text font="bold" xml:id="Thmtheorem2.3">.</text></title>"#,
    r#"<theorem class="ltx_theorem_example" inlist="thm theorem:example" xml:id="Thmexamplex1"> <tags> <tag>Example <ref labelref="LABEL:ex:a_1"/></tag> <tag role="refnum"><ref labelref="LABEL:ex:a_1"/></tag> <tag role="typerefnum">Example <ref labelref="LABEL:ex:a_1"/></tag> </tags> <title class="ltx_runin"><tag><text font="bold" xml:id="Thmexamplex1.1">Example </text><ref font="bold" labelref="LABEL:ex:a_1"/></tag><text font="bold" xml:id="Thmexamplex1.2"> </text>(Running example, continuing from p. <ref labelref="LABEL:ex:a_1"/>)<text font="bold" xml:id="Thmexamplex1.3">.</text></title>"#,
    r#"<text font="bold" xml:id="Thmexamplex2.2"> </text>(continuing from p. <ref labelref="LABEL:ex:a_1"/>)<text font="bold" xml:id="Thmexamplex2.3">.</text></title>"#,
    r#"<tag role="refnum">3</tag> <tag role="typerefnum">Example 3</tag>"#,
  ]);
}
