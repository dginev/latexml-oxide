//! Guards for Z18o, the REDs bundle int155a introduced (OXIDIZED_DESIGN_DIVERGENCES #992, #991, #983):
//! - an unmarked `\affil` the `annotate=new` walk gives the last author only as its stopgap is no carrier's of an
//!   address no one spells: the block's (`RUN_LEFTOVER`, review revZ18l F1) — on the Kind step's lines and the text
//!   reading's (one guard), and on an unrequested symbol line's `annotate=` placement (its own path);
//! - a mailbox link line in an affiliation is an e-mail line under the text before it (the Kind step's
//!   `mailbox_link_line`, user ruling 2026-10-20; review revZ18l F2);
//! - a marked bracketed piece the bracket spans whole is a remark, its join the counted Warning (review revZ18k3 R1);
//! - a class's own `\email` holding a class's font-size command is rehearsed within its budget (review revPC3 1).
//!
//! Each converts its repros and asserts the exact Error and Warning counts, every creator's name and contacts in order,
//! and the e-mail contact whole where one is made; each is red on its base.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

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

/// The creators of `xml` as `creators` gives them, against `expected`.
fn assert_creators(xml: &str, expected: &[&[&str]]) {
  let expected: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| s.to_string()).collect())
    .collect();
  assert_eq!(creators(xml), expected, "{xml}");
}

/// The e-mail contacts of `xml`, whole, in order.
fn email_contacts(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<contact ")
    .filter(|(at, _)| {
      xml[*at..]
        .split('>')
        .next()
        .is_some_and(|tag| tag.contains("role=\"email\""))
    })
    .map(|(at, _)| {
      &xml[at..at + xml[at..].find("</contact>").expect("closed contact") + "</contact>".len()]
    })
    .collect()
}

/// emulateapj's `\author{Ann Able$^{1}$ and Bob Baker$^{1}$}\affil{$^1$Univ A}` then an unmarked `\affil` holding only
/// an address that spells no one: `Email: zz@x.org` and a bare `zz@x.org` (the Kind step's address line), and
/// `Correspondence: \href{mailto:zz@x.org}{zz@x.org}` (its mailbox link line): the block's e-mail, never Bob Baker's,
/// whom the `annotate=new` walk reached only as its stopgap. Red (int155a4): Bob Baker's e-mail in each.
#[test]
fn aastex_unmarked_email_line_under_marked_names_is_the_blocks() {
  for (tex, email) in [
    (
      repro!(
        "sectioning-frontmatter",
        "aastex_affil_unmarked_email_line_under_marked_names_is_the_blocks"
      ),
      "<contact name=\"Email:\u{a0}\" role=\"email\">zz@x.org</contact>",
    ),
    (
      repro!(
        "sectioning-frontmatter",
        "aastex_affil_unmarked_bare_address_under_marked_names_is_the_blocks"
      ),
      "<contact name=\"Email:\u{a0}\" role=\"email\">zz@x.org</contact>",
    ),
    (
      repro!(
        "sectioning-frontmatter",
        "aastex_affil_unmarked_mailbox_link_under_marked_names_is_the_blocks"
      ),
      "<contact name=\"Correspondence:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
       href=\"mailto:zz@x.org\">zz@x.org</ref></contact>",
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_creators(&xml, &[
      &["Ann Able 1", "affiliation=1 Univ A"],
      &["Bob Baker 1", "affiliation=1 Univ A"],
      &["email=zz@x.org"],
    ]);
    assert_eq!(email_contacts(&xml), [email], "{xml}");
  }
}

/// `\affil{$^*$ Email: zz@uni.edu}` under `$^{N}$` names, no name requesting `*`: the `annotate=` placement an
/// unrequested symbol line keeps (`piled`) is the stopgap's, so the address is the block's, its `∗` kept. Distinct: the
/// pending entries' placement path (`Resolver::stopgap`). Red (int155a4): Bob Baker's e-mail, the `∗` dropped.
#[test]
fn aastex_unrequested_symbol_email_line_under_marked_names_is_the_blocks() {
  let (log, xml) = run(repro!(
    "sectioning-frontmatter",
    "aastex_affil_unrequested_symbol_email_line_under_marked_names_is_the_blocks"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able 1", "affiliation=1 Uni One"],
    &["Bob Baker 1", "affiliation=1 Uni One"],
    &["email=∗ zz@uni.edu"],
  ]);
  assert_eq!(email_contacts(&xml).len(), 1, "{xml}");
  assert!(
    email_contacts(&xml)[0].contains("∗</sup>zz@uni.edu</contact>"),
    "{xml}"
  );
}

/// An `\affil` that is one link to a mailbox printing no address — `\href{mailto:zz@x.org}{Write to us}`, `Contact:
/// \href{mailto:zz@x.org}{here}` (its label the e-mail's name), `\href{zz@x.org}{Write to us}` (a bare address as
/// target) — is an e-mail, the link kept: here the block's. Red (int155a4): Bob Baker's affiliation in each.
#[test]
fn aastex_mailbox_link_line_is_an_email() {
  for (tex, email) in [
    (
      repro!(
        "sectioning-frontmatter",
        "aastex_affil_mailto_link_line_is_an_email"
      ),
      ("Email:", "mailto:zz@x.org", "Write to us"),
    ),
    (
      repro!(
        "sectioning-frontmatter",
        "aastex_affil_labelled_mailto_link_line_is_an_email"
      ),
      ("Contact:", "mailto:zz@x.org", "here"),
    ),
    (
      repro!(
        "sectioning-frontmatter",
        "aastex_affil_bare_address_link_line_is_an_email"
      ),
      ("Email:", "zz@x.org", "Write to us"),
    ),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let (name, href, printed) = email;
    let printed_contact = format!("email={printed}");
    assert_creators(&xml, &[
      &["Ann Able 1", "affiliation=1 Univ A"],
      &["Bob Baker 1", "affiliation=1 Univ A"],
      &[printed_contact.as_str()],
    ]);
    assert_eq!(
      email_contacts(&xml),
      [format!(
        "<contact name=\"{name}\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
         href=\"{href}\">{printed}</ref></contact>"
      )],
      "{xml}"
    );
  }
}

/// A marked piece the bracket spans whole is no name: llncs's `Ann Able\inst{1}, (deceased)\inst{2}, Bob Baker\inst{2}
/// and [corresponding author]\inst{1}`, `Ann Able\inst{1} and [equal contribution]\inst{1} \and …`, article's `Ann
/// Able$^{1}$, (deceased)$^{\dagger}$, …` — each joined to the name before it, the join the safety net's counted
/// Warning. Red (int155a4): each remark a creator of its own, 0 Warnings.
#[test]
fn marked_bracket_remark_is_no_name() {
  let (log, xml) = run(repro!(
    "class-bindings",
    "llncs_author_marked_bracket_remark_is_no_name"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("(Ann Able | (deceased); Bob Baker | [corresponding author])"),
    "{log}"
  );
  assert_creators(&xml, &[
    &[
      "Ann Able, (deceased)",
      "affiliation=Univ One",
      "affiliation=Lab Two",
    ],
    &[
      "Bob Baker and [corresponding author]",
      "affiliation=Univ One",
      "affiliation=Lab Two",
    ],
  ]);
  let (log, xml) = run(repro!(
    "class-bindings",
    "llncs_author_marked_bracket_remark_after_and_is_no_name"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_creators(&xml, &[
    &["Ann Able and [equal contribution]", "affiliation=Univ One"],
    &["Bob Baker", "affiliation=Lab Two"],
  ]);
  let (log, xml) = run(repro!(
    "sectioning-frontmatter",
    "article_author_marked_bracket_remark_is_no_name"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("(Ann Able | (deceased))"), "{log}");
  assert_creators(&xml, &[
    &[
      "Ann Able 1 , (deceased)",
      "affiliation=Univ One",
      "affiliation=Died 2020",
    ],
    &["Bob Baker", "affiliation=Lab Two"],
  ]);
}

/// A class's own `\email` (1810.03464's vldb-modify.cls, stood in for in the preamble) holding the class's
/// `\normalsize`: the rehearsal reads the argument unexpanded and stays within its read budget, two creators. Red
/// (int155a2): `frontmatter:rehearsal_budget`, the block written whole.
#[test]
fn own_email_normalsize_rehearsal_reads_in_budget() {
  let (log, xml) = run(repro!(
    "sectioning-frontmatter",
    "own_email_normalsize_rehearsal_reads_in_budget"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("rehearsal_budget"), "{log}");
  assert_creators(&xml, &[
    &[
      "Peng Gao",
      "affiliation=UC Berkeley",
      "email=penggao@berkeley.edu",
    ],
    &[
      "Xusheng Xiao",
      "affiliation=Case Western Reserve University",
      "email=xusheng.xiao@case.edu",
    ],
  ]);
}
