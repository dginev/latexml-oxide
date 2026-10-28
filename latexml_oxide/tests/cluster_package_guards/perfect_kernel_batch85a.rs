//! Red/green guards for batch 85a: an unmarked `\author` block written as blocks of lines — a name line, its address
//! lines, then the next name line — is one creator per name line, each with its own lines (`opens_next_author`,
//! `name_groups`; OXIDIZED_DESIGN_DIVERGENCES #900). The next name line had been an affiliation contact of the first
//! author (silent). Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` and
//! asserts no Error or Warning and every creator's name and contacts (role and text) in order, against pdflatex.
//! Witnesses 1702.01413, 0801.1297, 1608.08479, 2201.06821, 1509.03383 (a spacer line before the next name),
//! gr-qc9906006 (a positive `\\[12pt]`, and an initial in a name over an institution), 1908.10448 (an institution under
//! a name with an initial), 1709.07791 (the names' styling), 2408.11742 (a JMLR `\name … \email …` line); 1702.07694
//! (a stacked run of names keeps its lines); 1701.05964, 2004.03971, 2503.11250, 1504.05438 (the JASA template's
//! `\hspace{.2cm}` before the first line break). Round 2 answers the 85a r1 review (findings 1-9), round 3 the r2
//! review's (findings 1-8). One guard per distinct path the rule relies on; the reviewers' other probe shapes are catalog
//! repros without a guard (author_block_{institution_split_across_lines, places_and_roles_under_an_institution,
//! next_name_after_a_gap_under_a_place, linebreak_length_gaps, marked_block_gaps, rows_of_names_share_their_lines}),
//! and the shapes it does not handle RED ones (article_plain_name_line_between_affiliations,
//! author_block_{stacked_names_each_a_creator, eponym_or_place_over_institution,
//! next_name_under_a_place_or_whole_institution, row_of_names_with_no_signal, uniform_gaps_hide_a_wider_gap,
//! name_line_with_trailing_text}).
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Converts the repro, checks it converts clean, and compares its creators.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = creators(&xml);
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(found, want, "{xml}");
}

/// A line that prints nothing — `\vspace{0.1cm}\\` (1702.01413), an empty line `\\ \\` (0801.1297, 1608.08479,
/// 2201.06821), `$ $\\` (1509.03383) — or a positive `\\[12pt]`, `\\*[8pt]` (the same gap; the 85a r1 review's finding
/// 6) before a name line: the next author, even over a line naming no institution (`IFLP-CCT-Conicet`, `Boston, MA`).
/// Red (int114a): one creator holding every line; 85a r1: Eve Eads and Fay Fox among Dan Dorn's lines.
#[test]
fn next_name_after_a_spacer_line() {
  assert_creators(repro!("author_block_next_name_after_a_spacer_line.tex"), &[
    &[
      "Ann Able",
      "affiliation=Mathematics Department",
      "affiliation=Aston University",
      "affiliation=B3 7ET, Birmingham, UK",
    ],
    &[
      "B. Baker",
      "affiliation=IFLP-CCT-Conicet",
      "affiliation=CC 727, 1900 La Plata, Argentina",
    ],
    &[
      "Cat Cole",
      "affiliation=School of Mathematics and Statistics,",
      "affiliation=The University of Melbourne",
    ],
    &["Dan Dorn", "affiliation=LPTMS, Orsay"],
    &["Eve Eads", "affiliation=Boston, MA"],
    &["Fay Fox", "affiliation=Cambridge, MA"],
  ]);
}

/// With no spacer line, a name line carrying a person's cue (an initial: `Bob B. Baker`, `Cat C. Cole`, 1908.10448's
/// `James M. Robins`) over an institution it does not go on with; the `\hspace{.2cm}` before the first line break parts
/// no names from the institution under them. Red (int114a): Ann Able and three creators named by the institution's
/// parts, each holding every later line. A cue-less name there (1908.10448's `Zach Shahn \\ IBM Research,`) stays a
/// line: the RED repro `article_plain_name_line_between_affiliations.tex`.
#[test]
fn next_name_under_its_institution() {
  assert_creators(
    repro!("author_block_next_name_under_its_institution.tex"),
    &[
      &[
        "Ann Able",
        "affiliation=Institute of Natural Sciences, School of Mathematical Sciences, MOE-LSC",
        "affiliation=and SJTU-Yale Center for Biostatistics and Data Science,",
        "affiliation=Shanghai Jiao Tong University,",
      ],
      &["Bob B. Baker", "affiliation=IBM Research,"],
      &[
        "Cat C. Cole",
        "affiliation=Department of Biostatistics and Epidemiology, Harvard University,",
      ],
      &[
        "Dan Dorn",
        "affiliation=Department of Economics, Universidad Torcuato Di Tella and CONICET",
      ],
    ],
  );
}

/// Name lines between italic affiliation lines (1709.07791): a name over a line styled as the block's affiliations,
/// even one whose words read as a name (`{\it Hanson Robotics}`) or whose markup is mistyped (`I{\it ONS …}`). Red
/// (int114a): Cat Cole and Dan Dorn among Bob Baker's affiliation contacts.
#[test]
fn next_name_styled_as_the_names() {
  assert_creators(repro!("author_block_next_name_styled_as_the_names.tex"), &[
    &[
      "Ann Able",
      "affiliation=Hanson Robotics &amp; OpenCog Foundation",
    ],
    &["Bob Baker", "affiliation=Institute of Noetic Sciences"],
    // (the mistyped `I{\it ONS …}`: the "I" outside the italic group)
    &["Cat Cole", "affiliation=I ONS &amp; OpenCog Foundation"],
    &["Dan Dorn", "affiliation=Hanson Robotics"],
    &["Eve Eads", "affiliation=Hong Kong Polytechnic University"],
  ]);
}

/// JMLR-style `\name Bob Baker \email bob@usc.edu` under `\addr …` (2408.11742, its tmlr.sty's declarations): the name
/// styled as the names, its address aside. Red (int114a): "Bob Baker bob@usc.edu" an affiliation contact of Ann Able.
#[test]
fn name_and_address_lines_styled_alike() {
  assert_creators(
    repro!("author_block_name_and_address_lines_styled_alike.tex"),
    &[
      &[
        "Ann Able",
        "email=ann@usc.edu",
        "affiliation=University of Southern California",
      ],
      &[
        "Bob Baker",
        "email=bob@usc.edu",
        "affiliation=University of Southern California",
      ],
    ],
  );
}

/// A line opening the next author that holds a row of names parted by a run of spacing (`\\ \\ Bob Baker~~~~Cat Cole`,
/// `\vspace{4pt} \\ Dan Dorn\;\;Eve Eads`; `{\bf Gil Gray}~~~{\bf Hal Hunt}` between italic lines) is parted as a
/// block's first names line is: a creator per name, each with the line under the row (the 85a r1 review's finding 5).
/// Red (int114a): the rows among Ann Able's lines, "Gil Gray Hal Hunt" one creator; 85a r1: one fused creator per row.
#[test]
fn row_of_names_opens_the_next_author() {
  assert_creators(
    repro!("author_block_row_of_names_opens_the_next_author.tex"),
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ B"],
      &["Cat Cole", "affiliation=Univ B"],
      &["Dan Dorn", "affiliation=Univ C"],
      &["Eve Eads", "affiliation=Univ C"],
      &["Fay Fox", "affiliation=Univ F"],
      &["Gil Gray", "affiliation=Univ G"],
      &["Hal Hunt", "affiliation=Univ G"],
    ],
  );
}

/// The JASA template's `\hspace{.2cm}` before the first line break: the names line and the lines under it are one
/// group's, parted by `name_groups` — an "and" line opens the next author and is dropped (1701.05964, 2004.03971,
/// 1504.05438; v1, v3), a `\\[5pt]` or `\\ \\` gap opens one (2503.11250; u1, u3), and the same without the `\hspace` (v2).
/// Red (int114a): creators named by the institution's comma parts; 85a r2: Nancy Flournoy, Han Lin Shang and
/// 2503.11250's later authors lines of the first, an "and" contact (the r2 review's finding 1).
#[test]
fn lines_after_spaced_names() {
  assert_creators(repro!("author_block_lines_after_spaced_names.tex"), &[
    &[
      "Ann Able",
      "affiliation=Children’s Core for Biomedical Statistics, Seattle Children’s Research Institute",
    ],
    &[
      "Bob Baker",
      "affiliation=Department of Statistics, University of Missouri",
    ],
    &[
      "Cat Cole",
      "affiliation=Department of Mathematics and Statistics",
      "affiliation=University of Cyprus",
    ],
    &[
      "Dan Lin Dorn",
      "affiliation=Department of Actuarial Studies and Business Analytics",
      "affiliation=Macquarie University",
    ],
    &[
      "Eve Eads",
      "affiliation=Institute of Mathematical Statistics and Actuarial Science, University of Bern, Switzerland",
    ],
    &[
      "Fay Fox",
      "affiliation=DTIS, ONERA, Paris-Saclay University, 91120, Palaiseau, France",
    ],
    &[
      "Gil Gray",
      "affiliation=Institute of Mathematical Statistics and Actuarial Science, University of Bern, Switzerland",
    ],
    &[
      "Hal Hunt",
      "affiliation=Institute of Mathematical Statistics and Actuarial Science, University of Bern, Switzerland",
    ],
    &[
      "Ida Ives",
      "affiliation=Institute of Statistics, University of Bern",
    ],
    &[
      "Jan Jones",
      "affiliation=DTIS, ONERA, Paris-Saclay University",
    ],
    &[
      "Kim Kerr",
      "affiliation=Department of Statistics, University of X",
    ],
    &[
      "Lee B. Lowe",
      "affiliation=Department of Statistics, University of Y",
    ],
    &[
      "Mia Moss",
      "affiliation=Department of Statistics, University of X",
    ],
    &[
      "Ned Nash",
      "affiliation=Department of Statistics, University of Y",
    ],
  ]);
}

/// The negatives, one guard for every path that keeps a name-shaped line a line: an institution whose words read as a
/// name (`Hanson Robotics`) over a name-shaped line, `Hong Kong` over the university it goes on with, a role (`Chief
/// Scientist`) over an institution, a closing `Costa Rica`, a name under the names line (`Gil Gray`, the Perl reading,
/// names under names deferred); a line going on with the one above (connectives: 1110.3939, 2510.15190, 1404.5443, the
/// 85a r1 review's r10; a building, math-ph9903017); a head noun going on with the line above or a cue-less head over an
/// institution (`Johns Hopkins \\ University, …`, `Los Alamos`, `San Diego`, `Max Planck`: the r1 review's finding 2); a
/// cue-less place or role over an institution (`Santa Barbara`, `Ann Arbor`, `Associate Dean`, `Vice Provost`,
/// `Cognitive Neuroscience`: findings 3-4); a common-noun phrase after a gap (`Technical Report`, `Lecture Notes`: finding
/// 7); a closing place under a style group, after a gap, or under a group every line of which a gap opens (the r2
/// review's findings 2 and 3: t1, t5, t6, t22, t23; a gap before every line from the second under the names line on,
/// or before every line down to the place: the r3 review's finding 2, g11, g12). `Gil Gray` under the names line
/// `Fay Fox` is asserted only where today's reading (her line, Perl's) and the right one (a creator sharing her lines:
/// the RED author_block_stacked_names_each_a_creator) agree: Fay Fox keeps "Department of Physics" and "Costa Rica",
/// and no creator holds them alone. Red (int114a): a creator "Computer Engineering" (#52's "and"-led opening); 85a r1:
/// twelve creators named by those lines; 85a r2: creators "Costa Rica", "Hong Kong"; 85a r3: "Santa Barbara".
#[test]
fn lines_that_read_as_names_stay_lines() {
  let (log, xml) = convert_with(
    repro!("author_block_lines_that_read_as_names_stay_lines.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut found = creators(&xml);
  let fay_lines = [
    "affiliation=Department of Physics",
    "affiliation=Costa Rica",
  ];
  // (Gil Gray, a creator only in the right reading, then sharing Fay Fox's lines)
  if let Some(at) = found.iter().position(|c| c[0] == "Gil Gray") {
    assert_eq!(found[at][1..], fay_lines, "{xml}");
    found.remove(at);
  }
  for creator in &mut found {
    if creator[0] == "Fay Fox" {
      creator.retain(|contact| contact != "affiliation=Gil Gray");
    }
  }
  let expected: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Hanson Robotics",
      "affiliation=Hong Kong",
      "affiliation=The Hong Kong University of Science and Technology",
      "affiliation=Chief Scientist",
      "affiliation=Google Research",
    ],
    &[
      "Fay Fox",
      "affiliation=Department of Physics",
      "affiliation=Costa Rica",
    ],
    &[
      "Hal Hunt",
      "affiliation=School of Physical and",
      "affiliation=Mathematical Sciences",
      "affiliation=Nanyang Technological University",
      "affiliation=Department of Electrical",
      "affiliation=and Computer Engineering",
      "affiliation=Western University",
      "affiliation=Department of Biomedical Engineering and",
      "affiliation=Computational Science",
      "affiliation=Aalto University",
      "affiliation=Institute of Theoretical and",
      "affiliation=Applied Mechanics",
      "affiliation=Polish Academy of Sciences",
      "affiliation=James Clerk Maxwell Building",
      "affiliation=University of Edinburgh",
    ],
    &[
      "Ida Ives",
      "affiliation=Department of Physics and Astronomy",
      "affiliation=Johns Hopkins",
      "affiliation=University, Baltimore, MD 21218",
      "affiliation=Theoretical Division",
      "affiliation=Los Alamos",
      "affiliation=National Laboratory, NM 87545",
      "affiliation=Department of Mathematics",
      "affiliation=San Diego",
      "affiliation=State University, CA",
      "affiliation=Albert Einstein Institute",
      "affiliation=Max Planck",
      "affiliation=Institute for Gravitational Physics",
    ],
    &[
      "Jan Jones",
      "affiliation=University of California",
      "affiliation=Santa Barbara",
      "affiliation=Kavli Institute for Theoretical Physics",
      "affiliation=University of Michigan",
      "affiliation=Ann Arbor",
      "affiliation=Center for the Study of Complex Systems",
    ],
    &[
      "Kim Kerr",
      "affiliation=Department of Medicine, University of X",
      "affiliation=Associate Dean",
      "affiliation=School of Medicine, University of X",
      "affiliation=Vice Provost",
      "affiliation=Office of Research, University of X",
      "affiliation=Department of Psychology",
      "affiliation=Cognitive Neuroscience",
      "affiliation=Research Unit, Aarhus",
      "affiliation=Technical Report",
      "affiliation=Department of Physics",
      "affiliation=Lecture Notes",
      "affiliation=Summer School on Physics",
    ],
    &[
      "Lee Lowe",
      "affiliation=University of X",
      "affiliation=Costa Rica",
    ],
    &[
      "Mia Moss",
      "affiliation=Department of Physics",
      "affiliation=University of X",
      "affiliation=Hong Kong",
    ],
    &[
      "Ned Nash",
      "affiliation=University of X",
      "affiliation=Hong Kong",
    ],
    &[
      "Oli Ott",
      "affiliation=University of X",
      "affiliation=Hong Kong",
    ],
    &[
      "Pam Poe",
      "affiliation=University of X",
      "affiliation=Hong Kong",
    ],
    &[
      "Quin Quay",
      "affiliation=University of X",
      "affiliation=Santa Barbara",
      "affiliation=California",
    ],
    &[
      "Rex Roe",
      "affiliation=University of X",
      "affiliation=Santa Barbara",
      "affiliation=California",
    ],
  ];
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(found, want, "{xml}");
}

/// The stacked-run veto, both sides. A run of name lines directly under the names line, each styled as the line above it
/// and reading as a name (1702.07694's `\name … \email …` lines over a shared `\addr` block), keeps the lines under it:
/// its last name never takes them alone by position, though an institution stands under it and it has an initial (the
/// 85a r1 review's finding 1). Asserted only where today's reading (Bob B. Baker and Cat C. Cole lines of Ann Able, the
/// Perl and int114a one) and the right one (each a creator sharing the shared lines: the RED
/// author_block_stacked_names_each_a_creator) agree: Ann Able keeps her e-mail and the three shared lines, and no
/// creator but the whole run holds any of them. After a gap, a line under rows of names (`\textbf{Fay Fox \quad Gil
/// Gray} \\ \textbf{Hal Hunt \quad Ida Ives\thanks{…}} \\ \\ CUHK MMLab`, 2604.18394, the r3 review's finding 1, k1) is
/// the rows' line, however they are styled; a name under a name-shaped line (`Hong Kong`) whose lines do not all read
/// as names (`Hanson Robotics` above it) opens the next author (q11; 0801.1297; the r2 review's finding 5a). Red in 85a
/// r1: Cat C. Cole a creator holding the run's lines; int114a: Eve Eads a line of Dan Dorn; 85a r3: a creator "CUHK
/// MMLab".
#[test]
fn stacked_names_keep_their_lines() {
  let (log, xml) = convert_with(
    repro!("author_block_stacked_names_keep_their_lines.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = creators(&xml);
  let creator = |name: &str| {
    found
      .iter()
      .find(|c| c[0] == name)
      .unwrap_or_else(|| panic!("no creator {name}: {xml}"))
  };
  let shared = [
    "affiliation=School of Operations Research",
    "affiliation=290 Rhodes Hall, Cornell University",
    "affiliation=Ithaca, NY 14853, USA",
  ];
  let ann = creator("Ann Able");
  assert_eq!(
    ann.get(1).map(String::as_str),
    Some("email=ann@x.edu"),
    "{xml}"
  );
  // (each holder of a shared line holds the three, in order, last)
  let holders: Vec<&str> = found
    .iter()
    .filter(|c| c.iter().any(|contact| shared.contains(&contact.as_str())))
    .map(|c| {
      assert!(c.ends_with(&shared.map(String::from)), "{xml}");
      c[0].as_str()
    })
    .collect();
  assert!(
    holders == ["Ann Able"] || holders == ["Ann Able", "Bob B. Baker", "Cat C. Cole"],
    "{holders:?}: {xml}"
  );
  assert_eq!(creator("Dan Dorn")[1..], [
    "affiliation=Hanson Robotics",
    "affiliation=Hong Kong"
  ]);
  assert_eq!(creator("Eve Eads")[1..], ["affiliation=Univ B"]);
  for name in ["Fay Fox", "Gil Gray"] {
    assert!(
      creator(name).iter().any(|c| c == "affiliation=CUHK MMLab"),
      "{xml}"
    );
  }
  assert!(!found.iter().any(|c| c[0].contains("CUHK")), "{xml}");
}
