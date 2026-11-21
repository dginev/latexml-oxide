//! Red/green guards for batch REGFIXE: v0.8.0 regressions — a class's `\@subjclass` placeholder taking an argument
//! handed over as amsart's stored value (edpsmath.cls), a `\usepackage{thm-autoref}` after thmtools read raw, and
//! thm-autoref's `\Autoref`, which the thmtools binding stands in for.
//! Repros `tools/perfect_kernel/repros/package-bindings/`; each pdflatex-checked.
use latexml::util::test::{assert_element, convert_files_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch78i::html_of,
};

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

/// thm-autoref.sty:60-95 `\Autoref*?{label}`, which the binding defines now that it stands in for thm-autoref: an
/// `\autoref` naming a theorem by its `Refname=` (`\<env>Autorefname`, the `autorefcap` tag), and one whose type has
/// none as `\autoref` does. Red (REGFIXE round 1): Error undefined `\Autoref`. pdflatex: 0 errors, "See theorem 1,
/// Theorem 1, Theorem 1 and lemma 1."
#[test]
fn thmtools_thm_autoref_capital_autoref() {
  let (log, xml) = convert_files_with(
    repro!("thmtools_thm_autoref_capital.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tag",
    &["role=\"autorefcap\""],
    "<tag role=\"autorefcap\">Theorem\u{a0}1<text xml:id=\"Thmthm1.2\"/></tag>",
  );
  assert!(!xml.contains("<tag role=\"autorefcap\">Lemma"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">See <ref class=\"ltx_refmacro_autoref\" labelref=\"LABEL:t1\" show=\"autoref\"/>, <ref \
     class=\"ltx_refmacro_autoref\" labelref=\"LABEL:t1\" show=\"autorefcap|autoref\"/>, <ref \
     class=\"ltx_refmacro_autoref ltx_nolink\" labelref=\"LABEL:t1\" show=\"autorefcap|autoref\"/> and <ref \
     class=\"ltx_refmacro_autoref\" labelref=\"LABEL:l1\" show=\"autorefcap|autoref\"/>.</p>",
  );
  let html = html_of(repro!("thmtools_thm_autoref_capital.tex"), &[]);
  assert!(
    html.contains(
      "See <a href=\"#Thmthm1\" title=\"Theorem 1.\" class=\"ltx_ref ltx_refmacro_autoref\"><span class=\"ltx_text \
       ltx_ref_tag\">theorem\u{a0}1</span></a>, <a href=\"#Thmthm1\" title=\"Theorem 1.\" class=\"ltx_ref \
       ltx_refmacro_autoref\"><span class=\"ltx_text ltx_ref_tag\">Theorem\u{a0}1</span></a>, <span class=\"ltx_ref \
       ltx_refmacro_autoref ltx_nolink ltx_ref_self\"><span class=\"ltx_text ltx_ref_tag\">Theorem\u{a0}1</span></span> \
       and <a href=\"#Thmlem1\" title=\"Lemma 1.\" class=\"ltx_ref ltx_refmacro_autoref\"><span class=\"ltx_text \
       ltx_ref_tag\">lemma\u{a0}1</span></a>.</p>"
    ),
    "{html}"
  );
}

/// thmtools alone loads thm-autoref (thmtools.sty:47-49), so `\Autoref` is there without a `\usepackage{thm-autoref}`
/// (1611.06061's 45, 1405.2249's `\RenewDocumentCommand\Autoref`: Error undefined `\Autoref` on MAIN935). pdflatex: 0
/// errors, "See Theorem 1 and lemma 1."
#[test]
fn thmtools_alone_defines_capital_autoref() {
  let html = html_of(repro!("thmtools_defines_autoref.tex"), &[]);
  assert!(
    html.contains(
      "See <a href=\"#Thmthm1\" title=\"Theorem 1.\" class=\"ltx_ref ltx_refmacro_autoref\"><span class=\"ltx_text \
       ltx_ref_tag\">Theorem\u{a0}1</span></a> and <a href=\"#Thmlem1\" title=\"Lemma 1.\" class=\"ltx_ref \
       ltx_refmacro_autoref\"><span class=\"ltx_text ltx_ref_tag\">lemma\u{a0}1</span></a>.</p>"
    ),
    "{html}"
  );
}
