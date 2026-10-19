//! Red/green guards for batch 77s: brief silent content losses outside the frontmatter — longtable's `\caption*`,
//! algpseudocodex's labelled lines, cleveref's `\label[type]`, T2A's Cyrillic і/ј, `{filecontents}` over a file on disk,
//! `\captionof` of a caption sub-type. Each guard asserts the diagnostics by count and the authored content in its
//! element.
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{assert_elements, assert_elements_with},
};

/// longtable.sty:471-474 `\LT@capti@n`: `\caption*{text}` is `\LT@c@ption\@gobble[]{text}` — the text alone, no
/// "Table n:", no List of Tables line; the table keeps the number its begin stepped (tables 1, 3, 4 around it).
/// Perl's `\lx@longtable@caption[]{}` has no star: "Table 2: *", the text lost (KNOWN_PERL_ERRORS #363). Repro
/// captions-floats/longtable_starred_caption_keeps_its_text.
#[test]
fn longtable_starred_caption_keeps_its_text() {
  let xml = assert_elements(
    r"\documentclass{article}
\usepackage{longtable}
\begin{document}
\begin{table}A\caption{A}\end{table}
\begin{longtable}{l}\caption*{LT star}\\ x\\\end{longtable}
\begin{table}B\caption{B}\end{table}
\begin{table}C\caption{C}\end{table}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "table",
      "S0.T2",
      r#"<table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><caption>LT star</caption><tabular xml:id="S0.T2.1"><tr xml:id="S0.T2.1.1"><td align="left" xml:id="S0.T2.1.1.1">x</td></tr></tabular></table>"#,
    )],
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Table 3</tag>B</caption>"#),
    "{xml}"
  );
}

/// algpseudocodex's box check runs through the algorithmicx binding's line hook (`\lx@algorithmicx@item@hook`):
/// its own `\lx@algorithmicx@item[]` dropped an `\item[label]`'s label, so `\Require` (algpseudocodex.sty:676
/// `\item[\algorithmicrequire]`) lost "Require:" and `\Statex` (`\item[]`, algorithmicx.sty:632) stepped the line
/// number. pdflatex: "Require: data", "1: x", "[Phase one] begins" unnumbered, "2: y"; the `\Statex` text is a break in
/// the line box still open (54). Repro list-structure/algpseudocodex_require_shows_its_label.
#[test]
fn algpseudocodex_require_shows_its_label() {
  assert_elements(
    r"\documentclass{article}
\usepackage{algorithm}
\usepackage{algpseudocodex}
\begin{document}
\begin{algorithmic}[1]
\Require data
\State x
\Statex [Phase one] begins
\State y
\end{algorithmic}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "listing",
      "p1.1",
      r#"<listing class="ltx_algpx_indlines" xml:id="p1.1"><listingline xml:id="algx1.l0"><tags><tag><text font="bold" xml:id="algx1.l0.1">Require:</text></tag></tags><text class="ltx_algpx_code" xml:id="algx1.l0.2">data</text></listingline><listingline xml:id="algx1.l1"><tags><tag><text fontsize="80%" xml:id="algx1.l1.1">1:</text></tag><tag role="refnum">1</tag></tags><text class="ltx_algpx_code" xml:id="algx1.l1.2">x<break/>[Phase one] begins</text></listingline><listingline xml:id="algx1.l2"><tags><tag><text fontsize="80%" xml:id="algx1.l2.1">2:</text></tag><tag role="refnum">2</tag></tags><text class="ltx_algpx_code" xml:id="algx1.l2.2">y</text></listingline></listing>"#,
    )],
  );
}

/// cleveref's `\label[type]{key}`: the object the label names takes the type in its cleveref tags, which `\cref`
/// prints — after `\caption` (the kernel's `\@caption@postlabel` reads the optional), before it, on a theorem and on an
/// equation. The capitalised form is derived from `\crefname` as cleveref.sty:1296-1325 derives it (`\MakeUppercase`
/// before the singular). pdflatex: "See Appendix Table 1, table 2, Appendix Figure 1, Lemma 1 and Appendix Table 1";
/// the type was dropped ("table 1", "figure 1", "equation 1"; 2404.11739, 2605.13648; Perl the same). Repro
/// package-bindings/cleveref_label_type_names_the_reference.
#[test]
fn cleveref_label_type_names_the_reference() {
  let tags = |name: &str, plural: &str, cap: &str, pluralcap: &str| {
    format!(
      r#"<tag role="creftype">{name}</tag><tag role="creftypecap">{cap}</tag><tag role="creftypeplural">{plural}</tag><tag role="creftypepluralcap">{pluralcap}</tag>"#
    )
  };
  let appendix_table = tags(
    "Appendix Table",
    "Appendix Tables",
    "Appendix Table",
    "Appendix Tables",
  );
  assert_elements(
    r"\documentclass{article}
\usepackage{amsthm}
\usepackage{cleveref}
\crefname{appendixtable}{Appendix Table}{Appendix Tables}
\crefname{appendixfigure}{Appendix Figure}{Appendix Figures}
\crefname{supp}{supplement}{supplements}
\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\begin{document}
See \cref{tab:a}, \cref{tab:b}, \Cref{fig:a}, \Cref{lem:a}, \cref{eq:a} and \Cref{tab:c}.
\begin{table}\caption{A}\label[appendixtable]{tab:a}\end{table}
\begin{table}\caption{B}\label{tab:b}\end{table}
\begin{figure}\caption{F}\label[appendixfigure]{fig:a}\end{figure}
\begin{lemma}\label[lemma]{lem:a} L.\end{lemma}
\begin{equation}\label[appendixtable]{eq:a} x=1\end{equation}
\begin{table}\label[supp]{tab:c}\caption{C}\end{table}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        &format!(
          r#"<table inlist="lot" labels="LABEL:tab:a" xml:id="S0.T1"><tags><tag>Table 1</tag>{appendix_table}<tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>A</toccaption><caption><tag close=": ">Table 1</tag>A</caption></table>"#
        ),
      ),
      (
        "table",
        "S0.T2",
        &format!(
          r#"<table inlist="lot" labels="LABEL:tab:b" xml:id="S0.T2"><tags><tag>Table 2</tag>{}<tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>B</toccaption><caption><tag close=": ">Table 2</tag>B</caption></table>"#,
          tags("table", "tables", "Table", "Tables")
        ),
      ),
      (
        "figure",
        "S0.F1",
        &format!(
          r#"<figure inlist="lof" labels="LABEL:fig:a" xml:id="S0.F1"><tags><tag>Figure 1</tag>{}<tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>F</toccaption><caption><tag close=": ">Figure 1</tag>F</caption></figure>"#,
          tags(
            "Appendix Figure",
            "Appendix Figures",
            "Appendix Figure",
            "Appendix Figures"
          )
        ),
      ),
      (
        "equation",
        "S0.E1",
        &format!(
          r#"<equation labels="LABEL:eq:a" xml:id="S0.E1"><tags><tag>(1)</tag>{appendix_table}<tag role="refnum">1</tag></tags><Math mode="display" tex="x=1" text="x = 1" xml:id="S0.E1.m1"><XMath xml:id="S0.E1.m1.1"><XMApp xml:id="S0.E1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></equation>"#
        ),
      ),
      (
        "table",
        "S0.T3",
        &format!(
          r#"<table inlist="lot" labels="LABEL:tab:c" xml:id="S0.T3"><tags><tag>Table 3</tag>{}<tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>C</toccaption><caption><tag close=": ">Table 3</tag>C</caption></table>"#,
          tags("supplement", "supplements", "Supplement", "Supplements")
        ),
      ),
    ],
  );
}

/// T2A sets Cyrillic і and ј (and І Ј) in the slots of Latin i and j (t2aenc.def:230-233), whose glyphs they share; the
/// symbols `\cyrii`/`\cyrje` (a UTF-8 `і`/`ј`, t2aenc.dfu) are the Cyrillic letters, where Perl's fontmap gave Latin
/// i/j. A Latin letter typed in T2A text stays Latin. OXIDIZED_DESIGN_DIVERGENCES #776; repro
/// fonts-nfss/t2a_cyrillic_i_and_je_are_cyrillic.
#[test]
fn t2a_cyrillic_i_and_je_are_cyrillic() {
  assert_elements(
    r"\documentclass{article}
\usepackage[T2A]{fontenc}
\usepackage[utf8]{inputenc}
\begin{document}
Ukrainian: і, Ї, Іван. Serbian: ј, Ј. Explicit: \cyrii\ \CYRII\ \cyrje\ \CYRJE. Latin: ij IJ.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">Ukrainian: \u{0456}, \u{0407}, \u{0406}\u{0432}\u{0430}\u{043D}. Serbian: \u{0458}, \u{0408}. \
       Explicit: \u{0456} \u{0406} \u{0458} \u{0408}. Latin: ij IJ.</p>",
    )],
  );
}

/// latex.ltx:18990-19004 `\filec@ntents`: a file `\openin` finds is kept unless `[force]`/`[overwrite]` — the
/// `democode.tex` beside the document, and a file an earlier `{filecontents}` wrote; the environment's lines are
/// read and dropped. pdflatex: "Disk version. Forced source. Fresh source." (KNOWN_PERL_ERRORS #279 residual). Repro
/// string-mouth/filecontents_disk_file.
#[test]
fn filecontents_keeps_a_file_on_disk() {
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\begin{filecontents}{democode}
Source version.
\end{filecontents}
\begin{filecontents}[overwrite]{forced.tex}
Forced source.
\end{filecontents}
\begin{filecontents*}{fresh.tex}
Fresh source.
\end{filecontents*}
\begin{filecontents*}{fresh.tex}
Second fresh.
\end{filecontents*}
\begin{document}
\input{democode} \input{forced} \input{fresh}
\end{document}",
    &[
      ("democode.tex", "Disk version.\n"),
      ("forced.tex", "Forced disk.\n"),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Disk version. Forced source. Fresh source.</p>",
  );
}

/// caption.sty:346-352 `\caption@checktype`: `\captionof{subfigure}` is refused ("Should not be used with subtype
/// `subfigure'"), the type left as it was, and the caption made by `\caption` — of the enclosing figure, or a
/// sub-caption inside a `{subfigure}`. Wrapped in the sub-type's environment, the caption's internals were read as its
/// width and, in a TikZ `\node`, cascaded mode errors (28 here) that emptied the figure and lost the text after it
/// (2008.05535; pdflatex 2 errors, the second TikZ's own). Repro captions-floats/captionof_subfigure_in_a_tikz_node.
#[test]
fn captionof_subtype_is_refused() {
  const REFUSED: &str = "Should not be used with subtype `subfigure'";
  let xml = assert_elements_with(
    r"\documentclass{article}
\usepackage{tikz}
\usepackage{caption}
\usepackage{subcaption}
\begin{document}
\begin{figure}
\centering
\begin{tikzpicture}
\draw (0,0) rectangle (2,1);
\node[text width=0.45\textwidth,align=left,anchor=south west] at (-0.7,-1.6) {\captionof{subfigure}{Normalized vertiport capacity = 1}};
\end{tikzpicture}
\caption{Whole.}
\end{figure}
Text after.
\end{document}",
    "ar5iv.sty",
    (1, 0),
    &[REFUSED],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p xml:id="p1.1">Text after.</p></para>"#,
    )],
  );
  for caption in [
    r#"<caption class="ltx_align_left ltx_centering"><tag close=": "><text fontsize="90%" xml:id="S0.F2.2">Figure 1</text></tag><text fontsize="90%" xml:id="S0.F2.3">Normalized vertiport capacity = 1</text></caption>"#,
    r#"<caption class="ltx_centering"><tag close=": "><text fontsize="90%" xml:id="S0.F2.4">Figure 2</text></tag><text fontsize="90%" xml:id="S0.F2.5">Whole.</text></caption>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
  // In a minipage the caption is the figure's ("Figure 1: First"); in a {subfigure} a sub-caption ("(a) Second").
  let xml = assert_elements_with(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subcaption}
\begin{document}
\begin{figure}
\begin{minipage}{0.4\textwidth}A\captionof{subfigure}{First}\label{f:a}\end{minipage}
\begin{subfigure}{0.4\textwidth}B\captionof{subfigure}{Second}\end{subfigure}
\caption{Whole.}
\end{figure}
See \ref{f:a}. Text after.
\end{document}",
    "ar5iv.sty",
    (2, 0),
    &[REFUSED, REFUSED],
    &[(
      "figure",
      "S0.F2.sf1",
      r#"<figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F2.sf1"><tags><tag><text fontsize="90%" xml:id="S0.F2.sf1.1">(a)</text></tag><tag role="refnum">2a</tag></tags><p xml:id="S0.F2.sf1.2">B</p><toccaption><tag close=" ">a</tag>Second</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F2.sf1.3">(a)</text></tag><text fontsize="90%" xml:id="S0.F2.sf1.4">Second</text></caption></figure>"#,
    )],
  );
  assert!(
    xml.contains(r#"<caption><tag close=": "><text fontsize="90%" xml:id="S0.F1.3">Figure 1</text></tag><text fontsize="90%" xml:id="S0.F1.4">First</text></caption>"#),
    "{xml}"
  );
}
