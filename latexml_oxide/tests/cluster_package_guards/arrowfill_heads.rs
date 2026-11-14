//! amsmath's `\arrowfill@#1#2#3#4` draws `#4#1 … \cleaders #2 … #3` (amsmath.sty:971-976): its heads #1 and #3 are the
//! relation, and `\ext@arrow`'s fill #5 (amsmath.sty:1012) is the arrow its labels sit on. Both were a `\to`, so mhchem's
//! `<=>` (mhchem.sty:1251-1254) read "→→" (witness 2405.01836). Repros `tools/perfect_kernel/repros/arrowfill/`.
use latexml::util::test::{convert_with, kpse_has, warning_count};

use super::perfect_kernel_batch46::error_count;

/// A repro: its name, source, the packages it needs, every `<Math>`'s `text=` in order, whole tokens it must hold.
type Row = (
  &'static str,
  &'static str,
  &'static [&'static str],
  &'static [&'static str],
  &'static [&'static str],
);

const ROWS: &[Row] = &[
  (
    "arrowfill_heads",
    include_str!("../../../tools/perfect_kernel/repros/arrowfill/arrowfill_heads.tex"),
    &["amsmath.sty"],
    &[
      "A leftharpoondown B",
      "A rightharpoonup B",
      "A leftrightarrow B",
      "A rightarrow B",
      "A maps-to B",
    ],
    &[
      r#"<XMTok name="leftharpoondown" role="RELOP">↽</XMTok>"#,
      r#"<XMTok name="rightharpoonup" role="RELOP">⇀</XMTok>"#,
    ],
  ),
  (
    "mhchem_arrows",
    include_str!("../../../tools/perfect_kernel/repros/arrowfill/mhchem_arrows.tex"),
    &["mhchem.sty"],
    // The species `A`, `B` are formulas since mhchem's formula hook (OXIDIZED_DESIGN_DIVERGENCES #1039; were text).
    &[
      "A",
      "[⇀↽]",
      "rightharpoonup",
      "leftharpoondown",
      "B",
      "A",
      "rightarrow",
      "B",
      "A",
      "leftrightarrow",
      "B",
    ],
    &[
      r#"<XMTok align="center" name="rightharpoonup" role="RELOP" scriptpos="mid3" width="20.0pt">⇀</XMTok>"#,
      r#"<XMTok align="center" name="leftharpoondown" role="RELOP" scriptpos="mid3" width="20.0pt">↽</XMTok>"#,
      r#"<XMTok align="center" name="leftrightarrow" role="RELOP" scriptpos="mid3" width="20.0pt">↔</XMTok>"#,
    ],
  ),
  (
    "ext_arrow_fills",
    include_str!("../../../tools/perfect_kernel/repros/arrowfill/ext_arrow_fills.tex"),
    &["chemarr.sty", "extarrows.sty"],
    &[
      "A [⇀a↽b] B",
      "(rightharpoonup ^ a) _ absent",
      "(leftharpoondown ^ absent) _ b",
      "A Relbar ^ a B",
      "A leftrightarrow ^ b B",
      "A twoheadrightarrow ^ c B",
      "A d@(rightarrow) B",
    ],
    &[r#"<XMTok name="twoheadrightarrow" role="RELOP">↠</XMTok>"#],
  ),
];

#[test]
fn arrowfill_keeps_its_heads() {
  for (name, tex, packages, texts, tokens) in ROWS {
    if let Some(missing) = packages.iter().find(|p| !kpse_has(p)) {
      eprintln!("SKIP {name}: {missing} is not in this TeX Live");
      continue;
    }
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}:\n{log}"
    );
    let found: Vec<&str> = xml
      .split("<Math ")
      .skip(1)
      .filter_map(|m| m.split(" text=\"").nth(1)?.split('"').next())
      .collect();
    assert_eq!(&found, texts, "{name}:\n{xml}");
    for token in *tokens {
      assert!(xml.contains(token), "{name}: no {token}\n{xml}");
    }
    assert!(
      !xml.contains(r#"name="to""#) && !xml.contains(r"style}"),
      "{name}: a `\\to` or a stray style switch\n{xml}"
    );
  }
}
