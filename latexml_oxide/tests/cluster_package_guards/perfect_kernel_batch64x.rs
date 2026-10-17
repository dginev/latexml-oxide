//! Red/green guards for perfect-kernel batch 64x: run 336's low-risk Error clusters — a `{centering}` in a titlepage,
//! natbib's `\bibitem` outside a bibliography, a `\bibitem` after a float in the bibliography, AMS-TeX's `\end` and
//! braced `\predefine`, a package's `\endabstract` without its `\abstract`, jheppub's one-argument `\abstract`, and
//! PoS without JHEP's journal macros.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// The XML of `tex` converted under ar5iv beside `files`, which must give no error and no warning.
fn convert_clean_files(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// 64x: `{titlepage}` neutralizes `\centering` as a no-op macro, not `\relax` (Perl latex_constructs.pool.ltxml:1168),
/// so a `\begin{centering}` inside it is an environment, as in pdflatex (hep-th9703192, cond-mat9607110,
/// hep-lat0402021; ~1,078 run-336 papers). Repro sectioning-frontmatter/titlepage_begin_centering.
#[test]
fn titlepage_begin_centering_is_defined() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_begin_centering.tex"
  ));
  assert_element(
    &xml,
    "titlepage",
    &[],
    r#"<titlepage xml:id="id1"><p xml:id="id1.1">Hello</p></titlepage>"#,
  );
}

/// 64x: natbib's `\bibitem` opens the bibliography when none is open, as the kernel's does (Perl
/// latex_constructs.pool.ltxml:4074-4076; natbib.sty.ltxml:489-493 lacks it): a reference list written in an
/// `{enumerate}` (quant-ph9706037, 1708.06608, 2003.07967, hep-ph0212045). Opened so, by the `\bibitem` alone, it has
/// the heading the PDF prints (user ruling 2026-10-10): here none, so no title and no table-of-contents entry, and the
/// emptied `{enumerate}` goes. Repro index-bib/natbib_bibitem_in_enumerate.
#[test]
fn natbib_bibitem_outside_a_bibliography_opens_one() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_in_enumerate.tex"
  ));
  assert_eq!(xml.matches("<bibliography ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," show="Number" yyseparator=","/>]</cite>.</p></para><bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="refnum">[1]</tag><tag role="key">a</tag></tags><bibblock> Able, A title.
</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="refnum">[2]</tag><tag role="key">b</tag></tags><bibblock> Baker, B title.
</bibblock></bibitem></biblist></bibliography></document>"#,
  );
}

/// 64x (user ruling 2026-10-10): a bibliography a `\bibitem` opens outside any bibliography takes the author's
/// heading line just before it — a paragraph of one short styled run — as its title, which leaves the body, so the
/// heading is not printed twice and `\refname` not at all (2003.07967, 2306.17386, 0806.0831); "Figure Captions"
/// likewise (hep-ph9804413), however many bibliographies came before. Repros
/// index-bib/munged_bibliography_after_bold_heading, index-bib/bibitem_enumerate_not_a_bibliography.
#[test]
fn implicit_bibliography_takes_the_heading_line() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/munged_bibliography_after_bold_heading.tex"
  ));
  assert_eq!(xml.matches("<bibliography ").count(), 1, "{xml}");
  assert_eq!(xml.matches("Literature").count(), 1, "{xml}");
  assert!(!xml.contains("References"), "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>Literature</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="refnum">[1]</tag><tag role="key">a</tag></tags><bibblock> Able, A title.
</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="refnum">[2]</tag><tag role="key">b</tag></tags><bibblock> Baker, B title.
</bibblock></bibitem></biblist></bibliography>"#,
  );
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/bibitem_enumerate_not_a_bibliography.tex"
  ));
  assert_eq!(xml.matches("Figure Captions").count(), 1, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"biba\""],
    r#"<bibliography inlist="toc" xml:id="biba"><title>Figure Captions</title><biblist><bibitem key="1" xml:id="biba.bib1"><tags><tag role="number">1</tag><tag role="refnum">[1]</tag><tag role="key">1</tag></tags><bibblock>
Pion multiplicity distribution.
</bibblock></bibitem><bibitem key="2" xml:id="biba.bib2"><tags><tag role="number">2</tag><tag role="refnum">[2]</tag><tag role="key">2</tag></tags><bibblock>
Kaon multiplicity distribution.
</bibblock></bibitem></biblist></bibliography>"#,
  );
  // Small capitals and `\Large` are heading styles too.
  for (shape, expected) in [
    (
      r#"\documentclass{article}
\begin{document}

Text \cite{a}.

\noindent\textsc{Literature}
\begin{enumerate}
\bibitem{a} A. Able, Title.
\end{enumerate}

\end{document}
"#,
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p></para><bibliography inlist="toc" xml:id="bib"><title>Literature</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem></biblist></bibliography></document>"#,
    ),
    (
      r#"\documentclass{article}
\begin{document}

Text \cite{a}.

\noindent{\Large Literature}
\begin{enumerate}
\bibitem{a} A. Able, Title.
\end{enumerate}

\end{document}
"#,
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p></para><bibliography inlist="toc" xml:id="bib"><title>Literature</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem></biblist></bibliography></document>"#,
    ),
  ] {
    let xml = convert_clean(shape);
    assert_element(&xml, "document", &[], expected);
  }
}

/// 64x (user ruling 2026-10-10): what is no heading line stays in the body, and the bibliography has no title — a text
/// paragraph (without the emptied list it ended with; the kernel's `\bibitem`, no natbib), a bold sentence (a full stop
/// after more than one word), a bold line of five words, a `\normalsize` line in a `\small` body (not set apart), and an
/// unstyled `\noindent{REFERENCES}` (0801.2349: a plain line in the PDF too).
#[test]
fn implicit_bibliography_leaves_a_text_paragraph() {
  let xml = convert_clean(
    "\\documentclass{article}\n\\begin{document}\nSome ordinary text before the list of references here.\n\\begin{enumerate}\n\\bibitem{a} Able, A title.\n\\end{enumerate}\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Some ordinary text before the list of references here.</p></para><bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> Able, A title.
</bibblock></bibitem></biblist></bibliography></document>"#,
  );
  for (shape, expected) in [
    (
      r#"\documentclass{article}
\begin{document}

Text \cite{a}.

\noindent{\bf This work was supported by NSF.}
\begin{enumerate}
\bibitem{a} A. Able, Title.
\end{enumerate}

\end{document}
"#,
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p></para><para class="ltx_noindent" xml:id="p2"><p xml:id="p2.1"><text font="bold" xml:id="p2.1.1">This work was supported by NSF.</text></p></para><bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem></biblist></bibliography></document>"#,
    ),
    (
      r#"\documentclass{article}
\begin{document}

Text \cite{a}.

\noindent{\bf Five words in this line}
\begin{enumerate}
\bibitem{a} A. Able, Title.
\end{enumerate}

\end{document}
"#,
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p></para><para class="ltx_noindent" xml:id="p2"><p xml:id="p2.1"><text font="bold" xml:id="p2.1.1">Five words in this line</text></p></para><bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem></biblist></bibliography></document>"#,
    ),
    (
      r#"\documentclass{article}
\begin{document}
\small
Text \cite{a}.

\noindent{\normalsize Literature}
\begin{enumerate}
\bibitem{a} A. Able, Title.
\end{enumerate}

\end{document}
"#,
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1"><text fontsize="90%" xml:id="p1.1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</text></p></para><para class="ltx_noindent" xml:id="p2"><p xml:id="p2.1"><text xml:id="p2.1.1">Literature</text></p></para><bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib1.1"> A. Able, Title.
</text></bibblock></bibitem></biblist></bibliography></document>"#,
    ),
    (
      r#"\documentclass{article}
\begin{document}

Text \cite{a}.

\noindent{REFERENCES}
\begin{enumerate}
\bibitem{a} A. Able, Title.
\end{enumerate}

\end{document}
"#,
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p></para><para class="ltx_noindent" xml:id="p2"><p xml:id="p2.1">REFERENCES</p></para><bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem></biblist></bibliography></document>"#,
    ),
  ] {
    let xml = convert_clean(shape);
    assert_element(&xml, "document", &[], expected);
  }
}

/// 64x (user ruling 2026-10-10): an implicit bibliography closed by the next `\section` (no list, no `\end`) is headed
/// the same way, and the section follows it.
#[test]
fn implicit_bibliography_closed_by_a_section() {
  let xml = convert_clean(
    r#"\documentclass{article}
\begin{document}
Text \cite{a}.

\noindent\textbf{Literature}

\bibitem{a} A. Able, Title.

\bibitem{b} B. Baker, Title.

\section{Appendix}
More.
\end{document}
"#,
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite>.</p></para><bibliography inlist="toc" xml:id="bib"><title>Literature</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> B. Baker, Title.
</bibblock></bibitem></biblist></bibliography><section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Appendix</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">More.</p></para></section></document>"#,
  );
}

/// The log's `Error:`/`Warning:` lines: `count` of them, `messages` among them (each as often as listed).
fn assert_diagnostics(log: &str, count: usize, messages: &[&str]) {
  assert_eq!(error_count(log) + warning_count(log), count, "{log}");
  for message in messages {
    let wanted = messages.iter().filter(|m| *m == message).count();
    let found = log
      .lines()
      .filter(|l| (l.starts_with("Error:") || l.starts_with("Warning:")) && l.contains(message))
      .count();
    assert!(
      found >= wanted,
      "{wanted} diagnostics {message:?} expected, {found} found:\n{log}"
    );
  }
}

/// 64x: phantom entries (OXIDIZED_DESIGN #155) are pruned wherever they land, and only phantoms. Here in an appendix
/// paragraph after an aasms4 `\references` that never ends (the shape of 2301.05139, 2403.05651; review round 3): the
/// two Errors are the malformed entries' own (asserted by message, not with `assert_elements_with`, whose schema check
/// this malformed output cannot pass), and the unnumbered entry the appendix's second paragraph became (`bib.bibx6`,
/// no tag) is the known residual (the bibliography's `\par` redirection outliving it), not the target — it keeps its
/// text. A table and a
/// figure after the list (d1): 64w left 3 malformed phantoms, here none and no Error.
#[test]
fn phantom_entries_are_pruned_anywhere() {
  let (log, xml) = latexml::util::test::convert_with(
    r#"\documentstyle[aasms4]{article}
\begin{document}
Text.

\references

Able, A. 1990, ApJ, 1

Baker, B. 1991, ApJ, 2

\newpage
\appendix
\section{Details}
First appendix paragraph.

Second appendix paragraph.
\begin{figure}
\caption{A figure.}
\end{figure}
\end{document}
"#,
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "<ltx:bibitem> isn't allowed in <ltx:p>",
    "<ltx:bibitem> isn't allowed in <ltx:bibblock>",
  ]);
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><p xml:id="p1.1">Text.</p></para><bibliography citestyle="authoryear" inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.bibx1"><bibblock>Able, A. 1990, ApJ, 1</bibblock></bibitem><bibitem xml:id="bib.bibx2"><bibblock>Baker, B. 1991, ApJ, 2</bibblock></bibitem></biblist></bibliography><appendix inlist="toc" xml:id="A1"><tags><tag>Appendix A</tag><tag role="autoref">Appendix A<text xml:id="A1.1"/></tag><tag role="refnum">A</tag><tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix A</tag>Details</title><toctitle><tag close=" ">A</tag>Details</toctitle><para xml:id="A1.p1"><p xml:id="A1.p1.1">First appendix paragraph.<bibitem xml:id="bib.bibx6"><bibblock>Second appendix paragraph.</bibblock></bibitem></p></para><figure inlist="lof" xml:id="A1.F1"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1<text xml:id="A1.F1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>A figure.</toccaption><caption><tag close=": ">Figure 1</tag>A figure.</caption></figure></appendix></document>"#,
  );
  let xml = convert_clean(
    r#"\documentstyle[aasms4]{article}
\begin{document}
Text.

\references

Able, A. 1990, ApJ, 1

Baker, B. 1991, ApJ, 2

\begin{table}
\caption{A table.}
\begin{tabular}{l}
a\\
\end{tabular}
\end{table}

\begin{figure}
\caption{A figure.}
\end{figure}

\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography citestyle="authoryear" inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.bibx1"><bibblock>Able, A. 1990, ApJ, 1
</bibblock></bibitem><bibitem xml:id="bib.bibx2"><bibblock>Baker, B. 1991, ApJ, 2
</bibblock></bibitem></biblist><table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1<text xml:id="S0.T1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>A table.</toccaption><caption><tag close=": ">Table 1</tag>A table.</caption><tabular vattach="middle" xml:id="S0.T1.2"><tbody><tr xml:id="S0.T1.2.1"><td align="left" xml:id="S0.T1.2.1.1">a</td></tr></tbody></tabular></table><figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1<text xml:id="S0.F1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>A figure.</toccaption><caption><tag close=": ">Figure 1</tag>A figure.</caption></figure></bibliography>"#,
  );
  // An entry holding only a font run (a `{\small` group before the first `\bibitem`, 0801.2060) is a phantom.
  let xml = convert_clean(
    r#"\documentclass{article}
\begin{document}
Text \cite{a,b}.
\begin{thebibliography}{9}

{\small \renewcommand\baselinestretch{.6}


\bibitem{a} Able.

\bibitem{b} Baker.
}
\end{thebibliography}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib1.1"> Able.
</text></bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib2.1"> Baker.
</text><text xml:id="bib.bib2.2"></text></bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64x (review round 4): a keyless entry is no phantom when it holds content — a figure panel (an aas `\references`
/// entry is keyless by design: astro-ph0102416's Figure 2 kept 1 of 12 panels, astro-ph9902080 lost `heating.ps`), a
/// formula, a rule — or carries the author's label (`\bibitem[Baker(2001)]{}` prints "[Baker(2001)]", dropped on 64w
/// too). The a1 shape's panel sits in a malformed entry (its one Error); the panel survives.
#[test]
fn keyless_entries_with_content_are_kept() {
  let (log, xml) = latexml::util::test::convert_with(
    r#"\documentstyle[aasms4]{article}
\usepackage{graphicx}
\begin{document}
Text.

\references

Able, A. 1990, ApJ, 1

\begin{figure}
\begin{minipage}{5cm}\includegraphics{p1}\end{minipage}

\begin{minipage}{5cm}\includegraphics{p2}\end{minipage}
\caption{Panels.}
\end{figure}
\end{document}
"#,
    Some("ar5iv.sty"),
  );
  // The second diagnostic is latex209.def's trap on the 2.09 document's `\usepackage` (pdflatex the same; user ruling
  // 2026-10-17, OXIDIZED_DESIGN_DIVERGENCES #725); graphicx still loads.
  assert_diagnostics(&log, 2, &[
    "<ltx:bibitem> isn't allowed in <ltx:figure>",
    "LaTeX2e command \\usepackage in LaTeX 2.09 document",
  ]);
  assert_eq!(xml.matches("<graphics ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "figure",
    &[],
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1<text xml:id="S0.F1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><block class="ltx_figure_panel ltx_minipage" vattach="middle" width="142.3pt" xml:id="S0.F1.2"><graphics graphic="p1" xml:id="S0.F1.g1"/></block><bibitem class="ltx_figure_panel" xml:id="bib.bibx4"><bibblock><inline-block class="ltx_minipage" vattach="middle" width="142.3pt" xml:id="bib.bibx4.1"><graphics graphic="p2" xml:id="bib.bibx4.g1"/></inline-block></bibblock></bibitem><toccaption><tag close=" ">1</tag>Panels.</toccaption><caption><tag close=": ">Figure 1</tag>Panels.</caption></figure>"#,
  );
  let xml = convert_clean(
    r#"\documentclass{article}\usepackage{graphicx}
\begin{document}
Text \cite{a,b,c,d}.
\begin{thebibliography}{9}
\bibitem{a} Able, A title.
\bibitem{} \includegraphics[width=2cm]{logo}
\bibitem{} $x^2$
\bibitem[]{c} 
\bibitem{d}
\bibitem{}\rule{1cm}{1pt}
\end{thebibliography}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> Able, A title.
</bibblock></bibitem><bibitem xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> <graphics graphic="logo" options="width=56.9055pt,keepaspectratio=true" xml:id="bib.bib2.g1"/></bibblock></bibitem><bibitem xml:id="bib.bib3"><tags><tag>[3]</tag><tag role="refnum">3</tag></tags><bibblock> <Math mode="inline" tex="x^{2}" text="x ^ 2" xml:id="bib.bib3.m1"><XMath xml:id="bib.bib3.m1.1"><XMApp xml:id="bib.bib3.m1.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></bibblock></bibitem><bibitem key="c" xml:id="bib.bibx1"><tags><tag>[]</tag></tags><bibblock></bibblock></bibitem><bibitem key="d" xml:id="bib.bib4"><tags><tag>[4]</tag><tag role="refnum">4</tag></tags><bibblock></bibblock></bibitem><bibitem xml:id="bib.bib5"><tags><tag>[5]</tag><tag role="refnum">5</tag></tags><bibblock><rule height="1.0pt" width="28.5pt"/></bibblock></bibitem></biblist></bibliography>"#,
  );
  let xml = convert_clean(
    r#"\documentclass{article}
\begin{document}
Text \cite{a}.
\begin{thebibliography}{9}
\bibitem[Able(2000)]{} Able, A.

\bibitem[Baker(2001)]{}

\bibitem{a} Cole.
\end{thebibliography}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.bibx1"><tags><tag>[Able(2000)]</tag><tag role="refnum">Able(2000)</tag></tags><bibblock> Able, A.
</bibblock></bibitem><bibitem xml:id="bib.bibx2"><tags><tag>[Baker(2001)]</tag><tag role="refnum">Baker(2001)</tag></tags><bibblock></bibblock></bibitem><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> Cole.
</bibblock></bibitem></biblist></bibliography>"#,
  );
  // natbib: the author's label keeps an empty entry too (`\@lbibitem`'s `#1`).
  let xml = convert_clean(
    r#"\documentclass{article}
\usepackage{natbib}
\begin{document}
Text \citet{a}.
\begin{thebibliography}{9}
\bibitem[Able(2000)]{} Able, A.

\bibitem[Baker(2001)]{}

\bibitem[Cole(2002)]{a} Cole.
\end{thebibliography}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2000</tag><tag role="authors">Able</tag><tag role="refnum">Able (2000)</tag></tags><bibblock> Able, A.
</bibblock></bibitem><bibitem xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="year">2001</tag><tag role="authors">Baker</tag><tag role="refnum">Baker (2001)</tag></tags><bibblock></bibblock></bibitem><bibitem key="a" xml:id="bib.bib3"><tags><tag role="number">3</tag><tag role="year">2002</tag><tag role="authors">Cole</tag><tag role="refnum">Cole (2002)</tag><tag role="key">a</tag></tags><bibblock> Cole.
</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64x (review rounds 4-5): only a TeX `\item` or `\bibitem` steps the bibliography's counter. `\item`s separated by
/// blank lines number 1, 2, 3, as pdflatex prints them: a blank line before an `\item` makes no entry, and the entry the
/// recovery opens for a paragraph no `\item` starts (`\lx@pseudo@bibitem`) takes no number.
#[test]
fn blank_line_items_number_consecutively() {
  let xml = convert_clean(
    r#"\documentclass{article}
\begin{document}
Text.
\begin{thebibliography}{9}
\item A. Able, Title.

\item B. Baker, Title.

\item C. Cole, Title.
\end{thebibliography}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock>A. Able, Title.
</bibblock></bibitem><bibitem xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock>B. Baker, Title.
</bibblock></bibitem><bibitem xml:id="bib.bib3"><tags><tag>[3]</tag><tag role="refnum">3</tag></tags><bibblock>C. Cole, Title.
</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64x (review round 7): a natbib `\bibitem` directly in the document body, in no list, is LaTeX's "Lonely \item"
/// (pdflatex 2, Perl 2 `malformed:ltx:bibitem`): reported in that category, once per `\bibitem`, and the entries still
/// recovered into a bibliography (1108.2593). Repro index-bib/natbib_bibitem_in_document_body.
#[test]
fn natbib_bibitem_in_the_document_body_is_reported() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_in_document_body.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "\\bibitem outside any list",
    "\\bibitem outside any list",
  ]);
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2011</tag><tag role="authors">Able</tag><tag role="refnum">Able (2011)</tag><tag role="key">a</tag></tags><bibblock> Able, A. 2011, ApJ.
</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="year">2012</tag><tag role="authors">Baker</tag><tag role="refnum">Baker (2012)</tag><tag role="key">b</tag></tags><bibblock> Baker, B. 2012, ApJ.
</bibblock></bibitem></biblist></bibliography>"#,
  );
  // In `{small}` (an environment that is no list) just as lonely; in `{flushleft}`, a `\trivlist` in latex.ltx, not
  // (pdflatex 2 and 0).
  let (log, xml) = latexml::util::test::convert_with(
    r#"\documentclass{article}
\usepackage{natbib}
\begin{document}
\section{Summary}
Text \citep{a}.
\section*{References}
\begin{small}
\bibitem[Able(2011)]{a} Able, A. 2011, ApJ.

\bibitem[Baker(2012)]{b} Baker, B. 2012, ApJ.
\end{small}
\end{document}
"#,
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "\\bibitem outside any list",
    "\\bibitem outside any list",
  ]);
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number"><text fontsize="90%" xml:id="bib.bib1.1">1</text></tag><tag role="year"><text fontsize="90%" xml:id="bib.bib1.2">2011</text></tag><tag role="authors"><text fontsize="90%" xml:id="bib.bib1.3">Able</text></tag><tag role="refnum"><text fontsize="90%" xml:id="bib.bib1.4">Able (2011)</text></tag><tag role="key"><text fontsize="90%" xml:id="bib.bib1.5">a</text></tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib1.6"> Able, A. 2011, ApJ.</text></bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag role="number"><text fontsize="90%" xml:id="bib.bib2.1">2</text></tag><tag role="year"><text fontsize="90%" xml:id="bib.bib2.2">2012</text></tag><tag role="authors"><text fontsize="90%" xml:id="bib.bib2.3">Baker</text></tag><tag role="refnum"><text fontsize="90%" xml:id="bib.bib2.4">Baker (2012)</text></tag><tag role="key"><text fontsize="90%" xml:id="bib.bib2.5">b</text></tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib2.6"> Baker, B. 2012, ApJ.</text></bibblock></bibitem></biblist></bibliography>"#,
  );
  let xml = convert_clean(
    r#"\documentclass{article}
\usepackage{natbib}
\begin{document}
\section{Summary}
Text \citep{a}.
\begin{flushleft}
\bibitem[Able(2011)]{a} Able, A. 2011, ApJ.

\bibitem[Baker(2012)]{b} Baker, B. 2012, ApJ.
\end{flushleft}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2011</tag><tag role="authors">Able</tag><tag role="refnum">Able (2011)</tag><tag role="key">a</tag></tags><bibblock> Able, A. 2011, ApJ.
</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="year">2012</tag><tag role="authors">Baker</tag><tag role="refnum">Baker (2012)</tag><tag role="key">b</tag></tags><bibblock> Baker, B. 2012, ApJ.
</bibblock></bibitem></biblist></bibliography>"#,
  );
  // A `\section` closes the bibliography the first one opened: the next is lonely again and opens its own (pdflatex 2).
  let (log, xml) = latexml::util::test::convert_with(
    r#"\documentclass{article}
\usepackage{natbib}
\begin{document}
\section{Summary}
Text \citep{a,c}.

\bibitem[Able(2011)]{a} Able, A. 2011, ApJ.

\section{Appendix}
More text.

\bibitem[Cole(2013)]{c} Cole, C. 2013, ApJ.
\end{document}
"#,
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "\\bibitem outside any list",
    "\\bibitem outside any list",
  ]);
  assert_eq!(xml.matches("<bibliography ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    r#"<bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2011</tag><tag role="authors">Able</tag><tag role="refnum">Able (2011)</tag><tag role="key">a</tag></tags><bibblock> Able, A. 2011, ApJ.</bibblock></bibitem></biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"biba\""],
    r#"<bibliography xml:id="biba"><biblist><bibitem key="c" xml:id="biba.bib1"><tags><tag role="number">1</tag><tag role="year">2013</tag><tag role="authors">Cole</tag><tag role="refnum">Cole (2013)</tag><tag role="key">c</tag></tags><bibblock> Cole, C. 2013, ApJ.</bibblock></bibitem></biblist></bibliography>"#,
  );
  // The heading that closes it also drops natbib's pending entry stop (`\BibitemShut{NoStop}`'s `\@bibstop`), which the
  // bibliography's postamble, never run, would have spent: else the next `\bibitem` (`\bibitem@fin`) put a stray "." at
  // the head of the second bibliography's first entry.
  let (log, xml) = latexml::util::test::convert_with(
    r#"\documentclass{article}
\usepackage{natbib}
\makeatletter
\newcommand\pendingstop{\def\@bibstop{NoStop}}
\newcommand\showstop{\ifx\@bibstop\@undefined Cleared.\else Pending.\fi}
\makeatother
\begin{document}
\section{Summary}
Text \citep{a,c}.

\bibitem[Able(2011)]{a} Able, A. 2011, ApJ\pendingstop

\section{Appendix}
\showstop

\bibitem[Cole(2013)]{c} Cole, C. 2013, ApJ.
\end{document}
"#,
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "\\bibitem outside any list",
    "\\bibitem outside any list",
  ]);
  assert!(
    xml.contains("Cleared.") && !xml.contains("Pending."),
    "{xml}"
  );
  // The same in a `{small}` the `\section*` falls in (pdflatex 2, Perl 2): its end, which the first bibliography made
  // its own, is not made again by the second, which would call itself (repro
  // index-bib/natbib_bibitem_in_size_environment_twice).
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_in_size_environment_twice.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "\\bibitem outside any list",
    "\\bibitem outside any list",
  ]);
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    r#"<bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number"><text fontsize="90%" xml:id="bib.bib1.1">1</text></tag><tag role="year"><text fontsize="90%" xml:id="bib.bib1.2">2011</text></tag><tag role="authors"><text fontsize="90%" xml:id="bib.bib1.3">Able</text></tag><tag role="refnum"><text fontsize="90%" xml:id="bib.bib1.4">Able (2011)</text></tag><tag role="key"><text fontsize="90%" xml:id="bib.bib1.5">a</text></tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib1.6"> Able, A. 2011, ApJ.</text></bibblock></bibitem></biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"biba\""],
    r#"<bibliography xml:id="biba"><biblist><bibitem key="c" xml:id="biba.bib1"><tags><tag role="number"><text fontsize="90%" xml:id="biba.bib1.1">1</text></tag><tag role="year"><text fontsize="90%" xml:id="biba.bib1.2">2013</text></tag><tag role="authors"><text fontsize="90%" xml:id="biba.bib1.3">Cole</text></tag><tag role="refnum"><text fontsize="90%" xml:id="biba.bib1.4">Cole (2013)</text></tag><tag role="key"><text fontsize="90%" xml:id="biba.bib1.5">c</text></tag></tags><bibblock><text fontsize="90%" xml:id="biba.bib1.6"> Cole, C. 2013, ApJ.</text></bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64x (review round 9): the bibliography a `\bibitem` opens outside any bibliography, having no group of its own, gives
/// `\bibliography` and `\bibliographystyle` back after its setup made them no-ops (Perl
/// latex_constructs.pool.ltxml:3839-3840): the author's real bibliography read later from the `.bbl` is printed, as
/// pdflatex prints it. natbib's (Perl keeps it, never opening one), the kernel's (Perl drops it: beyond Perl), and
/// AASTeX's bare `\references` (Perl runs no such setup). Repros index-bib/natbib_bibitem_then_real_bibliography,
/// index-bib/kernel_bibitem_then_real_bibliography, index-bib/aas_references_then_real_bibliography.
#[test]
fn bibliography_after_an_implicit_one_is_printed() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_then_real_bibliography.tex"
    ),
    &[(
      "t.bbl",
      include_str!(
        "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_then_real_bibliography.bbl"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 1, &["\\bibitem outside any list"]);
  assert_eq!(xml.matches("<bibliography ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    r#"<bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2011</tag><tag role="authors">Able</tag><tag role="refnum">Able (2011)</tag><tag role="key">a</tag></tags><bibblock> Able, A. 2011, ApJ.</bibblock></bibitem></biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"biba\""],
    r#"<bibliography inlist="toc" xml:id="biba"><title>References</title><biblist><bibitem key="z" xml:id="biba.bib1"><tags><tag role="number">1</tag><tag role="year">2020</tag><tag role="authors">Zed</tag><tag role="refnum">Zed [2020]</tag><tag role="key">z</tag></tags><bibblock> Zed, Z. 2020, Real Bibliography Entry.</bibblock></bibitem></biblist></bibliography>"#,
  );
  // the kernel's `\bibitem` (its Error is the known one of KPE #595's path, as base and Perl)
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/kernel_bibitem_then_real_bibliography.tex"
    ),
    &[(
      "t.bbl",
      include_str!(
        "../../../tools/perfect_kernel/repros/index-bib/kernel_bibitem_then_real_bibliography.bbl"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 1, &["<ltx:bibitem> isn't allowed in <ltx:p>"]);
  assert_eq!(xml.matches("<bibliography ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    r#"<bibliography bibstyle="plain" citestyle="numbers" sort="true" xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> Able, A. 2011, ApJ.</bibblock></bibitem></biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"biba\""],
    r#"<bibliography inlist="toc" xml:id="biba"><title>References</title><biblist><bibitem key="z" xml:id="biba.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> Zed, Z. 2020, Real Bibliography Entry.</bibblock></bibitem></biblist></bibliography>"#,
  );
  let xml = convert_clean_files(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/aas_references_then_real_bibliography.tex"
    ),
    &[(
      "t.bbl",
      include_str!(
        "../../../tools/perfect_kernel/repros/index-bib/aas_references_then_real_bibliography.bbl"
      ),
    )],
  );
  assert_eq!(xml.matches("<bibliography ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"bib\""],
    r#"<bibliography citestyle="numbers" inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="refnum">[1]</tag><tag role="key">a</tag></tags><bibblock> Able, A. 2011, ApJ.</bibblock></bibitem></biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &["xml:id=\"biba\""],
    r#"<bibliography inlist="toc" xml:id="biba"><title>References</title><biblist><bibitem key="z" xml:id="biba.bib1"><tags><tag role="number">1</tag><tag role="year">2020</tag><tag role="authors">Zed</tag><tag role="refnum">[1]</tag><tag role="key">z</tag></tags><bibblock> Zed, Z. 2020, Real Bibliography Entry.</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64x (review round 10): the lonely-bibliography flag a paper ends with does not reach the next paper a long-lived
/// worker converts on the same thread (cortex_worker: a `Converter` per paper, no engine reset between them): each
/// `Converter` starts from a fresh `State` (`Core::new`), so the second document's `\bibitem` in a bibliography is not
/// taken for a lonely one.
#[test]
fn lonely_bibliography_does_not_reach_the_next_document() {
  use std::rc::Rc;

  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};
  let first = "\\documentclass{article}\n\\usepackage{natbib}\n\\begin{document}\nText \\citep{a}.\n\n\\bibitem[Able(2011)]{a} Able, A. 2011, ApJ.\n\\end{document}\n";
  // in a bibliography (the flag set by hand, as a binding without the kernel's setup leaves it) and in no list: an
  // Error only if the first document's lonely flag were still set
  let second = "\\documentclass{article}\n\\usepackage{natbib}\n\\begin{document}\nText \\citep{b}.\n\\makeatletter\\@lx@inbibliographytrue\\makeatother\n\n\\bibitem[Baker(2012)]{b} Baker, B. 2012, ApJ.\n\\end{document}\n";
  let logs = std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let logs: Vec<String> = [first, second]
        .into_iter()
        .map(|tex| {
          let mut converter = Converter::from_config(opts.clone());
          converter.prepare_session(&opts).expect("a session");
          converter
            .convert_content_with_provenance("t.tex", tex.to_string())
            .log
        })
        .collect();
      latexml_core::reset_thread_engine();
      logs
    })
    .expect("a conversion thread")
    .join()
    .expect("the conversions");
  assert_eq!(
    logs[0].matches("\\bibitem outside any list").count(),
    1,
    "{}",
    logs[0]
  );
  assert_eq!(
    logs[1].matches("\\bibitem outside any list").count(),
    0,
    "{}",
    logs[1]
  );
}

/// 64x (review round 7): in the author's own `{references}` that its class makes no list (CS18.sty:176-179), after a
/// run-in `\acknowledgments` whose list frame a sectioning command has left behind, natbib's `\bibitem`s open the
/// bibliography (pdflatex 0 with that class, whose `\acknowledgments` leaves a trivlist open; 1411.7266 went 0 → 10 under a
/// stale list-item gate). Repro index-bib/natbib_bibitem_in_class_references, its .sty a reduced model of CS18.
#[test]
fn natbib_bibitem_in_a_class_references_environment() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_in_class_references.tex"
    ),
    &[(
      "natbib_bibitem_in_class_references.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_in_class_references.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title fontsize="90%">References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag role="number"><text fontsize="90%" xml:id="bib.bib1.1">1</text></tag><tag role="year"><text fontsize="90%" xml:id="bib.bib1.2">2011</text></tag><tag role="authors"><text fontsize="90%" xml:id="bib.bib1.3">Able</text></tag><tag role="refnum"><text fontsize="90%" xml:id="bib.bib1.4">Able (2011)</text></tag><tag role="key"><text fontsize="90%" xml:id="bib.bib1.5">a</text></tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib1.6"> Able, A. 2011, ApJ.
</text></bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag role="number"><text fontsize="90%" xml:id="bib.bib2.1">2</text></tag><tag role="year"><text fontsize="90%" xml:id="bib.bib2.2">2012</text></tag><tag role="authors"><text fontsize="90%" xml:id="bib.bib2.3">Baker</text></tag><tag role="refnum"><text fontsize="90%" xml:id="bib.bib2.4">Baker (2012)</text></tag><tag role="key"><text fontsize="90%" xml:id="bib.bib2.5">b</text></tag></tags><bibblock><text fontsize="90%" xml:id="bib.bib2.6"> Baker, B. 2012, ApJ.
</text></bibblock></bibitem></biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "itemize",
    &["class=\"ltx_trivlist\""],
    r#"<itemize class="ltx_trivlist" xml:id="S1.I1"><item xml:id="S1.I1.ix1"><tags><tag><text font="italic" fontsize="90%" xml:id="S1.I1.ix1.1">Acknowledgements.</text></tag></tags><para xml:id="S1.I1.ix1.p1"><p xml:id="S1.I1.ix1.p1.1"><text fontsize="90%" xml:id="S1.I1.ix1.p1.1.1">Thanks.</text></p></para></item></itemize>"#,
  );
}

/// 64x (review round 7): a natbib `\bibitem` after an `\item` of the author's list stays in the list, one item among its
/// items, as Perl and base keep it — the one `malformed` Error kept until the environment model (SYNC K21) lets a
/// bibliography leave the list element while its TeX group stays open (cond-mat0003103). RED repro
/// index-bib/natbib_bibitem_among_list_items.
#[test]
fn natbib_bibitem_among_list_items_keeps_its_error() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_among_list_items.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 1, &["<ltx:bibitem> isn't allowed in <ltx:item>"]);
  assert_element(
    &xml,
    "enumerate",
    &[],
    r#"<enumerate xml:id="Sx1.I1"><item xml:id="Sx1.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="Sx1.I1.i1.p1"><p xml:id="Sx1.I1.i1.p1.1">A. Able, Title.</p></para><bibitem key="v" xml:id="X-at-lx-at-bibliography0.bib1"><tags><tag role="number">1</tag><tag role="refnum">[1]</tag><tag role="key">v</tag></tags><bibblock> V. Vesna, Title.
</bibblock></bibitem></item><item xml:id="Sx1.I1.i2"><tags><tag>2.</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para xml:id="Sx1.I1.i2.p1"><p xml:id="Sx1.I1.i2.p1.1">B. Baker, Title.</p></para></item><item xml:id="Sx1.I1.i3"><tags><tag>3.</tag><tag role="refnum">3</tag><tag role="typerefnum">item 3</tag></tags><para xml:id="Sx1.I1.i3.p1"><p xml:id="Sx1.I1.i3.p1.1">C. Cole, Title.</p></para></item></enumerate>"#,
  );
  // An author's own `\list` environment the same way: its `\item`s stay items, the `\bibitem`s placed among them as base
  // and Perl place them (two placement Errors; pdflatex prints them as items).
  let (log, xml) = latexml::util::test::convert_with(
    r#"\documentclass{article}
\usepackage{natbib}
\newenvironment{refs}{\list{}{}}{\endlist}
\begin{document}
\section{Summary}
Text \citep{a,b}.
\begin{refs}
\item First plain item.
\bibitem[Able(2011)]{a} Able, A. 2011, ApJ.
\item Second plain item.
\bibitem[Baker(2012)]{b} Baker, B. 2012, ApJ.
\end{refs}
After the list.
\end{document}
"#,
    Some("ar5iv.sty"),
  );
  assert_diagnostics(&log, 2, &[
    "<ltx:bibitem> isn't allowed in <ltx:p>",
    "<ltx:bibitem> isn't allowed in <ltx:p>",
  ]);
  // (the list keeps base's shape, its id from the paragraph it interrupts included)
  assert_element(
    &xml,
    "itemize",
    &[],
    r#"<itemize xml:id="S1.p1.2"><item xml:id="S1.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags><para xml:id="S1.I1.i1.p1"><p xml:id="S1.I1.i1.p1.1">First plain item.<bibitem key="a" xml:id="X-at-lx-at-bibliography0.bib1"><tags><tag role="number">1</tag><tag role="year">2011</tag><tag role="authors">Able</tag><tag role="refnum">Able (2011)</tag><tag role="key">a</tag></tags><bibblock> Able, A. 2011, ApJ.</bibblock></bibitem></p></para></item><item xml:id="S1.I1.i2"><tags><tag role="typerefnum">2nd item</tag></tags><para xml:id="S1.I1.i2.p1"><p xml:id="S1.I1.i2.p1.1">Second plain item.<bibitem key="b" xml:id="X-at-lx-at-bibliography0.bib2"><tags><tag role="number">2</tag><tag role="year">2012</tag><tag role="authors">Baker</tag><tag role="refnum">Baker (2012)</tag><tag role="key">b</tag></tags><bibblock> Baker, B. 2012, ApJ.</bibblock></bibitem></p></para></item></itemize>"#,
  );
}

/// 64x (review round 6): `{thebibliography}`'s own `\item` starts an entry (and a blank line before it makes none); an
/// `\item` of a list inside an entry is that list's (pdflatex: [1] with its two bullets, [2], [3]).
#[test]
fn thebibliography_own_item_starts_an_entry() {
  let xml = convert_clean(
    r#"\documentclass{article}
\begin{document}
Text \cite{a}.
\begin{thebibliography}{9}
\bibitem{a} A. Able, Title, with notes:
\begin{itemize}
\item one

\item two
\end{itemize}

\item B. Baker, Title.

\item C. Cole, Title.
\end{thebibliography}
\end{document}
"#,
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title, with notes:
<itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p xml:id="S0.I1.i1.p1.1">one</p></para></item><item xml:id="S0.I1.i2"><tags><tag>•</tag><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I1.i2.p1"><p xml:id="S0.I1.i2.p1.1">two</p></para></item></itemize></bibblock></bibitem><bibitem xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock>B. Baker, Title.
</bibblock></bibitem><bibitem xml:id="bib.bib3"><tags><tag>[3]</tag><tag role="refnum">3</tag></tags><bibblock>C. Cole, Title.
</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64x (user ruling 2026-10-10): an untitled implicit bibliography is not in the table of contents, which listed it by
/// its id ("bib") for want of a title (`Info:expected:ids Missing Usable title`); pdflatex lists nothing for it. Repro
/// index-bib/untitled_implicit_bibliography_toc.
#[test]
fn untitled_implicit_bibliography_has_no_toc_entry() {
  let source = "../tools/perfect_kernel/repros/index-bib/untitled_implicit_bibliography_toc.tex";
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/untitled_implicit_bibliography_toc.tex"
  ));
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography xml:id="bib"><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Able, Title.
</bibblock></bibitem></biblist></bibliography>"#,
  );
  let posted = crate::cluster::convert_and_post_clean(source);
  assert_element(
    &posted,
    "toclist",
    &[],
    r#"<toclist><tocentry class="ltx_tocentry_section"><ref idref="S1" show="toctitle"><text class="ltx_ref_title"><tag close=" ">1</tag>Intro</text></ref></tocentry></toclist>"#,
  );
}

/// 64x: `ltx:biblist` auto-opens at half openability (Perl's fractional `autoOpen`, as `ltx:picture`'s), so a
/// `\bibitem` after a float that closed the list opens a new one (astro-ph9806175, 2103.15495, 0801.2060), while text
/// in the bibliography still takes the full auto-opener `ltx:para`. Repro index-bib/float_between_bibitems.
#[test]
fn bibitem_after_a_float_reopens_the_list() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/float_between_bibitems.tex"
  ));
  assert_element(
    &xml,
    "bibliography",
    &[],
    "<bibliography inlist=\"toc\" xml:id=\"bib\"><title>References</title><biblist><bibitem key=\"a\" xml:id=\"bib.bib1\"><tags><tag>[1]</tag><tag role=\"refnum\">1</tag></tags><bibblock> First reference.\n</bibblock></bibitem></biblist><figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>A figure.</toccaption><caption><tag close=\": \">Figure 1</tag>A figure.</caption></figure><biblist><bibitem key=\"b\" xml:id=\"bib.bib2\"><tags><tag>[2]</tag><tag role=\"refnum\">2</tag></tags><bibblock> Second reference.\n</bibblock></bibitem></biblist></bibliography>",
  );
  // An aasms4 `\references` whose lines end in `\\`, then a table and a figure: each `\\` and blank line after a
  // float makes a phantom keyless entry, pruned, and the list auto-opened for it goes with it (astro-ph0307431,
  // 0803.0740: 21 and 14 empty lists).
  // The aas `\references` binding titles it and sets its citestyle as Perl (aas_support.sty.ltxml:289-293; 64zd); the
  // classes print `\subsection*{REFERENCES}` (aasms4.sty:216, aaspp4.sty:207).
  let xml = convert_clean(
    "\\documentstyle[aasms4]{article}\n\\begin{document}\nText.\n\n\\references\n\n1. Costa, E. A title. (1997).\\\\\n2. Van Paradijs, J. Another.\\\\\n\n\\newpage\n\\rm\n\\begin{table}\n\\caption{A table.}\n\\end{table}\n\n\\begin{figure}\n\\caption{A figure.}\n\\end{figure}\n\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography citestyle="authoryear" inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.bibx1"><bibblock>1. Costa, E. A title. (1997).</bibblock></bibitem><bibitem xml:id="bib.bibx2"><bibblock>2. Van Paradijs, J. Another.</bibblock></bibitem></biblist><table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1<text xml:id="S0.T1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>A table.</toccaption><caption><tag close=": ">Table 1</tag>A table.</caption></table><figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1<text xml:id="S0.F1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>A figure.</toccaption><caption><tag close=": ">Figure 1</tag>A figure.</caption></figure></bibliography>"#,
  );
  // Text in a bibliography outside its list stays a paragraph (`ltx:para` outranks the half-open `ltx:biblist`).
  let xml = convert_clean(
    "\\documentclass{article}\n\\begin{document}\n\\begin{thebibliography}{9}\n\\bibitem{a} First.\n\\begin{figure}\\caption{F.}\\end{figure}\nNote.\n\\end{thebibliography}\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    "<bibliography inlist=\"toc\" xml:id=\"bib\"><title>References</title><biblist><bibitem key=\"a\" xml:id=\"bib.bib1\"><tags><tag>[1]</tag><tag role=\"refnum\">1</tag></tags><bibblock> First.\n</bibblock></bibitem></biblist><figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>F.</toccaption><caption><tag close=\": \">Figure 1</tag>F.</caption></figure><para xml:id=\"bib.p1\"><p xml:id=\"bib.p1.1\">Note.</p></para></bibliography>",
  );
}

/// 64x: AMS-TeX ends a document with plain's `\end` (Perl `\lx@end@document`, plain_constructs.pool.ltxml:284); the
/// amsppt document read LaTeX took it for `\end{<env>}` (1505.02369, 1901.01055, 2410.05768, math-ph9812020,
/// q-alg9510020). Repros amstex/amsppt_end, amstex/amsppt_bare_end.
#[test]
fn amstex_end_ends_the_document() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_end.tex"
  ));
  assert_element(
    &xml,
    "document",
    &[],
    "<document xmlns=\"http://dlmf.nist.gov/LaTeXML\"><title>A title</title><para xml:id=\"p1\"><p xml:id=\"p1.1\">Some text.\n</p></para></document>",
  );
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_bare_end.tex"
  ));
  assert_element(
    &xml,
    "document",
    &[],
    "<document xmlns=\"http://dlmf.nist.gov/LaTeXML\"><para xml:id=\"p1\"><p xml:id=\"p1.1\">Some text.\n</p></para></document>",
  );
}

/// 64x: amstex.tex:136 `\def\predefine#1#2{\let#1#2}` unwraps a braced argument, `\predefine\til{\~}` (math0012149,
/// math0012150, math0012155). Repro amstex/amstex_predefine_braced.
#[test]
fn amstex_predefine_takes_braced_arguments() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_predefine_braced.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">A word with ñ and <Math mode="inline" tex="\widetilde{x}" text="widetilde@(x)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="widetilde" role="OVERACCENT">~</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>.</p></para>"#,
  );
}

/// 64x: `\endabstract` is locked against class and package files with the kernel's `\abstract` (Perl
/// Package.pm:1993-1997 locks an environment's pair), so a package's `\def\endabstract{\par\endgroup…}`
/// (IMjournal.sty:81/83; 1702.00108, 2304.05575) does not end the kernel's abstract by closing a group it never opened;
/// a document's own `\endabstract`, paired with its own `\beginabstract` (gr-qc9406008), stays its own. Repro
/// sectioning-frontmatter/endabstract_locked_with_abstract.
#[test]
fn endabstract_is_locked_with_abstract() {
  let xml = convert_clean_files(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/endabstract_locked_with_abstract.tex"
    ),
    &[(
      "endabstract_locked_with_abstract.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/endabstract_locked_with_abstract.sty"
      ),
    )],
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">An abstract written with the package’s begin and end commands.</p></abstract><para xml:id="p1"><p xml:id="p1.1">Body.</p></para></document>"#,
  );
  // The document's own pair: `\endabstract` ends its `\beginabstract`'s list, not the kernel's abstract.
  let xml = convert_clean(
    "\\documentclass{article}\n\\def\\beginabstract{\\begin{list}{}{}\\item\\textbf{Abstract.} }\n\\def\\endabstract{\\end{list}}\n\\begin{document}\n\\beginabstract\nText.\n\\endabstract\nBody.\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para xml:id="p1"><itemize xml:id="p1.1"><item xml:id="S0.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p xml:id="S0.I1.i1.p1.1"><text font="bold" xml:id="S0.I1.i1.p1.1.1">Abstract.</text> Text.</p></para></item></itemize><p xml:id="p1.2">Body.</p></para></document>"#,
  );
}

/// 64x: jheppub.sty:66's `\abstract` takes one argument; its binding records that, so the kernel's locked `\abstract`
/// reads the braced abstract as an argument and a `\paragraph` inside it stays in the abstract (1903.02433).
/// Repro sectioning-frontmatter/jheppub_abstract_paragraph.
#[test]
fn jheppub_abstract_is_one_argument() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jheppub_abstract_paragraph.tex"
  ));
  assert_eq!(xml.matches("<abstract ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">First paragraph of the abstract.</p><paragraph inlist="toc" xml:id="S0.SS0.SSS0.Px1"><title/><para xml:id="S0.SS0.SSS0.Px1.p1"><p xml:id="S0.SS0.SSS0.Px1.p1.1">Second paragraph after an empty run-in heading.</p></para></paragraph></abstract>"#,
  );
}

/// 64x: PoS loads JHEP's binding without JHEP's journal-citation macros, which PoS.cls does not define: a paper's own
/// `\newcommand{\apj}{ApJ}` stands (1507.03892, 1510.07042, 1807.03003, 1908.00100); JHEP itself keeps them, and
/// PoS has its own `\pos`. Repro loader/pos_without_jhep_journal_macros.
#[test]
fn pos_has_no_jhep_journal_macros() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/loader/pos_without_jhep_journal_macros.tex"
  ));
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para class="ltx_align_left" xml:id="p1"><p xml:id="p1.1"><ref href="https://doi.org/10.1086/163513"><emph font="italic" xml:id="p1.1.1">ApJ</emph> <text font="bold" xml:id="p1.1.2">297</text> (1985) 166</ref>.</p></para>"#,
  );
  // 2602.22473's shape (a PoS-hep paper): its own `\newcommand{\am}` stands, where JHEP's `\am{}{}{}` swallowed
  // `\mgr`, `M` and `^` into "Ann. Math." inside the formula.
  let xml = convert_clean(
    "\\documentclass{PoS}\n\\newcommand{\\ld}{\\ensuremath{\\lambda}}\n\\newcommand{\\mgr}{\\ensuremath{m_{3/2}}}\n\\newcommand{\\am}{\\ensuremath{{\\rm a}_{3/2}}}\n\\begin{document}\n$ \\ld\\am\\mgr\nM^2S, $\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math mode="inline" tex="\lambda{\rm a}_{3/2}m_{3/2}M^{2}S," text="lambda * a _ (3 / 2) * m _ (3 / 2) * M ^ 2 * S" xml:id="p1.m1"><XMath xml:id="p1.m1.2"><XMDual xml:id="p1.m1.2.1"><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.1"/><XMWrap xml:id="p1.m1.2.1.2"><XMApp xml:id="p1.m1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="lambda" role="UNKNOWN">λ</XMTok><XMApp xml:id="p1.m1.1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">a</XMTok><XMApp xml:id="p1.m1.1.3.3"><XMTok fontsize="70%" meaning="divide" role="MULOP">/</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp><XMApp xml:id="p1.m1.1.4"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="p1.m1.1.4.3"><XMTok fontsize="70%" meaning="divide" role="MULOP">/</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp><XMApp xml:id="p1.m1.1.5"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">M</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">S</XMTok></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMath></Math>"#,
  );
  // PoS.cls v1.2.1:1384's own `\pos{<id>}` (1411.6666, 0906.1503).
  let xml = convert_clean(
    "\\documentclass{PoS}\n\\title{T}\n\\begin{document}\nSee \\pos{PoS(INTEGRAL08)029}.\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">See <ref font="typewriter" href="http://pos.sissa.it/cgi-bin/reader/contribution.cgi?id=PoS(INTEGRAL08)029">PoS(INTEGRAL08)029</ref>.</p></para>"#,
  );
  let xml = convert_clean(
    "\\documentclass{JHEP}\n\\title{T}\\abstract{A.}\n\\begin{document}\nSee \\prd{12}{1975}{3}.\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">See <ref href="http://www-spires.slac.stanford.edu/spires/find/hep/www?j=PHRVA%2CD12%2C3"><text font="italic" xml:id="p1.1.1">Phys. Rev. </text><text font="bold" xml:id="p1.1.2">D 12</text> (1975) 3</ref>.</p></para>"#,
  );
}
