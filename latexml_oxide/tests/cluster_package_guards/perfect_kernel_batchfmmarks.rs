//! FMRG: the red/green suite of frontmatter mark association (DG 2026-11-16: "since we have so many disagreeing examples
//! from arXiv, we likely should use red/green tests to avoid regressions"). Every arXiv witness and reviewer probe of the
//! shared-note campaign (briefs SHARED, SHMARK, SHGIVEN, SHREST, PURE) is a repro whose header records its pdflatex
//! ORACLE; each row asserts that oracle, never what we output: per creator its personname (the name alone, DG's
//! personname purity), every note it carries as `role[mark]=text` (the mark set, sorted), then its contacts in order.
//!
//! The rules the oracles are read by (FIDELITY_CHARTER §4, the mark rule; DG's rulings 1-4 of 2026-11-16):
//! - a note shared by several names is a full copy on each carrier, every copy with the printed `mark` (Perl's
//!   `relocateAnnotations`, Base_Utility.pool.ltxml:895-901); a one-carrier note may stay its name's contact, the mark
//!   consumed by it (ruling 2: shown under the name, unmarked);
//! - a mark on a name lives outside the personname: the note it answers, the contact it labels, or a mark-only
//!   `note role="footnotemark"` (answered by no note of the block; a title's note is the title's);
//! - a `\footnotemark`/`\samethanks`/`\authornotemark` on a name is a copy of the note it marks (Perl's
//!   `relocateFootnote_aux`, latex_constructs.pool.ltxml:518-527);
//! - a mark between two names is the nearer one's by the printed gaps, a tie the next name's; one followed by a
//!   separator, `\\` or the block's end is carried back; one alone on the line after `\\` is no name's (ruling 4).
//!
//! GREEN rows are asserted. RED rows are known gaps, each naming its row R line (or NEITHER: a contradiction with no
//! recorded gap, reported to the main session); their `_red` tests are ignored and list what is still red, and what
//! now matches its oracle and should be promoted. A mark's U+2217 (`$^*$`) is folded to `*` (the class's `\@fnsymbol`).
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batch84b::text;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A repro and its oracle: the Warnings it honestly counts, and each creator as [`creator_lines`] writes it.
struct Row {
  repro:    &'static str,
  tex:      &'static str,
  warnings: usize,
  creators: &'static [&'static str],
  /// The row R line a RED row waits on (empty for a GREEN row).
  moves:    &'static str,
}

macro_rules! row {
  ($name:literal, $warnings:expr, [$($creator:expr),* $(,)?] $(,)?) => {
    row!($name, $warnings, [$($creator),*], "")
  };
  ($name:literal, $warnings:expr, [$($creator:expr),* $(,)?], $moves:expr $(,)?) => {
    Row {
      repro: $name,
      tex: repro!($name),
      warnings: $warnings,
      creators: &[$($creator),*],
      moves: $moves,
    }
  };
}

/// The value of attribute `name` in the start tag `tag`.
fn attribute(tag: &str, name: &str) -> String {
  tag
    .split(&format!(" {name}=\""))
    .nth(1)
    .and_then(|value| value.split('"').next())
    .unwrap_or("")
    .to_string()
}

/// The text of a note or contact body, its `ltx:tags` left out.
fn body_text(body: &str) -> String {
  let mut body = body.to_string();
  while let (Some(from), Some(to)) = (body.find("<tags>"), body.find("</tags>")) {
    body.replace_range(from..to + "</tags>".len(), "");
  }
  text(&body)
}

/// Each creator of `xml`: `name ; note ; note | contact ; contact`, a note `role[mark]=text` (the notes sorted, a
/// mark's U+2217 folded to `*`), a contact `role=text` (in order); an element inside another is its parent's text.
fn creator_lines(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</creator>").expect("closed creator");
      let creator = &xml[start..end];
      let name = creator.find("<personname>").map_or(String::new(), |at| {
        let close = at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname");
        text(&creator[at + "<personname>".len()..close])
      });
      let (mut notes, mut contacts) = (Vec::new(), Vec::new());
      let mut pos = 0;
      while let Some(at) = ["<note ", "<contact "]
        .iter()
        .filter_map(|open| creator[pos..].find(open).map(|i| pos + i))
        .min()
      {
        let kind = if creator[at..].starts_with("<note ") {
          "note"
        } else {
          "contact"
        };
        let tag_end = at + creator[at..].find('>').expect("start tag");
        let tag = &creator[at..tag_end];
        let (body, next) = if tag.ends_with('/') {
          ("", tag_end + 1)
        } else {
          // the matching end tag, past any element of the same kind inside
          let (mut depth, mut scan) = (1, tag_end + 1);
          loop {
            let open = creator[scan..].find(&format!("<{kind} ")).map(|i| scan + i);
            let close = scan + creator[scan..].find(&format!("</{kind}>")).expect("closed");
            match open {
              Some(o) if o < close => {
                let o_end = o + creator[o..].find('>').expect("start tag");
                if !creator[o..o_end].ends_with('/') {
                  depth += 1;
                }
                scan = o_end + 1;
              },
              _ => {
                depth -= 1;
                if depth == 0 {
                  break (&creator[tag_end + 1..close], close + kind.len() + 3);
                }
                scan = close + kind.len() + 3;
              },
            }
          }
        };
        if kind == "note" {
          notes.push(format!(
            "{}[{}]={}",
            attribute(tag, "role"),
            attribute(tag, "mark").replace('\u{2217}', "*"),
            body_text(body)
          ));
        } else {
          contacts.push(format!("{}={}", attribute(tag, "role"), body_text(body)));
        }
        pos = next;
      }
      notes.sort();
      let mut line = std::iter::once(name)
        .chain(notes)
        .collect::<Vec<_>>()
        .join(" ; ");
      if !contacts.is_empty() {
        line = format!("{line} | {}", contacts.join(" ; "));
      }
      line
    })
    .collect()
}

/// The rows whose conversion contradicts their oracle (an Error, another Warning count, or other creators), each with
/// what it found.
fn contradicted(rows: &[Row]) -> Vec<(&'static str, String)> {
  rows
    .iter()
    .filter_map(|row| {
      let (log, xml, ()) = convert_with_setup_then(row.tex, Some("ar5iv.sty"), || {}, |_| {});
      let found = (error_count(&log), warning_count(&log), creator_lines(&xml));
      let oracle = (
        0,
        row.warnings,
        row
          .creators
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
      );
      (found != oracle).then(|| {
        let moves = if row.moves.is_empty() {
          String::new()
        } else {
          format!(" [{}]", row.moves)
        };
        let report = format!(
          "{}{moves}\n  oracle (E, W, creators): {oracle:?}\n  found:                   {found:?}",
          row.repro
        );
        (row.repro, report)
      })
    })
    .collect()
}

/// Every GREEN row matches its oracle.
#[track_caller]
fn assert_green(rows: &[Row]) {
  let wrong = contradicted(rows);
  assert!(
    wrong.is_empty(),
    "{} of {} GREEN rows contradict their pdflatex oracle:\n{}",
    wrong.len(),
    rows.len(),
    wrong
      .into_iter()
      .map(|(_, report)| report)
      .collect::<Vec<_>>()
      .join("\n")
  );
}

/// The RED rows: prints those now matching their oracle (to promote to GREEN), fails while any is still red.
#[track_caller]
fn assert_red_rows_now_green(rows: &[Row]) {
  let red = contradicted(rows);
  for row in rows {
    if !red.iter().any(|(repro, _)| *repro == row.repro) {
      println!("now GREEN, promote: {}", row.repro);
    }
  }
  assert!(
    red.is_empty(),
    "{} of {} RED rows still contradict their oracle:\n{}",
    red.len(),
    rows.len(),
    red
      .into_iter()
      .map(|(_, report)| report)
      .collect::<Vec<_>>()
      .join("\n")
  );
}

/// The GREEN rows of [`shared_notes_are_a_marked_copy_per_carrier`].
const SHARED_GREEN: &[Row] = &[
  row!(
    "authblk_symbol_legend_shared_note_copies_carry_the_mark",
    0,
    [
      r#"Ann Able ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ A, Town"#,
    ]
  ),
  row!("author_block_symbol_legend_note_mark_not_in_its_text", 0, [
    r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
    r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
    r#"Cat Cole | affiliation=Univ A, Town"#,
  ]),
  row!(
    "author_block_three_symbol_legend_notes_carry_their_marks",
    0,
    [
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ B"#,
      r#"Cat Cole ; thanks[+]=Core contributor | affiliation=Univ B"#,
      r#"Dan Dorn ; thanks[+]=Core contributor | affiliation=Univ A"#,
      r#"Eve Ezra ; thanks[†]=Corresponding author | affiliation=Univ A"#,
    ]
  ),
  row!("ieee_thanks_symbol_legend_note_mark_lifted", 0, [
    r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
    r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
    r#"Cat Cole ; thanks[†]=Corresponding author: Cat Cole | affiliation=Univ A, Town"#,
  ]),
  row!("emit_authblk_label_legend_is_its_carriers_note", 0, [
    r#"Ann Able ; thanks[*]=Corresponding author. | affiliation=Univ A ; email=ann@a.org"#,
    r#"Bob Baker ; thanks[†]=These authors contributed equally to this work. | affiliation=Univ A"#,
    r#"Cat Cole ; thanks[†]=These authors contributed equally to this work. | affiliation=Univ B"#,
  ]),
  row!("legend_kind_authblk_labelled_legends", 0, [
    r#"Ann Able ; thanks[†]=These authors contributed equally. | affiliation=Univ A, Town"#,
    r#"Bob Baker ; thanks[*]=Corresponding author ; thanks[†]=These authors contributed equally. | affiliation=Univ A, Town ; email=bob@a.edu"#,
    r#"Cat Cole | affiliation=Univ B, City"#,
  ]),
  row!("ieee_refmark_mixed_with_digit_marks", 0, [
    r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, City A"#,
    r#"Bob Bell ; thanks[*]=Equal contribution | affiliation=Univ A, City A ; affiliation=Univ B, City B"#,
    r#"Cat Cole ; thanks[†]=Corresponding author: Cat Cole cat@c.edu | affiliation=Univ C, City C"#,
    r#"Dan Dorn | affiliation=Univ B, City B"#,
  ]),
  row!("revtex_frontmatter_footnote_on_two_authors", 0, [
    r#"Ann Author ; thanks[*]=These two authors contributed equally to this work. | email=a@x.org ; affiliation=Univ A"#,
    r#"Bob Author | email=b@x.org ; affiliation=Univ B"#,
    r#"Cat Author ; thanks[*]=These two authors contributed equally to this work. | email=c@x.org ; affiliation=Univ C"#,
  ]),
  row!(
    "elsarticle_fntext_shared_by_two_names_is_their_marked_note",
    0,
    [
      r#"Ann Able ; thanks[*]=Corresponding authors. ; thanks[1]=These authors contributed equally. | address=Univ A"#,
      r#"Bob Baker ; thanks[*]=Corresponding authors. ; thanks[1]=These authors contributed equally. | address=Univ A"#,
      r#"Cat Cole | note=Visiting Univ B. ; address=Univ A"#,
    ]
  ),
  row!(
    "elsarticle_list_correspondence_item_is_its_names_note",
    0,
    [
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Corresponding author: Univ A, Town. | affiliation=Univ A"#,
    ]
  ),
  row!("list_note_led_by_the_names_mark_is_their_note", 0, [
    r#"Ann Able ; thanks[‡]=These authors contributed equally. | affiliation=Univ A"#,
    r#"Bob Baker ; thanks[‡]=These authors contributed equally. | affiliation=Univ A"#,
  ]),
  row!("list_note_plain_text_is_the_blocks_note", 0, [
    r#"Ann Able | affiliation=Univ A , C"#,
    r#"Bob Baker | affiliation=Univ B , C"#,
    r#" ; note[]=Both authors are with the Lab."#,
  ]),
  row!("fmmarks_legend_dollar_mark_and_typed_star", 0, [
    r#"Ann Able ; thanks[$]=these authors contributed equally to this work | affiliation=Univ A, Town ; affiliation=Univ B, City"#,
    r#"Bob Baker ; thanks[$]=these authors contributed equally to this work | affiliation=Univ A, Town"#,
    r#"Cat Cole | affiliation=Univ B, City"#,
    r#"Dan Dorn ; thanks[*]=corresponding author | affiliation=Univ A, Town"#,
  ]),
  row!(
    "fmmarks_elsarticle_shared_fnref_and_unreferenced_fntext",
    0,
    [
      r#"Ann Able ; thanks[1]=Equal. | address=Univ A"#,
      r#"Bob Baker ; thanks[1]=Equal. | address=Univ A"#,
      r#" | note=Unreferenced note."#,
    ]
  ),
];
/// The RED rows of [`shared_notes_are_a_marked_copy_per_carrier`].
const SHARED_RED: &[Row] = &[
  row!(
    "fmmarks_authblk_double_star_label_and_date_note_red",
    0,
    [
      r#"Ann Able ; footnotemark[§]= ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ A, Town ; affiliation=Present address: Lab B, City"#,
    ],
    r#"NEITHER: reported to main (the `§` label no `\affil` answers is dropped with no Warning)"#
  ),
  row!(
    "list_note_with_two_marks_is_the_blocks_note",
    0,
    [
      r#"Ann Able ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[*]= | affiliation=Univ A"#,
      r#" ; note[]=† Equal. ∗ Corresponding."#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "ieee_refmark_beside_a_starred_legend",
    2,
    [
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A"#,
      r#"Bob Bell | affiliation=Univ B"#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname); the refmark label leading an affiliation or legend text"#
  ),
  row!(
    "fmmarks_ieee_blockn_typed_star_thanks_red",
    0,
    [
      r#"Ann Able ; thanks[*]=Equal Contribution | affiliation=Univ A ; email=ann@a.edu"#,
      r#"Bob Baker ; thanks[*]=Equal Contribution | affiliation=Univ A ; email=bob@a.edu"#,
      r#"Cat Cole | affiliation=Univ B ; email=cat@b.edu"#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "fmmarks_thanks_holds_a_second_legend_red",
    0,
    [
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Corresponding Author. † Co-first authors | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Cat Cole | affiliation=Univ B"#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname) (PUREb B1: the note keeps its own ∗)"#
  ),
];
/// Shared notes (ruling 1): a legend or note several names print is a full copy on each carrier, every copy marked —
/// authblk symbol labels (2606.00446), the hand-typed legend lines of the kernel block (2407.07614, 1910.10432), IEEEtran
/// `\thanks{$^{*}$…}` and refmarks (2312.10807), REVTeX's joined footnote, elsarticle `\fnref`/`\fntext` (2102.07817),
/// a class's title-page list (2606.06621).
#[test]
fn shared_notes_are_a_marked_copy_per_carrier() { assert_green(SHARED_GREEN); }
/// The known gaps of [`shared_notes_are_a_marked_copy_per_carrier`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn shared_notes_are_a_marked_copy_per_carrier_red() { assert_red_rows_now_green(SHARED_RED); }

/// The GREEN rows of [`name_marks_live_outside_the_personname`].
const PURE_GREEN: &[Row] = &[row!("fmmarks_pure_hash_in_name", 0, [r#"The C# Team"#,])];
/// The RED rows of [`name_marks_live_outside_the_personname`].
const PURE_RED: &[Row] = &[
  row!(
    "author_unmatched_numeric_mark",
    0,
    [
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Bee ; footnotemark[1]="#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "fmmarks_pure_href_marks_red",
    0,
    [r#"Ann Able ; footnotemark[*]="#, r#"Bob Baker | url=†"#,],
    r#"PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "fmmarks_pure_isotope_collaboration_name_red",
    0,
    [r#"3 He Collaboration"#,],
    r#"row R (PURE residual: `$^{3}$He Collaboration`, the reader splits the name and drops the 3)"#
  ),
  row!(
    "fmmarks_pure_leading_digit_marks_red",
    1,
    [
      r#"Ann Able ; footnotemark[1]="#,
      r#"Bob Baker ; footnotemark[2]="#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "fmmarks_pure_letter_mark_and_note_opening_word_red",
    0,
    [r#"Ann Able ; footnotemark[a]= ; thanks[*]=a visitor at the Institute."#,],
    r#"PURE (fix/pure-personname: the marks out of the personname) (PUREb B2: a note's opening letter is its word)"#
  ),
  row!(
    "fmmarks_star_sup_beside_thanks_body_legend_red",
    0,
    [
      r#"Ann Able ; footnotemark[*]= ; thanks[*]=Email: ann@x.org"#,
      r#"Bob Baker ; footnotemark[*]= ; thanks[†]=Email: bob@x.org"#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname) (PUREb B1: a typed symbol beside a `\thanks` is no mark of it)"#
  ),
  row!(
    "ne2c_mark_after_math_comma",
    0,
    [
      r#"Ann Able ; endnotemark[1]="#,
      r#"Bob Baker ; endnotemark[1]= ; endnotemark[2]="#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "ne2e_mark_in_formula_past_thin_spaces",
    0,
    [
      r#"Ann Able ; footnotemark[*]= ; footnotemark[1]= ; thanks[*]=X"#,
      r#"Bob Baker"#,
    ],
    r#"PURE (fix/pure-personname: the marks out of the personname); row R (a note inside a formula: the in-formula `\footnotemark[1]` counted lost)"#
  ),
];
/// Name marks re-homed (DG 2026-11-16, personname purity): a mark typed on a name leaves the personname — the note it
/// answers, the contact it labels or a mark-only note — while name ink (`The C\# Team`, `$^{3}$He`) stays (revPURE;
/// 2406.11149, 2506.02863, 0707.3527, 0801.1438).
#[test]
fn name_marks_live_outside_the_personname() { assert_green(PURE_GREEN); }
/// The known gaps of [`name_marks_live_outside_the_personname`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn name_marks_live_outside_the_personname_red() { assert_red_rows_now_green(PURE_RED); }

/// The GREEN rows of [`a_mark_on_a_name_is_a_copy_of_its_note`].
const LINK_GREEN: &[Row] = &[
  row!("author_samethanks_is_a_copy_of_the_marked_note", 0, [
    r#"Ann Able ; thanks[*]=Equal Contribution. | affiliation=Univ A"#,
    r#"Bob Baker ; thanks[*]=Equal Contribution. | affiliation=Univ A"#,
    r#"Cat Cole | affiliation=Univ B"#,
  ]),
  row!("author_samethanks_mark_is_the_same_note", 0, [
    r#"Ann Able ; thanks[*]=Equal contribution."#,
    r#"Bob Baker ; thanks[*]=Equal contribution."#,
    r#"Cat Cole"#,
  ]),
  row!("acmart_authornotemark_is_a_copy_of_the_marked_note", 0, [
    r#"Ann Able ; thanks[*]=Equal contribution. | affiliation=Univ A , Town , X"#,
    r#"Bob Baker ; thanks[*]=Equal contribution. | affiliation=Univ A , Town , X"#,
    r#"Cat Cole | affiliation=Univ A , Town , X"#,
  ]),
  row!("author_thanks_shared_by_footnotemark_marks_kept", 0, [
    r#"Ann Able ; thanks[*]=Equal contribution. ; thanks[†]=Univ A"#,
    r#"Bob Baker ; thanks[*]=Equal contribution. ; thanks[‡]=Univ B"#,
    r#"Cat Cole ; thanks[†]=Univ A ; thanks[‡]=Univ B"#,
    r#"Dan Dorn ; thanks[‡]=Univ B"#,
  ]),
  row!("author_thanks_mark_after_a_title_thanks", 0, [
    r#"Ann Able ; thanks[†]=Note X."#,
    r#"Bob Baker ; footnotemark[*]="#,
  ]),
  row!("author_footnote_is_the_marked_thanks_note", 0, [
    r#"Ann Able ; thanks[*]=Univ X"#,
    r#"Bob Baker"#,
  ]),
  row!("article_title_block_marks_in_fnsymbol", 0, [
    r#"Ann ; footnote[*]=Equal."#,
    r#"Bob ; footnote[†]=Intern."#,
  ]),
  row!("ne2c_mark_after_note", 0, [
    r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
    r#"Bob Baker ; footnotemark[‡]="#,
  ]),
  row!("ne2c_mark_after_note_numbered", 0, [
    r#"Ann Able ; footnotemark[*]= ; thanks[*]=Univ X."#,
    r#"Bob Baker ; thanks[*]=Univ X."#,
    r#"Carl Cole ; thanks[†]=Univ Y."#,
  ]),
  row!("ne2c_mark_after_note_spaced", 0, [
    r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
    r#"Bob Baker ; thanks[‡]=Univ Y."#,
    r#"Carl Cole ; footnotemark[§]="#,
  ]),
  row!("ne2c_mark_after_footnote_glued", 0, [
    r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
    r#"Bob Baker ; thanks[*]=Univ X. ; thanks[‡]=Univ Y."#,
    r#"Carl Cole ; footnotemark[§]="#,
  ]),
  row!("ne2b_note_before_every_name", 0, [
    r#"Ann Able ; footnotemark[†]= ; thanks[*]=Lead note."#,
    r#"Bob Baker"#,
  ]),
  // the marks as printed; the Warning is the fail-safe split's (counted, Perl's split)
  row!("ne2b_counter_moved_between_notes", 1, [
    r#"Ann Able ; thanks[*]=Univ X. ; thanks[†]=Univ Z."#,
    r#"Bob Baker ; thanks[†]=Univ Z. ; thanks[‡]=Univ Y."#,
    r#"Carl Cole ; thanks[*]=Univ X."#,
  ]),
  row!("fmmarks_title_thanks_then_author_marks_link", 0, [
    r#"Ann Able ; thanks[†]=Equal contribution."#,
    r#"Bob Baker ; thanks[†]=Equal contribution."#,
    r#"Cy Cole ; footnotemark[*]="#,
  ]),
  row!("fmmarks_title_footnote_and_date_thanks_shift_marks", 0, [
    r#"Ann Able ; thanks[‡]=Equal contribution."#,
    r#"Bob Baker ; thanks[‡]=Equal contribution."#,
    r#"Cy Cole ; footnotemark[*]="#,
  ]),
  row!("fmmarks_counter_set_arabic_thanks", 0, [
    r#"Ann Able ; thanks[‡]=First note."#,
    r#"Bob Baker ; thanks[§]=Second note."#,
  ]),
  row!("fmmarks_spaced_marks_before_a_comma_fnsymbol_texts", 0, [
    r#"Ann Able ; footnote[*]=Univ A. Email: ann@a.edu ; footnote[‡]=Institute C."#,
    r#"Bob Baker ; footnote[†]=Univ B. Email: bob@b.edu"#,
  ]),
  row!("fmmarks_thanks_typed_star_text_and_footnotemark", 0, [
    r#"Ann Able ; thanks[*]=*Equal contribution | affiliation=Univ A"#,
    r#"Bob Baker ; thanks[*]=*Equal contribution | affiliation=Univ B"#,
  ]),
];
/// The RED rows of [`a_mark_on_a_name_is_a_copy_of_its_note`].
const LINK_RED: &[Row] = &[
  row!(
    "fmmarks_acmart_authornotemark_without_its_note_red",
    0,
    [
      r#"Ann Able ; thanks[*]=Ann Able and Bob Baker contributed equally. | email=ann@a.edu ; affiliation=Univ A , Town , Land"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Ann Able and Bob Baker contributed equally. | email=bob@a.edu ; affiliation=Univ A , Town , Land"#,
      r#"Cat Cole | email=cat@a.edu ; affiliation=Univ A , Town , Land"#,
    ],
    r#"NEITHER: reported to main (the mark-only note holds the glyph as its text)"#
  ),
  row!(
    "ne2d_given_mark_dropped_keeps_the_creators",
    0,
    [
      r#"Ann Able ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Carl Cole ; thanks[*]=Univ X. | affiliation=Univ B"#,
      r#"Dan Dory | affiliation=Univ B"#,
    ],
    r#"row R COMMAMARK (SHGIVEN open: a given mark after `Name,$^{1}$`, 1912.01737, 1811.08622, 2101.04763)"#
  ),
  row!(
    "ne2d_given_and_fresh_mark_dropped",
    0,
    [
      r#"Ann Able ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Carl Cole ; footnotemark[†]= | affiliation=Univ B"#,
      r#"Dan Dory ; footnotemark[‡]= | affiliation=Univ B"#,
    ],
    r#"row R COMMAMARK (SHGIVEN open: a given mark after `Name,$^{1}$`, 1912.01737, 1811.08622, 2101.04763)"#
  ),
];
/// Links: a `\footnotemark[n]`, `\samethanks[n]` or `\authornotemark[n]` on a name is a copy of the note it marks, its
/// mark as the class numbers it after the title's notes (2502.19103, 1911.13299, 1706.03762, 2607.15705, 2203.12881,
/// 1803.03669; revSHMARK p1, p2, p9).
#[test]
fn a_mark_on_a_name_is_a_copy_of_its_note() { assert_green(LINK_GREEN); }
/// The known gaps of [`a_mark_on_a_name_is_a_copy_of_its_note`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn a_mark_on_a_name_is_a_copy_of_its_note_red() { assert_red_rows_now_green(LINK_RED); }

/// The GREEN rows of [`a_mark_past_spacing_is_the_nearer_names`].
const SPACING_GREEN: &[Row] = &[
  row!("ne2e_mark_past_spacing_is_its_names", 0, [
    r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
    r#"Bob Baker ; thanks[*]=Univ X."#,
    r#"Carl Cole ; footnotemark[†]="#,
    r#"Dan Dory ; footnotemark[‡]="#,
  ]),
  row!("ne2h_marks_run_carried_back_whole", 0, [
    r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
    r#"Bob Baker"#,
    r#"Carl Cole"#,
  ]),
  // the mark as printed; the Warning over-counts it (row R: a prefix mark not first on its line, fails toward flagging)
  row!("ne2e_prefix_mark_past_spacing_before_and", 1, [
    r#"Ann Able"#,
    r#"Bob Baker ; footnotemark[†]="#,
    r#"Carl Cole"#,
  ]),
  // the marks as printed; the Warnings over-count them (the same row R line)
  row!("ne2e_prefix_marks_past_quad", 2, [
    r#"Ann Able ; footnotemark[*]="#,
    r#"Bob Baker ; footnotemark[†]="#,
    r#"Carl Cole ; footnotemark[*]="#,
  ]),
  row!("fmmarks_bare_marks_after_space_and_quad", 0, [
    r#"Ann Able ; footnotemark[*]="#,
    r#"Bob Baker ; thanks[†]=X"#,
    r#"Carl Cole ; footnotemark[‡]="#,
  ]),
  row!("fmmarks_mark_before_comma_after_quad", 0, [
    r#"Ann Able ; footnotemark[†]="#,
    r#"Bob Baker"#,
    r#"Carl Cole"#,
  ]),
  row!("fmmarks_mark_inside_the_names_font_group", 0, [
    r#"Ann Able ; footnotemark[*]="#,
    r#"Bob Baker"#,
  ]),
  row!("fmmarks_mark_then_line_break", 0, [
    r#"Ann Able ; footnotemark[†]= | affiliation=Univ A"#,
    r#"Bob Baker"#,
  ]),
  row!("fmmarks_marks_run_split_then_line_break", 0, [
    r#"Ann Able ; footnotemark[*]= ; footnotemark[†]= | affiliation=Univ A"#,
    r#"Bob Baker"#,
  ]),
];
/// The RED rows of [`a_mark_past_spacing_is_the_nearer_names`].
const SPACING_RED: &[Row] = &[
  row!(
    "ne2c_mark_on_no_name_writes_the_block_whole",
    1,
    [
      r#"Ann Able ; thanks[*]=Univ X."#,
      r#"Bob Baker ; footnotemark[‡]="#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block); ruling 4: a mark alone on the line after `\\` is no name's (counted)"#
  ),
  row!(
    "ne2e_prefix_mark_past_hfill",
    0,
    [
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[†]="#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2f_bare_mark_nearer_next_past_quads",
    0,
    [
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[‡]= ; footnotemark[†]="#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2f_mark_and_note_nearer_next",
    0,
    [
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[‡]=Y"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2f_mark_nearer_next_past_qquad",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2f_mark_nearer_next_past_quads",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2f_mark_nearer_next_past_ties",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2g_mark_in_group_with_previous_name",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2g_mark_in_its_own_group",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2f_marks_wider_after_and_tied",
    0,
    [
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole ; footnotemark[‡]="#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2g_mark_past_hfill_undecided",
    0,
    [r#"Ann Able ; footnotemark[†]="#, r#"Bob Baker"#,],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block) (the centred line leaves the fill 0pt: † 3.9pt after Ann, 11.8pt before Bob)"#
  ),
  row!(
    "ne2g_mark_then_orcid_nearer_next",
    0,
    [
      r#"Ann Able | orcid="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2g_mark_then_superscript_nearer_next",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[a]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block); PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "ne2h_marks_run_narrower_after_is_undecided",
    0,
    [
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "ne2h_marks_run_wider_before_is_undecided",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_bare_prefix_mark_after_quad_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Carl Cole ; footnotemark[†]="#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_gap_mark_then_thanks_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Ann note"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_gap_thanks_then_mark_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Ann note"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_after_wide_hspace_nearer_next_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_and_note_floating_between_names_red",
    0,
    [
      r#"Ann Able ; thanks[*]=Ann note body"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[‡]=Floating body"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_between_interword_spaces_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_carried_back_before_bold_name_red",
    0,
    [
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENFONT (proposed: a font command opening the next name stops the carry-back; SHGIVENe's s4)"#
  ),
  row!(
    "fmmarks_mark_nearer_next_bold_name_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_then_cite_nearer_next_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_then_footnotetext_nearer_next_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; thanks[†]=Bob note"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_then_hfill_before_next_red",
    0,
    [r#"Ann Able"#, r#"Bob Baker ; footnotemark[†]="#,],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_mark_then_superscript_label_of_next_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[b]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block); PURE (fix/pure-personname: the marks out of the personname)"#
  ),
  row!(
    "fmmarks_mark_then_thanks_before_next_name_red",
    0,
    [
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Bob note"#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_marks_run_split_by_wide_space_red",
    0,
    [
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_marks_three_names_one_line_red",
    0,
    [
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker"#,
      r#"Carl Cole ; footnotemark[†]="#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_tabular_cell_bare_prefix_mark_red",
    0,
    [
      r#"Ann Able ; thanks[*]=X | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[†]= | affiliation=Univ B"#,
    ],
    r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#
  ),
  row!(
    "fmmarks_tabular_cell_given_prefix_mark_red",
    0,
    [r#"Ann Able"#, r#"Bob Baker ; footnotemark[†]="#,],
    r#"NEITHER: reported to main (carried back across the cell boundary to Ann, no Warning)"#
  ),
  row!(
    "fmmarks_single_name_marks_after_newline_thin_spaces_red",
    0,
    [r#"Ann Able ; footnotemark[1]= ; footnotemark[2]="#,],
    r#"NEITHER: reported to main (the marks written * and †, not the printed 1 and 2)"#
  ),
];
/// Marks after spacing (ruling 4 and the nearer-name rule): a mark spacing parts from the names is the nearer name's by
/// the printed gaps (a tie the next's); followed by a separator, `\\` or the block's end it is carried back (1911.13299,
/// 2412.20504, 1807.02791; every revSHGIVEN probe).
#[test]
fn a_mark_past_spacing_is_the_nearer_names() { assert_green(SPACING_GREEN); }
/// The known gaps of [`a_mark_past_spacing_is_the_nearer_names`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn a_mark_past_spacing_is_the_nearer_names_red() { assert_red_rows_now_green(SPACING_RED); }

/// The GREEN rows of [`revtex_marks_follow_its_numbering`].
const REVTEX_GREEN: &[Row] = &[
  row!(
    "revtex40_name_footnote_unnumbered_is_its_note_contact",
    0,
    [
      r#"Ann Able | thanks=Equal contribution. ; affiliation=Univ A"#,
      r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
    ]
  ),
  row!("revtex41_name_footnote_is_its_note_contact", 0, [
    r#"Ann Able | note=ann@a.edu ; affiliation=Univ A, Town"#,
    r#"Bob Baker | note=Corresponding author: bob@b.edu ; affiliation=Univ A, Town"#,
  ]),
  row!("revtex42_name_footnote_is_its_note_contact", 0, [
    r#"Ann Able | thanks=Equal contribution. ; affiliation=Univ A"#,
    r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
    r#"Cy Cole | email=cy@b.edu ; affiliation=Univ B"#,
  ]),
  row!("revtex4_name_footnote_takes_its_number", 0, [
    r#"Ann Able | note=Visiting Univ C. ; affiliation=Univ A"#,
    r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
  ]),
  row!("revtex_name_footnote_given_its_number_steps_nothing", 0, [
    r#"Ann Able | note=Visiting Univ C. ; affiliation=Univ A"#,
    r#"Bob Baker ; thanks[*]=Equal contribution. | affiliation=Univ B"#,
    r#"Cy Cole ; thanks[*]=Equal contribution. | affiliation=Univ B"#,
  ]),
  row!("revtex_name_footnote_takes_its_number", 0, [
    r#"Ann Able | note=Visiting Univ C. ; affiliation=Univ A"#,
    r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
  ]),
  row!("revtex_repeated_thanks_is_the_authors_marked_note", 0, [
    r#"Ann Able | email=a@x.org ; thanks=These authors contributed equally. ; affiliation=Univ A"#,
    r#"Bob Baker ; thanks[†]=These authors contributed equally. | affiliation=Univ A"#,
    r#"Cat Cole ; thanks[†]=These authors contributed equally. | affiliation=Univ A"#,
    r#"Dan Dole | altaffiliation=Univ B. ; affiliation=Univ A"#,
  ]),
  row!("revtex_thanks_in_names_stay_unmarked", 0, [
    r#"Ann Able ; thanks[]=Equal. | affiliation=Univ A"#,
    r#"Bob Baker ; thanks[]=Equal. | affiliation=Univ A"#,
    r#"Cat Cole ; thanks[]=Other. | affiliation=Univ A"#,
  ]),
  row!("revtex_affiliation_footnotemark_link_is_no_note", 0, [
    r#"Ann Able ; thanks[*]=E-mail: ann@a.org | affiliation=Univ A, Town ; affiliation=Lab B, City"#,
    r#"Bob Baker ; thanks[†]=E-mail: bob@c.org | affiliation=Univ C, Village"#,
  ]),
  row!("emit_revtex_macro_name_with_thanks_written_once", 0, [
    r#"Ann Able ; thanks[]=Corresponding author. | affiliation=Univ A"#,
    r#"Bob Baker | affiliation=Univ B"#,
  ]),
  row!("fmmarks_revtex40_email_then_name_footnote", 0, [
    r#"Ann Able | email=ann@a.edu ; affiliation=Univ A"#,
    r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
  ]),
  row!("fmmarks_revtex40_two_name_footnotes", 0, [
    r#"Ann Able | affiliation=Univ A"#,
    r#"Bob Baker | note=Visiting Univ C. ; note=Second. ; affiliation=Univ B"#,
  ]),
  row!("fmmarks_revtex42_reprint_thanks_name_footnote_email", 0, [
    r#"Ann Able | thanks=Equal contribution. ; affiliation=Univ A"#,
    r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
    r#"Cy Cole | email=cy@b.edu ; affiliation=Univ B"#,
  ]),
  row!("fmmarks_revtex_shared_thanks_then_email", 0, [
    r#"Ann Able | thanks=Equal. ; affiliation=Univ A"#,
    r#"Bob Baker | thanks=Equal. ; email=b@x.org ; affiliation=Univ A"#,
  ]),
  row!(
    "fmmarks_revtex_footnotemark_answered_by_altaffiliation",
    0,
    [
      r#"Ann Able ; footnotemark[*]= | altaffiliation=These authors contribute equally to this work. ; affiliation=Univ A, Town"#,
      r#"Bob Baker ; footnotemark[*]= | altaffiliation=These authors contribute equally to this work. ; affiliation=Univ B, City"#,
      r#"Cat Cole | email=cat@a.edu ; affiliation=Univ A, Town"#,
    ]
  ),
];
/// The RED rows of [`revtex_marks_follow_its_numbering`].
const REVTEX_RED: &[Row] = &[
  row!(
    "revtex_affiliation_thanks_before_a_shared_note_red",
    0,
    [
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Visiting. | affiliation=Univ A"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Di Dorn ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    r#"row R (its own: REVTeX's affiliation footnote order; SHRESTc fail-safe)"#
  ),
  row!(
    "revtex_name_footnotemark_leaves_the_marks_unknown",
    0,
    [
      r#"Ann Able ; footnotemark[a]= | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    r#"OD #1054 fail-safe (a name `\footnotemark` leaves the counted marks unknown); Ann's invented "*" reported to main (the PDF prints "a")"#
  ),
  row!(
    "revtex_footnotes_numbered_as_printed",
    0,
    [
      r#"Ann Able ; thanks[†]=These authors contributed equally. ; thanks[‡]=Visiting Univ C. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally. ; thanks[‡]=Visiting Univ C. | affiliation=Univ B ; email=b@x.org"#,
    ],
    r#"row R REVTITLETHANKS (proposed; SHRESTb residual: a REVTeX title `\thanks` lands on the last author)"#
  ),
  row!(
    "fmmarks_revtex_title_thanks_then_shared_thanks_red",
    0,
    [
      r#"Ann Able ; thanks[†]=Equal contribution. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    r#"row R REVTITLETHANKS (proposed; SHRESTb residual: a REVTeX title `\thanks` lands on the last author)"#
  ),
  row!(
    "fmmarks_revtex_altaffiliation_then_shared_thanks_red",
    0,
    [
      r#"Ann Able | altaffiliation=Univ B ; affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    r#"NEITHER: reported to main (a content-check Warning for "Univ B", printed as Ann's altaffiliation and the others' affiliation)"#
  ),
];
/// REVTeX numbering: its `\thanks`, `\email`, `\altaffiliation` and name `\footnote` share one counter (4.0) or the
/// minipage letters (4.1/4.2 name footnotes); a note shared by names is one marked note per carrier, a one-carrier note
/// its contact; the affiliation's linking mark no note (1607.07280, 2301.00220, 2301.00155, 2605.04155, 2407.11130).
#[test]
fn revtex_marks_follow_its_numbering() { assert_green(REVTEX_GREEN); }
/// The known gaps of [`revtex_marks_follow_its_numbering`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn revtex_marks_follow_its_numbering_red() { assert_red_rows_now_green(REVTEX_RED); }

/// The GREEN rows of [`notes_are_marked_only_where_the_class_prints_marks`].
const CLASS_GREEN: &[Row] = &[
  row!("ieee_thanks_leading_label_is_no_mark", 0, [
    r#"Ann Able ; thanks[]=3 He detectors were funded by X."#,
    r#"Bob Baker ; thanks[]=* marks the starred items below."#,
  ]),
  row!("thanks_leading_symbol_text_is_no_mark_copy", 0, [
    r#"Ann Able ; thanks[*]=∗ -algebras are her field."#,
    r#"Bob Baker ; thanks[†]=3 He detectors were funded by X."#,
  ]),
  row!("thanks_bare_symbol_is_the_notes_mark", 0, [
    r#"Ann Able | affiliation=Univ A, Town"#,
    r#"Bob Baker ; thanks[*]=Corresponding author. | affiliation=Univ A, Town"#,
  ]),
  row!("fmmarks_thanks_text_opening_symbols_kept", 0, [
    r#"Ann Able ; thanks[*]=3 He detectors were funded by X."#,
    r#"Bob Baker ; thanks[†]=* marks the starred items."#,
    r#"Cy Cole ; thanks[‡]=∗ -algebras are studied."#,
  ]),
  row!("fmmarks_ieee_journal_thanks_unmarked", 0, [
    r#"Ann Able | membership=Student Member, IEEE"#,
    r#"Bob Baker ; thanks[]=A. Able and B. Baker are with Univ A (e-mail: ann@a.edu). ; thanks[]=Part of this research was presented at Conf X. This research is supported by Y. | membership=Senior Member, IEEE"#,
  ]),
];
/// The RED rows of [`notes_are_marked_only_where_the_class_prints_marks`].
const CLASS_RED: &[Row] = &[
  row!(
    "titling_thanksmarkseries_marks_red",
    0,
    [
      r#"Ann Able ; thanks[1]=First note."#,
      r#"Bob Baker ; thanks[2]=Second note."#,
    ],
    r#"row R SHAUTO"#
  ),
  row!(
    "maketitle_redefined_prints_no_marks_red",
    0,
    [
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    r#"row R SHAUTO"#
  ),
];
/// Class-printed marks against none: a note carries a mark only where its class prints one (IEEEtran and ieeeconf none,
/// 2606.00397, 1510.02728), and a symbol opening its text that is the text's own stays text (review revSHMARK S2).
#[test]
fn notes_are_marked_only_where_the_class_prints_marks() { assert_green(CLASS_GREEN); }
/// The known gaps of [`notes_are_marked_only_where_the_class_prints_marks`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn notes_are_marked_only_where_the_class_prints_marks_red() {
  assert_red_rows_now_green(CLASS_RED);
}
