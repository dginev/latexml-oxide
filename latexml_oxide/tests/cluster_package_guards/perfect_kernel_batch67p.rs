//! Red/green guards for perfect-kernel batch 67p: two binding gaps that raw-loaded TeX Live classes expose.
//! - xkeyval: an explicit empty family `<>` is the empty family (header `<prefix>@`, xkeyval.tex:83-87), not the
//!   file's default `\@currname.\@currext` — sapthesis.cls `\ProcessOptionsX[sap]<>` (2503.03277, 2403.20315).
//! - theorem.sty: the binding owns the internals classes set (`\theorem@bodyfont`, `\theorem@style`,
//!   `\theorem@headerfont`) and the `\th@<style> \the\theorem@bodyfont \@thm{…}{…}` sequence a theorem begins with —
//!   cc.cls's ccthm.sty `\theorem@bodyfont{\theorempreamble}` (1507.02955). theorem.sty does not load amsthm, so the
//!   binding (which borrows amsthm) leaves it unloaded and a paper's own later `\usepackage{amsthm}` takes over
//!   (the ifacconf template, 2211.14033, 2512.06733); ntheorem's binding borrows amsthm the same way. The no-op
//!   that ends a theorem's sequence is local to the theorem, so a raw `\@thm` keeps its head (round 2).
//!
//! Repros `tools/perfect_kernel/repros/package-bindings/{xkeyval_empty_family_processoptionsx,
//! theorem_bodyfont_internals,theorem_user_style_and_headerfont,theorem_then_amsthm_takes_over,
//! theorem_raw_thm_keeps_heads,theorem_style_taken_at_newtheorem}.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_files_with, convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv (with `files` beside it), which must give no error and no warning.
fn convert_clean(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = if files.is_empty() {
    convert_with(tex, Some("ar5iv.sty"))
  } else {
    convert_files_with(tex, files, Some("ar5iv.sty"))
  };
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $name
    ))
  };
}

/// `\ProcessOptionsX[sap]<>` and `\DeclareOptionX[sap]<>{…}` use the empty family, where
/// `\define@key[sap]{}{english}` put the key, and `\key@ifundefined[sap]{}{english}` finds it there (base: the class's
/// `\DeclareOptionX*` rejected `english` and `binding=0.6cm`, `\SAPlang`/`\SAPbinding` undefined, "key: missing").
#[test]
fn xkeyval_empty_family_processoptionsx() {
  let xml = convert_clean(repro!("xkeyval_empty_family_processoptionsx.tex"), &[(
    "xkvemptyfam.cls",
    repro!("xkvemptyfam.cls"),
  )]);
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">Lang: english; side: one; binding: 0.6cm; key: found.</p>"#,
  );
}

/// `\theorembodyfont{\upshape}` sets the body upright; a class's `\theorem@bodyfont{\theorempreamble}` runs its
/// preamble on the `\@thm{thm}{Theorem}` that follows it, after the plain style's italic (base: `\theorem@bodyfont`
/// undefined, `expected:Match` for `\@thm`, `\thmkind` undefined).
#[test]
fn theorem_bodyfont_internals() {
  let xml = convert_clean(repro!("theorem_bodyfont_internals.tex"), &[]);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmdefn1.p1\""],
    r#"<para xml:id="Thmdefn1.p1"><p xml:id="Thmdefn1.p1.1">An upright body.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmthm1.p1\""],
    r#"<para xml:id="Thmthm1.p1"><p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">An italic body of kind thm, noted.</text></p></para>"#,
  );
}

/// A document's own `\th@mine` runs at the theorem's start (slanted body), and only the first `\theoremheaderfont`
/// takes effect, for every head (base: the Remark italic, both heads italic).
#[test]
fn theorem_user_style_and_headerfont() {
  let xml = convert_clean(repro!("theorem_user_style_and_headerfont.tex"), &[]);
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title class="ltx_runin"><tag><text font="smallcaps" xml:id="Thmrem1.1">Remark 1</text></tag></title>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmrem1.p1\""],
    r#"<para xml:id="Thmrem1.p1"><p xml:id="Thmrem1.p1.1"><text font="slanted" xml:id="Thmrem1.p1.1.1">A slanted body.</text></p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmthm1.p1\""],
    r#"<para xml:id="Thmthm1.p1"><p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">An italic body.</text></p></para>"#,
  );
}

/// theorem.sty's body font is separate from the style: `\theorembodyfont{\upshape}` before `\theoremstyle{break}`
/// still sets the body upright (pdflatex: cmr10 only; base: the style's slanted font).
#[test]
fn theorem_bodyfont_outlives_theoremstyle() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{theorem}
\theorembodyfont{\upshape}
\theoremstyle{break}
\newtheorem{note}{Note}
\begin{document}
\begin{note}Upright.\end{note}
\end{document}
",
    &[],
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmnote1.p1\""],
    r#"<para xml:id="Thmnote1.p1"><p xml:id="Thmnote1.p1.1">Upright.</p></para>"#,
  );
}

/// theorem.sty does not load amsthm: after it, amsthm is not loaded, and the paper's own `\usepackage{amsthm}` takes
/// over `\newtheorem` and the styles, as in the ifacconf template (2211.14033, 2512.06733; base: `\theoremstyle`
/// undefined, the style name printed, the Definition italic).
#[test]
fn theorem_then_amsthm_takes_over() {
  let xml = convert_clean(repro!("theorem_then_amsthm_takes_over.tex"), &[]);
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">Before: amsthm not loaded.</p>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmdefinition1.p1\""],
    r#"<para xml:id="Thmdefinition1.p1"><p xml:id="Thmdefinition1.p1.1">An upright body.</p></para>"#,
  );
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title class="ltx_runin"><tag><text font="bold" xml:id="Thmdefinition1.1">Definition 1</text></tag><text font="bold" xml:id="Thmdefinition1.2">.</text></title>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmtheorem1.p1\""],
    r#"<para xml:id="Thmtheorem1.p1"><p xml:id="Thmtheorem1.p1.1"><text font="italic" xml:id="Thmtheorem1.p1.1.1">An italic body.</text></p></para>"#,
  );
}

/// A document that loaded amsthm before theorem.sty keeps it loaded (pdflatex: "amsthm loaded").
#[test]
fn amsthm_before_theorem_stays_loaded() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{amsthm}
\usepackage{theorem}
\makeatletter
\@ifpackageloaded{amsthm}{\def\state{amsthm loaded}}{\def\state{amsthm not loaded}}
\makeatother
\begin{document}
State: \state.
\end{document}
",
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">State: amsthm loaded.</p>"#,
  );
}

/// A raw `\@thm{<counter>}{<name>}` after `\usepackage{theorem}` keeps the kernel's head and counter step, also after
/// a binding theorem has run (pdflatex: "Theorem 1 A theorem." / "Claim 1 Raw one." / "Claim 2 Raw two."). Round 1
/// of 67p made `\@thm` a global no-op, which lost both Claim heads and merged the two bodies into one paragraph with
/// no diagnostic; the binding's own no-op is now local to each theorem's group.
#[test]
fn theorem_raw_thm_keeps_heads() {
  let xml = convert_clean(repro!("theorem_raw_thm_keeps_heads.tex"), &[]);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmthm1.p1\""],
    r#"<para xml:id="Thmthm1.p1"><p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">A theorem.</text></p></para>"#,
  );
  for (n, body) in [(1, "one"), (2, "two")] {
    assert_element(
      &xml,
      "item",
      &[&format!("xml:id=\"S0.I{n}.ix1\"")],
      &format!(
        r#"<item xml:id="S0.I{n}.ix1"><tags><tag><text font="bold" xml:id="S0.I{n}.ix1.1">Claim {n}</text></tag></tags><para xml:id="S0.I{n}.ix1.p1"><p xml:id="S0.I{n}.ix1.p1.1"><text font="italic" xml:id="S0.I{n}.ix1.p1.1.1">Raw {body}.</text></p></para></item>"#
      ),
    );
  }
}

/// ntheorem.sty loads only ifthen (:52), so the binding borrows amsthm (`require_package_borrowed`) and
/// `\@ifpackageloaded{amsthm}` stays false, as in pdflatex ("State: amsthm not loaded."); the borrowed machinery still
/// sets the break-style theorem.
#[test]
fn ntheorem_borrows_amsthm_unloaded() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{ntheorem}
\makeatletter
\@ifpackageloaded{amsthm}{\def\state{amsthm loaded}}{\def\state{amsthm not loaded}}
\makeatother
\theoremstyle{break}
\newtheorem{thm}{Theorem}
\begin{document}
State: \state.
\begin{thm}A theorem.\end{thm}
\end{document}
",
    &[],
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">State: amsthm not loaded.</p>"#,
  );
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title><tag><text font="bold" xml:id="Thmthm1.1">Theorem 1</text></tag><text font="bold" xml:id="Thmthm1.2">.</text></title>"#,
  );
}

/// PINS A DOCUMENTED LIMIT (KNOWN_PERL_ERRORS #686): the binding applies a binding style when `\newtheorem` runs,
/// where theorem.sty runs the style's `\th@<style>` at each use. A `\th@plain` redefined after the `\newtheorem` sets
/// the body slanted in pdflatex (CMSL10); ours keeps the plain style's italic. Repro
/// package-bindings/theorem_style_taken_at_newtheorem (RED); when the style is applied at use time, this assertion
/// flips to the slanted body.
#[test]
fn theorem_style_taken_at_newtheorem() {
  let xml = convert_clean(repro!("theorem_style_taken_at_newtheorem.tex"), &[]);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"Thmthm1.p1\""],
    r#"<para xml:id="Thmthm1.p1"><p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">A theorem.</text></p></para>"#,
  );
}
