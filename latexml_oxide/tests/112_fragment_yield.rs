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
        // See streaming_sweep/mod.rs: match the production binding setup.
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
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
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
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

/// The end of a picture is a seam request whatever the budget
/// (`\pgfsys@endpicture` → `stomach::request_fragment_yield`): a
/// tikzpicture's boxes reach the body as ONE box, invisible to the budget's
/// count, and pgf-PeriodicTable's ~47,000-box tables ran streaming past the
/// fuse between seams (PLANS 11). With a budget far above the fixture's box
/// count, the three pictures alone force at least three yields; the output
/// stays byte-identical to the eager path.
#[test]
fn a_picture_end_is_a_seam_request() {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let source = "tests/streaming/picture_seams.tex";
  let (eager_xml, eager_yields) = convert_with_budget(source, None);
  assert_eq!(eager_yields, 0, "no budget => the eager path never yields");
  let (streamed_xml, streamed_yields) = convert_with_budget(source, Some(1_000_000));
  assert_eq!(
    streamed_yields, 3,
    "three picture ends must request exactly three yields (nothing else reaches the budget)"
  );
  assert_eq!(
    eager_xml, streamed_xml,
    "picture-seam yields must not change the XML"
  );
}

/// Bytes in use on the C heap (glibc, all arenas): libxml2's, since the Rust
/// side allocates through mimalloc.
#[cfg(target_os = "linux")]
fn c_heap_in_use() -> usize { unsafe { libc::mallinfo2() }.uordblks }

/// The heap guard reads a process-wide counter, so under `cargo test` (one
/// process, tests in parallel threads) a sibling conversion landing between
/// its two readings would dwarf the threshold; every test in this binary
/// takes the lock (nextest isolates per process anyway).
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A removed subtree is freed, not just unlinked. pgfsys wraps every graphics
/// state change in a transient `svg:g` that `collapse_svg_group` removes once
/// it proves empty; a bare `unlink` left every one of them as a doc-owned
/// orphan no free ever reached (~5-6 MB per tikz picture on pgf-PeriodicTable,
/// 742 MB of orphans by fragment 256 of its manual). Converting the same
/// 2,000-stroke document repeatedly must leave the C heap where it was.
#[cfg(target_os = "linux")]
fn c_heap_growth_over_two_conversions(source: &str) -> (String, usize) {
  // Warm-up: caches, dumps and font tables settle on the first conversion.
  let (first, _) = convert_with_budget(source, None);
  let before = c_heap_in_use();
  for _ in 0..2 {
    let (xml, _) = convert_with_budget(source, None);
    assert_eq!(xml, first, "repeated conversions must agree");
  }
  (first, c_heap_in_use().saturating_sub(before))
}

#[cfg(target_os = "linux")]
#[test]
fn removed_subtrees_leave_no_c_heap_residue() {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let (prose, prose_growth) = c_heap_growth_over_two_conversions("tests/streaming/prose_only.tex");
  assert!(
    prose.contains("<equation"),
    "the control fixture must build"
  );
  let (first, growth) = c_heap_growth_over_two_conversions("tests/streaming/picture_groups.tex");
  assert!(first.contains("<svg:path"), "the fixture must draw");
  // Every in-process conversion leaves a constant ~37 MB on the C heap
  // regardless of the document (measured 73.3 MB over two prose-only
  // conversions; a session-level residue, tracked in PLANS 11). The leak this
  // guards is the per-picture surplus over that floor: with a bare `unlink`
  // the 2,000 transient groups here cost megabytes; freed, the surplus is
  // tens of kilobytes (measured 66 KB).
  let surplus = growth.saturating_sub(prose_growth);
  eprintln!(
    "C heap growth over two conversions: prose {prose_growth} B, pictures {growth} B, surplus {surplus} B"
  );
  assert!(
    surplus < 512 * 1024,
    "two more picture conversions grew the C heap {surplus} bytes past the prose control: \
     removed nodes are leaking again"
  );
}
