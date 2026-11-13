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

/// Issue #779: a standard class's size and paper options set `\textwidth` as its size file does in one column
/// (size1x.clo:108-126) and `\@ptsize`; pdflatex's values. Red: 345.0pt and 0 for every row.
#[test]
fn standard_class_size_option_sets_textwidth() {
  let tex = repro!(
    "class-bindings",
    "standard_class_size_option_sets_textwidth.tex"
  );
  for (class, width, ptsize) in [
    (r"\documentclass[12pt]{article}", "390.0pt", "2"),
    (r"\documentclass{article}", "345.0pt", "0"),
    (r"\documentclass[11pt]{report}", "360.0pt", "1"),
    (r"\documentclass[12pt,a5paper]{book}", "276.0pt", "2"),
  ] {
    let xml = clean(&tex.replace(r"\documentclass[12pt]{article}", class));
    assert_element(
      &xml,
      "ltx:p",
      &[],
      &format!("<p>TEXTWIDTH = {width}, PTSIZE = {ptsize}</p>"),
    );
  }
}

/// The verbatim lines `x=…` of `xml`, in order.
fn verbatim_lines(xml: &str) -> Vec<&str> {
  xml
    .match_indices("x=")
    .map(|(at, _)| &xml[at..at + xml[at..].find(['<', '\n']).unwrap_or(0)])
    .collect()
}

/// Issue #801: under upquote, `{verbatim}` and `{verbatim*}` print ' and ` upright as `\@noligs` makes them, like
/// fancyvrb's Verbatim and `\verb` (pdflatex: all four upright). Without upquote they stay the typewriter font's curly
/// quotes, as pdflatex prints them (`\verb` keeps its ASCII, OXIDIZED_DESIGN #144). Red: the last two curly.
#[test]
fn verbatim_quotes_follow_upquote() {
  if !kpse_has("upquote.sty") || !kpse_has("fancyvrb.sty") {
    return;
  }
  let tex = repro!("package-bindings", "verbatim_quotes_follow_upquote.tex");
  let xml = clean(tex);
  assert_eq!(
    verbatim_lines(&xml),
    ["x='s' `b`", "x='s' `b`", "x='s' `b`", "x='s'\u{2423}`b`"],
    "{xml}"
  );
  let xml = clean(&tex.replace("\\usepackage{upquote}\n", ""));
  assert_eq!(
    verbatim_lines(&xml),
    [
      "x=\u{2019}s\u{2019} \u{2018}b\u{2018}",
      "x='s' `b`",
      "x=\u{2019}s\u{2019} \u{2018}b\u{2018}",
      "x=\u{2019}s\u{2019}\u{2423}\u{2018}b\u{2018}"
    ],
    "{xml}"
  );
}
