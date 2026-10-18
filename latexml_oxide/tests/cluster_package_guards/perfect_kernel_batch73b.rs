//! Red/green guards for batch 73b: kernel shapes of the rc_struct6 study (agent_reports/2026-10-17_rc_struct6.md,
//! rows 2, 4, 5, 6, 9 and the kernel one-offs of row 10).
//! - A `$` in a display TeX sets as `\hbox{$…$}` (fleqn.clo's, iopart's and pasj's `{equation}`, sigma's amsmath
//!   displays, amsbsy's `\boldsymbol`, bm's) is text in that math (tex.web §1193, §1138).
//! - `\setlength` on a non-register runs `#1 #2\relax` (latex.ltx:10253); `\captionof` leaves `\@captype` set
//!   (caption.sty:391); a `\refstepcounter` in math whose label text cannot set is set as math.
//! - One-offs: `{center}`'s `\\`, a sectsty style ending in a command, a listings escape read where it runs, a
//!   declared math alphabet's argument, a `[short]` toc title's diagnostics.
//!
//! Every repro is checked against pdflatex (TL 2025; its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// Converts the repro `topic/name.tex` under tools/perfect_kernel/repros.
macro_rules! repro {
  ($path:literal) => {
    convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      None,
    )
  };
}

/// paragraph-mode/dollar_in_fleqn_clo_equation_is_text: fleqn.clo's `{equation}` is an `\hbox{$…$}`, so
/// `…$ for $…` is a formula, the text "for", a formula (pdflatex 0 errors; cond-mat0202518, hep-ex9806021). Before
/// 73b: 2 Errors "Missing $ closing display math". Its `\[` and `{displaymath}` open a `\bgroup` after the `$`, so an
/// author's `$` there errs in TeX too: paragraph-mode/dollar_in_fleqn_clo_bracket_still_errs keeps 4 Errors
/// (pdflatex 4).
#[test]
fn dollar_in_a_fleqn_clo_display_is_text() {
  let (stderr, xml) = repro!("paragraph-mode/dollar_in_fleqn_clo_equation_is_text.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "XMText", &[], r#"<XMText> for </XMText>"#);
  let (stderr, _) = repro!("paragraph-mode/dollar_in_fleqn_clo_bracket_still_errs.tex");
  assert_eq!(error_count(&stderr), 4, "{stderr}");
}

/// paragraph-mode/dollar_in_iopart_equation_is_text: iopart.cls:771-775's `{equation}` (1905.11073; pdflatex 0
/// errors with the class). Before 73b: 2 Errors.
#[test]
fn dollar_in_an_iopart_equation_is_text() {
  let (stderr, xml) = repro!("paragraph-mode/dollar_in_iopart_equation_is_text.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "XMText", &[], r#"<XMText> for </XMText>"#);
}

/// paragraph-mode/dollar_in_sigma_display_is_text: sigma.cls loads article with `fleqn` and amsmath with `fleqn`
/// (2111.06645; pdflatex: no error in the displays). Before 73b: 4 Errors.
#[test]
fn sigma_displays_are_flush_left_hboxes() {
  let (stderr, xml) = repro!("paragraph-mode/dollar_in_sigma_display_is_text.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document class="ltx_fleqn" xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><para xml:id="p1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="autoref">Equation 1</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="f(s)=0$ for all $s&gt;0." text="f@(s) = 0 * [ for all ] * s &gt; 0" xml:id="S0.E1.m1"><XMath><XMDual><XMRef idref="S0.E1.m1.1"/><XMWrap><XMApp xml:id="S0.E1.m1.1"><XMTok meaning="multirelation"/><XMApp><XMTok font="italic" role="UNKNOWN">f</XMTok><XMDual><XMRef idref="S0.E1.m1.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.2">s</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="0" role="NUMBER">0</XMTok><XMText> for all </XMText><XMTok font="italic" role="UNKNOWN">s</XMTok></XMApp><XMTok meaning="greater-than" role="RELOP">&gt;</XMTok><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMath></Math></equation><equation xml:id="S0.Ex1"><Math mode="display" tex="g$ and $h" text="g * [ and ] * h" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok><XMText> and </XMText><XMTok font="italic" role="UNKNOWN">h</XMTok></XMApp></XMath></Math></equation></para></document>"#,
  );
}

/// paragraph-mode/dollar_in_boldsymbol_is_text, dollar_in_bm_boldsymbol_is_text: amsbsy's `\boldsymbol` boxes its
/// argument, bm's emits its math shifts (2508.01295, 2506.20975; pdflatex 0 errors). Before 73b: 1 Error 2 Warnings,
/// 2 Errors 9 Warnings.
#[test]
fn dollar_in_boldsymbol_is_text() {
  let (stderr, xml) = repro!("paragraph-mode/dollar_in_boldsymbol_is_text.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m2\""],
    r#"<Math mode="inline" tex="\boldsymbol{\theta=\pi/2$ and $\phi=0}" text="theta = (pi / 2) * [ and ] * phi = 0" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="multirelation"/><XMTok font="bold italic" name="theta" role="UNKNOWN">θ</XMTok><XMTok font="bold" meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok font="bold" meaning="divide" role="MULOP">/</XMTok><XMTok font="bold italic" name="pi" role="UNKNOWN">π</XMTok><XMTok font="bold" meaning="2" role="NUMBER">2</XMTok></XMApp><XMText> and </XMText><XMTok font="bold italic" name="phi" role="UNKNOWN">ϕ</XMTok></XMApp><XMTok font="bold" meaning="equals" role="RELOP">=</XMTok><XMTok font="bold" meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math>"#,
  );
  let (stderr, xml) = repro!("paragraph-mode/dollar_in_bm_boldsymbol_is_text.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title><tag close=" ">1</tag><Math mode="inline" tex="\bm{e^{+}e^{-}}" text="e ^ + * e ^ -" xml:id="S1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post2"/><XMTok font="bold italic" role="UNKNOWN">e</XMTok><XMTok font="bold" fontsize="70%" meaning="plus" role="ADDOP">+</XMTok></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post2"/><XMTok font="bold italic" role="UNKNOWN">e</XMTok><XMTok font="bold" fontsize="70%" meaning="minus" role="ADDOP">-</XMTok></XMApp></XMApp></XMath></Math><Math mode="inline" tex="\bm{\rightarrow}" text="rightarrow" xml:id="S1.m2"><XMath><XMTok font="bold" name="rightarrow" role="ARROW">→</XMTok></XMath></Math><Math mode="inline" tex="\bm{J/\psi}" text="J / psi" xml:id="S1.m3"><XMath><XMApp><XMTok font="bold" meaning="divide" role="MULOP">/</XMTok><XMTok font="bold italic" role="UNKNOWN">J</XMTok><XMTok font="bold italic" name="psi" role="UNKNOWN">ψ</XMTok></XMApp></XMath></Math>, fits</title>"#,
  );
}

/// parameter-conditional/setlength_on_a_non_register_runs_its_code: `\setlength` before an `\item` runs
/// `\item F\relax…` (2201.08831, 2401.09168; pdflatex 0 errors). Before 73b: 1 Error, "aceFusion".
#[test]
fn setlength_on_a_non_register_runs_its_code() {
  let (stderr, xml) = repro!("parameter-conditional/setlength_on_a_non_register_runs_its_code.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "itemize",
    &[],
    r#"<itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>FaceFusion, a tool.</p></para></item><item xml:id="S0.I1.i2"><tags><tag>•</tag><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I1.i2.p1"><p>UBO.</p></para></item></itemize>"#,
  );
}

/// In the preamble what such a `\setlength` sets is TeX's "Missing \begin{document}" (pdflatex 1 error), which nothing
/// raises here: the `Variable` read's Error stands for it, with its recovery as before 73b (fancyheadings'
/// `\headrulewidth`, a macro).
#[test]
fn setlength_on_a_macro_in_the_preamble_keeps_its_error() {
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\def\\headrulewidth{0.4pt}\n\\setlength{\\headrulewidth}{0pt}\n\\begin{document}\nHello.\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("A <variable> was supposed to be here"),
    "{stderr}"
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>.4pt 0pt Hello.</p></para>"#,
  );
}

/// captions-floats/captionof_leaves_captype_outside_floats: after `\captionof{figure}` a bare `\caption` in a
/// minipage is Figure 2 (2206.08980, 2508.21533; pdflatex 0 errors). Before 73b: 1 Error.
#[test]
fn captionof_leaves_the_type_outside_floats() {
  let (stderr, xml) = repro!("captions-floats/captionof_leaves_captype_outside_floats.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    r#"<figure class="ltx_minipage" inlist="lof" vattach="middle" width="345.0pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">2</tag>Second caption.</toccaption><caption><tag close=": ">Figure 2</tag>Second caption.</caption></figure>"#,
  );
}

/// macro-state/refstepcounter_in_math_sets_a_math_label: `\theconsta` = `{\kappa_{\arabic{consta}}}` stepped in math
/// (1609.03070, 1904.00290; pdflatex 0 errors); an equation re-stepped in math keeps its text tags. Before 73b: 3
/// Errors.
#[test]
fn refstepcounter_in_math_sets_a_math_label() {
  let (stderr, xml) = repro!("macro-state/refstepcounter_in_math_sets_a_math_label.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "equation",
    &[],
    r#"<equation labels="LABEL:e:b" xml:id="S0.E1"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><Math mode="display" tex="x" text="x" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation>"#,
  );
}

/// paragraph-mode/centering_rebinds_the_line_break: `{center}` and `{flushleft}` let `\\` be a line break whatever
/// the document made it (hep-ph9607435; pdflatex 0 errors). Before 73b: 3 Errors.
#[test]
fn centering_rebinds_the_line_break() {
  let (stderr, xml) = repro!("paragraph-mode/centering_rebinds_the_line_break.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><resource src="LaTeXML.css" type="text/css"/><resource src="ltx-article.css" type="text/css"/><logical-block><para xml:id="p1"><p align="center">Patrick Huet</p><p align="center">Department of Physics</p></para></logical-block><logical-block><para xml:id="p2"><p align="left">A</p><p align="left">B</p></para></logical-block></document>"#,
  );
}

/// package-bindings/sectsty_font_ending_in_a_command: `\sectionfont{\large\centering\uppercase}` (1509.02957; pdflatex
/// 0 errors, "1 INTRODUCTION"). Before 73b: 1 Error.
#[test]
fn section_font_ending_in_a_command_takes_the_title() {
  let (stderr, xml) = repro!("package-bindings/sectsty_font_ending_in_a_command.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title class="ltx_align_center" fontsize="120%">INTRODUCTION</title>"#,
  );
}

/// listings-algorithms/listings_escape_verb_recatcodes: `%B(col\verb._.idx(j))%` (1106.5908; pdflatex 0 errors).
/// Before 73b: 1 Error.
#[test]
fn listings_escape_is_read_where_it_runs() {
  let (stderr, xml) = repro!("listings-algorithms/listings_escape_verb_recatcodes.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "listingline",
    &[],
    r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_space">  </text><text class="ltx_lst_comment" font="italic">C(i)<text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>val(j)<text class="ltx_lst_space"> </text>*<text class="ltx_lst_space"> </text>B(col<verbatim font="typewriter upright">_</verbatim>idx(j))C</text></listingline>"#,
  );
}

/// fonts-nfss/declared_math_alphabet_takes_one_argument: mathalfa's `\mathbb` in an amsmath `\frac` (1810.09080;
/// pdflatex 0 errors). Before 73b: 1 Error.
#[test]
fn declared_math_alphabet_takes_one_argument() {
  let (stderr, xml) = repro!("fonts-nfss/declared_math_alphabet_takes_one_argument.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMApp><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post2"/><XMTok font="blackboard" fontsize="70%" role="UNKNOWN">W</XMTok><XMTok font="italic" fontsize="50%" role="UNKNOWN">c</XMTok></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">w</XMTok></XMApp></XMath>"#,
  );
}

/// sectioning-frontmatter/short_toc_title_errs_only_where_printed (and _where_printed): a `[short]` toc title's
/// unbalanced `$` errs only when the contents are printed (2304.03934; pdflatex 0 and 3 errors). Before 73b: 1 Error
/// in both.
#[test]
fn short_toc_title_errs_only_where_printed() {
  let (stderr, xml) = repro!("sectioning-frontmatter/short_toc_title_errs_only_where_printed.tex");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "subsection",
    &[],
    r#"<subsection inlist="toc" xml:id="S1.SS1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title><tag close=" ">1.1</tag>Example: <Math mode="inline" tex="M(-2;1/2)" text="M@(list@(- 2, 1 / 2))" xml:id="S1.SS1.m1"><XMath><XMApp><XMTok font="italic" role="UNKNOWN">M</XMTok><XMDual><XMApp><XMTok meaning="list"/><XMRef idref="S1.SS1.m1.1"/><XMRef idref="S1.SS1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="S1.SS1.m1.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="PUNCT">;</XMTok><XMApp xml:id="S1.SS1.m1.2"><XMTok meaning="divide" role="MULOP">/</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></title><para xml:id="S1.SS1.p1"><p>Text.</p></para></subsection>"#,
  );
  let (stderr, _) = repro!("sectioning-frontmatter/short_toc_title_errs_where_printed.tex");
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Extra }, or forgotten $"), "{stderr}");
}
