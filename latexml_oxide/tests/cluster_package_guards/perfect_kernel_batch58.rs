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
