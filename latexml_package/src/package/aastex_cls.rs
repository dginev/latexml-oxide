use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aastex.cls.ltxml — AAS TeX document class

  // Ignorable options
  //
  // Note on `revtex4`: Perl #2698 (2026) makes it an explicit no-op
  // because the class now loads revtex4 unconditionally anyway. Adding
  // it here prevents the option from falling through to the article
  // fallback below and getting spuriously flagged.
  for option in [
    "10pt", "11pt", "12pt",
    "manuscript", "preprint", "preprint2", "longabstract",
    "tighten", "landscape",
    "aasms4", "aaspp4", "aas2pp4", "aj_pt4", "apjpt4", "astro",
    "flushrt", "anonymous",
    "revtex4",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Number equations within sections
  DeclareOption!("eqsecnum", "\\AtEndOfClass{\\eqsecnum}");

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();

  load_class("revtex4", Vec::new(), Tokens!())?;
  RequirePackage!("aas_support");
});
