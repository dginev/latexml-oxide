//! Red/green guards for perfect-kernel batch 65b (C5 step G4): an alignment column end is recognized by its command
//! code, as tex.web §342/§789 insert the v-template on `cur_cmd` ∈ {tab_mark, car_ret} and §783 reads the preamble —
//! a `\def\span`/`\def\cr` macro is a call, a `\let` copy of the primitive still ends the column (gullet.rs
//! `is_column_end`). Repros `tools/perfect_kernel/repros/kernel-alignment/{span_redefined_is_a_macro,
//! cr_redefined_is_a_macro,let_column_ends_keep_their_command}.tex`; round 2: a misplaced `\noalign` changes no
//! meaning, the frozen `\span` of `\multicolumn`/`\multispan` (user ruling 2026-10-11), the
//! frozen `\cr` at a file end and `\tabskip` by command; round 3: a misplaced `\noalign`'s body is an ordinary group,
//! aa.cls's `{longtab}` and aastex's `\tablewidth{0pt}`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_files_with, convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/",
      $name,
      ".tex"
    ))
  };
}

/// The `tex` attribute of every `<Math>`, in document order.
fn math_tex(xml: &str) -> Vec<&str> {
  xml
    .split("<Math ")
    .skip(1)
    .filter_map(|m| {
      let open = &m[..m.find('>')?];
      let start = open.find("tex=\"")? + 5;
      Some(&open[start..start + open[start..].find('"')?])
    })
    .collect()
}

/// math0004102's `\def \span { {\mathrm{span}} }` in an `eqnarray` cell: the macro is expanded in the cell, which
/// keeps "span" in its math (base: the `\span` ended the cell, `\mathrm` and `^` ran out of math, 1 error).
#[test]
fn redefined_span_is_a_macro_not_a_column_end() {
  let xml = convert_clean(repro!("span_redefined_is_a_macro"));
  assert_eq!(
    math_tex(&xml),
    [
      r"\displaystyle g^{v}_{1}={\mathrm{span}}\{h,g^{\alpha}\}x y",
      r"\displaystyle g^{v}_{1}={\mathrm{span}}\{h,g^{\alpha}\}",
      r"\displaystyle x",
      r"\displaystyle y",
    ],
    "{xml}"
  );
  // the fork's math and its first cell's
  assert_eq!(
    xml.matches(r#"<XMTok role="UNKNOWN">span</XMTok>"#).count(),
    2,
    "{xml}"
  );
}

/// A macro `\cr` (plain's `\bordermatrix` idiom) expands in the body — here to a third cell and the primitive row
/// end (base: it ended the row unexpanded, 4 errors).
#[test]
fn redefined_cr_is_a_macro_not_a_row_end() {
  let xml = convert_clean(repro!("cr_redefined_is_a_macro"));
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular xml:id="p1.1"><tr xml:id="p1.1.1"><td align="left" class="ltx_nopad_l" xml:id="p1.1.1.1">a</td><td align="left" class="ltx_nopad_l" xml:id="p1.1.1.2">b</td><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.3">z</td></tr><tr xml:id="p1.1.2"><td align="left" class="ltx_nopad_l" xml:id="p1.1.2.1">c</td><td align="left" class="ltx_nopad_l" xml:id="p1.1.2.2">d</td><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.2.3">e</td></tr></tabular>"#,
  );
}

/// `\let` copies of `\span` and `\cr` keep the command: in the preamble `\myspan` expands the next token as `\span`
/// does and `\mycr` ends it, in the body `\myspan` spans and `\mycr` ends the row (base: the preamble knew `\span` by
/// name only, 2 errors).
#[test]
fn let_copies_of_span_and_cr_keep_their_command() {
  let xml = convert_clean(repro!("let_column_ends_keep_their_command"));
  assert_element(
    &xml,
    "tabular",
    &[],
    // `[c]`, the `\quad` (an em space) of the first column's template, `[d]`
    concat!(
      r#"<tabular xml:id="p1.1"><tr xml:id="p1.1.1"><td align="left" class="ltx_nopad_l" xml:id="p1.1.1.1">[a]</td><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.2">[b]</td></tr><tr xml:id="p1.1.2"><td align="left" class="ltx_nopad_l ltx_nopad_r" colspan="2" xml:id="p1.1.2.1">[c]"#,
      "\u{2003}",
      r#"[d]</td></tr></tabular>"#
    ),
  );
}

/// The `(log, xml)` of `tex` under ar5iv, with exactly `errors` errors and `warnings` warnings, each `messages` entry
/// the text of one of them.
fn convert_counted(tex: &str, (errors, warnings): (usize, usize), messages: &[&str]) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), errors, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  for message in messages {
    assert!(log.contains(message), "{message:?} expected:\n{log}");
  }
  xml
}

const FROZEN_SPAN_WARNING: &str = "Warning:unexpected:\\span the document redefines \\span";

/// A stray `\hline` in body text is `\noalign cannot be used here`; the recovery changes no meaning (tex.web §1129),
/// so a later `\multicolumn` still spans (65b r1: a `\let\span\relax` leaked past the recovery's group and "C" fell
/// into column 2; 64ze: 7 errors).
#[test]
fn misplaced_noalign_leaves_span_alone() {
  let xml = convert_counted(repro!("misplaced_noalign_leaves_span_alone"), (1, 0), &[
    "Error:unexpected:\\noalign \\noalign cannot be used here",
  ]);
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p2.1"><thead><tr xml:id="p2.1.1"><td align="center" border="l r" colspan="2" thead="column" xml:id="p2.1.1.1">AB</td><td align="right" border="r" thead="column" xml:id="p2.1.1.2">C</td></tr></thead><tbody><tr xml:id="p2.1.2"><td align="left" border="l r" xml:id="p2.1.2.1">d</td><td align="center" border="r" xml:id="p2.1.2.2">e</td><td align="right" border="r" xml:id="p2.1.2.3">f</td></tr></tbody></tabular>"#,
  );
}

/// The frozen `\span` (user ruling 2026-10-11): under a document's `\renewcommand{\span}`, `\def\span` or
/// `\let\span\relax\DeclareMathOperator{\span}`, `\multicolumn` still spans its columns (TeX: "Misplaced \omit"),
/// the author's `\span` stays theirs in math, and the clash is one Warning per document (65b r1: `\omit cannot be
/// used here`, no colspan). Witnesses 1405.2652, 1802.10337.
#[test]
fn multicolumn_spans_with_the_frozen_span() {
  // q1 rules its rows with `\hline`: a top border on every cell, a bottom one on the last row
  for (tex, top, bottom, math) in [
    (
      repro!("multicolumn_frozen_span_renewcommand"),
      " t",
      "b ",
      r"V=\operatorname{span}\{v_{1},v_{2}\}",
    ),
    (
      repro!("multicolumn_frozen_span_def"),
      "",
      "",
      r"V={\rm span}\{v_{1},v_{2}\}",
    ),
    (
      repro!("multicolumn_frozen_span_declaremathoperator"),
      "",
      "",
      // the declared operator reverts as its name
      r"V=\span\{v_{1},v_{2}\}",
    ),
  ] {
    let xml = convert_counted(tex, (0, 1), &[FROZEN_SPAN_WARNING]);
    assert_element(
      &xml,
      "tabular",
      &[],
      &format!(
        r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p1.1.1"><thead><tr xml:id="p1.1.1.1"><td align="center" border="l r{top}" colspan="2" thead="column" xml:id="p1.1.1.1.1">Joint head</td><td align="center" border="r{top}" thead="column" xml:id="p1.1.1.1.2">Solo</td></tr></thead><tbody><tr xml:id="p1.1.1.2"><td align="center" border="{bottom}l r{top}" xml:id="p1.1.1.2.1">alpha</td><td align="center" border="{bottom}r{top}" xml:id="p1.1.1.2.2">beta</td><td align="center" border="{bottom}r{top}" xml:id="p1.1.1.2.3">gamma</td></tr></tbody></tabular>"#
      ),
    );
    assert!(
      math_tex(&xml).contains(&math),
      "{math:?} not in {:?}",
      math_tex(&xml)
    );
  }
}

/// latex.ltx's `\multispan` (`\omit\@multispan`, `\sp@n` = `\span\omit…`, :16749-16753) spans with the frozen `\span`
/// too: one Warning, the head spans two columns without a template (as `\omit` gives).
#[test]
fn multispan_spans_with_the_frozen_span() {
  let xml = convert_counted(repro!("multispan_frozen_span"), (0, 1), &[
    FROZEN_SPAN_WARNING,
  ]);
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p1.1.1"><thead><tr xml:id="p1.1.1.1"><td align="center" colspan="2" thead="column" xml:id="p1.1.1.1.1">Joint head</td><td align="center" border="r" thead="column" xml:id="p1.1.1.1.2">Solo</td></tr></thead><tbody><tr xml:id="p1.1.1.2"><td align="center" border="l r" xml:id="p1.1.1.2.1">alpha</td><td align="center" border="r" xml:id="p1.1.1.2.2">beta</td><td align="center" border="r" xml:id="p1.1.1.2.3">gamma</td></tr></tbody></tabular>"#,
  );
}

/// amsmath's `\hdotsfor` binding fills its cells with `&` (amsmath_sty.rs), so a redefined `\span` does not reach
/// it: no frozen `\span`, no Warning, a dots row of three cells.
#[test]
fn hdotsfor_ignores_a_redefined_span() {
  let xml = convert_clean(repro!("hdotsfor_redefined_span"));
  assert_eq!(
    math_tex(&xml),
    [
      r"V=\operatorname{span}\{v_{1},v_{2}\}",
      "\\begin{matrix}a&amp;b&amp;c\\\\&#10;\\ldots&amp;\\ldots&amp;\\ldots\\\\&#10;d&amp;e&amp;f\\end{matrix}",
    ],
    "{xml}"
  );
  assert_eq!(xml.matches("<XMCell").count(), 9, "{xml}");
}

/// tex.web §339: a file ending in an `\halign` preamble inserts `frozen_cr`; under the author's `\def\cr{X}` the
/// engine inserts its `\lx@frozen@cr`, so the preamble ends there and the text after the `\input` stays (65b r1:
/// the author's `\cr` ran and "after" was lost).
#[test]
fn file_end_in_a_preamble_inserts_the_frozen_cr() {
  let (log, xml) = convert_files_with(
    repro!("file_end_in_preamble_frozen_cr"),
    &[(
      "file_end_in_preamble_frozen_cr.snippet",
      include_str!(
        "../../../tools/perfect_kernel/repros/kernel-alignment/file_end_in_preamble_frozen_cr.snippet"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    log.contains("File ended while scanning preamble of \\halign"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">after</p></para>"#,
  );
}

/// tex.web §783 reads `\tabskip` in a preamble by command: `\let\mytabskip\tabskip` sets the glue before the second
/// column as `\tabskip` does (65b r1: matched by name, the assignment stayed template text).
#[test]
fn let_copy_of_tabskip_sets_the_preamble_glue() {
  let tex = repro!("let_tabskip_in_preamble");
  let table = r#"<tabular xml:id="p1.1"><tr xml:id="p1.1.1"><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.1">a</td><td align="left" class="ltx_nopad_r" xml:id="p1.1.1.2">b</td></tr><tr xml:id="p1.1.2"><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.2.1">c</td><td align="left" class="ltx_nopad_r" xml:id="p1.1.2.2">d</td></tr></tabular>"#;
  assert_element(&convert_clean(tex), "tabular", &[], table);
  // the primitive gives the same table
  let primitive = tex.replace(r"\mytabskip=1em", r"\tabskip=1em");
  assert_element(&convert_clean(&primitive), "tabular", &[], table);
}

/// tex.web §1126/§1129: a misplaced `\noalign` is "Misplaced \noalign" and its `{…}` an ordinary group, typeset where it
/// stands (64ze/65b r2 read the body unexpanded and dropped "visible words").
#[test]
fn misplaced_noalign_body_is_typeset() {
  let xml = convert_counted(repro!("misplaced_noalign_body_is_typeset"), (1, 0), &[
    "Error:unexpected:\\noalign \\noalign cannot be used here",
  ]);
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text visible words end.</p></para>"#,
  );
}

/// A stray `\hline` in a paragraph: one error, and its body (`\@@alignment@hline`) is inert off an alignment.
#[test]
fn stray_hline_in_a_paragraph_is_one_error() {
  let xml = convert_counted(repro!("stray_hline_in_a_paragraph"), (1, 0), &[
    "Error:unexpected:\\noalign \\noalign cannot be used here",
  ]);
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Some text more text.</p></para>"#,
  );
}

/// Each misplaced `\noalign`, `\omit` and `\span` is reported, as TeX reports each (pdflatex 4 errors; Perl's
/// never-closed recovery group silences the later ones; 64ze 7), and the table after them still spans. 65h: the
/// first in a group is the Error, the later ones its cascade, Warnings (Perl's count; run 340's 107 tables read
/// outside an alignment went Fatal:TooManyErrors with one Error each, `perfect_kernel_batch65h`).
#[test]
fn misplaced_commands_are_each_reported() {
  let xml = convert_counted(
    repro!("misplaced_noalign_omit_span_each_reported"),
    (1, 3),
    &[
      "Error:unexpected:\\noalign \\noalign cannot be used here",
      "Warning:unexpected:\\noalign \\noalign cannot be used here (in the cascade",
      "Warning:unexpected:\\omit \\omit cannot be used here (in the cascade",
      "Warning:unexpected:\\span \\span cannot be used here (in the cascade",
    ],
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text more again x y.</p></para>"#,
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p2.1"><thead><tr xml:id="p2.1.1"><td align="center" border="l r" colspan="2" thead="column" xml:id="p2.1.1.1">AB</td><td align="right" border="r" thead="column" xml:id="p2.1.1.2">C</td></tr></thead><tbody><tr xml:id="p2.1.2"><td align="left" border="l r" xml:id="p2.1.2.1">d</td><td align="center" border="r" xml:id="p2.1.2.2">e</td><td align="right" border="r" xml:id="p2.1.2.3">f</td></tr></tbody></tabular>"#,
  );
}

/// aa.cls:1430 makes `{longtab}` an environment (answers.sty `\Newassociation`); it wraps its `{longtable}`, which
/// keeps its caption and row (1611.08428; 65b r2: `\longtab{}` took the inner `\begin` as its argument, 6 errors).
#[test]
fn aa_longtab_environment_wraps_its_longtable() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aa_longtab_environment.tex"
  ));
  assert_element(
    &xml,
    "caption",
    &[],
    r#"<caption><tag close=": ">Table 1</tag>Cap</caption>"#,
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular xml:id="S0.T1.1"><tr xml:id="S0.T1.1.1"><td align="left" border="b t" xml:id="S0.T1.1.1.1">a</td><td align="left" border="b t" xml:id="S0.T1.1.1.2">b</td></tr></tabular>"#,
  );
}

/// aastex62.cls:1849-1858: `\tablewidth{0pt}` expands to nothing, so at the start of a `tabular` row the `\hline`s
/// after it are the row's rules (1805.09353; 65b r2: the binding's assignment opened a cell, 2 errors).
#[test]
fn tablewidth_zero_is_expandable() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aastex_tablewidth_zero_in_tabular.tex"
  ));
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="tab1.1"><tbody><tr xml:id="tab1.1.1"><td align="left" border="b tt" xml:id="tab1.1.1.1">a</td><td align="right" border="b tt" xml:id="tab1.1.1.2">b</td></tr></tbody></tabular>"#,
  );
}

/// A misplaced `\hline` inside a cell (after `\vspace` opened it): one error, and its body's `\@@alignment@hline` rules
/// no row — in TeX the `\hrule` errors in the cell and draws nothing (2511.12320, 2306.10087: stacked `border="bbbb…"`
/// and empty rows). The mark is local to the body's group: the proper `\hline` after `c & d` still rules e|f.
#[test]
fn misplaced_hline_in_a_cell_draws_no_rule() {
  let xml = convert_counted(
    repro!("misplaced_hline_in_a_cell_draws_no_rule"),
    (1, 0),
    &["Error:unexpected:\\noalign \\noalign cannot be used here"],
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="left" border="l r" xml:id="p1.1.1.1">a</td><td align="left" border="r" xml:id="p1.1.1.2">b</td></tr><tr xml:id="p1.1.2"><td align="left" border="l r" xml:id="p1.1.2.1">c</td><td align="left" border="r" xml:id="p1.1.2.2">d</td></tr><tr xml:id="p1.1.3"><td align="left" border="l r t" xml:id="p1.1.3.1">e</td><td align="left" border="r t" xml:id="p1.1.3.2">f</td></tr></tbody></tabular>"#,
  );
}

/// In math, a misplaced `\noalign`'s `{…}` is the ordinary group the real `{` opens: `$a \noalign{b+c} d$` reverts as
/// `a{b+c}d` (65b r3 swallowed the `{` for its own group: `{ab+c}d`), and `\noalign\bgroup…\egroup` is a group too.
#[test]
fn misplaced_noalign_in_math_keeps_its_braces() {
  let xml = convert_counted(
    repro!("misplaced_noalign_in_math_keeps_its_braces"),
    (3, 0),
    &["Error:unexpected:\\noalign \\noalign cannot be used here"],
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="a{b+c}d" text="a * b + c * d" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMApp></XMath></Math>"#,
  );
  assert_eq!(math_tex(&xml)[1], r"p{\hline}q", "{xml}");
  // `\noalign\bgroup…\egroup`: `\bgroup` is a left brace by meaning
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m3""#],
    r#"<Math mode="inline" tex="x{y}z" text="x * y * z" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMApp xml:id="p1.m3.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMath></Math>"#,
  );
}
