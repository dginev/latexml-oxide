//! Red/green guards for batch 78b: `\halign` inserts `\everycr` (tex.web §774 `init_align`, §799 `fin_row`;
//! KNOWN_PERL_ERRORS #1019), then AMS-TeX's active `@` (KNOWN_PERL_ERRORS #1018). Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the authored content in whole elements, against
//! pdftex/pdflatex. Witnesses math0005042 (lamstex `\newCD` counts its rows with `\everycr`), math0004017,
//! math0607639, math0009033 (AMS-TeX arrows and `@!`).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings, and each `(tag, xml:id, element)` as given.
fn assert_clean(tex: &str, expected: &[(&str, &str, &str)]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (tag, id, element) in expected {
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
  xml
}

/// `\everycr` runs after the preamble's `\cr` and after every row's (pdftex "count: 3"). Red (int40c, Perl alike):
/// "count: 0". Repro kernel-alignment/everycr_runs_after_every_cr (witness math0005042).
#[test]
fn everycr_runs_after_the_preamble_and_every_row() {
  assert_clean(
    repro!("kernel-alignment/everycr_runs_after_every_cr.tex"),
    &[("p", "id1.1", "<p xml:id=\"id1.1\">count: 3</p>")],
  );
}

/// A `\crcr` right after a `\cr` ends no row, so it inserts no `\everycr` (tex.web §785 `align_peek` skips it; pdftex
/// "count: 3"). Red (int40c): "count: 0". Repro kernel-alignment/everycr_skips_a_redundant_crcr.
#[test]
fn everycr_skips_a_redundant_crcr() {
  assert_clean(
    repro!("kernel-alignment/everycr_skips_a_redundant_crcr.tex"),
    &[("p", "id1.1", "<p xml:id=\"id1.1\">count: 3</p>")],
  );
}

/// An `\everycr` a cell sets locally ends with the cell, before the row's `\everycr` is inserted (§791 `fin_col`); a
/// `\global` one runs after that row and the next (pdftex "local: 0, global: 2"). Red (int40c): "global: 0". Repro
/// kernel-alignment/everycr_set_in_a_cell_ends_with_it.
#[test]
fn everycr_set_in_a_cell_ends_with_the_cell() {
  assert_clean(
    repro!("kernel-alignment/everycr_set_in_a_cell_ends_with_it.tex"),
    &[("p", "id1.1", "<p xml:id=\"id1.1\">local: 0, global: 2</p>")],
  );
}

/// Each of two nested `\halign`s inserts the `\everycr` current in its own group (pdftex "outer: 3, inner: 4"). Red
/// (int40c): "outer: 0, inner: 0". Repro kernel-alignment/everycr_nested_haligns_use_their_own.
#[test]
fn nested_haligns_run_their_own_everycr() {
  assert_clean(
    repro!("kernel-alignment/everycr_nested_haligns_use_their_own.tex"),
    &[("p", "id1.1", "<p xml:id=\"id1.1\">outer: 3, inner: 4</p>")],
  );
}

/// `\everycr{\noalign{\hrule}}` rules the alignment above its first row, between the rows and below the last, as
/// pdftex draws three rules. Red (int40c): no border. Repro kernel-alignment/everycr_noalign_rule_after_every_row.
#[test]
fn everycr_noalign_rule_rules_every_row() {
  assert_clean(
    repro!("kernel-alignment/everycr_noalign_rule_after_every_row.tex"),
    &[(
      "tabular",
      "id1.1",
      "<tabular xml:id=\"id1.1\"><tr xml:id=\"id1.1.1\"><td align=\"left\" border=\"t\" class=\"ltx_nopad_l\" \
       xml:id=\"id1.1.1.1\">a</td><td align=\"left\" border=\"t\" class=\"ltx_nopad_l ltx_nopad_r\" \
       xml:id=\"id1.1.1.2\">b</td></tr><tr xml:id=\"id1.1.2\"><td align=\"left\" border=\"b t\" class=\"ltx_nopad_l\" \
       xml:id=\"id1.1.2.1\">c</td><td align=\"left\" border=\"b t\" class=\"ltx_nopad_l ltx_nopad_r\" \
       xml:id=\"id1.1.2.2\">d</td></tr></tabular>",
    )],
  );
}

/// A LaTeX `tabular` starts with `\ialign`'s `\everycr{}` (latex.ltx:631, :16573), so a document's `\everycr` runs only
/// in the `\halign` after it (pdflatex "rows: 2"), and the tabular's rows stay unruled. Red (int40c): "rows: 0". Repro
/// kernel-alignment/everycr_latex_tabular_resets_it.
#[test]
fn a_latex_tabular_runs_no_everycr() {
  assert_clean(
    repro!("kernel-alignment/everycr_latex_tabular_resets_it.tex"),
    &[
      (
        "tabular",
        "p1.1",
        "<tabular vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr xml:id=\"p1.1.1\"><td align=\"left\" \
         xml:id=\"p1.1.1.1\">a</td></tr><tr xml:id=\"p1.1.2\"><td align=\"left\" xml:id=\"p1.1.2.1\">b</td></tr>\
         </tbody></tabular>",
      ),
      ("p", "p1.3", "<p xml:id=\"p1.3\">rows: 2</p>"),
    ],
  );
}

/// An `\halign` nested in a `tabular` cell finds `\everycr` empty: the table clears it in its group, as `\@array`'s
/// `\ialign` does (latex.ltx:631, :16573; pdflatex "rows: 0"). Red (78b r1): "rows: 3", the document-level `\everycr`
/// reaching the nested `\halign`. Repro kernel-alignment/everycr_cleared_in_a_bound_alignment (the rev78b review's
/// finding 2).
#[test]
fn a_bound_alignment_clears_everycr_for_what_it_holds() {
  assert_clean(
    repro!("kernel-alignment/everycr_cleared_in_a_bound_alignment.tex"),
    &[("p", "p1.2", "<p xml:id=\"p1.2\">rows: 0</p>")],
  );
}

/// AMS-TeX's active `@` reads the connectors amstex.tex's `\atdef@` defines, in any formula (math0004017's `@>f>>`):
/// a rightarrow with f over it and a leftarrow, no "@". Red (int40c): "The token T_ACTIVE[@] is not defined" per
/// arrow, "@>>>" printed. Repro amstex/amstex_active_at_reads_the_arrows; KNOWN_PERL_ERRORS #1018.
/// (The whole tree is the math parser's current reading of the arrow, a factor of a product; math is parked, so a
/// future math fix changes it knowingly.)
#[test]
fn amstex_active_at_reads_the_arrows() {
  assert_clean(repro!("amstex/amstex_active_at_reads_the_arrows.tex"), &[(
    "equation",
    "equationx1",
    "<equation xml:id=\"equationx1\"><Math mode=\"display\" tex=\"A@&gt;{f}&gt;{}&gt;B@&lt;{}&lt;{}&lt;C\" text=\"A \
       * rightarrow ^ f * B leftarrow C\" xml:id=\"equationx1.m1\"><XMath xml:id=\"equationx1.m1.1\"><XMApp \
       xml:id=\"equationx1.m1.1.1\"><XMTok class=\"ltx_horizontally_stretchy\" meaning=\"leftarrow\" role=\"ARROW\" \
       stretchy=\"true\" width=\"30.0pt\">←</XMTok><XMApp xml:id=\"equationx1.m1.1.1.2\"><XMTok meaning=\"times\" \
       role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">A</XMTok><XMApp \
       xml:id=\"equationx1.m1.1.1.2.3\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"mid1\"/><XMTok \
       class=\"ltx_horizontally_stretchy\" meaning=\"rightarrow\" role=\"ARROW\" stretchy=\"true\" \
       width=\"30.0pt\">→</XMTok><XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">f</XMTok></XMApp><XMTok \
       font=\"italic\" role=\"UNKNOWN\">B</XMTok></XMApp><XMTok font=\"italic\" role=\"UNKNOWN\">C</XMTok></XMApp>\
       </XMath></Math></equation>",
  )]);
}
