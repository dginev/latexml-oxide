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
/// `\tablehead` on — a head's `\\ \cline` (1002.1848; TeX's empty row is kept, as pdflatex sets it), rows without
/// `\startdata` (1411.3719) and after `\enddata` (1909.11243), a second `\enddata` and a `\label` after it
/// (1902.00588); notes, comments and labels after the data make no row.
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
        r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag>Table 1</tag>Cap</caption><tabular><thead><tr><td align="center" border="tt" thead="column">A</td><td align="center" border="tt" thead="column">B</td></tr></thead><tbody><tr><td border="t"/><td border="t"/></tr><tr><td align="left" border="t">a</td><td align="center" border="t">1</td></tr><tr><td align="left" border="b">b</td><td align="center" border="b">2</td></tr></tbody></tabular></table>"##,
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
    ],
  );
}
