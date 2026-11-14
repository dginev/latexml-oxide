//! `\usepackage{myBiblatex}` hits the versioned-package fallback -> the native
//! `biblatex` binding, which `find_file_fallback` double-runs (probe then load).
//! The non-idempotent `\let\blx@saved@cite\cite` used to capture biblatex's OWN
//! `\cite` on the 2nd init, so `\cite -> \blx@saved@cite -> \cite` looped to
//! `Fatal:Timeout:TokenLimit`/`Recursion` (witness 2605.03965; Perl never loads
//! biblatex on this name, so no loop). The save is now `\@ifundefined`-guarded.
use crate::cluster::convert_to_xml_contrib_clean;

#[test]
fn mybiblatex_fallback_does_not_loop() {
  // Red before the fix: \cite loops to Fatal:Timeout:TokenLimit (fatal status / no result).
  // Green: converts clean (convert_to_xml_contrib_clean asserts 0 errors + non-fatal).
  let _ = convert_to_xml_contrib_clean("tests/cluster_regressions/biblatex_mybiblatex_loop.tex");
}
