//! Red/green guards for batch 84a: an author line whose names horizontal space alone parts — a wide space (`\quad`,
//! `{\quad}`, `\hskip`, `\kern`), or a name's marks or notes and any space after them — is as many creators as pdflatex
//! sets names apart (OXIDIZED_DESIGN_DIVERGENCES #897; `space_parted_names`, `space_parts_names`). Each guard converts
//! its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` and asserts no Error or Warning and every
//! creator whole, against pdflatex. Witnesses 2406.07115, 2609.32353, 2310.15398 (mark then space); 2604.17501,
//! 2212.12575, 2411.14257 (wide space after a marked name); 2609.03563 (`{\quad}`); 1909.09562 (`\kern1em`);
//! 2312.06613, 2002.09131 (inside a wrapper); 0811.3255, 1311.0077 (aastex); 2406.12338 (names led by their notes);
//! 2212.12575's `Florian P.\ Busch` and 77n r2's ecai names stay whole.
use latexml::util::test::{convert_with, normalize_markup, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Every `<creator>` of `xml`, in document order, is EXACTLY the one `expected` lists at its place, and there are no
/// others.
#[track_caller]
fn assert_creators(xml: &str, expected: &[&str]) {
  let found: Vec<String> = xml
    .match_indices("<creator ")
    .filter_map(|(at, _)| xml_element(&xml[at..], "creator", &[]))
    .collect();
  let want: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(found, want, "{xml}");
}

/// Converts the repro and checks it converts clean.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// `Ann Able$^{1}$ \: Bob Baker$^{2}$\thanks{…} \ Cat Cole$^{1}$ \⏎Dan Dorn\textsuperscript{2}~Eve Ess$^{1,2}$ \, Fay
/// Fox$^{2}$`: six creators, each with the universities its marks name, the note Bob Baker's — the six names had been
/// one creator holding everything (silent; Perl the same).
#[test]
fn author_mark_then_space_separates_names() {
  let xml = clean(repro!("author_mark_then_space_separates_names.tex"));
  let univ = |u: &str| {
    format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ {u}</contact>")
  };
  let creator = |first: bool, name: &str, inner: &str| {
    format!(
      "<creator {}role=\"author\"><personname>{name}</personname>{inner}</creator>",
      if first {
        ""
      } else {
        "before=\"\u{2003}\u{2003}\" "
      }
    )
  };
  let expected = [
    creator(true, "Ann Able", &univ("A")),
    creator(
      false,
      "Bob Baker",
      &format!(
        "<note class=\"ltx_note_frontmatter ltx_thanks_contribution\" role=\"thanks\" xml:id=\"id1\">Equal \
         contribution.</note>{}",
        univ("B")
      ),
    ),
    creator(false, "Cat Cole", &univ("A")),
    creator(false, "Dan Dorn", &univ("B")),
    creator(false, "Eve Ess", &format!("{}{}", univ("A"), univ("B"))),
    creator(false, "Fay Fox", &univ("B")),
  ];
  assert_creators(
    &xml,
    &expected.iter().map(String::as_str).collect::<Vec<_>>(),
  );
}

/// `\textbf{Ann Able}$^{\star}$ \quad \textbf{Bob Baker}$^{\diamond}$ \quad Cat Cole \quad \textbf{Dan Dorn}\quad\quad
/// \textbf{Eve Ess}\textsuperscript{\rm 1}`: five creators, where the first four were one, flagged `merged_creators`
/// (the unmarked lines welded onto the name before them).
#[test]
fn author_wide_space_after_marked_name_separates() {
  let xml = clean(repro!("author_wide_space_after_marked_name_separates.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">\u{22c6}</sup></personname></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup \
     xml:id=\"id2\">\u{22c4}</sup></personname></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Dan Dorn</personname></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Eve Ess</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
  ]);
}

/// `Ann Able$^{1}${\quad} Bob Baker$^{1,2}${\quad} Cat Cole$^{3}$ \\ $^{1}$Univ A{\quad}$^{2}$Univ B{\quad}$^{3}$Univ
/// C`: three creators with their universities, where one creator held one affiliation "Univ A 2Univ B 3Univ C".
#[test]
fn author_braced_quad_separates_names() {
  let xml = clean(repro!("author_braced_quad_separates_names.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ A</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ B</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C</contact></creator>",
  ]);
}

/// llncs `Ann Able\inst{1} ⏎\kern1em ⏎Bob Baker\inst{2}`: two creators, each with its institute and e-mail.
#[test]
fn author_kern_separates_names() {
  let xml = clean(repro!("author_kern_separates_names.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ A</contact><contact name=\"E-mail\u{a0}\" role=\"email\"><text font=\"typewriter\" \
     xml:id=\"id1\">aa@x.edu</text></contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact><contact name=\"E-mail\u{a0}\" \
     role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">bb@y.edu</text></contact></creator>",
  ]);
}

/// `\normalsize{Ann Able\textsuperscript{1} \quad Bob Baker\textsuperscript{1,2} \hskip2em Cat
/// Cole\textsuperscript{2}}`: three creators with their universities.
#[test]
fn author_wide_space_inside_wrapper_separates() {
  let xml = clean(repro!("author_wide_space_inside_wrapper_separates.tex"));
  let univ = |u: &str, id: usize| {
    format!(
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"80%\" xml:id=\"id{id}\">Univ \
       {u}</text></contact>"
    )
  };
  let expected = [
    format!(
      "<creator role=\"author\"><personname>Ann Able</personname>{}</creator>",
      univ("A", 1)
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}{}</creator>",
      univ("A", 2),
      univ("B", 3)
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname>{}</creator>",
      univ("B", 4)
    ),
  ];
  assert_creators(
    &xml,
    &expected.iter().map(String::as_str).collect::<Vec<_>>(),
  );
}

/// emulateapj (aas_support, read cautiously) `Ann Able\altaffilmark{1} Bob B. Baker \altaffilmark{1} and Cat
/// Cole\altaffilmark{2}`: three creators, where "Ann Able Bob B. Baker" was one.
#[test]
fn author_aastex_mark_then_space_separates() {
  let xml = clean(repro!("author_aastex_mark_then_space_separates.tex"));
  let alt = |u: &str| {
    format!(
      "<contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Univ {u}</contact>"
    )
  };
  let expected = [
    format!(
      "<creator role=\"author\"><personname>Ann Able</personname>{}</creator>",
      alt("A")
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob B. Baker</personname>{}</creator>",
      alt("A")
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname>{}</creator>",
      alt("B")
    ),
  ];
  assert_creators(
    &xml,
    &expected.iter().map(String::as_str).collect::<Vec<_>>(),
  );
}

/// Negative control: `Florian P.\ Busch$^{1}$`, `J.\enskip Smith$^{1}$` stay whole names, and `Bob Baker$^{2}$ \quad
/// University of Somewhere` keeps the institution an affiliation, no creator.
#[test]
fn author_space_inside_a_name_stays() {
  let xml = clean(repro!("author_space_inside_a_name_stays.tex"));
  let univ =
    |u: &str| format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{u}</contact>");
  let expected = [
    format!(
      "<creator role=\"author\"><personname>Florian P. Busch</personname>{}</creator>",
      univ("Univ A")
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>J.\u{2002}Smith</personname>{}</creator>",
      univ("Univ A")
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}{}</creator>",
      univ("University of Somewhere"),
      univ("Univ B")
    ),
  ];
  assert_creators(
    &xml,
    &expected.iter().map(String::as_str).collect::<Vec<_>>(),
  );
}

/// Names led by their notes (2406.12338's `\thanks{…} {Carla Schenker,} ⏎\thanks{…} {Xiulin Wang,}`): three creators,
/// each with its own note, where one creator held the three names and notes, flagged `merged_creators`.
#[test]
fn author_prefix_notes_then_space_separate_names() {
  let xml = clean(repro!("author_prefix_notes_then_space_separate_names.tex"));
  let creator = |first: bool, name: &str, id: usize| {
    format!(
      "<creator {}role=\"author\"><personname>{name}</personname><note class=\"ltx_note_frontmatter \
       ltx_thanks_note\" role=\"thanks\" xml:id=\"id{id}\">{name} is with Univ {}.</note></creator>",
      if first {
        ""
      } else {
        "before=\"\u{2003}\u{2003}\" "
      },
      &name[..1]
    )
  };
  let expected = [
    creator(true, "Ann Able", 1),
    creator(false, "Bob Baker", 2),
    creator(false, "Cat Cole", 3),
  ];
  assert_creators(
    &xml,
    &expected.iter().map(String::as_str).collect::<Vec<_>>(),
  );
}

/// Names led by their marks, `$^{1}$Ann Able \: $^{2}$Bob Baker \: $^{1}$Cat Cole`: three creators, each with the
/// university its mark names, where one creator held both.
#[test]
fn author_prefix_marks_then_space_separate_names() {
  let xml = clean(repro!("author_prefix_marks_then_space_separate_names.tex"));
  let creator = |first: bool, name: &str, univ: &str| {
    format!(
      "<creator {}role=\"author\"><personname>{name}</personname><contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">Univ {univ}</contact></creator>",
      if first {
        ""
      } else {
        "before=\"\u{2003}\u{2003}\" "
      }
    )
  };
  let expected = [
    creator(true, "Ann Able", "A"),
    creator(false, "Bob Baker", "B"),
    creator(false, "Cat Cole", "A"),
  ];
  assert_creators(
    &xml,
    &expected.iter().map(String::as_str).collect::<Vec<_>>(),
  );
}

/// `Ann~{\sc Able}\rlap,${}^{1,}$\footnote{…}⏎Bob~{\sc Baker}${}^{2,}$\footnote{…}` (hep-ph0411288's ptptex block): the
/// comma `\rlap` sets parts the names as a plain comma, each name keeping its mark and note. Red twice: int111d gave
/// Ann's mark to Bob ("1,2"); 84a r1 moved the marks before the comma, `\rlap$` breaking the math (3 Errors).
#[test]
fn author_lapped_comma_before_marks() {
  let xml = clean(repro!("author_lapped_comma_before_marks.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann\u{a0}<text font=\"smallcaps\" xml:id=\"id1\">Able</text><sup \
     xml:id=\"id2\">1</sup></personname><contact name=\"Note:\u{a0}\" role=\"note\">E-mail: aa@x.edu</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob\u{a0}<text font=\"smallcaps\" \
     xml:id=\"id3\">Baker</text><sup xml:id=\"id4\">2</sup></personname><contact name=\"Note:\u{a0}\" \
     role=\"note\">E-mail: bb@y.edu</contact></creator>",
  ]);
}

/// `{Ann Able$^{1}$\thanks{…} ~ Bob Baker$^{2}$ ~ Cat Cole{$^{2}$} ~ Dan Dorn{$^{2}$}}` (2406.09417): a group holding
/// only marks is the name's marks, so the space after it parts the names — four creators. Red twice: int111d one
/// creator; 84a r2 "Cat Cole Dan Dorn" one creator, silent.
#[test]
fn author_grouped_marks_then_space_separate_names() {
  let xml = clean(repro!("author_grouped_marks_then_space_separate_names.tex"));
  let plain = |first: bool, name: &str, mark: &str, id: &str| {
    format!(
      "<creator {}role=\"author\"><personname>{name}<sup xml:id=\"{id}\">{mark}</sup></personname></creator>",
      if first {
        ""
      } else {
        "before=\"\u{2003}\u{2003}\" "
      }
    )
  };
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">1</sup></personname><note \
       class=\"ltx_note_frontmatter ltx_thanks_contribution\" role=\"thanks\" xml:id=\"id2\">Equal \
       contribution.</note></creator>",
    &plain(false, "Bob Baker", "2", "id3"),
    &plain(false, "Cat Cole", "2", "id4"),
    &plain(false, "Dan Dorn", "2", "id5"),
  ]);
}
