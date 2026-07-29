//! The digestion yield for streaming (fragmented) conversion.
//!
//! Streaming pass 1 needs `digest_next_body` to be able to STOP at a legal
//! fragment seam — handing back the boxes accumulated so far, with the gullet
//! and State untouched so the next call resumes exactly where it left off —
//! instead of accumulating the whole document in one monolithic list. The
//! precedent is the alignment early-return (`stomach.rs`, Perl #2775): an
//! unread-and-return that callers already survive.
//!
//! A seam is legal only BETWEEN top-level constructs: at the entry boxing
//! depth, in vertical mode, with no open alignment, math, or conditional, and
//! only for the driver-shaped call (`terminal_opt` none). Everything else —
//! mid-paragraph, mid-list, mid-tabular, mid-equation — must digest through.
//!
//! What this pins:
//!   * with no budget set, nothing yields (the eager path is untouched);
//!   * with an aggressive budget, yields DO happen (the mechanism is real,
//!     not a silently-never-firing predicate — a fake pass the plan's
//!     fail-toward-flagging rule forbids);
//!   * the output is BYTE-IDENTICAL either way: `digest_internal`'s outer
//!     loop re-enters after every yield, so the box stream — and the XML —
//!     must not change. This is the sprint's core parity gate in miniature.

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

/// Convert `source` in a fresh thread (State is thread-local; a second
/// conversion on the same thread is not supported), with an optional
/// fragment-yield budget in boxes. Returns the serialized XML and how many
/// times digestion yielded.
fn convert_with_budget(source: &str, budget: Option<usize>) -> (String, usize) {
  let source = source.to_string();
  std::thread::Builder::new()
    // Deep TeX expansions overflow the default 2 MiB test-thread stack.
    .stack_size(64 * 1024 * 1024)
    .spawn(move || {
      latexml::util::test::init_test_rss_cap();
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let cfg = Config {
        format: OutputFormat::XML,
        ..Config::default()
      };
      let mut c = Converter::from_config(cfg);
      c.initialize_session().expect("initialize");
      latexml_core::stomach::set_fragment_yield_budget(budget);
      let r = c.convert(source.clone());
      let yields = latexml_core::stomach::fragment_yield_count();
      let errors = latexml::util::test::error_count(&r.log);
      assert_eq!(
        errors, 0,
        "{source}: {errors} errors with budget {budget:?}"
      );
      let outcome = (r.result.expect("conversion produced XML"), yields);
      // See 114_streaming_corpus: free the thread's engine before exit.
      latexml_core::reset_thread_engine();
      outcome
    })
    .expect("spawn conversion thread")
    .join()
    .expect("conversion thread panicked")
}

#[test]
fn yield_changes_nothing_but_happens() {
  let source = "tests/streaming/yield_seams.tex";

  let (eager_xml, eager_yields) = convert_with_budget(source, None);
  assert_eq!(
    eager_yields, 0,
    "no budget => the eager path must never yield"
  );

  // A 3-box budget forces many yield opportunities across the fixture's
  // paragraphs, list, tabular, group and equation.
  let (streamed_xml, streamed_yields) = convert_with_budget(source, Some(3));
  assert!(
    streamed_yields > 0,
    "an aggressive budget must actually yield (predicate never firing would \
     make streaming silently degenerate to eager)"
  );
  assert_eq!(
    eager_xml, streamed_xml,
    "yielding must be invisible in the output: same box stream, same XML"
  );
}
