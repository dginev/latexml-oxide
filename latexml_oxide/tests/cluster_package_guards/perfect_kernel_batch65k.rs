//! Red/green guards for batch 65k: stub bindings load what their TeX Live packages load (mdframed's
//! kvoptions/xparse/etoolbox/zref-abspage/needspace/color and its `framemethod` package, mathpartir's keyval,
//! backref's kvoptions/kvsetkeys/rerunfilecheck, hyperref's `backref`/`pagebackref` loading backref) and define the
//! public names documents, `.bbl` files and classes use (mciteplus's switches, arydshln's shorthand environments,
//! nag's rule commands, mathpartir's separators), and the class bindings that load those stubs define their own
//! commands (pnas-new `\dates`/`\dropcap`, achemso `\SectionsOn`/`\AbstractOn`/the TOC-graphic lengths). Each stub
//! keeps its one `missing_file` warning, except backref loaded through hyperref's option, whose message is an Info. Repros under `tools/perfect_kernel/repros/stub-dependencies/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and exactly `warnings` warnings, each of them a
/// stub binding's `missing_file` notice.
fn convert_with_stub_warnings(tex: &str, warnings: usize) -> String {
  convert_with_stub_warnings_and_log(tex, warnings).1
}

/// The log and the XML of `tex`, with the checks of [`convert_with_stub_warnings`].
fn convert_with_stub_warnings_and_log(tex: &str, warnings: usize) -> (String, String) {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert_eq!(
    log
      .lines()
      .filter(|l| l.starts_with("Warning:missing_file:"))
      .count(),
    warnings,
    "{log}"
  );
  (log, xml)
}

/// The number of `log` lines that start with `prefix`.
fn lines_starting(log: &str, prefix: &str) -> usize {
  log.lines().filter(|l| l.starts_with(prefix)).count()
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stub-dependencies/",
      $name,
      ".tex"
    ))
  };
}

/// mdframed.sty:154-169: `[framemethod=TikZ]` loads tikz, so the document's `{tikzpicture}` is drawn (witness
/// 2101.02817). Repro stub-dependencies/mdframed_framemethod_tikz_loads_tikz.
#[test]
fn mdframed_framemethod_tikz_loads_tikz() {
  let xml = convert_with_stub_warnings(repro!("mdframed_framemethod_tikz_loads_tikz"), 1);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para framed="rectangle" xml:id="p1"><p xml:id="p1.1">Framed.</p></para>"#,
  );
  assert_eq!(xml.matches("<picture ").count(), 1, "{xml}");
  assert_eq!(
    xml.matches("<svg:path d=\"M 0 0 M 39.37 0 C").count(),
    1,
    "{xml}"
  );
}

/// mdframed.sty:72-77: zref-abspage (whose zref-base loads pdftexcmds and so iftex's `\ifpdf`) and needspace come with
/// mdframed (a pdflatex-validated probe). Repro stub-dependencies/mdframed_loads_its_dependencies.
#[test]
fn mdframed_loads_its_dependencies() {
  let xml = convert_with_stub_warnings(repro!("mdframed_loads_its_dependencies"), 1);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">PDF output.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Kept together.</p></para>"#,
  );
}

/// pnas-new.cls:270 `\dates` is a frontmatter date, :284 `\dropcap{T}` the letter, and :53-55 the class's
/// `[framemethod=tikz]{mdframed}` brings tikz (witnesses 2304.12786, 2402.03593, 2507.04951, 2002.07301). Repro
/// stub-dependencies/pnas_new_dates_dropcap.
#[test]
fn pnas_new_dates_dropcap_and_tikz() {
  let xml = convert_with_stub_warnings(repro!("pnas_new_dates_dropcap"), 1);
  assert_element(
    &xml,
    "date",
    &[r#"role="dates""#],
    r#"<date role="dates">Compiled on 1 May</date>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p xml:id="p1.1">This paper.<picture height="1" width="39.92" xml:id="p1.pic1"><svg:svg height="1" overflow="visible" version="1.1" viewBox="0 0 39.92 1" width="39.92"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,1) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)"><svg:path d="M 0 0 L 39.37 0" style="fill:none"/></svg:g></svg:svg></picture></p></para>"##,
  );
  assert_eq!(xml.matches("<para ").count(), 1, "{xml}");
  assert_eq!(xml.matches("<picture ").count(), 1, "{xml}");
}

/// mciteplus.sty:182-184 `\ifmciteErrorOnUnknown`, which an apsrevM `.bbl` switches off, and :170
/// `\mciteSetSublistLabelBeginEnd` (witness 0802.3383). Repro stub-dependencies/mciteplus_bbl_switches.
#[test]
fn mciteplus_bbl_switches() {
  let xml = convert_with_stub_warnings(repro!("mciteplus_bbl_switches"), 1);
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="a""#],
    r#"<bibitem key="a" xml:id="bib.bib1">
      <tags><tag>[1]</tag><tag role="refnum">1</tag></tags>
      <bibblock> A. Author, Journal 1 (2000).
</bibblock>
    </bibitem>"#,
  );
}

/// achemso.cls:1091-1102, 1160-1169, 1266-1269: `\SectionsOn`, `\AbstractOn` and the TOC graphic's lengths
/// (witnesses 1710.02414, 2006.15922, 2112.07721); the class binding loads mciteplus, the one warning. Repro
/// stub-dependencies/achemso_section_abstract_switches.
#[test]
fn achemso_section_abstract_switches() {
  let xml = convert_with_stub_warnings(repro!("achemso_section_abstract_switches"), 1);
  assert_element(
    &xml,
    "abstract",
    &[r#"xml:id="abstract1""#],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Short abstract.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="autoref">section 1<text xml:id="S1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Intro</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">Text.</p></para></section>"#,
  );
  assert_eq!(xml.matches("<section ").count(), 1, "{xml}");
}

/// arydshln.sty:697-715: `{Tabular}` is a tabular with dashed lines. Repro
/// stub-dependencies/arydshln_shorthand_environments.
#[test]
fn arydshln_shorthand_environments() {
  let xml = convert_with_stub_warnings(repro!("arydshln_shorthand_environments"), 1);
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="p1.1""#],
    r#"<tabular vattach="middle" xml:id="p1.1">
      <tbody>
        <tr xml:id="p1.1.1">
          <td align="right" border="b" class="ltx_border_r_dashed" xml:id="p1.1.1.1">a</td>
          <td align="center" border="b" xml:id="p1.1.1.2">b</td>
        </tr>
      </tbody>
    </tabular>"#,
  );
}

/// nag.sty:82-168, 534: the rule commands consume their arguments, `\ObsoleteClass` its optional reason too. Repro
/// stub-dependencies/nag_public_api.
#[test]
fn nag_public_api() {
  let xml = convert_with_stub_warnings(repro!("nag_public_api"), 1);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Body.</p></para>"#,
  );
  assert_eq!(xml.matches("<para ").count(), 1, "{xml}");
}

/// mathpartir.sty:62-68: `\mpr@andcr` separates premises (ottalt.sty; witness 2304.05276). Repro
/// stub-dependencies/mathpartir_premise_separators.
#[test]
fn mathpartir_premise_separators() {
  let xml = convert_with_stub_warnings(repro!("mathpartir_premise_separators"), 1);
  assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S0.Ex1""#],
    r#"<equation xml:id="S0.Ex1"><Math mode="display" tex="\frac{a\qquad b}{c}" text="fragments@(a, b) / c" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.3"><XMApp xml:id="S0.Ex1.m1.3.1"><XMTok mathstyle="display" meaning="divide" role="FRACOP"/><XMDual xml:id="S0.Ex1.m1.3.1.2"><XMApp xml:id="S0.Ex1.m1.3.1.2.1"><XMTok meaning="fragments"/><XMRef idref="S0.Ex1.m1.1" xml:id="S0.Ex1.m1.3.1.2.1.2"/><XMRef idref="S0.Ex1.m1.2" xml:id="S0.Ex1.m1.3.1.2.1.3"/></XMApp><XMWrap xml:id="S0.Ex1.m1.3.1.2.2"><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">a</XMTok><XMHint name="qquad" role="PUNCT" width="20pt" xml:id="S0.Ex1.m1.3.1.2.2.1"/><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">b</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math></equation>"#,
  );
  assert_eq!(xml.matches("<equation ").count(), 1, "{xml}");
}

/// hyperref.sty:2791-2810, 4587-4589: `[pagebackref]{hyperref}` loads backref, whose `\ifbackrefparscan`,
/// `\backrefsep` and `\backref` the document uses (witness 1809.07574). Loaded through hyperref's option, the backref
/// stub reports itself as an Info, not a Warning (witness 2203.07162; OXIDIZED_DESIGN_DIVERGENCES #547). Repro
/// stub-dependencies/hyperref_pagebackref_loads_backref.
#[test]
fn hyperref_pagebackref_loads_backref() {
  let (log, xml) =
    convert_with_stub_warnings_and_log(repro!("hyperref_pagebackref_loads_backref"), 0);
  assert_eq!(
    lines_starting(&log, "Info:missing_file:backref.sty"),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="b""#],
    r#"<bibitem key="b" xml:id="bib.bib1">
      <tags>
        <tag>[1]</tag>
        <tag role="autoref">1<text xml:id="bib.bib1.1"/></tag>
        <tag role="refnum">1</tag>
      </tags>
      <bibblock> B. Author, Book.</bibblock>
    </bibitem>"#,
  );
}

/// epstopdf.sty:149-152: infwarerr, kvoptions and pdftexcmds come with epstopdf, and pdftexcmds's iftex defines
/// `\ifpdf` (witness 2410.22009). epstopdf is a full binding: no warning. Repro
/// stub-dependencies/epstopdf_loads_pdftexcmds.
#[test]
fn epstopdf_loads_pdftexcmds() {
  let xml = convert_with_stub_warnings(repro!("epstopdf_loads_pdftexcmds"), 0);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">PDF output.</p></para>"#,
  );
}

/// hyperref.sty:2791-2810: `\PassOptionsToPackage{pagebackref}{hyperref}` loads backref as the option does, with the
/// Info, `\ifHy@backref` true (:2801) and backref's `hyperpageref` `\backref` empty (backref.sty:88-92). Repro
/// stub-dependencies/hyperref_passoptions_pagebackref_info.
#[test]
fn hyperref_passoptions_pagebackref_info() {
  let (log, xml) =
    convert_with_stub_warnings_and_log(repro!("hyperref_passoptions_pagebackref_info"), 0);
  assert_eq!(
    lines_starting(&log, "Info:missing_file:backref.sty"),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">backref, on, []</p></para>"#,
  );
}

/// A document's own `\usepackage{backref}` keeps the stub's Warning, and its `ref` option makes `\backref` print
/// `\backrefsectionsname` (backref.sty:83-85). Repro stub-dependencies/backref_direct_load_warns.
#[test]
fn backref_direct_load_warns() {
  let (log, xml) = convert_with_stub_warnings_and_log(repro!("backref_direct_load_warns"), 1);
  assert_eq!(
    lines_starting(&log, "Warning:missing_file:backref.sty"),
    1,
    "{log}"
  );
  assert_eq!(
    lines_starting(&log, "Info:missing_file:backref.sty"),
    0,
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">[sections ]</p></para>"#,
  );
}

/// hyperref.sty:2762-2810: `backref=none` and `pagebackref=false` load no backref and leave `\ifHy@backref` false.
/// Repro stub-dependencies/hyperref_backref_none_loads_nothing.
#[test]
fn hyperref_backref_none_loads_nothing() {
  let (log, xml) =
    convert_with_stub_warnings_and_log(repro!("hyperref_backref_none_loads_nothing"), 0);
  assert_eq!(
    lines_starting(&log, "Info:missing_file:backref.sty"),
    0,
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">no backref, off</p></para>"#,
  );
}

/// latex.ltx:18628 `\@onlypreamble\RequirePackage`: a `framemethod=tikz` set in the body loads no tikz (pdflatex:
/// "Can be used only in preamble"); the binding says so in an Info. Repro
/// stub-dependencies/mdframed_body_framemethod_loads_nothing.
#[test]
fn mdframed_body_framemethod_loads_nothing() {
  let (log, xml) =
    convert_with_stub_warnings_and_log(repro!("mdframed_body_framemethod_loads_nothing"), 1);
  assert_eq!(
    lines_starting(&log, "Info:unexpected:framemethod"),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">no tikz</p></para>"#,
  );
}

/// pnas-new.cls:402-428: `\showmatmethods` prints the stored Materials and Methods under its heading, `\showacknow`
/// the acknowledgments (witness 2304.12786 ships the class). Repro stub-dependencies/pnas_new_show_matmethods_acknow.
#[test]
fn pnas_new_show_matmethods_acknow() {
  let xml = convert_with_stub_warnings(repro!("pnas_new_show_matmethods_acknow"), 1);
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="Sx1""#],
    r#"<section xml:id="Sx1"><title>Materials and Methods</title><para xml:id="Sx1.p1"><p xml:id="Sx1.p1.1">Methods text.</p></para><acknowledgements inlist="toc" name="Acknowledgements" xml:id="Sx1.acknowledgements1">Thanks.</acknowledgements></section>"#,
  );
  assert_element(
    &xml,
    "acknowledgements",
    &[r#"xml:id="Sx1.acknowledgements1""#],
    r#"<acknowledgements inlist="toc" name="Acknowledgements" xml:id="Sx1.acknowledgements1">Thanks.</acknowledgements>"#,
  );
  assert_eq!(xml.matches("<acknowledgements").count(), 1, "{xml}");
}

/// arydshln.sty:697-715: `{Longtable}` is defined when longtable came first; its `:` column is dashed. Repro
/// stub-dependencies/arydshln_longtable_shorthand.
#[test]
fn arydshln_longtable_shorthand() {
  let xml = convert_with_stub_warnings(repro!("arydshln_longtable_shorthand"), 1);
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular xml:id="S0.T1.1"><tr xml:id="S0.T1.1.1"><td align="right" class="ltx_border_r_dashed" xml:id="S0.T1.1.1.1">a</td><td align="center" xml:id="S0.T1.1.1.2">b</td></tr></tabular>"#,
  );
  assert_eq!(xml.matches("<tabular ").count(), 1, "{xml}");
}
