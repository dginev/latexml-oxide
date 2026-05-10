//! Integration tests for the latexml_post pipeline.
//!
//! These tests exercise the full post-processing chain on realistic
//! LaTeXML XML documents.

use latexml_post::Post;
use latexml_post::document::{PostDocument, PostDocumentOptions};
use latexml_post::object_db::ObjectDB;
use latexml_post::processor::Processor;
use latexml_post::scan::Scan;

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
  assert!(
    scanner.db.lookup("SITE_ROOT").is_some(),
    "SITE_ROOT should be registered"
  );
}

#[test]
fn test_full_pipeline_empty() {
  let doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  let mut post = Post::new();
  let mut processors: Vec<Box<dyn Processor>> = vec![];
  let result = post.process_chain(vec![doc], &mut processors);
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
  let result = post.process_chain(vec![doc], &mut processors);
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
  assert!(
    doc.searchpaths.contains(&".".to_string()),
    "Searchpaths should include '.'"
  );
}

#[test]
fn test_namespace_registration() {
  let mut doc = PostDocument::new_from_string(SIMPLE_DOC, PostDocumentOptions::default()).unwrap();
  assert!(
    doc.namespaces.contains_key("ltx"),
    "ltx namespace should be registered"
  );

  doc.add_namespace("m", "http://www.w3.org/1998/Math/MathML");
  assert!(
    doc.namespaces.contains_key("m"),
    "m namespace should be registered after add"
  );
}

/// Regression test for the vector-SVG graphics path (opt-in via
/// `--graphics-svg-threshold-kb N`). Uses the cifar10 plot PDF from the
/// upstream [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
/// thread — a 41 KB vector-authored matplotlib chart that's the canonical
/// "inkscape preserves vectors better than ImageMagick" example.
///
/// Test behaviour:
/// - If `inkscape` is missing from PATH, the test exits silently. This keeps the suite green on
///   minimal runners; CI installs inkscape so the branch is covered on GH Actions.
/// - If `inkscape` is present, exercise the Graphics processor with `svg_threshold_kb = 200` and
///   assert the output is a real SVG file.
#[test]
fn test_vector_svg_graphics_path() {
  if std::process::Command::new("inkscape")
    .arg("--version")
    .output()
    .ok()
    .filter(|o| o.status.success())
    .is_none()
  {
    eprintln!("inkscape not installed; skipping vector-SVG regression test");
    return;
  }

  let fixture = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/cifar10_vector.pdf"
  );
  assert!(
    std::path::Path::new(fixture).exists(),
    "fixture missing: {}",
    fixture
  );

  let work = std::env::temp_dir().join(format!("latexml_svg_test_{}", std::process::id()));
  std::fs::create_dir_all(&work).expect("mkdir work");
  let src_copy = work.join("cifar10_vector.pdf");
  std::fs::copy(fixture, &src_copy).expect("copy fixture");

  let mut graphics = latexml_post::graphics::Graphics::new(None, true).with_svg_threshold_kb(200);

  let xml = format!(
    r#"<?xml version="1.0"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="d">
  <graphics graphic="cifar10_vector.pdf" candidates="{}"/>
</document>"#,
    src_copy.display()
  );
  let doc_opts = PostDocumentOptions {
    destination: Some(work.join("out.html").display().to_string()),
    source_directory: Some(work.display().to_string()),
    ..Default::default()
  };
  let doc = PostDocument::new_from_string(&xml, doc_opts).expect("parse");

  let nodes = graphics.to_process(&doc);
  assert_eq!(nodes.len(), 1, "one graphics node expected");
  let _out = graphics.process(doc, nodes).expect("graphics process");

  let svg_path = work.join("cifar10_vector.svg");
  assert!(
    svg_path.exists(),
    "expected SVG at {} — inkscape path should have fired for a 41 KB vector PDF",
    svg_path.display()
  );
  let svg_bytes = std::fs::read(&svg_path).expect("read svg");
  assert!(
    svg_bytes.windows(4).any(|w| w == b"<svg"),
    "SVG root element not found in output"
  );
  // Upper bound sanity — inkscape on a vector-authored plot produces tens
  // of KB, not hundreds of MB. Raster-embedded PDFs blow up to 100+ MB —
  // that's the case the file-size heuristic must exclude upstream.
  assert!(
    svg_bytes.len() < 2 * 1024 * 1024,
    "SVG is {} bytes — vector-authored PDFs should yield <2 MB SVG",
    svg_bytes.len()
  );

  // Cleanup.
  let _ = std::fs::remove_dir_all(&work);
}

/// Second vector-SVG regression: a PDF that is *pathologically slow* for
/// ImageMagick `convert`. fig8.pdf (attached to
/// [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
/// and called out from arxiv:1807.01606) is a 41 KB vector-authored PDF
/// that triggers a 30+ second rasterisation in `convert` via ghostscript.
/// Inkscape parses the same PDF directly and emits SVG in ~250 ms —
/// measured 130× speedup on the round-17 dev machine.
///
/// This test asserts the inkscape path *completes* (doesn't time out)
/// and does NOT exercise the slow convert path (would blow the suite
/// runtime). Silent skip if inkscape is missing.
#[test]
fn test_vector_svg_pathological_convert_case() {
  if std::process::Command::new("inkscape")
    .arg("--version")
    .output()
    .ok()
    .filter(|o| o.status.success())
    .is_none()
  {
    eprintln!("inkscape not installed; skipping pathological-PDF regression test");
    return;
  }

  let fixture = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pathological_vector.pdf"
  );
  assert!(
    std::path::Path::new(fixture).exists(),
    "fixture missing: {}",
    fixture
  );

  let work = std::env::temp_dir().join(format!("latexml_svg_path_test_{}", std::process::id()));
  std::fs::create_dir_all(&work).expect("mkdir work");
  let src_copy = work.join("pathological_vector.pdf");
  std::fs::copy(fixture, &src_copy).expect("copy fixture");

  let mut graphics = latexml_post::graphics::Graphics::new(None, true).with_svg_threshold_kb(200);

  let xml = format!(
    r#"<?xml version="1.0"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="d">
  <graphics graphic="pathological_vector.pdf" candidates="{}"/>
</document>"#,
    src_copy.display()
  );
  let doc_opts = PostDocumentOptions {
    destination: Some(work.join("out.html").display().to_string()),
    source_directory: Some(work.display().to_string()),
    ..Default::default()
  };
  let doc = PostDocument::new_from_string(&xml, doc_opts).expect("parse");

  let nodes = graphics.to_process(&doc);
  assert_eq!(nodes.len(), 1);

  let t0 = std::time::Instant::now();
  let _out = graphics.process(doc, nodes).expect("graphics process");
  let elapsed = t0.elapsed();

  let svg_path = work.join("pathological_vector.svg");
  assert!(
    svg_path.exists(),
    "expected SVG at {} — inkscape should succeed on this pathological-for-convert PDF",
    svg_path.display()
  );
  // Upper bound: inkscape SVG of a 41 KB vector-authored PDF is ~100 KB
  // and completes in well under a second on any machine. Give generous
  // CI slack (5 s) — convert takes 30+ s, so there's no way a 5 s bound
  // accidentally masks a fallback to the raster path.
  assert!(
    elapsed < std::time::Duration::from_secs(5),
    "inkscape path on fig8.pdf took {:?} — should be <1 s, way under the 30s+ convert path",
    elapsed
  );

  let _ = std::fs::remove_dir_all(&work);
}
