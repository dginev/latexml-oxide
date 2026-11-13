//! Guards for batch P23b (Option C P2-3b, OD #995): the frontmatter queue's `author:N` positions count names, by the
//! reading's cut at the `\and`-family separators (`queue_reading.rs` `entry_names`), not queued entries: a whole-list
//! `\author` (sv_support's, aa's, OmniBus's) queued as one entry holds a position a name, read by the queue's consumers
//! and by the resolver (`Resolver::author_entries`, the block's names recorded in the ledger). Each repro compiles clean
//! with pdflatex (svjour3 and aa with the classes arXiv papers ship; OmniBus's `nosuchclass` none); each row asserts
//! every creator's name and contacts, in order, and the exact Error and Warning counts.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/author_one_block/",
      $name,
      ".tex"
    ))
  };
}

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(ToString::to_string).collect())
    .collect()
}

/// An e-mail line of one address a name under a whole-list `\author` goes one address an author, as an `\and`-parted
/// llncs list's does (2403.04183's line): red on int169a, where the entry was one position and the line the block's
/// (a nameless creator). Rows, each a distinct path: the three bindings' whole-list `\author` (sv_support, aa, OmniBus);
/// marks inside a note and a box beside three names (the per-name marks reading); one address under the affiliation one
/// name requests (`queued_requesters` → `AddressPlacement::Authors`); a document macro holding only the separator (its
/// own tokens, `ArgSeparator::span`); the kernel `\author`'s list under a raw class's numbered store (P2-3c; red on
/// 5096ce95c6, the block's). OmniBus's Warning: the missing class binding. Distinct from
/// `perfect_kernel_batchp21::an_author_list_is_one_block_the_reading_cuts` (the reading's creators): the positions the queue
/// reads before the flush.
#[test]
fn a_whole_list_entry_holds_a_position_a_name() {
  let pair = |ann: &str, bob: &str| {
    rows(&[&[ann, "affiliation=1 Univ A", "email=u1@x.org"], &[
      bob,
      "affiliation=2 Univ B",
      "email=u2@x.org",
    ]])
  };
  for (name, tex, counts, expected) in [
    (
      "svjour3",
      repro!("svjour3_marked_list_email_line_one_address_a_name"),
      (0, 0),
      pair("Ann Able 1", "Bob Baker 2"),
    ),
    (
      "aa",
      repro!("aa_marked_list_email_line_one_address_a_name"),
      (0, 0),
      pair("Ann Able", "Bob Baker"),
    ),
    (
      "omnibus",
      repro!("omnibus_marked_list_email_line_one_address_a_name"),
      (0, 1),
      pair("Ann Able", "Bob Baker"),
    ),
    (
      "note and box",
      repro!("svjour3_marked_list_marks_in_a_note_and_a_box"),
      (0, 0),
      rows(&[
        &["Ann Able 1", "affiliation=1 Univ A", "email=u1@x.org"],
        &["Bob Baker 2", "affiliation=2 Univ B", "email=u2@x.org"],
        &["Cy Coe 1", "affiliation=1 Univ A", "email=u3@x.org"],
      ]),
    ),
    (
      "under one name's affiliation",
      repro!("svjour3_marked_list_address_under_one_names_affiliation"),
      (0, 0),
      rows(&[
        &["Ann Able 1", "affiliation=1 Univ A"],
        &["Bob Baker 2", "affiliation=2 Univ B", "email=u2@x.org"],
        &["affiliation=3 Lab C"],
      ]),
    ),
    (
      "macro holding the separator",
      repro!("svjour3_marked_list_macro_holding_only_the_separator"),
      (0, 0),
      pair("Ann Able 1", "Bob Baker 2"),
    ),
    (
      "kernel list, raw class store",
      repro!("kernel_raw_class_email_line_one_address_a_name"),
      (0, 0),
      rows(&[&["Ann Able", "affiliation=Univ A", "email=u1@x.org"], &[
        "Bob Baker",
        "affiliation=Univ B",
        "email=u2@x.org",
      ]]),
    ),
  ] {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    // (the note, Ann Able's: its mark requests nothing)
    let first = &xml[xml.find("<creator ").unwrap_or(0)..];
    let first = &first[..first.find("</creator>").unwrap_or(first.len())];
    let note = name == "note and box";
    assert_eq!(
      xml.matches("<note ").count(),
      usize::from(note),
      "{name}: {xml}"
    );
    assert_eq!(first.contains(">Note <sup"), note, "{name}: {xml}");
  }
}

/// A `*` printed after a name's `\inst` mark is the name's text, no request of the label `*`, where another name of the
/// whole-list block requests `*` by its `\inst{1*}` (2407.13764's shape): the emitter's kept `\inst` marks are each
/// name's own (`RCreator::class_requests`), not the block's. Red on P23b (49b1490df2): Ann Able's name "Ann Able*" and a
/// shown "∗" beside it. Rows: aa's and OmniBus's whole-list `\author` (`\lx@add@author@split`); llncs's list is cut at
/// the call, one name a block. Distinct from `a_whole_list_entry_holds_a_position_a_name`: the writer's marks, not the
/// queue's positions.
#[test]
fn a_star_after_an_inst_mark_is_the_names_text() {
  let expected = rows(&[
    &["Ann Able*", "affiliation=Univ A", "affiliation=Univ B"],
    &["Bob Baker ∗", "affiliation=Univ A"],
    &["Cy Coe", "affiliation=Univ B"],
  ]);
  for (name, tex, counts) in [
    (
      "aa",
      repro!("aa_star_after_inst_mark_stays_the_names_text"),
      (0, 0),
    ),
    (
      "omnibus",
      repro!("omnibus_star_after_inst_mark_stays_the_names_text"),
      (0, 1),
    ),
  ] {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    assert!(
      xml.contains("<personname>Ann Able*</personname>"),
      "{name}: {xml}"
    );
  }
}

/// An e-mail line under an affiliation entry two names request goes back into the entry without its e-mail command only
/// under a class's numbered `\institute` store (decided at its call on the store's lines,
/// `classify::store_unprinted_emails_unwrapped`), and only where that command, digested with the meaning TeX runs, prints
/// nothing in place: cjaa.cls's `\email` sets its addresses in `\@thanks`
/// (astro-ph0508574, 0911.3853, 0706.3426), a class's `\footnote`, a document macro's `\@thanks`; the Kind step takes
/// them out and the Link step gives each the author it spells (OD #956). A print in place keeps the line whole, as on
/// P23b. Rows, each a distinct path: the rerouted class setter printing nowhere (red on P23b: no e-mail, 3 Warnings), in
/// a note (red on P23b), in place with prose (red on P23c 093cd11411: the prose dropped, 0 Warnings), the document's
/// renewal printing in place (red on P23c: the label dropped), a document macro printing nowhere (red on P23b); and a
/// binding's list (aa, IEEEtran), no numbered store, with a document macro printing in place (red on P23c: "Corresponding
/// authors'", the label and the font dropped, 0 Warnings — review revP23c B1); the cutter's two `email_line_calls`
/// branches reading the bare line as the wrapped one (one address one requester, a run in author order: byte-identical
/// P23b-P23e), and a word printed in place before a note (red on P23d: "Contact" dropped, 0 Warnings). `contact`: the
/// pattern each row's contacts (or notes) open with, and how many.
#[test]
fn a_raw_class_store_email_line_goes_to_the_authors_it_spells() {
  macro_rules! store {
    ($name:literal) => {
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $name,
        ".tex"
      ))
    };
  }
  let pair = |wei: &str, dan: &str| {
    rows(&[
      &[
        "Wei Gao",
        "affiliation=Univ A, Town",
        wei,
        "affiliation=Univ B",
      ],
      &[
        "Dan Wei",
        "affiliation=Univ A, Town",
        dan,
        "affiliation=Univ B",
      ],
    ])
  };
  let plain = pair("email=whgao@x.org", "email=dwei@x.org");
  let listed = |mark: &str, other: &str| {
    let affiliation = format!("affiliation={mark} Univ A, Town Corresponding authors’");
    rows(&[
      &[
        &format!("Ann Able{}", if mark == "*" { " *" } else { "" }),
        &affiliation,
        "email=ann@x.org",
      ],
      &[
        &format!("Bob Best{}", if mark == "*" { " *" } else { "" }),
        &affiliation,
        "email=bob@y.org",
      ],
      &[
        &format!("Carl Cole{}", if mark == "*" { " †" } else { "" }),
        &format!("affiliation={other} Univ B"),
      ],
    ])
  };
  let typewriter = "<contact name=\"e-mails: \" role=\"email\"><text font=\"typewriter\"";
  let email = "<contact name=\"Email:\u{a0}\" role=\"email\">";
  for (name, tex, counts, expected, (contact, contacts)) in [
    (
      "class setter, \\@thanks",
      store!("raw_class_numbered_institute_email_each_address_its_author"),
      (0, 0),
      plain.clone(),
      (email, 2),
    ),
    (
      "class setter, a note",
      store!("raw_class_numbered_institute_email_note_each_address_its_author"),
      (0, 0),
      plain.clone(),
      (email, 2),
    ),
    (
      "class setter, in place with prose",
      store!("raw_class_numbered_institute_in_place_email_prose_kept"),
      (0, 2),
      pair(
        "email=Corresponding authors, e-mail: whgao@x.org",
        "email=Corresponding authors, e-mail: dwei@x.org",
      ),
      (
        r#"role="email">Corresponding authors, e-mail: <text font="typewriter""#,
        2,
      ),
    ),
    (
      "document's renewal, in place",
      store!("raw_class_numbered_institute_renewed_email_in_place_kept"),
      (0, 2),
      pair("email=E-mail: whgao@x.org", "email=E-mail: dwei@x.org"),
      (r#"role="email">E-mail: <text font="typewriter""#, 2),
    ),
    (
      "document macro, \\@thanks",
      store!("raw_class_numbered_institute_document_email_macro_unprinted"),
      (0, 1),
      plain.clone(),
      (email, 2),
    ),
    (
      "aa list, document macro in place",
      repro!("aa_marked_list_document_email_macro_line_kept_whole"),
      (0, 0),
      listed("1", "2"),
      (typewriter, 2),
    ),
    (
      "IEEEtran list, document macro in place",
      repro!("ieeetran_marked_list_document_email_macro_line_kept_whole"),
      (0, 0),
      listed("*", "†"),
      (typewriter, 2),
    ),
    (
      "class setter, \\@thanks, one address one requester",
      store!("raw_class_numbered_institute_email_one_address_one_requester"),
      (0, 0),
      rows(&[
        &["Wei Gao", "affiliation=Univ A, Town", "email=whgao@x.org"],
        &["Dan Wei", "affiliation=Univ B"],
      ]),
      (email, 1),
    ),
    (
      "class setter, \\@thanks, a run in author order",
      store!("raw_class_numbered_institute_email_run_in_author_order"),
      (0, 0),
      rows(&[
        &["Wei Gao", "affiliation=Univ A, Town", "email=whgao@x.org"],
        &["Dan Wei", "affiliation=Univ B", "email=dwei@x.org"],
      ]),
      (email, 2),
    ),
    (
      "class setter, a word in place and a note",
      store!("raw_class_numbered_institute_email_prose_in_place_note_kept_whole"),
      (0, 1),
      pair("email=Contact", "email=Contact"),
      ("@x.org</pubnote>", 2),
    ),
  ] {
    let (log, xml) = latexml::util::test::convert_files_with(tex, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(creators(&xml), expected, "{name}: {xml}");
    assert_eq!(xml.matches(contact).count(), contacts, "{name}: {xml}");
  }
}
