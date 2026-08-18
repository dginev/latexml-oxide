//! Conversion helpers shared by the `06_cluster_*` regression suites.
//!
//! Each `tests/*.rs` is its own crate, so this is included with `mod cluster;`
//! and every suite uses only the subset it needs — hence the blanket
//! `dead_code` allow, the standard shape for a `tests/` support module.
#![allow(dead_code)]

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

pub fn convert_clean(source: &str) {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    // Same contrib dispatcher the binaries install — without it,
    // contrib-provided bindings (mhchem, chemformula, …) resolve to
    // nothing in the test environment while working in production.
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  // Shared lax `Error:<class>:` counter — see util::test::error_count
  // (single source of truth for the signal-integrity pattern).
  let n_errors = latexml::util::test::error_count(&r.log);
  assert_eq!(
    n_errors, 0,
    "{source}: expected 0 errors but log contained {n_errors} Error:<class>: markers (status_code={})",
    r.status_code
  );
  assert!(
    r.status_code <= 1,
    "{source}: status_code {} (expected 0/1), status={:?}",
    r.status_code,
    r.status
  );
}
/// Convert and return the serialized XML (for structural assertions that the
/// 0-error `convert_clean` cannot express). STRICT: like `convert_clean`, this
/// asserts the conversion logged **zero** `Error:` markers — a structural test
/// that silently tolerates a conversion error is exactly the false-negative the
/// project's signal-integrity rule forbids. An input that is *supposed* to error
/// (a malformed / EOF-truncated specimen, parity with Perl) must use
/// `convert_expecting_errors`, which asserts the exact intended count.
pub fn convert_to_xml(source: &str) -> String { convert_expecting_errors(source, 0) }
/// Convert an input EXPECTED to emit exactly `n` soft `Error:` markers, returning
/// the serialized XML. `n == 0` is the strict/clean case (`convert_to_xml`); a
/// nonzero `n` is for an intentionally-malformed specimen whose error is the
/// correct, Perl-parity outcome. Mirrors `util::test`'s `INTENTIONALLY_FAILING`
/// contract: drift fails BOTH ways — *more* errors = a handling regression,
/// *fewer* = we silently stopped detecting the bad input — and a `Fatal:`
/// (status_code 3) is always a regression, since the point is graceful recovery.
pub fn convert_expecting_errors(source: &str, n: usize) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  // Shared lax `Error:<class>:` counter — see util::test::error_count.
  let n_errors = latexml::util::test::error_count(&r.log);
  assert_eq!(
    n_errors, n,
    "{source}: expected {n} error(s) but log contained {n_errors} Error:<class>: markers (status_code={})",
    r.status_code
  );
  assert!(
    r.status_code < 3,
    "{source}: conversion hit a Fatal (status_code={}) — must degrade gracefully",
    r.status_code
  );
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}
/// Convert AND run the post-processing pipeline, returning the post-processed
/// XML. `convert_to_xml` stops at the engine, so it cannot see anything
/// MakeBibliography/CrossRef do — a `<bibitem>` in its output came straight from
/// `\begin{thebibliography}`, not from an `ltx:bibentry` conversion. Use this
/// helper for post-stage regressions.
pub fn convert_and_post(source: &str) -> String { convert_and_post_opts(source, None) }
/// Like `convert_and_post` but with the `context` navigation TOC enabled — for
/// Like [`convert_and_post`], but ALSO gates the POST stage on zero errors.
///
/// [`convert_and_post`] only inherits `convert_to_xml`'s strict CORE-stage gate.
/// The bibliography is built during post-processing (the recursive BibTeX
/// session), and `run_post_processing` returns only the XML — so a post-stage
/// error flood passed every bibliography guard silently. That is exactly the
/// false negative CLAUDE.md's signal-integrity rule forbids: a broken
/// `ltx:bib-extract` that never closed produced 17 errors on
/// `bib_abstract_percent` and 203 on witness 2605.00184 while the guard stayed
/// green, because it only asserted on text presence.
///
/// Binds the ANSI-free `LOG_BUFFER` around the post run and counts `Error:`
/// markers with the shared `util::test::error_count`.
pub fn convert_and_post_clean(source: &str) -> String {
  let xml = convert_to_xml(source);
  latexml_core::util::logger::bind_log();
  let out = post_with(&xml, None);
  let log = latexml_core::util::logger::flush_log();
  let n = latexml::util::test::error_count(&log);
  assert_eq!(
    n, 0,
    "{source}: POST stage logged {n} Error:<class>: markers\n{log}"
  );
  out
}

/// [`convert_and_post_clean`] with the contrib bindings dispatched.
///
/// biblatex lives in `latexml_contrib`, so a bibliography guard that needs it
/// must come through here: under the plain config the dispatcher has no entry
/// and raw TeX loading is off, so `\addbibresource` stays undefined and the
/// document merely reports `Warning:missing_file:biblatex`. That is what makes a
/// biblatex guard silently unable to see its own feature.
pub fn convert_and_post_contrib_clean(source: &str) -> String {
  let xml = convert_to_xml_contrib_clean(source);
  latexml_core::util::logger::bind_log();
  let out = post_with(&xml, None);
  let log = latexml_core::util::logger::flush_log();
  let n = latexml::util::test::error_count(&log);
  assert_eq!(
    n, 0,
    "{source}: POST stage logged {n} Error:<class>: markers\n{log}"
  );
  out
}

/// Like [`convert_and_post_clean`] but with presentation-MathML ENABLED, so a
/// test can assert on the generated `<m:mi>`/`<m:mo>`/… (the pmml stage the
/// bibliography-focused `post_with` disables by default). Gates on 0 POST errors.
pub fn convert_and_post_pmml_clean(source: &str) -> String {
  let xml = convert_to_xml(source);
  latexml_core::util::logger::bind_log();
  let opts = latexml::post::PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                None,
    destination:               None,
    source_directory:          Some("tests/cluster_regressions"),
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
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             false,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  };
  let out = latexml::post::run_post_processing(&xml, &opts);
  let log = latexml_core::util::logger::flush_log();
  let n = latexml::util::test::error_count(&log);
  assert_eq!(
    n, 0,
    "{source}: POST(pmml) stage logged {n} Error:<class>: markers\n{log}"
  );
  out
}

/// the upstream LaTeXML#2316 / arXiv-fork behavior where frontmatter
/// (abstract/acknowledgements/bibliography) joins the navigation TOC.
/// Like [`convert_and_post`], but returns the ANSI-free POST-stage log
/// alongside the XML instead of gating on it.
///
/// [`convert_and_post_clean`] asserts the post stage logged NO errors, which is
/// the right gate for "the bibliography built cleanly". The opposite assertion
/// — that a diagnostic WAS raised — needs the log itself, and post_with is
/// private, so a guard cannot bind `LOG_BUFFER` around it from the outside.
/// Used by the guards for dropped raises (audit family F2).
pub fn convert_and_post_logging(source: &str) -> (String, String) {
  let xml = convert_to_xml(source);
  latexml_core::util::logger::bind_log();
  let out = post_with(&xml, None);
  let log = latexml_core::util::logger::flush_log();
  (out, log)
}

pub fn convert_and_post_navtoc(source: &str) -> String {
  convert_and_post_opts(source, Some("context"))
}
pub fn convert_and_post_opts(source: &str, navigationtoc: Option<&str>) -> String {
  let xml = convert_to_xml(source);
  post_with(&xml, navigationtoc)
}

/// Run post-processing over already-converted core XML.
fn post_with(xml: &str, navigationtoc: Option<&str>) -> String {
  // No `stylesheet`: the assertions are about MakeBibliography, so stop at the
  // post-processed ltx XML rather than running XSLT into HTML.
  let opts = latexml::post::PostOptions {
    pmml: false,
    cmml: false,
    keep_xmath: false,
    stylesheet: None,
    destination: None,
    source_directory: Some("tests/cluster_regressions"),
    site_directory: None,
    search_paths: &[],
    nodefaultresources: true,
    css_files: &[],
    js_files: &[],
    noinvisibletimes: false,
    plane1: true,
    hackplane1: false,
    mathtex: false,
    url_style: latexml_post::crossref::UrlStyle::File,
    navigationtoc,
    schemadocs: false,
    split: false,
    split_xpath: None,
    split_naming: None,
    xslt_parameters: &[],
    graphics_svg_threshold_kb: 0,
    graphicimages: false,
    timestamp: None,
    icon: None,
    whatsout: latexml_post::extract::Whatsout::default(),
  };
  latexml::post::run_post_processing(xml, &opts)
}
/// Convert and return the conversion log (for asserting the ABSENCE of a
/// Rust-only warning that `convert_clean` — which only counts `Error:` — misses).
pub fn convert_log(source: &str) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  r.log
}
/// Convert with the ar5iv profile preloaded — the production route that sets
/// `bibconfig=bbl,bib` PROGRAMMATICALLY (`ar5iv_sty.rs`). It cannot be set
/// from TeX source: `\usepackage[bibconfig={bbl,bib}]{latexml}` naive-splits
/// at the comma in BOTH engines (Perl `TrimmedCommaList` is not brace-aware),
/// leaving `['bbl']`.
pub fn convert_to_xml_ar5iv(source: &str) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    preload: Some(vec!["ar5iv.sty".to_string()]),
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}
/// Convert with the contrib bindings dispatched (biblatex lives in
/// latexml_contrib) and return the serialized XML.
pub fn convert_to_xml_contrib(source: &str) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}
/// `convert_to_xml_contrib` with the strict signal-integrity gate that
/// `convert_to_xml` applies to the core helpers: zero `Error:<class>:` markers
/// and a non-fatal status. Use this for contrib regressions whose whole point
/// is that the input stops erroring — tolerating an error there is exactly the
/// false negative the project's log-parsing rule forbids.
pub fn convert_to_xml_contrib_clean(source: &str) -> String {
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  let n_errors = latexml::util::test::error_count(&r.log);
  assert_eq!(
    n_errors, 0,
    "{source}: expected 0 errors but log contained {n_errors} Error:<class>: markers (status_code={})",
    r.status_code
  );
  assert!(
    r.status_code <= 1,
    "{source}: status_code {} (expected 0/1), status={:?}",
    r.status_code,
    r.status
  );
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}
/// Convert with `INCLUDE_STYLES` (the `--includestyles` / ar5iv mode) and return
/// the log: the only way to exercise a *raw-loaded* `.sty`, which is where the
/// TeX-local-vs-global split of issue #311 actually lives. A package with a Rust
/// binding installs its definitions globally already, so a bound package cannot
/// reproduce the bug.
pub fn convert_log_includestyles(source: &str) -> String {
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    include_styles: Some(true),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  r.log
}
/// `convert_log_includestyles`'s XML sibling, for content assertions on inputs
/// that need a raw-loaded `.sty`.
pub fn convert_xml_includestyles(source: &str) -> String {
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    include_styles: Some(true),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  r.result.expect("conversion produced no result").to_string()
}
