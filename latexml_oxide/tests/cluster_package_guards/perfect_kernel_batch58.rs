//! Red/green guards for perfect-kernel phase-58 batches: the reledmac and floatrow kernel roots
//! (58a) found in the 57cp review.
use super::perfect_kernel_batch57::{RAW, assert_elements};

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

/// 58b: `\lastbox` and `\unskip` in a paragraph never reach the vertical
/// material before it (tex.web §1080, §1091, §1105: the paragraph's list is its own), so reledmac's
/// `\autopar` (`\everypar{\setbox0=\lastbox}`) keeps every paragraph (KNOWN_PERL_ERRORS #401). Repro
/// boxes-groups/lastbox_stays_in_its_paragraph.
#[test]
fn lastbox_stays_in_its_paragraph() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/lastbox_stays_in_its_paragraph.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Kept line.Next paragraph.</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>Also kept.</p><p>Inner.</p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><p>[47.28606pt]</p></para>"##,
    ),
  ]);
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
