//! Guards for N2 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net", bar (2); OXIDIZED_DESIGN_DIVERGENCES #980):
//! the content invariant checked on the placed output (`resolver::placed`), under the switch's `=2` only, counting and never
//! answering. A word an address's domain spells is lost where no item holds it (`a_word_beside_its_domain_is_counted_lost`,
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

/// The content check's Warnings of `log`: its lost words and its excess, each as its list.
fn content_checks(log: &str) -> (Vec<String>, Vec<String>) {
  let listed = |what: &str| -> Vec<String> {
    log
      .lines()
      .filter(|line| line.contains("The frontmatter's content check") && line.contains(what))
      .filter_map(|line| {
        let open = line.rfind(" (")?;
        let close = line.rfind("); they are counted")?;
        Some(line[open + 2..close].to_string())
      })
      .collect()
  };
  (
    listed("holds nowhere"),
    listed("holds more often than they print"),
  )
}

/// A word printed beside an address its domain spells (`IDSIA` beside `@idsia.ch`) is no address's: placed nowhere, it is
/// counted lost, the line's list of local parts read whole (its inks parted at the list's commas); with the switch at 1 the
/// check doesn't run (1605.03019, review revN1r3).
#[test]
fn a_word_beside_its_domain_is_counted_lost() {
  let tex = repro!("emit_content_a_word_beside_its_domain");
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(
    content_checks(&log),
    (vec!["IDSIA".to_string()], vec![]),
    "{log}"
  );
  assert_eq!(creators(&xml)[..2], [
    vec!["Ann Able".to_string(), "email=ann@idsia.ch".to_string()],
    vec!["Bob Baker".to_string(), "email=bob@idsia.ch".to_string()],
  ]);
  let (log, _) = run(tex, 1);
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
}

/// An address placed more often than the page prints it is counted, each e-mail contact one copy: twice on one creator
/// (2607.00684's `mtablo@postech.ac.kr`), or kept in a note's text and written as a contact too (0807.0895, 2208.10209); the
/// output as the emitter wrote it.
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
  let (log, xml) = run(note, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    content_checks(&log),
    (vec![], vec!["ann@a.org bob@b.org".to_string()]),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "email=ann@a.org"]),
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
