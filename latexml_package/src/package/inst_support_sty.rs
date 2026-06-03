use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: inst_support.sty.ltxml (PR #2767)
  // This bit of code supports the \inst style institution markup
  // used by several document classes and styles [aa, llncs, sv]
  // HOWEVER, the care support is now built into Engine/Base_Utility
  // so this shouldn't be needed.

  // In inst style, the \author (used once for each author)
  // gets some "labels" which are used to connect to \affiliation
  // (also one per affiliation); the affiliation with the matching label
  // is attached to the author.

  // \author[marks]{author}
  DefMacro!("\\author{}",
    "\\lx@clear@creators[role=author]\\lx@splitting{\\lx@add@author}{\\and\\And,}{#1}");
  DefMacro!("\\institute{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\\lx@splitting{\\lx@add@contact[role=affiliation,labelseq=affiliation]}{\\and\\And}{#1}");
  DefMacro!("\\inst{}", "\\lx@request@frontmatter@annotation[affiliation]{#1}");

  // \and variants — Perl L41-45
  Let!("\\at", "\\and"); // Actually this is different than \and, but...
  Let!("\\iand", "\\and");
  Let!("\\nand", "\\and");
  Let!("\\lastand", "\\and");
  Let!("\\AND", "\\and");
});
