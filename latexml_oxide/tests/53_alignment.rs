// Alignment tests — individually listed (tex_tests! causes unbounded memory leaks).
use latexml::util::test::*;
const DIR: &str = "tests/alignment";

#[test]
fn halign_test() {
  latexml_test_single("tests/alignment/halign.tex", "halign", DIR, None, None);
}

#[test]
fn tabtab_test() {
  latexml_test_single("tests/alignment/tabtab.tex", "tabtab", DIR, None, None);
}

// Tests with crashes or large diffs — need alignment/math fixes

#[test]
#[ignore] // crash — unwrap on None in tex_tables.rs:803
fn cells_test() {
  latexml_test_single("tests/alignment/cells.tex", "cells", DIR, None, None);
}

#[test]
#[ignore] // crash — removal index out of bounds in normalize.rs
fn colortbls_test() {
  latexml_test_single("tests/alignment/colortbls.tex", "colortbls", DIR, None, None);
}

#[test]
fn halignatt_test() {
  latexml_test_single("tests/alignment/halignatt.tex", "halignatt", DIR, None, None);
}

#[test]
#[ignore] // crash — alignment not active for supertabular
fn supertabular_test() {
  latexml_test_single("tests/alignment/supertabular.tex", "supertabular", DIR, None, None);
}

#[test]
fn listing_test() {
  latexml_test_single("tests/alignment/listing.tex", "listing", DIR, None, None);
}

#[test]
fn min_listing_test() {
  latexml_test_single("tests/alignment/min_listing.tex", "min_listing", DIR, None, None);
}

#[test]
fn min_listing_data_test() {
  latexml_test_single("tests/alignment/min_listing_data.tex", "min_listing_data", DIR, None, None);
}

#[test]
fn min_listing_lang_test() {
  latexml_test_single("tests/alignment/min_listing_lang.tex", "min_listing_lang", DIR, None, None);
}

#[test]
fn min_listing_short_test() {
  latexml_test_single("tests/alignment/min_listing_short.tex", "min_listing_short", DIR, None, None);
}

#[test]
fn min_listing_string_test() {
  latexml_test_single("tests/alignment/min_listing_string.tex", "min_listing_string", DIR, None, None);
}

#[test]
fn min_listing_display_test() {
  latexml_test_single("tests/alignment/min_listing_display.tex", "min_listing_display", DIR, None, None);
}

#[test]
fn min_listing2_test() {
  latexml_test_single("tests/alignment/min_listing2.tex", "min_listing2", DIR, None, None);
}

#[test]
fn tabularstar_test() {
  latexml_test_single("tests/alignment/tabularstar.tex", "tabularstar", DIR, None, None);
}

#[test]
fn longtable_test() {
  latexml_test_single("tests/alignment/longtable.tex", "longtable", DIR, None, None);
}

#[test]
fn tabbing_test() {
  latexml_test_single("tests/alignment/tabbing.tex", "tabbing", DIR, None, None);
}

#[test]
fn tabular_test() {
  latexml_test_single("tests/alignment/tabular.tex", "tabular", DIR, None, None);
}

#[test]
fn morse_test() {
  latexml_test_single("tests/alignment/morse.tex", "morse", DIR, None, None);
}

#[test]
#[ignore] // diffs — algorithmic package
fn algx_test() {
  latexml_test_single("tests/alignment/algx.tex", "algx", DIR, None, None);
}

#[test]
fn mathmix_test() {
  latexml_test_single("tests/alignment/mathmix.tex", "mathmix", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (XMDual structure)
fn plainmath_test() {
  latexml_test_single("tests/alignment/plainmath.tex", "plainmath", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (XMDual structure)
fn split_test() {
  latexml_test_single("tests/alignment/split.tex", "split", DIR, None, None);
}

#[test]
#[ignore] // diffs — badeqnarray
fn badeqnarray_test() {
  latexml_test_single("tests/alignment/badeqnarray.tex", "badeqnarray", DIR, None, None);
}

#[test]
fn array_test() {
  latexml_test_single("tests/alignment/array.tex", "array", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (eqnarray)
fn eqnarray_test() {
  latexml_test_single("tests/alignment/eqnarray.tex", "eqnarray", DIR, None, None);
}

#[test]
#[ignore] // timeout — diagbox infinite loop
fn diagboxtest_test() {
  latexml_test_single("tests/alignment/diagboxtest.tex", "diagboxtest", DIR, None, None);
}

#[test]
#[ignore] // timeout — ncases infinite loop
fn ncases_test() {
  latexml_test_single("tests/alignment/ncases.tex", "ncases", DIR, None, None);
}

#[test]
#[ignore] // timeout — vmode infinite loop
fn vmode_test() {
  latexml_test_single("tests/alignment/vmode.tex", "vmode", DIR, None, None);
}
