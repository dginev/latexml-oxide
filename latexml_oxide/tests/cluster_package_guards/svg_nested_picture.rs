//! Batch 56ct: the SVG post-processor converts every `ltx:picture`, as
//! Perl's SVG.pm does. A picture NESTED in a foreign subtree — the
//! `\makebox(w,h)` inside a `\scalebox`ed box re-opens one — was
//! deep-cloned into the parent's `svg:foreignObject` and never converted
//! (the pre-collected original had been detached), so its text rendered
//! into an empty span (simplecd's jewel-case labels, sim-os-menus'
//! terminal text). The processor now iterates until no raw picture is
//! left.
use crate::cluster::convert_to_xml;

/// The HTML stage (where the SVG processor runs): the core XML through the
/// embedded html5 stylesheet, POST errors counted.
fn post_html(xml: &str) -> String {
  latexml_core::util::logger::bind_log();
  let opts = latexml::post::PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination:               None,
    source_directory:          Some("tests/cluster_regressions"),
    site_directory:            None,
    search_paths:              &[],
    nodefaultresources:        true,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    plane1:                    true,
    hackplane1:                false,
    mathtex:                   false,
    url_style:                 latexml_post::crossref::UrlStyle::File,
    navigationtoc:             None,
    schemadocs:                false,
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             false,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  };
  let out = latexml::post::run_post_processing(xml, &opts);
  let log = latexml_core::util::logger::flush_log();
  assert_eq!(
    latexml::util::test::error_count(&log),
    0,
    "POST errors:\n{log}"
  );
  out
}

/// The inner picture becomes an `svg` inside the outer picture's
/// `foreignObject`, its text intact (the whole inner `foreignObject`). Since
/// 56eq (#243) the `\parbox` in the picture `<g>` is a schema-valid
/// `<inline-block>` (an LR-box), so the inner foreignObject holds
/// `<span class="ltx_inline-block">` (was the schema-invalid `<div class="ltx_block">`);
/// the foreignObject dimensions (15.83×75.58) and the box width (56.9pt) are
/// unchanged (render-faithful).
#[test]
fn picture_nested_in_a_scaled_box_is_converted() {
  let xml = convert_to_xml("tests/cluster_regressions/picture_scaled_nested.tex");
  let html = post_html(&xml);
  latexml::util::test::assert_element(
    &html,
    "foreignObject",
    &[r#"width="75.58""#],
    r##"<foreignObject height="15.83" overflow="visible" width="75.58"><span class="ltx_foreignobject_container"><span class="ltx_foreignobject_content"><span class="ltx_inline-block ltx_parbox ltx_align_middle" style="width:56.9pt;"><span class="ltx_p ltx_align_center">SCALEDTEXTWORD box</span></span></span></span></foreignObject>"##,
  );
}
