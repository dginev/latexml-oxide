//! OXIDIZED_DESIGN #161 (surpass-Perl, approved 2026-08-31): the `DefPlain`
//! parameter type must skip blanks before its required `{`, like real TeX's
//! undelimited-argument scanning (tex.web `macro_call`) and LaTeXML's own
//! `{}` reader. Perl 0.8.8 errors `Expected opening '{'` when a
//! `\lstnewenvironment{x}[1][]` body sits on the NEXT line — the standard
//! documentation style (~148 TL doc manuals; ltxdockit.sty, cnltx-example.sty).

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{listings}\n\
    \\lstnewenvironment{ltxcode}[1][]\n\
    \x20 {\\lstset{#1}}\n\
    \x20 {}\n\
    \\begin{document}\n\
    \\begin{ltxcode}\n\
    hello code\n\
    \\end{ltxcode}\n\
    \\end{document}\n";

#[test]
fn lstnewenvironment_body_on_next_line_defines_cleanly() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Expected opening '{'"),
    "DefPlain must skip the newline before the body brace:\n{stderr}",
  );
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "the definition and its use must digest cleanly:\n{stderr}",
  );
  assert!(
    xml.contains("<listing"),
    "\\begin{{ltxcode}} should produce an ltx:listing:\n{xml}",
  );
  // OXIDIZED_DESIGN #162: the body's FIRST line must survive. The
  // optional-arg probe crosses the newline after `\begin{ltxcode}` and
  // unreads the body's first char; the raw-line reader must not then
  // discard that line as "leftover of the \begin line" (Perl 0.8.8 drops
  // it — base64 `data` came back holding only the later lines).
  // "aGVsbG8gY29kZQ==" = base64("hello code").
  assert!(
    xml.contains("data=\"aGVsbG8gY29kZQ==\""),
    "the listing body (incl. first line) must survive as data:\n{xml}",
  );
}
