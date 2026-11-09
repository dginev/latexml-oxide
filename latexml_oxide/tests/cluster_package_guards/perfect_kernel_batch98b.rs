//! Red/green guards for batch 98b: four mechanisms that lost printed frontmatter words (the dev lost-words census;
//! OXIDIZED_DESIGN_DIVERGENCES #949-#952). An author block opening with lines that print nothing reads its first
//! printing line as the names, and a nameless creator's lines are the block's when no named author is left to take
//! them (`coalesce_empty_creators`, insert.rs); IEEEtran's grades are read under authblk's `\author` too, and
//! `\IEEEpeerreviewmaketitle` is `\relax` outside the peer-review modes (ieeetran_cls.rs); a `\thanks` inside an
//! `\inst{…}` is the note of the name it is printed on (`\lx@request@frontmatter@annotation`, frontmatter_api.rs);
//! svjour3's text before `\at` is kept where it prints a word no author's name does (sv_support_sty.rs). Each guard
//! converts its repro under `tools/perfect_kernel/repros/` from the binary, eager and `--streaming`, asserts the same
//! XML both ways, no Error and no Warning, and every creator (name and contacts in order). One guard per path: the
//! leading spacer lines (one author; two after a gap), the nameless block's holder, the authblk list and labelled
//! paths, the peer-review title, the `\inst` request (aa's and llncs's author lines), the svjour3 piece; and (round 2,
//! the rev98b review) a list opened before svjour3's `\at`, its structure no word of the line and several names in it
//! no one's, and a later block a line break opens over an institution, no names.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batcht1::red_at_two,
};

macro_rules! repro {
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns the XML.
#[track_caller]
fn both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (streamed_log, streamed) = cli_conversion(tex, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (0, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  eager
}

/// `expected` as owned strings, for comparing with [`creators`].
fn owned(expected: &[&[&str]]) -> Vec<Vec<String>> {
  expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect()
}

/// `{}~\\[0mm]~\\ \large Ann Able\\ …` (hep-lat9207028, 0812.2273, 0901.1754, hep-lat0105010, math0306336; int126a: no
/// creator at all) names Ann Able; and `\ \\ Ann Able \\ Univ X \\ \\ Bob Baker \\ Univ Y` (solv-int9710025; int126a:
/// Ann Able and her line after Bob Baker's) is two authors, each with their line.
#[test]
fn leading_spacer_lines_are_no_names_line() {
  let xml = both_ways(repro!(
    "sectioning-frontmatter",
    "author_block_leading_spacer_lines_are_no_names_line"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[&[
      "Ann Able",
      "affiliation=Department of Physics",
      "affiliation=University of X"
    ]])
  );
  let xml = both_ways(repro!(
    "sectioning-frontmatter",
    "author_block_leading_spacer_line"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[&["Ann Able", "affiliation=University of X"], &[
      "Bob Baker",
      "affiliation=University of Y"
    ],])
  );
}

/// An author block of affiliation lines and no name (1804.00021's `\authorblockA` with its names commented out;
/// int126a: no creator) keeps its lines on the block's creator, an empty name over them (`<personname/>`), as Perl does;
/// since 106a its "Email:" line is the block's e-mail contact (the Kind step for an entry's address line, OD #966).
#[test]
fn nameless_block_keeps_its_lines() {
  if red_at_two(r#"2 false content-check Warnings (X placed more often, XEmail missing)"#) {
    return;
  }
  let xml = both_ways(repro!(
    "class-bindings",
    "ieeetran_affiliation_block_without_names_is_the_blocks"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[&[
      "affiliation=Center for Data Science (CDS) University of X",
      "email=ann@x.edu"
    ]])
  );
  assert!(xml.contains("<personname/>"), "{xml}");
}

/// IEEEtran under authblk (1803.04687, 2208.04191; IEEEtaes 2505.19506, IEEEoj 2506.05637): each author keeps its
/// grade as a membership contact — the last one's on the line after its name (2202.10869) — and the note stays the
/// last author's (int126a: no grade).
#[test]
fn authblk_author_reads_ieee_grades() {
  let xml = both_ways(repro!(
    "class-bindings",
    "ieeetran_authblk_membership_grades"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &["Ann Able", "membership=Student Member, IEEE"],
      &["Bob Bell", "membership=Senior Member, IEEE"],
      &["Cat Cole", "membership=Fellow, IEEE"],
    ])
  );
  assert_eq!(
    author_note(&xml, "Cat Cole"),
    "A. Able, B. Bell and C. Cole are with the University of X.",
    "{xml}"
  );
}

/// authblk's labelled `\author[1]{Ann Able,~\IEEEmembership{Senior Member,~IEEE,}}` (2307.05035; int126a: no grade)
/// keeps its grade and its labelled affiliation; the unlabelled-grade author beside it is untouched. An empty label is
/// handed on as written (`\author[]{Wei Xiong}`, 2005.04149, 2105.12254, 2312.15778; 98b's first build: the brace
/// list of addresses welded into every author's affiliation).
#[test]
fn authblk_labelled_author_reads_its_grade() {
  let xml = both_ways(repro!(
    "class-bindings",
    "ieeetran_authblk_labelled_author_grade"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &[
        "Ann Able",
        "membership=Senior Member, IEEE",
        "affiliation=University of X"
      ],
      &["Bob Bell", "affiliation=University of Y"],
    ])
  );
  let xml = both_ways(repro!(
    "class-bindings",
    "ieeetran_authblk_empty_label_authors_share_the_lines"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &[
        "Ann Able",
        "affiliation=Department of Physics, University of X",
        "email=aable@x.edu"
      ],
      &[
        "Bob Bell",
        "affiliation=Department of Physics, University of X",
        "email=bbell@x.edu"
      ],
    ])
  );
}

/// `\IEEEpeerreviewmaketitle` before `\title` in a journal paper (1909.01874; int126a: no title, no creator) does
/// nothing, as IEEEtran.cls:4930-4931 defines it outside the peer-review modes.
#[test]
fn peerreviewmaketitle_outside_peerreview_is_relax() {
  let xml = both_ways(repro!(
    "class-bindings",
    "ieeetran_peerreviewmaketitle_outside_peerreview_is_relax"
  ));
  assert!(xml.contains("<title>A Title</title>"), "{xml}");
  assert_eq!(
    creators(&xml),
    owned(&[&["Ann Able", "membership=Member, IEEE"]])
  );
}

/// A `\thanks` inside `\inst{…}` is the note of the author it is printed on, the rest of the operand the request
/// (aa 2303.08174, 2311.02521, 2601.20473; llncs 1808.09829; int126a: the note lost), beside the `\fnmsep\thanks`
/// form after the `\inst`, which was kept before.
#[test]
fn thanks_inside_inst_is_the_authors_note() {
  let xml = both_ways(repro!(
    "class-bindings",
    "aa_thanks_inside_inst_is_the_authors_note"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &["Ann Able", "affiliation=University of X"],
      &["Bob Bell", "affiliation=University of Y"],
      &[
        "Cat Cole",
        "affiliation=University of X",
        "affiliation=University of Y"
      ],
    ])
  );
  for (author, note) in [
    ("Bob Bell", "NASA Sagan Fellow"),
    ("Cat Cole", "Hubble Fellow"),
  ] {
    assert_eq!(author_note(&xml, author), note, "{xml}");
  }
  let xml = both_ways(repro!(
    "class-bindings",
    "llncs_thanks_inside_inst_is_the_authors_note"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &["Ann Able", "affiliation=University of X"],
      &["Bob Bell", "affiliation=University of Y"],
      &["Cat Cole", "affiliation=University of Y"],
    ])
  );
  for (author, note) in [
    ("Bob Bell", "Corresponding author: bob@y.edu"),
    ("Cat Cole", "Supported by Z."),
  ] {
    assert_eq!(author_note(&xml, author), note, "{xml}");
  }
}

/// The text of the one `<note>` of the creator named `author`.
#[track_caller]
fn author_note(xml: &str, author: &str) -> String {
  let text = super::perfect_kernel_batch84b::text;
  let creator = xml
    .match_indices("<creator ")
    .map(|(start, _)| &xml[start..start + xml[start..].find("</creator>").expect("closed creator")])
    .find(|creator| {
      creator.find("<personname>").is_some_and(|at| {
        let close = at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname");
        text(&creator[at + "<personname>".len()..close]) == author
      })
    })
    .unwrap_or_else(|| panic!("no creator {author}: {xml}"));
  let notes: Vec<&str> = creator
    .match_indices("<note ")
    .map(|(at, _)| &creator[at..])
    .collect();
  assert_eq!(notes.len(), 1, "{creator}");
  let note = notes[0];
  let body =
    &note[note.find('>').expect("note tag") + 1..note.find("</note>").expect("closed note")];
  let body = body
    .rfind("</tags>")
    .map_or(body, |at| &body[at + "</tags>".len()..]);
  text(body)
}

/// svjour3's text before each `\at` (1711.01894's "Contact author:", 2107.13864's "Corresponding author", 1705.01399's
/// fuller name; int126a: only the owner's label) is the affiliation's first line where it prints a word no author's
/// name does, and nothing where it is only the name.
#[test]
fn svjour3_text_before_at_is_kept() {
  let xml = both_ways(repro!(
    "class-bindings",
    "svjour3_institute_text_before_at_is_kept"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &[
        "Ann Able",
        "affiliation=Contact author: Ann Able University of X",
        "email=ann@x.edu"
      ],
      &[
        "Bob B. Baker",
        "affiliation=Bob Bernard Baker University of Y"
      ],
      &["Cat Cole", "affiliation=University of Z"],
    ])
  );
}

/// A list opened before svjour3's `\at` (2305.05902, 2306.07927, 2605.09054; 98b: the line "∗ Bob Baker" kept for the
/// word "itemize", 1 Error) is structure, no word of the line: the name's author holds the list, whose `\begin` is kept
/// as the address closes it; one naming two or more authors before its one `\at` (2312.14898; int126a and 98b: the
/// list on the last name's alone, 98b its address twice) names no one, each address its author's and the list shared.
/// Distinct from [`svjour3_text_before_at_is_kept`]: the list commands, not the line's words, and the several names.
#[test]
fn svjour3_list_before_at_is_structure() {
  let xml = both_ways(repro!(
    "class-bindings",
    "svjour3_institute_list_before_at_names_its_author"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[&["Ann Able 1"], &[
      "Bob Baker 1,2",
      "affiliation=1 item 1 University of X 2 item 2 University of Y"
    ],])
  );
  assert_eq!(xml.matches("<itemize").count(), 1, "{xml}");
  let xml = both_ways(repro!(
    "class-bindings",
    "svjour3_institute_list_before_at_names_several_authors"
  ));
  assert_eq!(
    creators(&xml),
    owned(&[
      &["Ann Able 1", "email=ann@x.edu"],
      &["Bob Baker 2", "email=bob@y.edu"],
      &["affiliation=1 item 1 University of X 2 item 2 University of Y"],
    ])
  );
  assert_eq!(xml.matches("<itemize").count(), 1, "{xml}");
  // (the list on the block's creator, #159's shared orphan holder: no name of its own)
  assert_eq!(xml.matches("<creator ").count(), 3, "{xml}");
  assert_eq!(xml.matches("<personname").count(), 2, "{xml}");
}

/// A block a line break opens after an author separator, its first line an institution (`\author{Ann Able \and ~\\
/// University of X}`, `… \and \\ University of X`: the rev98b review's pb1, pb6; 98b: a creator "University of X" for
/// both, int126a for pb6), has no names: its line is Ann Able's. Distinct from
/// [`leading_spacer_lines_are_no_names_line`]: a later block, its first line no name.
#[test]
fn institution_after_and_and_a_break_is_no_name() {
  for (name, tex) in [
    (
      "pb1",
      repro!(
        "sectioning-frontmatter",
        "author_block_institution_after_and_spacer_line"
      ),
    ),
    (
      "pb6",
      repro!(
        "sectioning-frontmatter",
        "author_block_institution_after_and_line_break"
      ),
    ),
  ] {
    assert_eq!(
      creators(&both_ways(tex)),
      owned(&[&["Ann Able", "affiliation=University of X"]]),
      "{name}"
    );
  }
}
