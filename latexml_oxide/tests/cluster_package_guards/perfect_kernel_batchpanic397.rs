//! Guards for the run-397 panics: a register read with arguments evaluated without them (the panel-grid rule's
//! stated widths, pgfmath's register lookup; 2105.01581), an alignment reverted during its own absorption
//! (1901.06584), and titlesec handing its before-code the kernel's title wrapper (ccjnl.cls; 2208.03460, 2208.09785,
//! 2209.08255). Each guard is one code path.

use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

use super::perfect_kernel_batch84b::text;

macro_rules! repro {
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name,
      ".tex"
    ))
  };
}

fn run(tex: &str) -> (String, String) { convert_with(tex, Some("ar5iv.sty")) }

/// The words of `xml`, whitespace collapsed.
fn words(xml: &str) -> String { text(xml).split_whitespace().collect::<Vec<_>>().join(" ") }

/// The panel-grid rule skips `\fontcharwd\font`m`, `\fontcharht…`, `\fontdimen6\font`, `\dimen0` among a stated
/// width's registers (2105.01581): the figure keeps its panels, caption and text.
#[test]
fn a_stated_width_through_a_register_with_arguments_keeps_the_figure() {
  let (log, xml) = run(repro!(
    "captions-floats",
    "a_stated_width_through_a_register_with_arguments"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let words = words(&xml);
  for caption in ["(a) A", "(b) B", "Figure 1 C"] {
    assert!(words.contains(caption), "{caption}: {words}");
  }
  assert!(words.contains("x — y z w v"), "{words}");
}

/// pgfmath does not run a computed register's getter without its arguments: TeX errs on it too.
#[test]
fn pgfmath_a_register_with_arguments_errs() {
  let (log, _) = run(repro!("tikz-pgf", "pgfmath_a_register_with_arguments_errs"));
  assert_eq!((error_count(&log), warning_count(&log)), (2, 2), "{log}");
  assert_eq!(
    log
      .matches("pgfmath cannot read \\fontcharwd without its arguments")
      .count(),
    2,
    "{log}"
  );
}

/// A nested `\ooalign` in node math (1901.06584) no longer panics: the outer one was reverted while absorbed, its
/// inner math's malformed close closing the math around it (3 Errors, 3 Warnings). Since HALIGNINT the node's text
/// has TeX's `\halign` (OXIDIZED_DESIGN_DIVERGENCES #1167): both alignments are tabulars, as outside a picture.
#[test]
fn a_nested_ooalign_in_node_math_reverts() {
  let (log, xml) = run(repro!(
    "graphics-tikz",
    "a_nested_ooalign_in_node_math_reverts"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="p1.pic1.1""#],
    r#"<tabular class="ltx_markedasmath" vattach="top" xml:id="p1.pic1.1">
      <tr xml:id="p1.pic1.1.1">
        <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.1.1.1"><tabular class="ltx_markedasmath" vattach="top" xml:id="p1.pic1.1.1.1.1">
          <tr xml:id="p1.pic1.1.1.1.1.1">
            <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.pic1.1.1.1.1.1.1">L</td>
          </tr>
        </tabular></td>
      </tr>
    </tabular>"#,
  );
}

/// titlesec's before-code takes the author's title (`#4{#8}`), not `{\lx@format@title@font@@{subsection}…}`:
/// mfirstuc's `\capitalisefmtwords` capitalizes it (ccjnl.cls; 2208.03460, 2208.09785, 2209.08255).
#[test]
fn titlesec_before_code_takes_the_bare_title() {
  let (log, xml) = run(repro!(
    "package-bindings",
    "titlesec_before_code_takes_the_bare_title"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let title = xml
    .split("<title")
    .nth(1)
    .and_then(|rest| rest.split("</title>").next())
    .map(|title| words(&format!("<title{title}")))
    .unwrap_or_default();
  assert_eq!(title, "0.1 Decoding The Data", "{xml}");
}
