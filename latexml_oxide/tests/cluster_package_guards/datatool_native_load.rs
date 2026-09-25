//! The native `\DTLloaddb` CSV load (`datatool_sty.rs`): the raw
//! `datatool.sty` loads whole and the default CSV load writes the four
//! database registers the way datatool's own reload does. The harness
//! converts each fixture with the native ON and OFF (`LATEXML_DATATOOL_NATIVE=0`)
//! and requires byte-identical core XML; the typing golden is pdflatex's.
//! The fixtures name their CSV relative to the crate root, the test cwd.

fn both_ways(fixture: &str) -> String {
  let tex = std::fs::read_to_string(format!("tests/cluster_regressions/datatool/{fixture}.tex"))
    .expect("fixture");
  unsafe { std::env::remove_var("LATEXML_DATATOOL_NATIVE") };
  let (stderr_on, on) = super::convert(&tex, true);
  assert_eq!(super::error_count(&stderr_on), 0, "native ON: {stderr_on}");
  unsafe { std::env::set_var("LATEXML_DATATOOL_NATIVE", "0") };
  let (stderr_off, off) = super::convert(&tex, true);
  unsafe { std::env::remove_var("LATEXML_DATATOOL_NATIVE") };
  assert_eq!(
    super::error_count(&stderr_off),
    0,
    "native OFF: {stderr_off}"
  );
  assert_eq!(
    on, off,
    "native and raw loads must give byte-identical XML for {fixture}"
  );
  on
}

/// A quoted field with an embedded separator, an empty field, an interior
/// quote, a blank line, spaces trimmed inside and outside quotes; row and
/// column counts, the key index, the last-loaded name.
#[test]
fn default_csv_load_matches_the_raw_engine() {
  let xml = both_ways("contract");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>M:4/3/2/t. R:4. [Smith, J.—12—item four][Doe—3.5—][Roe—-7—a ”b” c][Poe—1000—spaced] [pct:50% off][tie:a b][cmd:<emph font="italic">x</emph>]</p>"##,
  );
}

/// `\\DTLifeq` on strings (datatool-base.sty:9075/:9052): exact, spaces kept,
/// folded under `*`; `\\DTLifstringeq` never goes numeric; inside a raw
/// `\\DTLforeach*` + `\\DTLforeachkeyinrow` walk (tikz-network's shape,
/// tikz-network.sty:804-816) the loop state is untouched.
#[test]
fn string_comparisons_match_the_raw_engine() {
  let xml = both_ways("ifeq_strings");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>A:YNYYNNY. B:(12)[hit:3.5](3.5)(-7)(1000).</p>"##,
  );
}

/// Both operands numeric: an fp equality (`\\DTLifnumeq`, datatool-base.sty:8685).
/// The raw path in this engine types `5` as a string and answers N to all
/// five; the golden is pdflatex's, so this runs the native comparator only.
#[test]
fn numeric_comparisons_follow_pdflatex() {
  let tex = std::fs::read_to_string("tests/cluster_regressions/datatool/ifeq_numbers.tex")
    .expect("fixture");
  unsafe { std::env::remove_var("LATEXML_DATATOOL_NATIVE") };
  let (stderr, xml) = super::convert(&tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>C:YYYYN.</p>"##);
}

/// A data row wider than the header (a trailing separator): the uncovered
/// column's key and header are `Column<n>` (datatool.sty:12305,
/// `\dtldefaultkey` :10844) and it has an index — three keys entries.
/// The golden is pdflatex's (the raw path here types the columns 0).
#[test]
fn ragged_rows_get_default_column_keys() {
  let tex =
    std::fs::read_to_string("tests/cluster_regressions/datatool/ragged.tex").expect("fixture");
  unsafe { std::env::remove_var("LATEXML_DATATOOL_NATIVE") };
  let (stderr, xml) = super::convert(&tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>M:2/3/3. K:“db@plist@elt@w “db@col@id@w 1“db@col@id@end@ “db@key@id@w a“db@key@id@end@ “db@type@id@w 1“db@type@id@end@ “db@header@id@w a“db@header@id@end@ “db@col@id@w 1“db@col@id@end@ “db@plist@elt@end@ “db@plist@elt@w “db@col@id@w 2“db@col@id@end@ “db@key@id@w b“db@key@id@end@ “db@type@id@w 1“db@type@id@end@ “db@header@id@w b“db@header@id@end@ “db@col@id@w 2“db@col@id@end@ “db@plist@elt@end@ “db@plist@elt@w “db@col@id@w 3“db@col@id@end@ “db@key@id@w Column3“db@key@id@end@ “db@type@id@w 0“db@type@id@end@ “db@header@id@w Column3“db@header@id@end@ “db@col@id@w 3“db@col@id@end@ “db@plist@elt@end@</p>"##,
  );
}

/// Options and a second load of a name go to the raw `\DTLread`.
#[test]
fn options_and_reloads_stay_raw() {
  let xml = both_ways("tiny");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>A:1/2. [name—qty][a—1]</p>"##);
}

/// The column type is the maximum datum type over the non-empty cells
/// (datatool.sty:12562-12585, datatool-base.sty:3489): `qty` decimal. The
/// golden is pdflatex's register dump; the raw path here types every
/// column string, so this test runs the native load only.
#[test]
fn column_types_follow_the_datum_parser() {
  let tex =
    std::fs::read_to_string("tests/cluster_regressions/datatool/types.tex").expect("fixture");
  unsafe { std::env::remove_var("LATEXML_DATATOOL_NATIVE") };
  let (stderr, xml) = super::convert(&tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>K:“db@plist@elt@w “db@col@id@w 1“db@col@id@end@ “db@key@id@w name“db@key@id@end@ “db@type@id@w 0“db@type@id@end@ “db@header@id@w name“db@header@id@end@ “db@col@id@w 1“db@col@id@end@ “db@plist@elt@end@ “db@plist@elt@w “db@col@id@w 2“db@col@id@end@ “db@key@id@w qty“db@key@id@end@ “db@type@id@w 2“db@type@id@end@ “db@header@id@w qty“db@header@id@end@ “db@col@id@w 2“db@col@id@end@ “db@plist@elt@end@ “db@plist@elt@w “db@col@id@w 3“db@col@id@end@ “db@key@id@w note“db@key@id@end@ “db@type@id@w 0“db@type@id@end@ “db@header@id@w note“db@header@id@end@ “db@col@id@w 3“db@col@id@end@ “db@plist@elt@end@</p>"##,
  );
}
