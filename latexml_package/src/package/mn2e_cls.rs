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
  // `usedcolumn`, which mn2e_support reads as `@usedcolumn`; Perl mn2e.cls.ltxml:24 declares `usedcolum` (KNOWN_PERL_ERRORS
  // #500). The misspelling is kept as an alias.
  for option in ["usedcolumn", "usedcolum"] {
    DeclareOption!(option, {
      AssignValue!("@usedcolumn" => 1i64);
    });
  }
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
