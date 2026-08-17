//! In-memory join of multiple core-XML documents into one.
//!
//! An arXiv submission may ship several top-level `.tex` files — a main paper
//! plus Supplementary-Material documents (see [`crate::main_tex::find_top_level_texs`]).
//! Each is converted **independently** (its own `\documentclass`, its own core
//! XML), then this module stitches them into a single core-XML document that the
//! normal post-processing pipeline renders as one output: the main first, each
//! supplement appended as a top-level appendix `<section>` titled by the
//! supplement's own `\title`.
//!
//! This is the **non-streaming** join — it holds the parsed supplements in
//! memory — which suits the overwhelmingly common case where a main+supplement
//! pair is small. Very large submissions that need streaming are a separate
//! concern (a post-pass "join" over separate core-XML files, tracked
//! separately); this path deliberately keeps the whole pipeline downstream of a
//! single `<document>` unchanged.
//!
//! **Id de-confliction.** Both documents number from `S1`, share the label
//! namespace (`LABEL:sec:intro`), etc. Every supplement's id/ref/label space is
//! rewritten with a per-source prefix (`as1_`, `as2_`, …) before splicing, so
//! intra-supplement `\ref`s still resolve and never collide with the main. A
//! supplement that cross-`\ref`s *into the main* will not resolve — faithful to
//! arXiv, whose separately-compiled PDFs cannot cross-reference either.

use latexml_core::common::xml::XML_NS;
use latexml_post::document::{PostDocument, PostDocumentOptions};
use libxml::tree::{Node, NodeType};

/// Frontmatter elements dropped when a supplement becomes an appendix section
/// (its `<title>` is promoted to the section heading; its own author/abstract
/// block does not belong mid-document).
const DROP_FRONTMATTER: &[&str] = &[
  "resource",
  "creator",
  "date",
  "abstract",
  "keywords",
  "classification",
];

/// Join a `main` core-XML string with zero or more `supplements`, returning the
/// combined core XML. With no supplements the main is returned verbatim.
pub fn join_core_documents(main: &str, supplements: &[String]) -> Result<String, String> {
  if supplements.is_empty() {
    return Ok(main.to_string());
  }
  let mut appendices = String::new();
  for (i, supp) in supplements.iter().enumerate() {
    appendices.push_str(&build_appendix(supp, i + 1)?);
  }
  splice_before_document_close(main, &appendices)
}

/// Parse one supplement, prefix its id/ref/label space, and render it as a
/// top-level appendix `<section>` string (heading = the supplement's `<title>`).
fn build_appendix(supp_xml: &str, idx: usize) -> Result<String, String> {
  let doc = PostDocument::new_from_string(supp_xml, PostDocumentOptions::default())
    .map_err(|e| format!("supplement {idx} parse failed: {e}"))?;
  let root = doc
    .get_document_element()
    .ok_or_else(|| format!("supplement {idx} has no root <document>"))?;
  let prefix = format!("as{idx}_");
  prefix_id_space(&root, &prefix);

  let mut title_xml = String::new();
  let mut body = String::new();
  for child in root.get_child_nodes() {
    if child.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let name = child.get_name();
    if name == "title" && title_xml.is_empty() {
      title_xml = doc.node_to_string(&child);
    } else if !DROP_FRONTMATTER.contains(&name.as_str()) {
      body.push_str(&doc.node_to_string(&child));
    }
  }
  if title_xml.is_empty() {
    title_xml = "<title>Supplementary Material</title>".to_string();
  }
  // `class="ltx_appendix"` is the thin presentation hook for XSLT/CSS. `xml:id`
  // is already namespaced by the reserved `xml:` prefix on re-parse; the section
  // inherits the main document's default namespace at the splice point.
  Ok(format!(
    "<section class=\"ltx_appendix\" inlist=\"toc\" xml:id=\"as{idx}\">{title_xml}{body}</section>"
  ))
}

/// Rewrite every id, reference and label under `node` with `prefix`, so a
/// supplement's id space cannot collide with the main's. `inlist` (TOC list
/// membership, e.g. `"toc"`) is intentionally left untouched so the appendix
/// still joins the combined table of contents.
fn prefix_id_space(node: &Node, prefix: &str) {
  if node.get_type() == Some(NodeType::ElementNode) {
    let mut n = node.clone();
    // `xml:id` is namespaced (reserved xml: prefix) — read NS-aware, write bare.
    if let Some(v) = n.get_attribute_ns("id", XML_NS) {
      n.set_attribute("xml:id", &format!("{prefix}{v}")).ok();
    }
    for attr in ["idref", "fragid"] {
      if let Some(v) = n.get_attribute(attr) {
        n.set_attribute(attr, &format!("{prefix}{v}")).ok();
      }
    }
    for attr in ["labels", "labelref"] {
      if let Some(v) = n.get_attribute(attr) {
        let rewritten = v
          .split_whitespace()
          .map(|tok| prefix_label(tok, prefix))
          .collect::<Vec<_>>()
          .join(" ");
        n.set_attribute(attr, &rewritten).ok();
      }
    }
    if let Some(v) = n.get_attribute("href")
      && let Some(frag) = v.strip_prefix('#')
    {
      n.set_attribute("href", &format!("#{prefix}{frag}")).ok();
    }
  }
  for child in node.get_child_nodes() {
    prefix_id_space(&child, prefix);
  }
}

/// Prefix a single label token, preserving the `LABEL:` sentinel.
fn prefix_label(tok: &str, prefix: &str) -> String {
  match tok.strip_prefix("LABEL:") {
    Some(rest) => format!("LABEL:{prefix}{rest}"),
    None => format!("{prefix}{tok}"),
  }
}

/// Insert `appendices` immediately before the main document's closing
/// `</document>` tag. The core-XML root is always a single `<document>` (default
/// LaTeXML namespace), so its last close tag is an unambiguous splice point.
fn splice_before_document_close(main: &str, appendices: &str) -> Result<String, String> {
  let close = "</document>";
  match main.rfind(close) {
    Some(pos) => {
      let mut out = String::with_capacity(main.len() + appendices.len());
      out.push_str(&main[..pos]);
      out.push_str(appendices);
      out.push_str(&main[pos..]);
      Ok(out)
    },
    None => Err("main document has no </document> close tag to splice into".to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const MAIN: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<document xmlns=\"http://dlmf.nist.gov/LaTeXML\">",
    "<title>Main Paper</title>",
    "<section inlist=\"toc\" labels=\"LABEL:sec:intro\" xml:id=\"S1\">",
    "<para xml:id=\"S1.p1\"><p>See <ref labelref=\"LABEL:sec:intro\"/>.</p></para>",
    "</section>",
    "</document>\n"
  );
  const SUPP: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<document xmlns=\"http://dlmf.nist.gov/LaTeXML\">",
    "<resource src=\"LaTeXML.css\" type=\"text/css\"/>",
    "<title>Supplementary Information for Main Paper</title>",
    "<section inlist=\"toc\" labels=\"LABEL:sec:extra\" xml:id=\"S1\">",
    "<para xml:id=\"S1.p1\"><p>See <ref labelref=\"LABEL:sec:extra\"/>.</p></para>",
    "</section>",
    "</document>\n"
  );

  #[test]
  fn no_supplements_returns_main_verbatim() {
    assert_eq!(join_core_documents(MAIN, &[]).unwrap(), MAIN);
  }

  #[test]
  fn supplement_appended_as_prefixed_appendix() {
    let joined = join_core_documents(MAIN, &[SUPP.to_string()]).unwrap();
    // Exactly one <document> — a single joined core document.
    assert_eq!(joined.matches("<document").count(), 1);
    // The main's ids/labels are untouched…
    assert!(joined.contains("xml:id=\"S1\""));
    assert!(joined.contains("LABEL:sec:intro"));
    // …the supplement's are prefixed, so they cannot collide.
    assert!(joined.contains("xml:id=\"as1_S1\""));
    assert!(joined.contains("xml:id=\"as1_S1.p1\""));
    assert!(joined.contains("LABEL:as1_sec:extra"));
    assert!(joined.contains("labelref=\"LABEL:as1_sec:extra\""));
    // The supplement is an appendix titled by its own <title>.
    assert!(joined.contains("class=\"ltx_appendix\""));
    assert!(joined.contains("Supplementary Information for Main Paper"));
    // The supplement's per-doc CSS <resource> is dropped.
    let appendix_start = joined.find("ltx_appendix").unwrap();
    assert!(!joined[appendix_start..].contains("<resource"));
    // Appendix is inside the document (before its close).
    let close = joined.rfind("</document>").unwrap();
    assert!(appendix_start < close);
  }

  #[test]
  fn two_supplements_get_distinct_prefixes() {
    let joined = join_core_documents(MAIN, &[SUPP.to_string(), SUPP.to_string()]).unwrap();
    assert!(joined.contains("xml:id=\"as1_S1\""));
    assert!(joined.contains("xml:id=\"as2_S1\""));
    assert_eq!(joined.matches("class=\"ltx_appendix\"").count(), 2);
  }
}
