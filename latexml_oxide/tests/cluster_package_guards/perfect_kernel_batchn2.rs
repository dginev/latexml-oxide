//! Guards for N2 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net", bar (2); OXIDIZED_DESIGN_DIVERGENCES #980):
//! the content invariant checked on the placed output (`resolver::placed`), under the switch's `=2` only, counting and never
//! answering. A word an address's domain spells is never written by the address (`a_word_beside_its_domain_is_kept`,
//! 1605.03019); an address placed more often than printed, twice on one creator or in a note and a contact
//! (`an_address_placed_more_often_than_printed_is_counted`, 2607.00684, 0807.0895); a block whose every word is placed once,
//! a shared entry and an ORCID logo among them, counts nothing (`a_clean_block_counts_nothing`).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The content check's Warnings of `log`: its lost words and its excess, each as its list (the Warning's count checked
/// against it).
pub(super) fn content_checks(log: &str) -> (Vec<String>, Vec<String>) {
  let listed = |what: &str| -> Vec<String> {
    log
      .lines()
      .filter_map(|line| {
        line
          .split_once("Frontmatter content check: ")?
          .1
          .strip_suffix('.')
      })
      .filter(|rest| rest.contains(what))
      .filter_map(|rest| {
        let (count, rest) = rest.split_once(' ')?;
        let (_, list) = rest.split_once(": ")?;
        assert_eq!(
          count.parse::<usize>().ok(),
          Some(list.split(' ').count()),
          "{rest}"
        );
        Some(list.to_string())
      })
      .collect()
  };
  (
    listed("are missing from the placed frontmatter"),
    listed("more often than the author blocks print them"),
  )
}

/// A word printed beside an address its domain spells (`IDSIA` beside `@idsia.ch`) is no address's word: the line is both
/// authors' affiliation (LO1: a line printing no address under a single names row belongs to every author of the row), each
/// address its own author's, and the check counts nothing; it never counts a domain as writing `IDSIA`. N2 alone (before
/// LO1) placed `IDSIA` nowhere and counted it lost; with the switch at 1 the check doesn't run (1605.03019, review
/// revN1r3).
#[test]
fn a_word_beside_its_domain_is_kept() {
  let tex = repro!("emit_content_a_word_beside_its_domain");
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
  assert_eq!(creators(&xml), [
    vec![
      "Ann Able".to_string(),
      "affiliation=IDSIA, 6928 Manno, Switzerland".to_string(),
      "email=ann@idsia.ch".to_string()
    ],
    vec![
      "Bob Baker".to_string(),
      "affiliation=IDSIA, 6928 Manno, Switzerland".to_string(),
      "email=bob@idsia.ch".to_string()
    ],
  ]);
  let (log, _) = run(tex, 1);
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
}

/// An address placed more often than the page prints it is counted, each e-mail contact one copy: twice on one creator
/// (2607.00684's `mtablo@postech.ac.kr`), or kept in one creator's note and written as another's contact too (2410.20285's
/// `antonis@ucsb.edu` in Örwall's note); the output as the emitter wrote it. An address a note keeps on its own owner is
/// no longer written as a contact too (0807.0895, 2208.10209; FX3): that half counts nothing.
#[test]
fn an_address_placed_more_often_than_printed_is_counted() {
  let (log, xml) = run(repro!("emit_content_an_address_placed_twice"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 4), "{log}");
  assert_eq!(
    content_checks(&log),
    (vec![], vec!["xtra@b.org".to_string()]),
    "{log}"
  );
  let row = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"]),
    row(&[
      "Bob Baker",
      "affiliation=Univ B, City",
      "email=bob.baker@b.org"
    ]),
    row(&[
      "Cat Cole",
      "affiliation=Univ B, City",
      "email=cat.cole@b.org"
    ]),
    row(&["email=xtra@b.org", "email=xtra@b.org"]),
  ]);
  let note = repro!("emit_content_an_address_a_note_keeps");
  // (FX3: a note's link the written note still prints, on the contact's own owner, is that note's text alone — Ann's
  // address no contact; FX3b: Bob's address in Ann's note stays his contact, its only tie to him, and the excess is
  // counted, as before FX3)
  let (log, xml) = run(note, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    content_checks(&log),
    (vec![], vec!["bob@b.org".to_string()]),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker", "email=bob@b.org"]),
  ]);
  assert!(
    xml.contains("Univ A, Town (<text font=\"typewriter\" xml:id=\"id1.1\">ann@a.org</text>"),
    "{xml}"
  );
  let (log, _) = run(note, 1);
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
}

/// A block whose every printed word is placed once counts nothing: an entry several creators share once (revtex's one
/// `\affiliation` under two authors, 2103.04797), an ORCID link's logo beside a name, a `\thanks`, an `\inst` list.
#[test]
fn a_clean_block_counts_nothing() {
  let revtex = "\\documentclass{revtex4-1}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\email{ann@a.org}\n\\affiliation{Univ \
                A, Town}\n\\author{Bob Baker}\n\\affiliation{Univ A, Town}\n\\author{Cat Cole}\n\\affiliation{Univ C, \
                City}\n\\maketitle\nBody.\n\\end{document}\n";
  let row = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
  for (tex, expected) in [
    (repro!("emit_content_a_clean_block_counts_nothing"), vec![
      row(&["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"]),
      row(&[
        "Bob Baker",
        "affiliation=Univ A, Town",
        "affiliation=Univ B, City",
      ]),
    ]),
    (revtex, vec![
      row(&["Ann Able", "email=ann@a.org", "affiliation=Univ A, Town"]),
      row(&["Bob Baker", "affiliation=Univ A, Town"]),
      row(&["Cat Cole", "affiliation=Univ C, City"]),
    ]),
  ] {
    let (log, xml) = run(tex, 2);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), expected, "{xml}");
  }
}
