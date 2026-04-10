use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4.cls.ltxml — RevTeX 4 document class

  // Generally ignorable options
  for option in [
    "overload", "checkin", "preprint", "manuscript", "showpacs", "noshowpacs",
    "showkeys", "noshowkeys", "balancelastpage", "nobalancelastpage",
    "preprintnumbers", "nopreprintnumbers", "bibnotes", "nobibnotes",
    "footinbib", "nofootinbib", "altaffilletter", "altaffilsymbol",
    "superbib", "citeautoscript", "raggedbottom", "flushbottom", "tightenlines",
    "lengthcheck", "eqsecnum", "secnumarabic", "fleqn", "floats", "endfloats",
    "titlepage", "notitlepage", "groupedaddress", "unsortedaddress", "runinaddress",
    "superscriptaddress", "byrevtex", "floatfix", "nofloatfix", "ltxgridinfo",
    "outputdebug", "raggedfooter", "newabstract", "oldabstract",
    // sub-styles
    "aps", "pra", "prb", "prc", "prd", "pre", "prl", "prstab", "rmp",
    "osa", "osameet", "opex", "tops", "josa",
    // package options
    "amsfonts", "amssymb", "amsmath",
    "noamsfonts", "noamssymb", "noamsmath",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("revtex4_support");

  // Perl L58: load AMS packages after article+revtex4_support.
  // Perl tracks which ones via DeclareOption handlers, but since most revtex4
  // papers use amsmath, we load them unconditionally (Perl's default for revtex4).
  for pkg in ["amsfonts", "amssymb", "amsmath"] {
    RequirePackage!(pkg);
  }
});
