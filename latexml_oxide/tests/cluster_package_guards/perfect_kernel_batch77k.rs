//! Red/green guard for batch 77k: sole-cause `unexpected:_` shapes of the projection samples, part 2
//! (agent_reports/sole_underscore_2026-10-19): a caption is a paragraph of a vertical box, pstricks boxes in math are
//! math, a blkarray block's preamble formats its rows, an AASTeX table's `$` is active from its start, and a revtex
//! preprint number is an identifier. The guards assert the three measures: no Error, the authored text present, its
//! element.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Perl's `\@@caption PBoxContents` (latex_constructs.pool.ltxml:3368): the caption is a paragraph set in a vertical
/// box, as amsart sets it (`\vbox{\noindent…}`), so `$$…$$` there is display math (1005.1555, 1907.01498, 2310.11556,
/// 1404.2359). A `\label` after the caption names the figure, also when the caption ends in a display.
#[test]
fn caption_display_math_is_display() {
  let (log, xml) = convert_with(
    "\\documentclass{amsart}\n\\begin{document}\n\\begin{figure}\n\\caption{The ratio for large $i$, $$ 1.60 < \
     b_{u_i}/a_{u_i} < 1.66 $$ and more, so $$ b_{l_i} < 1.48.$$}\\label{fig:r}\n\\end{figure}\nSee \
     Figure~\\ref{fig:r}.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<figure inlist="lof" labels="LABEL:fig:r" xml:id="S0.F1">"#),
    "{xml}"
  );
  let caption = squash(
    r#"</Math>, <inline-block> <equation xml:id="S0.Ex3"> <Math mode="display"
    tex="1.60&lt;b_{u_{i}}/a_{u_{i}}&lt;1.66" text="1.60 &lt; b _ u _ i / a _ u _ i &lt; 1.66" xml:id="S0.Ex3.m1">"#,
  );
  assert!(squash(&xml).contains(&caption), "{xml}");
  assert!(
    squash(&xml).contains(
      r#"</equation> <p>and more, so</p> <equation xml:id="S0.Ex4"> <Math mode="display" tex="b_{l_{i}}&lt;1.48.""#
    ),
    "{xml}"
  );
}

/// An ordinary caption stays one run of text in the caption, its inline math inline.
#[test]
fn caption_without_display_is_one_text_run() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n\\begin{figure}\n\\caption{Plain $x_1$ text.}\\label{f}\n\\end{figure}\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<figure inlist="lof" labels="LABEL:f" xml:id="S0.F1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Figure 1</tag>Plain <Math mode="inline" tex="x_{1}" text="x _ 1" xml:id="S0.F1.m2">"#),
    "{xml}"
  );
  assert!(xml.contains("</Math> text.</caption>"), "{xml}");
  assert!(!xml.contains("<inline-block"), "{xml}");
}

/// pst-code-box.tex:10-17: in math, with `\ifpsmathbox` (the default), pstricks sets a box's material as a formula —
/// an `\rput` body (2603.19804, 0707.0546, 1010.2311) and a `\pscirclebox` (1106.3964); in text it stays text.
#[test]
fn pstricks_box_in_math_is_math() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{pstricks}\n\\begin{document}\n$x \\rput(1,1){V_1}$ and \\[ \
     \\begin{array}{l} \\rput[B](-0.5,0){x_1} \\end{array} \\]\nText $a\\pscirclebox[linewidth=0.3pt]{\\mathcal{D}_t\\kappa} \
     b$ and \\psframebox{text} end.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    squash(&xml).contains(
      r#"<g transform="translate(39.37,39.37)"> <Math mode="inline" tex="\textstyle{V_{1}}" text="V _ 1" xml:id="p1.m1.pic1.m1">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<Math mode="inline" tex="\displaystyle{x_{1}}" text="x _ 1" xml:id="S0.Ex1.m1.pic1.m1">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<Math mode="inline" tex="a{\pscirclebox{\mathcal{D}_t\kappa}}b" text="a * D _ t * kappa * b" xml:id="p1.m2">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text framed="rectangle">text</text> end.</p>"#),
    "{xml}"
  );
  // the formula's `tex` is the author's: the boxes revert to their argument as written
  assert!(
    xml.contains(r#"tex="x\rput(28.45274,28.45274){V_1}""#),
    "{xml}"
  );
}

/// pst-code-box.tex:14 sets the material in a group of its own inside the box's formula (`$\m@th\textstyle{…}$`), so
/// an author's `$…$` in a box called in math is a formula ended inside that group: latex errs ("Missing } inserted",
/// "Extra }, or forgotten $"), and so do we, the letter set as text — not a silent text "q" (review probe p1 D). Our
/// count is larger: the kernel's recovery from "Extra }, or forgotten $" ends the formula where TeX drops the brace.
#[test]
fn pstricks_math_box_keeps_its_group() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{pstricks}\n\\begin{document}\nD: $\\psframebox{$q$}$\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (7, 1), "{log}");
  assert!(
    xml.contains(r#"<p>D: <text class="ltx_markedasmath" framed="rectangle">q</text></p>"#),
    "{xml}"
  );
}

/// blkarray.sty:868-922: a block's preamble formats its rows, `>{$\footnotesize}c<{$}` included, and its delimiters
/// are no columns (1706.05431, 2207.06508, 2210.16910).
#[test]
fn blkarray_block_preamble_formats_its_rows() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{blkarray}\n\\begin{document}\n\\begin{equation}\n\
     \\begin{blockarray}{cc}\n\\begin{block}{*{2}{>{$\\footnotesize}c<{$}}}\n$x_{1}$ & $y_{2}$ \\\\\n\\end{block}\n\
     \\begin{block}{(cc)} 0 & 1 \\\\ \\end{block}\n\\end{blockarray}\n\\end{equation}\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"text="Array[[x _ 1, y _ 2], [0, 1]]""#),
    "{xml}"
  );
  // two rows: the blocks' boundaries add none
  assert_eq!(xml.matches("<XMRow>").count(), 2, "{xml}");
}

/// The control: a plain `{cc}` block in a display keeps math cells, so an author's `$x_{1}$` there leaves math —
/// pdflatex errs too, and so do we.
#[test]
fn blkarray_block_text_cell_in_display_errs() {
  let (log, _xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{blkarray}\n\\begin{document}\n\\begin{equation}\n\\begin{blockarray}{cc}\n\
     \\begin{block}{cc}\n$x_{1}$ & $y_{2}$ \\\\\n\\end{block}\n\\end{blockarray}\n\\end{equation}\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
}

/// aastex701.cls:11298-11299 and :8851-8857: the deluxetable's `$` is active from its start, so the head's `$m_{G}$` in
/// a `C` column is the cell's math (2411.04196), and a hidden `h` cell is math in a box nobody uses (1907.13126).
#[test]
fn aastex_table_dollar_and_hidden_column() {
  let (log, xml) = convert_with(
    "\\documentclass{aastex701}\n\\begin{document}\n\\begin{deluxetable}{cCh}\n\\tablehead{\\colhead{A} & $m_{G}$ & \
     \\colhead{H}}\n\\startdata\n1 & 2 & {grp1.8_diffuse} \\\\\n\\enddata\n\\end{deluxetable}\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<td align="center" border="tt" thead="column"><Math mode="inline" tex="m_{G}" text="m _ G" xml:id="m1">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<td border="b t"/>"#), "{xml}");
  assert!(!xml.contains("diffuse"), "{xml}");
}

/// revtex4-1.cls:2516-2519, aps4-1.rtx:216-221: REVTeX 4.1 prints the `\preprint` numbers only under `\preprint@sw`
/// and never a preamble one; unprinted they are `ltx:rdf` identifiers read as written (2408.13616, 1912.04576,
/// 2110.00437, 1207.6705, and 1205.1463's preamble one), printed a pubnote (1205.1463's body ones).
#[test]
fn revtex_preprint_number_is_an_identifier() {
  let body = "\\preprint{PRE_AMBLE}\n\\begin{document}\n\\preprint{DMRG_Sampled_Variance \\\\ LA-UR-12}\n\\title{T}\\author{Daniel \
              F.}\n\\affiliation{Inst}\\begin{abstract}Abs\\end{abstract}\\maketitle\nText.\n\\end{document}";
  let (log, xml) = convert_with(
    &format!("\\documentclass[aps,prl,reprint]{{revtex4-1}}\n{body}"),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    squash(&xml).contains(
      r#"<rdf property="dcterms:identifier">PRE_AMBLE</rdf> <rdf property="dcterms:identifier">DMRG_Sampled_Variance</rdf> <rdf property="dcterms:identifier">LA-UR-12</rdf>"#
    ),
    "{xml}"
  );
  assert!(!xml.contains("<pubnote"), "{xml}");

  let (log, xml) = convert_with(
    "\\documentclass[aps,prl,preprint]{revtex4-1}\n\\preprint{PRE_AMBLE}\n\\begin{document}\n\\preprint{CERN-PH-TH \
     \\\\ LA-UR-12}\n\\title{T}\\author{A}\\maketitle\nText.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      "<pubnote name=\"Preprint:\u{a0}\" role=\"preprint\">CERN-PH-TH <break/>LA-UR-12</pubnote>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<rdf property="dcterms:identifier">PRE_AMBLE</rdf>"#),
    "{xml}"
  );
}

/// A printed number is read as TeX reads it, Perl's `{}` argument: its math is math and `~` a tie, as pdflatex prints
/// "Fe2YZ" and "DESY 11–210" (review probe r6).
#[test]
fn revtex_printed_preprint_is_read_as_tex() {
  let (log, xml) = convert_with(
    "\\documentclass[aps,prb,preprint]{revtex4-1}\n\\begin{document}\n\\preprint{Gasi \\textit{et al.}, Fe$_{2}$YZ}\n\
     \\preprint{DESY~11--210}\n\\preprint{$\\mathrm{UCB}$-PTH}\n\\title{T}\\author{A}\\maketitle\nText.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      "<pubnote name=\"Preprint:\u{a0}\" role=\"preprint\">Gasi <text font=\"italic\">et al.</text>, \
       Fe<sub><text font=\"italic\">2</text></sub>YZ</pubnote>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains("<pubnote name=\"Preprint:\u{a0}\" role=\"preprint\">DESY\u{a0}11–210</pubnote>"),
    "{xml}"
  );
  assert!(
    squash(&xml).contains(
      r#"<pubnote name="Preprint: " role="preprint"><Math mode="inline" tex="\mathrm{UCB}" text="UCB" xml:id="m2"> <XMath> <XMTok role="UNKNOWN">UCB</XMTok> </XMath> </Math>-PTH</pubnote>"#
    ),
    "{xml}"
  );
  assert!(!xml.contains("<rdf"), "{xml}");
}

/// An unprinted number is split only at its top-level `\\`: a `\\` in a group or an environment is the block's own
/// (FERMILAB's `{minipage}`/`{flushright}` layout, a `\parbox`), review probes r4 and r5.
#[test]
fn revtex_unprinted_preprint_splits_only_top_level_lines() {
  let (log, xml) = convert_with(
    "\\documentclass[aps,prl,twocolumn]{revtex4-1}\n\\begin{document}\n\\preprint{\n\\hfill\n\\begin{minipage}[t]{3in}\n\
     \\begin{flushright}\nFERMILAB--PUB--07--666--A\\\\\n\\end{flushright}\n\\end{minipage}\n}\n\
     \\preprint{\\parbox{2in}{A-1\\\\B-2}}\n\\title{T}\\author{A}\\maketitle\nText.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<rdf property="dcterms:identifier"><p align="right" class="ltx_minipage" vattach="top" width="216.8pt">FERMILAB–PUB–07–666–A</p></rdf>"#
    ),
    "{xml}"
  );
  assert!(
    squash(&xml).contains(
      r#"<rdf property="dcterms:identifier"><inline-block class="ltx_parbox" vattach="middle" width="144.5pt"> <p>A-1</p> <p>B-2</p> </inline-block></rdf>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<rdf ").count(), 2, "{xml}");
}

/// REVTeX 3.1's aps.sty:965-968 defines `\@preprint` only under `\ifpreprintsty` and `\@maketitle` prints it (:1010), with
/// no reset at the document's start: in the preprint style a `\preprint` is printed wherever it is called, otherwise it
/// is metadata (review probes r7, r8; 305 of 8,247 REVTeX 3 papers of 9601-0001).
#[test]
fn revtex3_preprint_style_prints_the_number() {
  for (head, preamble, number) in [
    (
      "\\documentstyle[preprint,aps]{revtex}",
      "",
      "\\preprint{UCB-PTH-95/01}",
    ),
    (
      "\\documentclass[preprint,aps]{revtex}",
      "\\preprint{UCB-PTH-95/01}",
      "",
    ),
  ] {
    let (log, xml) = convert_with(
      &format!(
        "{head}\n{preamble}\n\\begin{{document}}\n{number}\n\\title{{T}}\\author{{A}}\\maketitle\nText.\n\\end{{document}}"
      ),
      None,
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{head}: {log}"
    );
    assert!(
      xml.contains("<pubnote name=\"Preprint:\u{a0}\" role=\"preprint\">UCB-PTH-95/01</pubnote>"),
      "{head}: {xml}"
    );
    assert!(!xml.contains("<rdf"), "{head}: {xml}");
  }
  let (log, xml) = convert_with(
    "\\documentstyle[aps]{revtex}\n\\begin{document}\n\\preprint{UCB_PTH-95/02}\n\\title{T}\\author{A}\\maketitle\nText.\n\
     \\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<rdf property="dcterms:identifier">UCB_PTH-95/02</rdf>"#),
    "{xml}"
  );
  assert!(!xml.contains("<pubnote"), "{xml}");
}

/// REVTeX 4.0 empties the store once at load (revtex4.cls:4618-4619), so a preamble `\preprint` is printed under
/// `preprintnumbers`, and its `preprint` style forces `\preprint@sw` on after the options (:3811-3812), so
/// `nopreprintnumbers` does not hide it (review probe r3); otherwise it is metadata (1207.6705).
#[test]
fn revtex40_preprint_numbers_follow_the_class() {
  for (options, preamble, number) in [
    ("aps,prl,preprintnumbers", "\\preprint{PRE-2004/01}", ""),
    (
      "aps,prl,preprint,nopreprintnumbers",
      "",
      "\\preprint{PRE-2004/01}",
    ),
  ] {
    let (log, xml) = convert_with(
      &format!(
        "\\documentclass[{options}]{{revtex4}}\n{preamble}\n\\begin{{document}}\n{number}\n\\title{{T}}\\author{{A}}\\maketitle\n\
         Text.\n\\end{{document}}"
      ),
      None,
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{options}: {log}"
    );
    assert!(
      xml.contains("<pubnote name=\"Preprint:\u{a0}\" role=\"preprint\">PRE-2004/01</pubnote>"),
      "{options}: {xml}"
    );
    assert!(!xml.contains("<rdf"), "{options}: {xml}");
  }
  let (log, xml) = convert_with(
    "\\documentclass[aps,prb,twocolumn]{revtex4}\n\\begin{document}\n\\preprint{22241-LDA_v1}\n\\title{T}\\author{A}\
     \\maketitle\nText.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<rdf property="dcterms:identifier">22241-LDA_v1</rdf>"#),
    "{xml}"
  );
}
