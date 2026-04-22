use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: inst_support.sty.ltxml — 122 lines
  // Supports the \inst style institution markup used by svjour, llncs, aa classes.
  // Authors go in single \author separated by \and; institutes in \institute separated by \and.
  // \inst{n} links author to n-th institute.

  // Perl L43-44: \author[]{} — optional arg for OmniBus use, splits by \and or comma.
  // Redefines the generic \author to accept an optional prefix.
  DefMacro!("\\author[]{}", sub[(_opt, authors)] {
    let parts = split_tokens(authors, vec![T_CS!("\\and"), T_OTHER!(",")]);
    let mut out = Vec::new();
    for part in parts {
      out.push(T_CS!("\\lx@author"));
      out.push(T_BEGIN!());
      out.extend(part.unlist());
      out.push(T_END!());
    }
    out
  });

  // \inst{number} — generates institutemark + emailmark contacts — Perl L49-54.
  // Perl L53-54: \inst{} splits the arg by comma so \inst{1,2} issues two
  // \@inst{1}\@inst{2}. Prior Rust just passed the raw arg through as a
  // single \@inst, so multi-institute refs collapsed into one mark.
  DefConstructor!("\\@@@inst{}",
    "^<ltx:contact role='institutemark' _mark='#1'>#1</ltx:contact><ltx:contact role='emailmark' _mark='#1'>#1</ltx:contact>");
  DefMacro!("\\@inst{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@inst{#1}}");
  DefMacro!("\\inst{}", sub[(numbers)] {
    let parts = split_tokens(numbers, vec![T_OTHER!(",")]);
    let mut out = Vec::new();
    for part in parts {
      out.push(T_CS!("\\@inst"));
      out.push(T_BEGIN!());
      out.extend(part.unlist());
      out.push(T_END!());
    }
    out
  });

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

  // Email inside institute — Perl L73-77. name comes from \emailname,
  // mark from the current \theinst counter value. Prior Rust dropped both
  // attributes, so the post-pass that pairs emails to authors had nothing
  // to match against.
  DefMacro!("\\emailname", "E-mail");
  DefConstructor!("\\@in@inst@email{}",
    "<ltx:note role='email' name='#name' mark='#mark'>#1</ltx:note>",
    properties => sub[_args] {
      let name = stomach::digest(T_CS!("\\emailname"))
        .map(|d| d.to_string()).unwrap_or_default();
      let mark = gullet::do_expand(T_CS!("\\theinst"))
        .map(|t| t.to_string()).unwrap_or_default();
      Ok(stored_map!("name" => name, "mark" => mark))
    });

  // Institute note — Perl L80-83. mark property enables the post-pass that
  // relocates institute text into the matching ltx:creator by _mark. Also
  // flips `inPreamble` → 0 since institutes get digested inside \author
  // blocks that were otherwise in-preamble.
  DefConstructor!("\\@add@institute{}",
    "<ltx:note role='institutetext' mark='#mark'>#1</ltx:note>",
    bounded => true,
    before_digest => {
      state::assign_value("inPreamble", false, None);
    },
    properties => sub[_args] {
      let mark = gullet::do_expand(T_CS!("\\theinst"))
        .map(|t| t.to_string()).unwrap_or_default();
      Ok(stored_map!("mark" => mark))
    });

  // Note: Perl has Tag('ltx:note', afterClose => \&relocateInstitute) which
  // does DOM surgery to move institute text into the matching ltx:creator.
  // This requires deep clone and DOM manipulation not yet available in Rust.
  // The institute notes will appear as standalone ltx:note elements.
});
