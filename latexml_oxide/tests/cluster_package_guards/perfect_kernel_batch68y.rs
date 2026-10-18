//! Red/green guards for batch 68y: the AAS, A&A, emulateapj, iopart and mn2e binding shapes of the rc_struct2
//! structural first-error clusters (`&`, `\noalign`, `^`): each class's own definition, where Perl's binding differs
//! (KNOWN_PERL_ERRORS #871-#878).
//!
//! Every repro is checked against pdflatex (its header holds the oracle; the A&A, mn2e and iopart classes are the
//! witness papers' copies, not in TeX Live).
use super::perfect_kernel_batch57::{RAW, assert_elements};

/// class-bindings/aa_tablefootmark_text_mark: aa.cls v9.1 :1672-1681 sets `\tablefootmark`/`\tablefoottext`'s mark in
/// text: `$^\dag$` is a superscript dagger (2211.06931), and `\tablefoottext{a}` still pairs with `\tablefootmark{a}`
/// by `mark`.
#[test]
fn aa_tablefootmark_sets_the_mark_in_text() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aa_tablefootmark_text_mark.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "table",
      "tab1",
      r##"<table xml:id="tab1"><tabular vattach="middle"><tbody><tr><td align="center">Shift<note mark="a" role="footnote" xml:id="footnotex1"><tags><tag>a</tag><tag role="refnum">a</tag><tag role="typerefnum">footnote a</tag></tags>Modified Julian date.</note></td><td align="center">b<note mark="${}^{\dagger}$" role="footnote" xml:id="footnotex2"><tags><tag><sup><text font="italic">†</text></sup></tag><tag role="refnum"><sup><text font="italic">†</text></sup></tag><tag role="typerefnum">footnote <sup><text font="italic">†</text></sup></tag></tags>Second.</note></td></tr></tbody></tabular><note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags><text font="bold">Notes.</text> </note></table>"##,
    )],
  );
}

/// class-bindings/aa_ion_robust_cell_head: aa.cls `\DeclareRobustCommand{\ion}[2]`: a cell starting with `\ion` is not
/// expanded by the cell's look for `\omit`, so its `&` stays a tab (0801.2767): three cells per row, as pdflatex.
#[test]
fn aa_ion_is_robust_at_a_cell_head() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aa_ion_robust_cell_head.tex"),
    RAW,
    (0, 0),
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><tabular class="ltx_guessed_headers" vattach="middle"><thead><tr><td align="left" thead="column">4101</td><td align="left" thead="column">H<Math mode="inline" tex="\delta" text="delta" xml:id="p1.m1"><XMath><XMTok font="italic" name="delta" role="UNKNOWN">δ</XMTok></XMath></Math></td><td align="center" thead="column">24.0</td></tr></thead><tbody><tr><td align="left">4267</td><td align="left">C <text font="smallcaps">ii</text></td><td align="center">0.3</td></tr></tbody></tabular></para>"##,
    )],
  );
}

/// class-bindings/aastex_tablenotetext_math_mark: aastex701.cls :11967-11969 sets the `\tablenotetext` mark in math:
/// `^\dagger` (2503.18379) and `\dagger` are marked "†", `a`/`b` pair with their `\tablenotemark`s.
#[test]
fn aastex_tablenotetext_mark_is_math() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aastex_tablenotetext_math_mark.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "table",
      "S0.T1",
      r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Table 1</tag>Cap<note mark="†" role="footnotetext">Third note.</note><note mark="†" role="footnotetext">Fourth note.</note></caption><tabular><thead><tr><td align="center" border="tt" thead="column">A</td><td align="center" border="tt" thead="column">B</td></tr></thead><tbody><tr><td align="center" border="b t">1<note mark="a" role="footnote">First note.</note></td><td align="center" border="b t">2<note mark="b" role="footnote">Second note.</note></td></tr></tbody></tabular></table>"##,
    )],
  );
}

/// class-bindings/emulateapj_deluxetable_open_from_head: emulateapj.cls:1430-1460: the deluxetable's rows are rows from
/// `\tablehead` on — a head's `\\ \cline` (1002.1848: an empty row and a rule `\startdata` makes the continuation
/// head, on no first page, so neither is here), rows without
/// `\startdata` (1411.3719) and after `\enddata` (1909.11243), a second `\enddata` and a `\label` after it
/// (1902.00588); notes, comments and labels after the data make no row; in the document's own `{longtable}` the head
/// is its rows and `\tablecaption` its caption (1810.10130).
#[test]
fn emulateapj_deluxetable_is_open_from_its_head() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/emulateapj_deluxetable_open_from_head.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag>Table 1</tag>Cap</caption><tabular><thead><tr><td align="center" border="tt" thead="column">A</td><td align="center" border="tt" thead="column">B</td></tr></thead><tbody><tr><td align="left" border="t">a</td><td align="center" border="t">1</td></tr><tr><td align="left" border="b">b</td><td align="center" border="b">2</td></tr></tbody></tabular></table>"##,
      ),
      (
        "table",
        "S0.T2",
        r##"<table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="autoref">Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Cap two</toccaption><caption><tag>Table 2</tag>Cap two</caption><tabular><thead><tr><td align="left" border="tt" thead="column">parameter</td><td align="center" border="tt" thead="column">value</td></tr></thead><tbody><tr><td align="left" border="t"><Math mode="inline" tex="a_{0}" text="a _ 0" xml:id="S0.T2.m1"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math></td><td align="center" border="t">-2.55</td></tr><tr><td align="left" border="b"><Math mode="inline" tex="a_{1}" text="a _ 1" xml:id="S0.T2.m2"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></td><td align="center" border="b">-0.26</td></tr></tbody></tabular></table>"##,
      ),
      (
        "table",
        "S0.T3",
        r##"<table inlist="lot" labels="LABEL:tab:x LABEL:tab:y" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="autoref">Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Cap three</toccaption><caption><tag>Table 3</tag>Cap three</caption><tabular class="ltx_figure_panel"><thead><tr><td align="center" border="tt" thead="column">A</td><td align="center" border="tt" thead="column">B</td><td align="center" border="tt" thead="column">C</td></tr></thead><tbody><tr><td align="left" border="t">a</td><td align="center" border="t">1<note mark="a" role="footnote">First note.</note></td><td align="center" border="t">x</td></tr><tr><td align="left" border="b">b</td><td align="center" border="b">2</td><td align="center" border="b"><Math mode="inline" tex="y" text="y" xml:id="S0.T3.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel ltx_parbox" vattach="middle" width="345.0pt"><text fontsize="80%">Note. — A comment.</text></p></table>"##,
      ),
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><p>See Table <ref labelref="LABEL:tab:y"/>.</p></para>"##,
      ),
      (
        "table",
        "S0.T4",
        r##"<table inlist="lot" labels="LABEL:obs" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="autoref">Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><caption><tag>Table 4</tag>Galaxies</caption><toccaption><tag close=" ">4</tag>Galaxies</toccaption><tabular><thead><tr><td align="center" thead="column">name</td><td align="center" thead="column">z</td></tr></thead><tbody><tr><td align="left" border="t">G1</td><td align="left" border="t">1.99</td></tr></tbody></tabular></table>"##,
      ),
    ],
  );
}

/// class-bindings/aastex_decimals_noalign: aastex701.cls :12014 `\decimals` is a `\noalign` in a `tabular` (1805.06850:
/// `\decimals \tableline` at the row start), an assignment before a deluxetable's data; both split the `D` cells at the
/// point.
#[test]
fn aastex_decimals_is_a_noalign_in_a_tabular() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aastex_decimals_noalign.tex"),
    RAW,
    (0, 0),
    &[
      (
        "table",
        "tab1",
        r##"<table xml:id="tab1"><tabular vattach="middle"><tbody><tr><td align="left" border="t">a</td><td align="right" border="t" class="ltx_norightpad"><Math mode="inline" tex="1" text="1" xml:id="m1"><XMath><XMTok meaning="1" role="NUMBER">1</XMTok></XMath></Math></td><td align="left" border="t" class="ltx_noleftpad">.<Math mode="inline" tex="5" text="5" xml:id="m2"><XMath><XMTok meaning="5" role="NUMBER">5</XMTok></XMath></Math></td><td align="center" border="t">x</td></tr><tr><td align="left" border="b">b</td><td align="right" border="b" class="ltx_norightpad"><Math mode="inline" tex="-22" text="- 22" xml:id="m3"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok meaning="22" role="NUMBER">22</XMTok></XMApp></XMath></Math></td><td align="left" border="b" class="ltx_noleftpad">.<Math mode="inline" tex="75" text="75" xml:id="m4"><XMath><XMTok meaning="75" role="NUMBER">75</XMTok></XMath></Math></td><td align="center" border="b">y</td></tr></tbody></tabular></table>"##,
      ),
      (
        "table",
        "tab2",
        r##"<table xml:id="tab2"><tabular><thead><tr><td align="center" border="tt" thead="column">A</td><td align="center" border="tt" colspan="2" thead="column">B</td><td align="center" border="tt" thead="column">C</td></tr></thead><tbody><tr><td align="left" border="t">a</td><td align="right" border="t" class="ltx_norightpad"><Math mode="inline" tex="1" text="1" xml:id="m5"><XMath><XMTok meaning="1" role="NUMBER">1</XMTok></XMath></Math></td><td align="left" border="t" class="ltx_noleftpad">.<Math mode="inline" tex="5" text="5" xml:id="m6"><XMath><XMTok meaning="5" role="NUMBER">5</XMTok></XMath></Math></td><td align="center" border="t">x</td></tr><tr><td align="left" border="b">b</td><td align="right" border="b" class="ltx_norightpad"><Math mode="inline" tex="-22" text="- 22" xml:id="m7"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok meaning="22" role="NUMBER">22</XMTok></XMApp></XMath></Math></td><td align="left" border="b" class="ltx_noleftpad">.<Math mode="inline" tex="75" text="75" xml:id="m8"><XMath><XMTok meaning="75" role="NUMBER">75</XMTok></XMath></Math></td><td align="center" border="b">y</td></tr></tbody></tabular></table>"##,
      ),
    ],
  );
}

/// class-bindings/aastex_dcolumn_D_with_arguments: A document's dcolumn after aastex replaces the class's `D`
/// (aastex701.cls :12010, :6286-6291): `D{.}{.}{6}` (via the document's `d`, 1607.06811) is dcolumn's point-aligned
/// column.
#[test]
fn aastex_d_column_with_arguments_is_dcolumns() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aastex_dcolumn_D_with_arguments.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "table",
      "tab1",
      r##"<table xml:id="tab1"><tabular vattach="middle"><tbody><tr><td align="left"><Math mode="inline" tex="x" text="x" xml:id="m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td><td/><td align="char:." class="ltx_nopad_r"><Math mode="inline" tex="-0.50^{a}" text="- 0.50 ^ a" xml:id="m2"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="0.50" role="NUMBER">0.50</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp></XMApp></XMath></Math></td></tr></tbody></tabular></table>"##,
    )],
  );
}

/// class-bindings/mn2e_usedcolumn_column_types: mn2e.cls v2.2 :1210-1216 `usedcolumn`: `d{2.1}` is dcolumn's
/// `D{.}{.}{2.1}` (astro-ph0701731, astro-ph0312263).
#[test]
fn mn2e_usedcolumn_column_types() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/mn2e_usedcolumn_column_types.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><tabular class="ltx_guessed_headers" vattach="middle"><tbody><tr><td align="right" thead="row">A</td><td align="char:."><Math mode="inline" tex="1.5" text="1.5" xml:id="p1.m1"><XMath><XMTok meaning="1.5" role="NUMBER">1.5</XMTok></XMath></Math></td></tr><tr><td align="right" thead="row">B</td><td align="char:."><Math mode="inline" tex="22.25" text="22.25" xml:id="p1.m2"><XMath><XMTok meaning="22.25" role="NUMBER">22.25</XMTok></XMath></Math></td></tr></tbody></tabular></para>"##,
    )],
  );
}

/// class-bindings/iopart_row_spacers_noalign: iopart.cls :935-938: `\ns`/`\bs` after a row's `\\` are `\noalign`
/// spacers, so the `\mr` after `\ns` is the rule between the two rows (1502.00309).
#[test]
fn iopart_row_spacers_are_noalign() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/iopart_row_spacers_noalign.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><tabular class="ltx_guessed_headers" vattach="middle"><tbody><tr><td align="left" border="t" thead="row">a</td><td align="center" border="t">b</td></tr><tr><td align="left" border="b t" thead="row">c</td><td align="center" border="b t">d</td></tr></tbody></tabular></para>"##,
    )],
  );
}

/// class-bindings/emulateapj_startdata_ends_head: emulateapj.cls:1324 `\startdata` is `\kill\caption{…}\endhead…`:
/// it ends the row in progress — the second head line written without a `\\` (1605.03270) stays a row of its own,
/// not the start of the first data row — and the rows since `\tablehead` are the heading (2002.02307), without the
/// empty row after the head's `\\` and the `\cline` before `\startdata`, which TeX prints on no first page (only in
/// the continuation head, KNOWN_PERL_ERRORS #874); the head's own rule is under the last head line, above the first
/// data row, as TeX's first page prints it (1605.03270 Table 1). Rows TeX prints only in the continuation head are
/// kept (OD #663); the head prints the stored caption, "Cap two" again on Table 3, as TeX (emulateapj.cls:1333, OD #740),
/// and a `\tablecaption` after `\tablehead` is kept too, after the tabular, unnumbered (OD #664).
#[test]
fn emulateapj_startdata_ends_the_head() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/emulateapj_startdata_ends_head.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "table",
        "S0.T1",
        r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Sample</toccaption><caption><tag>Table 1</tag>Sample</caption><tabular><thead><tr><td align="center" border="tt" thead="column">AAA</td><td align="center" border="tt" thead="column">BBB</td><td align="center" border="tt" thead="column">CCC</td></tr><tr><td thead="column"/><td align="center" thead="column">Name</td><td align="center" thead="column">(pc)</td></tr></thead><tbody><tr><td align="left" border="t">xx1</td><td align="right" border="t">yy1</td><td align="right" border="t">zz1</td></tr><tr><td align="left" border="b">xx2</td><td align="right" border="b">yy2</td><td align="right" border="b">zz2</td></tr></tbody></tabular></table>"##,
      ),
      (
        "table",
        "S0.T2",
        r##"<table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="autoref">Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Cap two</toccaption><caption><tag>Table 2</tag>Cap two</caption><tabular><thead><tr><td align="center" border="tt" thead="column">AAA</td><td align="center" border="tt" thead="column">BBB</td></tr><tr><td align="left" thead="column">XROW</td><td align="center" thead="column">YROW</td></tr></thead><tbody><tr><td align="left" border="b t">a1</td><td align="center" border="b t">1</td></tr></tbody></tabular></table>"##,
      ),
      (
        "table",
        "S0.T3",
        r##"<table inlist="lot" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="autoref">Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Cap two</toccaption><caption><tag>Table 3</tag>Cap two</caption><tabular><thead><tr><td align="center" border="tt" thead="column">AAA</td><td align="center" border="tt" thead="column">BBB</td></tr></thead><tbody><tr><td align="left" border="b t">c1</td><td align="center" border="b t">3</td></tr></tbody></tabular><caption>Late cap</caption></table>"##,
      ),
    ],
  );
}

/// class-bindings/aastex_tablenote_mark_shown: a note pairs with the mark that shows the same, both sides read by one
/// function (`shown_mark`): `c, d` with `c,d` (printed as set, paired ignoring spaces), `*` with `\ast` (TeX's one
/// asterisk), `\tablenotemark{$\dagger$}` with `\dagger`, its mark "†", `\rm{b}` with `b` (1710.01715), the note mark
/// `$a$` set in the class's `\hbox{$#1$}` as the text "a" (1807.05231), with no error and its body kept, and only
/// what an argument shows: `\hyperlink{tn5}{e}` is "e", `\raisebox{1pt}{g}` is "g".
#[test]
fn aastex_tablenote_marks_pair_by_what_they_show() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aastex_tablenote_mark_shown.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "table",
      "S0.T1",
      r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Marks</toccaption><caption><tag close=": ">Table 1</tag>Marks</caption><tabular><thead><tr><td align="center" border="tt" thead="column">X</td><td align="center" border="tt" thead="column">Y</td></tr></thead><tbody><tr><td align="left" border="t">r1</td><td align="center" border="t">1<note mark="c, d" role="footnote">Note CD.</note></td></tr><tr><td align="left">r2</td><td align="center">2<note mark="*" role="footnote">Note AST.</note></td></tr><tr><td align="left">r3</td><td align="center">3<note mark="†" role="footnote">Note DAG.</note></td></tr><tr><td align="left">r4</td><td align="center">4<note mark="b" role="footnote">Note B.</note></td></tr><tr><td align="left">r5</td><td align="center">5<note mark="a" role="footnote">Note A.</note></td></tr><tr><td align="left">r6</td><td align="center">6<note mark="e" role="footnote">Note E.</note></td></tr><tr><td align="left" border="b">r7</td><td align="center" border="b">7<note mark="g" role="footnote">Note G.</note></td></tr></tbody></tabular></table>"##,
    )],
  );
}
