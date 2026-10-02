//! Red/green guards for perfect-kernel phase-58 batches: the reledmac and floatrow kernel roots
//! (58a) found in the 57cp review.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with};

/// 58a (reledmac root C): the optional `=` of `\setbox`, `\font`, `\openin`, `\openout` is scanned
/// with expansion (tex.web §405), so a conditional or a macro may supply it; matched unexpanded (Perl
/// `SkipMatch:=`, KNOWN_PERL_ERRORS #398) the "=" was typeset and the box went to the page. Repro
/// expansion-primitives/optional_equals_expands.
#[test]
fn optional_equals_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/optional_equals_expands.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    ("para", "p1", r##"<para xml:id="p1"><p>A Up.</p></para>"##),
    ("para", "p2", r##"<para xml:id="p2"><p>B Eq.</p></para>"##),
    ("para", "p3", r##"<para xml:id="p3"><p>C.</p></para>"##),
    ("para", "p4", r##"<para xml:id="p4"><p>D.</p></para>"##),
  ]);
}

/// 58a (reledmac root D): `\setcounter`, `\addtocounter`, `\stepcounter`, `\refstepcounter` and
/// `\pagenumbering` are macros, as in latex.ltx, so etoolbox patches them (KNOWN_PERL_ERRORS #399). Repro
/// macro-state/counter_commands_are_patchable.
#[test]
fn counter_commands_are_patchable() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/counter_commands_are_patchable.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>set foo. add foo. step foo. 6.
ref foo, 7.
pages roman, i.</p></para>"##,
  )]);
}

/// 58a (58a review finding 2): `\stepcounter` adds with `\addtocounter{#1}\@ne` and resets each
/// counter within by stepping it from -1, and `\refstepcounter` steps with `\stepcounter`
/// (latex.ltx:10132-10137, :14956-14967), so the patches see every step; calc's `\stepcounter`
/// advances without `\addtocounter` (calc.sty:64-69). The xml:id bookkeeping stays Perl's
/// (`AddToCounter` defines `\@<ctr>@ID` as `StepCounter` does). Repro
/// macro-state/counter_steps_go_through_the_patches.
#[test]
fn counter_steps_go_through_the_patches() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/counter_steps_go_through_the_patches.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>afoo abar sbar sfoo afoo abar sbar sfoo.</p></para>"##,
  )]);
  let calc = tex.replace(
    "\\usepackage{etoolbox}",
    "\\usepackage{calc}\\usepackage{etoolbox}",
  );
  assert_elements(&calc, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>sbar sfoo sbar sfoo.</p></para>"##,
  )]);
}

/// 58a (58a review finding 1): the matter commands switch the page numbering, as book.cls:284-291,
/// amsbook.cls:944-945 and llncs.cls:250-253 do (roman or Roman, then arabic). Repro
/// macro-state/matter_commands_set_the_page_numbering.
#[test]
fn matter_commands_set_the_page_numbering() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/matter_commands_set_the_page_numbering.tex"
  );
  for (class, front) in [("book", "i"), ("amsbook", "i"), ("llncs", "I")] {
    let doc = tex.replace(
      "\\documentclass{book}",
      &format!("\\documentclass{{{class}}}"),
    );
    let expected = format!(r##"<para xml:id="p1"><p>F {front}. R i. M 1.</p></para>"##);
    assert_elements(&doc, RAW, (0, 0), &[("para", "p1", expected.as_str())]);
  }
}

/// 58b (reledmac root B): `\vsplit` breaks only where tex.web's `vert_break` can (§970-974: glue
/// after a non-discardable item, a kern before glue or a box; a box after a box stands for its
/// interline glue — glue at the top or after glue, a rule and a box after a rule are no break),
/// takes the last break that fits (§974), prunes glue and kerns from the top of the remainder
/// (§968) and voids an emptied register (§977). Repro
/// boxes-groups/vsplit_breaks_only_where_tex_can.
#[test]
fn vsplit_breaks_only_where_tex_can() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/vsplit_breaks_only_where_tex_can.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>[1][2][2][2][2][3][1][2]\n[2]\n[3]\n[3]\n[3]\n[3][2]</p></para>",
  )]);
}

/// 58b (reledmac root A): a text group in a paragraph is a `horizontal` List without a width, so
/// it is measured in its line, not as an `\hsize`-wide paragraph of its own. Repro
/// boxes-groups/text_group_is_measured_in_its_line.
#[test]
fn text_group_is_measured_in_its_line() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/text_group_is_measured_in_its_line.tex"
  );
  let line = "[6.94444pt+1.94444pt]\n";
  let expected = format!(
    r##"<para xml:id="p1"><p>{}[42.94444pt+0.0pt]
[18.94444pt+12.0pt]
[6.86111pt+0.0pt]</p></para>"##,
    line.repeat(7)
  );
  assert_elements(tex, RAW, (0, 0), &[("para", "p1", expected.as_str())]);
}

/// 58b: a note in its line is its mark — not its body (tex.web §1100: the text is
/// an insertion) nor every registered tag form (hyperref + cleveref: six) nor, for a
/// `\footnotemark`, its arguments; a `\footnotetext` and a
/// `\marginpar` are nothing there (`note_size_in_line`, `out_of_line_size`). Repro
/// boxes-groups/notes_measure_as_their_mark.
#[test]
fn notes_measure_as_their_mark() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/notes_measure_as_their_mark.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>[1 note adds: 5.00002pt]</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>[2 no note: 78.44458pt]</p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><p>[3 marginpar: 6.83331pt+0.0pt]</p></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p>[4 footnotetext: 6.83331pt+0.0pt]</p></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><p>[5 footnotemark: 19.58339pt]</p></para>"##,
    ),
  ]);
}

/// 58c: a size switch sets the leading as latex.ltx's `\@setfontsize` does (`\fontsize{#2}{#3}`,
/// `\baselineskip` = #3 × `\baselinestretch`; size10.clo's leadings), and `\begin{document}` runs
/// `\normalsize` (latex.ltx:9497), so a preamble `\baselineskip` does not reach the body. Repro
/// boxes-groups/size_switches_set_the_leading.
#[test]
fn size_switches_set_the_leading() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/size_switches_set_the_leading.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[N 12.0pt] <text fontsize="90%">[S 11.0pt]</text> <text fontsize="80%">[F 9.5pt]</text>
<text fontsize="120%">[L 14.0pt]</text> [N2 12.0pt]
[R 18.0pt]</p></para>"##,
  )]);
}

/// 58c: the bindings' size switches expand as LaTeX's do (a parameterless size switch is robust,
/// `\small` is a macro for the primitive `\lx@size@small`), so appending to `\normalsize` with `\expandafter\def` or `\appto`
/// keeps the switch rather than calling itself. Repro boxes-groups/size_commands_can_be_appended_to.
#[test]
fn size_commands_can_be_appended_to() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/size_commands_can_be_appended_to.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text [D 3.0pt] [N 12.0pt]. <text fontsize="90%">And [E 4.0pt].</text> <text font="italic" fontsize="120%">L [14.0pt]</text></p></para>"##,
  )]);
  // A copy of `\normalsize` and a robust redefinition calling it (the kernel's documented
  // `\NewCommandCopy`): the size primitive is not `\normalsize␣`, the name the redefinition writes
  // to, so `\begin{document}`'s `\normalsize` does not loop (`Fatal:Timeout:PushbackLimit` before).
  let copy = r"\documentclass{article}
\NewCommandCopy\oldnormalsize\normalsize
\DeclareRobustCommand\normalsize{\oldnormalsize\setlength\abovedisplayskip{3pt}}
\begin{document}
Text [D \the\abovedisplayskip] [N \the\baselineskip].
\end{document}
";
  assert_elements(copy, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text [D 3.0pt] [N 12.0pt].</p></para>"##,
  )]);
}

/// 58c: `\linespread` and setspace's commands and environments set `\baselinestretch`, which every
/// size switch's leading is multiplied by. Repro
/// boxes-groups/line_spacing_commands_set_the_stretch.
#[test]
fn line_spacing_commands_set_the_stretch() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/line_spacing_commands_set_the_stretch.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>[N 20.00409pt] <text fontsize="90%">[S 18.33708pt]</text></p><p>[SS 12.0pt]</p><p>[SP 15.0pt]</p><p>[L 24.0pt]</p></para>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table xml:id="tab1"><p>[T 12.0pt]</p></table>"##,
    ),
  ]);
}

/// 58c: a floating environment resets its text to the body font and size (latex.ltx
/// `\@floatboxreset`, `reset_float_box`); a sub-float, a wrapfigure, an algorithm2e `[H]` and a
/// supertabular reset nothing and keep the surrounding `\small`, and a float after the supertabular
/// resets again. Repro boxes-groups/only_floats_reset_the_size.
#[test]
fn only_floats_reset_the_size() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/only_floats_reset_the_size.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "fig1",
      r##"<figure xml:id="fig1"><p>[F 12.0pt]</p></figure>"##,
    ),
    (
      "figure",
      "S0.F1.sf1",
      r##"<figure xml:id="S0.F1.sf1"><tags><tag>(a)</tag><tag role="refnum">1a</tag></tags><p><text fontsize="90%">[SF 11.0pt]</text></p></figure>"##,
    ),
    (
      "figure",
      "fig3",
      r##"<figure float="right" width="24%" xml:id="fig3"><p><text fontsize="90%">[W 11.0pt]</text></p></figure>"##,
    ),
    (
      "float",
      "algorithm1",
      r##"<float class="ltx_algorithm" xml:id="algorithm1"><tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag></tags><listing class="ltx_lst_numbers_left"><listingline>[A 12.0pt];</listingline><listingline/></listing><toccaption><tag close=" ">1</tag>y</toccaption><caption><tag close=" "><text font="bold">Algorithm 1</text></tag>y</caption></float>"##,
    ),
    (
      "float",
      "algorithm2",
      r##"<float class="ltx_algorithm" xml:id="algorithm2"><tags><tag><text font="bold">Algorithm 2</text></tag><tag role="refnum">2</tag></tags><listing class="ltx_lst_numbers_left"><listingline><text fontsize="90%">[H 11.0pt];</text></listingline><listingline/></listing><toccaption><tag close=" "><text fontsize="90%">2</text></tag><text fontsize="90%">x</text></toccaption><caption fontsize="90%"><tag close=" "><text font="bold">Algorithm 2</text></tag>x</caption></float>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table xml:id="tab1"><tabular><tr><td align="left"><text fontsize="90%">[ST 9]</text></td></tr></tabular></table>"##,
    ),
    (
      "figure",
      "fig4",
      r##"<figure xml:id="fig4"><p>[G 12.0pt]</p></figure>"##,
    ),
  ]);
}

/// 58c: an algorithm2e placement other than exactly `H` (`[!H]`, `[tbH]`) floats, so it resets the
/// size like any float. Repro boxes-groups/algorithm_mixed_h_placement_floats.
#[test]
fn algorithm_mixed_h_placement_floats() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/algorithm_mixed_h_placement_floats.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "algorithm1",
      r##"<float class="ltx_algorithm" xml:id="algorithm1"><tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag></tags><listing class="ltx_lst_numbers_left"><listingline>[X 12.0pt];</listingline><listingline/></listing><toccaption><tag close=" ">1</tag>z</toccaption><caption><tag close=" "><text font="bold">Algorithm 1</text></tag>z</caption></float>"##,
    ),
    (
      "float",
      "algorithm2",
      r##"<float class="ltx_algorithm" xml:id="algorithm2"><tags><tag><text font="bold">Algorithm 2</text></tag><tag role="refnum">2</tag></tags><listing class="ltx_lst_numbers_left"><listingline>[Y 12.0pt];</listingline><listingline/></listing><toccaption><tag close=" ">2</tag>w</toccaption><caption><tag close=" "><text font="bold">Algorithm 2</text></tag>w</caption></float>"##,
    ),
  ]);
}

/// 58d: a vertical `\kern` is vertical space (tex.web §1057-1061): it stacks in a `\vbox`,
/// `\unkern` removes it and `\lastkern` reads its height (KPE #406). Repro
/// boxes-groups/vertical_kern_is_vertical_space.
#[test]
fn vertical_kern_is_vertical_space() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/vertical_kern_is_vertical_space.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[21.83331pt,0.0pt] [U 18.83331pt] [L 3.0pt] [K 9.83331pt] [N 15.83331pt] [T 6.83331pt,15.0pt]</p></para>"##,
  )]);
}

/// 58d: a vertical `\kern` starts no paragraph and runs no `\everypar` (tex.web §1090). The two
/// vertical-mode `\hbox`es sharing one `<p>` with an en space (U+2002) between them is Perl's
/// shape, pinned for parity: pdflatex stacks them. Repro
/// boxes-groups/vertical_kern_starts_no_paragraph.
#[test]
fn vertical_kern_starts_no_paragraph() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/vertical_kern_starts_no_paragraph.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    ("para", "p1", r##"<para xml:id="p1"><p>B1 B2</p></para>"##),
    ("para", "p2", r##"<para xml:id="p2"><p>[1]</p></para>"##),
  ]);
}

/// 58d: the rule right after `\leaders` is a rule specification (tex.web §1078/§1084): an `\hrule`
/// ends no paragraph and a vertical-mode `\vrule` starts none, while a box filler's own `\hrule`
/// executes; a `\protected` or robust rule macro is the rule too (KPE #405). Repro
/// boxes-groups/leaders_rule_keeps_the_paragraph.
#[test]
fn leaders_rule_keeps_the_paragraph() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/leaders_rule_keeps_the_paragraph.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>A<text class="ltx_leader"><rule height="1px" width="100%"/></text>B</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>X<text class="ltx_leader"><inline-block vattach="bottom"><p>Y</p><rule height="1px" width="2.0pt"/></inline-block></text>Z</p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><p><text class="ltx_leader"><rule/></text>[4]</p></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p>P<text class="ltx_leader"><rule height="1px" width="100%"/></text>Q</p></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><p>R<text class="ltx_leader"><rule height="1px" width="100%"/></text>S</p></para>"##,
    ),
  ]);
}

/// 58e: a `\noexpand`-ed token has `\relax`'s meaning in `scan_box` (tex.web §358, §1084), so
/// `\leaders` reads past it to the rule. Repro expansion-primitives/box_operand_skips_noexpanded_token.
#[test]
fn box_operand_skips_noexpanded_token() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/box_operand_skips_noexpanded_token.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>A<text class="ltx_leader"><rule height="1px" width="100%"/></text>B</p></para>"##,
  )]);
}

/// 58e: a horizontal `\kern` is as wide as its dimension (tex.web §1061, `hpack` §651-656). Repro
/// boxes-groups/horizontal_kern_has_its_width.
#[test]
fn horizontal_kern_has_its_width() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/horizontal_kern_has_its_width.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[10.0pt][24.58337pt]</p></para>"##,
  )]);
}

/// 58f: the picture fonts are the fonts preload.ltx loads (KPE #407), so `\tenln\char45` is
/// line10's right arrowhead (latex.ltx:16914) in the graphic family, not a hyphen in the text
/// font (witnesses 2605.02221, 2605.25087). Repro fonts-nfss/picture_fonts_are_fonts.
#[test]
fn picture_fonts_are_fonts() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/picture_fonts_are_fonts.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>[line10][linew10][lcircle10][lcirclew10] [select font \
     line10]\n<text font=\"graphic\">\u{2192}</text></p></para>",
  )]);
}

/// 58f: a character of a raw `\font` whose family has no standard metric is measured from its
/// own TFM, scaled as tex.web §571-572 does (`Font::measuring_tfm`, `Tbox::with_tfm_slot`), not
/// from the Unicode character it maps to (witness 2605.02221). Repro
/// fonts-nfss/line_font_char_has_its_tfm_width.
#[test]
fn line_font_char_has_its_tfm_width() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/line_font_char_has_its_tfm_width.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>[10.0pt]\n[10.0pt]\n[10.0pt]</p></para>",
  )]);
}

/// 58g: without `at`, `\font` loads a font at its TFM design size times `scaled` (tex.web §568;
/// KPE #409): manfnt is 10pt, not the 1pt of a name without digits, and bbm17 17.28pt. Repro
/// fonts-nfss/raw_font_scales_by_design_size.
#[test]
fn raw_font_scales_by_design_size() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/raw_font_scales_by_design_size.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>[13.88893pt]\n[13.36934pt]</p></para>",
  )]);
}

/// 58g: a font selected by `\font` ends at the next font selection, which returns to the NFSS
/// font it replaced — its encoding (T1 here: `<<` is not OT1's `¡¡`), family and size
/// (`Font::nfss_font`), through a chain of raw fonts too — and math characters take their mathcode
/// family, not the raw text font (tex.web §1151-1155); a colour change keeps the raw font. KPE
/// #410. Repro fonts-nfss/raw_font_encoding_ends_with_its_font.
#[test]
fn raw_font_ends_at_a_font_selection() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/raw_font_encoding_ends_with_its_font.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>x-y abc <Math mode=\"inline\" tex=\"\\tenln a+b\" text=\"a + b\" \
       xml:id=\"p1.m1\"><XMath><XMApp><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok \
       font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">b</XMTok></XMApp></XMath></Math>\nabc <text \
       color=\"#FF0000\">\u{2141}</text></p></para>",
    ),
    (
      "para",
      "p2",
      "<para xml:id=\"p2\"><p>&lt;&lt; <text font=\"italic\">&lt;&lt;</text> <text \
       font=\"sansserif\">S</text><text font=\"bold\">S</text> <text \
       fontsize=\"173%\">V</text><text font=\"bold\">V</text></p></para>",
    ),
  ]);
}

/// 58g: `\font` defines its identifier as `\nullfont` before it scans the size (tex.web §1257;
/// KPE #411), so `\font\y=cmr10 \y` reads a font where it looks for "at"; a pending
/// `\afterassignment` token follows the whole `\font` (§1269; DIVERGENCES #394), not the early
/// definition. Repro
/// fonts-nfss/font_name_is_defined_before_its_size.
#[test]
fn font_name_is_defined_before_its_size() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/font_name_is_defined_before_its_size.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    ("para", "p1", "<para xml:id=\"p1\"><p>abc</p></para>"),
    (
      "para",
      "p2",
      "<para xml:id=\"p2\"><p>[AA]<text fontsize=\"120%\">Hello</text> world.</p></para>",
    ),
  ]);
}

/// 58g: the early `\nullfont` definition skips a locked name (old papers' `\font\title=cmbx12`
/// against the LaTeXML binding, whose font definition the lock drops): the binding keeps working.
/// Repro fonts-nfss/font_keeps_a_locked_name.
#[test]
fn font_keeps_a_locked_name() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/font_keeps_a_locked_name.tex");
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>Text.</p></para>",
  )]);
  assert_element(&xml, "title", &[], "<title>A Title</title>");
}

/// 58h: a caption steps its float counter through `\refstepcounter`'s current meaning
/// (`ref_step_counter_by_meaning`), so a measuring pass's local rebinding is undone with its box.
/// Repro captions-floats/caption_steps_through_refstepcounter.
#[test]
fn caption_steps_through_refstepcounter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_steps_through_refstepcounter.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T1",
      r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>One</toccaption><caption><tag close=": ">Table 1</tag>One</caption></table>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>After: 1.</p></para>"##,
    ),
  ]);
}

/// 58h: each floatrow floatbox numbers its float once (witness kaytannollista-latexia). The
/// caption, captured into floatrow's layout (58i), still sits in a nested `<figure>` panel
/// (SYNC_STATUS floatrow row). Repro
/// captions-floats/floatrow_floatbox_steps_its_counter_once.
#[test]
fn floatrow_floatbox_steps_its_counter_once() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_floatbox_steps_its_counter_once.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T1",
      r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block vattach="bottom"><para xml:id="S0.T1.p1"><p align="center" vattach="bottom">Body one</p><rule depth="0.0pt" height="0.0pt" width="100%"/></para><figure vattach="bottom" xml:id="S0.T1.fig1"><toccaption><tag close=" ">1</tag>One</toccaption><caption><tag close=": ">Table 1</tag>One</caption></figure></inline-logical-block></p></table>"##,
    ),
    (
      "table",
      "S0.T2",
      r##"<table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block vattach="bottom"><para xml:id="S0.T2.p1"><p align="center" vattach="bottom">Body two</p><rule depth="0.0pt" height="0.0pt" width="100%"/></para><figure vattach="bottom" xml:id="S0.T2.fig1"><toccaption><tag close=" ">2</tag>Two</toccaption><caption><tag close=": ">Table 2</tag>Two</caption></figure></inline-logical-block></p></table>"##,
    ),
  ]);
}

/// 58h: an equation inside a floatbox steps its counter once (`\refstepcounter{equation}`,
/// latex.ltx:15737-15738). Repro captions-floats/floatbox_steps_its_equation_once.
#[test]
fn floatbox_steps_its_equation_once() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatbox_steps_its_equation_once.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "equation",
      "S0.E1",
      r##"<equation class="ltx_centering" xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="x=1" text="x = 1" xml:id="S0.E1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></equation>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>After: eq=1.</p><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><Math mode="display" tex="y=2" text="y = 2" xml:id="S0.E2.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></equation></para>"##,
    ),
    (
      "equation",
      "S0.E2",
      r##"<equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><Math mode="display" tex="y=2" text="y = 2" xml:id="S0.E2.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></equation>"##,
    ),
  ]);
}

/// 58h: under hyperref and cleveref (a wrapper around the kernel's `\refstepcounter`) a float
/// still continues, a shared-counter theorem keeps its own type's tags, and the wrapper gets the
/// counter (witnesses 2605.17685, 2605.02998). Repro
/// captions-floats/steps_keep_their_type_and_continuation_through_a_wrapper.
#[test]
fn steps_keep_their_type_and_continuation_through_a_wrapper() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/steps_keep_their_type_and_continuation_through_a_wrapper.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F1a",
      r##"<figure inlist="lof" labels="LABEL:fb" xml:id="S0.F1a"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1</tag><tag role="creftype">fig.</tag><tag role="creftypecap">Figure</tag><tag role="creftypeplural">figs.</tag><tag role="creftypepluralcap">Figures</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>A cont</toccaption><caption><tag close=": ">Figure 1</tag>A cont</caption></figure>"##,
    ),
    (
      "theorem",
      "Thmthm2",
      r##"<theorem class="ltx_theorem_lem" inlist="thm theorem:lem" labels="LABEL:l1" xml:id="Thmthm2"><tags><tag>Lemma 2</tag><tag role="autoref">2</tag><tag role="creftype">Lemma</tag><tag role="creftypecap">Lemma</tag><tag role="refnum">2</tag><tag role="typerefnum">Lemma 2</tag></tags><title class="ltx_runin"><tag><text font="bold">Lemma 2</text></tag></title><para xml:id="Thmthm2.p1"><p><text font="italic">B</text></p></para></theorem>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Refs: <ref labelref="LABEL:fa"/> <ref labelref="LABEL:fb"/> <ref labelref="LABEL:fc"/> <ref labelref="LABEL:t1"/> <ref labelref="LABEL:l1"/> <ref labelref="LABEL:t2"/>; value 2.</p></para>"##,
    ),
  ]);
}

/// 58h: a float continues through a `\refstepcounter` copied from the kernel's (caption's
/// suppressed `\stepcounter`, caption.sty:557-568). Repro
/// captions-floats/continued_float_through_a_copied_refstepcounter.
#[test]
fn continued_float_through_a_copied_refstepcounter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/continued_float_through_a_copied_refstepcounter.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F1a",
      r##"<figure inlist="lof" labels="LABEL:fb" xml:id="S0.F1a"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>A cont</toccaption><caption><tag close=": ">Figure 1</tag>A cont</caption></figure>"##,
    ),
    (
      "figure",
      "S0.F2",
      r##"<figure inlist="lof" labels="LABEL:fc" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="autoref">Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">2</tag>B</toccaption><caption><tag close=": ">Figure 2</tag>B</caption></figure>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Refs: <ref labelref="LABEL:fa"/> <ref labelref="LABEL:fb"/> <ref labelref="LABEL:fc"/>; value 2.</p></para>"##,
    ),
  ]);
}

/// 58h: a continued float keeps its number for its first caption step only — a second caption,
/// or a `\caption` after a `\phantomcaption`, steps (caption.sty:590-599) — with the kernel's
/// `\refstepcounter` and through hyperref's wrapper. Repros
/// captions-floats/continued_float_continues_one_step_{kernel,wrapper}.
#[test]
fn continued_float_continues_one_step() {
  let kernel = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/continued_float_continues_one_step_kernel.tex"
  );
  assert_elements(kernel, RAW, (0, 0), &[
    (
      "figure",
      "S0.F2",
      r##"<figure inlist="lof" labels="LABEL:b LABEL:c" xml:id="S0.F2"><tags><tag><text fontsize="90%">Figure 2</text></tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">1</tag>B</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">B</text></caption><toccaption><tag close=" ">2</tag>C</toccaption><caption><tag close=": "><text fontsize="90%">Figure 2</text></tag><text fontsize="90%">C</text></caption></figure>"##,
    ),
    (
      "figure",
      "S0.F6",
      r##"<figure inlist="lof" labels="LABEL:h LABEL:hh" xml:id="S0.F6"><tags><tag><text fontsize="90%">Figure 6</text></tag><tag role="refnum">6</tag><tag role="typerefnum">Figure 6</tag></tags><toccaption><tag close=" ">6</tag>H</toccaption><caption><tag close=": "><text fontsize="90%">Figure 6</text></tag><text fontsize="90%">H</text></caption></figure>"##,
    ),
    (
      "figure",
      "S0.F7",
      r##"<figure inlist="lof" labels="LABEL:i" xml:id="S0.F7"><tags><tag><text fontsize="90%">Figure 7</text></tag><tag role="refnum">7</tag><tag role="typerefnum">Figure 7</tag></tags><toccaption><tag close=" ">7</tag>I</toccaption><caption><tag close=": "><text fontsize="90%">Figure 7</text></tag><text fontsize="90%">I</text></caption></figure>"##,
    ),
  ]);
  let wrapper = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/continued_float_continues_one_step_wrapper.tex"
  );
  assert_elements(wrapper, RAW, (0, 0), &[
    (
      "figure",
      "S0.F2",
      r##"<figure inlist="lof" labels="LABEL:b LABEL:c" xml:id="S0.F2"><tags><tag><text fontsize="90%">Figure 2</text></tag><tag role="autoref">Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">1</tag>B</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">B</text></caption><toccaption><tag close=" ">2</tag>C</toccaption><caption><tag close=": "><text fontsize="90%">Figure 2</text></tag><text fontsize="90%">C</text></caption></figure>"##,
    ),
    (
      "figure",
      "S0.F6",
      r##"<figure inlist="lof" labels="LABEL:h LABEL:hh" xml:id="S0.F6"><tags><tag><text fontsize="90%">Figure 6</text></tag><tag role="autoref">Figure 6</tag><tag role="refnum">6</tag><tag role="typerefnum">Figure 6</tag></tags><toccaption><tag close=" ">6</tag>H</toccaption><caption><tag close=": "><text fontsize="90%">Figure 6</text></tag><text fontsize="90%">H</text></caption></figure>"##,
    ),
    (
      "figure",
      "S0.F7",
      r##"<figure inlist="lof" labels="LABEL:i" xml:id="S0.F7"><tags><tag><text fontsize="90%">Figure 7</text></tag><tag role="autoref">Figure 7</tag><tag role="refnum">7</tag><tag role="typerefnum">Figure 7</tag></tags><toccaption><tag close=" ">7</tag>I</toccaption><caption><tag close=": "><text fontsize="90%">Figure 7</text></tag><text fontsize="90%">I</text></caption></figure>"##,
    ),
  ]);
}

/// 58i: floatrow places a `\floatfoot` where its caption box `\@floatcapt` is filled; the
/// kernel's caption goes through `\lx@setfloatcapt`, which the binding points at it (floatrow.sty:85-102,
/// :363-418). Repro captions-floats/floatrow_floatfoot_keeps_its_text.
#[test]
fn floatrow_floatfoot_keeps_its_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_floatfoot_keeps_its_text.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "table",
    "S0.T1",
    r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block vattach="bottom"><para xml:id="S0.T1.p2"><p align="center" vattach="bottom">Body</p><rule depth="0.0pt" height="0.0pt" width="100%"/></para><logical-block vattach="top"><figure vattach="bottom" xml:id="S0.T1.fig1"><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Table 1</tag>Cap</caption></figure><para xml:id="S0.T1.p1"><p vattach="bottom"><text fontsize="80%">A foot note.</text></p></para></logical-block></inline-logical-block></p></table>"##,
  )]);
}

/// 58i: a sub-float's caption stays in its panel while the floatbox captures the main caption
/// (caption.sty:648, `\caption@subtypehook`); `\caption*` is captured too. Repro
/// captions-floats/floatrow_subcaption_stays_in_its_panel.
#[test]
fn floatrow_subcaption_stays_in_its_panel() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_subcaption_stays_in_its_panel.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F1",
      r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block vattach="bottom"><logical-block vattach="bottom"><figure align="center" inlist="lof" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags><p>SubA</p><toccaption><tag close=" ">a</tag>SA</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">SA</text></caption></figure><figure align="center" inlist="lof" xml:id="S0.F1.sf2"><tags><tag><text fontsize="90%">(b)</text></tag><tag role="refnum">1b</tag></tags><p>SubB</p><toccaption><tag close=" ">b</tag>SB</toccaption><caption><tag close=" "><text fontsize="90%">(b)</text></tag><text fontsize="90%">SB</text></caption></figure></logical-block><rule depth="0.0pt" height="0.0pt" width="100%"/><logical-block vattach="top"><figure vattach="bottom" xml:id="S0.F1.fig1"><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure><para xml:id="S0.F1.p1"><p vattach="bottom"><text fontsize="80%">Main foot.</text></p></para></logical-block></inline-logical-block></p></figure>"##,
    ),
    (
      "figure",
      "fig2",
      r##"<figure xml:id="fig2"><p vattach="bottom" width="345.0pt"><inline-logical-block vattach="bottom"><para xml:id="p2"><p align="center" vattach="bottom">Body star</p><rule depth="0.0pt" height="0.0pt" width="100%"/></para><logical-block vattach="top"><figure vattach="bottom" xml:id="fig1"><caption>Starred</caption></figure><para xml:id="p1"><p vattach="bottom"><text fontsize="80%">Star foot.</text></p></para></logical-block></inline-logical-block></p></figure>"##,
    ),
    (
      "figure",
      "S0.F2",
      r##"<figure inlist="lof" xml:id="S0.F2"><tags><tag><text fontsize="90%">Figure 2</text></tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block vattach="bottom"><para xml:id="S0.F2.p2"><p align="center" vattach="bottom">Body short</p><rule depth="0.0pt" height="0.0pt" width="100%"/></para><logical-block vattach="top"><figure vattach="bottom" xml:id="S0.F2.fig1"><toccaption><tag close=" ">2</tag>Short</toccaption><caption><tag close=": "><text fontsize="90%">Figure 2</text></tag><text fontsize="90%">Long</text></caption></figure><para xml:id="S0.F2.p1"><p vattach="bottom"><text fontsize="80%">Short foot.</text></p></para></logical-block></inline-logical-block></p></figure>"##,
    ),
  ]);
}

/// 58i: a `pspicture` has its declared size (DIVERGENCES #397), so floatrow's empty-object test
/// (floatrow.sty:366-369) keeps it. Repro captions-floats/floatrow_keeps_an_object_it_measures_empty.
#[test]
fn floatrow_keeps_an_object_it_measures_empty() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_keeps_an_object_it_measures_empty.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "figure",
    "S0.F1",
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block><figure vattach="bottom" xml:id="S0.F1.fig1"><block vattach="bottom"><picture class="ltx_centering" fill="none" height="284.53pt" stroke="none" unitlength="28.45pt" width="284.53pt" xml:id="S0.F1.pic1"><line fill="none" points="0,0 196.85,196.85" stroke="black" stroke-width="0.8"/></picture></block><rule depth="0.0pt" height="0.0pt" width="100%"/><toccaption><tag close=" ">1</tag>CapX</toccaption><caption><tag close=": ">Figure 1</tag>CapX</caption></figure></inline-logical-block></p></figure>"##,
  )]);
}

/// 58i: floatrow's empty-object test gobbles an empty floatbox object and its frame; a non-empty
/// object is framed. Repro captions-floats/floatrow_drops_an_empty_object.
#[test]
fn floatrow_drops_an_empty_object() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_drops_an_empty_object.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F2",
      r##"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p align="center" vattach="bottom" width="345.0pt"><inline-logical-block><figure vattach="bottom" xml:id="S0.F2.fig1"><rule depth="0.0pt" height="0.0pt" width="100%"/><toccaption><tag close=" ">2</tag>CapB</toccaption><caption><tag close=": ">Figure 2</tag>CapB</caption></figure></inline-logical-block></p></figure>"##,
    ),
    (
      "figure",
      "S0.F3",
      r##"<figure inlist="lof" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><p vattach="bottom" width="345.0pt"><inline-logical-block><figure vattach="bottom" xml:id="S0.F3.fig1"><rule align="center" framed="rectangle" height="28.5pt" vattach="bottom" width="28.5pt"/><rule depth="0.0pt" height="0.0pt" width="100%"/><toccaption><tag close=" ">3</tag>CapC</toccaption><caption><tag close=": ">Figure 3</tag>CapC</caption></figure></inline-logical-block></p></figure>"##,
    ),
  ]);
}

/// 58i: a `pspicture` is a box of its declared size, corners in either order (DIVERGENCES #397),
/// so `\settowidth` and `\resizebox` read it. Repro boxes-groups/pspicture_has_its_declared_size.
#[test]
fn pspicture_has_its_declared_size() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/pspicture_has_its_declared_size.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>[113.81097pt]
[56.90549pt]
[56.90549pt]</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><inline-block depth="0.0pt" height="106.7pt" width="142.3pt" xscale="1.25000053628533" xtranslate="14.2pt" yscale="1.25000053628533" ytranslate="-10.7pt"><picture fill="none" height="85.36pt" origin-x="-28.45pt" origin-y="-28.45pt" stroke="none" unitlength="28.45pt" width="113.81pt" xml:id="p2.pic1"><g transform="translate(39.37,39.37)"><rect fill="none" height="39.37" stroke="black" stroke-width="0.8" width="78.74" x="0" y="0"/></g></picture></inline-block></para>"##,
    ),
  ]);
}

/// 58j: a list numbered by the counter of the list around it (`\\usecounter{enumi}` around an
/// `enumerate`) hangs on the outer item in force, expanded once — the id formatters referred to each
/// other and looped (Fatal:Timeout:PushbackLimit; Perl loops too, KPE #414). Repro
/// list-structure/numbered_list_with_a_nested_enumerate_converts.
#[test]
fn numbered_list_with_a_nested_enumerate_converts() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/numbered_list_with_a_nested_enumerate_converts.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><itemize><item xml:id="S0.I1.i1"><tags><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.p1"><p>x</p><enumerate xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.i1"><tags><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>y</p></para></item></enumerate></para></item></itemize></para>"##,
  )]);
}

/// 58j: the shared-counter loop through a list in between, after an earlier nested enumerate
/// (items keep their list's id), sibling lists and later items hanging on their own item, and
/// counter names holding a digit (KPE #414). Repro
/// list-structure/list_numbered_by_an_enclosing_counter.
#[test]
fn list_numbered_by_an_enclosing_counter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/list_numbered_by_an_enclosing_counter.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.p1"><p>p</p><enumerate xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.i1"><tags><tag>(a)</tag><tag role="refnum">1a</tag><tag role="typerefnum">item 1a</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>q</p></para></item></enumerate></para></item></enumerate><itemize><item xml:id="S0.I2.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I2.i1.p1"><p>b</p><enumerate xml:id="S0.I2.i1.I1"><item xml:id="S0.I2.i1.I1.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I2.i1.I1.i1.p1"><p>b1</p></para></item></enumerate><enumerate xml:id="S0.I2.i1.I2"><item xml:id="S0.I2.i1.I2.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I2.i1.I2.i1.p1"><p>c1</p></para></item></enumerate></para></item><item xml:id="S0.I2.i2"><tags><tag>(2)</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para xml:id="S0.I2.i2.p1"><p>d</p><enumerate xml:id="S0.I2.i2.I3"><item xml:id="S0.I2.i2.I3.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I2.i2.I3.i1.p1"><p>d1</p></para></item></enumerate></para></item></itemize></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><itemize><item xml:id="S0.I3.i1"><tags><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I3.i1.p1"><p>x</p><itemize xml:id="S0.I3.i1.I4"><item xml:id="S0.I3.i1.I4.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I3.i1.I4.i1.p1"><p>y</p><enumerate xml:id="S0.I3.i1.I4.i1.I1"><item xml:id="S0.I3.i1.I4.i1.I1.i1"><tags><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I3.i1.I4.i1.I1.i1.p1"><p>z</p></para></item></enumerate></para></item></itemize></para></item></itemize></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><itemize><item xml:id="S0.I4.i1"><tags><tag role="refnum">1</tag></tags><para xml:id="S0.I4.i1.p1"><p>s</p><enumerate xml:id="S0.I4.i1.I5"><item xml:id="S0.I4.i1.I5.i1"><tags><tag>1.</tag><tag role="refnum">1.</tag></tags><para xml:id="S0.I4.i1.I5.i1.p1"><p>t</p></para></item></enumerate></para></item></itemize></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><itemize><item xml:id="S0.I5.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I5.i1.p1"><p>x</p><itemize xml:id="S0.I5.i1.I6"><item xml:id="S0.I5.i1.I6.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I5.i1.I6.i1.p1"><p>y</p><enumerate xml:id="S0.I5.i1.I6.i1.I1"><item xml:id="S0.I5.i1.I6.i1.I1.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I5.i1.I6.i1.I1.i1.p1"><p>z</p></para></item><item xml:id="S0.I5.i1.I6.i1.I1.i2"><tags><tag>(2)</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para xml:id="S0.I5.i1.I6.i1.I1.i2.p1"><p>z2</p></para></item></enumerate></para></item><item xml:id="S0.I5.i1.I6.i2"><tags><tag>•</tag><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I5.i1.I6.i2.p1"><p>y2</p><enumerate xml:id="S0.I5.i1.I6.i2.I1"><item xml:id="S0.I5.i1.I6.i2.I1.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I5.i1.I6.i2.I1.i1.p1"><p>w</p></para></item></enumerate></para></item></itemize></para></item><item xml:id="S0.I5.i2"><tags><tag>(2)</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para xml:id="S0.I5.i2.p1"><p>x2</p></para></item></itemize></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><itemize><item xml:id="S0.I6.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I6.i1.p1"><p>b</p><enumerate xml:id="S0.I6.i1.I7"><item xml:id="S0.I6.i1.I7.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I6.i1.I7.i1.p1"><p>b1</p></para></item><item xml:id="S0.I6.i1.I7.i2"><tags><tag>(2)</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para xml:id="S0.I6.i1.I7.i2.p1"><p>b2</p></para></item><item xml:id="S0.I6.i1.I7.i3"><tags><tag>(3)</tag><tag role="refnum">3</tag><tag role="typerefnum">item 3</tag></tags><para xml:id="S0.I6.i1.I7.i3.p1"><p>b3</p></para></item></enumerate><enumerate xml:id="S0.I6.i1.I8"><item xml:id="S0.I6.i1.I8.i1"><tags><tag>(1)</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I6.i1.I8.i1.p1"><p>c1</p></para></item></enumerate></para></item><item xml:id="S0.I6.i2"><tags><tag>(2)</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para xml:id="S0.I6.i2.p1"><p>d</p></para></item></itemize></para>"##,
    ),
    (
      "para",
      "p6",
      r##"<para xml:id="p6"><enumerate xml:id="S0.I7"><item xml:id="S0.I7.i1"><tags><tag>1.</tag><tag role="refnum">1.</tag></tags><para xml:id="S0.I7.i1.p1"><p>a</p><enumerate xml:id="S0.I7.i1.I9"><item xml:id="S0.I7.i1.I9.i1"><tags><tag>(a)</tag><tag role="refnum">(a)</tag></tags><para xml:id="S0.I7.i1.I9.i1.p1"><p>b</p></para></item></enumerate></para></item><item xml:id="S0.I7.i2"><tags><tag>2.</tag><tag role="refnum">2.</tag></tags><para xml:id="S0.I7.i2.p1"><p>c</p><itemize xml:id="S0.I7.i2.I1"><item xml:id="S0.I7.i2.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I7.i2.I1.i1.p1"><p>e</p></para></item></itemize></para></item></enumerate></para>"##,
    ),
  ]);
}

/// 58k: a package that redefines `\everypar` as a macro (rlbicig.sty:62-64) still gets its paragraph
/// list run, because the paragraph hook reads the register latex.ltx:9070 allocated (`\newtoks
/// \everypar`, named by number in the para hook at :9072-9077), not `\everypar`'s current meaning.
/// Repro expansion-primitives/everypar_redefined_by_a_package_still_fires.
#[test]
fn everypar_redefined_by_a_package_still_fires() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everypar_redefined_by_a_package_still_fires.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>[EP]XHello world.</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>[EP]XSecond.</p></para>"##,
    ),
  ]);
}

/// 58k: `\let\everypar\mytoks` re-points the control sequence, not the paragraph hook, which keeps
/// running the register allocated at latex.ltx:9070. Repro
/// expansion-primitives/everypar_let_to_another_register_keeps_the_list.
#[test]
fn everypar_let_to_another_register_keeps_the_list() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everypar_let_to_another_register_keeps_the_list.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>XHello world.</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>XThen [Y].</p></para>"##,
    ),
  ]);
}

/// 58k: a package that re-points `\everypar` in the preamble at a new register chained from the
/// kernel's (arabicore.sty:123-128, babel's rlbabel.def:121-125) keeps its chain:
/// `\begin{document}`'s `\everypar{}` (latex.ltx:9498) empties the register `\everypar` names then.
/// Repro expansion-primitives/everypar_chained_in_the_preamble_survives_the_document.
#[test]
fn everypar_chained_in_the_preamble_survives_the_document() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everypar_chained_in_the_preamble_survives_the_document.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[H]XHello.</p></para>"##,
  )]);
}

/// 58k: entering math reads `\everymath`/`\everydisplay` only when they are registers, quietly (Perl
/// Stomach.pm:527-530), so a package that redefines `\everymath` as a macro does not warn at every
/// formula. Repro expansion-primitives/everymath_redefined_by_a_package_is_quiet.
#[test]
fn everymath_redefined_by_a_package_is_quiet() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everymath_redefined_by_a_package_is_quiet.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>A <Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> and <Math mode="inline" tex="y" text="y" xml:id="p1.m2"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math>.</p></para>"##,
  )]);
}

/// 58l: enotez's `\printendnotes` lists the notes. enotez fills its list only from the previous run's
/// `.aux` (enotez.sty:323-340); the binding records each note as it is made, the notes are
/// `ltx:note role="endnote"` and the list is an `ltx:TOC` of them (no dangling `enz.N` link). Repro
/// index-bib/enotez_notes_are_listed.
#[test]
fn enotez_notes_are_listed() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/enotez_notes_are_listed.tex");
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "S1.p1",
      r##"<para xml:id="S1.p1"><p>Alpha<note inlist="ent ent0" mark="1" role="endnote" xml:id="endnote1"><tags><tag><sup>1</sup></tag><tag role="autoref">1</tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>First note text.</note> and beta<note inlist="ent ent0" mark="*" role="endnote" xml:id="endnotex1"><tags><tag><sup>*</sup></tag><tag role="autoref">*</tag><tag role="refnum">*</tag><tag role="typerefnum">endnote *</tag></tags>Starred note text.</note>
gamma<note inlist="ent ent0" mark="2" role="endnote" xml:id="endnote2"><tags><tag><sup>2</sup></tag><tag role="autoref">2</tag><tag role="refnum">2</tag><tag role="typerefnum">endnote 2</tag></tags>Third note text.</note></p></para>"##,
    ),
    (
      "section",
      "Sx1",
      r##"<section xml:id="Sx1"><title>Notes</title><TOC lists="ent0" scope="global" show="refnum &gt; note"/></section>"##,
    ),
  ]);
}

/// 58l: enotez's `split=section` lists each section's notes under its own title — a starred section
/// and an appendix section included — and `reset=true` restarts the numbering (enotez.sty:884-905's
/// `\section` prepend is replaced by comparing what the headings changed).
/// Repro index-bib/enotez_split_lists_each_section.
#[test]
fn enotez_split_lists_each_section() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/enotez_split_lists_each_section.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "S1.p1",
      r##"<para xml:id="S1.p1"><p>Alpha<note inlist="ent ent1" mark="1" role="endnote" xml:id="endnote1"><tags><tag><sup>1</sup></tag><tag role="autoref">1</tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>First note text.</note> and beta<note inlist="ent ent1" mark="*" role="endnote" xml:id="endnotex1"><tags><tag><sup>*</sup></tag><tag role="autoref">*</tag><tag role="refnum">*</tag><tag role="typerefnum">endnote *</tag></tags>Starred note text.</note></p></para>"##,
    ),
    (
      "para",
      "S2.p1",
      r##"<para xml:id="S2.p1"><p>Delta<note inlist="ent ent2" mark="1" role="endnote" xml:id="endnote1a"><tags><tag><sup>1</sup></tag><tag role="autoref">1</tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>Fourth note.</note></p></para>"##,
    ),
    (
      "para",
      "Sx1.p1",
      r##"<para xml:id="Sx1.p1"><p>Epsilon<note inlist="ent ent3" mark="1" role="endnote" xml:id="endnote1b"><tags><tag><sup>1</sup></tag><tag role="autoref">1</tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>Fifth note.</note></p></para>"##,
    ),
    (
      "para",
      "A1.p1",
      r##"<para xml:id="A1.p1"><p>Zeta<note inlist="ent ent4" mark="1" role="endnote" xml:id="endnote1c"><tags><tag><sup>1</sup></tag><tag role="autoref">1</tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>Sixth note.</note></p></para>"##,
    ),
    (
      "appendix",
      "Ax1",
      r##"<appendix xml:id="Ax1"><title>Notes</title><subsection xml:id="Ax1.SSx1"><title>Notes for section 1</title><TOC lists="ent1" scope="global" show="refnum &gt; note"/></subsection><subsection xml:id="Ax1.SSx2"><title>Notes for section 2</title><TOC lists="ent2" scope="global" show="refnum &gt; note"/></subsection><subsection xml:id="Ax1.SSx3"><title>Notes for section 2</title><TOC lists="ent3" scope="global" show="refnum &gt; note"/></subsection><subsection xml:id="Ax1.SSx4"><title>Notes for section A</title><TOC lists="ent4" scope="global" show="refnum &gt; note"/></subsection></appendix>"##,
    ),
  ]);
}

/// 58l: enotez's `split=chapter` lists each chapter's notes once, whatever sections it has, and
/// `reset=true` restarts the numbering per chapter. Repro index-bib/enotez_split_by_chapter.
#[test]
fn enotez_split_by_chapter() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/enotez_split_by_chapter.tex");
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "Ch1.S1.p1",
      r##"<para xml:id="Ch1.S1.p1"><p>Alpha<note inlist="ent ent1" mark="1" role="endnote" xml:id="endnote1"><tags><tag><sup>1</sup></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>First.</note> beta<note inlist="ent ent1" mark="2" role="endnote" xml:id="endnote2"><tags><tag><sup>2</sup></tag><tag role="refnum">2</tag><tag role="typerefnum">endnote 2</tag></tags>Second.</note></p></para>"##,
    ),
    (
      "para",
      "Ch1.S2.p1",
      r##"<para xml:id="Ch1.S2.p1"><p>Gamma<note inlist="ent ent1" mark="3" role="endnote" xml:id="endnote3"><tags><tag><sup>3</sup></tag><tag role="refnum">3</tag><tag role="typerefnum">endnote 3</tag></tags>Third.</note></p></para>"##,
    ),
    (
      "chapter",
      "Chx1",
      r##"<chapter xml:id="Chx1"><title>Notes</title><section xml:id="Chx1.Sx1"><title>Notes for chapter 1</title><TOC lists="ent1" scope="global" show="refnum &gt; note"/></section><section xml:id="Chx1.Sx2"><title>Notes for chapter 2</title><TOC lists="ent2" scope="global" show="refnum &gt; note"/></section></chapter>"##,
    ),
    (
      "para",
      "Ch2.p1",
      r##"<para xml:id="Ch2.p1"><p>Delta<note inlist="ent ent2" mark="1" role="endnote" xml:id="endnote1a"><tags><tag><sup>1</sup></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>Fourth.</note></p></para>"##,
    ),
  ]);
}

/// 58l: enotez's `\endnotemark`/`\endnotetext` pair into one note, with or without an optional mark;
/// a second `\printendnotes` lists only the notes made after the first and `\printendnotes*` lists
/// them all (enotez_en.tex:167-170). Repro index-bib/enotez_marks_texts_and_repeated_lists.
#[test]
fn enotez_marks_texts_and_repeated_lists() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/enotez_marks_texts_and_repeated_lists.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Alpha<note inlist="ent ent0" mark="1" role="endnote" xml:id="endnote1"><tags><tag><sup>1</sup></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>First.</note> beta<note inlist="ent ent0" mark="†" role="endnote" xml:id="endnotex2"><tags><tag><sup>†</sup></tag><tag role="refnum">†</tag><tag role="typerefnum">endnote †</tag></tags>Dagger.</note></p></para>"##,
    ),
    (
      "section",
      "Sx1",
      r##"<section xml:id="Sx1"><title>Notes</title><TOC lists="ent0" scope="global" show="refnum &gt; note"/><para xml:id="Sx1.p1"><p>Gamma<note inlist="ent ent1" mark="2" role="endnote" xml:id="endnote2"><tags><tag><sup>2</sup></tag><tag role="refnum">2</tag><tag role="typerefnum">endnote 2</tag></tags>Second.</note></p></para></section>"##,
    ),
    (
      "section",
      "Sx2",
      r##"<section xml:id="Sx2"><title>Notes</title><TOC lists="ent1" scope="global" show="refnum &gt; note"/></section>"##,
    ),
    (
      "section",
      "Sx3",
      r##"<section xml:id="Sx3"><title>Notes</title><TOC lists="ent" scope="global" show="refnum &gt; note"/></section>"##,
    ),
  ]);
}

/// 58l: under enotez's `split=section` in a book, each chapter's starred sections start their own
/// splits and `reset=true` restarts the numbering there — the chapter stays in the split key, so two
/// chapters' notes never merge; an `\endnotetext` whose `\endnotemark` came before a heading pairs
/// with its mark. Repro index-bib/enotez_split_by_section_in_a_book.
#[test]
fn enotez_split_by_section_in_a_book() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/enotez_split_by_section_in_a_book.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "Ch1.Sx1.p1",
      r##"<para xml:id="Ch1.Sx1.p1"><p>Alpha<note inlist="ent ent1" mark="1" role="endnote" xml:id="endnote1"><tags><tag><sup>1</sup></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>First.</note></p></para>"##,
    ),
    (
      "para",
      "Ch2.Sx1.p1",
      r##"<para xml:id="Ch2.Sx1.p1"><p>Beta<note inlist="ent ent2" mark="1" role="endnote" xml:id="endnote1a"><tags><tag><sup>1</sup></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>Second.</note></p></para>"##,
    ),
    (
      "para",
      "Ch2.S1.p1",
      r##"<para xml:id="Ch2.S1.p1"><p>Gamma<note inlist="ent ent3" mark="1" role="endnote" xml:id="endnote1b"><tags><tag><sup>1</sup></tag><tag role="refnum">1</tag><tag role="typerefnum">endnote 1</tag></tags>Third.</note></p></para>"##,
    ),
    (
      "chapter",
      "Chx1",
      r##"<chapter xml:id="Chx1"><title>Notes</title><subsection xml:id="Chx1.S2.SSx1"><title>Notes for section 1.0</title><TOC lists="ent1" scope="global" show="refnum &gt; note"/></subsection><subsection xml:id="Chx1.S2.SSx2"><title>Notes for section 2.0</title><TOC lists="ent2" scope="global" show="refnum &gt; note"/></subsection><subsection xml:id="Chx1.S2.SSx3"><title>Notes for section 2.2</title><TOC lists="ent3" scope="global" show="refnum &gt; note"/></subsection></chapter>"##,
    ),
  ]);
}

/// 58m: a raw class's `\department{…}` store and its `[optional]{mandatory}` `\institute` store reach
/// the frontmatter as affiliations beside its `\institution` (bfhlayout.sty:735-738, printed only in a
/// title-page footer layer), all three: the stores are read at `\maketitle`, each adding its
/// affiliations (the kernel's `\lx@add@affiliations` replaced every queued one). Repro
/// sectioning-frontmatter/department_and_institute_stores_reach_the_frontmatter.
#[test]
fn department_and_institute_stores_reach_the_frontmatter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/department_and_institute_stores_reach_the_frontmatter.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton Muster</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Bern University</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Technik und Informatik</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Mikro- und Medizintechnik</contact></creator>",
  );
  assert_eq!(xml.matches("role=\"affiliation\"").count(), 3, "{xml}");
}

/// 58m: a raw class's store set twice keeps its last value, and one set empty keeps none, as the
/// class's `\@maketitle` would print them: the store is read once, at `\maketitle`. Repro
/// sectioning-frontmatter/store_set_twice_keeps_its_last_value.
#[test]
fn store_set_twice_keeps_its_last_value() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_set_twice_keeps_its_last_value.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton Muster</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Second</contact></creator>",
  );
  assert_eq!(xml.matches("role=\"affiliation\"").count(), 1, "{xml}");
}

/// 58m: a raw class's affiliation store set before `\author` reaches the author: stores are read
/// once, at `\maketitle` (`harvest_stores`), as the class's `\@maketitle` reads them. The class's
/// brackets around the store stay in its deposited title page (RED
/// sectioning-frontmatter/deposit_of_only_punctuation_is_dropped). Repro
/// sectioning-frontmatter/store_set_before_the_author_reaches_it.
#[test]
fn store_set_before_the_author_reaches_it() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_set_before_the_author_reaches_it.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[]Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept</contact></creator>",
  );
}

/// 58m: a raw class's affiliation store annotates every author it prints it under (bfh-ci prints
/// the institution once under the whole author list). Repro
/// sectioning-frontmatter/store_affiliations_reach_every_author.
#[test]
fn store_affiliations_reach_every_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_affiliations_reach_every_author.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton Muster</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Bern University</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before="],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cindy Example</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Bern University</contact></creator>",
  );
}

/// 58m: a raw class's store that annotates no creator is handed to the frontmatter as it is set, so
/// one set after `\maketitle` is kept (pittetd.cls:503-527 prints `\@keywords` at the end of its
/// abstract). Repro sectioning-frontmatter/store_set_after_maketitle_is_kept.
#[test]
fn store_set_after_maketitle_is_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_set_after_maketitle_is_kept.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">alpha, beta</keywords>",
  );
}

/// 58m: a raw class's editor store makes its creator after the authors' store annotations are
/// placed, so an affiliation store annotates the authors, not the editor. Repro
/// sectioning-frontmatter/editor_store_leaves_affiliations_to_the_authors.
#[test]
fn editor_store_leaves_affiliations_to_the_authors() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/editor_store_leaves_affiliations_to_the_authors.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Ed: Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Inst</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["role=\"editor\""],
    "<creator role=\"editor\"><personname>Eddie</personname></creator>",
  );
}

/// 58m: a raw class's store left at its class default reaches the frontmatter only through
/// `\maketitle`, which reads it: without one, no placeholder is handed on. Repro
/// sectioning-frontmatter/store_default_needs_a_maketitle.
#[test]
fn store_default_needs_a_maketitle() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_default_needs_a_maketitle.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text without maketitle.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton</personname></creator>",
  );
  assert!(
    !xml.contains("Placeholder") && !xml.contains("Default"),
    "{xml}"
  );
}

/// 58m: an accumulating store (`\g@addto@macro`) set after `\maketitle` hands on the value just
/// given, not the accumulated store. Repro
/// sectioning-frontmatter/accumulating_store_set_late_adds_its_value.
#[test]
fn accumulating_store_set_late_adds_its_value() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/accumulating_store_set_late_adds_its_value.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept A</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept B</contact></creator>",
  );
}

/// 58m: with no author, a speaker store's creator is made before the annotating stores are read, so
/// the affiliation annotates it. Repro sectioning-frontmatter/speaker_store_carries_the_affiliation.
#[test]
fn speaker_store_carries_the_affiliation() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/speaker_store_carries_the_affiliation.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Speaker:</p></para>"##,
    ),
    ("para", "p2", r##"<para xml:id="p2"><p>()Text.</p></para>"##),
  ]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"speaker\"><personname>Sam</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Inst</contact></creator>",
  );
}

/// 58m: superscript marks label a store's affiliations only when the authors carry marks to match;
/// else they annotate every author, marks kept. Repro
/// sectioning-frontmatter/marked_store_affiliations_without_author_marks.
#[test]
fn marked_store_affiliations_without_author_marks() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/marked_store_affiliations_without_author_marks.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup>1</sup>Dept One</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup>2</sup>Dept Two</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before="],
    "<creator before=\"  \" role=\"author\"><personname>Berta</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup>1</sup>Dept One</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup>2</sup>Dept Two</contact></creator>",
  );
}

/// 58m: a marked affiliation store matches the authors' marks without `\\maketitle` too: the stores
/// are read at the frontmatter fallback, before its queue is digested. Repro
/// sectioning-frontmatter/marked_store_affiliations_match_without_maketitle.
#[test]
fn marked_store_affiliations_match_without_maketitle() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/marked_store_affiliations_match_without_maketitle.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Anton</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept One</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before="],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Berta</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Dept Two</contact></creator>",
  );
}

/// 58m: a raw class's affiliation store in a document with no `\author` stays the class's to print:
/// an affiliation is placed only on a creator, so with none the deposited `\@maketitle` keeps the
/// store (courseoutline.cls:150, Outline.tex). Repro
/// sectioning-frontmatter/store_without_an_author_stays_the_class_s.
#[test]
fn store_without_an_author_stays_the_class_s() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_without_an_author_stays_the_class_s.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[Department of Ancient Studies]Text.</p></para>"##,
  )]);
  assert!(!xml.contains("<creator"), "{xml}");
}

/// 58n: a note in a section title or a caption is one note. The toc copy (`toctitle`,
/// `toccaption`) stands in for the moving argument LaTeX writes to the .toc/.lof untypeset
/// (latex.ltx:17351-17362, :17401-17412), so it is digested with notes, `\label`, `\index`
/// neutralized (`\lx@toc@copy@neutralize`); digested as typeset, a heading's note took two numbers
/// and a caption's appeared twice, shifting every later note (2605.15775). Repro
/// captions-floats/note_in_a_title_or_caption_steps_once.
#[test]
fn note_in_a_title_or_caption_steps_once() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/note_in_a_title_or_caption_steps_once.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "section",
    "S1",
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>One<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>In title.</note></title><para xml:id="S1.p1"><p>A<note mark="2" role="footnote" xml:id="footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>After.</note></p></para><table inlist="lot" xml:id="S1.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Table 1</tag>Cap<note mark="3" role="footnote" xml:id="footnote3"><tags><tag>3</tag><tag role="refnum">3</tag><tag role="typerefnum">footnote 3</tag></tags>In caption.</note></caption></table><para xml:id="S1.p2"><p>B<note mark="4" role="footnote" xml:id="footnote4"><tags><tag>4</tag><tag role="refnum">4</tag><tag role="typerefnum">footnote 4</tag></tags>Last.</note></p></para></section>"##,
  )]);
}

/// 58n: a `\\label` with cleveref's optional type after a caption (`\\label[algorithm]{…}`) converts:
/// the toc copy gobbles `\\label` through plain `o m` gobblers; LaTeX's ltcmd expandable `\\@gobble@om`
/// left the toc copy's frame open (2605.13648). Repro captions-floats/optional_label_after_a_caption_converts.
#[test]
fn optional_label_after_a_caption_converts() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/optional_label_after_a_caption_converts.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>See <ref labelref="LABEL:alg:a" show="creftype~refnum"/>.</p></para>"##,
    ),
    (
      "figure",
      "S0.F1",
      r##"<figure inlist="lof" labels="LABEL:f" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="creftype">fig.</tag><tag role="creftypecap">Figure</tag><tag role="creftypeplural">figs.</tag><tag role="creftypepluralcap">Figures</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Figure 1</tag>Cap</caption></figure>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>And <ref labelref="LABEL:f"/>.</p></para>"##,
    ),
  ]);
}

/// 58n: a `\footnotemark` in a heading pairs with the `\footnotetext` after it (the toc copy no
/// longer steps the counter). Repro captions-floats/footnotemark_in_a_heading_pairs_with_its_text.
#[test]
fn footnotemark_in_a_heading_pairs_with_its_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/footnotemark_in_a_heading_pairs_with_its_text.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "section",
    "S1",
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Two<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Text.</note></title><para xml:id="S1.p1"><p>A<note mark="2" role="footnote" xml:id="footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>After.</note></p></para></section>"##,
  )]);
}

/// 58p: `\today` follows the babel language. LaTeXML maps babel's German and French `.ldf` files to
/// bindings whose `\date<lang>` was an empty stub, so German and French printed the English date; the
/// bindings define them as germanb.ldf:124-130, ngermanb.ldf:107-113 and french.ldf:332-345 do.
/// babel's own warning about the 1901 `german` name is pdflatex's too. Repro
/// babel-lang/today_follows_the_babel_language.
#[test]
fn today_follows_the_babel_language() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/babel-lang/today_follows_the_babel_language.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (0, 1),
    &["The name 'german' with the 'ldf' mechanism is"],
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><p>1. Oktober 2026</p></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><p>1. Oktober 2026</p></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para xml:id="p3"><p><text xml:lang="fr">1<sup>er</sup> octobre 2026</text></p></para>"##,
      ),
    ],
  );
}

/// 58q: acmart's copyright statement is frontmatter — `\\setcopyright` selects the owner and permission
/// texts acmart prints at `\\maketitle` (acmart.cls:1956-2200, :2264-2296): the permission text is a
/// `license` pubnote, `\\copyright\\ <year>\\ <owner>` the copyright date (the binding stored the mode's
/// keyword, KPE #419). Repro singletons/acmart_copyright_statement_is_frontmatter.
#[test]
fn acmart_copyright_statement_is_frontmatter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/acmart_copyright_statement_is_frontmatter.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "date",
    &["role=\"copyright\""],
    "<date name=\"\u{a9}\" role=\"copyright\">2020 Copyright held by the owner/author(s).</date>",
  );
  assert_element(
    &xml,
    "pubnote",
    &["role=\"license\""],
    "<pubnote role=\"license\">Permission to make digital or hard copies of all or part of this\nwork for personal or classroom use is granted without fee provided\nthat copies are not made or distributed for profit or commercial\nadvantage and that copies bear this notice and the full citation on\nthe first page. Copyrights for third-party components of this work\nmust be honored. For all other uses, contact the\nowner/author(s).</pubnote>",
  );
}

/// 58q: acmart's public-domain mode prints the year without a copyright sign beside its statement
/// (acmart.cls:2289-2294). Repro singletons/acmart_public_domain_year_has_no_copyright_sign.
#[test]
fn acmart_public_domain_year_has_no_copyright_sign() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/acmart_public_domain_year_has_no_copyright_sign.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "date",
    &["role=\"copyrightyear\""],
    "<date role=\"copyrightyear\">2020.</date>",
  );
  assert_element(
    &xml,
    "pubnote",
    &["role=\"license\""],
    "<pubnote role=\"license\">This paper is authored by an employee(s) of the United States\nGovernment and is in the public domain. Non-exclusive copying or\nredistribution is allowed, provided that the article citation is\ngiven and the authors and agency are clearly identified as its\nsource. Request permissions from\nowner/author(s).</pubnote>",
  );
  assert!(!xml.contains("role=\"copyright\""), "{xml}");
}

/// 58q: acmart's `nonacm` option prints no copyright statement (acmart.cls:2265-2269). Repro
/// singletons/acmart_nonacm_prints_no_statement.
#[test]
fn acmart_nonacm_prints_no_statement() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/acmart_nonacm_prints_no_statement.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert!(
    !xml.contains("Copyright held")
      && !xml.contains("Permission to make")
      && !xml.contains("role=\"license\"")
      && !xml.contains("role=\"copyright\""),
    "{xml}"
  );
}

/// 58q: a document that renews `\\footnotetextcopyrightpermission` to nothing prints no statement, as
/// acmart prints it through that command (acmart.cls:2271). Repro
/// singletons/acmart_renewed_sink_prints_no_statement.
#[test]
fn acmart_renewed_sink_prints_no_statement() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/acmart_renewed_sink_prints_no_statement.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(&xml, "title", &[], "<title>T</title>");
  assert!(
    !xml.contains("Copyright held")
      && !xml.contains("Permission to make")
      && !xml.contains("role=\"license\"")
      && !xml.contains("role=\"copyright\""),
    "{xml}"
  );
}

/// 58q: acmart reads `nonacm` from its own options as xkeyval does (acmart.cls:111): an option given by
/// `\\PassOptionsToClass` counts, a braced value is unwrapped, a key name is case-sensitive (`NonACM` is
/// not the key) and the last setting wins. Repro singletons/acmart_options_follow_xkeyval.
#[test]
fn acmart_options_follow_xkeyval() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/acmart_options_follow_xkeyval.tex"
  );
  let class = "\\PassOptionsToClass{nonacm}{acmart}\n\\documentclass[sigconf]{acmart}";
  assert!(tex.contains(class));
  for (options, printed) in [
    (class, false),
    ("\\documentclass[sigconf, nonacm = {true}]{acmart}", false),
    ("\\documentclass[sigconf,NonACM]{acmart}", true),
    ("\\documentclass[sigconf,nonacm,nonacm=false]{acmart}", true),
    (
      "\\documentclass[sigconf,nonacm=false,nonacm]{acmart}",
      false,
    ),
  ] {
    let xml = assert_elements(&tex.replace(class, options), RAW, (0, 0), &[]);
    assert_eq!(
      (
        xml.contains("role=\"license\""),
        xml.contains("role=\"copyright\"")
      ),
      (printed, printed),
      "{options}: {xml}"
    );
  }
}

/// 58q: a class that loads acmart keeps acmart's own setters. The binding's `\\copyrightyear` has the
/// store shape of K11's table, but the reroute takes only what a raw class defined
/// (`frontmatter_stores.rs` `defined_by_a_binding`), so under `nonacm` nothing is printed and in ACM
/// mode a `none` year carries no copyright sign, as without the wrapper. Repro
/// singletons/acmart_under_a_wrapper_class_keeps_its_setters.
#[test]
fn acmart_under_a_wrapper_class_keeps_its_setters() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/acmart_under_a_wrapper_class_keeps_its_setters.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert!(
    !xml.contains("role=\"license\"") && !xml.contains("role=\"copyright"),
    "{xml}"
  );
  let acm = tex
    .replace(
      "\\LoadClass[sigconf,nonacm]{acmart}",
      "\\LoadClass[sigconf]{acmart}",
    )
    .replace("\\setcopyright{rightsretained}", "\\setcopyright{none}");
  let xml = assert_elements(&acm, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "date",
    &["role=\"copyrightyear\""],
    "<date role=\"copyrightyear\">2020.</date>",
  );
  assert!(!xml.contains("role=\"copyright\""), "{xml}");
  let renewed = tex
    .replace(
      "\\LoadClass[sigconf,nonacm]{acmart}\n",
      "\\LoadClass[sigconf,nonacm]{acmart}\n\\renewcommand\\keywords[1]{\\gdef\\@keywords{#1}}\n",
    )
    .replace("\\copyrightyear{2020}", "\\keywords{alpha, beta}");
  let xml = assert_elements(&renewed, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">alpha, beta</keywords>",
  );
}

/// 58q: a title-page setter `\\let` to an undefined control sequence is no definition, so a package
/// loaded after it logs nothing (the binding-load snapshot of K11's setters). Repro
/// singletons/setter_let_undefined_loads_cleanly.
#[test]
fn setter_let_undefined_loads_cleanly() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/setter_let_undefined_loads_cleanly.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Hello.</p></para>"##,
  )]);
}

/// 58q: a copyright year without a holder is the year alone (KPE #418: Perl's
/// `\\ifx.\\lx@copyright@holder.` never holds, ", 2020"). Repro
/// singletons/copyright_year_without_holder_is_the_year.
#[test]
fn copyright_year_without_holder_is_the_year() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/copyright_year_without_holder_is_the_year.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "date",
    &["role=\"copyright\""],
    "<date name=\"\u{a9}\" role=\"copyright\">2020</date>",
  );
}
