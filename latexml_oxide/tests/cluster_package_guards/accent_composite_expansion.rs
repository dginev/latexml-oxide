//! Batch 56cq: T1's `\DeclareTextComposite` for this engine — an accent on
//! a letter with a precomposed form EXPANDS to that character
//! (`setup_binding_language::accent_composite`), so an expand-until-not-a-cs
//! loop over a passage terminates as it does under T1 pdflatex; other
//! shapes keep the `\lx@applyaccent` primitive.

/// bibleref-parse.sty:495-507 `\brp@@expandcs` re-expands a control
/// sequence with `\expandafter` until a character comes back; the inert
/// accent primitive made `\brp@parse{IK\"onige}` (the manual's German book
/// names) loop for the whole 420 s cap. Under T1 the composite ends it.
#[test]
fn expand_until_character_loop_over_an_accent_terminates() {
  if !latexml::util::test::kpse_has("bibleref-parse.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{bibleref-parse}\n\\makeatletter\n\\begin{document}\n\\def\\brp@range#1#2#3#4#5{RANGE[#1|#2]}%\n\\brp@parse{IK\\\"onige}%\n\\brp@result\n\\brp@parse{IKings}\\brp@result\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>RANGE[1Kgs|]RANGE[1Kgs|]</p></para>"##,
  );
}

/// The composite is one expansion step of the accent (`\expandafter` sees
/// the character); an empty or multi-letter group, a digit and a letter
/// with no precomposed form still take the primitive.
#[test]
fn accent_on_a_letter_expands_to_the_precomposed_character() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\expandafter\\def\\expandafter\\x\\expandafter{\\\"o}\\x \\\"o \\'e \\c{c} \\~n \\^{\\i} \\k{a} \\v{c} \\H{o} \\d{a} \\t{oo} \\\"{} \\\"{ab} \\\"5 \\b{x}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>öö é ç ñ î ą č ő ạ o͡o ¨ äb 5̈ x̱</p></para>"##,
  );
}
