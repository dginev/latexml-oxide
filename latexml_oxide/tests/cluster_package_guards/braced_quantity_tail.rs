//! What follows the value in a braced typed argument (`{Dimension}`, `{Glue}`,
//! `{Number}`, `[Dimension]`) stays in the input, as TeX has it: latex.ltx hands
//! these arguments to `\setlength#1#2{#1 #2\relax}` (10253) and its siblings,
//! whose scan leaves the rest to be read next. Perl's `readingFromMouth` dropped
//! it (KNOWN_PERL_ERRORS #275). Assignments scan by the register's type, and
//! with calc a length is evaluated whole (calc.sty:51-56, 86). OXIDIZED_DESIGN
//! #317. Repros in `tools/perfect_kernel/repros/expansion-primitives/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

fn assert_para(xml: &str, id: &str, expected_p: &str) {
  assert_element(
    xml,
    "para",
    &[&format!("xml:id=\"{id}\"")],
    &format!("<para xml:id=\"{id}\">{expected_p}</para>"),
  );
}

/// `\setlength`, `\addtolength`, `\setcounter` and `\addtocounter` read their
/// value as TeX's `#1 #2\relax` does: the tail is typeset after the assignment
/// (pdflatex "xG", "yH", "zY5", "wW6"), and a conditional the scan cut closes
/// after its taken branch's tail ("xV[3.0pt]"). All were dropped before.
#[test]
fn assignment_tail_follows_the_assignment() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/braced_value_tail_after_assignment.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "p1", "<p>xG</p>");
  assert_para(&xml, "p2", "<p>yH</p>");
  assert_para(&xml, "p3", "<p>zY5</p>");
  assert_para(&xml, "p4", "<p>wW6</p>");
  assert_para(&xml, "p5", "<p>xV[3.0pt]</p>");
  // The drain stops at the `\fi`: a dimen scan ends at `3\U` (tex.web §455), so
  // `\the\xd` is the assigned 6pt; a glue scan's `plus` look-ahead (§461) reads
  // the old 5pt first — both pdflatex's.
  assert_para(&xml, "p6", "<p>6.0ptD</p>");
  assert_para(&xml, "p7", "<p>5.0ptS</p>");
}

/// A skip register keeps its stretch and shrink through `\setlength` and
/// `\addtolength`, `\hspace` reads a skip, and a `\dimen` register's `plus` is
/// text, as in pdflatex.
#[test]
fn skip_lengths_keep_stretch() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/braced_length_skip_keeps_stretch.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "p1", "<p>[1.0pt plus 2.0pt minus 3.0pt]</p>");
  assert_para(&xml, "p2", "<p>[3.0pt plus 1.0pt minus 1.0pt]</p>");
  assert_para(&xml, "p3", "<p>[2.0pt plus 1.0fil minus 3.0pt]</p>");
  assert_para(&xml, "p4", "<p>ABCD</p>");
  assert_para(&xml, "p5", "<p>plus 1pt[0.0pt]</p>");
}

/// A box or space command's tail is kept, read right after the command (TeX
/// typesets it just before the box: "Ax B", "xMN"), with one warning each.
#[test]
fn box_command_tail_follows_the_box() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/braced_length_tail_box_commands.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 3, "{stderr}");
  assert_eq!(
    stderr
      .matches("Unexpected text after the value in an argument of")
      .count(),
    3,
    "{stderr}"
  );
  // The 1em space is an EM SPACE (`dimension_to_spaces`).
  assert_para(&xml, "p1", "<p>A\u{2003}xB</p>");
  assert_para(
    &xml,
    "p2",
    "<p><text align=\"center\" width=\"10.0pt\">M</text>xN</p>",
  );
  assert_para(&xml, "p3", "<p>xZ</p>");
}

/// With calc, a braced length is one expression: the `\makebox` width is
/// `\widthof{ab}+\widthof{cdefgh}` (pdflatex 38.6pt), not its first term.
#[test]
fn calc_evaluates_a_braced_length() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/calc_braced_length_expression.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "p1", "<p>[38.6112pt]</p>");
  assert_para(
    &xml,
    "p2",
    "<p><text align=\"center\" width=\"38.6pt\">M</text>N</p>",
  );
  assert_para(&xml, "p3", "<p>[7.0pt]</p>");
  assert_para(&xml, "p4", "<p>[1.0pt plus 2.0pt]</p>");
  assert_para(&xml, "p5", "<p>[3.0pt plus 1.0pt minus 1.0pt]</p>");
  assert_para(&xml, "p6", "<p>[7]</p>");
}

/// calc reports the token it cannot parse after a term and consumes it
/// (calc.sty:281-284; pdflatex 3 errors, no `x` typeset).
#[test]
fn calc_reports_an_unparsable_tail() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/calc_braced_length_invalid_tail.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("`x' invalid at this point"), "{stderr}");
  assert_para(&xml, "p1", "<p>A\u{2003}B</p>");
  assert_para(&xml, "p2", "<p>G</p>");
  assert_para(&xml, "p3", "<p>Y</p>");
}

/// A picture length is `\@defaultunitsset`'s `\dimexpr<arg>\unitlength` with the
/// rest discarded (latex.ltx:16766-16767): `1cm` and `.5\linewidth` are lengths,
/// `10pt` a circle diameter, and nothing of them is typeset. The `{Float}` read
/// took each for a number of units.
#[test]
fn picture_lengths_are_default_units() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/picture_length_default_units.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // `\line(1,0){10}` and `\line(1,0){1cm}` at 1mm per unit.
  let line = r##"<line points="0,0 39.44,0" stroke="#000000" stroke-width="0.4"/>"##;
  assert_eq!(xml.matches(line).count(), 2, "{xml}");
  assert_element(
    &xml,
    "line",
    &[r#"terminators="-&gt;""#],
    r##"<line points="0,0 239.09,0" stroke="#000000" stroke-width="0.4" terminators="-&gt;"/>"##,
  );
  assert_element(
    &xml,
    "circle",
    &[r#"r="6.93""#],
    r##"<circle fill="none" r="6.93" stroke="#000000" stroke-width="0.4" x="0" y="0"/>"##,
  );
}

/// floatflt's `{floatingtable}` argument is the table (floatflt.sty:131-139), not a
/// width: its cells reach the float, which is as wide as the table.
#[test]
fn floatingtable_keeps_its_table() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/floatingtable_table_argument.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r#"<table float="right" inlist="lot" width="20%" xml:id="S0.T1">
    <tags>
      <tag>Table 1</tag>
      <tag role="refnum">1</tag>
      <tag role="typerefnum">Table 1</tag>
    </tags>
    <tabular vattach="middle">
      <tbody>
        <tr>
          <td align="center">Alpha</td>
          <td align="center">Beta</td>
        </tr>
      </tbody>
    </tabular>
    <toccaption><tag close=" ">1</tag>Cap</toccaption>
    <caption><tag close=": ">Table 1</tag>Cap</caption>
  </table>"#,
  );
}

/// A `\newlength` is a skip, so `\setlength` now stores glue in it; epigraph's
/// binding reads `\epigraphwidth` and `\epigraphrule` at their natural width
/// (it matched a stored dimension only, and fell back to 0pt and 0.4pt).
#[test]
fn epigraph_reads_its_skip_lengths() {
  let tex = "\\documentclass{article}\n\\usepackage{epigraph}\n\
             \\setlength{\\epigraphwidth}{0.6\\textwidth}\n\\setlength{\\epigraphrule}{0pt}\n\
             \\begin{document}\n\\epigraph{Quote text}{Source}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "quote",
    &[r#"class="ltx_epigraph""#],
    r#"<quote class="ltx_epigraph" cssstyle="width:207pt; margin-left:auto;">
      <block class="ltx_epigraph_text" cssstyle="text-align:left; ">
        <p><text fontsize="90%">Quote text</text></p>
      </block>
      <block class="ltx_epigraph_source" cssstyle="border-top:solid 0pt; text-align:right; ">
        <p><text fontsize="90%">Source</text></p>
      </block>
    </quote>"#,
  );
}

/// hyperref's `\hypercalcbp` evaluates `\dimexpr(#1)\relax` (hyperref.sty:364-366):
/// `1in+1in` is 144bp and `\textwidth-2cm` 287bp (pdflatex 143.99962, 287.01747),
/// where the `{Dimension}` read took the first term alone.
#[test]
fn hypercalcbp_evaluates_its_expression() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\begin{document}\n\
             [\\hypercalcbp{1in+1in}] [\\hypercalcbp{\\textwidth-2cm}] [\\hypercalcbp{72.27pt}]\n\
             \\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>[143.99997810943339] [287.0181795075109] [72.00000425649907]</p></para>",
  );
}

/// pgf 3.1's `\pgfsys@declarepattern` has fifteen arguments, a transformation
/// matrix before the code (pgfcorepatterns.code.tex:159-164). Read with Perl's
/// nine, the matrix's `{1.0}` was the code and `{0.0}` the flag, whose `.0`
/// then re-entered the input; before that, the unread rest ran the pattern
/// code after the still-open `<svg:pattern>`, so everything drawn after it —
/// the `Label` node here — landed inside the invisible `<svg:defs>`. The filled
/// path takes the pattern as its fill (Perl's shape; the per-path colour group
/// painted it solid black).
#[test]
fn pgf_pattern_declares_its_code_and_nothing_else() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\usetikzlibrary{patterns}\n\
             \\begin{document}\n\\begin{tikzpicture}\\fill[pattern=north east lines] (0,0) \
             rectangle (1,1);\\node at (2,0) {Label};\\draw (0,0)--(3,0);\\end{tikzpicture}\n\
             \\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The first `<svg:defs>` is the pattern's, and holds the pattern alone.
  assert_element(
    &xml,
    "svg:defs",
    &[],
    r#"<svg:defs>
            <svg:pattern height="4" id="pgfpat3" patternUnits="userSpaceOnUse" width="4">
              <svg:symbol id="pgfsym3">
                <svg:g stroke-width="0.4pt">
                  <svg:path d="M 0 0 L 4.29 4.29" style="fill:none"/>
                </svg:g>
                <svg:g/>
              </svg:symbol>
            </svg:pattern>
          </svg:defs>"#,
  );
  assert_element(
    &xml,
    "svg:g",
    &[r#"fill="url(#pgfupat1)""#],
    r#"<svg:g fill="url(#pgfupat1)">
            <svg:path d="M 0 0 M 0 0 L 0 39.37 L 39.37 39.37 L 39.37 0 Z M 39.37 39.37" style="stroke:none"/>
          </svg:g>"#,
  );
  assert_element(
    &xml,
    "svg:foreignObject",
    &[],
    r#"<svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:2.4em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="33.25">Label</svg:foreignObject>"#,
  );
}

/// picture.sty takes calc's error over to accept a length before `\unitlength`
/// (picture.sty:79-120); calc reports only while `\calc@error` is its own. The
/// circles are sized by their lengths (circledsteps' `\circle{\csteps@YLength}`,
/// a 4mm `\circle`). Witnesses 2605.09094, 2605.10684.
#[test]
fn calc_error_redefinition_owns_the_recovery() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/calc_error_redefined_picture_sty.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // circledsteps' own "Failed to patch" warning (its `\patchcmd` of pict2e's
  // `\@oval`, which the pict2e binding does not define).
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Failed to patch either picture.sty or pict2e.sty"),
    "{stderr}"
  );
  for r in ["7.23", "7.87"] {
    assert_element(
      &xml,
      "circle",
      &[&format!("r=\"{r}\"")],
      &format!(
        "<circle fill=\"none\" r=\"{r}\" stroke=\"#000000\" stroke-width=\"0.4\" x=\"0\" y=\"0\"/>"
      ),
    );
  }
}

/// A column type's tail is dropped with a warning, not read as more column
/// letters: `p{\textwidth-1pt}c` stays two columns (TeX typesets the `-1pt` in
/// every cell of the first; with calc it is part of the width).
#[test]
fn column_type_tail_is_not_a_column() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\
             \\begin{tabular}{p{\\textwidth-1pt}c}\na & b\\\\\n\\end{tabular}\n\
             \\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("('-1pt'); it is dropped (TeX typesets it in every cell of the column)"),
    "{stderr}"
  );
  assert_eq!(xml.matches("<td ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "td",
    &[r#"align="center""#],
    r#"<td align="center">b</td>"#,
  );
}

/// An undefined control sequence the scan met was reported and stubbed when it
/// was expanded; TeX discards it (tex.web §370), so it neither comes back as
/// `<ERROR>` text nor, with calc, as "invalid at this point". Witnesses
/// 2605.25073 (`\hspace{6\@p@t}`, USG.cls missing), 2605.08378.
#[test]
fn undefined_cs_in_a_length_is_discarded() {
  for repro in [
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/braced_length_undefined_cs.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/calc_braced_length_undefined_cs.tex"
    ),
  ] {
    let (stderr, xml) = convert(repro, true);
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(
      stderr.contains("Error:undefined:\\undefinedlen"),
      "{stderr}"
    );
    assert_eq!(warning_count(&stderr), 2, "{stderr}");
    assert_para(&xml, "p1", "<p>CD</p>");
    assert_para(&xml, "p2", "<p>EF[0.0pt]</p>");
  }
}

/// Between alignment rows a command's argument rest has no place: booktabs'
/// unused widths are read untyped, and `\cmidrule[lr]{1-2} \cmidrule[lr]{3-4}`
/// (a typo for `(lr)`) keeps both rules and puts nothing in the next row
/// (2605.27476, 2605.25272); `\addlinespace[2pt plus 1pt]` neither.
#[test]
fn between_rows_tail_opens_no_row() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/braced_length_tail_between_rows.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr>").count(), 3, "{xml}");
  assert_element(
    &xml,
    "td",
    &[r#"border="t""#],
    r#"<td align="left" border="t">e</td>"#,
  );
  assert_element(
    &xml,
    "td",
    &[r#"border="bb""#],
    r#"<td align="left" border="bb">i</td>"#,
  );
  assert_eq!(xml.matches(r#"border="t""#).count(), 4, "{xml}");
}

/// `\setcounter` of an undefined counter goes to `\@nocounterr` without
/// reading the value (latex.ltx:10115-10122): pdflatex "AB C", no `x`.
#[test]
fn undefined_counter_reads_no_value() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/setcounter_undefined_counter_value.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "p1", "<p>AB C</p>");
}

/// A `fil` glue reverts with a space after it, so the `l` after
/// `\hspace{\stretch{1}}` is not read as another `l` of the unit (tex.web §454).
#[test]
fn fil_glue_reversion_ends_its_unit() {
  let tex =
    "\\documentclass{article}\n\\begin{document}\n$a\\hspace{\\stretch{1}}l$\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="a\hskip 0.0pt plus 1.0fill l" text="a * l" xml:id="p1.m1">
        <XMath>
          <XMApp>
            <XMTok meaning="times" role="MULOP">⁢</XMTok>
            <XMTok font="italic" role="UNKNOWN">a</XMTok>
            <XMTok font="italic" role="UNKNOWN">l</XMTok>
          </XMApp>
        </XMath>
      </Math>"#,
  );
}

/// Package registers the `.sty` declares with `\newdimen` are dimens, with the
/// package's initial values (lineno's `\linenumbersep` and
/// `\quotelinenumbersep` are 10pt, floatflt's `\htdone` 0pt): as counts, the
/// assignment's tail printed ".5pt" (arXiv 2605.07149). pdflatex's output.
#[test]
fn package_registers_are_dimens() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/package_registers_are_dimens.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "p1", "<p>[10.0pt][10.0pt][0.0pt]</p>");
  assert_para(&xml, "p2", "<p>[2.5pt][1.5pt]</p>");
}

/// Self-skip helper: is this file in the host TeX tree?
fn kpsewhich_has(name: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// multido's Number variable starts as written and steps in fixed point
/// (multido.tex:193-197, 215-283): `\n=1+1` is `1`, whose calc `\y*\n` has no
/// `.0` to report; `2.00+-3.05` keeps two decimals. pdflatex's output
/// (bardiag, bardiag2: 10 and 4 calc errors in 56jr).
#[test]
fn multido_number_variable_starts_as_written() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/multido_number_variable_as_written.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "p1", "<p>[1:1.0pt][2:2.0pt][3:3.0pt]</p>");
  assert_para(&xml, "p2", "<p>[2.00][-1.05][-4.10]</p>");
  // A space before the list's end is no decimal.
  assert_para(&xml, "p3", "<p>[0][0.5][1.0]</p>");
}

/// pstricks reads an angle argument whole (pstricks.tex:990-999): a
/// coordinate is the angle of its vector, a node's and PostScript code are
/// unresolved, and none of the argument is typeset (pst-eucl-docBG's
/// `\pstMarkAngle`: `(F)(A_1)` and a "Script _" error in 56jr).
#[test]
fn pstricks_angle_argument_is_read_whole() {
  if !kpsewhich_has("pstricks.sty") {
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/pstricks_angle_argument_read_whole.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("The PostScript length '! 0.5 2 mul' is not evaluated"),
    "{stderr}"
  );
  // The (1,1) and (0,1) angles are 45 and 90 degrees, `\diag` expands to
  // (1,1); the arcs at a node's and a PostScript angle are not drawn, and the
  // PostScript radius is 0.
  assert_para(
    &xml,
    "p1",
    r#"<picture fill="none" height="113.81pt" stroke="none" unitlength="28.45pt" width="113.81pt" xml:id="p1.pic1">
      <arc angle1="45" angle2="90" fill="none" r="39.37" stroke="black" stroke-width="0.8" x="0" y="0"/>
      <arc angle1="45" angle2="180" fill="none" r="19.69" stroke="black" stroke-width="0.8" x="0" y="0"/>
      <circle fill="none" r="0" stroke="black" stroke-width="0.8" x="39.37" y="39.37"/>
    </picture><p>Z</p>"#,
  );
}

/// nicematrix's `X[<keys>]` column takes its keys, `c` centring the cells
/// (nicematrix.sty:2788-2826); read as columns, `m]` was an `m` column of
/// width `]` (nicematrix manuals: 5 calc errors each in 56jr).
#[test]
fn nicematrix_x_column_takes_its_keys() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/nicematrix_x_column_keys.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr>
      <td align="left"><inline-block vattach="top"><p align="center">a rather long text</p></inline-block></td>
      <td align="left"><inline-block vattach="top"><p align="center">another text</p></inline-block></td>
    </tr>"#,
  );
  // `X[r]` right-aligns its cell (`\raggedleft`); `X[m]` and `X` justify.
  assert_element(
    &xml,
    "p",
    &[r#"class="ltx_align_right""#],
    r#"<p class="ltx_align_right">B</p>"#,
  );
  assert_eq!(xml.matches("ltx_align_right").count(), 1, "{xml}");
}

/// tabularray's `\NewColumnType` defines a column type whose body, its
/// arguments put in, is more colspec (tabularray.sty:3291-3334): `Y` is its
/// default `Q[c]`. A no-op, the spec became the template and a `b` read `l`
/// as its width (non-decimal-units: 11 -> 15 errors in 56jr).
#[test]
fn tabularray_new_column_type_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/tabularray_new_column_type.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr>
      <td align="right">a</td>
      <td align="right" border="r">1.2.3</td>
      <td align="center">x</td>
      <td align="left">u</td>
    </tr>"#,
  );
}

/// tabularray's rule options `|[1pt]`, `|[dashed]` and its `j`/`t{…}` columns
/// are part of the colspec (tabularray.sty:3172-3181, 3336-3346); bailing on
/// them made the spec the template (logoetalab-doc: 0 -> 2 errors in 56jr).
#[test]
fn tabularray_rule_options_are_colspec() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/tabularray_rule_options_colspec.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr>
      <td align="left" border="l r" thead="row">alpha</td>
      <td align="left" vattach="top"><inline-block vattach="top" width="56.9pt"><p>beta</p></inline-block></td>
    </tr>"#,
  );
}

/// With calc, `\resizebox`'s height is one expression (graphics.sty:555-568
/// `\setlength`): `\ht\bx+\dp\bx` scales B to 8.8pt, its width in
/// proportion (`!`), and nothing of it is typeset (perfectcut: 4 -> 134
/// warnings in 56jr).
#[test]
fn calc_evaluates_a_resizebox_length() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/calc_resizebox_length_expression.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(
    &xml,
    "p1",
    r#"<p>A<inline-block depth="0.0pt" height="8.8pt" width="9.1pt" xscale="1.28455344462606" xtranslate="1.0pt" yscale="1.28455344462606" ytranslate="-1.0pt"><p>B</p></inline-block>C</p>"#,
  );
  assert_para(
    &xml,
    "p2",
    r#"<p>D<inline-block depth="0.0pt" height="6.8pt" width="6.8pt" xscale="1" xtranslate="0.0pt" yscale="1" ytranslate="0.0pt"><p>E</p></inline-block>F</p>"#,
  );
}

/// `\DeclareTextAccent` keeps its slot argument in the command it defines
/// (latex.ltx:9903-9904): nothing after the slot comes back as input
/// (amsldoc-vi: 2 tail warnings in 56jr).
#[test]
fn declare_text_accent_keeps_its_slot() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/declare_text_accent_slot_stays_stored.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    !stderr.contains("Unexpected text after the value"),
    "{stderr}"
  );
  // The one warning: the slot is not a number (read at declaration, as Perl).
  assert!(
    stderr.contains("Missing number, treated as zero"),
    "{stderr}"
  );
  assert_para(&xml, "p1", "<p>X</p>");
}

/// pdfcomment loads calc and ifthen (pdfcomment.sty:1345-1346): the minipage
/// is `\linewidth-2\fboxsep` wide, nothing of it typeset, and `\ifthenelse`
/// is defined (dataref-doc: 6 -> 58 warnings in 56jr).
#[test]
fn pdfcomment_loads_calc_and_ifthen() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/pdfcomment_loads_calc_and_ifthen.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para class="ltx_minipage" vattach="middle" width="339.0pt" xml:id="p1"><p>A</p></para>"#,
  );
  assert_para(&xml, "p2", "<p>B</p>");
  assert_para(&xml, "p3", "<p>C</p>");
}

/// A binding loads calc where its package does (diagbox.sty:26, animate.sty:23,
/// savetrees.sty:194 by default, breqn.sty:56, jmlr.cls:46): a document's
/// `\linewidth-2cm` is one expression, whose `-2cm` 56jr typeset otherwise.
#[test]
fn bindings_load_calc_where_their_packages_do() {
  let body =
    "\\begin{document}\n\\begin{minipage}{\\linewidth-2cm}A\\end{minipage}B\n\\end{document}\n";
  let minipage =
    r#"<para class="ltx_minipage" vattach="middle" width="288.1pt" xml:id="p1"><p>A</p></para>"#;
  for (preamble, warnings) in [
    ("\\documentclass{article}\\usepackage{diagbox}", 0),
    ("\\documentclass{article}\\usepackage{animate}", 0),
    ("\\documentclass{article}\\usepackage{savetrees}", 0),
    // The stub's own "breqn.sty is not implemented" warning.
    ("\\documentclass{article}\\usepackage{breqn}", 1),
    ("\\documentclass{jmlr}", 0),
  ] {
    let (stderr, xml) = convert(&format!("{preamble}\n{body}"), true);
    assert_eq!(error_count(&stderr), 0, "{preamble}\n{stderr}");
    assert_eq!(warning_count(&stderr), warnings, "{preamble}\n{stderr}");
    assert_element(&xml, "para", &[r#"xml:id="p1""#], minipage);
    assert_para(&xml, "p2", "<p>B</p>");
  }
}
