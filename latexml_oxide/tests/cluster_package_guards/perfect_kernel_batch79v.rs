//! Red/green guards for batch 79v (complete breakage, continued: user 2026-10-24): a content `\input` runs latex.ltx's
//! file hooks, so currfile knows the file it is in. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\input{currfiledir/part}` under currfile: content.rs `input_content_with_file_hooks` sets `\CurrentFile` and
/// `\CurrentFilePath` and runs `file/before` ahead of the file's tokens and `file/after` after them, as latex.ltx's
/// `\InputIfFileExists` does (:19646-19673), so filehook's `file/before` code pushes currfile's names
/// (filehook-2020.sty:280-285): `\currfiledir` is the part's directory inside it, `\input{\currfiledir other}` finds the
/// sibling, and the main file's names are back after it. Red (int91b): "Can't find TeX file other", the part's
/// `\currfiledir` empty and `\currfilename` the main file's. Witness 2110.08325. Repro
/// loader/currfile_dir_inside_input.
#[test]
fn currfile_knows_the_input_file_it_is_in() {
  let (log, xml) = convert_files_with(
    repro!("loader/currfile_dir_inside_input.tex"),
    &[
      (
        "currfiledir/part.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/currfiledir/part.tex"),
      ),
      (
        "currfiledir/other.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/currfiledir/other.tex"),
      ),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("Part says dir [currfiledir/] name [part.tex]."),
    "{xml}"
  );
  assert!(
    xml.contains("Other text from [currfiledir/other.tex]."),
    "{xml}"
  );
  assert!(xml.contains("] dir []."), "{xml}");
}
