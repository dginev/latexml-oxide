use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: inst_support.sty.ltxml — 122 lines
  // Supports the \inst style institution markup used by svjour, llncs, aa classes.
  // Authors go in single \author separated by \and; institutes in \institute separated by \and.
  // \inst{n} links author to n-th institute.

  // \inst{number} — generates institutemark + emailmark contacts — Perl L49-54
  DefConstructor!("\\@@@inst{}",
    "^<ltx:contact role='institutemark' _mark='#1'>#1</ltx:contact><ltx:contact role='emailmark' _mark='#1'>#1</ltx:contact>");
  DefMacro!("\\@inst{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@inst{#1}}");
  DefMacro!("\\inst{}", "\\@inst{#1}");

  // \and variants — Perl L56-60
  Let!("\\at", "\\and");
  Let!("\\iand", "\\and");
  Let!("\\nand", "\\and");
  Let!("\\lastand", "\\and");
  Let!("\\AND", "\\and");

  // Institute counter and mark — Perl L46, L62
  NewCounter!("inst", "document");
  DefMacro!("\\@institutemark{}", "\\lx@contact{institutemark}{#1}");

  // \institute{...} — split by \and, each piece becomes an \@add@institute — Perl L63-70
  DefMacro!("\\institute{}",
    "\\bgroup\\setcounter{inst}{1}\\let\\and\\institute@and\\let\\iand\\institute@and\\let\\nand\\institute@and\\let\\lastand\\institute@and\\let\\at\\institute@and\\let\\email\\@in@inst@email\\@new@institute#1\\@end@institute\\egroup");
  DefMacro!("\\institute@and", "\\@end@institute\\stepcounter{inst}\\@new@institute");
  DefMacro!("\\@new@institute XUntil:\\@end@institute", "\\if.#1.\\else\\@add@institute{#1}\\fi");
  Let!("\\@end@institute", "\\relax");

  // Email inside institute — Perl L73-77
  DefMacro!("\\emailname", "E-mail");
  DefConstructor!("\\@in@inst@email{}", "<ltx:note role='email'>#1</ltx:note>");

  // Institute note — Perl L80-83
  DefConstructor!("\\@add@institute{}", "<ltx:note role='institutetext'>#1</ltx:note>",
    bounded => true);

  // Note: Perl has Tag('ltx:note', afterClose => \&relocateInstitute) which
  // does DOM surgery to move institute text into the matching ltx:creator.
  // This requires deep clone and DOM manipulation not yet available in Rust.
  // The institute notes will appear as standalone ltx:note elements.
});
