//! TeX's `scan_left_brace` (tex.web) uses plain `get_x_token`, so a
//! `\protected` macro EXPANDS while hunting the required `{` of a
//! <general text>; protection inhibits expansion only during body
//! absorption. Live-probed (pdflatex 2026-08-31): `\protected\def\pp
//! {{abc}}\expanded\pp` typesets abc. Our read_balanced brace hunt read
//! with fully_expand=false, erroring `Expected opening '{'` — one error
//! per `\xinttheexpr` (its `\expanded\csname XINTexprprint…` lands on a
//! \protected macro), the sweep-11 `expected:{` cluster (~16 xint docs;
//! witnesses sim-os-menus-doc, ipsum-doc, tikz-bagua-en). Same
//! argument-scanning-fidelity family as OXIDIZED_DESIGN #161.

const TEX: &str = "\\documentclass{article}\n\
    \\protected\\def\\pp{{abc}}\n\
    \\begin{document}\n\
    X\\expanded\\pp Y\n\
    \\end{document}\n";

#[test]
fn brace_hunt_expands_protected_macros() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "protected macro must expand in the brace hunt:\n{stderr}"
  );
  assert!(xml.contains("XabcY"), "expanded body must survive:\n{xml}");
}
