//! Red/green guards for batch 66e: AMS-TeX's `\tag` (amstex.tex:587-605 in a display; :629-640, 718-760, 797-818,
//! 876-900 in `\align`, `\alignat`, `\gather`), with the markup amsmath's `\tag` gives. Repros
//! `tools/perfect_kernel/repros/amstex/amstex_tag_with_math_text.tex`, `amstex/amstex_align_row_tags.tex`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// AMS-TeX's `\tag#1$$` takes the tag text up to the display's closing `$$` and sets it as text
/// (`\tagform@` = `\hbox{\rm(#1)}`), so a `$'$` in it is inline math, not the display's end: `\let\tag\eqno` (Perl's
/// too, KNOWN_PERL_ERRORS #643) closed the display there ("Missing $", 1012.3760, 1112.1423; `\tag {$*$}` 1108.6264).
/// The tags are amsmath's `\tag{…}` ones (`\tag{1.2$'$}`); a quoted tag is set without parentheses
/// (`\maketag@@@"#1"#2\maketag@{\hbox{\rm#1}}`), the `\tag*` form.
#[test]
fn amstex_display_tag_is_a_text_tag() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_tag_with_math_text.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for tags in [
    "<tags>\n        <tag>(1.2<sup xml:id=\"equationx1.1\"><text font=\"italic\" xml:id=\"equationx1.1.1\">\u{2032}</text></sup>)</tag>\n        <tag role=\"refnum\">1.2<sup xml:id=\"equationx1.2\"><text font=\"italic\" xml:id=\"equationx1.2.1\">\u{2032}</text></sup></tag>\n      </tags>",
    "<tags>\n        <tag>(2)</tag>\n        <tag role=\"refnum\">2</tag>\n      </tags>",
    "<tags>\n        <tag>A</tag>\n        <tag role=\"refnum\">A</tag>\n      </tags>",
  ] {
    assert!(xml.contains(tags), "{tags}\n{xml}");
  }
  // `\tag {$*$}`: the tag's inline math, as in amsmath's `\tag{$*$}`
  for tag in [
    "<tag>(<Math mode=\"inline\" tex=\"*\" text=\"*\" xml:id=\"equationx2.m2\">",
    "<tag role=\"refnum\"><Math mode=\"inline\" tex=\"*\" text=\"*\" xml:id=\"equationx2.m3\">",
  ] {
    assert!(xml.contains(tag), "{tag}\n{xml}");
  }
  assert_eq!(xml.matches("<tags>").count(), 4, "{xml}");
  assert_eq!(xml.matches("<equation ").count(), 4, "{xml}");
  // the displays end where they end: one display Math each
  assert_eq!(xml.matches("<Math mode=\"display\"").count(), 4, "{xml}");
}

/// Inside `\align`, AMS-TeX's `\tag` opens the row's tag cell (`&&`, amstex.tex:633-635): the tag runs to the row's
/// end, `\\` or `\endalign` (1012.3760: `\tag 6.16\\`, `&=\sum_\alpha … \tag 2.1` before `\endalign`), a `$'$` in it
/// is inline math, and the tag is the row's: the lone-`aligned` rearrangement gives each row equation its own.
#[test]
fn amstex_align_row_tags_run_to_the_row_end() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_align_row_tags.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for row in [
    "<equation xml:id=\"equationx1X\">\n        <tags>\n          <tag>(6.16)</tag>\n          <tag role=\"refnum\">6.16</tag>\n        </tags>",
    "<equation xml:id=\"equationx1Xa\">\n        <tags>\n          <tag>(6.17<sup xml:id=\"equationx1Xa.1\"><text font=\"italic\" xml:id=\"equationx1Xa.1.1\">\u{2032}</text></sup>)</tag>\n          <tag role=\"refnum\">6.17<sup xml:id=\"equationx1Xa.2\"><text font=\"italic\" xml:id=\"equationx1Xa.2.1\">\u{2032}</text></sup></tag>\n        </tags>",
    "<equation xml:id=\"equationx1Xb\">\n        <tags>\n          <tag>(2.1)</tag>\n          <tag role=\"refnum\">2.1</tag>\n        </tags>",
  ] {
    assert!(xml.contains(row), "{row}\n{xml}");
  }
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
  assert!(
    xml.contains("<equationgroup xml:id=\"equationx1\">\n      <equation "),
    "{xml}"
  );
  assert!(!xml.contains("_tags"), "{xml}");
}

/// amstex.tex:590-593: `\TagsAsMath` sets the tag in math (`\tagform@{$\m@th#1$}`), `\TagsAsText` back in text; the
/// amsppt binding's no-op stubs hid the pool's switches.
#[test]
fn amstex_tags_as_math_switch() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\documentstyle{amsppt}\n\\document\n\\TagsAsMath\n$$\na=b\\tag 1.2'\n$$\n\\TagsAsText\n$$\nc=d\\tag 3\n$$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<tag>(<Math mode=\"inline\" tex=\"1.2^{\\prime}\""),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<tags>\n        <tag>(3)</tag>\n        <tag role=\"refnum\">3</tag>\n      </tags>"
    ),
    "{xml}"
  );
}
