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
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Perl L41-45: amsfonts/amssymb/amsmath options push the package into
  // @revtex_toload; no-variants remove it. Packages are NOT loaded unless
  // explicitly requested (otherwise amsmath's `\pmatrix` would clobber the
  // plain-TeX `\pmatrix{…}` form, breaking documents like 0810.1407 whose
  // equation bodies use `\pmatrix{s\cr 0\cr}`).
  for pkg in ["amsfonts", "amssymb", "amsmath"] {
    let pkg_name = pkg;
    DeclareOption!(pkg, { RequirePackage!(pkg_name); });
    DeclareOption!(&s!("no{pkg}"), None);
  }

  // Perl L47-49: osajnl defines \ocis -> \pacs
  DeclareOption!("osajnl", {
    DefMacro!("\\ocis", "\\pacs");
  });

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("revtex4_support");
});
