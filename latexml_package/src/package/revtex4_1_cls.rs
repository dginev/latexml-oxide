use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4-1.cls.ltxml
  for option in ["overload", "preprint", "manuscript", "showpacs", "noshowpacs",
    "showkeys", "noshowkeys", "balancelastpage", "nobalancelastpage",
    "preprintnumbers", "nopreprintnumbers", "bibnotes", "nobibnotes",
    "footinbib", "nofootinbib", "altaffilletter", "altaffilsymbol",
    "superbib", "citeautoscript", "raggedbottom", "flushbottom",
    "tightenlines", "lengthcheck", "eqsecnum", "secnumarabic",
    "fleqn", "floats", "endfloats", "titlepage", "notitlepage",
    "groupedaddress", "unsortedaddress", "runinaddress",
    "superscriptaddress", "byrevtex", "floatfix", "nofloatfix",
    "ltxgridinfo", "outputdebug", "raggedfooter",
    "newabstract", "oldabstract", "checkin"].iter()
  {
    DeclareOption!(*option, None);
  }
  for substyle in ["aps", "pra", "prb", "prc", "prd", "pre", "prl",
    "prstab", "rmp", "osa", "osameet", "opex", "tops", "josa"].iter()
  {
    DeclareOption!(*substyle, None);
  }
  for pkg in ["amsfonts", "amssymb", "amsmath", "noamsfonts", "noamssymb", "noamsmath"].iter() {
    DeclareOption!(*pkg, None);
  }
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("revtex4_support");
});
