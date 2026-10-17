//! Red/green guards for batch 71a: binding shapes from the rc_struct5 study (agent_reports/2026-10-17_rc_struct5.md).
//! B1 amsrefs `{biblist}` without `{bibdiv}`; T1 mn2e defines no `\ion`; P1 appolb.cls; U1 tkz-euclide's old
//! `\usetkzobj`; N1 siunitx decimal uncertainty; L1 txfonts `\lambdabar`; the small ones: forest's and todonotes' TikZ
//! libraries, pgfmath `Mod`, amsmath/amsfonts macro accents, breqn's `\sb`/`\sp` (their undelimited argument read as its
//! definition says, with OXIDIZED_DESIGN_DIVERGENCES #721's Warning where TeX errs).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

fn fatal_count(log: &str) -> usize { log.lines().filter(|l| l.starts_with("Fatal:")).count() }

/// The OXIDIZED_DESIGN_DIVERGENCES #721 Warnings: an undelimited argument that is a command reading a mandatory
/// argument, where TeX errs (`amsmath_sty.rs` `\lx@undelimited@argument`).
fn undelimited_warnings(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|l| {
      l.starts_with("Warning:malformed:") && l.contains(" alone as its argument, where TeX errs")
    })
    .collect()
}

/// amsrefs.sty:532-538: `{biblist}` stands alone, so an author's `\section*{References}\begin{biblist}` is the
/// document's bibliography, headed as the PDF heads it (1002.1686, 1112.3058, 2111.14165).
#[test]
fn amsrefs_biblist_without_bibdiv() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/amsrefs_biblist_without_bibdiv.tex"
  ));
  assert_eq!(
    (error_count(&log), warning_count(&log), fatal_count(&log)),
    (0, 0, 0),
    "{log}"
  );
  assert!(!xml.contains("<section"), "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography citestyle="numbers" inlist="toc" xml:id="bib"><title>References</title><biblist><bibentry key="A" type="article" xml:id="bib.bib1"><bib-name role="author"><surname>Doe</surname><givenname>J.</givenname></bib-name><bib-title>T</bib-title><bib-related role="host" type="journal"><bib-title>J</bib-title></bib-related><bib-date role="publication">2000</bib-date><bib-data role="self" type="BibTeX">@article{A, author = {Doe, J.}, title = {T}, journal = {J}, year = {2000}}</bib-data></bibentry></biblist></bibliography>"#,
  );
}

/// mn2e.cls defines no `\ion`, so the author's `\newcommand{\ion}` is the document's (0908.0351, astro-ph0501147).
#[test]
fn mn2e_author_ion_wins() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mn2e_author_ion_wins.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">the prominent He\u{2009}<text font=\"smallcaps\" xml:id=\"p1.1.1\">I</text> lines.</p>",
  );
}

/// mnras.cls keeps its `\ion` as each version defines it: v3.2 (TL, :488) on amsmath's `\text` where amsmath is loaded,
/// v3.0 (1809.05472's copy, :477) `\textrm{#1}\,\textsc{…}` without it (1809.05472 loads no amsmath; pdflatex with
/// TL's v3.2: "Undefined control sequence").
#[test]
fn mnras_ion_by_version() {
  let (log, xml) = convert_counted(
    "\\documentclass{mnras}\n\\usepackage{amsmath}\n\\begin{document}\nthe \\ion{He}{II} line\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">the <text xml:id=\"p1.1.1\">He\u{2009}<text font=\"smallcaps\" xml:id=\"p1.1.1.1\">ii</text></text> line</p>",
  );
  let (log, xml) = convert_counted(
    "\\documentclass{mnras}\n\\begin{document}\nthe \\ion{He}{II} line\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">the He <text font="smallcaps" xml:id="p1.1.1">ii</text> line</p>"#,
  );
}

/// appolb.cls:322 `\PACS` is the classification after the abstract (0712.3606, 1007.2277).
#[test]
fn appolb_pacs() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/appolb_pacs.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "classification",
    &[],
    r#"<classification name="PACS numbers:" scheme="pacs">27.75.-q</classification>"#,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A</personname><contact name=\"Address:\u{a0}\" role=\"address\">B</contact></creator>",
  );
}

/// appolb.cls:299, :304 running heads and :1019-1033 abbreviations (hep-ph0403117).
#[test]
fn appolb_running_heads_and_abbreviations() {
  let (log, xml) = convert_counted(
    "\\documentclass{appolb}\n\\begin{document}\n\\title{T}\\author{A}\\maketitle\n\\headtitle{Short T}\\headauthor{A. B.}\nSee \\ie, this and \\etal\\ here.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "note",
    &[r#"role="runningtitle""#],
    r#"<note role="runningtitle" xml:id="id1">Short T</note>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="runningauthor""#],
    r#"<note role="runningauthor" xml:id="id2">A. B.</note>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">See <text font="italic" xml:id="p1.1.1">i.e.</text>, this and <text font="italic" xml:id="p1.1.2">et al.</text> here.</p>"#,
  );
}

/// The rest of appolb.cls's public commands (2210.03396's copy): :278-282 `\preprint`, :283-285 the `\volume`/`\No`
/// counts, :1019 `\anymode`, :1053-1057 `\half`/`\Half`, :1061 `\Tr`, :1063 `\Eq`, :1065 `\seteqprefix`
/// (pdflatex with that copy: 0 errors, "1/2 + 1/2 + Tr A and x^2, see Eq. (A1.1)").
#[test]
fn appolb_class_commands() {
  let (log, xml) = convert_counted(
    "\\documentclass{appolb}\n\\eqsec\n\\begin{document}\n\\preprint{TPJU-1/2004}\n\\title{T}\\author{A}\\address{B}\\maketitle\n\\section{S} $\\half + \\Half + \\Tr A$ and \\anymode{x^2}, see \\Eq{x}.\n\\seteqprefix{A}\n\\begin{equation}a\\label{x}\\end{equation}\nVolume \\the\\volume, No \\the\\No.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "pubnote",
    &[r#"role="preprint""#],
    r#"<pubnote name="Preprint: " role="preprint">TPJU-1/2004</pubnote>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="S1.p1.1""#],
    r#"<p xml:id="S1.p1.1"><Math mode="inline" tex="{\textstyle\frac{1}{2}}+\frac{1}{2}+\mathop{\operator@font Tr}\nolimits A" text="1 / 2 + 1 / 2 + Tr@(A)" xml:id="S1.p1.m1"><XMath xml:id="S1.p1.m1.1"><XMApp xml:id="S1.p1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="S1.p1.m1.1.1.2"><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp xml:id="S1.p1.m1.1.1.3"><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp xml:id="S1.p1.m1.1.1.4"><XMTok role="BIGOP" scriptpos="post1">Tr</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok></XMApp></XMApp></XMath></Math> and <Math mode="inline" tex="x^{2}" text="x ^ 2" xml:id="S1.p1.m2"><XMath xml:id="S1.p1.m2.1"><XMApp xml:id="S1.p1.m2.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math>, see Eq. (<ref labelref="LABEL:x"/>).</p>"#,
  );
  assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S1.E1""#],
    r#"<equation labels="LABEL:x" xml:id="S1.E1"><tags><tag>(1.1)</tag><tag role="refnum">A1.1</tag></tags><Math mode="display" tex="a" text="a" xml:id="S1.E1.m1"><XMath xml:id="S1.E1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></equation>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="S1.p1.2""#],
    r#"<p xml:id="S1.p1.2">Volume 0, No 0.</p>"#,
  );
}

/// tkz-euclide's pre-v3 `\usetkzobj{all}` loads nothing more: v3+ loads every object (1506.03598, 2005.14485).
#[test]
fn tkz_euclide_old_usetkzobj() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/tkz_euclide_old_usetkzobj.tex"
  ));
  // Without the loaders the picture is the same, and the Error ("Undefined control sequence") is what fails.
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "svg:path",
    &[],
    r#"<svg:path d="M 0 0 M 0 0 L 78.74 0" style="fill:none"/>"#,
  );
}

/// siunitx.sty:700-848: a decimal marker in the bracketed uncertainty sets the uncertainty in the number's last places
/// (2311.18489, 2504.14399).
#[test]
fn siunitx_uncertainty_with_decimal_marker() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/siunitx_uncertainty_with_decimal_marker.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="4.02(52)\text{\,}\mathrm{m}" text="(4.02 * 52) * meter" xml:id="p1.m1"><XMath xml:id="p1.m1.4a"><XMApp xml:id="p1.m1.4a.1"><XMText meaning="times" role="MULOP" xml:id="p1.m1.1"> </XMText><XMApp xml:id="p1.m1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="4.02" role="NUMBER">4.02</XMTok><XMDual xml:id="p1.m1.2.4"><XMRef idref="p1.m1.3" xml:id="p1.m1.2.4.1"/><XMWrap xml:id="p1.m1.2.4.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="52" role="NUMBER" xml:id="p1.m1.3">52</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok class="ltx_unit" meaning="meter" role="ID">m</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="1.08(3)" text="1.08 * 3" xml:id="p1.m2"><XMath xml:id="p1.m2.2"><XMApp xml:id="p1.m2.2.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="1.08" role="NUMBER">1.08</XMTok><XMDual xml:id="p1.m2.2.1.3"><XMRef idref="p1.m2.1" xml:id="p1.m2.2.1.3.1"/><XMWrap xml:id="p1.m2.2.1.3.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="3" role="NUMBER" xml:id="p1.m2.1">3</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m3""#],
    r#"<Math mode="inline" tex="11.0(17)\text{\,}\mathrm{V}" text="(11.0 * 17) * V" xml:id="p1.m3"><XMath xml:id="p1.m3.4a"><XMApp xml:id="p1.m3.4a.1"><XMText meaning="times" role="MULOP" xml:id="p1.m3.1"> </XMText><XMApp xml:id="p1.m3.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="11.0" role="NUMBER">11.0</XMTok><XMDual xml:id="p1.m3.2.4"><XMRef idref="p1.m3.3" xml:id="p1.m3.2.4.1"/><XMWrap xml:id="p1.m3.2.4.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="17" role="NUMBER" xml:id="p1.m3.3">17</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="UNKNOWN">V</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "XMDual",
    &[r#"xml:id="p1.m1.2.4""#],
    r#"<XMDual xml:id="p1.m1.2.4"><XMRef idref="p1.m1.3" xml:id="p1.m1.2.4.1"/><XMWrap xml:id="p1.m1.2.4.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="52" role="NUMBER" xml:id="p1.m1.3">52</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual>"#,
  );
  // the number's places padded to the uncertainty's; more integer digits than the number's is no match (siunitx.sty:758)
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{siunitx}\n\\begin{document}\n\\num{4.0(0.52)} \\num{4(0.5)}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="4.00(52)" text="4.00 * 52" xml:id="p1.m1"><XMath xml:id="p1.m1.2"><XMApp xml:id="p1.m1.2.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="4.00" role="NUMBER">4.00</XMTok><XMDual xml:id="p1.m1.2.1.3"><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.3.1"/><XMWrap xml:id="p1.m1.2.1.3.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="52" role="NUMBER" xml:id="p1.m1.1">52</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="4.0(5)" text="4.0 * 5" xml:id="p1.m2"><XMath xml:id="p1.m2.2"><XMApp xml:id="p1.m2.2.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="4.0" role="NUMBER">4.0</XMTok><XMDual xml:id="p1.m2.2.1.3"><XMRef idref="p1.m2.1" xml:id="p1.m2.2.1.3.1"/><XMWrap xml:id="p1.m2.2.1.3.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="5" role="NUMBER" xml:id="p1.m2.1">5</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
  );
  let (log, _) = convert_counted(
    "\\documentclass{article}\n\\usepackage{siunitx}\n\\begin{document}\n\\num{1.5(12.3)}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
}

/// txfonts.sty:727-728 `\lambdaslash`, `\lambdabar` (0801.4523, 2104.03192).
#[test]
fn txfonts_lambdabar() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/txfonts_lambdabar.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<XMTok font="italic" name="lambdabar" role="UNKNOWN">ƛ</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="italic" name="lambdaslash" role="UNKNOWN">ƛ</XMTok>"#),
    "{xml}"
  );
}

/// forest.sty:50-53, :129: its TikZ libraries and its key family (2012.09112, 1910.06571, 2003.06880).
#[test]
fn forest_loads_its_tikz_libraries() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/forest_loads_its_tikz_libraries.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // shapes.arrows' `single arrow` (without it a rectangle), calc's `($(A)+(2,0)$)` and fit's box around A and B
  assert_element(
    &xml,
    "svg:path",
    &[r#"d="M 19.69 0"#],
    r#"<svg:path d="M 19.69 0 L 0.23 19.46 L 0.23 9.62 L -10.08 9.62 L -10.08 -9.62 L 0.23 -9.62 L 0.23 -19.46 Z" style="fill:none"/>"#,
  );
  assert_element(
    &xml,
    "svg:path",
    &[r#"d="M -14.97"#],
    r#"<svg:path d="M -14.97 -24.7 h 108.11 v 49.39 h -108.11 Z" style="fill:none"/>"#,
  );
}

/// todonotes.sty:32 loads positioning (1807.00512, 2107.03711).
#[test]
fn todonotes_loads_positioning() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/todonotes_loads_positioning.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // positioning's `right=of A` on the grid: B's centre 1cm (28.45pt) right of A's (without it B is at A's right edge)
  assert_element(
    &xml,
    "svg:svg",
    &[],
    r##"<svg:svg height="18.68" overflow="visible" version="1.1" viewBox="0 0 58.68 18.68" width="58.68"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,18.68) matrix(1 0 0 -1 0 0) translate(9.8,0) translate(0,9.34)"><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 -5.19 -4.73)"><svg:foreignObject height="9.46" overflow="visible" style="--ltx-fo-width:0.75em;--ltx-fo-height:0.68em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="10.38"><text xml:id="p1.pic1.1">A</text></svg:foreignObject></svg:g><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 34.47 -4.73)"><svg:foreignObject height="9.46" overflow="visible" style="--ltx-fo-width:0.71em;--ltx-fo-height:0.68em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="9.8"><text xml:id="p1.pic1.2">B</text></svg:foreignObject></svg:g></svg:g></svg:svg>"##,
  );
}

/// pgfmathfunctions.basic.code.tex:263-297: `mod` truncates, `Mod` adds the divisor to a negative `mod` only, so a
/// multiple of the divisor is 0 (1811.01404).
#[test]
fn pgfmath_mod_of_a_multiple_is_zero() {
  let (log, _) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/pgfmath_mod_of_a_multiple_is_zero.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let values: Vec<&str> = log
    .split("MOD(")
    .skip(1)
    .map(|s| s.split_whitespace().next().unwrap_or(""))
    .collect();
  assert_eq!(
    values,
    ["-10)=0", "-5)=0", "-4)=1", "-1)=4", "0)=0", "4)=4", "5)=0"],
    "{log}"
  );
  let (log, _) = convert_counted(
    "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\pgfmathparse{mod(-7,5)}\\typeout{TRUNC=\\pgfmathresult}\\pgfmathmod{-7}{5}\\typeout{MACRO=\\pgfmathresult}\\pgfmathMod{-7}{5}\\typeout{FLOOR=\\pgfmathresult}\n\\end{document}\n",
  );
  assert!(
    log.contains("TRUNC=-2.0") && log.contains("MACRO=-2.0") && log.contains("FLOOR=3.0"),
    "{log}"
  );
}

/// amsmath.sty:754-797: an accent's accentee is an undelimited argument, so a parameterless macro after it is accented
/// whole (0801.1527), and `\Hat` is `\hat` (:892). Its argument is read as its definition says (user ruling 2026-10-17,
/// OXIDIZED_DESIGN_DIVERGENCES #721): no mandatory argument braced, as TeX braces it (`\hat\relax x`, `\hat\vec x`:
/// TeX's `\showlists` has the hat over an empty nucleus, over the vec over one, with x outside); a mandatory one
/// (`\mathbf`, a robust `\vv` through its `\protect \vv␣`) unbraced, read with what follows, and one Warning that
/// TeX errs there (pdflatex: 6 and 8 errors).
#[test]
fn amsmath_accent_macro_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/amsmath_accent_macro_argument.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\tilde{m_{\gamma^{\prime}}}_{a}^{2}" text="((tilde@(m _ (gamma ^ prime))) _ a) ^ 2" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp xml:id="p1.m1.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMApp xml:id="p1.m1.1.1.2.2"><XMTok name="tilde" role="OVERACCENT" stretchy="false">~</XMTok><XMApp xml:id="p1.m1.1.1.2.2.2"><XMTok role="SUBSCRIPTOP" scriptpos="post3"/><XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="p1.m1.1.1.2.2.2.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post4"/><XMTok font="italic" fontsize="70%" name="gamma" role="UNKNOWN">γ</XMTok><XMTok fontsize="50%" name="prime" role="SUPOP">′</XMTok></XMApp></XMApp></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="\hat{m_{\gamma^{\prime}}}_{b}" text="(hat@(m _ (gamma ^ prime))) _ b" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMApp xml:id="p1.m2.1.1.2"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMApp xml:id="p1.m2.1.1.2.2"><XMTok role="SUBSCRIPTOP" scriptpos="post3"/><XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="p1.m2.1.1.2.2.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post4"/><XMTok font="italic" fontsize="70%" name="gamma" role="UNKNOWN">γ</XMTok><XMTok fontsize="50%" name="prime" role="SUPOP">′</XMTok></XMApp></XMApp></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
  );
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/amsmath_accent_command_argument_warns.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(
    undelimited_warnings(&log),
    [
      "Warning:malformed:\\hat amsmath's \\hat takes \\mathbf alone as its argument, where TeX errs: \\mathbf reads an \
       argument of its own; read as \\hat{\\mathbf…}, with \\mathbf's argument",
      "Warning:malformed:\\hat amsmath's \\hat takes \\vv alone as its argument, where TeX errs: \\vv reads an argument \
       of its own; read as \\hat{\\vv…}, with \\vv's argument",
    ],
    "{log}"
  );
  for (m, expected) in (1..).zip([
    r#"<Math mode="inline" tex="\hat{x}" text="hat@(x)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\vec{\nabla}" text="vec@(nabla)" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\dot{\gamma}_{1}" text="(dot@(gamma)) _ 1" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMApp xml:id="p1.m3.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMApp xml:id="p1.m3.1.1.2"><XMTok name="dot" role="OVERACCENT">˙</XMTok><XMTok font="italic" name="gamma" role="UNKNOWN">γ</XMTok></XMApp><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\hat{m_{\gamma}}_{a}" text="(hat@(m _ gamma)) _ a" xml:id="p1.m4"><XMath xml:id="p1.m4.1"><XMApp xml:id="p1.m4.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMApp xml:id="p1.m4.1.1.2"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMApp xml:id="p1.m4.1.1.2.2"><XMTok role="SUBSCRIPTOP" scriptpos="post3"/><XMTok font="italic" role="UNKNOWN">m</XMTok><XMTok font="italic" fontsize="70%" name="gamma" role="UNKNOWN">γ</XMTok></XMApp></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\hat{}x" text="hat@(absent) * x" xml:id="p1.m5"><XMath xml:id="p1.m5.1"><XMApp xml:id="p1.m5.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="p1.m5.1.1.2"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok meaning="absent"/></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\hat{\vec{}}x" text="hat@(vec@(absent)) * x" xml:id="p1.m6"><XMath xml:id="p1.m6.1"><XMApp xml:id="p1.m6.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="p1.m6.1.1.2"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMApp xml:id="p1.m6.1.1.2.2"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok meaning="absent"/></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\hat{\mathbf{x}}" text="hat@(x)" xml:id="p1.m7"><XMath xml:id="p1.m7.1"><XMApp xml:id="p1.m7.1.1"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok font="bold" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
    r#"<Math mode="inline" tex="\hat{\mathbf{y}}" text="hat@(y)" xml:id="p1.m8"><XMath xml:id="p1.m8.1"><XMApp xml:id="p1.m8.1.1"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok font="bold" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"#,
  ]) {
    assert_element(&xml, "Math", &[&format!(r#"xml:id="p1.m{m}""#)], expected);
  }
}

/// An ltcmd accentee is judged by its signature: an all-`m` `\NewDocumentCommand` through its `\cs code`, with the
/// Warning (pdflatex: 10 errors with the two below); an `{o}` one braced, as TeX braces it.
#[test]
fn amsmath_accent_ltcmd_argument() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\NewDocumentCommand\\vv{m}{\\mathbf{#1}}\n\\NewDocumentCommand\\so{o}{\\mathbf{q}}\n\\begin{document}\n$\\hat\\vv{x}$ $\\hat\\so$\n\\end{document}\n",
  );
  assert_eq!(
    (
      error_count(&log),
      warning_count(&log),
      undelimited_warnings(&log).len()
    ),
    (0, 1, 1),
    "{log}"
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\hat{\mathbf{x}}" text="hat@(x)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok font="bold" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="\hat{\mathbf{q}}" text="hat@(q)" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMTok font="bold" role="UNKNOWN">q</XMTok></XMApp></XMath></Math>"#,
  );
}

/// One OXIDIZED_DESIGN_DIVERGENCES #721 Warning per occurrence TeX typesets: a heading's and a caption's toc copy
/// (`\lx@toc@copy@neutralize`'s written copy) is what LaTeX writes to the .toc/.lof, not typesets there; an
/// alignment's cells, a `[short]` toc title, an unnumbered heading each once (pdflatex errs at each typeset one).
#[test]
fn undelimited_argument_warning_once_per_occurrence() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n\\tableofcontents\n\\section{On $\\hat\\mathbf{x}$ fields}\n\\section[Short $\\check\\mathbf{s}$]{Long $\\dot\\mathbf{l}$}\n\\subsection*{Star $\\ddot\\mathbf{t}$}\n\\begin{figure}\\caption{A $\\tilde\\mathbf{w}$ map}\\end{figure}\n\\begin{align}\na &= \\bar\\mathbf{b} \\\\\nc &= \\vec\\mathrm{d}\n\\end{align}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  let readers: Vec<&str> = undelimited_warnings(&log)
    .iter()
    .map(|l| l.split_whitespace().next().unwrap_or(""))
    .collect();
  assert_eq!(
    readers,
    [
      "Warning:malformed:\\hat",
      "Warning:malformed:\\dot",
      "Warning:malformed:\\ddot",
      "Warning:malformed:\\tilde",
      "Warning:malformed:\\bar",
      "Warning:malformed:\\vec",
    ],
    "{log}"
  );
  assert_eq!(warning_count(&log), 6, "{log}");
  assert_element(
    &xml,
    "toctitle",
    &[],
    r#"<toctitle><tag close=" ">2</tag>Short <Math mode="inline" tex="\check{\mathbf{s}}" text="check@(s)" xml:id="S2.m2"><XMath xml:id="S2.m2.1"><XMApp xml:id="S2.m2.1.1"><XMTok name="check" role="OVERACCENT" stretchy="false">ˇ</XMTok><XMTok font="bold" role="UNKNOWN">s</XMTok></XMApp></XMath></Math></toctitle>"#,
  );
}

/// amsfonts.sty:78-87 `\widehat#1` (2609.01909).
#[test]
fn amsfonts_widehat_macro_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/amsfonts_widehat_macro_argument.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &[r#"mode="display""#],
    r#"<Math mode="display" tex="\widehat{C_{\mathrm{BA}}}_{\rm CF}=1" text="(widehat@(C _ BA)) _ CF = 1" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMApp xml:id="S0.Ex1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.Ex1.m1.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMApp xml:id="S0.Ex1.m1.1.1.2.2"><XMTok name="widehat" role="OVERACCENT">^</XMTok><XMApp xml:id="S0.Ex1.m1.1.1.2.2.2"><XMTok role="SUBSCRIPTOP" scriptpos="post3"/><XMTok font="italic" role="UNKNOWN">C</XMTok><XMTok fontsize="70%" role="UNKNOWN">BA</XMTok></XMApp></XMApp><XMTok fontsize="70%" role="UNKNOWN">CF</XMTok></XMApp><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
  );
  // amsfonts' `\widehat#1` takes a mandatory argument of its own: `\hat\widehat x` is no nested accent (TeX errs)
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath,amssymb}\n\\begin{document}\n$\\widehat\\mathbf{x}$ $\\hat\\widehat y$\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(undelimited_warnings(&log).len(), 2, "{log}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\widehat{\mathbf{x}}" text="widehat@(x)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="widehat" role="OVERACCENT">^</XMTok><XMTok font="bold" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="\hat{\widehat{y}}" text="hat@(widehat@(y))" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok><XMApp xml:id="p1.m2.1.1.2"><XMTok name="widehat" role="OVERACCENT">^</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMApp></XMath></Math>"#,
  );
}

/// The breqn stub's own Warning, which every breqn document carries by design ("breqn.sty is minimally stubbed": its
/// displays are equations without breqn's line breaking; breqn_sty.rs).
fn breqn_stub_warnings(log: &str) -> usize {
  log
    .lines()
    .filter(|l| l.starts_with("Warning:missing_file:breqn.sty breqn.sty is minimally stubbed"))
    .count()
}

/// mathstyle.sty:124-127 (breqn): `\sb`/`\sp` read their script as an argument (2509.22335).
#[test]
fn breqn_script_macro_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/breqn_script_macro_argument.tex"
  ));
  // the breqn stub's Warning, and no other
  assert_eq!(
    (
      error_count(&log),
      warning_count(&log),
      breqn_stub_warnings(&log)
    ),
    (0, 1, 1),
    "{log}"
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\Delta_{\tau}\doteq 1" text="Delta _ tau approaches-limit 1" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="approaches-limit" name="doteq" role="RELOP">≐</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="Delta" role="UNKNOWN">Δ</XMTok><XMTok font="italic" fontsize="70%" name="tau" role="UNKNOWN">τ</XMTok></XMApp><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
  );
  // a script that is a command reading an argument is read as before, with one Warning each (pdflatex: "Argument of
  // \text@ has an extra }"; OXIDIZED_DESIGN_DIVERGENCES #721); `x_\alpha`, `x_\infty` are braced, as TeX braces them
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{breqn}\n\\begin{document}\n$x_\\text{abc}$ and $y^\\mathrm{T}$ and $z_\\alpha^\\infty$\n\\end{document}\n",
  );
  assert_eq!(
    (
      error_count(&log),
      warning_count(&log),
      breqn_stub_warnings(&log)
    ),
    (0, 3, 1),
    "{log}"
  );
  assert_eq!(undelimited_warnings(&log).len(), 2, "{log}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="x_{\text{abc}}" text="x _ [abc]" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMText xml:id="p1.m1.1.1.3"><text fontsize="70%" xml:id="p1.m1.1.1.3.1">abc</text></XMText></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="y^{\mathrm{T}}" text="y ^ T" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" role="UNKNOWN">T</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m3""#],
    r#"<Math mode="inline" tex="z_{\alpha}^{\infty}" text="(z _ alpha) ^ infinity" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMApp xml:id="p1.m3.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp xml:id="p1.m3.1.1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">z</XMTok><XMTok font="italic" fontsize="70%" name="alpha" role="UNKNOWN">α</XMTok></XMApp><XMTok fontsize="70%" meaning="infinity" name="infty" role="ID">∞</XMTok></XMApp></XMath></Math>"#,
  );
}
