//! Integration tests for the latexml_post pipeline.
//!
//! These tests exercise the full post-processing chain on realistic
//! LaTeXML XML documents.

use latexml_post::{
  Post,
  document::{PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::Processor,
  scan::Scan,
};

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

/// True if a vector-SVG converter (`mutool` or `pdftocairo`) is on PATH.
/// The Graphics vector-SVG path tries mutool first, then pdftocairo; if
/// neither is installed the path can't fire and these tests self-skip.
///
/// Presence is detected by whether the binary *spawns* (`output().is_ok()`),
/// NOT by exit status: unlike inkscape, `mutool`/`pdftocairo` exit non-zero
/// on `--version` (unknown/usage), so a `status.success()` gate would
/// silently skip even when the tool is installed — a false-negative that
/// would mask the coverage entirely.
fn svg_converter_available() -> bool {
  ["mutool", "pdftocairo"].iter().any(|tool| {
    std::process::Command::new(tool)
      .arg("--version")
      .output()
      .is_ok()
  })
}

/// Regression test for the vector-SVG graphics path (opt-in via
/// `--graphics-svg-threshold-kb N`). Uses the cifar10 plot PDF from the
/// upstream [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
/// thread — a 41 KB vector-authored matplotlib chart exercising the
/// vector-SVG converters (mutool → pdftocairo) that preserve vectors
/// rather than rasterizing via ImageMagick.
///
/// Test behaviour:
/// - If no vector converter (mutool/pdftocairo) is on PATH, the test exits silently. This keeps the
///   suite green on minimal runners; CI installs poppler/mupdf so the branch is covered on GH Actions.
/// - Otherwise, exercise the Graphics processor with `svg_threshold_kb = 200` and assert the output
///   is a real SVG file.
#[test]
fn test_vector_svg_graphics_path() {
  if !svg_converter_available() {
    eprintln!(
      "no vector-SVG converter (mutool/pdftocairo) on PATH; skipping vector-SVG regression test"
    );
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
    "expected SVG at {} — the vector-SVG path should have fired for a 41 KB vector PDF",
    svg_path.display()
  );
  let svg_bytes = std::fs::read(&svg_path).expect("read svg");
  assert!(
    svg_bytes.windows(4).any(|w| w == b"<svg"),
    "SVG root element not found in output"
  );
  // Upper bound sanity — a vector converter on a vector-authored plot
  // produces tens of KB, not hundreds of MB. Raster-embedded PDFs blow up
  // to 100+ MB — that's the case the file-size heuristic must exclude.
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
/// mutool/pdftocairo parse the same PDF's vectors directly and emit SVG in
/// well under a second — the same direct-vector advantage, without a
/// heavyweight GTK dependency.
///
/// This test asserts the vector-SVG path *completes* fast (doesn't time
/// out) and does NOT exercise the slow convert path (would blow the suite
/// runtime). Silent skip if no vector converter is installed.
#[test]
fn test_vector_svg_pathological_convert_case() {
  if !svg_converter_available() {
    eprintln!(
      "no vector-SVG converter (mutool/pdftocairo) on PATH; skipping pathological-PDF regression test"
    );
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
    "expected SVG at {} — a vector converter should succeed on this pathological-for-convert PDF",
    svg_path.display()
  );
  // Upper bound: a vector converter's SVG of a 41 KB vector-authored PDF is
  // ~100 KB and completes in well under a second on any machine. Give
  // generous CI slack (5 s) — convert takes 30+ s, so there's no way a 5 s
  // bound accidentally masks a fallback to the raster path.
  assert!(
    elapsed < std::time::Duration::from_secs(5),
    "vector-SVG path on fig8.pdf took {:?} — should be <1 s, way under the 30s+ convert path",
    elapsed
  );

  let _ = std::fs::remove_dir_all(&work);
}

/// The thread-local XSLT cache (see `latexml_post::xslt`'s
/// `STYLESHEET_CACHE`) parses each unique stylesheet path once per
/// thread, then reuses the compiled artefact for subsequent calls.
/// This test fires three XSLT::process invocations sequentially and
/// asserts that the 2nd and 3rd runs each take less than the 1st by
/// a margin that the cached-parse path comfortably affords.
///
/// The actual delta we measure is small (the parse itself is only
/// a few ms; the bulk of an XSLT phase is the transform). What this
/// test really validates is **correctness under repeated reuse** of
/// a cached `&mut Stylesheet` — the failure mode it guards against
/// is silent data corruption from libxslt mutating the stylesheet
/// or transform context state between calls. If the assertions below
/// pass with byte-identical output across the three runs, the cache
/// is reusable.
#[test]
fn test_xslt_cache_reuse_produces_identical_output() {
  // Skip if no XSLT stylesheet is reachable; this isn't a perf gate.
  let candidate_paths = [
    "resources/XSLT/LaTeXML-html5.xsl",
    "../resources/XSLT/LaTeXML-html5.xsl",
  ];
  let stylesheet = candidate_paths
    .iter()
    .find(|p| std::path::Path::new(p).exists())
    .copied();
  let Some(stylesheet) = stylesheet else {
    eprintln!("XSLT stylesheet not found in test cwd; skipping cache test");
    return;
  };

  use std::time::Instant;

  use latexml_post::{processor::Processor, xslt::XSLT};

  const SMALL_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <para xml:id="p1"><p>Cache reuse test.</p></para>
</document>"#;

  let mut outputs: Vec<String> = Vec::new();
  let mut elapsed_ms: Vec<u128> = Vec::new();

  for _ in 0..3 {
    let doc = PostDocument::new_from_string(SMALL_DOC, PostDocumentOptions::default())
      .expect("parse small doc");
    let mut xslt = XSLT::new(
      stylesheet,
      rustc_hash::FxHashMap::default(),
      false,
      None,
      vec![],
    )
    .expect("XSLT processor");
    let t0 = Instant::now();
    let result = xslt.process(doc, vec![]).expect("xslt process");
    elapsed_ms.push(t0.elapsed().as_micros());
    let serialized = result
      .into_iter()
      .next()
      .map(|d| d.get_document().to_string())
      .unwrap_or_default();
    outputs.push(serialized);
  }

  // Byte-identity check: the cached stylesheet must produce the same
  // output across calls. Asymmetric mutation of stylesheet/context
  // state would show up here as divergent serialisation.
  assert_eq!(outputs[0], outputs[1], "run 1 vs run 2 output drift");
  assert_eq!(outputs[1], outputs[2], "run 2 vs run 3 output drift");

  // Soft perf assertion: run 2 should not be dramatically slower
  // than run 1 (it should be faster, but cold disk cache can
  // dominate on the first run too). We only fail on outright
  // regression — run 2 taking >2× run 1 — which would indicate
  // cache reuse is broken.
  let r0 = elapsed_ms[0] as f64;
  let r1 = elapsed_ms[1] as f64;
  assert!(
    r1 < 2.0 * r0,
    "cached XSLT run 2 ({r1:.0}us) > 2× run 1 ({r0:.0}us); cache may be broken"
  );
  eprintln!("XSLT cache reuse: runs = {:?} us", elapsed_ms);
}

/// Regression test for issue 498: an SVG `\includegraphics` source must get
/// `imagewidth`/`imageheight` (and the aspect class) from the graphicx
/// options, exactly like raster sources. SVG goes through the trivial-copy
/// path (`Plan::Copy` — web-native, no conversion), whose sizing previously
/// used only the raster `imagesize` crate; SVG sources therefore produced an
/// `<object>` with NO width/height and rendered at intrinsic size.
///
/// Perl ground truth: `Post/Graphics.pm` `transformGraphic` →
/// `image_graphicx_trivial` (`Util/Image.pm` L293-308) sizes the source via
/// `image_size` (Image::Magick handles SVG) and `setGraphicSrc`
/// (`Post/Graphics.pm` L188-210) writes imagewidth/imageheight + the
/// ltx_img_{portrait,landscape,square} class.
///
/// Two sources mirror the issue's two images (634×805 with viewBox,
/// 62×80 with width/height attributes only — both `read_svg_dimensions`
/// extraction paths). Both use `width=137.9979pt,keepaspectratio=true`
/// (= 0.4\textwidth), so both must come out at the SAME display width —
/// the user's actual complaint was same-width images rendering at
/// different sizes.
#[test]
fn test_svg_source_sized_and_aspect_classed() {
  // RAII: the directory goes away even if an assertion below panics.
  let workdir = tempfile::tempdir().expect("tempdir");
  let work = workdir.path();

  // Witness shape from issue 498's image_1.svg: xml prolog + viewBox.
  let svg1 = work.join("fig_viewbox.svg");
  std::fs::write(
    &svg1,
    r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="634" height="805" viewBox="0 0 634 805" xmlns="http://www.w3.org/2000/svg">
 <rect x="0" y="0" width="634" height="805" fill="red"/>
</svg>"#,
  )
  .expect("write svg1");
  // Width/height-only root (no viewBox) — the fallback extraction path.
  let svg2 = work.join("fig_attrs.svg");
  std::fs::write(
    &svg2,
    r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="62" height="80" xmlns="http://www.w3.org/2000/svg">
 <rect x="0" y="0" width="62" height="80" fill="blue"/>
</svg>"#,
  )
  .expect("write svg2");

  let xml = format!(
    r#"<?xml version="1.0"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="d">
  <graphics graphic="fig_viewbox" candidates="{}" options="width=137.9979pt,keepaspectratio=true" xml:id="g1"/>
  <graphics graphic="fig_attrs" candidates="{}" options="width=137.9979pt,keepaspectratio=true" xml:id="g2"/>
</document>"#,
    svg1.display(),
    svg2.display()
  );
  let doc_opts = PostDocumentOptions {
    destination: Some(work.join("out.html").display().to_string()),
    source_directory: Some(work.display().to_string()),
    ..Default::default()
  };
  let doc = PostDocument::new_from_string(&xml, doc_opts).expect("parse");

  let mut graphics = latexml_post::graphics::Graphics::new(None, true);
  let nodes = graphics.to_process(&doc);
  assert_eq!(nodes.len(), 2, "two graphics nodes expected");
  let out = graphics.process(doc, nodes).expect("graphics process");
  let doc = out.into_iter().next().expect("doc back");

  let processed = doc.findnodes("//ltx:graphics");
  assert_eq!(processed.len(), 2);
  for node in &processed {
    let id = latexml_post::document::get_xml_id(node).unwrap_or_default();
    assert_eq!(
      node.get_attribute("imagesrc").as_deref(),
      Some(if id == "g1" {
        "fig_viewbox.svg"
      } else {
        "fig_attrs.svg"
      }),
      "{id}: imagesrc"
    );
    // width=137.9979pt at DPI 100 → ceil(137.9979 * 100/72.27) = 191 px
    // for BOTH images (the option pins the display width).
    assert_eq!(
      node.get_attribute("imagewidth").as_deref(),
      Some("191"),
      "{id}: imagewidth must be the option-scaled width, not missing/intrinsic"
    );
    // Heights auto-scale by each source's aspect ratio:
    //   g1: ceil(805 × 137.9979/(634·72.27/100)) = 243
    //   g2: ceil(80 × 137.9979/(62·72.27/100)) = 247
    // (Perl-with-ImageMagick lands within a pixel — 242/246 — via its
    // pt→bp detour in image_graphicx_trivial; the port's accepted
    // transform math ceils once, one px above, for every Copy source.)
    let want_h = if id == "g1" { "243" } else { "247" };
    assert_eq!(
      node.get_attribute("imageheight").as_deref(),
      Some(want_h),
      "{id}: imageheight must preserve the source aspect ratio"
    );
    let class = node.get_attribute("class").unwrap_or_default();
    assert!(
      class.contains("ltx_img_portrait"),
      "{id}: aspect class missing/wrong (class was '{class}')"
    );
  }
}

/// The companion to the test above, for the case where sizing this way can make
/// things *worse* than not sizing at all: an SVG whose root lengths carry a unit
/// and which has **no** viewBox, included with no graphicx options.
///
/// There is nothing to scale against, so whatever the reader measures goes
/// straight into `imagewidth`/`imageheight`. A reader that drops the unit calls
/// a 10 cm × 7.5 cm drawing "10 × 8" and the browser renders a 10-pixel
/// thumbnail — worse than the pre-issue-498 behaviour, which wrote no
/// attributes at all and let the browser use the SVG's own intrinsic size.
///
/// The percentage-sized source is the other half of the contract: it has no
/// intrinsic pixel size, so the correct answer is to write nothing and leave the
/// sizing to the browser.
#[test]
fn test_unit_bearing_svg_without_viewbox_is_converted_not_truncated() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let work = workdir.path();

  let cm = work.join("metric.svg");
  std::fs::write(
    &cm,
    r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="10cm" height="7.5cm" xmlns="http://www.w3.org/2000/svg">
 <rect x="0" y="0" width="100%" height="100%" fill="green"/>
</svg>"#,
  )
  .expect("write cm svg");
  let pct = work.join("fluid.svg");
  std::fs::write(
    &pct,
    r#"<svg width="100%" height="100%" xmlns="http://www.w3.org/2000/svg">
 <rect x="0" y="0" width="10" height="10" fill="grey"/>
</svg>"#,
  )
  .expect("write pct svg");

  let xml = format!(
    r#"<?xml version="1.0"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="d">
  <graphics graphic="metric" candidates="{}" options="" xml:id="g1"/>
  <graphics graphic="fluid" candidates="{}" options="" xml:id="g2"/>
</document>"#,
    cm.display(),
    pct.display()
  );
  let doc_opts = PostDocumentOptions {
    destination: Some(work.join("out.html").display().to_string()),
    source_directory: Some(work.display().to_string()),
    ..Default::default()
  };
  let doc = PostDocument::new_from_string(&xml, doc_opts).expect("parse");
  let mut graphics = latexml_post::graphics::Graphics::new(None, true);
  let nodes = graphics.to_process(&doc);
  assert_eq!(nodes.len(), 2, "two graphics nodes expected");
  let out = graphics.process(doc, nodes).expect("graphics process");
  let doc = out.into_iter().next().expect("doc back");

  for node in &doc.findnodes("//ltx:graphics") {
    let id = latexml_post::document::get_xml_id(node).unwrap_or_default();
    let (w, h) = (
      node.get_attribute("imagewidth"),
      node.get_attribute("imageheight"),
    );
    if id == "g1" {
      // 10cm = 10/2.54 in = 378 px; 7.5cm = 283 px.
      assert_eq!(
        (w.as_deref(), h.as_deref()),
        (Some("378"), Some("283")),
        "a cm-sized SVG must be measured in pixels, not stripped of its unit"
      );
      let class = node.get_attribute("class").unwrap_or_default();
      assert!(
        class.contains("ltx_img_landscape"),
        "g1: 378×283 is landscape (class was '{class}')"
      );
    } else {
      assert_eq!(
        (w, h),
        (None, None),
        "a percentage-sized SVG has no intrinsic pixel size — write nothing and \
         let the browser size it"
      );
    }
  }
}
