//! Red/green guard for batch 77q: run-367 Error→Fatal regressions, part 1 — amsmath's old-form matrices, `\binom`
//! under a redefined `\atop`, setstack.sty, proof.sty's premises, minted's listings escapes. Each guard asserts the
//! diagnostics by count, the authored content and its element.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{assert_elements, assert_elements_with},
};

const OLD_FORM: &str = "Package amsmath Error: Old form `\\";

/// amsmath's `{matrix}`/`{pmatrix}`/`{cases}` start with `\matrix@check` (amsmath.sty:1068-1093, :1116-1118): the
/// environment when `\@currenvir` names the command, else "Old form `\pmatrix' should be \begin{pmatrix}" for Plain
/// TeX's `\pmatrix{…\cr…}`. pdflatex: that error once per old-form call (19 errors in all, its rows lost); here the
/// error and the Plain-TeX matrix with its rows. The binding's alignment held the `\cr`s in a cell (15 errors; 11
/// papers of run 367 a Fatal: hep-ph0110005, hep-ph9509392, hep-ph9511429, hep-ph9702336, hep-th9908038,
/// nucl-th0305048, 0805.4522, 1003.3524, hep-th9910005, nucl-th0206027, quant-ph0607049). KNOWN_PERL_ERRORS #993.
#[test]
fn amsmath_old_form_matrices_keep_their_rows() {
  let xml = assert_elements_with(
    r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{equation} x=\pmatrix{t & u\cr b & c} \end{equation}
\begin{equation} y=\matrix{1 & 2\cr 3 & 4} \end{equation}
\begin{equation} |z|=\cases{z & if $z>0$\cr -z & otherwise} \end{equation}
\begin{equation} w=\begin{pmatrix} p & q\\ r & s\end{pmatrix}+\begin{cases} 1 & a\\ 0 & b\end{cases} \end{equation}
After.
\end{document}",
    "ar5iv.sty",
    (3, 0),
    &[
      "Old form `\\pmatrix' should be \\begin{pmatrix}",
      "Old form `\\matrix' should be \\begin{matrix}",
      "Old form `\\cases' should be \\begin{cases}",
    ],
    &[
      (
        "equation",
        "S0.E1",
        r#"<equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="x=\pmatrix{t&amp;u\cr b&amp;c}" text="x = matrix@(Array[[t, u], [b, c]])" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.2"><XMApp xml:id="S0.E1.m1.2.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMDual xml:id="S0.E1.m1.2.1.3"><XMApp xml:id="S0.E1.m1.2.1.3.1"><XMTok meaning="matrix"/><XMRef idref="S0.E1.m1.1" xml:id="S0.E1.m1.2.1.3.1.2"/></XMApp><XMWrap xml:id="S0.E1.m1.2.1.3.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMArray xml:id="S0.E1.m1.1"><XMRow xml:id="S0.E1.m1.1.1"><XMCell align="center" xml:id="S0.E1.m1.1.1.1"><XMTok font="italic" role="UNKNOWN">t</XMTok></XMCell><XMCell align="center" xml:id="S0.E1.m1.1.1.2"><XMTok font="italic" role="UNKNOWN">u</XMTok></XMCell></XMRow><XMRow xml:id="S0.E1.m1.1.2"><XMCell align="center" xml:id="S0.E1.m1.1.2.1"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell><XMCell align="center" xml:id="S0.E1.m1.1.2.2"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell></XMRow></XMArray><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></equation>"#,
      ),
      (
        "equation",
        "S0.E3",
        r#"<equation xml:id="S0.E3"><tags><tag>(3)</tag><tag role="refnum">3</tag></tags><Math mode="display" tex="|z|=\cases{z&amp;if $z&gt;0$\cr-z&amp;otherwise}" text="absolute-value@(z) = cases@(z, [if &gt;z0], - z, [otherwise])" xml:id="S0.E3.m1"><XMath xml:id="S0.E3.m1.6"><XMApp xml:id="S0.E3.m1.6.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMDual xml:id="S0.E3.m1.6.1.2"><XMApp xml:id="S0.E3.m1.6.1.2.1"><XMTok meaning="absolute-value"/><XMRef idref="S0.E3.m1.1" xml:id="S0.E3.m1.6.1.2.1.2"/></XMApp><XMWrap xml:id="S0.E3.m1.6.1.2.2"><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E3.m1.1">z</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual xml:id="S0.E3.m1.6.1.3"><XMApp xml:id="S0.E3.m1.6.1.3.1"><XMTok meaning="cases"/><XMRef idref="S0.E3.m1.2" xml:id="S0.E3.m1.6.1.3.1.2"/><XMRef idref="S0.E3.m1.3" xml:id="S0.E3.m1.6.1.3.1.3"/><XMRef idref="S0.E3.m1.4" xml:id="S0.E3.m1.6.1.3.1.4"/><XMRef idref="S0.E3.m1.5" xml:id="S0.E3.m1.6.1.3.1.5"/></XMApp><XMWrap xml:id="S0.E3.m1.6.1.3.2"><XMTok role="OPEN" stretchy="true">{</XMTok><XMArray xml:id="S0.E3.m1.6.1.3.2.2"><XMRow xml:id="S0.E3.m1.6.1.3.2.2.1"><XMCell align="left" xml:id="S0.E3.m1.6.1.3.2.2.1.1"><XMTok font="italic" role="UNKNOWN" xml:id="S0.E3.m1.2">z</XMTok></XMCell><XMCell align="left" xml:id="S0.E3.m1.6.1.3.2.2.1.2"><XMText xml:id="S0.E3.m1.3">if <Math mode="inline" tex="z&gt;0" text="z &gt; 0" xml:id="S0.E3.m1.m1"><XMath xml:id="S0.E3.m1.m1.1"><XMApp xml:id="S0.E3.m1.m1.1.1"><XMTok meaning="greater-than" role="RELOP">&gt;</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math></XMText></XMCell></XMRow><XMRow xml:id="S0.E3.m1.6.1.3.2.2.2"><XMCell align="left" xml:id="S0.E3.m1.6.1.3.2.2.2.1"><XMApp xml:id="S0.E3.m1.4"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMCell><XMCell align="left" xml:id="S0.E3.m1.6.1.3.2.2.2.2"><XMText xml:id="S0.E3.m1.5">otherwise</XMText></XMCell></XMRow></XMArray></XMWrap></XMDual></XMApp></XMath></Math></equation>"#,
      ),
    ],
  );
  for text in [
    "text=\"y = matrix@(Array[[1, 2], [3, 4]])\"",
    "text=\"w = matrix@(Array[[p, q], [r, s]]) + cases@(1, a, 0, b)\"",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// The old aa.cls loads no amsmath, and its Plain-TeX `\pmatrix{…}` is no error there (astro-ph/0002145); a
/// document's own `\usepackage{amsmath}` (the binding re-run, Perl aa.cls.ltxml:57) brings amsmath's old-form error
/// back, as pdflatex gives it. The matrix keeps its rows either way.
#[test]
fn aa_old_form_matrix_errs_under_the_documents_amsmath() {
  let (log, xml) = convert_with(
    r"\documentclass{aa}
\usepackage{amsmath}
\begin{document}
\begin{equation}
B=\pmatrix{1 & 2\cr 3 & 4}
\end{equation}
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains(OLD_FORM), "{log}");
  assert!(
    xml.contains("text=\"B = matrix@(Array[[1, 2], [3, 4]])\""),
    "{xml}"
  );
}

/// The old-form test body shared by the beyond-original guards: whether amsmath counts as loaded, then a Plain-TeX
/// `\pmatrix{…}`.
const OLD_FORM_BODY: &str = r"\begin{document}
\makeatletter\@ifpackageloaded{amsmath}{AMSLOADED}{AMSUNLOADED}\makeatother
\begin{equation}
B=\pmatrix{1 & 2\cr 3 & 4}
\end{equation}
After.
\end{document}";

const OLD_FORM_ROWS: &str = "text=\"B = matrix@(Array[[1, 2], [3, 4]])\"";

/// Bindings that load amsmath where their original file does not (`amsmath_sty::require_amsmath_beyond_original`):
/// spie.cls (TeX Live's; 1003.3524), ws-procs9x6.cls and ws-ijmpb.cls without a copy that loads it (nucl-th0206027,
/// quant-ph0607049), Scientific Word 2.5's tcilatex.tex (0805.4522), sn-jnl.cls without such a copy, the old aa.cls
/// (astro-ph/0002145), OmniBus's autoload for a class we do not know. In pdflatex amsmath is not loaded there and
/// `\pmatrix{…}` is Plain TeX's matrix, no error (spie: 0 errors, AMSUNLOADED); here its rows, no "Old form" error,
/// `\@ifpackageloaded{amsmath}` false. KNOWN_PERL_ERRORS #993.
#[test]
fn old_form_matrix_is_plain_tex_where_amsmath_is_not_loaded() {
  for (preamble, warnings) in [
    (r"\documentclass{spie}", 0),
    (r"\documentclass{ws-procs9x6}", 0),
    (r"\documentclass{ws-ijmpb}", 0),
    (r"\documentclass{article}\input{tcilatex}", 0),
    (r"\documentclass{sn-jnl}", 0),
    (r"\documentclass{aa}", 0),
  ] {
    let (log, xml) = convert_with(&format!("{preamble}\n{OLD_FORM_BODY}"), Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{preamble}\n{log}"
    );
    assert!(xml.contains(OLD_FORM_ROWS), "{preamble}\n{xml}");
    assert!(xml.contains("AMSUNLOADED"), "{preamble}\n{xml}");
  }
  // OmniBus's autoload at `\begin{align}`, then the old form
  let (log, xml) = convert_with(
    r"\documentclass{nosuchclassbatchsevenseven}
\begin{document}
\begin{align} x &= y \end{align}
\makeatletter\@ifpackageloaded{amsmath}{AMSLOADED}{AMSUNLOADED}\makeatother
\begin{equation}
B=\pmatrix{1 & 2\cr 3 & 4}
\end{equation}
After.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Can't find binding for class nosuchclassbatchsevenseven"),
    "{log}"
  );
  assert!(
    xml.contains(OLD_FORM_ROWS) && xml.contains("AMSUNLOADED"),
    "{xml}"
  );
  // the whole equation, spie's
  let (_, xml) = convert_with(
    &format!("\\documentclass{{spie}}\n{OLD_FORM_BODY}"),
    Some("ar5iv.sty"),
  );
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S0.E1\""],
    r#"<equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="autoref">Equation 1<text xml:id="S0.E1.1"/></tag><tag role="refnum">1</tag></tags><Math mode="display" tex="B=\pmatrix{1&amp;2\cr 3&amp;4}" text="B = matrix@(Array[[1, 2], [3, 4]])" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.2"><XMApp xml:id="S0.E1.m1.2.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">B</XMTok><XMDual xml:id="S0.E1.m1.2.1.3"><XMApp xml:id="S0.E1.m1.2.1.3.1"><XMTok meaning="matrix"/><XMRef idref="S0.E1.m1.1" xml:id="S0.E1.m1.2.1.3.1.2"/></XMApp><XMWrap xml:id="S0.E1.m1.2.1.3.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMArray xml:id="S0.E1.m1.1"><XMRow xml:id="S0.E1.m1.1.1"><XMCell align="center" xml:id="S0.E1.m1.1.1.1"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell><XMCell align="center" xml:id="S0.E1.m1.1.1.2"><XMTok meaning="2" role="NUMBER">2</XMTok></XMCell></XMRow><XMRow xml:id="S0.E1.m1.1.2"><XMCell align="center" xml:id="S0.E1.m1.1.2.1"><XMTok meaning="3" role="NUMBER">3</XMTok></XMCell><XMCell align="center" xml:id="S0.E1.m1.1.2.2"><XMTok meaning="4" role="NUMBER">4</XMTok></XMCell></XMRow></XMArray><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></equation>"#,
  );
}

/// The same bindings once the real amsmath.sty loads — the document's `\usepackage{amsmath}` (before or after
/// tcilatex), a binding whose original loads it (mathtools), a paper's copy of the class or of tcilatex.tex that loads
/// it (ws-ijmpa.cls:1087 in 1912.08245's copy; Scientific Word 5.0's tcilatex.tex:851 in 2212.12000's): amsmath's
/// binding runs again, and `\pmatrix{…}` is amsmath's "Old form" error, once, its rows kept (pdflatex, spie with
/// amsmath or mathtools: the old-form error and 5 errors after it, AMSLOADED). KNOWN_PERL_ERRORS #993.
#[test]
fn old_form_matrix_errs_once_the_document_loads_amsmath() {
  let cls_copy = r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{ws-procs9x6}\LoadClass{article}\RequirePackage{amsmath}";
  let tci_copy = r"\typeout{TCILATEX Macros for Scientific Word 5.0}\RequirePackage{amsmath}";
  for (preamble, files) in [
    (r"\documentclass{spie}\usepackage{amsmath}", &[][..]),
    (r"\documentclass{spie}\usepackage{mathtools}", &[][..]),
    (r"\documentclass{ws-procs9x6}\usepackage{amsmath}", &[][..]),
    (r"\documentclass{sn-jnl}\usepackage{amsmath}", &[][..]),
    (
      r"\documentclass{article}\input{tcilatex}\usepackage{amsmath}",
      &[][..],
    ),
    (
      r"\documentclass{article}\usepackage{amsmath}\input{tcilatex}",
      &[][..],
    ),
    (
      r"\documentclass{ws-procs9x6}",
      &[("ws-procs9x6.cls", cls_copy)][..],
    ),
    (
      r"\documentclass{article}\input{tcilatex}",
      &[("tcilatex.tex", tci_copy)][..],
    ),
    // A real load after a binding's beyond-original one (spie's) is still the real load.
    (
      r"\documentclass{spie}\input{tcilatex}",
      &[("tcilatex.tex", tci_copy)][..],
    ),
  ] {
    let (log, xml) = convert_files_with(
      &format!("{preamble}\n{OLD_FORM_BODY}"),
      files,
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (1, 0),
      "{preamble}\n{log}"
    );
    assert!(
      log.contains("Old form `\\pmatrix' should be \\begin{pmatrix}"),
      "{preamble}\n{log}"
    );
    assert!(xml.contains(OLD_FORM_ROWS), "{preamble}\n{xml}");
    assert!(xml.contains("AMSLOADED"), "{preamble}\n{xml}");
  }
}

/// AMS-TeX's `\matrix … \endmatrix` and `\cases … \endcases` (amstex.tex:512-552) are the amsmath binding's
/// environment forms, never its LaTeX old-form branch: 1411.5748's `\left(\matrix \format … \endmatrix\right)` (2 → 59
/// errors with the old-form branch). Knuth's `tex` with amstex: 0 errors.
#[test]
fn amstex_matrix_reads_to_endmatrix() {
  let (log, xml) = convert_with(
    r"\input amstex
\documentstyle{amsppt}
\document
Let
$$c = \left(\matrix a & b \\ c & d \endmatrix\right), \quad
|x| = \cases x & x>0 \\ -x & \text{otherwise}\endcases$$
After.
\enddocument",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      "text=\"formulae@(c = matrix@(Array[[a, b], [c, d]]), absolute-value@(x) = cases@(x, x &gt; 0, - x, \
       [otherwise]))\""
    ),
    "{xml}"
  );
}

/// amsmath's `\binom`/`\tbinom`/`\dbinom` are `\genfrac` (amsmath.sty:239-241), never the user-level `\atop`, which a
/// document may redefine with arguments (`\renewcommand{\atop}[2]`, 1010.0429, 1209.4365, 1308.4325, 1806.09142,
/// 1904.01263): its second argument was the closing brace ("Missing { inserted"). pdflatex: 0 errors.
/// KNOWN_PERL_ERRORS #994.
#[test]
fn binom_ignores_a_redefined_atop() {
  assert_elements(
    r"\documentclass{article}
\usepackage{amsmath}
\renewcommand{\atop}[2]{\genfrac{}{}{0pt}{}{#1}{#2}}
\begin{document}
\begin{equation}
\begin{split}
q&=\binom{n}{k}^2 \\
p&=\tbinom{a}{b} \\
r&=\dbinom{c}{d}+\atop{x}{y}.
\end{split}
\end{equation}
After.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "equation",
      "S0.E1",
      r#"<equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="\begin{split}q&amp;=\binom{n}{k}^{2}\\&#10;p&amp;=\tbinom{a}{b}\\&#10;r&amp;=\dbinom{c}{d}+\genfrac{}{}{0.0pt}{}{x}{y}.\end{split}" text="q = (binomial@(n, k)) ^ 2 * p = binomial@(a, b) * r = binomial@(c, d) + x FRACOP y" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.22a"><XMDual xml:id="S0.E1.m1.22a.1"><XMDual xml:id="S0.E1.m1.22a.1.1"><XMRef idref="S0.E1.m1.1" xml:id="S0.E1.m1.22a.1.1.1"/><XMWrap xml:id="S0.E1.m1.22a.1.1.2"><XMApp xml:id="S0.E1.m1.1"><XMTok meaning="multirelation"/><XMRef idref="S0.E1.m1.2" xml:id="S0.E1.m1.1.2"/><XMRef idref="S0.E1.m1.3" xml:id="S0.E1.m1.1.3"/><XMApp xml:id="S0.E1.m1.1.4"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp xml:id="S0.E1.m1.1.4.2"><XMTok role="SUPERSCRIPTOP" scriptpos="post6"/><XMRef idref="S0.E1.m1.4" xml:id="S0.E1.m1.1.4.2.2"/><XMRef idref="S0.E1.m1.8" xml:id="S0.E1.m1.1.4.2.3"/></XMApp><XMRef idref="S0.E1.m1.9" xml:id="S0.E1.m1.1.4.3"/></XMApp><XMRef idref="S0.E1.m1.10" xml:id="S0.E1.m1.1.5"/><XMApp xml:id="S0.E1.m1.1.6"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMRef idref="S0.E1.m1.11" xml:id="S0.E1.m1.1.6.2"/><XMRef idref="S0.E1.m1.15" xml:id="S0.E1.m1.1.6.3"/></XMApp><XMRef idref="S0.E1.m1.17" xml:id="S0.E1.m1.1.7"/><XMApp xml:id="S0.E1.m1.1.8"><XMRef idref="S0.E1.m1.18" xml:id="S0.E1.m1.1.8.1"/><XMRef idref="S0.E1.m1.19" xml:id="S0.E1.m1.1.8.2"/><XMRef idref="S0.E1.m1.23" xml:id="S0.E1.m1.1.8.3"/></XMApp></XMApp><XMTok role="PERIOD"/></XMWrap></XMDual><XMArray colsep="0pt" name="aligned" xml:id="S0.E1.m1.22a.1.2"><XMRow xml:id="S0.E1.m1.22a.1.2.1"><XMCell align="right" xml:id="S0.E1.m1.22a.1.2.1.1"><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.2">q</XMTok></XMCell><XMCell align="left" xml:id="S0.E1.m1.22a.1.2.1.2"><XMApp xml:id="S0.E1.m1.22a.1.2.1.2.1"><XMTok meaning="equals" role="RELOP" xml:id="S0.E1.m1.3">=</XMTok><XMTok meaning="absent"/><XMApp xml:id="S0.E1.m1.22a.1.2.1.2.1.2"><XMTok role="SUPERSCRIPTOP" scriptpos="post6"/><XMDual xml:id="S0.E1.m1.4"><XMApp xml:id="S0.E1.m1.4.2"><XMTok meaning="binomial" name="binom" role="UNKNOWN"/><XMRef idref="S0.E1.m1.6" xml:id="S0.E1.m1.4.2.2"/><XMRef idref="S0.E1.m1.7" xml:id="S0.E1.m1.4.2.3"/></XMApp><XMWrap xml:id="S0.E1.m1.4.3"><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp xml:id="S0.E1.m1.5"><XMTok mathstyle="display" role="FRACOP" thickness="0pt"/><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.6">n</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.7">k</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER" xml:id="S0.E1.m1.8">2</XMTok></XMApp></XMApp></XMCell></XMRow><XMRow xml:id="S0.E1.m1.22a.1.2.2"><XMCell align="right" xml:id="S0.E1.m1.22a.1.2.2.1"><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.9">p</XMTok></XMCell><XMCell align="left" xml:id="S0.E1.m1.22a.1.2.2.2"><XMApp xml:id="S0.E1.m1.22a.1.2.2.2.1"><XMTok meaning="equals" role="RELOP" xml:id="S0.E1.m1.10">=</XMTok><XMTok meaning="absent"/><XMDual xml:id="S0.E1.m1.11"><XMApp xml:id="S0.E1.m1.11.2"><XMTok meaning="binomial" name="tbinom" role="UNKNOWN"/><XMRef idref="S0.E1.m1.13" xml:id="S0.E1.m1.11.2.2"/><XMRef idref="S0.E1.m1.14" xml:id="S0.E1.m1.11.2.3"/></XMApp><XMWrap xml:id="S0.E1.m1.11.3"><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp xml:id="S0.E1.m1.12"><XMTok mathstyle="text" role="FRACOP" thickness="0pt"/><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.E1.m1.13">a</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.E1.m1.14">b</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMCell></XMRow><XMRow xml:id="S0.E1.m1.22a.1.2.3"><XMCell align="right" xml:id="S0.E1.m1.22a.1.2.3.1"><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.15">r</XMTok></XMCell><XMCell align="left" xml:id="S0.E1.m1.22a.1.2.3.2"><XMWrap xml:id="S0.E1.m1.22a.1.2.3.2.1"><XMApp xml:id="S0.E1.m1.16"><XMTok meaning="equals" role="RELOP" xml:id="S0.E1.m1.17">=</XMTok><XMTok meaning="absent"/><XMApp xml:id="S0.E1.m1.16.2"><XMTok meaning="plus" role="ADDOP" xml:id="S0.E1.m1.18">+</XMTok><XMDual xml:id="S0.E1.m1.19"><XMApp xml:id="S0.E1.m1.19.2"><XMTok meaning="binomial" name="dbinom" role="UNKNOWN"/><XMRef idref="S0.E1.m1.21" xml:id="S0.E1.m1.19.2.2"/><XMRef idref="S0.E1.m1.22" xml:id="S0.E1.m1.19.2.3"/></XMApp><XMWrap xml:id="S0.E1.m1.19.3"><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp xml:id="S0.E1.m1.20"><XMTok mathstyle="display" role="FRACOP" thickness="0pt"/><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.21">c</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.22">d</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual><XMApp xml:id="S0.E1.m1.23"><XMTok mathstyle="display" role="FRACOP" thickness="0.0pt"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMApp></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMCell></XMRow></XMArray></XMDual></XMath></Math></equation>"#,
    )],
  );
}

/// A document's `\atop` redefined BEFORE amsmath loads: `\binom` is still the kernel's fraction (amsmath.sty:239-241
/// `\genfrac`). pdflatex: the binomials intact, and 1 error, amsgen's "Unable to properly define \@@atop; primitive
/// \atop no longer primitive" (amsgen.sty:27-38), which our `\@saveprimitive` (Perl's `\let`, amsgen_sty.rs) does not
/// raise. 77q r1's `\binom` on the saved `\@@atop` lost its lower part (`n ⁢ (FRACOP k absent)`).
#[test]
fn binom_ignores_an_atop_redefined_before_amsmath() {
  assert_elements_with(
    r"\documentclass{article}
\renewcommand{\atop}[2]{\genfrac{}{}{0pt}{}{#1}{#2}}
\usepackage{amsmath}
\begin{document}
\begin{equation}
q=\binom{n}{k}+\tbinom{a}{b}+\dbinom{c}{d}
\end{equation}
After.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[],
    &[(
      "equation",
      "S0.E1",
      r#"<equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="q=\binom{n}{k}+\tbinom{a}{b}+\dbinom{c}{d}" text="q = binomial@(n, k) + binomial@(a, b) + binomial@(c, d)" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.10"><XMApp xml:id="S0.E1.m1.10.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">q</XMTok><XMApp xml:id="S0.E1.m1.10.1.3"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMDual xml:id="S0.E1.m1.10.1.3.2"><XMApp xml:id="S0.E1.m1.10.1.3.2.1"><XMTok meaning="binomial" name="binom" role="UNKNOWN"/><XMRef idref="S0.E1.m1.2" xml:id="S0.E1.m1.10.1.3.2.1.2"/><XMRef idref="S0.E1.m1.3" xml:id="S0.E1.m1.10.1.3.2.1.3"/></XMApp><XMWrap xml:id="S0.E1.m1.10.1.3.2.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp xml:id="S0.E1.m1.1"><XMTok mathstyle="display" role="FRACOP" thickness="0pt"/><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.2">n</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.3">k</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual><XMDual xml:id="S0.E1.m1.10.1.3.3"><XMApp xml:id="S0.E1.m1.10.1.3.3.1"><XMTok meaning="binomial" name="tbinom" role="UNKNOWN"/><XMRef idref="S0.E1.m1.5" xml:id="S0.E1.m1.10.1.3.3.1.2"/><XMRef idref="S0.E1.m1.6" xml:id="S0.E1.m1.10.1.3.3.1.3"/></XMApp><XMWrap xml:id="S0.E1.m1.10.1.3.3.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp xml:id="S0.E1.m1.4"><XMTok mathstyle="text" role="FRACOP" thickness="0pt"/><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.E1.m1.5">a</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.E1.m1.6">b</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual><XMDual xml:id="S0.E1.m1.10.1.3.4"><XMApp xml:id="S0.E1.m1.10.1.3.4.1"><XMTok meaning="binomial" name="dbinom" role="UNKNOWN"/><XMRef idref="S0.E1.m1.8" xml:id="S0.E1.m1.10.1.3.4.1.2"/><XMRef idref="S0.E1.m1.9" xml:id="S0.E1.m1.10.1.3.4.1.3"/></XMApp><XMWrap xml:id="S0.E1.m1.10.1.3.4.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMApp xml:id="S0.E1.m1.7"><XMTok mathstyle="display" role="FRACOP" thickness="0pt"/><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.8">c</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.9">d</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math></equation>"#,
    )],
  );
}

/// setstack.sty (IOP 1998; in neither TeX Live nor 1008.1543, 1306.6377, 2206.02651) copies amsmath's `\substack`,
/// `\overset`, `\text`, `\boxed`, …: the stand-in binding gives the amsmath bindings' own. pdflatex (0803.1550's
/// copy): 0 errors.
#[test]
fn setstack_defines_the_amsmath_stacking_commands() {
  assert_elements(
    r"\documentclass{article}
\usepackage{amsthm,setstack}
\begin{document}
\begin{eqnarray}
Z=\sum_{\substack{a\cr b}} x \quad \text{and} \quad \boxed{E=mc^2}
\end{eqnarray}
After.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "equation",
      "S0.E1",
      r#"<equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle Z=\sum_{\begin{subarray}{c}a\cr b\end{subarray}}x\quad\text{and}\quad\boxed{E=mc^{2}}" text="fragments@(Z = (sum _ (list@(Array[[a], [b]])))@(x), [and], E = m * c ^ 2)" xml:id="S0.E1.m2"><XMath xml:id="S0.E1.m2.4a"><XMDual xml:id="S0.E1.m2.4a.1"><XMApp xml:id="S0.E1.m2.4a.1.1"><XMTok meaning="fragments"/><XMRef idref="S0.E1.m2.1" xml:id="S0.E1.m2.4a.1.1.2"/><XMRef idref="S0.E1.m2.3" xml:id="S0.E1.m2.4a.1.1.3"/><XMRef idref="S0.E1.m2.4" xml:id="S0.E1.m2.4a.1.1.4"/></XMApp><XMWrap xml:id="S0.E1.m2.4a.1.2"><XMApp xml:id="S0.E1.m2.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">Z</XMTok><XMApp xml:id="S0.E1.m2.1.3"><XMApp xml:id="S0.E1.m2.1.3.1"><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMDual xml:id="S0.E1.m2.1.3.1.3"><XMApp xml:id="S0.E1.m2.1.3.1.3.1"><XMTok meaning="list"/><XMRef idref="S0.E1.m2.2" xml:id="S0.E1.m2.1.3.1.3.1.2"/></XMApp><XMArray rowsep="0pt" xml:id="S0.E1.m2.2"><XMRow xml:id="S0.E1.m2.2.1"><XMCell align="center" xml:id="S0.E1.m2.2.1.1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMCell></XMRow><XMRow xml:id="S0.E1.m2.2.2"><XMCell align="center" xml:id="S0.E1.m2.2.2.1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">b</XMTok></XMCell></XMRow></XMArray></XMDual></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp><XMHint name="quad" role="PUNCT" width="10pt" xml:id="S0.E1.m2.4a.1.2.1"/><XMText xml:id="S0.E1.m2.3">and</XMText><XMHint name="quad" role="PUNCT" width="10pt" xml:id="S0.E1.m2.4a.1.2.2"/><XMApp enclose="box" xml:id="S0.E1.m2.4"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">E</XMTok><XMApp xml:id="S0.E1.m2.4.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="S0.E1.m2.4.3.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMApp></XMWrap></XMDual></XMath></Math><MathBranch><td align="right" xml:id="S0.E1.1"><Math mode="inline" tex="\displaystyle Z=\sum_{\begin{subarray}{c}a\cr b\end{subarray}}x\quad\text{and}\quad\boxed{E=mc^{2}}" text="fragments@(Z = (sum _ (list@(Array[[a], [b]])))@(x), [and], E = m * c ^ 2)" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.5"><XMDual xml:id="S0.E1.m1.5.1"><XMApp xml:id="S0.E1.m1.5.1.1"><XMTok meaning="fragments"/><XMRef idref="S0.E1.m1.1" xml:id="S0.E1.m1.5.1.1.2"/><XMRef idref="S0.E1.m1.3" xml:id="S0.E1.m1.5.1.1.3"/><XMRef idref="S0.E1.m1.4" xml:id="S0.E1.m1.5.1.1.4"/></XMApp><XMWrap xml:id="S0.E1.m1.5.1.2"><XMApp xml:id="S0.E1.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">Z</XMTok><XMApp xml:id="S0.E1.m1.1.3"><XMApp xml:id="S0.E1.m1.1.3.1"><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMDual xml:id="S0.E1.m1.1.3.1.3"><XMApp xml:id="S0.E1.m1.1.3.1.3.1"><XMTok meaning="list"/><XMRef idref="S0.E1.m1.2" xml:id="S0.E1.m1.1.3.1.3.1.2"/></XMApp><XMArray rowsep="0pt" xml:id="S0.E1.m1.2"><XMRow xml:id="S0.E1.m1.2.1"><XMCell align="center" xml:id="S0.E1.m1.2.1.1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMCell></XMRow><XMRow xml:id="S0.E1.m1.2.2"><XMCell align="center" xml:id="S0.E1.m1.2.2.1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">b</XMTok></XMCell></XMRow></XMArray></XMDual></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp><XMHint name="quad" role="PUNCT" width="10pt" xml:id="S0.E1.m1.5.1.2.1"/><XMText xml:id="S0.E1.m1.3">and</XMText><XMHint name="quad" role="PUNCT" width="10pt" xml:id="S0.E1.m1.5.1.2.2"/><XMApp enclose="box" xml:id="S0.E1.m1.4"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">E</XMTok><XMApp xml:id="S0.E1.m1.4.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="S0.E1.m1.4.3.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMApp></XMWrap></XMDual></XMath></Math></td></MathBranch></MathFork></equation>"#,
    )],
  );
}

/// proof.sty's premises are its own `\halign` (proof.sty:164-167): amsmath's `\\` there ends a row of premises, not
/// the `{align*}`'s (2011.13396). pdflatex: 0 errors, the premises in two rows. KNOWN_PERL_ERRORS #995.
#[test]
fn proof_premise_rows_stay_in_the_inference() {
  assert_elements(
    r"\documentclass{article}
\usepackage{amsmath,proof}
\begin{document}
\begin{align*}
\infer[r]{C}{A & \\ B}
\end{align*}
\[ \infer{C}{a = b & x \\[1pt] {c + d}} \]
\begin{align*}
\infer{E}{s \\ [t] u}
\end{align*}
After.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "equation",
        "S0.Ex1",
        r#"<equation xml:id="S0.Ex1"><MathFork><Math tex="\displaystyle CA\\B" text="infer@(C, Array[[A], [B]])" xml:id="S0.Ex1.m2"><XMath xml:id="S0.Ex1.m2.1a"><XMDual xml:id="S0.Ex1.m2.1a.1"><XMApp xml:id="S0.Ex1.m2.1a.1.1"><XMTok meaning="infer"/><XMRef idref="S0.Ex1.m2.2" xml:id="S0.Ex1.m2.1a.1.1.2"/><XMRef idref="S0.Ex1.m2.1" xml:id="S0.Ex1.m2.1a.1.1.3"/></XMApp><XMArray vattach="bottom" xml:id="S0.Ex1.m2.1a.1.2"><XMRow xml:id="S0.Ex1.m2.1a.1.2.1"><XMCell xml:id="S0.Ex1.m2.1a.1.2.1.1"><XMArray xml:id="S0.Ex1.m2.1"><XMRow xml:id="S0.Ex1.m2.1.1"><XMCell xml:id="S0.Ex1.m2.1.1.1"><XMTok font="italic" role="UNKNOWN">A</XMTok></XMCell></XMRow><XMRow xml:id="S0.Ex1.m2.1.2"><XMCell xml:id="S0.Ex1.m2.1.2.1"><XMTok font="italic" role="UNKNOWN">B</XMTok></XMCell></XMRow></XMArray></XMCell><XMCell rowspan="3" xml:id="S0.Ex1.m2.1a.1.2.1.2"><XMTok font="italic" role="UNKNOWN">r</XMTok></XMCell></XMRow><XMRow xml:id="S0.Ex1.m2.1a.1.2.2"><XMCell xml:id="S0.Ex1.m2.1a.1.2.2.1"><XMApp xml:id="S0.Ex1.m2.1a.1.2.2.1.1"><XMTok font="italic" role="OVERACCENT">‾</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m2.2">C</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math><MathBranch><td align="right" xml:id="S0.Ex1.1"><Math mode="inline" tex="\displaystyle CA\\B" text="infer@(C, Array[[A], [B]])" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.4"><XMDual xml:id="S0.Ex1.m1.4.1"><XMApp xml:id="S0.Ex1.m1.4.1.1"><XMTok meaning="infer"/><XMRef idref="S0.Ex1.m1.2" xml:id="S0.Ex1.m1.4.1.1.2"/><XMRef idref="S0.Ex1.m1.1" xml:id="S0.Ex1.m1.4.1.1.3"/></XMApp><XMArray vattach="bottom" xml:id="S0.Ex1.m1.4.1.2"><XMRow xml:id="S0.Ex1.m1.4.1.2.1"><XMCell xml:id="S0.Ex1.m1.4.1.2.1.1"><XMArray xml:id="S0.Ex1.m1.1"><XMRow xml:id="S0.Ex1.m1.1.1"><XMCell xml:id="S0.Ex1.m1.1.1.1"><XMTok font="italic" role="UNKNOWN">A</XMTok></XMCell></XMRow><XMRow xml:id="S0.Ex1.m1.1.2"><XMCell xml:id="S0.Ex1.m1.1.2.1"><XMTok font="italic" role="UNKNOWN">B</XMTok></XMCell></XMRow></XMArray></XMCell><XMCell rowspan="3" xml:id="S0.Ex1.m1.4.1.2.1.2"><XMTok font="italic" role="UNKNOWN">r</XMTok></XMCell></XMRow><XMRow xml:id="S0.Ex1.m1.4.1.2.2"><XMCell xml:id="S0.Ex1.m1.4.1.2.2.1"><XMApp xml:id="S0.Ex1.m1.4.1.2.2.1.1"><XMTok font="italic" role="OVERACCENT">‾</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">C</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math></td></MathBranch></MathFork></equation>"#,
      ),
      (
        "equation",
        "S0.Ex2",
        r#"<equation xml:id="S0.Ex2"><Math mode="display" tex="C\lx@proof@logical@and a=bx\\{c+d}" text="infer@(C, Array[[(a = b) and x], [c + d]])" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.3"><XMDual xml:id="S0.Ex2.m1.3.1"><XMApp xml:id="S0.Ex2.m1.3.1.1"><XMTok meaning="infer"/><XMRef idref="S0.Ex2.m1.2" xml:id="S0.Ex2.m1.3.1.1.2"/><XMRef idref="S0.Ex2.m1.1" xml:id="S0.Ex2.m1.3.1.1.3"/></XMApp><XMArray vattach="bottom" xml:id="S0.Ex2.m1.3.1.2"><XMRow xml:id="S0.Ex2.m1.3.1.2.1"><XMCell xml:id="S0.Ex2.m1.3.1.2.1.1"><XMArray xml:id="S0.Ex2.m1.1"><XMRow xml:id="S0.Ex2.m1.1.1"><XMCell xml:id="S0.Ex2.m1.1.1.1"><XMApp xml:id="S0.Ex2.m1.1.1.1.1"><XMTok meaning="and" name="lx@proof@logical@and" role="ADDOP"> </XMTok><XMApp xml:id="S0.Ex2.m1.1.1.1.1.2"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMCell></XMRow><XMRow xml:id="S0.Ex2.m1.1.2"><XMCell xml:id="S0.Ex2.m1.1.2.1"><XMApp xml:id="S0.Ex2.m1.1.2.1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMCell></XMRow></XMArray></XMCell></XMRow><XMRow xml:id="S0.Ex2.m1.3.1.2.2"><XMCell xml:id="S0.Ex2.m1.3.1.2.2.1"><XMApp xml:id="S0.Ex2.m1.3.1.2.2.1.1"><XMTok font="italic" role="OVERACCENT">‾</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.2">C</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math></equation>"#,
      ),
      (
        "equation",
        "S0.Ex3",
        r#"<equation xml:id="S0.Ex3"><MathFork><Math tex="\displaystyle Es\\[t]u" text="infer@(E, Array[[s], [delimited-[]@(t) * u]])" xml:id="S0.Ex3.m2"><XMath xml:id="S0.Ex3.m2.1a"><XMDual xml:id="S0.Ex3.m2.1a.1"><XMApp xml:id="S0.Ex3.m2.1a.1.1"><XMTok meaning="infer"/><XMRef idref="S0.Ex3.m2.3" xml:id="S0.Ex3.m2.1a.1.1.2"/><XMRef idref="S0.Ex3.m2.1" xml:id="S0.Ex3.m2.1a.1.1.3"/></XMApp><XMArray vattach="bottom" xml:id="S0.Ex3.m2.1a.1.2"><XMRow xml:id="S0.Ex3.m2.1a.1.2.1"><XMCell xml:id="S0.Ex3.m2.1a.1.2.1.1"><XMArray xml:id="S0.Ex3.m2.1"><XMRow xml:id="S0.Ex3.m2.1.2"><XMCell xml:id="S0.Ex3.m2.1.2.1"><XMTok font="italic" role="UNKNOWN">s</XMTok></XMCell></XMRow><XMRow xml:id="S0.Ex3.m2.1.3"><XMCell xml:id="S0.Ex3.m2.1.3.1"><XMApp xml:id="S0.Ex3.m2.1.3.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual xml:id="S0.Ex3.m2.1.3.1.1.2"><XMApp xml:id="S0.Ex3.m2.1.3.1.1.2.1"><XMTok meaning="delimited-[]"/><XMRef idref="S0.Ex3.m2.2" xml:id="S0.Ex3.m2.1.3.1.1.2.1.2"/></XMApp><XMWrap xml:id="S0.Ex3.m2.1.3.1.1.2.2"><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m2.2">t</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMCell></XMRow></XMArray></XMCell></XMRow><XMRow xml:id="S0.Ex3.m2.1a.1.2.2"><XMCell xml:id="S0.Ex3.m2.1a.1.2.2.1"><XMApp xml:id="S0.Ex3.m2.1a.1.2.2.1.1"><XMTok font="italic" role="OVERACCENT">‾</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m2.3">E</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math><MathBranch><td align="right" xml:id="S0.Ex3.1"><Math mode="inline" tex="\displaystyle Es\\[t]u" text="infer@(E, Array[[s], [delimited-[]@(t) * u]])" xml:id="S0.Ex3.m1"><XMath xml:id="S0.Ex3.m1.3a"><XMDual xml:id="S0.Ex3.m1.3a.1"><XMApp xml:id="S0.Ex3.m1.3a.1.1"><XMTok meaning="infer"/><XMRef idref="S0.Ex3.m1.3" xml:id="S0.Ex3.m1.3a.1.1.2"/><XMRef idref="S0.Ex3.m1.1" xml:id="S0.Ex3.m1.3a.1.1.3"/></XMApp><XMArray vattach="bottom" xml:id="S0.Ex3.m1.3a.1.2"><XMRow xml:id="S0.Ex3.m1.3a.1.2.1"><XMCell xml:id="S0.Ex3.m1.3a.1.2.1.1"><XMArray xml:id="S0.Ex3.m1.1"><XMRow xml:id="S0.Ex3.m1.1.2"><XMCell xml:id="S0.Ex3.m1.1.2.1"><XMTok font="italic" role="UNKNOWN">s</XMTok></XMCell></XMRow><XMRow xml:id="S0.Ex3.m1.1.3"><XMCell xml:id="S0.Ex3.m1.1.3.1"><XMApp xml:id="S0.Ex3.m1.1.3.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual xml:id="S0.Ex3.m1.1.3.1.1.2"><XMApp xml:id="S0.Ex3.m1.1.3.1.1.2.1"><XMTok meaning="delimited-[]"/><XMRef idref="S0.Ex3.m1.2" xml:id="S0.Ex3.m1.1.3.1.1.2.1.2"/></XMApp><XMWrap xml:id="S0.Ex3.m1.1.3.1.1.2.2"><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.2">t</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMCell></XMRow></XMArray></XMCell></XMRow><XMRow xml:id="S0.Ex3.m1.3a.1.2.2"><XMCell xml:id="S0.Ex3.m1.3a.1.2.2.1"><XMApp xml:id="S0.Ex3.m1.3a.1.2.2.1.1"><XMTok font="italic" role="OVERACCENT">‾</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.3">E</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math></td></MathBranch></MathFork></equation>"#,
      ),
    ],
  );
}

/// In a tikz-cd cell `&` is active (`\pgfmatrixnextcell`, tikz.code.tex:4064-4068), and proof.sty's premises still
/// split at it (2205.02142: 44 errors). pdflatex: 0 errors.
#[test]
fn proof_premises_split_on_an_active_ampersand() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsmath,proof,tikz}
\usetikzlibrary{cd}
\begin{document}
\begin{tikzcd}
A & \vcenter{\infer{C}{a & b}} \ar[l] \\
 P & Q
\end{tikzcd}
After.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &["text=\"infer@(C, a and b)\""],
    r#"<Math mode="inline" tex="C\lx@proof@logical@and ab" text="infer@(C, a and b)" xml:id="p1.pic1.m2.m1"><XMath xml:id="p1.pic1.m2.m1.3"><XMDual xml:id="p1.pic1.m2.m1.3.1"><XMApp xml:id="p1.pic1.m2.m1.3.1.1"><XMTok meaning="infer"/><XMRef idref="p1.pic1.m2.m1.2" xml:id="p1.pic1.m2.m1.3.1.1.2"/><XMRef idref="p1.pic1.m2.m1.1" xml:id="p1.pic1.m2.m1.3.1.1.3"/></XMApp><XMArray vattach="bottom" xml:id="p1.pic1.m2.m1.3.1.2"><XMRow xml:id="p1.pic1.m2.m1.3.1.2.1"><XMCell xml:id="p1.pic1.m2.m1.3.1.2.1.1"><XMApp xml:id="p1.pic1.m2.m1.1"><XMTok meaning="and" name="lx@proof@logical@and" role="ADDOP"> </XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMCell></XMRow><XMRow xml:id="p1.pic1.m2.m1.3.1.2.2"><XMCell xml:id="p1.pic1.m2.m1.3.1.2.2.1"><XMApp xml:id="p1.pic1.m2.m1.3.1.2.2.1.1"><XMTok font="italic" role="OVERACCENT">‾</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.pic1.m2.m1.2">C</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math>"#,
  );
}

/// minted's options are its own (minted.sty:1005 `mathescape=false`, :1013 `texcl=false`, :1019 `escapeinside=`): a
/// document's `\lstset{mathescape=true,escapechar=|}` for its listings does not reach a minted body that listings
/// stands in for (1812.09490: the `$` of `sed "s/$/…/"` opened math). pdflatex: minted's own errors only (no
/// Pygments, a harness limit).
#[test]
fn minted_body_ignores_the_document_listings_escapes() {
  assert_elements(
    r#"\documentclass{article}
\usepackage[draft=true]{minted}
\usepackage{listings}
\lstset{mathescape=true,escapechar=|}
\begin{document}
\begin{minted}{bash}
sed "s/$/ -a 127/" | bash
\end{minted}
After.
\end{document}"#,
    "ar5iv.sty",
    (0, 0),
    &[(
      "listing",
      "p1.1",
      r#"<listing class="ltx_lstlisting" data="c2VkICJzLyQvIC1hIDEyNy8iIHwgYmFzaA==" dataencoding="base64" datamimetype="text/plain" xml:id="p1.1"><listingline xml:id="lstnumberx1"><text class="ltx_lst_keyword" font="bold" xml:id="lstnumberx1.1">sed</text><text class="ltx_lst_space" xml:id="lstnumberx1.2"> </text><text class="ltx_lst_string" xml:id="lstnumberx1.3">”s/$/<text class="ltx_lst_space" xml:id="lstnumberx1.3.1">␣</text>-a<text class="ltx_lst_space" xml:id="lstnumberx1.3.2">␣</text>127/”</text><text class="ltx_lst_space" xml:id="lstnumberx1.4"> </text>|<text class="ltx_lst_space" xml:id="lstnumberx1.5"> </text><text class="ltx_lst_identifier" xml:id="lstnumberx1.6">bash</text></listingline></listing>"#,
    )],
  );
}
