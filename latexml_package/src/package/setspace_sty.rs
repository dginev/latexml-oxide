//! setspace.sty — line spacing (no-op in LaTeXML)
//! Perl: setspace.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  DefMacro!("\\singlespacing", None);
  DefMacro!("\\onehalfspacing", None);
  DefMacro!("\\doublespacing", None);
  DefMacro!("\\setstretch{}", None);
  DefMacro!("\\SetSinglespace{}", None);
  DefMacro!("\\setdisplayskipstretch{}", None);
  DefMacro!("\\restore@spacing", None);

  DefEnvironment!("{singlespace}", "#body");
  DefEnvironment!("{onehalfspace}", "#body");
  DefEnvironment!("{doublespace}", "#body");
  DefEnvironment!("{spacing}{}", "#body");
});
