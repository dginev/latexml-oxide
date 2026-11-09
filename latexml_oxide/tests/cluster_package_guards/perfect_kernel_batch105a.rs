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

/// A code's line (`98.80.Cq`: one unclaimed word, `cq`) and a date's (`3 March 2021`) below the floor are kept as the
/// net's note lines, one Warning each; the address beside them is the author's contact. An MSC code (`05C12`), whose
/// letters and digits are no content words, is kept the same way. Distinct path: the shapes kept.
#[test]
fn a_code_or_date_line_below_the_floor_is_a_note_line() {
  let tex = repro!("net_shaped_store_below_the_floor_is_kept");
  let msc = tex.replace("\\Pacs{98.80.Cq}", "\\Pacs{05C12}");
  assert_ne!(msc, tex);
}

/// An undefined font switch inside the store's bold group leaves its `ltx:ERROR` in the line's `ltx:text`: its text is
/// no printed word, so the Warning names "lunarfund" alone (2601.14329's `{\songti\zihao{-4}#2}`). The fixture errs on
/// purpose (pdflatex too). Distinct path: `text_without_errors`.
#[test]
fn an_error_at_depth_in_a_note_line_is_no_word() {
  let tex = repro!("net_line_words_skip_an_error_at_depth");
  for streaming in [false, true] {
    let (log, _) = cli_conversion(tex, streaming);
    assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
    assert_eq!(log.matches("Error:undefined:\\songti").count(), 1, "{log}");
  }
}
