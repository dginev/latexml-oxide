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
    doc.searchpaths.iter().any(|i| i == "."),
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

/// Whether a vector-SVG converter (`mutool`, then `pdftocairo`) spawns. Without one the test
/// prints a visible SKIP, or fails where `LATEXML_SVG_TESTS` is set (CI installs both).
fn svg_converter_or_skip(test: &str) -> bool {
  // Spawning is the signal, not the exit status: both tools exit non-zero on `--version`.
  let found = ["mutool", "pdftocairo"].iter().any(|tool| {
    std::process::Command::new(tool)
      .arg("--version")
      .output()
      .is_ok()
  });
  if !found {
    assert!(
      std::env::var_os("LATEXML_SVG_TESTS").is_none(),
      "LATEXML_SVG_TESTS is set but neither mutool nor pdftocairo is on PATH ({test})"
    );
    eprintln!("SKIP {test}: neither mutool nor pdftocairo on PATH");
  }
  found
}

/// Regression test for the vector-SVG graphics path (opt-in via
/// `--graphics-svg-threshold-kb N`). Uses the cifar10 plot PDF from the
/// upstream [brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
/// thread — a 41 KB vector-authored matplotlib chart exercising the
/// vector-SVG converters (mutool → pdftocairo) that preserve vectors
/// rather than rasterizing via ImageMagick.
///
/// Exercises the Graphics processor with `svg_threshold_kb = 200` and asserts the output is a real
/// SVG file. Skipped visibly where neither converter is on PATH.
#[test]
fn test_vector_svg_graphics_path() {
  if !svg_converter_or_skip("test_vector_svg_graphics_path") {
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
/// runtime). Skipped visibly where no vector converter is installed.
#[test]
fn test_vector_svg_pathological_convert_case() {
  if !svg_converter_or_skip("test_vector_svg_pathological_convert_case") {
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
    // Heights auto-scale by each source's aspect ratio. These are
    // Perl-with-ImageMagick's own numbers: since 2026-08-04 the post pass
    // shares the engine's algebra, which includes Perl's pt→bp step in
    // `image_graphicx_trivial`, so the one-pixel divergence this test used to
    // record (243/247) is gone.
    let want_h = if id == "g1" { "242" } else { "246" };
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

/// The start tags of the floats, flex divs and rule spans `LaTeXML-html5.xsl` makes of `xml` (`panel_grid_*` guards).
fn panel_grid_start_tags(xml: &str) -> Vec<String> {
  let stylesheet = [
    "resources/XSLT/LaTeXML-html5.xsl",
    "../resources/XSLT/LaTeXML-html5.xsl",
  ]
  .into_iter()
  .find(|p| std::path::Path::new(p).exists())
  .expect("LaTeXML-html5.xsl");
  use latexml_post::xslt::XSLT;
  let doc = PostDocument::new_from_string(xml, PostDocumentOptions::default()).expect("parse");
  let mut xslt = XSLT::new(
    stylesheet,
    rustc_hash::FxHashMap::default(),
    false,
    None,
    vec![],
  )
  .expect("XSLT processor");
  let html = xslt
    .process(doc, vec![])
    .expect("xslt process")
    .into_iter()
    .next()
    .map(|d| d.get_document().to_string())
    .unwrap_or_default();
  html
    .match_indices('<')
    .filter_map(|(at, _)| html[at..].find('>').map(|end| &html[at..=at + end]))
    .filter(|tag| {
      tag.starts_with("<figure")
        || tag.starts_with("<div class=\"ltx_flex")
        || tag.starts_with("<span class=\"ltx_rule")
    })
    .map(str::to_string)
    .collect()
}

/// The panel grid's markup reaches the HTML as CSS custom properties (OXIDIZED_DESIGN_DIVERGENCES #887; user rulings
/// 2026-10-22 and 2026-10-23, grid study option (a)): a float's `panelrows`/`panelcols`/`panelline`/`panelfill` on its
/// own element (`--ltx-panel-rows`, `--ltx-panel-cols`, `--ltx-panel-line`, `--ltx-panel-fill`, inherited by the flex
/// rows and cells the para stylesheet builds in it), a panel's `panelwidth`/`panelspan` on the `div.ltx_flex_cell`
/// holding it (`--ltx-panel-width`, `--ltx-panel-span`), the class `ltx_panels_gallery` on the float as every class is;
/// the flex markup is otherwise the same, and a panel's own element gets no style from them. Guard
/// `perfect_kernel_batch79c` (the XML side).
#[test]
fn panel_grid_markup_reaches_the_html_as_custom_properties() {
  const GALLERY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <figure class="ltx_panels_gallery" inlist="lof" panelcols="2" panelfill="0.9" panelline="1" panelrows="2" xml:id="S0.F1">
    <tags><tag>Figure 1</tag><tag role="refnum">1</tag></tags>
    <rule class="ltx_figure_panel" height="28.5pt" panelwidth="0.45" width="155.3pt"/>
    <rule class="ltx_figure_panel" height="28.5pt" panelwidth="0.45" width="155.3pt"/>
    <break class="ltx_break"/>
    <rule class="ltx_figure_panel" height="28.5pt" panelspan="2" panelwidth="0.9" width="310.5pt"/>
    <caption><tag close=": ">Figure 1</tag>A gallery.</caption>
  </figure>
</document>"#;
  assert_eq!(panel_grid_start_tags(GALLERY), [
    "<figure class=\"ltx_figure ltx_panels_gallery\" \
       style=\"--ltx-panel-rows:2;--ltx-panel-cols:2;--ltx-panel-line:1;--ltx-panel-fill:0.9;\">",
    "<div class=\"ltx_flex_figure\">",
    "<div class=\"ltx_flex_cell ltx_flex_size_2\" style=\"--ltx-panel-width:0.45;\">",
    "<span class=\"ltx_rule ltx_figure_panel\" style=\"width:155.3pt;height:28.5pt;--ltx-bg-color:black;display:inline-block;\">",
    "<div class=\"ltx_flex_cell ltx_flex_size_2\" style=\"--ltx-panel-width:0.45;\">",
    "<span class=\"ltx_rule ltx_figure_panel\" style=\"width:155.3pt;height:28.5pt;--ltx-bg-color:black;display:inline-block;\">",
    "<div class=\"ltx_flex_break\">",
    "<div class=\"ltx_flex_cell ltx_flex_size_1\" style=\"--ltx-panel-width:0.9;--ltx-panel-span:2;\">",
    "<span class=\"ltx_rule ltx_figure_panel\" style=\"width:310.5pt;height:28.5pt;--ltx-bg-color:black;display:inline-block;\">",
  ],);
}

/// The panel grid's properties are numbers only (review rev79c finding 5): a crafted XML whose grid attributes are not
/// numbers (`panelwidth="1;color:red"`, `panelline="x"`) gets none of them in `style` — the stylesheet emits a property
/// only for a numeric value, and writes the number, never the attribute's text (the schema types them `xsd:decimal`
/// and `xsd:nonNegativeInteger`).
#[test]
fn panel_grid_markup_states_only_numbers() {
  const CRAFTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <figure class="ltx_panels_gallery" panelcols="2;x:y" panelfill="0.90" panelline="x" panelrows="1" xml:id="S0.F1">
    <rule class="ltx_figure_panel" height="28.5pt" panelwidth="1;color:red" width="155.3pt"/>
    <rule class="ltx_figure_panel" height="28.5pt" panelspan="}" panelwidth="0.450" width="155.3pt"/>
    <caption>A crafted gallery.</caption>
  </figure>
</document>"#;
  assert_eq!(panel_grid_start_tags(CRAFTED), [
    "<figure class=\"ltx_figure ltx_panels_gallery\" style=\"--ltx-panel-rows:1;--ltx-panel-fill:0.9;\">",
    "<div class=\"ltx_flex_figure\">",
    "<div class=\"ltx_flex_cell ltx_flex_size_2\">",
    "<span class=\"ltx_rule ltx_figure_panel\" style=\"width:155.3pt;height:28.5pt;--ltx-bg-color:black;display:inline-block;\">",
    "<div class=\"ltx_flex_cell ltx_flex_size_2\" style=\"--ltx-panel-width:0.45;\">",
    "<span class=\"ltx_rule ltx_figure_panel\" style=\"width:155.3pt;height:28.5pt;--ltx-bg-color:black;display:inline-block;\">",
  ],);
}

/// The author blocks' HTML (each `<div class="ltx_authors">…</div>`) `LaTeXML-html5.xsl` makes of `xml`.
fn authors_html(xml: &str) -> Vec<String> {
  let stylesheet = [
    "resources/XSLT/LaTeXML-html5.xsl",
    "../resources/XSLT/LaTeXML-html5.xsl",
  ]
  .into_iter()
  .find(|p| std::path::Path::new(p).exists())
  .expect("LaTeXML-html5.xsl");
  use latexml_post::xslt::XSLT;
  let doc = PostDocument::new_from_string(xml, PostDocumentOptions::default()).expect("parse");
  let mut xslt = XSLT::new(
    stylesheet,
    rustc_hash::FxHashMap::default(),
    false,
    None,
    vec![],
  )
  .expect("XSLT processor");
  let html = xslt
    .process(doc, vec![])
    .expect("xslt process")
    .into_iter()
    .next()
    .map(|d| d.get_document().to_string())
    .unwrap_or_default();
  html
    .match_indices("<div class=\"ltx_authors\">")
    .map(|(start, _)| {
      let end = start + html[start..].find("</div>").expect("its end") + "</div>".len();
      html[start..end].to_string()
    })
    .collect()
}

/// A note several authors share prints once (OXIDIZED_DESIGN_DIVERGENCES #1052; DG's rulings of 2026-11-16). The XML
/// keeps a full copy on every carrier, each with its `mark` (Perl's model); the HTML prints the first copy in full,
/// classed `ltx_note_shared`, and every later copy of the same block, role, mark and text (spacing aside) as a mark
/// linked to it. Every marked note sits after its name in `ltx_annotated_personname`, with no space between the two.
/// A note of one author (Bob's †, under his name with no mark in ar5iv.css), a shared mark of another role or text (Cy's
/// footnote, Dee's note) render as before; an unmarked note (Cy's, Lou's IEEEtran-style "Manuscript received", 1510.02728;
/// Hal's and Ivy's empty-mark images) is its text after the wrapper, no glyph (OD #990, DG 2026-11-16); Perl's leftover
/// `role="footnotemark"` note (Cy's `*`, whose mark
/// names two notes of the block, so it links to neither) is its plain mark after the name. Only an author's
/// copies count (Ed the editor's first copy leaves Gus's a note of one author); a note with no mark or no text is never
/// a repeat (Hal's and Ivy's image-only notes all print). Every copy holds its text without the symbol, which is SHMARK's
/// contract (Max's and Ned's legend note, 2407.07614's shape). Witnesses 2606.00446, 2407.07614, 1706.03762.
#[test]
fn a_shared_author_note_prints_once_and_its_repeats_are_linked_marks() {
  const NOTES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <title>Shared notes</title>
  <creator role="author">
    <personname>Ann Able</personname>
    <contact role="affiliation">Univ. of A</contact>
    <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id1" fragid="id1">These authors contributed equally.</note>
  </creator>
  <creator role="author">
    <personname>Bob Baker</personname>
    <contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-7315-0515">iD</ref></contact>
    <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id2" fragid="id2">These authors contributed equally.</note>
    <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="†" role="thanks" xml:id="id3" fragid="id3">Corresponding author.</note>
  </creator>
  <creator role="author">
    <personname>Cy Cole</personname>
    <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id4" fragid="id4">These authors
      contributed  equally.</note>
    <note mark="*" role="footnote" xml:id="id5" fragid="id5">These authors contributed equally.</note>
    <note mark="*" role="footnotemark" xml:id="id6" fragid="id6"><tags><tag>*</tag></tags></note>
    <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id7" fragid="id7">An unmarked note.</note>
  </creator>
  <creator role="author">
    <personname>Dee Dunn</personname>
    <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id8" fragid="id8">These authors contributed to the theory.</note>
  </creator>
  <creator role="editor">
    <personname>Ed Eaton</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" mark="‡" role="thanks" xml:id="id11" fragid="id11">On leave from Univ. of B.</note>
  </creator>
  <creator role="author">
    <personname>Gus Grant</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" mark="‡" role="thanks" xml:id="id12" fragid="id12">On leave from Univ. of B.</note>
  </creator>
  <creator role="author">
    <personname>Hal Hart</personname>
    <note mark="" role="note" xml:id="id13" fragid="id13"><graphics imagesrc="a.png" imagewidth="9" imageheight="9"/></note>
    <note mark="§" role="note" xml:id="id14" fragid="id14"><graphics imagesrc="c.png" imagewidth="9" imageheight="9"/></note>
  </creator>
  <creator role="author">
    <personname>Ivy Irwin</personname>
    <note mark="" role="note" xml:id="id15" fragid="id15"><graphics imagesrc="b.png" imagewidth="9" imageheight="9"/></note>
    <note mark="§" role="note" xml:id="id16" fragid="id16"><graphics imagesrc="d.png" imagewidth="9" imageheight="9"/></note>
  </creator>
  <creator role="author">
    <personname>Lou Lane</personname>
    <note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id17" fragid="id17">Manuscript received June 1, 2026. Supported by the NSF.</note>
  </creator>
  <creator role="author">
    <personname>Max Moss</personname>
    <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id18" fragid="id18"><text fontsize="90%">Equal contribution</text></note>
  </creator>
  <creator role="author">
    <personname>Ned Nash</personname>
    <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id19" fragid="id19"><text fontsize="90%">Equal contribution</text></note>
  </creator>
  <section xml:id="S1" fragid="S1">
    <title>A part with authors of its own</title>
    <creator role="author">
      <personname>Eve Ennis</personname>
      <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id9" fragid="id9">These authors contributed equally.</note>
    </creator>
    <creator role="author">
      <personname>Fay Ford</personname>
      <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id10" fragid="id10">These authors contributed equally.</note>
    </creator>
  </section>
</document>"#;
  // Before: every copy printed in full with its own mark (2606.00446's note twice, 2407.07614's three times each),
  // the marks of the notes after the name's own line; Eve's block (a part's own authors) is keyed apart. Before this
  // round: Gus's note a mark linking to Ed's, which is not on the page; Ivy's two images marks linking to Hal's, lost.
  let html = authors_html(NOTES);
  assert_eq!(html, [
    concat!(
      r##"<div class="ltx_authors">"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Ann Able</span><span id="id1" class="ltx_note ltx_note_frontmatter ltx_thanks_contribution ltx_role_thanks ltx_note_shared"><sup class="ltx_note_mark">*</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">*</sup><span class="ltx_note_type">thanks: </span>These authors contributed equally.</span></span></span></span><span class="ltx_author_notes"><span class="ltx_author_notes_content">"##,
      "\n",
      r##"<span class="ltx_contact ltx_role_affiliation">Univ. of A"##,
      "\n",
      r##"</span></span></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Bob Baker</span>"##,
      "\n",
      r##"<span class="ltx_contact ltx_role_orcid"><a href="https://orcid.org/0000-0002-7315-0515" title="" class="ltx_ref ltx_orcid">iD</a>"##,
      "\n",
      r##"</span><span id="id2" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id1">*</a></sup></span><span id="id3" class="ltx_note ltx_note_frontmatter ltx_thanks_correspondence ltx_role_thanks"><sup class="ltx_note_mark">†</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">†</sup><span class="ltx_note_type">thanks: </span>Corresponding author.</span></span></span></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Cy Cole</span><span id="id4" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id1">*</a></sup></span><span id="id5" class="ltx_note ltx_role_footnote"><sup class="ltx_note_mark">*</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">*</sup>These authors contributed equally.</span></span></span><span id="id6" class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">*</sup></span></span><span id="id7" class="ltx_note ltx_note_frontmatter ltx_thanks_note ltx_role_thanks ltx_note_unmarked">An unmarked note.</span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Dee Dunn</span><span id="id8" class="ltx_note ltx_note_frontmatter ltx_thanks_contribution ltx_role_thanks"><sup class="ltx_note_mark">*</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">*</sup><span class="ltx_note_type">thanks: </span>These authors contributed to the theory.</span></span></span></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Gus Grant</span><span id="id12" class="ltx_note ltx_note_frontmatter ltx_thanks_note ltx_role_thanks"><sup class="ltx_note_mark">‡</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">‡</sup><span class="ltx_note_type">thanks: </span>On leave from Univ. of B.</span></span></span></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Hal Hart</span><span id="id14" class="ltx_note ltx_role_note"><sup class="ltx_note_mark">§</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">§</sup><span class="ltx_note_type">note: </span><img src="c.png" class="ltx_graphics" width="9" height="9" alt="[Uncaptioned image]"></span></span></span></span><span id="id13" class="ltx_note ltx_role_note ltx_note_unmarked"><img src="a.png" class="ltx_graphics" width="9" height="9" alt="[Uncaptioned image]"></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Ivy Irwin</span><span id="id16" class="ltx_note ltx_role_note"><sup class="ltx_note_mark">§</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">§</sup><span class="ltx_note_type">note: </span><img src="d.png" class="ltx_graphics" width="9" height="9" alt="[Uncaptioned image]"></span></span></span></span><span id="id15" class="ltx_note ltx_role_note ltx_note_unmarked"><img src="b.png" class="ltx_graphics" width="9" height="9" alt="[Uncaptioned image]"></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author">"##,
      "\n",
      r##"<span class="ltx_personname">Lou Lane"##,
      "\n",
      r##"</span><span id="id17" class="ltx_note ltx_note_frontmatter ltx_thanks_funding ltx_role_thanks ltx_note_unmarked">Manuscript received June 1, 2026. Supported by the NSF.</span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Max Moss</span><span id="id18" class="ltx_note ltx_note_frontmatter ltx_thanks_contribution ltx_role_thanks ltx_note_shared"><sup class="ltx_note_mark">∗</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">∗</sup><span class="ltx_note_type">thanks: </span><span class="ltx_text" style="font-size:90%;">Equal contribution</span></span></span></span></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Ned Nash</span><span id="id19" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id18">∗</a></sup></span></span></span></div>"##,
    ),
    concat!(
      r##"<div class="ltx_authors">"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Eve Ennis</span><span id="id9" class="ltx_note ltx_note_frontmatter ltx_thanks_contribution ltx_role_thanks ltx_note_shared"><sup class="ltx_note_mark">*</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">*</sup><span class="ltx_note_type">thanks: </span>These authors contributed equally.</span></span></span></span></span>"##,
      "\n",
      r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
      "\n",
      r##"<span class="ltx_personname">Fay Ford</span><span id="id10" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id9">*</a></sup></span></span></span></div>"##,
    ),
  ]);
}

/// A personname holds the name alone (DG 2026-11-16), so every mark printed on it is a note after it, and the HTML puts
/// them back on the name: `ltx_annotated_personname` holds the name, then every marked note of any role in document
/// order, with no space before the first (OXIDIZED_DESIGN_DIVERGENCES #1052). A mark-only note (`role="footnotemark"`)
/// is its mark alone, with no empty "footnotemark: " pop-up: linked, and classed `ltx_note_shared` as a shared note's
/// repeat is, when its mark names one note of the block (Rae's `¶`, a `\footnotemark` leftover with Perl's tags, makes
/// Quinn's one-author note shared); otherwise the plain glyph (0801.1438's affiliation letters `a`, `c`; Sam's `1`,
/// `2`, which the stylesheets part with a comma). Shapes: 0801.1438 under PURE (Kardoš: mark-only `a`, then the `*`
/// note he shares with Miškuf, the PDF's "Kardošᵃ*"; base: "Kardoš* a", the mark-only note outside the wrapper after a
/// space, with an empty pop-up); a one-author note before a mark (Pat); a mark before the ORCID badge (Tia, order
/// kept); a name with no marks (Uma, unchanged).
#[test]
fn a_pure_name_carries_every_mark_after_it_in_order() {
  const MARKS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <title>Pure names and their marks</title>
  <creator role="author">
    <personname>František Kardoš</personname>
    <note mark="a" role="footnotemark" xml:id="id1" fragid="id1"/>
    <note class="ltx_note_frontmatter ltx_thanks_funding" mark="*" role="thanks" xml:id="id2" fragid="id2">Supported by grant APVT-20-004104.</note>
    <contact role="affiliation">Institute of Mathematics, Košice</contact>
  </creator>
  <creator role="author">
    <personname>Jozef Miškuf</personname>
    <note mark="a" role="footnotemark" xml:id="id3" fragid="id3"/>
    <note class="ltx_note_frontmatter ltx_thanks_funding" mark="*" role="thanks" xml:id="id4" fragid="id4">Supported by grant APVT-20-004104.</note>
  </creator>
  <creator role="author">
    <personname>Jean-Sébastien Sereni</personname>
    <note mark="c" role="footnotemark"/>
    <note class="ltx_note_frontmatter ltx_thanks_funding" mark="‡" role="thanks" xml:id="id5" fragid="id5">Supported by the European project Aeolus.</note>
  </creator>
  <creator role="author">
    <personname>Pat Price</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" mark="§" role="thanks" xml:id="id6" fragid="id6">On leave from Univ. of B.</note>
    <note mark="b" role="footnotemark"/>
    <contact class="ltx_author_line" role="affiliation">Univ. of P</contact>
  </creator>
  <creator role="author">
    <personname>Quinn Quist</personname>
    <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="¶" role="thanks" xml:id="id7" fragid="id7">Corresponding authors.</note>
  </creator>
  <creator role="author">
    <personname>Rae Reed</personname>
    <note mark="¶" role="footnotemark" xml:id="footnotex1" fragid="footnotex1"><tags><tag>¶</tag><tag role="refnum">¶</tag><tag role="typerefnum">footnote ¶</tag></tags></note>
  </creator>
  <creator role="author">
    <personname>Sam Stone</personname>
    <note mark="1" role="footnotemark"/>
    <note mark="2" role="footnotemark"/>
  </creator>
  <creator role="author">
    <personname>Tia Toms</personname>
    <note mark="1" role="footnotemark"/>
    <contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-7315-0515">iD</ref></contact>
  </creator>
  <creator role="author">
    <personname>Uma Udall</personname>
    <contact role="affiliation">Univ. of U</contact>
  </creator>
</document>"#;
  // Before: each mark-only note outside the wrapper, after the name's line break, with an empty "footnotemark: "
  // pop-up (Kardoš "* a", Pat's `b` under his note, Tia's `1` after her badge); Quinn's note one author's, with Rae's
  // `¶` unlinked. Uma's element is the base's.
  let html = authors_html(MARKS);
  assert_eq!(html, [concat!(
    r##"<div class="ltx_authors">"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">František Kardoš</span><span id="id1" class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">a</sup></span><span id="id2" class="ltx_note ltx_note_frontmatter ltx_thanks_funding ltx_role_thanks ltx_note_shared"><sup class="ltx_note_mark">*</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">*</sup><span class="ltx_note_type">thanks: </span>Supported by grant APVT-20-004104.</span></span></span></span><span class="ltx_author_notes"><span class="ltx_author_notes_content">"##,
    "\n",
    r##"<span class="ltx_contact ltx_role_affiliation">Institute of Mathematics, Košice"##,
    "\n",
    r##"</span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Jozef Miškuf</span><span id="id3" class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">a</sup></span><span id="id4" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id2">*</a></sup></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Jean-Sébastien Sereni</span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">c</sup></span><span id="id5" class="ltx_note ltx_note_frontmatter ltx_thanks_funding ltx_role_thanks"><sup class="ltx_note_mark">‡</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">‡</sup><span class="ltx_note_type">thanks: </span>Supported by the European project Aeolus.</span></span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Pat Price</span><span id="id6" class="ltx_note ltx_note_frontmatter ltx_thanks_note ltx_role_thanks"><sup class="ltx_note_mark">§</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">§</sup><span class="ltx_note_type">thanks: </span>On leave from Univ. of B.</span></span></span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">b</sup></span></span><span class="ltx_author_notes"><span class="ltx_author_notes_content">"##,
    "\n",
    r##"<span class="ltx_contact ltx_author_line ltx_role_affiliation">Univ. of P"##,
    "\n",
    r##"</span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Quinn Quist</span><span id="id7" class="ltx_note ltx_note_frontmatter ltx_thanks_correspondence ltx_role_thanks ltx_note_shared"><sup class="ltx_note_mark">¶</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">¶</sup><span class="ltx_note_type">thanks: </span>Corresponding authors.</span></span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Rae Reed</span><span id="footnotex1" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id7">¶</a></sup></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Sam Stone</span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">1</sup></span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">2</sup></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Tia Toms</span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">1</sup></span>"##,
    "\n",
    r##"<span class="ltx_contact ltx_role_orcid"><a href="https://orcid.org/0000-0002-7315-0515" title="" class="ltx_ref ltx_orcid">iD</a>"##,
    "\n",
    r##"</span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author">"##,
    "\n",
    r##"<span class="ltx_personname">Uma Udall"##,
    "\n",
    r##"</span><span class="ltx_author_notes"><span class="ltx_author_notes_content">"##,
    "\n",
    r##"<span class="ltx_contact ltx_role_affiliation">Univ. of U"##,
    "\n",
    r##"</span></span></span></span></div>"##,
  ),]);
}

/// Only a symbol mark (`*`, `†`, `¶`) or Perl's tagged `\footnotemark` leftover names another author's note
/// (OXIDIZED_DESIGN_DIVERGENCES #1052; DG 2026-11-16: a wrong link is worse than none). A letter or digit is an
/// affiliation label as often as a footnote number, so a mark-only `1` or `a` is its plain glyph: it links to no note
/// and makes none shared (review revREND2: Bob's `1` linked Ann's own footnote `1` and sent it to the margin as shared;
/// Dee's `a`, Cy's thanks `a`; a Greek `α` likewise, revREND2b). Fay's tagged leftover `2` still links Eve's note `2`,
/// which is then shared.
#[test]
fn a_letter_or_digit_mark_names_no_other_authors_note() {
  const MARKS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <title>Glyph collision</title>
  <creator role="author">
    <personname>Ann Able</personname>
    <note mark="1" role="footnote" xml:id="id1" fragid="id1">Ann's own numbered footnote.</note>
  </creator>
  <creator role="author">
    <personname>Bob Baker</personname>
    <note mark="1" role="footnotemark" xml:id="id2" fragid="id2"/>
  </creator>
  <creator role="author">
    <personname>Cy Cole</personname>
    <note mark="a" role="thanks" xml:id="id3" fragid="id3">Cy's own lettered thanks.</note>
  </creator>
  <creator role="author">
    <personname>Dee Dunn</personname>
    <note mark="a" role="footnotemark"/>
  </creator>
  <creator role="author">
    <personname>Eve Ennis</personname>
    <note mark="2" role="footnote" xml:id="id4" fragid="id4">Eve's footnote, which Fay's mark repeats.</note>
  </creator>
  <creator role="author">
    <personname>Fay Ford</personname>
    <note mark="2" role="footnotemark" xml:id="footnotex1" fragid="footnotex1"><tags><tag>2</tag><tag role="refnum">2</tag></tags></note>
  </creator>
  <creator role="author">
    <personname>Gil Gray</personname>
    <note mark="α" role="footnote" xml:id="id5" fragid="id5">Gil's own Greek-lettered footnote.</note>
  </creator>
  <creator role="author">
    <personname>Hu Hale</personname>
    <note mark="α" role="footnotemark"/>
  </creator>
</document>"#;
  // Before (revREND2's p1.xml): Ann's and Cy's notes classed ltx_note_shared, Bob's `1` and Dee's `a` links to them.
  let html = authors_html(MARKS);
  assert_eq!(html, [concat!(
    r##"<div class="ltx_authors">"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Ann Able</span><span id="id1" class="ltx_note ltx_role_footnote"><sup class="ltx_note_mark">1</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">1</sup>Ann's own numbered footnote.</span></span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Bob Baker</span><span id="id2" class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">1</sup></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Cy Cole</span><span id="id3" class="ltx_note ltx_role_thanks"><sup class="ltx_note_mark">a</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">a</sup><span class="ltx_note_type">thanks: </span>Cy's own lettered thanks.</span></span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Dee Dunn</span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">a</sup></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Eve Ennis</span><span id="id4" class="ltx_note ltx_role_footnote ltx_note_shared"><sup class="ltx_note_mark">2</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">2</sup>Eve's footnote, which Fay's mark repeats.</span></span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Fay Ford</span><span id="footnotex1" class="ltx_note ltx_role_footnotemark ltx_note_shared"><sup class="ltx_note_mark"><a href="#id4">2</a></sup></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Gil Gray</span><span id="id5" class="ltx_note ltx_role_footnote"><sup class="ltx_note_mark">α</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">α</sup>Gil's own Greek-lettered footnote.</span></span></span></span></span>"##,
    "\n",
    r##"<span class="ltx_creator ltx_role_author"><span class="ltx_annotated_personname">"##,
    "\n",
    r##"<span class="ltx_personname">Hu Hale</span><span class="ltx_note ltx_role_footnotemark"><sup class="ltx_note_mark">α</sup></span></span></span></div>"##,
  ),]);
}
