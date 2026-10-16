//! Red/green guards for batch 68b (binding gaps G3, core package bindings): the macros and environments that
//! hyperref, natbib, breqn, url, blkarray, pict2e and amsthm define and papers call, which their bindings lacked.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> (String, String) { convert_files_with(tex, &[], Some("ar5iv.sty")) }

/// hyperref internals papers call (hyperref.sty:130-135, :2078-2090, :6312-6314, :6655-6658, :6735-6742, :7443,
/// :7886-7901, :8019-8068, :8126-8134; hpdftex.def:396; psdextra.def:75): `\Hy@Info` (1310.6851), the raised-link
/// hooks and space-factor pair (2403.10429, 2510.01943), `\@skiphyperreftrue` (1806.02596), `\thispdfpagelabel`
/// (1810.05756), `\hyper@refstepcounter` (2102.05485), `\hyper@link` with `\@extra@b@citeb` (1403.6472),
/// `\hyperpage`/`\hyperindexformat` in an index (0712.1142, 1011.1558), `\psdmapshortnames` under `psdextra`
/// (1801.02767) and `\@refstar` (2204.02601). Repro package-bindings/hyperref_internals_called_by_papers.
#[test]
fn hyperref_internals_called_by_papers() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/hyperref_internals_called_by_papers.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p1.1\""],
    "<p xml:id=\"S1.p1.1\">See <ref idref=\"cite.knuth\">Knuth’s book</ref> and <text \
     xml:id=\"S1.p1.1.1\">word</text>.\nSection <ref class=\"ltx_nolink\" labelref=\"LABEL:s1\"/>.</p>",
  );
  for (id, entry) in [
    (
      "idx.1.1",
      "<indexphrase key=\"absolute value, 95, 101\">absolute value, 95, 101\n</indexphrase>",
    ),
    (
      "idx.1.2",
      "<indexphrase key=\"range, 3–5\">range, 3–5\n</indexphrase>",
    ),
    (
      "idx.1.3",
      "<indexphrase key=\"list, 7, 9\">list, 7, 9\n</indexphrase>",
    ),
    (
      "idx.1.4",
      "<indexphrase key=\"bold, 42\">bold, <text font=\"bold\" \
       xml:id=\"idx.1.4.1\">42</text>\n</indexphrase>",
    ),
  ] {
    assert_element(
      &xml,
      "indexentry",
      &[&format!("xml:id=\"{id}\"")],
      &format!("<indexentry xml:id=\"{id}\">{entry}</indexentry>"),
    );
  }
}

/// breqn's display environments (breqn.sty:441-472, :1724-1784, :2027-2041) and parameters (:179-198, :383-436):
/// `{dmath}` with a `label`, `{dgroup}` numbered as subequations, `{dgroup*}` an unnumbered group with the
/// `{dsuspend}` text, `{dseries}`/`{dseries*}` a display of in-line formulas (0903.2087, 1111.2451, 1606.07330,
/// 1201.6197, 1706.02708, 2008.13194, 1805.03313). The one warning is the stub's own notice. Repro
/// package-bindings/breqn_display_environments.
#[test]
fn breqn_display_environments() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/breqn_display_environments.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("breqn.sty is minimally stubbed"), "{log}");
  assert!(
    xml.contains("<equation labels=\"LABEL:eq:one\" xml:id=\"S0.E1\">"),
    "{xml}"
  );
  for tags in [
    "<equationgroup xml:id=\"S0.E2\">\n      <tags>\n        <tag>(2)</tag>",
    "<equation xml:id=\"S0.E2.1\">\n        <tags>\n          <tag>(2a)</tag>",
    "<equation xml:id=\"S0.E2.2\">\n        <tags>\n          <tag>(2b)</tag>",
    "<equation xml:id=\"S0.E3\">\n      <tags>\n        <tag>(3)</tag>",
  ] {
    assert!(xml.contains(tags), "{tags}\n{xml}");
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.EGx1.1\""],
    "<p xml:id=\"S0.EGx1.1\">,\n\nand</p>",
  );
  assert_eq!(xml.matches("<equationgroup ").count(), 2, "{xml}");
  // the `{dseries}` formulas stay in-line formulas of the one display
  assert!(xml.contains("<text xml:id=\"S0.E3.1\">and\n"), "{xml}");
  assert!(
    xml.contains("<p xml:id=\"p1.5\">See (<ref labelref=\"LABEL:eq:one\"/>).</p>"),
    "{xml}"
  );
}

/// natbib's bibliography-label and list internals (natbib.sty:622-645, :658-668, :974-981) and its punctuation
/// (:239-260): `\NAT@biblabelnum` under `\ifNAT@numbers`, which the `numbers` option sets (1605.02776),
/// `\@bibsetup` in a document's own list (1102.3365), `\NAT@open`/`\NAT@close` (1906.11206),
/// `\NAT@force@numbers` (2508.10404). Repro package-bindings/natbib_label_and_list_internals.
#[test]
fn natbib_label_and_list_internals() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/natbib_label_and_list_internals.tex"
  );
  let (log, xml) = convert(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("</cite> and [Demidovich, p.\u{a0}66].</p>"),
    "{xml}"
  );
  // `numbers`: the author-year label `\dinatlabel` would print is not typeset
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> F.\u{a0}Abbors. A paper.</bibblock>",
  );
  // author-year: the label is typeset, and `\NAT@force@numbers` reports the numeric bibliography
  let (log, xml) = convert(&tex.replace("[numbers]{natbib}", "{natbib}"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Bibliography not compatible with author-year citations"),
    "{log}"
  );
  assert!(
    xml.contains("</cite> and (Demidovich, p.\u{a0}66).</p>"),
    "{xml}"
  );
}

/// url.sty:63 `\UrlTildeSpecial` and :127-133 `\Url@FormatString` over a document's `\Url@String` (the JOSS/pandoc
/// `\href@Urllike`, 1902.08702; 0907.0729). Repro package-bindings/url_format_string_and_tilde.
#[test]
fn url_format_string_and_tilde() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/url_format_string_and_tilde.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "ref",
    &["class=\"ltx_href\""],
    "<ref class=\"ltx_href\" font=\"typewriter\" \
     href=\"https://doi.org/10.1/abc\">https://doi.org/10.1/abc</ref>",
  );
  assert!(
    xml.contains("</text>hossein/contents/pseudo8.pdf</p>"),
    "{xml}"
  );
  assert!(xml.contains("tex=\"\\scriptstyle\\sim\""), "{xml}");
}

/// blkarray.sty:934-947 `\BAmulticolumn`, :1677-1683 `\BAnewcolumntype`, :1063 `\BAextrarowheight`, :1005-1012
/// `\BAnoalign` (1401.4921, 1302.2340, 1312.6738, 1912.05110). Repro package-bindings/blkarray_public_commands.
#[test]
fn blkarray_public_commands() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/blkarray_public_commands.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<XMRow ").count(), 3, "{xml}");
  assert_element(
    &xml,
    "XMCell",
    &["colspan=\"2\""],
    "<XMCell align=\"center\" colspan=\"2\" xml:id=\"S0.Ex1.m1.1.1.2.4\"><XMTok meaning=\"0\" \
     role=\"NUMBER\">0</XMTok></XMCell>",
  );
  // a `\BAnewcolumntype` letter is read through the alignment template's rewrite
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{blkarray}\\BAnewcolumntype{s}{r}\\begin{document}\n\
     $\\begin{blockarray}{sc}a&b\\\\\\end{blockarray}$\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<XMCell align=\"right\""), "{xml}");
}

/// pict2e.sty:667-684 `\arc`/`\arc*`: a stroked circle, a quarter arc, and the filled sector from its centre (the mode
/// 0 `\pIIe@arc` draws a line to the arc's start, :643-646). Witnesses 1201.4907, 1503.03277. Repro
/// package-bindings/pict2e_arc.
#[test]
fn pict2e_arc_and_sector() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/pict2e_arc.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<line ").count(), 3, "{xml}");
  assert!(
    xml.contains(
      "<line fill=\"#000000\" points=\"0,0 19.69,0 19.44,3.08 18.72,6.08 17.54,8.94 15.93,11.57 13.92,13.92 \
       0,0 0,0\" stroke=\"none\" stroke-width=\"0.4\"/>"
    ),
    "{xml}"
  );
}

/// amsthm.sty:294-295 `\ifmeasuring@` and `\iffirstchoice@`, which setstack.sty's copy of amstext's `\text@` sets
/// (0803.1550, 0806.2059). Repro package-bindings/amsthm_firstchoice_conditional.
#[test]
fn amsthm_firstchoice_conditional() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/amsthm_firstchoice_conditional.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<XMText xml:id=\"p1.m1.1.1.3.2\">for all </XMText>"),
    "{xml}"
  );
}
