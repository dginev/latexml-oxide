//! GA1: an undefined `\c@<ctr>` is the error TeX and Perl raise; the counter leniency lives only where Perl's is.
//! Perl's `generateErrorStub` (State.pm:532-553) has no counter branch; its "Counter '<ctr>' was not defined; assuming
//! 0" is `CounterValue`'s (Package.pm:712), the counter readers', ours `counter::dialect::counter_value`. A new module:
//! no existing one holds the undefined-control-sequence stub's rows.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// A whole element: its tag, identifying attributes, the element.
type Element = (&'static str, &'static [&'static str], &'static str);
/// One repro: its preload (`RAW` where the CLI's raw loading matters), its file, the exact Error and Warning counts, the "assuming 0" Warnings among them, the first Error's head
/// (empty for none), and whole elements.
type Row = (
  Option<&'static str>,
  &'static str,
  usize,
  usize,
  usize,
  &'static str,
  &'static [Element],
);

/// The CLI's raw loading of packages without a binding.
const RAW: &str = "[rawstyles,rawclasses]latexml.sty";

const ROWS: &[Row] = &[
  // The undefined `\c@nosuch` read by `\the`: Perl's two errors (pdflatex's two `!`), no "assuming 0" Warning; "Value: 0."
  // as Perl and the PDF.
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/undefined_counter_register_is_an_error.tex"
    ),
    2,
    0,
    0,
    "Error:undefined:\\c@nosuch",
    &[("p", &[], r#"<p>Value: 0.</p>"#)],
  ),
  // `\arabic{nosuch}` keeps `CounterValue`'s Warning; `\value{nosuch}` is TeX's `\csname` → `\relax` (latex.ltx:10127),
  // read as a number "Missing number": a Warning, today's core severity, not GA1e's to set (GA1: an Error on `\c@nosuch`).
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/counter_readers_keep_their_leniency.tex"
    ),
    0,
    2,
    1,
    "",
    &[("p", &[], "<p>Reader: 0.\nValue: 0.</p>")],
  ),
  // `\edef\x{\value{later}}` before `\newcounter{later}`: `\x` is `macro:->\c@later`, 3 once set, nothing reported
  // (KNOWN_PERL_ERRORS #1231; tikz-timing.sty:2015-2019, 2102.06643, 2409.18120; GA1d: 1 error, 1 warning).
  (
    None,
    include_str!("../../../tools/perfect_kernel/repros/macro-state/value_expands_as_csname.tex"),
    0,
    0,
    0,
    "",
    &[(
      "p",
      &[],
      r#"<p>Meaning: <text font="typewriter">macro:-&gt;\c@later </text>. Value: 3.</p>"#,
    )],
  ),
  // subfig's `\ContinuedFloat` in a float type without a sub-float counter (subfig.sty:587's `\@ifundefined` guard):
  // the number steps back, no sub-counter is touched, nothing is reported. Witness 1910.02851.
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfig_continuedfloat_in_a_float_without_subfloats.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "float",
        &[r#"xml:id="algorithm1""#],
        r#"<float class="ltx_float_algorithm" framed="top" inlist="loa" labels="LABEL:alg:a" placement="H" xml:id="algorithm1">
          <tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag>
          <tag role="typerefnum">Algorithm 1</tag></tags>
          <toccaption><tag close=" ">1</tag>Factorization</toccaption>
          <caption><tag close=" "><text font="bold">Algorithm 1</text></tag>Factorization</caption>
          <p framed="topbottom">First part.</p></float>"#,
      ),
      (
        "float",
        &[r#"xml:id="algorithm1a""#],
        r#"<float class="ltx_float_algorithm" framed="top" inlist="loa" labels="LABEL:alg:b" placement="H" xml:id="algorithm1a">
          <tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag>
          <tag role="typerefnum">Algorithm 1</tag></tags>
          <toccaption><tag close=" ">1</tag>Factorization (continued)</toccaption>
          <caption><tag close=" "><text font="bold">Algorithm 1</text></tag>Factorization (continued)</caption>
          <p framed="topbottom">Second part.</p></float>"#,
      ),
      (
        "para",
        &[r#"xml:id="p1""#],
        r#"<para xml:id="p1"><p>See <ref labelref="LABEL:alg:a"/> and <ref labelref="LABEL:alg:b"/>.</p></para>"#,
      ),
    ],
  ),
  // tabularray's own counters (tabularray.sty:185-194): defined at the binding's load, the table's size set before its
  // cells, each cell's place set at its template (`\lx@tblr@cell@open`); read by `\the`, `\the…` and `\arabic`,
  // inside a cell's text and in a `\textbf`; back to 0 after the table, as pdflatex prints. Witness: the manual
  // non-decimal-units (26 errors on GA1).
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_rownum_colnum_counters.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "tabular",
        &[],
        r#"<tabular vattach="middle"><tbody>
          <tr><td align="left">a1.1</td><td align="left">b1.2/2.3</td><td align="left">c1-3</td></tr>
          <tr><td align="left">d2.1</td><td align="left">e<text font="bold">2.2</text></td>
          <td align="left">f2.3</td></tr></tbody></tabular>"#,
      ),
      ("p", &[], "<p>after: 0.0/0.0</p>"),
    ],
  ),
  // `\c@rownum` is a count whatever loaded first (colortbl.sty:269-274; tabularray.sty:185, :1682): xcolor's `[table]`
  // before tabularray (2601.01074, 2409.12172: GA1b's `expected:register` Warnings, cells "b0.2"/"d0"), and in the
  // same document a `\rowcolors` tabular whose rows read the count colortbl.sty:261 steps (`\the\rownum`, `\number`,
  // `\therownum`; on int160a1 3 errors 12 warnings), each cell as pdflatex. Converted with raw loading on (`RAW`), as
  // the CLI does, so tabularray's `ninecolors` (required with xcolor) loads: 0 W (review revGA1c).
  (
    Some(RAW),
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_after_xcolor_table_rownum_is_a_count.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "table",
        &[r#"placement="h""#],
        r#"<table placement="h" xml:id="tab1"><tabular vattach="middle"><tbody>
          <tr><td align="left">a</td><td align="left">b1.2</td></tr>
          <tr><td align="left">c</td><td align="left">d2</td></tr></tbody></tabular></table>"#,
      ),
      (
        "table",
        &[r#"placement="t""#],
        r##"<table placement="t" xml:id="tab2"><tabular vattach="middle"><tbody>
          <tr><td align="left">a</td><td align="left">b1.1</td></tr>
          <tr backgroundcolor="#FFFFFF"><td align="left"><text backgroundcolor="#FFFFFF">c</text></td>
          <td align="left"><text backgroundcolor="#FFFFFF">d2.2</text></td></tr>
          <tr backgroundcolor="#808080"><td align="left"><text backgroundcolor="#808080">e</text></td>
          <td align="left"><text backgroundcolor="#808080">f3</text></td></tr></tbody></tabular></table>"##,
      ),
    ],
  ),
  // The other load order: tabularray's `\c@rownum` first, colortbl.sty:272 lets `\rownum` to it (xcolor `[table]`);
  // raw loading on, as above.
  (
    Some(RAW),
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_before_xcolor_table_rownum_is_a_count.tex"
    ),
    0,
    0,
    0,
    "",
    &[(
      "tabular",
      &[],
      r#"<tabular vattach="middle"><tbody>
        <tr><td align="left">a</td><td align="left">b1.2</td></tr>
        <tr><td align="left">c</td><td align="left">d2</td></tr></tbody></tabular>"#,
    )],
  ),
  // colortbl alone after tabularray: `\rownum` is tabularray's per-cell count (GA1b "b0.1", "d0": a second count).
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_before_colortbl_rownum_is_its_count.tex"
    ),
    0,
    0,
    0,
    "",
    &[(
      "tabular",
      &[],
      r#"<tabular vattach="middle"><tbody>
        <tr><td align="left">a</td><td align="left">b1.1</td></tr>
        <tr><td align="left">c</td><td align="left">d2</td></tr></tbody></tabular>"#,
    )],
  ),
  // xcolor `[table]` alone, `\therownum` outside a table: colortbl's count, "row 0" (GA1b: 2 Warnings).
  (
    None,
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/colortbl_therownum.tex"),
    0,
    0,
    0,
    "",
    &[("p", &[], "<p>row 0</p>")],
  ),
  // `\c@rowcount` counts no empty last row (tabularray.sty:3722-3731): a trailing `\hline`, `\\[2pt]` … `\\*`, the
  // `expand=` path (`\lx@tblr@rules@shape`) and a `\SetCell[r=2]` table (GA1b "a3", "c3.2", "xm4.2", "z3/4").
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_rowcount_counts_no_empty_last_row.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "table",
        &[r#"placement="h""#],
        r#"<table placement="h" xml:id="tab1"><tabular vattach="middle"><tbody>
          <tr><td align="left" border="t">a2</td><td align="left" border="t">b</td></tr>
          <tr><td align="left" border="b t">c</td><td align="left" border="b t">d</td></tr></tbody></tabular></table>"#,
      ),
      (
        "table",
        &[r#"placement="t""#],
        r#"<table placement="t" xml:id="tab2"><tabular vattach="middle"><tbody>
          <tr><td align="left" cssstyle="padding-bottom: 2.0pt">a2</td>
          <td align="left" cssstyle="padding-bottom: 2.0pt">b</td></tr>
          <tr><td align="left">c</td><td align="left">d</td></tr></tbody></tabular></table>"#,
      ),
      (
        "table",
        &[r#"placement="b""#],
        r#"<table placement="b" xml:id="tab3"><tabular vattach="middle"><tbody>
          <tr><td align="left" border="t">a2</td><td align="left" border="t">b</td></tr>
          <tr><td align="left" border="b">c2.2</td><td align="left" border="b">d</td></tr></tbody></tabular></table>"#,
      ),
      (
        "table",
        &[r#"placement="p""#],
        r#"<table placement="p" xml:id="tab4"><tabular class="ltx_guessed_headers" vattach="middle"><tbody>
          <tr><td align="left" border="l r t" rowspan="2" thead="column">xm3.2</td>
          <td align="left" border="r t" thead="column">y1.2</td></tr>
          <tr><td align="left" border="r t">y2.2</td></tr>
          <tr><td align="left" border="b l r t">z3/3</td><td align="left" border="b r t">w3.2</td></tr>
          </tbody></tabular></table>"#,
      ),
    ],
  ),
  // `thebibliography` names its list's counter as `\usecounter{enumiv}` does (article.cls:573, latex.ltx:16048): an
  // asl.cls entry composing its number from `\value{\@listctr}` reads enumiv, 1 then 2, with nothing reported (GA1
  // alone: 3 errors on `\c@`; witnesses 1005.4193, 0705.0395). The `[…]` left in the text is
  // item_label_in_thebibliography_is_the_entry_tag_red.
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/listctr_after_thebibliography.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "bibitem",
        &[r#"xml:id="bib.bib1""#],
        "<bibitem xml:id=\"bib.bib1\"><tags><tag>[1]</tag><tag role=\"refnum\">1</tag></tags>
          <bibblock>[[1]]First entry.\n</bibblock></bibitem>",
      ),
      (
        "bibitem",
        &[r#"xml:id="bib.bib2""#],
        "<bibitem xml:id=\"bib.bib2\"><tags><tag>[2]</tag><tag role=\"refnum\">2</tag></tags>
          <bibblock>[[2]]Second entry.\n</bibblock></bibitem>",
      ),
    ],
  ),
  // varioref.sty:803's `\c@vrcnt`, which cleveref's varioref support steps (1602.00476, 2409.11413, 2505.14566; GA1
  // alone: 3 errors on `\c@vrcnt`).
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/varioref_defines_vrcnt.tex"
    ),
    0,
    0,
    0,
    "",
    &[("p", &[], "<p>Count: 1, 1.</p>")],
  ),
  // A list is numbered only under `\@nmbrlisttrue` (latex.ltx:16024): memoir.cls:4580's raw `\list{}{}` inside that
  // bibliography keeps `\@listctr` = enumiv but numbers nothing (the `\@listctr` fix alone tagged it "0()A"); a
  // `\usecounter` list there and both shapes in an enumerate keep pdflatex's numbers. Raw loading on, as the CLI.
  (
    Some(RAW),
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/raw_list_numbers_only_under_nmbrlist.tex"
    ),
    0,
    1,
    0,
    "",
    &[
      (
        "bibitem",
        &[r#"xml:id="bib.bib1""#],
        r#"<bibitem key="a" xml:id="bib.bib1"><tags><tag>[1] </tag><tag role="refnum">1</tag></tags>
          <bibblock> First.
<itemize>
            <item xml:id="chapter0.section0.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags>
              <para xml:id="chapter0.section0.I1.i1.p1"><p>Inner one.</p></para></item>
            <item xml:id="chapter0.section0.I1.i2"><tags><tag role="typerefnum">2nd item</tag></tags>
              <para xml:id="chapter0.section0.I1.i2.p1"><p>Inner two.</p></para></item>
          </itemize></bibblock></bibitem>"#,
      ),
      (
        "bibitem",
        &[r#"xml:id="bib.bib2""#],
        r#"<bibitem key="b" xml:id="bib.bib2"><tags><tag>[2] </tag><tag role="refnum">2</tag></tags>
          <bibblock> Second.
<itemize>
            <item xml:id="chapter0.section0.I2.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags>
              <para xml:id="chapter0.section0.I2.i1.p1"><p>Counted one.</p></para></item>
            <item xml:id="chapter0.section0.I2.i2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags>
              <para xml:id="chapter0.section0.I2.i2.p1"><p>Counted two.</p></para></item>
          </itemize></bibblock></bibitem>"#,
      ),
      (
        "item",
        &[r#"xml:id="chapter0.section0.I3.i1""#],
        r#"<item xml:id="chapter0.section0.I3.i1">
          <tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags>
          <para xml:id="chapter0.section0.I3.i1.p1"><p>Outer.</p>
            <itemize>
              <item xml:id="chapter0.section0.I3.i1.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags>
                <para xml:id="chapter0.section0.I3.i1.I1.i1.p1"><p>Nested.</p></para>
              </item></itemize>
            <itemize>
              <item xml:id="chapter0.section0.I3.i1.I2.i1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags>
                <para xml:id="chapter0.section0.I3.i1.I2.i1.p1"><p>Counted.</p></para>
              </item></itemize></para></item>"#,
      ),
    ],
  ),
  // A raw `\@trivlist` in `thebibliography` invents no number (GA1d's hidden refnums "0A", "0B"; revGA1d): unnumbered,
  // as int163a, the trivlist before it untouched; pdflatex's [2], [3] wait for bibitem_steps_the_list_counter_red.
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/raw_trivlist_in_thebibliography_invents_no_number.tex"
    ),
    0,
    0,
    0,
    "",
    &[(
      "bibitem",
      &[r#"xml:id="bib.bib1""#],
      r#"<bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags>
        <bibblock> First.
<itemize class="ltx_trivlist" xml:id="S0.I1">
          <item xml:id="S0.I1.ix1"><tags><tag/></tags><para xml:id="S0.I1.ix1.p1"><p>tr1</p></para></item>
          <item xml:id="S0.I1.ix2"><tags><tag/></tags><para xml:id="S0.I1.ix2.p1"><p>tr2</p></para></item>
        </itemize><itemize>
          <item xml:id="S0.I2.i1"><tags><tag role="typerefnum">1st item</tag></tags>
            <para xml:id="S0.I2.i1.p1"><p>raw1</p></para></item>
          <item xml:id="S0.I2.i2"><tags><tag role="typerefnum">2nd item</tag></tags>
            <para xml:id="S0.I2.i2.p1"><p>raw2</p></para></item></itemize></bibblock></bibitem>"#,
    )],
  ),
  // The control of footnoterecall_counter_defined_by_an_earlier_creator_red: B's `\footnoterecall` reads A's counter
  // (the block written in order, 1 counted Warning); no Error since `\value` is `\csname` (2407.13730, 2606.19583).
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/footnoterecall_two_authors_control.tex"
    ),
    0,
    1,
    0,
    "",
    &[
      (
        "creator",
        &[r#"role="author""#],
        "<creator role=\"author\"><personname>A</personname>\
         <contact name=\"Note:\u{a0}\" role=\"note\">X</contact></creator>",
      ),
      (
        "creator",
        &["before=\"\u{2003}\u{2003}\""],
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>B</personname>\
         <note mark=\"*\" role=\"footnotemark\" xml:id=\"footnotex1\"><tags><tag>*</tag><tag role=\"refnum\">*</tag>\
         <tag role=\"typerefnum\">footnote *</tag></tags></note></creator>",
      ),
    ],
  ),
  // A footnote's `[<n>]` read as `\c@mpfn #1\relax` (latex.ltx:17652-17657, :17678-17682) when the scan leaves nothing
  // to print: `\value{nosuch}`, `\relax` are TeX's 0 with its "Missing number" (the core's Warning; pdflatex 7 `!`, E's
  // `$\dagger$` stays as given and silent, OD #851), `2\relax` is 2, `\fnsymbol` at 0 nothing (revGA1e B1; int163a,
  // GA1e: the marks empty and silent; 2407.13730, 2606.19583).
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/footnotemark_missing_number_reads_as_tex.tex"
    ),
    0,
    6,
    0,
    "",
    &[
      (
        "note",
        &[r#"xml:id="footnotex1""#],
        r#"<note mark="0" role="footnotemark" xml:id="footnotex1"><tags><tag>0</tag><tag role="refnum">0</tag>
          <tag role="typerefnum">footnote 0</tag></tags></note>"#,
      ),
      (
        "note",
        &[r#"xml:id="footnotex2""#],
        r#"<note mark="0" role="footnotemark" xml:id="footnotex2"><tags><tag>0</tag><tag role="refnum">0</tag>
          <tag role="typerefnum">footnote 0</tag></tags></note>"#,
      ),
      (
        "note",
        &[r#"xml:id="footnotex3""#],
        r#"<note mark="2" role="footnotemark" xml:id="footnotex3"><tags><tag>2</tag><tag role="refnum">2</tag>
          <tag role="typerefnum">footnote 2</tag></tags></note>"#,
      ),
      (
        "note",
        &[r#"xml:id="footnotex5""#],
        r#"<note mark="0" role="footnote" xml:id="footnotex5"><tags><tag>0</tag><tag role="refnum">0</tag>
          <tag role="typerefnum">footnote 0</tag></tags>Body.</note>"#,
      ),
      (
        "note",
        &[r#"xml:id="footnotex6""#],
        r#"<note role="footnotemark" xml:id="footnotex6"><tags><tag role="typerefnum">footnote </tag></tags></note>"#,
      ),
      (
        "note",
        &[r#"xml:id="footnotex7""#],
        r#"<note mark="†" role="footnotemark" xml:id="footnotex7"><tags><tag>†</tag><tag role="refnum">†</tag>
          <tag role="typerefnum">footnote †</tag></tags></note>"#,
      ),
    ],
  ),
  // `\footnotetext[\value{nosuch}]` is marked 0 as the `\footnotemark[\value{nosuch}]` it answers, and pairs with it
  // (`_fnvalue` 0), as pdflatex's "0 Paired." (2 `!`); int163a, GA1e: an empty mark and an unpaired text, silent.
  (
    None,
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/footnotetext_missing_number_pairs_with_its_mark.tex"
    ),
    0,
    2,
    0,
    "",
    &[
      (
        "note",
        &[r#"xml:id="footnotex1""#],
        r#"<note mark="0" role="footnote" xml:id="footnotex1"><tags><tag>0</tag><tag role="refnum">0</tag>
          <tag role="typerefnum">footnote 0</tag></tags>Paired.</note>"#,
      ),
      (
        "note",
        &[r#"xml:id="footnote1""#],
        r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag>
          <tag role="typerefnum">footnote 1</tag></tags>Real.</note>"#,
      ),
    ],
  ),
];

/// Repros macro-state/undefined_counter_register_is_an_error (GA0 finding 1), macro-state/
/// counter_readers_keep_their_leniency and captions-floats/subfig_continuedfloat_in_a_float_without_subfloats
/// (witness 1910.02851), alignment-bindings/tabularray_rownum_colnum_counters (GA1b), the `\c@rownum` count in every
/// load order of xcolor `[table]`, colortbl and tabularray and tabularray's `\c@rowcount` (GA1c), the bibliography's
/// `\@listctr` and the `\@nmbrlisttrue` test it needs, varioref's `\c@vrcnt` (GA1d), `\value` as `\csname`, a raw
/// `\@trivlist` in a bibliography, the two-author `\footnoterecall` control (GA1e), a footnote's `[<n>]` that is no
/// number (GA1f): exact counts, the first Error, whole elements.
#[test]
fn an_undefined_counter_register_is_the_error_tex_raises() {
  for (preload, tex, errors, warnings, assuming, first_error, elements) in ROWS {
    let (stderr, xml) = convert_with(tex, *preload);
    let head = tex.lines().next().unwrap_or_default();
    assert_eq!(error_count(&stderr), *errors, "{head}\n{stderr}");
    assert_eq!(warning_count(&stderr), *warnings, "{head}\n{stderr}");
    let assumed = stderr
      .lines()
      .filter(|l| l.starts_with("Warning:") && l.contains("was not defined; assuming 0"));
    assert_eq!(
      assumed.count(),
      *assuming,
      "{head}: only a counter reader says \"assuming 0\"\n{stderr}"
    );
    if !first_error.is_empty() {
      let first = stderr
        .lines()
        .find(|l| l.starts_with("Error:"))
        .unwrap_or_default();
      assert!(
        first.starts_with(first_error),
        "{head}: first error {first:?}\n{stderr}"
      );
    }
    for (tag, attrs, element) in *elements {
      assert_element(&xml, tag, attrs, element);
    }
  }
}

/// The `Warn!` calls in `generate_error_stub`'s body, comment lines left out. The stub raises Perl's errors only
/// (State.pm:532-553): a leniency there would silence an undefined control sequence by its name, in every context.
fn stub_warnings(source: &str) -> usize {
  let start = source
    .find("pub fn generate_error_stub(")
    .expect("generate_error_stub is in state.rs");
  let body = &source[start..];
  let end = body.find("\n}\n").expect("the stub's closing brace");
  body[..end]
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches("Warn!(").count())
    .sum()
}

/// Fence F4 (GA0): `generate_error_stub` holds no `Warn!` (GA1 deleted its `\c@<ctr>` branch).
#[test]
fn the_undefined_stub_raises_errors_only() {
  assert_eq!(
    stub_warnings(include_str!("../../../latexml_core/src/state.rs")),
    0,
    "generate_error_stub warns: Perl's generateErrorStub (State.pm:532-553) raises Error:undefined only. A leniency \
     belongs to the reader Perl gives it (CounterValue, Package.pm:712: counter::dialect::counter_value), not to the stub"
  );
}
