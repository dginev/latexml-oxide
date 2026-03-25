use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mn2e.cls.ltxml

  // Generally ignorable options
  for option in [
    "draft", "twocolumn", "onecolumn", "letters", "landscape", "galley",
    "referee", "doublespacing",
  ].iter() {
    DeclareOption!(*option, None);
  }

  DeclareOption!("usenatbib", {
    AssignValue!("@usenatbib" => 1i64);
  });
  DeclareOption!("usedcolum", {
    AssignValue!("@usedcolum" => 1i64);
  });
  DeclareOption!("usegraphicx", {
    AssignValue!("@usegraphicx" => 1i64);
  });
  DeclareOption!("useAMS", {
    AssignValue!("@useAMS" => 1i64);
  });

  // Anything else is for article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("mn2e_support");
});
