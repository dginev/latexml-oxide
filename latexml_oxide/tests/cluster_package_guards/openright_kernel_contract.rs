//! book.cls L52/L98/L119 and report.cls L52/L98-99/L117: `\newif
//! \if@openright` with true/false defaults respectively, driven by the
//! openright/openany class options. Derived classes and docs poke the
//! switch directly (toptesi.sty L329-342, amscls-doc handbooks) — the
//! sweep-12 `\if@openright` cluster. Same kernel-contract precedent as
//! `\if@mainmatter` (commit dba2a7eab0).

const TEX: &str = "\\documentclass[openright]{report}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    A[\\if@openright OR\\else OA\\fi]\n\
    \\@openrightfalse B[\\if@openright OR\\else OA\\fi]\n\
    \\end{document}\n";

#[test]
fn openright_switch_and_options_work() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "openright contract must digest:\n{stderr}"
  );
  assert!(
    xml.contains("A[OR]") && xml.contains("B[OA]"),
    "option must set the switch and the setter must flip it:\n{xml}",
  );
}
