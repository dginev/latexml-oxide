//! Red/green guards for PLACEFIX: DG's 2026-11-19 rulings on run 397's misplaced-element clusters
//! (`PLACE397/REPORT.md`). An undefined BibTeX month macro is a Warning, as bibtex(1) has it (OD #1096); a theorem or
//! proof in an abstract is dissolved into it (OD #1097; witnesses 1908.02189 T1, 0804.0728 T2);
//! nomencl's list in a float's box keeps its box (OD #1098; 1410.5511); a bibliography inside a float or a heading floats out to
//! the narrative level (OD #1099; cond-mat0108156, hep-ph0004227). Every repro is checked against pdflatex (its header).
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

/// block-model/theorem_in_abstract_stays: in `{abstract}` a labelled theorem (referenced from the body) followed by
/// text, an equation, a list and a quote, two theorems in a row, a proof and a `\newtheorem*{keywords}` (pdflatex 0
/// errors). Before: 5 Errors "isn't allowed in <ltx:abstract>"; round 1 wrapped the rest of the abstract in an
/// `ltx:inline-logical-block`. Now no theorem element: each title opens its first paragraph, which takes the id.
#[test]
fn theorem_in_abstract_is_dissolved() {
  let (log, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/block-model/theorem_in_abstract_stays.tex"),
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    !xml.contains("<theorem") && !xml.contains("<proof") && !xml.contains("inline-logical-block"),
    "{xml}"
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>We prove one thing.</p><p xml:id="Thmtheorem1"><text class="ltx_runin"><text font="bold">Theorem 1</text><text font="bold">.</text></text> <text font="italic">Every <Math mode="inline" tex="x" text="x" xml:id="Thmtheorem1.p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> is <Math mode="inline" tex="x" text="x" xml:id="Thmtheorem1.p1.m2"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>.</text></p><p>Then a display</p><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="x=x" text="x = x" xml:id="S0.E1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math></equation><p>and a list:</p><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.p1"><p>One.</p></para></item></enumerate><quote><p>Quoted.</p></quote><p xml:id="Thmtheorem2"><text class="ltx_runin"><text font="bold">Theorem 2</text><text font="bold">.</text></text> <text font="italic">Two.</text></p><p xml:id="Thmtheorem3"><text class="ltx_runin"><text font="bold">Theorem 3</text><text font="bold">.</text></text> <text font="italic">Three.</text></p><p><text class="ltx_runin" font="italic">Proof.</text><text font="italic"> </text>Trivial.∎</p><p xml:id="Thmkeywordsx1"><text class="ltx_runin"><text font="bold">Keywords</text><text font="bold">.</text></text> <text font="italic">identity, reflexivity</text></p></abstract>"#,
  );
  // The dissolved theorem keeps its id only: the abstract admits no `labels`/`ltx:tags` (DG 2026-11-20), so the body's
  // `\ref{t}` finds no target and stays unresolved.
  let post = convert_and_post_clean(
    "../tools/perfect_kernel/repros/block-model/theorem_in_abstract_stays.tex",
  );
  assert!(
    post.contains(r#"labelref="LABEL:t""#) && !post.contains(r#"idref="Thmtheorem1""#),
    "{post}"
  );
}

/// block-model/theorem_in_abstract_proclaim: amsppt's `\proclaim{Theorem}` in `\abstract` (tex 0 errors; math0111087):
/// a display title, not a run-in one, so it is made bold when run in (the other path of the dissolution's title).
#[test]
fn proclaim_in_abstract_is_dissolved() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/theorem_in_abstract_proclaim.tex"
    ),
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>We prove the following.</p><p xml:id="Thmx1"><text class="ltx_runin" font="bold">Theorem</text> Every <Math mode="inline" tex="x" text="x" xml:id="Thmx1.p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> is <Math mode="inline" tex="x" text="x" xml:id="Thmx1.p1.m2"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>.</p><p>More.</p></abstract>"#,
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

/// block-model/nomenclature_in_a_minipage: the same list in a `{minipage}` in the text (pdflatex 0 errors) floats out
/// to the section's glossary, error-free, as on base: the in-place list is for a float only (review S1).
#[test]
fn nomenclature_in_a_minipage_is_the_sectional_glossary() {
  let (log, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/block-model/nomenclature_in_a_minipage.tex"),
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  // The document's own child (two-space indent), after the paragraph, as on base.
  assert!(xml.contains("</para>\n  <glossary "), "{xml}");
  assert!(!xml.contains("ltx_nomenclature"), "{xml}");
  assert_element(
    &xml,
    "glossary",
    &[],
    r#"<glossary lists="nomenclature" role="nomenclature" xml:id="glo.nomenclature"><title>Nomenclature</title></glossary>"#,
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

/// block-model/theorem_in_unended_abstract{,_group}: an abstract still open at `\end{document}` — a brace-less
/// `\abstract` no heading ends (1108.5171), an `\abstract{` whose group never closes (math0006149) — swallowed the main
/// matter: its theorems stay theorems, with base's Error (DG 2026-11-20). Two terminal paths (the body's own loop, and
/// `until_terminal_inside_group`). `\abstract{…}` closed by its group ends there; the theorem after it is the body's.
#[test]
fn theorem_in_unended_abstract_stays_a_theorem() {
  for (repro, warnings) in [
    ("theorem_in_unended_abstract", 0),
    ("theorem_in_unended_abstract_group", 1),
  ] {
    let (log, xml) = convert_with(
      &std::fs::read_to_string(format!(
        "../tools/perfect_kernel/repros/block-model/{repro}.tex"
      ))
      .unwrap(),
      None,
    );
    assert_eq!(error_count(&log), 1, "{repro}: {log}");
    assert!(
      log.contains("<ltx:theorem> isn't allowed in <ltx:abstract>"),
      "{repro}: {log}"
    );
    assert_eq!(warning_count(&log), warnings, "{repro}: {log}");
    assert_element(
      &xml,
      "abstract",
      &[],
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Summary.</p><theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" xml:id="Thmtheorem1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold">Theorem 1</text></tag></title><para xml:id="Thmtheorem1.p1"><p><text font="italic">Body theorem.</text></p></para></theorem><p>Text.</p></abstract>"#,
    );
  }
  let (log, xml) = convert_with(
    r"\documentclass{article}\newtheorem{theorem}{Theorem}\begin{document}
\abstract{Summary.} \begin{theorem}Body theorem.\end{theorem} Text.\end{document}",
    None,
  );
  assert_eq!(error_count(&log) + warning_count(&log), 0, "{log}");
  assert!(xml.contains("</abstract>\n  <theorem "), "{xml}");
}
