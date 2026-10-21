//! Red/green guards for batch 78a (quality batch 5: tables, fonts, ligatures, citations, sectioning). Each guard
//! converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the authored content
//! in whole elements. Witnesses 0712.2739, 2608.01828 (jfm), 2602.22000 (multirow), 1803.08052 (boldmath),
//! nucl-ex0511052 and 2208.08472 (ligatures), 2402.17192 and 2410.22329 (biblatex), 2605.04767 (IEEEtran),
//! 2607.29279 (llncs); round 2: 1711.01065, 2205.10181, 2507.07213, 1805.00129 (jfm copies under caption loaders),
//! 2312.13002, 2111.12004 (jfm copies under floatrow), 1905.01770 (svmult).
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
/// class-bindings/jfm_caption. (The empty `<text xml:id=…/>` closing the `autoref` tag is not the class's: every
/// hyperref figure's autoref tag has it, an article's too — the `\null` that ends hyperref's autoref text,
/// `\lx@autorefnum@@`, hyperref_sty.rs; a known artifact, pinned as it is.)
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

/// `\multirow` in a p column spans its rows: the row span may cover the p cell below, whose paragraph box has an empty
/// body (the `paragraph_cell_body` property `Digested::is_empty_paragraph_cell` reads), so the span is not truncated. Red (int40c, Perl the same): no rowspan, a second empty `<td>`.
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

/// The document's caption package — here subcaption's — replaces a jfm copy's own `\@makecaption`: "Figure 1:" in the
/// caption font, no small caps, the sub-caption's "(a)" at its size. Red (78a r1): `close=". "`, the labels and the
/// "(a)" in small caps. Witnesses 1711.01065, 2205.10181, 2507.07213. Repro
/// class-bindings/jfm_caption/jfm_copy_caption_yields_to_subcaption.
#[test]
fn jfm_copy_caption_yields_to_caption_loaders() {
  assert_clean(
    repro!("class-bindings/jfm_caption/jfm_copy_caption_yields_to_subcaption.tex"),
    &[("jfm.cls", repro!("class-bindings/jfm_caption/jfm.cls"))],
    &[(
      "figure",
      "S0.F1",
      "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.1\">Figure \
       1</text></tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.2\"/></tag><tag role=\"refnum\">1</tag><tag \
       role=\"typerefnum\">Figure 1</tag></tags><figure inlist=\"lof\" xml:id=\"S0.F1.sf1\"><tags><tag><text \
       fontsize=\"90%\" xml:id=\"S0.F1.sf1.1\">(a)</text></tag><tag role=\"autoref\">Figure\u{a0}1a<text \
       xml:id=\"S0.F1.sf1.2\"/></tag><tag role=\"refnum\">1a</tag></tags><toccaption><tag close=\" \">a</tag>Left.\
       </toccaption><caption><tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">(a)</text></tag><text \
       fontsize=\"90%\" xml:id=\"S0.F1.sf1.4\">Left.</text></caption></figure><toccaption><tag close=\" \">1</tag>\
       Schematic.</toccaption><caption><tag close=\": \"><text fontsize=\"90%\" xml:id=\"S0.F1.3\">Figure 1</text>\
       </tag><text fontsize=\"90%\" xml:id=\"S0.F1.4\">Schematic.</text></caption></figure>",
    )],
  );
  // subfig loads caption too (1805.00129): "Figure 1:" upright, "(a)" plain.
  assert_clean(
    repro!("class-bindings/jfm_caption/jfm_copy_caption_yields_to_subfig.tex"),
    &[("jfm.cls", repro!("class-bindings/jfm_caption/jfm.cls"))],
    &[(
      "figure",
      "S0.F1",
      "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text \
       xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><figure \
       xml:id=\"S0.F1.sf1\"><tags><tag>(a)</tag><tag role=\"autoref\">1a<text xml:id=\"S0.F1.sf1.1\"/></tag><tag \
       role=\"refnum\">1a</tag></tags><p xml:id=\"S0.F1.sf1.2\">x</p><toccaption><tag close=\" \">a</tag>Left.\
       </toccaption><caption><tag close=\" \">(a)</tag>Left.</caption></figure><toccaption><tag close=\" \">1</tag>\
       Schematic.</toccaption><caption><tag close=\": \">Figure 1</tag>Schematic.</caption></figure>",
    )],
  );
}

/// The old subfigure package loads no caption: the jfm copy's format stays ("Figure 1." in small caps, "Table 1"
/// after `\removefullpoint`), the sub-caption's "(a)" at subfigure's size. Control (78a r1 the same). Witness
/// 0712.2739. Repro class-bindings/jfm_caption/jfm_copy_caption_kept_with_subfigure.
#[test]
fn jfm_copy_caption_kept_with_subfigure() {
  assert_clean(
    repro!("class-bindings/jfm_caption/jfm_copy_caption_kept_with_subfigure.tex"),
    &[("jfm.cls", repro!("class-bindings/jfm_caption/jfm.cls"))],
    &[
      (
        "figure",
        "S0.F1",
        "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag><text font=\"smallcaps\" xml:id=\"S0.F1.1\">Figure \
         1</text></tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.2\"/></tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Figure 1</tag></tags><figure xml:id=\"S0.F1.sf1\"><tags><tag><text fontsize=\"80%\" \
         xml:id=\"S0.F1.sf1.1\">(a)</text></tag><tag role=\"autoref\">1(a)<text xml:id=\"S0.F1.sf1.2\"/></tag><tag \
         role=\"refnum\">1(a)</tag></tags><p xml:id=\"S0.F1.sf1.3\">x</p><toccaption><tag close=\" \">(a)</tag>Left.\
         </toccaption><caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F1.sf1.4\">(a)</text></tag><text \
         fontsize=\"80%\" xml:id=\"S0.F1.sf1.5\">Left.</text></caption></figure><toccaption><tag close=\" \">1</tag>\
         Schematic.</toccaption><caption><tag close=\". \"><text font=\"smallcaps\" xml:id=\"S0.F1.3\">Figure 1</text>\
         </tag>Schematic.</caption></figure>",
      ),
      (
        "table",
        "S0.T1",
        "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag><text font=\"smallcaps\" xml:id=\"S0.T1.1\">Table \
         1</text></tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.2\"/></tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Values.</toccaption><caption>\
         <tag close=\" \"><text font=\"smallcaps\" xml:id=\"S0.T1.3\">Table 1</text></tag>Values.</caption></table>",
      ),
    ],
  );
}

/// floatrow loads caption's kernel alone (floatrow.sty:39 `caption3`), which keeps the class's `\@makecaption`: the jfm
/// copy's "Figure 1." in small caps stays, "Table 1" after `\removefullpoint`. Red (78a r2 before
/// `\lx@caption@kernel@only`): caption3's stand-in, the caption binding, read as caption loaded — "Figure 1:" upright.
/// Witnesses 2312.13002, 2111.12004. Repro class-bindings/jfm_caption/jfm_copy_caption_kept_with_floatrow.
#[test]
fn jfm_copy_caption_kept_with_floatrow() {
  assert_clean(
    repro!("class-bindings/jfm_caption/jfm_copy_caption_kept_with_floatrow.tex"),
    &[("jfm.cls", repro!("class-bindings/jfm_caption/jfm.cls"))],
    &[
      (
        "figure",
        "S0.F1",
        "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag><text font=\"smallcaps\" xml:id=\"S0.F1.1\">Figure \
         1</text></tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.2\"/></tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Schematic.</toccaption>\
         <caption><tag close=\". \"><text font=\"smallcaps\" xml:id=\"S0.F1.3\">Figure 1</text></tag>Schematic.\
         </caption></figure>",
      ),
      (
        "table",
        "S0.T1",
        "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag><text font=\"smallcaps\" xml:id=\"S0.T1.1\">Table \
         1</text></tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.2\"/></tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Values.</toccaption><caption>\
         <tag close=\" \"><text font=\"smallcaps\" xml:id=\"S0.T1.3\">Table 1</text></tag>Values.</caption></table>",
      ),
    ],
  );
}

/// svmult's `\backmatter` keeps the chapters numbered (svmult.cls:478-484, `%\@mainmatterfalse` commented out): the
/// back-matter chapter is "Chapter 2". Red (78a r1, book's `\backmatter`): "Afterword" unnumbered, "1.2 Notes".
/// Witness 1905.01770's svmult.cls. Repro sectioning-frontmatter/svmult_backmatter_chapters_stay_numbered.
#[test]
fn svmult_backmatter_chapters_stay_numbered() {
  assert_clean(
    repro!("sectioning-frontmatter/svmult_backmatter_chapters_stay_numbered.tex"),
    &[],
    &[(
      "chapter",
      "Ch2",
      "<chapter inlist=\"toc\" xml:id=\"Ch2\"><tags><tag>Chapter 2</tag><tag role=\"refnum\">2</tag><tag \
       role=\"typerefnum\">Chapter 2</tag></tags><title><tag close=\" \">Chapter 2</tag>Afterword</title><toctitle><tag \
       close=\" \">2</tag>Afterword</toctitle><section inlist=\"toc\" xml:id=\"Ch2.S1\"><tags><tag>2.1</tag><tag \
       role=\"refnum\">2.1</tag><tag role=\"typerefnum\">\u{00A7}2.1</tag></tags><title><tag close=\" \">2.1</tag>\
       Notes</title><para xml:id=\"Ch2.S1.p1\"><p xml:id=\"Ch2.S1.p1.1\">Text.</p></para></section></chapter>",
    )],
  );
}

/// book's `\appendix` restarts the sections (book.cls:520-524 `\setcounter{section}{0}`): after `\backmatter` no
/// chapter step does it, so the appendix's first section is ".1" as in pdflatex. Red (78a r1): ".3". Repro
/// sectioning-frontmatter/book_appendix_after_backmatter_restarts_sections.
#[test]
fn book_appendix_after_backmatter_restarts_sections() {
  assert_clean(
    repro!("sectioning-frontmatter/book_appendix_after_backmatter_restarts_sections.tex"),
    &[],
    &[(
      "appendix",
      "Ax1",
      "<appendix inlist=\"toc\" xml:id=\"Ax1\"><title>Extra</title><section inlist=\"toc\" xml:id=\"Ax1.S1\"><tags>\
       <tag>.1</tag><tag role=\"refnum\">.1</tag><tag role=\"typerefnum\">1.\u{00A7}</tag></tags><title><tag \
       close=\" \">.1</tag>Proofs</title><para xml:id=\"Ax1.S1.p1\"><p xml:id=\"Ax1.S1.p1.1\">P.</p></para></section>\
       </appendix>",
    )],
  );
}

/// A numeric `\textcite` of an entry with no author keeps the brackets around its number ("[2]"; biblatex prints the
/// labeltitle before them, numeric.cbx:23-41 — OXIDIZED_DESIGN_DIVERGENCES #786's residual); a named one is "Hampe
/// [1]". The label is the post cross-referencer's (CrossRef.pm:541), so the repro is converted to HTML in a directory
/// of its own, where its `filecontents*` writes the `.bib` the bibliography reads. Red (78a r1, Perl the same): a bare
/// "2". Repro index-bib/biblatex_textcite_numeric_without_author.
#[test]
fn biblatex_numeric_textcite_without_author_keeps_its_brackets() {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(
    workdir.path().join("t.tex"),
    repro!("index-bib/biblatex_textcite_numeric_without_author.tex"),
  )
  .expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html");
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    "<p class=\"ltx_p\" id=\"p1.1\"><cite class=\"ltx_cite ltx_citemacro_citet\">Hampe [<a class=\"ltx_ref\" \
     href=\"#bib.bib1\" title=\"T\">1</a>]</cite> showed it.\n<cite class=\"ltx_cite ltx_citemacro_citet\">[<a \
     class=\"ltx_ref\" href=\"#bib.bib2\" title=\"No Author Misc\">2</a>]</cite> said it.</p>",
  );
}
