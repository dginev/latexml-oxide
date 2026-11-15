//! The >2 GiB core→post handoff must not fail on libxml2's i32 buffer limit.
//!
//! `xmlReadMemory` takes its buffer length as a C `int`, so a single-invocation
//! `.tex → .htm` conversion whose serialized core XML reaches `i32::MAX` bytes
//! died with `Error:post:parse … Document too large for i32` and echoed raw
//! XML into the `.htm`. First witness: the 131 MB book's 2.68 GB core XML
//! (laptop UAT, 2026-07-31). The fix spills the oversized handoff to a temp
//! file and takes the `PostInput::File` streaming-parser arm — the path that
//! already parses that document in the two-invocation flow.
//!
//! CI cannot allocate a real >2 GiB fixture, so the spill path is driven with
//! `LATEXML_POST_MEM_PARSE_LIMIT=1` (test-only override) on a small document:
//! every handoff then takes the spill route. The success `Info!` line is the
//! engagement proof — without it a silently-failing spill would fall back to
//! the memory parse and this test would pass vacuously.

use latexml::{converter::Converter, post::PostOptions};
use latexml_core::common::{Config, OutputFormat};

fn convert_core_xml() -> String {
  let config = Config {
    format: OutputFormat::XML,
    ..Config::default()
  };
  let mut converter = Converter::from_config(config);
  converter
    .initialize_session()
    .expect("can initialize session");
  let resp = converter.convert("tests/structure/article.tex".to_string());
  resp.result.expect("conversion produced XML output")
}

fn post_opts() -> PostOptions<'static> {
  PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination:               None,
    source_directory:          Some("tests/structure"),
    site_directory:            None,
    search_paths:              &[],
    default_resources:         false,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    plane1:                    true,
    hackplane1:                false,
    mathtex:                   false,
    url_style:                 latexml_post::crossref::UrlStyle::File,
    navigationtoc:             None,
    schemadocs:                false,
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             true,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  }
}

#[test]
fn oversized_handoff_spills_and_output_is_identical() {
  // Info level: the engagement proof below is an `Info!` record, and without
  // an installed logger the capture buffer never sees it.
  let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
  let xml = convert_core_xml();

  // Baseline: the ordinary in-memory handoff.
  let baseline = latexml::post::run_post_processing(&xml, &post_opts());
  assert!(
    baseline.contains("<html"),
    "baseline post-processing must produce HTML"
  );

  // Forced-spill: every handoff is "oversized" under a 1-byte limit.
  // SAFETY: nextest runs each test in its own process; nothing else reads
  // the environment concurrently in this one.
  unsafe { std::env::set_var("LATEXML_POST_MEM_PARSE_LIMIT", "1") };
  let outcome = latexml::post::run_post_processing_logged(&xml, &post_opts());
  unsafe { std::env::remove_var("LATEXML_POST_MEM_PARSE_LIMIT") };

  // Engagement proof: the spill really happened (not a silent fallback).
  assert!(
    outcome.log.contains("oversized handoff"),
    "the spill path must engage under the test limit — log was:\n{}",
    outcome.log
  );
  // The spilled temp file must be cleaned up.
  let leftovers: Vec<_> = std::fs::read_dir(std::env::temp_dir())
    .expect("temp dir readable")
    .filter_map(|e| e.ok())
    .filter(|e| {
      e.file_name()
        .to_string_lossy()
        .starts_with(&format!("latexml-post-handoff-{}", std::process::id()))
    })
    .collect();
  assert!(
    leftovers.is_empty(),
    "spilled handoff files must be removed: {leftovers:?}"
  );
  // And the output must be byte-identical to the in-memory parse.
  assert_eq!(
    baseline, outcome.html,
    "the spill route must not change the post-processed output"
  );
}

/// An unsplit document whose sweeps both change the page: Scan names the root,
/// MakeIndex fills `\printindex`, MakeBibliography converts `lit.bib`.
const RESIDENT_TEX: &str = r#"\documentclass{article}
\usepackage{makeidx}
\makeindex
\begin{document}
\section{One}
Text\index{alpha} with a footnote\footnote{A note.} and a citation \cite{Lamport:1994}.

\begin{tabular}{|l|r|}\hline a & 1\\ \hline\end{tabular}
\[ x^2 + y_1 = \frac{a}{b} \]
\section{Two}
More\index{beta} text with entities: \& $<$ $>$ ``quotes''.
\bibliographystyle{plain}
\bibliography{lit}
\printindex
\end{document}
"#;

/// The one page of an unsplit document stays resident between post's passes
/// instead of going through disk: its render must be byte-identical to the
/// disk-staged page's (`LATEXML_POST_SPILL_PAGE=1`), through Scan's root id and
/// both sweeps' changes. Resident needs a clean parse, asserted first so the
/// comparison cannot pass vacuously.
#[test]
fn resident_page_renders_as_the_disk_staged_one() {
  let dir = tempfile::tempdir().expect("temp dir");
  let tex = dir.path().join("resident.tex");
  std::fs::write(&tex, RESIDENT_TEX).expect("write fixture");
  std::fs::copy("tests/structure/lit.bib", dir.path().join("lit.bib")).expect("copy lit.bib");
  let config = Config {
    format: OutputFormat::XML,
    ..Config::default()
  };
  let mut converter = Converter::from_config(config);
  converter
    .initialize_session()
    .expect("can initialize session");
  let xml = converter
    .convert(tex.to_string_lossy().into_owned())
    .result
    .expect("conversion produced XML output");
  let parsed = latexml_post::document::PostDocument::new_from_string(&xml, Default::default())
    .expect("core XML parses");
  assert!(
    parsed.get_document().is_well_formed() && parsed.get_document().is_ns_valid(),
    "the fixture must parse cleanly, or the page is staged on disk either way"
  );
  drop(parsed);

  let src = dir.path().to_string_lossy().into_owned();
  let opts = PostOptions {
    source_directory: Some(&src),
    ..post_opts()
  };
  let resident = latexml::post::run_post_processing_logged(&xml, &opts);
  // SAFETY: nextest runs each test in its own process; nothing else reads
  // the environment concurrently in this one.
  unsafe { std::env::set_var("LATEXML_POST_SPILL_PAGE", "1") };
  let staged = latexml::post::run_post_processing_logged(&xml, &opts);
  unsafe { std::env::remove_var("LATEXML_POST_SPILL_PAGE") };

  for needle in [
    "ltx_indexlist",
    "ltx_bibitem",
    "Lamport",
    "ltx_note",
    "ltx_tabular",
    "<math",
  ] {
    assert!(
      resident.html.contains(needle),
      "{needle} missing from the resident render:\n{}",
      resident.html
    );
  }
  assert_eq!(
    resident.html, staged.html,
    "the resident page must render as the disk-staged one"
  );
  assert_eq!(resident.log, staged.log);
}
