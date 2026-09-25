//! Red/green guards for perfect-kernel batch 49 (PLANS P53/P51/P54a).
use super::perfect_kernel_batch46::{convert, error_count};

/// P53: `\@currenvir` for a `DefEnvironment!` environment is the
/// environment name's CHARACTER tokens (Perl Package.pm:1927 `DefMacroI`
/// → TokenizeInternal; latex.ltx:15350 `\def\@currenvir{#1}`), so the
/// `\ifx\reserved@a\@currenvir` idiom (`\@checkend` latex.ltx:15394,
/// collectbox:208, storebox:36, nag:258, powerdot:529 …) matches. RED: one
/// multi-char letter token — stringified right, never `\ifx`-equal, so
/// every such consumer took the wrong branch silently (Perl: `SAME`).
/// `lstlisting` (listings binding) set the same single-token shape.
#[test]
fn currenvir_is_character_tokens() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{listings}
\makeatletter
\def\check#1{\def\a{#1}\edef\b{\@currenvir}\ifx\a\b SAME\else DIFF\fi}
\begin{document}
\begin{center}\check{center}\end{center}
\begin{itemize}\item \check{itemize}\end{itemize}
\begin{lstlisting}[title={\check{lstlisting}}]
x
\end{lstlisting}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("DIFF"), "{xml}");
  // center, itemize, and the listing title (rendered as title + caption).
  assert_eq!(xml.matches("SAME").count(), 4, "{xml}");
}

/// P51: `\definecolor[ps]{name}{model}{PostScript}` still REGISTERS the
/// color, with the model's white as its non-PostScript value
/// (xcolor.sty:531-533, `\XC@clr@<model>@white` :510-516). RED: dropped
/// entirely (Perl xcolor.sty.ltxml:403-409 too — KNOWN_PERL_ERRORS), so
/// `\color{lambda}` errored 101× → `Fatal:TooManyErrors`. Witness
/// xcolor/xcolor2 (xcolor2.tex:143 defines, :134 uses in `\multiput`).
#[test]
fn xcolor_ps_color_registers_as_model_white() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\definecolor[ps]{lambda}{rgb}{Red Corr Green Corr Blue Corr}
\providecolor[ps]{mu}{cmyk}{0 0 0 0}
\textcolor{lambda}{hello} and \textcolor{mu}{there}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("Ignoring definition of postscript color"),
    "{stderr}"
  );
  assert!(xml.contains("<text color=\"#FFFFFF\">hello"), "{xml}");
  assert!(xml.contains("<text color=\"#FFFFFF\">there"), "{xml}");
}

/// P54a: ctex's pdfTeX layer requires CJKpunct (ctex-engine-pdftex.def:122),
/// whose raw `\CJKpunct@utfasymbol` (CJKpunct.sty:449) routes the six
/// declared punctuation codepoints through `\CJK@punctchar{\CJK@uniPunct}…`
/// — supplied by CJK.enc:291 / `*.chr` in real CJK, never loaded behind the
/// CJK binding. RED: 2 `undefined` per doc across 18 ctex manuals
/// (Perl identical). Witnesses: jnuexam/jnuexam (2→0), joinbox/joinbox,
/// suanpan-l3/suanpan-l3. `fontset=none`: the default fandol fontset depends
/// on the host tree, and on the TL 2025 vendor tree pdflatex itself stops with
/// "CTeX fontset `fandol' is unavailable in current mode".
#[test]
fn ctex_cjkpunct_unicode_punctuation() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[scheme=plain,fontset=none]{ctex}
\begin{document}
A“B”C—D…E‘F’G
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("CJK@punctchar") && !stderr.contains("CJK@uniPunct"),
    "{stderr}"
  );
  assert!(xml.contains("A“B”C—D…E‘F’G"), "{xml}");
}
