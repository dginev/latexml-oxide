//! filehook.sty — hooks for input files
//! Perl: filehook.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // this comparison can't reliably work with latexml's subroutine
  // macro definitions, so default it to *true* to avoid needless warnings
  DefMacro!("\\filehook@cmp{}{}", "\\@firstoftwo");

  InputDefinitions!("filehook", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
