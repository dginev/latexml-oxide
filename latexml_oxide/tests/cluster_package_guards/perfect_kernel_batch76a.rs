//! Red/green guards for batch 76a: a document's `\halign` redefinition runs where an alignment the binding builds
//! reaches `\halign` in TeX (tex_tables.rs `run_halign_redefinition_prelude`, OXIDIZED_DESIGN_DIVERGENCES #760) — not
//! for `tabbing`, which latex.ltx builds from boxes. cryptocode.sty:950-957 (and its `mode=text` branch :760-770)
//! redefines `\halign` inside `\pseudocode` to bind `\@pc@lb` and the line-numbering `\\` just before the primitive;
//! amsmath's `flalign*` reaches it. Without the prelude every pseudocode line after the first lost its number,
//! silently, and `mode=text` erred `undefined:\@pc@lb` (witnesses 1804.05141, 2106.03626, 2107.06049, 2108.04600,
//! 2202.07253). Repro `tools/perfect_kernel/repros/kernel-alignment/cryptocode_halign_redefinition_numbers_lines.tex`.
//!
//! The cryptocode guards need cryptocode.sty: CI installs it with `texlive-science`, and every TeX Live since 2021
//! ships the same v0.44 (2021/08/15) — the `\pcln` padding pinned below is that file's (`\hspace{1ex}`,
//! `\pclnspace` = 1em at `\scriptsize`).
use latexml::util::test::{assert_element, convert_with, kpse_has};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// Whether TeX Live has cryptocode.sty (the guards below that load it skip otherwise, saying so).
fn has_cryptocode() -> bool {
  let found = kpse_has("cryptocode.sty");
  if !found {
    eprintln!("skipped: cryptocode.sty is not in the TeX tree");
  }
  found
}

/// The line-number cell of pseudocode row `row` (`\pcln`, cryptocode.sty:656-662: `\pclnstyle{N}:\hspace{\pclnspace}`).
fn number_td(row: u32, n: u32) -> String {
  format!(
    r#"<td align="right" xml:id="S0.Ex{row}.1"><Math mode="inline" tex="\displaystyle\hskip 3.87498pt\text{{\scriptsize {n}}}:\hskip 9.24994pt" text="[{n}] colon absent" xml:id="S0.Ex{row}.m1"><XMath xml:id="S0.Ex{row}.m1.1"><XMApp xml:id="S0.Ex{row}.m1.1.1"><XMTok fontsize="90%" name="colon" role="METARELOP" rpadding="9.3pt">:</XMTok><XMText lpadding="3.9pt" xml:id="S0.Ex{row}.m1.1.1.2"><text fontsize="70%" xml:id="S0.Ex{row}.m1.1.1.2.1">{n}</text></XMText><XMTok meaning="absent"/></XMApp></XMath></Math></td>"#
  )
}

/// The statement cell of math-mode pseudocode row `row`: `var \gets n`.
fn math_statement_td(row: u32, var: &str, n: u32) -> String {
  format!(
    r#"<td align="left" xml:id="S0.Ex{row}.2"><Math mode="inline" tex="\displaystyle\vphantom{{\rule[1.93748pt]{{0.0pt}}{{0.0pt}}}}{var}\leftarrow {n}" text="{var} leftarrow {n}" xml:id="S0.Ex{row}.m2"><XMath xml:id="S0.Ex{row}.m2.1"><XMApp xml:id="S0.Ex{row}.m2.1.1"><XMTok fontsize="90%" name="leftarrow" role="ARROW">←</XMTok><XMTok font="italic" fontsize="90%" role="UNKNOWN">{var}</XMTok><XMTok fontsize="90%" meaning="{n}" role="NUMBER">{n}</XMTok></XMApp></XMath></Math></td>"#
  )
}

/// The statement cell of text-mode pseudocode row `row` (the `{varwidth}` of `\@pc@modebegin`, cryptocode.sty:763).
fn text_statement_td(row: u32, text: &str) -> String {
  format!(
    r#"<td align="left" xml:id="S0.Ex{row}.2"><inline-block class="ltx_markedasmath ltx_minipage" vattach="middle" width="113.8pt" xml:id="S0.Ex{row}.2.1"><p xml:id="S0.Ex{row}.2.1.1"><text fontsize="90%" xml:id="S0.Ex{row}.2.1.1.1">{text}</text></p></inline-block></td>"#
  )
}

/// The repro's two blocks (with `width=` so cryptocode skips its measuring pass, whose `\got@maxcolwd` sum the binding
/// does not fill — a separate residual). pdflatex: "1: a←1 2: b←2 3: c←3" and "1: first line 2: second line 3: third
/// line", 0 errors. Every row keeps its number and its statement, one row each.
#[test]
fn cryptocode_halign_redefinition_numbers_every_line() {
  if !has_cryptocode() {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{cryptocode}
\begin{document}
\procedure[linenumbering, width=4cm]{Title}{
a \gets 1 \\
b \gets 2 \\
c \gets 3
}
\procedure[mode=text, linenumbering, width=4cm]{Title}{
first line \\
second line \\[1mm]
third line
}
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<equation ").count(), 6, "{xml}");
  for (row, var, n) in [(1, "a", 1), (2, "b", 2), (3, "c", 3)] {
    let id = format!("xml:id=\"S0.Ex{row}.1\"");
    assert_element(&xml, "td", &[&id], &number_td(row, n));
    let id = format!("xml:id=\"S0.Ex{row}.2\"");
    assert_element(&xml, "td", &[&id], &math_statement_td(row, var, n));
  }
  for (row, n, text) in [
    (4, 1, "first line"),
    (5, 2, "second line"),
    (6, 3, "third line"),
  ] {
    let id = format!("xml:id=\"S0.Ex{row}.1\"");
    assert_element(&xml, "td", &[&id], &number_td(row, n));
    let id = format!("xml:id=\"S0.Ex{row}.2\"");
    assert_element(&xml, "td", &[&id], &text_statement_td(row, text));
  }
}

/// The redefinition is scoped by the document's own group: a plain `\halign` and a tabular after the pseudocode block
/// are set as without it (pdflatex: "x y / z w", "p q / r s", 0 errors). (`tabbing` never runs it at all:
/// `halign_redefinition_runs_for_a_tabular_not_for_a_tabbing`.)
#[test]
fn alignments_after_a_pseudocode_block_are_unaffected() {
  if !has_cryptocode() {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{cryptocode}
\begin{document}
\procedure[linenumbering, width=4cm]{Title}{
a \gets 1 \\
b \gets 2
}

\halign{#\hfil&\quad#\hfil\cr x&y\cr z&w\cr}

\begin{tabular}{ll}
p & q \\
r & s
\end{tabular}
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "td", &["xml:id=\"S0.Ex2.1\""], &number_td(2, 2));
  assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p2.1\""],
    r#"<tabular xml:id="p2.1"><tr xml:id="p2.1.1"><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p2.1.1.1">x</td><td align="left" class="ltx_nopad_r" xml:id="p2.1.1.2">y</td></tr><tr xml:id="p2.1.2"><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p2.1.2.1">z</td><td align="left" class="ltx_nopad_r" xml:id="p2.1.2.2">w</td></tr></tabular>"#,
  );
  assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p3.1\""],
    r#"<tabular vattach="middle" xml:id="p3.1"><tbody><tr xml:id="p3.1.1"><td align="left" xml:id="p3.1.1.1">p</td><td align="left" xml:id="p3.1.1.2">q</td></tr><tr xml:id="p3.1.2"><td align="left" xml:id="p3.1.2.1">r</td><td align="left" xml:id="p3.1.2.2">s</td></tr></tbody></tabular>"#,
  );
}

/// An `\halign` macro with a global side effect, in scope around a `tabbing`, a tabular and an `align`: latex.ltx's
/// tabbing is built from boxes and never reaches `\halign`, the tabular's `\@array` does once. pdflatex prints
/// "TABBING:0.", "TABULAR:1.", "ALIGN:3." (0 errors): amsmath's measuring pass runs the `align`'s `\halign` twice, the
/// binding builds it once — "ALIGN:2." here, a deliberate limit (OXIDIZED_DESIGN_DIVERGENCES #760).
#[test]
fn halign_redefinition_runs_for_a_tabular_not_for_a_tabbing() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\makeatletter
\let\primhalign\halign
\makeatother
\newcount\hcount
\begin{document}
\begingroup
\def\halign{\global\advance\hcount1\relax\primhalign}
\begin{tabbing}
one \= two \\
three \> four
\end{tabbing}
TABBING:\the\hcount.

\begin{tabular}{ll} p & q \end{tabular} TABULAR:\the\hcount.

\begin{align} a &= b \end{align}
ALIGN:\the\hcount.
\endgroup
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p1.1\""],
    r#"<tabular class="ltx_tabbing" xml:id="p1.1"><tr xml:id="p1.1.1"><td align="left" xml:id="p1.1.1.1">one</td><td align="left" xml:id="p1.1.1.2">two</td></tr><tr xml:id="p1.1.2"><td align="left" xml:id="p1.1.2.1">three</td><td align="left" xml:id="p1.1.2.2">four</td></tr></tabular>"#,
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    r#"<p xml:id="p2.1">TABBING:0.</p>"#,
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p3.2\""],
    r#"<p xml:id="p3.2">TABULAR:1.</p>"#,
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p4.1\""],
    r#"<p xml:id="p4.1">ALIGN:2.</p>"#,
  );
}

/// The conservative boundary: a redefinition whose expansion does not itself end with the primitive (here it reaches it
/// through a second macro) is not run, so the alignment is set as without it. pdflatex runs it ("RAN"); the binding
/// prints "NOTRUN" — the one-level shape is all `run_halign_redefinition_prelude` executes (OXIDIZED_DESIGN_DIVERGENCES
/// #760, a deliberate limit).
#[test]
fn halign_redefinition_not_ending_with_the_primitive_is_not_run() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\makeatletter
\let\lx@primhalign\halign
\def\lx@reach{\lx@primhalign}
\makeatother
\begin{document}
\begingroup
\makeatletter
\def\halign{\gdef\marker{RAN}\lx@reach}
\makeatother
\begin{align}
a &= b \\
c &= d
\end{align}
\endgroup
\ifdefined\marker RAN\else NOTRUN\fi
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">NOTRUN</p>"#,
  );
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
}

/// Material the redefinition typesets before the primitive of a display alignment: TeX errs "Improper \halign inside
/// $$'s" and deletes it (tex.web §776 `flush_math`), keeping the rows — one Warning here, the rows kept, the marker set.
#[test]
fn halign_redefinition_material_before_a_display_alignment_is_reported() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\makeatletter
\let\lx@primhalign\halign
\makeatother
\begin{document}
\begingroup
\makeatletter
\def\halign{\gdef\marker{RAN}X\lx@primhalign}
\makeatother
\begin{align}
a &= b \\
c &= d
\end{align}
\endgroup
\ifdefined\marker RAN\else NOTRUN\fi
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "Warning:unexpected:\\halign Improper \\halign inside $$'s: the redefined \\halign typeset material before its \
       display alignment, which is deleted: X"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">RAN</p>"#,
  );
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  // the deleted material is in no formula
  assert!(
    !xml.contains(">X</XMTok>") && !xml.contains("tex=\"\\displaystyle X"),
    "{xml}"
  );
}

/// Outside display math TeX typesets that material where the redefinition runs, before the alignment, with no error
/// (pdflatex: "XYZ" above "p q", 0 errors): it precedes the tabular.
#[test]
fn halign_redefinition_material_before_a_tabular_is_kept() {
  let tex = r"\documentclass{article}
\let\primhalign\halign
\begin{document}
\begingroup
\def\halign{XYZ\primhalign}
\begin{tabular}{ll} p & q \end{tabular}
\endgroup
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">XYZ<tabular vattach="middle" xml:id="p1.1.1"><tbody><tr xml:id="p1.1.1.1"><td align="left" xml:id="p1.1.1.1.1">p</td><td align="left" xml:id="p1.1.1.1.2">q</td></tr></tbody></tabular></p>"#,
  );
}

/// A pseudocode in a TikZ node: inside the picture `\halign` is the primitive's SVG stand-in `\lxSVG@halign`
/// (`\lxSVG@installcommands`), which cryptocode saves as `\@pc@halign`; the prelude ends with it all the same (witness
/// 2107.05054, 11 errors → 0). pdflatex: "1: first line 2: second line", 0 errors.
#[test]
fn cryptocode_pseudocode_in_a_tikz_node_numbers_every_line() {
  if !has_cryptocode() {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage{cryptocode}
\begin{document}
\begin{tikzpicture}
\node[draw] (a) {\pseudocode[mode=text, linenumbering, width=4cm]{
first line \\
second line
}};
\end{tikzpicture}
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (row, text) in [(1, "first line"), (2, "second line")] {
    let id = format!("xml:id=\"S0.Ex{row}.1\"");
    assert_element(&xml, "td", &[&id], &number_td(row, row));
    let id = format!("xml:id=\"S0.Ex{row}.2\"");
    assert_element(&xml, "td", &[&id], &text_statement_td(row, text));
  }
}

/// cases.sty's `numcases` sets its `\halign` inside a `\vbox` (cases.sty:73-75), not as the display: TeX keeps the
/// material the redefinition typesets (pdflatex prints "XYZ" above the cases, 0 errors), so no Warning and the material
/// stays.
#[test]
fn halign_redefinition_material_before_numcases_is_kept() {
  let tex = r"\documentclass{article}
\usepackage{cases}
\let\primhalign\halign
\begin{document}
\begingroup
\def\halign{XYZ\primhalign}
\begin{numcases}{f(x)=}
1 & if $x>0$\\
0 & otherwise
\end{numcases}
\endgroup
\end{document}
";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("XYZ"), "{xml}");
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
}
