//! Red/green guards for ALGO397 (two run-397 Error clusters): a paper's own algorithm2e.sty older than
//! `\algocf@capseparator` captions as that version prints (cs0008002 and ~200 other shippers of 1996-2005 copies), and
//! forest's `\useforestlibrary` loads the library file (2606.20837, prooftrees 0.9.2's `ext.tagging`). Each guard
//! converts its pdflatex-checked repro under `tools/perfect_kernel/repros/` and asserts exact Error and Warning counts
//! and whole elements.
use latexml::util::test::{Requirements, convert_files_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The `<caption>` of `tex` converted beside the 1995/12/08 algorithm2e.sty stand-in, after 0 errors and 0 warnings.
#[track_caller]
fn algorithm2e_1995_caption(tex: &str) -> String {
  let (log, xml) = convert_files_with(
    tex,
    &[(
      "algorithm2e.sty",
      repro!("listings-algorithms/algorithm2e_1995/algorithm2e.sty"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml_element(&xml, "caption", &[]).unwrap_or_else(|| panic!("no caption: {xml}"))
}

/// A shipped algorithm2e.sty release 1.16 (`[1995/12/08]`) captions through float.sty (`\newfloat{algocf}` in
/// `\algostyle`): `[ruled]` prints "Algorithm 1 Alg one." (float.sty `\floatc@ruled`, no colon; pdflatex). The
/// `\algostyle` branch of algorithm2e_sty.rs's defaults, the name `\algorithmname`. Red (main 917): 3 Errors, undefined
/// `\algorithmcfname`, `\algocf@typo`, `\algocf@capseparator`. Witnesses cs0008002, math0201131. Repro
/// listings-algorithms/algorithm2e_1995.
#[test]
fn algorithm2e_1995_ruled_caption_closes_with_a_space() {
  assert_eq!(
    algorithm2e_1995_caption(repro!(
      "listings-algorithms/algorithm2e_1995/algorithm2e_1995_ruled_caption.tex"
    )),
    "<caption><tag close=\" \"><text font=\"bold\" xml:id=\"algorithm1.3\">Algorithm\u{a0}1</text></tag>Alg \
     one.</caption>"
  );
}

/// The same copy `[boxed]` prints "Algorithm 1: Alg one." (float.sty `\floatc@plain`), the colon releases 2.x and the
/// 2003-2005 copies print in every style (0711.3765, math0210133; math0511378, math0501346, cs0602002): the
/// `\algocf@capseparator` default.
/// Red (main 917): the same 3 Errors. Repro listings-algorithms/algorithm2e_1995.
#[test]
fn algorithm2e_1995_boxed_caption_closes_with_its_colon() {
  assert_eq!(
    algorithm2e_1995_caption(repro!(
      "listings-algorithms/algorithm2e_1995/algorithm2e_1995_boxed_caption.tex"
    )),
    "<caption><tag close=\": \"><text font=\"bold\" xml:id=\"algorithm1.3\">Algorithm\u{a0}1</text></tag>Alg \
     one.</caption>"
  );
}

/// forest.sty:162-170 `\useforestlibrary*{lxdemo}` loads forest-lib-lxdemo.sty (`\RequirePackage`), which records the
/// library (`\ProvidesForestLibrary`, :173-176) and declares the socket the document plugs into, as
/// forest-lib-ext.tagging.sty:92 does for prooftrees 0.9.2 (2606.20837); the tree is still drawn. Red (main 917):
/// 2 Errors, the socket undeclared and `\lxdemoname` undefined. Repro graphics-tikz/forest_library_raw.
#[test]
fn forest_library_loads_the_library_file() {
  let (log, xml) = convert_files_with(
    repro!("graphics-tikz/forest_library_raw/forest_library_loads_the_library_file.tex"),
    &[(
      "forest-lib-lxdemo.sty",
      repro!("graphics-tikz/forest_library_raw/forest-lib-lxdemo.sty"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml_element(&xml, "p", &[]).as_deref(),
    Some("<p xml:id=\"p1.1\">Made by the demo library: LOADED.</p>"),
    "{xml}"
  );
  assert_eq!(xml.matches("class=\"ltx_forest_tree\"").count(), 1, "{xml}");
}

/// prooftrees 0.9.2 (TL 2026; prooftrees.sty:44-48) loads forest's `ext.tagging` library and plugs into its socket
/// (:1115): 0 errors. Its tree is forest's bare `\forest` form, which the binding discards with one Warning
/// (forest_sty.rs `\forest`, a known stub). Red (main 917 on the distro TL): `Socket 'tagsupport/forest/setup'
/// undeclared!`. Witness 2606.20837. Skipped where forest-ext's library is absent (TL 2025).
#[test]
fn prooftrees_loads_forests_tagging_library() {
  if !Requirements::packages(&["forest-lib-ext.tagging.sty"]).met() {
    return;
  }
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\usepackage{prooftrees}
\begin{document}
\begin{prooftree}{}
  [p \lor q, just=Ass. [p] [q]]
\end{prooftree}
\end{document}",
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    xml_element(&xml, "para", &[]).as_deref(),
    Some("<para xml:id=\"p1\"><ERROR xml:id=\"p1.1\">{forest}</ERROR></para>"),
    "{xml}"
  );
}
