//! Red/green guards for batch REGFIXD: v0.8.0 regressions — a paper's renamed copy of amsppt.sty read raw (its
//! self-restoring `\title` recursing over the lock to `Fatal:oom`), and amsfonts' `\newsymbol` in the body.
//! Repros `tools/perfect_kernel/repros/{amstex,package-bindings}/`; each pdftex/pdflatex-checked.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A local style that names itself `amsppt.sty` (`\def\filename{amsppt.sty}`, its `\csname amsppt.sty\endcsname`
/// load guard) is a renamed copy: the amsppt binding stands in for it, as for a style that `\input`s amsppt. Red
/// (MAIN935): read raw, `Fatal:oom`. Witnesses math0410032, math0108062, math0012097, math0207033, math0107068,
/// 1003.5377. Repro amstex/amstex_copied_style_takes_the_binding.
#[test]
fn amsppt_copy_takes_the_binding() {
  let (log, xml) = convert_files_with(
    repro!("amstex/amstex_copied_style_takes_the_binding.tex"),
    &[("amscopy.sty", repro!("amstex/amscopy.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Warning:missing_file:amscopy"), "{log}");
  assert_element(&xml, "title", &[], "<title>Hello</title>");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Body text.</p>",
  );
}

/// A document definition of a locked command without parameters whose body names the command itself redefines it on
/// use (amsppt's `\def\title{\let\savedef@\title \def\title##1\endtitle{…}\title}`): no value, refused as Perl refuses
/// it. Red (MAIN935): the outer taken as a value, the inner refused, `\title` recursed to `Fatal:oom`. A fail-safe:
/// the braced binding reads `H`, `\endtitle` is undefined (1 Error), every word kept. Repro
/// amstex/amstex_self_redefining_value_is_refused.
#[test]
fn self_redefining_value_is_refused() {
  let (log, xml) = convert_files_with(
    repro!("amstex/amstex_self_redefining_value_is_refused.tex"),
    &[("amsselfdef.sty", repro!("amstex/amsselfdef.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:\\endtitle"), "{log}");
  assert!(log.contains("Info:ignore:\\title"), "{log}");
  assert_element(&xml, "title", &[], "<title>H</title>");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">ello<ERROR class=\"undefined\" xml:id=\"p1.1.1\">\\endtitle</ERROR>Body text.</p>",
  );
}

/// amsfonts.sty:118-129 `\newsymbol` is a `\global\mathchardef`, legal in the body, where `\DeclareMathSymbol` is
/// `\@onlypreamble`: the symbol is declared, globally. Red (MAIN935): "Can be used only in preamble", then `\rtimes`
/// undefined. Witnesses math0008232, 2407.16937. Repro package-bindings/amsfonts_newsymbol_in_the_body.
#[test]
fn newsymbol_in_the_body() {
  let (log, xml) = convert_with(
    repro!("package-bindings/amsfonts_newsymbol_in_the_body.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(
    log.matches("Obsolete command \\newsymbol").count(),
    2,
    "{log}"
  );
  for (id, name, glyph, a, b) in [
    ("p1.m1.1.1", "rtimes", "⋊", "a", "b"),
    ("p1.m2.1.1", "ltimes", "⋉", "c", "d"),
  ] {
    assert_element(
      &xml,
      "XMApp",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<XMApp xml:id=\"{id}\"><XMTok name=\"{name}\" role=\"BINOP\">{glyph}</XMTok><XMTok font=\"italic\" \
         role=\"UNKNOWN\">{a}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">{b}</XMTok></XMApp>"
      ),
    );
  }
}
