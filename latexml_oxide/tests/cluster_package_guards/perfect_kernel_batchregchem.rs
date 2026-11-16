//! REGCHEM (cutoff sandbox regression, cluster E): a `\ce` formula holding a macro that expands to a math shift
//! (2606.03586 `\ce{\alphasic}`, `\alphasic` = `$\alpha$-\ce{SiC}`) is left to mhchem's own output, as before #850's
//! semantic stage 1: its text-mode layout sets the macro in `\text{…}` (mhchem.sty:2934), where the `$` is TeX's.
use latexml::util::test::{convert_bytes_files_then, kpse_has};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
};

/// Each shape of the repro (the witness's two, the macro after a unit, after an amount, lifted under a charge, a
/// wrapper macro, a robust macro), its paragraph's text and its `alttext`s in order; `\ce{\mm}` (`\ensuremath`, a
/// protected macro) and the literal `{$x$}` (mhchem reads it) stay chemistry math.
#[test]
fn mhchem_macro_math_shift_is_mhchems_own_output() {
  if !kpse_has("mhchem.sty") {
    eprintln!("SKIP: mhchem.sty is not in this TeX Live");
    return;
  }
  let (log, _, (html, post_log)) = convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/package-bindings/mhchem_formula_macro_math_shift.tex"
    ),
    &[],
    Some("ar5iv.sty"),
    |xml, source_dir| {
      super::perfect_kernel_batch64zf::post_logged(
        xml,
        source_dir.to_str().expect("a UTF-8 path"),
      )
    },
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(error_count(&post_log), 0, "{post_log}");
  let paragraphs: Vec<String> = html
    .match_indices("<p ")
    .map(|(at, _)| {
      let p = &html[at..at + html[at..].find("</p>").expect("a closed <p>")];
      // The visible text: a `<math>`'s own, not its `<annotation>`.
      let mut visible = String::new();
      let mut rest = p;
      while let Some(open) = rest.find("<annotation") {
        visible.push_str(&rest[..open]);
        rest = &rest[open + rest[open..].find("</annotation>").expect("a closed annotation")..];
      }
      visible.push_str(rest);
      text(&visible).split_whitespace().collect::<Vec<_>>().join(" ")
    })
    .collect();
  let alttexts: Vec<&str> = html
    .match_indices("alttext=\"")
    .map(|(at, m)| {
      let from = at + m.len();
      &html[from..from + html[from..].find('"').expect("a closed alttext")]
    })
    .collect();
  assert_eq!(
    paragraphs,
    [
      "α - Si C and β - Ga 2 O 3",
      "H 2 \u{2062} α - Si C and 2 α - Si C",
      "α − Si C \u{2062} 2+ and H 2 \u{2062} O α - Si C",
      "α - Si C and α - Si C",
      "x and x H 2 O",
    ],
    "paragraphs"
  );
  // The witness's formulas: α and SiC apart, no `$` in their alttext; the macro after a unit (H2) is mhchem's own
  // `\text{…}`, its `$` the `tex` of a text holding math, as before #850.
  assert_eq!(
    alttexts,
    [
      r"\alpha",
      r"\mathord{\mathrm{Si}}\mathord{\mathrm{C}}",
      r"\beta",
      r"\mathord{\mathrm{Ga}}_{\mathrm{2}}\mathord{\mathrm{O}}_{\mathrm{3}}",
      r"\text{H}{\vphantom{\text{X}}}_{\smash[t]{\text{2}}}\text{$\alpha$-{}$\mathord{\mathrm{Si}}\mathord{\mathrm{C}}$}",
      r"\alpha",
      r"\mathord{\mathrm{Si}}\mathord{\mathrm{C}}",
      r"\alpha",
      r"-{}\mathord{\mathrm{Si}}\mathord{\mathrm{C}}\text{}{\vphantom{\text{X}}}^{\text{2+}}",
      r"\text{H}{\vphantom{\text{X}}}_{\smash[t]{\text{2}}}\text{O}\text{$\alpha$-{}$\mathord{\mathrm{Si}}\mathord{\mathrm{C}}$}",
      r"\alpha",
      r"\mathord{\mathrm{Si}}\mathord{\mathrm{C}}",
      r"\alpha",
      r"\mathord{\mathrm{Si}}\mathord{\mathrm{C}}",
      r"\mathrm{x}",
      r"\mathrm{\mathrm{\text{$\text{$x$}$}}H}_{\mathrm{2}}\mathord{\mathrm{O}}",
    ],
    "alttexts"
  );
}
