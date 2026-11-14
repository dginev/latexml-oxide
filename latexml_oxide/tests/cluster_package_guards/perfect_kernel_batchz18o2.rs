//! Guards for Z18o2, the scope of the stopgap tag (`RUN_LEFTOVER`, OXIDIZED_DESIGN_DIVERGENCES #992):
//! - a run that is the whole group of names printed since the role's last call is no stopgap: the place ties the
//!   entry to them (`Resolver::print_group`, review revZ18o R-A) — on the unmarked line's walk and on an unrequested
//!   symbol line's `annotate=` placement (a guard each, distinct paths);
//! - the entry the stopgap alone gave its carriers says so to the flush (`_stopgap`), whose Link step of the address the
//!   Kind step takes out of its text reads its carriers as no owners (`insert::link_entry_addresses`).
//!
//! Each converts its repros and asserts the exact Error and Warning counts, every creator's name and contacts in order,
//! and the e-mail contact whole.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batchz4::run,
  perfect_kernel_batchz18o::{assert_creators, email_contacts},
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

const EMAIL: &str = "<contact name=\"Email:\u{a0}\" role=\"email\">zz@x.org</contact>";

/// emulateapj's `\author{Ann Able$^{1}$}\affil{$^1$Univ A}\author{Bob Baker$^{2}$}\affil{$^2$Univ B}\affil{Email:
/// zz@x.org}` prints the address in Bob Baker's own block: his e-mail (red on int156a0: the block's, silently). Two
/// `\author` calls before their shared `\affil` are one group in print, as one call's `Ann Able$^{1}$ and Bob
/// Baker$^{1}$` is: the walk reaches one name of it, so the address stays the block's — the group, not the call, decides.
#[test]
fn aastex_email_line_in_its_own_author_group_is_that_groups() {
  let (log, xml) = run(repro!(
    "aastex_affil_email_line_continuing_own_author_block_is_its_authors"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able 1", "affiliation=1 Univ A"], &[
    "Bob Baker 2",
    "affiliation=2 Univ B",
    "email=zz@x.org",
  ]]);
  assert_eq!(email_contacts(&xml), [EMAIL], "{xml}");

  let (log, xml) = run(repro!(
    "aastex_affil_email_line_under_two_author_calls_is_the_blocks"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able 1", "affiliation=1 Univ A"],
    &["Bob Baker 1", "affiliation=1 Univ A"],
    &["email=zz@x.org"],
  ]);
  assert_eq!(email_contacts(&xml), [EMAIL], "{xml}");
}

/// `\affil{$^*$ Email: zz@x.org}`, no name requesting `*`, after Bob Baker's own `\author` call and `\affil`: Bob Baker's
/// e-mail. Distinct: the pending entries' placement path (`Resolver::stopgap`). Red (int156a0): the block's, `∗ zz@x.org`.
/// The `∗` pdflatex prints is dropped from Bob's contact with no Warning (as on int155a4): a KNOWN LOSS this guard pins,
/// OD #992's Known RED (review revZ18o2 R1), not a target.
#[test]
fn aastex_unrequested_symbol_line_in_its_own_author_group_is_that_groups() {
  let (log, xml) = run(repro!(
    "aastex_affil_unrequested_symbol_line_continuing_own_author_block_is_its_authors"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able 1", "affiliation=1 Univ A"], &[
    "Bob Baker 2",
    "affiliation=2 Univ B",
    "email=zz@x.org",
  ]]);
  assert_eq!(
    email_contacts(&xml),
    ["<contact name=\"Email: \" role=\"email\">zz@x.org</contact>"],
    "{xml}"
  );
}

/// `\affil{Univ C; zz@x.org}` under `Ann Able$^{1}$ and Bob Baker$^{1}$`: Univ C stays Bob Baker's (Perl's `new`), the
/// address the flush's Kind step takes out of it is the block's (red on int156a0: Bob Baker's). After Bob Baker's own
/// `\author` call and `\affil` the entry is no stopgap: both his.
#[test]
fn aastex_line_address_the_flush_takes_out_follows_the_stopgap() {
  let (log, xml) = run(repro!(
    "aastex_affil_unmarked_line_address_under_marked_names_is_the_blocks"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able 1", "affiliation=1 Univ A"],
    &["Bob Baker 1", "affiliation=1 Univ A", "affiliation=Univ C"],
    &["email=zz@x.org"],
  ]);
  assert_eq!(email_contacts(&xml), [EMAIL], "{xml}");

  let (log, xml) = run(repro!(
    "aastex_affil_unmarked_line_address_continuing_own_author_block_is_its_authors"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&["Ann Able 1", "affiliation=1 Univ A"], &[
    "Bob Baker 2",
    "affiliation=2 Univ B",
    "affiliation=Univ C",
    "email=zz@x.org",
  ]]);
  assert_eq!(email_contacts(&xml), [EMAIL], "{xml}");
}
