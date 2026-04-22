use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl pgfplots.sty.ltxml — port handles the same InputDefinitions
  // flow that pulls in the raw pgfplots.sty on top of our pgf-latexml
  // shim. Perl L24 marks `\pgfplots@iffileexists` `locked => 1` so the
  // raw-TeX load can't clobber our \IfFileExists alias; Rust now mirrors.
  //
  // Still unported (Perl L27-33): compat-mode detection + autoset to
  // `mostrecent`. That requires Expand of `\pgfk@/pgfplots/compat/*` CSes
  // from the raw-sty body — safe to omit since pgfplots defaults to a
  // usable compat level without the autoset.
  DefMacro!("\\pgfplots@iffileexists", "\\IfFileExists", locked => true);
  InputDefinitions!("pgfplots", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
