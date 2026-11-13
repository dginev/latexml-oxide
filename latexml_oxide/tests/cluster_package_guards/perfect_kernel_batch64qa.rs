//! Red/green guards for perfect-kernel batch 64qa: the class bindings behind run 336's first-error undefined commands —
//! World Scientific's `\bodymatter` and table rules, iopart's `\ioptwocol`, revtex's `\rev@citealpnum`, svmult's
//! `\toctitle`/`\tocauthor`, mdpi's `\history`/`\changeurlcolor`, WileyNJD's `\state`, ptephy's author block and
//! quantumarticle's `\begin{document}` packages.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// Every creator of `xml`, whole and in document order.
fn creators(xml: &str) -> Vec<String> {
  let mut creators = Vec::new();
  let mut rest = xml;
  while let Some(i) = rest.find("<creator ") {
    let end = i
      + rest[i..]
        .find("</creator>")
        .map_or(0, |j| j + "</creator>".len());
    creators.push(rest[i..end].to_string());
    rest = &rest[end..];
  }
  creators
}

/// 64qa: ws-procs9x6.cls:586-591 `\bodymatter` (`\body`) restarts the footnotes and letters them, and the ws table
/// rules (ws-procs9x6.cls:721-730) add a rule between empty rows of negative height, which the alignment drops: three
/// rows, ruled above the head, below it and at the bottom (0704.0883, 0704.0076). Repro
/// singletons/ws_procs_bodymatter_and_table_rules.
#[test]
fn ws_procs_bodymatter_and_table_rules() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/singletons/ws_procs_bodymatter_and_table_rules.tex"
  ));
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="S1.T1.2"><thead><tr xml:id="S1.T1.2.1"><td align="center" border="t" thead="column" xml:id="S1.T1.2.1.1">A</td><td align="center" border="t" class="ltx_nopad_r" thead="column" xml:id="S1.T1.2.1.2">B</td></tr></thead><tbody><tr xml:id="S1.T1.2.2"><td align="center" border="t" xml:id="S1.T1.2.2.1">1</td><td align="center" border="t" class="ltx_nopad_r" xml:id="S1.T1.2.2.2">2</td></tr><tr xml:id="S1.T1.2.3"><td align="center" border="b" xml:id="S1.T1.2.3.1">3</td><td align="center" border="b" class="ltx_nopad_r" xml:id="S1.T1.2.3.2">4</td></tr></tbody></tabular>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="footnote""#],
    concat!(
      r#"<note mark="a" role="footnote" xml:id="footnote1"><tags><tag>a</tag><tag role="autoref">footnote"#,
      "\u{a0}",
      r#"a<text xml:id="footnote1.1"/></tag><tag role="refnum">a</tag><tag role="typerefnum">footnote a</tag></tags>Note.</note>"#
    ),
  );
}

/// 64qa: iopart.cls:1095-1097 `\ioptwocol` sets the two-column geometry and calls `\twocolumn` (1004.1944). Repro
/// singletons/iopart_ioptwocol.
#[test]
fn iopart_ioptwocol() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/singletons/iopart_ioptwocol.tex"
  ));
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Intro</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">Text.</p></para></section>"#,
  );
}

/// 64qa: revtex4-1.cls:6939-6945 `\rev@citealpnum` is the bare-number `\citenum` (1002.2610). Repro
/// singletons/revtex_rev_citealpnum.
#[test]
fn revtex_rev_citealpnum() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/singletons/revtex_rev_citealpnum.tex"
  ));
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="n""#],
    r#"<bibitem key="n" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="refnum">[2]</tag><tag role="key">n</tag></tags><bibblock> Obtained in Ref. <cite class="ltx_citemacro_citenum"><bibref bibrefs="k" class="ltx_cite_sort ltx_cite_compress" separator="," show="Number" yyseparator=","/></cite>.</bibblock></bibitem>"#,
  );
}

/// 64qa: svmult.cls:939/942 `\newtoks\tocauthor`, `\newtoks\toctitle` — assignments, printing nothing (0704.1469).
/// Repro singletons/svmult_toctitle_tocauthor.
#[test]
fn svmult_toctitle_tocauthor() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/singletons/svmult_toctitle_tocauthor.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>A. Author</personname>\n  </creator>"
  ]);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// 64qa: mdpi.cls:384 `\history` is the author's dates, printed under an accepted paper's abstract (1003.3225), and
/// `\changeurlcolor` (mdpi.cls:295) a `\hypersetup`. Repro sectioning-frontmatter/mdpi_history_is_frontmatter.
#[test]
fn mdpi_history_is_frontmatter() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/mdpi_history_is_frontmatter.tex"
  ));
  // a later `\history` replaces the earlier (mdpi.cls:384 `\gdef`); the document's own id (`id1`) is lastpage's
  // `LastPage` label (65v: the mdpi versions load lastpage), so the note is `id1.1`
  assert_eq!(xml.matches(r#"role="history""#).count(), 1, "{xml}");
  assert_element(
    &xml,
    "note",
    &[r#"role="history""#],
    r#"<note role="history" xml:id="id1.1">Received: 28 March 2020; Accepted: 7 May 2020</note>"#,
  );
}

/// 64qa: a submitted mdpi manuscript prints no `\history` (mdpi.cls:664-666). Repro
/// sectioning-frontmatter/mdpi_history_not_printed_when_submitted.
#[test]
fn mdpi_history_not_printed_when_submitted() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/mdpi_history_not_printed_when_submitted.tex"
  ));
  assert_eq!(xml.matches(r#"role="history""#).count(), 0, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// 64qa: WileyNJD-v2.cls:1220 `\state` prints its argument, an address part (1705.06379). Repro
/// sectioning-frontmatter/wileynjd_state_address_part.
#[test]
fn wileynjd_state_address_part() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/wileynjd_state_address_part.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Naval Surface Warfare Center, Florida, United States</contact>\n  </creator>"
  ]);
}

/// 64qa: quantumarticle.cls:1266-1279 loads tikz at `\begin{document}`, and with it graphicx (2110.03913). Repro
/// singletons/quantumarticle_loads_tikz_graphicx.
#[test]
fn quantumarticle_loads_tikz_graphicx() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/singletons/quantumarticle_loads_tikz_graphicx.tex"
  ));
  // mwe's images are found in the TeX tree, wherever it is installed
  let texmf = regex::Regex::new(r#"candidates="[^"]*/(example-image-a\.[a-z]+)""#).unwrap();
  let xml = texmf.replace_all(&xml, r#"candidates="TEXMF/$1""#);
  assert_element(
    &xml,
    "graphics",
    &[],
    r#"<graphics candidates="TEXMF/example-image-a.png" graphic="example-image-a" options="width=56.9055pt,keepaspectratio=true" xml:id="p1.g1"/>"#,
  );
}

/// 64qa: macro marks (`\name{K.~Fushimi}{\Tokushima,}\thanks{Now at …}`, 1801.03251; `{\AFFicrr,\AFFipmu}`,
/// 2209.07273) are expanded before the kernel reads the author line, so they link and a `\thanks` after them stays a
/// note. Repro sectioning-frontmatter/ptephy_name_macro_marks_split.
#[test]
fn ptephy_name_macro_marks_split() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ptephy_name_macro_marks_split.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Lab B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Lab B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Coe</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_address\" role=\"thanks\" xml:id=\"id1\">Now at Lab C</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>"
  ]);
}

/// 64qa: unlabelled affiliations in a row are all the authors' (`annotate=run`; 1605.07339). Repro
/// sectioning-frontmatter/ptephy_unlabelled_affils_shared.
#[test]
fn ptephy_unlabelled_affils_shared() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ptephy_unlabelled_affils_shared.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Lab B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Lab B</contact>\n  </creator>"
  ]);
}

/// 64qa: authblk-TI's `\author{…}\affil{text}` pairs: each unmarked `\affil` the authors' given since the one before
/// (authblk-TI.sty:104-121; 2311.07297). Repro sectioning-frontmatter/ptephy_v1_authblk_interleaved.
#[test]
fn ptephy_v1_authblk_interleaved() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ptephy_v1_authblk_interleaved.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Lab B</contact>\n  </creator>"
  ]);
}
