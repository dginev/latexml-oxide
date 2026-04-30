//! Post-processing pipeline API.
//!
//! Provides a public interface to the LaTeXML post-processing pipeline
//! (Scan → Bibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups).
//! Used by both the `latexml_oxide` binary and the `cortex_worker` binary.

use latexml_post::document::{PostDocument, PostDocumentOptions};
use latexml_post::object_db::ObjectDB;
use latexml_post::processor::Processor;
use once_cell::sync::Lazy;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static POST_AUDIT: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_POST_AUDIT").is_ok());

/// Options for the post-processing pipeline.
pub struct PostOptions<'a> {
  pub pmml:                      bool,
  pub cmml:                      bool,
  pub keep_xmath:                bool,
  pub stylesheet:                Option<&'a str>,
  pub destination:               Option<&'a str>,
  pub source_directory:          Option<&'a str>,
  pub nodefaultresources:        bool,
  pub css_files:                 &'a [String],
  pub js_files:                  &'a [String],
  pub noinvisibletimes:          bool,
  pub mathtex:                   bool,
  pub navigationtoc:             Option<&'a str>,
  pub split:                     bool,
  pub split_xpath:               Option<String>,
  pub split_naming:              Option<&'a str>,
  pub xslt_parameters:           &'a [String],
  /// If > 0, try `inkscape` for PDF graphics smaller than this many KB
  /// (vector-preservation path). Fall back to ImageMagick `convert` on
  /// failure or timeout. Tracks upstream brucemiller/LaTeXML#902.
  pub graphics_svg_threshold_kb: u32,
}

/// Run the post-processing pipeline on XML output.
///
/// Executes: Scan → MakeBibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups.
pub fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  let PostOptions {
    pmml,
    cmml,
    keep_xmath,
    stylesheet,
    destination,
    source_directory,
    nodefaultresources,
    css_files,
    js_files,
    noinvisibletimes,
    mathtex,
    navigationtoc,
    split,
    ref split_xpath,
    split_naming,
    xslt_parameters,
    graphics_svg_threshold_kb,
  } = *opts;

  let mut doc_opts = PostDocumentOptions::default();
  if let Some(dest) = destination {
    doc_opts.destination = Some(dest.to_string());
  }
  if let Some(src_dir) = source_directory {
    doc_opts.source_directory = Some(src_dir.to_string());
    let mut sp = doc_opts.searchpaths.take().unwrap_or_default();
    sp.push(src_dir.to_string());
    doc_opts.searchpaths = Some(sp);
  }
  let audit = *POST_AUDIT;
  let audit_start = |name: &str| -> Option<(String, std::time::Instant)> {
    if audit {
      Some((name.to_string(), std::time::Instant::now()))
    } else {
      None
    }
  };
  let audit_end = |started: Option<(String, std::time::Instant)>| {
    if let Some((name, t0)) = started {
      let ms = t0.elapsed().as_millis();
      log::info!("POST_AUDIT phase {} took {}ms", name, ms);
    }
  };

  let t_parse = audit_start("PostDocument::new_from_string");
  let doc = match PostDocument::new_from_string(xml, doc_opts) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Post-processing: failed to parse XML: {}", e);
      return xml.to_string();
    },
  };
  audit_end(t_parse);

  // Phase 1: Scan
  let t_scan = audit_start("Scan");
  let db = ObjectDB::new();
  let mut scanner = latexml_post::scan::Scan::new(db);
  let scan_nodes = scanner.to_process(&doc);
  let doc = match scanner.process(doc, scan_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: Scan failed: {}", e);
      return xml.to_string();
    },
  };
  audit_end(t_scan);

  // Phase 1.5: MakeBibliography
  let t_bib = audit_start("MakeBibliography");
  let db = scanner.db;
  let mut bibmaker = latexml_post::make_bibliography::MakeBibliography::new(db, false);
  let bib_nodes = bibmaker.to_process(&doc);
  let doc = if !bib_nodes.is_empty() {
    match bibmaker.process(doc, bib_nodes) {
      Ok(mut docs) => docs.remove(0),
      Err(e) => {
        eprintln!("Post-processing: MakeBibliography failed: {}", e);
        return xml.to_string();
      },
    }
  } else {
    doc
  };
  audit_end(t_bib);

  // Phase 2: CrossRef
  let t_xref = audit_start("CrossRef");
  let db = bibmaker.db;
  let mut crossref =
    latexml_post::crossref::CrossRef::new(db, latexml_post::crossref::UrlStyle::File, true);
  let xref_nodes = crossref.to_process(&doc);
  let doc = match crossref.process(doc, xref_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: CrossRef failed: {}", e);
      return xml.to_string();
    },
  };
  audit_end(t_xref);

  // Phase 2.5: Graphics
  let t_gfx = audit_start("Graphics");
  let mut graphics_proc = latexml_post::graphics::Graphics::new(None, true)
    .with_svg_threshold_kb(graphics_svg_threshold_kb);
  let graphics_nodes = graphics_proc.to_process(&doc);
  let doc = if !graphics_nodes.is_empty() {
    match graphics_proc.process(doc, graphics_nodes) {
      Ok(mut docs) => docs.remove(0),
      Err(e) => {
        eprintln!("Post-processing: Graphics failed: {}", e);
        return xml.to_string();
      },
    }
  } else {
    doc
  };
  audit_end(t_gfx);

  // Phase 2.6: SVG (convert ltx:picture children to svg:svg + svg:* elements)
  // Without this, the XSLT picture template falls back to "as-TeX" mode and
  // emits an empty span — figures with picture-environment content are lost.
  // Phase 2.6: SVG
  // Convert ltx:picture elements to inline SVG for HTML output.
  //
  // The latexml_post::svg::SVG processor works correctly but causes a
  // use-after-free crash in libxml2 during PostDocument cleanup (nodes
  // unlinked by replace_node are freed but still referenced in the idcache).
  //
  // Workaround: extract SVG fragments from the INTERMEDIATE XML using
  // string processing (no libxml2 involvement), then inject them into
  // the final HTML AFTER XSLT completes.
  let t_svg = audit_start("SVG extraction");
  let svg_fragments = extract_svg_fragments(xml);
  audit_end(t_svg);

  // Phase 2.75: Split
  let doc = if split {
    if let Some(ref xpath) = split_xpath {
      let naming = match split_naming {
        Some("id") | None => latexml_post::split::SplitNaming::Id,
        Some("idrelative") => latexml_post::split::SplitNaming::IdRelative,
        Some("label") => latexml_post::split::SplitNaming::Label,
        Some("labelrelative") => latexml_post::split::SplitNaming::LabelRelative,
        Some(other) => {
          eprintln!("Unknown splitnaming '{}', using 'id'", other);
          latexml_post::split::SplitNaming::Id
        },
      };
      let mut splitter = latexml_post::split::Split::new(xpath, naming, false);
      let split_nodes = splitter.to_process(&doc);
      match splitter.process(doc, split_nodes) {
        Ok(mut docs) => {
          if docs.len() > 1 {
            eprintln!("Split into {} documents", docs.len());
          }
          docs.remove(0)
        },
        Err(e) => {
          eprintln!("Post-processing: Split failed: {}", e);
          return xml.to_string();
        },
      }
    } else {
      doc
    }
  } else {
    doc
  };

  // Phase 3: MathML + XSLT
  let mut post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  let intent_literal = xml.contains("package=\"ar5iv");

  if pmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_presentation()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes)
        .with_mathtex(mathtex)
        .with_intent_literal(intent_literal),
    ));
  }
  if cmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_content()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes),
    ));
  }
  if let Some(xsl_path) = stylesheet {
    let mut searchpaths = vec![".".to_string()];
    if let Ok(exe) = std::env::current_exe() {
      if let Some(project_root) = exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
      {
        searchpaths.insert(0, project_root.display().to_string());
      }
    }
    let mut xslt_params = std::collections::HashMap::new();
    if !css_files.is_empty() {
      xslt_params.insert("CSS".to_string(), format!("\"{}\"", css_files.join("|")));
    }
    if !js_files.is_empty() {
      xslt_params.insert(
        "JAVASCRIPT".to_string(),
        format!("\"{}\"", js_files.join("|")),
      );
    }
    if let Some(navtoc) = navigationtoc {
      xslt_params.insert("NAVIGATIONTOC".to_string(), format!("\"{}\"", navtoc));
    }
    for param in xslt_parameters {
      if let Some((key, value)) = param.split_once('=') {
        xslt_params.insert(key.to_string(), format!("\"{}\"", value));
      }
    }
    match latexml_post::xslt::XSLT::new(
      xsl_path,
      xslt_params,
      nodefaultresources,
      None,
      searchpaths,
    ) {
      Ok(xslt) => processors.push(Box::new(xslt)),
      Err(e) => eprintln!("Post-processing: XSLT error: {}", e),
    }
  }

  let t_chain = audit_start("process_chain");
  let chain_result = post.process_chain(doc, &mut processors);
  audit_end(t_chain);
  match chain_result {
    Ok(results) => {
      let t_serialize = audit_start("to_xml_string");
      let output = results[0].to_xml_string();
      audit_end(t_serialize);
      if stylesheet.is_some_and(|s| s.contains("html")) {
        // Strip <?xml version...?> prolog: HTML5 must NOT have an XML declaration.
        // libxml2's to_string() includes it by default; we strip it here.
        let output = regex::Regex::new(r"^<\?xml[^?]*\?>\s*")
          .unwrap()
          .replace(&output, "")
          .to_string();
        let re = regex::Regex::new(
          r"<(span|div|p|a|td|th|tr|section|article|figure|figcaption|pre|code|em|strong|b|i|u|sub|sup|small|cite)(\s[^>]*)?/>",
        )
        .unwrap();
        let output = re.replace_all(&output, "<$1$2></$1>").to_string();
        let void_close_re = regex::Regex::new(
          r"</(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)>",
        )
        .unwrap();
        let output = void_close_re.replace_all(&output, "").to_string();
        let void_selfclose_re = regex::Regex::new(
          r"<(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)(\s[^>]*?)\s*/>",
        )
        .unwrap();
        let mut output = void_selfclose_re.replace_all(&output, "<$1$2>").to_string();
        // Phase G: inject SVG fragments into empty ltx_picture spans
        if !svg_fragments.is_empty() {
          for (pic_id, svg_html) in &svg_fragments {
            // Replace <span id="ID" class="ltx_picture" style="..."></span>
            // with <span id="ID" class="ltx_picture" style="...">SVG_CONTENT</span>
            let pattern = format!(
              r#"<span id="{}" class="ltx_picture"([^>]*)></span>"#,
              regex::escape(pic_id)
            );
            if let Ok(re) = regex::Regex::new(&pattern) {
              output = re
                .replace(&output, |caps: &regex::Captures| {
                  format!(
                    r#"<span id="{}" class="ltx_picture"{}>{}</span>"#,
                    pic_id, &caps[1], svg_html
                  )
                })
                .to_string();
            }
          }
        }
        output
      } else {
        output
      }
    },
    Err(e) => {
      eprintln!("Post-processing failed: {}", e);
      xml.to_string()
    },
  }
}

/// Extract SVG fragments from intermediate LaTeXML XML.
///
/// Finds `<picture>` elements, converts their children to inline SVG HTML.
/// Uses a lightweight regex+string approach (no libxml2) to avoid the
/// use-after-free crash in PostDocument cleanup.
///
/// Returns (picture_id, svg_html) pairs for post-XSLT injection.
fn extract_svg_fragments(xml: &str) -> Vec<(String, String)> {
  let mut fragments = Vec::new();
  // Match <picture ... xml:id="ID" ... width="W" height="H" ...>CONTENT</picture>
  let picture_re = regex::Regex::new(r#"(?s)<picture([^>]*)>(.*?)</picture>"#).unwrap();
  let id_re = regex::Regex::new(r#"xml:id="([^"]+)""#).unwrap();
  let width_re = regex::Regex::new(r#"width="([^"]+)""#).unwrap();
  let height_re = regex::Regex::new(r#"height="([^"]+)""#).unwrap();

  for pic_caps in picture_re.captures_iter(xml) {
    let attrs = &pic_caps[1];
    let content = &pic_caps[2];
    let id = id_re
      .captures(attrs)
      .map(|c| c[1].to_string())
      .unwrap_or_default();
    let width = width_re.captures(attrs).and_then(|c| parse_tex_dim(&c[1]));
    let height = height_re.captures(attrs).and_then(|c| parse_tex_dim(&c[1]));

    if id.is_empty() || content.trim().is_empty() {
      continue;
    }

    let w = width.unwrap_or(100.0);
    let h = height.unwrap_or(100.0);

    // Build inline SVG: coordinate system has y-flip (TeX origin bottom-left, SVG top-left)
    let mut svg_content = format!(
      r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{w:.2}" height="{h:.2}" overflow="visible">"#,
    );
    svg_content.push_str(&format!(
      r#"<g transform="translate(0,{h:.2}) scale(1,-1)">"#,
    ));

    // Convert ltx picture children to SVG elements
    svg_content.push_str(&convert_picture_children_to_svg(content));

    svg_content.push_str("</g></svg>");
    fragments.push((id, svg_content));
  }
  fragments
}

/// Convert LaTeXML picture children (g, line, text, circle, etc.) to SVG elements.
fn convert_picture_children_to_svg(content: &str) -> String {
  use std::sync::LazyLock;
  static G_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<g([^>]*)>(.*?)</g>"#).unwrap());
  static TRANSFORM_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"transform="([^"]+)""#).unwrap());
  static LINE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<line\s+points="([^"]+)"([^/]*)/?>"#).unwrap());
  static CIRCLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<circle([^/]*)/?>"#).unwrap());
  static ELLIPSE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<ellipse([^/]*)/?>"#).unwrap());
  static RECT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<rect([^/]*)/?>"#).unwrap());
  static POLYGON_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<polygon([^/]*)/?>"#).unwrap());
  static PATH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<path([^/]*)/?>"#).unwrap());
  static BEZIER_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<bezier\s+points="([^"]+)"([^/]*)/?>"#).unwrap());
  static TEXT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<text([^>]*)>(.*?)</text>"#).unwrap());

  let mut svg = String::new();

  for g_caps in G_RE.captures_iter(content) {
    let g_attrs = &g_caps[1];
    let g_content = &g_caps[2];

    // Extract transform
    let transform = TRANSFORM_RE.captures(g_attrs).map(|c| c[1].to_string());

    if let Some(t) = &transform {
      svg.push_str(&format!(r#"<g transform="{t}">"#));
    } else {
      svg.push_str("<g>");
    }

    // <line points="x1,y1 x2,y2" stroke="..." stroke-width="..."/>
    for line_caps in LINE_RE.captures_iter(g_content) {
      let points = &line_caps[1];
      let rest_attrs = &line_caps[2];
      let coords: Vec<&str> = points.split_whitespace().collect();
      if coords.len() >= 2 {
        let p1: Vec<&str> = coords[0].split(',').collect();
        let p2: Vec<&str> = coords[1].split(',').collect();
        if p1.len() == 2 && p2.len() == 2 {
          svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}"{}/>"#,
            p1[0], p1[1], p2[0], p2[1], rest_attrs
          ));
        }
      }
    }

    // <circle cx="..." cy="..." r="..." .../>
    for circle_caps in CIRCLE_RE.captures_iter(g_content) {
      svg.push_str(&format!("<circle{}/>", &circle_caps[1]));
    }

    // <ellipse cx="..." cy="..." rx="..." ry="..." .../>
    for ellipse_caps in ELLIPSE_RE.captures_iter(g_content) {
      svg.push_str(&format!("<ellipse{}/>", &ellipse_caps[1]));
    }

    // <rect x="..." y="..." width="..." height="..." .../>
    for rect_caps in RECT_RE.captures_iter(g_content) {
      svg.push_str(&format!("<rect{}/>", &rect_caps[1]));
    }

    // <polygon points="..." .../>
    for polygon_caps in POLYGON_RE.captures_iter(g_content) {
      svg.push_str(&format!("<polygon{}/>", &polygon_caps[1]));
    }

    // <path d="..." .../>
    for path_caps in PATH_RE.captures_iter(g_content) {
      svg.push_str(&format!("<path{}/>", &path_caps[1]));
    }

    // <bezier points="x1,y1 x2,y2 x3,y3 x4,y4" .../>
    // Convert to SVG cubic bezier path
    for bez_caps in BEZIER_RE.captures_iter(g_content) {
      let points = &bez_caps[1];
      let rest = &bez_caps[2];
      let coords: Vec<&str> = points.split_whitespace().collect();
      if coords.len() >= 4 {
        // SVG cubic bezier: M x0,y0 C x1,y1 x2,y2 x3,y3
        let d = format!(
          "M {} C {} {} {}",
          coords[0], coords[1], coords[2], coords[3]
        );
        svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
      } else if coords.len() >= 3 {
        // Quadratic bezier: M x0,y0 Q x1,y1 x2,y2
        let d = format!("M {} Q {} {}", coords[0], coords[1], coords[2]);
        svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
      }
    }

    // <arc .../> — arc segments (rarely used, stub for now)
    // <wedge .../> — filled wedges (rarely used, stub for now)

    // <text>...</text> — wrap in SVG text with y-flip correction
    for text_caps in TEXT_RE.captures_iter(g_content) {
      let text_attrs = &text_caps[1];
      let text_content = &text_caps[2];
      svg.push_str(&format!(
        r#"<g transform="scale(1,-1)"><text{text_attrs}>{text_content}</text></g>"#,
      ));
    }

    svg.push_str("</g>");
  }

  // Also handle direct children not inside <g> (e.g. top-level <bezier>, <line>)
  // These appear directly inside <picture> without a <g> wrapper
  let direct_bezier_re =
    regex::Regex::new(r#"(?m)^\s*<bezier\s+points="([^"]+)"([^/]*)/?>"#).unwrap();
  for bez_caps in direct_bezier_re.captures_iter(content) {
    let points = &bez_caps[1];
    let rest = &bez_caps[2];
    let coords: Vec<&str> = points.split_whitespace().collect();
    if coords.len() >= 4 {
      let d = format!(
        "M {} C {} {} {}",
        coords[0], coords[1], coords[2], coords[3]
      );
      svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
    } else if coords.len() >= 3 {
      let d = format!("M {} Q {} {}", coords[0], coords[1], coords[2]);
      svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
    }
  }

  svg
}

/// Parse a TeX dimension string (e.g. "100.0pt") to pixels.
fn parse_tex_dim(s: &str) -> Option<f64> {
  let s = s.trim();
  if let Some(rest) = s.strip_suffix("pt") {
    rest.parse::<f64>().ok().map(|v| v * 96.0 / 72.27)
  } else if let Some(rest) = s.strip_suffix("px") {
    rest.parse::<f64>().ok()
  } else {
    s.parse::<f64>().ok()
  }
}
