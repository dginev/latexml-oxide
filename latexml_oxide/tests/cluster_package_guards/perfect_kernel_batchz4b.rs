//! Guards for Z4b (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D; OXIDIZED_DESIGN_DIVERGENCES #980): `LATEXML_FM_EMIT=2`, each call
//! queuing one whole block, reaches `=1`'s creators (`the_whole_block_queue_reaches_one`: a queue position names a call's entry, an address placed by
//! the names it held whole is placed at the flush), and the emitter's safety net, the reading's word check: a block whose reading loses other ink
//! keeps it, the ink a note of the author block (`a_block_losing_other_ink_keeps_its_reading`; Perl's whole-block `\lx@add@author` only where the
//! names fail), an e-mail's label written as printed (`an_email_label_is_written_as_printed`), a block no rehearsal reads written whole
//! (`an_unread_block_is_written_whole`). Z4br2's RED: `annotate_made_reaches_every_name_red`.

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run, same_markup},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `=2` writes `=1`'s creators, XML whole, ids masked, clean: IEEEtran's e-mail line under one `\IEEEauthorblockN` of three names, each address on the
/// name it spells at the flush (2010.07442; `gkt@`, initials a whole entry spells for no one, 2405.01963), and Z4's shapes of marks, notes and order.
#[test]
fn the_whole_block_queue_reaches_one() {
  for tex in [
    repro!("emit_whole_queue_an_address_by_its_names_at_the_flush"),
    repro!("emit_marks_a_block_answers_or_shows"),
    repro!("emit_marks_a_per_author_class_keeps_on_its_names"),
    repro!("emit_order_each_call_its_place_on_its_author"),
    repro!("emit_notes_a_footnote_and_a_thanks_their_kinds"),
  ] {
    let (one_log, one) = run(tex, 1);
    let (two_log, two) = run(tex, 2);
    assert_eq!(
      (error_count(&one_log), warning_count(&one_log)),
      (error_count(&two_log), warning_count(&two_log)),
      "{two_log}"
    );
    assert_eq!(error_count(&two_log), 0, "{two_log}");
    assert_eq!(creators_xml(&two), creators_xml(&one), "{tex}");
  }
  let (_, two) = run(
    repro!("emit_whole_queue_an_address_by_its_names_at_the_flush"),
    2,
  );
  assert_eq!(creators(&two), [
    vec![
      "Ann Able 1",
      "affiliation=Univ A, Town",
      "email=ann.able@a.edu"
    ],
    vec![
      "Bob Baker 2",
      "affiliation=Univ B, City",
      "email=bob.baker@b.edu"
    ],
    vec![
      "George K. Tan 2",
      "affiliation=Univ B, City",
      "email=gkt@b.edu"
    ],
  ]);
}

/// The rows of [`a_block_losing_other_ink_keeps_its_reading`]: the block, and the creators the switch on writes (2605.16481's address no reader
/// reads; 2607.28418's link anchor printing two addresses, its target the block's; a third name after the lost line), the last the nameless creator of
/// the block's own, and the ink its note prints.
const LOST_INK_ROWS: &[(&str, &[&[&str]], &str)] = &[
  (
    "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\\n$^{1}$Univ A \\\\\n\\texttt{ann.able@outlook}, \\texttt{bob@b.edu}",
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A", "email=bob@b.edu"],
      &[],
    ],
    "ann.able@outlook",
  ),
  (
    "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\ $^{1}$Univ A \\\\ \\small{\\href{mailto:email@domain}{ann@a.edu; bob@b.edu}}",
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A"],
      &["email=email@domain"],
    ],
    "ann@a.edu; bob@b.edu",
  ),
  (
    "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\ $^{1}$Univ A \\\\ \\texttt{ann.able@outlook}, \\texttt{bob@b.edu} \\\\ Cat Cole$^{2}$ \\\\ $^{2}$Univ B",
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A", "email=bob@b.edu"],
      &["Cat Cole", "affiliation=Univ B"],
      &[],
    ],
    "ann.able@outlook",
  ),
];

/// A block whose reading holds every name it prints and no place for other ink keeps its reading, the ink its tokens as a note of the author block, in
/// one counted Warning (2605.16481: 3 creators, not one whole; 2607.28418); with either switch, and off the writer's two.
#[test]
fn a_block_losing_other_ink_keeps_its_reading() {
  let template = repro!("emit_a_block_losing_other_ink_keeps_its_reading");
  for &(block, expected, ink) in LOST_INK_ROWS {
    let tex = template.replace(LOST_INK_ROWS[0].0, block);
    assert!(tex.contains(block));
    let (off_log, off) = run(&tex, 0);
    assert_eq!(
      (error_count(&off_log), warning_count(&off_log)),
      (0, 0),
      "{off_log}"
    );
    assert!(creators(&off).len() >= 2, "{off}");
    for emit in [1, 2] {
      let (log, xml) = run(&tex, emit);
      assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
      assert!(
        log.contains("they are kept as a note of the author block"),
        "{log}"
      );
      assert_eq!(creators(&xml), expected, "{block}");
      let block_own = creators_xml(&xml).pop().unwrap_or_default();
      assert!(
        block_own.contains("role=\"note\"") && block_own.contains(ink),
        "{block_own}"
      );
    }
  }
}

/// The rows of [`an_email_label_is_written_as_printed`]: the block, and its e-mail contacts' `name`s.
const EMAIL_LABEL_ROWS: &[(&str, &[&str])] = &[
  (
    "Ann Able$^{1}$, Bob Baker$^{2}$\\\\ $^{1}$Univ A; $^{2}$Univ B\\\\ E-mail: \\email{ann@a.org}",
    &["E-mail: "],
  ),
  (
    "Ann Able$^{1}$ and Bob Baker$^{1}$\\\\ $^{1}$Univ A\\\\ E-mails: \\texttt{ann@a.org}, \\texttt{bob@a.org}",
    &["E-mails: ", "E-mails: "],
  ),
  (
    "Ann Able\\\\ Univ A\\\\ E-mail address: ann@a.org \\and Bob Baker\\\\ Univ B\\\\ email bob@b.org",
    &["E-mail address: ", "email "],
  ),
  ("Ann Able\\\\ Univ A\\\\ (e-mail: ann@a.org)", &["e-mail: "]),
];

/// The e-mail contacts' `name`s of `xml`, in order.
fn email_names(xml: &str) -> Vec<String> {
  let contact = Regex::new(r#"<contact\b[^>]*\bname="([^"]*)"[^>]*\brole="email""#).expect("regex");
  contact
    .captures_iter(xml)
    .map(|c| c[1].replace('\u{a0}', " "))
    .collect()
}

/// An address the block prints after its label word is named by it as printed, off and with either switch alike, no word lost and no fallback: PoS's raw
/// `E-mail: \email{…}` (1412.5940, 1612.08209, 1812.02252, 1907.01311, 1005.2532, 0802.3384), a list's label on each of its addresses (2511.22090), `E-mail
/// address:` and `email` without a colon (1209.0869), in parentheses (1405.3793); and a label in the address's own ink (`E-mail: \texttt{…}`, `Email: …`:
/// 1009.1965, 1807.01821, 1509.01480, 2205.04084), the creators as off's.
#[test]
fn an_email_label_is_written_as_printed() {
  let template = repro!("emit_an_email_label_as_printed");
  for &(block, names) in EMAIL_LABEL_ROWS {
    let tex = template.replace(EMAIL_LABEL_ROWS[0].0, block);
    assert!(tex.contains(block));
    for emit in [0, 1, 2] {
      let (log, xml) = run(&tex, emit);
      assert_eq!(
        (error_count(&log), warning_count(&log)),
        (0, 0),
        "{block}: {log}"
      );
      assert_eq!(email_names(&xml), names, "{block}, {emit}");
    }
  }
  let emitted = same_markup(repro!("emit_an_email_label_word_is_no_loss"), &[]);
  assert_eq!(emitted.len(), 2, "{emitted:#?}");
  let (log, _) = run(repro!("emit_an_email_label_word_is_no_loss"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
}

/// A block past the positions, which no rehearsal reads (2204.09614, 2404.06718), is with `=2` its queued whole entry, Perl's `\lx@add@author` of
/// it (the one whole-block writer, `author_block::whole_block_author`), in one counted Warning and no `merged_creators` Error; with the switch off the
/// writer cuts it.
#[test]
fn an_unread_block_is_written_whole() {
  let tex = repro!("emit_an_unread_block_is_written_whole");
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("can't read an author block (unstamped: out of positions): it is written whole"),
    "{log}"
  );
  let whole = format!("Ann Able and Bob Baker {}", vec!["Lab"; 5632].join(" "));
  assert_eq!(creators(&xml), [vec![whole]]);
  let (off_log, off) = run(tex, 0);
  assert_eq!(error_count(&off_log), 0, "{off_log}");
  assert_eq!(creators(&off).len(), 2, "{off}");
}

/// RED (review revZ4b finding 4): a tail a raw macro appends to `\@author` is handed to the creators the call made (`annotate={made}`,
/// frontmatter_stores.rs:7497); under `=2` the call made one whole entry, so only the last name holds the row. Off and `=1` give every name the row.
#[test]
#[ignore = "RED: `lx_author_made` counts the call's queued entries, one under =2 (frontmatter_stores.rs `annotate={made}`)"]
fn annotate_made_reaches_every_name_red() {
  let template = repro!("emit_whole_queue_annotate_made_red");
  for names in [
    "Ann Able \\and Bob Baker",
    "Ann Able, Bob Baker and Cat Cole",
  ] {
    let tex = template.replace("Ann Able \\and Bob Baker", names);
    let (_, off) = run(&tex, 0);
    let (log, two) = run(&tex, 2);
    assert_eq!(error_count(&log), 0, "{log}");
    assert!(creators(&off).iter().all(|c| c.len() == 2), "{off}");
    assert_eq!(creators(&two), creators(&off), "{names}");
  }
}
