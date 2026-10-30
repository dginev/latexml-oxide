//! Red/green guards for batch 92e: a frontmatter note reads as the class prints it, and a note with no ink is none
//! (FIDELITY_CHARTER §2 and §4, the user's rulings 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #915, #916).
//!
//! The label: a binding's note named by its role default (`\lx@pubnote@<role>@name`, "Preprint: ", "Journal: ", …)
//! where the class prints another label is given the class's by its `name=` key. One guard per binding the 92e audit
//! fixed, each converting its repro under `tools/perfect_kernel/repros/class-bindings/` from the binary, eager and
//! `--streaming`, with no Error and no Warning, and asserting the whole `ltx:pubnote`: aa's and svjour's `\offprints`
//! "Send offprint requests to:" (aa.cls :337-340, svjour3.cls :371-374; 0903.3512, 1902.07328, 0901.1015), jheppub's
//! `\arxivnumber` "ArXiv ePrint:" (jheppub.sty:223), acmart's `\acmISBN` "ACM ISBN" (acmart.cls:2320), iopart's
//! `\submitto` "Submitted to:" (iopart.cls:271-272), AASTeX's `\submitjournal` "Submitted to" (aastex631.cls:5876),
//! IEEEoj's and ieeetj's `\doiinfo` and the old LIPIcs classes' `\DOI` "Digital Object Identifier" (IEEEoj.cls:3458,
//! ieeetj.cls:3440, lipics.cls:637).
//!
//! The ink: a note (`ltx:pubnote`, a document's or a creator's `ltx:note`) whose digested content typesets nothing is
//! not emitted, whether our extraction emptied it (`\offprints{\email{…}}`: astro-ph0405280, astro-ph0608669) or the
//! author wrote it empty (`\thanks{}`: 2004.01582, 2401.15379, astro-ph0608669); its address stays the contact the
//! binding made of it, and no mark of it is left in the title or the name. A note with any ink stays, an author-typed
//! label alone too (mdpi's `\corres{Correspondence: }`, 1609.03883): the no-change row.
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns it.
#[track_caller]
fn clean_both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(eager, streamed);
  eager
}

/// Checks the conversion of `tex` holds exactly one note of `role`, the whole `expected` element.
#[track_caller]
fn the_note(tex: &str, role: &str, expected: &str) -> String {
  let xml = clean_both_ways(tex);
  let attr = format!("role=\"{role}\"");
  let of_role = xml
    .match_indices("<pubnote ")
    .filter(|(at, _)| {
      xml[*at..]
        .split('>')
        .next()
        .is_some_and(|tag| tag.contains(&attr))
    })
    .count();
  assert_eq!(of_role, 1, "{xml}");
  assert_element(&xml, "pubnote", &[&attr], expected);
  xml
}

/// aa's `\offprints{O. Kochukhov, \email{…}}` (the ruling's example): the note the class's label names, its remainder
/// "O. Kochukhov, " kept (the address is the author's contact, where the binding puts an `\email`).
#[test]
fn aa_offprints_note_reads_send_offprint_requests() {
  let xml = the_note(
    repro!("aa_offprints_note_reads_send_offprint_requests"),
    "preprint",
    r#"<pubnote name="Send offprint requests to: " role="preprint">O. Kochukhov, </pubnote>"#,
  );
  assert_eq!(
    creators(&xml),
    vec![vec![
      "O. Kochukhov".to_string(),
      "email=oleg@x.se".to_string(),
      "affiliation=Uppsala University".to_string(),
    ]],
    "{xml}"
  );
}

/// The same in svjour3 (sv_support's `\offprints`, which svjour's copies print alike), its tie a no-break space.
#[test]
fn svjour3_offprints_note_reads_send_offprint_requests() {
  let xml = the_note(
    repro!("svjour3_offprints_note_reads_send_offprint_requests"),
    "preprint",
    "<pubnote name=\"Send offprint requests to:\u{a0}\" role=\"preprint\">O. Kochukhov, </pubnote>",
  );
  assert_eq!(
    creators(&xml),
    vec![vec![
      "O. Kochukhov".to_string(),
      "email=oleg@x.se".to_string(),
      "affiliation=Uppsala University".to_string(),
    ]],
    "{xml}"
  );
}

/// jheppub's `\arxivnumber`: "ArXiv ePrint:", not the role's "arXiv:".
#[test]
fn jheppub_arxivnumber_note_reads_arxiv_eprint() {
  the_note(
    repro!("jheppub_arxivnumber_note_reads_arxiv_eprint"),
    "arxiv",
    "<pubnote name=\"ArXiv ePrint:\u{a0}\" role=\"arxiv\">1234.56789</pubnote>",
  );
}

/// acmart's `\acmISBN` (a conference format, where the class prints it): "ACM ISBN", not the role's "ISBN:".
#[test]
fn acmart_isbn_note_reads_acm_isbn() {
  the_note(
    repro!("acmart_isbn_note_reads_acm_isbn"),
    "isbn",
    "<pubnote name=\"ACM\u{a0}ISBN\u{a0}\" role=\"isbn\">978-1-4503-0000-0</pubnote>",
  );
}

/// iopart's `\submitto`: "Submitted to:", not the role's "Journal:".
#[test]
fn iopart_submitto_note_reads_submitted_to() {
  the_note(
    repro!("iopart_submitto_note_reads_submitted_to"),
    "journal",
    "<pubnote name=\"Submitted to:\u{a0}\" role=\"journal\">J. Phys. A: Math. Gen.</pubnote>",
  );
}

/// AASTeX's `\submitjournal`: "Submitted to", not the role's "Journal:".
#[test]
fn aastex631_submitjournal_note_reads_submitted_to() {
  the_note(
    repro!("aastex631_submitjournal_note_reads_submitted_to"),
    "journal",
    "<pubnote name=\"Submitted to\u{a0}\" role=\"journal\">ApJ</pubnote>",
  );
}

/// IEEEoj's `\doiinfo`, and ieeetj's (its own binding, the same line): "Digital Object Identifier", not "DOI:".
#[test]
fn ieeeoj_doiinfo_note_reads_digital_object_identifier() {
  let tex = repro!("ieeeoj_doiinfo_note_reads_digital_object_identifier");
  let want = "<pubnote name=\"Digital Object Identifier\u{a0}\" role=\"doi\">10.1109/OJCOMS.2024.011100</pubnote>";
  the_note(tex, "doi", want);
  the_note(
    &tex.replace("\\documentclass{IEEEoj}", "\\documentclass{ieeetj}"),
    "doi",
    want,
  );
}

/// The 2016-2019 LIPIcs classes' `\DOI`: "Digital Object Identifier", not "DOI:" (lipics's tie a plain space).
#[test]
fn lipics_doi_note_reads_digital_object_identifier() {
  the_note(
    repro!("lipics_doi_note_reads_digital_object_identifier"),
    "doi",
    r#"<pubnote name="Digital Object Identifier " role="doi">10.4230/LIPIcs.X.2018.1</pubnote>"#,
  );
}

/// astro-ph0405280's and astro-ph0608669's `\offprints{\email{…}}`, and the latter's `\title{…\thanks{}}`: no note
/// at all — the e-mail is still the contact the binding made of it, and the title keeps no mark.
#[test]
fn aa_offprints_only_email_leaves_no_note() {
  let xml = clean_both_ways(repro!("aa_offprints_only_email_leaves_no_note"));
  assert!(!xml.contains("<pubnote"), "{xml}");
  assert!(!xml.contains("<note"), "{xml}");
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert_eq!(
    creators(&xml),
    vec![
      vec!["O. Kochukhov".to_string(), "affiliation=Univ A".to_string()],
      vec![
        "B. Baker".to_string(),
        "email=oleg@x.se".to_string(),
        "affiliation=Univ B".to_string(),
      ],
    ],
    "{xml}"
  );
}

/// The kernel's `\thanks{}` in a title and in a name (2004.01582, 2401.15379): neither is a note, and neither leaves
/// a mark; the author's other `\thanks{Univ A}` stays.
#[test]
fn kernel_empty_thanks_leave_no_note() {
  let xml = clean_both_ways(repro!("kernel_empty_thanks_leave_no_note"));
  assert!(!xml.contains("<pubnote"), "{xml}");
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author">
    <personname>A. Able</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Univ A</note>
  </creator>"#,
  );
}

/// mdpi's `\corres{Correspondence: }` (1609.03883): an author-typed label alone is ink, and the note stays whole.
#[test]
fn mdpi_corres_label_only_note_is_kept() {
  let xml = clean_both_ways(repro!("mdpi_corres_label_only_note_is_kept"));
  assert_element(
    &xml,
    "note",
    &["role=\"corresponding\""],
    r#"<note role="corresponding" xml:id="id9">Correspondence: </note>"#,
  );
}
