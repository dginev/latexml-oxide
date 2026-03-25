use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: blindtext.sty.ltxml
  // Note: \languagename is assumed as defined by blindtext, and it so happens that
  //       pdflatex has parts of babel defined by default. For now, just request babel loaded
  RequirePackage!("babel", options => vec!["english".to_string()]);

  InputDefinitions!("blindtext", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
