//! Guards for OC1c, Option C's step P1 on the API route (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md` "Segmentation
//! is digested"; OXIDIZED_DESIGN_DIVERGENCES #993): a binding that hands one whole argument to the API makes one creator
//! call holding the `\and` — acmart's `\author` with no comma, authblk's where its spelling test finds no separator —
//! and that call parts its authors as pdflatex does. The reading cuts it at the `Sep(And)` at its own level, inside its
//! `\lx@personname` (`resolver::api::creator_call_pieces`), and the emitter writes one creator call per piece
//! (`emit::creator_call_cut`): by the tokens between the `\and`s where the `\and`'s own tokens part no ink, by each
//! name's text where a macro holds them together (one counted Warning a piece).
//!
//! Each row is a distinct path: no author block, the queue route (`emit::cut_creator_calls`) by tokens — a literal
//! `\and`, a `\csname`, an active character, a document wrapper, each `\and` spelled by other tokens — and by text (a
//! macro); authblk's spelling-test fallback, whose `\affil` run the cut widens (`run_cap`); and beside an author block,
//! the emission's per-creator loop. Each asserts the exact Error and Warning counts and every creator's name and
//! contacts in order.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// A repro and its name, for a table's failure message to say which row broke.
macro_rules! row {
  ($dir:literal, $name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/",
        $dir,
        "/",
        $name,
        ".tex"
      )),
    )
  };
}

/// An API creator call parts at its own `\and`, on each route that makes one.
#[test]
fn an_api_creator_call_parts_at_its_own_and() {
  let both = &[&["Ann Able"][..], &["Bob Baker"][..]][..];
  for ((name, tex), counts, expected) in [
    (
      row!("class-bindings", "acmart_author_and_parts_the_names"),
      (0, 0),
      both,
    ),
    (
      row!("class-bindings", "acmart_author_csname_and_parts_the_names"),
      (0, 0),
      both,
    ),
    (
      row!("class-bindings", "acmart_author_active_and_parts_the_names"),
      (0, 0),
      both,
    ),
    (
      row!(
        "class-bindings",
        "acmart_author_wrapper_and_parts_the_names"
      ),
      (0, 0),
      both,
    ),
    (
      row!(
        "class-bindings",
        "acmart_author_macro_held_and_parts_the_names"
      ),
      (0, 2),
      both,
    ),
    (
      row!(
        "sectioning-frontmatter",
        "authblk_author_macro_held_and_parts_the_names"
      ),
      (0, 2),
      &[
        &["Cy Cole", "affiliation=Univ C"][..],
        &["Ann Able", "affiliation=Univ X", "affiliation=Univ Y"][..],
        &["Bob Baker", "affiliation=Univ X", "affiliation=Univ Y"][..],
        &["Dee Dunn", "affiliation=Univ D"][..],
      ][..],
    ),
    (
      row!(
        "class-bindings",
        "acmart_author_and_call_beside_a_block_parts_the_names"
      ),
      (0, 0),
      &[
        &["Cy Cole"][..],
        &["Dee Dunn"][..],
        &["Ann Able"][..],
        &["Bob Baker", "email=bob@y.org"][..],
        &["Eve Egan", "email=eve@z.org"][..],
      ][..],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
  }
}
