//! Two kernel contracts raw class files depend on: (1) babel assigns
//! `\language=\l@<main>` DURING package load (babel.sty L1136-1142 →
//! L828 `\bbl@patterns`), so a preamble `\iflanguage{english}{..}{..}`
//! under `[italian]` takes the FALSE branch — we never set `\language`,
//! so every non-English doc mis-branched (toptesi topfront manuals: 34
//! undefined-CS errors from a block real LaTeX skips). (2) report/book
//! define `\@endpart` (report.cls L318-327), invoked by `\@part`/`\@spart`
//! and directly by raw classes (toptesi.sty L448).

const TEX: &str = "\\documentclass{report}\n\
    \\usepackage[italian]{babel}\n\
    \\iflanguage{english}{\\def\\langprobe{EN}}{\\def\\langprobe{IT}}\n\
    \\begin{document}\n\
    [lang:\\langprobe]\n\
    \\part{Prima Parte}\n\
    testo\n\
    \\end{document}\n";

#[test]
fn babel_language_register_and_endpart() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "kernel contracts must digest cleanly:\n{stderr}"
  );
  assert!(
    xml.contains("[lang:IT]"),
    "\\iflanguage must take the non-English branch under [italian]:\n{xml}"
  );
  assert!(
    xml.contains("Prima Parte"),
    "\\part content lost (\\@endpart contract):\n{xml}"
  );
}
