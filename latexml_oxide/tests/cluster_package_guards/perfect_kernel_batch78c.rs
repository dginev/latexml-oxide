//! Red/green guards for batch 78c (quality batch 6): subcaption's caption size, IEEEtran's heading forms, acmart's
//! numeric citations, `\hhline` columns, AMS-TeX's left tags and its `@"` connector, the jmlr/jmlr2e acknowledgement
//! headings, svmult's contribution numbering. Each guard converts its repro under `tools/perfect_kernel/repros/`,
//! asserts the diagnostics by count and the authored content in whole elements, against pdflatex/pdftex. Witnesses
//! 2112.15328 (acmart), 1512.04829 and 2511.10879 (jmlr2e), 1801.05601 (svmult), math-ph0607013 (AMS-TeX `@"`).
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

/// subcaption sets the sub-captions `\small` (subcaption.sty:218-222 `\captionsetup[sub]{font+=small}`), the float's
/// own caption and a table's at the normal size (pdflatex `\f@size` 9 and 10). Red (int43a; Perl
/// subcaption.sty.ltxml:34-35): every caption at 90 %. Repro captions-floats/subcaption_main_caption_keeps_the_normal_size.
#[test]
fn subcaption_main_caption_keeps_the_normal_size() {
  assert_clean(
    repro!("captions-floats/subcaption_main_caption_keeps_the_normal_size.tex"),
    &[],
    &[
      (
        "figure",
        "S0.F1",
        "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Figure 1</tag></tags><figure inlist=\"lof\" xml:id=\"S0.F1.sf1\"><tags><tag>\
         <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.1\">(a)</text></tag><tag role=\"refnum\">1a</tag></tags>\
         <p xml:id=\"S0.F1.sf1.2\">A</p><toccaption><tag close=\" \">a</tag>Left.</toccaption><caption>\
         <tag close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">(a)</text></tag>\
         <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.4\">Left.</text></caption></figure><toccaption>\
         <tag close=\" \">1</tag>Schematic.</toccaption><caption><tag close=\": \">Figure 1</tag>Schematic.\
         </caption></figure>",
      ),
      (
        "table",
        "S0.T1",
        "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Values.</toccaption>\
         <caption><tag close=\": \">Table 1</tag>Values.</caption></table>",
      ),
    ],
  );
}

/// IEEEtran's headings print `\the<level>dis` (IEEEtran.cls:5397 `\@seccntformat`, :2570-2587): "I.", "A.", "1)",
/// where a `\ref` reads "I-A"; an appendix keeps "Appendix A" and its subsection is "A." (`\ref` "A-A"), as pdflatex.
/// Red (int43a; Perl the same): "I", "I-A", "I-A1", "A-A". Repro
/// sectioning-frontmatter/ieeetran_headings_print_their_heading_forms.
#[test]
fn ieeetran_headings_print_their_heading_forms() {
  assert_clean(
    repro!("sectioning-frontmatter/ieeetran_headings_print_their_heading_forms.tex"),
    &[],
    &[
      (
        "section",
        "S1",
        "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>I.</tag><tag role=\"refnum\">I</tag>\
         <tag role=\"typerefnum\">\u{00A7}I</tag></tags><title><tag close=\" \">I.</tag>\
         <text font=\"smallcaps\" xml:id=\"S1.1\">Intro</text></title><toctitle><tag close=\" \">I</tag>Intro\
         </toctitle><subsection inlist=\"toc\" labels=\"LABEL:s\" xml:id=\"S1.SS1\"><tags><tag>A.</tag>\
         <tag role=\"refnum\"><text xml:id=\"S1.SS1.1\">I-A</text></tag><tag role=\"typerefnum\">\u{00A7}\
         <text xml:id=\"S1.SS1.2\">I-A</text></tag></tags><title><tag close=\" \">A.</tag>\
         <text font=\"italic\" xml:id=\"S1.SS1.3\">Sub</text></title><toctitle><tag close=\" \">\
         <text xml:id=\"S1.SS1.4\">I-A</text></tag>Sub</toctitle>\
         <subsubsection inlist=\"toc\" xml:id=\"S1.SS1.SSS1\"><tags><tag>1)</tag><tag role=\"refnum\">\
         <text xml:id=\"S1.SS1.SSS1.1\">I-A</text>1</tag><tag role=\"typerefnum\">\u{00A7}\
         <text xml:id=\"S1.SS1.SSS1.2\">I-A</text>1</tag></tags><title><tag close=\" \">1)</tag>Deep</title>\
         <toctitle><tag close=\" \"><text xml:id=\"S1.SS1.SSS1.3\">I-A</text>1</tag>Deep</toctitle>\
         <para xml:id=\"S1.SS1.SSS1.p1\"><p xml:id=\"S1.SS1.SSS1.p1.1\">Text <ref labelref=\"LABEL:s\"/>.</p>\
         </para></subsubsection></subsection></section>",
      ),
      (
        "appendix",
        "A1",
        "<appendix inlist=\"toc\" xml:id=\"A1\"><tags><tag>Appendix A</tag><tag role=\"refnum\">A</tag>\
         <tag role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix A</tag>Proofs</title>\
         <toctitle><tag close=\" \">A</tag>Proofs</toctitle>\
         <subsection inlist=\"toc\" labels=\"LABEL:a\" xml:id=\"A1.SS1\"><tags><tag>A.</tag><tag role=\"refnum\">\
         <text xml:id=\"A1.SS1.1\">A-A</text></tag><tag role=\"typerefnum\">\u{00A7}<text xml:id=\"A1.SS1.2\">A-A\
         </text></tag></tags><title><tag close=\" \">A.</tag><text font=\"italic\" xml:id=\"A1.SS1.3\">First</text>\
         </title><toctitle><tag close=\" \"><text xml:id=\"A1.SS1.4\">A-A</text></tag>First</toctitle>\
         <para xml:id=\"A1.SS1.p1\"><p xml:id=\"A1.SS1.p1.1\">Body <ref labelref=\"LABEL:a\"/>.</p></para>\
         </subsection></appendix>",
      ),
    ],
  );
  // compsoc numbers arabic (IEEEtran.cls:2545-2549, Perl IEEEtran.cls.ltxml:160-174): "1", "1.1", "1.1.1", refs
  // "1, 1.1, 1.1.1, A, A.1"; under conference each heading form ends in a period ("1.", "1.1.", :2570-2575). Red (78c
  // r1, rev78c probes ie_compsoc, ie_compsoc_conference): "I", "I.1", "I.1.1", refs "I", "I-A", "I-A1".
  assert_clean(
    repro!("sectioning-frontmatter/ieeetran_compsoc_headings_are_arabic.tex"),
    &[],
    &[
      (
        "section",
        "S1",
        "<section inlist=\"toc\" labels=\"LABEL:s1\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">§1</tag></tags><title><tag close=\" \">1</tag>\
         <text font=\"smallcaps\" xml:id=\"S1.1\">Intro</text></title><toctitle>\
         <tag close=\" \">1</tag>Intro</toctitle><subsection inlist=\"toc\" labels=\"LABEL:s11\" xml:id=\"S1.SS1\">\
         <tags><tag>1.1</tag><tag role=\"refnum\">1.1</tag><tag role=\"typerefnum\">§1.1</tag></tags><title>\
         <tag close=\" \">1.1</tag><text font=\"italic\" xml:id=\"S1.SS1.1\">Sub</text></title><toctitle>\
         <tag close=\" \">1.1</tag>Sub</toctitle>\
         <subsubsection inlist=\"toc\" labels=\"LABEL:s111\" xml:id=\"S1.SS1.SSS1\"><tags><tag>1.1.1</tag>\
         <tag role=\"refnum\">1.1.1</tag><tag role=\"typerefnum\">§1.1.1</tag></tags><title>\
         <tag close=\" \">1.1.1</tag>Subsub</title><toctitle><tag close=\" \">1.1.1</tag>Subsub</toctitle>\
         <para xml:id=\"S1.SS1.SSS1.p1\">\
         <p xml:id=\"S1.SS1.SSS1.p1.1\">Refs: <ref labelref=\"LABEL:s1\"/>, <ref labelref=\"LABEL:s11\"/>, \
         <ref labelref=\"LABEL:s111\"/>, <ref labelref=\"LABEL:a1\"/>, <ref labelref=\"LABEL:a11\"/>.</p>\
         </para></subsubsection></subsection></section>",
      ),
      (
        "appendix",
        "A1",
        "<appendix inlist=\"toc\" labels=\"LABEL:a1\" xml:id=\"A1\"><tags><tag>Appendix A</tag>\
         <tag role=\"refnum\">A</tag><tag role=\"typerefnum\">Appendix A</tag></tags><title>\
         <tag close=\" \">Appendix A</tag>Proof</title><toctitle><tag close=\" \">A</tag>Proof</toctitle>\
         <subsection inlist=\"toc\" labels=\"LABEL:a11\" xml:id=\"A1.SS1\"><tags><tag>A.1</tag>\
         <tag role=\"refnum\">A.1</tag><tag role=\"typerefnum\">§A.1</tag></tags><title><tag close=\" \">A.1</tag>\
         <text font=\"italic\" xml:id=\"A1.SS1.1\">Lemma</text></title><toctitle>\
         <tag close=\" \">A.1</tag>Lemma</toctitle><para xml:id=\"A1.SS1.p1\"><p xml:id=\"A1.SS1.p1.1\">Text.</p>\
         </para></subsection></appendix>",
      ),
    ],
  );
  assert_clean(
    repro!("sectioning-frontmatter/ieeetran_compsoc_conference_headings_are_arabic.tex"),
    &[],
    &[
      (
        "section",
        "S1",
        "<section inlist=\"toc\" labels=\"LABEL:s1\" xml:id=\"S1\"><tags><tag>1.</tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">§1</tag></tags><title><tag close=\" \">1.</tag>\
         <text font=\"smallcaps\" xml:id=\"S1.1\">Intro</text></title><toctitle>\
         <tag close=\" \">1</tag>Intro</toctitle><subsection inlist=\"toc\" labels=\"LABEL:s11\" xml:id=\"S1.SS1\">\
         <tags><tag>1.1.</tag><tag role=\"refnum\">1.1</tag><tag role=\"typerefnum\">§1.1</tag></tags><title>\
         <tag close=\" \">1.1.</tag><text font=\"italic\" xml:id=\"S1.SS1.1\">Sub</text></title><toctitle>\
         <tag close=\" \">1.1</tag>Sub</toctitle>\
         <subsubsection inlist=\"toc\" labels=\"LABEL:s111\" xml:id=\"S1.SS1.SSS1\"><tags><tag>1.1.1.</tag>\
         <tag role=\"refnum\">1.1.1</tag><tag role=\"typerefnum\">§1.1.1</tag></tags><title>\
         <tag close=\" \">1.1.1.</tag>Subsub</title><toctitle><tag close=\" \">1.1.1</tag>Subsub</toctitle>\
         <para xml:id=\"S1.SS1.SSS1.p1\">\
         <p xml:id=\"S1.SS1.SSS1.p1.1\">Refs: <ref labelref=\"LABEL:s1\"/>, <ref labelref=\"LABEL:s11\"/>, \
         <ref labelref=\"LABEL:s111\"/>, <ref labelref=\"LABEL:a1\"/>, <ref labelref=\"LABEL:a11\"/>.</p>\
         </para></subsubsection></subsection></section>",
      ),
      (
        "appendix",
        "A1",
        "<appendix inlist=\"toc\" labels=\"LABEL:a1\" xml:id=\"A1\"><tags><tag>Appendix A</tag>\
         <tag role=\"refnum\">A</tag><tag role=\"typerefnum\">Appendix A</tag></tags><title>\
         <tag close=\" \">Appendix A</tag>Proof</title><toctitle><tag close=\" \">A</tag>Proof</toctitle>\
         <subsection inlist=\"toc\" labels=\"LABEL:a11\" xml:id=\"A1.SS1\"><tags><tag>A.1.</tag>\
         <tag role=\"refnum\">A.1</tag><tag role=\"typerefnum\">§A.1</tag></tags><title><tag close=\" \">A.1.</tag>\
         <text font=\"italic\" xml:id=\"A1.SS1.1\">Lemma</text></title><toctitle>\
         <tag close=\" \">A.1</tag>Lemma</toctitle><para xml:id=\"A1.SS1.p1\"><p xml:id=\"A1.SS1.p1.1\">Text.</p>\
         </para></subsection></appendix>",
      ),
    ],
  );
}

/// The options of a `\usepackage[<options>]{caption}` after subcaption, whose binding loaded caption first, are
/// caption's keys, as caption3.sty:416-429 runs a repeat load's: the caption at 90 % with a bold label and ". "
/// (pdflatex `\f@size` 9, CMBX9; witness 1602.00134). caption's v1 options are first-load-only there (caption's
/// "footnotesize undefined" Error), an unknown-key Warning each here, the caption unchanged. Red (78c r1): the options
/// dropped silently. Repros captions-floats/caption_options_after_subcaption_apply,
/// captions-floats/caption_v1_options_after_subcaption_apply.
#[test]
fn caption_options_after_subcaption_apply() {
  assert_clean(
    repro!("captions-floats/caption_options_after_subcaption_apply.tex"),
    &[],
    &[(
      "figure",
      "S0.F1",
      "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag>\
         <tag role=\"typerefnum\">Figure 1</tag></tags><p xml:id=\"S0.F1.1\">A</p><toccaption>\
         <tag close=\" \">1</tag>Schematic.</toccaption><caption><tag close=\". \">\
         <text font=\"bold\" fontsize=\"90%\" xml:id=\"S0.F1.2\">Figure 1</text></tag>\
         <text fontsize=\"90%\" xml:id=\"S0.F1.3\">Schematic.</text></caption></figure>",
    )],
  );
  let (log, xml) = convert_with(
    repro!("captions-floats/caption_v1_options_after_subcaption_apply.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag>\
     <tag role=\"typerefnum\">Figure 1</tag></tags><p xml:id=\"S0.F1.1\">A</p><toccaption>\
     <tag close=\" \">1</tag>Schematic.</toccaption><caption><tag close=\": \">Figure 1</tag>Schematic.</caption>\
     </figure>",
  );
}

/// subfig's `caption=false` and floatrow load caption's kernel alone (caption3: subfig.sty:107-128, floatrow.sty:39),
/// so a later `\usepackage[bf,small]{caption}` is caption's first load and its v1 options are valid (caption.sty:114-147):
/// the caption at 90 % with a bold label, no diagnostics (pdflatex `\f@size` 9, CMBX9; 2509.03211, 2409.17596,
/// 2304.03621). Red (78c r2): an unknown-key Warning each, the options dropped. Repros
/// captions-floats/caption_v1_options_after_caption3_apply, captions-floats/caption_v1_options_after_floatrow_apply.
#[test]
fn caption_v1_options_after_caption3_apply() {
  for tex in [
    repro!("captions-floats/caption_v1_options_after_caption3_apply.tex"),
    repro!("captions-floats/caption_v1_options_after_floatrow_apply.tex"),
  ] {
    let (log, xml) = convert_with(tex, Some(PRELOAD));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_element(
      &xml,
      "caption",
      &[],
      "<caption><tag close=\": \"><text font=\"bold\" fontsize=\"90%\" xml:id=\"S0.F1.2\">Figure 1</text></tag>\
       <text fontsize=\"90%\" xml:id=\"S0.F1.3\">Schematic.</text></caption>",
    );
  }
}

/// acmart's default citation style is `\citestyle{acmnumeric}` (acmart.cls:385-393): "[1, 2]" (witness 2112.15328);
/// `\citestyle{acmauthoryear}` (:379-384) gives "[Able 2001]". Red (int43a; Perl acmart.cls.ltxml:23 only loads
/// natbib): "(Able, 2001; …)". Repros index-bib/acmart_citations_are_numeric_by_default,
/// index-bib/acmart_citestyle_acmauthoryear.
#[test]
fn acmart_citations_are_numeric_by_default() {
  assert_clean(
    repro!("index-bib/acmart_citations_are_numeric_by_default.tex"),
    &[],
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">See <cite class=\"ltx_citemacro_citep\">[\
         <bibref bibrefs=\"b,a\" separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>.</p>",
    )],
  );
  assert_clean(
    repro!("index-bib/acmart_citestyle_acmauthoryear.tex"),
    &[],
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">See <cite class=\"ltx_citemacro_citep\">[\
         <bibref bibrefs=\"a\" separator=\";\" show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase> \
         </bibrefphrase></bibref>]</cite>.</p>",
    )],
  );
}

/// `\hhline` rules the columns it names (hhline.sty:61-96): `-` a rule, `=` a double one, `~` none, `*{4}{-|}` four
/// rules; under colortbl a `>{<code>}` is the rules' colour code, no column and no warning (colortbl.sty:364-369).
/// Red (int43a; Perl hhline.sty.ltxml:24 `\hline`): every column ruled once. Repros
/// alignment-bindings/hhline_rules_the_columns_it_names, alignment-bindings/hhline_colortbl_color_code_is_skipped.
/// The `thead="row"` on the plain data cells a, b, e, f, i, j (and a, c) is the kernel's header guess
/// (`ltx_guessed_headers`) reading the new borders, not hhline's and not the PDF's: pinned as the heuristic gives it
/// today (KNOWN_PERL_ERRORS #1024 open; a double rule after a header row loses its `<thead>` the same way, as
/// `\hline\hline` does: 1706.03044, 2207.13493).
#[test]
fn hhline_rules_the_columns_it_names() {
  assert_clean(
    repro!("alignment-bindings/hhline_rules_the_columns_it_names.tex"),
    &[],
    &[(
      "tabular",
      "p1.1",
      "<tabular class=\"ltx_guessed_headers\" vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr xml:id=\"p1.1.1\">\
         <td align=\"center\" border=\"l r t\" thead=\"row\" xml:id=\"p1.1.1.1\">a</td>\
         <td align=\"center\" border=\"r t\" thead=\"row\" xml:id=\"p1.1.1.2\">b</td>\
         <td align=\"center\" border=\"r t\" xml:id=\"p1.1.1.3\">c</td>\
         <td align=\"center\" border=\"r t\" xml:id=\"p1.1.1.4\">d</td></tr><tr xml:id=\"p1.1.2\">\
         <td align=\"center\" border=\"l r\" thead=\"row\" xml:id=\"p1.1.2.1\">e</td>\
         <td align=\"center\" border=\"r\" thead=\"row\" xml:id=\"p1.1.2.2\">f</td>\
         <td align=\"center\" border=\"r t\" xml:id=\"p1.1.2.3\">g</td>\
         <td align=\"center\" border=\"r t\" xml:id=\"p1.1.2.4\">h</td></tr><tr xml:id=\"p1.1.3\">\
         <td align=\"center\" border=\"l r tt\" thead=\"row\" xml:id=\"p1.1.3.1\">i</td>\
         <td align=\"center\" border=\"r tt\" thead=\"row\" xml:id=\"p1.1.3.2\">j</td>\
         <td align=\"center\" border=\"r\" xml:id=\"p1.1.3.3\">k</td>\
         <td align=\"center\" border=\"r t\" xml:id=\"p1.1.3.4\">l</td></tr></tbody></tabular>",
    )],
  );
  assert_clean(
    repro!("alignment-bindings/hhline_colortbl_color_code_is_skipped.tex"),
    &[],
    &[(
      "tabular",
      "p1.1",
      "<tabular class=\"ltx_guessed_headers\" vattach=\"middle\" xml:id=\"p1.1\"><tbody><tr xml:id=\"p1.1.1\">\
       <td align=\"center\" thead=\"row\" xml:id=\"p1.1.1.1\">a</td><td align=\"center\" xml:id=\"p1.1.1.2\">b</td>\
       </tr><tr xml:id=\"p1.1.2\"><td align=\"center\" border=\"t\" thead=\"row\" xml:id=\"p1.1.2.1\">c</td>\
       <td align=\"center\" xml:id=\"p1.1.2.2\">d</td></tr></tbody></tabular>",
    )],
  );
}

/// AMS-TeX sets its tags on the left unless `\TagsOnRight` (amstex.tex:587-590, `\TagsOnLeft` at load): the document
/// carries `ltx_leqno`, as pdftex sets "(1)" left of "a=b"; `\TagsOnRight` clears it. Red (int43a; Perl
/// AmSTeX.pool.ltxml:83-84 no-ops): no `ltx_leqno`. Repros amstex/amstex_tags_are_on_the_left_by_default,
/// amstex/amstex_tags_on_right.
#[test]
fn amstex_tags_are_on_the_left_by_default() {
  let xml = assert_clean(
    repro!("amstex/amstex_tags_are_on_the_left_by_default.tex"),
    &[],
    &[],
  );
  assert!(
    xml.contains("<document xmlns=\"http://dlmf.nist.gov/LaTeXML\" class=\"ltx_leqno\">"),
    "{xml}"
  );
  let xml = assert_clean(repro!("amstex/amstex_tags_on_right.tex"), &[], &[]);
  assert!(
    xml.contains("<document xmlns=\"http://dlmf.nist.gov/LaTeXML\">"),
    "{xml}"
  );
}

/// AMS-TeX's `@"` connector (amstex.tex:173-190) spaces the nested quotes after it: “ ‘Hi’ ” with six-per-em spaces
/// for its kerns, as pdftex. Red (int43a): `Error:undefined:@"` twice. Repro
/// amstex/amstex_quote_connector_spaces_the_quotes (KNOWN_PERL_ERRORS #1018).
#[test]
fn amstex_quote_connector_spaces_the_quotes() {
  assert_clean(
    repro!("amstex/amstex_quote_connector_spaces_the_quotes.tex"),
    &[],
    &[(
      "p",
      "id1.1",
      "<p xml:id=\"id1.1\">He said \u{201C}\u{2006}\u{2018}Hi\u{2019}\u{2006}\u{201D} twice.</p>",
    )],
  );
}

/// The acknowledgements carry the heading the class prints (jmlr.cls:595 "Acknowledgments"; a jmlr2e.sty copy's own:
/// 2511.10879's "Acknowledgments and Disclosure of Funding", 1512.04829's "Acknowledgments"). Red (int43a): the slug
/// "acknowledgments-disclosure-of-funding", which the XSLT printed. Repros class-bindings/jmlr_acks.
#[test]
fn jmlr_acknowledgements_carry_the_printed_heading() {
  let tex = repro!("class-bindings/jmlr_acks/jmlr2e_acks_heading_is_the_copys.tex");
  for (copy, name) in [
    (
      repro!("class-bindings/jmlr_acks/new/jmlr2e.sty"),
      "Acknowledgments and Disclosure of Funding",
    ),
    (
      repro!("class-bindings/jmlr_acks/old/jmlr2e.sty"),
      "Acknowledgments",
    ),
  ] {
    assert_clean(tex, &[("jmlr2e.sty", copy)], &[(
      "acknowledgements",
      "acknowledgements1",
      &format!(
        "<acknowledgements inlist=\"toc\" name=\"{name}\" xml:id=\"acknowledgements1\">We thank Ann.</acknowledgements>"
      ),
    )]);
  }
  // Without a shipped copy, the heading of most copies (45 of 64) and of jmlr.cls:595 (rev78c).
  assert_clean(tex, &[], &[(
    "acknowledgements",
    "acknowledgements1",
    "<acknowledgements inlist=\"toc\" name=\"Acknowledgments\" xml:id=\"acknowledgements1\">We thank Ann.\
     </acknowledgements>",
  )]);
  assert_clean(
    repro!("class-bindings/jmlr_acks/jmlr_cls_acks_heading.tex"),
    &[],
    &[(
      "acknowledgements",
      "acknowledgements1",
      "<acknowledgements inlist=\"toc\" name=\"Acknowledgments\" xml:id=\"acknowledgements1\">We thank Ann.\
       </acknowledgements>",
    )],
  );
}

/// svmult's `\maketitle` numbers a contribution under a `\title` as a chapter (svmult.cls:2141-2152,
/// `\refstepcounter{chapter}`): "1.1 First", `\ref` "1.1"; the main-matter `\appendix` is a `\par` (:713): "1.2
/// Proofs", a section. Under a `\title*` the sections are "1" and "2". Red (int43a; Perl svmult.cls.ltxml:40-42):
/// "0.1", an `ltx:appendix` ".2" (witness 1801.05601). Repros sectioning-frontmatter/svmult_maketitle_numbers_the_contribution,
/// sectioning-frontmatter/svmult_title_star_numbers_sections_alone.
#[test]
fn svmult_maketitle_numbers_the_contribution() {
  assert_clean(
    repro!("sectioning-frontmatter/svmult_maketitle_numbers_the_contribution.tex"),
    &[],
    &[(
      "section",
      "Ch1.S2",
      "<section inlist=\"toc\" xml:id=\"Ch1.S2\"><tags><tag>1.2</tag><tag role=\"refnum\">1.2</tag>\
         <tag role=\"typerefnum\">\u{00A7}1.2</tag></tags><title><tag close=\" \">1.2</tag>Proofs</title>\
         <para xml:id=\"Ch1.S2.p1\"><p xml:id=\"Ch1.S2.p1.1\">Proof.</p></para></section>",
    )],
  );
  assert_clean(
    repro!("sectioning-frontmatter/svmult_title_star_numbers_sections_alone.tex"),
    &[],
    &[(
      "section",
      "Ch0.S2",
      "<section inlist=\"toc\" xml:id=\"Ch0.S2\"><tags><tag>2</tag><tag role=\"refnum\">2</tag>\
         <tag role=\"typerefnum\">\u{00A7}2</tag></tags><title><tag close=\" \">2</tag>Proofs</title>\
         <para xml:id=\"Ch0.S2.p1\"><p xml:id=\"Ch0.S2.p1.1\">Proof.</p></para></section>",
    )],
  );
}
