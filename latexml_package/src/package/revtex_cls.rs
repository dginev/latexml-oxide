use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex.cls.ltxml
  // Ignorable options
  for option in ["manuscript", "eqsecnum", "preprint", "tighten", "floats"].iter() {
    DeclareOption!(*option, None);
  }
  // Sub-styles
  for substyle in ["aps", "osa", "aip", "pra", "prb", "prc", "prd", "prl", "rmp", "seg"].iter() {
    DeclareOption!(*substyle, None);
  }
  // Package-loading options (simplified — just declare them)
  for pkg in ["amsfonts", "amssymb", "noamsfonts", "noamssymb"].iter() {
    DeclareOption!(*pkg, None);
  }
  // Pass other options to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("natbib", options => vec![String::from("numbers")]);
  RequirePackage!("revtex3_support");
});
