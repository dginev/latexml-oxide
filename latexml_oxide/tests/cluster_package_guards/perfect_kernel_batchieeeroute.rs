//! Guards for IEEEROUTE, IEEEtran's block notes routed by kind (`route_author_thanks`, ieeetran_cls.rs;
//! OXIDIZED_DESIGN_DIVERGENCES #1072; DG 2026-11-17): a `\thanks` in a plain `\author` saying "<names> is/are with …"
//! gives the named authors that affiliation and their e-mails, "Corresponding author: <name>" that author's
//! correspondence; any other note, or one naming no author, stays the title's. Each asserts the exact Error and Warning
//! counts, every creator's name and contacts in order, and the pubnotes whole.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batchz4::run, perfect_kernel_batchz18o::assert_creators};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The title's notes, each whole.
fn pubnotes(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<pubnote")
    .map(|(at, _)| {
      &xml[at..at + xml[at..].find("</pubnote>").expect("closed") + "</pubnote>".len()]
    })
    .collect()
}

/// 2003.11216's shape: a shared "A. Able and C. Coe are with …. E-mail: \{aable,ccoe\}@…" (the brace list split, each
/// address to the author it spells), "B. Baker is with …", "Corresponding author: Cy Coe." and a funding note — the
/// funding note alone stays the title's. Red (MTREDUCEd): four pubnotes, no author's affiliation. Distinct: the
/// sentence-per-note path, the brace list, the correspondence.
#[test]
fn ieee_is_with_notes_route_to_the_named_authors() {
  let (log, xml) = run(repro!("ieee_thanks_is_with_routes_to_the_named_authors"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &[
      "Ann Able",
      "affiliation=the Laboratory of Things, Univ A, Town 100871, Land.",
      "email=aable@unia.edu",
    ],
    &[
      "Bob Baker",
      "membership=Senior Member, IEEE",
      "affiliation=the College of Stuff, Univ B, City, Land.",
      "email=bbaker@unib.edu",
    ],
    &[
      "Cy Coe",
      "membership=Member, IEEE",
      "affiliation=the Laboratory of Things, Univ A, Town 100871, Land.",
      "email=ccoe@unia.edu",
      "correspondent=Corresponding author: Cy Coe.",
    ],
  ]);
  assert_eq!(
    pubnotes(&xml),
    [
      "<pubnote role=\"thanks\">This work was supported in part by the Science Fund under grant 42.</pubnote>"
    ],
    "{xml}"
  );
}

/// 2206.13704's shape: one note of a sentence per author, each "… , (e-mail: x@y)." Red (MTREDUCEd): one pubnote.
/// Distinct: several clauses in one note, the parenthesised label.
#[test]
fn ieee_one_note_routes_each_sentence() {
  let (log, xml) = run(repro!("ieee_thanks_one_note_one_sentence_per_author"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &[
      "Ann Able",
      "membership=Member, IEEE",
      "affiliation=Graduate School of Science, Univ A, Town 739-8527, Land",
      "email=aa@unia.jp",
    ],
    &[
      "Bob Baker",
      "affiliation=Faculty of Letters, Univ B, City 108-8345, Land",
      "email=bb@unib.jp",
    ],
  ]);
  assert_eq!(pubnotes(&xml), Vec::<&str>::new(), "{xml}");
}

/// The fail-safe: "C. Able and Q. Zed are with …" names no author (an initial not Ann Able's, a surname no one's), so
/// it stays the title's note whole, as the funding note does. Green before and after: the fail-safe keeps today's output.
#[test]
fn ieee_note_naming_no_author_stays_the_titles() {
  let (log, xml) = run(repro!("ieee_thanks_naming_no_author_stays_the_titles"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able"], &["Bob Baker"]]);
  assert_eq!(
    pubnotes(&xml),
    [
      "<pubnote role=\"thanks\">This work was supported in part by the Science Fund under grant 42.</pubnote>",
      "<pubnote role=\"thanks\">C. Able and Q. Zed are with Univ Z, Town. E-mail: {ca,qz}@z.org</pubnote>",
    ],
    "{xml}"
  );
}

/// IEEEROUTEb B1: an affiliation is one sentence and at most an e-mail tail; a funding sentence after it (1901.06512,
/// 2003.04031, 2008.12874, 2007.06695) keeps the note whole. Red (IEEEROUTE): the funding sentence Ann's affiliation.
/// Distinct: the sentence end of the affiliation.
#[test]
fn ieee_affiliation_is_one_sentence_else_the_note_stays_whole() {
  let (log, xml) = run(repro!("ieee_thanks_affiliation_one_sentence_else_whole"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able"], &["Bob Baker"], &[
    "Cy Coe",
    "affiliation=Univ C, Land.",
    "email=cc@c.org",
  ]]);
  assert_eq!(
    pubnotes(&xml),
    [
      "<pubnote role=\"thanks\">A. Able is with Univ A, Town. This work was supported by the Science Fund under grant \
       42.</pubnote>",
      "<pubnote role=\"thanks\">B. Baker is with Univ B, City (e-mail: bb@b.org). This work was supported by the Arts \
       Fund.</pubnote>",
    ],
    "{xml}"
  );
}

/// IEEEROUTEb B2a: a sentence starts only after a '.' ending no initial, so names after "C." or "J.-L." start none and
/// their note stays whole (revIEEEROUTE probes p1, p2). Red (IEEEROUTE): "Univ B, and C." Bob's affiliation, Univ H
/// Dupont-Moreau's alone. Distinct: the clause start.
#[test]
fn ieee_names_after_an_initial_keep_the_note_whole() {
  let (log, xml) = run(repro!("ieee_thanks_names_after_an_initial_stay_whole"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able", "affiliation=Univ A."],
    &["Bob Baker"],
    &["Cy Coe"],
    &["Dan Dee"],
    &["Hans van Berg"],
    &["Jean-Luc Dupont-Moreau"],
  ]);
  assert_eq!(
    pubnotes(&xml),
    [
      "<pubnote role=\"thanks\">B. Baker is with Univ B, and C. Coe is with Univ C.</pubnote>",
      "<pubnote role=\"thanks\">D. Dee is with Univ D. A. Able was with Univ Z.</pubnote>",
      "<pubnote role=\"thanks\">H. van Berg and J.-L. Dupont-Moreau are with Univ H (e-mail: {hvb,jldm}@h.org).</pubnote>",
    ],
    "{xml}"
  );
}

/// IEEEROUTEb B2c, B2b, B3: a trailing run of unlabelled addresses is the e-mail tail (1903.07552, 2109.04749), and a
/// contact goes right after its own name and grade, not past the blank line before the notes (2106.02811). Red
/// (IEEEROUTE): Dan Dee's contacts on Cy Coe too, "(dd@d.org)." inside the affiliation, a content-check Warning.
/// Distinct: the bare address run, the placement.
#[test]
fn ieee_unlabelled_addresses_route_by_the_name() {
  let (log, xml) = run(repro!("ieee_thanks_unlabelled_addresses_by_the_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &[
      "Ann Able",
      "affiliation=Univ AB, Town",
      "email=aable@ab.edu",
      "membership=Member, IEEE",
    ],
    &[
      "Bob Baker",
      "membership=Member, IEEE",
      "affiliation=Univ AB, Town",
      "email=bbaker@ab.edu",
    ],
    &[
      "Cy Coe",
      "membership=Member, IEEE",
      "affiliation=Univ C, Land",
      "email=cc@c.org",
    ],
    &[
      "Dan Dee",
      "membership=Fellow, IEEE",
      "affiliation=Univ D, Land",
      "email=dd@d.org",
    ],
  ]);
  assert_eq!(pubnotes(&xml), Vec::<&str>::new(), "{xml}");
}
