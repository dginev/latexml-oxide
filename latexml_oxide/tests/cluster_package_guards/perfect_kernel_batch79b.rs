//! Red/green guards for batch 79b (audit #363: class begin-document code the bindings dropped). Item 1: sn-jnl's
//! `{appendices}` is appendix.sty's, re-dressed by the class's begin-document code (sn-jnl.cls:1365, :1409-1426) to
//! letter the figures, tables and equations of each appendix ("Fig. A1", "(A1)"); a copy with classic Mac line ends is
//! read line by line (`shipped_class::class_code`). Item 2: the Springer classes' `\vec` is the class's bold vector
//! (svmult.cls:1015-1030, svjour3.cls:631-641, svproc.cls:389-404, llncs.cls:393-397), not the kernel's arrow accent,
//! under the classes' options (KNOWN_PERL_ERRORS #1143).
//!
//! Each guard converts its pdflatex-checked repro under `tools/perfect_kernel/repros/class-bindings/` (its header holds
//! the oracle), or a variant of it checked the same way, and asserts the elements whole, after 0 errors and 0 warnings.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// `markup` with its `~` the no-break space an autoref tag holds ("section 3").
fn nbsp(markup: &str) -> String { markup.replace('~', "\u{a0}") }

/// Convert `tex` with the files beside it, 0 errors and 0 warnings.
fn convert_clean(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = convert_files_with(tex, files, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

const SNJNL: &str = repro!("snjnl_appendices_number_floats_by_letter.tex");

/// The first appendix of snjnl_appendices_number_floats_by_letter: its figure, table and equation numbered by its
/// letter (pdflatex with 2609.33801's sn-jnl.cls: "Fig. A1 y", "Table A1 t", "(A1)").
const SNJNL_APPENDIX_A: &str = concat!(
  r#"<appendix inlist="toc" labels="LABEL:s2" xml:id="A1"><tags><tag>Appendix A</tag><tag role="autoref">Appendix~A<text xml:id="A1.1"/></tag><tag role="refnum">A</tag><tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix A</tag>App</title><toctitle><tag close=" ">A</tag>App</toctitle><figure inlist="lof" labels="LABEL:f2" xml:id="A1.F1"><tags><tag>Figure A1</tag><tag role="autoref">Figure~A1<text xml:id="A1.F1.1"/></tag><tag role="refnum">A1</tag><tag role="typerefnum">Figure A1</tag></tags><toccaption><tag close=" ">A1</tag>y</toccaption><caption><tag close=": ">Figure A1</tag>y</caption></figure>"#,
  r#"<table inlist="lot" labels="LABEL:t2" xml:id="A1.T1"><tags><tag>Table A1</tag><tag role="autoref">Table~A1<text xml:id="A1.T1.1"/></tag><tag role="refnum">A1</tag><tag role="typerefnum">Table A1</tag></tags><toccaption><tag close=" ">A1</tag>t</toccaption><caption><tag close=": ">Table A1</tag>t</caption><tabular vattach="middle" xml:id="A1.T1.2"><tbody><tr xml:id="A1.T1.2.1"><td align="center" xml:id="A1.T1.2.1.1">z</td></tr></tbody></tabular></table>"#,
  r#"<para xml:id="A1.p1"><equation labels="LABEL:e2" xml:id="A1.E1"><tags><tag>(A1)</tag><tag role="autoref">Equation~A1<text xml:id="A1.E1.1"/></tag><tag role="refnum">A1</tag></tags><Math mode="display" tex="b" text="b" xml:id="A1.E1.m1"><XMath xml:id="A1.E1.m1.1"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></equation></para></appendix>"#,
);

/// class-bindings/snjnl_appendices_number_floats_by_letter (witnesses 2609.24634, 2609.11123, 2609.04047): the class
/// as distributed (no copy) loads appendix.sty with `[title]` and its begin-document code renews `{appendices}`, so the
/// appendices are "Appendix A", "Appendix B" holding "Figure A1", "Table A1", "(A1)" and "(B2)" (the equation counter
/// is reset only where the environment begins). Red (int70a): a plain "§2" with "Figure 2", "Table 1", "(2)", "(3)".
#[test]
fn snjnl_appendices_number_floats_by_the_appendix_letter() {
  let xml = convert_clean(SNJNL, &[]);
  assert_element(
    &xml,
    "appendix",
    &[r#"xml:id="A1""#],
    &nbsp(SNJNL_APPENDIX_A),
  );
  assert_element(
    &xml,
    "equation",
    &[r#"xml:id="A2.E2""#],
    &nbsp(
      r#"<equation xml:id="A2.E2"><tags><tag>(B2)</tag><tag role="autoref">Equation~B2<text xml:id="A2.E2.1"/></tag><tag role="refnum">B2</tag></tags><Math mode="display" tex="c" text="c" xml:id="A2.E2.m1"><XMath xml:id="A2.E2.m1.1"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></equation>"#,
    ),
  );
}

/// A section after `\end{appendices}`: the class's `\end{oldappendices}` calls an `\endoldappendices` it never defines,
/// so appendix.sty's counter restore does not run and only the group's end restores the section form — pdflatex "3
/// After" and "Fig. C2" (the `\gdef`'d `\thefigure`). Red (79b r1 before the end): "Appendix C".
#[test]
fn snjnl_section_after_the_appendices_counts_on() {
  let tex = SNJNL.replace(
    "\\end{appendices}",
    "\\end{appendices}\n\\section{After}\\begin{figure}\\caption{w}\\end{figure}",
  );
  let xml = convert_clean(&tex, &[]);
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S3""#],
    &nbsp(concat!(
      r#"<section inlist="toc" xml:id="S3"><tags><tag>3</tag><tag role="autoref">section~3<text xml:id="S3.1"/></tag><tag role="refnum">3</tag><tag role="typerefnum">§3</tag></tags><title><tag close=" ">3</tag>After</title>"#,
      r#"<figure inlist="lof" xml:id="S3.F2"><tags><tag>Figure C2</tag><tag role="autoref">Figure~C2<text xml:id="S3.F2.1"/></tag><tag role="refnum">C2</tag><tag role="typerefnum">Figure C2</tag></tags><toccaption><tag close=" ">C2</tag>w</toccaption><caption><tag close=": ">Figure C2</tag>w</caption></figure></section>"#,
    )),
  );
}

/// A paper's copy of sn-jnl.cls saved with classic Mac line ends (23 of the census's 164 copies: 2609.04047's,
/// 2106.13602's) is read line by line, as pdflatex reads it ("Table A1" in 2609.04047's PDF): the begin-document code
/// it has is found. Red (79b r1): the whole copy one comment, the tables "Table 17".
#[test]
fn snjnl_copy_with_carriage_returns_letters_its_appendix_floats() {
  let cls = "%% sn-jnl.cls\r\\NeedsTeXFormat{LaTeX2e}%\r\\ProvidesClass{sn-jnl}%\r\\LoadClass{article}%\r\
             \\usepackage[title]{appendix}%\r\\@ifpackageloaded{appendix}{%\r\\AtBeginDocument{%\r\
             \\let\\oldappendices\\appendices%\r\\let\\oldendappendices\\endappendices%\r\
             \\renewenvironment{appendices}{%\r\\setcounter{figure}{0}%\r\\setcounter{table}{0}%\r\
             \\setcounter{equation}{0}%\r\\begin{oldappendices}%\r\
             \\gdef\\thefigure{\\@Alph\\c@section\\arabic{figure}}%\r\
             \\gdef\\thetable{\\@Alph\\c@section\\arabic{table}}%\r\
             \\gdef\\theequation{\\@Alph\\c@section\\arabic{equation}}%\r}{\\end{oldappendices}}\r}%\r}{}\r";
  let xml = convert_clean(SNJNL, &[("sn-jnl.cls", cls)]);
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="A1.T1""#],
    &nbsp(
      r#"<table inlist="lot" labels="LABEL:t2" xml:id="A1.T1"><tags><tag>Table A1</tag><tag role="autoref">Table~A1<text xml:id="A1.T1.1"/></tag><tag role="refnum">A1</tag><tag role="typerefnum">Table A1</tag></tags><toccaption><tag close=" ">A1</tag>t</toccaption><caption><tag close=": ">Table A1</tag>t</caption><tabular vattach="middle" xml:id="A1.T1.2"><tbody><tr xml:id="A1.T1.2.1"><td align="center" xml:id="A1.T1.2.1.1">z</td></tr></tbody></tabular></table>"#,
    ),
  );
}

/// `$\vec{v} + \vec{\omega} = x_{\vec{k}}$` with `\vec` the class's bold: `latin` the font of v and k, `greek` of ω.
fn bold_vectors(latin: &str, greek: &str) -> String {
  format!(
    r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="{latin}" role="UNKNOWN">v</XMTok><XMTok font="{greek}" name="omega" role="UNKNOWN">ω</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="{latin}" fontsize="70%" role="UNKNOWN">k</XMTok></XMApp></XMApp></XMath>"#
  )
}

/// `$\vec{v} + \vec{\omega} = x_{\vec{k}}$` with `\vec` the kernel's arrow accent.
const ARROW_VECTORS: &str = r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="p1.m1.1.1.2.2"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.2.3"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" name="omega" role="UNKNOWN">ω</XMTok></XMApp></XMApp><XMApp xml:id="p1.m1.1.1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp xml:id="p1.m1.1.1.3.3"><XMTok fontsize="70%" name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">k</XMTok></XMApp></XMApp></XMApp></XMath>"#;

/// svmult_vec_is_the_class_s_bold's `\[ \vec{F} = m\vec{a} \]` with `\vec` the class's bold, F and a in `font`.
fn bold_display(font: &str) -> String {
  format!(
    r#"<XMath xml:id="Ch0.Ex1.m1.1"><XMApp xml:id="Ch0.Ex1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="{font}" role="UNKNOWN">F</XMTok><XMApp xml:id="Ch0.Ex1.m1.1.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">m</XMTok><XMTok font="{font}" role="UNKNOWN">a</XMTok></XMApp></XMApp></XMath>"#
  )
}

/// svmult_vec_is_the_class_s_bold's `\[ \vec{F} = m\vec{a} \]` with `\vec` the kernel's arrow accent.
const ARROW_DISPLAY: &str = r#"<XMath xml:id="Ch0.Ex1.m1.1"><XMApp xml:id="Ch0.Ex1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="Ch0.Ex1.m1.1.1.2"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" role="UNKNOWN">F</XMTok></XMApp><XMApp xml:id="Ch0.Ex1.m1.1.1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="Ch0.Ex1.m1.1.1.3.3"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok></XMApp></XMApp></XMApp></XMath>"#;

/// Assert the formula `<XMath xml:id="{id}">` of `xml` whole.
fn assert_formula(xml: &str, id: &str, expected: &str) {
  assert_element(xml, "XMath", &[&format!(r#"xml:id="{id}""#)], expected);
}

/// Assert both formulas of svmult_vec_is_the_class_s_bold whole: the inline one, then the display.
fn assert_svmult_formulas(xml: &str, inline: &str, display: &str) {
  assert_formula(xml, "p1.m1.1", inline);
  assert_formula(xml, "Ch0.Ex1.m1.1", display);
}

/// class-bindings/svmult_vec_is_the_class_s_bold (witnesses 2512.18788, hep-ex0701025): `\vec` is the upright bold
/// letter, bold italic Greek (pdflatex: CMBX10 and CMMIB10). Red (int70a): the kernel's OVERACCENT arrow.
#[test]
fn svmult_vec_is_the_class_s_bold() {
  let xml = convert_clean(repro!("svmult_vec_is_the_class_s_bold.tex"), &[]);
  assert_svmult_formulas(
    &xml,
    &bold_vectors("bold", "bold italic"),
    &bold_display("bold"),
  );
}

/// The options: `vecphys` sets the bold italic; `vecarrow` too in the class as distributed, whose test compares
/// `arrow` with a `\fig@type` that is `figure` by `\begin{document}` (pdflatex: CMMIB10 for both); a copy whose test
/// runs in the class body (0812.2480's, 37 census copies) keeps the arrow under `vecarrow` (pdflatex: CMMI10 and the
/// accent), and an older copy without options (1405.1421's) sets the bold italic whatever is given.
#[test]
fn svmult_vec_options_as_the_copy_reads_them() {
  let tex = repro!("svmult_vec_is_the_class_s_bold.tex");
  let bold_italic = bold_vectors("bold italic", "bold italic");
  let phys = convert_clean(&tex.replace("{svmult}", "[vecphys]{svmult}"), &[]);
  assert_svmult_formulas(&phys, &bold_italic, &bold_display("bold italic"));
  let arrow = convert_clean(&tex.replace("{svmult}", "[vecarrow]{svmult}"), &[]);
  assert_svmult_formulas(&arrow, &bold_italic, &bold_display("bold italic"));
  let body_test = "\\ProvidesClass{svmult}\n\\LoadClass{book}\n\\DeclareOption{vecphys}{\\def\\vec@style{phys}}\n\
                   \\DeclareOption{vecarrow}{\\def\\vec@style{arrow}}\n\\ProcessOptions\n\\def\\fig@type{arrow}\n\
                   \\ifx\\vec@style\\fig@type\\else\n\\@ifundefined{vec@style}{\\def\\vec#1{\\ensuremath{\\mathbf{#1}}}}\
                   {\\def\\vec#1{\\ensuremath{\\boldsymbol{#1}}}}\n\\fi\n\\def\\fig@type{figure}\n";
  let arrow = convert_clean(&tex.replace("{svmult}", "[vecarrow]{svmult}"), &[(
    "svmult.cls",
    body_test,
  )]);
  assert_svmult_formulas(&arrow, ARROW_VECTORS, ARROW_DISPLAY);
  let plain = convert_clean(tex, &[("svmult.cls", body_test)]);
  assert_svmult_formulas(
    &plain,
    &bold_vectors("bold", "bold italic"),
    &bold_display("bold"),
  );
  let old = "\\ProvidesClass{svmult}\n\\LoadClass{book}\n\\def\\vec#1{\\mathchoice{\\mbox{\\boldmath$\\displaystyle#1$}}\
             {\\mbox{\\boldmath$\\textstyle#1$}}{\\mbox{\\boldmath$\\scriptstyle#1$}}\
             {\\mbox{\\boldmath$\\scriptscriptstyle#1$}}}\n";
  let old = convert_clean(tex, &[("svmult.cls", old)]);
  assert_svmult_formulas(&old, &bold_italic, &bold_display("bold italic"));
}

/// class-bindings/llncs_vec_is_bold_italic (witnesses 2607.20650, 2607.18197, 2604.00509): `\vec` is the bold italic
/// letter (pdflatex: CMMIB10); `orivec` keeps the arrow accent. Red (int70a): the arrow in both.
#[test]
fn llncs_vec_is_bold_italic() {
  let tex = repro!("llncs_vec_is_bold_italic.tex");
  let xml = convert_clean(tex, &[]);
  assert_formula(&xml, "p1.m1.1", &bold_vectors("bold italic", "bold italic"));
  let orivec = convert_clean(&tex.replace("{llncs}", "[orivec]{llncs}"), &[]);
  assert_formula(&orivec, "p1.m1.1", ARROW_VECTORS);
}

/// class-bindings/svjour3_vec_is_upright_bold (witnesses 2609.05466, 2509.03659, 2207.14807): `\vec` is the upright
/// bold letter, in text too (`\ensuremath`); under an EPJ journal option, whose svepj.clo turns `\if@mathematic` off
/// (0710.1789's), svjour's is the bold italic; svepjST.clo (1504.01694's, :213-238) sets `\ensuremath{\mathbf{#1}}` at
/// the class's end, the Greek medium (its PDF: CMBX10, no CMMIB), and a paper's copy of the journal file is read for
/// either. Red (int70a): the arrow, and a "should only appear in math mode" warning.
#[test]
fn svjour3_vec_is_upright_bold() {
  let tex = repro!("svjour3_vec_is_upright_bold.tex");
  // the text-mode `\vec{w}` of the repro, `w` in `font`
  let text_vector = |font: &str| {
    format!(r#"<XMath xml:id="p1.m2.1"><XMTok font="{font}" role="UNKNOWN">w</XMTok></XMath>"#)
  };
  let xml = convert_clean(tex, &[]);
  assert_formula(&xml, "p1.m1.1", &bold_vectors("bold", "bold italic"));
  assert_formula(&xml, "p1.m2.1", &text_vector("bold"));
  let epj = convert_clean(&tex.replace("{svjour3}", "[epj]{svjour}"), &[]);
  assert_formula(&epj, "p1.m1.1", &bold_vectors("bold italic", "bold italic"));
  assert_formula(&epj, "p1.m2.1", &text_vector("bold italic"));
  let epjst = convert_clean(&tex.replace("{svjour3}", "[epjST]{svjour}"), &[]);
  assert_formula(&epjst, "p1.m1.1", &bold_vectors("bold", "italic"));
  assert_formula(&epjst, "p1.m2.1", &text_vector("bold"));
  // a shipped journal file that sets neither: the class's form
  let clo = "\\def\\journalname#1{}\n";
  let own = convert_clean(&tex.replace("{svjour3}", "[epjST]{svjour}"), &[(
    "svepjST.clo",
    clo,
  )]);
  assert_formula(&own, "p1.m1.1", &bold_vectors("bold", "bold italic"));
  assert_formula(&own, "p1.m2.1", &text_vector("bold"));
}
