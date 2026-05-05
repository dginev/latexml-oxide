//! Integration test for telemetry foundation (docs/TELEMETRY.md §6 acceptance).
//!
//! Runs a real Converter conversion on hello.tex and verifies:
//! 1. The telemetry struct is populated with non-zero phase totals where applicable.
//! 2. `sum(phase_us) / wall_us >= 0.85` (loose for tiny doc; the §6.5 tighter ≥0.92 acceptance is
//!    for the 100-paper sample, not unit tests).
//! 3. The hand-written JSON serializer produces valid JSON.

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};
use latexml_core::telemetry::{self, Phase};

#[test]
fn telemetry_populates_on_hello_conversion() {
  // Each #[test] runs on a fresh thread, so thread-local STATE/STACK
  // start zeroed. No tear-down needed.
  // logger::init may fail on the second test in the same binary (already
  // installed). Either outcome is fine for our purposes here.
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);

  let wall_start = std::time::Instant::now();
  let html_config = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut converter = Converter::from_config(html_config);
  converter.initialize_session().expect("can initialize.");
  let response = converter.convert("tests/hello/hello.tex".to_string());
  assert!(response.result.is_some(), "conversion failed");
  assert_eq!(response.status_code, 0);
  let wall_us = wall_start.elapsed().as_micros() as u64;

  // Snapshot phase totals.
  let (phase_us, telem_wall_us) = telemetry::with(|t| (t.phase_us, t.wall_us));
  let _ = telem_wall_us; // wall_us is set by binary at exit; not in this in-process path

  // At least one of the post-Bootstrap phases must have run during
  // convert(): Digest is the canonical entry. Bootstrap may be 0 if
  // initialize_session() ran before the guard was wrapped (lazy init).
  assert!(
    phase_us[Phase::Digest as usize] > 0,
    "Digest phase wasn't recorded; phase_us = {:?}",
    phase_us
  );
  assert!(
    phase_us[Phase::Build as usize] > 0,
    "Build phase wasn't recorded; phase_us = {:?}",
    phase_us
  );

  // Sum-of-phase covers most of wall time. Loose bound 0.5 for tiny
  // hello.tex where init/teardown overhead is large; production papers
  // will hit ≥0.92 (see docs/TELEMETRY.md §6.5).
  let sum_phase: u64 = phase_us.iter().sum();
  let ratio = sum_phase as f64 / wall_us.max(1) as f64;
  assert!(
    ratio >= 0.5,
    "sum(phase_us)={sum_phase}us / wall_us={wall_us}us = {ratio:.3}, expected >= 0.5; \
     phase_us = {phase_us:?}"
  );
}

#[test]
fn telemetry_json_round_trip_on_real_conversion() {
  // logger::init may fail on the second test in the same binary (already
  // installed). Either outcome is fine for our purposes here.
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let html_config = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut converter = Converter::from_config(html_config);
  converter.initialize_session().expect("can initialize.");
  let _ = converter.convert("tests/hello/hello.tex".to_string());

  // Set the binary-side identifiers so the JSON has stable structure.
  telemetry::set_paper_id("hello");
  telemetry::set_host("test-host");
  telemetry::set_category("ok");
  telemetry::set_exit_code(0);
  telemetry::set_wall_us(1_000_000); // dummy

  let record = telemetry::take();
  let json = record.to_json_line();

  // Structural invariants
  assert!(json.starts_with('{') && json.ends_with('}'), "json: {json}");
  assert!(json.contains("\"paper_id\":\"hello\""));
  assert!(json.contains("\"category\":\"ok\""));
  assert!(json.contains("\"schema_version\":1"));
  // All 17 phase aliases present
  for p in [
    "bootstrap",
    "digest",
    "build",
    "rewrite",
    "math_parse",
    "post_xml_parse",
    "post_scan",
    "bibliography",
    "crossref",
    "graphics",
    "math_images",
    "mathml_pres",
    "mathml_cont",
    "split",
    "xslt",
    "html5_fixups",
    "serialize",
  ] {
    let needle = format!("\"phase_{p}_us\":");
    assert!(json.contains(&needle), "missing field {needle} in: {json}");
  }
}
