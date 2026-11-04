//! Red/green guards for batch 105a: the unclaimed-text net's lines below its floor (FIDELITY_CHARTER §4 "No printed
//! frontmatter word is lost", every loss counted; OXIDIZED_DESIGN_DIVERGENCES #953). A line of a store only a raw
//! class's title page prints, with fewer than 3 content words no frontmatter field holds, was removed with no Warning
//! (base_utilities/frontmatter/insert.rs `keep_unclaimed_net_notes`). Now a line of e-mail addresses alone is the
//! e-mail contacts of the note's creator, with no Warning, whether or not the addresses' words are used elsewhere
//! (aaskaiid.sty `\email@toks`: 2606.27706, 2607.25721); a code's or a date's line, or one a dateline or MSC/PACS label
//! leads, is kept as the net's note line, with its Warning (`code_or_date_word`, `leading_label`: europhys `\Pacs`,
//! ourlematema `\MSC`, tADR2e `\received`); any other short line is not written but counted, one
//! `Warning:frontmatter:unclaimed` naming the store and its words in printed order (2103.14934). A line's words are any
//! script's letter runs, each Han ideograph one (2206.01007, 2601.14329), and no `ltx:ERROR`'s text at any depth
//! (2601.14329). A store at its class default stays no candidate, pending the placeholder ruling (the RED repro
//! `net_class_default_placeholder_is_counted_red`). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`
//! (`perfect_kernel_batch98c::converted`), and asserts the frontmatter whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batch98c::converted,
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

/// A line of addresses alone below the floor (`\the\email@toks`'s `ann.able@lunar.org`: "ann" the author's name, two
/// words unclaimed) is the sole author's e-mail contact, no note and no Warning. With every word of the address used
/// elsewhere (the title "Lunar Org Notes"), the address no kept text holds is the contact all the same (review rev105a
/// S2: it went with no Warning). Distinct path: the contact writer.
#[test]
fn an_address_line_below_the_floor_is_an_email_contact() {
  let tex = repro!("net_shaped_store_below_the_floor_is_kept")
    .replace("\\Pacs{98.80.Cq}\n", "")
    .replace("\\recd{3 March 2021}\n", "");
  let contact = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann.able@lunar.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(
    converted(&tex, 0),
    format!("<title>A Title</title> {contact}")
  );
  let used = tex.replace("\\title{A Title}", "\\title{Lunar Org Notes}");
  assert_ne!(used, tex);
  assert_eq!(
    converted(&used, 0),
    format!("<title>Lunar Org Notes</title> {contact}")
  );
}

/// A code's line (`98.80.Cq`: one unclaimed word, `cq`) and a date's (`3 March 2021`) below the floor are kept as the
/// net's note lines, one Warning each; the address beside them is the author's contact. An MSC code (`05C12`), whose
/// letters and digits are no content words, is kept the same way. Distinct path: the shapes kept.
#[test]
fn a_code_or_date_line_below_the_floor_is_a_note_line() {
  let tex = repro!("net_shaped_store_below_the_floor_is_kept");
  let frontmatter = |code: &str| {
    format!(
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann.able@lunar.org</contact> <note role="note" xml:id="id1">{code}</note> <note role="note" xml:id="id2">3 March 2021</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
    )
  };
  let msc = tex.replace("\\Pacs{98.80.Cq}", "\\Pacs{05C12}");
  assert_ne!(msc, tex);
  assert_eq!(converted(&msc, 2), frontmatter("05C12"));
  assert_eq!(converted(tex, 2), frontmatter("98.80.Cq"));
}

/// A line a label leads below the floor — a dateline's before a year ("Received: 2021", "Accepted 2021"), a
/// classification's before a code ("MSC 2020: 05C12") — is kept whole as the net's note line, one Warning each; a
/// leading word of no label list ("ICML 2021") is counted and not kept, its words in printed order. Distinct path:
/// `leading_label`.
#[test]
fn a_labelled_code_or_date_line_below_the_floor_is_a_note_line() {
  let tex = repro!("net_labelled_code_or_date_line_is_kept");
  assert_eq!(
    converted(tex, 4),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <note role="note" xml:id="id1">Received: 2021</note> <note role="note" xml:id="id2">Accepted 2021</note> <note role="note" xml:id="id3">MSC 2020: 05C12</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  let (log, _) = cli_conversion(tex, false);
  assert!(
    log.contains(
      "The title page prints \\@xd with 2 word(s) no frontmatter field holds, too few to keep as a note: icml 2021"
    ),
    "{log}"
  );
}

/// Lines in scripts other than Latin: a run of ideographs (`北京师范大学天文系`, each ideograph a word) clears the floor
/// and is the net's note line; a Cyrillic word (`Москва`, active under `[T2A]{fontenc}`, a store with text all the
/// same) is counted and not kept. Both went with no Warning. Distinct path: `net_content_words` beyond ASCII and
/// `holds_text`'s active letters.
#[test]
fn a_line_in_another_script_is_counted() {
  let tex = repro!("net_non_latin_store_lines_are_counted");
  assert_eq!(
    converted(tex, 2),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <note role="note" xml:id="id1">北京师范大学天文系</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  let (log, _) = cli_conversion(tex, false);
  assert!(
    log.contains(
      "The title page prints \\@xb with 1 word(s) no frontmatter field holds, too few to keep as a note: москва"
    ),
    "{log}"
  );
}

/// An undefined font switch inside the store's bold group leaves its `ltx:ERROR` in the line's `ltx:text`: its text is
/// no printed word, so the Warning names "lunarfund" alone (2601.14329's `{\songti\zihao{-4}#2}`). The fixture errs on
/// purpose (pdflatex too). Distinct path: `text_without_errors`.
#[test]
fn an_error_at_depth_in_a_note_line_is_no_word() {
  let tex = repro!("net_line_words_skip_an_error_at_depth");
  let mut xmls = Vec::new();
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
    assert_eq!(log.matches("Error:undefined:\\songti").count(), 1, "{log}");
    assert!(
      log.contains(
        "The title page prints \\@xa with 1 word(s) no frontmatter field holds, too few to keep as a note: lunarfund\n"
      ),
      "{log}"
    );
    xmls.push(xml.split_whitespace().collect::<Vec<_>>().join(" "));
  }
  assert_eq!(xmls[0], xmls[1]);
  assert!(
    xmls[0].contains(
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1">"#
    ),
    "{}",
    xmls[0]
  );
}

/// A one-word store of no shape (`\grant{Lunarfund}`) is not written but counted: one Warning naming the store and the
/// word. (The class's placeholder beside it, `\@recd`'s "Received 00 Month 20XX", is no candidate: a known gap pending
/// the placeholder ruling, the RED repro `net_class_default_placeholder_is_counted_red`.) Distinct path: the counted
/// loss.
#[test]
fn a_short_unshaped_store_is_counted_and_not_kept() {
  let tex = repro!("net_short_store_is_counted_not_kept");
  let plain = r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(converted(tex, 1), plain);
  let (log, _) = cli_conversion(tex, false);
  assert!(
    log.contains(
      "The title page prints \\@grant with 1 word(s) no frontmatter field holds, too few to keep as a note: lunarfund"
    ),
    "{log}"
  );
}
