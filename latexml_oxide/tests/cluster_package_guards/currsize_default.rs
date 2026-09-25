//! OXIDIZED_DESIGN #165: `\@currsize` must be defined (default
//! `\normalsize`) — real LaTeX's begin-document invariant, which our font
//! primitives (and Perl's) never establish via `\@setfontsize`. Raw
//! packages (linguex family) call `{\@currsize …}` to restore text size.

const TEX: &str = "\\documentclass{article}\n\
    \\begin{document}\n\
    x{\\makeatletter\\@currsize\\makeatother restored}\n\
    \\end{document}\n";

#[test]
fn currsize_is_defined_and_usable() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "\\@currsize must be defined (begin-document invariant):\n{stderr}",
  );
  assert!(
    xml.contains("restored"),
    "content after \\@currsize lost:\n{xml}"
  );
}
