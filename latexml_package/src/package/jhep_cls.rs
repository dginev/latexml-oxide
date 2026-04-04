//! JHEP.cls — Journal of High Energy Physics document class
//! Perl: JHEP.cls.ltxml — 314 lines (mostly journal abbreviation macros)
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L26-35: Class options
  DeclareOption!("proceedings", {});
  DeclareOption!("published", {});
  DeclareOption!("hyper", {});
  DeclareOption!("nohyper", {});
  DeclareOption!("notoc", {});
  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("amssymb");

  // Perl L40-58: Frontmatter
  DefMacro!("\\speaker{}", "\\@add@frontmatter{ltx:creator}[role=speaker]{\\@personname{#1}}");
  DefConstructor!("\\@@@abstract{}", "^ <ltx:abstract name='#name'>#1</ltx:abstract>",
    properties => { stored_map!("name" => Stored::from("Abstract")) }
  );
  DefMacro!("\\abstract{}", "\\@add@to@frontmatter{ltx:abstract}{\\@@@abstract{#1}}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\JHEPcopydate{}", "\\@add@frontmatter{ltx:date}[role=copydate]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedicated]{#1}");
  DefMacro!("\\conference{}", "\\@add@frontmatter{ltx:note}[role=conference]{#1}");
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Perl L61-64: Acknowledgements environment
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='Acknowledgments'>");
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");

  // Perl L67-76: Misc macros
  DefMacro!("\\hash", "\\#");
  DefMacro!("\\secstyle", "\\bfseries");
  DefMacro!("\\militarytime", "\\time");
  Let!("\\textref", "\\ref");
  DefMacro!("\\tocsecs", "");
  DefMacro!("\\logo", "JHEP");
  DefMacro!("\\JHEP{}", "");
  DefMacro!("\\PrHEP{}", "");
  DefMacro!("\\Proof", "\\emph{Proof.}\\ ");

  // Perl L80-83: Figure/table macros (map to environments)
  DefMacro!("\\FIGURE[]{}", "#2");
  DefMacro!("\\TABLE[]{}", "#2");
  DefMacro!("\\EPSFIGURE[]{}{}", "\\begin{figure}[#1]\\epsfig{file=#2}\\caption{#3}\\end{figure}");
  DefMacro!("\\TABULAR[]{}{}{}",
    "\\begin{table}[#1]\\begin{tabular}{#2}#3\\end{tabular}\\caption{#4}\\end{table}");

  // Perl L133-137: Hyperref stubs
  DefMacro!("\\JHEPspecialurl Semiverbatim", "");
  DefMacro!("\\base Semiverbatim", "");
  DefMacro!("\\name Semiverbatim", "");

  // Perl L143: SPIRES URL generator
  DefMacro!("\\@spires{}", "\\href{http://www-spires.slac.stanford.edu/spires/find/hep/www?j=#1}");

  // Perl L295-313: Names
  DefMacro!("\\acknowlname", "Acknowledgments");
  DefMacro!("\\receivedname", "Received");
  DefMacro!("\\revisedname", "Revised");
  DefMacro!("\\acceptedname", "Accepted");
  DefMacro!("\\JHEP@todaysname", "");
  DefMacro!("\\preprintname", "PREPRINT");
  DefMacro!("\\daboraliasname", "DABO-R");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\refname", "References");
});
