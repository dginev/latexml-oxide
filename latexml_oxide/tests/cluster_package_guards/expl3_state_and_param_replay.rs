//! Batch-12 engine contracts (misdefined:# cluster, agent-bisected):
//! (1) a nested raw load must not leave the half-ExplSyntax state
//! space=ignored while `_` is not a letter (l3backend under ctex/jlreq
//! did; pgfkeys' space-delimited `\def\: {…}` idiom then broke corpus-
//! wide); (2) `\ProvidesExplFile` turns expl3 syntax ON (expl3.sty
//! L33-47) — the siunitx binding stubbed it to nothing (Perl-origin);
//! (3) `\@ifnextchar` re-scans its branches as macro bodies, collapsing
//! `##`→`#` (latex.ltx L1756-1760; adtreesdoc witness, Perl shares).

fn convert(tex: &str) -> (String, String) { super::convert(tex, true) }

#[test]
fn ifnextchar_collapses_doubled_params() {
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\makeatletter\n\
       \\def\\outerm#1{\\@ifnextchar[{X#1Y}{X#1Y}}\n\\makeatother\n\
       \\begin{document}\n\\outerm{\\def\\inner##1{[##1]}}\\inner{A}\n\\end{document}\n",
  );
  assert!(
    !stderr.contains("misdefined:#"),
    "PARAM leaked through \\@ifnextchar branch replay:\n{stderr}"
  );
  assert!(
    xml.contains("XY[A]"),
    "\\inner must receive its argument after ## collapse:\n{xml}"
  );
}

#[test]
fn provides_expl_file_turns_syntax_on() {
  let workdir_tex = "\\documentclass{article}\n\\usepackage{siunitx}\n\
      \\usepackage{numerica}\n\\begin{document}x\\end{document}\n";
  let (stderr, _) = convert(workdir_tex);
  assert!(
    !stderr.contains("Script _ can only appear in math mode"),
    "expl3 file read with LaTeX catcodes after siunitx (ProvidesExplFile stub):\n{stderr}"
  );
}
