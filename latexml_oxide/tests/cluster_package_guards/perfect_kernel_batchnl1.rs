//! Guards for NL1, the names line of the one engine's reading ported from the old reader's procedure
//! (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; the rule table RT3, rows N1-N3, C2-C5, S1, A2): each fixture
//! converted with the emitter's switch off and at `LATEXML_FM_EMIT=2`, the creators (name and every contact, in order)
//! the same both ways. One guard per rule: a marked name (N1-N3), glued initials (S1), a collaboration remark (C2-C5),
//! a comma a command draws inside a word (A2).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

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

/// `tex` with `files` beside it converted under `emit`: the log and the XML.
fn run_with(tex: &str, files: &[(&str, &str)], emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    files,
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(Some(true));
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// `tex`'s creators off and at `=2`, each run clean (`warnings` each way).
fn both(tex: &str, warnings: (usize, usize)) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  both_with(tex, &[], warnings)
}

/// [`both`] with `files` beside `tex`.
fn both_with(
  tex: &str,
  files: &[(&str, &str)],
  warnings: (usize, usize),
) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  let (off_log, off) = if files.is_empty() {
    run(tex, 0)
  } else {
    run_with(tex, files, 0)
  };
  let (log, xml) = if files.is_empty() {
    run(tex, 2)
  } else {
    run_with(tex, files, 2)
  };
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, warnings.0),
    "{off_log}"
  );
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings.1),
    "{log}"
  );
  (creators(&off), creators(&xml))
}

/// Rows N1-N3: a names line's piece carrying its own mark is a name whatever its words — the generic split never reads
/// an institution word (`R. New$^2$`, 0904.4122; `S. Michelle Consiglio,$^{1}$`, 2103.02641; `Alexander New$^2$`,
/// 2311.16860), and a per-author class's line a surname after an initial (`Gary P. Centers,$^{1}$`, revtex,
/// 1901.10843). Red (MV1 `=2`): each the name before's affiliation. An unmarked piece keeps the reading's institution
/// (`John Smith, University of Toronto`, the main session's decision 2026-11-08), off a creator of its own.
#[test]
fn a_marked_name_with_an_institution_word_is_a_name() {
  let (off, two) = both(
    repro!("emit_a_marked_name_with_an_institution_word_is_a_name"),
    (0, 0),
  );
  let want = [
    vec!["A. M. Broomhall", "affiliation=Univ A"],
    vec!["S. Michelle Consiglio", "affiliation=Univ A"],
    vec!["R. New", "affiliation=Univ B"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let (off, two) = both(
    repro!("revtex_marked_list_surname_after_initial_is_a_name"),
    (0, 0),
  );
  let alpha = "affiliation=University of Alpha";
  let beta = "affiliation=University of Beta";
  assert_eq!(two, [
    vec!["Ann Able", alpha],
    vec!["Bob Baker", alpha],
    vec!["Gary P. Centers", alpha],
    vec!["Dan Dee", beta],
    vec!["Eve Eden 1,2", alpha, beta],
  ]);
  assert_eq!(off.len(), 5);
  let (off, two) = both(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^1$, University of \
     Toronto\\\\ $^1$Univ A}\n\\maketitle\nx\n\\end{document}\n",
    (0, 0),
  );
  assert_eq!(off, [vec!["Ann Able", "affiliation=Univ A"], vec![
    "University of Toronto"
  ],]);
  assert_eq!(two, [vec![
    "Ann Able",
    "affiliation=University of Toronto",
    "affiliation=Univ A"
  ]]);
}

/// Row S1 (a rule of the reading alone): initials after a comma join the name before only after a bare surname; a word
/// of glued initials and a surname is read through its initials (`unglued_initials`, as `name_shaped` reads it):
/// `P.A.Vshivtseva$^{a}$, E.T.\, Einiev` is two names (2409.13569). Red (MV1 `=2`): one creator, `merged_creators`.
#[test]
fn glued_initials_are_no_surname_before_initials() {
  let (off, two) = both(
    repro!("emit_glued_initials_are_no_surname_before_initials"),
    (0, 0),
  );
  let want = [
    vec!["M.O. Astashenkov", "affiliation=Univ A"],
    vec!["P.A.Vshivtseva", "affiliation=Univ A"],
    vec!["E.T. Einiev", "affiliation=Univ B"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
}

/// Rows C2-C5: a name's collaboration remark is the name's `collaboration` contact, no part of the name
/// (FIDELITY_CHARTER §4, OXIDIZED_DESIGN_DIVERGENCES #946), read whole where a note, a note reference, a box or a
/// command's own spacing parts it into pieces (C3, C4: `Kalanand Mishra\footnote{…} (for the \babar\ collaboration)`,
/// 0711.1544; elsarticle's `Ann Able (for the XYZ\fnref{c1} Collaboration)`, 1212.4278, 1408.0414), after the name's
/// notes when it names whom the author speaks for (C2: webofc's `for CAPP/IBS`, 1702.03664), and as a line of its own
/// under one name (C5: `Carolyn Brown$^1$\\ on behalf of the Bcool project team`, 1310.6507). Red (MV1 `=2`): each
/// in the personname, or "Collaboration)" the contact. The remark parted by a note reference and the name before it are
/// written from their text at `=2`, a Warning each (the name's events don't spell it; the remark's span holds the
/// reference); `\babar`'s letters likewise (its span prints a macro).
#[test]
fn a_collaboration_remark_is_the_names_contact() {
  let (off, two) = both(
    repro!("emit_a_collaboration_remark_parted_by_a_note_and_a_box_is_a_contact"),
    (0, 1),
  );
  assert_eq!(off, [vec![
    "Kalanand Mishra",
    "note=Email: k@x.edu",
    "collaboration=(for the B A B AR collaboration)",
    "affiliation=Univ A"
  ]]);
  assert_eq!(two, [vec![
    "Kalanand Mishra",
    "collaboration=(for the B A B A R collaboration)",
    "note=Email: k@x.edu",
    "affiliation=Univ A"
  ]]);
  let (off, two) = both(
    repro!("elsarticle_collaboration_remark_note_reference_is_the_names"),
    (0, 2),
  );
  let want = [vec![
    "Ann Able",
    "collaboration=(for the XYZ Collaboration)",
    "address=Univ X",
    "note=A list of members is at the end.",
  ]];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let (off, two) = both_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc_remark_after_the_notes_is_a_contact.tex"
    ),
    &[(
      "webofc.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc.cls"),
    )],
    (0, 0),
  );
  assert_eq!(off, [vec![
    "Ann Able",
    "affiliation=Univ A, Korea",
    "email=ann@a.edu",
    "collaboration=for CAPP/IBS"
  ]]);
  assert_eq!(two, [vec![
    "Ann Able",
    "email=ann@a.edu",
    "collaboration=for CAPP/IBS",
    "affiliation=Univ A, Korea"
  ]]);
  let (off, two) = both(
    repro!("emit_a_line_speaking_for_a_collaboration_is_a_contact"),
    (0, 0),
  );
  let want = [
    vec!["Ann Able", "affiliation=Univ A"],
    vec!["Bob Baker", "affiliation=Univ A"],
    vec![
      "Carolyn Brown",
      "collaboration=on behalf of the Bcool project team",
      "affiliation=Univ A",
    ],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
}

/// Row A2: the ink one command draws inside a word is the word's letters — combelow's `\cb{t}` stacks a `t` over a
/// comma in an alignment (combelow.sty:16), so its comma and row break part no names: `Bogdan Rai\cb{t}\u{a}`
/// (2507.13598) is one name, written from its tokens as off writes it. Red (MV1 `=2`): the line every author's
/// affiliation, two creators lost.
#[test]
fn a_comma_a_command_draws_inside_a_name_parts_no_names() {
  let (off, two) = both(
    repro!("emit_a_comma_a_command_draws_inside_a_name_parts_no_names"),
    (0, 0),
  );
  let want = [
    vec!["Ann Able", "affiliation=Univ A"],
    vec!["Bob Baker", "affiliation=Univ A"],
    vec!["Bogdan Rai t , ă", "affiliation=Univ B"],
    vec!["Sarah Adel Bargal", "affiliation=Univ A"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
}
