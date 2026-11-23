//! Red/green guards for batch LTXOWN: the format's own modules (babel.sty, babel.def, expl3.sty, xparse.sty,
//! l3keys2e.sty) are read from the format's TeX tree, not from a paper's copy, while the format holds them
//! (OXIDIZED_DESIGN_DIVERGENCES #1142; DG ruling 2026-11-23). Each skipped copy is one counted `shipped_copy` Warning; a
//! copy byte-identical to the tree's is read silently. Repros under `tools/perfect_kernel/repros/loader/format_modules/`,
//! pdflatex-checked: TL 2025 fails on both copies, and prints the asserted text without them.
use latexml::util::test::{
  assert_element, convert_files_with, dump_available, kpse_has, kpse_text,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/loader/format_modules/",
      $path
    ))
  };
}

/// The `Warning:shipped_copy:<file>` lines of `log`, their objects in order.
fn skipped_copies(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|line| line.strip_prefix("Warning:shipped_copy:"))
    .filter_map(|rest| rest.split_whitespace().next())
    .collect()
}

/// Is the format's babel the TL 2025 one, whose switch.def is a proxy for babel.def (the loop a 3.9 copy runs into)?
/// Prints the visible SKIP line when not.
fn babel_in_the_format(test: &str) -> bool {
  let proxy = kpse_text("switch.def").is_some_and(|text| text.contains("\\input babel.def"));
  if !dump_available() || !proxy {
    eprintln!("SKIP {test}: no latex dump, or this TeX Live's switch.def is not babel.def's proxy");
    return false;
  }
  true
}

/// (A) babel 3.9l's babel.sty and babel.def shipped (1511.02101; 1408.3933, 1410.2241, 1511.05750, 1602.08358,
/// 1601.02763): their `\ifx\bbl@version\bbl@tempa\else\input switch.def` reads TL's switch.def, which inputs babel.def,
/// the copy again. Red (INT081C): Fatal:Stomach:Recursion, "Package loading depth exceeded 500", no document. Green: the
/// tree's babel.sty, which reads no babel.def, one Warning, the copy's marker undefined.
#[test]
fn a_shipped_babel_3_9l_is_not_read() {
  if !babel_in_the_format("a_shipped_babel_3_9l_is_not_read") {
    return;
  }
  let (log, xml) = convert_files_with(
    repro!("babel_copy/babel_3_9l_copy_1511.02101.tex"),
    &[
      ("babel.sty", repro!("babel_copy/babel.sty")),
      ("babel.def", repro!("babel_copy/babel.def")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(skipped_copies(&log), ["babel.sty"], "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Hello world.</p>",
  );
}

/// (B) expl3 2012's expl3.sty (its l3bootstrap.sty:201-241 file-stack wrap and status reset) under a 2012 xparse.sty and
/// l3keys2e.sty (1211.3724; 1207.4642, 1211.6751, 1304.5586). Red (INT081C): "Missing number", 2 Errors and
/// Fatal:Timeout:Recursion. Green: the tree's three files, one Warning each in load order, `\NewDocumentCommand` the
/// format's, no copy's marker defined.
#[test]
fn shipped_expl3_xparse_l3keys2e_are_not_read() {
  if !dump_available() || !kpse_has("expl3.ltx") {
    eprintln!(
      "SKIP shipped_expl3_xparse_l3keys2e_are_not_read: no latex dump, or no expl3 in this format"
    );
    return;
  }
  let (log, xml) = convert_files_with(
    repro!("expl3_copy/expl3_2012_copy_1211.3724.tex"),
    &[
      ("expl3.sty", repro!("expl3_copy/expl3.sty")),
      ("xparse.sty", repro!("expl3_copy/xparse.sty")),
      ("l3keys2e.sty", repro!("expl3_copy/l3keys2e.sty")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert_eq!(
    skipped_copies(&log),
    ["xparse.sty", "expl3.sty", "l3keys2e.sty"],
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Hello world.</p>",
  );
}

/// A shipped babel.sty byte-identical to the tree's is the tree's file: read, no Warning (the identity branch of
/// `format_module_path`; papers that ship TL's own babel.sty beside a fresh build).
#[test]
fn a_copy_identical_to_the_trees_is_read_silently() {
  if !babel_in_the_format("a_copy_identical_to_the_trees_is_read_silently") {
    return;
  }
  let tree = kpse_text("babel.sty").expect("the tree's babel.sty");
  let (log, xml) = convert_files_with(
    repro!("babel_copy/babel_3_9l_copy_1511.02101.tex"),
    &[("babel.sty", &tree)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Hello world.</p>",
  );
}
