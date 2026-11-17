//! Guards for NE2 (`agent_reports/2026-11-09_NE2_report.md`): Errors `=2` raised that off doesn't, the
//! flip's no-harm gate. One guard per mechanism, each off and at `=2`: the footnote counter's steps of an author block's
//! notes run where they print (1507.05018, 2505.07115); an IJCAI sectioned block goes to its section split, its
//! `labelseq=author` affiliation the N-th author's (2202.05511); initials a gap joins to a name end no surname-first name
//! (cond-mat9905290); "representing" speaks for a collaboration (hep-ex9903035). A creator's labels written back as plain
//! characters (1910.05766) is the unit test `emitter::emit::tests::a_creators_labels_are_plain_characters`. NE2b (review
//! revNE2): a counter move no name carries writes the block whole, with its Warning (1507.05018's Ombao, A1); a note
//! before every name steps before the first name (A2). NE2c (review revNE2r2 Q2): a mark a name prints past its note stays
//! with it, the note's step before it (1610.06031), and so does one past spacing (1911.13299's `\ \footnotemark[2]`; DG
//! ruling 2026-11-16); a `\footnotemark` no name and no item writes (on the line a `\\` opens) wrote the block whole,
//! with its Warning. NE2d (user ruling 2026-11-09): a dropped mark given its number keeps the creators, its loss
//! counted (`\samethanks[1]`, 1912.01737; 1811.08622). SHGNEXT (row R SHGIVENNEXT; DG rulings 2026-11-16): one mark
//! ledger decides which name a mark or note beside spacing goes to, the nearer by the glue printed, a tie the next
//! name's (2412.20504), the next name's written before its text; a run holding other ink apart from both names, or one a
//! fill's width decides, stays undecided and counted; a bare mark no name writes keeps its footnote step where it
//! prints, its glyph counted lost (the block was written whole).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators, perfect_kernel_batchshmark::notes_in,
  perfect_kernel_batchz3::in_a_directory,
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

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex`: the log and the XML.
fn run(tex: &str) -> (String, String) {
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, &[], Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// A note's footnote step is its creator's, where it prints: `=2` wrote the notes apart, after every creator, so Bob
/// Baker's `\addtocounter{footnote}{-2}\footnotemark` ran at the counter's 0 (5 x "Counter too large", the mark empty);
/// now his mark is the "*" pdflatex prints, as off's.
/// FN1 (OD #1001; the user's ruling 2026-11-11): the note the name held is the creator's, right after
/// `</personname>`, so the name reads without its tags; the notes and marks are asserted kept below; pin updated (the
/// note's move only).
#[test]
fn a_notes_footnote_step_runs_where_it_prints() {
  let (log, xml) = run(repro!("ne2_thanks_addtocounter_order"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "mark=*", "mark=†"]),
    row(&["Bob Baker", "mark=*"])
  ]);
  let notes: Vec<&str> = xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
    .collect();
  assert_eq!(
    notes,
    [
      "<note class=\"ltx_note_frontmatter ltx_thanks_note\" mark=\"*\" role=\"thanks\" \
       xml:id=\"id1\"",
      "<note class=\"ltx_note_frontmatter ltx_thanks_note\" mark=\"†\" role=\"thanks\" \
       xml:id=\"id2\"",
      "<note class=\"ltx_note_frontmatter ltx_thanks_note\" idref=\"id1\" mark=\"*\" role=\"thanks\" xml:id=\"footnote1\"",
    ],
    "{xml}"
  );
}

/// kr.sty's `\affiliations` and `\emails` are defined only in its `\@maketitle`: `=2` queued the block whole before the
/// IJCAI split, and its piece `Bob Baker \affiliations Univ X` was digested (undefined `\affiliations`). Now the split
/// runs first, as off. Its `labelseq=author` affiliation is Perl's `author:1`, Ann Able's, off; `=2` reads it as the
/// page prints it, under both names (2202.05511 alike).
#[test]
fn an_ijcai_block_is_split_at_its_sections() {
  let sty = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ne2_kr_affiliations.sty"
  );
  let univ = "affiliation=Univ X, City, Country";
  let bob = row(&["Bob Baker", univ, "email=bob@x.edu"]);
  let expected = [row(&["Ann Able", univ, "email=ann@x.edu"]), bob];
  let (log, xml) = in_a_directory(
    repro!("ne2_kr_affiliations"),
    &[("ne2_kr_affiliations.sty", sty)],
    &[],
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected);
}

/// `F. M.\, Munteanu` after a comma: the surname-first rule (`Santucho, V.`) joined the initials to "Yongmin Kim" and the
/// thin space's gap rule joined "Munteanu" after them, one creator (merged_creators under `=2`). Initials a gap joins to a
/// name end no piece: four creators, each with what its marks request — off too, whose † affiliation went to a nameless
/// creator.
#[test]
fn initials_a_thin_space_joins_are_a_name_of_their_own() {
  let lab = "affiliation=∗ Lab A, Los Alamos";
  let dept = "affiliation=† Dept B, Boston";
  let expected = [
    row(&["Yongmin Kim", lab]),
    row(&["F. M. Munteanu", lab, dept]),
    row(&["C. H. Perry", lab, dept]),
    row(&["D. G. Rickel", lab]),
  ];
  let (log, xml) = run(repro!("ne2_merged_kim"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected);
}

/// `\author{representing the ATLAS Pixel Collaboration}`, a call of its own: the names' collaboration, as off frames it
/// ("representing" in `SPEAKS_FOR`), not a creator (creator_not_a_name under `=2`).
#[test]
fn representing_a_collaboration_is_its_contact() {
  let expected = [row(&[
    "John Richardson",
    "affiliation=Lawrence Berkeley Laboratory",
    "collaboration=representing the ATLAS Pixel Collaboration",
  ])];
  let (log, xml) = run(repro!("ne2_representing_collab"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected);
}

/// An author block whose digestion moves the footnote counter where no name's run or mark carries it (Bob Baker's
/// `\addtocounter{footnote}{-2}` between his notes, 1507.05018's Ombao): `=2` lost the move and Ann Able's
/// `\footnotemark` after her note, so Carl Cole printed † where LaTeX prints *, with no Warning (review revNE2 A1). The
/// block is written as Perl's `\lx@add@authors` splits it at its `\and`s (CR7, OD #988: it was one whole-block creator,
/// Perl's tabular branch), its pieces digested in order: Ann †, Carl *, every note kept, the counted Warning. Off reads it per name, as pdflatex prints the marks. (A move a mark's box covers,
/// `\addtocounter{footnote}{-2}\footnotemark`, is carried: `a_notes_footnote_step_runs_where_it_prints`.)
/// FN1 (OD #1001; the user's ruling 2026-11-11): the note the name held is the creator's, right after
/// `</personname>`, so the name reads without its tags; the notes and marks are asserted kept below; pin updated (the
/// note's move only).
#[test]
fn a_counter_move_no_name_carries_writes_the_block_as_perl_splits_it() {
  let expected = vec![
    row(&["Ann Able", "mark=†", "mark=*"]),
    row(&["Bob Baker", "mark=‡", "mark=†"]),
    row(&["Carl Cole", "mark=*"]),
  ];
  let warnings = 1;
  let (log, xml) = run(repro!("ne2b_counter_moved_between_notes"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  assert_eq!(creators(&xml), expected);
  let notes: Vec<&str> = xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
    .collect();
  // (every note kept, each mark as pdflatex prints it: Ann's footnotemark †, Carl's *; each creator's in order, Ann's mark
  // and thanks, Bob's two thanks, Carl's mark)
  let (mark_ann, mark_carl) = (
    "<note class=\"ltx_note_frontmatter ltx_thanks_note\" mark=\"†\" role=\"thanks\" xml:id=\"footnote2\"",
    "<note class=\"ltx_note_frontmatter ltx_thanks_note\" idref=\"id1\" mark=\"*\" role=\"thanks\" xml:id=\"footnote1\"",
  );
  // (each `\thanks` with the mark pdflatex prints for it, OD #1053; Ann's copy of Bob's † the first in document order,
  // whose id Bob's names, OD #1059; revMARKKEY1 NIT: Bob's held it before)
  let thanks = |n: u8, mark: &str| {
    format!(
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="{mark}" role="thanks" xml:id="id{n}""#
    )
  };
  let expected = vec![
    mark_ann.to_string(),
    thanks(1, "*"),
    thanks(2, "‡"),
    "<note class=\"ltx_note_frontmatter ltx_thanks_note\" idref=\"footnote2\" mark=\"†\" role=\"thanks\" xml:id=\"id3\""
      .to_string(),
    mark_carl.to_string(),
  ];
  assert_eq!(notes, expected, "{xml}");
}

/// A note before every name steps the counter before the first creator's name, where LaTeX reads it: `=2` stepped it
/// after Ann Able's personname, so her `\footnotemark` printed * where pdflatex prints † (review revNE2 A2). Distinct
/// from `a_notes_footnote_step_runs_where_it_prints`: the step before a name, not after one.
/// FN1 (OD #1001; the user's ruling 2026-11-11): the note the name held is the creator's, right after
/// `</personname>`, so the name reads without its tags; the notes and marks are asserted kept below; pin updated (the
/// note's move only).
#[test]
fn a_note_before_every_name_steps_before_it() {
  let (log, xml) = run(repro!("ne2b_note_before_every_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "mark=†", "mark=*"]),
    row(&["Bob Baker"])
  ]);
}

/// The marks' notes of `xml` (a `\footnotemark`'s, an `\endnotemark`'s), each its opening tag, in document order.
fn footnotemarks(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
    // (a mark-only note's tag closes itself, `<note mark="1" role="footnotemark" xml:id="id1"/>`: PURE)
    .map(|tag| tag.trim_end_matches('/'))
    .filter(|tag| tag.contains(" mark="))
    .collect()
}

/// Each creator's [`footnotemarks`], in document order: a mark that moved to a neighbouring creator fails.
fn marks_by_creator(xml: &str) -> Vec<Vec<&str>> {
  xml
    .split("<creator ")
    .skip(1)
    .map(|c| footnotemarks(&c[..c.find("</creator>").expect("a creator's end")]))
    .collect()
}

/// A mark a name prints past its note stays with the name, in source order, the note's footnote step before it: `=2`'s
/// past-marks scan stopped at the note, so Ann Able's `\footnotemark` was dropped with no Warning and the next name's mark
/// printed one symbol short (Bob †, Carl ‡). Each shape off and at `=2` as pdflatex prints it: glued, past `\ `, `~` and
/// `\,`, a given `\footnotemark[1]`, past a `\footnote`, past a formula's `$^,$` (1610.06031's `G.
/// Citti\thanks{...}\footnotemark[1]`, review revNE2r2 Q2; 1703.08733's
/// `\protect\endnotemark[1]$^,$\protect\endnotemark[2]`). Distinct from `a_notes_footnote_step_runs_where_it_prints`:
/// the note's step inside the name's tokens, between its text and its mark. SHGIVEN (DG ruling 2026-11-16, 4): a mark
/// past a spacing run (` \ `, `\quad`, `~~`, `\hspace`: a Gap) is its name's too — a bare one wrote the block whole
/// with a Warning (1911.13299's shape, `ne2e_mark_past_spacing_is_its_names`); the given ones are
/// `a_given_mark_on_no_name_keeps_the_creators`'.
/// FN1 (OD #1001; the user's ruling 2026-11-11): the note the name held is the creator's, right after
/// `</personname>`, so the name reads without its tags; the notes and marks are asserted kept below; pin updated (the
/// note's move only).
#[test]
fn a_mark_after_its_names_note_stays_with_it() {
  let mark = |m: &str, n: &str| format!(r#"<note mark="{m}" role="footnotemark" xml:id="{n}""#);
  // (SHMARK, OD #1053: each `\thanks` note with the mark it prints, and a given mark a copy of the note it marks)
  let thanks = |m: &str, n: &str| {
    format!(
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="{m}" role="thanks" xml:id="{n}""#
    )
  };
  for (tex, expected, marks) in [
    (
      repro!("ne2c_mark_after_note"),
      vec![
        row(&["Ann Able", "mark=†", "mark=*"]),
        row(&["Bob Baker", "mark=‡"]),
      ],
      vec![
        mark("†", "footnote2"),
        thanks("*", "id1"),
        mark("‡", "footnote3"),
      ],
    ),
    (
      repro!("ne2c_mark_after_note_spaced"),
      vec![
        row(&["Ann Able", "mark=†", "mark=*"]),
        row(&["Bob Baker", "mark=‡"]),
        row(&["Carl Cole", "mark=§"]),
      ],
      vec![
        mark("†", "footnote2"),
        thanks("*", "id1"),
        thanks("‡", "id2"),
        mark("§", "footnote4"),
      ],
    ),
    (
      repro!("ne2c_mark_after_note_numbered"),
      vec![
        row(&["Ann Able", "mark=*", "mark=*"]),
        row(&["Bob Baker", "mark=*"]),
        row(&["Carl Cole", "mark=†"]),
      ],
      vec![
        mark("*", "footnotex1"),
        thanks("*", "id1"),
        // (one note with Ann's, `idref`: OD #1059)
        thanks("*", "footnotex2").replacen(" mark=", r#" idref="id1" mark="#, 1),
        thanks("†", "id2"),
      ],
    ),
    (
      repro!("ne2c_mark_after_footnote_glued"),
      vec![
        row(&["Ann Able", "mark=†", "mark=*"]),
        row(&["Bob Baker", "mark=*", "mark=‡"]),
        row(&["Carl Cole", "mark=§"]),
      ],
      vec![
        mark("†", "footnote2"),
        thanks("*", "id1"),
        // (one note with Ann's, `idref`: OD #1059)
        thanks("*", "footnotex1").replacen(" mark=", r#" idref="id1" mark="#, 1),
        thanks("‡", "id2"),
        mark("§", "footnote4"),
      ],
    ),
    (
      repro!("ne2c_mark_after_math_comma"),
      vec![
        row(&["Ann Able", "mark=1"]),
        // (the endnote marks beside the name too; the `$^,$` printed between them is left at the name's end)
        row(&["Bob Baker", "mark=1", "mark=2"]),
      ],
      ["1", "2", "3"]
        .iter()
        .zip(["1", "1", "2"])
        .map(|(n, m)| {
          format!(r#"<note inlist="ent" mark="{m}" role="endnotemark" xml:id="endnotex{n}""#)
        })
        .collect(),
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), expected);
    assert_eq!(footnotemarks(&xml), marks, "{xml}");
  }
  // (past spacing, each creator its own marks: Ann † bare, after her note's step; Bob * and Carl † given; Dan ‡ bare;
  // Bob's `[1]` a copy of Ann's `\thanks` note, OD #1053's Link step; Carl's `[2]` marks a bare mark, no note to copy)
  let (log, xml) = run(repro!("ne2e_mark_past_spacing_is_its_names"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "mark=†", "mark=*"]),
    row(&["Bob Baker", "mark=*"]),
    row(&["Carl Cole", "mark=†"]),
    row(&["Dan Dory", "mark=‡"]),
  ]);
  assert_eq!(
    marks_by_creator(&xml),
    [
      vec![mark("†", "footnote2"), thanks("*", "id1")],
      vec![thanks("*", "footnotex1").replacen(" mark=", r#" idref="id1" mark="#, 1)],
      vec![mark("†", "footnotex2")],
      vec![mark("‡", "footnote3")],
    ],
    "{xml}"
  );
}

/// A bare `\footnotemark` no name of the reading writes and no item holds (on the line a `\\` opens under Ann Able's name:
/// no name's, DG ruling 2026-11-16, 4) was dropped with no Warning, Bob Baker's mark one symbol short; NE2c then wrote the
/// block whole, Perl's whole-block creator, the two names clumped into one. Now its footnote step is kept where it prints
/// (the mark typeset into a box no one uses) and its glyph counted lost (row R SHGIVENNEXT's counter-step rule): Ann with
/// her note ∗, Bob ‡ as pdflatex prints them, 1 Warning. One that opens a line or cell with a name after it keeps no
/// step apart: the reading took the cell `\and \footnotemark Bob Baker` for a labelled line, Bob Ann's affiliation, so
/// the block is still written whole (REGFIX's, its Warning). Distinct from
/// `a_counter_move_no_name_carries_writes_the_block_as_perl_splits_it`: a mark, not a counter move, that the reading has
/// no place for.
#[test]
fn a_mark_on_no_name_keeps_its_footnote_step() {
  let (log, xml) = run(repro!("ne2c_mark_on_no_name_keeps_its_footnote_step"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("on no name and no item, their footnote steps kept where they print; lost: †"),
    "{log}"
  );
  assert_eq!(notes_in(&xml), [
    row(&["Ann Able", "thanks[*]=Univ X."]),
    row(&["Bob Baker", "footnotemark[‡]="]),
  ]);
  let (log, xml) = run(repro!(
    "ne2i_bare_mark_opening_a_cell_writes_the_block_whole"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("on no name and no item; the block is written whole"),
    "{log}"
  );
  assert_eq!(notes_in(&xml), [row(&[
    "Ann Able and Bob Baker",
    "footnotemark[†]=",
    "thanks[*]=X"
  ])]);
}

/// A `\footnotemark` given its number that no name and no item writes (`D. R. G. Schleicher,$^{1}$\samethanks[1]`,
/// 1912.01737's document `\samethanks`; `Cheng Xu\textsuperscript{1},\footnotemark[1]`, 1811.08622) re-references a note
/// written already and steps no counter (latex.ltx's `\@xfootnotemark`), so no later mark moves: NE2c's decline wrote the
/// block whole and threw away a right reading. Now the names are written as read, the marks' loss counted by 1 Warning
/// naming them (user ruling 2026-11-09). A block that also drops a bare `\footnotemark` (1111.1979's shape) keeps its
/// creators too, the bare mark's step kept where it prints (`a_mark_on_no_name_keeps_its_footnote_step`): both losses
/// one Warning, Dan's mark ‡ as pdflatex's. Distinct from `a_mark_on_no_name_keeps_its_footnote_step`: a mark that
/// steps no counter.
/// FN1 (OD #1001; the user's ruling 2026-11-11, which accepts this shape): a block written whole holds its notes
/// after the joint name, each with its mark, so the name reads without their tags; pin updated (the notes' move only).
/// SHGIVEN (DG ruling 2026-11-16, 4, refining the 2026-11-09 one): a given mark only spacing parts from its name is that
/// name's, as pdflatex prints it and Perl keeps it — 1911.13299's `Vivek Ramanujan \thanks{…} \ \footnotemark[2]`, its
/// four marks lost with a Warning ("lost: †, ‡, ‡, ‡"), now each on its name with no Warning (asserted first), each a
/// copy of the note it marks (SHMARK's Link step, OD #1053; pin updated at the merge); a given mark glued past a name's
/// comma (the rows after) is that name's since MARKKEY1 (COMMAMARK, OD #1057, #1058), its loss gone.
/// SHGNEXT (row R SHGIVENNEXT): the block dropping a bare mark is no longer written whole (its third row; pin updated).
#[test]
fn a_given_mark_on_no_name_keeps_the_creators() {
  let (log, xml) = run(repro!("author_thanks_shared_by_footnotemark_marks_kept"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "mark=†", "mark=*"]),
    row(&["Bob Baker", "mark=*", "mark=‡"]),
    row(&["Cat Cole", "mark=†", "mark=‡"]),
    row(&["Dan Dorn", "mark=‡"]),
  ]);
  // (pdflatex's "Ann Able ∗ †  Bob Baker ∗ ‡  Cat Cole † ‡  Dan Dorn ‡": each carried mark on its name as a copy of
  // the note it marks, SHMARK's Link step, OD #1053 — Ann's † and Dan's ‡ the carry's; a name's own `\thanks` note
  // after the marks its tokens write, as REGFIX orders them)
  assert_eq!(notes_in(&xml), [
    row(&[
      "Ann Able",
      "thanks[†]=Univ A",
      "thanks[*]=Equal contribution."
    ]),
    row(&[
      "Bob Baker",
      "thanks[*]=Equal contribution.",
      "thanks[‡]=Univ B"
    ]),
    row(&["Cat Cole", "thanks[†]=Univ A", "thanks[‡]=Univ B"]),
    row(&["Dan Dorn", "thanks[‡]=Univ B"]),
  ]);
  // (MARKKEY1, COMMAMARK — DG 2026-11-16: a mark glued past a name's comma and its label is the left name's, measured
  // from the comma's ink, 1912.01737, 1811.08622 — no longer lost: Bob's `\samethanks[1]` and Carl's `\footnotemark[1]`
  // are copies of Ann's note 1, OD #1058; the counted loss is gone)
  let expected = vec![
    row(&["Ann Able", "mark=*", "affiliation=Univ A"]),
    row(&["Bob Baker", "mark=*", "affiliation=Univ A"]),
    row(&["Carl Cole", "mark=*", "affiliation=Univ B"]),
    row(&["Dan Dory", "affiliation=Univ B"]),
  ];
  let (log, xml) = run(repro!("ne2d_given_mark_dropped_keeps_the_creators"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected);
  assert_eq!(notes_in(&xml), [
    row(&["Ann Able", "thanks[*]=Univ X."]),
    row(&["Bob Baker", "thanks[*]=Univ X."]),
    row(&["Carl Cole", "thanks[*]=Univ X."]),
    row(&["Dan Dory"]),
  ]);

  // (a bare `\footnotemark` past a comma too: Carl's †, Dan's own ‡, each a mark-only note on its name, its step kept)
  let (log, xml) = run(repro!("ne2d_given_and_fresh_mark_dropped"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "mark=*", "affiliation=Univ A"]),
    row(&["Bob Baker", "mark=*", "affiliation=Univ A"]),
    row(&["Carl Cole", "mark=†", "affiliation=Univ B"]),
    row(&["Dan Dory", "mark=‡", "affiliation=Univ B"]),
  ]);
  assert_eq!(notes_in(&xml), [
    row(&["Ann Able", "thanks[*]=Univ X."]),
    row(&["Bob Baker", "thanks[*]=Univ X."]),
    row(&["Carl Cole", "footnotemark[†]="]),
    row(&["Dan Dory", "footnotemark[‡]="]),
  ]);
}

/// A `\footnotemark` glued to the next name past spacing is the name's before it, glued proving nothing (DG 2026-11-17,
/// re-pinned from 2026-11-16's nearer-by-glue rule): `Ann Able \quad \footnotemark[2]Bob Baker` gives Ann the †, written
/// once — Bob's run, whose span the mark runs into, takes it no more (`emit::block_names`' `written_before`; the cut
/// first wrote † on both). REGFIX's Warnings for such a mark ("lost: †") over-counted: the ledger writes it, so they are
/// gone. A fill before it changes nothing (`Ann Able\thanks{X} \hfill \footnotemark Bob Baker`: Ann's ∗ †; REGFIX
/// wrote the block whole).
/// A mark inside a formula is written with the formula whole, its `\thanks` twice, so 2511.12633's `Xunzhi
/// Xiang$^{1}\thanks{…}\,\,\footnotemark[2]$` stays lost and counted. Distinct from
/// `a_mark_beside_spacing_is_the_nearer_names`: a mark glued to the next name.
#[test]
fn a_mark_a_name_follows_past_spacing_is_not_carried_back() {
  let mark = |m: &str, n: &str| format!(r#"<note mark="{m}" role="footnotemark" xml:id="{n}""#);
  let thanks = |m: &str, n: &str| {
    format!(
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="{m}" role="thanks" xml:id="{n}""#
    )
  };
  for (tex, (warnings, says), expected, marks, notes) in [
    (
      repro!("ne2e_prefix_mark_past_spacing_before_and"),
      (0, &[][..]),
      vec![
        row(&["Ann Able", "mark=†"]),
        row(&["Bob Baker"]),
        row(&["Carl Cole"]),
      ],
      vec![vec![mark("†", "footnotex1")], vec![], vec![]],
      0,
    ),
    (
      repro!("ne2e_prefix_marks_past_quad"),
      (
        1,
        &[
          "reaches no contact carrying its label, or a labelled contact no name's mark reaches (*)",
        ][..],
      ),
      vec![
        row(&["Ann Able", "mark=*"]),
        row(&["Bob Baker", "mark=†"]),
        row(&["Carl Cole", "mark=*"]),
      ],
      vec![
        vec![mark("*", "footnotex1")],
        vec![mark("†", "footnotex2")],
        vec![mark("*", "footnotex3")],
      ],
      0,
    ),
    (
      repro!("ne2e_prefix_mark_past_hfill"),
      (0, &[][..]),
      vec![row(&["Ann Able", "mark=†", "mark=*"]), row(&["Bob Baker"])],
      vec![vec![mark("†", "footnote2"), thanks("*", "id1")], vec![]],
      1,
    ),
    (
      repro!("ne2e_mark_in_formula_past_thin_spaces"),
      (1, &["on no name and no item; lost: ∗"][..]),
      vec![row(&["Ann Able", "mark=1", "mark=*"]), row(&["Bob Baker"])],
      vec![vec![mark("1", "id1"), thanks("*", "id2")], vec![]],
      1,
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    for says in says {
      assert!(log.contains(says), "{says}: {log}");
    }
    assert_eq!(creators(&xml), expected);
    assert_eq!(marks_by_creator(&xml), marks, "{xml}");
    assert_eq!(xml.matches(r#" role="thanks""#).count(), notes, "{xml}");
  }
}

/// A `\footnotemark` printed between two names belongs to the name before it, whatever the glue on each side, and opens
/// the next name only at a line's or cell's start or under the block's edge proof (DG 2026-11-17, superseding 2026-11-16's
/// nearer-by-glue rule these rows were first pinned on; `resolver::mark_ledger`). Rows (each 0 Warnings, re-pinned to the
/// default where the print is ambiguous): ` \qquad † \ `, ` \quad † ~~`, ` \quad †\quad` (Ann's); a bare mark before
/// Bob's own (Ann's †, his ‡); 2412.20504's `\footnotemark[1] \ \footnotemark[2] \quad Qingyi Si` (Ann's) and a mark
/// between Bob and Carl (Bob's ‡); a run between two names, whatever its widest gap (Ann's); one past each of two names on
/// a line; a mark between two interword spaces (Ann's); a bold name past the gap (Ann's); a mark opening the next
/// `tabular` cell (Bob's, review revSHGIVEN p3tab; TABCELLMARK). REGFIX lost them all ("lost: †"), or wrote the block whole, or
/// carried the tabular cell's mark back to Ann with no Warning; SHGIVENb-e carried them back or grew Bob's run over them
/// (reviews revSHGIVENb-e). Distinct from `a_mark_a_name_follows_past_spacing_is_not_carried_back`: spacing on both
/// sides of the marks.
#[test]
fn a_mark_beside_spacing_is_the_nearer_names() {
  let three = |ann: &[&str], bob: &[&str], carl: &[&str]| {
    let named = |name: &str, marks: &[&str]| {
      std::iter::once(name.to_string())
        .chain(marks.iter().map(|m| format!("footnotemark[{m}]=")))
        .collect::<Vec<_>>()
    };
    vec![
      named("Ann Able", ann),
      named("Bob Baker", bob),
      named("Carl Cole", carl),
    ]
  };
  for (tex, expected) in [
    (
      repro!("ne2f_mark_nearer_next_past_qquad"),
      three(&["†"], &[], &[]),
    ),
    (
      repro!("ne2f_mark_nearer_next_past_ties"),
      three(&["†"], &[], &[]),
    ),
    (
      repro!("ne2f_mark_nearer_next_past_quads"),
      three(&["†"], &[], &[]),
    ),
    (repro!("ne2f_bare_mark_nearer_next_past_quads"), vec![
      row(&["Ann Able", "footnotemark[†]=", "thanks[*]=X"]),
      row(&["Bob Baker", "footnotemark[‡]="]),
    ]),
    (
      repro!("ne2f_marks_wider_after_and_tied"),
      three(&["*", "†"], &["‡"], &[]),
    ),
    (
      repro!("ne2h_marks_run_wider_before_goes_to_the_next_name"),
      three(&["*", "†"], &[], &[]),
    ),
    (
      repro!("ne2h_marks_run_parted_at_its_widest_gap"),
      three(&["*", "†"], &[], &[]),
    ),
    (
      repro!("ne2h_marks_run_carried_back_whole"),
      three(&["*", "†"], &[], &[]),
    ),
    (
      repro!("fmmarks_marks_run_split_by_wide_space_red"),
      three(&["*", "†"], &[], &[]),
    ),
    (
      repro!("fmmarks_marks_three_names_one_line_red"),
      three(&["*"], &["†"], &[]),
    ),
    (
      repro!("fmmarks_mark_between_interword_spaces_red"),
      three(&["†"], &[], &[]),
    ),
    (
      repro!("fmmarks_bare_prefix_mark_after_quad_red"),
      three(&["*"], &[], &["†"]),
    ),
    (
      repro!("fmmarks_mark_carried_back_before_bold_name_red"),
      three(&["*"], &[], &[]),
    ),
    (
      repro!("fmmarks_mark_nearer_next_bold_name_red"),
      three(&["†"], &[], &[]),
    ),
    (repro!("fmmarks_tabular_cell_given_prefix_mark_red"), vec![
      row(&["Ann Able"]),
      row(&["Bob Baker", "footnotemark[†]="]),
    ]),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(notes_in(&xml), expected, "{xml}");
  }
}

/// A note printed beside spacing goes with the marks the mark ledger gives a name: the reading's owner of the note and the
/// writer's place of its footnote step are one decision — between two names the name's before, glued to the next name or
/// not (DG 2026-11-17, re-pinned from 2026-11-16's nearer-by-glue rule; `fmmarks_mark_then_thanks_before_next_name_red`'s
/// "Bob note" is Ann's by that default, the print ambiguous), its step in print order so the marks after it print theirs
/// (a `\thanks` glued to a bare mark; in either order beside a given mark; a bare mark and a note set apart, `\ \thanks{…}
/// \ `; a note glued to Bob's name past a mark). REGFIX kept the note on Ann (the
/// note's place in the source), lost the mark or wrote the block whole; revSHGIVENd found the owner hung on the source
/// order. Distinct from `a_mark_beside_spacing_is_the_nearer_names`: the reading's note owner and the step order.
#[test]
fn a_note_beside_spacing_goes_to_the_nearer_name() {
  for (tex, expected) in [
    (repro!("ne2f_mark_and_note_nearer_next"), vec![
      row(&["Ann Able", "footnotemark[†]=", "thanks[*]=X", "thanks[‡]=Y"]),
      row(&["Bob Baker"]),
      row(&["Carl Cole"]),
    ]),
    (repro!("fmmarks_gap_mark_then_thanks_red"), vec![
      row(&["Ann Able", "footnotemark[†]=", "thanks[*]=Ann note"]),
      row(&["Bob Baker"]),
      row(&["Carl Cole"]),
    ]),
    (repro!("fmmarks_gap_thanks_then_mark_red"), vec![
      row(&["Ann Able", "footnotemark[†]=", "thanks[*]=Ann note"]),
      row(&["Bob Baker"]),
      row(&["Carl Cole"]),
    ]),
    (
      repro!("fmmarks_mark_and_note_floating_between_names_red"),
      vec![
        row(&[
          "Ann Able",
          "footnotemark[†]=",
          "thanks[*]=Ann note body",
          "thanks[‡]=Floating body",
        ]),
        row(&["Bob Baker"]),
        row(&["Carl Cole"]),
      ],
    ),
    (
      repro!("fmmarks_mark_then_thanks_before_next_name_red"),
      vec![
        row(&["Ann Able", "footnotemark[†]=", "thanks[*]=Bob note"]),
        row(&["Bob Baker"]),
        row(&["Carl Cole"]),
      ],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(notes_in(&xml), expected, "{xml}");
  }
}

/// A mark between two names past spacing is the name's before (DG 2026-11-17, re-pinned from 2026-11-16's nearer-by-glue
/// rule), written once on it and nothing else moved with it: a mark in Ann's group and one in a group of its own
/// (`{\small\footnotemark[2]}`) are Ann's, Bob's `\textsc` kept; one beside other ink apart from both names (a
/// superscript, an ORCID) or past a fill is Ann's too, no loss counted (REGFIX and SHGNEXT left it undecided, counted;
/// SHGIVENc/d grew the next name's run back over the spacing — reviews revSHGIVENc/d). Distinct from
/// `a_mark_beside_spacing_is_the_nearer_names`: a group around the mark, other ink, a fill.
#[test]
fn a_mark_nearer_the_next_name_moves_alone() {
  let lost = "given their number (a note's, written already) on no name and no item; lost: †";
  let bob = |marks: &[&str]| {
    std::iter::once("Bob Baker")
      .chain(marks.iter().copied())
      .map(ToString::to_string)
      .collect::<Vec<_>>()
  };
  for (tex, (warnings, ann, bobs)) in [
    (
      repro!("ne2g_mark_in_group_with_previous_name"),
      (0, &["Ann Able", "footnotemark[†]="][..], bob(&[])),
    ),
    (
      repro!("ne2g_mark_in_its_own_group"),
      (0, &["Ann Able", "footnotemark[†]="][..], bob(&[])),
    ),
    (
      repro!("ne2g_mark_then_superscript_nearer_next"),
      (
        0,
        &["Ann Able", "footnotemark[a]=", "footnotemark[†]="][..],
        bob(&[]),
      ),
    ),
    (
      repro!("ne2g_mark_then_orcid_nearer_next"),
      (0, &["Ann Able", "footnotemark[†]="][..], bob(&[])),
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    assert!(warnings == 0 || log.contains(lost), "{log}");
    assert_eq!(
      notes_in(&xml),
      [row(ann), bobs, row(&["Carl Cole"])],
      "{xml}"
    );
    // (the superscript `a` Ann's, outside her name: PURE's mark-only note, OD #1055)
    let a_mark = r#"<note mark="a" role="footnotemark""#;
    assert_eq!(
      (xml.matches(a_mark).count(), xml.matches("<sup").count()),
      (usize::from(tex.contains("$^{a}$")), 0),
      "{xml}"
    );
    assert_eq!(
      xml.matches("orcid.org").count(),
      usize::from(tex.contains("orcidlink{")),
      "{xml}"
    );
    // (Bob's `\textsc` kept in his personname, in his own size)
    assert_eq!(
      xml
        .matches(r#"<personname><text font="smallcaps" xml:id="#)
        .count(),
      usize::from(tex.contains(r"\textsc{Bob Baker}")),
      "{xml}"
    );
  }
  // (a fill before the mark, a quad after it: Ann's, no width measured)
  let (log, xml) = run(repro!("ne2g_mark_past_hfill_undecided"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(notes_in(&xml), [
    row(&["Ann Able", "footnotemark[†]="]),
    row(&["Bob Baker"])
  ]);
}
