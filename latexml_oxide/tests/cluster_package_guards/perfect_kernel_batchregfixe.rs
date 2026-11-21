//! Red/green guards for batch REGFIXE: v0.8.0 regressions — a class's `\@subjclass` placeholder taking an argument
//! handed over as amsart's stored value (edpsmath.cls), and a `\usepackage{thm-autoref}` after thmtools read raw.
//! Repros `tools/perfect_kernel/repros/package-bindings/`; each pdflatex-checked.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $path
    ))
  };
}

/// edpsmath.cls:326 `\def\@subjclass#1{\@MSSG@SBJCTCLSS}` after `\LoadClass{amsart}` is a placeholder, not a stored
/// value (amsart.cls:553-564): `\maketitle` hands over neither it nor a `\@keywords#1` one, where `{\@subjclass}` read
/// `}` as its argument and swallowed the title page into `<classification>` (2004.12879, 1110.0287, 2603.24523,
/// 1501.03645). pdflatex: 0 errors, "T A Body.", no classification.
#[test]
fn a_subjclass_placeholder_taking_an_argument_is_no_stored_value() {
  let (log, xml) = convert_files_with(
    repro!("amsart_subjclass_placeholder.tex"),
    &[(
      "subjclass_placeholder.cls",
      repro!("subjclass_placeholder.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    !xml.contains("<classification") && !xml.contains("<keywords"),
    "{xml}"
  );
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>A</personname></creator>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Body.</p>",
  );
}

/// thmtools.sty:47-49 loads thm-kv, thm-autoref and thm-listof, so a later `\usepackage` of them is a no-op; read raw,
/// thm-autoref.sty:44-45 extended thm-patch's `\thmt@newtheorem@predefinition`, which the binding never defines
/// (2308.15546, 2206.04091, 2009.07990). pdflatex: 0 errors, "List of Theorems … Theorem 1 (Foo). True. See Theorem 1."
#[test]
fn thmtools_loads_thm_autoref() {
  let (log, xml) = convert_files_with(
    repro!("thmtools_loads_thm_autoref.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "TOC",
    &["lists=\"thm\""],
    "<TOC lists=\"thm\" scope=\"global\"><title>List of Theorems</title></TOC>",
  );
  assert_element(
    &xml,
    "tag",
    &["role=\"autoref\""],
    "<tag role=\"autoref\">Theorem\u{a0}1<text xml:id=\"Thmtheorem1.1\"/></tag>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">See <ref class=\"ltx_refmacro_autoref\" labelref=\"LABEL:t\" show=\"autoref\"/>.</p>",
  );
}
