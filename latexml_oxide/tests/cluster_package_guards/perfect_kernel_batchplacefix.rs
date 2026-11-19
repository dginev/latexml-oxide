//! Red/green guards for PLACEFIX: DG's 2026-11-19 rulings on run 397's misplaced-element clusters
//! (`PLACE397/REPORT.md`). An undefined BibTeX month macro is a Warning, as bibtex(1) has it (OD #1095); a theorem or
//! proof in an abstract stays there in an `ltx:inline-logical-block` (OD #1096; witnesses 1908.02189 T1, 0804.0728 T2);
//! nomencl's list in a box keeps its box (OD #1097; 1410.5511); a bibliography inside a float or a heading floats out to
//! the narrative level (OD #1098; cond-mat0108156, hep-ph0004227). Every repro is checked against pdflatex (its header).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

use crate::cluster::{convert_and_post_clean, convert_and_post_logging, convert_to_xml};

/// index-bib/bibtex_month_undefined: `month = june` under `plain` (bibtex: "Warning--string name "june" is undefined").
/// Before: an Error (Perl BibTeX.pm:307-310); now a Warning, the entry kept.
#[test]
fn bibtex_undefined_month_is_a_warning() {
  let (html, log) =
    convert_and_post_logging("../tools/perfect_kernel/repros/index-bib/bibtex_month_undefined.tex");
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains("The BibTeX macro 'june' is not defined"),
    "{log}"
  );
  assert_element(
    &html,
    "bibitem",
    &[],
    r#"<bibitem class="ltx_bib_article" fragid="bib.bib1" key="a" type="article" xml:id="bib.bib1"><tags><tag class="ltx_bib_number" role="number">1</tag><tag class="ltx_bib_author" role="authors">Author</tag><tag class="ltx_bib_year" role="year">2001</tag><tag class="ltx_bib_title" role="title">Title</tag><tag class="ltx_bib_key" close="]" open="[" role="refnum">1</tag></tags><bibblock xml:space="preserve"><text class="ltx_bib_author">A. Author</text><text class="ltx_bib_year"> (2001)</text></bibblock><bibblock xml:space="preserve"><text class="ltx_bib_title">Title</text>.</bibblock><bibblock xml:space="preserve"><text class="ltx_bib_journal">Journal</text>.</bibblock><bibblock class="ltx_bib_cited">Cited by: <ref idref="p1" show="typerefnum">p1</ref>.</bibblock></bibitem>"#,
  );
}

/// block-model/theorem_in_abstract_stays: a theorem (labelled, referenced from the body), a proof and a
/// `\newtheorem*{keywords}` in `{abstract}` (pdflatex 0 errors). Before: 3 Errors "isn't allowed in <ltx:abstract>".
#[test]
fn theorem_in_abstract_stays() {
  let (log, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/block-model/theorem_in_abstract_stays.tex"),
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>We prove one thing.</p><inline-logical-block><theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" labels="LABEL:t" xml:id="Thmtheorem1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold">Theorem 1</text></tag><text font="bold">.</text></title><para xml:id="Thmtheorem1.p1"><p><text font="italic">Every <Math mode="inline" tex="x" text="x" xml:id="Thmtheorem1.p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> is <Math mode="inline" tex="x" text="x" xml:id="Thmtheorem1.p1.m2"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>.</text></p></para></theorem><proof><title class="ltx_runin" font="italic">Proof.</title><para xml:id="abstract1.p1"><p>Trivial.∎</p></para></proof><theorem class="ltx_theorem_keywords" inlist="thm theorem:keywords" xml:id="Thmkeywordsx1"><tags><tag>Keywords</tag><tag role="typerefnum">Keywords</tag></tags><title class="ltx_runin"><tag><text font="bold">Keywords</text></tag><text font="bold">.</text></title><para xml:id="Thmkeywordsx1.p1"><p><text font="italic">identity, reflexivity</text></p></para></theorem></inline-logical-block></abstract>"#,
  );
}

/// block-model/nomenclature_in_a_box: `\printnomenclature` in `{framed}` in a `{table}` (pdflatex 0 errors). Before: 1
/// Error "<ltx:glossary> isn't allowed in <ltx:block>"; now the bold heading and the post-filled description in the box.
#[test]
fn nomenclature_in_a_box_stays_in_the_box() {
  let xml =
    convert_and_post_clean("../tools/perfect_kernel/repros/block-model/nomenclature_in_a_box.tex");
  assert_element(
    &xml,
    "table",
    &[],
    r##"<table fragid="S1.tab1" placement="t" xml:id="S1.tab1"><block cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle"><p><text font="bold">Nomenclature</text></p><description class="ltx_nomenclature" fragid="glo.nomenclature" xml:id="glo.nomenclature"><item fragid="glo.nomenclature.nomencl.1" xml:id="glo.nomenclature.nomencl.1"><tags><tag><Math mode="inline" tex="Q" text="Q" xml:id="S1.p1.m3a"><XMath><XMTok font="italic" role="UNKNOWN">Q</XMTok></XMath></Math></tag></tags><para><p>heat</p></para></item></description></block></table>"##,
  );
}

/// block-model/bibliography_floats_out_of_a_float: a `{thebibliography}` in a `{table}` and one in a `\subsection*`'s
/// argument (pdflatex 0 errors). Before: 3 Errors "<ltx:bibliography> isn't allowed in <ltx:table>/<ltx:title>".
#[test]
fn bibliography_floats_out_of_a_float() {
  let xml = convert_to_xml(
    "../tools/perfect_kernel/repros/block-model/bibliography_floats_out_of_a_float.tex",
  );
  assert_element(
    &xml,
    "section",
    &[],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Intro</title><para xml:id="S1.p1"><p>Body text <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite> and <cite class="ltx_citemacro_cite">[<bibref bibrefs="b" separator="," yyseparator=","/>]</cite>.</p></para><table placement="h" xml:id="S1.tab1"/><bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, Title.</bibblock></bibitem></biblist></bibliography><para xml:id="S1.p2"><p>After the table.</p></para><subsection xml:id="S1.SSx1"><title/><bibliography inlist="toc" xml:id="biba"><title>References</title><biblist><bibitem key="b" xml:id="biba.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> B. Author, Book.</bibblock></bibitem></biblist></bibliography><para xml:id="S1.SSx1.p1"><p>After the heading.</p></para></subsection></section>"#,
  );
}
