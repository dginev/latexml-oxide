//! Red/green guards for batch 77s: brief silent content losses outside the frontmatter — longtable's `\caption*`,
//! algpseudocodex's labelled lines, cleveref's `\label[type]`, T2A's Cyrillic і/ј, `{filecontents}` over a file on
//! disk, `\captionof` of a caption sub-type. Each guard asserts the diagnostics by count and the authored content in its
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
/// before the singular). pdflatex: "See Appendix Table 1, table 2, Appendix Figure 1, Lemma 1, Appendix Table 1 and
/// ?? supp."; the type was dropped ("table 1", "figure 1", "equation 1"; 2404.11739, 2605.13648; Perl the same). The
/// sixth, a `\label[supp]` before the `\caption`, is cleveref's error in pdflatex ("Use of \cref@override@label@type
/// doesn't match its definition", the reference "??"); here the label names the float, LaTeXML's rule for a label in a
/// float, and the float takes the type: "Supplement 3", 0 errors. Repro
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

/// A `\label[type]{key}` in the caption's text (sect09.rs `\@@@hack@caption@`) and in a longtable (longtable_sty.rs
/// `\lx@longtable@label`) reads cleveref's optional type, as the label after a `\caption` does: read as the key, the
/// float was labelled `[` and "Fooappendixtable]tin" printed. pdflatex: "See appendix table 1, appendix table 2,
/// appendix table 4 and table 3." Repro package-bindings/cleveref_label_type_in_caption_text.
#[test]
fn label_type_in_caption_text() {
  let tags = |n: u32, name: &str| {
    let cap = if name == "table" {
      "Table"
    } else {
      "Appendix table"
    };
    format!(
      r#"<tags><tag>Table {n}</tag><tag role="creftype">{name}</tag><tag role="creftypecap">{cap}</tag><tag role="creftypeplural">{name}s</tag><tag role="creftypepluralcap">{cap}s</tag><tag role="refnum">{n}</tag><tag role="typerefnum">Table {n}</tag></tags>"#
    )
  };
  assert_elements(
    r"\documentclass{article}
\usepackage{longtable}
\usepackage{cleveref}
\crefname{appendixtable}{appendix table}{appendix tables}
\begin{document}
See \cref{tin}, \cref{tout}, \cref{lt} and \cref{plain}.
\begin{table}\caption{Foo\label[appendixtable]{tin}}\end{table}
\begin{table}\caption{Bar}\label[appendixtable]{tout}\end{table}
\begin{table}\caption{Baz\label{plain}}\end{table}
\begin{longtable}{l}\caption{Long}\label[appendixtable]{lt}\\ x\\\end{longtable}
End.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        &format!(
          r#"<table inlist="lot" labels="LABEL:tin" xml:id="S0.T1">{}<toccaption><tag close=" ">1</tag>Foo</toccaption><caption><tag close=": ">Table 1</tag>Foo</caption></table>"#,
          tags(1, "appendix table")
        ),
      ),
      (
        "table",
        "S0.T3",
        &format!(
          r#"<table inlist="lot" labels="LABEL:plain" xml:id="S0.T3">{}<toccaption><tag close=" ">3</tag>Baz</toccaption><caption><tag close=": ">Table 3</tag>Baz</caption></table>"#,
          tags(3, "table")
        ),
      ),
      (
        "table",
        "S0.T4",
        &format!(
          r#"<table inlist="lot" labels="LABEL:lt" xml:id="S0.T4">{}<caption><tag close=": ">Table 4</tag>Long</caption><toccaption><tag close=" ">4</tag>Long</toccaption><tabular xml:id="S0.T4.1"><tr xml:id="S0.T4.1.1"><td align="left" xml:id="S0.T4.1.1.1">x</td></tr></tabular></table>"#,
          tags(4, "appendix table")
        ),
      ),
    ],
  );
}

/// caption.sty:348 `\caption@checktype` refuses a sub-float subfig's `\newsubfloat` made (`\sf@counterlist`) as it does
/// caption's own sub-types: `\captionof{subfigure}{First}` in a minipage is the figure's "Figure 1: First" and the
/// `\subfloat` is "(a)" (it was panel (a), the `\subfloat` (b)). Outside any float the `\caption` it falls back to is
/// `\caption@OutsideFloat` (caption.sty:200-207): a second error and the text dropped, "CCC" alone ("CCCLone" printed).
/// pdflatex: 3 errors, "Figure 1: First", "(a) SubA", "Figure 2: Whole", "CCC", "End text." (KNOWN_PERL_ERRORS #999).
/// Repro captions-floats/captionof_subfig_subfloat_is_refused.
#[test]
fn captionof_subfig_subfloat_is_refused() {
  assert_elements_with(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subfig}
\begin{document}
Start.
\begin{figure}
\begin{minipage}{.4\textwidth}AAA\captionof{subfigure}{First}\end{minipage}
\subfloat[SubA]{XXX}
\caption{Whole}
\end{figure}
\begin{minipage}{.4\textwidth}CCC\captionof{subtable}[List]{Lone}\end{minipage}

End text.
\end{document}",
    "ar5iv.sty",
    (3, 0),
    &[
      "Should not be used with subtype `subfigure'",
      "Should not be used with subtype `subtable'",
      "\\caption outside float",
    ],
    &[
      (
        "figure",
        "S0.F2",
        r#"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="138.0pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.1">AAA</p><toccaption><tag close=" ">1</tag>First</toccaption><caption><tag close=": ">Figure 1</tag>First</caption></figure><break class="ltx_break"/><figure class="ltx_figure_panel" xml:id="S0.F2.sf1"><tags><tag>(a)</tag><tag role="refnum">2a</tag></tags><p xml:id="S0.F2.sf1.1">XXX</p><toccaption><tag close=" ">a</tag>SubA</toccaption><caption><tag close=" ">(a)</tag>SubA</caption></figure><toccaption><tag close=" ">2</tag>Whole</toccaption><caption><tag close=": ">Figure 2</tag>Whole</caption></figure>"#,
      ),
      (
        "para",
        "p2",
        r#"<para xml:id="p2"><p xml:id="p2.1"><inline-block class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="p2.1.1"><p xml:id="p2.1.1.1">CCC</p></inline-block></p></para>"#,
      ),
    ],
  );
}

/// With subfigure.sty `subfigure` is a command (`\subfigure[..][..]{..}`), no environment and no caption sub-type,
/// so caption accepts `\captionof{subfigure}` and the caption is a sub-float's: the binding's `{@subfigure}` panel
/// hosts it (`captionof_environment@subfigure`), formatted as caption's own caption — its `colon` separator and its
/// fonts — where `\subfigure`'s panel keeps subfigure.sty's (footnotesize, a space). `\begin{subfigure}` called
/// `\subfigure` with the caption's internals as its arguments: 20 errors, "subfigureFirst", the figure's caption
/// numbered "(c)" and the text after it drawn in (2508.00020); r2's panel read "(a) First" in subfigure.sty's format
/// (1608.05355, 1806.00503). pdflatex: 0 errors, "(a): First", "(b) SubA", "Figure 1: Whole". subfig loaded after it
/// makes no `subfigure` of its own (subfig.sty:298 `\@ifundefined{c@subfigure}{\newsubfloat{figure}}{}`), so its
/// `\sf@counterlist` does not name it and caption still accepts the type (2508.00020's preamble: 52 errors at int34a,
/// 0 now and Figs. 4-9 as the PDF numbers them). Repros captions-floats/captionof_subfigure_sty_is_a_panel,
/// captions-floats/captionof_subfigure_sty_before_subfig.
#[test]
fn captionof_subfigure_sty_is_a_panel() {
  assert_elements(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subfigure}
\begin{document}
Start.
\begin{figure}
\begin{minipage}{.4\textwidth}AAA\captionof{subfigure}{First}\end{minipage}
\subfigure[SubA]{XXX}
\caption{Whole}
\end{figure}
End text.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "figure",
        "S0.F1",
        r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure class="ltx_figure_panel ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.F0.sf1"><tags><tag><text fontsize="80%" xml:id="S0.F0.sf1.1">(a)</text></tag><tag role="refnum">0(a)</tag></tags><p class="ltx_figure_panel" xml:id="S0.F0.sf1.2">AAA</p><break class="ltx_break"/><toccaption><tag close=" ">(a)</tag>First</toccaption><caption><tag close=": ">(a)</tag>First</caption></figure><break class="ltx_break"/><figure class="ltx_figure_panel" xml:id="S0.F1.sf2"><tags><tag><text fontsize="80%" xml:id="S0.F1.sf2.1">(b)</text></tag><tag role="refnum">1(b)</tag></tags><p xml:id="S0.F1.sf2.2">XXX</p><toccaption><tag close=" ">(b)</tag>SubA</toccaption><caption><tag close=" "><text fontsize="80%" xml:id="S0.F1.sf2.3">(b)</text></tag><text fontsize="80%" xml:id="S0.F1.sf2.4">SubA</text></caption></figure><toccaption><tag close=" ">1</tag>Whole</toccaption><caption><tag close=": ">Figure 1</tag>Whole</caption></figure>"#,
      ),
      (
        "para",
        "p2",
        r#"<para xml:id="p2"><p xml:id="p2.1">End text.</p></para>"#,
      ),
    ],
  );
  assert_elements(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subfigure}
\usepackage{subfig}
\begin{document}
Start.
\begin{figure}
\begin{minipage}{.4\textwidth}AAA\captionof{subfigure}{First}\end{minipage}
\subfigure[SubA]{XXX}
\caption{Whole}
\end{figure}
End text.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[(
      "figure",
      "S0.F0.sf1",
      r#"<figure class="ltx_figure_panel ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.F0.sf1"><tags><tag><text fontsize="80%" xml:id="S0.F0.sf1.1">(a)</text></tag><tag role="refnum">0a</tag></tags><p class="ltx_figure_panel" xml:id="S0.F0.sf1.2">AAA</p><break class="ltx_break"/><toccaption><tag close=" ">a</tag>First</toccaption><caption><tag close=": ">(a)</tag>First</caption></figure>"#,
    )],
  );
}

/// The starred `\captionof*{subfigure}` is hosted as the unstarred one (`\@scaptionof`, `captionof_host`): it opened the
/// undefined `{subfigure*}` (an Error and a visible `{subfigure*}` node). `\captionsetup{labelformat=empty}` before a
/// hosted panel drops its label, as for `\captionof{figure}` (2605.30420's 20 panels print no label). pdflatex: 0
/// errors, "First" (no label), "(b) SubA", "Figure 1: Whole", "Star"; it then numbers the second figure's caption
/// "(a): Whole2", the type `subfigure` left set in the float (caption.sty:391) where ours keeps the float's own type
/// (`\lx@caption@of@set@after`, guard `perfect_kernel_gemini::captionof_in_a_float_keeps_the_float_type`). Repro
/// captions-floats/captionof_star_subfigure_sty_is_a_panel.
#[test]
fn captionof_star_subfigure_sty_is_a_panel() {
  assert_elements(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subfigure}
\begin{document}
\begin{figure}
\begin{minipage}{.4\textwidth}AAA\captionsetup{labelformat=empty}\captionof{subfigure}{First}\end{minipage}
\subfigure[SubA]{XXX}
\caption{Whole}
\end{figure}
\begin{figure}
\captionof*{subfigure}{Star}
\caption{Whole2}
\end{figure}
End text.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "figure",
        "S0.F0.sf1",
        r#"<figure class="ltx_figure_panel ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.F0.sf1"><tags><tag><text fontsize="80%" xml:id="S0.F0.sf1.1">(a)</text></tag><tag role="refnum">0(a)</tag></tags><p class="ltx_figure_panel" xml:id="S0.F0.sf1.2">AAA</p><break class="ltx_break"/><toccaption><tag close=" ">(a)</tag>First</toccaption><caption>First</caption></figure>"#,
      ),
      (
        "figure",
        "S0.F2",
        r#"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><figure xml:id="S0.F2.fig1"><caption>Star</caption></figure><toccaption><tag close=" ">2</tag>Whole2</toccaption><caption><tag close=": ">Figure 2</tag>Whole2</caption></figure>"#,
      ),
      (
        "para",
        "p1",
        r#"<para xml:id="p1"><p xml:id="p1.1">End text.</p></para>"#,
      ),
    ],
  );
}

/// A `\captionof` takes the caption options in force where it stands: the enclosing float's
/// (`\captionsetup[table]{…}`, applied when the table begins), then its own type's on top (caption.sty:300-320
/// `\caption@@settype` → `\caption@setoptions`; `caption_setting`'s `lx@caption@outer`). 2502.11752's
/// `\captionof{subtable}` panels in a table under `\captionsetup[table]{labelfont=bf,labelsep=space}` print "(a) For
/// participant 1" (r2 and before: subfigure.sty's format; a colon with the separator alone). pdflatex: "(a) For one"
/// with a bold label, "Figure 1. Fig in table" with a bold label, "Table 1 Whole", "(a). Sub in fig", "Figure 2.
/// FWhole", "(a): Loose"; with `labelformat=simple` "(a) For one", where the label read " (a)" (`\bothIfFirst`
/// tested the name unexpanded). Repro captions-floats/captionof_takes_the_enclosing_float_options.
#[test]
fn captionof_takes_the_enclosing_float_options() {
  assert_elements(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subfigure}
\captionsetup[table]{labelfont=bf,labelsep=space}
\captionsetup[figure]{labelsep=period}
\begin{document}
\begin{table}
\begin{minipage}{.4\textwidth}AAA\captionof{subtable}{For one}\end{minipage}
\begin{minipage}{.4\textwidth}BBB\captionof{figure}{Fig in table}\end{minipage}
\caption{Whole}
\end{table}
\begin{figure}
\begin{minipage}{.4\textwidth}CCC\captionof{subfigure}{Sub in fig}\end{minipage}
\caption{FWhole}
\end{figure}
Out \captionof{subfigure}{Loose} end.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><figure class="ltx_figure_panel ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.T0.st1"><tags><tag><text fontsize="80%" xml:id="S0.T0.st1.1">(a)</text></tag><tag role="refnum">0(a)</tag></tags><p class="ltx_figure_panel" xml:id="S0.T0.st1.2">AAA</p><break class="ltx_break"/><toccaption><tag close=" ">(a)</tag>For one</toccaption><caption><tag close=" "><text font="bold" xml:id="S0.T0.st1.3">(a)</text></tag>For one</caption></figure><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="138.0pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p class="ltx_figure_panel" xml:id="S0.F1.1">BBB</p><break class="ltx_break"/><toccaption><tag close=" ">1</tag>Fig in table</toccaption><caption><tag close=". "><text font="bold" xml:id="S0.F1.2">Figure 1</text></tag>Fig in table</caption></figure><toccaption><tag close=" ">1</tag>Whole</toccaption><caption><tag close=" "><text font="bold" xml:id="S0.T1.1">Table 1</text></tag>Whole</caption></table>"#,
      ),
      (
        "figure",
        "S0.F2",
        r#"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><figure class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.F1.sf1"><tags><tag><text fontsize="80%" xml:id="S0.F1.sf1.1">(a)</text></tag><tag role="refnum">1(a)</tag></tags><p class="ltx_figure_panel" xml:id="S0.F1.sf1.2">CCC</p><break class="ltx_break"/><toccaption><tag close=" ">(a)</tag>Sub in fig</toccaption><caption><tag close=". ">(a)</tag>Sub in fig</caption></figure><toccaption><tag close=" ">2</tag>FWhole</toccaption><caption><tag close=". ">Figure 2</tag>FWhole</caption></figure>"#,
      ),
    ],
  );
  // `labelformat=simple` for a type with no name (caption3.sty:54-65 `\bothIfFirst` expands its first argument):
  // "(a) For one", no leading space; `parens` "Figure (1): Paren".
  assert_elements(
    r"\documentclass{article}
\usepackage{caption}
\usepackage{subfigure}
\captionsetup[table]{labelfont=bf,labelformat=simple,labelsep=space,name=Supplementary Table}
\begin{document}
\begin{table}
\begin{minipage}{.4\textwidth}AAA\captionof{subtable}{For one}\end{minipage}
\caption{Whole}
\end{table}
\begin{figure}
\captionsetup{labelformat=parens}\caption{Paren}
\end{figure}
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><figure class="ltx_minipage" vattach="middle" width="138.0pt" xml:id="S0.T0.st1"><tags><tag><text fontsize="80%" xml:id="S0.T0.st1.1">(a)</text></tag><tag role="refnum">0(a)</tag></tags><p class="ltx_figure_panel" xml:id="S0.T0.st1.2">AAA</p><break class="ltx_break"/><toccaption><tag close=" ">(a)</tag>For one</toccaption><caption><tag close=" "><text font="bold" xml:id="S0.T0.st1.3">(a)</text></tag>For one</caption></figure><toccaption><tag close=" ">1</tag>Whole</toccaption><caption><tag close=" "><text font="bold" xml:id="S0.T1.1">Supplementary Table 1</text></tag>Whole</caption></table>"#,
      ),
      (
        "figure",
        "S0.F1",
        r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Paren</toccaption><caption><tag close=": ">Figure (1)</tag>Paren</caption></figure>"#,
      ),
    ],
  );
}

/// cleveref.sty:1309 derives both names of the other variant with a case change, or neither, as the SINGULAR is empty
/// or not (`\expandafter\ifx\@tempa\@empty`): `\crefname{note}{}{notes}` gives `\Cref` the plural "notes" unchanged;
/// `\Crefname{memo}{Memo}{}` gives `\cref` "memo" and an empty plural. pdflatex: "See notes 1 and 2, 1, 1 and 2 and
/// memo 1." Was per name ("Notes"). Repro package-bindings/cleveref_derived_name_follows_the_singular.
#[test]
fn cleveref_derived_name_follows_the_singular() {
  assert_elements(
    r"\documentclass{article}
\usepackage{cleveref}
\crefname{note}{}{notes}
\Crefname{memo}{Memo}{}
\newtheorem{note}{Note}
\newtheorem{memo}{Memo}
\begin{document}
\begin{note}\label{n:a}A.\end{note}
\begin{note}\label{n:b}B.\end{note}
\begin{memo}\label{m:a}C.\end{memo}
\begin{memo}\label{m:b}D.\end{memo}
See \Cref{n:a,n:b}, \cref{n:a}, \cref{m:a,m:b} and \cref{m:a}.
\end{document}",
    "ar5iv.sty",
    (0, 0),
    &[
      (
        "theorem",
        "Thmnote1",
        r#"<theorem class="ltx_theorem_note" inlist="thm theorem:note" labels="LABEL:n:a" xml:id="Thmnote1"><tags><tag>Note 1</tag><tag role="creftypeplural">notes</tag><tag role="creftypepluralcap">notes</tag><tag role="refnum">1</tag><tag role="typerefnum">Note 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmnote1.1">Note 1</text></tag></title><para xml:id="Thmnote1.p1"><p xml:id="Thmnote1.p1.1"><text font="italic" xml:id="Thmnote1.p1.1.1">A.</text></p></para></theorem>"#,
      ),
      (
        "theorem",
        "Thmmemo1",
        r#"<theorem class="ltx_theorem_memo" inlist="thm theorem:memo" labels="LABEL:m:a" xml:id="Thmmemo1"><tags><tag>Memo 1</tag><tag role="creftype">memo</tag><tag role="creftypecap">Memo</tag><tag role="refnum">1</tag><tag role="typerefnum">Memo 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmmemo1.1">Memo 1</text></tag></title><para xml:id="Thmmemo1.p1"><p xml:id="Thmmemo1.p1.1"><text font="italic" xml:id="Thmmemo1.p1.1.1">C.</text></p></para></theorem>"#,
      ),
    ],
  );
}
