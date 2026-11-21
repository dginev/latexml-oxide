//! Red/green guards for batch REGFIXL, a regression of the v0.8.0 per-paper check: ulem's declaration form (`\ul`
//! from `\useunder{\uline}{\ul}{}`, tablesgenerator's output) in a `\multicolumn` body ran out of input. Repros under
//! `tools/perfect_kernel/repros/loader/`.
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
