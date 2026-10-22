//! Red/green guards for batch 78p (quality batch 13): tabularray's rule keys are drawn as cell borders, `\patchcmd`
//! rescans its patched text, a `\label` after an inner equation names the theorem, delarray delimits a `tabular`,
//! mathabx's `\curvearrowbotleftright`, an unused glossary entry raises nothing, and `multicols` in `algorithmic`
//! makes no pagination the listing cannot hold. Each guard converts its repro under `tools/perfect_kernel/repros/`
//! and asserts no Error or Warning and the authored content in whole elements, against pdflatex (fidelity3 audit
//! N12, N14, N16-N19; the SYNC tabularray row).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// tabularray's `hlines,vlines` over an all-empty table (the manual's `\makeEmptyTable`, tabularray.tex:2859-2870) keep
/// the grid, bordered as a `tabular` with `|c|c|c|` and `\hline`s is. Red (int52a): no table at all, every cell empty
/// and unbordered, pruned. Repro alignment-bindings/tblr_all_empty_table_keeps_its_rules.
#[test]
fn tblr_rules_keep_an_empty_table() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/tblr_all_empty_table_keeps_its_rules.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="p2.1"><tbody>
      <tr xml:id="p2.1.1"><td border="l r t" xml:id="p2.1.1.1"/><td border="r t" xml:id="p2.1.1.2"/><td border="r t" xml:id="p2.1.1.3"/></tr>
      <tr xml:id="p2.1.2"><td border="b l r t" xml:id="p2.1.2.1"/><td border="b r t" xml:id="p2.1.2.2"/><td border="b r t" xml:id="p2.1.2.3"/></tr>
    </tbody></tabular>"#,
  );
}

/// The rule keys as tabularray reads their values (`{<index>}{<cross>}{<style>}`, `{<cross>}{<style>}`, `<style>`) and
/// a rowspec's `|`s: an hline over some columns, a second line at a place (a double rule), a vline over some rows;
/// `text=` and white rules draw nothing. Red (int52a): no border anywhere. Repro
/// alignment-bindings/tblr_rule_keys_become_cell_borders.
#[test]
fn tblr_rule_keys_become_cell_borders() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/tblr_rule_keys_become_cell_borders.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (the kernel guesses the header row and column from the rules, as it does a `tabular`'s)
  for (id, table) in [
    (
      "p1.1",
      r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" border="t" thead="row" xml:id="p1.1.1.1">a</td><td align="center" border="t" xml:id="p1.1.1.2">b</td><td align="center" border="t" xml:id="p1.1.1.3">c</td></tr><tr xml:id="p1.1.2"><td align="center" border="b" thead="row" xml:id="p1.1.2.1">d</td><td align="center" border="b t" xml:id="p1.1.2.2">e</td><td align="center" border="b t" xml:id="p1.1.2.3">f</td></tr></tbody></tabular>"#,
    ),
    (
      "p1.2",
      r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p1.2"><thead><tr xml:id="p1.2.1"><td align="center" border="r tt" thead="column row" xml:id="p1.2.1.1">a</td><td align="center" border="tt" thead="column" xml:id="p1.2.1.2">b</td><td align="center" border="tt" thead="column" xml:id="p1.2.1.3">c</td></tr></thead><tbody><tr xml:id="p1.2.2"><td align="center" thead="row" xml:id="p1.2.2.1">d</td><td align="center" xml:id="p1.2.2.2">e</td><td align="center" xml:id="p1.2.2.3">f</td></tr></tbody></tabular>"#,
    ),
    (
      "p1.3",
      r#"<tabular vattach="middle" xml:id="p1.3"><tbody><tr xml:id="p1.3.1"><td align="center" border="l t" xml:id="p1.3.1.1">a</td><td align="center" border="r t" xml:id="p1.3.1.2">b</td></tr><tr xml:id="p1.3.2"><td align="center" border="b l t" xml:id="p1.3.2.1">c</td><td align="center" border="b r t" xml:id="p1.3.2.2">d</td></tr></tbody></tabular>"#,
    ),
    (
      "p1.4",
      r#"<tabular vattach="middle" xml:id="p1.4"><tbody><tr xml:id="p1.4.1"><td align="left" border="l r" xml:id="p1.4.1.1">a</td><td align="left" border="r" xml:id="p1.4.1.2">b</td></tr><tr xml:id="p1.4.2"><td align="left" border="l r" xml:id="p1.4.2.1">c</td><td align="left" border="r" xml:id="p1.4.2.2">d</td></tr></tbody></tabular>"#,
    ),
  ] {
    assert_element(&xml, "tabular", &[&format!("xml:id=\"{id}\"")], table);
  }
}

/// `\patchcmd{\prArg}{|}{\mid}` on `o_t|s_t` rescans `o_t\mid s_t` (etoolbox.sty:1358-1374, the `\meaning` and the
/// `\detokenize`d texts). Red (int52a, and Perl): `\mids`, undefined. Witness 2011.13897. Repro
/// expansion-primitives/patchcmd_replacement_rescans_a_control_word.
#[test]
fn patchcmd_replacement_rescans_a_control_word() {
  let (log, xml) = convert_with(
    repro!("expansion-primitives/patchcmd_replacement_rescans_a_control_word.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"tex="p(o_{t}\mid s_{t})""#), "{xml}");
}

/// A patch's search text ends a control word where `\meaning` does (`\foo ` never matches `\foobar`), a nested
/// parameter is `##`, and a replacement before a letter keeps its control word (pdflatex 0 errors: "[aXFy]" and
/// "αb" → "βb"; the nested patch etoolbox refuses for its `#`, our `\etb@ifpattern` stub accepts as before).
#[test]
fn patchcmd_matches_whole_control_words() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{etoolbox}\\makeatletter\\def\\foo@bar{F}\\def\\foo@barx{G}\
     \\def\\qq#1{[#1X\\foo@barx y]}\\patchcmd{\\qq}{\\foo@bar}{\\relax}{}{}\\patchcmd{\\qq}{X}{\\foo@bar}{}{}\
     \\def\\word{\\alpha b}\\patchcmd{\\word}{\\alpha}{\\beta}{}{}\\makeatother\
     \\begin{document}\\qq{a}, $\\word$.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">[aFGy], <Math mode="inline" tex="\beta b""#),
    "{xml}"
  );
}

/// A `\label` after `\end{equation}`, still inside the theorem, names the theorem: LaTeX's `\@currentlabel` reverts
/// when the equation's group closes (latex.ltx:14963-14967). Red (int52a, and Perl): the equation carried both labels,
/// `\ref{thm}` printed its number. Witness math0107195 ("Theorem 5.3"). Repro
/// cross-references/label_after_an_inner_equation_names_the_theorem.
#[test]
fn label_after_an_inner_equation_names_the_theorem() {
  let (log, xml) = convert_with(
    repro!("cross-references/label_after_an_inner_equation_names_the_theorem.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"labels="LABEL:thm" xml:id="Thmtheorem1""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:eq" xml:id="S0.E1">"#),
    "{xml}"
  );
}

/// The label's step decides, not the nearest element: after a closed table, equation or list the label names the
/// section, and a label after `\caption` in its float, or on an item, still names those (their steps are in scope).
/// pdflatex "Refs: 1 1 1 1 1 1.": the section, the figure, then the section for `tt`, `e2` and `l1`, the item.
#[test]
fn label_after_a_closed_step_names_the_step_in_scope() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\begin{document}\\section{One}\\label{s1}\
     \\begin{figure}\\caption{F}\\label{f1}\\end{figure}\
     \\begin{table}\\caption{T}\\end{table}\\label{tt}\
     \\begin{equation}c=d\\end{equation}\\label{e2}\
     \\begin{enumerate}\\item x\\label{i1}\\end{enumerate}\\label{l1}\
     Refs: \\ref{s1} \\ref{f1} \\ref{tt} \\ref{e2} \\ref{i1} \\ref{l1}.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"labels="LABEL:e2 LABEL:l1 LABEL:s1 LABEL:tt""#),
    "{xml}"
  );
  assert!(xml.contains(r#"labels="LABEL:f1""#), "{xml}");
  assert!(xml.contains(r#"<item labels="LABEL:i1""#), "{xml}");
}

/// delarray delimits a `tabular` as it does an `array` (delarray.sty:43-58 through array.sty's `\@tabarray`): the
/// tabular in a formula between the delimiters. Red (int52a): "(" read as the template, two "Extra alignment tab".
/// Witness 1304.5109. Repro alignment-bindings/delarray_delimits_a_tabular.
#[test]
fn delarray_delimits_a_tabular() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/delarray_delimits_a_tabular.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"text="Array[[[a], [b]], [[c], [d]]]""#)
      && xml.contains(r#"<XMTok role="OPEN" stretchy="true">(</XMTok>"#)
      && xml.contains(r#"<XMTok role="CLOSE" stretchy="true">)</XMTok>"#),
    "{xml}"
  );
}

/// A plain tabular, and one inside a delimited one's cell, keep their templates; a `[t]` position is read before the
/// delimiter (1304.5109 `\begin{tabular}[t]\{{l}.`).
#[test]
fn delarray_plain_tabular_in_a_delimited_one() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{delarray}\\begin{document}\
     A \\begin{tabular}[t]({c|c}) x & \\begin{tabular}{cc} p & q \\end{tabular} \\end{tabular} B.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">A <Math mode="inline" tex="\left(\begin{tabular}[t]{c|c}x&amp;\begin{tabular}[]{cc}p&amp;q\end{tabular}\end{tabular}\right)" text="Array[[[x], [pq]]]" xml:id="p1.m1"><XMath xml:id="p1.m1.2"><XMDual xml:id="p1.m1.2.1"><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.1"/><XMWrap xml:id="p1.m1.2.1.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMArray vattach="top" xml:id="p1.m1.1"><XMRow xml:id="p1.m1.1.1"><XMCell align="center" border="r" xml:id="p1.m1.1.1.1"><XMText xml:id="p1.m1.1.1.1.1"><text xml:id="p1.m1.1.1.1.1.1">x</text></XMText></XMCell><XMCell align="center" xml:id="p1.m1.1.1.2"><XMText xml:id="p1.m1.1.1.2.1"><tabular vattach="middle" xml:id="p1.m1.1.1.2.1.1"><tr xml:id="p1.m1.1.1.2.1.1.1"><td align="center" xml:id="p1.m1.1.1.2.1.1.1.1"><text xml:id="p1.m1.1.1.2.1.1.1.1.1">p</text></td><td align="center" xml:id="p1.m1.1.1.2.1.1.1.2"><text xml:id="p1.m1.1.1.2.1.1.1.2.1">q</text></td></tr></tabular></XMText></XMCell></XMRow></XMArray><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math> B.</p>"#,
  );
}

/// mathabx's `\curvearrowbotleftright` (mathabx.dcl:639) is an arrow, U+293B. Red (int52a, and Perl): undefined.
/// Witness 2308.12945. Repro package-bindings/mathabx_curvearrowbotleftright.
#[test]
fn mathabx_curvearrowbotleftright() {
  let (log, xml) = convert_with(
    repro!("package-bindings/mathabx_curvearrowbotleftright.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<XMTok name=\"curvearrowbotleftright\" role=\"ARROW\">\u{293B}</XMTok>"),
    "{xml}"
  );
}

/// An unused glossary entry's fields raise nothing: they are typeset only where the entry is used. Red (int52a, and
/// Perl): the definition's digest raised `\undefineddesc`. Witness 2104.09252. Repro
/// index-bib/unused_glossary_entry_raises_nothing.
#[test]
fn unused_glossary_entry_raises_nothing() {
  let (log, xml) = convert_with(
    repro!("index-bib/unused_glossary_entry_raises_nothing.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "glossarydefinition",
    &[],
    r#"<glossarydefinition inlist="main" key="unused"><glossaryphrase key="unused" role="description">Never <ERROR class="undefined" xml:id="id1">\undefineddesc</ERROR> used</glossaryphrase><glossaryphrase key="unused" role="name">unused</glossaryphrase><glossaryphrase key="unused" role="sort">unused</glossaryphrase></glossarydefinition>"#,
  );
}

/// A used entry in a printed glossary raises its definition's error, as pdflatex does where the glossary prints the
/// description (pdflatex 1 "Undefined control sequence", `\glo@used@desc`; `\gls` prints the name); an unused entry
/// with the same fault raises nothing.
#[test]
fn printed_glossary_entry_raises_its_definition_errors() {
  let (log, _xml) = convert_with(
    "\\documentclass{article}\\usepackage{glossaries}\\makeglossaries\
     \\newglossaryentry{used}{name={used},description={Has \\badone{} inside}}\
     \\newglossaryentry{idle}{name={idle},description={Has \\badtwo{} inside}}\
     \\begin{document}Text \\gls{used}.\\printglossary\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("\\badone") && !log.contains("\\badtwo"),
    "{log}"
  );
}

/// `multicols` inside `algorithmic` makes no pagination marker the listing cannot hold; the lines are kept. Red (int52a,
/// and Perl): "<ltx:pagination> isn't allowed in <ltx:listing>". Witnesses 1708.03274, 1405.5689. Repro
/// listings-algorithms/multicols_in_algorithmic_keeps_no_markers.
#[test]
fn multicols_in_algorithmic_keeps_no_markers() {
  let (log, xml) = convert_with(
    repro!("listings-algorithms/multicols_in_algorithmic_keeps_no_markers.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<pagination"), "{xml}");
  assert_eq!(xml.matches("<listingline ").count(), 3, "{xml}");
}

/// In the flow the markers stay as before, around the columns and after a heading argument.
#[test]
fn multicols_in_the_flow_keep_their_markers() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{multicol}\\begin{document}A\
     \\begin{multicols}{2}[\\section*{H}]x\n\n\\columnbreak\ny\\end{multicols}B\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section xml:id="Sx1"><title>H</title><para xml:id="Sx1.p1"><pagination role="start_2_columns"/><p xml:id="Sx1.p1.1">x</p></para><para xml:id="Sx1.p2"><p xml:id="Sx1.p2.1">y</p><pagination role="end_2_columns"/><p xml:id="Sx1.p2.2">B</p></para></section>"#,
  );
}

/// The rules are drawn as the table ends, where a last `\\` and a blank line leave no row (tabularray.sty:3722-3731
/// counts a last one-cell row whose text, trimmed of `\par`s, is empty as none): the last row keeps its bottom rule,
/// and a `cmd=` column's cell reads no text past the table. Witness 2607.25933 (`column{1}={cmd=\mbox}`, `hlines`,
/// `vlines` in a `\resizebox`; a 78p draft: "Missing } inserted", the closing rule lost). pdflatex 0 errors.
#[test]
fn tblr_rules_end_a_table_with_a_cmd_column() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{xcolor}\\usepackage{graphicx}\\usepackage{tabularray}\\begin{document}\
     \\resizebox{\\textwidth}{!}{\\begin{tblr}{colspec={Q[l] *{2}{Q[c]}},column{1}={font=\\bfseries,cmd=\\mbox},\
     hlines={gray},vlines={gray}}\nModel & a & b \\\\\n\nOverall & c & d \\\\\n\n\\end{tblr}%\n}\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.1.2\""],
    r#"<tr xml:id="p1.1.1.2"><td align="left" border="b l r t" xml:id="p1.1.1.2.1"><text xml:id="p1.1.1.2.1.1">Overall</text></td><td align="center" border="b r t" xml:id="p1.1.1.2.2">c</td><td align="center" border="b r t" xml:id="p1.1.1.2.3">d</td></tr>"#,
  );
}

/// tabularray draws no hline through a spanned cell (tabularray.sty:2730-2750, the `omit` of the rows it covers), so a
/// row two `\SetCell[r=2]` cells cover whole stays empty and is pruned, as without rules; the rule above "c d" stays.
/// Witness 2502.08151 (its five-column header row; a 78p draft kept an empty `<tr/>` under `rowspan="2"` cells).
/// pdflatex 0 errors.
#[test]
fn tblr_rules_skip_rows_inside_a_span() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{tabularray}\\begin{document}\\begin{tblr}{hlines}\n\
     \\SetCell[r=2]{c} A & \\SetCell[r=2]{c} B \\\\\n & \\\\\nc & d \\\\\n\\end{tblr}\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="left" border="b t" xml:id="p1.1.1.1">A</td><td align="left" border="b t" xml:id="p1.1.1.2">B</td></tr><tr xml:id="p1.1.2"><td align="left" border="b t" xml:id="p1.1.2.1">c</td><td align="left" border="b t" xml:id="p1.1.2.2">d</td></tr></tbody></tabular>"#,
  );
}

/// caption's `\captionof` steps its counter in the current group (caption.sty:390-391), which LaTeXML wraps in a float
/// of its own: a `\label` after it, past a conditional's `\else…\fi`, names that float (pdflatex "Refs: 1 1."), not the
/// section. Witness 2402.05535 (`\ifcoloredproof\captionof{figure}{…}\else\caption{…}\fi\label{…}`, a 78p draft's
/// label on the enclosing item).
#[test]
fn label_after_captionof_names_its_float() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{caption}\\newif\\ifcp\\cptrue\\begin{document}\\section{One}\
     \\begin{center}X\\ifcp\\captionof{figure}{Cap}\\else\\fi\\label{fig:a}\\end{center}\
     \\begin{minipage}{0.4\\textwidth}\\captionof{table}{T}\\ifcp\\fi\\label{tab:b}\\end{minipage}\
     Refs: \\ref{fig:a} \\ref{tab:b}.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<figure align="center" inlist="lof" labels="LABEL:fig:a""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<table inlist="lot" labels="LABEL:tab:b""#),
    "{xml}"
  );
  assert!(!xml.contains(r#"<section labels="#), "{xml}");
}
