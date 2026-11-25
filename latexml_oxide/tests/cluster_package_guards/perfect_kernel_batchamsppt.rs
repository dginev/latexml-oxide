//! Red/green guards for batch AMSPPT: amsppt's heads and theorem-like blocks, ported from Perl amsppt.sty.ltxml:94-235.
//! A head opens its unit and leaves it open for its body (the next head at its level or above, or the document's end,
//! closes it); `\proclaim`, `\definition`, `\remark`, `\example` and `\demo` take their id from the one `theorem` counter.
use super::perfect_kernel_batch57::{assert_elements, assert_elements_with};

const AR5IV: &str = "ar5iv.sty";

/// A repro and its whole elements: (tag, id, markup).
type Row = (
  &'static str,
  &'static [(&'static str, &'static str, &'static str)],
);

/// Each repro converts once, 0 errors / 0 warnings; the unit holding its body and blocks is asserted whole. Rows: a
/// `\proclaim` of two paragraphs in its section (math9201249); every head (`\head`, `\subhead`, `\subsubhead`,
/// `\subheading{}`, `\specialhead`) holding its body and the lower units; the five block kinds numbered by one counter;
/// `\Refs` as back matter after the open heads, and a `\specialhead References` heading the bibliography (math0304264);
/// a `\ref` field digested as amsppt's `\makerefbox` boxes it, its `\global\@fwdreftrue` executed (math0610862), and
/// `\paper`'s `\it` held in its field: the final period upright (amsppt.sty:956-965; it was italic before AMSPPTRES);
/// a field of a rule kept, `\bysame`'s `\hrulefill` (TeX keeps a box of width), and one ended by `\endgraf` trimmed;
/// `\roster` keeping its items, tagged as amsppt.sty:723-736 prints them (`\item`, `\item"(b)"`, `\item[5]`, the document's
/// `\therosteritem`), nested in a `\proclaim`, and `\roster\runinitem` (alg-geom9502009).
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
          r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="1" xml:id="bibliography0.bibx1"><tags><tag role="refnum">1</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">Title</text>.</bibblock></bibitem></biblist></bibliography>"#,
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
          r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="1" xml:id="bibliography0.bibx1"><tags><tag role="refnum">1</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">Title</text>.</bibblock></bibitem></biblist></bibliography>"#,
        ),
      ],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_bibfield_is_digested.tex"),
      &[(
        "bibliography",
        "bib",
        r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="??" xml:id="bibliography0.bibx1"><tags><tag role="refnum">??</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">Title</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx1.3">J</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx1.4">1990</text>).</bibblock></bibitem></biblist></bibliography>"#,
      )],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_rule_field_is_kept.tex"),
      &[(
        "bibliography",
        "bib",
        r##"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="A" xml:id="bibliography0.bibx1"><tags><tag role="refnum">A</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">First</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx1.3">J</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx1.4">1990</text>).</bibblock></bibitem><bibitem key="B" xml:id="bibliography0.bibx2"><tags><tag role="refnum">B</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx2.1"><text width="30.0pt" xml:id="bibliography0.bibx2.1.1"><rule class="ltx_filled_leader" height="1px" width="30.0pt"/></text> </text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx2.2">Second</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx2.3">K</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx2.4">1991</text>).</bibblock></bibitem><bibitem key="C" xml:id="bibliography0.bibx3"><tags><tag role="refnum">C</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx3.1"><rule depth="0.0pt" height="0.4pt" width="30.0pt"/></text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx3.2">Third</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx3.3">L</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx3.4">1992</text>).</bibblock></bibitem><bibitem key="D" xml:id="bibliography0.bibx4"><tags><tag role="refnum">D</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx4.1">D. Fourth</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx4.2">Fourth</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx4.3">M</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx4.4">1993</text>).</bibblock></bibitem><bibitem key="E" xml:id="bibliography0.bibx5"><tags><tag role="refnum">E</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx5.1">E. Fifth</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx5.2">Fifth</text>, <text class="ltx_bib_paperinfo" xml:id="bibliography0.bibx5.3">Preprint, available via <break/>a web page</text>.</bibblock></bibitem></biblist></bibliography>"##,
      )],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_roster_keeps_its_items.tex"),
      &[(
        "section",
        "C0.Sx1",
        r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Lists</title><para xml:id="C0.Sx1.p1"><enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>(1)</tag></tags><para xml:id="Ix1.ix1.p1"><p xml:id="Ix1.ix1.p1.1">Plain item.</p></para></item><item xml:id="Ix1.ix2"><tags><tag>(b)</tag></tags><para xml:id="Ix1.ix2.p1"><p xml:id="Ix1.ix2.p1.1">Labelled item.</p></para></item><item xml:id="Ix1.ix3"><tags><tag>(5)</tag></tags><para xml:id="Ix1.ix3.p1"><p xml:id="Ix1.ix3.p1.1">Set item.</p></para></item><item xml:id="Ix1.ix4"><tags><tag>(6)</tag></tags><para xml:id="Ix1.ix4.p1"><p xml:id="Ix1.ix4.p1.1">Next item.</p></para></item></enumerate><p xml:id="C0.Sx1.p1.1">After.</p><enumerate xml:id="Ix2"><item xml:id="Ix2.ix1"><tags><tag>(i)</tag></tags><para xml:id="Ix2.ix1.p1"><p xml:id="Ix2.ix1.p1.1">Roman one.</p></para></item><item xml:id="Ix2.ix2"><tags><tag>(ii)</tag></tags><para xml:id="Ix2.ix2.p1"><p xml:id="Ix2.ix2.p1.1">Roman two.</p></para></item></enumerate></para></section>"#,
      )],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_roster_nested.tex"),
      &[(
        "section",
        "C0.Sx1",
        r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Nested</title><theorem class="ltx_theorem_proclaim" xml:id="Thmx1"><title>Theorem 1</title><para xml:id="Thmx1.p1"><enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>(1)</tag></tags><para xml:id="Ix1.ix1.p1"><p xml:id="Ix1.ix1.p1.1">One.</p></para></item><item xml:id="Ix1.ix2"><tags><tag>(2)</tag></tags><para xml:id="Ix1.ix2.p1"><p xml:id="Ix1.ix2.p1.1">Two:</p><enumerate xml:id="Ix2"><item xml:id="Ix2.ix1"><tags><tag>(a)</tag></tags><para xml:id="Ix2.ix1.p1"><p xml:id="Ix2.ix1.p1.1">Inner one.</p></para></item><item xml:id="Ix2.ix2"><tags><tag>(1)</tag></tags><para xml:id="Ix2.ix2.p1"><p xml:id="Ix2.ix2.p1.1">Inner two.</p></para></item></enumerate></para></item></enumerate></para></theorem></section>"#,
      )],
    ),
    (
      include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_roster_runinitem.tex"),
      &[(
        "section",
        "C0.Sx1",
        r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Run-in</title><para xml:id="C0.Sx1.p1"><p xml:id="C0.Sx1.p1.1">Text</p><enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>(1)</tag></tags><para xml:id="Ix1.ix1.p1"><p xml:id="Ix1.ix1.p1.1">first,</p></para></item><item xml:id="Ix1.ix2"><tags><tag>(2)</tag></tags><para xml:id="Ix1.ix2.p1"><p xml:id="Ix1.ix2.p1.1">second,</p></para></item><item xml:id="Ix1.ix3"><tags><tag>(z)</tag></tags><para xml:id="Ix1.ix3.p1"><p xml:id="Ix1.ix3.p1.1">third.</p></para></item></enumerate><p xml:id="C0.Sx1.p1.2">After.</p></para></section>"#,
      )],
    ),
  ];
  for (tex, expected) in rows {
    let xml = assert_elements(tex, AR5IV, (0, 0), expected);
    // no paragraph is left at the document's level (whose ids would be `p1`, `p2`, …)
    assert!(!xml.contains(r#"xml:id="p1""#), "{xml}");
  }
  // a field marker outside a `\ref` (math9204232's `\Refs\ra\key{HM2}`) runs to the end of the input: printed where it
  // stands, the references after it kept (pdftex 5 `!`, the references lost; Perl 0 Errors, lost)
  assert_elements_with(
    include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_stray_field_after_refs.tex"),
    AR5IV,
    (0, 1),
    &["expected:\\@end@bibfield"],
    &[(
      "bibliography",
      "bib",
      r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem xml:id="bib.1"><bibblock>B2
</bibblock></bibitem><bibitem key="A" xml:id="bibliography0.bibx1"><tags><tag role="refnum">A</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">First</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx1.3">J</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx1.4">1990</text>).</bibblock></bibitem><bibitem key="B2" xml:id="bibliography0.bibx2"><tags><tag role="refnum">B2</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx2.1">B. Writer</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx2.2">Second</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx2.3">K</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx2.4">1991</text>).</bibblock></bibitem></biblist></bibliography>"#,
    )],
  );
  // a field marker inside a group the field opened: a `{` and a `\begingroup` closed first (a Warning each; the author's
  // own closer later an Error, as on INT081T), inside an `\hbox` the field stays in the box (a Warning); every word kept
  // (pdftex 2 `!`, A's and B's fields before the marker lost)
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/amstex/amsppt_field_marker_inside_a_group.tex"
    ),
    AR5IV,
    (2, 3),
    &[
      "A reference field ends inside a group it opened",
      "A reference field ends inside a group it opened",
      "A reference field marker inside a box",
    ],
    &[(
      "bibliography",
      "bib",
      r##"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="A" xml:id="bibliography0.bibx1"><tags><tag role="refnum">A</tag></tags><bibblock><text class="ltx_bib_authors" font="smallcaps" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">Title</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx1.3">J</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx1.4">1990</text>).</bibblock></bibitem><bibitem key="B" xml:id="bibliography0.bibx2"><tags><tag role="refnum">B</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx2.1">B. Writer</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx2.2">Second</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx2.3">K</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx2.4">1991</text>).</bibblock></bibitem><bibitem key="C" xml:id="bibliography0.bibx3"><tags><tag role="refnum">C</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx3.1">C. Third</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx3.2">ThirdL</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx3.3">1992</text>).</bibblock></bibitem></biblist></bibliography>"##,
    )],
  );
  assert!(xml.contains(">After the references.</p>"), "{xml}");
  // a `\ref` left open is closed by its `\par` (a blank line, the next `\ref`, `\endRefs`: amsppt.sty:886-889), with
  // TeX's Error; the stray fields after the blank line printed once, untagged (pdftex 8 `!`, the same words)
  let xml = assert_elements_with(
    include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_ref_missing_endref.tex"),
    AR5IV,
    (4, 0),
    &[
      "You seem to have a missing or misspelled",
      "You seem to have a missing or misspelled",
      "You seem to have a missing or misspelled",
    ],
    &[(
      "bibliography",
      "bib",
      r##"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="A" xml:id="bibliography0.bibx1"><tags><tag role="refnum">A</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx1.1">A. Author</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx1.2">First</text>.</bibblock></bibitem><bibitem xml:id="bib.1"><bibblock><text class="ltx_bib_journal" xml:id="bib.1.1">J</text> (<text class="ltx_bib_year" xml:id="bib.1.2">1990</text>).</bibblock></bibitem><bibitem key="B" xml:id="bibliography0.bibx2"><tags><tag role="refnum">B</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx2.1">B. Writer</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx2.2">Second</text>.</bibblock></bibitem><bibitem key="C" xml:id="bibliography0.bibx3"><tags><tag role="refnum">C</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx3.1">C. Third</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx3.2">Third</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx3.3">L</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx3.4">1992</text>).</bibblock></bibitem><bibitem key="D" xml:id="bibliography0.bibx4"><tags><tag role="refnum">D</tag></tags><bibblock><text class="ltx_bib_authors" xml:id="bibliography0.bibx4.1">D. Fourth</text>, <text class="ltx_bib_paper" font="italic" xml:id="bibliography0.bibx4.2">Fourth</text>, <text class="ltx_bib_journal" xml:id="bibliography0.bibx4.3">M</text> (<text class="ltx_bib_year" xml:id="bibliography0.bibx4.4">1993</text>).</bibblock></bibitem></biblist></bibliography>"##,
    )],
  );
  assert!(xml.contains(">After the references.</p>"), "{xml}");
  // the error side of `\roster\runinitem`: elsewhere `\runinitem` is undefined, as in TeX (pdftex 1 `!`, Perl 1 Error)
  assert_elements_with(
    include_str!("../../../tools/perfect_kernel/repros/amstex/amsppt_roster_runinitem_stray.tex"),
    AR5IV,
    (1, 0),
    &["undefined:\\runinitem"],
    &[(
      "section",
      "C0.Sx1",
      r#"<section inlist="toc" xml:id="C0.Sx1"><title>1. Stray</title><para xml:id="C0.Sx1.p1"><enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>(1)</tag></tags><para xml:id="Ix1.ix1.p1"><p xml:id="Ix1.ix1.p1.1">First, <ERROR class="undefined" xml:id="Ix1.ix1.p1.1.1">\runinitem</ERROR>second.</p></para></item></enumerate><p xml:id="C0.Sx1.p1.1">After.</p></para></section>"#,
    )],
  );
}
