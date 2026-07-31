//! The soft-RSS yield branch must not fire with nothing accumulated.
//!
//! `digest_next_body`'s yield predicate ORs the box budget with a soft-RSS
//! test — and that test is a LEVEL test (`rss > watermark`) with no
//! hysteresis. A document whose irreducible resident floor sits above the
//! watermark latches it on for the whole run and yields at EVERY legal seam,
//! accumulating almost nothing between yields.
//!
//! Measured on the 131 MB witness at `--max-memory 48000` (watermark = fuse/3
//! = 12 GB, pass-1 RSS 13.3-14.9 GB — above it throughout): 24,051,712 yields
//! producing 459,579 segments averaging 5.5 KB, against a ~2.0 M-box budget
//! that alone would have yielded ~12 times. The same binary on a witness that
//! never crosses its watermark yields 8 times.
//!
//! What this pins:
//!   * the degenerate regime is REAL — with the floor at 1, a latched
//!     soft-RSS trigger yields far more than the box budget alone would;
//!   * a floor collapses it — same latched trigger, floor at N, yields drop
//!     by roughly N;
//!   * neither regime changes the output.
//!
//! The `114_streaming_*` sweeps cannot cover this: they run a 3-box budget, so
//! `accumulated >= budget` always wins and the soft branch is never consulted.

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

/// Convert with the soft-RSS trigger LATCHED ON (threshold 1 KiB — any real
/// process is above it) and the box budget set so high it can never fire, so
/// the soft branch alone drives yielding. Returns (xml, yields).
fn convert_soft_latched(source: &str, floor: usize) -> (String, usize) {
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
      // Budget high enough that the box-count branch can never win, so the
      // soft-RSS branch is the only thing that can yield.
      latexml_core::stomach::set_fragment_yield_budget(Some(usize::MAX));
      latexml_core::stomach::set_fragment_yield_rss_soft_kb(Some(1));
      latexml_core::stomach::set_soft_yield_min_boxes(floor);
      let r = c.convert(source.clone());
      let yields = latexml_core::stomach::fragment_yield_count();
      let outcome = (r.result.expect("conversion produced XML"), yields);
      latexml_core::reset_thread_engine();
      outcome
    })
    .expect("spawn conversion thread")
    .join()
    .expect("conversion thread panicked")
}

/// Eager reference: no budget, no soft trigger.
fn convert_eager(source: &str) -> String {
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
      let r = c.convert(source.clone());
      let xml = r.result.expect("conversion produced XML");
      latexml_core::reset_thread_engine();
      xml
    })
    .expect("spawn conversion thread")
    .join()
    .expect("conversion thread panicked")
}

#[test]
fn soft_rss_yield_needs_a_floor() {
  let source = "tests/streaming/yield_seams.tex";
  let eager_xml = convert_eager(source);

  // Floor 1 == the pre-fix behaviour: the latched trigger fires at every
  // legal seam. This is the degenerate regime the witness hit.
  let (degenerate_xml, degenerate_yields) = convert_soft_latched(source, 1);
  assert!(
    degenerate_yields > 0,
    "a latched soft-RSS trigger with no floor must yield — if this is 0 the \
     test is not exercising the soft branch at all and proves nothing"
  );

  // A floor large enough to swallow this fixture entirely: the soft branch
  // can never reach it, so yielding collapses.
  let (floored_xml, floored_yields) = convert_soft_latched(source, usize::MAX);
  assert_eq!(
    floored_yields, 0,
    "with an unreachable floor the soft branch must never fire (got \
     {floored_yields} yields; degenerate regime gave {degenerate_yields})"
  );
  assert!(
    degenerate_yields > floored_yields,
    "the floor must actually suppress yields: {degenerate_yields} -> \
     {floored_yields}"
  );

  // Neither regime may change the output.
  assert_eq!(
    eager_xml, degenerate_xml,
    "latched soft-RSS yielding must be invisible in the output"
  );
  assert_eq!(
    eager_xml, floored_xml,
    "flooring the soft-RSS branch must be invisible in the output"
  );
}
