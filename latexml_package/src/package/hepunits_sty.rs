use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: hepunits.sty.ltxml — extension of siunitx with additional units
  RequirePackage!("siunitx");
  InputDefinitions!("hepunits", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
