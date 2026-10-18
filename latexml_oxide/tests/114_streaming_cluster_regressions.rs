//! Forced-streaming corpus sweep — the wide version of `113_streaming_core`.
//! Every fixture converts twice (eager, and streaming with an aggressive
//! 3-box budget); the XML must be byte-identical and both conversions error-free.
//! One suite per test binary — see `streaming_sweep/mod.rs` for why.
//!
//! `cluster_regressions` (174 fixtures) accretes ~8.3 GB of libxml2 residue in one
//! process — at the RSS fuse. It is swept in TWO shards across two binaries (this is
//! shard 0; `_b` is shard 1) so each process stays well under the cap.

mod streaming_sweep;
use streaming_sweep::sweep_dir_shard;

#[test]
fn streaming_matches_eager_on_cluster_regressions_shard0() {
  sweep_dir_shard("tests/cluster_regressions", 0, 2);
}

/// OXIDIZED_DESIGN_DIVERGENCES #738: an `\index` entry is typeset only where its index is printed — eagerly, a document
/// without `\printindex` keeps no mark; a streamed conversion's fragment yield digests the pending entries at once (the
/// anchors are built and spilled with their fragment, before a later `\printindex` could fill them), as Perl typesets
/// every entry. The one fixture the sweep lets differ for it, compared there without its `<indexmark>`s
/// (`streaming_sweep::STREAMING_DIVERGENT_FIXTURES`).
#[test]
fn streaming_yield_typesets_pending_index_entries() {
  assert_eq!(streaming_sweep::STREAMING_DIVERGENT_FIXTURES, [
    ("cluster_regressions/verb_in_index.tex", "indexmark"),
    (
      "cluster_regressions/biblatex_category_filter_timing.tex",
      "bibliography"
    ),
    (
      "cluster_regressions/biblatex_category_lists_in_newrefsection.tex",
      "bibliography"
    ),
  ]);
  let source = "tests/cluster_regressions/verb_in_index.tex";
  let (eager, eager_errors) = streaming_sweep::convert(source, None);
  let (streamed, streamed_errors) = streaming_sweep::convert(source, Some(3));
  assert_eq!((eager_errors, streamed_errors), (0, 0));
  assert_eq!(eager.matches("<indexmark").count(), 0, "{eager}");
  assert_eq!(streamed.matches("<indexmark").count(), 5, "{streamed}");
}

/// OXIDIZED_DESIGN_DIVERGENCES #762: a biblatex list filtered by a category is decided once the document is digested
/// (each entry deferred to the end, `\biblatex@select`; the list `\biblatex@if@selected` when it is built) — eagerly
/// after the whole document, so the `\addtocategory` after the list counts, as biblatex's `.aux` makes it for pdflatex
/// (Selected {web, goossens}); a streamed conversion builds the pending entries at a fragment's yield and the list when
/// it absorbs it, before that `\addtocategory`, and finds the category empty. The fixture the sweep compares without its
/// `<bibliography>`s (`streaming_sweep::STREAMING_DIVERGENT_FIXTURES`).
#[test]
fn streaming_builds_a_category_list_when_absorbed() {
  let source = "tests/cluster_regressions/biblatex_category_filter_timing.tex";
  let (eager, eager_errors) = streaming_sweep::convert(source, None);
  let (streamed, streamed_errors) = streaming_sweep::convert(source, Some(3));
  assert_eq!((eager_errors, streamed_errors), (0, 0));
  assert_eq!(eager.matches("<bibliography ").count(), 6, "{eager}");
  assert!(eager.contains("<title>Selected</title>"), "{eager}");
  assert_eq!(streamed.matches("<bibliography ").count(), 5, "{streamed}");
  assert!(!streamed.contains("<title>Selected</title>"), "{streamed}");
  // The same in a refsection: refsection 2 files knuth in the category after refsection 1's Cat1 list (77a r5), so
  // streamed, Cat1 is empty and not built — 3 lists of the eager 4 (counted, 77a r6).
  let source = "tests/cluster_regressions/biblatex_category_lists_in_newrefsection.tex";
  let (eager, eager_errors) = streaming_sweep::convert(source, None);
  let (streamed, streamed_errors) = streaming_sweep::convert(source, Some(3));
  assert_eq!((eager_errors, streamed_errors), (0, 0));
  assert_eq!(eager.matches("<bibliography ").count(), 4, "{eager}");
  assert!(
    eager.contains(r#"<title fontsize="90%">Cat1</title>"#),
    "{eager}"
  );
  assert_eq!(streamed.matches("<bibliography ").count(), 3, "{streamed}");
  assert!(!streamed.contains("Cat1</title>"), "{streamed}");
}
