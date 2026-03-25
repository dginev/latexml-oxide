use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsart.sty.ltxml

  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "oneside", "twoside", "onecolumn", "twocolumn",
    "symbold", "ussrhead", "nameyear", "doublespacing", "reviewcopy",
  ].iter() {
    DeclareOption!(*option, None);
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

  ProcessOptions!();
  RequirePackage!("elsart_support");
});
