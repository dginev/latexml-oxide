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
