//! Red/green guards for the int19 fix round (the bundle A/B's worse papers, agent_reports/2026-10-17_payload_int19.md).
//! Item 1 — the token after a display's closing `$` is read in the display's own mode again (69b r2's enclosing-mode
//! read reverted: with Perl's "Ignoring" recovery it cascaded, arXiv 2207.12569). Item 2 — amstext's `\text` digests
//! its argument inside box braces (amstext.sty:28-39), so an alignment's `\cr` in it never ends the cell around it
//! (arXiv 1703.08664). Item 3 — amsmath's `\boxed` is `\fbox{\m@th$\displaystyle#1$}` (amsmath.sty:399), so an author's
//! `$` in its argument is text in that math, as 69c's E1 (arXiv 2207.12569).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// expansion-primitives/display_closing_dollar_text_after_the_display_kept: arXiv 2207.12569's
/// `$${\boxed{\mbox{…}}}\Longrightarrow{\boxed{$u$\mbox{-}$S$\mbox{-Artinian rings}}}…$$` (pdflatex 0 errors). 69b r2
/// read the token after each `$` in `\boxed` under the enclosing mode, as text, then re-read it in the display Perl's
/// recovery leaves open: 8 errors here, 6 -> 101 + Fatal on the paper, the paragraph after the display lost. Item 1
/// restored base's 6 (the six `$` in `\boxed`); item 3 makes them text (`dollar_in_boxed_is_text`): 0 errors, the
/// display closed by its `$$`, the paragraph after it kept.
#[test]
fn display_closing_dollar_text_after_the_display_kept() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/display_closing_dollar_text_after_the_display_kept.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>We have the implications</p>"), "{xml}");
  assert!(
    xml.contains("<p>But neither of implications can be reversed.</p>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<Math mode=\"display\"").count(), 1, "{xml}");
}

/// kernel-alignment/text_argument_is_braced_in_an_alignment_cell: arXiv 1703.08664's `\mWord` — `\text{\ $\bgroup
/// \let\\=\crcr … \halign{…} \egroup$}` — in `{align}` cells (pdflatex 0 errors). amstext.sty:28-39 sets `\text`'s
/// argument in `\hbox{{…}}`, so its `\let\\=\crcr` is read at a raised `align_state` (tex.web §342) and ends no cell
/// of the align; the constructor's argument, read before it is digested, had its braces counted already and the
/// `\crcr` ended the align's cell (67int18b 12 errors, 69c 9, Perl 16; on the paper 58 -> 101 + Fatal). Now 0 errors,
/// each matrix a tabular in the text (KNOWN_PERL_ERRORS #931).
#[test]
fn text_argument_is_braced_in_an_alignment_cell() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/text_argument_is_braced_in_an_alignment_cell.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The first matrix, (d-1, d-2 over 0, 0), each entry a formula, the template's `,` column between.
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_markedasmath" vattach="bottom"><tr><td align="center"><Math mode="inline" tex="{d-1}" text="d - 1" xml:id="S0.E1.m1.m1.m1.mf"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></td><td align="left" class="ltx_nopad_l ltx_nopad_r">,</td><td align="center"><Math mode="inline" tex="{d-2}" text="d - 2" xml:id="S0.E1.m1.m1.m2.mf"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></td></tr><tr><td align="center"><Math mode="inline" tex="{0}" text="0" xml:id="S0.E1.m1.m1.m3.mf"><XMath><XMTok meaning="0" role="NUMBER">0</XMTok></XMath></Math></td><td align="left" class="ltx_nopad_l ltx_nopad_r">,</td><td align="center"><Math mode="inline" tex="{0}" text="0" xml:id="S0.E1.m1.m1.m4.mf"><XMath><XMTok meaning="0" role="NUMBER">0</XMTok></XMath></Math></td></tr></tabular>"#,
  );
  assert_eq!(xml.matches("<tabular ").count(), 4, "{xml}");
  assert!(xml.contains("<p>after text.</p>"), "{xml}");
}

/// paragraph-mode/dollar_in_boxed_is_text: amsmath.sty:399 `\boxed` = `\fbox{\m@th$\displaystyle#1$}`, so
/// `\boxed{$u$\mbox{-}$S$…}` in a display, `\boxed{$v$ rings}` inline and `\boxed{$w$ ideals}` in text set u, v, w as
/// text in the box's math and what follows the second `$` as math (pdflatex 0 errors: "u-S-Artinian rings", "vrings",
/// "wideals"; arXiv 2207.12569). Before int19: 4 errors (each `$` met the enclosing math; KNOWN_PERL_ERRORS #907).
#[test]
fn dollar_in_boxed_is_text() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/paragraph-mode/dollar_in_boxed_is_text.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The display's second box: u and S text, the `\mbox`es between.
  assert_element(
    &xml,
    "XMApp",
    &["enclose=\"box\""],
    r#"<XMApp enclose="box"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMText>u</XMText><XMText>-</XMText><XMText>S</XMText><XMText>-Artinian rings</XMText></XMApp>"#,
  );
  // In text: w as text, "ideals" math.
  assert_element(
    &xml,
    "Math",
    &["framed=\"rectangle\""],
    r#"<Math framed="rectangle" mode="display" text="[w] * i * d * e * a * l * s" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMText>w</XMText><XMTok font="italic" role="UNKNOWN">i</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">l</XMTok><XMTok font="italic" role="UNKNOWN">s</XMTok></XMApp></XMath></Math>"#,
  );
}
