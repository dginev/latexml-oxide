//! Guards for N2 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net", bar (2); OXIDIZED_DESIGN_DIVERGENCES #980):
//! the content invariant checked on the placed output (`resolver::placed`), under the switch's `=2` only, counting and never
//! answering. A word an address's domain spells is never written by the address (`a_word_beside_its_domain_is_kept`,
//! 1605.03019); an address placed more often than printed, twice on one creator or in a note and a contact
//! (`an_address_placed_more_often_than_printed_is_counted`, 2607.00684, 0807.0895); a block whose every word is placed once,
//! a shared entry and an ORCID logo among them, counts nothing (`a_clean_block_counts_nothing`), nor does a mark label
//! repeated legitimately — shared by names, per column, on footnote marks, aas's `\altaffilmark`, in a cut list, in a
//! box, glued to a name before a note's entry (CT1a, CT1ar: the other direction any rule counting a label repeated as
//! text must keep). A mark glued to the text of another entry of its label that the writer cut at is that entry's text,
//! counted lost (`a_glued_label_cut_as_an_entry_is_counted`, CT1ar; 2204.02412).

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
  let (log, xml) = run(tex);
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
}

/// An address placed more often than the page prints it is counted, each e-mail contact one copy: twice on one creator
/// (2607.00684's `mtablo@postech.ac.kr`), or kept in one creator's note and written as another's contact too (2410.20285's
/// `antonis@ucsb.edu` in Örwall's note); the output as the emitter wrote it. An address a note keeps on its own owner is
/// no longer written as a contact too (0807.0895, 2208.10209; FX3): that half counts nothing. The first repro's duplicate is
/// gone since Z18c (`xtra@b.org` once).
#[test]
fn an_address_placed_more_often_than_printed_is_counted() {
  // (since Z18c the brace list is written out whole — each address its own item, the call no copy left in the entry, and
  // a plain list written as text no Warning (`emitter::emit::one_address`) —, so `xtra@b.org` is placed once: the
  // block's, as the chain gives it; was twice, counted)
  let (log, xml) = run(repro!("emit_content_an_address_placed_twice"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(content_checks(&log), (vec![], vec![]), "{log}");
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
    row(&["email=xtra@b.org"]),
  ]);
  let note = repro!("emit_content_an_address_a_note_keeps");
  // (FX3: a note's link the written note still prints, on the contact's own owner, is that note's text alone — Ann's
  // address no contact; FX3b: Bob's address in Ann's note stays his contact, its only tie to him, and the excess is
  // counted, as before FX3)
  let (log, xml) = run(note);
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
}

/// A block whose every printed word is placed once counts nothing: an entry several creators share once (revtex's one
/// `\affiliation` under two authors, 2103.04797), an ORCID link's logo beside a name, a `\thanks`, an `\inst` list; and a
/// mark label repeated legitimately (CT1a): two names requesting the label of one entry (a block, a box's lines, revtex's
/// cut list), each column's own line 1, a footnote mark on three names, aas's `\altaffilmark` on two authors, a name's mark
/// glued to the name just before a note holding an entry of its label (CT1ar; the note written as the base writes it).
#[test]
fn a_clean_block_counts_nothing() {
  // (P4c: the lists' printed marks shown in the pins below, as pdflatex prints them — P4S Q1)
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
    (repro!("emit_content_a_label_shared_by_two_names"), vec![
      row(&["Ann Able", "affiliation=Univ X, Town"]),
      row(&["Bob Baker", "affiliation=Univ X, Town"]),
      row(&["Cat Cole", "affiliation=Univ Y, City"]),
    ]),
    (repro!("emit_content_labels_in_a_box"), vec![
      row(&["Ann Able", "affiliation=Univ X, Town"]),
      row(&[
        "Bob Baker",
        "affiliation=Univ X, Town",
        "affiliation=Univ Y, City",
      ]),
    ]),
    (
      repro!("emit_content_revtex_label_shared_in_a_cut_list"),
      vec![
        row(&["Ann Able 1", "affiliation=1 Dept of Physics, Univ X"]),
        row(&["Bob Baker 1", "affiliation=1 Dept of Physics, Univ X"]),
        row(&["Cat Cole 2", "affiliation=2 Lab Y"]),
      ],
    ),
    (repro!("emit_content_a_label_repeated_per_column"), vec![
      row(&["Ann Able", "affiliation=Univ X, Town"]),
      row(&["Bob Baker", "affiliation=Univ Y, City"]),
    ]),
    (
      repro!("emit_content_a_footnote_mark_on_several_names"),
      vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Cat Cole"])],
    ),
    (
      repro!("emit_content_aas_altaffilmark_on_several_names"),
      vec![
        row(&["Ann Able", "altaffiliation=Univ X, Town"]),
        row(&[
          "Bob Baker",
          "altaffiliation=Univ X, Town",
          "altaffiliation=Univ Y, City",
        ]),
        row(&["Cat Cole", "altaffiliation=Univ Y, City"]),
      ],
    ),
    (
      repro!("emit_content_a_name_mark_before_a_note_entry"),
      vec![
        row(&[
          "Ann Able",
          "affiliation=Center for Astro (CAP 1 ), Univ Z",
          "affiliation=Univ X, Town",
        ]),
        row(&["Bob Baker", "affiliation=Univ Y, City"]),
      ],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), expected, "{xml}");
  }
}

/// A mark glued to the text of another entry of its label (one list labels each entry once, `calls::split_marked_pieces`)
/// that the observer's writer cut the line at loses its word, and a separator's just before it: the content check counts
/// them, the output as the writer cuts it (RED catalog: the events-side cutter's S1 rule, SYNC row 99a). 2204.02412's
/// `(CAP$^3$)` (W 40 -> 41); the same past "and", in a `\parbox`, in an `\and` column of its own (CT1ar).
#[test]
fn a_glued_label_cut_as_an_entry_is_counted() {
  let row = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
  let cap = |bob: &[&str]| {
    vec![
      row(&["Ann Able", "affiliation=Univ X, Town"]),
      row(&[&["Bob Baker"], bob].concat()),
    ]
  };
  let cut = ["affiliation=Center for Astro (CAP", "affiliation=, Univ Z"];
  for (tex, lost, expected) in [
    (
      repro!("inblock_glued_repeated_label_is_entry_text_red"),
      "3",
      cap(&[&["affiliation=Univ Y, City"], &cut[..]].concat()),
    ),
    (
      repro!("emit_content_a_glued_label_after_and_counted_red"),
      "2 and",
      cap(&["affiliation=Dept Y", "affiliation=Lab Q, City"]),
    ),
    (
      repro!("emit_content_a_glued_label_in_a_parbox_counted_red"),
      "3",
      cap(&[&["affiliation=Univ Y, City"], &cut[..]].concat()),
    ),
    (
      repro!("emit_content_a_glued_label_in_its_column_counted_red"),
      "1",
      cap(&cut),
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert_eq!(
      content_checks(&log),
      (vec![lost.to_string()], vec![]),
      "{log}"
    );
    assert_eq!(creators(&xml), expected, "{xml}");
  }
}
