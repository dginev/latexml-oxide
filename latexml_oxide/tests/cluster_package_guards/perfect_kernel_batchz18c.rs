//! Guards for Z18c: the frontmatter emitter learns what llncs's old split knew about affiliation lines
//! (OXIDIZED_DESIGN_DIVERGENCES #984), before llncs's unmarked `\author` list goes to the emitter as one block. One guard
//! per rule moved from the old split (`row_email_calls`, `row_address_owners`): the separators and labels around an
//! affiliation entry's inner e-mail calls leave it (`shared_row::entry_email_cut`); the inner e-mail calls after the
//! first are the entry's, placed by the Link step's chain (`EntryOwnerFacts`); an address line ending an entry takes the
//! separator before it (`address_cuts`). Each converts its repro and asserts no Error or Warning and every creator's name
//! and contacts in order.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex`, checks it converts clean and compares its creators; returns the XML.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) -> String {
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  xml
}

/// `\institute{Univ X\\ \email{p1@x.edu}, \email{p2@x.edu}}` under `\inst{1}` marks (Z18b's 15 llncs papers: 1709.03776,
/// 2410.06985, 2609.00838, cs0606057, …): the institute keeps only its own text, the separators before each call gone
/// with it. Red (ERR2): "Univ X ," on both. The e-mail label naming a call goes with it, as its contact's name
/// (`Email:`); a label of its own the old rule recognised goes with it in its text (`Corresponding author:`); a line
/// break stays where text the entry keeps follows the call (2310.01324's `\email{…} \url{…}` line: the link its own
/// contact).
#[test]
fn inner_email_separators_and_labels_leave_the_institute() {
  let tex = repro!(
    "class-bindings",
    "llncs_inner_email_separators_leave_the_institute.tex"
  );
  assert_creators(tex, &[
    &["Ann Able", "affiliation=Univ X", "email=p1@x.edu"],
    &["Bob Baker", "affiliation=Univ X", "email=p2@x.edu"],
  ]);
  let labelled = tex.replace(
    r"\institute{Univ X\\ \email{p1@x.edu}, \email{p2@x.edu}}",
    r"\institute{Univ X, Town; Email: \email{p1@x.edu}; \email{p2@x.edu}}",
  );
  assert_ne!(labelled, tex);
  let xml = assert_creators(&labelled, &[
    &["Ann Able", "affiliation=Univ X, Town", "email=p1@x.edu"],
    &["Bob Baker", "affiliation=Univ X, Town", "email=p2@x.edu"],
  ]);
  assert!(
    creators_xml(&xml)[0].contains("<contact name=\"Email:\u{a0}\" role=\"email\">"),
    "{xml}"
  );
  let own_label = tex
    .replace(r"Bob Baker\inst{1}", r"Bob Baker\inst{2}")
    .replace(
      r"\institute{Univ X\\ \email{p1@x.edu}, \email{p2@x.edu}}",
      r"\institute{Univ X\\ \email{p1@x.edu} \and Univ Y\\ Corresponding author: \email{p2@y.edu}}",
    );
  assert_creators(&own_label, &[
    &["Ann Able", "affiliation=Univ X", "email=p1@x.edu"],
    &[
      "Bob Baker",
      "affiliation=Univ Y",
      "email=Corresponding author: p2@y.edu",
    ],
  ]);
  let link_line = tex.replace(
    r"\institute{Univ X\\ \email{p1@x.edu}, \email{p2@x.edu}}",
    r"\institute{Univ X\\ \email{p1@x.edu}, \email{p2@x.edu} \url{https://x.org/lab}}",
  );
  assert_creators(&link_line, &[
    &[
      "Ann Able",
      "affiliation=Univ X",
      "url=https://x.org/lab",
      "email=p1@x.edu",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ X",
      "url=https://x.org/lab",
      "email=p2@x.edu",
    ],
  ]);
}

/// `\institute{Univ X\\ \email{a1@x.edu}, \email{b2@x.edu}, \email{c3@x.edu}}` under three unmarked names: each call
/// the entry's, the chain placing it one per carrier in order (the old split's `row_address_owners`). Red (Z18b3, and
/// the marked form at ERR2): b2@ and c3@ Cid Cole's — a call digested after another took no label and stayed on the
/// last author as their own call.
#[test]
fn institute_email_calls_are_placed_in_author_order() {
  let tex = repro!(
    "class-bindings",
    "llncs_institute_email_calls_in_author_order.tex"
  );
  let want: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X", "email=a1@x.edu"],
    &["Bob Baker", "affiliation=Univ X", "email=b2@x.edu"],
    &["Cid Cole", "affiliation=Univ X", "email=c3@x.edu"],
  ];
  assert_creators(tex, want);
  let marked = tex.replace(
    r"\author{Ann Able \and Bob Baker \and Cid Cole}",
    r"\author{Ann Able\inst{1} \and Bob Baker\inst{1} \and Cid Cole\inst{1}}",
  );
  assert_ne!(marked, tex);
  assert_creators(&marked, want);
}

/// `\institute{CVLab, EPFL, \\ \texttt{\{able, baker\}@epfl.ch}}` (2407.18381): an address line ending the entry takes
/// the separator the line before it ends in, unmarked (llncs's shared institute) and under `\inst{1}` marks (the entry the
/// emitter writes, its address line cut by the reading's take-out). Red (Z18b3): "CVLab, EPFL," on both.
#[test]
fn last_address_line_takes_the_separator_before_it() {
  let tex = repro!(
    "class-bindings",
    "llncs_institute_last_address_line_takes_its_separator.tex"
  );
  let want: &[&[&str]] = &[
    &["Ann Able", "affiliation=CVLab, EPFL", "email=able@epfl.ch"],
    &[
      "Bob Baker",
      "affiliation=CVLab, EPFL",
      "email=baker@epfl.ch",
    ],
  ];
  assert_creators(tex, want);
  let marked = tex.replace(
    r"\author{Ann Able \and Bob Baker}",
    r"\author{Ann Able\inst{1} \and Bob Baker\inst{1}}",
  );
  assert_ne!(marked, tex);
  assert_creators(&marked, want);
}
