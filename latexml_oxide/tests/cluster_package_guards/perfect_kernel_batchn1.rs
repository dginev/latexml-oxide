//! Guards for N1 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net"; OXIDIZED_DESIGN_DIVERGENCES #980): the emitter's checks of the
//! reading, each from digested facts, with the switch on. An e-mail's label in any language by its place on its line
//! (`an_email_label_in_any_language_is_written_as_printed`); the separator check (`a_role_heading_in_a_name_is_counted`) and the marks check
//! (`a_mark_reaching_no_contact_is_counted`), each counted in its Warning. N1r2 (review revN1): a kept note cuts out its written address by
//! position, the note of the author whose line it is (`a_note_cuts_out_its_written_address`); a lost name fails the names, the block written whole
//! (`a_lost_name_writes_the_block_whole`).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

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

/// The e-mail contacts of `xml`, in order: each `name` and text.
fn emails(xml: &str) -> Vec<(String, String)> {
  let contact =
    Regex::new(r#"<contact\b[^>]*\bname="([^"]*)"[^>]*\brole="email"[^>]*>(.*?)</contact>"#)
      .expect("regex");
  contact
    .captures_iter(xml)
    .map(|c| (c[1].replace('\u{a0}', " "), c[2].to_string()))
    .collect()
}

/// An e-mail's label in any language is what its line prints before the address's run, one word or two and a colon, written as printed: no word
/// lost, no note doubling the address (review revZ4br2's `Correo electr\'onico:`, `Courriel :`); with either switch.
#[test]
fn an_email_label_in_any_language_is_written_as_printed() {
  let template = repro!("emit_an_email_label_in_any_language");
  let printed = "Correo electr\\'onico: ann@a.org";
  for (label, name) in [
    (printed, "Correo electrónico: "),
    ("Courriel : ann@a.org", "Courriel : "),
  ] {
    let tex = template.replace(printed, label);
    for emit in [1, 2] {
      let (log, xml) = run(&tex, emit);
      assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
      assert_eq!(
        emails(&xml),
        [(name.to_string(), "ann@a.org".to_string())],
        "{label}"
      );
      assert!(!xml.contains("role=\"note\""), "{xml}");
    }
  }
}

/// The separator check: a name holding a label before a colon and the words after it, a role's heading read into it, is counted, the reading kept
/// (2205.07715's revtex `\author{Editor: …}`, hep-ph0311131's `{\bf Authors:} F.~Arleo$^a$, …`); with either switch.
#[test]
fn a_role_heading_in_a_name_is_counted() {
  let block = repro!("emit_net_a_role_heading_in_a_name");
  let revtex = "\\documentclass{revtex4-1}\n\\begin{document}\n\\title{T}\n\\author{Editor: Ann Able}\n\\affiliation{Univ A}\n\\author{Bob \
                Baker}\n\\affiliation{Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
  for (tex, warnings, heading) in [
    (block, 2, "Authors: | Ann Able"),
    (revtex, 1, "Editor: | Ann Able"),
  ] {
    for emit in [1, 2] {
      let (log, xml) = run(tex, emit);
      assert_eq!(
        (error_count(&log), warning_count(&log)),
        (0, warnings),
        "{log}"
      );
      assert!(
        log.contains(&format!(
          "safety net: a separator its author block prints between names parts no creators ({heading}); they are counted"
        )),
        "{log}"
      );
      let first = heading.replace(" |", "");
      assert_eq!(creators(&xml), [
        vec![first, "affiliation=Univ A".to_string()],
        vec!["Bob Baker".to_string(), "affiliation=Univ B".to_string()],
      ]);
    }
  }
}

/// The marks check: a name's mark whose label leads a line in its run of marks (`$^1$ $^2$ Univ A`), which no item of the name's creator carries, is
/// counted, the reading kept (1703.06483: the mark stays printed on Bob Baker's name); with either switch.
#[test]
fn a_mark_reaching_no_contact_is_counted() {
  let tex = repro!("emit_net_a_mark_reaches_no_contact");
  for emit in [1, 2] {
    let (log, xml) = run(tex, emit);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains("reaches no contact carrying its label, or a labelled contact no name's mark reaches (2); they are counted"),
      "{log}"
    );
    assert_eq!(creators(&xml), [
      vec!["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"],
      vec!["Bob Baker 2", "affiliation=Univ A, Town", "email=bob@b.org"],
      vec!["Cat Cole", "affiliation=Univ C, City", "email=cat@c.org"],
    ]);
  }
}

/// Per creator of `xml`, the text of each of its notes (`role="note"`), in order.
pub(super) fn notes(xml: &str) -> Vec<Vec<String>> {
  let creator = Regex::new(r"(?s)<creator\b.*?</creator>").expect("regex");
  let note = Regex::new(r#"(?s)<note role="note"[^>]*>(.*?)</note>"#).expect("regex");
  let tag = Regex::new(r"<[^>]+>").expect("regex");
  creator
    .find_iter(xml)
    .map(|c| {
      note
        .captures_iter(c.as_str())
        .map(|n| tag.replace_all(&n[1], "").to_string())
        .collect()
    })
    .collect()
}

/// A guard row: the block, the creators, and each creator's notes.
type NoteRow = (
  &'static str,
  &'static [&'static [&'static str]],
  &'static [&'static [&'static str]],
);

/// The rows of [`a_note_cuts_out_its_written_address`]: the block, the creators the switch on writes, and each one's notes.
const NOTE_ROWS: &[NoteRow] = &[
  // (2110.03506: a name's word the address repeats, `K. Hobbs (kerianne.hobbs@…)`)
  (
    "Ann L. Able, Bob Baker, and Cat Cole \\\\ POC: A.\\ Able (ann.able@a.org)\\\\ Approved for public release.",
    &[
      &[
        "Ann L. Able",
        "email=ann.able@a.org",
        "affiliation=Approved for public release.",
      ],
      &["Bob Baker", "affiliation=Approved for public release."],
      &["Cat Cole", "affiliation=Approved for public release."],
    ],
    &[&["POC: A. Able"], &[], &[]],
  ),
  // (2203.10777: a line the reading loses under the affiliation it repeats: the ink printing her address a note of hers, cut out; its
  // other inks the block's, a line's address owning none)
  (
    "Ann Able\\\\ Institute of Things, Universit\\\"at Dresden,\\\\ Dresden, Saxony 01062, Germany (ann.able@tu-dresden.de)\\\\ and \\\\ Bob \
     Baker\\\\ Institute of Things, Universit\\\"at Dresden,\\\\ Dresden, Saxony 01062, Germany (bob.baker@tu-dresden.de)",
    &[
      &[
        "Ann Able",
        "affiliation=Institute of Things, Universität Dresden",
        "email=ann.able@tu-dresden.de",
      ],
      &[
        "Bob Baker",
        "affiliation=Institute of Things, Universität Dresden",
        "email=bob.baker@tu-dresden.de",
      ],
      &[],
    ],
    &[&["Germany"], &["Germany"], &[
      "Dresden",
      "Saxony 01062",
      "Dresden",
      "Saxony 01062",
    ]],
  ),
  // (a label after the address, which no address run takes: the address's author's)
  (
    "Ann Able and Bob Baker \\\\ Univ A \\\\ ann.able@a.org (corresponding)",
    &[
      &["Ann Able", "affiliation=Univ A", "email=ann.able@a.org"],
      &["Bob Baker", "affiliation=Univ A"],
    ],
    &[&["corresponding"], &[]],
  ),
];

/// A kept note of ink the reading holds no place for is its lost words' own text, its written address runs cut out by position, not a range of words
/// out of a multiset (`POC: K. Hobbs (kerianne.hobbs`, 2110.03506; `Germany (martin.waltz@tu-dresden`, 2203.10777: review revN1); the note of the
/// author whose line it is by the reading's rule of an address; in one counted Warning, with either switch, off clean.
#[test]
fn a_note_cuts_out_its_written_address() {
  let template = repro!("emit_net_a_note_cuts_its_written_address");
  for &(block, expected, held) in NOTE_ROWS {
    let tex = template.replace(NOTE_ROWS[0].0, block);
    assert!(tex.contains(block));
    let (off_log, _) = run(&tex, 0);
    assert_eq!(
      (error_count(&off_log), warning_count(&off_log)),
      (0, 0),
      "{off_log}"
    );
    for emit in [1, 2] {
      let (log, xml) = run(&tex, emit);
      assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
      assert!(
        log.contains("they are kept as a note of the author block"),
        "{log}"
      );
      assert_eq!(creators(&xml), expected, "{block}");
      assert_eq!(notes(&xml), held, "{block}");
    }
  }
}

/// A lost lettered Name of a block that is no author's line by its addresses fails the names whatever its line holds: the block is written whole, Perl's
/// whole-block creator, not a name kept as a note (JAIR's `\name Shaul Markovitch \email …\\`, 1109.6618; 1312.0286, 1111.6082, 1905.12960,
/// 1807.03929; a column of its own, 1805.08288, 2002.00084: review revN1); in one counted Warning, with either switch, off clean.
#[test]
fn a_lost_name_writes_the_block_whole() {
  let tex = repro!("emit_net_a_lost_name_writes_the_block_whole");
  let block = "Ann Able ann@a.org\\\\ Bob Baker bob@b.org\\\\ Univ A\\\\ Town, Country";
  let columns = "\\begin{minipage}{.4\\linewidth}\\centering Ann Able\\\\ ann.able@a.org\\end{minipage}\\begin{minipage}{.4\\linewidth}\\centering Bob \
                 Baker\\\\ bob.baker@b.org\\end{minipage}\\\\ Univ A";
  for (block, whole) in [
    (
      block,
      "Ann Able ann@a.org Bob Baker bob@b.org Univ A Town, Country",
    ),
    (
      columns,
      "Ann Able ann.able@a.org Bob Baker bob.baker@b.org Univ A",
    ),
  ] {
    let tex = tex.replace(
      "Ann Able ann@a.org\\\\ Bob Baker bob@b.org\\\\ Univ A\\\\ Town, Country",
      block,
    );
    assert!(tex.contains(block));
    let (off_log, _) = run(&tex, 0);
    assert_eq!(
      (error_count(&off_log), warning_count(&off_log)),
      (0, 0),
      "{off_log}"
    );
    for emit in [1, 2] {
      let (log, xml) = run(&tex, emit);
      assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
      assert!(
        log.contains("the block is written whole, as Perl's whole-block creator"),
        "{log}"
      );
      assert_eq!(creators(&xml), [vec![whole.to_string()]], "{block}");
      assert_eq!(notes(&xml), [Vec::<String>::new()], "{block}");
    }
  }
}
