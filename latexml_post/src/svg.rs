//! SVG rendering processor.
//!
//! Port of `LaTeXML::Post::SVG` (522 lines of Perl).
//! Converts `ltx:picture` elements to SVG by traversing LaTeXML picture
//! primitives (g, path, line, rect, circle, ellipse, polygon, bezier,
//! arc, wedge, text, dots) and generating corresponding SVG elements.
//!
//! The coordinate system is mirrored: LaTeXML uses a bottom-left origin
//! with y increasing upward; SVG uses top-left with y increasing downward.
//! The top-level picture gets `transform="translate(0,h) scale(1,-1)"`.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::f64::consts::PI;

use crate::document::{NodeData, PostDocument, element_children_iter};
use crate::processor::{ProcessResult, Processor};

const SVG_URI: &str = "http://www.w3.org/2000/svg";
const DPI: f64 = 96.0;

/// SVG post-processor.
///
/// Port of `LaTeXML::Post::SVG`.
pub struct SVG {
  name: String,
}

impl Default for SVG {
  fn default() -> Self { Self::new() }
}

impl SVG {
  pub fn new() -> Self { SVG { name: "SVG".to_string() } }

  /// Convert a single ltx:picture node to SVG.
  ///
  /// Port of `SVG::ProcessSVG`.
  fn process_svg(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let h = node
      .get_attribute("height")
      .map(|s| to_px(&s))
      .unwrap_or(0.0);

    // Build a group with the y-flip transform
    let children: Vec<NodeData> = element_children_iter(node)
      .filter_map(|child| self.convert_node(doc, &child))
      .collect();

    let g_transform = format!("translate(0,{:.2}) scale(1,-1)", h);
    let g = NodeData::Element {
      tag: "svg:g".to_string(),
      attributes: Some(HashMap::from_iter([("transform".to_string(), g_transform)])),
      children,
    };

    // Build the outer svg:svg element
    let width = node.get_attribute("width").map(|s| to_px(&s));
    let height = node.get_attribute("height").map(|s| to_px(&s));
    let clip = node.get_attribute("clip").as_deref() == Some("true");

    let mut svg_attrs = HashMap::default();
    svg_attrs.insert("version".to_string(), "1.1".to_string());
    if let Some(w) = width {
      svg_attrs.insert("width".to_string(), format!("{:.2}", w));
    }
    if let Some(h) = height {
      svg_attrs.insert("height".to_string(), format!("{:.2}", h));
    }
    if !clip {
      svg_attrs.insert("overflow".to_string(), "visible".to_string());
    }

    Some(NodeData::Element {
      tag:        "svg:svg".to_string(),
      attributes: Some(svg_attrs),
      children:   vec![g],
    })
  }

  /// Dispatch conversion of a single element.
  ///
  /// Port of `convertNode` + converter dispatch table.
  fn convert_node(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let tag = doc.get_qname(node)?;
    match tag.as_str() {
      "ltx:picture" => self.convert_picture(doc, node),
      "ltx:g" => self.convert_g(doc, node),
      "ltx:path" => self.convert_path(doc, node),
      "ltx:line" => self.convert_line(doc, node),
      "ltx:polygon" => self.convert_polygon(doc, node),
      "ltx:rect" => self.convert_rect(doc, node),
      "ltx:circle" => self.convert_circle(doc, node),
      "ltx:ellipse" => self.convert_ellipse(doc, node),
      "ltx:bezier" => self.convert_bezier(doc, node),
      "ltx:arc" => self.convert_arc(doc, node),
      "ltx:wedge" => self.convert_wedge(doc, node),
      "ltx:dots" => self.convert_dots(doc, node),
      "ltx:text" => self.convert_text(doc, node),
      _ => {
        // Foreign element: wrap in svg:foreignObject
        self.convert_foreign(doc, node)
      },
    }
  }

  fn convert_picture(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let h = node
      .get_attribute("height")
      .map(|s| to_px(&s))
      .unwrap_or(0.0);
    let children = self.convert_children(doc, node);
    Some(NodeData::Element {
      tag: "svg:g".to_string(),
      attributes: Some(HashMap::from_iter([(
        "transform".to_string(),
        format!("translate(0,{:.2}) scale(1,-1)", h),
      )])),
      children,
    })
  }

  fn convert_g(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let mut attrs = self.copy_valid_attrs(node);
    // Check for framed+fillframe
    if node.get_attribute("framed").as_deref() == Some("true")
      && node.get_attribute("fillframe").as_deref() == Some("true")
    {
      let fill = node
        .get_attribute("fill")
        .unwrap_or_else(|| "white".to_string());
      attrs.insert(
        "filter".to_string(),
        format!("url(#bg{})", fill.replace('#', "")),
      );
    }
    let children = self.convert_children(doc, node);
    Some(NodeData::Element {
      tag: "svg:g".to_string(),
      attributes: if attrs.is_empty() { None } else { Some(attrs) },
      children,
    })
  }

  fn convert_path(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let attrs = self.copy_valid_attrs(node);
    let children = self.convert_children(doc, node);
    Some(NodeData::Element {
      tag: "svg:path".to_string(),
      attributes: if attrs.is_empty() { None } else { Some(attrs) },
      children,
    })
  }

  fn convert_line(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let mut attrs = self.copy_valid_attrs(node);
    if let Some(points) = node.get_attribute("points") {
      attrs.insert("d".to_string(), format!("M {}", points));
    }
    let children = self.convert_children(doc, node);
    Some(NodeData::Element {
      tag: "svg:path".to_string(),
      attributes: Some(attrs),
      children,
    })
  }

  fn convert_polygon(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let mut attrs = self.copy_valid_attrs(node);
    if let Some(points) = node.get_attribute("points") {
      attrs.insert("d".to_string(), format!("M {} z", points));
    }
    let children = self.convert_children(doc, node);
    Some(NodeData::Element {
      tag: "svg:path".to_string(),
      attributes: Some(attrs),
      children,
    })
  }

  fn convert_rect(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let attrs = self.copy_valid_attrs(node);
    if let Some(part) = node.get_attribute("part") {
      // Partial rect (rounded corner subset) → path
      let x = parse_dim(&node.get_attribute("x").unwrap_or_default());
      let y = parse_dim(&node.get_attribute("y").unwrap_or_default());
      let w = parse_dim(&node.get_attribute("width").unwrap_or_default());
      let h = parse_dim(&node.get_attribute("height").unwrap_or_default());
      let rx = parse_dim(&node.get_attribute("rx").unwrap_or_default());
      let d = oval_path(&part, x, y, w, h, rx);
      let mut path_attrs = attrs;
      path_attrs.insert("d".to_string(), d);
      Some(NodeData::Element {
        tag:        "svg:path".to_string(),
        attributes: Some(path_attrs),
        children:   self.convert_children(doc, node),
      })
    } else {
      Some(NodeData::Element {
        tag:        "svg:rect".to_string(),
        attributes: if attrs.is_empty() { None } else { Some(attrs) },
        children:   self.convert_children(doc, node),
      })
    }
  }

  fn convert_circle(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let mut attrs = self.copy_valid_attrs(node);
    if let Some(x) = node.get_attribute("x") {
      attrs.insert("cx".to_string(), x);
    }
    if let Some(y) = node.get_attribute("y") {
      attrs.insert("cy".to_string(), y);
    }
    Some(NodeData::Element {
      tag:        "svg:circle".to_string(),
      attributes: Some(attrs),
      children:   self.convert_children(doc, node),
    })
  }

  fn convert_ellipse(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let mut attrs = self.copy_valid_attrs(node);
    if let Some(x) = node.get_attribute("x") {
      attrs.insert("cx".to_string(), x);
    }
    if let Some(y) = node.get_attribute("y") {
      attrs.insert("cy".to_string(), y);
    }
    Some(NodeData::Element {
      tag:        "svg:ellipse".to_string(),
      attributes: Some(attrs),
      children:   self.convert_children(doc, node),
    })
  }

  fn convert_bezier(&self, doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let mut attrs = self.copy_valid_attrs(node);
    if let Some(points) = node.get_attribute("points") {
      let coords: Vec<f64> = explode_coord(&points);
      let n = coords.len() / 2;
      if n >= 2 {
        let x0 = coords[0];
        let y0 = coords[1];
        let cmd = match n {
          4 => "C",
          3 => "Q",
          _ => "T",
        };
        let rest: String = coords[2..]
          .chunks(2)
          .map(|p| format!("{:.2},{:.2}", p[0], p.get(1).unwrap_or(&0.0)))
          .collect::<Vec<_>>()
          .join(" ");
        attrs.insert(
          "d".to_string(),
          format!("M {:.2},{:.2} {} {}", x0, y0, cmd, rest),
        );
      }
    }
    if node.get_attribute("displayedpoints").is_some() {
      attrs.insert("stroke-dasharray".to_string(), "2".to_string());
    }
    Some(NodeData::Element {
      tag:        "svg:path".to_string(),
      attributes: Some(attrs),
      children:   self.convert_children(doc, node),
    })
  }

  fn convert_arc(&self, _doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let x = parse_dim(&node.get_attribute("x").unwrap_or_default());
    let y = parse_dim(&node.get_attribute("y").unwrap_or_default());
    let r = parse_dim(&node.get_attribute("r").unwrap_or_default());
    let a1 = parse_dim(&node.get_attribute("angle1").unwrap_or_default());
    let a2 = parse_dim(&node.get_attribute("angle2").unwrap_or_default());

    let mut bb = a2 - a1;
    if bb < 0.0 {
      bb += 360.0;
    }
    let large_arc = if bb > 180.0 { 1 } else { 0 };

    let a1r = a1 * PI / 180.0;
    let a2r = a2 * PI / 180.0;
    let x1 = x + r * a1r.cos();
    let y1 = y + r * a1r.sin();
    let x2 = x + r * a2r.cos();
    let y2 = y + r * a2r.sin();

    let d = format!(
      "M {:.2} {:.2} A {:.2} {:.2} 0 {} 1 {:.2} {:.2}",
      x1, y1, r, r, large_arc, x2, y2
    );

    Some(NodeData::Element {
      tag:        "svg:path".to_string(),
      attributes: Some(HashMap::from_iter([("d".to_string(), d)])),
      children:   vec![],
    })
  }

  fn convert_wedge(&self, _doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let x = parse_dim(&node.get_attribute("x").unwrap_or_default());
    let y = parse_dim(&node.get_attribute("y").unwrap_or_default());
    let r = parse_dim(&node.get_attribute("r").unwrap_or_default());
    let a1 = parse_dim(&node.get_attribute("angle1").unwrap_or_default());
    let a2 = parse_dim(&node.get_attribute("angle2").unwrap_or_default());

    let mut bb = a2 - a1;
    if bb < 0.0 {
      bb += 360.0;
    }
    let large_arc = if bb > 180.0 { 1 } else { 0 };

    let a1r = a1 * PI / 180.0;
    let a2r = a2 * PI / 180.0;
    let x1 = x + r * a1r.cos();
    let y1 = y + r * a1r.sin();
    let x2 = x + r * a2r.cos();
    let y2 = y + r * a2r.sin();

    let d = format!(
      "M {:.2} {:.2} L {:.2} {:.2} A {:.2} {:.2} 0 {} 1 {:.2} {:.2} z",
      x, y, x1, y1, r, r, large_arc, x2, y2
    );

    let attrs = self.copy_valid_attrs(node);
    let mut all_attrs = attrs;
    all_attrs.insert("d".to_string(), d);

    Some(NodeData::Element {
      tag:        "svg:path".to_string(),
      attributes: Some(all_attrs),
      children:   vec![],
    })
  }

  fn convert_dots(&self, _doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let points = node.get_attribute("points").unwrap_or_default();
    let coords = explode_coord(&points);
    let dotsize = node
      .get_attribute("dotsize")
      .unwrap_or_else(|| "2".to_string());

    let mut circles = Vec::new();
    for chunk in coords.chunks(2) {
      if chunk.len() == 2 {
        circles.push(NodeData::Element {
          tag:        "svg:circle".to_string(),
          attributes: Some(HashMap::from_iter([
            ("cx".to_string(), format!("{:.2}", chunk[0])),
            ("cy".to_string(), format!("{:.2}", chunk[1])),
            ("r".to_string(), dotsize.clone()),
          ])),
          children:   vec![],
        });
      }
    }

    Some(NodeData::Element {
      tag:        "svg:g".to_string(),
      attributes: None,
      children:   circles,
    })
  }

  fn convert_text(&self, _doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let x = node.get_attribute("x").unwrap_or_else(|| "0".to_string());
    let y = node.get_attribute("y").unwrap_or_else(|| "0".to_string());
    let text = node.get_content();

    let mut attrs = HashMap::default();
    attrs.insert("x".to_string(), x);
    attrs.insert("y".to_string(), y);
    // Text needs to be un-flipped
    attrs.insert("transform".to_string(), "scale(1,-1)".to_string());

    // Font attributes
    if let Some(fontsize) = node.get_attribute("fontsize") {
      attrs.insert("font-size".to_string(), fontsize);
    }
    if let Some(font) = node.get_attribute("font") {
      if font.contains("italic") {
        attrs.insert("font-style".to_string(), "italic".to_string());
      } else if font.contains("slanted") {
        attrs.insert("font-style".to_string(), "oblique".to_string());
      } else if font.contains("bold") {
        attrs.insert("font-weight".to_string(), "bold".to_string());
      } else if font.contains("smallcaps") {
        attrs.insert("font-variant".to_string(), "small-caps".to_string());
      }
    }
    if let Some(fill) = node.get_attribute("fill") {
      attrs.insert("fill".to_string(), fill);
    }

    Some(NodeData::Element {
      tag:        "svg:text".to_string(),
      attributes: Some(attrs),
      children:   vec![NodeData::Text(text)],
    })
  }

  /// Wrap foreign (non-picture) elements in svg:foreignObject.
  fn convert_foreign(&self, _doc: &PostDocument, node: &Node) -> Option<NodeData> {
    let width = node
      .get_attribute("width")
      .or_else(|| node.get_attribute("imagewidth"))
      .unwrap_or_else(|| "1pt".to_string());
    let height = node
      .get_attribute("height")
      .or_else(|| node.get_attribute("imageheight"))
      .unwrap_or_else(|| "1pt".to_string());
    let depth = node
      .get_attribute("depth")
      .unwrap_or_else(|| "0pt".to_string());

    let h_px = to_px(&height);
    let d_px = to_px(&depth);
    let y = h_px + d_px;

    let fo = NodeData::Element {
      tag:        "svg:foreignObject".to_string(),
      attributes: Some(HashMap::from_iter([
        ("width".to_string(), format!("{:.2}", to_px(&width))),
        ("height".to_string(), format!("{:.2}", h_px)),
        ("overflow".to_string(), "visible".to_string()),
      ])),
      children:   vec![NodeData::XmlNode(node.clone())],
    };

    Some(NodeData::Element {
      tag:        "svg:g".to_string(),
      attributes: Some(HashMap::from_iter([(
        "transform".to_string(),
        format!("translate(0,{:.2}) scale(1,-1)", y),
      )])),
      children:   vec![fo],
    })
  }

  /// Convert all element children of a node.
  fn convert_children(&self, doc: &PostDocument, node: &Node) -> Vec<NodeData> {
    element_children_iter(node)
      .filter_map(|child| self.convert_node(doc, &child))
      .collect()
  }

  /// Copy valid SVG attributes from a LaTeXML node.
  fn copy_valid_attrs(&self, node: &Node) -> HashMap<String, String> {
    let mut attrs = HashMap::default();
    let props = node.get_properties();
    for (key, value) in &props {
      match key.as_str() {
        "d" | "r" | "rx" | "ry" | "x" | "y" | "width" | "height" | "cx" | "cy" | "x1" | "y1"
        | "x2" | "y2" | "fill" | "stroke" | "stroke-width" | "stroke-dasharray"
        | "stroke-linecap" | "stroke-linejoin" | "opacity" | "fill-opacity" | "stroke-opacity"
        | "transform" | "style" | "class" => {
          attrs.insert(key.clone(), value.clone());
        },
        "xml:id" => {},                // Skip: IDs handled separately
        k if k.starts_with('_') => {}, // Skip internal attributes
        _ => {},
      }
    }
    attrs
  }
}

impl Processor for SVG {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:picture[child::*[not(local-name()='svg')]]")
  }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    if !nodes.is_empty() {
      doc.add_namespace("svg", SVG_URI);
    }
    for node in &nodes {
      if let Some(svg) = self.process_svg(&doc, node) {
        let node_mut = node.clone();
        doc.replace_node(&node_mut, &[svg]);
      }
    }
    Ok(vec![doc])
  }
}

// ======================================================================
// Utility functions

/// Convert a TeX dimension string to SVG pixels.
///
/// Port of `to_px`.
fn to_px(s: &str) -> f64 {
  let trimmed = s.trim();
  if let Some(pt) = trimmed.strip_suffix("pt") {
    pt.trim().parse::<f64>().unwrap_or(0.0) * DPI / 72.27
  } else if let Some(px) = trimmed.strip_suffix("px") {
    px.trim().parse::<f64>().unwrap_or(0.0)
  } else if let Some(em) = trimmed.strip_suffix("em") {
    em.trim().parse::<f64>().unwrap_or(0.0) * 10.0 // rough
  } else {
    trimmed.parse::<f64>().unwrap_or(0.0)
  }
}

/// Parse a dimension string to a float.
fn parse_dim(s: &str) -> f64 {
  let trimmed = s.trim();
  let num: String = trimmed
    .chars()
    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
    .collect();
  num.parse::<f64>().unwrap_or(0.0)
}

/// Explode a space/comma-separated coordinate string into floats.
fn explode_coord(s: &str) -> Vec<f64> {
  s.split([' ', ','])
    .filter(|s| !s.is_empty())
    .filter_map(|s| s.trim().parse::<f64>().ok())
    .collect()
}

/// Generate an SVG path for a partial rounded rectangle.
///
/// Port of `ovalPath`.
fn oval_path(part: &str, x: f64, y: f64, w: f64, h: f64, r: f64) -> String {
  match part {
    "t" => format!(
      "M {} {} L {} {} A {} {} 0 0 1 {} {} L {} {} A {} {} 0 0 1 {} {} L {} {}",
      x,
      y - h / 2.0,
      x,
      y - r,
      r,
      r,
      x + r,
      y,
      x + w - r,
      y,
      r,
      r,
      x + w,
      y - r,
      x + w,
      y - h / 2.0
    ),
    "b" => format!(
      "M {} {} L {} {} A {} {} 0 0 1 {} {} L {} {} A {} {} 0 0 1 {} {} L {} {}",
      x + w,
      y - h / 2.0,
      x + w,
      y - h + r,
      r,
      r,
      x + w - r,
      y - h,
      x + r,
      y - h,
      r,
      r,
      x,
      y - h + r,
      x,
      y - h / 2.0
    ),
    _ => format!("M {} {} L {} {}", x, y, x + w, y), // fallback
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn approx(a: f64, b: f64) -> bool { (a - b).abs() < 1e-6 }

  #[test]
  fn to_px_pt_scales_by_dpi_over_72_27() {
    // 72.27pt = 1 inch = DPI pixels.
    assert!(approx(to_px("72.27pt"), DPI));
    assert!(approx(to_px("0pt"), 0.0));
  }

  #[test]
  fn to_px_px_is_identity() {
    assert!(approx(to_px("42px"), 42.0));
    assert!(approx(to_px("  3.5px "), 3.5));
  }

  #[test]
  fn to_px_em_rough_x10() {
    // Rough approximation per the code comment; lock it in.
    assert!(approx(to_px("2em"), 20.0));
  }

  #[test]
  fn to_px_bare_number_is_f64() {
    assert!(approx(to_px("7"), 7.0));
    assert!(approx(to_px("  -1.5 "), -1.5));
  }

  #[test]
  fn to_px_unknown_unit_is_zero() {
    // No known suffix and no leading number → parse fails → 0.0.
    assert!(approx(to_px("xyz"), 0.0));
  }

  #[test]
  fn parse_dim_takes_leading_numeric_prefix() {
    assert!(approx(parse_dim("12.5pt"), 12.5));
    assert!(approx(parse_dim("-3.0em"), -3.0));
    assert!(approx(parse_dim("+7px"), 7.0));
  }

  #[test]
  fn parse_dim_no_leading_number_is_zero() {
    assert!(approx(parse_dim("pt"), 0.0));
    assert!(approx(parse_dim(""), 0.0));
  }

  #[test]
  fn explode_coord_splits_on_space_and_comma() {
    assert_eq!(explode_coord("1 2 3"), vec![1.0, 2.0, 3.0]);
    assert_eq!(explode_coord("1,2,3"), vec![1.0, 2.0, 3.0]);
    assert_eq!(explode_coord("1, 2 3"), vec![1.0, 2.0, 3.0]);
  }

  #[test]
  fn explode_coord_skips_empty_and_non_numeric() {
    // Empty tokens are filtered; non-numeric tokens are filtered by `parse.ok()`.
    assert_eq!(explode_coord(",, 4  5,,"), vec![4.0, 5.0]);
    assert_eq!(explode_coord("1 abc 2"), vec![1.0, 2.0]);
  }

  #[test]
  fn explode_coord_empty_input_is_empty() {
    assert!(explode_coord("").is_empty());
    assert!(explode_coord("   ").is_empty());
  }

  #[test]
  fn oval_path_top_starts_and_ends_at_half_height() {
    // For the 't' branch the path begins at (x, y-h/2) and ends at (x+w, y-h/2).
    let s = oval_path("t", 10.0, 20.0, 100.0, 40.0, 5.0);
    assert!(s.starts_with("M 10 0 ")); // y - h/2 = 20 - 20 = 0
    assert!(s.ends_with(" 110 0")); // x+w, y-h/2
  }

  #[test]
  fn oval_path_bottom_traces_reverse_direction() {
    // For the 'b' branch the path begins at (x+w, y-h/2) and ends at (x, y-h/2).
    let s = oval_path("b", 10.0, 20.0, 100.0, 40.0, 5.0);
    assert!(s.starts_with("M 110 0 "));
    assert!(s.ends_with(" 10 0"));
  }

  #[test]
  fn oval_path_unknown_part_uses_simple_line_fallback() {
    let s = oval_path("?", 0.0, 0.0, 50.0, 10.0, 2.0);
    assert_eq!(s, "M 0 0 L 50 0");
  }
}
