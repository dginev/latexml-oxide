//! amsldoc.cls — the AMS documentation class (amsldoc user's guides).
//!
//! Raw-load the real class, then replace `\@nobslash` with an
//! expansion-time equivalent. The raw definition
//! (`\def\@nobslash#1{\ifnum`#1=\bslchar\else#1\fi}`, amsldoc.cls L109)
//! rides inside `\index` arguments via `\string`/`\expandafter` chains
//! (L85); our `\index` SanitizedVerbatim untex→retokenize roundtrip welds
//! its catcode-12 `\` into fake CSes (`\=`, `\fi` destroyed), producing the
//! "Expected a relational token … Got \bslchar" pair and empty/garbage
//! index entries (witnesses amsldoc-it/itamsldoc, amsldoc-vn/amsldoc-vi;
//! real TeX resolves the \ifnum inside \protected@write's edef before the
//! out-of-band .idx string ever exists). Resolving the charcode-92 test at
//! EXPANSION time keeps only plain characters entering the roundtrip.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("amsldoc", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  amsdoc_patch_nobslash()?;
});

/// Shared amsldoc/amsdtx patch: `\@nobslash <tok>` expands to nothing when
/// <tok> is the (catcode-12) backslash character, else to the token itself —
/// the raw `\ifnum`#1=\bslchar` test resolved at expansion time.
pub(crate) fn amsdoc_patch_nobslash() -> Result<()> {
  DefMacro!("\\@nobslash Token", sub[(tok)] {
    let is_backslash = tok.with_str(|s| s == "\\");
    if is_backslash {
      Ok(Tokens!())
    } else {
      Ok(Tokens!(tok))
    }
  });
  Ok(())
}
