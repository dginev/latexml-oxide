//! Red/green guards for batch 66b: run 340's Fatal origins still Fatal at the int8 head (cortex `Stomach:Recursion`,
//! `Timeout:PushbackLimit`, `Mouth:EoF`, `Document:Malformed`; sample `~/data/pk_agents/main/run340/fatal/`).
//! Repros `tools/perfect_kernel/repros/*`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// The Cambridge University Press classes keep LaTeX's `\caption` as `\@@caption` for their `\contcaption`
/// (fac.cls:803-805, ijsipaper.cls:876-878, mscs.cls:794-795), the name Perl gives the constructor of the
/// `ltx:caption` (latex_constructs.pool.ltxml:3368), so every caption called itself (Perl: `deep_recursion`).
/// The constructor is `\lx@@caption`; `\@@caption` stays its alias for bindings, and a class's own meaning
/// for it is the class's. Witnesses 1209.5152, 1311.2960, 1410.5131, 1610.02500, 2205.14586, 2212.13027,
/// 1604.04575.
#[test]
fn cup_contcaption_keeps_its_caption() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/cup_contcaption_lets_caption.tex"
  ));
  assert_eq!(xml.matches("<figure ").count(), 3, "{xml}");
  assert_eq!(
    xml.matches(r#"<tag role="refnum">1</tag>"#).count(),
    2,
    "{xml}"
  );
  assert_eq!(
    xml.matches(r#"<tag role="refnum">2</tag>"#).count(),
    1,
    "{xml}"
  );
  for text in [
    "First</caption>",
    "First, continued</caption>",
    "Second</caption>",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// latex.ltx:1245-1257 `\@xargdef`: a command with an optional argument is `\@protected@testopt\X\\X{default}`, and
/// the macro taking `[#1]` is the internal `\\X`, which the amsart idiom `\edef\X{\@dblarg\csname\string\X\endcsname}`
/// calls directly (bcp.cls:151-153 `\abtitle`, cmfne.cls:460,483). Without it `\@abtitle` was never set, and
/// `\uppercasenonmath\@abtitle` turned the undefined name into a definition that contains itself (Fatal
/// `Stomach:Recursion`). Witnesses 1510.00296, 1704.01336, 1207.2066, 2003.00661, 2210.02246.
#[test]
fn newcommand_optional_argument_defines_its_internal_macro() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/newcommand_optional_internal_cs.tex"
  ));
  assert!(
    xml.contains(
      "<p xml:id=\"p1.1\">T[New developments] S[New developments]\nT[Long] S[Short](a/b)</p>"
    ),
    "{xml}"
  );
}

/// A locked robust binding owns its calls through its inner `\cs␣` too: cmfne.cls:1772-1778 copies cite.sty's
/// `\@namedef{cite }{…\@citex…}` and `\@citex` (`\@cite{\citen{#2}}`), and with our cite binding's `\citen` = `\cite`
/// (Perl cite.sty.ltxml) the kernel `\cite`'s wrapper reached the class's inner, which called `\cite` again (Fatal
/// `Stomach:Recursion`). Witness 2210.02246.
#[test]
fn locked_robust_cite_keeps_its_inner_definition() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/locked_robust_inner_cite.tex"
  ));
  assert!(xml.contains(r#"<cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>"#), "{xml}");
}

/// A paper's mdpi.cls (2105.08579's, 2201.05363's, 1809.00601's, abridged) loads paracol (:55) and, after the preamble,
/// runs its own `\maketitle` and opens `\begin{paracol}{2}` for the first page's side column, then `\switchcolumn`s to
/// the body (`\AfterEndPreamble`, 2105.08579's :880-887); the template's paper closes it with `\end{paracol}` before
/// the references. The binding follows the copy the paper ships, so that `\end{paracol}` closes the environment the
/// class opened, as in pdflatex (run 340: the `\endgroup` error of ~470 mdpi papers) — once, whether or not the paper
/// also calls `\maketitle` (which the class has made a no-op).
#[test]
fn mdpi_opens_the_paracol_the_paper_closes() {
  for maketitle in ["", "\\maketitle\n"] {
    let (log, xml) = latexml::util::test::convert_files_with(
      &format!(
        r"\documentclass{{Definitions/mdpi}}
\Title{{A title}}
\begin{{document}}
{maketitle}\section{{Introduction}}
Body text.
\end{{paracol}}
\reftitle{{References}}
\end{{document}}
"
      ),
      &[(
        "Definitions/mdpi.cls",
        r"\ProvidesClass{mdpi}[2021/01/01 MDPI paper class]
\RequirePackage{paracol}
\AfterEndPreamble{\maketitle\let\maketitle\relax\columnratio{0.23}\begin{paracol}{2}\switchcolumn}
",
      )],
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{maketitle}{log}"
    );
    assert!(xml.contains("Body text."), "{xml}");
    assert!(xml.contains(">Introduction</title>"), "{xml}");
  }
}

/// An mdpi.cls copy that never opens paracol (1307.3961's 2013 one) leaves `\maketitle` as it was: nothing is left
/// open at `\end{document}`.
#[test]
fn mdpi_without_paracol_opens_nothing() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{Definitions/mdpi}
\Title{A title}
\begin{document}
\maketitle
Body text.
\end{document}
",
    &[(
      "Definitions/mdpi.cls",
      "\\ProvidesClass{mdpi}[2013/04/05 MDPI paper class]\n\\RequirePackage{graphicx}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Body text."), "{xml}");
}

/// `:locked@files` (the kernel abstract's end half, sect05.rs, the IMjournal fix) is a lock against class and package
/// files, which `install_definition` applied to a `\def` but a `\let` bypassed: arabtex's asect.sty:297
/// `\let\endabstract\endquotation`, with alatex.sty:105-109's raw `\end` calling it, closed the kernel abstract's body
/// as a quotation (2 errors). Witness 2401.14559.
#[test]
fn file_let_of_a_files_locked_end_half_is_refused() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/file_let_of_locked_end_half.tex"
  ));
  assert_eq!(xml.matches("<abstract").count(), 1, "{xml}");
  assert!(xml.contains("Abstract text."), "{xml}");
  assert!(xml.contains("Body."), "{xml}");
}

/// mn2e-breakabs.sty (shipped by MNRAS papers; :87-95) redefines only the end half of mn2e.cls's `{keywords}`
/// (:992-1012, a `\list`): the binding's keywords begin half reads its body, so the package's `\endlist` met no list
/// (2 errors). The binding's end half is locked against files, as the abstract's is (sect05.rs). Witnesses 1403.3615,
/// 1607.03149.
#[test]
fn mn2e_breakabs_keeps_the_keywords_end_half() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{mn2e}
\usepackage{mn2e-breakabs}
\begin{document}
\title{T}\author{A}
\maketitle
\begin{abstract}
Abs.
\end{abstract}
\begin{keywords}
k1; k2
\end{keywords}
\section{Intro}
Text.
\end{document}
",
    &[(
      "mn2e-breakabs.sty",
      r"\ProvidesPackage{mn2e-breakabs}[2009/04/07 v1.0 abstract breaking package]
\def\endkeywords{\if@twocolumn
  \endlist\addvspace{37pt plus 0.5\baselineskip}\finish@SFBbox
 \else
  \endlist
 \fi
 \@thanks
 \gdef\@thanks{}
 \SFB@keywordstrue
}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("k1; k2"), "{xml}");
  assert!(xml.contains("Text."), "{xml}");
}

/// A file may still clear a files-locked name before defining it anew (projlib-author.sty:245-246 `\cs_undefine:N
/// \endabstract`, then scontents' `\NewDocumentEnvironment{abstract}`; the homework and jwjournal classes): a `\let`
/// to undefined or `\relax` replaces nothing, so it is not refused, and expl3's "already defined" check passes as in
/// pdflatex.
#[test]
fn file_let_clearing_a_files_locked_name_is_kept() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{clearabs}
\begin{document}
\begin{abstract}
Abstract text.
\end{abstract}
Body.
\end{document}
",
    &[(
      "clearabs.sty",
      r"\ProvidesPackage{clearabs}
\ExplSyntaxOn
\cs_undefine:N \abstract
\cs_undefine:N \endabstract
\NewDocumentEnvironment { abstract } { } { } { }
\ExplSyntaxOff
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p xml:id=\"abstract1.1\">Abstract text.</p>"),
    "{xml}"
  );
  assert!(xml.contains("<p xml:id=\"p1.1\">Body.</p>"), "{xml}");
}
