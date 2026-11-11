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
//! the emission's per-creator loop; two cut calls before one `\affil` (its run summed over both, rewritten once) and two
//! spelled alike (each matched to its own queue entry). Each asserts the exact Error and Warning counts and every
//! creator's name and contacts in order. A call whose ink is not all inside its `\and`'s group stays whole (ink before
//! the group, ink after a font's group), and each piece keeps the call's keyvals (`role="editor"`), each its own test.

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
    // (two cut calls before one `\affil`: its run counts both calls' pieces, summed before it is rewritten once)
    (
      row!(
        "sectioning-frontmatter",
        "authblk_two_macro_held_calls_before_one_affil"
      ),
      (0, 4),
      &[
        &["Ann Able", "affiliation=Univ X"][..],
        &["Bob Baker", "affiliation=Univ X"][..],
        &["Cy Cole", "affiliation=Univ X"][..],
        &["Dee Dunn", "affiliation=Univ X"][..],
      ][..],
    ),
    // (two cut calls spelled alike: each matched to its own queue entry)
    (
      row!(
        "sectioning-frontmatter",
        "authblk_two_alike_macro_held_calls_each_own_affil"
      ),
      (0, 5),
      &[
        &["Ann Able", "affiliation=Univ X"][..],
        &["Bob Baker", "affiliation=Univ X"][..],
        &["Ann Able", "affiliation=Univ Y"][..],
        &["Bob Baker", "affiliation=Univ Y"][..],
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

/// A call whose ink is not all inside its `\and`'s group stays whole, its merge counted (`merged_creators`): a piece cut
/// there would print the ink outside the group once a piece, inventing names (revOC1c B1). Ink before the group on the
/// token path, ink after a font's group on the text path.
#[test]
fn an_api_creator_call_with_ink_outside_its_group_stays_whole() {
  for ((name, tex), expected) in [
    (
      row!(
        "class-bindings",
        "acmart_author_and_in_a_group_with_ink_before_stays_whole"
      ),
      "Ann Able and Bob Baker",
    ),
    (
      row!(
        "class-bindings",
        "acmart_author_and_in_a_font_with_ink_after_stays_whole"
      ),
      "Ann Able and Bob Baker Jr.",
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (1, 0),
      "{name}: {log}"
    );
    assert_eq!(
      log.matches("Error:frontmatter:merged_creators").count(),
      1,
      "{name}: {log}"
    );
    assert_eq!(
      creators(&xml),
      vec![vec![expected.to_string()]],
      "{name}: {xml}"
    );
  }
}

/// Each piece of a cut call keeps the call's keyvals: acmart's `\editor{Ann Able \and Bob Baker}`, two creators of
/// `role="editor"` beside the author's `role="author"` (revOC1c S2; `creators` compares names and contacts only).
#[test]
fn each_piece_of_a_cut_call_keeps_its_keyvals() {
  let (name, tex) = row!(
    "class-bindings",
    "acmart_editor_and_parts_the_names_keeps_the_role"
  );
  let (log, xml) = run(tex);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 0),
    "{name}: {log}"
  );
  assert_eq!(
    creators(&xml),
    vec![
      vec!["Cy Cole".to_string()],
      vec!["Ann Able".to_string()],
      vec!["Bob Baker".to_string()]
    ],
    "{name}: {xml}"
  );
  let roles: Vec<&str> = xml
    .split("<creator ")
    .skip(1)
    .filter_map(|creator| creator.split("role=\"").nth(1)?.split('"').next())
    .collect();
  assert_eq!(roles, ["author", "editor", "editor"], "{name}: {xml}");
}
