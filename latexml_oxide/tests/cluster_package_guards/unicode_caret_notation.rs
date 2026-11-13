//! XeTeX/LuaTeX extended caret notation: `^^^^hhhh` (and `^^^^^^hhhhhh`)
//! produce one Unicode scalar. Packages PROBE for a Unicode engine with
//! it — newunicodechar.sty L52-56 `\edef\next{\@gobble^^^^0021}` fell
//! into its 8-bit branch without it and raised "ASCII character
//! requested" for EVERY \newunicodechar call (9-doc cluster: eigo,
//! verifica ×5, tikz-trackschematic ×2, uspace). Unicode-native-engine
//! precedent: same as providing \Ucharcat.

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{newunicodechar}\n\
    \\newunicodechar{\u{00D7}}{x}\n\
    \\begin{document}\n\
    C[^^^^0041] U[3\u{00D7}4] S[^^^^^^01d49e]\n\
    \\end{document}\n";

#[test]
fn four_and_six_caret_forms_scan() {
  let (stderr, xml) = super::convert_with(TEX, Some("[rawstyles]latexml.sty"));
  assert!(
    !stderr.contains("Error:"),
    "newunicodechar must take its Unicode branch:\n{stderr}"
  );
  assert!(
    xml.contains("C[A]") && xml.contains("U[3x4]") && xml.contains("S[\u{1D49E}]"),
    "caret forms must scan and the active-char mapping must fire:\n{xml}",
  );
}
