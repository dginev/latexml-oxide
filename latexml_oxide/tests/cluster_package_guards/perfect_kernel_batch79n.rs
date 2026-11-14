//! Red/green guards for batch 79n (complete breakage, continued: user 2026-10-24): a file name's surrounding blanks are
//! no part of it, so `\input{ x.tex}`, `\input{dir/x }` and `\include{ dir/x}` read the file as pdflatex does. Each
//! guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\input{ data_table.tex}` and `\input{sub/enc }` read `data_table.tex` and `sub/enc.tex`, and `\include{ sub/enc}`
/// reads it again (content.rs `input`, sect13.rs `\IfFileExists`/`\InputIfFileExists`, sect11.rs `\include`: the name
/// trimmed, as tex.web §526 `scan_file_name` skips leading blanks and a blank ends the name); both labels resolve. Red
/// (int86b): "Can't find TeX file  data_table.tex" and "sub/enc ", the `\include` silently dropped ("No file"), the
/// references dangling. Witnesses 2109.01356, 2509.24347, 1804.08035, 2410.05458, 2407.11677, 2002.10466. Repro
/// loader/input_name_surrounding_blanks.
#[test]
fn a_file_names_surrounding_blanks_are_no_part_of_it() {
  let (log, xml) = convert_files_with(
    repro!("loader/input_name_surrounding_blanks.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("Inner A text.").count(), 1, "{xml}");
  assert_eq!(xml.matches("Inner B text.").count(), 2, "{xml}");
  assert!(xml.contains(r#"labels="LABEL:a LABEL:b""#), "{xml}");
  assert!(xml.contains(r#"<ref labelref="LABEL:a"/>"#), "{xml}");
}
