//! Red/green guards for batch AMSPPT: amsppt's heads and theorem-like blocks, ported from Perl amsppt.sty.ltxml:94-235.
//! A head opens its unit and leaves it open for its body (the next head at its level or above, or the document's end,
//! closes it); `\proclaim`, `\definition`, `\remark`, `\example` and `\demo` take their id from the one `theorem` counter.
use super::perfect_kernel_batch57::assert_elements;

const AR5IV: &str = "ar5iv.sty";

/// A repro and its whole elements: (tag, id, markup).
type Row = (
  &'static str,
  &'static [(&'static str, &'static str, &'static str)],
);

/// Each repro converts once, 0 errors / 0 warnings; the unit holding its body and blocks is asserted whole. Rows: a
/// `\proclaim` of two paragraphs in its section (math9201249); every head (`\head`, `\subhead`, `\subsubhead`,
/// `\subheading{}`, `\specialhead`) holding its body and the lower units; the five block kinds numbered by one counter;
/// `\Refs` as back matter after the open heads, and a `\specialhead References` heading the bibliography (math0304264).
#[test]
fn amsppt_heads_and_theorems() {
  let rows: &[Row] = &[
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_proclaim_has_its_id.tex"),
      &[(
        "section",
        "C0.Sx1",
        r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Intro</title><theorem class="ltx_theorem_proclaim" xml:id="Thmx1"><title>Theorem 1</title><para xml:id="Thmx1.p1"><p xml:id="Thmx1.p1.1">First paragraph.</p></para><para xml:id="Thmx1.p2"><p xml:id="Thmx1.p2.1">Second paragraph with <Math mode="inline" tex="x" text="x" xml:id="Thmx1.p2.m1"><XMath xml:id="Thmx1.p2.m1.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>.</p></para></theorem><para xml:id="C0.Sx1.p1"><p xml:id="C0.Sx1.p1.1">After the theorem.</p></para></section>"#,
      )],
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/amstex/amsppt_heads_hold_their_bodies.tex"
      ),
      &[
        (
          "section",
          "C0.Sx1",
          r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. First</title><para xml:id="C0.Sx1.p1"><p xml:id="C0.Sx1.p1.1">Body of the first section.</p></para><subsection inlist="toc" xml:id="C0.Sx1.SSx1"><title>1.1. Sub</title><para xml:id="C0.Sx1.SSx1.p1"><p xml:id="C0.Sx1.SSx1.p1.1">Body of the subsection.</p></para><subsubsection inlist="toc" xml:id="C0.Sx1.SSx1.SSSx1"><title>1.1.1. Subsub</title><para xml:id="C0.Sx1.SSx1.SSSx1.p1"><p xml:id="C0.Sx1.SSx1.SSSx1.p1.1">Body of the subsubsection.</p></para></subsubsection></subsection></section>"#,
        ),
        (
          "section",
          "C0.Sx2",
          r#"<section inlist="toc" xml:id="C0.Sx2"><title>2. Second</title><para xml:id="C0.Sx2.p1"><p xml:id="C0.Sx2.p1.1">Body of the second section.</p></para><subsection inlist="toc" xml:id="C0.Sx2.SSx1"><title>2.1. Braced</title><para xml:id="C0.Sx2.SSx1.p1"><p xml:id="C0.Sx2.SSx1.p1.1">Body of the braced subsection.</p></para></subsection></section>"#,
        ),
        (
          "chapter",
          "Cx1",
          r#"<chapter inlist="toc" xml:id="Cx1"><title>Appendix</title><para xml:id="Cx1.p1"><p xml:id="Cx1.p1.1">Body of the appendix.</p></para><section inlist="toc" xml:id="Cx1.Sx1"><title>A.1. Inside</title><para xml:id="Cx1.Sx1.p1"><p xml:id="Cx1.Sx1.p1.1">Body of the appendix section.</p></para></section></chapter>"#,
        ),
      ],
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/amstex/amsppt_theorem_kinds_share_the_counter.tex"
      ),
      &[(
        "section",
        "C0.Sx1",
        r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Intro</title><theorem class="ltx_theorem_definition" xml:id="Thmx1"><title>Definition 1</title><para xml:id="Thmx1.p1"><p xml:id="Thmx1.p1.1">A definition.</p></para></theorem><theorem class="ltx_theorem_remark" xml:id="Thmx2"><title>Remark 2</title><para xml:id="Thmx2.p1"><p xml:id="Thmx2.p1.1">A remark.</p></para></theorem><theorem class="ltx_theorem_example" xml:id="Thmx3"><title>Example 3</title><para xml:id="Thmx3.p1"><p xml:id="Thmx3.p1.1">An example.</p></para></theorem><theorem class="ltx_theorem_demonstration" xml:id="Thmx4"><title>Proof</title><para xml:id="Thmx4.p1"><p xml:id="Thmx4.p1.1">A proof.</p></para></theorem><theorem class="ltx_theorem_proclaim" xml:id="Thmx5"><title>Theorem 4</title><para xml:id="Thmx5.p1"><p xml:id="Thmx5.p1.1">A theorem.</p></para></theorem></section>"#,
      )],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_refs_close_the_heads.tex"),
      &[
        (
          "section",
          "C0.Sx1",
          r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Intro</title><subsection inlist="toc" xml:id="C0.Sx1.SSx1"><title>1.1. Sub</title><para xml:id="C0.Sx1.SSx1.p1"><p xml:id="C0.Sx1.SSx1.p1.1">Text.</p></para></subsection></section>"#,
        ),
        (
          "bibliography",
          "bib",
          r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="1" xml:id="bibliography0.bibx1"><tags><tag role="refnum">1</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">Title</text><text font="italic" xml:id="bibliography0.bibx1.3">.</text></bibblock></bibitem></biblist></bibliography>"#,
        ),
      ],
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/amstex/amsppt_specialhead_heads_the_references.tex"
      ),
      &[
        (
          "chapter",
          "Cx1",
          r#"<chapter inlist="toc" xml:id="Cx1"><title>1. Intro</title><para xml:id="Cx1.p1"><p xml:id="Cx1.p1.1">Text.</p></para></chapter>"#,
        ),
        (
          "bibliography",
          "bib",
          r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="1" xml:id="bibliography0.bibx1"><tags><tag role="refnum">1</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">Title</text><text font="italic" xml:id="bibliography0.bibx1.3">.</text></bibblock></bibitem></biblist></bibliography>"#,
        ),
      ],
    ),
  ];
  for (tex, expected) in rows {
    let xml = assert_elements(tex, AR5IV, (0, 0), expected);
    // no paragraph is left at the document's level (whose ids would be `p1`, `p2`, …)
    assert!(!xml.contains(r#"xml:id="p1""#), "{xml}");
  }
}
