//! filehook.sty — hooks for input files
//! Perl: filehook.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl filehook.sty.ltxml L18-20: this comparison can't reliably work
  // with latexml's subroutine macro definitions, so default it to *true*
  // to avoid needless warnings. Perl adds `locked => 1` — critical here
  // because the very next line (InputDefinitions) pulls in the raw
  // filehook.sty which itself defines \filehook@cmp; without the lock,
  // the raw-sty redefinition replaces our always-true stub.
  DefMacro!("\\filehook@cmp{}{}", "\\@firstoftwo", locked => true);

  InputDefinitions!("filehook", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
