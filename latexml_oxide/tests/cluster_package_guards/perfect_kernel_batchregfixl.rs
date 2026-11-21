//! Red/green guards for batch REGFIXL, a regression of the v0.8.0 per-paper check: ulem's declaration form (`\ul`
//! from `\useunder{\uline}{\ul}{}`, tablesgenerator's output) in a `\multicolumn` body ran out of input. Repros under
//! `tools/perfect_kernel/repros/loader/`. Round 2: a declaration that never meets its `}` errs; nested lines. Round 3:
//! a cell's end and a declaration made directly in another's text err too, as pdflatex is fatal there.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// A declaration's group ends with the cell: latex.ltx:16603-16607, :16664-16676 sets a `\multicolumn` body in the
/// template's `{…}`, which ends `\ul` (ulem.sty:286-293) before the column's after-code, so the cell keeps its borders.
/// The body is spliced without braces here (`\lx@alignment@multicolumn`), where the balanced read of the rest of the
/// group ran out of input (3 errors; 2312.12023 `\multicolumn{1}{c|}{\ul 29.9970}`: 5 errors). RED on INTREGFIX4.
#[test]
fn ulem_declaration_ends_with_the_cell() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/ulem_declaration_ends_with_the_cell.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p1.1"><thead><tr xml:id="p1.1.1"><td align="left" border="l" thead="column row" xml:id="p1.1.1.1">a</td><td align="center" border="r" thead="column" xml:id="p1.1.1.2"><text class="ltx_ulem_uline" xml:id="p1.1.1.2.1">6.7</text></td></tr></thead><tbody><tr xml:id="p1.1.2"><td align="left" border="l r" thead="row" xml:id="p1.1.2.1"><text class="ltx_ulem_uline" xml:id="p1.1.2.1.1">2.9</text></td><td align="center" border="r" xml:id="p1.1.2.2">b</td></tr><tr xml:id="p1.1.3"><td align="left" border="l r" thead="row" xml:id="p1.1.3.1">x <text class="ltx_ulem_uline" xml:id="p1.1.3.1.1">3.1 y</text></td><td align="center" border="r" xml:id="p1.1.3.2">z</td></tr></tbody></tabular>"#,
  );
}

/// The group's own `}` ends the declaration and is read again for the group (ulem.sty:107 `\UL@swender`): a `{…}` group,
/// a nested one, a box whose reader ends at its `}` (`\mbox`, `\fbox`), another of ulem's lines (`\sout`), and an
/// underline command of the document's own, which keeps the read of the rest of the group as its argument. The same
/// markup as the argument read gave (2406.03441, 2405.20343); distinct from the cell guard: the `}` is the closer, not a
/// cell's or a body's end.
#[test]
fn ulem_declaration_closes_with_its_group() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/ulem_declaration_closes_with_its_group.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r##"<p xml:id="p1.1">One <text class="ltx_ulem_uline" xml:id="p1.1.1">a <text class="ltx_ulem_uline" xml:id="p1.1.1.1">b</text> c</text> d. Two <text class="ltx_ulem_uline" font="bold" xml:id="p1.1.2">x</text> y. Three <text class="ltx_ulem_sout" xml:id="p1.1.3">gone</text> kept. Four <text class="ltx_ulem_uline" xml:id="p1.1.4">m</text> n. Five <text class="ltx_ulem_uline" xml:id="p1.1.5">p q</text> <text class="ltx_ulem_uline" cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle" xml:id="p1.1.6">r s</text> after.</p>"##,
  );
}

/// A declaration outside any group (`Text \ul{x} y.`) never meets its group's `}`: pdflatex's fatal "File ended while
/// scanning use of \UL@on". One Error names it, besides the open group's Warning at `\end{document}`; the text stays
/// underlined to the paragraph's end, the next paragraph kept (REGFIXL review item 1). RED on REGFIXL: the Warning only.
#[test]
fn ulem_declaration_unclosed_errs() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/ulem_declaration_unclosed_errs.tex");
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
  assert!(
    log.contains("Error:expected:} Missing } to end the underline declaration made by \\useunder"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text <text class="ltx_ulem_uline" xml:id="p1.1.1">x y.</text></p></para>"#,
  );
  assert!(xml.contains(r#"<p xml:id="p2.1">Next para.</p>"#), "{xml}");
}

/// Two declarations nested in one `\multicolumn` body: the body-end marker closes the inner line and is read again for
/// the outer one; in one `{…}` group the `}`, read again, does. Each line tells its own frame from the outer line's
/// (`lx@ulem@body` holds its depth): before, the inner line's end closed the outer's group too, leaving the `{…}` open
/// (1 warning; RED on REGFIXL). pdflatex is fatal here (`\so` opens a brace nothing closes); the content-first recovery
/// keeps the nesting and the cell's borders, and (REGFIXL3) one Error per inner declaration, which TeX never closes
/// (INTREGFIX4 errs too; RED on REGFIXL2: 0 errors). Distinct from the cell guard: one end closes two lines.
#[test]
fn ulem_declaration_nested_lines_in_a_cell() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/ulem_declaration_nested_lines_in_a_cell.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert_eq!(
    log
      .matches("is made in another declaration's text, whose end does not end it")
      .count(),
    2,
    "{log}"
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" border="l r" xml:id="p1.1.1.1">a <text class="ltx_ulem_uline" xml:id="p1.1.1.1.1">b <text class="ltx_ulem_sout" xml:id="p1.1.1.1.1.1">c</text></text></td><td align="center" border="r" xml:id="p1.1.1.2">d</td></tr></tbody></tabular>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><p xml:id="p2.1">Para a <text class="ltx_ulem_uline" xml:id="p2.1.1">b <text class="ltx_ulem_sout" xml:id="p2.1.1.1">c</text></text> d.</p></para>"#,
  );
}

/// A declaration closed by its cell's `&` or `\\` (`a & \ul 6.7 & {\ul 8} \\`): no group TeX's reading of the argument
/// counts ends there, pdflatex's fatal "File ended while scanning use of \UL@on" (ulem.sty:286-293). One Error per such
/// cell, naming where it opened; the cell's text underlined, the braced `{\ul 8}` silent, the next row and the text after
/// the table kept (REGFIXL3 ruling). RED on REGFIXL2: 0 errors. Distinct: the cell end is the closer (stomach.rs
/// digest_body's alignment return), not a `}` or the body marker.
#[test]
fn ulem_declaration_cell_end_errs() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/ulem_declaration_cell_end_errs.tex");
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert_eq!(
    log
      .matches("Error:expected:} Missing } to end the underline declaration made by \\useunder")
      .count(),
    2,
    "{log}"
  );
  assert_eq!(
    log
      .matches("underlines the text up to its cell's end.")
      .count(),
    2,
    "{log}"
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="left" xml:id="p1.1.1.1">a</td><td align="center" xml:id="p1.1.1.2"><text class="ltx_ulem_uline" xml:id="p1.1.1.2.1">6.7</text></td><td align="right" xml:id="p1.1.1.3"><text class="ltx_ulem_uline" xml:id="p1.1.1.3.1">8</text></td></tr><tr xml:id="p1.1.2"><td align="left" xml:id="p1.1.2.1">b</td><td align="center" xml:id="p1.1.2.2">c</td><td align="right" xml:id="p1.1.2.3"><text class="ltx_ulem_uline" xml:id="p1.1.2.3.1">5</text></td></tr></tbody></tabular>"#,
  );
  assert!(xml.contains(r#"<p xml:id="p1.2">After.</p>"#), "{xml}");
}
