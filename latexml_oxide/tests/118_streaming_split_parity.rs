//! Parity gate for the streaming split front-end (`latexml_post::stream_split`).
//!
//! `STREAMING_POST_DESIGN_2026-07-06.md` §3.3 (non-negotiable): the streaming
//! split MUST produce byte-identical rendered pages to the whole-DOM split,
//! else it is a silent divergence. This fixture deliberately exercises every
//! `Split::process_pages` quirk the streaming port replicates:
//!
//! * multi-level nesting (chapter > section) and a page directly under root;
//! * adjacent page runs both with and without intervening whitespace (run
//!   grouping — ANY sibling node breaks a run);
//! * a pre-existing `ltx:TOC[@lists='toc']` (suppresses that level's
//!   generated TOC);
//! * `inlist="toc"` on one page (level-wide propagation to siblings);
//! * a back-matter *wrapper* (non-page element containing pages: a section,
//!   an appendix and a bibliography whose split arms carry
//!   `preceding-sibling` predicates) — the DOM-descent path;
//! * an id-less page (`FOO1` naming + Scan's `xml:id="Document"` mutation);
//! * template copies: `<?latexml?>` PIs, root-level `ltx:date`,
//!   `ltx:resource`, an `ltx:navigation` to excise and re-add, the root
//!   `class` merge, and inherited `xml:lang`;
//! * an *unused* extra namespace declaration on the root (the streaming path
//!   re-declares enclosing namespaces on every page root; the DOM path hoists
//!   only used ones — must be invisible in the rendered HTML);
//! * math (MathML conversion) and an `ltx:picture` (SVG extraction).
//!
//! Both paths run over the same file; every written page must be
//! byte-identical, and the streamed run must PROVE engagement via its
//! `stream-split` log marker (otherwise a silently-failing stream falls back
//! to the DOM path and this test passes vacuously).

use std::path::Path;

use latexml::post::PostOptions;

/// The `make_splitpaths("section")` union (the CLI builds this from
/// `--splitat=section`; reproduced literally here since that helper is
/// binary-private).
const SPLIT_XPATH: &str = "//ltx:section | //ltx:bibliography[preceding-sibling::ltx:section or parent::ltx:part or parent::ltx:chapter] | //ltx:appendix[preceding-sibling::ltx:section or parent::ltx:part or parent::ltx:chapter] | //ltx:index[preceding-sibling::ltx:section or parent::ltx:part or parent::ltx:chapter] | //ltx:part | //ltx:bibliography[preceding-sibling::ltx:part] | //ltx:appendix[preceding-sibling::ltx:part] | //ltx:index[preceding-sibling::ltx:part] | //ltx:chapter | //ltx:bibliography[preceding-sibling::ltx:chapter or parent::ltx:part] | //ltx:appendix[preceding-sibling::ltx:chapter or parent::ltx:part] | //ltx:index[preceding-sibling::ltx:chapter or parent::ltx:part]";

/// A hand-authored core XML exercising the split quirks. Kept structurally
/// close to real LaTeXML core output (default `ltx` namespace, `<?latexml?>`
/// PIs, `para/p` nesting, XMath).
const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml searchpaths="."?>
<?latexml class="book"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xmlns:unused="http://example.org/unused" class="ltx_root_class" xml:lang="en">
  <resource src="fake.css" type="text/css"/>
  <date role="creation">2026-07-31</date>
  <title>Parity Book</title>
  <navigation class="ltx_nav_probe"/>
  <chapter xml:id="Ch1">
    <title>Chapter One</title>
    <para xml:id="Ch1.p1"><p>Intro with math <Math mode="inline" xml:id="Ch1.m1" tex="x^2"><XMath><XMTok role="ID" xml:id="Ch1.m1.1">x</XMTok></XMath></Math>.</p></para>
    <section xml:id="Ch1.S1" labels="LABEL:sec:one"><title>Alpha</title><para xml:id="Ch1.S1.p1"><p>Alpha text.</p></para></section><section xml:id="Ch1.S2"><title>Beta</title><para xml:id="Ch1.S2.p1"><p>Beta text with <picture xml:id="Ch1.S2.pic1" width="10pt" height="10pt"><g><line points="0,0 5,5" stroke="black"/></g></picture> a picture.</p></para></section>
    <!-- a comment breaks the run -->
    <section xml:id="Ch1.S3"><title>Gamma</title><para xml:id="Ch1.S3.p1"><p>Gamma text.</p></para></section>
    <section><title>NoId</title><para xml:id="noid.p1"><p>An id-less page.</p></para></section>
  </chapter>
  <chapter xml:id="Ch2" xml:lang="de">
    <title>Chapter Two</title>
    <TOC lists="toc"/>
    <section xml:id="Ch2.S1"><title>Delta</title><para xml:id="Ch2.S1.p1"><p>Delta text.</p></para></section>
    <section xml:id="Ch2.S2" inlist="toc"><title>Epsilon</title><para xml:id="Ch2.S2.p1"><p>Epsilon text.</p></para></section>
  </chapter>
  <backmatter>
    <section xml:id="BM.S1"><title>Back Section</title><para xml:id="BM.S1.p1"><p>Backmatter section.</p></para></section>
    <appendix xml:id="A1"><title>Appendix</title><para xml:id="A1.p1"><p>Appendix text.</p></para></appendix>
    <bibliography xml:id="bib"><title>References</title><biblist xml:id="bib.L1"><bibitem xml:id="bib.item1" key="k1"><tags><tag role="refnum">1</tag></tags><bibblock>An entry.</bibblock></bibitem></biblist></bibliography>
  </backmatter>
</document>
"#;

fn post_opts<'a>(dest: &'a str, src_dir: &'a str) -> PostOptions<'a> {
  PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination:               Some(dest),
    source_directory:          Some(src_dir),
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
    split:                     true,
    split_xpath:               Some(SPLIT_XPATH.to_string()),
    split_naming:              Some("id"),
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             true,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  }
}

/// Recursively collect (relative path → bytes) for every file under `root`.
fn collect_tree(root: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
  let mut out = std::collections::BTreeMap::new();
  let mut stack = vec![root.to_path_buf()];
  while let Some(dir) = stack.pop() {
    for entry in std::fs::read_dir(&dir).expect("readable dir") {
      let entry = entry.expect("dir entry");
      let path = entry.path();
      if path.is_dir() {
        stack.push(path);
      } else {
        let rel = path
          .strip_prefix(root)
          .expect("under root")
          .to_string_lossy()
          .into_owned();
        out.insert(rel, std::fs::read(&path).expect("readable file"));
      }
    }
  }
  out
}

#[test]
fn streaming_split_pages_are_byte_identical_to_dom_split() {
  let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
  let work = tempfile::tempdir().expect("workdir");
  let source = work.path().join("book.xml");
  std::fs::write(&source, FIXTURE).expect("fixture written");
  let src_dir = work.path().to_string_lossy().into_owned();

  // Whole-DOM reference run (streaming explicitly off).
  let dom_dir = work.path().join("dom");
  std::fs::create_dir(&dom_dir).unwrap();
  let dom_dest = dom_dir.join("book.html").to_string_lossy().into_owned();
  // SAFETY: nextest gives this test its own process; env mutation is
  // sequential and single-threaded here.
  unsafe { std::env::set_var("LATEXML_POST_STREAM_SPLIT", "0") };
  let dom_outcome = latexml::post::run_post_processing_from_file_logged(
    &source.to_string_lossy(),
    &post_opts(&dom_dest, &src_dir),
  );
  assert!(
    !dom_outcome.log.contains("stream-split"),
    "the DOM reference run must not engage streaming — log:\n{}",
    dom_outcome.log
  );
  assert!(
    dom_outcome.log.contains("Split into"),
    "the DOM run must split — log:\n{}",
    dom_outcome.log
  );

  // Streamed run (forced on).
  let stream_dir = work.path().join("stream");
  std::fs::create_dir(&stream_dir).unwrap();
  let stream_dest = stream_dir.join("book.html").to_string_lossy().into_owned();
  unsafe { std::env::set_var("LATEXML_POST_STREAM_SPLIT", "1") };
  let stream_outcome = latexml::post::run_post_processing_from_file_logged(
    &source.to_string_lossy(),
    &post_opts(&stream_dest, &src_dir),
  );
  unsafe { std::env::remove_var("LATEXML_POST_STREAM_SPLIT") };

  // Engagement proof — without it a failing stream falls back to the DOM
  // path and the comparison below passes vacuously.
  assert!(
    stream_outcome.log.contains("stream-split"),
    "the streaming split must engage under LATEXML_POST_STREAM_SPLIT=1 — log:\n{}",
    stream_outcome.log
  );

  // Byte-identical page trees.
  let dom_tree = collect_tree(&dom_dir);
  let stream_tree = collect_tree(&stream_dir);
  let dom_names: Vec<&String> = dom_tree.keys().collect();
  let stream_names: Vec<&String> = stream_tree.keys().collect();
  assert_eq!(
    dom_names, stream_names,
    "both paths must write the same page set"
  );
  // root(book) + Ch1 + its (S1,S2,S3,FOO1) + Ch2 + its (S1,S2) + BM.S1 + A1 + bib = 12.
  assert_eq!(dom_names.len(), 12, "unexpected page count: {dom_names:?}");
  for (name, dom_bytes) in &dom_tree {
    let stream_bytes = &stream_tree[name];
    assert_eq!(
      String::from_utf8_lossy(dom_bytes),
      String::from_utf8_lossy(stream_bytes),
      "page '{name}' differs between the DOM and streaming splits"
    );
  }
  // And the returned main output (the root page) matches too.
  assert_eq!(
    dom_outcome.html, stream_outcome.html,
    "main output differs between the two paths"
  );

  // ---- Auto-gate: with no LATEXML_POST_STREAM_SPLIT, a tiny threshold makes
  // this fixture "large", so the size gate alone must engage the streaming
  // split. In the SAME test fn (not a sibling #[test]) because plain
  // `cargo test` runs sibling tests as threads of one process and the env
  // mutations would race; nextest isolates processes, but the guard should
  // hold under both runners.
  let auto_dir = work.path().join("auto");
  std::fs::create_dir(&auto_dir).unwrap();
  let auto_dest = auto_dir.join("book.html").to_string_lossy().into_owned();
  unsafe { std::env::set_var("LATEXML_POST_STREAM_THRESHOLD", "1") };
  let auto_outcome = latexml::post::run_post_processing_from_file_logged(
    &source.to_string_lossy(),
    &post_opts(&auto_dest, &src_dir),
  );
  unsafe { std::env::remove_var("LATEXML_POST_STREAM_THRESHOLD") };
  assert!(
    auto_outcome.log.contains("stream-split"),
    "the size gate must auto-engage the streaming split — log:\n{}",
    auto_outcome.log
  );
  assert_eq!(
    auto_outcome.html, dom_outcome.html,
    "auto-gated output must match the reference"
  );
}
