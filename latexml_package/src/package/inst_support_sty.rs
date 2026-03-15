use crate::prelude::*;
LoadDefinitions!({
  // Perl: inst_support.sty.ltxml
  // Supports the \inst style institution markup used by svjour, llncs, aa classes

  // \inst{number} — generates footnote marks for author-institution linking
  // Simplified version: just generates the superscript marks
  DefConstructor!("\\@@@inst{}", "^<ltx:contact role='institutemark' _mark='#1'>#1</ltx:contact><ltx:contact role='emailmark' _mark='#1'>#1</ltx:contact>");
  DefMacro!("\\@inst{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@inst{#1}}");
  DefMacro!("\\inst{}", "\\@inst{#1}");

  Let!("\\at", "\\and");
  Let!("\\iand", "\\and");
  Let!("\\nand", "\\and");
  Let!("\\lastand", "\\and");
  Let!("\\AND", "\\and");

  NewCounter!("inst", "document");
  DefMacro!("\\@institutemark{}", "\\lx@contact{institutemark}{#1}");

  // Simplified \institute — just absorb the text
  DefMacro!("\\institute{}", "");

  DefMacro!("\\emailname", "E-mail");
});
