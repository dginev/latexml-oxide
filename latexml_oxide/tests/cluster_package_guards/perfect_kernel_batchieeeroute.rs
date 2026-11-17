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
