//! Red/green guards for batch 68b (binding gaps G3, core package bindings): the macros and environments that
//! hyperref, natbib, breqn, url, blkarray, pict2e and amsthm define and papers call, which their bindings lacked.
use latexml::util::test::{assert_element, convert_files_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> (String, String) { convert_files_with(tex, &[], Some("ar5iv.sty")) }

/// The whole `<tag attrs>` element, whitespace-normalized, as `assert_element` compares it: for an element whose
/// opening part (start tag, tags, first child) is the assertion and whose formulas are pinned elsewhere.
fn element(xml: &str, tag: &str, attrs: &[&str]) -> String {
  xml_element(xml, tag, attrs)
    .unwrap_or_else(|| panic!("no <{tag} {}> element\n{xml}", attrs.join(" ")))
}

/// hyperref internals papers call (hyperref.sty:130-135, :2078-2090, :6312-6314, :6655-6658, :6727-6733, :7443,
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
    element(&xml, "equation", &["xml:id=\"S0.E1\""]).starts_with(
      "<equation labels=\"LABEL:eq:one\" xml:id=\"S0.E1\"><tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>\
       <Math mode=\"display\" tex=\"a=b+c\""
    ),
    "{xml}"
  );
  // `{dgroup}`: the subequation group (2) of (2a)/(2b)
  assert!(
    element(&xml, "equationgroup", &["xml:id=\"S0.E2\""]).starts_with(
      "<equationgroup xml:id=\"S0.E2\"><tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags><equation \
       xml:id=\"S0.E2.1\">"
    ),
    "{xml}"
  );
  for (id, num, var, val) in [("S0.E2.1", "2a", "x", "1"), ("S0.E2.2", "2b", "y", "2")] {
    assert_element(
      &xml,
      "equation",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<equation xml:id=\"{id}\"><tags><tag>({num})</tag><tag role=\"refnum\">{num}</tag></tags><Math \
         mode=\"display\" tex=\"{var}={val}\" text=\"{var} = {val}\" xml:id=\"{id}.m1\"><XMath \
         xml:id=\"{id}.m1.1\"><XMApp xml:id=\"{id}.m1.1.1\"><XMTok meaning=\"equals\" \
         role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">{var}</XMTok><XMTok meaning=\"{val}\" \
         role=\"NUMBER\">{val}</XMTok></XMApp></XMath></Math></equation>"
      ),
    );
  }
  // `{dgroup*}`: an unnumbered group, its equations unnumbered, the `{dsuspend}` text a paragraph of the group
  assert!(
    element(&xml, "equationgroup", &["xml:id=\"S0.EGx1\""]).starts_with(
      "<equationgroup xml:id=\"S0.EGx1\"><equation xml:id=\"S0.Ex1\"><Math mode=\"display\""
    ),
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.EGx1.1\""],
    "<p xml:id=\"S0.EGx1.1\">,\n\nand</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.EGx1.2\""],
    "<p xml:id=\"S0.EGx1.2\">.</p>",
  );
  assert_eq!(xml.matches("<equationgroup ").count(), 2, "{xml}");
  // `{dseries}`: one numbered display of in-line formulas and the text between; `{dseries*}` unnumbered
  assert!(
    element(&xml, "equation", &["xml:id=\"S0.E3\""]).starts_with(
      "<equation xml:id=\"S0.E3\"><tags><tag>(3)</tag><tag role=\"refnum\">3</tag></tags><Math mode=\"inline\" \
       tex=\"u=e^{ix}\""
    ),
    "{xml}"
  );
  assert!(
    element(&xml, "text", &["xml:id=\"S0.E3.1\""])
      .starts_with("<text xml:id=\"S0.E3.1\">and<Math mode=\"inline\" tex=\"v=e^{-ix}\""),
    "{xml}"
  );
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S0.Ex3\""],
    "<equation xml:id=\"S0.Ex3\"><Math mode=\"inline\" tex=\"w=0\" text=\"w = 0\" xml:id=\"S0.Ex3.m1\"><XMath \
     xml:id=\"S0.Ex3.m1.1\"><XMApp xml:id=\"S0.Ex3.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok \
     font=\"italic\" role=\"UNKNOWN\">w</XMTok><XMTok meaning=\"0\" role=\"NUMBER\">0</XMTok></XMApp></XMath></Math>\
     </equation>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.5\""],
    "<p xml:id=\"p1.5\">See (<ref labelref=\"LABEL:eq:one\"/>).</p>",
  );
}

/// breqn's group options and display parameters (68b r2): `{dgroup}` reads its options as the breqn keys
/// (breqn.sty:1793 `\setkeys{breqn}{#1}`), so a `label` names the group (:1808; 2607.02409's four
/// `\begin{dgroup}[label=…]`); a `label` takes a pending `labelprefix` once (:383-388); the parameters have breqn's
/// types and values (:179-198: `\eqbreakdepth` a count, `\eqindent` the macro `C`, `\eqbinoffset`/`\eqdelimoffset`
/// muskips, `\eqstyle` a token list). pdflatex: "[2] [C] [10000] [15.0mu minus -3.0mu] [2.0mu] [] [5]", "See 1, 2
/// and 3.". Repro package-bindings/breqn_group_label_and_parameters.
#[test]
fn breqn_group_label_and_parameters() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/breqn_group_label_and_parameters.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("breqn.sty is minimally stubbed"), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">[2] [C] [10000] [15.0mu minus -3.0mu] [2.0mu] []\n[5]</p>",
  );
  assert!(
    element(&xml, "equationgroup", &["xml:id=\"S0.E1\""]).starts_with(
      "<equationgroup labels=\"LABEL:grp\" xml:id=\"S0.E1\"><tags><tag>(1)</tag><tag \
       role=\"refnum\">1</tag></tags><equation xml:id=\"S0.E1.1\"><tags><tag>(1a)</tag>"
    ),
    "{xml}"
  );
  for (id, label, num) in [("S0.E2", "eq:one", "2"), ("S0.E3", "two", "3")] {
    assert!(
      element(&xml, "equation", &[&format!("xml:id=\"{id}\"")]).starts_with(&format!(
        "<equation labels=\"LABEL:{label}\" xml:id=\"{id}\"><tags><tag>({num})</tag><tag \
         role=\"refnum\">{num}</tag></tags>"
      )),
      "{xml}"
    );
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.2\""],
    "<p xml:id=\"p1.2\">See <ref labelref=\"LABEL:grp\"/>, <ref labelref=\"LABEL:eq:one\"/> and <ref \
     labelref=\"LABEL:two\"/>.</p>",
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
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">See <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a\" separator=\",\" \
     show=\"Number\" yyseparator=\",\"/>]</cite> and [Demidovich, p.\u{a0}66].</p>",
  );
  // `numbers`: the author-year label `\dinatlabel` would print is not typeset
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> F.\u{a0}Abbors. A paper.</bibblock>",
  );
  // author-year: the label is typeset, and `\NAT@force@numbers` is natbib's error (natbib.sty:974-981), as
  // pdflatex raises it; the citations made before it stay author-year
  let (log, xml) = convert(&tex.replace("[numbers]{natbib}", "{natbib}"));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains("Package natbib Error: Bibliography not compatible with author-year citations"),
    "{log}"
  );
  assert!(
    element(&xml, "p", &["xml:id=\"p1.1\""]).ends_with("</cite> and (Demidovich, p.\u{a0}66).</p>"),
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
  assert_element(
    &xml,
    "text",
    &["xml:id=\"p1.1.1\""],
    "<text xml:id=\"p1.1.1\" yoffset=\"1.9pt\"><Math mode=\"inline\" tex=\"\\scriptstyle\\sim\" text=\"similar-to\" \
     xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMTok fontsize=\"70%\" meaning=\"similar-to\" name=\"sim\" \
     role=\"RELOP\">∼</XMTok></XMath></Math></text>",
  );
  assert!(
    element(&xml, "p", &["xml:id=\"p1.1\""])
      .ends_with("now. http://www.math.princeton.edu/<text xml:id=\"p1.1.1\" yoffset=\"1.9pt\"><Math \
       mode=\"inline\" tex=\"\\scriptstyle\\sim\" text=\"similar-to\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMTok \
       fontsize=\"70%\" meaning=\"similar-to\" name=\"sim\" role=\"RELOP\">∼</XMTok></XMath></Math></text>\
       hossein/contents/pseudo8.pdf</p>"),
    "{xml}"
  );
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
  // the sector closes once at its centre (the filled polyline closes itself; `\pIIe@closepath` adds no second point)
  assert_element(
    &xml,
    "line",
    &["fill=\"#000000\""],
    "<line fill=\"#000000\" points=\"0,0 19.69,0 19.44,3.08 18.72,6.08 17.54,8.94 15.93,11.57 13.92,13.92 0,0\" \
     stroke=\"none\" stroke-width=\"0.4\"/>",
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
