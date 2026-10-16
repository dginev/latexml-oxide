//! Red/green guards for batch 68k: the commands of class bindings whose papers ship their own copy of the class —
//! gretsi, PoS, JHEP3, the World Scientific procs and journal classes, svmult, pnas-new, interact, mdpi, svjour3,
//! arxstspdf/arximspdf and siamart/siamonline. Each repro's pdflatex reference text is in its header.
use latexml::util::test::{convert_files_with, convert_with};

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

/// A clean conversion of a class-bindings repro beside the minimal copy of its class (`file`, the copy's name in the
/// paper's source), squashed.
macro_rules! clean_with_copy {
  ($path:literal, $file:literal, $copy:literal) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/class-bindings/",
        $path
      )),
      &[(
        $file,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/class-bindings/",
          $copy
        )),
      )],
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

/// gretsi.cls v1.2 (1001.1912's copy) and v1.3 (2304.10248's): `\titre`, the marked names (`\coord`, `\auteur[4]`)
/// requesting the affiliations `\adresse`/`\affils` label, the French and English abstracts, `\legende` (1001.1912,
/// 0706.1914, 1311.3768, 1512.06999, 2206.05928, 2304.10248, 2310.15635).
#[test]
fn gretsi_frontmatter_follows_the_release() {
  let xml = clean_with_copy!(
    "gretsi_v12/gretsi_v12_frontmatter.tex",
    "gretsi.cls",
    "gretsi_v12/gretsi.cls"
  );
  assert_all(&xml, &[
    "<title>Méthode du point proximal</title>",
    r#"<personname>Ziad <text font="smallcaps" xml:id="id1">Naja<sup xml:id="id1.1"><text font="upright" xml:id="id1.1.1">1</text></sup></text></personname> <contact name="Affiliation: " role="affiliation">Univ Paris-Sud 11</contact> </creator>"#,
    r#"<personname>Pierre <text font="smallcaps" xml:id="id2">Duhamel<sup xml:id="id2.1"><text font="upright" xml:id="id2.1.1">2</text></sup></text></personname> <contact name="Email: " role="email">ziad.naja@lss.fr</contact> <contact name="Affiliation: " role="affiliation">CNRS<break/>Gif-sur-Yvette</contact> </creator>"#,
    r#"<abstract inlist="toc" name="Résumé" xml:id="abstract1"> <p xml:id="abstract1.1">Cet article est basé sur la méthode.</p> </abstract> <abstract inlist="toc" name="Abstract" xml:id="abstract2"> <p xml:id="abstract2.1"><text xml:lang="en" xml:id="abstract2.1.1">This paper recalls the proximal point method.</text></p> </abstract>"#,
    r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>Une figure.</caption>"#,
  ]);
  let xml = clean!("gretsi_v13_frontmatter.tex");
  assert_all(&xml, &[
    "<title>Hotelling Deflation</title>",
    r#"<personname>Mohamed <text font="smallcaps" xml:id="id1">Seddik<sup xml:id="id1.1"><text font="upright" xml:id="id1.1.1">1</text></sup></text></personname> <contact name="Email: " role="email">mohamed.seddik@tii.ae</contact> <contact name="Affiliation: " role="affiliation">Technology Innovation Institute, Abu Dhabi</contact> </creator>"#,
    r#"<personname>Maxime <text font="smallcaps" xml:id="id2">Guillaud<sup xml:id="id2.1"><text font="upright" xml:id="id2.1.1">2</text></sup></text></personname> <contact name="Email: " role="email">maxime.guillaud@inria.fr</contact> <contact name="Affiliation: " role="affiliation">Inria, Villeurbanne</contact> </creator>"#,
    r#"<abstract inlist="toc" name="Résumé" xml:id="abstract1"> <p xml:id="abstract1.1"><text xml:lang="fr" xml:id="abstract1.1.1">Cet article étudie la déflation.</text></p> </abstract> <abstract inlist="toc" name="Abstract" xml:id="abstract2"> <p xml:id="abstract2.1">This paper studies deflation.</p> </abstract>"#,
  ]);
}

/// PoS.cls `\@firstaubox`, `\PoSpaper@url`, `\PoSspecialurl` (`\relax` from `\begin{document}`, so a body call prints
/// its URL) and the AAS abbreviations a copy pastes in (0912.1441, 1606.09248, 1812.08139, 0810.0108).
#[test]
fn pos_class_commands() {
  let xml = clean_with_copy!(
    "pos_aas/pos_class_commands.tex",
    "PoS.cls",
    "pos_aas/PoS.cls"
  );
  assert_all(&xml, &[
    r#"<para xml:id="p1"> <p xml:id="p1.1">Our tool is available at:</p> <p align="center" xml:id="p1.2">http://tool.example.org/</p> <p xml:id="p1.3">As shown in ApJ, MNRAS and A&amp;A.</p> </para>"#,
  ]);
}

/// JHEP3.cls `\arXivid` (0704.3719, 0709.3309) and `\@firstaubox` (hep-ph0112089).
#[test]
fn jhep3_arxivid_links_the_identifier() {
  let xml = clean!("jhep3_arxivid.tex");
  assert_all(&xml, &[
    r#"<pubnote name="Preprint: " role="preprint"><ref font="typewriter" href="http://arxiv.org/abs/0704.3719 [hep-th]">arXiv:0704.3719 [hep-th]</ref></pubnote>"#,
    r#"<bibblock> B. Author, <emph font="italic" xml:id="bib.bib1.2">A paper</emph>, <ref font="typewriter" href="http://arxiv.org/abs/0706.2722">arXiv:0706.2722</ref>. </bibblock>"#,
  ]);
}

/// ws-procs9x6.cls's cross-reference names, `\citelow`, `{tabnote}`, `\tablefont`, `\wstoc`, `{algorithm}`,
/// `{unnumlist}`, `\figurebox` (0704.0369, 1605.04791, 1302.0454, 1109.0078, 1402.0787, nucl-th0412108).
#[test]
fn ws_procs_class_commands() {
  let xml = clean!("ws_procs_class_commands.tex");
  assert_all(&xml, &[
    r#"<p xml:id="S1.p1.1">As Eq. (<ref labelref="LABEL:e"/>) in Sec. <ref labelref="LABEL:sec:i"/>, Fig. <ref labelref="LABEL:f"/> and Table <ref labelref="LABEL:t"/> show (Equation (<ref labelref="LABEL:e"/>), Section <ref labelref="LABEL:sec:i"/>, Figure <ref labelref="LABEL:f"/>, Table <ref labelref="LABEL:t"/>); see also <cite class="ltx_citemacro_cite">[<bibref bibrefs="k" separator="," yyseparator=","/>]</cite>.</p>"#,
    r#"<p class="ltx_figure_panel" xml:id="S1.T1.3"><text font="italic" fontsize="80%" xml:id="S1.T1.3.1">Note<text font="upright" xml:id="S1.T1.3.1.1">: Values in units.</text></text></p>"#,
    r#"<title class="ltx_runin"><tag><text font="bold" xml:id="S1.Thmalgorithm1.2">Algorithm 1.1</text></tag><text font="bold" xml:id="S1.Thmalgorithm1.3">.</text></title>"#,
    r#"<p xml:id="S1.I1.i1.p1.1">First.</p>"#,
  ]);
}

/// ws-ijmpe.cls `{otherinfo}`, `{tabfootnote}` with `\tabmark`, `{tabnote}[Source]`, `\titlefont`, `\authorfont`
/// (0811.0496, 0712.0330, 1404.6651, 1111.6854).
#[test]
fn ws_journal_table_notes() {
  let xml = clean!("ws_ijmpe_table_notes.tex");
  assert_all(&xml, &[
    r#"<p align="center" xml:id="p1.1"><text fontsize="80%" xml:id="p1.1.1">Published in: Int. J. Mod. Phys. E, Vol. 18 (2009)</text></p>"#,
    r#"<p class="ltx_figure_panel" xml:id="S1.T1.3"><text fontsize="80%" xml:id="S1.T1.3.1"><sup xml:id="S1.T1.3.1.1"><text font="italic" fontsize="125%" xml:id="S1.T1.3.1.1.1">a</text></sup></text><text fontsize="80%" xml:id="S1.T1.3.2">A footnote.</text></p>"#,
    r#"<p class="ltx_figure_panel" xml:id="S1.T1.4"><text font="italic" fontsize="80%" xml:id="S1.T1.4.1">Source<text font="upright" xml:id="S1.T1.4.1.1">: From the survey.</text></text></p>"#,
  ]);
}

/// svmult.cls `\ts`, `\if@runhead`, `\svparindent`, `\greeksym` (the Symbol font's Greek), `\subfigures` (1a, 1b, then
/// 2), `{abbrsymblist}` and the AAS abbreviations a copy pastes in (1001.0767, 0803.2826, 1607.02736, 1111.1183,
/// 1310.5064, 0810.2505, 1711.01112).
#[test]
fn svmult_class_commands() {
  let xml = clean_with_copy!(
    "svmult_aas/svmult_class_commands.tex",
    "svmult.cls",
    "svmult_aas/svmult.cls"
  );
  assert_all(&xml, &[
    r#"<p xml:id="p1.1">Running heads on.</p>"#,
    r#"<p xml:id="Ch0.S1.p1.1">A thin space, an indent of 12.0pt, an upright <text class="ltx_markedasmath" xml:id="Ch0.S1.p1.1.1">Δ</text> and <text xml:id="Ch0.S1.p1.1.2">μ</text>, and ApJ, MNRAS and A&amp;A.</p>"#,
    r#"<caption><tag close=": ">Figure 1a</tag>First.</caption>"#,
    r#"<caption><tag close=": ">Figure 1b</tag>Second.</caption>"#,
    r#"<caption><tag close=": ">Figure 2</tag>Third.</caption>"#,
    r#"<section xml:id="Ch0.Sx1"> <title>List of Abbreviations and Symbols</title>"#,
  ]);
}

/// pnas-new.cls v1.47 `\Firstpage`, `\Parasplit`/`\Endparasplit` (one paragraph), `\dataavail`, and v1.46 `\bibsplit`
/// (2602.17158, 2603.21852, 2208.11618). The one warning is the binding's mdframed stub, as for every pnas-new paper.
#[test]
fn pnas_layout_and_data_availability() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/pnas_layout_and_dataavail.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("mdframed.sty is only minimally stubbed"),
    "{log}"
  );
  let xml = squash(&xml);
  assert_all(&xml, &[
    r#"<para xml:id="p1"> <p xml:id="p1.1">First paragraph, continued across the column.</p> </para> <paragraph xml:id="S0.SS0.SSS0.Px1"> <title>Data, Materials, and Software Availability.</title> <para xml:id="S0.SS0.SSS0.Px1.p1"> <p xml:id="S0.SS0.SSS0.Px1.p1.1">The data are on GitHub.</p> </para> </paragraph>"#,
  ]);
}

/// interact.cls `{abbreviations}`, `{jelcode}`, `{pacscode}`, the upright Greek letters and their bold forms,
/// `{tabnote}` (1901.07015, 2306.12924, 1610.08232, 1709.03559, 1711.08876).
#[test]
fn interact_class_commands() {
  let xml = clean!("interact_class_commands.tex");
  assert_all(&xml, &[
    r#"<para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Abbreviations: </text>GDP: gross domestic product</p> </para>"#,
    r#"<classification scheme="JEL">C21; C58</classification> <classification scheme="PACS">02.50.Ey</classification>"#,
    r#"<XMTok name="umu" role="UNKNOWN">μ</XMTok>"#,
    r#"<XMTok name="upartial" role="UNKNOWN">∂</XMTok>"#,
    r#"<XMTok font="bold" name="udelta" role="UNKNOWN">δ</XMTok>"#,
    r#"<p class="ltx_figure_panel" xml:id="S1.T1.3"><text font="italic" fontsize="80%" xml:id="S1.T1.3.1">Note:<text font="upright" xml:id="S1.T1.3.1.1"> Values in units.</text></text></p>"#,
  ]);
}

/// mdpi.cls `\orcidA` (the iD the paper gives `\orcidauthorA`), `\conference`, `\fourthnote`, `\conflictofinterest`,
/// `\useofartificialintelligence` and the AAS abbreviations a copy pastes in (2311.18672, 1709.06513, 1902.06114,
/// 1710.04728, 2609.34305, 2212.11355); `\linksupplementary` (2208.12066) is `mdpi_supplementary_link_under_the_doi`'s.
#[test]
fn mdpi_class_commands() {
  let xml = clean_with_copy!(
    "mdpi_aas/mdpi_class_commands.tex",
    "Definitions/mdpi.cls",
    "mdpi_aas/Definitions/mdpi.cls"
  );
  assert_all(&xml, &[
    r#"<personname>Ann Author<sup xml:id="id1">1</sup></personname> <contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097">"#,
    r#"<note role="fourthnote" xml:id="id14">A fourth note.</note>"#,
    r#"<p xml:id="S1.p1.1">As shown in ApJ and MNRAS.</p>"#,
    r#"<para class="ltx_noindent" xml:id="Sx2.p2"> <p xml:id="Sx2.p2.1"><text font="bold" xml:id="Sx2.p2.1.1">Conflicts of Interest:</text> None declared.</p> </para> <para class="ltx_noindent" xml:id="Sx2.p3"> <p xml:id="Sx2.p3.1"><text font="bold" xml:id="Sx2.p3.1.1">Use of Artificial Intelligence:</text> No AI was used.</p> </para>"#,
  ]);
}

/// svjour3.cls `\lastand` (an `\and`), the 2023 edition's `{funding}`, `{dataavailability}`, `{conflict}`, and the
/// economics edition's `\JEL` (1907.12906, 2411.13175, 2307.15129, 1402.2599).
#[test]
fn svjour3_editions_commands() {
  let xml = clean!("svjour3_statements.tex");
  assert_all(&xml, &[
    "<personname>Silvia Chiappa</personname>",
    "<personname>Bob Builder</personname>",
    "<personname>Ulrich Paquet</personname>",
    r#"<tag><text font="bold" fontsize="90%" xml:id="S1.I1.ix1.1">Funding</text></tag>"#,
    r#"<tag><text font="bold" fontsize="90%" xml:id="S1.I2.ix1.1">Data Availability</text></tag> </tags> <para xml:id="S1.I2.ix1.p1"> <p xml:id="S1.I2.ix1.p1.1"><text fontsize="90%" xml:id="S1.I2.ix1.p1.1.1">The data are available on request.</text></p> </para> </item>"#,
    r#"<tag><text font="bold" fontsize="90%" xml:id="S1.I3.ix1.1">Conflict of interest</text></tag> </tags> <para xml:id="S1.I3.ix1.p1"> <p xml:id="S1.I3.ix1.p1.1"><text fontsize="90%" xml:id="S1.I3.ix1.p1.1.1">The authors declare none.</text></p> </para> </item>"#,
  ]);
  let xml = clean!("svjour3_jel_classification.tex");
  assert_all(&xml, &[
    r#"<classification name="JEL Classification:" scheme="JEL">C21 <Math mode="inline" tex="\cdot" text="cdot" xml:id="m1"> <XMath xml:id="m1.1"> <XMTok name="cdot" role="MULOP">⋅</XMTok> </XMath> </Math> C58</classification>"#,
  ]);
}

/// arxstspdf.cls `\relateddois`, `\freefootnotetext`, `\shref`, `\ccline`, `\bhline` (1210.0385, 1011.1160, 1008.4373,
/// 1010.0304).
#[test]
fn arxstspdf_class_commands() {
  let xml = clean!("arxstspdf_class_commands.tex");
  assert_all(&xml, &[
    r#"<note role="relateddois" xml:id="id1">Discussed in <ref class="ltx_href" href="http://dx.doi.org/10.1214/12-STS381A">10.1214/12-STS381A</ref>.</note> <note role="footnotetext" xml:id="footnotex1">This conversation was conducted in spring 2008.</note>"#,
    r#"<p xml:id="S1.p1.1">See <ref class="ltx_href" href="http://www.example.org/a/b">http://www.example.org/a/b</ref>"#,
    r#"<tbody> <tr xml:id="S1.p1.1.1.2"> <td align="center" border="b t" xml:id="S1.p1.1.1.2.1">1</td> <td align="center" border="b t" xml:id="S1.p1.1.1.2.2">2</td> <td align="center" border="b t" xml:id="S1.p1.1.1.2.3">3</td> </tr> </tbody>"#,
  ]);
}

/// siamart.cls `\proofbox`, `{MSCcode}`, `{AM}`, `{DOI}`, `{romannum}`, `{remunerate}`, `\appendixnotitle` (1805.01916,
/// 2101.02911, 1801.00197, 1901.02925, 1902.07263, 1601.00044, 1703.08345).
#[test]
fn siamart_class_commands() {
  let xml = clean!("siamart_class_commands.tex");
  assert_all(&xml, &[
    r#"<classification scheme="MSC">65N30</classification> <classification scheme="AMS">65N12</classification>"#,
    r#"<p xml:id="p1.p1.1"><text xml:id="p1.p1.1.1">DOI. </text>10.1137/xxxx</p>"#,
    r#"<p xml:id="S1.p1.1">The proof ends with ∎.</p>"#,
    r#"<tag>(ii)</tag>"#,
    r#"<p xml:id="S1.I2.i2.p1.1">Two.</p>"#,
    r#"<title><tag close=" ">Appendix A</tag></title>"#,
  ]);
}

/// Round 2 (review findings 1-3, 5-8). A conversion of a class-bindings repro beside the files of its source
/// (`files`: name in the source, repro path), with its log and squashed XML.
fn convert_beside(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let (log, xml) = convert_files_with(tex, files, Some("ar5iv.sty"));
  (log, squash(&xml))
}

/// mdpi.cls `\linksupplementary{file}` links the article's supplementary file under its DOI: the paper's `\doinum`,
/// else the default the copy computes from its journal table, the volume, the issue and the article number
/// (2208.12066's :523-530, 10.3390/s23001234; `preprints`, 10.3390/23001234); with neither the copy nor a journal it
/// names, the file's name and one Warning (2208.12066, 2602.00080, 1901.06933, 2311.18672, 2311.15904, 2111.09774).
#[test]
fn mdpi_supplementary_link_under_the_doi() {
  let copies = [
    (
      "Definitions/mdpi.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/mdpi_doi/Definitions/mdpi.cls"
      ),
    ),
    (
      "Definitions/journalnames.tex",
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/mdpi_doi/Definitions/journalnames.tex"
      ),
    ),
  ];
  let (log, xml) = convert_beside(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/mdpi_doi/mdpi_linksupplementary.tex"
    ),
    &copies,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<p xml:id="S1.p1.1">The supporting information can be downloaded at: <ref class="ltx_url" font="typewriter" href="https://www.mdpi.com/article/10.3390/s23001234/s1">https://www.mdpi.com/article/10.3390/s23001234/s1</ref>.</p>"#,
  ]);
  // (`preprints`, whose row's abbreviation is empty: 2111.09774, 2211.05018, 2411.01130)
  let (log, xml) = convert_beside(
    &include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/mdpi_doi/mdpi_linksupplementary.tex"
    )
    .replace("[sensors,", "[preprints,"),
    &copies,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<p xml:id="S1.p1.1">The supporting information can be downloaded at: <ref class="ltx_url" font="typewriter" href="https://www.mdpi.com/article/10.3390/23001234/s1">https://www.mdpi.com/article/10.3390/23001234/s1</ref>.</p>"#,
  ]);
  let (log, xml) = convert_beside(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/mdpi_doi/mdpi_linksupplementary_doinum.tex"
    ),
    &copies,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<p xml:id="S1.p1.1">The supporting information can be downloaded at: <ref class="ltx_url" font="typewriter" href="https://www.mdpi.com/article/10.3390/s99999999/s1">https://www.mdpi.com/article/10.3390/s99999999/s1</ref>.</p>"#,
  ]);
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/mdpi_doi/mdpi_linksupplementary_nodoi.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Warning:missing:\\linksupplementary"), "{log}");
  assert_all(&squash(&xml), &[
    r#"<p xml:id="S1.p1.1">The supporting information can be downloaded at: s1.</p>"#,
  ]);
}

/// gretsi.cls prints the abstracts the paper stores, the French one first, the last call of each winning: v1.2 (1001.1912's
/// copy, :219-224) after `\englishabstract` twice and `\frenchabstract` (2206.05928, 2205.13867, 2411.07577); v1.3
/// (2304.10248's `\maketitle`) after `\abstract` and `\resume` (2304.07234). The text between v1.3's `\auteur` calls
/// is kept with the name before it, past the separators (2508.03934).
#[test]
fn gretsi_abstracts_in_class_order() {
  let (log, xml) = convert_beside(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/gretsi_v12/gretsi_v12_abstract_order.tex"
    ),
    &[(
      "gretsi.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/gretsi_v12/gretsi.cls"),
    )],
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<abstract inlist="toc" name="Résumé" xml:id="abstract1"> <p xml:id="abstract1.1">Le résumé.</p> </abstract> <abstract inlist="toc" name="Abstract" xml:id="abstract2"> <p xml:id="abstract2.1">The English abstract.</p> </abstract>"#,
  ]);
  assert!(!xml.contains("A first English draft"), "{xml}");
  let xml = clean!("gretsi_v13_abstract_order.tex");
  assert_all(&xml, &[
    r#"<abstract inlist="toc" name="Résumé" xml:id="abstract1"> <p xml:id="abstract1.1"><text xml:lang="fr" xml:id="abstract1.1.1">Le résumé.</text></p> </abstract> <abstract inlist="toc" name="Abstract" xml:id="abstract2"> <p xml:id="abstract2.1">The English abstract.</p> </abstract>"#,
  ]);
}

/// svmult.cls `\if@runhead` follows the version's option (v1.1 `runningheads`, astro-ph0108509's copy :37-38; v5.x
/// `norunningheads`, :115-116: 0803.2826, 0808.2157), v1.1's `\micro` is Euler's upright Greek mu (U+03BC, :74;
/// astro-ph0108509, hep-ph0301145), and `\subfigures` numbers a `\title*` contribution's figures without the chapter,
/// whatever its counter (:1918-1920 `\if@numart\else\thechapter.\fi`; 1310.5064, 1411.2805).
#[test]
fn svmult_options_micro_and_chapter_subfigures() {
  let (log, xml) = convert_beside(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/svmult_v11/svmult_v11_runningheads_micro.tex"
    ),
    &[(
      "svmult.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/svmult_v11/svmult.cls"),
    )],
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_all(&xml, &[
    "<p xml:id=\"p1.1\">Running heads on.A 5 <text xml:id=\"p1.1.1\">\u{3bc}</text>m grain.</p>",
  ]);
  let xml = clean_with_copy!(
    "svmult_aas/svmult_norunningheads_subfigures.tex",
    "svmult.cls",
    "svmult_aas/svmult.cls"
  );
  assert_all(&xml, &[
    "Running heads off.",
    r#"<caption><tag close=": ">Figure 1a</tag>First.</caption>"#,
    r#"<caption><tag close=": ">Figure 1b</tag>Second.</caption>"#,
  ]);
}

/// gretsi v1.3 `\auteurs`: the text between the `\auteur` calls past the separators is the name before it ("(chair)"),
/// the comma between two calls the list's (2508.03934). (Known residual, reported in round 2: David Picard's "Lab B"
/// goes to Simon Lepage alone — the comma of Simon's shown marks "1,2" splits a stray label `2` off his fuzzy name label,
/// the pattern of `svjour3_marked_pieces_after_a_break`'s witness.)
#[test]
fn gretsi_v13_auteurs_keep_the_text_between_calls() {
  let xml = clean!("gretsi_v13_abstract_order.tex");
  assert_all(&xml, &[
    r#"<creator role="author"> <personname>Simon <text font="smallcaps" xml:id="id1">Lepage<sup xml:id="id1.1"><text font="upright" xml:id="id1.1.1">1,2</text></sup></text></personname> <contact name="Email: " role="email">s.lepage@x.com</contact> <contact name="Affiliation: " role="affiliation">Lab A, Paris</contact> <contact name="Affiliation: " role="affiliation">Lab B, Marne</contact> </creator>"#,
    r#"<personname>Jeremie <text font="smallcaps" xml:id="id2">Mary<sup xml:id="id2.1"><text font="upright" xml:id="id2.1.1">1</text></sup></text></personname> <contact name="Email: " role="email">j.mary@x.com</contact> <contact name="Affiliation: " role="affiliation">Lab A, Paris</contact> </creator>"#,
    r#"<personname>David <text font="smallcaps" xml:id="id3">Picard<sup xml:id="id3.1"><text font="upright" xml:id="id3.1.1">2</text></sup></text> (chair)</personname> <contact name="Email: " role="email">d.picard@x.fr</contact>"#,
  ]);
}

/// interact.cls :223-224 `\ubvarrho`, `\ubvarsigma`: `\boldsymbol{\varrho}`, `\boldsymbol{\varsigma}` (1711.05866's copy).
#[test]
fn interact_bold_var_greek() {
  let xml = clean!("interact_class_commands.tex");
  assert_all(&xml, &[
    r#"<XMTok font="bold italic" name="varrho" role="UNKNOWN">ϱ</XMTok>"#,
    r#"<XMTok font="bold italic" name="varsigma" role="UNKNOWN">ς</XMTok>"#,
  ]);
}

/// svjour3 `\institute` pieces each opened by a `\\` after `\and` (`\and\\ \textsuperscript{2} Univ B`) still lead with
/// their marks: each author gets the institutes its shown marks name (1810.10167).
#[test]
fn svjour3_marked_pieces_after_a_break() {
  let xml = clean!("svjour3_marked_pieces_after_a_break.tex");
  assert_all(&xml, &[
    r#"<personname>Fan Zhang<sup xml:id="id2">1,3</sup></personname> <contact name="Affiliation: " role="affiliation"> Univ A, Shanghai</contact> <contact name="Affiliation: " role="affiliation"> Univ C, Beijing</contact> </creator>"#,
    r#"<personname>Yaohua Hu<sup xml:id="id3">2</sup></personname> <contact name="Affiliation: " role="affiliation"> Univ B, Shenzhen</contact> </creator>"#,
  ]);
  assert!(!xml.contains("<creator role=\"author\"> <contact"), "{xml}");
}
