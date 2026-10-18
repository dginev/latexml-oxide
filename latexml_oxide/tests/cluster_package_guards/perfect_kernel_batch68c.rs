//! Red/green guards for batch 68c (binding gaps G3 B): author files and symbol packages.
//! - jmlr2e borrows amsthm and gains `{conjecture}`/`{axiom}`; spconf's further author lines; supertabular's
//!   `\ST@newpage` and breakurl's character lists (dgruyter.sty); article's page styles and `\@makecaption`; mathpazo's
//!   `\mathbold`; mathastext's user commands and versions; wasysym 2.4's characters; biblatex's `\RN`/`\Rn` and
//!   configuration commands; the ICML notice strings (the paper's copy's, else the version's); amsmath's display
//!   switches and `\intkern@`.
//!
//! Every repro (`tools/perfect_kernel/repros/{package-bindings,class-bindings,ams-internals}/`) is checked against
//! pdflatex (the author-file styles with a witness's shipped copy beside it).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_files_with, convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv (with `files` beside it), which must give no error and no warning; and its
/// log.
fn convert_clean(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let (log, xml) = if files.is_empty() {
    convert_with(tex, Some("ar5iv.sty"))
  } else {
    convert_files_with(tex, files, Some("ar5iv.sty"))
  };
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (log, xml)
}

/// The no-break space an `\autoref` name is joined to its number with.
const NBSP: char = '\u{a0}';

macro_rules! repro {
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name
    ))
  };
}

/// jmlr2e borrows amsthm: the paper's `\let\proof\relax` and own `\usepackage{amsthm}` load amsthm's proof
/// (pdflatex "Proof. Body. □"; 67int6: undefined `{proof}`). 1302.6584, 1310.4849.
#[test]
fn jmlr2e_relaxed_proof_takes_amsthm() {
  let (_, xml) = convert_clean(
    repro!("package-bindings", "jmlr2e_relaxed_proof_then_amsthm.tex"),
    &[],
  );
  assert_element(
    &xml,
    "proof",
    &[],
    r#"<proof xml:id="id1"><title class="ltx_runin" font="italic">Proof.</title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">Body.∎</text></p></para></proof>"#,
  );
}

/// jmlr2e.sty:388-396: `conjecture` and `axiom` share `theorem`'s counter, `example` has its own (pdflatex "Theorem 1 …
/// Conjecture 2 … Axiom 3 … Example 1"). 0907.5309, 1108.0065.
#[test]
fn jmlr2e_conjecture_axiom_and_example() {
  let (_, xml) = convert_clean(
    repro!("package-bindings", "jmlr2e_conjecture_and_axiom.tex"),
    &[],
  );
  assert_element(
    &xml,
    "theorem",
    &[r#"xml:id="Thmtheorem2""#],
    &format!(
      r#"<theorem class="ltx_theorem_conjecture" inlist="thm theorem:conjecture" xml:id="Thmtheorem2"><tags><tag>Conjecture 2</tag><tag role="autoref">Theorem{NBSP}2<text xml:id="Thmtheorem2.1"/></tag><tag role="refnum">2</tag><tag role="typerefnum">Conjecture 2</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmtheorem2.2">Conjecture 2</text></tag><text font="bold" xml:id="Thmtheorem2.3">.</text></title><para xml:id="Thmtheorem2.p1"><p xml:id="Thmtheorem2.p1.1"><text font="italic" xml:id="Thmtheorem2.p1.1.1">Every even number is even.</text></p></para></theorem>"#
    ),
  );
  assert_element(
    &xml,
    "theorem",
    &[r#"xml:id="Thmtheorem3""#],
    &format!(
      r#"<theorem class="ltx_theorem_axiom" inlist="thm theorem:axiom" xml:id="Thmtheorem3"><tags><tag>Axiom 3</tag><tag role="autoref">Theorem{NBSP}3<text xml:id="Thmtheorem3.1"/></tag><tag role="refnum">3</tag><tag role="typerefnum">Axiom 3</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmtheorem3.2">Axiom 3</text></tag><text font="bold" xml:id="Thmtheorem3.3">.</text></title><para xml:id="Thmtheorem3.p1"><p xml:id="Thmtheorem3.p1.1"><text font="italic" xml:id="Thmtheorem3.p1.1.1">Zero is a number.</text></p></para></theorem>"#
    ),
  );
  assert_element(
    &xml,
    "theorem",
    &[r#"xml:id="Thmexample1""#],
    r#"<theorem class="ltx_theorem_example" inlist="thm theorem:example" xml:id="Thmexample1"><tags><tag>Example 1</tag><tag role="autoref">1<text xml:id="Thmexample1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Example 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmexample1.2">Example 1</text></tag><text font="bold" xml:id="Thmexample1.3">.</text></title><para xml:id="Thmexample1.p1"><p xml:id="Thmexample1.p1.1"><text font="italic" xml:id="Thmexample1.p1.1.1">An example.</text></p></para></theorem>"#,
  );
}

/// spconf's `\secondlinename` adds its names to `\name`'s (pdflatex: a second line of names; 67int6: undefined).
/// 1701.07847, 1710.09338.
#[test]
fn spconf_second_line_adds_authors() {
  let (_, xml) = convert_clean(
    repro!("package-bindings", "spconf_second_line_names.tex"),
    &[],
  );
  let names: Vec<&str> = xml
    .split("<personname>")
    .skip(1)
    .filter_map(|rest| rest.split('<').next())
    .collect();
  assert_eq!(names, ["Ann One", "Bob Two", "Cy Three"], "{xml}");
}

/// spconf's `\threeauthors{names}{address}×3`: three creators with their affiliations (pdflatex: three columns;
/// 67int6: undefined). 0708.3022.
#[test]
fn spconf_threeauthors_gives_three_creators() {
  let (_, xml) = convert_clean(repro!("package-bindings", "spconf_threeauthors.tex"), &[]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Ann One</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Inst A</contact></creator>",
  );
  assert_eq!(xml.matches(r#"role="affiliation""#).count(), 3, "{xml}");
}

/// dgruyter.sty:1704-1706 wraps supertabular's `\ST@newpage`, :2052-2058 extends breakurl's break characters in DVI
/// mode (latex 0 errors; 67int6: 3 undefined). 1503.04650, 1711.03341.
#[test]
fn dgruyter_patches_supertabular_and_breakurl() {
  let (_, xml) = convert_clean(
    repro!(
      "package-bindings",
      "dgruyter_patches_supertabular_and_breakurl.tex"
    ),
    &[],
  );
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr xml:id="tab1.1.1"><td align="left" xml:id="tab1.1.1.1">a</td><td align="left" xml:id="tab1.1.1.2">b</td></tr>"#,
  );
}

/// article.cls:131-168 page styles and :480-489 `\@makecaption` (pdflatex "Table 1: Gaps of the instances."; 67int6:
/// undefined `\ps@myheadings`, `\@makecaption`). 0711.0180, 1004.5433, 1010.5412.
#[test]
fn article_page_styles_and_makecaption() {
  let (_, xml) = convert_clean(
    repro!("class-bindings", "article_page_styles_and_makecaption.tex"),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1"><text width="345.0pt" xml:id="p1.1.1"><text xml:id="p1.1.1.1">Table 1: Gaps of the instances.</text></text></p>"#,
  );
}

/// mathpazo.sty:72 `\mathbold` is bold italic, `\PazoBB` the blackboard alphabet (67int6: undefined). 0807.0088.
#[test]
fn mathpazo_mathbold_and_pazobb() {
  let (_, xml) = convert_clean(repro!("package-bindings", "mathpazo_mathbold.tex"), &[]);
  assert!(
    xml.contains(r#"<XMTok font="bold italic" role="UNKNOWN">x</XMTok>"#)
      && xml.contains(r#"<XMTok font="blackboard" role="UNKNOWN">R</XMTok>"#),
    "{xml}"
  );
}

/// mathastext's font and spacing commands take their arguments and leave no text (pdflatex "Text a + b and x.";
/// 67int6: 6 undefined). 1712.02869.
#[test]
fn mathastext_user_commands_take_their_arguments() {
  let (_, xml) = convert_clean(
    repro!("package-bindings", "mathastext_user_commands.tex"),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text <Math mode="inline" tex="a+b" text="a + b" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math> and <Math mode="inline" tex="\mathsf{x}" text="x" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMTok font="sansserif" role="UNKNOWN">x</XMTok></XMath></Math>.</p>"#,
  );
}

/// mathastext.sty:2585-2588 `\MTversion{bold}` selects bold math (`\mathversion{#2}` first), :2411-2420
/// `\MTDeclareVersion` declares a selectable version, and :843-846 the `\Mathastext…` names of the font choices
/// (pdflatex "Bold x and sans z.", x bold; 68c r1: `\MTversion` a no-op, 3 undefined). 68c review B4.
#[test]
fn mathastext_version_selects_math_version() {
  let (_, xml) = convert_clean(
    repro!(
      "package-bindings",
      "mathastext_version_selects_math_version.tex"
    ),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Bold <Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="bold italic" role="UNKNOWN">x</XMTok></XMath></Math> and sans <Math mode="inline" tex="z" text="z" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math>.</p>"#,
  );
}

/// wasysym 2.4's characters (wasysym.sty:262-270; 67int6: 5 undefined). 1001.4255, 1103.0853.
#[test]
fn wasysym_new_characters() {
  let (_, xml) = convert_clean(repro!("package-bindings", "wasysym_new_characters.tex"), &[
  ]);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">§\u{a0}1, §\u{a0}2, 5\u{2009}€, ſ, ⌘.</p>",
  );
}

/// biblatex.sty:3948-3960 `\RN`/`\Rn`, and the configuration commands taking their arguments (pdflatex "Cases IV and
/// iii."; 67int6: 8 undefined). 2011.12274, 1707.02797, 2406.05146, 2207.06399.
#[test]
fn biblatex_roman_numerals_and_config_commands() {
  let (_, xml) = convert_clean(
    repro!("package-bindings", "biblatex_roman_and_config_commands.tex"),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Cases IV and iii.</p>"#,
  );
}

/// A biber `.bbl` with entry sets (`\set`, `\inset`) and a data annotation (`\annotation`, five arguments): each
/// entry prints (67int6: undefined `\set`, `\inset`, `\annotation`). 1110.6541, 2001.10285.
#[test]
fn biblatex_bbl_sets_and_annotations() {
  let bbl = r"% $ biblatex auxiliary file $
% $ biblatex bbl format version 2.8 $
\refsection{0}
  \sortlist[entry]{none/global/}
    \entry{a}{article}{}
      \inset{s}
      \name{author}{1}{}{%
        {{hash=1}{%
           family={Able},
           familyi={A\bibinitperiod}}}%
      }
      \field{title}{First title}
      \field{year}{2001}
      \annotation{item}{author}{1}{}{highlight}
    \endentry
    \entry{s}{set}{}
      \set{a}
      \field{title}{The set}
    \endentry
  \endsortlist
\endrefsection
";
  let (_, xml) = convert_clean(
    r"\documentclass{article}
\usepackage{biblatex}
\begin{document}
See \cite{a}.
\printbibliography
\end{document}",
    &[("t.bbl", bbl)],
  );
  assert_element(
    &xml,
    "bibentry",
    &[r#"key="a""#],
    r#"<bibentry key="a" type="article" xml:id="bib.bib1"><bib-name role="author"><surname>Able</surname></bib-name><bib-title>First title</bib-title><bib-date role="publication">2001</bib-date><bib-data role="self" type="BibTeX">@article{a, author = {Able}, title = {First title}, date = {2001}}</bib-data><bib-data role="labelnumber">1</bib-data></bibentry>"#,
  );
  // The set entry prints as one more entry, its member not inside it (biblatex_sty.rs `\biblatex@bbl@set`). Each
  // carries its label number, its place in the datalist (77a: biblatex.sty:8891-8923 `\blx@bbl@labelnumber`).
  assert_element(
    &xml,
    "bibentry",
    &[r#"key="s""#],
    r#"<bibentry key="s" type="set" xml:id="bib.bib2"><bib-title>The set</bib-title><bib-data role="self" type="BibTeX">@set{s, title = {The set}}</bib-data><bib-data role="labelnumber">2</bib-data></bibentry>"#,
  );
}

/// icml2026.sty:129-177, :504-516: the notice string a paper's own `\printAffiliationsAndNotice` prints, under
/// `[accepted]` the 2026 proceedings line, and the flag it sets (pdflatex footnote "Proceedings of the 43rd …";
/// 67int6: undefined `\icml@noticeprintedtrue`, `\Notice@String`). 2601.22575, 2601.23143.
#[test]
fn icml2026_notice_string_and_flag() {
  let (_, xml) = convert_clean(
    repro!(
      "package-bindings",
      "icml2026_notice_from_own_printaffiliations.tex"
    ),
    &[],
  );
  let note = xml
    .split(r#"<note role="footnotetext""#)
    .nth(1)
    .and_then(|rest| rest.split("</note>").next())
    .unwrap_or_default();
  assert!(
    note.contains("Proceedings of the ")
      && note.contains("Machine Learning</text>, Seoul, South Korea. PMLR 306, 2026."),
    "{xml}"
  );
}

/// `\Notice@String` under `[accepted]` as icml2024.sty sets it (:101-103, `\ICML@appearing`, the 41st-ICML
/// proceedings line; pdflatex with 2406.00116's copy), not the review notice (68c r1: "Under review by the …"). 68c
/// review B1; 2406.00116, 2406.00274.
#[test]
fn icml2024_accepted_notice_is_proceedings() {
  let (_, xml) = convert_clean(
    repro!(
      "package-bindings",
      "icml2024_accepted_notice_is_proceedings.tex"
    ),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Notice: <text font="italic" xml:id="p1.1.1">Proceedings of the <Math mode="inline" tex="\mathit{41}^{st}" text="41 ^ (s * t)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="41" role="NUMBER">41</XMTok><XMApp xml:id="p1.m1.1.1.3"><XMTok font="upright" meaning="times" role="MULOP">⁢</XMTok><XMTok fontsize="70%" role="UNKNOWN">s</XMTok><XMTok fontsize="70%" role="UNKNOWN">t</XMTok></XMApp></XMApp></XMath></Math> International Conference on Machine Learning</text>, Vienna, Austria. PMLR 235, 2024. Copyright 2024 by the author(s)..</p>"#,
  );
}

/// A version whose proceedings line is not known (icml2020's copies disagree: PMLR 108, 119 or empty): `[accepted]`
/// sets an empty notice (pdflatex with 2009.02479's copy "Notice: ."; 68c r1: the review notice). 68c review B1.
#[test]
fn icml2020_accepted_notice_unknown_is_empty() {
  let (_, xml) = convert_clean(
    repro!(
      "package-bindings",
      "icml2020_accepted_notice_unknown_is_empty.tex"
    ),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Notice: .</p>"#,
  );
}

/// The paper's copy's own `\ICML@appearing` (a workshop's line: 2507.02987, 2506.09099, 2606.28747, 2604.14572), not
/// the version's proceedings line (68c r1: the 42nd-ICML line). 68c review B1.
#[test]
fn icml2025_customised_copy_notice() {
  let (_, xml) = convert_clean(
    repro!("package-bindings", "icml2025_customised_copy_notice.tex"),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Notice: <text font="italic" xml:id="p1.1.1">Proceedings of the ICML 2025 Workshop on Tiny Things</text>, Vancouver, Canada. 2025..</p>"#,
  );
}

/// amsmath.sty:1280-1290 display switches and :653 `\intkern@` (pdflatex 0 errors; 67int6: 4 undefined). 1606.07128,
/// 2004.13893.
#[test]
fn amsmath_switches_and_intkern() {
  let (_, xml) = convert_clean(
    repro!("ams-internals", "amsmath_switches_and_intkern.tex"),
    &[],
  );
  assert_eq!(
    xml.matches(r#"name="intop" role="INTOP""#).count(),
    2,
    "{xml}"
  );
}
