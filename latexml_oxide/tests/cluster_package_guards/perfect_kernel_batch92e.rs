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
//! label alone too (mdpi's `\corres{Correspondence: }`, 1609.03883): the no-change row. A note is inkless only when
//! it provably prints nothing — every item in it known blank (white space, glue and kerns, breaks, penalties,
//! phantoms and struts, an unframed box, a constructor whose literal template prints only its arguments, over blank
//! arguments: `util_text.rs` `leaves_no_ink`) —; any doubt is ink: a command's own text (`\thanks{\dots}`,
//! `\thanks{\LaTeX}`, aa's `\offprints{\LaTeX}`, revsymb's `\openone`), a formula's symbol, an element of its own, a
//! construction hook, a rule, a frame, a leader, a background colour, an undefined command's marker. The frontmatter
//! observer reads each note by the same test on the same input.
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batchfms2a2::{resolved, tuples},
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

/// mdpi's `\corres{Correspondence: }` (1609.03883): an author-typed label alone is ink, and the note stays whole. A
/// document's note holding only a space or a tie, `\datepublished{ }` and `\externaleditor{~}` (2609.00671;
/// 1508.00510's lipics `\Copyright{\@empty}` the same shape), is none: the document's notes are exactly the others, in
/// order.
#[test]
fn mdpi_corres_label_only_note_is_kept() {
  let xml = without_ids(&clean_both_ways(repro!(
    "mdpi_corres_label_only_note_is_kept"
  )));
  let notes: Vec<&str> = xml
    .match_indices("<note ")
    .map(|(at, _)| {
      &xml[at..at + xml[at..].find("</note>").expect("a closed note") + "</note>".len()]
    })
    .collect();
  assert_eq!(notes, [
    r#"<note role="volume">1</note>"#,
    r#"<note role="issue">1</note>"#,
    r#"<note role="articlenumber">0</note>"#,
    r#"<note role="year">2026</note>"#,
    r#"<note role="copyright-year">2026</note>"#,
    r#"<note role="received">1 June 2026</note>"#,
    r#"<note role="accepted">1 July 2026</note>"#,
    r#"<note role="address">Univ A</note>"#,
    r#"<note role="corresponding">Correspondence: </note>"#,
  ]);
}

/// `xml` without its `xml:id` attributes, which number every element and so move when a note before them goes.
fn without_ids(xml: &str) -> String {
  let mut out = String::with_capacity(xml.len());
  let mut rest = xml;
  while let Some(at) = rest.find(" xml:id=\"") {
    out.push_str(&rest[..at]);
    let value = at + " xml:id=\"".len();
    rest = &rest[value + rest[value..].find('"').expect("closed xml:id") + 1..];
  }
  out.push_str(rest);
  out
}

/// A class's publication note by the same test: aa's `\offprints{\LaTeX}` is the note its
/// class labels, holding the logo.
#[test]
fn aa_offprints_command_text_is_ink() {
  let xml = without_ids(&clean_both_ways(repro!("aa_offprints_command_text_is_ink")));
  assert_eq!(xml.matches("<pubnote").count(), 1, "{xml}");
  assert_element(
    &xml,
    "pubnote",
    &["role=\"preprint\""],
    r#"<pubnote name="Send offprint requests to: " role="preprint"><text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X</text></pubnote>"#,
  );
}

/// The frontmatter observer reads a note by the XML's test (`Event::inkless`, OD #837, #916): an
/// empty `\thanks{}` is no note of its author's but one the ruling dropped (`ruling4:inkless`), and of the
/// rule-and-frame repro's eight notes the four that print stay, the four blank ones dropped.
#[test]
fn observer_reads_a_note_by_the_ink_test() {
  let tuples = |tex: &str| tuples(&resolved(tex, Some("ar5iv.sty")));
  assert_eq!(tuples(repro!("kernel_empty_thanks_leave_no_note")), [
    "C A. Able [] | line:names",
    "  n Univ A | note:adjacent",
    "D note  | note:adjacent ruling4:inkless",
  ]);
  // (its four notes that print no word are dropped by the emit and counted, one Warning: UNC; the reading keeps them)
  let (log, _, json) = latexml::util::test::convert_with_setup_then(
    repro!("kernel_thanks_rule_and_frame_are_ink_red"),
    Some("ar5iv.sty"),
    || {},
    |_| latexml_engine::base_utilities::fm_reading_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  let json = super::perfect_kernel_batchfms2a1::parsed(&json);
  assert_eq!(
    super::perfect_kernel_batchfms2a2::tuples(&json["resolved"]),
    [
      "C Ann Able [] | line:names",
      "  n  | note:adjacent",
      "C Bob Baker [] | line:names sep:column",
      "  n  | note:adjacent",
      "C Cat Cole [] | line:names sep:column",
      "  n  | note:adjacent",
      "C Dan Doe [] | line:names sep:column",
      "  n  | note:adjacent",
      "C Eve Ell [] | line:names sep:column",
      "C Fay Fox [] | line:names sep:column",
      "C Gil Gray [] | line:names sep:column",
      "C Ida Ivy [] | line:names sep:column",
      "D note  | note:adjacent ruling4:inkless",
      "D note  | note:adjacent ruling4:inkless",
      "D note  | note:adjacent ruling4:inkless",
      // (a `\phantom`'s letters are no ink of the note's text: invisible ink, OD #928 — 94a r2's B-2)
      "D note  | note:adjacent ruling4:inkless",
    ]
  );
}
