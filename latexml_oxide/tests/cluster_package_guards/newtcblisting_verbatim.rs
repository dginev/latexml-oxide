//! Regression test: a `\newtcblisting`-defined code box captures its body
//! verbatim and CLOSES at `\end{name}` (ar5iv #504 / #569 / #570).
//!
//! The tcolorbox `listings` library's `\newtcblisting` reads its body as a code
//! listing. The raw library's body capture did not integrate with LaTeXML's
//! verbatim reader, so the listing ran past its `\end{name}` and swallowed the
//! following content — a `\section` after the box ended up nested inside
//! `<ltx:verbatim>` (`<ltx:section> isn't allowed in <ltx:verbatim>`) and the
//! document failed to close. The binding now delegates `\newtcblisting` to
//! listings' `\lstnewenvironment`, whose verbatim reader terminates correctly.

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage[most]{tcolorbox}\n\
    \\tcbuselibrary{listings}\n\
    \\newtcblisting{mycodebox}[1][]{listing only,#1}\n\
    \\begin{document}\n\
    \\section{First}\n\
    \\begin{mycodebox}\n\
    some code line\n\
    another line\n\
    \\end{mycodebox}\n\
    \\section{Second}\n\
    Text after the box.\n\
    \\end{document}\n";

#[test]
fn newtcblisting_body_is_verbatim_and_closes() {
  let (stderr, xml) = super::convert(TEX, false);
  // No malformed-nesting / unclosed errors: the box body must not swallow the
  // following section.
  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "newtcblisting box should close cleanly, stderr had errors:\n{stderr}",
  );

  // The second section and the text after the box survive OUTSIDE the listing.
  assert!(
    xml.contains("Text after the box"),
    "content after the box was swallowed by the listing:\n{xml}",
  );
  // Two real sections are present (the second didn't get eaten).
  assert_eq!(
    xml.matches("<section").count(),
    2,
    "expected 2 sections (First, Second) outside the box:\n{xml}",
  );
}
