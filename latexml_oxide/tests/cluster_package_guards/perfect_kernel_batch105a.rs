//! Red/green guards for batch 105a: the unclaimed-text net's lines below its floor (FIDELITY_CHARTER §4 "No printed
//! frontmatter word is lost", every loss counted; OXIDIZED_DESIGN_DIVERGENCES #953). A line of a store only a raw
//! class's title page prints, with fewer than 3 content words no frontmatter field holds, was removed with no Warning
//! (base_utilities/frontmatter/insert.rs `keep_unclaimed_net_notes`). Now a line of e-mail addresses alone is the
//! e-mail contacts of the note's creator, with no Warning (aaskaiid.sty `\email@toks`: 2606.27706, 2607.25721); a
//! code's or a date's line is kept as the net's note line, with its Warning (`code_or_date_word`: europhys `\Pacs`,
//! ourlematema `\MSC`, tADR2e `\received`); any other short line is not written but counted, one
//! `Warning:frontmatter:unclaimed` naming the store and its words (2103.14934). A store at its class default stays no
//! candidate. Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the
//! binary, eager and `--streaming` (`perfect_kernel_batch98c::converted`), and asserts the frontmatter whole.
use super::{perfect_kernel_batch89a::cli_conversion, perfect_kernel_batch98c::converted};

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
/// words unclaimed) is the sole author's e-mail contact, no note and no Warning. Distinct path: the contact writer.
#[test]
fn an_address_line_below_the_floor_is_an_email_contact() {
  let tex = repro!("net_shaped_store_below_the_floor_is_kept")
    .replace("\\Pacs{98.80.Cq}\n", "")
    .replace("\\recd{3 March 2021}\n", "");
  assert_eq!(
    converted(&tex, 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann.able@lunar.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A code's line (`98.80.Cq`: one unclaimed word, `cq`) and a date's (`3 March 2021`) below the floor are kept as the
/// net's note lines, one Warning each; the address beside them is the author's contact. An MSC code (`05C12`), whose
/// letters and digits are no content words, is kept the same way. Distinct path: the shapes kept.
#[test]
fn a_code_or_date_line_below_the_floor_is_a_note_line() {
  let tex = repro!("net_shaped_store_below_the_floor_is_kept");
  let msc = tex.replace("\\Pacs{98.80.Cq}", "\\Pacs{05C12}");
  assert_ne!(msc, tex);
  assert!(converted(&msc, 2).contains(r#"<note role="note" xml:id="id1">05C12</note>"#));
  assert_eq!(
    converted(tex, 2),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann.able@lunar.org</contact> <note role="note" xml:id="id1">98.80.Cq</note> <note role="note" xml:id="id2">3 March 2021</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A one-word store of no shape (`\grant{Lunarfund}`) is not written but counted: one Warning naming the store and the
/// word. With the store unset, the class's placeholder (`\@recd`'s "Received 00 Month 20XX") is no candidate: the same
/// frontmatter, no Warning. Distinct path: the counted loss.
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
  let unset = tex.replace("\\grant{Lunarfund}\n", "");
  assert_ne!(unset, tex);
  assert_eq!(converted(&unset, 0), plain);
}
