//! Batch 56bz: luatex's direction primitives read their argument with
//! `scan_direction` — ONE expanded token when it is a direction primitive
//! (the internal form `\pagedir\bodydir`), else three letters (`TLT`). The
//! former `\def\pagedir#1#2#3{}` gobblers over-scanned the internal form:
//! babel.sty:1163-1165's `\AtBeginDocument{\pagedir\bodydir}` (under
//! `\bbl@engine=1`) reached into the following hook chunks and ate
//! babel.def:2262's `\DeclareTextCompositeCommand`, whose four arguments
//! then typeset as the junk paragraph `¨OT1aä` before every luatex-profile
//! babel manual's title (S2 cluster A, 98 docs; Perl's babel binding never
//! reaches this code). Pure token scanning — no texlua needed.

#[test]
fn direction_primitive_scans_one_internal_direction_token() {
  let tex = "\\documentclass{article}\n\\usepackage[english]{babel}\n\\begin{document}\nx\n\\end{document}\n";
  let (stderr, xml) = super::convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  // The body is exactly the one paragraph `x`: nothing typeset by the hooks.
  let body = xml
    .split("<para")
    .nth(1)
    .and_then(|s| s.split("</para>").next())
    .unwrap_or("");
  assert_eq!(
    body.split('>').skip(1).collect::<Vec<_>>().join(">").trim(),
    "<p>x</p>",
    "{xml}"
  );
  assert!(!xml.contains("OT1a"), "{xml}");

  // The keyword form and the internal form both absorb exactly their
  // argument: the text after each is intact (the spaces after the control
  // words `\bodydir` are the tokenizer's, gone in real TeX too), and
  // `\the\bodydir` is the direction keyword.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\textdir TLT A\\pagedir\\bodydir B\\mathdir\\the\\bodydir C[\\the\\pagedir]\n\\end{document}\n";
  let (stderr, xml) = super::convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>ABC[TLT]</p>"), "{xml}");
}
