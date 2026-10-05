use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // extsizes' `extbook` (extbook.cls) is the standard `book` with more body sizes (8pt-20pt, extbook.cls:37-44);
  // the sizes are page layout, every other option is `book`'s. Without a binding it fell to OmniBus (see
  // `extarticle`, whose OmniBus guesses blocked opticajnl.cls's `\journal`, 2403.09007).
  // The size is kept in `\@ptsize` as points (extbook.cls:9, 37-44; 10 by `\ExecuteOptions`, :67), which a raw class
  // built on it may test; the `book` binding sets its own `0`, so the chosen size is restored once it has loaded.
  Digest!("\\def\\lx@extsizes@ptsize{10}")?;
  DeclareOption!("8pt",  "\\def\\lx@extsizes@ptsize{8}");
  DeclareOption!("9pt",  "\\def\\lx@extsizes@ptsize{9}");
  DeclareOption!("10pt", "\\def\\lx@extsizes@ptsize{10}");
  DeclareOption!("11pt", "\\def\\lx@extsizes@ptsize{11}");
  DeclareOption!("12pt", "\\def\\lx@extsizes@ptsize{12}");
  DeclareOption!("14pt", "\\def\\lx@extsizes@ptsize{14}");
  DeclareOption!("17pt", "\\def\\lx@extsizes@ptsize{17}");
  DeclareOption!("20pt", "\\def\\lx@extsizes@ptsize{20}");
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{book}")?;
  });
  ProcessOptions!();
  LoadClass!("book");
  Let!("\\@ptsize", "\\lx@extsizes@ptsize");
});
