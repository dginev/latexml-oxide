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

/// The separator a creator after the first carries (`before`): two em spaces, the kernel's `\and` (OD #915's
/// guards assert it whole).
const EM2: &str = "\u{2003}\u{2003}";
/// The em space a coloured box's blank content reads as (`\hspace{2em}`, `\strut\hspace{1em}`).
const EM: &str = "\u{2003}";
/// The en space of `\LaTeXe`'s logo and the no-break space after a role's label.
const EN: &str = "\u{2002}";
const NBSP: &str = "\u{a0}";

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

/// The document's frontmatter as converted, its `xml:id`s aside ([`without_ids`]): from its title to its first
/// paragraph or section.
fn frontmatter(xml: &str) -> String {
  let xml = without_ids(xml);
  let start = xml.find("<title").expect("a title");
  let end = ["<para", "<section"]
    .iter()
    .filter_map(|tag| xml[start..].find(tag))
    .min()
    .expect("a paragraph or section");
  xml[start..start + end].trim_end().to_string()
}

/// The ink test reads what a note prints, not only its text boxes: a command that writes its own
/// text — an ellipsis, a TeX logo — is ink though it shows no argument, in a creator's note (`\lx@annotate@frontmatter`)
/// and in the title's publication note (`\lx@add@frontmatter`), so each note stays whole.
#[test]
fn kernel_thanks_command_text_is_ink() {
  let xml = clean_both_ways(repro!("kernel_thanks_command_text_is_ink"));
  let latex = r#"<text class="ltx_LaTeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.1em">L<text cssstyle="font-variant:small-caps;" yoffset="0.4ex">a</text>T<text cssstyle="font-variant:small-caps;font-size:120%" yoffset="-0.2ex">e</text>X"#;
  let tex = r#"<text class="ltx_TeX_logo" cssstyle="letter-spacing:-0.2em; margin-right:0.2em">T<text cssstyle="font-variant:small-caps;font-size:120%;" yoffset="-0.2ex">e</text>X</text>"#;
  let note = r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>T</title>
  <creator role="author">
    <personname>Ann Able</personname>
    {note}…</note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Bob Baker</personname>
    {note}{latex}</text></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Cat Cole</personname>
    {note}{tex}</note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Dan Doe</personname>
    {note}{latex}{EN}2<text cssstyle="font-style:italic" yoffset="-0.3ex">ε</text></text></note>
  </creator>
  <pubnote name="Thanks:{NBSP}" role="thanks">…</pubnote>"#
    )
  );
}

/// A formula's symbol is ink (`$\vdots$`, `$\dotsc$`, `\ensuremath{\ldots}`); an empty formula
/// `$ $` prints nothing, and its note is none, as pdflatex prints a bare mark.
#[test]
fn kernel_thanks_math_token_is_ink() {
  let xml = clean_both_ways(repro!("kernel_thanks_math_token_is_ink"));
  let note = |before: &str, name: &str, tex: &str, token: &str| {
    format!(
      r#"<creator{before} role="author">
    <personname>{name}</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks"><Math mode="inline" tex="\{tex}" text="{tex}">
        <XMath>
          <XMTok name="{tex}" role="ID">{token}</XMTok>
        </XMath>
      </Math></note>
  </creator>"#
    )
  };
  let next = format!(" before=\"{EM2}\"");
  assert_eq!(
    frontmatter(&xml),
    format!(
      "<title>T</title>\n  {}\n  {}\n  {}\n  <creator{next} role=\"author\">\n    <personname>Dan Doe</personname>\n  \
       </creator>",
      note("", "Ann Able", "vdots", "⋮"),
      note(&next, "Bob Baker", "dotsc", "…"),
      note(&next, "Cat Cole", "ldots", "…"),
    )
  );
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

/// A rule with a width and a height, a framed box (empty or not) and a leader print, and their
/// notes stay; a strut (`\rule{0pt}{2ex}`), `\quad`, an empty `\mbox{}`, a phantom and the title's tie print nothing,
/// and theirs are none — material the test knows to be blank, each a sibling of the A/B's 18 drops.
#[test]
fn kernel_thanks_rule_and_frame_are_ink() {
  let xml = clean_both_ways(repro!("kernel_thanks_rule_and_frame_are_ink"));
  let note = r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">"#;
  let bare = |name: &str| {
    format!(
      "<creator before=\"{EM2}\" role=\"author\">\n    <personname>{name}</personname>\n  </creator>"
    )
  };
  assert_eq!(
    frontmatter(&xml),
    format!(
      r##"<title>T</title>
  <creator role="author">
    <personname>Ann Able</personname>
    {note}<rule height="4.3pt" width="10.0pt"/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Bob Baker</personname>
    {note}<text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle"/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Cat Cole</personname>
    {note}<text align="center" cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle" width="10.0pt"/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Dan Doe</personname>
    {note}<text class="ltx_leader"><rule height="1px" width="100%"/></text></note>
  </creator>
  {}
  {}
  {}
  {}"##,
      bare("Eve Ell"),
      bare("Fay Fox"),
      bare("Gil Gray"),
      bare("Ida Ivy"),
    )
  );
}

/// An undefined command is no known blank item: the note stays, holding its error marker, and the
/// conversion reports the one Error (the repro fails pdflatex too: it guards the error path only).
#[test]
fn kernel_thanks_undefined_command_is_ink() {
  let tex = repro!("kernel_thanks_undefined_command_is_ink");
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("\\undefinedcs"), "{log}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(eager, streamed);
  assert_eq!(
    frontmatter(&eager),
    r#"<title>T</title>
  <creator role="author">
    <personname>Ann Able</personname>
    <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks"><ERROR class="undefined">\undefinedcs</ERROR></note>
  </creator>"#
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
  assert_eq!(tuples(repro!("kernel_thanks_rule_and_frame_are_ink")), [
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
    "D note X | note:adjacent ruling4:inkless",
  ]);
}

/// A constructor is a transparent wrapper only when its whole construction is a template printing only its arguments
/// and it has an argument (`util_text.rs` `prints_only_arguments`, OD #916): revsymb's `\openone` (`"1"` in the
/// blackboard font, no argument) is a formula's symbol, its note kept; `\emph{}`'s hook closing its element cannot be
/// read, so its blank note is kept too (in doubt, ink); `$\mathbb{}$` is a font switch over nothing, its note none. The
/// observer reads each note so.
#[test]
fn kernel_thanks_constructor_text_is_ink() {
  let tex = repro!("kernel_thanks_constructor_text_is_ink");
  let xml = clean_both_ways(tex);
  let note = r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>T</title>
  <creator role="author">
    <personname>Ann Able</personname>
    {note}<Math mode="inline" tex="\openone" text="1">
        <XMath>
          <XMTok font="blackboard" role="UNKNOWN">1</XMTok>
        </XMath>
      </Math></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Bob Baker</personname>
    {note}<emph font="italic"/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Cat Cole</personname>
  </creator>"#
    )
  );
  assert_eq!(tuples(&resolved(tex, Some("ar5iv.sty"))), [
    "C Ann Able [] | line:names",
    "  n  | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  n  | note:adjacent",
    "C Cat Cole [] | line:names sep:column",
    "D note  | note:adjacent ruling4:inkless",
  ]);
}

/// A background colour is ink, whatever it is behind: `\colorbox{c}{…}` is
/// `\hbox{\pagecolor{c}…}`, and pdflatex prints the coloured block over nothing, a space, a phantom or a strut, inside
/// `\mbox` too, so each note stays; `\textcolor{red}{}` prints nothing, its note none. XML and observer alike.
#[test]
fn kernel_thanks_colorbox_is_ink() {
  let tex = repro!("kernel_thanks_colorbox_is_ink");
  let xml = clean_both_ways(tex);
  let note = r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r##"<title>T</title>
  <creator role="author">
    <personname>Ann Able</personname>
    {note}<text/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Bob Baker</personname>
    {note}<text backgroundcolor="#FFFF00">{EM}</text></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Cat Cole</personname>
    {note}<text backgroundcolor="#FFFF00" class="ltx_phantom">XX</text></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Dan Doe</personname>
    {note}<text backgroundcolor="#D9D9D9">{EM}</text></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Eve Ell</personname>
    {note}<text/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Fay Fox</personname>
  </creator>"##
    )
  );
  assert_eq!(tuples(&resolved(tex, Some("ar5iv.sty"))), [
    "C Ann Able [] | line:names",
    "  n  | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  n  | note:adjacent",
    "C Cat Cole [] | line:names sep:column",
    "  n XX | note:adjacent",
    "C Dan Doe [] | line:names sep:column",
    "  n  | note:adjacent",
    "C Eve Ell [] | line:names sep:column",
    "  n  | note:adjacent",
    "C Fay Fox [] | line:names sep:column",
    "D note  | note:adjacent ruling4:inkless",
  ]);
}

/// The XML and the observer read a note on the same input: the author block's empty-mark filter
/// (`drop_empty_marks`) leaves a note's text as written, so `\thanks{\textsuperscript{}}` keeps its superscript — an
/// element of its own, in doubt ink — and both keep the note; the empty mark beside Bob's name is still no mark.
#[test]
fn kernel_thanks_empty_superscript_reads_alike() {
  let tex = repro!("kernel_thanks_empty_superscript_reads_alike");
  let xml = clean_both_ways(tex);
  let note = r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>T</title>
  <creator role="author">
    <personname>Ann Able</personname>
    {note}<sup/></note>
  </creator>
  <creator before="{EM2}" role="author">
    <personname>Bob Baker</personname>
    {note}B</note>
  </creator>"#
    )
  );
  assert_eq!(tuples(&resolved(tex, Some("ar5iv.sty"))), [
    "C Ann Able [] | line:names",
    "  n  | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  n B | note:adjacent",
  ]);
}
