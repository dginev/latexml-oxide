//! Red/green guards for batch 68a: journal class bindings define the commands their classes define (binding gaps G2,
//! journals B: aa, acmart, aastex63/aastex631, mnras, llncs, jfm, tac, aamas; the run-347 inventory
//! bindgaps/g23_macros.tsv). Each repro is checked against pdflatex with TeX Live's class or a witness's copy.
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

/// aa.cls v7 (`\if@referee`, `\if@onecolumn`, 1201.4282) and v9.1 (`\arxiv`, `{theorem}`, `\topics`, `\LEt`,
/// `\yearCop`, `\etalname`; the language editor's `\raa`, `\jpb`: 2008.09582).
#[test]
fn aa_class_commands_v7_v91() {
  let xml = clean!("aa_class_commands_v7_v91.tex");
  assert!(
    xml.contains(r#"<note role="topics" xml:id="id1">The XXL Survey: second series</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<p xml:id="p1.1">Mode REF cols 2. See <text font="typewriter" xml:id="p1.1.1">arXiv:2101.00001</text> and Res. Astron. Astrophys., J. Phys. B. <text color="#FF0000" font="bold" xml:id="p1.1.2">[Note 1: check this] </text></p>"##),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" xml:id="Thmtheorem1"> <tags> <tag>Theorem 1</tag>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<p xml:id="p2.1">et al.</p>"#), "{xml}");
}

/// acmart.cls (TL 2025 v2.16): the conference parts `\acmConference@…`, the `\@received` history, `\@subtitle`, the
/// sigconf title fonts, the title-block registers, `\acksname`, and environ's `\NewEnviron` (acmart.cls:288).
#[test]
fn acmart_conference_parts_and_title_fonts() {
  let xml = clean!("acmart_conference_parts_and_title_fonts.tex");
  assert!(
    xml.contains(r#"<pubnote name="Conference: " role="conference">The Conference; August 2026; Toronto, Canada</pubnote>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Date August 2026; short KDD ’26; venue Toronto, Canada. History: Received 20 February 2007; revised 12 March 2009. Sub: Sub. <text font="sansserif bold" fontsize="298%" xml:id="p1.1.1">Big title</text> <text fontsize="173%" xml:id="p1.1.2">Au</text> <text fontsize="120%" xml:id="p1.1.3">Af</text> [inside]</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<section xml:id="Sx1"> <title>Acknowledgments</title>"#),
    "{xml}"
  );
}

/// aastex631.cls (2107.07519's copy): the edit and affiliation switches, `\authorcomment` (turned off: prints nothing),
/// `\mdash`, `\apjsecfont`, `\aspconf`, `\tablereferences`, `\resetapptablenumbers`.
#[test]
fn aastex63x_table_and_edit_commands() {
  let xml = clean!("aastex63x_table_and_edit_commands.tex");
  assert!(
    xml.contains(
      r#"<p xml:id="S1.p1.1">Note and — and X, Ast. Soc. of the Pac. Conference Series.</p>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text fontsize="80%" xml:id="S1.T1.3.1">References. — Ref1</text>"#),
    "{xml}"
  );
}

/// mnras.cls (TL 2025 v3.2, :1399-1479): `\doi`, `\eprint` (arXiv, dblp), `\mniiiauthor`; v3.0's `\apspr`.
#[test]
fn mnras_reference_links() {
  let xml = clean!("mnras_reference_links.tex");
  assert!(
    xml.contains(r#"<bibblock> Author A., Bee B., Cee C. 2020, MNRAS, 1, 2, <ref class="ltx_href" href="http://dx.doi.org/10.1093/mnras/staa_1">doi:10.1093/mnras/staa˙1</ref> </bibblock>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<bibblock> Author A., et al. 2021, preprint (<ref class="ltx_href" font="typewriter" href="http://arxiv.org/abs/2101.00001">arXiv:2101.00001</ref>), <ref class="ltx_href" href="http://dblp.uni-trier.de/rec/bibtex/abc.xml">dblp:abc</ref>, Astrophys. Space Phys. Res.</bibblock>"#),
    "{xml}"
  );
}

/// llncs.cls (TL 2025 v2.26): the option switches, `\spn@wtheorem` and `\@spothm` sharing the theorem counter under
/// `envcountsame`, the language switches, `\@subtitle`, `\institutename`, `\phantomsection`.
#[test]
fn llncs_class_internals() {
  let xml = clean!("llncs_class_internals.tex");
  assert!(xml.contains(r#"<p xml:id="p1.1">SUB:S</p>"#), "{xml}");
  assert!(xml.contains(r#"<p xml:id="p2.1">RH SAME</p>"#), "{xml}");
  // One theorem counter under envcountsame, as pdflatex numbers them (KNOWN_PERL_ERRORS #743, fixed in 68g; the
  // binding numbered `{theorem}` within the section and printed "Fact 0.2", "Assumption 0.3").
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="Thmtheorem2"> <tags> <tag>Fact 2</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_assumption" inlist="thm theorem:assumption" xml:id="Thmtheorem3"> <tags> <tag>Assumption 3</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p3.1">Abstract/Proof</p>"#),
    "{xml}"
  );
}

/// jfm.cls (1702.03412's copy): the figure-legend symbols, `\rmDelta`, `\returnthesubequation` and the title-page
/// switches.
#[test]
fn jfm_legend_symbols_and_subequations() {
  let xml = clean!("jfm_legend_symbols_and_subequations.tex");
  assert!(
    xml.contains(r#"<text xml:id="p1.1.5">- - - -</text>, <text xml:id="p1.1.6">——</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text xml:id="p1.1.9">— — —</text>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="\blacksquare""#), "{xml}");
  assert!(xml.contains(r#"tex="\Delta x""#), "{xml}");
  assert!(xml.contains(r#"<tag>(1b)</tag>"#), "{xml}");
}

/// tac.cls (2401.05974's 2023 copy): `\mathrmdef` and its kin, `\dedication` (a pubnote, as aa's and svjour's), `\endproofbox`, `{references}` and
/// `\refs`…`\endrefs` as bibliographies.
#[test]
fn tac_class_commands() {
  let xml = clean!("tac_class_commands.tex");
  assert!(
    xml.contains(r#"<pubnote name="Dedication: " role="dedication">To C</pubnote>"#),
    "{xml}"
  );
  for tex in [
    r"{\rm Hom}(A,B)",
    r"{\rm\bf C}",
    r"\mathop{\rm{}colim}\nolimits F",
    r"{\mathcal{A}}",
    r"{\sf S}",
  ] {
    assert!(xml.contains(&format!(r#"tex="{tex}""#)), "{tex}: {xml}");
  }
  assert!(
    xml.contains(r#"<rule depth="0.0pt" height="5.0pt" width="5.0pt"/>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<bibliography ").count(), 2, "{xml}");
  assert!(
    xml.contains(r#"<bibitem key="joint" xml:id="biba.bibx1"> <tags> <tag>[Behrens 2005]</tag>"#),
    "{xml}"
  );
}

/// aamas.cls 2026 (2602.10598's copy :1435): a preamble `\doi{SUFFIX}` is the paper's DOI under 10.65109.
#[test]
fn aamas_doi() {
  let xml = clean!("aamas_doi.tex");
  assert!(
    xml.contains(r#"<pubnote name="DOI: " role="doi">10.65109/SEHD8631</pubnote>"#),
    "{xml}"
  );
  assert_eq!(xml.matches(r#"role="doi""#).count(), 1, "{xml}");
}

/// aamas.cls 2021 v1.78 (2510.21200's, 2602.05888's copy) defines no `\doi`: a reference list's own
/// `\providecommand{\doi}[1]{doi: #1}` prints the DOI (pdflatex: "Title. doi: 10.1145/12345."), where the 2026 copy's
/// `\doi` took it silently.
#[test]
fn aamas_doi_older_copy() {
  let xml = clean!("aamas_doi_older_copy.tex");
  assert!(
    xml.contains(r#"<bibblock> A. Author. Title. doi: 10.1145/12345. </bibblock>"#),
    "{xml}"
  );
  assert_eq!(xml.matches(r#"role="doi""#).count(), 0, "{xml}");
}

/// aamas.cls 2026 (2602.10598's copy :1435-1436, shipped): the paper's copy defines `\doi`, so the preamble's
/// `\doi{SUFFIX}` stays the DOI note.
#[test]
fn aamas_doi_2026_copy() {
  let xml = clean!("aamas_doi_2026_copy.tex");
  assert!(
    xml.contains(r#"<pubnote name="DOI: " role="doi">10.65109/SEHD8631</pubnote>"#),
    "{xml}"
  );
  assert_eq!(xml.matches(r#"role="doi""#).count(), 1, "{xml}");
}

/// aa.cls before v8.2 (1201.4282's v7.0) has no `\arxiv` and no `\etalname`: a paper's own definitions win over the
/// binding's (pdflatex: "Text MyJPB, ARX(1234.5678), EA.").
#[test]
fn aa_class_newer_commands_yield() {
  let xml = clean!("aa_class_newer_commands_yield.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Text MyJPB, ARX(1234.5678), EA.</p>"#),
    "{xml}"
  );
}

/// AASTeX 5.2 (1001.0026's aastex.cls) has none of the 6.2+ commands aas_support adds: a paper's own definitions win
/// (pdflatex: "A–B, CAP(c), BB(x,y), AC(1,n), SF, FN(f), TR(t), RA.").
#[test]
fn aastex5_newer_commands_yield() {
  let xml = clean!("aastex5_newer_commands_yield.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">A–B, CAP(c), BB(x,y), AC(1,n), SF, FN(f), TR(t), RA.</p>"#),
    "{xml}"
  );
}

/// `\authorcomment`'s label follows the version: aastex63 without a copy prints "(Author note: …)" (2205.09859's
/// aastex63.cls :5731), a shipped aastex63.cls with aastex631's numbered label "(Author note 1: …)" (2312.04630).
#[test]
fn aastex63_authorcomment_label() {
  let xml = clean!("aastex63_authorcomment_label.tex");
  assert!(
    xml.contains(r##"<p xml:id="S1.p1.1">Note <text color="#000000" font="bold" xml:id="S1.p1.1.1">(Author note: check) </text> here.</p>"##),
    "{xml}"
  );
  let xml = clean!("aastex63_copy_numbered_authorcomment.tex");
  assert!(
    xml.contains(r##"<p xml:id="S1.p1.1">Note <text color="#000000" font="bold" xml:id="S1.p1.1.1">(Author note 1: check) </text> here.</p>"##),
    "{xml}"
  );
}
