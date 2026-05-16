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

  // Defensive stubs for filehook-2020/filehook-2019 commands. The raw
  // filehook.sty selects one of these sub-files via
  // `\@ifl@t@r\fmtversion{2020/10/01}{...filehook-2020}{...filehook-2019}`.
  // Our search-paths-only find_file can't locate the versioned sub-files
  // in TL (they're not in user paths), so the load falls back to
  // filehook.sty itself — leaving the hooks undefined. Pre-define them
  // as no-ops so downstream packages (pbalance, etc.) don't crash.
  // Witnesses: 2405.18977, 2406.01136, 2406.01832 (all use pbalance).
  DefMacro!("\\AtEndOfPackageFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtBeginOfPackageFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtEndOfClassFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtBeginOfClassFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtBeginOfEveryFile{}", "");
  DefMacro!("\\AtEndOfEveryFile{}", "");
  DefMacro!("\\AtBeginOfFiles{}", "");
  DefMacro!("\\AtEndOfFiles{}", "");
  DefMacro!("\\AtBeginOfInputFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtEndOfInputFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtBeginOfIncludeFile OptionalMatch:* {}{}", "");
  DefMacro!("\\AtEndOfIncludeFile OptionalMatch:* {}{}", "");

  InputDefinitions!("filehook", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
