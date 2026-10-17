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
/// titled as the heading, holding both entries, no empty one before it, with the ids of the one bibliography (`bib`,
/// `bib.bibN`: the continuation took the pruned one's number and id).
#[test]
fn heading_inside_thebibliography_makes_one_bibliography() {
  let xml = clean!("index-bib/heading_inside_thebibliography.tex");
  assert_eq!(count(&xml, "<bibliography "), 1, "{xml}");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author, J. 1 (2000). </bibblock> </bibitem> <bibitem key="b" xml:id="bib.bib2"> <tags> <tag>[2]</tag> <tag role="refnum">2</tag> </tags> <bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> <para xml:id="p2"> <p xml:id="p2.1">After.</p> </para>"#,
  ]);
  assert!(!xml.contains("<section"), "{xml}");
}

/// Topical headings between groups of entries (hep-th0109182): the environment's own References heading stays over
/// them, each group is a bibliography of its own, and the entries are numbered on through the one list. The first group
/// took the kept heading's number (`bib.bib1`), its own id the duplicate's `biba` (Document.pm recordID), so the next
/// group's `biba` is `bibaa` (OXIDIZED_DESIGN_DIVERGENCES #653).
#[test]
fn heading_inside_thebibliography_topical_groups_number_on() {
  let xml = clean!("index-bib/heading_inside_thebibliography_topical.tex");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist/> </bibliography> <bibliography inlist="toc" xml:id="biba"> <title>General Background</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author, J. 1 (2000). </bibblock> </bibitem> </biblist> </bibliography> <bibliography inlist="toc" xml:id="bibaa"> <title>Special Topics</title> <biblist> <bibitem key="b" xml:id="biba.bib2"> <tags> <tag>[2]</tag> <tag role="refnum">2</tag> </tags> <bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> <bibliography inlist="toc" xml:id="bibb"> <title>More</title> <biblist> <bibitem key="c" xml:id="bibb.bib3"> <tags> <tag>[3]</tag> <tag role="refnum">3</tag> </tags> <bibblock> C. Author, J. 3 (2002). </bibblock> </bibitem> </biblist> </bibliography> <para xml:id="p2"> <p xml:id="p2.1">After.</p> </para>"#,
  ]);
}

/// revtex4-1 (natbib's `\bibitem`, 2008.08910): no "Lonely \item" for the entries after the heading.
#[test]
fn heading_inside_thebibliography_revtex_natbib_entries_stay_in_the_list() {
  let xml = clean!("index-bib/heading_inside_thebibliography_revtex.tex");
  assert_eq!(count(&xml, "<bibliography "), 1, "{xml}");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag role="number">1</tag> <tag role="refnum">[1]</tag> <tag role="key">a</tag> </tags> <bibblock> A. Author, J. 1 (2000). </bibblock> </bibitem> <bibitem key="b" xml:id="bib.bib2"> <tags> <tag role="number">2</tag> <tag role="refnum">[2]</tag> <tag role="key">b</tag> </tags> <bibblock> B. Author, J. 2 (2001). </bibblock> </bibitem> </biblist> </bibliography> </document>"#,
  ]);
}

/// A beamer frame inside `{thebibliography}` (1301.0681): the frame holds the bibliography, no empty one before it.
#[test]
fn heading_inside_thebibliography_beamer_frame_holds_it() {
  let xml = clean!("index-bib/heading_inside_thebibliography_frame.tex");
  assert_eq!(count(&xml, "<bibliography "), 1, "{xml}");
  assert_all(&xml, &[
    r#"<subsection xml:id="id2"> <bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1">"#,
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

/// An entry after the heading keeps its `\\` and its paragraphs (hep-th0109182 [17]): the continuation undoes the entry
/// redirection its setup armed as `\restoring@bibitem` does, so there is one entry, as without the heading.
#[test]
fn heading_inside_thebibliography_entry_keeps_its_paragraphs() {
  let xml = clean!("index-bib/heading_inside_thebibliography_entry_paragraphs.tex");
  assert_eq!(count(&xml, "<bibitem "), 2, "{xml}");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Author,<break/>J. 1 (2000). Erratum: J. 1, 5 (2001). </bibblock> </bibitem> <bibitem key="b" xml:id="bib.bib2">"#,
  ]);
}

/// A `\paragraph` and its text between two entries: the list goes on after it with no title and no TOC entry of its own
/// (pdflatex prints one heading).
#[test]
fn heading_inside_thebibliography_unheaded_continuation_has_no_title() {
  let xml = clean!("index-bib/heading_inside_thebibliography_paragraph_aside.tex");
  assert_eq!(count(&xml, "<title>References</title>"), 1, "{xml}");
  assert_all(&xml, &[
    r#"<p xml:id="S1.SS0.SSS0.Px1.p1.1">An aside inside the list.</p> </para> </paragraph> <bibliography xml:id="biba"> <biblist> <bibitem key="b" xml:id="biba.bib2"> <tags> <tag>[2]</tag> <tag role="refnum">2</tag> </tags> <bibblock> B. Author, J. 2. </bibblock> </bibitem> </biblist> </bibliography> </document>"#,
  ]);
}

/// After the B1 shape, a separate bibliography is the second: `biba` with `biba.bib1` (the continuation used no number
/// of its own).
#[test]
fn heading_inside_thebibliography_then_a_second_bibliography_is_biba() {
  let xml = clean!("index-bib/heading_inside_thebibliography_then_second_bibliography.tex");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bib1">"#,
    r#"<bibitem key="b" xml:id="bib.bib2">"#,
    r#"<bibliography inlist="toc" xml:id="biba"> <title>References</title> <biblist> <bibitem key="c" xml:id="biba.bib1">"#,
  ]);
  assert_eq!(count(&xml, "<bibliography "), 2, "{xml}");
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

/// lipics-v2021's ORCID slot by what it holds (lipics-v2021.cls:797 links it as written): an email address is the author's
/// email, once when the email slot holds it too (2607.06446, all five authors `{}{gerhard@uni-bremen.de}`); a bare iD the
/// author's orcid.org page; any other address the class's link to it.
#[test]
fn lipics_orcid_slot_is_linked_as_written() {
  let xml = clean!("class-bindings/lipics_orcid_slot_as_written.tex");
  assert_all(&xml, &[
    r#"<personname>Gerd Gast</personname> <contact name="Affiliation: " role="affiliation">Uni Bremen</contact> <contact name="Email: " role="email">gerhard@uni-bremen.de</contact> </creator>"#,
    r#"<personname>Hans Hof</personname> <contact name="Affiliation: " role="affiliation">Uni Bremen</contact> <contact name="Email: " role="email">hans_h@uni-bremen.de</contact> </creator>"#,
    r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-009X" title="ORCID 0000-0002-1825-009X">"#,
    r#"<personname>Jan Jung</personname> <contact name="Affiliation: " role="affiliation">Uni L</contact> <contact role="orcid"><ref class="ltx_href" href="https://example.org/~jan"><svg:svg class="ltx_orcidlogo""#,
  ]);
  assert_eq!(count(&xml, r#"role="email""#), 2, "{xml}");
  assert!(
    !xml.contains(r#"href="gerhard"#) && !xml.contains(r#"href="hans"#),
    "{xml}"
  );
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

/// apacite's short cite family takes the `<pre-note>` as `\cite` does (apacite.sty:277-283, :304-310): 1805.05444's
/// `\let\cite\shortcite` then `\cite<e.g.>[]{Rigden_2017}` cites its key, the pre-note before it (natbib's
/// `[e.g.][]`: only an absent post-note swaps, natbib.sty.ltxml:113).
#[test]
fn apacite_shortcite_takes_the_angled_prenote() {
  let xml = clean!("package-bindings/apacite_shortcite_prenote.tex");
  assert_all(&xml, &[
    r#"Stomatal closure <cite class="ltx_citemacro_citep">(e.g. <bibref bibrefs="Rigden_2017" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>)</cite>."#,
    r#"Soil <cite class="ltx_citemacro_citep"><bibref bibrefs="Or_2013" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>, p. 3</cite>.</p>"#,
  ]);
  assert!(!xml.contains("&gt;[]"), "{xml}");
}

/// `\DeclareGraphicsRule` typesets nothing (graphics.sty:150-155; figlatex's `{.pdftex_t}`, 1605.08743).
#[test]
fn declaregraphicsrule_typesets_nothing() {
  let xml = clean!("package-bindings/declaregraphicsrule_typesets_nothing.tex");
  assert_all(&xml, &[
    r#"<para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para> </document>"#,
  ]);
  assert!(!xml.contains("pdftex"), "{xml}");
}

/// In a dcolumn `D` cell the `$` of a box it holds is a math shift (`\multirow{2}{*}{$E_{B}^{PBE}$}`, 2306.05928), the
/// cell's own `$2.5$` keeps its number in the column's math.
#[test]
fn dcolumn_box_in_cell_keeps_its_math_shift() {
  let xml = clean!("package-bindings/dcolumn_box_in_cell_math_shift.tex");
  assert_all(&xml, &[
    r#"<td align="char:." thead="column" xml:id="p1.1.1.3"><text class="ltx_markedasmath" xml:id="p1.1.1.3.1"><Math mode="inline" tex="E_{\mathrm{B}}^{\mathrm{PBE}}" text="(E _ B) ^ PBE" xml:id="p1.m2.m1">"#,
    r#"<td align="char:." xml:id="p1.1.2.3"><Math mode="inline" tex="2.5" text="2.5" xml:id="p1.m4"> <XMath xml:id="p1.m4.1"> <XMTok meaning="2.5" role="NUMBER">2.5</XMTok> </XMath> </Math></td>"#,
  ]);
}

/// arxbj's binding loads elsart_support without elsart.cls's theorems (arxbj.cls:1418-1431 defines `{thm}` only): the
/// author's `\newcommand{\prob}` is theirs (1601.03906).
#[test]
fn arxbj_author_command_is_not_shadowed_by_an_elsart_theorem() {
  let xml = clean!("class-bindings/arxbj_author_command_over_elsart_theorem.tex");
  assert_all(&xml, &[
    r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" xml:id="Thmthm1">"#,
    r#"<XMTok role="SUBSCRIPTOP" scriptpos="post1"/> <XMTok role="UNKNOWN">P</XMTok> <XMTok font="italic" fontsize="70%" role="UNKNOWN">t</XMTok> </XMApp>"#,
  ]);
  assert!(!xml.contains("ltx_theorem_prob"), "{xml}");
}

/// quantumarticle's `\doi` under natbib (quantumarticle.cls:1396-1400; 1902.03164): a doi.org link showing the DOI.
#[test]
fn quantumarticle_doi_is_the_class_link() {
  let xml = clean!("class-bindings/quantumarticle_doi_detokenized.tex");
  assert_all(&xml, &[
    r#"<bibblock> A. Author. A paper. <ref class="ltx_href" href="https://doi.org/10.1007/978-3-540-27821-4_23">DOI: 10.1007/978-3-540-27821-4_23</ref>. </bibblock>"#,
  ]);
}

/// sn-jnl without its `pdflatex` option keeps hyperref's `\href{url}{text}` (OXIDIZED_DESIGN_DIVERGENCES #654: the
/// class's `\let\href\burlalt`, sn-jnl.cls:1462-1467, is not made; 2405.04967, 2207.12959, 2603.09685): the text linked
/// to the URL. The template's swapped order (2401.05832) stays RED (class-bindings/sn_jnl_href_swapped_order).
#[test]
fn sn_jnl_href_keeps_hyperref_order_without_pdflatex() {
  let xml = clean!("class-bindings/sn_jnl_href_keeps_hyperref_order.tex");
  assert_all(&xml, &[
    r#"<p xml:id="p1.1">Data is available at <ref class="ltx_href" href="https://doi.org/10.5281/zenodo.1">Dataset</ref>.</p>"#,
  ]);
}

/// natbib's `\citep[pre][]{k}` keeps the pre-note (natbib.sty:688-690; Perl natbib.sty.ltxml:113 swaps only an absent
/// post-note): "(e.g. Rigden, 2017)", "(e.g. Rigden, 2017, p. 3)", "(Rigden, 2017, p. 3)", "Rigden (see 2017)".
#[test]
fn natbib_empty_postnote_keeps_the_prenote() {
  let xml = clean!("package-bindings/natbib_empty_postnote_keeps_prenote.tex");
  assert_all(&xml, &[
    r#"A <cite class="ltx_citemacro_citep">(e.g. <bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>)</cite>."#,
    r#"B <cite class="ltx_citemacro_citep">(e.g. <bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>, p. 3)</cite>."#,
    r#"C <cite class="ltx_citemacro_citep">(<bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>, p. 3)</cite>."#,
    r#"D <cite class="ltx_citemacro_citet"><bibref bibrefs="k" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(see </bibrefphrase> <bibrefphrase>)</bibrefphrase> </bibref></cite>."#,
  ]);
}

/// natbib's `\cite` with an optional argument, even an empty one, is parenthetical (natbib.sty:694-701; Perl textual
/// for an empty one, KNOWN_PERL_ERRORS #867); `\cite{k}` alone is textual in author-year mode.
#[test]
fn natbib_cite_with_an_optional_argument_is_parenthetical() {
  let xml = clean!("package-bindings/natbib_cite_optional_is_parenthetical.tex");
  let par = r#"<cite class="ltx_citemacro_cite">(<bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>)</cite>."#;
  assert_all(&xml, &[
    r#"A <cite class="ltx_citemacro_cite"><bibref bibrefs="k" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase> <bibrefphrase>)</bibrefphrase> </bibref></cite>."#,
    r#"B <cite class="ltx_citemacro_cite">(<bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>, p. 5)</cite>."#,
    &format!("C {par}"),
    &format!("D {par}"),
    r#"E <cite class="ltx_citemacro_cite">(see <bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>)</cite>."#,
  ]);
}

/// apacite's notes go on to natbib as read (apacite.sty:313-327 `\@cite<pre>[post]`): a lone `[note]` is the post-note
/// (2002.00730's `\cite[p.~5]`, 2005.00039's `\cite[chapter 10]`), a `<pre>` alone the pre-note; `\cite` is parenthetical
/// with or without notes (apacite.sty:259-267, OXIDIZED_DESIGN_DIVERGENCES #655), `\citeA` textual.
#[test]
fn apacite_notes_absent_stay_absent() {
  let xml = clean!("package-bindings/apacite_notes_as_read.tex");
  let bibref = r#"<bibref bibrefs="a" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>"#;
  let textual = r#"<bibref bibrefs="a" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase>"#;
  assert_all(&xml, &[
    &format!(r#"A <cite class="ltx_citemacro_cite">({bibref})</cite>."#),
    &format!(r#"B <cite class="ltx_citemacro_cite">({bibref}, p. 5)</cite>."#),
    &format!(r#"C <cite class="ltx_citemacro_cite">(e.g. {bibref})</cite>."#),
    &format!(r#"D <cite class="ltx_citemacro_cite">(e.g. {bibref}, p. 5)</cite>."#),
    &format!(
      r#"E <cite class="ltx_citemacro_citet">{textual} <bibrefphrase>, p. 6)</bibrefphrase> </bibref></cite>."#
    ),
    &format!(r#"F <cite class="ltx_citemacro_citep">{bibref}, p. 7</cite>."#),
    &format!(r#"G <cite class="ltx_citemacro_citep">({bibref}, p. 8)</cite>."#),
    &format!(
      r#"H <cite class="ltx_citemacro_citet">{textual} <bibrefphrase>)</bibrefphrase> </bibref></cite>."#
    ),
    r#"I <cite class="ltx_citemacro_citeyear"><bibref bibrefs="a" separator=";" show="Year" yyseparator=","/>, p. 9</cite>."#,
  ]);
  assert!(
    !xml.contains("p. 5 <bibref") && !xml.contains("(p. "),
    "{xml}"
  );
}

/// apacite's citation mode chooses its `\cite` (apacite.sty:71-110): under `natbibapa` apacite defines none (:995-1037)
/// and natbib's own stays — `\cite{a}` textual, `\cite[p.~5]{a}` parenthetical (natbib.sty:694-701); the classic
/// default's parenthetical `\cite` is `apacite_notes_absent_stay_absent`'s A. pdflatex: "A Smith (2001). B (Smith, 2001,
/// p. 5). C (Smith, 2001). D Smith (2001). E (e.g. Smith, 2001)." Witnesses 2307.11525, 2406.19732 (the review of 68w r3:
/// every `\cite` was parenthetical).
#[test]
fn apacite_natbibapa_keeps_natbibs_cite() {
  let xml = clean!("package-bindings/apacite_citation_modes.tex");
  let bibref = r#"<bibref bibrefs="a" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>"#;
  let textual = r#"<bibref bibrefs="a" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase> <bibrefphrase>)</bibrefphrase> </bibref>"#;
  assert_all(&xml, &[&format!(
    r#"<p xml:id="p1.1">A <cite class="ltx_citemacro_cite">{textual}</cite>. B <cite class="ltx_citemacro_cite">({bibref}, p. 5)</cite>. C <cite class="ltx_citemacro_citep">({bibref})</cite>. D <cite class="ltx_citemacro_citet">{textual}</cite>. E <cite class="ltx_citemacro_citep">(e.g. {bibref})</cite>.</p>"#
  )]);
}

/// Under apacite's `natbibemu` `\cite` is `\citep` when a `[` follows and `\citet` otherwise, starred alike
/// (apacite.sty:883-892). TL 2025's pdflatex errs in the option's own `\citep`/`\citet` (repro header); read from the
/// source: "A Smith (2001). B (Smith, 2001, p. 5). C Smith (2001)."
#[test]
fn apacite_natbibemu_cite_is_citet_or_citep() {
  let xml = clean!("package-bindings/apacite_natbibemu_cite.tex");
  assert_all(&xml, &[
    r#"<p xml:id="p1.1">A <cite class="ltx_citemacro_citet"><bibref bibrefs="a" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase> <bibrefphrase>)</bibrefphrase> </bibref></cite>. B <cite class="ltx_citemacro_citep">(<bibref bibrefs="a" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>, p. 5)</cite>. C <cite class="ltx_citemacro_citet"><bibref bibrefs="a" separator=";" show="FullAuthors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase> <bibrefphrase>)</bibrefphrase> </bibref></cite>.</p>"#,
  ]);
}

/// nmbib's `\citeall[p.~5]` keeps its lone note the post-note (`\citet*` over the notes as read).
#[test]
fn nmbib_citeall_lone_note_is_the_postnote() {
  let xml = clean!("package-bindings/nmbib_citeall_postnote.tex");
  assert_all(&xml, &[
    r#"A <cite class="ltx_citemacro_citet"><bibref bibrefs="a" separator=";" show="FullAuthors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase> <bibrefphrase>, p. 5)</bibrefphrase> </bibref></cite>."#,
  ]);
}

/// natbib's `\citealt`/`\citealp` pass a present empty post-note on as `[]` (KNOWN_PERL_ERRORS #868): `\citealt[e.g.][]`
/// keeps "e.g." the pre-note, "Smith and Jones e.g. 2001" as pdflatex prints it.
#[test]
fn natbib_citealt_empty_postnote_keeps_the_prenote() {
  let xml = clean!("package-bindings/natbib_citealt_empty_postnote.tex");
  assert_all(&xml, &[
    r#"A <cite class="ltx_citemacro_citet"><bibref bibrefs="a" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>e.g. </bibrefphrase> <bibrefphrase/> </bibref></cite>."#,
    r#"B <cite class="ltx_citemacro_citep">e.g. <bibref bibrefs="a" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref></cite>."#,
    r#"C <cite class="ltx_citemacro_citet"><bibref bibrefs="a" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase/> <bibrefphrase>, p. 5</bibrefphrase> </bibref></cite>."#,
    r#"D <cite class="ltx_citemacro_citep">e.g. <bibref bibrefs="a" separator=";" show="AuthorsPhrase1Year" yyseparator=","> <bibrefphrase>, </bibrefphrase> </bibref>, p. 3</cite>."#,
  ]);
  assert!(!xml.contains(", e.g."), "{xml}");
}

/// A labelled entry before a heading inside `{thebibliography}` (`\bibitem[S01]{a}`, which steps no `@bibitem`) keeps the
/// first bibliography: the continuation has its own number, its entries `biba.bibxN` (OXIDIZED_DESIGN_DIVERGENCES #653).
#[test]
fn heading_inside_thebibliography_after_a_labelled_entry_keeps_both() {
  let xml = clean!("index-bib/heading_inside_thebibliography_labelled_entry.tex");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="a" xml:id="bib.bibx1"> <tags> <tag>[S01]</tag> <tag role="refnum">S01</tag> </tags> <bibblock> A. Author, J. 1. </bibblock> </bibitem> </biblist> </bibliography>"#,
    r#"<bibliography inlist="toc" xml:id="biba"> <title>More references</title> <biblist> <bibitem key="b" xml:id="biba.bibx2">"#,
    r#"<bibitem key="c" xml:id="biba.bibx3">"#,
  ]);
  assert_eq!(count(&xml, "<bibliography "), 2, "{xml}");
}

/// A paragraph before the first heading inside `{thebibliography}` is the first bibliography's entry: the topical groups
/// after it have numbers of their own (`biba`, `bibb`), their entries numbered on (`biba.bib1`, `bibb.bib2`).
#[test]
fn heading_inside_thebibliography_after_a_paragraph_keeps_its_ids() {
  let xml = clean!("index-bib/heading_inside_thebibliography_preamble_paragraph.tex");
  assert_all(&xml, &[
    r#"<bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem xml:id="bib.bibx1"> <bibblock>Sorted by topic. </bibblock> </bibitem> </biblist> </bibliography>"#,
    r#"<bibliography inlist="toc" xml:id="biba"> <title>Books</title> <biblist> <bibitem key="a" xml:id="biba.bib1">"#,
    r#"<bibliography inlist="toc" xml:id="bibb"> <title>Articles</title> <biblist> <bibitem key="b" xml:id="bibb.bib2">"#,
  ]);
  assert!(!xml.contains("bibaa"), "{xml}");
}
