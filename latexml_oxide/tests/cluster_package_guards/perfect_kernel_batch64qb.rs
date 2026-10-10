//! Red/green guards for perfect-kernel batch 64qb: binding gaps behind run-336 first errors (paspconf's RevTeX-3
//! frontmatter, caption2's interface, amsmath's accent internals) and three roots of stray `&`/`_`/`^`
//! (`\tablebreak`, `\sidecaption`, jmlr2e's `\keywords` as a command).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// paspconf.sty is the PASP conference substyle of AAS RevTeX 3: `\affil` (L72), `\altaffilmark`/`\altaffiltext`
/// (L77-78) through the aaspp binding, `\deg` the degree sign (L228), keyless references (L188), the conference
/// discussion (L163-170), and keywords the PDF does not print (L121) kept as metadata (astro-ph9712155,
/// astro-ph9512088, astro-ph9909358). Repro sectioning-frontmatter/paspconf_revtex3_frontmatter.
#[test]
fn paspconf_revtex3_frontmatter() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/paspconf_revtex3_frontmatter.tex"
  ));
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>A. Author</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Some Institute, Some City</contact><contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">Visiting astronomer.</contact></creator>",
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"abstract1.m1\""],
    "<Math mode=\"inline\" tex=\"30\\arcdeg\" text=\"30 * °\" xml:id=\"abstract1.m1\"><XMath xml:id=\"abstract1.m1.1\"><XMApp xml:id=\"abstract1.m1.1.1\"><XMTok meaning=\"times\" role=\"MULOP\">⁢</XMTok><XMTok meaning=\"30\" role=\"NUMBER\">30</XMTok><XMTok role=\"UNKNOWN\">°</XMTok></XMApp></XMath></Math>",
  );
  // RevTeX 3's `\reference` takes no key (paspconf.sty:188): each entry is whole, a math `\O` included.
  assert_element(
    &xml,
    "biblist",
    &[],
    "<biblist><bibitem xml:id=\"bib.bib1\"><tags><tag role=\"number\">1</tag><tag role=\"refnum\">(1)</tag></tags><bibblock>Kent, S. M. 1988, AJ, 96, 1570</bibblock></bibitem><bibitem xml:id=\"bib.bib2\"><tags><tag role=\"number\">2</tag><tag role=\"refnum\">(2)</tag></tags><bibblock><Math mode=\"inline\" tex=\"\\O \" text=\"Ø\" xml:id=\"bib.bib2.m1\"><XMath xml:id=\"bib.bib2.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">Ø</XMTok></XMath></Math>stensen, R. et al. 1996, A&amp;A, 309, 59</bibblock></bibitem></biblist>",
  );
  assert_element(
    &xml,
    "section",
    &["xml:id=\"Sx1\""],
    "<section xml:id=\"Sx1\"><title>Discussion</title><para class=\"ltx_noindent\" xml:id=\"Sx1.p1\"><p xml:id=\"Sx1.p1.1\"><text font=\"italic\" xml:id=\"Sx1.p1.1.1\">B. Asker</text>:Why?</p></para><para class=\"ltx_noindent\" xml:id=\"Sx1.p2\"><p xml:id=\"Sx1.p2.1\"><text font=\"italic\" xml:id=\"Sx1.p2.1.1\">A. Author</text>:Because.</p></para></section>",
  );
  assert_element(
    &xml,
    "rdf",
    &[],
    "<rdf about=\"\" content=\"stars, disks\" property=\"dcterms:subject\"/>",
  );
  assert_eq!(xml.matches("<classification").count(), 0, "{xml}");
}

/// caption2.sty's interface (style registry :97-142, parameters, options :179-229) is layout, taken; the label's
/// delimiter and separator (:147) close the caption tag — `.`, w-art.cls's `\enskip` (1012.0703) and a document's
/// `~~~` (math0601389, 1307.4815).
/// Repro captions-floats/caption2_interface.
#[test]
fn caption2_interface() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption2_interface.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    "<figure inlist=\"lof\" labels=\"LABEL:fig:a\" placement=\"h\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><p align=\"center\" xml:id=\"S0.F1.1\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F1.1.1\">Picture</text></p><toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>A caption</toccaption><caption class=\"ltx_centering\"><tag close=\". \">Figure 1</tag>A caption</caption></figure>",
  );
  // A skip in the delimiter (w-art.cls:3091 `\enskip`) is its spaces in the attribute, not its TeX.
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    "<figure inlist=\"lof\" placement=\"h\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><p xml:id=\"S0.F2.1\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F2.1.1\">Second</text></p><toccaption><tag close=\" \">2</tag>Two</toccaption><caption><tag close=\"\u{2002} \">Figure 2</tag>Two</caption></figure>",
  );
  // `\hskip` on both format branches (`\enskip` is plain.tex's `\hskip` in the dump, a primitive box without it):
  // the close joins the list's items, the skip by its attribute form.
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F3\""],
    "<figure inlist=\"lof\" placement=\"h\" xml:id=\"S0.F3\"><tags><tag>Figure 3</tag><tag role=\"refnum\">3</tag><tag role=\"typerefnum\">Figure 3</tag></tags><p xml:id=\"S0.F3.1\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F3.1.1\">Third</text></p><toccaption><tag close=\" \">3</tag>Three</toccaption><caption><tag close=\".\u{2002} \">Figure 3</tag>Three</caption></figure>",
  );
  assert_element(
    &xml,
    "table",
    &["xml:id=\"S0.T1\""],
    "<table inlist=\"lot\" placement=\"h\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Numbers</toccaption><caption><tag close=\"\u{a0}\u{a0}\u{a0}\">Table 1</tag>Numbers</caption><tabular vattach=\"middle\" xml:id=\"S0.T1.1\"><tbody><tr xml:id=\"S0.T1.1.1\"><td align=\"center\" xml:id=\"S0.T1.1.1.1\">1</td></tr></tbody></tabular></table>",
  );
}

/// The copied `\widebar` snippet drives amsmath's accent internals (amsmath.sty:832-891): each use is a bar over its
/// argument, and `\skewchar\textfont\@tempa` reads the family number rather than typesetting `\char1` (1205.2794,
/// 2201.03882). Repro macro-state/amsmath_widebar_snippet.
#[test]
fn amsmath_widebar_snippet() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/amsmath_widebar_snippet.tex"
  ));
  assert_element(
    &xml,
    "XMath",
    &["xml:id=\"p1.m1.1\""],
    "<XMath xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok name=\"overline\" role=\"OVERACCENT\" stretchy=\"true\">¯</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">X</XMTok></XMApp></XMath>",
  );
  assert_element(
    &xml,
    "XMath",
    &["xml:id=\"p1.m2.1\""],
    "<XMath xml:id=\"p1.m2.1\"><XMApp xml:id=\"p1.m2.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMApp xml:id=\"p1.m2.1.1.2\"><XMTok name=\"overline\" role=\"OVERACCENT\" stretchy=\"true\">¯</XMTok><XMTok font=\"caligraphic\" role=\"UNKNOWN\">A</XMTok></XMApp><XMTok fontsize=\"70%\" meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath>",
  );
}

/// AASTeX's `\tablebreak` ends the row (aastex701.cls:12652), a next row led by `[` included (astro-ph0604363,
/// 1811.07447). Repro alignment-bindings/aastex_tablebreak_ends_row.
#[test]
fn aastex_tablebreak_ends_row() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/aastex_tablebreak_ends_row.tex"
  ));
  assert_element(
    &xml,
    "tbody",
    &[],
    "<tbody><tr xml:id=\"S0.T1.2.2\"><td align=\"left\" border=\"t\" xml:id=\"S0.T1.2.2.1\">x</td><td align=\"center\" border=\"t\" xml:id=\"S0.T1.2.2.2\">1</td></tr><tr xml:id=\"S0.T1.2.3\"><td align=\"left\" cssstyle=\"padding-bottom: 0.0pt\" xml:id=\"S0.T1.2.3.1\">y</td><td align=\"center\" cssstyle=\"padding-bottom: 0.0pt\" xml:id=\"S0.T1.2.3.2\">2</td></tr><tr xml:id=\"S0.T1.2.4\"><td align=\"left\" xml:id=\"S0.T1.2.4.1\">[Fe/H]</td><td align=\"center\" xml:id=\"S0.T1.2.4.2\">3</td></tr><tr xml:id=\"S0.T1.2.5\"><td align=\"left\" border=\"b\" xml:id=\"S0.T1.2.5.1\">w</td><td align=\"center\" border=\"b\" xml:id=\"S0.T1.2.5.2\">4</td></tr></tbody>",
  );
}

/// svmult's `\sidecaption` takes only `[pos]` (svmult.cls:1827-1831): the float's body is kept whole (1804.03859).
/// Repro captions-floats/svmult_sidecaption_position_only.
#[test]
fn svmult_sidecaption_position_only() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/svmult_sidecaption_position_only.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"Ch0.F1\""],
    "<figure inlist=\"lof\" placement=\"b\" xml:id=\"Ch0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><p xml:id=\"Ch0.F1.1\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"Ch0.F1.1.1\">my_saddle_plot</text></p><toccaption><tag close=\" \">1</tag>Phase portrait.</toccaption><caption><tag close=\": \">Figure 1</tag>Phase portrait.</caption></figure>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"Ch0.F2\""],
    "<figure inlist=\"lof\" placement=\"b\" xml:id=\"Ch0.F2\"><tags><tag>Figure 2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><p xml:id=\"Ch0.F2.1\"><text cssstyle=\"padding:3.0pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"Ch0.F2.1.1\">second_plot</text></p><toccaption><tag close=\" \">2</tag>Second.</toccaption><caption><tag close=\": \">Figure 2</tag>Second.</caption></figure>",
  );
}

/// jmlr2e's `\keywords{…}` written as a command is the keywords environment around its argument (jmlr2e.sty:151-153);
/// the document after it stays out of the classification (1401.6686). Repro
/// sectioning-frontmatter/jmlr2e_keywords_as_command.
#[test]
fn jmlr2e_keywords_as_command() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jmlr2e_keywords_as_command.tex"
  ));
  assert_element(
    &xml,
    "classification",
    &["scheme=\"keywords\""],
    "<classification scheme=\"keywords\">Message Passing, Decimation</classification>",
  );
  assert_eq!(
    xml.matches("<classification scheme=\"keywords\">").count(),
    2,
    "{xml}"
  );
  assert_element(
    &xml,
    "XMath",
    &["xml:id=\"S1.Ex1.m1.1\""],
    "<XMath xml:id=\"S1.Ex1.m1.1\"><XMApp xml:id=\"S1.Ex1.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMApp xml:id=\"S1.Ex1.m1.1.1.2\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">i</XMTok></XMApp><XMApp xml:id=\"S1.Ex1.m1.1.1.3\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok><XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok></XMApp></XMApp></XMath>",
  );
}
