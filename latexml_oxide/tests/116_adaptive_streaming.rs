//! Lever B — single-pass eager→streaming continuation (batch 56fk).
//!
//! The eager loop yields at legal seams once the fragment-yield knobs are
//! armed; when a yield is RSS-driven (the process is over the spill
//! watermark), `digest_adaptive` hands the accumulated bodies to streaming
//! pass 1 as fragment 1 and keeps digesting from the same gullet position —
//! no from-scratch `StreamingRestart`, which re-read the whole source (the
//! >180 s component on source2e/source3/datatool/glossaries/pgf-spectra LSE).
//!
//! What this pins:
//!   * the transition happens ONCE and digestion runs ONCE (`digest_setup_count`
//!     == 1; a restart would make it 2), engaging the spill (segments staged);
//!   * every picture survives the hand-off (all 40 `svg:path`), 0 errors;
//!   * a document that never crosses the watermark (control: no knobs) is
//!     built whole — no yields, no segments, identical XML.
use std::sync::Mutex;

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

static SERIAL: Mutex<()> = Mutex::new(());

/// (xml, yields, digest_setups, spilled_segments)
fn convert(source: &str, force_rss_yield: bool) -> (String, usize, usize, usize) {
  let source = source.to_string();
  std::thread::Builder::new()
    .stack_size(64 * 1024 * 1024)
    .spawn(move || {
      latexml::util::test::init_test_rss_cap();
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let cfg = Config {
        format: OutputFormat::XML,
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut c = Converter::from_config(cfg);
      c.initialize_session().expect("initialize");
      if force_rss_yield {
        // A 1 MiB spill watermark (any real process is above it) with a
        // one-box floor: the first legal seam after `\begin{document}` is an
        // RSS-driven yield and transitions. The yield knobs are NOT pre-armed —
        // the adaptive path arms them itself, exactly as a production run does.
        // The watermark is an in-process override, never the process env
        // (read-only for us).
        latexml_core::stomach::set_spill_watermark_override(Some(1 << 20));
        latexml_core::stomach::set_soft_yield_min_boxes(1);
      }
      let r = c.convert(source.clone());
      let errors = latexml::util::test::error_count(&r.log);
      assert_eq!(
        errors, 0,
        "{source}: {errors} errors (force_rss_yield={force_rss_yield}):\n{}",
        r.log
      );
      let out = (
        r.result.expect("conversion produced XML"),
        latexml_core::stomach::fragment_yield_count(),
        latexml::core_interface::digest_setup_count(),
        latexml_core::document::spilled_segment_count(),
      );
      latexml_core::stomach::set_spill_watermark_override(None);
      latexml_core::reset_thread_engine();
      out
    })
    .expect("spawn conversion thread")
    .join()
    .expect("conversion thread panicked")
}

#[test]
fn rss_driven_yield_transitions_to_streaming_in_a_single_digest() {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let source = "tests/streaming/adaptive_pictures.tex";
  let (eager_xml, eager_yields, eager_setups, eager_segments) = convert(source, false);
  assert_eq!(eager_yields, 0, "control: no knobs => no yields");
  assert_eq!(eager_segments, 0, "control: nothing spilled");
  assert_eq!(eager_setups, 1);
  assert_eq!(eager_xml.matches("<svg:path").count(), 40, "{eager_xml}");

  let (streamed_xml, yields, setups, segments) = convert(source, true);
  assert!(yields > 0, "the RSS-driven seam fired");
  assert!(segments > 0, "streaming pass 1 engaged (segments staged)");
  assert_eq!(
    setups, 1,
    "ONE digestion — the transition is a continuation, not a restart"
  );
  assert_eq!(
    streamed_xml.matches("<svg:path").count(),
    40,
    "every picture survived the hand-off:\n{streamed_xml}"
  );
  assert!(streamed_xml.contains("Done."), "{streamed_xml}");
  assert_eq!(
    streamed_xml, eager_xml,
    "eager and adaptive-streamed output are identical"
  );
}
