//! WRONGFIX (DG 2026-11-16: wrong, invented or mis-linked marks are zero-tolerance, a wrong glyph worse than none): four
//! marks the 3,000-paper dev sample showed wrong, one test per code path, each row asserting its repro's pdflatex oracle
//! in the FMRG suite's two layers (`perfect_kernel_batchfmmarks::Row`): a note between two names is the name's before,
//! the next name's only at a line's or cell's start or under the block's edge proof (`resolver::mark_ledger`,
//! 2406.12338; DG 2026-11-17; WRONGFIXb); a symbol typed at the head of a thanks note no name prints is its text
//! (`insert::mark_creator_notes`, 2204.08920); a mark character inside a name's bracket is the name's ink
//! (`events::lexer::ink`, `resolver::lines::joined_single_name`, 1107.2497); an elsarticle `\cortext` is marked by its
//! carriers' route (`elsarticle_cls`'s `\lx@els@cormark`, 2512.04868, 1308.5495).
use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batchfmmarks::{Row, assert_green, creator_lines, page_lines, page_of};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A note between two names is the next name's only under the block's edge proof — its first note before its first
/// name, none after its last: `\thanks{…}Alice One, \thanks{…}Bob Two` (2406.12338, math0606768: each name given the
/// next name's note before), braced with its comma or not. No proof, the default, never the next name: a lone text
/// `\footnotemark` past a spaced comma (undecided, its Warning), and a `\\` line opening with a mark, maybe an
/// affiliation's (revWRONGFIX S2). Distinct paths: a note past a spaced comma, past a glued one, the Mark event, and a
/// line start that proves nothing.
#[test]
fn a_note_before_a_name_is_that_names() {
  assert_green(&[
    Row {
      repro:      "wrongfix_thanks_before_each_name_is_that_names",
      tex:        repro!("wrongfix_thanks_before_each_name_is_that_names"),
      warnings:   0,
      creators:   &[
        "Alice One ; thanks[*]=Alice note.",
        "Bob Two ; thanks[†]=Bob note.",
        "Carol Three ; thanks[‡]=Carol note.",
      ],
      moves:      "",
      page:       &[
        "Alice One ; * plain",
        "Bob Two ; † plain",
        "Carol Three ; ‡ plain",
        "margin: Alice note. ; Bob note. ; Carol note.",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_thanks_before_each_braced_name_is_that_names",
      tex:        repro!("wrongfix_thanks_before_each_braced_name_is_that_names"),
      warnings:   0,
      creators:   &[
        "Alice One ; thanks[*]=Alice note.",
        "Bob Two ; thanks[†]=Bob note.",
        "Carol Three ; thanks[‡]=Carol note.",
      ],
      moves:      "",
      page:       &[
        "Alice One ; * plain",
        "Bob Two ; † plain",
        "Carol Three ; ‡ plain",
        "margin: Alice note. ; Bob note. ; Carol note.",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_footnotemark_after_a_spaced_comma_goes_to_no_next_name",
      tex:        repro!("wrongfix_footnotemark_after_a_spaced_comma_goes_to_no_next_name"),
      warnings:   1,
      creators:   &["Alice One", "Bob Two"],
      moves:      "",
      page:       &["Alice One", "Bob Two"],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_affiliation_line_mark_proves_no_left_side",
      tex:        repro!("wrongfix_affiliation_line_mark_proves_no_left_side"),
      warnings:   0,
      // (the line's `\footnotemark[1]` prints Note A's ∗: the affiliation follows it)
      creators:   &[
        "Ann Able ; thanks[*]=Note A. | affiliation=University of Somewhere",
        "Bob Baker",
      ],
      moves:      "",
      page:       &["Ann Able | under: Note A.", "Bob Baker"],
      page_moves: "",
    },
  ]);
}

/// A note set off by its own single spaces between two names on a line is the name's before, though article's `\rlap`
/// prints its mark against the next name (article.cls:206): the e-mail in each note names its owner (revWRONGFIX B1 —
/// 1405.7029, 2109.12466, 1811.10030, 2605.31402, 1412.3142, 1705.11098, 1711.04571, 2002.10402, each note moved to the
/// next author by its spaces). Distinct: a plain space each side; a `\,`, no Gap event, read as the note's own space;
/// labels before it, in math and in text; a glued comma and labels; revtex's names line.
#[test]
fn a_note_between_two_names_is_the_name_befores() {
  assert_green(&[
    Row {
      repro:      "wrongfix_thanks_between_spaces_is_the_name_befores",
      tex:        repro!("wrongfix_thanks_between_spaces_is_the_name_befores"),
      warnings:   0,
      creators:   &[
        "Ann Able ; thanks[*]=ann@x.org",
        "Bob Baker ; thanks[†]=bob@x.org",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; * plain",
        "Bob Baker ; † plain",
        "margin: ann@x.org ; bob@x.org",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_thanks_after_a_thin_space_is_the_name_befores",
      tex:        repro!("wrongfix_thanks_after_a_thin_space_is_the_name_befores"),
      warnings:   0,
      creators:   &[
        "Ann Able ; thanks[*]=ann@x.org",
        "Bob Baker ; thanks[†]=bob@x.org",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; * plain",
        "Bob Baker ; † plain",
        "margin: ann@x.org ; bob@x.org",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_thanks_after_labels_and_a_thin_space_is_the_name_befores",
      tex:        repro!("wrongfix_thanks_after_labels_and_a_thin_space_is_the_name_befores"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[1,2]= ; thanks[*]=ann@x.org",
        "Bob Baker ; footnotemark[3]= ; thanks[†]=bob@x.org",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; 1,2 plain ; * plain",
        "Bob Baker ; 3 plain ; † plain",
        "margin: ann@x.org ; bob@x.org",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_thanks_after_a_comma_and_labels_is_the_name_befores",
      tex:        repro!("wrongfix_thanks_after_a_comma_and_labels_is_the_name_befores"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[1]= ; thanks[*]=ann@x.org",
        "Bob Baker ; footnotemark[2]= ; thanks[†]=bob@x.org",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; 1 plain ; * plain",
        "Bob Baker ; 2 plain ; † plain",
        "margin: ann@x.org ; bob@x.org",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_thanks_after_a_textsuperscript_is_the_name_befores",
      tex:        repro!("wrongfix_thanks_after_a_textsuperscript_is_the_name_befores"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[1]= ; thanks[*]=ann@x.org",
        "Bob Baker ; footnotemark[2]= ; thanks[†]=bob@x.org",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; 1 plain ; * plain",
        "Bob Baker ; 2 plain ; † plain",
        "margin: ann@x.org ; bob@x.org",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_revtex_thanks_after_labels_is_the_name_befores",
      tex:        repro!("wrongfix_revtex_thanks_after_labels_is_the_name_befores"),
      warnings:   0,
      creators:   &[
        "S. D. Odintsov ; thanks[]=odintsov@ieec.uab.es | affiliation=1) ICREA, Barcelona, Spain ; affiliation=2) IEEC, Barcelona, Spain",
        "V.K. Oikonomou ; thanks[]=v.k.oikonomou1979@gmail.com | affiliation=3) TUSUR, Tomsk, Russia ; affiliation=4) TSPU, Tomsk, Russia",
      ],
      moves:      "",
      page:       &[
        "S. D. Odintsov | under: odintsov@ieec.uab.es",
        "V.K. Oikonomou | under: v.k.oikonomou1979@gmail.com",
      ],
      page_moves: "",
    },
  ]);
}

/// A symbol typed at the head of a thanks note is the note's mark only where its creator shows that mark: under
/// INTERSPEECH2022.sty's `\thanks`, which prints none on the name (`\gdef\thefootnote{}`, :184), the body's `$^*$`
/// stays its text, as the legend prints it "∗ We will release …" (2204.08920; Bob's note was given the mark ∗ no name
/// prints). The style's title code is the repro's `INTERSPEECH2022.sty`, written beside it as a paper ships it (the copy
/// read after the stand-in binding).
#[test]
fn a_symbol_no_name_prints_stays_the_notes_text() {
  let sty =
    include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/INTERSPEECH2022.sty");
  let (log, xml, page) = convert_files_with_setup_then(
    repro!("wrongfix_unmarked_thanks_keeps_its_typed_symbol"),
    &[("INTERSPEECH2022.sty", sty)],
    Some("ar5iv.sty"),
    || {},
    |xml| page_lines(&page_of(xml)),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creator_lines(&xml), [
    "Alice One",
    "Bob Two ; thanks[]=∗ We will release the code."
  ]);
  let (lines, problems): (Vec<String>, Vec<String>) = page;
  assert!(problems.is_empty(), "{problems:?}");
  assert_eq!(lines, [
    "Alice One",
    "Bob Two | under: ∗ We will release the code."
  ]);
}

/// The same rule under IEEEtran, whose `\thanks` prints no mark: a note led by `\IEEEauthorrefmark{1}` no name prints
/// keeps the symbol as its text — neither the footnote symbol nor the creator's own sequence label `author:1` is a mark
/// the name shows (2405.05531; a draft read `author:1` as the mark "1" its reference mark keys by). Distinct from the
/// INTERSPEECH row: the reference mark's number against the creator's labels.
#[test]
fn an_ieee_reference_mark_no_name_prints_stays_the_notes_text() {
  assert_green(&[Row {
    repro:      "wrongfix_ieee_refmark_on_no_name_stays_text",
    tex:        repro!("wrongfix_ieee_refmark_on_no_name_stays_text"),
    warnings:   0,
    creators:   &[
      "Ann Able ; thanks[]=* Corresponding author: ann@x.org ; thanks[]=A. Able is with Univ A.",
    ],
    moves:      "",
    page:       &["Ann Able | under: * Corresponding author: ann@x.org ; A. Able is with Univ A."],
    page_moves: "",
  }]);
}

/// A mark character in text inside a bracket the name's run opened is the bracket's ink: `H. Chen (x\P)` is the name
/// "H. Chen (x¶)", its ")" kept, no mark (1107.2497's GB2312 name read as "³»¶" without the CJK package, its ¶ a mark-only
/// note and the ")" lost before); and in revtex's names line such a name counts by its name before the bracket, so the
/// list is four names, not one (`joined_single_name`; one creator and two Warnings before, with any bracket).
#[test]
fn a_mark_character_inside_a_names_bracket_is_its_text() {
  assert_green(&[
    Row {
      repro:      "wrongfix_mark_char_inside_a_names_bracket_is_ink",
      tex:        repro!("wrongfix_mark_char_inside_a_names_bracket_is_ink"),
      warnings:   0,
      creators:   &["H. Chen (x¶)", "M. Baldo"],
      moves:      "",
      page:       &["H. Chen (x¶)", "M. Baldo"],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_revtex_name_list_with_a_bracket_remark",
      tex:        repro!("wrongfix_revtex_name_list_with_a_bracket_remark"),
      warnings:   0,
      creators:   &[
        "H. Chen (x¶) | affiliation=INFN",
        "M. Baldo | affiliation=INFN",
        "G. F. Burgio | affiliation=INFN",
        "H.-J. Schulze | affiliation=INFN",
      ],
      moves:      "",
      page:       &["H. Chen (x¶)", "M. Baldo", "G. F. Burgio", "H.-J. Schulze"],
      page_moves: "",
    },
  ]);
}

/// An elsarticle `\cortext` note is marked by its carriers' route: names listing its key in `\author[…]` print the `cnote`
/// number (elsarticle.cls:181-185, :251-256), "1" — 2512.04868's two names both marked ∗ before —, whether the
/// `\cortext` comes after the authors or before them, and nothing where an `\address` shares the key, its letter printed
/// instead (:126-131, 1711.08299); names reaching it by `\corref` print ∗ (:188-190, 1308.5495); and where the number is
/// another mark of the same name's too (an `\fntext` key's), the ∗ the footnote prints, two notes never sharing a mark
/// (revWRONGFIX S1). Distinct: the key route, its decision deferred past the authors, the key an address shares, the
/// `\corref` route, and the number taken.
#[test]
fn an_elsarticle_cortext_is_marked_by_its_route() {
  assert_green(&[
    Row {
      repro:      "wrongfix_elsarticle_cortext_key_prints_its_number",
      tex:        repro!("wrongfix_elsarticle_cortext_key_prints_its_number"),
      warnings:   0,
      creators:   &[
        "Alice One ; thanks[1]=Corresponding author | affiliation=Some University, X, Y",
        "Bob Two ; thanks[1]=Corresponding author | affiliation=Some University, X, Y",
      ],
      moves:      "",
      page:       &[
        "Alice One ; 1→note",
        "Bob Two ; 1→note",
        "margin: Corresponding author",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_elsarticle_cortext_before_its_keyed_authors",
      tex:        repro!("wrongfix_elsarticle_cortext_before_its_keyed_authors"),
      warnings:   0,
      creators:   &[
        "Alice One ; thanks[1]=Corresponding author | affiliation=Some University, X, Y",
        "Bob Two ; thanks[1]=Corresponding author | affiliation=Some University, X, Y",
      ],
      moves:      "",
      page:       &[
        "Alice One ; 1→note",
        "Bob Two ; 1→note",
        "margin: Corresponding author",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_elsarticle_cortext_key_shared_with_an_address",
      tex:        repro!("wrongfix_elsarticle_cortext_key_shared_with_an_address"),
      warnings:   0,
      creators:   &[
        "Alice One | address=Some University ; correspondent=Corresponding author",
        "Bob Two | address=Some University ; correspondent=Corresponding author",
      ],
      moves:      "",
      page:       &["Alice One", "Bob Two"],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_elsarticle_cortext_number_taken_falls_back_to_star",
      tex:        repro!("wrongfix_elsarticle_cortext_number_taken_falls_back_to_star"),
      warnings:   0,
      creators:   &[
        "Alice One ; thanks[*]=Cor C ; thanks[1]=Footnote F | address=Some University",
        "Bob Two ; thanks[*]=Cor C ; thanks[1]=Footnote F | address=Some University",
      ],
      moves:      "",
      page:       &[
        "Alice One ; 1→note ; *→note",
        "Bob Two ; 1→note ; *→note",
        "margin: Cor C ; Footnote F",
      ],
      page_moves: "",
    },
    Row {
      repro:      "wrongfix_elsarticle_corref_shared_stays_star",
      tex:        repro!("wrongfix_elsarticle_corref_shared_stays_star"),
      warnings:   0,
      creators:   &[
        "Alice One ; thanks[*]=Corresponding author | affiliation=Some University, X, Y",
        "Bob Two ; thanks[*]=Corresponding author | affiliation=Some University, X, Y",
      ],
      moves:      "",
      page:       &[
        "Alice One ; *→note",
        "Bob Two ; *→note",
        "margin: Corresponding author",
      ],
      page_moves: "",
    },
  ]);
}
