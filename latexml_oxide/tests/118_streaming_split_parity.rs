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
    default_resources:         false,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    plane1:                    true,
    hackplane1:                false,
    mathtex:                   false,
    unicodemath:               false,
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

  // ---- Default: no env at all. Every split run streams, from a file and from
  // the in-memory core handoff alike (the one-invocation `.tex → .html` path,
  // which stages the handoff to a file first), with the reference's pages.
  for (label, in_memory) in [("default-file", false), ("default-memory", true)] {
    let dir = work.path().join(label);
    std::fs::create_dir(&dir).unwrap();
    let dest = dir.join("book.html").to_string_lossy().into_owned();
    let outcome = if in_memory {
      latexml::post::run_post_processing_owned_logged(
        FIXTURE.to_string(),
        &post_opts(&dest, &src_dir),
      )
    } else {
      latexml::post::run_post_processing_from_file_logged(
        &source.to_string_lossy(),
        &post_opts(&dest, &src_dir),
      )
    };
    assert!(
      outcome.log.contains("streaming split engaged"),
      "{label}: a split run must stream by default — log:\n{}",
      outcome.log
    );
    assert_eq!(
      outcome.log.contains("streaming-split handoff"),
      in_memory,
      "{label}: only the in-memory handoff is staged — log:\n{}",
      outcome.log
    );
    assert_eq!(outcome.html, dom_outcome.html, "{label}: main output");
    assert_eq!(
      collect_tree(&dir),
      dom_tree,
      "{label}: pages differ from the whole-DOM split"
    );
  }

  // ---- Opt-out: `LATEXML_POST_STREAM_SPLIT=0` keeps an in-memory handoff in
  // memory and splits the whole DOM.
  let off_dir = work.path().join("off-memory");
  std::fs::create_dir(&off_dir).unwrap();
  let off_dest = off_dir.join("book.html").to_string_lossy().into_owned();
  unsafe { std::env::set_var("LATEXML_POST_STREAM_SPLIT", "0") };
  let off_outcome = latexml::post::run_post_processing_owned_logged(
    FIXTURE.to_string(),
    &post_opts(&off_dest, &src_dir),
  );
  unsafe { std::env::remove_var("LATEXML_POST_STREAM_SPLIT") };
  assert!(
    !off_outcome.log.contains("stream-split") && !off_outcome.log.contains("staged"),
    "the opt-out must neither stage nor stream — log:\n{}",
    off_outcome.log
  );
  assert_eq!(collect_tree(&off_dir), dom_tree, "opt-out pages");
  // No handoff file is left beside the destinations.
  for label in ["default-memory", "off-memory"] {
    let stale: Vec<_> = std::fs::read_dir(work.path().join(label))
      .unwrap()
      .filter_map(|e| e.ok())
      .filter(|e| {
        e.file_name()
          .to_string_lossy()
          .starts_with("latexml-post-handoff-")
      })
      .collect();
    assert!(stale.is_empty(), "{label}: handoff left behind: {stale:?}");
  }
}

/// The single-invocation CLI (`.tex → split .html`) takes the streaming split
/// by default — byte-identical to the whole-DOM split it replaced — while a
/// run that does not split neither stages nor streams anything.
#[test]
fn cli_split_streams_by_default_and_unsplit_stays_eager() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let mut tex = String::from("\\documentclass{article}\n\\begin{document}\n");
  for i in 1..=4 {
    tex.push_str(&format!(
      "\\section{{S{i}}}\\label{{s{i}}}\nSee \\ref{{s1}} and $x_{{{i}}}^2$.\n\n"
    ));
  }
  tex.push_str("\\end{document}\n");
  let run = |stream: Option<&str>, split: bool| {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("a.tex"), &tex).unwrap();
    let mut cmd = std::process::Command::new(bin);
    // A destination directory that does not exist yet: the handoff staged
    // beside it must create it, or the run silently stays whole-DOM.
    cmd.args(["a.tex", "--format=html5", "--dest=site/a.html"]);
    if split {
      cmd.arg("--splitat=section");
    }
    cmd
      .env_remove("LATEXML_POST_STREAM_SPLIT")
      .env_remove("LATEXML_SPILL_AT_MIB")
      .current_dir(work.path());
    if let Some(v) = stream {
      cmd.env("LATEXML_POST_STREAM_SPLIT", v);
    }
    let out = cmd.output().expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "stderr:\n{stderr}");
    assert!(
      stderr.contains("Conversion complete: No obvious problems"),
      "stderr:\n{stderr}"
    );
    let mut tree = collect_tree(work.path());
    tree.retain(|name, _| name.ends_with(".html"));
    (tree, stderr)
  };
  let (streamed, streamed_log) = run(None, true);
  let (dom, dom_log) = run(Some("0"), true);
  assert!(
    streamed_log.contains("streaming-split handoff")
      && streamed_log.contains("streaming split engaged"),
    "the split run must stage its handoff and stream:\n{streamed_log}"
  );
  // The core serialized its result into the handoff file itself: no document-sized String was built only to be
  // written there (issue 777).
  assert!(
    // The path is the platform's (`site\latexml-post-handoff-…` on Windows).
    streamed_log
      .replace('\\', "/")
      .contains("staged to site/latexml-post-handoff-")
      && streamed_log.contains(".xml by the core serializer")
      && !streamed_log.contains("could not"),
    "the split run's core result must reach post through the file:\n{streamed_log}"
  );
  assert!(
    !dom_log.contains("stream-split") && !dom_log.contains("staged"),
    "the opt-out must not stream:\n{dom_log}"
  );
  assert_eq!(
    streamed.len(),
    5,
    "root + 4 sections: {:?}",
    streamed.keys()
  );
  assert_eq!(
    streamed, dom,
    "the streamed split must match the whole-DOM split"
  );

  let (unsplit, unsplit_log) = run(None, false);
  assert_eq!(unsplit.keys().collect::<Vec<_>>(), ["site/a.html"]);
  assert!(
    !unsplit_log.contains("stream-split")
      && !unsplit_log.contains("staged")
      && !unsplit_log.contains("parallel"),
    "an unsplit run takes the eager path:\n{unsplit_log}"
  );
}

/// A bare `--dest=a.html` stages the handoff and the page spill in ".", the
/// destination's directory, never in the system temp dir (a RAM-backed tmpfs
/// on many hosts, outside the `--max-memory` accounting). An unusable TMPDIR
/// proves it: staging there would fail.
#[test]
fn cli_bare_dest_stages_beside_the_destination() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let mut tex = String::from("\\documentclass{article}\n\\begin{document}\n");
  for i in 1..=3 {
    tex.push_str(&format!("\\section{{S{i}}}\nText {i}.\n\n"));
  }
  tex.push_str("\\end{document}\n");
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("a.tex"), &tex).unwrap();
  let out = std::process::Command::new(bin)
    .args([
      "a.tex",
      "--format=html5",
      "--dest=a.html",
      "--splitat=section",
    ])
    .env_remove("LATEXML_POST_STREAM_SPLIT")
    .env_remove("LATEXML_SPILL_AT_MIB")
    .env("TMPDIR", work.path().join("no-such-tmp"))
    .current_dir(work.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert!(out.status.success(), "stderr:\n{stderr}");
  assert!(
    stderr.contains("Conversion complete: No obvious problems"),
    "stderr:\n{stderr}"
  );
  assert!(
    // The path is the platform's (`.\latexml-post-handoff-…` on Windows).
    stderr
      .replace('\\', "/")
      .contains("staged to ./latexml-post-handoff-")
      && stderr.contains("streaming split engaged"),
    "the handoff must be staged in the destination's directory and streamed:\n{stderr}"
  );
  assert!(
    !work.path().join("no-such-tmp").exists(),
    "nothing may be staged in TMPDIR"
  );
  let mut names: Vec<String> = std::fs::read_dir(work.path())
    .expect("readdir")
    .filter_map(|e| e.ok())
    .map(|e| e.file_name().to_string_lossy().into_owned())
    .filter(|n| n.ends_with(".html") || n.contains("latexml-post-"))
    .collect();
  names.sort();
  assert_eq!(
    names,
    ["S1.html", "S2.html", "S3.html", "a.html"],
    "the pages, and nothing staged left behind"
  );
}

/// An `ltx:resource` nested inside content, before any page (root content) or
/// inside one: the whole-DOM split copies the `//ltx:resource` of what remains
/// around each page into its template, the stream only the root's direct ones.
/// The default split hands such a document to the whole-DOM split (an Info):
/// its pages, no Warning. The second document holds its resource in a wrapper
/// (`ltx:backmatter` around a page), whose content the stream reads by DOM
/// descent, not as bulk.
#[test]
fn nested_resource_split_falls_back_to_the_whole_dom_split() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  const NESTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML">
<title>T</title>
<para xml:id="p0"><p>Intro</p><resource src="early.css" type="text/css"/></para>
<section xml:id="S1"><title>S1</title><para xml:id="S1.p1"><p>Text 1.</p></para></section>
<section xml:id="S2"><title>S2</title><para xml:id="S2.p1"><p>Text 2.</p><resource src="mid.css" type="text/css"/></para></section>
<section xml:id="S3"><title>S3</title><para xml:id="S3.p1"><p>Text 3.</p></para></section>
</document>
"#;
  const WRAPPED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML">
<title>T</title>
<section xml:id="S1"><title>S1</title><para xml:id="S1.p1"><p>Text 1.</p></para></section>
<backmatter><para xml:id="B.p1"><p>Back.</p><resource src="early.css" type="text/css"/></para>
<section xml:id="S2"><title>S2</title><para xml:id="S2.p1"><p>Text 2.</p></para></section></backmatter>
</document>
"#;
  let work = tempfile::tempdir().expect("workdir");
  std::fs::write(work.path().join("nest.xml"), NESTED).expect("fixture written");
  std::fs::write(work.path().join("wrap.xml"), WRAPPED).expect("fixture written");
  for css in ["early.css", "mid.css"] {
    std::fs::write(work.path().join(css), "p {}\n").expect("stylesheet written");
  }
  let run = |source: &str, label: &str, stream: Option<&str>| {
    let mut cmd = std::process::Command::new(bin);
    cmd
      .args([source, "--format=html5", "--splitat=section"])
      .arg(format!("--dest={label}/index.html"))
      .env_remove("LATEXML_POST_STREAM_SPLIT")
      .current_dir(work.path());
    if let Some(v) = stream {
      cmd.env("LATEXML_POST_STREAM_SPLIT", v);
    }
    let out = cmd.output().expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "{label}:\n{stderr}");
    assert!(
      stderr.contains("Conversion complete: No obvious problems"),
      "{label}: no Warning:\n{stderr}"
    );
    let mut tree = collect_tree(&work.path().join(label));
    tree.retain(|name, _| name.ends_with(".html"));
    (tree, stderr)
  };
  let sheets = |tree: &std::collections::BTreeMap<String, Vec<u8>>| -> Vec<(String, Vec<&str>)> {
    tree
      .iter()
      .map(|(name, page)| {
        let html = std::str::from_utf8(page).expect("utf-8 page");
        let found = ["early.css", "mid.css"]
          .into_iter()
          .filter(|css| html.contains(&format!("href=\"{css}\"")))
          .collect();
        (name.clone(), found)
      })
      .collect()
  };
  /// Each page's stylesheets, by page name.
  type Sheets<'a> = &'a [(&'a str, &'a [&'a str])];
  let expected: [(&str, Sheets); 2] = [
    ("nest.xml", &[
      ("S1.html", &["early.css"]),
      ("S2.html", &["early.css", "mid.css"]),
      ("S3.html", &["early.css"]),
      ("index.html", &["early.css"]),
    ]),
    // The whole-DOM split gives S1 no copy of the backmatter's resource.
    ("wrap.xml", &[
      ("S1.html", &[]),
      ("S2.html", &["early.css"]),
      ("index.html", &["early.css"]),
    ]),
  ];
  for (source, want) in expected {
    let stem = source.trim_end_matches(".xml");
    let (dom, _) = run(source, &format!("{stem}_dom"), Some("0"));
    let (default, default_log) = run(source, &format!("{stem}_default"), None);
    let want: Vec<(String, Vec<&str>)> = want
      .iter()
      .map(|(name, css)| (name.to_string(), css.to_vec()))
      .collect();
    assert_eq!(
      sheets(&dom),
      want,
      "{source}: the whole-DOM split's stylesheets per page"
    );
    assert_eq!(
      default_log
        .matches("a nested ltx:resource is not in the page templates")
        .count(),
      1,
      "{source}: the stream must hand this document to the whole-DOM split:\n{default_log}"
    );
    assert_eq!(
      default, dom,
      "{source}: the pages must be the whole-DOM split's"
    );
  }
}

/// The page Split extracted each page from is the parent Scan files it under (Perl Split.pm:132-133
/// `parent_id => $$entry{upid}`, Scan.pm:99), in the streamed split and the whole-DOM one alike.
/// `\@addtoreset{chapter}{part}` renumbers Part B's chapters `Ch1a`, `Ch2a` while their sections keep
/// `Ch1.S1`, `Ch2.S1a`: the id-prefix guess filed them under Part A's `Ch1`, and every chapter
/// under the site root rather than its part. Expected values are Perl 0.8.8's head links (less the
/// sibling relations OXIDIZED_DESIGN_DIVERGENCES #1134 drops), titles and nav anchors.
/// Witness: issue 777's book, 9,262 of 115,519 pages navigating into the wrong part.
#[test]
fn split_pages_navigate_under_the_parent_split_gave_them() {
  type Page = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static [(&'static str, &'static str, &'static str)],
  );
  const B1: &str = "In Chapter 1 Book1 ‣ Part II B";
  const PERF: &str = "In Chapter 2 Perf ‣ Part I A";
  const EXPECTED: &[Page] = &[
    (
      "Ch1.S1.html",
      "1.1 X1 ‣ Chapter 1 Book1 ‣ Part II B",
      &[
        ("up", "Ch1a.html"),
        ("up up", "Pt2.html"),
        ("start", "renum.html"),
        ("prev", "Ch1a.html"),
        ("next", "Ch1.S2.html"),
      ],
      &[
        ("up", "Ch1a.html", "In Part II B"),
        ("prev", "Ch1a.html", "In Part II B"),
        ("next", "Ch1.S2.html", B1),
      ],
    ),
    (
      "Ch1.S2.html",
      "1.2 X2 ‣ Chapter 1 Book1 ‣ Part II B",
      &[
        ("up", "Ch1a.html"),
        ("up up", "Pt2.html"),
        ("start", "renum.html"),
        ("prev", "Ch1.S1.html"),
        ("next", "Ch1.S3.html"),
      ],
      &[
        ("up", "Ch1a.html", "In Part II B"),
        ("prev", "Ch1.S1.html", B1),
        ("next", "Ch1.S3.html", B1),
      ],
    ),
    (
      "Ch1.S3.html",
      "1.3 X3 ‣ Chapter 1 Book1 ‣ Part II B",
      &[
        ("up", "Ch1a.html"),
        ("up up", "Pt2.html"),
        ("start", "renum.html"),
        ("prev", "Ch1.S2.html"),
        ("next", "Ch2a.html"),
      ],
      &[
        ("up", "Ch1a.html", "In Part II B"),
        ("prev", "Ch1.S2.html", B1),
        ("next", "Ch2a.html", "In Part II B"),
      ],
    ),
    (
      "Ch1.html",
      "Chapter 1 Intro ‣ Part I A",
      &[
        ("up", "Pt1.html"),
        ("start", "renum.html"),
        ("prev", "Pt1.html"),
        ("next", "Ch2.html"),
      ],
      &[
        ("up", "Pt1.html", ""),
        ("prev", "Pt1.html", ""),
        ("next", "Ch2.html", "In Part I A"),
      ],
    ),
    (
      "Ch1a.html",
      "Chapter 1 Book1 ‣ Part II B",
      &[
        ("up", "Pt2.html"),
        ("start", "renum.html"),
        ("prev", "Pt2.html"),
        ("next", "Ch1.S1.html"),
      ],
      &[
        ("up", "Pt2.html", ""),
        ("prev", "Pt2.html", ""),
        ("next", "Ch1.S1.html", B1),
      ],
    ),
    (
      "Ch2.S1.html",
      "2.1 PS one ‣ Chapter 2 Perf ‣ Part I A",
      &[
        ("up", "Ch2.html"),
        ("up up", "Pt1.html"),
        ("start", "renum.html"),
        ("prev", "Ch2.html"),
        ("next", "Ch2.S2.html"),
      ],
      &[
        ("up", "Ch2.html", "In Part I A"),
        ("prev", "Ch2.html", "In Part I A"),
        ("next", "Ch2.S2.html", PERF),
      ],
    ),
    (
      "Ch2.S1a.html",
      "2.1 Y1 ‣ Chapter 2 Book2 ‣ Part II B",
      &[
        ("up", "Ch2a.html"),
        ("up up", "Pt2.html"),
        ("start", "renum.html"),
        ("prev", "Ch2a.html"),
      ],
      &[
        ("up", "Ch2a.html", "In Part II B"),
        ("prev", "Ch2a.html", "In Part II B"),
      ],
    ),
    (
      "Ch2.S2.html",
      "2.2 PS two ‣ Chapter 2 Perf ‣ Part I A",
      &[
        ("up", "Ch2.html"),
        ("up up", "Pt1.html"),
        ("start", "renum.html"),
        ("prev", "Ch2.S1.html"),
        ("next", "Pt2.html"),
      ],
      &[
        ("up", "Ch2.html", "In Part I A"),
        ("prev", "Ch2.S1.html", PERF),
        ("next", "Pt2.html", ""),
      ],
    ),
    (
      "Ch2.html",
      "Chapter 2 Perf ‣ Part I A",
      &[
        ("up", "Pt1.html"),
        ("start", "renum.html"),
        ("prev", "Ch1.html"),
        ("next", "Ch2.S1.html"),
      ],
      &[
        ("up", "Pt1.html", ""),
        ("prev", "Ch1.html", "In Part I A"),
        ("next", "Ch2.S1.html", PERF),
      ],
    ),
    (
      "Ch2a.html",
      "Chapter 2 Book2 ‣ Part II B",
      &[
        ("up", "Pt2.html"),
        ("start", "renum.html"),
        ("prev", "Ch1.S3.html"),
        ("next", "Ch2.S1a.html"),
      ],
      &[
        ("up", "Pt2.html", ""),
        ("prev", "Ch1.S3.html", B1),
        ("next", "Ch2.S1a.html", "In Chapter 2 Book2 ‣ Part II B"),
      ],
    ),
    (
      "Pt1.html",
      "Part I A",
      &[
        ("start", "renum.html"),
        ("prev", "renum.html"),
        ("next", "Ch1.html"),
      ],
      &[
        ("prev", "renum.html", ""),
        ("next", "Ch1.html", "In Part I A"),
      ],
    ),
    (
      "Pt2.html",
      "Part II B",
      &[
        ("start", "renum.html"),
        ("prev", "Ch2.S2.html"),
        ("next", "Ch1a.html"),
      ],
      &[
        ("prev", "Ch2.S2.html", PERF),
        ("next", "Ch1a.html", "In Part II B"),
      ],
    ),
    (
      "renum.html",
      "Untitled Document",
      &[("next", "Pt1.html")],
      &[("next", "Pt1.html", "")],
    ),
  ];
  // The value of `name="…"` in `tag`, if any.
  fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    Some(&tag[at..at + tag[at..].find('"')?])
  }
  // Every `<open …>` tag of the page.
  fn tags<'a>(html: &'a str, open: &'a str) -> impl Iterator<Item = &'a str> {
    html
      .match_indices(open)
      .filter_map(|(at, _)| Some(&html[at..at + html[at..].find('>')?]))
  }
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let tex = "\\documentclass{book}\\makeatletter\\@addtoreset{chapter}{part}\\makeatother\n\
             \\begin{document}\\part{A}\\chapter{Intro}x\\chapter{Perf}\\section{PS one}a\\section{PS two}b\n\
             \\part{B}\\chapter{Book1}\\section{X1}x\\section{X2}x\\section{X3}x\\chapter{Book2}\\section{Y1}y\n\
             \\end{document}\n";
  let run = |stream: Option<&str>| {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("renum.tex"), tex).unwrap();
    let mut cmd = std::process::Command::new(bin);
    cmd
      .args([
        "renum.tex",
        "--format=html5",
        "--splitat=section",
        "--dest=renum.html",
      ])
      .env_remove("LATEXML_POST_STREAM_SPLIT")
      .current_dir(work.path());
    if let Some(v) = stream {
      cmd.env("LATEXML_POST_STREAM_SPLIT", v);
    }
    let out = cmd.output().expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
      out.status.success() && stderr.contains("Conversion complete: No obvious problems"),
      "stderr:\n{stderr}"
    );
    assert_eq!(
      stderr.contains("streaming split engaged"),
      stream.is_none(),
      "{stderr}"
    );
    let mut tree = collect_tree(work.path());
    tree.retain(|name, _| name.ends_with(".html"));
    tree
  };
  let streamed = run(None);
  let dom = run(Some("0"));
  for (label, tree) in [("streamed", &streamed), ("whole-DOM", &dom)] {
    assert_eq!(
      tree.keys().map(String::as_str).collect::<Vec<_>>(),
      EXPECTED.iter().map(|page| page.0).collect::<Vec<_>>(),
      "{label} split"
    );
    for &(page, title, head, anchors) in EXPECTED {
      let html = std::str::from_utf8(&tree[page]).unwrap();
      let got_title = html
        .split_once("<title>")
        .and_then(|(_, rest)| rest.split_once("</title>"))
        .map(|(title, _)| title);
      assert_eq!(got_title, Some(title), "{label} {page} title");
      let got_head: Vec<_> = tags(html, "<link ")
        .filter_map(|tag| Some((attr(tag, "rel")?, attr(tag, "href")?)))
        .filter(|(rel, _)| *rel != "stylesheet")
        .collect();
      assert_eq!(got_head, head, "{label} {page} head links");
      let mut got_anchors = Vec::new();
      for tag in tags(html, "<a ").filter(|tag| attr(tag, "class") == Some("ltx_ref")) {
        let (Some(rel), Some(href)) = (attr(tag, "rel"), attr(tag, "href")) else {
          continue;
        };
        let anchor = (rel, href, attr(tag, "title").unwrap_or(""));
        if !got_anchors.contains(&anchor) {
          got_anchors.push(anchor);
        }
      }
      assert_eq!(
        got_anchors, anchors,
        "{label} {page} header/footer navigation"
      );
    }
  }
  assert_eq!(
    streamed, dom,
    "the streamed split must match the whole-DOM split"
  );
}
