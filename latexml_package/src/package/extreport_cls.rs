use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // extsizes' `extreport` (extreport.cls) is the standard `report` with more body sizes (8pt-20pt, extreport.cls:52-59);
  // the sizes are page layout, every other option is `report`'s. Without a binding it fell to OmniBus (see
  // `extarticle`, whose OmniBus guesses blocked opticajnl.cls's `\journal`, 2403.09007).
  // The size is kept in `\@ptsize` as points (extreport.cls:25, 52-59; 10 by `\ExecuteOptions`, :82), which a raw class
  // built on it may test; the `report` binding sets its own `0`, so the chosen size is restored once it has loaded.
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
    Digest!("\\PassOptionsToClass{\\CurrentOption}{report}")?;
  });
  ProcessOptions!();
  LoadClass!("report");
  Let!("\\@ptsize", "\\lx@extsizes@ptsize");
});
