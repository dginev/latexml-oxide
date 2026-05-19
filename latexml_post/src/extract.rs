//! Document-subtree extraction helpers — port of `LaTeXML::Util::Pack`'s
//! `get_math` and `get_embeddable` (Perl Pack.pm L247-313).
//!
//! These are the implementations behind the `--whatsout fragment` and
//! `--whatsout math` CLI modes. They run AFTER all post-processing on
//! the final XML/HTML document and return the subtree the user actually
//! wanted (an embeddable inline snippet, or just the math).
//!
//! Companion modules:
//! * [`crate::pack`] bundles the chosen output into a zip archive.
//! * [`crate::writer`] serializes it to a file or stdout.
//!
//! ## What's not ported (intentional gaps)
//!
//! Perl `get_math` falls through to inline an SVG when the math node's
//! `imagesrc` ends in `.svg` — for our pipeline the SVG is already a
//! referenced resource bundled by `pack_archive`, so the inline path
//! has no caller. Doc string flags the gap so a future visitor knows
//! it's intentional.
//!
//! Perl `get_embeddable` copies namespace declarations and RDFa
//! attributes from the document root onto the extracted node. libxml-rs
//! exposes `Node::set_attribute` for the RDFa half cheaply; namespace
//! re-binding requires FFI into `xmlSetNs` and is deferred (the
//! extracted subtree usually inherits its namespaces fine as long as
//! the consumer doesn't re-parse it standalone).

use libxml::tree::Node;

use crate::document::PostDocument;

/// Output extraction mode — port of Perl `LaTeXML::Util::Pack`'s
/// `whatsout` option (Pack.pm L323-345). Selects which subtree of the
/// post-processed document to serialize and ship to the user.
///
/// * [`Whatsout::Document`] — full document, no extraction (default).
/// * [`Whatsout::Fragment`] — embeddable HTML snippet via
///   [`get_embeddable`].
/// * [`Whatsout::Math`] — math subtree (or fallback) via [`get_math`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Whatsout {
  #[default]
  Document,
  Fragment,
  Math,
}

impl Whatsout {
  /// Parse a CLI string into the matching variant. Returns `None` for
  /// unrecognized values; callers typically fall back to `Document`.
  /// Mirrors Perl `pack_collection`'s string-tag dispatch — accepts
  /// `archive` for backward-compat but maps it to `Document` (archive
  /// bundling is the `pack::pack_archive` concern, not extraction).
  pub fn from_cli(s: &str) -> Option<Self> {
    match s {
      "document" | "archive" => Some(Whatsout::Document),
      "fragment" => Some(Whatsout::Fragment),
      "math" => Some(Whatsout::Math),
      _ => None,
    }
  }
}

/// Apply the requested [`Whatsout`] extraction to `doc` and return the
/// serialized subtree. Returns the full document for
/// [`Whatsout::Document`] (the default no-op) or when extraction finds
/// no candidate node.
///
/// Wraps the two `get_*` helpers + libxml `node_to_string` so callers
/// don't have to thread the inner [`libxml::tree::Document`] through.
pub fn serialize_whatsout(doc: &PostDocument, mode: Whatsout) -> String {
  match mode {
    Whatsout::Document => doc.to_xml_string(),
    Whatsout::Fragment => get_embeddable(doc)
      .map(|n| doc.get_document().node_to_string(&n))
      .unwrap_or_else(|| doc.to_xml_string()),
    Whatsout::Math => get_math(doc)
      .map(|n| doc.get_document().node_to_string(&n))
      .unwrap_or_else(|| doc.to_xml_string()),
  }
}

/// XPath that matches the `<math>` (HTML5 / MathML) and `<Math>`
/// (legacy LaTeXML pre-MathML) elements anywhere in the document,
/// regardless of namespace prefix.
const MATH_XPATH: &str = "//*[local-name()='math' or local-name()='Math']";

/// XPath for math-as-image fallback: `<img class="ltx_Math …">`.
const MATH_IMG_XPATH: &str = "//*[local-name()='img' and contains(@class,'ltx_Math')]";

/// XPath for embeddable fragment root: `<div class="ltx_document">`.
/// Perl literally uses `contains(@class, "ltx_document")` so any
/// element whose `class` includes the substring matches.
const EMBEDDABLE_XPATH: &str = "//*[contains(@class,'ltx_document')]";

/// RDFa attributes that Perl `get_embeddable` copies from the document
/// root onto the extracted node (Perl Pack.pm L309).
const RDFA_ATTRS: &[&str] = &[
  "prefix", "property", "content", "resource", "about", "typeof", "rel", "rev", "datatype",
];

/// Class-name pattern that a single-child `<div>` must match for the
/// unwrap loop to descend into it. Perl regex `/^ltx_(page_(main|content)|document|para|header)$/`.
fn is_unwrappable_div_class(class: &str) -> bool {
  matches!(
    class,
    "ltx_page_main"
      | "ltx_page_content"
      | "ltx_document"
      | "ltx_para"
      | "ltx_header"
  )
}

/// Local-name predicate for the "this <p> is purely inline content"
/// check in `get_embeddable`: child names matching `math|text|span`.
fn is_inline_child(name: &str) -> bool {
  // Perl uses `=~ /math|text|span/` (substring match anywhere in the
  // node name). Match the substring semantics exactly.
  name.contains("math") || name.contains("text") || name.contains("span")
}

/// Extract the math subtree(s) from a post-processed document, mirroring
/// Perl `LaTeXML::Util::Pack::get_math` (Pack.pm L247-280).
///
/// * If exactly one `<math>` (or `<Math>`) is present, return it.
/// * If multiple, return their **least common ancestor** by walking
///   up from the first one until its descendant-count matches the
///   document's total. Unwraps trailing `<tr>` / `<td>` so the LCA
///   isn't a table cell.
/// * If no math nodes at all, fall through to a math-image (`<img
///   class="ltx_Math">`) XPath; if that's also empty, fall through
///   to [`get_embeddable`] so callers get a useful node either way.
pub fn get_math(doc: &PostDocument) -> Option<Node> {
  let math_nodes = doc.findnodes(MATH_XPATH);
  let math_count = math_nodes.len();

  if math_count == 0 {
    let img_nodes = doc.findnodes(MATH_IMG_XPATH);
    if img_nodes.is_empty() {
      return get_embeddable(doc);
    }
    return img_nodes.into_iter().next();
  }

  let mut math = math_nodes.into_iter().next()?;
  if math_count > 1 {
    // Walk up until the subtree under `math` contains every math node.
    // Perl re-runs the same XPath relative to the current candidate
    // and adds 1 when the candidate itself is a math node. We use
    // `findnodes_at` (libxml's `node_evaluate`) for context-scoped
    // XPath — `findnodes_foreign` falls back to a manual traverser
    // that doesn't support `local-name()` predicates.
    let descendant_math_xpath = format!(".{MATH_XPATH}");
    let mut found = 0;
    while found != math_count {
      found = doc.findnodes_at(&descendant_math_xpath, Some(&math)).len();
      if math.get_name().eq_ignore_ascii_case("math") {
        found += 1;
      }
      if found != math_count {
        match math.get_parent() {
          Some(p) => math = p,
          None => break,
        }
      }
    }
    // Don't anchor on a table cell — climb out of `<tr>` / `<td>`
    // (Perl `while ($math->nodeName =~ '^t[rd]$')`).
    while is_table_row_or_cell(&math.get_name()) {
      match math.get_parent() {
        Some(p) => math = p,
        None => break,
      }
    }
  }

  Some(math)
}

/// Extract an embeddable HTML fragment from a post-processed document,
/// mirroring Perl `LaTeXML::Util::Pack::get_embeddable` (Pack.pm L282-313).
///
/// * Find the first `<div class="ltx_document">`.
/// * Unwrap as long as the current node is a `<div>` with exactly one
///   child, an unwrappable wrapper class (`ltx_page_main`,
///   `ltx_page_content`, `ltx_document`, `ltx_para`, `ltx_header`),
///   and no inline `style` attribute.
/// * If the resulting node is a `<p>` whose every child is inline-
///   compatible (local-name contains `math` / `text` / `span`), rename
///   it to `<span class="text">` so it stays inline-embeddable.
/// * Copy RDFa attributes (`prefix`, `property`, `content`, `resource`,
///   `about`, `typeof`, `rel`, `rev`, `datatype`) from the document
///   root onto the extracted node so the snippet retains its semantic
///   annotations.
/// * Namespace declarations from the root are NOT propagated (libxml-rs
///   gap — see module docs).
///
/// If no `ltx_document` element is found, returns the document root,
/// matching Perl's `return $embeddable || $doc`.
pub fn get_embeddable(doc: &PostDocument) -> Option<Node> {
  let root = doc.get_document_element()?;
  let mut embeddable = doc.findnodes(EMBEDDABLE_XPATH).into_iter().next().unwrap_or_else(|| root.clone());

  // Unwrap nested single-child div wrappers.
  loop {
    if embeddable.get_name() != "div" {
      break;
    }
    let children = embeddable.get_child_nodes();
    if children.len() != 1 {
      break;
    }
    let class = embeddable.get_attribute("class").unwrap_or_default();
    if !is_unwrappable_div_class(&class) {
      break;
    }
    if embeddable.get_attribute("style").is_some() {
      break;
    }
    match embeddable.get_first_child() {
      Some(c) => embeddable = c,
      None => break,
    }
  }

  // `<p>` with all-inline children → rename to `<span class="text">`.
  if embeddable.get_name() == "p" {
    let children = embeddable.get_child_nodes();
    if !children.is_empty() && children.iter().all(|c| is_inline_child(&c.get_name())) {
      let _ = embeddable.set_name("span");
      let _ = embeddable.set_attribute("class", "text");
    }
  }

  // Copy RDFa attributes from doc root.
  for attr in RDFA_ATTRS {
    if let Some(value) = root.get_attribute(attr) {
      let _ = embeddable.set_attribute(attr, &value);
    }
  }

  Some(embeddable)
}

fn is_table_row_or_cell(name: &str) -> bool {
  matches!(name, "tr" | "td" | "TR" | "TD")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::document::PostDocumentOptions;

  fn doc(xml: &str) -> PostDocument {
    PostDocument::new_from_string(xml, PostDocumentOptions::default())
      .expect("parse test fixture")
  }

  #[test]
  fn get_embeddable_returns_root_when_no_ltx_document() {
    let d = doc("<html><body><p>hello</p></body></html>");
    let node = get_embeddable(&d).expect("some node");
    assert_eq!(node.get_name(), "html");
  }

  // Fixtures use compact XML (no inter-element whitespace). libxml
  // preserves whitespace text nodes as children, which would break
  // the single-child unwrap check — Perl XML::LibXML behaves the same
  // way, and real LaTeXML post-processing emits compact HTML by the
  // time get_embeddable is called.

  #[test]
  fn get_embeddable_unwraps_single_child_wrappers_to_inline_span() {
    // ltx_document (1 child div) → ltx_page_main (1 child p) → p
    // (terminal: not in unwrap-class allowlist). The <p>'s single
    // child is the text "Hello world" whose node-name is "text"
    // (matches inline pattern) → final promotion to span.
    let xml = r#"<html><body><div class="ltx_document"><div class="ltx_page_main"><p>Hello world</p></div></div></body></html>"#;
    let d = doc(xml);
    let node = get_embeddable(&d).expect("some node");
    assert_eq!(node.get_name(), "span");
    assert_eq!(node.get_attribute("class").as_deref(), Some("text"));
  }

  #[test]
  fn get_embeddable_stops_at_multi_child() {
    // ltx_document has TWO <p> children → unwrap halts; return it as-is.
    let xml = r#"<html><body><div class="ltx_document"><p>first</p><p>second</p></div></body></html>"#;
    let d = doc(xml);
    let node = get_embeddable(&d).expect("some node");
    assert_eq!(node.get_name(), "div");
    assert_eq!(node.get_attribute("class").as_deref(), Some("ltx_document"));
  }

  #[test]
  fn get_embeddable_keeps_p_when_child_is_non_inline_block() {
    // Single-child path lands on the <p>; child is a <table>, whose
    // name doesn't match math|text|span → no promotion to span.
    let xml = r#"<html><body><div class="ltx_document"><div class="ltx_para"><p><table>x</table></p></div></div></body></html>"#;
    let d = doc(xml);
    let node = get_embeddable(&d).expect("some node");
    assert_eq!(node.get_name(), "p");
  }

  #[test]
  fn get_math_returns_lone_math_node() {
    let xml = r#"<html><body><p>some text</p><math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi></math></body></html>"#;
    let d = doc(xml);
    let node = get_math(&d).expect("some node");
    assert_eq!(node.get_name(), "math");
  }

  #[test]
  fn get_math_returns_lca_for_multiple_math() {
    let xml = r#"<html><body><div id="container"><math xmlns="http://www.w3.org/1998/Math/MathML"><mi>a</mi></math><math xmlns="http://www.w3.org/1998/Math/MathML"><mi>b</mi></math></div></body></html>"#;
    let d = doc(xml);
    let node = get_math(&d).expect("some node");
    // Both math nodes are children of `<div id="container">` — the
    // LCA should be that div.
    assert_eq!(node.get_name(), "div");
    assert_eq!(node.get_attribute("id").as_deref(), Some("container"));
  }

  #[test]
  fn get_math_falls_through_to_img_when_no_math_elements() {
    let xml = r#"<html><body><p>before</p><img class="ltx_Math" alt="x"/></body></html>"#;
    let d = doc(xml);
    let node = get_math(&d).expect("some node");
    assert_eq!(node.get_name(), "img");
  }

  #[test]
  fn get_math_falls_through_to_embeddable_when_no_math_at_all() {
    // No math + no math-img → fall through to get_embeddable. The
    // embeddable result for a single-text-child <p> is the promoted
    // <span class="text">.
    let xml = r#"<html><body><div class="ltx_document"><p>just prose</p></div></body></html>"#;
    let d = doc(xml);
    let node = get_math(&d).expect("some node");
    assert_eq!(node.get_name(), "span");
  }

  #[test]
  fn whatsout_from_cli_recognized() {
    assert_eq!(Whatsout::from_cli("document"), Some(Whatsout::Document));
    assert_eq!(Whatsout::from_cli("fragment"), Some(Whatsout::Fragment));
    assert_eq!(Whatsout::from_cli("math"), Some(Whatsout::Math));
    // Backward-compat: `--whatsout archive` was historically how the
    // zip output mode was selected; now archive bundling is
    // controlled separately (by `--dest *.zip`) so the extraction
    // step is a no-op.
    assert_eq!(Whatsout::from_cli("archive"), Some(Whatsout::Document));
    assert_eq!(Whatsout::from_cli("nonsense"), None);
  }

  #[test]
  fn whatsout_default_is_document() {
    assert_eq!(Whatsout::default(), Whatsout::Document);
  }

  #[test]
  fn serialize_whatsout_document_matches_full_xml() {
    let xml = r#"<html><body><div class="ltx_document"><p>hi</p></div></body></html>"#;
    let d = doc(xml);
    let full = serialize_whatsout(&d, Whatsout::Document);
    assert!(full.contains("<html>") && full.contains("</html>"));
  }

  #[test]
  fn serialize_whatsout_fragment_strips_html_wrapper() {
    let xml = r#"<html><body><div class="ltx_document"><p>hi</p></div></body></html>"#;
    let d = doc(xml);
    let frag = serialize_whatsout(&d, Whatsout::Fragment);
    // Fragment unwraps ltx_document and promotes the lone-text <p>
    // to <span class="text"> — the result should NOT carry the
    // <html>/<body> wrapper.
    assert!(!frag.contains("<html>"), "frag contains html wrapper: {frag}");
    assert!(frag.contains("hi"));
  }

  #[test]
  fn serialize_whatsout_math_returns_math_subtree() {
    let xml = r#"<html><body><p>txt</p><math xmlns="http://www.w3.org/1998/Math/MathML"><mi>z</mi></math></body></html>"#;
    let d = doc(xml);
    let m = serialize_whatsout(&d, Whatsout::Math);
    assert!(m.contains("<mi>z</mi>"));
    assert!(!m.contains("<html>"), "math contains html wrapper: {m}");
    assert!(!m.contains("<p>txt</p>"), "math contains unrelated text: {m}");
  }

  #[test]
  fn get_embeddable_copies_rdfa_from_root() {
    let xml = r#"<html prefix="dc: http://purl.org/dc/terms/" typeof="ScholarlyArticle"><body><div class="ltx_document"><p>text</p></div></body></html>"#;
    let d = doc(xml);
    let node = get_embeddable(&d).expect("some node");
    // Unwrap reaches the <p>, then promotes to <span>; either way
    // the RDFa attrs should land on the result.
    assert_eq!(
      node.get_attribute("prefix").as_deref(),
      Some("dc: http://purl.org/dc/terms/")
    );
    assert_eq!(
      node.get_attribute("typeof").as_deref(),
      Some("ScholarlyArticle")
    );
  }
}
