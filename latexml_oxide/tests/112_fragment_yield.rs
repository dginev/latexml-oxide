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
  // Every conversion here leaves a constant ~37 MB regardless of the
  // document (73.3 MB over two prose-only conversions): the fresh thread's
  // `#[thread_local]` engine roots, which are RUST memory that this test
  // binary counts only because it links no mimalloc (see
  // `latexml_core::reset_thread_engine`). The leak this guards is the
  // per-picture surplus over that floor: with a bare `unlink` the 2,000
  // transient groups here cost megabytes; freed, the surplus is well under a
  // megabyte. The exact residue is libxml2-version-dependent arena/dict memory
  // (measured 66 KB on older libxml2, ~665 KB on libxml2 2.15) — the threshold
  // is set comfortably above that band and far below the multi-megabyte leak
  // signature, so it still catches a regression without tracking libxml2's
  // allocator.
  let surplus = growth.saturating_sub(prose_growth);
  eprintln!(
    "C heap growth over two conversions: prose {prose_growth} B, pictures {growth} B, surplus {surplus} B"
  );
  assert!(
    surplus < 1536 * 1024,
    "two more picture conversions grew the C heap {surplus} bytes past the prose control: \
     removed nodes are leaking again"
  );
}

/// Split one generated book of `sections` sections (four paragraphs each) and
/// drop every page.
#[cfg(target_os = "linux")]
fn split_and_drop_book(sections: usize) -> usize {
  use latexml_post::{
    document::{PostDocument, PostDocumentOptions},
    processor::Processor,
    split::{Split, SplitNaming},
  };
  let mut xml = String::from(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document xmlns=\"http://dlmf.nist.gov/LaTeXML\" xml:id=\"Doc\">",
  );
  for s in 1..=sections {
    xml.push_str(&format!(
      "<section xml:id=\"S{s}\"><title>Section {s}</title>"
    ));
    for p in 1..=4 {
      xml.push_str(&format!(
        "<para xml:id=\"S{s}.p{p}\"><p>Paragraph {p} of section {s}: the quick brown fox jumps over the lazy dog.</p></para>"
      ));
    }
    xml.push_str("</section>");
  }
  xml.push_str("</document>");
  let options = PostDocumentOptions {
    destination: Some("/nonexistent/index.html".to_string()),
    ..PostDocumentOptions::default()
  };
  let doc = PostDocument::new_from_string(&xml, options).expect("parse the generated book");
  let mut splitter = Split::new("//ltx:section", SplitNaming::Id, false);
  let nodes = splitter.to_process(&doc);
  let docs = splitter
    .process(doc, nodes)
    .expect("split the generated book");
  docs.len()
}

/// Post's `Split` frees each page's original subtree once it is copied into
/// the page's own document. An unlinked doc-owned node is freed by nobody (the
/// same rust-libxml rule as above), so every page leaked its source subtree and
/// the whole original DOM stayed live through post: 818 MB of C heap at page
/// 1024 of a 1,600-subsection book. Splitting the same book repeatedly must
/// leave the C heap where it was. It lives here to share `SERIAL` and
/// `c_heap_in_use`.
#[cfg(target_os = "linux")]
#[test]
fn split_pages_leave_no_c_heap_residue() {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  // Warm-up: the XPath cache, the dictionary and libxml2's arenas settle.
  assert_eq!(
    split_and_drop_book(400),
    401,
    "every section must become a page"
  );
  let before = c_heap_in_use();
  for _ in 0..2 {
    assert_eq!(split_and_drop_book(400), 401);
  }
  let growth = c_heap_in_use().saturating_sub(before);
  eprintln!("C heap growth over two splits: {growth} B");
  // Leaking, the 400 sections' source subtrees cost about a megabyte per split;
  // freed, the residue is allocator noise.
  assert!(
    growth < 256 * 1024,
    "two more splits grew the C heap {growth} bytes: split pages are leaking their source subtrees"
  );
}

/// Split restores every `ltx:navigation` (unlinked before the split) into each
/// page, so a page holding one keeps its source subtree: the nav's namespaces
/// may be declared on the page (`xmlns:foo` on S1, a redeclared default on S2),
/// and freeing the page left the nav pointing at freed `xmlNs`. Navigation comes
/// only from `--input xml`. The child's glibc perturbs freed memory, so the
/// read faults on a plain build too (the CI `asan` job runs this test).
#[cfg(target_os = "linux")]
#[test]
fn split_keeps_pages_whose_namespaces_a_navigation_uses() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(
    work.path().join("navpage.xml"),
    r#"<?xml version="1.0" encoding="UTF-8"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Doc"><title>T</title>
<section xml:id="S1" xmlns:foo="urn:example:foo" inlist="toc"><title>A</title><navigation><inline-block foo:mark="m1"><text>NAVONE</text></inline-block></navigation><para xml:id="S1.p1"><p>alpha</p></para></section>
<section xml:id="S2" xmlns="http://dlmf.nist.gov/LaTeXML" inlist="toc"><title>B</title><navigation><inline-block><text>NAVTWO</text></inline-block></navigation><para xml:id="S2.p1"><p>beta</p></para></section></document>
"#,
  )
  .unwrap();
  let out = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "navpage.xml",
      "--splitat=section",
      "--format=xml",
      "--dest=index.xml",
    ])
    .env(
      "GLIBC_TUNABLES",
      "glibc.malloc.tcache_count=0:glibc.malloc.perturb=165",
    )
    .current_dir(work.path())
    .output()
    .expect("spawn latexml_oxide");
  assert!(
    out.status.success(),
    "splitting a document whose pages hold navigation failed ({:?}):\n{}",
    out.status,
    String::from_utf8_lossy(&out.stderr)
  );
  // S2's navigation follows the written S1: the streamed split cannot give
  // it to S1, so it steps aside for the whole-DOM split, which does.
  let stderr = String::from_utf8_lossy(&out.stderr);
  assert_eq!(
    stderr
      .matches("ltx:navigation appeared after the first page was written")
      .count(),
    1,
    "the streamed split must hand this document to the whole-DOM split:\n{stderr}"
  );
  assert!(!stderr.contains("Warning:"), "{stderr}");
  for page in ["index.xml", "S1.xml", "S2.xml"] {
    let xml = std::fs::read_to_string(work.path().join(page)).expect("read a split page");
    let navs: Vec<&str> = xml
      .split("<navigation>")
      .skip(1)
      .filter_map(|rest| rest.split_once("</navigation>").map(|(nav, _)| nav))
      .collect();
    assert_eq!(
      navs.len(),
      2,
      "{page} must carry both restored navigations:\n{xml}"
    );
    assert!(
      // The first gains CrossRef's links; `foo:mark`'s prefix is not pinned.
      navs[0].starts_with("<inline-block ")
        && navs[0].contains("><text>NAVONE</text></inline-block><title>")
        && navs[1] == "<inline-block><text>NAVTWO</text></inline-block>",
      "{page} must carry both navigations' content, in order:\n{xml}"
    );
  }

/// The C heap two more conversions of `source` leave past two of the prose control (the per-thread floor).
#[cfg(target_os = "linux")]
fn c_heap_surplus_over_prose(source: &str) -> (String, usize) {
  let (_, prose_growth) = c_heap_growth_over_two_conversions("tests/streaming/prose_only.tex");
  let (first, growth) = c_heap_growth_over_two_conversions(source);
  let surplus = growth.saturating_sub(prose_growth);
  eprintln!(
    "{source}: C heap growth over two conversions {growth} B, {surplus} B past the prose control"
  );
  (first, surplus)
}

/// An aligned row's `_Capture_` cells are freed once the MathFork has copied them: `add_column_to_math_fork` copies
/// every cell child into the main branch and the td, and the detached cell was left to nobody (~15 KB per cell; 9 MB
/// of the n100 generated book's residue at the start of post).
#[cfg(target_os = "linux")]
#[test]
fn mathfork_capture_cells_leave_no_c_heap_residue() {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let (first, surplus) = c_heap_surplus_over_prose("tests/streaming/align_capture_cells.tex");
  assert_eq!(
    first.matches("<MathFork").count(),
    300,
    "150 align* of two rows each fork"
  );
  assert!(
    surplus < 1536 * 1024,
    "two more align* conversions grew the C heap {surplus} bytes past the prose control: the copied `_Capture_` \
     cells are leaking again"
  );
}

/// The math parser frees the spacing hints it folds into padding (`filter_hints`); unlinked, every `\,` stayed
/// allocated for the process lifetime. Measured surplus: 20.2 MB with the hints left unfreed (base, three runs within
/// 2 KB), 0 B freed, against the 1.5 MiB threshold.
#[cfg(target_os = "linux")]
#[test]
fn folded_math_hints_leave_no_c_heap_residue() {
  let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
  let (first, surplus) = c_heap_surplus_over_prose("tests/streaming/math_hints.tex");
  assert_eq!(
    first.matches("<XMHint").count(),
    0,
    "every hint is folded into padding"
  );
  assert_eq!(
    first.matches(r#"rpadding="1.7pt""#).count(),
    300 * 13,
    "and is kept as its neighbour's padding"
  );
  assert!(
    surplus < 1536 * 1024,
    "two more hint-dense conversions grew the C heap {surplus} bytes past the prose control: the folded hints are \
     leaking again"
  );
}
