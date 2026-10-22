//! Red/green guards for batch 78u (frontmatter content loss): a note written inside an author's mark
//! (`\textsuperscript{1,2\thanks{…}}`, `$^{1,\thanks{…}}$`, `$^{1\footnotemark[1]}$`) keeps the mark's labels and the
//! note; `classify_thanks` reads its funding words as words; a footnote's `[\numexpr…]` is a number. Each guard
//! converts its repro under `tools/perfect_kernel/repros/` (pdflatex-checked) and asserts the diagnostics by count and
//! the authored content in whole elements.
use latexml::util::test::{convert_with, normalize_markup, xml_element};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::assert_elements_with,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Every `<creator …>` element of `tex`'s XML in document order, whole (`xml_element`), against `expected`, after 0
/// errors and 0 warnings: the creators of a title block have no id to pick one by.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&str]) {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut found = Vec::new();
  let mut rest = xml.as_str();
  while let Some(element) = xml_element(rest, "creator", &[]) {
    found.push(element);
    let open = rest.find("<creator").unwrap();
    rest = &rest[open + 1..];
  }
  let expected: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(found, expected, "{xml}");
}

/// latex.ltx:17215: `Bob Beta\textsuperscript{\rm 1,\rm 2\thanks{…}}` prints "1,2" and the note's mark in the
/// superscript and the text as a footnote. Red (int59a): Bob's mark "1" (the label `2\thanks{…}` answered nothing and
/// printed nothing) and no note. Witness 2512.05464. Repro
/// sectioning-frontmatter/author_thanks_inside_a_mark_keeps_the_mark_and_the_note.
#[test]
fn author_thanks_inside_a_mark_keeps_the_mark_and_the_note() {
  assert_creators(
    repro!("sectioning-frontmatter/author_thanks_inside_a_mark_keeps_the_mark_and_the_note.tex"),
    &[
      r#"<creator role="author"><personname>Ann Alpha<sup xml:id="id1">1</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Beta<sup xml:id="id2">1,2</sup></personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Work done during an internship.</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Gamma<sup xml:id="id4">1</sup></personname></creator>"#,
    ],
  );
}

/// A `\thanks` inside a math mark, under a font command, or alone in it: the labels stay the mark and the note is the
/// author's; a mark holding only the note goes. Red (int59a): "1", "2", nothing, and no note. Witnesses 2501.01423,
/// 2402.13113, 2212.00809. Repro sectioning-frontmatter/author_math_mark_thanks_keeps_the_mark_and_the_note.
#[test]
fn author_math_mark_thanks_keeps_the_mark_and_the_note() {
  assert_creators(
    repro!("sectioning-frontmatter/author_math_mark_thanks_keeps_the_mark_and_the_note.tex"),
    &[
      r#"<creator role="author"><personname>Ann Alpha<sup xml:id="id1">1</sup></personname><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id2">Corresponding author.</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Beta<sup xml:id="id3">2</sup></personname><note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id4">Equal contribution.</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Gamma</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id5">Topical group convener.</note></creator>"#,
    ],
  );
}

/// `$^{1\footnotemark[1]}$` prints the label and the note's mark; the label `1\footnotemark[1]` printed nothing. Red
/// (int59a): Ann and Bob no mark, no note mark. The order is a residual: the unanswered label's superscript is made at
/// the end of the name, after the note mark (pdflatex "Ann Alpha1∗"). Witness 2403.04164. Repro
/// sectioning-frontmatter/author_footnotemark_inside_a_mark_is_kept.
#[test]
fn author_footnotemark_inside_a_mark_is_kept() {
  assert_creators(
    repro!("sectioning-frontmatter/author_footnotemark_inside_a_mark_is_kept.tex"),
    &[
      r#"<creator role="author"><personname>Ann Alpha<note mark="*" role="footnotemark" xml:id="footnotex1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags></note><sup xml:id="id1">1</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Beta<note mark="*" role="footnotemark" xml:id="footnotex2"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags></note><sup xml:id="id2">2</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Gamma<sup xml:id="id3">1</sup></personname><note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id4">These authors contributed equally.</note></creator>"#,
    ],
  );
}

/// A macro of optional arguments only inside a mark that makes a note (`\textsuperscript{\rm 1,2,\equalcontrib}`,
/// `\textsuperscript{2\samethanks}`) is the note. Red (int59a): the labels "1,2" and "1", Cy nothing, no note. The
/// order residual of `author_footnotemark_inside_a_mark_is_kept` holds for Cy. Witnesses 2603.17529, 2309.12862. Repro
/// sectioning-frontmatter/author_note_macro_inside_a_mark_keeps_the_note.
#[test]
fn author_note_macro_inside_a_mark_keeps_the_note() {
  assert_creators(
    repro!("sectioning-frontmatter/author_note_macro_inside_a_mark_keeps_the_note.tex"),
    &[
      r#"<creator role="author"><personname>Ann Alpha<sup xml:id="id1">1,2</sup></personname><note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id2">Equal contribution.</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Beta<sup xml:id="id3">1</sup></personname><note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id4">Equal contribution.</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Gamma<note mark="†" role="footnotemark" xml:id="footnotex1"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags></note><sup xml:id="id5">2</sup></personname></creator>"#,
    ],
  );
}

/// `classify_thanks` reads "erc" as a word: an author's `\thanks{Laboratory of Machine Perception, …}` is a note, and
/// one naming its funder funding. Red (int59a): the first `ltx_thanks_funding`. Witness 2509.05368 ("Machine
/// Perception").
#[test]
fn thanks_institution_with_erc_in_a_word_is_no_funding() {
  assert_creators(
    r"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\thanks{Laboratory of Machine Perception, Peking University} \and Bob Baker\thanks{Supported by NSF grant 123.}}
\maketitle
Body.
\end{document}",
    &[
      r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Laboratory of Machine Perception, Peking University</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Baker</personname><note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id2">Supported by NSF grant 123.</note></creator>"#,
    ],
  );
}

/// latex.ltx:17678 `\c@footnote #1\relax` reads an e-TeX expression whole: `\footnotemark[\numexpr\value{footnote}-1]`
/// is the number 1, and `\footnotetext[\numexpr\value{footnote}-1]{…}` its text, one note. Red (int59a): two "Missing
/// number, treated as zero" Warnings, the mark empty and its text apart. KNOWN_PERL_ERRORS #1135; witness 2210.14698.
/// Repro package-bindings/footnote_mark_numexpr_is_a_number.
#[test]
fn footnote_mark_numexpr_is_a_number() {
  assert_elements_with(
    repro!("package-bindings/footnote_mark_numexpr_is_a_number.tex"),
    PRELOAD,
    (0, 0),
    &[],
    &[(
      "p",
      "p1.1",
      r#"<p xml:id="p1.1">A<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>One.</note> B<note mark="2" role="footnote" xml:id="footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>Two.</note><note mark="1" role="footnote" xml:id="footnotex1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Again one.</note> C.</p>"#,
    )],
  );
}
