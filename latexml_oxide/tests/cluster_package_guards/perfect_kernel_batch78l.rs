//! Red/green guards for batch 78l (quality batch 12): `\textcircled` of a box circles the text the box typesets, not
//! its TeX source; tabularray's `\SetCell[c=<n>,r=<m>]` spans columns and rows; `\caption*` takes caption's fonts.
//! Each guard converts its repro under `tools/perfect_kernel/repros/` and asserts no Error or Warning and the
//! authored content in whole elements, against pdflatex. Probes tcirc.tex, tblr.tex, capstar.tex (sel78k).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\textcircled{\raisebox{-0.9pt}{1}}` is a circled 1 (latex.ltx:10057, a text accent over the typeset argument):
/// the circle holds the text of a box's one content argument and of a plain formula's body (sect13.rs
/// `typeset_text`), never the whatsit's reversion; a `\scalebox{.8}{3}`, whose factor is a box argument too, keeps
/// its source (78l r2, the fail-safe for a whatsit with two box arguments). Red (int49a, and Perl):
/// "\raisebox{-0.9pt}{1}⃝ and $x$⃝, \scalebox{.8}{3}⃝, \makebox[4.30554pt]{5}⃝". KNOWN_PERL_ERRORS #361. Repro
/// fonts-nfss/textcircled_of_a_box_circles_its_text.
#[test]
fn textcircled_of_a_box_circles_its_text() {
  let (log, xml) = convert_with(
    repro!("fonts-nfss/textcircled_of_a_box_circles_its_text.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">① and ⓧ, \\scalebox{.8}{3}⃝, ⑤.</p>",
  );
}

/// A `\label` beside the digit typesets nothing of the circle, and an inner font command circles its text
/// (pdflatex 0 errors: a circled 1 and a circled b).
#[test]
fn textcircled_keeps_keys_out_of_the_circle() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\begin{document}\\section{S}\\label{s}A \\textcircled{\\label{q}1}, B \
     \\textcircled{\\textit{b}}.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"S1.p1.1\">A ①, B ⓑ.</p>");
}

/// The spacing around the circled text is the circle's layout, not its text, and a `\phantom` pads without text
/// (`isSpace`): `\textcircled{\raisebox{-.8pt}{{\hspace{-1.1mm} \small 1}}}` (2501.14912), `\textcircled{\phantom{0}
/// \makebox[0pt][c]{2}\phantom{0}}` (2501.11185), `\textcircled{\hspace{-0.08cm} \raisebox{-.7pt}{3}}` (2505.12250).
/// pdflatex 0 errors: three circled digits. A 78l draft gave "1⃝", "020⃝" and "3⃝".
#[test]
fn textcircled_ignores_spacing_and_phantoms() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\begin{document}A \\textcircled{\\raisebox{-.8pt}{{\\hspace{-1.1mm} \\small 1}}}, B \
     \\textcircled{\\phantom{0}\\makebox[0pt][c]{2}\\phantom{0}}, C \\textcircled{\\hspace{-0.08cm} \
     \\raisebox{-.7pt}{3}}.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">A ①, B ②, C ③.</p>");
}

/// A formula with structure has no text the circle can hold: a script, a fraction or a root inside it makes
/// `typeset_text` give up and the circle keeps the argument's source, as int49a and Perl (KNOWN_PERL_ERRORS #361);
/// a plain formula is circled (Ⓒ). Witness 2508.14614 (`\textcircled{$q_0$}`). Red (78l r1): "②, ②, q0⃝, Ⓒ", the
/// fraction, the root and the subscript lost. pdflatex 0 errors. Repro
/// fonts-nfss/textcircled_of_a_formula_keeps_its_source.
#[test]
fn textcircled_of_a_formula_keeps_its_source() {
  let (log, xml) = convert_with(
    repro!("fonts-nfss/textcircled_of_a_formula_keeps_its_source.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">$\\frac{1}{2}$⃝, $\\sqrt{2}$⃝, $q_{0}$⃝, Ⓒ.</p>",
  );
}

/// A rule typesets no text (`\strut`'s `\vrule`, `\rule{1pt}{1pt}`: dimensions only), and the circled numbers go
/// to 50 (U+3251-325F, U+32B1-32BF; Perl's table stops at 20). pdflatex 0 errors. Red (78l r1): "\vruleheight=8.39996pt,
/// depth=3.60004pt,width=0.0pt2⃝", "\rule{1.0pt}{1.0pt}⃝", "25⃝", "42⃝".
#[test]
fn textcircled_rules_show_nothing_and_numbers_reach_fifty() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\begin{document}A \\textcircled{\\strut 2}, B \\textcircled{\\rule{1pt}{1pt}}, C \
     \\textcircled{25}, D \\textcircled{42}, E \\textcircled{51}.\\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">A ②, B ⃝, C ㉕, D ㊷, E 51⃝.</p>",
  );
}

/// tabularray's `\SetCell[c=2]{c}` spans two columns over the empty cell after it, at a row's start or after a cell,
/// and `\SetCell[r=2]{c}` two rows (tabularray.sty:2542-2615; the binding's body pass `tblr_span_cells`); a row that
/// ends before its covered cells (`\SetCell[c=3]{c} All\\`) still spans them, as in 2305.00635. Red (int49a): no `colspan`/`rowspan`, a stray `<td/>` after "Wide". Repro
/// alignment-bindings/tblr_setcell_spans_columns_and_rows.
#[test]
fn tblr_setcell_spans_columns_and_rows() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/tblr_setcell_spans_columns_and_rows.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // the rules (`hlines,vlines`) as cell borders (78p); the kernel guesses the last row a footer, as a `tabular`'s
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" border="l r t" colspan="2" thead="row" xml:id="p1.1.1.1">Wide</td><td align="center" border="r t" xml:id="p1.1.1.2">C</td></tr><tr xml:id="p1.1.2"><td align="center" border="l r t" rowspan="2" thead="row" xml:id="p1.1.2.1">Tall</td><td align="center" border="r t" xml:id="p1.1.2.2">B2</td><td align="center" border="r t" xml:id="p1.1.2.3">C2</td></tr><tr xml:id="p1.1.3"><td align="center" border="r t" xml:id="p1.1.3.1">B3</td><td align="center" border="r t" xml:id="p1.1.3.2">C3</td></tr><tr xml:id="p1.1.4"><td align="center" border="l r t" thead="row" xml:id="p1.1.4.1">A4</td><td align="center" border="r t" colspan="2" xml:id="p1.1.4.2">Both</td></tr></tbody><tfoot><tr xml:id="p1.1.5"><td align="center" border="b l r t" colspan="3" thead="row" xml:id="p1.1.5.1">All</td></tr></tfoot></tabular>"#,
  );
}

/// A `\SetCell` after a row-level table command still starts its cell (tabularray's extractor takes every table
/// command at a cell's start, tabularray.sty:3775-3860): after `\hline`, `\hline[1pt]`, `\cline{1-2}`,
/// `\SetRow{gray9}` and a row end's `*`. Witnesses 2308.16907 (11 `\hline \SetCell`), 2507.01654, 2305.00635,
/// 2502.18449, 2404.12789, 2601.16161, 2604.10925, 2606.09108. pdflatex 0 errors. Red (78l r1): only "W" spans, a stray
/// `<td/>` beside "T", "W2", "W3" and "W4". Repro alignment-bindings/tblr_setcell_after_a_row_rule_spans.
#[test]
fn tblr_setcell_after_a_row_rule_spans() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/tblr_setcell_after_a_row_rule_spans.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tbody",
    &[],
    "<tbody><tr xml:id=\"p1.1.1\"><td align=\"center\" border=\"l r t\" rowspan=\"2\" \
     xml:id=\"p1.1.1.1\">T</td><td align=\"center\" border=\"r t\" xml:id=\"p1.1.1.2\">A</td><td align=\"center\" \
     border=\"r t\" xml:id=\"p1.1.1.3\">B</td></tr><tr xml:id=\"p1.1.2\"><td align=\"center\" border=\"r\" \
     xml:id=\"p1.1.2.1\">C</td><td align=\"center\" border=\"r\" xml:id=\"p1.1.2.2\">D</td></tr><tr \
     xml:id=\"p1.1.3\"><td align=\"center\" border=\"l r t\" colspan=\"2\" xml:id=\"p1.1.3.1\">W</td><td \
     align=\"center\" border=\"r t\" xml:id=\"p1.1.3.2\">E</td></tr><tr xml:id=\"p1.1.4\"><td align=\"center\" \
     border=\"l r t\" colspan=\"2\" xml:id=\"p1.1.4.1\">W2</td><td align=\"center\" border=\"r t\" \
     xml:id=\"p1.1.4.2\">F</td></tr><tr xml:id=\"p1.1.5\"><td align=\"center\" border=\"l r\" colspan=\"2\" \
     xml:id=\"p1.1.5.1\">W3</td><td align=\"center\" border=\"r\" xml:id=\"p1.1.5.2\">G</td></tr><tr \
     xml:id=\"p1.1.6\"><td align=\"center\" border=\"l r t\" colspan=\"3\" \
     xml:id=\"p1.1.6.1\">W4</td></tr></tbody>",
  );
}

/// `\caption*` under `[font=small,labelfont=bf,textfont=it]` is italic at 90 % with no tag, as caption sets it
/// (caption.sty:210-227, caption3.sty:1686-1715; caption_sty.rs `caption_star_format_tokens`); the unstarred
/// caption keeps its bold label. Red (int49a): `<caption>Starred caption</caption>`. Repro
/// captions-floats/caption_star_takes_the_caption_fonts.
#[test]
fn caption_star_takes_the_caption_fonts() {
  let (log, xml) = convert_with(
    repro!("captions-floats/caption_star_takes_the_caption_fonts.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"fig1\""],
    "<figure xml:id=\"fig1\"><caption font=\"italic\" fontsize=\"90%\">Starred caption</caption></figure>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag \
     role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Normal caption</toccaption><caption><tag \
     close=\": \"><text font=\"bold\" fontsize=\"90%\" xml:id=\"S0.F1.1\">Figure 1</text></tag><text font=\"italic\" \
     fontsize=\"90%\" xml:id=\"S0.F1.2\">Normal caption</text></caption></figure>",
  );
}

/// Every starred caption takes the type's settings (`\captionsetup[table]{textfont=sc}` replaces `textfont=it`) and
/// the text format (`textformat=period`): a table's, longtable's (caption.sty:1167 `\LT@makecaption`) and
/// `\captionof*`'s (caption.sty:389-391). pdflatex 0 errors: "Starred caption." italic, "Starred table." and "Long
/// star." in small caps, all small, no labels.
#[test]
fn caption_star_takes_the_type_settings_everywhere() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage[font=small,labelfont=bf,textfont=it,textformat=period]{caption}\
     \\usepackage{longtable}\\captionsetup[table]{textfont=sc}\\begin{document}\
     \\begin{figure}\\caption*{Starred caption}\\end{figure}\\begin{table}\\caption*{Starred table}\\end{table}\
     \\begin{longtable}{c}\\caption*{Long star}\\\\ a\\\\ \\end{longtable}Text \\captionof*{figure}{Of star}.\
     \\end{document}",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let captions: Vec<String> = (0..)
    .map_while(|n| {
      xml.match_indices("<caption").nth(n).map(|(at, _)| {
        let end = xml[at..]
          .find("</caption>")
          .map_or(xml.len(), |e| at + e + "</caption>".len());
        xml[at..end].to_string()
      })
    })
    .collect();
  assert_eq!(
    captions,
    [
      "<caption font=\"italic\" fontsize=\"90%\">Starred caption.</caption>",
      "<caption font=\"smallcaps\" fontsize=\"90%\">Starred table.</caption>",
      "<caption font=\"smallcaps\" fontsize=\"90%\">Long star.</caption>",
      "<caption font=\"italic\" fontsize=\"90%\">Of star.</caption>",
    ],
    "{xml}"
  );
}
