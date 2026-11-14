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
/// carry the box's width as `--ltx-box-width` (BM Q2.4); a plain minipage keeps its `width` attribute. The property
/// comes before varwidth's own `width:` (which then wins), and a unit lifted through nested boxes keeps the innermost
/// box's width (revQAB57 B1). In the HTML every box's width is that property (DG 2026-11-14), a rule's stays `width:`
/// (OD #1045).
#[test]
fn section_in_a_box_keeps_the_box_width() {
  let rows: [(&str, &str); 3] = [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/block-model/section_in_a_box_keeps_the_box_width.tex"
      ),
      "\n  <para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">Plain box: <inline-block class=\"ltx_minipage\" vattach=\"top\" width=\"85.4pt\" xml:id=\"p1.1.1\">\n        <p xml:id=\"p1.1.1.1\">Text in a plain minipage.</p>\n      </inline-block></p>\n  </para>\n  <subsection class=\"ltx_minipage\" cssstyle=\"--ltx-box-width:113.8pt\" xml:id=\"S0.SSx1\">\n    <title>Starred</title>\n    <para xml:id=\"S0.SSx1.p1\">\n      <p xml:id=\"S0.SSx1.p1.1\">Text A.</p>\n    </para>\n  </subsection>\n  <section class=\"ltx_parbox\" cssstyle=\"--ltx-box-width:142.3pt\" inlist=\"toc\" xml:id=\"S1\">\n    <tags>\n      <tag>1</tag>\n      <tag role=\"refnum\">1</tag>\n      <tag role=\"typerefnum\">§1</tag>\n    </tags>\n    <title><tag close=\" \">1</tag>In a parbox</title>\n    <para xml:id=\"S1.p1\">\n      <p xml:id=\"S1.p1.1\">Text B.</p>\n    </para>\n    <para xml:id=\"S1.p2\">\n      <p xml:id=\"S1.p2.1\"><inline-sectional-block class=\"ltx_minipage\" cssstyle=\"--ltx-box-width:170.7pt\" framed=\"rectangle\" xml:id=\"S1.p2.1.1\">\n          <section inlist=\"toc\" xml:id=\"S2\">\n            <tags>\n              <tag>2</tag>\n              <tag role=\"refnum\">2</tag>\n              <tag role=\"typerefnum\">§2</tag>\n            </tags>\n            <title><tag close=\" \">2</tag>Framed</title>\n            <para xml:id=\"S2.p1\">\n              <p xml:id=\"S2.p1.1\">F.</p>\n            </para>\n          </section>\n        </inline-sectional-block></p>\n    </para>\n  </section>\n</document>",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/block-model/section_in_varwidth_keeps_its_own_width.tex"
      ),
      "\n  <section class=\"ltx_minipage\" cssstyle=\"--ltx-box-width:63.4pt;width:max-content;max-width:227.6pt\" inlist=\"toc\" xml:id=\"S1\">\n    <tags>\n      <tag>1</tag>\n      <tag role=\"refnum\">1</tag>\n      <tag role=\"typerefnum\">§1</tag>\n    </tags>\n    <title><tag close=\" \">1</tag>In varwidth</title>\n    <para xml:id=\"S1.p1\">\n      <p xml:id=\"S1.p1.1\">Short.</p>\n    </para>\n    <para xml:id=\"S1.p2\">\n      <p xml:id=\"S1.p2.1\"><inline-sectional-block class=\"ltx_minipage\" cssstyle=\"--ltx-box-width:75.1pt;width:max-content;max-width:227.6pt\" framed=\"rectangle\" xml:id=\"S1.p2.1.1\">\n          <subsection xml:id=\"S1.SSx1\">\n            <title>Framed vw</title>\n            <para xml:id=\"S1.SSx1.p1\">\n              <p xml:id=\"S1.SSx1.p1.1\">Short.</p>\n            </para>\n          </subsection>\n        </inline-sectional-block></p>\n    </para>\n  </section>\n</document>",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/block-model/section_in_nested_boxes_keeps_the_inner_width.tex"
      ),
      "\n  <section class=\"ltx_minipage\" cssstyle=\"--ltx-box-width:85.4pt\" inlist=\"toc\" xml:id=\"S1\">\n    <tags>\n      <tag>1</tag>\n      <tag role=\"refnum\">1</tag>\n      <tag role=\"typerefnum\">§1</tag>\n    </tags>\n    <title><tag close=\" \">1</tag>Inner</title>\n    <para xml:id=\"S1.p1\">\n      <p xml:id=\"S1.p1.1\">Text inner.</p>\n    </para>\n    <para xml:id=\"S1.p2\">\n      <p xml:id=\"S1.p2.1\">Outer text.</p>\n    </para>\n  </section>\n</document>",
    ),
  ];
  for (tex, expected) in rows {
    let (log, xml) = convert_counted(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(body(&xml), expected, "{xml}");
  }
  let css = include_str!("../../../latexml_post/resources/CSS/LaTeXML.css");
  assert!(
    css.contains(concat!(
      ":where([style*=\"--ltx-box-width:\"]:not([style^=\"width:\"], [style*=\";width:\"], [style*=\"; width:\"])) {\n",
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
/// `\makebox`, the rotated box's outer (its height and depth the properties too, QAB57d), inner and `\parbox`, the rule (its ink) and the math `\hspace`'s `mspace`
/// (MathML's own width) at `width:`, the varwidth (its own `width:max-content` after the property), the wrapfigure, the
/// lifted section.
const BOX_WIDTH_STYLES: &[&str] = &[
  "--ltx-box-width:85.4pt;",
  "--ltx-box-width:56.9pt;",
  "--ltx-box-width:113.8pt;",
  "--ltx-box-width:6.9pt;--ltx-box-height:56.9pt;--ltx-box-depth:0.0pt;",
  "--ltx-box-width:56.9pt;transform:translate(-25.0pt,-25.0pt) rotate(-90deg) ;",
  "--ltx-box-width:56.9pt;",
  "width:85.4pt;height:2.0pt;--ltx-bg-color:black;display:inline-block;",
  "width:2.85em;",
  "--ltx-box-width:22.3pt;width:max-content;max-width:227.6pt;",
  "--ltx-box-width:30%;",
  "--ltx-box-width:113.8pt;",
];

/// A LaTeXML box's height and depth are `--ltx-box-height` and `--ltx-box-depth` (OD #1045, DG 2026-11-14), each only
/// where its attribute is set (a depth-only box from XML: no LaTeX constructor writes one); a `\raisebox{y}[h][d]`
/// keeps its `bottom:` shift, a rule's height and MathML's `mspace` stay as they were. `\parbox[t][3cm]` has no height
/// to carry (RED `parbox_and_minipage_height_dropped_red`).
#[test]
fn box_heights_ride_the_custom_properties() {
  // Every styled start tag of the page, its LaTeXML logo dropped.
  let styled = |html: &str| -> Vec<String> {
    let page = html.split("class=\"ltx_page_logo\"").next().unwrap_or(html);
    page
      .split('<')
      .skip(1)
      .filter_map(|rest| rest.split('>').next())
      .filter(|tag| tag.contains("style=\""))
      .map(|tag| format!("<{tag}>"))
      .collect()
  };
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/box_heights_ride_the_custom_properties.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let html = super::svg_nested_picture::post_html(&xml);
  assert_eq!(styled(&html), BOX_HEIGHT_TAGS, "{html}");

  let xml = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document xmlns=\"http://dlmf.nist.gov/LaTeXML\">",
    "<para xml:id=\"p1\"><p>A <text depth=\"3.0pt\">d</text> <text height=\"5.0pt\">h</text> ",
    "<text cssstyle=\"height:20pt\" depth=\"3.0pt\" height=\"10.0pt\">a</text></p></para></document>\n"
  );
  let html = super::svg_nested_picture::post_html(xml);
  assert_eq!(
    styled(&html),
    [
      "<span class=\"ltx_text\" style=\"--ltx-box-depth:3.0pt;\">",
      "<span class=\"ltx_text\" style=\"--ltx-box-height:5.0pt;\">",
      "<span class=\"ltx_text\" style=\"--ltx-box-height:10.0pt;--ltx-box-depth:3.0pt;height:20pt;\">",
    ],
    "{html}"
  );

  let css = include_str!("../../../latexml_post/resources/CSS/LaTeXML.css");
  for rule in [
    ":where([style*=\"--ltx-box-height:\"]:not([style^=\"height:\"], [style*=\";height:\"], [style*=\"; height:\"])) {\n  height: var(--ltx-box-height) !important;\n}",
    ":where([style*=\"--ltx-box-height:\"][style*=\"--ltx-box-depth:\"]:not([style^=\"height:\"], [style*=\";height:\"], [style*=\"; height:\"])) {\n  height: calc(var(--ltx-box-height) + var(--ltx-box-depth)) !important;\n}",
    ":where([style*=\"--ltx-box-depth:\"]:not([style^=\"vertical-align:\"], [style*=\";vertical-align:\"], [style*=\"; vertical-align:\"], [style*=\"; bottom:\"])) {\n  vertical-align: calc(-1 * var(--ltx-box-depth)) !important;\n}",
  ] {
    assert!(css.contains(rule), "LaTeXML.css lacks {rule}");
  }
}

/// The styled elements of box_heights_ride_the_custom_properties's HTML, in document order: `\raisebox{2pt}[10pt][3pt]`,
/// `\raisebox{0pt}[0pt][0pt]`, `\raisebox{3pt}[6pt]` (each its `bottom:` shift), `\raisebox{3pt}` (TeX's `\raise`), the
/// `\parbox[t][3cm]`, the rotated box's outer and inner, the text rule and the math rule (their ink), the `mspace`.
const BOX_HEIGHT_TAGS: &[&str] = &[
  r#"<span id="p1.1.1" class="ltx_text" style="--ltx-box-height:10.0pt;--ltx-box-depth:3.0pt;position:relative; bottom:2.0pt;">"#,
  r#"<span id="p1.1.2" class="ltx_text" style="--ltx-box-height:0.0pt;--ltx-box-depth:0.0pt;position:relative; bottom:0.0pt;">"#,
  r#"<span id="p1.1.3" class="ltx_text" style="--ltx-box-height:6.0pt;position:relative; bottom:3.0pt;">"#,
  r#"<span id="p1.1.4" class="ltx_text" style="vertical-align:3.0pt;">"#,
  r#"<span id="p2.1.1" class="ltx_inline-block ltx_parbox ltx_align_top" style="--ltx-box-width:113.8pt;">"#,
  r#"<span id="p2.1.2" class="ltx_inline-block ltx_transformed_outer" style="--ltx-box-width:6.9pt;--ltx-box-height:28.9pt;--ltx-box-depth:0.0pt;">"#,
  r#"<span class="ltx_transformed_inner" style="--ltx-box-width:28.9pt;transform:translate(-11.0pt,-11.0pt) rotate(-90deg) ;">"#,
  r#"<span class="ltx_rule" style="width:28.5pt;height:2.0pt;position:relative; bottom:-1.0pt;--ltx-bg-color:black;display:inline-block;">"#,
  r#"<span class="ltx_rule" style="width:0.0pt;height:12.9pt;position:relative; bottom:-4.3pt;--ltx-bg-color:black;display:inline-block;">"#,
  r#"<mspace style="width:2.85em;" width="2.85em">"#,
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
