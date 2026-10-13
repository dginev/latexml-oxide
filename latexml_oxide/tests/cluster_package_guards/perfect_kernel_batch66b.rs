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
/// 1604.04575, 2112.07588.
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

/// A locked robust binding keeps its inner `\cs␣` against the class and package files: cmfne.cls:1772-1778 copies
/// cite.sty's `\@namedef{cite }{…\@citex…}` and `\@citex` (`\@cite{\citen{#2}}`), and with our cite binding's
/// `\citen` = `\cite` (Perl cite.sty.ltxml) the kernel `\cite`'s wrapper reached the class's inner, which called `\cite`
/// again (Fatal `Stomach:Recursion`). The repro's package stands for the class. Witnesses 2210.02246, 2303.09770.
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
    assert_eq!(
      xml.matches("<title>A title</title>").count(),
      1,
      "{maketitle}{xml}"
    );
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
  let abstract_ = latexml::util::test::xml_element(&xml, "abstract", &[]).unwrap_or_default();
  assert!(
    abstract_.contains("Abstract text.") && !abstract_.contains("Body."),
    "{xml}"
  );
  let body = xml.find("<para xml:id=\"p1\">").unwrap_or(0);
  assert!(
    body > xml.find("</abstract>").unwrap_or(usize::MAX),
    "{xml}"
  );
  assert!(xml[body..].contains("Body."), "{xml}");
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
  assert!(
    xml.contains("<keywords name=\"Keywords:\u{a0}\">k1; k2\n</keywords>"),
    "{xml}"
  );
  assert!(xml.contains("Text."), "{xml}");
}

/// A package redefining both halves of `{keywords}` (`\renewenvironment{keywords}{\begin{list}…}{\end{list}}`) is
/// refused for both: the pair is locked against files, so the binding's begin half never meets the package's end half.
#[test]
fn mn2e_package_redefining_both_keywords_halves_keeps_the_pair() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{mn2e}
\usepackage{kwboth}
\begin{document}
\title{T}\author{A}
\maketitle
\begin{keywords}
k1; k2
\end{keywords}
\section{Intro}
Text.
\end{document}
",
    &[(
      "kwboth.sty",
      "\\ProvidesPackage{kwboth}\n\\renewenvironment{keywords}{\\begin{list}{}{}\\item[]}{\\end{list}}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<keywords name=\"Keywords:\u{a0}\">k1; k2\n</keywords>"),
    "{xml}"
  );
  assert!(xml.contains("Text."), "{xml}");
}

/// A file may still clear a files-locked name before defining it anew (projlib-author.sty:245-246 `\cs_undefine:N
/// \endabstract`, then scontents' `\NewDocumentEnvironment{abstract}`; the homework and jwjournal classes): a `\let`
/// to an undefined token replaces nothing, so it is not refused, and expl3's "already defined" check passes as in
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

/// When the cleared name's new definition is refused, the binding is put back (`state::restore_cleared_binding`): the
/// kernel's `\endabstract`, which the braceless `\abstract … \endabstract` form calls by name (as a raw `\end`,
/// alatex.sty:105-109, would), still closes the abstract.
#[test]
fn refused_definition_of_a_cleared_name_restores_the_binding() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{clearabs}
\begin{document}
\abstract Abstract text.
\endabstract
Body.
\end{document}
",
    &[(
      "clearabs.sty",
      "\\ProvidesPackage{clearabs}\n\\let\\endabstract\\undefined\n\\def\\endabstract{\\par}\n",
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

/// Every declarator built on `\@xargdef` defines the internal `\\cs` its optional argument reaches: the begin macro of
/// `\newenvironment`/`\renewenvironment` (latex.ltx:1324-1331) and `\DeclareRobustCommand`'s inner `\cs␣`
/// (:1412-1428, internal `\\cs␣`), as `\newcommand`'s. pdflatex: "E[a](b) Q[f](g) R[c](e)".
#[test]
fn every_xargdef_declarator_defines_the_internal_macro() {
  let xml = convert(
    r"\documentclass{article}
\newenvironment{envx}[2][d]{E[#1](#2)}{}
\renewenvironment{quote}[2][q]{Q[#1](#2)}{}
\DeclareRobustCommand\rob[2][d]{R[#1](#2)}
\begin{document}
\csname\string\envx\endcsname[a]{b} \csname\string\quote\endcsname[f]{g} \csname\string\rob\space\endcsname[c]{e}
\end{document}
",
  );
  assert!(
    xml.contains("<p xml:id=\"p1.1\">E[a](b) Q[f](g) R[c](e)</p>"),
    "{xml}"
  );
}

/// The restore holds on the `\csname` path most file definitions take (`\@namedef`, `\expandafter\def\csname…`, ltcmd's
/// `\cs_set_eq:cc`): `\csname` makes the cleared name `\relax` (tex.web §372) before the refused definition, and the
/// binding still comes back.
#[test]
fn refused_namedef_of_a_cleared_name_restores_the_binding() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{clearabs}
\begin{document}
\abstract Abstract text.
\endabstract
Body.
\end{document}
",
    &[(
      "clearabs.sty",
      "\\ProvidesPackage{clearabs}\n\\let\\endabstract\\undefined\n\\@namedef{endabstract}{\\par}\n",
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

/// The kept-aside binding comes back once: after a restore, a `\relax` the document gives the name is its own, and a
/// later package's refused `\@namedef` leaves it so (`state::restore_cleared_binding`).
#[test]
fn a_cleared_binding_is_restored_once() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{clra}
\let\endabstract\relax
\usepackage{clrb}
\begin{document}
\texttt{\meaning\endabstract}
\end{document}
",
    &[
      (
        "clra.sty",
        "\\ProvidesPackage{clra}\n\\let\\endabstract\\undefined\n\\def\\endabstract{\\par}\n",
      ),
      (
        "clrb.sty",
        "\\ProvidesPackage{clrb}\n\\@namedef{endabstract}{\\par}\n",
      ),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(">\\relax</text>"), "{xml}");
}

/// A class's `\let` of a locked robust binding to its own robust command takes `\let`'s plain path (TeX's, Perl's `Let`):
/// the binding's inner `\cite␣`, locked against the files (OXIDIZED_DESIGN #568), is not overwritten by the robust
/// wrapper copy.
#[test]
fn robust_let_over_a_locked_binding_keeps_its_inner() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{zqcite}
\begin{document}
\texttt{\expandafter\meaning\csname cite \endcsname}
\end{document}
",
    &[(
      "zqcite.sty",
      "\\ProvidesPackage{zqcite}\n\\DeclareRobustCommand\\mycite[1]{MY(#1)}\n\\let\\cite\\mycite\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(">macro:#1#2-&gt;CODE("), "{xml}");
  assert!(!xml.contains("MY("), "{xml}");
}

/// A package's `\let` of a locked constructor whose inner is free to its own robust command keeps the constructor,
/// while a `\let` to a plain macro takes `\let`'s plain path (Perl's `Let`): bera.sty:10-23 `\def\fveTeX`,
/// `\let\TeX=\fveTeX` and `\DeclareRobustCommand{\fveLaTeX}`, `\let\LaTeX=\fveLaTeX` (the enumitem, colorspace and
/// mathador manuals) leave the `\LaTeX` logo and the package's `\TeX`, as 65int8f did.
#[test]
fn robust_let_over_a_locked_constructor_keeps_it() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{article}
\usepackage{zqlogo}
\begin{document}
A \LaTeX{} and \TeX{} logo.
\end{document}
",
    &[(
      "zqlogo.sty",
      r"\ProvidesPackage{zqlogo}
\def\zqTeX{T\kern-.1667em\lower.5ex\hbox{E}\kern-.125emX}
\DeclareRobustCommand{\zqLaTeX}{L\kern-.3em{\sbox\z@ T\vbox to\ht\z@{\hbox{\scriptsize A}\vss}}\kern-.1em\zqTeX}
\let\TeX=\zqTeX
\let\LaTeX=\zqLaTeX
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("class=\"ltx_LaTeX_logo\"").count(), 1, "{xml}");
  assert_eq!(xml.matches("class=\"ltx_TeX_logo\"").count(), 0, "{xml}");
  assert!(!xml.contains("<inline-block"), "{xml}");
}

/// A definition the lock refuses still defines its `\@xargdef` internal, which the class's own `\@dblarg` wrapper calls
/// when its refused setter is replayed: cmfne.cls:462-483 `\renewcommand{\author}[2][]{…\gdef\firstshortauthor{#1}…}`
/// then `\edef\author{\@dblarg\csname\string\author\endcsname}`, whose `\enddoc@text` prints the name line
/// (2210.02246). The kernel keeps the author.
#[test]
fn refused_dblarg_setter_reaches_its_internal() {
  let (log, xml) = latexml::util::test::convert_files_with(
    r"\documentclass{cmfx}
\title{A title}
\author[Valeriy V. Katrakhov (1949--2010)]{V.~V.~Katrakhov}
\begin{document}
\maketitle
Text.
\end{document}
",
    &[(
      "cmfx.cls",
      r"\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{cmfx}
\LoadClass{article}
\renewcommand{\author}[2][]{\gdef\firstshortauthor{#1}\gdef\authors{#2}}
\edef\author{\noexpand\@dblarg\expandafter\noexpand\csname\string\author\endcsname}
\let\firstshortauthor\@empty
\AtEndDocument{\par\firstshortauthor\par}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<personname>").count(), 1, "{xml}");
  assert!(
    xml.contains("<personname>V.\u{a0}V.\u{a0}Katrakhov</personname>"),
    "{xml}"
  );
  // The class's end-of-document line, once and after the body: printed by the replayed setter's store, not a leaked
  // short author.
  assert_eq!(xml.matches("Valeriy").count(), 1, "{xml}");
  let text = xml.find("<p xml:id=\"p1.1\">Text.</p>");
  let line = xml.find("<p xml:id=\"p2.1\">Valeriy V. Katrakhov (1949–2010)</p>");
  assert!(text.is_some() && line.is_some() && text < line, "{xml}");
}
