//! Guards for Z18d, review revZ18c's should-fixes: the entry's e-mail cut and the row's split share one reader of a
//! call (`read_call`, `classify/entry_emails.rs` since P27b), so a later call in printed braces is read as the row read it; the emitter's "markup
//! lost" Warning tests the text it writes (`emit.rs` `one_address`). Each converts its repro and asserts every creator's
//! name and contacts in order and the exact Error and Warning counts.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// `\email{a1@x.edu}, \email{\{b2@x.edu\}}, \email{c3@x.edu}` in a marked institute (review revZ18c S2): the braced
/// call after the first is read as one address (`single_braced`, 2410.04091, the reader the old split had), so it is
/// the entry's and the chain places it at Bob Baker's place. Red (Z18c): Bob Baker had no e-mail and Cid Cole held
/// `{b2@x.edu}` and c3@, with no Warning.
#[test]
fn a_later_braced_email_call_keeps_its_author() {
  let (log, xml) = run(repro!("llncs_later_braced_email_call_keeps_its_author.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    [
      ["Ann Able", "affiliation=Univ X, Town", "email=a1@x.edu"],
      ["Bob Baker", "affiliation=Univ X, Town", "email={b2@x.edu}"],
      ["Cid Cole", "affiliation=Univ X, Town", "email=c3@x.edu"],
    ]
    .map(|c| c.map(str::to_string).to_vec())
    .to_vec(),
    "{xml}"
  );
}

/// `\email{\{ann.able, bob.baker\}@a.edu\footnote{Both equal.}}` (review revZ18c S3): the list is plain characters and
/// control symbols, but the address text written of it holds the note's name, so each address is counted "markup lost"
/// (as at ERR2), beside the content check's excess. Red (Z18c): only the content check's Warning. The name in the text
/// is pre-existing and RED (`llncs_list_email_note_name_leaks_into_address.tex`), so the e-mails are not pinned here.
#[test]
fn a_list_address_written_with_a_name_is_counted() {
  let (log, xml) = run(repro!(
    "llncs_list_email_note_written_as_text_is_counted.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  let lost: Vec<&str> = log
    .lines()
    .filter(|line| line.starts_with("Warning:") && line.contains("its markup lost"))
    .collect();
  assert_eq!(lost.len(), 2, "{log}");
  assert!(lost[0].contains("writes \"ann.able@a.edu\\"), "{log}");
  assert!(lost[1].contains("writes \"bob.baker@a.edu\\"), "{log}");
  let names: Vec<String> = creators(&xml).into_iter().map(|c| c[0].clone()).collect();
  assert_eq!(names, ["Ann Able", "Bob Baker"], "{xml}");
}

/// A plain list whose domain holds a control symbol (`\{a,b\}@x\_y.org`) prints `a@x_y.org` and `b@x_y.org`, all the
/// text written: no "markup lost" Warning counts a loss there (review revZ18d finding 1 — a well-formedness test of the
/// written address counted both). Only a control sequence's name or a tie in the written text is counted (the guard
/// above). Distinct: the written-text test's false side.
#[test]
fn a_plain_list_with_an_escaped_domain_counts_no_lost_markup() {
  let (log, _) = run(
    "\\documentclass{llncs}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\inst{1} \\and Bob Baker\\inst{1}}\n\
     \\institute{Univ A, Town\\\\ \\email{\\{a,b\\}@x\\_y.org}}\n\\maketitle\nBody.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(!log.contains("its markup lost"), "{log}");
}
