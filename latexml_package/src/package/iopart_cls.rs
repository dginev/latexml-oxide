use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: iopart.cls.ltxml

  // foreach my $option (qw()) { DeclareOption($option, undef); }
  // (empty list — no ignorable options)

  // Anything else gets passed to article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("iopart_support");
});
