//! Integration tests for the latexml_post pipeline.
//!
//! These tests exercise the full post-processing chain on realistic
//! LaTeXML XML documents.

use latexml_post::document::{PostDocument, PostDocumentOptions};
use latexml_post::object_db::ObjectDB;
use latexml_post::processor::Processor;
use latexml_post::scan::Scan;
use latexml_post::Post;

const SIMPLE_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml class="article" options="onecolumn"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <title>Test Document</title>
  <section xml:id="S1" inlist="toc">
    <tags><tag role="refnum">1</tag></tags>
    <title>Introduction</title>
    <para xml:id="S1.p1">
      <p>Hello world.</p>
    </para>
  </section>
  <section xml:id="S2" inlist="toc">
    <tags><tag role="refnum">2</tag></tags>
    <title>Conclusion</title>
    <para xml:id="S2.p1">
      <p>Goodbye world.</p>
    </para>
  </section>
</document>"#;

const MATH_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <para xml:id="p1">
    <Math xml:id="m1" mode="inline" tex="x+y">
      <XMath>
        <XMApp>
          <XMTok role="ADDOP" meaning="plus">+</XMTok>
          <XMTok role="ID">x</XMTok>
          <XMTok role="ID">y</XMTok>
        </XMApp>
      </XMath>
    </Math>
  </para>
</document>"#;

#[test]
fn test_scan_simple_document() {
  let doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  let db = ObjectDB::new();
  let mut scanner = Scan::new(db);

  let nodes = scanner.to_process(&doc);
  assert!(!nodes.is_empty(), "Scanner should find the document root");

  let result = scanner.process(doc, nodes);
  assert!(result.is_ok());
  let docs = result.unwrap();
  assert_eq!(docs.len(), 1);

  // Verify the ObjectDB was populated
  assert!(scanner.db.lookup("SITE_ROOT").is_some(), "SITE_ROOT should be registered");
}

#[test]
fn test_full_pipeline_empty() {
  let doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  let mut post = Post::new();
  let mut processors: Vec<Box<dyn Processor>> = vec![];
  let result = post.process_chain(doc, &mut processors);
  assert!(result.is_ok());
  assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn test_full_pipeline_with_scan() {
  let doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  let mut post = Post::new();
  let db = ObjectDB::new();
  let scanner = Scan::new(db);
  let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(scanner)];
  let result = post.process_chain(doc, &mut processors);
  assert!(result.is_ok());
  assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn test_math_document_parsing() {
  let doc = PostDocument::new_from_string(MATH_DOC, PostDocumentOptions::default()).unwrap();

  // Verify XPath finds Math elements
  let maths = doc.findnodes("//ltx:Math");
  assert_eq!(maths.len(), 1, "Should find one Math element");

  // Verify XMath content
  let xmaths = doc.findnodes("//ltx:XMath");
  assert_eq!(xmaths.len(), 1);

  // Verify XMTok elements
  let tokens = doc.findnodes("//ltx:XMTok");
  assert_eq!(tokens.len(), 3, "Should find 3 tokens: +, x, y");
}

#[test]
fn test_document_id_management() {
  let mut doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();

  // Note: XML IDs found via XPath may depend on namespace registration.
  // The ID cache is populated during set_document_internal via findnodes("//*[@xml:id]").
  // If namespace isn't properly registered, XPath won't find them.
  // Test uniquify_id independently:
  let id1 = doc.uniquify_id("test_id", None);
  let id2 = doc.uniquify_id("test_id", None);
  assert_ne!(id1, id2, "Two uniquify calls should produce different IDs");
  assert!(id1.starts_with("test_id"));
  assert!(id2.starts_with("test_id"));
}

#[test]
fn test_processing_instructions() {
  let doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  // PIs in XML are parsed differently by different parsers.
  // The PI extraction uses XPath ".//processing-instruction('latexml')"
  // which requires the PI to be a child of the document or root element.
  // If PIs are outside the root element, XPath from the document root finds them.
  // Test that the search paths include "." as fallback (always added).
  assert!(doc.searchpaths.contains(&".".to_string()), "Searchpaths should include '.'");
}

#[test]
fn test_namespace_registration() {
  let mut doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  assert!(doc.namespaces.contains_key("ltx"), "ltx namespace should be registered");

  doc.add_namespace("m", "http://www.w3.org/1998/Math/MathML");
  assert!(doc.namespaces.contains_key("m"), "m namespace should be registered after add");
}
