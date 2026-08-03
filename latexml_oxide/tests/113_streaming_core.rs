//! The streaming (fragmented) core pipeline — the sprint's parity gate.
//!
//! Streaming mode interleaves digest→build in fragments, spills closed
//! subtrees to disk, finishes them in a second, streaming pass, and splices
//! the processed segments back at their placeholders during serialization.
//! Perl is strictly digest-all→build-all, so this is a sanctioned divergence
//! (OXIDIZED_DESIGN) that must be INVISIBLE in the output: byte-identical
//! XML, or it cannot ship.
//!
//! Both assertions matter, in this order:
//!   1. the streaming machinery actually ENGAGED (yields + spills happened —
//!      a predicate that never fires would pass any comparison vacuously,
//!      exactly the false-negative the signal-integrity rule forbids);
//!   2. the output is byte-identical to the eager conversion.
//!
//! The fixture spans the constructs the edge-case catalog calls out:
//! sectioning at three depths (the spill spine), labels + refs across
//! fragments, display math with equation numbers, a tabular alignment, a
//! footnote pair inside one paragraph, verbatim, a group crossing content,
//! and non-ASCII text.

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

/// Convert in a fresh thread (State is thread-local), eager or streaming.
/// Returns (xml, yields, spilled_segments).
fn convert(source: &str, streaming: Option<usize>) -> (String, usize, usize) {
  let source = source.to_string();
  std::thread::Builder::new()
    .stack_size(64 * 1024 * 1024)
    .spawn(move || {
      latexml::util::test::init_test_rss_cap();
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let cfg = Config {
        format: OutputFormat::XML,
        streaming,
        // The contrib dispatcher the binaries install (see cluster/mod.rs):
        // without it, contrib-provided bindings resolve to nothing here while
        // working in production, and a fixture needing one degrades — in
        // MODE-DEPENDENT shapes, which reads as a phantom streaming
        // divergence (witness omnibus_natbib_bbl_sideload: 101 errors, an
        // empty eager doc, a partial streamed one).
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut c = Converter::from_config(cfg);
      c.initialize_session().expect("initialize");
      let r = c.convert(source.clone());
      let errors = latexml::util::test::error_count(&r.log);
      assert_eq!(
        errors, 0,
        "{source}: {errors} errors (streaming={streaming:?}):\n{}",
        r.log
      );
      let outcome = (
        r.result.expect("conversion produced XML"),
        latexml_core::stomach::fragment_yield_count(),
        latexml_core::document::spilled_segment_count(),
      );
      // See 114_streaming_corpus: free the thread's engine before exit.
      latexml_core::reset_thread_engine();
      outcome
    })
    .expect("spawn conversion thread")
    .join()
    .expect("conversion thread panicked")
}

/// The math-dense variant of the byte-identity gate: many formulas across
/// yield seams — chained ambiguous relations, elisions, XMDual-heavy
/// fractions/arrays, and `\label`/`\ref` between fragments (the idstore/XMRef
/// path). Guards the per-segment `parse_math` lifecycle: the deferred-discard
/// queue and idstore snapshot must be empty at every healthy segment
/// boundary, so the stale-state entry sweep (`sweep_stale_math_state`, the
/// pooled-worker dead-docref panic fix) stays a no-op and cannot touch memory
/// a later fragment's parse still needs.
#[test]
fn streaming_math_dense_is_byte_identical_to_eager() {
  let source = "tests/streaming/math_dense.tex";

  let (eager_xml, eager_yields, eager_spills) = convert(source, None);
  assert_eq!(eager_yields, 0, "eager must not yield");
  assert_eq!(eager_spills, 0, "eager must not spill");

  let (streamed_xml, yields, spills) = convert(source, Some(3));
  assert!(yields > 0, "streaming must actually yield");
  assert!(spills > 0, "streaming must actually spill closed subtrees");
  assert_eq!(
    eager_xml, streamed_xml,
    "math-dense streaming output must be byte-identical to eager"
  );
}

#[test]
fn streaming_is_byte_identical_to_eager() {
  let source = "tests/streaming/streaming_gate.tex";

  let (eager_xml, eager_yields, eager_spills) = convert(source, None);
  assert_eq!(eager_yields, 0, "eager must not yield");
  assert_eq!(eager_spills, 0, "eager must not spill");

  let (streamed_xml, yields, spills) = convert(source, Some(3));
  assert!(yields > 0, "streaming must actually yield");
  assert!(spills > 0, "streaming must actually spill closed subtrees");
  if eager_xml != streamed_xml {
    // Locate the first divergence for a diagnosable failure.
    let byte = eager_xml
      .bytes()
      .zip(streamed_xml.bytes())
      .position(|(a, b)| a != b)
      .unwrap_or_else(|| eager_xml.len().min(streamed_xml.len()));
    let lo = byte.saturating_sub(200);
    panic!(
      "streaming output diverges from eager at byte {byte}\n--- eager ---\n{}\n--- streamed ---\n{}",
      &eager_xml[lo..(byte + 200).min(eager_xml.len())],
      &streamed_xml[lo..(byte + 200).min(streamed_xml.len())],
    );
  }
}
