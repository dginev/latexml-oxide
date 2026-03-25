use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsart.cls.ltxml

  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "oneside", "twoside", "onecolumn", "twocolumn",
    "symbold", "ussrhead", "nameyear", "doublespacing", "reviewcopy",
  ].iter() {
    DeclareOption!(option, None);
  }

  DeclareOption!("seceqn", {
    AssignValue!("@seceqn" => 1i64);
  });
  DeclareOption!("secthm", {
    AssignValue!("@secthm" => 1i64);
  });
  DeclareOption!("amsthm", {
    AssignValue!("@amsthm" => 1i64);
  });

  // Anything else is for article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("elsart_support");
});
