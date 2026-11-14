//! atveryend.sty — hooks at the very end of the document
//! Perl: atveryend.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // atveryend.sty 2024-08-06 v1.12 (= base/atveryend-ltx.sty): each command adds to a kernel hook, which the
  // end-document code runs (sect02.rs `\lx@enddocument@hooks`). Perl's no-ops dropped the code (QA47B).
  DefMacro!("\\AfterLastShipout", "\\AddToHook{enddocument/afterlastpage}");
  DefMacro!("\\AtVeryEndDocument", "\\AddToHook{enddocument/afteraux}");
  DefMacro!("\\AtEndAfterFileList", "\\AddToHook{enddocument/info}");
  DefMacro!("\\AtVeryVeryEnd", "\\AddToHook{enddocument/end}");
  DefMacro!("\\BeforeClearDocument", "\\AddToHook{enddocument}");
});
