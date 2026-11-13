//! Guards for the bindings batch of 2026-11-13 (issues #772, #760, #801, #787, #773, #779): one test per issue, each a
//! binding's own mechanism, its repro converted once in-process and checked whole, with exact Error/Warning counts.

use latexml::util::test::{assert_element, convert_with, error_count, kpse_has, warning_count};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex` (no preload) and asserts it converts with no Error and no Warning; returns the XML.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// Issue #772: `\import@path` is import.sty's own macro, set by `\import`/`\subimport` and restored after the input
/// (import.sty:83, :93), empty outside (:142). Red: `Error:undefined:\import@path`.
#[test]
fn import_path_is_the_package_macro() {
  if !kpse_has("import.sty") {
    return;
  }
  let xml = clean(repro!(
    "package-bindings",
    "import_path_is_the_package_macro.tex"
  ));
  assert_element(&xml, "ltx:p", &[], "<p>[][a/][a/b/]\n[a/]\n[]</p>");
}
