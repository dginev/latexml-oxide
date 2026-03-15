use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4_support.sty.ltxml — support macros for RevTeX4 class

  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  // RequirePackage!("revsymb"); // not yet ported
  RequirePackage!("url");
  RequirePackage!("longtable");
  // RequirePackage!("dcolumn"); // not yet ported

  // 4.3 Title/Author
  DefMacro!("\\title[]{}", "\\@add@frontmatter{ltx:title}{#2}");
  DefMacro!("\\doauthor{}{}{}", "#1 #2 #3");
  DefMacro!("\\address", "\\affiliation");

  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\altaddress", "\\altaffiliation");
  DefMacro!("\\altaffiliation", "\\affiliation");
  DefMacro!("\\andname", "and");
  DefMacro!("\\collaboration", "");
  DefMacro!("\\noaffiliation", "");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email [] Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#2}}");
  DefConstructor!("\\@@@homepage{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\homepage Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@homepage{#1}}");

  DefMacro!("\\firstname", "");
  DefConstructor!("\\surname{}", "#1", enter_horizontal => true);

  // 4.4 Abstract
  DefMacro!("\\abstractname", "Abstract");

  // 4.5 PACS
  DefMacro!("\\pacs{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");

  // 4.6 Keywords
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // 4.7 Preprint
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");

  // Extra
  DefMacro!("\\blankaffiliation", "");
  DefMacro!("\\checkindate", "\\today");

  DefMacro!("\\received[]{}", "\\@add@frontmatter{ltx:date}[role=received]{#2}");
  DefMacro!("\\revised[]{}", "\\@add@frontmatter{ltx:date}[role=revised]{#2}");
  DefMacro!("\\accepted[]{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#2}");
  DefMacro!("\\published[]{}", "\\@add@frontmatter{ltx:date}[role=published]{#2}");

  // 5.3 Widetext
  DefMacro!("\\widetext", "");
  DefMacro!("\\endwidetext", "");
  DefMacro!("\\narrowtext", "");
  DefMacro!("\\endnarrowtext", "");
  DefMacro!("\\mediumtext", "");
  DefMacro!("\\endmediumtext", "");

  // 5.5 Acknowledgements
  Tag!("ltx:acknowledgements", auto_close => true);
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\endacknowledgements", "\\endacknowledgments");

  // Section numbering style
  DefMacro!("\\thesection", "\\Roman{section}");

  // Grid / column macros
  DefMacro!("\\thepagegrid", "one");
  DefMacro!("\\onecolumngrid", "");
  DefMacro!("\\twocolumngrid", "");
  DefMacro!("\\restorecolumngrid", "");
  DefPrimitive!("\\twocolumn", None);
  DefConstructor!("\\rotatebox{Number}{}", "#2", enter_horizontal => true);
  DefMacro!("\\pagesofar", "");

  // Endnotes
  NewCounter!("endnote");

  // Math
  Let!("\\case", "\\frac");
  Let!("\\slantfrac", "\\frac");
  DefConstructor!("\\text{}", "<ltx:text _noautoclose='true'>#1</ltx:text>",
    mode => "restricted_horizontal");

  // Citations
  DefMacro!("\\onlinecite", "\\citealp");
  Let!("\\textcite", "\\citet");

  // Tables
  DefEnvironment!("{ruledtabular}", "#body");
  DefMacro!("\\squeezetable", "");
  DefMacro!("\\toprule", "\\hline\\hline");
  DefMacro!("\\colrule", "\\hline");
  DefMacro!("\\botrule", "\\hline\\hline");
  DefMacro!("\\frstrut", "");
  DefMacro!("\\lrstrut", "");
  Let!("\\tablenote", "\\footnote");
  Let!("\\tablenotemark", "\\footnotemark");
  Let!("\\tablenotetext", "\\footnotetext");
  Let!("\\tableline", "\\colrule");

  // Floats
  DefPrimitive!("\\printfigures", None);
  DefPrimitive!("\\printtables", None);
  DefMacro!("\\oneapage", "");
  DefMacro!("\\printendnotes", "");

  // Turnpage
  DefEnvironment!("{turnpage}", "#body");

  // Extra
  DefMacro!("\\MakeTextLowercase", "\\lowercase");
  DefMacro!("\\MakeTextUppercase", "\\uppercase");
  DefMacro!("\\NoCaseChange", "");

  // Macro & control stubs
  DefMacro!("\\absbox", "");
  DefMacro!("\\addstuff{}{}", "");
  DefMacro!("\\appdef{}{}", "");
  DefMacro!("\\gappdef{}{}", "");
  DefMacro!("\\prepdef{}{}", "");
  DefMacro!("\\lineloop{}", "");
  DefMacro!("\\loopuntil{}", "");
  DefMacro!("\\loopwhile{}", "");
  DefMacro!("\\traceoutput", "");
  DefMacro!("\\tracingplain", "");
  DefMacro!("\\removephantombox", "");
  DefMacro!("\\removestuff", "");
  DefMacro!("\\replacestuff{}{}", "");

  // i18n
  DefMacro!("\\copyrightname", "??");
  DefMacro!("\\journalname", "??");
  DefMacro!("\\lofname", "List of Figures");
  DefMacro!("\\lotname", "List of Tables");
  DefMacro!("\\notesname", "Notes");
  DefMacro!("\\numbername", "number");
  DefMacro!("\\ppname", "pp");
  DefMacro!("\\tocname", "Contents");
  DefMacro!("\\volumename", "volume");

  // Document info
  DefMacro!("\\volumenumber{}", "#1");
  DefMacro!("\\volumeyear{}", "#1");
  DefMacro!("\\issuenumber{}", "#1");
  DefMacro!("\\bibinfo{}{}", "#2");
  DefMacro!("\\eprint{}", "eprint #1");
  DefMacro!("\\eid{}", "#1");

  // Extra stubs
  DefMacro!("\\flushing", "");
  DefMacro!("\\triggerpar", "\\par");
  DefMacro!("\\fullinterlineskip", "");

  DefMacro!("\\FL", "");
  DefMacro!("\\FR", "");
  DefMacro!("\\draft", "");
  DefMacro!("\\tighten", "");

  // Journal abbreviations
  DefMacro!("\\ao", "Appl.~Opt.~");
  DefMacro!("\\ap", "Appl.~Phys.~");
  DefMacro!("\\apl", "Appl.~Phys.~Lett.~");
  DefMacro!("\\apj", "Astrophys.~J.~");
  DefMacro!("\\bell", "Bell Syst.~Tech.~J.~");
  DefMacro!("\\jqe", "IEEE J.~Quantum Electron.~");
  DefMacro!("\\jcp", "J.~Chem.~Phys.~");
  DefMacro!("\\jmo", "J.~Mod.~Opt.~");
  DefMacro!("\\josa", "J.~Opt.~Soc.~Am.~");
  DefMacro!("\\josaa", "J.~Opt.~Soc.~Am.~A ");
  DefMacro!("\\josab", "J.~Opt.~Soc.~Am.~B ");
  DefMacro!("\\nat", "Nature (London) ");
  DefMacro!("\\oc", "Opt.~Commun.~");
  DefMacro!("\\ol", "Opt.~Lett.~");
  DefMacro!("\\pl", "Phys.~Lett.~");
  DefMacro!("\\pra", "Phys.~Rev.~A ");
  DefMacro!("\\prb", "Phys.~Rev.~B ");
  DefMacro!("\\prc", "Phys.~Rev.~C ");
  DefMacro!("\\prd", "Phys.~Rev.~D ");
  DefMacro!("\\pre", "Phys.~Rev.~E ");
  DefMacro!("\\prl", "Phys.~Rev.~Lett.~");
  DefMacro!("\\rmp", "Rev.~Mod.~Phys.~");

  // Internal macros
  DefMacro!("\\@revmess{}{}", "");
  DefMacro!("\\@ptsize", "0");

  // newif stubs
  TeX!(r"
  \newif\ifpreprintsty \global\preprintstyfalse
  \newif\if@amsfonts  \@amsfontsfalse
  \newif\if@amssymbols  \@amssymbolsfalse
  \newif\if@titlepage  \@titlepagefalse
  \newif\if@tightenlines \@tightenlinesfalse
  \newif\if@floats \@floatsfalse
  \newif\ifsecnumbers \global\secnumbersfalse
  ");

  DefMacro!("\\replace@command{}{}", "\\global\\let#1#2 #1");
});
