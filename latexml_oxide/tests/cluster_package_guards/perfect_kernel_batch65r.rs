//! Red/green guards for batch 65r: bindings load what their TeX Live packages load (round 2 of 65k's mechanism).
//! amsrefs loads url, rkeyval, textcmds and mathscinet and defines `\bibselect`, `\resetbiblist`, `{biblist*}`;
//! thm-restate loads thmtools; elsarticle's `1p`/`3p`/`5p` load geometry; xy's frame feature requires curve (the pdf
//! driver's frame file does, xypdf-fr.tex:60); revtex defines `\sanitize@url`; and a repeat `\usepackage[…]{hyperref}`
//! after a class binding's own hyperref load takes its options through `\hypersetup`, so `colorlinks` loads color.
//! Repros under `tools/perfect_kernel/repros/stub-dependencies/`; the `\bibselect` guards are in
//! `06_cluster_bibliography` (post-processing).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The log and the XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> (String, String) {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  (log, xml)
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stub-dependencies/",
      $name,
      ".tex"
    ))
  };
}

/// amsrefs.sty:45-46, 213-217: url, rkeyval, textcmds and mathscinet come with amsrefs, so a `\bib` entry's
/// `\cprime`, `\germ` and `\url` are defined (witnesses 1907.08888, 2602.02342, math0410226). Repro
/// stub-dependencies/amsrefs_loads_mathscinet_url.
#[test]
fn amsrefs_loads_mathscinet_url() {
  let (log, xml) = convert_clean(repro!("amsrefs_loads_mathscinet_url"));
  assert_element(
    &xml,
    "bib-name",
    &[r#"role="author""#],
    r#"<bib-name role="author"><surname>Gelʹfand</surname><givenname>I.</givenname></bib-name>"#,
  );
  assert_element(
    &xml,
    "bib-note",
    &[r#"role="annotation""#],
    r#"<bib-note role="annotation"><ref class="ltx_nolink ltx_url" font="typewriter" href="http://x.org">http://x.org</ref></bib-note>"#,
  );
  assert!(
    xml.contains(r#"<XMTok font="fraktur" role="UNKNOWN">g</XMTok>"#),
    "{xml}"
  );
  for file in [
    "rkeyval.sty",
    "textcmds.sty",
    "mathscinet_sty.rs",
    "url_sty.rs",
  ] {
    assert!(log.contains(file), "{file} not loaded:\n{log}");
  }
}

/// amsrefs.sty:179, 213-218: under `lite` rkeyval loads (`\DefineSimpleKey`) but textcmds and mathscinet do not;
/// `{biblist}[\resetbiblist{99}]` (:569, :651) takes its settings (witnesses 2604.08274, 1904.00652). Repro
/// stub-dependencies/amsrefs_lite_rkeyval.
#[test]
fn amsrefs_lite_rkeyval() {
  let (log, xml) = convert_clean(repro!("amsrefs_lite_rkeyval"));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Cite <cite class="ltx_citemacro_cite">[<bibref bibrefs="A" separator="," yyseparator=","/>]</cite>.
No cprime.</p>"#,
  );
  assert_eq!(xml.matches("<bibentry ").count(), 1, "{xml}");
  assert!(log.contains("rkeyval.sty"), "{log}");
  assert!(!log.contains("mathscinet_sty.rs"), "{log}");
}

/// hyperref.sty:4532-4537: `colorlinks` loads color. The elsarticle binding loads hyperref where the real class does
/// not, so the author's `\usepackage[colorlinks=true]{hyperref}` is a repeat load whose new options go through
/// `\hypersetup` (witnesses 2609.24844, 2609.13110, 1711.02465). Repro
/// stub-dependencies/hyperref_colorlinks_after_class_preload.
#[test]
fn hyperref_colorlinks_after_class_preload() {
  let (_, xml) = convert_clean(repro!("hyperref_colorlinks_after_class_preload"));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r##"<p xml:id="p1.1">Some <text color="#FF0000" xml:id="p1.1.1">red</text> text.</p>"##,
  );
}

/// thm-restate.sty:26 `\RequirePackage{thmtools}`; thmtools requires thm-restate back, and the cycle ends silently,
/// as `\ver@thmtools.sty` ends it in LaTeX (witnesses 2412.02079, 2411.02602). Repro
/// stub-dependencies/thm_restate_loads_thmtools.
#[test]
fn thm_restate_loads_thmtools() {
  let (log, xml) = convert_clean(repro!("thm_restate_loads_thmtools"));
  assert_eq!(
    xml
      .matches(r#"<theorem class="ltx_theorem_theorem""#)
      .count(),
    1,
    "{xml}"
  );
  assert!(
    log.contains("Info:recursion:thm-restate.sty"),
    "the cycle's Info:\n{log}"
  );
}

/// elsarticle.cls:1252-1300: `3p` loads geometry, so the document's `\geometry` is defined (witnesses 2609.13883,
/// 2404.14059). Repro stub-dependencies/elsarticle_3p_loads_geometry.
#[test]
fn elsarticle_3p_loads_geometry() {
  let (_, xml) = convert_clean(repro!("elsarticle_3p_loads_geometry"));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// The class's one-column text width reaches geometry (1p 384pt, 3p 468pt); the two-column layouts keep the class
/// default (elsarticle.cls:1252-1300; OXIDIZED_DESIGN_DIVERGENCES #561).
#[test]
fn elsarticle_layouts_set_geometry_text_width() {
  for (options, width) in [
    ("1p", "384.0pt"),
    ("3p", "468.0pt"),
    ("5p", "345.0pt"),
    ("3p,twocolumn", "345.0pt"),
  ] {
    let tex = format!(
      "\\documentclass[{options}]{{elsarticle}}\n\\begin{{document}}\n\\makeatletter W\\the\\Gm@tw.\\makeatother\n\\end{{document}}\n"
    );
    let (_, xml) = convert_clean(&tex);
    assert!(xml.contains(&format!("W{width}.")), "{options}: {xml}");
  }
}

/// xyv2.tex:154 `\xyrequire{frame}`, and the frame support of the driver requires curve (xypdf-fr.tex:60), so a curved
/// arrow `\ar@/^/[r]` under `\usepackage{xypic}` draws its `\crvi` curve (witnesses 2604.10154, 1207.7025). Repro
/// stub-dependencies/xypic_curved_arrow.
#[test]
fn xypic_curved_arrow() {
  let (_, xml) = convert_clean(repro!("xypic_curved_arrow"));
  assert_eq!(xml.matches("<picture").count(), 1, "{xml}");
  assert_eq!(
    xml.matches(r#"<svg:path d="M 0 0 C 17.43"#).count(),
    1,
    "{xml}"
  );
}

/// revtex4-1.cls:2378, revtex4-2.cls:2399 `\sanitize@url` (`%`, `~`, `_` other), which JASA.cls, ujp.sty and cej.cls
/// call (witnesses 2006.04679, 1702.04290, 1510.07897). Repro stub-dependencies/revtex_sanitize_url.
#[test]
fn revtex_sanitize_url() {
  let (_, xml) = convert_clean(repro!("revtex_sanitize_url"));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Mail: <text font="typewriter" xml:id="p1.1.1">a_b%c</text></p>"#,
  );
}

/// PoS.cls loads color (v1.2.1 :174, v1.2.4 and v1.2.6 :162), so a PoS paper's `\textcolor` needs no package of its
/// own (witnesses 1904.06989, 1309.7883). Repro stub-dependencies/pos_loads_color.
#[test]
fn pos_loads_color() {
  let (_, xml) = convert_clean(repro!("pos_loads_color"));
  assert!(
    xml.contains(r##"Some <text color="#FF0000" xml:id="p1.1.1">red</text> text."##),
    "{xml}"
  );
}
