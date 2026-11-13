//! `\shipout` and the parskip binding (batch 56jk, sweep #124 triage).
//!
//! `\shipout` is a primitive (tex_file_io.rs) that scans its box operand as
//! `\setbox` does and emits the box in place: LaTeX's `\shipout`
//! (`\__shipout_execute:`, latex.ltx:19911-19917) ends in `\tex_shipout:D`
//! (latex.ltx:19986), which no engine defined, so every `\shipout` was
//! `undefined:\tex_shipout:D` once `\afterassignment` fired after `\setbox`
//! (witness coverpage/SimpleSample). Perl has no `\shipout`: it drops a
//! register operand's box silently and errors on a braced one (#314).
//! The dump must carry `\tex_shipout:D` as an alias of the primitive: a dump
//! built before the primitive existed leaves it undefined.
//!
//! The parskip binding raw-loads parskip.sty, which requires kvoptions and
//! etoolbox and processes its options (witnesses liftarm, polyomino:
//! `undefined:\AtEndPreamble`). Repros are in
//! `tools/perfect_kernel/repros/{boxes-groups,loader}/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

#[test]
fn shipout_emits_the_box_in_place() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/shipout_box_register_simplesample.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // Where `\box255` puts the same box (a vbox in vertical mode), before the body.
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para vattach="bottom" xml:id="p1"><p>Cover text</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Body text.</p></para>"#,
  );
  // Shipped content appears exactly once (a duplicate after the body is the
  // feature's main risk).
  assert_eq!(xml.matches("Cover text").count(), 1, "{xml}");
  assert_eq!(xml.matches("<para ").count(), 2, "{xml}");
}

/// Every box operand: a `\vbox` body (LaTeX's `\afterassignment` takes the
/// `\aftergroup` branch), an `\hbox` inside a paragraph, `\copy`, and a box
/// voided by an earlier `\shipout\box0`, which LaTeX reports as pdflatex does
/// ("Ignoring void shipout box", latex.ltx:19944).
#[test]
fn shipout_takes_every_box_operand() {
  let tex = r"\documentclass{article}
\begin{document}
\shipout\vbox{Cover page}
Body text \shipout\hbox{inline page} continues.
\newsavebox\mybox\sbox\mybox{Saved}\shipout\copy\mybox
\setbox0\hbox{A}\shipout\box0 \shipout\box0 End.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Ignoring void shipout box"), "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para vattach="bottom" xml:id="p1"><p>Cover page</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    "<para xml:id=\"p2\"><p>Body text inline page continues.\nSavedAEnd.</p></para>",
  );
  for once in ["Cover page", "inline page", "Saved"] {
    assert_eq!(xml.matches(once).count(), 1, "{once}: {xml}");
  }
  assert_eq!(xml.matches("<para ").count(), 2, "{xml}");
}

#[test]
fn parskip_loads_kvoptions_and_etoolbox() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/parskip_loads_kvoptions_etoolbox_liftarm.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para class="ltx_noindent" xml:id="p1"><p>ok</p></para>"#,
  );
}

/// parskip.sty:45-58: its options: `skip` sets `\parskip`, `indent` keeps a
/// non-zero `\parindent`, so the paragraph is no longer `ltx_noindent`.
#[test]
fn parskip_processes_its_package_options() {
  let tex = r"\documentclass{article}
\usepackage[skip=10pt,indent=5pt]{parskip}
\begin{document}
\the\parskip, \the\parindent
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>10.0pt, 5.0pt</p></para>"#,
  );
}
