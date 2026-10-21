//! Red/green guards for batch 78a (quality batch 5: tables, fonts, ligatures, citations, sectioning). Each guard
//! converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the authored content
//! in whole elements. Witnesses 0712.2739, 2608.01828 (jfm), 2602.22000 (multirow), 1803.08052 (boldmath),
//! nucl-ex0511052 and 2208.08472 (ligatures), 2402.17192 and 2410.22329 (biblatex), 2605.04767 (IEEEtran),
//! 2607.29279 (llncs).
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex` (with `files` beside it): 0 errors, 0 warnings, and each `(tag, xml:id, element)` as given.
fn assert_clean(tex: &str, files: &[(&str, &str)], expected: &[(&str, &str, &str)]) -> String {
  let (log, xml) = if files.is_empty() {
    convert_with(tex, Some(PRELOAD))
  } else {
    convert_files_with(tex, files, Some(PRELOAD))
  };
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (tag, id, element) in expected {
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
  xml
}

/// A jfm copy without caption sets its own `\@makecaption` (0712.2739's jfm.cls:953-958): "Figure 1." with the label
/// in small caps, and `\removefullpoint` drops the period once. Red (int40c): `<tag close=": ">` upright. Repro
/// class-bindings/jfm_caption.
#[test]
fn jfm_copy_captions_end_the_label_with_a_period() {
  assert_clean(
    repro!("class-bindings/jfm_caption/jfm_copy_caption_ends_with_a_period.tex"),
    &[("jfm.cls", repro!("class-bindings/jfm_caption/jfm.cls"))],
    &[
      (
        "figure",
        "S0.F1",
        "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag><text font=\"smallcaps\" xml:id=\"S0.F1.1\">Figure \
         1</text></tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.2\"/></tag><tag role=\"refnum\">1</tag><tag \
         role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Schematic.</toccaption><caption>\
         <tag close=\". \"><text font=\"smallcaps\" xml:id=\"S0.F1.3\">Figure 1</text></tag>Schematic.</caption>\
         </figure>",
      ),
      (
        "table",
        "S0.T1",
        "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag><text font=\"smallcaps\" xml:id=\"S0.T1.1\">Table \
         1</text></tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.2\"/></tag><tag role=\"refnum\">1</tag><tag \
         role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Values.</toccaption><caption>\
         <tag close=\" \"><text font=\"smallcaps\" xml:id=\"S0.T1.3\">Table 1</text></tag>Values.</caption></table>",
      ),
    ],
  );
}

/// A jfm copy that loads caption keeps its options (2608.01828's jfm.cls:1837 `labelsep=period`): "Figure 1.". Red
/// (int40c): `close=": "`. Repro class-bindings/jfm_caption_package.
#[test]
fn jfm_copy_caption_package_keeps_its_options() {
  let xml = assert_clean(
    repro!("class-bindings/jfm_caption_package/jfm_copy_caption_package_keeps_its_options.tex"),
    &[(
      "jfm.cls",
      repro!("class-bindings/jfm_caption_package/jfm.cls"),
    )],
    &[],
  );
  assert!(
    xml.contains(
      "<caption><tag close=\". \"><text fontsize=\"90%\" xml:id=\"S0.F1.2\">Figure 1</text></tag><text \
       fontsize=\"80%\" xml:id=\"S0.F1.3\">Schematic.</text></caption>"
    ),
    "{xml}"
  );
}

/// `\multirow` in a p column spans its rows: the empty p cell below is empty (its paragraph box is the cell's
/// `content_box`), so the span is not truncated. Red (int40c, Perl the same): no rowspan, a second empty `<td>`.
/// Witness 2602.22000. Repro alignment-bindings/multirow_in_a_paragraph_column_spans_its_rows.
#[test]
fn multirow_in_a_paragraph_column_spans_its_rows() {
  assert_clean(
    repro!("alignment-bindings/multirow_in_a_paragraph_column_spans_its_rows.tex"),
    &[],
    &[(
      "tabular",
      "p1.1",
      "<tabular vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr xml:id=\"p1.1.1\"><td align=\"left\" border=\"l r\" \
       rowspan=\"2\" vattach=\"top\" xml:id=\"p1.1.1.1\"><inline-block vattach=\"top\" width=\"56.9pt\" \
       xml:id=\"p1.1.1.1.1\"><p xml:id=\"p1.1.1.1.1.1\">Spanned text</p></inline-block></td><td align=\"center\" \
       border=\"r\" xml:id=\"p1.1.1.2\">a</td></tr><tr xml:id=\"p1.1.2\"><td align=\"center\" border=\"r\" \
       xml:id=\"p1.1.2.1\">b</td></tr></tbody></tabular>",
    )],
  );
}

/// `\unboldmath` and `\mathversion{normal}` return to the normal math version; a group end undoes `\boldmath`. Red
/// (int40c, Perl the same, KNOWN_PERL_ERRORS #687): y and v bold. Witness 1803.08052. Repro
/// fonts-nfss/boldmath_undone_by_unboldmath_and_group.
#[test]
fn unboldmath_restores_the_normal_math_version() {
  let tok = |id: &str, font: &str, letter: &str| {
    format!(
      "<Math mode=\"inline\" tex=\"{letter}\" text=\"{letter}\" xml:id=\"{id}\"><XMath xml:id=\"{id}.1\"><XMTok \
       font=\"{font}\" role=\"UNKNOWN\">{letter}</XMTok></XMath></Math>"
    )
  };
  let expected = format!(
    "<p xml:id=\"p1.1\">{} {} {} {}</p>",
    tok("p1.m1", "bold italic", "x"),
    tok("p1.m2", "italic", "y"),
    tok("p1.m3", "bold italic", "u"),
    tok("p1.m4", "italic", "v")
  );
  assert_clean(
    repro!("fonts-nfss/boldmath_undone_by_unboldmath_and_group.tex"),
    &[],
    &[("p", "p1.1", &expected)],
  );
  let expected = format!(
    "<p xml:id=\"p1.1\">{} {}</p>",
    tok("p1.m1", "bold italic", "x"),
    tok("p1.m2", "italic", "y")
  );
  assert_clean(
    "\\documentclass{article}\\begin{document}\\mathversion{bold}$x$ \\mathversion{normal}$y$\\end{document}",
    &[],
    &[("p", "p1.1", &expected)],
  );
}

/// Text after a formula `cleanup_math` turns into spaces is ligated: "1989–2008." in every paragraph. Red (int40c,
/// RUST-ONLY): "1989--2008." after `$\,$`. Witnesses nucl-ex0511052, 2208.08472. Repro
/// fonts-nfss/ligature_after_space_only_formula.
#[test]
fn text_after_a_space_only_formula_is_ligatured() {
  assert_clean(
    repro!("fonts-nfss/ligature_after_space_only_formula.tex"),
    &[],
    &[
      ("p", "p1.1", "<p xml:id=\"p1.1\">A 1989\u{2013}2008.</p>"),
      (
        "p",
        "p2.1",
        "<p xml:id=\"p2.1\">B 500\u{2006}hPa 1989\u{2013}2008.</p>",
      ),
      (
        "p",
        "p3.1",
        "<p xml:id=\"p3.1\">C 500\u{2006}hPa 1989\u{2013}2008.</p>",
      ),
    ],
  );
}

/// biblatex's `\citeauthor` prints the name in every style, and a numeric `\textcite` "Name [1]" (biblatex.def
/// :2497-2506, numeric.cbx `textcite`). Red (int40c): both "[1]". Witnesses 2402.17192, 2410.22329. Repro
/// index-bib/biblatex_citeauthor_numeric_prints_name.
#[test]
fn biblatex_citeauthor_and_textcite_print_the_name_in_a_numeric_style() {
  assert_clean(
    repro!("index-bib/biblatex_citeauthor_numeric_prints_name.tex"),
    &[],
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\"><cite class=\"ltx_citemacro_citeauthor\"><bibref bibrefs=\"Hampe_2017\" separator=\",\" \
       show=\"Authors\" yyseparator=\",\"/></cite> showed it.\n<cite class=\"ltx_citemacro_citet\"><bibref \
       bibrefs=\"Hampe_2017\" separator=\",\" show=\"Authors Phrase1NumberPhrase2\" yyseparator=\",\">\
       <bibrefphrase>[</bibrefphrase><bibrefphrase>]</bibrefphrase></bibref></cite> showed it too.</p>",
    )],
  );
}

/// An unnumbered section takes the class's title font, as a numbered one (IEEEtran's small caps). Red (int40c):
/// `<title>Acknowledgment</title>`. Witness 2605.04767. Repro sectioning-frontmatter/unnumbered_section_takes_the_title_font.
#[test]
fn unnumbered_section_takes_the_title_font() {
  assert_clean(
    repro!("sectioning-frontmatter/unnumbered_section_takes_the_title_font.tex"),
    &[],
    &[(
      "section",
      "Sx1",
      "<section xml:id=\"Sx1\"><title font=\"smallcaps\">Acknowledgment</title><para xml:id=\"Sx1.p1\"><p \
       xml:id=\"Sx1.p1.1\">Thanks.</p></para></section>",
    )],
  );
}

/// titlesec's after-code closes what its format opened in an unnumbered title too (tufte-common.def:1613-1619's
/// `\begin{fullwidth}` … `\end{fullwidth}`): 0 errors, the title in the format's font, the body after it upright.
/// Repro sectioning-frontmatter/titlesec_after_code_closes_an_unnumbered_title.
#[test]
fn titlesec_after_code_closes_an_unnumbered_title() {
  assert_clean(
    repro!("sectioning-frontmatter/titlesec_after_code_closes_an_unnumbered_title.tex"),
    &[],
    &[(
      "section",
      "Sx1",
      "<section xml:id=\"Sx1\"><title font=\"italic\">Starred</title><para xml:id=\"Sx1.p1\"><p \
       xml:id=\"Sx1.p1.1\">Body.</p></para></section>",
    )],
  );
}

/// nicematrix's `\Block{i-j}<tokens>{content}` sets the tokens before the content (nicematrix.sty:7258). Red
/// (int40c): "¡Tall¿". Repro alignment-bindings/nicematrix_block_angle_tokens_apply_to_the_content.
#[test]
fn nicematrix_block_angle_tokens_apply_to_the_content() {
  assert_clean(
    repro!("alignment-bindings/nicematrix_block_angle_tokens_apply_to_the_content.tex"),
    &[],
    &[(
      "td",
      "p1.1.1.1",
      "<td align=\"center\" xml:id=\"p1.1.1.1\"><text font=\"bold\" xml:id=\"p1.1.1.1.1\">Tall</text></td>",
    )],
  );
}

/// book's back-matter chapters are unnumbered and leave the chapter counter alone; their sections keep numbering
/// under the last main chapter ("1.2"). Red (int40c, Perl the same): "Chapter 2 Afterword", "2.1 Notes". Repro
/// sectioning-frontmatter/book_backmatter_chapters_are_unnumbered.
#[test]
fn book_backmatter_chapters_are_unnumbered() {
  assert_clean(
    repro!("sectioning-frontmatter/book_backmatter_chapters_are_unnumbered.tex"),
    &[],
    &[(
      "chapter",
      "Chx2",
      "<chapter inlist=\"toc\" xml:id=\"Chx2\"><title>Afterword</title><section inlist=\"toc\" \
       xml:id=\"Chx2.S2\"><tags><tag>1.2</tag><tag role=\"refnum\">1.2</tag><tag \
       role=\"typerefnum\">\u{00A7}1.2</tag></tags><title><tag close=\" \">1.2</tag>Notes</title><para \
       xml:id=\"Chx2.S2.p1\"><p xml:id=\"Chx2.S2.p1.1\">A.</p></para></section></chapter>",
    )],
  );
}

/// xsavebox's saved box prints its content (`\pdfxform` keeps the box, `\pdfrefxform` places it). Red (int40c): an
/// empty `<text width="22.5pt"/>`. Repro boxes-groups/xsavebox_saved_box_prints_its_content.
#[test]
fn xsavebox_saved_box_prints_its_content() {
  assert_clean(
    repro!("boxes-groups/xsavebox_saved_box_prints_its_content.tex"),
    &[],
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">A: <text align=\"center\" width=\"22.5pt\" xml:id=\"p1.1.1\">Hello</text>.</p>",
    )],
  );
}

/// llncs's item labels (llncs.cls:521-522): a bold en dash at the first level, a bullet at the second — article's the
/// other way round. Red (int40c, Perl the same): "•" then "–". Witness 2607.29279. Repro
/// class-bindings/llncs_item_labels_are_the_class_s.
#[test]
fn llncs_item_labels_are_the_class_s() {
  assert_clean(
    repro!("class-bindings/llncs_item_labels_are_the_class_s.tex"),
    &[],
    &[(
      "item",
      "S0.I1.i1",
      "<item xml:id=\"S0.I1.i1\"><tags><tag><text font=\"bold\" xml:id=\"S0.I1.i1.1\">\u{2013}</text></tag><tag \
       role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p xml:id=\"S0.I1.i1.p1.1\">One</p>\
       <itemize xml:id=\"S0.I1.i1.I1\"><item xml:id=\"S0.I1.i1.I1.i1\"><tags><tag>\u{2022}</tag><tag \
       role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I1.i1.I1.i1.p1\"><p \
       xml:id=\"S0.I1.i1.I1.i1.p1.1\">Inner</p></para></item></itemize></para></item>",
    )],
  );
}
