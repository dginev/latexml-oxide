//! Red/green guards for batch CLUSTFIX3, four more Error roots of run 397 where TeX is clean: amsmath's `\[`/`\]` as
//! `{equation*}`, etoolbox's `\robustify` of a native binding, the native pgfkeys accessors over a paper's own pgf 2.00
//! `pgfkeys.code.tex`, and doublespace's `\setstretch`. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// amsmath.sty:2950-2951 makes `\[`/`\]` `\begin{equation*}`/`\end{equation*}`, so `\[ … \end{equation*}` and
/// `\begin{equation*} … \]` pair (math0306159, math0401181, math0508341; OD #1091). Before: "Can't close environment
/// equation*", the second pair's `\]` ran the kernel's cheap intertext, and a `\tag` in `\[` was dropped without a
/// diagnostic (2008.12392, 2409.02265). A matched `\[ … \]` is unchanged.
/// Repro ams-internals/amsmath_display_brackets_pair_with_equation_star.
#[test]
fn amsmath_display_brackets() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/ams-internals/amsmath_display_brackets_pair_with_equation_star.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_eq!(xml.matches("<equation ").count(), 4, "{xml}");
  for (id, tex) in [("S0.Ex1", "a+b=c."), ("S0.Ex2", "x=y."), ("S0.Ex3", "p=q.")] {
    let head = format!("<equation xml:id=\"{id}\">\n      <Math mode=\"display\" tex=\"{tex}\"");
    assert!(xml.contains(&head), "{xml}");
  }
  let tagged = concat!(
    "<equation xml:id=\"S0.Ex4\">\n      <tags>\n        <tag>(T)</tag>\n",
    "        <tag role=\"refnum\">T</tag>\n      </tags>\n      <Math mode=\"display\" tex=\"r=s\""
  );
  assert!(xml.contains(tagged), "{xml}");
  for words in [
    "We have",
    "and more text.",
    "then text.",
    "end, and a tag:",
    "done.",
  ] {
    assert!(xml.contains(&format!("<p>{words}</p>")), "{xml}");
  }
}

/// etoolbox's `\robustify` of a command that is a macro in LaTeX and a native binding here is a no-op (it is robust
/// already): `\uparrow`, `\sum`, `\it`, `\uline` had "not a macro" (1508.06145, 1702.00233), and `\robustify{\ref}`
/// left `\ref` an undefined `\etb@resrvda` (wiley-article.cls:771-775, 1812.10085), the reference lost. A TeX
/// primitive or a register keeps etoolbox's Error, as pdflatex's (`\hbox`, a constructor here, `\relax`,
/// `\parindent`; OD #1092). Repros package-bindings/robustify_of_a_native_binding and
/// robustify_of_a_tex_primitive_errs.
#[test]
fn robustify_of_a_native_binding() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/robustify_of_a_native_binding.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let p = concat!(
    r#"<p>See Section"#,
    "\u{a0}",
    r#"<ref labelref="LABEL:s:a"/> here. Text <text font="italic">slanted</text>, "#,
    r#"<text class="ltx_ulem_uline">under</text> and <Math mode="inline" "#,
    r#"tex="\sum_{i}x_{i}\uparrow" "#
  );
  assert!(xml.contains(p), "{xml}");

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/robustify_of_a_tex_primitive_errs.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (3, 0), "{log}");
  for message in [
    r"\hbox not a macro",
    r"\relax undefined",
    r"\parindent not a macro",
  ] {
    assert!(
      log.contains(&format!("Package etoolbox Error: {message}")),
      "{log}"
    );
  }
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Box x.</p></para>"#,
  );
}

/// The native pgfkeys accessors (pgfkeys_code_tex.rs) are installed only over a file of TeX Live's shape: a paper's
/// own pgf 2.00 `pgfkeys.code.tex` (delimited `\pgfkeys@ifcsname`, no `\pgfkeys@firstoftwo`; 1108.0463, 1010.2550)
/// keeps its raw code. Before: 6 Errors (`\pgfkeys@secondoftwo` undefined, Extra `\endcsname`, …) and "[A:x]" lost.
/// Repro tikz-pgf/pgf2_pgfkeys (its stand-in files beside it).
#[test]
fn pgfkeys_natives_skip_a_pgf2_pgfkeys() {
  let (log, xml) = convert_files_with(
    include_str!("../../../tools/perfect_kernel/repros/tikz-pgf/pgf2_pgfkeys/pgf2_pgfkeys.tex"),
    &[
      (
        "pgfkeys.sty",
        include_str!("../../../tools/perfect_kernel/repros/tikz-pgf/pgf2_pgfkeys/pgfkeys.sty"),
      ),
      (
        "pgfkeys.code.tex",
        include_str!("../../../tools/perfect_kernel/repros/tikz-pgf/pgf2_pgfkeys/pgfkeys.code.tex"),
      ),
    ],
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Text [A:x] end.</p></para>"#,
  );
}

/// doublespace.sty:66 (TeX Live has none; the copy quant-ph0004056 ships) defines `\setstretch`: undefined before
/// (physics0003041, quant-ph0005132). Repro package-bindings/doublespace_setstretch.
#[test]
fn doublespace_setstretch() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/doublespace_setstretch.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Text one.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Text two.</p></para>"#,
  );
}
