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
