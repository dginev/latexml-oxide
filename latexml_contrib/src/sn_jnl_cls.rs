//! Stub for sn-jnl.cls (Springer Nature journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");

  // sn-jnl frontmatter — gobble.
  DefMacro!("\\bmhead{}", "\\subsubsection*{#1}");
  DefMacro!("\\bmsection{}", "\\section*{#1}");
  DefMacro!("\\sectiontitle{}", "");
  DefMacro!("\\headtype{}", "");
  DefMacro!("\\extralength{}", "");
  DefMacro!("\\theHfigure{}", "");
  DefMacro!("\\theHtable{}", "");

  // Author-block
  DefMacro!("\\author*[]{}", "\\author{#2}");
  DefMacro!("\\affil[]{}", "");
  DefMacro!("\\affil*[]{}", "");
  DefMacro!("\\equalcont{}", "");
  DefMacro!("\\presentaddress{}", "");
  DefMacro!("\\fnm{}", "#1");
  DefMacro!("\\sur{}", "#1");

  // Frontmatter envs
  DefEnvironment!("{abstract}", "<ltx:abstract>#body</ltx:abstract>");
  DefEnvironment!("{declarations}", "<ltx:acknowledgements name='declarations'>#body</ltx:acknowledgements>");
  DefEnvironment!("{appendices}", "<ltx:appendix>#body</ltx:appendix>");
});
