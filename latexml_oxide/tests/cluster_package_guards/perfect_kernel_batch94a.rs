//! Guards for batch 94a (frontmatter refactor stage 7, "name parting last": the writer's re-part,
//! OXIDIZED_DESIGN_DIVERGENCES #928; `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`). Where the observer's reading parts
//! the names and today's old reader parted them otherwise, the writer writes the creators again from the reading — each
//! name from its span's ink, its items copied from today's or made from their spans — and keeps them only when they read
//! as the reading and hold every word today's creators held. Each guard converts its repro under the shadow switch in
//! process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), asserts the
//! reading's tuples and the XML's creators whole, and no Error or Warning; and the default mode (the fleet's) writes the
//! same XML, notes included. Every repro compiles clean with pdflatex (TL 2025).

use serde_json::Value;

use super::{
  perfect_kernel_batch92a::assert_reading,
  perfect_kernel_batch93a::{default_mode_creators, owned},
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// A name's span clipped where the next event starts, less the group it opens (`name_tokens`): the names after `\\ \bf
/// \Large \textbf{…}` carry the cover of `\textbf`'s argument in their ink's span, so the first of them spanned the
/// rest (1909.05311, aaai20). Today's old reader held "Cid Cole, Dee Dunn" as one creator (an Error,
/// `merged_creators`); written, four creators, each its affiliation made from its `\ `-separated marked line — and the
/// Error, which reads the authors as written (`flag_merged_creators` after the writer), is gone.
#[test]
fn names_after_a_bold_line_are_reparted() {
  let tex = repro!("author_names_reparted_after_a_bold_line.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=1 Univ X"],
    &["Bob Baker", "affiliation=2 Univ Y"],
    &["Cid Cole", "affiliation=1 Univ X"],
    &["Dee Dunn", "affiliation=2 Univ Y"],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
    "C Cid Cole [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Dee Dunn [2] | line:names sep:comma",
    "  a Univ Y | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// A layout inside `\author{}` that today's old reader keeps as one creator's name: a minipage of `\quad`-parted names
/// and marked lines (2609.32870), and a `\makebox{\begin{tabular}…}` of comma-parted names, institution rows and a
/// `$^{\dagger}$` legend row (2609.33104). The word check reads a layout's cells and rows as parting the words beside
/// them (`dom::words_without_marks`: the name in one row and the institution in the next read glued otherwise); an
/// affiliation in a `{\small\normalfont …}` group keeps its switches; the legend is the dagger name's note (ruling 2,
/// `Made::Legend`), the asterisk line in the minipage the reading's affiliation of the starred name.
#[test]
fn names_in_a_layout_are_reparted() {
  let tex = repro!("author_names_reparted_in_a_minipage.tex");
  let xml: &[&[&str]] = &[
    &["Ann Able", "affiliation=1 Univ X"],
    &["Bob Baker", "affiliation=2 Univ Y"],
    &[
      "Cid Cole",
      "affiliation=1 Univ X",
      "affiliation=∗ Equal advising.",
    ],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Univ X | ruling1:request",
    "C Bob Baker [2] | line:names sep:gap",
    "  a Univ Y | ruling1:request",
    "C Cid Cole [1,*] | line:names sep:gap",
    "  a Univ X | ruling1:request",
    "  a Equal advising. | ruling2:symbol-line",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("author_names_reparted_in_a_tabular_with_a_legend.tex");
  let reading = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=1 Univ X"],
      &["Bob Baker", "affiliation=2 Univ Y"],
      &["Cid Cole", "affiliation=2 Univ Y"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | ruling1:request",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ Y | ruling1:request",
      "C Cid Cole [2,†] | line:names sep:comma",
      "  a Univ Y | ruling1:request ruling1:parted",
      "  n Corresponding author | ruling2:symbol-line ruling1:parted",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &["Ann Able", "affiliation=1 Univ X"],
      &["Bob Baker", "affiliation=2 Univ Y"],
      &[
        "Cid Cole",
        "affiliation=2 Univ Y",
        "note=† Corresponding author"
      ],
    ])
  );
}

/// An entry of several pieces, which the reading sources at its first Name (`entry_span`): the writer reads on through
/// the events after it until their ink is the entry's text — the word "and" inside it (`Department of Physics and
/// Astronomy`, 1304.1574's `Computer Science and Engineering`) the entry's —, from a minipage of `$^1${\it …}` lines
/// today's old reader kept in the one creator's name with `Ann Able$^{\,1}$ and~ Bob Baker$^{\,2}$` (2307.03384); the
/// "and" between the names a separator the block printed, which no creator keeps.
#[test]
fn an_entry_of_several_pieces_is_written_whole() {
  let tex = repro!("author_names_reparted_entries_of_several_pieces.tex");
  let xml: &[&[&str]] = &[
    &[
      "Ann Able",
      "affiliation=Department of Physics and Astronomy, Univ X, Japan",
    ],
    &[
      "Bob Baker",
      "affiliation=Department of Physics, Univ Y, Japan",
    ],
  ];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a Department of Physics and Astronomy, Univ X, Japan | ruling1:request",
    "C Bob Baker [2] | line:names sep:gap",
    "  a Department of Physics, Univ Y, Japan | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}

/// What today's name held besides the name moves to where the reading puts it: an unmarked line under the name
/// (`{\footnotesize for the Example collaboration}`, R12's row above) was in today's personname (1502.03246) and is the
/// name's affiliation, made from its tokens; and `\and` columns whose last is a line of marked entries over an e-mail
/// line, each piece a creator of today's (2308.10621): the legend is the starred names' note, the entry the line under
/// it continues read on over the break (`line:continued`), each item in print order.
#[test]
fn what_a_name_held_is_written_where_the_reading_puts_it() {
  let tex = repro!("author_names_reparted_a_line_under_the_name.tex");
  let xml: &[&[&str]] = &[&[
    "Ann Able",
    "affiliation=for the Example collaboration",
    "affiliation=Univ X, Sometown, Country",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a for the Example collaboration | line:unmarked-joined R12:row-above",
    "  a Univ X, Sometown, Country | ruling1:request",
  ]);
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(default_mode_creators(tex), owned(xml));
  let tex = repro!("author_names_reparted_columns_with_a_continued_line.tex");
  let reading = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=1 Univ X"],
      &[
        "Bob Baker",
        "affiliation=1 Univ X",
        "affiliation=2 Lab Y ann.able, bob.baker@lab.org",
      ],
      &["Cid Cole", "affiliation=1 Univ X"],
    ]),
    &[
      "C Ann Able [*,1] | line:names",
      "  a Univ X | ruling1:request",
      "  n equal contribution | ruling2:symbol-line",
      "C Bob Baker [*,1,2] | line:names sep:column",
      "  a Univ X | ruling1:request",
      "  a Lab Y ann.able, bob.baker@lab.org | ruling1:request line:continued",
      "  n equal contribution | ruling2:symbol-line",
      "C Cid Cole [1] | line:names sep:column",
      "  a Univ X | ruling1:request",
    ],
  );
  assert_eq!(reading.3, Value::from("written"));
  assert_eq!(
    default_mode_creators(tex),
    owned(&[
      &[
        "Ann Able",
        "affiliation=1 Univ X",
        "note=∗ equal contribution"
      ],
      &[
        "Bob Baker",
        "affiliation=1 Univ X",
        "affiliation=2 Lab Y ann.able, bob.baker@lab.org",
        "note=∗ equal contribution"
      ],
      &["Cid Cole", "affiliation=1 Univ X"],
    ])
  );
}

/// The fail-safe: what the re-part built is taken back, and today's output stays, when it does not read as the
/// reading. An `\affil{$^1$…}` inside `\author` is one Contact event whose text keeps its mark's ink ("1Department of
/// Physics, Univ X"), the affiliation made from it reads without it (2405.08158, 1602.05576: ar-1col's affiliations in
/// the last name, left by the fail-safe — the reading's, not the writer's, to fix).
#[test]
fn a_repart_that_does_not_read_as_the_reading_is_taken_back() {
  let tex = repro!("author_names_reparted_declined_mark_in_contact.tex");
  let xml: &[&[&str]] = &[&["Ann Able 1"], &[
    "Bob Baker Department of Physics, Univ X Department of Physics, Univ Y 2,1",
  ]];
  let reading = assert_reading(tex, Some(xml), &[
    "C Ann Able [1] | line:names",
    "  a 1Department of Physics, Univ X | owner:adjacent owner:upfront-grid",
    "C Bob Baker [2] | line:names sep:word",
    "  a 2Department of Physics, Univ Y | owner:adjacent",
  ]);
  assert_eq!(reading.3, Value::from("declined: the creators' names"));
  assert_eq!(default_mode_creators(tex), owned(xml));
}
