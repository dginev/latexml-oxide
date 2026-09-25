//! OXIDIZED_DESIGN #170 (surpass-Perl, user-approved 2026-08-31): text
//! accents carry the LaTeX kernel's ROBUST structure — `\u` is a plain
//! macro `\protect \u␣` with the real accent in the space-suffixed CS —
//! so `\meaning\u` starts with `macro:`, as in real LaTeX. Both Perl
//! (protected primitive) and our previous eTeX-protected macro made
//! `\meaning` start with `\protected`, and tikzmath's 4-char meaning
//! sniff (tikzlibrarymath.code.tex L22-46) then misclassified accent-CS
//! variables (`\tikzmath{\u=int(...);}`) as keywords — the 11-doc
//! `Error:latex:(tikz) Unknown function or keyword '\lx@applyaccent…'`
//! cluster (witnesses cahierprof-doc, tikz-mirror-lens, colorblind_doc,
//! sunpath.track). The accent must still typeset.

const TEX: &str = "\\documentclass{article}\n\
    \\begin{document}\n\
    M[\\meaning\\u]\n\
    A[\\u{o}]\n\
    \\end{document}\n";

#[test]
fn accent_meaning_is_kernel_robust() {
  let (_stderr, xml) = super::convert(TEX, false);
  // Typeset \meaning output font-decodes `\`/`>` via OT1 (“/-¿ glyphs;
  // wisdom_ot1_angle_brackets_inverted), so assert the discriminating
  // prefix: `macro:` — NOT `\protected macro:` — is what tikzmath's
  // meaning sniff reads.
  assert!(
    xml.contains("M[macro:-") && !xml.contains("protected macro"),
    "\\meaning of a text accent must have the kernel robust shape:\n{xml}",
  );
  assert!(
    xml.contains("A[\u{014F}]") || xml.contains("A[o\u{0306}]"),
    "the accent must still typeset o-breve:\n{xml}",
  );
}
