//! Guards for NE2 (`agent_reports/2026-11-09_NE2_report.md`): Errors `=2` raised that off doesn't, the
//! flip's no-harm gate. One guard per mechanism, each off and at `=2`: the footnote counter's steps of an author block's
//! notes run where they print (1507.05018, 2505.07115); an IJCAI sectioned block goes to its section split, its
//! `labelseq=author` affiliation the N-th author's (2202.05511); initials a gap joins to a name end no surname-first name
//! (cond-mat9905290); "representing" speaks for a collaboration (hep-ex9903035). A creator's labels written back as plain
//! characters (1910.05766) is the unit test `emitter::emit::tests::a_creators_labels_are_plain_characters`. NE2b (review
//! revNE2): a counter move no name carries writes the block whole, with its Warning (1507.05018's Ombao, A1); a note
//! before every name steps before the first name (A2). NE2c (review revNE2r2 Q2): a mark a name prints past its note stays
//! with it, the note's step before it (1610.06031); a `\footnotemark` no name and no item writes writes the block whole,
//! with its Warning (1911.13299). NE2d (user ruling 2026-11-09): only a dropped mark that steps the counter declines; one
//! given its number (`\samethanks[1]`, 1912.01737; 1811.08622) keeps the creators, its loss counted.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz3::in_a_directory};

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
#[test]
fn a_notes_footnote_step_runs_where_it_prints() {
  let (log, xml) = run(repro!("ne2_thanks_addtocounter_order"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker * * footnote *"])
  ]);
  let notes: Vec<&str> = xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
    .collect();
  assert_eq!(
    notes,
    [
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1""#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2""#,
      r#"<note mark="*" role="footnotemark" xml:id="footnote1""#,
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
    row(&["Yongmin Kim *", lab]),
    row(&["F. M. Munteanu ∗,†", lab, dept]),
    row(&["C. H. Perry ∗,†", lab, dept]),
    row(&["D. G. Rickel *", lab]),
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
/// block is written whole, Perl's whole-block creator, its tokens digested in order: Ann †, Carl *, every note kept, the
/// counted Warning. Off reads it per name, as pdflatex prints the marks. (A move a mark's box covers,
/// `\addtocounter{footnote}{-2}\footnotemark`, is carried: `a_notes_footnote_step_runs_where_it_prints`.)
#[test]
fn a_counter_move_no_name_carries_writes_the_block_whole() {
  let expected = vec![row(&[
    "Ann Able † † footnote † and Bob Baker and Carl Cole * * footnote *",
  ])];
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
  // (every note kept, each mark as pdflatex prints it: Ann's footnotemark †, Carl's *; the block written whole holds both
  // marks in its one creator, before the thanks notes)
  let (mark_ann, mark_carl) = (
    r#"<note mark="†" role="footnotemark" xml:id="footnote2""#,
    r#"<note mark="*" role="footnotemark" xml:id="footnote1""#,
  );
  let thanks = |n: u8| {
    format!(r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id{n}""#)
  };
  let mut expected = vec![mark_ann.to_string(), thanks(1), thanks(2), thanks(3)];
  expected.insert(1, mark_carl.to_string());
  assert_eq!(notes, expected, "{xml}");
}

/// A note before every name steps the counter before the first creator's name, where LaTeX reads it: `=2` stepped it
/// after Ann Able's personname, so her `\footnotemark` printed * where pdflatex prints † (review revNE2 A2). Distinct
/// from `a_notes_footnote_step_runs_where_it_prints`: the step before a name, not after one.
#[test]
fn a_note_before_every_name_steps_before_it() {
  let (log, xml) = run(repro!("ne2b_note_before_every_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able † † footnote †"]),
    row(&["Bob Baker"])
  ]);
}

/// The marks' notes of `xml` (a `\footnotemark`'s, an `\endnotemark`'s), each its opening tag, in document order.
fn footnotemarks(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
    .filter(|tag| tag.contains(" mark="))
    .collect()
}

/// A mark a name prints past its note stays with the name, in source order, the note's footnote step before it: `=2`'s
/// past-marks scan stopped at the note, so Ann Able's `\footnotemark` was dropped with no Warning and the next name's mark
/// printed one symbol short (Bob †, Carl ‡). Each shape off and at `=2` as pdflatex prints it: glued, past `\ `, `~` and
/// `\,`, a given `\footnotemark[1]`, past a `\footnote`, past a formula's `$^,$` (1610.06031's `G.
/// Citti\thanks{...}\footnotemark[1]`, review revNE2r2 Q2; 1703.08733's
/// `\protect\endnotemark[1]$^,$\protect\endnotemark[2]`). Distinct from `a_notes_footnote_step_runs_where_it_prints`:
/// the note's step inside the name's tokens, between its text and its mark.
#[test]
fn a_mark_after_its_names_note_stays_with_it() {
  let mark = |m: &str, n: &str| format!(r#"<note mark="{m}" role="footnotemark" xml:id="{n}""#);
  for (tex, expected, marks) in [
    (
      repro!("ne2c_mark_after_note"),
      vec![
        row(&["Ann Able † † footnote †"]),
        row(&["Bob Baker ‡ ‡ footnote ‡"]),
      ],
      vec![mark("†", "footnote2"), mark("‡", "footnote3")],
    ),
    (
      repro!("ne2c_mark_after_note_spaced"),
      vec![
        row(&["Ann Able † † footnote †"]),
        row(&["Bob Baker"]),
        row(&["Carl Cole § § footnote §"]),
      ],
      vec![mark("†", "footnote2"), mark("§", "footnote4")],
    ),
    (
      repro!("ne2c_mark_after_note_numbered"),
      vec![
        row(&["Ann Able * * footnote *"]),
        row(&["Bob Baker * * footnote *"]),
        row(&["Carl Cole"]),
      ],
      vec![mark("*", "footnotex1"), mark("*", "footnotex2")],
    ),
    (
      repro!("ne2c_mark_after_footnote_glued"),
      vec![
        row(&["Ann Able † † footnote †"]),
        row(&["Bob Baker * * footnote *", "note=Univ Y."]),
        row(&["Carl Cole § § footnote §"]),
      ],
      vec![
        mark("†", "footnote2"),
        mark("*", "footnotex1"),
        mark("§", "footnote4"),
      ],
    ),
    (
      repro!("ne2c_mark_after_math_comma"),
      vec![
        row(&["Ann Able 1 1 endnote 1"]),
        row(&["Bob Baker 1 1 endnote 1 , 2 2 endnote 2"]),
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
}

/// A `\footnotemark` no name of the `=2` reading writes and no item holds (on the line a `\\` opens under Ann Able's name,
/// 1911.13299's `Vivek Ramanujan \thanks{equal contribution} \\ \footnotemark[2]`) was dropped with no Warning, Bob
/// Baker's mark one symbol short. The block is written whole, Perl's whole-block creator, its tokens digested in order:
/// Ann †, Bob ‡, the note kept, the counted Warning (review revNE2r2 Q2: never a silent loss). Off reads Ann's † as the
/// mark of an affiliation line of hers. Distinct from `a_counter_move_no_name_carries_writes_the_block_whole`: a mark, not
/// a counter move, that the reading has no place for.
#[test]
fn a_mark_on_no_name_writes_the_block_whole() {
  let expected = vec![row(&[
    "Ann Able † † footnote † and Bob Baker ‡ ‡ footnote ‡",
  ])];
  let warnings = 1;
  let (log, xml) = run(repro!("ne2c_mark_on_no_name_writes_the_block_whole"));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  assert_eq!(creators(&xml), expected);
  assert_eq!(
    footnotemarks(&xml),
    [
      r#"<note mark="†" role="footnotemark" xml:id="footnote2""#,
      r#"<note mark="‡" role="footnotemark" xml:id="footnote3""#,
    ],
    "{xml}"
  );
  assert_eq!(
    xml
      .matches(r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks""#)
      .count(),
    1,
    "{xml}"
  );
}

/// A `\footnotemark` given its number that no name and no item writes (`D. R. G. Schleicher,$^{1}$\samethanks[1]`,
/// 1912.01737's document `\samethanks`; `Cheng Xu\textsuperscript{1},\footnotemark[1]`, 1811.08622) re-references a note
/// written already and steps no counter (latex.ltx's `\@xfootnotemark`), so no later mark moves: NE2c's decline wrote the
/// block whole and threw away a right reading. Now the names are written as read, the marks' loss counted by 1 Warning
/// naming them (user ruling 2026-11-09). A block that also drops a bare `\footnotemark` (which steps the counter: the next
/// mark would print one symbol short; 1111.1979) is still written whole. Distinct from
/// `a_mark_on_no_name_writes_the_block_whole`: the dropped mark's step decides, the same check's other branch.
#[test]
fn a_given_mark_on_no_name_keeps_the_creators() {
  let names = |carl: &str| {
    vec![
      row(&["Ann Able", "affiliation=Univ A"]),
      row(&["Bob Baker", "affiliation=Univ A"]),
      row(&[carl, "affiliation=Univ B"]),
      row(&["Dan Dory", "affiliation=Univ B"]),
    ]
  };
  let expected = names("Carl Cole");
  let marks: Vec<String> = vec![];
  let (log, xml) = run(repro!("ne2d_given_mark_dropped_keeps_the_creators"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("writes 2 footnote mark(s) of its author block given their number (a note's, written already) on no name and no item; lost: *, *"),
    "{log}"
  );

  assert_eq!(creators(&xml), expected);
  assert_eq!(footnotemarks(&xml), marks, "{xml}");
  assert_eq!(
    xml
      .matches(r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks""#)
      .count(),
    1,
    "{xml}"
  );

  // (a bare `\footnotemark` dropped too: the block written whole, every mark kept, its Warning)
  let expected = vec![row(&[
    "Ann Able, 1 Bob Baker, 1 * * footnote * Carl Cole, 2 † † footnote † Dan Dory 2 ‡ ‡ footnote ‡ 1 Univ A 2 Univ B",
  ])];
  let (log, xml) = run(repro!("ne2d_given_and_fresh_mark_dropped"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("on no name and no item; the block is written whole"),
    "{log}"
  );

  assert_eq!(creators(&xml), expected);
  assert_eq!(
    footnotemarks(&xml),
    [
      r#"<note mark="*" role="footnotemark" xml:id="footnotex1""#,
      r#"<note mark="†" role="footnotemark" xml:id="footnote2""#,
      r#"<note mark="‡" role="footnotemark" xml:id="footnote3""#,
    ],
    "{xml}"
  );
}
