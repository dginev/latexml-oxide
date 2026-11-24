//! Red/green guards for batch REGTAILD (row R REGTAIL: papers an old build converted cleanly that later erred): under
//! amsthm a `\relax` counter argument names no counter; sidecap's floats are the kernel's figure and table; mathtools'
//! paired delimiters are robust commands. Repros under `tools/perfect_kernel/repros/package-bindings/`.
use latexml::util::test::{assert_element, convert_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The `<tags>` of the theorem `id`.
fn theorem_tags(xml: &str, id: &str) -> String {
  let theorem = xml_element(xml, "theorem", &[&format!("xml:id=\"{id}\"")])
    .unwrap_or_else(|| panic!("no theorem {id}\n{xml}"));
  xml_element(&theorem, "tags", &[]).unwrap_or_else(|| panic!("no tags in {theorem}"))
}

fn tags(display: &str, refnum: &str) -> String {
  format!(
    "<tags><tag>{display}</tag><tag role=\"refnum\">{refnum}</tag><tag role=\"typerefnum\">{display}</tag></tags>"
  )
}

/// amsthm.sty:94/:114 `\ifx\relax#2\relax`: under amsthm a `\relax` in the shared-counter or the parent argument names
/// no counter, as thm-kv.sty:401 passes AVT.sty:583's `numberwithin=\ifAVT@notes…\relax\else\relax\fi` (2208.14960,
/// 2309.00854, 2409.18804: 1891 Errors). Red (INT081I): 17 Errors per form, "Theorem 0.1". The kernel's `\@xnthm` has
/// no such test and errs, as pdflatex (6 there; ours at each `\csname the\relax\endcsname`).
#[test]
fn amsthm_relax_counter_argument_names_no_counter() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/amsthm_newtheorem_relax_names_no_counter.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, display, refnum) in [
    ("Thmtheorem1", "Theorem 1", "1"),
    ("Thmlemma1", "Lemma 1", "1"),
    ("S1.Thmclaim1", "Claim 1.1", "1.1"),
  ] {
    assert_eq!(theorem_tags(&xml, id), tags(display, refnum), "{xml}");
  }
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/thmtools_numberwithin_relax_names_no_counter.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, display) in [("Thmremark1", "Remark 1"), ("Thmtheorem1", "Theorem 1")] {
    assert_eq!(theorem_tags(&xml, id), tags(display, "1"), "{xml}");
  }
  let (log, _) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/newtheorem_relax_parent_without_amsthm_errs.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (17, 0), "{log}");
}

/// sidecap.sty:177-182, :225: `SCfigure`/`SCtable` and their starred forms are the figure/table floats themselves
/// (`\@float`), never the `\figure` a class redefined (solarphysics.cls:1714, 2006.04669: a boxed body, "<ltx:caption>
/// isn't allowed in <ltx:block>"; 1510.03282, 1511.05217, 2202.13838). The second optional is the placement.
/// Red (INT081I): the preamble's `\figure`/`\table`, two `ltx:float`s. OXIDIZED_DESIGN_DIVERGENCES #1156.
#[test]
fn sidecap_float_is_the_kernel_float() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/sidecap_float_over_a_class_figure.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<float"), "{xml}");
  let figure = |id: &str, n: &str, attrs: &str, caption: &str| {
    format!(
      "<figure inlist=\"lof\"{attrs} xml:id=\"{id}\">{tags}<rule height=\"28.5pt\" width=\"56.9pt\"/>\
       <toccaption><tag close=\" \">{n}</tag>{caption}</toccaption>\
       <caption><tag close=\": \">Figure {n}</tag>{caption}</caption></figure>",
      tags = tags(&format!("Figure {n}"), n)
    )
  };
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    &figure(
      "S0.F1",
      "1",
      " labels=\"LABEL:f1\" placement=\"h\"",
      "A side caption.",
    ),
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    &figure("S0.F2", "2", "", "A wide one."),
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    &format!(
      "<table inlist=\"lot\" labels=\"LABEL:t1\" xml:id=\"S0.T1\">{}<tabular vattach=\"middle\" \
       xml:id=\"S0.T1.1\"><tbody><tr xml:id=\"S0.T1.1.1\"><td align=\"center\" xml:id=\"S0.T1.1.1.1\">x</td></tr>\
       </tbody></tabular><toccaption><tag close=\" \">1</tag>A side table.</toccaption>\
       <caption><tag close=\": \">Table 1</tag>A side table.</caption></table>",
      tags("Table 1", "1")
    ),
  );
}
