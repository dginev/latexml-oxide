use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // extsizes' `extarticle` (extarticle.cls) is the standard `article` with more body sizes (8pt-20pt, extarticle.cls:51-58);
  // the sizes are page layout, every other option is `article`'s. Without a binding it fell to OmniBus, whose
  // guesses then blocked a raw class built on it (opticajnl.cls `\LoadClass{extarticle}` lost its `\journal`,
  // 2403.09007).
  // The size is kept in `\@ptsize` as points (extarticle.cls:25, 51-58; 10 by `\ExecuteOptions`, :80), which a raw class
  // built on it may test; the `article` binding sets its own `0`, so the chosen size is restored once it has loaded.
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
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  Let!("\\@ptsize", "\\lx@extsizes@ptsize");
});
