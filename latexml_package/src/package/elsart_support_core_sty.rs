//! elsart_support_core.sty — Elsevier journal article support (core)
//! Perl: elsart_support_core.sty.ltxml — 191 lines
//! Shared by elsart.cls and elsarticle.cls
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Frontmatter environment
  DefEnvironment!("{frontmatter}", "#body");

  // Author/affiliation — Perl L32-48
  DefMacro!("\\author[]{}", "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#2}}");
  DefMacro!("\\address[]{}", "\\lx@contact{address}{#2}");
  DefConstructor!("\\thanks[]{}", "<ltx:note role='thanks'>#2</ltx:note>");
  DefMacro!("\\thanksref{}", "");
  DefMacro!("\\corauth[]{}", "\\lx@contact{correspondent}{#2}");
  DefMacro!("\\corref{}", "");
  DefMacro!("\\corauthref{}", "");
  DefMacro!("\\cortext[]{}", "");
  DefMacro!("\\collab OptionalMatch:* {}", "\\author{#1}");
  Let!("\\collaboration", "\\collab");
  DefMacro!("\\tnotetext[]{}", "");
  DefMacro!("\\fntext[]{}", "");
  DefMacro!("\\tnoteref{}", "");
  DefMacro!("\\fnref{}", "");

  // Title/metadata — Perl L60-106
  DefMacro!("\\runauthor{}", "");
  DefMacro!("\\runtitle{}", "");
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");
  DefMacro!("\\ead Optional:email Semiverbatim",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{#2}}");
  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#1'>#2</ltx:contact>");
  DefMacro!("\\sep", "\\unskip,\\space");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\communicated{}", "\\@add@frontmatter{ltx:date}[role=communicated]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedicated]{#1}");
  DefMacro!("\\presented{}", "\\@add@frontmatter{ltx:date}[role=presented]{#1}");
  DefMacro!("\\articletype{}", "\\@add@frontmatter{ltx:note}[role=articletype]{#1}");
  DefMacro!("\\issue{}", "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\journal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:date}[role=publication]{#1}");
  DefMacro!("\\FullCopyrightText", "");
  DefMacro!("\\copyear{}", "\\@add@frontmatter{ltx:date}[role=copyright]{#1}");
  DefMacro!("\\copyrightholder{}", "\\@add@frontmatter{ltx:note}[role=copyrightholder]{#1}");
  Let!("\\copyrightyear", "\\copyear");
  DefMacro!("\\RUNART", "");
  DefMacro!("\\RUNDATE", "");
  DefMacro!("\\RUNJNL", "");
  DefMacro!("\\company{}", "");
  DefMacro!("\\aid{}", "");
  DefMacro!("\\ssdi{}{}", "");
  DefMacro!("\\readRCS Until:$ Until:$", "");
  DefMacro!("\\RCSdate", "");
  DefMacro!("\\RCSfile", "");
  DefMacro!("\\RCSversion", "");
  DefMacro!("\\firstpage{}", "");
  DefMacro!("\\lastpage{}", "");
  DefMacro!("\\preface", "");
  DefMacro!("\\theHaddress", "");
  DefMacro!("\\theaddress", "");
  Let!("\\ESpagenumber", "\\arabic");

  // Acknowledgements — Perl L123-125
  DefConstructor!("\\ack", "<ltx:acknowledgements>");
  DefConstructor!("\\endack", "</ltx:acknowledgements>");

  // Acknowledgements tag — Perl L125
  Tag!("ltx:acknowledgements", auto_close => true);

  // Keywords — Perl L130-153
  // keyword environment and macros with XUntil pattern
  DefMacro!("\\begin{keyword}", "\\begingroup\\@keyword");
  DefMacro!("\\end{keyword}", "\\@keyword@cut\\endgroup");
  DefMacro!("\\keyword", "\\@keyword");
  DefMacro!("\\endkeyword", "\\@keyword@cut");
  DefMacro!("\\PACS", "\\@keyword@cut\\@PACS");
  DefMacro!("\\MSC[]", "\\@keyword@cut\\@MSC{#1}");
  DefMacro!("\\JEL", "\\@keyword@cut\\@JEL");
  DefMacro!("\\UK", "\\@keyword@cut\\@UK");

  DefMacro!("\\@keyword XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\@PACS XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");
  DefMacro!("\\@MSC{} XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme={#1 MSC}]{#2}");
  DefMacro!("\\@JEL XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=JEL]{#1}");
  DefMacro!("\\@UK XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=UK]{#1}");
  DefConstructor!("\\@keyword@cut", "");

  // Document structure — Perl L158-163
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");

  // Theorems — Perl L168-175
  Let!("\\newdefinition", "\\newtheorem");
  Let!("\\newproof", "\\newtheorem");

  // Registers — Perl L180-183
  DefRegister!("\\eqnarraycolsep" => Dimension!("1pt"));
  DefRegister!("\\eqnbaselineskip" => Glue!("14pt"));
  DefRegister!("\\eqnlineskip" => Glue!("2pt"));
  DefRegister!("\\eqntopsep" => Glue!("12pt"));

  // Figures — Perl L186-191
  DefMacro!("\\printfigures{}", "");
  DefMacro!("\\printtables{}", "");
  DefMacro!("\\MARK{}", "");
  DefMacro!("\\mpfootnotemark", "");

  // Float environment
  DefEnvironment!("{esmark}",  "#body");
  DefMacro!("\\figmark{}{}", "");
  DefMacro!("\\tabmark{}{}", "");
});
