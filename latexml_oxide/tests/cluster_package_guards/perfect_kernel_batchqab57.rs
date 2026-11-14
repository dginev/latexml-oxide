//! Guards for batch QAB57 (three Bruce Q&A residuals): under OmniBus the document's definitions win over its guesses
//! (QA58); a section lifted out of a box keeps the box's width as `--ltx-box-width` (QA24); latexml.sty's
//! `pdfoutput`/`nopdfoutput` options override the guessed output mode (QA64).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The body of `<document …>`, from its first child on, with the stylesheet resources dropped.
fn body(xml: &str) -> String {
  let start = xml.find("<document").unwrap_or(0);
  let rest = &xml[start..];
  let rest = &rest[rest.find('>').map_or(0, |i| i + 1)..];
  rest
    .lines()
    .filter(|l| !l.trim_start().starts_with("<resource "))
    .collect::<Vec<_>>()
    .join("\n")
}

/// Under OmniBus (a class with no binding) the document's own definitions of its locked frontmatter guesses win, every
/// definer of the `\newcommand` family and `\newenvironment` (BM Q3.12, DG 2026-11-14); a package's begin-document hook
/// is not the document, and apacite's `\providecommand{\doi}` keeps OmniBus's link (user ruling 2026-10-01).
#[test]
fn omnibus_guesses_yield_to_the_document() {
  let rows: [(&str, &str); 2] = [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/singletons/omnibus_document_redefines_frontmatter_guesses.tex"
      ),
      "\n  <para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">X E:a@b K:k A:ad O:0 D:10 F:u.\nK:k.</p>\n  </para>\n</document>",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/singletons/omnibus_package_hook_keeps_guessed_doi.tex"
      ),
      "\n  <para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">X <ref href=\"https://doi.org/10.1000/xyz\">10.1000/xyz</ref>.</p>\n  </para>\n</document>",
    ),
  ];
  for (tex, expected) in rows {
    let (log, xml) = convert_counted(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert_eq!(body(&xml), expected, "{xml}");
  }
}

/// A sectioning unit lifted out of a minipage or `\parbox` (OD #250), and a framed box's `inline-sectional-block`,
/// carry the box's width as `--ltx-box-width` (BM Q2.4); a plain minipage keeps its `width` attribute. In the HTML every
/// box's width is that property (DG 2026-11-14), a rule's stays `width:` (OD #1045).
#[test]
fn section_in_a_box_keeps_the_box_width() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/section_in_a_box_keeps_the_box_width.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for open in [
    r#"<inline-block class="ltx_minipage" vattach="top" width="85.4pt" xml:id="p1.1.1">"#,
    r#"<subsection class="ltx_minipage" cssstyle="--ltx-box-width:113.8pt" xml:id="S0.SSx1">"#,
    r#"<section class="ltx_parbox" cssstyle="--ltx-box-width:142.3pt" inlist="toc" xml:id="S1">"#,
    r#"<inline-sectional-block class="ltx_minipage" cssstyle="--ltx-box-width:170.7pt" framed="rectangle" xml:id="S1.p2.1.1">"#,
    r#"<section inlist="toc" xml:id="S2">"#,
  ] {
    assert_eq!(xml.matches(open).count(), 1, "{open}\n{xml}");
  }
  assert_eq!(xml.matches("--ltx-box-width").count(), 3, "{xml}");
  let css = include_str!("../../../latexml_post/resources/CSS/LaTeXML.css");
  assert!(
    css.contains(concat!(
      ":where([style*=\"--ltx-box-width:\"]:not([style*=\";width:\"], [style*=\"; width:\"])) {\n",
      "  width: var(--ltx-box-width, auto) !important;\n}"
    )),
    "LaTeXML.css applies --ltx-box-width"
  );

  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/box_widths_ride_the_custom_property.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let html = super::svg_nested_picture::post_html(&xml);
  let styles: Vec<&str> = html
    .split("style=\"")
    .skip(1)
    .filter_map(|rest| rest.split('"').next())
    .filter(|style| style.contains("width"))
    .collect();
  assert_eq!(styles, BOX_WIDTH_STYLES, "{html}");
}

/// The width styles of box_widths_ride_the_custom_property's HTML, in document order: the minipage, the `\parbox`, the
/// `\makebox`, the rotated box's outer, inner and `\parbox`, the rule (its ink) and the math `\hspace`'s `mspace`
/// (MathML's own width) at `width:`, the varwidth (its own `width:max-content` after the property), the wrapfigure, the
/// lifted section.
const BOX_WIDTH_STYLES: &[&str] = &[
  "--ltx-box-width:85.4pt;",
  "--ltx-box-width:56.9pt;",
  "--ltx-box-width:113.8pt;",
  "--ltx-box-width:6.9pt;height:56.9pt;vertical-align:-0.0pt;",
  "--ltx-box-width:56.9pt;transform:translate(-25.0pt,-25.0pt) rotate(-90deg) ;",
  "--ltx-box-width:56.9pt;",
  "width:85.4pt;height:2.0pt;--ltx-bg-color:black;display:inline-block;",
  "width:2.85em;",
  "--ltx-box-width:22.3pt;width:max-content;max-width:227.6pt;",
  "--ltx-box-width:30%;",
  "--ltx-box-width:113.8pt;",
];

/// latexml.sty's `nopdfoutput`/`pdfoutput` options set the output mode over the guess from the source (OD #1046; BM
/// Q4.2). The drivers read `\pdfoutput` as they load: given after graphicx, the driver graphicx chose stays; preloaded
/// (`[nopdfoutput]ar5iv.sty`, passed on to latexml.sty) it decides before the document starts.
#[test]
fn latexml_pdfoutput_options_override_the_mode() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/latexml_nopdfoutput_option.tex"
  );
  let option = "\\usepackage[nopdfoutput]{latexml}\n";
  let after_graphicx = tex.replace(option, "").replace(
    "\\usepackage{ifpdf}\n",
    &format!("\\usepackage{{ifpdf}}\n{option}"),
  );
  let no_option = tex.replace(option, "");
  for (source, preload, mode) in [
    (tex.to_string(), None, "Mode: DVI, 0, dvips.def."),
    (after_graphicx, None, "Mode: DVI, 0, pdftex.def."),
    (
      no_option.clone(),
      Some("[nopdfoutput]ar5iv.sty"),
      "Mode: DVI, 0, dvips.def.",
    ),
    (no_option, Some("ar5iv.sty"), "Mode: PDF, 1, pdftex.def."),
  ] {
    let (log, xml) = latexml::util::test::convert_with(&source, preload);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(xml.contains(&format!(">{mode}</p>")), "{mode}\n{xml}");
  }
}
