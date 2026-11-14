//! Guards for Z18g: the llncs reroute's last line rules (OXIDIZED_DESIGN_DIVERGENCES #984). One guard per mechanism:
//! a line of link pieces read as addresses once their inks are joined, in the shared `\institute` (the Kind step for an
//! address line's joined reading) and in the `\author` block (`resolver::joined_address_pieces`, a different path: the
//! reading's events); a shared institute holding the paper's own marks kept whole (`resolver::hand_marked_share`); an
//! ORCID icon link keeping its iD (`frontmatter_fields::orcid_icon_link`); a link whose text is the binding's e-mail
//! call read as that call (the rehearsal's `\lx@fm@obs@href`). The 85b guards hold the shared institute's call units,
//! its unread later line and its first line. Each converts its repro and asserts the Error and Warning counts and every
//! creator's name and contacts in order.

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

/// Converts `tex`, checks its Error and Warning counts and compares its creators; returns the XML.
#[track_caller]
fn assert_creators(tex: &str, counts: (usize, usize), expected: &[&[&str]]) -> String {
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), counts, "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  xml
}

/// llncs's shared `\institute{Univ X, Town\\ \mailsa, \mailsb, \mailsc}`, three `\urldef`'d `\path` pieces printing
/// `{ann.able, bob.baker, cat.cole}@x.edu`: each address its author's, the institute every author's (1504.01099). Red
/// (Z18e): the list in every author's institute. The two Warnings are the content check's, as on int154x: the rehearsal
/// prints the `\path` delimiters (OT1's `|`, an em dash) before each piece, so it reads `cat.cole}@x.edu` as a word —
/// hiding those delimiters made the emitter's address cut take the list apart and leave that piece (a residual).
#[test]
fn llncs_institute_urldef_pieces_are_one_address_line() {
  let institute = "affiliation=Univ X, Town";
  assert_creators(
    repro!("llncs_institute_urldef_pieces_are_one_address_line.tex"),
    (0, 2),
    &[
      &["Ann Able", institute, "email=ann.able@x.edu"],
      &["Bob Baker", institute, "email=bob.baker@x.edu"],
      &["Cat Cole", institute, "email=cat.cole@x.edu"],
    ],
  );
}

/// llncs's `\author` block with an address line of `\url` pieces (`\url{ann.able@x.edu}, \{\url{bob.baker,
/// cat.cole}}\}\url{@x.edu}`, `\url{cat.cole}\url{@y.edu}`): one e-mail contact per line once joined, each address its
/// author's (1710.01968). Red (Z18e): `cat.cole @y.edu` and `@y.edu` URLs on every author, `ann.able@` glued to the next
/// local parts. Distinct from the institute's: the block's events are read (`read_events`), not the written entry.
/// Re-expected in Z18i (review revZ18h B1r): the three addresses no link of the run targets — bob.baker@, cat.cole@x.edu,
/// cat.cole@y.edu — are written as text and their lost `\url` links counted, 3 Warnings (Z18h lost them silently);
/// ann.able@ keeps its own `\url` link.
#[test]
fn llncs_author_url_pieces_are_one_address_line() {
  let (x, y) = ("affiliation=Univ X", "affiliation=Univ Y");
  assert_creators(
    repro!("llncs_author_url_pieces_are_one_address_line.tex"),
    (0, 3),
    &[
      &["Ann Able", x, "email=ann.able@x.edu", y],
      &["Bob Baker", x, "email=bob.baker@x.edu", y],
      &[
        "Cat Cole",
        x,
        "email=cat.cole@x.edu",
        y,
        "email=cat.cole@y.edu",
      ],
    ],
  );
}

/// llncs's one `\institute{\affmark{1} Univ A \quad \affmark{2} Univ B\\ \affmark{3} Univ C}` holding the paper's own
/// marks (2608.21425): shape (c) by the marks it prints, each entry its mark's, as the `$^{1}$` spelling's (P4b4a: the
/// cutter reads where entries start as the Cut's decision does, `list_marks`). Red (Z18e): each line on the names
/// showing its first mark, with "markup lost" Warnings; Z18g..int165a kept it whole on the block (the token dispatch saw
/// no mark).
#[test]
fn llncs_institute_hand_marks_cut() {
  assert_creators(repro!("llncs_institute_hand_marks_cut.tex"), (0, 0), &[
    &["Ann Able 1", "affiliation=1 Univ A"],
    &[
      "Bob Baker 2,3",
      "affiliation=2 Univ B",
      "affiliation=3 Univ C",
    ],
  ]);
}

/// An ORCID link of llncs's `\author` block printing no word (`\href{https://orcid.org/\orcidauthorA}{\orcidicon}`, the
/// template's `\xdef` leaving the iD's macro in the address): the kernel's ORCID link to the iD, its title naming it,
/// the icon inside (2008.02173, 2009.11534). Red (Z18e): a plain link, the iD's title lost.
#[test]
fn llncs_author_orcid_icon_keeps_the_id() {
  let xml = assert_creators(
    repro!("llncs_author_orcid_icon_keeps_the_id.tex"),
    (0, 0),
    &[&["Ann Able", "orcid=", "affiliation=Univ X"], &[
      "Bob Baker",
      "affiliation=Univ X",
    ]],
  );
  assert!(
    xml.contains(
      r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097"><rule"#
    ),
    "{xml}"
  );
}

/// `\thanks{Univ X, Town. E-mail: \href{mailto:ann.able@x.edu}{\email{ann.able@x.edu}}. Funded by Y.}` under llncs: the
/// link is the e-mail call it holds, the address Ann Able's e-mail (1004.3134). Red (Z18e): the address lost.
#[test]
fn llncs_thanks_link_holding_email_keeps_the_address() {
  let xml = assert_creators(
    repro!("llncs_thanks_link_holding_email_keeps_the_address.tex"),
    (0, 0),
    &[&["Ann Able", "email=ann.able@x.edu", "affiliation=Univ X"]],
  );
  // (the note keeps its other words, the link in place)
  assert!(
    xml.contains("Univ X, Town. E-mail:") && xml.contains(r#"href="mailto:ann.able@x.edu""#),
    "{xml}"
  );
  assert!(xml.contains("Funded by Y."), "{xml}");
}
