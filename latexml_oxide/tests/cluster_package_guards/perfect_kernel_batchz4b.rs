//! Guards for Z4b (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, Route D; OXIDIZED_DESIGN_DIVERGENCES #980): `LATEXML_FM_EMIT=2`, each call
//! queuing one whole block, reaches `=1`'s creators (`the_whole_block_queue_reaches_one`: a queue position names a call's entry, an address placed by
//! the names it held whole is placed at the flush), and the emitter's safety net, the reading's word check: a block whose reading loses other ink
//! keeps it, the ink a note of the author block (`a_block_losing_other_ink_keeps_its_reading`; Perl's whole-block `\lx@add@author` only where the
//! names fail), an e-mail's label written as printed (`an_email_label_is_written_as_printed`), a block no rehearsal reads written whole
//! (`an_unread_block_is_written_whole`), and a tail a raw macro appends to `\@author` handed to every name the call made
//! (`annotate_made_reaches_every_name`, Z4br2's RED, green since LO2).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchn1::notes,
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
    let (two_log, _) = run(tex);
    assert_eq!(
      (error_count(&two_log), warning_count(&two_log)),
      (0, 0),
      "{two_log}"
    );
  }
  let (_, two) = run(repro!(
    "emit_whole_queue_an_address_by_its_names_at_the_flush"
  ));
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
/// reads; a third name after the lost line), the creator whose XML holds the ink's note (the one author its address spells, by the reading's own rule:
/// 2605.16481's Aiden Yiliu Li, review revZ4br2), and the ink its note prints.
const LOST_INK_ROWS: &[(&str, &[&[&str]], usize, &str)] = &[
  (
    "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\\n$^{1}$Univ A \\\\\n\\texttt{ann.able@outlook}, \\texttt{bob@b.edu}",
    &[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ A",
      "email=bob@b.edu",
    ]],
    0,
    "ann.able@outlook",
  ),
  (
    "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\ $^{1}$Univ A \\\\ \\texttt{ann.able@outlook}, \\texttt{bob@b.edu} \\\\ Cat Cole$^{2}$ \\\\ $^{2}$Univ B",
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A", "email=bob@b.edu"],
      &["Cat Cole", "affiliation=Univ B"],
    ],
    0,
    "ann.able@outlook",
  ),
];

/// A block whose reading holds every name it prints and no place for other ink keeps its reading, the ink its tokens as a note, in one counted Warning
/// (2605.16481: 3 creators, not one whole; 2607.28418): the note of the one author its address spells, else of the author block (the link's row below);
/// with either switch, and off the writer's two.
#[test]
fn a_block_losing_other_ink_keeps_its_reading() {
  let template = repro!("emit_a_block_losing_other_ink_keeps_its_reading");
  for &(block, expected, holder, ink) in LOST_INK_ROWS {
    let tex = template.replace(LOST_INK_ROWS[0].0, block);
    assert!(tex.contains(block));
    let (log, xml) = run(&tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains("they are kept as a note of the author block"),
      "{log}"
    );
    assert_eq!(creators(&xml), expected, "{block}");
    // (the note's whole text, on its holder alone)
    let held: Vec<Vec<String>> = (0..expected.len())
      .map(|i| {
        if i == holder {
          vec![ink.to_string()]
        } else {
          Vec::new()
        }
      })
      .collect();
    assert_eq!(notes(&xml), held, "{block}");
  }
  // (a link's anchor printing two addresses, 2607.28418, its addresses spelling two authors: the block's note; its unprinted target, which the
  // reading writes as an address, counted as written more often than printed, the link-target RED below — and at `=2` again by the content
  // check on the placed output, its words placed where no ink prints them: N2)
  let link = "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\ $^{1}$Univ A \\\\ \\small{\\href{mailto:email@domain}{ann@a.edu; bob@b.edu}}";
  let tex = template.replace(LOST_INK_ROWS[0].0, link);
  let (log, xml) = run(&tex);
  let placed = 1;
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 2 + placed),
    "{log}"
  );
  assert!(
    log.contains("they are kept as a note of the author block")
      && log.contains("more often than it prints them (email); they are counted"),
    "{log}"
  );
  assert_eq!(
    log
      .matches(
        "Frontmatter content check: 2 words or addresses appear in the placed frontmatter more often than the author \
         blocks print them: domain email."
      )
      .count(),
    placed,
    "{log}"
  );
  let own = creators_xml(&xml).pop().unwrap_or_default();
  assert!(!own.contains("<personname>"), "{own}");
  assert_eq!(
    notes(&xml).pop(),
    Some(vec!["ann@a.edu; bob@b.edu".to_string()]),
    "{xml}"
  );
}

/// RED (review revZ4br2 finding 2): a `mailto:` link whose anchor prints other addresses, 2607.28418's `\href{mailto:email@domain}{Hhz029@hotmail.com;
/// xyshen@eitech.edu.cn}`, is no e-mail `email@domain`: the reading writes its unprinted target as the contact's address (`link_target`), a reading
/// change. The word check counts it (a target prints only as its ink's own text).
#[test]
#[ignore = "RED: the reading writes a mailto link's unprinted target as its address (resolver `link_target`)"]
fn an_unprinted_link_target_is_no_address_red() {
  let template = repro!("emit_a_block_losing_other_ink_keeps_its_reading");
  let link = "Ann Able$^{1}$ \\quad Bob Baker$^{1}$ \\\\ $^{1}$Univ A \\\\ \\small{\\href{mailto:email@domain}{ann@a.edu; bob@b.edu}}";
  let (_, xml) = run(&template.replace(LOST_INK_ROWS[0].0, link));
  assert!(!xml.contains("email@domain</contact>"), "{xml}");
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
    let (log, xml) = run(&tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{block}: {log}"
    );
    assert_eq!(email_names(&xml), names, "{block}");
  }
  let emitted = same_markup(repro!("emit_an_email_label_word_is_no_loss"), &[]);
  assert_eq!(emitted.len(), 2, "{emitted:#?}");
  let (log, _) = run(repro!("emit_an_email_label_word_is_no_loss"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
}

/// A block past the positions, which no rehearsal reads (one call past the author field's 32,767; 2204.09614, 2404.06718 are read since positions
/// are per call, `perfect_kernel_batchf1`), is with `=2` its queued whole entry, Perl's `\lx@add@author` of
/// it (the one whole-block writer, `author_block::whole_block_author`), in one counted Warning and no `merged_creators` Error; with the switch off the
/// writer cuts it.
#[test]
fn an_unread_block_is_written_whole() {
  let tex = repro!("emit_an_unread_block_is_written_whole");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("can't read an author block (unstamped: out of positions): it is written whole"),
    "{log}"
  );
  let whole = format!("Ann Able and Bob Baker {}", vec!["Lab"; 11264].join(" "));
  assert_eq!(creators(&xml), [vec![whole]]);
}

/// A tail a raw macro appends to `\@author` is handed to the creators the call made (`annotate={made}`, frontmatter_stores.rs
/// `author_tail_calls`); under `=2` the call queued one whole entry, so the count reached the last name alone (review revZ4b finding
/// 4, RED until LO2). Where the call made every queued creator, the tail is every creator's, Perl's `annotate=all`
/// (`relocateAnnotations`' every parent, Base_Utility.pool.ltxml:520), as off and `=1` give it them.
#[test]
fn annotate_made_reaches_every_name() {
  let template = repro!("emit_whole_queue_annotate_made");
  for names in [
    "Ann Able \\and Bob Baker",
    "Ann Able, Bob Baker and Cat Cole",
  ] {
    let tex = template.replace("Ann Able \\and Bob Baker", names);
    let (log, _) = run(&tex);
    assert_eq!(error_count(&log), 0, "{log}");
  }
}
