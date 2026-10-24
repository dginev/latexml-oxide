//! Red/green guards for batch 79u (complete breakage, continued: user 2026-10-24): a document's `\newcommand` over
//! LaTeXML's deprecated letter names, a listings file name between blanks, amsxtra's superscript periods, array.sty's
//! cell group under a `>{…}` prefix, a note's own `\\`, a longtable caption's note. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\newcommand{\DUAL}{…}`, `\newcommand{\FCN}{…}`, `\newcommand{\ROLE}[1]{…}`: base_deprecated.rs defines LaTeXML's
/// deprecated letter names as kernel stubs, so the document's definitions stand, as in LaTeX, where they do not
/// exist. Red (int90a): "Ignoring redefinition", `\DUAL` = `\lx@dual` left the `{multline}` and the proposition
/// unclosed (7 errors, "After." lost). Witness 1804.03034. Repro macro-state/newcommand_over_latexml_deprecated_name.
#[test]
fn a_document_newcommand_defines_a_deprecated_latexml_name() {
  let (log, xml) = convert_files_with(
    repro!("macro-state/newcommand_over_latexml_deprecated_name.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("After."), "{xml}");
  assert!(xml.contains('♯'), "{xml}");
  assert!(xml.contains(">role<"), "{xml}");
}

/// `\lstinputlisting{⏎ listing_file/add.c⏎ }`: listings_sty.rs `listings_read_raw_file` takes the name between its
/// blanks, as TeX's file name scanner does (tex.web §526). Red (int90a): "Can't read listings file", the listing
/// empty. Witness 2511.21509. Repro listings-algorithms/listings_input_file_name_spaced.
#[test]
fn a_listings_file_name_between_blanks_is_read() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/listings_input_file_name_spaced.tex"),
    &[(
      "listing_file/add.c",
      include_str!("../../../tools/perfect_kernel/repros/listings-algorithms/listing_file/add.c"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(">add<"), "{xml}");
  assert!(xml.contains("Done."), "{xml}");
}

/// amsxtra's `\spdot`, `\spddot`, `\spdddot` in an `{align*}` row: amsxtra_sty.rs superscripts boxed text-font periods
/// (amsxtra.sty:32-34). Red (int90a): the binding's `^{\dot}` gave the accent the group's `}` ("Missing { inserted"
/// at the next `\\`, 5 errors, "Done." lost). Witness 0907.3938. Repro package-bindings/amsxtra_spdot_in_alignment.
#[test]
fn amsxtra_superscript_periods_keep_the_alignment() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/amsxtra_spdot_in_alignment.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Done."), "{xml}");
  // (three periods are LaTeXML's ellipsis ligature)
  for periods in [">.<", ">..<", ">…<"] {
    assert!(xml.contains(periods), "{periods}: {xml}");
  }
}

/// The hidden-column idiom `\newcolumntype{H}{@{}>{\lrbox0}l<{\endlrbox}}`: tex_tables.rs `text_column` sets an l/c/r
/// cell under a `>{…}` prefix in array.sty's group (`\d@llarbegin…\d@llarend`), which `\lrbox` closes and reopens in
/// its box. Red (int90a): "\endgroup Attempt to close non-boxing group", the table and "Done." lost (7 errors).
/// Witness 2106.01404. Repro kernel-alignment/array_lrbox_hidden_column.
#[test]
fn an_lrbox_hidden_column_hides_its_cells() {
  let (log, xml) = convert_files_with(
    repro!("kernel-alignment/array_lrbox_hidden_column.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Done."), "{xml}");
  assert_eq!(xml.matches("<tr ").count(), 2, "{xml}");
  assert!(!xml.contains("hidden") && !xml.contains("gone"), "{xml}");
  for cell in [">a<", ">b<", ">c<", ">d<"] {
    assert!(xml.contains(cell), "{cell}: {xml}");
  }
}

/// A `\footnote` with a `\\` in a `p` cell, and one after a longtable `\caption`: sect03.rs sets a note's `\\` as
/// `\@normalcr` (`\@footnotetext`'s `\@parboxrestore`), and tex_tables.rs `caption_row_keeps_its_content` sets the
/// caption row's note after the caption's text. Red (int90a): the note's `\\` ended the row ("} Attempt to close a
/// group that switched to mode horizontal", 17 errors, "Done." lost); the caption's note was dropped with its row at 0
/// errors. Witness 2201.11330. Repro captions-floats/footnote_with_newline_in_table.
#[test]
fn a_note_in_a_table_keeps_its_line_break_and_the_caption_its_note() {
  let (log, xml) = convert_files_with(
    repro!("captions-floats/footnote_with_newline_in_table.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Done."), "{xml}");
  assert_eq!(xml.matches("<note ").count(), 2, "{xml}");
  assert!(xml.contains("first line<break/>second line"), "{xml}");
  let caption = xml
    .split("<caption>")
    .nth(1)
    .and_then(|c| c.split("</caption>").next())
    .unwrap_or_default();
  assert!(
    caption.contains("The caption<note ")
      && caption.contains("caption note<break/>its second line"),
    "{xml}"
  );
}
