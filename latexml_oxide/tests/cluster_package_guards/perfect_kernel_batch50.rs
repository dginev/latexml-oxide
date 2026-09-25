use super::perfect_kernel_batch46::{convert, error_count};

/// P50: `\filename@parse` is latex.ltx:228-281's own macro, so
/// `\filename@area/base/ext` keep the ARGUMENT's tokens. The Perl-shaped
/// primitive re-lettered the pieces (`ExplodeText`), so a caller that
/// `\@onelevel@sanitize`d its argument first (currfile.sty:78-85 and its
/// `\ifx\@tempa\currfilename` compare; import.sty; docstrip) got a catcode
/// mismatch real LaTeX never has. RED: T4/T5 answered `no`. Witnesses: the
/// currfile users in the TL doc corpus (pythontex/pythontex, knowledge/
/// knowledge, milsymb/milsymb, dlrg/dlrg), repro `cf4.tex`.
#[test]
fn filename_parse_keeps_argument_catcodes() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\begin{document}
\edef\x{abc}\@onelevel@sanitize\x \edef\y{abc}
T1:\ifx\x\y yes\else no\fi

\def\@filef@und{sub/dir/cf4.tar.tex}\@onelevel@sanitize\@filef@und
\expandafter\filename@parse\expandafter{\@filef@und}
\edef\a{\filename@base}\edef\b{\filename@ext}\edef\c{\filename@area}
\edef\p{\detokenize{cf4.tar}}\edef\q{\detokenize{tex}}\edef\r{\detokenize{sub/dir/}}
T4:\ifx\a\p yes\else no\fi
T5:\ifx\b\q yes\else no\fi
T6:\ifx\c\r yes\else no\fi

\filename@parse{plain}
T7:\ifx\filename@ext\relax yes\else no\fi
\edef\d{\filename@base}\edef\e{plain}
T8:\ifx\d\e yes\else no\fi
\end{document}
",
    false,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for probe in ["T1:no", "T4:yes", "T5:yes", "T6:yes", "T7:yes", "T8:yes"] {
    assert!(xml.contains(probe), "missing {probe}\n{xml}");
  }
}

/// calc `\widthof{$…$}` evaluated INSIDE inline math (mhchem.sty:2898-2904
/// `\mhchem@minispace`, run from the prescript path `\ce{^{227}Th}` under an
/// open `\ensuremath`, :2781): Perl calc.sty.ltxml:140 measures the argument
/// in a fresh `restricted_horizontal` box, so its nested `$…$` is its own
/// math; the Rust port digested it in the ambient (math) mode and the inner
/// `$` closed the ENCLOSING math frame. RED: 6× `Error:unexpected:
/// \lx@end@inline@math Attempt to end mode math` (Perl 0). Witness: TL doc
/// corpus mhchem/mhchem (67 → 13; the rest is the SHARED `\ce` in `align*`
/// R3c family).
#[test]
fn calc_widthof_in_math_measures_own_box() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[version=4]{mhchem}
\usepackage{calc}
\newlength\mylen
\begin{document}
\ce{^{227}Th}

X $a\setlength{\mylen}{\widthof{$b$}}c$ Y
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("lx@end@inline@math"), "{stderr}");
  // Prescript stays inside ONE well-formed inline Math.
  assert!(
    xml.contains(r#"<XMTok role="SUPERSCRIPTOP" scriptpos="pre1"/>"#),
    "{xml}"
  );
  assert!(xml.contains("227"), "{xml}");
  assert_eq!(xml.matches("<Math ").count(), 2, "{xml}");
  assert!(
    xml.contains("X <Math") && xml.contains("</Math> Y"),
    "{xml}"
  );
}

/// verbatim.sty:107-112 `\verbatim@start#1` swallows a following control
/// sequence (`\if\noexpand#1\noexpand~` is true for any CS), which is the
/// documented `\verbatim@start\relax` idiom for opening a capture from a
/// macro body (curve2e-manual.tex:95 `{Esempio}`, newfile.sty:131
/// `\writeverbatim`). RED: the pending `\relax` was serialised as the
/// first captured line (`<verbatim>\relax\n…`), and the `\write`-then-
/// `\verbatiminput` round trip carried it. Oracle pdflatex: no such line.
/// Witnesses: curve2e/curve2e-manual (100+Fatal → 41, all 26
/// `missing_file` gone), digiconfigs/digiconfigs (9 → 5).
#[test]
fn verbatim_start_relax_idiom() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{verbatim}
\makeatletter
\newwrite\example@out
\newenvironment{Esempio}{\par
\begingroup
\@bsphack
\immediate\openout\example@out\jobname-temp.tex
\let\do\@makeother\dospecials\catcode`\^^M\active
\def\verbatim@processline{%
  \immediate\write\example@out{\the\verbatim@line}}%
\verbatim@start\relax}%
{\immediate\closeout\example@out\@esphack\endgroup
\verbatiminput{\jobname-temp.tex}
\input{\jobname-temp}%
\par}
\makeatother
\begin{document}
before
\begin{Esempio}
Hello \textbf{world} 1
second line
\end{Esempio}
after
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains(r"\relax"), "{xml}");
  assert!(
    xml.contains("<verbatim font=\"typewriter\">Hello \\textbf{world} 1\nsecond line\n</verbatim>"),
    "{xml}"
  );
  assert!(
    xml.contains("<p>Hello <text font=\"bold\">world</text> 1\nsecond line</p>"),
    "{xml}"
  );
}
