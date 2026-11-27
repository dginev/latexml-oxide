//! Red/green guards for ABSNAME: a frontmatter name with spacing in it is the text pdflatex prints, never its TeX
//! spelling. Repros `tools/perfect_kernel/repros/sectioning-frontmatter/`. OXIDIZED_DESIGN_DIVERGENCES #1170.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const SPACING: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_name_spacing_is_not_its_tex_spelling.tex"
);
const WITNESS_BODY: &str = "\\abstractname{\\large\\bfseries Abstract\\vspace*{6pt}}";

/// One row per path: `\vspace*`'s vertical glue is nothing (1607.06945, sigsam.sty:99); `\hskip1em` is its em space,
/// not the `\quad` it reverts to (1312.7183, math0609457, arxstspdf.cls); a name of only vertical glue is no label
/// (1502.02294, jkas.cls); `\kern` is its spaces; `\par` is a word boundary, a space (`Sum\par mary`; a trailing one
/// kept, as Perl's `setAttribute` writes it); the renewed `\refname`, an element's text, stays "References". The amsart
/// row reaches the names through the calls' keyvals (the e-mail contact, the keywords); the caption row a separator's
/// `\par` through the tag's `close` (pdflatex: "Figure 1." then "First caption").
#[test]
fn frontmatter_name_spacing_is_its_attribute_form() {
  let abstract_of = |name: &str| {
    format!(
      "<abstract inlist=\"toc\"{name} xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">Text.</p></abstract>"
    )
  };
  assert_eq!(SPACING.matches(WITNESS_BODY).count(), 1);
  for (body, name) in [
    (WITNESS_BODY, " name=\"Abstract\""),
    (
      "\\abstractname{\\textit{Abstract.}\\hskip1em}",
      " name=\"Abstract.\u{2003}\"",
    ),
    ("\\abstractname{{}\\vspace{-14mm}}", ""),
    (
      "\\abstractname{Abstract\\kern3pt}",
      " name=\"Abstract\u{2005}\"",
    ),
    (
      "\\abstractname{Abstract\\par\\smallskip}",
      " name=\"Abstract \"",
    ),
    ("\\abstractname{Sum\\par mary}", " name=\"Sum mary\""),
    // the paragraph end a raw `\vskip` inserts is real (tex.web §1094); only a `\vspace`'s (a `\vadjust`) is not
    ("\\abstractname{Sum\\vskip2pt mary}", " name=\"Sum mary\""),
    // before any word TeX is still in vertical mode: no boundary (2603.28538)
    ("\\abstractname{\\vskip-3mm Abstract}", " name=\"Abstract\""),
    ("\\abstractname{\\par}", ""),
  ] {
    let tex = SPACING.replacen(WITNESS_BODY, body, 1);
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{body}\n{log}"
    );
    assert_element(&xml, "abstract", &[], &abstract_of(name));
    assert!(
      xml.contains("<bibliography inlist=\"toc\" xml:id=\"bib\">\n    <title>References</title>"),
      "{body}\n{xml}"
    );
  }

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/frontmatter_names_spacing_through_keyvals.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A. Author</personname><contact name=\"E-mail\u{2009}: \" \
     role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">a@b.c</text></contact></creator>",
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    "<abstract inlist=\"toc\" name=\"Abstract.\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">Text.</p></abstract>",
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Key words:\u{a0}\">alpha, beta</keywords>",
  );

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/caption_separator_par_is_a_word_boundary.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\". \">Figure 1</tag>First caption</caption>",
  );
}
