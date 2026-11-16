//! Guards for NE2 (`agent_reports/2026-11-09_NE2_report.md`): Errors `=2` raised that off doesn't, the
//! flip's no-harm gate. One guard per mechanism, each off and at `=2`: the footnote counter's steps of an author block's
//! notes run where they print (1507.05018, 2505.07115); an IJCAI sectioned block goes to its section split, its
//! `labelseq=author` affiliation the N-th author's (2202.05511); initials a gap joins to a name end no surname-first name
//! (cond-mat9905290); "representing" speaks for a collaboration (hep-ex9903035). A creator's labels written back as plain
//! characters (1910.05766) is the unit test `emitter::emit::tests::a_creators_labels_are_plain_characters`. NE2b (review
//! revNE2): a counter move no name carries writes the block whole, with its Warning (1507.05018's Ombao, A1); a note
//! before every name steps before the first name (A2). NE2c (review revNE2r2 Q2): a mark a name prints past its note stays
//! with it, the note's step before it (1610.06031), and so does one past spacing (1911.13299's `\ \footnotemark[2]`; DG
//! ruling 2026-11-16); a `\footnotemark` no name and no item writes (on the line a `\\` opens) writes the block whole,
//! with its Warning. NE2d (user ruling 2026-11-09): only a dropped mark that steps the counter declines; one given its
//! number (`\samethanks[1]`, 1912.01737; 1811.08622) keeps the creators, its loss counted. NE2f (DG ruling 2026-11-16):
//! a mark with spacing on both sides between two names is the nearer name's by the gaps' TeX widths, the next one's on
//! a tie (review revSHGIVENb; 2412.20504).

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
/// FN1 (OD #1001; the user's ruling 2026-11-11): the note the name held is the creator's, right after
/// `</personname>`, so the name reads without its tags; the notes and marks are asserted kept below; pin updated (the
/// note's move only).
#[test]
fn a_notes_footnote_step_runs_where_it_prints() {
  let (log, xml) = run(repro!("ne2_thanks_addtocounter_order"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
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
/// block is written as Perl's `\lx@add@authors` splits it at its `\and`s (CR7, OD #988: it was one whole-block creator,
/// Perl's tabular branch), its pieces digested in order: Ann †, Carl *, every note kept, the counted Warning. Off reads it per name, as pdflatex prints the marks. (A move a mark's box covers,
/// `\addtocounter{footnote}{-2}\footnotemark`, is carried: `a_notes_footnote_step_runs_where_it_prints`.)
/// FN1 (OD #1001; the user's ruling 2026-11-11): the note the name held is the creator's, right after
/// `</personname>`, so the name reads without its tags; the notes and marks are asserted kept below; pin updated (the
/// note's move only).
#[test]
fn a_counter_move_no_name_carries_writes_the_block_as_perl_splits_it() {
  let expected = vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Carl Cole"])];
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
    r#"<note mark="†" role="footnotemark" xml:id="footnote2""#,
    r#"<note mark="*" role="footnotemark" xml:id="footnote1""#,
  );
  let thanks = |n: u8| {
    format!(r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id{n}""#)
  };
  let expected = vec![
    mark_ann.to_string(),
    thanks(1),
    thanks(2),
    thanks(3),
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
  assert_eq!(creators(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
}

/// The marks' notes of `xml` (a `\footnotemark`'s, an `\endnotemark`'s), each its opening tag, in document order.
fn footnotemarks(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
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
  for (tex, expected, marks) in [
    (
      repro!("ne2c_mark_after_note"),
      vec![row(&["Ann Able"]), row(&["Bob Baker"])],
      vec![mark("†", "footnote2"), mark("‡", "footnote3")],
    ),
    (
      repro!("ne2c_mark_after_note_spaced"),
      vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Carl Cole"])],
      vec![mark("†", "footnote2"), mark("§", "footnote4")],
    ),
    (
      repro!("ne2c_mark_after_note_numbered"),
      vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Carl Cole"])],
      vec![mark("*", "footnotex1"), mark("*", "footnotex2")],
    ),
    (
      repro!("ne2c_mark_after_footnote_glued"),
      vec![
        row(&["Ann Able"]),
        row(&["Bob Baker", "note=Univ Y."]),
        row(&["Carl Cole"]),
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
        row(&["Ann Able"]),
        // (the endnote marks beside the name too; the `$^,$` printed between them is left at the name's end)
        row(&["Bob Baker ,"]),
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
  // (past spacing, each creator its own marks: Ann † bare, after her note's step; Bob * and Carl † given; Dan ‡ bare)
  let (log, xml) = run(repro!("ne2e_mark_past_spacing_is_its_names"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker"]),
    row(&["Carl Cole"]),
    row(&["Dan Dory"]),
  ]);
  assert_eq!(
    marks_by_creator(&xml),
    [
      vec![mark("†", "footnote2")],
      vec![mark("*", "footnotex1")],
      vec![mark("†", "footnotex2")],
      vec![mark("‡", "footnote3")],
    ],
    "{xml}"
  );
}

/// A `\footnotemark` no name of the `=2` reading writes and no item holds (on the line a `\\` opens under Ann Able's name;
/// 1911.13299, first read as this shape, prints `\ \footnotemark[2]`, its name's: SHGIVEN) was dropped with no Warning, Bob
/// Baker's mark one symbol short. The block is written whole, Perl's whole-block creator, its tokens digested in order:
/// Ann †, Bob ‡, the note kept, the counted Warning (review revNE2r2 Q2: never a silent loss). Off reads Ann's † as the
/// mark of an affiliation line of hers. Distinct from `a_counter_move_no_name_carries_writes_the_block_as_perl_splits_it`: a mark, not
/// a counter move, that the reading has no place for.
/// FN1 (OD #1001; the user's ruling 2026-11-11, which accepts this shape): a block written whole holds its notes
/// after the joint name, each with its mark, so the name reads without their tags; pin updated (the notes' move only).
#[test]
fn a_mark_on_no_name_writes_the_block_whole() {
  let expected = vec![row(&["Ann Able and Bob Baker"])];
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
/// FN1 (OD #1001; the user's ruling 2026-11-11, which accepts this shape): a block written whole holds its notes
/// after the joint name, each with its mark, so the name reads without their tags; pin updated (the notes' move only).
/// SHGIVEN (DG ruling 2026-11-16, 4, refining the 2026-11-09 one): a given mark only spacing parts from its name is that
/// name's, as pdflatex prints it and Perl keeps it — 1911.13299's `Vivek Ramanujan \thanks{…} \ \footnotemark[2]`, its
/// four marks lost with a Warning ("lost: †, ‡, ‡, ‡"), now each on its name with no Warning (asserted first); a given
/// mark past a name's comma (the rows after) is still no name's, its loss counted.
#[test]
fn a_given_mark_on_no_name_keeps_the_creators() {
  let (log, xml) = run(repro!(
    "author_thanks_shared_by_footnotemark_marks_kept_red"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker"]),
    row(&["Cat Cole"]),
    row(&["Dan Dorn"]),
  ]);
  // (each creator's marks, in order: Ann †, Bob * ‡, Dan ‡ — pdflatex's "Ann Able ∗ †  Bob Baker ∗ ‡  Cat Cole † ‡  Dan
  // Dorn ‡", whose ∗ of Ann and † ‡ of Cat are their `\thanks` notes')
  let mark =
    |m: &str, n: &str| format!(r#"<note mark="{m}" role="footnotemark" xml:id="footnotex{n}""#);
  assert_eq!(
    marks_by_creator(&xml),
    [
      vec![mark("†", "1")],
      vec![mark("*", "2"), mark("‡", "3")],
      vec![],
      vec![mark("‡", "4")],
    ],
    "{xml}"
  );
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
    "Ann Able, 1 Bob Baker, 1 Carl Cole, 2 Dan Dory 2 1 Univ A 2 Univ B",
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

/// The carry past spacing (`a_mark_after_its_names_note_stays_with_it`'s ne2e row, DG ruling 2026-11-16, 4) takes a
/// `\footnotemark` back onto the name before it only when a separator, a `\\` or the block's end follows the mark's run
/// of marks and notes, or a spacing run wider than the one before it (`a_mark_between_two_names_is_the_nearer_ones`): a
/// mark a name follows glued is that name's prefix (`Ann Able \quad \footnotemark[2]Bob Baker`,
/// pdflatex's "†Bob Baker"; review revSHGIVEN 1), so each keeps REGFIX's output and its counts, never a mark invented
/// on the previous name. Nor does it take a mark inside a formula, written with the formula whole, its `\thanks` twice:
/// 2511.12633's `Xunzhi Xiang$^{1}\thanks{…}\,\,\footnotemark[2]$` stays lost and counted. Distinct from the ne2e row:
/// the carry's two exclusions. REGFIX's prefix Warnings ("lost: †" though Bob carries †) over-count: pinned by count.
#[test]
fn a_mark_a_name_follows_past_spacing_is_not_carried_back() {
  let mark = |m: &str, n: &str| format!(r#"<note mark="{m}" role="footnotemark" xml:id="{n}""#);
  let thanks = r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks""#;
  for (tex, (warnings, says), expected, marks, notes) in [
    (
      repro!("ne2e_prefix_mark_past_spacing_before_and"),
      (
        1,
        &["given their number (a note's, written already) on no name and no item; lost: †"][..],
      ),
      vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Carl Cole"])],
      vec![vec![], vec![mark("†", "footnotex1")], vec![]],
      0,
    ),
    (
      repro!("ne2e_prefix_marks_past_quad"),
      (
        2,
        &[
          "lost: †, *",
          "reaches no contact carrying its label, or a labelled contact no name's mark reaches (*)",
        ][..],
      ),
      vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Carl Cole"])],
      vec![
        vec![mark("*", "footnotex1")],
        vec![mark("†", "footnotex2")],
        vec![mark("*", "footnotex3")],
      ],
      0,
    ),
    (
      // (no place for Bob's prefix: the block written whole, `mark_dropped`'s Warning)
      repro!("ne2e_prefix_mark_past_hfill"),
      (
        1,
        &["on no name and no item; the block is written whole"][..],
      ),
      vec![row(&["Ann Able Bob Baker"])],
      vec![vec![mark("†", "footnote2")]],
      1,
    ),
    (
      repro!("ne2e_mark_in_formula_past_thin_spaces"),
      (1, &["on no name and no item; lost: ∗"][..]),
      vec![row(&["Ann Able 1"]), row(&["Bob Baker"])],
      vec![vec![], vec![]],
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
    assert_eq!(xml.matches(thanks).count(), notes, "{xml}");
  }
}

/// Each creator's count of `\thanks` notes, in document order.
fn thanks_by_creator(xml: &str) -> Vec<usize> {
  xml
    .split("<creator ")
    .skip(1)
    .map(|c| {
      c[..c.find("</creator>").expect("a creator's end")]
        .matches(r#"role="thanks""#)
        .count()
    })
    .collect()
}

/// A `\footnotemark` with spacing on both sides between two names is the nearer name's, by the TeX widths of the two
/// gaps as pdflatex prints them; on a tie, the next name's prefix (DG ruling 2026-11-16). The carry past spacing took it
/// back onto the name before whenever a Gap followed it (review revSHGIVENb, `Ann Able \qquad \footnotemark[2] \ Bob
/// Baker`: pdflatex's 27pt | 4pt, Bob's), and REGFIX lost it with a Warning or wrote the block whole. Rows: the Gap after
/// the narrower (` \ `, `~~`, `\quad`; a bare mark and a `\thanks` together; a bare mark past a note, its counter step
/// in order), the Gap after the wider (2412.20504's `\footnotemark[1] \ \footnotemark[2] \quad Qingyi Si`) and the tie.
/// Distinct from `a_mark_a_name_follows_past_spacing_is_not_carried_back`: a Gap on both sides of the mark, which grows
/// the next name's run over the gap after it.
#[test]
fn a_mark_between_two_names_is_the_nearer_ones() {
  let mark = |m: &str, n: &str| format!(r#"<note mark="{m}" role="footnotemark" xml:id="{n}""#);
  let three = || vec![row(&["Ann Able"]), row(&["Bob Baker"]), row(&["Carl Cole"])];
  let bobs = || vec![vec![], vec![mark("†", "footnotex1")], vec![]];
  for (tex, expected, marks, thanks) in [
    (
      repro!("ne2f_mark_nearer_next_past_qquad"),
      three(),
      bobs(),
      vec![0, 0, 0],
    ),
    (
      repro!("ne2f_mark_nearer_next_past_ties"),
      three(),
      bobs(),
      vec![0, 0, 0],
    ),
    (
      repro!("ne2f_mark_nearer_next_past_quads"),
      three(),
      bobs(),
      vec![0, 0, 0],
    ),
    (
      repro!("ne2f_mark_and_note_nearer_next"),
      three(),
      vec![vec![], vec![mark("†", "footnote2")], vec![]],
      vec![1, 1, 0],
    ),
    (
      repro!("ne2f_bare_mark_nearer_next_past_quads"),
      vec![row(&["Ann Able"]), row(&["Bob Baker"])],
      vec![vec![], vec![mark("†", "footnote2"), mark("‡", "footnote3")]],
      vec![1, 0],
    ),
    (
      repro!("ne2f_marks_wider_after_and_tied"),
      three(),
      vec![
        vec![mark("*", "footnotex1"), mark("†", "footnotex2")],
        vec![],
        vec![mark("‡", "footnotex3")],
      ],
      vec![0, 0, 0],
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), expected);
    assert_eq!(marks_by_creator(&xml), marks, "{xml}");
    assert_eq!(thanks_by_creator(&xml), thanks, "{xml}");
  }
}
