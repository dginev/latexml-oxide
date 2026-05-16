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
  // Real sn-jnl.cls loads geometry for page setup — papers commonly
  // call \\geometry{margin=2cm} without an explicit usepackage.
  // Witness 2503.06846.
  RequirePackage!("geometry");

  // sn-jnl frontmatter — gobble.
  DefMacro!("\\bmhead{}", "\\subsubsection*{#1}");
  DefMacro!("\\bmsection{}", "\\section*{#1}");
  DefMacro!("\\sectiontitle{}", "");
  DefMacro!("\\headtype{}", "");
  DefMacro!("\\extralength{}", "");
  DefMacro!("\\theHfigure{}", "");
  DefMacro!("\\theHtable{}", "");

  // Author-block — preserve author-supplied affiliation / equalcont /
  // presentaddress content as ltx:note frontmatter.
  DefMacro!("\\author*[]{}", "\\author{#2}");
  DefMacro!("\\affil[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\affil*[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\equalcont{}",
    "\\@add@frontmatter{ltx:note}[role=equal-contributors]{#1}");
  DefMacro!("\\presentaddress{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#1}");
  // Name part helpers (first-name, surname) — emit inline.
  DefMacro!("\\fnm{}", "#1");
  DefMacro!("\\sur{}", "#1");

  // Frontmatter envs
  DefEnvironment!("{abstract}", "<ltx:abstract>#body</ltx:abstract>");
  DefEnvironment!("{declarations}", "<ltx:acknowledgements name='declarations'>#body</ltx:acknowledgements>");
  DefEnvironment!("{appendices}", "<ltx:appendix>#body</ltx:appendix>");
});
